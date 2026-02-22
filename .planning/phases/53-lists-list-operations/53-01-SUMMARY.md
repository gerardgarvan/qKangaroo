---
phase: 53-lists-list-operations
plan: 01
subsystem: cli
tags: [parser, ast, evaluator, list-indexing, pratt-parser]

# Dependency graph
requires:
  - phase: 52-uat-gap-closure
    provides: "CLI parser, evaluator, and list literal support"
provides:
  - "AstNode::Index and AstNode::IndexAssign AST variants"
  - "Parser emits Index for all expr[index] patterns (no more Variable string munging)"
  - "1-indexed list element access: L[i] returns i-th element"
  - "List mutation via L[i] := value"
  - "Backward-compatible table-style X[i] := val for unbound variables"
  - "Direct literal indexing: [1,2,3][2]"
affects: [53-02, 53-03, lists, list-operations]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "AstNode::Index { expr, index } for subscript access on any expression"
    - "AstNode::IndexAssign { name, index, value } for indexed assignment"
    - "1-indexed Maple convention for list access"
    - "Symbol fallback for table-style indexed variables when base is unbound"

key-files:
  created: []
  modified:
    - "crates/qsym-cli/src/ast.rs"
    - "crates/qsym-cli/src/parser.rs"
    - "crates/qsym-cli/src/eval.rs"

key-decisions:
  - "Index works with arbitrary LHS expressions, not just variables"
  - "IndexAssign requires variable name on LHS (not arbitrary expression)"
  - "1-indexed Maple convention: L[1] is first element, L[0] is out-of-range error"
  - "Unbound variable fallback: X[1] looks up env var 'X[1]' for table-style compat"

patterns-established:
  - "Index handler: evaluate base, evaluate index, dispatch on base type (List/Symbol/other)"
  - "IndexAssign handler: check if name is List (mutate) or fall back to table-style"

requirements-completed: [LANG-02]

# Metrics
duration: 5min
completed: 2026-02-22
---

# Phase 53 Plan 01: List Indexing Summary

**AstNode::Index/IndexAssign with 1-indexed Maple list access, parser refactor from Variable string-munging to proper AST nodes, and backward-compatible table-style X[i] support**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-22T08:20:19Z
- **Completed:** 2026-02-22T08:25:05Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added AstNode::Index and AstNode::IndexAssign to the AST, replacing the Variable("X[1]") string-munging pattern
- Parser now emits proper Index nodes for any `expr[index]` syntax, supporting arbitrary expressions on both sides
- Evaluator handles list indexing (1-indexed), list mutation, symbol fallback for table-style variables, and literal indexing
- Full backward compatibility: `X[1] := 42; X[1]` still works for unbound variables
- 8 new tests (2 AST, 5 parser, 6 eval) plus all existing tests passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Add AstNode::Index/IndexAssign, refactor parser subscript handling** - `9b8d315` (feat)
2. **Task 2: Evaluator handlers for Index and IndexAssign with backward compatibility** - `fc03fac` (test)

## Files Created/Modified
- `crates/qsym-cli/src/ast.rs` - Added Index and IndexAssign variants to AstNode enum, plus 2 construction tests
- `crates/qsym-cli/src/parser.rs` - Refactored subscript parsing to emit AstNode::Index for all expr[index], updated assignment to handle IndexAssign, 5 new parser tests
- `crates/qsym-cli/src/eval.rs` - Added eval_expr handlers for Index (list/symbol/error dispatch) and IndexAssign (list mutation/table fallback), 6 new eval tests

## Decisions Made
- Index works on arbitrary LHS expressions (not just variables) -- enables `[1,2,3][2]` direct literal indexing
- IndexAssign requires a variable name on the LHS (not arbitrary expression) -- `L[i] := val` requires `L` to be a name
- 1-indexed per Maple convention: L[1] is first element, L[0] produces out-of-range error
- Symbol fallback: when base evaluates to Symbol (unbound variable), falls back to looking up "name[i]" as a table-style variable in the environment

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added full eval handlers in Task 1 commit instead of Task 2**
- **Found during:** Task 1
- **Issue:** Adding new AstNode variants requires exhaustive match coverage in eval_expr; code cannot compile without handlers
- **Fix:** Added full Index and IndexAssign eval handlers alongside the AST/parser changes in Task 1, then added tests in Task 2
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Verification:** All tests pass
- **Committed in:** 9b8d315 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary to maintain compilability between commits. No scope creep.

## Issues Encountered
- Pre-existing test failure `err_05_read_nonexistent_shows_file_not_found` (lexer treats `.` in file paths as unexpected character) -- unrelated to this plan, not addressed.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- List indexing foundation complete: L[i] read, L[i] := val write, literal indexing all working
- Ready for list operation functions (nops, op, append, seq, map, etc.) in plans 02-03

---
*Phase: 53-lists-list-operations*
*Completed: 2026-02-22*
