//! q-Gosper algorithm for indefinite q-hypergeometric summation.
//!
//! This module implements the foundational components of the q-Gosper algorithm,
//! following the approach of Koornwinder (1993) and Paule-Riese (1997).
//!
//! The q-Gosper algorithm decides whether a q-hypergeometric term t_k has a
//! q-hypergeometric antidifference S_k (i.e., S_{k+1} - S_k = t_k), and if so,
//! finds a rational function certificate y(x) such that S_k = y(q^k) * t_k.
//!
//! Key components:
//! - [`extract_term_ratio`]: Convert a `HypergeometricSeries` into a rational
//!   function t_{k+1}/t_k in x = q^k.
//! - [`q_dispersion`]: Find all non-negative integers j where gcd(a(x), b(q^j*x))
//!   is non-trivial -- the key input to the Gosper normal form decomposition.
//! - [`QGosperResult`]: Result of the q-Gosper algorithm (summable with certificate, or not).
//! - [`GosperNormalForm`]: The sigma/tau/c decomposition of a term ratio.

use crate::number::QRat;
use crate::poly::{QRatPoly, QRatRationalFunc, poly_gcd};
use super::{QMonomial, HypergeometricSeries};

/// Result of the q-Gosper algorithm.
#[derive(Clone, Debug)]
pub enum QGosperResult {
    /// The sum has a q-hypergeometric antidifference with the given rational
    /// function certificate y(x), where S_k = y(q^k) * t_k.
    Summable {
        /// The rational function certificate.
        certificate: QRatRationalFunc,
    },
    /// No q-hypergeometric antidifference exists.
    NotSummable,
}

/// Gosper normal form decomposition of a term ratio.
///
/// Given the term ratio r(x) = t_{k+1}/t_k as a rational function of x = q^k,
/// the normal form decomposes r(x) = sigma(x)/tau(x) * c(qx)/c(x) where:
/// - gcd(sigma(x), tau(q^j * x)) = 1 for all j >= 1
/// - c(x) captures the "shiftable" common factors
#[derive(Clone, Debug)]
pub struct GosperNormalForm {
    /// Numerator factor after removing shiftable common factors.
    pub sigma: QRatPoly,
    /// Denominator factor after removing shiftable common factors.
    pub tau: QRatPoly,
    /// Adjustment polynomial capturing the shiftable common factors.
    pub c: QRatPoly,
}

// ---- Private helpers ----

/// Raise a QRat to a signed integer power via repeated squaring.
///
/// For negative exponents, computes base^|exp| then inverts.
/// Panics if base is zero and exp is negative.
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

/// Evaluate a QMonomial c * q^power at a specific q value.
///
/// Returns c * q_val^power. Handles positive, zero, and negative powers.
fn eval_qmonomial(mono: &QMonomial, q_val: &QRat) -> QRat {
    if mono.power == 0 {
        return mono.coeff.clone();
    }
    let q_pow = qrat_pow_i64(q_val, mono.power);
    &mono.coeff * &q_pow
}

// ---- Public functions ----

/// Extract the term ratio t_{k+1}/t_k of a hypergeometric series as a rational
/// function of x = q^k.
///
/// For a series _r phi_s (a_1,...,a_r; b_1,...,b_s; q, z), the term ratio is:
///
/// ```text
/// t_{k+1}/t_k = prod_i (1 - a_i * x) / [(1 - q*x) * prod_j (1 - b_j * x)]
///               * (-1)^{1+s-r} * x^{1+s-r} * z
/// ```
///
/// where each a_i, b_j, z are evaluated at the given q_val.
///
/// # Arguments
/// * `series` - The hypergeometric series whose term ratio to extract.
/// * `q_val` - The value of q (the base).
///
/// # Returns
/// A `QRatRationalFunc` representing the term ratio as a function of x = q^k.
pub fn extract_term_ratio(
    series: &HypergeometricSeries,
    q_val: &QRat,
) -> QRatRationalFunc {
    let r = series.r() as i64;
    let s = series.s() as i64;

    // Build numerator: product of (1 - a_i_eval * x) for each upper param
    let mut numer = QRatPoly::one();
    for a_i in &series.upper {
        let a_i_eval = eval_qmonomial(a_i, q_val);
        // Factor (1 - a_i_eval * x) = a_i_eval * (-1/a_i_eval + x) but simpler as linear:
        // linear(constant, x_coeff) = constant + x_coeff * x
        // We want 1 - a_i_eval * x = 1 + (-a_i_eval) * x
        let factor = QRatPoly::linear(QRat::one(), -a_i_eval);
        numer = &numer * &factor;
    }

    // Build denominator: (1 - q*x) * product of (1 - b_j_eval * x)
    // The (q;q) factor gives (1 - q*x) = 1 + (-q_val) * x
    let q_factor = QRatPoly::linear(QRat::one(), -q_val.clone());
    let mut denom = q_factor;
    for b_j in &series.lower {
        let b_j_eval = eval_qmonomial(b_j, q_val);
        let factor = QRatPoly::linear(QRat::one(), -b_j_eval);
        denom = &denom * &factor;
    }

    // Extra factor: (-1)^{1+s-r} * z * x^{1+s-r}
    let extra = 1 + s - r;
    let z_eval = eval_qmonomial(&series.argument, q_val);
    let sign: i64 = if extra % 2 == 0 { 1 } else { -1 };
    let sign_rat = QRat::from((sign, 1i64));
    let extra_coeff = &sign_rat * &z_eval;

    if extra >= 0 {
        // Multiply numerator by extra_coeff * x^extra
        let extra_mono = QRatPoly::monomial(extra_coeff, extra as usize);
        numer = &numer * &extra_mono;
    } else {
        // Multiply denominator by x^|extra|, and scale numerator by extra_coeff
        let abs_extra = (-extra) as usize;
        let denom_mono = QRatPoly::monomial(QRat::one(), abs_extra);
        denom = &denom * &denom_mono;
        numer = numer.scalar_mul(&extra_coeff);
    }

    QRatRationalFunc::new(numer, denom)
}

/// Find all non-negative integers j such that gcd(a(x), b(q^j * x)) has degree >= 1.
///
/// This is the q-dispersion set, a key input to the Gosper normal form decomposition.
/// The upper bound on j is deg(a) * deg(b) from resultant theory.
///
/// # Arguments
/// * `a` - First polynomial.
/// * `b` - Second polynomial.
/// * `q_val` - The q-shift parameter.
///
/// # Returns
/// A sorted vector of non-negative integers j where gcd(a(x), b(q^j*x)) is non-trivial.
pub fn q_dispersion(
    a: &QRatPoly,
    b: &QRatPoly,
    q_val: &QRat,
) -> Vec<i64> {
    q_dispersion_range(a, b, q_val, 0)
}

/// Find all positive integers j such that gcd(a(x), b(q^j * x)) has degree >= 1.
///
/// Same as `q_dispersion` but starting from j=1 (excludes j=0). This variant
/// is needed by the Gosper normal form decomposition in Plan 02.
pub(crate) fn q_dispersion_positive(
    a: &QRatPoly,
    b: &QRatPoly,
    q_val: &QRat,
) -> Vec<i64> {
    q_dispersion_range(a, b, q_val, 1)
}

/// Internal helper: find all integers j >= start such that gcd(a(x), b(q^j*x))
/// has degree >= 1, up to the bound deg(a)*deg(b).
fn q_dispersion_range(
    a: &QRatPoly,
    b: &QRatPoly,
    q_val: &QRat,
    start: i64,
) -> Vec<i64> {
    if a.is_zero() || b.is_zero() {
        return Vec::new();
    }

    let deg_a = match a.degree() {
        Some(d) => d,
        None => return Vec::new(),
    };
    let deg_b = match b.degree() {
        Some(d) => d,
        None => return Vec::new(),
    };

    // If either polynomial is constant (degree 0), no nontrivial GCD is possible
    // with a linear-or-higher polynomial.
    if deg_a == 0 || deg_b == 0 {
        return Vec::new();
    }

    let j_max = (deg_a * deg_b) as i64;
    let mut result = Vec::new();

    for j in start..=j_max {
        let b_shifted = b.q_shift_n(q_val, j);
        let g = poly_gcd(a, &b_shifted);
        if g.degree().unwrap_or(0) >= 1 {
            result.push(j);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Helper: create QRat from integer ----
    fn qr(n: i64) -> QRat {
        QRat::from((n, 1i64))
    }

    fn qr_frac(n: i64, d: i64) -> QRat {
        QRat::from((n, d))
    }

    // ========================================
    // eval_qmonomial tests
    // ========================================

    #[test]
    fn test_eval_qmonomial_zero_power() {
        // c * q^0 = c regardless of q
        let mono = QMonomial::new(qr(5), 0);
        assert_eq!(eval_qmonomial(&mono, &qr(7)), qr(5));
    }

    #[test]
    fn test_eval_qmonomial_positive_power() {
        // 3 * 2^2 = 12
        let mono = QMonomial::new(qr(3), 2);
        assert_eq!(eval_qmonomial(&mono, &qr(2)), qr(12));
    }

    #[test]
    fn test_eval_qmonomial_negative_power() {
        // 1 * 2^{-1} = 1/2
        let mono = QMonomial::q_power(-1);
        assert_eq!(eval_qmonomial(&mono, &qr(2)), qr_frac(1, 2));
    }

    // ========================================
    // extract_term_ratio tests
    // ========================================

    #[test]
    fn test_extract_term_ratio_2phi1() {
        // _2phi1(q^{-2}, q^2; q^3; q, q)
        // upper = [q^{-2}, q^2], lower = [q^3], z = q
        // Using q_val = 2:
        //   a1_eval = 2^{-2} = 1/4
        //   a2_eval = 2^2 = 4
        //   b1_eval = 2^3 = 8
        //   extra = 1 + 1 - 2 = 0 => just z_eval = 2
        //   numer = (1 - x/4)(1 - 4x) * 2 = 2*(1 - x/4)(1 - 4x)
        //   denom = (1 - 2x)(1 - 8x)
        let series = HypergeometricSeries {
            upper: vec![QMonomial::q_power(-2), QMonomial::q_power(2)],
            lower: vec![QMonomial::q_power(3)],
            argument: QMonomial::q_power(1),
        };
        let q_val = qr(2);
        let ratio = extract_term_ratio(&series, &q_val);

        // Check degrees: numer should have degree 2+0=2 (extra=0 means monomial x^0=1)
        // Actually extra=0, sign=1, extra_coeff = 1*2 = 2
        // numer = (1 - 1/4 * x)(1 - 4x) * monomial(2, 0) where monomial(2,0) = constant 2
        // numer = 2*(1 - 1/4*x)(1 - 4x)
        // = 2*(1 - 4x - x/4 + x^2) = 2*(1 - 17/4*x + x^2)
        // = 2 - 17/2*x + 2x^2
        // denom = (1 - 2x)(1 - 8x) = 1 - 10x + 16x^2

        // After auto-reduction in QRatRationalFunc, check that ratio is correct
        // by evaluating at a specific x value.
        // At x = 1/10:
        // numer(1/10) = 2 - 17/20 + 2/100 = 2 - 0.85 + 0.02 = 200/100 - 85/100 + 2/100 = 117/100
        // denom(1/10) = 1 - 1 + 16/100 = 16/100 = 4/25
        // ratio(1/10) = (117/100) / (4/25) = (117/100) * (25/4) = 2925/400 = 117/16
        let x_val = qr_frac(1, 10);
        let val = ratio.eval(&x_val).unwrap();
        assert_eq!(val, qr_frac(117, 16));

        // Also verify degrees
        assert_eq!(ratio.numer.degree(), Some(2));
        assert_eq!(ratio.denom.degree(), Some(2));
    }

    #[test]
    fn test_extract_term_ratio_1phi0() {
        // _1phi0(q^{-3}; ; q, q)
        // upper = [q^{-3}], lower = [], z = q
        // r=1, s=0, extra = 1+0-1 = 0
        // sign = 1, extra_coeff = 1 * q_val = q_val
        // Using q_val = 3:
        //   a1_eval = 3^{-3} = 1/27
        //   z_eval = 3
        //   extra_coeff = 1 * 3 = 3
        //   numer = (1 - x/27) * monomial(3, 0) = 3*(1 - x/27) = 3 - x/9
        //   denom = (1 - 3x)  [only the (q;q) factor]
        let series = HypergeometricSeries {
            upper: vec![QMonomial::q_power(-3)],
            lower: vec![],
            argument: QMonomial::q_power(1),
        };
        let q_val = qr(3);
        let ratio = extract_term_ratio(&series, &q_val);

        // numer = 3 - x/9 = 3 + (-1/9)x
        // denom = 1 - 3x
        // After monic normalization of denom:
        // denom lc = -3, so divide both by -3:
        // numer_normalized = (3 + (-1/9)x) / (-3) = -1 + x/27
        // denom_normalized = (1 - 3x) / (-3) = -1/3 + x
        // Hmm, let's just evaluate to check.
        // At x = 1: numer = 3 - 1/9 = 26/9, denom = 1 - 3 = -2
        // ratio(1) = (26/9) / (-2) = -13/9
        let val = ratio.eval(&qr(1)).unwrap();
        assert_eq!(val, qr_frac(-13, 9));

        assert_eq!(ratio.numer.degree(), Some(1));
        assert_eq!(ratio.denom.degree(), Some(1));
    }

    #[test]
    fn test_extract_term_ratio_vandermonde_verification() {
        // q-Vandermonde: _2phi1(a, q^{-n}; c; q, cq^n/a)
        // Use a = q^2, n = 2, c = q^3, so:
        //   upper = [q^2, q^{-2}], lower = [q^3], z = c*q^n/a = q^3 * q^2 / q^2 = q^3
        // With q_val = 2:
        //   a_eval = 4, q_neg2_eval = 1/4, b_eval = 8, z_eval = 8
        //   extra = 1+1-2 = 0, sign = 1, extra_coeff = 8
        //   numer = (1-4x)(1-x/4) * 8 = 8*(1 - 4x - x/4 + x^2)
        //         = 8*(1 - 17x/4 + x^2) = 8 - 34x + 8x^2
        //   denom = (1-2x)(1-8x) = 1 - 10x + 16x^2
        let series = HypergeometricSeries {
            upper: vec![QMonomial::q_power(2), QMonomial::q_power(-2)],
            lower: vec![QMonomial::q_power(3)],
            argument: QMonomial::q_power(3),
        };
        let q_val = qr(2);
        let ratio = extract_term_ratio(&series, &q_val);

        // Verify at x = 1/3:
        // numer(1/3) = 8 - 34/3 + 8/9 = 72/9 - 102/9 + 8/9 = -22/9
        // denom(1/3) = 1 - 10/3 + 16/9 = 9/9 - 30/9 + 16/9 = -5/9
        // ratio(1/3) = (-22/9) / (-5/9) = 22/5
        let val = ratio.eval(&qr_frac(1, 3)).unwrap();
        assert_eq!(val, qr_frac(22, 5));

        // Verify at x = 1/5:
        // numer(1/5) = 8 - 34/5 + 8/25 = 200/25 - 170/25 + 8/25 = 38/25
        // denom(1/5) = 1 - 10/5 + 16/25 = 25/25 - 50/25 + 16/25 = -9/25
        // ratio(1/5) = (38/25) / (-9/25) = -38/9
        let val2 = ratio.eval(&qr_frac(1, 5)).unwrap();
        assert_eq!(val2, qr_frac(-38, 9));
    }

    // ========================================
    // q_dispersion tests
    // ========================================

    #[test]
    fn test_q_dispersion_coprime() {
        // a(x) = x + 1, b(x) = x + 3, q = 2
        // b(q^j * x) = q^j*x + 3
        // For gcd(x+1, 2^j*x + 3) to be nontrivial, they need a common root.
        // x+1 has root x=-1. b(2^j*x) at x=-1: -2^j + 3.
        // -2^j + 3 = 0 => 2^j = 3, which has no integer solution.
        // So dispersion should be empty.
        let a = QRatPoly::from_i64_coeffs(&[1, 1]); // x + 1
        let b = QRatPoly::from_i64_coeffs(&[3, 1]); // x + 3
        let result = q_dispersion(&a, &b, &qr(2));
        assert!(result.is_empty());
    }

    #[test]
    fn test_q_dispersion_j0_common() {
        // a(x) = (1-x), b(x) = (1-x), q=2
        // gcd(1-x, 1-x) = 1-x, nontrivial. j=0 should be in result.
        let a = QRatPoly::from_i64_coeffs(&[1, -1]); // 1 - x
        let b = QRatPoly::from_i64_coeffs(&[1, -1]); // 1 - x
        let result = q_dispersion(&a, &b, &qr(2));
        assert!(result.contains(&0));
    }

    #[test]
    fn test_q_dispersion_1mx_vs_1m2x_q2() {
        // a(x) = (1-x), b(x) = (1-2x), q=2
        // b(q^j*x) = 1 - 2*2^j*x = 1 - 2^{j+1}*x
        // gcd(1-x, 1-2^{j+1}*x):
        //   Common root of (1-x) is x=1.
        //   (1-2^{j+1}*x) at x=1: 1-2^{j+1} = 0 => j+1 = 0 impossible for j>=0.
        //   So dispersion is empty.
        let a = QRatPoly::from_i64_coeffs(&[1, -1]); // 1 - x
        let b = QRatPoly::from_i64_coeffs(&[1, -2]); // 1 - 2x
        let result = q_dispersion(&a, &b, &qr(2));
        assert!(result.is_empty());
    }

    #[test]
    fn test_q_dispersion_q1_degenerate() {
        // q=1 is degenerate: q^j = 1 for all j, so b(q^j*x) = b(x) always.
        // a(x) = (1-x)(1-2x), b(x) = (1-x)(1-3x)
        // gcd(a, b) = 1-x for all j (since shifting by q=1 does nothing).
        // j_max = deg(a)*deg(b) = 2*2 = 4, so j=0,1,2,3,4 are all in dispersion.
        let a = &QRatPoly::from_i64_coeffs(&[1, -1]) * &QRatPoly::from_i64_coeffs(&[1, -2]);
        let b = &QRatPoly::from_i64_coeffs(&[1, -1]) * &QRatPoly::from_i64_coeffs(&[1, -3]);
        let result = q_dispersion(&a, &b, &qr(1));
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_q_dispersion_shift_match() {
        // a(x) = (1-2x), b(x) = (1-x), q=2
        // b(q^j*x) = 1 - 2^j*x
        // gcd(1-2x, 1-2^j*x) nontrivial when 2^j = 2, i.e., j=1.
        // Root of (1-2x) is x=1/2.
        // (1-2^j*x) at x=1/2: 1 - 2^j/2 = 1 - 2^{j-1}.
        // This is zero when j-1=0, i.e., j=1.
        let a = QRatPoly::from_i64_coeffs(&[1, -2]); // 1 - 2x
        let b = QRatPoly::from_i64_coeffs(&[1, -1]); // 1 - x
        let result = q_dispersion(&a, &b, &qr(2));
        assert!(result.contains(&1));
        // j=0: gcd(1-2x, 1-x) = 1 (roots 1/2 and 1 are different) -> not in dispersion
        assert!(!result.contains(&0));
    }

    #[test]
    fn test_q_dispersion_multiple_shifts() {
        // a(x) = (1-x)(1-4x), b(x) = (1-x), q=2
        // b(q^j*x) = 1 - 2^j*x
        // Roots of a: x=1 and x=1/4.
        // For j=0: b(x)=1-x, root x=1. gcd with a includes (1-x) factor. j=0 in result.
        // For j=2: b(q^2*x) = 1-4x, root x=1/4. gcd with a includes (1-4x) factor. j=2 in result.
        // For j=1: b(q*x) = 1-2x, root x=1/2. Neither 1 nor 1/4 equals 1/2. Not in result.
        let a = &QRatPoly::from_i64_coeffs(&[1, -1]) * &QRatPoly::from_i64_coeffs(&[1, -4]);
        let b = QRatPoly::from_i64_coeffs(&[1, -1]); // 1 - x
        let result = q_dispersion(&a, &b, &qr(2));
        assert!(result.contains(&0));
        assert!(result.contains(&2));
        assert!(!result.contains(&1));
    }

    // ========================================
    // q_dispersion_positive tests
    // ========================================

    #[test]
    fn test_q_dispersion_positive_excludes_j0() {
        // Same setup as test_q_dispersion_j0_common: a=b=1-x, q=2
        // q_dispersion returns [0], q_dispersion_positive should return []
        let a = QRatPoly::from_i64_coeffs(&[1, -1]);
        let b = QRatPoly::from_i64_coeffs(&[1, -1]);
        let result = q_dispersion_positive(&a, &b, &qr(2));
        assert!(result.is_empty());
    }

    #[test]
    fn test_q_dispersion_positive_keeps_positive_shifts() {
        // a(x) = (1-x)(1-4x), b(x) = (1-x), q=2
        // Full dispersion = [0, 2]. Positive dispersion should be [2].
        let a = &QRatPoly::from_i64_coeffs(&[1, -1]) * &QRatPoly::from_i64_coeffs(&[1, -4]);
        let b = QRatPoly::from_i64_coeffs(&[1, -1]);
        let result = q_dispersion_positive(&a, &b, &qr(2));
        assert_eq!(result, vec![2]);
    }

    // ========================================
    // Zero / constant polynomial edge cases
    // ========================================

    #[test]
    fn test_q_dispersion_zero_poly() {
        let a = QRatPoly::zero();
        let b = QRatPoly::from_i64_coeffs(&[1, -1]);
        assert!(q_dispersion(&a, &b, &qr(2)).is_empty());
        assert!(q_dispersion(&b, &a, &qr(2)).is_empty());
    }

    #[test]
    fn test_q_dispersion_constant_poly() {
        // gcd of a constant and any poly is at most constant (degree 0) => empty
        let a = QRatPoly::constant(qr(5));
        let b = QRatPoly::from_i64_coeffs(&[1, -1]);
        assert!(q_dispersion(&a, &b, &qr(2)).is_empty());
    }

    // ========================================
    // qrat_pow_i64 tests
    // ========================================

    #[test]
    fn test_qrat_pow_i64_positive() {
        assert_eq!(qrat_pow_i64(&qr(3), 4), qr(81));
    }

    #[test]
    fn test_qrat_pow_i64_zero() {
        assert_eq!(qrat_pow_i64(&qr(7), 0), qr(1));
    }

    #[test]
    fn test_qrat_pow_i64_negative() {
        assert_eq!(qrat_pow_i64(&qr(2), -3), qr_frac(1, 8));
    }
}
