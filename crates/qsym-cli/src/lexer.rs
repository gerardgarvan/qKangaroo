//! Lexer (tokenizer) for the q-Kangaroo Maple-style grammar.
//!
//! Converts a source string into a sequence of [`SpannedToken`] values,
//! each carrying its [`Span`] byte range in the original source.

use crate::error::ParseError;
use crate::token::{Span, SpannedToken, Token};

/// Tokenize a source string into a sequence of spanned tokens.
///
/// The returned vector always ends with a [`Token::Eof`] token whose span
/// points to the end of the input string.
///
/// # Errors
///
/// Returns [`ParseError`] if an unrecognized character is encountered.
pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>, ParseError> {
    let bytes = input.as_bytes();
    let mut pos = 0usize;
    let mut tokens = Vec::new();

    while pos < bytes.len() {
        let b = bytes[pos];

        // Skip whitespace (space and tab; newlines won't appear in REPL single-line input)
        if b == b' ' || b == b'\t' {
            pos += 1;
            continue;
        }

        // Single-character tokens
        let single = match b {
            b'+' => Some(Token::Plus),
            b'-' => Some(Token::Minus),
            b'*' => Some(Token::Star),
            b'/' => Some(Token::Slash),
            b'^' => Some(Token::Caret),
            b'%' => Some(Token::Percent),
            b'(' => Some(Token::LParen),
            b')' => Some(Token::RParen),
            b',' => Some(Token::Comma),
            b';' => Some(Token::Semi),
            _ => None,
        };

        if let Some(token) = single {
            tokens.push(SpannedToken {
                token,
                span: Span::new(pos, pos + 1),
            });
            pos += 1;
            continue;
        }

        // Two-character greedy match for `:` -- `:=` is Assign, `:` alone is Colon
        if b == b':' {
            if pos + 1 < bytes.len() && bytes[pos + 1] == b'=' {
                tokens.push(SpannedToken {
                    token: Token::Assign,
                    span: Span::new(pos, pos + 2),
                });
                pos += 2;
            } else {
                tokens.push(SpannedToken {
                    token: Token::Colon,
                    span: Span::new(pos, pos + 1),
                });
                pos += 1;
            }
            continue;
        }

        // Numeric literals: consecutive ASCII digits
        if b.is_ascii_digit() {
            let start = pos;
            while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                pos += 1;
            }
            let word = &input[start..pos];
            let token = match word.parse::<i64>() {
                Ok(n) => Token::Integer(n),
                Err(_) => Token::BigInteger(word.to_string()),
            };
            tokens.push(SpannedToken {
                token,
                span: Span::new(start, pos),
            });
            continue;
        }

        // Identifiers and keywords: [a-zA-Z_][a-zA-Z0-9_]*
        if b.is_ascii_alphabetic() || b == b'_' {
            let start = pos;
            while pos < bytes.len()
                && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_')
            {
                pos += 1;
            }
            let word = &input[start..pos];
            let token = match word {
                "infinity" => Token::Infinity,
                "q" => Token::Q,
                _ => Token::Ident(word.to_string()),
            };
            tokens.push(SpannedToken {
                token,
                span: Span::new(start, pos),
            });
            continue;
        }

        // Unknown character
        let c = input[pos..].chars().next().unwrap();
        return Err(ParseError::new(
            format!("unexpected character '{}'", c),
            Span::new(pos, pos + c.len_utf8()),
        ));
    }

    // Append Eof sentinel
    tokens.push(SpannedToken {
        token: Token::Eof,
        span: Span::new(pos, pos),
    });

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: extract just the Token variants from tokenize result.
    fn tokens(input: &str) -> Vec<Token> {
        tokenize(input).unwrap().into_iter().map(|st| st.token).collect()
    }

    #[test]
    fn test_simple_tokens() {
        let toks = tokens("+ - * / ^ % ( ) , ; :");
        assert_eq!(
            toks,
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Caret,
                Token::Percent,
                Token::LParen,
                Token::RParen,
                Token::Comma,
                Token::Semi,
                Token::Colon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_assign_vs_colon() {
        // `:=` should be Assign
        let toks = tokens("f := 5");
        assert_eq!(
            toks,
            vec![
                Token::Ident("f".to_string()),
                Token::Assign,
                Token::Integer(5),
                Token::Eof,
            ]
        );

        // `:` alone should be Colon
        let toks = tokens("f:");
        assert_eq!(
            toks,
            vec![
                Token::Ident("f".to_string()),
                Token::Colon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_integer() {
        let toks = tokens("42");
        assert_eq!(toks, vec![Token::Integer(42), Token::Eof]);
    }

    #[test]
    fn test_big_integer() {
        let toks = tokens("99999999999999999999999");
        assert_eq!(
            toks,
            vec![
                Token::BigInteger("99999999999999999999999".to_string()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_keywords() {
        let toks = tokens("q infinity");
        assert_eq!(toks, vec![Token::Q, Token::Infinity, Token::Eof]);
    }

    #[test]
    fn test_identifiers() {
        let toks = tokens("aqprod etaq ETAR partition_count f");
        assert_eq!(
            toks,
            vec![
                Token::Ident("aqprod".to_string()),
                Token::Ident("etaq".to_string()),
                Token::Ident("ETAR".to_string()),
                Token::Ident("partition_count".to_string()),
                Token::Ident("f".to_string()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_function_call_tokens() {
        let toks = tokens("aqprod(q,q,infinity,20)");
        assert_eq!(
            toks,
            vec![
                Token::Ident("aqprod".to_string()),
                Token::LParen,
                Token::Q,
                Token::Comma,
                Token::Q,
                Token::Comma,
                Token::Infinity,
                Token::Comma,
                Token::Integer(20),
                Token::RParen,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_assignment_tokens() {
        let toks = tokens("f := etaq(1,1,20)");
        assert_eq!(
            toks,
            vec![
                Token::Ident("f".to_string()),
                Token::Assign,
                Token::Ident("etaq".to_string()),
                Token::LParen,
                Token::Integer(1),
                Token::Comma,
                Token::Integer(1),
                Token::Comma,
                Token::Integer(20),
                Token::RParen,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_multi_statement() {
        let toks = tokens("f := 5; g := 10:");
        assert_eq!(
            toks,
            vec![
                Token::Ident("f".to_string()),
                Token::Assign,
                Token::Integer(5),
                Token::Semi,
                Token::Ident("g".to_string()),
                Token::Assign,
                Token::Integer(10),
                Token::Colon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_unknown_char() {
        let err = tokenize("f @ g").unwrap_err();
        assert!(err.message.contains("unexpected character '@'"));
        assert_eq!(err.span.start, 2);
    }

    #[test]
    fn test_empty_input() {
        let toks = tokens("");
        assert_eq!(toks, vec![Token::Eof]);
    }

    #[test]
    fn test_whitespace_handling() {
        let toks = tokens("  q  ");
        assert_eq!(toks, vec![Token::Q, Token::Eof]);
    }
}
