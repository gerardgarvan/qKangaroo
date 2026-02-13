//! Expression arena with hash-consing deduplication.
//!
//! The `ExprArena` is the central data structure of Q-Symbolic. All expressions
//! are stored here, and hash-consing ensures that structurally identical
//! expressions share a single `ExprRef`.

use crate::expr::{Expr, ExprRef};
use crate::number::{QInt, QRat};
use crate::symbol::SymbolRegistry;
use rustc_hash::FxHashMap;

/// The expression arena: owns all expression data with hash-consing deduplication.
///
/// Guarantees:
/// - Structurally identical expressions get the same `ExprRef`.
/// - `ExprRef` comparison is O(1) structural equality.
/// - Append-only: expressions are never removed or modified.
///
/// # Example
///
/// ```
/// use qsym_core::{ExprArena, Expr};
///
/// let mut arena = ExprArena::new();
/// let x = arena.intern_symbol("x");
/// let x2 = arena.intern_symbol("x");
/// assert_eq!(x, x2); // same symbol -> same ExprRef
/// ```
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

    /// The core operation: intern an expression.
    ///
    /// If a structurally identical expression already exists in the arena,
    /// returns the existing `ExprRef`. Otherwise, stores the expression
    /// and returns a new `ExprRef`.
    ///
    /// This is the only way to create `ExprRef` values.
    pub fn intern(&mut self, expr: Expr) -> ExprRef {
        if let Some(&existing) = self.dedup.get(&expr) {
            return existing;
        }
        let id = ExprRef(self.nodes.len() as u32);
        self.nodes.push(expr.clone());
        self.dedup.insert(expr, id);
        id
    }

    /// O(1) lookup: retrieve the expression for a given `ExprRef`.
    ///
    /// # Panics
    ///
    /// Panics if the `ExprRef` is invalid (should never happen with
    /// arena-allocated refs).
    pub fn get(&self, r: ExprRef) -> &Expr {
        &self.nodes[r.0 as usize]
    }

    /// Number of interned expressions (unique by structure).
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Convenience: intern an integer value.
    pub fn intern_int(&mut self, val: impl Into<rug::Integer>) -> ExprRef {
        self.intern(Expr::Integer(QInt(val.into())))
    }

    /// Convenience: intern a rational value from numerator and denominator.
    pub fn intern_rat(
        &mut self,
        num: impl Into<rug::Integer>,
        den: impl Into<rug::Integer>,
    ) -> ExprRef {
        let rational = rug::Rational::from((num.into(), den.into()));
        self.intern(Expr::Rational(QRat(rational)))
    }

    /// Convenience: intern a symbol by name.
    ///
    /// Interns the name in the `SymbolRegistry` first, then interns
    /// the `Expr::Symbol` in the arena. Calling with the same name
    /// always returns the same `ExprRef`.
    pub fn intern_symbol(&mut self, name: &str) -> ExprRef {
        let sym_id = self.symbols.intern(name);
        self.intern(Expr::Symbol(sym_id))
    }

    /// Accessor for the symbol registry (for rendering, debugging).
    pub fn symbols(&self) -> &SymbolRegistry {
        &self.symbols
    }

    /// Mutable accessor for the symbol registry.
    pub fn symbols_mut(&mut self) -> &mut SymbolRegistry {
        &mut self.symbols
    }
}

impl Default for ExprArena {
    fn default() -> Self {
        Self::new()
    }
}
