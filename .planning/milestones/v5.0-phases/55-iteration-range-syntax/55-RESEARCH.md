# Phase 55: Iteration with Range Syntax - Research

**Researched:** 2026-02-22
**Domain:** Lexer/parser extension + evaluator special-case for Maple-style add/mul/seq
**Confidence:** HIGH

## Summary

This phase adds Maple-style `add(expr, i=a..b)`, `mul(expr, i=a..b)`, and `seq(expr, i=a..b)` to the q-Kangaroo CLI. The key challenge is that these are NOT normal function calls -- the first argument is an expression *template* containing the iteration variable `i`, which must NOT be evaluated eagerly. The second argument uses `=` (normally comparison) and `..` (not yet a token) in a special way.

The codebase already has strong precedent for this pattern. The `subs(var=val, expr)` function intercepts AST nodes at the evaluator level, interpreting `AstNode::Compare { op: Eq }` as a substitution pair rather than a boolean comparison. The `for` loop already implements variable save/restore scoping. The proposed solution follows both patterns: add `..` as a new `Token::DotDot` and `AstNode::Range` node, then special-case `add`/`mul`/`seq` in the evaluator's `FuncCall` handler to interpret the second argument's `=` as `var=range` (like `subs`) and evaluate the first argument repeatedly with variable substitution (like `for`).

**Primary recommendation:** Add `Token::DotDot` to the lexer, `AstNode::Range { lo, hi }` to the AST, then special-case `add`/`mul`/`seq` in `eval_expr`'s `FuncCall` branch (right next to the `subs` special-case) to perform AST-level argument interception.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| ITER-01 | `add(expr, i=a..b)` computes symbolic summation (Maple-style) | Evaluator special-case intercepts AST, loops i from a to b, evaluates expr each iteration, accumulates via `eval_add`. Uses existing `for`-loop scoping pattern. |
| ITER-02 | `mul(expr, i=a..b)` computes symbolic product (Maple-style) | Same mechanism as ITER-01 but accumulates via `eval_mul` with identity `Value::Integer(1)`. |
| ITER-03 | `seq(expr, i=a..b)` generates a list/sequence | Same mechanism as ITER-01 but collects results into `Vec<Value>`, returns `Value::List`. |
</phase_requirements>

## Standard Stack

### Core

No new external dependencies required. All changes are within `qsym-cli` crate.

| Module | File | Purpose | Why Standard |
|--------|------|---------|--------------|
| `token.rs` | `crates/qsym-cli/src/token.rs` | Add `Token::DotDot` variant | Follows established pattern for new operators |
| `lexer.rs` | `crates/qsym-cli/src/lexer.rs` | Lex `..` as `Token::DotDot` | Two-char greedy match like `:=`, `->`, `<=` |
| `ast.rs` | `crates/qsym-cli/src/ast.rs` | Add `AstNode::Range { lo, hi }` variant | Captures `a..b` as a first-class AST node |
| `parser.rs` | `crates/qsym-cli/src/parser.rs` | Parse `..` as infix operator producing `Range` | Pratt parser infix binding power slot |
| `eval.rs` | `crates/qsym-cli/src/eval.rs` | Special-case add/mul/seq in FuncCall handler | Follows subs() pattern for AST interception |
| `help.rs` | `crates/qsym-cli/src/help.rs` | Add help entries for add, mul, seq | Follows FUNC_HELP pattern |
| `repl.rs` | `crates/qsym-cli/src/repl.rs` | Add add, mul, seq to completion list | Follows canonical_function_names pattern |

## Architecture Patterns

### Pattern 1: Lexer -- Two-Character Token `..`

**What:** Add `Token::DotDot` to the token enum and lex `..` in the lexer.

**When to use:** The `.` character is currently unrecognized by the lexer (`unexpected character '.'`). We need to handle exactly `..` (two dots). A single `.` should remain an error (or could be lexed separately for future use, but keeping it as error is simpler and correct).

**Implementation approach:**

```rust
// In token.rs: add to Token enum
/// `..` range operator.
DotDot,

// In lexer.rs: add before the "unknown character" fallback
if b == b'.' {
    if pos + 1 < bytes.len() && bytes[pos + 1] == b'.' {
        tokens.push(SpannedToken {
            token: Token::DotDot,
            span: Span::new(pos, pos + 2),
        });
        pos += 2;
    } else {
        return Err(ParseError::new(
            "unexpected character '.'".to_string(),
            Span::new(pos, pos + 1),
        ));
    }
    continue;
}
```

**Placement in lexer.rs:** Insert the `b'.'` check right before the final "Unknown character" block (around line 299). This follows the same pattern as `:=` and `->` greedy matching.

### Pattern 2: AST Node -- `AstNode::Range`

**What:** Add a `Range` variant to `AstNode` to represent `a..b` in the AST.

```rust
/// Range expression: `lo..hi` (used in add/mul/seq iteration).
Range {
    lo: Box<AstNode>,
    hi: Box<AstNode>,
},
```

**Key design decision:** The range is a first-class AST node, NOT just a special case inside function calls. This means `1..5` can be parsed anywhere (it will produce a Range node). The evaluator will error if a Range appears in a context that doesn't expect it, which is fine. This is cleaner than trying to special-case the parser only inside function call argument lists.

### Pattern 3: Parser -- `..` as Infix Operator

**What:** Add `Token::DotDot` to the `infix_bp()` function with appropriate binding power.

**Binding power analysis:**

Current operator precedence (from `infix_bp`):
- `or`: (3, 4)
- `and`: (5, 6)
- `=, <>, <, >, <=, >=`: (9, 10)
- `+, -`: (11, 12)
- `*, /`: (13, 14)
- `^`: (17, 18)
- Function call / subscript: 19

For `i=1..5`, we need `..` to bind tighter than `=` so that it parses as `i = (1..5)` not `(i=1) .. 5`. But `..` should bind LOOSER than arithmetic so `1+2..5` parses as `(1+2)..5`.

**Correct binding power for `..`:** `(10, 10)` -- this way:
- In `i = 1..5`: `=` has r_bp=10, then `1..5` is parsed with `..` having l_bp=10 (>= min_bp=10), so it binds.
- In `1+2..3+4`: `+` has l_bp=11 > 10, so `1+2` is fully parsed before `..`, and `3+4` on the right side also works since r_bp=10 < 11.

**Non-associativity check safety:** The parser's non-associativity check (lines 479-488) specifically matches comparison tokens (`Equal | NotEqual | Less | Greater | LessEq | GreaterEq`). Since `DotDot` is not in that set, it won't trigger the check. So `i=1..5` parses correctly as:
1. `i` is Variable
2. `=` is comparison with l_bp=9, r_bp=10
3. RHS is parsed with min_bp=10
4. `1` is Integer
5. `..` has l_bp=10 >= min_bp=10, so it captures
6. `5` is parsed as RHS of `..`
7. Result: `Compare(Eq, Variable("i"), Range(Integer(1), Integer(5)))`

**CRITICAL implementation detail -- the `unreachable!()` trap:** The infix match block in `expr_bp()` (parser.rs lines 437-503) has a final `_ => unreachable!()` arm. When `DotDot` is added to `infix_bp()`, the code will enter the infix handling, but if there is no match arm for `DotDot`, it will hit `unreachable!()` and panic. The `DotDot` match arm MUST be added to the match block alongside the arithmetic, comparison, and boolean arms.

**Implementation:**

```rust
// In infix_bp():
Token::DotDot => Some((10, 10)),

// In expr_bp() infix match block (lines 437-501), add BEFORE the `_ => unreachable!()`:
Token::DotDot => {
    lhs = AstNode::Range {
        lo: Box::new(lhs),
        hi: Box::new(rhs),
    };
}
```

### Pattern 4: Evaluator -- AST-Level Special-Case for add/mul/seq

**What:** Intercept `add`, `mul`, and `seq` function calls in the `AstNode::FuncCall` handler, BEFORE arguments are evaluated.

**This follows the exact same pattern as `subs()`** (lines 1158-1191 in eval.rs).

**Key insight:** The function call `add(i^2, i=1..5)` is parsed as:
```
FuncCall {
    name: "add",
    args: [
        BinOp { op: Pow, lhs: Variable("i"), rhs: Integer(2) },
        Compare { op: Eq, lhs: Variable("i"), rhs: Range(Integer(1), Integer(5)) }
    ]
}
```

The evaluator special-case must:
1. Validate exactly 2 args
2. Extract the iteration spec from `args[1]`:
   - Must be `Compare { op: Eq, lhs: Variable(var), rhs: Range(lo, hi) }`
   - Evaluate `lo` and `hi` to get integer bounds
3. Save the current value of `var` (like `eval_for_loop`)
4. Loop from `lo` to `hi`:
   - Set `var = i` in the environment
   - Evaluate `args[0]` (the body expression)
   - Accumulate the result (add: sum, mul: product, seq: collect to list)
5. Restore `var` to its saved value

**Implementation sketch:**

```rust
// In eval_expr, inside AstNode::FuncCall, after the subs special-case:

if name == "add" || name == "mul" || name == "seq" {
    if args.len() != 2 {
        return Err(EvalError::WrongArgCount { ... });
    }
    // Extract iteration variable and range from args[1]
    let (var_name, lo, hi) = match &args[1] {
        AstNode::Compare { op: CompOp::Eq, lhs, rhs } => {
            let var = match lhs.as_ref() {
                AstNode::Variable(v) => v.clone(),
                _ => return Err(EvalError::Other(
                    format!("{}: expected variable on left of =", name)
                )),
            };
            let (lo_node, hi_node) = match rhs.as_ref() {
                AstNode::Range { lo, hi } => (lo.as_ref(), hi.as_ref()),
                _ => return Err(EvalError::Other(
                    format!("{}: expected range (a..b) on right of =", name)
                )),
            };
            let lo_val = eval_expr(lo_node, env)?;
            let hi_val = eval_expr(hi_node, env)?;
            let lo_i = value_to_i64(&lo_val, &format!("{} range start", name))?;
            let hi_i = value_to_i64(&hi_val, &format!("{} range end", name))?;
            (var, lo_i, hi_i)
        }
        _ => return Err(EvalError::Other(
            format!("{}: second argument must be var=a..b", name)
        )),
    };

    // Save and restore iteration variable (same pattern as eval_for_loop)
    let saved = env.variables.remove(&var_name);

    let result = (|| -> Result<Value, EvalError> {
        match name.as_str() {
            "add" => {
                let mut acc = Value::Integer(QInt::from(0i64));
                for i in lo..=hi {
                    env.set_var(&var_name, Value::Integer(QInt::from(i)));
                    let val = eval_expr(&args[0], env)?;
                    acc = eval_add(acc, val, env)?;
                }
                Ok(acc)
            }
            "mul" => {
                let mut acc = Value::Integer(QInt::from(1i64));
                for i in lo..=hi {
                    env.set_var(&var_name, Value::Integer(QInt::from(i)));
                    let val = eval_expr(&args[0], env)?;
                    acc = eval_mul(acc, val, env)?;
                }
                Ok(acc)
            }
            "seq" => {
                let mut items = Vec::new();
                for i in lo..=hi {
                    env.set_var(&var_name, Value::Integer(QInt::from(i)));
                    let val = eval_expr(&args[0], env)?;
                    items.push(val);
                }
                Ok(Value::List(items))
            }
            _ => unreachable!(),
        }
    })();

    // Restore variable
    match saved {
        Some(old_val) => env.set_var(&var_name, old_val),
        None => { env.variables.remove(&var_name); }
    }

    return result;
}
```

### Pattern 5: Handling `eval_add` and `eval_mul` Visibility

**What:** The `eval_add` and `eval_mul` functions are currently defined in eval.rs (lines 1799 and 2061). They take `(Value, Value, &mut Environment)` and return `Result<Value, EvalError>`. They are used by the `eval_binop` dispatcher. They should be accessible from the new special-case code since it lives in the same module.

**No visibility changes needed** -- the add/mul/seq special-case code is inside `eval_expr` in the same file. However, if the implementation is factored into a separate `eval_iteration_func` function, it must still be in the same module (eval.rs) to access these private functions.

### Pattern 6: Empty Range Handling

**What:** When `lo > hi` (empty range), Maple returns:
- `add`: 0 (identity for addition)
- `mul`: 1 (identity for multiplication)
- `seq`: empty list `[]`

This is the mathematically correct behavior and must be implemented. The Rust `lo..=hi` inclusive range naturally produces zero iterations when `lo > hi`, so the initial accumulator values (0, 1, empty Vec) are returned. No special case code is needed.

### Pattern 7: Registration Points for New Functions

**What:** Three separate lists must be updated when adding new functions:

1. **`ALL_FUNCTION_NAMES`** (eval.rs line 6431) -- for fuzzy matching / "Did you mean?" suggestions
2. **`canonical_function_names()`** (repl.rs line 66) -- for tab completion
3. **`general_help()`** (help.rs line 16) -- for the grouped function listing

Additionally:
4. **`get_signature()`** (eval.rs) -- for error message signatures
5. **`FUNC_HELP`** (help.rs line 185) -- for per-function help entries

### Anti-Patterns to Avoid

- **Don't make add/mul/seq go through dispatch():** These functions need AST access (unevaluated first argument). They CANNOT be dispatched after argument evaluation like normal functions. They MUST be special-cased in the `FuncCall` handler, exactly like `subs()`.

- **Don't add a new token for `=` in iteration context:** The existing `Token::Equal` / `AstNode::Compare { Eq }` is perfect. The `subs` function already interprets `Compare(Eq)` as a substitution pair. We do the same for the iteration variable binding.

- **Don't try to support `..` only inside function calls:** Making `..` a general infix operator is cleaner. If someone writes `1..5` at the top level, the evaluator can produce a sensible error ("range expressions are only valid inside add/mul/seq").

- **Don't forget variable restoration on error:** The `eval_for_loop` function uses a closure pattern `(|| { ... })()` to ensure the variable is restored even if the body errors. The add/mul/seq implementation MUST do the same.

- **Don't forget the `unreachable!()` in parser.rs:** The infix match block ends with `_ => unreachable!()`. Any new token added to `infix_bp()` MUST have a corresponding match arm in the infix handler block, or the parser will panic at runtime.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Value addition | Custom accumulator | `eval_add()` (line 1799) | Handles all type combinations: Integer+Integer, Series+Series, mixed numeric, Series+scalar |
| Value multiplication | Custom accumulator | `eval_mul()` (line 2061) | Same type promotion logic |
| Variable scoping | Custom scope mechanism | Save/restore pattern from `eval_for_loop` (line 1612-1633) | Already handles save, restore, undefined var case |
| Integer extraction from Value | Manual match | `value_to_i64()` (line 1568) | Standard error formatting |

## Common Pitfalls

### Pitfall 1: Eager Evaluation of Body Expression

**What goes wrong:** If `add(i^2, i=1..5)` evaluates `i^2` before the special-case kicks in, it will error with "undefined variable 'i'" or use the wrong value if `i` is already defined.
**Why it happens:** The normal FuncCall path evaluates all arguments before dispatching.
**How to avoid:** The special-case MUST be checked BEFORE the normal evaluation path. Place it right after the `subs` check in the FuncCall handler, before the "evaluate all args" block (line 1221).
**Warning signs:** Tests with undefined iteration variable failing.

### Pitfall 2: `=` Parsed as Assignment Instead of Comparison

**What goes wrong:** `i=1..5` could be parsed as `i := (1..5)` if the parser sees `=` after an identifier as assignment.
**Why it happens:** In the parser, `:=` is the assignment token, `=` is the comparison token. So `i=1..5` with a bare `=` is correctly parsed as comparison. This is NOT a pitfall in this codebase.
**How to avoid:** Already handled -- the lexer distinguishes `:=` (Token::Assign) from `=` (Token::Equal). No action needed.

### Pitfall 3: Variable Not Restored After Error

**What goes wrong:** If the body expression errors (e.g., division by zero at some iteration), the iteration variable is left set to its last value, corrupting the outer scope.
**Why it happens:** Early return from loop without cleanup.
**How to avoid:** Use the closure pattern from `eval_for_loop`: `let result = (|| { ... })(); /* restore here */ result`
**Warning signs:** Tests checking variable state after failed add/mul/seq calls.

### Pitfall 4: `..` Binding Power Wrong

**What goes wrong:** `add(i^2, i=1+2..5)` should parse as `i=(1+2)..5`, not `i=1+(2..5)`.
**Why it happens:** Wrong binding power for `..` operator.
**How to avoid:** Set `DotDot` binding power to (10, 10). This is tighter than `=` (9,10) but looser than `+` (11,12). So `1+2..3+4` parses as `(1+2)..(3+4)`.
**Warning signs:** Tests with arithmetic in range bounds giving wrong results.

### Pitfall 5: Forgetting to Register in ALL_FUNCTION_NAMES and canonical_function_names

**What goes wrong:** `add`, `mul`, `seq` won't appear in tab completion or fuzzy matching suggestions.
**Why it happens:** New functions need to be added to 5 places: ALL_FUNCTION_NAMES, canonical_function_names(), general_help(), get_signature(), and FUNC_HELP.
**How to avoid:** Add to all 5 lists. Create a checklist.
**Warning signs:** Tab completion not working, "Did you mean?" not suggesting, `help add` returning nothing.

### Pitfall 6: DotDot in token_name for Error Messages

**What goes wrong:** If `..` appears in an unexpected position, the error message shows something unhelpful.
**Why it happens:** `token_name()` function doesn't have a case for `Token::DotDot`.
**How to avoid:** Add `Token::DotDot => "'..'".to_string()` to the `token_name()` function in parser.rs.

### Pitfall 7: Range Evaluation in General Context

**What goes wrong:** `1..5` at the top level produces a confusing error or panics.
**Why it happens:** The evaluator doesn't have a case for `AstNode::Range`.
**How to avoid:** Add a match arm for `AstNode::Range` in `eval_expr` that returns an error: "range expressions (a..b) are only valid inside add(), mul(), or seq()".

### Pitfall 8: Parser `unreachable!()` Panic

**What goes wrong:** Adding `Token::DotDot` to `infix_bp()` without a corresponding match arm in the infix handler (parser.rs line 437-502) causes a runtime panic at `_ => unreachable!()` (line 502).
**Why it happens:** The infix operator match block handles arithmetic, comparison, and boolean tokens, with a catch-all unreachable. Any token returned by `infix_bp()` MUST have a corresponding handler.
**How to avoid:** Add a `Token::DotDot` arm to the match block that constructs `AstNode::Range`.
**Warning signs:** Panic on any input containing `..`.

### Pitfall 9: Negative Range Bounds

**What goes wrong:** `add((-1)^i, i=-3..3)` should work with negative loop bounds but could fail if bounds are not handled as signed integers.
**Why it happens:** `value_to_i64` already handles negative values, so this should work naturally.
**How to avoid:** Include tests with negative range bounds.
**Warning signs:** Tests with `i=-5..-1` failing.

## Code Examples

### Lexer Addition (verified pattern from existing `:=` and `->` handling)

```rust
// In lexer.rs, before the "Unknown character" block (~line 299):
if b == b'.' {
    if pos + 1 < bytes.len() && bytes[pos + 1] == b'.' {
        tokens.push(SpannedToken {
            token: Token::DotDot,
            span: Span::new(pos, pos + 2),
        });
        pos += 2;
    } else {
        return Err(ParseError::new(
            "unexpected character '.'".to_string(),
            Span::new(pos, pos + 1),
        ));
    }
    continue;
}
```

### Parser Infix Handler (verified pattern from existing comparison operators)

```rust
// In infix_bp():
Token::DotDot => Some((10, 10)),

// In expr_bp() infix match block (lines 437-501),
// add new arm BEFORE `_ => unreachable!()`:
Token::DotDot => {
    lhs = AstNode::Range {
        lo: Box::new(lhs),
        hi: Box::new(rhs),
    };
}
```

### Evaluator Special-Case (follows subs() pattern, lines 1158-1191)

```rust
// In eval_expr, AstNode::FuncCall handler, after subs special-case:
if name == "add" || name == "mul" || name == "seq" {
    return eval_iteration_func(name, args, env);
}

// Separate function:
fn eval_iteration_func(
    name: &str,
    args: &[AstNode],
    env: &mut Environment,
) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::WrongArgCount {
            function: name.to_string(),
            expected: "2".to_string(),
            got: args.len(),
            signature: format!("{}(expr, var=a..b)", name),
        });
    }
    // ... extract var, lo, hi from args[1]
    // ... save/restore variable, iterate, accumulate
}
```

### Variable Save/Restore (from eval_for_loop, line 1612-1633)

```rust
let saved = env.variables.remove(&var_name);

let result = (|| -> Result<Value, EvalError> {
    // ... loop body ...
})();

match &saved {
    Some(old_val) => env.set_var(&var_name, old_val.clone()),
    None => { env.variables.remove(&var_name); }
}

result
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No `..` operator | Add `Token::DotDot` and `AstNode::Range` | This phase | Enables Maple-style range syntax |
| No add/mul/seq | AST-level special-case (like subs) | This phase | Maple compatibility for iteration |
| Manual for-loop accumulation | Single-expression add/mul/seq | This phase | Concise mathematical notation |

**Maple compatibility notes (from official docs):**
- Maple's `add` returns 0 for empty range (lo > hi) -- we must match
- Maple's `mul` returns 1 for empty range -- we must match
- Maple restores the iteration variable after execution -- we must match
- Maple's `add`/`mul`/`seq` first argument is evaluated eagerly per iteration (NOT symbolically) -- we match this naturally

## Open Questions

1. **Should `..` support negative step / reverse ranges?**
   - What we know: Maple's `add(f, i=5..1)` returns 0 (empty sum). The `for` loop requires explicit `by -1` for reverse iteration.
   - What's unclear: Whether users would expect `add(f, i=5..1)` to iterate in reverse.
   - Recommendation: Follow Maple -- empty range returns identity (0 for add, 1 for mul, [] for seq). If users want reverse, they can use `add(f, i=-5..-1)` or `for` loop. This is simpler and Maple-compatible.

2. **Should Range be usable outside add/mul/seq?**
   - What we know: `1..5` will parse successfully as `AstNode::Range` anywhere.
   - What's unclear: Whether to eventually support ranges in for-loops (`for i in 1..5`) or as standalone values.
   - Recommendation: For this phase, `AstNode::Range` in eval_expr produces an error. Future phases can add Range as a value type if needed. Keep it simple now.

3. **Should we support a step argument?**
   - What we know: Maple supports `add(f, i=0..10, 2)` with optional step. The `for` loop supports `by step`.
   - Recommendation: Defer step support. The 2-argument form covers all success criteria. Add 3-argument step support in a future phase if needed.

## Sources

### Primary (HIGH confidence)
- **Codebase analysis** -- lexer.rs, token.rs, ast.rs, parser.rs, eval.rs, help.rs, repl.rs read in full
- **eval_for_loop** (eval.rs lines 1586-1634) -- variable save/restore pattern
- **subs special-case** (eval.rs lines 1158-1191) -- AST interception pattern
- **infix_bp** (parser.rs lines 600-611) -- operator precedence reference
- **eval_add / eval_mul** (eval.rs lines 1799, 2061) -- value accumulation functions
- **Parser infix match block** (parser.rs lines 437-503) -- `unreachable!()` trap at line 502

### Secondary (MEDIUM confidence)
- [Maple add documentation](https://www.maplesoft.com/support/help/Maple/view.aspx?path=add) -- confirms variable restoration, empty range semantics

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all changes in well-understood qsym-cli crate, no external dependencies
- Architecture: HIGH -- follows established patterns (subs AST interception, for-loop scoping), clear binding power analysis
- Pitfalls: HIGH -- identified from direct code reading, all have concrete prevention strategies

**Research date:** 2026-02-22
**Valid until:** 2026-06-22 (stable internal architecture, no external dependency concerns)
