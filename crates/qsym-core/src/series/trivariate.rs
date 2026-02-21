//! Trivariate formal power series: Laurent polynomial in two outer variables
//! (a, b) with FormalPowerSeries coefficients in the inner (q) variable.
//!
//! Represents f(a, b, q) = sum_{r,s} c_{r,s}(q) * a^r * b^s + O(q^N)

use std::collections::BTreeMap;

use crate::number::QRat;
use crate::symbol::SymbolId;
use super::FormalPowerSeries;
use super::arithmetic;

/// A trivariate formal power series: Laurent polynomial in two outer
/// variables with `FormalPowerSeries` coefficients in the inner variable.
///
/// For example, `(q + q^2)*a^2*b + q*a*b^(-1) + O(q^10)` has:
/// - `outer_var_a` = "a", `outer_var_b` = "b"
/// - `terms` = {(2, 1) -> (q + q^2), (1, -1) -> q}
/// - `inner_variable` = SymbolId for "q"
/// - `truncation_order` = 10
#[derive(Clone, Debug)]
pub struct TrivariateSeries {
    /// Name of the first outer variable (e.g., "a").
    pub outer_var_a: String,
    /// Name of the second outer variable (e.g., "b").
    pub outer_var_b: String,
    /// Terms: (a_exponent, b_exponent) -> FPS coefficient in q.
    pub terms: BTreeMap<(i64, i64), FormalPowerSeries>,
    /// Symbol for the inner variable (e.g., q).
    pub inner_variable: SymbolId,
    /// Truncation order for the inner variable.
    pub truncation_order: i64,
}

impl TrivariateSeries {
    /// Create the zero trivariate series: 0 + O(q^N).
    pub fn zero(
        outer_var_a: String,
        outer_var_b: String,
        inner_variable: SymbolId,
        truncation_order: i64,
    ) -> Self {
        Self {
            outer_var_a,
            outer_var_b,
            terms: BTreeMap::new(),
            inner_variable,
            truncation_order,
        }
    }

    /// True if all coefficients are zero.
    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    /// The truncation order for the inner variable.
    pub fn truncation_order(&self) -> i64 {
        self.truncation_order
    }
}

impl PartialEq for TrivariateSeries {
    fn eq(&self, other: &Self) -> bool {
        self.outer_var_a == other.outer_var_a
            && self.outer_var_b == other.outer_var_b
            && self.inner_variable == other.inner_variable
            && self.truncation_order == other.truncation_order
            && self.terms == other.terms
    }
}

impl Eq for TrivariateSeries {}

// ---------------------------------------------------------------------------
// Arithmetic operations
// ---------------------------------------------------------------------------

/// Negate every FPS coefficient: -f(a, b, q).
pub fn trivariate_negate(a: &TrivariateSeries) -> TrivariateSeries {
    let mut terms = BTreeMap::new();
    for (&key, fps) in &a.terms {
        terms.insert(key, arithmetic::negate(fps));
    }
    TrivariateSeries {
        outer_var_a: a.outer_var_a.clone(),
        outer_var_b: a.outer_var_b.clone(),
        terms,
        inner_variable: a.inner_variable,
        truncation_order: a.truncation_order,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number::QRat;
    use crate::symbol::SymbolRegistry;

    /// Helper to create a test SymbolId for "q".
    fn test_q() -> (SymbolRegistry, SymbolId) {
        let mut reg = SymbolRegistry::new();
        let sym_q = reg.intern("q");
        (reg, sym_q)
    }

    /// Helper: create a monomial FPS = c*q^k + O(q^N).
    fn mono_fps(sym: SymbolId, c: i64, k: i64, order: i64) -> FormalPowerSeries {
        FormalPowerSeries::monomial(sym, QRat::from((c, 1i64)), k, order)
    }

    #[test]
    fn zero_creation_and_is_zero() {
        let (_reg, sym_q) = test_q();
        let ts = TrivariateSeries::zero(
            "a".to_string(),
            "b".to_string(),
            sym_q,
            10,
        );
        assert!(ts.is_zero());
        assert_eq!(ts.truncation_order(), 10);
    }

    #[test]
    fn negate_trivariate() {
        let (_reg, sym_q) = test_q();
        let fps = mono_fps(sym_q, 1, 1, 10); // q + O(q^10)
        let mut terms = BTreeMap::new();
        terms.insert((1, 2), fps);
        let ts = TrivariateSeries {
            outer_var_a: "a".to_string(),
            outer_var_b: "b".to_string(),
            terms,
            inner_variable: sym_q,
            truncation_order: 10,
        };
        let neg = trivariate_negate(&ts);
        let coeff = neg.terms.get(&(1, 2)).unwrap();
        assert_eq!(coeff.coeff(1), -QRat::one());
    }

    #[test]
    fn equality() {
        let (_reg, sym_q) = test_q();
        let fps = mono_fps(sym_q, 1, 1, 10);
        let mut terms1 = BTreeMap::new();
        terms1.insert((1, 0), fps.clone());
        let ts1 = TrivariateSeries {
            outer_var_a: "a".to_string(),
            outer_var_b: "b".to_string(),
            terms: terms1,
            inner_variable: sym_q,
            truncation_order: 10,
        };
        let mut terms2 = BTreeMap::new();
        terms2.insert((1, 0), fps);
        let ts2 = TrivariateSeries {
            outer_var_a: "a".to_string(),
            outer_var_b: "b".to_string(),
            terms: terms2,
            inner_variable: sym_q,
            truncation_order: 10,
        };
        assert_eq!(ts1, ts2);
    }
}
