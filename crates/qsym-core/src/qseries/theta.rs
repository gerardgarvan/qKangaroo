//! Jacobi theta functions: theta2, theta3, theta4.
//!
//! These are the classical Jacobi theta functions expressed as infinite products
//! and computed as truncated formal power series via [`InfiniteProductGenerator`].
//!
//! # Product representations
//!
//! - `theta3(q) = (q^2; q^2)_inf * (-q; q^2)_inf^2`
//!   = prod_{n>=1}(1 - q^{2n}) * [prod_{n>=0}(1 + q^{2n+1})]^2
//!
//! - `theta4(q) = (q^2; q^2)_inf * (q; q^2)_inf^2`
//!   = prod_{n>=1}(1 - q^{2n}) * [prod_{n>=0}(1 - q^{2n+1})]^2
//!
//! - `theta2(q) = 2*q^{1/4} * prod_{n>=1}(1 - q^{2n})(1 + q^{2n})^2`
//!   Returned as a series in X = q^{1/4} (see [`theta2`] docs).

use crate::number::QRat;
use crate::symbol::SymbolId;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::series::generator::InfiniteProductGenerator;

/// Compute (q^2; q^2)_inf = prod_{n>=1}(1 - q^{2n}).
///
/// This factor is shared by theta3 and theta4.
fn q2_q2_inf(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let initial = FormalPowerSeries::one(variable, truncation_order);
    let num_factors = (truncation_order + 1) / 2;

    let mut ipg = InfiniteProductGenerator::new(
        initial,
        1, // start at n=1
        Box::new(move |n, var, trunc| {
            // Factor n: (1 - q^{2n})
            let mut factor = FormalPowerSeries::one(var, trunc);
            factor.set_coeff(2 * n, -QRat::one());
            factor
        }),
    );
    ipg.ensure_order(num_factors);
    ipg.into_series()
}

/// Compute theta3(q) = prod_{n>=1}(1 - q^{2n}) * [prod_{n>=0}(1 + q^{2n+1})]^2.
///
/// theta3 is the Jacobi theta function whose series expansion has nonzero
/// coefficients only at perfect square exponents:
///
/// theta3(q) = 1 + 2q + 2q^4 + 2q^9 + 2q^16 + 2q^25 + ...
///           = sum_{n=-inf}^{inf} q^{n^2}
///
/// # Arguments
///
/// - `variable`: The SymbolId for the series variable (typically "q").
/// - `truncation_order`: Compute to O(q^truncation_order).
pub fn theta3(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    // Factor 1: (q^2; q^2)_inf = prod_{n>=1}(1 - q^{2n})
    let factor1 = q2_q2_inf(variable, truncation_order);

    // Factor 2: (-q; q^2)_inf = prod_{n>=0}(1 + q^{2n+1})
    let initial2 = FormalPowerSeries::one(variable, truncation_order);
    let num_factors2 = (truncation_order + 1) / 2;

    let mut ipg2 = InfiniteProductGenerator::new(
        initial2,
        0, // start at n=0
        Box::new(move |n, var, trunc| {
            // Factor n: (1 + q^{2n+1})
            let mut factor = FormalPowerSeries::one(var, trunc);
            factor.set_coeff(2 * n + 1, QRat::one());
            factor
        }),
    );
    ipg2.ensure_order(num_factors2);
    let factor2 = ipg2.into_series();

    // theta3 = factor1 * factor2^2
    let factor2_squared = arithmetic::mul(&factor2, &factor2);
    arithmetic::mul(&factor1, &factor2_squared)
}

/// Compute theta4(q) = prod_{n>=1}(1 - q^{2n}) * [prod_{n>=0}(1 - q^{2n+1})]^2.
///
/// theta4 is the Jacobi theta function with alternating signs at perfect squares:
///
/// theta4(q) = 1 - 2q + 2q^4 - 2q^9 + 2q^16 - 2q^25 + ...
///           = sum_{n=-inf}^{inf} (-1)^n q^{n^2}
///
/// # Arguments
///
/// - `variable`: The SymbolId for the series variable (typically "q").
/// - `truncation_order`: Compute to O(q^truncation_order).
pub fn theta4(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    // Factor 1: (q^2; q^2)_inf = prod_{n>=1}(1 - q^{2n})
    let factor1 = q2_q2_inf(variable, truncation_order);

    // Factor 2: (q; q^2)_inf = prod_{n>=0}(1 - q^{2n+1})
    let initial2 = FormalPowerSeries::one(variable, truncation_order);
    let num_factors2 = (truncation_order + 1) / 2;

    let mut ipg2 = InfiniteProductGenerator::new(
        initial2,
        0, // start at n=0
        Box::new(move |n, var, trunc| {
            // Factor n: (1 - q^{2n+1})
            let mut factor = FormalPowerSeries::one(var, trunc);
            factor.set_coeff(2 * n + 1, -QRat::one());
            factor
        }),
    );
    ipg2.ensure_order(num_factors2);
    let factor2 = ipg2.into_series();

    // theta4 = factor1 * factor2^2
    let factor2_squared = arithmetic::mul(&factor2, &factor2);
    arithmetic::mul(&factor1, &factor2_squared)
}

/// Compute theta2(q) as a series in X = q^{1/4}.
///
/// theta2(q) = 2*q^{1/4} * prod_{n>=1}(1 - q^{2n})(1 + q^{2n})^2
///
/// Since our FPS uses integer exponents, we substitute q = X^4 and return
/// a series in X where the `variable` parameter represents q^{1/4}:
///
/// theta2 = 2*X * prod_{n>=1}(1 - X^{8n})(1 + X^{8n})^2
///
/// The returned series has nonzero coefficients only at odd perfect square
/// exponents: 1, 9, 25, 49, 81, 121, 169, ... i.e., (2k+1)^2 for k=0,1,2,...
/// Each such coefficient equals 2.
///
/// # Convention
///
/// The `variable` parameter conceptually represents X = q^{1/4}. The caller
/// should interpret exponent `e` in the result as q^{e/4}.
///
/// # Arguments
///
/// - `variable`: The SymbolId for the series variable (represents q^{1/4}).
/// - `truncation_order`: Compute to O(X^truncation_order).
pub fn theta2(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    // Factor 1: prod_{n>=1}(1 - X^{8n})
    let initial1 = FormalPowerSeries::one(variable, truncation_order);
    let num_factors1 = (truncation_order + 7) / 8 + 1;

    let mut ipg1 = InfiniteProductGenerator::new(
        initial1,
        1, // start at n=1
        Box::new(move |n, var, trunc| {
            // Factor n: (1 - X^{8n})
            let mut factor = FormalPowerSeries::one(var, trunc);
            factor.set_coeff(8 * n, -QRat::one());
            factor
        }),
    );
    ipg1.ensure_order(num_factors1);
    let factor1 = ipg1.into_series();

    // Factor 2: prod_{n>=1}(1 + X^{8n})
    let initial2 = FormalPowerSeries::one(variable, truncation_order);

    let mut ipg2 = InfiniteProductGenerator::new(
        initial2,
        1, // start at n=1
        Box::new(move |n, var, trunc| {
            // Factor n: (1 + X^{8n})
            let mut factor = FormalPowerSeries::one(var, trunc);
            factor.set_coeff(8 * n, QRat::one());
            factor
        }),
    );
    ipg2.ensure_order(num_factors1);
    let factor2 = ipg2.into_series();

    // Compute: factor1 * factor2^2
    let factor2_squared = arithmetic::mul(&factor2, &factor2);
    let product = arithmetic::mul(&factor1, &factor2_squared);

    // Multiply by the prefactor 2*X (monomial with coeff 2 at exponent 1)
    let prefactor = FormalPowerSeries::monomial(
        variable,
        QRat::from((2, 1i64)),
        1,
        truncation_order,
    );
    arithmetic::mul(&prefactor, &product)
}
