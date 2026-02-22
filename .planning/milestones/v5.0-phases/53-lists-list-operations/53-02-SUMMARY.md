---
phase: 53-lists-list-operations
plan: 02
subsystem: cli
tags: [evaluator, dispatch, list-operations, nops, op, map, sort]

# Dependency graph
requires:
  - phase: 53-lists-list-operations
    plan: 01
    provides: "List indexing (AstNode::Index), Value::List, 1-indexed access"
provides:
  - "nops(expr) -- count operands of lists, series, scalars"
  - "op(i, expr) -- extract i-th operand (1-indexed), series returns [exp, coeff]"
  - "map(f, list) -- apply builtin, procedure, or lambda to each list element"
  - "sort(list) -- ascending sort for numeric, symbol, and string values"
  - "compare_values_for_sort helper for mixed Integer/Rational ordering"
affects: [53-03, lists, list-operations]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "List operation dispatch: nops/op/map/sort match arms in Pattern U section"
    - "map calls call_procedure for Value::Procedure, dispatch for Value::Symbol"
    - "sort uses closure-based error propagation (sort_error Option<String>)"
    - "compare_values_for_sort: cross-type Integer/Rational comparison via QRat::from"

key-files:
  created: []
  modified:
    - "crates/qsym-cli/src/eval.rs"
    - "crates/qsym-cli/src/help.rs"
    - "crates/qsym-cli/src/repl.rs"

key-decisions:
  - "nops on series counts nonzero terms (FPS stores only nonzero coefficients)"
  - "op on series returns [exponent, coefficient] list (Maple-compatible)"
  - "map accepts Symbol (dispatches via eval) or Procedure (calls call_procedure)"
  - "sort error handling via closure captures sort_error to defer EvalError propagation"

patterns-established:
  - "List operation functions follow expect_args + match on Value type pattern"
  - "compare_values_for_sort returns Option<Ordering> for type-safe mixed comparisons"

requirements-completed: [LIST-01, LIST-02, LIST-03, LIST-04]

# Metrics
duration: 7min
completed: 2026-02-22
---

# Phase 53 Plan 02: List Operation Functions Summary

**nops/op/map/sort dispatch functions with mixed-type sorting, lambda/proc map, series operand extraction, help entries, and 105 tab-completable functions**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-22T08:26:45Z
- **Completed:** 2026-02-22T08:33:54Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added 4 list operation functions (nops, op, map, sort) to the CLI dispatch system
- nops works on lists, series, integers, rationals, symbols, and bivariate series
- op extracts i-th operand with Maple-compatible [exponent, coefficient] for series terms
- map supports builtin functions (via Symbol dispatch), user-defined procedures, and lambdas
- sort handles integers, rationals, mixed numeric, symbols, and strings with proper error reporting
- 16 new dispatch unit tests + 4 integration tests (parse+eval), all 794 tests passing
- Help entries and tab completion for all 4 functions (103 FUNC_HELP, 105 canonical names)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add nops, op, map, sort dispatch entries and ALL_FUNCTION_NAMES** - `d4e0224` (feat)
2. **Task 2: Help entries, tab completion, function count updates** - `0befb9c` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/eval.rs` - 4 dispatch arms (Pattern U), compare_values_for_sort helper, get_signature entries, ALL_FUNCTION_NAMES (90), 16 new tests
- `crates/qsym-cli/src/help.rs` - 4 FuncHelp entries (Group U), "List Operations:" category in general_help, 103 count, 5 new tests
- `crates/qsym-cli/src/repl.rs` - 4 entries in canonical_function_names (105 total)

## Decisions Made
- nops on FPS uses iter().count() since FPS stores only nonzero coefficients (no filter needed)
- BivariateSeries nops accesses .terms field directly (no .iter() method, unlike FPS)
- op uses coeff.denom() == 1 check instead of is_integer() (QRat lacks that method)
- map dispatches Symbol names through the full dispatch() function for Maple compatibility
- sort defers error via Option<String> in closure since sort_by can't return Result

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] BivariateSeries uses .terms.iter() instead of .iter()**
- **Found during:** Task 1
- **Issue:** Plan suggested `bvs.iter()` but BivariateSeries has no `iter()` method; uses `.terms` BTreeMap directly
- **Fix:** Changed to `bvs.terms.iter()`
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** All tests pass
- **Committed in:** d4e0224

**2. [Rule 1 - Bug] Used denom() == 1 instead of is_integer() for QRat**
- **Found during:** Task 1
- **Issue:** Plan suggested `coeff.is_integer()` but QRat has no `is_integer()` method
- **Fix:** Used `*coeff.denom() == 1` to check if rational is an integer
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** op series test passes with correct Integer/Rational distinction
- **Committed in:** d4e0224

---

**Total deviations:** 2 auto-fixed (2 bugs in plan pseudocode)
**Impact on plan:** Minor API corrections. No scope creep.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- List operations complete: nops, op, map, sort all working with help and tab completion
- Ready for list construction functions (append, seq, etc.) in plan 03

---
*Phase: 53-lists-list-operations*
*Completed: 2026-02-22*
