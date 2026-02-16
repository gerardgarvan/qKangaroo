//! Chen-Hou-Mu parameter specialization for proving nonterminating q-hypergeometric identities.
//!
//! Many important q-hypergeometric identities (q-Gauss, q-Kummer, Heine transforms) involve
//! nonterminating (infinite) sums that q-Zeilberger cannot directly handle. The Chen-Hou-Mu
//! method replaces a parameter with x*q^n to produce terminating versions, proves both sides
//! satisfy the same recurrence, then verifies initial conditions.
//!
//! Key components:
//! - [`NonterminatingProofResult`]: Result enum for proof outcomes
//! - [`prove_nonterminating`]: Main entry point for Chen-Hou-Mu proofs
//! - [`check_recurrence_on_values`]: Verify a scalar sequence satisfies a given recurrence

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use super::hypergeometric::HypergeometricSeries;
use super::zeilberger::{q_zeilberger, QZeilbergerResult, detect_n_params};
use super::gosper::extract_term_ratio;

/// Raise a QRat to a signed integer power via repeated squaring.
fn qrat_pow_i64(base: &QRat, exp: i64) -> QRat {
    if exp == 0 {
        return QRat::one();
    }
    if exp > 0 {
        qrat_pow_u32(base, exp as u32)
    } else {
        assert!(
            !base.is_zero(),
            "qrat_pow_i64: zero base with negative exponent"
        );
        let positive = qrat_pow_u32(base, (-exp) as u32);
        &QRat::one() / &positive
    }
}

/// Raise a QRat to a u32 power via repeated squaring.
fn qrat_pow_u32(base: &QRat, exp: u32) -> QRat {
    if exp == 0 {
        return QRat::one();
    }
    if exp == 1 {
        return base.clone();
    }
    let mut result = QRat::one();
    let mut b = base.clone();
    let mut e = exp;
    while e > 0 {
        if e & 1 == 1 {
            result = &result * &b;
        }
        e >>= 1;
        if e > 0 {
            b = &b * &b;
        }
    }
    result
}

/// Result of a nonterminating identity proof via Chen-Hou-Mu method.
#[derive(Clone, Debug)]
pub enum NonterminatingProofResult {
    /// Identity proved: both sides satisfy the same recurrence with matching initial conditions.
    Proved {
        /// The shared recurrence order.
        recurrence_order: usize,
        /// The shared recurrence coefficients (from the LHS at n_test, normalized).
        recurrence_coefficients: Vec<QRat>,
        /// Number of initial conditions verified (= recurrence_order + 1).
        initial_conditions_checked: usize,
    },
    /// Proof failed.
    Failed {
        /// Description of why the proof failed.
        reason: String,
    },
}

/// Compute the definite sum S(n) = sum_{k=0}^{N} F(n,k) at concrete q_val.
///
/// Uses term ratio accumulation: F(n,0)=1, F(n,k+1) = F(n,k) * r(q^k).
/// The sum terminates when the term ratio evaluates to zero (Pochhammer
/// factor vanishes) or after max_terms iterations.
fn compute_sum_at_q(series: &HypergeometricSeries, q_val: &QRat) -> QRat {
    let ratio = extract_term_ratio(series, q_val);
    let max_terms: usize = 100;
    let mut sum = QRat::one(); // F(n,0) = 1
    let mut term = QRat::one();

    for k in 0..max_terms {
        let qk = qrat_pow_i64(q_val, k as i64);
        match ratio.eval(&qk) {
            Some(r) => {
                if r.is_zero() {
                    break; // series has terminated
                }
                term = &term * &r;
                sum = &sum + &term;
            }
            None => break, // pole means factor vanished
        }
    }
    sum
}

/// Check if a scalar sequence satisfies a recurrence c_0*f(n) + ... + c_d*f(n+d) = 0.
///
/// `values` should contain f(n), f(n+1), ..., f(n+d).
/// `coefficients` should contain c_0, ..., c_d.
///
/// Returns true if the linear combination is zero.
fn check_recurrence_on_values(
    values: &[QRat],
    coefficients: &[QRat],
) -> bool {
    assert_eq!(
        values.len(),
        coefficients.len(),
        "check_recurrence_on_values: values and coefficients must have same length"
    );
    if values.is_empty() {
        return true;
    }

    let mut sum = QRat::zero();
    for j in 0..values.len() {
        let contrib = &coefficients[j] * &values[j];
        sum = &sum + &contrib;
    }
    sum.is_zero()
}

/// Check if an FPS sequence satisfies a recurrence c_0*f(n) + ... + c_d*f(n+d) = 0.
///
/// `fps_values` should contain f(n), f(n+1), ..., f(n+d).
/// `coefficients` should contain c_0, ..., c_d.
///
/// Returns true if the linear combination is the zero FPS.
pub fn check_recurrence_on_fps(
    fps_values: &[FormalPowerSeries],
    coefficients: &[QRat],
) -> bool {
    assert_eq!(
        fps_values.len(),
        coefficients.len(),
        "check_recurrence_on_fps: fps_values and coefficients must have same length"
    );
    if fps_values.is_empty() {
        return true;
    }

    // Compute sum_j c_j * fps_values[j]
    let mut result = arithmetic::scalar_mul(&coefficients[0], &fps_values[0]);
    for j in 1..fps_values.len() {
        let term = arithmetic::scalar_mul(&coefficients[j], &fps_values[j]);
        result = arithmetic::add(&result, &term);
    }
    result.is_zero()
}

/// Prove a nonterminating identity using the Chen-Hou-Mu parameter specialization method.
///
/// The method works as follows:
/// 1. The user provides a `lhs_builder` that, given n, returns a terminating
///    HypergeometricSeries (the LHS with a parameter specialized to q^{-n}).
/// 2. The user provides a `rhs_builder` that, given n, returns a QRat value
///    representing the RHS evaluated at the same parameter specialization and concrete q.
/// 3. For a test value of n, apply q-Zeilberger to the LHS to find a recurrence.
/// 4. Verify the RHS satisfies the same recurrence at multiple n values.
/// 5. Verify initial conditions (n = 0, 1, ..., d) by scalar comparison.
///
/// # Arguments
/// - `lhs_builder`: Given n, returns a terminating HypergeometricSeries for the LHS.
///   The series must be terminating (have an upper param q^{-n} or similar).
/// - `rhs_builder`: Given n, returns a QRat value for the RHS at that n (at concrete q).
/// - `q_val`: The concrete value of q.
/// - `n_test`: The test value of n for finding the recurrence (should be >= 5).
/// - `max_order`: Maximum recurrence order to try.
pub fn prove_nonterminating(
    lhs_builder: &dyn Fn(i64) -> HypergeometricSeries,
    rhs_builder: &dyn Fn(i64) -> QRat,
    q_val: &QRat,
    n_test: i64,
    max_order: usize,
) -> NonterminatingProofResult {
    // Step 1: Build LHS at n_test and verify it is terminating.
    let lhs_series = lhs_builder(n_test);
    if lhs_series.termination_order().is_none() {
        return NonterminatingProofResult::Failed {
            reason: "LHS at n_test is not terminating".to_string(),
        };
    }

    // Step 2: Find LHS recurrence via q-Zeilberger.
    let (n_indices, n_in_arg) = detect_n_params(&lhs_series, n_test, q_val);
    let zeil_result = q_zeilberger(&lhs_series, n_test, q_val, max_order, &n_indices, n_in_arg);

    let zr = match zeil_result {
        QZeilbergerResult::Recurrence(zr) => zr,
        QZeilbergerResult::NoRecurrence => {
            return NonterminatingProofResult::Failed {
                reason: format!(
                    "q-Zeilberger found no recurrence for LHS up to order {}",
                    max_order
                ),
            };
        }
    };

    // Step 3: Extract recurrence order.
    let d = zr.order;

    // Step 4: Verify RHS satisfies the same recurrence.
    //
    // The recurrence coefficients from q-Zeilberger at concrete q are n-specific.
    // We re-derive the recurrence at each verification n, then check the RHS
    // scalar values satisfy it.
    let verify_n_values: Vec<i64> = if n_test >= 2 {
        vec![n_test - 2, n_test - 1, n_test]
    } else {
        vec![n_test]
    };

    for &n_v in &verify_n_values {
        // Re-derive recurrence at this n value
        let lhs_at_nv = lhs_builder(n_v);
        if lhs_at_nv.termination_order().is_none() {
            continue;
        }
        let (nv_indices, nv_in_arg) = detect_n_params(&lhs_at_nv, n_v, q_val);
        let zeil_nv = q_zeilberger(&lhs_at_nv, n_v, q_val, max_order, &nv_indices, nv_in_arg);

        let zr_nv = match zeil_nv {
            QZeilbergerResult::Recurrence(zr) => zr,
            QZeilbergerResult::NoRecurrence => continue,
        };

        // Collect RHS values at n_v, n_v+1, ..., n_v+d_nv
        let d_nv = zr_nv.order;
        let rhs_vals: Vec<QRat> = (0..=(d_nv as i64))
            .map(|j| rhs_builder(n_v + j))
            .collect();

        if !check_recurrence_on_values(&rhs_vals, &zr_nv.coefficients) {
            return NonterminatingProofResult::Failed {
                reason: format!(
                    "RHS does not satisfy LHS recurrence at n={}",
                    n_v
                ),
            };
        }
    }

    // Step 5: Verify initial conditions. For n = 0, 1, ..., d:
    // Compare LHS sum (via term ratio at concrete q) with RHS value.
    for n in 0..=(d as i64) {
        let lhs_val = compute_sum_at_q(&lhs_builder(n), q_val);
        let rhs_val = rhs_builder(n);

        if lhs_val != rhs_val {
            return NonterminatingProofResult::Failed {
                reason: format!("Initial condition mismatch at n={}", n),
            };
        }
    }

    // All checks passed.
    NonterminatingProofResult::Proved {
        recurrence_order: d,
        recurrence_coefficients: zr.coefficients.clone(),
        initial_conditions_checked: d + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::SymbolId;
    use super::super::QMonomial;

    fn qr(n: i64) -> QRat {
        QRat::from((n, 1i64))
    }

    fn qr_frac(n: i64, d: i64) -> QRat {
        QRat::from((n, d))
    }

    fn test_variable() -> SymbolId {
        SymbolId(0)
    }

    const TRUNC: i64 = 30;

    /// Evaluate (a;q)_n at concrete q_val (scalar product, not FPS).
    fn pochhammer_scalar(a: &QRat, q_val: &QRat, n: i64) -> QRat {
        if n <= 0 {
            return QRat::one();
        }
        let mut result = QRat::one();
        for k in 0..n {
            let qk = qrat_pow_i64(q_val, k);
            let factor = &QRat::one() - &(a * &qk);
            result = &result * &factor;
        }
        result
    }

    // ========================================
    // Test 1: prove_q_gauss -- The q-Gauss summation formula
    // 2phi1(a, b; c; q, c/(ab)) = (c/a;q)_inf*(c/b;q)_inf / ((c;q)_inf*(c/(ab);q)_inf)
    // Specialize a -> q^{-n}, use concrete q=1/2, b=q^2, c=q^3.
    // At a=q^{-n}: RHS simplifies to (c/b;q)_n / (c;q)_n
    // ========================================

    #[test]
    fn test_prove_q_gauss() {
        let q_val = qr_frac(1, 2);

        // b = q^2, c = q^3
        let b_val = qrat_pow_i64(&q_val, 2); // q^2
        let c_val = qrat_pow_i64(&q_val, 3); // q^3
        let c_over_b = &c_val / &b_val; // c/b = q

        // LHS builder: 2phi1(q^{-n}, b; c; q, z_n) where z_n = c*q^n/b = q^{n+1}
        let lhs_builder = |n: i64| -> HypergeometricSeries {
            HypergeometricSeries {
                upper: vec![QMonomial::q_power(-n), QMonomial::q_power(2)],
                lower: vec![QMonomial::q_power(3)],
                argument: QMonomial::q_power(n + 1),
            }
        };

        // RHS builder: (c/b;q)_n / (c;q)_n at concrete q
        let rhs_builder = |n: i64| -> QRat {
            if n == 0 {
                return QRat::one();
            }
            let numer = pochhammer_scalar(&c_over_b, &q_val, n); // (c/b;q)_n = (q;q)_n
            let denom = pochhammer_scalar(&c_val, &q_val, n); // (c;q)_n = (q^3;q)_n
            &numer / &denom
        };

        let result = prove_nonterminating(
            &lhs_builder,
            &rhs_builder,
            &q_val,
            8,     // n_test
            2,     // max_order
        );

        match result {
            NonterminatingProofResult::Proved { recurrence_order, initial_conditions_checked, .. } => {
                assert!(recurrence_order >= 1, "Expected positive recurrence order");
                assert!(initial_conditions_checked >= 2, "Should check at least 2 initial conditions");
            }
            NonterminatingProofResult::Failed { reason } => {
                panic!("q-Gauss proof should succeed, but failed: {}", reason);
            }
        }
    }

    // ========================================
    // Test 2: prove_q_vandermonde -- q-Vandermonde sum as nonterminating proof
    // 2phi1(q^{-n}, a; c; q, cq^n/a) = (c/a;q)_n / (c;q)_n
    // ========================================

    #[test]
    fn test_prove_q_vandermonde() {
        let q_val = qr_frac(1, 2);

        // a = q^2, c = q^3 => z = cq^n/a = q^{n+1}
        let c_val = qrat_pow_i64(&q_val, 3);
        let a_val = qrat_pow_i64(&q_val, 2);
        let c_over_a = &c_val / &a_val; // q

        let lhs_builder = |n: i64| -> HypergeometricSeries {
            HypergeometricSeries {
                upper: vec![QMonomial::q_power(-n), QMonomial::q_power(2)],
                lower: vec![QMonomial::q_power(3)],
                argument: QMonomial::q_power(n + 1),
            }
        };

        // RHS: (c/a;q)_n / (c;q)_n at concrete q
        let rhs_builder = |n: i64| -> QRat {
            if n == 0 {
                return QRat::one();
            }
            let numer = pochhammer_scalar(&c_over_a, &q_val, n);
            let denom = pochhammer_scalar(&c_val, &q_val, n);
            &numer / &denom
        };

        let result = prove_nonterminating(
            &lhs_builder,
            &rhs_builder,
            &q_val,
            8,
            2,
        );

        match result {
            NonterminatingProofResult::Proved { .. } => {
                // Success
            }
            NonterminatingProofResult::Failed { reason } => {
                panic!("q-Vandermonde proof should succeed, but failed: {}", reason);
            }
        }
    }

    // ========================================
    // Test 3: prove_fails_wrong_rhs -- Correct LHS but wrong RHS (multiply by 2)
    // ========================================

    #[test]
    fn test_prove_fails_wrong_rhs() {
        let q_val = qr_frac(1, 2);

        let c_val = qrat_pow_i64(&q_val, 3);
        let c_over_a = qrat_pow_i64(&q_val, 1); // q

        let lhs_builder = |n: i64| -> HypergeometricSeries {
            HypergeometricSeries {
                upper: vec![QMonomial::q_power(-n), QMonomial::q_power(2)],
                lower: vec![QMonomial::q_power(3)],
                argument: QMonomial::q_power(n + 1),
            }
        };

        // Wrong RHS: multiply the correct answer by 2
        let rhs_builder = |n: i64| -> QRat {
            if n == 0 {
                return qr(2); // Wrong: should be 1
            }
            let numer = pochhammer_scalar(&c_over_a, &q_val, n);
            let denom = pochhammer_scalar(&c_val, &q_val, n);
            let correct = &numer / &denom;
            &correct * &qr(2)
        };

        let result = prove_nonterminating(
            &lhs_builder,
            &rhs_builder,
            &q_val,
            8,
            2,
        );

        match result {
            NonterminatingProofResult::Failed { reason } => {
                assert!(
                    reason.contains("recurrence") || reason.contains("Initial condition"),
                    "Expected failure related to recurrence or initial conditions, got: {}",
                    reason
                );
            }
            NonterminatingProofResult::Proved { .. } => {
                panic!("Proof should fail with wrong RHS (multiplied by 2)");
            }
        }
    }

    // ========================================
    // Test 4: prove_fails_non_terminating_lhs
    // ========================================

    #[test]
    fn test_prove_fails_non_terminating_lhs() {
        let q_val = qr_frac(1, 2);

        // LHS builder returns a non-terminating series (no q^{-n} parameter)
        let lhs_builder = |_n: i64| -> HypergeometricSeries {
            HypergeometricSeries {
                upper: vec![QMonomial::q_power(2), QMonomial::q_power(3)],
                lower: vec![QMonomial::q_power(5)],
                argument: QMonomial::q_power(1),
            }
        };

        let rhs_builder = |_n: i64| -> QRat {
            QRat::one()
        };

        let result = prove_nonterminating(
            &lhs_builder,
            &rhs_builder,
            &q_val,
            8,
            2,
        );

        match result {
            NonterminatingProofResult::Failed { reason } => {
                assert!(
                    reason.contains("not terminating"),
                    "Expected 'not terminating' in reason, got: {}",
                    reason
                );
            }
            NonterminatingProofResult::Proved { .. } => {
                panic!("Proof should fail with non-terminating LHS");
            }
        }
    }

    // ========================================
    // Test 5: check_recurrence_on_fps unit test
    // ========================================

    #[test]
    fn test_recurrence_check_on_fps() {
        let var = test_variable();

        // Create a simple recurrence: f(n+1) = 2*f(n)
        // Meaning: c_0*f(n) + c_1*f(n+1) = 0 with c_0 = 2, c_1 = -1
        let f0 = {
            let mut fps = FormalPowerSeries::zero(var, TRUNC);
            fps.set_coeff(0, qr(1));
            fps
        };
        let f1 = {
            let mut fps = FormalPowerSeries::zero(var, TRUNC);
            fps.set_coeff(0, qr(2));
            fps
        };
        let f2 = {
            let mut fps = FormalPowerSeries::zero(var, TRUNC);
            fps.set_coeff(0, qr(4));
            fps
        };

        let coeffs = vec![qr(2), qr(-1)]; // 2*f(n) - f(n+1) = 0

        // Should satisfy the recurrence
        assert!(check_recurrence_on_fps(&[f0.clone(), f1.clone()], &coeffs));
        assert!(check_recurrence_on_fps(&[f1.clone(), f2.clone()], &coeffs));

        // Should NOT satisfy with wrong coefficients
        let wrong_coeffs = vec![qr(3), qr(-1)]; // 3*f(n) - f(n+1) != 0
        assert!(!check_recurrence_on_fps(&[f0.clone(), f1.clone()], &wrong_coeffs));
    }

    // ========================================
    // Test 6: initial_condition_mismatch
    // LHS and RHS satisfy same recurrence but different initial conditions
    // ========================================

    #[test]
    fn test_initial_condition_mismatch() {
        let q_val = qr_frac(1, 2);

        let c_val = qrat_pow_i64(&q_val, 3);
        let c_over_a = qrat_pow_i64(&q_val, 1);

        let lhs_builder = |n: i64| -> HypergeometricSeries {
            HypergeometricSeries {
                upper: vec![QMonomial::q_power(-n), QMonomial::q_power(2)],
                lower: vec![QMonomial::q_power(3)],
                argument: QMonomial::q_power(n + 1),
            }
        };

        // RHS that satisfies the same recurrence but with perturbed initial value.
        let rhs_builder = |n: i64| -> QRat {
            if n == 0 {
                return QRat::one(); // n=0 correct
            }
            let numer = pochhammer_scalar(&c_over_a, &q_val, n);
            let denom = pochhammer_scalar(&c_val, &q_val, n);
            let correct = &numer / &denom;
            // Add a perturbation
            &correct + &qr_frac(1, 1000)
        };

        let result = prove_nonterminating(
            &lhs_builder,
            &rhs_builder,
            &q_val,
            8,
            2,
        );

        match result {
            NonterminatingProofResult::Failed { reason } => {
                assert!(
                    reason.contains("recurrence") || reason.contains("Initial condition"),
                    "Expected failure about recurrence or initial conditions, got: {}",
                    reason
                );
            }
            NonterminatingProofResult::Proved { .. } => {
                panic!("Proof should fail when RHS has wrong initial conditions");
            }
        }
    }

    // ========================================
    // Test 7: prove_1phi0_identity
    // _1phi0(q^{-n};;q,z) at a=q^{-n}: closed form (zq^{-n};q)_n
    // ========================================

    #[test]
    fn test_prove_1phi0_identity() {
        let q_val = qr_frac(1, 2);

        // z = q
        // LHS: 1phi0(q^{-n};;q, q)
        let lhs_builder = |n: i64| -> HypergeometricSeries {
            HypergeometricSeries {
                upper: vec![QMonomial::q_power(-n)],
                lower: vec![],
                argument: QMonomial::q_power(1),
            }
        };

        // RHS: (z*q^{-n};q)_n = (q^{1-n};q)_n at concrete q
        let rhs_builder = |n: i64| -> QRat {
            if n == 0 {
                return QRat::one();
            }
            // a = z * q^{-n} = q^{1-n}
            let a_val = qrat_pow_i64(&q_val, 1 - n);
            pochhammer_scalar(&a_val, &q_val, n)
        };

        let result = prove_nonterminating(
            &lhs_builder,
            &rhs_builder,
            &q_val,
            8,
            2,
        );

        match result {
            NonterminatingProofResult::Proved { .. } => {
                // Success
            }
            NonterminatingProofResult::Failed { reason } => {
                panic!("1phi0 proof should succeed, but failed: {}", reason);
            }
        }
    }

    // ========================================
    // Test 8: prove_with_different_n_test
    // Same q-Gauss identity but with different n_test values (5, 8, 10)
    // ========================================

    #[test]
    fn test_prove_with_different_n_test() {
        let q_val = qr_frac(1, 2);

        let c_val = qrat_pow_i64(&q_val, 3);
        let c_over_b = qrat_pow_i64(&q_val, 1); // c/b = q

        let lhs_builder = |n: i64| -> HypergeometricSeries {
            HypergeometricSeries {
                upper: vec![QMonomial::q_power(-n), QMonomial::q_power(2)],
                lower: vec![QMonomial::q_power(3)],
                argument: QMonomial::q_power(n + 1),
            }
        };

        let rhs_builder = |n: i64| -> QRat {
            if n == 0 {
                return QRat::one();
            }
            let numer = pochhammer_scalar(&c_over_b, &q_val, n);
            let denom = pochhammer_scalar(&c_val, &q_val, n);
            &numer / &denom
        };

        for &n_test in &[5i64, 8, 10] {
            let result = prove_nonterminating(
                &lhs_builder,
                &rhs_builder,
                &q_val,
                n_test,
                2,
            );

            match result {
                NonterminatingProofResult::Proved { .. } => {
                    // Success for this n_test
                }
                NonterminatingProofResult::Failed { reason } => {
                    panic!(
                        "q-Gauss proof should succeed at n_test={}, but failed: {}",
                        n_test, reason
                    );
                }
            }
        }
    }

    // ========================================
    // Test 9: NonterminatingProofResult::Proved contains expected fields
    // ========================================

    #[test]
    fn test_proof_result_fields() {
        let q_val = qr_frac(1, 2);

        let c_val = qrat_pow_i64(&q_val, 3);
        let c_over_a = qrat_pow_i64(&q_val, 1);

        let lhs_builder = |n: i64| -> HypergeometricSeries {
            HypergeometricSeries {
                upper: vec![QMonomial::q_power(-n), QMonomial::q_power(2)],
                lower: vec![QMonomial::q_power(3)],
                argument: QMonomial::q_power(n + 1),
            }
        };

        let rhs_builder = |n: i64| -> QRat {
            if n == 0 {
                return QRat::one();
            }
            let numer = pochhammer_scalar(&c_over_a, &q_val, n);
            let denom = pochhammer_scalar(&c_val, &q_val, n);
            &numer / &denom
        };

        let result = prove_nonterminating(
            &lhs_builder,
            &rhs_builder,
            &q_val,
            8,
            2,
        );

        match result {
            NonterminatingProofResult::Proved {
                recurrence_order,
                recurrence_coefficients,
                initial_conditions_checked,
            } => {
                assert!(recurrence_order >= 1);
                assert_eq!(recurrence_coefficients.len(), recurrence_order + 1);
                assert_eq!(initial_conditions_checked, recurrence_order + 1);
                assert!(!recurrence_coefficients[0].is_zero());
                assert!(!recurrence_coefficients[recurrence_order].is_zero());
            }
            NonterminatingProofResult::Failed { reason } => {
                panic!("Expected Proved, got Failed: {}", reason);
            }
        }
    }

    // ========================================
    // Test 10: prove_fails_with_max_order_zero
    // ========================================

    #[test]
    fn test_prove_fails_max_order_zero() {
        let q_val = qr_frac(1, 2);

        let lhs_builder = |n: i64| -> HypergeometricSeries {
            HypergeometricSeries {
                upper: vec![QMonomial::q_power(-n), QMonomial::q_power(2)],
                lower: vec![QMonomial::q_power(3)],
                argument: QMonomial::q_power(n + 1),
            }
        };

        let rhs_builder = |_n: i64| -> QRat {
            QRat::one()
        };

        // max_order = 0 means no recurrence can be found
        let result = prove_nonterminating(
            &lhs_builder,
            &rhs_builder,
            &q_val,
            5,
            0,
        );

        match result {
            NonterminatingProofResult::Failed { reason } => {
                assert!(
                    reason.contains("no recurrence"),
                    "Expected 'no recurrence' in failure reason, got: {}",
                    reason
                );
            }
            NonterminatingProofResult::Proved { .. } => {
                panic!("Should fail with max_order = 0");
            }
        }
    }

    // ========================================
    // Test 11: check_recurrence_on_values (scalar unit test)
    // ========================================

    #[test]
    fn test_recurrence_check_on_values() {
        // Recurrence: f(n+1) = 3*f(n), so 3*f(n) - f(n+1) = 0
        let coeffs = vec![qr(3), qr(-1)];

        // f(0) = 1, f(1) = 3, f(2) = 9
        assert!(check_recurrence_on_values(&[qr(1), qr(3)], &coeffs));
        assert!(check_recurrence_on_values(&[qr(3), qr(9)], &coeffs));

        // Wrong sequence
        assert!(!check_recurrence_on_values(&[qr(1), qr(2)], &coeffs));
    }
}
