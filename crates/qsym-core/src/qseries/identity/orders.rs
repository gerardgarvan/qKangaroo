//! Order of vanishing at cusps for eta quotients.
//!
//! Uses Garvan's cuspord formula (derived from Ligozat) to compute the
//! invariant order of an eta quotient at a cusp of Gamma_0(N), and
//! provides cusp width computation.
//!
//! # Conventions
//!
//! - **cuspord** (invariant order): the "raw" order of vanishing at a cusp.
//! - **cuspORD** (weighted order): cuspord * cusp_width.
//! - For a weight-0 modular function, sum of cuspORDs across all cusps = 0.
//!
//! # References
//!
//! - Ligozat (1975), "Courbes modulaires de genre 1"
//! - Garvan's ETA package: cuspord and cuspORD functions

use crate::number::QRat;
use super::cusps::{Cusp, gcd};
use super::eta::EtaExpression;

/// Compute the invariant order of vanishing (cuspord) of an eta quotient at a cusp.
///
/// For f = prod_{delta | N} eta(delta * tau)^{r_delta}, the invariant order
/// at cusp a/c of Gamma_0(N) is:
///
/// ```text
/// cuspord(f, a/c) = sum_{delta | N} gcd(c, delta)^2 * r_delta / (24 * delta)
/// ```
///
/// The invariant order depends only on c (the denominator), not on a (the numerator).
///
/// For the cusp at infinity (1/0): the invariant order equals the q-shift
/// = sum(delta * r_delta) / 24.
pub fn eta_order_at_cusp(eta: &EtaExpression, cusp: &Cusp) -> QRat {
    if cusp.is_infinity() {
        // Order at infinity = q-shift = sum(delta * r_delta) / 24
        return eta.q_shift();
    }

    let c = cusp.denom.abs();

    let mut sum = QRat::zero();
    for (&delta, &r_delta) in &eta.factors {
        if r_delta == 0 {
            continue;
        }
        let gcd_c_delta = gcd(c, delta);
        // Contribution: gcd(c, delta)^2 * r_delta / (24 * delta)
        let numer = gcd_c_delta * gcd_c_delta * r_delta;
        let denom = 24 * delta;
        sum = sum + QRat::from((numer, denom));
    }

    sum
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

/// Compute the total weighted order (sum of cuspORDs) of an eta quotient across all cusps.
///
/// total = sum_{cusp s} cusp_width(s) * cuspord(f, s)
///
/// For a weight-0 modular function on Gamma_0(N), this total should be 0.
/// This serves as a correctness check for the formula implementation.
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
