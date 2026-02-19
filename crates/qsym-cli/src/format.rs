//! Output formatting for evaluator [`Value`] types.
//!
//! Formats each `Value` variant as a human-readable string suitable for
//! REPL output. Series and numbers delegate to their `Display` impls;
//! structured types (List, Dict, Pair) are formatted with bracket notation.
//!
//! Also provides [`format_latex`] for LaTeX rendering of any `Value`.

use std::cmp::Ordering;
use std::fmt::Write;

use qsym_core::number::QRat;
use qsym_core::series::FormalPowerSeries;

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
        Value::String(s) => s.clone(),
        Value::None => "NONE".to_string(),
        Value::Infinity => "infinity".to_string(),
        Value::Symbol(name) => name.clone(),
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

// ---------------------------------------------------------------------------
// LaTeX formatting
// ---------------------------------------------------------------------------

/// Format a [`Value`] as a LaTeX string.
///
/// # Output conventions
///
/// - **Series**: uses `fps_to_latex` (ported from Python's `QSeries.latex()`)
/// - **Integer**: plain number
/// - **Rational**: `\frac{numer}{denom}` (handles negative)
/// - **List**: comma-joined in `\left[...\right]`
/// - **Dict**: `\{key: val, ...\}`
/// - **Pair**: `\left(a, b\right)`
/// - **Bool**: `\text{true}` / `\text{false}`
/// - **None**: `\text{NONE}`
/// - **Infinity**: `\infty`
pub fn format_latex(val: &Value) -> String {
    match val {
        Value::Series(fps) => fps_to_latex(fps),
        Value::Integer(n) => format!("{}", n),
        Value::Rational(r) => {
            let is_negative = r.0.cmp0() == Ordering::Less;
            let numer = r.numer();
            let denom = r.denom();
            if *denom == *rug::Integer::ONE {
                format!("{}", numer)
            } else if is_negative {
                // -3/7 → "-\frac{3}{7}"
                let abs_numer = numer.clone().abs();
                format!("-\\frac{{{}}}{{{}}}", abs_numer, denom)
            } else {
                format!("\\frac{{{}}}{{{}}}", numer, denom)
            }
        }
        Value::List(items) => {
            let parts: Vec<String> = items.iter().map(format_latex).collect();
            format!("\\left[{}\\right]", parts.join(", "))
        }
        Value::Dict(entries) => {
            let parts: Vec<String> = entries
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_latex(v)))
                .collect();
            format!("\\{{{}\\}}", parts.join(", "))
        }
        Value::Pair(a, b) => {
            format!("\\left({}, {}\\right)", format_latex(a), format_latex(b))
        }
        Value::Bool(b) => {
            if *b {
                "\\text{true}".to_string()
            } else {
                "\\text{false}".to_string()
            }
        }
        Value::String(s) => format!("\\text{{{}}}", s),
        Value::None => "\\text{NONE}".to_string(),
        Value::Infinity => "\\infty".to_string(),
        Value::Symbol(name) => name.clone(),
    }
}

/// Convert a `FormalPowerSeries` to LaTeX notation.
///
/// Ported from `crates/qsym-python/src/series.rs` `QSeries::latex()`.
/// Shows up to 20 terms (first 15 + `\cdots` + last 2 if more),
/// followed by `O(q^{N})`.
fn fps_to_latex(fps: &FormalPowerSeries) -> String {
    let trunc = fps.truncation_order();
    let terms: Vec<(&i64, &QRat)> = fps.iter().collect();
    let total = terms.len();

    if total == 0 {
        return format!("O(q^{{{}}})", trunc);
    }

    // Determine which terms to show.
    let (show_first, show_last, ellipsis) = if total > 20 {
        (15, 2, true)
    } else {
        (total, 0, false)
    };

    let mut result = String::new();

    // Write first group of terms
    for (i, (k, c)) in terms.iter().enumerate().take(show_first) {
        latex_term(&mut result, i == 0, **k, c);
    }

    // Ellipsis and last terms
    if ellipsis {
        let _ = write!(result, " + \\cdots");
        let start = total - show_last;
        for (k, c) in &terms[start..] {
            latex_term(&mut result, false, **k, c);
        }
    }

    // Truncation order
    let _ = write!(result, " + O(q^{{{}}})", trunc);

    result
}

/// Format a single term of a LaTeX series representation.
///
/// Ported from `crates/qsym-python/src/series.rs` `latex_term()`.
fn latex_term(out: &mut String, first: bool, k: i64, c: &QRat) {
    let is_negative = c.0.cmp0() == Ordering::Less;
    let abs_c = if is_negative { -c.clone() } else { c.clone() };
    let abs_numer = abs_c.numer().clone();
    let abs_denom = abs_c.denom().clone();
    let abs_is_one = abs_numer.cmp0() != Ordering::Equal && abs_numer == abs_denom;
    let denom_is_one = abs_denom == *rug::Integer::ONE;

    // Sign
    if first {
        if is_negative {
            out.push('-');
        }
    } else if is_negative {
        let _ = write!(out, " - ");
    } else {
        let _ = write!(out, " + ");
    }

    // Format coefficient + variable
    if k == 0 {
        // Constant term: just the coefficient
        if denom_is_one {
            let _ = write!(out, "{}", abs_numer);
        } else {
            let _ = write!(out, "\\frac{{{}}}{{{}}}", abs_numer, abs_denom);
        }
    } else if abs_is_one {
        // Coefficient is 1: just the variable part
        if k == 1 {
            out.push('q');
        } else {
            let _ = write!(out, "q^{{{}}}", k);
        }
    } else {
        // General coefficient * variable
        let coeff_str = if denom_is_one {
            format!("{}", abs_numer)
        } else {
            format!("\\frac{{{}}}{{{}}}", abs_numer, abs_denom)
        };
        if k == 1 {
            let _ = write!(out, "{} q", coeff_str);
        } else {
            let _ = write!(out, "{} q^{{{}}}", coeff_str, k);
        }
    }
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

    // -- format_latex tests -------------------------------------------------

    #[test]
    fn format_latex_integer() {
        let val = Value::Integer(QInt::from(42i64));
        assert_eq!(format_latex(&val), "42");
    }

    #[test]
    fn format_latex_negative_rational() {
        let val = Value::Rational(QRat::from((-3i64, 7i64)));
        assert_eq!(format_latex(&val), "-\\frac{3}{7}");
    }

    #[test]
    fn format_latex_positive_rational() {
        let val = Value::Rational(QRat::from((3i64, 7i64)));
        assert_eq!(format_latex(&val), "\\frac{3}{7}");
    }

    #[test]
    fn format_latex_series() {
        let mut reg = SymbolRegistry::new();
        let sym_q = reg.intern("q");
        let fps = FormalPowerSeries::monomial(sym_q, QRat::one(), 1, 10);
        let val = Value::Series(fps);
        let result = format_latex(&val);
        assert!(result.contains("q"), "expected 'q' in: {}", result);
        assert!(result.contains("O(q^{"), "expected 'O(q^{{' in: {}", result);
    }

    #[test]
    fn format_latex_infinity() {
        assert_eq!(format_latex(&Value::Infinity), "\\infty");
    }

    #[test]
    fn format_latex_bool() {
        assert_eq!(format_latex(&Value::Bool(true)), "\\text{true}");
        assert_eq!(format_latex(&Value::Bool(false)), "\\text{false}");
    }

    #[test]
    fn format_latex_none() {
        assert_eq!(format_latex(&Value::None), "\\text{NONE}");
    }

    #[test]
    fn format_latex_list() {
        let val = Value::List(vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(2i64)),
        ]);
        assert_eq!(format_latex(&val), "\\left[1, 2\\right]");
    }

    #[test]
    fn format_latex_empty_series() {
        let mut reg = SymbolRegistry::new();
        let sym_q = reg.intern("q");
        let fps = FormalPowerSeries::zero(sym_q, 10);
        let val = Value::Series(fps);
        let result = format_latex(&val);
        assert_eq!(result, "O(q^{10})");
    }
}
