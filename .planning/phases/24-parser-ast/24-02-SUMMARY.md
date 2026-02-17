---
phase: 24-parser-ast
plan: 02
subsystem: parser
tags: [lexer, parser, pratt, tokenizer, cli, repl]

requires:
  - phase: 24-parser-ast
    plan: 01
    provides: "Token, AstNode, Span, SpannedToken, ParseError types"
provides:
  - "tokenize() function: &str -> Result<Vec<SpannedToken>, ParseError>"
  - "parse() function: &str -> Result<Vec<Stmt>, ParseError>"
  - "Pratt parser with correct operator precedence for Maple-style grammar"
  - "Non-associative exponentiation enforcement"
  - "Multi-statement parsing with ;/: terminators"
affects: [25-evaluator, 26-repl-shell]

tech-stack:
  added: []
  patterns:
    - "Pratt (TDOP) parsing with explicit binding powers for precedence"
    - "Greedy lexer with := vs : disambiguation"
    - "Integer overflow detection to BigInteger(String) at lex time"

key-files:
  created:
    - crates/qsym-cli/src/lexer.rs
    - crates/qsym-cli/src/parser.rs
  modified:
    - crates/qsym-cli/src/lib.rs

key-decisions:
  - "Binding powers: := (2,1) < +/- (3,4) < */ (5,6) < unary- (7) < ^ (9,10) < funcall (11)"
  - "Non-associative ^ via post-parse check (error if next token is also ^)"
  - "Function call as postfix operator (highest precedence) requiring Variable lhs"
  - "Empty statements (;;) silently skipped, not errors"

patterns-established:
  - "Pratt parser pattern: expr_bp(min_bp) with prefix NUD + infix LED loop"
  - "tokenize() -> parse() pipeline: lexer produces token stream, parser consumes it"
  - "Error messages include token names for user clarity"

requirements-completed: [PARSE-01, PARSE-02, PARSE-03, PARSE-04]

duration: 3min
completed: 2026-02-17
---

# Phase 24 Plan 02: Lexer & Pratt Parser Summary

**Complete Maple-style lexer and Pratt parser with 48 tests covering function calls, assignment, arithmetic with precedence, non-associative ^, multi-statement chaining, and error diagnostics**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-17T18:22:36Z
- **Completed:** 2026-02-17T18:26:02Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Implemented tokenize() lexer handling all 17 token types with byte-offset spans, := vs : disambiguation, integer overflow to BigInteger, and keyword recognition (q, infinity)
- Implemented Pratt parser with correct Maple-style operator precedence: assignment < additive < multiplicative < unary negation < exponentiation < function call
- Non-associative exponentiation enforced (2^3^4 is a parse error per Maple convention)
- Multi-statement parsing with ; (print) and : (suppress) terminators, empty statement skipping
- 48 new tests (12 lexer + 36 parser) covering all PARSE-01 through PARSE-04 requirements
- 68 total tests in qsym-cli (20 type tests from Plan 01 + 48 new)

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement the lexer (tokenizer)** - `4406aea` (feat)
2. **Task 2: Implement the Pratt parser with comprehensive tests** - `9d1b318` (feat)

## Files Created/Modified
- `crates/qsym-cli/src/lexer.rs` - Tokenizer: tokenize() producing Vec<SpannedToken> with 12 tests
- `crates/qsym-cli/src/parser.rs` - Pratt parser: parse() producing Vec<Stmt> with 36 tests
- `crates/qsym-cli/src/lib.rs` - Added pub mod lexer and pub mod parser

## Decisions Made
- Binding power table: := (2,1) < +/- (3,4) < */ (5,6) < unary- (r_bp=7) < ^ (9,10) < funcall (l_bp=11)
- Non-associative ^ enforced by post-parse check: after parsing a^b, if next token is also ^, return error with suggestion to use parentheses
- Function call treated as postfix operator with highest precedence, only allowed on Variable lhs
- Assignment treated as right-associative infix operator (r_bp=1 < l_bp=2), only allowed on Variable lhs
- Empty statements between consecutive ;/: silently skipped (no error)
- Parser stores source string for future error rendering (currently #[allow(dead_code)])

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed test_assign_non_variable_error assertion**
- **Found during:** Task 2 (parser tests)
- **Issue:** Test initially expected "expected ';', ':', or end of input" but the parser correctly produces "left side of := must be a variable name" for `3 := 5` since := is detected in the infix loop
- **Fix:** Updated test assertion to match actual (correct) error message
- **Files modified:** crates/qsym-cli/src/parser.rs
- **Verification:** All 68 tests pass
- **Committed in:** 9d1b318 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug in test assertion)
**Impact on plan:** Trivial test assertion correction. No scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- parse() is the complete entry point for Phase 25 (evaluator) to consume
- All AST node types (Integer, BigInteger, Q, Infinity, LastResult, Variable, BinOp, Neg, FuncCall, Assign) are produced
- Statement terminators (Semi, Colon, Implicit) ready for REPL output control
- Error messages include byte spans for diagnostic rendering

## Self-Check: PASSED

All 3 files verified present. Both task commits (4406aea, 9d1b318) confirmed in git log.

---
*Phase: 24-parser-ast*
*Completed: 2026-02-17*
