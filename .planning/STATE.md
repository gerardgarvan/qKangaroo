# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-21)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output.
**Current focus:** v5.0 Maximum Maple Compatibility -- Phase 53: Lists & List Operations

## Current Position

Phase: 53 (second of 5 in v5.0)
Plan: 02 complete
Status: Executing
Last activity: 2026-02-22 -- Phase 53 plan 02 (list operation functions) complete

Progress: [##########..........] 52/56 phases (93% overall)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 151
- Total phases: 52 complete (v1.0-v4.0, v5.0 phase 52)
- Total milestones: 10 complete (v1.0-v1.6, v2.0, v3.0, v4.0)
- Average duration: ~5 min/plan
- Total execution time: ~10.7 hours

## Accumulated Context

### Decisions

All decisions logged in PROJECT.md Key Decisions table and milestone archives.
v4.0 decisions archived in .planning/milestones/v4.0-ROADMAP.md.

- 52-01: Fix division hang in eval_div (CLI layer) not core arithmetic::invert()
- 52-01: Unicode normalization before tokenization (string contents also normalized)
- 52-01: print() returns last value (not NULL like Maple)
- 52-02: while and for share the same od-depth counter in REPL is_incomplete
- 52-02: while-loop does not introduce a new scope (unlike for-loop)
- 52-02: Safety limit set at 1,000,000 iterations
- 52-03: Symbol true/false handled in is_truthy, not in parser or lexer
- 52-03: ? prefix check placed before word splitting in parse_command
- 53-01: Index works on arbitrary LHS expressions (not just variables)
- 53-01: 1-indexed Maple convention; L[0] is out-of-range error
- 53-01: Symbol fallback for table-style X[i] when base is unbound
- 53-02: nops on FPS counts nonzero terms directly (sparse storage)
- 53-02: op on series returns [exponent, coefficient] list
- 53-02: map dispatches Symbol names through dispatch() for Maple compatibility
- 53-02: sort defers error via Option<String> in closure (sort_by can't return Result)

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-22
Stopped at: Completed 53-02-PLAN.md (list operation functions)
Resume: Execute Phase 53 plan 03 (list construction functions)
