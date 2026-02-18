# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-17)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** v1.5 Interactive REPL -- COMPLETE

## Current Position

Phase: 28 of 28 (Binary Packaging)
Plan: 1 of 1 in current phase
Status: Complete
Last activity: 2026-02-18 -- Phase 28 complete (Binary Packaging: release profile, CI workflow, --version flag)

Progress: [##############################] 100% (79/79 plans -- v1.0-v1.5 complete)

## Performance Metrics

### v1.0-v1.4 Summary

- Total plans completed: 70
- Average duration: ~6 min/plan
- Total execution time: ~7 hours

### v1.5 Interactive REPL

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 24 - Parser & AST | 2/2 | 5min | 2.5min |
| 25 - Evaluator & Function Dispatch | 3/3 | 20min | 6.7min |
| 26 - REPL Shell & Session | 2/2 | 12min | 6min |
| 27 - Output Commands & Polish | 1/1 | 5min | 5min |
| 28 - Binary Packaging | 1/1 | 3min | 3min |

## Accumulated Context

### Decisions

All v1.0-v1.4 decisions logged in PROJECT.md Key Decisions table.

v1.5 decisions:
- qsym-cli depends only on qsym-core -- hand-written parser, no external libraries (except rustyline for REPL)
- AST nodes carry no span information (simplicity; can add Spanned<T> wrapper later)
- q is a reserved keyword token (Token::Q), not treated as an identifier
- BigInteger stored as String for arbitrary precision; evaluator converts to QInt
- Binding powers: := (2,1) < +/- (3,4) < */ (5,6) < unary- (7) < ^ (9,10) < funcall (11)
- Non-associative ^ enforced via post-parse check (error if chained)
- Function call as postfix operator requiring Variable lhs
- Empty statements (;;) silently skipped
- Value enum: 9 variants (Series, Integer, Rational, List, Dict, Pair, Bool, None, Infinity)
- Series + Integer promotes integer to constant FPS (matches Maple behavior)
- Integer / Integer produces Rational (exact arithmetic)
- 16 Maple aliases (case-insensitive): numbpart→partition_count, qphihyper→phi, etc.
- Levenshtein fuzzy matching for "did you mean" suggestions on unknown functions
- Panic catching via catch_unwind(AssertUnwindSafe(...)) for rug type safety
- prove_nonterminating returns informative error (requires closures, Python-only)
- rustyline 17.0 with derive feature for REPL line editing
- CompletionType::Circular (zsh-style Tab cycling)
- Auto-insert ( after function name completion
- Paren-counting Validator for multi-line input
- Commands intercepted before parser; lines with := always pass through to parser
- History file next to executable (.q_kangaroo_history)
- help system: 8 categories, 81 function entries, Commands section
- home crate pinned to 0.5.11 for Rust 1.85 compatibility
- complete_inner() extracted from Completer for testability
- Static FUNC_HELP array for zero-allocation help lookup
- format_latex ported from qsym-python FPS-level (not qsym-core Expr-level)
- save command writes format_value plain text output, not LaTeX
- LTO + strip + codegen-units=1 reduces release binary from ~4.5MB to ~1.4MB
- Bundle 5 MinGW DLLs on Windows (no static GMP linking)
- Separate cli-release.yml from existing release.yml (Python wheels vs CLI binaries)
- --version flag uses simple args check (no clap dependency)

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-18
Stopped at: Phase 28 complete. All 79 plans across 28 phases shipped. Project complete.
Resume file: N/A
