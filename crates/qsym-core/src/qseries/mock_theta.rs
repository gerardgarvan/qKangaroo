//! Classical mock theta functions of Ramanujan (orders 3, 5, and 7).
//!
//! This module implements all 20 classical mock theta functions as explicit q-series
//! with term-by-term accumulation and incremental denominator products.
//!
//! **Third-order (7 functions):** f, phi, psi, chi, omega, nu, rho
//! **Fifth-order (10 functions):** f0, f1, F0, F1, phi0, phi1, psi0, psi1, chi0, chi1
//! **Seventh-order (3 functions):** F0, F1, F2
//!
//! Each function takes a `SymbolId` (the series variable, typically "q") and a
//! `truncation_order`, and returns a `FormalPowerSeries`.
//!
//! # References
//!
//! - OEIS A000025 (f3), A053250 (phi3), A053251 (psi3), A053252 (chi3),
//!   A053253 (omega3), A053254 (nu3), A053255 (rho3)
//! - OEIS A053256 (f0_5), A053257 (f1_5), A053258 (F0_5), A053259 (F1_5),
//!   A053260 (phi0_5)

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::symbol::SymbolId;
use super::{QMonomial, PochhammerOrder, aqprod};

// ===========================================================================
// Private helper functions
// ===========================================================================

/// Returns `1 + q^m` as FPS. If m >= truncation_order, returns FPS::one.
fn make_factor_1_plus_q_m(m: i64, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut f = FormalPowerSeries::one(variable, truncation_order);
    if m > 0 && m < truncation_order {
        f.set_coeff(m, QRat::one());
    } else if m == 0 {
        // 1 + q^0 = 2
        f.set_coeff(0, QRat::from((2, 1)));
    }
    f
}

/// Returns `1 - q^m` as FPS. If m >= truncation_order, returns FPS::one.
fn make_factor_1_minus_q_m(m: i64, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut f = FormalPowerSeries::one(variable, truncation_order);
    if m > 0 && m < truncation_order {
        f.set_coeff(m, -QRat::one());
    } else if m == 0 {
        // 1 - q^0 = 0
        return FormalPowerSeries::zero(variable, truncation_order);
    }
    f
}

/// Returns `1 - q^m + q^{2m}` as FPS (for chi3 and related functions).
fn make_factor_cyclotomic3(m: i64, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut f = FormalPowerSeries::one(variable, truncation_order);
    if m > 0 && m < truncation_order {
        f.set_coeff(m, -QRat::one());
    }
    if m > 0 && 2 * m < truncation_order {
        f.set_coeff(2 * m, QRat::one());
    }
    f
}

/// Returns `1 + q^m + q^{2m}` as FPS (for rho3 factors).
fn make_factor_1_plus_q_m_plus_q2m(m: i64, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut f = FormalPowerSeries::one(variable, truncation_order);
    if m > 0 && m < truncation_order {
        f.set_coeff(m, QRat::one());
    }
    if m > 0 && 2 * m < truncation_order {
        f.set_coeff(2 * m, QRat::one());
    }
    f
}

/// Negate the variable: maps coeff[k] -> coeff[k] * (-1)^k.
/// This is the formal substitution q -> -q.
fn negate_variable(fps: &FormalPowerSeries) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(fps.variable(), fps.truncation_order());
    for (&k, v) in fps.iter() {
        if k % 2 == 0 {
            result.set_coeff(k, v.clone());
        } else {
            result.set_coeff(k, -v.clone());
        }
    }
    result
}

// ===========================================================================
// Third-order mock theta functions (7 functions)
// ===========================================================================

/// Third-order mock theta function f(q).
///
/// f(q) = sum_{n>=0} q^{n^2} / (-q;q)_n^2
///
/// OEIS A000025: 1, 1, -2, 3, -3, 3, -5, 7, -6, 6, -10, 12, -11, ...
pub fn mock_theta_f3(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // (-q;q)_0^2 = 1
    let mut denom_sq = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * n;
        if q_exp >= truncation_order {
            break;
        }

        // term_n = q^{n^2} / denom_sq = q^{n^2} * invert(denom_sq)
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom_sq);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);

        // Update: denom_sq *= (1+q^{n+1})^2
        let factor = make_factor_1_plus_q_m(n + 1, variable, truncation_order);
        denom_sq = arithmetic::mul(&denom_sq, &factor);
        denom_sq = arithmetic::mul(&denom_sq, &factor);
    }
    result
}

/// Third-order mock theta function phi(q).
///
/// phi(q) = sum_{n>=0} q^{n^2} / (-q^2;q^2)_n
///
/// OEIS A053250
pub fn mock_theta_phi3(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // (-q^2;q^2)_0 = 1
    let mut denom = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * n;
        if q_exp >= truncation_order {
            break;
        }

        // term = q^{n^2} * invert(denom)
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);

        // Update: denom *= (1 + q^{2(n+1)})
        // (-q^2;q^2)_{n+1} = (-q^2;q^2)_n * (1 + q^{2(n+1)})
        let factor = make_factor_1_plus_q_m(2 * (n + 1), variable, truncation_order);
        denom = arithmetic::mul(&denom, &factor);
    }
    result
}

/// Third-order mock theta function psi(q).
///
/// psi(q) = sum_{n>=1} q^{n^2} / (q;q^2)_n
///
/// OEIS A053251. Note: starts at n=1, so constant term is 0.
pub fn mock_theta_psi3(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // (q;q^2)_0 = 1
    let mut denom = FormalPowerSeries::one(variable, truncation_order);

    for n in 1i64.. {
        let q_exp = n * n;
        if q_exp >= truncation_order {
            break;
        }

        // Update denom: denom *= (1 - q^{2n-1})
        // (q;q^2)_n = (1-q)(1-q^3)...(1-q^{2n-1})
        let factor = make_factor_1_minus_q_m(2 * n - 1, variable, truncation_order);
        denom = arithmetic::mul(&denom, &factor);

        // term = q^{n^2} * invert(denom)
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);
    }
    result
}

/// Third-order mock theta function chi(q).
///
/// chi(q) = sum_{n>=0} q^{n^2} / prod_{k=1}^{n} (1 - q^k + q^{2k})
///
/// OEIS A053252
pub fn mock_theta_chi3(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // For n=0: empty product = 1
    let mut denom = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * n;
        if q_exp >= truncation_order {
            break;
        }

        // For n > 0, update denom first: denom *= (1 - q^n + q^{2n})
        if n > 0 {
            let factor = make_factor_cyclotomic3(n, variable, truncation_order);
            denom = arithmetic::mul(&denom, &factor);
        }

        // term = q^{n^2} * invert(denom)
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);
    }
    result
}

/// Third-order mock theta function omega(q).
///
/// omega(q) = sum_{n>=0} q^{2n(n+1)} / (q;q^2)_{n+1}^2
///
/// OEIS A053253
pub fn mock_theta_omega3(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // (q;q^2)_0 = 1; we start with (q;q^2)_1 = (1-q)
    let mut denom = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = 2 * n * (n + 1);
        if q_exp >= truncation_order {
            break;
        }

        // Update denom: denom *= (1 - q^{2n+1})
        // (q;q^2)_{n+1} = (1-q)(1-q^3)...(1-q^{2n+1})
        let factor = make_factor_1_minus_q_m(2 * n + 1, variable, truncation_order);
        denom = arithmetic::mul(&denom, &factor);

        // denom_sq = denom^2
        let denom_sq = arithmetic::mul(&denom, &denom);

        // term = q^{2n(n+1)} * invert(denom_sq)
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom_sq);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);
    }
    result
}

/// Third-order mock theta function nu(q).
///
/// nu(q) = sum_{n>=0} q^{n(n+1)} / (-q;q^2)_{n+1}
///
/// OEIS A053254. Factors are (1+q)(1+q^3)(1+q^5)...
pub fn mock_theta_nu3(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // (-q;q^2)_0 = 1
    let mut denom = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * (n + 1);
        if q_exp >= truncation_order {
            break;
        }

        // Update denom: denom *= (1 + q^{2n+1})
        // (-q;q^2)_{n+1} = (1+q)(1+q^3)...(1+q^{2n+1})
        let factor = make_factor_1_plus_q_m(2 * n + 1, variable, truncation_order);
        denom = arithmetic::mul(&denom, &factor);

        // term = q^{n(n+1)} * invert(denom)
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);
    }
    result
}

/// Third-order mock theta function rho(q).
///
/// rho(q) = sum_{n>=0} q^{2n(n+1)} / prod_{k=0}^{n} (1 + q^{2k+1} + q^{4k+2})
///
/// OEIS A053255
pub fn mock_theta_rho3(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // Empty product for k from 0 to -1 (n=-1 would be empty, but loop starts at n=0)
    let mut denom = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = 2 * n * (n + 1);
        if q_exp >= truncation_order {
            break;
        }

        // Update denom: denom *= (1 + q^{2n+1} + q^{4n+2})
        let factor = make_factor_1_plus_q_m_plus_q2m(2 * n + 1, variable, truncation_order);
        denom = arithmetic::mul(&denom, &factor);

        // term = q^{2n(n+1)} * invert(denom)
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);
    }
    result
}

// ===========================================================================
// Fifth-order mock theta functions (10 functions)
// ===========================================================================

/// Fifth-order mock theta function f0(q).
///
/// f0(q) = sum_{n>=0} q^{n^2} / (-q;q)_n
///
/// OEIS A053256
pub fn mock_theta_f0_5(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    let mut denom = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * n;
        if q_exp >= truncation_order {
            break;
        }

        // term = q^{n^2} * invert(denom)
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);

        // Update: denom *= (1 + q^{n+1})
        let factor = make_factor_1_plus_q_m(n + 1, variable, truncation_order);
        denom = arithmetic::mul(&denom, &factor);
    }
    result
}

/// Fifth-order mock theta function f1(q).
///
/// f1(q) = sum_{n>=0} q^{n^2+n} / (-q;q)_n
///
/// OEIS A053257
pub fn mock_theta_f1_5(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    let mut denom = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * n + n;
        if q_exp >= truncation_order {
            break;
        }

        // term = q^{n^2+n} * invert(denom)
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);

        // Update: denom *= (1 + q^{n+1})
        let factor = make_factor_1_plus_q_m(n + 1, variable, truncation_order);
        denom = arithmetic::mul(&denom, &factor);
    }
    result
}

/// Fifth-order mock theta function F0(q) (capital F).
///
/// F0(q) = sum_{n>=0} q^{2n^2} / (q;q^2)_n
///
/// OEIS A053258. Note: (q;q^2)_0 = 1 (empty product).
pub fn mock_theta_cap_f0_5(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    let mut denom = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = 2 * n * n;
        if q_exp >= truncation_order {
            break;
        }

        // term = q^{2n^2} * invert(denom)
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);

        // Update: denom *= (1 - q^{2n+1})
        // (q;q^2)_{n+1} = (q;q^2)_n * (1 - q^{2n+1})
        // because (q;q^2)_n = (1-q)(1-q^3)...(1-q^{2n-1})
        // next factor is (1 - q^{2(n+1)-1}) = (1 - q^{2n+1})
        let factor = make_factor_1_minus_q_m(2 * n + 1, variable, truncation_order);
        denom = arithmetic::mul(&denom, &factor);
    }
    result
}

/// Fifth-order mock theta function F1(q) (capital F).
///
/// F1(q) = sum_{n>=0} q^{2n^2+2n} / (q;q^2)_{n+1}
///
/// OEIS A053259. Denominator (q;q^2)_{n+1} = (1-q)(1-q^3)...(1-q^{2n+1}).
pub fn mock_theta_cap_f1_5(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    let mut denom = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = 2 * n * n + 2 * n;
        if q_exp >= truncation_order {
            break;
        }

        // Update denom: denom *= (1 - q^{2n+1})
        // (q;q^2)_{n+1} = (1-q)(1-q^3)...(1-q^{2n+1})
        let factor = make_factor_1_minus_q_m(2 * n + 1, variable, truncation_order);
        denom = arithmetic::mul(&denom, &factor);

        // term = q^{2n^2+2n} * invert(denom)
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);
    }
    result
}

/// Fifth-order mock theta function phi0(q).
///
/// phi0(q) = sum_{n>=0} (-q;q^2)_n * q^{n^2}
///
/// OEIS A053260. Note: numerator product, not denominator.
/// (-q;q^2)_n = (1+q)(1+q^3)...(1+q^{2n-1})
pub fn mock_theta_phi0_5(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // (-q;q^2)_0 = 1 (empty product)
    let mut numer_prod = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * n;
        if q_exp >= truncation_order {
            break;
        }

        // For n >= 1, update numerator product: numer_prod *= (1 + q^{2n-1})
        if n >= 1 {
            let factor = make_factor_1_plus_q_m(2 * n - 1, variable, truncation_order);
            numer_prod = arithmetic::mul(&numer_prod, &factor);
        }

        // term = numer_prod * q^{n^2}
        let q_mono = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let term = arithmetic::mul(&numer_prod, &q_mono);
        result = arithmetic::add(&result, &term);
    }
    result
}

/// Fifth-order mock theta function phi1(q).
///
/// phi1(q) = sum_{n>=0} (-q;q^2)_n * q^{(n+1)^2}
///
/// Same numerator product as phi0, but power is (n+1)^2.
pub fn mock_theta_phi1_5(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    let mut numer_prod = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = (n + 1) * (n + 1);
        if q_exp >= truncation_order {
            break;
        }

        // For n >= 1, update numerator product: numer_prod *= (1 + q^{2n-1})
        if n >= 1 {
            let factor = make_factor_1_plus_q_m(2 * n - 1, variable, truncation_order);
            numer_prod = arithmetic::mul(&numer_prod, &factor);
        }

        // term = numer_prod * q^{(n+1)^2}
        let q_mono = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let term = arithmetic::mul(&numer_prod, &q_mono);
        result = arithmetic::add(&result, &term);
    }
    result
}

/// Fifth-order mock theta function psi0(q).
///
/// psi0(q) = sum_{n>=0} (-1;q)_n * q^{n(n+1)/2}
///
/// (-1;q)_n = prod_{k=0}^{n-1} (1 - (-1)*q^k) = prod_{k=0}^{n-1} (1 + q^k)
/// For n=0: empty product = 1.
/// For n=1: (1+q^0) = 2.
/// For n=2: (1+1)(1+q) = 2(1+q).
pub fn mock_theta_psi0_5(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // (-1;q)_0 = 1
    let mut numer_prod = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * (n + 1) / 2;
        if q_exp >= truncation_order {
            break;
        }
        // n(n+1)/2 is always an integer

        // For n >= 1, update: numer_prod *= (1 + q^{n-1})
        // (-1;q)_n = prod_{k=0}^{n-1}(1+q^k), so going from n-1 to n adds factor (1+q^{n-1})
        if n >= 1 {
            let factor = make_factor_1_plus_q_m(n - 1, variable, truncation_order);
            numer_prod = arithmetic::mul(&numer_prod, &factor);
        }

        // term = numer_prod * q^{n(n+1)/2}
        let q_mono = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let term = arithmetic::mul(&numer_prod, &q_mono);
        result = arithmetic::add(&result, &term);
    }
    result
}

/// Fifth-order mock theta function psi1(q).
///
/// psi1(q) = sum_{n>=0} (-q;q)_n * q^{n(n+1)/2}
///
/// (-q;q)_n = (1+q)(1+q^2)...(1+q^n). For n=0: empty product = 1.
pub fn mock_theta_psi1_5(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // (-q;q)_0 = 1
    let mut numer_prod = FormalPowerSeries::one(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * (n + 1) / 2;
        if q_exp >= truncation_order {
            break;
        }

        // For n >= 1, update: numer_prod *= (1 + q^n)
        // (-q;q)_n = (1+q)(1+q^2)...(1+q^n), going from n-1 to n adds factor (1+q^n)
        if n >= 1 {
            let factor = make_factor_1_plus_q_m(n, variable, truncation_order);
            numer_prod = arithmetic::mul(&numer_prod, &factor);
        }

        // term = numer_prod * q^{n(n+1)/2}
        let q_mono = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let term = arithmetic::mul(&numer_prod, &q_mono);
        result = arithmetic::add(&result, &term);
    }
    result
}

/// Fifth-order mock theta function chi0(q).
///
/// chi0(q) = 2*F0(q) - phi0(-q)
///
/// where phi0(-q) means phi0 evaluated with q -> -q.
pub fn mock_theta_chi0_5(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let f0 = mock_theta_cap_f0_5(variable, truncation_order);
    let phi0 = mock_theta_phi0_5(variable, truncation_order);
    let phi0_negq = negate_variable(&phi0);

    // chi0 = 2*F0 - phi0(-q)
    let two_f0 = arithmetic::scalar_mul(&QRat::from((2, 1)), &f0);
    arithmetic::sub(&two_f0, &phi0_negq)
}

/// Fifth-order mock theta function chi1(q).
///
/// chi1(q) = 2*F1(q) + q^{-1}*phi1(-q)
///
/// The q^{-1} factor means shifting phi1(-q) down by 1 power.
/// phi1(-q) always starts at q^1 (since phi1 starts at q^1), so shift
/// by -1 produces non-negative powers.
pub fn mock_theta_chi1_5(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let f1 = mock_theta_cap_f1_5(variable, truncation_order);
    let phi1 = mock_theta_phi1_5(variable, truncation_order);
    let phi1_negq = negate_variable(&phi1);

    // q^{-1} * phi1(-q): shift by -1
    let phi1_negq_shifted = arithmetic::shift(&phi1_negq, -1);

    // chi1 = 2*F1 + q^{-1}*phi1(-q)
    let two_f1 = arithmetic::scalar_mul(&QRat::from((2, 1)), &f1);
    arithmetic::add(&two_f1, &phi1_negq_shifted)
}

// ===========================================================================
// Seventh-order mock theta functions (3 functions)
// ===========================================================================

/// Seventh-order mock theta function F0(q).
///
/// F0(q) = sum_{n>=0} q^{n^2} / (q^{n+1};q)_n
///
/// Denominator (q^{n+1};q)_n = (1-q^{n+1})(1-q^{n+2})...(1-q^{2n}).
/// For n=0: empty product = 1.
///
/// NOTE: Denominator base shifts with n, so incremental products cannot be used.
/// Uses aqprod directly for each n. O(N^2) total (N ~ sqrt(truncation_order)).
pub fn mock_theta_cap_f0_7(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * n;
        if q_exp >= truncation_order {
            break;
        }

        // (q^{n+1}; q)_n
        let denom = aqprod(
            &QMonomial::q_power(n + 1),
            variable,
            PochhammerOrder::Finite(n),
            truncation_order,
        );

        if n == 0 {
            // denom = 1, term = q^0 = 1
            let term = FormalPowerSeries::monomial(variable, QRat::one(), 0, truncation_order);
            result = arithmetic::add(&result, &term);
        } else {
            let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
            let denom_inv = arithmetic::invert(&denom);
            let term = arithmetic::mul(&numer, &denom_inv);
            result = arithmetic::add(&result, &term);
        }
    }
    result
}

/// Seventh-order mock theta function F1(q).
///
/// F1(q) = sum_{n>=0} q^{n^2} / (q^n;q)_n
///
/// Denominator (q^n;q)_n = (1-q^n)(1-q^{n+1})...(1-q^{2n-1}).
/// For n=0: empty product = 1. For n=1: (1-q).
pub fn mock_theta_cap_f1_7(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * n;
        if q_exp >= truncation_order {
            break;
        }

        // (q^n; q)_n
        let denom = aqprod(
            &QMonomial::q_power(n),
            variable,
            PochhammerOrder::Finite(n),
            truncation_order,
        );

        if n == 0 {
            // denom = 1 (order 0), term = 1
            let term = FormalPowerSeries::monomial(variable, QRat::one(), 0, truncation_order);
            result = arithmetic::add(&result, &term);
        } else {
            let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
            let denom_inv = arithmetic::invert(&denom);
            let term = arithmetic::mul(&numer, &denom_inv);
            result = arithmetic::add(&result, &term);
        }
    }
    result
}

/// Seventh-order mock theta function F2(q).
///
/// F2(q) = sum_{n>=0} q^{n^2+n} / (q^{n+1};q)_{n+1}
///
/// Denominator (q^{n+1};q)_{n+1} = (1-q^{n+1})(1-q^{n+2})...(1-q^{2n+1}).
pub fn mock_theta_cap_f2_7(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);

    for n in 0i64.. {
        let q_exp = n * n + n;
        if q_exp >= truncation_order {
            break;
        }

        // (q^{n+1}; q)_{n+1}
        let denom = aqprod(
            &QMonomial::q_power(n + 1),
            variable,
            PochhammerOrder::Finite(n + 1),
            truncation_order,
        );

        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);
    }
    result
}
