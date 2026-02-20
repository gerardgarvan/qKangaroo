---
phase: 37-new-functions-theta-jacobi
plan: 01
subsystem: cli
tags: [jacobi-product, theta-series, q-series, value-type, dispatch]

# Dependency graph
requires:
  - phase: 34-product-theta-maple-dispatch
    provides: etaq multi-delta dispatch pattern, Maple-style product functions
provides:
  - Value::JacobiProduct first-class type with full arithmetic
  - JAC(a,b) constructor function
  - theta(z, q, T) general theta series function
  - jac2prod(JP, q, T) product notation converter
  - jac2series(JP, q, T) series expansion converter
  - normalize_jacobi_product canonical form helper
  - jacobi_product_to_fps expansion helper
  - qrat_pow rational exponentiation helper
affects: [37-02-qs2jaccombo, help-text, integration-tests]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "JacobiProduct value type with Vec<(i64,i64,i64)> representation"
    - "normalize_jacobi_product for canonical factor ordering"
    - "print-and-return dispatch pattern for jac2prod/jac2series"
    - "Multi-case match on first arg type (theta z handling)"

key-files:
  created: []
  modified:
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/format.rs

key-decisions:
  - "JacobiProduct uses Vec<(a,b,exponent)> sorted by (b,a) as canonical form"
  - "JAC(a,b) validates b>0 but allows a=0 (degenerate case handled by etaq)"
  - "theta dispatches on z type: numeric, monomial, symbol (with warning)"
  - "jac2prod/jac2series use print-and-return pattern matching Phase 36 find* functions"
  - "Add/sub with JacobiProduct gives helpful error directing to jac2series()"
  - "env.symbols.name(sym) used for product notation formatting (not resolve)"

patterns-established:
  - "Value::JacobiProduct arithmetic in eval_mul/eval_div/eval_pow with normalize"
  - "jacobi_product_to_fps via etaq for each factor with invert for negative exponents"
  - "format_product_notation for explicit finite product display"

requirements-completed: [NEW-01, NEW-02, NEW-03]

# Metrics
duration: 8min
completed: 2026-02-19
---

# Phase 37 Plan 01: JacobiProduct Type + theta/jac2prod/jac2series Summary

**First-class JacobiProduct value type with JAC(a,b) constructor, full arithmetic (mul/div/pow), canonical normalization, display formatting (plain + LaTeX), and three new dispatch functions: theta(z,q,T), jac2prod(JP,q,T), jac2series(JP,q,T)**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-19T23:49:43Z
- **Completed:** 2026-02-19T23:57:19Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Value::JacobiProduct is a fully functional first-class value type: created via JAC(a,b), combined with *, /, ^, displayed in JAC notation and LaTeX (q^a;q^b)_inf notation
- theta(z, q, T) handles three z types: numeric direct substitution, monomial auto-substitution, and bare symbol with warning
- jac2prod prints explicit product notation like (1-q)(1-q^6)(1-q^11)... and returns FPS; jac2series prints standard series and returns FPS
- Cross-validated: jac2series(JAC(1,5), q, 20) produces identical coefficients to etaq(1, 5, q, 20)
- 26 new unit tests (18 eval + 6 format + 2 normalize) covering all new functionality

## Task Commits

Each task was committed atomically:

1. **Task 1: JacobiProduct value type, JAC constructor, arithmetic, and formatting** - `29191a1` (feat)
2. **Task 2: theta, jac2prod, jac2series dispatch functions** - `e7de799` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/eval.rs` - Value::JacobiProduct variant, normalize_jacobi_product, qrat_pow, jacobi_product_to_fps, format_product_notation helpers; JAC/theta/jac2prod/jac2series dispatch; arithmetic extensions in eval_mul/eval_div/eval_pow/eval_add/eval_sub; 20 unit tests
- `crates/qsym-cli/src/format.rs` - format_jacobi_product and format_jacobi_product_latex display functions; format_value and format_latex match arms; 6 unit tests

## Decisions Made
- JacobiProduct uses `Vec<(i64, i64, i64)>` where each triple is (a, b, exponent), maintained in canonical form sorted by (b, a) with merged exponents and zero-exponent removal
- JAC(a, b) validates b > 0 but allows a = 0 at the symbolic level (etaq will correctly return zero series for degenerate case)
- theta(z, q, T) uses T as both sum range (-T..T) and FPS truncation order, consistent with other theta functions
- Add/subtract with JacobiProduct returns helpful error: "cannot add/subtract ... use jac2series() to expand first"
- Used env.symbols.name(sym) rather than resolve() for product notation formatting
- jac2series uses crate::format::format_value for consistent display output

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed QMonomial field name reference**
- **Found during:** Task 2 (theta monomial handling)
- **Issue:** Plan referenced `mono.coeff_num` and `mono.coeff_den` but actual QMonomial struct uses `mono.coeff: QRat` and `mono.power: i64`
- **Fix:** Used `mono.coeff` directly as QRat instead of separate num/den fields
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** dispatch_theta_monomial_z test passes with correct coefficients

**2. [Rule 1 - Bug] Fixed symbol registry method name**
- **Found during:** Task 2 (jac2prod dispatch)
- **Issue:** Plan referenced `env.symbols.resolve(sym).unwrap_or("q")` but actual method is `env.symbols.name(sym)` returning `&str`
- **Fix:** Used `env.symbols.name(sym)` directly
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** dispatch_jac2prod_returns_series test passes

---

**Total deviations:** 2 auto-fixed (2 bugs: incorrect API references in plan)
**Impact on plan:** Minor API name corrections. No scope change.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- JacobiProduct type and infrastructure fully operational for Plan 02 (qs2jaccombo)
- jacobi_product_to_fps helper ready for reuse
- Help text and integration tests deferred to Plan 02 or a later plan

---
*Phase: 37-new-functions-theta-jacobi*
*Completed: 2026-02-19*
