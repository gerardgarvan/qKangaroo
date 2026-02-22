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

/// Result of factoring a bivariate (z,q)-polynomial into product form.
///
/// Represents: `scalar * prod (1 - z^{z_pow} * q^{q_pow})^{mult}`
/// plus any remaining unfactored (1-q^i) factors.
#[derive(Clone, Debug)]
pub struct ZQFactorization {
    /// Bivariate factors: (z_power, q_power) -> multiplicity.
    /// A factor (z_pow, q_pow) represents (1 - z^{z_pow} * q^{q_pow}).
    pub factors: Vec<(i64, i64, i64)>,
    /// Pure q-factors extracted: i -> multiplicity for (1-q^i)^mult.
    pub q_factors: BTreeMap<i64, i64>,
    /// Overall rational scalar.
    pub scalar: QRat,
    /// Whether the factorization fully accounts for the input.
    pub is_exact: bool,
}

/// Factor a bivariate (z,q)-series into products of (1-z^a*q^b) terms.
///
/// Given f(z,q) as a BivariateSeries (Laurent polynomial in z with FPS
/// coefficients in q), attempts to decompose it as:
///
///   f(z,q) = scalar * prod (1 - z^{a_i} * q^{b_i})^{m_i}
///
/// The algorithm uses trial division in the z-direction: for each candidate
/// factor (1-z*q^a), it performs polynomial division on the z-coefficients
/// (each of which is an FPS in q).
pub fn zqfactor(f: &crate::series::bivariate::BivariateSeries) -> ZQFactorization {
    use crate::series::bivariate::BivariateSeries;

    if f.is_zero() {
        return ZQFactorization {
            factors: Vec::new(),
            q_factors: BTreeMap::new(),
            scalar: QRat::zero(),
            is_exact: true,
        };
    }

    let trunc = f.truncation_order();
    let inner_var = f.inner_variable;

    let mut current = f.clone();
    let mut factors: Vec<(i64, i64, i64)> = Vec::new();
    let mut scalar = QRat::one();

    // Extract scalar from the constant-in-z term at q^0
    if let Some(z0_fps) = current.terms.get(&0) {
        let c0 = z0_fps.coeff(0);
        if !c0.is_zero() && c0 != QRat::one() {
            let inv = QRat::one() / c0.clone();
            scalar = c0;
            let mut new_terms = BTreeMap::new();
            for (&z_exp, fps) in &current.terms {
                new_terms.insert(z_exp, scale_fps(fps, &inv));
            }
            current = BivariateSeries {
                outer_variable: current.outer_variable.clone(),
                terms: new_terms,
                inner_variable: inner_var,
                truncation_order: trunc,
            };
        }
    }

    let max_q_degree = trunc - 1;

    // Interleave positive and negative z-factor extraction.
    // For each q-power a, try extracting both (1-z*q^a) and (1-z^{-1}*q^a)
    // before moving to a+1. This ensures factors are found regardless of
    // the order they appear in the product.

    // First handle a=0: only (1-z), since (1-z^{-1}) = -(1/z)(z-1) is related
    loop {
        match try_bivariate_divide_pos(&current, 0, inner_var, trunc) {
            Some(quotient) => {
                if let Some(entry) = factors.iter_mut().find(|(zp, qp, _)| *zp == 1 && *qp == 0) {
                    entry.2 += 1;
                } else {
                    factors.push((1, 0, 1));
                }
                current = quotient;
            }
            None => break,
        }
        if current.is_zero() || current.terms.len() <= 1 { break; }
    }

    // For a >= 1, try both (1-z*q^a) and (1-z^{-1}*q^a) at each level
    for a in 1..=max_q_degree {
        if current.is_zero() || current.terms.len() <= 1 { break; }

        // Keep extracting at this level until neither factor divides
        let mut found_any = true;
        while found_any {
            found_any = false;

            // Try (1-z*q^a)
            loop {
                match try_bivariate_divide_pos(&current, a, inner_var, trunc) {
                    Some(quotient) => {
                        if let Some(entry) = factors.iter_mut().find(|(zp, qp, _)| *zp == 1 && *qp == a) {
                            entry.2 += 1;
                        } else {
                            factors.push((1, a, 1));
                        }
                        current = quotient;
                        found_any = true;
                    }
                    None => break,
                }
                if current.is_zero() || current.terms.len() <= 1 { break; }
            }

            // Try (1-z^{-1}*q^a)
            loop {
                match try_bivariate_divide_neg(&current, a, inner_var, trunc) {
                    Some(quotient) => {
                        if let Some(entry) = factors.iter_mut().find(|(zp, qp, _)| *zp == -1 && *qp == a) {
                            entry.2 += 1;
                        } else {
                            factors.push((-1, a, 1));
                        }
                        current = quotient;
                        found_any = true;
                    }
                    None => break,
                }
                if current.is_zero() || current.terms.len() <= 1 { break; }
            }
        }
    }

    // Check if what remains is purely a function of q (z^0 term only)
    let mut q_factors_map = BTreeMap::new();
    let remaining_is_q_only = current.terms.len() <= 1
        && current.terms.keys().all(|&k| k == 0);

    if remaining_is_q_only {
        if let Some(q_fps) = current.terms.get(&0) {
            if !q_fps.is_zero() && q_fps.coeff(0) != QRat::zero() {
                // Try to factor the remaining q-polynomial
                let qf = qfactor(q_fps);
                q_factors_map = qf.factors;
                scalar = scalar * qf.scalar;
            }
        }
    }

    let is_exact = remaining_is_q_only;

    ZQFactorization {
        factors,
        q_factors: q_factors_map,
        scalar,
        is_exact,
    }
}

/// Try to divide a BivariateSeries by (1 - z * q^a).
///
/// Division algorithm for f = g * (1 - z*q^a):
///   c_k(q) = d_k(q) - q^a * d_{k-1}(q)
/// => d_k(q) = c_k(q) + q^a * d_{k-1}(q)
///
/// Working from lowest z-power upward. If the remainder at the
/// highest z-power is zero, the division is exact.
fn try_bivariate_divide_pos(
    f: &crate::series::bivariate::BivariateSeries,
    a: i64,
    inner_var: crate::symbol::SymbolId,
    trunc: i64,
) -> Option<crate::series::bivariate::BivariateSeries> {
    use crate::series::arithmetic;
    use crate::series::bivariate::BivariateSeries;

    if f.terms.is_empty() {
        return None;
    }

    let z_lo = *f.terms.keys().next()?;
    let z_hi = *f.terms.keys().next_back()?;

    if z_hi <= z_lo {
        return None;
    }

    let zero_fps = FormalPowerSeries::zero(inner_var, trunc);

    // g has z-powers from z_lo to z_hi-1
    let mut d: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();

    // d[z_lo] = c[z_lo]
    let c_lo = f.terms.get(&z_lo).unwrap_or(&zero_fps);
    d.insert(z_lo, c_lo.clone());

    // For k = z_lo+1 to z_hi-1: d[k] = c[k] + q^a * d[k-1]
    for k in (z_lo + 1)..z_hi {
        let c_k = f.terms.get(&k).unwrap_or(&zero_fps);
        let d_prev = d.get(&(k - 1)).unwrap_or(&zero_fps);
        let shifted = shift_fps_exponents(d_prev, a, trunc);
        let dk = arithmetic::add(c_k, &shifted);
        if !dk.is_zero() {
            d.insert(k, dk);
        }
    }

    // Check: c[z_hi] + q^a * d[z_hi-1] should be zero
    let c_hi = f.terms.get(&z_hi).unwrap_or(&zero_fps);
    let d_prev = d.get(&(z_hi - 1)).unwrap_or(&zero_fps);
    let shifted = shift_fps_exponents(d_prev, a, trunc);
    let remainder = arithmetic::add(c_hi, &shifted);

    if remainder.is_zero() {
        // Clean up zero entries
        d.retain(|_, v| !v.is_zero());
        Some(BivariateSeries {
            outer_variable: f.outer_variable.clone(),
            terms: d,
            inner_variable: inner_var,
            truncation_order: trunc,
        })
    } else {
        None
    }
}

/// Try to divide a BivariateSeries by (1 - z^{-1} * q^a).
///
/// Division algorithm for f = g * (1 - z^{-1}*q^a):
///   c_k(q) = d_k(q) - q^a * d_{k+1}(q)
/// => d_k(q) = c_k(q) + q^a * d_{k+1}(q)
///
/// Working from highest z-power downward.
fn try_bivariate_divide_neg(
    f: &crate::series::bivariate::BivariateSeries,
    a: i64,
    inner_var: crate::symbol::SymbolId,
    trunc: i64,
) -> Option<crate::series::bivariate::BivariateSeries> {
    use crate::series::arithmetic;
    use crate::series::bivariate::BivariateSeries;

    if f.terms.is_empty() {
        return None;
    }

    let z_lo = *f.terms.keys().next()?;
    let z_hi = *f.terms.keys().next_back()?;

    if z_hi <= z_lo {
        return None;
    }

    let zero_fps = FormalPowerSeries::zero(inner_var, trunc);

    // g has z-powers from z_lo+1 to z_hi
    let mut d: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();

    // d[z_hi] = c[z_hi]
    let c_hi = f.terms.get(&z_hi).unwrap_or(&zero_fps);
    d.insert(z_hi, c_hi.clone());

    // For k = z_hi-1 down to z_lo+1: d[k] = c[k] + q^a * d[k+1]
    for k in ((z_lo + 1)..z_hi).rev() {
        let c_k = f.terms.get(&k).unwrap_or(&zero_fps);
        let d_next = d.get(&(k + 1)).unwrap_or(&zero_fps);
        let shifted = shift_fps_exponents(d_next, a, trunc);
        let dk = arithmetic::add(c_k, &shifted);
        if !dk.is_zero() {
            d.insert(k, dk);
        }
    }

    // Check: c[z_lo] + q^a * d[z_lo+1] should be zero
    let c_lo = f.terms.get(&z_lo).unwrap_or(&zero_fps);
    let d_next = d.get(&(z_lo + 1)).unwrap_or(&zero_fps);
    let shifted = shift_fps_exponents(d_next, a, trunc);
    let remainder = arithmetic::add(c_lo, &shifted);

    if remainder.is_zero() {
        d.retain(|_, v| !v.is_zero());
        Some(BivariateSeries {
            outer_variable: f.outer_variable.clone(),
            terms: d,
            inner_variable: inner_var,
            truncation_order: trunc,
        })
    } else {
        None
    }
}

/// Shift all exponents in an FPS by `shift`: f(q) -> q^shift * f(q).
fn shift_fps_exponents(f: &FormalPowerSeries, shift: i64, trunc: i64) -> FormalPowerSeries {
    let mut coeffs = BTreeMap::new();
    for (&k, v) in f.iter() {
        let new_k = k + shift;
        if new_k < trunc {
            coeffs.insert(new_k, v.clone());
        }
    }
    FormalPowerSeries::from_coeffs(f.variable(), coeffs, trunc)
}
