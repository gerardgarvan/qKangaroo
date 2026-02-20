# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-18)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** v2.0 Maple Compatibility -- Phase 37 complete, ready for Phase 38

## Current Position

Phase: 37 of 40 (New Functions - Theta & Jacobi) -- COMPLETE
Plan: 2 of 2 in current phase (all plans complete)
Status: Phase 37 complete, ready for Phase 38
Last activity: 2026-02-20 -- Plan 37-02 complete (qs2jaccombo + help + integration tests)

Progress: [####################################          ] 106/? plans (v2.0 phases 33-40 pending)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 106
- Total phases: 37 complete (v1.0-v1.6 + Phases 33-37), 3 in progress/planned (v2.0 phases 38-40)
- Total milestones: 7 complete (v1.0-v1.6), 1 in progress (v2.0)
- Average duration: ~5 min/plan
- Total execution time: ~8.4 hours

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 33-01 | Symbol foundation | 6min | 2 | 8 |
| 33-02 | Symbol arithmetic | 10min | 2 | 6 |
| 33-03 | Symbol-aware dispatch | 7min | 2 | 5 |
| 34-01 | Product/theta Maple dispatch | 12min | 2 | 1 |
| 34-02 | Numbpart canonical + help + tests | 6min | 2 | 4 |
| 35-01 | Series analysis Maple dispatch | 8min | 2 | 4 |
| 35-02 | Help text + integration tests | 4min | 2 | 2 |
| 36-01 | Pub monomial generators + findcong_garvan | 4min | 2 | 2 |
| 36-02 | Garvan-compatible relation discovery dispatch | 5min | 1 | 1 |
| 36-03 | Help text + integration tests | 6min | 2 | 2 |
| 37-01 | JacobiProduct type + theta/jac2prod/jac2series | 8min | 2 | 2 |
| 37-02 | qs2jaccombo + help + integration tests | 9min | 2 | 4 |

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
- 36-01: findcong_garvan uses rug::Integer GCD with abs_ref for sign safety
- 36-01: trial_factor returns Vec<(i64, u32)> for simple prime-power iteration
- 36-01: generate_nonhom_monomials delegates to generate_monomials for each degree level
- 36-01: Test truncation uses t=99 with partition_gf(100) to respect O(q^100) bound
- 36-02: SL label validation uses strict match (labels.len() == candidates.len())
- 36-02: validate_unique_labels uses HashSet for O(n) duplicate detection
- 36-02: is_prime uses trial division (6k+-1) for small p validation at dispatch time
- 36-02: default_labels generates X[1]..X[k] matching Garvan convention
- 36-02: findmaxind returns 1-based indices matching Garvan convention
- 36-02: findcong dispatches to findcong_garvan with auto-scan algorithm
- 36-02: findpoly uses fixed topshift=10 matching Garvan's dim2:=dim1+10
- 36-03: Help examples use two-line assign-then-call format matching Maple documentation style
- 36-03: findcong integration tests use partition_gf(201) with T=200 to avoid boundary access error
- 36-03: Script-based integration tests use colon terminators for multi-statement separation
- 37-01: JacobiProduct uses Vec<(a,b,exponent)> sorted by (b,a) as canonical form
- 37-01: JAC(a,b) validates b>0 but allows a=0 (degenerate case handled by etaq)
- 37-01: theta dispatches on z type: numeric, monomial, symbol (with warning)
- 37-01: jac2prod/jac2series use print-and-return pattern matching Phase 36 find* functions
- 37-01: Add/sub with JacobiProduct gives helpful error directing to jac2series()
- 37-01: env.symbols.name(sym) used for product notation formatting
- 37-02: qs2jaccombo Phase A uses jacprodmake is_exact; Phase B uses findlincombo over candidate JAC basis
- 37-02: Candidate JAC basis generated from periods identified by jacprodmake, fallback to 2..min(T,20)
- 37-02: Help integration tests cannot use -c flag (help is REPL command); use functional tests instead
- 37-02: complete_theta test updated from 3 to 4 candidates (theta added alongside theta2/3/4)

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-20
Stopped at: Completed 37-02-PLAN.md (Phase 37 complete)
Resume file: .planning/phases/38-*/38-01-PLAN.md (next phase)
