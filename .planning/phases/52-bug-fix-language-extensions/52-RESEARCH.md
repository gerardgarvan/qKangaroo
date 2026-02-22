# Phase 52: Bug Fix & Language Extensions - Research

**Researched:** 2026-02-21
**Domain:** CLI evaluator bug fix (division hang) + language extensions (while, print, Unicode)
**Confidence:** HIGH

## Summary

This phase addresses four requirements: a critical bug where dividing by exact polynomials (POLYNOMIAL_ORDER sentinel = 1 billion) causes the invert loop to hang; adding `while...do...od` loop support; adding a `print()` built-in function for intermediate output; and making the lexer resilient to pasted Unicode operator characters.

The bug is well-understood and localized. The `arithmetic::invert()` function in `qsym-core` iterates from 1 to `truncation_order`. When a 3-arg `aqprod(q,q,N)` produces an FPS with `truncation_order = POLYNOMIAL_ORDER = 1_000_000_000`, any division involving that series triggers a loop of 1 billion iterations. The fix must cap the effective truncation order before calling `invert()`. The three language features are straightforward extensions of the existing Pratt parser and AST-walking evaluator infrastructure.

**Primary recommendation:** Fix the POLYNOMIAL_ORDER division bug by capping truncation_order in `eval_div` before calling `arithmetic::invert`; use `env.default_order` (currently 20) as the fallback. Add while-loop support by adding `AstNode::WhileLoop` and mirroring the for-loop pattern. Add `print()` as a special-case in eval (like RETURN/subs). Add Unicode normalization at the start of `tokenize()`.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| BUG-01 | Division by exact polynomial (POLYNOMIAL_ORDER sentinel) completes in bounded time | Root cause identified: `arithmetic::invert()` loops to `truncation_order` (1B). Fix: cap to `min(a.trunc, b.trunc)` with POLYNOMIAL_ORDER replaced by `env.default_order` in eval_div |
| LANG-01 | `while...do...od` loops execute with boolean/comparison conditions | Token::While already defined in token.rs line 67, lexer maps "while" at line 209. Need: AstNode::WhileLoop, parser case, eval_while_loop fn. Mirror existing for-loop pattern |
| LANG-03 | Unicode operator resilience for pasted text | Lexer operates on bytes (line 18: `let bytes = input.as_bytes()`). Need: prepend a Unicode normalization step to `tokenize()` that replaces multi-byte chars before byte-level lexing |
| LANG-04 | `print(expr)` displays intermediate results during loops/procedures | Add as special-case in eval_expr's FuncCall match (like RETURN at line 1072). Use `format_value()` + `println!()`. Return the value so `print(x)` is usable in expressions |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| qsym-core | local | FPS arithmetic, invert() function | Project's own core engine |
| qsym-cli | local | Parser, evaluator, lexer, REPL | Project's own CLI crate |

### Supporting
No additional dependencies needed. All four requirements are implementable with existing Rust standard library features.

## Architecture Patterns

### Existing Codebase Structure (relevant files)
```
crates/qsym-cli/src/
  token.rs      # Token enum (Token::While already exists at line 67)
  lexer.rs      # tokenize() function, byte-level lexer
  ast.rs        # AstNode enum (needs WhileLoop variant)
  parser.rs     # Pratt parser (needs while-loop parsing)
  eval.rs       # eval_expr(), dispatch(), eval_for_loop() (needs while + print + bug fix)
  format.rs     # format_value() (used by print())
  repl.rs       # ReplHelper (needs "while" keyword completion, "print" function completion)
  help.rs       # Help text (needs while + print docs)
crates/qsym-core/src/
  series/arithmetic.rs  # invert() at line 106 -- the bug location
```

### Pattern 1: For-Loop Implementation (template for while-loop)
**What:** The for-loop pattern shows exactly how control flow is implemented
**When to use:** Follow this pattern for while-loop

Key components:
1. **Token:** `Token::While` already exists (token.rs:67)
2. **Lexer:** `"while" => Token::While` already exists (lexer.rs:209)
3. **AST node:** Need `AstNode::WhileLoop { condition: Box<AstNode>, body: Vec<Stmt> }`
4. **Parser:** Add case in `expr_bp` prefix section (near line 202-244, between ForLoop and Proc)
5. **Evaluator:** Add `eval_while_loop()` function, called from eval_expr match

```rust
// AST node (ast.rs)
WhileLoop {
    condition: Box<AstNode>,
    body: Vec<Stmt>,
}

// Parser (parser.rs, in expr_bp prefix match)
Token::While => {
    self.advance(); // consume 'while'
    let condition = self.expr_bp(0)?;
    self.expect(&Token::Do, "'do' after while condition")?;
    let body = self.parse_stmt_sequence(&[Token::Od])?;
    self.expect(&Token::Od, "'od' to close while loop")?;
    AstNode::WhileLoop { condition: Box::new(condition), body }
}

// Evaluator (eval.rs)
fn eval_while_loop(
    condition: &AstNode,
    body: &[Stmt],
    env: &mut Environment,
) -> Result<Value, EvalError> {
    let mut last = Value::None;
    let max_iterations = 1_000_000; // safety limit
    let mut count = 0;
    loop {
        let cond_val = eval_expr(condition, env)?;
        if !is_truthy(&cond_val)? {
            break;
        }
        last = eval_stmt_sequence(body, env)?;
        count += 1;
        if count >= max_iterations {
            return Err(EvalError::Other(
                "while loop exceeded maximum iteration count (1000000)".into()
            ));
        }
    }
    Ok(last)
}
```

### Pattern 2: Special-Case Function (template for print)
**What:** Functions that need AST-level interception are handled as special cases before dispatch
**When to use:** `print()` needs access to `env.symbols` for formatting, not just Value args

Existing special cases in eval_expr FuncCall match (eval.rs:1071-1134):
- `RETURN` (line 1072-1084): intercepts before eval, produces EarlyReturn
- `subs` (line 1089-1119): intercepts before eval, processes AST directly

```rust
// In eval_expr, AstNode::FuncCall match, before procedure/dispatch:
if name == "print" {
    if args.is_empty() {
        return Err(EvalError::WrongArgCount { ... });
    }
    // Evaluate all arguments
    for arg in args {
        let val = eval_expr(arg, env)?;
        println!("{}", crate::format::format_value(&val, &env.symbols));
    }
    // Return last arg value (Maple behavior)
    let last = eval_expr(args.last().unwrap(), env)?;
    return Ok(last);
}
```

Actually, to avoid double evaluation, evaluate args first, print them, return last:
```rust
if name == "print" {
    if args.is_empty() {
        return Err(EvalError::WrongArgCount { ... });
    }
    let mut last_val = Value::None;
    for arg in args {
        let val = eval_expr(arg, env)?;
        println!("{}", crate::format::format_value(&val, &env.symbols));
        last_val = val;
    }
    return Ok(last_val);
}
```

### Pattern 3: Bug Fix -- POLYNOMIAL_ORDER Division Cap
**What:** When dividing by a series with `truncation_order == POLYNOMIAL_ORDER`, cap the effective order
**When to use:** In `eval_div()` before calling `arithmetic::invert()`

The root cause chain:
1. `aqprod(q,q,N)` (3-arg) -> sets `truncation_order = POLYNOMIAL_ORDER = 1_000_000_000` (eval.rs:3150)
2. `1/aqprod(q,q,N)` -> eval_div line 2076-2079: calls `arithmetic::invert(fps)` with fps.truncation_order = 1B
3. `arithmetic::invert()` (arithmetic.rs:106): `for n in 1..trunc` where trunc = 1B -> HANGS

The fix: Before calling `arithmetic::invert()` in `eval_div`, detect POLYNOMIAL_ORDER and replace with the other operand's truncation_order (or `env.default_order` if the other operand is a scalar).

**Critical insight:** The `mul()` function already handles this correctly -- it uses `min(a.trunc, b.trunc)`. But `invert()` only sees one series and uses its own truncation_order. The fix belongs in `eval_div()`, not in `arithmetic::invert()`, because only the evaluator knows about the POLYNOMIAL_ORDER sentinel.

There are 4 code paths in `eval_div` that call `invert()`:
1. Line 2053: `(Series, Series)` -> `arithmetic::invert(b)` -- **needs fix if b has POLYNOMIAL_ORDER**
2. Line 2078: `(scalar, Series)` -> `arithmetic::invert(fps)` -- **needs fix if fps has POLYNOMIAL_ORDER**
3. Line 2167: inside `series_div_general` -> `arithmetic::invert(&shifted_denom)` -- **inherited from callers**
4. Line 2173: inside `series_div_general` -> `arithmetic::invert(denom_fps)` -- **inherited from callers**

**Best fix strategy:** Create a helper `fn cap_polynomial_order(fps: &FormalPowerSeries, other_order: i64) -> FormalPowerSeries` that creates a copy with capped truncation_order when the original has POLYNOMIAL_ORDER. Apply at each call site in eval_div.

Alternatively, simpler approach: add a helper function `effective_trunc(a, b)` that computes the correct working truncation order, then truncate both operands before division. When one side is POLYNOMIAL_ORDER, use the other's order. When both are POLYNOMIAL_ORDER, use `env.default_order`.

```rust
/// Cap a series' truncation_order if it's the POLYNOMIAL_ORDER sentinel.
/// Uses `fallback` as the replacement order.
fn cap_poly_order(fps: &FormalPowerSeries, fallback: i64) -> FormalPowerSeries {
    if fps.truncation_order() == POLYNOMIAL_ORDER {
        arithmetic::truncate(fps, fallback)
    } else {
        fps.clone()
    }
}
```

Then in each division case:
```rust
(Value::Series(a), Value::Series(b)) => {
    let effective_order = match (a.truncation_order() == POLYNOMIAL_ORDER,
                                  b.truncation_order() == POLYNOMIAL_ORDER) {
        (true, true) => env.default_order,
        (true, false) => b.truncation_order(),
        (false, true) => a.truncation_order(),
        (false, false) => a.truncation_order().min(b.truncation_order()),
    };
    let b_capped = if b.truncation_order() == POLYNOMIAL_ORDER {
        arithmetic::truncate(b, effective_order)
    } else {
        b.clone()
    };
    let inv = arithmetic::invert(&b_capped);
    Ok(Value::Series(arithmetic::mul(a, &inv)))
}
```

### Pattern 4: Unicode Normalization in Lexer
**What:** Replace common Unicode operator lookalikes with ASCII equivalents before tokenizing
**When to use:** At the very start of `tokenize()`, before byte-level processing

The lexer currently operates on bytes only (lexer.rs line 18: `let bytes = input.as_bytes()`). Unicode multi-byte characters will hit the "Unknown character" error at line 282-286. The fix is a string replacement pass before byte processing.

```rust
// At the start of tokenize(), before `let bytes = input.as_bytes()`:
fn normalize_unicode(input: &str) -> String {
    input
        .replace('\u{2227}', "^")  // LOGICAL AND -> caret
        .replace('\u{00B7}', "*")  // MIDDLE DOT -> star
        .replace('\u{2212}', "-")  // MINUS SIGN -> hyphen-minus
        .replace('\u{00D7}', "*")  // MULTIPLICATION SIGN -> star
        .replace('\u{2013}', "-")  // EN DASH -> hyphen-minus
        .replace('\u{2014}', "-")  // EM DASH -> hyphen-minus
        .replace('\u{2018}', "'")  // LEFT SINGLE QUOTATION -> apostrophe
        .replace('\u{2019}', "'")  // RIGHT SINGLE QUOTATION -> apostrophe
        .replace('\u{201C}', "\"") // LEFT DOUBLE QUOTATION -> quote
        .replace('\u{201D}', "\"") // RIGHT DOUBLE QUOTATION -> quote
}

pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>, ParseError> {
    let normalized = normalize_unicode(input);
    let bytes = normalized.as_bytes();
    // ... rest of tokenizer uses `normalized` instead of `input`
    // Note: spans will refer to normalized positions, not original
}
```

**Important caveat:** Span positions will shift if multi-byte chars are replaced with single-byte ASCII. This is acceptable since the normalization is a convenience feature and error reporting on normalized input is still useful. However, the `source` field should match what was tokenized.

### Anti-Patterns to Avoid
- **Modifying `arithmetic::invert()` in qsym-core:** The core engine should not know about the POLYNOMIAL_ORDER sentinel -- that's a CLI concept. Fix belongs in eval.rs.
- **Adding iteration limit to `invert()` in core:** This would mask bugs instead of fixing the root cause.
- **Making print() go through dispatch():** It needs access to `env.symbols` for `format_value()`, which dispatch doesn't provide. Use special-case pattern.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Value formatting for print() | Custom print formatter | `format::format_value()` | Already handles all 17 Value variants |
| Truthiness for while condition | Custom condition checker | `is_truthy()` (eval.rs:1224) | Already handles Bool and Integer |
| Statement body parsing for while | Custom body parser | `parse_stmt_sequence()` (parser.rs:539) | Already handles nested control flow |
| Variable save/restore for while | Custom scope management | Not needed | while-loops don't introduce scope (unlike for-loops) |

## Common Pitfalls

### Pitfall 1: POLYNOMIAL_ORDER Propagation Through series_div_general
**What goes wrong:** The `series_div_general` function at eval.rs:2155 also calls `invert()`. If callers pass series with POLYNOMIAL_ORDER, it will also hang.
**Why it happens:** `series_div_general` is called from FractionalPowerSeries division paths (lines 2091, 2097, 2106, 2119).
**How to avoid:** Either cap before calling `series_div_general`, or add capping inside it. Since `series_div_general` is a private helper, capping inside is cleaner.
**Warning signs:** Test `theta2(q,20)/q^(1/4)` where theta2 might return POLYNOMIAL_ORDER.

### Pitfall 2: While Loop Infinite Iteration
**What goes wrong:** A user writes `while 1 do x od` and the REPL hangs forever.
**Why it happens:** No iteration limit on while loops.
**How to avoid:** Add a maximum iteration guard (1,000,000 is reasonable). Return clear error message.
**Warning signs:** Any while loop that doesn't modify its condition variable.

### Pitfall 3: Unicode Span Mismatch
**What goes wrong:** After Unicode normalization, byte offsets in error messages don't match the original input.
**Why it happens:** Multi-byte Unicode chars (3-4 bytes) replaced with single-byte ASCII.
**How to avoid:** Accept the mismatch (error messages reference normalized text) or track an offset map. The simple approach (accept mismatch) is fine since this is a convenience feature.
**Warning signs:** Error messages pointing to wrong column in Unicode-heavy input.

### Pitfall 4: print() Inside eval_stmt_sequence
**What goes wrong:** `eval_stmt_sequence` (eval.rs:1392) calls `eval_expr` and ignores statement terminators. So `print(x);` in a for-loop body would print via `print()` AND potentially via the REPL's statement output handler.
**Why it happens:** `eval_stmt_sequence` only returns the last value; it doesn't print intermediates.
**How to avoid:** `print()` uses `println!()` directly. The REPL only prints the final result of the for-loop (the last iteration's value). No double-printing occurs because `eval_stmt_sequence` doesn't print anything -- only the top-level `eval_stmt` does.
**Warning signs:** Test `for n from 1 to 3 do print(n): od:` -- should print 1, 2, 3 with no extra output.

### Pitfall 5: While Loop Not Added to parse_stmt_sequence Terminators
**What goes wrong:** `while` inside `if` or `for` body fails to parse.
**Why it happens:** `parse_stmt_sequence` stops at terminators like `Od`, `Fi`, `End`. It does NOT need to stop at `While` because `While` is a prefix expression, not a terminator.
**How to avoid:** While is parsed as a prefix expression in `expr_bp()`, just like ForLoop. No changes to `parse_stmt_sequence` needed.
**Warning signs:** Nested `if...then while...do...od fi` failing.

## Code Examples

### BUG-01: Current bug reproduction path
```rust
// eval.rs line 3146-3151: aqprod 3-arg sets POLYNOMIAL_ORDER
if args.len() == 3 {
    let n = extract_i64(name, args, 2)?;
    let result = qseries::aqprod(&monomial, sym, PochhammerOrder::Finite(n), POLYNOMIAL_ORDER);
    Ok(Value::Series(result))  // result.truncation_order == 1_000_000_000
}

// eval.rs line 2076-2079: scalar / Series
(_, Value::Series(fps)) if value_to_qrat(&left).is_some() => {
    let const_fps = value_to_constant_fps(&left, fps.variable(), fps.truncation_order()).unwrap();
    // const_fps has truncation_order = 1_000_000_000
    let inv = arithmetic::invert(fps);  // <-- HANGS: loops 1 billion times
    Ok(Value::Series(arithmetic::mul(&const_fps, &inv)))
}

// arithmetic.rs line 106-135: invert loops to truncation_order
pub fn invert(a: &FormalPowerSeries) -> FormalPowerSeries {
    let trunc = a.truncation_order;  // = 1_000_000_000
    for n in 1..trunc {  // <-- 1 BILLION iterations
        // ...
    }
}
```

### LANG-01: Maple while-loop syntax
```
# Basic while loop
x := 1: while x < 100 do x := x * 2: od: x;
# Expected output: 128

# While with boolean condition
found := 0: n := 1: while found = 0 do
    if n > 10 then found := 1: fi:
    n := n + 1:
od: n;
```

### LANG-03: Unicode characters that appear when pasting from PDFs/papers
```
# These should all work after normalization:
q^5       # U+2227 LOGICAL AND used as caret
3 * 5     # U+00B7 MIDDLE DOT used as multiplication
x - 1     # U+2212 MINUS SIGN used as subtraction
2 * 3     # U+00D7 MULTIPLICATION SIGN used as multiplication
```

### LANG-04: print() usage in loops
```
# Print intermediate values during a computation
for n from 1 to 5 do
    print(aqprod(q, q, n, 20)):
od:

# Print inside a while loop
x := 1: while x < 100 do print(x): x := x * 2: od:
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 3-arg aqprod used default_order | 3-arg aqprod uses POLYNOMIAL_ORDER | Phase 48 (v4.0) | Created BUG-01 |
| No exact polynomial support | POLYNOMIAL_ORDER sentinel | Phase 48 (v4.0) | Exact poly display works but division hangs |
| While token defined, unused | Still unused | Phase 35 (v2.0) | Token ready, no parser/eval support |

## Open Questions

1. **Should `arithmetic::truncate` exist or do we need to build it?**
   - What we know: There is a `series` function in dispatch that can truncate, but we need a helper in eval.rs
   - What's unclear: Whether there's a `truncate` function in arithmetic.rs already
   - Recommendation: Check if `FormalPowerSeries` has a method to change truncation_order; if not, create a simple helper that clones and sets truncation_order + removes coefficients >= new order

2. **While-loop: should it support `for...while` combined syntax like Maple?**
   - What we know: Maple supports `for n from 1 while condition do body od`
   - What's unclear: Whether researchers need this combined form
   - Recommendation: Start with standalone `while...do...od` only. The combined form can be added later.

3. **print() return value: should it return NONE or the value printed?**
   - What we know: Maple's `print()` returns NULL. But returning the value is more useful for `x := print(expr)`.
   - Recommendation: Return the last printed value (more useful in scripting contexts).

## Sources

### Primary (HIGH confidence)
- Direct code inspection of eval.rs lines 2050-2175 (eval_div), 1483-1531 (eval_for_loop), 3146-3151 (aqprod 3-arg)
- Direct code inspection of arithmetic.rs lines 106-135 (invert function)
- Direct code inspection of lexer.rs (tokenize function, byte-level processing)
- Direct code inspection of parser.rs (Pratt parser, for-loop pattern at lines 202-244)
- Direct code inspection of token.rs (Token::While at line 67)
- Direct code inspection of ast.rs (AstNode enum, existing variants)

### Secondary (MEDIUM confidence)
- Maple documentation for while-loop syntax and print() behavior (from training data)

## Metadata

**Confidence breakdown:**
- BUG-01 fix: HIGH - root cause fully traced through source code, fix strategy clear
- LANG-01 while-loop: HIGH - Token exists, for-loop pattern provides exact template
- LANG-03 Unicode: HIGH - lexer byte-level processing understood, normalization straightforward
- LANG-04 print(): HIGH - special-case function pattern established by RETURN/subs

**Research date:** 2026-02-21
**Valid until:** Indefinite (internal codebase, no external dependencies)
