//! Q-Symbolic Core: expression IR, arena, and symbolic computation engine.
//!
//! This crate provides the foundational expression representation for the
//! Q-Symbolic symbolic computation engine for q-series.

pub mod arena;
pub mod canonical;
pub mod expr;
pub mod number;
pub mod poly;
pub mod qseries;
pub mod render;
pub mod series;
pub mod simplify;
pub mod symbol;

// Re-export key types at crate root for convenience.
pub use arena::ExprArena;
pub use expr::{Expr, ExprRef};
pub use number::{QInt, QRat};
pub use poly::{Factorization, QRatPoly, QRatRationalFunc, factor_over_q, poly_gcd, poly_resultant};
pub use symbol::{SymbolId, SymbolRegistry};
