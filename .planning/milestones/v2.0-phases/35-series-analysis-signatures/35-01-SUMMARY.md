---
phase: 35-series-analysis-signatures
plan: 01
subsystem: api
tags: [maple-compat, sift, prodmake, etamake, jacprodmake, mprodmake, qetamake, qfactor, series-analysis]

# Dependency graph
requires:
  - phase: 34-product-theta-signatures
    provides: "extract_symbol_id helper, Maple-style dispatch pattern for q-series functions"
provides:
  - "Maple-style sift(s, q, n, k, T) with k-range validation"
  - "Maple-style prodmake/etamake/mprodmake/qetamake(f, q, T) 3-arg dispatch"
  - "Maple-style jacprodmake(f, q, T) and jacprodmake(f, q, T, P) with period filter"
  - "Maple-style qfactor(f, q) and qfactor(f, q, T)"
  - "Core jacprodmake_with_period_filter function"
affects: [35-02, 36-identity-proving-signatures]

# Tech tracking
tech-stack:
  added: []
  patterns: ["jacprodmake_impl internal dispatch with optional period_divisor"]

key-files:
  created: []
  modified:
    - crates/qsym-core/src/qseries/prodmake.rs
    - crates/qsym-core/src/qseries/mod.rs
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/help.rs

key-decisions:
  - "sift validates k range at CLI level (0 <= k < n), core sift normalizes j internally"
  - "sift truncates input series to T before calling core sift for Maple-accurate truncation"
  - "jacprodmake_impl refactored with period_divisor: Option<i64> for code reuse"
  - "qfactor accepts optional T arg for Maple compat but ignores it (already degree-bounded)"
  - "No backward compat -- old arg counts produce WrongArgCount errors"

patterns-established:
  - "Series analysis Maple dispatch: extract series at 0, symbol at 1, params at 2+"

requirements-completed: [SIG-08, SIG-09, SIG-10, SIG-11, SIG-12, SIG-13, SIG-14]

# Metrics
duration: 8min
completed: 2026-02-19
---

# Phase 35 Plan 01: Series Analysis Signatures Summary

**Maple-style dispatch for 7 series analysis functions (sift, prodmake, etamake, jacprodmake, mprodmake, qetamake, qfactor) with jacprodmake period filter and k-range validation**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-19T21:13:30Z
- **Completed:** 2026-02-19T21:21:04Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Core jacprodmake refactored with optional period_divisor parameter via jacprodmake_impl
- All 7 series analysis functions migrated to Garvan's exact Maple calling conventions
- sift gains k-range validation (0 <= k < n) and T-based truncation
- 9 unit tests pass (7 rewritten + 2 new), 380 CLI + 90 integration tests zero regressions
- Help system updated with Maple signatures and examples

## Task Commits

Each task was committed atomically:

1. **Task 1: Add optional period_divisor to core jacprodmake** - `72b5ce3` (feat)
2. **Task 2: Rewrite dispatch blocks and unit tests for all 7 functions** - `cbbc3ad` (feat)

## Files Created/Modified
- `crates/qsym-core/src/qseries/prodmake.rs` - Added jacprodmake_with_period_filter, refactored to jacprodmake_impl with optional period_divisor, added unit test
- `crates/qsym-core/src/qseries/mod.rs` - Re-exported jacprodmake_with_period_filter
- `crates/qsym-cli/src/eval.rs` - Rewrote 7 dispatch blocks to Maple signatures, updated get_signature, rewrote 7+2 unit tests, updated 2 integration tests
- `crates/qsym-cli/src/help.rs` - Updated signatures and examples for all 7 functions

## Decisions Made
- sift validates k range at CLI level (0 <= k < n) with descriptive error, while core sift normalizes j internally
- sift truncates input series to min(T, series.truncation_order) before calling core sift for Maple-accurate output
- jacprodmake_impl uses Option<i64> period_divisor to share code between filtered and unfiltered paths
- qfactor accepts optional T argument for Maple compatibility but ignores it since our qfactor is already degree-bounded
- No backward compatibility -- old arg counts produce WrongArgCount errors as specified

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed fps.coeffs() -> fps.iter() API mismatch**
- **Found during:** Task 2 (sift dispatch)
- **Issue:** Plan used `fps.coeffs()` but FormalPowerSeries uses `fps.iter()` for coefficient iteration
- **Fix:** Changed to `fps.iter()` which returns `Iterator<Item = (&i64, &QRat)>`
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Committed in:** cbbc3ad (Task 2 commit)

**2. [Rule 1 - Bug] Updated integration tests using old signatures**
- **Found during:** Task 2 (regression scan)
- **Issue:** Two integration tests (`integration_sift_partition_congruence`, `integration_qfactor_qbin`, variable persistence test with prodmake) used old arg counts
- **Fix:** Updated to Maple-style: `sift(f, q, 5, 4, 30)`, `qfactor(qbin(5,2,20), q)`, `prodmake(f, q, 10)`
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Committed in:** cbbc3ad (Task 2 commit)

**3. [Rule 2 - Missing Critical] Updated help system signatures**
- **Found during:** Task 2 (regression scan)
- **Issue:** help.rs still showed old signatures and examples for all 7 functions
- **Fix:** Updated all FuncHelp entries with Maple-style signatures and examples
- **Files modified:** crates/qsym-cli/src/help.rs
- **Committed in:** cbbc3ad (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (2 bug, 1 missing critical)
**Impact on plan:** All auto-fixes necessary for correctness. No scope creep.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 7 series analysis functions now use Maple-style signatures exclusively
- Phase 35 Plan 02 can proceed with remaining series analysis functions
- Pattern established for future function migrations: extract series at 0, symbol at 1, params at 2+

---
*Phase: 35-series-analysis-signatures*
*Completed: 2026-02-19*
