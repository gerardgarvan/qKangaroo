---
phase: 23-verification-cross-references
plan: 01
subsystem: testing
tags: [installation, verification, check-script, python]

# Dependency graph
requires:
  - phase: 10-pypi-packaging
    provides: "q_kangaroo package with partition_count, QSession, etaq"
provides:
  - "check_install.py: single-script installation verification with end-user and --dev modes"
affects: [documentation, CI]

# Tech tracking
tech-stack:
  added: []
  patterns: ["colored pass/fail output with ANSI fallback", "argparse --dev mode for developer checks"]

key-files:
  created:
    - check_install.py
  modified:
    - crates/qsym-python/python/q_kangaroo/__init__.py

key-decisions:
  - "Used standard library only (no third-party dependencies for the check script)"
  - "ANSI color with auto-detection and NO_COLOR support"

patterns-established:
  - "check_install.py pattern: end-user checks always run, --dev adds build prerequisite checks"

requirements-completed: [VRFY-01, VRFY-02]

# Metrics
duration: 5min
completed: 2026-02-17
---

# Phase 23 Plan 01: Installation Verification Script Summary

**check_install.py with 4 end-user checks (Python version, import, GMP, computation) and 5 --dev checks (Rust, Cargo, Maturin, GMP headers, C compiler)**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-17T14:19:09Z
- **Completed:** 2026-02-17T14:24:21Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Created check_install.py with colored PASS/FAIL output for 4 end-user checks
- Added --dev mode with 5 additional build-from-source prerequisite checks
- Fixed __init__.py import name mismatch with pyo3-exported function names

## Task Commits

Each task was committed atomically:

1. **Task 1: Create check_install.py with end-user verification checks** - `9e2cf8a` (feat)
2. **Task 2: Add --dev flag with build-from-source prerequisite checks** - included in Task 1 commit (complete script written in one pass)

## Files Created/Modified
- `check_install.py` - Installation verification script (238 lines) with end-user and --dev modes
- `crates/qsym-python/python/q_kangaroo/__init__.py` - Fixed import names to match pyo3 exports

## Decisions Made
- Used standard library only (sys, os, subprocess, argparse, platform) -- no pip install needed to run
- ANSI color auto-detection with NO_COLOR env var support
- GMP header search checks C_INCLUDE_PATH, then platform-specific common paths

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed __init__.py import names mismatching pyo3 exports**
- **Found during:** Task 1 (running check_install.py)
- **Issue:** __init__.py imported `q_zeilberger_fn`, `verify_wz_fn`, `q_petkovsek_fn`, `prove_nonterminating_fn`, `find_transformation_chain_fn` from `_q_kangaroo`, but the Rust code uses `#[pyo3(name = "...")]` to export them without the `_fn` suffix (e.g., `q_zeilberger` not `q_zeilberger_fn`)
- **Fix:** Changed imports to use the actual pyo3-exported names: `q_zeilberger`, `verify_wz`, `q_petkovsek`, `prove_nonterminating`, `find_transformation_chain`
- **Files modified:** crates/qsym-python/python/q_kangaroo/__init__.py
- **Verification:** `from q_kangaroo import partition_count, QSession, etaq` succeeds; all check_install.py checks pass
- **Committed in:** 9e2cf8a (Task 1 commit)

**2. [Rule 3 - Blocking] Rebuilt stale .pyd native extension**
- **Found during:** Task 1 (running check_install.py)
- **Issue:** The compiled _q_kangaroo.pyd was stale, missing newer function exports
- **Fix:** Ran `maturin develop` to rebuild the native extension
- **Files modified:** (build artifact only, not committed)
- **Verification:** All 83 functions now exported from .pyd
- **Committed in:** N/A (build artifact)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Bug fix was necessary for q_kangaroo to be importable at all. No scope creep.

## Issues Encountered
None beyond the deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- check_install.py ready for users and contributors
- All installation verification paths tested and working

---
*Phase: 23-verification-cross-references*
*Completed: 2026-02-17*
