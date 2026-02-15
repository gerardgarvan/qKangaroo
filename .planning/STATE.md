# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-14)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** Phase 10 -- CI & Distribution

## Current Position

Phase: 10 of 12 (CI & Distribution)
Plan: 1 of 2 in current phase
Status: 10-01 complete, ready for 10-02
Last activity: 2026-02-15 -- Completed 10-01 (PyPI packaging metadata, LICENSE, CITATION.cff)

Progress: [###################################.........] 80% (35/44 plans -- 32 v1.0 + 3 v1.1 complete, 9 v1.1 remaining)

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

## v1.1 Performance Metrics

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 9 - Package Rename & Structure | 2/2 | 4 min | 2 min |
| 10 - PyPI Packaging & Metadata | 1/2 | 1 min | 1 min |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
v1.0 decisions preserved in MILESTONES.md.

Key v1.1 decisions:
- [v1.1]: Package name q-kangaroo (PyPI) / q_kangaroo (import) -- user-chosen
- [v1.1]: GitHub Pages for documentation hosting
- [v1.1]: CI targets Linux + Windows (macOS deferred)
- [v1.1]: DOC + UX merged into single Phase 12 (shared delivery boundary)
- [09-01]: Rust crate names unchanged (qsym-core, qsym-python) -- only Python-facing names renamed
- [09-01]: cdylib uses leading underscore (_q_kangaroo) per maturin/PyO3 convention
- [09-02]: Old qsymbolic package uninstalled before rebuild for clean verification
- [09-02]: PROJECT.md updated with 3 q_kangaroo references (architecture diagram, API example, directory tree)
- [10-01]: ABI3 feature via maturin features in pyproject.toml (not Cargo.toml) to avoid feature conflicts
- [10-01]: DLL loading prefers bundled package directory, falls back to MINGW_BIN env var then hardcoded path
- [10-01]: Placeholder author/owner fields -- user fills before publish

### Pending Todos

None yet.

### Blockers/Concerns

- [Build]: Windows build requires MinGW GCC 14.2.0 + pre-built GMP in PATH
- [Build]: PyO3 builds require PYO3_PYTHON pointing to Python 3.14 and PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
- [Rename]: RESOLVED -- qsymbolic -> q_kangaroo rename completed in 09-01, fully verified in 09-02 (9 Python tests pass)
- [DLL Bundling]: RESOLVED -- DLL include config and package-relative loading implemented in 10-01

## Session Continuity

Last session: 2026-02-15
Stopped at: Completed 10-01-PLAN.md (PyPI packaging metadata) -- ready for 10-02 (wheel build)
Resume file: .planning/phases/10-pypi-packaging-metadata/10-01-SUMMARY.md
