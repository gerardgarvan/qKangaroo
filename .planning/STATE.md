# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-14)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** Phase 9 -- Package Rename & Structure

## Current Position

Phase: 9 of 12 (Package Rename & Structure)
Plan: 0 of 2 in current phase
Status: Ready to plan
Last activity: 2026-02-14 -- Roadmap created for v1.1 milestone

Progress: [################################..........] 73% (32/44 plans -- 32 v1.0 complete, 12 v1.1 remaining)

## v1.0 Performance Metrics

**Velocity:**
- Total plans completed: 32
- Average duration: 7 min
- Total execution time: 3.7 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 - Expression Foundation | 3/3 | 37 min | 12 min |
| 2 - Simplification & Series Engine | 3/3 | 14 min | 5 min |
| 3 - Core q-Series & Partitions | 4/4 | 11 min | 3 min |
| 4 - Series Analysis | 7/7 | 57 min | 8 min |
| 5 - Python API | 4/4 | 20 min | 5 min |
| 6 - Hypergeometric Series | 4/4 | 35 min | 9 min |
| 7 - Identity Proving | 4/4 | 25 min | 6 min |
| 8 - Mock Theta & Bailey Chains | 4/4 | 32 min | 8 min |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
v1.0 decisions preserved in MILESTONES.md.

Key v1.1 decisions:
- [v1.1]: Package name q-kangaroo (PyPI) / q_kangaroo (import) -- user-chosen
- [v1.1]: GitHub Pages for documentation hosting
- [v1.1]: CI targets Linux + Windows (macOS deferred)
- [v1.1]: DOC + UX merged into single Phase 12 (shared delivery boundary)

### Pending Todos

None yet.

### Blockers/Concerns

- [Build]: Windows build requires MinGW GCC 14.2.0 + pre-built GMP in PATH
- [Build]: PyO3 builds require PYO3_PYTHON pointing to Python 3.14 and PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
- [Rename]: qsymbolic -> q_kangaroo rename touches many files (Cargo.toml, pyproject.toml, __init__.py, all tests, all imports)
- [Research]: Windows GMP DLL bundling strategy needs validation during Phase 10 planning

## Session Continuity

Last session: 2026-02-14
Stopped at: Roadmap created for v1.1 -- ready to plan Phase 9
Resume file: .planning/ROADMAP.md
