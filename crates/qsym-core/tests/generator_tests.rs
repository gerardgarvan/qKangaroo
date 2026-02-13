//! Integration tests for InfiniteProductGenerator verifying mathematical
//! correctness against known q-series identities.
//!
//! Tests verify:
//! - Euler function (q;q)_inf via pentagonal number theorem (OEIS A010815)
//! - Partition function p(n) via series inversion (OEIS A000041)
//! - Jacobi triple product identity (z=1 case: theta_3)
//! - Generator incremental reuse
//! - End-to-end identity: (q;q)_inf * 1/(q;q)_inf = 1

use qsym_core::number::QRat;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;
use qsym_core::series::{FormalPowerSeries, arithmetic};
use qsym_core::series::generator::{
    InfiniteProductGenerator, euler_function_generator, qpochhammer_inf_generator,
};

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
// 1. Euler function tests (pentagonal number theorem) -- OEIS A010815
// ===========================================================================

/// Verify all 30 coefficients of (q;q)_inf match the pentagonal number theorem.
/// Nonzero only at generalized pentagonal numbers: 0, 1, 2, 5, 7, 12, 15, 22, 26.
#[test]
fn euler_function_coefficients_to_30() {
    let q = q_var();
    let mut ipg = euler_function_generator(q, 30);
    ipg.ensure_order(30);
    let fps = ipg.series();

    // Expected: nonzero at generalized pentagonal numbers k(3k-1)/2
    // with signs: +, -, -, +, +, -, -, +, +, ...
    let expected: Vec<(i64, i64)> = vec![
        (0, 1), (1, -1), (2, -1), (5, 1), (7, 1),
        (12, -1), (15, -1), (22, 1), (26, 1),
    ];

    for n in 0..30 {
        let expected_val = expected.iter()
            .find(|&&(exp, _)| exp == n)
            .map(|&(_, v)| qrat(v))
            .unwrap_or_else(QRat::zero);
        assert_eq!(
            fps.coeff(n), expected_val,
            "Euler function coefficient at q^{} should be {}, got {}",
            n, expected_val, fps.coeff(n)
        );
    }
}

/// Extend verification to O(q^60) to check additional pentagonal numbers.
#[test]
fn euler_function_to_60() {
    let q = q_var();
    let mut ipg = euler_function_generator(q, 60);
    ipg.ensure_order(60);
    let fps = ipg.series();

    // All generalized pentagonal numbers below 60:
    // k=0: 0, k=1: 1, k=-1: 2, k=2: 5, k=-2: 7,
    // k=3: 12, k=-3: 15, k=4: 22, k=-4: 26,
    // k=5: 35, k=-5: 40, k=6: 51, k=-6: 57
    let expected: Vec<(i64, i64)> = vec![
        (0, 1), (1, -1), (2, -1), (5, 1), (7, 1),
        (12, -1), (15, -1), (22, 1), (26, 1),
        (35, -1), (40, -1), (51, 1), (57, 1),
    ];

    // Spot-check the higher pentagonal numbers
    assert_eq!(fps.coeff(35), qrat(-1), "q^35 should be -1");
    assert_eq!(fps.coeff(40), qrat(-1), "q^40 should be -1");
    assert_eq!(fps.coeff(51), qrat(1), "q^51 should be +1");
    assert_eq!(fps.coeff(57), qrat(1), "q^57 should be +1");

    // Check that non-pentagonal exponents are zero in the higher range
    for n in 30..60 {
        let is_pentagonal = expected.iter().any(|&(exp, _)| exp == n);
        if !is_pentagonal {
            assert_eq!(
                fps.coeff(n), QRat::zero(),
                "Euler function coefficient at q^{} should be 0",
                n
            );
        }
    }
}

/// Verify the sparsity count: exactly 13 generalized pentagonal numbers below 60.
#[test]
fn euler_function_sparsity() {
    let q = q_var();
    let mut ipg = euler_function_generator(q, 60);
    ipg.ensure_order(60);
    let fps = ipg.series();

    // Pentagonal numbers below 60: 0,1,2,5,7,12,15,22,26,35,40,51,57 = 13 values
    assert_eq!(
        fps.num_nonzero(), 13,
        "Euler function to O(q^60) should have exactly 13 nonzero coefficients"
    );
}

// ===========================================================================
// 2. Partition function tests -- OEIS A000041
// ===========================================================================

/// Compute p(n) = [q^n] 1/(q;q)_inf and verify against known values.
#[test]
fn partition_function_via_inversion() {
    let q = q_var();
    let mut ipg = euler_function_generator(q, 25);
    ipg.ensure_order(25);
    let euler = ipg.into_series();

    // Invert to get 1/(q;q)_inf = sum p(n) q^n
    let partition = arithmetic::invert(&euler);

    let expected_p: Vec<i64> = vec![
        1, 1, 2, 3, 5, 7, 11, 15, 22, 30, 42,
        56, 77, 101, 135, 176, 231, 297, 385, 490, 627,
    ];

    for (n, &p_n) in expected_p.iter().enumerate() {
        assert_eq!(
            partition.coeff(n as i64), qrat(p_n),
            "p({}) should be {}, got {}",
            n, p_n, partition.coeff(n as i64)
        );
    }
}

/// Extended partition function test: all p(n) for n >= 0 are positive.
#[test]
fn partition_function_extended() {
    let q = q_var();
    let trunc = 50;
    let mut ipg = euler_function_generator(q, trunc);
    ipg.ensure_order(trunc);
    let euler = ipg.into_series();
    let partition = arithmetic::invert(&euler);

    // All coefficients from q^0 to q^49 should be positive (nonzero)
    for n in 0..trunc {
        let c = partition.coeff(n);
        assert!(
            !c.is_zero(),
            "p({}) should be positive, got zero",
            n
        );
        // p(n) is always a positive integer, so numerator > 0 and denominator = 1
        assert_eq!(
            c.0.cmp0(), std::cmp::Ordering::Greater,
            "p({}) should be positive, got {}",
            n, c
        );
    }

    // All partition values are nonzero, so num_nonzero should be exactly trunc
    assert_eq!(
        partition.num_nonzero(), trunc as usize,
        "1/(q;q)_inf to O(q^{}) should have {} nonzero coefficients",
        trunc, trunc
    );
}

// ===========================================================================
// 3. Jacobi triple product tests (z=1 case: theta_3)
// ===========================================================================

/// Verify the Jacobi triple product identity at z=1:
/// prod_{n=1}^{inf} (1-q^{2n})(1+q^{2n-1})^2 = sum_{n=-inf}^{inf} q^{n^2} = theta_3(q)
///
/// theta_3 = 1 + 2q + 2q^4 + 2q^9 + 2q^16 + 2q^25 + 2q^36 + 2q^49 + ...
#[test]
fn jacobi_triple_product_z1_theta3() {
    let q = q_var();
    let trunc: i64 = 50;

    // P1 = prod_{n=1}^{inf} (1 - q^{2n})
    let mut p1 = InfiniteProductGenerator::new(
        FormalPowerSeries::one(q, trunc),
        1,
        Box::new(move |k, var, tr| {
            let mut f = FormalPowerSeries::one(var, tr);
            f.set_coeff(2 * k, -QRat::one()); // (1 - q^{2k})
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
            f.set_coeff(2 * k - 1, QRat::one()); // (1 + q^{2k-1})
            f
        }),
    );
    p2.ensure_order(trunc);

    // Product side: P1 * P2 * P2 = P1 * P2^2
    let p2_series = p2.into_series();
    let p2_squared = arithmetic::mul(&p2_series, &p2_series);
    let product_side = arithmetic::mul(p1.series(), &p2_squared);

    // Sum side: theta_3(q) = sum_{n=-inf}^{inf} q^{n^2}
    // = 1 + 2*q + 2*q^4 + 2*q^9 + 2*q^16 + 2*q^25 + 2*q^36 + 2*q^49
    let mut sum_side = FormalPowerSeries::zero(q, trunc);
    sum_side.set_coeff(0, QRat::one()); // q^{0^2} = 1 (from n=0)
    let mut n: i64 = 1;
    while n * n < trunc {
        // n and -n both contribute q^{n^2}, so coefficient is 2
        let coeff_val = sum_side.coeff(n * n) + qrat(2);
        sum_side.set_coeff(n * n, coeff_val);
        n += 1;
    }

    // Verify all coefficients match
    for k in 0..trunc {
        assert_eq!(
            product_side.coeff(k), sum_side.coeff(k),
            "Jacobi triple product (z=1): mismatch at q^{}: product={}, sum={}",
            k, product_side.coeff(k), sum_side.coeff(k)
        );
    }
}

/// Simpler spot-check of theta_3 to O(q^20).
#[test]
fn jacobi_triple_product_z1_spot_check() {
    let q = q_var();
    let trunc: i64 = 20;

    // Build the product side: prod(1-q^{2n}) * prod(1+q^{2n-1})^2
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
    let result = arithmetic::mul(p1.series(), &p2_squared);

    // Spot checks
    assert_eq!(result.coeff(0), qrat(1), "theta_3: coeff(0) = 1");
    assert_eq!(result.coeff(1), qrat(2), "theta_3: coeff(1) = 2");
    assert_eq!(result.coeff(2), QRat::zero(), "theta_3: coeff(2) = 0");
    assert_eq!(result.coeff(3), QRat::zero(), "theta_3: coeff(3) = 0");
    assert_eq!(result.coeff(4), qrat(2), "theta_3: coeff(4) = 2");
    assert_eq!(result.coeff(9), qrat(2), "theta_3: coeff(9) = 2");
    assert_eq!(result.coeff(16), qrat(2), "theta_3: coeff(16) = 2");

    // All non-square exponents below 20 should be 0
    let perfect_squares: Vec<i64> = vec![0, 1, 4, 9, 16];
    for k in 0..trunc {
        if !perfect_squares.contains(&k) {
            assert_eq!(
                result.coeff(k), QRat::zero(),
                "theta_3: coeff({}) should be 0 (not a perfect square)",
                k
            );
        }
    }
}

// ===========================================================================
// 4. Generator reuse tests
// ===========================================================================

/// Test incremental order expansion: ensure_order(5) then ensure_order(10)
/// reuses the partial product from the first call.
#[test]
fn generator_incremental_order() {
    let q = q_var();
    let mut ipg = euler_function_generator(q, 10);

    // First, compute to O(q^5)
    ipg.ensure_order(5);
    assert_eq!(ipg.factors_included(), 5);

    // Check low-order coefficients
    assert_eq!(ipg.series().coeff(0), qrat(1));
    assert_eq!(ipg.series().coeff(1), qrat(-1));
    assert_eq!(ipg.series().coeff(2), qrat(-1));
    assert_eq!(ipg.series().coeff(3), QRat::zero());
    assert_eq!(ipg.series().coeff(4), QRat::zero());

    // Now extend to O(q^10)
    ipg.ensure_order(10);
    assert_eq!(ipg.factors_included(), 10);

    // Verify higher coefficients
    assert_eq!(ipg.series().coeff(5), qrat(1));
    assert_eq!(ipg.series().coeff(7), qrat(1));
    assert_eq!(ipg.series().coeff(6), QRat::zero());
    assert_eq!(ipg.series().coeff(8), QRat::zero());
    assert_eq!(ipg.series().coeff(9), QRat::zero());
}

/// After ensure_order(N), calling ensure_order(N) again is a no-op.
#[test]
fn generator_already_complete() {
    let q = q_var();
    let mut ipg = euler_function_generator(q, 20);

    ipg.ensure_order(15);
    let included_after_first = ipg.factors_included();
    assert_eq!(included_after_first, 15);

    // Calling again with same order should not change anything
    ipg.ensure_order(15);
    assert_eq!(ipg.factors_included(), included_after_first);

    // Calling with lower order should also be a no-op
    ipg.ensure_order(10);
    assert_eq!(ipg.factors_included(), included_after_first);
}

// ===========================================================================
// 5. Simple series identity tests (end-to-end)
// ===========================================================================

/// Sanity check: (1-q)(1+q) = 1 - q^2.
#[test]
fn identity_1_minus_q_times_1_plus_q_via_generator() {
    let q = q_var();
    let trunc = 10;

    let a = {
        let mut s = FormalPowerSeries::one(q, trunc);
        s.set_coeff(1, -QRat::one());
        s
    };
    let b = {
        let mut s = FormalPowerSeries::one(q, trunc);
        s.set_coeff(1, QRat::one());
        s
    };

    let result = arithmetic::mul(&a, &b);

    // Should be 1 - q^2
    assert_eq!(result.coeff(0), qrat(1));
    assert_eq!(result.coeff(1), QRat::zero());
    assert_eq!(result.coeff(2), qrat(-1));
    for k in 3..trunc {
        assert_eq!(result.coeff(k), QRat::zero());
    }
}

/// Identity: (q;q)_inf * 1/(q;q)_inf = 1 + O(q^N).
/// All non-constant coefficients should be exactly zero.
#[test]
fn identity_euler_squared_times_partition_squared() {
    let q = q_var();
    let trunc = 30;

    let mut ipg = euler_function_generator(q, trunc);
    ipg.ensure_order(trunc);
    let euler = ipg.into_series();

    let partition = arithmetic::invert(&euler);
    let product = arithmetic::mul(&euler, &partition);

    // Should be 1 + O(q^30): coeff(0)=1, all others=0
    assert_eq!(product.coeff(0), qrat(1), "constant term should be 1");
    for k in 1..trunc {
        assert_eq!(
            product.coeff(k), QRat::zero(),
            "(q;q)_inf * 1/(q;q)_inf coefficient at q^{} should be 0, got {}",
            k, product.coeff(k)
        );
    }
}

/// Verify that qpochhammer_inf_generator with a=1, offset=1 gives
/// the same Euler function as euler_function_generator.
#[test]
fn qpochhammer_general_generator() {
    let q = q_var();
    let trunc = 30;

    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);

    let mut qpoch_gen = qpochhammer_inf_generator(QRat::one(), 1, q, trunc);
    qpoch_gen.ensure_order(trunc);

    // Compare coefficient by coefficient
    for k in 0..trunc {
        assert_eq!(
            euler_gen.series().coeff(k),
            qpoch_gen.series().coeff(k),
            "euler vs qpochhammer(1,1) mismatch at q^{}",
            k
        );
    }
}

// ===========================================================================
// 6. Performance sanity check
// ===========================================================================

/// Generate Euler function to O(q^100). Should complete quickly.
/// Verify specific pentagonal number coefficients at higher orders.
#[test]
fn euler_function_order_100() {
    let q = q_var();
    let mut ipg = euler_function_generator(q, 101);
    ipg.ensure_order(101);
    let fps = ipg.series();

    // k=7: pentagonal number = 7*(3*7-1)/2 = 7*20/2 = 70, sign = (-1)^7 = -1
    assert_eq!(fps.coeff(70), qrat(-1), "q^70 should be -1 (k=7)");

    // k=-7: pentagonal number = 7*(3*7+1)/2 = 7*22/2 = 77, sign = (-1)^{-7} = -1
    assert_eq!(fps.coeff(77), qrat(-1), "q^77 should be -1 (k=-7)");

    // k=8: pentagonal number = 8*(3*8-1)/2 = 8*23/2 = 92, sign = (-1)^8 = +1
    assert_eq!(fps.coeff(92), qrat(1), "q^92 should be +1 (k=8)");

    // k=-8: pentagonal number = 8*(3*8+1)/2 = 8*25/2 = 100, sign = (-1)^{-8} = +1
    assert_eq!(fps.coeff(100), qrat(1), "q^100 should be +1 (k=-8)");

    // Non-pentagonal numbers in the range should be zero
    assert_eq!(fps.coeff(71), QRat::zero(), "q^71 should be 0");
    assert_eq!(fps.coeff(80), QRat::zero(), "q^80 should be 0");
    assert_eq!(fps.coeff(99), QRat::zero(), "q^99 should be 0");
}
