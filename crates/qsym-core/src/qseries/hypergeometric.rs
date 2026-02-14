//! Basic hypergeometric series: _r phi_s and bilateral _r psi_s.
//!
//! Provides:
//! - [`HypergeometricSeries`]: parameters of _r phi_s
//! - [`BilateralHypergeometricSeries`]: parameters of _r psi_s
//! - [`eval_phi`]: evaluate _r phi_s to O(q^T) as FPS
//! - [`eval_psi`]: evaluate _r psi_s to O(q^T) as FPS
//! - [`SummationResult`]: closed-form result of a summation formula
//! - [`TransformationResult`]: transformed series + prefactor
//! - [`verify_transformation`]: verify a transformation by FPS comparison

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::symbol::SymbolId;
use super::{QMonomial, PochhammerOrder, aqprod};

/// Parameters of a basic hypergeometric series _r phi_s.
///
/// Represents: _r phi_s (a_1, ..., a_r ; b_1, ..., b_s ; q, z)
/// where each a_i and b_j is a QMonomial (c * q^m).
///
/// The series is defined as:
/// ```text
/// sum_{n=0}^{inf} [(a_1;q)_n * ... * (a_r;q)_n] / [(q;q)_n * (b_1;q)_n * ... * (b_s;q)_n]
///     * [(-1)^n * q^{n(n-1)/2}]^{1+s-r} * z^n
/// ```
#[derive(Clone, Debug)]
pub struct HypergeometricSeries {
    /// Upper parameters a_1, ..., a_r
    pub upper: Vec<QMonomial>,
    /// Lower parameters b_1, ..., b_s
    pub lower: Vec<QMonomial>,
    /// The argument z (as QMonomial)
    pub argument: QMonomial,
}

impl HypergeometricSeries {
    /// Number of upper parameters (r).
    pub fn r(&self) -> usize {
        self.upper.len()
    }

    /// Number of lower parameters (s).
    pub fn s(&self) -> usize {
        self.lower.len()
    }

    /// Check if the series is terminating (some a_i = q^{-n} with n >= 0).
    /// Returns the smallest termination order if found.
    pub fn termination_order(&self) -> Option<i64> {
        let mut min_term: Option<i64> = None;
        for a in &self.upper {
            if let Some(n) = a.is_q_neg_power() {
                match min_term {
                    None => min_term = Some(n),
                    Some(cur) => {
                        if n < cur {
                            min_term = Some(n);
                        }
                    }
                }
            }
        }
        min_term
    }
}

/// Parameters of a bilateral hypergeometric series _r psi_s.
///
/// Represents: _r psi_s (a_1, ..., a_r ; b_1, ..., b_s ; q, z)
///
/// The series is defined as:
/// ```text
/// sum_{n=-inf}^{inf} [(a_1;q)_n * ... * (a_r;q)_n] / [(b_1;q)_n * ... * (b_s;q)_n]
///     * [(-1)^n * q^{n(n-1)/2}]^{s-r} * z^n
/// ```
///
/// Note: NO (q;q)_n in denominator (unlike phi). The extra factor exponent is s-r (not 1+s-r).
#[derive(Clone, Debug)]
pub struct BilateralHypergeometricSeries {
    /// Upper parameters a_1, ..., a_r
    pub upper: Vec<QMonomial>,
    /// Lower parameters b_1, ..., b_s
    pub lower: Vec<QMonomial>,
    /// The argument z (as QMonomial)
    pub argument: QMonomial,
}

impl BilateralHypergeometricSeries {
    /// Number of upper parameters (r).
    pub fn r(&self) -> usize {
        self.upper.len()
    }

    /// Number of lower parameters (s).
    pub fn s(&self) -> usize {
        self.lower.len()
    }
}

/// Result of attempting to apply a summation formula.
#[derive(Clone, Debug)]
pub enum SummationResult {
    /// Formula applied; returns the closed-form FPS.
    ClosedForm(FormalPowerSeries),
    /// Formula does not apply (parameters don't match the pattern).
    NotApplicable,
}

/// Result of applying a transformation formula.
#[derive(Clone, Debug)]
pub struct TransformationResult {
    /// The scalar/product prefactor, evaluated as FPS.
    pub prefactor: FormalPowerSeries,
    /// The transformed hypergeometric series.
    pub transformed: HypergeometricSeries,
}

// ---------------------------------------------------------------------------
// Helper: build FPS for (1 - coeff * q^m)
// ---------------------------------------------------------------------------

/// Create the 2-term FPS: 1 - coeff*q^m, truncated to O(q^trunc).
///
/// - If m == 0: single-term FPS with value (1 - coeff) at q^0.
/// - If m > 0 and m < trunc: two terms, 1 at q^0 and -coeff at q^m.
/// - If m >= trunc or m < 0: just 1 at q^0 (the q^m term is beyond truncation or below zero).
fn one_minus_cq_m(coeff: &QRat, m: i64, variable: SymbolId, trunc: i64) -> FormalPowerSeries {
    let mut f = FormalPowerSeries::one(variable, trunc);
    if m == 0 {
        // (1 - coeff) at q^0
        let val = QRat::one() - coeff.clone();
        f.set_coeff(0, val);
    } else if m > 0 && m < trunc {
        f.set_coeff(m, -coeff.clone());
    }
    // else: m >= trunc or m < 0 => just 1
    f
}

// ---------------------------------------------------------------------------
// eval_phi: evaluate _r phi_s
// ---------------------------------------------------------------------------

/// Evaluate _r phi_s to O(q^T) as a FormalPowerSeries.
///
/// Uses FPS-based term accumulation: each term is computed from the previous
/// via multiplication by the ratio FPS.
///
/// The n-th term ratio (from term n to term n+1) is:
/// ```text
/// ratio = [prod_i (1 - a_i*q^{a_i.power+n})]
///       / [(1 - q^{n+1}) * prod_j (1 - b_j*q^{b_j.power+n})]
///       * [(-1)^{extra} * q^{n*extra}] * z
/// ```
/// where extra = 1 + s - r.
pub fn eval_phi(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let r = series.r();
    let s = series.s();
    let extra = 1 + s as i64 - r as i64;

    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    let mut term = FormalPowerSeries::one(variable, truncation_order);

    let max_n = series.termination_order()
        .map(|n| n.min(truncation_order))
        .unwrap_or(truncation_order);

    for n in 0..=max_n {
        // Accumulate current term
        result = arithmetic::add(&result, &term);

        if n == max_n {
            break;
        }

        // Build the ratio for stepping from term n to term n+1.
        // Numerator: product of (1 - a_i.coeff * q^{a_i.power + n}) for each upper param
        let mut numer = FormalPowerSeries::one(variable, truncation_order);
        for a in &series.upper {
            let factor = one_minus_cq_m(&a.coeff, a.power + n, variable, truncation_order);
            numer = arithmetic::mul(&numer, &factor);
        }

        // Denominator: (1 - q^{n+1}) * product of (1 - b_j.coeff * q^{b_j.power + n})
        let mut denom = one_minus_cq_m(&QRat::one(), n + 1, variable, truncation_order);
        for b in &series.lower {
            let factor = one_minus_cq_m(&b.coeff, b.power + n, variable, truncation_order);
            denom = arithmetic::mul(&denom, &factor);
        }

        // Invert denominator once (optimization: single inversion per step)
        let denom_inv = arithmetic::invert(&denom);

        // Combine numerator and inverted denominator
        let mut ratio = arithmetic::mul(&numer, &denom_inv);

        // Extra factor: [(-1)^n * q^{n(n-1)/2}]^{extra} at step n
        // Going from term n to n+1, this contributes the RATIO of extra factors:
        // [(-1)^{n+1} * q^{n(n+1)/2}]^extra / [(-1)^n * q^{n(n-1)/2}]^extra
        // = [(-1) * q^n]^extra
        // = (-1)^extra * q^{n*extra}
        if extra != 0 {
            let sign = if extra % 2 == 0 { QRat::one() } else { -QRat::one() };
            let q_shift = n * extra;
            if q_shift >= 0 && q_shift < truncation_order {
                let extra_fps = FormalPowerSeries::monomial(variable, sign, q_shift, truncation_order);
                ratio = arithmetic::mul(&ratio, &extra_fps);
            } else if q_shift < 0 {
                // Negative shift: the monomial q^{negative} is below q^0, skip
                // This means the term contribution is effectively zero for truncation
                let extra_fps = FormalPowerSeries::monomial(variable, sign, q_shift, truncation_order);
                ratio = arithmetic::mul(&ratio, &extra_fps);
            } else {
                // q_shift >= truncation_order: ratio becomes zero in truncation
                break;
            }
        }

        // Argument factor: z.coeff * q^{z.power}
        let z_fps = FormalPowerSeries::monomial(
            variable,
            series.argument.coeff.clone(),
            series.argument.power,
            truncation_order,
        );
        ratio = arithmetic::mul(&ratio, &z_fps);

        // Update term
        term = arithmetic::mul(&term, &ratio);
        if term.is_zero() {
            break;
        }
    }
    result
}

// ---------------------------------------------------------------------------
// eval_psi: evaluate _r psi_s (bilateral)
// ---------------------------------------------------------------------------

/// Evaluate _r psi_s bilateral series to O(q^T) as a FormalPowerSeries.
///
/// The bilateral series sums from n = -inf to +inf. We split into:
/// - Positive part (n >= 0): similar to eval_phi but without (q;q)_n denominator
///   and with extra factor exponent s-r instead of 1+s-r.
/// - Negative part (n < 0): each negative-n term is computed directly using
///   aqprod with negative order.
///
/// ```text
/// _r psi_s = sum_{n=-inf}^{inf} [(a_1;q)_n * ... * (a_r;q)_n]
///          / [(b_1;q)_n * ... * (b_s;q)_n]
///          * [(-1)^n * q^{n(n-1)/2}]^{s-r} * z^n
/// ```
pub fn eval_psi(
    series: &BilateralHypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let r = series.r();
    let s = series.s();
    let extra = s as i64 - r as i64;

    // Compute positive part: n = 0, 1, 2, ...
    let positive = eval_psi_positive(series, variable, truncation_order, extra);

    // Compute negative part: n = -1, -2, ...
    let negative = eval_psi_negative(series, variable, truncation_order, extra);

    arithmetic::add(&positive, &negative)
}

/// Positive part of bilateral series (n >= 0).
///
/// Like eval_phi but:
/// - No (q;q)_n denominator
/// - Extra factor exponent is `extra` (= s-r), not 1+s-r
fn eval_psi_positive(
    series: &BilateralHypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
    extra: i64,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    let mut term = FormalPowerSeries::one(variable, truncation_order);

    for n in 0..=truncation_order {
        result = arithmetic::add(&result, &term);

        if n == truncation_order {
            break;
        }

        // Numerator: product of (1 - a_i.coeff * q^{a_i.power + n})
        let mut numer = FormalPowerSeries::one(variable, truncation_order);
        for a in &series.upper {
            let factor = one_minus_cq_m(&a.coeff, a.power + n, variable, truncation_order);
            numer = arithmetic::mul(&numer, &factor);
        }

        // Denominator: product of (1 - b_j.coeff * q^{b_j.power + n}) -- NO (q;q)_n
        let mut denom = FormalPowerSeries::one(variable, truncation_order);
        for b in &series.lower {
            let factor = one_minus_cq_m(&b.coeff, b.power + n, variable, truncation_order);
            denom = arithmetic::mul(&denom, &factor);
        }

        let denom_inv = arithmetic::invert(&denom);
        let mut ratio = arithmetic::mul(&numer, &denom_inv);

        // Extra factor ratio: (-1)^extra * q^{n*extra}
        if extra != 0 {
            let sign = if extra % 2 == 0 { QRat::one() } else { -QRat::one() };
            let q_shift = n * extra;
            let extra_fps = FormalPowerSeries::monomial(variable, sign, q_shift, truncation_order);
            ratio = arithmetic::mul(&ratio, &extra_fps);
        }

        // Argument z
        let z_fps = FormalPowerSeries::monomial(
            variable,
            series.argument.coeff.clone(),
            series.argument.power,
            truncation_order,
        );
        ratio = arithmetic::mul(&ratio, &z_fps);

        term = arithmetic::mul(&term, &ratio);
        if term.is_zero() {
            break;
        }
    }
    result
}

/// Check if (a;q)_{-m} has a pole (i.e., the denominator (aq^{-m};q)_m contains a zero factor).
///
/// A pole occurs when a.coeff == 1 and 0 < a.power <= m.
fn has_negative_pochhammer_pole(a: &QMonomial, m: i64) -> bool {
    if a.coeff == QRat::one() && a.power > 0 && a.power <= m {
        return true;
    }
    false
}

/// Negative part of bilateral series (n = -1, -2, ...).
///
/// For each negative n = -m (m > 0), compute the term directly:
/// ```text
/// T_{-m} = [prod_i (a_i;q)_{-m}] / [prod_j (b_j;q)_{-m}]
///        * [(-1)^{-m} * q^{(-m)(-m-1)/2}]^{extra} * z^{-m}
/// ```
///
/// Uses aqprod with negative order for each parameter.
/// Terms where any Pochhammer symbol has a pole are skipped (they represent
/// a 0/0 cancellation that would need L'Hopital-type analysis).
fn eval_psi_negative(
    series: &BilateralHypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
    extra: i64,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);

    // Determine max number of negative terms to compute.
    // The z^{-m} factor contributes q^{-m * z.power}. For z.power > 0, these go to
    // increasingly negative q-powers. For z.power == 0, there's no q-power separation.
    // We cap at truncation_order for safety.
    let max_neg = truncation_order;

    for m in 1..=max_neg {
        // Check for poles: if any upper or lower param has a pole at -m, skip this term.
        let has_pole = series.upper.iter().any(|a| has_negative_pochhammer_pole(a, m))
            || series.lower.iter().any(|b| has_negative_pochhammer_pole(b, m));
        if has_pole {
            // When the numerator has a pole, the term is 0 (zero from (a;q)_{-m}).
            // When the denominator has a pole, the term is infinite (skip).
            // When both have poles, it's 0/0 (skip -- would need careful analysis).
            //
            // For the common case where only the numerator has a pole:
            // (a;q)_{-m} with pole means (aq^{-m};q)_m = 0, so 1/0 = pole in the
            // Pochhammer value. But the actual bilateral series term involves the
            // product of all Pochhammers. If ANY numerator Pochhammer has a zero
            // (making the overall numerator zero), the term is zero.
            //
            // Detect: numerator zero kills the term (skip as zero).
            // Denominator zero means the term diverges (also skip for safety).
            continue;
        }

        // Compute (a_i;q)_{-m} for each upper parameter
        let mut term = FormalPowerSeries::one(variable, truncation_order);
        for a in &series.upper {
            let poch = aqprod(a, variable, PochhammerOrder::Finite(-m), truncation_order);
            term = arithmetic::mul(&term, &poch);
        }

        // Divide by (b_j;q)_{-m} for each lower parameter
        for b in &series.lower {
            let poch = aqprod(b, variable, PochhammerOrder::Finite(-m), truncation_order);
            let poch_inv = arithmetic::invert(&poch);
            term = arithmetic::mul(&term, &poch_inv);
        }

        // Extra factor: [(-1)^{-m} * q^{(-m)(-m-1)/2}]^{extra}
        // (-m)(-m-1)/2 = m(m+1)/2. (-1)^{-m} = (-1)^m.
        if extra != 0 {
            let m_i64 = m;
            let sign_base = if m_i64 % 2 == 0 { QRat::one() } else { -QRat::one() };
            let q_power_base = m_i64 * (m_i64 + 1) / 2;

            let (sign, q_pow) = if extra > 0 {
                let s = if extra % 2 == 0 { QRat::one() } else { sign_base.clone() };
                (s, q_power_base * extra)
            } else {
                let abs_extra = -extra;
                let s = if abs_extra % 2 == 0 { QRat::one() } else { sign_base.clone() };
                (s, q_power_base * extra)
            };

            let extra_fps = FormalPowerSeries::monomial(variable, sign, q_pow, truncation_order);
            term = arithmetic::mul(&term, &extra_fps);
        }

        // Argument: z^{-m} = z.coeff^{-m} * q^{-m * z.power}
        if series.argument.coeff.is_zero() {
            break;
        }
        let z_coeff_inv = QRat::one() / series.argument.coeff.clone();
        let mut z_neg_m_coeff = QRat::one();
        for _ in 0..m {
            z_neg_m_coeff = z_neg_m_coeff * z_coeff_inv.clone();
        }
        let z_neg_m_power = -m * series.argument.power;
        let z_fps = FormalPowerSeries::monomial(variable, z_neg_m_coeff, z_neg_m_power, truncation_order);
        term = arithmetic::mul(&term, &z_fps);

        if term.is_zero() {
            continue;
        }

        result = arithmetic::add(&result, &term);
    }
    result
}

// ---------------------------------------------------------------------------
// verify_transformation
// ---------------------------------------------------------------------------

/// Verify a transformation by expanding both sides and comparing FPS coefficients.
///
/// Returns true if:
/// `eval_phi(original) == prefactor * eval_phi(transformed)`
pub fn verify_transformation(
    original: &HypergeometricSeries,
    result: &TransformationResult,
    variable: SymbolId,
    truncation_order: i64,
) -> bool {
    let lhs = eval_phi(original, variable, truncation_order);
    let rhs_series = eval_phi(&result.transformed, variable, truncation_order);
    let rhs = arithmetic::mul(&result.prefactor, &rhs_series);
    lhs == rhs
}
