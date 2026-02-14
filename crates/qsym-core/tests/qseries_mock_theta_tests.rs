//! Comprehensive tests for all 20 classical mock theta functions.
//!
//! Tests verify:
//! - Third-order (7): f, phi, psi, chi, omega, nu, rho against known coefficient sequences
//! - Fifth-order (10): f0, f1, F0, F1, phi0, phi1, psi0, psi1, chi0, chi1
//! - Seventh-order (3): F0, F1, F2
//! - Structural relations (chi0 = 2*F0 - phi0(-q), chi1 = 2*F1 + q^{-1}*phi1(-q))
//! - Truncation consistency
//! - All functions terminate and produce non-trivial results
//! - All coefficients are integers

use qsym_core::number::QRat;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;
use qsym_core::qseries::*;

/// Helper: create a SymbolId for "q".
fn q_var() -> SymbolId {
    let mut arena = ExprArena::new();
    arena.symbols_mut().intern("q")
}

/// Helper: create QRat from i64.
fn qrat(n: i64) -> QRat {
    QRat::from((n, 1i64))
}

/// Helper: verify FPS coefficients against expected values.
/// expected[i] is the expected coefficient of q^{start + i}.
fn verify_coefficients(
    fps: &qsym_core::series::FormalPowerSeries,
    start: i64,
    expected: &[i64],
    name: &str,
) {
    for (i, &e) in expected.iter().enumerate() {
        let power = start + i as i64;
        if power >= fps.truncation_order() {
            break;
        }
        let actual = fps.coeff(power);
        let expected_qrat = qrat(e);
        assert_eq!(
            actual, expected_qrat,
            "{}: coeff at q^{} expected {} but got {}",
            name, power, e, actual
        );
    }
}

// ===========================================================================
// Third-order mock theta function tests (7 tests)
// ===========================================================================

/// OEIS A000025: f(q) = sum q^{n^2}/(-q;q)_n^2
/// Verified: first 12 terms match OEIS A000025.
#[test]
fn test_mock_theta_f3_oeis() {
    let q = q_var();
    let fps = mock_theta_f3(q, 25);

    // Coefficients verified by term-by-term hand computation and cross-checked
    // against structural properties.
    let expected = [
        1, 1, -2, 3, -3, 3, -5, 7, -6, 6, -10, 12, -11,
        13, -17, 20, -21, 21, -27, 34, -33, 36, -46, 51, -53,
    ];
    verify_coefficients(&fps, 0, &expected, "f3");
}

/// OEIS A053250: phi(q) = sum q^{n^2}/(-q^2;q^2)_n
#[test]
fn test_mock_theta_phi3_oeis() {
    let q = q_var();
    let fps = mock_theta_phi3(q, 25);

    let expected = [
        1, 1, 0, -1, 1, 1, -1, -1, 0, 2, 0, -2, 1,
        1, -1, -2, 1, 3, -1, -2, 1, 2, -2, -3, 1,
    ];
    verify_coefficients(&fps, 0, &expected, "phi3");
}

/// OEIS A053251: psi(q) = sum_{n>=1} q^{n^2}/(q;q^2)_n
#[test]
fn test_mock_theta_psi3_oeis() {
    let q = q_var();
    let fps = mock_theta_psi3(q, 25);

    let expected = [
        0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 5, 5, 6,
        7, 8, 9, 11, 12, 13, 16, 17, 19, 22, 24, 27,
    ];
    verify_coefficients(&fps, 0, &expected, "psi3");
}

/// OEIS A053252: chi(q) = sum q^{n^2}/prod_{k=1}^n (1-q^k+q^{2k})
#[test]
fn test_mock_theta_chi3_oeis() {
    let q = q_var();
    let fps = mock_theta_chi3(q, 25);

    let expected = [
        1, 1, 1, 0, 0, 0, 1, 1, 0, 0, -1, 0, 1,
        1, 1, -1, 0, 0, 0, 1, 0, 0, -1, 0, 1,
    ];
    verify_coefficients(&fps, 0, &expected, "chi3");
}

/// OEIS A053253: omega(q) = sum q^{2n(n+1)}/(q;q^2)_{n+1}^2
#[test]
fn test_mock_theta_omega3_coefficients() {
    let q = q_var();
    let fps = mock_theta_omega3(q, 25);

    let expected = [
        1, 2, 3, 4, 6, 8, 10, 14, 18, 22, 29, 36, 44,
        56, 68, 82, 101, 122, 146, 176, 210, 248, 296, 350, 410,
    ];
    verify_coefficients(&fps, 0, &expected, "omega3");
}

/// OEIS A053254: nu(q) = sum q^{n(n+1)}/(-q;q^2)_{n+1}
#[test]
fn test_mock_theta_nu3_coefficients() {
    let q = q_var();
    let fps = mock_theta_nu3(q, 25);

    let expected = [
        1, -1, 2, -2, 2, -3, 4, -4, 5, -6, 6, -8, 10,
        -10, 12, -14, 15, -18, 20, -22, 26, -29, 32, -36, 40,
    ];
    verify_coefficients(&fps, 0, &expected, "nu3");
}

/// OEIS A053255: rho(q) = sum q^{2n(n+1)}/prod_{k=0}^n (1+q^{2k+1}+q^{4k+2})
#[test]
fn test_mock_theta_rho3_coefficients() {
    let q = q_var();
    let fps = mock_theta_rho3(q, 25);

    let expected = [
        1, -1, 0, 1, 0, -1, 1, -1, 0, 1, -1, 0, 2,
        -1, -1, 1, -1, -1, 2, -1, 0, 2, -1, -1, 2,
    ];
    verify_coefficients(&fps, 0, &expected, "rho3");
}

// ===========================================================================
// Fifth-order mock theta function tests (10 tests)
// ===========================================================================

/// OEIS A053256: f0(q) = sum q^{n^2}/(-q;q)_n
#[test]
fn test_mock_theta_f0_5_oeis() {
    let q = q_var();
    let fps = mock_theta_f0_5(q, 25);

    let expected = [
        1, 1, -1, 1, 0, 0, -1, 1, 0, 1, -2, 1, -1,
        2, -2, 2, -1, 1, -3, 2, -1, 3, -3, 2, -2,
    ];
    verify_coefficients(&fps, 0, &expected, "f0_5");
}

/// OEIS A053257: f1(q) = sum q^{n^2+n}/(-q;q)_n
#[test]
fn test_mock_theta_f1_5_oeis() {
    let q = q_var();
    let fps = mock_theta_f1_5(q, 25);

    let expected = [
        1, 0, 1, -1, 1, -1, 2, -2, 1, -1, 2, -2, 2,
        -2, 2, -3, 3, -2, 3, -4, 4, -4, 4, -5, 5,
    ];
    verify_coefficients(&fps, 0, &expected, "f1_5");
}

/// OEIS A053258: F0(q) = sum q^{2n^2}/(q;q^2)_n
#[test]
fn test_mock_theta_cap_f0_5_oeis() {
    let q = q_var();
    let fps = mock_theta_cap_f0_5(q, 25);

    let expected = [
        1, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 3, 3,
        3, 4, 4, 4, 5, 6, 6, 7, 8, 8, 10, 11,
    ];
    verify_coefficients(&fps, 0, &expected, "cap_f0_5");
}

/// OEIS A053259: F1(q) = sum q^{2n^2+2n}/(q;q^2)_{n+1}
#[test]
fn test_mock_theta_cap_f1_5_oeis() {
    let q = q_var();
    let fps = mock_theta_cap_f1_5(q, 25);

    let expected = [
        1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 5,
        6, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 18,
    ];
    verify_coefficients(&fps, 0, &expected, "cap_f1_5");
}

/// OEIS A053260: phi0(q) = sum (-q;q^2)_n * q^{n^2}
#[test]
fn test_mock_theta_phi0_5_oeis() {
    let q = q_var();
    let fps = mock_theta_phi0_5(q, 25);

    let expected = [
        1, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 1,
        1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 2,
    ];
    verify_coefficients(&fps, 0, &expected, "phi0_5");
}

/// phi1(q) = sum (-q;q^2)_n * q^{(n+1)^2}
#[test]
fn test_mock_theta_phi1_5_coefficients() {
    let q = q_var();
    let fps = mock_theta_phi1_5(q, 25);

    let expected = [
        0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1,
        1, 0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 1,
    ];
    verify_coefficients(&fps, 0, &expected, "phi1_5");
}

/// psi0(q) = sum (-1;q)_n * q^{n(n+1)/2}
#[test]
fn test_mock_theta_psi0_5_coefficients() {
    let q = q_var();
    let fps = mock_theta_psi0_5(q, 25);

    let expected = [
        1, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2,
        4, 2, 4, 4, 2, 4, 4, 4, 6, 6, 4, 6,
    ];
    verify_coefficients(&fps, 0, &expected, "psi0_5");
}

/// psi1(q) = sum (-q;q)_n * q^{n(n+1)/2}
#[test]
fn test_mock_theta_psi1_5_coefficients() {
    let q = q_var();
    let fps = mock_theta_psi1_5(q, 25);

    let expected = [
        1, 1, 1, 1, 1, 1, 2, 1, 1, 2, 2, 2, 2,
        2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 5,
    ];
    verify_coefficients(&fps, 0, &expected, "psi1_5");
}

/// chi0(q) = 2*F0(q) - phi0(-q)
#[test]
fn test_mock_theta_chi0_5_coefficients() {
    let q = q_var();
    let fps = mock_theta_chi0_5(q, 25);

    let expected = [
        1, 1, 1, 2, 1, 3, 2, 3, 3, 5, 3, 6, 5,
        7, 7, 9, 7, 12, 11, 13, 13, 17, 15, 21, 20,
    ];
    verify_coefficients(&fps, 0, &expected, "chi0_5");
}

/// chi1(q) = 2*F1(q) + q^{-1}*phi1(-q)
#[test]
fn test_mock_theta_chi1_5_coefficients() {
    let q = q_var();
    let fps = mock_theta_chi1_5(q, 25);

    // Note: chi1 has truncation_order = 24 (due to shift by -1 of phi1)
    let expected = [
        1, 2, 2, 3, 3, 4, 4, 6, 5, 7, 8, 9, 9,
        12, 12, 15, 15, 18, 19, 23, 23, 27, 30, 33,
    ];
    verify_coefficients(&fps, 0, &expected, "chi1_5");
}

// ===========================================================================
// Seventh-order mock theta function tests (3 tests)
// ===========================================================================

/// F0(q) = sum q^{n^2}/(q^{n+1};q)_n (seventh order)
#[test]
fn test_mock_theta_cap_f0_7_coefficients() {
    let q = q_var();
    let fps = mock_theta_cap_f0_7(q, 25);

    let expected = [
        1, 1, 0, 1, 1, 1, 0, 2, 1, 2, 1, 2, 1,
        3, 2, 3, 3, 3, 2, 5, 3, 5, 4, 6, 5,
    ];
    verify_coefficients(&fps, 0, &expected, "cap_f0_7");
}

/// F1(q) = sum q^{n^2}/(q^n;q)_n (seventh order)
#[test]
fn test_mock_theta_cap_f1_7_coefficients() {
    let q = q_var();
    let fps = mock_theta_cap_f1_7(q, 25);

    let expected = [
        1, 1, 1, 1, 2, 1, 2, 2, 2, 3, 3, 2, 4,
        4, 4, 4, 6, 5, 6, 6, 7, 8, 9, 8, 10,
    ];
    verify_coefficients(&fps, 0, &expected, "cap_f1_7");
}

/// F2(q) = sum q^{n^2+n}/(q^{n+1};q)_{n+1} (seventh order)
#[test]
fn test_mock_theta_cap_f2_7_coefficients() {
    let q = q_var();
    let fps = mock_theta_cap_f2_7(q, 25);

    let expected = [
        1, 1, 2, 1, 2, 2, 3, 2, 3, 3, 4, 4, 5,
        4, 6, 5, 7, 7, 8, 8, 10, 9, 11, 11, 13,
    ];
    verify_coefficients(&fps, 0, &expected, "cap_f2_7");
}

// ===========================================================================
// Structural relation tests
// ===========================================================================

/// Verify chi0(q) = 2*F0(q) - phi0(-q) by computing all three independently.
#[test]
fn test_mock_theta_chi0_5_relation() {
    let q = q_var();
    let trunc = 25;

    let chi0 = mock_theta_chi0_5(q, trunc);
    let f0 = mock_theta_cap_f0_5(q, trunc);
    let phi0 = mock_theta_phi0_5(q, trunc);

    // Negate variable (q -> -q)
    let mut phi0_neg = qsym_core::series::FormalPowerSeries::zero(q, trunc);
    for (&k, v) in phi0.iter() {
        if k % 2 == 0 {
            phi0_neg.set_coeff(k, v.clone());
        } else {
            phi0_neg.set_coeff(k, -v.clone());
        }
    }

    // 2*F0 - phi0(-q)
    let two_f0 = qsym_core::series::arithmetic::scalar_mul(&qrat(2), &f0);
    let expected = qsym_core::series::arithmetic::sub(&two_f0, &phi0_neg);

    for k in 0..trunc {
        assert_eq!(
            chi0.coeff(k),
            expected.coeff(k),
            "chi0 relation failed at q^{}: chi0={}, 2*F0-phi0(-q)={}",
            k, chi0.coeff(k), expected.coeff(k),
        );
    }
}

/// Verify chi1(q) = 2*F1(q) + q^{-1}*phi1(-q).
#[test]
fn test_mock_theta_chi1_5_relation() {
    let q = q_var();
    let trunc = 25;

    let chi1 = mock_theta_chi1_5(q, trunc);
    let f1 = mock_theta_cap_f1_5(q, trunc);
    let phi1 = mock_theta_phi1_5(q, trunc);

    // Negate variable (q -> -q)
    let mut phi1_neg = qsym_core::series::FormalPowerSeries::zero(q, trunc);
    for (&k, v) in phi1.iter() {
        if k % 2 == 0 {
            phi1_neg.set_coeff(k, v.clone());
        } else {
            phi1_neg.set_coeff(k, -v.clone());
        }
    }

    // q^{-1} * phi1(-q) = shift by -1
    let phi1_neg_shifted = qsym_core::series::arithmetic::shift(&phi1_neg, -1);

    // 2*F1 + q^{-1}*phi1(-q)
    let two_f1 = qsym_core::series::arithmetic::scalar_mul(&qrat(2), &f1);
    let expected = qsym_core::series::arithmetic::add(&two_f1, &phi1_neg_shifted);

    // Use min truncation order (shift reduces it by 1)
    let check_trunc = chi1.truncation_order().min(expected.truncation_order());
    for k in 0..check_trunc {
        assert_eq!(
            chi1.coeff(k),
            expected.coeff(k),
            "chi1 relation failed at q^{}: chi1={}, 2*F1+q^{{-1}}*phi1(-q)={}",
            k, chi1.coeff(k), expected.coeff(k),
        );
    }
}

// ===========================================================================
// Truncation consistency test
// ===========================================================================

/// Verify that computing f3 with truncation_order=20 and =50 gives the same first 20 coefficients.
#[test]
fn test_mock_theta_truncation_consistency() {
    let q = q_var();
    let fps_20 = mock_theta_f3(q, 20);
    let fps_50 = mock_theta_f3(q, 50);

    for k in 0..20 {
        assert_eq!(
            fps_20.coeff(k),
            fps_50.coeff(k),
            "Truncation consistency failed for f3 at q^{}",
            k,
        );
    }
}

// ===========================================================================
// Termination and non-trivial output test
// ===========================================================================

/// Call each of the 20 functions with truncation_order=30, verify they return
/// without hanging and produce non-trivial results.
#[test]
fn test_mock_theta_all_functions_terminate() {
    let q = q_var();
    let trunc = 30;

    let functions: Vec<(&str, qsym_core::series::FormalPowerSeries)> = vec![
        ("f3", mock_theta_f3(q, trunc)),
        ("phi3", mock_theta_phi3(q, trunc)),
        ("psi3", mock_theta_psi3(q, trunc)),
        ("chi3", mock_theta_chi3(q, trunc)),
        ("omega3", mock_theta_omega3(q, trunc)),
        ("nu3", mock_theta_nu3(q, trunc)),
        ("rho3", mock_theta_rho3(q, trunc)),
        ("f0_5", mock_theta_f0_5(q, trunc)),
        ("f1_5", mock_theta_f1_5(q, trunc)),
        ("cap_f0_5", mock_theta_cap_f0_5(q, trunc)),
        ("cap_f1_5", mock_theta_cap_f1_5(q, trunc)),
        ("phi0_5", mock_theta_phi0_5(q, trunc)),
        ("phi1_5", mock_theta_phi1_5(q, trunc)),
        ("psi0_5", mock_theta_psi0_5(q, trunc)),
        ("psi1_5", mock_theta_psi1_5(q, trunc)),
        ("chi0_5", mock_theta_chi0_5(q, trunc)),
        ("chi1_5", mock_theta_chi1_5(q, trunc)),
        ("cap_f0_7", mock_theta_cap_f0_7(q, trunc)),
        ("cap_f1_7", mock_theta_cap_f1_7(q, trunc)),
        ("cap_f2_7", mock_theta_cap_f2_7(q, trunc)),
    ];

    for (name, fps) in &functions {
        assert!(
            fps.num_nonzero() > 5,
            "{} should have many nonzero coefficients (got {})",
            name,
            fps.num_nonzero()
        );
    }
}

/// Verify all mock theta functions produce integer coefficients (QRat with denominator 1).
#[test]
fn test_mock_theta_integer_coefficients() {
    let q = q_var();
    let trunc = 20;

    let functions: Vec<(&str, qsym_core::series::FormalPowerSeries)> = vec![
        ("f3", mock_theta_f3(q, trunc)),
        ("phi3", mock_theta_phi3(q, trunc)),
        ("psi3", mock_theta_psi3(q, trunc)),
        ("chi3", mock_theta_chi3(q, trunc)),
        ("omega3", mock_theta_omega3(q, trunc)),
        ("nu3", mock_theta_nu3(q, trunc)),
        ("rho3", mock_theta_rho3(q, trunc)),
        ("f0_5", mock_theta_f0_5(q, trunc)),
        ("f1_5", mock_theta_f1_5(q, trunc)),
        ("cap_f0_5", mock_theta_cap_f0_5(q, trunc)),
        ("cap_f1_5", mock_theta_cap_f1_5(q, trunc)),
        ("phi0_5", mock_theta_phi0_5(q, trunc)),
        ("phi1_5", mock_theta_phi1_5(q, trunc)),
        ("psi0_5", mock_theta_psi0_5(q, trunc)),
        ("psi1_5", mock_theta_psi1_5(q, trunc)),
        ("chi0_5", mock_theta_chi0_5(q, trunc)),
        ("chi1_5", mock_theta_chi1_5(q, trunc)),
        ("cap_f0_7", mock_theta_cap_f0_7(q, trunc)),
        ("cap_f1_7", mock_theta_cap_f1_7(q, trunc)),
        ("cap_f2_7", mock_theta_cap_f2_7(q, trunc)),
    ];

    for (name, fps) in &functions {
        for (&k, v) in fps.iter() {
            assert!(
                v.denom() == &rug::Integer::from(1),
                "{}: coefficient at q^{} is {}, expected integer",
                name, k, v
            );
        }
    }
}
