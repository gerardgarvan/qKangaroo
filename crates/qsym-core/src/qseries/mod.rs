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
//!   [`findcong`], [`findnonhom`], [`findhomcombo`], [`findnonhomcombo`], [`Congruence`]

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

pub use factoring::{qfactor, QFactorization};
pub use linalg::{rational_null_space, build_coefficient_matrix, modular_null_space};
pub use relations::{findlincombo, findhom, findpoly, PolynomialRelation, findcong, findnonhom, findhomcombo, findnonhomcombo, Congruence};
pub use partitions::{partition_count, partition_gf, distinct_parts_gf, odd_parts_gf, bounded_parts_gf};
pub use pochhammer::aqprod;
pub use prodmake::{prodmake, InfiniteProductForm, etamake, EtaQuotient, jacprodmake, JacobiProductForm, mprodmake, qetamake, QEtaForm};
pub use products::{etaq, jacprod, tripleprod, quinprod, winquist};
pub use qbinomial::qbin;
pub use rank_crank::{rank_gf, crank_gf};
pub use theta::{theta2, theta3, theta4};
pub use utilities::{sift, qdegree, lqdegree};

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
