//! q-Kangaroo: symbolic q-series computation.
//!
//! Supports multiple modes of operation:
//!
//! - **Interactive REPL:** Line editing (rustyline), persistent history,
//!   multi-line input, tab completion, session commands, error recovery.
//! - **Script execution:** `q-kangaroo script.qk`
//! - **Expression evaluation:** `q-kangaroo -c "expr"`
//! - **Piped input:** `echo "expr" | q-kangaroo`

use std::io::{self, BufRead, IsTerminal};
use std::process::ExitCode;

use rustyline::config::Config;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{CompletionType, EditMode, Editor};

use qsym_cli::commands::{execute_command, parse_command, CommandResult};
use qsym_cli::environment::Environment;
use qsym_cli::repl::ReplHelper;
use qsym_cli::script;

// ---------------------------------------------------------------------------
// CLI mode enum
// ---------------------------------------------------------------------------

/// Parsed CLI mode of operation.
enum CliMode {
    Interactive { quiet: bool, verbose: bool },
    Script { path: String, verbose: bool },
    Expression { expr: String, verbose: bool },
    Piped { verbose: bool },
    Help,
    Version,
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

/// Parse command-line arguments into a [`CliMode`].
///
/// Implements a hand-written state machine that handles:
/// - `--help` / `-h` -> Help
/// - `--version` / `-V` -> Version
/// - `--quiet` / `-q` -> quiet flag (interactive only)
/// - `--verbose` / `-v` -> verbose flag (all modes)
/// - `-c EXPRESSION` -> Expression mode
/// - `--` -> end of options, next positional is filename
/// - Unknown flags -> error with `--help` suggestion
/// - Positional arg -> Script filename
/// - No args + TTY -> Interactive; No args + pipe -> Piped
fn parse_args() -> Result<CliMode, String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();

    let mut quiet = false;
    let mut verbose = false;
    let mut expr: Option<String> = None;
    let mut file: Option<String> = None;
    let mut dashdash = false;

    let mut i = 0;
    while i < raw.len() {
        let arg = &raw[i];

        if dashdash {
            // After `--`, treat everything as a filename
            file = Some(arg.clone());
            break;
        }

        match arg.as_str() {
            "--help" | "-h" => return Ok(CliMode::Help),
            "--version" | "-V" => return Ok(CliMode::Version),
            "--quiet" | "-q" => quiet = true,
            "--verbose" | "-v" => verbose = true,
            "-c" => {
                i += 1;
                if i >= raw.len() {
                    return Err("option '-c' requires an argument\nTry 'q-kangaroo --help' for more information.".to_string());
                }
                expr = Some(raw[i].clone());
            }
            "--" => {
                dashdash = true;
            }
            s if s.starts_with('-') => {
                return Err(format!(
                    "unknown option '{}'\nTry 'q-kangaroo --help' for more information.",
                    arg
                ));
            }
            _ => {
                // Positional argument: filename
                file = Some(arg.clone());
                break;
            }
        }

        i += 1;
    }

    if let Some(e) = expr {
        Ok(CliMode::Expression { expr: e, verbose })
    } else if let Some(path) = file {
        Ok(CliMode::Script { path, verbose })
    } else if io::stdin().is_terminal() {
        Ok(CliMode::Interactive { quiet, verbose })
    } else {
        Ok(CliMode::Piped { verbose })
    }
}

// ---------------------------------------------------------------------------
// Help text
// ---------------------------------------------------------------------------

/// Print the usage / help text and return.
fn print_usage() {
    let version = env!("CARGO_PKG_VERSION");
    println!("q-kangaroo {} -- symbolic q-series computation", version);
    println!();
    println!("USAGE:");
    println!("  q-kangaroo [OPTIONS] [FILE]");
    println!("  q-kangaroo -c EXPRESSION");
    println!("  command | q-kangaroo");
    println!();
    println!("OPTIONS:");
    println!("  -h, --help       Show this help message and exit");
    println!("  -V, --version    Show version and exit");
    println!("  -c EXPRESSION    Evaluate expression and exit");
    println!("  -q, --quiet      Suppress banner in interactive mode");
    println!("  -v, --verbose    Show per-statement timing");
    println!("  --               End of options (treat next arg as filename)");
    println!();
    println!("EXAMPLES:");
    println!("  q-kangaroo script.qk         Execute a script file");
    println!("  q-kangaroo -c \"etaq(1,1,20)\"  Evaluate an expression");
    println!("  echo \"1+1\" | q-kangaroo       Pipe input");
    println!();
    println!("In interactive mode, type 'help' for available functions.");
}

// ---------------------------------------------------------------------------
// ASCII banner
// ---------------------------------------------------------------------------

/// Print the welcome banner with ASCII kangaroo, version, and hint.
fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        r#"
                                      /)
                                    '  \
                                   /    \
                                 /'    q \
                               /'         \        q-Kangaroo v{}
                          _.-'      /`-.   )       Symbolic q-series computation
               __.-'"""""'         /    `-'
             /'                   (  _._           Type 'help' for commands
            /       __      (       ___ \          'quit' to exit
          /'          \      \.____/   \_)
         /             \   \  |
        /   __          |   \_)
       |   /  \_        |  _/
       |  /     )      / /'
      .' |    _/      /_/
      |  |  /'     __/'
      |  |  \    /'
      |  |   \   \
     /  /     \   \
  __/' _/       \  `----.
 (___/'          `------'
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
// Mode runners
// ---------------------------------------------------------------------------

/// Evaluate a single expression and exit.
fn run_expression(expr: &str, verbose: bool) -> ExitCode {
    let mut env = Environment::new();
    let result = script::execute_source(expr, &mut env, verbose);
    if let Some(msg) = result.error_message() {
        eprintln!("{}", msg);
    }
    ExitCode::from(result.exit_code())
}

/// Execute a script file and exit.
fn run_script(path: &str, verbose: bool) -> ExitCode {
    let mut env = Environment::new();
    let result = script::execute_file(path, &mut env, verbose);
    if let Some(msg) = result.error_message() {
        eprintln!("{}", msg);
    }
    ExitCode::from(result.exit_code())
}

/// Read all piped stdin, evaluate, and exit.
fn run_piped(verbose: bool) -> ExitCode {
    let stdin = io::stdin();
    let source: String = stdin
        .lock()
        .lines()
        .map(|l| l.unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    let mut env = Environment::new();
    let result = script::execute_source(&source, &mut env, verbose);
    if let Some(msg) = result.error_message() {
        eprintln!("{}", msg);
    }
    ExitCode::from(result.exit_code())
}

/// Run the interactive REPL with line editing, history, and tab completion.
fn run_interactive(quiet: bool, verbose: bool) {
    if !quiet {
        print_banner();
    }

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

                // Handle CLI-style flags typed at REPL prompt (UAT feedback)
                if trimmed == "--help" || trimmed == "-h" {
                    print_usage();
                    continue;
                }
                if trimmed == "--version" || trimmed == "-V" {
                    println!("q-kangaroo {}", env!("CARGO_PKG_VERSION"));
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
                        CommandResult::ReadFile(path) => {
                            let result = script::execute_file(&path, &mut env, verbose);
                            if let Some(msg) = result.error_message() {
                                eprintln!("{}", msg);
                            }
                            // Update var names after script execution
                            let var_names: Vec<String> =
                                env.variables.keys().cloned().collect();
                            if let Some(helper) = rl.helper_mut() {
                                helper.update_var_names(var_names);
                            }
                            continue;
                        }
                    }
                }

                // Parse and evaluate
                match qsym_cli::parser::parse(trimmed) {
                    Ok(stmts) => {
                        for stmt in &stmts {
                            let start = if verbose {
                                Some(std::time::Instant::now())
                            } else {
                                None
                            };
                            match qsym_cli::eval::eval_stmt_safe(stmt, &mut env) {
                                Ok(Some(val)) => {
                                    println!("{}", qsym_cli::format::format_value(&val));
                                    if let Some(t) = start {
                                        eprintln!("  [{:.3}s]", t.elapsed().as_secs_f64());
                                    }
                                }
                                Ok(None) => {
                                    if let Some(t) = start {
                                        eprintln!("  [{:.3}s]", t.elapsed().as_secs_f64());
                                    }
                                }
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

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

fn main() -> ExitCode {
    match parse_args() {
        Err(msg) => {
            eprintln!("q-kangaroo: {}", msg);
            ExitCode::from(script::EXIT_USAGE)
        }
        Ok(CliMode::Help) => {
            print_usage();
            ExitCode::SUCCESS
        }
        Ok(CliMode::Version) => {
            println!("q-kangaroo {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(CliMode::Expression { expr, verbose }) => run_expression(&expr, verbose),
        Ok(CliMode::Script { path, verbose }) => run_script(&path, verbose),
        Ok(CliMode::Piped { verbose }) => run_piped(verbose),
        Ok(CliMode::Interactive { quiet, verbose }) => {
            run_interactive(quiet, verbose);
            ExitCode::SUCCESS
        }
    }
}
