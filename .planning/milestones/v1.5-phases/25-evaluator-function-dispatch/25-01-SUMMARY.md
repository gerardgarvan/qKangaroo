---
phase: 25-evaluator-function-dispatch
plan: 01
subsystem: evaluator
tags: [eval, ast-walker, arithmetic, alias, levenshtein, panic-catching, environment]

# Dependency graph
requires:
  - phase: 24-parser-ast
    provides: "AstNode, Stmt, Terminator, Pratt parser, lexer, tokens"
provides:
  - "Value enum with 9 variants (Series, Integer, Rational, List, Dict, Pair, Bool, None, Infinity)"
  - "eval_expr recursive AST walker"
  - "eval_stmt/eval_stmt_safe with panic catching"
  - "Environment struct with SymbolRegistry, sym_q, variables, last_result"
  - "format_value for all Value variants"
  - "Argument extraction helpers (expect_args, extract_i64, extract_qrat, extract_series, etc.)"
  - "resolve_alias for 16 Maple function names"
  - "find_similar_names with Levenshtein edit distance"
  - "dispatch stub (Plans 02/03 fill in)"
  - "AstNode::List and parser support for [...] list literals"
affects: [25-02, 25-03, 26-repl-shell]

# Tech tracking
tech-stack:
  added: [rug (direct dep for BigInteger parsing)]
  patterns: [Value-enum tagged union, catch_unwind+AssertUnwindSafe for panic recovery, Levenshtein fuzzy matching]

key-files:
  created:
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/environment.rs
    - crates/qsym-cli/src/format.rs
  modified:
    - crates/qsym-cli/src/token.rs
    - crates/qsym-cli/src/ast.rs
    - crates/qsym-cli/src/lexer.rs
    - crates/qsym-cli/src/parser.rs
    - crates/qsym-cli/src/lib.rs
    - crates/qsym-cli/Cargo.toml

key-decisions:
  - "Series + Integer promotes integer to constant FPS (matches Maple behavior)"
  - "Integer / Integer produces Rational (not truncated Integer)"
  - "rug added as direct dependency for BigInteger string parsing"
  - "Dispatch stub returns UnknownFunction -- Plans 02/03 replace body"

patterns-established:
  - "Value enum: tagged union for all evaluator return types"
  - "eval_binop: match on (left_type, right_type) with promotion rules"
  - "eval_stmt_safe: catch_unwind(AssertUnwindSafe) wrapping eval_stmt"
  - "resolve_alias: case-insensitive match on lowercase name"
  - "Argument extraction helpers: expect_args + extract_TYPE pattern"

requirements-completed: [SESS-01, OUT-01]

# Metrics
duration: 7min
completed: 2026-02-18
---

# Phase 25 Plan 01: Evaluator Foundation Summary

**AST evaluator with Value enum, series/integer/rational arithmetic, environment with SymbolRegistry, panic catching via catch_unwind, and Levenshtein fuzzy matching for 79+16 function names**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-18T02:09:28Z
- **Completed:** 2026-02-18T02:16:01Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Complete AST evaluator that walks all AstNode variants and produces Value results
- Full arithmetic dispatch for Series, Integer, and Rational with cross-type promotion
- Parser extension for [...] list literals (empty, nested, expression elements)
- Environment with SymbolRegistry, sym_q, variable storage, and last_result
- Panic catching wraps eval_stmt in catch_unwind for qsym-core robustness
- Alias resolution for all 16 Maple function names (case-insensitive)
- Levenshtein-based "Did you mean?" suggestions for unknown functions
- 7 argument extraction helpers ready for Plans 02/03 dispatch functions

## Task Commits

Each task was committed atomically:

1. **Task 1: Parser extension for list literals and evaluator foundation types** - `bd0e01e` (feat)
2. **Task 2: Evaluator core with Value enum, expression evaluation, arithmetic, and panic catching** - `c950742` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/eval.rs` - Value enum, EvalError, eval_expr, eval_stmt, eval_stmt_safe, dispatch stub, alias resolution, fuzzy matching, argument helpers (640 lines code + 370 lines tests)
- `crates/qsym-cli/src/environment.rs` - Environment struct with SymbolRegistry, sym_q, variables, last_result, default_order
- `crates/qsym-cli/src/format.rs` - format_value for all 9 Value variants including matrix display
- `crates/qsym-cli/src/token.rs` - Added LBracket/RBracket variants
- `crates/qsym-cli/src/ast.rs` - Added AstNode::List variant
- `crates/qsym-cli/src/lexer.rs` - Added [ and ] tokenization
- `crates/qsym-cli/src/parser.rs` - Added list literal parsing in expr_bp prefix, 8 new parser tests
- `crates/qsym-cli/src/lib.rs` - Added eval, environment, format modules
- `crates/qsym-cli/Cargo.toml` - Added rug dependency

## Decisions Made
- **Series + Integer promotion:** `q + 1` produces `1 + q + O(q^20)` by converting integer to constant FPS. This matches Maple behavior and is the most intuitive for researchers.
- **Integer division produces Rational:** `3/4` evaluates to `QRat(3/4)`, not `QInt(0)`. Mathematicians expect exact arithmetic.
- **rug direct dependency:** Added `rug = "1.28"` to qsym-cli for `Integer::from_str_radix` in BigInteger handling. QInt has pub field but qsym-cli didn't previously depend on rug directly.
- **Dispatch stub pattern:** dispatch() returns UnknownFunction with fuzzy suggestions immediately. Plans 02/03 fill in the match arms. This means the error path (alias + fuzzy) is tested now, not deferred.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added rug as direct dependency for BigInteger parsing**
- **Found during:** Task 2 (eval_expr BigInteger handling)
- **Issue:** eval.rs used `rug::Integer::from_str_radix()` but qsym-cli didn't directly depend on rug
- **Fix:** Added `rug = "1.28"` to crates/qsym-cli/Cargo.toml
- **Files modified:** crates/qsym-cli/Cargo.toml
- **Verification:** Compiles, BigInteger test passes
- **Committed in:** c950742 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Trivial dependency addition, no scope creep.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- eval.rs dispatch stub is ready for Plans 02 and 03 to fill in all 79 function handlers
- Argument extraction helpers are tested and ready for use in dispatch functions
- Environment provides sym_q for all series-generating functions
- format_value handles all Value variants for REPL output
- 141 tests provide regression safety

---
*Phase: 25-evaluator-function-dispatch*
*Completed: 2026-02-18*
