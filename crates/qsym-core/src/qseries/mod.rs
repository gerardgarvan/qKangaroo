//! Q-series types and functions: q-Pochhammer symbols, q-binomial coefficients,
//! named products, theta functions, partition functions, and rank/crank.
//!
//! This module provides the foundational building blocks for all q-series computations:
//! - [`QMonomial`]: represents `c * q^m` used as the `a` parameter in q-Pochhammer symbols
//! - [`PochhammerOrder`]: finite or infinite order for q-Pochhammer products
//! - [`aqprod`]: general q-Pochhammer symbol (a;q)_n
//! - [`qbin`]: q-binomial (Gaussian) coefficient [n choose k]_q
//! - Named products: [`etaq`], [`jacprod`], [`tripleprod`], [`quinprod`], [`winquist`]
//! - Theta functions: [`theta2`], [`theta3`], [`theta4`]
//! - Partition functions: [`partition_count`], [`partition_gf`], [`distinct_parts_gf`],
//!   [`odd_parts_gf`], [`bounded_parts_gf`]
//! - Rank/crank: [`rank_gf`], [`crank_gf`]
//! - Series analysis: [`prodmake`] (Andrews' algorithm for series-to-product conversion),
//!   [`etamake`], [`jacprodmake`], [`mprodmake`], [`qetamake`] (post-processing)
//! - Factoring: [`qfactor`], [`QFactorization`] -- decompose polynomials into (1-q^i) factors
//! - Utilities: [`sift`], [`qdegree`], [`lqdegree`] -- subsequence extraction and degree bounds
//! - Linear algebra: [`rational_null_space`], [`build_coefficient_matrix`], [`modular_null_space`]
//! - Relation discovery: [`findlincombo`], [`findhom`], [`findpoly`], [`PolynomialRelation`],
//!   [`findcong`], [`findnonhom`], [`findhomcombo`], [`findnonhomcombo`], [`Congruence`],
//!   [`findlincombomodp`], [`findhommodp`], [`findhomcombomodp`], [`findmaxind`], [`findprod`]
//! - Identity proving: [`identity`] module for JAC/ETA symbolic models, cusps, and proving engine
//! - Mock theta functions: [`mock_theta`] module for all 20 classical mock theta functions
//!   (7 third-order, 10 fifth-order, 3 seventh-order)
//! - Appell-Lerch sums: [`appell_lerch_m`], [`universal_mock_theta_g2`], [`universal_mock_theta_g3`],
//!   [`ZwegersCompletion`]
//! - q-Gosper algorithm: [`q_gosper`], [`extract_term_ratio`], [`q_dispersion`],
//!   [`QGosperResult`], [`GosperNormalForm`], [`gosper_normal_form`], [`solve_key_equation`]
//! - q-Zeilberger algorithm: [`q_zeilberger`], [`ZeilbergerResult`], [`QZeilbergerResult`],
//!   [`detect_n_params`], [`verify_wz_certificate`], [`verify_recurrence_fps`],
//!   creative telescoping for definite q-hypergeometric summation with WZ verification
//! - q-Petkovsek algorithm: [`q_petkovsek`], [`QPetkovsekResult`], [`ClosedForm`],
//!   solving constant-coefficient q-recurrences for q-hypergeometric closed forms
//! - Nonterminating identity proofs: [`prove_nonterminating`], [`NonterminatingProofResult`],
//!   Chen-Hou-Mu parameter specialization for nonterminating q-hypergeometric identities

pub mod appell_lerch;
pub mod factoring;
pub mod linalg;
pub mod partitions;
pub mod pochhammer;
pub mod prodmake;
pub mod products;
pub mod qbinomial;
pub mod rank_crank;
pub mod relations;
pub mod theta;
pub mod utilities;
pub mod hypergeometric;
pub mod identity;
pub mod mock_theta;
pub mod bailey;
pub mod gosper;
pub mod zeilberger;
pub mod petkovsek;
pub mod nonterminating;

pub use factoring::{qfactor, QFactorization, zqfactor, ZQFactorization};
pub use hypergeometric::{HypergeometricSeries, BilateralHypergeometricSeries, eval_phi, eval_psi, SummationResult, TransformationResult, try_q_gauss, try_q_vandermonde, try_q_saalschutz, try_q_kummer, try_q_dixon, try_all_summations, heine_transform_1, heine_transform_2, heine_transform_3, sears_transform, watson_transform, bailey_4phi3_q2, TransformationStep, TransformationChainResult, find_transformation_chain};
pub use linalg::{rational_null_space, build_coefficient_matrix, modular_null_space};
pub use relations::{findlincombo, findhom, findpoly, PolynomialRelation, findcong, findcong_garvan, findnonhom, findhomcombo, findnonhomcombo, Congruence, findlincombomodp, findhommodp, findhomcombomodp, findmaxind, findprod, generate_monomials, generate_nonhom_monomials};
pub use partitions::{partition_count, partition_gf, distinct_parts_gf, odd_parts_gf, bounded_parts_gf};
pub use pochhammer::aqprod;
pub use prodmake::{prodmake, InfiniteProductForm, etamake, EtaQuotient, jacprodmake, jacprodmake_with_period_filter, JacobiProductForm, mprodmake, qetamake, QEtaForm};
pub use products::{etaq, jacprod, tripleprod, quinprod, winquist};
pub use qbinomial::qbin;
pub use rank_crank::{rank_gf, crank_gf};
pub use theta::{theta2, theta3, theta4};
pub use utilities::{sift, qdegree, lqdegree};
pub use identity::{JacFactor, JacExpression, EtaExpression, ModularityResult, Cusp, cuspmake, cuspmake1, num_cusps_gamma0, eta_order_at_cusp, cusp_width, total_order, ProofResult, EtaIdentity, prove_eta_identity, IdentityEntry, IdentityDatabase};
pub use mock_theta::{
    mock_theta_f3, mock_theta_phi3, mock_theta_psi3, mock_theta_chi3,
    mock_theta_omega3, mock_theta_nu3, mock_theta_rho3,
    mock_theta_f0_5, mock_theta_f1_5, mock_theta_cap_f0_5, mock_theta_cap_f1_5,
    mock_theta_phi0_5, mock_theta_phi1_5, mock_theta_psi0_5, mock_theta_psi1_5,
    mock_theta_chi0_5, mock_theta_chi1_5,
    mock_theta_cap_f0_7, mock_theta_cap_f1_7, mock_theta_cap_f2_7,
};
pub use appell_lerch::{appell_lerch_m, appell_lerch_bilateral, universal_mock_theta_g2, universal_mock_theta_g3, ZwegersCompletion};
pub use bailey::{BaileyPair, BaileyPairType, BaileyDatabase, bailey_lemma, bailey_chain, weak_bailey_lemma, verify_bailey_pair, bailey_discover, DiscoveryResult};
pub use gosper::{QGosperResult, GosperNormalForm, extract_term_ratio, q_dispersion, gosper_normal_form, solve_key_equation, q_gosper};
pub use zeilberger::{ZeilbergerResult, QZeilbergerResult, q_zeilberger, detect_n_params, verify_wz_certificate, verify_recurrence_fps};
pub use petkovsek::{q_petkovsek, QPetkovsekResult, ClosedForm};
pub use nonterminating::{prove_nonterminating, NonterminatingProofResult};

use crate::number::QRat;

/// A monomial of the form `coeff * q^power`, used as the `a` parameter
/// in q-Pochhammer symbols (a;q)_n.
///
/// For example:
/// - `QMonomial::q_power(1)` represents `q` (i.e., `1 * q^1`)
/// - `QMonomial::constant(c)` represents `c * q^0`
/// - `QMonomial::new(c, m)` represents `c * q^m`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QMonomial {
    /// The scalar coefficient.
    pub coeff: QRat,
    /// The power of the variable.
    pub power: i64,
}

impl QMonomial {
    /// Create a new QMonomial: `coeff * q^power`.
    pub fn new(coeff: QRat, power: i64) -> Self {
        Self { coeff, power }
    }

    /// Shorthand for `1 * q^m`.
    pub fn q_power(m: i64) -> Self {
        Self {
            coeff: QRat::one(),
            power: m,
        }
    }

    /// Shorthand for `c * q^0`.
    pub fn constant(c: QRat) -> Self {
        Self {
            coeff: c,
            power: 0,
        }
    }

    /// Multiply two QMonomials: (c1*q^p1) * (c2*q^p2) = (c1*c2)*q^{p1+p2}.
    pub fn mul(&self, other: &QMonomial) -> QMonomial {
        QMonomial::new(self.coeff.clone() * other.coeff.clone(), self.power + other.power)
    }

    /// Divide: (c1*q^p1) / (c2*q^p2) = (c1/c2)*q^{p1-p2}.
    /// Panics if other.coeff is zero.
    pub fn div(&self, other: &QMonomial) -> QMonomial {
        assert!(!other.coeff.is_zero(), "Cannot divide QMonomial by zero");
        QMonomial::new(self.coeff.clone() / other.coeff.clone(), self.power - other.power)
    }

    /// Check if this is q^{-n} for some n >= 0 (coeff=1, power <= 0).
    /// Returns Some(n) where n = -power, or None.
    pub fn is_q_neg_power(&self) -> Option<i64> {
        if self.coeff == QRat::one() && self.power <= 0 {
            Some(-self.power)
        } else {
            None
        }
    }

    /// Attempt to compute sqrt of this QMonomial.
    /// Returns Some(sqrt(c)*q^{p/2}) if c is a perfect rational square and p is even.
    /// Returns None otherwise.
    pub fn try_sqrt(&self) -> Option<QMonomial> {
        if self.power % 2 != 0 {
            return None;
        }
        let num = self.coeff.numer().clone();
        let den = self.coeff.denom().clone();
        // Negative coefficient has no real sqrt
        if num < 0 {
            return None;
        }
        let num_sqrt = num.clone().sqrt();
        let den_sqrt = den.clone().sqrt();
        let num_check = rug::Integer::from(&num_sqrt * &num_sqrt);
        let den_check = rug::Integer::from(&den_sqrt * &den_sqrt);
        if num_check == num && den_check == den {
            let sqrt_coeff = QRat::from(rug::Rational::from((num_sqrt, den_sqrt)));
            Some(QMonomial::new(sqrt_coeff, self.power / 2))
        } else {
            None
        }
    }

    /// Negate: -(c*q^p) = (-c)*q^p.
    pub fn neg(&self) -> QMonomial {
        QMonomial::new(-self.coeff.clone(), self.power)
    }

    /// Check if this QMonomial represents zero (coeff is zero).
    pub fn is_zero(&self) -> bool {
        self.coeff.is_zero()
    }

    /// QMonomial representing 1 (= 1*q^0).
    pub fn one() -> QMonomial {
        QMonomial::q_power(0)
    }

    /// QMonomial representing q (= 1*q^1).
    pub fn q() -> QMonomial {
        QMonomial::q_power(1)
    }
}

/// The order parameter for a q-Pochhammer symbol (a;q)_n.
///
/// - `Finite(n)`: product of `|n|` factors (positive, zero, or negative)
/// - `Infinite`: infinite product (a;q)_inf
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PochhammerOrder {
    /// Finite order: (a;q)_n with n an integer (positive, zero, or negative).
    Finite(i64),
    /// Infinite order: (a;q)_inf = prod_{k=0}^{inf} (1 - a*q^k).
    Infinite,
}
