---
phase: 20-new-vignettes-migration
plan: 01
subsystem: docs
tags: [jupyter, notebooks, tutorial, getting-started, series-analysis]

requires:
  - phase: 12-documentation-ux-polish
    provides: "Sphinx docs infrastructure and existing notebook examples"
  - phase: 19-vignette-expansion
    provides: "Expanded partition_congruences, theta_identities, hypergeometric notebooks"
provides:
  - "getting_started.ipynb newcomer onboarding tutorial"
  - "series_analysis.ipynb research workflow tutorial"
affects: [20-new-vignettes-migration]

tech-stack:
  added: []
  patterns: ["Pre-computed output notebooks with nbformat 4 structure"]

key-files:
  created:
    - docs/examples/getting_started.ipynb
    - docs/examples/series_analysis.ipynb
  modified: []

key-decisions:
  - "Used (q;q)_3 not (1;q)_3 for first aqprod demo to avoid zero from (1-1) factor"
  - "Combined findhom and findpoly into single section with findpoly live demo and findhom described"
  - "Used pgf50 for sift demos in series_analysis to keep outputs manageable"

patterns-established:
  - "Tutorial notebooks follow import -> compute -> explain -> verify progression"

requirements-completed: [DOC-09, DOC-10]

duration: 9min
completed: 2026-02-17
---

# Phase 20 Plan 01: New Vignettes -- Getting Started and Series Analysis

**Two new tutorial notebooks: getting_started.ipynb (22 cells, zero-to-identity newcomer path) and series_analysis.ipynb (32 cells, prodmake/etamake/sift/relation discovery pipeline)**

## Performance

- **Duration:** 9 min
- **Started:** 2026-02-17T02:11:20Z
- **Completed:** 2026-02-17T02:20:52Z
- **Tasks:** 2
- **Files created:** 2

## Accomplishments
- Created getting_started.ipynb: 22-cell newcomer tutorial covering QSession, aqprod, partition_gf, partition_count, etaq, jacprod, findcong, and prove_eta_id
- Created series_analysis.ipynb: 32-cell research workflow tutorial covering prodmake, etamake, jacprodmake, mprodmake, sift, qfactor, findlincombo, findpoly, and findcong
- All code cells have pre-computed outputs verified against existing notebook data

## Task Commits

Each task was committed atomically:

1. **Task 1: Create getting_started.ipynb** - `8732ea2` (feat)
2. **Task 2: Create series_analysis.ipynb** - `bfba96f` (feat)

## Files Created/Modified
- `docs/examples/getting_started.ipynb` - Newcomer onboarding tutorial from installation to first identity verification
- `docs/examples/series_analysis.ipynb` - Series analysis workflow demonstrating the full prodmake/etamake/sift/relation discovery pipeline

## Decisions Made
- Used `aqprod(s, 1, 1, 1, 3, 10)` for first demo (computing $(q;q)_3$) instead of `aqprod(s, 1, 1, 0, 3, 10)` which would give $(1;q)_3 = 0$ due to the $(1-1)$ factor at $k=0$
- Combined findhom and findpoly into a single section, with findpoly getting the live demo (Euler's theorem as $P(x,y) = x-y = 0$) and findhom described in the text
- Used pgf50 (not pgf100) for sift demos in series_analysis to keep output lengths manageable

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed aqprod parameters in getting_started.ipynb**
- **Found during:** Task 1 (getting_started.ipynb creation)
- **Issue:** Plan specified `aqprod(s, 1, 1, 0, 3, 10)` to compute $(1;q)_3$, but with $a = q^0 = 1$, $(1;q)_3 = (1-1)(1-q)(1-q^2) = 0$
- **Fix:** Changed to `aqprod(s, 1, 1, 1, 3, 10)` which computes $(q;q)_3 = (1-q)(1-q^2)(1-q^3)$ correctly
- **Files modified:** docs/examples/getting_started.ipynb
- **Verification:** Output matches hand computation: $1 - q - q^2 + q^4 + q^5 - q^6$
- **Committed in:** 8732ea2 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Bug fix essential for correctness. No scope creep.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Two new tutorial notebooks ready for Sphinx integration
- Complements existing 5 notebooks (partition_congruences, theta_identities, hypergeometric_summation, mock_theta_functions, bailey_chains)
- Ready for phase 20-02 and 20-03

## Self-Check: PASSED

- [x] docs/examples/getting_started.ipynb exists
- [x] docs/examples/series_analysis.ipynb exists
- [x] Commit 8732ea2 exists (Task 1)
- [x] Commit bfba96f exists (Task 2)

---
*Phase: 20-new-vignettes-migration*
*Completed: 2026-02-17*
