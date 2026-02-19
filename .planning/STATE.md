# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-18)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** v2.0 Maple Compatibility -- Phase 35 complete, ready for Phase 36

## Current Position

Phase: 35 of 40 (Series Analysis Signatures) -- COMPLETE
Plan: 2 of 2 in current phase (all done)
Status: Phase 35 complete, ready for Phase 36
Last activity: 2026-02-19 -- Plan 35-02 complete (help text + 12 integration tests)

Progress: [###################################           ] 100/? plans (v2.0 phases 33-40 pending)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 100
- Total phases: 35 complete (v1.0-v1.6 + Phases 33-35), 5 planned (v2.0 phases 36-40)
- Total milestones: 7 complete (v1.0-v1.6), 1 in progress (v2.0)
- Average duration: ~5 min/plan
- Total execution time: ~8 hours

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 33-01 | Symbol foundation | 6min | 2 | 8 |
| 33-02 | Symbol arithmetic | 10min | 2 | 6 |
| 33-03 | Symbol-aware dispatch | 7min | 2 | 5 |
| 34-01 | Product/theta Maple dispatch | 12min | 2 | 1 |
| 34-02 | Numbpart canonical + help + tests | 6min | 2 | 4 |
| 35-01 | Series analysis Maple dispatch | 8min | 2 | 4 |
| 35-02 | Help text + integration tests | 4min | 2 | 2 |

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
- 34-01: jacprod Maple-style uses JAC(a,b)/JAC(b,3b) per Garvan source, distinct from legacy JAC(a,b)
- 34-01: qbin Garvan form uses tight truncation then re-wraps with POLYNOMIAL_ORDER sentinel
- 34-01: etaq multi-delta validates non-empty list and positive deltas using EvalError::Other
- 34-01: Used arithmetic::invert + mul for series division (no arithmetic::div exists)
- 34-02: numbpart is canonical, partition_count is alias (reversed direction)
- 34-02: numbpart(n,m) uses bounded_parts_gf to count bounded partitions
- 34-02: help(partition_count) redirects to numbpart via function_help lookup
- 34-02: Piped help tests replaced with -c flag tests (help commands only work in interactive REPL)
- 35-01: sift validates k range at CLI level, core sift normalizes j internally
- 35-01: sift truncates input series to T before calling core sift for Maple-accurate truncation
- 35-01: jacprodmake_impl uses Option<i64> period_divisor for code reuse
- 35-01: qfactor accepts optional T arg for Maple compat but ignores it (already degree-bounded)
- 35-01: No backward compat for series analysis functions -- old arg counts error
- 35-02: Help examples use two-line format (assign then call) matching Maple documentation style
- 35-02: qfactor integration test uses aqprod(q, q, 5, 20) for complete polynomial factoring
- 35-02: Old-signature error tests check for exact "expects N arguments" message format

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-19
Stopped at: Completed 35-02-PLAN.md (Phase 35 complete)
Resume file: .planning/phases/35-series-analysis-signatures/35-02-SUMMARY.md
