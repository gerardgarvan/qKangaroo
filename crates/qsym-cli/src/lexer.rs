//! Lexer (tokenizer) for the q-Kangaroo Maple-style grammar.
//!
//! Converts a source string into a sequence of [`SpannedToken`] values,
//! each carrying its [`Span`] byte range in the original source.

use crate::error::ParseError;
use crate::token::{Span, SpannedToken, Token};

/// Replace common Unicode math operator lookalikes with ASCII equivalents.
/// This allows text pasted from PDFs and papers to parse correctly.
fn normalize_unicode(input: &str) -> String {
    input
        .replace('\u{2227}', "^")   // LOGICAL AND -> caret
        .replace('\u{00B7}', "*")   // MIDDLE DOT -> star
        .replace('\u{2212}', "-")   // MINUS SIGN -> hyphen-minus
        .replace('\u{00D7}', "*")   // MULTIPLICATION SIGN -> star
        .replace('\u{2013}', "-")   // EN DASH -> hyphen-minus
        .replace('\u{2014}', "-")   // EM DASH -> hyphen-minus
        .replace('\u{2018}', "'")   // LEFT SINGLE QUOTATION -> apostrophe
        .replace('\u{2019}', "'")   // RIGHT SINGLE QUOTATION -> apostrophe
        .replace('\u{201C}', "\"")  // LEFT DOUBLE QUOTATION -> double quote
        .replace('\u{201D}', "\"")  // RIGHT DOUBLE QUOTATION -> double quote
}

/// Tokenize a source string into a sequence of spanned tokens.
///
/// The returned vector always ends with a [`Token::Eof`] token whose span
/// points to the end of the input string.  Unicode math operators are
/// normalized to ASCII equivalents before byte-level lexing.
///
/// # Errors
///
/// Returns [`ParseError`] if an unrecognized character is encountered.
pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>, ParseError> {
    let normalized = normalize_unicode(input);
    let bytes = normalized.as_bytes();
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

        // Ditto operator vs string literal disambiguation.
        // Bare `"` followed by a delimiter/operator/whitespace/EOF is the Maple ditto
        // operator (reference to last result). Otherwise it starts a string literal.
        if b == b'"' {
            let next = if pos + 1 < bytes.len() { bytes[pos + 1] } else { 0 };
            let is_ditto = pos + 1 >= bytes.len()
                || matches!(
                    next,
                    b',' | b')' | b';' | b':' | b'+' | b'-' | b'*' | b'/'
                        | b'^' | b']' | b'<' | b'>' | b'=' | b' ' | b'\t'
                        | b'\n' | b'\r'
                );
            if is_ditto {
                tokens.push(SpannedToken {
                    token: Token::Ditto,
                    span: Span::new(pos, pos + 1),
                });
                pos += 1;
                continue;
            }
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

        // Two-character greedy match for `-` -- `->` is Arrow, `-` alone is Minus
        if b == b'-' {
            if pos + 1 < bytes.len() && bytes[pos + 1] == b'>' {
                tokens.push(SpannedToken {
                    token: Token::Arrow,
                    span: Span::new(pos, pos + 2),
                });
                pos += 2;
            } else {
                tokens.push(SpannedToken {
                    token: Token::Minus,
                    span: Span::new(pos, pos + 1),
                });
                pos += 1;
            }
            continue;
        }

        // Single-character tokens
        let single = match b {
            b'+' => Some(Token::Plus),
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
            let word = &normalized[start..pos];
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
            let word = &normalized[start..pos];
            let token = match word {
                "infinity" => Token::Infinity,
                "for" => Token::For,
                "from" => Token::From,
                "to" => Token::To,
                "by" => Token::By,
                "do" => Token::Do,
                "od" => Token::Od,
                "while" => Token::While,
                "if" => Token::If,
                "then" => Token::Then,
                "elif" => Token::Elif,
                "else" => Token::Else,
                "fi" => Token::Fi,
                "end" => Token::End,
                "proc" => Token::Proc,
                "local" => Token::Local,
                "option" => Token::OptionKw,
                "and" => Token::And,
                "or" => Token::Or,
                "not" => Token::Not,
                _ => Token::Ident(word.to_string()),
            };
            tokens.push(SpannedToken {
                token,
                span: Span::new(start, pos),
            });
            continue;
        }

        // Multi-character comparison operators: <, <=, <>, >, >=, =
        if b == b'<' {
            if pos + 1 < bytes.len() && bytes[pos + 1] == b'=' {
                tokens.push(SpannedToken {
                    token: Token::LessEq,
                    span: Span::new(pos, pos + 2),
                });
                pos += 2;
            } else if pos + 1 < bytes.len() && bytes[pos + 1] == b'>' {
                tokens.push(SpannedToken {
                    token: Token::NotEqual,
                    span: Span::new(pos, pos + 2),
                });
                pos += 2;
            } else {
                tokens.push(SpannedToken {
                    token: Token::Less,
                    span: Span::new(pos, pos + 1),
                });
                pos += 1;
            }
            continue;
        }

        if b == b'>' {
            if pos + 1 < bytes.len() && bytes[pos + 1] == b'=' {
                tokens.push(SpannedToken {
                    token: Token::GreaterEq,
                    span: Span::new(pos, pos + 2),
                });
                pos += 2;
            } else {
                tokens.push(SpannedToken {
                    token: Token::Greater,
                    span: Span::new(pos, pos + 1),
                });
                pos += 1;
            }
            continue;
        }

        if b == b'=' {
            tokens.push(SpannedToken {
                token: Token::Equal,
                span: Span::new(pos, pos + 1),
            });
            pos += 1;
            continue;
        }

        // Unknown character
        let c = normalized[pos..].chars().next().unwrap();
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

    // =======================================================
    // Comparison operator lexing
    // =======================================================

    #[test]
    fn test_less_than() {
        let toks = tokens("x < 5");
        assert_eq!(toks, vec![Token::Ident("x".to_string()), Token::Less, Token::Integer(5), Token::Eof]);
    }

    #[test]
    fn test_less_equal() {
        let toks = tokens("x <= 5");
        assert_eq!(toks, vec![Token::Ident("x".to_string()), Token::LessEq, Token::Integer(5), Token::Eof]);
    }

    #[test]
    fn test_not_equal() {
        let toks = tokens("x <> 5");
        assert_eq!(toks, vec![Token::Ident("x".to_string()), Token::NotEqual, Token::Integer(5), Token::Eof]);
    }

    #[test]
    fn test_greater_than() {
        let toks = tokens("x > 5");
        assert_eq!(toks, vec![Token::Ident("x".to_string()), Token::Greater, Token::Integer(5), Token::Eof]);
    }

    #[test]
    fn test_greater_equal() {
        let toks = tokens("x >= 5");
        assert_eq!(toks, vec![Token::Ident("x".to_string()), Token::GreaterEq, Token::Integer(5), Token::Eof]);
    }

    #[test]
    fn test_equal() {
        let toks = tokens("x = 5");
        assert_eq!(toks, vec![Token::Ident("x".to_string()), Token::Equal, Token::Integer(5), Token::Eof]);
    }

    #[test]
    fn test_equal_vs_assign() {
        let eq = tokens("x = 5");
        let assign = tokens("x := 5");
        assert_eq!(eq[1], Token::Equal);
        assert_eq!(assign[1], Token::Assign);
    }

    // =======================================================
    // Keyword lexing
    // =======================================================

    #[test]
    fn test_for_loop_keywords() {
        let toks = tokens("for n from 1 to 5 do n od");
        assert_eq!(
            toks,
            vec![
                Token::For, Token::Ident("n".to_string()), Token::From,
                Token::Integer(1), Token::To, Token::Integer(5),
                Token::Do, Token::Ident("n".to_string()), Token::Od, Token::Eof,
            ]
        );
    }

    #[test]
    fn test_boolean_keywords() {
        let toks = tokens("if x and y or not z");
        assert_eq!(
            toks,
            vec![
                Token::If, Token::Ident("x".to_string()), Token::And,
                Token::Ident("y".to_string()), Token::Or, Token::Not,
                Token::Ident("z".to_string()), Token::Eof,
            ]
        );
    }

    // =======================================================
    // Procedure keyword lexing
    // =======================================================

    #[test]
    fn test_lex_proc_keyword() {
        let toks = tokens("proc");
        assert_eq!(toks, vec![Token::Proc, Token::Eof]);
    }

    #[test]
    fn test_lex_local_keyword() {
        let toks = tokens("local");
        assert_eq!(toks, vec![Token::Local, Token::Eof]);
    }

    #[test]
    fn test_lex_option_keyword() {
        let toks = tokens("option");
        assert_eq!(toks, vec![Token::OptionKw, Token::Eof]);
    }

    // =======================================================
    // Ditto operator lexing
    // =======================================================

    #[test]
    fn test_ditto_before_comma() {
        let toks = tokens("f(\",q,100)");
        assert_eq!(
            toks,
            vec![
                Token::Ident("f".to_string()),
                Token::LParen,
                Token::Ditto,
                Token::Comma,
                Token::Ident("q".to_string()),
                Token::Comma,
                Token::Integer(100),
                Token::RParen,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_ditto_before_rparen() {
        let toks = tokens("f(\")");
        assert_eq!(
            toks,
            vec![
                Token::Ident("f".to_string()),
                Token::LParen,
                Token::Ditto,
                Token::RParen,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_ditto_before_semicolon() {
        let toks = tokens("\";");
        assert_eq!(toks, vec![Token::Ditto, Token::Semi, Token::Eof]);
    }

    #[test]
    fn test_ditto_before_space_plus() {
        let toks = tokens("\" + 1");
        assert_eq!(
            toks,
            vec![Token::Ditto, Token::Plus, Token::Integer(1), Token::Eof]
        );
    }

    #[test]
    fn test_ditto_at_eof() {
        let toks = tokens("\"");
        assert_eq!(toks, vec![Token::Ditto, Token::Eof]);
    }

    #[test]
    fn test_string_literal_still_works() {
        let toks = tokens(r#""hello""#);
        assert_eq!(toks, vec![Token::StringLit("hello".to_string()), Token::Eof]);
    }

    #[test]
    fn test_empty_string_still_works() {
        let toks = tokens("\"\"");
        assert_eq!(toks, vec![Token::StringLit("".to_string()), Token::Eof]);
    }

    // =======================================================
    // Arrow operator lexing
    // =======================================================

    #[test]
    fn test_lex_arrow() {
        let toks = tokens("q -> expr");
        assert_eq!(
            toks,
            vec![
                Token::Ident("q".to_string()),
                Token::Arrow,
                Token::Ident("expr".to_string()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_lex_minus_still_works() {
        let toks = tokens("a - b");
        assert_eq!(
            toks,
            vec![
                Token::Ident("a".to_string()),
                Token::Minus,
                Token::Ident("b".to_string()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_lex_arrow_no_spaces() {
        let toks = tokens("q->expr");
        assert_eq!(
            toks,
            vec![
                Token::Ident("q".to_string()),
                Token::Arrow,
                Token::Ident("expr".to_string()),
                Token::Eof,
            ]
        );
    }

    // =======================================================
    // Unicode normalization tests (LANG-03)
    // =======================================================

    #[test]
    fn test_normalize_unicode_all_replacements() {
        let input = "\u{2227}\u{00B7}\u{2212}\u{00D7}\u{2013}\u{2014}\u{2018}\u{2019}\u{201C}\u{201D}";
        let result = normalize_unicode(input);
        assert_eq!(result, "^*-*--''\"\"");
    }

    #[test]
    fn test_normalize_unicode_passthrough() {
        // ASCII input should pass through unchanged
        let input = "q^5 + 3*x - 1";
        let result = normalize_unicode(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_unicode_logical_and_as_caret() {
        // U+2227 LOGICAL AND should become caret for exponentiation
        let toks = tokens("q\u{2227}5");
        assert_eq!(
            toks,
            vec![Token::Ident("q".to_string()), Token::Caret, Token::Integer(5), Token::Eof]
        );
    }

    #[test]
    fn test_unicode_multiplication_sign() {
        // U+00D7 MULTIPLICATION SIGN should become star
        let toks = tokens("3 \u{00D7} 5");
        assert_eq!(
            toks,
            vec![Token::Integer(3), Token::Star, Token::Integer(5), Token::Eof]
        );
    }

    #[test]
    fn test_unicode_minus_sign() {
        // U+2212 MINUS SIGN should become hyphen-minus
        let toks = tokens("x \u{2212} 1");
        assert_eq!(
            toks,
            vec![Token::Ident("x".to_string()), Token::Minus, Token::Integer(1), Token::Eof]
        );
    }

    #[test]
    fn test_unicode_middle_dot() {
        // U+00B7 MIDDLE DOT should become star
        let toks = tokens("3 \u{00B7} 5");
        assert_eq!(
            toks,
            vec![Token::Integer(3), Token::Star, Token::Integer(5), Token::Eof]
        );
    }

    #[test]
    fn test_unicode_en_dash_as_minus() {
        // U+2013 EN DASH should become minus
        let toks = tokens("x \u{2013} 1");
        assert_eq!(
            toks,
            vec![Token::Ident("x".to_string()), Token::Minus, Token::Integer(1), Token::Eof]
        );
    }

    #[test]
    fn test_unicode_mixed_with_ascii() {
        // Mix of Unicode and ASCII operators in same expression
        let toks = tokens("q\u{2227}5 + 3 \u{00D7} x \u{2212} 1");
        assert_eq!(
            toks,
            vec![
                Token::Ident("q".to_string()), Token::Caret, Token::Integer(5),
                Token::Plus, Token::Integer(3), Token::Star,
                Token::Ident("x".to_string()), Token::Minus, Token::Integer(1),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_unicode_smart_quotes_normalized() {
        // Smart quotes are normalized to ASCII -- this means strings containing
        // smart quotes will also be normalized. This is acceptable behavior since
        // normalization happens before tokenization.
        let toks = tokens("\u{201C}hello\u{201D}");
        assert_eq!(
            toks,
            vec![Token::StringLit("hello".to_string()), Token::Eof]
        );
    }
}
