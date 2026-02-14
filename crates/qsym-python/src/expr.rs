//! QExpr: Python-facing expression handle with operator overloads and rendering.
//!
//! Each QExpr holds an `Arc<Mutex<SessionInner>>` back-reference, keeping the
//! session alive as long as any expression exists. This is critical for GC safety:
//! QExpr does NOT implement Drop/dealloc that locks the session (Pitfall 2).

use pyo3::prelude::*;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use qsym_core::canonical;
use qsym_core::expr::ExprRef;
use qsym_core::render;
use qsym_core::simplify::SimplificationEngine;

use crate::session::SessionInner;

/// A handle to a symbolic expression within a QSession.
///
/// Supports Python operators (+, *, -, **) and rendering via
/// __repr__ (Unicode) and _repr_latex_ (LaTeX for Jupyter).
///
/// QExpr objects keep their session alive via Arc reference counting.
/// Creating and discarding thousands of QExpr objects is safe and
/// will not deadlock or crash.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct QExpr {
    pub(crate) session: Arc<Mutex<SessionInner>>,
    pub(crate) expr_ref: ExprRef,
}

#[pymethods]
impl QExpr {
    /// Unicode string representation for the REPL.
    fn __repr__(&self) -> String {
        let inner = self.session.lock().unwrap();
        format!("{}", inner.arena.display(self.expr_ref))
    }

    /// String representation (same as __repr__).
    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// LaTeX representation for Jupyter notebooks, wrapped in $...$.
    fn _repr_latex_(&self) -> String {
        let inner = self.session.lock().unwrap();
        let latex = render::to_latex(&inner.arena, self.expr_ref);
        format!("${}$", latex)
    }

    /// LaTeX string without dollar-sign wrappers.
    fn latex(&self) -> String {
        let inner = self.session.lock().unwrap();
        render::to_latex(&inner.arena, self.expr_ref)
    }

    /// Addition: self + other
    fn __add__(&self, other: &QExpr) -> QExpr {
        let mut inner = self.session.lock().unwrap();
        let result = canonical::make_add(&mut inner.arena, vec![self.expr_ref, other.expr_ref]);
        QExpr {
            session: Arc::clone(&self.session),
            expr_ref: result,
        }
    }

    /// Reflected addition: other + self (commutative)
    fn __radd__(&self, other: &QExpr) -> QExpr {
        self.__add__(other)
    }

    /// Multiplication: self * other
    fn __mul__(&self, other: &QExpr) -> QExpr {
        let mut inner = self.session.lock().unwrap();
        let result = canonical::make_mul(&mut inner.arena, vec![self.expr_ref, other.expr_ref]);
        QExpr {
            session: Arc::clone(&self.session),
            expr_ref: result,
        }
    }

    /// Reflected multiplication: other * self (commutative)
    fn __rmul__(&self, other: &QExpr) -> QExpr {
        self.__mul__(other)
    }

    /// Unary negation: -self
    fn __neg__(&self) -> QExpr {
        let mut inner = self.session.lock().unwrap();
        let result = canonical::make_neg(&mut inner.arena, self.expr_ref);
        QExpr {
            session: Arc::clone(&self.session),
            expr_ref: result,
        }
    }

    /// Subtraction: self - other (implemented as self + (-other))
    fn __sub__(&self, other: &QExpr) -> QExpr {
        let mut inner = self.session.lock().unwrap();
        let neg_other = canonical::make_neg(&mut inner.arena, other.expr_ref);
        let result = canonical::make_add(&mut inner.arena, vec![self.expr_ref, neg_other]);
        QExpr {
            session: Arc::clone(&self.session),
            expr_ref: result,
        }
    }

    /// Exponentiation: self ** exp
    ///
    /// The _modulo parameter is required by Python's __pow__ protocol but unused.
    fn __pow__(&self, exp: &QExpr, _modulo: Option<&Bound<'_, PyAny>>) -> QExpr {
        let mut inner = self.session.lock().unwrap();
        let result = canonical::make_pow(&mut inner.arena, self.expr_ref, exp.expr_ref);
        QExpr {
            session: Arc::clone(&self.session),
            expr_ref: result,
        }
    }

    /// Structural equality: O(1) via ExprRef comparison (hash-consing).
    fn __eq__(&self, other: &QExpr) -> bool {
        self.expr_ref == other.expr_ref
    }

    /// Hash based on the ExprRef value.
    fn __hash__(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.expr_ref.hash(&mut hasher);
        hasher.finish()
    }

    /// Apply the simplification engine to this expression.
    fn simplify(&self) -> QExpr {
        let mut inner = self.session.lock().unwrap();
        let engine = SimplificationEngine::new();
        let result = engine.simplify(self.expr_ref, &mut inner.arena);
        QExpr {
            session: Arc::clone(&self.session),
            expr_ref: result,
        }
    }

    /// Return the variant name of the underlying expression (e.g., "Symbol", "Add", "Mul").
    fn variant(&self) -> String {
        let inner = self.session.lock().unwrap();
        inner.arena.get(self.expr_ref).variant_name().to_string()
    }
}
