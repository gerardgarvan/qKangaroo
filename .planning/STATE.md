# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-16)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** v1.3 Documentation & Vignettes — COMPLETE (all 4 phases shipped).

## Current Position

Phase: 21 of 21 (Sphinx Site Polish) -- COMPLETE
Plan: 2 of 2 (all plans complete)
Status: Phase 21 complete, v1.3 milestone complete
Last activity: 2026-02-17 — Completed 21-02 (API cross-links to notebooks)

Progress: [==================================================] 100% (v1.3: 4/4 phases complete)

## Performance Metrics

### v1.0 Core Engine

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

### v1.1 Polish & Publish

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 9 - Package Rename & Structure | 2/2 | 4 min | 2 min |
| 10 - PyPI Packaging & Metadata | 2/2 | 5 min | 2.5 min |
| 11 - CI/CD Pipeline | 2/2 | 2 min | 1 min |
| 12 - Documentation & UX Polish | 4/4 | 57 min | 14 min |

### v1.2 Algorithmic Identity Proving

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 13 - Polynomial Infrastructure | 3/3 | 13 min | 4 min |
| 14 - q-Gosper Algorithm | 3/3 | 19 min | 6 min |
| 15 - q-Zeilberger & WZ Certificates | 3/3 | 62 min | 21 min |
| 16 - Extensions | 3/3 | 18 min | 6 min |
| 17 - Python API & Docs | 2/2 | 12 min | 6 min |

### v1.3 Documentation & Vignettes

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 18 - Docstring Enrichment | 4/4 | 26 min | 7 min |
| 19 - Vignette Expansion | 3/3 | 20 min | 7 min |
| 20 - New Vignettes & Migration | 3/3 | 19 min | 6 min |
| 21 - Sphinx Site Polish | 2/2 | 5 min | 2.5 min |

## Accumulated Context

### Decisions

All decisions logged in PROJECT.md Key Decisions table.
v1.0 decisions preserved in MILESTONES.md.
v1.1 decisions preserved in milestones/v1.1-ROADMAP.md.
v1.2 decisions preserved in milestones/v1.2-ROADMAP.md.
- 20-03: Listed all 9 notebooks in index.rst (series_analysis/identity_proving to be created by 20-01/20-02)
- 20-03: Corrected API signatures from plan draft to match actual dsl.rs exports
- 20-02: q-Vandermonde as running example for identity proving notebook (first-order recurrence, clean pipeline)
- 20-02: Fixed 3 plan parameter bugs: non-summable example, nonterminating RHS bases, transformation chain pair
- 20-01: Used (q;q)_3 not (1;q)_3 for first aqprod demo to avoid zero from (1-1) factor
- 21-02: Absolute :doc: paths (/examples/name) for cross-links, seealso after all autofunction directives
- 21-01: Tip/note/seealso admonitions for audience paths; function guide organized by task type not implementation group
- 21-01: Sears/Watson transforms noted as internal to try_summation/find_transformation_chain (not separate Python DSL exports)
- 20-01: Combined findhom/findpoly into single section with findpoly live demo

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-17
Stopped at: Completed 21-01-PLAN.md (landing page, function guide, examples gallery). Both phase 21 plans complete.
Resume file: .planning/phases/21-sphinx-site-polish/21-01-SUMMARY.md
