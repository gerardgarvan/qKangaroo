//! Order of vanishing at cusps for eta quotients.
//!
//! Uses the Ligozat formula to compute the order of an eta quotient
//! at a cusp of Gamma_0(N), and provides cusp width computation.
//!
//! # References
//!
//! - Ligozat (1975), "Courbes modulaires de genre 1"
//! - Garvan's ETA package: cuspord and cuspORD functions

use crate::number::QRat;
use super::cusps::{Cusp, gcd};
use super::eta::EtaExpression;

/// Compute the order of vanishing of an eta quotient at a cusp via the Ligozat formula.
///
/// For f = prod_{delta | N} eta(delta * tau)^{r_delta}, the order at cusp a/c
/// of Gamma_0(N) is:
///
/// ```text
/// ord_{a/c}(f) = (N / 24) * sum_{delta | N} [ gcd(c, delta)^2 * r_delta / (gcd(c, N/c) * c * delta) ]
/// ```
///
/// The order depends only on c (the denominator), not on a (the numerator).
///
/// For the cusp at infinity (1/0): the order equals the q-shift = sum(delta * r_delta) / 24.
pub fn eta_order_at_cusp(eta: &EtaExpression, cusp: &Cusp) -> QRat {
    let n = eta.level;

    if cusp.is_infinity() {
        // Order at infinity = q-shift = sum(delta * r_delta) / 24
        return eta.q_shift();
    }

    let c = cusp.denom.abs();
    let g = gcd(c, n / c);

    let mut sum = QRat::zero();
    for (&delta, &r_delta) in &eta.factors {
        if r_delta == 0 {
            continue;
        }
        let gcd_c_delta = gcd(c, delta);
        // Contribution: gcd(c, delta)^2 * r_delta / (gcd(c, N/c) * c * delta)
        let numer = gcd_c_delta * gcd_c_delta * r_delta;
        let denom = g * c * delta;
        sum = sum + QRat::from((numer, denom));
    }

    // Multiply by N/24
    sum * QRat::from((n, 24i64))
}

/// Compute the width of a cusp on Gamma_0(N).
///
/// For cusp a/c (where c is the denominator):
///   width = N / gcd(c^2, N)
///
/// For infinity (c=0): width = 1.
pub fn cusp_width(n: i64, cusp: &Cusp) -> i64 {
    if cusp.is_infinity() {
        return 1;
    }
    let c = cusp.denom.abs();
    n / gcd(c * c, n)
}

/// Compute the total weighted order of an eta quotient across all cusps.
///
/// total = sum_{cusp s} width(s) * ord_s(f)
///
/// For a weight-0 modular function on Gamma_0(N), this total should be 0.
/// This serves as a correctness check for the Ligozat formula implementation.
pub fn total_order(eta: &EtaExpression, cusps: &[Cusp]) -> QRat {
    let n = eta.level;
    let mut total = QRat::zero();
    for cusp in cusps {
        let ord = eta_order_at_cusp(eta, cusp);
        let width = QRat::from((cusp_width(n, cusp), 1i64));
        total = total + ord * width;
    }
    total
}
