//! QSession: Python-facing arena owner with frozen+Mutex pattern.
//!
//! QSession is `#[pyclass(frozen)]` so it can be shared across Python threads.
//! All arena access goes through `Arc<Mutex<SessionInner>>`, which QExpr
//! objects also hold to keep the session alive via reference counting.

use pyo3::prelude::*;
use std::sync::{Arc, Mutex};

use qsym_core::expr::Expr;
use qsym_core::number::{QInt, QRat};
use qsym_core::qseries::{self, QMonomial, PochhammerOrder};
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;

use crate::expr::QExpr;
use crate::series::QSeries;

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

    /// Generate a single q-series from a named generator function.
    ///
    /// Same func_name and params as batch_generate, but for a single computation.
    ///
    /// Supported func_name values (generator functions only):
    ///   - "aqprod": params = [coeff_num, coeff_den, power, n_or_sentinel]
    ///     where n=-1 means Infinite order, else Finite(n)
    ///   - "etaq": params = [b, t]
    ///   - "jacprod": params = [a, b]
    ///   - "tripleprod": params = [coeff_num, coeff_den, power]
    ///   - "quinprod": params = [coeff_num, coeff_den, power]
    ///   - "theta2" / "theta3" / "theta4": params = [] (no params)
    ///   - "partition_gf": params = []
    ///   - "distinct_parts_gf": params = []
    ///   - "odd_parts_gf": params = []
    ///   - "bounded_parts_gf": params = [max_parts]
    ///   - "rank_gf": params = [z_num, z_den]
    ///   - "crank_gf": params = [z_num, z_den]
    ///   - "qbin": params = [n, k]
    ///
    /// Does NOT support analysis functions (prodmake, etamake, qfactor) or
    /// relation discovery functions (findlincombo, findhom, etc.).
    /// Use the individual DSL functions directly for those.
    fn generate(
        &self,
        func_name: &str,
        params: Vec<i64>,
        truncation_order: i64,
    ) -> PyResult<QSeries> {
        let mut inner = self.inner.lock().unwrap();
        let sym_q = inner.get_or_create_symbol_id("q");
        let fps = dispatch_generator(func_name, &params, sym_q, truncation_order)?;
        Ok(QSeries { fps })
    }

    /// Batch parameter search: generate q-series over a parameter grid.
    ///
    /// Iterates over param_grid, calling the named generator function for each
    /// parameter set. The session lock is held once for the entire batch.
    ///
    /// IMPORTANT: This method supports only GENERATOR-type functions that produce
    /// FormalPowerSeries from parameters. It does NOT support:
    ///   - Analysis functions (prodmake, etamake, qfactor) -- these take a QSeries input
    ///   - Relation discovery functions (findlincombo, findhom, etc.) -- these take multiple series
    ///   - Use the individual DSL functions directly for analysis and relation discovery.
    ///
    /// See generate() for the full list of supported func_name values.
    ///
    /// Returns: list of (params, QSeries) tuples
    ///
    /// ```python
    /// s = QSession()
    /// results = s.batch_generate("etaq", [[b, 1] for b in range(1, 6)], 20)
    /// for params, series in results:
    ///     print(f"etaq(b={params[0]}, t={params[1]}): {series}")
    /// ```
    fn batch_generate(
        &self,
        func_name: &str,
        param_grid: Vec<Vec<i64>>,
        truncation_order: i64,
    ) -> PyResult<Vec<(Vec<i64>, QSeries)>> {
        let mut inner = self.inner.lock().unwrap();
        let sym_q = inner.get_or_create_symbol_id("q");

        let mut results = Vec::with_capacity(param_grid.len());
        for params in param_grid {
            let fps = dispatch_generator(func_name, &params, sym_q, truncation_order)?;
            results.push((params, QSeries { fps }));
        }

        Ok(results)
    }
}

/// Dispatch a named generator function to the corresponding qsym_core function.
///
/// This is a standalone helper (not a method) so both `generate` and `batch_generate`
/// can call it without code duplication.
fn dispatch_generator(
    func_name: &str,
    params: &[i64],
    sym_q: SymbolId,
    truncation_order: i64,
) -> PyResult<qsym_core::series::FormalPowerSeries> {
    match func_name {
        "aqprod" => {
            if params.len() < 4 {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "aqprod requires 4 params: [coeff_num, coeff_den, power, n] (n=-1 for infinite)",
                ));
            }
            let monomial = QMonomial::new(QRat::from((params[0], params[1])), params[2]);
            let poch_order = if params[3] == -1 {
                PochhammerOrder::Infinite
            } else {
                PochhammerOrder::Finite(params[3])
            };
            Ok(qseries::aqprod(&monomial, sym_q, poch_order, truncation_order))
        }
        "etaq" => {
            if params.len() < 2 {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "etaq requires 2 params: [b, t]",
                ));
            }
            Ok(qseries::etaq(params[0], params[1], sym_q, truncation_order))
        }
        "jacprod" => {
            if params.len() < 2 {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "jacprod requires 2 params: [a, b]",
                ));
            }
            Ok(qseries::jacprod(params[0], params[1], sym_q, truncation_order))
        }
        "tripleprod" => {
            if params.len() < 3 {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "tripleprod requires 3 params: [coeff_num, coeff_den, power]",
                ));
            }
            let monomial = QMonomial::new(QRat::from((params[0], params[1])), params[2]);
            Ok(qseries::tripleprod(&monomial, sym_q, truncation_order))
        }
        "quinprod" => {
            if params.len() < 3 {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "quinprod requires 3 params: [coeff_num, coeff_den, power]",
                ));
            }
            let monomial = QMonomial::new(QRat::from((params[0], params[1])), params[2]);
            Ok(qseries::quinprod(&monomial, sym_q, truncation_order))
        }
        "theta2" => Ok(qseries::theta2(sym_q, truncation_order)),
        "theta3" => Ok(qseries::theta3(sym_q, truncation_order)),
        "theta4" => Ok(qseries::theta4(sym_q, truncation_order)),
        "partition_gf" => Ok(qseries::partition_gf(sym_q, truncation_order)),
        "distinct_parts_gf" => Ok(qseries::distinct_parts_gf(sym_q, truncation_order)),
        "odd_parts_gf" => Ok(qseries::odd_parts_gf(sym_q, truncation_order)),
        "bounded_parts_gf" => {
            if params.is_empty() {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "bounded_parts_gf requires 1 param: [max_parts]",
                ));
            }
            Ok(qseries::bounded_parts_gf(params[0], sym_q, truncation_order))
        }
        "rank_gf" => {
            if params.len() < 2 {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "rank_gf requires 2 params: [z_num, z_den]",
                ));
            }
            let z = QRat::from((params[0], params[1]));
            Ok(qseries::rank_gf(&z, sym_q, truncation_order))
        }
        "crank_gf" => {
            if params.len() < 2 {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "crank_gf requires 2 params: [z_num, z_den]",
                ));
            }
            let z = QRat::from((params[0], params[1]));
            Ok(qseries::crank_gf(&z, sym_q, truncation_order))
        }
        "qbin" => {
            if params.len() < 2 {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "qbin requires 2 params: [n, k]",
                ));
            }
            Ok(qseries::qbin(params[0], params[1], sym_q, truncation_order))
        }
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown generator function '{}'. Supported: aqprod, etaq, jacprod, \
             tripleprod, quinprod, theta2, theta3, theta4, partition_gf, \
             distinct_parts_gf, odd_parts_gf, bounded_parts_gf, rank_gf, \
             crank_gf, qbin. Note: analysis functions (prodmake, qfactor) and \
             relation discovery functions (findlincombo, findhom, etc.) are not \
             supported -- use the individual DSL functions directly.",
            func_name
        ))),
    }
}
