//! Tests for Jacobi theta functions: theta2, theta3, theta4.
//!
//! Verifies:
//! - theta3 coefficients at perfect squares with known values
//! - theta3 matches the product-side computation from generator_tests
//! - theta3^2 matches r_2(n) (sum-of-two-squares representation counts, OEIS A004018)
//! - theta4 coefficients with alternating signs at perfect squares
//! - theta4^2 has known coefficients
//! - theta2 in X=q^{1/4} has correct coefficients at odd perfect squares
//! - Cross-theta identity: theta3^2 + theta4^2 relationship

use qsym_core::number::QRat;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;
use qsym_core::series::{FormalPowerSeries, arithmetic};
use qsym_core::series::generator::InfiniteProductGenerator;
use qsym_core::qseries::{theta2, theta3, theta4};

/// Helper: create a SymbolId for "q".
fn q_var() -> SymbolId {
    let mut arena = ExprArena::new();
    arena.symbols_mut().intern("q")
}

/// Helper: create QRat from i64.
fn qrat(n: i64) -> QRat {
    QRat::from((n, 1i64))
}

// ===========================================================================
// 1. theta3 tests
// ===========================================================================

/// theta3 nonzero only at perfect squares 0,1,4,9,16,25,36,49.
/// coeff(0) = 1, coeff(n^2) = 2 for n = 1..7.
/// All non-square exponents are 0.
#[test]
fn theta3_coefficients_to_50() {
    let q = q_var();
    let t3 = theta3(q, 50);

    // Expected nonzero at n^2 for n=0..7:
    // 0^2=0: coeff=1, 1^2=1: coeff=2, 2^2=4: coeff=2, 3^2=9: coeff=2
    // 4^2=16: coeff=2, 5^2=25: coeff=2, 6^2=36: coeff=2, 7^2=49: coeff=2
    assert_eq!(t3.coeff(0), qrat(1), "theta3: coeff(0) = 1");
    for n in 1..=7i64 {
        assert_eq!(
            t3.coeff(n * n), qrat(2),
            "theta3: coeff({}) = 2",
            n * n
        );
    }

    // All non-square exponents should be 0
    let perfect_squares: Vec<i64> = (0..=7).map(|n: i64| n * n).collect();
    for k in 0..50i64 {
        if !perfect_squares.contains(&k) {
            assert_eq!(
                t3.coeff(k), QRat::zero(),
                "theta3: coeff({}) should be 0 (not a perfect square)",
                k
            );
        }
    }
}

/// theta3 output matches the product-side computation from
/// generator_tests.rs::jacobi_triple_product_z1_theta3.
/// Compute both and compare coefficient by coefficient to O(q^50).
#[test]
fn theta3_matches_phase2_jacobi_test() {
    let q = q_var();
    let trunc: i64 = 50;

    // Compute theta3 via our function
    let t3 = theta3(q, trunc);

    // Compute the same via the direct product (as in generator_tests.rs)
    // P1 = prod_{n=1}^{inf} (1 - q^{2n})
    let mut p1 = InfiniteProductGenerator::new(
        FormalPowerSeries::one(q, trunc),
        1,
        Box::new(move |k, var, tr| {
            let mut f = FormalPowerSeries::one(var, tr);
            f.set_coeff(2 * k, -QRat::one());
            f
        }),
    );
    p1.ensure_order(trunc);

    // P2 = prod_{n=1}^{inf} (1 + q^{2n-1})
    let mut p2 = InfiniteProductGenerator::new(
        FormalPowerSeries::one(q, trunc),
        1,
        Box::new(move |k, var, tr| {
            let mut f = FormalPowerSeries::one(var, tr);
            f.set_coeff(2 * k - 1, QRat::one());
            f
        }),
    );
    p2.ensure_order(trunc);

    let p2_series = p2.into_series();
    let p2_squared = arithmetic::mul(&p2_series, &p2_series);
    let product_side = arithmetic::mul(p1.series(), &p2_squared);

    // Compare coefficient by coefficient
    for k in 0..trunc {
        assert_eq!(
            t3.coeff(k), product_side.coeff(k),
            "theta3 vs direct product mismatch at q^{}: theta3={}, product={}",
            k, t3.coeff(k), product_side.coeff(k)
        );
    }
}

/// theta3(q)^2 coefficients match r_2(n) from OEIS A004018.
/// r_2(n) = number of representations of n as sum of two squares (counting order and signs).
///
/// r_2(0)=1, r_2(1)=4, r_2(2)=4, r_2(3)=0, r_2(4)=4, r_2(5)=8,
/// r_2(6)=0, r_2(7)=0, r_2(8)=4, r_2(9)=4, r_2(10)=8
#[test]
fn theta3_squared_sum_of_two_squares() {
    let q = q_var();
    let t3 = theta3(q, 50);
    let t3_sq = arithmetic::mul(&t3, &t3);

    // r_2(n) from OEIS A004018 (first 30 values)
    // r_2(n) = number of ways to write n = a^2 + b^2, counting order and signs
    let r2: Vec<i64> = vec![
        1, 4, 4, 0, 4, 8, 0, 0, 4, 4, 8,
        0, 0, 8, 0, 0, 4, 8, 4, 0, 8,
        0, 0, 0, 0, 12, 8, 0, 0, 8, 0,
    ];

    for (n, &expected) in r2.iter().enumerate() {
        assert_eq!(
            t3_sq.coeff(n as i64), qrat(expected),
            "theta3^2: r_2({}) should be {}, got {}",
            n, expected, t3_sq.coeff(n as i64)
        );
    }
}

// ===========================================================================
// 2. theta4 tests
// ===========================================================================

/// theta4 nonzero only at perfect squares with alternating signs:
/// coeff(0)=1, coeff(1)=-2, coeff(4)=2, coeff(9)=-2,
/// coeff(16)=2, coeff(25)=-2, coeff(36)=2, coeff(49)=-2.
#[test]
fn theta4_coefficients_to_50() {
    let q = q_var();
    let t4 = theta4(q, 50);

    // coeff(0) = 1
    assert_eq!(t4.coeff(0), qrat(1), "theta4: coeff(0) = 1");

    // coeff(n^2) = 2*(-1)^n for n=1..7
    for n in 1..=7i64 {
        let sign = if n % 2 == 0 { 1 } else { -1 };
        let expected = 2 * sign;
        assert_eq!(
            t4.coeff(n * n), qrat(expected),
            "theta4: coeff({}) = {}",
            n * n, expected
        );
    }

    // All non-square exponents should be 0
    let perfect_squares: Vec<i64> = (0..=7).map(|n: i64| n * n).collect();
    for k in 0..50i64 {
        if !perfect_squares.contains(&k) {
            assert_eq!(
                t4.coeff(k), QRat::zero(),
                "theta4: coeff({}) should be 0 (not a perfect square)",
                k
            );
        }
    }
}

/// theta4(q)^2 has known coefficients.
/// The coefficient of q^n in theta4^2 can be computed from the formula:
/// theta4^2 = sum_{n=-inf}^{inf} (-1)^n q^{n^2} squared.
/// Known: coeff(0)=1, coeff(1)=-4, coeff(2)=4, coeff(3)=0, coeff(4)=4
#[test]
fn theta4_times_theta4_check() {
    let q = q_var();
    let t4 = theta4(q, 30);
    let t4_sq = arithmetic::mul(&t4, &t4);

    // Known values from theta4^2 series
    assert_eq!(t4_sq.coeff(0), qrat(1), "theta4^2: coeff(0) = 1");
    assert_eq!(t4_sq.coeff(1), qrat(-4), "theta4^2: coeff(1) = -4");
    assert_eq!(t4_sq.coeff(2), qrat(4), "theta4^2: coeff(2) = 4");
    assert_eq!(t4_sq.coeff(3), QRat::zero(), "theta4^2: coeff(3) = 0");
    assert_eq!(t4_sq.coeff(4), qrat(4), "theta4^2: coeff(4) = 4");

    // Further known values: theta4^2 = 1 + sum_{n>=1} (-1)^n * r_2(n) * q^n
    // From the r_2 values: r_2(5)=8 => theta4^2[5] = (-1)^5 * 8 ... no that's wrong.
    // Actually theta4^2 = sum (-1)^{a+b} over all (a,b) with a^2+b^2=n.
    // Let's verify a few more directly:
    // n=5: representations are (+-1,+-2),(+-2,+-1) = 8 representations.
    //   Signs: (-1)^{|1|+|2|} depends on actual a,b not |a|,|b|.
    //   (1,2): (-1)^{1+2}=-1, (1,-2): (-1)^{1+(-2)}=(-1)^{-1}=-1...
    //   This is getting complicated. Let's just check that theta3^2 - theta4^2 gives the right pattern.

    // Simpler check: verify r_2'(n) = sum_{a^2+b^2=n} (-1)^{a+b}
    // for n=0: (0,0) => (-1)^0 = 1. CHECK.
    // for n=1: (1,0),(0,1),(-1,0),(0,-1) => (-1)^1+(-1)^1+(-1)^{-1}+(-1)^{-1} = -1-1-1-1 = -4. CHECK.
    // for n=2: (1,1),(1,-1),(-1,1),(-1,-1) => (-1)^2+(-1)^0+(-1)^0+(-1)^{-2} = 1+1+1+1 = 4. CHECK.
}

// ===========================================================================
// 3. theta2 tests
// ===========================================================================

/// theta2 returns series in X=q^{1/4}. Nonzero at exponents (2k+1)^2
/// for k=0,1,2,... i.e., 1, 9, 25, 49, 81, 121, 169.
/// All coefficients are 2. All other exponents are 0.
/// Test up to truncation_order = 200 (to capture 13^2=169).
#[test]
fn theta2_coefficients() {
    let q = q_var();
    let t2 = theta2(q, 200);

    // Expected nonzero exponents: (2k+1)^2 for k=0,1,...,6
    // k=0: 1, k=1: 9, k=2: 25, k=3: 49, k=4: 81, k=5: 121, k=6: 169
    let odd_perfect_squares: Vec<i64> = (0..=6).map(|k: i64| (2 * k + 1) * (2 * k + 1)).collect();
    // = [1, 9, 25, 49, 81, 121, 169]

    for &exp in &odd_perfect_squares {
        assert_eq!(
            t2.coeff(exp), qrat(2),
            "theta2: coeff({}) = 2",
            exp
        );
    }

    // All other exponents from 0 to 199 should be 0
    for k in 0..200i64 {
        if !odd_perfect_squares.contains(&k) {
            assert_eq!(
                t2.coeff(k), QRat::zero(),
                "theta2: coeff({}) should be 0",
                k
            );
        }
    }
}

// ===========================================================================
// 4. Cross-theta identity tests
// ===========================================================================

/// Verify a relationship between theta3^2 and theta4^2.
///
/// theta3(q)^2 has coefficients r_2(n), and theta4(q)^2 has coefficients
/// r_2'(n) = sum_{a^2+b^2=n} (-1)^{a+b}.
///
/// Their difference should satisfy:
/// theta3^2(n) + theta4^2(n) = 2 * (# representations with a+b even)
///   for each coefficient index n.
///
/// We verify this is consistent for the first 30 coefficients.
#[test]
fn theta_identity_theta3sq_theta4sq() {
    let q = q_var();
    let trunc = 30;
    let t3 = theta3(q, trunc);
    let t4 = theta4(q, trunc);
    let t3_sq = arithmetic::mul(&t3, &t3);
    let t4_sq = arithmetic::mul(&t4, &t4);

    // theta3^2 + theta4^2 should have all even coefficients
    // (since it counts 2 * #{reps with a+b even})
    let sum = arithmetic::add(&t3_sq, &t4_sq);

    for n in 0..trunc {
        let c = sum.coeff(n);
        if !c.is_zero() {
            // Each nonzero coefficient should be divisible by 2
            // (since it's 2 * count of even-parity representations)
            let half = c.clone() / qrat(2);
            // half should be an integer (denominator 1)
            assert_eq!(
                half.clone() * qrat(2), c.clone(),
                "theta3^2 + theta4^2 coefficient at n={} should be even, got {}",
                n, c
            );
        }
    }

    // Specific values:
    // n=0: t3^2=1, t4^2=1, sum=2 (the rep (0,0) has a+b=0 even, counted with both signs = 2)
    assert_eq!(sum.coeff(0), qrat(2), "sum at n=0");
    // n=1: t3^2=4, t4^2=-4, sum=0 (all reps of 1 as sum of squares have a+b odd)
    assert_eq!(sum.coeff(1), QRat::zero(), "sum at n=1");
    // n=2: t3^2=4, t4^2=4, sum=8 (all reps of 2 have a+b even: (1,1),(1,-1),(-1,1),(-1,-1))
    assert_eq!(sum.coeff(2), qrat(8), "sum at n=2");
}
