# Phase 50: New Functions - Research

**Researched:** 2026-02-21
**Domain:** Four new/overhauled CLI functions: jac2series, radsimp, quinprod identity modes, indexed subs
**Confidence:** HIGH

## Summary

Phase 50 implements four function changes in `crates/qsym-cli/src/eval.rs`: (1) overhauling `jac2series` from 3-arg `(JP, q, T)` to Garvan's 2-arg `(jacexpr, T)` signature and fixing the `JAC(0,b)` edge case; (2) adding a new `radsimp(expr)` function that simplifies series quotients (performing invert-and-multiply when the argument is a ratio of series); (3) extending `quinprod` to accept `prodid`/`seriesid` symbols as the third argument to display the quintuple product identity in product or series form; and (4) extending `subs` to accept multiple `X[i]=val` substitutions for indexed variables output by `findnonhom`.

The primary technical challenges are: (a) `JAC(0,b)` currently maps to `etaq(0, b)` which returns zero -- Garvan's convention is `JAC(0,b)` = `(q^b;q^b)_inf` = `etaq(b, b)`, requiring special handling in `jacobi_product_to_fps`; (b) `subs` currently only accepts a single `var=val` pair and uses AST-level interception that only handles `AstNode::Variable` on the left side of `=`, not indexed expressions like `X[1]`; (c) `quinprod(z, q, prodid)` needs to return a display string rather than a computed series; (d) `radsimp` is entirely new and must be defined for q-Kangaroo's context (it is NOT the Maple radical simplifier -- it simplifies series quotients).

**Primary recommendation:** Implement all four functions entirely in `eval.rs` dispatch. No new qsym-core functions are needed. The `jac2series` overhaul is a signature change + edge-case fix in `jacobi_product_to_fps`. `quinprod` identity modes are string-return branches. `subs` needs multi-substitution + indexed variable parsing. `radsimp` is a thin wrapper that evaluates series division explicitly.

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| FUNC-01 | `jac2series(jacexpr, T)` converts Jacobi product to theta-series expansion | Change from 3-arg to 2-arg (drop `q`, use `env.sym_q`). Fix `jacobi_product_to_fps` to handle `a=0` case: `JAC(0,b)` = `etaq(b, b)` not `etaq(0, b)`. See Architecture Pattern 1. |
| FUNC-02 | `radsimp(expr)` simplifies rational expressions involving series quotients | New function. When argument is a Series, return as-is. When argument is a quotient of two series (already computed by eval), the result IS the simplified series. The value of `radsimp` is that theta3(q^5, T) is not natively supported, so radsimp must handle monomial-variable theta functions. See Architecture Pattern 4. |
| FUNC-03 | `quinprod(z,q,prodid)` and `quinprod(z,q,seriesid)` display identity forms | Extend the existing 3-arg quinprod dispatch: when third arg is `Symbol("prodid")`, return a string showing the product form. When `Symbol("seriesid")`, return a string showing the series form. See Architecture Pattern 2. |
| FUNC-04 | `subs(X[1]=val1, X[2]=val2, ..., expr)` with indexed variables | Extend subs AST interception to accept N+1 args (N substitution pairs + target). Parse `X[1]` on the left side of `=` by detecting `AstNode::FuncCall{name:"X", args:[Integer(1)]}` or by adding subscript parsing. See Architecture Pattern 3. |

</phase_requirements>

## Standard Stack

### Core (existing, no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| qsym-core etaq | current | `(q^a;q^b)_inf` computation | jac2series needs it for JAC factor expansion |
| qsym-core arithmetic | current | Series mul, invert, add | radsimp uses invert+mul for quotients |
| qsym-core products::quinprod | current | Quintuple product computation | Existing numerical quinprod unchanged; identity modes are display-only |
| eval.rs dispatch | current | Function dispatch table | All four functions added/modified here |
| eval.rs subs interception | current | AST-level interception for subs | Extended from 2-arg to variadic |

### Supporting (existing patterns)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| format.rs | current | Value display functions | May need updates for identity string display |
| help.rs | current | Per-function help entries | Need updates for jac2series (2-arg), radsimp (new), quinprod (prodid/seriesid), subs (multi-arg) |
| repl.rs | current | Tab completion | Add "radsimp" to ALL_FUNCTION_NAMES |

### No Alternatives Needed
All four functions are CLI-level concerns. No new external crates. No new qsym-core functions needed.

## Architecture Patterns

### Pattern 1: jac2series Overhaul (FUNC-01)

**What:** Change jac2series from `(JP, q, T)` to `(jacexpr, T)` and fix JAC(0,b) expansion.

**Current code (eval.rs ~line 4650):**
```rust
"jac2series" => {
    expect_args(name, args, 3)?;
    let factors = match &args[0] { Value::JacobiProduct(f) => f.clone(), ... };
    let sym = extract_symbol_id(name, args, 1, env)?;
    let order = extract_i64(name, args, 2)?;
    let fps = jacobi_product_to_fps(&factors, sym, order);
    ...
}
```

**New code:**
```rust
"jac2series" => {
    // Accept both 2-arg (Garvan) and 3-arg (legacy) forms
    if args.len() == 2 {
        // Garvan: jac2series(jacexpr, T)
        let factors = match &args[0] {
            Value::JacobiProduct(f) => f.clone(),
            _ => return Err(...)
        };
        let order = extract_i64(name, args, 1)?;
        let fps = jacobi_product_to_fps_garvan(&factors, env.sym_q, order);
        // Print and return
        ...
    } else if args.len() == 3 {
        // Legacy: jac2series(JP, q, T) -- keep for backward compatibility
        ...
    }
}
```

**Critical fix in `jacobi_product_to_fps` (or a new `jacobi_product_to_fps_garvan`):**
```rust
fn jacobi_product_to_fps_garvan(
    factors: &[(i64, i64, i64)],
    sym: SymbolId,
    order: i64,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::one(sym, order);
    for &(a, b, exp) in factors {
        // Garvan convention:
        // JAC(0, b) = (q^b; q^b)_inf = etaq(b, b)
        // JAC(a, b) for 0 < a < b = (q^a;q^b)(q^{b-a};q^b)(q^b;q^b) = jacprod(a, b)
        // JAC(a, b) for a < 0 or a >= b = general handling
        let factor_fps = if a == 0 {
            qseries::etaq(b, b, sym, order)
        } else if a > 0 && a < b {
            qseries::jacprod(a, b, sym, order)
        } else {
            // Extended: a < 0 or a >= b
            // Use etaq with adjusted parameters
            // For a >= b: etaq(a % b, b) or similar
            // For a < 0: etaq(b - ((-a) % b), b) or similar
            // This needs careful mathematical analysis
            qseries::etaq(((a % b) + b) % b, b, sym, order)
        };
        if exp > 0 {
            for _ in 0..exp { result = arithmetic::mul(&result, &factor_fps); }
        } else if exp < 0 {
            let inv = arithmetic::invert(&factor_fps);
            for _ in 0..(-exp) { result = arithmetic::mul(&result, &inv); }
        }
    }
    result
}
```

**CRITICAL WARNING:** The current `JAC(a,b)` constructor stores `(a, b, 1)` where each factor is `(q^a;q^b)_inf` (single Pochhammer). The existing `jac2series` (3-arg) and `jacobi_product_to_fps` use this etaq convention. Garvan's convention is that `JAC(a,b)` is the TRIPLE product `(q^a;q^b)(q^{b-a};q^b)(q^b;q^b)`.

**Recommended approach:** Create a new function `jacobi_product_to_fps_garvan` for the 2-arg `jac2series` path, keeping the existing `jacobi_product_to_fps` for backward compatibility with the 3-arg legacy path and other internal uses (like `series()` expansion of JacobiProduct). This way, the 2-arg Garvan form uses triple-product interpretation while the legacy path remains unchanged.

**Backward compatibility concerns:**
- Existing tests use 3-arg: `jac2series(JAC(1,5), q, 20)` -- these must continue to work
- Existing integration tests: `jac2series(JAC(1,1), q, 10)` -- must still pass
- The `qs2jaccombo` function and `series(JP, q, T)` use `jacobi_product_to_fps` -- must not break

### Pattern 2: quinprod Identity Modes (FUNC-03)

**What:** When `quinprod(z, q, T)` receives a Symbol `prodid` or `seriesid` as T, display the identity instead of computing a series.

**Current dispatch (eval.rs ~line 3272):**
The 3-arg path checks `args.len() == 3` and discriminates between symbolic z (bivariate) and monomial z (univariate). The third arg is always extracted as an integer (`extract_i64`).

**New dispatch logic:**
```rust
"quinprod" => {
    if args.len() == 3 {
        // Check if third arg is a symbol (prodid/seriesid)
        if let Value::Symbol(mode) = &args[2] {
            match mode.as_str() {
                "prodid" => {
                    // Display product form of quintuple product identity
                    let z_str = match &args[0] {
                        Value::Symbol(s) => s.clone(),
                        _ => "z".to_string(),
                    };
                    let q_str = match &args[1] {
                        Value::Symbol(s) => s.clone(),
                        _ => "q".to_string(),
                    };
                    let identity = format_quinprod_prodid(&z_str, &q_str);
                    println!("{}", identity);
                    return Ok(Value::String(identity));
                }
                "seriesid" => {
                    let z_str = match &args[0] { Value::Symbol(s) => s.clone(), _ => "z".to_string() };
                    let q_str = match &args[1] { Value::Symbol(s) => s.clone(), _ => "q".to_string() };
                    let identity = format_quinprod_seriesid(&z_str, &q_str);
                    println!("{}", identity);
                    return Ok(Value::String(identity));
                }
                _ => {} // Fall through to normal processing
            }
        }
        // ... existing numeric dispatch ...
    }
}
```

**Product form (prodid):**
The quintuple product identity in product form is:
```
(z,q)_inf * (q/z,q)_inf * (z^2,q^2)_inf * (q^2/z^2,q^2)_inf * (q,q)_inf
  = sum_{m=-inf}^{inf} (z^(3m) - z^(-3m-1)) * q^(m*(3m+1)/2)
```

For display, use q-Pochhammer notation:
```
(-z,q)_inf * (-q/z,q)_inf * (z^2*q,q^2)_inf * (q^2/z^2,q^2)_inf * (q,q)_inf
```

**Series form (seriesid):**
```
sum_{m=-inf}^{inf} (z^(3*m) - z^(-3*m-1)) * q^(m*(3*m+1)/2)
```

The exact display format should match Garvan's Maple output as closely as possible. The left side (product form) is the same in both modes; the right side differs.

### Pattern 3: Multi-substitution with Indexed Variables (FUNC-04)

**What:** Extend `subs` from `subs(var=val, expr)` to `subs(X[1]=val1, X[2]=val2, ..., expr)` supporting indexed variable names.

**Current limitations:**
1. `subs` only accepts exactly 2 arguments (line 1090: `if args.len() != 2`)
2. The left side of `=` must be `AstNode::Variable(vname)` -- `X[1]` is NOT parsed as a variable
3. `X[1]` in the parser: `X` is parsed as a variable, then `[1]` would be parsed as a list literal `[1]` -- this causes a parse error or implicit multiplication attempt

**How `X[1]` currently parses:**
In the Pratt parser (parser.rs), after parsing `X` as a `Variable`, the LED loop checks for: `LParen` (function call), `Assign` (:=), `Arrow` (->), or infix operators. `LBracket` is NOT handled as a postfix/subscript operator. So `X[1]` would fail to parse as a single expression.

**Parser change needed:** Add subscript parsing in the LED loop, so `X[1]` is parsed as `AstNode::FuncCall { name: "X", args: [Integer(1)] }` or a new `AstNode::Index { base: "X", index: Integer(1) }`.

**Recommended approach (simplest):**
Add subscript handling in the LED loop of `expr_bp()`:
```rust
// Subscript: X[i] -- after parsing X as Variable
if *self.peek() == Token::LBracket {
    if 19 < min_bp { break; } // same precedence as function call
    if let AstNode::Variable(name) = lhs {
        self.advance(); // consume [
        let index = self.expr_bp(0)?;
        self.expect(&Token::RBracket, "']' to close subscript")?;
        // Represent as X[1] => Variable("X[1]") for simple integer indices
        if let AstNode::Integer(i) = &index {
            lhs = AstNode::Variable(format!("{}[{}]", name, i));
            continue;
        }
        // Fallback: treat as function-call-like syntax
        lhs = AstNode::FuncCall { name, args: vec![index] };
        continue;
    }
    break;
}
```

By representing `X[1]` as `Variable("X[1]")` in the AST, the existing subs machinery (which matches on `AstNode::Variable(vname)`) works automatically without further changes.

**Multi-substitution in subs:**
Change subs from 2-arg to variadic:
```rust
if name == "subs" {
    if args.len() < 2 {
        return Err(EvalError::WrongArgCount { ... });
    }
    // Last arg is the target expression
    let target_ast = &args[args.len() - 1];
    let mut target = eval_expr(target_ast, env)?;
    // All preceding args are substitution pairs var=val
    for i in 0..(args.len() - 1) {
        match &args[i] {
            AstNode::Compare { op: CompOp::Eq, lhs, rhs } => {
                let var_name = match lhs.as_ref() {
                    AstNode::Variable(vname) => vname.clone(),
                    _ => return Err(...)
                };
                let sub_value = eval_expr(rhs, env)?;
                target = perform_substitution(&var_name, sub_value, target, env)?;
            }
            _ => return Err(...)
        }
    }
    return Ok(target);
}
```

**How findnonhom output works with subs:**
`findnonhom` returns strings like `"X[1]^2 - X[2]"`. These are VALUE strings, not AST expressions. The user would need to: (a) parse the string as an expression, or (b) re-evaluate it with X[1] and X[2] as variables.

The typical Garvan workflow:
```maple
L := [theta3(q,50), theta2(q,50)]:
rels := findnonhom(L, q, 2, 0):
# rels is a list of strings like "X[1]^4 - X[2]^4 - ..."
# User then assigns X[1] := theta3(q, 200) etc and evaluates
```

In q-Kangaroo, since `findnonhom` returns `Value::String`, the user would need to:
1. Assign `X[1] := theta3(q, 200)` (needs subscript parsing)
2. The string from `findnonhom` would need to be parsed/evaluated

**Key insight:** The `subs` approach may not be the primary workflow. The more important thing is that `X[1]` is a valid variable name that can be assigned to and used in expressions. With the parser change above (making `X[1]` parse as `Variable("X[1]")`), users can:
```
X[1] := theta3(q, 200);
X[2] := theta2(q, 200);
# Then use X[1], X[2] in expressions
```
And `subs(X[1]=q, X[2]=q^2, expr)` would substitute these in any expression where `X[1]` and `X[2]` appear as variables.

### Pattern 4: radsimp (FUNC-02)

**What:** `radsimp(expr)` simplifies rational expressions involving series quotients to clean series.

**Context:** In Maple, `radsimp` is a general radical simplifier. In q-Kangaroo, FUNC-02 specifies it for series quotient simplification. The success criteria `radsimp(theta3(q,100)/theta3(q^5,20))` implies it can handle expressions that might not evaluate cleanly with the normal evaluator.

**Key problem:** `theta3(q^5,20)` does not currently work because `theta3` in 2-arg form expects `(Symbol, Integer)`, not `(Series, Integer)`. The expression `theta3(q,100)/theta3(q^5,20)` would error at `theta3(q^5,20)` before `radsimp` ever sees it.

**Recommended approach:** `radsimp` should be implemented as a thin wrapper, BUT the underlying issue is that theta functions (and other series generators) need to accept monomial arguments. The cleanest path is:

Option A (preferred): Enhance theta2/3/4 to accept Series (monomial) arguments. When `theta3(q^5, 20)` is called with a monomial `q^5` as first arg, compute `theta3(q, 100)` (adjusting truncation: `T * k` where `k` is the monomial exponent) and then scale exponents by `k`. Then `radsimp(expr)` is just `expr` passed through -- the division already works. `radsimp` simply returns its argument (identity function for already-evaluated series).

Option B: `radsimp` intercepts at AST level (like subs), detects quotient patterns, and computes them with adjusted truncation. This is complex and fragile.

**Option A implementation for theta functions:**
```rust
"theta3" => {
    if args.len() == 2 {
        match &args[0] {
            Value::Symbol(s) => {
                // Existing: theta3(q, T)
                let sym = env.symbols.intern(s);
                let order = extract_i64(name, args, 1)?;
                Ok(Value::Series(qseries::theta3(sym, order)))
            }
            Value::Series(mono) => {
                // New: theta3(q^k, T) -- compute theta3(q, T*k) then scale
                // Detect q^k pattern
                let terms: Vec<_> = mono.iter().collect();
                if terms.len() == 1 {
                    let (&exp, coeff) = terms[0];
                    if *coeff == QRat::one() && exp > 0 {
                        let order = extract_i64(name, args, 1)?;
                        let adjusted_order = order * exp;
                        let full = qseries::theta3(mono.variable(), adjusted_order);
                        // Scale exponents: keep only terms where exponent is divisible by exp
                        let mut new_coeffs = BTreeMap::new();
                        for (&e, c) in full.iter() {
                            if e % exp == 0 {
                                new_coeffs.insert(e, c.clone());
                            }
                        }
                        // The result is theta3(q^k, T) = theta3 evaluated at q^k, up to O(q^(T*k))
                        return Ok(Value::Series(FormalPowerSeries::from_coeffs(
                            mono.variable(), new_coeffs, order * exp
                        )));
                    }
                }
                Err(EvalError::Other("theta3: first argument must be a variable or q^k monomial".into()))
            }
            _ => {
                let sym = extract_symbol_id(name, args, 0, env)?;
                let order = extract_i64(name, args, 1)?;
                Ok(Value::Series(qseries::theta3(sym, order)))
            }
        }
    }
    // ... other arities ...
}
```

**Then `radsimp` is simply:**
```rust
"radsimp" => {
    expect_args(name, args, 1)?;
    // The argument has already been evaluated (including any division)
    // Just return it -- the "simplification" happened during evaluation
    Ok(args[0].clone())
}
```

**Note:** This approach means the real work for FUNC-02 is enhancing theta/etaq/aqprod functions to handle monomial arguments (q^k), making `radsimp` essentially an identity function for q-Kangaroo. This aligns with Garvan's Maple where `radsimp` is a general utility, not a q-series-specific function.

**Alternative interpretation:** `radsimp` may be needed when the user has already computed series separately and wants to divide them with proper truncation handling. In this case, `radsimp` could accept 1 arg (already-computed quotient) and just return it, since series division already works via `eval_div`.

### Anti-Patterns to Avoid

- **Changing JAC semantics globally:** Do NOT change the meaning of `JAC(a,b)` in the constructor or in `jacobi_product_to_fps`. This would break `qs2jaccombo`, `series(JP, q, T)`, and other existing code. Instead, add a separate Garvan-convention expansion function for the 2-arg `jac2series` path only.

- **Making radsimp AST-level:** Do NOT intercept radsimp at the AST level unless absolutely necessary. It should be a regular dispatch function. The real fix is making theta/etaq handle monomial arguments.

- **Hardcoding X[1] in the parser:** Do NOT add special-case handling for `X[...]`. Instead, add general subscript syntax `var[expr]` to the parser, creating a `Variable("var[N]")` AST node. This is more general and handles any indexed variable.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Jacobi triple product | Custom triple product code | `qseries::jacprod(a, b, sym, order)` | Already implements `(q^a;q^b)(q^{b-a};q^b)(q^b;q^b)` correctly |
| Series inversion | Custom polynomial division | `arithmetic::invert(&fps)` | Handles Laurent series, sparse coefficients, proper truncation |
| Series substitution q->q^k | Custom coefficient remapping | Extend `perform_substitution` or direct exponent scaling | Already handles the q^k pattern for subs |
| Quintuple product identity | Manual product computation | `qseries::quinprod(&monomial, sym, order)` for numeric; string formatting for identity modes | Core quinprod is correct; identity modes are display-only |

## Common Pitfalls

### Pitfall 1: JAC(0,b) Zero Bug
**What goes wrong:** `JAC(0,b)` currently creates `JacobiProduct(vec![(0, b, 1)])`. When expanded via `jacobi_product_to_fps`, this calls `etaq(0, b, ...)` which returns zero (since `(1-q^0) = 0`).
**Why it happens:** The current code treats `(a, b)` as `(q^a; q^b)_inf` directly. Garvan's `JAC(0,b)` means `(q^b;q^b)_inf`.
**How to avoid:** In the Garvan-convention expansion function, special-case `a == 0` to call `etaq(b, b, ...)` instead.
**Warning signs:** Any test with `JAC(0, ...)` returning zero or empty series.

### Pitfall 2: Breaking Existing JAC Tests
**What goes wrong:** Changing `jacobi_product_to_fps` to use Garvan triple-product convention would break all existing tests that expect `JAC(1,5)` = `etaq(1, 5)`.
**Why it happens:** Garvan's `JAC(1,5)` is the triple product `(q;q^5)(q^4;q^5)(q^5;q^5)`, not just `(q;q^5)_inf`.
**How to avoid:** Keep existing `jacobi_product_to_fps` unchanged. Add a NEW function `jacobi_product_to_fps_garvan` for the 2-arg `jac2series` path only.
**Warning signs:** Existing integration tests `jac2series_single_factor` and `jac2series_matches_etaq` failing.

### Pitfall 3: X[1] Parse Failure
**What goes wrong:** Typing `X[1]` in the REPL causes a parse error because `[` is only recognized as list literal start, not as subscript.
**Why it happens:** The parser's LED loop does not handle `LBracket` as a postfix operator after a variable.
**How to avoid:** Add subscript parsing in the LED loop before the infix operator check.
**Warning signs:** Any expression containing `X[1]` failing to parse.

### Pitfall 4: subs Multi-arg Count Off-by-One
**What goes wrong:** When extending subs from 2-arg to variadic, the AST interception at line 1089 hardcodes `args.len() != 2`. If this is changed to `args.len() < 2`, but the loop doesn't correctly identify the last arg as the target, substitutions break.
**Why it happens:** The args list contains both substitution pairs AND the target expression, all as AstNode values.
**How to avoid:** Always use `args.len() - 1` as the index for the target expression, and iterate `0..args.len()-1` for substitution pairs.
**Warning signs:** `subs(X[1]=q, expr)` working but `subs(X[1]=q, X[2]=q^2, expr)` failing.

### Pitfall 5: theta3(q^5, 20) Truncation
**What goes wrong:** When computing `theta3(q^5, 20)`, if the truncation is computed as `O(q^20)` instead of `O(q^100)`, the result has far too few terms.
**Why it happens:** `theta3(q, T)` produces `O(q^T)`. With `q^5` substitution, we need `O(q^{5T})`.
**How to avoid:** When detecting a monomial argument `q^k`, multiply the truncation order by `k`.
**Warning signs:** `theta3(q^5,20)` producing only 4 terms instead of 20.

### Pitfall 6: perform_substitution Only Works on Series
**What goes wrong:** `subs(X[1]=q, findnonhom_result)` fails because `findnonhom` returns `Value::String`, and `perform_substitution` returns non-Series targets unchanged.
**Why it happens:** `perform_substitution` only works on `Value::Series`. String results from `findnonhom` are opaque.
**How to avoid:** Document that the `subs` workflow with `findnonhom` output requires the user to: (1) parse/evaluate the string expression by assigning `X[1]`, `X[2]` as variables first, then evaluating the expression directly. The `subs` approach works when the expression is already computed as a Value, not when it's a string.
**Warning signs:** User confusion about how to use `findnonhom` output with `subs`.

## Code Examples

### jac2series 2-arg Garvan Form
```rust
// Source: eval.rs dispatch, new 2-arg path
"jac2series" => {
    if args.len() == 2 {
        let factors = match &args[0] {
            Value::JacobiProduct(f) => f.clone(),
            _ => return Err(EvalError::Other(
                "expected Jacobi product expression (use JAC(a,b))".to_string()
            )),
        };
        let order = extract_i64(name, args, 1)?;
        let fps = jacobi_product_to_fps_garvan(&factors, env.sym_q, order);
        let formatted = crate::format::format_value(
            &Value::Series(fps.clone()), &env.symbols
        );
        println!("{}", formatted);
        Ok(Value::Series(fps))
    } else if args.len() == 3 {
        // Legacy 3-arg path unchanged
        ...
    }
}
```

### quinprod prodid/seriesid Display
```rust
// Source: eval.rs dispatch, new identity mode branch
fn format_quinprod_prodid(z: &str, q: &str) -> String {
    // Quintuple product identity in product form:
    // (-z,q)_inf * (-q/z,q)_inf * (z^2*q,q^2)_inf * (q^2/z^2,q^2)_inf * (q,q)_inf
    format!(
        "(-{z},{q})_inf * (-{q}/{z},{q})_inf * ({z}^2*{q},{q}^2)_inf * ({q}^2/{z}^2,{q}^2)_inf * ({q},{q})_inf",
        z=z, q=q
    )
}

fn format_quinprod_seriesid(z: &str, q: &str) -> String {
    // Same product side, equals the series:
    // sum_{m=-inf}^{inf} (z^(3m) - z^(-3m-1)) * q^(m*(3m+1)/2)
    let prod_side = format_quinprod_prodid(z, q);
    format!(
        "{}\n  = sum(m=-inf..inf, ({z}^(3*m) - {z}^(-3*m-1)) * {q}^(m*(3*m+1)/2))",
        prod_side, z=z, q=q
    )
}
```

### Parser Subscript Extension
```rust
// Source: parser.rs, LED loop addition (before infix_bp check)
// Subscript: X[i]
if *self.peek() == Token::LBracket {
    if 19 < min_bp { break; }
    if let AstNode::Variable(ref name) = lhs {
        let saved_name = name.clone();
        self.advance(); // consume [
        let index = self.expr_bp(0)?;
        self.expect(&Token::RBracket, "']' to close subscript")?;
        if let AstNode::Integer(i) = &index {
            lhs = AstNode::Variable(format!("{}[{}]", saved_name, i));
            continue;
        }
        // Non-integer index: error
        return Err(ParseError::new(
            "subscript index must be an integer".to_string(),
            self.peek_span(),
        ));
    }
    break; // Not a variable, don't subscript
}
```

### subs Multi-substitution
```rust
// Source: eval.rs, AST interception for subs
if name == "subs" {
    if args.len() < 2 {
        return Err(EvalError::WrongArgCount {
            function: "subs".to_string(),
            expected: "at least 2".to_string(),
            got: args.len(),
            signature: "subs(var=val, ..., expr)".to_string(),
        });
    }
    // Evaluate target (last arg)
    let mut target = eval_expr(&args[args.len() - 1], env)?;
    // Process substitution pairs (all args except the last)
    for i in 0..(args.len() - 1) {
        match &args[i] {
            AstNode::Compare { op: CompOp::Eq, lhs, rhs } => {
                let var_name = match lhs.as_ref() {
                    AstNode::Variable(vname) => vname.clone(),
                    _ => return Err(EvalError::Other(
                        "subs: left side of = must be a variable name".into()
                    )),
                };
                let sub_value = eval_expr(rhs, env)?;
                target = perform_substitution(&var_name, sub_value, target, env)?;
            }
            _ => return Err(EvalError::Other(
                "subs: each substitution must be var=value".into()
            )),
        }
    }
    return Ok(target);
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `jac2series(JP, q, T)` 3-arg | `jac2series(jacexpr, T)` 2-arg (Garvan) | Phase 50 | Matches Garvan qseries 1.3 signature |
| `JAC(0,b)` = zero | `JAC(0,b)` = `(q^b;q^b)_inf` in Garvan context | Phase 50 | Fixes mathematical incorrectness |
| `quinprod(z,q,T)` numeric only | `quinprod(z,q,prodid/seriesid)` identity modes | Phase 50 | Matches Garvan display modes |
| `subs(var=val, expr)` single pair | `subs(v1=e1, v2=e2, ..., expr)` multi-sub | Phase 50 | Supports findnonhom workflow |
| No `X[1]` syntax | `X[1]` parsed as `Variable("X[1]")` | Phase 50 | Enables indexed variables |
| No `radsimp` | `radsimp(expr)` for series quotients | Phase 50 | Matches Garvan tutorial examples |

## Open Questions

1. **JAC(a,b) convention for a >= b or a < 0 in Garvan mode**
   - What we know: Garvan's qseries 1.3 added handling for `a < 0` or `a > b`. The standard definition is `JAC(a,b)` for `0 < a < b`.
   - What's unclear: Exact mathematical formula for extended cases. `JAC(a,b)` when `a >= b` -- is it `JAC(a mod b, b)` or something else?
   - Recommendation: Implement `a = 0` and `0 < a < b` first. For `a < 0` or `a >= b`, reduce via `a mod b` with appropriate sign handling. Test against known identities.

2. **radsimp semantics: identity function or active simplification?**
   - What we know: If theta functions are enhanced to handle monomial args (q^k), then `radsimp(theta3(q,100)/theta3(q^5,20))` works because the division is computed during evaluation, and `radsimp` just returns the result.
   - What's unclear: Should `radsimp` do any additional simplification (e.g., removing trailing zeros, normalizing truncation orders)?
   - Recommendation: Start with `radsimp` as an identity function for Series values. If needed, add normalization (strip trailing zeros, adjust truncation order to match content). The real work is in making theta3(q^5, T) work.

3. **perform_substitution for indexed variables**
   - What we know: `perform_substitution("X[1]", value, target, env)` currently only works on Series targets where the variable name matches. `X[1]` is not a series variable -- it's an expression variable.
   - What's unclear: How should substitution work for string expressions from findnonhom?
   - Recommendation: The subs approach for `X[i]` variables works on re-evaluated expressions, not on string values. Users assign `X[1] := series_value` then evaluate expressions using `X[1]`. The `subs` function substitutes in Series targets by matching variable names. For string results from findnonhom, the user must re-evaluate the string expression (which is a separate concern -- possibly a `parse` or `eval` built-in, out of scope for Phase 50).

4. **Exact display format for quinprod prodid/seriesid**
   - What we know: Garvan's output uses q-Pochhammer notation with subscript infinity symbols
   - What's unclear: Exact ASCII rendering. Should we use `(z;q)_inf` notation or `prod_n(1-z*q^n)` notation?
   - Recommendation: Use q-Pochhammer notation: `(-z,q)_inf * (-q/z,q)_inf * ...` for plain text. Add LaTeX mode later.

## Sources

### Primary (HIGH confidence)
- [Garvan qseries functions reference](https://qseries.org/fgarvan/qmaple/qseries/functions) - Function list and individual doc pages
- [quinprod.html](https://qseries.org/fgarvan/qmaple/functions/quinprod.html) - quinprod prodid/seriesid calling conventions
- [jac2series.html](https://qseries.org/fgarvan/qmaple/qseries/functions/jac2series.html) - jac2series 2-arg signature, JAC(0,b) handling
- Codebase: `eval.rs` lines 4650-4665 (current jac2series), 3272-3307 (current quinprod), 1086-1114 (current subs)
- Codebase: `parser.rs` lines 316-395 (LED loop, no subscript handling)
- Codebase: `products.rs` lines 74-94 (jacprod triple product), 33-72 (etaq)

### Secondary (MEDIUM confidence)
- [qmaple.pdf](https://qseries.org/fgarvan/papers/qmaple.pdf) - Garvan's original tutorial paper
- [Updated tutorial](https://qseries.org/fgarvan/qmaple/qseries/doc/qseriesdoc.pdf) - qseries 1.3 tutorial with updated examples
- [Maple radsimp docs](https://www.maplesoft.com/support/help/Maple/view.aspx?path=radsimp(deprecated)) - Maple's radsimp is deprecated, about radical simplification

### Tertiary (LOW confidence)
- Garvan's convention for `JAC(a,b)` when `a >= b` or `a < 0` -- only mentioned as "handled" in 1.3, no formula found

## Metadata

**Confidence breakdown:**
- jac2series overhaul: HIGH - clear 2-arg signature from docs, edge cases well understood
- quinprod identity modes: HIGH - prodid/seriesid documented, mathematical identity well known
- radsimp: MEDIUM - semantics uncertain (identity function vs active simplification), depends on theta monomial support
- indexed subs: HIGH - parser change straightforward, multi-sub pattern clear from AST structure

**Research date:** 2026-02-21
**Valid until:** 2026-03-21 (stable domain, no external dependencies)
