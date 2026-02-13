//! Q-binomial (Gaussian) coefficient [n choose k]_q.
//!
//! The q-binomial coefficient is defined as:
//! [n choose k]_q = (q;q)_n / ((q;q)_k * (q;q)_{n-k})
//!
//! Equivalently, using the iterative product formula:
//! [n choose k]_q = prod_{i=1}^{k} (1 - q^{n-k+i}) / (1 - q^i)

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::symbol::SymbolId;

/// Compute the q-binomial (Gaussian) coefficient [n choose k]_q as a formal power series.
///
/// # Arguments
///
/// - `n`: The top parameter (non-negative integer).
/// - `k`: The bottom parameter.
/// - `variable`: The SymbolId for the series variable.
/// - `truncation_order`: Compute to O(q^truncation_order).
///
/// # Returns
///
/// A `FormalPowerSeries` representing [n choose k]_q. The result is always a polynomial
/// of degree k*(n-k).
pub fn qbin(n: i64, k: i64, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    // Edge cases
    if k < 0 || k > n {
        return FormalPowerSeries::zero(variable, truncation_order);
    }
    if k == 0 || k == n {
        return FormalPowerSeries::one(variable, truncation_order);
    }

    // Use the product formula:
    // [n choose k]_q = prod_{i=1}^{k} (1 - q^{n-k+i}) / (1 - q^i)
    //
    // Compute numerator = prod_{i=1}^{k} (1 - q^{n-k+i})
    // Compute denominator = prod_{i=1}^{k} (1 - q^i)
    // Result = numerator * invert(denominator)

    let mut numerator = FormalPowerSeries::one(variable, truncation_order);
    let mut denominator = FormalPowerSeries::one(variable, truncation_order);

    for i in 1..=k {
        // Numerator factor: (1 - q^{n-k+i})
        let num_exp = n - k + i;
        let mut num_factor = FormalPowerSeries::one(variable, truncation_order);
        num_factor.set_coeff(num_exp, -QRat::one());
        numerator = arithmetic::mul(&numerator, &num_factor);

        // Denominator factor: (1 - q^i)
        let mut den_factor = FormalPowerSeries::one(variable, truncation_order);
        den_factor.set_coeff(i, -QRat::one());
        denominator = arithmetic::mul(&denominator, &den_factor);
    }

    let inv_denominator = arithmetic::invert(&denominator);
    arithmetic::mul(&numerator, &inv_denominator)
}
