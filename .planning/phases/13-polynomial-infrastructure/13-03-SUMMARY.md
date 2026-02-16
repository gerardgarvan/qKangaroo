---
phase: 13-polynomial-infrastructure
plan: 03
subsystem: polynomial
tags: [QRatRationalFunc, rational-function, auto-simplification, cross-cancellation, polynomial-arithmetic, rug]

# Dependency graph
requires:
  - phase: 13-polynomial-infrastructure
    plan: 01
    provides: "QRatPoly dense polynomial type with arithmetic, div_rem, exact_div, scalar_div"
  - phase: 13-polynomial-infrastructure
    plan: 02
    provides: "poly_gcd (subresultant PRS), poly_resultant, q_shift/q_shift_n"
provides:
  - "QRatRationalFunc rational function type with auto-simplification"
  - "Rational function arithmetic (add, sub, mul, div, neg) with cross-cancellation"
  - "Rational function q_shift/q_shift_n, eval, is_zero/is_polynomial"
  - "PartialEq/Eq via canonical normalized form"
  - "Phase 13 integration tests verifying POLY-01 through POLY-05"
affects: [14-q-gosper, 15-q-zeilberger]

# Tech tracking
tech-stack:
  added: []
  patterns: [auto-simplification-on-construction, cross-cancellation-multiplication, monic-denominator-invariant, canonical-form-equality]

key-files:
  created:
    - crates/qsym-core/src/poly/ratfunc.rs
  modified:
    - crates/qsym-core/src/poly/mod.rs
    - crates/qsym-core/src/lib.rs

key-decisions:
  - "Auto-reduce via poly_gcd on every construction (not lazy) -- ensures canonical form at all times"
  - "Cross-cancellation in mul: gcd(a,d) and gcd(c,b) before multiplying, reducing intermediate sizes"
  - "Monic denominator invariant: divide both numer/denom by leading_coeff(denom) in constructor"
  - "Negation bypasses constructor since it preserves coprimality and monicity"

patterns-established:
  - "QRatRationalFunc canonical form: lowest terms + monic denom enables PartialEq via field comparison"
  - "rf_add/rf_sub/rf_mul/rf_div named methods + std::ops trait delegation (4 variants each)"
  - "Integration tests in mod.rs::integration_tests verify cross-module interactions"

# Metrics
duration: 5min
completed: 2026-02-16
---

# Phase 13 Plan 03: QRatRationalFunc Rational Function Type Summary

**Auto-simplifying rational function type with cross-cancellation multiplication, canonical form equality, and q-shift support for q-Gosper/q-Zeilberger phases**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-16T04:26:25Z
- **Completed:** 2026-02-16T04:31:36Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- QRatRationalFunc with 3 invariants (lowest terms, monic denom, nonzero denom) enforced on every construction
- Cross-cancellation optimization in multiplication keeps intermediate polynomial sizes small
- Full arithmetic: add, sub, mul, div, neg with std::ops trait impls (4 variants each + 2 for Neg)
- 23 integration tests verifying all 5 Phase 13 success criteria (POLY-01 through POLY-05)
- 51 new tests total (28 ratfunc unit + 23 integration), total crate tests: 722 (up from 671)
- Phase 13 polynomial infrastructure complete -- ready for Phase 14 q-Gosper

## Task Commits

Each task was committed atomically:

1. **Task 1: QRatRationalFunc struct with auto-simplification** - `0583e48` (feat)
2. **Task 2: Integration tests and final verification** - `a95c1ab` (test)

## Files Created/Modified
- `crates/qsym-core/src/poly/ratfunc.rs` - QRatRationalFunc struct, constructor, arithmetic (cross-cancellation mul), q_shift, eval, Display, PartialEq/Eq, std::ops traits, 28 tests
- `crates/qsym-core/src/poly/mod.rs` - Added `pub mod ratfunc`, re-export QRatRationalFunc, 23 integration tests
- `crates/qsym-core/src/lib.rs` - Re-exported QRatRationalFunc, poly_gcd, poly_resultant at crate root

## Decisions Made
- Auto-reduce via poly_gcd on every construction rather than lazily. This costs a GCD per construction but ensures the canonical form invariant holds at all times, enabling direct equality comparison and predictable expression sizes.
- Cross-cancellation in multiplication: compute gcd(self.numer, other.denom) and gcd(other.numer, self.denom) before multiplying. This is the standard optimization from computer algebra that prevents intermediate coefficient explosion.
- Negation bypasses the full constructor since negating the numerator preserves both coprimality and the monic denominator invariant. This avoids an unnecessary GCD computation.
- Re-exported poly_gcd and poly_resultant at crate root (lib.rs) alongside QRatPoly and QRatRationalFunc for convenient downstream use.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Complete polynomial infrastructure: QRatPoly + poly_gcd + poly_resultant + QRatRationalFunc
- Ready for Phase 14 (q-Gosper): rational functions are the coefficient type in Gosper's key equation
- Ready for Phase 15 (q-Zeilberger): rational function arithmetic used throughout the algorithm
- All 144 poly module tests passing, 722 total crate tests

## Self-Check: PASSED

- All 3 files verified present on disk
- Commits `0583e48` and `a95c1ab` verified in git log
- 722 tests passing (0 failures)

---
*Phase: 13-polynomial-infrastructure*
*Completed: 2026-02-16*
