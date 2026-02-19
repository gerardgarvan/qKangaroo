//! Script execution engine for non-interactive modes.
//!
//! Provides [`execute_source()`] and [`execute_file()`] for running
//! q-Kangaroo code from script files, `-c` expressions, and piped stdin.

use crate::environment::Environment;
use crate::eval;
use crate::format::format_value;

// ---------------------------------------------------------------------------
// Exit code constants (sysexits-compatible)
// ---------------------------------------------------------------------------

/// Normal termination.
pub const EXIT_SUCCESS: u8 = 0;
/// Evaluation error (runtime error).
pub const EXIT_EVAL_ERROR: u8 = 1;
/// Bad CLI usage (unknown flag, missing argument).
pub const EXIT_USAGE: u8 = 2;
/// Parse error in input (syntax error).
pub const EXIT_PARSE_ERROR: u8 = 65;
/// Script file not found or unreadable.
pub const EXIT_FILE_NOT_FOUND: u8 = 66;
/// Caught panic from qsym-core.
pub const EXIT_PANIC: u8 = 70;
/// I/O error (permission denied, disk full, etc.).
pub const EXIT_IO_ERROR: u8 = 74;

// ---------------------------------------------------------------------------
// ScriptResult
// ---------------------------------------------------------------------------

/// Result of executing a script or expression.
pub enum ScriptResult {
    /// All statements executed successfully.
    Success,
    /// A parse error occurred.
    ParseError(String),
    /// An eval error occurred.
    EvalError(String),
    /// A caught panic.
    Panic(String),
    /// Script file not found (exit code 66).
    FileNotFound(String),
    /// I/O error reading file (exit code 74).
    IoError(String),
}

impl ScriptResult {
    /// Convert to an exit code.
    pub fn exit_code(&self) -> u8 {
        match self {
            ScriptResult::Success => EXIT_SUCCESS,
            ScriptResult::ParseError(_) => EXIT_PARSE_ERROR,
            ScriptResult::EvalError(_) => EXIT_EVAL_ERROR,
            ScriptResult::Panic(_) => EXIT_PANIC,
            ScriptResult::FileNotFound(_) => EXIT_FILE_NOT_FOUND,
            ScriptResult::IoError(_) => EXIT_IO_ERROR,
        }
    }

    /// Get the error message, if any.
    pub fn error_message(&self) -> Option<&str> {
        match self {
            ScriptResult::Success => None,
            ScriptResult::ParseError(msg) => Some(msg),
            ScriptResult::EvalError(msg) => Some(msg),
            ScriptResult::Panic(msg) => Some(msg),
            ScriptResult::FileNotFound(msg) => Some(msg),
            ScriptResult::IoError(msg) => Some(msg),
        }
    }
}

// ---------------------------------------------------------------------------
// Statement line tracking
// ---------------------------------------------------------------------------

/// Find the byte offset of the first token in each statement.
///
/// Statements are separated by `;` or `:` tokens. Returns a `Vec<usize>`
/// where entry `i` is the byte offset of the first token in statement `i`.
fn compute_stmt_starts(source: &str) -> Vec<usize> {
    let tokens = match crate::lexer::tokenize(source) {
        Ok(t) => t,
        Err(_) => return vec![0],
    };
    let mut starts = Vec::new();
    let mut expect_start = true;
    for st in &tokens {
        if st.token == crate::token::Token::Eof {
            break;
        }
        if expect_start {
            starts.push(st.span.start);
            expect_start = false;
        }
        if matches!(st.token, crate::token::Token::Semi | crate::token::Token::Colon) {
            expect_start = true;
        }
    }
    if starts.is_empty() {
        starts.push(0);
    }
    starts
}

/// Compute the 1-indexed source line number for statement `stmt_idx`.
fn compute_stmt_line(source: &str, _stmts: &[crate::ast::Stmt], stmt_idx: usize) -> usize {
    let starts = compute_stmt_starts(source);
    let offset = starts.get(stmt_idx).copied().unwrap_or(0);
    crate::error::byte_offset_to_line_col(source, offset).0
}

// ---------------------------------------------------------------------------
// execute_source / execute_source_with_context
// ---------------------------------------------------------------------------

/// Execute a source string containing one or more statements.
///
/// Thin wrapper around [`execute_source_with_context()`] with no filename.
pub fn execute_source(
    source: &str,
    env: &mut Environment,
    verbose: bool,
) -> ScriptResult {
    execute_source_with_context(source, env, verbose, None)
}

/// Execute a source string with optional filename context for error messages.
///
/// Parses the entire source (comments and newlines handled by the lexer)
/// and evaluates each statement. Results of non-suppressed statements
/// (those with `;` or implicit terminator) are printed to stdout.
///
/// If `verbose` is true, per-statement timing is printed to stderr.
/// If `filename` is `Some`, parse errors show `filename:line:col` and eval
/// errors show `filename:line`.
///
/// Stops on the first error (fail-fast).
pub fn execute_source_with_context(
    source: &str,
    env: &mut Environment,
    verbose: bool,
    filename: Option<&str>,
) -> ScriptResult {
    let stmts = match crate::parser::parse(source) {
        Ok(stmts) => stmts,
        Err(e) => {
            let msg = match filename {
                Some(f) => e.render_for_file(source, f),
                None => e.render(source),
            };
            return ScriptResult::ParseError(msg);
        }
    };

    for (stmt_idx, stmt) in stmts.iter().enumerate() {
        let start = if verbose {
            Some(std::time::Instant::now())
        } else {
            None
        };

        match eval::eval_stmt_safe(stmt, env) {
            Ok(Some(val)) => {
                println!("{}", format_value(&val));
                if let Some(t) = start {
                    eprintln!("  [{:.3}s]", t.elapsed().as_secs_f64());
                }
            }
            Ok(None) => {
                if let Some(t) = start {
                    eprintln!("  [{:.3}s]", t.elapsed().as_secs_f64());
                }
            }
            Err(e) => {
                let base_msg = format!("{}", e);
                let msg = match filename {
                    Some(f) => {
                        let line = compute_stmt_line(source, &stmts, stmt_idx);
                        format!("{}:{}: {}", f, line, base_msg)
                    }
                    None => base_msg,
                };
                return if matches!(e, eval::EvalError::Panic(_)) {
                    ScriptResult::Panic(msg)
                } else {
                    ScriptResult::EvalError(msg)
                };
            }
        }
    }

    ScriptResult::Success
}

// ---------------------------------------------------------------------------
// execute_file
// ---------------------------------------------------------------------------

/// Execute a script file by path.
///
/// Reads the entire file into memory and passes it to
/// [`execute_source_with_context()`] with the filename for error context.
///
/// Returns `ScriptResult::FileNotFound` (exit 66) if the file does not exist,
/// or `ScriptResult::IoError` (exit 74) for other I/O failures.
pub fn execute_file(
    path: &str,
    env: &mut Environment,
    verbose: bool,
) -> ScriptResult {
    match std::fs::read_to_string(path) {
        Ok(source) => execute_source_with_context(&source, env, verbose, Some(path)),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                ScriptResult::FileNotFound(format!("file not found: '{}': {}", path, e))
            }
            _ => {
                ScriptResult::IoError(format!("cannot read '{}': {}", path, e))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_source_simple() {
        let mut env = Environment::new();
        let result = execute_source("1 + 1", &mut env, false);
        assert!(matches!(result, ScriptResult::Success));
    }

    #[test]
    fn test_execute_source_with_comments() {
        let mut env = Environment::new();
        let source = "# This is a comment\n1 + 1";
        let result = execute_source(source, &mut env, false);
        assert!(matches!(result, ScriptResult::Success));
    }

    #[test]
    fn test_execute_source_multiline() {
        let mut env = Environment::new();
        let source = "aqprod(\n  1, 1, 1,\n  infinity, 20\n)";
        let result = execute_source(source, &mut env, false);
        if let Some(msg) = result.error_message() {
            panic!("Expected success, got error: {}", msg);
        }
        assert!(matches!(result, ScriptResult::Success));
    }

    #[test]
    fn test_execute_source_multi_statement() {
        let mut env = Environment::new();
        let source = "f := etaq(1,1,20):\ng := etaq(2,1,20):\nf * g";
        let result = execute_source(source, &mut env, false);
        assert!(matches!(result, ScriptResult::Success));
    }

    #[test]
    fn test_execute_source_parse_error() {
        let mut env = Environment::new();
        let result = execute_source("1 + + 2", &mut env, false);
        assert!(matches!(result, ScriptResult::ParseError(_)));
    }

    #[test]
    fn test_execute_source_undefined_var_returns_symbol() {
        // After Phase 33: undefined variables return Symbol values (success),
        // not eval errors.
        let mut env = Environment::new();
        let result = execute_source("undefined_var", &mut env, false);
        assert!(matches!(result, ScriptResult::Success));
    }

    #[test]
    fn test_execute_source_eval_error() {
        // Use a real eval error: wrong argument count for etaq
        let mut env = Environment::new();
        let result = execute_source("etaq(1)", &mut env, false);
        assert!(matches!(result, ScriptResult::EvalError(_)));
    }

    #[test]
    fn test_execute_file_not_found() {
        let mut env = Environment::new();
        let result = execute_file("/nonexistent/path/script.qk", &mut env, false);
        assert!(matches!(result, ScriptResult::FileNotFound(_)));
        if let ScriptResult::FileNotFound(msg) = result {
            assert!(msg.contains("file not found"));
        }
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(ScriptResult::Success.exit_code(), 0);
        assert_eq!(ScriptResult::ParseError("x".into()).exit_code(), 65);
        assert_eq!(ScriptResult::EvalError("x".into()).exit_code(), 1);
        assert_eq!(ScriptResult::Panic("x".into()).exit_code(), 70);
    }

    #[test]
    fn test_exit_code_file_not_found() {
        assert_eq!(
            ScriptResult::FileNotFound("file not found".into()).exit_code(),
            66
        );
    }

    #[test]
    fn test_exit_code_io_error() {
        assert_eq!(
            ScriptResult::IoError("permission denied".into()).exit_code(),
            74
        );
    }

    #[test]
    fn test_error_message_variants() {
        assert!(ScriptResult::FileNotFound("msg".into()).error_message().is_some());
        assert!(ScriptResult::IoError("msg".into()).error_message().is_some());
    }

    #[test]
    fn test_execute_source_assignment_persists() {
        let mut env = Environment::new();
        let result = execute_source("x := 42:", &mut env, false);
        assert!(matches!(result, ScriptResult::Success));
        assert!(env.get_var("x").is_some());
    }

    #[test]
    fn test_execute_source_empty() {
        let mut env = Environment::new();
        let result = execute_source("", &mut env, false);
        assert!(matches!(result, ScriptResult::Success));
    }

    #[test]
    fn test_execute_source_only_comments() {
        let mut env = Environment::new();
        let result = execute_source("# just a comment\n# another", &mut env, false);
        assert!(matches!(result, ScriptResult::Success));
    }
}
