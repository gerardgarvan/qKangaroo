# Phase 33: Symbolic Variable Foundation - Research

**Researched:** 2026-02-19
**Domain:** CLI evaluator extension -- symbols, q-as-parameter, polynomial arithmetic, variable management
**Confidence:** HIGH (all findings based on direct codebase analysis)

## Summary

Phase 33 adds a `Value::Symbol` variant to the evaluator, makes undefined variable names produce symbols instead of errors, demotes `q` from a keyword token to a regular identifier (falling through to symbol evaluation), enables q-polynomial arithmetic in the REPL, makes the series display system use the actual variable name from the `SymbolId` instead of hardcoded `"q"`, and adds `restart`/`anames`/unassign commands.

The core implementation requires changes across 7 files in qsym-cli and 1 file in qsym-core. The most architecturally significant change is making `q` no longer a keyword -- currently `Token::Q` is a dedicated token variant and `AstNode::Q` is a dedicated AST variant. Both need to be eliminated, with `q` becoming a regular `Token::Ident("q")` / `AstNode::Variable("q")` that falls through to the symbol evaluation path. The second major change is adding `Value::Symbol(String)` to the Value enum and changing the evaluator's `AstNode::Variable` handling from "error on unknown" to "return symbol on unknown."

**Primary recommendation:** Implement in 4 plans: (1) Value::Symbol + symbol fallback + q demotion, (2) q-polynomial arithmetic and display, (3) variable management commands (restart/anames/unassign), (4) function dispatch awareness of symbol/series args for q-as-parameter.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Silent Maple-like: typing an undefined name returns a Symbol with no warning or annotation
- ALL undefined names become symbols (not just single-letter) -- matches Maple exactly
- No typo detection or "did you mean?" suggestions
- `q` is NOT pre-defined or special -- it behaves like any other undefined name (becomes a symbol on first use), matching Maple's behavior when importing the q-series package
- `q` is NOT protected from reassignment -- `q := 5` is allowed (Maple-like, user's responsibility)
- Any symbol can be used as the base variable -- `etaq(t, 1, 20)` works and produces a series in `t`
- Series internally tracks which symbol was used as its variable and displays accordingly -- `etaq(t, 1, 20)` displays terms in `t`, not `q`
- Full q-polynomials accepted as function arguments -- `q^2`, `q^(-1)`, `2*q^3`, `q^2 + q + 1` all work
- Standalone polynomial arithmetic works at the REPL -- `(q^2 + 1) * (q + 1)` evaluates and returns `q^3 + q^2 + q + 1`
- Display matches Maple: polynomials (finite terms) display as exact polynomials without truncation; infinite series display with `O(q^n)` notation
- Maple unassign syntax: `x := 'x'` turns a defined variable back into a bare symbol
- `restart` command clears all user-defined variables at once (Maple-style)
- Variable listing command available (like Maple's `anames()`) to show all currently defined variables and values

### Claude's Discretion
- Symbol arithmetic outside function calls: how symbols behave in expressions like `f + 1` or `a * b` (minimum needed for phase success criteria)
- Symbol display formatting in LaTeX mode
- Internal representation of q-polynomials passed to functions (evaluate to FormalPowerSeries vs keep symbolic)
- What exactly `restart` clears (user vars only vs also `ans` and output history)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SYM-01 | Bare variable names evaluate to a Symbol value without error | Add `Value::Symbol(String)` variant; change `AstNode::Variable` eval from `Err(UnknownVariable)` to `Ok(Value::Symbol(name))` |
| SYM-02 | `q` can be passed as a function parameter (e.g., `etaq(q, 1, 20)`) | Demote `Token::Q` to `Token::Ident("q")`; remove `AstNode::Q`; add `extract_symbol()` helper and teach function dispatch to accept `Value::Symbol` as variable parameter |
| SYM-03 | Monomial expressions like `q^2` can be used as function arguments | `Value::Symbol` ^ `Value::Integer` produces `Value::Series(monomial)`; function dispatch uses `extract_series_or_monomial()` to accept either `Value::Series` or `Value::Symbol` args |
| SYM-04 | Assigned variables still work; assignment takes precedence over symbol fallback | Already works -- `env.get_var()` returns `Some` for assigned vars, `None` triggers new symbol fallback path |
</phase_requirements>

## Standard Stack

### Core (no new dependencies)

This phase requires zero new library dependencies. All changes are to existing Rust source in `qsym-cli` and `qsym-core`:

| Crate | Files Modified | Purpose |
|-------|---------------|---------|
| qsym-cli | token.rs, lexer.rs, ast.rs, parser.rs, eval.rs, format.rs, commands.rs, repl.rs | Symbol support, q demotion, polynomial display, commands |
| qsym-core | series/display.rs | Variable-name-aware series display |

## Architecture Patterns

### Current Architecture (What Exists)

```
token.rs: Token::Q is a keyword variant (separate from Token::Ident)
lexer.rs: "q" -> Token::Q (line 145)
ast.rs:   AstNode::Q is a dedicated variant (separate from AstNode::Variable)
parser.rs: Token::Q -> AstNode::Q (line 148-151)
eval.rs:  AstNode::Q -> FormalPowerSeries::monomial(env.sym_q, 1, 1, order) (line 622-630)
          AstNode::Variable(name) -> Err(UnknownVariable) when not in env (line 642-647)
          All function dispatch uses env.sym_q for the variable (42 occurrences)
display.rs: let var = "q" hardcoded (line 14)
format.rs: fps_to_latex uses hardcoded "q" in LaTeX output
```

### Target Architecture (What We Build)

```
token.rs: Token::Q removed; q is Token::Ident("q")
lexer.rs: "q" -> Token::Ident("q".to_string()) (no special case)
ast.rs:   AstNode::Q removed; q is AstNode::Variable("q")
parser.rs: Token::Ident("q") -> AstNode::Variable("q") (falls through existing Ident path)
eval.rs:  AstNode::Variable(name) -> env.get_var(name) else Value::Symbol(name)
          Value::Symbol("q") + arithmetic -> FPS monomial (q^1)
          Value::Symbol("q") ^ Value::Integer(n) -> FPS monomial (q^n)
          Function dispatch accepts Value::Symbol as variable parameter
          Function dispatch accepts Value::Series as monomial/polynomial parameter
value:    Value::Symbol(String) added as 11th variant
display.rs: uses SymbolRegistry to look up variable name (no longer hardcoded)
format.rs: format_value/format_latex handle Value::Symbol; series display uses actual variable name
commands.rs: restart command, anames function, unassign via x := 'x'
```

### Pattern 1: Symbol Evaluation Fallback

**What:** When a variable name is not found in the environment, return a `Value::Symbol` instead of an error.

**Current code (eval.rs line 642-647):**
```rust
AstNode::Variable(name) => match env.get_var(name) {
    Some(val) => Ok(val.clone()),
    None => Err(EvalError::UnknownVariable {
        name: name.clone(),
    }),
},
```

**New code:**
```rust
AstNode::Variable(name) => match env.get_var(name) {
    Some(val) => Ok(val.clone()),
    None => Ok(Value::Symbol(name.clone())),
},
```

**Impact:** This single change satisfies SYM-01 and SYM-04 simultaneously.

### Pattern 2: Q Demotion (Token + AST)

**What:** Remove `Token::Q` and `AstNode::Q` so `q` flows through the normal identifier path.

**Lexer change (lexer.rs line 143-146):**
```rust
// Before:
"q" => Token::Q,
// After: remove this branch, q falls through to Token::Ident
```

**Parser change:** Remove the `Token::Q => { self.advance(); AstNode::Q }` arm. Since `q` is now `Token::Ident("q")`, it falls through to the existing `Token::Ident` arm and produces `AstNode::Variable("q")`.

**Eval change:** Remove the `AstNode::Q => { ... }` arm. When `q` is not assigned, the symbol fallback produces `Value::Symbol("q")`. Arithmetic operations on `Value::Symbol("q")` with integers produce `Value::Series` monomials.

### Pattern 3: Symbol Arithmetic (Lazy FPS Promotion)

**What:** When a `Value::Symbol` participates in arithmetic, promote it to a `Value::Series` (monomial q^1) and proceed with series arithmetic.

**Recommended approach for symbol arithmetic:**
```rust
/// Try to promote a Value::Symbol to a FPS monomial (var^1).
fn symbol_to_series(name: &str, env: &mut Environment) -> Value {
    let sym_id = env.symbols.intern(name);
    Value::Series(FormalPowerSeries::monomial(sym_id, QRat::one(), 1, env.default_order))
}
```

Operations:
- `Value::Symbol ^ Value::Integer(n)` -> monomial series `var^n` (handles `q^2`, `q^(-1)`)
- `Value::Integer * Value::Symbol` -> `coeff * var^1` series (handles `2*q`)
- `Value::Symbol + Value::Symbol` -> promote both, series add (handles `q + q`)
- `Value::Symbol + Value::Integer` -> promote symbol, series add (handles `q + 1`)
- `Value::Symbol + Value::Series` -> promote symbol, series add (handles `q + f`)

The key insight is that `Value::Symbol` only needs to survive until it hits arithmetic -- at that point it immediately becomes a `Value::Series`. This keeps the Value enum simple and reuses all existing series arithmetic.

### Pattern 4: Variable-Aware Display

**What:** The FormalPowerSeries Display trait currently uses hardcoded `"q"`. It needs to use the actual variable name.

**Problem:** `fmt::Display` has no access to the `SymbolRegistry`. The `FormalPowerSeries` stores `variable: SymbolId` but Display can't resolve this to a name.

**Recommended solutions (choose one):**

**Option A (recommended): Store variable name directly in FPS.**
Add a `variable_name: String` field (or `Option<String>`) to `FormalPowerSeries`. Set it during construction. Display uses it. This avoids threading the registry everywhere.

**Option B: Pass variable name through CLI format functions.**
Keep the core Display as-is (hardcoded "q"). In the CLI's `format_value()` and `format_latex()`, use a custom `format_series(fps, &env.symbols)` function instead of `fps.Display`. This keeps qsym-core unchanged but requires CLI format functions to accept the environment.

**Option C: Add a `display_with` method on FPS.**
```rust
impl FormalPowerSeries {
    pub fn display_with<'a>(&'a self, registry: &'a SymbolRegistry) -> FpsDisplay<'a> { ... }
}
```

**Recommendation:** Option B is cleanest for this phase. The CLI already owns the `Environment` and `SymbolRegistry`. Change `format_value` and `format_latex` to accept `&SymbolRegistry` as a parameter, and use `env.symbols.name(fps.variable())` to get the display name. This keeps the core crate unchanged and localizes the change to CLI formatting. The core Display can keep `"q"` as a default/fallback.

### Pattern 5: Function Dispatch with Symbol Arguments

**What:** Functions like `etaq` need to accept `Value::Symbol("q")` as their variable parameter.

**Current `etaq` dispatch (eval.rs line 993-1001):**
```rust
"etaq" => {
    expect_args(name, args, 3)?;
    let b = extract_i64(name, args, 0)?;
    let t = extract_i64(name, args, 1)?;
    let order = extract_i64(name, args, 2)?;
    let result = qseries::etaq(b, t, env.sym_q, order);
    Ok(Value::Series(result))
}
```

**Phase 33 goal -- `etaq(q, 1, 20)` works:**
The function dispatch needs to detect that argument 0 is `Value::Symbol("q")` (the variable), extract the symbol, look up or intern its `SymbolId`, and rearrange the remaining arguments accordingly.

**New helper:**
```rust
/// Extract a SymbolId from a Value::Symbol, interning it in the registry.
fn extract_symbol(name: &str, args: &[Value], index: usize, env: &mut Environment) -> Result<SymbolId, EvalError> {
    match &args[index] {
        Value::Symbol(s) => Ok(env.symbols.intern(s)),
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "symbol (variable name)",
            got: other.type_name().to_string(),
        }),
    }
}
```

**New signature support for `etaq`:**
```rust
"etaq" => {
    // Support both old: etaq(b, t, order) and new: etaq(q, b, order) or etaq(q, b, t)
    // When first arg is a Symbol, it's the variable parameter (Maple-style)
    if args.len() >= 2 && matches!(&args[0], Value::Symbol(_)) {
        // Maple-style: etaq(var, b_or_delta, order)
        let sym = extract_symbol(name, args, 0, env)?;
        let b = extract_i64(name, args, 1)?;
        let order = extract_i64(name, args, 2)?;
        let result = qseries::etaq(b, 1, sym, order);
        Ok(Value::Series(result))
    } else {
        // Legacy: etaq(b, t, order)
        expect_args(name, args, 3)?;
        let b = extract_i64(name, args, 0)?;
        let t = extract_i64(name, args, 1)?;
        let order = extract_i64(name, args, 2)?;
        let result = qseries::etaq(b, t, env.sym_q, order);
        Ok(Value::Series(result))
    }
}
```

Note: The exact signature mapping for `etaq` will be refined in Phase 34. For Phase 33, the goal is just "q is accepted as a function parameter and the result is the correct q-series." The simplest approach is to recognize `Value::Symbol` as the first arg and use it as the variable.

### Pattern 6: Extracting Monomials from Series Arguments

**What:** `aqprod(q^2, q, 5)` needs to work. Here `q^2` evaluates to `Value::Series` (a monomial with one term), `q` is `Value::Symbol`, and `5` is `Value::Integer`.

**Recommended helper:**
```rust
/// Extract a QMonomial from a Value that is either a Symbol (meaning var^1)
/// or a Series with a single nonzero term (monomial).
fn extract_monomial_from_value(
    name: &str,
    args: &[Value],
    index: usize,
) -> Result<QMonomial, EvalError> {
    match &args[index] {
        Value::Series(fps) => {
            // Check it's a monomial (single nonzero term)
            let terms: Vec<_> = fps.iter().collect();
            if terms.len() == 1 {
                let (&power, coeff) = terms[0];
                Ok(QMonomial::new(coeff.clone(), power))
            } else if terms.is_empty() {
                Ok(QMonomial::new(QRat::zero(), 0))
            } else {
                // It's a polynomial, not a monomial
                Err(EvalError::ArgType { ... })
            }
        }
        Value::Symbol(_) => {
            // Symbol is equivalent to var^1
            Ok(QMonomial::q_power(1))
        }
        Value::Integer(n) => {
            // Integer is a constant monomial
            Ok(QMonomial::constant(QRat::from(n.clone())))
        }
        ...
    }
}
```

### Pattern 7: Polynomial Display (No Truncation)

**What:** A `FormalPowerSeries` with finite terms (no truncation) should display without `O(q^n)`.

**Decision insight:** Polynomials are FPS with a "natural" truncation order that equals max_power + 1, but the user decision says "polynomials display as exact polynomials without truncation." This means: when the FPS came from polynomial arithmetic (not a function that produces infinite series), suppress the `O(q^n)` suffix.

**Options:**
1. **Add a flag to FPS:** `is_polynomial: bool` -- if true, display omits `O(...)`.
2. **Use a sentinel truncation order:** e.g., `i64::MAX` means "exact polynomial."
3. **Check if all coefficients are below truncation order and the series appears finite.**

**Recommendation:** Option 2 is simplest. Use `i64::MAX` as the truncation order for exact polynomials. The display code already only prints `O(q^N)` at the end -- if N is `i64::MAX`, suppress it. When symbols participate in arithmetic, produce polynomials with `i64::MAX` truncation. When series from functions (which have finite truncation) are added to polynomials, the result takes the finite truncation order.

### Anti-Patterns to Avoid

- **Don't make Symbol carry a SymbolId.** The Symbol variant should hold a `String`, not a `SymbolId`, because symbols exist independently of the evaluation context. The SymbolId is only needed when a symbol is used as a series variable (at that point, intern it).

- **Don't try to make all Value types work with all arithmetic ops.** Symbol arithmetic should eagerly promote to Series. Don't build a full symbolic algebra system -- that's not the goal. If someone types `f + 1` where `f` is undefined, it's acceptable to produce a TypeError since `f` is not a series variable. The minimum needed is: symbol^integer -> monomial, integer*symbol -> monomial, symbol used as function variable parameter.

- **Don't break existing tests.** The q demotion changes how the parser and many tests work. Currently `parse("q")` returns `AstNode::Q`. After the change, it returns `AstNode::Variable("q")`. All test assertions referencing `AstNode::Q` must be updated. Similarly, `Token::Q` references in lexer tests must change.

- **Don't change env.sym_q yet.** Keep `env.sym_q` as the cached SymbolId for "q". It's still useful as a default. Functions that don't receive an explicit variable symbol should continue using `env.sym_q`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Symbol interning | Custom string dedup | Existing `SymbolRegistry` in qsym-core | Already built, O(1) lookup, stable IDs |
| Polynomial arithmetic | New polynomial type | `FormalPowerSeries` with high truncation | FPS already handles sparse coefficients, all arithmetic ops exist |
| Series display | Separate polynomial formatter | Modified FPS Display with truncation sentinel | One code path for both polynomials and series |
| Edit distance for "did you mean" | N/A (DISABLED for variables) | N/A | User decision: no typo detection for variables |

## Common Pitfalls

### Pitfall 1: Token::Q Removal Cascade
**What goes wrong:** Removing `Token::Q` breaks every test that references `Token::Q` or `AstNode::Q` -- potentially 30+ test assertions across lexer.rs, parser.rs, eval.rs, and integration tests.
**Why it happens:** `q` is deeply embedded as a keyword throughout the system.
**How to avoid:** Do the Token/AST changes in a single plan. Use search-and-replace. Change `Token::Q` to `Token::Ident("q".to_string())` in all test assertions. Change `AstNode::Q` to `AstNode::Variable("q".to_string())`.
**Warning signs:** Compilation errors in test modules after removing `Token::Q` / `AstNode::Q`.

### Pitfall 2: UnknownVariable Error Tests Break
**What goes wrong:** Existing tests that assert `EvalError::UnknownVariable` for undefined names will fail because undefined names now produce `Value::Symbol`.
**Why it happens:** The symbol fallback means `eval_expr(AstNode::Variable("unknown"))` no longer errors.
**How to avoid:** Update tests: `eval_variable_not_found` should now assert `Value::Symbol("unknown")` instead of `EvalError::UnknownVariable`. The integration test `c_flag_eval_error` testing `undefined_var` will need updating -- this is actually a design question (does `-c "f"` print the symbol `f` or still error?). Per user decision (silent, Maple-like), it should print `f`.
**Warning signs:** Integration test `c_flag_eval_error` / `exit_02_eval_error_exit_code` failures.

### Pitfall 3: Display Without SymbolRegistry Access
**What goes wrong:** The `fmt::Display for FormalPowerSeries` in qsym-core cannot access the `SymbolRegistry` because Display takes no extra parameters.
**Why it happens:** Rust's Display trait signature is fixed: `fn fmt(&self, f: &mut Formatter) -> Result`.
**How to avoid:** Don't try to fix the core Display. Instead, have the CLI's `format_value()` and `format_latex()` use a custom series formatting function that takes `&SymbolRegistry`. The core Display can remain as a fallback with `"q"` (useful for debug output).
**Warning signs:** Trying to pass `SymbolRegistry` through Display leads to lifetime issues and trait object problems.

### Pitfall 4: Existing Function Call `aqprod(q,q,infinity,20)` Breaks
**What goes wrong:** Currently `q` in `aqprod(q,q,infinity,20)` evaluates to `Value::Series(q^1 + O(q^20))`. After Phase 33, it evaluates to `Value::Symbol("q")`. The existing `extract_i64` calls for `aqprod` will reject `Value::Symbol`.
**Why it happens:** `aqprod` currently expects all-integer args (decomposed monomial). With q demotion, users passing `q` get a Symbol.
**How to avoid:** This is intentional -- Phase 33 needs to update aqprod dispatch to recognize the new signature. But the OLD signature `aqprod(1, 1, 1, infinity, 20)` must also continue working. Use argument count and first-arg type to disambiguate.
**Warning signs:** Existing eval.rs tests for `aqprod(q,q,infinity,20)` pattern failing.

### Pitfall 5: Unassign Syntax `x := 'x'` Requires Lexer/Parser Changes
**What goes wrong:** The lexer doesn't have single-quote string literals. `'x'` will cause a parse error.
**Why it happens:** The lexer only supports double-quoted strings (`"hello"`).
**How to avoid:** Add single-quoted string support to the lexer (producing `Token::StringLit` or a new `Token::QuotedIdent` variant), then handle `x := 'x'` in the eval Assign arm as "if value is a quoted identifier matching the variable name, remove the binding."
**Warning signs:** Parse error when user types `x := 'x'`.

### Pitfall 6: Polynomial Truncation Order Sentinel
**What goes wrong:** Using `i64::MAX` as a polynomial truncation order might cause overflow in arithmetic operations that compute `min(trunc_a, trunc_b)` or `trunc_a + trunc_b`.
**Why it happens:** Existing series arithmetic uses truncation orders in comparisons and arithmetic.
**How to avoid:** Audit all uses of `truncation_order` in qsym-core arithmetic. The `min()` calls are safe (min(i64::MAX, 20) = 20). Addition/subtraction of truncation orders may not occur. Verify this. If any overflow risk exists, use a large-but-not-MAX sentinel like `1_000_000_000`.
**Warning signs:** Integer overflow panics during polynomial-series mixed arithmetic.

## Code Examples

### Example 1: Value::Symbol Variant

```rust
// In eval.rs, add to Value enum:
/// A symbolic variable name (undefined name fallback).
Symbol(String),

// In type_name():
Value::Symbol(_) => "symbol",

// In format_value():
Value::Symbol(name) => name.clone(),

// In format_latex():
Value::Symbol(name) => name.clone(),
```

### Example 2: Lexer Q Demotion

```rust
// In lexer.rs, change the keyword match:
let token = match word {
    "infinity" => Token::Infinity,
    // "q" removed -- falls through to Ident
    _ => Token::Ident(word.to_string()),
};
```

### Example 3: Series Format with Variable Name

```rust
// In format.rs, change format_value:
pub fn format_value(val: &Value, symbols: &SymbolRegistry) -> String {
    match val {
        Value::Series(fps) => format_series(fps, symbols),
        Value::Symbol(name) => name.clone(),
        // ... other arms unchanged
    }
}

fn format_series(fps: &FormalPowerSeries, symbols: &SymbolRegistry) -> String {
    let var = symbols.name(fps.variable());
    // ... same logic as current Display but using `var` from registry
}
```

### Example 4: Restart Command

```rust
// In commands.rs, add to parse_command:
"restart" => {
    if words.len() == 1 && !trimmed.contains('(') {
        Some(Command::Restart)
    } else {
        None
    }
}

// In execute_command:
Command::Restart => {
    env.reset();
    CommandResult::Output("Restart.".to_string())
}
```

### Example 5: Anames Function

```rust
// In dispatch(), add:
"anames" => {
    expect_args(name, args, 0)?;
    let mut names: Vec<String> = env.variables.keys().cloned().collect();
    names.sort();
    Ok(Value::List(names.into_iter().map(|n| Value::String(n)).collect()))
}
```

### Example 6: Unassign via Quoted Identifier

```rust
// In eval.rs, Assign handling:
AstNode::Assign { name, value } => {
    // Check for unassign: x := 'x'
    if let AstNode::StringLit(s) = value.as_ref() {
        if s == name {
            env.variables.remove(name);
            return Ok(Value::Symbol(name.clone()));
        }
    }
    let val = eval_expr(value, env)?;
    env.set_var(name, val.clone());
    Ok(val)
}
```

Note: This requires the single-quote lexer extension to produce a `StringLit` for `'x'`, or a new `QuotedIdent` token type.

## Discretion Recommendations

### Symbol Arithmetic Outside Function Calls
**Recommendation:** Support only the minimum needed for the success criteria and Maple-compatible workflows:
- `Symbol ^ Integer` -> Series monomial (needed for `q^2`, `q^(-1)`)
- `Integer * Symbol` and `Symbol * Integer` -> Series monomial (needed for `2*q^3`)
- `Symbol + Symbol`, `Symbol + Integer`, etc. -> promote both to Series, use series arithmetic (needed for `q^2 + q + 1`)
- `Symbol + Series` / `Series + Symbol` -> promote symbol, series add
- Pure symbol arithmetic like `a * b` where neither is a known series variable: produce a TypeError. This is acceptable because Maple's q-series package also doesn't do general symbolic algebra.
- Rationale: Lazy promotion to FPS handles everything needed. No need for a symbolic algebra system.

### Symbol Display in LaTeX Mode
**Recommendation:** Symbols in LaTeX should display as-is: `Value::Symbol("q")` -> `q`, `Value::Symbol("alpha")` -> `\alpha` (detect Greek names). For simplicity in Phase 33, just output the raw name. Greek letter detection can be added in Phase 40 (docs).

### Internal Representation of Q-Polynomials
**Recommendation:** Q-polynomials ARE `FormalPowerSeries` with a high sentinel truncation order (e.g., `i64::MAX` or a chosen large value). This avoids a separate Polynomial type and reuses all existing series arithmetic. When a polynomial (sentinel truncation) interacts with a series (finite truncation), the result uses the finite truncation order.

### What `restart` Clears
**Recommendation:** `restart` clears user variables AND `last_result` (the `%` reference) AND resets `default_order` to 20. This matches the existing `env.reset()` method behavior. It does NOT clear the symbol registry (that's append-only by design). The `restart` command is essentially the same as `clear` but with the Maple-standard name. Consider making `clear` an alias for `restart` or keeping both.

## File-by-File Change Map

| File | Changes | Lines Affected |
|------|---------|---------------|
| `crates/qsym-cli/src/token.rs` | Remove `Token::Q` variant | ~3 lines, plus test updates |
| `crates/qsym-cli/src/lexer.rs` | Remove `"q" => Token::Q` keyword match | ~1 line, plus test updates (~10 tests) |
| `crates/qsym-cli/src/ast.rs` | Remove `AstNode::Q` variant | ~2 lines, plus test updates |
| `crates/qsym-cli/src/parser.rs` | Remove `Token::Q => AstNode::Q` arm; remove `token_name(Token::Q)` | ~5 lines, plus test updates (~5 tests) |
| `crates/qsym-cli/src/eval.rs` | Add `Value::Symbol`; change Variable fallback; remove `AstNode::Q` arm; add symbol arithmetic in eval_pow/eval_mul/eval_add/eval_sub; update function dispatch for symbol args; add `anames` function; add unassign logic in Assign handling | ~100-150 new/changed lines, ~30 test updates |
| `crates/qsym-cli/src/format.rs` | Add `&SymbolRegistry` param to `format_value`/`format_latex`; add `format_series`; handle `Value::Symbol`; handle polynomial display (suppress O(...) for sentinel truncation) | ~50-80 new/changed lines |
| `crates/qsym-cli/src/commands.rs` | Add `Command::Restart`; parse "restart"; add single-quote lexer for unassign | ~20-30 new lines |
| `crates/qsym-cli/src/repl.rs` | Update `canonical_function_names` to include `anames`; update call sites for `format_value` with new signature | ~5-10 changed lines |
| `crates/qsym-core/src/series/display.rs` | Optionally: suppress `O(...)` for sentinel truncation order. Or leave as-is if CLI handles all display. | ~5 lines |
| `crates/qsym-cli/src/main.rs` | Update `format_value` calls to pass `&env.symbols` | ~5 changed lines |
| `crates/qsym-cli/src/script.rs` | Update `format_value` calls | ~2-3 changed lines |
| `crates/qsym-cli/tests/cli_integration.rs` | Update tests expecting "undefined variable" error to expect symbol output instead | ~10-15 changed lines |

## Test Impact Analysis

### Tests That Must Change

**eval.rs tests (~15 changes):**
- `eval_variable_not_found`: now expects `Value::Symbol("unknown")` not `EvalError::UnknownVariable`
- `eval_error_display_unknown_var`: may become dead (no more UnknownVariable for variables)
- `eval_q_creates_series`: logic changes -- `q` is now `Variable("q")` -> `Symbol("q")`, not a series. Standalone `q` returns `Symbol`, only `q^1` or arithmetic involving `q` produces a series.
- All tests using `AstNode::Q`: change to `AstNode::Variable("q".to_string())`
- All tests checking `Token::Q` in lexer: change to `Token::Ident("q".to_string())`
- `eval_series_add` (q+q), `eval_scalar_mul_series` (3*q): These currently use `AstNode::Q`. Need to change to `AstNode::Variable("q")` and verify that Symbol arithmetic produces correct series.

**Integration tests (~5 changes):**
- `c_flag_eval_error`: `undefined_var` no longer errors -- now prints `undefined_var` as a symbol
- `exit_02_eval_error_exit_code`: same issue
- `exit_02_eval_error_in_script`: `undefined_var` in script now succeeds
- `err_01_eval_error_shows_filename_line`: `undefined_var` no longer errors
- `err_04_script_fail_fast`: `undefined_a` / `undefined_b` no longer error

**These integration tests need new scenarios for actual errors** (e.g., type errors, wrong arg counts) since undefined variables are no longer errors.

### New Tests Needed

- Symbol evaluation: `f` -> `Value::Symbol("f")`
- Symbol assignment precedence: `x := 42; x` -> 42
- Q demotion: `q` -> `Value::Symbol("q")`
- Q reassignment: `q := 5; q` -> 5
- Symbol arithmetic: `q^2` -> series with single term at power 2
- Polynomial arithmetic: `(q+1)*(q+1)` -> `1 + 2*q + q^2`
- Polynomial display: no `O(...)` suffix
- `etaq(q, 1, 20)` -> correct series (symbol as variable parameter)
- `aqprod(q^2, q, 5)` -> correct polynomial
- `restart` clears variables
- `anames()` lists variables
- `x := 'x'` unassigns x
- Variable-aware display: `etaq(t, 1, 20)` shows terms in `t`
- Format with SymbolRegistry: series in different variables display correctly

## Open Questions

1. **Single-quote lexer extension for unassign**
   - What we know: Maple uses `x := 'x'` for unassign. The lexer currently only has double-quoted strings.
   - What's unclear: Should single quotes produce a `Token::StringLit` or a new `Token::QuotedIdent`? Should single quotes be usable for regular strings too?
   - Recommendation: Add single-quoted string support as `Token::StringLit` (same as double-quoted). The Assign handler checks if the value is a string matching the variable name and treats it as unassign. This is the simplest approach and avoids a new token type.

2. **`format_value` signature change propagation**
   - What we know: Adding `&SymbolRegistry` to `format_value` changes every call site.
   - What's unclear: How many call sites exist? (Likely: repl.rs, main.rs, script.rs, commands.rs, plus test helpers.)
   - Recommendation: Accept the signature change. It's a one-time cost. Alternatively, store a reference to the registry in a thread-local or use a different approach, but the explicit parameter is cleanest.

3. **Polynomial sentinel value**
   - What we know: Need a way to distinguish "exact polynomial" from "truncated series."
   - What's unclear: Whether `i64::MAX` causes any arithmetic issues in qsym-core.
   - Recommendation: Audit qsym-core `truncation_order` usage. If safe, use `i64::MAX`. If not, use a module-level constant like `const POLYNOMIAL_ORDER: i64 = 1_000_000_000`.

## Sources

### Primary (HIGH confidence)
- Direct codebase analysis of all files listed in the change map
- `crates/qsym-cli/src/eval.rs` -- Value enum (line 28-49), EvalError (line 74-106), eval_expr (line 607-681), eval_binop (line 707-913), dispatch (line 952+)
- `crates/qsym-cli/src/token.rs` -- Token enum (line 8-51), Token::Q is keyword (line 16)
- `crates/qsym-cli/src/lexer.rs` -- "q" -> Token::Q (line 145)
- `crates/qsym-cli/src/ast.rs` -- AstNode::Q (line 33)
- `crates/qsym-cli/src/parser.rs` -- Token::Q -> AstNode::Q (line 148-151), function call (line 210-225)
- `crates/qsym-cli/src/environment.rs` -- Environment, sym_q, reset() (line 18-67)
- `crates/qsym-core/src/series/display.rs` -- hardcoded "q" (line 14)
- `crates/qsym-core/src/series/mod.rs` -- FormalPowerSeries struct with variable:SymbolId (line 31-39)
- `crates/qsym-core/src/symbol.rs` -- SymbolId, SymbolRegistry (line 15, 30)
- `.planning/phases/33-symbolic-variable-foundation/33-CONTEXT.md` -- user decisions
- `.planning/ROADMAP.md` -- Phase 33-34 success criteria
- `.planning/REQUIREMENTS.md` -- SYM-01 through SYM-04

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies, all changes to existing code
- Architecture: HIGH - thorough analysis of all affected files, clear change map
- Pitfalls: HIGH - identified through direct code analysis of Token::Q usage, test assertions, Display trait limitations
- Discretion areas: HIGH - recommendations grounded in codebase analysis and Maple compatibility goals

**Research date:** 2026-02-19
**Valid until:** 2026-03-19 (stable -- no external dependency changes)
