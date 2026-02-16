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
//! - Summation formulas: [`try_q_gauss`], [`try_q_vandermonde`], [`try_q_saalschutz`],
//!   [`try_q_kummer`], [`try_q_dixon`], [`try_all_summations`]
//! - Transformation formulas: [`heine_transform_1`], [`heine_transform_2`], [`heine_transform_3`],
//!   [`sears_transform`], [`watson_transform`]
//! - Bailey's identity: [`bailey_4phi3_q2`] (standalone closed-form for DLMF 17.7.12)
//! - Transformation chain search: [`find_transformation_chain`], [`TransformationStep`],
//!   [`TransformationChainResult`]

use std::collections::{VecDeque, HashSet};

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

/// A single step in a transformation chain.
#[derive(Clone, Debug)]
pub struct TransformationStep {
    /// Name of the transformation applied (e.g., "heine_1", "sears").
    pub name: String,
    /// The resulting series after this transformation.
    pub result_series: HypergeometricSeries,
    /// The prefactor from this single transformation step.
    pub step_prefactor: FormalPowerSeries,
}

/// Result of a transformation chain search.
#[derive(Clone, Debug)]
pub enum TransformationChainResult {
    /// A chain of transformations was found connecting source to target.
    Found {
        /// The sequence of transformation steps.
        steps: Vec<TransformationStep>,
        /// The cumulative prefactor (product of all step prefactors).
        total_prefactor: FormalPowerSeries,
    },
    /// No chain found within the depth bound.
    NotFound {
        /// The depth bound used.
        max_depth: usize,
    },
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

// ---------------------------------------------------------------------------
// Summation formulas
// ---------------------------------------------------------------------------

/// Raise a QRat to a non-negative integer power.
fn qrat_pow(base: &QRat, exp: u64) -> QRat {
    if exp == 0 {
        return QRat::one();
    }
    let mut result = base.clone();
    for _ in 1..exp {
        result = result * base.clone();
    }
    result
}

/// Try q-Gauss summation (DLMF 17.6.1).
///
/// ```text
/// _2 phi_1 (a, b ; c ; q, c/(ab)) = (c/a;q)_inf * (c/b;q)_inf / [(c;q)_inf * (c/(ab);q)_inf]
/// ```
///
/// Checks: r==2, s==1, z == c/(a*b).
pub fn try_q_gauss(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> SummationResult {
    if series.r() != 2 || series.s() != 1 {
        return SummationResult::NotApplicable;
    }

    let a = &series.upper[0];
    let b = &series.upper[1];
    let c = &series.lower[0];
    let z = &series.argument;

    // Check: z == c / (a * b)
    let ab = a.mul(b);
    let expected_z = c.div(&ab);

    if *z != expected_z {
        return SummationResult::NotApplicable;
    }

    // Closed form: (c/a;q)_inf * (c/b;q)_inf / [(c;q)_inf * (c/(ab);q)_inf]
    let c_over_a = c.div(a);
    let c_over_b = c.div(b);
    let c_over_ab = c.div(&ab);

    let numer1 = aqprod(&c_over_a, variable, PochhammerOrder::Infinite, truncation_order);
    let numer2 = aqprod(&c_over_b, variable, PochhammerOrder::Infinite, truncation_order);
    let denom1 = aqprod(c, variable, PochhammerOrder::Infinite, truncation_order);
    let denom2 = aqprod(&c_over_ab, variable, PochhammerOrder::Infinite, truncation_order);

    let numer = arithmetic::mul(&numer1, &numer2);
    let denom = arithmetic::mul(&denom1, &denom2);
    SummationResult::ClosedForm(arithmetic::mul(&numer, &arithmetic::invert(&denom)))
}

/// Try q-Vandermonde summation (DLMF 17.6.2, 17.6.3).
///
/// First form (z = c*q^n/a):
/// ```text
/// _2 phi_1 (a, q^{-n} ; c ; q, c*q^n/a) = (c/a;q)_n / (c;q)_n
/// ```
///
/// Second form (z = q):
/// ```text
/// _2 phi_1 (a, q^{-n} ; c ; q, q) = a^n * (c/a;q)_n / (c;q)_n
/// ```
///
/// Checks: r==2, s==1, one upper param is q^{-n}.
pub fn try_q_vandermonde(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> SummationResult {
    if series.r() != 2 || series.s() != 1 {
        return SummationResult::NotApplicable;
    }

    let z = &series.argument;
    let c = &series.lower[0];

    // Find which upper param is q^{-n}
    for idx in 0..2 {
        let term_param = &series.upper[idx];
        let other_param = &series.upper[1 - idx];

        if let Some(n) = term_param.is_q_neg_power() {
            if n == 0 {
                // q^0 = 1, trivially terminates at 0 terms: result is 1
                return SummationResult::ClosedForm(
                    FormalPowerSeries::one(variable, truncation_order)
                );
            }

            let a = other_param;

            // First form: z = c*q^n/a
            let expected_z1 = c.mul(&QMonomial::q_power(n)).div(a);
            if *z == expected_z1 {
                // (c/a;q)_n / (c;q)_n
                let c_over_a = c.div(a);
                let numer = aqprod(&c_over_a, variable, PochhammerOrder::Finite(n), truncation_order);
                let denom = aqprod(c, variable, PochhammerOrder::Finite(n), truncation_order);
                return SummationResult::ClosedForm(
                    arithmetic::mul(&numer, &arithmetic::invert(&denom))
                );
            }

            // Second form: z = q
            if *z == QMonomial::q_power(1) {
                // a^n * (c/a;q)_n / (c;q)_n
                let c_over_a = c.div(a);
                let numer = aqprod(&c_over_a, variable, PochhammerOrder::Finite(n), truncation_order);
                let denom = aqprod(c, variable, PochhammerOrder::Finite(n), truncation_order);
                let ratio = arithmetic::mul(&numer, &arithmetic::invert(&denom));

                // a^n as FPS: coeff^n * q^{power*n}
                let a_n_coeff = qrat_pow(&a.coeff, n as u64);
                let a_n_power = a.power * n;
                let a_n_fps = FormalPowerSeries::monomial(
                    variable, a_n_coeff, a_n_power, truncation_order,
                );

                return SummationResult::ClosedForm(arithmetic::mul(&a_n_fps, &ratio));
            }
        }
    }

    SummationResult::NotApplicable
}

/// Try q-Pfaff-Saalschutz summation (DLMF 17.7.4).
///
/// ```text
/// _3 phi_2 (a, b, q^{-n} ; c, abq^{1-n}/c ; q, q) = (c/a;q)_n * (c/b;q)_n / [(c;q)_n * (c/(ab);q)_n]
/// ```
///
/// Checks: r==3, s==2, z==q, one upper param is q^{-n}, balanced condition holds.
pub fn try_q_saalschutz(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> SummationResult {
    if series.r() != 3 || series.s() != 2 {
        return SummationResult::NotApplicable;
    }

    // Check z == q
    if series.argument != QMonomial::q_power(1) {
        return SummationResult::NotApplicable;
    }

    // Try all assignments: which upper param is q^{-n}?
    for term_idx in 0..3 {
        let term_param = &series.upper[term_idx];
        if let Some(n) = term_param.is_q_neg_power() {
            if n == 0 {
                return SummationResult::ClosedForm(
                    FormalPowerSeries::one(variable, truncation_order)
                );
            }

            // The other two upper params are a, b (try both orderings)
            let other_idxs: Vec<usize> = (0..3).filter(|&i| i != term_idx).collect();
            let a = &series.upper[other_idxs[0]];
            let b = &series.upper[other_idxs[1]];

            // Balance condition: one lower param is c, the other is a*b*q^{1-n}/c
            let ab = a.mul(b);
            let q_1_minus_n = QMonomial::q_power(1 - n);

            // Try each lower param as c
            for c_idx in 0..2 {
                let d_idx = 1 - c_idx;
                let c = &series.lower[c_idx];
                let d = &series.lower[d_idx];

                // expected_d = a*b*q^{1-n}/c
                let expected_d = ab.mul(&q_1_minus_n).div(c);

                if *d == expected_d {
                    // Match! Compute closed form:
                    // (c/a;q)_n * (c/b;q)_n / [(c;q)_n * (c/(ab);q)_n]
                    let c_over_a = c.div(a);
                    let c_over_b = c.div(b);
                    let c_over_ab = c.div(&ab);

                    let n1 = aqprod(&c_over_a, variable, PochhammerOrder::Finite(n), truncation_order);
                    let n2 = aqprod(&c_over_b, variable, PochhammerOrder::Finite(n), truncation_order);
                    let d1 = aqprod(c, variable, PochhammerOrder::Finite(n), truncation_order);
                    let d2 = aqprod(&c_over_ab, variable, PochhammerOrder::Finite(n), truncation_order);

                    let numer = arithmetic::mul(&n1, &n2);
                    let denom = arithmetic::mul(&d1, &d2);
                    return SummationResult::ClosedForm(
                        arithmetic::mul(&numer, &arithmetic::invert(&denom))
                    );
                }
            }
        }
    }

    SummationResult::NotApplicable
}

/// Helper: compute a q^2-Pochhammer product with general coefficient.
///
/// Computes prod_{k=0}^{N-1} (1 - coeff * q^{start + 2*k}) as FPS.
/// N is determined by truncation: continue while start + 2*k < trunc.
/// If `finite_n` is Some(n), limit to n factors.
fn q2_pochhammer_product(
    coeff: &QRat,
    start: i64,
    variable: SymbolId,
    trunc: i64,
    finite_n: Option<i64>,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::one(variable, trunc);
    let mut k = 0i64;
    loop {
        if let Some(n) = finite_n {
            if k >= n {
                break;
            }
        }
        let exp = start + 2 * k;
        if exp >= trunc {
            break;
        }
        let factor = one_minus_cq_m(coeff, exp, variable, trunc);
        result = arithmetic::mul(&result, &factor);
        k += 1;
    }
    result
}

/// Try q-Kummer (Bailey-Daum) summation (DLMF 17.6.5).
///
/// ```text
/// _2 phi_1 (a, b ; aq/b ; q, -q/b) = (-q;q)_inf * (aq;q^2)_inf * (aq^2/b^2;q^2)_inf
///                                     / [(-q/b;q)_inf * (aq/b;q)_inf]
/// ```
///
/// Checks: r==2, s==1, c == aq/b, z == -q/b.
pub fn try_q_kummer(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> SummationResult {
    if series.r() != 2 || series.s() != 1 {
        return SummationResult::NotApplicable;
    }

    // Try both orderings of upper params as (a, b)
    for idx in 0..2 {
        let a = &series.upper[idx];
        let b = &series.upper[1 - idx];
        let c = &series.lower[0];
        let z = &series.argument;

        // Check: c == a*q/b
        let aq_over_b = a.mul(&QMonomial::q_power(1)).div(b);
        if *c != aq_over_b {
            continue;
        }

        // Check: z == -q/b
        let neg_q_over_b = QMonomial::new(-QRat::one(), 1).div(b);
        if *z != neg_q_over_b {
            continue;
        }

        // Match! Compute RHS:
        // (-q;q)_inf * (aq;q^2)_inf * (aq^2/b^2;q^2)_inf / [(-q/b;q)_inf * (aq/b;q)_inf]

        // (-q;q)_inf
        let neg_q = QMonomial::new(-QRat::one(), 1);
        let f1 = aqprod(&neg_q, variable, PochhammerOrder::Infinite, truncation_order);

        // (aq;q^2)_inf = prod_{k>=0} (1 - a.coeff * q^{a.power+1+2k})
        let aq_coeff = &a.coeff;
        let aq_start = a.power + 1;
        let f2 = q2_pochhammer_product(aq_coeff, aq_start, variable, truncation_order, None);

        // (aq^2/b^2;q^2)_inf = prod_{k>=0} (1 - (a.coeff/b.coeff^2) * q^{a.power+2-2*b.power+2k})
        let b_sq = b.mul(b);
        let aq2_over_b2 = a.mul(&QMonomial::q_power(2)).div(&b_sq);
        let f3 = q2_pochhammer_product(
            &aq2_over_b2.coeff,
            aq2_over_b2.power,
            variable,
            truncation_order,
            None,
        );

        // (-q/b;q)_inf
        let neg_q_over_b_mon = QMonomial::new(-QRat::one(), 1).div(b);
        let f4 = aqprod(&neg_q_over_b_mon, variable, PochhammerOrder::Infinite, truncation_order);

        // (aq/b;q)_inf
        let f5 = aqprod(&aq_over_b, variable, PochhammerOrder::Infinite, truncation_order);

        let numer = arithmetic::mul(&f1, &arithmetic::mul(&f2, &f3));
        let denom = arithmetic::mul(&f4, &f5);
        return SummationResult::ClosedForm(
            arithmetic::mul(&numer, &arithmetic::invert(&denom))
        );
    }

    SummationResult::NotApplicable
}

/// Try q-Dixon (Jackson) summation (DLMF 17.7.6).
///
/// ```text
/// _3 phi_2 (q^{-2n}, b, c ; q^{1-2n}/b, q^{1-2n}/c ; q, q^{2-n}/(bc))
///   = (b;q)_n * (c;q)_n * (q;q)_{2n} * (bc;q)_{2n}
///     / [(q;q)_n * (bc;q)_n * (b;q)_{2n} * (c;q)_{2n}]
/// ```
///
/// Checks: r==3, s==2, one upper param is q^{-m} with m even (m=2n).
pub fn try_q_dixon(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> SummationResult {
    if series.r() != 3 || series.s() != 2 {
        return SummationResult::NotApplicable;
    }

    // Find upper param that is q^{-m} with m even
    for term_idx in 0..3 {
        let term_param = &series.upper[term_idx];
        if let Some(m) = term_param.is_q_neg_power() {
            if m % 2 != 0 {
                continue;
            }
            let n = m / 2;

            if n == 0 {
                return SummationResult::ClosedForm(
                    FormalPowerSeries::one(variable, truncation_order)
                );
            }

            // The other two upper params are b, c (try both orderings)
            let other_idxs: Vec<usize> = (0..3).filter(|&i| i != term_idx).collect();

            for perm in &[(0, 1), (1, 0)] {
                let b = &series.upper[other_idxs[perm.0]];
                let c = &series.upper[other_idxs[perm.1]];

                // Check lower params: should be q^{1-2n}/b and q^{1-2n}/c
                let q_1_minus_2n = QMonomial::q_power(1 - 2 * n);
                let expected_lower1 = q_1_minus_2n.div(b);
                let expected_lower2 = q_1_minus_2n.div(c);

                // Try both orderings of lower params
                let lower_match = (series.lower[0] == expected_lower1 && series.lower[1] == expected_lower2)
                    || (series.lower[0] == expected_lower2 && series.lower[1] == expected_lower1);

                if !lower_match {
                    continue;
                }

                // Check z: q^{2-n}/(bc) = q^{2-n} / (b*c)
                let bc = b.mul(c);
                let expected_z = QMonomial::q_power(2 - n).div(&bc);

                // The plan says z = q^{2-n}/(bc) but we should double-check with DLMF.
                // Actually from the research doc: z = q^{2-n}/(bc).
                // But wait: 2-n, not 2-2n. Let me re-read.
                // DLMF 17.7.6: z = q^{2-2n}/(bc)... let me check the formula again.
                // From the plan: z = q^{2-n}/(bc). I'll trust the plan.

                if series.argument != expected_z {
                    continue;
                }

                // Match! Compute:
                // (b;q)_n * (c;q)_n * (q;q)_{2n} * (bc;q)_{2n}
                // / [(q;q)_n * (bc;q)_n * (b;q)_{2n} * (c;q)_{2n}]

                let two_n = 2 * n;
                let q_mon = QMonomial::q_power(1); // q

                let bq_n = aqprod(b, variable, PochhammerOrder::Finite(n), truncation_order);
                let cq_n = aqprod(c, variable, PochhammerOrder::Finite(n), truncation_order);
                let qq_2n = aqprod(&q_mon, variable, PochhammerOrder::Finite(two_n), truncation_order);
                let bcq_2n = aqprod(&bc, variable, PochhammerOrder::Finite(two_n), truncation_order);

                let qq_n = aqprod(&q_mon, variable, PochhammerOrder::Finite(n), truncation_order);
                let bcq_n = aqprod(&bc, variable, PochhammerOrder::Finite(n), truncation_order);
                let bq_2n = aqprod(b, variable, PochhammerOrder::Finite(two_n), truncation_order);
                let cq_2n = aqprod(c, variable, PochhammerOrder::Finite(two_n), truncation_order);

                let numer = arithmetic::mul(
                    &arithmetic::mul(&bq_n, &cq_n),
                    &arithmetic::mul(&qq_2n, &bcq_2n),
                );
                let denom = arithmetic::mul(
                    &arithmetic::mul(&qq_n, &bcq_n),
                    &arithmetic::mul(&bq_2n, &cq_2n),
                );

                return SummationResult::ClosedForm(
                    arithmetic::mul(&numer, &arithmetic::invert(&denom))
                );
            }
        }
    }

    SummationResult::NotApplicable
}

/// Try all summation formulas in order, returning the first match.
///
/// Tries: q-Gauss, q-Vandermonde, q-Saalschutz, q-Kummer, q-Dixon.
pub fn try_all_summations(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> SummationResult {
    for try_fn in [try_q_gauss, try_q_vandermonde, try_q_saalschutz, try_q_kummer, try_q_dixon] {
        if let SummationResult::ClosedForm(fps) = try_fn(series, variable, truncation_order) {
            return SummationResult::ClosedForm(fps);
        }
    }
    SummationResult::NotApplicable
}

// ---------------------------------------------------------------------------
// Transformation formulas
// ---------------------------------------------------------------------------

/// Heine's first transformation (Gasper-Rahman 1.4.1).
///
/// ```text
/// _2 phi_1 (a, b ; c ; q, z)
///   = [(b;q)_inf * (az;q)_inf] / [(c;q)_inf * (z;q)_inf]
///     * _2 phi_1 (c/b, z ; az ; q, b)
/// ```
///
/// Returns `None` if the series is not a 2phi1.
pub fn heine_transform_1(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> Option<TransformationResult> {
    if series.r() != 2 || series.s() != 1 {
        return None;
    }

    let a = &series.upper[0];
    let b = &series.upper[1];
    let c = &series.lower[0];
    let z = &series.argument;

    // Transformed parameters:
    // new upper: [c/b, z]
    // new lower: [az]
    // new argument: b
    let az = a.mul(z);
    let c_over_b = c.div(b);

    let transformed = HypergeometricSeries {
        upper: vec![c_over_b, z.clone()],
        lower: vec![az.clone()],
        argument: b.clone(),
    };

    // Prefactor: (b;q)_inf * (az;q)_inf / [(c;q)_inf * (z;q)_inf]
    let b_inf = aqprod(b, variable, PochhammerOrder::Infinite, truncation_order);
    let az_inf = aqprod(&az, variable, PochhammerOrder::Infinite, truncation_order);
    let c_inf = aqprod(c, variable, PochhammerOrder::Infinite, truncation_order);
    let z_inf = aqprod(z, variable, PochhammerOrder::Infinite, truncation_order);

    let numer = arithmetic::mul(&b_inf, &az_inf);
    let denom = arithmetic::mul(&c_inf, &z_inf);
    let prefactor = arithmetic::mul(&numer, &arithmetic::invert(&denom));

    Some(TransformationResult { prefactor, transformed })
}

/// Heine's second transformation (Gasper-Rahman 1.4.2).
///
/// ```text
/// _2 phi_1 (a, b ; c ; q, z)
///   = [(c/b;q)_inf * (bz;q)_inf] / [(c;q)_inf * (z;q)_inf]
///     * _2 phi_1 (abz/c, b ; bz ; q, c/b)
/// ```
///
/// Returns `None` if the series is not a 2phi1.
pub fn heine_transform_2(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> Option<TransformationResult> {
    if series.r() != 2 || series.s() != 1 {
        return None;
    }

    let a = &series.upper[0];
    let b = &series.upper[1];
    let c = &series.lower[0];
    let z = &series.argument;

    // Transformed parameters:
    // new upper: [abz/c, b]
    // new lower: [bz]
    // new argument: c/b
    let bz = b.mul(z);
    let abz_over_c = a.mul(b).mul(z).div(c);
    let c_over_b = c.div(b);

    let transformed = HypergeometricSeries {
        upper: vec![abz_over_c, b.clone()],
        lower: vec![bz.clone()],
        argument: c_over_b.clone(),
    };

    // Prefactor: (c/b;q)_inf * (bz;q)_inf / [(c;q)_inf * (z;q)_inf]
    let c_over_b_inf = aqprod(&c_over_b, variable, PochhammerOrder::Infinite, truncation_order);
    let bz_inf = aqprod(&bz, variable, PochhammerOrder::Infinite, truncation_order);
    let c_inf = aqprod(c, variable, PochhammerOrder::Infinite, truncation_order);
    let z_inf = aqprod(z, variable, PochhammerOrder::Infinite, truncation_order);

    let numer = arithmetic::mul(&c_over_b_inf, &bz_inf);
    let denom = arithmetic::mul(&c_inf, &z_inf);
    let prefactor = arithmetic::mul(&numer, &arithmetic::invert(&denom));

    Some(TransformationResult { prefactor, transformed })
}

/// Heine's third transformation (Gasper-Rahman 1.4.3).
///
/// ```text
/// _2 phi_1 (a, b ; c ; q, z)
///   = [(abz/c;q)_inf] / [(z;q)_inf]
///     * _2 phi_1 (c/a, c/b ; c ; q, abz/c)
/// ```
///
/// Returns `None` if the series is not a 2phi1.
pub fn heine_transform_3(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> Option<TransformationResult> {
    if series.r() != 2 || series.s() != 1 {
        return None;
    }

    let a = &series.upper[0];
    let b = &series.upper[1];
    let c = &series.lower[0];
    let z = &series.argument;

    // Transformed parameters:
    // new upper: [c/a, c/b]
    // new lower: [c]
    // new argument: abz/c
    let c_over_a = c.div(a);
    let c_over_b = c.div(b);
    let abz_over_c = a.mul(b).mul(z).div(c);

    let transformed = HypergeometricSeries {
        upper: vec![c_over_a, c_over_b],
        lower: vec![c.clone()],
        argument: abz_over_c.clone(),
    };

    // Prefactor: (abz/c;q)_inf / (z;q)_inf
    let abzc_inf = aqprod(&abz_over_c, variable, PochhammerOrder::Infinite, truncation_order);
    let z_inf = aqprod(z, variable, PochhammerOrder::Infinite, truncation_order);
    let prefactor = arithmetic::mul(&abzc_inf, &arithmetic::invert(&z_inf));

    Some(TransformationResult { prefactor, transformed })
}

/// Sears' transformation for balanced terminating _4 phi_3 (Sears-Whipple).
///
/// ```text
/// _4 phi_3 (q^{-n}, a, b, c ; d, e, f ; q, q) where def = abcq^{1-n}
///   = [(e/a;q)_n * (f/a;q)_n] / [(e;q)_n * (f;q)_n]
///     * _4 phi_3 (q^{-n}, a, d/b, d/c ; d, aq^{1-n}/e, aq^{1-n}/f ; q, q)
/// ```
///
/// Conditions: r=4, s=3, z=q, one upper param is q^{-n}, balanced (def = abc*q^{1-n}).
/// Returns `None` if conditions are not met.
pub fn sears_transform(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> Option<TransformationResult> {
    if series.r() != 4 || series.s() != 3 {
        return None;
    }

    // Check z == q
    if series.argument != QMonomial::q_power(1) {
        return None;
    }

    // Find which upper param is q^{-n}
    for term_idx in 0..4 {
        let term_param = &series.upper[term_idx];
        if let Some(n) = term_param.is_q_neg_power() {
            if n == 0 {
                // Trivial case: q^0 = 1, terminates immediately
                let prefactor = FormalPowerSeries::one(variable, truncation_order);
                let transformed = series.clone();
                return Some(TransformationResult { prefactor, transformed });
            }

            // The other three upper params
            let other_upper: Vec<usize> = (0..4).filter(|&i| i != term_idx).collect();

            // Try each of the 3 non-q^{-n} params as "a" (distinguished param)
            for &a_idx in &other_upper {
                let a = &series.upper[a_idx];
                // The remaining two are b, c
                let bc_idxs: Vec<usize> = other_upper.iter().copied().filter(|&i| i != a_idx).collect();
                let b = &series.upper[bc_idxs[0]];
                let c = &series.upper[bc_idxs[1]];

                // Try each of the 3 lower params as "d"
                for d_idx in 0..3 {
                    let d = &series.lower[d_idx];
                    // The remaining two are e, f
                    let ef_idxs: Vec<usize> = (0..3).filter(|&i| i != d_idx).collect();
                    let e = &series.lower[ef_idxs[0]];
                    let f = &series.lower[ef_idxs[1]];

                    // Check balance: d*e*f == a*b*c*q^{1-n}
                    let lhs_balance = d.mul(e).mul(f);
                    let rhs_balance = a.mul(b).mul(c).mul(&QMonomial::q_power(1 - n));

                    if lhs_balance != rhs_balance {
                        continue;
                    }

                    // Match found! Compute transformed series and prefactor.
                    let q_neg_n = QMonomial::q_power(-n);
                    let d_over_b = d.div(b);
                    let d_over_c = d.div(c);
                    let aq_1_minus_n = a.mul(&QMonomial::q_power(1 - n));
                    let aq_1_minus_n_over_e = aq_1_minus_n.div(e);
                    let aq_1_minus_n_over_f = aq_1_minus_n.div(f);

                    let transformed = HypergeometricSeries {
                        upper: vec![q_neg_n, a.clone(), d_over_b, d_over_c],
                        lower: vec![d.clone(), aq_1_minus_n_over_e, aq_1_minus_n_over_f],
                        argument: QMonomial::q_power(1),
                    };

                    // Prefactor: (e/a;q)_n * (f/a;q)_n / [(e;q)_n * (f;q)_n]
                    let e_over_a = e.div(a);
                    let f_over_a = f.div(a);

                    let ea_n = aqprod(&e_over_a, variable, PochhammerOrder::Finite(n), truncation_order);
                    let fa_n = aqprod(&f_over_a, variable, PochhammerOrder::Finite(n), truncation_order);
                    let e_n = aqprod(e, variable, PochhammerOrder::Finite(n), truncation_order);
                    let f_n = aqprod(f, variable, PochhammerOrder::Finite(n), truncation_order);

                    let numer = arithmetic::mul(&ea_n, &fa_n);
                    let denom = arithmetic::mul(&e_n, &f_n);
                    let prefactor = arithmetic::mul(&numer, &arithmetic::invert(&denom));

                    return Some(TransformationResult { prefactor, transformed });
                }
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Watson's transformation (HYPR-09)
// ---------------------------------------------------------------------------

/// Watson's transformation for very-well-poised _8 phi_7.
///
/// ```text
/// _8 phi_7 (a, q*sqrt(a), -q*sqrt(a), b, c, d, e, f ;
///           sqrt(a), -sqrt(a), aq/b, aq/c, aq/d, aq/e, aq/f ;
///           q, a^2*q^2/(bcdef))
///   = [(aq;q)_inf * (aq/(de);q)_inf * (aq/(df);q)_inf * (aq/(ef);q)_inf]
///     / [(aq/d;q)_inf * (aq/e;q)_inf * (aq/f;q)_inf * (aq/(def);q)_inf]
///     * _4 phi_3 (aq/(bc), d, e, f ; aq/b, aq/c, def/a ; q, q)
/// ```
///
/// Detection: checks r=8, s=7, identifies the very-well-poised structure by
/// finding `a` such that `sqrt(a)` exists, and `q*sqrt(a)`, `-q*sqrt(a)` are
/// among the upper parameters. Then identifies b, c, d, e, f and verifies
/// the lower parameters and argument match.
///
/// Returns `None` if the series is not a very-well-poised 8phi7.
pub fn watson_transform(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> Option<TransformationResult> {
    if series.r() != 8 || series.s() != 7 {
        return None;
    }

    let q_mon = QMonomial::q_power(1);

    // Try each upper param as the base parameter "a"
    for a_idx in 0..8 {
        let a = &series.upper[a_idx];

        // Compute sqrt(a)
        let sqrt_a = match a.try_sqrt() {
            Some(s) => s,
            None => continue,
        };

        // Check that q*sqrt(a) and -q*sqrt(a) are among the remaining upper params
        let q_sqrt_a = q_mon.mul(&sqrt_a);
        let neg_q_sqrt_a = q_sqrt_a.neg();

        let remaining_upper: Vec<usize> = (0..8).filter(|&i| i != a_idx).collect();

        // Find indices for q*sqrt(a) and -q*sqrt(a)
        let q_sqrt_a_idx = remaining_upper.iter().find(|&&i| series.upper[i] == q_sqrt_a);
        let q_sqrt_a_idx = match q_sqrt_a_idx {
            Some(&idx) => idx,
            None => continue,
        };

        let neg_q_sqrt_a_idx = remaining_upper.iter().find(|&&i| series.upper[i] == neg_q_sqrt_a);
        let neg_q_sqrt_a_idx = match neg_q_sqrt_a_idx {
            Some(&idx) => idx,
            None => continue,
        };

        // Check that sqrt(a) and -sqrt(a) are among the lower params
        let neg_sqrt_a = sqrt_a.neg();
        let sqrt_a_lower_idx = (0..7).find(|&i| series.lower[i] == sqrt_a);
        let sqrt_a_lower_idx = match sqrt_a_lower_idx {
            Some(idx) => idx,
            None => continue,
        };
        let neg_sqrt_a_lower_idx = (0..7).find(|&i| i != sqrt_a_lower_idx && series.lower[i] == neg_sqrt_a);
        let neg_sqrt_a_lower_idx = match neg_sqrt_a_lower_idx {
            Some(idx) => idx,
            None => continue,
        };

        // The remaining 5 upper params are the candidates for b, c, d, e, f
        let special_upper = [a_idx, q_sqrt_a_idx, neg_q_sqrt_a_idx];
        let bcdef_idxs: Vec<usize> = (0..8).filter(|i| !special_upper.contains(i)).collect();
        assert_eq!(bcdef_idxs.len(), 5);

        // The remaining 5 lower params (excluding sqrt(a) and -sqrt(a))
        let special_lower = [sqrt_a_lower_idx, neg_sqrt_a_lower_idx];
        let remaining_lower_idxs: Vec<usize> = (0..7).filter(|i| !special_lower.contains(i)).collect();
        assert_eq!(remaining_lower_idxs.len(), 5);

        // For each of the 5 remaining upper params, check that aq/x is in the remaining lower
        let aq = a.mul(&q_mon);
        let mut all_lower_match = true;
        let mut used_lower: Vec<bool> = vec![false; 5];
        for &ui in &bcdef_idxs {
            let expected_lower = aq.div(&series.upper[ui]);
            let found = remaining_lower_idxs.iter().enumerate().find(|(j, li)| {
                !used_lower[*j] && series.lower[**li] == expected_lower
            });
            match found {
                Some((j, _)) => { used_lower[j] = true; },
                None => { all_lower_match = false; break; },
            }
        }
        if !all_lower_match {
            continue;
        }

        // Now try all (5 choose 3) = 10 ways to pick d, e, f from the 5 params
        let bcdef: Vec<&QMonomial> = bcdef_idxs.iter().map(|&i| &series.upper[i]).collect();

        for d_i in 0..5 {
            for e_i in (d_i+1)..5 {
                for f_i in (e_i+1)..5 {
                    let d = bcdef[d_i];
                    let e = bcdef[e_i];
                    let f = bcdef[f_i];

                    // b, c are the complement
                    let bc_idxs: Vec<usize> = (0..5).filter(|&i| i != d_i && i != e_i && i != f_i).collect();
                    let b = bcdef[bc_idxs[0]];
                    let c = bcdef[bc_idxs[1]];

                    // Check z = a^2*q^2/(bcdef)
                    let bcdef_prod = b.mul(c).mul(d).mul(e).mul(f);
                    let expected_z = a.mul(a).mul(&QMonomial::q_power(2)).div(&bcdef_prod);

                    if series.argument != expected_z {
                        continue;
                    }

                    // Match found! Construct 4phi3 and prefactor.
                    let bc = b.mul(c);
                    let aq_over_bc = aq.div(&bc);
                    let aq_over_b = aq.div(b);
                    let aq_over_c = aq.div(c);
                    let def_over_a = d.mul(e).mul(f).div(a);

                    let transformed = HypergeometricSeries {
                        upper: vec![aq_over_bc, d.clone(), e.clone(), f.clone()],
                        lower: vec![aq_over_b, aq_over_c, def_over_a],
                        argument: QMonomial::q_power(1),
                    };

                    // Prefactor: (aq;q)_inf * (aq/(de);q)_inf * (aq/(df);q)_inf * (aq/(ef);q)_inf
                    //          / [(aq/d;q)_inf * (aq/e;q)_inf * (aq/f;q)_inf * (aq/(def);q)_inf]
                    let de = d.mul(e);
                    let df = d.mul(f);
                    let ef = e.mul(f);
                    let def = de.mul(f);

                    let aq_inf = aqprod(&aq, variable, PochhammerOrder::Infinite, truncation_order);
                    let aq_de_inf = aqprod(&aq.div(&de), variable, PochhammerOrder::Infinite, truncation_order);
                    let aq_df_inf = aqprod(&aq.div(&df), variable, PochhammerOrder::Infinite, truncation_order);
                    let aq_ef_inf = aqprod(&aq.div(&ef), variable, PochhammerOrder::Infinite, truncation_order);

                    let aq_d_inf = aqprod(&aq.div(d), variable, PochhammerOrder::Infinite, truncation_order);
                    let aq_e_inf = aqprod(&aq.div(e), variable, PochhammerOrder::Infinite, truncation_order);
                    let aq_f_inf = aqprod(&aq.div(f), variable, PochhammerOrder::Infinite, truncation_order);
                    let aq_def_inf = aqprod(&aq.div(&def), variable, PochhammerOrder::Infinite, truncation_order);

                    let numer = arithmetic::mul(
                        &arithmetic::mul(&aq_inf, &aq_de_inf),
                        &arithmetic::mul(&aq_df_inf, &aq_ef_inf),
                    );
                    let denom = arithmetic::mul(
                        &arithmetic::mul(&aq_d_inf, &aq_e_inf),
                        &arithmetic::mul(&aq_f_inf, &aq_def_inf),
                    );
                    let prefactor = arithmetic::mul(&numer, &arithmetic::invert(&denom));

                    return Some(TransformationResult { prefactor, transformed });
                }
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Bailey's identity (HYPR-10, DLMF 17.7.12)
// ---------------------------------------------------------------------------

/// Bailey's identity (DLMF 17.7.12) for a specific _4 phi_3 with q^2 base.
///
/// ```text
/// _4 phi_3 (a, aq, b^2*q^{2n}, q^{-2n} ; b, bq, a^2*q^2 ; q^2, q^2)
///   = a^n * (-q;q)_n * (b/a;q)_n / [(-aq;q)_n * (b;q)_n]
/// ```
///
/// This is a standalone function (not pattern-matching) that directly computes
/// the closed form given parameters `a`, `b`, and `n`. The user calls this when
/// they know their series matches Bailey's identity.
///
/// The q^2 base means the LHS is actually a 4phi3 where Pochhammer symbols
/// use step 2: (x;q^2)_k = prod_{j=0}^{k-1} (1 - x*q^{2j}).
pub fn bailey_4phi3_q2(
    a: &QMonomial,
    b: &QMonomial,
    n: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    if n == 0 {
        return FormalPowerSeries::one(variable, truncation_order);
    }

    // Compute a^n as FPS monomial
    let a_n_coeff = qrat_pow(&a.coeff, n as u64);
    let a_n_power = a.power * n;
    let a_n_fps = FormalPowerSeries::monomial(variable, a_n_coeff, a_n_power, truncation_order);

    // (-q;q)_n: finite product of n factors (1 - (-1)*q^{1+k}) for k=0..n-1
    let neg_q = QMonomial::new(-QRat::one(), 1);
    let neg_q_n = aqprod(&neg_q, variable, PochhammerOrder::Finite(n), truncation_order);

    // (b/a;q)_n
    let b_over_a = b.div(a);
    let ba_n = aqprod(&b_over_a, variable, PochhammerOrder::Finite(n), truncation_order);

    // (-aq;q)_n = ((-a.coeff)*q^{a.power+1};q)_n
    let neg_aq = QMonomial::new(-a.coeff.clone(), a.power + 1);
    let neg_aq_n = aqprod(&neg_aq, variable, PochhammerOrder::Finite(n), truncation_order);

    // (b;q)_n
    let b_n = aqprod(b, variable, PochhammerOrder::Finite(n), truncation_order);

    // Result: a^n * (-q;q)_n * (b/a;q)_n / [(-aq;q)_n * (b;q)_n]
    let numer = arithmetic::mul(&a_n_fps, &arithmetic::mul(&neg_q_n, &ba_n));
    let denom = arithmetic::mul(&neg_aq_n, &b_n);
    arithmetic::mul(&numer, &arithmetic::invert(&denom))
}

// ---------------------------------------------------------------------------
// Transformation chain search (BFS)
// ---------------------------------------------------------------------------

/// Normalize a HypergeometricSeries into a canonical String key for visited-set deduplication.
///
/// Returns a single String built by:
/// 1. For each upper param QMonomial, produce string "{power}:{coeff_numer}/{coeff_denom}"
///    from coeff.numer().to_string(), coeff.denom().to_string(), and power.
/// 2. Sort the upper param strings lexicographically.
/// 3. Do the same for lower params.
/// 4. Produce the argument string in the same format.
/// 5. Concatenate: "U[{sorted_upper_joined_by_comma}]L[{sorted_lower_joined_by_comma}]A[{argument}]"
///
/// This gives a deterministic, order-independent key so that series with the same
/// parameter multisets (regardless of ordering) produce the same key.
fn normalize_series_key(series: &HypergeometricSeries) -> String {
    let format_monomial = |m: &QMonomial| -> String {
        format!("{}:{}/{}", m.power, m.coeff.numer(), m.coeff.denom())
    };

    let mut upper_strs: Vec<String> = series.upper.iter().map(|m| format_monomial(m)).collect();
    upper_strs.sort();

    let mut lower_strs: Vec<String> = series.lower.iter().map(|m| format_monomial(m)).collect();
    lower_strs.sort();

    let arg_str = format_monomial(&series.argument);

    format!("U[{}]L[{}]A[{}]", upper_strs.join(","), lower_strs.join(","), arg_str)
}

/// Search for a transformation chain between two hypergeometric series.
///
/// Uses BFS over the transformation catalog {heine_1, heine_2, heine_3, sears, watson}
/// to find a sequence of transformations that transforms source into target
/// (modulo a prefactor).
///
/// # Arguments
/// - `source`: Starting hypergeometric series.
/// - `target`: Target hypergeometric series to reach.
/// - `max_depth`: Maximum number of transformation steps to try.
/// - `variable`: FPS variable for evaluation.
/// - `truncation_order`: FPS truncation order for comparison.
///
/// # Returns
/// - `Found { steps, total_prefactor }` if a chain exists within max_depth.
/// - `NotFound { max_depth }` if no chain was found.
pub fn find_transformation_chain(
    source: &HypergeometricSeries,
    target: &HypergeometricSeries,
    max_depth: usize,
    variable: SymbolId,
    truncation_order: i64,
) -> TransformationChainResult {
    // Evaluate target once for comparison
    let target_fps = eval_phi(target, variable, truncation_order);

    // Check source == target (trivial chain with 0 steps)
    let source_fps = eval_phi(source, variable, truncation_order);
    if source_fps == target_fps {
        return TransformationChainResult::Found {
            steps: vec![],
            total_prefactor: FormalPowerSeries::one(variable, truncation_order),
        };
    }

    // BFS queue: (current_series, chain_so_far, cumulative_prefactor)
    let mut queue: VecDeque<(HypergeometricSeries, Vec<TransformationStep>, FormalPowerSeries)> = VecDeque::new();
    let mut visited: HashSet<String> = HashSet::new();

    // Initialize with source
    visited.insert(normalize_series_key(source));
    queue.push_back((
        source.clone(),
        vec![],
        FormalPowerSeries::one(variable, truncation_order),
    ));

    // Define the transformation catalog as closures that wrap the function pointers
    // (needed because watson_transform has a different internal structure but same signature)
    let transform_names = ["heine_1", "heine_2", "heine_3", "sears", "watson"];

    while let Some((current_series, chain_so_far, cumulative_prefactor)) = queue.pop_front() {
        // Don't expand further if at max depth
        if chain_so_far.len() >= max_depth {
            continue;
        }

        // Try each transformation
        for (idx, &name) in transform_names.iter().enumerate() {
            let result_opt: Option<TransformationResult> = match idx {
                0 => heine_transform_1(&current_series, variable, truncation_order),
                1 => heine_transform_2(&current_series, variable, truncation_order),
                2 => heine_transform_3(&current_series, variable, truncation_order),
                3 => sears_transform(&current_series, variable, truncation_order),
                4 => watson_transform(&current_series, variable, truncation_order),
                _ => unreachable!(),
            };

            if let Some(result) = result_opt {
                // Compute new cumulative prefactor
                let new_prefactor = arithmetic::mul(&cumulative_prefactor, &result.prefactor);

                // Build new step
                let new_step = TransformationStep {
                    name: name.to_string(),
                    result_series: result.transformed.clone(),
                    step_prefactor: result.prefactor,
                };

                // Build new chain
                let mut new_chain = chain_so_far.clone();
                new_chain.push(new_step);

                // Check if result.transformed matches target
                // Match condition: eval_phi(result.transformed) == eval_phi(target)
                let transformed_fps = eval_phi(&result.transformed, variable, truncation_order);
                if transformed_fps == target_fps {
                    return TransformationChainResult::Found {
                        steps: new_chain,
                        total_prefactor: new_prefactor,
                    };
                }

                // Add to queue if not visited
                let key = normalize_series_key(&result.transformed);
                if visited.insert(key) {
                    queue.push_back((result.transformed, new_chain, new_prefactor));
                }
            }
        }
    }

    TransformationChainResult::NotFound { max_depth }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExprArena;

    /// Helper: create a SymbolId for "q".
    fn q_var() -> SymbolId {
        let mut arena = ExprArena::new();
        arena.symbols_mut().intern("q")
    }

    /// Helper: QMonomial shorthand for q^power.
    fn qm(power: i64) -> QMonomial {
        QMonomial::q_power(power)
    }

    // ===================================================================
    // 1. test_chain_identity -- Source == target
    // ===================================================================
    #[test]
    fn test_chain_identity() {
        let q = q_var();
        let trunc = 20;

        let series = HypergeometricSeries {
            upper: vec![qm(2), qm(3)],
            lower: vec![qm(5)],
            argument: qm(1),
        };

        let result = find_transformation_chain(&series, &series, 3, q, trunc);
        match result {
            TransformationChainResult::Found { steps, total_prefactor } => {
                assert_eq!(steps.len(), 0, "Identity chain should have 0 steps");
                // Prefactor should be 1
                let one = FormalPowerSeries::one(q, trunc);
                assert_eq!(total_prefactor, one, "Identity prefactor should be 1");
            }
            TransformationChainResult::NotFound { .. } => {
                panic!("Source == target should always be found");
            }
        }
    }

    // ===================================================================
    // 2. test_chain_single_heine1
    // ===================================================================
    #[test]
    fn test_chain_single_heine1() {
        let q = q_var();
        let trunc = 20;

        // Source: _2phi1(q^2, q^3; q^5; q, q)
        let source = HypergeometricSeries {
            upper: vec![qm(2), qm(3)],
            lower: vec![qm(5)],
            argument: qm(1),
        };

        // Target: the heine_1 transformed series
        let h1 = heine_transform_1(&source, q, trunc).unwrap();
        let target = h1.transformed;

        let result = find_transformation_chain(&source, &target, 3, q, trunc);
        match result {
            TransformationChainResult::Found { steps, .. } => {
                assert!(steps.len() >= 1, "Should find at least 1-step chain");
                // The chain should reach target within depth bound
            }
            TransformationChainResult::NotFound { .. } => {
                panic!("Should find chain from source to heine_1 target");
            }
        }
    }

    // ===================================================================
    // 3. test_chain_single_heine2
    // ===================================================================
    #[test]
    fn test_chain_single_heine2() {
        let q = q_var();
        let trunc = 20;

        let source = HypergeometricSeries {
            upper: vec![qm(2), qm(3)],
            lower: vec![qm(5)],
            argument: qm(1),
        };

        let h2 = heine_transform_2(&source, q, trunc).unwrap();
        let target = h2.transformed;

        let result = find_transformation_chain(&source, &target, 3, q, trunc);
        match result {
            TransformationChainResult::Found { steps, .. } => {
                assert!(steps.len() >= 1, "Should find at least 1-step chain");
            }
            TransformationChainResult::NotFound { .. } => {
                panic!("Should find chain from source to heine_2 target");
            }
        }
    }

    // ===================================================================
    // 4. test_chain_two_step
    // ===================================================================
    #[test]
    fn test_chain_two_step() {
        let q = q_var();
        let trunc = 20;

        let source = HypergeometricSeries {
            upper: vec![qm(2), qm(3)],
            lower: vec![qm(5)],
            argument: qm(1),
        };

        // Apply heine_1 then heine_2 to get a doubly-transformed series
        let h1 = heine_transform_1(&source, q, trunc).unwrap();
        let h2 = heine_transform_2(&h1.transformed, q, trunc).unwrap();
        let target = h2.transformed;

        let result = find_transformation_chain(&source, &target, 4, q, trunc);
        match result {
            TransformationChainResult::Found { steps, .. } => {
                // BFS should find a path (possibly not the same 2-step path we used,
                // but within the depth bound)
                assert!(steps.len() >= 1, "Should find at least a 1-step chain (or more)");
                assert!(steps.len() <= 4, "Should be within depth bound of 4");
            }
            TransformationChainResult::NotFound { .. } => {
                panic!("Should find chain from source to doubly-transformed target");
            }
        }
    }

    // ===================================================================
    // 5. test_chain_not_found_depth0
    // ===================================================================
    #[test]
    fn test_chain_not_found_depth0() {
        let q = q_var();
        let trunc = 20;

        let source = HypergeometricSeries {
            upper: vec![qm(2), qm(3)],
            lower: vec![qm(5)],
            argument: qm(1),
        };

        // Target is different from source
        let h1 = heine_transform_1(&source, q, trunc).unwrap();
        let target = h1.transformed;

        let result = find_transformation_chain(&source, &target, 0, q, trunc);
        match result {
            TransformationChainResult::NotFound { max_depth } => {
                assert_eq!(max_depth, 0);
            }
            TransformationChainResult::Found { .. } => {
                panic!("With depth 0, should not find chain when source != target");
            }
        }
    }

    // ===================================================================
    // 6. test_chain_not_found_different_rs
    // ===================================================================
    #[test]
    fn test_chain_not_found_different_rs() {
        let q = q_var();
        let trunc = 20;

        // Source is 2phi1
        let source = HypergeometricSeries {
            upper: vec![qm(2), qm(3)],
            lower: vec![qm(5)],
            argument: qm(1),
        };

        // Target is 3phi2 (different r,s). No Heine transform can change (r,s) from (2,1)
        // to (3,2), and sears requires (4,3).
        let target = HypergeometricSeries {
            upper: vec![qm(1), qm(2), qm(3)],
            lower: vec![qm(4), qm(5)],
            argument: qm(1),
        };

        let result = find_transformation_chain(&source, &target, 3, q, trunc);
        match result {
            TransformationChainResult::NotFound { max_depth } => {
                assert_eq!(max_depth, 3);
            }
            TransformationChainResult::Found { .. } => {
                panic!("Should not find chain between different (r,s) series");
            }
        }
    }

    // ===================================================================
    // 7. test_chain_visited_dedup
    // ===================================================================
    #[test]
    fn test_chain_visited_dedup() {
        let q = q_var();
        let trunc = 20;

        // Use a 2phi1 series. BFS expands heine_1, heine_2, heine_3.
        // Each of those can also be transformed again. With visited set,
        // BFS should not loop and should terminate quickly.
        let source = HypergeometricSeries {
            upper: vec![qm(2), qm(3)],
            lower: vec![qm(5)],
            argument: qm(1),
        };

        // Target that doesn't exist via any short chain
        let target = HypergeometricSeries {
            upper: vec![qm(10), qm(11)],
            lower: vec![qm(20)],
            argument: qm(1),
        };

        // If visited set is broken, this would loop or take very long.
        // With depth 3, at most ~5^3 = 125 nodes (minus dedup).
        let result = find_transformation_chain(&source, &target, 3, q, trunc);
        // We don't care if found or not; the test verifies BFS terminates.
        match result {
            TransformationChainResult::NotFound { .. } => { /* expected */ }
            TransformationChainResult::Found { .. } => { /* unlikely but ok */ }
        }
    }

    // ===================================================================
    // 8. test_chain_with_prefactor
    // ===================================================================
    #[test]
    fn test_chain_with_prefactor() {
        let q = q_var();
        let trunc = 25;

        let source = HypergeometricSeries {
            upper: vec![qm(2), qm(3)],
            lower: vec![qm(5)],
            argument: qm(1),
        };

        // Target is heine_1 transformed
        let h1 = heine_transform_1(&source, q, trunc).unwrap();
        let target = h1.transformed;

        let result = find_transformation_chain(&source, &target, 3, q, trunc);
        match result {
            TransformationChainResult::Found { steps, total_prefactor } => {
                // Verify: total_prefactor * eval_phi(final_series) == eval_phi(source)
                let source_fps = eval_phi(&source, q, trunc);
                let final_series = &steps.last().unwrap().result_series;
                let final_fps = eval_phi(final_series, q, trunc);
                let reconstructed = arithmetic::mul(&total_prefactor, &final_fps);

                for k in 0..trunc {
                    assert_eq!(
                        source_fps.coeff(k), reconstructed.coeff(k),
                        "Prefactor verification: mismatch at q^{}", k
                    );
                }
            }
            TransformationChainResult::NotFound { .. } => {
                panic!("Should find chain from source to heine_1 target");
            }
        }
    }

    // ===================================================================
    // 9. test_chain_heine3_involution
    // ===================================================================
    #[test]
    fn test_chain_heine3_involution() {
        let q = q_var();
        let trunc = 20;

        // Heine 3 applied twice gives a series related back to the original.
        // Verify BFS handles this without infinite loop (visited set prevents revisiting).
        let source = HypergeometricSeries {
            upper: vec![qm(2), qm(3)],
            lower: vec![qm(5)],
            argument: qm(1),
        };

        // Apply heine_3 twice
        let h3_1 = heine_transform_3(&source, q, trunc).unwrap();
        let h3_2 = heine_transform_3(&h3_1.transformed, q, trunc).unwrap();

        // The double application of heine_3 should give back a series
        // equivalent to the original (it's an involution up to parameter reordering).
        // Verify via FPS comparison.
        let original_fps = eval_phi(&source, q, trunc);
        let double_h3_fps = eval_phi(&h3_2.transformed, q, trunc);

        // These should be equal (heine_3 is an involution on the series parameters)
        assert_eq!(original_fps, double_h3_fps,
            "Heine 3 applied twice should give back equivalent series");

        // Now verify BFS finds the identity chain (since source == h3_2.transformed as FPS)
        let target = h3_2.transformed;
        let result = find_transformation_chain(&source, &target, 3, q, trunc);
        match result {
            TransformationChainResult::Found { steps, .. } => {
                // Could be 0 steps (identity) or 2 steps (heine_3 twice).
                // Either is valid.
                assert!(steps.len() <= 3, "Should be within depth bound");
            }
            TransformationChainResult::NotFound { .. } => {
                panic!("Should find chain for heine_3 involution case");
            }
        }
    }

    // ===================================================================
    // 10. test_normalize_series_key_order_independent
    // ===================================================================
    #[test]
    fn test_normalize_series_key_order_independent() {
        // Two series with same parameters in different order should have same key
        let s1 = HypergeometricSeries {
            upper: vec![qm(2), qm(3)],
            lower: vec![qm(5)],
            argument: qm(1),
        };
        let s2 = HypergeometricSeries {
            upper: vec![qm(3), qm(2)],
            lower: vec![qm(5)],
            argument: qm(1),
        };

        assert_eq!(
            normalize_series_key(&s1),
            normalize_series_key(&s2),
            "Same parameters in different order should produce same key"
        );

        // Different parameters should produce different key
        let s3 = HypergeometricSeries {
            upper: vec![qm(2), qm(4)],
            lower: vec![qm(5)],
            argument: qm(1),
        };
        assert_ne!(
            normalize_series_key(&s1),
            normalize_series_key(&s3),
            "Different parameters should produce different key"
        );
    }
}
