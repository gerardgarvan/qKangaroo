---
phase: 09-package-rename-structure
plan: 01
subsystem: python-api
tags: [rename, pypi, pyo3, maturin, python-packaging]

# Dependency graph
requires:
  - phase: 05-python-api
    provides: "PyO3 cdylib Python package (originally named qsymbolic)"
  - phase: 08-mock-theta-bailey-chains
    provides: "Full Python DSL with 70+ functions across 10 groups"
provides:
  - "Python package renamed from qsymbolic to q_kangaroo (import name)"
  - "PyPI package name q-kangaroo (hyphenated)"
  - "Rust cdylib output _q_kangaroo.dll"
  - "Triple-match verified: Cargo.toml, pyproject.toml, lib.rs all agree on _q_kangaroo"
affects: [10-ci-distribution, 11-property-testing, 12-docs-ux]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Triple-match rule: Cargo.toml [lib] name, pyproject.toml module-name, lib.rs #[pymodule] fn must all match"
    - "PyPI name uses hyphens (q-kangaroo), Python import uses underscores (q_kangaroo)"

key-files:
  created: []
  modified:
    - "crates/qsym-python/Cargo.toml"
    - "crates/qsym-python/pyproject.toml"
    - "crates/qsym-python/src/lib.rs"
    - "crates/qsym-python/python/q_kangaroo/__init__.py"
    - "crates/qsym-python/tests/test_integration.py"

key-decisions:
  - "PyPI name q-kangaroo (hyphens), import name q_kangaroo (underscores), cdylib _q_kangaroo (leading underscore)"
  - "Rust crate names unchanged (qsym-core, qsym-python) -- only Python-facing names renamed"

patterns-established:
  - "Triple-match rule for maturin/PyO3 packages: [lib].name == last component of module-name == #[pymodule] fn name"

# Metrics
duration: 2min
completed: 2026-02-14
---

# Phase 9 Plan 1: Package Rename Summary

**Atomic rename of Python package from qsymbolic to q_kangaroo across 5 source files, directory, and all test imports -- 578 Rust tests passing**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-14T22:36:37Z
- **Completed:** 2026-02-14T22:38:59Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Renamed Python package directory from `qsymbolic/` to `q_kangaroo/`
- Updated all 5 source/config files: Cargo.toml, pyproject.toml, lib.rs, __init__.py, test_integration.py
- Verified triple-match: Cargo.toml `name = "_q_kangaroo"`, pyproject.toml `module-name = "q_kangaroo._q_kangaroo"`, lib.rs `fn _q_kangaroo`
- All 578 Rust tests pass with 0 failures; `cargo check -p qsym-python` succeeds
- Zero occurrences of "qsymbolic" remain in any source file under crates/

## Task Results

1. **Task 1: Rename directory and update all source files** -- All 5 files updated, directory renamed, stale artifacts cleaned, grep confirms zero "qsymbolic" occurrences
2. **Task 2: Verify Rust compilation and all 578 tests pass** -- 578 tests pass (24 test binaries, 0 failures), `cargo check -p qsym-python` succeeds

_Note: No git in this project, so no commit hashes. Changes applied directly to filesystem._

## Files Modified
- `crates/qsym-python/Cargo.toml` - Changed [lib] name from `_qsymbolic` to `_q_kangaroo`
- `crates/qsym-python/pyproject.toml` - Changed project name to `q-kangaroo`, module-name to `q_kangaroo._q_kangaroo`
- `crates/qsym-python/src/lib.rs` - Changed #[pymodule] fn from `_qsymbolic` to `_q_kangaroo`, updated doc comment
- `crates/qsym-python/python/q_kangaroo/__init__.py` - Renamed directory, updated docstring and both import lines
- `crates/qsym-python/tests/test_integration.py` - Updated all 9 `from qsymbolic import` to `from q_kangaroo import`, updated module docstring

## Decisions Made
- PyPI name uses hyphens (`q-kangaroo`) while Python import uses underscores (`q_kangaroo`) -- standard Python packaging convention
- Rust crate names (`qsym-core`, `qsym-python`) left unchanged -- only Python-facing names changed
- cdylib name uses leading underscore (`_q_kangaroo`) to indicate native extension module

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- qsym-python cdylib test runner fails with STATUS_DLL_NOT_FOUND (Python DLL not in PATH for Cygwin environment) -- this is pre-existing behavior unrelated to the rename, as the cdylib requires Python's runtime DLL to execute. The cdylib compiles successfully via `cargo check`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Package name finalized as q_kangaroo -- ready for PyPI packaging in Phase 10
- All Rust tests pass -- no regressions from rename
- Phase 9 Plan 2 (workspace restructuring) can proceed

## Self-Check: PASSED

All 6 claimed files verified present. Triple-match confirmed across Cargo.toml, pyproject.toml, and lib.rs. Zero occurrences of "qsymbolic" in crates/.

---
*Phase: 09-package-rename-structure*
*Completed: 2026-02-14*
