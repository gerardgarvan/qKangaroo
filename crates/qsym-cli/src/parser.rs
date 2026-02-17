//! Pratt parser for the q-Kangaroo Maple-style grammar.
//!
//! Converts a token stream (produced by [`crate::lexer::tokenize`]) into an AST
//! of [`AstNode`] values wrapped in [`Stmt`] statements. Implements operator
//! precedence via a top-down operator-precedence (Pratt) parser.

use crate::ast::{AstNode, BinOp, Stmt, Terminator};
use crate::error::ParseError;
use crate::lexer::tokenize;
use crate::token::{Span, SpannedToken, Token};

/// Parse a source string into a list of statements.
///
/// This is the main entry point. It tokenizes the input, then parses the
/// token stream into statements separated by `;` (print) or `:` (suppress).
///
/// # Errors
///
/// Returns [`ParseError`] on lexer or parser errors with byte-offset spans.
pub fn parse(input: &str) -> Result<Vec<Stmt>, ParseError> {
    let tokens = tokenize(input)?;
    let mut parser = Parser {
        tokens,
        pos: 0,
        source: input.to_string(),
    };
    parser.parse_line()
}

/// Internal parser state holding the token stream and current position.
struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    #[allow(dead_code)]
    source: String,
}

impl Parser {
    /// Peek at the current token without consuming it.
    fn peek(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos].token
        } else {
            &Token::Eof
        }
    }

    /// Get the span of the current token.
    fn peek_span(&self) -> Span {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].span
        } else {
            let end = self.source.len();
            Span::new(end, end)
        }
    }

    /// Consume and return the current token.
    fn advance(&mut self) -> SpannedToken {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            tok
        } else {
            SpannedToken {
                token: Token::Eof,
                span: Span::new(self.source.len(), self.source.len()),
            }
        }
    }

    /// Consume the current token if it matches `expected`, otherwise return an error.
    fn expect(&mut self, expected: &Token, context: &str) -> Result<SpannedToken, ParseError> {
        if self.peek() == expected {
            Ok(self.advance())
        } else {
            let span = self.peek_span();
            let found = token_name(self.peek());
            Err(ParseError::new(
                format!("expected {}, found {}", context, found),
                span,
            ))
        }
    }

    /// True if we've reached the end of input.
    fn at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    /// Parse a line of input into zero or more statements.
    fn parse_line(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();

        loop {
            // Skip consecutive semicolons/colons (empty statements like `;;`)
            while matches!(self.peek(), Token::Semi | Token::Colon) {
                self.advance();
            }

            if self.at_end() {
                break;
            }

            // Parse expression
            let node = self.expr_bp(0)?;

            // Determine terminator
            let terminator = match self.peek() {
                Token::Semi => {
                    self.advance();
                    Terminator::Semi
                }
                Token::Colon => {
                    self.advance();
                    Terminator::Colon
                }
                Token::Eof => Terminator::Implicit,
                _ => {
                    let span = self.peek_span();
                    let found = token_name(self.peek());
                    return Err(ParseError::new(
                        format!("expected ';', ':', or end of input, found {}", found),
                        span,
                    ));
                }
            };

            stmts.push(Stmt { node, terminator });
        }

        Ok(stmts)
    }

    /// Pratt parser: parse an expression with minimum binding power `min_bp`.
    fn expr_bp(&mut self, min_bp: u8) -> Result<AstNode, ParseError> {
        // --- Prefix / NUD ---
        let mut lhs = match self.peek().clone() {
            Token::Integer(n) => {
                self.advance();
                AstNode::Integer(n)
            }
            Token::BigInteger(ref s) => {
                let s = s.clone();
                self.advance();
                AstNode::BigInteger(s)
            }
            Token::Q => {
                self.advance();
                AstNode::Q
            }
            Token::Infinity => {
                self.advance();
                AstNode::Infinity
            }
            Token::Percent => {
                self.advance();
                AstNode::LastResult
            }
            Token::Ident(ref name) => {
                let name = name.clone();
                self.advance();
                AstNode::Variable(name)
            }
            Token::Minus => {
                self.advance();
                let rhs = self.expr_bp(7)?; // prefix minus r_bp = 7
                AstNode::Neg(Box::new(rhs))
            }
            Token::LParen => {
                self.advance();
                let inner = self.expr_bp(0)?;
                self.expect(&Token::RParen, "')' to close grouping")?;
                inner
            }
            _ => {
                let span = self.peek_span();
                let found = token_name(self.peek());
                return Err(ParseError::new(
                    format!("expected expression, found {}", found),
                    span,
                ));
            }
        };

        // --- Infix / LED loop ---
        loop {
            // Function call (postfix): l_bp = 11
            if *self.peek() == Token::LParen {
                if 11 < min_bp {
                    break;
                }
                // Only variables can be called as functions
                if let AstNode::Variable(name) = lhs {
                    self.advance(); // consume LParen
                    let args = self.parse_arg_list()?;
                    self.expect(&Token::RParen, "')' to close function call")?;
                    lhs = AstNode::FuncCall { name, args };
                    continue;
                } else {
                    // Not a function call -- could be something like `5(...)`, just break
                    break;
                }
            }

            // Assignment: l_bp = 2, r_bp = 1
            if *self.peek() == Token::Assign {
                if 2 < min_bp {
                    break;
                }
                // Left side must be a variable
                if let AstNode::Variable(name) = lhs {
                    self.advance(); // consume :=
                    let value = self.expr_bp(1)?;
                    lhs = AstNode::Assign {
                        name,
                        value: Box::new(value),
                    };
                    continue;
                } else {
                    let span = self.peek_span();
                    return Err(ParseError::new(
                        "left side of := must be a variable name".to_string(),
                        span,
                    ));
                }
            }

            // Infix binary operators
            if let Some((l_bp, r_bp)) = infix_bp(self.peek()) {
                if l_bp < min_bp {
                    break;
                }

                let op_token = self.peek().clone();
                self.advance();
                let rhs = self.expr_bp(r_bp)?;

                let op = match op_token {
                    Token::Plus => BinOp::Add,
                    Token::Minus => BinOp::Sub,
                    Token::Star => BinOp::Mul,
                    Token::Slash => BinOp::Div,
                    Token::Caret => BinOp::Pow,
                    _ => unreachable!(),
                };

                lhs = AstNode::BinOp {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };

                // Non-associative exponentiation: if next token is also ^, error
                if op == BinOp::Pow && *self.peek() == Token::Caret {
                    let span = self.peek_span();
                    return Err(ParseError::new(
                        "ambiguous exponentiation: use parentheses, e.g., (a^b)^c or a^(b^c)"
                            .to_string(),
                        span,
                    ));
                }

                continue;
            }

            // No matching operator -- break the loop
            break;
        }

        Ok(lhs)
    }

    /// Parse a comma-separated list of arguments (for function calls).
    fn parse_arg_list(&mut self) -> Result<Vec<AstNode>, ParseError> {
        // Zero-arg call: f()
        if *self.peek() == Token::RParen {
            return Ok(Vec::new());
        }

        let mut args = vec![self.expr_bp(0)?];

        while *self.peek() == Token::Comma {
            self.advance(); // consume comma
            args.push(self.expr_bp(0)?);
        }

        Ok(args)
    }
}

/// Return the left and right binding powers for infix operators.
fn infix_bp(token: &Token) -> Option<(u8, u8)> {
    match token {
        Token::Plus | Token::Minus => Some((3, 4)),
        Token::Star | Token::Slash => Some((5, 6)),
        Token::Caret => Some((9, 10)),
        _ => None,
    }
}

/// Human-readable name for a token (used in error messages).
fn token_name(token: &Token) -> String {
    match token {
        Token::Integer(n) => format!("integer '{}'", n),
        Token::BigInteger(s) => format!("integer '{}'", s),
        Token::Infinity => "'infinity'".to_string(),
        Token::Q => "'q'".to_string(),
        Token::Ident(name) => format!("identifier '{}'", name),
        Token::Plus => "'+'".to_string(),
        Token::Minus => "'-'".to_string(),
        Token::Star => "'*'".to_string(),
        Token::Slash => "'/'".to_string(),
        Token::Caret => "'^'".to_string(),
        Token::Assign => "':='".to_string(),
        Token::Percent => "'%'".to_string(),
        Token::LParen => "'('".to_string(),
        Token::RParen => "')'".to_string(),
        Token::Comma => "','".to_string(),
        Token::Semi => "';'".to_string(),
        Token::Colon => "':'".to_string(),
        Token::Eof => "end of input".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: parse and return the single statement's AstNode.
    fn parse_expr(input: &str) -> AstNode {
        let stmts = parse(input).unwrap();
        assert_eq!(stmts.len(), 1, "expected exactly 1 statement, got {}", stmts.len());
        stmts.into_iter().next().unwrap().node
    }

    // =======================================================
    // PARSE-01: Function calls
    // =======================================================

    #[test]
    fn test_simple_function_call() {
        let stmts = parse("aqprod(q,q,infinity,20)").unwrap();
        assert_eq!(stmts.len(), 1);
        let stmt = &stmts[0];
        assert_eq!(stmt.terminator, Terminator::Implicit);
        assert_eq!(
            stmt.node,
            AstNode::FuncCall {
                name: "aqprod".to_string(),
                args: vec![AstNode::Q, AstNode::Q, AstNode::Infinity, AstNode::Integer(20)],
            }
        );
    }

    #[test]
    fn test_zero_arg_function() {
        let node = parse_expr("f()");
        assert_eq!(
            node,
            AstNode::FuncCall {
                name: "f".to_string(),
                args: vec![],
            }
        );
    }

    #[test]
    fn test_nested_function_call() {
        let node = parse_expr("f(g(1))");
        assert_eq!(
            node,
            AstNode::FuncCall {
                name: "f".to_string(),
                args: vec![AstNode::FuncCall {
                    name: "g".to_string(),
                    args: vec![AstNode::Integer(1)],
                }],
            }
        );
    }

    #[test]
    fn test_partition_count() {
        let node = parse_expr("partition_count(50)");
        assert_eq!(
            node,
            AstNode::FuncCall {
                name: "partition_count".to_string(),
                args: vec![AstNode::Integer(50)],
            }
        );
    }

    // =======================================================
    // PARSE-02: Assignment
    // =======================================================

    #[test]
    fn test_simple_assignment() {
        let node = parse_expr("f := 5");
        assert_eq!(
            node,
            AstNode::Assign {
                name: "f".to_string(),
                value: Box::new(AstNode::Integer(5)),
            }
        );
    }

    #[test]
    fn test_assignment_with_function() {
        let node = parse_expr("f := etaq(1,1,20)");
        assert_eq!(
            node,
            AstNode::Assign {
                name: "f".to_string(),
                value: Box::new(AstNode::FuncCall {
                    name: "etaq".to_string(),
                    args: vec![AstNode::Integer(1), AstNode::Integer(1), AstNode::Integer(20)],
                }),
            }
        );
    }

    #[test]
    fn test_assignment_prints() {
        let stmts = parse("f := 5;").unwrap();
        assert_eq!(stmts.len(), 1);
        assert_eq!(stmts[0].terminator, Terminator::Semi);
    }

    #[test]
    fn test_assignment_suppresses() {
        let stmts = parse("f := 5:").unwrap();
        assert_eq!(stmts.len(), 1);
        assert_eq!(stmts[0].terminator, Terminator::Colon);
    }

    #[test]
    fn test_assign_non_variable_error() {
        let err = parse("3 := 5").unwrap_err();
        assert!(
            err.message.contains("left side of := must be a variable name"),
            "got: {}",
            err.message
        );
    }

    // =======================================================
    // PARSE-03: Arithmetic
    // =======================================================

    #[test]
    fn test_addition() {
        let node = parse_expr("f + g");
        assert_eq!(
            node,
            AstNode::BinOp {
                op: BinOp::Add,
                lhs: Box::new(AstNode::Variable("f".to_string())),
                rhs: Box::new(AstNode::Variable("g".to_string())),
            }
        );
    }

    #[test]
    fn test_subtraction() {
        let node = parse_expr("f - g");
        assert_eq!(
            node,
            AstNode::BinOp {
                op: BinOp::Sub,
                lhs: Box::new(AstNode::Variable("f".to_string())),
                rhs: Box::new(AstNode::Variable("g".to_string())),
            }
        );
    }

    #[test]
    fn test_multiplication() {
        let node = parse_expr("f * g");
        assert_eq!(
            node,
            AstNode::BinOp {
                op: BinOp::Mul,
                lhs: Box::new(AstNode::Variable("f".to_string())),
                rhs: Box::new(AstNode::Variable("g".to_string())),
            }
        );
    }

    #[test]
    fn test_division() {
        let node = parse_expr("3 / 4");
        assert_eq!(
            node,
            AstNode::BinOp {
                op: BinOp::Div,
                lhs: Box::new(AstNode::Integer(3)),
                rhs: Box::new(AstNode::Integer(4)),
            }
        );
    }

    #[test]
    fn test_unary_negation() {
        let node = parse_expr("-f");
        assert_eq!(
            node,
            AstNode::Neg(Box::new(AstNode::Variable("f".to_string())))
        );
    }

    #[test]
    fn test_scalar_mul() {
        let node = parse_expr("3*f");
        assert_eq!(
            node,
            AstNode::BinOp {
                op: BinOp::Mul,
                lhs: Box::new(AstNode::Integer(3)),
                rhs: Box::new(AstNode::Variable("f".to_string())),
            }
        );
    }

    #[test]
    fn test_precedence_add_mul() {
        // a + b * c -> Add(a, Mul(b, c))
        let node = parse_expr("a + b * c");
        assert_eq!(
            node,
            AstNode::BinOp {
                op: BinOp::Add,
                lhs: Box::new(AstNode::Variable("a".to_string())),
                rhs: Box::new(AstNode::BinOp {
                    op: BinOp::Mul,
                    lhs: Box::new(AstNode::Variable("b".to_string())),
                    rhs: Box::new(AstNode::Variable("c".to_string())),
                }),
            }
        );
    }

    #[test]
    fn test_precedence_neg_mul() {
        // -3*f -> Mul(Neg(3), f) because unary minus (bp=7) > mul (bp=5,6)
        let node = parse_expr("-3*f");
        assert_eq!(
            node,
            AstNode::BinOp {
                op: BinOp::Mul,
                lhs: Box::new(AstNode::Neg(Box::new(AstNode::Integer(3)))),
                rhs: Box::new(AstNode::Variable("f".to_string())),
            }
        );
    }

    #[test]
    fn test_exponentiation() {
        let node = parse_expr("q^5");
        assert_eq!(
            node,
            AstNode::BinOp {
                op: BinOp::Pow,
                lhs: Box::new(AstNode::Q),
                rhs: Box::new(AstNode::Integer(5)),
            }
        );
    }

    #[test]
    fn test_non_assoc_caret() {
        let err = parse("2^3^4").unwrap_err();
        assert!(
            err.message.contains("ambiguous exponentiation"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn test_parens_override() {
        // (a + b) * c -> Mul(Add(a, b), c)
        let node = parse_expr("(a + b) * c");
        assert_eq!(
            node,
            AstNode::BinOp {
                op: BinOp::Mul,
                lhs: Box::new(AstNode::BinOp {
                    op: BinOp::Add,
                    lhs: Box::new(AstNode::Variable("a".to_string())),
                    rhs: Box::new(AstNode::Variable("b".to_string())),
                }),
                rhs: Box::new(AstNode::Variable("c".to_string())),
            }
        );
    }

    #[test]
    fn test_complex_expr() {
        // 3 * aqprod(q,q,infinity,20) + 1
        // -> Add(Mul(3, FuncCall), 1)
        let node = parse_expr("3 * aqprod(q,q,infinity,20) + 1");
        assert_eq!(
            node,
            AstNode::BinOp {
                op: BinOp::Add,
                lhs: Box::new(AstNode::BinOp {
                    op: BinOp::Mul,
                    lhs: Box::new(AstNode::Integer(3)),
                    rhs: Box::new(AstNode::FuncCall {
                        name: "aqprod".to_string(),
                        args: vec![AstNode::Q, AstNode::Q, AstNode::Infinity, AstNode::Integer(20)],
                    }),
                }),
                rhs: Box::new(AstNode::Integer(1)),
            }
        );
    }

    // =======================================================
    // PARSE-04: Literals and keywords
    // =======================================================

    #[test]
    fn test_integer_literal() {
        let node = parse_expr("50");
        assert_eq!(node, AstNode::Integer(50));
    }

    #[test]
    fn test_big_integer_literal() {
        let node = parse_expr("99999999999999999999999");
        assert_eq!(
            node,
            AstNode::BigInteger("99999999999999999999999".to_string())
        );
    }

    #[test]
    fn test_infinity() {
        let node = parse_expr("infinity");
        assert_eq!(node, AstNode::Infinity);
    }

    #[test]
    fn test_q_keyword() {
        let node = parse_expr("q");
        assert_eq!(node, AstNode::Q);
    }

    #[test]
    fn test_percent_last_result() {
        let node = parse_expr("%");
        assert_eq!(node, AstNode::LastResult);
    }

    #[test]
    fn test_percent_in_expr() {
        let node = parse_expr("% + 1");
        assert_eq!(
            node,
            AstNode::BinOp {
                op: BinOp::Add,
                lhs: Box::new(AstNode::LastResult),
                rhs: Box::new(AstNode::Integer(1)),
            }
        );
    }

    // =======================================================
    // Statement chaining
    // =======================================================

    #[test]
    fn test_multi_statement_semi() {
        let stmts = parse("f := 5; g := 10").unwrap();
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0].terminator, Terminator::Semi);
        assert_eq!(stmts[1].terminator, Terminator::Implicit);
        assert_eq!(
            stmts[0].node,
            AstNode::Assign {
                name: "f".to_string(),
                value: Box::new(AstNode::Integer(5)),
            }
        );
        assert_eq!(
            stmts[1].node,
            AstNode::Assign {
                name: "g".to_string(),
                value: Box::new(AstNode::Integer(10)),
            }
        );
    }

    #[test]
    fn test_multi_statement_mixed() {
        let stmts = parse("f := etaq(1,1,20); g := etaq(2,1,20); f * g").unwrap();
        assert_eq!(stmts.len(), 3);
        assert_eq!(stmts[0].terminator, Terminator::Semi);
        assert_eq!(stmts[1].terminator, Terminator::Semi);
        assert_eq!(stmts[2].terminator, Terminator::Implicit);
        // Third statement is f * g
        assert_eq!(
            stmts[2].node,
            AstNode::BinOp {
                op: BinOp::Mul,
                lhs: Box::new(AstNode::Variable("f".to_string())),
                rhs: Box::new(AstNode::Variable("g".to_string())),
            }
        );
    }

    #[test]
    fn test_colon_suppresses() {
        let stmts = parse("f := 5: g := 10;").unwrap();
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0].terminator, Terminator::Colon);
        assert_eq!(stmts[1].terminator, Terminator::Semi);
    }

    #[test]
    fn test_double_semicolon() {
        let stmts = parse("f := 5;; g := 10").unwrap();
        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn test_trailing_semicolon() {
        let stmts = parse("f := 5;").unwrap();
        assert_eq!(stmts.len(), 1);
        assert_eq!(stmts[0].terminator, Terminator::Semi);
    }

    #[test]
    fn test_empty_input() {
        let stmts = parse("").unwrap();
        assert_eq!(stmts.len(), 0);
    }

    // =======================================================
    // Error cases
    // =======================================================

    #[test]
    fn test_error_unexpected_token() {
        let err = parse(")").unwrap_err();
        assert!(
            err.message.contains("expected expression"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn test_error_missing_rparen() {
        let err = parse("f(1,2").unwrap_err();
        assert!(
            err.message.contains("')'"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn test_error_has_span() {
        // "f @ g" has unknown char at position 2
        let err = parse("f @ g").unwrap_err();
        assert_eq!(err.span.start, 2);
    }
}
