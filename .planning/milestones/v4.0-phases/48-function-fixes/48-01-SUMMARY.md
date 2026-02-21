---
phase: 48-function-fixes
plan: 01
subsystem: cli
tags: [aqprod, theta, q-pochhammer, polynomial, garvan, dispatch]

# Dependency graph
requires:
  - phase: 33-maple-compat
    provides: "aqprod Maple-style dispatch, theta function dispatch, POLYNOMIAL_ORDER sentinel"
provides:
  - "aqprod 3-arg produces exact polynomial (POLYNOMIAL_ORDER sentinel)"
  - "theta2/3/4 accept 1-arg, 2-arg (q,T), and 3-arg (a,q,T) Garvan forms"
  - "Updated help entries and signatures for aqprod, theta2/3/4"
affects: [48-function-fixes, qmaple-parity]

# Tech tracking
tech-stack:
  added: []
  patterns: ["multi-arity dispatch with WrongArgCount fallback for theta functions"]

key-files:
  created: []
  modified:
    - "crates/qsym-cli/src/eval.rs"
    - "crates/qsym-cli/src/help.rs"

key-decisions:
  - "aqprod 3-arg uses POLYNOMIAL_ORDER directly (not tight+rewrap) since sparse multiply is efficient"
  - "theta 3-arg form theta3(a,q,T) extracts variable from args[1], ignoring args[0] when a==q"

patterns-established:
  - "Multi-arity theta dispatch: 1-arg legacy, 2-arg Garvan (q,T), 3-arg Garvan (a,q,T)"

requirements-completed: [FIX-01, FIX-02]

# Metrics
duration: 4min
completed: 2026-02-21
---

# Phase 48 Plan 01: Function Fixes Summary

**Fixed aqprod 3-arg to return exact polynomials via POLYNOMIAL_ORDER sentinel; added 2-arg and 3-arg Garvan theta dispatch**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-21T05:56:46Z
- **Completed:** 2026-02-21T06:01:03Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- aqprod(q,q,5) now returns the full exact polynomial (q;q)_5 without O(...) truncation
- theta2/3/4 accept Garvan's 2-arg form theta3(q,T) and 3-arg form theta3(a,q,T)
- 7 new unit tests covering all new dispatch paths (674 total CLI lib tests pass)
- Help entries updated with new signatures for all four functions

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix aqprod 3-arg truncation and add theta 1/2/3-arg dispatch** - `f2912f5` (feat)
2. **Task 2: Update help entries for aqprod and theta signatures** - `91a1527` (docs)

## Files Created/Modified
- `crates/qsym-cli/src/eval.rs` - Fixed aqprod 3-arg to use POLYNOMIAL_ORDER; added multi-arity theta2/3/4 dispatch (1/2/3-arg); updated get_signature(); added 7 tests
- `crates/qsym-cli/src/help.rs` - Updated signatures and descriptions for aqprod, theta2, theta3, theta4

## Decisions Made
- Used POLYNOMIAL_ORDER directly as truncation_order for aqprod 3-arg (simpler than tight+rewrap since sparse multiply only iterates nonzero coefficients)
- theta 3-arg form theta3(a,q,T) extracts variable from args[1] and ignores args[0] -- when a==q this gives standard theta3; for a!=q the existing theta(z,q,T) function handles the generalized case

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected expected coefficients in aqprod polynomial test**
- **Found during:** Task 1 (dispatch_aqprod_maple_3arg_polynomial_order test)
- **Issue:** Plan specified coeff(12)=-1 and coeff(7)=1 for (q;q)_5, but the actual expansion (1-q)(1-q^2)(1-q^3)(1-q^4)(1-q^5) = 1 - q - q^2 + q^5 + q^6 + q^7 - q^8 - q^9 - q^10 + q^13 + q^14 - q^15 has coeff(12)=0 and additional nonzero terms at 6,8,9,10,13,14
- **Fix:** Updated test assertions to match correct polynomial expansion
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** Test passes with correct coefficients
- **Committed in:** f2912f5 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug in plan's test expectations)
**Impact on plan:** Test correction only. All functional changes match plan exactly.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- aqprod and theta fixes complete, ready for Plan 48-02 (qfactor and min)
- 1 pre-existing integration test failure (err_05_read_nonexistent_shows_file_not_found) unrelated to this plan

---
*Phase: 48-function-fixes*
*Completed: 2026-02-21*
