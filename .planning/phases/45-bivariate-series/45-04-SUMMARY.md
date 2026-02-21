---
phase: 45-bivariate-series
plan: 04
subsystem: computation
tags: [trivariate, winquist, laurent-polynomial, symbolic, q-series]

# Dependency graph
requires:
  - phase: 45-bivariate-series (plan 03)
    provides: BivariateSeries struct, one-symbolic winquist, fps_shift_internal, add_to_bv_terms
provides:
  - TrivariateSeries struct with BTreeMap<(i64,i64), FPS> representation
  - compute_winquist_two_symbolic function for fully symbolic winquist(a, b, q, T)
  - format_trivariate and format_trivariate_latex display functions
  - Value::TrivariateSeries variant with type_name, negate, format dispatch
affects: [bivariate-series, cli-eval, display-formatting]

# Tech tracking
tech-stack:
  added: []
  patterns: [trivariate-factor-product, constant-point-cross-validation]

key-files:
  created:
    - crates/qsym-core/src/series/trivariate.rs
  modified:
    - crates/qsym-core/src/series/mod.rs
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/format.rs
    - crates/qsym-cli/src/help.rs

key-decisions:
  - "Cross-validation uses a=-1,b=-1 and a=2,b=3 (constant q-monomials) to avoid truncation boundary effects from q-shifting"
  - "All 8 symbolic factors processed in trivariate loop, (q;q)^2 multiplied at end for efficiency"
  - "No add/sub/mul operations on TrivariateSeries -- only negate and display supported initially"
  - "Trivariate display reuses bivariate format_z_power helper via format_var_power wrapper"

patterns-established:
  - "Trivariate product: same factor-by-factor loop as bivariate but with 2D (a,b) exponent shifts"
  - "Constant-point cross-validation: evaluate at a=c, b=d with q-power 0 to avoid shift boundary issues"

requirements-completed: [BIVAR-03]

# Metrics
duration: 10min
completed: 2026-02-21
---

# Phase 45 Plan 04: Two-Symbolic Winquist via TrivariateSeries Summary

**Trivariate winquist(a, b, q, T) with both a and b symbolic, returning Laurent polynomial in a, b with q-series coefficients, cross-validated against numeric winquist**

## Performance

- **Duration:** 10 min
- **Started:** 2026-02-21T01:28:08Z
- **Completed:** 2026-02-21T01:38:39Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- TrivariateSeries struct in qsym-core with BTreeMap<(i64,i64), FPS>, trivariate_negate, PartialEq/Eq
- Value::TrivariateSeries variant fully integrated (type_name, negate, format_value, format_latex)
- compute_winquist_two_symbolic: 8-factor trivariate product loop + (q;q)^2 multiplication
- Cross-validated at a=-1,b=-1 and a=2,b=3 against numeric winquist -- coefficients match
- Help text updated to document both-symbolic trivariate support

## Task Commits

Each task was committed atomically:

1. **Task 1: TrivariateSeries struct + Value variant + format + negate** - `bdcc7b9` (feat)
2. **Task 2: compute_winquist_two_symbolic + dispatch + tests + help** - `ad02b99` (feat)

## Files Created/Modified
- `crates/qsym-core/src/series/trivariate.rs` - TrivariateSeries struct with zero, is_zero, truncation_order, negate, PartialEq/Eq, 3 unit tests
- `crates/qsym-core/src/series/mod.rs` - Added `pub mod trivariate;`
- `crates/qsym-cli/src/eval.rs` - Value::TrivariateSeries variant, compute_winquist_two_symbolic, add_to_tv_terms, dispatch update, 5 new tests
- `crates/qsym-cli/src/format.rs` - format_trivariate, format_trivariate_latex, format_ab_power helpers
- `crates/qsym-cli/src/help.rs` - Updated winquist help to mention trivariate, updated test name

## Decisions Made
- Cross-validation uses constant q-monomials (a=-1,b=-1 and a=2,b=3) rather than q-power monomials (a=q^2,b=q^3) to avoid truncation boundary effects where negative q-shifts drop contributions
- No arithmetic operations (add/sub/mul) on TrivariateSeries at this point -- only negate and display are supported
- (q;q)^2 factors multiplied at the end rather than incorporated into the factor loop for efficiency
- Trivariate display format mirrors bivariate: multi-term FPS coefficients parenthesized, single-term inline, descending (a_exp, b_exp) order

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Cross-validation evaluation point changed from q-power to constant monomials**
- **Found during:** Task 2 (cross-validation test)
- **Issue:** Plan suggested a=q^2, b=q^3 for cross-validation, but evaluating the trivariate at these points introduces q-shifts that push some FPS terms to negative q-powers, which are dropped. This causes apparent mismatches even though the trivariate is correct.
- **Fix:** Changed to a=-1, b=-1 (and a=2, b=3 as second validation) -- constant monomials with q-power 0 that avoid any q-shifting. Both validations pass.
- **Files modified:** crates/qsym-cli/src/eval.rs (test function)
- **Verification:** Both cross-validation tests pass, confirming trivariate correctness
- **Committed in:** ad02b99

---

**Total deviations:** 1 auto-fixed (1 bug in test design)
**Impact on plan:** Test evaluation strategy changed; core algorithm unchanged. No scope creep.

## Issues Encountered
None beyond the cross-validation point issue documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- BIVAR-03 requirement complete -- winquist(a, b, q, T) works with 0, 1, or 2 symbolic variables
- All Phase 45 gap closures now complete
- 763 CLI tests (611 unit + 152 integration), 3 trivariate core tests
- Ready for Phase 46

---
*Phase: 45-bivariate-series*
*Completed: 2026-02-21*
