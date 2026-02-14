//! Appell-Lerch sums, universal mock theta functions, and Zwegers completions.
//!
//! This module provides:
//! - [`appell_lerch_m`]: The Appell-Lerch sum m(q^a, q, q^b) following Hickerson-Mortenson notation
//! - [`universal_mock_theta_g2`]: Universal mock theta function g2(q^a, q)
//! - [`universal_mock_theta_g3`]: Universal mock theta function g3(q^a, q)
//! - [`ZwegersCompletion`]: Symbolic representation of Zwegers completions for mock theta functions
//!
//! The Appell-Lerch sum m(x,q,z) is the canonical building block for all classical mock theta
//! functions. The universal mock theta functions g2 and g3 (Gordon-McIntosh) provide another
//! unified representation: even-order mock theta functions are specializations of g2, and
//! odd-order mock theta functions are specializations of g3.
//!
//! Zwegers completions are represented symbolically since the non-holomorphic correction involves
//! the complementary error function (erfc), which cannot be computed in exact rational arithmetic.

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
        // Multiply num and denom by q^{ak}: = q^{ak} / (q^{ak} - 1) = -q^{ak} / (1 - q^{ak})
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

/// Compute the Jacobi theta function j(q^z_pow; q) as an FPS.
///
/// j(z; q) = (z;q)_inf * (q/z;q)_inf * (q;q)_inf
///
/// For z = q^z_pow:
/// j(q^b; q) = (q^b;q)_inf * (q^{1-b};q)_inf * (q;q)_inf
fn jacobi_theta_j(
    z_pow: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let z_mono = QMonomial::q_power(z_pow);
    let qz_inv = QMonomial::q_power(1 - z_pow);
    let q_mono = QMonomial::q_power(1);

    let part1 = aqprod(&z_mono, variable, PochhammerOrder::Infinite, truncation_order);
    let part2 = aqprod(&qz_inv, variable, PochhammerOrder::Infinite, truncation_order);
    let part3 = aqprod(&q_mono, variable, PochhammerOrder::Infinite, truncation_order);

    arithmetic::mul(&arithmetic::mul(&part1, &part2), &part3)
}

/// Compute the Appell-Lerch sum m(q^{a_pow}, q, q^{z_pow}) as a formal power series.
///
/// Following Hickerson-Mortenson notation:
/// ```text
/// m(x, q, z) = (1/j(z;q)) * sum_{r in Z} (-1)^r * q^{r(r-1)/2} * z^r / (1 - x*q^r*z)
/// ```
///
/// Specializing x = q^{a_pow}, z = q^{z_pow}:
/// ```text
/// m(q^a, q, q^b) = (1/j(q^b;q)) * sum_{r in Z} (-1)^r * q^{r(r-1)/2 + b*r} / (1 - q^{a+r+b})
/// ```
///
/// # Restrictions
///
/// The parameter z_pow must be such that j(q^{z_pow}; q) is nonzero, which means z_pow should
/// not be 0 or 1 (and more generally not a non-negative integer <= 0 in the relevant sense).
/// In practice, j(q^b; q) = 0 when b is a non-positive integer or when 1-b is a non-positive
/// integer, because one of the infinite products has a vanishing first factor.
///
/// # Arguments
///
/// * `a_pow` - Integer such that x = q^{a_pow}
/// * `z_pow` - Integer such that z = q^{z_pow} (must yield nonzero j(z;q))
/// * `variable` - The SymbolId for the series variable q
/// * `truncation_order` - Compute to O(q^truncation_order)
pub fn appell_lerch_m(
    a_pow: i64,
    z_pow: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    // Compute j(z;q) and its inverse
    let j_z = jacobi_theta_j(z_pow, variable, truncation_order);
    assert!(
        !j_z.is_zero(),
        "appell_lerch_m: j(q^{};q) is zero -- z_pow={} gives degenerate Appell-Lerch sum",
        z_pow,
        z_pow
    );
    let j_z_inv = arithmetic::invert(&j_z);

    // Bilateral sum: positive part (r >= 0) + negative part (r < 0)
    let mut bilateral_sum = FormalPowerSeries::zero(variable, truncation_order);

    // Positive direction: r = 0, 1, 2, ...
    for r in 0i64.. {
        let q_exp = r * (r - 1) / 2 + z_pow * r;
        // Break if the minimum contribution exceeds truncation (for r > 0)
        if q_exp >= truncation_order && r > 0 {
            break;
        }
        // Skip if q_exp is already beyond truncation for r=0 (shouldn't happen since q_exp=0)
        if q_exp >= truncation_order {
            break;
        }

        let sign: QRat = if r % 2 == 0 { QRat::one() } else { -QRat::one() };

        let denom_pow = a_pow + r + z_pow;
        if denom_pow == 0 {
            // Pole in geometric series -- skip this r
            // In the full Appell-Lerch sum, these poles cancel with j(z;q)
            continue;
        }

        let geom = geometric_series_q_power(denom_pow, variable, truncation_order);
        let numer = FormalPowerSeries::monomial(variable, sign, q_exp, truncation_order);
        let term = arithmetic::mul(&numer, &geom);
        bilateral_sum = arithmetic::add(&bilateral_sum, &term);
    }

    // Negative direction: r = -1, -2, -3, ...
    for r_abs in 1i64.. {
        let r = -r_abs;
        let q_exp = r * (r - 1) / 2 + z_pow * r;
        // r*(r-1)/2 for negative r:
        // r=-1: (-1)(-2)/2 = 1
        // r=-2: (-2)(-3)/2 = 3
        // r=-3: (-3)(-4)/2 = 6
        // So q_exp = r_abs*(r_abs+1)/2 - z_pow*r_abs
        // This grows quadratically, so will eventually exceed truncation_order
        if q_exp >= truncation_order {
            break;
        }
        // For negative q_exp, the monomial constructor handles it (sets coeff if < trunc)
        // But we also need the geometric series terms to overlap with the valid range

        let sign: QRat = if r_abs % 2 == 0 { QRat::one() } else { -QRat::one() };

        let denom_pow = a_pow + r + z_pow;
        if denom_pow == 0 {
            continue;
        }

        let geom = geometric_series_q_power(denom_pow, variable, truncation_order);
        let numer = FormalPowerSeries::monomial(variable, sign, q_exp, truncation_order);
        let term = arithmetic::mul(&numer, &geom);
        bilateral_sum = arithmetic::add(&bilateral_sum, &term);
    }

    arithmetic::mul(&bilateral_sum, &j_z_inv)
}

/// Compute the universal mock theta function g3(q^{a_pow}, q) as a formal power series.
///
/// ```text
/// g3(x, q) = sum_{n>=0} q^{n(n+1)/2} / [(x;q)_{n+1} * (q/x;q)_{n+1}]
/// ```
///
/// For x = q^{a_pow}:
/// ```text
/// g3(q^a, q) = sum_{n>=0} q^{n(n+1)/2} / [(q^a;q)_{n+1} * (q^{1-a};q)_{n+1}]
/// ```
///
/// Gordon and McIntosh showed that all odd-order mock theta functions are
/// specializations of g3.
///
/// # Arguments
///
/// * `a_pow` - Integer such that x = q^{a_pow}
/// * `variable` - The SymbolId for the series variable q
/// * `truncation_order` - Compute to O(q^truncation_order)
pub fn universal_mock_theta_g3(
    a_pow: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);

    // Running denominator products, maintained incrementally:
    // denom1 = (q^a; q)_{n+1}, starting from (q^a; q)_1 = (1 - q^a)
    // denom2 = (q^{1-a}; q)_{n+1}, starting from (q^{1-a}; q)_1 = (1 - q^{1-a})
    //
    // Initialize for n=0: we need (q^a;q)_1 and (q^{1-a};q)_1
    let a_mono = QMonomial::q_power(a_pow);
    let qa_inv_mono = QMonomial::q_power(1 - a_pow);

    let mut denom1 = aqprod(&a_mono, variable, PochhammerOrder::Finite(1), truncation_order);
    let mut denom2 = aqprod(&qa_inv_mono, variable, PochhammerOrder::Finite(1), truncation_order);

    for n in 0i64.. {
        let q_exp = n * (n + 1) / 2;
        if q_exp >= truncation_order {
            break;
        }

        // term = q^{n(n+1)/2} / (denom1 * denom2)
        let denom = arithmetic::mul(&denom1, &denom2);
        let denom_inv = arithmetic::invert(&denom);
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);

        // Update denominators for next iteration:
        // (q^a;q)_{n+2} = (q^a;q)_{n+1} * (1 - q^{a+n+1})
        // (q^{1-a};q)_{n+2} = (q^{1-a};q)_{n+1} * (1 - q^{1-a+n+1})
        let factor1_exp = a_pow + n + 1;
        let mut f1 = FormalPowerSeries::one(variable, truncation_order);
        if factor1_exp == 0 {
            f1.set_coeff(0, QRat::zero());
        } else if factor1_exp < truncation_order {
            f1.set_coeff(factor1_exp, -QRat::one());
        }
        denom1 = arithmetic::mul(&denom1, &f1);

        let factor2_exp = (1 - a_pow) + n + 1;
        let mut f2 = FormalPowerSeries::one(variable, truncation_order);
        if factor2_exp == 0 {
            f2.set_coeff(0, QRat::zero());
        } else if factor2_exp < truncation_order {
            f2.set_coeff(factor2_exp, -QRat::one());
        }
        denom2 = arithmetic::mul(&denom2, &f2);
    }

    result
}

/// Compute the universal mock theta function g2(q^{a_pow}, q) as a formal power series.
///
/// ```text
/// g2(x, q) = x^{-1} * (-q;q)_inf * sum_{n>=0} q^{n(n+1)/2} * (-q;q)_n / [(x;q)_{n+1} * (q/x;q)_{n+1}]
/// ```
///
/// For x = q^{a_pow}:
/// ```text
/// g2(q^a, q) = q^{-a} * (-q;q)_inf * sum_{n>=0} q^{n(n+1)/2} * (-q;q)_n / [(q^a;q)_{n+1} * (q^{1-a};q)_{n+1}]
/// ```
///
/// Gordon and McIntosh showed that all even-order mock theta functions are
/// specializations of g2.
///
/// # Note on the q^{-a} prefactor
///
/// Since FPS cannot represent negative powers, the result is shifted: if a_pow > 0,
/// the inner sum is computed first, then coefficients are shifted down by a_pow.
/// This means the result represents g2(q^a, q) * q^{a_pow} at the FPS level when
/// the shift would produce negative exponents. For a_pow = 0, q^{-a} = 1 (no shift).
///
/// # Arguments
///
/// * `a_pow` - Integer such that x = q^{a_pow} (should be >= 0 for FPS representability)
/// * `variable` - The SymbolId for the series variable q
/// * `truncation_order` - Compute to O(q^truncation_order)
pub fn universal_mock_theta_g2(
    a_pow: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    // Compute (-q;q)_inf
    let neg_q = QMonomial::new(-QRat::one(), 1);
    let neg_q_inf = aqprod(&neg_q, variable, PochhammerOrder::Infinite, truncation_order);

    // Running products for denominator and numerator Pochhammer
    let a_mono = QMonomial::q_power(a_pow);
    let qa_inv_mono = QMonomial::q_power(1 - a_pow);

    // denom1 = (q^a;q)_{n+1}, denom2 = (q^{1-a};q)_{n+1}
    let mut denom1 = aqprod(&a_mono, variable, PochhammerOrder::Finite(1), truncation_order);
    let mut denom2 = aqprod(&qa_inv_mono, variable, PochhammerOrder::Finite(1), truncation_order);

    // numer_poch = (-q;q)_n, starting at n=0: (-q;q)_0 = 1
    let mut numer_poch = FormalPowerSeries::one(variable, truncation_order);

    // Inner sum: sum_{n>=0} q^{n(n+1)/2} * (-q;q)_n / [(q^a;q)_{n+1} * (q^{1-a};q)_{n+1}]
    let mut inner_sum = FormalPowerSeries::zero(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * (n + 1) / 2;
        if q_exp >= truncation_order {
            break;
        }

        // term = q^{n(n+1)/2} * numer_poch / (denom1 * denom2)
        let denom = arithmetic::mul(&denom1, &denom2);
        let denom_inv = arithmetic::invert(&denom);
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let term = arithmetic::mul(&arithmetic::mul(&numer, &numer_poch), &denom_inv);
        inner_sum = arithmetic::add(&inner_sum, &term);

        // Update numer_poch: (-q;q)_{n+1} = (-q;q)_n * (1 + q^{n+1})
        let m = n + 1;
        let mut npf = FormalPowerSeries::one(variable, truncation_order);
        if m < truncation_order {
            npf.set_coeff(m, QRat::one()); // 1 + q^m
        }
        numer_poch = arithmetic::mul(&numer_poch, &npf);

        // Update denominators
        let factor1_exp = a_pow + n + 1;
        let mut f1 = FormalPowerSeries::one(variable, truncation_order);
        if factor1_exp == 0 {
            f1.set_coeff(0, QRat::zero());
        } else if factor1_exp < truncation_order {
            f1.set_coeff(factor1_exp, -QRat::one());
        }
        denom1 = arithmetic::mul(&denom1, &f1);

        let factor2_exp = (1 - a_pow) + n + 1;
        let mut f2 = FormalPowerSeries::one(variable, truncation_order);
        if factor2_exp == 0 {
            f2.set_coeff(0, QRat::zero());
        } else if factor2_exp < truncation_order {
            f2.set_coeff(factor2_exp, -QRat::one());
        }
        denom2 = arithmetic::mul(&denom2, &f2);
    }

    // Multiply by (-q;q)_inf
    let mut result = arithmetic::mul(&neg_q_inf, &inner_sum);

    // Apply q^{-a_pow} shift: shift coefficients down by a_pow
    // This means coeff[k] of result becomes coeff[k - a_pow] of final
    if a_pow > 0 {
        result = arithmetic::shift(&result, -a_pow);
    } else if a_pow < 0 {
        // q^{-a_pow} = q^{|a_pow|}, shift up
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
    /// Third-order mock theta completions have weight 1/2 and level
    /// determined by the specific function. The non-holomorphic part
    /// involves a period integral of a weight-3/2 unary theta function.
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
            level: 2,       // third-order mock theta functions have level 2 or divisors
        }
    }

    /// Create a completion for a fifth-order mock theta function.
    ///
    /// Fifth-order mock theta completions also have weight 1/2 but with
    /// different level and correction structure.
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
    /// Given coefficients (c1, c2) for each pair, verifies that
    /// c1 * self.holomorphic_part + c2 * other.holomorphic_part
    /// matches a target FPS (e.g., a theta function quotient) to the given order.
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

        // Check that combo - target = 0 to the minimum truncation order
        let diff = arithmetic::sub(&combo, target);
        diff.is_zero()
    }

    /// Check that this completion's holomorphic part is nonzero and has
    /// the expected structure (nontrivial series).
    pub fn is_nontrivial(&self) -> bool {
        !self.holomorphic_part.is_zero()
    }
}
