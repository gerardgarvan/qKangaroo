# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-18)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** v2.0 Maple Compatibility -- Phase 40 complete

## Current Position

Phase: 40 of 40 (Documentation)
Plan: 5 of 5 in current phase (COMPLETE)
Status: Phase 40 complete -- all 5 plans executed
Last activity: 2026-02-20 -- Completed 40-05 (Worked examples Garvan rewrite)

Progress: [##########################################    ] 115/115 plans (v2.0 phases 33-40)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 115
- Total phases: 40 complete (v1.0-v1.6 + Phases 33-40)
- Total milestones: 7 complete (v1.0-v1.6), 1 in progress (v2.0)
- Average duration: ~5 min/plan
- Total execution time: ~8.5 hours

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
| 38-01 | Analysis/discovery dispatch (lqdegree0/checkmult/checkprod/findprod) | 6min | 2 | 1 |
| 38-02 | Help text + integration tests | 3min | 2 | 2 |
| 39-01 | Descending power ordering | 7min | 2 | 5 |
| 39-02 | Backward compatibility tests | 5min | 2 | 1 |
| 40-01 | Manual chapters 05-07 Garvan signatures | 6min | 2 | 3 |
| 40-03 | Peripheral doc fixes (counts, tab completion, README) | 3min | 2 | 7 |
| 40-02 | Series Analysis & Relations manual chapters | 4min | 2 | 2 |
| 40-04 | Maple migration guide rewrite | 2min | 1 | 1 |
| 40-05 | Worked examples Garvan rewrite | 20min | 1 | 2 |

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
- 38-01: checkprod_impl as private eval.rs helper (not qsym-core) for simplicity
- 38-01: gcd_i64 private helper in eval.rs (not reusing qsym-core private gcd)
- 38-01: Old 3-arg findprod fully replaced by new 4-arg Garvan version
- 38-01: Value comparison in tests uses pattern matching (Value lacks PartialEq)
- 38-02: checkmult/checkprod placed in Series Analysis section of general help (not Relations)
- 38-02: findprod description updated to 'search for product identities in series list'
- 39-01: FormalPowerSeries::iter() returns impl DoubleEndedIterator to enable .rev()
- 39-01: fps_to_latex() uses iter().rev().collect() to reverse terms Vec once
- 39-02: winquist legacy is 7-arg (not 4-arg as plan stated) -- corrected test
- 39-02: etaq has no 4-arg legacy form -- skipped backward_compat_etaq_legacy_4arg
- 39-02: All backward_compat tests validate output correctness, not just exit code 0
- 40-01: Chapter 05 has 11 func-entry blocks (7 original + 4 Jacobi); theta placed in ch07 per domain
- 40-01: Garvan cross-references added to chapter intro and aqprod description per CONTEXT.md
- 40-01: General theta entry placed before theta2/3/4 specializations in chapter 07
- 40-02: All legacy manual signatures replaced entirely -- no dual-signature documentation entries
- 40-02: New functions lqdegree0/checkmult/checkprod placed after qetamake in chapter 08
- 40-02: findcong entry shows all 3 overloaded forms with [B, A, R] output format
- 40-02: findprod entry explicitly notes completely different semantics from legacy version
- 40-02: findmaxind documented as 1-based indices per Garvan convention
- 40-03: Chapter 04 function listing expanded from 8 groups to 9 (added Jacobi Products) with full enumeration
- 40-03: DOC-02 confirmed: help.rs already fully updated with 89 Garvan-canonical entries
- 40-03: DOC-04 confirmed: Python API uses own calling conventions, unaffected by v2.0 changes
- 40-04: Migration guide organized by workflow (eta products, series analysis, congruences, relations, theta/Jacobi, products) not alphabetically
- 40-04: Two-column tables confirm identical syntax rather than only listing differences
- 40-04: Hypergeometric triple encoding identified as main remaining divergence area
- 40-05: etaq Garvan dispatch fixed: t=delta instead of t=1 (bug fix for delta>1)
- 40-05: Jacobi triple product uses eta-quotient verification (engine cannot express (-q;q^2)_inf via aqprod)
- 40-05: Mock theta section restructured: rhs=mf+4*mpsi to demonstrate findlincombo (Watson relation not verifiable)
- 40-05: findcong examples use findcong(QS, T, LM) form showing compound congruences

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-20
Stopped at: Completed 40-05-PLAN.md (Worked examples Garvan rewrite) -- Phase 40 complete
Resume file: .planning/phases/40-documentation/
