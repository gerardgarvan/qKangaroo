//! Bivariate formal power series: Laurent polynomial in an outer variable
//! with FormalPowerSeries coefficients in the inner (q) variable.
//!
//! Represents f(z, q) = sum_{k} c_k(q) * z^k + O(q^N)
//! where each c_k(q) is a `FormalPowerSeries`.

use std::collections::BTreeMap;

use crate::number::QRat;
use crate::symbol::SymbolId;
use super::FormalPowerSeries;
use super::arithmetic;

/// A bivariate formal power series: Laurent polynomial in `outer_variable`
/// with `FormalPowerSeries` coefficients in the inner variable.
///
/// For example, `(q + q^2)*z + (1 - q)*z^{-1} + O(q^10)` has:
/// - `outer_variable` = "z"
/// - `terms` = {1 -> (q + q^2), -1 -> (1 - q)}
/// - `inner_variable` = SymbolId for "q"
/// - `truncation_order` = 10
#[derive(Clone, Debug)]
pub struct BivariateSeries {
    /// Name of the outer (Laurent) variable (e.g., "z").
    pub outer_variable: String,
    /// Laurent expansion: z-exponent -> FPS coefficient in the inner variable.
    pub terms: BTreeMap<i64, FormalPowerSeries>,
    /// Symbol for the inner variable (e.g., q).
    pub inner_variable: SymbolId,
    /// Truncation order for the inner variable (all FPS coefficients are
    /// known to this precision).
    pub truncation_order: i64,
}

impl BivariateSeries {
    /// Create the zero bivariate series: 0 + O(q^N).
    pub fn zero(outer_variable: String, inner_variable: SymbolId, truncation_order: i64) -> Self {
        Self {
            outer_variable,
            terms: BTreeMap::new(),
            inner_variable,
            truncation_order,
        }
    }

    /// Create a bivariate series with a single z^k term.
    ///
    /// The inner variable and truncation order are extracted from the FPS.
    pub fn from_single_term(outer_variable: String, z_exp: i64, fps: FormalPowerSeries) -> Self {
        let inner_variable = fps.variable();
        let truncation_order = fps.truncation_order();
        let mut terms = BTreeMap::new();
        if !fps.is_zero() {
            terms.insert(z_exp, fps);
        }
        Self {
            outer_variable,
            terms,
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

    /// The name of the outer variable.
    pub fn outer_variable(&self) -> &str {
        &self.outer_variable
    }
}

impl PartialEq for BivariateSeries {
    fn eq(&self, other: &Self) -> bool {
        self.outer_variable == other.outer_variable
            && self.inner_variable == other.inner_variable
            && self.truncation_order == other.truncation_order
            && self.terms == other.terms
    }
}

impl Eq for BivariateSeries {}

// ---------------------------------------------------------------------------
// Arithmetic operations
// ---------------------------------------------------------------------------

/// Negate every FPS coefficient: -f(z, q).
pub fn bivariate_negate(a: &BivariateSeries) -> BivariateSeries {
    let mut terms = BTreeMap::new();
    for (&z_exp, fps) in &a.terms {
        terms.insert(z_exp, arithmetic::negate(fps));
    }
    BivariateSeries {
        outer_variable: a.outer_variable.clone(),
        terms,
        inner_variable: a.inner_variable,
        truncation_order: a.truncation_order,
    }
}

/// Add two bivariate series. Variables must match.
///
/// Merges the BTreeMaps, adding FPS coefficients for matching z-exponents.
/// Zero FPS entries are removed.
pub fn bivariate_add(a: &BivariateSeries, b: &BivariateSeries) -> BivariateSeries {
    assert_eq!(
        a.outer_variable, b.outer_variable,
        "Cannot add bivariate series with different outer variables: '{}' vs '{}'",
        a.outer_variable, b.outer_variable
    );
    assert_eq!(
        a.inner_variable, b.inner_variable,
        "Cannot add bivariate series with different inner variables"
    );
    let trunc = a.truncation_order.min(b.truncation_order);
    let mut terms = BTreeMap::new();

    // Copy terms from a (re-truncated)
    for (&z_exp, fps_a) in &a.terms {
        let truncated = truncate_fps(fps_a, trunc);
        if !truncated.is_zero() {
            terms.insert(z_exp, truncated);
        }
    }
    // Add terms from b
    for (&z_exp, fps_b) in &b.terms {
        let truncated_b = truncate_fps(fps_b, trunc);
        if let Some(existing) = terms.remove(&z_exp) {
            let sum = arithmetic::add(&existing, &truncated_b);
            if !sum.is_zero() {
                terms.insert(z_exp, sum);
            }
        } else if !truncated_b.is_zero() {
            terms.insert(z_exp, truncated_b);
        }
    }

    BivariateSeries {
        outer_variable: a.outer_variable.clone(),
        terms,
        inner_variable: a.inner_variable,
        truncation_order: trunc,
    }
}

/// Subtract two bivariate series: a - b.
pub fn bivariate_sub(a: &BivariateSeries, b: &BivariateSeries) -> BivariateSeries {
    bivariate_add(a, &bivariate_negate(b))
}

/// Multiply two bivariate series (convolution in z-exponents).
///
/// For each pair of terms (z^a, f_a) and (z^b, f_b), contributes
/// f_a(q) * f_b(q) at z^(a+b).
pub fn bivariate_mul(a: &BivariateSeries, b: &BivariateSeries) -> BivariateSeries {
    assert_eq!(
        a.outer_variable, b.outer_variable,
        "Cannot multiply bivariate series with different outer variables: '{}' vs '{}'",
        a.outer_variable, b.outer_variable
    );
    assert_eq!(
        a.inner_variable, b.inner_variable,
        "Cannot multiply bivariate series with different inner variables"
    );
    let trunc = a.truncation_order.min(b.truncation_order);
    let mut terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();

    for (&za, fa) in &a.terms {
        for (&zb, fb) in &b.terms {
            let z_exp = za + zb;
            let product = arithmetic::mul(fa, fb);
            let truncated = truncate_fps(&product, trunc);
            if truncated.is_zero() {
                continue;
            }
            if let Some(existing) = terms.remove(&z_exp) {
                let sum = arithmetic::add(&existing, &truncated);
                if !sum.is_zero() {
                    terms.insert(z_exp, sum);
                }
            } else {
                terms.insert(z_exp, truncated);
            }
        }
    }

    BivariateSeries {
        outer_variable: a.outer_variable.clone(),
        terms,
        inner_variable: a.inner_variable,
        truncation_order: trunc,
    }
}

/// Multiply a bivariate series by a scalar rational number.
pub fn bivariate_scalar_mul(s: &QRat, a: &BivariateSeries) -> BivariateSeries {
    if s.is_zero() {
        return BivariateSeries::zero(
            a.outer_variable.clone(),
            a.inner_variable,
            a.truncation_order,
        );
    }
    let mut terms = BTreeMap::new();
    for (&z_exp, fps) in &a.terms {
        let product = arithmetic::scalar_mul(s, fps);
        if !product.is_zero() {
            terms.insert(z_exp, product);
        }
    }
    BivariateSeries {
        outer_variable: a.outer_variable.clone(),
        terms,
        inner_variable: a.inner_variable,
        truncation_order: a.truncation_order,
    }
}

/// Multiply every FPS coefficient by a given FPS (for Series * BivariateSeries).
///
/// Uses min truncation order between the FPS and the bivariate series.
pub fn bivariate_fps_mul(fps: &FormalPowerSeries, a: &BivariateSeries) -> BivariateSeries {
    let trunc = fps.truncation_order().min(a.truncation_order);
    let mut terms = BTreeMap::new();
    for (&z_exp, coeff) in &a.terms {
        let product = arithmetic::mul(fps, coeff);
        let truncated = truncate_fps(&product, trunc);
        if !truncated.is_zero() {
            terms.insert(z_exp, truncated);
        }
    }
    BivariateSeries {
        outer_variable: a.outer_variable.clone(),
        terms,
        inner_variable: a.inner_variable,
        truncation_order: trunc,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Re-truncate an FPS to a (possibly lower) truncation order.
fn truncate_fps(fps: &FormalPowerSeries, trunc: i64) -> FormalPowerSeries {
    if fps.truncation_order() <= trunc {
        return fps.clone();
    }
    FormalPowerSeries::from_coeffs(
        fps.variable(),
        fps.iter().map(|(&k, v)| (k, v.clone())).collect(),
        trunc,
    )
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

    /// Helper: create a constant FPS = c + O(q^N).
    fn const_fps(sym: SymbolId, c: i64, order: i64) -> FormalPowerSeries {
        FormalPowerSeries::monomial(sym, QRat::from((c, 1i64)), 0, order)
    }

    /// Helper: create a monomial FPS = c*q^k + O(q^N).
    fn mono_fps(sym: SymbolId, c: i64, k: i64, order: i64) -> FormalPowerSeries {
        FormalPowerSeries::monomial(sym, QRat::from((c, 1i64)), k, order)
    }

    #[test]
    fn zero_creation_and_is_zero() {
        let (_reg, sym_q) = test_q();
        let bv = BivariateSeries::zero("z".to_string(), sym_q, 10);
        assert!(bv.is_zero());
        assert_eq!(bv.truncation_order(), 10);
        assert_eq!(bv.outer_variable(), "z");
    }

    #[test]
    fn from_single_term() {
        let (_reg, sym_q) = test_q();
        let fps = mono_fps(sym_q, 1, 1, 10); // q + O(q^10)
        let bv = BivariateSeries::from_single_term("z".to_string(), 2, fps);
        assert!(!bv.is_zero());
        assert_eq!(bv.terms.len(), 1);
        assert_eq!(bv.truncation_order(), 10);
        // The z^2 term is q + O(q^10)
        let coeff = bv.terms.get(&2).unwrap();
        assert_eq!(coeff.coeff(1), QRat::one());
    }

    #[test]
    fn negate_bivariate() {
        let (_reg, sym_q) = test_q();
        let fps = mono_fps(sym_q, 1, 1, 10); // q + O(q^10)
        let bv = BivariateSeries::from_single_term("z".to_string(), 1, fps);
        let neg = bivariate_negate(&bv);
        let coeff = neg.terms.get(&1).unwrap();
        assert_eq!(coeff.coeff(1), -QRat::one());
    }

    #[test]
    fn add_overlapping_and_disjoint() {
        let (_reg, sym_q) = test_q();
        // a = q*z + O(q^10)
        let a = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 1, 1, 10));
        // b = 2*q*z + q*z^{-1} + O(q^10)
        let mut b_terms = BTreeMap::new();
        b_terms.insert(1, mono_fps(sym_q, 2, 1, 10));  // 2q at z^1
        b_terms.insert(-1, mono_fps(sym_q, 1, 1, 10)); // q at z^{-1}
        let b = BivariateSeries {
            outer_variable: "z".to_string(),
            terms: b_terms,
            inner_variable: sym_q,
            truncation_order: 10,
        };
        let sum = bivariate_add(&a, &b);
        assert_eq!(sum.terms.len(), 2); // z^1 and z^{-1}
        // z^1 coeff: q + 2q = 3q
        let z1 = sum.terms.get(&1).unwrap();
        assert_eq!(z1.coeff(1), QRat::from((3i64, 1i64)));
        // z^{-1} coeff: q
        let zm1 = sum.terms.get(&-1).unwrap();
        assert_eq!(zm1.coeff(1), QRat::one());
    }

    #[test]
    fn add_cancellation() {
        let (_reg, sym_q) = test_q();
        // a = q*z + O(q^10)
        let a = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 1, 1, 10));
        // b = -q*z + O(q^10)
        let b = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, -1, 1, 10));
        let sum = bivariate_add(&a, &b);
        // z^1 coeff should cancel to zero and be removed
        assert!(sum.is_zero(), "expected zero after cancellation, got {:?}", sum.terms);
    }

    #[test]
    fn sub_basic() {
        let (_reg, sym_q) = test_q();
        let a = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 3, 1, 10));
        let b = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 1, 1, 10));
        let diff = bivariate_sub(&a, &b);
        let z1 = diff.terms.get(&1).unwrap();
        assert_eq!(z1.coeff(1), QRat::from((2i64, 1i64)));
    }

    #[test]
    fn mul_convolution() {
        let (_reg, sym_q) = test_q();
        // a = 1*z + 1*z^{-1} = z + z^{-1}  (with constant q^0 = 1 coefficients)
        let mut a_terms = BTreeMap::new();
        a_terms.insert(1, const_fps(sym_q, 1, 10));
        a_terms.insert(-1, const_fps(sym_q, 1, 10));
        let a = BivariateSeries {
            outer_variable: "z".to_string(),
            terms: a_terms,
            inner_variable: sym_q,
            truncation_order: 10,
        };
        // b = 1*z - 1*z^{-1} = z - z^{-1}
        let mut b_terms = BTreeMap::new();
        b_terms.insert(1, const_fps(sym_q, 1, 10));
        b_terms.insert(-1, const_fps(sym_q, -1, 10));
        let b = BivariateSeries {
            outer_variable: "z".to_string(),
            terms: b_terms,
            inner_variable: sym_q,
            truncation_order: 10,
        };
        // (z + z^{-1})(z - z^{-1}) = z^2 - z^{-2}
        let product = bivariate_mul(&a, &b);
        assert_eq!(product.terms.len(), 2, "expected 2 terms, got {:?}", product.terms);
        // z^2 coefficient = 1
        let z2 = product.terms.get(&2).unwrap();
        assert_eq!(z2.coeff(0), QRat::one());
        // z^{-2} coefficient = -1
        let zm2 = product.terms.get(&-2).unwrap();
        assert_eq!(zm2.coeff(0), -QRat::one());
        // z^0 terms should cancel (1*(-1) + 1*1 = 0)
        assert!(product.terms.get(&0).is_none(), "z^0 should cancel");
    }

    #[test]
    fn scalar_mul_basic() {
        let (_reg, sym_q) = test_q();
        let a = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 1, 1, 10));
        let two = QRat::from((2i64, 1i64));
        let result = bivariate_scalar_mul(&two, &a);
        let z1 = result.terms.get(&1).unwrap();
        assert_eq!(z1.coeff(1), QRat::from((2i64, 1i64)));
    }

    #[test]
    fn scalar_mul_by_zero() {
        let (_reg, sym_q) = test_q();
        let a = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 1, 1, 10));
        let result = bivariate_scalar_mul(&QRat::zero(), &a);
        assert!(result.is_zero());
    }

    #[test]
    fn fps_mul_basic() {
        let (_reg, sym_q) = test_q();
        // a = q*z + O(q^10)
        let a = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 1, 1, 10));
        // fps = q + O(q^10)
        let fps = mono_fps(sym_q, 1, 1, 10);
        let result = bivariate_fps_mul(&fps, &a);
        // z^1 coeff should be q * q = q^2
        let z1 = result.terms.get(&1).unwrap();
        assert_eq!(z1.coeff(2), QRat::one());
        assert_eq!(z1.coeff(1), QRat::zero());
    }

    #[test]
    #[should_panic(expected = "different outer variables")]
    fn mismatched_outer_variable_panics() {
        let (_reg, sym_q) = test_q();
        let a = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 1, 1, 10));
        let b = BivariateSeries::from_single_term("w".to_string(), 1, mono_fps(sym_q, 1, 1, 10));
        let _ = bivariate_add(&a, &b);
    }

    #[test]
    fn equality() {
        let (_reg, sym_q) = test_q();
        let a = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 1, 1, 10));
        let b = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 1, 1, 10));
        assert_eq!(a, b);
    }

    #[test]
    fn inequality_different_terms() {
        let (_reg, sym_q) = test_q();
        let a = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 1, 1, 10));
        let b = BivariateSeries::from_single_term("z".to_string(), 1, mono_fps(sym_q, 2, 1, 10));
        assert_ne!(a, b);
    }
}
