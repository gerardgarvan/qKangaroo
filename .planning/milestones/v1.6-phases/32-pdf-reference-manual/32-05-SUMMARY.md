---
phase: 32-pdf-reference-manual
plan: 05
subsystem: documentation
tags: [typst, worked-examples, maple-migration, index, in-dexter, q-series]

# Dependency graph
requires:
  - phase: 32-01
    provides: manual infrastructure, template.typ with repl/func-entry helpers, main.typ
provides:
  - Worked Examples chapter with 6 complete research workflow examples
  - Maple Migration chapter with all 17 aliases and complete 81-function mapping
  - Back-of-book index chapter via in-dexter make-index
affects: [32-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Worked examples follow generate-discover-verify research loop pattern"
    - "Maple migration uses per-group sub-tables for readability"
    - "Index generation delegated to chapter file (not main.typ) to avoid duplication"

key-files:
  created:
    - manual/chapters/13-worked-examples.typ
  modified:
    - manual/chapters/14-maple-migration.typ
    - manual/chapters/15-appendix.typ
    - manual/main.typ

key-decisions:
  - "6 worked examples spanning all major function groups (products, partitions, theta, Bailey, hypergeometric, mock theta)"
  - "Moved make-index from main.typ into chapter 15 to avoid duplicate index generation"
  - "Complete function mapping tables organized by 10 groups with Extensions marked for q-Kangaroo-only functions"

patterns-established:
  - "Worked example structure: math context, citation, REPL workflow steps, takeaway"

requirements-completed: [DOC-03]

# Metrics
duration: 4min
completed: 2026-02-19
---

# Phase 32 Plan 05: Closing Chapters Summary

**6 worked examples (Euler, Ramanujan, Jacobi, Rogers-Ramanujan, Heine, mock theta) with citations, plus complete Maple migration table mapping all 81 functions, and back-of-book index**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-19T00:05:11Z
- **Completed:** 2026-02-19T00:09:27Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Worked Examples chapter with 6 extended examples covering identity verification, research workflows, Bailey chains, hypergeometric transformations, and mock theta relations
- Maple Migration chapter with all 17 resolve_alias() mappings and complete 81-function mapping organized by 10 groups
- Back-of-book index chapter with make-index generation (moved out of main.typ to avoid duplication)

## Task Commits

Each task was committed atomically:

1. **Task 1: Write Worked Examples chapter** - `bfae241` (feat)
2. **Task 2: Write Maple Migration and Index chapters** - `157d078` (feat)

## Files Created/Modified
- `manual/chapters/13-worked-examples.typ` - 6 worked examples with math context, citations, REPL transcripts, and takeaways
- `manual/chapters/14-maple-migration.typ` - 17-alias table, 81-function complete mapping, key differences section
- `manual/chapters/15-appendix.typ` - Back-of-book index via make-index(title: none)
- `manual/main.typ` - Removed duplicate make-index call (now in chapter 15)

## Decisions Made
- Included all 6 planned examples (Euler pentagonal, Ramanujan congruences, Jacobi triple product, Rogers-Ramanujan/Bailey, Heine hypergeometric, mock theta/Watson relation)
- Each example follows consistent structure: mathematical context, citation, multi-step REPL workflow, takeaway
- Migration table organized into 10 sub-tables by function group (matching help.rs grouping) rather than a single flat table
- Extension functions (no Maple equivalent) marked with "Extension" label and "---" in Maple column
- Moved make-index from main.typ into chapter 15 to keep index generation in one place

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Moved make-index from main.typ to chapter 15**
- **Found during:** Task 2
- **Issue:** main.typ already had `#make-index(title: [Index])` after including chapter 15. Adding a second make-index in chapter 15 would create a duplicate index.
- **Fix:** Put `= Index` heading and `#make-index(title: none)` in chapter 15, removed the make-index block from main.typ
- **Files modified:** manual/main.typ, manual/chapters/15-appendix.typ
- **Verification:** Only one make-index call exists across all files
- **Committed in:** 157d078 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary to prevent duplicate index generation. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 15 manual chapters are now complete (00-title through 15-index)
- Plan 32-06 (CI PDF build and --help update) can proceed independently
- Manual is ready for Typst compilation once all chapter content is finalized

## Self-Check: PASSED

- FOUND: manual/chapters/13-worked-examples.typ
- FOUND: manual/chapters/14-maple-migration.typ
- FOUND: manual/chapters/15-appendix.typ
- FOUND: manual/main.typ
- FOUND: bfae241 (Task 1 commit)
- FOUND: 157d078 (Task 2 commit)

---
*Phase: 32-pdf-reference-manual*
*Completed: 2026-02-19*
