---
phase: 13-polynomial-infrastructure
plan: 01
subsystem: polynomial
tags: [QRatPoly, dense-polynomial, polynomial-arithmetic, euclidean-division, horner-eval, rug]

# Dependency graph
requires:
  - phase: 01-expression-foundation
    provides: "QRat arbitrary-precision rational type with arithmetic ops"
provides:
  - "QRatPoly dense univariate polynomial type"
  - "Full polynomial arithmetic (Add, Sub, Mul, Neg by val and ref)"
  - "Euclidean division (div_rem, exact_div, pseudo_rem)"
  - "Content/primitive_part/make_monic"
  - "Horner evaluation, Display, PartialEq/Eq"
affects: [13-02 GCD, 13-03 resultant, 13-04 q-shift, 14-polynomial-algorithms]

# Tech tracking
tech-stack:
  added: []
  patterns: [dense-ascending-vec, trailing-zero-invariant, four-variant-trait-impls]

key-files:
  created:
    - crates/qsym-core/src/poly/mod.rs
    - crates/qsym-core/src/poly/arithmetic.rs
  modified:
    - crates/qsym-core/src/lib.rs

key-decisions:
  - "Dense Vec<QRat> ascending-degree storage with trailing-zero normalization invariant"
  - "Content = gcd(numerators)/lcm(denominators) for rational coefficients"
  - "Four trait impl variants per operation (val/val, ref/ref, val/ref, ref/val) matching QRat pattern"

patterns-established:
  - "QRatPoly canonical form: empty vec for zero, last element always nonzero"
  - "from_vec constructor always normalizes; all operations produce normalized output"
  - "Schoolbook multiplication with zero-coefficient skip optimization"

# Metrics
duration: 4min
completed: 2026-02-16
---

# Phase 13 Plan 01: QRatPoly Dense Polynomial Type Summary

**Dense univariate QRatPoly with full arithmetic, Euclidean division, content/monic, and Horner eval over arbitrary-precision rationals**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-16T04:14:02Z
- **Completed:** 2026-02-16T04:18:29Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- QRatPoly struct with 8 constructors, 7 query methods, and canonical trailing-zero invariant
- Content/primitive_part/make_monic for both integer and rational coefficient polynomials
- Complete arithmetic: Add, Sub, Mul, Neg (4 variants each), scalar_mul, scalar_div
- Euclidean div_rem, exact_div (with remainder assertion), pseudo_rem (fraction-free)
- Horner evaluation returning exact QRat, human-readable Display formatting
- 69 new tests (31 struct/query + 38 arithmetic including ring axiom verification)
- Total crate tests: 647 (up from 578)

## Task Commits

Each task was committed atomically:

1. **Task 1: QRatPoly struct, constructors, queries, and Display** - `7196790` (feat)
2. **Task 2: Polynomial arithmetic operations** - `03ad617` (feat)

## Files Created/Modified
- `crates/qsym-core/src/poly/mod.rs` - QRatPoly struct, constructors, queries, content/primitive/monic, eval, Display, PartialEq, 31 tests
- `crates/qsym-core/src/poly/arithmetic.rs` - Add/Sub/Mul/Neg traits, scalar ops, div_rem/exact_div/pseudo_rem, 38 tests
- `crates/qsym-core/src/lib.rs` - Added `pub mod poly` and `pub use poly::QRatPoly`

## Decisions Made
- Dense Vec<QRat> ascending-degree storage with trailing-zero normalization invariant (canonical form enables direct Vec comparison for equality)
- Content computed as gcd(numerators)/lcm(denominators) to handle rational coefficients correctly
- Four trait impl variants per arithmetic operation (val/val, ref/ref, val/ref, ref/val) following the pattern established by QRat in number.rs
- scalar_mul/scalar_div as methods on QRatPoly (not traits) since they take QRat scalars, not QRatPoly

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- QRatPoly foundation complete, ready for Plan 02 (polynomial GCD via subresultant PRS)
- All constructors, arithmetic, and division operations available for GCD algorithm implementation
- pseudo_rem specifically designed for subresultant PRS coefficient control

## Self-Check: PASSED

- All 4 files verified present on disk
- Commits `7196790` and `03ad617` verified in git log
- 647 tests passing (0 failures)

---
*Phase: 13-polynomial-infrastructure*
*Completed: 2026-02-16*
