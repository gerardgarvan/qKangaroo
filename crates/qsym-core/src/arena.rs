//! Expression arena with hash-consing deduplication.
//!
//! Placeholder -- will be implemented in Task 2.

use crate::expr::{Expr, ExprRef};
use crate::symbol::SymbolRegistry;
use rustc_hash::FxHashMap;

/// The expression arena: owns all expression data with hash-consing deduplication.
///
/// Guarantees:
/// - Structurally identical expressions get the same `ExprRef`.
/// - `ExprRef` comparison is O(1) structural equality.
/// - Append-only: expressions are never removed.
pub struct ExprArena {
    nodes: Vec<Expr>,
    dedup: FxHashMap<Expr, ExprRef>,
    symbols: SymbolRegistry,
}

impl ExprArena {
    /// Create an empty arena.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            dedup: FxHashMap::default(),
            symbols: SymbolRegistry::new(),
        }
    }

    /// Accessor for the symbol registry.
    pub fn symbols(&self) -> &SymbolRegistry {
        &self.symbols
    }
}

impl Default for ExprArena {
    fn default() -> Self {
        Self::new()
    }
}
