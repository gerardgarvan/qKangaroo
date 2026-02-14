//! JAC symbolic representation: Jacobi triple products as structured data.
//!
//! JAC(a, b) = (q^a; q^b)_inf * (q^{b-a}; q^b)_inf * (q^b; q^b)_inf
//!
//! A JacExpression represents: scalar * q^shift * prod_i JAC(a_i, b_i)^{e_i}

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::symbol::SymbolId;
use crate::qseries::products::jacprod;

use super::fps_pow;

/// A single JAC(a, b)^exponent factor in a Jacobi product expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JacFactor {
    /// First parameter (must satisfy 0 < a < b).
    pub a: i64,
    /// Modulus parameter.
    pub b: i64,
    /// Power of this JAC factor.
    pub exponent: i64,
}

impl JacFactor {
    /// Create a new JacFactor with validation: requires 0 < a < b.
    pub fn new(a: i64, b: i64, exponent: i64) -> Self {
        assert!(a > 0, "JacFactor::new: a must be > 0, got a={}", a);
        assert!(a < b, "JacFactor::new: a must be < b, got a={}, b={}", a, b);
        Self { a, b, exponent }
    }

    /// Check if this factor has valid parameters: 0 < a < b.
    pub fn is_valid(&self) -> bool {
        self.a > 0 && self.a < self.b
    }
}

/// A symbolic Jacobi product expression: scalar * q^{q_shift} * prod_i JAC(a_i, b_i)^{e_i}.
///
/// This captures the algebraic form without expanding to a formal power series.
#[derive(Clone, Debug)]
pub struct JacExpression {
    /// Rational scalar prefactor.
    pub scalar: QRat,
    /// Fractional power of q prefactor (e.g., 0, 1/24).
    pub q_shift: QRat,
    /// List of JAC(a, b)^exponent factors.
    pub factors: Vec<JacFactor>,
}

impl JacExpression {
    /// Create a new JacExpression from components.
    pub fn new(scalar: QRat, q_shift: QRat, factors: Vec<JacFactor>) -> Self {
        Self {
            scalar,
            q_shift,
            factors,
        }
    }

    /// Convenience constructor for a single JAC(a, b)^1 with scalar=1, q_shift=0.
    pub fn single(a: i64, b: i64) -> Self {
        Self {
            scalar: QRat::one(),
            q_shift: QRat::zero(),
            factors: vec![JacFactor::new(a, b, 1)],
        }
    }

    /// True if there are no JAC factors.
    pub fn is_empty(&self) -> bool {
        self.factors.is_empty()
    }

    /// Expand this JacExpression to a formal power series.
    ///
    /// Each JAC(a, b)^e factor is expanded via `jacprod(a, b, variable, truncation_order)`
    /// and raised to the power e. The results are multiplied together and scaled by the
    /// scalar prefactor.
    ///
    /// # Panics
    ///
    /// Panics if `q_shift` is a non-integer fraction (FPS only supports integer exponents).
    pub fn to_series(&self, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
        let mut result = FormalPowerSeries::one(variable, truncation_order);

        for factor in &self.factors {
            let jac_fps = jacprod(factor.a, factor.b, variable, truncation_order);
            let powered = fps_pow(&jac_fps, factor.exponent);
            result = arithmetic::mul(&result, &powered);
        }

        // Apply scalar
        if self.scalar != QRat::one() {
            result = arithmetic::scalar_mul(&self.scalar, &result);
        }

        // Apply q_shift
        if !self.q_shift.is_zero() {
            // Check that q_shift is an integer
            let denom = self.q_shift.denom();
            assert!(
                *denom == rug::Integer::from(1) || *denom == rug::Integer::from(-1),
                "JacExpression::to_series: q_shift {} is fractional; FPS only supports integer exponents",
                self.q_shift
            );
            let shift = self.q_shift.0.to_f64() as i64;
            let monomial = FormalPowerSeries::monomial(variable, QRat::one(), shift, truncation_order);
            result = arithmetic::mul(&monomial, &result);
        }

        result
    }
}
