//! Error types for the q-Kangaroo parser.
//!
//! Provides [`ParseError`] with caret-style error rendering for clear
//! user-facing diagnostics.

use crate::token::Span;
use std::fmt;

/// A parse error with source location.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Human-readable error description.
    pub message: String,
    /// Byte-offset span where the error occurred.
    pub span: Span,
}

impl ParseError {
    /// Create a new parse error.
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }

    /// Render this error with a caret pointing to the error location.
    ///
    /// Produces output like:
    /// ```text
    /// parse error at column 5: unexpected token
    ///   aqprod(q, 10
    ///       ^
    /// ```
    ///
    /// Column is 1-indexed (derived from `span.start` byte offset, which
    /// equals the column for ASCII input).
    pub fn render(&self, source: &str) -> String {
        let col = self.span.start;
        let col_display = col + 1; // 1-indexed for human display
        let spaces = " ".repeat(col + 2); // 2 for "  " prefix
        format!(
            "parse error at column {}: {}\n  {}\n{}^",
            col_display, self.message, source, spaces
        )
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "parse error at byte {}: {}",
            self.span.start, self.message
        )
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_caret_at_start() {
        let err = ParseError::new("unexpected token", Span::new(0, 1));
        let rendered = err.render("1 + 2");
        assert_eq!(
            rendered,
            "parse error at column 1: unexpected token\n  1 + 2\n  ^"
        );
    }

    #[test]
    fn render_caret_at_middle() {
        let err = ParseError::new("expected ')'", Span::new(4, 5));
        let rendered = err.render("f(q, 10");
        assert_eq!(
            rendered,
            "parse error at column 5: expected ')'\n  f(q, 10\n      ^"
        );
    }

    #[test]
    fn render_caret_at_end() {
        let err = ParseError::new("unexpected end of input", Span::new(3, 3));
        let rendered = err.render("1 +");
        assert_eq!(
            rendered,
            "parse error at column 4: unexpected end of input\n  1 +\n     ^"
        );
    }

    #[test]
    fn display_format() {
        let err = ParseError::new("bad token", Span::new(7, 8));
        assert_eq!(format!("{}", err), "parse error at byte 7: bad token");
    }

    #[test]
    fn error_trait() {
        let err = ParseError::new("test", Span::new(0, 1));
        // Verify it implements std::error::Error
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn constructor() {
        let err = ParseError::new("msg", Span::new(3, 5));
        assert_eq!(err.message, "msg");
        assert_eq!(err.span.start, 3);
        assert_eq!(err.span.end, 5);
    }
}
