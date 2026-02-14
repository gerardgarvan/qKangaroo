---
phase: 05-python-api
plan: 01
subsystem: api
tags: [pyo3, maturin, cdylib, gmp, python, ffi]

# Dependency graph
requires:
  - phase: 01-expression-foundation
    provides: ExprArena, ExprRef, rug/GMP infrastructure
provides:
  - "cdylib crate (qsym-python) compiling against qsym-core with GMP"
  - "maturin build pipeline producing installable Python wheel"
  - "Minimal _qsymbolic native module importable from Python"
affects: [05-02, 05-03, 05-04]

# Tech tracking
tech-stack:
  added: [pyo3 0.23.5, maturin 1.11.5, Python 3.14.2]
  patterns: [cdylib crate with PyO3, maturin mixed python/rust layout, ABI3 forward compatibility]

key-files:
  created:
    - crates/qsym-python/Cargo.toml
    - crates/qsym-python/pyproject.toml
    - crates/qsym-python/src/lib.rs
    - crates/qsym-python/python/qsymbolic/__init__.py
  modified:
    - Cargo.toml
    - .gitignore

key-decisions:
  - "PyO3 0.23 with PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 for Python 3.14 support"
  - "maturin mixed layout: native module _qsymbolic, Python package qsymbolic/"
  - "version() function exercises GMP via ExprArena create/drop to prove linkage"

patterns-established:
  - "cdylib build with use-system-libs GMP on Windows/Cygwin via MinGW"
  - "maturin develop --release in virtualenv for local development"

# Metrics
duration: 4min
completed: 2026-02-14
---

# Phase 5 Plan 1: Python Crate Scaffold Summary

**PyO3 cdylib crate scaffolded with GMP linkage validated end-to-end: Rust compile, maturin wheel build, and Python 3.14 import all succeed**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-14T02:09:31Z
- **Completed:** 2026-02-14T02:14:02Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Scaffolded crates/qsym-python/ with cdylib crate depending on qsym-core (and transitively on rug/GMP)
- Validated the highest-risk item: GMP+cdylib+PyO3+Windows/Cygwin toolchain compiles and links successfully
- Full end-to-end pipeline proven: Rust cdylib -> maturin wheel -> Python 3.14 import -> GMP runtime exercise
- Established maturin mixed layout pattern for future binding development

## Task Commits

Each task was committed atomically:

1. **Task 1: Scaffold qsym-python crate** - `99bec1a` (feat)
2. **Task 2: Validate end-to-end Python import** - `1a1882d` (chore)

## Files Created/Modified
- `crates/qsym-python/Cargo.toml` - cdylib crate config with pyo3 0.23 + qsym-core dependency
- `crates/qsym-python/pyproject.toml` - maturin build config with mixed python/rust layout
- `crates/qsym-python/src/lib.rs` - Minimal #[pymodule] with version() exercising GMP
- `crates/qsym-python/python/qsymbolic/__init__.py` - Pure Python re-export layer
- `Cargo.toml` - Workspace updated to include qsym-python member
- `Cargo.lock` - Lockfile updated with pyo3 dependencies
- `.gitignore` - Added .venv, __pycache__, *.pyc, *.pyd patterns

## Decisions Made
- **PyO3 0.23 + ABI3 forward compatibility:** Python 3.14.2 is installed but PyO3 0.23 only supports up to 3.13. Setting PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 resolves this. The alternative was upgrading to PyO3 0.28 but 0.23 was chosen per research recommendation for stability.
- **maturin mixed layout:** Native module named `_qsymbolic` (underscore prefix), Python package `qsymbolic/` imports from it. This avoids the name collision pitfall documented in research.
- **use-system-libs GMP works for cdylib:** The research flagged this as the highest-risk unknown. Confirmed that the existing system-libs GMP (via MinGW packages) links correctly into the cdylib shared library. No need to switch to bundled GMP build.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] PyO3 0.23 incompatible with Python 3.14**
- **Found during:** Task 1 (cargo check)
- **Issue:** PyO3 0.23.5 maximum supported Python version is 3.13, but system has Python 3.14.2
- **Fix:** Set PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 environment variable as suggested by PyO3 error message
- **Files modified:** None (environment variable, not source change)
- **Verification:** cargo check passes, maturin develop succeeds, Python import works
- **Committed in:** 99bec1a (part of Task 1)

**2. [Rule 2 - Missing Critical] Added Python artifacts to .gitignore**
- **Found during:** Task 2 (maturin develop creates .venv and .pyc files)
- **Issue:** .venv/, __pycache__/, *.pyc, *.pyd not in .gitignore
- **Fix:** Added Python-specific patterns to .gitignore
- **Files modified:** .gitignore
- **Verification:** git status shows clean after build artifacts created
- **Committed in:** 1a1882d (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical)
**Impact on plan:** Both fixes necessary for correct operation. No scope creep.

## Issues Encountered
None - both tasks succeeded on first attempt after the PyO3 version compatibility fix.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- cdylib build pipeline fully validated, ready for QSession/QExpr binding development (Plan 02)
- Build requires: `export PATH="/c/mingw64-gcc/mingw64/bin:..."` and `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1`
- maturin develop in virtualenv at crates/qsym-python/.venv

## Self-Check: PASSED

All 6 key files verified on disk. Both task commits (99bec1a, 1a1882d) verified in git log.

---
*Phase: 05-python-api*
*Completed: 2026-02-14*
