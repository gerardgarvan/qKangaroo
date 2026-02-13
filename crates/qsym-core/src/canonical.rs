//! Canonical construction helpers for commutative operations.
//!
//! These functions ensure that `Add` and `Mul` children are always sorted
//! by `ExprRef` (u32 numeric order), so that `a + b` and `b + a` produce
//! the same `ExprRef` after hash-consing.
//!
//! **Rule:** Never construct `Expr::Add` or `Expr::Mul` directly.
//! Always use `make_add` and `make_mul` to ensure canonical form.

use crate::arena::ExprArena;
use crate::expr::{Expr, ExprRef};
use crate::number::QInt;
use smallvec::SmallVec;

/// Construct a canonical addition.
///
/// - Sorts children by ExprRef (u32 numeric order).
/// - Deduplicates consecutive identical refs.
/// - 0 children -> integer zero (additive identity).
/// - 1 child -> returns that child directly (no wrapping).
/// - 2+ children -> `Expr::Add(sorted_children)`.
pub fn make_add(arena: &mut ExprArena, mut children: Vec<ExprRef>) -> ExprRef {
    children.sort();
    children.dedup();
    match children.len() {
        0 => arena.intern(Expr::Integer(QInt::from(0i64))),
        1 => children[0],
        _ => arena.intern(Expr::Add(children)),
    }
}

/// Construct a canonical multiplication.
///
/// - Sorts children by ExprRef (u32 numeric order).
/// - Deduplicates consecutive identical refs.
/// - 0 children -> integer one (multiplicative identity).
/// - 1 child -> returns that child directly (no wrapping).
/// - 2+ children -> `Expr::Mul(sorted_children)`.
pub fn make_mul(arena: &mut ExprArena, mut children: Vec<ExprRef>) -> ExprRef {
    children.sort();
    children.dedup();
    match children.len() {
        0 => arena.intern(Expr::Integer(QInt::from(1i64))),
        1 => children[0],
        _ => arena.intern(Expr::Mul(children)),
    }
}

/// Construct a negation: `-child`.
pub fn make_neg(arena: &mut ExprArena, child: ExprRef) -> ExprRef {
    arena.intern(Expr::Neg(child))
}

/// Construct exponentiation: `base^exp`.
pub fn make_pow(arena: &mut ExprArena, base: ExprRef, exp: ExprRef) -> ExprRef {
    arena.intern(Expr::Pow(base, exp))
}

/// Construct a q-Pochhammer symbol: `(base; nome)_order`.
pub fn make_qpochhammer(
    arena: &mut ExprArena,
    base: ExprRef,
    nome: ExprRef,
    order: ExprRef,
) -> ExprRef {
    arena.intern(Expr::QPochhammer { base, nome, order })
}

/// Construct a Jacobi theta function: `theta_index(nome)`.
///
/// `index` must be 1-4.
pub fn make_jacobi_theta(arena: &mut ExprArena, index: u8, nome: ExprRef) -> ExprRef {
    debug_assert!((1..=4).contains(&index), "Jacobi theta index must be 1-4");
    arena.intern(Expr::JacobiTheta { index, nome })
}

/// Construct a Dedekind eta function: `eta(tau)`.
pub fn make_dedekind_eta(arena: &mut ExprArena, tau: ExprRef) -> ExprRef {
    arena.intern(Expr::DedekindEta(tau))
}

/// Construct a basic hypergeometric series: `_r phi_s (upper; lower; nome, argument)`.
pub fn make_basic_hypergeometric(
    arena: &mut ExprArena,
    upper: SmallVec<[ExprRef; 4]>,
    lower: SmallVec<[ExprRef; 4]>,
    nome: ExprRef,
    argument: ExprRef,
) -> ExprRef {
    arena.intern(Expr::BasicHypergeometric {
        upper,
        lower,
        nome,
        argument,
    })
}
