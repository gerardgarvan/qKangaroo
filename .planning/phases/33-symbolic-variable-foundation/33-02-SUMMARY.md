---
phase: 33-symbolic-variable-foundation
plan: 02
subsystem: eval, format, display
tags: [symbol-arithmetic, polynomial-display, variable-aware-formatting]

# Dependency graph
requires:
  - phase: 33-01
    provides: "Value::Symbol variant and q demotion from keyword"
provides:
  - "Symbol arithmetic: q^2, 2*q, q+1, (q+1)*(q+1) all work"
  - "POLYNOMIAL_ORDER sentinel for exact polynomial display"
  - "Variable-aware series formatting via SymbolRegistry"
  - "Polynomial display without O(...) truncation"
affects: [33-03-symbol-aware-functions]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "POLYNOMIAL_ORDER sentinel (1 billion) distinguishes polynomials from truncated series"
    - "value_to_series promotes Symbol/Integer/Rational to FPS for mixed arithmetic"
    - "format_series uses SymbolRegistry.name() for variable-aware display"
    - "value_to_constant_fps preserves series truncation order to maintain polynomial semantics"

key-files:
  created: []
  modified:
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/format.rs
    - crates/qsym-cli/src/main.rs
    - crates/qsym-cli/src/script.rs
    - crates/qsym-cli/src/commands.rs
    - crates/qsym-core/src/series/display.rs

key-decisions:
  - "POLYNOMIAL_ORDER = 1 billion as sentinel; min() with real orders gives correct behavior"
  - "value_to_constant_fps uses series truncation order (not env.default_order) to preserve polynomial semantics"
  - "Core Display impl also suppresses O(...) for polynomial sentinel (defense-in-depth)"
  - "format_value/format_latex accept &SymbolRegistry for variable name resolution"

patterns-established:
  - "Symbol promotion: Value::Symbol -> FPS monomial via value_to_series"
  - "Polynomial sentinel: truncation_order >= POLYNOMIAL_ORDER -> no O(...) in output"
  - "Variable-aware display: format_series uses symbols.name(fps.variable())"

requirements-completed: [SYM-03]

# Metrics
duration: 10min
completed: 2026-02-19
---

# Phase 33 Plan 02: Symbol Arithmetic and Polynomial Display Summary

**Symbol arithmetic (q^2, 2*q, polynomial expressions) with variable-aware formatting and O(...) suppression for exact polynomials**

## Performance

- **Duration:** 10 min
- **Started:** 2026-02-19T17:25:49Z
- **Completed:** 2026-02-19T17:35:33Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Implemented full symbol arithmetic: pow, mul, add, sub, div, negate for Value::Symbol
- Added POLYNOMIAL_ORDER sentinel constant (1 billion) for exact polynomial truncation
- Added symbol_to_series and value_to_series helper functions for FPS promotion
- Changed eval_binop/eval_add/eval_sub/eval_mul/eval_div/eval_pow/eval_negate signatures to &mut Environment
- Rewrote format_value and format_latex to accept &SymbolRegistry for variable name lookup
- Implemented format_series with variable-aware display and polynomial O(...) suppression
- Updated fps_to_latex and latex_term for variable names and polynomial detection
- Updated all call sites across main.rs, script.rs, commands.rs, and eval.rs tests
- Added O(...) suppression in core display.rs for polynomial sentinel
- Fixed value_to_constant_fps to preserve series truncation order for polynomial semantics
- All 350 unit tests, 58 integration tests, and 274 core tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement symbol arithmetic in eval.rs** - `14a3d8a` (feat)
2. **Task 2: Variable-aware series formatting and polynomial display** - `3a19a5a` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/eval.rs` - POLYNOMIAL_ORDER constant, symbol_to_series, value_to_series, Symbol arms in all arithmetic ops, 8 new tests, updated format_value calls
- `crates/qsym-cli/src/format.rs` - format_value/format_latex accept &SymbolRegistry, new format_series function, updated fps_to_latex/latex_term, 7 new tests
- `crates/qsym-cli/src/main.rs` - Updated format_value call to pass &env.symbols
- `crates/qsym-cli/src/script.rs` - Updated format_value call to pass &env.symbols
- `crates/qsym-cli/src/commands.rs` - Updated format_latex and format_value calls to pass &env.symbols
- `crates/qsym-core/src/series/display.rs` - Added O(...) suppression for polynomial sentinel in core Display

## Decisions Made
- POLYNOMIAL_ORDER = 1,000,000,000 as sentinel value: large enough to never interfere with real truncation orders, small enough to avoid overflow in min() comparisons
- value_to_constant_fps now takes (val, symbol_id, order) instead of (val, env), using the companion series' truncation order to preserve polynomial semantics when doing Series + scalar
- Core Display impl also suppresses O(...) for polynomial sentinel as defense-in-depth (the CLI format_series is primary)
- format_value/format_latex signatures changed to accept &SymbolRegistry, propagated through all internal helpers

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed polynomial + scalar losing polynomial semantics**
- **Found during:** Task 2 (E2E verification)
- **Issue:** `2*q^3 + q + 1` showed `O(q^20)` because value_to_constant_fps created constants with env.default_order (20) instead of the series' POLYNOMIAL_ORDER. The arithmetic::add min() operation then reduced the truncation to 20.
- **Fix:** Changed value_to_constant_fps signature to accept explicit (symbol_id, order) parameters. All Series+scalar/scalar-Series arms now pass the series' own truncation_order, preserving polynomial semantics.
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Committed in:** 3a19a5a (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Fix necessary for correctness. Without it, polynomial + scalar silently truncated to default_order.

## Issues Encountered
None beyond the deviation documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Symbol arithmetic is fully operational for Plan 33-03 (symbol-aware functions)
- Variable-aware formatting ready for functions like etaq(t, 1, 20) to display in variable t
- POLYNOMIAL_ORDER sentinel enables clean polynomial display across the codebase

## Self-Check: PASSED

All 6 modified files verified present. Both commit hashes (14a3d8a, 3a19a5a) verified in git log.

---
*Phase: 33-symbolic-variable-foundation*
*Completed: 2026-02-19*
