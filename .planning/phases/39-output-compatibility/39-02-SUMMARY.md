---
phase: 39-output-compatibility
plan: 02
subsystem: testing
tags: [backward-compat, regression, integration-tests, legacy-signatures]

# Dependency graph
requires:
  - phase: 39-output-compatibility
    provides: "Descending power ordering in all series output"
  - phase: 34-product-theta-signatures
    provides: "Dual dispatch for etaq/aqprod/jacprod/tripleprod/quinprod/winquist/qbin"
  - phase: 35-series-analysis-signatures
    provides: "Maple-style sift/prodmake/etamake/qfactor dispatch"
  - phase: 36-relation-discovery-signatures
    provides: "Garvan-style findlincombo/findcong dispatch"
  - phase: 37-jacobi-products
    provides: "theta/jac2series/JAC functions"
  - phase: 38-analysis-discovery-functions
    provides: "checkmult/lqdegree0 functions"
provides:
  - "Dedicated backward_compat integration test section (21 tests)"
  - "Regression coverage for all v1.x legacy function signatures"
  - "Cross-validation proving legacy and Garvan etaq produce identical results"
affects: [40-testing-validation]

# Tech tracking
tech-stack:
  added: []
  patterns: ["backward_compat_ test naming prefix for regression suite"]

key-files:
  created: []
  modified:
    - crates/qsym-cli/tests/cli_integration.rs

key-decisions:
  - "winquist legacy is 7-arg (not 4-arg as plan stated) -- corrected test"
  - "etaq has no 4-arg legacy form -- skipped backward_compat_etaq_legacy_4arg"
  - "All tests validate output correctness (not just exit code 0)"

patterns-established:
  - "backward_compat_ prefix for dedicated regression tests"
  - "Cross-validation pattern: compute both forms, subtract, assert zero difference"

requirements-completed: [COMPAT-01, COMPAT-02]

# Metrics
duration: 5min
completed: 2026-02-20
---

# Phase 39 Plan 02: Backward Compatibility Tests Summary

**21 dedicated backward_compat integration tests verifying all v1.x legacy function signatures and Maple-compatible dispatch produce correct results**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-20T03:37:42Z
- **Completed:** 2026-02-20T03:42:49Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- 21 backward_compat integration tests covering all product/theta, analysis, discovery, and utility functions
- Every legacy v1.x signature tested: etaq(3-arg), aqprod(5-arg), jacprod(3-arg), tripleprod(4-arg), quinprod(4-arg), winquist(7-arg), qbin(3-arg), numbpart, partition_count
- Maple-signature analysis tests: sift, prodmake, etamake, qfactor
- Garvan-signature discovery tests: findlincombo, findcong
- New function tests: theta, jac2series, checkmult, lqdegree0
- Cross-validation test proves `etaq(1,1,20) - etaq(q,1,20) = O(q^20)` (zero difference)
- Full suite passes: 570 CLI tests + 863 core tests = 1433 total, zero failures

## Task Commits

Each task was committed atomically:

1. **Task 1: Add backward_compat tests for product/theta functions** - `4c055d8` (test)
2. **Task 2: Add backward_compat tests for analysis, discovery, and utility functions** - `6b297e7` (test)

## Files Created/Modified
- `crates/qsym-cli/tests/cli_integration.rs` - Added 21 backward_compat integration tests (232 lines)

## Decisions Made
- winquist legacy form is 7-arg (a_cn, a_cd, a_p, b_cn, b_cd, b_p, order), not 4-arg as plan stated -- corrected test accordingly
- etaq has no 4-arg legacy form (only 3-arg b,t,order) -- skipped backward_compat_etaq_legacy_4arg from plan
- All tests validate output correctness (O(q^T) truncation markers, coefficient values, result dicts) not just exit code 0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected winquist legacy arg count from 4 to 7**
- **Found during:** Task 1 (product/theta backward compat tests)
- **Issue:** Plan specified `winquist(1, 1, 1, 20)` (4-arg) but legacy winquist takes 7 args: (a_cn, a_cd, a_p, b_cn, b_cd, b_p, order)
- **Fix:** Changed test to `winquist(1, 1, 1, 1, 1, 1, 20)` (correct 7-arg form)
- **Files modified:** crates/qsym-cli/tests/cli_integration.rs
- **Verification:** Test passes and produces expected output
- **Committed in:** 4c055d8 (Task 1 commit)

**2. [Rule 1 - Bug] Removed non-existent etaq 4-arg legacy test**
- **Found during:** Task 1 (product/theta backward compat tests)
- **Issue:** Plan specified `backward_compat_etaq_legacy_4arg` for `etaq(1, 2, 1, 20)` but etaq only has 3-arg legacy form (b, t, order)
- **Fix:** Skipped this test entirely -- no 4-arg etaq form exists
- **Files modified:** N/A (test not added)
- **Verification:** etaq dispatch confirmed: 3-arg only
- **Committed in:** 4c055d8 (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs in plan specification)
**Impact on plan:** Both corrections necessary for test accuracy. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 39 (Output Compatibility) complete with both plans done
- Dedicated backward_compat regression suite provides safety net for Phase 40 (Testing & Validation)
- All 1433 tests pass with zero failures across core and CLI

---
*Phase: 39-output-compatibility*
*Completed: 2026-02-20*
