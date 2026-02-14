//! Appell-Lerch sums, universal mock theta functions, and Zwegers completions.
//!
//! This module provides:
//! - [`appell_lerch_m`]: The Appell-Lerch bilateral sum m(q^a, q, q^b)
//! - [`appell_lerch_bilateral`]: The raw bilateral sum (without j(z;q) normalization)
//! - [`universal_mock_theta_g3`]: Universal mock theta function g3(q^a, q)
//! - [`universal_mock_theta_g2`]: Universal mock theta function g2(q^a, q)
//! - [`ZwegersCompletion`]: Symbolic representation of Zwegers completions
//!
//! # Mathematical Background
//!
//! The Appell-Lerch sum m(x,q,z) following Hickerson-Mortenson:
//! ```text
//! m(x, q, z) = (1/j(z;q)) * sum_{r in Z} (-1)^r * q^{r(r-1)/2} * z^r / (1 - x*q^r*z)
//! ```
//! where j(z;q) = (z;q)_inf * (q/z;q)_inf * (q;q)_inf is the Jacobi theta function.
//!
//! When x and z are specialized to powers of q (x = q^a, z = q^b with integer a, b),
//! the Jacobi theta j(q^b;q) vanishes for all integer b (since one of the infinite
//! products always has a factor (1-q^0) = 0). Therefore, the full normalized m(x,q,z)
//! is not directly computable for integer parameters via FPS. Instead, we provide:
//!
//! 1. [`appell_lerch_bilateral`]: The bilateral sum WITHOUT j(z;q) normalization
//! 2. [`appell_lerch_m`]: Wrapper that computes the bilateral sum (useful for verifying
//!    identities where the j(z;q) factor cancels from both sides)
//!
//! The universal mock theta functions g2 and g3 (Gordon-McIntosh) have similar
//! denominator degeneracies for integer parameters. For x = q^a with integer a >= 2,
//! the Pochhammer product (q/x;q)_{n+1} = (q^{1-a};q)_{n+1} vanishes for n >= a-1.
//! We sum only the non-degenerate terms (n = 0 to max_n where denominators are nonzero).

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::symbol::SymbolId;

use super::{QMonomial, PochhammerOrder, aqprod};

/// Compute 1/(1 - q^k) as a formal power series (geometric series expansion).
///
/// - If k > 0: sum_{m>=0} q^{mk} for mk < truncation_order
/// - If k < 0: 1/(1 - q^{-|k|}) = -q^{|k|} * sum_{m>=0} q^{m|k|}
/// - If k == 0: panics (pole at 1/(1-1))
fn geometric_series_q_power(
    k: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    assert!(k != 0, "geometric_series_q_power: k=0 is a pole (1/(1-1) undefined)");

    if k > 0 {
        // 1/(1 - q^k) = 1 + q^k + q^{2k} + ...
        let mut result = FormalPowerSeries::zero(variable, truncation_order);
        let mut exp = 0i64;
        while exp < truncation_order {
            result.set_coeff(exp, QRat::one());
            exp += k;
        }
        result
    } else {
        // k < 0, let ak = |k|
        // 1/(1 - q^k) = 1/(1 - q^{-ak})
        // = q^{ak} / (q^{ak} - 1) = -q^{ak} / (1 - q^{ak})
        // = -q^{ak} * sum_{m>=0} q^{m*ak}
        let ak = -k;
        let mut result = FormalPowerSeries::zero(variable, truncation_order);
        let mut exp = ak;
        while exp < truncation_order {
            result.set_coeff(exp, -QRat::one());
            exp += ak;
        }
        result
    }
}

/// Compute the raw bilateral sum of the Appell-Lerch function (without j(z;q) normalization).
///
/// ```text
/// S(q^a, q, q^b) = sum_{r in Z} (-1)^r * q^{r(r-1)/2 + b*r} / (1 - q^{a+r+b})
/// ```
///
/// This is the bilateral sum appearing in the Appell-Lerch definition:
/// m(x, q, z) = S(x, q, z) / j(z; q)
///
/// For integer parameters, j(z;q) = 0 and the full m is not directly computable,
/// but the bilateral sum S is well-defined (skipping r values where a+r+b = 0).
///
/// # Arguments
///
/// * `a_pow` - Integer such that x = q^{a_pow}
/// * `z_pow` - Integer such that z = q^{z_pow}
/// * `variable` - The SymbolId for the series variable q
/// * `truncation_order` - Compute to O(q^truncation_order)
pub fn appell_lerch_bilateral(
    a_pow: i64,
    z_pow: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let mut bilateral_sum = FormalPowerSeries::zero(variable, truncation_order);

    // Helper: compute a term (-1)^r * q^{q_exp} / (1 - q^{denom_pow}) and add to sum.
    // When q_exp < 0, the geometric series needs extended truncation to ensure
    // all contributions to coefficients [0, truncation_order) are captured.
    let add_term = |bilateral: &mut FormalPowerSeries, q_exp: i64, denom_pow: i64, sign: QRat| {
        // The product q^{q_exp} * geometric_series needs the geometric series
        // to cover exponents up to (truncation_order - q_exp) so that the
        // product covers [0, truncation_order) after shifting by q_exp.
        let effective_trunc = if q_exp < 0 {
            truncation_order - q_exp  // extend to compensate for negative shift
        } else {
            truncation_order
        };

        let geom = geometric_series_q_power(denom_pow, variable, effective_trunc);
        let numer = FormalPowerSeries::monomial(variable, sign, q_exp, effective_trunc);
        let term = arithmetic::mul(&numer, &geom);

        // Truncate the term back to the target truncation_order
        let mut truncated_term = FormalPowerSeries::zero(variable, truncation_order);
        for (&k, v) in term.iter() {
            if k < truncation_order {
                truncated_term.set_coeff(k, v.clone());
            }
        }
        *bilateral = arithmetic::add(bilateral, &truncated_term);
    };

    // Positive direction: r = 0, 1, 2, ...
    for r in 0i64.. {
        let q_exp = r * (r - 1) / 2 + z_pow * r;
        if q_exp >= truncation_order && r > 0 {
            break;
        }
        if q_exp >= truncation_order {
            break;
        }

        let sign: QRat = if r % 2 == 0 { QRat::one() } else { -QRat::one() };

        let denom_pow = a_pow + r + z_pow;
        if denom_pow == 0 {
            continue;
        }

        add_term(&mut bilateral_sum, q_exp, denom_pow, sign);
    }

    // Negative direction: r = -1, -2, -3, ...
    for r_abs in 1i64.. {
        let r = -r_abs;
        let q_exp = r * (r - 1) / 2 + z_pow * r;
        if q_exp >= truncation_order {
            break;
        }

        let sign: QRat = if r_abs % 2 == 0 { QRat::one() } else { -QRat::one() };

        let denom_pow = a_pow + r + z_pow;
        if denom_pow == 0 {
            continue;
        }

        add_term(&mut bilateral_sum, q_exp, denom_pow, sign);
    }

    bilateral_sum
}

/// Compute the Appell-Lerch sum m(q^{a_pow}, q, q^{z_pow}) as a formal power series.
///
/// For integer specializations (z = q^{z_pow}), the Jacobi theta j(z;q) vanishes,
/// so the full normalized m(x,q,z) is not directly available. This function returns
/// the raw bilateral sum, which is the numerator of the Appell-Lerch expression.
///
/// This is equivalent to [`appell_lerch_bilateral`] and is useful for:
/// - Verifying identities where j(z;q) factors cancel from both sides
/// - Cross-checking mock theta function representations
/// - Computing structural relations between Appell-Lerch sums
///
/// # Arguments
///
/// * `a_pow` - Integer such that x = q^{a_pow}
/// * `z_pow` - Integer such that z = q^{z_pow}
/// * `variable` - The SymbolId for the series variable q
/// * `truncation_order` - Compute to O(q^truncation_order)
pub fn appell_lerch_m(
    a_pow: i64,
    z_pow: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    appell_lerch_bilateral(a_pow, z_pow, variable, truncation_order)
}

/// Compute the universal mock theta function g3(q^{a_pow}, q) as a formal power series.
///
/// ```text
/// g3(x, q) = sum_{n>=0} q^{n(n+1)/2} / [(x;q)_{n+1} * (q/x;q)_{n+1}]
/// ```
///
/// For x = q^{a_pow} with integer a_pow >= 2, the denominator (q^{1-a};q)_{n+1}
/// has negative-exponent factors that we handle algebraically:
/// ```text
/// (q^{1-a};q)_{n+1} = (-1)^{n+1} * q^{-S} * prod_{k=0}^n (1 - q^{a-1-k})
/// ```
/// where S = (n+1)(a-1) - n(n+1)/2. The sum includes only terms where the
/// denominator is non-degenerate (n < a-1).
///
/// # Arguments
///
/// * `a_pow` - Integer such that x = q^{a_pow} (must be >= 2 for nontrivial result)
/// * `variable` - The SymbolId for the series variable q
/// * `truncation_order` - Compute to O(q^truncation_order)
pub fn universal_mock_theta_g3(
    a_pow: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);

    let max_valid_n = compute_max_valid_n(a_pow);

    // For a_pow >= 2: use the algebraic identity to avoid negative-exponent FPS.
    // For each valid n (0..=max_n):
    //   term = (-1)^{n+1} * q^{(n+1)(a-1)} / [(q^a;q)_{n+1} * prod_{k=0}^n (1-q^{a-1-k})]
    //
    // We maintain running products incrementally:
    //   denom1 = (q^a;q)_{n+1}: factors (1-q^a), (1-q^{a+1}), ... all positive exponents
    //   denom2_pos = prod_{k=0}^n (1-q^{a-1-k}): factors (1-q^{a-1}), (1-q^{a-2}), ...

    if a_pow <= 1 {
        // For a_pow <= 1, no valid terms exist (max_valid_n <= -1)
        return result;
    }

    // Initialize for n=0:
    // denom1 = (q^a;q)_1 = (1 - q^a)
    // denom2_pos = (1 - q^{a-1})
    let mut denom1 = {
        let mut f = FormalPowerSeries::one(variable, truncation_order);
        f.set_coeff(a_pow, -QRat::one());
        f
    };
    let mut denom2_pos = {
        let mut f = FormalPowerSeries::one(variable, truncation_order);
        if a_pow - 1 > 0 && a_pow - 1 < truncation_order {
            f.set_coeff(a_pow - 1, -QRat::one());
        }
        f
    };

    for n in 0i64.. {
        // Check valid range
        if let Some(max_n) = max_valid_n {
            if n > max_n {
                break;
            }
        }

        // Numerator power: (n+1)*(a-1)
        let q_exp = (n + 1) * (a_pow - 1);
        if q_exp >= truncation_order {
            break;
        }

        // Sign: (-1)^{n+1}
        let sign: QRat = if (n + 1) % 2 == 0 { QRat::one() } else { -QRat::one() };

        // Denominator = denom1 * denom2_pos (all positive-exponent FPS)
        let denom = arithmetic::mul(&denom1, &denom2_pos);
        if denom.is_zero() || denom.coeff(0).is_zero() {
            break;
        }

        let denom_inv = arithmetic::invert(&denom);
        let numer = FormalPowerSeries::monomial(variable, sign, q_exp, truncation_order);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);

        // Update for next iteration (n -> n+1):
        // denom1: (q^a;q)_{n+2} = denom1 * (1 - q^{a+n+1})
        let f1_exp = a_pow + n + 1;
        if f1_exp > 0 && f1_exp < truncation_order {
            let mut f1 = FormalPowerSeries::one(variable, truncation_order);
            f1.set_coeff(f1_exp, -QRat::one());
            denom1 = arithmetic::mul(&denom1, &f1);
        }

        // denom2_pos: multiply by (1 - q^{a-2-n})
        // At n, the next factor index is k=n+1, giving exponent a-1-(n+1) = a-2-n.
        let f2_exp = a_pow - 2 - n;
        if f2_exp > 0 && f2_exp < truncation_order {
            let mut f2 = FormalPowerSeries::one(variable, truncation_order);
            f2.set_coeff(f2_exp, -QRat::one());
            denom2_pos = arithmetic::mul(&denom2_pos, &f2);
        } else if f2_exp == 0 {
            // Factor is (1-1) = 0 -- this means next n is invalid, but we break via max_valid_n
            break;
        }
        // If f2_exp < 0, we've exhausted valid factors (shouldn't happen with max_valid_n)
    }

    result
}

/// Compute the maximum valid n for g3/g2 denominator products.
///
/// Returns None if no restriction (all n valid), or Some(max_n) if
/// the denominator vanishes for n > max_n.
fn compute_max_valid_n(a_pow: i64) -> Option<i64> {
    // (q^a;q)_{n+1} has factor (1-q^{a+k}) for k=0..n
    // Vanishes when a + k = 0, i.e., k = -a. Need 0 <= k <= n, so n >= -a.
    // This applies when a <= 0: vanishes for n >= -a, so max_n = -a - 1.
    //
    // (q^{1-a};q)_{n+1} has factor (1-q^{1-a+k}) for k=0..n
    // Vanishes when 1-a+k = 0, i.e., k = a-1. Need 0 <= k <= n, so n >= a-1.
    // This applies when a >= 2: vanishes for n >= a-1, so max_n = a - 2.
    //
    // For a = 1: both restrictions give k = -1 (not in range) and k = 0.
    // (q;q)_{n+1}: a=1, factor (1-q^{1+k}), never zero for k >= 0. Fine.
    // (1;q)_{n+1}: 1-a=0, factor (1-q^{0+k}). k=0: (1-1)=0. So n >= 0 causes vanishing.
    // Actually that's the initial factor! (q^0;q)_1 = (1-1) = 0.
    // So for a = 1, max_n = -1 (no valid terms). But we handle this via the zero-product check.
    //
    // For a = 0: (q^0;q)_{n+1} = 0 for all n >= 0. No valid terms.

    let limit1 = if a_pow <= 0 {
        Some(-a_pow - 1) // from first Pochhammer
    } else {
        None
    };
    let limit2 = if a_pow >= 2 {
        Some(a_pow - 2) // from second Pochhammer
    } else if a_pow == 1 {
        Some(-1i64) // special: (q^0;q)_1 = 0, no valid terms
    } else {
        None
    };

    match (limit1, limit2) {
        (None, None) => None,
        (Some(l), None) => {
            if l < 0 { Some(-1) } else { Some(l) }
        }
        (None, Some(l)) => {
            if l < 0 { Some(-1) } else { Some(l) }
        }
        (Some(l1), Some(l2)) => {
            let l = l1.min(l2);
            if l < 0 { Some(-1) } else { Some(l) }
        }
    }
}

/// Compute the universal mock theta function g2(q^{a_pow}, q) as a formal power series.
///
/// ```text
/// g2(x, q) = x^{-1} * (-q;q)_inf * sum_{n>=0} q^{n(n+1)/2} * (-q;q)_n / [(x;q)_{n+1} * (q/x;q)_{n+1}]
/// ```
///
/// Like g3, uses the algebraic identity for negative-exponent Pochhammer factors.
/// The q^{-a} prefactor is applied as a shift in the FPS.
///
/// # Arguments
///
/// * `a_pow` - Integer such that x = q^{a_pow} (must be >= 2 for nontrivial result)
/// * `variable` - The SymbolId for the series variable q
/// * `truncation_order` - Compute to O(q^truncation_order)
pub fn universal_mock_theta_g2(
    a_pow: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    if a_pow <= 1 {
        return FormalPowerSeries::zero(variable, truncation_order);
    }

    // Compute (-q;q)_inf
    let neg_q = QMonomial::new(-QRat::one(), 1);
    let neg_q_inf = aqprod(&neg_q, variable, PochhammerOrder::Infinite, truncation_order);

    let max_valid_n = compute_max_valid_n(a_pow);

    // Same positive-exponent approach as g3 for the denominator.
    // For each valid n:
    //   inner_term = (-1)^{n+1} * q^{(n+1)(a-1)} * (-q;q)_n / [denom1 * denom2_pos]

    let mut denom1 = {
        let mut f = FormalPowerSeries::one(variable, truncation_order);
        f.set_coeff(a_pow, -QRat::one());
        f
    };
    let mut denom2_pos = {
        let mut f = FormalPowerSeries::one(variable, truncation_order);
        if a_pow - 1 > 0 && a_pow - 1 < truncation_order {
            f.set_coeff(a_pow - 1, -QRat::one());
        }
        f
    };

    let mut numer_poch = FormalPowerSeries::one(variable, truncation_order);
    let mut inner_sum = FormalPowerSeries::zero(variable, truncation_order);

    for n in 0i64.. {
        if let Some(max_n) = max_valid_n {
            if n > max_n {
                break;
            }
        }

        let q_exp = (n + 1) * (a_pow - 1);
        if q_exp >= truncation_order {
            break;
        }

        let sign: QRat = if (n + 1) % 2 == 0 { QRat::one() } else { -QRat::one() };

        let denom = arithmetic::mul(&denom1, &denom2_pos);
        if denom.is_zero() || denom.coeff(0).is_zero() {
            break;
        }

        let denom_inv = arithmetic::invert(&denom);
        let numer = FormalPowerSeries::monomial(variable, sign, q_exp, truncation_order);
        let term = arithmetic::mul(&arithmetic::mul(&numer, &numer_poch), &denom_inv);
        inner_sum = arithmetic::add(&inner_sum, &term);

        // Update numer_poch: (-q;q)_{n+1} = (-q;q)_n * (1 + q^{n+1})
        let m = n + 1;
        let mut npf = FormalPowerSeries::one(variable, truncation_order);
        if m < truncation_order {
            npf.set_coeff(m, QRat::one());
        }
        numer_poch = arithmetic::mul(&numer_poch, &npf);

        // Update denom1
        let f1_exp = a_pow + n + 1;
        if f1_exp > 0 && f1_exp < truncation_order {
            let mut f1 = FormalPowerSeries::one(variable, truncation_order);
            f1.set_coeff(f1_exp, -QRat::one());
            denom1 = arithmetic::mul(&denom1, &f1);
        }

        // Update denom2_pos
        let f2_exp = a_pow - 2 - n;
        if f2_exp > 0 && f2_exp < truncation_order {
            let mut f2 = FormalPowerSeries::one(variable, truncation_order);
            f2.set_coeff(f2_exp, -QRat::one());
            denom2_pos = arithmetic::mul(&denom2_pos, &f2);
        } else if f2_exp <= 0 {
            break;
        }
    }

    // Multiply by (-q;q)_inf
    let mut result = arithmetic::mul(&neg_q_inf, &inner_sum);

    // Apply q^{-a_pow} shift
    if a_pow != 0 {
        result = arithmetic::shift(&result, -a_pow);
    }

    result
}

/// Symbolic representation of a Zwegers completion for a mock theta function.
///
/// Zwegers showed that mock theta functions can be "completed" by adding a
/// non-holomorphic correction term to form harmonic Maass forms. Since the
/// correction involves the complementary error function (erfc), which is
/// transcendental, this struct provides a symbolic container rather than
/// attempting exact computation.
///
/// The completed function is:
/// ```text
/// hat_f(tau) = f(q) + R(tau)
/// ```
/// where f(q) is the holomorphic part (the mock theta function) and R(tau)
/// is the non-holomorphic correction involving erfc.
#[derive(Clone, Debug)]
pub struct ZwegersCompletion {
    /// Name of the mock theta function being completed
    pub mock_theta_name: String,
    /// The holomorphic part (the mock theta function itself as FPS)
    pub holomorphic_part: FormalPowerSeries,
    /// Description of the non-holomorphic correction R(z; tau)
    pub correction_description: String,
    /// Known modular weight of the completed form (numerator, denominator)
    pub weight: (i64, i64),
    /// Known modular level
    pub level: i64,
}

impl ZwegersCompletion {
    /// Create a completion for a third-order mock theta function.
    ///
    /// Third-order mock theta completions have weight 1/2 and level 2.
    /// The non-holomorphic part involves a period integral of a weight-3/2
    /// unary theta function.
    pub fn third_order(name: &str, holomorphic: FormalPowerSeries) -> Self {
        Self {
            mock_theta_name: name.to_string(),
            holomorphic_part: holomorphic,
            correction_description: format!(
                "R(tau) = non-holomorphic Eichler integral of weight 3/2 unary theta \
                 function associated with third-order mock theta function {}. \
                 Involves sum over half-integers of sgn(n)*erfc(|n|*sqrt(2*Im(tau))).",
                name
            ),
            weight: (1, 2), // weight 1/2
            level: 2,
        }
    }

    /// Create a completion for a fifth-order mock theta function.
    ///
    /// Fifth-order mock theta completions have weight 1/2 with level 5.
    pub fn fifth_order(name: &str, holomorphic: FormalPowerSeries) -> Self {
        Self {
            mock_theta_name: name.to_string(),
            holomorphic_part: holomorphic,
            correction_description: format!(
                "R(tau) = non-holomorphic Eichler integral of weight 3/2 theta function \
                 associated with fifth-order mock theta function {}.",
                name
            ),
            weight: (1, 2), // weight 1/2
            level: 5,
        }
    }

    /// Create a completion with custom parameters.
    pub fn custom(
        name: &str,
        holomorphic: FormalPowerSeries,
        correction_desc: &str,
        weight: (i64, i64),
        level: i64,
    ) -> Self {
        Self {
            mock_theta_name: name.to_string(),
            holomorphic_part: holomorphic,
            correction_description: correction_desc.to_string(),
            weight,
            level,
        }
    }

    /// Verify that two completions have compatible holomorphic parts
    /// by checking a linear relation between them as FPS.
    ///
    /// Verifies that c1 * self.holomorphic_part + c2 * other.holomorphic_part
    /// matches a target FPS to the minimum truncation order.
    ///
    /// Returns true if the linear combination matches the target.
    pub fn verify_linear_relation(
        &self,
        other: &ZwegersCompletion,
        c1: &QRat,
        c2: &QRat,
        target: &FormalPowerSeries,
    ) -> bool {
        let part1 = arithmetic::scalar_mul(c1, &self.holomorphic_part);
        let part2 = arithmetic::scalar_mul(c2, &other.holomorphic_part);
        let combo = arithmetic::add(&part1, &part2);

        let diff = arithmetic::sub(&combo, target);
        diff.is_zero()
    }

    /// Check that this completion's holomorphic part is nonzero.
    pub fn is_nontrivial(&self) -> bool {
        !self.holomorphic_part.is_zero()
    }
}
