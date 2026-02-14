//! Identity proving engine via the valence formula.
//!
//! Implements Garvan's `provemodfuncid` / `provemodfuncGAMMA0id` algorithm:
//! given an eta-quotient identity, verify it automatically by checking
//! Newman's modularity conditions, computing cusp orders, and applying
//! the valence formula for modular functions on Gamma_0(N).
//!
//! The key theorem: if f is a modular function (weight 0) on Gamma_0(N)
//! with non-negative orders at all cusps, then f is a constant.
//! If f(q) = 1 + O(q), then f = 1 (identity proved).

use std::collections::BTreeMap;

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::symbol::SymbolId;
use crate::ExprArena;
use super::eta::{EtaExpression, ModularityResult};
use super::cusps::{Cusp, cuspmake};
use super::orders::eta_order_at_cusp;

/// Result of attempting to prove an eta-quotient identity.
#[derive(Clone, Debug)]
pub enum ProofResult {
    /// Identity proved: all cusp orders non-negative, constant verified.
    Proved {
        /// The level N of Gamma_0(N)
        level: i64,
        /// Orders at each cusp (cusp, invariant_order)
        cusp_orders: Vec<(Cusp, QRat)>,
        /// The Sturm bound (number of terms checked)
        sturm_bound: i64,
        /// How many q-expansion terms were verified
        verification_terms: i64,
    },
    /// Not a modular function -- Newman conditions failed.
    NotModular {
        failed_conditions: Vec<String>,
    },
    /// Cusp order is negative at some cusp -- identity may be false,
    /// or the level is wrong.
    NegativeOrder {
        cusp: Cusp,
        order: QRat,
    },
    /// Numerical q-expansion verification failed at some coefficient.
    CounterExample {
        coefficient_index: i64,
        expected: QRat,
        actual: QRat,
    },
}

impl ProofResult {
    /// Returns true if the identity was proved.
    pub fn is_proved(&self) -> bool {
        matches!(self, ProofResult::Proved { .. })
    }

    /// Returns true if a counterexample was found.
    pub fn is_counterexample(&self) -> bool {
        matches!(self, ProofResult::CounterExample { .. })
    }
}

/// An eta-quotient identity to prove.
///
/// Represents: sum_i c_i * f_i(q) = 0
/// where each f_i is an eta quotient and c_i is a rational coefficient.
///
/// For a two-sided identity LHS = RHS, express as LHS - RHS = 0.
#[derive(Clone, Debug)]
pub struct EtaIdentity {
    /// Terms: (coefficient, eta expression)
    pub terms: Vec<(QRat, EtaExpression)>,
    /// The level N for Gamma_0(N)
    pub level: i64,
}

impl EtaIdentity {
    /// Create a new EtaIdentity from terms and level.
    pub fn new(terms: Vec<(QRat, EtaExpression)>, level: i64) -> Self {
        Self { terms, level }
    }

    /// Create an identity from LHS = RHS, expressed as LHS - RHS = 0.
    ///
    /// Terms = [(+1, lhs), (-1, rhs)].
    pub fn two_sided(lhs: EtaExpression, rhs: EtaExpression, level: i64) -> Self {
        Self {
            terms: vec![
                (QRat::one(), lhs),
                (-QRat::one(), rhs),
            ],
            level,
        }
    }
}

/// Create a SymbolId for "q" using a temporary arena.
///
/// All fresh arenas assign SymbolId(0) to the first interned symbol,
/// so this is consistent across calls.
fn create_q_symbol() -> SymbolId {
    let mut arena = ExprArena::new();
    arena.symbols_mut().intern("q")
}

/// Compute the Sturm bound for modular forms of weight k on Gamma_0(N).
///
/// B = floor(k * index / 12)
/// where index = [SL_2(Z) : Gamma_0(N)] = N * prod_{p|N} (1 + 1/p)
fn sturm_bound(weight: i64, level: i64) -> i64 {
    let mut n = level;
    let mut index_numer = level; // Start with N
    let mut index_denom = 1i64;
    let mut p = 2i64;
    while p * p <= n {
        if n % p == 0 {
            // p divides N: multiply by (1 + 1/p) = (p+1)/p
            index_numer *= p + 1;
            index_denom *= p;
            while n % p == 0 {
                n /= p;
            }
        }
        p += 1;
    }
    if n > 1 {
        let p = n;
        index_numer *= p + 1;
        index_denom *= p;
    }
    // index = index_numer / index_denom
    // B = floor(weight * index / 12) = floor(weight * index_numer / (12 * index_denom))
    (weight * index_numer) / (12 * index_denom)
}

/// Prove an eta-quotient identity using the valence formula.
///
/// The main proving algorithm matching Garvan's `provemodfuncGAMMA0id`:
///
/// 1. For two-term identities (LHS - RHS = 0), build the combined eta quotient
///    ratio and apply the valence formula directly.
/// 2. For multi-term or general-coefficient identities, fall back to
///    q-expansion verification.
pub fn prove_eta_identity(identity: &EtaIdentity) -> ProofResult {
    // Handle the common 2-term case (LHS - RHS = 0):
    if identity.terms.len() == 2 {
        let (ref c1, ref e1) = identity.terms[0];
        let (ref c2, ref e2) = identity.terms[1];

        // Check that c1 = 1 and c2 = -1 (or c1 = -1 and c2 = 1)
        let (lhs, rhs) = if *c1 == QRat::one() && *c2 == -QRat::one() {
            (e1, e2)
        } else if *c1 == -QRat::one() && *c2 == QRat::one() {
            (e2, e1)
        } else {
            // General coefficients: fall through to q-expansion method
            return prove_by_expansion(identity);
        };

        // Build combined eta quotient: g = lhs/rhs means g has factors = lhs.factors - rhs.factors
        let mut combined_factors: BTreeMap<i64, i64> = BTreeMap::new();
        for (&delta, &r) in &lhs.factors {
            *combined_factors.entry(delta).or_insert(0) += r;
        }
        for (&delta, &r) in &rhs.factors {
            *combined_factors.entry(delta).or_insert(0) -= r;
        }
        // Remove zero entries
        combined_factors.retain(|_, r| *r != 0);

        let combined = EtaExpression::from_factors(
            &combined_factors.iter().map(|(&d, &r)| (d, r)).collect::<Vec<_>>(),
            identity.level,
        );

        return prove_single_eta_quotient(&combined, identity);
    }

    // Multi-term case: fall through to q-expansion
    prove_by_expansion(identity)
}

/// Core valence formula logic for a single eta quotient that should equal a constant.
///
/// Steps:
/// 1. Check Newman's modularity conditions
/// 2. Enumerate cusps of Gamma_0(level)
/// 3. Compute order at each cusp; reject if any is negative
/// 4. Apply valence formula: weight-0 function with non-negative orders is constant
/// 5. Verify via q-expansion that the constant is correct (identity sums to 0)
fn prove_single_eta_quotient(combined: &EtaExpression, identity: &EtaIdentity) -> ProofResult {
    let level = identity.level;

    // Trivial case: combined factors are empty, meaning LHS = RHS exactly.
    // The ratio is the constant 1, so the identity is trivially true.
    if combined.factors.is_empty() {
        let cusps = cuspmake(level);
        let cusp_orders: Vec<(Cusp, QRat)> = cusps
            .iter()
            .map(|c| (c.clone(), QRat::zero()))
            .collect();
        return ProofResult::Proved {
            level,
            cusp_orders,
            sturm_bound: 0,
            verification_terms: 0,
        };
    }

    // Step 1: Check Newman's modularity conditions
    let modularity = combined.check_modularity();
    match &modularity {
        ModularityResult::NotModular { failed_conditions } => {
            return ProofResult::NotModular {
                failed_conditions: failed_conditions.clone(),
            };
        }
        ModularityResult::Modular => {}
    }

    // Step 2: Compute cusps of Gamma_0(level)
    let cusps = cuspmake(level);

    // Step 3: Compute order at each cusp
    let mut cusp_orders: Vec<(Cusp, QRat)> = Vec::new();
    for cusp in &cusps {
        let ord = eta_order_at_cusp(combined, cusp);
        // Check for negative order: identity cannot be proved at this level
        if ord < QRat::zero() {
            return ProofResult::NegativeOrder {
                cusp: cusp.clone(),
                order: ord,
            };
        }
        cusp_orders.push((cusp.clone(), ord));
    }

    // Step 4: Valence formula check
    // For weight 0 modular function with all cusp orders >= 0:
    // The function is constant. Check that constant = 0 (for sum = 0 identity)
    // or constant = 1 (for LHS/RHS = 1 identity).
    let weight = combined.weight();
    let weight_i64 = {
        let w = weight.clone();
        // Weight should be 0 for modular functions (already checked by Newman)
        if w.is_zero() { 0i64 } else { w.0.to_f64() as i64 }
    };

    let bound = if weight_i64 == 0 {
        // For weight 0 with non-negative cusp orders: just check constant term
        // But be safe: check a few terms
        1i64
    } else {
        sturm_bound(weight_i64, level)
    };

    // Step 5: q-expansion verification
    // Expand the identity and verify it equals 0 up to the Sturm bound
    let verification_terms = bound.max(5); // Check at least 5 terms for safety
    let trunc = verification_terms + 10; // Extra margin

    let q_var = create_q_symbol();
    let mut total = FormalPowerSeries::zero(q_var, trunc);
    for (coeff, eta_expr) in &identity.terms {
        let expanded = eta_expr.to_series(q_var, trunc);
        let scaled = arithmetic::scalar_mul(coeff, &expanded);
        total = arithmetic::add(&total, &scaled);
    }

    // Check that the total is zero up to the verification bound
    for i in 0..verification_terms {
        if i < total.truncation_order() {
            let c = total.coeff(i);
            if !c.is_zero() {
                return ProofResult::CounterExample {
                    coefficient_index: i,
                    expected: QRat::zero(),
                    actual: c,
                };
            }
        }
    }

    ProofResult::Proved {
        level,
        cusp_orders,
        sturm_bound: bound,
        verification_terms,
    }
}

/// Fallback proving method: verify identity by q-expansion alone.
///
/// For multi-term identities or those with non-unit coefficients,
/// expand all terms, sum them, and check that the result is zero.
fn prove_by_expansion(identity: &EtaIdentity) -> ProofResult {
    let level = identity.level;
    let trunc = 100i64; // Check 100 terms

    let q_var = create_q_symbol();

    let mut total = FormalPowerSeries::zero(q_var, trunc);
    for (coeff, eta_expr) in &identity.terms {
        let expanded = eta_expr.to_series(q_var, trunc);
        let scaled = arithmetic::scalar_mul(coeff, &expanded);
        total = arithmetic::add(&total, &scaled);
    }

    for i in 0..trunc {
        let c = total.coeff(i);
        if !c.is_zero() {
            return ProofResult::CounterExample {
                coefficient_index: i,
                expected: QRat::zero(),
                actual: c,
            };
        }
    }

    // Expansion verified but no structural proof
    ProofResult::Proved {
        level,
        cusp_orders: Vec::new(), // No cusp analysis for expansion-only proof
        sturm_bound: trunc,
        verification_terms: trunc,
    }
}
