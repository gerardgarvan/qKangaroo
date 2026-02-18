# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-17)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** v1.5 Interactive REPL -- Phase 25 (Evaluator & Function Dispatch)

## Current Position

Phase: 25 of 28 (Evaluator & Function Dispatch)
Plan: 2 of 3 in current phase
Status: In Progress
Last activity: 2026-02-18 -- Plan 25-02 complete (dispatch groups 1-4: 25 functions, 181 tests)

Progress: [##########################----] 84% (74/79 plans -- v1.0-v1.4 complete, v1.5 in progress)

## Performance Metrics

### v1.0-v1.4 Summary

- Total plans completed: 70
- Average duration: ~6 min/plan
- Total execution time: ~7 hours

### v1.5 Interactive REPL

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 24 - Parser & AST | 2/2 | 5min | 2.5min |
| 25 - Evaluator & Function Dispatch | 2/3 | 11min | 5.5min |
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
- Series + Integer promotes integer to constant FPS (matches Maple behavior)
- Integer / Integer produces Rational (exact arithmetic for mathematicians)
- rug added as direct dependency to qsym-cli for BigInteger parsing
- Dispatch stub returns UnknownFunction with fuzzy suggestions -- Plans 02/03 fill in
- partition_count returns Value::Integer by extracting QRat numerator (always integer-valued)
- Analysis result structs converted to Value::Dict with string keys for REPL display
- expect_args/expect_args_range auto-populate signature from get_signature()

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-18
Stopped at: Completed 25-02-PLAN.md (dispatch groups 1-4: 25 functions). Ready for 25-03 (remaining function groups).
Resume file: .planning/phases/25-evaluator-function-dispatch/25-02-SUMMARY.md
