---
phase: 45-bivariate-series
plan: 01
subsystem: core, cli
tags: [bivariate, series, laurent, arithmetic, fps]

# Dependency graph
requires:
  - phase: 01-core-engine
    provides: FormalPowerSeries, arithmetic module, QRat, SymbolRegistry
provides:
  - BivariateSeries struct in qsym-core (series::bivariate)
  - Value::BivariateSeries variant in CLI evaluator
  - Bivariate arithmetic dispatch (add, sub, mul, negate, scalar_mul, fps_mul)
  - Display formatting (plain text and LaTeX) for bivariate series
affects: [45-02, 45-03, tripleprod, quinprod, winquist]

# Tech tracking
tech-stack:
  added: []
  patterns: [bivariate-as-btreemap-of-fps, laurent-polynomial-display]

key-files:
  created:
    - crates/qsym-core/src/series/bivariate.rs
  modified:
    - crates/qsym-core/src/series/mod.rs
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/format.rs

key-decisions:
  - "BivariateSeries uses BTreeMap<i64, FPS> for Laurent polynomial representation"
  - "Arithmetic follows free-function pattern matching series::arithmetic module"
  - "format_series made pub(crate) for reuse in bivariate coefficient display"
  - "Multi-term FPS coefficients parenthesized in display, single-term inline"
  - "Truncation propagation uses min(a, b) consistently across all operations"

patterns-established:
  - "Bivariate arithmetic: free functions matching series::arithmetic pattern"
  - "Mixed dispatch: BivariateSeries + scalar creates z^0 term wrapper"
  - "Laurent display: descending z-exponent order with parenthesized multi-term FPS"

requirements-completed: [BIVAR-04]

# Metrics
duration: 7min
completed: 2026-02-20
---

# Phase 45 Plan 01: BivariateSeries Foundation Summary

**BivariateSeries struct with BTreeMap<i64, FPS> Laurent representation, full arithmetic, CLI Value variant, and text/LaTeX display formatting**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-20T23:08:10Z
- **Completed:** 2026-02-20T23:15:42Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- BivariateSeries struct in qsym-core with 6 arithmetic operations (negate, add, sub, mul, scalar_mul, fps_mul)
- Value::BivariateSeries variant in CLI with full arithmetic dispatch including mixed types
- Display formatting for bivariate series as Laurent polynomials in plain text and LaTeX
- 22 new tests (13 core + 5 eval + 4 format) with zero regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: BivariateSeries struct and arithmetic in qsym-core** - `d78f90e` (feat)
2. **Task 2: Value::BivariateSeries variant, arithmetic dispatch, and display formatting** - `2b50914` (feat)

## Files Created/Modified
- `crates/qsym-core/src/series/bivariate.rs` - BivariateSeries struct, arithmetic functions, 13 unit tests
- `crates/qsym-core/src/series/mod.rs` - Added `pub mod bivariate` export
- `crates/qsym-cli/src/eval.rs` - Value::BivariateSeries variant, negate/add/sub/mul dispatch, 5 eval tests
- `crates/qsym-cli/src/format.rs` - format_bivariate and format_bivariate_latex functions, 4 format tests

## Decisions Made
- BivariateSeries uses BTreeMap<i64, FormalPowerSeries> mapping z-exponents to q-series coefficients
- Arithmetic follows free-function pattern (not trait impls) matching existing series::arithmetic module
- format_series promoted to pub(crate) for reuse in bivariate coefficient display
- Multi-term FPS coefficients displayed parenthesized; single-term displayed inline without parens
- Truncation propagation consistently uses min(a, b) across all binary operations

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- BivariateSeries foundation complete for tripleprod/quinprod/winquist dispatch (plan 45-02)
- All 741 tests pass (589 unit + 152 integration)

---
*Phase: 45-bivariate-series*
*Completed: 2026-02-20*
