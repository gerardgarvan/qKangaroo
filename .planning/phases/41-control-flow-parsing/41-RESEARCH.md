# Phase 41: Control Flow Parsing - Research

**Researched:** 2026-02-20
**Domain:** Lexer/parser extension for control flow (for-loops, conditionals, boolean/comparison operators) in a hand-written Pratt parser
**Confidence:** HIGH

## Summary

This phase extends the existing hand-written Pratt parser in `qsym-cli` to support Maple-style control flow: `for...from...to...do...od` loops, `if...then...elif...else...fi` conditionals, and boolean/comparison operators. The codebase has a clean, well-structured lexer/parser/AST pipeline with 18 token variants, 10 AST node variants, and a Pratt parser using binding powers for operator precedence. All changes are internal to the existing architecture -- no external dependencies needed.

The key challenge is integrating statement-level constructs (for-loops, if-conditionals) into an expression-based Pratt parser. Maple treats these as expressions (they return the value of the last evaluated statement), so they fit naturally as prefix-position atoms in the Pratt parser's NUD (null denotation) position. The lexer needs 13 new keyword tokens and 6 new operator tokens. The AST needs 3 new node variants (ForLoop, IfExpr, comparison/boolean ops -- the latter can reuse the existing BinOp pattern or add new variants). Operator precedence must follow Maple's hierarchy where comparisons bind looser than arithmetic but tighter than boolean `not`, and boolean operators bind loosest of all.

**Primary recommendation:** Add new keyword tokens in lexer.rs (extending the identifier-to-keyword match), add comparison/boolean operator tokens, extend the AST with ForLoop and IfExpr variants plus comparison/boolean operator enums, and parse for/if as prefix atoms in `expr_bp`. Do NOT evaluate these constructs -- Phase 42 handles evaluation.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SCRIPT-01 | User can write for-loops: `for n from 1 to 8 do stmts od` | Lexer keywords (for/from/to/by/do/od), ForLoop AST node with variable/from/to/by/body fields, parsed as prefix atom in Pratt parser |
| SCRIPT-06 | User can use if/elif/else/fi conditionals in procedures and at top level | Lexer keywords (if/then/elif/else/fi), IfExpr AST node with condition/then_body/elif_branches/else_body fields, parsed as prefix atom |
| SCRIPT-07 | User can use boolean operators (and, or, not) and comparison operators (=, <>, <, >, <=, >=) in conditions | Lexer keywords (and/or/not) and operator tokens (<, >, <=, >=, <>, =), new CompOp/BoolOp enums or extended BinOp, binding powers following Maple precedence |
</phase_requirements>

## Standard Stack

### Core
| Component | Location | Purpose | Why Standard |
|-----------|----------|---------|--------------|
| token.rs | `crates/qsym-cli/src/token.rs` | Token enum with 18 existing variants | Extend with ~19 new variants for keywords and operators |
| ast.rs | `crates/qsym-cli/src/ast.rs` | AstNode enum with 10 variants + BinOp enum | Add ForLoop, IfExpr, comparison/boolean node types |
| lexer.rs | `crates/qsym-cli/src/lexer.rs` | Hand-written byte-by-byte tokenizer | Extend keyword matching and multi-char operator lexing |
| parser.rs | `crates/qsym-cli/src/parser.rs` | Hand-written Pratt parser with `expr_bp(min_bp)` | Add prefix parsing for for/if, infix for comparisons/booleans |
| error.rs | `crates/qsym-cli/src/error.rs` | ParseError with span-based caret rendering | Reuse for all new parse error messages |

### Supporting
| Component | Purpose | When to Use |
|-----------|---------|-------------|
| repl.rs `is_incomplete()` | Bracket-counting multi-line validator | Must be extended to detect unclosed for..od and if..fi blocks |
| help.rs | Function/keyword completion list | Will eventually list new keywords but not required in Phase 41 |

## Architecture Patterns

### Current Binding Power Layout

```
Assignment:       l_bp=2,  r_bp=1   (right-associative)
Add/Sub:          l_bp=3,  r_bp=4   (left-associative)
Mul/Div:          l_bp=5,  r_bp=6   (left-associative)
Prefix neg:       r_bp=7
Pow (^):          l_bp=9,  r_bp=10  (non-associative, error on chaining)
Function call:    l_bp=11
```

### Extended Binding Power Layout (Maple-compatible)

Following Maple's documented precedence (HIGH to LOW: arithmetic > comparison > not > and > or > assignment), the new operators slot in as follows:

```
Assignment:       l_bp=2,  r_bp=1   (right-associative, unchanged)
  -- NEW: or:     l_bp=3,  r_bp=4   (left-associative, Maple: lowest boolean)
  -- NEW: and:    l_bp=5,  r_bp=6   (left-associative, Maple: above or)
  -- NEW: not:    prefix r_bp=7     (prefix unary, Maple: above and)
  -- NEW: comparisons (=, <>, <, >, <=, >=):
                  l_bp=9,  r_bp=10  (non-associative, Maple: above not)
Add/Sub:          l_bp=11, r_bp=12  (left-associative, RENUMBERED)
Mul/Div:          l_bp=13, r_bp=14  (left-associative, RENUMBERED)
Prefix neg:       r_bp=15           (RENUMBERED)
Pow (^):          l_bp=17, r_bp=18  (non-associative, RENUMBERED)
Function call:    l_bp=19           (RENUMBERED)
```

**CRITICAL: All existing binding powers must be renumbered to make room.** This is a breaking change to the internal numbers only -- the relative ordering and behavior of all existing operators remains identical. Using gaps of 2 per level (even/odd for left/right bp) is the standard Pratt parser approach and allows future insertions.

### Pattern 1: Keyword Tokens via Lexer Match Extension
**What:** New keywords are recognized in the existing identifier lexing path by extending the `match word { ... }` block.
**When to use:** All 13 new keywords (for, from, to, by, do, od, while, if, then, elif, else, fi, end).
**Example:**
```rust
// In lexer.rs, extend the keyword match in the identifier branch:
let token = match word {
    "infinity" => Token::Infinity,
    // New control flow keywords:
    "for" => Token::For,
    "from" => Token::From,
    "to" => Token::To,
    "by" => Token::By,
    "do" => Token::Do,
    "od" => Token::Od,
    "while" => Token::While,
    "if" => Token::If,
    "then" => Token::Then,
    "elif" => Token::Elif,
    "else" => Token::Else,
    "fi" => Token::Fi,
    "and" => Token::And,
    "or" => Token::Or,
    "not" => Token::Not,
    _ => Token::Ident(word.to_string()),
};
```

### Pattern 2: Multi-Character Operator Lexing
**What:** The `<`, `>`, `<=`, `>=`, `<>`, and `=` operators require greedy multi-character matching in the lexer.
**When to use:** Lexing comparison operators.
**Example:**
```rust
// In lexer.rs, before the unknown-character fallback:
// Handle '<', '<=', '<>'
if b == b'<' {
    if pos + 1 < bytes.len() && bytes[pos + 1] == b'=' {
        tokens.push(SpannedToken { token: Token::LessEq, span: Span::new(pos, pos + 2) });
        pos += 2;
    } else if pos + 1 < bytes.len() && bytes[pos + 1] == b'>' {
        tokens.push(SpannedToken { token: Token::NotEqual, span: Span::new(pos, pos + 2) });
        pos += 2;
    } else {
        tokens.push(SpannedToken { token: Token::Less, span: Span::new(pos, pos + 1) });
        pos += 1;
    }
    continue;
}
// Handle '>', '>='
if b == b'>' {
    if pos + 1 < bytes.len() && bytes[pos + 1] == b'=' {
        tokens.push(SpannedToken { token: Token::GreaterEq, span: Span::new(pos, pos + 2) });
        pos += 2;
    } else {
        tokens.push(SpannedToken { token: Token::Greater, span: Span::new(pos, pos + 1) });
        pos += 1;
    }
    continue;
}
// Handle '=' (comparison, NOT assignment -- ':=' is already handled above)
if b == b'=' {
    tokens.push(SpannedToken { token: Token::Equal, span: Span::new(pos, pos + 1) });
    pos += 1;
    continue;
}
```

### Pattern 3: ForLoop as Prefix Atom in Pratt Parser
**What:** `for` is parsed as a prefix (NUD) position in `expr_bp`, consuming all clauses up to `od`.
**When to use:** Parsing `for n from 1 to 5 do ... od`.
**Example:**
```rust
// In parser.rs expr_bp, add to the prefix match:
Token::For => {
    self.advance(); // consume 'for'
    // Loop variable name
    let var_name = match self.peek() {
        Token::Ident(name) => { let n = name.clone(); self.advance(); n }
        _ => return Err(ParseError::new("expected variable name after 'for'", self.peek_span()))
    };
    // Optional 'from' clause (default: 1)
    let from_expr = if *self.peek() == Token::From {
        self.advance();
        Some(self.expr_bp(0)?)
    } else {
        None // default to 1
    };
    // 'to' clause (required for now in our subset)
    self.expect(&Token::To, "'to' in for loop")?;
    let to_expr = self.expr_bp(0)?;
    // Optional 'by' clause (default: 1)
    let by_expr = if *self.peek() == Token::By {
        self.advance();
        Some(self.expr_bp(0)?)
    } else {
        None // default to 1
    };
    // 'do' keyword
    self.expect(&Token::Do, "'do' in for loop")?;
    // Parse body as statement sequence until 'od'
    let body = self.parse_stmt_sequence(&Token::Od)?;
    self.expect(&Token::Od, "'od' to close for loop")?;
    AstNode::ForLoop {
        var: var_name,
        from: Box::new(from_expr.unwrap_or(AstNode::Integer(1))),
        to: Box::new(to_expr),
        by: by_expr.map(Box::new),
        body,
    }
}
```

### Pattern 4: IfExpr as Prefix Atom in Pratt Parser
**What:** `if` is parsed as a prefix atom, consuming condition, then-body, optional elif chains, optional else, and `fi`.
**When to use:** Parsing `if cond then ... elif cond then ... else ... fi`.
**Example:**
```rust
Token::If => {
    self.advance(); // consume 'if'
    let condition = self.expr_bp(0)?;
    self.expect(&Token::Then, "'then' after if condition")?;
    let then_body = self.parse_stmt_sequence_until_elif_else_fi()?;
    let mut elif_branches = Vec::new();
    while *self.peek() == Token::Elif {
        self.advance(); // consume 'elif'
        let elif_cond = self.expr_bp(0)?;
        self.expect(&Token::Then, "'then' after elif condition")?;
        let elif_body = self.parse_stmt_sequence_until_elif_else_fi()?;
        elif_branches.push((elif_cond, elif_body));
    }
    let else_body = if *self.peek() == Token::Else {
        self.advance();
        Some(self.parse_stmt_sequence_until_fi()?)
    } else {
        None
    };
    self.expect(&Token::Fi, "'fi' to close if expression")?;
    AstNode::IfExpr {
        condition: Box::new(condition),
        then_body,
        elif_branches,
        else_body,
    }
}
```

### Pattern 5: Statement Sequence Parsing
**What:** Bodies of for-loops and if-conditionals contain multiple semicolon/colon-separated statements. Need a `parse_stmt_sequence` method that collects statements until a terminating keyword.
**When to use:** Parsing the body between `do...od` and `then...elif/else/fi`.
**Example:**
```rust
/// Parse statements until we see a terminating keyword token.
/// The terminating token is NOT consumed.
fn parse_stmt_sequence(&mut self, terminators: &[&Token]) -> Result<Vec<Stmt>, ParseError> {
    let mut stmts = Vec::new();
    loop {
        // Skip empty statements (consecutive ; or :)
        while matches!(self.peek(), Token::Semi | Token::Colon) {
            self.advance();
        }
        // Check for terminator
        if terminators.iter().any(|t| self.peek() == *t) || self.at_end() {
            break;
        }
        let node = self.expr_bp(0)?;
        let terminator = match self.peek() {
            Token::Semi => { self.advance(); Terminator::Semi }
            Token::Colon => { self.advance(); Terminator::Colon }
            _ if terminators.iter().any(|t| self.peek() == *t) => Terminator::Implicit,
            Token::Eof => Terminator::Implicit,
            _ => {
                // The expression might be followed by the terminator keyword
                // (e.g., the last stmt before 'od' doesn't need ;)
                Terminator::Implicit
            }
        };
        stmts.push(Stmt { node, terminator });
    }
    Ok(stmts)
}
```

### Pattern 6: Comparison and Boolean Operators as Infix in Pratt Parser
**What:** Comparison operators (`=`, `<>`, `<`, `>`, `<=`, `>=`) and boolean operators (`and`, `or`) are infix operators in the Pratt parser. `not` is a prefix operator.
**When to use:** Parsing conditions in if/while/for-while.
**Example:**
```rust
// Extend infix_bp to include comparison and boolean operators:
fn infix_bp(token: &Token) -> Option<(u8, u8)> {
    match token {
        Token::Or => Some((3, 4)),                          // lowest: or
        Token::And => Some((5, 6)),                         // above or: and
        Token::Equal | Token::NotEqual |
        Token::Less | Token::Greater |
        Token::LessEq | Token::GreaterEq => Some((9, 10)), // above not: comparisons
        Token::Plus | Token::Minus => Some((11, 12)),       // arithmetic add/sub
        Token::Star | Token::Slash => Some((13, 14)),       // arithmetic mul/div
        Token::Caret => Some((17, 18)),                     // exponentiation
        _ => None,
    }
}

// In expr_bp prefix section, add 'not':
Token::Not => {
    self.advance();
    let rhs = self.expr_bp(7)?; // prefix 'not' binds at r_bp=7
    AstNode::UnaryOp { op: UnaryOp::Not, operand: Box::new(rhs) }
}
```

### AST Node Additions

```rust
// New variants for AstNode:

/// For loop: `for var from expr to expr [by expr] do stmts od`
ForLoop {
    var: String,
    from: Box<AstNode>,
    to: Box<AstNode>,
    by: Option<Box<AstNode>>,
    body: Vec<Stmt>,
},

/// Conditional: `if cond then stmts [elif cond then stmts]* [else stmts] fi`
IfExpr {
    condition: Box<AstNode>,
    then_body: Vec<Stmt>,
    elif_branches: Vec<(AstNode, Vec<Stmt>)>,
    else_body: Option<Vec<Stmt>>,
},

/// Comparison operation: `lhs op rhs`
Compare {
    op: CompOp,
    lhs: Box<AstNode>,
    rhs: Box<AstNode>,
},

/// Boolean NOT: `not expr`
Not(Box<AstNode>),

/// Boolean AND/OR: `lhs and rhs` / `lhs or rhs`
BoolOp {
    op: BoolBinOp,
    lhs: Box<AstNode>,
    rhs: Box<AstNode>,
},
```

```rust
// New enums:

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompOp {
    Eq,       // =
    NotEq,    // <>
    Less,     // <
    Greater,  // >
    LessEq,   // <=
    GreaterEq,// >=
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolBinOp {
    And,
    Or,
}
```

### Token Additions

```rust
// New Token variants (19 total new variants):

// Control flow keywords (13):
For,      // `for`
From,     // `from`
To,       // `to`
By,       // `by`
Do,       // `do`
Od,       // `od`
While,    // `while`
If,       // `if`
Then,     // `then`
Elif,     // `elif`
Else,     // `else`
Fi,       // `fi`

// Boolean keywords (3):
And,      // `and`
Or,       // `or`
Not,      // `not`

// Comparison operators (6):
Equal,    // `=`
NotEqual, // `<>`
Less,     // `<`
Greater,  // `>`
LessEq,   // `<=`
GreaterEq,// `>=`

// Total new variants: 22 (13 keyword + 3 boolean + 6 comparison)
```

### Anti-Patterns to Avoid

- **Treating for/if as statements, not expressions:** In Maple, `if`/`for` return values. Even though Phase 41 only parses (not evaluates), the AST should model them as expression-level nodes (AstNode variants), not as a separate Statement enum. This keeps the Pratt parser's single `expr_bp` entry point clean.

- **Adding `=` as an alias for `:=`:** The bare `=` is comparison only. The existing `:=` (Token::Assign) is for assignment. These must NEVER be conflated. The lexer already handles `:` vs `:=` greedy matching; `=` alone is always comparison.

- **Hard-coding `from 1` in the parser:** The parser should store `None` for an omitted `from` clause and let the evaluator (Phase 42) supply the default. Alternatively, the parser can desugar to `AstNode::Integer(1)` as a default -- either approach works, but be explicit and consistent.

- **Making comparisons associative:** In Maple, `a < b < c` is NOT valid (comparisons are non-associative). The parser should either reject chained comparisons or treat them as non-associative (same l_bp and r_bp, which naturally prevents chaining in a Pratt parser when l_bp < r_bp is false for the same operator).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| N/A -- this is all hand-written parser | N/A | N/A | This project uses a hand-written Pratt parser by design (Phase 24 decision). No parser generators or external libraries. All changes are internal modifications to existing Rust source files. |

**Key insight:** This phase is purely an extension of existing hand-written infrastructure. There are no deceptively complex sub-problems that warrant external libraries. The Pratt parser pattern is well-understood and the existing codebase demonstrates it cleanly.

## Common Pitfalls

### Pitfall 1: Keyword-Identifier Collision
**What goes wrong:** New keywords like `for`, `if`, `to`, `from`, `by`, `and`, `or`, `not` were previously valid identifiers (variable names). After making them keywords, any user who has a variable named `for` or `to` will get a parse error.
**Why it happens:** The lexer eagerly matches keywords before identifiers.
**How to avoid:** This is acceptable and matches Maple behavior (these are reserved words in Maple too). Document that `for`, `if`, `then`, `else`, `elif`, `fi`, `do`, `od`, `from`, `to`, `by`, `while`, `and`, `or`, `not` are now reserved keywords. Users who have variables with these names must rename them.
**Warning signs:** Existing test scripts that use `to` or `from` as variable names will break.

### Pitfall 2: Semicolons Inside Loop/If Bodies
**What goes wrong:** The current `parse_line()` treats `;` and `:` as top-level statement terminators and stops parsing. Inside a for-loop or if-block body, `;` separates statements but does NOT end the overall construct.
**Why it happens:** The existing parser has no concept of nested statement sequences.
**How to avoid:** The new `parse_stmt_sequence` method must be aware of terminating keywords (od, elif, else, fi) and treat `;`/`:` as intra-body separators, not as signals to return to the top level. The outer `parse_line()` should recognize `for` and `if` tokens and delegate to `expr_bp(0)` which handles them as prefix atoms that internally consume the full `for...od` or `if...fi` block.
**Warning signs:** `for n from 1 to 3 do n; od` fails to parse because `;` prematurely terminates.

### Pitfall 3: Token::Colon Ambiguity with Statement Suppression
**What goes wrong:** `:` is currently used as both a statement terminator (suppress output) and part of `:=` (assignment). Inside loop/if bodies, `:` as a statement terminator must not be confused with the end of the entire for/if construct.
**Why it happens:** The parser expects `:` to be a top-level terminator.
**How to avoid:** Inside `parse_stmt_sequence`, `:` is consumed as a statement terminator (like `;`). The loop only breaks when it sees the terminating keyword token (od, elif, else, fi). This is safe because `:` is never a keyword that could end a block.
**Warning signs:** `for n from 1 to 3 do x := n: od` fails because `:` is misinterpreted.

### Pitfall 4: Equal Sign Disambiguation (= vs :=)
**What goes wrong:** Adding `=` as Token::Equal could conflict with `:=` (Token::Assign).
**Why it happens:** The lexer processes `:=` greedy-first (the `:` handler already checks for `=` after `:`). A bare `=` is a separate character.
**How to avoid:** The lexer already handles `:` by checking if the next char is `=` (producing Token::Assign) or not (producing Token::Colon). A bare `=` (not preceded by `:`) is always Token::Equal. The lexer processes `<` before `=`, so `<=` is handled by the `<` handler. The `=` handler only fires for standalone `=`. This is clean and unambiguous.
**Warning signs:** None if implemented in the correct order: handle `<` (with `<=`, `<>` lookahead), handle `:` (with `:=` lookahead), then handle `=` standalone.

### Pitfall 5: REPL Multiline Detection
**What goes wrong:** In the REPL, typing `for n from 1 to 3 do` on one line should trigger multi-line mode (the user hasn't typed `od` yet), but `is_incomplete()` currently only counts brackets `(` and `[`.
**Why it happens:** The bracket-counting logic doesn't know about keyword-delimited blocks.
**How to avoid:** Extend `is_incomplete()` to track for/do/od depth and if/fi depth. Count `for`/`do` as openers, `od` as closers. Count `if` as opener, `fi` as closer. This requires a lightweight keyword scan (not full tokenization -- just word matching on whitespace-split tokens or a simple state machine).
**Warning signs:** REPL doesn't wait for `od`/`fi` -- user gets parse error instead of continuation prompt.

### Pitfall 6: Binding Power Renumbering Breaks Existing Tests
**What goes wrong:** Renumbering binding powers changes internal numeric values but should NOT change parse results. If any test checks internal bp values directly, it would break.
**Why it happens:** Tests might reference specific bp numbers.
**How to avoid:** Verify that NO tests check binding power numbers directly -- they all test parse output (AST structure), which is correct. The existing test suite should pass unchanged after renumbering as long as relative ordering is preserved.
**Warning signs:** Grep for literal bp values in tests (e.g., `expr_bp(7)` references in test code). Current tests only use `parse()` and `parse_expr()` helpers, which call `expr_bp(0)`, so this is safe.

## Code Examples

### New Token Variants (token.rs)
```rust
// Source: Existing codebase pattern (token.rs line 8-50)
// Add after existing variants, before Eof:

/// `for` keyword.
For,
/// `from` keyword.
From,
/// `to` keyword.
To,
/// `by` keyword.
By,
/// `do` keyword (opens loop body).
Do,
/// `od` keyword (closes loop body).
Od,
/// `while` keyword.
While,
/// `if` keyword.
If,
/// `then` keyword.
Then,
/// `elif` keyword.
Elif,
/// `else` keyword.
Else,
/// `fi` keyword (closes conditional).
Fi,
/// `and` boolean operator.
And,
/// `or` boolean operator.
Or,
/// `not` boolean operator.
Not,
/// `=` comparison (equality).
Equal,
/// `<>` comparison (not equal).
NotEqual,
/// `<` comparison (less than).
Less,
/// `>` comparison (greater than).
Greater,
/// `<=` comparison (less or equal).
LessEq,
/// `>=` comparison (greater or equal).
GreaterEq,
```

### Updated token_name (parser.rs)
```rust
// Source: Existing codebase pattern (parser.rs line 320-343)
// Add entries for all new token variants:
Token::For => "'for'".to_string(),
Token::From => "'from'".to_string(),
Token::To => "'to'".to_string(),
Token::By => "'by'".to_string(),
Token::Do => "'do'".to_string(),
Token::Od => "'od'".to_string(),
Token::While => "'while'".to_string(),
Token::If => "'if'".to_string(),
Token::Then => "'then'".to_string(),
Token::Elif => "'elif'".to_string(),
Token::Else => "'else'".to_string(),
Token::Fi => "'fi'".to_string(),
Token::And => "'and'".to_string(),
Token::Or => "'or'".to_string(),
Token::Not => "'not'".to_string(),
Token::Equal => "'='".to_string(),
Token::NotEqual => "'<>'".to_string(),
Token::Less => "'<'".to_string(),
Token::Greater => "'>'".to_string(),
Token::LessEq => "'<='".to_string(),
Token::GreaterEq => "'>='".to_string(),
```

### Test for For-Loop Parsing
```rust
#[test]
fn test_for_loop_basic() {
    let node = parse_expr("for n from 1 to 5 do n od");
    assert!(matches!(node, AstNode::ForLoop { .. }));
    if let AstNode::ForLoop { var, from, to, by, body } = &node {
        assert_eq!(var, "n");
        assert_eq!(*from, Box::new(AstNode::Integer(1)));
        assert_eq!(*to, Box::new(AstNode::Integer(5)));
        assert!(by.is_none());
        assert_eq!(body.len(), 1);
    }
}
```

### Test for If-Conditional Parsing
```rust
#[test]
fn test_if_elif_else() {
    let stmts = parse("if x > 0 then 1 elif x = 0 then 0 else -1 fi").unwrap();
    assert_eq!(stmts.len(), 1);
    if let AstNode::IfExpr { condition, then_body, elif_branches, else_body } = &stmts[0].node {
        assert!(elif_branches.len() == 1);
        assert!(else_body.is_some());
    } else {
        panic!("Expected IfExpr");
    }
}
```

### Test for Comparison Operators
```rust
#[test]
fn test_comparison_less_than() {
    let node = parse_expr("x < 5");
    assert_eq!(
        node,
        AstNode::Compare {
            op: CompOp::Less,
            lhs: Box::new(AstNode::Variable("x".to_string())),
            rhs: Box::new(AstNode::Integer(5)),
        }
    );
}

#[test]
fn test_comparison_not_equal() {
    let node = parse_expr("x <> 0");
    assert_eq!(
        node,
        AstNode::Compare {
            op: CompOp::NotEq,
            lhs: Box::new(AstNode::Variable("x".to_string())),
            rhs: Box::new(AstNode::Integer(0)),
        }
    );
}
```

### Test for Boolean Operators and Precedence
```rust
#[test]
fn test_boolean_and_or_precedence() {
    // "a > 0 and b < 10 or c = 5" should parse as "(a>0 and b<10) or (c=5)"
    // because 'and' binds tighter than 'or'
    let node = parse_expr("a > 0 and b < 10 or c = 5");
    if let AstNode::BoolOp { op: BoolBinOp::Or, lhs, rhs } = &node {
        assert!(matches!(lhs.as_ref(), AstNode::BoolOp { op: BoolBinOp::And, .. }));
        assert!(matches!(rhs.as_ref(), AstNode::Compare { op: CompOp::Eq, .. }));
    } else {
        panic!("Expected BoolOp::Or at top level");
    }
}

#[test]
fn test_not_prefix() {
    let node = parse_expr("not x > 5");
    // 'not' binds looser than '>': not (x > 5)
    assert!(matches!(node, AstNode::Not(_)));
    if let AstNode::Not(inner) = &node {
        assert!(matches!(inner.as_ref(), AstNode::Compare { op: CompOp::Greater, .. }));
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Only arithmetic operators in Pratt parser | Add comparison + boolean operators | Phase 41 | Binding power table grows from 5 to 11 operator levels |
| No statement-level constructs in parser | for/if parsed as prefix atoms returning AstNode | Phase 41 | Parser gains `parse_stmt_sequence` for nested blocks |
| `is_incomplete` only counts `(`/`[` | Must also track `for..od` and `if..fi` nesting | Phase 41 | REPL multi-line detection works for control flow |
| Keywords: only `infinity` | 16 new keywords: for/from/to/by/do/od/while/if/then/elif/else/fi/and/or/not + infinity | Phase 41 | Variable names `for`, `if`, `to`, etc. become reserved |

## Open Questions

1. **Should `from` be required or optional?**
   - What we know: Maple makes `from` optional (defaults to 1). The phase description uses `for n from 1 to 8` (with explicit from).
   - What's unclear: Whether to require `from` for simplicity or match Maple's optional behavior.
   - Recommendation: Make `from` optional (default to `AstNode::Integer(1)` in parser). This matches Maple and is straightforward to implement. The additional parser logic is minimal (just an `if *self.peek() == Token::From` check).

2. **Should `by` clause be supported in Phase 41?**
   - What we know: Maple supports `by` for step size. The phase description doesn't mention it explicitly.
   - What's unclear: Whether `by` is in scope for Phase 41.
   - Recommendation: Add `by` as an optional AST field (parsed but not evaluated until Phase 42). The lexer/token overhead is minimal, and adding it later would require touching the same code again.

3. **Should `while` clause in for-loops be supported?**
   - What we know: Maple supports `for n from 1 to 10 while cond do ... od`. The phase requirements don't mention it.
   - What's unclear: Whether this is in scope.
   - Recommendation: Add the `while` token to the lexer but do NOT parse it in for-loops yet. This reserves the keyword without implementing the feature. If someone uses `while` as a variable name, it will become reserved -- better to do this now than later.

4. **Non-associativity of comparison operators**
   - What we know: Maple comparisons are non-associative. `a < b < c` is a syntax error in Maple.
   - What's unclear: Whether our parser should reject this or silently parse left-to-right.
   - Recommendation: Make comparisons non-associative by giving them the same l_bp and r_bp (e.g., both 9). In a Pratt parser with `l_bp < min_bp` as the break condition, equal l_bp and min_bp causes a break, preventing chaining. This is the cleanest approach.

5. **Should `end do` / `end if` alternatives be supported?**
   - What we know: Maple supports both `od`/`fi` and `end do`/`end if`. The phase description only mentions `od` and `fi`.
   - What's unclear: Whether to support the `end` keyword alternatives.
   - Recommendation: Only support `od` and `fi` in Phase 41. `end do` and `end if` require lookahead from `end` and add complexity. Can be added later if needed. Reserve the `end` keyword token anyway for future use.

## Sources

### Primary (HIGH confidence)
- **Codebase analysis** -- Direct reading of all source files in `crates/qsym-cli/src/` (token.rs, ast.rs, parser.rs, lexer.rs, eval.rs, error.rs, repl.rs, script.rs, environment.rs, lib.rs)
- [Maple operator precedence](https://www.maplesoft.com/support/help/maple/view.aspx?path=operators/precedence) -- Official Maple precedence table confirming: arithmetic > comparison > not > and > or
- [Maple do/od syntax](https://www.maplesoft.com/support/help/Maple/view.aspx?path=do) -- Official Maple for-loop syntax with all optional clauses
- [Maple if/fi syntax](https://www.maplesoft.com/support/help/maple/view.aspx?path=if) -- Official Maple conditional syntax with elif chains

### Secondary (MEDIUM confidence)
- [Maple conditional programming](https://www.maplesoft.com/applications/view.aspx?SID=1559&view=html) -- Application Center tutorial on if/elif/else/fi usage patterns
- [Maple loop examples](https://www.maplesoft.com/applications/view.aspx?SID=1558&view=html) -- Application Center tutorial on for-loop variants

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- Direct analysis of existing codebase, all files read in full
- Architecture: HIGH -- Pratt parser extension patterns are well-established; binding power renumbering is mechanical
- Pitfalls: HIGH -- Based on direct code analysis of parser.rs, lexer.rs, and repl.rs; identified all interaction points
- Operator precedence: HIGH -- Verified against official Maple documentation

**Research date:** 2026-02-20
**Valid until:** 2026-03-20 (stable -- internal codebase patterns, no external dependencies)
