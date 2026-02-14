//! Q-series utility functions: subsequence extraction and degree bounds.
//!
//! - [`sift`]: extract arithmetic subsequence f(m*i+j) from a series
//! - [`qdegree`]: highest nonzero exponent
//! - [`lqdegree`]: lowest nonzero exponent

use crate::series::FormalPowerSeries;

/// Extract the arithmetic subsequence of a formal power series.
///
/// `sift(f, m, j)` produces a new series g where:
///   g(q^i) = f(q^{m*i + j})
///
/// That is, the coefficient of q^i in the output equals the coefficient
/// of q^{m*i + j} in the input.
///
/// This is the core operation for studying partition congruences.
/// For example, `sift(partition_gf, 5, 4)` extracts the subsequence
/// p(5n+4), which Ramanujan proved is always divisible by 5.
///
/// # Arguments
///
/// - `f`: the input series
/// - `m`: the modulus (step size), must be > 0
/// - `j`: the residue class (offset); reduced mod m to [0, m)
///
/// # Panics
///
/// Panics if m <= 0.
pub fn sift(f: &FormalPowerSeries, m: i64, j: i64) -> FormalPowerSeries {
    assert!(m > 0, "sift modulus must be positive, got {}", m);

    // Normalize j into [0, m)
    let j_norm = ((j % m) + m) % m;

    // Compute new truncation order:
    // We need m*i + j_norm < f.truncation_order()
    // i.e., i < (f.truncation_order() - j_norm) / m
    let f_trunc = f.truncation_order();
    let new_trunc = if j_norm < f_trunc {
        (f_trunc - j_norm - 1) / m + 1
    } else {
        0
    };

    let variable = f.variable();
    let mut result = FormalPowerSeries::zero(variable, new_trunc);

    let mut i: i64 = 0;
    loop {
        let src_exp = m * i + j_norm;
        if src_exp >= f_trunc || i >= new_trunc {
            break;
        }
        let c = f.coeff(src_exp);
        if !c.is_zero() {
            result.set_coeff(i, c);
        }
        i += 1;
    }

    result
}

/// Return the highest exponent with a nonzero coefficient.
///
/// For a polynomial, this is the degree. For a truncated series,
/// this is the largest exponent with a stored nonzero term.
///
/// Returns `None` for the zero series.
pub fn qdegree(f: &FormalPowerSeries) -> Option<i64> {
    f.iter().last().map(|(&k, _)| k)
}

/// Return the lowest exponent with a nonzero coefficient.
///
/// This is the order (valuation) of the series -- the smallest k
/// such that [q^k]f != 0.
///
/// Returns `None` for the zero series.
///
/// Wraps `FormalPowerSeries::min_order()` for API consistency with `qdegree`.
pub fn lqdegree(f: &FormalPowerSeries) -> Option<i64> {
    f.min_order()
}
