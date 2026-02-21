# Phase 47: Parser & Language Extensions - Research

**Researched:** 2026-02-20
**Domain:** Pratt parser extensions, lambda functions, fractional exponents, Maple compatibility
**Confidence:** HIGH

## Summary

Phase 47 adds four language-level features to the q-Kangaroo CLI: a ditto operator (`"`) for referencing the last computed result, arrow/lambda function syntax (`q -> expr`), fractional q-powers (`q^(1/4)`), and relaxed option/local ordering in procedure definitions. All four features are well-scoped modifications to the existing hand-written Pratt parser (1,678 lines), lexer (614 lines), AST (359 lines), and evaluator (10,405 lines).

The existing codebase already has all the infrastructure needed: `AstNode::LastResult` and `Token::Percent` implement `%` as a last-result reference (ditto `"` follows the same pattern), `AstNode::ProcDef` with params/locals/options/body provides the template for lambda definitions, `eval_pow` handles Symbol^Integer and Symbol^Rational cases (fractional powers extend this), and the proc parser already has local-then-option ordering that just needs to accept either order.

**Primary recommendation:** Implement in three plans as specified. Each feature is isolated to specific files with no cross-dependencies, enabling parallel development.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LANG-01 | Ditto operator `"` references last computed result | Lexer change: treat bare `"` as Token::Ditto (new token). Parser maps to AstNode::LastResult (existing). Evaluator unchanged. See "Ditto Operator" section below. |
| LANG-02 | Arrow operator for lambda functions: `F := q -> expr` | New Token::Arrow, new AstNode::Lambda, new Value variant or reuse Procedure. Pratt parser adds `->` as infix with binding power between assignment and `or`. See "Arrow/Lambda" section. |
| LANG-04 | Fractional q-powers: `q^(1/4)`, `theta2(q,100)/q^(1/4)` | Modify eval_pow to allow Symbol^Rational with fractional denominators. Introduce rational exponent BTreeMap keys or multiply-through-by-denominator strategy. See "Fractional Powers" section. |
| LANG-05 | option/local either order in procedures | Parser change only: try both `local` and `option` in a loop. No AST or evaluator changes. See "Option/Local Reorder" section. |
</phase_requirements>

## Standard Stack

### Core (no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| qsym-cli (local) | - | Parser, lexer, AST, evaluator | All changes are in this crate |
| qsym-core (local) | - | FormalPowerSeries, arithmetic | Fractional power support may touch series ops |
| rug | 1.26 | Arbitrary precision rationals | QRat already supports fractional arithmetic |

No new external dependencies required. All four features are implemented entirely within existing code.

## Architecture Patterns

### Relevant File Structure
```
crates/qsym-cli/src/
  token.rs      # Token enum (add Arrow, Ditto)
  lexer.rs      # tokenize() (add "->" and bare `"` recognition)
  ast.rs        # AstNode enum (add Lambda variant)
  parser.rs     # Pratt parser (add arrow infix, ditto prefix, proc reorder)
  eval.rs       # eval_expr, eval_pow, call_procedure (lambda eval, fractional pow)
  format.rs     # format_value (lambda display)
  repl.rs       # ReplHelper (update keyword list for "->")
  help.rs       # Help text updates
```

### Pattern: Adding a New Operator to the Pratt Parser

The established pattern for adding operators to this parser:

1. **Token:** Add variant to `Token` enum in `token.rs`
2. **Lexer:** Add recognition in `tokenize()` in `lexer.rs`
3. **Parser:** Add to NUD (prefix) or LED (infix) handling in `parser.rs`
4. **AST:** Add variant to `AstNode` enum in `ast.rs` (if needed)
5. **Evaluator:** Add `eval_expr` match arm in `eval.rs`
6. **Format:** Add display in `format.rs`
7. **Tests:** Add tests at each layer

### Pattern: Binding Power Table

Current binding power assignments (from `infix_bp` and parser code):

| BP (left, right) | Operator |
|-------------------|----------|
| (2, 1) | `:=` assignment |
| (3, 4) | `or` |
| (5, 6) | `and` |
| prefix 7 | `not` |
| (9, 10) | comparisons `= <> < > <= >=` |
| (11, 12) | `+ -` |
| (13, 14) | `* /` |
| prefix 15 | unary `-` |
| (17, 18) | `^` |
| postfix 19 | function call `f(...)` |

The arrow operator `->` should bind looser than everything except assignment, so it can capture the full RHS expression. Recommended BP: **(2, 1)** -- same level as assignment, handled as special case in the LED loop similar to how `:=` is handled.

### Anti-Patterns to Avoid
- **Do NOT modify FormalPowerSeries to store fractional exponents.** The BTreeMap<i64, QRat> structure uses integer keys by design. Fractional powers should be handled at the evaluator level by multiplying exponents by the LCD (least common denominator).
- **Do NOT make `"` a general string-start character.** It must be disambiguated from string literals based on context (standalone `"` vs `"text"`).
- **Do NOT add a separate Value::Lambda variant unless necessary.** Reuse Value::Procedure with a single-param proc for simplicity.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Lambda storage | New Value variant with closure semantics | Value::Procedure (existing) | Lambda is just a proc with one param and implicit return |
| Fractional series exponents | Custom BTreeMap<QRat, QRat> series | Scale exponents by LCD, use existing i64 BTreeMap | All arithmetic ops assume integer exponents |
| Ditto state management | New state tracking | Environment.last_result (existing) | Already stores last result for `%` |

## Common Pitfalls

### Pitfall 1: Ditto `"` vs String Literal `"text"`
**What goes wrong:** The lexer currently treats `"` as the start of a string literal. A bare `"` followed by `,` or `)` or `;` would currently cause "unterminated string literal" error.
**Why it happens:** Double-quote has dual meaning: Maple ditto (bare) vs string delimiter (paired).
**How to avoid:** In the lexer, after seeing `"`, peek at the next character. If it's `,`, `)`, `;`, `:`, `+`, `-`, `*`, `/`, `^`, whitespace-then-operator, or EOF, emit `Token::Ditto`. Otherwise, continue as string literal. More precisely: if the character after `"` is NOT a printable character that could start string content (i.e., it's a delimiter, operator, or EOF), treat as ditto.
**Better approach:** After `"`, check if the next non-whitespace character is another `"` (closing quote) -- if so, it is an empty string `""`. If the immediately next character is a delimiter/operator/EOF, it is ditto. Otherwise, it is a string literal start. The key insight: `"` as ditto always appears where an expression is expected and is immediately followed by a delimiter, not by string content.
**Simplest approach:** After seeing `"`, if the next byte is `,`, `)`, `;`, `:`, space, `+`, `-`, `*`, `/`, `^`, `<`, `>`, `=`, `]`, or EOF, emit Token::Ditto. Otherwise proceed with string literal parsing. This handles `etamake(",q,100)` -- the `"` is followed by `,`.

### Pitfall 2: Arrow Operator Precedence
**What goes wrong:** `F := q -> theta3(q,500)/theta3(q^5,100)` must parse as `F := (q -> (theta3(q,500)/theta3(q^5,100)))`, not `(F := q) -> ...`.
**Why it happens:** Arrow and assignment have similar conceptual precedence, but arrow must bind its entire RHS.
**How to avoid:** Handle `->` as a special case in the LED loop (similar to `:=`), not in the general `infix_bp` table. When we see `->`, the LHS must be a Variable (the parameter name), and the RHS is parsed with `expr_bp(0)` to capture everything. The arrow only appears after a variable and before a full expression.
**Binding:** Arrow should be detected in the LED loop after checking for `:=` but with right-associativity (r_bp = 0 so the full RHS is captured). Since `F := q -> expr` parses as `F := (q -> expr)`, the arrow is first seen when the parser is inside the RHS of `:=` (which used r_bp = 1). Arrow's l_bp should be > 1 so it is captured by `:=`. Using l_bp = 2 (same as `:=` left) with r_bp = 1 means it will be right-associative relative to itself and bind tightly enough to be captured as a `:=` RHS.

### Pitfall 3: Fractional Power Scale Factor
**What goes wrong:** `q^(1/4)` produces a series with fractional exponent, but FormalPowerSeries only supports integer exponents (BTreeMap<i64, QRat>).
**Why it happens:** The internal representation uses i64 keys.
**How to avoid:** When the exponent of a `Symbol^Rational(p/d)` is encountered, create a FPS in terms of a scaled variable. Specifically, `q^(1/4)` becomes a monomial at exponent 1 in a series "in q^(1/4)" -- but we don't actually change the variable. Instead, conceptually we scale: the series works in q where q maps to q^(1/4). When dividing `theta2(q,100)` by `q^(1/4)`, we need to multiply all exponents by 4, creating a series in `q^(1/4)` with integer exponents, then perform division.
**Recommended strategy:** The simplest implementation: `q^(1/N)` creates a FPS monomial at exponent 1 with a hidden "scale factor" of N. For division `series / q^(1/N)`, rescale the series exponents by N (multiply all keys by N), subtract 1 from the first exponent, and the result is a series in q^(1/N) displayed with fractional exponents. Display would need to show `q^(k/N)` for key k.
**Alternative (simpler):** Since the primary use case is `theta2(q,100)/q^(1/4)` where theta2 already has integer exponents starting at 1: theta2(q,100) = 2*q + 2*q^9 + ... This represents 2*q^(1/4+0) for Jacobi theta2 which is really sum_{n>=0} 2*q^((2n+1)^2). Wait -- the current implementation uses q^1 as the base variable, not q^(1/4). The actual mathematical theta2 has q^(1/4) prefactor. The implementation computes everything in integer powers of q using the substitution q -> q^8 trick.

So `theta2(q,100)/q^(1/4)` means: take the integer-exponent series (exponents 1, 9, 25, 49, ...) and divide by q^(1/4). The result should have exponents 3/4, 35/4, 99/4, ... which are NOT integer. This is the core challenge.

**Recommended approach:** Store a rational offset/denominator at the Value level. Add a new Value variant `Value::FractionalSeries { series: FormalPowerSeries, exponent_denom: i64, exponent_offset_numer: i64 }` or more simply, during display, detect when division by q^(p/d) was performed and adjust the display accordingly. Actually, the cleanest approach: extend FormalPowerSeries with an optional exponent denominator field, or handle this entirely in the evaluator by rescaling.

**Cleanest approach for this phase:**
1. When `Symbol^(p/d)` is encountered (d > 1), create a Value::Series where exponents are multiplied by d. So `q^(1/4)` creates a monomial at exponent 1 with a "hidden" understanding that exponent k means q^(k/4).
2. When dividing a normal series by q^(1/4), rescale the numerator's exponents by 4, then subtract the 1 from each exponent (i.e., shift by -1).
3. The resulting series has exponents in q^(1/4) space. A "fractional_denom" field on the Value or a wrapper is needed for display.

**Actually simplest:** A new `Value::FractionalPowerSeries { inner: FormalPowerSeries, denom: i64 }` where exponent k in the inner FPS represents q^(k/denom). This keeps the core FPS unchanged. Division/multiplication with fractional series unifies denominators (LCD). Display formats as q^(k/d).

### Pitfall 4: Option/Local Order -- Don't Break Existing Code
**What goes wrong:** Changing the parser to accept either order could accidentally break the existing "local first, option second" parsing.
**Why it happens:** The current parser has sequential if-checks for local then option.
**How to avoid:** Replace the two sequential if-blocks with a loop that accepts either keyword in any order, accumulating both lists. Test both orderings: `local then option`, `option then local`, `only local`, `only option`, `neither`.

### Pitfall 5: Lambda Call Syntax
**What goes wrong:** After `F := q -> expr`, the user calls `F(q)`. But F is stored as a Procedure. The existing FuncCall evaluation checks `env.get_var(name)` for Procedure and calls `call_procedure`. This should work automatically IF we store lambdas as Procedure values.
**Why it happens:** The call path already dispatches to user-defined procedures.
**How to avoid:** Store lambda as `Value::Procedure(Procedure { params: vec![param], body: [expr as implicit-return stmt], ... })`. This reuses the entire procedure calling infrastructure including memo, locals, etc.

## Code Examples

### Example 1: Ditto Token in Lexer (lexer.rs)

```rust
// In tokenize(), modify the string literal section:
if b == b'"' {
    let start = pos;
    // Check if this is a ditto operator (bare `"` not followed by string content)
    // Ditto appears where an expression is expected, followed by delimiter
    let next = if pos + 1 < bytes.len() { bytes[pos + 1] } else { 0 };
    let is_ditto = next == b',' || next == b')' || next == b';'
        || next == b':' || next == b'+' || next == b'-'
        || next == b'*' || next == b'/' || next == b'^'
        || next == b']' || next == b'<' || next == b'>'
        || next == b'=' || next == 0  // EOF
        || next == b' ' || next == b'\t' || next == b'\n' || next == b'\r';
    if is_ditto {
        tokens.push(SpannedToken {
            token: Token::Ditto,
            span: Span::new(pos, pos + 1),
        });
        pos += 1;
        continue;
    }
    // Otherwise, parse as string literal (existing code)
    pos += 1; // skip opening quote
    // ... existing string literal parsing ...
}
```

### Example 2: Arrow Operator in Parser (parser.rs)

```rust
// In the LED loop, after the Assignment check:

// Arrow operator: l_bp = 2, captures entire RHS
if *self.peek() == Token::Arrow {
    if 2 < min_bp {
        break;
    }
    if let AstNode::Variable(param) = lhs {
        self.advance(); // consume ->
        let body = self.expr_bp(0)?; // capture full RHS
        lhs = AstNode::Lambda {
            param,
            body: Box::new(body),
        };
        continue;
    } else {
        let span = self.peek_span();
        return Err(ParseError::new(
            "left side of '->' must be a parameter name".to_string(),
            span,
        ));
    }
}
```

### Example 3: Lambda Evaluation (eval.rs)

```rust
AstNode::Lambda { param, body } => {
    Ok(Value::Procedure(Procedure {
        name: String::new(),
        params: vec![param.clone()],
        locals: vec![],
        remember: false,
        body: vec![Stmt {
            node: body.as_ref().clone(),
            terminator: Terminator::Implicit,
        }],
        memo: Rc::new(RefCell::new(HashMap::new())),
    }))
}
```

### Example 4: Option/Local Reorder (parser.rs)

```rust
// Replace the sequential if-blocks with a loop:
let mut locals = vec![];
let mut options = vec![];
loop {
    if *self.peek() == Token::Local {
        self.advance();
        locals.extend(self.parse_ident_list()?);
        self.expect(&Token::Semi, "';' after local declarations")?;
    } else if *self.peek() == Token::OptionKw {
        self.advance();
        options.extend(self.parse_ident_list()?);
        self.expect(&Token::Semi, "';' after option declarations")?;
    } else {
        break;
    }
}
```

### Example 5: Fractional Power in eval_pow (eval.rs)

```rust
// In eval_pow, Symbol^Rational case:
(Value::Symbol(name), Value::Rational(r)) => {
    let numer = r.0.numer().to_i64().ok_or_else(|| EvalError::Other(
        "exponent numerator too large".to_string(),
    ))?;
    let denom = r.0.denom().to_i64().ok_or_else(|| EvalError::Other(
        "exponent denominator too large".to_string(),
    ))?;
    if denom == 1 {
        // Integer exponent (existing behavior)
        let sym_id = env.symbols.intern(name);
        let fps = FormalPowerSeries::monomial(sym_id, QRat::one(), numer, POLYNOMIAL_ORDER);
        Ok(Value::Series(fps))
    } else {
        // Fractional exponent: create FractionalPowerSeries
        Ok(Value::FractionalPowerSeries {
            inner: FormalPowerSeries::monomial(
                env.symbols.intern(name), QRat::one(), numer, POLYNOMIAL_ORDER
            ),
            denom,
        })
    }
}
```

## Design Decisions

### LANG-01 (Ditto): Token Disambiguation Strategy

The ditto operator `"` conflicts with string literal delimiters. Three approaches:

1. **Context-free lookahead (RECOMMENDED):** After `"`, check if the next byte is a delimiter/operator/EOF. If yes, emit Ditto; otherwise parse as string literal. This is simple, handles all known use cases (`etamake(",q,100)`, `" + 1`, `" * f`), and requires no parser changes.

2. **Different character:** Use a different ditto character (e.g., `'`). Rejected because Maple uses `"` and we want compatibility.

3. **Parser-level disambiguation:** Emit StringLit always, then in the parser check for empty or special strings. Rejected because it requires the parser to know about ditto semantics.

### LANG-02 (Lambda): Representation Choice

Two options for lambda storage:

1. **Reuse Value::Procedure (RECOMMENDED):** A lambda `q -> expr` becomes `Procedure { params: ["q"], body: [expr], locals: [], remember: false }`. This reuses all calling infrastructure (arity check, variable save/restore, memo). The only difference from `proc(q) expr; end` is syntactic.

2. **New Value::Lambda variant:** Would require duplicating the call_procedure logic or refactoring to a shared trait. Not worth the complexity for single-param functions.

### LANG-04 (Fractional Powers): Implementation Strategy

Three levels of implementation, in order of increasing complexity:

1. **Display-only (MINIMUM VIABLE):** `q^(1/4)` evaluates to a FPS monomial at exponent 1 with a "scale factor" metadata. Division `series / q^(1/4)` is an error. This doesn't meet the requirement.

2. **New Value variant (RECOMMENDED):** Add `Value::FractionalPowerSeries { inner: FormalPowerSeries, denom: i64 }` where exponent k in `inner` represents `q^(k/denom)`. Arithmetic between fractional series uses LCD to unify denominators. Display shows `q^(k/d)` format. This is self-contained and doesn't modify qsym-core.

3. **Modify FormalPowerSeries:** Add a `exponent_denom: i64` field to FPS itself. Rejected: too invasive, would require updating all 89+ functions that create/manipulate FPS.

For option 2, the key operations:
- `q^(1/4)` -> FractionalPowerSeries { inner: monomial(q, 1, 1, POLY_ORDER), denom: 4 }
- `theta2(q,100) / q^(1/4)` -> rescale theta2 exponents by 4 (multiply all keys by 4), create FractionalPowerSeries { inner: shifted_series, denom: 4 }
- Display: for each (k, coeff) in inner, display as `coeff*q^(k/denom)` or `coeff*q^k` when k%denom==0

### LANG-05 (Option/Local Reorder): Simple Loop

Replace the two sequential if-blocks with a loop. The loop accepts `local` or `option` in any order, accumulating both. Multiple `local` blocks or multiple `option` blocks are allowed (Maple allows this). The loop terminates when neither keyword is seen.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `%` only for last result | `%` works, `"` needs adding | Phase 47 | Maple compatibility |
| proc() only for functions | proc() works, `->` lambda needed | Phase 47 | Concise function definitions |
| Integer exponents only | Integer exponents, fractional needed | Phase 47 | Theta function manipulation |
| local-then-option only | Need either order | Phase 47 | Maple compatibility |

## Open Questions

1. **Multi-param lambdas?**
   - What we know: The requirement shows single-param `q -> expr`. Maple supports `(a,b) -> expr`.
   - What's unclear: Should we support multi-param lambdas now?
   - Recommendation: Implement single-param only for now. The `(a,b) -> expr` syntax would conflict with parenthesized expressions. Defer to a future phase.

2. **Fractional power arithmetic completeness?**
   - What we know: The primary use case is `theta2(q,100)/q^(1/4)`.
   - What's unclear: What operations should FractionalPowerSeries support? Add, sub, mul, div? Powers?
   - Recommendation: Support div (series/q^(p/d)), mul (series*q^(p/d)), and display. Add/sub between two fractional series with same denom. Error for incompatible operations. This covers the theta2 use case.

3. **Ditto in scripts vs REPL?**
   - What we know: `"` ditto uses `env.last_result`, which is set after each statement.
   - What's unclear: In scripts with multiple statements, does ditto refer to the previous statement's result?
   - Recommendation: Yes, same as `%`. Both `%` and `"` reference `env.last_result`, which is updated after each statement evaluation.

## Sources

### Primary (HIGH confidence)
- **Codebase inspection:** Direct reading of parser.rs (1,678 lines), lexer.rs (614 lines), ast.rs (359 lines), eval.rs (10,405 lines), token.rs (194 lines), environment.rs (153 lines), repl.rs (580 lines), main.rs (389 lines), format.rs (1,153 lines)
- **FormalPowerSeries:** series/mod.rs -- BTreeMap<i64, QRat> with integer exponents confirmed
- **theta2 implementation:** qseries/theta.rs lines 140-186 -- uses integer powers (1, 9, 25, 49...) representing q^(2n+1)^2 / 8 scaling

### Secondary (MEDIUM confidence)
- **Maple ditto semantics:** Based on general Maple knowledge -- `"` references the last computed result, equivalent to `%` in some versions

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all changes are in existing code, no new dependencies
- Architecture: HIGH - patterns directly observed from codebase (Token/Lexer/Parser/AST/Eval pipeline)
- Pitfalls: HIGH - ditto/string disambiguation identified from direct lexer code reading
- Fractional powers: MEDIUM - implementation strategy is sound but the Value variant approach has arithmetic edge cases

**Research date:** 2026-02-20
**Valid until:** 2026-03-20 (stable codebase, no external dependency changes)
