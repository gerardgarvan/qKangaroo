# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-18)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** Phase 30 - Script Execution & CLI Flags (v1.6 CLI Hardening & Manual)

## Current Position

Phase: 30 of 32 (Script Execution & CLI Flags)
Plan: 1 of 3 in current phase
Status: Executing
Last activity: 2026-02-18 -- 30-01 complete (script execution engine)

Progress: [========================......] 83/TBD plans (v1.0-v1.5 complete, v1.6 Phase 29-30 in progress)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 82
- Total phases: 29 complete, 3 remaining
- Total milestones: 6 complete (v1.0-v1.5), 1 in progress (v1.6)
- Average duration: ~5 min/plan
- Total execution time: ~8 hours

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 29-01 | Static GMP Linking | 100min | 2 | 2 |
| 29-02 | CI Release Workflow | 2min | 2 | 1 |
| 30-01 | Script Execution Engine | 5min | 2 | 8 |

## Accumulated Context

### Decisions

All v1.0-v1.5 decisions logged in PROJECT.md Key Decisions table.

v1.6-relevant decisions:
- Phase 30-01: Sysexits-compatible exit codes (0=success, 1=eval-error, 2=usage, 65=parse-error, 66=file-not-found, 70=panic)
- Phase 30-01: Script engine uses fail-fast semantics (stops on first error)
- Phase 30-01: String escape sequences limited to \\, \", \n, \t
- Phase 29-01: Remove use-system-libs feature entirely for static GMP/MPFR/MPC linking
- Phase 29-01: Clear .cargo/config.toml completely to prevent accidental dynamic linking
- Phase 29-01: Pre-build static GMP/MPFR/MPC for Cygwin local dev via gmp-mpfr-sys cache
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
Stopped at: Completed 30-01-PLAN.md (script execution engine)
Resume file: N/A
