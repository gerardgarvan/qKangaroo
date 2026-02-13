//! Phased simplification engine with guaranteed termination.
//!
//! The engine applies 4 rule phases in priority order:
//! 1. **normalize** -- Flatten nested Add/Mul, combine numeric constants
//! 2. **cancel** -- Eliminate identity elements, zero annihilation
//! 3. **collect** -- Combine like terms, collect powers
//! 4. **simplify_arith** -- Double negation, neg of constants, pow-of-pow
//!
//! If any phase changes the expression (detected via ExprRef != comparison,
//! which is O(1) thanks to hash-consing), restart from phase 1.
//! Total restarts capped at `max_iterations` for guaranteed termination.

pub mod rules;
pub mod traverse;

use crate::arena::ExprArena;
use crate::expr::ExprRef;
use traverse::bottom_up_apply;

/// The simplification engine with phased rule application and fixpoint detection.
pub struct SimplificationEngine {
    max_iterations: usize,
}

impl SimplificationEngine {
    /// Create a new engine with default max_iterations (100).
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
        }
    }

    /// Create a new engine with a custom iteration cap.
    pub fn with_max_iterations(max_iterations: usize) -> Self {
        Self { max_iterations }
    }

    /// Simplify an expression by applying all 4 phases until fixpoint or cap.
    ///
    /// The algorithm:
    /// 1. Set `current = expr`, `iterations = 0`
    /// 2. Loop while iterations < max_iterations:
    ///    a. Apply phase 1 (normalize) bottom-up. If result != current, restart.
    ///    b. Apply phase 2 (cancel) bottom-up. If changed, restart.
    ///    c. Apply phase 3 (collect) bottom-up. If changed, restart.
    ///    d. Apply phase 4 (simplify_arith) bottom-up. If changed, restart.
    ///    e. If no phase changed anything, break (fixpoint reached).
    /// 3. Return current.
    pub fn simplify(&self, expr: ExprRef, arena: &mut ExprArena) -> ExprRef {
        let mut current = expr;
        let mut iterations = 0;

        while iterations < self.max_iterations {
            // Phase 1: normalize
            let after_normalize = bottom_up_apply(current, arena, &rules::normalize);
            if after_normalize != current {
                current = after_normalize;
                iterations += 1;
                continue;
            }

            // Phase 2: cancel
            let after_cancel = bottom_up_apply(current, arena, &rules::cancel);
            if after_cancel != current {
                current = after_cancel;
                iterations += 1;
                continue;
            }

            // Phase 3: collect
            let after_collect = bottom_up_apply(current, arena, &rules::collect);
            if after_collect != current {
                current = after_collect;
                iterations += 1;
                continue;
            }

            // Phase 4: simplify_arith
            let after_arith = bottom_up_apply(current, arena, &rules::simplify_arith);
            if after_arith != current {
                current = after_arith;
                iterations += 1;
                continue;
            }

            // Fixpoint reached: no phase changed anything
            break;
        }

        current
    }
}

impl Default for SimplificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience free function: simplify with default settings.
pub fn simplify(expr: ExprRef, arena: &mut ExprArena) -> ExprRef {
    SimplificationEngine::new().simplify(expr, arena)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::{make_add, make_neg};
    use crate::expr::Expr;
    use crate::number::QInt;

    #[test]
    fn smoke_atoms_unchanged() {
        let mut arena = ExprArena::new();
        let five = arena.intern(Expr::Integer(QInt::from(5i64)));
        let result = simplify(five, &mut arena);
        assert_eq!(result, five, "Integer atom should be unchanged by simplification");
    }

    #[test]
    fn smoke_double_negation() {
        let mut arena = ExprArena::new();
        let x = arena.intern_symbol("x");
        let neg_x = make_neg(&mut arena, x);
        let neg_neg_x = make_neg(&mut arena, neg_x);
        let result = simplify(neg_neg_x, &mut arena);
        assert_eq!(result, x, "Neg(Neg(x)) should simplify to x");
    }

    #[test]
    fn smoke_add_zero_identity() {
        let mut arena = ExprArena::new();
        let zero = arena.intern(Expr::Integer(QInt::zero()));
        let x = arena.intern_symbol("x");
        let sum = make_add(&mut arena, vec![zero, x]);
        let result = simplify(sum, &mut arena);
        assert_eq!(result, x, "Add([0, x]) should simplify to x");
    }
}
