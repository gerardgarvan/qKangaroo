# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-20)

**Core value:** Every example in Garvan's "q-Product Tutorial" (qmaple.pdf) runs correctly in q-Kangaroo.
**Current focus:** v3.0 Phase 45 - Bivariate Series (in progress)

## Current Position

Phase: 45 of 46 (Bivariate Series)
Plan: 2 of 3 in phase 45 (plan 02 complete)
Status: Plan 45-02 complete, ready for plan 45-03
Last activity: 2026-02-20 -- Plan 45-02 executed (2 tasks, 10 new tests, 751 total CLI tests)

Progress: [==========================================........] 87% (127/~145 plans est.)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 127
- Total phases: 44 complete (v1.0-v2.0 + Phases 41-44), 2 in progress (v3.0)
- Total milestones: 8 complete (v1.0-v1.6, v2.0)
- Average duration: ~5 min/plan
- Total execution time: ~9.2 hours

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 42 | 01 | 4 min | 1 | 1 |
| 42 | 02 | 7 min | 2 | 7 |
| 43 | 01 | 9 min | 2 | 3 |
| 43 | 02 | 7 min | 2 | 3 |
| 44 | 01 | 8 min | 2 | 7 |
| 44 | 02 | 5 min | 2 | 3 |
| 45 | 01 | 7 min | 2 | 4 |
| 45 | 02 | 11 min | 2 | 2 |

## Accumulated Context

### Decisions

All decisions logged in PROJECT.md Key Decisions table and milestone archives.
v2.0 decisions archived in .planning/milestones/v2.0-phases/.

- 42-01: Integer-to-Rational promotion for mixed comparisons via From<QInt> for QRat
- 42-01: is_truthy accepts Bool and Integer (nonzero=true), rejects other types
- 42-01: Boolean operators require Bool operands for type safety
- 42-01: For-loop uses closure pattern for guaranteed variable restore
- 42-01: RETURN intercepted before normal arg evaluation in FuncCall
- 42-02: Procedure struct uses Rc<RefCell<HashMap>> for shared memo table across clones
- 42-02: OptionKw token name avoids collision with Rust Option type
- 42-02: Local variables intentionally not initialized (returns Symbol, Maple behavior)
- 42-02: parse_ident_list helper reused for params, locals, and options
- 42-02: "end" keyword decrements proc_depth in REPL (od/fi handle for/if separately)
- 43-01: Rational exponent arms use denom==1 check then delegate to Integer arms
- 43-01: floor uses rug::Integer::from(floor_ref()) for zero-copy computation
- 43-01: legendre validates p >= 3 and odd but does not check primality (matches Maple)
- 43-01: L alias is case-insensitive via resolve_alias lowercase
- 43-02: series() uses min(T, original_order) semantics -- never extends beyond computed data
- 43-02: expand() 1-arg form uses env.default_order for JacobiProduct conversion
- 43-02: expand() accepts 1 or 3 args, rejects 2 with clear error
- 43-02: series() accepts JacobiProduct, Integer, Rational in addition to Series
- 44-01: Cyclotomic trial division scans from highest n down to 1 for correct factor discovery
- 44-01: fps_to_qratpoly requires POLYNOMIAL_ORDER sentinel to reject truncated series
- 44-01: Negative leading coefficient handled by negating both scalar and primitive part
- 44-01: Factor display uses descending degree order within each parenthesized factor
- 44-02: AST interception catches Compare(Eq) before evaluation so q=1 is not converted to Bool
- 44-02: evaluate_fps_at_rational handles negative exponents via inversion with zero-check
- 44-02: Exponent scaling preserves POLYNOMIAL_ORDER sentinel for exact polynomials
- 44-02: Mismatched variable name in subs returns target unchanged (no-op, not error)
- 45-01: BivariateSeries uses BTreeMap<i64, FPS> for Laurent polynomial representation
- 45-01: Arithmetic follows free-function pattern matching series::arithmetic module
- 45-01: format_series made pub(crate) for reuse in bivariate coefficient display
- 45-01: Multi-term FPS coefficients parenthesized in display, single-term inline
- 45-01: Truncation propagation uses min(a, b) consistently across all operations
- 45-02: Symbolic z detection via Symbol name comparison (z != q triggers bivariate path)
- 45-02: Cross-validation uses z=-q^m to avoid product zeros at integer q-powers
- 45-02: Quinprod validation uses direct coefficient verification against sum formula
- 45-02: Bivariate sum forms have truncation boundary effects when evaluated at z=c*q^m

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-20
Stopped at: Completed 45-02-PLAN.md
Resume file: N/A
