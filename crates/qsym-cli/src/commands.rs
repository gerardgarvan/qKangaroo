//! Session commands for the q-Kangaroo REPL.
//!
//! Handles built-in commands (`quit`, `exit`, `clear`, `set precision`, `help`)
//! that are intercepted before the expression parser. Only bare command patterns
//! are matched -- lines containing `:=` or function-call syntax fall through to
//! the parser.

use crate::environment::Environment;
use crate::help;

// ---------------------------------------------------------------------------
// Command enum
// ---------------------------------------------------------------------------

/// A parsed REPL command (distinct from an expression).
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Display help, optionally for a specific topic.
    Help(Option<String>),
    /// Set the default truncation order.
    SetPrecision(i64),
    /// Clear all variables and reset session state.
    Clear,
    /// Exit the REPL.
    Quit,
}

// ---------------------------------------------------------------------------
// CommandResult enum
// ---------------------------------------------------------------------------

/// The result of executing a command.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    /// No output, continue the REPL loop.
    Continue,
    /// Exit the REPL.
    Quit,
    /// Print this string and continue the REPL loop.
    Output(String),
}

// ---------------------------------------------------------------------------
// parse_command
// ---------------------------------------------------------------------------

/// Try to parse the input line as a built-in command.
///
/// Returns `Some(Command)` if the line matches a command pattern, or `None`
/// if it should be passed to the expression parser.
///
/// **Anti-pattern avoidance:** Lines containing `:=` (assignment) are never
/// treated as commands. Lines where a command keyword is followed by `(` are
/// assumed to be function calls and passed to the parser.
pub fn parse_command(line: &str) -> Option<Command> {
    let trimmed = line.trim();

    // Never intercept assignments
    if trimmed.contains(":=") {
        return None;
    }

    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    let first = words[0].to_lowercase();

    match first.as_str() {
        "quit" | "exit" => {
            // Only bare quit/exit (no extra args that look like expressions)
            if words.len() == 1 {
                Some(Command::Quit)
            } else {
                None
            }
        }
        "clear" => {
            // Only bare "clear" -- if followed by `(` it's a function call
            if words.len() == 1 && !trimmed.contains('(') {
                Some(Command::Clear)
            } else {
                None
            }
        }
        "help" => {
            // "help" or "help <topic>" but not "help(...)" (function call)
            if trimmed.contains('(') {
                return None;
            }
            if words.len() == 1 {
                Some(Command::Help(None))
            } else if words.len() == 2 {
                Some(Command::Help(Some(words[1].to_string())))
            } else {
                None
            }
        }
        "set" => {
            if words.len() >= 2 && words[1].to_lowercase() == "precision" {
                if words.len() == 3 {
                    match words[2].parse::<i64>() {
                        Ok(n) => Some(Command::SetPrecision(n)),
                        Err(_) => Some(Command::SetPrecision(-1)), // signal error
                    }
                } else if words.len() == 2 {
                    // "set precision" with no number
                    Some(Command::SetPrecision(-1))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// execute_command
// ---------------------------------------------------------------------------

/// Execute a parsed command, modifying the environment as needed.
pub fn execute_command(cmd: Command, env: &mut Environment) -> CommandResult {
    match cmd {
        Command::Quit => CommandResult::Quit,
        Command::Clear => {
            env.reset();
            CommandResult::Output("Session cleared.".to_string())
        }
        Command::SetPrecision(n) if n > 0 => {
            env.default_order = n;
            CommandResult::Output(format!("Truncation order set to {}.", n))
        }
        Command::SetPrecision(_) => CommandResult::Output(
            "Error: precision must be a positive integer. Usage: set precision N".to_string(),
        ),
        Command::Help(None) => CommandResult::Output(help::general_help()),
        Command::Help(Some(topic)) => match help::function_help(&topic) {
            Some(text) => CommandResult::Output(text),
            None => CommandResult::Output(format!(
                "Unknown function '{}'. Type 'help' for a list of available functions.",
                topic
            )),
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- parse_command tests ------------------------------------------------

    #[test]
    fn parse_quit() {
        assert_eq!(parse_command("quit"), Some(Command::Quit));
    }

    #[test]
    fn parse_exit() {
        assert_eq!(parse_command("exit"), Some(Command::Quit));
    }

    #[test]
    fn parse_quit_case_insensitive() {
        assert_eq!(parse_command("QUIT"), Some(Command::Quit));
        assert_eq!(parse_command("Quit"), Some(Command::Quit));
        assert_eq!(parse_command("EXIT"), Some(Command::Quit));
    }

    #[test]
    fn parse_clear() {
        assert_eq!(parse_command("clear"), Some(Command::Clear));
    }

    #[test]
    fn parse_clear_case_insensitive() {
        assert_eq!(parse_command("CLEAR"), Some(Command::Clear));
        assert_eq!(parse_command("Clear"), Some(Command::Clear));
    }

    #[test]
    fn parse_clear_assignment_passthrough() {
        // "clear := 5" is an assignment, not a command
        assert_eq!(parse_command("clear := 5"), None);
    }

    #[test]
    fn parse_help_bare() {
        assert_eq!(parse_command("help"), Some(Command::Help(None)));
    }

    #[test]
    fn parse_help_topic() {
        assert_eq!(
            parse_command("help aqprod"),
            Some(Command::Help(Some("aqprod".to_string())))
        );
    }

    #[test]
    fn parse_help_assignment_passthrough() {
        assert_eq!(parse_command("help := 42"), None);
    }

    #[test]
    fn parse_help_function_call_passthrough() {
        assert_eq!(parse_command("help(x)"), None);
    }

    #[test]
    fn parse_set_precision_valid() {
        assert_eq!(
            parse_command("set precision 50"),
            Some(Command::SetPrecision(50))
        );
    }

    #[test]
    fn parse_set_precision_case_insensitive() {
        assert_eq!(
            parse_command("SET PRECISION 30"),
            Some(Command::SetPrecision(30))
        );
    }

    #[test]
    fn parse_set_precision_invalid_value() {
        // "set precision foo" -- intent is clear, signal error
        assert_eq!(
            parse_command("set precision foo"),
            Some(Command::SetPrecision(-1))
        );
    }

    #[test]
    fn parse_set_precision_missing_value() {
        // "set precision" with no number
        assert_eq!(
            parse_command("set precision"),
            Some(Command::SetPrecision(-1))
        );
    }

    #[test]
    fn parse_expression_passthrough() {
        // Regular expressions pass through to parser
        assert_eq!(parse_command("aqprod(q,q,5,20)"), None);
    }

    #[test]
    fn parse_assignment_passthrough() {
        assert_eq!(parse_command("f := 42"), None);
    }

    #[test]
    fn parse_empty_line() {
        assert_eq!(parse_command(""), None);
        assert_eq!(parse_command("   "), None);
    }

    #[test]
    fn parse_whitespace_trimming() {
        assert_eq!(parse_command("  quit  "), Some(Command::Quit));
        assert_eq!(parse_command("  set  precision  50  "), Some(Command::SetPrecision(50)));
    }

    // -- execute_command tests ----------------------------------------------

    #[test]
    fn execute_quit() {
        let mut env = Environment::new();
        assert_eq!(execute_command(Command::Quit, &mut env), CommandResult::Quit);
    }

    #[test]
    fn execute_clear_resets_environment() {
        let mut env = Environment::new();
        use qsym_core::number::QInt;
        use crate::eval::Value;
        env.set_var("x", Value::Integer(QInt::from(42i64)));
        env.default_order = 50;
        env.last_result = Some(Value::Integer(QInt::from(7i64)));

        let result = execute_command(Command::Clear, &mut env);
        assert_eq!(result, CommandResult::Output("Session cleared.".to_string()));
        assert!(env.variables.is_empty());
        assert!(env.last_result.is_none());
        assert_eq!(env.default_order, 20);
    }

    #[test]
    fn execute_set_precision_valid() {
        let mut env = Environment::new();
        let result = execute_command(Command::SetPrecision(50), &mut env);
        assert_eq!(
            result,
            CommandResult::Output("Truncation order set to 50.".to_string())
        );
        assert_eq!(env.default_order, 50);
    }

    #[test]
    fn execute_set_precision_invalid() {
        let mut env = Environment::new();
        let result = execute_command(Command::SetPrecision(-1), &mut env);
        assert!(matches!(result, CommandResult::Output(ref s) if s.contains("Error")));
        // default_order should NOT be changed
        assert_eq!(env.default_order, 20);
    }

    #[test]
    fn execute_set_precision_zero() {
        let mut env = Environment::new();
        let result = execute_command(Command::SetPrecision(0), &mut env);
        assert!(matches!(result, CommandResult::Output(ref s) if s.contains("Error")));
        assert_eq!(env.default_order, 20);
    }

    #[test]
    fn execute_help() {
        let mut env = Environment::new();
        let result = execute_command(Command::Help(None), &mut env);
        assert!(matches!(result, CommandResult::Output(_)));
    }

    #[test]
    fn execute_help_topic() {
        let mut env = Environment::new();
        let result = execute_command(Command::Help(Some("aqprod".to_string())), &mut env);
        assert!(matches!(result, CommandResult::Output(_)));
    }
}
