# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-18)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** v2.0 Maple Compatibility -- Phase 34: Product & Theta Signatures

## Current Position

Phase: 34 of 40 (Product & Theta Signatures)
Plan: 0 of TBD in current phase
Status: Ready to plan
Last activity: 2026-02-19 -- Phase 33 complete (3/3 plans, verified)

Progress: [################################              ] 95/? plans (v2.0 phases 33-40 pending)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 95
- Total phases: 33 complete (v1.0-v1.6 + Phase 33), 7 planned (v2.0 phases 34-40)
- Total milestones: 7 complete (v1.0-v1.6), 1 in progress (v2.0)
- Average duration: ~5 min/plan
- Total execution time: ~8 hours

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 33-01 | Symbol foundation | 6min | 2 | 8 |
| 33-02 | Symbol arithmetic | 10min | 2 | 6 |
| 33-03 | Symbol-aware dispatch | 7min | 2 | 5 |

## Accumulated Context

### Decisions

All decisions logged in PROJECT.md Key Decisions table and milestone archives.

- 33-01: Series-dependent tests restructured to use pre-assigned variables instead of AstNode::Q
- 33-01: EvalError::UnknownVariable retained despite no longer being raised by variable eval
- 33-01: Integration tests use etaq(1) wrong arg count for real eval errors instead of undefined_var
- 33-02: POLYNOMIAL_ORDER = 1 billion as sentinel for exact polynomial display
- 33-02: value_to_constant_fps uses series truncation order to preserve polynomial semantics
- 33-02: format_value/format_latex accept &SymbolRegistry for variable name resolution
- 33-03: restart implemented as both Command (REPL) and dispatch function (scripts)
- 33-03: 3-arg aqprod(monomial, var, n) uses n for both Pochhammer count and truncation order
- 33-03: Single-quote strings reuse Token::StringLit (no new token variant)
- 33-03: anames() returns sorted list for deterministic output

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-19
Stopped at: Phase 34 context gathered, ready to plan
Resume file: .planning/phases/34-product-theta-signatures/34-CONTEXT.md
