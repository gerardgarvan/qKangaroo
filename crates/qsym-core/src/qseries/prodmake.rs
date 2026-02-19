//! Andrews' algorithm (prodmake) and post-processing functions for series-to-product
//! conversion.
//!
//! Given a formal power series f(q) = 1 + b_1*q + b_2*q^2 + ... + O(q^T), the
//! `prodmake` function finds exponents a_n such that:
//!
//!   f(q) = prod_{n=1}^{T-1} (1 - q^n)^{-a_n} + O(q^T)
//!
//! The algorithm uses the logarithmic derivative identity:
//!   q*f'(q)/f(q) = sum_{n>=1} c_n * q^n where c_n = sum_{d|n} d*a_d
//!
//! Step 1: Recover c_n from the series coefficients b_n via:
//!   c_n = n*b_n - sum_{j=1}^{n-1} c_j * b_{n-j}
//!
//! Step 2: Recover a_n from c_n via Mobius inversion:
//!   n*a_n = sum_{d|n} mu(n/d) * c_d
//!
//! Post-processing functions interpret prodmake output in mathematically meaningful forms:
//! - [`etamake`]: eta-quotient form prod eta(d*tau)^{r_d}
//! - [`jacprodmake`]: Jacobi product form prod JAC(a,b)^exp
//! - [`mprodmake`]: (1+q^n) product form
//! - [`qetamake`]: (q^d;q^d)_inf notation
//!
//! # References
//!
//! - Andrews' algorithm as described in Garvan's q-series Maple package
//! - Garvan (1998), "A q-product tutorial for a q-series MAPLE package"

use std::collections::BTreeMap;

use crate::number::QRat;
use crate::series::FormalPowerSeries;

/// The result of `prodmake`: exponents a_n in prod_{n>=1} (1-q^n)^{-a_n}.
///
/// Each entry in `exponents` maps n to a_n. Only nonzero exponents are stored.
/// Positive a_n means (1-q^n) appears in the denominator (i.e., divide by (1-q^n)^{a_n}).
/// Negative a_n means (1-q^n) appears in the numerator.
///
/// For example, the Euler function (q;q)_inf = prod(1-q^n) has a_n = -1 for all n,
/// since (q;q)_inf = prod (1-q^n)^{-(-1)}.
#[derive(Clone, Debug)]
pub struct InfiniteProductForm {
    /// Exponents: maps n -> a_n where product is prod (1-q^n)^{-a_n}
    pub exponents: BTreeMap<i64, QRat>,
    /// How many terms were used to determine this product
    pub terms_used: i64,
}

/// Mobius function mu(n).
///
/// - mu(1) = 1
/// - mu(n) = (-1)^k if n is a product of k distinct primes
/// - mu(n) = 0 if n has a squared prime factor
///
/// Uses trial factorization, which is efficient for the small values
/// of n encountered in prodmake (typically n < truncation_order < 1000).
pub(crate) fn mobius(n: i64) -> i64 {
    assert!(n >= 1, "mobius: n must be positive, got {}", n);
    if n == 1 {
        return 1;
    }

    let mut remaining = n;
    let mut num_factors = 0i64;

    // Trial division
    let mut p = 2i64;
    while p * p <= remaining {
        if remaining % p == 0 {
            remaining /= p;
            num_factors += 1;
            // Check if p^2 divides n
            if remaining % p == 0 {
                return 0;
            }
        }
        p += 1;
    }

    // If remaining > 1, it is a prime factor
    if remaining > 1 {
        num_factors += 1;
    }

    if num_factors % 2 == 0 { 1 } else { -1 }
}

/// Return all positive divisors of n in ascending order.
///
/// Uses trial division up to sqrt(n). Efficient for the small values
/// of n encountered in prodmake.
pub(crate) fn divisors(n: i64) -> Vec<i64> {
    assert!(n >= 1, "divisors: n must be positive, got {}", n);

    let mut small = Vec::new();
    let mut large = Vec::new();

    let mut d = 1i64;
    while d * d <= n {
        if n % d == 0 {
            small.push(d);
            if d != n / d {
                large.push(n / d);
            }
        }
        d += 1;
    }

    // large is in descending order; reverse and append
    large.reverse();
    small.extend(large);
    small
}

/// Andrews' algorithm: recover infinite product exponents from series coefficients.
///
/// Given f(q) = sum b_n q^n, finds a_n such that
/// f(q) = prod_{n>=1} (1 - q^n)^{-a_n} + O(q^T).
///
/// The input series is automatically normalized:
/// - If f has a nonzero minimum order k (i.e., f = c * q^k * g(q) with g(0) != 0),
///   the q^k factor is stripped and the algorithm runs on g(q)/g(0).
/// - If f(0) != 1, it is divided by f(0) to normalize the constant term.
///
/// # Arguments
///
/// - `f`: The formal power series to analyze.
/// - `max_n`: Maximum exponent to recover. Capped at `f.truncation_order() - 1`.
///
/// # Panics
///
/// Panics if the series is identically zero.
pub fn prodmake(f: &FormalPowerSeries, max_n: i64) -> InfiniteProductForm {
    assert!(!f.is_zero(), "prodmake: cannot analyze the zero series");

    // Cap max_n at truncation_order - 1
    let effective_max = max_n.min(f.truncation_order() - 1);
    if effective_max < 1 {
        return InfiniteProductForm {
            exponents: BTreeMap::new(),
            terms_used: 0,
        };
    }

    // Normalize: extract min_order shift and scalar
    // If the series has min_order > 0, we need to work with the shifted version.
    let min_ord = f.min_order().unwrap_or(0);

    // Build the normalized series: constant term = 1
    // For coefficient access, we shift indices by min_ord
    let b0 = f.coeff(min_ord);
    assert!(
        !b0.is_zero(),
        "prodmake: leading coefficient must be nonzero"
    );

    // b_n for the normalized series: b_normalized[n] = f.coeff(min_ord + n) / b0
    // We only need b_normalized[0..=effective_max]
    let inv_b0 = QRat::one() / b0;

    // Closure to get normalized coefficient b_n
    let b = |n: i64| -> QRat {
        if min_ord + n >= f.truncation_order() {
            QRat::zero()
        } else {
            f.coeff(min_ord + n) * inv_b0.clone()
        }
    };

    // Step 1: Compute c_n values via the recurrence
    // c_n = n*b_n - sum_{j=1}^{n-1} c_j * b_{n-j}
    let mut c: BTreeMap<i64, QRat> = BTreeMap::new();
    for n in 1..=effective_max {
        let bn = b(n);
        let n_rat = QRat::from((n, 1i64));
        let mut val = n_rat * bn;

        for j in 1..n {
            if let Some(cj) = c.get(&j) {
                let b_nmj = b(n - j);
                if !b_nmj.is_zero() {
                    val = val - cj.clone() * b_nmj;
                }
            }
        }

        if !val.is_zero() {
            c.insert(n, val);
        }
    }

    // Step 2: Recover a_n via Mobius inversion
    // n * a_n = sum_{d|n} mu(n/d) * c_d
    let mut exponents = BTreeMap::new();
    for n in 1..=effective_max {
        let mut sum = QRat::zero();
        for d in divisors(n) {
            if let Some(cd) = c.get(&d) {
                let mu_val = mobius(n / d);
                if mu_val != 0 {
                    let mu_rat = QRat::from((mu_val, 1i64));
                    sum = sum + mu_rat * cd.clone();
                }
            }
        }
        if !sum.is_zero() {
            let n_rat = QRat::from((n, 1i64));
            let a_n = sum / n_rat;
            exponents.insert(n, a_n);
        }
    }

    InfiniteProductForm {
        exponents,
        terms_used: effective_max,
    }
}

// ============================================================================
// Post-processing result types
// ============================================================================

/// Eta-quotient representation: prod eta(d*tau)^{r_d}.
///
/// The Dedekind eta function is eta(d*tau) = q^{d/24} * (q^d;q^d)_inf.
/// An eta-quotient is a product of eta functions with integer exponents.
///
/// For example, the Euler function (q;q)_inf corresponds to eta(tau)^1
/// with q_shift = 1/24.
#[derive(Clone, Debug)]
pub struct EtaQuotient {
    /// Maps d -> r_d where result is prod eta(d*tau)^{r_d}
    pub factors: BTreeMap<i64, i64>,
    /// q-shift prefactor exponent: sum_d r_d * d / 24
    pub q_shift: QRat,
}

/// Jacobi product form: prod JAC(a,b)^exp.
///
/// JAC(a,b) = (q^a;q^b)_inf * (q^{b-a};q^b)_inf * (q^b;q^b)_inf
/// is the Jacobi triple product with parameters a and b.
#[derive(Clone, Debug)]
pub struct JacobiProductForm {
    /// Maps (a, b) -> exponent where result is prod JAC(a,b)^exp
    pub factors: BTreeMap<(i64, i64), i64>,
    /// Scalar prefactor
    pub scalar: QRat,
    /// Whether conversion was successful (all exponents fit JAC pattern)
    pub is_exact: bool,
}

/// Q-eta form: prod (q^d;q^d)_inf^{r_d}.
///
/// Like eta-quotient but without the q^{d/24} prefactors.
/// (q^d;q^d)_inf = prod_{k>=1}(1 - q^{dk}).
#[derive(Clone, Debug)]
pub struct QEtaForm {
    /// Maps d -> r_d where result is prod (q^d;q^d)_inf^{r_d}
    pub factors: BTreeMap<i64, i64>,
    /// Power of q prefactor (residual after removing q-eta factors)
    pub q_shift: QRat,
}

// ============================================================================
// etamake
// ============================================================================

/// Express a series as an eta-quotient: prod eta(d*tau)^{r_d}.
///
/// Runs prodmake to get exponents a_n in prod (1-q^n)^{-a_n}, then
/// recovers the eta-quotient exponents r_d via Mobius inversion.
///
/// The key identity: eta(d*tau) = q^{d/24} * prod_{k>=1}(1 - q^{dk}).
/// So the exponent of (1-q^n) in prod eta(d*tau)^{r_d} is sum_{d|n} r_d.
/// Since prodmake gives (1-q^n)^{-a_n}, we have sum_{d|n} r_d = -a_n,
/// and r_d is recovered by Mobius inversion: r_n = sum_{d|n} mu(n/d) * (-a_d).
///
/// # Arguments
///
/// - `f`: The formal power series to analyze.
/// - `max_n`: Maximum exponent to recover.
pub fn etamake(f: &FormalPowerSeries, max_n: i64) -> EtaQuotient {
    let product = prodmake(f, max_n);
    let effective_max = product.terms_used;

    // Compute e_n = -a_n (exponent of (1-q^n) in the product)
    let mut e: BTreeMap<i64, QRat> = BTreeMap::new();
    for n in 1..=effective_max {
        let a_n = product.exponents.get(&n).cloned().unwrap_or_else(QRat::zero);
        if !a_n.is_zero() {
            e.insert(n, -a_n);
        }
    }

    // Recover r_d via Mobius inversion: r_n = sum_{d|n} mu(n/d) * e_d
    let mut factors = BTreeMap::new();
    for n in 1..=effective_max {
        let mut sum = QRat::zero();
        for d in divisors(n) {
            if let Some(ed) = e.get(&d) {
                let mu_val = mobius(n / d);
                if mu_val != 0 {
                    let mu_rat = QRat::from((mu_val, 1i64));
                    sum = sum + mu_rat * ed.clone();
                }
            }
        }
        if !sum.is_zero() {
            // Convert to i64 (should be integer for valid eta-quotients)
            let r_d = sum.0.to_f64() as i64;
            factors.insert(n, r_d);
        }
    }

    // Compute q_shift = sum_d r_d * d / 24
    let mut q_shift = QRat::zero();
    for (&d, &r_d) in &factors {
        if r_d != 0 {
            let contribution = QRat::from((r_d * d, 24i64));
            q_shift = q_shift + contribution;
        }
    }

    EtaQuotient { factors, q_shift }
}

// ============================================================================
// qetamake
// ============================================================================

/// Express a series in (q^d;q^d)_inf notation: prod (q^d;q^d)_inf^{r_d}.
///
/// Like etamake but outputs in q-Pochhammer notation instead of eta notation.
/// The difference is that eta(d*tau) = q^{d/24} * (q^d;q^d)_inf, so the
/// q-eta form strips the q^{d/24} prefactors.
///
/// The q_shift in QEtaForm is the residual shift NOT accounted for by the
/// (q^d;q^d)_inf factors (typically zero for series starting at q^0).
///
/// # Arguments
///
/// - `f`: The formal power series to analyze.
/// - `max_n`: Maximum exponent to recover.
pub fn qetamake(f: &FormalPowerSeries, max_n: i64) -> QEtaForm {
    let eta = etamake(f, max_n);

    QEtaForm {
        factors: eta.factors,
        // The q_shift in qeta notation is 0 because (q^d;q^d)_inf has no
        // intrinsic q-shift. The eta q_shift = sum r_d*d/24 comes from the
        // eta definition. In qeta notation there is no such shift unless the
        // original series has an explicit q^k prefactor.
        q_shift: QRat::zero(),
    }
}

// ============================================================================
// mprodmake
// ============================================================================

/// Express a series as a product of (1+q^n) factors.
///
/// Since (1+q^n) = (1-q^{2n})/(1-q^n), the prodmake exponents a_n
/// for (1-q^n)^{-a_n} can be converted to (1+q^n) exponents.
///
/// Algorithm: work from n=1 upward. For each n, extract the (1+q^n)
/// exponent m_n from the residual prodmake exponents:
///   m_n = a_n (current residual)
///   Then update: a_n -= m_n (becomes 0), a_{2n} += m_n
///
/// This is because (1+q^n)^{m_n} = (1-q^{2n})^{m_n} / (1-q^n)^{m_n},
/// which in prodmake notation is (1-q^n)^{-m_n} * (1-q^{2n})^{m_n},
/// i.e., contributes -m_n to a_n and +m_n to a_{2n} (wait, that's wrong).
///
/// Actually: prodmake convention is prod (1-q^n)^{-a_n}.
/// (1+q^n)^{m_n} = [(1-q^{2n})/(1-q^n)]^{m_n}
///               = (1-q^{2n})^{m_n} * (1-q^n)^{-m_n}
///               = (1-q^n)^{-m_n} * (1-q^{2n})^{-(- m_n)}
///
/// So (1+q^n)^{m_n} contributes:
///   a_n gets +m_n (from the (1-q^n)^{-m_n} part, since -a_n = -m_n means a_n = m_n)
/// Wait, let me be more careful.
///
/// If we have (1+q^n)^{m_n} and want to express it in prodmake form:
///   (1+q^n) = (1-q^{2n}) / (1-q^n)
///   (1+q^n)^{m_n} = (1-q^{2n})^{m_n} * (1-q^n)^{-m_n}
///
/// In prodmake form prod (1-q^k)^{-a_k}:
///   (1-q^n)^{-m_n} corresponds to a_n += m_n
///   (1-q^{2n})^{m_n} = (1-q^{2n})^{-(-m_n)} corresponds to a_{2n} += -m_n
///
/// So: a_n += m_n, a_{2n} -= m_n.
/// Inversely: m_n = a_n (residual), then a_n -= m_n = 0, a_{2n} += m_n.
///
/// # Arguments
///
/// - `f`: The formal power series to analyze.
/// - `max_n`: Maximum exponent to recover.
///
/// # Returns
///
/// Maps n -> m_n where result is prod (1+q^n)^{m_n}. Only nonzero entries stored.
pub fn mprodmake(f: &FormalPowerSeries, max_n: i64) -> BTreeMap<i64, i64> {
    let product = prodmake(f, max_n);
    let effective_max = product.terms_used;

    // Work with mutable copy of prodmake exponents as i64
    // (they should be integers for valid products)
    let mut a: BTreeMap<i64, i64> = BTreeMap::new();
    for (&n, exp) in &product.exponents {
        let val = exp.0.to_f64() as i64;
        if val != 0 {
            a.insert(n, val);
        }
    }

    let mut m_result: BTreeMap<i64, i64> = BTreeMap::new();

    for n in 1..=effective_max {
        let a_n = *a.get(&n).unwrap_or(&0);
        if a_n == 0 {
            continue;
        }
        let m_n = a_n;
        m_result.insert(n, m_n);

        // Update residuals: a_n becomes 0, a_{2n} gets +m_n
        a.remove(&n);
        let two_n = 2 * n;
        if two_n <= effective_max {
            let a_2n = a.get(&two_n).copied().unwrap_or(0);
            let new_val = a_2n + m_n;
            if new_val != 0 {
                a.insert(two_n, new_val);
            } else {
                a.remove(&two_n);
            }
        }
    }

    m_result
}

// ============================================================================
// jacprodmake
// ============================================================================

/// Express a series as a product of Jacobi triple products JAC(a,b).
///
/// JAC(a,b) = (q^a;q^b)_inf * (q^{b-a};q^b)_inf * (q^b;q^b)_inf
///
/// The algorithm:
/// 1. Run prodmake to get exponents a_n.
/// 2. Search for the best period b that explains the exponent pattern.
/// 3. For each candidate period b, group exponents by residue class mod b.
/// 4. Extract JAC(r,b) factors for each residue r in 1..b/2.
/// 5. Handle residue 0 separately (it contributes to the (q^b;q^b)_inf part).
///
/// # Arguments
///
/// - `f`: The formal power series to analyze.
/// - `max_n`: Maximum exponent to recover.
pub fn jacprodmake(f: &FormalPowerSeries, max_n: i64) -> JacobiProductForm {
    jacprodmake_impl(f, max_n, None)
}

/// Express a series as a product of Jacobi triple products JAC(a,b),
/// restricting the period search to divisors of `pp` (excluding 1).
///
/// This matches Garvan's `jacprodmake(f, q, T, PP)` where PP constrains
/// which periods are tried. Only divisors of PP greater than 1 are tested
/// as candidate periods.
///
/// # Arguments
///
/// - `f`: The formal power series to analyze.
/// - `max_n`: Maximum exponent to recover.
/// - `pp`: Period filter; only divisors of pp > 1 are tested as candidate periods.
pub fn jacprodmake_with_period_filter(f: &FormalPowerSeries, max_n: i64, pp: i64) -> JacobiProductForm {
    jacprodmake_impl(f, max_n, Some(pp))
}

/// Internal implementation of jacprodmake with optional period filter.
fn jacprodmake_impl(f: &FormalPowerSeries, max_n: i64, period_divisor: Option<i64>) -> JacobiProductForm {
    let product = prodmake(f, max_n);
    let effective_max = product.terms_used;

    if effective_max < 1 {
        return JacobiProductForm {
            factors: BTreeMap::new(),
            scalar: QRat::one(),
            is_exact: true,
        };
    }

    // Convert exponents to i64
    let mut a: BTreeMap<i64, i64> = BTreeMap::new();
    for (&n, exp) in &product.exponents {
        let val = exp.0.to_f64() as i64;
        if val != 0 {
            a.insert(n, val);
        }
    }

    if a.is_empty() {
        return JacobiProductForm {
            factors: BTreeMap::new(),
            scalar: QRat::one(),
            is_exact: true,
        };
    }

    // Determine candidate periods based on period_divisor filter
    let candidates: Vec<i64> = match period_divisor {
        Some(pp) => {
            // Garvan: numtheory[divisors](PP) minus {1}
            // Only test divisors of pp that are > 1 and <= effective_max
            divisors(pp).into_iter().filter(|&d| d > 1 && d <= effective_max).collect()
        }
        None => {
            // Default: try all periods from 1 to effective_max
            (1..=effective_max).collect()
        }
    };

    let mut best_b = 0i64;
    let mut best_score = 0i64; // number of exponents explained
    let mut best_factors: BTreeMap<(i64, i64), i64> = BTreeMap::new();
    let mut best_residual_zero = true;

    let total_nonzero = a.len() as i64;

    for b in candidates {
        let result = try_period(&a, b, effective_max);
        if result.explained >= best_score {
            best_score = result.explained;
            best_b = b;
            best_factors = result.factors;
            best_residual_zero = result.residual_zero;
        }
        // Early exit if all exponents explained
        if best_score == total_nonzero && best_residual_zero {
            break;
        }
    }

    let _ = best_b; // suppress unused warning

    JacobiProductForm {
        factors: best_factors,
        scalar: QRat::one(),
        is_exact: best_residual_zero && best_score == total_nonzero,
    }
}

/// Result of trying a specific period for jacprodmake.
struct PeriodResult {
    factors: BTreeMap<(i64, i64), i64>,
    explained: i64,
    residual_zero: bool,
}

/// Try to fit prodmake exponents to JAC factors with a given period b.
///
/// JAC(r,b) contributes:
///   - At positions n where n mod b == r: exponent from (q^r;q^b)_inf
///   - At positions n where n mod b == b-r: exponent from (q^{b-r};q^b)_inf
///   - At positions n where n mod b == 0: exponent from (q^b;q^b)_inf
///
/// The prodmake convention is prod (1-q^n)^{-a_n}. Each of the three sub-products
/// of JAC(r,b) contributes -1 to the exponent at corresponding positions (for JAC^1).
/// So JAC(r,b)^e contributes -e to a_n at positions in residue classes r, b-r, and 0 (mod b).
///
/// Actually, let's think in terms of (1-q^n) exponents directly:
///   (q^r;q^b)_inf = prod_{k>=0}(1-q^{r+kb}) contributes exponent +1 at each n=r+kb
///   so in prodmake's -a_n convention, a_n gets -1 at those positions.
///
/// For JAC(r,b)^e:
///   a_n gets -e at all n with n mod b == r
///   a_n gets -e at all n with n mod b == b-r  (if r != b-r)
///   a_n gets -e at all n with n mod b == 0
fn try_period(a: &BTreeMap<i64, i64>, b: i64, max_n: i64) -> PeriodResult {
    let mut factors: BTreeMap<(i64, i64), i64> = BTreeMap::new();
    let mut residual = a.clone();

    // For each residue class r = 1 to b/2, try to extract JAC(r,b)
    let half_b = b / 2;
    for r in 1..=half_b {
        if r == b - r {
            // Special case: r = b/2 (only when b is even)
            // JAC(r,b) has (q^r;q^b)_inf and (q^{b-r};q^b)_inf = (q^r;q^b)_inf again,
            // plus (q^b;q^b)_inf. So residue class r gets double contribution.
            // We handle this specially.
            continue; // Handle after the loop if needed
        }

        // Check if all exponents in residue class r (mod b) share a common value
        let mut exps_r: Vec<i64> = Vec::new();
        let mut n = r;
        while n <= max_n {
            exps_r.push(*residual.get(&n).unwrap_or(&0));
            n += b;
        }

        if exps_r.is_empty() {
            continue;
        }

        // For JAC(r,b)^e: a_n = -e at positions n mod b == r.
        // So the common value should be -e, giving e = -common_value.
        // But only if ALL values in this residue class are the same.
        let first = exps_r[0];
        let all_same_r = exps_r.iter().all(|&v| v == first);

        // Check the complementary residue class b-r
        let mut exps_br: Vec<i64> = Vec::new();
        n = b - r;
        while n <= max_n {
            exps_br.push(*residual.get(&n).unwrap_or(&0));
            n += b;
        }

        let first_br = if exps_br.is_empty() { first } else { exps_br[0] };
        let all_same_br = exps_br.iter().all(|&v| v == first_br);

        if all_same_r && all_same_br && first == first_br && first != 0 {
            let e = -first; // JAC(r,b)^e means a_n = -e at those positions
            factors.insert((r, b), e);

            // Remove from residual
            n = r;
            while n <= max_n {
                residual.remove(&n);
                n += b;
            }
            n = b - r;
            while n <= max_n {
                residual.remove(&n);
                n += b;
            }
            // Also remove (q^b;q^b)_inf contribution: a_n gets -e at positions 0 mod b
            n = b;
            while n <= max_n {
                let val = residual.get(&n).copied().unwrap_or(0);
                let new_val = val + first; // removing the -e contribution (val - (-e) = val + e = val - first)
                // Wait: a_n gets -e = first at these positions from JAC(r,b)^e
                // So residual[n] should be reduced by first (i.e., we subtract first)
                let new_val2 = val - first;
                if new_val2 != 0 {
                    residual.insert(n, new_val2);
                } else {
                    residual.remove(&n);
                }
                let _ = new_val;
                n += b;
            }
        }
    }

    // Handle r = b/2 case (when b is even)
    if b % 2 == 0 {
        let r = b / 2;
        // JAC(r, b) where r = b/2: (q^r;q^b)_inf * (q^r;q^b)_inf * (q^b;q^b)_inf
        // So residue class r gets DOUBLE contribution: a_n = -2e at positions n mod b == r
        // And residue class 0 gets -e at positions n mod b == 0
        let mut exps_r: Vec<i64> = Vec::new();
        let mut n = r;
        while n <= max_n {
            exps_r.push(*residual.get(&n).unwrap_or(&0));
            n += b;
        }

        if !exps_r.is_empty() {
            let first = exps_r[0];
            let all_same = exps_r.iter().all(|&v| v == first);

            if all_same && first != 0 && first % 2 == 0 {
                let e = -first / 2; // double contribution
                factors.insert((r, b), e);

                n = r;
                while n <= max_n {
                    residual.remove(&n);
                    n += b;
                }
                // Remove (q^b;q^b)_inf contribution
                n = b;
                while n <= max_n {
                    let val = residual.get(&n).copied().unwrap_or(0);
                    let adj = first / 2; // = -e, the contribution to residue 0
                    let new_val = val - adj;
                    if new_val != 0 {
                        residual.insert(n, new_val);
                    } else {
                        residual.remove(&n);
                    }
                    n += b;
                }
            }
        }
    }

    let explained = a.len() as i64 - residual.len() as i64;
    let residual_zero = residual.values().all(|&v| v == 0);

    PeriodResult {
        factors,
        explained,
        residual_zero,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobius_values() {
        assert_eq!(mobius(1), 1);
        assert_eq!(mobius(2), -1);
        assert_eq!(mobius(3), -1);
        assert_eq!(mobius(4), 0); // 4 = 2^2
        assert_eq!(mobius(5), -1);
        assert_eq!(mobius(6), 1); // 6 = 2*3, two distinct primes
        assert_eq!(mobius(7), -1);
        assert_eq!(mobius(8), 0); // 8 = 2^3
        assert_eq!(mobius(9), 0); // 9 = 3^2
        assert_eq!(mobius(10), 1); // 10 = 2*5
        assert_eq!(mobius(12), 0); // 12 = 2^2 * 3
        assert_eq!(mobius(30), -1); // 30 = 2*3*5, three distinct primes
    }

    #[test]
    fn test_divisors() {
        assert_eq!(divisors(1), vec![1]);
        assert_eq!(divisors(2), vec![1, 2]);
        assert_eq!(divisors(6), vec![1, 2, 3, 6]);
        assert_eq!(divisors(7), vec![1, 7]);
        assert_eq!(divisors(12), vec![1, 2, 3, 4, 6, 12]);
        assert_eq!(divisors(36), vec![1, 2, 3, 4, 6, 9, 12, 18, 36]);
    }

    #[test]
    fn test_jacprodmake_with_period_filter() {
        use crate::symbol::SymbolRegistry;
        use crate::qseries::jacprod;

        // Build JAC(1,5) series to O(q^20) -- its period is 5
        let mut reg = SymbolRegistry::new();
        let sym_q = reg.intern("q");
        let series = jacprod(1, 5, sym_q, 20);

        // Unfiltered jacprodmake should find the correct decomposition
        let unfiltered = jacprodmake(&series, 10);
        assert!(unfiltered.is_exact, "unfiltered jacprodmake should be exact");
        assert!(!unfiltered.factors.is_empty(), "unfiltered should have factors");

        // Filtered with pp=10 (divisors: 2,5,10) -- period 5 is among them
        let filtered = jacprodmake_with_period_filter(&series, 10, 10);
        assert!(filtered.is_exact, "filtered with pp=10 should be exact (5 divides 10)");
        assert_eq!(
            unfiltered.factors, filtered.factors,
            "filtered and unfiltered should produce same factors when PP contains the correct period"
        );

        // Filtered with pp=7 (divisors: 7) -- period 5 is NOT among them
        let bad_filter = jacprodmake_with_period_filter(&series, 10, 7);
        // Should either not be exact or produce different/empty factors
        assert!(
            !bad_filter.is_exact || bad_filter.factors != unfiltered.factors,
            "filtered with pp=7 should not match unfiltered result (period 5 not a divisor of 7)"
        );
    }
}
