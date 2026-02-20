---
phase: 43-expression-operations
plan: 01
subsystem: cli
tags: [eval_pow, rational, floor, legendre, number-theory, rug]

# Dependency graph
requires:
  - phase: 42-procedures-evaluation
    provides: "for-loop, procedure framework, Rational arithmetic"
provides:
  - "eval_pow Rational exponent arms (Symbol, Series, Integer, Rational, JacobiProduct)"
  - "floor() builtin function"
  - "legendre() builtin function with L alias"
  - "Number Theory category in help system"
affects: [44-display-formatting, 45-advanced-operations]

# Tech tracking
tech-stack:
  added: [rug::Integer::legendre, rug::Rational::floor_ref]
  patterns: [denom-check-then-delegate for Rational exponents]

key-files:
  created: []
  modified:
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/help.rs
    - crates/qsym-cli/src/repl.rs

key-decisions:
  - "Rational exponent arms all share denom==1 check pattern before delegating to existing Integer arms"
  - "floor uses rug::Integer::from(floor_ref()) for zero-copy floor computation"
  - "legendre validates p >= 3 and odd but does not check primality (matches Maple behavior)"
  - "L alias is case-insensitive (resolve_alias lowercases input)"

patterns-established:
  - "Pattern: Rational-to-Integer delegation -- check denom==1, extract numer, delegate to Integer arm"
  - "Pattern: Number Theory function group in dispatch, signatures, help, and completion"

requirements-completed: [SERIES-03, UTIL-01, UTIL-02]

# Metrics
duration: 9min
completed: 2026-02-20
---

# Phase 43 Plan 01: Expression Operations Summary

**Rational exponent support in eval_pow for all 5 Value types, plus floor() and legendre()/L() number-theory builtins with GMP integration**

## Performance

- **Duration:** 9 min
- **Started:** 2026-02-20T20:25:27Z
- **Completed:** 2026-02-20T20:34:08Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- eval_pow now handles Rational exponents for Symbol, Series, Integer, Rational, and JacobiProduct (fixes for-loop q^(n*n) type gap)
- floor() function returns correct integer floor for positive and negative rationals using GMP
- legendre(m, p) computes Legendre symbol using GMP's optimized algorithm, with L() alias
- Number Theory category added to help system with documentation for both functions
- Tab completion updated with floor and legendre (93 canonical function names)

## Task Commits

Each task was committed atomically:

1. **Task 1: eval_pow Rational exponent arms** - `d088ecc` (feat)
2. **Task 2: floor(), legendre(), L alias, signatures, help entries** - `f455234` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/eval.rs` - 5 new eval_pow Rational arms, floor/legendre dispatch, alias, signatures, ALL_FUNCTION_NAMES, 21 new tests
- `crates/qsym-cli/src/help.rs` - Number Theory category, 2 FuncHelp entries, L redirect, 4 new tests, count updated to 91
- `crates/qsym-cli/src/repl.rs` - Tab completion updated with floor/legendre (93 canonical names)

## Decisions Made
- Rational exponent arms use consistent denom==1 check before delegating to existing Integer arms (avoids code duplication)
- floor() uses rug::Integer::from(floor_ref()) rather than Assign trait (simpler, no trait import needed)
- legendre() validates p >= 3 and p is odd, but does not check primality (matches Maple/GMP behavior)
- L alias is lowercase in resolve_alias (consistent with all other alias handling)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated repl.rs canonical_function_names and count test**
- **Found during:** Task 2 (adding floor/legendre)
- **Issue:** repl.rs has its own canonical_function_names list and count test that would fail without the new functions
- **Fix:** Added "floor" and "legendre" to repl's canonical list, updated expected count from 91 to 93
- **Files modified:** crates/qsym-cli/src/repl.rs
- **Verification:** canonical_function_count test passes
- **Committed in:** f455234 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Auto-fix necessary to keep repl completion list in sync. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- 91 functions with help entries, 93 tab-completable names
- Rational exponent support enables for-loop series construction patterns from qmaple.pdf
- floor() and legendre() ready for use in advanced number-theoretic computations
- Test count: 551 unit + 152 integration = 703 total (was 682)

## Self-Check: PASSED

- [x] eval.rs exists and modified
- [x] help.rs exists and modified
- [x] repl.rs exists and modified
- [x] SUMMARY.md created
- [x] Commit d088ecc found (Task 1)
- [x] Commit f455234 found (Task 2)
- [x] 551 unit + 152 integration = 703 tests all passing

---
*Phase: 43-expression-operations*
*Completed: 2026-02-20*
