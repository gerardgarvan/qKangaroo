//! q-Petkovsek algorithm for solving constant-coefficient q-difference equations.
//!
//! Given a recurrence c_0*S(n) + c_1*S(n+1) + ... + c_d*S(n+d) = 0 with constant
//! QRat coefficients (as produced by q-Zeilberger at concrete q), this module finds
//! all q-hypergeometric solutions and optionally expresses them as q-Pochhammer products.
//!
//! Key components:
//! - [`QPetkovsekResult`]: Solution with ratio and optional closed-form decomposition
//! - [`ClosedForm`]: Representation as q-Pochhammer products with q-power prefactor
//! - [`q_petkovsek`]: Main entry point for solving constant-coefficient recurrences

use crate::number::QRat;
use super::QMonomial;

// ---- Private helpers (duplicated from gosper.rs/zeilberger.rs) ----

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

// ---- Data types ----

/// Closed-form representation of a q-hypergeometric solution.
///
/// Represents: scalar * q^{q_power_coeff * n*(n-1)/2} * prod_i (numer_factors_i; q)_n / prod_j (denom_factors_j; q)_n
///
/// The q_power_coeff uses the n*(n-1)/2 convention, matching the natural normalization
/// of q-Pochhammer products. For example, (a;q)_n contains a factor with q-power
/// proportional to n*(n-1)/2. This convention does NOT cover simple geometric sequences
/// like r^n = q^{mn}; such geometric behavior is captured entirely by
/// QPetkovsekResult.ratio, and ClosedForm is only produced when the ratio genuinely
/// factors into Pochhammer terms.
#[derive(Clone, Debug)]
pub struct ClosedForm {
    /// Scalar prefactor (from S(0) normalization).
    pub scalar: QRat,
    /// Coefficient for q-power: the solution includes factor q^{q_power_coeff * n*(n-1)/2}.
    /// Uses n*(n-1)/2 convention (matching q-Pochhammer natural normalization).
    pub q_power_coeff: i64,
    /// Pochhammer numerator factors: each (a_i; q)_n in the product.
    pub numer_factors: Vec<QMonomial>,
    /// Pochhammer denominator factors: each (b_j; q)_n in the product.
    pub denom_factors: Vec<QMonomial>,
}

/// Result of the q-Petkovsek algorithm for a single solution.
#[derive(Clone, Debug)]
pub struct QPetkovsekResult {
    /// The ratio y(n+1)/y(n) as a QRat constant (for constant-coefficient recurrences).
    /// This always holds the exact solution ratio, even when closed_form is None.
    pub ratio: QRat,
    /// Closed-form representation as q-Pochhammer factors, if decomposition succeeded.
    /// None when the ratio cannot be cleanly decomposed into Pochhammer terms
    /// (the ratio itself is still the valid solution).
    pub closed_form: Option<ClosedForm>,
}

// ---- Divisor helper for rational root theorem ----

/// Find all positive divisors of an integer.
///
/// Returns the sorted list of positive divisors of |n|.
/// For n = 0, returns an empty vector.
/// Caps at a reasonable limit to avoid explosion on huge numbers.
fn positive_divisors(n: &rug::Integer) -> Vec<rug::Integer> {
    if *n == 0 {
        return Vec::new();
    }
    let abs_n = n.clone().abs();
    if abs_n == 1 {
        return vec![rug::Integer::from(1)];
    }

    // For very large numbers, cap trial division to avoid explosion
    let max_trial = 10_000u64;

    let mut divisors = Vec::new();
    let mut i = rug::Integer::from(1);
    let sqrt_n = abs_n.clone().sqrt();

    loop {
        if i > sqrt_n || i > max_trial {
            break;
        }

        let (quotient, remainder) = abs_n.clone().div_rem(i.clone());
        if remainder == 0 {
            divisors.push(i.clone());
            if quotient != i {
                divisors.push(quotient);
            }
        }

        i += 1;
    }

    divisors.sort();
    divisors
}

// ---- Core algorithm ----

/// Solve a constant-coefficient q-recurrence for q-hypergeometric solutions.
///
/// Given recurrence c_0*S(n) + c_1*S(n+1) + ... + c_d*S(n+d) = 0
/// where all c_j are QRat constants (as produced by q-Zeilberger at concrete q),
/// find all q-hypergeometric solutions.
///
/// The key simplification: since coefficients are constants (not polynomials in q^n),
/// a q-hypergeometric solution y(n+1)/y(n) = r must satisfy the characteristic
/// equation c_0 + c_1*r + c_2*r^2 + ... + c_d*r^d = 0.
///
/// For order 1: r = -c_0/c_1 (unique solution).
/// For order 2+: solve the polynomial c_0 + c_1*r + ... + c_d*r^d = 0 for rational roots.
///
/// # Panics
///
/// Panics if `coefficients` has length < 2 (need at least order 1).
/// Panics if the leading coefficient (last element) is zero.
pub fn q_petkovsek(
    coefficients: &[QRat],
    q_val: &QRat,
) -> Vec<QPetkovsekResult> {
    assert!(
        coefficients.len() >= 2,
        "q_petkovsek: need at least 2 coefficients (order >= 1), got {}",
        coefficients.len()
    );
    let d = coefficients.len() - 1;
    assert!(
        !coefficients[d].is_zero(),
        "q_petkovsek: leading coefficient c_{} must be non-zero",
        d
    );

    if d == 1 {
        // Order 1: unique solution r = -c_0/c_1
        let ratio = -(&coefficients[0] / &coefficients[1]);
        let closed_form = try_decompose_ratio(&ratio, q_val);
        return vec![QPetkovsekResult { ratio, closed_form }];
    }

    // Order 2+: solve characteristic polynomial via Rational Root Theorem.
    //
    // The characteristic equation is: c_0 + c_1*r + c_2*r^2 + ... + c_d*r^d = 0
    //
    // To apply the Rational Root Theorem, we need integer coefficients.
    // Multiply through by lcm of all coefficient denominators.
    let mut lcm_denom = rug::Integer::from(1);
    for c in coefficients.iter() {
        let d_i = c.denom().clone();
        lcm_denom = lcm_denom.lcm(&d_i);
    }

    // Integer coefficients: a_j = c_j * lcm_denom
    let int_coeffs: Vec<rug::Integer> = coefficients.iter().map(|c| {
        let scaled = c.clone() * QRat::from(rug::Rational::from(lcm_denom.clone()));
        // This should now be an integer
        scaled.numer().clone()
    }).collect();

    // Rational Root Theorem: any rational root p/s has
    // p dividing int_coeffs[0] (constant term) and s dividing int_coeffs[d] (leading)
    let constant_term = &int_coeffs[0];
    let leading_coeff = &int_coeffs[d];

    if constant_term.clone() == 0 {
        // r = 0 is a root. Factor it out and continue.
        // For now, handle simply: if c_0 = 0, then r = 0 is a solution.
        let mut results = vec![QPetkovsekResult {
            ratio: QRat::zero(),
            closed_form: None,
        }];
        // Try other roots from the remaining polynomial
        // coefficients[1..] give us a degree-(d-1) polynomial in r
        // with roots as the other solutions (after dividing by r).
        if d >= 2 {
            let reduced: Vec<QRat> = coefficients[1..].to_vec();
            let mut sub_results = q_petkovsek(&reduced, q_val);
            results.append(&mut sub_results);
        }
        return results;
    }

    let p_divisors = positive_divisors(constant_term);
    let s_divisors = positive_divisors(leading_coeff);

    // Cap candidates to avoid combinatorial explosion
    if p_divisors.len() * s_divisors.len() > 5000 {
        // Too many candidates -- return empty (can't efficiently enumerate)
        return Vec::new();
    }

    // Generate all candidate rational roots p/s (with both signs for p)
    let mut candidates: Vec<QRat> = Vec::new();
    for p in &p_divisors {
        for s in &s_divisors {
            // +p/s
            let pos = QRat::from(rug::Rational::from((p.clone(), s.clone())));
            candidates.push(pos);
            // -p/s
            let neg = QRat::from(rug::Rational::from((-p.clone(), s.clone())));
            candidates.push(neg);
        }
    }

    // Deduplicate candidates (in case gcd(p,s) > 1 produces duplicates)
    candidates.sort_by(|a, b| {
        let diff = a.clone() - b.clone();
        if diff.is_zero() {
            std::cmp::Ordering::Equal
        } else if diff > QRat::zero() {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });
    candidates.dedup_by(|a, b| a.clone() == b.clone());

    // Test each candidate
    let mut results = Vec::new();
    for candidate in &candidates {
        if eval_char_poly(coefficients, candidate).is_zero() {
            let closed_form = try_decompose_ratio(candidate, q_val);
            results.push(QPetkovsekResult {
                ratio: candidate.clone(),
                closed_form,
            });
        }
    }

    results
}

/// Evaluate the characteristic polynomial c_0 + c_1*r + ... + c_d*r^d at r = val.
fn eval_char_poly(coefficients: &[QRat], val: &QRat) -> QRat {
    // Use Horner's method: ((c_d * r + c_{d-1}) * r + c_{d-2}) * r + ... + c_0
    let d = coefficients.len() - 1;
    let mut result = coefficients[d].clone();
    for j in (0..d).rev() {
        result = &(&result * val) + &coefficients[j];
    }
    result
}

/// Attempt to express the ratio S(n+1)/S(n) = r as q-Pochhammer factors.
///
/// Strategy:
/// 1. Check if ratio = q^m for integer m in range -20..=20.
///    If so, return None -- geometric behavior is captured by QPetkovsekResult.ratio.
/// 2. Try to express ratio as (1-q^a)/(1-q^b) for small a,b.
///    If found, build ClosedForm with the corresponding Pochhammer factors.
/// 3. Try product of two such ratios.
/// 4. If nothing works, return None.
fn try_decompose_ratio(ratio: &QRat, q_val: &QRat) -> Option<ClosedForm> {
    if ratio.is_zero() {
        return None;
    }

    // Step 1: Check if ratio = q^m for some integer m
    for m in -20i64..=20 {
        let qm = qrat_pow_i64(q_val, m);
        if ratio == &qm {
            // Geometric sequence captured by QPetkovsekResult.ratio
            return None;
        }
    }

    // Step 2: Try ratio = (1-q^a)/(1-q^b) for small a,b (both nonzero)
    // This corresponds to a single Pochhammer ratio step.
    for a in -10i64..=10 {
        if a == 0 { continue; }
        let qa = qrat_pow_i64(q_val, a);
        let numer = &QRat::one() - &qa; // 1 - q^a
        if numer.is_zero() { continue; }

        for b in -10i64..=10 {
            if b == 0 { continue; }
            let qb = qrat_pow_i64(q_val, b);
            let denom = &QRat::one() - &qb; // 1 - q^b
            if denom.is_zero() { continue; }

            let candidate = &numer / &denom;
            if &candidate == ratio {
                return Some(ClosedForm {
                    scalar: QRat::one(),
                    q_power_coeff: 0,
                    numer_factors: vec![QMonomial::q_power(a)],
                    denom_factors: vec![QMonomial::q_power(b)],
                });
            }
        }
    }

    // Step 3: Try ratio = (1-q^a1)(1-q^a2) / ((1-q^b1)(1-q^b2))
    // This is a product of two Pochhammer ratio steps.
    // Enumerate small combinations.
    for a1 in -6i64..=6 {
        if a1 == 0 { continue; }
        let qa1 = qrat_pow_i64(q_val, a1);
        let n1 = &QRat::one() - &qa1;
        if n1.is_zero() { continue; }

        for a2 in a1..=6 {
            if a2 == 0 { continue; }
            let qa2 = qrat_pow_i64(q_val, a2);
            let n2 = &QRat::one() - &qa2;
            if n2.is_zero() { continue; }
            let numer_prod = &n1 * &n2;

            for b1 in -6i64..=6 {
                if b1 == 0 { continue; }
                let qb1 = qrat_pow_i64(q_val, b1);
                let d1 = &QRat::one() - &qb1;
                if d1.is_zero() { continue; }

                for b2 in b1..=6 {
                    if b2 == 0 { continue; }
                    let qb2 = qrat_pow_i64(q_val, b2);
                    let d2 = &QRat::one() - &qb2;
                    if d2.is_zero() { continue; }
                    let denom_prod = &d1 * &d2;
                    if denom_prod.is_zero() { continue; }

                    let candidate = &numer_prod / &denom_prod;
                    if &candidate == ratio {
                        return Some(ClosedForm {
                            scalar: QRat::one(),
                            q_power_coeff: 0,
                            numer_factors: vec![
                                QMonomial::q_power(a1),
                                QMonomial::q_power(a2),
                            ],
                            denom_factors: vec![
                                QMonomial::q_power(b1),
                                QMonomial::q_power(b2),
                            ],
                        });
                    }
                }
            }
        }
    }

    // Step 4: Nothing found
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qseries::zeilberger::{q_zeilberger, QZeilbergerResult};
    use crate::qseries::HypergeometricSeries;

    fn qr(n: i64) -> QRat {
        QRat::from((n, 1i64))
    }

    fn qr_frac(n: i64, d: i64) -> QRat {
        QRat::from((n, d))
    }

    // ========================================
    // Test 1: Order-1 simple ratio
    // ========================================

    #[test]
    fn test_order1_simple() {
        // c_0 = 1, c_1 = -2 => ratio = -c_0/c_1 = -1/(-2) = 1/2
        let coeffs = vec![qr(1), qr(-2)];
        let q_val = qr_frac(1, 3); // arbitrary q
        let results = q_petkovsek(&coeffs, &q_val);

        assert_eq!(results.len(), 1, "Order-1 should have exactly 1 solution");
        assert_eq!(results[0].ratio, qr_frac(1, 2), "Ratio should be 1/2");
    }

    // ========================================
    // Test 2: Order-1 q-Vandermonde recurrence
    // ========================================

    #[test]
    fn test_order1_q_vandermonde() {
        // Feed the q-Vandermonde recurrence from q-Zeilberger at q=1/3, n=5
        let n_val = 5i64;
        let q_val = qr_frac(1, 3);

        let series = HypergeometricSeries {
            upper: vec![QMonomial::q_power(-n_val), QMonomial::q_power(2)],
            lower: vec![QMonomial::q_power(3)],
            argument: QMonomial::q_power(n_val + 1),
        };

        let result = q_zeilberger(&series, n_val, &q_val, 3, &[0], true);
        let zr = match result {
            QZeilbergerResult::Recurrence(zr) => zr,
            QZeilbergerResult::NoRecurrence => panic!("Expected recurrence"),
        };

        let pet_results = q_petkovsek(&zr.coefficients, &q_val);
        assert_eq!(pet_results.len(), 1, "Order-1 Vandermonde should have 1 solution");

        // The ratio should be -c_0/c_1
        let expected_ratio = -(&zr.coefficients[0] / &zr.coefficients[1]);
        assert_eq!(pet_results[0].ratio, expected_ratio,
            "Ratio mismatch: got {}, expected {}", pet_results[0].ratio, expected_ratio);
    }

    // ========================================
    // Test 3: Order-1 from 1phi0 recurrence
    // ========================================

    #[test]
    fn test_order1_1phi0() {
        let n_val = 3i64;
        let q_val = qr(2);

        let series = HypergeometricSeries {
            upper: vec![QMonomial::q_power(-n_val)],
            lower: vec![],
            argument: QMonomial::q_power(1),
        };

        let result = q_zeilberger(&series, n_val, &q_val, 3, &[0], false);
        let zr = match result {
            QZeilbergerResult::Recurrence(zr) => zr,
            QZeilbergerResult::NoRecurrence => panic!("Expected recurrence for 1phi0"),
        };

        let pet_results = q_petkovsek(&zr.coefficients, &q_val);
        assert!(!pet_results.is_empty(), "Should find at least one solution for 1phi0");

        // For order 1, ratio should be -c_0/c_1
        if zr.order == 1 {
            let expected = -(&zr.coefficients[0] / &zr.coefficients[1]);
            assert_eq!(pet_results[0].ratio, expected);
        }
    }

    // ========================================
    // Test 4: Order-2 two rational roots
    // ========================================

    #[test]
    fn test_order2_two_roots() {
        // Characteristic poly: (r - 1/2)(r - 1/3) = r^2 - 5/6*r + 1/6
        // c_0 = 1/6, c_1 = -5/6, c_2 = 1
        let coeffs = vec![qr_frac(1, 6), qr_frac(-5, 6), qr(1)];
        let q_val = qr_frac(1, 5); // arbitrary q

        let results = q_petkovsek(&coeffs, &q_val);
        assert_eq!(results.len(), 2, "Should find 2 rational roots");

        let mut ratios: Vec<QRat> = results.iter().map(|r| r.ratio.clone()).collect();
        ratios.sort_by(|a, b| {
            let diff = a.clone() - b.clone();
            if diff > QRat::zero() { std::cmp::Ordering::Greater }
            else if diff.is_zero() { std::cmp::Ordering::Equal }
            else { std::cmp::Ordering::Less }
        });

        assert_eq!(ratios[0], qr_frac(1, 3), "First root should be 1/3");
        assert_eq!(ratios[1], qr_frac(1, 2), "Second root should be 1/2");
    }

    // ========================================
    // Test 5: Order-2 no rational roots
    // ========================================

    #[test]
    fn test_order2_no_rational_roots() {
        // c_0=1, c_1=0, c_2=1 => r^2 + 1 = 0, roots = +/-i (no rational roots)
        let coeffs = vec![qr(1), qr(0), qr(1)];
        let q_val = qr(2);

        let results = q_petkovsek(&coeffs, &q_val);
        assert!(results.is_empty(), "r^2+1=0 should have no rational roots");
    }

    // ========================================
    // Test 6: Order-2 one rational root
    // ========================================

    #[test]
    fn test_order2_one_rational_root() {
        // Characteristic poly: (r - 2)(r^2 + 1) = r^3 - 2*r^2 + r - 2
        // = r^3 - 2*r^2 + r - 2
        // c_0 = -2, c_1 = 1, c_2 = -2, c_3 = 1
        let coeffs = vec![qr(-2), qr(1), qr(-2), qr(1)];
        let q_val = qr(3);

        let results = q_petkovsek(&coeffs, &q_val);
        assert_eq!(results.len(), 1, "Should find exactly 1 rational root");
        assert_eq!(results[0].ratio, qr(2), "The rational root should be 2");
    }

    // ========================================
    // Test 7: Order-2 repeated root
    // ========================================

    #[test]
    fn test_order2_repeated_root() {
        // (r - 3)^2 = r^2 - 6r + 9
        // c_0 = 9, c_1 = -6, c_2 = 1
        let coeffs = vec![qr(9), qr(-6), qr(1)];
        let q_val = qr(2);

        let results = q_petkovsek(&coeffs, &q_val);
        // Should find at least one instance of root r=3
        assert!(!results.is_empty(), "Should find the repeated root r=3");
        assert_eq!(results[0].ratio, qr(3), "Root should be 3");
    }

    // ========================================
    // Test 8: Closed form for q-power ratio returns None
    // ========================================

    #[test]
    fn test_closed_form_q_power() {
        // If ratio = q^m, closed_form should be None (geometric captured by ratio)
        let q_val = qr(2);
        for m in &[-3i64, -1, 0, 1, 2, 5] {
            let ratio = qrat_pow_i64(&q_val, *m);
            let result = try_decompose_ratio(&ratio, &q_val);
            assert!(result.is_none(),
                "ratio = q^{} should return None (geometric), got {:?}", m, result);
        }
    }

    // ========================================
    // Test 9: Closed form for arbitrary ratio returns None
    // ========================================

    #[test]
    fn test_closed_form_none() {
        // An arbitrary rational not decomposable into (1-q^a)/(1-q^b)
        let q_val = qr(2);
        let ratio = qr_frac(7, 13);
        let result = try_decompose_ratio(&ratio, &q_val);
        assert!(result.is_none(),
            "7/13 at q=2 should not decompose into Pochhammer factors");
    }

    // ========================================
    // Test 10: Roundtrip q-Zeilberger -> q-Petkovsek
    // ========================================

    #[test]
    fn test_roundtrip_zeilberger() {
        // q-Vandermonde: _2phi1(q^{-n}, q^2; q^3; q, q^{n+1}) with n=5, q=1/3
        let n_val = 5i64;
        let q_val = qr_frac(1, 3);

        let series = HypergeometricSeries {
            upper: vec![QMonomial::q_power(-n_val), QMonomial::q_power(2)],
            lower: vec![QMonomial::q_power(3)],
            argument: QMonomial::q_power(n_val + 1),
        };

        // Step 1: q-Zeilberger to get recurrence
        let zr_result = q_zeilberger(&series, n_val, &q_val, 3, &[0], true);
        let zr = match zr_result {
            QZeilbergerResult::Recurrence(zr) => zr,
            QZeilbergerResult::NoRecurrence => panic!("Expected recurrence"),
        };
        assert_eq!(zr.order, 1, "q-Vandermonde should have order-1 recurrence");

        // Step 2: q-Petkovsek to solve the recurrence
        let pet_results = q_petkovsek(&zr.coefficients, &q_val);
        assert_eq!(pet_results.len(), 1, "Should find exactly 1 solution");

        // Step 3: Verify the solution ratio
        let ratio = &pet_results[0].ratio;
        assert!(!ratio.is_zero(), "Solution ratio should be non-zero");

        // The ratio should satisfy c_0 + c_1*r = 0
        let check = &(&zr.coefficients[0] + &(&zr.coefficients[1] * ratio));
        assert!(check.is_zero(),
            "Solution ratio should satisfy characteristic equation: c_0 + c_1*r = {}", check);
    }

    // ========================================
    // Test 11: Empty coefficients panics
    // ========================================

    #[test]
    #[should_panic(expected = "need at least 2 coefficients")]
    fn test_empty_coefficients_panics() {
        let q_val = qr(2);
        q_petkovsek(&[], &q_val);
    }

    // ========================================
    // Test 12: Leading zero panics
    // ========================================

    #[test]
    #[should_panic(expected = "leading coefficient")]
    fn test_leading_zero_panics() {
        let coeffs = vec![qr(1), qr(0)];
        let q_val = qr(2);
        q_petkovsek(&coeffs, &q_val);
    }

    // ========================================
    // Test 13: Single coefficient panics
    // ========================================

    #[test]
    #[should_panic(expected = "need at least 2 coefficients")]
    fn test_single_coefficient_panics() {
        let q_val = qr(2);
        q_petkovsek(&[qr(5)], &q_val);
    }

    // ========================================
    // Test 14: Closed form succeeds for Pochhammer-type ratio
    // ========================================

    #[test]
    fn test_closed_form_pochhammer_ratio() {
        // ratio = (1 - q^a) / (1 - q^b) for specific a, b
        let q_val = qr(2);
        let a = 2i64;
        let b = 3i64;

        let qa = qrat_pow_i64(&q_val, a); // q^2 = 4
        let qb = qrat_pow_i64(&q_val, b); // q^3 = 8

        // ratio = (1 - 4) / (1 - 8) = -3 / -7 = 3/7
        let ratio = &(&QRat::one() - &qa) / &(&QRat::one() - &qb);

        let result = try_decompose_ratio(&ratio, &q_val);
        assert!(result.is_some(),
            "ratio = (1-q^2)/(1-q^3) should decompose at q=2");

        let cf = result.unwrap();
        assert_eq!(cf.numer_factors.len(), 1);
        assert_eq!(cf.denom_factors.len(), 1);
        assert_eq!(cf.numer_factors[0].power, a);
        assert_eq!(cf.denom_factors[0].power, b);
    }

    // ========================================
    // Test 15: Positive divisors helper
    // ========================================

    #[test]
    fn test_positive_divisors() {
        let divs_12 = positive_divisors(&rug::Integer::from(12));
        assert_eq!(divs_12, vec![
            rug::Integer::from(1),
            rug::Integer::from(2),
            rug::Integer::from(3),
            rug::Integer::from(4),
            rug::Integer::from(6),
            rug::Integer::from(12),
        ]);

        let divs_1 = positive_divisors(&rug::Integer::from(1));
        assert_eq!(divs_1, vec![rug::Integer::from(1)]);

        let divs_0 = positive_divisors(&rug::Integer::from(0));
        assert!(divs_0.is_empty());

        // Negative number: should use absolute value
        let divs_neg6 = positive_divisors(&rug::Integer::from(-6));
        assert_eq!(divs_neg6, vec![
            rug::Integer::from(1),
            rug::Integer::from(2),
            rug::Integer::from(3),
            rug::Integer::from(6),
        ]);
    }

    // ========================================
    // Test 16: Eval char poly (Horner)
    // ========================================

    #[test]
    fn test_eval_char_poly() {
        // p(r) = 2 + 3*r + r^2
        let coeffs = vec![qr(2), qr(3), qr(1)];

        // p(0) = 2
        assert_eq!(eval_char_poly(&coeffs, &qr(0)), qr(2));

        // p(1) = 2 + 3 + 1 = 6
        assert_eq!(eval_char_poly(&coeffs, &qr(1)), qr(6));

        // p(-1) = 2 - 3 + 1 = 0
        assert_eq!(eval_char_poly(&coeffs, &qr(-1)), qr(0));
    }

    // ========================================
    // Test 17: Order-3 with known roots
    // ========================================

    #[test]
    fn test_order3_known_roots() {
        // (r - 1)(r - 2)(r - 3) = r^3 - 6r^2 + 11r - 6
        // c_0 = -6, c_1 = 11, c_2 = -6, c_3 = 1
        let coeffs = vec![qr(-6), qr(11), qr(-6), qr(1)];
        let q_val = qr(2);

        let results = q_petkovsek(&coeffs, &q_val);
        assert_eq!(results.len(), 3, "Should find all 3 rational roots");

        let mut ratios: Vec<i64> = results.iter().map(|r| {
            // Convert to i64 for easy comparison
            let n = r.ratio.numer().to_i64().unwrap();
            let d = r.ratio.denom().to_i64().unwrap();
            assert_eq!(d, 1);
            n
        }).collect();
        ratios.sort();
        assert_eq!(ratios, vec![1, 2, 3]);
    }
}
