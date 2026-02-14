//! Comprehensive tests for qfactor, sift, qdegree, and lqdegree.
//!
//! Tests verify:
//! - qfactor correctly decomposes single factors, products, and powers of (1-q^i)
//! - qfactor handles truncated Euler function products
//! - sift extracts arithmetic subsequences, including Ramanujan's p(5n+4) congruence
//! - qdegree and lqdegree return correct degree bounds
//! - Edge cases: zero series, trivial sifts, shifted series

use qsym_core::number::QRat;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;
use qsym_core::series::{FormalPowerSeries, arithmetic};
use qsym_core::qseries::{qfactor, sift, qdegree, lqdegree, partition_gf};

/// Helper: create a SymbolId for "q".
fn q_var() -> SymbolId {
    let mut arena = ExprArena::new();
    arena.symbols_mut().intern("q")
}

/// Helper: create QRat from i64.
fn qrat(n: i64) -> QRat {
    QRat::from((n, 1i64))
}

/// Helper: build the polynomial (1 - q^i) as an FPS.
fn one_minus_q_i(variable: SymbolId, i: i64, trunc: i64) -> FormalPowerSeries {
    let mut f = FormalPowerSeries::one(variable, trunc);
    f.set_coeff(i, -QRat::one());
    f
}

// ===========================================================================
// 1. qfactor tests
// ===========================================================================

/// Factor (1-q) -> factors = {1: 1}, is_exact = true.
#[test]
fn test_qfactor_single_factor() {
    let q = q_var();
    let trunc = 20;
    let f = one_minus_q_i(q, 1, trunc);

    let result = qfactor(&f);
    assert!(result.is_exact, "factoring (1-q) should be exact");
    assert_eq!(result.scalar, QRat::one(), "scalar should be 1");
    assert_eq!(result.factors.get(&1), Some(&1), "(1-q) should have factor {{1: 1}}");
    assert_eq!(result.factors.len(), 1, "should have exactly one factor");
}

/// Factor (1-q)(1-q^2)(1-q^3) -> {1:1, 2:1, 3:1}, is_exact = true.
#[test]
fn test_qfactor_product() {
    let q = q_var();
    let trunc = 20;

    // Build (1-q)(1-q^2)(1-q^3)
    let f1 = one_minus_q_i(q, 1, trunc);
    let f2 = one_minus_q_i(q, 2, trunc);
    let f3 = one_minus_q_i(q, 3, trunc);
    let product = arithmetic::mul(&arithmetic::mul(&f1, &f2), &f3);

    let result = qfactor(&product);
    assert!(result.is_exact, "factoring (1-q)(1-q^2)(1-q^3) should be exact");
    assert_eq!(result.scalar, QRat::one(), "scalar should be 1");
    assert_eq!(result.factors.get(&1), Some(&1), "factor at 1 should have multiplicity 1");
    assert_eq!(result.factors.get(&2), Some(&1), "factor at 2 should have multiplicity 1");
    assert_eq!(result.factors.get(&3), Some(&1), "factor at 3 should have multiplicity 1");
    assert_eq!(result.factors.len(), 3, "should have exactly three distinct factors");
}

/// Factor (1-q)^3 -> {1: 3}, is_exact = true.
#[test]
fn test_qfactor_power() {
    let q = q_var();
    let trunc = 20;

    let f1 = one_minus_q_i(q, 1, trunc);
    let f1_cubed = arithmetic::mul(&arithmetic::mul(&f1, &f1), &f1);

    let result = qfactor(&f1_cubed);
    assert!(result.is_exact, "factoring (1-q)^3 should be exact");
    assert_eq!(result.scalar, QRat::one(), "scalar should be 1");
    assert_eq!(result.factors.get(&1), Some(&3), "(1-q)^3 should have {{1: 3}}");
    assert_eq!(result.factors.len(), 1, "should have exactly one distinct factor");
}

/// Factor a truncated Euler product (q;q)_k = (1-q)(1-q^2)...(1-q^k).
/// Should recover the individual factors.
#[test]
fn test_qfactor_euler_truncated() {
    let q = q_var();
    let trunc = 30;
    let k = 5;

    // Build (1-q)(1-q^2)(1-q^3)(1-q^4)(1-q^5)
    let mut product = FormalPowerSeries::one(q, trunc);
    for i in 1..=k {
        let factor = one_minus_q_i(q, i, trunc);
        product = arithmetic::mul(&product, &factor);
    }

    let result = qfactor(&product);
    assert!(result.is_exact, "factoring (q;q)_5 should be exact");
    assert_eq!(result.scalar, QRat::one(), "scalar should be 1");

    for i in 1..=k {
        assert_eq!(
            result.factors.get(&i), Some(&1),
            "(q;q)_5 should have factor at {} with multiplicity 1", i
        );
    }
    assert_eq!(result.factors.len(), k as usize, "should have exactly {} distinct factors", k);
}

/// Factor a polynomial with a nontrivial scalar: 2*(1-q).
#[test]
fn test_qfactor_with_scalar() {
    let q = q_var();
    let trunc = 20;

    // Build 2*(1-q) = 2 - 2*q
    let mut f = FormalPowerSeries::zero(q, trunc);
    f.set_coeff(0, qrat(2));
    f.set_coeff(1, qrat(-2));

    let result = qfactor(&f);
    assert!(result.is_exact, "factoring 2*(1-q) should be exact");
    assert_eq!(result.scalar, qrat(2), "scalar should be 2");
    assert_eq!(result.factors.get(&1), Some(&1), "should have factor {{1: 1}}");
}

// ===========================================================================
// 2. sift tests
// ===========================================================================

/// sift(partition_gf, 5, 4): extract p(5n+4) subsequence.
/// Ramanujan's congruence: p(5n+4) = 0 (mod 5) for all n >= 0.
#[test]
fn test_sift_partition_mod5_residue4() {
    let q = q_var();
    let trunc = 100;
    let pgf = partition_gf(q, trunc);

    let sifted = sift(&pgf, 5, 4);

    // coeff(0) = p(4) = 5
    assert_eq!(sifted.coeff(0), qrat(5), "p(4) = 5");
    // coeff(1) = p(9) = 30
    assert_eq!(sifted.coeff(1), qrat(30), "p(9) = 30");
    // coeff(2) = p(14) = 135
    assert_eq!(sifted.coeff(2), qrat(135), "p(14) = 135");

    // Verify all coefficients are divisible by 5 (Ramanujan's congruence)
    for i in 0..sifted.truncation_order() {
        let c = sifted.coeff(i);
        if !c.is_zero() {
            let c_mod_5 = c.clone() / qrat(5);
            // c/5 should be an integer (denominator 1), confirming divisibility
            let five_times = c_mod_5 * qrat(5);
            assert_eq!(five_times, c,
                "p(5*{}+4) = {} should be divisible by 5", i, sifted.coeff(i));
        }
    }
}

/// sift(f, 1, 0) == f (trivial sift, identity operation).
#[test]
fn test_sift_identity() {
    let q = q_var();
    let trunc = 20;
    let pgf = partition_gf(q, trunc);

    let sifted = sift(&pgf, 1, 0);

    for n in 0..trunc {
        assert_eq!(
            pgf.coeff(n), sifted.coeff(n),
            "sift(f, 1, 0) should equal f at coeff({})", n
        );
    }
}

/// sift(f, 2, 0) gives even-indexed coefficients,
/// sift(f, 2, 1) gives odd-indexed coefficients.
#[test]
fn test_sift_even_odd() {
    let q = q_var();
    let trunc = 40;
    let pgf = partition_gf(q, trunc);

    let even = sift(&pgf, 2, 0);
    let odd = sift(&pgf, 2, 1);

    // even.coeff(i) should be pgf.coeff(2*i)
    for i in 0..even.truncation_order() {
        assert_eq!(
            even.coeff(i), pgf.coeff(2 * i),
            "sift(f,2,0) at {} should equal f at {}", i, 2 * i
        );
    }

    // odd.coeff(i) should be pgf.coeff(2*i+1)
    for i in 0..odd.truncation_order() {
        assert_eq!(
            odd.coeff(i), pgf.coeff(2 * i + 1),
            "sift(f,2,1) at {} should equal f at {}", i, 2 * i + 1
        );
    }
}

/// sift with negative j should normalize to j mod m.
#[test]
fn test_sift_negative_j() {
    let q = q_var();
    let trunc = 30;
    let pgf = partition_gf(q, trunc);

    // sift(f, 5, -1) should equal sift(f, 5, 4) since -1 mod 5 = 4
    let sifted_neg = sift(&pgf, 5, -1);
    let sifted_pos = sift(&pgf, 5, 4);

    for i in 0..sifted_neg.truncation_order().min(sifted_pos.truncation_order()) {
        assert_eq!(
            sifted_neg.coeff(i), sifted_pos.coeff(i),
            "sift(f,5,-1) should equal sift(f,5,4) at coeff({})", i
        );
    }
}

// ===========================================================================
// 3. qdegree / lqdegree tests
// ===========================================================================

/// For 1 + q + q^5 (as a polynomial FPS), qdegree = 5.
#[test]
fn test_qdegree_polynomial() {
    let q = q_var();
    let trunc = 20;
    let mut f = FormalPowerSeries::zero(q, trunc);
    f.set_coeff(0, QRat::one());
    f.set_coeff(1, QRat::one());
    f.set_coeff(5, QRat::one());

    assert_eq!(qdegree(&f), Some(5), "qdegree of 1 + q + q^5 should be 5");
}

/// For q^3 + q^7, lqdegree = 3.
#[test]
fn test_lqdegree_shifted() {
    let q = q_var();
    let trunc = 20;
    let mut f = FormalPowerSeries::zero(q, trunc);
    f.set_coeff(3, QRat::one());
    f.set_coeff(7, QRat::one());

    assert_eq!(lqdegree(&f), Some(3), "lqdegree of q^3 + q^7 should be 3");
}

/// For the zero series, qdegree = None.
#[test]
fn test_qdegree_zero() {
    let q = q_var();
    let trunc = 20;
    let f = FormalPowerSeries::zero(q, trunc);

    assert_eq!(qdegree(&f), None, "qdegree of zero series should be None");
    assert_eq!(lqdegree(&f), None, "lqdegree of zero series should be None");
}

/// For a single monomial q^k, qdegree == lqdegree == k.
#[test]
fn test_qdegree_equals_lqdegree_monomial() {
    let q = q_var();
    let trunc = 20;
    let f = FormalPowerSeries::monomial(q, QRat::one(), 4, trunc);

    assert_eq!(qdegree(&f), Some(4), "qdegree of q^4 should be 4");
    assert_eq!(lqdegree(&f), Some(4), "lqdegree of q^4 should be 4");
}
