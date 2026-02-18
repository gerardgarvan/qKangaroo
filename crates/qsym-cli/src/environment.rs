//! Variable environment and session state for the q-Kangaroo evaluator.
//!
//! The [`Environment`] holds user-defined variables, the last computed result,
//! a symbol registry (for `SymbolId`s used by `FormalPowerSeries`), and the
//! default truncation order.

use std::collections::HashMap;

use qsym_core::symbol::{SymbolId, SymbolRegistry};

use crate::eval::Value;

/// The evaluator's runtime environment.
///
/// Created once at REPL start and persists across lines. Holds all state
/// needed to evaluate expressions: variable bindings, the last result
/// (for `%` reference), and shared symbol interning for series construction.
pub struct Environment {
    /// User-defined variables (name -> value).
    pub variables: HashMap<String, Value>,
    /// Last computed result (for `%` reference).
    pub last_result: Option<Value>,
    /// Symbol registry (owns all interned symbol names).
    pub symbols: SymbolRegistry,
    /// Cached `SymbolId` for the variable "q".
    pub sym_q: SymbolId,
    /// Default truncation order for series construction.
    pub default_order: i64,
}

impl Environment {
    /// Create a new environment with default settings.
    ///
    /// Interns "q" as a symbol and sets the default truncation order to 20.
    pub fn new() -> Self {
        let mut symbols = SymbolRegistry::new();
        let sym_q = symbols.intern("q");
        Self {
            variables: HashMap::new(),
            last_result: None,
            symbols,
            sym_q,
            default_order: 20,
        }
    }

    /// Store a variable binding.
    pub fn set_var(&mut self, name: &str, val: Value) {
        self.variables.insert(name.to_string(), val);
    }

    /// Look up a variable by name.
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qsym_core::number::QInt;

    #[test]
    fn new_environment_has_sym_q() {
        let env = Environment::new();
        assert_eq!(env.symbols.name(env.sym_q), "q");
    }

    #[test]
    fn default_order_is_20() {
        let env = Environment::new();
        assert_eq!(env.default_order, 20);
    }

    #[test]
    fn set_and_get_variable() {
        let mut env = Environment::new();
        env.set_var("x", Value::Integer(QInt::from(42i64)));
        let val = env.get_var("x");
        assert!(val.is_some());
        if let Some(Value::Integer(n)) = val {
            assert_eq!(*n, QInt::from(42i64));
        } else {
            panic!("expected Integer value");
        }
    }

    #[test]
    fn get_missing_variable_returns_none() {
        let env = Environment::new();
        assert!(env.get_var("nonexistent").is_none());
    }

    #[test]
    fn last_result_initially_none() {
        let env = Environment::new();
        assert!(env.last_result.is_none());
    }
}
