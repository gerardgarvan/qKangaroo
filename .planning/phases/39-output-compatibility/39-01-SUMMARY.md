---
phase: 39-output-compatibility
plan: 01
subsystem: ui
tags: [formatting, display, descending-order, polynomial, series, latex]

# Dependency graph
requires:
  - phase: 33-symbol-foundation
    provides: "format_series() and fps_to_latex() in format.rs"
provides:
  - "Descending power ordering in all series/polynomial output (plain + LaTeX)"
  - "DoubleEndedIterator on FormalPowerSeries::iter() for .rev() support"
affects: [40-testing-validation, manual, documentation]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Descending polynomial display via .rev() on BTreeMap iterator"]

key-files:
  created: []
  modified:
    - crates/qsym-cli/src/format.rs
    - crates/qsym-cli/src/help.rs
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/tests/cli_integration.rs
    - crates/qsym-core/src/series/mod.rs

key-decisions:
  - "FormalPowerSeries::iter() return type changed from impl Iterator to impl DoubleEndedIterator to enable .rev()"
  - "fps_to_latex() uses iter().rev().collect() to reverse terms Vec once, then existing iteration code works unchanged"

patterns-established:
  - "Series output uses descending power ordering (highest power first, constant last)"

requirements-completed: [OUT-03, COMPAT-02]

# Metrics
duration: 7min
completed: 2026-02-20
---

# Phase 39 Plan 01: Descending Power Ordering Summary

**Maple-style descending polynomial display via .rev() on BTreeMap iterator in format_series() and fps_to_latex()**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-20T03:28:16Z
- **Completed:** 2026-02-20T03:35:22Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- All series and polynomial output now displays in descending power order (q^N + ... + q + 1)
- Both plain text format_series() and LaTeX fps_to_latex() use reversed iteration
- O(q^T) truncation marker still appears at the end of output
- All 418 CLI unit tests, 131 integration tests, and 281 core tests pass with zero regressions
- ~30 help example_output strings updated to reflect descending ordering

## Task Commits

Each task was committed atomically:

1. **Task 1: Update all test assertions and help examples to descending order** - `37856cb` (test)
2. **Task 2: Implement descending iteration in format_series() and fps_to_latex()** - `23d7fa1` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/format.rs` - Added .rev() to format_series() and fps_to_latex() iterators; updated 2 unit test assertions
- `crates/qsym-cli/src/help.rs` - Updated ~30 example_output strings from ascending to descending ordering
- `crates/qsym-cli/src/eval.rs` - Updated 1 integration test assertion for descending order
- `crates/qsym-cli/tests/cli_integration.rs` - Updated 3 test assertions (qbin, theta, jac2series) for descending order
- `crates/qsym-core/src/series/mod.rs` - Changed FormalPowerSeries::iter() return type to impl DoubleEndedIterator

## Decisions Made
- Changed FormalPowerSeries::iter() return type from `impl Iterator` to `impl DoubleEndedIterator` to enable .rev() -- cleaner than collecting to Vec and reversing in each caller
- fps_to_latex() uses `fps.iter().rev().collect()` to reverse the terms Vec once, then existing iteration code (first N terms, ellipsis, last terms) works unchanged with descending semantics

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated eval.rs test assertion for descending order**
- **Found during:** Task 2 (format code change)
- **Issue:** eval.rs `integration_format_etaq_starts_with_1` test expected ascending output
- **Fix:** Renamed to `integration_format_etaq_descending_order`, changed assertion to check for `+ 1 + O(q^20)` pattern
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** All 418 CLI unit tests pass
- **Committed in:** 23d7fa1 (Task 2 commit)

**2. [Rule 3 - Blocking] Changed FormalPowerSeries::iter() return type to DoubleEndedIterator**
- **Found during:** Task 2 (format code change)
- **Issue:** fps.iter() returned `impl Iterator` which doesn't support .rev(); BTreeMap::iter() does support DoubleEndedIterator but the wrapper hid this
- **Fix:** Changed return type to `impl DoubleEndedIterator<Item = (&i64, &QRat)>`
- **Files modified:** crates/qsym-core/src/series/mod.rs
- **Verification:** All 281 core tests + 418 CLI tests pass
- **Committed in:** 23d7fa1 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Descending output ordering complete, ready for Phase 39 Plan 02 (coefficient display formatting)
- All tests pass with zero regressions across core and CLI

---
*Phase: 39-output-compatibility*
*Completed: 2026-02-20*
