//! ETA symbolic representation: eta quotients as structured data.
//!
//! An eta quotient is: prod_{delta | N} eta(delta * tau)^{r_delta}
//! where eta(tau) = q^{1/24} * (q;q)_inf is the Dedekind eta function.
//!
//! The `EtaExpression` struct captures the structure (delta -> r_delta mapping)
//! and provides methods for computing weight, q-shift, and validating
//! Newman's modularity conditions on Gamma_0(N).

use std::collections::BTreeMap;

use rug::ops::Pow;

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::symbol::SymbolId;
use crate::qseries::products::etaq;
use crate::qseries::prodmake::EtaQuotient;

use super::fps_pow;

/// Result of checking Newman's modularity conditions for an eta quotient.
#[derive(Clone, Debug)]
pub enum ModularityResult {
    /// All Newman conditions satisfied for modular function on Gamma_0(N).
    Modular,
    /// One or more conditions failed.
    NotModular { failed_conditions: Vec<String> },
}

impl ModularityResult {
    /// Returns true if the eta quotient is modular.
    pub fn is_modular(&self) -> bool {
        matches!(self, ModularityResult::Modular)
    }
}

/// A symbolic eta quotient: prod_{delta | N} eta(delta * tau)^{r_delta}.
///
/// Captures the structure of an eta quotient without expanding to a formal
/// power series. Provides methods for structural analysis (weight, q-shift,
/// modularity checks) and conversion to FPS.
#[derive(Clone, Debug)]
pub struct EtaExpression {
    /// Maps delta -> r_delta (only nonzero entries).
    pub factors: BTreeMap<i64, i64>,
    /// The level N (all deltas must divide N).
    pub level: i64,
}

impl EtaExpression {
    /// Create a new EtaExpression. Validates that every delta divides the level.
    pub fn new(factors: BTreeMap<i64, i64>, level: i64) -> Self {
        for &delta in factors.keys() {
            assert!(
                level % delta == 0,
                "EtaExpression::new: delta {} does not divide level {}",
                delta,
                level
            );
        }
        Self { factors, level }
    }

    /// Convenience constructor from a slice of (delta, r_delta) pairs.
    pub fn from_factors(pairs: &[(i64, i64)], level: i64) -> Self {
        let mut factors = BTreeMap::new();
        for &(delta, r_delta) in pairs {
            if r_delta != 0 {
                factors.insert(delta, r_delta);
            }
        }
        Self::new(factors, level)
    }

    /// Convert from the existing `prodmake::EtaQuotient` struct.
    ///
    /// Computes the level as the LCM of all deltas.
    pub fn from_etaquotient(eq: &EtaQuotient) -> Self {
        if eq.factors.is_empty() {
            return Self {
                factors: BTreeMap::new(),
                level: 1,
            };
        }
        let mut level = 1i64;
        for &delta in eq.factors.keys() {
            level = lcm(level, delta);
        }
        Self {
            factors: eq.factors.clone(),
            level,
        }
    }

    /// Compute the weight: sum(r_delta) / 2.
    pub fn weight(&self) -> QRat {
        let sum: i64 = self.factors.values().sum();
        QRat::from((sum, 2i64))
    }

    /// Compute the q-shift: sum(delta * r_delta) / 24.
    pub fn q_shift(&self) -> QRat {
        let sum: i64 = self.factors.iter().map(|(&d, &r)| d * r).sum();
        QRat::from((sum, 24i64))
    }

    /// Check Newman's four conditions for modularity on Gamma_0(N).
    ///
    /// The conditions are:
    /// 0. Every delta divides N (enforced by constructor).
    /// 1. sum(delta * r_delta) is divisible by 24.
    /// 2. sum((N/delta) * r_delta) is divisible by 24.
    /// 3. prod(delta^|r_delta|) is a perfect square.
    /// 4. sum(r_delta) == 0 (weight zero for modular functions).
    pub fn check_modularity(&self) -> ModularityResult {
        let mut errors = Vec::new();

        // Condition 0: divisibility (re-check)
        for &delta in self.factors.keys() {
            if self.level % delta != 0 {
                errors.push(format!(
                    "delta {} does not divide level {}",
                    delta, self.level
                ));
            }
        }

        // Condition 1: sum(delta * r_delta) divisible by 24
        let sum1: i64 = self.factors.iter().map(|(&d, &r)| d * r).sum();
        if sum1 % 24 != 0 {
            errors.push(format!(
                "sum(delta * r_delta) = {} is not divisible by 24",
                sum1
            ));
        }

        // Condition 2: sum((N/delta) * r_delta) divisible by 24
        let sum2: i64 = self
            .factors
            .iter()
            .map(|(&d, &r)| (self.level / d) * r)
            .sum();
        if sum2 % 24 != 0 {
            errors.push(format!(
                "sum((N/delta) * r_delta) = {} is not divisible by 24",
                sum2
            ));
        }

        // Condition 3: prod(delta^|r_delta|) is a perfect square
        let mut product = rug::Integer::from(1);
        for (&delta, &r) in &self.factors {
            let r_abs = r.unsigned_abs() as u32;
            let delta_int = rug::Integer::from(delta);
            product *= delta_int.pow(r_abs);
        }
        let sqrt = product.clone().sqrt();
        let sqrt_sq = rug::Integer::from(&sqrt * &sqrt);
        if sqrt_sq != product {
            errors.push("prod(delta^|r_delta|) is not a perfect square".to_string());
        }

        // Condition 4: weight zero (sum(r_delta) == 0)
        let sum_r: i64 = self.factors.values().sum();
        if sum_r != 0 {
            errors.push(format!(
                "sum(r_delta) = {} (weight {} is not zero)",
                sum_r,
                QRat::from((sum_r, 2i64))
            ));
        }

        if errors.is_empty() {
            ModularityResult::Modular
        } else {
            ModularityResult::NotModular {
                failed_conditions: errors,
            }
        }
    }

    /// Expand this eta quotient to a formal power series.
    ///
    /// Algorithm:
    /// 1. Compute total q-shift = sum(delta * r_delta) / 24.
    /// 2. Verify the q-shift is an integer.
    /// 3. For each (delta, r_delta): compute etaq(delta, delta, var, trunc)^{r_delta}.
    /// 4. Multiply by q^{total_q_shift}.
    ///
    /// # Panics
    ///
    /// Panics if the total q-shift is not an integer.
    pub fn to_series(&self, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
        let total_q_shift = self.q_shift();

        // Verify q-shift is an integer
        let denom = total_q_shift.denom();
        assert!(
            *denom == rug::Integer::from(1) || *denom == rug::Integer::from(-1),
            "EtaExpression::to_series: q_shift {} is not an integer; \
             this eta quotient does not have integer q-powers",
            total_q_shift
        );
        let shift_i64 = total_q_shift.0.to_f64() as i64;

        // Build the product of (q^delta; q^delta)_inf^{r_delta}
        let mut result = FormalPowerSeries::one(variable, truncation_order);
        for (&delta, &r_delta) in &self.factors {
            let eta_delta = etaq(delta, delta, variable, truncation_order);
            let powered = fps_pow(&eta_delta, r_delta);
            result = arithmetic::mul(&result, &powered);
        }

        // Multiply by q^{shift}
        if shift_i64 != 0 {
            let monomial =
                FormalPowerSeries::monomial(variable, QRat::one(), shift_i64, truncation_order);
            result = arithmetic::mul(&monomial, &result);
        }

        result
    }
}

/// Greatest common divisor of two integers.
fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Least common multiple of two integers.
fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        return 0;
    }
    (a / gcd(a, b)) * b
}
