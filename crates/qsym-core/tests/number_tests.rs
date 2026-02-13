//! Comprehensive edge-case tests for QInt and QRat arithmetic.
//!
//! These tests verify arbitrary precision arithmetic correctness per
//! Plan 01-02: BigInt/BigRat Arithmetic Edge Cases.
//!
//! Covers: zero identity, overflow from machine word, sign handling,
//! zero annihilation, power edge cases, auto-reduction, hash invariants,
//! Display formatting, and large-value precision.

use qsym_core::{QInt, QRat};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Helper to compute the hash of a value.
fn hash_of<T: Hash>(val: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}

// =============================================================================
// Integer Edge Cases
// =============================================================================

#[test]
fn int_zero_plus_zero() {
    // 0 + 0 = 0 (zero identity)
    let a = QInt::from(0i64);
    let b = QInt::from(0i64);
    let result = a + b;
    assert_eq!(result, QInt::from(0i64));
    assert!(result.is_zero());
}

#[test]
fn int_overflow_from_machine_word() {
    // i64::MAX + 1 should produce the correct big value (no overflow)
    let max = QInt::from(i64::MAX);
    let one = QInt::from(1i64);
    let result = max + one;
    // i64::MAX = 9223372036854775807, so +1 = 9223372036854775808
    let expected_str = "9223372036854775808";
    assert_eq!(result.to_string(), expected_str);
}

#[test]
fn int_neg_one_times_x() {
    // -1 * x = -x (sign handling)
    let neg_one = QInt::from(-1i64);
    let x = QInt::from(42i64);
    let result = neg_one * x;
    assert_eq!(result, QInt::from(-42i64));
}

#[test]
fn int_zero_times_anything() {
    // 0 * anything = 0 (zero annihilation)
    let zero = QInt::from(0i64);
    let big = QInt::from(999_999_999i64);
    let result = zero * big;
    assert!(result.is_zero());
    assert_eq!(result, QInt::from(0i64));
}

#[test]
fn int_zero_to_the_zero() {
    // 0^0 = 1 (mathematical convention, GMP returns 1)
    let zero = QInt::from(0i64);
    let result = zero.pow_u32(0);
    assert_eq!(result, QInt::from(1i64));
}

#[test]
fn int_negative_base_odd_exponent() {
    // (-2)^3 = -8
    let base = QInt::from(-2i64);
    let result = base.pow_u32(3);
    assert_eq!(result, QInt::from(-8i64));
}

#[test]
fn int_negative_base_even_exponent() {
    // (-3)^2 = 9
    let base = QInt::from(-3i64);
    let result = base.pow_u32(2);
    assert_eq!(result, QInt::from(9i64));
}

#[test]
fn int_large_power_exceeds_u64() {
    // 2^64 = 18446744073709551616 (exceeds u64::MAX)
    let two = QInt::from(2i64);
    let result = two.pow_u32(64);
    assert_eq!(result.to_string(), "18446744073709551616");
}

#[test]
fn int_very_large_power() {
    // 2^128 should produce exact result
    let two = QInt::from(2i64);
    let result = two.pow_u32(128);
    assert_eq!(
        result.to_string(),
        "340282366920938463463374607431768211456"
    );
}

#[test]
fn int_negative_zero_equals_zero() {
    // -0 == 0 (GMP normalizes -0 to 0)
    let zero = QInt::from(0i64);
    let neg_zero = -QInt::from(0i64);
    assert_eq!(zero, neg_zero);
}

#[test]
fn int_one_to_any_power() {
    // 1^n = 1 for any n
    let one = QInt::from(1i64);
    assert_eq!(one.pow_u32(0), QInt::from(1i64));
    assert_eq!(one.pow_u32(1), QInt::from(1i64));
    assert_eq!(one.pow_u32(100), QInt::from(1i64));
    assert_eq!(one.pow_u32(1000), QInt::from(1i64));
}

#[test]
fn int_subtraction_to_negative() {
    // 3 - 7 = -4
    let a = QInt::from(3i64);
    let b = QInt::from(7i64);
    let result = a - b;
    assert_eq!(result, QInt::from(-4i64));
}

#[test]
fn int_double_negation() {
    // -(-x) = x
    let x = QInt::from(42i64);
    let result = -(-x);
    assert_eq!(result, QInt::from(42i64));
}

#[test]
fn int_large_multiplication() {
    // Large number multiplication should be exact
    let a = QInt::from(i64::MAX);
    let b = QInt::from(i64::MAX);
    let result = &a * &b;
    // i64::MAX^2 = 85070591730234615847396907784232501249
    assert_eq!(
        result.to_string(),
        "85070591730234615847396907784232501249"
    );
}

#[test]
fn int_div_exact() {
    // 10 / 2 = 5 (exact integer division)
    let a = QInt::from(10i64);
    let b = QInt::from(2i64);
    let result = a / b;
    assert_eq!(result, QInt::from(5i64));
}

#[test]
fn int_div_truncates() {
    // 7 / 3 = 2 (truncated integer division)
    let a = QInt::from(7i64);
    let b = QInt::from(3i64);
    let result = a / b;
    assert_eq!(result, QInt::from(2i64));
}

#[test]
#[should_panic]
fn int_div_by_zero_panics() {
    // Division by zero must panic (not silently produce wrong result)
    let a = QInt::from(42i64);
    let zero = QInt::from(0i64);
    let _result = a / zero;
}

// =============================================================================
// Rational Edge Cases
// =============================================================================

#[test]
fn rat_auto_reduction() {
    // (6, 4) should auto-reduce to 3/2
    let r = QRat::from((6i64, 4i64));
    assert_eq!(r.numer().to_string(), "3");
    assert_eq!(r.denom().to_string(), "2");
}

#[test]
fn rat_double_negative_normalizes() {
    // (-3, -5) should normalize to 3/5
    let r = QRat::from((-3i64, -5i64));
    assert_eq!(r.numer().to_string(), "3");
    assert_eq!(r.denom().to_string(), "5");
}

#[test]
fn rat_negative_numerator() {
    // (-3, 5) should keep negative in numerator: -3/5
    let r = QRat::from((-3i64, 5i64));
    assert_eq!(r.numer().to_string(), "-3");
    assert_eq!(r.denom().to_string(), "5");
}

#[test]
fn rat_negative_denominator_normalizes() {
    // (3, -5) should normalize to (-3, 5) per GMP convention
    let r = QRat::from((3i64, -5i64));
    assert_eq!(r.numer().to_string(), "-3");
    assert_eq!(r.denom().to_string(), "5");
}

#[test]
fn rat_zero_numerator() {
    // (0, n) = 0 for any nonzero n
    let r = QRat::from((0i64, 7i64));
    assert!(r.is_zero());
    assert_eq!(r.numer().to_string(), "0");
    assert_eq!(r.denom().to_string(), "1");
}

#[test]
#[should_panic]
fn rat_zero_denominator_panics() {
    // (n, 0) must panic (undefined)
    let _r = QRat::from((5i64, 0i64));
}

#[test]
fn rat_addition_common_denominator() {
    // 1/3 + 1/6 = 1/2
    let a = QRat::from((1i64, 3i64));
    let b = QRat::from((1i64, 6i64));
    let result = a + b;
    assert_eq!(result, QRat::from((1i64, 2i64)));
}

#[test]
fn rat_exact_cancellation() {
    // 1/2 - 1/2 = 0
    let a = QRat::from((1i64, 2i64));
    let b = QRat::from((1i64, 2i64));
    let result = a - b;
    assert!(result.is_zero());
    assert_eq!(result, QRat::zero());
}

#[test]
fn rat_multiplicative_inverse() {
    // (1/3) * (3/1) = 1
    let a = QRat::from((1i64, 3i64));
    let b = QRat::from((3i64, 1i64));
    let result = a * b;
    assert_eq!(result, QRat::one());
}

#[test]
fn rat_division() {
    // (1/2) / (1/3) = 3/2
    let a = QRat::from((1i64, 2i64));
    let b = QRat::from((1i64, 3i64));
    let result = a / b;
    assert_eq!(result, QRat::from((3i64, 2i64)));
}

#[test]
#[should_panic]
fn rat_division_by_zero_panics() {
    // Division by zero rational must panic
    let a = QRat::from((1i64, 2i64));
    let zero = QRat::zero();
    let _result = a / zero;
}

#[test]
fn rat_large_values_no_precision_loss() {
    // Large numerator and denominator arithmetic should not lose precision
    // Use 2^64 as numerator, 2^64 - 1 as denominator
    let big_num: rug::Integer = rug::Integer::from(1u64) << 64;
    let big_den: rug::Integer = rug::Integer::from(big_num.clone() - 1u32);
    let r = QRat::from(rug::Rational::from((big_num.clone(), big_den.clone())));
    // Should not auto-reduce (numerator and denominator are coprime since 2^64 is even, 2^64-1 is odd)
    assert_eq!(r.numer().to_string(), "18446744073709551616");
    assert_eq!(r.denom().to_string(), "18446744073709551615");
}

#[test]
fn rat_large_addition_exact() {
    // Adding two large rationals should produce exact result
    // 1/(2^64) + 1/(2^64) = 2/(2^64) = 1/(2^63)
    let big_den: rug::Integer = rug::Integer::from(1u64) << 64;
    let a = QRat::from(rug::Rational::from((rug::Integer::from(1), big_den.clone())));
    let b = QRat::from(rug::Rational::from((rug::Integer::from(1), big_den)));
    let result = a + b;
    let expected_den: rug::Integer = rug::Integer::from(1u64) << 63;
    let expected = QRat::from(rug::Rational::from((rug::Integer::from(1), expected_den)));
    assert_eq!(result, expected);
}

#[test]
fn rat_from_qint() {
    // QInt -> QRat should produce integer-valued rational
    let qi = QInt::from(42i64);
    let qr = QRat::from(qi);
    assert_eq!(qr, QRat::from((42i64, 1i64)));
    assert_eq!(qr.denom().to_string(), "1");
}

#[test]
fn rat_negation() {
    // -(1/3) = -1/3
    let r = QRat::from((1i64, 3i64));
    let neg_r = -r;
    assert_eq!(neg_r, QRat::from((-1i64, 3i64)));
}

// =============================================================================
// Hash Invariants
// =============================================================================

#[test]
fn hash_int_equal_values_same_hash() {
    // For all a, b: if a == b then hash(a) == hash(b)
    let a = QInt::from(42i64);
    let b = QInt::from(42i64);
    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn hash_int_zero_and_neg_zero_same() {
    // QInt(0) and QInt(-0) must hash the same
    let zero = QInt::from(0i64);
    let neg_zero = -QInt::from(0i64);
    assert_eq!(zero, neg_zero);
    assert_eq!(hash_of(&zero), hash_of(&neg_zero));
}

#[test]
fn hash_int_large_values() {
    // Hash invariant for large values
    let a = QInt::from(2i64).pow_u32(100);
    let b = QInt::from(2i64).pow_u32(100);
    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn hash_int_different_values_different_hash() {
    // Different values should (almost certainly) produce different hashes
    let a = QInt::from(1i64);
    let b = QInt::from(2i64);
    assert_ne!(a, b);
    // Not strictly required but overwhelmingly likely with a good hash
    assert_ne!(hash_of(&a), hash_of(&b));
}

#[test]
fn hash_rat_equal_values_same_hash() {
    let a = QRat::from((3i64, 4i64));
    let b = QRat::from((3i64, 4i64));
    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn hash_rat_reduced_same_hash() {
    // QRat(2,4) and QRat(1,2) must hash the same (auto-reduction means == holds)
    let a = QRat::from((2i64, 4i64));
    let b = QRat::from((1i64, 2i64));
    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn hash_rat_zero_variants_same() {
    // 0/1, 0/5, 0/1000 should all be equal and hash the same
    let a = QRat::from((0i64, 1i64));
    let b = QRat::from((0i64, 5i64));
    let c = QRat::from((0i64, 1000i64));
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(hash_of(&a), hash_of(&b));
    assert_eq!(hash_of(&b), hash_of(&c));
}

#[test]
fn hash_rat_double_negative_same() {
    // (-3,-5) and (3,5) should hash the same after normalization
    let a = QRat::from((-3i64, -5i64));
    let b = QRat::from((3i64, 5i64));
    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
}

// =============================================================================
// Display Formatting
// =============================================================================

#[test]
fn display_int_positive() {
    let n = QInt::from(42i64);
    assert_eq!(n.to_string(), "42");
}

#[test]
fn display_int_negative() {
    let n = QInt::from(-7i64);
    assert_eq!(n.to_string(), "-7");
}

#[test]
fn display_int_zero() {
    let n = QInt::from(0i64);
    assert_eq!(n.to_string(), "0");
}

#[test]
fn display_int_large() {
    let n = QInt::from(2i64).pow_u32(64);
    assert_eq!(n.to_string(), "18446744073709551616");
}

#[test]
fn display_rat_fraction() {
    // QRat should display as "num/den"
    let r = QRat::from((3i64, 4i64));
    assert_eq!(r.to_string(), "3/4");
}

#[test]
fn display_rat_negative() {
    let r = QRat::from((-1i64, 3i64));
    assert_eq!(r.to_string(), "-1/3");
}

#[test]
fn display_rat_integer_valued() {
    // An integer-valued rational (den == 1) should display appropriately
    // rug::Rational Display for integer-valued outputs just the number
    let r = QRat::from((5i64, 1i64));
    assert_eq!(r.to_string(), "5");
}

#[test]
fn display_rat_zero() {
    let r = QRat::zero();
    assert_eq!(r.to_string(), "0");
}

#[test]
fn display_rat_reduced() {
    // (6, 4) auto-reduces to 3/2, so display should show "3/2"
    let r = QRat::from((6i64, 4i64));
    assert_eq!(r.to_string(), "3/2");
}

// =============================================================================
// Reference-based Arithmetic (avoids moves)
// =============================================================================

#[test]
fn int_ref_add() {
    let a = QInt::from(10i64);
    let b = QInt::from(20i64);
    let result = &a + &b;
    assert_eq!(result, QInt::from(30i64));
    // a and b are still usable
    assert_eq!(a, QInt::from(10i64));
    assert_eq!(b, QInt::from(20i64));
}

#[test]
fn int_ref_sub() {
    let a = QInt::from(10i64);
    let b = QInt::from(3i64);
    let result = &a - &b;
    assert_eq!(result, QInt::from(7i64));
}

#[test]
fn int_ref_mul() {
    let a = QInt::from(6i64);
    let b = QInt::from(7i64);
    let result = &a * &b;
    assert_eq!(result, QInt::from(42i64));
}

#[test]
fn rat_ref_add() {
    let a = QRat::from((1i64, 4i64));
    let b = QRat::from((1i64, 4i64));
    let result = &a + &b;
    assert_eq!(result, QRat::from((1i64, 2i64)));
}

#[test]
fn rat_ref_sub() {
    let a = QRat::from((3i64, 4i64));
    let b = QRat::from((1i64, 4i64));
    let result = &a - &b;
    assert_eq!(result, QRat::from((1i64, 2i64)));
}

#[test]
fn rat_ref_mul() {
    let a = QRat::from((2i64, 3i64));
    let b = QRat::from((3i64, 4i64));
    let result = &a * &b;
    assert_eq!(result, QRat::from((1i64, 2i64)));
}
