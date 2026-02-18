# Phase 30: Script Execution & CLI Flags - Research

**Researched:** 2026-02-18
**Domain:** CLI argument parsing, script execution, TTY detection, non-interactive mode
**Confidence:** HIGH

## Summary

Phase 30 adds three non-interactive execution modes to q-Kangaroo: script file execution (`q-kangaroo script.qk`), piped stdin (`echo "1+1" | q-kangaroo`), and expression evaluation (`q-kangaroo -c "expr"`). It also adds `--help`, `--quiet`, `--verbose`, and `--` flag handling, plus a `read("file.qk")` function for loading scripts within the REPL.

The current `main.rs` is a 157-line monolithic function that only handles `--version` before launching the interactive REPL. All new functionality fits naturally into the existing hand-rolled argument parsing approach (no external crate needed), the existing `parser::parse()` + `eval::eval_stmt_safe()` pipeline, and `std::io::IsTerminal` (stable since Rust 1.70, our toolchain is 1.85). The main challenge is factoring out a reusable "execute source text" function that can be shared between script files, `-c` expressions, piped stdin, and `read()`.

**Primary recommendation:** Create a new `script.rs` module providing `execute_source()` and `execute_file()` functions. Extend the lexer to handle `#` comments and `\n`/`\r` whitespace. Add string literal support (`Token::StringLit`) to the lexer/parser for `read("file.qk")` syntax. Hand-roll argument parsing in `main.rs` (~80 lines) extending the existing `--version` pattern.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| std::io::IsTerminal | Rust 1.70+ | Detect TTY for stdin/stdout | Stable in stdlib, no external crate needed |
| std::io::BufRead | Rust stable | Line-by-line reading of scripts/stdin | Standard approach for line-oriented input |
| std::fs::read_to_string | Rust stable | Read script files | Simplest API for whole-file reads |
| std::process::ExitCode | Rust 1.61+ | Structured exit codes from main | Allows destructors to run (unlike process::exit) |
| std::env::args | Rust stable | CLI argument iteration | Already used for --version |

### Supporting
No additional dependencies needed. All functionality is achievable with std alone.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled arg parsing | clap | Massive dependency (~100 crates) for ~6 flags; project convention is hand-rolled |
| std::io::IsTerminal | is-terminal crate | External dep unnecessary since Rust 1.70 stabilized IsTerminal |
| std::process::ExitCode | std::process::exit() | exit() skips destructors; ExitCode allows clean shutdown |
| std::fs::read_to_string | BufReader line-by-line | Whole-file read is simpler for small scripts; no streaming needed |

**Installation:**
```bash
# No new dependencies needed
```

## Architecture Patterns

### Recommended Module Changes
```
crates/qsym-cli/src/
  main.rs         # Refactor: argument parsing, mode dispatch
  script.rs       # NEW: execute_source(), execute_file(), execute_reader()
  lexer.rs        # Modify: add # comments, \n/\r whitespace, string literals
  token.rs        # Modify: add Token::StringLit(String)
  ast.rs          # Modify: add AstNode::StringLit(String)
  eval.rs         # Modify: add read() function dispatch
  parser.rs       # Modify: handle Token::StringLit in expr_bp prefix
  commands.rs     # Modify: add "read" as a session command (read file.qk)
  lib.rs          # Modify: add pub mod script
  repl.rs         # No changes needed
  environment.rs  # No changes needed
  error.rs        # No changes needed
  format.rs       # No changes needed
  help.rs         # No changes needed
```

### Pattern 1: Argument Parsing State Machine
**What:** Parse CLI args into a structured `CliMode` enum before dispatching
**When to use:** Extending the existing --version handling in main.rs

```rust
// Hand-rolled argument parsing (~80 lines)
enum CliMode {
    Interactive { quiet: bool, verbose: bool },
    Script { path: String, verbose: bool },
    Expression { expr: String, verbose: bool },
    Piped { verbose: bool },
    Help,
    Version,
}

fn parse_args() -> Result<CliMode, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut quiet = false;
    let mut verbose = false;
    let mut expr: Option<String> = None;
    let mut file: Option<String> = None;
    let mut dashdash = false;

    let mut i = 0;
    while i < args.len() {
        if dashdash {
            // After --, everything is a filename
            file = Some(args[i].clone());
            break;
        }
        match args[i].as_str() {
            "--help" | "-h" => return Ok(CliMode::Help),
            "--version" | "-V" => return Ok(CliMode::Version),
            "--quiet" | "-q" => quiet = true,
            "--verbose" | "-v" => verbose = true,
            "-c" => {
                i += 1;
                if i >= args.len() {
                    return Err("option '-c' requires an argument".into());
                }
                expr = Some(args[i].clone());
            }
            "--" => dashdash = true,
            arg if arg.starts_with('-') => {
                return Err(format!(
                    "unknown option '{}'\nTry 'q-kangaroo --help' for more information.",
                    arg
                ));
            }
            _ => {
                file = Some(args[i].clone());
                break;
            }
        }
        i += 1;
    }

    match (expr, file) {
        (Some(e), _) => Ok(CliMode::Expression { expr: e, verbose }),
        (_, Some(f)) => Ok(CliMode::Script { path: f, verbose }),
        _ => {
            // Check if stdin is a TTY
            use std::io::IsTerminal;
            if std::io::stdin().is_terminal() {
                Ok(CliMode::Interactive { quiet, verbose })
            } else {
                Ok(CliMode::Piped { verbose })
            }
        }
    }
}
```

### Pattern 2: Shared Execution Engine (script.rs)
**What:** A module providing reusable functions for non-interactive execution
**When to use:** All non-REPL modes share this code

```rust
use std::io::{self, BufRead};

use crate::environment::Environment;
use crate::error::ParseError;
use crate::eval::{self, Value};
use crate::format::format_value;

/// Result of executing a script/expression.
pub enum ScriptResult {
    /// All statements executed successfully.
    Success,
    /// A parse error occurred.
    ParseError(String),
    /// An eval error occurred.
    EvalError(String),
    /// A panic was caught.
    Panic(String),
}

/// Execute a source string (may contain multiple lines/statements).
///
/// Handles: # comments, multi-line paren continuation, semicolons.
/// Prints results of non-suppressed statements to stdout.
/// Returns on first error.
pub fn execute_source(
    source: &str,
    env: &mut Environment,
    verbose: bool,
) -> ScriptResult {
    // Strip # comments, join continuation lines, split on statement boundaries
    let cleaned = strip_comments(source);
    // Parse and eval each logical line
    for logical_line in split_logical_lines(&cleaned) {
        let trimmed = logical_line.trim();
        if trimmed.is_empty() { continue; }

        match crate::parser::parse(trimmed) {
            Ok(stmts) => {
                for stmt in &stmts {
                    let start = if verbose { Some(std::time::Instant::now()) } else { None };
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
                        Err(e) => return ScriptResult::EvalError(format!("{}", e)),
                    }
                }
            }
            Err(e) => return ScriptResult::ParseError(e.render(trimmed)),
        }
    }
    ScriptResult::Success
}

/// Execute a file by path. Returns appropriate ScriptResult.
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

/// Strip `#` line comments from source text.
fn strip_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| {
            if let Some(pos) = line.find('#') {
                &line[..pos]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Split source into logical lines, joining lines with unclosed parens.
fn split_logical_lines(source: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;

    for line in source.lines() {
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(line.trim());

        for ch in line.chars() {
            match ch {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                _ => {}
            }
        }

        if depth <= 0 {
            result.push(std::mem::take(&mut current));
            depth = 0;
        }
    }

    if !current.is_empty() {
        result.push(current);
    }
    result
}
```

### Pattern 3: Lexer Extension for Comments and Newlines
**What:** Extend the lexer to treat `#` as line-comment start and `\n`/`\r` as whitespace
**When to use:** Script files contain newlines and comments (unlike single REPL lines)

```rust
// In lexer.rs tokenize():
// Extend the whitespace skip at the top:
if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
    pos += 1;
    continue;
}

// Add comment handling (before single-character tokens):
if b == b'#' {
    // Skip to end of line
    while pos < bytes.len() && bytes[pos] != b'\n' {
        pos += 1;
    }
    continue;
}
```

**Important:** This is safe because the existing lexer comment at line 26 says "newlines won't appear in REPL single-line input" -- rustyline strips newlines before passing to the parser. Adding `\n`/`\r` as whitespace is backward-compatible.

### Pattern 4: String Literals for read() Function
**What:** Add `Token::StringLit(String)` to support `read("file.qk")` syntax
**When to use:** The read() function needs a filename argument

```rust
// In token.rs, add to Token enum:
/// String literal (double-quoted).
StringLit(String),

// In lexer.rs, add string literal handling:
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
                _ => { value.push(bytes[pos] as char); pos += 1; }
            }
        } else {
            value.push(bytes[pos] as char);
            pos += 1;
        }
    }
    if pos >= bytes.len() {
        return Err(ParseError::new("unterminated string literal", Span::new(start, pos)));
    }
    pos += 1; // skip closing quote
    tokens.push(SpannedToken {
        token: Token::StringLit(value),
        span: Span::new(start, pos),
    });
    continue;
}

// In ast.rs, add:
/// String literal value.
StringLit(String),

// In parser.rs expr_bp prefix section, add:
Token::StringLit(ref s) => {
    let s = s.clone();
    self.advance();
    AstNode::StringLit(s)
}

// In eval.rs, add Value::String and handle it:
// Add to Value enum:
/// String value (for filenames in read()).
String(String),

// In eval_expr for AstNode::StringLit:
AstNode::StringLit(s) => Ok(Value::String(s.clone())),
```

### Pattern 5: read() Function in Dispatch
**What:** Add `read("file.qk")` as a function call AND `read file.qk` as a session command
**When to use:** EXEC-06 requires loading scripts within the REPL

```rust
// In eval.rs dispatch(), add before the catch-all:
"read" => {
    expect_args(name, args, 1)?;
    match &args[0] {
        Value::String(path) => {
            // execute_file returns ScriptResult, convert to Value
            match crate::script::execute_file(path, env, false) {
                crate::script::ScriptResult::Success => Ok(Value::None),
                crate::script::ScriptResult::ParseError(msg) => {
                    Err(EvalError::Panic(msg))
                }
                crate::script::ScriptResult::EvalError(msg) => {
                    Err(EvalError::Panic(msg))
                }
                crate::script::ScriptResult::Panic(msg) => {
                    Err(EvalError::Panic(msg))
                }
            }
        }
        _ => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: 0,
            expected: "string",
            got: args[0].type_name().to_string(),
        }),
    }
}

// In commands.rs, add "read" as a session command:
"read" => {
    if trimmed.contains('(') {
        return None; // read("file") is a function call, pass to parser
    }
    if words.len() == 2 {
        Some(Command::Read(words[1].to_string()))
    } else {
        None
    }
}
```

### Pattern 6: Main Function Refactoring
**What:** Restructure main() into mode dispatch
**When to use:** All modes need to be dispatched from main

```rust
use std::process::ExitCode;

fn main() -> ExitCode {
    match parse_args() {
        Err(msg) => {
            eprintln!("q-kangaroo: {}", msg);
            ExitCode::from(2)
        }
        Ok(CliMode::Help) => {
            print_usage();
            ExitCode::SUCCESS
        }
        Ok(CliMode::Version) => {
            println!("q-kangaroo {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(CliMode::Expression { expr, verbose }) => {
            run_expression(&expr, verbose)
        }
        Ok(CliMode::Script { path, verbose }) => {
            run_script(&path, verbose)
        }
        Ok(CliMode::Piped { verbose }) => {
            run_piped(verbose)
        }
        Ok(CliMode::Interactive { quiet, verbose }) => {
            run_interactive(quiet, verbose);
            ExitCode::SUCCESS
        }
    }
}
```

### Anti-Patterns to Avoid
- **Don't use `process::exit()` in library code:** It skips destructors. Return `ExitCode` from `main()` instead.
- **Don't parse args after TTY detection:** Parse all flags first, then check TTY only if no explicit mode was selected.
- **Don't duplicate the eval loop:** Factor out `execute_source()` and reuse it everywhere.
- **Don't add `#` comment handling in script.rs pre-processing AND in the lexer:** Handle it in one place (the lexer) for consistency.
- **Don't read script files line-by-line and parse each line:** Read the whole file, strip comments, join continuation lines, then parse logical lines. This correctly handles multi-line expressions.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TTY detection | Custom platform-specific code | `std::io::IsTerminal` | Cross-platform, stable since Rust 1.70 |
| Exit codes | Magic numbers | Named constants (module-level `const`) | Readability, sysexits convention |
| Argument parsing | Complex state machine | Simple ~80-line hand-rolled parser | Only 6 flags; clap would be 100+ transitive deps |

**Key insight:** The project's convention is to hand-roll rather than add dependencies, but use std library types where available. `IsTerminal` and `ExitCode` are both stable and require zero new deps.

## Common Pitfalls

### Pitfall 1: Lexer Regression with Newline Handling
**What goes wrong:** Adding `\n` as whitespace in the lexer could break the REPL multi-line validator
**Why it happens:** The REPL's rustyline validator counts brackets to decide when input is incomplete, then joins the multi-line input into a single string. rustyline includes `\n` in the joined input.
**How to avoid:** This is actually safe because the lexer already treats `\n` as an unknown character error. Making it whitespace is strictly more permissive. The rustyline Validator works on the raw input *before* tokenization (it just counts brackets), so it's unaffected.
**Warning signs:** If REPL multi-line stops working after the lexer change, check the validator.

### Pitfall 2: Comment Handling Inside Strings
**What goes wrong:** If you strip `#` comments at the source level before lexing, you could corrupt string literals containing `#`
**Why it happens:** Naive `line.find('#')` doesn't respect string boundaries
**How to avoid:** Handle `#` in the lexer (after string literal processing), NOT in a pre-processing step. If using the pre-processing approach in script.rs (for join-continuation logic), be aware strings are not yet supported so this is safe *for now*, but the lexer approach is more future-proof.
**Warning signs:** Test `read("file#1.qk")` to verify `#` inside strings is preserved.

### Pitfall 3: Script Continuation Lines and Statement Boundaries
**What goes wrong:** A multi-line statement like `aqprod(q, q,\n  infinity, 20)` must be joined before parsing
**Why it happens:** The current `parser::parse()` expects a single logical line with no newlines acting as statement terminators
**How to avoid:** With the lexer treating `\n` as whitespace, the entire file can be passed to `parse()` as one string. The parser already splits on `;` and `:` terminators. Multi-line expressions with unclosed parens work naturally because `\n` is just whitespace. No explicit line-joining is needed if the lexer approach is used.
**Warning signs:** If parsing fails on multi-line scripts, check that `\n` is treated as whitespace in the lexer.

### Pitfall 4: Exit Code Semantics
**What goes wrong:** Returning wrong exit codes confuses shell scripts
**Why it happens:** Mixing up "parse error" vs "eval error" vs "usage error" exit codes
**How to avoid:** Use sysexits-compatible constants: 0=success, 1=general eval error, 2=usage error (bad flags), 65=data error (parse error), 66=no input (file not found), 74=I/O error. Document them clearly.
**Warning signs:** Test `echo $?` after each error type.

### Pitfall 5: Piped Input with Empty Trailing Newline
**What goes wrong:** `echo "1+1" | q-kangaroo` sends "1+1\n" which could cause trailing empty statement
**Why it happens:** `echo` appends a newline
**How to avoid:** Trim whitespace from read lines, skip empty lines. The lexer treating `\n` as whitespace handles this naturally.
**Warning signs:** Test `printf "1+1" | q-kangaroo` (no trailing newline) vs `echo "1+1" | q-kangaroo`.

### Pitfall 6: read() Circular Inclusion
**What goes wrong:** A script calls `read("itself.qk")` creating infinite recursion
**Why it happens:** No include-depth tracking
**How to avoid:** For v1 scope, don't add circular detection -- just let it stack overflow and be caught by `catch_unwind`. Document the limitation. A depth counter can be added later.
**Warning signs:** Stack overflow in read() tests.

### Pitfall 7: String Literal Token Breaking Existing Tests
**What goes wrong:** Adding `Token::StringLit` variant breaks `PartialEq` exhaustive matches
**Why it happens:** Test assertions using exact token lists will still work since `#[derive(PartialEq)]` handles the new variant automatically. But `token_name()` helper in parser.rs needs updating.
**How to avoid:** Add the `Token::StringLit` case to `token_name()` and any match expressions.
**Warning signs:** Compiler errors about non-exhaustive patterns.

## Code Examples

### TTY Detection
```rust
// Source: https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html
use std::io::IsTerminal;

let is_interactive = std::io::stdin().is_terminal();
```

### ExitCode from Main
```rust
// Source: https://doc.rust-lang.org/std/process/struct.ExitCode.html
use std::process::ExitCode;

fn main() -> ExitCode {
    // ...
    ExitCode::from(2u8) // usage error
}
```

### Buffered Stdin Reading (for piped mode)
```rust
// Source: https://doc.rust-lang.org/std/io/trait.BufRead.html
use std::io::{self, BufRead};

fn run_piped(verbose: bool) -> ExitCode {
    let stdin = io::stdin();
    let source: String = stdin.lock().lines()
        .map(|l| l.unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    let mut env = Environment::new();
    match execute_source(&source, &mut env, verbose) {
        ScriptResult::Success => ExitCode::SUCCESS,
        ScriptResult::ParseError(msg) => {
            eprintln!("{}", msg);
            ExitCode::from(65)
        }
        ScriptResult::EvalError(msg) => {
            eprintln!("{}", msg);
            ExitCode::from(1)
        }
        ScriptResult::Panic(msg) => {
            eprintln!("{}", msg);
            ExitCode::from(70)
        }
    }
}
```

### Help Text Format
```rust
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
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| is-terminal crate | std::io::IsTerminal | Rust 1.70 (June 2023) | No external dep needed |
| `std::process::exit()` | `ExitCode` from main | Rust 1.61 (May 2022) | Clean shutdown, destructors run |
| atty crate | std::io::IsTerminal | Rust 1.70 (June 2023) | atty was the old standard, now deprecated |

**Deprecated/outdated:**
- `atty` crate: Superseded by `std::io::IsTerminal` in Rust 1.70
- `is-terminal` crate: Was the bridge before stdlib stabilization, now unnecessary

## Exit Code Constants

| Code | Constant Name | Meaning | sysexits Equivalent |
|------|---------------|---------|---------------------|
| 0 | EXIT_SUCCESS | Normal termination | EX_OK |
| 1 | EXIT_EVAL_ERROR | Evaluation error | EX_SOFTWARE |
| 2 | EXIT_USAGE | Bad CLI usage (unknown flag, missing arg) | EX_USAGE |
| 65 | EXIT_PARSE_ERROR | Parse error in input | EX_DATAERR |
| 66 | EXIT_FILE_NOT_FOUND | Script file not found | EX_NOINPUT |
| 70 | EXIT_PANIC | Caught panic | EX_SOFTWARE |
| 74 | EXIT_IO_ERROR | I/O error reading file | EX_IOERR |

```rust
// Define as module-level constants in script.rs or main.rs
pub const EXIT_SUCCESS: u8 = 0;
pub const EXIT_EVAL_ERROR: u8 = 1;
pub const EXIT_USAGE: u8 = 2;
pub const EXIT_PARSE_ERROR: u8 = 65;
pub const EXIT_FILE_NOT_FOUND: u8 = 66;
pub const EXIT_PANIC: u8 = 70;
pub const EXIT_IO_ERROR: u8 = 74;
```

## Design Decisions

### Comment-Stripping: Lexer vs Pre-processor
**Decision: Handle `#` comments in the lexer.**

Rationale:
- The lexer is the single source of truth for tokenization
- If string literals are added, the lexer naturally handles `#` inside strings
- The REPL multi-line validator is unaffected (it only counts brackets)
- The alternative (pre-processing in script.rs) works for now but would break if strings with `#` were ever needed

### read() Syntax: Function Call vs Session Command
**Decision: Support BOTH `read("file.qk")` as a function call AND `read file.qk` as a session command.**

Rationale:
- `read("file.qk")` is the canonical Maple-style syntax, needs string literals in the parser
- `read file.qk` follows the same pattern as `save file.txt` (existing command), easier to type
- The session command form avoids needing string literals for simple cases
- `read(expr)` where expr is a variable holding a string could be useful later

### Newline Handling: Lexer Whitespace vs Line Joining
**Decision: Treat `\n`/`\r` as whitespace in the lexer.**

Rationale:
- The parser already splits statements on `;` and `:` terminators
- With newlines as whitespace, multi-line expressions like `f(\n  1, 2\n)` parse naturally
- No need for explicit line-joining logic in script.rs
- Backward-compatible: REPL never sent newlines to the lexer before (rustyline joins internally)
- Simpler implementation than the alternative (joining lines based on paren depth in script.rs)

### Verbose Timing: stdout vs stderr
**Decision: Print timing info to stderr.**

Rationale:
- Timing is diagnostic, not output
- Keeps stdout clean for piping results to other programs
- Consistent with how compilers report timing (e.g., `time` command uses stderr)

## Open Questions

1. **read() and Variable Scope**
   - What we know: The REPL's Environment is mutable. read() executes in the same Environment, so variables defined in a read file are visible afterward.
   - What's unclear: Should read() return the last evaluated value, or always return None?
   - Recommendation: Return None (like Maple's `read`). The script's side effects (variable assignments) are the primary purpose.

2. **Script Error Behavior**
   - What we know: EXEC-01 says "execute and exit." The milestone research says exit code 1 for eval errors.
   - What's unclear: Should the script stop on first error, or continue and report all errors?
   - Recommendation: Stop on first error (fail-fast). This is the standard behavior for scripting languages.

3. **String Literal Scope**
   - What we know: We need strings for `read("file.qk")`. Adding `Token::StringLit` and `Value::String` is minimal.
   - What's unclear: Should string literals be usable in other contexts (e.g., variable assignment `s := "hello"`)?
   - Recommendation: Yes, implement them generically. The infrastructure is the same cost. But don't add string operations (concatenation, etc.) -- that's future scope.

## Testing Strategy

### Unit Tests (in-module #[cfg(test)])
- Lexer: `#` comment stripping, `\n`/`\r` as whitespace, string literal tokenization
- Parser: string literal parsing, multi-line expression parsing
- Eval: `read()` dispatch, `Value::String` handling
- Commands: `read file.qk` session command parsing

### Integration Tests (tests/ directory or main.rs tests)
- Argument parsing: all flag combinations, unknown flags, `--` separator
- Script execution: file with comments, multi-line, multiple statements
- Piped input: expression evaluation, banner suppression
- `-c` mode: expression evaluation, error handling
- `--help` output verification
- Exit code verification (requires Command::new for subprocess testing)

### Subprocess Tests (most reliable for CLI testing)
```rust
use std::process::Command;

#[test]
fn test_c_flag_evaluates_expression() {
    let output = Command::new(env!("CARGO_BIN_EXE_q-kangaroo"))
        .args(["-c", "1 + 1"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "2");
}

#[test]
fn test_unknown_flag_exit_code_2() {
    let output = Command::new(env!("CARGO_BIN_EXE_q-kangaroo"))
        .arg("--badopt")
        .output()
        .expect("failed to run");
    assert_eq!(output.status.code(), Some(2));
}
```

**Note:** `env!("CARGO_BIN_EXE_q-kangaroo")` resolves to the test binary path. This works in integration tests (`tests/` directory) but NOT in unit tests. For subprocess tests, create `tests/cli_integration.rs`.

## Sources

### Primary (HIGH confidence)
- Codebase inspection: `crates/qsym-cli/src/main.rs` (157 lines, current entry point)
- Codebase inspection: `crates/qsym-cli/src/lexer.rs` (tokenizer, no newline/comment support)
- Codebase inspection: `crates/qsym-cli/src/parser.rs` (Pratt parser, `parse()` entry point)
- Codebase inspection: `crates/qsym-cli/src/eval.rs` (~2400 lines, `dispatch()` function with match arms)
- Codebase inspection: `crates/qsym-cli/src/commands.rs` (session commands, `parse_command()`)
- Codebase inspection: `crates/qsym-cli/src/environment.rs` (Environment struct, variables)
- [IsTerminal docs](https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html) - stabilized Rust 1.70
- [ExitCode docs](https://doc.rust-lang.org/std/process/struct.ExitCode.html) - stabilized Rust 1.61

### Secondary (MEDIUM confidence)
- [BufRead docs](https://doc.rust-lang.org/std/io/trait.BufRead.html) - line-by-line reading
- [Rust CLI exit codes](https://rust-cli.github.io/book/in-depth/exit-code.html) - sysexits convention
- [sysexits.h convention](https://github.com/benwilber/exitcode) - exit code values

### Tertiary (LOW confidence)
- None -- all claims verified against codebase and official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all std library, verified against Rust 1.85 toolchain
- Architecture: HIGH - based on direct codebase inspection, patterns match existing conventions
- Pitfalls: HIGH - identified through codebase analysis and understanding of lexer/parser interaction
- Exit codes: MEDIUM - sysexits convention is well-established but exact mapping is a design choice

**Research date:** 2026-02-18
**Valid until:** 2026-04-18 (stable domain, no fast-moving dependencies)
