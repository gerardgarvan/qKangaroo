---
phase: 31-error-hardening-exit-codes
plan: 01
subsystem: cli
tags: [error-handling, exit-codes, panic-translation, repl, diagnostics]

# Dependency graph
requires:
  - phase: 28-repl-polish
    provides: REPL infrastructure with script execution and command dispatch
provides:
  - FileNotFound (exit 66) and IoError (exit 74) ScriptResult variants
  - filename:line:col error rendering for script parse errors
  - filename:line context for script eval errors
  - Human-friendly panic message translation
  - REPL graceful handling of --help/--version at prompt
affects: [31-error-hardening-exit-codes, error-reporting, cli-ux]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ErrorKind dispatch in execute_file for distinct exit codes"
    - "Statement-to-line mapping via lexer token spans"
    - "Panic message translation with contains() for robustness"
    - "execute_source_with_context pattern for filename threading"

key-files:
  created: []
  modified:
    - crates/qsym-cli/src/script.rs
    - crates/qsym-cli/src/error.rs
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/main.rs
    - crates/qsym-cli/tests/cli_integration.rs

key-decisions:
  - "Used lexer tokenize() to find statement boundary byte offsets for line number computation"
  - "Panic translation uses contains() matching for robustness against minor wording changes"
  - "read() function errors use EvalError::Other instead of EvalError::Panic for non-panic failures"

patterns-established:
  - "ErrorKind dispatch: NotFound -> exit 66, other I/O -> exit 74"
  - "execute_source_with_context threading filename through error chain"

requirements-completed: [EXIT-05, EXIT-07, ERR-01, ERR-02, ERR-03, ERR-04, ERR-05]

# Metrics
duration: 5min
completed: 2026-02-18
---

# Phase 31 Plan 01: Error Hardening & Exit Codes Summary

**Distinct exit codes (66/74) for file errors, filename:line:col in script diagnostics, panic message translation, and REPL --help/--version handling**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-18T22:52:10Z
- **Completed:** 2026-02-18T22:57:59Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- FileNotFound (exit 66) and IoError (exit 74) variants with ErrorKind dispatch in execute_file
- filename:line:col rendering for parse errors and filename:line for eval errors in scripts
- byte_offset_to_line_col and render_for_file public helpers for error positioning
- Panic message translation (zero constant term, division by zero, cannot invert zero, index out of bounds)
- REPL handles --help/-h and --version/-V typed at the prompt without crashing
- read() function uses EvalError::Other for file/parse errors instead of Panic

## Task Commits

Each task was committed atomically:

1. **Task 1: ScriptResult variants, exit codes, execute_file dispatch, filename-threaded execute_source** - `5a0305a` (feat)
2. **Task 2: Panic translation, read() error improvement, REPL --help handling** - `2e29292` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/script.rs` - Added FileNotFound/IoError variants, EXIT_IO_ERROR, execute_source_with_context, compute_stmt_starts/line
- `crates/qsym-cli/src/error.rs` - Added byte_offset_to_line_col() and render_for_file() with tests
- `crates/qsym-cli/src/eval.rs` - Added translate_panic_message(), updated eval_stmt_safe, updated read() dispatch
- `crates/qsym-cli/src/main.rs` - Added --help/-h and --version/-V handling at REPL prompt
- `crates/qsym-cli/tests/cli_integration.rs` - Updated file-not-found tests for new error message and exit code 66

## Decisions Made
- Used lexer tokenize() to compute statement boundary byte offsets, then byte_offset_to_line_col for line numbers (avoids needing spans on Stmt AST nodes)
- Panic translation uses contains() matching for robustness against minor upstream wording changes
- read() function now returns EvalError::Other for file/parse errors (not Panic), so REPL shows "Error: ..." instead of "Error: computation failed: ..."

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed read() dispatch for new ScriptResult variants in Task 1**
- **Found during:** Task 1 (compiling after adding ScriptResult variants)
- **Issue:** eval.rs `read` function's match on ScriptResult was non-exhaustive after adding FileNotFound/IoError
- **Fix:** Updated match in eval.rs to handle all 6 variants (pulled forward from Task 2 plan)
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** cargo test compilation succeeded
- **Committed in:** 5a0305a (part of Task 1 commit)

**2. [Rule 1 - Bug] Fixed translate_panic_message thread prefix stripping**
- **Found during:** Task 2 (translate_panic_thread_prefix_stripped test)
- **Issue:** Plan's code used `"': "` delimiter which doesn't match Rust's actual panic format `"thread 'main' panicked at 'msg'"`
- **Fix:** Changed to `contains("panicked at")` extraction with proper quote/trailing stripping and recursive translation
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** All 7 translate_panic tests pass
- **Committed in:** 2e29292 (Task 2 commit)

**3. [Rule 1 - Bug] Updated existing eval_stmt_safe_catches_panic test assertion**
- **Found during:** Task 2 (test now sees translated message)
- **Issue:** Test asserted `msg.contains("zero constant term")` but translated message says "constant term is zero"
- **Fix:** Updated assertion to check for "constant term is zero"
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** Test passes
- **Committed in:** 2e29292 (Task 2 commit)

**4. [Rule 1 - Bug] Updated integration tests for new error messages**
- **Found during:** Task 2 verification (integration tests)
- **Issue:** script_file_not_found and dashdash_separator tests checked for "cannot read" but new code emits "file not found"
- **Fix:** Updated assertions to match new error message format and exit code 66
- **Files modified:** crates/qsym-cli/tests/cli_integration.rs
- **Verification:** All 37 integration tests pass
- **Committed in:** 2e29292 (Task 2 commit)

---

**Total deviations:** 4 auto-fixed (1 blocking, 3 bugs)
**Impact on plan:** All fixes necessary for correctness. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Error hardening infrastructure complete
- Ready for 31-02: integration tests for error hardening scenarios
- All 335 lib tests + 37 integration tests passing

---
*Phase: 31-error-hardening-exit-codes*
*Completed: 2026-02-18*
