---
phase: 33-symbolic-variable-foundation
plan: 03
subsystem: eval, lexer, commands
tags: [symbol-dispatch, maple-style, etaq, aqprod, anames, restart, unassign, single-quote]

# Dependency graph
requires:
  - phase: 33-01
    provides: "Value::Symbol variant and q demotion from keyword"
  - phase: 33-02
    provides: "Symbol arithmetic (q^2, 2*q, etc.) and variable-aware formatting"
provides:
  - "Maple-style etaq(var, b, order) dispatch accepting Value::Symbol as first arg"
  - "Maple-style aqprod(monomial, var, n) dispatch with monomial extraction"
  - "anames() function listing all user-defined variables"
  - "restart() function clearing session state"
  - "Command::Restart for REPL restart command"
  - "Single-quote string literals in lexer for Maple unassign syntax"
  - "x := 'x' unassign logic removing variable bindings"
  - "14 integration tests covering all Phase 33 success criteria"
affects: [34-maple-compat]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "extract_symbol_id: promotes Value::Symbol to SymbolId via env.symbols.intern"
    - "extract_monomial_from_arg: extracts QMonomial from Symbol (var^1), Series (monomial), or Integer"
    - "Maple-style dispatch: detect Value::Symbol as first arg to switch between new and legacy signatures"
    - "Single-quote strings produce Token::StringLit (same as double-quote, no escape processing)"
    - "Unassign pattern: AstNode::Assign where value is StringLit matching name removes binding"

key-files:
  created: []
  modified:
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/lexer.rs
    - crates/qsym-cli/src/commands.rs
    - crates/qsym-cli/src/repl.rs
    - crates/qsym-cli/tests/cli_integration.rs

key-decisions:
  - "restart implemented as both Command (REPL) and dispatch function (scripts) for consistency"
  - "3-arg aqprod(monomial, var, n) uses n for both Pochhammer count and truncation order"
  - "Single-quote strings reuse Token::StringLit (no new token variant needed)"
  - "anames() returns sorted list of Value::String for consistent output"

patterns-established:
  - "Maple-style dispatch: check first arg type to select signature variant"
  - "Symbol extraction: extract_symbol_id helper interns symbol name to SymbolId"
  - "Monomial extraction: extract_monomial_from_arg handles Symbol, Series, Integer"

requirements-completed: [SYM-02, SYM-03]

# Metrics
duration: 7min
completed: 2026-02-19
---

# Phase 33 Plan 03: Symbol-Aware Function Dispatch Summary

**Maple-style etaq(q,1,20) and aqprod(q^2,q,5) dispatch with anames/restart/unassign variable management and single-quote lexer support**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-19T17:38:44Z
- **Completed:** 2026-02-19T17:46:01Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Implemented Maple-style function dispatch: etaq(q, 1, 20) and aqprod(q^2, q, 5) accept symbolic variable args
- Added extract_symbol_id and extract_monomial_from_arg helper functions for type-safe argument extraction
- Added anames() function returning sorted list of user-defined variable names
- Added restart() function (dispatch) and Command::Restart (REPL) for session reset
- Added single-quote string literal support in lexer for Maple unassign syntax
- Implemented x := 'x' unassign logic removing variable bindings
- Updated tab completion with restart command and anames/restart functions (83 canonical names)
- Added 14 new integration tests covering all Phase 33 success criteria
- All 365 unit tests + 72 integration tests pass (437 total)

## Task Commits

Each task was committed atomically:

1. **Task 1: Function dispatch with symbol args + single-quote lexer + variable management** - `1302c18` (feat)
2. **Task 2: End-to-end integration tests for all Phase 33 success criteria** - `5a5e07b` (test)

## Files Created/Modified
- `crates/qsym-cli/src/eval.rs` - extract_symbol_id, extract_monomial_from_arg, Maple-style etaq/aqprod dispatch, anames(), restart(), unassign logic, 7 new unit tests
- `crates/qsym-cli/src/lexer.rs` - Single-quote string literal tokenization, 3 new tests
- `crates/qsym-cli/src/commands.rs` - Command::Restart variant, parse/execute handlers, 5 new tests
- `crates/qsym-cli/src/repl.rs` - restart in command_names, anames/restart in canonical function names (83 total)
- `crates/qsym-cli/tests/cli_integration.rs` - 14 new integration tests for SYM-02, SYM-03, SYM-04, polynomials, variable management

## Decisions Made
- Implemented restart as both a Command (for REPL) and a dispatch function (for script mode), since scripts go through the expression parser, not the command handler
- 3-arg aqprod(monomial, var, n) uses n for both Pochhammer count and truncation order; 4-arg form exists for separate control
- Single-quote strings produce the same Token::StringLit as double-quote strings (no new token type), matching Maple's behavior where 'x' is literally the name x
- anames() returns sorted list to ensure deterministic output across runs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] restart() implemented as dispatch function for script mode**
- **Found during:** Task 1 (restart command implementation)
- **Issue:** Plan specified `restart` only as a Command, but the script executor does not call parse_command() -- it goes directly through the parser. Bare `restart:` in a script would be parsed as a variable, not trigger a reset.
- **Fix:** Added `restart()` as a 0-arg dispatch function in addition to Command::Restart. Integration test uses `restart()` syntax instead of bare `restart:`.
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** `restart_function_in_script` integration test passes
- **Committed in:** 1302c18 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Fix necessary for restart to work in scripts. No scope creep -- both REPL command and function form implemented.

## Issues Encountered
None beyond the deviation documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All Phase 33 requirements complete: SYM-01 (33-01), SYM-02 (33-03), SYM-03 (33-02 + 33-03), SYM-04 (33-01)
- Symbol variable foundation ready for v2.0 Maple compatibility phases
- 437 total tests (365 unit + 72 integration) provide comprehensive regression coverage
- Variable management (anames, restart, unassign) ready for researcher workflows

## Self-Check: PASSED

All 5 modified files verified present. Both commit hashes (1302c18, 5a5e07b) verified in git log.

---
*Phase: 33-symbolic-variable-foundation*
*Completed: 2026-02-19*
