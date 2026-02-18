# Phase 26: REPL Shell & Session - Research

**Researched:** 2026-02-17
**Domain:** Interactive REPL with line editing, completion, help, session commands
**Confidence:** HIGH

## Summary

Phase 26 wires up the interactive REPL loop for q-Kangaroo. The parser (Phase 24) and evaluator (Phase 25) are complete with 81 function dispatch arms, panic-catching `eval_stmt_safe`, and caret-style `ParseError::render`. The `main.rs` is currently a placeholder printing "q-Kangaroo REPL (not yet implemented)". This phase adds rustyline-based line editing, custom tab completion cycling through canonical function names (with auto-inserted parenthesis), a help system with per-function docs, and session commands (`set precision N`, `clear`, `quit`/`exit`).

The standard library for this in Rust is **rustyline** (v17.0.1), a mature readline implementation with 25M+ downloads. It provides exactly the three features we need out of the box: `CompletionType::Circular` for zsh-style Tab cycling, the `Validator` trait for multi-line input detection, and file-based history persistence. The `rustyline-derive` crate provides derive macros to reduce boilerplate for the `Helper` composite trait.

**Primary recommendation:** Use rustyline 17.0.1 with `CompletionType::Circular`, a custom `Completer` that returns `Pair { display: "aqprod", replacement: "aqprod(" }` for function names, and `MatchingBracketValidator` for auto-detecting unclosed parentheses.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Prompt: `q> ` (short and clean)
- Welcome banner with ASCII art kangaroo, version number, and hint ("Type 'help' for commands, 'quit' to exit")
- History file stored alongside the executable (portable -- history travels with the binary)
- Cycle through candidates with Tab (zsh-style), not bash-style list display
- Tab-completing a function name auto-inserts opening parenthesis: `aqp` -> `aqprod(`
- Canonical function names only in completions -- no Maple aliases in Tab candidates (aliases still work when typed)
- Bare `help` shows grouped function list organized by category (Products, Partitions, Theta, Analysis, Relations, Hypergeometric, Mock Theta/Bailey, Identity Proving) with one-line descriptions
- `help` also includes a separate "Commands" section at bottom listing session commands (set, clear, quit, help, latex, save)
- Per-function help (`help aqprod`) shows signature + description + usage example with sample output
- No Maple alias mentions in help text -- keep it clean
- `clear` resets everything: variables, %, and precision back to default (20)
- `quit`, `exit`, and Ctrl-D all exit the REPL
- Session commands: set precision, clear, quit/exit, help -- no additional commands needed

### Claude's Discretion
- Multi-line input approach
- Whether tab completes user-defined variables (in addition to functions)
- `set precision N` retroactive behavior

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope. Note: `latex` and `save` commands are Phase 27 scope.
</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rustyline | 17.0.1 | Line editing, history, tab completion | 25M+ downloads, battle-tested readline for Rust, supports Windows |
| rustyline-derive | latest | Derive macros for Helper/Completer/Hinter/Highlighter/Validator | Reduces boilerplate for composite Helper trait |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none needed) | - | - | All other dependencies already in qsym-cli |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| rustyline | reedline (nushell) | More features but heavier, async-oriented, less simple for our needs |
| rustyline | linenoise-rs | Less maintained, fewer features |

**Installation (add to `crates/qsym-cli/Cargo.toml`):**
```toml
[dependencies]
rustyline = { version = "17.0", features = ["derive"] }
```

The `derive` feature enables the `rustyline-derive` proc macros. Default features include `custom-bindings`, `with-dirs`, and `with-file-history`.

**Windows compatibility:** rustyline uses `windows-sys` 0.61.0 for Windows console APIs, which works with both MSVC and GNU targets. The `x86_64-pc-windows-gnu` target used by this project is fully supported.

## Architecture Patterns

### Recommended Project Structure
```
crates/qsym-cli/src/
  main.rs          # REPL loop, rustyline Editor setup, command dispatch
  repl.rs          # NEW: ReplHelper struct, Completer/Validator/Hinter/Highlighter impls
  help.rs          # NEW: Help system (grouped list, per-function docs)
  commands.rs      # NEW: Session command parsing and execution (set, clear, quit)
  lib.rs           # Add: mod repl, mod help, mod commands
  ast.rs           # (existing, unchanged)
  environment.rs   # (existing, minor: add reset() method for `clear`)
  error.rs         # (existing, unchanged)
  eval.rs          # (existing, unchanged)
  format.rs        # (existing, unchanged)
  lexer.rs         # (existing, unchanged)
  parser.rs        # (existing, unchanged)
  token.rs         # (existing, unchanged)
```

### Pattern 1: ReplHelper Composite Struct
**What:** A single struct implements all four rustyline traits via derive delegation.
**When to use:** Always -- this is how rustyline expects custom behavior to be wired up.
**Example:**
```rust
// Source: rustyline docs + official example
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationResult, Validator, ValidationContext};
use rustyline::{Context, Helper};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};

#[derive(Helper, Highlighter, Hinter)]
struct ReplHelper {
    function_names: Vec<&'static str>,  // ALL_FUNCTION_NAMES from eval.rs
    env_vars: std::sync::Arc<std::sync::Mutex<Vec<String>>>,  // shared with env
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Find word start (scan backwards for non-alphanumeric/underscore)
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &line[start..pos];

        if prefix.is_empty() {
            return Ok((start, vec![]));
        }

        let mut candidates = Vec::new();
        for &name in &self.function_names {
            if name.starts_with(prefix) {
                candidates.push(Pair {
                    display: name.to_string(),
                    replacement: format!("{}(", name),  // Auto-insert paren
                });
            }
        }

        // Optionally: also complete user-defined variable names
        // (recommended -- see discretion section)

        Ok((start, candidates))
    }
}

impl Validator for ReplHelper {
    fn validate(
        &self,
        ctx: &mut ValidationContext<'_>,
    ) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();
        // Count unmatched parens/brackets
        let mut paren_depth: i32 = 0;
        let mut bracket_depth: i32 = 0;
        for ch in input.chars() {
            match ch {
                '(' => paren_depth += 1,
                ')' => paren_depth -= 1,
                '[' => bracket_depth += 1,
                ']' => bracket_depth -= 1,
                _ => {}
            }
        }
        if paren_depth > 0 || bracket_depth > 0 {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}
```

### Pattern 2: REPL Main Loop
**What:** The top-level read-eval-print loop using rustyline's `Editor`.
**When to use:** In main.rs.
**Example:**
```rust
use rustyline::config::Config;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, EditMode, Editor};

fn main() -> rustyline::Result<()> {
    print_banner();

    let config = Config::builder()
        .completion_type(CompletionType::Circular)  // zsh-style cycling
        .edit_mode(EditMode::Emacs)
        .auto_add_history(true)
        .max_history_size(1000)
        .expect("valid history size")
        .build();

    let helper = ReplHelper::new();
    let mut rl: Editor<ReplHelper, _> = Editor::with_config(config)?;
    rl.set_helper(Some(helper));

    // Load history from alongside executable
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

                // Check for session commands first
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
                match parser::parse(trimmed) {
                    Ok(stmts) => {
                        for stmt in &stmts {
                            match eval_stmt_safe(stmt, &mut env) {
                                Ok(Some(val)) => println!("{}", format_value(&val)),
                                Ok(None) => {}  // colon-suppressed
                                Err(e) => eprintln!("{}", e),
                            }
                        }
                    }
                    Err(e) => eprintln!("{}", e.render(trimmed)),
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: cancel current line, don't exit
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D: exit
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history(&history_path);
    Ok(())
}
```

### Pattern 3: History File Alongside Executable
**What:** Store `.q_kangaroo_history` next to the binary, not in `$HOME`.
**When to use:** For portable deployment (user decision: "history travels with the binary").
**Example:**
```rust
fn history_file_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".q_kangaroo_history")
}
```

### Pattern 4: Command Dispatch (Before Parser)
**What:** Intercept `help`, `set`, `clear`, `quit`, `exit` before sending to the expression parser.
**When to use:** These are REPL meta-commands, not mathematical expressions.
**Example:**
```rust
enum Command {
    Help(Option<String>),       // help | help aqprod
    SetPrecision(i64),          // set precision 30
    Clear,                      // clear
    Quit,                       // quit | exit
}

enum CommandResult {
    Continue,
    Quit,
    Output(String),
}

fn parse_command(line: &str) -> Option<Command> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.first().map(|s| s.to_lowercase()).as_deref() {
        Some("quit") | Some("exit") => Some(Command::Quit),
        Some("clear") => Some(Command::Clear),
        Some("help") => {
            let topic = parts.get(1).map(|s| s.to_string());
            Some(Command::Help(topic))
        }
        Some("set") if parts.get(1).map(|s| s.to_lowercase()).as_deref() == Some("precision") => {
            parts.get(2).and_then(|s| s.parse::<i64>().ok()).map(Command::SetPrecision)
        }
        _ => None,  // Not a command, send to parser
    }
}
```

### Anti-Patterns to Avoid
- **Parsing commands in the expression parser:** Session commands like `help`, `set precision`, `quit` must be intercepted BEFORE the Pratt parser. The parser would choke on these (they aren't valid expressions).
- **Sharing mutable Environment with ReplHelper:** The completer runs while the user is typing; it must not hold a mutable borrow on Environment. Use `Arc<Mutex<>>` for the variable name list, or just snapshot variable names periodically.
- **Blocking the main thread in help output:** Long help output should just print directly, no paging needed for a math REPL.
- **Mixing Maple aliases into completions:** User explicitly said "canonical function names only in completions."

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Line editing (cursor, insert, delete) | Custom terminal raw mode handling | rustyline | Terminal handling is platform-specific, brittle on Windows |
| History persistence | Custom file read/write with dedup | rustyline `load_history`/`save_history` | Edge cases: file locking, encoding, size limits |
| Tab completion cycling | Custom state machine for Tab presses | `CompletionType::Circular` | rustyline handles multi-press cycling, display, and cursor positioning |
| Bracket matching for multi-line | Custom parser-based incomplete detection | `Validator` trait with paren counting | Simple, correct, and integrates with rustyline's line continuation |

**Key insight:** rustyline handles all the terminal/platform complexity. Our code only needs to implement the domain-specific parts: which names to complete, what help text to show, and what session commands to support.

## Common Pitfalls

### Pitfall 1: History File Path on Windows
**What goes wrong:** `std::env::current_exe()` can fail or return unexpected paths on Windows, especially when run via symlinks or from Cygwin.
**Why it happens:** The function is platform-specific and documented as potentially unreliable.
**How to avoid:** Use a fallback chain: `current_exe().parent()` first, then fall back to current working directory `"."`. Never unwrap blindly.
**Warning signs:** History not persisting between sessions.

### Pitfall 2: Tab Completion With Parenthesis Doubling
**What goes wrong:** User types `aqprod(`, hits backspace to `aqpro`, then Tab-completes back to `aqprod(`. Now the `(` is correct. But if user types `aqprod` (no paren) and Tab completes, they get `aqprod(` -- good. But if they're already inside `func(aqp` and Tab completes, they get `func(aqprod(` which is correct. The edge case is when the cursor is right before an existing `(`.
**Why it happens:** The `replacement` field always includes `(`.
**How to avoid:** Check if the character at `pos` in the line is already `(`. If so, use `replacement` without the paren. This is a small but important UX detail.
**Warning signs:** Users getting `aqprod((` with double parens.

### Pitfall 3: Command vs Expression Ambiguity
**What goes wrong:** A user defines a variable named `help` or `clear` via `help := 42`, and then `help` is intercepted as a command instead of evaluated as a variable reference.
**Why it happens:** Command dispatch happens before the parser.
**How to avoid:** Only intercept bare `help`, `clear`, `quit`, `exit`, and `set precision N` patterns. The presence of `:=` or `;` or `(` in the line is a signal it's an expression, not a command. Simple heuristic: if the line matches the exact command pattern, treat as command; otherwise pass to parser.
**Warning signs:** Users unable to use `clear` as a variable name.

### Pitfall 4: Environment Borrowing During Completion
**What goes wrong:** Compilation error because `ReplHelper` needs to read `env.variables` for variable name completion, but `env` is mutably borrowed during `eval_stmt_safe`.
**Why it happens:** Rust's borrow checker prevents aliased mutable borrows. The `Helper` is stored in `Editor` and called during `readline()`, which is before `eval_stmt_safe`.
**How to avoid:** Two approaches: (a) Don't complete variables (simplest), or (b) After each evaluation, update a `Vec<String>` of variable names in the helper via `rl.helper_mut()`. This works because `readline()` and `eval` never run simultaneously.
**Warning signs:** Borrow checker errors at compile time.

### Pitfall 5: Multi-line Validator Being Too Aggressive
**What goes wrong:** Typing `f(1, 2) + g(3` followed by Enter doesn't submit -- the validator sees unclosed `(` and returns `Incomplete`. But the user intended to type it as-is and get a parse error.
**Why it happens:** The paren-counting validator can't distinguish "user isn't done typing" from "user made a syntax error."
**How to avoid:** Only return `Incomplete` when the last non-whitespace character suggests continuation (like `,` or an operator or backslash). Or: require explicit backslash `\` for continuation, and skip the paren-counting validator. Given our use case (math expressions), paren-counting is actually quite reliable because incomplete parens are almost always "not done yet."
**Warning signs:** Users unable to submit lines with intentional syntax errors.

## Code Examples

### Complete Completer Implementation (verified pattern)
```rust
// Source: rustyline docs Completer trait + Pair struct
use rustyline::completion::{Completer, Pair};
use rustyline::Context;

// The 79 canonical function names from eval.rs ALL_FUNCTION_NAMES
const COMPLETABLE_FUNCTIONS: &[&str] = &[
    "aqprod", "qbin", "etaq", "jacprod", "tripleprod", "quinprod", "winquist",
    "theta2", "theta3", "theta4",
    "partition_count", "partition_gf", "distinct_parts_gf", "odd_parts_gf",
    "bounded_parts_gf", "rank_gf", "crank_gf",
    "sift", "qdegree", "lqdegree", "qfactor",
    "prodmake", "etamake", "jacprodmake", "mprodmake", "qetamake",
    "findlincombo", "findhomcombo", "findnonhomcombo",
    "findlincombomodp", "findhomcombomodp",
    "findhom", "findnonhom", "findhommodp", "findmaxind", "findprod", "findcong",
    "findpoly",
    "phi", "psi", "try_summation", "heine1", "heine2", "heine3",
    "sears_transform", "watson_transform", "find_transformation_chain",
    "mock_theta_f3", "mock_theta_phi3", "mock_theta_psi3",
    "mock_theta_chi3", "mock_theta_omega3", "mock_theta_nu3", "mock_theta_rho3",
    "mock_theta_f0_5", "mock_theta_f1_5",
    "mock_theta_cap_f0_5", "mock_theta_cap_f1_5",
    "mock_theta_phi0_5", "mock_theta_phi1_5",
    "mock_theta_psi0_5", "mock_theta_psi1_5",
    "mock_theta_chi0_5", "mock_theta_chi1_5",
    "mock_theta_cap_f0_7", "mock_theta_cap_f1_7", "mock_theta_cap_f2_7",
    "appell_lerch_m", "universal_mock_theta_g2", "universal_mock_theta_g3",
    "bailey_weak_lemma", "bailey_apply_lemma", "bailey_chain", "bailey_discover",
    "prove_eta_id", "search_identities",
    "q_gosper", "q_zeilberger", "verify_wz", "q_petkovsek",
    "prove_nonterminating",
];

// Session commands also completable
const COMPLETABLE_COMMANDS: &[&str] = &[
    "help", "quit", "exit", "clear", "set",
];

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &line[start..pos];

        if prefix.is_empty() {
            return Ok((start, vec![]));
        }

        let next_char = line.get(pos..pos+1);
        let has_paren_after = next_char == Some("(");

        let mut candidates = Vec::new();

        // Complete function names
        for &name in COMPLETABLE_FUNCTIONS {
            if name.starts_with(prefix) {
                let replacement = if has_paren_after {
                    name.to_string()
                } else {
                    format!("{}(", name)
                };
                candidates.push(Pair {
                    display: name.to_string(),
                    replacement,
                });
            }
        }

        // Complete session commands (only at start of line)
        if start == 0 {
            for &cmd in COMPLETABLE_COMMANDS {
                if cmd.starts_with(prefix) {
                    candidates.push(Pair {
                        display: cmd.to_string(),
                        replacement: cmd.to_string(),  // No paren for commands
                    });
                }
            }
        }

        // Complete user-defined variable names (recommended)
        if let Ok(vars) = self.env_var_names.lock() {
            for var_name in vars.iter() {
                if var_name.starts_with(prefix) {
                    candidates.push(Pair {
                        display: var_name.clone(),
                        replacement: var_name.clone(),  // No paren for variables
                    });
                }
            }
        }

        Ok((start, candidates))
    }
}
```

### Help System Category Structure
```rust
// Source: eval.rs group organization
struct FunctionHelp {
    name: &'static str,
    signature: &'static str,
    one_liner: &'static str,
    description: &'static str,
    example: &'static str,
    example_output: &'static str,
}

struct HelpCategory {
    name: &'static str,
    functions: &'static [FunctionHelp],
}

// 8 categories matching eval.rs groups:
// 1. Products (7): aqprod, qbin, etaq, jacprod, tripleprod, quinprod, winquist
// 2. Partitions (7): partition_count, partition_gf, distinct_parts_gf, odd_parts_gf,
//                     bounded_parts_gf, rank_gf, crank_gf
// 3. Theta (3): theta2, theta3, theta4
// 4. Analysis (9): sift, qdegree, lqdegree, qfactor, prodmake, etamake,
//                   jacprodmake, mprodmake, qetamake
// 5. Relations (12): findlincombo, findhomcombo, findnonhomcombo, findlincombomodp,
//                     findhomcombomodp, findhom, findnonhom, findhommodp,
//                     findmaxind, findprod, findcong, findpoly
// 6. Hypergeometric (9): phi, psi, try_summation, heine1, heine2, heine3,
//                         sears_transform, watson_transform, find_transformation_chain
// 7. Mock Theta/Bailey (27): 20 mock theta + 3 Appell-Lerch + 4 Bailey
// 8. Identity Proving (5): prove_eta_id, search_identities, q_gosper,
//                           q_zeilberger, verify_wz, q_petkovsek, prove_nonterminating
```

### Environment Reset for `clear`
```rust
// Add to environment.rs
impl Environment {
    /// Reset the environment to its initial state.
    ///
    /// Clears all variables, resets last_result, and restores default_order to 20.
    /// Does NOT reset the symbol registry (sym_q must remain valid).
    pub fn reset(&mut self) {
        self.variables.clear();
        self.last_result = None;
        self.default_order = 20;
    }
}
```

### Rustyline Config Setup
```rust
// Source: rustyline 17.0.1 Config docs
let config = Config::builder()
    .completion_type(CompletionType::Circular)   // zsh-style Tab cycling
    .edit_mode(EditMode::Emacs)                  // Emacs keybindings (standard)
    .auto_add_history(true)                      // Auto-add non-empty lines
    .max_history_size(10_000)                    // Generous history
    .expect("valid max_history_size")
    .history_duplicates(HistoryDuplicates::IgnoreConsecutive)
    .build();
```

## Discretion Recommendations

### Multi-line Input: Use Paren-Counting Validator
**Recommendation:** Implement a custom `Validator` that returns `Incomplete` when unmatched `(` or `[` remain at end of input. This is the most natural approach for mathematical expressions where unclosed parens always mean "user isn't done." Backslash continuation adds cognitive overhead for no benefit.

**Rationale:** Users will frequently type complex nested calls like `sift(partition_gf(100), 5, 4)`. If they hit Enter after typing `sift(partition_gf(100), 5,` the REPL should wait for the closing `)`. Paren counting handles this correctly with zero user effort.

The continuation prompt should be `.. ` (two dots, space) to show the line continues.

### Variable Name Completion: Yes, Include Them
**Recommendation:** Complete user-defined variable names in addition to function names. Variables get no `(` suffix. This is standard REPL behavior and users expect it.

**Implementation:** After each successful evaluation, update the helper's variable name list via `rl.helper_mut().unwrap().update_var_names(&env.variables)`. This is safe because `readline()` and evaluation never overlap.

### `set precision N` Behavior: Forward-Only
**Recommendation:** `set precision N` should only affect new computations. Stored variables retain their original precision. Retroactive recomputation would require tracking the original expression for each variable, which is complex and surprising.

**Rationale:** In Maple and similar systems, changing `Digits` affects new computations only. Variables already computed keep their values. This matches researcher expectations.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| rustyline < 10 (manual trait impls) | rustyline 17 with derive macros | ~2023 | Less boilerplate for Helper setup |
| readline (C library via FFI) | Pure Rust rustyline | 2016+ | No C dependency, cross-platform |
| No Validator (single-line only) | Validator trait for multi-line | rustyline 6+ | Natural multi-line editing |

**Deprecated/outdated:**
- `rustyline::Editor::new()` without generics is the old API; current API uses `Editor::<H, _>::with_config(config)` with explicit helper type

## Open Questions

1. **`latex` and `save` commands in help text**
   - What we know: User wants the help Commands section to list `latex` and `save`, but these are Phase 27 scope
   - What's unclear: Should Phase 26 list them as "(coming soon)" or omit them?
   - Recommendation: Include them in the help output with a note "(not yet available)" so the help text is correct when Phase 27 ships. Commands section in help.rs should list all planned commands.

2. **ASCII art kangaroo design**
   - What we know: User wants an ASCII art kangaroo in the welcome banner
   - What's unclear: Exact design
   - Recommendation: Use a compact 5-7 line ASCII kangaroo. Keep it small so it doesn't dominate the terminal. Include version from `env!("CARGO_PKG_VERSION")`.

## Sources

### Primary (HIGH confidence)
- rustyline 17.0.1 docs (docs.rs) - CompletionType, Completer trait, Validator trait, Config, Pair struct
- rustyline GitHub example (github.com/kkawakam/rustyline/blob/master/examples/example.rs) - Full Helper pattern
- Existing codebase: eval.rs ALL_FUNCTION_NAMES (79 canonical names), get_signature(), resolve_alias()
- Existing codebase: environment.rs Environment struct, eval.rs eval_stmt_safe

### Secondary (MEDIUM confidence)
- rustyline crates.io - version info, download counts, feature flags
- WebSearch: rustyline vs reedline comparison for ecosystem positioning

### Tertiary (LOW confidence)
- None -- all findings verified against official sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - rustyline is the clear standard for Rust REPLs, verified API docs
- Architecture: HIGH - pattern follows official rustyline example, adapted to existing codebase
- Pitfalls: HIGH - derived from actual API analysis and known Windows/Cygwin constraints

**Research date:** 2026-02-17
**Valid until:** 2026-04-17 (rustyline is stable, unlikely to change significantly in 60 days)
