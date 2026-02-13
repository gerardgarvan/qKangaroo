//! Arithmetic operations on formal power series.
//!
//! All binary operations assert that both series use the same variable.
//! Truncation order is propagated correctly: binary ops use min(a, b).

use crate::number::QRat;
use super::FormalPowerSeries;

/// Add two formal power series, truncating to min precision.
/// Time: O(|a| + |b|), Space: O(|a| + |b|)
pub fn add(a: &FormalPowerSeries, b: &FormalPowerSeries) -> FormalPowerSeries {
    assert_eq!(a.variable, b.variable, "Cannot add series in different variables");
    todo!("add not yet implemented")
}

/// Subtract two formal power series: a - b.
pub fn sub(a: &FormalPowerSeries, b: &FormalPowerSeries) -> FormalPowerSeries {
    assert_eq!(a.variable, b.variable, "Cannot subtract series in different variables");
    todo!("sub not yet implemented")
}

/// Negate a formal power series: -f(q).
pub fn negate(a: &FormalPowerSeries) -> FormalPowerSeries {
    todo!("negate not yet implemented")
}

/// Multiply a formal power series by a scalar (QRat).
pub fn scalar_mul(s: &QRat, a: &FormalPowerSeries) -> FormalPowerSeries {
    todo!("scalar_mul not yet implemented")
}

/// Multiply two formal power series, truncating during computation.
/// Time: O(|a| * |b|), Space: O(N) where N = truncation order
pub fn mul(a: &FormalPowerSeries, b: &FormalPowerSeries) -> FormalPowerSeries {
    assert_eq!(a.variable, b.variable, "Cannot multiply series in different variables");
    todo!("mul not yet implemented")
}

/// Invert a formal power series: compute 1/f(q).
/// Requires f(0) != 0 (panics otherwise).
/// Uses the recurrence: c[n] = (-1/a0) * sum_{k=1}^{n} a[k]*c[n-k]
pub fn invert(a: &FormalPowerSeries) -> FormalPowerSeries {
    let a0 = a.coeff(0);
    assert!(!a0.is_zero(), "Cannot invert series with zero constant term");
    todo!("invert not yet implemented")
}

/// Shift a series by k: multiply by q^k.
/// shift(f, k) produces q^k * f(q) with truncation_order = f.truncation_order + k.
pub fn shift(a: &FormalPowerSeries, k: i64) -> FormalPowerSeries {
    todo!("shift not yet implemented")
}
