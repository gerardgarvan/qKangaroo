//! Unicode terminal rendering for expressions.
//!
//! Placeholder for Plan 01-03. Will implement `fmt::Display` for terminal rendering
//! with Greek characters and subscript/superscript digits.

use crate::arena::ExprArena;
use crate::expr::ExprRef;
use std::fmt;

/// Wrapper type for displaying an expression in Unicode format.
pub struct DisplayExpr<'a> {
    pub arena: &'a ExprArena,
    pub expr: ExprRef,
}

impl<'a> fmt::Display for DisplayExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Placeholder -- full implementation in Plan 01-03
        write!(f, "<expr:{}>", self.expr)
    }
}
