---
phase: 04-series-analysis
plan: 04
subsystem: qseries
tags: [etamake, jacprodmake, mprodmake, qetamake, eta-quotient, jacobi-product, mobius-inversion, post-processing]

# Dependency graph
requires:
  - phase: 04-series-analysis
    plan: 01
    provides: prodmake function, InfiniteProductForm type, mobius/divisors helpers
provides:
  - etamake: series to eta-quotient form via Mobius inversion
  - jacprodmake: series to JAC(a,b) Jacobi product form with period search
  - mprodmake: series to (1+q^n) product form
  - qetamake: series to (q^d;q^d)_inf notation
  - EtaQuotient, JacobiProductForm, QEtaForm result types
affects: [05-qseries-parity, 07-identity-proving]

# Tech tracking
tech-stack:
  added: []
  patterns: [post-processing-of-prodmake, Mobius-inversion-for-eta-recovery, iterative-extraction-for-mprodmake, period-search-for-jacprodmake]

key-files:
  created: []
  modified:
    - crates/qsym-core/src/qseries/prodmake.rs
    - crates/qsym-core/src/qseries/mod.rs
    - crates/qsym-core/tests/qseries_prodmake_tests.rs

key-decisions:
  - "Mobius inversion for etamake: r_n = sum_{d|n} mu(n/d) * (-a_d) rather than iterative subtraction, reusing existing mobius/divisors helpers"
  - "QRat-to-i64 conversion in eta/qeta factors via to_f64() cast -- valid since prodmake returns integer exponents for valid products"
  - "Period search in jacprodmake tries all b from 1 to max_n, picking best coverage -- simple but correct for typical product sizes"

patterns-established:
  - "Post-processing chain: prodmake -> etamake/jacprodmake/mprodmake/qetamake each call prodmake internally and reinterpret exponents"
  - "Period-search with residue-class grouping for Jacobi product recognition"

# Metrics
duration: 5min
completed: 2026-02-14
---

# Phase 4 Plan 4: etamake/jacprodmake/mprodmake/qetamake Summary

**Four post-processing functions interpreting prodmake output as eta-quotients, Jacobi products, (1+q^n) products, and q-eta forms, with 14 new integration tests verifying round-trip correctness on Euler function, partition GF, JAC(1,5), JAC(2,5), and non-periodic series**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-14T00:40:14Z
- **Completed:** 2026-02-14T00:46:05Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Implemented etamake recovering eta-quotient exponents r_d via Mobius inversion of prodmake output
- Implemented jacprodmake with period search and residue-class grouping to recover JAC(a,b) parameters
- Implemented mprodmake converting (1-q^n) exponents to (1+q^n) exponents via iterative extraction
- Implemented qetamake as thin wrapper over etamake stripping the q^{d/24} prefactors
- Added EtaQuotient, JacobiProductForm, QEtaForm result types
- 14 new integration tests covering all four functions, including exact/non-exact cases
- All 23 prodmake tests passing (9 existing + 14 new)

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement etamake, jacprodmake, mprodmake, qetamake** - `2823efb` (feat)
2. **Task 2: Test all series-to-product post-processing functions** - `a4433cb` (test)

## Files Created/Modified
- `crates/qsym-core/src/qseries/prodmake.rs` - Added EtaQuotient, JacobiProductForm, QEtaForm types and etamake, jacprodmake, mprodmake, qetamake functions (+472 lines)
- `crates/qsym-core/src/qseries/mod.rs` - Updated module doc and re-exports for new types and functions
- `crates/qsym-core/tests/qseries_prodmake_tests.rs` - 14 new integration tests (+415 lines, 747 total)

## Decisions Made
- **Mobius inversion for etamake:** Used r_n = sum_{d|n} mu(n/d) * (-a_d) which is mathematically clean and reuses the existing mobius/divisors helpers rather than the iterative top-down subtraction approach
- **QRat-to-i64 conversion:** eta/qeta factor exponents converted via `to_f64() as i64` since prodmake always returns integer exponents for valid infinite products
- **Period search strategy:** jacprodmake tries all periods b from 1 to max_n, picking the one that explains the most exponents; simple O(n^2) approach works well for typical truncation orders

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- **Pre-existing test failure:** `test_findpoly_no_relation` in `qseries_relations_tests.rs` fails (from plan 04-03). Verified this failure exists on the base commit before any 04-04 changes. Not a regression from this plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- All series-to-product conversion functions complete (prodmake + 4 post-processing)
- Ready for Phase 5 q-series parity and any future plans that need product form analysis
- etamake/qetamake provide foundation for modular form theory applications
- jacprodmake enables automatic Jacobi product identification from series data

## Self-Check: PASSED

- FOUND: crates/qsym-core/src/qseries/prodmake.rs
- FOUND: crates/qsym-core/src/qseries/mod.rs
- FOUND: crates/qsym-core/tests/qseries_prodmake_tests.rs
- FOUND: .planning/phases/04-series-analysis/04-04-SUMMARY.md
- FOUND: commit 2823efb (Task 1)
- FOUND: commit a4433cb (Task 2)

---
*Phase: 04-series-analysis*
*Completed: 2026-02-14*
