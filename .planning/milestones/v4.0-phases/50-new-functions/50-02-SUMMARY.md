---
phase: 50-new-functions
plan: 02
subsystem: cli
tags: [subscript-variables, subs, theta-monomial, radsimp, parser]

# Dependency graph
requires:
  - phase: 50-new-functions
    plan: 01
    provides: "jac2series Garvan form, quinprod identity modes"
provides:
  - "Parser subscript syntax X[i] for indexed variables"
  - "Multi-arg subs(var=val, ..., expr) substitution"
  - "Theta3/theta4 monomial argument support theta3(q^k, T)"
  - "radsimp(expr) identity simplification function"
affects: [cli-functions, findnonhom-workflow]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Subscript parsing as Variable name mangling: X[1] -> Variable('X[1]')"
    - "Monomial argument detection via Value::Series single-term pattern match"
    - "Exponent scaling for theta monomial: compute base series then multiply exponents by k"

key-files:
  created: []
  modified:
    - "crates/qsym-cli/src/parser.rs"
    - "crates/qsym-cli/src/eval.rs"
    - "crates/qsym-cli/tests/cli_integration.rs"

key-decisions:
  - "Subscript X[i] implemented as name mangling (Variable('X[i]')) rather than a separate AST node"
  - "Theta monomial uses exponent scaling (not filtering) for correct q^k substitution semantics"
  - "radsimp is identity function since series division already simplifies during evaluation"
  - "theta2 monomial rejected with helpful error (half-integer exponent complexity)"

patterns-established:
  - "Subscript binding power matches function call (19) for consistent precedence"
  - "Multi-arg subs loops over all but last arg as substitution pairs"

requirements-completed: [FUNC-02, FUNC-04]

# Metrics
duration: 7min
completed: 2026-02-21
---

# Phase 50 Plan 02: Subscript Variables, Multi-arg Subs, Theta Monomial, and radsimp Summary

**Parser subscript syntax X[i], multi-arg subs for indexed variable substitution, theta3/theta4 monomial argument support for q^k evaluation, and radsimp identity simplification**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-21T16:07:19Z
- **Completed:** 2026-02-21T16:14:29Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- X[1] parses as Variable("X[1]") and works seamlessly with assignment and lookup
- subs accepts multiple var=val pairs: subs(X[1]=q, X[2]=q^2, expr) with backward compat
- theta3(q^k, T) and theta4(q^k, T) compute correctly by scaling exponents
- radsimp(expr) returns evaluated argument unchanged (simplification already happened during evaluation)
- theta2 monomial arg rejected with helpful "use subs instead" message

## Task Commits

Each task was committed atomically:

1. **Task 1: Parser subscript syntax and multi-arg subs** - `d502136` (feat)
2. **Task 2: Theta monomial support, radsimp, and integration tests** - `9ec59c2` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/parser.rs` - Added subscript parsing in LED loop (X[i] -> Variable("X[i]")), 3 parser unit tests
- `crates/qsym-cli/src/eval.rs` - Multi-arg subs loop, theta3/theta4 monomial support, theta2 monomial rejection, radsimp dispatch, get_signature updates, ALL_FUNCTION_NAMES update, 5 unit tests
- `crates/qsym-cli/tests/cli_integration.rs` - 5 integration tests (subscript assignment, subs, theta3_monomial, radsimp_series, radsimp_quotient)

## Decisions Made
- Subscript X[i] uses name mangling (`Variable("X[i]")`) rather than introducing a new AST node -- this avoids changes to the entire eval pipeline while providing full functionality
- Theta monomial algorithm computes base theta3(q, T) then scales all exponents by k, producing correct results (NOT filtering multiples from theta3(q, T*k))
- radsimp is a pure identity function since the REPL already evaluates/simplifies during argument evaluation (division already computed)
- theta2 monomial support deliberately omitted due to half-integer exponent complexity; users directed to use `subs(q=q^k, theta2(q,T))` instead

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed theta monomial algorithm from filtering to scaling**
- **Found during:** Task 2 (theta3 monomial implementation)
- **Issue:** Plan's initial algorithm filtered theta3(q, T*exp) for multiples of exp then divided exponents, which is mathematically incorrect. theta3(q^k) requires exponent SCALING, not filtering.
- **Fix:** Changed algorithm to compute theta3(q, T) then multiply all exponents by k. Truncation order set to T*k.
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** Unit test dispatch_theta3_monomial verifies correct coefficients at q^0, q^2, q^8, q^18
- **Committed in:** 9ec59c2 (Task 2 commit)

**2. [Rule 1 - Bug] Removed unnecessary println from radsimp dispatch**
- **Found during:** Task 2 (radsimp implementation)
- **Issue:** Plan included `println!` in radsimp dispatch, but the REPL already handles display of returned values. Adding println would cause double output.
- **Fix:** Removed println, returning `Ok(args[0].clone())` directly (matching pattern of other identity-like functions)
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Committed in:** 9ec59c2 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs in plan specification)
**Impact on plan:** Both fixes essential for correctness. No scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- All FUNC-02 and FUNC-04 requirements complete
- Phase 50 fully done: jac2series Garvan, quinprod identity modes, subscript vars, multi-arg subs, theta monomial, radsimp
- Ready for Phase 51 (if any remaining phases)

## Self-Check: PASSED

- All 3 modified files exist on disk
- Commit d502136 (Task 1) verified in git log
- Commit 9ec59c2 (Task 2) verified in git log
- SUMMARY.md created at .planning/phases/50-new-functions/50-02-SUMMARY.md

---
*Phase: 50-new-functions*
*Completed: 2026-02-21*
