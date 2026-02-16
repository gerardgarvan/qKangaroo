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

/// Decompose the term ratio numer(x)/denom(x) into Gosper normal form.
///
/// The Gosper normal form writes r(x) = sigma(x)/tau(x) * c(qx)/c(x) where:
/// - gcd(sigma(x), tau(q^j * x)) = 1 for all j >= 1
/// - c(x) captures the "shiftable" common factors
///
/// This decomposition is the key prerequisite for solving the Gosper key equation.
///
/// # Arguments
/// * `numer` - Numerator of the term ratio.
/// * `denom` - Denominator of the term ratio.
/// * `q_val` - The q-shift parameter.
///
/// # Returns
/// A `GosperNormalForm` with sigma, tau, c satisfying the decomposition.
pub fn gosper_normal_form(
    numer: &QRatPoly,
    denom: &QRatPoly,
    q_val: &QRat,
) -> GosperNormalForm {
    let mut sigma = numer.clone();
    let mut tau = denom.clone();
    let mut c = QRatPoly::one();

    loop {
        let disp = q_dispersion_positive(&sigma, &tau, q_val);
        if disp.is_empty() {
            break;
        }

        let j_max = *disp.last().unwrap();

        // Compute tau shifted by j_max: tau(q^{j_max} * x)
        let tau_shifted = tau.q_shift_n(q_val, j_max);

        // Compute GCD of sigma and tau_shifted, make it monic for consistency
        let g = poly_gcd(&sigma, &tau_shifted).make_monic();

        // If GCD is constant (degree 0), the dispersion was spurious -- break
        if g.is_constant() {
            break;
        }

        // Update sigma: divide out g
        sigma = sigma.exact_div(&g);

        // Compute g unshifted: g(q^{-j_max} * x), then divide tau by it
        let g_unshifted = g.q_shift_n(q_val, -j_max);
        tau = tau.exact_div(&g_unshifted);

        // Accumulate into c: multiply by g(q^{-i} * x) for i = 1, 2, ..., j_max
        // This gives c(qx)/c(x) = g(x) / g(q^{-j_max} * x), which is what we need
        // since sigma_new * g / (tau_new * g_unshift) = sigma/tau requires
        // c_new(qx)/c_new(x) = g(x) / g(q^{-j_max} * x)
        for i in 1..=j_max {
            let g_back = g.q_shift_n(q_val, -i);
            c = &c * &g_back;
        }
    }

    // Verification (debug mode only): check the decomposition identity
    // sigma(x)/tau(x) * c(qx)/c(x) = numer(x)/denom(x)
    debug_assert!({
        let c_shifted = c.q_shift(q_val);
        let reconstructed = QRatRationalFunc::new(&sigma * &c_shifted, &tau * &c);
        let original = QRatRationalFunc::new(numer.clone(), denom.clone());
        reconstructed == original
    }, "Normal form decomposition failed: sigma/tau * c(qx)/c(x) != numer/denom");

    GosperNormalForm { sigma, tau, c }
}

/// Solve the Gosper key equation: sigma(x) * f(qx) - tau(x) * f(x) = c_poly(x).
///
/// Finds a polynomial f(x) satisfying the key equation, or returns None if no
/// polynomial solution exists. This is the core step that determines whether
/// a q-hypergeometric term is indefinitely summable.
///
/// # Arguments
/// * `sigma` - The sigma polynomial from the normal form.
/// * `tau` - The tau polynomial from the normal form.
/// * `c_poly` - The c polynomial from the normal form.
/// * `q_val` - The q-shift parameter.
///
/// # Returns
/// `Some(f)` if a polynomial solution exists, `None` otherwise.
pub fn solve_key_equation(
    sigma: &QRatPoly,
    tau: &QRatPoly,
    c_poly: &QRatPoly,
    q_val: &QRat,
) -> Option<QRatPoly> {
    // Edge case: c_poly is zero => f = 0 is trivially a solution
    if c_poly.is_zero() {
        return Some(QRatPoly::zero());
    }

    let d_c = c_poly.degree().unwrap();
    let sigma_zero = sigma.is_zero() || sigma.degree().is_none();
    let tau_zero = tau.is_zero() || tau.degree().is_none();

    // Edge case: both sigma and tau are zero, but c is nonzero => impossible
    if sigma_zero && tau_zero {
        return None;
    }

    // Edge case: only sigma is zero => -tau(x)*f(x) = c(x), so f = -c/tau if divisible
    if sigma_zero {
        let neg_c = -c_poly.clone();
        let (q, r) = neg_c.div_rem(tau);
        if r.is_zero() {
            return Some(q);
        } else {
            return None;
        }
    }

    // Edge case: only tau is zero => sigma(x)*f(qx) = c(x)
    // This means we need deg(sigma) + deg(f) = deg(c), so deg(f) = d_c - d_sigma
    if tau_zero {
        let d_sigma = sigma.degree().unwrap();
        if d_c < d_sigma {
            return None;
        }
        let deg_f = d_c - d_sigma;
        return try_solve_with_degree(sigma, tau, c_poly, q_val, deg_f);
    }

    // Normal case: both nonzero
    let d_sigma = sigma.degree().unwrap();
    let d_tau = tau.degree().unwrap();

    // Compute candidate degree bound for f
    let candidates = compute_degree_candidates(sigma, tau, c_poly, q_val, d_sigma, d_tau, d_c);

    // Try each candidate degree bound
    for deg_f in candidates {
        if let Some(f) = try_solve_with_degree(sigma, tau, c_poly, q_val, deg_f) {
            return Some(f);
        }
    }

    None
}

/// Compute candidate degree bounds for the polynomial f in the key equation.
fn compute_degree_candidates(
    sigma: &QRatPoly,
    tau: &QRatPoly,
    _c_poly: &QRatPoly,
    q_val: &QRat,
    d_sigma: usize,
    d_tau: usize,
    d_c: usize,
) -> Vec<usize> {
    let mut candidates = Vec::new();

    if d_sigma != d_tau {
        // Degree of LHS is max(d_sigma, d_tau) + deg_f
        let max_st = d_sigma.max(d_tau);
        if d_c >= max_st {
            candidates.push(d_c - max_st);
        }
        // Also try one higher as a fallback
        if d_c + 1 >= max_st {
            candidates.push(d_c - max_st + 1);
        }
    } else {
        // d_sigma == d_tau: leading terms may cancel
        let lc_sigma = sigma.leading_coeff().unwrap();
        let lc_tau = tau.leading_coeff().unwrap();
        let ratio = &lc_tau / &lc_sigma;

        // Try to find deg_f such that q_val^{deg_f} = ratio
        let mut found_match = false;
        for d in 0..=d_c {
            if qrat_pow_i64(q_val, d as i64) == ratio {
                candidates.push(d);
                found_match = true;
                break;
            }
        }

        // Fallback: the "non-cancellation" case
        if !found_match || d_c >= d_sigma {
            let fallback = if d_c >= d_sigma { d_c - d_sigma } else { 0 };
            if !candidates.contains(&fallback) {
                candidates.push(fallback);
            }
        }

        // Also try one higher than each candidate as fallback
        let extra: Vec<usize> = candidates.iter().map(|&d| d + 1).collect();
        for d in extra {
            if !candidates.contains(&d) {
                candidates.push(d);
            }
        }
    }

    candidates
}

/// Try to solve the key equation with a specific degree bound for f.
fn try_solve_with_degree(
    sigma: &QRatPoly,
    tau: &QRatPoly,
    c_poly: &QRatPoly,
    q_val: &QRat,
    deg_f: usize,
) -> Option<QRatPoly> {
    let d_sigma = sigma.degree().unwrap_or(0);
    let d_tau = tau.degree().unwrap_or(0);
    let d_c = c_poly.degree().unwrap_or(0);

    // Number of unknowns: f_0, f_1, ..., f_{deg_f}
    let n_unknowns = deg_f + 1;

    // The LHS sigma(x)*f(qx) - tau(x)*f(x) has max degree max(d_sigma + deg_f, d_tau + deg_f)
    // We need enough equations to match all coefficients up to d_c
    let max_lhs_deg = d_sigma.max(d_tau) + deg_f;
    let n_equations = max_lhs_deg.max(d_c) + 1;

    // Build the linear system A * [f_0, ..., f_{deg_f}]^T = b
    // A[k][j] = sigma.coeff(k-j) * q_val^j - tau.coeff(k-j)
    // b[k] = c_poly.coeff(k)
    let mut matrix = vec![vec![QRat::zero(); n_unknowns]; n_equations];
    let mut rhs = vec![QRat::zero(); n_equations];

    // Precompute q_val powers
    let mut q_powers = Vec::with_capacity(n_unknowns);
    for j in 0..n_unknowns {
        q_powers.push(qrat_pow_i64(q_val, j as i64));
    }

    for k in 0..n_equations {
        for j in 0..n_unknowns {
            if k >= j {
                let idx = k - j;
                let sigma_contrib = &sigma.coeff(idx) * &q_powers[j];
                let tau_contrib = tau.coeff(idx);
                matrix[k][j] = &sigma_contrib - &tau_contrib;
            }
            // If k < j, both sigma.coeff(negative) and tau.coeff(negative) are 0
        }
        rhs[k] = c_poly.coeff(k);
    }

    // Solve via Gaussian elimination
    solve_linear_system(&matrix, &rhs).map(|coeffs| QRatPoly::from_vec(coeffs))
}

/// Solve the linear system Ax = b via Gaussian elimination (RREF) over Q.
///
/// The system may be overdetermined (more equations than unknowns).
/// Returns None if the system is inconsistent. Returns Some(solution) for
/// the unique solution if one exists. For underdetermined systems, returns
/// the solution with free variables set to zero.
fn solve_linear_system(matrix: &[Vec<QRat>], rhs: &[QRat]) -> Option<Vec<QRat>> {
    let m = matrix.len();
    if m == 0 {
        return Some(Vec::new());
    }
    let n = matrix[0].len();
    if n == 0 {
        // Check consistency: all rhs must be zero
        if rhs.iter().all(|r| r.is_zero()) {
            return Some(Vec::new());
        } else {
            return None;
        }
    }

    // Build augmented matrix [A | b]
    let mut aug: Vec<Vec<QRat>> = Vec::with_capacity(m);
    for i in 0..m {
        let mut row = Vec::with_capacity(n + 1);
        row.extend(matrix[i].iter().cloned());
        row.push(rhs[i].clone());
        aug.push(row);
    }

    let n_cols = n + 1; // augmented column count

    // Forward elimination with partial pivoting -> RREF
    let mut pivot_cols: Vec<usize> = Vec::new();
    let mut pivot_row = 0;

    for col in 0..n {
        if pivot_row >= m {
            break;
        }

        // Find a row with nonzero entry in this column
        let mut found = None;
        for row in pivot_row..m {
            if !aug[row][col].is_zero() {
                found = Some(row);
                break;
            }
        }

        let some_row = match found {
            Some(r) => r,
            None => continue, // free variable
        };

        // Swap to pivot position
        if some_row != pivot_row {
            aug.swap(some_row, pivot_row);
        }

        // Scale pivot row to make pivot = 1
        let pivot_val = aug[pivot_row][col].clone();
        for j in 0..n_cols {
            let val = aug[pivot_row][j].clone();
            aug[pivot_row][j] = &val / &pivot_val;
        }

        // Eliminate all other entries in this column
        for row in 0..m {
            if row == pivot_row {
                continue;
            }
            if aug[row][col].is_zero() {
                continue;
            }
            let factor = aug[row][col].clone();
            for j in 0..n_cols {
                let sub = &factor * &aug[pivot_row][j];
                let val = aug[row][j].clone();
                aug[row][j] = &val - &sub;
            }
        }

        pivot_cols.push(col);
        pivot_row += 1;
    }

    // Check consistency: any row with all-zero LHS but nonzero RHS => inconsistent
    for row in 0..m {
        let all_zero_lhs = (0..n).all(|j| aug[row][j].is_zero());
        if all_zero_lhs && !aug[row][n].is_zero() {
            return None; // inconsistent
        }
    }

    // Extract solution: set free variables to zero, read pivot variables from augmented column
    let mut solution = vec![QRat::zero(); n];
    let pivot_col_to_row: std::collections::HashMap<usize, usize> =
        pivot_cols.iter().enumerate().map(|(row, &col)| (col, row)).collect();

    for &pc in &pivot_cols {
        let row = pivot_col_to_row[&pc];
        solution[pc] = aug[row][n].clone();
    }

    Some(solution)
}

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

    // ========================================
    // gosper_normal_form tests
    // ========================================

    /// Helper: verify the Gosper normal form decomposition identity.
    /// sigma(x)/tau(x) * c(qx)/c(x) = numer(x)/denom(x)
    fn verify_normal_form(numer: &QRatPoly, denom: &QRatPoly, nf: &GosperNormalForm, q_val: &QRat) {
        let c_shifted = nf.c.q_shift(q_val);
        let reconstructed = QRatRationalFunc::new(&nf.sigma * &c_shifted, &nf.tau * &nf.c);
        let original = QRatRationalFunc::new(numer.clone(), denom.clone());
        assert_eq!(
            reconstructed, original,
            "Normal form identity failed: sigma/tau * c(qx)/c(x) != numer/denom\n\
             sigma={}, tau={}, c={}\nnumer={}, denom={}",
            nf.sigma, nf.tau, nf.c, numer, denom
        );
    }

    #[test]
    fn test_gosper_normal_form_already_coprime() {
        // a(x) = 1-x, b(x) = 1-3x, q=2
        // These should have no positive dispersion.
        // Result: sigma=a, tau=b, c=1
        let a = QRatPoly::from_i64_coeffs(&[1, -1]); // 1 - x
        let b = QRatPoly::from_i64_coeffs(&[1, -3]); // 1 - 3x
        let q_val = qr(2);

        let nf = gosper_normal_form(&a, &b, &q_val);

        // sigma and tau should be proportional to a and b (rational function equality)
        verify_normal_form(&a, &b, &nf, &q_val);

        // c should be constant (no shiftable factors extracted)
        assert!(nf.c.is_constant(), "c should be constant for coprime case, got: {}", nf.c);

        // Verify q-coprimality: q_dispersion_positive of the resulting sigma, tau should be empty
        let remaining_disp = q_dispersion_positive(&nf.sigma, &nf.tau, &q_val);
        assert!(remaining_disp.is_empty(), "sigma and tau should be q-coprime after normal form");
    }

    #[test]
    fn test_gosper_normal_form_simple_dispersion_j1() {
        // Construct a(x) and b(x) with nontrivial gcd(a(x), b(q*x)):
        // q_val = 2
        // a(x) = (1-x)(1-4x)  — roots at x=1 and x=1/4
        // b(x) = (1-2x)(1-6x) — roots at x=1/2 and x=1/6
        // b(2x) = (1-4x)(1-12x) — roots at x=1/4 and x=1/12
        // gcd(a(x), b(2x)) = (1-4x) -- nontrivial, so j=1 is in dispersion
        let a = &QRatPoly::from_i64_coeffs(&[1, -1]) * &QRatPoly::from_i64_coeffs(&[1, -4]);
        let b = &QRatPoly::from_i64_coeffs(&[1, -2]) * &QRatPoly::from_i64_coeffs(&[1, -6]);
        let q_val = qr(2);

        let nf = gosper_normal_form(&a, &b, &q_val);

        // Verify reconstruction identity
        verify_normal_form(&a, &b, &nf, &q_val);

        // Verify q-coprimality
        let remaining_disp = q_dispersion_positive(&nf.sigma, &nf.tau, &q_val);
        assert!(remaining_disp.is_empty(),
            "sigma and tau should be q-coprime, but got dispersion {:?}", remaining_disp);

        // c should not be constant (some factor was extracted)
        assert!(!nf.c.is_constant(), "c should be non-constant when dispersion was non-empty");
    }

    #[test]
    fn test_gosper_normal_form_verification_identity_multiple_points() {
        // Test the reconstruction identity at several evaluation points
        let a = &QRatPoly::from_i64_coeffs(&[1, -1]) * &QRatPoly::from_i64_coeffs(&[1, -4]);
        let b = &QRatPoly::from_i64_coeffs(&[1, -2]) * &QRatPoly::from_i64_coeffs(&[1, -6]);
        let q_val = qr(2);

        let nf = gosper_normal_form(&a, &b, &q_val);

        // Verify at several x values: sigma(x)/tau(x) * c(qx)/c(x) == a(x)/b(x)
        for x_val in &[qr_frac(1, 7), qr_frac(3, 11), qr_frac(1, 100), qr_frac(5, 3)] {
            let c_at_x = nf.c.eval(x_val);
            let c_at_qx = nf.c.eval(&(&q_val * x_val));
            if c_at_x.is_zero() {
                continue; // skip evaluation where c(x) = 0
            }
            let lhs_numer = &nf.sigma.eval(x_val) * &c_at_qx;
            let lhs_denom = &nf.tau.eval(x_val) * &c_at_x;
            let lhs = &lhs_numer / &lhs_denom;

            let rhs_numer = a.eval(x_val);
            let rhs_denom = b.eval(x_val);
            if rhs_denom.is_zero() {
                continue;
            }
            let rhs = &rhs_numer / &rhs_denom;

            assert_eq!(lhs, rhs,
                "Normal form evaluation mismatch at x={}: lhs={} != rhs={}", x_val, lhs, rhs);
        }
    }

    #[test]
    fn test_gosper_normal_form_constant_polys() {
        // Edge case: constant numer and denom
        let a = QRatPoly::constant(qr(3));
        let b = QRatPoly::constant(qr(5));
        let q_val = qr(2);

        let nf = gosper_normal_form(&a, &b, &q_val);
        verify_normal_form(&a, &b, &nf, &q_val);
        assert!(nf.c.is_constant(), "c should be 1 for constant polys");
    }

    #[test]
    fn test_gosper_normal_form_q_coprimality_check() {
        // After decomposition, verify gcd(sigma(x), tau(q^j*x)) = 1 for all j >= 1
        // Use a case with multiple dispersion values
        // a(x) = (1-x)(1-4x)(1-8x), b(x) = (1-2x), q=2
        // b(2x) = (1-4x), b(4x) = (1-8x)
        // So dispersion = {1, 2} for (a, b)
        let f1 = QRatPoly::from_i64_coeffs(&[1, -1]);
        let f2 = QRatPoly::from_i64_coeffs(&[1, -4]);
        let f3 = QRatPoly::from_i64_coeffs(&[1, -8]);
        let a = &(&f1 * &f2) * &f3;
        let b = QRatPoly::from_i64_coeffs(&[1, -2]);
        let q_val = qr(2);

        let nf = gosper_normal_form(&a, &b, &q_val);
        verify_normal_form(&a, &b, &nf, &q_val);

        // Verify q-coprimality for j = 1, 2, 3, 4
        for j in 1..=4 {
            let tau_shifted = nf.tau.q_shift_n(&q_val, j);
            let g = poly_gcd(&nf.sigma, &tau_shifted);
            assert!(g.is_constant(),
                "gcd(sigma, tau(q^{}*x)) should be 1, but got degree {:?}",
                j, g.degree());
        }
    }
}
