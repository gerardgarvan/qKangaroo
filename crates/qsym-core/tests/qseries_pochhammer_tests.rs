//! Comprehensive tests for q-Pochhammer symbol (aqprod) and q-binomial coefficient (qbin).
//!
//! Tests verify:
//! - aqprod edge cases: order zero, a=1 vanishing, zero coefficient
//! - aqprod finite positive: known product expansions
//! - aqprod finite negative: inversion identities
//! - aqprod infinite: matches Euler function generator
//! - aqprod infinite with negative coefficient: distinct parts generating function
//! - qbin: Gaussian polynomial coefficients for small parameters
//! - qbin: edge cases, symmetry

use qsym_core::number::QRat;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;
use qsym_core::series::generator::euler_function_generator;
use qsym_core::qseries::{QMonomial, PochhammerOrder, aqprod, qbin};

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
// 1. aqprod tests
// ===========================================================================

/// aqprod(any, q, Finite(0)) = 1 for any a.
#[test]
fn aqprod_order_zero() {
    let q = q_var();
    let a = QMonomial::q_power(3); // arbitrary

    let fps = aqprod(&a, q, PochhammerOrder::Finite(0), 20);

    assert_eq!(fps.coeff(0), qrat(1), "constant term should be 1");
    for k in 1..20 {
        assert_eq!(fps.coeff(k), QRat::zero(), "coeff({}) should be 0", k);
    }
}

/// aqprod(1*q^0, q, Finite(5)) = 0 since first factor is (1-1)=0.
#[test]
fn aqprod_a_is_one_finite() {
    let q = q_var();
    let a = QMonomial::q_power(0); // a = 1*q^0 = 1

    let fps = aqprod(&a, q, PochhammerOrder::Finite(5), 20);

    assert!(fps.is_zero(), "product should be zero since (1-1) factor vanishes");
}

/// aqprod(1*q^1, q, Finite(2)) = (1-q)(1-q^2) = 1 - q - q^2 + q^3.
#[test]
fn aqprod_q_order_2() {
    let q = q_var();
    let a = QMonomial::q_power(1); // a = q

    let fps = aqprod(&a, q, PochhammerOrder::Finite(2), 20);

    // (1-q)(1-q^2) = 1 - q - q^2 + q^3
    assert_eq!(fps.coeff(0), qrat(1), "coeff(0) = 1");
    assert_eq!(fps.coeff(1), qrat(-1), "coeff(1) = -1");
    assert_eq!(fps.coeff(2), qrat(-1), "coeff(2) = -1");
    assert_eq!(fps.coeff(3), qrat(1), "coeff(3) = 1");
    for k in 4..20 {
        assert_eq!(fps.coeff(k), QRat::zero(), "coeff({}) should be 0", k);
    }
}

/// aqprod(1*q^2, q, Finite(3)) = (1-q^2)(1-q^3)(1-q^4).
/// Expansion: 1 - q^2 - q^3 - q^4 + q^5 + q^6 + q^7 - q^9
#[test]
fn aqprod_q_squared_order_3() {
    let q = q_var();
    let a = QMonomial::q_power(2); // a = q^2

    let fps = aqprod(&a, q, PochhammerOrder::Finite(3), 20);

    // (1-q^2)(1-q^3)(1-q^4)
    // = (1 - q^2 - q^3 + q^5)(1 - q^4)
    // = 1 - q^2 - q^3 + q^5 - q^4 + q^6 + q^7 - q^9
    // = 1 - q^2 - q^3 - q^4 + q^5 + q^6 + q^7 - q^9
    assert_eq!(fps.coeff(0), qrat(1), "coeff(0) = 1");
    assert_eq!(fps.coeff(1), QRat::zero(), "coeff(1) = 0");
    assert_eq!(fps.coeff(2), qrat(-1), "coeff(2) = -1");
    assert_eq!(fps.coeff(3), qrat(-1), "coeff(3) = -1");
    assert_eq!(fps.coeff(4), qrat(-1), "coeff(4) = -1");
    assert_eq!(fps.coeff(5), qrat(1), "coeff(5) = 1");
    assert_eq!(fps.coeff(6), qrat(1), "coeff(6) = 1");
    assert_eq!(fps.coeff(7), qrat(1), "coeff(7) = 1");
    assert_eq!(fps.coeff(8), QRat::zero(), "coeff(8) = 0");
    assert_eq!(fps.coeff(9), qrat(-1), "coeff(9) = -1");
    for k in 10..20 {
        assert_eq!(fps.coeff(k), QRat::zero(), "coeff({}) should be 0", k);
    }
}

/// aqprod(0*q^0, q, Finite(5)) = 1 since all factors are (1-0)=1.
#[test]
fn aqprod_zero_coeff() {
    let q = q_var();
    let a = QMonomial::new(QRat::zero(), 0); // a = 0

    let fps = aqprod(&a, q, PochhammerOrder::Finite(5), 20);

    assert_eq!(fps.coeff(0), qrat(1), "constant term should be 1");
    for k in 1..20 {
        assert_eq!(fps.coeff(k), QRat::zero(), "coeff({}) should be 0", k);
    }
}

/// aqprod(1*q^2, q, Finite(-1)) = 1/(1-q).
/// (q^2;q)_{-1} = 1/(q^2*q^{-1};q)_1 = 1/(q;q)_1 = 1/(1-q)
/// 1/(1-q) = 1 + q + q^2 + q^3 + ...
#[test]
fn aqprod_negative_order_minus_1() {
    let q = q_var();
    let a = QMonomial::q_power(2); // a = q^2
    let trunc = 20;

    let fps = aqprod(&a, q, PochhammerOrder::Finite(-1), trunc);

    // 1/(1-q) = sum_{k>=0} q^k (geometric series)
    for k in 0..trunc {
        assert_eq!(
            fps.coeff(k), qrat(1),
            "1/(1-q): coeff({}) should be 1", k
        );
    }
}

/// aqprod(1*q^3, q, Finite(-2)) = 1/(q;q)_2 = 1/[(1-q)(1-q^2)]
/// = 1/(1 - q - q^2 + q^3)
/// Coefficients: 1, 1, 2, 2, 3, 3, 4, 4, ...
#[test]
fn aqprod_negative_order_minus_2() {
    let q = q_var();
    let a = QMonomial::q_power(3); // a = q^3
    let trunc = 20;

    let fps = aqprod(&a, q, PochhammerOrder::Finite(-2), trunc);

    // 1/[(1-q)(1-q^2)] -- this is the generating function for partitions
    // into parts of size at most 2. Coefficients: floor(k/2)+1.
    // c[0]=1, c[1]=1, c[2]=2, c[3]=2, c[4]=3, c[5]=3, c[6]=4, c[7]=4, ...
    let expected: Vec<i64> = vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10];

    for (k, &val) in expected.iter().enumerate() {
        assert_eq!(
            fps.coeff(k as i64), qrat(val),
            "1/[(1-q)(1-q^2)]: coeff({}) should be {}", k, val
        );
    }
}

/// aqprod(q, q, Infinite, 30) should match euler_function_generator (which is (q;q)_inf).
#[test]
fn aqprod_infinite_matches_euler() {
    let q = q_var();
    let trunc = 30;

    let fps = aqprod(&QMonomial::q_power(1), q, PochhammerOrder::Infinite, trunc);

    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler = euler_gen.series();

    for k in 0..trunc {
        assert_eq!(
            fps.coeff(k), euler.coeff(k),
            "aqprod(q, q, inf) vs euler: mismatch at q^{}", k
        );
    }
}

/// aqprod(-q, q, Infinite) = prod_{k>=0}(1+q^{k+1}) = prod_{n>=1}(1+q^n)
/// This is the distinct parts generating function (OEIS A000009):
/// 1, 1, 1, 2, 2, 3, 4, 5, 6, 8, 10, ...
#[test]
fn aqprod_infinite_minus_q() {
    let q = q_var();
    let trunc = 30;

    // a = -1 * q^1 means factor k is (1 - (-1)*q^{1+k}) = (1 + q^{k+1})
    let a = QMonomial::new(-QRat::one(), 1);
    let fps = aqprod(&a, q, PochhammerOrder::Infinite, trunc);

    // OEIS A000009: number of partitions into distinct parts
    let expected: Vec<i64> = vec![
        1, 1, 1, 2, 2, 3, 4, 5, 6, 8,
        10, 12, 15, 18, 22, 27, 32, 38, 46, 54,
        64, 76, 89, 104, 122, 142, 165, 192, 222, 256,
    ];

    for (k, &val) in expected.iter().enumerate() {
        assert_eq!(
            fps.coeff(k as i64), qrat(val),
            "distinct parts GF: coeff({}) should be {}", k, val
        );
    }
}

// ===========================================================================
// 2. qbin tests
// ===========================================================================

/// [4 choose 2]_q = 1 + q + 2q^2 + q^3 + q^4
#[test]
fn qbin_4_2() {
    let q = q_var();
    let fps = qbin(4, 2, q, 20);

    assert_eq!(fps.coeff(0), qrat(1), "[4,2]_q: coeff(0) = 1");
    assert_eq!(fps.coeff(1), qrat(1), "[4,2]_q: coeff(1) = 1");
    assert_eq!(fps.coeff(2), qrat(2), "[4,2]_q: coeff(2) = 2");
    assert_eq!(fps.coeff(3), qrat(1), "[4,2]_q: coeff(3) = 1");
    assert_eq!(fps.coeff(4), qrat(1), "[4,2]_q: coeff(4) = 1");
    for k in 5..20 {
        assert_eq!(fps.coeff(k), QRat::zero(), "[4,2]_q: coeff({}) should be 0", k);
    }
}

/// [5 choose 2]_q = 1 + q + 2q^2 + 2q^3 + 2q^4 + q^5 + q^6
#[test]
fn qbin_5_2() {
    let q = q_var();
    let fps = qbin(5, 2, q, 20);

    assert_eq!(fps.coeff(0), qrat(1), "[5,2]_q: coeff(0) = 1");
    assert_eq!(fps.coeff(1), qrat(1), "[5,2]_q: coeff(1) = 1");
    assert_eq!(fps.coeff(2), qrat(2), "[5,2]_q: coeff(2) = 2");
    assert_eq!(fps.coeff(3), qrat(2), "[5,2]_q: coeff(3) = 2");
    assert_eq!(fps.coeff(4), qrat(2), "[5,2]_q: coeff(4) = 2");
    assert_eq!(fps.coeff(5), qrat(1), "[5,2]_q: coeff(5) = 1");
    assert_eq!(fps.coeff(6), qrat(1), "[5,2]_q: coeff(6) = 1");
    for k in 7..20 {
        assert_eq!(fps.coeff(k), QRat::zero(), "[5,2]_q: coeff({}) should be 0", k);
    }
}

/// [5 choose 0]_q = 1
#[test]
fn qbin_n_0() {
    let q = q_var();
    let fps = qbin(5, 0, q, 20);

    assert_eq!(fps.coeff(0), qrat(1), "[5,0]_q: coeff(0) = 1");
    for k in 1..20 {
        assert_eq!(fps.coeff(k), QRat::zero(), "[5,0]_q: coeff({}) should be 0", k);
    }
}

/// [5 choose 5]_q = 1
#[test]
fn qbin_n_n() {
    let q = q_var();
    let fps = qbin(5, 5, q, 20);

    assert_eq!(fps.coeff(0), qrat(1), "[5,5]_q: coeff(0) = 1");
    for k in 1..20 {
        assert_eq!(fps.coeff(k), QRat::zero(), "[5,5]_q: coeff({}) should be 0", k);
    }
}

/// [6 choose 2]_q == [6 choose 4]_q (symmetry property)
#[test]
fn qbin_symmetry() {
    let q = q_var();
    let trunc = 20;

    let fps_6_2 = qbin(6, 2, q, trunc);
    let fps_6_4 = qbin(6, 4, q, trunc);

    for k in 0..trunc {
        assert_eq!(
            fps_6_2.coeff(k), fps_6_4.coeff(k),
            "[6,2]_q vs [6,4]_q: mismatch at q^{}", k
        );
    }
}

/// [5 choose 1]_q = 1 + q + q^2 + q^3 + q^4
#[test]
fn qbin_n_1() {
    let q = q_var();
    let fps = qbin(5, 1, q, 20);

    // [n choose 1]_q = 1 + q + q^2 + ... + q^{n-1}
    for k in 0..5 {
        assert_eq!(fps.coeff(k), qrat(1), "[5,1]_q: coeff({}) = 1", k);
    }
    for k in 5..20 {
        assert_eq!(fps.coeff(k), QRat::zero(), "[5,1]_q: coeff({}) should be 0", k);
    }
}
