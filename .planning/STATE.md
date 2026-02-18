# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-18)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** Phase 29 - Static Linking (v1.6 CLI Hardening & Manual)

## Current Position

Phase: 29 of 32 (Static Linking)
Plan: 0 of TBD in current phase
Status: Ready to plan
Last activity: 2026-02-18 -- Roadmap created for v1.6

Progress: [========================......] 79/TBD plans (v1.0-v1.5 complete, v1.6 starting)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 79
- Total phases: 28 complete, 4 planned
- Total milestones: 6 complete (v1.0-v1.5), 1 in progress (v1.6)
- Average duration: ~5 min/plan
- Total execution time: ~8 hours

## Accumulated Context

### Decisions

All v1.0-v1.5 decisions logged in PROJECT.md Key Decisions table.

v1.6-relevant prior decisions:
- Phase 28: Bundle MinGW DLLs (not static GMP) -- now being reversed in Phase 29
- Phase 24: Hand-written Pratt parser -- no external parser libs; same approach for CLI arg parsing

### Pending Todos

None.

### Blockers/Concerns

- Phase 29: Verify libgcc_s_seh-1.dll and libwinpthread-1.dll are also eliminated (likely transitive deps)
- Phase 29: First GMP-from-source CI build will be slow (~2-5 min); subsequent builds cached

## Session Continuity

Last session: 2026-02-18
Stopped at: Roadmap created for v1.6; ready to plan Phase 29
Resume file: N/A
