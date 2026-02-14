//! Order of vanishing at cusps for eta quotients.
//!
//! Uses the Ligozat formula to compute the order of an eta quotient
//! at a cusp of Gamma_0(N), and provides cusp width computation.
//!
//! Placeholder -- implementation in Task 2.

use crate::number::QRat;
use super::cusps::{Cusp, gcd};
use super::eta::EtaExpression;

/// Compute the order of vanishing of an eta quotient at a cusp via the Ligozat formula.
pub fn eta_order_at_cusp(_eta: &EtaExpression, _cusp: &Cusp) -> QRat {
    unimplemented!("Task 2")
}

/// Compute the width of a cusp on Gamma_0(N).
pub fn cusp_width(_n: i64, _cusp: &Cusp) -> i64 {
    unimplemented!("Task 2")
}

/// Compute the total weighted order of an eta quotient across all cusps.
pub fn total_order(_eta: &EtaExpression, _cusps: &[Cusp]) -> QRat {
    unimplemented!("Task 2")
}
