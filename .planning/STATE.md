# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-13)

**Core value:** Every function in Garvan's Maple packages works correctly in Q-Symbolic, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** Phase 2 - Simplification & Series Engine

## Current Position

Phase: 2 of 8 (Simplification & Series Engine)
Plan: 1 of 3 in current phase
Status: In Progress
Last activity: 2026-02-13 -- Completed 02-01-PLAN.md

Progress: [####......] 17%

## Performance Metrics

**Velocity:**
- Total plans completed: 4
- Average duration: 10 min
- Total execution time: 0.7 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 - Expression Foundation | 3/3 | 37 min | 12 min |
| 2 - Simplification & Series Engine | 1/3 | 4 min | 4 min |

**Recent Trend:**
- Last 5 plans: 26 min, 5 min, 6 min, 4 min
- Trend: accelerating

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
- [01-02]: Division-by-zero panics (assert!) rather than Result -- matches rug and Rust Div convention
- [01-02]: Integer division is truncating (floor toward zero) per rug default and Rust convention
- [01-03]: Always-brace policy for LaTeX sub/superscripts to eliminate edge-case bugs
- [01-03]: ASCII fallback for non-numeric Unicode sub/superscripts (digits only get Unicode rendering)
- [01-03]: Neg detection in Add: renders as subtraction (a - b) not addition of negative (a + -b)
- [02-01]: Hardcoded 'q' as display variable name in FPS Display impl -- no SymbolRegistry access; Phase 3+ can add display_with_arena
- [02-01]: Shift adjusts truncation_order by k (shift(f, k) has trunc = f.trunc + k)
- [02-01]: pub(crate) fields on FPS -- arithmetic accesses directly, external users use API
- [02-01]: PartialEq compares variable + truncation_order + coefficient maps (value equality)

### Pending Todos

None yet.

### Blockers/Concerns

- [Research]: Andrews' algorithm (prodmake/etamake/jacprodmake) needs implementation strategy research in Phase 4
- [Research]: Identity proving (Phase 7) needs deep research on cusp theory and valence formula
- [Research]: Mock theta and Bailey chains (Phase 8) need algorithm extraction from academic literature
- [Build]: Windows build requires MinGW GCC 14.2.0 + pre-built GMP in PATH. See .cargo/config.toml for env vars. Must use `export PATH="/c/mingw64-gcc/mingw64/bin:/c/cygwin64/bin:/c/Users/Owner/.cargo/bin:$PATH"` before cargo commands.

## Session Continuity

Last session: 2026-02-13
Stopped at: Completed 02-01-PLAN.md (FPS data structure and series arithmetic with 33 TDD tests)
Resume file: .planning/phases/02-simplification-series-engine/02-01-SUMMARY.md
