# Plan 41-01 Summary: Tokens, AST Types, Lexer, and Operator Parsing

**Status:** Complete
**Tests:** 152 integration + 448 total (all pass)

## What was done

### Task 1: Token variants, AST types, and lexer extensions

- **token.rs**: Added 22 new Token variants: 13 control flow keywords (For, From, To, By, Do, Od, While, If, Then, Elif, Else, Fi, End), 3 boolean operators (And, Or, Not), 6 comparison operators (Equal, NotEqual, Less, Greater, LessEq, GreaterEq)
- **ast.rs**: Added CompOp enum (6 variants), BoolBinOp enum (2 variants), and 5 new AstNode variants (Compare, Not, BoolOp, ForLoop, IfExpr)
- **lexer.rs**: Extended keyword matching for all 16 new words (for/from/to/by/do/od/while/if/then/elif/else/fi/end/and/or/not). Added multi-char operator lexing for `<`, `<=`, `<>`, `>`, `>=`, `=` with proper greedy matching.

### Task 2: Parser binding power renumbering and comparison/boolean parsing

- **parser.rs**: Renumbered binding powers to Maple-compatible layout:
  - Assignment: 2/1 (unchanged)
  - or: 3/4 (new)
  - and: 5/6 (new)
  - not prefix: 7 (new)
  - comparisons: 9/10 (new)
  - add/sub: 11/12 (was 3/4)
  - mul/div: 13/14 (was 5/6)
  - prefix neg: 15 (was 7)
  - pow: 17/18 (was 9/10)
  - function call: 19 (was 11)
- Added comparison operators producing Compare AST nodes
- Added boolean and/or producing BoolOp AST nodes
- Added prefix not producing Not AST node
- Non-associativity check for chained comparisons
- Updated token_name() for all 22 new variants

### Evaluator stub
- **eval.rs**: Added stub arm for Compare/Not/BoolOp/ForLoop/IfExpr returning "control flow not yet implemented"

## Verification

- All 152 integration tests pass
- All existing parser tests pass unchanged (binding power renumbering preserves relative ordering)
- `:=` and `=` are cleanly disambiguated
- Comparison operators are non-associative (chaining errors with clear message)
