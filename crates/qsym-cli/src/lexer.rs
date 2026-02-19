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

        // Skip whitespace (space, tab, newline, carriage return)
        if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
            pos += 1;
            continue;
        }

        // Skip # line comments
        if b == b'#' {
            while pos < bytes.len() && bytes[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }

        // String literals (double-quoted)
        if b == b'"' {
            let start = pos;
            pos += 1; // skip opening quote
            let mut value = String::new();
            while pos < bytes.len() && bytes[pos] != b'"' {
                if bytes[pos] == b'\\' && pos + 1 < bytes.len() {
                    match bytes[pos + 1] {
                        b'\\' => { value.push('\\'); pos += 2; }
                        b'"' => { value.push('"'); pos += 2; }
                        b'n' => { value.push('\n'); pos += 2; }
                        b't' => { value.push('\t'); pos += 2; }
                        _ => { value.push(bytes[pos] as char); pos += 1; }
                    }
                } else {
                    value.push(bytes[pos] as char);
                    pos += 1;
                }
            }
            if pos >= bytes.len() {
                return Err(ParseError::new(
                    "unterminated string literal".to_string(),
                    Span::new(start, pos),
                ));
            }
            pos += 1; // skip closing quote
            tokens.push(SpannedToken {
                token: Token::StringLit(value),
                span: Span::new(start, pos),
            });
            continue;
        }

        // Single-quoted string literals (for Maple unassign syntax: x := 'x')
        if b == b'\'' {
            let start = pos;
            pos += 1; // skip opening quote
            let mut value = String::new();
            while pos < bytes.len() && bytes[pos] != b'\'' {
                value.push(bytes[pos] as char);
                pos += 1;
            }
            if pos >= bytes.len() {
                return Err(ParseError::new(
                    "unterminated single-quoted string".to_string(),
                    Span::new(start, pos),
                ));
            }
            pos += 1; // skip closing quote
            tokens.push(SpannedToken {
                token: Token::StringLit(value),
                span: Span::new(start, pos),
            });
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
            b'[' => Some(Token::LBracket),
            b']' => Some(Token::RBracket),
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
        assert_eq!(toks, vec![Token::Ident("q".to_string()), Token::Infinity, Token::Eof]);
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
                Token::Ident("q".to_string()),
                Token::Comma,
                Token::Ident("q".to_string()),
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
        assert_eq!(toks, vec![Token::Ident("q".to_string()), Token::Eof]);
    }

    #[test]
    fn test_comment_skipping() {
        let toks = tokens("1 + 2 # this is ignored");
        assert_eq!(toks, vec![Token::Integer(1), Token::Plus, Token::Integer(2), Token::Eof]);
    }

    #[test]
    fn test_comment_full_line() {
        let toks = tokens("# comment\n1");
        assert_eq!(toks, vec![Token::Integer(1), Token::Eof]);
    }

    #[test]
    fn test_newline_as_whitespace() {
        let toks = tokens("1\n+\n2");
        assert_eq!(toks, vec![Token::Integer(1), Token::Plus, Token::Integer(2), Token::Eof]);
    }

    #[test]
    fn test_string_literal() {
        let toks = tokens(r#""hello""#);
        assert_eq!(toks, vec![Token::StringLit("hello".to_string()), Token::Eof]);
    }

    #[test]
    fn test_string_escape_quote() {
        let toks = tokens(r#""say \"hi\"""#);
        assert_eq!(toks, vec![Token::StringLit("say \"hi\"".to_string()), Token::Eof]);
    }

    #[test]
    fn test_string_unterminated() {
        let err = tokenize(r#""hello"#).unwrap_err();
        assert!(err.message.contains("unterminated"), "got: {}", err.message);
    }

    #[test]
    fn test_comment_after_string() {
        let toks = tokens(r#""file.qk" # comment"#);
        assert_eq!(toks, vec![Token::StringLit("file.qk".to_string()), Token::Eof]);
    }

    #[test]
    fn test_multiline_expression() {
        let toks = tokens("aqprod(\n  q,q,\n  infinity,20\n)");
        let expected = tokens("aqprod(q,q,infinity,20)");
        assert_eq!(toks, expected);
    }

    #[test]
    fn test_single_quote_string() {
        let toks = tokens("'hello'");
        assert_eq!(toks, vec![Token::StringLit("hello".to_string()), Token::Eof]);
    }

    #[test]
    fn test_unassign_syntax() {
        let toks = tokens("x := 'x'");
        assert_eq!(
            toks,
            vec![
                Token::Ident("x".to_string()),
                Token::Assign,
                Token::StringLit("x".to_string()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_single_quote_unterminated() {
        let err = tokenize("'hello").unwrap_err();
        assert!(err.message.contains("unterminated"), "got: {}", err.message);
    }
}
