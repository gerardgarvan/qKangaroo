//! Unicode terminal rendering for all Expr variants.
//!
//! Uses Greek Unicode characters, subscript/superscript digits, and
//! ASCII fallback for expressions that cannot be fully represented in Unicode.

use crate::arena::ExprArena;
use crate::expr::{Expr, ExprRef};
use std::fmt;

/// A display wrapper that renders an expression using Unicode characters.
///
/// Implements `fmt::Display`, so it integrates with `format!`, `println!`, etc.
///
/// # Example
///
/// ```ignore
/// let arena = ExprArena::new();
/// // ... intern expressions ...
/// println!("{}", arena.display(expr));
/// ```
pub struct DisplayExpr<'a> {
    /// The arena containing all expression data.
    pub arena: &'a ExprArena,
    /// The expression to render.
    pub expr: ExprRef,
}

impl ExprArena {
    /// Create a `DisplayExpr` for Unicode terminal rendering.
    pub fn display(&self, expr: ExprRef) -> DisplayExpr<'_> {
        DisplayExpr { arena: self, expr }
    }
}

/// Context for parenthesization decisions.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctx {
    None,
    PowBase,
    MulFactor,
    NegChild,
}

impl<'a> fmt::Display for DisplayExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_expr(f, self.arena, self.expr, Ctx::None)
    }
}

/// Recursive expression writer.
fn write_expr(
    f: &mut fmt::Formatter<'_>,
    arena: &ExprArena,
    expr: ExprRef,
    ctx: Ctx,
) -> fmt::Result {
    match arena.get(expr) {
        Expr::Integer(n) => write!(f, "{}", n.0),

        Expr::Rational(r) => write!(f, "{}/{}", r.0.numer(), r.0.denom()),

        Expr::Symbol(id) => {
            let name = arena.symbols().name(*id);
            if let Some(ch) = greek_to_unicode(name) {
                write!(f, "{}", ch)
            } else {
                write!(f, "{}", name)
            }
        }

        Expr::Infinity => write!(f, "\u{221e}"),

        Expr::Undefined => write!(f, "undefined"),

        Expr::Add(terms) => {
            let need_parens =
                ctx == Ctx::PowBase || ctx == Ctx::MulFactor || ctx == Ctx::NegChild;
            if need_parens {
                write!(f, "(")?;
            }
            write_add(f, arena, terms)?;
            if need_parens {
                write!(f, ")")?;
            }
            Ok(())
        }

        Expr::Mul(factors) => {
            let need_parens = ctx == Ctx::PowBase;
            if need_parens {
                write!(f, "(")?;
            }
            write_mul(f, arena, factors)?;
            if need_parens {
                write!(f, ")")?;
            }
            Ok(())
        }

        Expr::Neg(child) => {
            let child = *child;
            if needs_parens_for_neg(arena, child) {
                write!(f, "-(")?;
                write_expr(f, arena, child, Ctx::None)?;
                write!(f, ")")
            } else {
                write!(f, "-")?;
                write_expr(f, arena, child, Ctx::NegChild)
            }
        }

        Expr::Pow(base, exp) => {
            let base = *base;
            let exp = *exp;
            write_expr(f, arena, base, Ctx::PowBase)?;

            // Try Unicode superscript for numeric exponents
            if let Some(sup) = try_unicode_superscript(arena, exp) {
                write!(f, "{}", sup)
            } else {
                // Fallback: ^exp or ^(expr) for compound
                if is_compound(arena, exp) {
                    write!(f, "^(")?;
                    write_expr(f, arena, exp, Ctx::None)?;
                    write!(f, ")")
                } else {
                    write!(f, "^")?;
                    write_expr(f, arena, exp, Ctx::None)
                }
            }
        }

        Expr::QPochhammer { base, nome, order } => {
            let base = *base;
            let nome = *nome;
            let order = *order;
            write!(f, "(")?;
            write_expr(f, arena, base, Ctx::None)?;
            write!(f, ";")?;
            write_expr(f, arena, nome, Ctx::None)?;
            write!(f, ")")?;
            // Render order as subscript
            write_subscript_expr(f, arena, order)
        }

        Expr::JacobiTheta { index, nome } => {
            let nome = *nome;
            let index = *index;
            write!(f, "\u{03b8}")?; // theta
            write!(f, "{}", unicode_subscript(index as u64))?;
            write!(f, "(")?;
            write_expr(f, arena, nome, Ctx::None)?;
            write!(f, ")")
        }

        Expr::DedekindEta(tau) => {
            let tau = *tau;
            write!(f, "\u{03b7}(")?; // eta
            write_expr(f, arena, tau, Ctx::None)?;
            write!(f, ")")
        }

        Expr::BasicHypergeometric {
            upper,
            lower,
            nome,
            argument,
        } => {
            let r = upper.len() as u64;
            let s = lower.len() as u64;
            let nome = *nome;
            let argument = *argument;
            let upper: Vec<ExprRef> = upper.iter().copied().collect();
            let lower: Vec<ExprRef> = lower.iter().copied().collect();

            // _r phi _s (...)
            write!(f, "{}\u{03c6}{}", unicode_subscript(r), unicode_subscript(s))?;
            write!(f, "(")?;
            // upper params
            for (i, &u) in upper.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write_expr(f, arena, u, Ctx::None)?;
            }
            write!(f, ";")?;
            // lower params
            for (i, &l) in lower.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write_expr(f, arena, l, Ctx::None)?;
            }
            write!(f, ";")?;
            write_expr(f, arena, nome, Ctx::None)?;
            write!(f, ",")?;
            write_expr(f, arena, argument, Ctx::None)?;
            write!(f, ")")
        }
    }
}

/// Write an Add expression, detecting Neg children for minus signs.
fn write_add(
    f: &mut fmt::Formatter<'_>,
    arena: &ExprArena,
    terms: &[ExprRef],
) -> fmt::Result {
    for (i, &term) in terms.iter().enumerate() {
        if i == 0 {
            write_expr(f, arena, term, Ctx::None)?;
        } else if let Expr::Neg(inner) = arena.get(term) {
            let inner = *inner;
            if needs_parens_for_neg(arena, inner) {
                write!(f, " - (")?;
                write_expr(f, arena, inner, Ctx::None)?;
                write!(f, ")")?;
            } else {
                write!(f, " - ")?;
                write_expr(f, arena, inner, Ctx::NegChild)?;
            }
        } else {
            write!(f, " + ")?;
            write_expr(f, arena, term, Ctx::None)?;
        }
    }
    Ok(())
}

/// Write a Mul expression, parenthesizing Add children.
fn write_mul(
    f: &mut fmt::Formatter<'_>,
    arena: &ExprArena,
    factors: &[ExprRef],
) -> fmt::Result {
    for (i, &factor) in factors.iter().enumerate() {
        if i > 0 {
            write!(f, "*")?;
        }
        write_expr(f, arena, factor, Ctx::MulFactor)?;
    }
    Ok(())
}

/// Whether an expression needs parens when negated.
fn needs_parens_for_neg(arena: &ExprArena, expr: ExprRef) -> bool {
    matches!(
        arena.get(expr),
        Expr::Add(_) | Expr::Mul(_) | Expr::Pow(_, _)
    )
}

/// Whether an expression is compound (non-atomic).
fn is_compound(arena: &ExprArena, expr: ExprRef) -> bool {
    matches!(
        arena.get(expr),
        Expr::Add(_) | Expr::Mul(_) | Expr::Neg(_) | Expr::Pow(_, _)
    )
}

/// Try to render a numeric exponent as Unicode superscript characters.
/// Returns None if the exponent is not a simple integer.
fn try_unicode_superscript(arena: &ExprArena, expr: ExprRef) -> Option<String> {
    match arena.get(expr) {
        Expr::Integer(n) => unicode_superscript_int(n.0.to_i64()?),
        Expr::Neg(inner) => {
            // Check if it's Neg(Integer(n)) -- render as superscript minus + digits
            if let Expr::Integer(n) = arena.get(*inner) {
                let val = n.0.to_i64()?;
                // val should be positive since it's inside Neg
                let mut result = String::new();
                result.push('\u{207b}'); // superscript minus
                for ch in val.to_string().chars() {
                    result.push(superscript_digit(ch)?);
                }
                Some(result)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Convert an integer (possibly negative) to Unicode superscript string.
fn unicode_superscript_int(n: i64) -> Option<String> {
    let mut result = String::new();
    if n < 0 {
        result.push('\u{207b}'); // superscript minus
        for ch in (-n).to_string().chars() {
            result.push(superscript_digit(ch)?);
        }
    } else {
        for ch in n.to_string().chars() {
            result.push(superscript_digit(ch)?);
        }
    }
    Some(result)
}

/// Map a digit character to its Unicode superscript equivalent.
fn superscript_digit(ch: char) -> Option<char> {
    match ch {
        '0' => Some('\u{2070}'),
        '1' => Some('\u{00b9}'),
        '2' => Some('\u{00b2}'),
        '3' => Some('\u{00b3}'),
        '4' => Some('\u{2074}'),
        '5' => Some('\u{2075}'),
        '6' => Some('\u{2076}'),
        '7' => Some('\u{2077}'),
        '8' => Some('\u{2078}'),
        '9' => Some('\u{2079}'),
        _ => None,
    }
}

/// Convert a non-negative integer to Unicode subscript digit characters.
pub fn unicode_subscript(n: u64) -> String {
    if n == 0 {
        return "\u{2080}".to_string();
    }
    let mut result = String::new();
    for ch in n.to_string().chars() {
        result.push(subscript_digit(ch));
    }
    result
}

/// Map a digit character to its Unicode subscript equivalent.
fn subscript_digit(ch: char) -> char {
    match ch {
        '0' => '\u{2080}',
        '1' => '\u{2081}',
        '2' => '\u{2082}',
        '3' => '\u{2083}',
        '4' => '\u{2084}',
        '5' => '\u{2085}',
        '6' => '\u{2086}',
        '7' => '\u{2087}',
        '8' => '\u{2088}',
        '9' => '\u{2089}',
        _ => ch, // fallback
    }
}

/// Write subscript rendering for a q-Pochhammer order expression.
/// Uses Unicode subscript digits for integers, infinity symbol for Infinity,
/// and ASCII fallback for symbolic orders.
fn write_subscript_expr(
    f: &mut fmt::Formatter<'_>,
    arena: &ExprArena,
    expr: ExprRef,
) -> fmt::Result {
    match arena.get(expr) {
        Expr::Integer(n) => {
            // Try to convert to u64 for subscript rendering
            if let Some(val) = n.0.to_u64() {
                write!(f, "{}", unicode_subscript(val))
            } else if let Some(val) = n.0.to_i64() {
                // Negative integer: use subscript minus + digits
                if val < 0 {
                    write!(f, "\u{208b}")?; // subscript minus
                    write!(f, "{}", unicode_subscript((-val) as u64))
                } else {
                    write!(f, "{}", unicode_subscript(val as u64))
                }
            } else {
                // Very large integer -- ASCII fallback
                write!(f, "_{}", n.0)
            }
        }
        Expr::Infinity => write!(f, "\u{221e}"),
        _ => {
            // Symbolic order: ASCII fallback
            write!(f, "_")?;
            write_expr(f, arena, expr, Ctx::None)
        }
    }
}

/// Map a Greek letter name to its Unicode character.
pub fn greek_to_unicode(name: &str) -> Option<char> {
    match name {
        // Lowercase
        "alpha" => Some('\u{03b1}'),
        "beta" => Some('\u{03b2}'),
        "gamma" => Some('\u{03b3}'),
        "delta" => Some('\u{03b4}'),
        "epsilon" => Some('\u{03b5}'),
        "zeta" => Some('\u{03b6}'),
        "eta" => Some('\u{03b7}'),
        "theta" => Some('\u{03b8}'),
        "iota" => Some('\u{03b9}'),
        "kappa" => Some('\u{03ba}'),
        "lambda" => Some('\u{03bb}'),
        "mu" => Some('\u{03bc}'),
        "nu" => Some('\u{03bd}'),
        "xi" => Some('\u{03be}'),
        "pi" => Some('\u{03c0}'),
        "rho" => Some('\u{03c1}'),
        "sigma" => Some('\u{03c3}'),
        "tau" => Some('\u{03c4}'),
        "upsilon" => Some('\u{03c5}'),
        "phi" => Some('\u{03c6}'),
        "chi" => Some('\u{03c7}'),
        "psi" => Some('\u{03c8}'),
        "omega" => Some('\u{03c9}'),
        // Uppercase
        "Gamma" => Some('\u{0393}'),
        "Delta" => Some('\u{0394}'),
        "Theta" => Some('\u{0398}'),
        "Lambda" => Some('\u{039b}'),
        "Xi" => Some('\u{039e}'),
        "Pi" => Some('\u{03a0}'),
        "Sigma" => Some('\u{03a3}'),
        "Upsilon" => Some('\u{03a5}'),
        "Phi" => Some('\u{03a6}'),
        "Psi" => Some('\u{03a8}'),
        "Omega" => Some('\u{03a9}'),
        _ => None,
    }
}
