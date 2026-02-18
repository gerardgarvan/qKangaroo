# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-18)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** Phase 31 - Error Hardening & Exit Codes (v1.6 CLI Hardening & Manual)

## Current Position

Phase: 31 of 32 (Error Hardening & Exit Codes)
Plan: 2 of 2 in current phase (PHASE COMPLETE)
Status: Phase 31 complete
Last activity: 2026-02-18 -- Completed 31-02 (error hardening integration tests)

Progress: [===========================...] 86/TBD plans (v1.0-v1.5 complete, v1.6 Phases 29-31 complete)

## Performance Metrics

### Cumulative Summary

- Total plans completed: 86
- Total phases: 31 complete, 1 remaining
- Total milestones: 6 complete (v1.0-v1.5), 1 in progress (v1.6)
- Average duration: ~5 min/plan
- Total execution time: ~8 hours

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 29-01 | Static GMP Linking | 100min | 2 | 2 |
| 29-02 | CI Release Workflow | 2min | 2 | 1 |
| 30-01 | Script Execution Engine | 5min | 2 | 8 |
| 30-02 | CLI Arg Parsing & Mode Dispatch | 5min | 2 | 3 |
| 30-03 | CLI Integration Tests | 2min | 1 | 1 |
| 31-01 | Error Hardening Infrastructure | 5min | 2 | 5 |
| 31-02 | Error Hardening Integration Tests | 4min | 1 | 2 |

## Accumulated Context

### Decisions

All v1.0-v1.5 decisions logged in PROJECT.md Key Decisions table.

v1.6-relevant decisions:
- Phase 31-02: Custom panic hook (set_hook) suppresses raw "thread panicked" output for clean error messages
- Phase 30-03: Integration tests use env!(CARGO_BIN_EXE_q-kangaroo) for binary path resolution
- Phase 30-03: Windows backslash escaping in read() test paths via replace('\\', '\\\\')
- Phase 30-02: Hand-written argument parser (no clap) consistent with zero-external-deps philosophy
- Phase 31-01: Panic translation uses contains() matching for robustness against upstream wording changes
- Phase 31-01: read() errors use EvalError::Other (not Panic) for file/parse errors
- Phase 31-01: Statement line mapping via lexer tokenize() byte offsets for filename:line context
- Phase 30-02: read() function propagates script errors via EvalError::Panic (updated in 31-01)
- Phase 30-02: CommandResult::ReadFile defers execution to main loop (needs env + verbose)
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
Stopped at: Completed 31-02-PLAN.md (error hardening integration tests) -- Phase 31 complete
Resume file: N/A
