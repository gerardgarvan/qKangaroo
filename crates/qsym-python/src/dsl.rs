//! Python DSL functions for q-series operations.
//!
//! Each function wraps a corresponding qsym_core function, taking a QSession
//! to obtain SymbolId for the series variable, and returning QSeries results.
//!
//! CRITICAL: All functions use `session_inner.get_or_create_symbol_id("q")` to
//! get a SymbolId (NOT `arena.intern_symbol("q")` which returns ExprRef).

use pyo3::prelude::*;

use qsym_core::number::QRat;
use qsym_core::qseries::{self, QMonomial, PochhammerOrder};

use crate::series::QSeries;
use crate::session::QSession;

// ===========================================================================
// GROUP 1: Pochhammer and q-Binomial
// ===========================================================================

/// Compute the general q-Pochhammer symbol (a; q)_n as a formal power series.
///
/// The base monomial a = (coeff_num/coeff_den) * q^power.
/// If n is None, computes (a;q)_inf (infinite product).
/// If n is Some(k), computes (a;q)_k (finite product).
///
/// ```python
/// s = QSession()
/// qq = aqprod(s, 1, 1, 1, None, 20)  # (q;q)_inf to O(q^20)
/// ```
#[pyfunction]
#[pyo3(signature = (session, coeff_num, coeff_den, power, n, order))]
pub fn aqprod(
    session: &QSession,
    coeff_num: i64,
    coeff_den: i64,
    power: i64,
    n: Option<i64>,
    order: i64,
) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let monomial = QMonomial::new(QRat::from((coeff_num, coeff_den)), power);
    let poch_order = match n {
        None => PochhammerOrder::Infinite,
        Some(k) => PochhammerOrder::Finite(k),
    };
    let fps = qseries::aqprod(&monomial, sym_q, poch_order, order);
    QSeries { fps }
}

/// Compute the q-binomial (Gaussian) coefficient [n choose k]_q.
///
/// ```python
/// s = QSession()
/// b = qbin(s, 5, 2, 20)  # [5 choose 2]_q
/// ```
#[pyfunction]
pub fn qbin(session: &QSession, n: i64, k: i64, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::qbin(n, k, sym_q, order);
    QSeries { fps }
}

// ===========================================================================
// GROUP 2: Named Products
// ===========================================================================

/// Compute the generalized eta product: (q^b; q^t)_inf.
///
/// ```python
/// s = QSession()
/// e = etaq(s, 1, 1, 20)  # (q;q)_inf = Euler function
/// ```
#[pyfunction]
pub fn etaq(session: &QSession, b: i64, t: i64, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::etaq(b, t, sym_q, order);
    QSeries { fps }
}

/// Compute the Jacobi triple product JAC(a, b).
///
/// JAC(a,b) = (q^a;q^b)_inf * (q^{b-a};q^b)_inf * (q^b;q^b)_inf
///
/// Requires 0 < a < b.
#[pyfunction]
pub fn jacprod(session: &QSession, a: i64, b: i64, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::jacprod(a, b, sym_q, order);
    QSeries { fps }
}

/// Compute the Jacobi triple product with monomial parameter z = (coeff_num/coeff_den) * q^power.
#[pyfunction]
pub fn tripleprod(
    session: &QSession,
    coeff_num: i64,
    coeff_den: i64,
    power: i64,
    order: i64,
) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let monomial = QMonomial::new(QRat::from((coeff_num, coeff_den)), power);
    let fps = qseries::tripleprod(&monomial, sym_q, order);
    QSeries { fps }
}

/// Compute the quintuple product with monomial parameter z = (coeff_num/coeff_den) * q^power.
#[pyfunction]
pub fn quinprod(
    session: &QSession,
    coeff_num: i64,
    coeff_den: i64,
    power: i64,
    order: i64,
) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let monomial = QMonomial::new(QRat::from((coeff_num, coeff_den)), power);
    let fps = qseries::quinprod(&monomial, sym_q, order);
    QSeries { fps }
}

/// Compute Winquist's identity product with parameters a and b.
///
/// a = (a_cn/a_cd) * q^a_p, b = (b_cn/b_cd) * q^b_p.
#[pyfunction]
#[pyo3(signature = (session, a_cn, a_cd, a_p, b_cn, b_cd, b_p, order))]
pub fn winquist(
    session: &QSession,
    a_cn: i64,
    a_cd: i64,
    a_p: i64,
    b_cn: i64,
    b_cd: i64,
    b_p: i64,
    order: i64,
) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let a = QMonomial::new(QRat::from((a_cn, a_cd)), a_p);
    let b = QMonomial::new(QRat::from((b_cn, b_cd)), b_p);
    let fps = qseries::winquist(&a, &b, sym_q, order);
    QSeries { fps }
}

// ===========================================================================
// GROUP 3: Theta Functions
// ===========================================================================

/// Compute the Jacobi theta function theta2(q).
///
/// Returned as a series in X = q^{1/4}, where the variable represents q^{1/4}.
#[pyfunction]
pub fn theta2(session: &QSession, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::theta2(sym_q, order);
    QSeries { fps }
}

/// Compute the Jacobi theta function theta3(q).
///
/// theta3(q) = 1 + 2q + 2q^4 + 2q^9 + ... = sum_{n=-inf}^{inf} q^{n^2}
#[pyfunction]
pub fn theta3(session: &QSession, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::theta3(sym_q, order);
    QSeries { fps }
}

/// Compute the Jacobi theta function theta4(q).
///
/// theta4(q) = 1 - 2q + 2q^4 - 2q^9 + ... = sum_{n=-inf}^{inf} (-1)^n q^{n^2}
#[pyfunction]
pub fn theta4(session: &QSession, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::theta4(sym_q, order);
    QSeries { fps }
}
