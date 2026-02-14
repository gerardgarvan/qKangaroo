//! QSession: Python-facing arena owner with frozen+Mutex pattern.
//!
//! QSession is `#[pyclass(frozen)]` so it can be shared across Python threads.
//! All arena access goes through `Arc<Mutex<SessionInner>>`, which QExpr
//! objects also hold to keep the session alive via reference counting.

use pyo3::prelude::*;
use std::sync::{Arc, Mutex};

use qsym_core::expr::Expr;
use qsym_core::number::QInt;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;

use crate::expr::QExpr;

/// Internal session state: owns the ExprArena.
///
/// NOT a pyclass -- this is the shared interior behind Arc<Mutex<>>.
pub(crate) struct SessionInner {
    pub arena: ExprArena,
}

impl SessionInner {
    /// Get or create a SymbolId for the given name.
    ///
    /// This returns SymbolId (NOT ExprRef). DSL functions need SymbolId
    /// because all qsym_core::qseries functions take `variable: SymbolId`.
    ///
    /// IMPORTANT: Do NOT confuse with arena.intern_symbol(name) which
    /// returns ExprRef. The q-series functions require SymbolId, obtained
    /// via arena.symbols_mut().intern(name).
    pub fn get_or_create_symbol_id(&mut self, name: &str) -> SymbolId {
        self.arena.symbols_mut().intern(name)
    }
}

/// A symbolic computation session owning an expression arena.
///
/// Create symbols, integers, rationals, and build expressions using
/// Python operators (+, *, -, **) on the returned QExpr objects.
///
/// ```python
/// s = QSession()
/// q = s.symbol("q")
/// a = s.symbol("a")
/// expr = q + a
/// print(repr(expr))     # Unicode rendering
/// print(expr._repr_latex_())  # LaTeX for Jupyter
/// ```
#[pyclass(frozen)]
#[derive(Clone)]
pub struct QSession {
    pub(crate) inner: Arc<Mutex<SessionInner>>,
}

#[pymethods]
impl QSession {
    /// Create a new empty session.
    #[new]
    fn new() -> Self {
        QSession {
            inner: Arc::new(Mutex::new(SessionInner {
                arena: ExprArena::new(),
            })),
        }
    }

    /// Intern a symbol by name, returning a QExpr handle.
    fn symbol(&self, name: &str) -> QExpr {
        let mut inner = self.inner.lock().unwrap();
        let expr_ref = inner.arena.intern_symbol(name);
        QExpr {
            session: Arc::clone(&self.inner),
            expr_ref,
        }
    }

    /// Intern multiple symbols from a whitespace-separated string.
    ///
    /// ```python
    /// q, a, b = s.symbols("q a b")
    /// ```
    fn symbols(&self, names: &str) -> Vec<QExpr> {
        let mut inner = self.inner.lock().unwrap();
        names
            .split_whitespace()
            .map(|name| {
                let expr_ref = inner.arena.intern_symbol(name);
                QExpr {
                    session: Arc::clone(&self.inner),
                    expr_ref,
                }
            })
            .collect()
    }

    /// Create an integer literal expression.
    fn integer(&self, val: i64) -> QExpr {
        let mut inner = self.inner.lock().unwrap();
        let expr_ref = inner.arena.intern(Expr::Integer(QInt::from(val)));
        QExpr {
            session: Arc::clone(&self.inner),
            expr_ref,
        }
    }

    /// Create a rational literal expression (num/den).
    ///
    /// Raises ValueError if denominator is zero.
    fn rational(&self, num: i64, den: i64) -> PyResult<QExpr> {
        if den == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "denominator cannot be zero",
            ));
        }
        let mut inner = self.inner.lock().unwrap();
        let expr_ref = inner.arena.intern_rat(num, den);
        Ok(QExpr {
            session: Arc::clone(&self.inner),
            expr_ref,
        })
    }

    /// Create an infinity expression.
    fn infinity(&self) -> QExpr {
        let mut inner = self.inner.lock().unwrap();
        let expr_ref = inner.arena.intern(Expr::Infinity);
        QExpr {
            session: Arc::clone(&self.inner),
            expr_ref,
        }
    }

    /// Return (arena_size, symbol_count) for diagnostics.
    fn stats(&self) -> (usize, usize) {
        let inner = self.inner.lock().unwrap();
        (inner.arena.len(), inner.arena.symbols().len())
    }
}
