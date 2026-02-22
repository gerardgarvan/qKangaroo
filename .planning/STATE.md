# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-21)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output.
**Current focus:** v5.0 Maximum Maple Compatibility -- Phase 54: Series & Utility Functions

## Current Position

Phase: 54 (third of 5 in v5.0)
Plan: 01 complete
Status: Plan 54-01 complete
Last activity: 2026-02-22 -- Phase 54 Plan 01 executed (9 utility functions, 39 new tests)

Progress: [##########..........] 53/56 phases (95% overall)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 152
- Total phases: 53 complete (v1.0-v4.0, v5.0 phases 52-53)
- Total milestones: 10 complete (v1.0-v1.6, v2.0, v3.0, v4.0)
- Average duration: ~5 min/plan
- Total execution time: ~10.8 hours

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 54-01 | series-utility-functions | 7min | 2 | 4 |

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
- 54-01: coeff returns Integer when denom==1, Rational otherwise
- 54-01: type() accepts both Symbol and String as type name argument
- 54-01: cat() returns Value::Symbol matching Maple's cat behavior
- 54-01: modp/mods use ((a%p)+p)%p for correct negative handling
- 54-01: Unknown type names in type() return false (not error)

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-22
Stopped at: Completed 54-01-PLAN.md
Resume: Check if more plans in Phase 54, or advance to Phase 55
