---
phase: 02-simplification-series-engine
plan: 01
subsystem: series-engine
tags: [rust, formal-power-series, btreemap, sparse, truncation, q-series, tdd]

# Dependency graph
requires:
  - phase: 01-expression-foundation
    provides: QRat arbitrary-precision rationals, SymbolId for variable identity, ExprArena for symbol interning
provides:
  - FormalPowerSeries struct with BTreeMap<i64, QRat> sparse storage
  - Series arithmetic (add, sub, mul, negate, scalar_mul, invert, shift)
  - Display formatting for FPS ("1 - q + 2*q^3 + O(q^10)")
  - 33 TDD tests covering all operations
affects:
  - 02-02 (simplification engine may need series expansion for verification)
  - 03-core-qseries (q-Pochhammer expansion produces FormalPowerSeries)
  - 03-partitions (partition generating function uses invert on Euler function)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Sparse FPS via BTreeMap<i64, QRat> -- only nonzero coefficients stored"
    - "Truncation enforced DURING multiplication (break inner loop when ka+kb >= trunc)"
    - "Binary ops use min(a.truncation_order, b.truncation_order)"
    - "Inversion via recurrence c[n] = (-1/a0) * sum a[k]*c[n-k]"
    - "Zero-cleaning after every operation: set_coeff removes zeros, retain after mul"

key-files:
  created:
    - crates/qsym-core/src/series/mod.rs
    - crates/qsym-core/src/series/arithmetic.rs
    - crates/qsym-core/src/series/display.rs
    - crates/qsym-core/tests/series_tests.rs
  modified:
    - crates/qsym-core/src/lib.rs

key-decisions:
  - "Hardcoded 'q' as display variable name -- no SymbolRegistry access in Display impl; Phase 3+ can add display_with_arena"
  - "Shift adjusts truncation_order by k (shift(f, k) has trunc = f.trunc + k) -- semantically correct for q^k * f(q)"
  - "PartialEq compares variable, truncation_order, and coefficient maps (after zero removal) -- no structural canonicalization needed"
  - "pub(crate) fields on FPS -- arithmetic module accesses coefficients directly for performance, external users go through API"

patterns-established:
  - "Construct FPS via zero/one/monomial/from_coeffs factory methods, never directly"
  - "Access coefficients via coeff(k) which panics at/above truncation -- fail-fast on misuse"
  - "All binary series ops assert same variable -- no silent cross-variable arithmetic"
  - "Multiplication truncates during computation (O(N) space), never creates O(N^2) intermediates"

# Metrics
duration: 4min
completed: 2026-02-13
---

# Phase 2 Plan 01: FPS Data Structure and Series Arithmetic Summary

**Sparse FormalPowerSeries with BTreeMap coefficients, truncated arithmetic (add/sub/mul/negate/scalar_mul/invert/shift), and human-readable Display -- verified by 33 TDD tests including (1-q)(1+q)=1-q^2 and 1/(1-q) geometric series**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-13T22:25:27Z
- **Completed:** 2026-02-13T22:29:44Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- FormalPowerSeries struct with BTreeMap<i64, QRat> sparse storage and explicit truncation order tracking
- All 7 arithmetic operations implemented: add, sub, mul, negate, scalar_mul, invert, shift
- Multiplication truncates DURING computation via early break on sorted BTreeMap iteration -- O(N) space, not O(N^2)
- Inversion via coefficient recurrence verified: 1/(1-q) produces all-ones geometric series to any precision
- Display formats series as "1 - q + 2*q^3 + O(q^10)" with proper sign handling and coefficient omission
- 33 new tests pass, 203 total tests pass with zero regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: RED - FormalPowerSeries struct and comprehensive failing tests** - `5d0dc53` (test)
2. **Task 2: GREEN - Implement all FPS arithmetic, pass all tests** - `bf9072d` (feat)

_TDD: Task 1 created 33 tests (10 passing for construction, 23 failing for stubs). Task 2 implemented all stubs, making all 33 pass._

## Files Created/Modified

- `crates/qsym-core/src/series/mod.rs` - FormalPowerSeries struct with zero/one/monomial/from_coeffs constructors, coeff/set_coeff accessors, PartialEq
- `crates/qsym-core/src/series/arithmetic.rs` - add, sub, negate, scalar_mul, mul (truncated), invert (recurrence), shift operations
- `crates/qsym-core/src/series/display.rs` - Display impl: "1 - q + 2*q^3 + O(q^10)" format
- `crates/qsym-core/src/lib.rs` - Added `pub mod series;` module declaration
- `crates/qsym-core/tests/series_tests.rs` - 33 tests: 9 construction, 4 add, 3 sub/negate, 3 scalar_mul, 5 mul, 3 invert, 2 shift, 4 display

## Decisions Made

1. **Hardcoded 'q' in Display** -- Display impl has no access to SymbolRegistry (would require arena reference). Using "q" as the variable name since that is the primary use case. A `display_with_arena` method can be added in Phase 3+ when needed.

2. **Shift adjusts truncation_order** -- shift(f, k) produces a series with trunc = f.trunc + k. This is semantically correct: if f is known to O(q^T), then q^k * f is known to O(q^{T+k}).

3. **pub(crate) fields** -- FormalPowerSeries fields are pub(crate) so arithmetic.rs can access coefficients directly for performance. External users go through the public API (coeff, set_coeff, iter).

4. **PartialEq is value equality** -- Two FPS are equal iff same variable, same truncation_order, and identical coefficient maps. This is straightforward since zero-removal is enforced.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- FormalPowerSeries is ready for use by q-Pochhammer expansion (Phase 3)
- Inversion works correctly for partition generating function 1/(q;q)_inf
- Plan 02-02 (simplification engine) can proceed independently
- Display provides human-readable output for debugging series computations

## Self-Check: PASSED

- crates/qsym-core/src/series/mod.rs: FOUND
- crates/qsym-core/src/series/arithmetic.rs: FOUND
- crates/qsym-core/src/series/display.rs: FOUND
- crates/qsym-core/tests/series_tests.rs: FOUND
- Commit 5d0dc53 (Task 1): FOUND
- Commit bf9072d (Task 2): FOUND
- `cargo test` passes all 203 tests with zero failures

---
*Phase: 02-simplification-series-engine*
*Completed: 2026-02-13*
