//! Pratt parser for the q-Kangaroo Maple-style grammar.
//!
//! Converts a token stream (produced by [`crate::lexer::tokenize`]) into an AST
//! of [`AstNode`] values wrapped in [`Stmt`] statements. Implements operator
//! precedence via a top-down operator-precedence (Pratt) parser.

use crate::ast::{AstNode, BinOp, BoolBinOp, CompOp, Stmt, Terminator};
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
            Token::Infinity => {
                self.advance();
                AstNode::Infinity
            }
            Token::Percent => {
                self.advance();
                AstNode::LastResult
            }
            Token::Ditto => {
                self.advance();
                AstNode::LastResult
            }
            Token::StringLit(ref s) => {
                let s = s.clone();
                self.advance();
                AstNode::StringLit(s)
            }
            Token::Ident(ref name) => {
                let name = name.clone();
                self.advance();
                AstNode::Variable(name)
            }
            Token::Minus => {
                self.advance();
                let rhs = self.expr_bp(15)?; // prefix minus r_bp = 15
                AstNode::Neg(Box::new(rhs))
            }
            Token::Not => {
                self.advance();
                let rhs = self.expr_bp(7)?; // prefix not r_bp = 7
                AstNode::Not(Box::new(rhs))
            }
            Token::LParen => {
                self.advance();
                let inner = self.expr_bp(0)?;
                self.expect(&Token::RParen, "')' to close grouping")?;
                inner
            }
            Token::LBracket => {
                self.advance();
                // Parse comma-separated list items (reuse arg_list pattern)
                let items = if *self.peek() == Token::RBracket {
                    Vec::new()
                } else {
                    let mut items = vec![self.expr_bp(0)?];
                    while *self.peek() == Token::Comma {
                        self.advance(); // consume comma
                        items.push(self.expr_bp(0)?);
                    }
                    items
                };
                self.expect(&Token::RBracket, "']' to close list")?;
                AstNode::List(items)
            }
            Token::For => {
                self.advance(); // consume 'for'
                // Variable name (required)
                let var_name = match self.peek().clone() {
                    Token::Ident(name) => { self.advance(); name }
                    _ => {
                        let span = self.peek_span();
                        return Err(ParseError::new(
                            "expected variable name after 'for'".to_string(),
                            span,
                        ));
                    }
                };
                // Optional 'from' clause (default: Integer(1))
                let from_expr = if *self.peek() == Token::From {
                    self.advance(); // consume 'from'
                    self.expr_bp(0)?
                } else {
                    AstNode::Integer(1)
                };
                // Required 'to' clause
                self.expect(&Token::To, "'to' in for loop")?;
                let to_expr = self.expr_bp(0)?;
                // Optional 'by' clause
                let by_expr = if *self.peek() == Token::By {
                    self.advance(); // consume 'by'
                    Some(Box::new(self.expr_bp(0)?))
                } else {
                    None
                };
                // Required 'do' keyword
                self.expect(&Token::Do, "'do' in for loop")?;
                // Parse body statements until 'od'
                let body = self.parse_stmt_sequence(&[Token::Od])?;
                self.expect(&Token::Od, "'od' to close for loop")?;
                AstNode::ForLoop {
                    var: var_name,
                    from: Box::new(from_expr),
                    to: Box::new(to_expr),
                    by: by_expr,
                    body,
                }
            }
            Token::Proc => {
                self.advance(); // consume 'proc'
                self.expect(&Token::LParen, "'(' after 'proc'")?;
                let params = self.parse_ident_list()?;
                self.expect(&Token::RParen, "')' to close proc parameters")?;

                // Optional: local and option declarations (either order, any combination)
                let mut locals = vec![];
                let mut options = vec![];
                loop {
                    if *self.peek() == Token::Local {
                        self.advance();
                        locals.extend(self.parse_ident_list()?);
                        self.expect(&Token::Semi, "';' after local declarations")?;
                    } else if *self.peek() == Token::OptionKw {
                        self.advance();
                        options.extend(self.parse_ident_list()?);
                        self.expect(&Token::Semi, "';' after option declarations")?;
                    } else {
                        break;
                    }
                }

                // Body statements until 'end'
                let body = self.parse_stmt_sequence(&[Token::End])?;
                self.expect(&Token::End, "'end' to close proc")?;

                // Optional 'proc' after 'end' (Maple allows "end proc")
                if *self.peek() == Token::Proc {
                    self.advance();
                }

                AstNode::ProcDef { params, locals, options, body }
            }
            Token::If => {
                self.advance(); // consume 'if'
                let condition = self.expr_bp(0)?;
                self.expect(&Token::Then, "'then' after if condition")?;
                let then_body = self.parse_stmt_sequence(&[Token::Elif, Token::Else, Token::Fi])?;
                let mut elif_branches = Vec::new();
                while *self.peek() == Token::Elif {
                    self.advance(); // consume 'elif'
                    let elif_cond = self.expr_bp(0)?;
                    self.expect(&Token::Then, "'then' after elif condition")?;
                    let elif_body = self.parse_stmt_sequence(&[Token::Elif, Token::Else, Token::Fi])?;
                    elif_branches.push((elif_cond, elif_body));
                }
                let else_body = if *self.peek() == Token::Else {
                    self.advance(); // consume 'else'
                    Some(self.parse_stmt_sequence(&[Token::Fi])?)
                } else {
                    None
                };
                self.expect(&Token::Fi, "'fi' to close if expression")?;
                AstNode::IfExpr {
                    condition: Box::new(condition),
                    then_body,
                    elif_branches,
                    else_body,
                }
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
            // Function call (postfix): l_bp = 19
            if *self.peek() == Token::LParen {
                if 19 < min_bp {
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

                // Build the appropriate AST node based on operator type
                match &op_token {
                    Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Caret => {
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
                        // Non-associative exponentiation
                        if op == BinOp::Pow && *self.peek() == Token::Caret {
                            let span = self.peek_span();
                            return Err(ParseError::new(
                                "ambiguous exponentiation: use parentheses, e.g., (a^b)^c or a^(b^c)"
                                    .to_string(),
                                span,
                            ));
                        }
                    }
                    Token::Equal | Token::NotEqual | Token::Less | Token::Greater
                    | Token::LessEq | Token::GreaterEq => {
                        let op = match op_token {
                            Token::Equal => CompOp::Eq,
                            Token::NotEqual => CompOp::NotEq,
                            Token::Less => CompOp::Less,
                            Token::Greater => CompOp::Greater,
                            Token::LessEq => CompOp::LessEq,
                            Token::GreaterEq => CompOp::GreaterEq,
                            _ => unreachable!(),
                        };
                        lhs = AstNode::Compare {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        };
                        // Non-associative comparisons
                        if matches!(self.peek(),
                            Token::Equal | Token::NotEqual | Token::Less | Token::Greater
                            | Token::LessEq | Token::GreaterEq)
                        {
                            let span = self.peek_span();
                            return Err(ParseError::new(
                                "comparison operators are non-associative: use parentheses".to_string(),
                                span,
                            ));
                        }
                    }
                    Token::And | Token::Or => {
                        let op = match op_token {
                            Token::And => BoolBinOp::And,
                            Token::Or => BoolBinOp::Or,
                            _ => unreachable!(),
                        };
                        lhs = AstNode::BoolOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        };
                    }
                    _ => unreachable!(),
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

    /// Parse a comma-separated list of identifiers.
    ///
    /// Returns empty vec if next token is not an Ident. Stops when the next
    /// token after an identifier is not a Comma (does NOT consume terminator).
    fn parse_ident_list(&mut self) -> Result<Vec<String>, ParseError> {
        let mut names = Vec::new();
        if let Token::Ident(ref name) = self.peek().clone() {
            names.push(name.clone());
            self.advance();
            while *self.peek() == Token::Comma {
                self.advance(); // consume comma
                match self.peek().clone() {
                    Token::Ident(ref name) => {
                        names.push(name.clone());
                        self.advance();
                    }
                    _ => {
                        let span = self.peek_span();
                        return Err(ParseError::new(
                            "expected identifier after ','".to_string(),
                            span,
                        ));
                    }
                }
            }
        }
        Ok(names)
    }

    /// Parse a sequence of statements until a terminator token is seen.
    /// The terminating token is NOT consumed.
    fn parse_stmt_sequence(&mut self, terminators: &[Token]) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();

        loop {
            // Skip consecutive semicolons/colons (empty statements)
            while matches!(self.peek(), Token::Semi | Token::Colon) {
                self.advance();
            }

            // Stop if we see a terminator or EOF
            if terminators.iter().any(|t| t == self.peek()) || self.at_end() {
                break;
            }

            // Parse expression
            let node = self.expr_bp(0)?;

            // Determine terminator
            let terminator = if matches!(self.peek(), Token::Semi) {
                self.advance();
                Terminator::Semi
            } else if matches!(self.peek(), Token::Colon) {
                self.advance();
                Terminator::Colon
            } else {
                // No explicit terminator -- implicit (last stmt before block end)
                Terminator::Implicit
            };

            stmts.push(Stmt { node, terminator });
        }

        Ok(stmts)
    }
}

/// Return the left and right binding powers for infix operators.
fn infix_bp(token: &Token) -> Option<(u8, u8)> {
    match token {
        Token::Or => Some((3, 4)),
        Token::And => Some((5, 6)),
        Token::Equal | Token::NotEqual | Token::Less | Token::Greater
        | Token::LessEq | Token::GreaterEq => Some((9, 10)),
        Token::Plus | Token::Minus => Some((11, 12)),
        Token::Star | Token::Slash => Some((13, 14)),
        Token::Caret => Some((17, 18)),
        _ => None,
    }
}

/// Human-readable name for a token (used in error messages).
fn token_name(token: &Token) -> String {
    match token {
        Token::Integer(n) => format!("integer '{}'", n),
        Token::BigInteger(s) => format!("integer '{}'", s),
        Token::Infinity => "'infinity'".to_string(),
        Token::Ident(name) => format!("identifier '{}'", name),
        Token::Plus => "'+'".to_string(),
        Token::Minus => "'-'".to_string(),
        Token::Star => "'*'".to_string(),
        Token::Slash => "'/'".to_string(),
        Token::Caret => "'^'".to_string(),
        Token::Assign => "':='".to_string(),
        Token::Percent => "'%'".to_string(),
        Token::Ditto => "'\"' (ditto)".to_string(),
        Token::LParen => "'('".to_string(),
        Token::RParen => "')'".to_string(),
        Token::LBracket => "'['".to_string(),
        Token::RBracket => "']'".to_string(),
        Token::Comma => "','".to_string(),
        Token::Semi => "';'".to_string(),
        Token::Colon => "':'".to_string(),
        Token::StringLit(s) => format!("string \"{}\"", s),
        Token::For => "'for'".to_string(),
        Token::From => "'from'".to_string(),
        Token::To => "'to'".to_string(),
        Token::By => "'by'".to_string(),
        Token::Do => "'do'".to_string(),
        Token::Od => "'od'".to_string(),
        Token::While => "'while'".to_string(),
        Token::If => "'if'".to_string(),
        Token::Then => "'then'".to_string(),
        Token::Elif => "'elif'".to_string(),
        Token::Else => "'else'".to_string(),
        Token::Fi => "'fi'".to_string(),
        Token::End => "'end'".to_string(),
        Token::Proc => "'proc'".to_string(),
        Token::Local => "'local'".to_string(),
        Token::OptionKw => "'option'".to_string(),
        Token::And => "'and'".to_string(),
        Token::Or => "'or'".to_string(),
        Token::Not => "'not'".to_string(),
        Token::Equal => "'='".to_string(),
        Token::NotEqual => "'<>'".to_string(),
        Token::Less => "'<'".to_string(),
        Token::Greater => "'>'".to_string(),
        Token::LessEq => "'<='".to_string(),
        Token::GreaterEq => "'>='".to_string(),
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
                args: vec![AstNode::Variable("q".to_string()), AstNode::Variable("q".to_string()), AstNode::Infinity, AstNode::Integer(20)],
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
                lhs: Box::new(AstNode::Variable("q".to_string())),
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
                        args: vec![AstNode::Variable("q".to_string()), AstNode::Variable("q".to_string()), AstNode::Infinity, AstNode::Integer(20)],
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
        assert_eq!(node, AstNode::Variable("q".to_string()));
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

    // =======================================================
    // PARSE-05: List literals
    // =======================================================

    #[test]
    fn test_list_integers() {
        let node = parse_expr("[1, 2, 3]");
        assert_eq!(
            node,
            AstNode::List(vec![
                AstNode::Integer(1),
                AstNode::Integer(2),
                AstNode::Integer(3),
            ])
        );
    }

    #[test]
    fn test_list_variables() {
        let node = parse_expr("[f, g]");
        assert_eq!(
            node,
            AstNode::List(vec![
                AstNode::Variable("f".to_string()),
                AstNode::Variable("g".to_string()),
            ])
        );
    }

    #[test]
    fn test_empty_list() {
        let node = parse_expr("[]");
        assert_eq!(node, AstNode::List(vec![]));
    }

    #[test]
    fn test_nested_list() {
        let node = parse_expr("[[1, 2], [3, 4]]");
        assert_eq!(
            node,
            AstNode::List(vec![
                AstNode::List(vec![AstNode::Integer(1), AstNode::Integer(2)]),
                AstNode::List(vec![AstNode::Integer(3), AstNode::Integer(4)]),
            ])
        );
    }

    #[test]
    fn test_list_as_function_arg() {
        let node = parse_expr("findlincombo(target, [f, g], 20)");
        assert_eq!(
            node,
            AstNode::FuncCall {
                name: "findlincombo".to_string(),
                args: vec![
                    AstNode::Variable("target".to_string()),
                    AstNode::List(vec![
                        AstNode::Variable("f".to_string()),
                        AstNode::Variable("g".to_string()),
                    ]),
                    AstNode::Integer(20),
                ],
            }
        );
    }

    #[test]
    fn test_list_single_element() {
        let node = parse_expr("[42]");
        assert_eq!(node, AstNode::List(vec![AstNode::Integer(42)]));
    }

    #[test]
    fn test_list_with_expressions() {
        let node = parse_expr("[1 + 2, 3 * 4]");
        assert_eq!(
            node,
            AstNode::List(vec![
                AstNode::BinOp {
                    op: BinOp::Add,
                    lhs: Box::new(AstNode::Integer(1)),
                    rhs: Box::new(AstNode::Integer(2)),
                },
                AstNode::BinOp {
                    op: BinOp::Mul,
                    lhs: Box::new(AstNode::Integer(3)),
                    rhs: Box::new(AstNode::Integer(4)),
                },
            ])
        );
    }

    #[test]
    fn test_list_missing_rbracket() {
        let err = parse("[1, 2").unwrap_err();
        assert!(
            err.message.contains("']'"),
            "got: {}",
            err.message
        );
    }

    // =======================================================
    // PARSE-06: String literals
    // =======================================================

    #[test]
    fn test_string_literal_parse() {
        let node = parse_expr(r#""file.qk""#);
        assert_eq!(node, AstNode::StringLit("file.qk".to_string()));
    }

    #[test]
    fn test_string_in_function_call() {
        let node = parse_expr(r#"read("file.qk")"#);
        assert_eq!(
            node,
            AstNode::FuncCall {
                name: "read".to_string(),
                args: vec![AstNode::StringLit("file.qk".to_string())],
            }
        );
    }

    // =======================================================
    // PARSE-07: Comparison operators
    // =======================================================

    #[test]
    fn test_compare_less() {
        let node = parse_expr("x < 5");
        assert_eq!(
            node,
            AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::Variable("x".to_string())),
                rhs: Box::new(AstNode::Integer(5)),
            }
        );
    }

    #[test]
    fn test_compare_not_equal() {
        let node = parse_expr("x <> 0");
        assert_eq!(
            node,
            AstNode::Compare {
                op: CompOp::NotEq,
                lhs: Box::new(AstNode::Variable("x".to_string())),
                rhs: Box::new(AstNode::Integer(0)),
            }
        );
    }

    #[test]
    fn test_compare_equal() {
        let node = parse_expr("x = 5");
        assert_eq!(
            node,
            AstNode::Compare {
                op: CompOp::Eq,
                lhs: Box::new(AstNode::Variable("x".to_string())),
                rhs: Box::new(AstNode::Integer(5)),
            }
        );
    }

    #[test]
    fn test_compare_less_equal() {
        let node = parse_expr("x <= 5");
        assert_eq!(
            node,
            AstNode::Compare {
                op: CompOp::LessEq,
                lhs: Box::new(AstNode::Variable("x".to_string())),
                rhs: Box::new(AstNode::Integer(5)),
            }
        );
    }

    #[test]
    fn test_compare_greater() {
        let node = parse_expr("x > 5");
        assert_eq!(
            node,
            AstNode::Compare {
                op: CompOp::Greater,
                lhs: Box::new(AstNode::Variable("x".to_string())),
                rhs: Box::new(AstNode::Integer(5)),
            }
        );
    }

    #[test]
    fn test_compare_greater_equal() {
        let node = parse_expr("x >= 5");
        assert_eq!(
            node,
            AstNode::Compare {
                op: CompOp::GreaterEq,
                lhs: Box::new(AstNode::Variable("x".to_string())),
                rhs: Box::new(AstNode::Integer(5)),
            }
        );
    }

    #[test]
    fn test_compare_non_associative() {
        let err = parse("a < b < c").unwrap_err();
        assert!(
            err.message.contains("non-associative"),
            "got: {}",
            err.message
        );
    }

    // =======================================================
    // PARSE-08: Boolean operators
    // =======================================================

    #[test]
    fn test_bool_and() {
        let node = parse_expr("a and b");
        assert_eq!(
            node,
            AstNode::BoolOp {
                op: BoolBinOp::And,
                lhs: Box::new(AstNode::Variable("a".to_string())),
                rhs: Box::new(AstNode::Variable("b".to_string())),
            }
        );
    }

    #[test]
    fn test_bool_or() {
        let node = parse_expr("a or b");
        assert_eq!(
            node,
            AstNode::BoolOp {
                op: BoolBinOp::Or,
                lhs: Box::new(AstNode::Variable("a".to_string())),
                rhs: Box::new(AstNode::Variable("b".to_string())),
            }
        );
    }

    #[test]
    fn test_bool_not() {
        let node = parse_expr("not x");
        assert_eq!(
            node,
            AstNode::Not(Box::new(AstNode::Variable("x".to_string())))
        );
    }

    // =======================================================
    // PARSE-09: Precedence with new operators
    // =======================================================

    #[test]
    fn test_precedence_compare_and_or() {
        // a > 0 and b < 10 or c = 5
        // => Or(And(Greater(a,0), Less(b,10)), Eq(c,5))
        let node = parse_expr("a > 0 and b < 10 or c = 5");
        assert_eq!(
            node,
            AstNode::BoolOp {
                op: BoolBinOp::Or,
                lhs: Box::new(AstNode::BoolOp {
                    op: BoolBinOp::And,
                    lhs: Box::new(AstNode::Compare {
                        op: CompOp::Greater,
                        lhs: Box::new(AstNode::Variable("a".to_string())),
                        rhs: Box::new(AstNode::Integer(0)),
                    }),
                    rhs: Box::new(AstNode::Compare {
                        op: CompOp::Less,
                        lhs: Box::new(AstNode::Variable("b".to_string())),
                        rhs: Box::new(AstNode::Integer(10)),
                    }),
                }),
                rhs: Box::new(AstNode::Compare {
                    op: CompOp::Eq,
                    lhs: Box::new(AstNode::Variable("c".to_string())),
                    rhs: Box::new(AstNode::Integer(5)),
                }),
            }
        );
    }

    #[test]
    fn test_precedence_not_binds_looser_than_compare() {
        // not x > 5 => Not(Compare(Greater, x, 5))
        let node = parse_expr("not x > 5");
        assert_eq!(
            node,
            AstNode::Not(Box::new(AstNode::Compare {
                op: CompOp::Greater,
                lhs: Box::new(AstNode::Variable("x".to_string())),
                rhs: Box::new(AstNode::Integer(5)),
            }))
        );
    }

    #[test]
    fn test_precedence_arithmetic_tighter_than_compare() {
        // a + b < c * d => Compare(Less, Add(a,b), Mul(c,d))
        let node = parse_expr("a + b < c * d");
        assert_eq!(
            node,
            AstNode::Compare {
                op: CompOp::Less,
                lhs: Box::new(AstNode::BinOp {
                    op: BinOp::Add,
                    lhs: Box::new(AstNode::Variable("a".to_string())),
                    rhs: Box::new(AstNode::Variable("b".to_string())),
                }),
                rhs: Box::new(AstNode::BinOp {
                    op: BinOp::Mul,
                    lhs: Box::new(AstNode::Variable("c".to_string())),
                    rhs: Box::new(AstNode::Variable("d".to_string())),
                }),
            }
        );
    }

    #[test]
    fn test_assignment_still_works() {
        let node = parse_expr("x := 5");
        assert_eq!(
            node,
            AstNode::Assign {
                name: "x".to_string(),
                value: Box::new(AstNode::Integer(5)),
            }
        );
    }

    // =======================================================
    // PARSE-10: For-loop parsing
    // =======================================================

    #[test]
    fn test_for_basic() {
        let node = parse_expr("for n from 1 to 5 do n od");
        assert_eq!(
            node,
            AstNode::ForLoop {
                var: "n".to_string(),
                from: Box::new(AstNode::Integer(1)),
                to: Box::new(AstNode::Integer(5)),
                by: None,
                body: vec![Stmt {
                    node: AstNode::Variable("n".to_string()),
                    terminator: Terminator::Implicit,
                }],
            }
        );
    }

    #[test]
    fn test_for_default_from() {
        let node = parse_expr("for n to 10 do n od");
        if let AstNode::ForLoop { from, to, .. } = &node {
            assert_eq!(**from, AstNode::Integer(1));
            assert_eq!(**to, AstNode::Integer(10));
        } else {
            panic!("Expected ForLoop, got {:?}", node);
        }
    }

    #[test]
    fn test_for_with_by() {
        let node = parse_expr("for k from 0 to 8 by 2 do k od");
        if let AstNode::ForLoop { var, by, .. } = &node {
            assert_eq!(var, "k");
            assert_eq!(*by, Some(Box::new(AstNode::Integer(2))));
        } else {
            panic!("Expected ForLoop");
        }
    }

    #[test]
    fn test_for_multi_stmt_body() {
        let node = parse_expr("for n from 1 to 3 do x := n; x od");
        if let AstNode::ForLoop { body, .. } = &node {
            assert_eq!(body.len(), 2);
            assert_eq!(body[0].terminator, Terminator::Semi);
            assert_eq!(body[1].terminator, Terminator::Implicit);
        } else {
            panic!("Expected ForLoop");
        }
    }

    #[test]
    fn test_for_body_colon_suppress() {
        let node = parse_expr("for n from 1 to 3 do x := n: x od");
        if let AstNode::ForLoop { body, .. } = &node {
            assert_eq!(body.len(), 2);
            assert_eq!(body[0].terminator, Terminator::Colon);
        } else {
            panic!("Expected ForLoop");
        }
    }

    #[test]
    fn test_for_body_func_call() {
        let node = parse_expr("for n from 1 to 5 do print(n) od");
        if let AstNode::ForLoop { body, .. } = &node {
            assert_eq!(body.len(), 1);
            assert!(matches!(&body[0].node, AstNode::FuncCall { .. }));
        } else {
            panic!("Expected ForLoop");
        }
    }

    #[test]
    fn test_for_error_no_var() {
        let err = parse("for 3 from 1 to 5 do x od").unwrap_err();
        assert!(err.message.contains("expected variable name after 'for'"), "got: {}", err.message);
    }

    #[test]
    fn test_for_error_no_to() {
        let err = parse("for n from 1 do n od").unwrap_err();
        assert!(err.message.contains("'to'"), "got: {}", err.message);
    }

    #[test]
    fn test_for_error_no_do() {
        let err = parse("for n from 1 to 5 n od").unwrap_err();
        assert!(err.message.contains("'do'"), "got: {}", err.message);
    }

    #[test]
    fn test_for_error_no_od() {
        let err = parse("for n from 1 to 5 do n").unwrap_err();
        assert!(err.message.contains("'od'"), "got: {}", err.message);
    }

    // =======================================================
    // PARSE-11: If/elif/else/fi parsing
    // =======================================================

    #[test]
    fn test_if_simple() {
        let node = parse_expr("if x > 0 then 1 fi");
        if let AstNode::IfExpr { condition, then_body, elif_branches, else_body } = &node {
            assert!(matches!(**condition, AstNode::Compare { op: CompOp::Greater, .. }));
            assert_eq!(then_body.len(), 1);
            assert!(elif_branches.is_empty());
            assert!(else_body.is_none());
        } else {
            panic!("Expected IfExpr");
        }
    }

    #[test]
    fn test_if_else() {
        let node = parse_expr("if x > 0 then 1 else 0 fi");
        if let AstNode::IfExpr { else_body, .. } = &node {
            assert!(else_body.is_some());
            assert_eq!(else_body.as_ref().unwrap().len(), 1);
        } else {
            panic!("Expected IfExpr");
        }
    }

    #[test]
    fn test_if_elif_else() {
        let node = parse_expr("if x > 0 then 1 elif x = 0 then 0 else -1 fi");
        if let AstNode::IfExpr { elif_branches, else_body, .. } = &node {
            assert_eq!(elif_branches.len(), 1);
            assert!(else_body.is_some());
        } else {
            panic!("Expected IfExpr");
        }
    }

    #[test]
    fn test_if_multiple_elif() {
        let node = parse_expr("if a then 1 elif b then 2 elif c then 3 fi");
        if let AstNode::IfExpr { elif_branches, else_body, .. } = &node {
            assert_eq!(elif_branches.len(), 2);
            assert!(else_body.is_none());
        } else {
            panic!("Expected IfExpr");
        }
    }

    #[test]
    fn test_if_multi_stmt_bodies() {
        let node = parse_expr("if x > 0 then a := 1; a else b := 2; b fi");
        if let AstNode::IfExpr { then_body, else_body, .. } = &node {
            assert_eq!(then_body.len(), 2);
            assert_eq!(else_body.as_ref().unwrap().len(), 2);
        } else {
            panic!("Expected IfExpr");
        }
    }

    #[test]
    fn test_if_error_no_then() {
        let err = parse("if x > 0 1 fi").unwrap_err();
        assert!(err.message.contains("'then'"), "got: {}", err.message);
    }

    #[test]
    fn test_if_error_no_fi() {
        let err = parse("if x > 0 then 1").unwrap_err();
        assert!(err.message.contains("'fi'"), "got: {}", err.message);
    }

    // =======================================================
    // PARSE-12: Nested control flow
    // =======================================================

    #[test]
    fn test_for_containing_if() {
        let node = parse_expr("for n from 1 to 3 do if n > 1 then n fi od");
        if let AstNode::ForLoop { body, .. } = &node {
            assert_eq!(body.len(), 1);
            assert!(matches!(&body[0].node, AstNode::IfExpr { .. }));
        } else {
            panic!("Expected ForLoop");
        }
    }

    #[test]
    fn test_if_containing_for() {
        let node = parse_expr("if x > 0 then for n from 1 to 5 do n od fi");
        if let AstNode::IfExpr { then_body, .. } = &node {
            assert_eq!(then_body.len(), 1);
            assert!(matches!(&then_body[0].node, AstNode::ForLoop { .. }));
        } else {
            panic!("Expected IfExpr");
        }
    }

    // =======================================================
    // PARSE-13: Procedure definitions
    // =======================================================

    #[test]
    fn test_parse_proc_simple() {
        let node = parse_expr("proc(n) n*n; end");
        if let AstNode::ProcDef { params, locals, options, body } = &node {
            assert_eq!(params, &["n"]);
            assert!(locals.is_empty());
            assert!(options.is_empty());
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected ProcDef, got {:?}", node);
        }
    }

    #[test]
    fn test_parse_proc_locals() {
        let node = parse_expr("proc(n) local k; k := n; end");
        if let AstNode::ProcDef { params, locals, .. } = &node {
            assert_eq!(params, &["n"]);
            assert_eq!(locals, &["k"]);
        } else {
            panic!("Expected ProcDef");
        }
    }

    #[test]
    fn test_parse_proc_option_remember() {
        let node = parse_expr("proc(n) option remember; n; end");
        if let AstNode::ProcDef { options, .. } = &node {
            assert_eq!(options, &["remember"]);
        } else {
            panic!("Expected ProcDef");
        }
    }

    #[test]
    fn test_parse_proc_full() {
        let node = parse_expr("proc(n) local k; option remember; k := n*n; k; end");
        if let AstNode::ProcDef { params, locals, options, body } = &node {
            assert_eq!(params, &["n"]);
            assert_eq!(locals, &["k"]);
            assert_eq!(options, &["remember"]);
            assert_eq!(body.len(), 2);
        } else {
            panic!("Expected ProcDef");
        }
    }

    #[test]
    fn test_parse_proc_end_proc() {
        let node = parse_expr("proc(n) n; end proc");
        if let AstNode::ProcDef { params, .. } = &node {
            assert_eq!(params, &["n"]);
        } else {
            panic!("Expected ProcDef");
        }
    }

    #[test]
    fn test_parse_proc_empty_params() {
        let node = parse_expr("proc() 42; end");
        if let AstNode::ProcDef { params, body, .. } = &node {
            assert!(params.is_empty());
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected ProcDef");
        }
    }

    #[test]
    fn test_parse_proc_multiple_locals() {
        let node = parse_expr("proc(a, b) local x, y, z; x; end");
        if let AstNode::ProcDef { params, locals, .. } = &node {
            assert_eq!(params, &["a", "b"]);
            assert_eq!(locals, &["x", "y", "z"]);
        } else {
            panic!("Expected ProcDef");
        }
    }

    #[test]
    fn test_parse_proc_assign() {
        let node = parse_expr("f := proc(n) n; end");
        if let AstNode::Assign { name, value } = &node {
            assert_eq!(name, "f");
            assert!(matches!(value.as_ref(), AstNode::ProcDef { .. }));
        } else {
            panic!("Expected Assign with ProcDef, got {:?}", node);
        }
    }

    // =======================================================
    // PARSE-14: Ditto operator
    // =======================================================

    #[test]
    fn test_ditto_as_expression() {
        let node = parse_expr("\x22");
        assert_eq!(node, AstNode::LastResult);
    }

    #[test]
    fn test_ditto_in_function_arg() {
        let node = parse_expr("etamake(\x22,q,100)");
        assert_eq!(
            node,
            AstNode::FuncCall {
                name: "etamake".to_string(),
                args: vec![
                    AstNode::LastResult,
                    AstNode::Variable("q".to_string()),
                    AstNode::Integer(100),
                ],
            }
        );
    }

    #[test]
    fn test_ditto_in_arithmetic() {
        let node = parse_expr("\x22 + 1");
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
    // PARSE-15: Proc option/local reorder
    // =======================================================

    #[test]
    fn test_proc_option_before_local() {
        let node = parse_expr("proc(n) option remember; local k; k; end");
        if let AstNode::ProcDef { params, locals, options, body } = &node {
            assert_eq!(params, &["n"]);
            assert_eq!(locals, &["k"]);
            assert_eq!(options, &["remember"]);
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected ProcDef, got {:?}", node);
        }
    }

    #[test]
    fn test_proc_local_before_option() {
        let node = parse_expr("proc(n) local k; option remember; k; end");
        if let AstNode::ProcDef { params, locals, options, body } = &node {
            assert_eq!(params, &["n"]);
            assert_eq!(locals, &["k"]);
            assert_eq!(options, &["remember"]);
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected ProcDef, got {:?}", node);
        }
    }

    #[test]
    fn test_proc_option_only() {
        let node = parse_expr("proc(n) option remember; n; end");
        if let AstNode::ProcDef { locals, options, .. } = &node {
            assert!(locals.is_empty());
            assert_eq!(options, &["remember"]);
        } else {
            panic!("Expected ProcDef, got {:?}", node);
        }
    }

    #[test]
    fn test_proc_local_only() {
        let node = parse_expr("proc(n) local k; k; end");
        if let AstNode::ProcDef { locals, options, .. } = &node {
            assert_eq!(locals, &["k"]);
            assert!(options.is_empty());
        } else {
            panic!("Expected ProcDef, got {:?}", node);
        }
    }

    #[test]
    fn test_proc_neither_local_nor_option() {
        let node = parse_expr("proc(n) n; end");
        if let AstNode::ProcDef { locals, options, .. } = &node {
            assert!(locals.is_empty());
            assert!(options.is_empty());
        } else {
            panic!("Expected ProcDef, got {:?}", node);
        }
    }
}
