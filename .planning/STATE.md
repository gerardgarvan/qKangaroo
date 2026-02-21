# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-21)

**Core value:** Every example in Garvan's "q-Product Tutorial" (qmaple.pdf) runs correctly in q-Kangaroo.
**Current focus:** v4.0 Full qmaple.pdf Parity -- Phase 50 in progress, plan 01 complete

## Current Position

Phase: 50 of 51 (New Functions)
Plan: 1 of 2 in current phase (1 complete)
Status: Plan 50-01 complete, ready for 50-02
Last activity: 2026-02-21 -- Completed 50-01 (jac2series Garvan form + quinprod identity modes)

Progress: [|||||||||||||||||||||||||||||||||||||||||||||||||░░░] 96% (49/51 phases)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 141
- Total phases: 46 complete (v1.0-v3.0), 5 planned (v4.0)
- Total milestones: 9 complete (v1.0-v1.6, v2.0, v3.0)
- Average duration: ~5 min/plan
- Total execution time: ~10.5 hours

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 47    | 01   | 6min     | 2     | 5     |
| 47    | 03   | 42min    | 2     | 2     |
| 47    | 02   | 3min     | 2     | 5     |
| 48    | 01   | 4min     | 2     | 2     |
| 48    | 02   | 5min     | 2     | 2     |
| 49    | 01   | 10min    | 2     | 4     |
| 49    | 02   | 4min     | 2     | 4     |
| 50    | 01   | 5min     | 2     | 2     |

## Accumulated Context

### Decisions

All decisions logged in PROJECT.md Key Decisions table and milestone archives.
v3.0 decisions archived in .planning/milestones/v3.0-ROADMAP.md.

- 47-01: Ditto disambiguated via byte-lookahead (next char after quote) rather than parser-level context
- 47-01: Proc option/local uses loop for either-order parsing
- 47-03: Monomial division uses shift+scalar instead of invert to avoid O(POLYNOMIAL_ORDER) loop
- 47-03: Fractional exponents displayed with GCC reduction (2/4 -> 1/2)
- 47-03: FractionalPowerSeries auto-simplifies back to Series when all keys are denom-multiples
- 47-02: Arrow l_bp=2 matches assignment level so F := q -> expr parses correctly
- 47-02: Lambda desugars to Value::Procedure (no new Value variant needed)
- 48-01: aqprod 3-arg uses POLYNOMIAL_ORDER directly (not tight+rewrap) since sparse multiply is efficient
- 48-01: theta 3-arg form theta3(a,q,T) extracts variable from args[1], ignoring args[0] when a==q
- 48-02: qfactor 2-arg uses match on Value::Symbol vs Value::Integer for disambiguation
- 48-02: min/max return original Value (preserving Integer vs Rational type) via index tracking
- 49-01: QProduct test assertions use matches!() not is_empty() since qbin factorization can produce empty factors
- 49-01: Format functions added in Task 1 (not Task 2) to resolve compilation dependency
- 49-02: EtaQuotient variant placed after QProduct in Value enum
- 49-02: Format functions added in Task 1 (not Task 2) to resolve compilation dependency (same pattern as 49-01)
- 50-01: Separate jacobi_product_to_fps_garvan function preserves backward compat for legacy 3-arg path
- 50-01: Identity modes return Value::String (not Series) for formatted display

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-21
Stopped at: Completed 50-01-PLAN.md (jac2series Garvan form + quinprod identity modes)
Resume: Continue with 50-02
