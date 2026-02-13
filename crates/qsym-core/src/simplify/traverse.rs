//! Bottom-up traversal of ExprArena DAG with recursive child simplification.
//!
//! The `bottom_up_apply` function recursively simplifies children of an
//! expression (bottom-up), then applies a rule function to the result.
//! Hash-consing gives O(1) change detection via ExprRef comparison.

use crate::arena::ExprArena;
use crate::canonical::{make_add, make_mul, make_neg, make_pow};
use crate::expr::{Expr, ExprRef};
use smallvec::SmallVec;

/// Apply `rule_fn` bottom-up over the expression DAG.
///
/// 1. Clone the Expr from the arena (avoids borrow conflict).
/// 2. Recursively apply to all children based on variant.
/// 3. If any child changed, reconstruct the node via canonical constructors.
/// 4. Apply `rule_fn` to the (possibly reconstructed) node.
/// 5. Return the final ExprRef.
pub fn bottom_up_apply(
    expr: ExprRef,
    arena: &mut ExprArena,
    rule_fn: &dyn Fn(ExprRef, &mut ExprArena) -> ExprRef,
) -> ExprRef {
    let node = arena.get(expr).clone();

    let with_simplified_children = match node {
        Expr::Add(children) => {
            let new_children: Vec<ExprRef> = children
                .iter()
                .map(|&c| bottom_up_apply(c, arena, rule_fn))
                .collect();
            if new_children == children {
                expr
            } else {
                make_add(arena, new_children)
            }
        }

        Expr::Mul(children) => {
            let new_children: Vec<ExprRef> = children
                .iter()
                .map(|&c| bottom_up_apply(c, arena, rule_fn))
                .collect();
            if new_children == children {
                expr
            } else {
                make_mul(arena, new_children)
            }
        }

        Expr::Neg(child) => {
            let new_child = bottom_up_apply(child, arena, rule_fn);
            if new_child == child {
                expr
            } else {
                make_neg(arena, new_child)
            }
        }

        Expr::Pow(base, exp) => {
            let new_base = bottom_up_apply(base, arena, rule_fn);
            let new_exp = bottom_up_apply(exp, arena, rule_fn);
            if new_base == base && new_exp == exp {
                expr
            } else {
                make_pow(arena, new_base, new_exp)
            }
        }

        Expr::QPochhammer { base, nome, order } => {
            let new_base = bottom_up_apply(base, arena, rule_fn);
            let new_nome = bottom_up_apply(nome, arena, rule_fn);
            let new_order = bottom_up_apply(order, arena, rule_fn);
            if new_base == base && new_nome == nome && new_order == order {
                expr
            } else {
                arena.intern(Expr::QPochhammer {
                    base: new_base,
                    nome: new_nome,
                    order: new_order,
                })
            }
        }

        Expr::JacobiTheta { index, nome } => {
            let new_nome = bottom_up_apply(nome, arena, rule_fn);
            if new_nome == nome {
                expr
            } else {
                arena.intern(Expr::JacobiTheta {
                    index,
                    nome: new_nome,
                })
            }
        }

        Expr::DedekindEta(tau) => {
            let new_tau = bottom_up_apply(tau, arena, rule_fn);
            if new_tau == tau {
                expr
            } else {
                arena.intern(Expr::DedekindEta(new_tau))
            }
        }

        Expr::BasicHypergeometric {
            upper,
            lower,
            nome,
            argument,
        } => {
            let new_upper: SmallVec<[ExprRef; 4]> = upper
                .iter()
                .map(|&u| bottom_up_apply(u, arena, rule_fn))
                .collect();
            let new_lower: SmallVec<[ExprRef; 4]> = lower
                .iter()
                .map(|&l| bottom_up_apply(l, arena, rule_fn))
                .collect();
            let new_nome = bottom_up_apply(nome, arena, rule_fn);
            let new_argument = bottom_up_apply(argument, arena, rule_fn);
            if new_upper.as_slice() == upper.as_slice()
                && new_lower.as_slice() == lower.as_slice()
                && new_nome == nome
                && new_argument == argument
            {
                expr
            } else {
                arena.intern(Expr::BasicHypergeometric {
                    upper: new_upper,
                    lower: new_lower,
                    nome: new_nome,
                    argument: new_argument,
                })
            }
        }

        // Atoms: no children to recurse into
        Expr::Integer(_) | Expr::Rational(_) | Expr::Symbol(_) | Expr::Infinity | Expr::Undefined => {
            expr
        }
    };

    // Apply the rule function to the node with simplified children
    rule_fn(with_simplified_children, arena)
}
