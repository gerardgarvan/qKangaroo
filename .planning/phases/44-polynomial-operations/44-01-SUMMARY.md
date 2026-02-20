---
phase: 44-polynomial-operations
plan: 01
subsystem: core, cli
tags: [factorization, cyclotomic, polynomial, q-series]

# Dependency graph
requires:
  - phase: 13-poly-arithmetic
    provides: QRatPoly with div_rem, exact_div, content, primitive_part
provides:
  - cyclotomic_poly(n) for computing Phi_n(x)
  - factor_over_q() for polynomial factorization over Q[x]
  - Factorization struct with display_with_var()
  - CLI factor() function with help and tab completion
affects: [44-02-polynomial-operations, garvan-tutorial]

# Tech tracking
tech-stack:
  added: []
  patterns: [cyclotomic-trial-division, fps-to-qratpoly-conversion]

key-files:
  created:
    - crates/qsym-core/src/poly/cyclotomic.rs
    - crates/qsym-core/src/poly/factor.rs
  modified:
    - crates/qsym-core/src/poly/mod.rs
    - crates/qsym-core/src/lib.rs
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/help.rs
    - crates/qsym-cli/src/repl.rs

key-decisions:
  - "Cyclotomic trial division scans from highest n down to 1 for correct factor discovery"
  - "fps_to_qratpoly requires POLYNOMIAL_ORDER sentinel to reject truncated series"
  - "Negative leading coefficient handled by negating both scalar and primitive part"
  - "Factor display uses descending degree order within each parenthesized factor"

patterns-established:
  - "fps_to_qratpoly conversion pattern for polynomial operations on FPS values"
  - "Polynomial Operations category in help system (Group 14)"

requirements-completed: [POLY-01]

# Metrics
duration: 8min
completed: 2026-02-20
---

# Phase 44 Plan 01: Polynomial Factorization Summary

**Cyclotomic polynomial computation and Q[x] factorization via trial division, with CLI factor() dispatch**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-20T22:21:48Z
- **Completed:** 2026-02-20T22:29:31Z
- **Tasks:** 2
- **Files modified:** 7 (2 created, 5 modified)

## Accomplishments
- Cyclotomic polynomial Phi_n(x) computation via recursive division over divisors of n
- factor_over_q() decomposes polynomials into cyclotomic and irreducible factors with content extraction
- factor() CLI function with truncated-series rejection, help under "Polynomial Operations", tab completion
- 29 new tests (9 cyclotomic + 17 factor core + 3 CLI dispatch)
- Total test count: 746 (568 CLI unit + 152 CLI integration + 26 new core)

## Task Commits

Each task was committed atomically:

1. **Task 1: Core cyclotomic and factoring modules** - `95bb645` (feat)
2. **Task 2: CLI factor() dispatch, help, and tab completion** - `ee5b7fb` (feat)

## Files Created/Modified
- `crates/qsym-core/src/poly/cyclotomic.rs` - Cyclotomic polynomial Phi_n(x) via recursive division
- `crates/qsym-core/src/poly/factor.rs` - Factorization struct, factor_over_q(), display formatting
- `crates/qsym-core/src/poly/mod.rs` - Module declarations and re-exports for cyclotomic and factor
- `crates/qsym-core/src/lib.rs` - Crate-level re-exports for Factorization and factor_over_q
- `crates/qsym-cli/src/eval.rs` - factor dispatch, fps_to_qratpoly helper, ALL_FUNCTION_NAMES, get_signature
- `crates/qsym-cli/src/help.rs` - factor help entry, Polynomial Operations category, count updates
- `crates/qsym-cli/src/repl.rs` - factor in tab completion, count update to 96

## Decisions Made
- Cyclotomic trial division scans n from degree down to 1 to find primitive factors first
- fps_to_qratpoly validates POLYNOMIAL_ORDER sentinel (rejects truncated series with clear error)
- Negative leading coefficient on primitive part: negate both scalar and polynomial
- Factor display uses descending degree order within parenthesized factors (e.g., "q^2-q+1")
- Sorting: factors sorted by degree ascending, then by coefficient comparison for tiebreaking

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- factor() is fully functional for q-polynomials produced by expand(), aqprod(), etc.
- Ready for Phase 44 Plan 02 (additional polynomial operations)
- fps_to_qratpoly conversion pattern established for reuse by future polynomial operations

## Self-Check: PASSED

- cyclotomic.rs: FOUND
- factor.rs: FOUND
- SUMMARY.md: FOUND
- Commit 95bb645: FOUND
- Commit ee5b7fb: FOUND

---
*Phase: 44-polynomial-operations*
*Completed: 2026-02-20*
