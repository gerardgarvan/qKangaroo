# Phase 43: Expression Operations - Research

**Researched:** 2026-02-20
**Domain:** CLI evaluator -- series truncation, product expansion, runtime exponent arithmetic, floor, Legendre symbol
**Confidence:** HIGH

## Summary

Phase 43 adds five capabilities to the q-Kangaroo CLI evaluator: (1) a `series(expr, q, T)` function to re-truncate a computed series to a different order, (2) an `expand(expr)` function to force expansion of products into polynomial/series form, (3) runtime integer arithmetic in q-exponents so that `q^(k*(3*k+1)/2)` works correctly inside for-loops, (4) a `floor(x)` function for rationals, and (5) a `legendre(m, p)` function for the Legendre symbol.

All five features are purely CLI-side changes in `crates/qsym-cli/src/eval.rs` (function dispatch, `eval_pow` enhancement) plus help entries. The core library (`qsym-core`) already has all necessary infrastructure: `FormalPowerSeries` with truncation semantics, `rug::Rational::floor_ref()`, and `rug::Integer::legendre()`. No new core library code is needed.

**Primary recommendation:** Implement all five features as additions to the existing `dispatch()` function and `eval_pow()` match arms in `eval.rs`, plus help/signature entries. The runtime exponent fix is a 5-line change to `eval_pow`; the functions are 10-30 lines each.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SERIES-01 | `series(expr, q, T)` truncates to O(q^T) | FPS already has truncation_order field; create new FPS copying only coefficients < T. Dispatch as 3-arg function. |
| SERIES-02 | `expand(expr)` expands products into polynomial form | Eager evaluation already expands Series*Series. Main use: convert JacobiProduct to series via existing `jacobi_product_to_fps()`. Also useful as identity/type-check for series. |
| SERIES-03 | Runtime integer arithmetic in q-exponents: `q^(n*n)`, `q^(k*(3*k+1)/2)` | `eval_pow` currently handles `Symbol ^ Integer` but NOT `Symbol ^ Rational`. Division in evaluator produces Rational even for exact integer results. Fix: add `Symbol ^ Rational` arm that checks denom==1 and extracts integer exponent. Same for `Series ^ Rational`. |
| UTIL-01 | `floor(x)` for rational numbers | `rug::Rational::floor_ref()` + `rug::Integer::assign()` provides exact floor. Dispatch as 1-arg function accepting Integer (identity) or Rational. |
| UTIL-02 | `legendre(m, p)` for the Legendre symbol | `rug::Integer::legendre(&p)` is a GMP built-in returning i32 (-1, 0, 1). Dispatch as 2-arg function extracting two integers. Also register `L` as an alias. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rug | 1.28 | `Rational::floor_ref()`, `Integer::legendre()` | Already a dependency; GMP provides O(1) floor and optimized Legendre via quadratic reciprocity |
| qsym-core FormalPowerSeries | n/a | Sparse BTreeMap series with truncation_order | All series operations already built |

### Supporting
No new dependencies needed. All required functionality exists in rug 1.28 and the existing codebase.

## Architecture Patterns

### Pattern 1: Function Dispatch (existing pattern)
**What:** Add new function names to the `dispatch()` match in `eval.rs`
**When to use:** For `series`, `expand`, `floor`, `legendre`
**Example:**
```rust
// In dispatch() match:
"series" => {
    expect_args(name, args, 3)?;
    let fps = extract_series(name, args, 0)?;
    let _sym = extract_symbol_id(name, args, 1, env)?;
    let order = extract_i64(name, args, 2)?;
    // Re-truncate: keep only coefficients below new order
    let mut new_coeffs = BTreeMap::new();
    for (&k, v) in fps.iter() {
        if k < order {
            new_coeffs.insert(k, v.clone());
        }
    }
    Ok(Value::Series(FormalPowerSeries::from_coeffs(
        fps.variable(), new_coeffs, order
    )))
}
```

### Pattern 2: eval_pow Extension (existing pattern)
**What:** Add match arms to `eval_pow()` for `Symbol ^ Rational` and `Series ^ Rational`
**When to use:** For SERIES-03 (runtime exponent arithmetic)
**Example:**
```rust
// In eval_pow() match, BEFORE the catch-all:
(Value::Symbol(name), Value::Rational(r)) => {
    // Check if rational is actually an integer (denom == 1)
    if r.denom() == &rug::Integer::from(1) {
        let exp = r.numer().to_i64().ok_or_else(|| EvalError::Other(
            "exponent too large".to_string(),
        ))?;
        let sym_id = env.symbols.intern(name);
        Ok(Value::Series(FormalPowerSeries::monomial(
            sym_id, QRat::one(), exp, POLYNOMIAL_ORDER
        )))
    } else {
        Err(EvalError::Other(format!(
            "q-exponent must be an integer, got {}/{}",
            r.numer(), r.denom()
        )))
    }
}
```

### Pattern 3: Registration Checklist
**What:** Every new function needs entries in 4 places
**Where:**
1. `dispatch()` match arm -- the implementation
2. `get_signature()` match arm -- error message signature
3. `ALL_FUNCTION_NAMES` array -- for fuzzy matching
4. `general_help()` text -- category listing
5. `FUNC_HELP` array -- per-function help entry
6. (optional) `resolve_alias()` -- if aliases are needed (e.g., `L` -> `legendre`)

### Anti-Patterns to Avoid
- **Adding to qsym-core for CLI-only features:** `floor()` and `legendre()` are evaluator functions, not core math operations. Keep them in `eval.rs`.
- **Forgetting POLYNOMIAL_ORDER sentinel:** When `series()` re-truncates, the new truncation order should be the explicit `T` argument, NOT `POLYNOMIAL_ORDER`. Only exact polynomials (like from `qbin`) use the sentinel.
- **Not handling JacobiProduct in expand():** The primary use case for `expand()` is converting `JacobiProduct` values to series. Just returning a `Series` unchanged is correct but `JacobiProduct` must be handled.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Floor of rational | Custom integer division with sign handling | `rug::Rational::floor_ref()` + `Integer::assign()` | GMP handles all edge cases (negative numbers, large values) correctly |
| Legendre symbol | Euler's criterion or quadratic reciprocity | `rug::Integer::legendre(&p)` | GMP uses optimized algorithm; handles edge cases (p=2, m=0) |
| Series truncation | Manual coefficient filtering | `FormalPowerSeries::from_coeffs()` with filtered BTreeMap | Already strips zeros and enforces invariants |
| JacobiProduct to series | New conversion code | Existing `jacobi_product_to_fps()` helper | Already implemented for `jac2series` function |

**Key insight:** Every mathematical operation needed (floor, legendre, series truncation, product expansion) already exists either in rug or in the existing codebase. This phase is purely about wiring up dispatch entries and fixing one type-matching gap in `eval_pow`.

## Common Pitfalls

### Pitfall 1: Integer Division Producing Rational
**What goes wrong:** `k*(3*k+1)/2` evaluates to `Value::Rational` even when the result is an integer (e.g., k=1 gives 2/1). Then `q ^ Rational` fails with a type error.
**Why it happens:** The evaluator's `eval_div(Integer, Integer)` always returns `Rational`. This is by design for general arithmetic but creates a gap in exponentiation.
**How to avoid:** Add `Symbol ^ Rational` and `Series ^ Rational` arms to `eval_pow` that check `denom() == 1` and extract the integer exponent. Also add `JacobiProduct ^ Rational` for completeness.
**Warning signs:** Tests like `q^(4/2)` or `q^(6/3)` failing.

### Pitfall 2: series() With Higher Order Than Original
**What goes wrong:** User calls `series(f, q, 100)` on a series computed to O(q^20). The result should still be O(q^20), not O(q^100), because we don't have information beyond the original truncation.
**Why it happens:** Naive implementation just sets truncation_order = T without checking.
**How to avoid:** Use `min(T, original_truncation_order)` as the new truncation order.
**Warning signs:** Coefficients appearing as zero when they're actually unknown.

### Pitfall 3: expand() Not Handling All Input Types
**What goes wrong:** User passes an Integer or Rational to `expand()` and gets a type error.
**Why it happens:** Only matching on `Value::Series` and `Value::JacobiProduct`.
**How to avoid:** `expand()` should be permissive: Series returns as-is, JacobiProduct converts via `jacobi_product_to_fps()`, Integer/Rational return as-is (or wrap as constant series), Symbol returns as-is.
**Warning signs:** `expand(3)` erroring instead of returning 3.

### Pitfall 4: Legendre Symbol p Must Be Odd Prime
**What goes wrong:** User calls `legendre(a, 4)` and gets wrong result.
**Why it happens:** Legendre symbol is only defined for odd prime moduli. GMP's `legendre()` assumes p is an odd prime -- behavior is undefined otherwise.
**How to avoid:** Validate that p is odd and > 2. Alternatively, use GMP's `jacobi()` which is more general (works for any odd positive integer). Document the requirement.
**Warning signs:** `legendre(1, 2)` giving unexpected results.

### Pitfall 5: Forgetting to Update ALL_FUNCTION_NAMES Count
**What goes wrong:** The existing test `fn all_function_names_count()` checks `ALL_FUNCTION_NAMES.len() >= 78`. After adding new functions, this assertion should still pass (it will, since we're adding). But the test is a lower bound check.
**Why it happens:** The test is a sanity check. No change needed, but be aware it exists.
**How to avoid:** After adding functions to `ALL_FUNCTION_NAMES`, verify the test still passes.

## Code Examples

### series(expr, q, T) Implementation
```rust
// Source: derived from FormalPowerSeries::from_coeffs() API
"series" => {
    expect_args(name, args, 3)?;
    let fps = extract_series(name, args, 0)?;
    let _sym = extract_symbol_id(name, args, 1, env)?;
    let order = extract_i64(name, args, 2)?;
    // Use min to prevent "upward truncation" beyond known precision
    let effective_order = order.min(fps.truncation_order());
    let new_coeffs: BTreeMap<i64, QRat> = fps.iter()
        .filter(|(&k, _)| k < effective_order)
        .map(|(&k, v)| (k, v.clone()))
        .collect();
    Ok(Value::Series(FormalPowerSeries::from_coeffs(
        fps.variable(), new_coeffs, effective_order
    )))
}
```

### expand(expr) Implementation
```rust
// Source: reuses existing jacobi_product_to_fps() from eval.rs
"expand" => {
    expect_args(name, args, 1)?;
    match &args[0] {
        Value::Series(_) => Ok(args[0].clone()),  // already expanded
        Value::JacobiProduct(factors) => {
            // Need truncation order -- use default_order from env
            let fps = jacobi_product_to_fps(factors, env.sym_q, env.default_order);
            Ok(Value::Series(fps))
        }
        Value::Integer(_) | Value::Rational(_) => Ok(args[0].clone()),
        other => Err(EvalError::Other(format!(
            "expand: cannot expand {}", other.type_name()
        ))),
    }
}
```

Note: `expand()` for JacobiProduct needs a truncation order. Since there's no explicit order argument, use `env.default_order`. Alternatively, accept an optional second argument: `expand(expr)` or `expand(expr, T)`.

### floor(x) Implementation
```rust
// Source: rug::Rational::floor_ref() + rug::Integer::assign()
"floor" => {
    expect_args(name, args, 1)?;
    match &args[0] {
        Value::Integer(_) => Ok(args[0].clone()),
        Value::Rational(r) => {
            let mut result = rug::Integer::new();
            result.assign(r.0.floor_ref());
            Ok(Value::Integer(QInt(result)))
        }
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: 0,
            expected: "number (integer or rational)",
            got: other.type_name().to_string(),
        }),
    }
}
```

### legendre(m, p) Implementation
```rust
// Source: rug::Integer::legendre(&p) returns i32 in {-1, 0, 1}
"legendre" => {
    expect_args(name, args, 2)?;
    let m = extract_i64(name, args, 0)?;
    let p = extract_i64(name, args, 1)?;
    if p < 2 {
        return Err(EvalError::Other(
            "legendre: second argument must be an odd prime >= 3".to_string()
        ));
    }
    let m_int = rug::Integer::from(m);
    let p_int = rug::Integer::from(p);
    let result = m_int.legendre(&p_int);
    Ok(Value::Integer(QInt::from(result as i64)))
}
```

### eval_pow Fix for Symbol ^ Rational
```rust
// Add BEFORE the catch-all `_ => Err(...)` in eval_pow:
(Value::Symbol(name), Value::Rational(r)) => {
    if r.denom() == &rug::Integer::from(1) {
        let exp = r.numer().to_i64().ok_or_else(|| EvalError::Other(
            "exponent too large".to_string(),
        ))?;
        let sym_id = env.symbols.intern(name);
        let fps = FormalPowerSeries::monomial(sym_id, QRat::one(), exp, POLYNOMIAL_ORDER);
        Ok(Value::Series(fps))
    } else {
        Err(EvalError::Other(format!(
            "exponent must be an integer, got {}", r.0
        )))
    }
}
(Value::Series(fps), Value::Rational(r)) => {
    if r.denom() == &rug::Integer::from(1) {
        let exp = r.numer().to_i64().ok_or_else(|| EvalError::Other(
            "exponent too large".to_string(),
        ))?;
        let result = series_pow(fps, exp);
        Ok(Value::Series(result))
    } else {
        Err(EvalError::Other(format!(
            "series exponent must be an integer, got {}", r.0
        )))
    }
}
(Value::Integer(base), Value::Rational(r)) => {
    if r.denom() == &rug::Integer::from(1) {
        let exp = r.numer().to_i64().ok_or_else(|| EvalError::Other(
            "exponent too large".to_string(),
        ))?;
        // Delegate to existing Integer^Integer logic
        eval_pow(Value::Integer(base.clone()), Value::Integer(QInt(r.numer().clone())), env)
    } else {
        Err(EvalError::Other(format!(
            "integer exponent must be an integer, got {}", r.0
        )))
    }
}
(Value::Rational(base), Value::Rational(r)) => {
    if r.denom() == &rug::Integer::from(1) {
        let exp = r.numer().to_i64().ok_or_else(|| EvalError::Other(
            "exponent too large".to_string(),
        ))?;
        eval_pow(Value::Rational(base.clone()), Value::Integer(QInt(r.numer().clone())), env)
    } else {
        Err(EvalError::Other(format!(
            "rational exponent must be an integer, got {}", r.0
        )))
    }
}
(Value::JacobiProduct(factors), Value::Rational(r)) => {
    if r.denom() == &rug::Integer::from(1) {
        let exp = r.numer().to_i64().ok_or_else(|| EvalError::Other(
            "exponent too large".to_string(),
        ))?;
        let scaled: Vec<_> = factors.iter().map(|&(a, b, e)| (a, b, e * exp)).collect();
        Ok(Value::JacobiProduct(normalize_jacobi_product(scaled)))
    } else {
        Err(EvalError::Other(format!(
            "Jacobi product exponent must be an integer, got {}", r.0
        )))
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No series re-truncation | `series(f, q, T)` function | Phase 43 | Users can control truncation order post-computation |
| No expand function | `expand(expr)` function | Phase 43 | JacobiProduct values can be converted to series without jac2series |
| Symbol^Rational fails | Symbol^Rational works when denom=1 | Phase 43 | Runtime arithmetic in q-exponents works in for-loops |
| No floor function | `floor(x)` builtin | Phase 43 | Enables number-theoretic computations |
| No Legendre symbol | `legendre(m, p)` builtin | Phase 43 | Enables quadratic residue computations |

## Design Decisions

### series() Semantics
- `series(expr, q, T)` takes 3 arguments: a series, a symbol (for Maple compatibility), and the target truncation order
- The symbol argument is validated but not used (the series already has its variable)
- When T > original truncation order, use `min(T, original)` to prevent false precision
- When T <= 0, return the zero series (valid edge case)

### expand() Semantics
- `expand(expr)` takes 1 argument (or 1-2 if we want optional truncation order for JacobiProduct)
- For Series: return as-is (already expanded)
- For JacobiProduct: convert to series using `env.default_order` as truncation order
- For Integer/Rational: return as-is
- Alternative: require `expand(JP, q, T)` with explicit order (mirrors jac2series). This is more explicit but less Maple-like. Recommendation: support both `expand(expr)` and `expand(expr, q, T)`.

### legendre() vs jacobi()
- GMP provides both `Integer::legendre()` (odd prime p only) and `Integer::jacobi()` (any odd positive n)
- Maple uses `numtheory[legendre]` which assumes p is prime
- Recommendation: implement as `legendre(m, p)` using GMP's `legendre()`, validate p >= 3 and p is odd. Also register `L` as an alias per UTIL-02 spec.
- Note: GMP's `legendre()` does NOT verify primality -- it just assumes p is odd prime. Passing a composite will silently give wrong results. A warning in help text is sufficient.

### Alias for legendre
- UTIL-02 mentions `L(m, p)` as an alternative syntax
- `L` is a single uppercase letter -- could conflict with user variable names
- Recommendation: register `L` as an alias via `resolve_alias()`, document it in help

## Open Questions

1. **expand() truncation order for JacobiProduct**
   - What we know: JacobiProduct has no inherent truncation order; `jac2series(JP, q, T)` requires explicit T
   - What's unclear: Should `expand(JP)` use `env.default_order` implicitly, or should it require `expand(JP, q, T)`?
   - Recommendation: Accept both forms. `expand(expr)` uses default_order; `expand(expr, q, T)` uses explicit T. This matches Maple flexibility.

2. **series() on non-Series values**
   - What we know: Maple's `series()` can operate on arbitrary expressions
   - What's unclear: Should `series(3, q, 10)` work (returning 3 as a constant series)?
   - Recommendation: Accept Integer/Rational and wrap as constant series. Accept JacobiProduct and convert+truncate. This makes `series()` more flexible.

## Sources

### Primary (HIGH confidence)
- `crates/qsym-cli/src/eval.rs` -- existing dispatch, eval_pow, Value enum, function registration pattern
- `crates/qsym-core/src/series/mod.rs` -- FormalPowerSeries API, from_coeffs(), truncation_order semantics
- `crates/qsym-core/src/series/arithmetic.rs` -- mul, add, invert, shift operations
- `crates/qsym-core/src/number.rs` -- QInt, QRat wrappers, numer/denom accessors
- [rug::Integer docs](https://docs.rs/rug/latest/rug/struct.Integer.html) -- legendre() method confirmed
- [rug::Rational docs](https://docs.rs/rug/latest/rug/struct.Rational.html) -- floor_ref() method confirmed

### Secondary (MEDIUM confidence)
- [Garvan qmaple.pdf](https://qseries.org/fgarvan/papers/qmaple.pdf) -- Maple series/expand semantics for q-series context
- [rug crate](https://docs.rs/crate/rug/latest) -- version 1.28 compatibility confirmed

### Tertiary (LOW confidence)
- None -- all findings verified against source code and official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in use, APIs verified in source code
- Architecture: HIGH -- follows exact existing patterns (dispatch, eval_pow), verified against 89 existing functions
- Pitfalls: HIGH -- identified from direct code analysis (eval_div always produces Rational, truncation_order invariants)
- Code examples: HIGH -- derived from existing code patterns with verified APIs

**Research date:** 2026-02-20
**Valid until:** 2026-03-20 (stable codebase, no external dependency changes expected)
