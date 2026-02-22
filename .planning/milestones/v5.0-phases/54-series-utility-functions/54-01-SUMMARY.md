---
phase: 54-series-utility-functions
plan: 01
subsystem: cli
tags: [coeff, degree, numer, denom, modp, mods, type, evalb, cat, maple-compat]

# Dependency graph
requires:
  - phase: 53-lists-list-operations
    provides: dispatch pattern, list operations infrastructure, function count baseline
provides:
  - 9 Maple-compatible utility functions: coeff, degree, numer, denom, modp, mods, type, evalb, cat
  - Series coefficient extraction via coeff(f, q, n)
  - Polynomial degree via degree(f, q)
  - Rational decomposition via numer/denom
  - Modular arithmetic via modp/mods
  - Type checking via type(expr, t)
  - Boolean evaluation via evalb(expr)
  - Name concatenation via cat(s1, s2, ...)
affects: [55-advanced-functions, 56-scripting-enhancements]

# Tech tracking
tech-stack:
  added: []
  patterns: [Series Coefficient & Utility dispatch group]

key-files:
  created: []
  modified:
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/help.rs
    - crates/qsym-cli/src/repl.rs
    - crates/qsym-cli/tests/cli_integration.rs

key-decisions:
  - "coeff returns Integer when denominator==1, Rational otherwise"
  - "type accepts both Symbol and String as type name argument"
  - "cat returns Value::Symbol (not String) matching Maple's cat behavior"
  - "modp/mods use i64 arithmetic with ((a%p)+p)%p pattern for correct negative handling"
  - "Unknown type names in type() return false (not error) for forward compatibility"

patterns-established:
  - "Pattern V: Series Coefficient & Utility dispatch group in eval.rs"

requirements-completed: [SERIES-01, SERIES-02, SERIES-03, UTIL-01, UTIL-02, UTIL-03, UTIL-04]

# Metrics
duration: 7min
completed: 2026-02-22
---

# Phase 54 Plan 01: Series & Utility Functions Summary

**9 Maple-compatible functions (coeff, degree, numer, denom, modp, mods, type, evalb, cat) with dispatch arms, help entries, tab completion, and 39 new tests**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-22T17:40:22Z
- **Completed:** 2026-02-22T17:47:26Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added 9 dispatch arms for Maple-compatible utility functions covering series coefficients, rational decomposition, modular arithmetic, type checking, boolean evaluation, and name concatenation
- 9 FUNC_HELP entries with signatures, descriptions, and examples; new "Series Coefficients & Utility" category in general help
- 39 new tests total: 16 dispatch unit tests, 9 parse+eval integration tests in eval.rs, and 14 CLI integration tests

## Task Commits

Each task was committed atomically:

1. **Task 1: Add 9 dispatch arms, signatures, and ALL_FUNCTION_NAMES** - `1103e8d` (feat)
2. **Task 2: Help entries, tab completion, general_help, count updates, and integration tests** - `3ddbfc2` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/eval.rs` - 9 dispatch arms, 9 signatures, 9 ALL_FUNCTION_NAMES entries, 25 unit/integration tests
- `crates/qsym-cli/src/help.rs` - 9 FUNC_HELP entries, new general_help category, count updates (103->112)
- `crates/qsym-cli/src/repl.rs` - 9 canonical_function_names entries for tab completion (105->114)
- `crates/qsym-cli/tests/cli_integration.rs` - 14 end-to-end integration tests

## Decisions Made
- coeff(f,q,n) returns Integer when coefficient denominator==1, Rational otherwise (follows existing pattern from op/series dispatch)
- type(expr, t) accepts both Symbol and String as the type name argument, matching Maple's flexibility
- cat() returns Value::Symbol (not Value::String) matching Maple's cat behavior where the result is a name
- modp/mods operate on i64 extracted values using ((a%p)+p)%p for correct negative handling
- Unknown type names in type() return false rather than an error, for forward compatibility

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Value assert_eq in dispatch unit tests**
- **Found during:** Task 1
- **Issue:** Value enum doesn't implement PartialEq, so assert_eq! on Values failed to compile
- **Fix:** Rewrote all 16 dispatch unit tests to use pattern matching instead of assert_eq!
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Committed in:** 1103e8d

**2. [Rule 1 - Bug] Fixed coeff_series integration test expected value**
- **Found during:** Task 2
- **Issue:** coeff(aqprod(q,q,inf,20), q, 5) returns 1 (not -1), since pentagonal number k=2 gives (-1)^2=1
- **Fix:** Updated test assertion from -1 to 1
- **Files modified:** crates/qsym-cli/tests/cli_integration.rs
- **Committed in:** 3ddbfc2

**3. [Rule 1 - Bug] Fixed type_series integration test aqprod call**
- **Found during:** Task 2
- **Issue:** aqprod(q,q,infinity) requires 4 args with explicit truncation order
- **Fix:** Changed to aqprod(q,q,infinity,20) with explicit truncation
- **Files modified:** crates/qsym-cli/tests/cli_integration.rs
- **Committed in:** 3ddbfc2

---

**Total deviations:** 3 auto-fixed (3 bugs)
**Impact on plan:** All auto-fixes necessary for correctness. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- 112 functions now have help entries, 114 in ALL_FUNCTION_NAMES and tab completion
- All 819 lib tests + 174 integration tests pass (1 pre-existing failure in err_05_read_nonexistent unrelated to this plan)
- Ready for Phase 54 Plan 02 or next phase

---
*Phase: 54-series-utility-functions*
*Completed: 2026-02-22*
