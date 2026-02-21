---
phase: 45-bivariate-series
plan: 03
subsystem: cli
tags: [bivariate, winquist, pochhammer, dispatch, symbolic]

# Dependency graph
requires:
  - phase: 45-bivariate-series
    provides: BivariateSeries struct, bivariate arithmetic, tripleprod/quinprod bivariate
provides:
  - compute_winquist_one_symbolic via direct Pochhammer factor product
  - compute_pochhammer_bivariate for individual (c*z^p*q^s; q)_inf factors
  - Bivariate dispatch for winquist(z, b, q, T) with symbolic z
  - Bivariate dispatch for winquist(a, z, q, T) with symbolic z
  - Two-symbolic-variable error message for winquist
  - Updated winquist help text mentioning symbolic variable support
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: [direct-pochhammer-bivariate-with-q-shift, combined-factor-loop-with-global-shift]

key-files:
  created: []
  modified:
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/help.rs

key-decisions:
  - "Direct Pochhammer factor approach instead of tripleprod decomposition -- avoids mathematical issues with negative q-exponents in sum forms"
  - "Global q-shift in combined factor loop to handle negative q-offsets from (a/b;q)_inf factor"
  - "Cross-validation uses z=-1 (constant) not z=-q because bivariate FPS cannot represent negative q-exponents needed for z=c*q^m evaluation"
  - "Garvan convention: factor 6 is (q^2/(ab);q)_inf with offset 2-ap-bp, not (q/(ab);q)_inf"

patterns-established:
  - "Combined Pochhammer loop: multiply all bivariate factors in single loop with shared internal q-shift"
  - "Bivariate z=-1 validation: evaluate at constant z to avoid truncation boundary effects"

requirements-completed: [BIVAR-03]

# Metrics
duration: 15min
completed: 2026-02-20
---

# Phase 45 Plan 03: Winquist Bivariate Dispatch Summary

**Winquist bivariate dispatch via direct Pochhammer factor product with global q-shift, cross-validated at z=-1 against numeric computation**

## Performance

- **Duration:** 15 min (across two context windows)
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- compute_winquist_one_symbolic: 6 bivariate Pochhammer factors combined in single loop with global q-shift, multiplied by 4 concrete factors at end
- compute_pochhammer_bivariate: generic (c*z^p*q^s; q)_inf with internal q-shift for negative offsets
- Winquist dispatch detects symbolic a or b, routes to bivariate path, errors on two-symbolic
- 9 new tests: 2 dispatch, 2 cross-validation (zero-offset and negative-offset), 1 Pochhammer basic, 1 Pochhammer negative-offset, 1 two-symbolic error, 1 univariate preservation, 1 help text
- All 760 CLI tests pass (608 lib + 152 integration)

## Task Commits

Each task was committed atomically:

1. **Task 1: Winquist bivariate dispatch via direct Pochhammer factors** - `b745d11` (feat)
2. **Task 2: Winquist help text update for symbolic support** - `f0d1a52` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/eval.rs` - compute_winquist_one_symbolic, compute_pochhammer_bivariate, add_to_bv_terms, fps_shift_internal helper functions, updated winquist dispatch with Symbol detection, 8 new tests
- `crates/qsym-cli/src/help.rs` - Updated winquist description mentioning symbolic variable and bivariate output, 1 new test

## Decisions Made
- Used direct Pochhammer factor approach (6 bivariate factors computed in combined loop) instead of plan's suggested tripleprod decomposition, because the sum-form TP(a*b) and TP(a/b) have mathematical issues with negative q-exponents that cannot be represented in FPS
- Global q-shift technique: all internal FPS indices are offset by q_shift = |min_offset| * z_bound to accommodate negative true q-exponents from the (a/b;q)_inf factor
- Cross-validation at z=-1 instead of z=-q: evaluating a bivariate at z=c*q^m requires negative q-exponents (from positive z-exponents) that are not stored in the FPS representation; z=-1 avoids this issue entirely
- Garvan convention confirmed: factor 6 in the Winquist product is (q^2/(ab);q)_inf, giving offset 2-ap-bp (not 1-ap-bp)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tripleprod decomposition fails for negative q-exponents**
- **Found during:** Task 1
- **Issue:** Plan suggested using compute_tripleprod_bivariate_shifted for TP(ab) and TP(a/b), but these shifted sum forms produce terms with negative q-exponents (e.g., n*(n-1)/2 + shift*n < 0 for negative shift and moderate n) that cannot be stored in FPS
- **Fix:** Replaced with direct Pochhammer factor computation using a combined loop with global q-shift. Each of the 6 bivariate Pochhammer factors is multiplied in sequentially, with internal FPS indices offset to accommodate negative true q-exponents
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Committed in:** b745d11 (Task 1 commit)

**2. [Rule 1 - Bug] Factor 6 offset was initially wrong (1-bp instead of 2-bp)**
- **Found during:** Task 1
- **Issue:** Initially computed q^1/(ab) = (1/bc)*z^{-1}*q^{1-bp}, but the correct Garvan convention is q^2/(ab) giving offset 2-bp
- **Fix:** Verified against the numeric winquist implementation which uses offset 2-ap-bp, confirmed by Garvan's Winquist product definition
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Committed in:** b745d11 (Task 1 commit)

**3. [Rule 1 - Bug] Validation test used z=-q which requires negative q-coefficients**
- **Found during:** Task 1
- **Issue:** Test evaluated bivariate at z=-q (z^n = (-1)^n * q^n), which shifts q-exponents by z_exp. For positive z-exponents, this requires negative q-coefficients (q^{-n}) which are not stored
- **Fix:** Changed validation to use z=-1 (z^n = (-1)^n, no q-shift), consistent with the truncation boundary insight from plan 45-02
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Committed in:** b745d11 (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (3 bugs)
**Impact on plan:** Implementation approach changed from tripleprod decomposition to direct Pochhammer factor product. Core functionality and all requirements met.

## Issues Encountered
None beyond the mathematical issues documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 45 (Bivariate Series) is now complete: all 3 plans executed
- All 760 CLI tests pass (608 unit + 152 integration)
- BivariateSeries foundation, tripleprod/quinprod dispatch, and winquist dispatch all working

---
*Phase: 45-bivariate-series*
*Completed: 2026-02-20*
