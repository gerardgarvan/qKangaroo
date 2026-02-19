---
phase: 32-pdf-reference-manual
plan: 01
subsystem: docs
tags: [typst, pdf, manual, reference, q-series]

# Dependency graph
requires:
  - phase: 31-error-hardening
    provides: exit codes, error messages, script engine
provides:
  - "Typst project infrastructure (main.typ, template.typ)"
  - "func-entry template for all 81 function reference entries"
  - "repl/repl-block helpers for REPL transcript examples"
  - "Introductory chapters 01-04 (Quick Start, Installation, CLI Usage, Expression Language)"
  - "Stub files for chapters 05-15 (ready for Wave 2 plans)"
affects: [32-02, 32-03, 32-04, 32-05, 32-06]

# Tech tracking
tech-stack:
  added: [typst, in-dexter]
  patterns: [multi-file typst, func-entry template, repl transcript blocks]

key-files:
  created:
    - manual/main.typ
    - manual/template.typ
    - manual/chapters/00-title.typ
    - manual/chapters/01-quick-start.typ
    - manual/chapters/02-installation.typ
    - manual/chapters/03-cli-usage.typ
    - manual/chapters/04-expression-language.typ
    - manual/chapters/05-products.typ (stub)
    - manual/chapters/06-partitions.typ (stub)
    - manual/chapters/07-theta.typ (stub)
    - manual/chapters/08-series-analysis.typ (stub)
    - manual/chapters/09-relations.typ (stub)
    - manual/chapters/10-hypergeometric.typ (stub)
    - manual/chapters/11-mock-theta-bailey.typ (stub)
    - manual/chapters/12-identity-proving.typ (stub)
    - manual/chapters/13-worked-examples.typ (stub)
    - manual/chapters/14-maple-migration.typ (stub)
    - manual/chapters/15-appendix.typ (stub)
  modified: []

key-decisions:
  - "New Computer Modern 11pt body + DejaVu Sans Mono 9pt code fonts"
  - "in-dexter v0.7.2 for back-of-book index with index-main for function entries"
  - "func-entry template uses 3-column params table, repl pairs for examples"
  - "Chapter headings auto-pagebreak via show rule"
  - "Version variable defined in template.typ for centralized version control"

patterns-established:
  - "func-entry template: name, signature, description, math-def, params, examples, edge-cases, related"
  - "repl(input, output) for single REPL examples"
  - "repl-block(content) for multi-line REPL transcripts"
  - "index-main[name] for function index entries in func-entry"

requirements-completed: [DOC-02]

# Metrics
duration: 4min
completed: 2026-02-18
---

# Phase 32 Plan 01: Typst Manual Infrastructure & Introductory Chapters Summary

**Typst project with func-entry/repl templates, title page, and 4 introductory chapters covering Quick Start, Installation, CLI (6 flags, 7 commands, 7 exit codes), and Expression Language (operators, types, q keyword)**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-18T23:58:10Z
- **Completed:** 2026-02-19T00:02:11Z
- **Tasks:** 2
- **Files created:** 18

## Accomplishments
- Created complete Typst project infrastructure with main.typ (master document including all 15 chapters), template.typ (func-entry + repl templates), and title page
- Wrote Quick Start tutorial covering install, first expression, first script, and help system
- Wrote CLI Usage chapter documenting all 6 flags, 4 execution modes, 7 session commands, and 7 exit codes from source code
- Wrote Expression Language chapter covering all operators, types, q keyword, infinity, lists, comments, and 10 value types
- Created stub files for chapters 05-15 so main.typ includes resolve

## Task Commits

Each task was committed atomically:

1. **Task 1: Create manual/ directory structure, main.typ, and template.typ** - `88eacc3` (feat)
2. **Task 2: Write introductory chapters 01-04** - `08a3c5a` (feat)

## Files Created/Modified
- `manual/main.typ` - Master document with page setup, font config, includes for all 15 chapters
- `manual/template.typ` - func-entry template, repl/repl-block helpers, version variable
- `manual/chapters/00-title.typ` - Title page with version, subtitle, description
- `manual/chapters/01-quick-start.typ` - Quick Start tutorial (install, first expression, first script, help)
- `manual/chapters/02-installation.typ` - Installation guide (binaries, source, Python API)
- `manual/chapters/03-cli-usage.typ` - CLI flags, modes, session commands, exit codes, error messages
- `manual/chapters/04-expression-language.typ` - Operators, types, q keyword, infinity, lists, comments
- `manual/chapters/05-15*.typ` - Stub files for function reference and appendix chapters

## Decisions Made
- New Computer Modern 11pt body + DejaVu Sans Mono 9pt code -- standard academic fonts bundled with Typst
- in-dexter v0.7.2 for back-of-book index generation with sub-entries and page consolidation
- func-entry template parameters: name, signature, description, math-def, params (triples), examples (pairs), edge-cases, related
- Version variable centralized in template.typ (not main.typ) for single-point updates
- Chapter headings use pagebreak via show rule for clean chapter starts
- US Letter paper with 1in margins, justified text, heading numbering "1.1"
- No global equation numbering (per plan: most equations are display-only)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- func-entry template is ready for Wave 2 plans to write function reference chapters
- repl and repl-block helpers are ready for all REPL transcript examples
- Stub files for chapters 05-15 are in place; each Wave 2 plan just overwrites the stub
- main.typ already includes all 15 chapters, so no modification needed as content is added

## Self-Check: PASSED

All 18 created files verified present. Both task commits (88eacc3, 08a3c5a) verified in git log.

---
*Phase: 32-pdf-reference-manual*
*Completed: 2026-02-18*
