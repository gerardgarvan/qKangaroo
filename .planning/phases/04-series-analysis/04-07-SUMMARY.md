---
phase: 04-series-analysis
plan: 07
subsystem: series-analysis
tags: [modular-arithmetic, relation-discovery, linear-independence, product-form-search, q-series, finite-fields]

# Dependency graph
requires:
  - phase: 04-series-analysis
    provides: "rational_null_space, modular_null_space, build_coefficient_matrix from linalg.rs (plan 04-03)"
  - phase: 04-series-analysis
    provides: "findlincombo, findhom, findhomcombo, generate_monomials, compute_monomial_series, fps_pow from plan 04-05"
  - phase: 04-series-analysis
    provides: "findcong, findnonhom, findnonhomcombo from plan 04-06"
  - phase: 04-series-analysis
    provides: "prodmake from prodmake.rs (plan 04-01)"
provides:
  - "findlincombomodp: discover linear combinations mod a prime p"
  - "findhommodp: find homogeneous polynomial relations over Z/pZ"
  - "findhomcombomodp: express target as homogeneous combo mod p"
  - "findmaxind: identify maximal linearly independent subset of series"
  - "findprod: brute-force search for linear combinations with nice product forms"
  - "Complete QSER-19 relation discovery suite (12+ functions)"
affects: [05-python-api]

# Tech tracking
tech-stack:
  added: []
  patterns: [modular-coefficient-matrix, qrat-to-modp-conversion, pivot-column-independence, odometer-brute-force-search]

key-files:
  created: []
  modified:
    - crates/qsym-core/src/qseries/relations.rs
    - crates/qsym-core/src/qseries/mod.rs
    - crates/qsym-core/tests/qseries_relations_tests.rs

key-decisions:
  - "Local mod_inv_local/mod_pow_local helpers in relations.rs rather than importing from linalg to avoid pub exposure of internal helpers"
  - "QRat-to-modp conversion via rug Integer::is_divisible check + Fermat inverse, returns None if denominator divisible by p"
  - "findmaxind uses inline Gaussian elimination (same RREF algorithm) rather than calling rational_null_space, to directly extract pivot columns"
  - "findprod uses brute-force odometer iteration over [-max_coeff, max_coeff]^k coefficient space with prodmake integer-exponent check"

patterns-established:
  - "Modular relation discovery pattern: convert QRat coefficients to Z/pZ, build i64 matrix, use modular_null_space"
  - "Independence detection via pivot column extraction from RREF Gaussian elimination"
  - "Product form search via bounded brute-force with prodmake integer-exponent verification"

# Metrics
duration: 7min
completed: 2026-02-14
---

# Phase 4 Plan 7: Modular Relations and Product Search Summary

**findlincombomodp/findhommodp/findhomcombomodp for finite field relation discovery, findmaxind for basis selection, and findprod for product form search -- completing all 12+ QSER-19 relation functions**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-14T00:55:40Z
- **Completed:** 2026-02-14T01:02:33Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Implemented 5 new relation discovery functions completing the full QSER-19 suite
- findlincombomodp, findhommodp, findhomcombomodp enable relation discovery over finite fields Z/pZ using modular_null_space
- findmaxind identifies linearly independent subsets via Gaussian elimination pivot columns
- findprod performs bounded brute-force search for linear combinations with integer prodmake exponents
- Added QRat-to-modp conversion infrastructure (mod_inv_local, mod_pow_local, qrat_to_mod_p, build_modp_coefficient_matrix)
- 7 new tests (23 total relation tests, 379 total passing) including full suite smoke test

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement findlincombomodp, findhommodp, findhomcombomodp, findmaxind, findprod** - `36a98ff` (feat)
2. **Task 2: Test modp variants, findmaxind, and findprod** - `f343f69` (test)

## Files Created/Modified
- `crates/qsym-core/src/qseries/relations.rs` - Added 5 new public functions, modular arithmetic helpers, QRat-to-modp conversion, coefficient matrix builder (+563 lines)
- `crates/qsym-core/src/qseries/mod.rs` - Re-export all 5 new functions, updated module documentation
- `crates/qsym-core/tests/qseries_relations_tests.rs` - 7 new tests for all 5 functions plus full suite smoke test (+215 lines)

## Decisions Made
- Used local mod_inv_local/mod_pow_local helpers in relations.rs rather than importing from linalg -- avoids exposing internal linalg helpers as public and keeps the modular arithmetic self-contained within the module that uses it
- QRat-to-modp conversion handles denominators divisible by p gracefully (returns None, propagated up to skip the series)
- findmaxind performs inline RREF rather than calling rational_null_space to directly extract pivot column indices -- more efficient than computing the full null space just to count dimensions
- findprod uses simple odometer-style brute force over [-max_coeff, max_coeff]^k -- adequate for the small search spaces typical in q-series research (k=1-3 series, max_coeff=1-3)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 4 complete: all QSER-09 through QSER-19 requirements implemented
- Full relation discovery suite: findlincombo, findhom, findpoly, findcong, findnonhom, findhomcombo, findnonhomcombo, findlincombomodp, findhommodp, findhomcombomodp, findmaxind, findprod
- All 379 tests passing with no regressions (7 new + 372 existing)
- Ready for Phase 5 (Python API) -- all Rust functions stabilized

## Self-Check: PASSED

- All 3 key files verified present on disk
- Commit 36a98ff (Task 1) verified in git log
- Commit f343f69 (Task 2) verified in git log
- 379 total tests passing (7 new + 372 existing)

---
*Phase: 04-series-analysis*
*Completed: 2026-02-14*
