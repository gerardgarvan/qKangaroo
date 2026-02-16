//! Python DSL functions for q-series operations.
//!
//! Each function wraps a corresponding qsym_core function, taking a QSession
//! to obtain SymbolId for the series variable, and returning QSeries results.
//!
//! CRITICAL: All functions use `session_inner.get_or_create_symbol_id("q")` to
//! get a SymbolId (NOT `arena.intern_symbol("q")` which returns ExprRef).

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyDict, PyList};

use qsym_core::number::QRat;
use qsym_core::series::{FormalPowerSeries, arithmetic};
use qsym_core::qseries::{
    self, QMonomial, PochhammerOrder, HypergeometricSeries, SummationResult,
    BaileyDatabase, bailey_lemma, bailey_chain, weak_bailey_lemma, bailey_discover,
    QGosperResult,
    q_zeilberger, QZeilbergerResult, detect_n_params,
    verify_wz_certificate,
    q_petkovsek,
    prove_nonterminating, NonterminatingProofResult,
    find_transformation_chain, TransformationChainResult, TransformationStep,
};

use crate::convert::{qint_to_python, qrat_to_python};
use crate::series::QSeries;
use crate::session::QSession;

/// Helper: extract a slice of FPS references from a Vec of PyRef<QSeries>.
fn extract_fps_refs<'a>(series_list: &'a [PyRef<'a, QSeries>]) -> Vec<&'a FormalPowerSeries> {
    series_list.iter().map(|s| &s.fps).collect()
}

/// Helper: convert a Vec<QRat> to a Python list of Fractions.
fn qrat_vec_to_pylist<'py>(py: Python<'py>, v: &[QRat]) -> PyResult<Bound<'py, PyList>> {
    let items: Vec<PyObject> = v
        .iter()
        .map(|c| qrat_to_python(py, c).map(|obj| obj.into()))
        .collect::<PyResult<_>>()?;
    Ok(PyList::new(py, &items)?)
}

/// Helper: convert a Vec<Vec<QRat>> to a Python list of lists of Fractions.
fn qrat_matrix_to_pylist<'py>(py: Python<'py>, m: &[Vec<QRat>]) -> PyResult<Bound<'py, PyList>> {
    let rows: Vec<PyObject> = m
        .iter()
        .map(|row| qrat_vec_to_pylist(py, row).map(|list| list.into()))
        .collect::<PyResult<_>>()?;
    Ok(PyList::new(py, &rows)?)
}

// ===========================================================================
// GROUP 1: Pochhammer and q-Binomial
// ===========================================================================

/// Compute the q-Pochhammer symbol $(a; q)_n$ as a formal power series.
///
/// Evaluates the general q-shifted factorial (q-Pochhammer symbol).
/// The base monomial is $a = \frac{\text{coeff\_num}}{\text{coeff\_den}} \cdot q^{\text{power}}$.
/// Use ``n=None`` for the infinite product $(a; q)_\infty$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// coeff_num : int
///     Numerator of the monomial coefficient.
/// coeff_den : int
///     Denominator of the monomial coefficient.
/// power : int
///     Power of $q$ in the monomial $a$.
/// n : int or None
///     Product length. ``None`` for $(a;q)_\infty$, or an integer $k$ for $(a;q)_k$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The q-Pochhammer symbol as a formal power series truncated to $O(q^{\text{order}})$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, aqprod
/// >>> s = QSession()
/// >>> qq_inf = aqprod(s, 1, 1, 1, None, 20)   # (q;q)_inf (Euler function)
/// >>> qq_5 = aqprod(s, 1, 1, 1, 5, 20)         # (q;q)_5 (finite product)
///
/// See Also
/// --------
/// etaq : Generalized eta product $(q^b; q^t)_\infty$.
/// qbin : q-binomial coefficient using q-Pochhammer symbols.
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

/// Compute the q-binomial (Gaussian) coefficient $\binom{n}{k}_q$.
///
/// The q-binomial coefficient is defined by
/// $\binom{n}{k}_q = \frac{(q;q)_n}{(q;q)_k (q;q)_{n-k}}$
/// and satisfies the recurrence
/// $\binom{n}{k}_q = \binom{n-1}{k-1}_q + q^k \binom{n-1}{k}_q$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// n : int
///     Top parameter of the q-binomial.
/// k : int
///     Bottom parameter of the q-binomial.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The q-binomial coefficient as a polynomial in $q$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, qbin
/// >>> s = QSession()
/// >>> b = qbin(s, 5, 2, 20)  # [5 choose 2]_q
///
/// See Also
/// --------
/// aqprod : General q-Pochhammer symbol.
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

/// Compute the generalized eta product $(q^b; q^t)_\infty$.
///
/// Evaluates the infinite product $\prod_{k=0}^{\infty}(1 - q^{b + kt})$.
/// When $b = t$, this gives $(q^t; q^t)_\infty$, related to the Dedekind eta
/// function $\eta(\tau) = q^{1/24} (q;q)_\infty$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// b : int
///     Starting exponent. Must be positive.
/// t : int
///     Step size. Must be positive.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The infinite product as a formal power series truncated to $O(q^{\text{order}})$.
///
/// Raises
/// ------
/// ValueError
///     If ``b`` or ``t`` is not positive.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq
/// >>> s = QSession()
/// >>> euler = etaq(s, 1, 1, 20)  # (q;q)_inf = Euler function
/// >>> eta5 = etaq(s, 1, 5, 20)   # (q;q^5)_inf
///
/// See Also
/// --------
/// aqprod : General q-Pochhammer symbol $(a;q)_n$.
/// jacprod : Jacobi triple product JAC(a, b).
#[pyfunction]
pub fn etaq(session: &QSession, b: i64, t: i64, order: i64) -> PyResult<QSeries> {
    if b <= 0 {
        return Err(PyValueError::new_err(format!(
            "etaq(): parameter 'b' must be positive, got b={}", b
        )));
    }
    if t <= 0 {
        return Err(PyValueError::new_err(format!(
            "etaq(): parameter 't' must be positive, got t={}", t
        )));
    }
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::etaq(b, t, sym_q, order);
    Ok(QSeries { fps })
}

/// Compute the Jacobi triple product $\text{JAC}(a, b)$.
///
/// Evaluates the three-factor product
/// $\text{JAC}(a,b) = (q^a; q^b)_\infty \cdot (q^{b-a}; q^b)_\infty \cdot (q^b; q^b)_\infty$.
/// Requires $0 < a < b$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// a : int
///     First exponent. Must satisfy $0 < a < b$.
/// b : int
///     Period. Must satisfy $b > a > 0$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The Jacobi triple product as a formal power series.
///
/// Raises
/// ------
/// ValueError
///     If ``a`` and ``b`` do not satisfy $0 < a < b$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, jacprod
/// >>> s = QSession()
/// >>> j = jacprod(s, 1, 5, 20)  # JAC(1, 5)
///
/// See Also
/// --------
/// etaq : Generalized eta product.
/// tripleprod : Jacobi triple product with monomial parameter.
#[pyfunction]
pub fn jacprod(session: &QSession, a: i64, b: i64, order: i64) -> PyResult<QSeries> {
    if a <= 0 || a >= b {
        return Err(PyValueError::new_err(format!(
            "jacprod(): requires 0 < a < b, got a={}, b={}", a, b
        )));
    }
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::jacprod(a, b, sym_q, order);
    Ok(QSeries { fps })
}

/// Compute the Jacobi triple product with monomial parameter $z$.
///
/// Evaluates $(z; q)_\infty \cdot (q/z; q)_\infty \cdot (q; q)_\infty$
/// where $z = \frac{\text{coeff\_num}}{\text{coeff\_den}} \cdot q^{\text{power}}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// coeff_num : int
///     Numerator of the monomial coefficient for $z$.
/// coeff_den : int
///     Denominator of the monomial coefficient for $z$.
/// power : int
///     Power of $q$ in the monomial $z$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The triple product as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, tripleprod
/// >>> s = QSession()
/// >>> tp = tripleprod(s, 1, 1, 1, 20)  # z = q
///
/// See Also
/// --------
/// jacprod : Jacobi triple product JAC(a, b) with integer parameters.
/// quinprod : Quintuple product identity.
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

/// Compute the quintuple product with monomial parameter $z$.
///
/// Evaluates the five-factor quintuple product identity with
/// $z = \frac{\text{coeff\_num}}{\text{coeff\_den}} \cdot q^{\text{power}}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// coeff_num : int
///     Numerator of the monomial coefficient for $z$.
/// coeff_den : int
///     Denominator of the monomial coefficient for $z$.
/// power : int
///     Power of $q$ in the monomial $z$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The quintuple product as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, quinprod
/// >>> s = QSession()
/// >>> qp = quinprod(s, 1, 1, 1, 20)  # z = q
///
/// See Also
/// --------
/// tripleprod : Jacobi triple product.
/// winquist : Winquist's identity product.
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

/// Compute Winquist's identity product with two monomial parameters $a$ and $b$.
///
/// Evaluates the 10-factor product from Winquist's identity, where
/// $a = \frac{a\_cn}{a\_cd} \cdot q^{a\_p}$ and $b = \frac{b\_cn}{b\_cd} \cdot q^{b\_p}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// a_cn : int
///     Numerator of the coefficient for monomial $a$.
/// a_cd : int
///     Denominator of the coefficient for monomial $a$.
/// a_p : int
///     Power of $q$ in monomial $a$.
/// b_cn : int
///     Numerator of the coefficient for monomial $b$.
/// b_cd : int
///     Denominator of the coefficient for monomial $b$.
/// b_p : int
///     Power of $q$ in monomial $b$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The Winquist product as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, winquist
/// >>> s = QSession()
/// >>> w = winquist(s, 1, 1, 1, 1, 1, 2, 20)
///
/// See Also
/// --------
/// quinprod : Quintuple product identity.
/// tripleprod : Jacobi triple product.
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

/// Compute the Jacobi theta function $\theta_2(q)$.
///
/// Evaluates $\theta_2(q) = 2q^{1/4} \sum_{n=0}^{\infty} q^{n(n+1)}$.
/// The result is returned as a series in $X = q^{1/4}$, where the
/// variable represents $q^{1/4}$ (not $q$ itself).
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// order : int
///     Truncation order for the resulting series (in terms of $q^{1/4}$).
///
/// Returns
/// -------
/// QSeries
///     $\theta_2(q)$ as a series in $q^{1/4}$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, theta2
/// >>> s = QSession()
/// >>> t2 = theta2(s, 40)
///
/// See Also
/// --------
/// theta3 : Jacobi theta function $\theta_3(q)$.
/// theta4 : Jacobi theta function $\theta_4(q)$.
#[pyfunction]
pub fn theta2(session: &QSession, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::theta2(sym_q, order);
    QSeries { fps }
}

/// Compute the Jacobi theta function $\theta_3(q) = 1 + 2\sum_{n=1}^{\infty} q^{n^2}$.
///
/// Also written as $\theta_3(q) = \sum_{n=-\infty}^{\infty} q^{n^2}$.
/// This is the classical third Jacobi theta function.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     $\theta_3(q)$ as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, theta3
/// >>> s = QSession()
/// >>> t3 = theta3(s, 20)  # 1 + 2q + 2q^4 + 2q^9 + ...
///
/// See Also
/// --------
/// theta2 : Jacobi theta function $\theta_2(q)$.
/// theta4 : Jacobi theta function $\theta_4(q)$.
#[pyfunction]
pub fn theta3(session: &QSession, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::theta3(sym_q, order);
    QSeries { fps }
}

/// Compute the Jacobi theta function $\theta_4(q) = 1 + 2\sum_{n=1}^{\infty} (-1)^n q^{n^2}$.
///
/// Also written as $\theta_4(q) = \sum_{n=-\infty}^{\infty} (-1)^n q^{n^2}$.
/// This is the classical fourth Jacobi theta function.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     $\theta_4(q)$ as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, theta4
/// >>> s = QSession()
/// >>> t4 = theta4(s, 20)  # 1 - 2q + 2q^4 - 2q^9 + ...
///
/// See Also
/// --------
/// theta2 : Jacobi theta function $\theta_2(q)$.
/// theta3 : Jacobi theta function $\theta_3(q)$.
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

/// Compute $p(n)$, the number of integer partitions of $n$.
///
/// Uses the pentagonal number recurrence for efficient computation.
/// No session is needed -- this is a pure arithmetic function.
///
/// Parameters
/// ----------
/// n : int
///     The integer to partition. Must be non-negative.
///
/// Returns
/// -------
/// int
///     The partition count $p(n)$. Returns a Python ``int``, not ``Fraction``.
///
/// Examples
/// --------
/// >>> from q_kangaroo import partition_count
/// >>> partition_count(5)
/// 7
/// >>> partition_count(100)
/// 190569292536040
///
/// Notes
/// -----
/// Computed via Euler's pentagonal recurrence with $O(n\sqrt{n})$ complexity.
///
/// See Also
/// --------
/// partition_gf : Generating function $\sum p(n) q^n$.
#[pyfunction]
pub fn partition_count(py: Python<'_>, n: i64) -> PyResult<PyObject> {
    let result = qseries::partition_count(n);
    // partition_count always returns an integer; extract numerator as QInt
    let numer = qsym_core::number::QInt(result.numer().clone());
    let obj = qint_to_python(py, &numer)?;
    Ok(obj.into())
}

/// Compute the partition generating function $\sum_{n \ge 0} p(n) q^n = \frac{1}{(q;q)_\infty}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The partition generating function as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, partition_gf
/// >>> s = QSession()
/// >>> pgf = partition_gf(s, 20)  # 1 + q + 2q^2 + 3q^3 + 5q^4 + ...
///
/// See Also
/// --------
/// partition_count : Compute $p(n)$ directly.
/// distinct_parts_gf : Partitions into distinct parts.
/// odd_parts_gf : Partitions into odd parts.
#[pyfunction]
pub fn partition_gf(session: &QSession, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::partition_gf(sym_q, order);
    QSeries { fps }
}

/// Generating function for partitions into distinct parts: $\prod_{n \ge 1}(1+q^n)$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The distinct-parts generating function as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, distinct_parts_gf
/// >>> s = QSession()
/// >>> dpgf = distinct_parts_gf(s, 20)
///
/// See Also
/// --------
/// partition_gf : Unrestricted partition generating function.
/// odd_parts_gf : Partitions into odd parts (equal to distinct parts by Euler).
#[pyfunction]
pub fn distinct_parts_gf(session: &QSession, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::distinct_parts_gf(sym_q, order);
    QSeries { fps }
}

/// Generating function for partitions into odd parts: $\prod_{k \ge 0} \frac{1}{1 - q^{2k+1}}$.
///
/// By Euler's theorem, this equals the distinct-parts generating function.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The odd-parts generating function as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, odd_parts_gf
/// >>> s = QSession()
/// >>> opgf = odd_parts_gf(s, 20)
///
/// See Also
/// --------
/// distinct_parts_gf : Partitions into distinct parts (equal by Euler's theorem).
/// partition_gf : Unrestricted partition generating function.
#[pyfunction]
pub fn odd_parts_gf(session: &QSession, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::odd_parts_gf(sym_q, order);
    QSeries { fps }
}

/// Generating function for partitions with parts of size at most ``max_part``.
///
/// Computes $\prod_{k=1}^{\text{max\_part}} \frac{1}{1 - q^k}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// max_part : int
///     Maximum allowed part size. Must be positive.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The bounded-parts generating function as a formal power series.
///
/// Raises
/// ------
/// ValueError
///     If ``max_part`` is not positive.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, bounded_parts_gf
/// >>> s = QSession()
/// >>> bp = bounded_parts_gf(s, 5, 20)
///
/// See Also
/// --------
/// partition_gf : Unrestricted partition generating function.
#[pyfunction]
pub fn bounded_parts_gf(session: &QSession, max_part: i64, order: i64) -> PyResult<QSeries> {
    if max_part <= 0 {
        return Err(PyValueError::new_err(format!(
            "bounded_parts_gf(): parameter 'max_part' must be positive, got max_part={}", max_part
        )));
    }
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let fps = qseries::bounded_parts_gf(max_part, sym_q, order);
    Ok(QSeries { fps })
}

/// Compute the rank generating function $R(z, q)$.
///
/// The rank of a partition is the largest part minus the number of parts.
/// The parameter $z$ is specified as the rational number $z\_num / z\_den$.
/// At $z = 1$, this returns the partition generating function (removable singularity).
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// z_num : int
///     Numerator of the parameter $z$.
/// z_den : int
///     Denominator of the parameter $z$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The rank generating function as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, rank_gf
/// >>> s = QSession()
/// >>> r = rank_gf(s, 1, 1, 20)  # z=1 gives partition_gf
///
/// See Also
/// --------
/// crank_gf : Crank generating function $C(z, q)$.
/// partition_gf : Unrestricted partition generating function.
#[pyfunction]
pub fn rank_gf(session: &QSession, z_num: i64, z_den: i64, order: i64) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let z = QRat::from((z_num, z_den));
    let fps = qseries::rank_gf(&z, sym_q, order);
    QSeries { fps }
}

/// Compute the crank generating function $C(z, q)$.
///
/// The crank statistic was introduced by Andrews and Garvan to explain
/// Ramanujan's partition congruences. The parameter $z$ is specified as
/// $z\_num / z\_den$. At $z = 1$, this returns the partition generating function.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// z_num : int
///     Numerator of the parameter $z$.
/// z_den : int
///     Denominator of the parameter $z$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The crank generating function as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, crank_gf
/// >>> s = QSession()
/// >>> c = crank_gf(s, 1, 1, 20)  # z=1 gives partition_gf
///
/// See Also
/// --------
/// rank_gf : Rank generating function $R(z, q)$.
/// partition_gf : Unrestricted partition generating function.
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

/// Factor a q-polynomial into $(1 - q^i)$ components.
///
/// Attempts to express the series as $c \cdot \prod_i (1 - q^i)^{m_i}$.
///
/// Parameters
/// ----------
/// series : QSeries
///     The series to factor.
///
/// Returns
/// -------
/// dict
///     A dictionary with keys:
///     - ``"scalar"`` (Fraction): the leading scalar coefficient $c$.
///     - ``"factors"`` (dict[int, int]): map from $i$ to multiplicity $m_i$.
///     - ``"is_exact"`` (bool): whether the factorization is exact.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, qfactor
/// >>> s = QSession()
/// >>> f = qfactor(etaq(s, 1, 1, 20))
/// >>> f["is_exact"]
/// True
///
/// See Also
/// --------
/// prodmake : Recover infinite product exponents (Andrews' algorithm).
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

/// Extract arithmetic subsequence: $g[i] = f[mi + j]$.
///
/// Sifts the series $f$ to produce a new series whose $i$-th coefficient
/// is the $(mi + j)$-th coefficient of $f$. This operation is critical
/// for studying partition congruences.
///
/// Parameters
/// ----------
/// series : QSeries
///     The input series $f$.
/// m : int
///     The modulus (step size).
/// j : int
///     The residue (offset).
///
/// Returns
/// -------
/// QSeries
///     The sifted series $g$ with $g[i] = f[mi + j]$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, partition_gf, sift
/// >>> s = QSession()
/// >>> pgf = partition_gf(s, 100)
/// >>> sifted = sift(pgf, 5, 4)  # coefficients at indices 4, 9, 14, ...
///
/// See Also
/// --------
/// findcong : Discover partition congruences automatically.
#[pyfunction]
#[pyo3(name = "sift")]
pub fn sift_fn(series: &QSeries, m: i64, j: i64) -> QSeries {
    QSeries {
        fps: qseries::sift(&series.fps, m, j),
    }
}

/// Return the highest nonzero exponent (degree) of a series.
///
/// Parameters
/// ----------
/// series : QSeries
///     The input series.
///
/// Returns
/// -------
/// int or None
///     The largest exponent with a nonzero coefficient, or ``None`` if the series is zero.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, qbin, qdegree
/// >>> s = QSession()
/// >>> qdegree(qbin(s, 5, 2, 20))
/// 6
///
/// See Also
/// --------
/// lqdegree : Lowest nonzero exponent (valuation).
#[pyfunction]
pub fn qdegree(series: &QSeries) -> Option<i64> {
    qseries::qdegree(&series.fps)
}

/// Return the lowest nonzero exponent (valuation) of a series.
///
/// Parameters
/// ----------
/// series : QSeries
///     The input series.
///
/// Returns
/// -------
/// int or None
///     The smallest exponent with a nonzero coefficient, or ``None`` if the series is zero.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, lqdegree
/// >>> s = QSession()
/// >>> lqdegree(etaq(s, 1, 1, 20))
/// 0
///
/// See Also
/// --------
/// qdegree : Highest nonzero exponent (degree).
#[pyfunction]
pub fn lqdegree(series: &QSeries) -> Option<i64> {
    qseries::lqdegree(&series.fps)
}

/// Recover infinite product exponents from series coefficients (Andrews' algorithm).
///
/// Given a series $f(q)$, finds exponents $e_n$ such that
/// $f(q) \approx \prod_{n=1}^{N} (1 - q^n)^{e_n}$.
/// Uses log-derivative recurrence and Mobius inversion.
///
/// Parameters
/// ----------
/// series : QSeries
///     The input series to decompose.
/// max_n : int
///     Maximum factor index $N$ to compute.
///
/// Returns
/// -------
/// dict
///     A dictionary with keys:
///     - ``"factors"`` (dict[int, Fraction]): map from $n$ to exponent $e_n$.
///     - ``"terms_used"`` (int): number of series terms consumed.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, partition_gf, prodmake
/// >>> s = QSession()
/// >>> pm = prodmake(partition_gf(s, 50), 20)
/// >>> pm["factors"][1]  # exponent of (1-q) should be -1
/// Fraction(-1, 1)
///
/// See Also
/// --------
/// etamake : Express as eta-quotient.
/// jacprodmake : Express as Jacobi products.
/// qfactor : Factor a polynomial.
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

/// Express a series as an eta-quotient: $\prod_d \eta(d\tau)^{r_d}$.
///
/// Converts the prodmake output into Dedekind eta function notation.
///
/// Parameters
/// ----------
/// series : QSeries
///     The input series to decompose.
/// max_n : int
///     Maximum factor index for the underlying prodmake.
///
/// Returns
/// -------
/// dict
///     A dictionary with keys:
///     - ``"factors"`` (dict[int, int]): map from divisor $d$ to exponent $r_d$.
///     - ``"q_shift"`` (Fraction): the $q$-power prefactor.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, partition_gf, etamake
/// >>> s = QSession()
/// >>> em = etamake(partition_gf(s, 50), 20)
///
/// See Also
/// --------
/// prodmake : Underlying Andrews' algorithm.
/// etaq : Compute an eta product directly.
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

/// Express a series as a Jacobi product form: $\prod \text{JAC}(a, b)^{e}$.
///
/// Searches for a representation using Jacobi triple products with
/// period detection and residue-class grouping.
///
/// Parameters
/// ----------
/// series : QSeries
///     The input series to decompose.
/// max_n : int
///     Maximum factor index for the underlying prodmake.
///
/// Returns
/// -------
/// dict
///     A dictionary with keys:
///     - ``"factors"`` (dict[tuple[int,int], int]): map from $(a, b)$ to exponent.
///     - ``"scalar"`` (Fraction): overall scalar factor.
///     - ``"is_exact"`` (bool): whether the decomposition covers all factors exactly.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, jacprod, jacprodmake
/// >>> s = QSession()
/// >>> jpm = jacprodmake(jacprod(s, 1, 5, 30), 20)
///
/// See Also
/// --------
/// prodmake : Underlying Andrews' algorithm.
/// jacprod : Compute a Jacobi product directly.
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

/// Express a series as a product of $(1 + q^n)$ factors.
///
/// Iteratively extracts factors using $(1 + q^n) = (1 - q^{2n})/(1 - q^n)$.
///
/// Parameters
/// ----------
/// series : QSeries
///     The input series to decompose.
/// max_n : int
///     Maximum factor index.
///
/// Returns
/// -------
/// dict[int, int]
///     Map from $n$ to the multiplicity of $(1 + q^n)$ in the factorization.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, distinct_parts_gf, mprodmake
/// >>> s = QSession()
/// >>> mp = mprodmake(distinct_parts_gf(s, 30), 20)
///
/// See Also
/// --------
/// prodmake : General infinite product decomposition.
#[pyfunction]
pub fn mprodmake(py: Python<'_>, series: &QSeries, max_n: i64) -> PyResult<PyObject> {
    let result = qseries::mprodmake(&series.fps, max_n);
    let dict = PyDict::new(py);
    for (&n, &m_n) in &result {
        dict.set_item(n, m_n)?;
    }
    Ok(dict.into())
}

/// Express a series in $(q^d; q^d)_\infty$ notation.
///
/// Similar to etamake but uses q-Pochhammer notation instead of Dedekind eta.
///
/// Parameters
/// ----------
/// series : QSeries
///     The input series to decompose.
/// max_n : int
///     Maximum factor index.
///
/// Returns
/// -------
/// dict
///     A dictionary with keys:
///     - ``"factors"`` (dict[int, int]): map from $d$ to exponent of $(q^d; q^d)_\infty$.
///     - ``"q_shift"`` (Fraction): the $q$-power prefactor.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, partition_gf, qetamake
/// >>> s = QSession()
/// >>> qem = qetamake(partition_gf(s, 50), 20)
///
/// See Also
/// --------
/// etamake : Express as eta-quotient.
/// prodmake : General infinite product decomposition.
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

// ===========================================================================
// GROUP 6: Relation Discovery (exact rational)
// ===========================================================================

/// Find $f$ as a linear combination of basis series: $f = \sum_i c_i \cdot g_i$.
///
/// Uses exact rational arithmetic (RREF over $\mathbb{Q}$).
///
/// Parameters
/// ----------
/// target : QSeries
///     The target series $f$ to express as a combination.
/// candidates : list[QSeries]
///     The basis series $[g_0, g_1, \ldots]$.
/// topshift : int
///     Number of leading coefficients to ignore (shift the comparison window).
///
/// Returns
/// -------
/// list[Fraction] or None
///     Coefficients $[c_0, c_1, \ldots]$ such that $f = \sum c_i g_i$,
///     or ``None`` if no linear combination exists.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, findlincombo
/// >>> s = QSession()
/// >>> f = etaq(s, 1, 1, 30)
/// >>> result = findlincombo(f, [f], 0)
/// >>> result  # [Fraction(1, 1)]
///
/// See Also
/// --------
/// findhom : Homogeneous polynomial relations among series.
/// findhomcombo : Express target as a homogeneous combination.
#[pyfunction]
pub fn findlincombo(
    py: Python<'_>,
    target: &QSeries,
    candidates: Vec<PyRef<'_, QSeries>>,
    topshift: i64,
) -> PyResult<Option<PyObject>> {
    let fps_refs = extract_fps_refs(&candidates);
    match qseries::findlincombo(&target.fps, &fps_refs, topshift) {
        None => Ok(None),
        Some(coeffs) => {
            let list = qrat_vec_to_pylist(py, &coeffs)?;
            Ok(Some(list.into()))
        }
    }
}

/// Find all homogeneous degree-$d$ polynomial relations among series.
///
/// Discovers all polynomial relations of exact degree $d$ among the given
/// series, using monomials of total degree $d$.
///
/// Parameters
/// ----------
/// series_list : list[QSeries]
///     The series to search for relations among.
/// degree : int
///     The homogeneous degree $d$.
/// topshift : int
///     Number of leading coefficients to ignore.
///
/// Returns
/// -------
/// list[list[Fraction]]
///     A list of relation vectors. Each inner list gives coefficients for the
///     monomials of degree $d$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, findhom
/// >>> s = QSession()
/// >>> rels = findhom([etaq(s, 1, 5, 30), etaq(s, 2, 5, 30)], 2, 0)
///
/// See Also
/// --------
/// findnonhom : Non-homogeneous polynomial relations.
/// findpoly : Polynomial relation $P(x, y) = 0$ between two series.
#[pyfunction]
pub fn findhom(
    py: Python<'_>,
    series_list: Vec<PyRef<'_, QSeries>>,
    degree: i64,
    topshift: i64,
) -> PyResult<PyObject> {
    let fps_refs = extract_fps_refs(&series_list);
    let result = qseries::findhom(&fps_refs, degree, topshift);
    let list = qrat_matrix_to_pylist(py, &result)?;
    Ok(list.into())
}

/// Find a polynomial relation $P(x, y) = 0$ between two series.
///
/// Searches for a bivariate polynomial $P$ of degree at most ``deg_x`` in $x$
/// and ``deg_y`` in $y$ such that $P(x, y) = 0$ when $x$ and $y$ are the
/// given series.
///
/// Parameters
/// ----------
/// x : QSeries
///     First series.
/// y : QSeries
///     Second series.
/// deg_x : int
///     Maximum degree in $x$.
/// deg_y : int
///     Maximum degree in $y$.
/// topshift : int
///     Number of leading coefficients to ignore.
///
/// Returns
/// -------
/// dict or None
///     ``None`` if no relation found, otherwise a dictionary with keys:
///     - ``"coefficients"`` (list[list[Fraction]]): coefficient grid $P_{i,j}$.
///     - ``"deg_x"`` (int): degree in $x$.
///     - ``"deg_y"`` (int): degree in $y$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, findpoly
/// >>> s = QSession()
/// >>> result = findpoly(etaq(s, 1, 5, 50), etaq(s, 2, 5, 50), 3, 3, 0)
///
/// See Also
/// --------
/// findhom : Homogeneous multivariate relations.
/// findlincombo : Linear combination search.
#[pyfunction]
pub fn findpoly(
    py: Python<'_>,
    x: &QSeries,
    y: &QSeries,
    deg_x: i64,
    deg_y: i64,
    topshift: i64,
) -> PyResult<Option<PyObject>> {
    match qseries::findpoly(&x.fps, &y.fps, deg_x, deg_y, topshift) {
        None => Ok(None),
        Some(rel) => {
            let dict = PyDict::new(py);
            let coeffs = qrat_matrix_to_pylist(py, &rel.coefficients)?;
            dict.set_item("coefficients", coeffs)?;
            dict.set_item("deg_x", rel.deg_x)?;
            dict.set_item("deg_y", rel.deg_y)?;
            Ok(Some(dict.into()))
        }
    }
}

/// Discover congruences among the coefficients of a series.
///
/// Tests whether the series satisfies congruences of the form
/// $a(mn + b) \equiv 0 \pmod{r}$ for the given moduli $m$.
///
/// Parameters
/// ----------
/// series : QSeries
///     The series whose coefficients to test.
/// moduli : list[int]
///     List of moduli $m$ to test for congruences.
///
/// Returns
/// -------
/// list[dict]
///     A list of found congruences, each a dictionary with keys:
///     - ``"modulus"`` (int): the modulus $m$.
///     - ``"residue"`` (int): the residue $b$.
///     - ``"divisor"`` (int): the divisor $r$ such that $r | a(mn + b)$ for all $n$.
///
/// Notes
/// -----
/// This function does NOT take a ``topshift`` parameter.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, partition_gf, findcong
/// >>> s = QSession()
/// >>> congs = findcong(partition_gf(s, 200), [5, 7, 11])
///
/// See Also
/// --------
/// sift : Extract arithmetic subsequences manually.
#[pyfunction]
pub fn findcong(py: Python<'_>, series: &QSeries, moduli: Vec<i64>) -> PyResult<PyObject> {
    let result = qseries::findcong(&series.fps, &moduli);
    let items: Vec<PyObject> = result
        .iter()
        .map(|c| {
            let dict = PyDict::new(py);
            dict.set_item("modulus", c.modulus_m).unwrap();
            dict.set_item("residue", c.residue_b).unwrap();
            dict.set_item("divisor", c.divisor_r).unwrap();
            dict.into()
        })
        .collect();
    let list = PyList::new(py, &items)?;
    Ok(list.into())
}

/// Find all non-homogeneous polynomial relations of degree $\le d$ among series.
///
/// Includes monomials of all degrees from 0 through $d$ (not just exact degree $d$).
///
/// Parameters
/// ----------
/// series_list : list[QSeries]
///     The series to search for relations among.
/// degree : int
///     Maximum total degree $d$.
/// topshift : int
///     Number of leading coefficients to ignore.
///
/// Returns
/// -------
/// list[list[Fraction]]
///     A list of relation vectors covering monomials of all degrees $0, \ldots, d$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, findnonhom
/// >>> s = QSession()
/// >>> rels = findnonhom([etaq(s, 1, 5, 30), etaq(s, 2, 5, 30)], 2, 0)
///
/// See Also
/// --------
/// findhom : Homogeneous (exact degree) relations.
/// findnonhomcombo : Express a target as a non-homogeneous combination.
#[pyfunction]
pub fn findnonhom(
    py: Python<'_>,
    series_list: Vec<PyRef<'_, QSeries>>,
    degree: i64,
    topshift: i64,
) -> PyResult<PyObject> {
    let fps_refs = extract_fps_refs(&series_list);
    let result = qseries::findnonhom(&fps_refs, degree, topshift);
    let list = qrat_matrix_to_pylist(py, &result)?;
    Ok(list.into())
}

/// Express target as a homogeneous degree-$d$ combination of basis series.
///
/// Searches for coefficients $c_i$ such that the target equals
/// $\sum c_i \cdot M_i$ where $M_i$ are monomials of degree $d$ in the basis series.
///
/// Parameters
/// ----------
/// target : QSeries
///     The target series.
/// candidates : list[QSeries]
///     The basis series.
/// degree : int
///     The homogeneous degree $d$.
/// topshift : int
///     Number of leading coefficients to ignore.
///
/// Returns
/// -------
/// list[Fraction] or None
///     Coefficients for the degree-$d$ monomials, or ``None`` if no combination exists.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, findhomcombo
/// >>> s = QSession()
/// >>> result = findhomcombo(etaq(s, 1, 5, 30), [etaq(s, 1, 5, 30)], 1, 0)
///
/// See Also
/// --------
/// findlincombo : Linear (degree 1) combination.
/// findnonhomcombo : Non-homogeneous combination.
#[pyfunction]
pub fn findhomcombo(
    py: Python<'_>,
    target: &QSeries,
    candidates: Vec<PyRef<'_, QSeries>>,
    degree: i64,
    topshift: i64,
) -> PyResult<Option<PyObject>> {
    let fps_refs = extract_fps_refs(&candidates);
    match qseries::findhomcombo(&target.fps, &fps_refs, degree, topshift) {
        None => Ok(None),
        Some(coeffs) => {
            let list = qrat_vec_to_pylist(py, &coeffs)?;
            Ok(Some(list.into()))
        }
    }
}

/// Express target as a non-homogeneous degree $\le d$ combination of basis series.
///
/// Like ``findhomcombo`` but includes monomials of all degrees from 0 through $d$.
///
/// Parameters
/// ----------
/// target : QSeries
///     The target series.
/// candidates : list[QSeries]
///     The basis series.
/// degree : int
///     Maximum total degree $d$.
/// topshift : int
///     Number of leading coefficients to ignore.
///
/// Returns
/// -------
/// list[Fraction] or None
///     Coefficients for all monomials up to degree $d$, or ``None`` if no combination exists.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, findnonhomcombo
/// >>> s = QSession()
/// >>> result = findnonhomcombo(etaq(s, 1, 5, 30), [etaq(s, 1, 5, 30)], 2, 0)
///
/// See Also
/// --------
/// findhomcombo : Homogeneous combination.
/// findlincombo : Linear combination.
#[pyfunction]
pub fn findnonhomcombo(
    py: Python<'_>,
    target: &QSeries,
    candidates: Vec<PyRef<'_, QSeries>>,
    degree: i64,
    topshift: i64,
) -> PyResult<Option<PyObject>> {
    let fps_refs = extract_fps_refs(&candidates);
    match qseries::findnonhomcombo(&target.fps, &fps_refs, degree, topshift) {
        None => Ok(None),
        Some(coeffs) => {
            let list = qrat_vec_to_pylist(py, &coeffs)?;
            Ok(Some(list.into()))
        }
    }
}

// ===========================================================================
// GROUP 7: Relation Discovery (modular and structural)
// ===========================================================================

/// Find a linear combination modulo $p$: $f \equiv \sum_i c_i \cdot g_i \pmod{p}$.
///
/// Coefficients are integers modulo $p$, not exact rationals.
///
/// Parameters
/// ----------
/// target : QSeries
///     The target series $f$.
/// candidates : list[QSeries]
///     The basis series $[g_0, g_1, \ldots]$.
/// p : int
///     The prime modulus.
/// topshift : int
///     Number of leading coefficients to ignore.
///
/// Returns
/// -------
/// list[int] or None
///     Coefficients $[c_0, c_1, \ldots]$ as integers mod $p$,
///     or ``None`` if no combination exists.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, findlincombomodp
/// >>> s = QSession()
/// >>> result = findlincombomodp(etaq(s, 1, 1, 30), [etaq(s, 1, 1, 30)], 7, 0)
///
/// See Also
/// --------
/// findlincombo : Exact rational version.
/// findhommodp : Homogeneous relations mod $p$.
#[pyfunction]
pub fn findlincombomodp(
    _py: Python<'_>,
    target: &QSeries,
    candidates: Vec<PyRef<'_, QSeries>>,
    p: i64,
    topshift: i64,
) -> Option<Vec<i64>> {
    let fps_refs = extract_fps_refs(&candidates);
    qseries::findlincombomodp(&target.fps, &fps_refs, p, topshift)
}

/// Find homogeneous degree-$d$ relations modulo $p$.
///
/// Coefficients are integers modulo $p$, not exact rationals.
///
/// Parameters
/// ----------
/// series_list : list[QSeries]
///     The series to search for relations among.
/// p : int
///     The prime modulus.
/// degree : int
///     The homogeneous degree $d$.
/// topshift : int
///     Number of leading coefficients to ignore.
///
/// Returns
/// -------
/// list[list[int]]
///     A list of relation vectors with integer coefficients mod $p$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, findhommodp
/// >>> s = QSession()
/// >>> rels = findhommodp([etaq(s, 1, 5, 30), etaq(s, 2, 5, 30)], 7, 2, 0)
///
/// See Also
/// --------
/// findhom : Exact rational version.
/// findlincombomodp : Linear combination mod $p$.
#[pyfunction]
pub fn findhommodp(
    _py: Python<'_>,
    series_list: Vec<PyRef<'_, QSeries>>,
    p: i64,
    degree: i64,
    topshift: i64,
) -> Vec<Vec<i64>> {
    let fps_refs = extract_fps_refs(&series_list);
    qseries::findhommodp(&fps_refs, p, degree, topshift)
}

/// Express target as a homogeneous degree-$d$ combination modulo $p$.
///
/// Coefficients are integers modulo $p$, not exact rationals.
///
/// Parameters
/// ----------
/// target : QSeries
///     The target series.
/// candidates : list[QSeries]
///     The basis series.
/// p : int
///     The prime modulus.
/// degree : int
///     The homogeneous degree $d$.
/// topshift : int
///     Number of leading coefficients to ignore.
///
/// Returns
/// -------
/// list[int] or None
///     Coefficients for degree-$d$ monomials as integers mod $p$,
///     or ``None`` if no combination exists.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, findhomcombomodp
/// >>> s = QSession()
/// >>> result = findhomcombomodp(etaq(s, 1, 5, 30), [etaq(s, 1, 5, 30)], 7, 1, 0)
///
/// See Also
/// --------
/// findhomcombo : Exact rational version.
#[pyfunction]
pub fn findhomcombomodp(
    _py: Python<'_>,
    target: &QSeries,
    candidates: Vec<PyRef<'_, QSeries>>,
    p: i64,
    degree: i64,
    topshift: i64,
) -> Option<Vec<i64>> {
    let fps_refs = extract_fps_refs(&candidates);
    qseries::findhomcombomodp(&target.fps, &fps_refs, p, degree, topshift)
}

/// Find the maximal linearly independent subset of the given series.
///
/// Uses inline Gaussian elimination to extract pivot columns.
///
/// Parameters
/// ----------
/// series_list : list[QSeries]
///     The series to test for linear independence.
/// topshift : int
///     Number of leading coefficients to ignore.
///
/// Returns
/// -------
/// list[int]
///     Indices of a maximal linearly independent subset.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, findmaxind
/// >>> s = QSession()
/// >>> indices = findmaxind([etaq(s, 1, 5, 30), etaq(s, 2, 5, 30)], 0)
///
/// See Also
/// --------
/// findlincombo : Test if a series is a combination of others.
#[pyfunction]
pub fn findmaxind(series_list: Vec<PyRef<'_, QSeries>>, topshift: i64) -> Vec<usize> {
    let fps_refs = extract_fps_refs(&series_list);
    qseries::findmaxind(&fps_refs, topshift)
}

/// Search for linear combinations of series with nice infinite product forms.
///
/// Brute-force search over integer coefficient vectors in
/// $[-\text{max\_coeff}, \text{max\_coeff}]^k$ checking which combinations
/// have nice product representations via prodmake.
///
/// Parameters
/// ----------
/// series_list : list[QSeries]
///     The series to combine.
/// max_coeff : int
///     Maximum absolute value of coefficients in the search.
/// max_exp : int
///     Maximum exponent to check in the product form.
///
/// Returns
/// -------
/// list[list[int]]
///     Coefficient vectors for combinations that have nice product forms.
///
/// Notes
/// -----
/// This function does NOT take a ``topshift`` parameter.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, etaq, findprod
/// >>> s = QSession()
/// >>> results = findprod([etaq(s, 1, 5, 30), etaq(s, 2, 5, 30)], 2, 10)
///
/// See Also
/// --------
/// prodmake : Decompose a single series into product form.
#[pyfunction]
pub fn findprod(
    _py: Python<'_>,
    series_list: Vec<PyRef<'_, QSeries>>,
    max_coeff: i64,
    max_exp: i64,
) -> Vec<Vec<i64>> {
    let fps_refs = extract_fps_refs(&series_list);
    qseries::findprod(&fps_refs, max_coeff, max_exp)
}

// ===========================================================================
// GROUP 8: Hypergeometric Series
// ===========================================================================

/// Helper: parse a list of (num, den, power) tuples into Vec<QMonomial>.
fn parse_qmonomials(params: Vec<(i64, i64, i64)>) -> Vec<QMonomial> {
    params.iter()
        .map(|(n, d, p)| QMonomial::new(QRat::from((*n, *d)), *p))
        .collect()
}

/// Evaluate the basic hypergeometric series ${}_r\phi_s$ (DLMF 17.4.1).
///
/// Computes ${}_r\phi_s(a_1, \ldots, a_r; b_1, \ldots, b_s; q, z)$ as a
/// formal power series truncated to $O(q^{\text{order}})$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// upper : list[tuple[int, int, int]]
///     Upper parameters $[a_1, \ldots, a_r]$. Each is ``(num, den, power)``
///     representing the q-monomial $\frac{num}{den} \cdot q^{power}$.
/// lower : list[tuple[int, int, int]]
///     Lower parameters $[b_1, \ldots, b_s]$, same tuple format.
/// z_num : int
///     Numerator of the argument $z$ coefficient.
/// z_den : int
///     Denominator of the argument $z$ coefficient.
/// z_pow : int
///     Power of $q$ in the argument $z$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The basic hypergeometric series as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, phi
/// >>> s = QSession()
/// >>> result = phi(s, [(1,1,2)], [], 1, 1, 1, 20)  # _1phi0(q^2; -; q, q)
///
/// See Also
/// --------
/// psi : Bilateral hypergeometric series ${}_r\psi_s$.
/// try_summation : Try closed-form summation formulas.
/// heine1 : Heine's first transformation for ${}_2\phi_1$.
#[pyfunction]
#[pyo3(signature = (session, upper, lower, z_num, z_den, z_pow, order))]
pub fn phi(
    session: &QSession,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64,
    z_den: i64,
    z_pow: i64,
    order: i64,
) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let series = HypergeometricSeries {
        upper: parse_qmonomials(upper),
        lower: parse_qmonomials(lower),
        argument: QMonomial::new(QRat::from((z_num, z_den)), z_pow),
    };
    let fps = qseries::eval_phi(&series, sym_q, order);
    QSeries { fps }
}

/// Evaluate the bilateral hypergeometric series ${}_r\psi_s$.
///
/// Computes ${}_r\psi_s(a_1, \ldots, a_r; b_1, \ldots, b_s; q, z)$ by
/// summing over both positive and negative indices.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// upper : list[tuple[int, int, int]]
///     Upper parameters. Each is ``(num, den, power)`` representing $\frac{num}{den} \cdot q^{power}$.
/// lower : list[tuple[int, int, int]]
///     Lower parameters, same tuple format.
/// z_num : int
///     Numerator of the argument $z$ coefficient.
/// z_den : int
///     Denominator of the argument $z$ coefficient.
/// z_pow : int
///     Power of $q$ in the argument $z$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The bilateral hypergeometric series as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, psi
/// >>> s = QSession()
/// >>> result = psi(s, [(1,1,2)], [(1,1,5)], 1, 1, 1, 20)
///
/// See Also
/// --------
/// phi : Basic (unilateral) hypergeometric series ${}_r\phi_s$.
#[pyfunction]
#[pyo3(signature = (session, upper, lower, z_num, z_den, z_pow, order))]
pub fn psi(
    session: &QSession,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64,
    z_den: i64,
    z_pow: i64,
    order: i64,
) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let bilateral = qseries::BilateralHypergeometricSeries {
        upper: parse_qmonomials(upper),
        lower: parse_qmonomials(lower),
        argument: QMonomial::new(QRat::from((z_num, z_den)), z_pow),
    };
    let fps = qseries::eval_psi(&bilateral, sym_q, order);
    QSeries { fps }
}

/// Try closed-form summation formulas on a hypergeometric series.
///
/// Tests the series against known summation formulas: q-Gauss, q-Vandermonde,
/// q-Saalschutz, q-Kummer, and q-Dixon. Returns the closed form if any
/// formula matches, otherwise ``None``.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// upper : list[tuple[int, int, int]]
///     Upper parameters. Each is ``(num, den, power)`` representing $\frac{num}{den} \cdot q^{power}$.
/// lower : list[tuple[int, int, int]]
///     Lower parameters, same tuple format.
/// z_num : int
///     Numerator of the argument $z$ coefficient.
/// z_den : int
///     Denominator of the argument $z$ coefficient.
/// z_pow : int
///     Power of $q$ in the argument $z$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries or None
///     The closed-form evaluation if a summation formula applies, otherwise ``None``.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, try_summation
/// >>> s = QSession()
/// >>> closed = try_summation(s, [(1,1,1),(1,1,2)], [(1,1,5)], 1, 1, 2, 30)
///
/// See Also
/// --------
/// phi : Direct evaluation of ${}_r\phi_s$.
/// heine1 : Heine's first transformation for ${}_2\phi_1$.
#[pyfunction]
#[pyo3(signature = (session, upper, lower, z_num, z_den, z_pow, order))]
pub fn try_summation(
    session: &QSession,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64,
    z_den: i64,
    z_pow: i64,
    order: i64,
) -> Option<QSeries> {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let series = HypergeometricSeries {
        upper: parse_qmonomials(upper),
        lower: parse_qmonomials(lower),
        argument: QMonomial::new(QRat::from((z_num, z_den)), z_pow),
    };
    match qseries::try_all_summations(&series, sym_q, order) {
        SummationResult::ClosedForm(fps) => Some(QSeries { fps }),
        SummationResult::NotApplicable => None,
    }
}

/// Apply Heine's first transformation to a ${}_2\phi_1$ series.
///
/// Transforms ${}_2\phi_1(a, b; c; q, z)$ into a product prefactor times
/// a different ${}_2\phi_1$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// upper : list[tuple[int, int, int]]
///     Upper parameters $[a, b]$. Each is ``(num, den, power)``.
/// lower : list[tuple[int, int, int]]
///     Lower parameters $[c]$. Each is ``(num, den, power)``.
/// z_num : int
///     Numerator of the argument $z$ coefficient.
/// z_den : int
///     Denominator of the argument $z$ coefficient.
/// z_pow : int
///     Power of $q$ in the argument $z$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// tuple[QSeries, QSeries]
///     ``(prefactor, result)`` where ``result = prefactor * transformed_phi``.
///
/// Raises
/// ------
/// ValueError
///     If the series is not a ${}_2\phi_1$ (requires exactly 2 upper and 1 lower parameter).
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, heine1
/// >>> s = QSession()
/// >>> prefactor, result = heine1(s, [(1,1,2),(1,1,3)], [(1,1,5)], 1, 1, 1, 30)
///
/// See Also
/// --------
/// heine2 : Heine's second transformation.
/// heine3 : Heine's third transformation.
/// phi : Direct evaluation of ${}_r\phi_s$.
#[pyfunction]
#[pyo3(signature = (session, upper, lower, z_num, z_den, z_pow, order))]
pub fn heine1(
    session: &QSession,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64,
    z_den: i64,
    z_pow: i64,
    order: i64,
) -> PyResult<(QSeries, QSeries)> {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let series = HypergeometricSeries {
        upper: parse_qmonomials(upper),
        lower: parse_qmonomials(lower),
        argument: QMonomial::new(QRat::from((z_num, z_den)), z_pow),
    };
    match qseries::heine_transform_1(&series, sym_q, order) {
        Some(result) => {
            let transformed_eval = qseries::eval_phi(&result.transformed, sym_q, order);
            let combined = arithmetic::mul(&result.prefactor, &transformed_eval);
            Ok((QSeries { fps: result.prefactor }, QSeries { fps: combined }))
        }
        None => Err(PyValueError::new_err(
            "heine1(): requires a 2phi1 series (2 upper, 1 lower parameters)"
        )),
    }
}

/// Apply Heine's second transformation to a ${}_2\phi_1$ series.
///
/// Transforms ${}_2\phi_1(a, b; c; q, z)$ into a product prefactor times
/// a different ${}_2\phi_1$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// upper : list[tuple[int, int, int]]
///     Upper parameters $[a, b]$. Each is ``(num, den, power)``.
/// lower : list[tuple[int, int, int]]
///     Lower parameters $[c]$. Each is ``(num, den, power)``.
/// z_num : int
///     Numerator of the argument $z$ coefficient.
/// z_den : int
///     Denominator of the argument $z$ coefficient.
/// z_pow : int
///     Power of $q$ in the argument $z$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// tuple[QSeries, QSeries]
///     ``(prefactor, result)`` where ``result = prefactor * transformed_phi``.
///
/// Raises
/// ------
/// ValueError
///     If the series is not a ${}_2\phi_1$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, heine2
/// >>> s = QSession()
/// >>> prefactor, result = heine2(s, [(1,1,2),(1,1,3)], [(1,1,5)], 1, 1, 1, 30)
///
/// See Also
/// --------
/// heine1 : Heine's first transformation.
/// heine3 : Heine's third transformation.
#[pyfunction]
#[pyo3(signature = (session, upper, lower, z_num, z_den, z_pow, order))]
pub fn heine2(
    session: &QSession,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64,
    z_den: i64,
    z_pow: i64,
    order: i64,
) -> PyResult<(QSeries, QSeries)> {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let series = HypergeometricSeries {
        upper: parse_qmonomials(upper),
        lower: parse_qmonomials(lower),
        argument: QMonomial::new(QRat::from((z_num, z_den)), z_pow),
    };
    match qseries::heine_transform_2(&series, sym_q, order) {
        Some(result) => {
            let transformed_eval = qseries::eval_phi(&result.transformed, sym_q, order);
            let combined = arithmetic::mul(&result.prefactor, &transformed_eval);
            Ok((QSeries { fps: result.prefactor }, QSeries { fps: combined }))
        }
        None => Err(PyValueError::new_err(
            "heine2(): requires a 2phi1 series (2 upper, 1 lower parameters)"
        )),
    }
}

/// Apply Heine's third transformation to a ${}_2\phi_1$ series.
///
/// Transforms ${}_2\phi_1(a, b; c; q, z)$ into a product prefactor times
/// a different ${}_2\phi_1$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// upper : list[tuple[int, int, int]]
///     Upper parameters $[a, b]$. Each is ``(num, den, power)``.
/// lower : list[tuple[int, int, int]]
///     Lower parameters $[c]$. Each is ``(num, den, power)``.
/// z_num : int
///     Numerator of the argument $z$ coefficient.
/// z_den : int
///     Denominator of the argument $z$ coefficient.
/// z_pow : int
///     Power of $q$ in the argument $z$.
/// order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// tuple[QSeries, QSeries]
///     ``(prefactor, result)`` where ``result = prefactor * transformed_phi``.
///
/// Raises
/// ------
/// ValueError
///     If the series is not a ${}_2\phi_1$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, heine3
/// >>> s = QSession()
/// >>> prefactor, result = heine3(s, [(1,1,2),(1,1,3)], [(1,1,5)], 1, 1, 1, 30)
///
/// See Also
/// --------
/// heine1 : Heine's first transformation.
/// heine2 : Heine's second transformation.
#[pyfunction]
#[pyo3(signature = (session, upper, lower, z_num, z_den, z_pow, order))]
pub fn heine3(
    session: &QSession,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64,
    z_den: i64,
    z_pow: i64,
    order: i64,
) -> PyResult<(QSeries, QSeries)> {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");
    let series = HypergeometricSeries {
        upper: parse_qmonomials(upper),
        lower: parse_qmonomials(lower),
        argument: QMonomial::new(QRat::from((z_num, z_den)), z_pow),
    };
    match qseries::heine_transform_3(&series, sym_q, order) {
        Some(result) => {
            let transformed_eval = qseries::eval_phi(&result.transformed, sym_q, order);
            let combined = arithmetic::mul(&result.prefactor, &transformed_eval);
            Ok((QSeries { fps: result.prefactor }, QSeries { fps: combined }))
        }
        None => Err(PyValueError::new_err(
            "heine3(): requires a 2phi1 series (2 upper, 1 lower parameters)"
        )),
    }
}

// ===========================================================================
// GROUP 9: Identity Proving and Database
// ===========================================================================

/// Prove an eta-quotient identity via the valence formula.
///
/// Each side is given as a list of $(\delta, r_\delta)$ pairs representing
/// $\prod_\delta \eta(\delta\tau)^{r_\delta}$. The proof uses the Sturm
/// bound to verify equality of modular forms.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session (kept for API consistency).
/// lhs_factors : list[tuple[int, int]]
///     Left-hand side factors as $(\delta, r_\delta)$ pairs.
/// rhs_factors : list[tuple[int, int]]
///     Right-hand side factors as $(\delta, r_\delta)$ pairs.
/// level : int
///     The modular level $N$. Must be positive.
///
/// Returns
/// -------
/// dict
///     A dictionary with key ``"status"`` being one of:
///     - ``"proved"``: identity verified up to Sturm bound (includes ``"sturm_bound"``, ``"cusp_orders"``).
///     - ``"not_modular"``: modularity conditions failed (includes ``"failed_conditions"``).
///     - ``"negative_order"``: cusp order is negative (includes ``"cusp"``, ``"order"``).
///     - ``"counterexample"``: coefficient mismatch found (includes ``"coefficient_index"``).
///
/// Raises
/// ------
/// ValueError
///     If ``level`` is not positive.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, prove_eta_id
/// >>> s = QSession()
/// >>> result = prove_eta_id(s, [(5, 6)], [(1, 6)], 5)
/// >>> result["status"]
/// 'proved'
///
/// See Also
/// --------
/// search_identities : Search the identity database.
/// etamake : Express a series as an eta-quotient.
#[pyfunction]
#[pyo3(signature = (session, lhs_factors, rhs_factors, level))]
pub fn prove_eta_id(
    session: &QSession,
    lhs_factors: Vec<(i64, i64)>,
    rhs_factors: Vec<(i64, i64)>,
    level: i64,
) -> PyResult<PyObject> {
    let _ = session; // Session not needed for proving, but keeps API consistent
    if level <= 0 {
        return Err(PyValueError::new_err(format!(
            "prove_eta_id(): parameter 'level' must be positive, got level={}", level
        )));
    }

    use qsym_core::qseries::identity::{EtaExpression, EtaIdentity, prove_eta_identity, ProofResult};

    let lhs = EtaExpression::from_factors(&lhs_factors, level);
    let rhs = EtaExpression::from_factors(&rhs_factors, level);
    let identity = EtaIdentity::two_sided(lhs, rhs, level);
    let result = prove_eta_identity(&identity);

    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        match &result {
            ProofResult::Proved { level, cusp_orders, sturm_bound, verification_terms } => {
                dict.set_item("status", "proved")?;
                dict.set_item("level", *level)?;
                dict.set_item("sturm_bound", *sturm_bound)?;
                dict.set_item("verification_terms", *verification_terms)?;
                let cusps_list: Vec<(String, String)> = cusp_orders.iter()
                    .map(|(c, o)| (format!("{}", c), format!("{}", o)))
                    .collect();
                dict.set_item("cusp_orders", cusps_list)?;
            }
            ProofResult::NotModular { failed_conditions } => {
                dict.set_item("status", "not_modular")?;
                dict.set_item("failed_conditions", failed_conditions.clone())?;
            }
            ProofResult::NegativeOrder { cusp, order } => {
                dict.set_item("status", "negative_order")?;
                dict.set_item("cusp", format!("{}", cusp))?;
                dict.set_item("order", format!("{}", order))?;
            }
            ProofResult::CounterExample { coefficient_index, expected, actual } => {
                dict.set_item("status", "counterexample")?;
                dict.set_item("coefficient_index", *coefficient_index)?;
                dict.set_item("expected", format!("{}", expected))?;
                dict.set_item("actual", format!("{}", actual))?;
            }
        }
        Ok(dict.into())
    })
}

/// Search the identity database by tag, function, or pattern.
///
/// Searches the built-in identity database (or a custom TOML file) for
/// matching entries.
///
/// Parameters
/// ----------
/// query : str
///     Search query string.
/// search_type : str, optional
///     Type of search: ``"tag"``, ``"function"``, or ``"pattern"`` (default).
/// db_path : str or None, optional
///     Path to a custom identity database TOML file. Uses the built-in
///     database if ``None``.
///
/// Returns
/// -------
/// list[dict]
///     A list of matching identity entries, each a dictionary with keys
///     ``"id"``, ``"name"``, ``"tags"``, ``"functions"``, and optionally
///     ``"author"`` and ``"year"``.
///
/// Examples
/// --------
/// >>> from q_kangaroo import search_identities
/// >>> results = search_identities("classical", search_type="tag")
/// >>> results = search_identities("eta", search_type="function")
/// >>> results = search_identities("partition", search_type="pattern")
///
/// See Also
/// --------
/// prove_eta_id : Prove an eta-quotient identity.
#[pyfunction]
#[pyo3(signature = (query, search_type = "pattern", db_path = None))]
pub fn search_identities(
    py: Python<'_>,
    query: &str,
    search_type: &str,
    db_path: Option<&str>,
) -> PyResult<PyObject> {
    use qsym_core::qseries::identity::IdentityDatabase;

    // Load from specified path or use embedded default database
    let toml_content = if let Some(path) = db_path {
        std::fs::read_to_string(path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(format!("Cannot read {}: {}", path, e)))?
    } else {
        // Use embedded default database
        include_str!("../../../data/identities/classical_identities.toml").to_string()
    };

    let db = IdentityDatabase::load_from_toml(&toml_content)
        .map_err(|e| PyValueError::new_err(e))?;

    let results: Vec<&qsym_core::qseries::identity::IdentityEntry> = match search_type {
        "tag" => db.search_by_tag(query),
        "function" => db.search_by_function(query),
        "pattern" | _ => db.search_by_pattern(query),
    };

    let py_results: Vec<PyObject> = results.iter().map(|entry| {
        let dict = PyDict::new(py);
        dict.set_item("id", &entry.id).unwrap();
        dict.set_item("name", &entry.name).unwrap();
        dict.set_item("tags", &entry.tags).unwrap();
        dict.set_item("functions", &entry.functions).unwrap();
        if let Some(ref citation) = entry.citation {
            if let Some(ref author) = citation.author {
                dict.set_item("author", author).unwrap();
            }
            if let Some(year) = citation.year {
                dict.set_item("year", year).unwrap();
            }
        }
        dict.into()
    }).collect();

    Ok(PyList::new(py, &py_results)?.into())
}

// ===========================================================================
// GROUP 10: Mock Theta Functions, Appell-Lerch Sums & Bailey Machinery
// ===========================================================================

// ---------------------------------------------------------------------------
// 10a. Mock theta function DSL (20 functions)
// ---------------------------------------------------------------------------

/// Helper: construct a QMonomial from (num, den, power) triple.
fn qmonomial_from_tuple(num: i64, den: i64, pow: i64) -> QMonomial {
    let coeff = QRat::from((num, den));
    QMonomial::new(coeff, pow)
}

/// Third-order mock theta function $f(q) = \sum_{n=0}^{\infty} \frac{q^{n^2}}{(-q;q)_n^2}$ (Ramanujan).
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_f3
/// >>> s = QSession()
/// >>> f3 = mock_theta_f3(s, 20)
#[pyfunction]
pub fn mock_theta_f3(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_f3(var, truncation_order);
    Ok(QSeries { fps })
}

/// Third-order mock theta function $\phi(q) = \sum_{n=0}^{\infty} \frac{q^{n^2}}{(-q^2;q^2)_n}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_phi3(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_phi3(var, truncation_order);
    Ok(QSeries { fps })
}

/// Third-order mock theta function $\psi(q) = \sum_{n=1}^{\infty} \frac{q^{n^2}}{(q;q^2)_n}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_psi3(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_psi3(var, truncation_order);
    Ok(QSeries { fps })
}

/// Third-order mock theta function $\chi(q) = \sum_{n=0}^{\infty} \frac{q^{n^2} (-q;q)_n}{(-q^3;q^3)_n}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_chi3(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_chi3(var, truncation_order);
    Ok(QSeries { fps })
}

/// Third-order mock theta function $\omega(q) = \sum_{n=0}^{\infty} \frac{q^{2n(n+1)}}{(q;q^2)_{n+1}^2}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_omega3(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_omega3(var, truncation_order);
    Ok(QSeries { fps })
}

/// Third-order mock theta function $\nu(q) = \sum_{n=0}^{\infty} \frac{q^{n(n+1)}}{(-q;q^2)_{n+1}}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_nu3(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_nu3(var, truncation_order);
    Ok(QSeries { fps })
}

/// Third-order mock theta function $\rho(q) = \sum_{n=0}^{\infty} \frac{q^{2n(n+1)}}{(q;q^2)_{n+1}(q^3;q^6)_{n+1}}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_rho3(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_rho3(var, truncation_order);
    Ok(QSeries { fps })
}

/// Fifth-order mock theta function $f_0(q) = \sum_{n=0}^{\infty} \frac{q^{n^2}}{(-q;q)_n}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_f0_5(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_f0_5(var, truncation_order);
    Ok(QSeries { fps })
}

/// Fifth-order mock theta function $f_1(q) = \sum_{n=1}^{\infty} \frac{q^{n(n+1)}}{(-q;q)_n}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_f1_5(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_f1_5(var, truncation_order);
    Ok(QSeries { fps })
}

/// Fifth-order mock theta function $F_0(q) = \sum_{n=0}^{\infty} \frac{q^{2n^2}}{(q;q^2)_n}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_cap_f0_5(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_cap_f0_5(var, truncation_order);
    Ok(QSeries { fps })
}

/// Fifth-order mock theta function $F_1(q) = \sum_{n=0}^{\infty} \frac{q^{2n(n+1)}}{(q;q^2)_{n+1}}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_cap_f1_5(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_cap_f1_5(var, truncation_order);
    Ok(QSeries { fps })
}

/// Fifth-order mock theta function $\phi_0(q) = \sum_{n=0}^{\infty} q^{n^2} (-q;q^2)_n$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_phi0_5(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_phi0_5(var, truncation_order);
    Ok(QSeries { fps })
}

/// Fifth-order mock theta function $\phi_1(q) = \sum_{n=0}^{\infty} q^{(n+1)^2} (-q;q^2)_n$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_phi1_5(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_phi1_5(var, truncation_order);
    Ok(QSeries { fps })
}

/// Fifth-order mock theta function $\psi_0(q) = \sum_{n=0}^{\infty} q^{n(n+1)/2} (-q;q)_n$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_psi0_5(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_psi0_5(var, truncation_order);
    Ok(QSeries { fps })
}

/// Fifth-order mock theta function $\psi_1(q) = \sum_{n=1}^{\infty} q^{n(n+1)/2} (-q;q)_{n-1}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_psi1_5(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_psi1_5(var, truncation_order);
    Ok(QSeries { fps })
}

/// Fifth-order mock theta function $\chi_0(q) = \sum_{n=0}^{\infty} \frac{q^n (-q;q)_n}{(q;q^2)_{n+1}}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_chi0_5(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_chi0_5(var, truncation_order);
    Ok(QSeries { fps })
}

/// Fifth-order mock theta function $\chi_1(q) = \sum_{n=0}^{\infty} \frac{q^n (-q;q)_n}{(q;q^2)_{n+1}}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_chi1_5(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_chi1_5(var, truncation_order);
    Ok(QSeries { fps })
}

/// Seventh-order mock theta function $F_0(q) = \sum_{n=0}^{\infty} \frac{q^{n^2}}{(q^n;q)_n}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_cap_f0_7(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_cap_f0_7(var, truncation_order);
    Ok(QSeries { fps })
}

/// Seventh-order mock theta function $F_1(q) = \sum_{n=1}^{\infty} \frac{q^{n^2}}{(q^n;q)_n}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_cap_f1_7(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_cap_f1_7(var, truncation_order);
    Ok(QSeries { fps })
}

/// Seventh-order mock theta function $F_2(q) = \sum_{n=0}^{\infty} \frac{q^{n(n+1)}}{(q^{n+1};q)_{n+1}}$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The mock theta function as a formal power series.
#[pyfunction]
pub fn mock_theta_cap_f2_7(session: &QSession, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::mock_theta_cap_f2_7(var, truncation_order);
    Ok(QSeries { fps })
}

// ---------------------------------------------------------------------------
// 10b. Appell-Lerch sums and universal mock theta functions (3 functions)
// ---------------------------------------------------------------------------

/// Compute the Appell-Lerch bilateral sum $m(q^a, q, q^z)$ as a formal power series.
///
/// Returns the raw bilateral sum (not divided by $j(z;q)$, which vanishes
/// for integer parameters).
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// a_pow : int
///     Power of $q$ in the first argument.
/// z_pow : int
///     Power of $q$ in the third argument.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The Appell-Lerch sum as a formal power series.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, appell_lerch_m
/// >>> s = QSession()
/// >>> result = appell_lerch_m(s, 3, 2, 20)
///
/// See Also
/// --------
/// universal_mock_theta_g2 : Universal mock theta function $g_2$.
/// universal_mock_theta_g3 : Universal mock theta function $g_3$.
#[pyfunction]
pub fn appell_lerch_m(session: &QSession, a_pow: i64, z_pow: i64, truncation_order: i64) -> PyResult<QSeries> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::appell_lerch_m(a_pow, z_pow, var, truncation_order);
    Ok(QSeries { fps })
}

/// Compute the universal mock theta function $g_2(q^a, q)$ as a formal power series.
///
/// The universal mock theta function $g_2$ is defined using Appell-Lerch
/// sums with a positive-exponent algebraic identity.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// a_pow : int
///     Power of $q$ in the first argument. Must be $\ge 2$ for nontrivial result.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The universal mock theta function as a formal power series.
///
/// Raises
/// ------
/// ValueError
///     If ``a_pow`` is less than 2.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, universal_mock_theta_g2
/// >>> s = QSession()
/// >>> result = universal_mock_theta_g2(s, 3, 20)
///
/// See Also
/// --------
/// universal_mock_theta_g3 : Universal mock theta function $g_3$.
/// appell_lerch_m : Appell-Lerch bilateral sum.
#[pyfunction]
pub fn universal_mock_theta_g2(session: &QSession, a_pow: i64, truncation_order: i64) -> PyResult<QSeries> {
    if a_pow < 2 {
        return Err(PyValueError::new_err(format!(
            "universal_mock_theta_g2(): parameter 'a_pow' must be >= 2, got a_pow={}", a_pow
        )));
    }
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::universal_mock_theta_g2(a_pow, var, truncation_order);
    Ok(QSeries { fps })
}

/// Compute the universal mock theta function $g_3(q^a, q)$ as a formal power series.
///
/// The universal mock theta function $g_3$ is defined using Appell-Lerch
/// sums with a positive-exponent algebraic identity.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// a_pow : int
///     Power of $q$ in the first argument. Must be $\ge 2$ for nontrivial result.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// QSeries
///     The universal mock theta function as a formal power series.
///
/// Raises
/// ------
/// ValueError
///     If ``a_pow`` is less than 2.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, universal_mock_theta_g3
/// >>> s = QSession()
/// >>> result = universal_mock_theta_g3(s, 3, 20)
///
/// See Also
/// --------
/// universal_mock_theta_g2 : Universal mock theta function $g_2$.
/// appell_lerch_m : Appell-Lerch bilateral sum.
#[pyfunction]
pub fn universal_mock_theta_g3(session: &QSession, a_pow: i64, truncation_order: i64) -> PyResult<QSeries> {
    if a_pow < 2 {
        return Err(PyValueError::new_err(format!(
            "universal_mock_theta_g3(): parameter 'a_pow' must be >= 2, got a_pow={}", a_pow
        )));
    }
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);
    let fps = qseries::universal_mock_theta_g3(a_pow, var, truncation_order);
    Ok(QSeries { fps })
}

// ---------------------------------------------------------------------------
// 10c. Bailey machinery DSL (4 functions)
// ---------------------------------------------------------------------------

/// Compute both sides of the weak Bailey lemma for a named pair.
///
/// The weak Bailey lemma states that if $(\alpha_n, \beta_n)$ is a Bailey
/// pair relative to $a$, then
/// $\sum q^{n^2} a^n \beta_n = \frac{1}{(aq;q)_\infty} \sum q^{n^2} a^n \alpha_n$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// pair_name : str
///     Name of the Bailey pair from the database (e.g. ``"unit"``,
///     ``"rogers-ramanujan"``, ``"q-binomial"``).
/// a_num : int
///     Numerator of the parameter $a$ coefficient.
/// a_den : int
///     Denominator of the parameter $a$ coefficient.
/// a_pow : int
///     Power of $q$ in the parameter $a$.
/// max_n : int
///     Maximum summation index.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// tuple[QSeries, QSeries]
///     ``(lhs, rhs)`` -- both sides of the weak Bailey lemma.
///
/// Raises
/// ------
/// ValueError
///     If ``pair_name`` is not found in the database.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, bailey_weak_lemma
/// >>> s = QSession()
/// >>> lhs, rhs = bailey_weak_lemma(s, "rogers-ramanujan", 1, 1, 0, 10, 20)
///
/// See Also
/// --------
/// bailey_apply_lemma : Apply the full Bailey lemma.
/// bailey_chain : Iterative Bailey chain.
/// bailey_discover : Automated Bailey pair discovery.
#[pyfunction]
#[pyo3(signature = (session, pair_name, a_num, a_den, a_pow, max_n, truncation_order))]
pub fn bailey_weak_lemma(
    session: &QSession,
    pair_name: &str,
    a_num: i64,
    a_den: i64,
    a_pow: i64,
    max_n: i64,
    truncation_order: i64,
) -> PyResult<(QSeries, QSeries)> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);

    let db = BaileyDatabase::new();
    let pairs = db.search_by_name(pair_name);
    let pair = pairs.first().ok_or_else(|| {
        PyValueError::new_err(format!(
            "bailey_weak_lemma(): pair '{}' not found in database. Available: 'unit', 'rogers-ramanujan', 'q-binomial'",
            pair_name
        ))
    })?;

    let a = qmonomial_from_tuple(a_num, a_den, a_pow);
    let (lhs, rhs) = weak_bailey_lemma(pair, &a, max_n, var, truncation_order);
    Ok((QSeries { fps: lhs }, QSeries { fps: rhs }))
}

/// Apply the Bailey lemma to transform a named pair with parameters $b$, $c$.
///
/// Given a Bailey pair $(\alpha_n, \beta_n)$ relative to $a$, produces a
/// new derived pair via the Bailey lemma with additional parameters $b$, $c$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// pair_name : str
///     Name of the Bailey pair from the database.
/// a : tuple[int, int, int]
///     Parameter $a$ as ``(num, den, power)`` giving $\frac{num}{den} \cdot q^{power}$.
/// b : tuple[int, int, int]
///     Parameter $b$ as ``(num, den, power)``.
/// c : tuple[int, int, int]
///     Parameter $c$ as ``(num, den, power)``.
/// max_n : int
///     Maximum summation index.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// dict
///     A dictionary with keys ``"name"`` (str), ``"pair_type"`` (str),
///     and ``"num_terms"`` (int).
///
/// Raises
/// ------
/// ValueError
///     If ``pair_name`` is not found in the database.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, bailey_apply_lemma
/// >>> s = QSession()
/// >>> result = bailey_apply_lemma(s, "unit", (1,1,2), (1,2,1), (1,3,1), 4, 15)
///
/// See Also
/// --------
/// bailey_weak_lemma : Weak Bailey lemma.
/// bailey_chain : Iterative Bailey chain.
#[pyfunction]
#[pyo3(signature = (session, pair_name, a, b, c, max_n, truncation_order))]
pub fn bailey_apply_lemma(
    py: Python<'_>,
    session: &QSession,
    pair_name: &str,
    a: (i64, i64, i64),
    b: (i64, i64, i64),
    c: (i64, i64, i64),
    max_n: i64,
    truncation_order: i64,
) -> PyResult<PyObject> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);

    let db = BaileyDatabase::new();
    let pairs = db.search_by_name(pair_name);
    let pair = pairs.first().ok_or_else(|| {
        PyValueError::new_err(format!(
            "bailey_apply_lemma(): pair '{}' not found in database. Available: 'unit', 'rogers-ramanujan', 'q-binomial'",
            pair_name
        ))
    })?;

    let a_mon = qmonomial_from_tuple(a.0, a.1, a.2);
    let b_mon = qmonomial_from_tuple(b.0, b.1, b.2);
    let c_mon = qmonomial_from_tuple(c.0, c.1, c.2);

    let derived = bailey_lemma(pair, &a_mon, &b_mon, &c_mon, max_n, var, truncation_order);

    let dict = PyDict::new(py);
    dict.set_item("name", &derived.name)?;
    dict.set_item("pair_type", "tabulated")?;
    dict.set_item("num_terms", max_n + 1)?;
    Ok(dict.into())
}

/// Apply the Bailey lemma iteratively (Bailey chain) to a named pair.
///
/// Produces a chain of derived Bailey pairs by repeatedly applying the
/// Bailey lemma. The chain has length ``depth + 1`` (original + derived pairs).
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// pair_name : str
///     Name of the initial Bailey pair from the database.
/// a : tuple[int, int, int]
///     Parameter $a$ as ``(num, den, power)``.
/// b : tuple[int, int, int]
///     Parameter $b$ as ``(num, den, power)``.
/// c : tuple[int, int, int]
///     Parameter $c$ as ``(num, den, power)``.
/// depth : int
///     Number of iterations to apply the lemma.
/// max_n : int
///     Maximum summation index.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// list[dict]
///     A list of dictionaries, each with keys ``"name"`` (str),
///     ``"pair_type"`` (str), and ``"index"`` (int).
///
/// Raises
/// ------
/// ValueError
///     If ``pair_name`` is not found in the database.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, bailey_chain
/// >>> s = QSession()
/// >>> chain = bailey_chain(s, "unit", (1,1,2), (1,2,1), (1,3,1), 2, 4, 15)
/// >>> len(chain)
/// 3
///
/// See Also
/// --------
/// bailey_apply_lemma : Single application of the Bailey lemma.
/// bailey_discover : Automated Bailey pair discovery.
#[pyfunction]
#[pyo3(name = "bailey_chain", signature = (session, pair_name, a, b, c, depth, max_n, truncation_order))]
pub fn bailey_chain_fn(
    py: Python<'_>,
    session: &QSession,
    pair_name: &str,
    a: (i64, i64, i64),
    b: (i64, i64, i64),
    c: (i64, i64, i64),
    depth: usize,
    max_n: i64,
    truncation_order: i64,
) -> PyResult<PyObject> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);

    let db = BaileyDatabase::new();
    let pairs = db.search_by_name(pair_name);
    let pair = pairs.first().ok_or_else(|| {
        PyValueError::new_err(format!(
            "bailey_chain(): pair '{}' not found in database. Available: 'unit', 'rogers-ramanujan', 'q-binomial'",
            pair_name
        ))
    })?;

    let a_mon = qmonomial_from_tuple(a.0, a.1, a.2);
    let b_mon = qmonomial_from_tuple(b.0, b.1, b.2);
    let c_mon = qmonomial_from_tuple(c.0, c.1, c.2);

    let chain = bailey_chain(pair, &a_mon, &b_mon, &c_mon, depth, max_n, var, truncation_order);

    let items: Vec<PyObject> = chain.iter().enumerate().map(|(i, p)| {
        let dict = PyDict::new(py);
        dict.set_item("name", &p.name).unwrap();
        dict.set_item("pair_type", format!("{:?}", p.pair_type).split('{').next().unwrap_or("unknown").trim()).unwrap();
        dict.set_item("index", i).unwrap();
        dict.into()
    }).collect();

    Ok(PyList::new(py, &items)?.into())
}

/// Automated Bailey pair discovery from series data.
///
/// Searches the database for a Bailey pair that explains the relationship
/// between the given LHS and RHS series. Tries trivial equality, weak
/// lemma matching, and chain depth search up to ``max_chain_depth``.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// lhs : QSeries
///     The left-hand side series.
/// rhs : QSeries
///     The right-hand side series.
/// a : tuple[int, int, int]
///     Parameter $a$ as ``(num, den, power)``.
/// max_chain_depth : int
///     Maximum Bailey chain depth to search.
/// truncation_order : int
///     Truncation order for the resulting series.
///
/// Returns
/// -------
/// dict
///     A dictionary with keys:
///     - ``"found"`` (bool): whether a matching pair was discovered.
///     - ``"pair_name"`` (str or None): name of the matching pair.
///     - ``"chain_depth"`` (int): depth of the chain used.
///     - ``"verification"`` (str): description of the verification result.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, bailey_discover
/// >>> s = QSession()
/// >>> result = bailey_discover(s, lhs, rhs, (1,1,0), 2, 20)
/// >>> result["found"]
///
/// See Also
/// --------
/// bailey_chain : Iterative Bailey chain.
/// bailey_weak_lemma : Weak Bailey lemma verification.
#[pyfunction]
#[pyo3(name = "bailey_discover", signature = (session, lhs, rhs, a, max_chain_depth, truncation_order))]
pub fn bailey_discover_fn(
    py: Python<'_>,
    session: &QSession,
    lhs: &QSeries,
    rhs: &QSeries,
    a: (i64, i64, i64),
    max_chain_depth: usize,
    truncation_order: i64,
) -> PyResult<PyObject> {
    let mut inner = session.inner.lock().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);

    let a_mon = qmonomial_from_tuple(a.0, a.1, a.2);
    let db = BaileyDatabase::new();

    let result = bailey_discover(&lhs.fps, &rhs.fps, &db, &a_mon, max_chain_depth, var, truncation_order);

    let dict = PyDict::new(py);
    dict.set_item("found", result.found)?;
    match &result.pair_name {
        Some(name) => dict.set_item("pair_name", name)?,
        None => dict.set_item("pair_name", py.None())?,
    };
    dict.set_item("chain_depth", result.chain_depth)?;
    dict.set_item("verification", &result.verification)?;
    Ok(dict.into())
}

// ===========================================================================
// GROUP 11: q-Gosper Algorithm
// ===========================================================================

/// Run the q-Gosper algorithm for indefinite q-hypergeometric summation.
///
/// Given a q-hypergeometric series defined by upper/lower parameters and argument,
/// determines whether the series has a q-hypergeometric antidifference (indefinite sum).
///
/// Parameters
/// ----------
/// upper : list[tuple[int, int, int]]
///     Upper parameters. Each is ``(num, den, power)`` representing `(num/den) * q^power`.
/// lower : list[tuple[int, int, int]]
///     Lower parameters, same tuple format.
/// z_num : int
///     Numerator of the argument z coefficient.
/// z_den : int
///     Denominator of the argument z coefficient.
/// z_pow : int
///     Power of q in the argument z.
/// q_num : int
///     Numerator of the concrete q value (e.g. 2 for q=2).
/// q_den : int
///     Denominator of the concrete q value (e.g. 1 for q=2).
///
/// Returns
/// -------
/// dict or None
///     If summable: ``{"summable": True, "certificate": "f(x)/g(x)", "numer": "...", "denom": "..."}``.
///     If not summable: ``{"summable": False}``.
///
/// Examples
/// --------
/// >>> from q_kangaroo import q_gosper
/// >>> # q-Vandermonde: 2phi1(q^{-3}, q^2; q^3; q, q^4)
/// >>> result = q_gosper([(1,1,-3), (1,1,2)], [(1,1,3)], 1, 1, 4, 2, 1)
/// >>> result["summable"]
/// True
#[pyfunction]
#[pyo3(signature = (upper, lower, z_num, z_den, z_pow, q_num, q_den))]
pub fn q_gosper_fn(
    py: Python<'_>,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64,
    z_den: i64,
    z_pow: i64,
    q_num: i64,
    q_den: i64,
) -> PyResult<PyObject> {
    let series = HypergeometricSeries {
        upper: parse_qmonomials(upper),
        lower: parse_qmonomials(lower),
        argument: QMonomial::new(QRat::from((z_num, z_den)), z_pow),
    };
    let q_val = QRat::from((q_num, q_den));

    let result = qseries::q_gosper(&series, &q_val);

    let dict = PyDict::new(py);
    match result {
        QGosperResult::Summable { certificate } => {
            dict.set_item("summable", true)?;
            dict.set_item("certificate", format!("{}", certificate))?;
            dict.set_item("numer", format!("{}", certificate.numer))?;
            dict.set_item("denom", format!("{}", certificate.denom))?;
        }
        QGosperResult::NotSummable => {
            dict.set_item("summable", false)?;
        }
    }
    Ok(dict.into())
}

// ===========================================================================
// GROUP 12: Algorithmic Summation (q-Zeilberger, WZ Verification, q-Petkovsek)
// ===========================================================================

/// Run the q-Zeilberger creative telescoping algorithm for definite q-hypergeometric summation.
///
/// Given a q-hypergeometric series `F(n,k)` defined by upper/lower parameters and argument,
/// finds a linear recurrence `c_0*S(n) + c_1*S(n+1) + ... + c_d*S(n+d) = 0` for
/// the definite sum `S(n) = sum_k F(n,k)`, along with a WZ proof certificate.
///
/// Parameters
/// ----------
/// upper : list[tuple[int, int, int]]
///     Upper parameters. Each is ``(num, den, power)`` representing `(num/den) * q^power`.
/// lower : list[tuple[int, int, int]]
///     Lower parameters, same tuple format.
/// z_num : int
///     Numerator of the argument z coefficient.
/// z_den : int
///     Denominator of the argument z coefficient.
/// z_pow : int
///     Power of q in the argument z.
/// n_val : int
///     Concrete value of the summation parameter n.
/// q_num : int
///     Numerator of the concrete q value (e.g. 2 for q=2).
/// q_den : int
///     Denominator of the concrete q value (e.g. 1 for q=2).
/// max_order : int
///     Maximum recurrence order to search.
/// n_param_indices : list[int] or None
///     Manual override: indices into upper parameters that depend on n.
///     If None, auto-detected via ``detect_n_params``.
/// n_is_in_argument : bool or None
///     Manual override: whether n appears in the argument z.
///     If None, auto-detected via ``detect_n_params``.
///
/// Returns
/// -------
/// dict
///     If recurrence found: ``{"found": True, "order": int, "coefficients": list[Fraction],
///     "certificate": str, "numer": str, "denom": str}``.
///     If no recurrence: ``{"found": False}``.
///
/// Examples
/// --------
/// >>> from q_kangaroo import q_zeilberger
/// >>> # q-Vandermonde: 2phi1(q^{-5}, q^2; q^3; q, q^4) at n=5, q=2
/// >>> result = q_zeilberger([(1,1,-5), (1,1,2)], [(1,1,3)], 1, 1, 4, 5, 2, 1, 3)
/// >>> result["found"]
/// True
/// >>> result["order"]
/// 1
///
/// See Also
/// --------
/// q_gosper : Indefinite q-hypergeometric summation (inner subroutine).
/// verify_wz : Independent verification of the WZ certificate.
/// q_petkovsek : Solve the recurrence for closed-form solutions.
#[pyfunction]
#[pyo3(name = "q_zeilberger", signature = (upper, lower, z_num, z_den, z_pow, n_val, q_num, q_den, max_order, n_param_indices=None, n_is_in_argument=None))]
pub fn q_zeilberger_fn(
    py: Python<'_>,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64,
    z_den: i64,
    z_pow: i64,
    n_val: i64,
    q_num: i64,
    q_den: i64,
    max_order: usize,
    n_param_indices: Option<Vec<usize>>,
    n_is_in_argument: Option<bool>,
) -> PyResult<PyObject> {
    let series = HypergeometricSeries {
        upper: parse_qmonomials(upper),
        lower: parse_qmonomials(lower),
        argument: QMonomial::new(QRat::from((z_num, z_den)), z_pow),
    };
    let q_val = QRat::from((q_num, q_den));

    // Auto-detect or use manual overrides
    let (indices, in_arg) = match (&n_param_indices, &n_is_in_argument) {
        (Some(idx), Some(flag)) => (idx.clone(), *flag),
        _ => {
            let (auto_idx, auto_flag) = detect_n_params(&series, n_val, &q_val);
            (
                n_param_indices.unwrap_or(auto_idx),
                n_is_in_argument.unwrap_or(auto_flag),
            )
        }
    };

    let result = q_zeilberger(&series, n_val, &q_val, max_order, &indices, in_arg);

    let dict = PyDict::new(py);
    match result {
        QZeilbergerResult::Recurrence(zr) => {
            dict.set_item("found", true)?;
            dict.set_item("order", zr.order)?;
            dict.set_item("coefficients", qrat_vec_to_pylist(py, &zr.coefficients)?)?;
            dict.set_item("certificate", format!("{}", zr.certificate))?;
            dict.set_item("numer", format!("{}", zr.certificate.numer))?;
            dict.set_item("denom", format!("{}", zr.certificate.denom))?;
        }
        QZeilbergerResult::NoRecurrence => {
            dict.set_item("found", false)?;
        }
    }
    Ok(dict.into())
}

/// Independently verify a WZ certificate for a q-hypergeometric identity.
///
/// Re-derives the q-Zeilberger recurrence and certificate, then verifies the
/// telescoping identity by checking that the recurrence holds at multiple
/// evaluation points up to ``max_k``.
///
/// Parameters
/// ----------
/// upper : list[tuple[int, int, int]]
///     Upper parameters. Each is ``(num, den, power)`` representing `(num/den) * q^power`.
/// lower : list[tuple[int, int, int]]
///     Lower parameters, same tuple format.
/// z_num : int
///     Numerator of the argument z coefficient.
/// z_den : int
///     Denominator of the argument z coefficient.
/// z_pow : int
///     Power of q in the argument z.
/// n_val : int
///     Concrete value of the summation parameter n.
/// q_num : int
///     Numerator of the concrete q value.
/// q_den : int
///     Denominator of the concrete q value.
/// max_order : int
///     Maximum recurrence order to search (passed to q_zeilberger).
/// max_k : int
///     Number of evaluation points for certificate verification.
/// n_param_indices : list[int] or None
///     Manual override: indices into upper parameters that depend on n.
///     If None, auto-detected via ``detect_n_params``.
/// n_is_in_argument : bool or None
///     Manual override: whether n appears in the argument z.
///     If None, auto-detected via ``detect_n_params``.
///
/// Returns
/// -------
/// dict
///     If verification succeeded: ``{"verified": True, "order": int,
///     "coefficients": list[Fraction], "certificate": str}``.
///     If no recurrence found: ``{"verified": False, "reason": "no recurrence found"}``.
///     If verification failed: ``{"verified": False, "reason": "verification failed"}``.
///
/// Examples
/// --------
/// >>> from q_kangaroo import verify_wz
/// >>> # Verify q-Vandermonde at n=5, q=2
/// >>> result = verify_wz([(1,1,-5), (1,1,2)], [(1,1,3)], 1, 1, 4, 5, 2, 1, 3, 20)
/// >>> result["verified"]
/// True
///
/// See Also
/// --------
/// q_zeilberger : Find the recurrence and certificate.
/// q_gosper : Indefinite q-hypergeometric summation.
#[pyfunction]
#[pyo3(name = "verify_wz", signature = (upper, lower, z_num, z_den, z_pow, n_val, q_num, q_den, max_order, max_k, n_param_indices=None, n_is_in_argument=None))]
pub fn verify_wz_fn(
    py: Python<'_>,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64,
    z_den: i64,
    z_pow: i64,
    n_val: i64,
    q_num: i64,
    q_den: i64,
    max_order: usize,
    max_k: usize,
    n_param_indices: Option<Vec<usize>>,
    n_is_in_argument: Option<bool>,
) -> PyResult<PyObject> {
    let series = HypergeometricSeries {
        upper: parse_qmonomials(upper),
        lower: parse_qmonomials(lower),
        argument: QMonomial::new(QRat::from((z_num, z_den)), z_pow),
    };
    let q_val = QRat::from((q_num, q_den));

    // Auto-detect or use manual overrides
    let (indices, in_arg) = match (&n_param_indices, &n_is_in_argument) {
        (Some(idx), Some(flag)) => (idx.clone(), *flag),
        _ => {
            let (auto_idx, auto_flag) = detect_n_params(&series, n_val, &q_val);
            (
                n_param_indices.unwrap_or(auto_idx),
                n_is_in_argument.unwrap_or(auto_flag),
            )
        }
    };

    // First, run q_zeilberger to get the recurrence and certificate
    let zeil_result = q_zeilberger(&series, n_val, &q_val, max_order, &indices, in_arg);

    let dict = PyDict::new(py);
    match zeil_result {
        QZeilbergerResult::Recurrence(zr) => {
            let verified = verify_wz_certificate(
                &series, n_val, &q_val, &zr.coefficients, &zr.certificate,
                &indices, in_arg, max_k,
            );
            dict.set_item("verified", verified)?;
            if verified {
                dict.set_item("order", zr.order)?;
                dict.set_item("coefficients", qrat_vec_to_pylist(py, &zr.coefficients)?)?;
                dict.set_item("certificate", format!("{}", zr.certificate))?;
            } else {
                dict.set_item("reason", "verification failed")?;
            }
        }
        QZeilbergerResult::NoRecurrence => {
            dict.set_item("verified", false)?;
            dict.set_item("reason", "no recurrence found")?;
        }
    }
    Ok(dict.into())
}

/// Solve a q-hypergeometric recurrence for closed-form solutions.
///
/// Given recurrence coefficients ``c_0*S(n) + c_1*S(n+1) + ... + c_d*S(n+d) = 0``
/// (typically from :func:`q_zeilberger`), finds all q-hypergeometric solutions
/// and optionally decomposes them into q-Pochhammer product forms.
///
/// Parameters
/// ----------
/// coefficients : list[tuple[int, int]]
///     Recurrence coefficients as ``(numerator, denominator)`` pairs representing
///     exact rational values. Must have at least 2 entries (order >= 1).
/// q_num : int
///     Numerator of the concrete q value.
/// q_den : int
///     Denominator of the concrete q value.
///
/// Returns
/// -------
/// list[dict]
///     Each dict represents one solution with keys:
///     - ``"ratio"``: Fraction -- the ratio y(n+1)/y(n).
///     - ``"has_closed_form"``: bool -- whether a Pochhammer decomposition was found.
///     - ``"scalar"``: Fraction (if has_closed_form) -- scalar prefactor.
///     - ``"q_power_coeff"``: int (if has_closed_form) -- coefficient in q^{c*n*(n-1)/2}.
///     - ``"numer_factors"``: list[tuple[str, int]] (if has_closed_form) --
///       numerator Pochhammer factors as (coeff_string, power) tuples.
///     - ``"denom_factors"``: list[tuple[str, int]] (if has_closed_form) --
///       denominator Pochhammer factors, same format.
///
/// Examples
/// --------
/// >>> from q_kangaroo import q_zeilberger, q_petkovsek
/// >>> result = q_zeilberger([(1,1,-5), (1,1,2)], [(1,1,3)], 1, 1, 4, 5, 2, 1, 3)
/// >>> if result["found"]:
/// ...     coeffs = [(c.numerator, c.denominator) for c in result["coefficients"]]
/// ...     solutions = q_petkovsek(coeffs, 2, 1)
/// ...     len(solutions) >= 1
/// True
///
/// See Also
/// --------
/// q_zeilberger : Find recurrence coefficients from a q-hypergeometric sum.
/// q_gosper : Indefinite q-hypergeometric summation.
#[pyfunction]
#[pyo3(name = "q_petkovsek", signature = (coefficients, q_num, q_den))]
pub fn q_petkovsek_fn(
    py: Python<'_>,
    coefficients: Vec<(i64, i64)>,
    q_num: i64,
    q_den: i64,
) -> PyResult<PyObject> {
    let coeffs: Vec<QRat> = coefficients
        .iter()
        .map(|(n, d)| QRat::from((*n, *d)))
        .collect();
    let q_val = QRat::from((q_num, q_den));

    let results = q_petkovsek(&coeffs, &q_val);

    let items: Vec<PyObject> = results
        .iter()
        .map(|r| {
            let dict = PyDict::new(py);
            dict.set_item("ratio", qrat_to_python(py, &r.ratio)?)?;
            let has_cf = r.closed_form.is_some();
            dict.set_item("has_closed_form", has_cf)?;
            if let Some(ref cf) = r.closed_form {
                dict.set_item("scalar", qrat_to_python(py, &cf.scalar)?)?;
                dict.set_item("q_power_coeff", cf.q_power_coeff)?;
                let numer: Vec<(String, i64)> = cf.numer_factors.iter()
                    .map(|m| (format!("{}", m.coeff), m.power))
                    .collect();
                dict.set_item("numer_factors", numer)?;
                let denom: Vec<(String, i64)> = cf.denom_factors.iter()
                    .map(|m| (format!("{}", m.coeff), m.power))
                    .collect();
                dict.set_item("denom_factors", denom)?;
            }
            Ok(dict.into())
        })
        .collect::<PyResult<_>>()?;

    Ok(PyList::new(py, &items)?.into())
}

// ===========================================================================
// GROUP 13: Identity Proving Extensions
// ===========================================================================

/// Raise a QRat to a signed integer power via repeated squaring.
/// Duplicated locally (private in gosper.rs, zeilberger.rs, nonterminating.rs).
fn qrat_pow_i64(base: &QRat, exp: i64) -> QRat {
    if exp == 0 {
        return QRat::one();
    }
    if exp > 0 {
        qrat_pow_u32(base, exp as u32)
    } else {
        assert!(
            !base.is_zero(),
            "qrat_pow_i64: zero base with negative exponent"
        );
        let positive = qrat_pow_u32(base, (-exp) as u32);
        &QRat::one() / &positive
    }
}

/// Raise a QRat to a u32 power via repeated squaring.
fn qrat_pow_u32(base: &QRat, exp: u32) -> QRat {
    if exp == 0 {
        return QRat::one();
    }
    if exp == 1 {
        return base.clone();
    }
    let mut result = QRat::one();
    let mut b = base.clone();
    let mut e = exp;
    while e > 0 {
        if e & 1 == 1 {
            result = &result * &b;
        }
        e >>= 1;
        if e > 0 {
            b = &b * &b;
        }
    }
    result
}

/// Compute (q^base_power; q)_n at a concrete q value.
///
/// Returns the product prod_{k=0}^{n-1} (1 - q^{base_power} * q^k)
/// = prod_{k=0}^{n-1} (1 - q^{base_power + k}).
fn pochhammer_scalar_val(q_val: &QRat, base_power: i64, n: i64) -> QRat {
    if n <= 0 {
        return QRat::one();
    }
    let mut result = QRat::one();
    for k in 0..n {
        let power = base_power + k;
        let q_pow = qrat_pow_i64(q_val, power);
        result = &result * &(&QRat::one() - &q_pow);
    }
    result
}

/// Prove a nonterminating q-hypergeometric identity via Chen-Hou-Mu method.
///
/// Uses parameter specialization to reduce a nonterminating identity to
/// a family of terminating ones, then proves both sides satisfy the same
/// recurrence via q-Zeilberger and checks initial conditions.
///
/// The LHS is a hypergeometric series with n-independent upper parameters
/// ``upper_fixed``, one n-dependent upper parameter $q^{\text{n\_param\_offset} - n}$,
/// lower parameters ``lower``, and argument $q^{\text{z\_pow\_offset} + n}$.
///
/// The RHS is a ratio of q-Pochhammer symbols at concrete q:
/// $\frac{\prod_b (q^b; q)_n}{\prod_b (q^b; q)_n}$ where numerator/denominator
/// bases are given by ``rhs_numer_bases`` and ``rhs_denom_bases``.
///
/// Parameters
/// ----------
/// upper_fixed : list[tuple[int, int, int]]
///     n-independent upper parameters as ``(coeff_num, coeff_den, power)`` triples.
/// n_param_offset : int
///     The n-dependent upper parameter is $q^{\text{offset} - n}$.
/// lower : list[tuple[int, int, int]]
///     Lower parameters as ``(coeff_num, coeff_den, power)`` triples.
/// z_pow_offset : int
///     The argument is $q^{\text{offset} + n}$.
/// rhs_numer_bases : list[int]
///     Each base $b$ contributes $(q^b; q)_n$ to the RHS numerator.
/// rhs_denom_bases : list[int]
///     Each base $b$ contributes $(q^b; q)_n$ to the RHS denominator.
/// q_num : int
///     Numerator of the concrete $q$ value.
/// q_den : int
///     Denominator of the concrete $q$ value.
/// n_test : int
///     Test value of $n$ for parameter specialization ($\ge 5$ recommended).
/// max_order : int
///     Maximum recurrence order to search via q-Zeilberger.
///
/// Returns
/// -------
/// dict
///     On success: ``{"proved": True, "recurrence_order": int,
///     "coefficients": list[Fraction], "initial_conditions_checked": int}``.
///     On failure: ``{"proved": False, "reason": str}``.
///
/// Examples
/// --------
/// >>> from q_kangaroo import prove_nonterminating
/// >>> # Prove the q-Gauss summation (nonterminating form):
/// >>> # 2phi1(a, b; c; q, c/(ab)) = (c/a; q)_inf * (c/b; q)_inf / ((c; q)_inf * (c/(ab); q)_inf)
/// >>> # Specialized: upper_fixed=[(1,1,1)], n_param_offset=2, lower=[(1,1,3)],
/// >>> # z_pow_offset=2, rhs_numer_bases=[2, 1], rhs_denom_bases=[3, 0]
/// >>> result = prove_nonterminating(
/// ...     [(1, 1, 1)], 2, [(1, 1, 3)], 2,
/// ...     [2, 1], [3, 0], 2, 1, 5, 3)
/// >>> result["proved"]
/// True
///
/// See Also
/// --------
/// q_zeilberger : Find recurrence via creative telescoping.
/// q_gosper : Indefinite q-hypergeometric summation.
#[pyfunction]
#[pyo3(name = "prove_nonterminating", signature = (upper_fixed, n_param_offset, lower, z_pow_offset, rhs_numer_bases, rhs_denom_bases, q_num, q_den, n_test, max_order))]
pub fn prove_nonterminating_fn(
    py: Python<'_>,
    upper_fixed: Vec<(i64, i64, i64)>,
    n_param_offset: i64,
    lower: Vec<(i64, i64, i64)>,
    z_pow_offset: i64,
    rhs_numer_bases: Vec<i64>,
    rhs_denom_bases: Vec<i64>,
    q_num: i64,
    q_den: i64,
    n_test: i64,
    max_order: usize,
) -> PyResult<PyObject> {
    let upper_fixed_qm = parse_qmonomials(upper_fixed);
    let lower_qm = parse_qmonomials(lower);
    let q_val = QRat::from((q_num, q_den));

    // Clone what we need for the closures
    let q_val_lhs = q_val.clone();
    let rhs_numer = rhs_numer_bases.clone();
    let rhs_denom = rhs_denom_bases.clone();

    let lhs_builder = move |n: i64| -> HypergeometricSeries {
        let mut upper = upper_fixed_qm.clone();
        upper.push(QMonomial::q_power(n_param_offset - n));
        HypergeometricSeries {
            upper,
            lower: lower_qm.clone(),
            argument: QMonomial::q_power(z_pow_offset + n),
        }
    };

    let rhs_builder = move |n: i64| -> QRat {
        if n == 0 {
            return QRat::one();
        }
        let mut numer = QRat::one();
        for &base in &rhs_numer {
            numer = &numer * &pochhammer_scalar_val(&q_val_lhs, base, n);
        }
        let mut denom = QRat::one();
        for &base in &rhs_denom {
            denom = &denom * &pochhammer_scalar_val(&q_val_lhs, base, n);
        }
        if denom.is_zero() {
            return QRat::zero();
        }
        &numer / &denom
    };

    let result = prove_nonterminating(&lhs_builder, &rhs_builder, &q_val, n_test, max_order);

    let dict = PyDict::new(py);
    match result {
        NonterminatingProofResult::Proved {
            recurrence_order,
            recurrence_coefficients,
            initial_conditions_checked,
        } => {
            dict.set_item("proved", true)?;
            dict.set_item("recurrence_order", recurrence_order)?;
            dict.set_item(
                "coefficients",
                qrat_vec_to_pylist(py, &recurrence_coefficients)?,
            )?;
            dict.set_item("initial_conditions_checked", initial_conditions_checked)?;
        }
        NonterminatingProofResult::Failed { reason } => {
            dict.set_item("proved", false)?;
            dict.set_item("reason", reason)?;
        }
    }
    Ok(dict.into())
}

/// Search for a chain of transformations between two hypergeometric series.
///
/// Uses breadth-first search over the catalog of known transformations
/// (Heine 1/2/3, Sears, Watson) to find a sequence connecting the source
/// series to the target series, with accumulated prefactors.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session (needed for symbolic variable lookup).
/// source_upper : list[tuple[int, int, int]]
///     Source series upper parameters as ``(coeff_num, coeff_den, power)`` triples.
/// source_lower : list[tuple[int, int, int]]
///     Source series lower parameters.
/// source_z_num : int
///     Source argument numerator coefficient.
/// source_z_den : int
///     Source argument denominator coefficient.
/// source_z_pow : int
///     Source argument power of $q$.
/// target_upper : list[tuple[int, int, int]]
///     Target series upper parameters.
/// target_lower : list[tuple[int, int, int]]
///     Target series lower parameters.
/// target_z_num : int
///     Target argument numerator coefficient.
/// target_z_den : int
///     Target argument denominator coefficient.
/// target_z_pow : int
///     Target argument power of $q$.
/// max_depth : int
///     Maximum chain length (BFS depth bound).
/// order : int
///     Truncation order for FPS comparison.
///
/// Returns
/// -------
/// dict
///     On success: ``{"found": True, "steps": list[dict], "total_prefactor": QSeries}``
///     where each step dict has ``"name"`` (str) and ``"prefactor"`` (QSeries).
///     On failure: ``{"found": False, "max_depth": int}``.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, find_transformation_chain
/// >>> s = QSession()
/// >>> # Search for a chain from 2phi1(a,b;c;q,z) to a transformed form
/// >>> result = find_transformation_chain(
/// ...     s,
/// ...     [(1,1,1), (1,1,2)], [(1,1,3)], 1, 1, 1,  # source
/// ...     [(1,1,3), (1,1,1)], [(1,1,2)], 1, 1, 1,   # target
/// ...     3, 20)
///
/// See Also
/// --------
/// heine1 : Heine's first transformation.
/// heine2 : Heine's second transformation.
/// heine3 : Heine's third transformation.
#[pyfunction]
#[pyo3(name = "find_transformation_chain", signature = (session, source_upper, source_lower, source_z_num, source_z_den, source_z_pow, target_upper, target_lower, target_z_num, target_z_den, target_z_pow, max_depth, order))]
pub fn find_transformation_chain_fn(
    py: Python<'_>,
    session: &QSession,
    source_upper: Vec<(i64, i64, i64)>,
    source_lower: Vec<(i64, i64, i64)>,
    source_z_num: i64,
    source_z_den: i64,
    source_z_pow: i64,
    target_upper: Vec<(i64, i64, i64)>,
    target_lower: Vec<(i64, i64, i64)>,
    target_z_num: i64,
    target_z_den: i64,
    target_z_pow: i64,
    max_depth: usize,
    order: i64,
) -> PyResult<PyObject> {
    // Lock session to get SymbolId for q, then drop the lock
    let variable = {
        let mut inner = session.inner.lock().unwrap();
        inner.get_or_create_symbol_id("q")
    };

    let source = HypergeometricSeries {
        upper: parse_qmonomials(source_upper),
        lower: parse_qmonomials(source_lower),
        argument: QMonomial::new(QRat::from((source_z_num, source_z_den)), source_z_pow),
    };

    let target = HypergeometricSeries {
        upper: parse_qmonomials(target_upper),
        lower: parse_qmonomials(target_lower),
        argument: QMonomial::new(QRat::from((target_z_num, target_z_den)), target_z_pow),
    };

    let result = find_transformation_chain(&source, &target, max_depth, variable, order);

    let dict = PyDict::new(py);
    match result {
        TransformationChainResult::Found {
            steps,
            total_prefactor,
        } => {
            dict.set_item("found", true)?;

            let step_list: Vec<PyObject> = steps
                .iter()
                .map(|step| {
                    let step_dict = PyDict::new(py);
                    step_dict.set_item("name", &step.name)?;
                    let prefactor_series = QSeries {
                        fps: step.step_prefactor.clone(),
                    };
                    let prefactor_obj = prefactor_series.into_pyobject(py)?;
                    step_dict.set_item("prefactor", prefactor_obj)?;

                    // Represent result_series as parameter lists
                    let upper_repr: Vec<(String, i64)> = step
                        .result_series
                        .upper
                        .iter()
                        .map(|m| (format!("{}", m.coeff), m.power))
                        .collect();
                    let lower_repr: Vec<(String, i64)> = step
                        .result_series
                        .lower
                        .iter()
                        .map(|m| (format!("{}", m.coeff), m.power))
                        .collect();
                    let arg_repr = (
                        format!("{}", step.result_series.argument.coeff),
                        step.result_series.argument.power,
                    );
                    step_dict.set_item("upper", upper_repr)?;
                    step_dict.set_item("lower", lower_repr)?;
                    step_dict.set_item("argument", arg_repr)?;

                    Ok(step_dict.into())
                })
                .collect::<PyResult<_>>()?;

            dict.set_item("steps", PyList::new(py, &step_list)?)?;

            let total_pf = QSeries {
                fps: total_prefactor,
            };
            let total_pf_obj = total_pf.into_pyobject(py)?;
            dict.set_item("total_prefactor", total_pf_obj)?;
        }
        TransformationChainResult::NotFound { max_depth } => {
            dict.set_item("found", false)?;
            dict.set_item("max_depth", max_depth)?;
        }
    }
    Ok(dict.into())
}
