---
phase: 05-python-api
plan: 03
subsystem: api
tags: [pyo3, python, q-series, formal-power-series, dsl, relation-discovery]

# Dependency graph
requires:
  - phase: 05-02
    provides: "QSession with SessionInner.get_or_create_symbol_id(), QExpr, convert helpers"
  - phase: 03-core-qseries-partitions
    provides: "qsym_core::qseries module (pochhammer, products, theta, partitions, rank_crank)"
  - phase: 04-series-analysis
    provides: "factoring, utilities, prodmake, linalg, relations modules"
provides:
  - "QSeries pyclass wrapping FormalPowerSeries with coefficient access and arithmetic"
  - "30+ DSL functions exposing all Phase 3-4 q-series functions to Python"
  - "Complete Python __init__.py with all exports"
affects: [05-04, 06-maple-parity, 07-identity-proving]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "QSeries owns FPS directly (not behind session Mutex) since FPS is standalone computation result"
    - "DSL functions lock session, get SymbolId via get_or_create_symbol_id, compute FPS, return QSeries"
    - "Complex Rust return types (QFactorization, InfiniteProductForm, etc.) converted to Python dicts"
    - "Vec<PyRef<QSeries>> pattern for accepting lists of series from Python"
    - "qrat_to_python for Fraction conversion, qint_to_python for integer conversion"

key-files:
  created:
    - "crates/qsym-python/src/series.rs"
    - "crates/qsym-python/src/dsl.rs"
  modified:
    - "crates/qsym-python/src/lib.rs"
    - "crates/qsym-python/python/qsymbolic/__init__.py"

key-decisions:
  - "QSeries owns FPS directly rather than behind Arc<Mutex> -- FPS is a standalone computation result, not an arena expression"
  - "partition_count extracts QRat numerator as QInt for Python int conversion (not Fraction)"
  - "sift DSL function named sift_fn in Rust, registered as 'sift' in Python via pyo3(name)"
  - "extract_fps_refs helper with explicit lifetime annotations for borrowing from PyRef slice"

patterns-established:
  - "DSL function pattern: lock session -> get_or_create_symbol_id -> call qsym_core -> wrap in QSeries"
  - "Dict conversion pattern: PyDict::new + set_item for complex Rust return types"
  - "Vec<PyRef<QSeries>> for accepting Python lists of series objects"

# Metrics
duration: 7min
completed: 2026-02-14
---

# Phase 5 Plan 3: QSeries and DSL Functions Summary

**QSeries pyclass wrapping FormalPowerSeries with 30+ DSL functions covering all Phase 3-4 q-series operations accessible from Python**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-14T02:25:23Z
- **Completed:** 2026-02-14T02:32:07Z
- **Tasks:** 4
- **Files modified:** 4

## Accomplishments
- QSeries pyclass with __getitem__ (returns Fraction), __repr__, __len__, arithmetic (+, *, -, unary -), invert, sift, coeffs, to_dict, degree, low_degree
- 10 DSL functions for Groups 1-3: aqprod, qbin, etaq, jacprod, tripleprod, quinprod, winquist, theta2, theta3, theta4
- 16 DSL functions for Groups 4-5: partition_count, partition_gf, distinct_parts_gf, odd_parts_gf, bounded_parts_gf, rank_gf, crank_gf, qfactor, sift, qdegree, lqdegree, prodmake, etamake, jacprodmake, mprodmake, qetamake
- 12 DSL functions for Groups 6-7: findlincombo, findhom, findpoly, findcong, findnonhom, findhomcombo, findnonhomcombo, findlincombomodp, findhommodp, findhomcombomodp, findmaxind, findprod
- Python __init__.py exports QSeries + all 38 DSL functions

## Task Commits

Each task was committed atomically:

1. **Task 1: QSeries pyclass wrapping FormalPowerSeries** - `1ee917f` (feat)
2. **Task 2: DSL functions for Groups 1-3 (Pochhammer, products, theta)** - `ccdb1d3` (feat)
3. **Task 3: DSL functions for Groups 4-5 (partitions, factoring, prodmake)** - `12abaed` (feat)
4. **Task 4: DSL functions for Groups 6-7 (relation discovery) + Python exports** - `2258076` (feat)

## Files Created/Modified
- `crates/qsym-python/src/series.rs` - QSeries pyclass wrapping FormalPowerSeries (153 lines)
- `crates/qsym-python/src/dsl.rs` - All 38 DSL functions across 7 groups (648 lines)
- `crates/qsym-python/src/lib.rs` - Module registration for series, dsl, and all pyfunction wrappers (81 lines)
- `crates/qsym-python/python/qsymbolic/__init__.py` - Python re-exports for QSeries + all DSL functions (82 lines)

## Decisions Made
- QSeries owns FPS directly (not Arc<Mutex>) since FPS is a standalone computation result, not a session-owned arena expression
- partition_count extracts the numerator from QRat as QInt for Python int conversion, since partition counts are always integers
- sift DSL function uses `#[pyo3(name = "sift")]` on `sift_fn` to avoid Rust name collision with the QSeries method
- extract_fps_refs helper uses explicit `'a` lifetime to properly borrow from `&'a [PyRef<'a, QSeries>]`

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All Phase 3-4 Rust functions are now accessible from Python
- Ready for Plan 05-04 (integration tests and Python-level verification)
- QSeries + DSL functions provide the complete computation interface for researchers

## Self-Check: PASSED

All 5 files verified present. All 4 task commits verified in git log.

---
*Phase: 05-python-api*
*Completed: 2026-02-14*
