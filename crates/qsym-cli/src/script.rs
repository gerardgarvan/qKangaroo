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
}

impl ScriptResult {
    /// Convert to an exit code.
    pub fn exit_code(&self) -> u8 {
        match self {
            ScriptResult::Success => EXIT_SUCCESS,
            ScriptResult::ParseError(_) => EXIT_PARSE_ERROR,
            ScriptResult::EvalError(_) => EXIT_EVAL_ERROR,
            ScriptResult::Panic(_) => EXIT_PANIC,
        }
    }

    /// Get the error message, if any.
    pub fn error_message(&self) -> Option<&str> {
        match self {
            ScriptResult::Success => None,
            ScriptResult::ParseError(msg) => Some(msg),
            ScriptResult::EvalError(msg) => Some(msg),
            ScriptResult::Panic(msg) => Some(msg),
        }
    }
}

// ---------------------------------------------------------------------------
// execute_source
// ---------------------------------------------------------------------------

/// Execute a source string containing one or more statements.
///
/// Parses the entire source (comments and newlines handled by the lexer)
/// and evaluates each statement. Results of non-suppressed statements
/// (those with `;` or implicit terminator) are printed to stdout.
///
/// If `verbose` is true, per-statement timing is printed to stderr.
///
/// Stops on the first error (fail-fast).
pub fn execute_source(
    source: &str,
    env: &mut Environment,
    verbose: bool,
) -> ScriptResult {
    // The lexer handles # comments and \n as whitespace, and the parser
    // splits statements on ; and : terminators. So we can pass the entire
    // source to parse() directly.
    let stmts = match crate::parser::parse(source) {
        Ok(stmts) => stmts,
        Err(e) => return ScriptResult::ParseError(e.render(source)),
    };

    for stmt in &stmts {
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
                let msg = format!("{}", e);
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
/// Reads the entire file into memory and passes it to [`execute_source()`].
/// Returns `ScriptResult::EvalError` if the file cannot be read.
pub fn execute_file(
    path: &str,
    env: &mut Environment,
    verbose: bool,
) -> ScriptResult {
    match std::fs::read_to_string(path) {
        Ok(source) => execute_source(&source, env, verbose),
        Err(e) => ScriptResult::EvalError(format!("cannot read '{}': {}", path, e)),
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
    fn test_execute_source_eval_error() {
        let mut env = Environment::new();
        let result = execute_source("undefined_var", &mut env, false);
        assert!(matches!(result, ScriptResult::EvalError(_)));
    }

    #[test]
    fn test_execute_file_not_found() {
        let mut env = Environment::new();
        let result = execute_file("/nonexistent/path/script.qk", &mut env, false);
        assert!(matches!(result, ScriptResult::EvalError(_)));
        if let ScriptResult::EvalError(msg) = result {
            assert!(msg.contains("cannot read"));
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
