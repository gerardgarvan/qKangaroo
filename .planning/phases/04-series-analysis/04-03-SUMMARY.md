---
phase: 04-series-analysis
plan: 03
subsystem: series-analysis
tags: [linear-algebra, gaussian-elimination, null-space, modular-arithmetic, qrat]

# Dependency graph
requires:
  - phase: 01-expression-foundation
    provides: "QRat exact rational arithmetic (zero, one, is_zero, arithmetic ops)"
  - phase: 02-simplification-series-engine
    provides: "FormalPowerSeries with coeff() and truncation_order()"
provides:
  - "rational_null_space: RREF-based kernel computation over Q"
  - "build_coefficient_matrix: extract FPS coefficients into matrix form"
  - "modular_null_space: kernel computation over Z/pZ"
  - "mod_pow: modular exponentiation helper"
affects: [04-04, 04-05, 04-06, 04-07, 04-08, 04-09, 04-10]

# Tech tracking
tech-stack:
  added: []
  patterns: [rref-gaussian-elimination, null-space-via-free-variables, modular-fermat-inverse]

key-files:
  created:
    - crates/qsym-core/src/qseries/linalg.rs
    - crates/qsym-core/tests/qseries_linalg_tests.rs
  modified:
    - crates/qsym-core/src/qseries/mod.rs

key-decisions:
  - "Fermat's little theorem for modular inverse (a^{p-2} mod p) rather than extended Euclidean"
  - "i128 intermediates in mod_mul to prevent overflow in modular arithmetic"
  - "Null space basis uses free-variable-equals-1 convention (canonical form)"

patterns-established:
  - "RREF null space pattern: row-reduce, identify pivot/free columns, build basis with 1 in free position and -rref[row][free_col] in pivot positions"
  - "Verification pattern: every null space test checks A*v=0 for all returned basis vectors"

# Metrics
duration: 4min
completed: 2026-02-14
---

# Phase 4 Plan 3: Rational Linear Algebra Summary

**Exact rational and modular Gaussian elimination with RREF null space computation for relation discovery**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-14T00:23:06Z
- **Completed:** 2026-02-14T00:26:55Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Implemented rational_null_space using exact QRat arithmetic with RREF and partial pivoting
- Implemented modular_null_space over Z/pZ with Fermat's little theorem inversion
- Implemented build_coefficient_matrix to extract FPS coefficients into matrix form for relation finding
- 15 tests covering rational null space, modular null space, coefficient matrix building, and an integration test

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement rational Gaussian elimination and null space** - `a8e9604` (feat)
2. **Task 2: Test linear algebra on known matrices** - `91d0055` (test)

## Files Created/Modified
- `crates/qsym-core/src/qseries/linalg.rs` - Rational and modular null space computation, coefficient matrix building, modular exponentiation
- `crates/qsym-core/src/qseries/mod.rs` - Register linalg module and re-export public API
- `crates/qsym-core/tests/qseries_linalg_tests.rs` - 15 tests for all linear algebra functions

## Decisions Made
- Used Fermat's little theorem (a^{p-2} mod p) for modular inverse rather than extended Euclidean algorithm -- simpler implementation, same complexity for prime moduli
- Used i128 intermediates in mod_mul to prevent i64 overflow during modular multiplication
- Null space basis uses canonical form: free variable entry = 1, pivot entries = negated RREF entries

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Integration test initially had wrong expected null space dimension (expected 1 for three proportional series, actual was 2). Fixed by using non-proportional series with a single linear relation: f1 = 1+q+q^2, f2 = q+2q^2, f3 = f1+f2 = 1+2q+3q^2.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- rational_null_space and build_coefficient_matrix are ready for use by findlincombo, findhom, findpoly (Plans 04-07+)
- modular_null_space ready for modp variants in Plan 04-06
- All 313 existing + new tests pass with no regressions

## Self-Check: PASSED

- All 4 key files verified present on disk
- Commit a8e9604 (Task 1) verified in git log
- Commit 91d0055 (Task 2) verified in git log
- 313 total tests passing (15 new + 298 existing)

---
*Phase: 04-series-analysis*
*Completed: 2026-02-14*
