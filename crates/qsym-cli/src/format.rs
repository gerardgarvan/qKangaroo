//! Output formatting for evaluator [`Value`] types.
//!
//! Formats each `Value` variant as a human-readable string suitable for
//! REPL output. Series use [`format_series`] with variable-name lookup from
//! `SymbolRegistry`; numbers delegate to their `Display` impls; structured
//! types (List, Dict, Pair) are formatted with bracket notation.
//!
//! Also provides [`format_latex`] for LaTeX rendering of any `Value`.

use std::cmp::Ordering;
use std::fmt::Write;

use qsym_core::number::QRat;
use qsym_core::series::FormalPowerSeries;
use qsym_core::symbol::SymbolRegistry;

use crate::eval::{Value, POLYNOMIAL_ORDER};

/// Format a [`Value`] as a human-readable string.
///
/// # Output conventions
///
/// - **Series**: variable-aware format with optional `O(var^N)` truncation
/// - **Integer**: plain number (e.g., `42`)
/// - **Rational**: fraction (e.g., `3/7`)
/// - **List**: `[item1, item2, ...]`; matrix (list-of-lists) puts each row on its own line
/// - **Dict**: `{key1: val1, key2: val2}`
/// - **Pair**: `(a, b)`
/// - **Bool**: `true` / `false`
/// - **None**: `NONE`
/// - **Infinity**: `infinity`
pub fn format_value(val: &Value, symbols: &SymbolRegistry) -> String {
    match val {
        Value::Series(fps) => format_series(fps, symbols),
        Value::Integer(n) => format!("{}", n),
        Value::Rational(r) => format!("{}", r),
        Value::List(items) => format_list(items, symbols),
        Value::Dict(entries) => format_dict(entries, symbols),
        Value::Pair(a, b) => format!("({}, {})", format_value(a, symbols), format_value(b, symbols)),
        Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::String(s) => s.clone(),
        Value::None => "NONE".to_string(),
        Value::Infinity => "infinity".to_string(),
        Value::Symbol(name) => name.clone(),
        Value::JacobiProduct(factors) => format_jacobi_product(factors),
    }
}

/// Format a JacobiProduct value as human-readable string.
/// Examples: "JAC(1,5)", "JAC(1,5)*JAC(2,5)", "JAC(1,5)^(-1)", "1" (empty product)
fn format_jacobi_product(factors: &[(i64, i64, i64)]) -> String {
    if factors.is_empty() {
        return "1".to_string();
    }
    let parts: Vec<String> = factors.iter().map(|&(a, b, exp)| {
        if exp == 1 {
            format!("JAC({},{})", a, b)
        } else {
            format!("JAC({},{})^({})", a, b, exp)
        }
    }).collect();
    parts.join("*")
}

/// Format a JacobiProduct value as LaTeX.
/// Uses (q^a;q^b)_\infty notation.
fn format_jacobi_product_latex(factors: &[(i64, i64, i64)]) -> String {
    if factors.is_empty() {
        return "1".to_string();
    }
    let parts: Vec<String> = factors.iter().map(|&(a, b, exp)| {
        let base = format!("(q^{{{}}};q^{{{}}})_{{\\infty}}", a, b);
        if exp == 1 {
            base
        } else {
            format!("{}^{{{}}}", base, exp)
        }
    }).collect();
    parts.join(" \\cdot ")
}

/// Format a list of values. For list-of-lists (matrix), put each inner
/// list on its own line for readability.
fn format_list(items: &[Value], symbols: &SymbolRegistry) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }

    // Check if this is a matrix (list of lists)
    let is_matrix = items.len() > 1
        && items.iter().all(|v| matches!(v, Value::List(_)));

    if is_matrix {
        let mut parts = Vec::new();
        for item in items {
            parts.push(format_value(item, symbols));
        }
        format!("[{}", parts.join(",\n ")) + "]"
    } else {
        let parts: Vec<String> = items.iter().map(|v| format_value(v, symbols)).collect();
        format!("[{}]", parts.join(", "))
    }
}

/// Format a dict (list of key-value pairs).
fn format_dict(entries: &[(String, Value)], symbols: &SymbolRegistry) -> String {
    if entries.is_empty() {
        return "{}".to_string();
    }
    let parts: Vec<String> = entries
        .iter()
        .map(|(k, v)| format!("{}: {}", k, format_value(v, symbols)))
        .collect();
    format!("{{{}}}", parts.join(", "))
}

// ---------------------------------------------------------------------------
// Series formatting (variable-aware, polynomial-aware)
// ---------------------------------------------------------------------------

/// Format a `FormalPowerSeries` using the symbol registry for variable names.
///
/// Polynomials (with `POLYNOMIAL_ORDER` sentinel) display without `O(...)`.
/// Truncated series display with `O(var^N)`.
fn format_series(fps: &FormalPowerSeries, symbols: &SymbolRegistry) -> String {
    let var = symbols.name(fps.variable());
    let trunc = fps.truncation_order();
    let is_polynomial = trunc >= POLYNOMIAL_ORDER;
    let mut first = true;
    let mut out = String::new();

    for (&k, c) in fps.iter() {
        let is_negative = c.0.cmp0() == Ordering::Less;
        let abs_c = if is_negative { -c.clone() } else { c.clone() };

        let abs_is_one = abs_c.0.numer().cmp0() != Ordering::Equal
            && *abs_c.0.numer() == *abs_c.0.denom();

        if first {
            if is_negative {
                out.push('-');
            }
            if k == 0 {
                let _ = write!(out, "{}", abs_c);
            } else if abs_is_one {
                if k == 1 {
                    let _ = write!(out, "{}", var);
                } else {
                    let _ = write!(out, "{}^{}", var, k);
                }
            } else {
                if k == 1 {
                    let _ = write!(out, "{}*{}", abs_c, var);
                } else {
                    let _ = write!(out, "{}*{}^{}", abs_c, var, k);
                }
            }
            first = false;
        } else {
            if is_negative {
                let _ = write!(out, " - ");
            } else {
                let _ = write!(out, " + ");
            }
            if k == 0 {
                let _ = write!(out, "{}", abs_c);
            } else if abs_is_one {
                if k == 1 {
                    let _ = write!(out, "{}", var);
                } else {
                    let _ = write!(out, "{}^{}", var, k);
                }
            } else {
                if k == 1 {
                    let _ = write!(out, "{}*{}", abs_c, var);
                } else {
                    let _ = write!(out, "{}*{}^{}", abs_c, var, k);
                }
            }
        }
    }

    // Append truncation order (only for non-polynomial series)
    if !is_polynomial {
        if first {
            let _ = write!(out, "O({}^{})", var, trunc);
        } else {
            let _ = write!(out, " + O({}^{})", var, trunc);
        }
    } else if first {
        // Polynomial with zero terms -> "0"
        out.push('0');
    }

    out
}

// ---------------------------------------------------------------------------
// LaTeX formatting
// ---------------------------------------------------------------------------

/// Format a [`Value`] as a LaTeX string.
///
/// # Output conventions
///
/// - **Series**: uses `fps_to_latex` with variable-aware names
/// - **Integer**: plain number
/// - **Rational**: `\frac{numer}{denom}` (handles negative)
/// - **List**: comma-joined in `\left[...\right]`
/// - **Dict**: `\{key: val, ...\}`
/// - **Pair**: `\left(a, b\right)`
/// - **Bool**: `\text{true}` / `\text{false}`
/// - **None**: `\text{NONE}`
/// - **Infinity**: `\infty`
pub fn format_latex(val: &Value, symbols: &SymbolRegistry) -> String {
    match val {
        Value::Series(fps) => fps_to_latex(fps, symbols),
        Value::Integer(n) => format!("{}", n),
        Value::Rational(r) => {
            let is_negative = r.0.cmp0() == Ordering::Less;
            let numer = r.numer();
            let denom = r.denom();
            if *denom == *rug::Integer::ONE {
                format!("{}", numer)
            } else if is_negative {
                // -3/7 -> "-\frac{3}{7}"
                let abs_numer = numer.clone().abs();
                format!("-\\frac{{{}}}{{{}}}", abs_numer, denom)
            } else {
                format!("\\frac{{{}}}{{{}}}", numer, denom)
            }
        }
        Value::List(items) => {
            let parts: Vec<String> = items.iter().map(|v| format_latex(v, symbols)).collect();
            format!("\\left[{}\\right]", parts.join(", "))
        }
        Value::Dict(entries) => {
            let parts: Vec<String> = entries
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_latex(v, symbols)))
                .collect();
            format!("\\{{{}\\}}", parts.join(", "))
        }
        Value::Pair(a, b) => {
            format!("\\left({}, {}\\right)", format_latex(a, symbols), format_latex(b, symbols))
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
        Value::JacobiProduct(factors) => format_jacobi_product_latex(factors),
    }
}

/// Convert a `FormalPowerSeries` to LaTeX notation.
///
/// Ported from `crates/qsym-python/src/series.rs` `QSeries::latex()`.
/// Shows up to 20 terms (first 15 + `\cdots` + last 2 if more),
/// followed by `O(var^{N})` (suppressed for polynomials).
fn fps_to_latex(fps: &FormalPowerSeries, symbols: &SymbolRegistry) -> String {
    let var = symbols.name(fps.variable());
    let trunc = fps.truncation_order();
    let is_polynomial = trunc >= POLYNOMIAL_ORDER;
    let terms: Vec<(&i64, &QRat)> = fps.iter().collect();
    let total = terms.len();

    if total == 0 {
        if is_polynomial {
            return "0".to_string();
        }
        return format!("O({}^{{{}}})", var, trunc);
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
        latex_term(&mut result, i == 0, **k, c, var);
    }

    // Ellipsis and last terms
    if ellipsis {
        let _ = write!(result, " + \\cdots");
        let start = total - show_last;
        for (k, c) in &terms[start..] {
            latex_term(&mut result, false, **k, c, var);
        }
    }

    // Truncation order (suppress for polynomials)
    if !is_polynomial {
        let _ = write!(result, " + O({}^{{{}}})", var, trunc);
    }

    result
}

/// Format a single term of a LaTeX series representation.
///
/// Ported from `crates/qsym-python/src/series.rs` `latex_term()`.
fn latex_term(out: &mut String, first: bool, k: i64, c: &QRat, var: &str) {
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
            let _ = write!(out, "{}", var);
        } else {
            let _ = write!(out, "{}^{{{}}}", var, k);
        }
    } else {
        // General coefficient * variable
        let coeff_str = if denom_is_one {
            format!("{}", abs_numer)
        } else {
            format!("\\frac{{{}}}{{{}}}", abs_numer, abs_denom)
        };
        if k == 1 {
            let _ = write!(out, "{} {}", coeff_str, var);
        } else {
            let _ = write!(out, "{} {}^{{{}}}", coeff_str, var, k);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qsym_core::number::{QInt, QRat};
    use qsym_core::series::FormalPowerSeries;
    use qsym_core::symbol::SymbolRegistry;

    fn empty_reg() -> SymbolRegistry {
        SymbolRegistry::new()
    }

    fn q_reg() -> (SymbolRegistry, qsym_core::symbol::SymbolId) {
        let mut reg = SymbolRegistry::new();
        let sym_q = reg.intern("q");
        (reg, sym_q)
    }

    #[test]
    fn format_integer() {
        let reg = empty_reg();
        let val = Value::Integer(QInt::from(42i64));
        assert_eq!(format_value(&val, &reg), "42");
    }

    #[test]
    fn format_negative_integer() {
        let reg = empty_reg();
        let val = Value::Integer(QInt::from(-7i64));
        assert_eq!(format_value(&val, &reg), "-7");
    }

    #[test]
    fn format_rational() {
        let reg = empty_reg();
        let val = Value::Rational(QRat::from((3i64, 7i64)));
        assert_eq!(format_value(&val, &reg), "3/7");
    }

    #[test]
    fn format_bool_true() {
        let reg = empty_reg();
        assert_eq!(format_value(&Value::Bool(true), &reg), "true");
    }

    #[test]
    fn format_bool_false() {
        let reg = empty_reg();
        assert_eq!(format_value(&Value::Bool(false), &reg), "false");
    }

    #[test]
    fn format_none() {
        let reg = empty_reg();
        assert_eq!(format_value(&Value::None, &reg), "NONE");
    }

    #[test]
    fn format_infinity() {
        let reg = empty_reg();
        assert_eq!(format_value(&Value::Infinity, &reg), "infinity");
    }

    #[test]
    fn format_empty_list() {
        let reg = empty_reg();
        let val = Value::List(vec![]);
        assert_eq!(format_value(&val, &reg), "[]");
    }

    #[test]
    fn format_integer_list() {
        let reg = empty_reg();
        let val = Value::List(vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(2i64)),
            Value::Integer(QInt::from(3i64)),
        ]);
        assert_eq!(format_value(&val, &reg), "[1, 2, 3]");
    }

    #[test]
    fn format_matrix() {
        let reg = empty_reg();
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
        let result = format_value(&val, &reg);
        assert!(result.contains("[1, 0]"));
        assert!(result.contains("[0, 1]"));
    }

    #[test]
    fn format_dict() {
        let reg = empty_reg();
        let val = Value::Dict(vec![
            ("a".to_string(), Value::Integer(QInt::from(1i64))),
            ("b".to_string(), Value::Integer(QInt::from(2i64))),
        ]);
        assert_eq!(format_value(&val, &reg), "{a: 1, b: 2}");
    }

    #[test]
    fn format_empty_dict() {
        let reg = empty_reg();
        let val = Value::Dict(vec![]);
        assert_eq!(format_value(&val, &reg), "{}");
    }

    #[test]
    fn format_pair() {
        let reg = empty_reg();
        let val = Value::Pair(
            Box::new(Value::Integer(QInt::from(1i64))),
            Box::new(Value::Integer(QInt::from(2i64))),
        );
        assert_eq!(format_value(&val, &reg), "(1, 2)");
    }

    #[test]
    fn format_series() {
        let (reg, sym_q) = q_reg();
        let fps = FormalPowerSeries::monomial(sym_q, QRat::one(), 1, 10);
        let val = Value::Series(fps);
        let result = format_value(&val, &reg);
        assert!(result.contains("q"), "expected 'q' in: {}", result);
        assert!(result.contains("O(q^10)"), "expected 'O(q^10)' in: {}", result);
    }

    // -- Polynomial vs Series display tests --------------------------------

    #[test]
    fn format_polynomial_no_truncation() {
        let (reg, sym_q) = q_reg();
        // Create a polynomial (POLYNOMIAL_ORDER sentinel) with terms at 0, 1, 2
        let mut coeffs = std::collections::BTreeMap::new();
        coeffs.insert(0, QRat::one());
        coeffs.insert(1, QRat::from((2i64, 1i64)));
        coeffs.insert(2, QRat::one());
        let fps = FormalPowerSeries::from_coeffs(sym_q, coeffs, POLYNOMIAL_ORDER);
        let val = Value::Series(fps);
        let result = format_value(&val, &reg);
        assert_eq!(result, "1 + 2*q + q^2");
        assert!(!result.contains("O("), "polynomial should not have O(...)");
    }

    #[test]
    fn format_series_with_truncation() {
        let (reg, sym_q) = q_reg();
        let fps = FormalPowerSeries::monomial(sym_q, QRat::one(), 1, 20);
        let val = Value::Series(fps);
        let result = format_value(&val, &reg);
        assert!(result.contains("O(q^20)"), "truncated series should have O(q^20): {}", result);
    }

    #[test]
    fn format_series_variable_name() {
        let mut reg = SymbolRegistry::new();
        let sym_t = reg.intern("t");
        let fps = FormalPowerSeries::monomial(sym_t, QRat::one(), 1, 10);
        let val = Value::Series(fps);
        let result = format_value(&val, &reg);
        assert!(result.contains("t"), "expected 't' in: {}", result);
        assert!(!result.contains("q"), "should not contain 'q': {}", result);
        assert!(result.contains("O(t^10)"), "expected 'O(t^10)' in: {}", result);
    }

    #[test]
    fn format_zero_polynomial() {
        let (reg, sym_q) = q_reg();
        let fps = FormalPowerSeries::zero(sym_q, POLYNOMIAL_ORDER);
        let val = Value::Series(fps);
        let result = format_value(&val, &reg);
        assert_eq!(result, "0");
    }

    // -- format_latex tests -------------------------------------------------

    #[test]
    fn format_latex_integer() {
        let reg = empty_reg();
        let val = Value::Integer(QInt::from(42i64));
        assert_eq!(format_latex(&val, &reg), "42");
    }

    #[test]
    fn format_latex_negative_rational() {
        let reg = empty_reg();
        let val = Value::Rational(QRat::from((-3i64, 7i64)));
        assert_eq!(format_latex(&val, &reg), "-\\frac{3}{7}");
    }

    #[test]
    fn format_latex_positive_rational() {
        let reg = empty_reg();
        let val = Value::Rational(QRat::from((3i64, 7i64)));
        assert_eq!(format_latex(&val, &reg), "\\frac{3}{7}");
    }

    #[test]
    fn format_latex_series() {
        let (reg, sym_q) = q_reg();
        let fps = FormalPowerSeries::monomial(sym_q, QRat::one(), 1, 10);
        let val = Value::Series(fps);
        let result = format_latex(&val, &reg);
        assert!(result.contains("q"), "expected 'q' in: {}", result);
        assert!(result.contains("O(q^{"), "expected 'O(q^{{' in: {}", result);
    }

    #[test]
    fn format_latex_infinity() {
        let reg = empty_reg();
        assert_eq!(format_latex(&Value::Infinity, &reg), "\\infty");
    }

    #[test]
    fn format_latex_bool() {
        let reg = empty_reg();
        assert_eq!(format_latex(&Value::Bool(true), &reg), "\\text{true}");
        assert_eq!(format_latex(&Value::Bool(false), &reg), "\\text{false}");
    }

    #[test]
    fn format_latex_none() {
        let reg = empty_reg();
        assert_eq!(format_latex(&Value::None, &reg), "\\text{NONE}");
    }

    #[test]
    fn format_latex_list() {
        let reg = empty_reg();
        let val = Value::List(vec![
            Value::Integer(QInt::from(1i64)),
            Value::Integer(QInt::from(2i64)),
        ]);
        assert_eq!(format_latex(&val, &reg), "\\left[1, 2\\right]");
    }

    #[test]
    fn format_latex_empty_series() {
        let (reg, sym_q) = q_reg();
        let fps = FormalPowerSeries::zero(sym_q, 10);
        let val = Value::Series(fps);
        let result = format_latex(&val, &reg);
        assert_eq!(result, "O(q^{10})");
    }

    #[test]
    fn format_latex_polynomial_no_truncation() {
        let (reg, sym_q) = q_reg();
        let mut coeffs = std::collections::BTreeMap::new();
        coeffs.insert(0, QRat::one());
        coeffs.insert(2, QRat::one());
        let fps = FormalPowerSeries::from_coeffs(sym_q, coeffs, POLYNOMIAL_ORDER);
        let val = Value::Series(fps);
        let result = format_latex(&val, &reg);
        assert_eq!(result, "1 + q^{2}");
        assert!(!result.contains("O("), "polynomial LaTeX should not have O(...)");
    }

    #[test]
    fn format_latex_variable_name() {
        let mut reg = SymbolRegistry::new();
        let sym_t = reg.intern("t");
        let fps = FormalPowerSeries::monomial(sym_t, QRat::one(), 1, 10);
        let val = Value::Series(fps);
        let result = format_latex(&val, &reg);
        assert!(result.contains("t"), "expected 't' in: {}", result);
        assert!(!result.contains("q"), "should not contain 'q': {}", result);
    }

    // -- JacobiProduct format tests ------------------------------------------

    #[test]
    fn format_jacobi_product_single() {
        let reg = empty_reg();
        let val = Value::JacobiProduct(vec![(1, 5, 1)]);
        assert_eq!(format_value(&val, &reg), "JAC(1,5)");
    }

    #[test]
    fn format_jacobi_product_multiple() {
        let reg = empty_reg();
        let val = Value::JacobiProduct(vec![(1, 5, 1), (2, 5, 1)]);
        assert_eq!(format_value(&val, &reg), "JAC(1,5)*JAC(2,5)");
    }

    #[test]
    fn format_jacobi_product_negative_exp() {
        let reg = empty_reg();
        let val = Value::JacobiProduct(vec![(1, 5, -1)]);
        assert_eq!(format_value(&val, &reg), "JAC(1,5)^(-1)");
    }

    #[test]
    fn format_jacobi_product_empty() {
        let reg = empty_reg();
        let val = Value::JacobiProduct(vec![]);
        assert_eq!(format_value(&val, &reg), "1");
    }

    #[test]
    fn format_jacobi_product_latex_single() {
        let reg = empty_reg();
        let val = Value::JacobiProduct(vec![(1, 5, 1)]);
        assert_eq!(format_latex(&val, &reg), "(q^{1};q^{5})_{\\infty}");
    }

    #[test]
    fn format_jacobi_product_latex_multi() {
        let reg = empty_reg();
        let val = Value::JacobiProduct(vec![(1, 5, 1), (2, 5, 2)]);
        let result = format_latex(&val, &reg);
        assert_eq!(result, "(q^{1};q^{5})_{\\infty} \\cdot (q^{2};q^{5})_{\\infty}^{2}");
    }
}
