# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-17)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** v1.5 Interactive REPL -- Phase 26 (REPL Shell & Session)

## Current Position

Phase: 26 of 28 (REPL Shell & Session)
Plan: 0 of 2 in current phase
Status: Ready
Last activity: 2026-02-17 -- Phase 25 complete (Evaluator & Function Dispatch: 81 functions dispatched, 213 tests)

Progress: [############################--] 95% (75/79 plans -- v1.0-v1.4 complete, v1.5 in progress)

## Performance Metrics

### v1.0-v1.4 Summary

- Total plans completed: 70
- Average duration: ~6 min/plan
- Total execution time: ~7 hours

### v1.5 Interactive REPL

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 24 - Parser & AST | 2/2 | 5min | 2.5min |
| 25 - Evaluator & Function Dispatch | 3/3 | 20min | 6.7min |
| 26 - REPL Shell & Session | 0/2 | - | - |
| 27 - Output Commands & Polish | 0/1 | - | - |
| 28 - Binary Packaging | 0/1 | - | - |

## Accumulated Context

### Decisions

All v1.0-v1.4 decisions logged in PROJECT.md Key Decisions table.

v1.5 decisions:
- qsym-cli depends only on qsym-core -- hand-written parser, no external libraries
- AST nodes carry no span information (simplicity; can add Spanned<T> wrapper later)
- q is a reserved keyword token (Token::Q), not treated as an identifier
- BigInteger stored as String for arbitrary precision; evaluator converts to QInt
- Binding powers: := (2,1) < +/- (3,4) < */ (5,6) < unary- (7) < ^ (9,10) < funcall (11)
- Non-associative ^ enforced via post-parse check (error if chained)
- Function call as postfix operator requiring Variable lhs
- Empty statements (;;) silently skipped
- Value enum: 9 variants (Series, Integer, Rational, List, Dict, Pair, Bool, None, Infinity)
- Series + Integer promotes integer to constant FPS (matches Maple behavior)
- Integer / Integer produces Rational (exact arithmetic)
- 16 Maple aliases (case-insensitive): numbpart→partition_count, qphihyper→phi, etc.
- Levenshtein fuzzy matching for "did you mean" suggestions on unknown functions
- Panic catching via catch_unwind(AssertUnwindSafe(...)) for rug type safety
- prove_nonterminating returns informative error (requires closures, Python-only)
- partition_count returns Value::Integer by extracting QRat numerator
- Analysis result structs converted to Value::Dict with string keys for REPL display
- expect_args/expect_args_range auto-populate signature from get_signature()
- Bailey pairs use integer codes (0=Unit, 1=RR, 2=QBinomial)
- Heine/sears/watson transforms return Pair(prefactor, evaluated_result)
- verify_wz is self-contained: runs q_zeilberger first then verifies

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-17
Stopped at: Phase 25 complete. Ready for Phase 26.
Resume file: N/A
