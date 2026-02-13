---
phase: 01-expression-foundation
plan: 03
subsystem: rendering
tags: [rust, latex, unicode, rendering, dlmf, q-series, snapshot-tests]

# Dependency graph
requires:
  - phase: 01-01
    provides: Expr enum (13 variants), ExprArena, SymbolRegistry, canonical constructors
affects:
  - 02-simplification (expression rendering aids debugging rewrite rules)
  - 03-core-qseries (q-Pochhammer and hypergeometric display for researcher output)
  - 05-python-api (Python display/repr delegates to these renderers)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Recursive arena traversal with context-dependent parenthesization
    - Always-brace policy for LaTeX sub/superscripts (q^{2} not q^2)
    - DLMF 17.2 matrix notation for basic hypergeometric series
    - Unicode superscript/subscript digit mapping with ASCII fallback
    - Greek letter lookup tables for both LaTeX commands and Unicode chars

key-files:
  created:
    - crates/qsym-core/tests/render_tests.rs
  modified:
    - crates/qsym-core/src/render/mod.rs
    - crates/qsym-core/src/render/latex.rs
    - crates/qsym-core/src/render/unicode.rs

key-decisions:
  - "LaTeX renderer uses always-brace policy for all subscripts/superscripts to eliminate edge-case bugs"
  - "Unicode renderer falls back to ASCII (^, _) for non-numeric sub/superscripts instead of partial Unicode letter subscripts"
  - "Neg detection in Add rendering: Neg children render as ' - x' instead of ' + -x' in both backends"
  - "BasicHypergeometric uses full DLMF 17.2 matrix notation in LaTeX, simplified subscript notation in Unicode"

patterns-established:
  - "to_latex(arena, expr) is the public LaTeX API; arena.display(expr) is the public Unicode API"
  - "ParenContext/Ctx enum controls parenthesization -- PowBase wraps Add/Mul, MulFactor wraps Add"
  - "Greek letter tables are duplicated in each backend (LaTeX commands vs Unicode codepoints) for independence"

# Metrics
duration: 6min
completed: 2026-02-13
---

# Phase 1 Plan 03: LaTeX and Unicode Rendering Summary

**DLMF 17.2 LaTeX rendering and Unicode terminal display for all 13 Expr variants, with 67 snapshot tests covering every variant plus nesting/edge cases**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-13T21:39:42Z
- **Completed:** 2026-02-13T21:46:13Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Unicode renderer (`DisplayExpr`) implements `fmt::Display` for all 13 Expr variants with Greek chars, superscript/subscript digits
- LaTeX renderer (`to_latex`) was already implemented (from prior execution); verified correct for all variants
- 67 snapshot tests: 32 LaTeX + 34 Unicode + 1 cross-backend consistency check
- Every Expr variant has at least one LaTeX test and one Unicode test
- Edge cases tested: nested Pow, compound bases, multi-digit exponents (12, -12), symbolic orders, zero order, all 4 theta indices, product of QPochhammer, Neg of Pow
- Context-aware parenthesization: Add wrapped in Mul/Pow/Neg contexts, Mul wrapped in Pow context
- Full test suite: 166 tests pass (11 unit + 33 arena + 55 number + 67 render)

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement LaTeX and Unicode rendering for all Expr variants** - `b4ab573` (feat)
2. **Task 2: Comprehensive rendering snapshot tests** - `84c4720` (test)

## Files Created/Modified

- `crates/qsym-core/src/render/mod.rs` (13 lines) - Public API: `pub mod latex; pub mod unicode;` with re-exports
- `crates/qsym-core/src/render/latex.rs` (260 lines) - LaTeX rendering with DLMF notation, Greek letter detection, parenthesization
- `crates/qsym-core/src/render/unicode.rs` (423 lines) - Unicode rendering with Greek chars, superscript/subscript digits, ASCII fallback
- `crates/qsym-core/tests/render_tests.rs` (778 lines) - 67 snapshot tests for both backends

## Decisions Made

1. **Always-brace policy for LaTeX** - All subscripts and superscripts use braces (`q^{2}`, `\theta_{2}`, `(a;q)_{5}`) even for single characters. Eliminates edge-case bugs with multi-character exponents.

2. **ASCII fallback for non-numeric Unicode sub/superscripts** - Unicode only supports digit subscripts/superscripts reliably. For symbolic orders like `(a;q)_n`, we use `_n` ASCII notation instead of attempting incomplete Unicode letter subscripts.

3. **Neg detection in Add** - When rendering `Add([a, Neg(b)])`, the second term renders as `a - b` (not `a + -b`). Both backends implement this consistently.

4. **DLMF 17.2 matrix notation for BasicHypergeometric** - LaTeX uses full `\begin{matrix}...\end{matrix}` notation. Unicode uses simplified `_r phi _s(upper;lower;q,z)` with subscript digits.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] LaTeX and mod.rs were already implemented**
- **Found during:** Task 1
- **Issue:** The `render/latex.rs` (260 lines) and `render/mod.rs` (13 lines) were already fully implemented and committed from a prior plan execution (commit 1783125 from 01-02). Only `render/unicode.rs` was a placeholder.
- **Fix:** Verified the existing LaTeX implementation was correct and matched plan requirements. Focused Task 1 on implementing unicode.rs.
- **Impact:** None -- the existing code was exactly what the plan specified.

## Issues Encountered

None -- all tests passed on first run.

## User Setup Required

None.

## Next Phase Readiness

- All rendering is complete: every Expr variant renders to both LaTeX and Unicode
- Phase 2 (simplification) can use rendering for debugging expression rewrites
- Phase 3 (q-series) can use rendering to display computed results
- Phase 5 (Python API) can delegate `__repr__`/`__str__` to these renderers

## Self-Check: PASSED

- All 4 files verified present on disk
- Commit b4ab573 (Task 1) verified in git log
- Commit 84c4720 (Task 2) verified in git log
- `cargo test` passes all 166 tests (11 unit + 33 arena + 55 number + 67 render + 1 doc-test)

---
*Phase: 01-expression-foundation*
*Completed: 2026-02-13*
