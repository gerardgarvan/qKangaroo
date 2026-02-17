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
    find_transformation_chain, TransformationChainResult,
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
/// The infinite product $(q;q)_\infty = \prod_{k=1}^{\infty}(1-q^k)$ (Euler function):
///
/// >>> from q_kangaroo import QSession, aqprod
/// >>> s = QSession()
/// >>> euler = aqprod(s, 1, 1, 1, None, 20)
/// >>> print(euler)  # 1 - q - q^2 + q^5 + q^7 - q^12 - q^15 + O(q^20)
///
/// The finite product $(q;q)_5 = (1-q)(1-q^2)(1-q^3)(1-q^4)(1-q^5)$:
///
/// >>> finite = aqprod(s, 1, 1, 1, 5, 20)
/// >>> print(finite)  # 1 - q - q^2 + q^5 + q^6 + q^7 - q^8 - q^9 - q^10 + q^13 + q^14 - q^15 + O(q^20)
///
/// The partition generating function is the reciprocal of $(q;q)_\infty$:
///
/// >>> from q_kangaroo import partition_gf
/// >>> pgf = partition_gf(s, 20)
/// >>> print(pgf)  # 1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + ...
///
/// Notes
/// -----
/// The q-Pochhammer symbol supports three regimes:
///
/// - **Finite** ($n > 0$): $(a;q)_n = \prod_{k=0}^{n-1}(1 - aq^k)$.
/// - **Infinite** ($n = \text{None}$): $(a;q)_\infty = \prod_{k=0}^{\infty}(1 - aq^k)$.
///   The fundamental building block: $(q;q)_\infty = \prod_{k=1}^{\infty}(1-q^k)$ is
///   Euler's function, whose reciprocal generates the partition numbers.
/// - **Negative** ($n < 0$): $(a;q)_{-n} = 1 / (aq^{-n};q)_n$, the shifted inversion.
///
/// See Also
/// --------
/// partition_gf : Partition generating function $1/(q;q)_\infty$.
/// etaq : Generalized eta product $(q^b; q^t)_\infty$.
/// qbin : q-binomial coefficient using q-Pochhammer symbols.
/// theta3 : Jacobi theta function (uses q-Pochhammer internally).
/// prodmake : Identify a series as an infinite product.
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
/// The Gaussian polynomial $\binom{4}{2}_q$:
///
/// >>> from q_kangaroo import QSession, qbin
/// >>> s = QSession()
/// >>> b = qbin(s, 4, 2, 20)
/// >>> print(b)  # 1 + q + 2*q^2 + q^3 + q^4 + O(q^20)
///
/// Symmetry: $\binom{n}{k}_q = \binom{n}{n-k}_q$:
///
/// >>> b52 = qbin(s, 5, 2, 20)
/// >>> b53 = qbin(s, 5, 3, 20)
/// >>> print(b52)  # 1 + q + 2*q^2 + 2*q^3 + 2*q^4 + q^5 + q^6 + O(q^20)
/// >>> # b53 gives the same polynomial, confirming symmetry
///
/// Notes
/// -----
/// The q-binomial coefficient $\binom{n}{k}_q$ (also called the Gaussian binomial
/// coefficient or Gaussian polynomial) counts the number of $k$-dimensional
/// subspaces of an $n$-dimensional vector space over $\mathbb{F}_q$. In the
/// classical limit, $\lim_{q \to 1} \binom{n}{k}_q = \binom{n}{k}$.
///
/// See Also
/// --------
/// aqprod : General q-Pochhammer symbol.
/// phi : Basic hypergeometric series (q-binomials appear in hypergeometric parameters).
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
/// The Euler function $(q;q)_\infty$:
///
/// >>> from q_kangaroo import QSession, etaq
/// >>> s = QSession()
/// >>> euler = etaq(s, 1, 1, 20)
/// >>> print(euler)  # 1 - q - q^2 + q^5 + q^7 - q^12 - q^15 + O(q^20)
///
/// The product $(q;q^5)_\infty$, which appears in Rogers-Ramanujan identities:
///
/// >>> rr1 = etaq(s, 1, 5, 30)
/// >>> print(rr1)  # 1 - q - q^6 + q^7 - q^11 + q^12 - q^16 + 2*q^17 - ...
///
/// The partition generating function is $1/(q;q)_\infty = 1/\text{etaq}(1,1,N)$:
///
/// >>> from q_kangaroo import partition_gf
/// >>> pgf = partition_gf(s, 20)
/// >>> print(pgf)  # 1 + q + 2*q^2 + 3*q^3 + 5*q^4 + ...
///
/// Notes
/// -----
/// The parameters $b$ (base) and $t$ (step) define the product
/// $(q^b; q^t)_\infty = \prod_{k=0}^{\infty}(1 - q^{b+kt})$.
/// The Dedekind eta function is $\eta(\tau) = q^{1/24}(q;q)_\infty$ where
/// $q = e^{2\pi i \tau}$, so ``etaq(1, 1, N)`` computes $(q;q)_\infty$
/// (the Euler function) which is $q^{-1/24}\eta(\tau)$.
///
/// See Also
/// --------
/// aqprod : General q-Pochhammer symbol $(a;q)_n$.
/// jacprod : Jacobi triple product JAC(a, b).
/// partition_gf : Partition generating function $1/(q;q)_\infty$.
/// prodmake : Identify a series as an infinite product.
/// etamake : Identify a series as an eta quotient.
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
/// JAC(1,5) appears in the Rogers-Ramanujan identities:
///
/// >>> from q_kangaroo import QSession, jacprod
/// >>> s = QSession()
/// >>> j15 = jacprod(s, 1, 5, 30)
/// >>> print(j15)  # 1 - q - q^4 + q^7 + q^13 - q^18 - q^27 + O(q^30)
///
/// JAC(2,5) is the companion product in the second Rogers-Ramanujan identity:
///
/// >>> j25 = jacprod(s, 2, 5, 30)
/// >>> print(j25)  # 1 - q^2 - q^3 + q^9 + q^11 - q^21 - q^24 + O(q^30)
///
/// Notes
/// -----
/// The Jacobi triple product identity states
/// $J(z, q) = (z; q)_\infty (q/z; q)_\infty (q; q)_\infty
///   = \sum_{n=-\infty}^{\infty} (-1)^n z^n q^{n(n-1)/2}$.
/// In the ``jacprod(a, b)`` notation, we set $z = q^a$ and use step $q^b$,
/// giving $\text{JAC}(a,b) = (q^a; q^b)_\infty (q^{b-a}; q^b)_\infty (q^b; q^b)_\infty$.
///
/// See Also
/// --------
/// etaq : Generalized eta product.
/// tripleprod : Jacobi triple product with monomial parameter.
/// theta3 : Jacobi theta function (expressible via triple products).
/// jacprodmake : Identify a series as a Jacobi product quotient.
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
/// With $z = -1$: the product $(-1;q)_\infty \cdot (-q;q)_\infty \cdot (q;q)_\infty$
/// yields a sum over triangular numbers:
///
/// >>> from q_kangaroo import QSession, tripleprod
/// >>> s = QSession()
/// >>> tp = tripleprod(s, -1, 1, 0, 20)  # z = -1
/// >>> print(tp)  # 2 + 2*q + 2*q^3 + 2*q^6 + 2*q^10 + 2*q^15 + O(q^20)
///
/// Note: $z = q$ (``power=1, coeff=1``) gives 0 because the factor
/// $(q/z; q)_\infty = (1; q)_\infty$ vanishes. Use ``jacprod`` for the standard
/// integer-exponent triple products.
///
/// See Also
/// --------
/// jacprod : Jacobi triple product JAC(a, b) with integer parameters.
/// quinprod : Quintuple product identity.
/// etaq : Generalized eta product.
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
/// With $z = -1$, the quintuple product gives twice the Euler function:
///
/// >>> from q_kangaroo import QSession, quinprod
/// >>> s = QSession()
/// >>> qp = quinprod(s, -1, 1, 0, 20)  # z = -1
/// >>> print(qp)  # 2 - 2*q - 2*q^2 + 2*q^5 + 2*q^7 - 2*q^12 - 2*q^15 + O(q^20)
///
/// This is $2 \cdot (q;q)_\infty$ -- the pentagonal number coefficients scaled by 2.
///
/// Notes
/// -----
/// The quintuple product identity is
/// $\prod_{n \ge 1}(1-q^n)(1-zq^n)(1-z^{-1}q^{n-1})(1-z^2q^{2n-1})(1-z^{-2}q^{2n-1})
///   = \sum_{n=-\infty}^{\infty}(z^{3n} - z^{-3n-1})q^{n(3n+1)/2}$.
/// Many standard parameter choices (e.g., $z = q$) lead to vanishing factors;
/// use $z = -1$ or non-integer monomials for non-trivial output.
///
/// See Also
/// --------
/// tripleprod : Jacobi triple product.
/// jacprod : Jacobi triple product JAC(a, b).
/// winquist : Winquist's identity product.
/// etaq : Generalized eta product.
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
/// The product is $(q;q)_\infty^2$ multiplied by 8 q-Pochhammer factors involving
/// $a$, $a^{-1}$, $b$, $b^{-1}$, $ab$, $(ab)^{-1}$, $a/b$, and $b/a$.
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
/// Many integer-monomial specializations vanish because one of the 8 factors
/// becomes $(1; q)_\infty = 0$. Use non-unit coefficients for non-trivial output:
///
/// >>> from q_kangaroo import QSession, winquist
/// >>> s = QSession()
/// >>> w = winquist(s, 1, 1, 1, 1, 1, 2, 20)
/// >>> # This is identically zero (a degenerate specialization)
///
/// Notes
/// -----
/// Winquist's identity (1969) expresses $(q;q)_{\\infty}^{10}$ as a double sum,
/// which Winquist used to give an elementary proof of Ramanujan's congruence
/// $p(11n+6) \\equiv 0 \\pmod{11}$. The identity relates an infinite product
/// involving two parameters $a$ and $b$ to
/// $\sum_{j,k} (-1)^{j+k} (a^{3j} b^{3k} - a^{-3j-1} b^{3k}) q^{(3j^2+j+3k^2+k)/2}$.
///
/// See Also
/// --------
/// quinprod : Quintuple product identity.
/// etaq : Generalized eta product.
/// findcong : Discover partition congruences (Winquist proved $p(11n+6) \equiv 0$).
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
/// The series is in $q^{1/4}$, so the exponents are multiples of 1/4 in $q$:
///
/// >>> from q_kangaroo import QSession, theta2
/// >>> s = QSession()
/// >>> t2 = theta2(s, 40)
/// >>> print(t2)  # 2*q + 2*q^9 + 2*q^25 + O(q^40)
///
/// Here the variable is $X = q^{1/4}$, so ``2*q`` means $2 X^1 = 2q^{1/4}$,
/// and ``2*q^9`` means $2 X^9 = 2q^{9/4}$.
///
/// Notes
/// -----
/// The second Jacobi theta function. In the $q^{1/4}$ convention used here,
/// $\theta_2(q) = 2q^{1/4}\sum_{n=0}^{\infty} q^{n(n+1)}$. The variable in
/// the returned series represents $X = q^{1/4}$, so the constant term is 0
/// and the $X^1$ coefficient is 2. The three Jacobi theta functions satisfy
/// the identity $\theta_2(q)^4 + \theta_4(q)^4 = \theta_3(q)^4$.
///
/// See Also
/// --------
/// theta3 : Jacobi theta function $\theta_3(q)$.
/// theta4 : Jacobi theta function $\theta_4(q)$.
/// etaq : Theta functions factor through eta products internally.
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
/// The nonzero coefficients occur at perfect squares:
///
/// >>> from q_kangaroo import QSession, theta3
/// >>> s = QSession()
/// >>> t3 = theta3(s, 20)
/// >>> print(t3)  # 1 + 2*q + 2*q^4 + 2*q^9 + 2*q^16 + O(q^20)
///
/// The coefficient of $q^n$ in $\theta_3(q)^2$ counts the number of
/// representations of $n$ as a sum of two squares $r_2(n)$.
///
/// Notes
/// -----
/// $\theta_3(q) = \sum_{n=-\infty}^{\infty} q^{n^2} = 1 + 2\sum_{n=1}^{\infty} q^{n^2}$.
/// The coefficient of $q^n$ in $\theta_3(q)^k$ counts the number of
/// representations of $n$ as a sum of $k$ squares (with signs and order).
/// Satisfies the Jacobi identity $\theta_3(q)^4 = \theta_2(q)^4 + \theta_4(q)^4$.
///
/// See Also
/// --------
/// theta2 : Jacobi theta function $\theta_2(q)$.
/// theta4 : Jacobi theta function $\theta_4(q)$.
/// aqprod : Product representation of theta functions.
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
/// The alternating version of $\theta_3$:
///
/// >>> from q_kangaroo import QSession, theta4
/// >>> s = QSession()
/// >>> t4 = theta4(s, 20)
/// >>> print(t4)  # 1 - 2*q + 2*q^4 - 2*q^9 + 2*q^16 + O(q^20)
///
/// Notes
/// -----
/// $\theta_4(q) = \sum_{n=-\infty}^{\infty} (-1)^n q^{n^2}$. This is the
/// alternating version of $\theta_3$. Has the product form
/// $\theta_4(q) = (q;q^2)_\infty^2 \cdot (q^2;q^2)_\infty$.
///
/// See Also
/// --------
/// theta2 : Jacobi theta function $\theta_2(q)$.
/// theta3 : Jacobi theta function $\theta_3(q)$.
/// aqprod : Product representation of theta functions.
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
/// >>> partition_count(0)
/// 1
/// >>> partition_count(5)
/// 7
/// >>> partition_count(100)
/// 190569292536040
///
/// Ramanujan's congruence $p(5n+4) \equiv 0 \pmod{5}$:
///
/// >>> partition_count(4)    # p(5*0+4) = 5, divisible by 5
/// 5
/// >>> partition_count(9)    # p(5*1+4) = 30, divisible by 5
/// 30
/// >>> partition_count(14)   # p(5*2+4) = 135, divisible by 5
/// 135
///
/// Notes
/// -----
/// Computed via Euler's pentagonal recurrence with $O(n\sqrt{n})$ complexity.
/// Ramanujan discovered the celebrated congruences:
///
/// - $p(5n+4) \equiv 0 \pmod{5}$
/// - $p(7n+5) \equiv 0 \pmod{7}$
/// - $p(11n+6) \equiv 0 \pmod{11}$
///
/// Use ``sift`` and ``findcong`` to discover and verify such congruences
/// computationally.
///
/// See Also
/// --------
/// partition_gf : Generating function $\sum p(n) q^n$.
/// sift : Extract arithmetic subsequences from a series.
/// findcong : Discover congruences in series coefficients.
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
/// >>> pgf = partition_gf(s, 20)
/// >>> print(pgf)
/// >>> # 1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + 11*q^6 + 15*q^7 + 22*q^8 + ...
///
/// The coefficients are the partition numbers: $p(0)=1, p(1)=1, p(2)=2, p(3)=3, p(4)=5, \ldots$
///
/// Notes
/// -----
/// $\sum_{n=0}^{\infty} p(n)q^n = \prod_{k=1}^{\infty} \frac{1}{1-q^k}
///   = \frac{1}{(q;q)_\infty}$.
/// This is one of the most important generating functions in combinatorics.
/// The connection to ``etaq`` is: ``partition_gf(s, N)`` equals the series
/// inverse of ``etaq(s, 1, 1, N)``.
///
/// See Also
/// --------
/// partition_count : Compute $p(n)$ directly.
/// distinct_parts_gf : Partitions into distinct parts.
/// odd_parts_gf : Partitions into odd parts.
/// etaq : Euler function $(q;q)_\infty$ (reciprocal of partition_gf).
/// aqprod : General q-Pochhammer symbol.
/// sift : Extract arithmetic subsequences for congruence analysis.
/// findcong : Discover partition congruences.
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
/// >>> print(dpgf)
/// >>> # 1 + q + q^2 + 2*q^3 + 2*q^4 + 3*q^5 + 4*q^6 + 5*q^7 + 6*q^8 + ...
///
/// By Euler's partition theorem, this equals odd_parts_gf:
///
/// >>> opgf = odd_parts_gf(s, 20)
/// >>> # dpgf and opgf have identical coefficients
///
/// Notes
/// -----
/// $\prod_{k=1}^{\infty}(1+q^k) = \frac{(q^2;q^2)_\infty}{(q;q)_\infty}$.
/// By Euler's partition theorem, the number of partitions of $n$ into distinct
/// parts equals the number of partitions of $n$ into odd parts. Thus this
/// generating function equals ``odd_parts_gf``.
///
/// See Also
/// --------
/// partition_gf : Unrestricted partition generating function.
/// odd_parts_gf : Partitions into odd parts (equal by Euler's theorem).
/// etaq : Eta products (distinct-parts = $(q^2;q^2)/(q;q)$).
/// mprodmake : Identify a product of $(1+q^n)$ factors.
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
/// >>> print(opgf)
/// >>> # 1 + q + q^2 + 2*q^3 + 2*q^4 + 3*q^5 + 4*q^6 + 5*q^7 + 6*q^8 + ...
///
/// The output is identical to distinct_parts_gf, verifying Euler's theorem.
///
/// Notes
/// -----
/// Euler's partition theorem (1748): the number of partitions of $n$ into
/// odd parts equals the number of partitions of $n$ into distinct parts.
/// The generating function is
/// $\prod_{k=0}^{\infty}\frac{1}{1-q^{2k+1}} = \frac{1}{(q;q^2)_\infty}$.
///
/// See Also
/// --------
/// distinct_parts_gf : Partitions into distinct parts (equal by Euler's theorem).
/// partition_gf : Unrestricted partition generating function.
/// etaq : Eta products.
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
/// Partitions with parts at most 3 (i.e., using only 1s, 2s, and 3s):
///
/// >>> from q_kangaroo import QSession, bounded_parts_gf
/// >>> s = QSession()
/// >>> bp = bounded_parts_gf(s, 3, 20)
/// >>> print(bp)
/// >>> # 1 + q + 2*q^2 + 3*q^3 + 4*q^4 + 5*q^5 + 7*q^6 + 8*q^7 + 10*q^8 + ...
///
/// For example, $q^6$ has coefficient 7: the partitions of 6 into parts $\le 3$
/// are 3+3, 3+2+1, 3+1+1+1, 2+2+2, 2+2+1+1, 2+1+1+1+1, 1+1+1+1+1+1.
///
/// Notes
/// -----
/// $\prod_{k=1}^{m}\frac{1}{1-q^k}$ counts partitions with largest part at
/// most $m$, or equivalently, partitions with at most $m$ parts (by
/// conjugation). As $m \to \infty$, this converges to ``partition_gf``.
///
/// See Also
/// --------
/// partition_gf : Unrestricted partition generating function.
/// qbin : Gaussian binomial (related to bounded partitions).
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
/// At $z = 1$, the rank generating function reduces to the partition generating
/// function (the rank refinement sums back to the total count):
///
/// >>> from q_kangaroo import QSession, rank_gf
/// >>> s = QSession()
/// >>> r = rank_gf(s, 1, 1, 20)
/// >>> print(r)
/// >>> # 1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + 11*q^6 + ...
///
/// Notes
/// -----
/// The rank of a partition is its largest part minus the number of parts.
/// Dyson (1944) conjectured that the rank modulo 5 (resp. 7) provides a
/// combinatorial explanation of Ramanujan's congruences
/// $p(5n+4) \equiv 0 \pmod{5}$ (resp. $p(7n+5) \equiv 0 \pmod{7}$),
/// later proved by Atkin and Swinnerton-Dyer (1954). The rank does not
/// explain the mod 11 congruence -- that requires the crank.
///
/// See Also
/// --------
/// crank_gf : Crank generating function $C(z, q)$.
/// partition_gf : Unrestricted partition generating function.
/// sift : Extract arithmetic subsequences.
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
/// At $z = 1$, the crank generating function reduces to the partition
/// generating function:
///
/// >>> from q_kangaroo import QSession, crank_gf
/// >>> s = QSession()
/// >>> c = crank_gf(s, 1, 1, 20)
/// >>> print(c)
/// >>> # 1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + 11*q^6 + ...
///
/// Notes
/// -----
/// The crank was introduced by Andrews and Garvan (1988) to explain all three
/// Ramanujan congruences. For a partition $\lambda$, the crank is defined as:
///
/// - the largest part, if $\lambda$ has no 1's;
/// - (number of parts larger than the number of 1's) minus (number of 1's),
///   otherwise.
///
/// The crank modulo 5, 7, and 11 provides equinumerous classes that explain
/// $p(5n+4) \equiv 0$, $p(7n+5) \equiv 0$, and $p(11n+6) \equiv 0$.
///
/// See Also
/// --------
/// rank_gf : Rank generating function $R(z, q)$.
/// partition_gf : Unrestricted partition generating function.
/// findcong : Discover congruences in series coefficients.
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
/// Factor $(q;q)_\infty$ truncated to 20 terms -- recovers the single factor
/// $(1-q)^1 \cdot (1-q^2)^1 \cdots$:
///
/// >>> from q_kangaroo import QSession, etaq, qfactor
/// >>> s = QSession()
/// >>> f = qfactor(etaq(s, 1, 1, 20))
/// >>> f["is_exact"]
/// True
/// >>> f["factors"]  # {1: 1, 2: 1, 3: 1, ..., 19: 1}
///
/// Factor a Gaussian binomial (a finite q-polynomial):
///
/// >>> f2 = qfactor(qbin(s, 4, 2, 20))
/// >>> f2["is_exact"]
/// True
///
/// Notes
/// -----
/// Uses top-down polynomial division to express a q-polynomial as
/// $c \cdot \prod_i (1-q^i)^{m_i}$. Exact factorization succeeds for products
/// of cyclotomic-type factors. For infinite series (truncated), the factorization
/// is approximate and ``is_exact`` may be ``False``.
///
/// See Also
/// --------
/// prodmake : Recover infinite product exponents (Andrews' algorithm).
/// etaq : Compute an eta product directly.
/// mprodmake : Decompose into $(1+q^n)$ factors.
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
/// Ramanujan's congruence $p(5n+4) \equiv 0 \pmod{5}$: sift out the
/// subsequence at residue 4 mod 5 and verify every coefficient is
/// divisible by 5:
///
/// >>> from q_kangaroo import QSession, partition_gf, sift
/// >>> s = QSession()
/// >>> pgf = partition_gf(s, 200)
/// >>> s5_4 = sift(pgf, 5, 4)
/// >>> # Coefficients: p(4)=5, p(9)=30, p(14)=135, p(19)=490, ...
/// >>> # Every coefficient is divisible by 5
///
/// Similarly, $p(7n+5) \equiv 0 \pmod{7}$:
///
/// >>> s7_5 = sift(pgf, 7, 5)
/// >>> # Coefficients: p(5)=7, p(12)=77, p(19)=490, ...
/// >>> # Every coefficient is divisible by 7
///
/// Notes
/// -----
/// The sift operation extracts the arithmetic subsequence $f[mi+j]$,
/// producing a new series $g$ with $g[n] = f[mn+j]$. This is the key tool
/// for studying partition congruences and other arithmetic properties of
/// q-series coefficients. For example, Ramanujan's three congruences
/// are discovered by sifting ``partition_gf`` at residues 4 mod 5,
/// 5 mod 7, and 6 mod 11.
///
/// See Also
/// --------
/// findcong : Discover partition congruences automatically.
/// partition_gf : Partition generating function to sift.
/// findlincombo : Express sifted series as linear combinations.
/// etaq : Eta products used in congruence analysis.
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
/// The Gaussian binomial $\binom{5}{2}_q = 1 + q + 2q^2 + 2q^3 + 2q^4 + q^5 + q^6$
/// is a polynomial of degree 6:
///
/// >>> from q_kangaroo import QSession, qbin, qdegree
/// >>> s = QSession()
/// >>> qdegree(qbin(s, 5, 2, 20))
/// 6
///
/// For a truncated infinite series, the degree equals the truncation order minus 1:
///
/// >>> from q_kangaroo import partition_gf
/// >>> qdegree(partition_gf(s, 20))
/// 19
///
/// See Also
/// --------
/// lqdegree : Lowest nonzero exponent (valuation).
/// qfactor : Factor a q-polynomial into cyclotomic components.
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
/// The partition generating function starts at $q^0$ (constant term 1):
///
/// >>> from q_kangaroo import QSession, partition_gf, lqdegree
/// >>> s = QSession()
/// >>> lqdegree(partition_gf(s, 20))
/// 0
///
/// An eta product $(q;q)_\infty = 1 - q - q^2 + \cdots$ also has valuation 0:
///
/// >>> from q_kangaroo import etaq
/// >>> lqdegree(etaq(s, 1, 1, 20))
/// 0
///
/// A sifted series may have a different valuation:
///
/// >>> from q_kangaroo import sift
/// >>> pgf = partition_gf(s, 100)
/// >>> lqdegree(sift(pgf, 5, 4))
/// 0
///
/// See Also
/// --------
/// qdegree : Highest nonzero exponent (degree).
/// sift : Extract arithmetic subsequences.
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
/// The partition generating function $\sum p(n) q^n = \prod_{k \ge 1} (1-q^k)^{-1}$,
/// so prodmake recovers exponent $-1$ for every factor:
///
/// >>> from q_kangaroo import QSession, partition_gf, prodmake
/// >>> s = QSession()
/// >>> pm = prodmake(partition_gf(s, 50), 20)
/// >>> pm["factors"][1]  # exponent of (1-q)
/// Fraction(-1, 1)
/// >>> pm["factors"][2]  # exponent of (1-q^2)
/// Fraction(-1, 1)
/// >>> all(pm["factors"][k] == Fraction(-1, 1) for k in range(1, 21))
/// True
///
/// For a theta function $\theta_3(q) = (q^2;q^2)_\infty \cdot (-q;q^2)_\infty^2$,
/// prodmake reveals the $(1-q^n)$ exponent pattern:
///
/// >>> from q_kangaroo import theta3
/// >>> pm3 = prodmake(theta3(s, 50), 20)
///
/// Notes
/// -----
/// Implements Andrews' algorithm: compute the log derivative $-f'/f$, find
/// recurrence coefficients $c_n$ via $q \cdot d/dq$, then use Mobius inversion
/// to recover $(1-q^n)$ exponents. The result is
/// $f = \prod_{n \ge 1} (1-q^n)^{a_n}$. Requires enough series terms
/// (at least ``max_n`` + a margin) for accurate recovery.
///
/// See Also
/// --------
/// etamake : Express as eta-quotient (groups prodmake output by divisor).
/// jacprodmake : Express as Jacobi products (residue-class grouping).
/// mprodmake : Extract $(1+q^n)$ factors from prodmake output.
/// qfactor : Factor a finite q-polynomial.
/// partition_gf : Partition generating function (classic prodmake target).
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
/// The partition generating function is $1/\eta(\tau)$ up to a $q$-shift,
/// so etamake recovers a single eta factor with exponent $-1$:
///
/// >>> from q_kangaroo import QSession, partition_gf, etamake
/// >>> s = QSession()
/// >>> em = etamake(partition_gf(s, 50), 20)
/// >>> em["factors"]  # {1: -1}
/// >>> em["q_shift"]  # Fraction(1, 24) -- the eta q-shift
///
/// Compare with prodmake, which gives individual $(1-q^n)$ exponents:
///
/// >>> from q_kangaroo import prodmake
/// >>> pm = prodmake(partition_gf(s, 50), 20)
/// >>> # pm["factors"] has 20 entries {1: -1, 2: -1, ..., 20: -1}
/// >>> # etamake groups these into a single eta: {1: -1}
///
/// Notes
/// -----
/// Groups the prodmake exponents by divisor $d$, producing the eta-quotient
/// form $\prod_d (q^d; q^d)_\infty^{e_d}$. Only succeeds when exponents have
/// finite eta-quotient support (i.e., the prodmake exponents $a_n$ satisfy
/// $a_n = \sum_{d|n} e_d$ for finitely many nonzero $e_d$).
///
/// See Also
/// --------
/// prodmake : Underlying Andrews' algorithm (raw exponents).
/// etaq : Compute an eta product directly.
/// jacprodmake : Express as Jacobi products (alternative grouping).
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
/// Recover the Jacobi product form of $\text{JAC}(1,5)$:
///
/// >>> from q_kangaroo import QSession, jacprod, jacprodmake
/// >>> s = QSession()
/// >>> jpm = jacprodmake(jacprod(s, 1, 5, 30), 20)
/// >>> jpm["is_exact"]
/// True
/// >>> # jpm["factors"] contains {(1, 5): 1}
///
/// Notes
/// -----
/// Searches for a period $m$ such that the prodmake exponents group into
/// residue classes modulo $m$, producing a representation in terms of
/// $(q^a; q^m)_\infty$ factors. The ``is_exact`` flag indicates whether all
/// prodmake exponents were successfully grouped. Series that are not naturally
/// Jacobi products may give ``is_exact = False``.
///
/// See Also
/// --------
/// prodmake : Underlying Andrews' algorithm (raw exponents).
/// etamake : Express as eta-quotient (alternative grouping).
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
/// The distinct-parts generating function $\prod_{k \ge 1}(1+q^k)$ should
/// recover exponent 1 for every factor:
///
/// >>> from q_kangaroo import QSession, distinct_parts_gf, mprodmake
/// >>> s = QSession()
/// >>> mp = mprodmake(distinct_parts_gf(s, 50), 20)
/// >>> mp  # {1: 1, 2: 1, 3: 1, ..., 20: 1}
///
/// Notes
/// -----
/// Extracts $(1+q^n)$ factors by using the identity
/// $(1+q^n) = (1-q^{2n})/(1-q^n)$. Iteratively peels off factors from
/// the prodmake representation, converting pairs of $(1-q^n)$ and
/// $(1-q^{2n})$ exponents into $(1+q^n)$ multiplicities.
///
/// See Also
/// --------
/// prodmake : General infinite product decomposition.
/// distinct_parts_gf : Generating function for distinct partitions.
/// etamake : Express as eta-quotient (alternative decomposition).
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
/// The result omits the $q^{1/24}$ factors that appear in eta notation.
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
/// The partition generating function is $1/(q;q)_\infty$:
///
/// >>> from q_kangaroo import QSession, partition_gf, qetamake
/// >>> s = QSession()
/// >>> qem = qetamake(partition_gf(s, 50), 20)
/// >>> qem["factors"]  # {1: -1}
/// >>> qem["q_shift"]  # Fraction(0, 1) -- no q-shift in Pochhammer notation
///
/// Compare with etamake, which includes the $q^{1/24}$ shift from the
/// Dedekind eta definition $\eta(\tau) = q^{1/24}(q;q)_\infty$:
///
/// >>> from q_kangaroo import etamake
/// >>> em = etamake(partition_gf(s, 50), 20)
/// >>> em["q_shift"]  # Fraction(1, 24)
///
/// See Also
/// --------
/// etamake : Express as eta-quotient (includes $q^{1/24}$ shifts).
/// prodmake : General infinite product decomposition (raw exponents).
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
/// Build an eta-quotient basis and express ``partition_gf`` as a linear
/// combination. Since $\sum p(n)q^n = 1/(q;q)_\infty$, the partition
/// generating function equals ``etaq(s, 1, -1, N)`` (exponent $-1$):
///
/// >>> from q_kangaroo import QSession, partition_gf, etaq, findlincombo
/// >>> s = QSession()
/// >>> N = 50
/// >>> pgf = partition_gf(s, N)
/// >>> basis = [etaq(s, 1, -1, N)]  # 1/(q;q)_inf
/// >>> result = findlincombo(pgf, basis, 0)
/// >>> result  # [Fraction(1, 1)]
///
/// Use ``topshift`` to ignore anomalous leading terms when searching
/// for relations among series with different leading behavior.
///
/// Notes
/// -----
/// Uses exact RREF (row reduction) over $\mathbb{Q}$ to find coefficients.
/// The ``topshift`` parameter allows ignoring leading coefficients that may be
/// anomalous or differ between series due to truncation effects.
///
/// See Also
/// --------
/// findhom : Homogeneous polynomial relations among series.
/// findhomcombo : Express target as a homogeneous combination.
/// findlincombomodp : Modular version (faster, avoids coefficient explosion).
/// sift : Extract arithmetic subsequences for targeted relation searches.
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
/// Search for degree-2 relations among eta-quotients at level 5. The
/// monomials of degree 2 in two variables are $f_0^2, f_0 f_1, f_1^2$:
///
/// >>> from q_kangaroo import QSession, etaq, findhom
/// >>> s = QSession()
/// >>> f0 = etaq(s, 1, 5, 50)  # (q;q^5)_inf
/// >>> f1 = etaq(s, 2, 5, 50)  # (q^2;q^5)_inf
/// >>> rels = findhom([f0, f1], 2, 0)
/// >>> # Each relation [a, b, c] means a*f0^2 + b*f0*f1 + c*f1^2 = 0
///
/// Notes
/// -----
/// Generates all monomials of degree $d$ in the given series, builds a
/// coefficient matrix from the series products, and finds the null space
/// via RREF. Returns all independent relations. The number of monomials
/// grows as $\binom{k+d-1}{d}$ where $k$ is the number of series.
///
/// See Also
/// --------
/// findnonhom : Non-homogeneous polynomial relations (mixed degrees).
/// findpoly : Polynomial relation $P(x, y) = 0$ between two series.
/// findhomcombo : Express a target as a homogeneous combination.
/// findhommodp : Modular version (faster for large coefficients).
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
/// Search for a polynomial relation between two eta-quotients:
///
/// >>> from q_kangaroo import QSession, etaq, findpoly
/// >>> s = QSession()
/// >>> x = etaq(s, 1, 5, 80)  # (q;q^5)_inf
/// >>> y = etaq(s, 2, 5, 80)  # (q^2;q^5)_inf
/// >>> result = findpoly(x, y, 3, 3, 0)
/// >>> # If found, result["coefficients"][i][j] is the coeff of x^i * y^j
///
/// Notes
/// -----
/// Searches for a bivariate polynomial $P(x,y) = \sum_{i,j} c_{ij} x^i y^j = 0$
/// by setting up a linear system from the series coefficients. The search space
/// has $(d_x + 1)(d_y + 1)$ unknowns, so sufficient series terms are needed.
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
/// Examples
/// --------
/// Discover Ramanujan's partition congruences automatically:
///
/// >>> from q_kangaroo import QSession, partition_gf, findcong
/// >>> s = QSession()
/// >>> congs = findcong(partition_gf(s, 200), [5, 7, 11])
/// >>> # Discovers:
/// >>> #   {"modulus": 5, "residue": 4, "divisor": 5}   -- p(5n+4) = 0 mod 5
/// >>> #   {"modulus": 7, "residue": 5, "divisor": 7}   -- p(7n+5) = 0 mod 7
/// >>> #   {"modulus": 11, "residue": 6, "divisor": 11} -- p(11n+6) = 0 mod 11
///
/// These are Ramanujan's three celebrated congruences (1919), the starting
/// point of the theory of partition congruences.
///
/// Notes
/// -----
/// Automatically sifts the series at each residue class modulo $m$, then
/// checks which residue classes have all coefficients divisible by $m$.
/// This is the automated tool for discovering Ramanujan-type congruences.
/// Does NOT take a ``topshift`` parameter. Provide enough series terms
/// (at least $m \times$ desired check depth) for reliable detection.
///
/// See Also
/// --------
/// sift : Extract arithmetic subsequences manually.
/// partition_gf : Partition generating function (classic target).
/// findlincombo : Express congruence witnesses as linear combinations.
/// partition_count : Verify individual congruences numerically.
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
/// Search for all polynomial relations of degree at most 2 among eta products.
/// Unlike ``findhom``, this includes the constant (degree 0) and linear
/// (degree 1) monomials alongside degree-2 monomials:
///
/// >>> from q_kangaroo import QSession, etaq, findnonhom
/// >>> s = QSession()
/// >>> f0 = etaq(s, 1, 5, 50)
/// >>> f1 = etaq(s, 2, 5, 50)
/// >>> rels = findnonhom([f0, f1], 2, 0)
/// >>> # Relation vectors cover monomials: [1, f0, f1, f0^2, f0*f1, f1^2]
///
/// Notes
/// -----
/// The monomial list includes all degrees from 0 (constant) through $d$.
/// For $k$ series at degree $d$, this is $\sum_{j=0}^{d} \binom{k+j-1}{j}$
/// monomials -- larger than ``findhom`` which uses only exact degree $d$.
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
/// Express a target as a degree-2 combination. With a single basis series $f$,
/// the degree-2 monomial is just $f^2$, so this checks if target $= c \cdot f^2$:
///
/// >>> from q_kangaroo import QSession, etaq, findhomcombo
/// >>> s = QSession()
/// >>> f = etaq(s, 1, 1, 50)
/// >>> target = etaq(s, 1, 2, 50)  # (q;q^2)_inf
/// >>> result = findhomcombo(target, [f], 1, 0)
/// >>> # Returns [Fraction(1, 1)] since target = 1 * f at degree 1
///
/// With multiple basis series, degree-$d$ monomials include all products
/// of $d$ factors chosen from the basis (with repetition).
///
/// See Also
/// --------
/// findlincombo : Linear (degree 1) combination.
/// findnonhomcombo : Non-homogeneous combination.
/// findhom : Find all homogeneous relations (without a target).
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
/// Express a target as a combination that may include a constant term,
/// linear terms, and quadratic terms:
///
/// >>> from q_kangaroo import QSession, etaq, findnonhomcombo
/// >>> s = QSession()
/// >>> target = etaq(s, 1, 5, 50)
/// >>> basis = [etaq(s, 1, 5, 50)]
/// >>> result = findnonhomcombo(target, basis, 2, 0)
/// >>> # Monomial order: [1, f, f^2]. Returns coefficients for each.
///
/// This is more flexible than ``findhomcombo`` when the relationship
/// involves mixed degrees (e.g., $\text{target} = a + b \cdot f + c \cdot f^2$).
///
/// See Also
/// --------
/// findhomcombo : Homogeneous combination (single degree only).
/// findlincombo : Linear combination (degree 1 special case).
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
/// Express $(q;q)_\infty$ as a linear combination of itself mod 7:
///
/// >>> from q_kangaroo import QSession, etaq, findlincombomodp
/// >>> s = QSession()
/// >>> f = etaq(s, 1, 1, 50)
/// >>> result = findlincombomodp(f, [f], 7, 0)
/// >>> result  # [1] -- coefficient 1 mod 7
///
/// Useful when exact rational coefficients become very large (e.g.,
/// searching for congruences among eta-quotients at high levels) and
/// modular arithmetic suffices.
///
/// Notes
/// -----
/// Uses modular arithmetic over $\mathbb{Z}/p\mathbb{Z}$ with Fermat inverse
/// ($a^{-1} = a^{p-2} \bmod p$). Faster than exact arithmetic and avoids
/// coefficient explosion that can occur with ``findlincombo`` on series
/// with large rational coefficients.
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
/// Search for degree-2 relations among eta products modulo 7:
///
/// >>> from q_kangaroo import QSession, etaq, findhommodp
/// >>> s = QSession()
/// >>> f0 = etaq(s, 1, 5, 50)
/// >>> f1 = etaq(s, 2, 5, 50)
/// >>> rels = findhommodp([f0, f1], 7, 2, 0)
/// >>> # Each relation [a, b, c] means a*f0^2 + b*f0*f1 + c*f1^2 = 0 mod 7
///
/// Notes
/// -----
/// The modular version is useful when exact rational coefficients become
/// unwieldy. Relations found mod $p$ may or may not lift to exact relations
/// over $\mathbb{Q}$, but they reveal structural constraints.
///
/// See Also
/// --------
/// findhom : Exact rational version.
/// findlincombomodp : Linear combination mod $p$.
/// findhomcombomodp : Express a target as a homogeneous combination mod $p$.
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
/// Express a target as a degree-1 combination mod 7:
///
/// >>> from q_kangaroo import QSession, etaq, findhomcombomodp
/// >>> s = QSession()
/// >>> f = etaq(s, 1, 5, 50)
/// >>> result = findhomcombomodp(f, [f], 7, 1, 0)
/// >>> result  # [1] -- coefficient 1 mod 7
///
/// See Also
/// --------
/// findhomcombo : Exact rational version.
/// findhommodp : Find all homogeneous relations mod $p$.
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
/// Given a collection of eta-quotients, find which are linearly independent:
///
/// >>> from q_kangaroo import QSession, etaq, findmaxind
/// >>> s = QSession()
/// >>> N = 50
/// >>> series = [etaq(s, 1, 5, N), etaq(s, 2, 5, N), etaq(s, 3, 5, N)]
/// >>> indices = findmaxind(series, 0)
/// >>> # Returns indices of a maximal linearly independent subset.
/// >>> # The remaining series are linear combinations of these.
///
/// This is a prerequisite step before using ``findlincombo``: first reduce
/// a large candidate set to its independent core, then search for relations.
///
/// Notes
/// -----
/// Uses inline Gaussian elimination to find pivot columns, identifying a
/// maximal linearly independent subset. The remaining series are redundant
/// (expressible as linear combinations of the independent ones). Use this
/// to trim a large basis before more expensive relation searches.
///
/// See Also
/// --------
/// findlincombo : Express a series as a combination of others.
/// findhom : Find polynomial relations among series.
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
/// Examples
/// --------
/// Search for integer combinations of eta products that have nice
/// infinite product forms:
///
/// >>> from q_kangaroo import QSession, etaq, findprod
/// >>> s = QSession()
/// >>> f0 = etaq(s, 1, 5, 50)
/// >>> f1 = etaq(s, 2, 5, 50)
/// >>> results = findprod([f0, f1], 2, 10)
/// >>> # Each result [a, b] means a*f0 + b*f1 has a nice product form.
/// >>> # Combinations like [1, 0] and [0, 1] (the inputs) always appear.
///
/// Notes
/// -----
/// Brute-force search over integer exponent vectors $[-M, M]^k$. Tests each
/// candidate exponent vector by computing the product via prodmake and
/// comparing. Exponential in the number of candidates -- use small candidate
/// lists and small ``max_coeff`` values. Does NOT take a ``topshift``
/// parameter.
///
/// See Also
/// --------
/// prodmake : Decompose a single series into product form.
/// etamake : Express as eta-quotient (after finding a combination).
/// findlincombo : Linear combination search (exact, not brute-force).
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
/// Euler's identity as a ${}_1\phi_0$:
///
/// >>> from q_kangaroo import QSession, phi
/// >>> s = QSession()
/// >>> result = phi(s, [(1,1,1)], [], 1, 1, 0, 20)
/// >>> # 1phi0(q; -; q, 1) = 1/(1-1)(1-q)... diverges; use terminating instead
///
/// A terminating ${}_2\phi_1$ (q-Vandermonde type):
///
/// >>> result = phi(s, [(1,1,-2), (1,1,1)], [(1,1,3)], 1, 1, 4, 20)
/// >>> # 2phi1(q^{-2}, q; q^3; q, q^4) -- terminates after 3 terms
///
/// A ${}_1\phi_0$ series:
///
/// >>> result = phi(s, [(1,1,2)], [], 1, 1, 1, 20)  # 1phi0(q^2; -; q, q)
///
/// Notes
/// -----
/// The basic hypergeometric series is defined as:
///
/// ${}_r\phi_s(a_1,\ldots,a_r; b_1,\ldots,b_s; q, z) =
///   \sum_{n=0}^{\infty}
///   \frac{(a_1;q)_n \cdots (a_r;q)_n}{(b_1;q)_n \cdots (b_s;q)_n (q;q)_n}
///   \left[(-1)^n q^{n(n-1)/2}\right]^{1+s-r} z^n$
///
/// Parameters are specified as q-monomials $(c \cdot q^p)$ via ``(num, den, power)``
/// tuples. For example, ``(1, 1, 2)`` represents $q^2$ and ``(1, 2, 0)``
/// represents $1/2$. DLMF 17.4.1.
///
/// See Also
/// --------
/// psi : Bilateral hypergeometric series ${}_r\psi_s$.
/// try_summation : Try closed-form summation formulas.
/// heine1 : Heine's first transformation for ${}_2\phi_1$.
/// q_gosper : q-Gosper indefinite summation algorithm.
/// q_zeilberger : q-Zeilberger definite summation algorithm.
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
/// Compute a bilateral ${}_1\psi_1$ series:
///
/// >>> from q_kangaroo import QSession, psi
/// >>> s = QSession()
/// >>> result = psi(s, [(1,1,1)], [(1,1,3)], 1, 1, 1, 20)
/// >>> # 1psi1(q; q^3; q, q) sums over n = -inf..inf
///
/// A bilateral series with two upper/lower parameters:
///
/// >>> result = psi(s, [(1,1,2)], [(1,1,5)], 1, 1, 1, 20)
///
/// Notes
/// -----
/// The bilateral hypergeometric series sums over both positive and negative
/// indices:
///
/// ${}_r\psi_s(a_1,\ldots,a_r; b_1,\ldots,b_s; q, z) =
///   \sum_{n=-\infty}^{\infty}
///   \frac{(a_1;q)_n \cdots (a_r;q)_n}{(b_1;q)_n \cdots (b_s;q)_n}
///   \left[(-1)^n q^{n(n-1)/2}\right]^{s-r} z^n$
///
/// The Ramanujan ${}_1\psi_1$ summation is one of the most important bilateral
/// identities, providing a closed-form product for the ${}_1\psi_1$ series.
/// Negative-index terms are computed via explicit $q$-Pochhammer products with
/// pole detection.
///
/// See Also
/// --------
/// phi : Basic (unilateral) hypergeometric series ${}_r\phi_s$.
/// try_summation : Try closed-form summation formulas.
/// tripleprod : Jacobi triple product (related to ${}_1\psi_1$).
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
/// Try summation on a ${}_2\phi_1$ that matches the q-Gauss formula:
///
/// >>> from q_kangaroo import QSession, try_summation
/// >>> s = QSession()
/// >>> closed = try_summation(s, [(1,1,1),(1,1,2)], [(1,1,5)], 1, 1, 2, 30)
/// >>> # Returns the closed-form product if q-Gauss applies, else None
///
/// A series that does not match any known formula returns ``None``:
///
/// >>> result = try_summation(s, [(1,1,1),(1,1,2),(1,1,3)], [(1,1,5)], 1, 1, 1, 30)
/// >>> # A 3phi1 -- no known summation formula applies
///
/// Notes
/// -----
/// Attempts six classical summation formulas in sequence:
///
/// 1. **q-Gauss** (DLMF 17.7.5): balanced ${}_2\phi_1$ with $z = c/(ab)$
/// 2. **q-Vandermonde** (two forms, DLMF 17.6.2): terminating ${}_2\phi_1$
/// 3. **q-Saalschutz** (DLMF 17.7.6): balanced terminating ${}_3\phi_2$
/// 4. **q-Kummer**: ${}_2\phi_1$ with $q^2$-Pochhammer base
/// 5. **q-Dixon** (DLMF 17.7.8): very-well-poised ${}_4\phi_3$
///
/// Returns the formula name and closed-form product if successful.
///
/// See Also
/// --------
/// phi : Direct evaluation of ${}_r\phi_s$.
/// heine1 : Heine's first transformation for ${}_2\phi_1$.
/// heine2 : Heine's second transformation.
/// heine3 : Heine's third transformation.
/// q_gosper : Algorithmic indefinite summation.
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
/// Apply Heine's first transformation to ${}_2\phi_1(q^2, q^3; q^5; q, q)$:
///
/// >>> from q_kangaroo import QSession, heine1
/// >>> s = QSession()
/// >>> prefactor, result = heine1(s, [(1,1,2),(1,1,3)], [(1,1,5)], 1, 1, 1, 30)
/// >>> # prefactor = (b;q)_inf * (az;q)_inf / ((c;q)_inf * (z;q)_inf)
/// >>> # result = prefactor * 2phi1(c/b, z; az; q, b)
///
/// Notes
/// -----
/// Heine's first transformation (DLMF 17.6.1):
///
/// ${}_2\phi_1(a, b; c; q, z) =
///   \frac{(b;q)_\infty(az;q)_\infty}{(c;q)_\infty(z;q)_\infty}
///   \cdot {}_2\phi_1(c/b, z; az; q, b)$
///
/// Valid for $|z| < 1$, $|b| < 1$. The transformation exchanges the role of
/// $z$ and $b$, often simplifying the convergence region or the parameter
/// structure. This is the most commonly used Heine transformation.
///
/// See Also
/// --------
/// heine2 : Heine's second transformation.
/// heine3 : Heine's third transformation.
/// phi : Direct evaluation of ${}_r\phi_s$.
/// try_summation : Try closed-form summation formulas.
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
/// Apply Heine's second transformation to ${}_2\phi_1(q^2, q^3; q^5; q, q)$:
///
/// >>> from q_kangaroo import QSession, heine2
/// >>> s = QSession()
/// >>> prefactor, result = heine2(s, [(1,1,2),(1,1,3)], [(1,1,5)], 1, 1, 1, 30)
/// >>> # prefactor = (c/b;q)_inf * (bz;q)_inf / ((c;q)_inf * (z;q)_inf)
/// >>> # result = prefactor * 2phi1(abz/c, b; bz; q, c/b)
///
/// Notes
/// -----
/// Heine's second transformation:
///
/// ${}_2\phi_1(a, b; c; q, z) =
///   \frac{(c/b;q)_\infty(bz;q)_\infty}{(c;q)_\infty(z;q)_\infty}
///   \cdot {}_2\phi_1(abz/c, b; bz; q, c/b)$
///
/// This transformation keeps $b$ as an upper parameter while modifying the
/// other parameters. Useful when the second transformation produces simpler
/// parameters than the first.
///
/// See Also
/// --------
/// heine1 : Heine's first transformation.
/// heine3 : Heine's third transformation.
/// phi : Direct evaluation of ${}_r\phi_s$.
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
/// Apply Heine's third transformation to ${}_2\phi_1(q^2, q^3; q^5; q, q)$:
///
/// >>> from q_kangaroo import QSession, heine3
/// >>> s = QSession()
/// >>> prefactor, result = heine3(s, [(1,1,2),(1,1,3)], [(1,1,5)], 1, 1, 1, 30)
/// >>> # result = prefactor * 2phi1(abz/c, c/b; c/(bz); q, z)
///
/// Notes
/// -----
/// Heine's third transformation is the composition of the first and second
/// transformations. It provides a third equivalent ${}_2\phi_1$ representation,
/// sometimes producing the simplest parameter set. Together with the first and
/// second transformations, it gives a complete set of Heine-type transformations
/// for ${}_2\phi_1$ series.
///
/// See Also
/// --------
/// heine1 : Heine's first transformation.
/// heine2 : Heine's second transformation.
/// phi : Direct evaluation of ${}_r\phi_s$.
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
/// Prove that $\eta(5\tau)^6 = \eta(\tau)^6$ at level 5 (this will fail --
/// it is a toy example; real identities require matching weight and level):
///
/// >>> from q_kangaroo import QSession, prove_eta_id
/// >>> s = QSession()
/// >>> result = prove_eta_id(s, [(5, 6)], [(1, 6)], 5)
/// >>> result["status"]  # 'proved', 'not_modular', 'counterexample', etc.
///
/// Prove a genuine eta-quotient identity. The factors ``[(d, r)]`` encode
/// $\prod_d \eta(d\tau)^r$. Both sides must have the same weight
/// ($\frac{1}{2}\sum r_\delta$) and be valid modular forms at level $N$:
///
/// >>> result = prove_eta_id(s, [(1, 2), (2, -1)], [(1, 2), (2, -1)], 2)
/// >>> result["status"]
/// 'proved'
///
/// Notes
/// -----
/// Proves identities of the form
/// $\prod_\delta \eta(\delta\tau)^{e_\delta} = \prod_\delta \eta(\delta\tau)^{f_\delta}$
/// by expanding both sides as formal power series and comparing coefficients up
/// to a Sturm bound. The Sturm bound guarantees that equality of sufficiently
/// many coefficients implies the identity holds for all $q$, because the
/// difference is a modular form of bounded weight with non-negative orders at
/// all cusps.
///
/// The algorithm first checks modularity conditions (integer weight, character,
/// cusp non-negativity), then compares series coefficients up to the Sturm bound
/// $\lfloor kN/12 \prod_{p|N}(1+1/p) \rfloor$ where $k$ is the weight.
///
/// See Also
/// --------
/// search_identities : Search the identity database.
/// etaq : Construct eta-quotients as series.
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
/// Search by tag to find all classical identities:
///
/// >>> from q_kangaroo import search_identities
/// >>> results = search_identities("classical", search_type="tag")
/// >>> # Returns list of dicts with id, name, tags, functions, author, year
///
/// Search by function name to find identities involving eta:
///
/// >>> results = search_identities("eta", search_type="function")
///
/// Free-text pattern search:
///
/// >>> results = search_identities("partition", search_type="pattern")
/// >>> for r in results:
/// ...     print(r["name"], r.get("author", ""))
///
/// Notes
/// -----
/// Builds a basis of identities from the built-in database (or a custom
/// TOML file) and searches for matches. The database includes classical
/// identities (Euler, Jacobi, Ramanujan, Rogers-Ramanujan, etc.) with
/// metadata: tags, involved functions, citations, and proof status.
/// Use ``search_type="tag"`` for categorical search, ``"function"`` for
/// identities involving a specific function, or ``"pattern"`` for free-text.
///
/// See Also
/// --------
/// prove_eta_id : Prove an eta-quotient identity.
/// etaq : Construct eta-quotients as series.
/// findhom : Find homogeneous polynomial relations among series.
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
/// >>> # 1 + q - 2*q^2 + 3*q^3 - 3*q^4 + 3*q^5 - 5*q^6 + ...
///
/// Notes
/// -----
/// $f(q) = \sum_{n \ge 0} \frac{q^{n^2}}{(-q;q)_n^2}$.
///
/// The most famous mock theta function, appearing in Ramanujan's last letter
/// to Hardy (January 1920). One of Ramanujan's original third-order mock
/// theta functions. Third-order mock theta functions transform like modular
/// forms of weight 1/2 up to a non-holomorphic correction (Zwegers, 2002).
///
/// See Also
/// --------
/// mock_theta_phi3 : Third-order $\phi(q)$.
/// mock_theta_psi3 : Third-order $\psi(q)$.
/// mock_theta_chi3 : Third-order $\chi(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
/// bailey_discover : Bailey pair representation.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_phi3
/// >>> s = QSession()
/// >>> phi3 = mock_theta_phi3(s, 20)
/// >>> # 1 + q + q^2 + 2*q^3 + 2*q^4 + 2*q^5 + ...
///
/// Notes
/// -----
/// $\phi(q) = \sum_{n \ge 0} \frac{q^{n^2}}{(-q^2;q^2)_n}$.
///
/// One of Ramanujan's original third-order mock theta functions from his
/// 1920 letter to Hardy. Third-order mock theta functions transform like
/// modular forms of weight 1/2 up to a non-holomorphic correction
/// (Zwegers, 2002).
///
/// See Also
/// --------
/// mock_theta_f3 : Third-order $f(q)$.
/// mock_theta_psi3 : Third-order $\psi(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_psi3
/// >>> s = QSession()
/// >>> psi3 = mock_theta_psi3(s, 20)
/// >>> # q + q^2 + q^3 + 2*q^4 + 2*q^5 + 2*q^6 + ...
///
/// Notes
/// -----
/// $\psi(q) = \sum_{n \ge 1} \frac{q^{n^2}}{(q;q^2)_n}$.
///
/// One of Ramanujan's original third-order mock theta functions from his
/// 1920 letter to Hardy. Note the summation starts at $n=1$.
/// Third-order mock theta functions transform like modular forms of
/// weight 1/2 up to a non-holomorphic correction (Zwegers, 2002).
///
/// See Also
/// --------
/// mock_theta_f3 : Third-order $f(q)$.
/// mock_theta_phi3 : Third-order $\phi(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_chi3
/// >>> s = QSession()
/// >>> chi3 = mock_theta_chi3(s, 20)
/// >>> # 1 + q + 2*q^2 + 2*q^3 + 3*q^4 + ...
///
/// Notes
/// -----
/// $\chi(q) = \sum_{n \ge 0} \frac{q^{n^2} (-q;q)_n}{(-q^3;q^3)_n}$.
///
/// One of Ramanujan's original third-order mock theta functions from his
/// 1920 letter to Hardy. The denominator involves a cyclotomic factor
/// $(1 - q^k + q^{2k})$ at each step. Third-order mock theta functions
/// transform like modular forms of weight 1/2 up to a non-holomorphic
/// correction (Zwegers, 2002).
///
/// See Also
/// --------
/// mock_theta_f3 : Third-order $f(q)$.
/// mock_theta_omega3 : Third-order $\omega(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_omega3
/// >>> s = QSession()
/// >>> omega3 = mock_theta_omega3(s, 20)
/// >>> # 1 + 2*q^2 + 3*q^4 + 4*q^6 + ...
///
/// Notes
/// -----
/// $\omega(q) = \sum_{n \ge 0} \frac{q^{2n(n+1)}}{(q;q^2)_{n+1}^2}$.
///
/// One of Ramanujan's original third-order mock theta functions from his
/// 1920 letter to Hardy. The exponents $2n(n+1)$ are always even, so
/// $\omega(q)$ has support only on even powers. Third-order mock theta
/// functions transform like modular forms of weight 1/2 up to a
/// non-holomorphic correction (Zwegers, 2002).
///
/// See Also
/// --------
/// mock_theta_nu3 : Third-order $\nu(q)$.
/// mock_theta_chi3 : Third-order $\chi(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_nu3
/// >>> s = QSession()
/// >>> nu3 = mock_theta_nu3(s, 20)
/// >>> # 1 + q^2 - q^3 + 2*q^4 + ...
///
/// Notes
/// -----
/// $\nu(q) = \sum_{n \ge 0} \frac{q^{n(n+1)}}{(-q;q^2)_{n+1}}$.
///
/// One of Ramanujan's original third-order mock theta functions from his
/// 1920 letter to Hardy. Third-order mock theta functions transform like
/// modular forms of weight 1/2 up to a non-holomorphic correction
/// (Zwegers, 2002).
///
/// See Also
/// --------
/// mock_theta_omega3 : Third-order $\omega(q)$.
/// mock_theta_f3 : Third-order $f(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_rho3
/// >>> s = QSession()
/// >>> rho3 = mock_theta_rho3(s, 20)
/// >>> # 1 + q^2 + q^4 + 2*q^6 + ...
///
/// Notes
/// -----
/// $\rho(q) = \sum_{n \ge 0} \frac{q^{2n(n+1)}}{(q;q^2)_{n+1}(1+q^{2n+1}+q^{2(2n+1)})}$.
///
/// One of Ramanujan's original third-order mock theta functions from his
/// 1920 letter to Hardy. The denominator contains the cyclotomic factor
/// $(1 + q^m + q^{2m})$ which distinguishes it from the other third-order
/// functions. Third-order mock theta functions transform like modular forms
/// of weight 1/2 up to a non-holomorphic correction (Zwegers, 2002).
///
/// See Also
/// --------
/// mock_theta_omega3 : Third-order $\omega(q)$.
/// mock_theta_chi3 : Third-order $\chi(q)$ (also has cyclotomic factor).
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_f0_5
/// >>> s = QSession()
/// >>> f0 = mock_theta_f0_5(s, 20)
/// >>> # 1 + q + q^2 + q^3 + 2*q^4 + ...
///
/// Notes
/// -----
/// $f_0(q) = \sum_{n \ge 0} \frac{q^{n^2}}{(-q;q)_n}$.
///
/// One of Ramanujan's fifth-order mock theta functions. The ten fifth-order
/// functions split into two groups of five, related by Watson's (1936)
/// analysis. $f_0$ and $f_1$ form a companion pair.
///
/// See Also
/// --------
/// mock_theta_f1_5 : Companion $f_1(q)$.
/// mock_theta_phi0_5 : Fifth-order $\phi_0(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_f1_5
/// >>> s = QSession()
/// >>> f1 = mock_theta_f1_5(s, 20)
/// >>> # q^2 + q^3 + q^4 + q^5 + 2*q^6 + ...
///
/// Notes
/// -----
/// $f_1(q) = \sum_{n \ge 1} \frac{q^{n(n+1)}}{(-q;q)_n}$.
///
/// One of Ramanujan's fifth-order mock theta functions. The ten fifth-order
/// functions split into two groups of five, related by Watson's (1936)
/// analysis. $f_1$ is the companion of $f_0$.
///
/// See Also
/// --------
/// mock_theta_f0_5 : Companion $f_0(q)$.
/// mock_theta_phi1_5 : Fifth-order $\phi_1(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_cap_f0_5
/// >>> s = QSession()
/// >>> F0 = mock_theta_cap_f0_5(s, 20)
/// >>> # 1 + q^2 + q^4 + q^6 + q^8 + ...
///
/// Notes
/// -----
/// $F_0(q) = \sum_{n \ge 0} \frac{q^{2n^2}}{(q;q^2)_n}$.
///
/// One of Ramanujan's fifth-order mock theta functions. The ten fifth-order
/// functions split into two groups of five, related by Watson's (1936)
/// analysis. $F_0$ and $F_1$ form a companion pair with even-indexed
/// exponents.
///
/// See Also
/// --------
/// mock_theta_cap_f1_5 : Companion $F_1(q)$.
/// mock_theta_f0_5 : Fifth-order $f_0(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_cap_f1_5
/// >>> s = QSession()
/// >>> F1 = mock_theta_cap_f1_5(s, 20)
/// >>> # 1 + q^2 + q^4 + q^6 + ...
///
/// Notes
/// -----
/// $F_1(q) = \sum_{n \ge 0} \frac{q^{2n(n+1)}}{(q;q^2)_{n+1}}$.
///
/// One of Ramanujan's fifth-order mock theta functions. The ten fifth-order
/// functions split into two groups of five, related by Watson's (1936)
/// analysis. $F_1$ is the companion of $F_0$.
///
/// See Also
/// --------
/// mock_theta_cap_f0_5 : Companion $F_0(q)$.
/// mock_theta_f1_5 : Fifth-order $f_1(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_phi0_5
/// >>> s = QSession()
/// >>> phi0 = mock_theta_phi0_5(s, 20)
/// >>> # 1 + q + q^2 + 2*q^3 + ...
///
/// Notes
/// -----
/// $\phi_0(q) = \sum_{n \ge 0} q^{n^2} (-q;q^2)_n$.
///
/// One of Ramanujan's fifth-order mock theta functions. The ten fifth-order
/// functions split into two groups of five, related by Watson's (1936)
/// analysis. $\phi_0$ and $\phi_1$ form a companion pair.
///
/// See Also
/// --------
/// mock_theta_phi1_5 : Companion $\phi_1(q)$.
/// mock_theta_psi0_5 : Fifth-order $\psi_0(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_phi1_5
/// >>> s = QSession()
/// >>> phi1 = mock_theta_phi1_5(s, 20)
/// >>> # q + q^2 + 2*q^3 + 2*q^4 + ...
///
/// Notes
/// -----
/// $\phi_1(q) = \sum_{n \ge 0} q^{(n+1)^2} (-q;q^2)_n$.
///
/// One of Ramanujan's fifth-order mock theta functions. The ten fifth-order
/// functions split into two groups of five, related by Watson's (1936)
/// analysis. $\phi_1$ is the companion of $\phi_0$.
///
/// See Also
/// --------
/// mock_theta_phi0_5 : Companion $\phi_0(q)$.
/// mock_theta_psi1_5 : Fifth-order $\psi_1(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_psi0_5
/// >>> s = QSession()
/// >>> psi0 = mock_theta_psi0_5(s, 20)
/// >>> # 1 + q + q^2 + q^3 + 2*q^4 + ...
///
/// Notes
/// -----
/// $\psi_0(q) = \sum_{n \ge 0} q^{n(n+1)/2} (-q;q)_n$.
///
/// One of Ramanujan's fifth-order mock theta functions. The ten fifth-order
/// functions split into two groups of five, related by Watson's (1936)
/// analysis. $\psi_0$ and $\psi_1$ form a companion pair with triangular
/// number exponents.
///
/// See Also
/// --------
/// mock_theta_psi1_5 : Companion $\psi_1(q)$.
/// mock_theta_chi0_5 : Fifth-order $\chi_0(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_psi1_5
/// >>> s = QSession()
/// >>> psi1 = mock_theta_psi1_5(s, 20)
/// >>> # q + q^2 + q^3 + 2*q^4 + ...
///
/// Notes
/// -----
/// $\psi_1(q) = \sum_{n \ge 1} q^{n(n+1)/2} (-q;q)_{n-1}$.
///
/// One of Ramanujan's fifth-order mock theta functions. The ten fifth-order
/// functions split into two groups of five, related by Watson's (1936)
/// analysis. $\psi_1$ is the companion of $\psi_0$.
///
/// See Also
/// --------
/// mock_theta_psi0_5 : Companion $\psi_0(q)$.
/// mock_theta_chi1_5 : Fifth-order $\chi_1(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_chi0_5
/// >>> s = QSession()
/// >>> chi0 = mock_theta_chi0_5(s, 20)
/// >>> # 1 + q + 2*q^2 + 2*q^3 + ...
///
/// Notes
/// -----
/// $\chi_0(q) = \sum_{n \ge 0} \frac{q^n (-q;q)_n}{(q;q^2)_{n+1}}$.
///
/// One of Ramanujan's fifth-order mock theta functions. The ten fifth-order
/// functions split into two groups of five, related by Watson's (1936)
/// analysis. $\chi_0$ and $\chi_1$ form a companion pair. Internally
/// computed via $q \to -q$ substitution composition.
///
/// See Also
/// --------
/// mock_theta_chi1_5 : Companion $\chi_1(q)$.
/// mock_theta_f0_5 : Fifth-order $f_0(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_chi1_5
/// >>> s = QSession()
/// >>> chi1 = mock_theta_chi1_5(s, 20)
/// >>> # 1 + q + q^2 + 2*q^3 + ...
///
/// Notes
/// -----
/// $\chi_1(q) = \sum_{n \ge 0} \frac{q^n (-q;q)_n}{(q;q^2)_{n+1}}$.
///
/// One of Ramanujan's fifth-order mock theta functions. The ten fifth-order
/// functions split into two groups of five, related by Watson's (1936)
/// analysis. $\chi_1$ is the companion of $\chi_0$. Internally
/// computed via $q \to -q$ substitution composition.
///
/// See Also
/// --------
/// mock_theta_chi0_5 : Companion $\chi_0(q)$.
/// mock_theta_f1_5 : Fifth-order $f_1(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_cap_f0_7
/// >>> s = QSession()
/// >>> F0_7 = mock_theta_cap_f0_7(s, 20)
/// >>> # 1 + q + q^2 + q^3 + 2*q^4 + ...
///
/// Notes
/// -----
/// $F_0(q) = \sum_{n \ge 0} \frac{q^{n^2}}{(q^n;q)_n}$.
///
/// Seventh-order mock theta function, first studied systematically by
/// Selberg and later by Andrews (1986). Connected to affine Lie algebra
/// characters. Uses per-term $q$-Pochhammer evaluation with shifted base
/// (cannot use incremental accumulation).
///
/// See Also
/// --------
/// mock_theta_cap_f1_7 : Seventh-order $F_1(q)$.
/// mock_theta_cap_f2_7 : Seventh-order $F_2(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
/// bailey_discover : Bailey pair representation.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_cap_f1_7
/// >>> s = QSession()
/// >>> F1_7 = mock_theta_cap_f1_7(s, 20)
/// >>> # q + q^2 + q^3 + q^4 + 2*q^5 + ...
///
/// Notes
/// -----
/// $F_1(q) = \sum_{n \ge 1} \frac{q^{n^2}}{(q^n;q)_n}$.
///
/// Seventh-order mock theta function, first studied systematically by
/// Selberg and later by Andrews (1986). Connected to affine Lie algebra
/// characters. Note the summation starts at $n=1$.
///
/// See Also
/// --------
/// mock_theta_cap_f0_7 : Seventh-order $F_0(q)$.
/// mock_theta_cap_f2_7 : Seventh-order $F_2(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
/// bailey_discover : Bailey pair representation.
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
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, mock_theta_cap_f2_7
/// >>> s = QSession()
/// >>> F2_7 = mock_theta_cap_f2_7(s, 20)
/// >>> # 1 + q + q^2 + 2*q^3 + ...
///
/// Notes
/// -----
/// $F_2(q) = \sum_{n \ge 0} \frac{q^{n(n+1)}}{(q^{n+1};q)_{n+1}}$.
///
/// Seventh-order mock theta function, first studied systematically by
/// Selberg and later by Andrews (1986). Connected to affine Lie algebra
/// characters. The exponents $n(n+1)$ are oblong numbers.
///
/// See Also
/// --------
/// mock_theta_cap_f0_7 : Seventh-order $F_0(q)$.
/// mock_theta_cap_f1_7 : Seventh-order $F_1(q)$.
/// appell_lerch_m : Appell-Lerch sum connection.
/// bailey_discover : Bailey pair representation.
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
/// Compute the Appell-Lerch sum $m(q^3, q, q^2)$:
///
/// >>> from q_kangaroo import QSession, appell_lerch_m
/// >>> s = QSession()
/// >>> result = appell_lerch_m(s, 3, 2, 20)
/// >>> # Returns the bilateral sum as a formal power series
///
/// The Appell-Lerch sum connects to mock theta functions. For example,
/// the third-order mock theta function $f(q)$ can be expressed via
/// Appell-Lerch sums.
///
/// Notes
/// -----
/// The Appell-Lerch sum is defined as:
///
/// $m(x,q,z) = \frac{1}{j(z;q)} \sum_{n=-\infty}^{\infty}
///   \frac{(-1)^n q^{n(n-1)/2} z^n}{1 - xzq^{n-1}}$
///
/// This function is central to the theory of mock theta functions
/// (Zwegers, 2002). Many mock theta functions can be expressed as linear
/// combinations of Appell-Lerch sums plus theta functions. For integer
/// parameters, $j(z;q) = 0$, so this returns the raw bilateral sum
/// without the $j$-division.
///
/// See Also
/// --------
/// universal_mock_theta_g2 : Zwegers $g_2$ function.
/// universal_mock_theta_g3 : Zwegers $g_3$ function.
/// mock_theta_f3 : Third-order $f(q)$ (expressible via Appell-Lerch).
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
/// Compute $g_2(q^3, q)$:
///
/// >>> from q_kangaroo import QSession, universal_mock_theta_g2
/// >>> s = QSession()
/// >>> result = universal_mock_theta_g2(s, 3, 20)
/// >>> # Returns the g2 function as a formal power series
///
/// Notes
/// -----
/// The Zwegers $g_2$ function is a building block for the universal mock
/// theta function. Used in the non-holomorphic completion of mock theta
/// functions, which restores modular transformation properties at the cost
/// of holomorphicity. Computed via a positive-exponent algebraic identity
/// to avoid FPS negative-power limitations.
///
/// See Also
/// --------
/// universal_mock_theta_g3 : Companion $g_3$ function.
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
/// Compute $g_3(q^3, q)$:
///
/// >>> from q_kangaroo import QSession, universal_mock_theta_g3
/// >>> s = QSession()
/// >>> result = universal_mock_theta_g3(s, 3, 20)
/// >>> # Returns the g3 function as a formal power series
///
/// Notes
/// -----
/// The Zwegers $g_3$ function is the companion to $g_2$, another building
/// block for the universal mock theta function. Together with $g_2$, it
/// provides the non-holomorphic corrections needed to complete mock theta
/// functions into harmonic Maass forms.
///
/// See Also
/// --------
/// universal_mock_theta_g2 : Companion $g_2$ function.
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
/// Verify the weak Bailey lemma with the Rogers-Ramanujan pair at $a=1$:
///
/// >>> from q_kangaroo import QSession, bailey_weak_lemma
/// >>> s = QSession()
/// >>> lhs, rhs = bailey_weak_lemma(s, "rogers-ramanujan", 1, 1, 0, 10, 20)
/// >>> # Both sides should agree -- this yields the first Rogers-Ramanujan identity
///
/// With the unit pair at $a = q^2$:
///
/// >>> lhs, rhs = bailey_weak_lemma(s, "unit", 1, 1, 2, 8, 20)
///
/// Notes
/// -----
/// The weak Bailey lemma: if $(\alpha_n, \beta_n)$ is a Bailey pair
/// relative to $a$, then
///
/// $\sum_{n \ge 0} q^{n^2} a^n \beta_n
///   = \frac{1}{(aq;q)_\infty} \sum_{n \ge 0} q^{n^2} a^n \alpha_n$
///
/// With the Rogers-Ramanujan pair at $a=1$, this yields the first
/// Rogers-Ramanujan identity:
/// $\sum_{n \ge 0} \frac{q^{n^2}}{(q;q)_n} = \prod_{n \ge 0} \frac{1}{(1-q^{5n+1})(1-q^{5n+4})}$.
///
/// Mock theta functions also arise from Bailey pairs, connecting the
/// Bailey machinery to the Zwegers theory.
///
/// See Also
/// --------
/// bailey_apply_lemma : Apply the full Bailey lemma.
/// bailey_chain : Iterative Bailey chain.
/// bailey_discover : Automated Bailey pair discovery.
/// mock_theta_f3 : Mock theta functions arise from Bailey pairs.
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
/// Apply the lemma to the unit pair with $a = q^2$, $b = q/2$, $c = q/3$:
///
/// >>> from q_kangaroo import QSession, bailey_apply_lemma
/// >>> s = QSession()
/// >>> result = bailey_apply_lemma(s, "unit", (1,1,2), (1,2,1), (1,3,1), 4, 15)
/// >>> result["name"]  # Name of the derived pair
///
/// Notes
/// -----
/// The (full) Bailey lemma transforms $(\alpha_n, \beta_n)$ relative to $a$
/// into a new pair $(\alpha'_n, \beta'_n)$ via parameters $b$, $c$:
///
/// $\beta'_n = \sum_{j=0}^{n}
///   \frac{(b;q)_j (c;q)_j (aq/bc)^j}{(q;q)_{n-j} (aq/b;q)_n (aq/c;q)_n}
///   \beta_j$
///
/// Iterating produces a Bailey chain, generating increasingly complex
/// $q$-series identities from simple seed pairs.
///
/// See Also
/// --------
/// bailey_weak_lemma : Weak Bailey lemma.
/// bailey_chain : Iterative Bailey chain.
/// bailey_discover : Automated discovery.
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
/// Build a depth-2 Bailey chain from the unit pair:
///
/// >>> from q_kangaroo import QSession, bailey_chain
/// >>> s = QSession()
/// >>> chain = bailey_chain(s, "unit", (1,1,2), (1,2,1), (1,3,1), 2, 4, 15)
/// >>> len(chain)  # 3 pairs: original + 2 derived
/// 3
/// >>> chain[0]["index"]  # 0 = seed pair
/// 0
///
/// The Rogers-Ramanujan identities arise as depth-1 chains from the unit
/// pair with appropriate parameter specializations.
///
/// Notes
/// -----
/// Iterates the Bailey lemma $k$ times from a seed pair, producing
/// increasingly complex identities. Each step in the chain transforms
/// $(\alpha_n, \beta_n) \to (\alpha'_n, \beta'_n)$ using the same
/// parameters $b$, $c$. The Rogers-Ramanujan identities arise as
/// depth-1 chains from the unit pair. Deeper chains produce Andrews-Gordon
/// type identities and other multi-sum Rogers-Ramanujan generalizations.
///
/// See Also
/// --------
/// bailey_apply_lemma : Single application of the Bailey lemma.
/// bailey_weak_lemma : Verify both sides of weak lemma.
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
/// Discover whether a relationship between two series has a Bailey pair
/// explanation:
///
/// >>> from q_kangaroo import QSession, bailey_discover, bailey_weak_lemma
/// >>> s = QSession()
/// >>> lhs, rhs = bailey_weak_lemma(s, "rogers-ramanujan", 1, 1, 0, 10, 20)
/// >>> result = bailey_discover(s, lhs, rhs, (1,1,0), 2, 20)
/// >>> result["found"]  # True -- rediscovers the Rogers-Ramanujan pair
/// True
///
/// Notes
/// -----
/// Automated discovery proceeds in three stages:
///
/// 1. **Trivial equality**: checks if LHS $=$ RHS directly
/// 2. **Weak lemma matching**: tests each database pair with the weak Bailey lemma
/// 3. **Chain depth search**: iterates the Bailey lemma up to ``max_chain_depth``
///    steps, checking if any chain from a database pair produces a match
///
/// Returns the construction path if found (pair name, chain depth). The
/// database contains 3 canonical pairs: Unit, Rogers-Ramanujan, and
/// q-Binomial.
///
/// See Also
/// --------
/// bailey_chain : Iterative Bailey chain.
/// bailey_weak_lemma : Weak Bailey lemma verification.
/// bailey_apply_lemma : Single lemma application.
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
