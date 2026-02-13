//! Rank and crank generating functions for partition theory.
//!
//! - [`crank_gf`]: C(z,q) = (q;q)_inf / [(zq;q)_inf * (q/z;q)_inf]
//! - [`rank_gf`]: R(z,q) = 1 + sum_{n>=1} q^{n^2} / [(zq;q)_n * (q/z;q)_n]
//!
//! Both functions reduce to the partition generating function 1/(q;q)_inf at z=1.

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::series::generator::{euler_function_generator, qpochhammer_inf_generator};
use crate::symbol::SymbolId;

use super::partitions::partition_gf;

/// Compute the crank generating function:
///   C(z, q) = (q;q)_inf / [(zq;q)_inf * (q/z;q)_inf]
///
/// At z=1, this has a removable singularity and equals the partition
/// generating function 1/(q;q)_inf. This case is handled specially.
///
/// # Arguments
///
/// - `z`: The crank parameter (a pure rational number, not a q-monomial).
/// - `variable`: The SymbolId for the series variable.
/// - `truncation_order`: Compute to O(q^truncation_order).
pub fn crank_gf(z: &QRat, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    // Special case: z=1 has removable singularity, return partition_gf
    if *z == QRat::one() {
        return partition_gf(variable, truncation_order);
    }

    // numerator: (q;q)_inf
    let mut euler_ipg = euler_function_generator(variable, truncation_order);
    euler_ipg.ensure_order(truncation_order);
    let numerator = euler_ipg.into_series();

    // denom1: (zq;q)_inf = prod_{k>=0}(1 - z*q^{1+k})
    let mut denom1_ipg = qpochhammer_inf_generator(z.clone(), 1, variable, truncation_order);
    denom1_ipg.ensure_order(truncation_order);
    let denom1 = denom1_ipg.into_series();

    // denom2: (q/z;q)_inf = prod_{k>=0}(1 - (1/z)*q^{1+k})
    let inv_z = QRat::one() / z.clone();
    let mut denom2_ipg = qpochhammer_inf_generator(inv_z, 1, variable, truncation_order);
    denom2_ipg.ensure_order(truncation_order);
    let denom2 = denom2_ipg.into_series();

    // C(z,q) = numerator / (denom1 * denom2)
    let denom_product = arithmetic::mul(&denom1, &denom2);
    let inv_denom = arithmetic::invert(&denom_product);
    arithmetic::mul(&numerator, &inv_denom)
}

/// Compute the rank generating function:
///   R(z, q) = 1 + sum_{n>=1} q^{n^2} / [(zq;q)_n * (q/z;q)_n]
///
/// At z=1, this has a removable singularity and equals the partition
/// generating function 1/(q;q)_inf. This case is handled specially.
///
/// # Arguments
///
/// - `z`: The rank parameter (a pure rational number).
/// - `variable`: The SymbolId for the series variable.
/// - `truncation_order`: Compute to O(q^truncation_order).
pub fn rank_gf(z: &QRat, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    // Special case: z=1 has removable singularity
    if *z == QRat::one() {
        return partition_gf(variable, truncation_order);
    }

    let inv_z = QRat::one() / z.clone();

    // Start with 1 (the n=0 term)
    let mut result = FormalPowerSeries::one(variable, truncation_order);

    let mut n: i64 = 1;
    while n * n < truncation_order {
        // numerator: q^{n^2}
        let q_n_sq = FormalPowerSeries::monomial(variable, QRat::one(), n * n, truncation_order);

        // (zq;q)_n: finite product prod_{k=0}^{n-1}(1 - z*q^{1+k})
        let mut zq_n = FormalPowerSeries::one(variable, truncation_order);
        for k in 0..n {
            let mut factor = FormalPowerSeries::one(variable, truncation_order);
            factor.set_coeff(k + 1, -z.clone());
            zq_n = arithmetic::mul(&zq_n, &factor);
        }

        // (q/z;q)_n: finite product prod_{k=0}^{n-1}(1 - (1/z)*q^{1+k})
        let mut qz_n = FormalPowerSeries::one(variable, truncation_order);
        for k in 0..n {
            let mut factor = FormalPowerSeries::one(variable, truncation_order);
            factor.set_coeff(k + 1, -inv_z.clone());
            qz_n = arithmetic::mul(&qz_n, &factor);
        }

        // denominator = (zq;q)_n * (q/z;q)_n
        let denom = arithmetic::mul(&zq_n, &qz_n);
        let inv_denom = arithmetic::invert(&denom);

        // term = q^{n^2} / denominator
        let term = arithmetic::mul(&q_n_sq, &inv_denom);

        result = arithmetic::add(&result, &term);

        n += 1;
    }

    result
}
