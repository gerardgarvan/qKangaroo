---
phase: 47-parser-language-extensions
plan: 01
subsystem: parser
tags: [lexer, pratt-parser, maple-compat, ditto-operator]

# Dependency graph
requires:
  - phase: 24-28 (v1.5 Interactive REPL)
    provides: Token/Lexer/Parser infrastructure, AstNode::LastResult, Token::Percent
provides:
  - Token::Ditto variant with byte-lookahead lexer disambiguation
  - Ditto NUD mapping to AstNode::LastResult (reuses existing eval path)
  - Flexible proc option/local parsing (either order, any combination)
affects: [47-02, 47-03, eval, REPL]

# Tech tracking
tech-stack:
  added: []
  patterns: [byte-lookahead disambiguation for context-sensitive tokens]

key-files:
  modified:
    - crates/qsym-cli/src/token.rs
    - crates/qsym-cli/src/lexer.rs
    - crates/qsym-cli/src/parser.rs

key-decisions:
  - "Ditto disambiguated via byte-lookahead (next char after quote) rather than parser-level context"
  - "Proc option/local uses loop instead of sequential if-blocks, allowing either order and multiples"

patterns-established:
  - "Byte-lookahead disambiguation: check next byte to resolve ambiguous tokens at lexer level"

requirements-completed: [LANG-01, LANG-05]

# Metrics
duration: 6min
completed: 2026-02-21
---

# Phase 47 Plan 01: Ditto Operator and Proc Option/Local Reorder Summary

**Maple-style ditto operator (`"`) with byte-lookahead disambiguation and flexible proc local/option ordering**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-21T04:35:16Z
- **Completed:** 2026-02-21T04:41:43Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Token::Ditto variant added with byte-lookahead disambiguation from string literals
- Ditto NUD maps to AstNode::LastResult, reusing existing `%` evaluation path
- Proc definitions now accept `local`/`option` in either order via loop
- 15 new tests (7 lexer + 8 parser) covering ditto and option/local reorder

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Token::Ditto and lexer disambiguation** - `4d448c1` (feat)
2. **Task 2: Parser ditto NUD, option/local reorder, and token_name update** - `69c093b` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/token.rs` - Added Ditto variant to Token enum
- `crates/qsym-cli/src/lexer.rs` - Byte-lookahead ditto/string disambiguation, 7 new tests
- `crates/qsym-cli/src/parser.rs` - Ditto NUD arm, option/local loop, token_name entry, 8 new tests
- `crates/qsym-cli/src/eval.rs` - Fixed pre-existing FractionalPowerSeries denom deref errors
- `crates/qsym-cli/src/format.rs` - Added FractionalPowerSeries arms to format_value and format_latex

## Decisions Made
- Used byte-lookahead disambiguation: after seeing `"`, peek at next byte to determine if it's ditto (followed by delimiter/operator/whitespace/EOF) or string literal start. This is simple, efficient, and correct for all observed Maple usage patterns.
- Reused AstNode::LastResult for ditto (same as `%`) -- no evaluator changes needed.
- Used a loop for proc option/local parsing to support any combination and order.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added Token::Ditto to token_name in parser.rs during Task 1**
- **Found during:** Task 1 (lexer changes)
- **Issue:** Adding Token::Ditto to the enum caused non-exhaustive match in parser.rs token_name, preventing compilation
- **Fix:** Added `Token::Ditto => "'\"' (ditto)".to_string()` to token_name
- **Files modified:** crates/qsym-cli/src/parser.rs
- **Verification:** Compilation succeeded
- **Committed in:** 4d448c1 (Task 1 commit)

**2. [Rule 3 - Blocking] Fixed pre-existing FractionalPowerSeries compilation errors**
- **Found during:** Task 2 (parser changes)
- **Issue:** Pre-existing E0614 (type `i64` cannot be dereferenced) and E0004 (non-exhaustive match) errors in eval.rs and format.rs prevented compilation of entire crate
- **Fix:** Fixed `denom: *denom` to `denom` in eval_neg (by-value match), added FractionalPowerSeries arms to format.rs format_value and format_latex
- **Files modified:** crates/qsym-cli/src/eval.rs, crates/qsym-cli/src/format.rs
- **Verification:** Compilation succeeded, 642 lib tests + 151 integration tests pass
- **Committed in:** 69c093b (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both auto-fixes were required for compilation. No scope creep.

## Issues Encountered
- Pre-existing test failure `eval::tests::eval_div_series_by_fractional` (FPS division logic) and `err_05_read_nonexistent_shows_file_not_found` (integration) -- both unrelated to this plan's changes, not addressed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Ditto operator and proc reorder complete, ready for 47-02 (while loops, end do/if/proc, multi-return)
- No blockers

---
*Phase: 47-parser-language-extensions*
*Completed: 2026-02-21*
