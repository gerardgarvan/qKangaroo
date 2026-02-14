//! Symbolic identity representations and proving engine.
//!
//! This module provides:
//! - [`jac`]: Jacobi triple product symbolic representation
//! - [`eta`]: Eta quotient symbolic representation with Newman modularity checks
//! - [`cusps`]: Cusp computation for congruence subgroups Gamma_0(N) and Gamma_1(N)
//! - [`orders`]: Order of vanishing at cusps for eta quotients (Ligozat formula)

pub mod jac;
pub mod eta;
pub mod cusps;
pub mod orders;

pub use jac::{JacFactor, JacExpression};
pub use eta::{EtaExpression, ModularityResult};
pub use cusps::{Cusp, cuspmake, cuspmake1, num_cusps_gamma0};
pub use orders::{eta_order_at_cusp, cusp_width, total_order};

use crate::series::{FormalPowerSeries, arithmetic};

/// Raise a formal power series to an integer power (positive, negative, or zero).
///
/// Uses repeated squaring for efficiency. Negative exponents use series inversion.
pub(crate) fn fps_pow(f: &FormalPowerSeries, n: i64) -> FormalPowerSeries {
    if n == 0 {
        return FormalPowerSeries::one(f.variable(), f.truncation_order());
    }
    let (base, exp) = if n < 0 {
        (arithmetic::invert(f), (-n) as u64)
    } else {
        (f.clone(), n as u64)
    };
    let mut result = FormalPowerSeries::one(base.variable(), base.truncation_order());
    let mut power = base;
    let mut e = exp;
    while e > 0 {
        if e & 1 == 1 {
            result = arithmetic::mul(&result, &power);
        }
        e >>= 1;
        if e > 0 {
            power = arithmetic::mul(&power, &power);
        }
    }
    result
}
