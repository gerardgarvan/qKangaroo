---
phase: 05-python-api
plan: 02
subsystem: api
tags: [pyo3, python, qsession, qexpr, operators, rendering, arc-mutex, gc-safety]

# Dependency graph
requires:
  - phase: 05-python-api
    plan: 01
    provides: cdylib crate scaffold, maturin build pipeline, PyO3 configuration
  - phase: 01-expression-foundation
    provides: ExprArena, ExprRef, Expr, canonical, render, simplify
provides:
  - "QSession pyclass: frozen+Mutex arena owner with symbol/integer/rational/infinity creation"
  - "QExpr pyclass: expression handles with +, *, -, ** operators and Unicode/LaTeX rendering"
  - "SessionInner.get_or_create_symbol_id() helper for DSL functions"
  - "convert.rs: rug QInt/QRat to Python int/Fraction conversion utilities"
  - "symbols() convenience function for SymPy-like Python interface"
affects: [05-03, 05-04]

# Tech tracking
tech-stack:
  added: []
  patterns: [frozen+Mutex pyclass, Arc back-reference for GC safety, DLL directory auto-registration]

key-files:
  created:
    - crates/qsym-python/src/session.rs
    - crates/qsym-python/src/expr.rs
    - crates/qsym-python/src/convert.rs
  modified:
    - crates/qsym-python/src/lib.rs
    - crates/qsym-python/python/qsymbolic/__init__.py

key-decisions:
  - "Used intern_rat() instead of direct rug::Rational to avoid rug dependency in qsym-python"
  - "Used std Hash trait on ExprRef for __hash__ since ExprRef.0 is pub(crate)"
  - "Auto-register MinGW DLL directory in __init__.py for Windows GMP shared library loading"

patterns-established:
  - "frozen+Mutex pattern: QSession is #[pyclass(frozen)] with Arc<Mutex<SessionInner>> for thread safety"
  - "Arc back-reference: QExpr holds Arc<Mutex<SessionInner>> keeping session alive across GC"
  - "No Drop/dealloc on QExpr that locks session: prevents deadlock with Python GC"

# Metrics
duration: 5min
completed: 2026-02-14
---

# Phase 5 Plan 2: Core Python API Summary

**QSession + QExpr API with operator overloads (+, *, -, **), Unicode/LaTeX rendering, and GC-safe Arc reference counting validated with 10k expression stress test**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-14T02:17:46Z
- **Completed:** 2026-02-14T02:22:27Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- QSession with frozen+Mutex pattern creates symbols, integers, rationals, infinity expressions
- QExpr supports all Python operators (+, *, -, **) producing canonical expressions via qsym_core
- __repr__ returns Unicode, _repr_latex_ returns $LaTeX$ for Jupyter, latex() returns raw LaTeX
- GC stress test: 10,000 expressions created/discarded without deadlock; session survives via Arc when QExpr outlives QSession
- SessionInner.get_or_create_symbol_id() ready for DSL functions in Plan 05-03
- symbols() convenience function provides SymPy-like interface

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement QSession, QExpr, and type conversions** - `047a5bb` (feat)
2. **Task 2: Update Python package and add GC stress test** - `4e38170` (feat)

## Files Created/Modified
- `crates/qsym-python/src/session.rs` - QSession pyclass with frozen+Mutex pattern, SessionInner with get_or_create_symbol_id()
- `crates/qsym-python/src/expr.rs` - QExpr pyclass with operator overloads, rendering, simplify, equality/hash
- `crates/qsym-python/src/convert.rs` - rug QInt/QRat to Python int/Fraction conversion via string
- `crates/qsym-python/src/lib.rs` - Module registration for session, expr, convert modules
- `crates/qsym-python/python/qsymbolic/__init__.py` - Re-exports, symbols() convenience function, DLL directory fix

## Decisions Made
- **intern_rat instead of rug::Rational:** qsym-python does not depend on rug directly; used ExprArena.intern_rat(num, den) which takes impl Into<rug::Integer>. Avoids adding rug as a direct dependency.
- **Hash trait for __hash__:** ExprRef.0 is pub(crate), so used std::hash::Hash + DefaultHasher on ExprRef rather than accessing the inner u32 directly.
- **DLL directory auto-registration:** Python 3.8+ on Windows does not search PATH for DLL dependencies of native extensions. Added os.add_dll_directory() call in __init__.py to find MinGW GMP shared library. Configurable via MINGW_BIN environment variable.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] rug crate not a direct dependency of qsym-python**
- **Found during:** Task 1 (cargo check)
- **Issue:** session.rs used rug::Rational::from() and rug::Integer::from() directly, but rug is only a transitive dependency via qsym-core
- **Fix:** Replaced with arena.intern_rat(num, den) which accepts i64 via Into<rug::Integer>
- **Files modified:** crates/qsym-python/src/session.rs
- **Verification:** cargo check passes
- **Committed in:** 047a5bb (Task 1)

**2. [Rule 3 - Blocking] ExprRef.0 is pub(crate), not accessible from qsym-python**
- **Found during:** Task 1 (cargo check)
- **Issue:** __hash__ tried to access self.expr_ref.0 which is pub(crate) in qsym-core
- **Fix:** Used std::hash::Hash trait on ExprRef with DefaultHasher instead
- **Files modified:** crates/qsym-python/src/expr.rs
- **Verification:** cargo check passes, hash equality verified in Python
- **Committed in:** 047a5bb (Task 1)

**3. [Rule 3 - Blocking] Windows DLL loading for GMP shared library**
- **Found during:** Task 2 (Python import)
- **Issue:** Python 3.8+ on Windows requires os.add_dll_directory() for native extension DLL dependencies; import failed with "DLL load failed"
- **Fix:** Added DLL directory auto-registration in __init__.py with MINGW_BIN env var override
- **Files modified:** crates/qsym-python/python/qsymbolic/__init__.py
- **Verification:** Import works without manual PATH setup
- **Committed in:** 4e38170 (Task 2)

---

**Total deviations:** 3 auto-fixed (3 blocking)
**Impact on plan:** All fixes necessary for compilation and runtime loading. No scope creep.

## Issues Encountered
None beyond the auto-fixed blocking issues above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Core QSession + QExpr API complete, ready for DSL function bindings (Plan 05-03)
- SessionInner.get_or_create_symbol_id() helper available for series variable parameter
- convert.rs utilities available for future result-to-Python conversions
- Build requires: `PYO3_PYTHON` pointing to virtualenv Python, `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1`

## Self-Check: PASSED

All 6 key files verified on disk. Both task commits (047a5bb, 4e38170) verified in git log.

---
*Phase: 05-python-api*
*Completed: 2026-02-14*
