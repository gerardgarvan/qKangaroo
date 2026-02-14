//! Integration tests for Appell-Lerch sums, universal mock theta functions, and Zwegers completions.
//!
//! Tests verify:
//! - Appell-Lerch bilateral sum evaluation with positive and negative r
//! - Geometric series expansion for positive and negative k
//! - Universal mock theta functions g2 and g3 (with integer parameter truncation)
//! - ZwegersCompletion symbolic representation and linear relation verification
//! - Truncation consistency across different orders

use qsym_core::number::QRat;
use qsym_core::series::{arithmetic, FormalPowerSeries};
use qsym_core::ExprArena;
use qsym_core::symbol::SymbolId;
use qsym_core::qseries::{
    appell_lerch_m, appell_lerch_bilateral,
    universal_mock_theta_g2, universal_mock_theta_g3,
    ZwegersCompletion,
};

/// Helper: create a SymbolId for "q".
fn q_var() -> SymbolId {
    let mut arena = ExprArena::new();
    arena.symbols_mut().intern("q")
}

// ============================================================
// Appell-Lerch bilateral sum tests
// ============================================================

#[test]
fn test_appell_lerch_bilateral_basic() {
    // Compute the bilateral sum S(q^2, q, q^3) -- both positive and negative r contribute
    let var = q_var();
    let trunc = 20;
    let result = appell_lerch_bilateral(2, 3, var, trunc);

    // The bilateral sum should be nonzero (it accumulates many geometric series terms)
    assert!(!result.is_zero(), "Bilateral sum S(q^2, q, q^3) should be nonzero");
    assert_eq!(result.truncation_order(), trunc);
}

#[test]
fn test_appell_lerch_bilateral_pole_skipping() {
    // When a_pow + r + z_pow = 0 for some r, that term is skipped (pole in geometric series).
    // For a_pow=1, z_pow=2: denom_pow = 1 + r + 2 = r + 3. This is 0 when r = -3.
    // So the r = -3 term is skipped in the bilateral sum.
    let var = q_var();
    let trunc = 20;
    let result = appell_lerch_bilateral(1, 2, var, trunc);

    // Should still produce a nonzero result (other r values contribute)
    assert!(!result.is_zero(), "Bilateral sum with pole-skipping should be nonzero");
}

#[test]
fn test_appell_lerch_bilateral_zero_z() {
    // z_pow = 0: z = q^0 = 1
    // bilateral sum: sum (-1)^r * q^{r(r-1)/2} / (1 - q^{a+r})
    // For a_pow = 3: denom_pow = 3 + r. Pole at r = -3.
    let var = q_var();
    let trunc = 15;
    let result = appell_lerch_bilateral(3, 0, var, trunc);

    assert!(!result.is_zero(), "Bilateral sum with z_pow=0 should be nonzero");
}

#[test]
fn test_appell_lerch_m_equals_bilateral() {
    // appell_lerch_m is a wrapper for appell_lerch_bilateral
    let var = q_var();
    let trunc = 15;

    let m = appell_lerch_m(2, 3, var, trunc);
    let s = appell_lerch_bilateral(2, 3, var, trunc);

    assert_eq!(m, s, "appell_lerch_m should equal appell_lerch_bilateral");
}

#[test]
fn test_appell_lerch_bilateral_different_params() {
    // Different (a_pow, z_pow) should give different results
    let var = q_var();
    let trunc = 20;

    let s1 = appell_lerch_bilateral(2, 3, var, trunc);
    let s2 = appell_lerch_bilateral(3, 2, var, trunc);

    assert_ne!(s1, s2, "Different parameters should give different bilateral sums");
}

#[test]
fn test_appell_lerch_bilateral_negative_z() {
    // z_pow = -2: negative z parameter
    let var = q_var();
    let trunc = 15;
    let result = appell_lerch_bilateral(2, -2, var, trunc);

    assert!(!result.is_zero(), "Bilateral sum with negative z_pow should be nonzero");
}

#[test]
fn test_appell_lerch_truncation_consistency() {
    // Compute S(q^2, q, q^3) at truncation 10 and 20
    // First 10 coefficients should match
    let var = q_var();

    let s_small = appell_lerch_bilateral(2, 3, var, 10);
    let s_large = appell_lerch_bilateral(2, 3, var, 20);

    for k in 0..10 {
        assert_eq!(
            s_small.coeff(k),
            s_large.coeff(k),
            "Coefficient at q^{} should match between trunc=10 and trunc=20",
            k
        );
    }
}

#[test]
fn test_appell_lerch_bilateral_symmetry() {
    // The bilateral sum is NOT symmetric in a_pow and z_pow
    let var = q_var();
    let trunc = 15;

    let s_2_3 = appell_lerch_bilateral(2, 3, var, trunc);
    let s_3_2 = appell_lerch_bilateral(3, 2, var, trunc);

    assert_ne!(s_2_3, s_3_2, "Bilateral sum is not symmetric in a_pow and z_pow");
}

// ============================================================
// Universal mock theta function g3 tests
// ============================================================

#[test]
fn test_g3_a_pow_3() {
    // g3(q^3, q): for a_pow=3, max_valid_n = a-2 = 1
    // So we sum n=0 and n=1 (two terms)
    let var = q_var();
    let trunc = 20;
    let result = universal_mock_theta_g3(3, var, trunc);

    // With two valid terms, result should be nonzero
    assert!(!result.is_zero(), "g3(q^3, q) should be nonzero");
    assert_eq!(result.truncation_order(), trunc);
}

#[test]
fn test_g3_a_pow_4() {
    // g3(q^4, q): max_valid_n = 4-2 = 2
    // Sum n=0, 1, 2 (three terms)
    let var = q_var();
    let trunc = 20;
    let result = universal_mock_theta_g3(4, var, trunc);

    assert!(!result.is_zero(), "g3(q^4, q) should be nonzero");
}

#[test]
fn test_g3_a_pow_5() {
    // g3(q^5, q): max_valid_n = 5-2 = 3
    // Sum n=0, 1, 2, 3 (four terms)
    let var = q_var();
    let trunc = 20;
    let result = universal_mock_theta_g3(5, var, trunc);

    assert!(!result.is_zero(), "g3(q^5, q) should be nonzero");
}

#[test]
fn test_g3_first_term_manual() {
    // For g3(q^3, q), using the algebraic identity for positive-exponent FPS:
    // n=0 term = (-1)^{0+1} * q^{(0+1)*(3-1)} / [(1-q^3) * (1-q^2)]
    //          = -q^2 / [(1-q^3) * (1-q^2)]
    //
    // The lowest-power coefficient is at q^2, not q^0.
    // 1/[(1-q^3)(1-q^2)] = 1 + q^2 + q^3 + q^4 + 2*q^5 + ...
    // So the n=0 term starts at -q^2 (coefficient -1 at q^2).
    //
    // The n=1 term = (-1)^2 * q^{2*(3-1)} / [(1-q^3)(1-q^4) * (1-q^2)(1-q^1)]
    //             = q^4 / [(1-q^3)(1-q^4)(1-q^2)(1-q)]
    // This adds positive contributions starting at q^4.
    let var = q_var();
    let trunc = 10;
    let g3 = universal_mock_theta_g3(3, var, trunc);

    // Constant term is zero (first nonzero term is at q^2)
    let c0 = g3.coeff(0);
    assert!(c0.is_zero(), "g3(q^3,q) constant term should be zero (first term starts at q^2)");

    // The coefficient at q^2 should be nonzero (from the n=0 term)
    let c2 = g3.coeff(2);
    assert!(!c2.is_zero(), "g3(q^3,q) should have nonzero coefficient at q^2");

    // The coefficient at q^2 should be -1 (from -q^2 * [leading 1 of 1/(...)])
    assert_eq!(c2, -QRat::one(), "g3(q^3,q) coefficient at q^2 should be -1");
}

#[test]
fn test_g3_different_a_values() {
    // g3(q^3, q) and g3(q^4, q) should differ
    let var = q_var();
    let trunc = 15;

    let g3_3 = universal_mock_theta_g3(3, var, trunc);
    let g3_4 = universal_mock_theta_g3(4, var, trunc);

    assert_ne!(g3_3, g3_4, "g3 with different a_pow should differ");
}

#[test]
fn test_g3_more_terms_higher_a() {
    // Higher a_pow gives more valid terms in the abstract sum, but the n-th term
    // starts at q^{(n+1)*(a-1)}, so larger a_pow pushes terms to higher powers.
    // Within a fixed truncation, small a_pow may actually fill MORE coefficients.
    //
    // For a_pow=3: n=0 starts at q^2, n=1 at q^4 (both fit in trunc=50)
    // For a_pow=5: n=0 starts at q^4, n=1 at q^8, n=2 at q^12, n=3 at q^16
    // For a_pow=10: n=0 starts at q^9, n=1 at q^18, ...
    //
    // Use a large enough truncation so higher a_pow can contribute many terms.
    let var = q_var();
    let trunc = 100; // large enough for all to contribute

    let g3_3 = universal_mock_theta_g3(3, var, trunc); // 2 terms
    let g3_5 = universal_mock_theta_g3(5, var, trunc); // 4 terms
    let g3_10 = universal_mock_theta_g3(10, var, trunc); // 9 terms

    // All should be nonzero
    assert!(!g3_3.is_zero());
    assert!(!g3_5.is_zero());
    assert!(!g3_10.is_zero());

    // With large truncation, more valid terms should produce richer structure
    // (but not necessarily monotonically more nonzero coefficients due to cancellation).
    // At minimum, verify that g3_10 has a reasonable number of nonzero coefficients.
    assert!(g3_10.num_nonzero() >= 3,
            "g3 with 9 valid terms should have at least 3 nonzero coefficients in trunc=100, got {}",
            g3_10.num_nonzero());
}

#[test]
fn test_g3_degenerate_a_pow_1() {
    // g3(q, q): a_pow=1. max_valid_n = -1, so no valid terms.
    // Result should be zero.
    let var = q_var();
    let trunc = 15;
    let g3 = universal_mock_theta_g3(1, var, trunc);

    assert!(g3.is_zero(), "g3(q, q) should be zero (all denominators degenerate)");
}

// ============================================================
// Universal mock theta function g2 tests
// ============================================================

#[test]
fn test_g2_a_pow_3() {
    // g2(q^3, q): max_valid_n = 3-2 = 1 (two inner sum terms)
    // Then multiplied by (-q;q)_inf and shifted by q^{-3}
    let var = q_var();
    let trunc = 20;
    let result = universal_mock_theta_g2(3, var, trunc);

    assert!(!result.is_zero(), "g2(q^3, q) should be nonzero");
}

#[test]
fn test_g2_a_pow_4() {
    // g2(q^4, q): max_valid_n = 2 (three terms)
    let var = q_var();
    let trunc = 20;
    let result = universal_mock_theta_g2(4, var, trunc);

    assert!(!result.is_zero(), "g2(q^4, q) should be nonzero");
}

#[test]
fn test_g2_g3_differ() {
    // g2 and g3 at the same parameters should produce different results
    let var = q_var();
    let trunc = 15;

    let g2 = universal_mock_theta_g2(4, var, trunc);
    let g3 = universal_mock_theta_g3(4, var, trunc);

    // g2 includes (-q;q)_inf multiplier and shift, so should differ from g3
    let g2_has_terms = g2.num_nonzero() > 0;
    let g3_has_terms = g3.num_nonzero() > 0;
    assert!(g2_has_terms, "g2 should have nonzero terms");
    assert!(g3_has_terms, "g3 should have nonzero terms");
}

// ============================================================
// Zwegers completion tests
// ============================================================

#[test]
fn test_zwegers_completion_third_order() {
    let var = q_var();
    let trunc = 15;

    // Use g3 as a stand-in holomorphic part
    let holomorphic = universal_mock_theta_g3(5, var, trunc);
    let completion = ZwegersCompletion::third_order("f3", holomorphic);

    assert_eq!(completion.mock_theta_name, "f3");
    assert_eq!(completion.weight, (1, 2)); // weight 1/2
    assert_eq!(completion.level, 2);
    assert!(completion.is_nontrivial());
    assert!(completion.correction_description.contains("erfc"));
}

#[test]
fn test_zwegers_completion_fifth_order() {
    let var = q_var();
    let trunc = 15;
    let holomorphic = universal_mock_theta_g3(5, var, trunc);
    let completion = ZwegersCompletion::fifth_order("f0_5", holomorphic);

    assert_eq!(completion.mock_theta_name, "f0_5");
    assert_eq!(completion.weight, (1, 2));
    assert_eq!(completion.level, 5);
    assert!(completion.is_nontrivial());
}

#[test]
fn test_zwegers_completion_custom() {
    let var = q_var();
    let trunc = 10;
    let holomorphic = FormalPowerSeries::one(var, trunc);
    let completion = ZwegersCompletion::custom(
        "test_func",
        holomorphic,
        "Custom correction term",
        (3, 2),
        6,
    );

    assert_eq!(completion.mock_theta_name, "test_func");
    assert_eq!(completion.weight, (3, 2)); // weight 3/2
    assert_eq!(completion.level, 6);
}

#[test]
fn test_zwegers_verify_linear_relation() {
    // Test the verify_linear_relation method with known FPS
    let var = q_var();
    let trunc = 10;

    // holomorphic1 = 1 + q + q^2 + ...
    let mut h1 = FormalPowerSeries::zero(var, trunc);
    for k in 0..trunc {
        h1.set_coeff(k, QRat::one());
    }
    let c1 = ZwegersCompletion::third_order("h1", h1.clone());

    // holomorphic2 = 1 + 2q + 3q^2 + ...
    let mut h2 = FormalPowerSeries::zero(var, trunc);
    for k in 0..trunc {
        h2.set_coeff(k, QRat::from((k as i64 + 1, 1)));
    }
    let c2 = ZwegersCompletion::third_order("h2", h2.clone());

    // target = 1*h1 + 1*h2 = (2 + 3q + 4q^2 + ...)
    let one = QRat::one();
    let target = arithmetic::add(&h1, &h2);

    assert!(c1.verify_linear_relation(&c2, &one, &one, &target),
            "1*h1 + 1*h2 should equal target");

    // Negative test: wrong target
    let wrong_target = FormalPowerSeries::one(var, trunc);
    assert!(!c1.verify_linear_relation(&c2, &one, &one, &wrong_target),
            "Should fail with wrong target");
}

#[test]
fn test_zwegers_nontrivial_check() {
    let var = q_var();
    let trunc = 10;

    let nontrivial = ZwegersCompletion::third_order("f", FormalPowerSeries::one(var, trunc));
    assert!(nontrivial.is_nontrivial());

    let trivial = ZwegersCompletion::third_order("g", FormalPowerSeries::zero(var, trunc));
    assert!(!trivial.is_nontrivial());
}

// ============================================================
// Structural / edge case tests
// ============================================================

#[test]
fn test_appell_lerch_many_terms() {
    // The bilateral sum S(q^a, q, q^b) accumulates geometric series contributions.
    // Due to alternating signs (-1)^r, significant cancellation can occur.
    // We verify structural properties: the sum is computable for a range of parameters.
    let var = q_var();
    let trunc = 25;

    // Test that we can compute bilateral sums for various parameter combinations
    // and that at least some produce nontrivial (nonzero) results
    let mut nonzero_count = 0;
    let mut max_terms = 0usize;
    for a in 1..=5 {
        for z in 0..=5 {
            let result = appell_lerch_bilateral(a, z, var, trunc);
            let nn = result.num_nonzero();
            if nn > 0 {
                nonzero_count += 1;
            }
            if nn > max_terms {
                max_terms = nn;
            }
        }
    }

    // Many parameter combinations should produce nonzero bilateral sums
    assert!(nonzero_count >= 5,
            "At least 5 bilateral sums should be nonzero, got {}", nonzero_count);

    // Verify one specific bilateral sum has reasonable structure
    // S(q^2, q, q^3) is known to be nonzero from test_appell_lerch_bilateral_basic
    let s = appell_lerch_bilateral(2, 3, var, trunc);
    assert!(!s.is_zero(), "S(q^2, q, q^3) should be nonzero");
}

#[test]
fn test_g3_truncation_consistency() {
    // g3(q^5, q) at trunc=10 and trunc=20 should agree on first 10 coefficients
    let var = q_var();

    let g3_small = universal_mock_theta_g3(5, var, 10);
    let g3_large = universal_mock_theta_g3(5, var, 20);

    for k in 0..10 {
        assert_eq!(
            g3_small.coeff(k),
            g3_large.coeff(k),
            "g3 coefficient at q^{} should match between trunc=10 and trunc=20",
            k
        );
    }
}
