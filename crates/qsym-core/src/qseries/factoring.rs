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

    // Get the initial degree of the normalized polynomial
    let initial_deg = match poly_degree(&current) {
        Some(d) if d > 0 => d,
        _ => {
            // Already a constant -- nothing to factor
            return QFactorization {
                factors,
                scalar,
                is_exact: current.coeff(0) == QRat::one(),
            };
        }
    };

    // Try dividing by (1-q^i) for i from largest down to smallest.
    // This ensures that we extract (1-q^3) before (1-q), preventing
    // the smaller factors from "stealing" divisibility that belongs
    // to larger cyclotomic factors (e.g., (1-q^2) = (1-q)(1+q)).
    //
    // We re-check the degree after each extraction since it may shrink.
    let mut i = initial_deg;
    while i >= 1 {
        match try_divide_by_1_minus_q_i(&current, i) {
            Some(quotient) => {
                *factors.entry(i).or_insert(0) += 1;
                current = quotient;
                // After extraction, reset i to the new degree (or stay at current i)
                // to check if (1-q^i) divides again.
                let new_deg = poly_degree(&current).unwrap_or(0);
                if new_deg < i {
                    i = new_deg;
                }
                // Otherwise stay at same i to try extracting again
            }
            None => {
                i -= 1;
            }
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
/// This performs POLYNOMIAL division, not series division. The quotient must
/// be a polynomial (finite number of terms, all within the original degree bounds).
///
/// Algorithm: process terms from lowest to highest. For each term c*q^k,
/// set quotient[k] += c, then "carry" c to remainder[k+i]. If the carry
/// would exceed the polynomial degree bound, the division is not exact and
/// we return None.
///
/// If after all terms the remainder is zero, return Some(quotient).
/// Otherwise return None.
fn try_divide_by_1_minus_q_i(f: &FormalPowerSeries, i: i64) -> Option<FormalPowerSeries> {
    // For polynomial division by (1-q^i), the quotient degree should be
    // at most deg(f) - i. If deg(f) < i, division cannot be exact (unless f=1 and i irrelevant).
    let f_deg = match f.iter().last() {
        Some((&k, _)) => k,
        None => return None, // zero polynomial
    };

    // If the polynomial degree is less than i, (1-q^i) cannot divide it
    // (unless f is a constant, but a nonzero constant is not divisible by (1-q^i))
    if f_deg < i {
        return None;
    }

    // The maximum degree the quotient can have
    let max_quotient_deg = f_deg - i;

    // Work with a mutable copy of coefficients as a BTreeMap
    let mut remainder: BTreeMap<i64, QRat> = f.coefficients.clone();
    let mut quotient: BTreeMap<i64, QRat> = BTreeMap::new();
    let trunc = f.truncation_order();
    let variable = f.variable();

    // Process terms in ascending order
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

        // If this term would put us beyond the polynomial quotient degree, fail
        if k > max_quotient_deg {
            return None;
        }

        // Set quotient[k] = c
        quotient.insert(k, c.clone());

        // remainder[k] -= c (becomes zero, remove it)
        remainder.remove(&k);

        // remainder[k+i] += c  (the "carry")
        let ki = k + i;
        if ki < trunc {
            let rentry = remainder.entry(ki).or_insert_with(QRat::zero);
            *rentry = rentry.clone() + c;
            if rentry.is_zero() {
                remainder.remove(&ki);
            }
        }
    }

    // Clean up zero entries from quotient
    quotient.retain(|_, v| !v.is_zero());

    // Clean up remainder
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
