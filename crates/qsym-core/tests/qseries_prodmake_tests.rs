//! Comprehensive tests for prodmake (Andrews' algorithm) verifying:
//!
//! - prodmake recovers exponents a_n = -1 for all n from the Euler function (q;q)_inf
//! - prodmake recovers exponents a_n = 1 for all n from 1/(q;q)_inf (partition GF)
//! - prodmake handles distinct_parts_gf correctly
//! - Round-trip verification: prodmake output re-expanded matches original series
//! - Edge cases and normalization

use qsym_core::number::QRat;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;
use qsym_core::qseries::{prodmake, partition_gf, distinct_parts_gf};
use qsym_core::series::FormalPowerSeries;
use qsym_core::series::generator::euler_function_generator;
use qsym_core::series::arithmetic;

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
// 1. prodmake on Euler function (q;q)_inf
// ===========================================================================

/// The Euler function (q;q)_inf = prod_{n>=1}(1 - q^n).
/// In the prodmake convention, f = prod (1-q^n)^{-a_n}, so
/// (q;q)_inf = prod (1-q^n)^{-(-1)}, meaning a_n = -1 for all n.
#[test]
fn prodmake_euler_function() {
    let q = q_var();
    let trunc = 25;
    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler_series = euler_gen.into_series();

    let result = prodmake(&euler_series, 20);

    // All exponents a_1..a_20 should be -1
    for n in 1..=20 {
        let a_n = result.exponents.get(&n).cloned().unwrap_or_else(QRat::zero);
        assert_eq!(
            a_n, qrat(-1),
            "Euler function: prodmake exponent a_{} should be -1, got {}", n, a_n
        );
    }
    assert_eq!(result.terms_used, 20);
}

// ===========================================================================
// 2. prodmake on partition generating function 1/(q;q)_inf
// ===========================================================================

/// The partition GF = 1/(q;q)_inf = prod_{n>=1} 1/(1-q^n) = prod (1-q^n)^{-1}.
/// So a_n = 1 for all n.
#[test]
fn prodmake_partition_gf() {
    let q = q_var();
    let trunc = 25;
    let pgf = partition_gf(q, trunc);

    let result = prodmake(&pgf, 20);

    // All exponents a_1..a_20 should be +1
    for n in 1..=20 {
        let a_n = result.exponents.get(&n).cloned().unwrap_or_else(QRat::zero);
        assert_eq!(
            a_n, qrat(1),
            "Partition GF: prodmake exponent a_{} should be 1, got {}", n, a_n
        );
    }
}

// ===========================================================================
// 3. prodmake on distinct parts generating function
// ===========================================================================

/// The distinct parts GF = prod_{n>=1}(1 + q^n).
///
/// We have 1 + q^n = (1 - q^{2n})/(1 - q^n), so:
///   prod(1 + q^n) = prod (1-q^{2n}) / prod (1-q^n)
///                 = prod (1-q^n)^{-a_n}
///
/// where a_n = 1 for odd n (from the denominator), and for even n:
///   a_{2k} comes from denominator (+1) minus numerator (-1 from q^{2k}),
///   specifically a_{2k} = 1 - 1 = 0 when 2k = 2m for some m that contributes once.
///
/// More precisely: prod(1+q^n) = prod_{n odd}(1-q^n)^{-1} * prod_{n even}(1-q^n)^{0}...
/// Wait, let's think more carefully.
///
/// prod(1+q^n) = prod (1-q^{2n})/(1-q^n) for n=1,2,3,...
///
/// In prodmake notation: f = prod (1-q^k)^{-a_k}
/// From denominator prod 1/(1-q^n): contributes a_n += 1 for all n
/// From numerator prod (1-q^{2n}): contributes a_{2n} -= 1 for all n
///
/// Net: a_k = 1 for all k, minus 1 for even k.
/// So a_k = 1 if k is odd, a_k = 0 if k is even.
#[test]
fn prodmake_distinct_parts_gf() {
    let q = q_var();
    let trunc = 25;
    let dpgf = distinct_parts_gf(q, trunc);

    let result = prodmake(&dpgf, 20);

    for n in 1..=20 {
        let a_n = result.exponents.get(&n).cloned().unwrap_or_else(QRat::zero);
        if n % 2 == 1 {
            assert_eq!(
                a_n, qrat(1),
                "Distinct parts GF: a_{} should be 1 (odd), got {}", n, a_n
            );
        } else {
            assert_eq!(
                a_n, qrat(0),
                "Distinct parts GF: a_{} should be 0 (even), got {}", n, a_n
            );
        }
    }
}

// ===========================================================================
// 4. Round-trip verification
// ===========================================================================

/// Take the partition GF, run prodmake, rebuild the product from exponents,
/// and verify coefficient-by-coefficient agreement with the original series.
#[test]
fn prodmake_round_trip_partition_gf() {
    let q = q_var();
    let trunc = 25;
    let pgf = partition_gf(q, trunc);

    let result = prodmake(&pgf, 20);

    // Rebuild the series from the exponents:
    // f = prod_{n=1}^{20} (1-q^n)^{-a_n}
    // For each n with a_n != 0, compute (1-q^n)^{-a_n} as a series and multiply.
    let mut rebuilt = FormalPowerSeries::one(q, trunc);
    for (&n, a_n) in &result.exponents {
        if a_n.is_zero() {
            continue;
        }
        // (1-q^n) as a series
        let mut factor = FormalPowerSeries::one(q, trunc);
        factor.set_coeff(n, -QRat::one());

        // Compute (1-q^n)^{|a_n|} or its inverse depending on sign of a_n
        // Since a_n should be an integer, extract it
        let a_val = a_n.0.to_f64();
        let abs_a = a_val.abs() as i64;

        // Compute (1-q^n)^{abs_a}
        let mut power_series = FormalPowerSeries::one(q, trunc);
        for _ in 0..abs_a {
            power_series = arithmetic::mul(&power_series, &factor);
        }

        // If a_n > 0, we need (1-q^n)^{-a_n}, so invert the power
        // If a_n < 0, we need (1-q^n)^{-a_n} = (1-q^n)^{|a_n|}, no inversion
        if a_val > 0.0 {
            power_series = arithmetic::invert(&power_series);
        }

        rebuilt = arithmetic::mul(&rebuilt, &power_series);
    }

    // Compare coefficients
    for k in 0..21 {
        assert_eq!(
            rebuilt.coeff(k), pgf.coeff(k),
            "Round-trip mismatch at q^{}: rebuilt={}, original={}", k, rebuilt.coeff(k), pgf.coeff(k)
        );
    }
}

/// Round-trip for the Euler function.
#[test]
fn prodmake_round_trip_euler() {
    let q = q_var();
    let trunc = 25;
    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler_series = euler_gen.into_series();

    let result = prodmake(&euler_series, 20);

    // Rebuild: each a_n = -1, so (1-q^n)^{-(-1)} = (1-q^n)^1
    let mut rebuilt = FormalPowerSeries::one(q, trunc);
    for (&n, a_n) in &result.exponents {
        if a_n.is_zero() {
            continue;
        }
        let mut factor = FormalPowerSeries::one(q, trunc);
        factor.set_coeff(n, -QRat::one());

        let a_val = a_n.0.to_f64();
        let abs_a = a_val.abs() as i64;

        let mut power_series = FormalPowerSeries::one(q, trunc);
        for _ in 0..abs_a {
            power_series = arithmetic::mul(&power_series, &factor);
        }

        if a_val > 0.0 {
            power_series = arithmetic::invert(&power_series);
        }

        rebuilt = arithmetic::mul(&rebuilt, &power_series);
    }

    for k in 0..21 {
        assert_eq!(
            rebuilt.coeff(k), euler_series.coeff(k),
            "Round-trip Euler mismatch at q^{}", k
        );
    }
}

// ===========================================================================
// 5. Edge cases
// ===========================================================================

/// prodmake on the constant series 1 + O(q^N) should return empty exponents.
#[test]
fn prodmake_constant_one() {
    let q = q_var();
    let series = FormalPowerSeries::one(q, 20);

    let result = prodmake(&series, 15);

    assert!(
        result.exponents.is_empty(),
        "prodmake of 1 should have no nonzero exponents, got {:?}", result.exponents
    );
}

/// prodmake on (1 - q)^{-2} = (sum (n+1) q^n) should give a_1 = 2, all others 0.
#[test]
fn prodmake_single_factor_squared() {
    let q = q_var();
    let trunc = 25;

    // Build (1-q)^{-2} = 1/(1-q)^2 = sum_{n>=0} (n+1) q^n
    let mut factor = FormalPowerSeries::one(q, trunc);
    factor.set_coeff(1, -QRat::one());
    let factor_sq = arithmetic::mul(&factor, &factor);
    let series = arithmetic::invert(&factor_sq);

    let result = prodmake(&series, 20);

    // Should have a_1 = 2
    let a1 = result.exponents.get(&1).cloned().unwrap_or_else(QRat::zero);
    assert_eq!(a1, qrat(2), "prodmake of 1/(1-q)^2: a_1 should be 2, got {}", a1);

    // All other exponents should be 0
    for n in 2..=20 {
        let a_n = result.exponents.get(&n).cloned().unwrap_or_else(QRat::zero);
        assert_eq!(
            a_n, qrat(0),
            "prodmake of 1/(1-q)^2: a_{} should be 0, got {}", n, a_n
        );
    }
}

/// prodmake on a series with non-unit constant term (normalization test).
/// Build 5 * partition_gf, which has constant term 5.
/// prodmake should normalize and still recover a_n = 1.
#[test]
fn prodmake_non_unit_constant() {
    let q = q_var();
    let trunc = 25;
    let pgf = partition_gf(q, trunc);

    // Multiply by 5
    let scaled = arithmetic::scalar_mul(&qrat(5), &pgf);

    let result = prodmake(&scaled, 15);

    for n in 1..=15 {
        let a_n = result.exponents.get(&n).cloned().unwrap_or_else(QRat::zero);
        assert_eq!(
            a_n, qrat(1),
            "Scaled partition GF: a_{} should be 1 after normalization, got {}", n, a_n
        );
    }
}

// ===========================================================================
// 6. prodmake on etaq(1, 2) = (q; q^2)_inf ... a more complex product
// ===========================================================================

/// Test prodmake on (1-q)(1-q^3)(1-q^5)... = prod_{k>=0}(1-q^{2k+1}).
/// This is prod (1-q^n)^{-a_n} where a_n = -1 for odd n, 0 for even n.
#[test]
fn prodmake_odd_factors_product() {
    let q = q_var();
    let trunc = 25;

    // Build prod_{k=0..}(1 - q^{2k+1}) manually
    let mut product = FormalPowerSeries::one(q, trunc);
    let mut k = 0i64;
    while 2 * k + 1 < trunc {
        let exp = 2 * k + 1;
        let mut factor = FormalPowerSeries::one(q, trunc);
        factor.set_coeff(exp, -QRat::one());
        product = arithmetic::mul(&product, &factor);
        k += 1;
    }

    let result = prodmake(&product, 20);

    for n in 1..=20 {
        let a_n = result.exponents.get(&n).cloned().unwrap_or_else(QRat::zero);
        if n % 2 == 1 {
            assert_eq!(
                a_n, qrat(-1),
                "Odd product: a_{} should be -1 (odd), got {}", n, a_n
            );
        } else {
            assert_eq!(
                a_n, qrat(0),
                "Odd product: a_{} should be 0 (even), got {}", n, a_n
            );
        }
    }
}
