# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-13)

**Core value:** Every function in Garvan's Maple packages works correctly in Q-Symbolic, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** Phase 1 - Expression Foundation

## Current Position

Phase: 1 of 8 (Expression Foundation)
Plan: 1 of 3 in current phase
Status: Executing
Last activity: 2026-02-13 -- Completed 01-01-PLAN.md

Progress: [#.........] 4%

## Performance Metrics

**Velocity:**
- Total plans completed: 1
- Average duration: 26 min
- Total execution time: 0.4 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 - Expression Foundation | 1/3 | 26 min | 26 min |

**Recent Trend:**
- Last 5 plans: 26 min
- Trend: baseline

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Roadmap]: 8 phases derived from 62 v1 requirements; strict linear dependency chain
- [Roadmap]: Python API placed after qseries parity (Phase 5) so Rust API stabilizes first
- [Roadmap]: Partition basics (PART-01 through PART-03) grouped with core q-series (Phase 3) since they are natural applications of q-Pochhammer
- [Roadmap]: Mock theta and Bailey chains grouped together (Phase 8) as the most advanced extensions
- [01-01]: Manual Hash impl for QInt/QRat via to_digits() + sign (rug types may not impl Hash on all platforms)
- [01-01]: Kept Neg as separate Expr variant (not Mul([-1, x])) for Phase 1 simplicity
- [01-01]: ExprRef u32 numeric ordering for canonical sort (deterministic within session)
- [01-01]: Pre-built GMP/MPFR system libs via MSYS2 packages for Windows GNU target
- [01-01]: GNU toolchain (x86_64-pc-windows-gnu) required; MSVC target unsupported by gmp-mpfr-sys

### Pending Todos

None yet.

### Blockers/Concerns

- [Research]: Andrews' algorithm (prodmake/etamake/jacprodmake) needs implementation strategy research in Phase 4
- [Research]: Identity proving (Phase 7) needs deep research on cusp theory and valence formula
- [Research]: Mock theta and Bailey chains (Phase 8) need algorithm extraction from academic literature
- [Build]: Windows build requires MinGW GCC 14.2.0 + pre-built GMP in PATH. See .cargo/config.toml for env vars. Must use `export PATH="/c/mingw64-gcc/mingw64/bin:/c/cygwin64/bin:/c/Users/Owner/.cargo/bin:$PATH"` before cargo commands.

## Session Continuity

Last session: 2026-02-13
Stopped at: Completed 01-01-PLAN.md (workspace scaffold + ExprArena)
Resume file: .planning/phases/01-expression-foundation/01-01-SUMMARY.md
