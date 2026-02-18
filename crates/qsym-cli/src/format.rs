//! Output formatting for evaluator [`Value`] types.
//!
//! Formats each `Value` variant as a human-readable string suitable for
//! REPL output. Series and numbers delegate to their `Display` impls;
//! structured types (List, Dict, Pair) are formatted with bracket notation.

use crate::eval::Value;

/// Format a [`Value`] as a human-readable string.
///
/// # Output conventions
///
/// - **Series**: uses `FormalPowerSeries::Display` (e.g., `1 - q + O(q^20)`)
/// - **Integer**: plain number (e.g., `42`)
/// - **Rational**: fraction (e.g., `3/7`)
/// - **List**: `[item1, item2, ...]`; matrix (list-of-lists) puts each row on its own line
/// - **Dict**: `{key1: val1, key2: val2}`
/// - **Pair**: `(a, b)`
/// - **Bool**: `true` / `false`
/// - **None**: `NONE`
/// - **Infinity**: `infinity`
pub fn format_value(val: &Value) -> String {
    match val {
        Value::Series(fps) => format!("{}", fps),
        Value::Integer(n) => format!("{}", n),
        Value::Rational(r) => format!("{}", r),
        Value::List(items) => format_list(items),
        Value::Dict(entries) => format_dict(entries),
        Value::Pair(a, b) => format!("({}, {})", format_value(a), format_value(b)),
        Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::None => "NONE".to_string(),
        Value::Infinity => "infinity".to_string(),
    }
}

/// Format a list of values. For list-of-lists (matrix), put each inner
/// list on its own line for readability.
fn format_list(items: &[Value]) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }

    // Check if this is a matrix (list of lists)
    let is_matrix = items.len() > 1
        && items.iter().all(|v| matches!(v, Value::List(_)));

    if is_matrix {
        let mut parts = Vec::new();
        for item in items {
            parts.push(format_value(item));
        }
        format!("[{}", parts.join(",\n ")) + "]"
    } else {
        let parts: Vec<String> = items.iter().map(format_value).collect();
        format!("[{}]", parts.join(", "))
    }
}

/// Format a dict (list of key-value pairs).
fn format_dict(entries: &[(String, Value)]) -> String {
    if entries.is_empty() {
        return "{}".to_string();
    }
    let parts: Vec<String> = entries
        .iter()
        .map(|(k, v)| format!("{}: {}", k, format_value(v)))
        .collect();
    format!("{{{}}}", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use qsym_core::number::{QInt, QRat};
    use qsym_core::series::FormalPowerSeries;
    use qsym_core::symbol::SymbolRegistry;

    #[test]
    fn format_integer() {
        let val = Value::Integer(QInt::from(42i64));
        assert_eq!(format_value(&val), "42");
    }

    #[test]
    fn format_negative_integer() {
        let val = Value::Integer(QInt::from(-7i64));
        assert_eq!(format_value(&val), "-7");
    }

    #[test]
    fn format_rational() {
        let val = Value::Rational(QRat::from((3i64, 7i64)));
        assert_eq!(format_value(&val), "3/7");
    }

    #[test]
    fn format_bool_true() {
        assert_eq!(format_value(&Value::Bool(true)), "true");
    }

    #[test]
    fn format_bool_false() {
        assert_eq!(format_value(&Value::Bool(false)), "false");
    }

    #[test]
    fn format_none() {
        assert_eq!(format_value(&Value::None), "NONE");
    }

    #[test]
    fn format_infinity() {
        assert_eq!(format_value(&Value::Infinity), "infinity");
    }

    #[test]
    fn format_empty_list() {
        let val = Value::List(vec![]);
        assert_eq!(format_value(&val), "[]");
    }

    #[test]
    fn format_integer_list() {
        let val = Value::List(vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(3i64)),
        ]);
        assert_eq!(format_value(&val), "[1, 2, 3]");
    }

    #[test]
    fn format_matrix() {
        let val = Value::List(vec![
            Value::List(vec![
                Value::Integer(QInt::from(1i64)),
                Value::Integer(QInt::from(0i64)),
            ]),
            Value::List(vec![
                Value::Integer(QInt::from(0i64)),
                Value::Integer(QInt::from(1i64)),
            ]),
        ]);
        let result = format_value(&val);
        assert!(result.contains("[1, 0]"));
        assert!(result.contains("[0, 1]"));
    }

    #[test]
    fn format_dict() {
        let val = Value::Dict(vec![
            ("a".to_string(), Value::Integer(QInt::from(1i64))),
            ("b".to_string(), Value::Integer(QInt::from(2i64))),
        ]);
        assert_eq!(format_value(&val), "{a: 1, b: 2}");
    }

    #[test]
    fn format_empty_dict() {
        let val = Value::Dict(vec![]);
        assert_eq!(format_value(&val), "{}");
    }

    #[test]
    fn format_pair() {
        let val = Value::Pair(
            Box::new(Value::Integer(QInt::from(1i64))),
            Box::new(Value::Integer(QInt::from(2i64))),
        );
        assert_eq!(format_value(&val), "(1, 2)");
    }

    #[test]
    fn format_series() {
        let mut reg = SymbolRegistry::new();
        let sym_q = reg.intern("q");
        let fps = FormalPowerSeries::monomial(sym_q, QRat::one(), 1, 10);
        let val = Value::Series(fps);
        let result = format_value(&val);
        assert!(result.contains("q"), "expected 'q' in: {}", result);
        assert!(result.contains("O(q^10)"), "expected 'O(q^10)' in: {}", result);
    }
}
