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

/// Comparison operator kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompOp {
    /// `=` equality.
    Eq,
    /// `<>` inequality.
    NotEq,
    /// `<` less than.
    Less,
    /// `>` greater than.
    Greater,
    /// `<=` less than or equal.
    LessEq,
    /// `>=` greater than or equal.
    GreaterEq,
}

/// Boolean binary operator kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolBinOp {
    /// `and` conjunction.
    And,
    /// `or` disjunction.
    Or,
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
    /// The `infinity` keyword.
    Infinity,
    /// String literal value.
    StringLit(String),
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
    /// Comparison expression: `lhs op rhs`.
    Compare {
        op: CompOp,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },
    /// Boolean NOT: `not expr`.
    Not(Box<AstNode>),
    /// Boolean binary: `lhs and/or rhs`.
    BoolOp {
        op: BoolBinOp,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },
    /// For loop: `for var from start to end [by step] do body od`.
    ForLoop {
        var: String,
        from: Box<AstNode>,
        to: Box<AstNode>,
        by: Option<Box<AstNode>>,
        body: Vec<Stmt>,
    },
    /// Conditional: `if cond then body [elif cond then body]* [else body] fi`.
    IfExpr {
        condition: Box<AstNode>,
        then_body: Vec<Stmt>,
        elif_branches: Vec<(AstNode, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
    },
    /// Procedure definition: `proc(params) [local vars;] [option opts;] body; end [proc]`.
    ProcDef {
        params: Vec<String>,
        locals: Vec<String>,
        options: Vec<String>,
        body: Vec<Stmt>,
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
        assert_ne!(AstNode::Variable("q".to_string()), AstNode::Infinity);
        assert_eq!(
            AstNode::Variable("q".to_string()),
            AstNode::Variable("q".to_string())
        );
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
            args: vec![AstNode::Variable("q".to_string()), AstNode::Integer(10)],
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
            node: AstNode::Variable("q".to_string()),
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
    fn ast_compare_construction() {
        let node = AstNode::Compare {
            op: CompOp::Less,
            lhs: Box::new(AstNode::Variable("x".to_string())),
            rhs: Box::new(AstNode::Integer(5)),
        };
        if let AstNode::Compare { op, lhs, rhs } = &node {
            assert_eq!(*op, CompOp::Less);
            assert_eq!(**lhs, AstNode::Variable("x".to_string()));
            assert_eq!(**rhs, AstNode::Integer(5));
        } else {
            panic!("Expected Compare variant");
        }
    }

    #[test]
    fn ast_boolop_construction() {
        let node = AstNode::BoolOp {
            op: BoolBinOp::And,
            lhs: Box::new(AstNode::Variable("a".to_string())),
            rhs: Box::new(AstNode::Variable("b".to_string())),
        };
        if let AstNode::BoolOp { op, lhs, rhs } = &node {
            assert_eq!(*op, BoolBinOp::And);
            assert_eq!(**lhs, AstNode::Variable("a".to_string()));
            assert_eq!(**rhs, AstNode::Variable("b".to_string()));
        } else {
            panic!("Expected BoolOp variant");
        }
    }

    #[test]
    fn ast_not_construction() {
        let node = AstNode::Not(Box::new(AstNode::Variable("x".to_string())));
        if let AstNode::Not(inner) = &node {
            assert_eq!(**inner, AstNode::Variable("x".to_string()));
        } else {
            panic!("Expected Not variant");
        }
    }

    #[test]
    fn ast_for_loop_construction() {
        let node = AstNode::ForLoop {
            var: "n".to_string(),
            from: Box::new(AstNode::Integer(1)),
            to: Box::new(AstNode::Integer(5)),
            by: None,
            body: vec![Stmt {
                node: AstNode::Variable("n".to_string()),
                terminator: Terminator::Implicit,
            }],
        };
        if let AstNode::ForLoop { var, from, to, by, body } = &node {
            assert_eq!(var, "n");
            assert_eq!(**from, AstNode::Integer(1));
            assert_eq!(**to, AstNode::Integer(5));
            assert!(by.is_none());
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected ForLoop variant");
        }
    }

    #[test]
    fn ast_if_expr_construction() {
        let node = AstNode::IfExpr {
            condition: Box::new(AstNode::Compare {
                op: CompOp::Eq,
                lhs: Box::new(AstNode::Variable("x".to_string())),
                rhs: Box::new(AstNode::Integer(0)),
            }),
            then_body: vec![Stmt {
                node: AstNode::Integer(1),
                terminator: Terminator::Implicit,
            }],
            elif_branches: vec![],
            else_body: Some(vec![Stmt {
                node: AstNode::Integer(2),
                terminator: Terminator::Implicit,
            }]),
        };
        if let AstNode::IfExpr { condition, then_body, elif_branches, else_body } = &node {
            assert!(matches!(**condition, AstNode::Compare { .. }));
            assert_eq!(then_body.len(), 1);
            assert!(elif_branches.is_empty());
            assert!(else_body.is_some());
        } else {
            panic!("Expected IfExpr variant");
        }
    }

    #[test]
    fn compop_variants() {
        let ops = [CompOp::Eq, CompOp::NotEq, CompOp::Less, CompOp::Greater, CompOp::LessEq, CompOp::GreaterEq];
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
