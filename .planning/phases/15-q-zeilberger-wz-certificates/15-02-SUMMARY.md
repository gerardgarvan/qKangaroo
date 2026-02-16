---
phase: 15-q-zeilberger-wz-certificates
plan: 02
subsystem: algorithms
tags: [q-zeilberger, wz-certificate, creative-telescoping, recurrence, public-api]

# Dependency graph
requires:
  - phase: 15-q-zeilberger-wz-certificates
    plan: 01
    provides: "try_creative_telescoping, ZeilbergerResult, QZeilbergerResult, build_shifted_series, compute_rj_values, construct_certificate_from_g, detect_n_params"
  - phase: 14-q-gosper-algorithm
    provides: "extract_term_ratio, gosper_normal_form, GosperNormalForm, QRatRationalFunc"
provides:
  - "Public q_zeilberger function accessible via qsym_core::qseries::q_zeilberger"
  - "Public detect_n_params heuristic for n-parameter auto-detection"
  - "WZ certificate as QRatRationalFunc that verifies telescoping identity at k values"
  - "7 new tests covering all success criteria"
affects: [15-03, python-api]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Include boundary k=0 in Lagrange interpolation for correct WZ certificate"]

key-files:
  created: []
  modified:
    - "crates/qsym-core/src/qseries/zeilberger.rs"
    - "crates/qsym-core/src/qseries/mod.rs"

key-decisions:
  - "Include k=0 boundary condition (G(n,0)=0) in Lagrange interpolation for certificate construction"
  - "Return certificate directly from try_creative_telescoping (avoid double QRatRationalFunc::new reduction)"
  - "detect_n_params made fully public with documented limitations for non-standard series"

patterns-established:
  - "WZ certificate interpolation must include boundary points where G=0 for R(x) to evaluate correctly at all non-terminated k values"

# Metrics
duration: 6min
completed: 2026-02-16
---

# Phase 15 Plan 02: Public q_zeilberger API with WZ Certificate Summary

**Public q_zeilberger API with re-exports, WZ certificate verification via boundary-aware Lagrange interpolation, and 7 tests covering recurrence, inspection, and telescoping identity**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-16T17:31:31Z
- **Completed:** 2026-02-16T17:37:19Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- q_zeilberger and detect_n_params publicly exported from qsym_core::qseries (SUCCESS CRITERION 1)
- ZeilbergerResult coefficients are inspectable QRat values (SUCCESS CRITERION 2)
- WZ certificate verifies telescoping identity sum_j c_j*F(n+j,k) = G(n,k+1)-G(n,k) at k=0,1,2 (SUCCESS CRITERION 3)
- 224 total tests (19 zeilberger), zero regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement q_zeilberger public function with WZ certificate extraction** - `fc22cfc` (feat)

## Files Created/Modified
- `crates/qsym-core/src/qseries/zeilberger.rs` - Public API, certificate bug fix, 7 new tests (~310 lines added)
- `crates/qsym-core/src/qseries/mod.rs` - Re-export q_zeilberger and detect_n_params, updated module docs

## Decisions Made

1. **Include k=0 boundary in Lagrange interpolation:** The original certificate construction (from 15-01) only interpolated at k=1..max_k where F(n,k)!=0. This missed the crucial boundary condition G(n,0)=0, causing R(q^0) to be non-zero instead of 0. Adding the (q^0, 0) interpolation point fixes the certificate so it correctly represents G(n,k)=R(q^k)*F(n,k) at all non-terminated k values.

2. **Return certificate directly from try_creative_telescoping:** The original code returned (coefficients, f_poly, gnf) and q_zeilberger re-constructed QRatRationalFunc::new(f_poly, gnf.c). Since QRatRationalFunc::new auto-reduces via poly_gcd, the double-construction could produce incorrect results. Now try_creative_telescoping returns the certificate directly.

3. **detect_n_params made fully public:** The heuristic function is useful for users who want automatic n-parameter detection. Documented limitations for non-standard series where users should provide indices explicitly.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Certificate Lagrange interpolation missing k=0 boundary**
- **Found during:** Task 1 (certificate verification test)
- **Issue:** construct_certificate_from_g only interpolated R(q^k) at k=1..max_k, missing the boundary G(n,0)=0. This caused R(q^0) != 0, breaking the WZ identity at k=0.
- **Fix:** Added (q^0, 0) as the first interpolation point, ensuring f(q^0) = 0 and R(1) = 0.
- **Files modified:** crates/qsym-core/src/qseries/zeilberger.rs
- **Committed in:** fc22cfc

**2. [Rule 1 - Bug] Double QRatRationalFunc construction causes incorrect certificate**
- **Found during:** Task 1 (certificate verification test)
- **Issue:** try_creative_telescoping built certificate_rf, extracted its numerator, returned it. q_zeilberger then wrapped it again with QRatRationalFunc::new(), which auto-reduces via poly_gcd -- potentially destroying the numerator/denominator relationship.
- **Fix:** Changed try_creative_telescoping to return the certificate QRatRationalFunc directly. q_zeilberger uses it as-is.
- **Files modified:** crates/qsym-core/src/qseries/zeilberger.rs
- **Committed in:** fc22cfc

---

**Total deviations:** 2 auto-fixed (2 bugs in certificate construction)
**Impact on plan:** Both fixes were essential for certificate correctness. No scope creep.

## Issues Encountered
- Certificate verification initially failed because the Lagrange interpolation lacked the k=0 boundary point. The direct term-value approach from 15-01 correctly computes G(n,k) values, but the interpolation step to recover R(x) as a rational function required all relevant evaluation points including boundaries.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- q_zeilberger is public and fully tested for Plan 15-03 (Python bindings, additional test cases)
- WZ certificate construction is verified and correct for non-terminated k values
- detect_n_params provides automatic parameter detection for common series forms

## Self-Check: PASSED

- [x] crates/qsym-core/src/qseries/zeilberger.rs -- FOUND
- [x] crates/qsym-core/src/qseries/mod.rs -- FOUND
- [x] Commit fc22cfc -- FOUND
- [x] 224/224 tests pass -- VERIFIED

---
*Phase: 15-q-zeilberger-wz-certificates*
*Completed: 2026-02-16*
