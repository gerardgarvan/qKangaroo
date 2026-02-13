//! Named infinite product functions for q-series.
//!
//! This module provides the five named infinite product functions from Garvan's
//! q-series package:
//! - [`etaq`]: generalized eta product (q^b; q^t)_inf
//! - [`jacprod`]: Jacobi triple product JAC(a,b) = (q^a;q^b)(q^{b-a};q^b)(q^b;q^b)
//! - [`tripleprod`]: Jacobi triple product with monomial parameter z
//! - [`quinprod`]: quintuple product identity
//! - [`winquist`]: Winquist's identity product

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::series::generator::{InfiniteProductGenerator, euler_function_generator, qpochhammer_inf_generator};
use crate::symbol::SymbolId;

#[allow(unused_imports)]
use super::QMonomial;

/// Compute the generalized eta product: (q^b; q^t)_inf = prod_{n=0}^{inf} (1 - q^{b + t*n})
///
/// # Arguments
///
/// - `b`: base exponent (must be > 0 for a nonzero result)
/// - `t`: step size (must be > 0)
/// - `variable`: the series variable
/// - `truncation_order`: compute to O(q^truncation_order)
///
/// # Special cases
///
/// - If b <= 0, the first factor has exponent b <= 0, meaning (1 - q^0) = 0 or similar,
///   making the entire product zero for b == 0 (since factor is 1-1=0).
///   For b < 0, factors with negative exponents also vanish.
///   We handle b <= 0 by returning the zero series.
pub fn etaq(b: i64, t: i64, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    assert!(t > 0, "etaq: step t must be positive, got {}", t);

    // If b <= 0: factor at n=0 is (1 - q^b). For b=0, that's (1-1)=0.
    // For b < 0, q^b has a negative exponent. In either case the product vanishes.
    if b <= 0 {
        return FormalPowerSeries::zero(variable, truncation_order);
    }

    // Build a custom InfiniteProductGenerator.
    // Factor n: (1 - q^{b + t*n})
    // Start index: 0
    // We need factors for n = 0, 1, 2, ... until b + t*n >= truncation_order.
    // The maximum number of factors needed: ceil((truncation_order - b) / t) + 1
    let initial = FormalPowerSeries::one(variable, truncation_order);
    let b_cap = b;
    let t_cap = t;

    let mut ipg = InfiniteProductGenerator::new(
        initial,
        0,
        Box::new(move |n, var, trunc| {
            let exp = b_cap + t_cap * n;
            let mut factor = FormalPowerSeries::one(var, trunc);
            if exp >= 0 && exp < trunc {
                factor.set_coeff(exp, -QRat::one());
            }
            factor
        }),
    );

    // Compute how many factors we actually need
    let max_factors = if truncation_order > b {
        ((truncation_order - b + t - 1) / t) + 1
    } else {
        1 // at least one factor
    };
    ipg.ensure_order(max_factors.max(1));
    ipg.into_series()
}

/// Compute the Jacobi triple product: JAC(a, b) = (q^a; q^b)_inf * (q^{b-a}; q^b)_inf * (q^b; q^b)_inf
///
/// # Arguments
///
/// - `a`: first parameter (must satisfy 0 < a < b)
/// - `b`: modulus parameter (must be > a > 0)
/// - `variable`: the series variable
/// - `truncation_order`: compute to O(q^truncation_order)
///
/// # Panics
///
/// Panics if a <= 0 or a >= b.
pub fn jacprod(a: i64, b: i64, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    assert!(a > 0 && a < b, "jacprod: requires 0 < a < b, got a={}, b={}", a, b);

    let p1 = etaq(a, b, variable, truncation_order);
    let p2 = etaq(b - a, b, variable, truncation_order);
    let p3 = etaq(b, b, variable, truncation_order);
    let temp = arithmetic::mul(&p1, &p2);
    arithmetic::mul(&temp, &p3)
}

/// Compute the Jacobi triple product with monomial parameter z.
///
/// tripleprod(z, q, T) = prod_{n>=1}(1-q^n) * prod_{n>=0}(1 - z*q^n) * prod_{n>=1}(1 - q^n/z)
///
/// Where z = c * q^m is a QMonomial.
///
/// # Panics
///
/// Panics if z.coeff is zero.
pub fn tripleprod(z: &QMonomial, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    assert!(!z.coeff.is_zero(), "tripleprod: z coefficient must be nonzero");

    let c = &z.coeff;
    let m = z.power;

    // Factor 1: (q;q)_inf = prod_{n>=1}(1 - q^n)
    let mut euler_gen = euler_function_generator(variable, truncation_order);
    euler_gen.ensure_order(truncation_order);
    let f1 = euler_gen.into_series();

    // Factor 2: prod_{n>=0}(1 - c*q^{m+n})
    // This is (c*q^m; q)_inf via qpochhammer_inf_generator(c, m, var, trunc)
    // Special case: if c==1 and m==0, first factor is (1-1)=0, product is zero.
    if *c == QRat::one() && m == 0 {
        return FormalPowerSeries::zero(variable, truncation_order);
    }
    let mut gen2 = qpochhammer_inf_generator(c.clone(), m, variable, truncation_order);
    gen2.ensure_order(truncation_order);
    let f2 = gen2.into_series();

    // Factor 3: prod_{n>=1}(1 - q^n/z) = prod_{n>=1}(1 - (1/c)*q^{n-m})
    // = (q^{1-m}/c; q)_inf = qpochhammer_inf_generator(1/c, 1-m, var, trunc)
    // First factor (n=1 / k=0): exponent = 1-m. If 1/c == 1 and 1-m == 0 => c==1, m==1,
    // then first factor is (1-1)=0.
    let inv_c = QRat::one() / c.clone();
    if inv_c == QRat::one() && (1 - m) == 0 {
        return FormalPowerSeries::zero(variable, truncation_order);
    }
    let mut gen3 = qpochhammer_inf_generator(inv_c, 1 - m, variable, truncation_order);
    gen3.ensure_order(truncation_order);
    let f3 = gen3.into_series();

    // Multiply all three factors
    let temp = arithmetic::mul(&f1, &f2);
    arithmetic::mul(&temp, &f3)
}

/// Compute the quintuple product.
///
/// quinprod(z, q, T) = prod_{n>=1}(1-q^n)(1-z*q^n)(1-z^{-1}*q^{n-1})(1-z^2*q^{2n-1})(1-z^{-2}*q^{2n-1})
///
/// Where z = c * q^m is a QMonomial.
///
/// # Panics
///
/// Panics if z.coeff is zero.
pub fn quinprod(z: &QMonomial, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    assert!(!z.coeff.is_zero(), "quinprod: z coefficient must be nonzero");

    let c = &z.coeff;
    let m = z.power;

    // Factor 1: (q;q)_inf -- euler function
    let mut euler_gen = euler_function_generator(variable, truncation_order);
    euler_gen.ensure_order(truncation_order);
    let f1 = euler_gen.into_series();

    // Factor 2: prod_{n>=1}(1 - c*q^{m+n}) = (c*q^{m+1}; q)_inf
    let mut gen2 = qpochhammer_inf_generator(c.clone(), m + 1, variable, truncation_order);
    gen2.ensure_order(truncation_order);
    let f2 = gen2.into_series();

    // Factor 3: prod_{n>=1}(1 - (1/c)*q^{n-1-m}) = ((1/c)*q^{-m}; q)_inf
    let inv_c = QRat::one() / c.clone();
    let mut gen3 = qpochhammer_inf_generator(inv_c.clone(), -m, variable, truncation_order);
    gen3.ensure_order(truncation_order);
    let f3 = gen3.into_series();

    // Factor 4: prod_{n>=1}(1 - c^2 * q^{2m + 2n - 1})
    // Exponent for n-th factor: 2m + 2n - 1. For n=1: 2m+1, n=2: 2m+3, ...
    // This is a step-2 product starting at exponent 2m+1.
    let c_sq = c.clone() * c.clone();
    let base4 = 2 * m + 1;
    let f4 = custom_step_product(c_sq.clone(), base4, 2, variable, truncation_order);

    // Factor 5: prod_{n>=1}(1 - (1/c^2) * q^{2n - 1 - 2m})
    // Exponent for n-th factor: 2n - 1 - 2m. For n=1: 1-2m, n=2: 3-2m, ...
    // This is a step-2 product starting at exponent 1-2m.
    let inv_c_sq = inv_c.clone() * inv_c.clone();
    let base5 = 1 - 2 * m;
    let f5 = custom_step_product(inv_c_sq, base5, 2, variable, truncation_order);

    // Multiply all 5 factors
    let temp = arithmetic::mul(&f1, &f2);
    let temp = arithmetic::mul(&temp, &f3);
    let temp = arithmetic::mul(&temp, &f4);
    arithmetic::mul(&temp, &f5)
}

/// Compute Winquist's identity product.
///
/// winquist(a, b, q, T) = (q;q)_inf^2 * prod of 8 q-Pochhammer factors
///
/// Where a = a_c * q^{a_p}, b = b_c * q^{b_p}.
///
/// The 8 factors are:
/// (a_c*q^{a_p}; q)_inf * (a_c^{-1}*q^{1-a_p}; q)_inf *
/// (b_c*q^{b_p}; q)_inf * (b_c^{-1}*q^{1-b_p}; q)_inf *
/// (a_c*b_c*q^{a_p+b_p}; q)_inf * (a_c^{-1}*b_c^{-1}*q^{2-a_p-b_p}; q)_inf *
/// (a_c*b_c^{-1}*q^{a_p-b_p}; q)_inf * (a_c^{-1}*b_c*q^{1-a_p+b_p}; q)_inf
///
/// # Panics
///
/// Panics if a.coeff or b.coeff is zero.
pub fn winquist(a: &QMonomial, b: &QMonomial, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    assert!(!a.coeff.is_zero(), "winquist: a coefficient must be nonzero");
    assert!(!b.coeff.is_zero(), "winquist: b coefficient must be nonzero");

    let ac = &a.coeff;
    let ap = a.power;
    let bc = &b.coeff;
    let bp = b.power;

    // (q;q)_inf^2
    let mut euler_gen = euler_function_generator(variable, truncation_order);
    euler_gen.ensure_order(truncation_order);
    let euler = euler_gen.into_series();
    let euler_sq = arithmetic::mul(&euler, &euler);

    let inv_ac = QRat::one() / ac.clone();
    let inv_bc = QRat::one() / bc.clone();

    // 8 Pochhammer factors, each is (coeff * q^offset; q)_inf
    let factors: Vec<(QRat, i64)> = vec![
        (ac.clone(),                        ap),
        (inv_ac.clone(),                    1 - ap),
        (bc.clone(),                        bp),
        (inv_bc.clone(),                    1 - bp),
        (ac.clone() * bc.clone(),           ap + bp),
        (inv_ac.clone() * inv_bc.clone(),   2 - ap - bp),
        (ac.clone() * inv_bc.clone(),       ap - bp),
        (inv_ac.clone() * bc.clone(),       1 - ap + bp),
    ];

    let mut result = euler_sq;
    for (coeff, offset) in factors {
        // Check for zero-product case: if coeff==1 and offset==0, product vanishes
        if coeff == QRat::one() && offset == 0 {
            return FormalPowerSeries::zero(variable, truncation_order);
        }
        let mut ipg = qpochhammer_inf_generator(coeff, offset, variable, truncation_order);
        ipg.ensure_order(truncation_order);
        let factor_series = ipg.into_series();
        result = arithmetic::mul(&result, &factor_series);
    }

    result
}

/// Helper: build an infinite product with custom step.
///
/// Computes prod_{n>=0}(1 - coeff * q^{base + step*n}) for n = 0, 1, 2, ...
/// until base + step*n >= truncation_order.
///
/// If the first factor has exponent < 0 or == 0 with coeff == 1, appropriate
/// handling is done (though the caller should check for vanishing cases).
fn custom_step_product(
    coeff: QRat,
    base: i64,
    step: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    assert!(step > 0, "custom_step_product: step must be positive");

    let initial = FormalPowerSeries::one(variable, truncation_order);
    let coeff_cap = coeff.clone();
    let base_cap = base;
    let step_cap = step;

    let mut ipg = InfiniteProductGenerator::new(
        initial,
        0,
        Box::new(move |n, var, trunc| {
            let exp = base_cap + step_cap * n;
            let mut factor = FormalPowerSeries::one(var, trunc);
            if exp >= 0 && exp < trunc {
                factor.set_coeff(exp, -coeff_cap.clone());
            }
            factor
        }),
    );

    // Number of factors needed: until base + step*n >= truncation_order
    let max_factors = if truncation_order > base {
        ((truncation_order - base + step - 1) / step) + 1
    } else {
        // All factors have exponent >= truncation_order, so they're all identity
        // We still need at least 1 factor in case base < trunc from negative base
        if base < 0 {
            // Some factors have negative exponents that get ignored, but we need
            // to process until exponent reaches trunc
            ((truncation_order - base + step - 1) / step) + 1
        } else {
            1
        }
    };

    ipg.ensure_order(max_factors.max(1));
    ipg.into_series()
}
