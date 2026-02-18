//! q-Kangaroo interactive REPL.
//!
//! Launches an interactive session with line editing (via rustyline),
//! persistent history, multi-line input via paren-counting, tab completion
//! (functions with auto-paren, commands, user variables), session commands,
//! and robust error recovery (parse errors, eval errors, and caught panics
//! never crash the loop).

use rustyline::config::Config;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{CompletionType, EditMode, Editor};

use qsym_cli::commands::{execute_command, parse_command, CommandResult};
use qsym_cli::environment::Environment;
use qsym_cli::repl::ReplHelper;

// ---------------------------------------------------------------------------
// ASCII banner
// ---------------------------------------------------------------------------

/// Print the welcome banner with ASCII kangaroo, version, and hint.
fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        r#"
      /\
     /  \     q-Kangaroo v{}
    | q> |    Symbolic q-series computation
    |    |
    /|  |\    Type 'help' for commands, 'quit' to exit
   (_|  |_)
"#,
        version
    );
}

// ---------------------------------------------------------------------------
// History file
// ---------------------------------------------------------------------------

/// Compute the history file path (next to the executable).
fn history_file_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".q_kangaroo_history")
}

// ---------------------------------------------------------------------------
// Main REPL loop
// ---------------------------------------------------------------------------

fn main() {
    print_banner();

    let config = Config::builder()
        .completion_type(CompletionType::Circular)
        .edit_mode(EditMode::Emacs)
        .auto_add_history(true)
        .max_history_size(10_000)
        .expect("valid max_history_size")
        .build();

    let helper = ReplHelper::new();
    let mut rl: Editor<ReplHelper, DefaultHistory> =
        Editor::with_config(config).expect("failed to create editor");
    rl.set_helper(Some(helper));

    let history_path = history_file_path();
    let _ = rl.load_history(&history_path);

    let mut env = Environment::new();

    loop {
        match rl.readline("q> ") {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Command dispatch (before parser)
                if let Some(cmd) = parse_command(trimmed) {
                    match execute_command(cmd, &mut env) {
                        CommandResult::Continue => continue,
                        CommandResult::Quit => break,
                        CommandResult::Output(text) => {
                            println!("{}", text);
                            continue;
                        }
                    }
                }

                // Parse and evaluate
                match qsym_cli::parser::parse(trimmed) {
                    Ok(stmts) => {
                        for stmt in &stmts {
                            match qsym_cli::eval::eval_stmt_safe(stmt, &mut env) {
                                Ok(Some(val)) => {
                                    println!("{}", qsym_cli::format::format_value(&val));
                                }
                                Ok(None) => {} // semicolon-suppressed or empty
                                Err(e) => eprintln!("{}", e),
                            }
                        }

                        // Update variable names in completer after eval
                        let var_names: Vec<String> = env.variables.keys().cloned().collect();
                        if let Some(helper) = rl.helper_mut() {
                            helper.update_var_names(var_names);
                        }
                    }
                    Err(e) => eprintln!("{}", e.render(trimmed)),
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: cancel current line, continue loop
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D: exit cleanly
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history on exit
    let _ = rl.save_history(&history_path);
}
