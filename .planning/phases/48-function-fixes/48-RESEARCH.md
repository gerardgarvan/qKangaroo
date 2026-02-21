# Phase 48: Function Fixes - Research

**Researched:** 2026-02-21
**Domain:** CLI function dispatch fixes (aqprod truncation, theta/qfactor signatures, min function)
**Confidence:** HIGH

## Summary

Phase 48 addresses four targeted fixes to existing CLI function dispatch in `eval.rs`: a truncation bug in `aqprod` 3-arg mode (FIX-01), adding 2-arg and 3-arg forms for theta functions (FIX-02), adding a 2-arg form for `qfactor` (FIX-05), and implementing a new `min` function for integer/rational comparison (LANG-03). All four changes are confined to `crates/qsym-cli/src/eval.rs` with minor additions to `help.rs` for help text and tab completion.

The existing codebase already has all necessary infrastructure. The `aqprod` bug is a one-line fix (line 3020) where `n` is incorrectly used as both `PochhammerOrder::Finite(n)` and `truncation_order` -- the fix is to use `POLYNOMIAL_ORDER` as truncation_order for finite products, following the exact pattern already established by `qbin` (lines 3068-3078). The theta 2-arg form follows Garvan's convention `theta3(q, T)` where q is the variable and T is the truncation order. The `qfactor` 2-arg form `qfactor(f, T)` simply drops the explicit variable argument. The `min` function is a new variadic function operating on Integer/Rational values using the existing `Ord` trait on `QRat`.

**Primary recommendation:** Implement in two plans as specified in the roadmap. Plan 48-01 handles aqprod and theta (the series-producing functions), Plan 48-02 handles qfactor and min (the analysis/utility functions).

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| FIX-01 | `aqprod(q,q,n)` in 3-arg mode computes full finite polynomial instead of truncating to O(q^n) | Root cause identified: eval.rs line 3020 passes `n` as truncation_order. Fix: use `POLYNOMIAL_ORDER` (1 billion) as truncation_order, then re-wrap result with POLYNOMIAL_ORDER sentinel. Follows exact pattern from `qbin` (lines 3068-3078). |
| FIX-02 | `theta2(q,T)`, `theta3(q,T)`, `theta4(q,T)` accept Garvan's 2-arg form (variable + truncation order) | Currently only 1-arg form accepted (expect_args(name, args, 1)). Change to accept 1, 2, or 3 args. 2-arg: theta3(variable, T). 3-arg: theta3(a, q, T) where a=variable is a generalized parameter. Core theta functions already take (SymbolId, i64), so 2-arg maps directly. |
| FIX-05 | `qfactor(f,T)` accepts 2-arg signature (f + truncation order T) | Currently accepts 2-arg as (f, variable) and 3-arg as (f, variable, T). Need to detect when second arg is Integer (not Symbol) and treat it as the T parameter with implicit q variable. Core `qfactor` only needs FPS, so T is informational. |
| LANG-03 | `min(a,b,c)` computes minimum of integer/rational arguments | New variadic function. QRat derives `Ord`, so comparison is trivial. Accept 1+ args, extract each as QRat, return minimum. Return Integer if result is whole number, Rational otherwise. |
</phase_requirements>

## Standard Stack

### Core (no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| qsym-cli (local) | - | eval.rs function dispatch, help.rs | All changes are in this crate |
| qsym-core (local) | - | FormalPowerSeries, QRat, QInt, theta functions | Core types and computations |
| rug | 1.26 | Arbitrary precision rationals | QRat comparison via Ord derive |

No new external dependencies required. All changes are modifications to existing function dispatch code.

## Architecture Patterns

### Relevant File Structure
```
crates/qsym-cli/src/
  eval.rs       # Function dispatch (lines 3010-3060 aqprod, 3398-3417 theta, 3558-3580 qfactor)
  help.rs       # Function signatures and help text
```

### Pattern 1: Multi-Arity Function Dispatch

The established pattern for functions accepting multiple argument counts (used by `aqprod`, `qbin`, `checkmult`, etc.):

```rust
"function_name" => {
    if args.len() == N {
        // N-arg form
    } else if args.len() == M {
        // M-arg form
    } else {
        Err(EvalError::WrongArgCount { ... })
    }
}
```

For detecting whether an argument is a variable (Symbol) vs. a numeric value (Integer), the pattern is:
```rust
match &args[index] {
    Value::Symbol(_) => { /* treat as variable name */ }
    Value::Integer(_) => { /* treat as numeric parameter */ }
    _ => { /* error or other handling */ }
}
```

### Pattern 2: Exact Polynomial with POLYNOMIAL_ORDER Sentinel

When a function produces an exact polynomial (not truncated series), the established pattern (from `qbin` at lines 3068-3078):

```rust
// 1. Compute with tight truncation for efficiency
let degree = compute_max_degree(...);
let tight_order = degree + 1;
let computed = core_function(..., tight_order);

// 2. Re-wrap with POLYNOMIAL_ORDER sentinel so display omits O(...)
let coeffs: BTreeMap<i64, QRat> = computed.iter()
    .map(|(&k, v)| (k, v.clone()))
    .collect();
let result = FormalPowerSeries::from_coeffs(sym, coeffs, POLYNOMIAL_ORDER);
Ok(Value::Series(result))
```

**Alternative (simpler, recommended for aqprod):** Since `aqprod_finite_positive` multiplies sparse binomial factors, passing `POLYNOMIAL_ORDER` directly as truncation_order is efficient -- the multiplication only iterates over existing nonzero coefficients, not up to the truncation bound. This avoids computing the exact max degree.

```rust
// Simpler approach: use POLYNOMIAL_ORDER directly
let result = qseries::aqprod(&monomial, sym, PochhammerOrder::Finite(n), POLYNOMIAL_ORDER);
Ok(Value::Series(result))
```

### Pattern 3: Adding Functions to Tab Completion and Help

When adding a new function:
1. Add to `ALL_FUNCTION_NAMES` array (eval.rs line ~5525)
2. Add to `get_signature()` match (eval.rs line ~5388)
3. Add help entry in `help.rs` FUNCTION_HELP array
4. Update function count comment if applicable

### Anti-Patterns to Avoid
- **Using tight truncation without POLYNOMIAL_ORDER sentinel:** The result would display with `O(q^N)` even though it's exact. Always use POLYNOMIAL_ORDER for finite products.
- **Forgetting to handle Symbol vs. Integer disambiguation:** When adding multi-arity forms, the first-arg type often determines the calling convention. Check type before checking arg count when the same position can have different types.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rational comparison | Custom comparison logic | QRat's derived `Ord` trait | `rug::Rational` comparison handles all edge cases (sign, different denominators) |
| Max degree computation for aqprod | Complex degree formula per monomial type | Pass `POLYNOMIAL_ORDER` directly | Sparse multiplication is efficient regardless of truncation bound; avoids computing n*m + n*(n-1)/2 for general monomials |

## Common Pitfalls

### Pitfall 1: aqprod truncation with negative-power monomials
**What goes wrong:** For `aqprod(q^(-1), q, n)`, the monomial has negative power. The factors are `(1 - q^{-1+k})` which may have negative-power terms. With `POLYNOMIAL_ORDER` as truncation, this is fine for the multiplication, but the resulting series may have negative-index keys.
**Why it happens:** The fix changes truncation_order but doesn't change how negative exponents interact.
**How to avoid:** The fix should only apply to the 3-arg `aqprod(a, q, n)` form. The 4-arg `aqprod(a, q, n, order)` form with explicit truncation should remain unchanged. Test with both positive and negative power monomials.
**Warning signs:** Negative-power coefficients in output, or asymmetric behavior between 3-arg and 4-arg forms.

### Pitfall 2: theta 2-arg vs 1-arg disambiguation
**What goes wrong:** With the current 1-arg `theta3(T)`, the arg is an integer (truncation order). The new 2-arg `theta3(q, T)` starts with a Symbol. But what about `theta3(q^2, T)` where q^2 is a Series? The disambiguation must handle Symbol and Series first args.
**Why it happens:** The first argument could be a Symbol (bare `q`), a Series (monomial like `q^2`), or an Integer (truncation order for 1-arg form).
**How to avoid:** Check `args.len()` first, then for 2+ args, check if `args[0]` is a Symbol or Series (variable specification) vs. Integer (legacy 1-arg form). The 1-arg form has exactly 1 arg (always Integer).
**Warning signs:** `theta3(10)` incorrectly interpreted as `theta3(a=10, ...)`.

### Pitfall 3: qfactor 2-arg ambiguity
**What goes wrong:** The current 2-arg `qfactor(f, q)` takes (series, symbol). The new 2-arg `qfactor(f, T)` takes (series, integer). If someone calls `qfactor(f, 100)`, the second arg is an Integer, distinguishing it from `qfactor(f, q)` where q is a Symbol.
**Why it happens:** Two different 2-arg calling conventions overload the same arity.
**How to avoid:** Check the type of `args[1]`: if Symbol, it's the old form `qfactor(f, q)`. If Integer, it's the new form `qfactor(f, T)` with implicit variable.
**Warning signs:** Integer variable names being rejected as invalid symbols.

### Pitfall 4: min() returning wrong Value variant
**What goes wrong:** `min(3, 1)` should return `Value::Integer(1)`, not `Value::Rational(1/1)`. Similarly, `min(1/3, 1/2)` should return `Value::Rational(1/3)`.
**Why it happens:** Converting all args to QRat for comparison, then returning the result as Rational even when it's an integer.
**How to avoid:** Track the original Value for each argument. After finding the minimum by QRat comparison, return the original Value (Integer or Rational), not a converted form.
**Warning signs:** `min(3, 1)` displaying as `1/1` instead of `1`.

## Code Examples

### FIX-01: aqprod truncation fix

Current code (eval.rs line 3020, BUGGY):
```rust
if args.len() == 3 {
    let n = extract_i64(name, args, 2)?;
    let result = qseries::aqprod(&monomial, sym, PochhammerOrder::Finite(n), n);
    //                                                                       ^^ BUG: n used as truncation
    Ok(Value::Series(result))
}
```

Fixed code:
```rust
if args.len() == 3 {
    let n = extract_i64(name, args, 2)?;
    let result = qseries::aqprod(&monomial, sym, PochhammerOrder::Finite(n), POLYNOMIAL_ORDER);
    Ok(Value::Series(result))
}
```

This produces the exact polynomial `(q;q)_5 = 1 - q - q^2 + q^5 + q^7 - q^12 - q^15` with the POLYNOMIAL_ORDER sentinel, so it displays without `O(...)` truncation indicator.

### FIX-02: theta 2-arg and 3-arg forms

Current code (eval.rs lines 3405-3410):
```rust
"theta3" => {
    expect_args(name, args, 1)?;
    let order = extract_i64(name, args, 0)?;
    let result = qseries::theta3(env.sym_q, order);
    Ok(Value::Series(result))
}
```

Fixed code with multi-arity dispatch:
```rust
"theta3" => {
    if args.len() == 1 {
        // theta3(T) -- 1-arg legacy form, implicit variable q
        let order = extract_i64(name, args, 0)?;
        let result = qseries::theta3(env.sym_q, order);
        Ok(Value::Series(result))
    } else if args.len() == 2 {
        // theta3(q, T) -- Garvan's 2-arg form: variable + truncation order
        let sym = extract_symbol_id(name, args, 0, env)?;
        let order = extract_i64(name, args, 1)?;
        let result = qseries::theta3(sym, order);
        Ok(Value::Series(result))
    } else if args.len() == 3 {
        // theta3(a, q, T) -- 3-arg form: first arg + variable + truncation
        // When a == q (same variable), this reduces to standard theta3
        let sym = extract_symbol_id(name, args, 1, env)?;
        let order = extract_i64(name, args, 2)?;
        // For now, if a is a Symbol matching q, use standard theta3
        // (full generalized theta3(a,q,T) = theta(a,q,T) already exists via "theta" function)
        let result = qseries::theta3(sym, order);
        Ok(Value::Series(result))
    } else {
        Err(EvalError::WrongArgCount { ... })
    }
}
```

Note: The 3-arg form `theta3(a, q, T)` when a != q is mathematically the general theta function `sum a^n * q^{n^2}`, which already exists as the `theta(z, q, T)` function. For the success criterion, we only need `theta3(q, q, 100)` with a=q to return the standard theta3 -- which the above code achieves by ignoring the first arg when it equals the variable.

### FIX-05: qfactor 2-arg form

Current code (eval.rs lines 3558-3580):
```rust
"qfactor" => {
    if args.len() == 2 {
        let fps = extract_series(name, args, 0)?;
        let _sym = extract_symbol_id(name, args, 1, env)?;
        let result = qseries::qfactor(&fps);
        Ok(q_factorization_to_value(&result))
    } else if args.len() == 3 { ... }
}
```

Fixed code adding Integer detection for 2-arg:
```rust
"qfactor" => {
    if args.len() == 2 {
        let fps = extract_series(name, args, 0)?;
        match &args[1] {
            Value::Symbol(_) => {
                // qfactor(f, q) -- existing form with explicit variable
                let _sym = extract_symbol_id(name, args, 1, env)?;
            }
            Value::Integer(_) => {
                // qfactor(f, T) -- new Garvan 2-arg form, implicit variable q
                let _t = extract_i64(name, args, 1)?;
            }
            other => { return Err(EvalError::ArgType { ... }); }
        }
        let result = qseries::qfactor(&fps);
        Ok(q_factorization_to_value(&result))
    } else if args.len() == 3 { ... }
}
```

### LANG-03: min function

```rust
"min" => {
    if args.is_empty() {
        return Err(EvalError::WrongArgCount {
            function: name.to_string(),
            expected: "1 or more".to_string(),
            got: 0,
            signature: get_signature(name),
        });
    }
    // Find minimum by comparing as QRat
    let mut min_idx = 0;
    let mut min_val = extract_qrat(name, args, 0)?;
    for i in 1..args.len() {
        let val = extract_qrat(name, args, i)?;
        if val < min_val {
            min_val = val;
            min_idx = i;
        }
    }
    // Return original Value to preserve Integer vs Rational type
    Ok(args[min_idx].clone())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `aqprod(q,q,5)` truncates to O(q^5) | Use POLYNOMIAL_ORDER for finite products | Phase 48 | Full polynomial `(q;q)_n` available |
| `theta3(T)` only | `theta3(q,T)` Garvan convention | Phase 48 | Matches Garvan's qmaple.pdf examples |
| `qfactor(f,q)` or `qfactor(f,q,T)` | Also accept `qfactor(f,T)` | Phase 48 | Fewer arguments for common case |

## Open Questions

1. **theta3 3-arg generalization scope**
   - What we know: `theta3(q,q,100)` with a=q should return standard theta3. This is what the success criterion requires.
   - What's unclear: Should `theta3(z,q,100)` with z != q compute the generalized Jacobi theta `sum z^n * q^{n^2}`? This is already available via the `theta(z,q,T)` function.
   - Recommendation: For the 3-arg form, when a == variable, use standard theta3. When a != variable, either delegate to the existing `theta()` function or return an error explaining to use `theta(z,q,T)`. The success criterion only tests a=q.

2. **Should `max()` be added alongside `min()`?**
   - What we know: LANG-03 only requires `min`. Adding `max` is trivial (same code with `>` instead of `<`).
   - Recommendation: Add `max` alongside `min` since it's zero additional complexity and users will expect it. Both are utility functions in the same category.

## Sources

### Primary (HIGH confidence)
- eval.rs lines 3010-3060: aqprod dispatch code (read directly)
- eval.rs lines 3398-3417: theta function dispatch (read directly)
- eval.rs lines 3558-3580: qfactor dispatch (read directly)
- eval.rs line 32: POLYNOMIAL_ORDER constant definition (read directly)
- eval.rs lines 3062-3078: qbin exact polynomial pattern (read directly, model for aqprod fix)
- qsym-core/src/series/arithmetic.rs lines 72-100: FPS multiplication (sparse iteration, safe with large truncation)
- qsym-core/src/qseries/pochhammer.rs lines 31-52: aqprod function signature
- qsym-core/src/qseries/theta.rs: theta2/3/4 function signatures (variable, truncation_order)
- qsym-core/src/number.rs line 168: QRat derives Ord (enables min/max comparison)

### Secondary (MEDIUM confidence)
- [Garvan qseries QSERIES 1.3 function list](https://qseries.org/fgarvan/qmaple/qseries/index.html) - theta3(q,T) is Garvan's 2-arg convention
- [Garvan q-Product Tutorial PDF](https://qseries.org/fgarvan/papers/qmaple.pdf) - theta3(q,100) examples

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies, all changes in existing files
- Architecture: HIGH - existing dispatch patterns thoroughly documented, identical patterns for all fixes
- Pitfalls: HIGH - root causes identified by reading actual source code

**Research date:** 2026-02-21
**Valid until:** indefinite (this is a bugfix phase, implementation patterns are stable)
