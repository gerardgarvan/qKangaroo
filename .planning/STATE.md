# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-18)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** Phase 29 - Static Linking (v1.6 CLI Hardening & Manual)

## Current Position

Phase: 29 of 32 (Static Linking)
Plan: 2 of 2 in current phase
Status: Phase 29 complete
Last activity: 2026-02-18 -- Completed 29-02 (CI release workflow)

Progress: [========================......] 81/TBD plans (v1.0-v1.5 complete, v1.6 Phase 29 complete)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 81
- Total phases: 28 complete + Phase 29 complete, remaining planned
- Total milestones: 6 complete (v1.0-v1.5), 1 in progress (v1.6)
- Average duration: ~5 min/plan
- Total execution time: ~8 hours

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 29-02 | CI Release Workflow | 2min | 2 | 1 |

## Accumulated Context

### Decisions

All v1.0-v1.5 decisions logged in PROJECT.md Key Decisions table.

v1.6-relevant decisions:
- Phase 29-02: Run cargo from MSYS2 shell on Windows CI for full build tool compatibility
- Phase 29-02: Cache gmp-mpfr-sys build artifacts in platform-specific directories
- Phase 29-02: Use objdump/ldd as CI gates to prevent dependency regressions
- Phase 28: Bundle MinGW DLLs (not static GMP) -- now reversed in Phase 29
- Phase 24: Hand-written Pratt parser -- no external parser libs; same approach for CLI arg parsing

### Pending Todos

None.

### Blockers/Concerns

- Phase 29: First GMP-from-source CI build will be slow (~2-5 min); subsequent builds cached
- Phase 29: libgcc_s_seh-1.dll and libwinpthread-1.dll confirmed NOT runtime dependencies (Rust statically links libgcc_eh and libpthread on windows-gnu) -- blocker resolved

## Session Continuity

Last session: 2026-02-18
Stopped at: Completed 29-02-PLAN.md (CI release workflow for static builds)
Resume file: N/A
