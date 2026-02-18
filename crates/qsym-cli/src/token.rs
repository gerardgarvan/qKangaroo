//! Token types for the q-Kangaroo lexer.
//!
//! Covers the full Maple-style grammar: integer literals, operators,
//! delimiters, keywords (q, infinity), and statement terminators.

/// A lexical token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Integer literal that fits in i64.
    Integer(i64),
    /// Integer literal too large for i64 (stored as decimal string).
    BigInteger(String),
    /// The `infinity` keyword.
    Infinity,
    /// The `q` keyword (reserved indeterminate, not a variable).
    Q,
    /// Identifier: function names (aqprod, etaq, ETAR) and user variables (f, g).
    Ident(String),
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `^`
    Caret,
    /// `:=` assignment operator.
    Assign,
    /// `%` ditto operator (reference to last result).
    Percent,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `,`
    Comma,
    /// `;` statement terminator (print result).
    Semi,
    /// `:` statement terminator (suppress output).
    Colon,
    /// End of input.
    Eof,
}

/// Byte-offset span in source text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    /// Start byte offset (inclusive).
    pub start: usize,
    /// End byte offset (exclusive).
    pub end: usize,
}

impl Span {
    /// Create a new span from start (inclusive) to end (exclusive).
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// A token paired with its source location.
#[derive(Debug, Clone)]
pub struct SpannedToken {
    /// The token value.
    pub token: Token,
    /// Source location of this token.
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_integer_equality() {
        assert_eq!(Token::Integer(42), Token::Integer(42));
        assert_ne!(Token::Integer(42), Token::Integer(43));
    }

    #[test]
    fn token_variants_distinct() {
        assert_ne!(Token::Q, Token::Infinity);
        assert_ne!(Token::Plus, Token::Minus);
        assert_ne!(Token::Semi, Token::Colon);
    }

    #[test]
    fn span_construction() {
        let s = Span::new(5, 10);
        assert_eq!(s.start, 5);
        assert_eq!(s.end, 10);
    }

    #[test]
    fn spanned_token_construction() {
        let st = SpannedToken {
            token: Token::Integer(99),
            span: Span::new(0, 2),
        };
        assert_eq!(st.token, Token::Integer(99));
        assert_eq!(st.span, Span::new(0, 2));
    }

    #[test]
    fn big_integer_stores_string() {
        let big = Token::BigInteger("99999999999999999999".to_string());
        if let Token::BigInteger(s) = &big {
            assert_eq!(s, "99999999999999999999");
        } else {
            panic!("Expected BigInteger variant");
        }
    }

    #[test]
    fn ident_stores_name() {
        let tok = Token::Ident("aqprod".to_string());
        if let Token::Ident(name) = &tok {
            assert_eq!(name, "aqprod");
        } else {
            panic!("Expected Ident variant");
        }
    }
}
