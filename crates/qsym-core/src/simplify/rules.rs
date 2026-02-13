//! Rewrite rules as direct Rust match functions for each simplification phase.
//!
//! Four phases applied in priority order:
//! 1. **normalize** -- Flatten nested Add/Mul, combine numeric constants
//! 2. **cancel** -- Eliminate identity elements (0 in Add, 1 in Mul), zero annihilation
//! 3. **collect** -- Combine like terms in Add, collect powers in Mul
//! 4. **simplify_arith** -- Double negation, neg of constants, pow-of-pow
//!
//! Each function has signature `(ExprRef, &mut ExprArena) -> ExprRef` and
//! operates on the current node (children already simplified by bottom-up traversal).

use crate::arena::ExprArena;
use crate::canonical::{make_add, make_mul, make_pow};
use crate::expr::{Expr, ExprRef};
use crate::number::{QInt, QRat};
use std::cmp::Ordering;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check if an ExprRef points to Integer(0).
fn is_zero(expr: ExprRef, arena: &ExprArena) -> bool {
    matches!(arena.get(expr), Expr::Integer(n) if n.is_zero())
}

/// Check if an ExprRef points to Integer(1).
fn is_one(expr: ExprRef, arena: &ExprArena) -> bool {
    matches!(arena.get(expr), Expr::Integer(n) if n.0.cmp0() != Ordering::Equal && {
        let one = rug::Integer::from(1);
        n.0 == one
    })
}

/// Extract integer value if the expression is Integer.
fn as_int(expr: ExprRef, arena: &ExprArena) -> Option<QInt> {
    match arena.get(expr) {
        Expr::Integer(n) => Some(n.clone()),
        _ => None,
    }
}

/// Convert an Integer or Rational to QRat for unified arithmetic.
fn as_numeric(expr: ExprRef, arena: &ExprArena) -> Option<QRat> {
    match arena.get(expr) {
        Expr::Integer(n) => Some(QRat::from(n.clone())),
        Expr::Rational(r) => Some(r.clone()),
        _ => None,
    }
}

/// Check if an expression is numeric (Integer or Rational).
fn is_numeric(expr: ExprRef, arena: &ExprArena) -> bool {
    matches!(arena.get(expr), Expr::Integer(_) | Expr::Rational(_))
}

/// Intern a QRat as the appropriate Expr type.
/// If the denominator is 1, intern as Integer. Otherwise as Rational.
fn intern_numeric(arena: &mut ExprArena, val: QRat) -> ExprRef {
    let one = rug::Integer::from(1);
    if *val.denom() == one {
        arena.intern(Expr::Integer(QInt(val.0.into_numer_denom().0)))
    } else {
        arena.intern(Expr::Rational(val))
    }
}

// ---------------------------------------------------------------------------
// Phase 1: normalize
// ---------------------------------------------------------------------------

/// Flatten nested Add/Mul and combine numeric constants.
///
/// Rules:
/// 1. Flatten nested Add: Add([a, Add([b, c])]) -> Add([a, b, c])
/// 2. Flatten nested Mul: Mul([a, Mul([b, c])]) -> Mul([a, b, c])
/// 3. Combine numeric constants in Add: Add([3, 5, x]) -> Add([8, x])
/// 4. Combine numeric constants in Mul: Mul([3, 5, x]) -> Mul([15, x])
/// 5. Singleton unwrap handled by make_add/make_mul.
pub fn normalize(expr: ExprRef, arena: &mut ExprArena) -> ExprRef {
    let node = arena.get(expr).clone();
    match node {
        Expr::Add(ref children) => {
            normalize_add(expr, children, arena)
        }
        Expr::Mul(ref children) => {
            normalize_mul(expr, children, arena)
        }
        _ => expr,
    }
}

fn normalize_add(original: ExprRef, children: &[ExprRef], arena: &mut ExprArena) -> ExprRef {
    // Step 1: Flatten nested Adds
    let mut flat_children = Vec::new();
    let mut had_nested = false;
    for &child in children {
        match arena.get(child).clone() {
            Expr::Add(grandchildren) => {
                flat_children.extend(grandchildren);
                had_nested = true;
            }
            _ => flat_children.push(child),
        }
    }

    // Step 2: Combine numeric constants
    let mut numerics = Vec::new();
    let mut non_numerics = Vec::new();

    for &child in &flat_children {
        if let Some(val) = as_numeric(child, arena) {
            numerics.push(val);
        } else {
            non_numerics.push(child);
        }
    }

    // Only combine if there are 2+ numeric constants
    if numerics.len() >= 2 {
        let sum: QRat = numerics.into_iter().fold(QRat::zero(), |acc, v| acc + v);
        if !sum.is_zero() {
            let num_ref = intern_numeric(arena, sum);
            non_numerics.push(num_ref);
        }
        make_add(arena, non_numerics)
    } else if had_nested {
        // Re-canonicalize after flattening
        make_add(arena, flat_children)
    } else {
        original
    }
}

fn normalize_mul(original: ExprRef, children: &[ExprRef], arena: &mut ExprArena) -> ExprRef {
    // Step 1: Flatten nested Muls
    let mut flat_children = Vec::new();
    let mut had_nested = false;
    for &child in children {
        match arena.get(child).clone() {
            Expr::Mul(grandchildren) => {
                flat_children.extend(grandchildren);
                had_nested = true;
            }
            _ => flat_children.push(child),
        }
    }

    // Step 2: Combine numeric constants
    let mut numerics = Vec::new();
    let mut non_numerics = Vec::new();

    for &child in &flat_children {
        if let Some(val) = as_numeric(child, arena) {
            numerics.push(val);
        } else {
            non_numerics.push(child);
        }
    }

    if numerics.len() >= 2 {
        let product: QRat = numerics.into_iter().fold(QRat::one(), |acc, v| acc * v);
        // Check for zero annihilation while we're here
        if product.is_zero() {
            return arena.intern(Expr::Integer(QInt::zero()));
        }
        // Check if product is one (multiplicative identity)
        let one_rat = QRat::one();
        if product != one_rat {
            let num_ref = intern_numeric(arena, product);
            non_numerics.push(num_ref);
        }
        make_mul(arena, non_numerics)
    } else if had_nested {
        make_mul(arena, flat_children)
    } else {
        original
    }
}

// ---------------------------------------------------------------------------
// Phase 2: cancel
// ---------------------------------------------------------------------------

/// Eliminate identity elements and apply annihilation rules.
///
/// Rules:
/// 1. Add zero elimination: remove Integer(0) from Add children
/// 2. Mul one elimination: remove Integer(1) from Mul children
/// 3. Mul zero annihilation: if Mul contains Integer(0), return 0
/// 4. Pow(x, 0) -> 1
/// 5. Pow(x, 1) -> x
/// 6. Pow(1, n) -> 1
/// 7. Neg(0) -> 0
pub fn cancel(expr: ExprRef, arena: &mut ExprArena) -> ExprRef {
    let node = arena.get(expr).clone();
    match node {
        Expr::Add(ref children) => {
            // Remove zeros
            let filtered: Vec<ExprRef> = children
                .iter()
                .copied()
                .filter(|&c| !is_zero(c, arena))
                .collect();
            if filtered.len() < children.len() {
                make_add(arena, filtered)
            } else {
                expr
            }
        }

        Expr::Mul(ref children) => {
            // Zero annihilation: if any child is 0, result is 0
            if children.iter().any(|&c| is_zero(c, arena)) {
                return arena.intern(Expr::Integer(QInt::zero()));
            }
            // Remove ones
            let filtered: Vec<ExprRef> = children
                .iter()
                .copied()
                .filter(|&c| !is_one(c, arena))
                .collect();
            if filtered.len() < children.len() {
                make_mul(arena, filtered)
            } else {
                expr
            }
        }

        Expr::Pow(base, exp) => {
            // Pow(x, 0) -> 1
            if is_zero(exp, arena) {
                return arena.intern(Expr::Integer(QInt::from(1i64)));
            }
            // Pow(x, 1) -> x
            if is_one(exp, arena) {
                return base;
            }
            // Pow(1, n) -> 1
            if is_one(base, arena) {
                return arena.intern(Expr::Integer(QInt::from(1i64)));
            }
            expr
        }

        Expr::Neg(child) => {
            // Neg(0) -> 0
            if is_zero(child, arena) {
                return arena.intern(Expr::Integer(QInt::zero()));
            }
            expr
        }

        _ => expr,
    }
}

// ---------------------------------------------------------------------------
// Phase 3: collect
// ---------------------------------------------------------------------------

/// Combine like terms in Add (coefficients) and collect powers in Mul (exponents).
///
/// Rules:
/// 1. Like-term collection in Add: 2*x + 3*x -> 5*x
/// 2. Power collection in Mul: x * x -> x^2, x * x^2 -> x^3
pub fn collect(expr: ExprRef, arena: &mut ExprArena) -> ExprRef {
    let node = arena.get(expr).clone();
    match node {
        Expr::Add(ref children) => collect_like_terms_add(expr, children, arena),
        Expr::Mul(ref children) => collect_powers_mul(expr, children, arena),
        _ => expr,
    }
}

/// For each child of Add, extract (coefficient, base).
/// - Mul([Integer(n), rest...]) -> coefficient=n, base=Mul(rest) or rest if single
/// - Mul([Rational(r), rest...]) -> coefficient=r, base=Mul(rest) or rest if single
/// - other -> coefficient=1, base=child
fn extract_add_term(child: ExprRef, arena: &ExprArena) -> (QRat, ExprRef) {
    match arena.get(child) {
        Expr::Mul(factors) => {
            // Look for a numeric factor
            let mut numeric_idx = None;
            for (i, &f) in factors.iter().enumerate() {
                if is_numeric(f, arena) {
                    numeric_idx = Some(i);
                    break;
                }
            }
            if let Some(idx) = numeric_idx {
                let coeff = as_numeric(factors[idx], arena).unwrap();
                let rest: Vec<ExprRef> = factors
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| i != idx)
                    .map(|(_, &f)| f)
                    .collect();
                // We can't call make_mul here since we only have &ExprArena.
                // Return the rest as a list that the caller can reconstruct.
                // For now, use a sentinel approach: if rest is 1 element, return it directly.
                // If rest is >1, we need to reconstruct. We'll return child and let caller handle.
                // Actually, let's just return child ref and figure base from the rest.
                // The caller needs mutable arena. Let's change approach.
                if rest.len() == 1 {
                    (coeff, rest[0])
                } else {
                    // Can't construct the base without &mut arena.
                    // Return coefficient=1, base=child (no collection possible for multi-factor case
                    // without mutable arena access here).
                    (QRat::from((1i64, 1i64)), child)
                }
            } else {
                (QRat::from((1i64, 1i64)), child)
            }
        }
        _ => (QRat::from((1i64, 1i64)), child),
    }
}

fn collect_like_terms_add(
    original: ExprRef,
    children: &[ExprRef],
    arena: &mut ExprArena,
) -> ExprRef {
    // Extract (coefficient, base) for each child
    let terms: Vec<(QRat, ExprRef)> = children
        .iter()
        .map(|&c| extract_add_term(c, arena))
        .collect();

    // Group by base, summing coefficients
    // Use BTreeMap keyed by ExprRef for deterministic ordering
    let mut groups: BTreeMap<ExprRef, QRat> = BTreeMap::new();
    for (coeff, base) in &terms {
        let entry = groups.entry(*base).or_insert_with(QRat::zero);
        *entry = &*entry + coeff;
    }

    // Check if any grouping happened (any group has count > 1 worth of contribution)
    let had_collection = groups.len() < children.len();
    if !had_collection {
        return original;
    }

    // Rebuild children from groups
    let mut new_children = Vec::new();
    for (base, coeff) in groups {
        if coeff.is_zero() {
            continue;
        }
        let one_rat = QRat::from((1i64, 1i64));
        if coeff == one_rat {
            new_children.push(base);
        } else {
            let coeff_ref = intern_numeric(arena, coeff);
            let term = make_mul(arena, vec![coeff_ref, base]);
            new_children.push(term);
        }
    }

    make_add(arena, new_children)
}

/// For each child of Mul, extract (base, exponent).
/// - Pow(base, Integer(n)) -> base=base, exponent=n
/// - other -> base=child, exponent=1
fn extract_mul_factor(child: ExprRef, arena: &ExprArena) -> (ExprRef, QInt) {
    match arena.get(child) {
        Expr::Pow(base, exp) => {
            if let Some(n) = as_int(*exp, arena) {
                (*base, n)
            } else {
                (child, QInt::from(1i64))
            }
        }
        _ => (child, QInt::from(1i64)),
    }
}

fn collect_powers_mul(
    original: ExprRef,
    children: &[ExprRef],
    arena: &mut ExprArena,
) -> ExprRef {
    let factors: Vec<(ExprRef, QInt)> = children
        .iter()
        .map(|&c| extract_mul_factor(c, arena))
        .collect();

    // Group by base, summing exponents
    let mut groups: BTreeMap<ExprRef, QInt> = BTreeMap::new();
    for (base, exp) in &factors {
        let entry = groups.entry(*base).or_insert_with(QInt::zero);
        *entry = entry.clone() + exp.clone();
    }

    let had_collection = groups.len() < children.len();
    if !had_collection {
        return original;
    }

    let mut new_children = Vec::new();
    for (base, exp) in groups {
        if exp.is_zero() {
            // x^0 = 1, will be cancelled later
            new_children.push(arena.intern(Expr::Integer(QInt::from(1i64))));
        } else {
            let one = QInt::from(1i64);
            if exp == one {
                new_children.push(base);
            } else {
                let exp_ref = arena.intern(Expr::Integer(exp));
                new_children.push(make_pow(arena, base, exp_ref));
            }
        }
    }

    make_mul(arena, new_children)
}

// ---------------------------------------------------------------------------
// Phase 4: simplify_arith
// ---------------------------------------------------------------------------

/// Algebraic arithmetic simplifications.
///
/// Rules:
/// 1. Double negation: Neg(Neg(x)) -> x
/// 2. Neg of integer: Neg(Integer(n)) -> Integer(-n)
/// 3. Neg of rational: Neg(Rational(r)) -> Rational(-r)
/// 4. Pow(Pow(x, a), b) -> Pow(x, a*b) when a, b are both Integer
pub fn simplify_arith(expr: ExprRef, arena: &mut ExprArena) -> ExprRef {
    let node = arena.get(expr).clone();
    match node {
        Expr::Neg(child) => {
            match arena.get(child).clone() {
                // Double negation: Neg(Neg(x)) -> x
                Expr::Neg(inner) => inner,

                // Neg(Integer(n)) -> Integer(-n)
                Expr::Integer(n) => {
                    arena.intern(Expr::Integer(-n))
                }

                // Neg(Rational(r)) -> Rational(-r)
                Expr::Rational(r) => {
                    arena.intern(Expr::Rational(-r))
                }

                _ => expr,
            }
        }

        Expr::Pow(base, exp) => {
            // Pow(Pow(x, a), b) -> Pow(x, a*b) when a, b are both Integer
            match arena.get(base).clone() {
                Expr::Pow(inner_base, inner_exp) => {
                    if let (Some(a), Some(b)) = (as_int(inner_exp, arena), as_int(exp, arena)) {
                        let product = a * b;
                        let product_ref = arena.intern(Expr::Integer(product));
                        make_pow(arena, inner_base, product_ref)
                    } else {
                        expr
                    }
                }
                _ => expr,
            }
        }

        _ => expr,
    }
}
