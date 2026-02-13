//! General q-Pochhammer symbol: (a;q)_n for finite and infinite orders.
//!
//! The q-Pochhammer symbol (a;q)_n is defined as:
//! - n = 0: 1
//! - n > 0: prod_{k=0}^{n-1} (1 - a * q^k)
//! - n < 0: 1 / prod_{k=1}^{|n|} (1 - a * q^{-k}) = 1 / (a*q^n; q)_{|n|}
//! - n = inf: prod_{k=0}^{inf} (1 - a * q^k)
//!
//! Here `a` is a QMonomial `c * q^m`, so the actual factor at index k is
//! `(1 - c * q^{m+k})` for positive order.

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::series::generator::qpochhammer_inf_generator;
use crate::symbol::SymbolId;

use super::{QMonomial, PochhammerOrder};

/// Compute the general q-Pochhammer symbol (a; q)_n as a formal power series.
///
/// # Arguments
///
/// - `a`: The base monomial `c * q^m`.
/// - `variable`: The SymbolId for the series variable.
/// - `n`: The order (finite or infinite).
/// - `truncation_order`: Compute to O(q^truncation_order).
///
/// # Returns
///
/// A `FormalPowerSeries` representing (a; q)_n truncated at the given order.
pub fn aqprod(
    a: &QMonomial,
    variable: SymbolId,
    n: PochhammerOrder,
    truncation_order: i64,
) -> FormalPowerSeries {
    match n {
        PochhammerOrder::Finite(0) => {
            FormalPowerSeries::one(variable, truncation_order)
        }
        PochhammerOrder::Finite(k) if k > 0 => {
            aqprod_finite_positive(a, variable, k, truncation_order)
        }
        PochhammerOrder::Finite(k) => {
            // k < 0
            aqprod_finite_negative(a, variable, k, truncation_order)
        }
        PochhammerOrder::Infinite => {
            aqprod_infinite(a, variable, truncation_order)
        }
    }
}

/// Compute (a;q)_n for n > 0: prod_{k=0}^{n-1} (1 - a.coeff * q^{a.power + k}).
fn aqprod_finite_positive(
    a: &QMonomial,
    variable: SymbolId,
    n: i64,
    truncation_order: i64,
) -> FormalPowerSeries {
    // Check if any factor vanishes: factor k is (1 - a.coeff * q^{a.power + k}).
    // This is zero when a.coeff == 1 and a.power + k == 0, i.e., k == -a.power.
    // For k in 0..n, the factor vanishes if a.coeff == 1 and 0 <= -a.power < n.
    if a.coeff == QRat::one() {
        let neg_power = -a.power;
        if neg_power >= 0 && neg_power < n {
            // Factor at k = -a.power is (1 - 1 * q^0) = 0
            return FormalPowerSeries::zero(variable, truncation_order);
        }
    }

    // If a.coeff is zero, all factors are (1 - 0) = 1, product is 1.
    if a.coeff.is_zero() {
        return FormalPowerSeries::one(variable, truncation_order);
    }

    // Build each factor (1 - a.coeff * q^{a.power + k}) and multiply sequentially.
    let mut result = FormalPowerSeries::one(variable, truncation_order);

    for k in 0..n {
        let exponent = a.power + k;
        // Factor = 1 - a.coeff * q^exponent
        let mut factor = FormalPowerSeries::one(variable, truncation_order);
        factor.set_coeff(exponent, -a.coeff.clone());
        result = arithmetic::mul(&result, &factor);
    }

    result
}

/// Compute (a;q)_n for n < 0:
/// (a;q)_{-|n|} = 1 / (a * q^n; q)_{|n|}
///
/// The shifted monomial has coeff = a.coeff, power = a.power + n.
fn aqprod_finite_negative(
    a: &QMonomial,
    variable: SymbolId,
    n: i64, // n < 0
    truncation_order: i64,
) -> FormalPowerSeries {
    let abs_n = -n; // |n|

    // Shifted a: coeff stays the same, power shifts by n (which is negative)
    let shifted_a = QMonomial::new(a.coeff.clone(), a.power + n);

    // Compute (shifted_a; q)_{|n|}
    let denominator = aqprod_finite_positive(&shifted_a, variable, abs_n, truncation_order);

    // Invert: 1 / denominator
    arithmetic::invert(&denominator)
}

/// Compute (a;q)_inf using the existing qpochhammer_inf_generator.
fn aqprod_infinite(
    a: &QMonomial,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    // Special case: if a.coeff == 1 and a.power == 0, first factor is (1-1)=0
    if a.coeff == QRat::one() && a.power == 0 {
        return FormalPowerSeries::zero(variable, truncation_order);
    }

    // If a.coeff is zero, all factors are 1, product is 1.
    if a.coeff.is_zero() {
        return FormalPowerSeries::one(variable, truncation_order);
    }

    let mut ipg = qpochhammer_inf_generator(
        a.coeff.clone(),
        a.power,
        variable,
        truncation_order,
    );
    ipg.ensure_order(truncation_order);
    ipg.into_series()
}
