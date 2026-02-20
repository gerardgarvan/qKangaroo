---
phase: 44-polynomial-operations
plan: 02
subsystem: cli
tags: [substitution, subs, polynomial, q-series, evaluation]

# Dependency graph
requires:
  - phase: 44-polynomial-operations
    provides: factor() CLI function, fps_to_qratpoly pattern, Polynomial Operations help category
provides:
  - subs(var=val, expr) for evaluating series at rational points
  - subs exponent scaling (q->q^k) for series transformation
  - AST-level interception pattern for Compare node before evaluation
affects: [garvan-tutorial, 45-tutorial-flow]

# Tech tracking
tech-stack:
  added: []
  patterns: [ast-interception-before-eval, fps-rational-evaluation, exponent-scaling]

key-files:
  created: []
  modified:
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/help.rs
    - crates/qsym-cli/src/repl.rs

key-decisions:
  - "AST interception catches Compare(Eq) before evaluation so q=1 is not converted to Bool"
  - "evaluate_fps_at_rational handles negative exponents via inversion with zero-check"
  - "Exponent scaling preserves POLYNOMIAL_ORDER sentinel for exact polynomials"
  - "Mismatched variable name in subs returns target unchanged (no-op, not error)"

patterns-established:
  - "AST-level interception for functions that need unevaluated arguments (subs, RETURN)"
  - "evaluate_fps_at_rational pattern for numeric evaluation of FPS"

requirements-completed: [POLY-02]

# Metrics
duration: 5min
completed: 2026-02-20
---

# Phase 44 Plan 02: subs() Substitution Summary

**subs(var=val, expr) with AST interception for q-series evaluation at rational points and exponent scaling via q->q^k**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-20T22:32:02Z
- **Completed:** 2026-02-20T22:36:47Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- AST-level interception of subs() in FuncCall branch prevents q=1 from becoming Bool
- Three substitution modes: q=rational (evaluate), q=0 (constant term), q=q^k (exponent scaling)
- subs appears in help under Polynomial Operations with documentation of all modes
- 13 new tests (11 unit + 2 help), total: 580 CLI unit + 152 CLI integration = 732

## Task Commits

Each task was committed atomically:

1. **Task 1: subs() AST interception and substitution logic** - `264cee6` (feat)
2. **Task 2: subs help entry and tab completion** - `8c54a9f` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/eval.rs` - subs AST interception, perform_substitution(), evaluate_fps_at_rational(), signatures, ALL_FUNCTION_NAMES, 11 unit tests
- `crates/qsym-cli/src/help.rs` - FuncHelp entry for subs, general_help listing, count updates (94->95), 2 new tests
- `crates/qsym-cli/src/repl.rs` - subs in canonical_function_names (96->97)

## Decisions Made
- AST interception catches Compare(Eq) before arg evaluation so q=1 is never interpreted as Bool
- evaluate_fps_at_rational handles negative exponents via inversion, with error on division by zero
- Exponent scaling preserves POLYNOMIAL_ORDER sentinel for exact polynomials, multiplies truncation_order for truncated series
- Variable name mismatch in subs returns target unchanged (no-op) rather than erroring

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- subs() is fully functional for all three substitution modes
- Combined with factor() from plan 44-01, Polynomial Operations category is complete
- Ready for next phase (tutorial flow or additional operations)

## Self-Check: PASSED

- eval.rs modified: FOUND
- help.rs modified: FOUND
- repl.rs modified: FOUND
- Commit 264cee6: FOUND
- Commit 8c54a9f: FOUND

---
*Phase: 44-polynomial-operations*
*Completed: 2026-02-20*
