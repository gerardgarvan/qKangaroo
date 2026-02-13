//! Partition functions: counting partitions and generating functions for
//! restricted partition types.
//!
//! - [`partition_count`]: p(n) via pentagonal number recurrence, O(n * sqrt(n))
//! - [`partition_gf`]: sum_{n>=0} p(n) q^n = 1/(q;q)_inf as FPS
//! - [`distinct_parts_gf`]: prod_{n>=1}(1+q^n) = (-q;q)_inf (OEIS A000009)
//! - [`odd_parts_gf`]: prod_{k>=0} 1/(1-q^{2k+1}) -- Euler's theorem dual
//! - [`bounded_parts_gf`]: prod_{k=1}^{m} 1/(1-q^k) -- at most m parts

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::series::generator::{euler_function_generator, qpochhammer_inf_generator};
use crate::symbol::SymbolId;

/// Compute p(n), the number of partitions of n, using the pentagonal number
/// recurrence. Runs in O(n * sqrt(n)) time.
///
/// The recurrence is:
///   p(i) = sum_{k=1}^{...} (-1)^{k+1} * [p(i - k(3k-1)/2) + p(i - k(3k+1)/2)]
///
/// # Returns
///
/// p(n) as a QRat (always a non-negative integer).
///
/// # Edge cases
///
/// - n < 0: returns 0
/// - n == 0: returns 1
pub fn partition_count(n: i64) -> QRat {
    if n < 0 {
        return QRat::zero();
    }
    if n == 0 {
        return QRat::one();
    }

    let nu = n as usize;
    let mut table = vec![QRat::zero(); nu + 1];
    table[0] = QRat::one();

    for i in 1..=nu {
        let mut sum = QRat::zero();
        let mut k: i64 = 1;
        loop {
            // Generalized pentagonal numbers:
            // g1 = k*(3k-1)/2  (for positive k)
            // g2 = k*(3k+1)/2  (for negative k, i.e., k -> -k gives k*(3k+1)/2)
            let g1 = (k * (3 * k - 1) / 2) as usize;
            if g1 > i {
                break;
            }
            let sign = if k % 2 == 1 {
                QRat::one()
            } else {
                -QRat::one()
            };

            sum = sum + sign.clone() * table[i - g1].clone();

            let g2 = (k * (3 * k + 1) / 2) as usize;
            if g2 <= i {
                sum = sum + sign * table[i - g2].clone();
            }

            k += 1;
        }
        table[i] = sum;
    }

    table[nu].clone()
}

/// Compute the full partition generating function:
///   sum_{n>=0} p(n) q^n = 1/(q;q)_inf
///
/// Uses the Euler function generator and series inversion.
pub fn partition_gf(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut ipg = euler_function_generator(variable, truncation_order);
    ipg.ensure_order(truncation_order);
    let euler = ipg.into_series();
    arithmetic::invert(&euler)
}

/// Generating function for partitions into distinct parts:
///   Q(q) = prod_{n>=1}(1 + q^n) = (-q; q)_inf
///
/// Coefficients are OEIS A000009.
///
/// Uses `qpochhammer_inf_generator(-1, 1, variable, truncation_order)` which
/// computes prod_{k>=0}(1 - (-1)*q^{1+k}) = prod_{k>=0}(1 + q^{k+1}) = prod_{n>=1}(1+q^n).
pub fn distinct_parts_gf(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut ipg = qpochhammer_inf_generator(-QRat::one(), 1, variable, truncation_order);
    ipg.ensure_order(truncation_order);
    ipg.into_series()
}

/// Generating function for partitions into odd parts:
///   prod_{k>=0} 1/(1 - q^{2k+1})
///
/// By Euler's theorem, this equals `distinct_parts_gf` coefficient-by-coefficient.
///
/// Implemented by computing the product prod_{k>=0}(1 - q^{2k+1}) and then inverting.
pub fn odd_parts_gf(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    // Build the product prod_{k=0}^{...} (1 - q^{2k+1})
    // Factor k has exponent 2k+1. We need factors while 2k+1 < truncation_order.
    let mut product = FormalPowerSeries::one(variable, truncation_order);

    let mut k: i64 = 0;
    while 2 * k + 1 < truncation_order {
        let exp = 2 * k + 1;
        let mut factor = FormalPowerSeries::one(variable, truncation_order);
        factor.set_coeff(exp, -QRat::one());
        product = arithmetic::mul(&product, &factor);
        k += 1;
    }

    arithmetic::invert(&product)
}

/// Generating function for partitions with at most `max_parts` parts:
///   prod_{k=1}^{max_parts} 1/(1-q^k)
///
/// Also counts partitions with largest part <= max_parts.
///
/// # Edge cases
///
/// - max_parts <= 0: returns 1 (empty product, inverted = 1)
pub fn bounded_parts_gf(
    max_parts: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    if max_parts <= 0 {
        return FormalPowerSeries::one(variable, truncation_order);
    }

    // Build prod_{k=1}^{max_parts} (1 - q^k)
    let mut product = FormalPowerSeries::one(variable, truncation_order);

    for k in 1..=max_parts {
        let mut factor = FormalPowerSeries::one(variable, truncation_order);
        factor.set_coeff(k, -QRat::one());
        product = arithmetic::mul(&product, &factor);
    }

    arithmetic::invert(&product)
}
