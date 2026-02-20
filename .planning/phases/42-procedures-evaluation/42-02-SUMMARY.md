---
phase: 42-procedures-evaluation
plan: 02
subsystem: qsym-cli
tags: [procedures, evaluation, memoization, parsing, repl]
dependency_graph:
  requires: [42-01]
  provides: [Procedure struct, Value::Procedure, proc/local/option tokens, AstNode::ProcDef, call_procedure, memoization]
  affects: [eval.rs, format.rs, repl.rs, token.rs, lexer.rs, ast.rs, parser.rs]
tech_stack:
  added: [Rc<RefCell<HashMap>> for memo tables]
  patterns: [save/restore local scoping, EarlyReturn catch at procedure boundary, keyword-counting REPL multiline]
key_files:
  created: []
  modified:
    - crates/qsym-cli/src/token.rs
    - crates/qsym-cli/src/lexer.rs
    - crates/qsym-cli/src/ast.rs
    - crates/qsym-cli/src/parser.rs
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/format.rs
    - crates/qsym-cli/src/repl.rs
decisions:
  - Procedure struct uses Rc<RefCell<HashMap<String,Value>>> for shared memo table across clones
  - OptionKw token name avoids collision with Rust Option type
  - Local variables intentionally not initialized (accessing returns Symbol, matching Maple behavior)
  - parse_ident_list helper reused for params, locals, and options parsing
  - "end" keyword decrements proc_depth in REPL (od/fi handle for/if separately)
  - Procedure name set during assignment (empty for anonymous procs)
metrics:
  duration: 7 min
  completed: 2026-02-20T20:06:06Z
  tasks: 2
  tests_added: 27
  tests_total: 682
  files_modified: 7
---

# Phase 42 Plan 02: Procedure Definition, Calling, and Memoization Summary

Maple-compatible procedure definitions with local scoping, early return, memoization, and REPL multiline support across 7 files.

## Task 1: Tokens, Lexer, AST, and Parser (commit 2bfc698)

Added three new token variants (`Token::Proc`, `Token::Local`, `Token::OptionKw`) to `token.rs` with corresponding keyword mappings in `lexer.rs`. Added `AstNode::ProcDef` with `params`, `locals`, `options`, and `body` fields to `ast.rs`. In `parser.rs`, implemented `Token::Proc` as a prefix parsing case in `expr_bp()` that handles the full Maple syntax:

```
proc(params) [local vars;] [option opts;] body; end [proc]
```

Created `parse_ident_list()` helper for comma-separated identifier parsing, reused for params, locals, and options. Added `token_name()` entries for new tokens. Added 11 new tests (3 lexer + 8 parser).

## Task 2: Evaluation, Calling, Memoization, Format, REPL (commit cea3be1)

**eval.rs**: Defined `Procedure` struct with `name`, `params`, `locals`, `remember`, `body`, and `memo` (Rc<RefCell<HashMap>>). Added `Value::Procedure` variant. `ProcDef` evaluation creates a `Procedure` with empty name (set during `:=` assignment). `FuncCall` checks environment for `Value::Procedure` before falling through to builtin `dispatch()`, enabling user procedures to shadow builtins.

Implemented `call_procedure()` with:
1. Arity check
2. Memo lookup (for `option remember`)
3. Variable save/restore for params and locals
4. Parameter binding
5. Body execution with `EarlyReturn` catch at procedure boundary
6. Unconditional variable restore (all paths: success, error, EarlyReturn)
7. Memo store on success

**format.rs**: `format_value` shows `proc(params) ... end proc`; `format_latex` shows `\text{proc}(params)`.

**repl.rs**: Added `proc_depth` counter to `is_incomplete()`. Keywords `proc` increments and `end` decrements. Updated `check_keyword()` signature with new parameter across all 4 call sites.

Added 16 new tests (13 eval + 3 REPL) covering: basic proc call, local scoping, local non-leaking, early return, return in nested for/if, option remember, memoized fibonacci (fib(10)=55), wrong arg count error, builtin shadowing, empty body, multiple statements, nested control flow, variable restore on error, and REPL multiline detection.

## Deviations from Plan

None -- plan executed exactly as written.

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Rc<RefCell<HashMap>> for memo | Shared across clones so memoization persists after assignment |
| OptionKw not Option | Avoids collision with Rust standard library Option type |
| Uninit locals return Symbol | Matches Maple behavior where undefined names are symbolic |
| "end" decrements proc_depth | Separate from od/fi which handle their own depth counters |
| Procedure name set on Assign | Anonymous procs get empty name, named on `:=` assignment |

## Test Summary

- **Before**: 503 unit + 152 integration = 655 total
- **Added**: 27 new (3 lexer + 8 parser + 13 eval + 3 REPL)
- **After**: 530 unit + 152 integration = 682 total
- All tests pass

## Self-Check: PASSED

- [x] crates/qsym-cli/src/token.rs -- modified, contains Proc/Local/OptionKw
- [x] crates/qsym-cli/src/lexer.rs -- modified, contains "proc"/"local"/"option" mappings
- [x] crates/qsym-cli/src/ast.rs -- modified, contains ProcDef variant
- [x] crates/qsym-cli/src/parser.rs -- modified, contains Token::Proc prefix case
- [x] crates/qsym-cli/src/eval.rs -- modified, contains Procedure struct and call_procedure
- [x] crates/qsym-cli/src/format.rs -- modified, contains Procedure display
- [x] crates/qsym-cli/src/repl.rs -- modified, contains proc_depth tracking
- [x] Commit 2bfc698 exists (Task 1)
- [x] Commit cea3be1 exists (Task 2)
