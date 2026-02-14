---
phase: 04-series-analysis
plan: 05
subsystem: series-analysis
tags: [relation-discovery, linear-algebra, null-space, polynomial-relations, q-series]

# Dependency graph
requires:
  - phase: 04-series-analysis
    provides: "rational_null_space, build_coefficient_matrix from linalg.rs (plan 04-03)"
  - phase: 02-simplification-series-engine
    provides: "FormalPowerSeries arithmetic (mul, add, scalar_mul, invert, negate)"
provides:
  - "findlincombo: discover f as linear combination of basis series via null space"
  - "findhom: find homogeneous degree-d polynomial relations among series"
  - "findpoly: find two-variable polynomial relation P(x,y) = 0"
  - "PolynomialRelation: structured type for polynomial relation coefficients"
affects: [04-06, 04-07]

# Tech tracking
tech-stack:
  added: []
  patterns: [coefficient-matrix-null-space-relation-discovery, monomial-enumeration, repeated-squaring-fps-power]

key-files:
  created:
    - crates/qsym-core/src/qseries/relations.rs
    - crates/qsym-core/tests/qseries_relations_tests.rs
  modified:
    - crates/qsym-core/src/qseries/mod.rs

key-decisions:
  - "Monomial ordering in findhom uses generate_monomials lexicographic order (last index varies fastest)"
  - "fps_pow uses repeated squaring for efficient power computation"
  - "findlincombo normalizes null space vector so first component (for f) equals 1, then negates remaining"

patterns-established:
  - "Relation discovery pattern: build candidate series list, extract coefficient matrix, compute null space, interpret vectors as relations"
  - "Monomial enumeration via recursive partitioning for homogeneous polynomial generation"

# Metrics
duration: 5min
completed: 2026-02-14
---

# Phase 4 Plan 5: Relation Discovery Summary

**findlincombo, findhom, findpoly relation discovery functions using coefficient-matrix null space with Jacobi theta identity verification**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-14T00:40:50Z
- **Completed:** 2026-02-14T00:46:29Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Implemented findlincombo to discover linear combinations of q-series via null space computation
- Implemented findhom to find all homogeneous degree-d polynomial relations among a set of series
- Implemented findpoly to discover two-variable polynomial relations P(x,y) = 0 with PolynomialRelation return type
- Added fps_pow helper with repeated squaring and generate_monomials for monomial enumeration
- 10 comprehensive tests including verification of the Jacobi theta identity theta3^4 = theta2^4 + theta4^4

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement findlincombo, findhom, findpoly with PolynomialRelation type** - `04f99a9` (feat)
2. **Task 2: Test relation discovery with known mathematical relations** - `d208aa7` (test)

## Files Created/Modified
- `crates/qsym-core/src/qseries/relations.rs` - findlincombo, findhom, findpoly, PolynomialRelation type, fps_pow, generate_monomials, compute_monomial_series
- `crates/qsym-core/src/qseries/mod.rs` - Register relations module and re-export public API
- `crates/qsym-core/tests/qseries_relations_tests.rs` - 10 tests covering all relation discovery functions

## Decisions Made
- Monomial ordering in findhom: uses generate_monomials with lexicographic order where the last index varies fastest (consistent with standard math convention)
- fps_pow uses repeated squaring for O(log n) multiplications instead of naive repeated multiplication
- findlincombo normalizes the null space vector so the first component (corresponding to f) equals 1, then negates the remaining components to extract coefficients

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed findhom trivial degree-1 test assertion**
- **Found during:** Task 2
- **Issue:** Original test assumed monomial ordering was [f, g] but generate_monomials produces [(0,1), (1,0)] putting g first
- **Fix:** Changed assertion to verify the relation holds algebraically (v[0]*g + v[1]*f = 0) instead of checking a hardcoded ratio
- **Files modified:** crates/qsym-core/tests/qseries_relations_tests.rs
- **Committed in:** d208aa7 (Task 2 commit)

**2. [Rule 1 - Bug] Fixed findlincombo/findpoly no-relation tests with insufficient overdetermination**
- **Found during:** Task 2
- **Issue:** With topshift=0, the coefficient matrix was square (or nearly so), allowing spurious null space vectors. Tests that expected None were finding false relations.
- **Fix:** Used higher topshift values (10-20) and more diverse coefficient patterns to ensure overdetermined systems correctly reject non-relations
- **Files modified:** crates/qsym-core/tests/qseries_relations_tests.rs
- **Committed in:** d208aa7 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 bug fixes in tests)
**Impact on plan:** Both fixes were test-level corrections. No changes to implementation code. No scope creep.

## Issues Encountered
- Jacobi theta identity test required computing theta2^4 directly as a q-series (via product formula) since theta2 returns a series in X = q^{1/4} which is incompatible with theta3/theta4 in q. Built a helper function theta2_fourth_power using the product identity theta2(q)^4 = 16q * prod(1-q^{2n})^4 * (1+q^{2n})^8.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- findlincombo, findhom, findpoly ready for use by higher-level relation discovery (findcong, findnonhom, etc. in plans 04-06, 04-07)
- PolynomialRelation type available for structured polynomial relation output
- All 366 tests passing with no regressions (10 new + 356 existing)

---
*Phase: 04-series-analysis*
*Completed: 2026-02-14*
