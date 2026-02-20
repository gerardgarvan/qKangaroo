---
phase: 33-symbolic-variable-foundation
plan: 01
subsystem: eval
tags: [symbol, value-enum, lexer, parser, evaluator, q-demotion]

# Dependency graph
requires:
  - phase: 32-cli-man-page
    provides: "Stable CLI with 349 tests and complete function dispatch"
provides:
  - "Value::Symbol(String) variant for undefined name fallback"
  - "q demoted from keyword to regular identifier"
  - "AstNode::Variable('q') instead of AstNode::Q"
  - "Token::Ident('q') instead of Token::Q"
affects: [33-02-symbol-arithmetic, 33-03-symbol-aware-functions]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Symbol fallback: undefined variables return Value::Symbol(name) instead of error"
    - "q is a regular identifier, not a keyword"

key-files:
  created: []
  modified:
    - crates/qsym-cli/src/token.rs
    - crates/qsym-cli/src/lexer.rs
    - crates/qsym-cli/src/ast.rs
    - crates/qsym-cli/src/parser.rs
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/format.rs
    - crates/qsym-cli/src/script.rs
    - crates/qsym-cli/tests/cli_integration.rs

key-decisions:
  - "Series-dependent tests restructured to use pre-assigned variables instead of AstNode::Q"
  - "Integration tests changed from undefined_var to etaq(1) wrong arg count for real eval errors"
  - "EvalError::UnknownVariable type retained for potential future use despite no longer being raised by variable eval"

patterns-established:
  - "Symbol fallback: undefined names return Value::Symbol(name), not errors"
  - "q is identifier: Token::Ident('q') -> AstNode::Variable('q') -> Value::Symbol('q')"

requirements-completed: [SYM-01, SYM-04]

# Metrics
duration: 6min
completed: 2026-02-19
---

# Phase 33 Plan 01: Symbol Variable Foundation Summary

**Value::Symbol variant with undefined-name fallback and q demotion from keyword to regular identifier**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-19T17:16:42Z
- **Completed:** 2026-02-19T17:23:13Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Added Value::Symbol(String) variant to the evaluator Value enum with type_name "symbol"
- Removed Token::Q and AstNode::Q entirely; q now flows through Token::Ident -> AstNode::Variable -> Value::Symbol
- Changed variable evaluation: undefined names return Value::Symbol(name) instead of EvalError::UnknownVariable
- Updated format_value and format_latex to display Symbol values as plain names
- Updated all 336 unit tests and 58 integration tests (394 total, all pass)
- Added 3 new integration tests for SYM-01 (bare symbols) and SYM-04 (assignment precedence)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Value::Symbol, remove Token::Q/AstNode::Q, implement symbol fallback** - `845e4a5` (feat)
2. **Task 2: Update all existing tests for the new symbol behavior** - `9902295` (test)

## Files Created/Modified
- `crates/qsym-cli/src/token.rs` - Removed Token::Q variant; q is now Token::Ident("q")
- `crates/qsym-cli/src/lexer.rs` - Removed "q" => Token::Q keyword match
- `crates/qsym-cli/src/ast.rs` - Removed AstNode::Q variant
- `crates/qsym-cli/src/parser.rs` - Removed Token::Q => AstNode::Q arm and token_name entry
- `crates/qsym-cli/src/eval.rs` - Added Value::Symbol, removed AstNode::Q eval, changed Variable to symbol fallback
- `crates/qsym-cli/src/format.rs` - Added Value::Symbol match arms to format_value and format_latex
- `crates/qsym-cli/src/script.rs` - Updated test for undefined_var now succeeding as symbol
- `crates/qsym-cli/tests/cli_integration.rs` - Updated 6 tests, added 3 new symbol behavior tests

## Decisions Made
- Restructured series arithmetic tests (eval_series_add, eval_scalar_mul_series, eval_series_plus_integer) to use pre-assigned series variables rather than AstNode::Q, since q no longer produces a series directly
- Retained EvalError::UnknownVariable type in the enum even though it's no longer raised by variable evaluation -- it may be useful for future function dispatch or explicit "strict mode"
- Integration tests that used `undefined_var` as an error trigger now use `etaq(1)` (wrong arg count) for a real eval error

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed exit_06_panic_invert_zero_constant test**
- **Found during:** Task 2 (integration test updates)
- **Issue:** Test used `1/(q * etaq(1,1,5))` which depended on q being a series. With q now being a symbol, `q * etaq(...)` is a type error (exit 1), not a panic (exit 70).
- **Fix:** Changed expression to `1/(etaq(1,1,5) - 1)` which creates a series with zero constant term, triggering the same panic behavior.
- **Files modified:** crates/qsym-cli/tests/cli_integration.rs
- **Verification:** Test passes with exit code 70 and correct error message
- **Committed in:** 9902295 (Task 2 commit)

**2. [Rule 3 - Blocking] Added format.rs Value::Symbol arms during Task 1**
- **Found during:** Task 1 (compilation check)
- **Issue:** Build failed because format_value and format_latex match blocks didn't cover Value::Symbol
- **Fix:** Added `Value::Symbol(name) => name.clone()` arms to both functions (planned for Task 2 but needed for compilation)
- **Files modified:** crates/qsym-cli/src/format.rs
- **Verification:** Build compiles successfully
- **Committed in:** 845e4a5 (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered
None beyond the deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Value::Symbol variant is ready for arithmetic operations (Plan 33-02)
- q flows through AstNode::Variable, ready for symbol-aware function dispatch (Plan 33-03)
- All existing functionality preserved with updated tests

## Self-Check: PASSED

All 9 files verified present. Both commit hashes (845e4a5, 9902295) verified in git log.

---
*Phase: 33-symbolic-variable-foundation*
*Completed: 2026-02-19*
