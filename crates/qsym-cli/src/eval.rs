//! Evaluator core for the q-Kangaroo REPL.
//!
//! Walks [`AstNode`] trees, manages a variable [`Environment`], performs
//! arithmetic on [`Value`] types, catches panics from qsym-core, and
//! dispatches function calls.

use qsym_core::number::{QInt, QRat};
use qsym_core::series::FormalPowerSeries;

/// A runtime value in the evaluator.
///
/// Unifies all possible return types from function calls and expressions.
#[derive(Clone, Debug)]
pub enum Value {
    /// Formal power series (most common return type).
    Series(FormalPowerSeries),
    /// Exact integer.
    Integer(QInt),
    /// Exact rational number.
    Rational(QRat),
    /// List of values.
    List(Vec<Value>),
    /// Key-value map (prodmake, etamake, qfactor results, etc.).
    Dict(Vec<(String, Value)>),
    /// Pair of values (heine transforms, bailey weak lemma).
    Pair(Box<Value>, Box<Value>),
    /// Boolean value.
    Bool(bool),
    /// None/null (try_summation returns None on failure).
    None,
    /// The infinity keyword.
    Infinity,
}

impl Value {
    /// Human-readable type name for error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Series(_) => "series",
            Value::Integer(_) => "integer",
            Value::Rational(_) => "rational",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Pair(_, _) => "pair",
            Value::Bool(_) => "bool",
            Value::None => "none",
            Value::Infinity => "infinity",
        }
    }
}
