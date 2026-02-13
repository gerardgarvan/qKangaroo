//! Arithmetic operations on formal power series.
//!
//! All binary operations assert that both series use the same variable.
//! Truncation order is propagated correctly: binary ops use min(a, b).

use std::collections::BTreeMap;

use crate::number::QRat;
use super::FormalPowerSeries;

/// Add two formal power series, truncating to min precision.
/// Time: O(|a| + |b|), Space: O(|a| + |b|)
pub fn add(a: &FormalPowerSeries, b: &FormalPowerSeries) -> FormalPowerSeries {
    assert_eq!(a.variable, b.variable, "Cannot add series in different variables");
    let trunc = a.truncation_order.min(b.truncation_order);
    let mut result = FormalPowerSeries::zero(a.variable, trunc);

    // Copy coefficients from a (only below truncation)
    for (&k, v) in &a.coefficients {
        if k < trunc {
            result.set_coeff(k, v.clone());
        }
    }
    // Add coefficients from b
    for (&k, v) in &b.coefficients {
        if k < trunc {
            let existing = result.coeff(k);
            let sum = existing + v.clone();
            result.set_coeff(k, sum);
        }
    }
    result
}

/// Subtract two formal power series: a - b.
pub fn sub(a: &FormalPowerSeries, b: &FormalPowerSeries) -> FormalPowerSeries {
    assert_eq!(a.variable, b.variable, "Cannot subtract series in different variables");
    add(a, &negate(b))
}

/// Negate a formal power series: -f(q).
pub fn negate(a: &FormalPowerSeries) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(a.variable, a.truncation_order);
    for (&k, v) in &a.coefficients {
        result.coefficients.insert(k, -v.clone());
    }
    result
}

/// Multiply a formal power series by a scalar (QRat).
pub fn scalar_mul(s: &QRat, a: &FormalPowerSeries) -> FormalPowerSeries {
    if s.is_zero() {
        return FormalPowerSeries::zero(a.variable, a.truncation_order);
    }
    let mut result = FormalPowerSeries::zero(a.variable, a.truncation_order);
    for (&k, v) in &a.coefficients {
        let product = s.clone() * v.clone();
        if !product.is_zero() {
            result.coefficients.insert(k, product);
        }
    }
    result
}

/// Multiply two formal power series, truncating during computation.
///
/// CRITICAL: checks `ka + kb < trunc` BEFORE computing each product term.
/// Since BTreeMap iterates in ascending order, once `ka + kb >= trunc`,
/// we break the inner loop.
///
/// Time: O(|a| * |b|), Space: O(N) where N = truncation order
pub fn mul(a: &FormalPowerSeries, b: &FormalPowerSeries) -> FormalPowerSeries {
    assert_eq!(a.variable, b.variable, "Cannot multiply series in different variables");
    let trunc = a.truncation_order.min(b.truncation_order);
    let mut coeffs: BTreeMap<i64, QRat> = BTreeMap::new();

    for (&ka, ca) in &a.coefficients {
        if ka >= trunc {
            break; // a is sorted ascending, all remaining ka >= trunc
        }
        for (&kb, cb) in &b.coefficients {
            let k = ka + kb;
            if k >= trunc {
                break; // b is sorted ascending, all remaining kb even larger
            }
            let product = ca.clone() * cb.clone();
            let entry = coeffs.entry(k).or_insert_with(QRat::zero);
            *entry = entry.clone() + product;
        }
    }

    // Clean up zeros from cancellation
    coeffs.retain(|_, v| !v.is_zero());

    FormalPowerSeries {
        coefficients: coeffs,
        variable: a.variable,
        truncation_order: trunc,
    }
}

/// Invert a formal power series: compute 1/f(q).
///
/// Requires f(0) != 0 (panics otherwise).
/// Uses the recurrence: c[0] = 1/a0, c[n] = (-1/a0) * sum_{k=1}^{n} a[k]*c[n-k]
pub fn invert(a: &FormalPowerSeries) -> FormalPowerSeries {
    let a0 = a.coeff(0);
    assert!(!a0.is_zero(), "Cannot invert series with zero constant term");
    let trunc = a.truncation_order;
    let inv_a0 = QRat::one() / a0;

    let mut result = FormalPowerSeries::zero(a.variable, trunc);
    result.set_coeff(0, inv_a0.clone());

    let neg_inv_a0 = -inv_a0;

    for n in 1..trunc {
        let mut sum = QRat::zero();
        // sum_{k=1}^{n} a[k] * c[n-k]
        for k in 1..=n {
            let ak = a.coeff(k);
            if ak.is_zero() {
                continue;
            }
            let cn_k = result.coeff(n - k);
            if cn_k.is_zero() {
                continue;
            }
            sum = sum + ak * cn_k;
        }
        let cn = neg_inv_a0.clone() * sum;
        result.set_coeff(n, cn);
    }
    result
}

/// Shift a series by k: multiply by q^k.
///
/// shift(f, k) produces q^k * f(q) with truncation_order = f.truncation_order + k.
/// Each exponent p in the original series moves to p + k.
pub fn shift(a: &FormalPowerSeries, k: i64) -> FormalPowerSeries {
    let new_trunc = a.truncation_order + k;
    let mut result = FormalPowerSeries::zero(a.variable, new_trunc);
    for (&p, v) in &a.coefficients {
        let new_p = p + k;
        if new_p < new_trunc {
            result.coefficients.insert(new_p, v.clone());
        }
    }
    result
}
