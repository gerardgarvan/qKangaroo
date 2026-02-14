//! Comprehensive tests for prodmake (Andrews' algorithm) and post-processing
//! functions (etamake, jacprodmake, mprodmake, qetamake) verifying:
//!
//! - prodmake recovers exponents a_n = -1 for all n from the Euler function (q;q)_inf
//! - prodmake recovers exponents a_n = 1 for all n from 1/(q;q)_inf (partition GF)
//! - prodmake handles distinct_parts_gf correctly
//! - Round-trip verification: prodmake output re-expanded matches original series
//! - Edge cases and normalization
//! - etamake correctly identifies eta-quotients
//! - jacprodmake recovers JAC parameters from Jacobi products
//! - mprodmake identifies (1+q^n) products
//! - qetamake produces (q^d;q^d)_inf notation

use qsym_core::number::QRat;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;
use qsym_core::qseries::{
    prodmake, partition_gf, distinct_parts_gf, jacprod,
    etamake, jacprodmake, mprodmake, qetamake,
};
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

// ===========================================================================
// 7. etamake tests
// ===========================================================================

/// etamake on (q;q)_inf should return eta(tau)^1 = {1: 1}, q_shift = 1/24.
///
/// (q;q)_inf = prod (1-q^n) = q^{-1/24} * eta(tau)
/// So etamake should find r_1 = 1 (eta(tau)^1).
/// q_shift = sum r_d * d / 24 = 1 * 1 / 24 = 1/24.
#[test]
fn test_etamake_euler_function() {
    let q = q_var();
    let trunc = 25;
    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler_series = euler_gen.into_series();

    let eta = etamake(&euler_series, 20);

    // Should have r_1 = 1, all others 0
    assert_eq!(
        *eta.factors.get(&1).unwrap_or(&0), 1,
        "etamake Euler: r_1 should be 1"
    );
    for n in 2..=20 {
        assert_eq!(
            *eta.factors.get(&n).unwrap_or(&0), 0,
            "etamake Euler: r_{} should be 0, got {}", n, eta.factors.get(&n).unwrap_or(&0)
        );
    }

    // q_shift = 1/24
    let expected_shift = QRat::from((1i64, 24i64));
    assert_eq!(eta.q_shift, expected_shift, "etamake Euler: q_shift should be 1/24");
}

/// etamake on 1/(q;q)_inf (partition GF) should return eta(tau)^{-1} = {1: -1}.
///
/// 1/(q;q)_inf = q^{1/24} / eta(tau)
/// q_shift = -1 * 1 / 24 = -1/24.
#[test]
fn test_etamake_partition_gf() {
    let q = q_var();
    let trunc = 25;
    let pgf = partition_gf(q, trunc);

    let eta = etamake(&pgf, 20);

    // Should have r_1 = -1
    assert_eq!(
        *eta.factors.get(&1).unwrap_or(&0), -1,
        "etamake partition GF: r_1 should be -1"
    );
    for n in 2..=20 {
        assert_eq!(
            *eta.factors.get(&n).unwrap_or(&0), 0,
            "etamake partition GF: r_{} should be 0", n
        );
    }

    // q_shift = -1/24
    let expected_shift = QRat::from((-1i64, 24i64));
    assert_eq!(eta.q_shift, expected_shift, "etamake partition GF: q_shift should be -1/24");
}

/// etamake on (q;q)_inf * (q^2;q^2)_inf should return {1: 1, 2: 1}.
///
/// This is prod (1-q^n) * prod (1-q^{2n}).
/// In prodmake form: a_n = -1 for all n, and additionally a_{2n} -= 1 for even n.
/// So a_n = -1 for odd n, a_n = -2 for even n.
///
/// etamake should recover r_1 = 1, r_2 = 1.
/// q_shift = (1*1 + 1*2)/24 = 3/24 = 1/8.
#[test]
fn test_etamake_two_eta_factors() {
    let q = q_var();
    let trunc = 30;

    // Build (q;q)_inf * (q^2;q^2)_inf
    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler = euler_gen.into_series();

    // (q^2;q^2)_inf = prod_{n>=1}(1 - q^{2n})
    let mut prod_even = FormalPowerSeries::one(q, trunc);
    for n in 1..trunc/2 {
        let mut factor = FormalPowerSeries::one(q, trunc);
        factor.set_coeff(2 * n, -QRat::one());
        prod_even = arithmetic::mul(&prod_even, &factor);
    }

    let combined = arithmetic::mul(&euler, &prod_even);

    let eta = etamake(&combined, 20);

    assert_eq!(
        *eta.factors.get(&1).unwrap_or(&0), 1,
        "etamake two factors: r_1 should be 1"
    );
    assert_eq!(
        *eta.factors.get(&2).unwrap_or(&0), 1,
        "etamake two factors: r_2 should be 1"
    );
    for n in 3..=20 {
        assert_eq!(
            *eta.factors.get(&n).unwrap_or(&0), 0,
            "etamake two factors: r_{} should be 0", n
        );
    }

    // q_shift = (1 + 2)/24 = 3/24 = 1/8
    let expected_shift = QRat::from((1i64, 8i64));
    assert_eq!(eta.q_shift, expected_shift, "etamake two factors: q_shift should be 1/8");
}

// ===========================================================================
// 8. jacprodmake tests
// ===========================================================================

/// jacprodmake on (q;q)_inf should find it as a JAC-like product.
///
/// (q;q)_inf = prod(1-q^n). In JAC terms this does not directly correspond to
/// a JAC(a,b) since JAC has three sub-products. But period b=1 is a special case:
/// JAC(0,1) doesn't exist (a must be > 0), so period 1 is handled via residue 0 only.
/// The result should show the product was analyzed, though no JAC(r,b) factors
/// with r > 0 will be found for a uniform-exponent product.
#[test]
fn test_jacprodmake_euler() {
    let q = q_var();
    let trunc = 25;
    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler_series = euler_gen.into_series();

    let jac = jacprodmake(&euler_series, 20);

    // Euler function has all a_n = -1, which is uniform across all residue classes.
    // The best period depends on the algorithm, but the key check is that it works
    // without panicking and returns a result.
    // For a uniform product, no specific JAC(r,b) pattern fits perfectly since
    // the exponents at residue 0 (mod b) can't be separated from the JAC contributions.
    assert!(
        !jac.factors.is_empty() || jac.is_exact,
        "jacprodmake on Euler should return a result"
    );
}

/// jacprodmake on jacprod(1, 5) should recover JAC(1, 5).
///
/// JAC(1,5) = (q;q^5)_inf * (q^4;q^5)_inf * (q^5;q^5)_inf
/// This is the product with period 5, residue 1 (and complementary 4).
#[test]
fn test_jacprodmake_jacobi_product() {
    let q = q_var();
    let trunc = 30;
    let jac_series = jacprod(1, 5, q, trunc);

    let result = jacprodmake(&jac_series, 25);

    // Should find JAC(1,5) with exponent 1
    let jac_1_5 = result.factors.get(&(1, 5)).copied().unwrap_or(0);
    assert_eq!(
        jac_1_5, 1,
        "jacprodmake on JAC(1,5): should find JAC(1,5)^1, got exponent {}",
        jac_1_5
    );
    assert!(
        result.is_exact,
        "jacprodmake on JAC(1,5): should be exact"
    );
}

/// jacprodmake on jacprod(2, 5) should recover JAC(2, 5).
///
/// JAC(2,5) = (q^2;q^5)_inf * (q^3;q^5)_inf * (q^5;q^5)_inf
#[test]
fn test_jacprodmake_jac_2_5() {
    let q = q_var();
    let trunc = 30;
    let jac_series = jacprod(2, 5, q, trunc);

    let result = jacprodmake(&jac_series, 25);

    let jac_2_5 = result.factors.get(&(2, 5)).copied().unwrap_or(0);
    assert_eq!(
        jac_2_5, 1,
        "jacprodmake on JAC(2,5): should find JAC(2,5)^1, got exponent {}",
        jac_2_5
    );
    assert!(
        result.is_exact,
        "jacprodmake on JAC(2,5): should be exact"
    );
}

/// jacprodmake on a non-periodic product should set is_exact = false.
///
/// Construct a product with linearly growing exponents: (1-q^n)^{-n} for n=1..max.
/// This has a_n = n which does not fit any JAC(r,b) pattern.
#[test]
fn test_jacprodmake_no_pattern() {
    let q = q_var();
    let trunc = 30;

    // Build prod (1-q^n)^{-n} for n=1..10
    // This is 1/(1-q)^1 * 1/(1-q^2)^2 * 1/(1-q^3)^3 * ...
    let mut series = FormalPowerSeries::one(q, trunc);
    for n in 1..=10 {
        let mut factor = FormalPowerSeries::one(q, trunc);
        factor.set_coeff(n, -QRat::one());
        // Compute (1-q^n)^n
        let mut power = FormalPowerSeries::one(q, trunc);
        for _ in 0..n {
            power = arithmetic::mul(&power, &factor);
        }
        // Invert to get (1-q^n)^{-n}
        power = arithmetic::invert(&power);
        series = arithmetic::mul(&series, &power);
    }

    let result = jacprodmake(&series, 15);

    // Non-periodic exponents should not be exactly representable as JAC products
    assert!(
        !result.is_exact,
        "jacprodmake on non-periodic exponents should set is_exact = false"
    );
}

/// jacprodmake on JAC(1,5)^2 should recover exponent 2.
#[test]
fn test_jacprodmake_squared() {
    let q = q_var();
    let trunc = 30;
    let jac1 = jacprod(1, 5, q, trunc);
    let jac_sq = arithmetic::mul(&jac1, &jac1);

    let result = jacprodmake(&jac_sq, 25);

    let jac_1_5 = result.factors.get(&(1, 5)).copied().unwrap_or(0);
    assert_eq!(
        jac_1_5, 2,
        "jacprodmake on JAC(1,5)^2: should find exponent 2, got {}",
        jac_1_5
    );
    assert!(
        result.is_exact,
        "jacprodmake on JAC(1,5)^2: should be exact"
    );
}

// ===========================================================================
// 9. mprodmake tests
// ===========================================================================

/// mprodmake on (-q;q)_inf = prod(1+q^n) should recover exponents 1 for all n.
///
/// The distinct parts GF is prod_{n>=1}(1 + q^n).
/// mprodmake should return m_n = 1 for all n.
#[test]
fn test_mprodmake_distinct_parts() {
    let q = q_var();
    let trunc = 25;
    let dpgf = distinct_parts_gf(q, trunc);

    let result = mprodmake(&dpgf, 20);

    // All m_1..m_20 should be 1 (each (1+q^n) factor appears once)
    for n in 1..=20 {
        let m_n = *result.get(&n).unwrap_or(&0);
        assert_eq!(
            m_n, 1,
            "mprodmake distinct parts: m_{} should be 1, got {}", n, m_n
        );
    }
}

/// mprodmake on prod(1+q^n)^2 should recover exponents 2 for all n.
#[test]
fn test_mprodmake_squared() {
    let q = q_var();
    let trunc = 25;
    let dpgf = distinct_parts_gf(q, trunc);
    let dpgf_sq = arithmetic::mul(&dpgf, &dpgf);

    let result = mprodmake(&dpgf_sq, 15);

    for n in 1..=15 {
        let m_n = *result.get(&n).unwrap_or(&0);
        assert_eq!(
            m_n, 2,
            "mprodmake squared: m_{} should be 2, got {}", n, m_n
        );
    }
}

/// mprodmake on 1/(1+q) should give m_1 = -1, all others 0.
///
/// 1/(1+q) = (1-q)/(1-q^2), so in (1+q^n) notation, (1+q)^{-1}.
#[test]
fn test_mprodmake_single_factor_inverse() {
    let q = q_var();
    let trunc = 25;

    // Build 1/(1+q) = 1 - q + q^2 - q^3 + ...
    let mut one_plus_q = FormalPowerSeries::one(q, trunc);
    one_plus_q.set_coeff(1, QRat::one());
    let series = arithmetic::invert(&one_plus_q);

    let result = mprodmake(&series, 15);

    assert_eq!(
        *result.get(&1).unwrap_or(&0), -1,
        "mprodmake 1/(1+q): m_1 should be -1"
    );
    for n in 2..=15 {
        assert_eq!(
            *result.get(&n).unwrap_or(&0), 0,
            "mprodmake 1/(1+q): m_{} should be 0", n
        );
    }
}

// ===========================================================================
// 10. qetamake tests
// ===========================================================================

/// qetamake on (q;q)_inf should return {1: 1} with q_shift = 0.
///
/// (q;q)_inf = (q;q)_inf^1 in q-eta notation.
/// The q-eta notation does NOT include the q^{d/24} factor from eta.
#[test]
fn test_qetamake_euler() {
    let q = q_var();
    let trunc = 25;
    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler_series = euler_gen.into_series();

    let qeta = qetamake(&euler_series, 20);

    assert_eq!(
        *qeta.factors.get(&1).unwrap_or(&0), 1,
        "qetamake Euler: r_1 should be 1"
    );
    for n in 2..=20 {
        assert_eq!(
            *qeta.factors.get(&n).unwrap_or(&0), 0,
            "qetamake Euler: r_{} should be 0", n
        );
    }

    // q_shift should be 0 (no prefactor in q-eta notation)
    assert_eq!(qeta.q_shift, QRat::zero(), "qetamake Euler: q_shift should be 0");
}

/// qetamake on 1/(q;q)_inf should return {1: -1} with q_shift = 0.
#[test]
fn test_qetamake_partition_gf() {
    let q = q_var();
    let trunc = 25;
    let pgf = partition_gf(q, trunc);

    let qeta = qetamake(&pgf, 20);

    assert_eq!(
        *qeta.factors.get(&1).unwrap_or(&0), -1,
        "qetamake partition GF: r_1 should be -1"
    );
    for n in 2..=20 {
        assert_eq!(
            *qeta.factors.get(&n).unwrap_or(&0), 0,
            "qetamake partition GF: r_{} should be 0", n
        );
    }

    assert_eq!(qeta.q_shift, QRat::zero(), "qetamake partition GF: q_shift should be 0");
}

/// qetamake on (q;q)_inf * (q^2;q^2)_inf should return {1: 1, 2: 1}.
#[test]
fn test_qetamake_two_factors() {
    let q = q_var();
    let trunc = 30;

    // Build (q;q)_inf * (q^2;q^2)_inf
    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler = euler_gen.into_series();

    let mut prod_even = FormalPowerSeries::one(q, trunc);
    for n in 1..trunc/2 {
        let mut factor = FormalPowerSeries::one(q, trunc);
        factor.set_coeff(2 * n, -QRat::one());
        prod_even = arithmetic::mul(&prod_even, &factor);
    }

    let combined = arithmetic::mul(&euler, &prod_even);
    let qeta = qetamake(&combined, 20);

    assert_eq!(*qeta.factors.get(&1).unwrap_or(&0), 1);
    assert_eq!(*qeta.factors.get(&2).unwrap_or(&0), 1);
    assert_eq!(qeta.q_shift, QRat::zero());
}
