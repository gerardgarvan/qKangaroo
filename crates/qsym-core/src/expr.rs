//! Expression types: the `Expr` enum and `ExprRef` handle.
//!
//! `ExprRef` is a lightweight, `Copy` handle (u32 index) into the `ExprArena`.
//! `Expr` is the actual expression data, stored in the arena.
//!
//! After hash-consing, structural equality is O(1): just compare `ExprRef` values.

use crate::number::{QInt, QRat};
use crate::symbol::SymbolId;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::fmt;

/// A reference to an expression in the arena.
///
/// This is just a u32 index. It is `Copy`, `Hash`, `Eq`, and `Ord`.
/// Two `ExprRef`s are equal if and only if they point to structurally
/// identical expressions (guaranteed by hash-consing).
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub struct ExprRef(pub(crate) u32);

impl fmt::Display for ExprRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// The expression enum representing all node types in the symbolic IR.
///
/// All variants are immutable once interned in the arena.
/// Hash and Eq are derived to support the dedup HashMap in ExprArena.
///
/// **Variants (14 total):**
/// - Atoms: Integer, Rational, Symbol, Infinity, Undefined
/// - Arithmetic: Add, Mul, Neg, Pow
/// - q-Specific: QPochhammer, JacobiTheta, DedekindEta, BasicHypergeometric
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Expr {
    // --- Atoms ---
    /// Arbitrary-precision integer.
    Integer(QInt),

    /// Exact rational number (always in lowest terms via rug).
    Rational(QRat),

    /// Interned variable/symbol name.
    Symbol(SymbolId),

    /// Formal infinity, used as order in (a;q)_inf.
    Infinity,

    /// Undefined/indeterminate result.
    Undefined,

    // --- Arithmetic (n-ary, canonically sorted) ---
    /// Sum of terms. Children are sorted by ExprRef for canonical form.
    /// Invariant: len >= 2 (enforced by `make_add`).
    Add(Vec<ExprRef>),

    /// Product of factors. Children are sorted by ExprRef for canonical form.
    /// Invariant: len >= 2 (enforced by `make_mul`).
    Mul(Vec<ExprRef>),

    /// Unary negation.
    Neg(ExprRef),

    /// Exponentiation: base^exponent.
    Pow(ExprRef, ExprRef),

    // --- q-Specific Primitives ---
    /// q-Pochhammer symbol: (base; nome)_order
    ///
    /// Standard notation: (a; q)_n where a is the base, q is the nome,
    /// and n is the order (which can be Integer, Symbol, or Infinity).
    QPochhammer {
        base: ExprRef,
        nome: ExprRef,
        order: ExprRef,
    },

    /// Jacobi theta function: theta_index(nome)
    ///
    /// index is 1-4 corresponding to theta_1 through theta_4.
    JacobiTheta {
        index: u8,
        nome: ExprRef,
    },

    /// Dedekind eta function: eta(tau)
    DedekindEta(ExprRef),

    /// Basic hypergeometric series: _r phi_s (upper; lower; nome, argument)
    ///
    /// SmallVec<[ExprRef; 4]> avoids heap allocation for common cases
    /// (most series are _2phi_1 or _3phi_2).
    BasicHypergeometric {
        upper: SmallVec<[ExprRef; 4]>,
        lower: SmallVec<[ExprRef; 4]>,
        nome: ExprRef,
        argument: ExprRef,
    },
}

impl Expr {
    /// Returns a human-readable name for the variant (for debugging).
    pub fn variant_name(&self) -> &'static str {
        match self {
            Expr::Integer(_) => "Integer",
            Expr::Rational(_) => "Rational",
            Expr::Symbol(_) => "Symbol",
            Expr::Infinity => "Infinity",
            Expr::Undefined => "Undefined",
            Expr::Add(_) => "Add",
            Expr::Mul(_) => "Mul",
            Expr::Neg(_) => "Neg",
            Expr::Pow(_, _) => "Pow",
            Expr::QPochhammer { .. } => "QPochhammer",
            Expr::JacobiTheta { .. } => "JacobiTheta",
            Expr::DedekindEta(_) => "DedekindEta",
            Expr::BasicHypergeometric { .. } => "BasicHypergeometric",
        }
    }
}
