//! LaTeX rendering for all Expr variants.
//!
//! Follows DLMF 17.2 notation for q-Pochhammer and basic hypergeometric series.
//! Uses an always-brace policy for subscripts and superscripts to avoid edge-case bugs.

use crate::arena::ExprArena;
use crate::expr::{Expr, ExprRef};

/// Context in which an expression appears, used to determine parenthesization.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ParenContext {
    /// Top-level or inside a structure that provides its own delimiters.
    None,
    /// Base of a Pow expression: compound bases need parens.
    PowBase,
    /// Factor in a Mul expression: Add children need parens.
    MulFactor,
    /// Child of Neg: compound expressions need parens.
    NegChild,
}

/// Render an expression as a LaTeX string.
///
/// Recursively traverses the arena, producing valid LaTeX for all Expr variants.
/// Uses always-brace policy for subscripts/superscripts (e.g., `q^{2}` not `q^2`).
pub fn to_latex(arena: &ExprArena, expr: ExprRef) -> String {
    render(arena, expr, ParenContext::None)
}

/// Recursive rendering with context-dependent parenthesization.
fn render(arena: &ExprArena, expr: ExprRef, ctx: ParenContext) -> String {
    match arena.get(expr) {
        Expr::Integer(n) => n.0.to_string(),

        Expr::Rational(r) => {
            let numer = r.0.numer();
            let denom = r.0.denom();
            // Handle negative rationals: render as -\frac{|p|}{q}
            if numer.cmp0() == std::cmp::Ordering::Less {
                format!("-\\frac{{{}}}{{{}}}", rug::Integer::from(-numer), denom)
            } else {
                format!("\\frac{{{}}}{{{}}}", numer, denom)
            }
        }

        Expr::Symbol(id) => {
            let name = arena.symbols().name(*id);
            render_symbol_latex(name)
        }

        Expr::Infinity => "\\infty".to_string(),

        Expr::Undefined => "\\text{undefined}".to_string(),

        Expr::Add(terms) => {
            let result = render_add(arena, terms);
            if ctx == ParenContext::PowBase
                || ctx == ParenContext::MulFactor
                || ctx == ParenContext::NegChild
            {
                format!("\\left({}\\right)", result)
            } else {
                result
            }
        }

        Expr::Mul(factors) => {
            let result = render_mul(arena, factors);
            if ctx == ParenContext::PowBase {
                format!("\\left({}\\right)", result)
            } else {
                result
            }
        }

        Expr::Neg(child) => {
            let child = *child;
            if needs_parens_for_neg(arena, child) {
                format!("-\\left({}\\right)", render(arena, child, ParenContext::None))
            } else {
                format!("-{}", render(arena, child, ParenContext::NegChild))
            }
        }

        Expr::Pow(base, exp) => {
            let base = *base;
            let exp = *exp;
            let base_str = render(arena, base, ParenContext::PowBase);
            let exp_str = render(arena, exp, ParenContext::None);
            format!("{}^{{{}}}", base_str, exp_str)
        }

        Expr::QPochhammer { base, nome, order } => {
            let base = *base;
            let nome = *nome;
            let order = *order;
            let base_str = render(arena, base, ParenContext::None);
            let nome_str = render(arena, nome, ParenContext::None);
            let order_str = render(arena, order, ParenContext::None);
            format!(
                "\\left({} ; {}\\right)_{{{}}}",
                base_str, nome_str, order_str
            )
        }

        Expr::JacobiTheta { index, nome } => {
            let nome = *nome;
            let index = *index;
            let nome_str = render(arena, nome, ParenContext::None);
            format!("\\theta_{{{}}}\\!\\left({}\\right)", index, nome_str)
        }

        Expr::DedekindEta(tau) => {
            let tau = *tau;
            let tau_str = render(arena, tau, ParenContext::None);
            format!("\\eta\\!\\left({}\\right)", tau_str)
        }

        Expr::BasicHypergeometric {
            upper,
            lower,
            nome,
            argument,
        } => {
            let r = upper.len();
            let s = lower.len();
            let nome = *nome;
            let argument = *argument;
            let upper: Vec<ExprRef> = upper.iter().copied().collect();
            let lower: Vec<ExprRef> = lower.iter().copied().collect();

            let upper_strs: Vec<String> = upper
                .iter()
                .map(|e| render(arena, *e, ParenContext::None))
                .collect();
            let lower_strs: Vec<String> = lower
                .iter()
                .map(|e| render(arena, *e, ParenContext::None))
                .collect();
            let nome_str = render(arena, nome, ParenContext::None);
            let arg_str = render(arena, argument, ParenContext::None);

            format!(
                "{{}}_{{{}}}\\phi_{{{}}}\\!\\left(\\begin{{matrix}} {} \\\\ {} \\end{{matrix}} ; {}, {}\\right)",
                r,
                s,
                upper_strs.join(", "),
                lower_strs.join(", "),
                nome_str,
                arg_str,
            )
        }
    }
}

/// Render an Add expression, detecting Neg children for minus signs.
fn render_add(arena: &ExprArena, terms: &[ExprRef]) -> String {
    let mut result = String::new();
    for (i, &term) in terms.iter().enumerate() {
        if i == 0 {
            result.push_str(&render(arena, term, ParenContext::None));
        } else {
            // Check if this term is a Neg -- render as " - x" instead of " + -x"
            if let Expr::Neg(inner) = arena.get(term) {
                let inner = *inner;
                if needs_parens_for_neg(arena, inner) {
                    result.push_str(&format!(
                        " - \\left({}\\right)",
                        render(arena, inner, ParenContext::None)
                    ));
                } else {
                    result.push_str(&format!(" - {}", render(arena, inner, ParenContext::NegChild)));
                }
            } else {
                result.push_str(&format!(" + {}", render(arena, term, ParenContext::None)));
            }
        }
    }
    result
}

/// Render a Mul expression, parenthesizing Add children.
fn render_mul(arena: &ExprArena, factors: &[ExprRef]) -> String {
    let parts: Vec<String> = factors
        .iter()
        .map(|&f| render(arena, f, ParenContext::MulFactor))
        .collect();
    parts.join(" \\cdot ")
}

/// Determines if a Neg child needs parentheses.
/// Compound expressions (Add, Mul, Add) need parens; atoms do not.
fn needs_parens_for_neg(arena: &ExprArena, expr: ExprRef) -> bool {
    matches!(
        arena.get(expr),
        Expr::Add(_) | Expr::Mul(_) | Expr::Pow(_, _)
    )
}

/// Map a symbol name to its LaTeX command if it is a recognized Greek letter.
/// Returns the original name (possibly with subscript handling) otherwise.
fn render_symbol_latex(name: &str) -> String {
    // Check for subscripted names like "x_1"
    if let Some(pos) = name.find('_') {
        let (base, sub) = name.split_at(pos);
        let sub_content = &sub[1..]; // skip the underscore
        let base_latex = greek_letter(base).unwrap_or(base);
        return format!("{}_{{{}}}", base_latex, sub_content);
    }

    if let Some(cmd) = greek_letter(name) {
        cmd.to_string()
    } else {
        name.to_string()
    }
}

/// Maps a Greek letter name to its LaTeX command.
/// Handles both lowercase and uppercase variants.
fn greek_letter(name: &str) -> Option<&'static str> {
    match name {
        // Lowercase
        "alpha" => Some("\\alpha"),
        "beta" => Some("\\beta"),
        "gamma" => Some("\\gamma"),
        "delta" => Some("\\delta"),
        "epsilon" => Some("\\epsilon"),
        "zeta" => Some("\\zeta"),
        "eta" => Some("\\eta"),
        "theta" => Some("\\theta"),
        "iota" => Some("\\iota"),
        "kappa" => Some("\\kappa"),
        "lambda" => Some("\\lambda"),
        "mu" => Some("\\mu"),
        "nu" => Some("\\nu"),
        "xi" => Some("\\xi"),
        "pi" => Some("\\pi"),
        "rho" => Some("\\rho"),
        "sigma" => Some("\\sigma"),
        "tau" => Some("\\tau"),
        "upsilon" => Some("\\upsilon"),
        "phi" => Some("\\phi"),
        "chi" => Some("\\chi"),
        "psi" => Some("\\psi"),
        "omega" => Some("\\omega"),
        // Uppercase (only those that differ from Latin)
        "Gamma" => Some("\\Gamma"),
        "Delta" => Some("\\Delta"),
        "Theta" => Some("\\Theta"),
        "Lambda" => Some("\\Lambda"),
        "Xi" => Some("\\Xi"),
        "Pi" => Some("\\Pi"),
        "Sigma" => Some("\\Sigma"),
        "Upsilon" => Some("\\Upsilon"),
        "Phi" => Some("\\Phi"),
        "Psi" => Some("\\Psi"),
        "Omega" => Some("\\Omega"),
        _ => None,
    }
}
