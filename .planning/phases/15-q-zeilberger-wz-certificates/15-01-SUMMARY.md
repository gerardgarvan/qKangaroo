---
phase: 15-q-zeilberger-wz-certificates
plan: 01
subsystem: algorithms
tags: [q-zeilberger, creative-telescoping, wz-certificates, recurrence, linear-algebra]

# Dependency graph
requires:
  - phase: 14-q-gosper-algorithm
    provides: "extract_term_ratio, gosper_normal_form, GosperNormalForm, QRatRationalFunc"
  - phase: 13-polynomial-infrastructure
    provides: "QRatPoly, QRatRationalFunc, Lagrange interpolation support"
provides:
  - "ZeilbergerResult and QZeilbergerResult data types"
  - "build_shifted_series for n-direction parameter shifting"
  - "compute_rj_values for numeric R_j(k) = F(n+j,k)/F(n,k)"
  - "try_creative_telescoping core loop with direct term-value solver"
  - "q_zeilberger public API for definite q-hypergeometric summation"
  - "construct_certificate_from_g via Lagrange interpolation"
  - "detect_n_params heuristic for n-parameter detection"
affects: [15-02, 15-03, python-api]

# Tech tracking
tech-stack:
  added: []
  patterns: ["direct term-value linear system for creative telescoping", "Lagrange interpolation for WZ certificate construction"]

key-files:
  created:
    - "crates/qsym-core/src/qseries/zeilberger.rs"
  modified:
    - "crates/qsym-core/src/qseries/mod.rs"

key-decisions:
  - "Direct term-value approach instead of polynomial key equation evaluation for creative telescoping"
  - "Lagrange interpolation for WZ certificate construction from G(n,k) values"
  - "Duplicate private helpers (qrat_pow_i64, eval_qmonomial, solve_linear_system) from gosper.rs rather than making them public"
  - "Boundary conditions G(n,0)=0 and G(n,max_k+1)=0 for telescoping sum"

patterns-established:
  - "Direct term-value approach: solve G(n,k+1)-G(n,k) = sum c_j F(n+j,k) as a linear system rather than polynomial key equation"
  - "Certificate recovery: Lagrange interpolation from f(q^k) = g_k * c(q^k) / F(n,k) evaluation points"

# Metrics
duration: 45min
completed: 2026-02-16
---

# Phase 15 Plan 01: q-Zeilberger Creative Telescoping Core Summary

**Creative telescoping for definite q-hypergeometric summation via direct term-value linear system with Lagrange-interpolated WZ certificates**

## Performance

- **Duration:** 45 min
- **Started:** 2026-02-16
- **Completed:** 2026-02-16
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- q-Zeilberger algorithm finds order-1 recurrence for q-Vandermonde sum (primary success criterion)
- Direct term-value approach correctly handles terminating series where polynomial key equation evaluation fails
- WZ proof certificates constructed via Lagrange interpolation from G(n,k) boundary-value solutions
- 12 passing tests covering shift construction, R_j computation, creative telescoping for multiple series types
- Full test suite: 217 tests, zero regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Create zeilberger.rs with data types, n-shift ratios, and creative telescoping solver** - `c9393ff` (feat)

## Files Created/Modified
- `crates/qsym-core/src/qseries/zeilberger.rs` - Core q-Zeilberger module: data types, shift computation, creative telescoping, certificate construction, 12 tests (~1050 lines)
- `crates/qsym-core/src/qseries/mod.rs` - Module declaration and type re-exports

## Decisions Made

1. **Direct term-value approach instead of polynomial key equation evaluation:** The plan proposed evaluating the polynomial key equation sigma(x)*f(qx) - tau(x)*f(x) = tau(x)*c(x)*sum c_j*R_j(x) at x=q^k. This fails for terminating series because R_j(k) = F(n+j,k)/F(n,k) is undefined when F(n,k)=0 (beyond termination) but F(n+j,k) may still be non-zero for larger shifts. The WZ certificate has poles that cancel these zeros, which point evaluation cannot capture. Instead, the direct approach solves G(n,k+1) - G(n,k) = sum c_j * F(n+j,k) where G values and c_j are unknowns, with boundary conditions G(n,0)=0 and G(n,max_k+1)=0.

2. **Lagrange interpolation for WZ certificates:** After solving for G(n,k) values, the certificate R(q^k) = G(n,k)/F(n,k) is converted to a rational function f(x)/c(x) by computing f(q^k) = G(n,k)*c(q^k)/F(n,k) at known points and interpolating.

3. **Duplicate private helpers:** qrat_pow_i64, eval_qmonomial, and solve_linear_system are duplicated from gosper.rs (where they are private) rather than making them pub(crate), to avoid modifying Phase 14 code.

4. **Boundary conditions:** G(n,0)=0 (initial condition for telescoping) and G(n,max_k+1)=0 (forced by the finite support of terminating series) constrain the linear system.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Polynomial key equation approach fails for terminating series**
- **Found during:** Task 1 (creative telescoping implementation)
- **Issue:** The plan specified evaluating the polynomial key equation at x=q^k, but R_j(k) = F(n+j,k)/F(n,k) is 0/0 when F(n,k)=0 (k beyond termination). Setting R_j(k)=0 in this case loses the contribution F(n+j,k) which may be non-zero for larger shifts (j>0 extends the support). All 7 creative telescoping tests failed with this approach.
- **Fix:** Replaced the polynomial key equation evaluation with a direct term-value approach. Solve G(n,k+1)-G(n,k) = sum c_j*F(n+j,k) directly as a linear system with G values and c_j as unknowns. This correctly accounts for all term values including beyond the original series termination.
- **Files modified:** crates/qsym-core/src/qseries/zeilberger.rs
- **Verification:** All 12 tests pass, including q-Vandermonde at d=1 with n=3,5 and q=2,3
- **Committed in:** c9393ff

---

**Total deviations:** 1 auto-fixed (1 bug - fundamental algorithm approach)
**Impact on plan:** The deviation was essential for correctness. The direct term-value approach is mathematically equivalent but avoids the 0/0 singularity at termination points. No scope creep.

## Issues Encountered
- The polynomial key equation evaluation approach (plan's primary recommendation) fundamentally cannot handle terminating q-hypergeometric series because R_j(k) has 0/0 indeterminacy at termination. This required a complete redesign of the core solver from polynomial-based to term-value-based. The direct approach is simpler and more robust.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- ZeilbergerResult and QZeilbergerResult types exported for Plan 15-02 (public API, WZ verification, Python bindings)
- try_creative_telescoping is pub(crate) for Plan 15-02's q_zeilberger wrapper to use
- build_shifted_series and compute_rj_values are pub(crate) for direct testing in Plan 15-03
- Certificate construction via Lagrange interpolation ready for verification in Plan 15-02

## Self-Check: PASSED

- [x] crates/qsym-core/src/qseries/zeilberger.rs -- FOUND
- [x] crates/qsym-core/src/qseries/mod.rs -- FOUND
- [x] .planning/phases/15-q-zeilberger-wz-certificates/15-01-SUMMARY.md -- FOUND
- [x] Commit c9393ff -- FOUND
- [x] 217/217 tests pass -- VERIFIED

---
*Phase: 15-q-zeilberger-wz-certificates*
*Completed: 2026-02-16*
