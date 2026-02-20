//! Formal power series with sparse rational coefficients.
//!
//! A `FormalPowerSeries` represents f(q) = sum_{k} c_k * q^k + O(q^N)
//! where only nonzero coefficients are stored (via `BTreeMap<i64, QRat>`).
//!
//! Invariants:
//! - All keys in `coefficients` are `< truncation_order`
//! - Missing keys have coefficient 0
//! - No key maps to `QRat::zero()` (enforced on insertion)
//! - `truncation_order` is always tracked explicitly

pub mod arithmetic;
pub mod bivariate;
pub mod display;
pub mod generator;

use std::collections::BTreeMap;

use crate::number::QRat;
use crate::symbol::SymbolId;

/// A formal power series in a single variable with sparse rational coefficients.
///
/// Represents f(q) = sum_{k=min_order}^{truncation_order-1} c_k * q^k + O(q^truncation_order)
///
/// Invariants:
/// - All keys in `coefficients` are < `truncation_order`
/// - Missing keys have coefficient 0
/// - No key maps to QRat::zero() (enforce on insertion)
/// - `truncation_order` is always tracked explicitly
#[derive(Clone, Debug)]
pub struct FormalPowerSeries {
    /// Sparse coefficients: exponent -> nonzero coefficient value
    pub(crate) coefficients: BTreeMap<i64, QRat>,
    /// Variable this series is in (usually "q")
    pub(crate) variable: SymbolId,
    /// Coefficients are exact for exponents < truncation_order.
    /// Everything at or above this order is unknown.
    pub(crate) truncation_order: i64,
}

impl FormalPowerSeries {
    /// Create the zero series: 0 + O(q^N)
    pub fn zero(variable: SymbolId, truncation_order: i64) -> Self {
        Self {
            coefficients: BTreeMap::new(),
            variable,
            truncation_order,
        }
    }

    /// Create the constant 1 series: 1 + O(q^N)
    pub fn one(variable: SymbolId, truncation_order: i64) -> Self {
        let mut fps = Self::zero(variable, truncation_order);
        if truncation_order > 0 {
            fps.coefficients.insert(0, QRat::one());
        }
        fps
    }

    /// Create a monomial: c * q^k + O(q^N)
    pub fn monomial(variable: SymbolId, coeff: QRat, power: i64, truncation_order: i64) -> Self {
        let mut fps = Self::zero(variable, truncation_order);
        if !coeff.is_zero() && power < truncation_order {
            fps.coefficients.insert(power, coeff);
        }
        fps
    }

    /// Construct from a coefficient map directly, stripping zero entries.
    pub fn from_coeffs(
        variable: SymbolId,
        coeffs: BTreeMap<i64, QRat>,
        truncation_order: i64,
    ) -> Self {
        let mut filtered = BTreeMap::new();
        for (k, v) in coeffs {
            if k < truncation_order && !v.is_zero() {
                filtered.insert(k, v);
            }
        }
        Self {
            coefficients: filtered,
            variable,
            truncation_order,
        }
    }

    /// Get coefficient of q^k. Returns QRat::zero() for missing entries.
    /// Panics if k >= truncation_order (coefficient is unknown).
    pub fn coeff(&self, k: i64) -> QRat {
        assert!(
            k < self.truncation_order,
            "Cannot access coefficient at q^{}: series only known to O(q^{})",
            k,
            self.truncation_order
        );
        self.coefficients
            .get(&k)
            .cloned()
            .unwrap_or_else(QRat::zero)
    }

    /// Set coefficient of q^k. Removes entry if value is zero.
    /// Ignores if k >= truncation_order (beyond truncation).
    pub fn set_coeff(&mut self, k: i64, value: QRat) {
        if k >= self.truncation_order {
            return;
        }
        if value.is_zero() {
            self.coefficients.remove(&k);
        } else {
            self.coefficients.insert(k, value);
        }
    }

    /// Number of nonzero coefficients stored.
    pub fn num_nonzero(&self) -> usize {
        self.coefficients.len()
    }

    /// Lowest power with nonzero coefficient, or None if zero series.
    pub fn min_order(&self) -> Option<i64> {
        self.coefficients.keys().next().copied()
    }

    /// The truncation order N: series is known exactly for exponents < N.
    pub fn truncation_order(&self) -> i64 {
        self.truncation_order
    }

    /// True if all coefficients are zero (the zero series).
    pub fn is_zero(&self) -> bool {
        self.coefficients.is_empty()
    }

    /// The variable this series is in.
    pub fn variable(&self) -> SymbolId {
        self.variable
    }

    /// Iterate over nonzero coefficients in ascending exponent order.
    ///
    /// The returned iterator also implements [`DoubleEndedIterator`], so
    /// callers can use `.rev()` for descending-power iteration.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&i64, &QRat)> {
        self.coefficients.iter()
    }
}

impl PartialEq for FormalPowerSeries {
    fn eq(&self, other: &Self) -> bool {
        self.variable == other.variable
            && self.truncation_order == other.truncation_order
            && self.coefficients == other.coefficients
    }
}

impl Eq for FormalPowerSeries {}
