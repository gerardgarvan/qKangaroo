//! Integration tests for JAC and ETA symbolic models.
//!
//! Tests cover:
//! - JacFactor construction and validation (valid + panic cases)
//! - JacExpression.to_series matching jacprod directly
//! - Multi-factor and negative-exponent JAC expressions
//! - EtaExpression weight and q_shift computation
//! - Newman modularity checks (pass and fail cases)
//! - EtaExpression.to_series FPS expansion
//! - from_etaquotient conversion from prodmake output

use qsym_core::number::QRat;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;
use qsym_core::qseries::identity::{JacFactor, JacExpression, EtaExpression, ModularityResult};
use qsym_core::qseries::{etaq, jacprod, etamake};
use qsym_core::series::{FormalPowerSeries, arithmetic};
use qsym_core::series::generator::euler_function_generator;
use std::collections::BTreeMap;

/// Helper: create a SymbolId for "q".
fn q_var() -> SymbolId {
    let mut arena = ExprArena::new();
    arena.symbols_mut().intern("q")
}

/// Helper: create QRat from numerator/denominator.
fn qrat(n: i64, d: i64) -> QRat {
    QRat::from((n, d))
}

// ===========================================================================
// Test 1: JacFactor construction and validation
// ===========================================================================

#[test]
fn jac_factor_valid_construction() {
    let f1 = JacFactor::new(1, 5, 1);
    assert_eq!(f1.a, 1);
    assert_eq!(f1.b, 5);
    assert_eq!(f1.exponent, 1);
    assert!(f1.is_valid());

    let f2 = JacFactor::new(2, 5, -1);
    assert_eq!(f2.a, 2);
    assert_eq!(f2.exponent, -1);
    assert!(f2.is_valid());
}

#[test]
#[should_panic(expected = "a must be > 0")]
fn jac_factor_zero_a_panics() {
    JacFactor::new(0, 5, 1);
}

#[test]
#[should_panic(expected = "a must be < b")]
fn jac_factor_a_equals_b_panics() {
    JacFactor::new(5, 5, 1);
}

// ===========================================================================
// Test 2: JacExpression::single and to_series
// ===========================================================================

#[test]
fn jac_expression_single_matches_jacprod() {
    let q = q_var();
    let trunc = 20;

    let expr = JacExpression::single(1, 5);
    let from_expr = expr.to_series(q, trunc);
    let from_direct = jacprod(1, 5, q, trunc);

    assert_eq!(
        from_expr, from_direct,
        "JacExpression::single(1,5).to_series should match jacprod(1,5)"
    );
}

// ===========================================================================
// Test 3: JacExpression with multiple factors
// ===========================================================================

#[test]
fn jac_expression_multi_factor() {
    let q = q_var();
    let trunc = 20;

    // JAC(1,5)^1 * JAC(2,5)^1
    let expr = JacExpression::new(
        QRat::one(),
        QRat::zero(),
        vec![JacFactor::new(1, 5, 1), JacFactor::new(2, 5, 1)],
    );
    let from_expr = expr.to_series(q, trunc);

    let j1 = jacprod(1, 5, q, trunc);
    let j2 = jacprod(2, 5, q, trunc);
    let from_direct = arithmetic::mul(&j1, &j2);

    assert_eq!(
        from_expr, from_direct,
        "JAC(1,5)*JAC(2,5) expression should match direct product"
    );
}

// ===========================================================================
// Test 4: JacExpression with negative exponent
// ===========================================================================

#[test]
fn jac_expression_negative_exponent() {
    let q = q_var();
    let trunc = 20;

    // JAC(1,2)^{-1}
    let expr = JacExpression::new(
        QRat::one(),
        QRat::zero(),
        vec![JacFactor::new(1, 2, -1)],
    );
    let inverse = expr.to_series(q, trunc);

    // JAC(1,2) directly
    let jac12 = jacprod(1, 2, q, trunc);

    // inverse * jac12 should be 1 + O(q^20)
    let product = arithmetic::mul(&inverse, &jac12);
    let one = FormalPowerSeries::one(q, trunc);

    assert_eq!(
        product, one,
        "JAC(1,2)^(-1) * JAC(1,2) should be 1"
    );
}

// ===========================================================================
// Test 5: EtaExpression construction and basic properties
// ===========================================================================

#[test]
fn eta_expression_delta_function() {
    // eta(tau)^{24} = Delta function: factors={1: 24}, level=1
    let eta = EtaExpression::from_factors(&[(1, 24)], 1);

    // weight = 24/2 = 12
    assert_eq!(eta.weight(), qrat(12, 1), "Delta weight should be 12");

    // q_shift = 1*24/24 = 1
    assert_eq!(eta.q_shift(), qrat(1, 1), "Delta q_shift should be 1");
}

// ===========================================================================
// Test 6: EtaExpression modularity check -- weight nonzero means NOT modular function
// ===========================================================================

#[test]
fn eta_expression_modularity_nonzero_weight() {
    // eta(tau)^{24}: sum(r_delta) = 24 != 0, so NOT a modular function (it's a modular form)
    let eta = EtaExpression::from_factors(&[(1, 24)], 1);
    let result = eta.check_modularity();

    assert!(
        !result.is_modular(),
        "eta(tau)^24 has weight 12, should NOT be a modular function"
    );
    match result {
        ModularityResult::NotModular { failed_conditions } => {
            assert!(
                failed_conditions.iter().any(|s| s.contains("sum(r_delta)")),
                "Should mention weight condition; got: {:?}",
                failed_conditions
            );
        }
        _ => panic!("Expected NotModular"),
    }
}

// ===========================================================================
// Test 7: EtaExpression modularity check -- known modular function passes
// ===========================================================================

#[test]
fn eta_expression_modularity_passes() {
    // f = eta(5*tau)^6 / eta(tau)^6 on Gamma_0(5)
    // factors = {1: -6, 5: 6}, level = 5
    let eta = EtaExpression::from_factors(&[(1, -6), (5, 6)], 5);

    // Verify conditions manually:
    // sum(r_delta) = -6 + 6 = 0: OK
    // sum(delta * r_delta) = 1*(-6) + 5*6 = -6+30 = 24, 24%24=0: OK
    // sum((N/delta) * r_delta) = 5*(-6) + 1*6 = -30+6 = -24, -24%24=0: OK
    // prod(delta^|r_delta|) = 1^6 * 5^6 = 5^6 = 15625, sqrt=125, 125^2=15625: OK

    let result = eta.check_modularity();

    assert!(
        result.is_modular(),
        "eta(5tau)^6/eta(tau)^6 should be modular; got: {:?}",
        match &result {
            ModularityResult::NotModular { failed_conditions } => failed_conditions.clone(),
            _ => vec![],
        }
    );
}

// ===========================================================================
// Test 8: EtaExpression modularity check fails (condition 1)
// ===========================================================================

#[test]
fn eta_expression_modularity_fails_condition1() {
    // factors = {1: 1, 2: -1}, level = 2
    // sum(delta * r_delta) = 1*1 + 2*(-1) = 1-2 = -1, -1%24 != 0: FAIL
    let eta = EtaExpression::from_factors(&[(1, 1), (2, -1)], 2);

    let result = eta.check_modularity();

    assert!(
        !result.is_modular(),
        "eta(tau)/eta(2tau) should fail modularity"
    );
}

// ===========================================================================
// Test 9: EtaExpression.to_series panics for fractional q-shift
// ===========================================================================

#[test]
#[should_panic(expected = "q_shift")]
fn eta_expression_to_series_fractional_q_shift_panics() {
    let q = q_var();
    // eta(tau)^1: q_shift = 1/24, NOT an integer
    let eta = EtaExpression::from_factors(&[(1, 1)], 1);
    let _ = eta.to_series(q, 20);
}

// ===========================================================================
// Test 10: EtaExpression.to_series for valid integral q-shift
// ===========================================================================

#[test]
fn eta_expression_to_series_valid() {
    let q = q_var();
    let trunc = 30;

    // f = eta(5*tau)^6 / eta(tau)^6 on Gamma_0(5)
    // q_shift = (1*(-6) + 5*6)/24 = 24/24 = 1
    let eta = EtaExpression::from_factors(&[(1, -6), (5, 6)], 5);
    let from_eta = eta.to_series(q, trunc);

    // Independently compute: q^1 * (q^5;q^5)_inf^6 * (q;q)_inf^{-6}
    let qq_inf = etaq(1, 1, q, trunc);
    let q5q5_inf = etaq(5, 5, q, trunc);

    // (q;q)_inf^{-6}
    let qq_inv = arithmetic::invert(&qq_inf);
    let mut qq_inv6 = FormalPowerSeries::one(q, trunc);
    for _ in 0..6 {
        qq_inv6 = arithmetic::mul(&qq_inv6, &qq_inv);
    }

    // (q^5;q^5)_inf^6
    let mut q5q5_6 = FormalPowerSeries::one(q, trunc);
    for _ in 0..6 {
        q5q5_6 = arithmetic::mul(&q5q5_6, &q5q5_inf);
    }

    // product * q^1
    let product = arithmetic::mul(&qq_inv6, &q5q5_6);
    let q_monomial = FormalPowerSeries::monomial(q, QRat::one(), 1, trunc);
    let expected = arithmetic::mul(&q_monomial, &product);

    // Compare first ~25 coefficients (well within truncation)
    for k in 0..25 {
        assert_eq!(
            from_eta.coeff(k),
            expected.coeff(k),
            "EtaExpression.to_series mismatch at q^{}: got {}, expected {}",
            k,
            from_eta.coeff(k),
            expected.coeff(k)
        );
    }
}

// ===========================================================================
// Test 11: from_etaquotient conversion
// ===========================================================================

#[test]
fn eta_expression_from_etaquotient() {
    let q = q_var();
    let trunc = 25;

    // Get the Euler function (q;q)_inf and run etamake
    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler_series = euler_gen.into_series();

    let eq = etamake(&euler_series, 20);

    // Convert to EtaExpression
    let eta_expr = EtaExpression::from_etaquotient(&eq);

    // Should have factors = {1: 1}
    assert_eq!(
        *eta_expr.factors.get(&1).unwrap_or(&0),
        1,
        "from_etaquotient: Euler function should have r_1 = 1"
    );

    // Level should be 1 (LCM of all deltas = LCM(1) = 1)
    assert_eq!(eta_expr.level, 1, "from_etaquotient: level should be 1");

    // Weight should be 1/2
    assert_eq!(
        eta_expr.weight(),
        qrat(1, 2),
        "from_etaquotient: weight should be 1/2"
    );
}

// ===========================================================================
// Test 12: JacExpression with scalar prefactor
// ===========================================================================

#[test]
fn jac_expression_with_scalar() {
    let q = q_var();
    let trunc = 20;

    // 2 * JAC(1,5)
    let expr = JacExpression::new(
        qrat(2, 1),
        QRat::zero(),
        vec![JacFactor::new(1, 5, 1)],
    );
    let from_expr = expr.to_series(q, trunc);

    let j15 = jacprod(1, 5, q, trunc);
    let from_direct = arithmetic::scalar_mul(&qrat(2, 1), &j15);

    assert_eq!(
        from_expr, from_direct,
        "2*JAC(1,5) expression should match direct scalar multiply"
    );
}

// ===========================================================================
// Test 13: EtaExpression weight and q_shift for multi-factor
// ===========================================================================

#[test]
fn eta_expression_multi_factor_properties() {
    // factors = {1: -6, 5: 6}, level = 5
    let eta = EtaExpression::from_factors(&[(1, -6), (5, 6)], 5);

    // weight = (-6 + 6)/2 = 0
    assert_eq!(eta.weight(), qrat(0, 1), "weight should be 0");

    // q_shift = (1*(-6) + 5*6)/24 = 24/24 = 1
    assert_eq!(eta.q_shift(), qrat(1, 1), "q_shift should be 1");
}

// ===========================================================================
// Test 14: EtaExpression empty construction
// ===========================================================================

#[test]
fn eta_expression_empty() {
    let eta = EtaExpression::new(BTreeMap::new(), 1);
    assert_eq!(eta.weight(), qrat(0, 1));
    assert_eq!(eta.q_shift(), qrat(0, 1));
    assert!(eta.factors.is_empty());
}

// ===========================================================================
// Test 15: JacExpression is_empty
// ===========================================================================

#[test]
fn jac_expression_is_empty() {
    let empty = JacExpression::new(QRat::one(), QRat::zero(), vec![]);
    assert!(empty.is_empty());

    let nonempty = JacExpression::single(1, 5);
    assert!(!nonempty.is_empty());
}
