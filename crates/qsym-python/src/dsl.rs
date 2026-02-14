//! Python DSL functions for q-series operations.
//!
//! Each function wraps a corresponding qsym_core function, taking a QSession
//! to obtain SymbolId for the series variable, and returning QSeries results.
//!
//! CRITICAL: All functions use `session_inner.get_or_create_symbol_id("q")` to
//! get a SymbolId (NOT `arena.intern_symbol("q")` which returns ExprRef).

use pyo3::prelude::*;
use pyo3::types::PyDict;

use qsym_core::number::QRat;
use qsym_core::qseries::{self, QMonomial, PochhammerOrder};

use crate::convert::{qint_to_python, qrat_to_python};
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

// ===========================================================================
// GROUP 4: Partition Functions
// ===========================================================================

/// Compute p(n), the number of partitions of n.
///
/// Returns a Python int (not Fraction) since partition counts are always integers.
/// No session needed -- pure computation on an integer.
///
/// ```python
/// print(partition_count(5))   # 7
/// print(partition_count(100)) # 190569292536040
/// ```
#[pyfunction]
pub fn partition_count(py: Python<'_>, n: i64) -> PyResult<PyObject> {
    let result = qseries::partition_count(n);
    // partition_count always returns an integer; extract numerator as QInt
    let numer = qsym_core::number::QInt(result.numer().clone());
    let obj = qint_to_python(py, &numer)?;
    Ok(obj.into())
}

/// Compute the partition generating function: sum_{n>=0} p(n) q^n = 1/(q;q)_inf.
#[pyfunction]
pub fn partition_gf(session: &QSession, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::partition_gf(sym_q, order);
    QSeries { fps }
}

/// Generating function for partitions into distinct parts: prod_{n>=1}(1+q^n).
#[pyfunction]
pub fn distinct_parts_gf(session: &QSession, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::distinct_parts_gf(sym_q, order);
    QSeries { fps }
}

/// Generating function for partitions into odd parts: prod_{k>=0} 1/(1-q^{2k+1}).
#[pyfunction]
pub fn odd_parts_gf(session: &QSession, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::odd_parts_gf(sym_q, order);
    QSeries { fps }
}

/// Generating function for partitions with at most max_part parts.
#[pyfunction]
pub fn bounded_parts_gf(session: &QSession, max_part: i64, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::bounded_parts_gf(max_part, sym_q, order);
    QSeries { fps }
}

/// Compute the rank generating function R(z, q).
///
/// z is specified as z_num/z_den. At z=1, returns the partition generating function.
#[pyfunction]
pub fn rank_gf(session: &QSession, z_num: i64, z_den: i64, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let z = QRat::from((z_num, z_den));
    let fps = qseries::rank_gf(&z, sym_q, order);
    QSeries { fps }
}

/// Compute the crank generating function C(z, q).
///
/// z is specified as z_num/z_den. At z=1, returns the partition generating function.
#[pyfunction]
pub fn crank_gf(session: &QSession, z_num: i64, z_den: i64, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let z = QRat::from((z_num, z_den));
    let fps = qseries::crank_gf(&z, sym_q, order);
    QSeries { fps }
}

// ===========================================================================
// GROUP 5: Factoring, Utilities, and Prodmake/Post-processing
// ===========================================================================

/// Factor a q-polynomial into (1-q^i) components.
///
/// Returns a dict: {"scalar": Fraction, "factors": {i: multiplicity, ...}, "is_exact": bool}
#[pyfunction]
pub fn qfactor(py: Python<'_>, series: &QSeries) -> PyResult<PyObject> {
    let result = qseries::qfactor(&series.fps);
    let dict = PyDict::new(py);
    dict.set_item("scalar", qrat_to_python(py, &result.scalar)?)?;

    let factors_dict = PyDict::new(py);
    for (&i, &mult) in &result.factors {
        factors_dict.set_item(i, mult)?;
    }
    dict.set_item("factors", factors_dict)?;
    dict.set_item("is_exact", result.is_exact)?;

    Ok(dict.into())
}

/// Extract arithmetic subsequence: g[i] = series[m*i + j].
///
/// Named `sift` in Python (registered as "sift" to avoid name conflict with
/// QSeries.sift() method, since Python sees them differently).
#[pyfunction]
#[pyo3(name = "sift")]
pub fn sift_fn(series: &QSeries, m: i64, j: i64) -> QSeries {
    QSeries {
        fps: qseries::sift(&series.fps, m, j),
    }
}

/// Highest nonzero exponent (degree) of a series.
#[pyfunction]
pub fn qdegree(series: &QSeries) -> Option<i64> {
    qseries::qdegree(&series.fps)
}

/// Lowest nonzero exponent (low degree / valuation) of a series.
#[pyfunction]
pub fn lqdegree(series: &QSeries) -> Option<i64> {
    qseries::lqdegree(&series.fps)
}

/// Andrews' algorithm: recover infinite product exponents from series coefficients.
///
/// Returns a dict: {"factors": {n: Fraction(exponent), ...}, "terms_used": int}
#[pyfunction]
pub fn prodmake(py: Python<'_>, series: &QSeries, max_n: i64) -> PyResult<PyObject> {
    let result = qseries::prodmake(&series.fps, max_n);
    let dict = PyDict::new(py);

    let factors_dict = PyDict::new(py);
    for (&n, exp) in &result.exponents {
        factors_dict.set_item(n, qrat_to_python(py, exp)?)?;
    }
    dict.set_item("factors", factors_dict)?;
    dict.set_item("terms_used", result.terms_used)?;

    Ok(dict.into())
}

/// Express a series as an eta-quotient: prod eta(d*tau)^{r_d}.
///
/// Returns a dict: {"factors": {d: int, ...}, "q_shift": Fraction}
#[pyfunction]
pub fn etamake(py: Python<'_>, series: &QSeries, max_n: i64) -> PyResult<PyObject> {
    let result = qseries::etamake(&series.fps, max_n);
    let dict = PyDict::new(py);

    let factors_dict = PyDict::new(py);
    for (&d, &r_d) in &result.factors {
        factors_dict.set_item(d, r_d)?;
    }
    dict.set_item("factors", factors_dict)?;
    dict.set_item("q_shift", qrat_to_python(py, &result.q_shift)?)?;

    Ok(dict.into())
}

/// Express a series as a Jacobi product form: prod JAC(a,b)^exp.
///
/// Returns a dict: {"factors": {(a,b): int, ...}, "scalar": Fraction, "is_exact": bool}
#[pyfunction]
pub fn jacprodmake(py: Python<'_>, series: &QSeries, max_n: i64) -> PyResult<PyObject> {
    let result = qseries::jacprodmake(&series.fps, max_n);
    let dict = PyDict::new(py);

    let factors_dict = PyDict::new(py);
    for (&(a, b), &exp) in &result.factors {
        factors_dict.set_item((a, b), exp)?;
    }
    dict.set_item("factors", factors_dict)?;
    dict.set_item("scalar", qrat_to_python(py, &result.scalar)?)?;
    dict.set_item("is_exact", result.is_exact)?;

    Ok(dict.into())
}

/// Express a series as a product of (1+q^n) factors.
///
/// Returns a dict: {n: int, ...}
#[pyfunction]
pub fn mprodmake(py: Python<'_>, series: &QSeries, max_n: i64) -> PyResult<PyObject> {
    let result = qseries::mprodmake(&series.fps, max_n);
    let dict = PyDict::new(py);
    for (&n, &m_n) in &result {
        dict.set_item(n, m_n)?;
    }
    Ok(dict.into())
}

/// Express a series in (q^d;q^d)_inf notation.
///
/// Returns a dict: {"factors": {d: int, ...}, "q_shift": Fraction}
#[pyfunction]
pub fn qetamake(py: Python<'_>, series: &QSeries, max_n: i64) -> PyResult<PyObject> {
    let result = qseries::qetamake(&series.fps, max_n);
    let dict = PyDict::new(py);

    let factors_dict = PyDict::new(py);
    for (&d, &r_d) in &result.factors {
        factors_dict.set_item(d, r_d)?;
    }
    dict.set_item("factors", factors_dict)?;
    dict.set_item("q_shift", qrat_to_python(py, &result.q_shift)?)?;

    Ok(dict.into())
}
