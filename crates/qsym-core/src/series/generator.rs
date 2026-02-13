//! Lazy generators for infinite product expansion.
//!
//! An `InfiniteProductGenerator` lazily computes the series expansion of an
//! infinite product by multiplying one factor at a time, truncating at each step.
//!
//! For example, the Euler function (q;q)_inf = prod_{k=1}^{inf} (1 - q^k) is
//! computed by starting with 1 and iteratively multiplying by (1 - q^k) for k=1,2,3,...
//! Once k >= target_order, factor k is 1 + O(q^target_order) and doesn't affect
//! the partial product below the truncation order.
//!
//! # Factories
//!
//! - [`euler_function_generator`]: (q;q)_inf = prod_{k=1}^{inf} (1 - q^k)
//! - [`qpochhammer_inf_generator`]: (a*q^offset; q)_inf = prod_{k=0}^{inf} (1 - a * q^{offset+k})

use crate::number::QRat;
use crate::symbol::SymbolId;
use super::{FormalPowerSeries, arithmetic};

/// Lazily generates coefficients of an infinite product by multiplying one factor
/// at a time, truncating at each step.
///
/// The generator maintains a partial product and the count of factors already
/// included. Calling `ensure_order(N)` multiplies in enough additional factors
/// so the partial product is correct to O(q^N).
///
/// # Reuse
///
/// After `ensure_order(5)`, calling `ensure_order(10)` only multiplies factors
/// 6 through 10 -- the first 5 are already included.
pub struct InfiniteProductGenerator {
    /// Current partial product (truncated series).
    partial_product: FormalPowerSeries,
    /// How many factors have been multiplied in. Factor indices
    /// `start_index..factors_included` have been applied.
    factors_included: i64,
    /// Starting factor index (e.g., 1 for (q;q)_inf which starts at k=1).
    start_index: i64,
    /// Function that generates the k-th factor as a FormalPowerSeries.
    /// Arguments: (k: factor index, variable: SymbolId, truncation_order: i64)
    factor_fn: Box<dyn FnMut(i64, SymbolId, i64) -> FormalPowerSeries>,
}

impl InfiniteProductGenerator {
    /// Create a new generator.
    ///
    /// - `initial`: The starting partial product (typically the constant 1 series).
    /// - `start_index`: The first factor index to apply.
    /// - `factor_fn`: A closure that produces the k-th factor as a series.
    pub fn new(
        initial: FormalPowerSeries,
        start_index: i64,
        factor_fn: Box<dyn FnMut(i64, SymbolId, i64) -> FormalPowerSeries>,
    ) -> Self {
        Self {
            partial_product: initial,
            factors_included: start_index, // no factors applied yet
            start_index,
            factor_fn,
        }
    }

    /// Ensure the partial product includes enough factors to be correct
    /// up to O(q^target_order).
    ///
    /// For products like prod_{k=1}^{inf} (1 - q^k), factor k only affects
    /// terms at q^k and above. Once k >= target_order, the factor is
    /// 1 + O(q^target_order), so it doesn't change the partial product below
    /// target_order. Thus we need at most target_order factors.
    pub fn ensure_order(&mut self, target_order: i64) {
        let var = self.partial_product.variable();
        while self.factors_included < target_order {
            let factor = (self.factor_fn)(self.factors_included, var, target_order);
            self.partial_product = arithmetic::mul(&self.partial_product, &factor);
            self.factors_included += 1;
        }
    }

    /// Get a reference to the current partial product series.
    pub fn series(&self) -> &FormalPowerSeries {
        &self.partial_product
    }

    /// Consume the generator and return the owned series.
    pub fn into_series(self) -> FormalPowerSeries {
        self.partial_product
    }

    /// How many factors have been included so far.
    pub fn factors_included(&self) -> i64 {
        self.factors_included
    }

    /// The starting factor index.
    pub fn start_index(&self) -> i64 {
        self.start_index
    }
}

/// Create a generator for the Euler function:
/// (q;q)_inf = prod_{k=1}^{inf} (1 - q^k)
///
/// The Euler function's coefficients encode the pentagonal number theorem:
/// only exponents k(3k-1)/2 (generalized pentagonal numbers) have nonzero
/// coefficients, with signs alternating in pairs: +, -, -, +, +, -, -, ...
///
/// # Arguments
///
/// - `variable`: The SymbolId for the series variable (typically "q").
/// - `truncation_order`: The series is computed to O(q^truncation_order).
pub fn euler_function_generator(
    variable: SymbolId,
    truncation_order: i64,
) -> InfiniteProductGenerator {
    let initial = FormalPowerSeries::one(variable, truncation_order);

    InfiniteProductGenerator::new(
        initial,
        1, // start at k=1 for (q;q)_inf
        Box::new(move |k, var, trunc| {
            // Factor = (1 - q^k)
            let mut factor = FormalPowerSeries::one(var, trunc);
            factor.set_coeff(k, -QRat::one());
            factor
        }),
    )
}

/// Create a generator for a general q-Pochhammer symbol with infinite order:
/// (a*q^offset; q)_inf = prod_{k=0}^{inf} (1 - a * q^{offset + k})
///
/// # Special cases
///
/// - `a = 1, offset = 1`: gives (q;q)_inf (same as `euler_function_generator`).
/// - `a = 1, offset = 0`: gives (1;q)_inf = 0 (since the first factor is 1-1=0).
///
/// # Arguments
///
/// - `a`: The scalar coefficient in each factor.
/// - `offset`: The base exponent; factor k has exponent `offset + k`.
/// - `variable`: The SymbolId for the series variable.
/// - `truncation_order`: The series is computed to O(q^truncation_order).
pub fn qpochhammer_inf_generator(
    a: QRat,
    offset: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> InfiniteProductGenerator {
    let initial = FormalPowerSeries::one(variable, truncation_order);

    InfiniteProductGenerator::new(
        initial,
        0, // start at k=0: first factor is (1 - a * q^{offset+0})
        Box::new(move |k, var, trunc| {
            // Factor = (1 - a * q^{offset+k})
            let exp = offset + k;
            let mut factor = FormalPowerSeries::one(var, trunc);
            if exp < trunc {
                factor.set_coeff(exp, -a.clone());
            }
            factor
        }),
    )
}
