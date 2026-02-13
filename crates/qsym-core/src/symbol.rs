//! Symbol interning: maps string names to compact `SymbolId` indices.
//!
//! The `SymbolRegistry` is append-only -- symbols are never removed.
//! This ensures `SymbolId` values are always valid.

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A compact identifier for an interned symbol name.
///
/// `SymbolId` is `Copy` and cheap to compare (u32 equality).
/// Use `SymbolRegistry::name()` to retrieve the original string.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub struct SymbolId(pub(crate) u32);

impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SymbolId({})", self.0)
    }
}

/// Append-only registry mapping symbol names to `SymbolId` indices.
///
/// Guarantees:
/// - Interning the same name twice returns the same `SymbolId`.
/// - `SymbolId` values are stable (never invalidated).
/// - O(1) lookup by name (via `FxHashMap`) and by id (via `Vec` index).
#[derive(Debug)]
pub struct SymbolRegistry {
    names: Vec<String>,
    lookup: FxHashMap<String, SymbolId>,
}

impl SymbolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            lookup: FxHashMap::default(),
        }
    }

    /// Intern a symbol name, returning its `SymbolId`.
    ///
    /// If the name has been interned before, returns the existing id.
    /// Otherwise, allocates a new id.
    pub fn intern(&mut self, name: &str) -> SymbolId {
        if let Some(&id) = self.lookup.get(name) {
            return id;
        }
        let id = SymbolId(self.names.len() as u32);
        self.names.push(name.to_owned());
        self.lookup.insert(name.to_owned(), id);
        id
    }

    /// Retrieve the name for a given `SymbolId`.
    ///
    /// # Panics
    ///
    /// Panics if the id is invalid (should never happen with append-only semantics).
    pub fn name(&self, id: SymbolId) -> &str {
        &self.names[id.0 as usize]
    }

    /// Number of interned symbols.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

impl Default for SymbolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_returns_same_id() {
        let mut reg = SymbolRegistry::new();
        let id1 = reg.intern("q");
        let id2 = reg.intern("q");
        assert_eq!(id1, id2);
    }

    #[test]
    fn different_names_different_ids() {
        let mut reg = SymbolRegistry::new();
        let q = reg.intern("q");
        let a = reg.intern("a");
        assert_ne!(q, a);
    }

    #[test]
    fn name_roundtrip() {
        let mut reg = SymbolRegistry::new();
        let id = reg.intern("tau");
        assert_eq!(reg.name(id), "tau");
    }

    #[test]
    fn len_tracks_unique_names() {
        let mut reg = SymbolRegistry::new();
        assert_eq!(reg.len(), 0);
        reg.intern("q");
        assert_eq!(reg.len(), 1);
        reg.intern("q"); // duplicate
        assert_eq!(reg.len(), 1);
        reg.intern("a");
        assert_eq!(reg.len(), 2);
    }
}
