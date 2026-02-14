//! Q-polynomial factoring: decompose a polynomial into (1-q^i) factors.
//!
//! - [`qfactor`]: factors a polynomial f(q) into prod (1-q^i)^{m_i} form
//! - [`QFactorization`]: result type holding factor multiplicities

use std::collections::BTreeMap;

use crate::number::QRat;
use crate::series::FormalPowerSeries;

/// Result of factoring a q-polynomial into cyclotomic-like factors.
///
/// Represents: `scalar * prod_{i} (1 - q^i)^{factors[i]}`
///
/// If `is_exact` is true, the polynomial was fully decomposed.
#[derive(Clone, Debug)]
pub struct QFactorization {
    /// Maps i -> multiplicity where product is prod (1-q^i)^{mult}.
    /// Only nonzero multiplicities are stored.
    pub factors: BTreeMap<i64, i64>,
    /// Scalar prefactor (the constant term of the original polynomial).
    pub scalar: QRat,
    /// Whether the factorization is exact (remainder is 1 after all divisions).
    pub is_exact: bool,
}

impl QFactorization {
    /// Create a trivial factorization: scalar * (no factors), exact.
    #[allow(dead_code)]
    fn trivial(scalar: QRat) -> Self {
        Self {
            factors: BTreeMap::new(),
            scalar,
            is_exact: true,
        }
    }
}

/// Factor a q-polynomial into (1-q^i) components.
///
/// Given a polynomial f(q) (a `FormalPowerSeries` that happens to be a polynomial),
/// attempts to write it as:
///
///   f(q) = scalar * prod_{i} (1 - q^i)^{m_i}
///
/// The algorithm:
/// 1. Extract scalar = f(0), divide by it.
/// 2. For i = 1, 2, ... up to the degree of the remaining polynomial:
///    - Repeatedly try to divide by (1-q^i) using iterative polynomial division.
///    - Each successful division increments multiplicity for i.
/// 3. If the remainder is 1 (constant polynomial), the factorization is exact.
///
/// # Panics
///
/// Panics if f is the zero series.
pub fn qfactor(f: &FormalPowerSeries) -> QFactorization {
    assert!(!f.is_zero(), "Cannot factor the zero polynomial");

    // Extract the scalar (constant term)
    let scalar = f.coeff(0);
    if scalar.is_zero() {
        // f has no constant term -- cannot factor into (1-q^i) form
        // since each (1-q^i) has constant term 1.
        // Return non-exact with the polynomial as-is.
        return QFactorization {
            factors: BTreeMap::new(),
            scalar: QRat::one(),
            is_exact: false,
        };
    }

    // Divide out the scalar to get a monic polynomial (constant term = 1)
    let inv_scalar = QRat::one() / scalar.clone();
    let mut current = scale_fps(f, &inv_scalar);

    let mut factors = BTreeMap::new();

    // Try dividing by (1-q^i) for i = 1, 2, ...
    // The maximum meaningful i is the degree of the current polynomial.
    loop {
        let deg = match poly_degree(&current) {
            Some(d) => d,
            None => break, // current is just a constant
        };
        if deg == 0 {
            break;
        }

        // Find the smallest i that divides current
        let mut found_any = false;
        for i in 1..=deg {
            loop {
                match try_divide_by_1_minus_q_i(&current, i) {
                    Some(quotient) => {
                        *factors.entry(i).or_insert(0) += 1;
                        current = quotient;
                        found_any = true;
                    }
                    None => break,
                }
            }
        }

        if !found_any {
            break;
        }
    }

    // Check if remainder is 1 (just constant term 1, no other terms)
    let is_exact = current.num_nonzero() <= 1
        && current.coeff(0) == QRat::one();

    QFactorization {
        factors,
        scalar,
        is_exact,
    }
}

/// Try to divide polynomial `f` by `(1 - q^i)` using iterative extraction.
///
/// Algorithm: process terms from lowest to highest. For each term c*q^k,
/// set quotient[k] += c, then subtract c*q^k*(1-q^i) from remainder,
/// which means: remainder[k] -= c, remainder[k+i] += c.
///
/// If after all terms the remainder is zero, return Some(quotient).
/// Otherwise return None.
fn try_divide_by_1_minus_q_i(f: &FormalPowerSeries, i: i64) -> Option<FormalPowerSeries> {
    // Work with a mutable copy of coefficients as a BTreeMap
    let mut remainder: BTreeMap<i64, QRat> = f.coefficients.clone();
    let mut quotient: BTreeMap<i64, QRat> = BTreeMap::new();
    let trunc = f.truncation_order();
    let variable = f.variable();

    // Process terms in ascending order
    // We iterate by pulling the smallest key repeatedly
    loop {
        // Find the smallest nonzero term in remainder
        let entry = {
            let mut first = None;
            for (k, v) in remainder.iter() {
                if !v.is_zero() {
                    first = Some((*k, v.clone()));
                    break;
                }
            }
            first
        };

        let (k, c) = match entry {
            Some(pair) => pair,
            None => break, // remainder is zero -- success
        };

        // Set quotient[k] += c
        let qentry = quotient.entry(k).or_insert_with(QRat::zero);
        *qentry = qentry.clone() + c.clone();

        // remainder[k] -= c (should become zero)
        remainder.remove(&k);

        // remainder[k+i] += c
        let ki = k + i;
        if ki < trunc {
            let rentry = remainder.entry(ki).or_insert_with(QRat::zero);
            *rentry = rentry.clone() + c;
            if rentry.is_zero() {
                remainder.remove(&ki);
            }
        }
        // If ki >= trunc, we're dividing a polynomial that extends beyond truncation.
        // For a true polynomial (all terms < trunc), this shouldn't happen if division is exact.
        // But if it does, the division may still work -- we just lose the high-order term.
    }

    // Clean up zero entries from quotient
    quotient.retain(|_, v| !v.is_zero());

    // Clean up remainder: any residual zeros
    remainder.retain(|_, v| !v.is_zero());

    if remainder.is_empty() {
        Some(FormalPowerSeries::from_coeffs(variable, quotient, trunc))
    } else {
        None
    }
}

/// Scale an FPS by a rational number.
fn scale_fps(f: &FormalPowerSeries, s: &QRat) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(f.variable(), f.truncation_order());
    for (&k, v) in f.iter() {
        let scaled = s.clone() * v.clone();
        result.set_coeff(k, scaled);
    }
    result
}

/// Return the highest exponent with a nonzero coefficient (polynomial degree).
fn poly_degree(f: &FormalPowerSeries) -> Option<i64> {
    f.iter().last().map(|(&k, _)| k)
}

/// Stub for two-variable q-factoring (zqfactor).
///
/// Garvan documents this as unreliable. Returns a non-exact factorization
/// with no factors extracted.
///
/// TODO: Implement proper two-variable factoring if needed for advanced use cases.
#[allow(dead_code)]
pub fn zqfactor(_f: &FormalPowerSeries) -> QFactorization {
    QFactorization {
        factors: BTreeMap::new(),
        scalar: QRat::one(),
        is_exact: false,
    }
}
