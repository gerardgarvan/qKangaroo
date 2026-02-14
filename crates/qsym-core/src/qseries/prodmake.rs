//! Andrews' algorithm (prodmake): recover infinite product exponents from series
//! coefficients.
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
fn mobius(n: i64) -> i64 {
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
fn divisors(n: i64) -> Vec<i64> {
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
}
