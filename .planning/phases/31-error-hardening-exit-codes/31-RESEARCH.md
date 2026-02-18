# Phase 31: Error Hardening & Exit Codes - Research

**Researched:** 2026-02-18
**Domain:** CLI error handling, exit code standards, error message formatting
**Confidence:** HIGH

## Summary

Phase 31 hardens the q-Kangaroo CLI's error reporting and exit code system. The existing codebase already has a solid foundation: `ScriptResult` enum with `exit_code()` mapping, `EvalError` enum with human-readable `Display`, `ParseError` with caret-style rendering, and `eval_stmt_safe()` with `catch_unwind` for panic capture. The work needed is incremental: filling gaps in exit code coverage (missing EXIT_IO_ERROR=74), adding filename:line:col context to script errors (ERR-01), translating common panic messages to friendlier text (ERR-02), differentiating file-not-found from generic eval errors (EXIT-05 vs current behavior), and ensuring REPL error-continuation semantics are properly contrasted with script fail-fast behavior (ERR-04, ERR-05).

The current `ParseError::render()` method only handles single-line source and uses absolute byte offset as column. For multiline scripts (the primary use case for ERR-01), it needs to compute line number and within-line column from the byte offset, and optionally prefix the error with `filename:line:col:`. The `execute_file()` function currently returns `ScriptResult::EvalError` for file-not-found, but EXIT-05 requires it return a distinct variant with exit code 66. Similarly, EXIT-07 requires exit code 74 for I/O errors, which has no constant or variant today.

**Primary recommendation:** Add `ScriptResult::IoError` and `ScriptResult::FileNotFound` variants, enhance `ParseError::render()` to support multiline source with optional filename prefix, and add a `translate_panic_message()` helper that maps common qsym-core assert messages to user-friendly text.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| EXIT-01 | Exit code 0 on success | Already implemented: `ScriptResult::Success.exit_code() == 0`. Verified in tests. |
| EXIT-02 | Exit code 1 on evaluation error in batch mode | Already implemented: `ScriptResult::EvalError.exit_code() == 1`. Verified in tests. |
| EXIT-03 | Exit code 2 on usage error (bad flags) | Already implemented: `parse_args()` returns `Err(msg)` -> `EXIT_USAGE` (2). Tested. |
| EXIT-04 | Exit code 65 on parse error in script input | Already implemented: `ScriptResult::ParseError.exit_code() == 65`. Tested. |
| EXIT-05 | Exit code 66 on file not found | GAP: `execute_file()` currently wraps file-not-found as `ScriptResult::EvalError` (exit 1). Need new `ScriptResult::FileNotFound` variant returning 66. |
| EXIT-06 | Exit code 70 on caught panic (internal error) | Already implemented: `ScriptResult::Panic.exit_code() == 70`. `eval_stmt_safe()` catches panics. |
| EXIT-07 | Exit code 74 on I/O error | GAP: No `EXIT_IO_ERROR` constant or `ScriptResult::IoError` variant. Need to add for I/O errors distinct from file-not-found. |
| ERR-01 | Script errors include filename:line:col context | GAP: `ParseError::render()` only shows column within single-line source. `EvalError` has no source location. Need `render_with_filename()` and line/col computation from byte offset. |
| ERR-02 | Common qsym-core panics translated to human-readable messages | PARTIAL: `eval_stmt_safe()` catches panics and extracts message string. Need a translation layer to map assert messages like "Cannot invert series with zero constant term" to friendlier text. |
| ERR-03 | File I/O errors display OS error message | PARTIAL: `execute_file()` already includes `{e}` (the OS error) in message. Need to ensure this works for all I/O paths and distinguishes not-found vs permission-denied vs other. |
| ERR-04 | Scripts fail-fast on first error; REPL continues | PARTIAL: Script `execute_source()` already returns on first error. REPL loop already continues on error (line 313 in main.rs: `Err(e) => eprintln!("{}", e)`). Need to verify read() behavior in scripts vs REPL. |
| ERR-05 | read() in REPL continues on error | PARTIAL: `read()` function call propagates errors via `EvalError::Panic`. In REPL, errors are caught per-statement. Need to verify the REPL read() command also continues gracefully. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| std::process::ExitCode | Rust std | Exit code from main() | Already used, type-safe exit codes |
| std::panic::catch_unwind | Rust std | Catch qsym-core panics | Already used in `eval_stmt_safe()` |
| std::io::ErrorKind | Rust std | Distinguish NotFound vs PermissionDenied vs other | Standard way to classify I/O errors |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none) | - | - | Zero external dependencies philosophy maintained |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled exit codes | `sysexits` crate | Project uses zero-deps philosophy; constants are trivial to define manually |
| Custom panic translation | `anyhow`/`eyre` | Overkill for this use case; project avoids external deps in CLI |

**No new dependencies needed.** All work is internal to existing Rust std facilities.

## Architecture Patterns

### Current Error Flow
```
User Input
  |
  v
parse_args() --Err--> exit(2)  [EXIT_USAGE]
  |
  Ok(CliMode)
  |
  v
execute_source() / execute_file()
  |
  +--> parse() --Err--> ScriptResult::ParseError [exit 65]
  |
  +--> eval_stmt_safe()
  |      |
  |      +--> catch_unwind(eval_stmt)
  |             |
  |             +--> Ok(Ok(val)) --> continue
  |             +--> Ok(Err(EvalError)) --> ScriptResult::EvalError [exit 1]
  |             +--> Err(panic) --> ScriptResult::Panic [exit 70]
  |
  +--> fs::read_to_string() --Err--> ScriptResult::EvalError [exit 1]  <-- BUG: should be 66 or 74
```

### Pattern 1: Enriched ScriptResult for I/O Errors
**What:** Add `FileNotFound(String)` and `IoError(String)` variants to `ScriptResult`
**When to use:** When `execute_file()` fails to read the script file
**Example:**
```rust
// In script.rs
pub const EXIT_IO_ERROR: u8 = 74;

pub enum ScriptResult {
    Success,
    ParseError(String),
    EvalError(String),
    Panic(String),
    FileNotFound(String),   // NEW: exit 66
    IoError(String),         // NEW: exit 74
}

impl ScriptResult {
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
}

pub fn execute_file(path: &str, env: &mut Environment, verbose: bool) -> ScriptResult {
    match std::fs::read_to_string(path) {
        Ok(source) => execute_source(&source, env, verbose),
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
```

### Pattern 2: Filename-Aware Error Rendering for Scripts
**What:** Add `render_for_file()` to `ParseError` that computes line/col from byte offset in multiline source
**When to use:** When displaying parse errors from script files
**Example:**
```rust
// In error.rs
impl ParseError {
    /// Render for a script file: shows filename:line:col prefix.
    pub fn render_for_file(&self, source: &str, filename: &str) -> String {
        let (line, col) = byte_offset_to_line_col(source, self.span.start);
        // Show the offending line
        let source_line = source.lines().nth(line - 1).unwrap_or("");
        let spaces = " ".repeat(col - 1 + 2); // 2 for "  " prefix
        format!(
            "{}:{}:{}: parse error: {}\n  {}\n{}^",
            filename, line, col, self.message, source_line, spaces
        )
    }
}

/// Convert a byte offset to 1-indexed (line, col).
fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}
```

### Pattern 3: Panic Message Translation
**What:** Map common qsym-core panic assert messages to user-friendly descriptions
**When to use:** In `eval_stmt_safe()` after catching a panic, before wrapping in `EvalError::Panic`
**Example:**
```rust
// In eval.rs
fn translate_panic_message(raw: &str) -> String {
    // Map common qsym-core panics to friendlier messages
    if raw.contains("Cannot invert series with zero constant term") {
        return "division by a series with zero constant term (the series starts at q^k with k > 0; \
                try shifting or extracting the leading power first)".to_string();
    }
    if raw.contains("division by zero") {
        return "division by zero".to_string();
    }
    if raw.contains("pseudo_rem: division by zero") {
        return "polynomial division by zero".to_string();
    }
    if raw.contains("Cannot invert zero") {
        return "cannot invert zero".to_string();
    }
    // Default: pass through, but strip Rust-internal noise
    raw.to_string()
}
```

### Pattern 4: Script execute_source with Filename Context
**What:** Thread filename through script execution so error messages include file context
**When to use:** In `execute_file()` and `execute_source()`
**Example:**
```rust
// execute_source gains optional filename parameter
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
    // ... rest of evaluation with filename context in error messages
}
```

### Anti-Patterns to Avoid
- **Changing exit codes for existing working cases:** EXIT-01 through EXIT-04 and EXIT-06 already work. Do not break them.
- **Swallowing I/O error details:** ERR-03 requires showing the OS error message. Always include `{e}` in formatted error strings.
- **Making REPL errors fatal:** ERR-04/ERR-05 explicitly require REPL to continue on errors. Never add `process::exit()` in the REPL path.
- **Exposing raw panic backtraces:** ERR-02 says "not a Rust panic backtrace". The `eval_stmt_safe()` catch already handles this, but make sure the translated message doesn't include "thread 'main' panicked at" text.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| I/O error classification | Manual string matching on error messages | `std::io::ErrorKind` match | Portable, exhaustive, standard Rust |
| Byte offset to line/col | Complex custom parser | Simple char_indices loop | ASCII-only grammar means bytes == chars for this project |
| Exit code values | Custom enum with manual numbering | Constants matching sysexits.h | Already partially done; just add the missing constant |

**Key insight:** The project's existing error infrastructure is well-designed. This phase is about filling specific gaps, not redesigning error handling.

## Common Pitfalls

### Pitfall 1: Multiline Source Caret Offset Bug
**What goes wrong:** `ParseError::render()` currently dumps the ENTIRE source string and uses the absolute byte offset as the caret position. For multiline scripts, this means the caret points to wrong location.
**Why it happens:** The render() was designed for single-line REPL input.
**How to avoid:** Use `render_for_file()` for multiline sources. Compute line number from byte offset, extract just that source line, compute column within that line.
**Warning signs:** Error messages showing the entire script source on one "line" in the caret display.

### Pitfall 2: File Not Found vs Other I/O Errors
**What goes wrong:** Currently `execute_file()` wraps ALL file read errors as `ScriptResult::EvalError` (exit 1). EXIT-05 wants 66 for not-found, EXIT-07 wants 74 for other I/O.
**Why it happens:** Original implementation didn't distinguish error types.
**How to avoid:** Match on `std::io::ErrorKind::NotFound` specifically.
**Warning signs:** `q-kangaroo nonexistent.qk` returning exit code 1 instead of 66.

### Pitfall 3: EvalError Context in Script Mode
**What goes wrong:** `EvalError` has no source location (line/col). In REPL, this is fine (one expression at a time). In scripts, user can't find which line caused "Error: undefined variable 'x'".
**Why it happens:** `AstNode` doesn't carry span information (noted in ast.rs line 23: "AstNode does not carry span information").
**How to avoid:** For script mode, track which statement number is being evaluated, compute the byte range of that statement from the parsed position, and convert to line number. Alternatively, since statements are evaluated sequentially, count which statement failed and map back to source lines.
**Warning signs:** Script eval errors showing no file/line context.

### Pitfall 4: read() Error Propagation Asymmetry
**What goes wrong:** `read()` function uses `EvalError::Panic` for ALL errors (parse, eval, file-not-found). This loses error type information.
**Why it happens:** Intentional simplification in Phase 30 -- read() maps everything to Panic.
**How to avoid:** Consider using more specific EvalError variants for read() errors, or at minimum ensure the error message itself is descriptive. For ERR-05, just verify REPL continues after read() errors.
**Warning signs:** read() of nonexistent file showing "computation failed" instead of "file not found".

### Pitfall 5: Breaking Existing Integration Tests
**What goes wrong:** Changing `ScriptResult` variants or exit codes could break the 486-line integration test file.
**Why it happens:** Tests assert specific exit codes and error message substrings.
**How to avoid:** Run existing tests first. The test at line 289 (`script_file_not_found`) asserts `code != 0` and `stderr.contains("cannot read")` -- changing exit code from 1 to 66 won't break the `!= 0` check. But update the test to assert `== 66` specifically.
**Warning signs:** Integration test failures after changes.

### Pitfall 6: Panic Translation Fragility
**What goes wrong:** Panic message strings from qsym-core assertions are implementation details that could change.
**Why it happens:** We're matching on assertion message text.
**How to avoid:** Use `contains()` rather than exact string equality. Keep translations in one function for easy maintenance. Fall back to raw message if no translation matches.
**Warning signs:** Users seeing raw assert messages after a qsym-core update changes wording.

## Code Examples

### Example 1: Current execute_file (showing the gap)
```rust
// Source: crates/qsym-cli/src/script.rs lines 132-141
pub fn execute_file(path: &str, env: &mut Environment, verbose: bool) -> ScriptResult {
    match std::fs::read_to_string(path) {
        Ok(source) => execute_source(&source, env, verbose),
        Err(e) => ScriptResult::EvalError(format!("cannot read '{}': {}", path, e)),
        // ^^^ BUG: should distinguish NotFound (66) from IoError (74)
    }
}
```

### Example 2: Current ParseError render (single-line only)
```rust
// Source: crates/qsym-cli/src/error.rs lines 38-46
pub fn render(&self, source: &str) -> String {
    let col = self.span.start;
    let col_display = col + 1;
    let spaces = " ".repeat(col + 2);
    format!(
        "parse error at column {}: {}\n  {}\n{}^",
        col_display, self.message, source, spaces
    )
    // ^^^ For multiline source, this dumps entire source as one "line"
}
```

### Example 3: Current panic catch (already works, needs translation)
```rust
// Source: crates/qsym-cli/src/eval.rs lines 545-561
pub fn eval_stmt_safe(stmt: &Stmt, env: &mut Environment) -> Result<Option<Value>, EvalError> {
    let result = catch_unwind(AssertUnwindSafe(|| eval_stmt(stmt, env)));
    match result {
        Ok(inner) => inner,
        Err(panic_payload) => {
            let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "internal computation error".to_string()
            };
            Err(EvalError::Panic(msg))
            // ^^^ msg is raw panic text like "Cannot invert series with zero constant term"
            //     ERR-02 wants this translated to friendlier language
        }
    }
}
```

### Example 4: Common qsym-core Panic Messages to Translate
```rust
// Source: crates/qsym-core/src/series/arithmetic.rs line 108
assert!(!a0.is_zero(), "Cannot invert series with zero constant term");

// Source: crates/qsym-core/src/number.rs lines 118, 129
assert!(rhs.0.cmp0() != Ordering::Equal, "QInt division by zero");

// Source: crates/qsym-core/src/number.rs lines 271, 282
// "QRat division by zero"

// Source: crates/qsym-core/src/poly/arithmetic.rs line 215
panic!("QRatPoly::pseudo_rem: division by zero");

// Source: crates/qsym-core/src/poly/arithmetic.rs line 154
assert!(!divisor.is_zero(), "QRatPoly::div_rem: division by zero");

// Source: crates/qsym-core/src/poly/mod.rs line 195
assert!(!c.is_zero(), "QRatPoly::scalar_div: division by zero");

// Source: crates/qsym-core/src/qseries/linalg.rs line 272
assert!(a != 0, "Cannot invert zero modulo {}", p);
```

### Example 5: REPL Error Continuation (already works)
```rust
// Source: crates/qsym-cli/src/main.rs lines 301-315
// In the REPL loop, eval errors just print and continue:
match qsym_cli::eval::eval_stmt_safe(stmt, &mut env) {
    Ok(Some(val)) => { /* print */ }
    Ok(None) => { /* noop */ }
    Err(e) => eprintln!("{}", e),  // Print error, loop continues
}
// Parse errors also just print and continue:
Err(e) => eprintln!("{}", e.render(trimmed)),
```

### Example 6: REPL ReadFile Error Handling (already continues)
```rust
// Source: crates/qsym-cli/src/main.rs lines 276-289
CommandResult::ReadFile(path) => {
    let result = script::execute_file(&path, &mut env, verbose);
    if let Some(msg) = result.error_message() {
        eprintln!("{}", msg);
    }
    // Falls through to continue -- REPL does NOT exit on read() error
    continue;
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| All file errors -> exit 1 | Distinguish NotFound (66) vs IoError (74) | This phase | Scripts/tools can detect file-not-found specifically |
| Column-only parse errors | filename:line:col for scripts | This phase | Multiline script errors become locatable |
| Raw panic messages | Translated human-readable messages | This phase | Users see "division by zero" not "QRatPoly::pseudo_rem: division by zero" |

## Open Questions

1. **EvalError source location tracking**
   - What we know: `AstNode` has no span info (by design comment in ast.rs). Parse errors have spans. Eval errors do not.
   - What's unclear: How to show line numbers for eval errors in scripts without adding spans to AstNode.
   - Recommendation: Track statement index during `execute_source()`. Each statement corresponds to a parsed span range. When eval fails at statement N, compute line number from statement position. This avoids changing AstNode. Alternatively, compute statement byte ranges from the parser output by recording start/end positions of each Stmt.

2. **read() error type granularity**
   - What we know: `read()` currently maps all errors to `EvalError::Panic`. This makes file-not-found look like an internal error.
   - What's unclear: Whether to change read() to use `EvalError::Other` with a descriptive message instead of `EvalError::Panic`.
   - Recommendation: Use `EvalError::Other` for read() file errors (they're not panics). Keep `EvalError::Panic` only for actual caught panics from qsym-core.

3. **UAT: --help at REPL prompt**
   - What we know: Typing `--help` at `q>` gives a parse error (the `--` is not valid syntax). Phase 30 UAT flagged this as minor.
   - What's unclear: Whether to handle this in this phase or separately.
   - Recommendation: Quick fix in command parsing: if trimmed input is `--help` or `-h` or `--version` or `-V`, show help/version. Low cost, high UX value. Include as minor task.

## Sources

### Primary (HIGH confidence)
- `crates/qsym-cli/src/script.rs` - ScriptResult enum, exit codes, execute_source/execute_file
- `crates/qsym-cli/src/eval.rs` - EvalError enum, eval_stmt_safe panic catcher, Display impl
- `crates/qsym-cli/src/error.rs` - ParseError, render() method, Span
- `crates/qsym-cli/src/main.rs` - CliMode, parse_args(), REPL loop, mode dispatch
- `crates/qsym-cli/src/commands.rs` - ReadFile command dispatch, read command
- `crates/qsym-cli/src/ast.rs` - AstNode (no spans), Stmt, Terminator
- `crates/qsym-cli/src/token.rs` - Span struct (start, end byte offsets)
- `crates/qsym-cli/tests/cli_integration.rs` - 486 lines of subprocess integration tests
- `crates/qsym-core/src/series/arithmetic.rs` - invert() assert (zero constant term)
- `crates/qsym-core/src/number.rs` - QInt/QRat division by zero asserts
- `crates/qsym-core/src/poly/arithmetic.rs` - polynomial division by zero panics
- [OpenBSD sysexits(3)](https://man.openbsd.org/sysexits) - sysexits.h standard values confirmed

### Secondary (MEDIUM confidence)
- [sysexits.h man page](https://www.man7.org/linux/man-pages/man3/sysexits.h.3head.html) - EX_NOINPUT=66, EX_SOFTWARE=70, EX_IOERR=74 values verified

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - No new dependencies, all std library
- Architecture: HIGH - Incremental changes to well-understood existing code
- Pitfalls: HIGH - Identified from direct codebase analysis, specific line numbers cited
- Exit codes: HIGH - Verified against sysexits.h standard from official BSD manpage

**Research date:** 2026-02-18
**Valid until:** 2026-03-20 (stable domain, no external deps)
