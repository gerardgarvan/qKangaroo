# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-17)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** v1.5 Interactive REPL -- Phase 24 (Parser & AST)

## Current Position

Phase: 24 of 28 (Parser & AST)
Plan: 1 of 2 in current phase
Status: Executing
Last activity: 2026-02-17 -- Plan 24-01 complete (AST, Token, Span, Error types)

Progress: [########################------] 79% (71/79 plans -- v1.0-v1.4 complete, v1.5 in progress)

## Performance Metrics

### v1.0-v1.4 Summary

- Total plans completed: 70
- Average duration: ~6 min/plan
- Total execution time: ~7 hours

### v1.5 Interactive REPL

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 24 - Parser & AST | 1/2 | 2min | 2min |
| 25 - Evaluator & Function Dispatch | 0/3 | - | - |
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

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-17
Stopped at: Completed 24-01-PLAN.md
Resume file: .planning/phases/24-parser-ast/24-01-SUMMARY.md
