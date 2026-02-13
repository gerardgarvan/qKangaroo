//! Comprehensive tests for FormalPowerSeries construction and arithmetic.
//!
//! TDD RED phase: all tests should compile but fail (due to todo!() panics
//! in arithmetic and display stubs).

use qsym_core::number::QRat;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;
use qsym_core::series::FormalPowerSeries;
use qsym_core::series::arithmetic;
use std::collections::BTreeMap;

/// Helper: create a SymbolId for "q" using ExprArena's symbol registry.
fn q_var() -> SymbolId {
    let mut arena = ExprArena::new();
    arena.symbols_mut().intern("q")
}

/// Helper: create QRat from an i64 value.
fn qrat(n: i64) -> QRat {
    QRat::from((n, 1i64))
}

/// Helper: create QRat from numerator/denominator.
fn qrat_frac(num: i64, den: i64) -> QRat {
    QRat::from((num, den))
}

// ===========================================================================
// 1. Construction tests
// ===========================================================================

#[test]
fn zero_series_is_zero() {
    let q = q_var();
    let z = FormalPowerSeries::zero(q, 10);
    assert!(z.is_zero());
    assert_eq!(z.num_nonzero(), 0);
    assert_eq!(z.truncation_order(), 10);
    assert_eq!(z.variable(), q);
    assert_eq!(z.min_order(), None);
}

#[test]
fn one_series_constant() {
    let q = q_var();
    let one = FormalPowerSeries::one(q, 10);
    assert!(!one.is_zero());
    assert_eq!(one.coeff(0), QRat::one());
    assert_eq!(one.coeff(1), QRat::zero());
    assert_eq!(one.coeff(9), QRat::zero());
    assert_eq!(one.num_nonzero(), 1);
}

#[test]
fn monomial_construction() {
    let q = q_var();
    let m = FormalPowerSeries::monomial(q, qrat(3), 5, 10);
    assert_eq!(m.coeff(5), qrat(3));
    assert_eq!(m.coeff(0), QRat::zero());
    assert_eq!(m.coeff(4), QRat::zero());
    assert_eq!(m.num_nonzero(), 1);
    assert_eq!(m.min_order(), Some(5));
}

#[test]
fn from_coeffs_strips_zeros() {
    let q = q_var();
    let mut coeffs = BTreeMap::new();
    coeffs.insert(0, qrat(1));
    coeffs.insert(1, QRat::zero()); // should be stripped
    coeffs.insert(2, qrat(3));
    let fps = FormalPowerSeries::from_coeffs(q, coeffs, 10);
    assert_eq!(fps.num_nonzero(), 2); // only 0 and 2, not 1
    assert_eq!(fps.coeff(0), qrat(1));
    assert_eq!(fps.coeff(1), QRat::zero());
    assert_eq!(fps.coeff(2), qrat(3));
}

#[test]
#[should_panic(expected = "Cannot access coefficient")]
fn coeff_panics_above_truncation() {
    let q = q_var();
    let z = FormalPowerSeries::zero(q, 10);
    z.coeff(10); // should panic: exponent == truncation_order
}

#[test]
fn monomial_beyond_truncation_is_zero() {
    let q = q_var();
    // Power >= truncation_order should result in zero series
    let m = FormalPowerSeries::monomial(q, qrat(5), 10, 10);
    assert!(m.is_zero());
}

#[test]
fn set_coeff_removes_zero() {
    let q = q_var();
    let mut fps = FormalPowerSeries::one(q, 10);
    assert_eq!(fps.num_nonzero(), 1);
    fps.set_coeff(0, QRat::zero());
    assert_eq!(fps.num_nonzero(), 0);
    assert!(fps.is_zero());
}

#[test]
fn equality_checks_all_fields() {
    let q = q_var();
    let a = FormalPowerSeries::one(q, 10);
    let b = FormalPowerSeries::one(q, 10);
    let c = FormalPowerSeries::one(q, 20); // different truncation
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn iter_returns_nonzero_ascending() {
    let q = q_var();
    let mut coeffs = BTreeMap::new();
    coeffs.insert(5, qrat(2));
    coeffs.insert(1, qrat(3));
    coeffs.insert(8, qrat(-1));
    let fps = FormalPowerSeries::from_coeffs(q, coeffs, 10);
    let pairs: Vec<(i64, QRat)> = fps.iter().map(|(&k, v)| (k, v.clone())).collect();
    assert_eq!(pairs.len(), 3);
    assert_eq!(pairs[0].0, 1);
    assert_eq!(pairs[1].0, 5);
    assert_eq!(pairs[2].0, 8);
}

// ===========================================================================
// 2. Addition tests
// ===========================================================================

#[test]
fn add_zero_identity() {
    let q = q_var();
    let f = FormalPowerSeries::one(q, 10);
    let z = FormalPowerSeries::zero(q, 10);
    let result = arithmetic::add(&f, &z);
    assert_eq!(result, f);
}

#[test]
fn add_disjoint_support() {
    let q = q_var();
    let a = FormalPowerSeries::monomial(q, qrat(1), 0, 10);
    let b = FormalPowerSeries::monomial(q, qrat(1), 1, 10);
    let result = arithmetic::add(&a, &b);
    assert_eq!(result.coeff(0), qrat(1));
    assert_eq!(result.coeff(1), qrat(1));
    assert_eq!(result.num_nonzero(), 2);
}

#[test]
fn add_overlapping_cancellation() {
    let q = q_var();
    // {0: 1, 1: 1} + {0: -1} = {1: 1}
    let mut coeffs_a = BTreeMap::new();
    coeffs_a.insert(0, qrat(1));
    coeffs_a.insert(1, qrat(1));
    let a = FormalPowerSeries::from_coeffs(q, coeffs_a, 10);

    let b = FormalPowerSeries::monomial(q, qrat(-1), 0, 10);
    let result = arithmetic::add(&a, &b);
    assert_eq!(result.coeff(0), QRat::zero());
    assert_eq!(result.coeff(1), qrat(1));
    assert_eq!(result.num_nonzero(), 1);
}

#[test]
fn add_truncation_min() {
    let q = q_var();
    let a = FormalPowerSeries::one(q, 10);
    let b = FormalPowerSeries::one(q, 5);
    let result = arithmetic::add(&a, &b);
    assert_eq!(result.truncation_order(), 5);
}

// ===========================================================================
// 3. Subtraction/negation tests
// ===========================================================================

#[test]
fn negate_flips_signs() {
    let q = q_var();
    // 1 - q
    let mut coeffs = BTreeMap::new();
    coeffs.insert(0, qrat(1));
    coeffs.insert(1, qrat(-1));
    let f = FormalPowerSeries::from_coeffs(q, coeffs, 10);

    let neg = arithmetic::negate(&f);
    assert_eq!(neg.coeff(0), qrat(-1));
    assert_eq!(neg.coeff(1), qrat(1));
}

#[test]
fn sub_equals_add_negate() {
    let q = q_var();
    let a = FormalPowerSeries::monomial(q, qrat(3), 2, 10);
    let b = FormalPowerSeries::monomial(q, qrat(5), 1, 10);
    let sub_result = arithmetic::sub(&a, &b);
    let add_neg_result = arithmetic::add(&a, &arithmetic::negate(&b));
    assert_eq!(sub_result, add_neg_result);
}

#[test]
fn sub_self_is_zero() {
    let q = q_var();
    let mut coeffs = BTreeMap::new();
    coeffs.insert(0, qrat(1));
    coeffs.insert(1, qrat(-1));
    coeffs.insert(3, qrat(7));
    let f = FormalPowerSeries::from_coeffs(q, coeffs, 10);

    let result = arithmetic::sub(&f, &f);
    assert!(result.is_zero());
}

// ===========================================================================
// 4. Scalar multiplication tests
// ===========================================================================

#[test]
fn scalar_mul_by_zero() {
    let q = q_var();
    let f = FormalPowerSeries::one(q, 10);
    let result = arithmetic::scalar_mul(&QRat::zero(), &f);
    assert!(result.is_zero());
}

#[test]
fn scalar_mul_by_one() {
    let q = q_var();
    let f = FormalPowerSeries::monomial(q, qrat(3), 2, 10);
    let result = arithmetic::scalar_mul(&QRat::one(), &f);
    assert_eq!(result, f);
}

#[test]
fn scalar_mul_distributes() {
    let q = q_var();
    let s = qrat(3);
    let a = FormalPowerSeries::monomial(q, qrat(2), 0, 10);
    let b = FormalPowerSeries::monomial(q, qrat(5), 1, 10);

    // s*(a+b)
    let sum = arithmetic::add(&a, &b);
    let lhs = arithmetic::scalar_mul(&s, &sum);

    // s*a + s*b
    let sa = arithmetic::scalar_mul(&s, &a);
    let sb = arithmetic::scalar_mul(&s, &b);
    let rhs = arithmetic::add(&sa, &sb);

    assert_eq!(lhs, rhs);
}

// ===========================================================================
// 5. Multiplication tests
// ===========================================================================

#[test]
fn mul_by_one_identity() {
    let q = q_var();
    let f = FormalPowerSeries::monomial(q, qrat(3), 2, 10);
    let one = FormalPowerSeries::one(q, 10);
    let result = arithmetic::mul(&f, &one);
    assert_eq!(result, f);
}

#[test]
fn mul_by_zero_annihilator() {
    let q = q_var();
    let f = FormalPowerSeries::monomial(q, qrat(3), 2, 10);
    let z = FormalPowerSeries::zero(q, 10);
    let result = arithmetic::mul(&f, &z);
    assert!(result.is_zero());
}

#[test]
fn mul_1_minus_q_times_1_plus_q() {
    let q = q_var();
    // (1 - q)
    let mut ca = BTreeMap::new();
    ca.insert(0, qrat(1));
    ca.insert(1, qrat(-1));
    let a = FormalPowerSeries::from_coeffs(q, ca, 10);

    // (1 + q)
    let mut cb = BTreeMap::new();
    cb.insert(0, qrat(1));
    cb.insert(1, qrat(1));
    let b = FormalPowerSeries::from_coeffs(q, cb, 10);

    // (1-q)(1+q) = 1 - q^2
    let result = arithmetic::mul(&a, &b);
    assert_eq!(result.coeff(0), qrat(1));
    assert_eq!(result.coeff(1), QRat::zero());
    assert_eq!(result.coeff(2), qrat(-1));
    assert_eq!(result.coeff(3), QRat::zero());
}

#[test]
fn mul_1_minus_q_squared() {
    let q = q_var();
    // (1 - q)
    let mut ca = BTreeMap::new();
    ca.insert(0, qrat(1));
    ca.insert(1, qrat(-1));
    let a = FormalPowerSeries::from_coeffs(q, ca, 10);

    // (1-q)^2 = 1 - 2q + q^2
    let result = arithmetic::mul(&a, &a);
    assert_eq!(result.coeff(0), qrat(1));
    assert_eq!(result.coeff(1), qrat(-2));
    assert_eq!(result.coeff(2), qrat(1));
    assert_eq!(result.coeff(3), QRat::zero());
}

#[test]
fn mul_truncation_enforced() {
    let q = q_var();
    // Create series with terms at q^3 and q^4
    let a = FormalPowerSeries::monomial(q, qrat(1), 3, 5);
    let b = FormalPowerSeries::monomial(q, qrat(1), 4, 5);
    // 3 + 4 = 7 >= 5, so product should have no terms
    let result = arithmetic::mul(&a, &b);
    assert!(result.is_zero());
    assert_eq!(result.truncation_order(), 5);
}

// ===========================================================================
// 6. Inversion tests
// ===========================================================================

#[test]
fn invert_one_is_one() {
    let q = q_var();
    let one = FormalPowerSeries::one(q, 10);
    let result = arithmetic::invert(&one);
    assert_eq!(result.coeff(0), QRat::one());
    for k in 1..10 {
        assert_eq!(result.coeff(k), QRat::zero(), "coeff({}) should be 0", k);
    }
}

#[test]
fn invert_1_minus_q() {
    let q = q_var();
    // 1/(1-q) = 1 + q + q^2 + q^3 + ... + O(q^N)
    let mut ca = BTreeMap::new();
    ca.insert(0, qrat(1));
    ca.insert(1, qrat(-1));
    let a = FormalPowerSeries::from_coeffs(q, ca, 20);

    let result = arithmetic::invert(&a);
    for k in 0..20 {
        assert_eq!(result.coeff(k), QRat::one(), "1/(1-q) coeff({}) should be 1", k);
    }
}

#[test]
#[should_panic(expected = "Cannot invert series with zero constant term")]
fn invert_panics_on_zero_constant_term() {
    let q = q_var();
    let f = FormalPowerSeries::monomial(q, qrat(1), 1, 10); // q + O(q^10), coeff(0) = 0
    arithmetic::invert(&f);
}

// ===========================================================================
// 7. Shift tests
// ===========================================================================

#[test]
fn shift_by_zero_identity() {
    let q = q_var();
    let f = FormalPowerSeries::one(q, 10);
    let result = arithmetic::shift(&f, 0);
    assert_eq!(result.coeff(0), QRat::one());
    assert_eq!(result.truncation_order(), 10);
}

#[test]
fn shift_monomial() {
    let q = q_var();
    // shift(1 + O(q^5), 3) = q^3 + O(q^8)
    let one = FormalPowerSeries::one(q, 5);
    let result = arithmetic::shift(&one, 3);
    assert_eq!(result.truncation_order(), 8);
    assert_eq!(result.coeff(3), QRat::one());
    assert_eq!(result.coeff(0), QRat::zero());
    assert_eq!(result.coeff(4), QRat::zero());
}

// ===========================================================================
// 8. Display tests
// ===========================================================================

#[test]
fn display_zero_series() {
    let q = q_var();
    let z = FormalPowerSeries::zero(q, 10);
    assert_eq!(format!("{}", z), "O(q^10)");
}

#[test]
fn display_constant() {
    let q = q_var();
    let one = FormalPowerSeries::one(q, 10);
    assert_eq!(format!("{}", one), "1 + O(q^10)");
}

#[test]
fn display_polynomial() {
    let q = q_var();
    // 1 - q + 2*q^3
    let mut coeffs = BTreeMap::new();
    coeffs.insert(0, qrat(1));
    coeffs.insert(1, qrat(-1));
    coeffs.insert(3, qrat(2));
    let fps = FormalPowerSeries::from_coeffs(q, coeffs, 10);
    assert_eq!(format!("{}", fps), "1 - q + 2*q^3 + O(q^10)");
}

#[test]
fn display_negative_leading_coefficient() {
    let q = q_var();
    // -q + 3*q^2
    let mut coeffs = BTreeMap::new();
    coeffs.insert(1, qrat(-1));
    coeffs.insert(2, qrat(3));
    let fps = FormalPowerSeries::from_coeffs(q, coeffs, 10);
    assert_eq!(format!("{}", fps), "-q + 3*q^2 + O(q^10)");
}
