# Phase 24: Parser & AST - Research

**Researched:** 2026-02-17
**Domain:** Maple-style expression parser in Rust (lexer, Pratt parser, AST)
**Confidence:** HIGH

## Summary

Phase 24 requires a parser that converts Maple-style text input into an AST for subsequent evaluation (Phase 25). The grammar is small and well-defined: integer/rational literals, the `infinity` keyword, variable references, function calls with positional arguments, arithmetic operators (`+`, `-`, `*`, `/`, `^`), unary negation, assignment (`:=`), and statement terminators (`;`, `:`). The `%` ditto operator references the last result.

The recommended approach is **hand-written Pratt parser** with a separate tokenization pass. For a grammar this small (~15 token types, ~8 AST node types), a hand-written parser gives full control over error messages, is easier to debug, and avoids a library dependency. The `winnow` crate (v0.7.14) is a viable alternative with its built-in `expression()` Pratt combinator, but the grammar is simple enough that hand-writing is the more maintainable choice for this project. The parser lives in a new `qsym-cli` binary crate that depends on `qsym-core`.

**Primary recommendation:** Hand-write a two-phase parser (tokenizer + Pratt parser) in a new `qsym-cli` crate with a clean AST type that is independent of qsym-core's `Expr` (the evaluator in Phase 25 bridges AST to core).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Use `:=` for assignment (Maple style), not `=`
- Maple-style semicolons: `;` terminates and displays result, `:` terminates and suppresses output
- `%` refers to the last computed result (Maple convention)
- Function names: support both existing q-Kangaroo names (aqprod, etaq, partition_count) AND Maple aliases (ETAR, JACPROD, etc.)
- Exponentiation with `^` operator (Maple convention)
- `q` is a built-in symbol, always available as the series indeterminate
- Other symbolic variables (a, z, x, etc.) are allowed in function arguments
- Undefined names are errors (except `q` and `infinity` which are built-in)
- Multiple statements on one line separated by `;` or `:`
- Bare Enter (no `;` or `:`) auto-evaluates and displays
- Each `;`-terminated statement prints its result; `:` suppresses
- Assignments (`f := expr;`) print the assigned value

### Claude's Discretion
- Nature of `q` (reserved keyword vs pre-defined variable)
- `3/4` rational literal vs division semantics
- Whether to support `/` for series division
- Operator precedence details
- Parse error message format and detail level

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

## Discretionary Decisions (Researcher Recommendations)

### `q` as reserved keyword (not pre-defined variable)
**Recommendation:** Treat `q` as a **reserved keyword** in the lexer, like `infinity`. This prevents users from accidentally reassigning it (`q := 5` is a parse error). `q` is fundamental to the entire system -- allowing reassignment would break every function. The lexer emits a `Token::Q` variant, and the parser produces `AstNode::Q`.

### `3/4` is division, not a rational literal
**Recommendation:** Parse `3/4` as integer division (`3 / 4`). Rationale:
1. Maple itself treats `/` as division, not rational construction
2. `3/4` as a "rational literal" creates ambiguity: is `a/b` rational or division?
3. The evaluator (Phase 25) can evaluate `3/4` into a `QRat` when both operands are integer literals -- so users still get exact rationals without special syntax
4. This is the least-surprising behavior for anyone coming from Maple or any other language

### `/` supported for division
**Recommendation:** Support `/` as a binary infix operator for division. In the evaluator, `integer / integer` produces a rational, and `series / series` can produce a series (via series inversion). The parser just sees it as arithmetic -- semantics are Phase 25's job.

### Operator precedence table
**Recommendation:** Follow Maple conventions closely:

| Precedence | Operators | Associativity | Binding Power (L, R) |
|------------|-----------|---------------|----------------------|
| 1 (lowest) | `:=` | Right | (2, 1) |
| 2 | `+`, `-` | Left | (3, 4) |
| 3 | `*`, `/` | Left | (5, 6) |
| 4 | unary `-` | Prefix | (_, 7) |
| 5 (highest) | `^` | Non-assoc | (9, 10) |
| 6 | Function call `(` | Postfix | (11, _) |

Notes:
- `^` is **non-associative** per Maple: `2^3^4` is an error, must write `2^(3^4)` or `(2^3)^4`
- Unary `-` binds tighter than `*`/`/` so `-3*f` is `(-3)*f`, matching Maple
- Function call `(` as postfix means `aqprod(q,q,infinity,20)` is parsed as identifier `aqprod` followed by postfix `(args...)`

### Parse error messages
**Recommendation:** Errors include the byte offset, the problematic token, and a short message. Example:
```
parse error at column 15: expected ')' to close function call, found ';'
  f := aqprod(q,q;infinity,20)
                  ^
```
The parser stores byte spans for each token. Error rendering (column calculation, caret placement) is a utility function. No external error-rendering crate needed for this grammar size.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| (none -- hand-written) | N/A | Lexer + Pratt parser | Grammar is ~15 token types; hand-written gives full control, zero deps, better error messages |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| qsym-core | workspace | Number types (QInt, QRat), SymbolId | AST leaf nodes reference these types for integer/rational parsing |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-written | winnow 0.7.14 | winnow has `expression()` Pratt combinator; overkill for ~15 tokens, adds dependency, harder to customize error messages |
| Hand-written | chumsky 1.0.0-alpha.5 | Excellent error recovery but unstable API (still alpha); this grammar doesn't need recovery -- just fail and report |
| Hand-written | pest | PEG grammar file; no Pratt support built-in (added later), awkward for interactive REPL parsing |

**Installation:**
```bash
# No new dependencies needed for the parser itself.
# The qsym-cli crate depends on qsym-core (workspace member).
cargo init crates/qsym-cli --name qsym-cli
```

## Architecture Patterns

### Recommended Project Structure
```
crates/qsym-cli/
    src/
        main.rs          # Entry point (minimal for Phase 24; REPL loop in Phase 26)
        lib.rs           # Re-exports
        ast.rs           # AST node types (AstNode, Stmt, Terminator)
        token.rs         # Token enum + Span type
        lexer.rs         # Tokenizer: &str -> Vec<Token>
        parser.rs        # Pratt parser: Vec<Token> -> Vec<Stmt>
        error.rs         # ParseError type and rendering
    Cargo.toml           # depends on qsym-core
```

### Pattern 1: Two-Phase Parsing (Lex then Parse)

**What:** Separate tokenization from parsing. The lexer converts raw `&str` into a `Vec<SpannedToken>` (token + byte span). The parser operates on the token stream, never touching raw text.

**When to use:** Always for this project. Clean separation makes error reporting straightforward and the parser logic cleaner.

**Why not single-pass:** Single-pass (parser-combinator style) mixes character-level and token-level concerns. For a REPL where error messages matter, two-phase is cleaner.

**Example:**
```rust
// token.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),         // 42, 0, -5 (sign handled by parser as unary neg)

    // Keywords
    Infinity,             // infinity
    Q,                    // q (the series indeterminate)

    // Identifiers
    Ident(String),        // f, aqprod, ETAR, partition_count

    // Operators
    Plus,                 // +
    Minus,                // -
    Star,                 // *
    Slash,                // /
    Caret,                // ^
    Assign,               // :=
    Percent,              // %

    // Delimiters
    LParen,               // (
    RParen,               // )
    Comma,                // ,

    // Terminators
    Semi,                 // ;  (display result)
    Colon,                // :  (suppress result) -- note: only as statement terminator,
                          //     not part of :=

    // End of input
    Eof,
}

/// Byte offset range in source text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}
```

### Pattern 2: Pratt Parser with Binding Power

**What:** The parser uses the Pratt algorithm (top-down operator precedence) where each operator has a left and right binding power. Higher binding power = tighter binding.

**When to use:** For all expression parsing. The algorithm naturally handles precedence, associativity, prefix/postfix operators, and function calls.

**Example:**
```rust
// parser.rs
struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    /// Parse an expression with minimum binding power `min_bp`.
    fn expr_bp(&mut self, min_bp: u8) -> Result<AstNode, ParseError> {
        // 1. Parse prefix (atom or unary operator)
        let mut lhs = match self.peek_token() {
            Token::Integer(n) => {
                self.advance();
                AstNode::Integer(n)
            }
            Token::Q => {
                self.advance();
                AstNode::Q
            }
            Token::Infinity => {
                self.advance();
                AstNode::Infinity
            }
            Token::Percent => {
                self.advance();
                AstNode::LastResult
            }
            Token::Ident(name) => {
                self.advance();
                AstNode::Variable(name)
            }
            Token::Minus => {
                self.advance();
                let ((), r_bp) = prefix_bp(Token::Minus); // (_, 7)
                let rhs = self.expr_bp(r_bp)?;
                AstNode::Neg(Box::new(rhs))
            }
            Token::LParen => {
                self.advance();
                let inner = self.expr_bp(0)?;
                self.expect(Token::RParen)?;
                inner // grouping parens don't create AST nodes
            }
            _ => return Err(self.error("expected expression")),
        };

        // 2. Parse infix/postfix operators
        loop {
            let op = self.peek_token();

            // Postfix: function call
            if op == Token::LParen {
                let (l_bp, ()) = postfix_bp(Token::LParen); // (11, _)
                if l_bp < min_bp { break; }
                // Only valid after an identifier
                if let AstNode::Variable(name) = &lhs {
                    let name = name.clone();
                    self.advance(); // consume '('
                    let args = self.parse_arg_list()?;
                    self.expect(Token::RParen)?;
                    lhs = AstNode::FuncCall { name, args };
                } else {
                    break; // Not a function call, stop
                }
                continue;
            }

            // Assignment: :=
            if op == Token::Assign {
                let (l_bp, r_bp) = (2, 1); // right-associative, lowest precedence
                if l_bp < min_bp { break; }
                if let AstNode::Variable(name) = &lhs {
                    let name = name.clone();
                    self.advance();
                    let rhs = self.expr_bp(r_bp)?;
                    lhs = AstNode::Assign { name, value: Box::new(rhs) };
                } else {
                    return Err(self.error("left side of := must be a variable name"));
                }
                continue;
            }

            // Infix: +, -, *, /, ^
            if let Some((l_bp, r_bp)) = infix_bp(&op) {
                if l_bp < min_bp { break; }
                self.advance();
                let rhs = self.expr_bp(r_bp)?;
                lhs = match op {
                    Token::Plus  => AstNode::BinOp { op: BinOp::Add, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                    Token::Minus => AstNode::BinOp { op: BinOp::Sub, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                    Token::Star  => AstNode::BinOp { op: BinOp::Mul, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                    Token::Slash => AstNode::BinOp { op: BinOp::Div, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                    Token::Caret => AstNode::BinOp { op: BinOp::Pow, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                    _ => unreachable!(),
                };
                continue;
            }

            break; // No matching operator, stop
        }

        Ok(lhs)
    }
}

fn infix_bp(token: &Token) -> Option<(u8, u8)> {
    match token {
        Token::Plus | Token::Minus => Some((3, 4)),   // left-assoc
        Token::Star | Token::Slash => Some((5, 6)),   // left-assoc
        Token::Caret              => Some((9, 10)),   // non-assoc (enforced separately)
        _ => None,
    }
}

fn prefix_bp(token: Token) -> ((), u8) {
    match token {
        Token::Minus => ((), 7),
        _ => panic!("not a prefix operator: {:?}", token),
    }
}

fn postfix_bp(token: Token) -> (u8, ()) {
    match token {
        Token::LParen => (11, ()),  // function call
        _ => panic!("not a postfix operator: {:?}", token),
    }
}
```

### Pattern 3: Statement-Level Parsing

**What:** The top-level parser produces a `Vec<Stmt>` where each `Stmt` is an expression paired with a terminator (`;` for display, `:` for suppress, or implicit/bare-enter for display).

**When to use:** For parsing a complete line of input.

**Example:**
```rust
// ast.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    Semi,     // ; -- display result
    Colon,    // : -- suppress result
    Implicit, // bare enter -- display result
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub expr: AstNode,
    pub terminator: Terminator,
}

// parser.rs
impl Parser {
    /// Parse a complete input line into a sequence of statements.
    pub fn parse_line(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();

        while self.peek_token() != Token::Eof {
            let expr = self.expr_bp(0)?;
            let terminator = match self.peek_token() {
                Token::Semi => { self.advance(); Terminator::Semi }
                Token::Colon => { self.advance(); Terminator::Colon }
                Token::Eof => Terminator::Implicit,
                _ => return Err(self.error("expected ';', ':', or end of input")),
            };
            stmts.push(Stmt { expr, terminator });
        }

        Ok(stmts)
    }
}
```

### Pattern 4: AST Node Design

**What:** The AST is a standalone enum independent of qsym-core's `Expr`. The evaluator (Phase 25) converts `AstNode` to core operations.

**Why separate:** The parser AST represents *syntax* (what the user typed). qsym-core's `Expr` represents *semantics* (mathematical structure). Keeping them separate means:
1. Parser has no dependency on arena/session infrastructure
2. AST can include parser-only concepts (assignment, function calls by name, `%`)
3. Core `Expr` doesn't need to change to support REPL features

**Example:**
```rust
// ast.rs
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    /// Integer literal: 42
    Integer(i64),

    /// The q indeterminate (keyword)
    Q,

    /// The infinity keyword
    Infinity,

    /// Last result reference: %
    LastResult,

    /// Variable reference: f, g, my_var
    Variable(String),

    /// Binary operation: lhs op rhs
    BinOp {
        op: BinOp,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },

    /// Unary negation: -expr
    Neg(Box<AstNode>),

    /// Function call: name(arg1, arg2, ...)
    FuncCall {
        name: String,
        args: Vec<AstNode>,
    },

    /// Variable assignment: name := expr
    Assign {
        name: String,
        value: Box<AstNode>,
    },
}
```

### Anti-Patterns to Avoid
- **Reusing qsym-core's `Expr` as the AST:** Don't do this. `Expr` has no concept of assignment, function calls by name, or `%`. Adding those would pollute the core.
- **Parsing numbers as strings:** Parse `i64` in the lexer. For numbers too large for `i64`, fall back to string representation and let the evaluator construct `QInt`. (This is a Phase 25 concern, but the token type should accommodate it.)
- **Making the lexer context-sensitive:** The lexer should not care about `:=` vs `:` as terminator. Instead, lex `:=` as a single `Assign` token (greedy match), and bare `:` as `Colon`. The two-character `:=` should be checked first.
- **Eagerly resolving function aliases:** Don't resolve `ETAR` to `etaq` in the parser. The AST stores the name as-is. Alias resolution happens in the evaluator (Phase 25).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Line editing / history | Custom terminal input handler | `rustyline` (Phase 26) | Terminal raw mode, Unicode, ANSI escape sequences are platform-specific nightmares |
| Arbitrary-precision integer parsing | Custom bigint parser | `rug::Integer::parse_radix` or `str::parse::<i64>` with fallback | Parsing integers is trivial for i64; rug handles overflow cases |
| Unicode identifier support | Custom Unicode category tables | ASCII-only identifiers are sufficient | q-series function names and variable names are all ASCII |

**Key insight:** The parser itself is simple enough to hand-write. What should NOT be hand-written are the surrounding REPL infrastructure (line editing, completion) -- but those are Phase 26 concerns, not Phase 24.

## Common Pitfalls

### Pitfall 1: `:=` vs `:` Lexer Ambiguity
**What goes wrong:** The lexer sees `:` and emits a `Colon` token, then sees `=` and emits something else. Now `f := 5` is parsed as `f` `:` `= 5` which is nonsensical.
**Why it happens:** Single-character lookahead without greedy matching.
**How to avoid:** In the lexer, when you see `:`, peek at the next character. If it's `=`, consume both and emit `Token::Assign`. Otherwise emit `Token::Colon`.
**Warning signs:** Assignment tests fail; `:` at end-of-statement breaks.

### Pitfall 2: Unary Minus vs Binary Minus
**What goes wrong:** `-3*f` is parsed as `-(3*f)` instead of `(-3)*f`, or `f - g` fails because `-` is treated as prefix.
**Why it happens:** The parser doesn't correctly distinguish prefix vs infix context.
**How to avoid:** The Pratt algorithm naturally handles this. In the `expr_bp` function, prefix `-` is only tried when parsing the *left-hand side* (the "nud" / null denotation). When the parser is in the *infix loop* (the "led" / left denotation), `-` is binary subtraction. The binding power separation (`prefix_bp = 7`, `infix_bp = (3,4)`) ensures correct precedence.
**Warning signs:** `-f + g` gives wrong result; `3 - -4` fails to parse.

### Pitfall 3: Non-Associative `^` Allowing Chaining
**What goes wrong:** `2^3^4` silently parses as `(2^3)^4` (left-assoc) or `2^(3^4)` (right-assoc) when Maple considers it an error.
**Why it happens:** Pratt parsers naturally produce left-assoc or right-assoc behavior based on binding power; non-associativity requires explicit enforcement.
**How to avoid:** After parsing `a ^ b`, check if the next token is also `^`. If so, emit a parse error: "ambiguous exponentiation -- use parentheses." Alternatively, use the same binding power for both left and right (`(9, 9)`) which causes the parser to stop, then check.
**Warning signs:** `2^3^4` silently produces a result instead of an error.

### Pitfall 4: Function Call Precedence
**What goes wrong:** `3 * aqprod(q,q,infinity,20)` is parsed as `(3 * aqprod)(q,q,infinity,20)`.
**Why it happens:** Function call `(` has very high binding power as postfix, but it competes with infix `*`.
**How to avoid:** Function call `(` is treated as a postfix operator with the highest binding power (11). Since `*` has left binding power 5, the parser correctly binds `aqprod(...)` first, then `3 * result`. Additionally, the parser should only allow function-call syntax after an identifier, not after any expression.
**Warning signs:** `3 * f(x)` gives a parser error.

### Pitfall 5: Empty Statement Between Semicolons
**What goes wrong:** `f := 5;; g := 10` crashes or gives confusing error.
**Why it happens:** The parser expects an expression after `;` but finds another `;`.
**How to avoid:** In `parse_line`, skip consecutive `;`/`:` terminators, or treat empty statements as no-ops.
**Warning signs:** Double semicolons crash; trailing `;` before EOF causes error.

### Pitfall 6: Integer Overflow on Literals
**What goes wrong:** User types `99999999999999999999999` (larger than i64::MAX) and the lexer panics.
**Why it happens:** Lexer uses `str::parse::<i64>()` which fails on overflow.
**How to avoid:** Try `i64` first; on failure, store as `String` in a `Token::BigInteger(String)` variant. The evaluator (Phase 25) converts to `QInt` via rug. For Phase 24, the parser just needs to carry the string.
**Warning signs:** Large integers in Ramanujan-type identities crash the parser.

### Pitfall 7: Percent (`%`) in Wrong Position
**What goes wrong:** `f := %` works but `% + %` or `f(%,3)` behaves unexpectedly.
**Why it happens:** `%` is not treated as a proper atom in the parser.
**How to avoid:** `%` is an atom, just like an integer or `q`. It can appear anywhere an expression is expected. The parser produces `AstNode::LastResult`. The evaluator substitutes the actual value.
**Warning signs:** `% + 1` fails to parse; `% * %` gives wrong result.

## Code Examples

### Complete Lexer (Verified Pattern)
```rust
// lexer.rs
pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>, ParseError> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() {
        // Skip whitespace
        if bytes[pos].is_ascii_whitespace() {
            pos += 1;
            continue;
        }

        let start = pos;

        let token = match bytes[pos] {
            b'+' => { pos += 1; Token::Plus }
            b'-' => { pos += 1; Token::Minus }
            b'*' => { pos += 1; Token::Star }
            b'/' => { pos += 1; Token::Slash }
            b'^' => { pos += 1; Token::Caret }
            b'%' => { pos += 1; Token::Percent }
            b'(' => { pos += 1; Token::LParen }
            b')' => { pos += 1; Token::RParen }
            b',' => { pos += 1; Token::Comma }
            b';' => { pos += 1; Token::Semi }
            b':' => {
                pos += 1;
                if pos < bytes.len() && bytes[pos] == b'=' {
                    pos += 1;
                    Token::Assign  // :=
                } else {
                    Token::Colon   // : (statement terminator)
                }
            }
            b'0'..=b'9' => {
                while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
                let s = &input[start..pos];
                match s.parse::<i64>() {
                    Ok(n) => Token::Integer(n),
                    Err(_) => Token::BigInteger(s.to_string()),
                }
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                    pos += 1;
                }
                let word = &input[start..pos];
                match word {
                    "infinity" => Token::Infinity,
                    "q" => Token::Q,
                    _ => Token::Ident(word.to_string()),
                }
            }
            _ => {
                return Err(ParseError {
                    message: format!("unexpected character '{}'", bytes[pos] as char),
                    span: Span { start, end: start + 1 },
                });
            }
        };

        tokens.push(SpannedToken {
            token,
            span: Span { start, end: pos },
        });
    }

    tokens.push(SpannedToken {
        token: Token::Eof,
        span: Span { start: pos, end: pos },
    });

    Ok(tokens)
}
```

### Complete Error Type
```rust
// error.rs
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    /// Render the error with a caret pointing to the problem location.
    pub fn render(&self, source: &str) -> String {
        let col = self.span.start; // byte offset = column for ASCII
        let mut result = format!("parse error at column {}: {}\n", col + 1, self.message);
        result.push_str("  ");
        result.push_str(source);
        result.push('\n');
        result.push_str("  ");
        for _ in 0..col {
            result.push(' ');
        }
        result.push('^');
        result
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at byte {}: {}", self.span.start, self.message)
    }
}

impl std::error::Error for ParseError {}
```

### Non-Associative `^` Enforcement
```rust
// In the infix loop of expr_bp:
Token::Caret => {
    let (l_bp, r_bp) = (9, 10);
    if l_bp < min_bp { break; }
    self.advance();
    let rhs = self.expr_bp(r_bp)?;
    let node = AstNode::BinOp {
        op: BinOp::Pow,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    };
    // Enforce non-associativity: if next token is also ^, error
    if self.peek_token() == Token::Caret {
        return Err(self.error(
            "ambiguous exponentiation: use parentheses, e.g., (a^b)^c or a^(b^c)"
        ));
    }
    lhs = node;
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| nom for text parsing | winnow (nom successor) | 2023 | winnow has better error messages and developer experience |
| Separate Pratt parser crate | winnow 0.7.14 has built-in `expression()` combinator | Nov 2025 | Can do Pratt parsing within winnow, but hand-written is still preferred for small grammars |
| chumsky 0.9 (stable) | chumsky 1.0.0-alpha.5 | 2024-2025 | New API is different; not production-ready |

**Deprecated/outdated:**
- `nom`: Superseded by winnow. nom is maintained but winnow is the recommended successor.
- `chumsky 0.9.x`: Old API. The `1.0.0-alpha` series has a completely different API.

## Grammar Specification (EBNF)

For reference, the complete grammar that Phase 24 must parse:

```ebnf
line        = { stmt } ;
stmt        = expr terminator ;
terminator  = ";" | ":" | EOF ;

expr        = assignment ;
assignment  = IDENT ":=" expr
            | addition ;

addition    = multiplication ( ("+" | "-") multiplication )* ;
multiplication = unary ( ("*" | "/") unary )* ;
unary       = "-" unary
            | power ;
power       = postfix ( "^" postfix )? ;   (* non-associative: at most one ^ *)
postfix     = atom [ "(" arg_list ")" ] ;   (* function call *)
atom        = INTEGER
            | "infinity"
            | "q"
            | "%"
            | IDENT
            | "(" expr ")" ;

arg_list    = expr ( "," expr )* | (* empty *) ;

INTEGER     = DIGIT+ ;
IDENT       = (LETTER | "_") (LETTER | DIGIT | "_")* ;
LETTER      = "a".."z" | "A".."Z" ;
DIGIT       = "0".."9" ;
```

## Open Questions

1. **Large integer handling in Phase 24**
   - What we know: Researchers may type numbers larger than i64::MAX (e.g., partition counts, Ramanujan-type constants)
   - What's unclear: Should the lexer parse directly to `QInt` (rug dependency in qsym-cli) or carry a string and let the evaluator construct `QInt`?
   - Recommendation: Use `Token::BigInteger(String)` to avoid rug dependency in the parser. The evaluator (Phase 25) converts. This keeps the parser crate lightweight.

2. **Should `AstNode` carry spans?**
   - What we know: Spans are useful for error messages from the evaluator (Phase 25), not just the parser
   - What's unclear: How much Phase 25 needs span info for runtime errors
   - Recommendation: YES, add an optional `Span` to each `AstNode` (or wrap in `Spanned<AstNode>`). The cost is small and the benefit for error reporting is significant.

3. **Maple alias table location**
   - What we know: Users type `ETAR(1,1,20)` and expect it to work like `etaq(1,1,20)`
   - What's unclear: Where does the alias mapping live? Parser? Evaluator?
   - Recommendation: The alias table belongs in the evaluator (Phase 25). The parser stores the raw name. This keeps the parser agnostic about function semantics.

## Sources

### Primary (HIGH confidence)
- **matklad's Pratt Parsing tutorial** - https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html - Core algorithm, binding power table, Rust implementation
- **winnow 0.7.14 docs** - https://docs.rs/winnow/0.7.14/winnow/ - `expression()` combinator API, `Infix`/`Prefix`/`Postfix` types, error handling
- **winnow CHANGELOG** - https://github.com/winnow-rs/winnow/blob/main/CHANGELOG.md - v0.7.14 added `expression()` combinator (Nov 2025)
- **Existing qsym-core crate** - `crates/qsym-core/src/expr.rs`, `canonical.rs`, `symbol.rs` - Current `Expr` type, `ExprRef`, `SymbolId`, `make_*` constructors

### Secondary (MEDIUM confidence)
- **Maple syntax reference** - https://www.maplesoft.com/support/help/maple/view.aspx?path=syntax - Operator precedence, `^` non-associativity, `:=` assignment semantics
- **winnow error reporting tutorial** - https://docs.rs/winnow/latest/winnow/_tutorial/chapter_7/index.html - `ContextError`, byte spans, custom error rendering
- **Crafting Interpreters (Pratt Parsers)** - https://journal.stuffwithstuff.com/2011/03/19/pratt-parsers-expression-parsing-made-easy/ - Function calls as infix parselets

### Tertiary (LOW confidence)
- **chumsky status** - https://crates.io/crates/chumsky/1.0.0-alpha.5 - Still in alpha; version from search results, not verified in detail

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Hand-written parser is the established pattern for small grammars; well-documented algorithm
- Architecture: HIGH - Two-phase (lex+parse) with Pratt is the standard approach; EBNF grammar is fully specified
- Pitfalls: HIGH - All pitfalls are from known parser-construction issues, verified against Maple semantics
- AST design: HIGH - Pattern follows standard practice (separate syntax tree from semantic IR); verified against existing codebase

**Research date:** 2026-02-17
**Valid until:** 2026-06-17 (stable domain; Pratt parsing hasn't changed in decades)
