---
phase: 06-hypergeometric-series
plan: 01
subsystem: qseries
tags: [hypergeometric, q-pochhammer, eval-phi, eval-psi, bilateral, q-binomial-theorem, q-gauss]

# Dependency graph
requires:
  - phase: 03-core-qseries-partitions
    provides: "QMonomial, PochhammerOrder, aqprod, FormalPowerSeries, arithmetic"
provides:
  - "HypergeometricSeries struct for _r phi_s representation"
  - "BilateralHypergeometricSeries struct for _r psi_s representation"
  - "eval_phi function for term-by-term FPS evaluation of _r phi_s"
  - "eval_psi function for bilateral _r psi_s evaluation with positive/negative index parts"
  - "QMonomial arithmetic: mul, div, is_q_neg_power, try_sqrt, neg, is_zero, one, q"
  - "SummationResult and TransformationResult types for summation/transformation formulas"
  - "verify_transformation function for FPS-based identity verification"
affects: [06-02, 06-03, 06-04, python-api-hypergeometric]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "FPS-based term accumulation for hypergeometric evaluation"
    - "Ratio recurrence: term_{n+1} = term_n * ratio(n) via FPS multiplication"
    - "Pole detection for bilateral series negative-index Pochhammer symbols"

key-files:
  created:
    - "crates/qsym-core/src/qseries/hypergeometric.rs"
    - "crates/qsym-core/tests/qseries_hypergeometric_tests.rs"
  modified:
    - "crates/qsym-core/src/qseries/mod.rs"

key-decisions:
  - "FPS-based term accumulation (not direct coefficient) for eval_phi: handles general QMonomial parameters correctly"
  - "Single inversion per step (accumulate denominator factors, invert once) for efficiency"
  - "Pole detection in eval_psi_negative: skip terms where Pochhammer at negative order has pole (a.coeff==1 && 0<a.power<=m)"
  - "rug::Integer completed explicitly for sqrt multiplication comparison (MulIncomplete cannot be compared directly)"

patterns-established:
  - "HypergeometricSeries struct: upper/lower Vec<QMonomial> + argument QMonomial"
  - "eval_phi ratio recurrence: build FPS for each (1 - c*q^m) factor, multiply/invert, accumulate"
  - "Bilateral eval_psi: split into positive (ratio recurrence) and negative (direct aqprod) parts"

# Metrics
duration: 7min
completed: 2026-02-14
---

# Phase 6 Plan 1: Hypergeometric Series Infrastructure Summary

**eval_phi/eval_psi for _r phi_s and _r psi_s with QMonomial arithmetic, verified against q-binomial theorem and q-Gauss summation**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-14T15:29:15Z
- **Completed:** 2026-02-14T15:36:07Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- QMonomial arithmetic methods (mul, div, is_q_neg_power, try_sqrt, neg, is_zero, one, q) enable parameter manipulation for summation and transformation formulas
- eval_phi correctly evaluates both terminating and non-terminating _r phi_s series using FPS-based term accumulation
- eval_psi evaluates bilateral _r psi_s series with robust pole detection for negative-index Pochhammer symbols
- 18 integration tests verify correctness against closed-form products (q-binomial theorem, q-Gauss summation)

## Task Commits

Each task was committed atomically:

1. **Task 1: QMonomial arithmetic helpers and hypergeometric structs** - `b08d5ff` (feat)
2. **Task 2: Integration tests for eval_phi and eval_psi** - `fa9c7f6` (test)

## Files Created/Modified
- `crates/qsym-core/src/qseries/hypergeometric.rs` - HypergeometricSeries, BilateralHypergeometricSeries structs, eval_phi, eval_psi, SummationResult, TransformationResult, verify_transformation
- `crates/qsym-core/src/qseries/mod.rs` - QMonomial arithmetic methods (mul/div/is_q_neg_power/try_sqrt/neg/is_zero/one/q), hypergeometric module declaration and re-exports
- `crates/qsym-core/tests/qseries_hypergeometric_tests.rs` - 18 integration tests covering QMonomial arithmetic, eval_phi (1phi0, 2phi1 terminating/non-terminating, q-Gauss), eval_psi bilateral, struct methods

## Decisions Made
- **FPS-based term accumulation over direct coefficient tracking:** Each term ratio involves factors like (1 - c*q^m) which distribute across multiple q-powers. FPS multiplication handles this correctly for arbitrary QMonomial parameters, while direct coefficient tracking only works for pure rational parameters (no q-dependence).
- **Single denominator inversion per step:** Accumulate all denominator factors via FPS multiplication, then invert once, rather than inverting each factor separately. Reduces inversions from (s+1) to 1 per step.
- **Pole detection in bilateral negative terms:** When computing (a;q)_{-m} via aqprod, the shifted parameter can create a vanishing factor in the denominator (pole). Detect this by checking `a.coeff == 1 && 0 < a.power <= m` and skip the term.
- **rug::Integer MulIncomplete completion:** The `&num_sqrt * &num_sqrt` expression produces a `MulIncomplete` type that cannot be compared directly. Wrap in `rug::Integer::from()` to complete the lazy computation before comparison.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed rug MulIncomplete comparison in try_sqrt**
- **Found during:** Task 1
- **Issue:** `&num_sqrt * &num_sqrt == num` fails to compile because rug lazy arithmetic types don't implement PartialEq<Integer>
- **Fix:** Wrap multiplication result in `rug::Integer::from()` before comparison
- **Files modified:** crates/qsym-core/src/qseries/mod.rs
- **Committed in:** b08d5ff

**2. [Rule 1 - Bug] Added Pochhammer pole detection in eval_psi_negative**
- **Found during:** Task 2
- **Issue:** eval_psi panicked with "Cannot invert series with zero constant term" when computing (q^m;q)_{-n} where the shifted parameter hit q^0
- **Fix:** Added `has_negative_pochhammer_pole()` check that detects when `a.coeff==1 && 0 < a.power <= m`, skipping such terms
- **Files modified:** crates/qsym-core/src/qseries/hypergeometric.rs
- **Committed in:** fa9c7f6

---

**Total deviations:** 2 auto-fixed (2 bug fixes)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- eval_phi and eval_psi are ready for Plans 02-04 to build summation formulas and transformation formulas on top
- SummationResult and TransformationResult types are defined, ready for q-Gauss, q-Vandermonde, q-Saalschutz implementations
- verify_transformation function ready for Heine, Sears, Watson, Bailey transformation verification
- All 396 existing tests continue to pass (no regressions)

## Self-Check: PASSED

- [x] crates/qsym-core/src/qseries/hypergeometric.rs - FOUND
- [x] crates/qsym-core/src/qseries/mod.rs - FOUND
- [x] crates/qsym-core/tests/qseries_hypergeometric_tests.rs - FOUND
- [x] Commit b08d5ff - FOUND
- [x] Commit fa9c7f6 - FOUND

---
*Phase: 06-hypergeometric-series*
*Completed: 2026-02-14*
