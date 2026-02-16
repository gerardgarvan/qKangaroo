---
phase: 13-polynomial-infrastructure
plan: 02
subsystem: polynomial
tags: [GCD, subresultant-PRS, resultant, q-shift, polynomial-arithmetic, rug]

# Dependency graph
requires:
  - phase: 13-polynomial-infrastructure
    plan: 01
    provides: "QRatPoly dense polynomial type with arithmetic, div_rem, pseudo_rem, content/primitive_part"
provides:
  - "poly_gcd via subresultant PRS (monic, no coefficient explosion)"
  - "poly_resultant via Euclidean algorithm over Q[x]"
  - "q_shift / q_shift_n for p(x) -> p(q^j * x) transformation"
affects: [13-03 rational functions, 14-q-Gosper q-dispersion, 15-q-Zeilberger]

# Tech tracking
tech-stack:
  added: []
  patterns: [subresultant-PRS-content-extraction, euclidean-resultant-over-field, coefficient-scaling-q-shift]

key-files:
  created:
    - crates/qsym-core/src/poly/gcd.rs
  modified:
    - crates/qsym-core/src/poly/mod.rs

key-decisions:
  - "Subresultant PRS for GCD with content extraction before PRS loop to reduce coefficient sizes"
  - "Euclidean algorithm (not subresultant) for resultant since Q[x] is a field (no growth issues)"
  - "q_shift/q_shift_n as methods on QRatPoly (not free functions) for cleaner API"
  - "qrat_pow_signed helper in mod.rs for signed-exponent rational power"

patterns-established:
  - "poly_gcd returns monic GCD (leading coefficient 1, unique over Q[x])"
  - "Resultant via recursive Euclidean formula: res(f,g) = (-1)^(mn) * lc(g)^(m-k) * res(g, f mod g)"
  - "q_shift(q) scales coefficient c_i by q^i in O(n) time"

# Metrics
duration: 4min
completed: 2026-02-16
---

# Phase 13 Plan 02: Polynomial GCD, Resultant, and q-Shift Summary

**Subresultant PRS GCD with content extraction, Euclidean resultant over Q[x], and O(n) q-shift for polynomial argument scaling**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-16T04:20:43Z
- **Completed:** 2026-02-16T04:24:24Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- poly_gcd via subresultant PRS with content extraction -- handles degree-10 polynomials without coefficient explosion
- poly_resultant via recursive Euclidean algorithm -- correctly detects shared roots (zero iff common factor)
- q_shift and q_shift_n methods on QRatPoly for p(x) -> p(q^j * x) transformation
- 24 new tests (15 GCD/resultant + 9 q-shift), total crate tests: 671 (up from 647)

## Task Commits

Each task was committed atomically:

1. **Task 1: Subresultant PRS GCD and resultant** - `5ea24b4` (feat)
2. **Task 2: q-shift operations** - `f5df68f` (feat)

## Files Created/Modified
- `crates/qsym-core/src/poly/gcd.rs` - poly_gcd (subresultant PRS), poly_resultant (Euclidean), qrat_pow helper, 15 tests
- `crates/qsym-core/src/poly/mod.rs` - q_shift/q_shift_n methods, qrat_pow_signed/qrat_pow_u32 helpers, pub mod gcd + re-exports, 9 tests

## Decisions Made
- Subresultant PRS for GCD with content extraction: primitive parts computed before entering the PRS loop, reducing coefficient sizes throughout the chain. The monic normalization at the end ensures unique GCD over Q[x].
- Euclidean algorithm for resultant (not subresultant tracking): since Q is a field, Euclidean remainder produces no coefficient growth, making the simpler recursive formula correct and efficient.
- q_shift/q_shift_n as methods on QRatPoly rather than free functions in gcd.rs, since they are conceptually "evaluation-like" operations that belong on the type.
- Separate qrat_pow_signed (signed exponent) in mod.rs vs qrat_pow (unsigned) in gcd.rs -- avoids coupling gcd.rs internals with the public q_shift API.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- GCD and resultant ready for rational function auto-simplification (Plan 03)
- q_shift ready for q-dispersion computation in Phase 14 (q-Gosper)
- Subresultant PRS empirically verified on degree-10 polynomials (no explosion)
- Research blocker "subresultant PRS coefficient growth for degree 5-30" can be considered resolved

## Self-Check: PASSED

- All 3 files verified present on disk
- Commits `5ea24b4` and `f5df68f` verified in git log
- 671 tests passing (0 failures)

---
*Phase: 13-polynomial-infrastructure*
*Completed: 2026-02-16*
