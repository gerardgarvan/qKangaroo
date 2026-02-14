//! Integration tests for cusp computation and order-at-cusp formulas.
//!
//! Tests cover:
//! - num_cusps_gamma0 against known values
//! - cuspmake for small N with exact cusp sets
//! - cuspmake count matches formula for N=1..50
//! - Cusp struct: infinity, normalization, display
//! - cusp_width known values
//! - eta_order_at_cusp (invariant order / cuspord) against known eta quotients
//! - total_order (weighted sum = 0) for weight-0 modular functions
//! - cuspmake1 basic correctness

use qsym_core::number::QRat;
use qsym_core::qseries::identity::{
    Cusp, cuspmake, cuspmake1, num_cusps_gamma0,
    EtaExpression, eta_order_at_cusp, cusp_width, total_order,
};

fn qrat(n: i64, d: i64) -> QRat {
    QRat::from((n, d))
}

// ============================================================================
// Test group 1: num_cusps_gamma0 -- known cusp counts
// ============================================================================

#[test]
fn num_cusps_gamma0_known_values() {
    assert_eq!(num_cusps_gamma0(1), 1);
    assert_eq!(num_cusps_gamma0(2), 2);
    assert_eq!(num_cusps_gamma0(3), 2);
    assert_eq!(num_cusps_gamma0(4), 3);
    assert_eq!(num_cusps_gamma0(5), 2);
    assert_eq!(num_cusps_gamma0(6), 4);
    assert_eq!(num_cusps_gamma0(7), 2);
    assert_eq!(num_cusps_gamma0(8), 4);
    assert_eq!(num_cusps_gamma0(10), 4);
    assert_eq!(num_cusps_gamma0(12), 6);
    assert_eq!(num_cusps_gamma0(24), 8);
    assert_eq!(num_cusps_gamma0(36), 12);
}

#[test]
fn num_cusps_gamma0_primes() {
    // For prime p, num_cusps = phi(gcd(1,p)) + phi(gcd(p,1)) = 1 + 1 = 2
    for &p in &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47] {
        assert_eq!(num_cusps_gamma0(p), 2,
            "num_cusps_gamma0({}) should be 2 for prime", p);
    }
}

// ============================================================================
// Test group 2: cuspmake -- correct cusps for small N
// ============================================================================

#[test]
fn cuspmake_n1() {
    let cusps = cuspmake(1);
    assert_eq!(cusps.len(), 1);
    assert!(cusps.contains(&Cusp::infinity()));
}

#[test]
fn cuspmake_n2() {
    let cusps = cuspmake(2);
    assert_eq!(cusps.len(), 2);
    assert!(cusps.contains(&Cusp::infinity()));
    assert!(cusps.contains(&Cusp::new(0, 1)));  // cusp at 0
}

#[test]
fn cuspmake_n4() {
    let cusps = cuspmake(4);
    assert_eq!(cusps.len(), 3);
    assert!(cusps.contains(&Cusp::infinity()));
    assert!(cusps.contains(&Cusp::new(0, 1)));  // cusp 0
    assert!(cusps.contains(&Cusp::new(1, 2)));  // cusp 1/2
}

#[test]
fn cuspmake_n5() {
    // Gamma_0(5) for prime 5: cusps are infinity and 0
    let cusps = cuspmake(5);
    assert_eq!(cusps.len(), 2);
    assert!(cusps.contains(&Cusp::infinity()));
    assert!(cusps.contains(&Cusp::new(0, 1)));
}

#[test]
fn cuspmake_n6() {
    let cusps = cuspmake(6);
    assert_eq!(cusps.len(), 4);
    assert!(cusps.contains(&Cusp::infinity()));
    assert!(cusps.contains(&Cusp::new(0, 1)));
    // Should also contain cusps with denom 2 and 3
}

#[test]
fn cuspmake_n12() {
    let cusps = cuspmake(12);
    assert_eq!(cusps.len(), 6);
    // Known cusps: inf, 0/1, 1/2, 1/3, 1/4, 1/6
    assert!(cusps.contains(&Cusp::infinity()));
    assert!(cusps.contains(&Cusp::new(0, 1)));
    assert!(cusps.contains(&Cusp::new(1, 2)));
    assert!(cusps.contains(&Cusp::new(1, 3)));
    assert!(cusps.contains(&Cusp::new(1, 4)));
    assert!(cusps.contains(&Cusp::new(1, 6)));
}

#[test]
fn cuspmake_n24() {
    let cusps = cuspmake(24);
    assert_eq!(cusps.len(), 8);
}

#[test]
fn cuspmake_count_matches_formula() {
    // Verify cuspmake output length matches num_cusps_gamma0 for N = 1..50
    for n in 1..=50 {
        let cusps = cuspmake(n);
        let expected = num_cusps_gamma0(n) as usize;
        assert_eq!(cusps.len(), expected,
            "cuspmake({}) returned {} cusps, expected {}", n, cusps.len(), expected);
    }
}

#[test]
fn cuspmake_no_duplicate_cusps() {
    // Verify no duplicates for various N
    for n in 1..=30 {
        let cusps = cuspmake(n);
        let mut sorted = cusps.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(cusps.len(), sorted.len(),
            "cuspmake({}) has duplicate cusps", n);
    }
}

// ============================================================================
// Test group 3: Cusp struct
// ============================================================================

#[test]
fn cusp_infinity() {
    let inf = Cusp::infinity();
    assert!(inf.is_infinity());
    assert_eq!(inf.numer, 1);
    assert_eq!(inf.denom, 0);
}

#[test]
fn cusp_new_reduces() {
    let c = Cusp::new(2, 4);
    assert_eq!(c.numer, 1);
    assert_eq!(c.denom, 2);
}

#[test]
fn cusp_new_negative_denom() {
    let c = Cusp::new(1, -3);
    assert_eq!(c.numer, -1);
    assert_eq!(c.denom, 3);
}

#[test]
fn cusp_new_zero_denom_is_infinity() {
    let c = Cusp::new(5, 0);
    assert!(c.is_infinity());
    assert_eq!(c.numer, 1);
    assert_eq!(c.denom, 0);
}

#[test]
fn cusp_new_zero_numer() {
    // 0/1 should be represented as 0/1
    let c = Cusp::new(0, 1);
    assert!(!c.is_infinity());
    assert_eq!(c.numer, 0);
    assert_eq!(c.denom, 1);
}

#[test]
fn cusp_display() {
    assert_eq!(format!("{}", Cusp::infinity()), "inf");
    assert_eq!(format!("{}", Cusp::new(1, 3)), "1/3");
    assert_eq!(format!("{}", Cusp::new(2, 5)), "2/5");
    assert_eq!(format!("{}", Cusp::new(0, 1)), "0/1");
}

// ============================================================================
// Test group 4: cusp_width
// ============================================================================

#[test]
fn cusp_width_infinity() {
    assert_eq!(cusp_width(12, &Cusp::infinity()), 1);
    assert_eq!(cusp_width(1, &Cusp::infinity()), 1);
    assert_eq!(cusp_width(24, &Cusp::infinity()), 1);
}

#[test]
fn cusp_width_zero() {
    // Width of cusp 0 = 0/1 on Gamma_0(N): N/gcd(1, N) = N/1 = N
    assert_eq!(cusp_width(5, &Cusp::new(0, 1)), 5);
    assert_eq!(cusp_width(12, &Cusp::new(0, 1)), 12);
    assert_eq!(cusp_width(1, &Cusp::new(0, 1)), 1);
}

#[test]
fn cusp_width_values_n12() {
    // For Gamma_0(12):
    // width(inf) = 1
    // width(0/1) = 12/gcd(1,12) = 12
    // width(1/2) = 12/gcd(4,12) = 12/4 = 3
    // width(1/3) = 12/gcd(9,12) = 12/3 = 4
    // width(1/4) = 12/gcd(16,12) = 12/4 = 3
    // width(1/6) = 12/gcd(36,12) = 12/12 = 1
    assert_eq!(cusp_width(12, &Cusp::infinity()), 1);
    assert_eq!(cusp_width(12, &Cusp::new(0, 1)), 12);
    assert_eq!(cusp_width(12, &Cusp::new(1, 2)), 3);
    assert_eq!(cusp_width(12, &Cusp::new(1, 3)), 4);
    assert_eq!(cusp_width(12, &Cusp::new(1, 4)), 3);
    assert_eq!(cusp_width(12, &Cusp::new(1, 6)), 1);
}

// ============================================================================
// Test group 5: eta_order_at_cusp -- invariant order (cuspord)
// ============================================================================

#[test]
fn eta_order_at_infinity_level5() {
    // f = eta(5*tau)^6 / eta(tau)^6 on Gamma_0(5)
    // Order at infinity = q_shift = sum(delta*r_delta)/24 = (1*(-6) + 5*6)/24 = 24/24 = 1
    let eta = EtaExpression::from_factors(&[(1, -6), (5, 6)], 5);
    let ord_inf = eta_order_at_cusp(&eta, &Cusp::infinity());
    assert_eq!(ord_inf, qrat(1, 1));
}

#[test]
fn eta_order_at_zero_level5() {
    // f = eta(5*tau)^6 / eta(tau)^6 on Gamma_0(5)
    // Cusp 0 = 0/1 has c=1.
    // cuspord = gcd(1,1)^2*(-6)/(24*1) + gcd(1,5)^2*6/(24*5)
    //         = 1*(-6)/24 + 1*6/120
    //         = -1/4 + 1/20
    //         = -5/20 + 1/20
    //         = -4/20
    //         = -1/5
    let eta = EtaExpression::from_factors(&[(1, -6), (5, 6)], 5);
    let ord_zero = eta_order_at_cusp(&eta, &Cusp::new(0, 1));
    assert_eq!(ord_zero, qrat(-1, 5));
}

#[test]
fn eta_total_order_zero_level5() {
    // f = eta(5*tau)^6 / eta(tau)^6 on Gamma_0(5)
    // Cusps: inf and 0/1
    // cuspORD(inf) = 1 * width(inf) = 1 * 1 = 1
    // cuspORD(0) = -1/5 * width(0) = -1/5 * 5 = -1
    // Total = 1 + (-1) = 0
    let eta = EtaExpression::from_factors(&[(1, -6), (5, 6)], 5);
    let cusps = cuspmake(5);
    let total = total_order(&eta, &cusps);
    assert_eq!(total, qrat(0, 1),
        "Total weighted order should be 0 for weight-0 eta quotient on level 5, got {}", total);
}

#[test]
fn eta_order_at_cusp_level4() {
    // eta(2*tau)^12 / (eta(tau)^6 * eta(4*tau)^6) on Gamma_0(4)
    // factors = {1: -6, 2: 12, 4: -6}, level = 4
    // weight = (-6+12-6)/2 = 0
    let eta = EtaExpression::from_factors(&[(1, -6), (2, 12), (4, -6)], 4);
    let cusps = cuspmake(4);
    assert_eq!(cusps.len(), 3);

    // Check total weighted order is 0
    let total = total_order(&eta, &cusps);
    assert_eq!(total, qrat(0, 1),
        "Total weighted order should be 0 for weight-0 eta quotient on level 4, got {}", total);
}

// ============================================================================
// Test group 6: total_order zero for multiple levels
// ============================================================================

#[test]
fn total_order_zero_level25() {
    // eta(tau)^{-6} * eta(25*tau)^6 on Gamma_0(25)
    let eta = EtaExpression::from_factors(&[(1, -6), (25, 6)], 25);
    let cusps = cuspmake(25);
    let total = total_order(&eta, &cusps);
    assert_eq!(total, qrat(0, 1),
        "Total weighted order for level 25 should be 0, got {}", total);
}

#[test]
fn total_order_zero_level7() {
    // eta(tau)^{-4} * eta(7*tau)^4 on Gamma_0(7), weight 0
    let eta = EtaExpression::from_factors(&[(1, -4), (7, 4)], 7);
    let cusps = cuspmake(7);
    let total = total_order(&eta, &cusps);
    assert_eq!(total, qrat(0, 1),
        "Total weighted order for level 7 should be 0, got {}", total);
}

#[test]
fn total_order_zero_level12() {
    // eta(tau)^{-1} * eta(12*tau)^1 on Gamma_0(12), weight 0
    let eta = EtaExpression::from_factors(&[(1, -1), (12, 1)], 12);
    let cusps = cuspmake(12);
    let total = total_order(&eta, &cusps);
    assert_eq!(total, qrat(0, 1),
        "Total weighted order for level 12 should be 0, got {}", total);
}

#[test]
fn total_order_zero_level6() {
    // eta(tau)^{-2} * eta(6*tau)^2 on Gamma_0(6), weight 0
    let eta = EtaExpression::from_factors(&[(1, -2), (6, 2)], 6);
    let cusps = cuspmake(6);
    let total = total_order(&eta, &cusps);
    assert_eq!(total, qrat(0, 1),
        "Total weighted order for level 6 should be 0, got {}", total);
}

// ============================================================================
// Test group 7: detailed order values for known eta quotients
// ============================================================================

#[test]
fn eta_orders_nonneg_check_level5() {
    // f = eta(5*tau)^6 / eta(tau)^6 on Gamma_0(5)
    // This has order 1 at infinity and -1/5 at cusp 0.
    // It's NOT holomorphic (has a pole at cusp 0).
    let eta = EtaExpression::from_factors(&[(1, -6), (5, 6)], 5);
    let cusps = cuspmake(5);

    let orders: Vec<(String, QRat)> = cusps.iter()
        .map(|c| (format!("{}", c), eta_order_at_cusp(&eta, c)))
        .collect();
    assert_eq!(orders.len(), 2);

    // Total weighted order should be 0
    let total = total_order(&eta, &cusps);
    assert_eq!(total, qrat(0, 1));
}

#[test]
fn eta_order_all_cusps_level4_explicit() {
    // eta(2*tau)^12 / (eta(tau)^6 * eta(4*tau)^6) on Gamma_0(4)
    let eta = EtaExpression::from_factors(&[(1, -6), (2, 12), (4, -6)], 4);
    // Cusps: inf, 0/1, 1/2
    let cusps = cuspmake(4);
    assert_eq!(cusps.len(), 3);

    // Verify each order and that the total is 0
    let mut total_weighted = QRat::zero();
    for cusp in &cusps {
        let ord = eta_order_at_cusp(&eta, cusp);
        let w = cusp_width(4, cusp);
        total_weighted = total_weighted + ord * QRat::from((w, 1i64));
    }
    assert_eq!(total_weighted, qrat(0, 1));
}

// ============================================================================
// Test group 8: cuspmake1 basic tests
// ============================================================================

#[test]
fn cuspmake1_n1() {
    let cusps = cuspmake1(1);
    assert_eq!(cusps.len(), 1);
    assert!(cusps.contains(&Cusp::infinity()));
}

#[test]
fn cuspmake1_n2() {
    let cusps = cuspmake1(2);
    // Gamma_1(2) should have same cusps as Gamma_0(2) since N=2, -I in Gamma_1(2)
    assert_eq!(cusps.len(), 2);
}

#[test]
fn cuspmake1_n5() {
    // Gamma_1(5) has more cusps than Gamma_0(5)
    let cusps1 = cuspmake1(5);
    let cusps0 = cuspmake(5);
    assert!(cusps1.len() >= cusps0.len(),
        "Gamma_1(N) should have at least as many cusps as Gamma_0(N): got {} vs {}",
        cusps1.len(), cusps0.len());
}

#[test]
fn cuspmake1_at_least_gamma0() {
    // For all N, Gamma_1(N) cusps >= Gamma_0(N) cusps
    for n in 1..=20 {
        let cusps0 = cuspmake(n);
        let cusps1 = cuspmake1(n);
        assert!(cusps1.len() >= cusps0.len(),
            "cuspmake1({}) = {} < cuspmake({}) = {}",
            n, cusps1.len(), n, cusps0.len());
    }
}

// ============================================================================
// Test group 9: cusp_width sum formula
// ============================================================================

#[test]
fn cusp_width_sum_equals_index() {
    // For Gamma_0(N): sum of widths across all cusps = [SL_2(Z) : Gamma_0(N)]
    // The index of Gamma_0(N) in SL_2(Z) is N * prod_{p|N}(1 + 1/p)
    for n in 1..=30i64 {
        let cusps = cuspmake(n);
        let width_sum: i64 = cusps.iter().map(|c| cusp_width(n, c)).sum();

        let index = psi_n(n);
        assert_eq!(width_sum, index,
            "Sum of widths for N={}: got {}, expected {} (index of Gamma_0({}))",
            n, width_sum, index, n);
    }
}

/// Compute the index of Gamma_0(N) in SL_2(Z): N * prod_{p|N}(1 + 1/p).
fn psi_n(n: i64) -> i64 {
    let mut result = n;
    let mut m = n;
    let mut p = 2i64;
    while p * p <= m {
        if m % p == 0 {
            while m % p == 0 {
                m /= p;
            }
            // Multiply by (1 + 1/p) = (p+1)/p
            result = result / p * (p + 1);
        }
        p += 1;
    }
    if m > 1 {
        result = result / m * (m + 1);
    }
    result
}

// ============================================================================
// Test group 10: total_order for many weight-0 eta quotients
// ============================================================================

#[test]
fn total_order_zero_systematic() {
    // For various primes p, test eta(tau)^{-k} * eta(p*tau)^k with k chosen
    // so that weight = 0 (any k works since sum r = -k+k = 0).
    for &p in &[2, 3, 5, 7, 11, 13] {
        for &k in &[1, 2, 4, 6, 12] {
            let eta = EtaExpression::from_factors(&[(1, -k), (p, k)], p);
            let cusps = cuspmake(p);
            let total = total_order(&eta, &cusps);
            assert_eq!(total, qrat(0, 1),
                "Total weighted order for eta(tau)^{} * eta({}*tau)^{} should be 0, got {}",
                -k, p, k, total);
        }
    }
}
