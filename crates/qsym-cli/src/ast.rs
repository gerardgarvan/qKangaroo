//! AST types for the q-Kangaroo parser.
//!
//! Represents syntax (what the user typed), not semantics (mathematical
//! structure). The evaluator converts AstNode into qsym-core Expr types.

/// Binary operator kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// Addition (`+`).
    Add,
    /// Subtraction (`-`).
    Sub,
    /// Multiplication (`*`).
    Mul,
    /// Division (`/`).
    Div,
    /// Exponentiation (`^`).
    Pow,
}

/// An AST node representing a parsed expression.
///
/// Note: AstNode does not carry span information. Parse errors are reported
/// from the parser's current position. If needed later, a `Spanned<T>` wrapper
/// can be added.
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    /// Small integer literal (fits in i64).
    Integer(i64),
    /// Large integer literal (decimal string, evaluator converts to QInt).
    BigInteger(String),
    /// The `q` indeterminate.
    Q,
    /// The `infinity` keyword.
    Infinity,
    /// `%` reference to last result.
    LastResult,
    /// Variable reference by name.
    Variable(String),
    /// Binary operation: `lhs op rhs`.
    BinOp {
        op: BinOp,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },
    /// Unary negation: `-expr`.
    Neg(Box<AstNode>),
    /// Function call: `name(arg1, arg2, ...)`.
    FuncCall {
        name: String,
        args: Vec<AstNode>,
    },
    /// List literal: `[expr1, expr2, ...]`.
    List(Vec<AstNode>),
    /// Variable assignment: `name := value`.
    Assign {
        name: String,
        value: Box<AstNode>,
    },
}

/// Statement terminator kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terminator {
    /// `;` -- print result.
    Semi,
    /// `:` -- suppress output.
    Colon,
    /// No explicit terminator (end of input).
    Implicit,
}

/// A parsed statement: an expression with a terminator.
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    /// The expression node.
    pub node: AstNode,
    /// How this statement was terminated.
    pub terminator: Terminator,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_integer_equality() {
        assert_eq!(AstNode::Integer(42), AstNode::Integer(42));
        assert_ne!(AstNode::Integer(42), AstNode::Integer(43));
    }

    #[test]
    fn ast_q_and_infinity() {
        assert_ne!(AstNode::Q, AstNode::Infinity);
        assert_eq!(AstNode::Q, AstNode::Q);
    }

    #[test]
    fn ast_binop_construction() {
        let node = AstNode::BinOp {
            op: BinOp::Add,
            lhs: Box::new(AstNode::Integer(1)),
            rhs: Box::new(AstNode::Integer(2)),
        };
        if let AstNode::BinOp { op, lhs, rhs } = &node {
            assert_eq!(*op, BinOp::Add);
            assert_eq!(**lhs, AstNode::Integer(1));
            assert_eq!(**rhs, AstNode::Integer(2));
        } else {
            panic!("Expected BinOp variant");
        }
    }

    #[test]
    fn ast_neg_construction() {
        let node = AstNode::Neg(Box::new(AstNode::Integer(5)));
        if let AstNode::Neg(inner) = &node {
            assert_eq!(**inner, AstNode::Integer(5));
        } else {
            panic!("Expected Neg variant");
        }
    }

    #[test]
    fn ast_func_call() {
        let node = AstNode::FuncCall {
            name: "aqprod".to_string(),
            args: vec![AstNode::Q, AstNode::Integer(10)],
        };
        if let AstNode::FuncCall { name, args } = &node {
            assert_eq!(name, "aqprod");
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected FuncCall variant");
        }
    }

    #[test]
    fn ast_assign() {
        let node = AstNode::Assign {
            name: "f".to_string(),
            value: Box::new(AstNode::Integer(42)),
        };
        if let AstNode::Assign { name, value } = &node {
            assert_eq!(name, "f");
            assert_eq!(**value, AstNode::Integer(42));
        } else {
            panic!("Expected Assign variant");
        }
    }

    #[test]
    fn stmt_with_terminator() {
        let stmt = Stmt {
            node: AstNode::Integer(1),
            terminator: Terminator::Semi,
        };
        assert_eq!(stmt.terminator, Terminator::Semi);

        let stmt2 = Stmt {
            node: AstNode::Q,
            terminator: Terminator::Colon,
        };
        assert_eq!(stmt2.terminator, Terminator::Colon);

        let stmt3 = Stmt {
            node: AstNode::LastResult,
            terminator: Terminator::Implicit,
        };
        assert_eq!(stmt3.terminator, Terminator::Implicit);
    }

    #[test]
    fn binop_variants() {
        // Verify all 5 BinOp variants are distinct
        let ops = [BinOp::Add, BinOp::Sub, BinOp::Mul, BinOp::Div, BinOp::Pow];
        for i in 0..ops.len() {
            for j in 0..ops.len() {
                if i == j {
                    assert_eq!(ops[i], ops[j]);
                } else {
                    assert_ne!(ops[i], ops[j]);
                }
            }
        }
    }
}
