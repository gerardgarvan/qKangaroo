# Phase 54: Series & Utility Functions - Research

**Researched:** 2026-02-22
**Domain:** CLI evaluator function dispatch -- series coefficient extraction, rational decomposition, modular arithmetic, type checking, boolean evaluation, string concatenation
**Confidence:** HIGH

## Summary

Phase 54 adds 9 new functions to the q-Kangaroo CLI evaluator: `coeff`, `degree`, `numer`, `denom`, `modp`, `mods`, `type`, `evalb`, and `cat`. All follow the established dispatch pattern in `eval.rs` -- match arms in the `dispatch()` function, entries in `ALL_FUNCTION_NAMES`, `get_signature()`, `canonical_function_names()`, `general_help()`, and `FUNC_HELP`. No new crate dependencies or core library changes are needed.

The core infrastructure already exists: `FormalPowerSeries::coeff(k)` extracts coefficients, `QRat::numer()/denom()` decompose rationals, `Value::type_name()` returns type strings, and comparisons already evaluate to `Value::Bool`. The work is purely additive: adding dispatch arms, help text, tab completion, and tests.

**Primary recommendation:** Implement all 9 functions in a single plan (one wave) since they are independent, purely additive to eval.rs, and follow the exact same pattern as the Phase 53 list operations.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SERIES-01 | `coeff(f, q, n)` extracts coefficient of q^n from series | FPS::coeff(k) API exists; dispatch arm matches series+symbol+integer pattern |
| SERIES-02 | `degree(f, q)` returns degree of polynomial/series | qseries::qdegree() exists; dispatch arm wraps with Maple 2-arg signature |
| SERIES-03 | `numer(f)` and `denom(f)` extract numerator/denominator | QRat::numer()/denom() exist; handle Integer/Rational/Series-coefficient cases |
| UTIL-01 | `modp(a, p)` and `mods(a, p)` for modular arithmetic | Pure integer math; modp = a.rem_euclid(p), mods = symmetric variant |
| UTIL-02 | `type(expr, t)` checks expression type | Value::type_name() returns strings; match symbol arg against type names |
| UTIL-03 | `evalb(expr)` evaluates boolean expression | Comparisons already evaluate to Value::Bool; evalb is identity on bools |
| UTIL-04 | `cat(s1, s2, ...)` concatenates strings/names | Variadic; format each arg as string, concatenate, return Value::Symbol |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| qsym-core | local | FormalPowerSeries::coeff(), QRat numer/denom, qdegree | Already in codebase |
| rug | 1.26 | Arbitrary precision Integer/Rational, rem_euc() for modular arithmetic | Already a dependency |

### Supporting
No new dependencies needed. All functions are implemented purely with existing Value types and qsym-core APIs.

## Architecture Patterns

### File Modification Map
```
crates/qsym-cli/src/eval.rs    -- dispatch arms, ALL_FUNCTION_NAMES, get_signature()
crates/qsym-cli/src/help.rs    -- FUNC_HELP entries, general_help() categories
crates/qsym-cli/src/repl.rs    -- canonical_function_names() entries
```

### Pattern: Adding a New Dispatch Function
Every new CLI function follows this exact 5-step pattern (established through 51 prior phases):

1. **Dispatch arm** in `dispatch()` function (~line 3304-5445):
```rust
"function_name" => {
    expect_args(name, args, N)?;
    // extract and validate args
    // compute result
    Ok(Value::SomeVariant(result))
}
```

2. **ALL_FUNCTION_NAMES** array (~line 6242): Add to the appropriate group comment
3. **get_signature()** function (~line 6060): Add match arm returning signature string
4. **FUNC_HELP** array in help.rs (~line 174): Add FuncHelp struct entry
5. **general_help()** in help.rs (~line 16): Add to appropriate category
6. **canonical_function_names()** in repl.rs (~line 66): Add to function list

### Pattern: Argument Extraction
Existing helpers that Phase 54 functions will use:
```rust
// Exact arg count
expect_args(name, args, 2)?;
// Range of arg counts
expect_args_range(name, args, 1, 3)?;
// Extract integer from args[i]
let n = extract_i64(name, args, 0)?;
// Extract rational from args[i]
let r = extract_qrat(name, args, 0)?;
// Extract series from args[i]
let fps = extract_series(name, args, 0)?;
// Extract symbol ID (for variable name like q)
let sym = extract_symbol_id(name, args, 1, env)?;
```

### Pattern: Returning Values
```rust
Ok(Value::Integer(QInt::from(42i64)))
Ok(Value::Rational(QRat::from((3i64, 4i64))))
Ok(Value::Bool(true))
Ok(Value::Symbol("abc".to_string()))
Ok(Value::None)
```

### Placement in dispatch()
New functions should be added as a new section before the "Unknown function" catch-all (`_ =>` at ~line 5437), after the List Operations section. Suggested section header:
```rust
// =================================================================
// Series Coefficient & Utility Functions
// =================================================================
```

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Coefficient extraction | Manual BTreeMap lookup | `FormalPowerSeries::coeff(k)` | Handles truncation bounds, zero coefficients |
| Highest degree | Manual iterator scan | `qseries::qdegree(&fps)` | Already tested, handles edge cases |
| Rational decomposition | Manual bigint division | `QRat::numer()` / `QRat::denom()` | rug handles canonical forms |
| Euclidean mod | Manual `%` with sign fix | `rug::Integer::rem_euc()` | Handles negative numbers correctly |
| Type name strings | Duplicated match arms | `Value::type_name()` | Already returns correct strings for all 19 variants |

## Common Pitfalls

### Pitfall 1: coeff() Truncation Order Check
**What goes wrong:** Calling `fps.coeff(k)` where `k >= fps.truncation_order()` panics.
**Why it happens:** The FPS `.coeff()` method asserts `k < truncation_order`.
**How to avoid:** Before extracting, check `n < fps.truncation_order()`. If `n >= truncation_order`, return an error like "coefficient at q^N is unknown (series truncated at O(q^T))".
**Warning signs:** Panic in tests when using `coeff(f, q, 25)` on a series truncated at O(q^20).

### Pitfall 2: modp/mods Sign Handling
**What goes wrong:** Rust's `%` operator is remainder (can be negative for negative dividend), but Maple's `modp` always returns non-negative.
**Why it happens:** `(-7) % 3 == -1` in Rust, but `modp(-7, 3) == 2` in Maple.
**How to avoid:** Use `rug::Integer::rem_euc()` or compute `((a % p) + p) % p` for i64 values.
**Warning signs:** Negative results from `modp`.

### Pitfall 3: mods() Symmetric Range
**What goes wrong:** `mods(5, 3)` should return `-1` (since 5 mod 3 = 2, and 2 > 3/2, so subtract 3 to get -1). Getting the boundary conditions wrong.
**Why it happens:** Maple's `mods(a, p)` returns the unique `r` in `[-(p-1)/2, (p-1)/2]` (for odd p) or `[-p/2+1, p/2]` such that `a === r (mod p)`.
**How to avoid:** Compute `r = modp(a, p)`, then if `r > p/2`, return `r - p`. For even p, Maple's convention is `r in (-p/2, p/2]`.
**Warning signs:** Wrong sign or boundary values at p/2.

### Pitfall 4: type() Second Argument Matching
**What goes wrong:** `type(42, integer)` works but `type(42, "integer")` doesn't, or vice versa.
**Why it happens:** The second argument could be either `Value::Symbol("integer")` (from unbound variable) or `Value::String("integer")` (from string literal).
**How to avoid:** Accept both `Value::Symbol(s)` and `Value::String(s)` as the type name. Extract the string and match against `Value::type_name()` return values.
**Warning signs:** Tests pass for symbol but fail for string, or vice versa.

### Pitfall 5: coeff() on Integer/Rational Values
**What goes wrong:** `coeff(42, q, 0)` -- user passes a constant, not a series.
**Why it happens:** In Maple, `coeff(42, q, 0)` returns 42 (the constant term).
**How to avoid:** Handle `Value::Integer` and `Value::Rational` as special cases: if `n == 0`, return the value; otherwise return 0.
**Warning signs:** Error message instead of expected result when passing constants.

### Pitfall 6: degree() on Constants
**What goes wrong:** `degree(42, q)` -- user passes a constant.
**Why it happens:** In Maple, `degree(42, q)` returns 0 (constant polynomial has degree 0).
**How to avoid:** Handle `Value::Integer` and `Value::Rational` as special cases returning 0.
**Warning signs:** Error instead of 0.

### Pitfall 7: numer/denom on Integer
**What goes wrong:** `numer(42)` should return 42, `denom(42)` should return 1.
**Why it happens:** Integer values don't have an explicit denominator.
**How to avoid:** Match `Value::Integer(n)` separately: numer returns n, denom returns 1.
**Warning signs:** Type error on integer input.

### Pitfall 8: cat() Display Formatting
**What goes wrong:** `cat(3, "x")` returns the wrong string representation.
**Why it happens:** Need to format each argument appropriately -- integers as digits, symbols as their name, strings as their content.
**How to avoid:** Use a consistent formatting approach: integers display as their decimal string, symbols as their name, strings as their content (no quotes).
**Warning signs:** Extra quotes or wrong number formatting in cat output.

### Pitfall 9: evalb() is Not Just Identity
**What goes wrong:** `evalb(1)` or `evalb(0)` -- user passes an integer.
**Why it happens:** In Maple, `evalb(expr)` evaluates its argument as a boolean. Since our system already evaluates comparisons, most calls will pass a `Value::Bool`. But integers should also work: `evalb(0)` -> false, `evalb(1)` -> true.
**How to avoid:** Handle `Value::Bool(b)` (return as-is), `Value::Integer(n)` (0 -> false, nonzero -> true), and error for other types.
**Warning signs:** Error on integer input to evalb.

### Pitfall 10: Function Count Synchronization
**What goes wrong:** Tests that check the function count (e.g., `assert_eq!(canonical.len(), 103)`) fail.
**Why it happens:** Adding functions without updating all the count assertions.
**How to avoid:** Search for `103` or current count in help.rs tests and update. Currently at 103 FUNC_HELP entries and 105 in canonical_function_names.
**Warning signs:** Test failure mentioning "expected 103, got 112" or similar.

## Code Examples

### coeff(f, q, n) -- Coefficient Extraction
```rust
// Source: FormalPowerSeries::coeff() in series/mod.rs line 92
"coeff" => {
    expect_args(name, args, 3)?;
    match &args[0] {
        Value::Series(fps) => {
            let _sym = extract_symbol_id(name, args, 1, env)?;
            let n = extract_i64(name, args, 2)?;
            if n >= fps.truncation_order() {
                return Err(EvalError::Other(format!(
                    "coeff: q^{} is beyond truncation order O(q^{})",
                    n, fps.truncation_order()
                )));
            }
            let c = fps.coeff(n);
            if *c.denom() == 1 {
                Ok(Value::Integer(QInt(c.numer().clone())))
            } else {
                Ok(Value::Rational(c))
            }
        }
        Value::Integer(n) => {
            let _sym = extract_symbol_id(name, args, 1, env)?;
            let exp = extract_i64(name, args, 2)?;
            if exp == 0 { Ok(Value::Integer(n.clone())) }
            else { Ok(Value::Integer(QInt::from(0i64))) }
        }
        Value::Rational(r) => {
            let _sym = extract_symbol_id(name, args, 1, env)?;
            let exp = extract_i64(name, args, 2)?;
            if exp == 0 { Ok(Value::Rational(r.clone())) }
            else { Ok(Value::Integer(QInt::from(0i64))) }
        }
        other => Err(EvalError::ArgType { ... })
    }
}
```

### degree(f, q) -- Polynomial/Series Degree
```rust
// Source: qseries::qdegree() in qseries/utilities.rs line 71
"degree" => {
    expect_args(name, args, 2)?;
    let _sym = extract_symbol_id(name, args, 1, env)?;
    match &args[0] {
        Value::Series(fps) => {
            match qseries::qdegree(fps) {
                Some(d) => Ok(Value::Integer(QInt::from(d))),
                None => Ok(Value::Integer(QInt::from(0i64))), // zero polynomial
            }
        }
        Value::Integer(_) | Value::Rational(_) => {
            Ok(Value::Integer(QInt::from(0i64))) // constants have degree 0
        }
        other => Err(EvalError::ArgType { ... })
    }
}
```

### numer(f) and denom(f) -- Rational Decomposition
```rust
// Source: QRat::numer()/denom() in number.rs lines 312-319
"numer" => {
    expect_args(name, args, 1)?;
    match &args[0] {
        Value::Rational(r) => Ok(Value::Integer(QInt(r.numer().clone()))),
        Value::Integer(n) => Ok(Value::Integer(n.clone())),
        other => Err(EvalError::ArgType { ... })
    }
}

"denom" => {
    expect_args(name, args, 1)?;
    match &args[0] {
        Value::Rational(r) => Ok(Value::Integer(QInt(r.denom().clone()))),
        Value::Integer(_) => Ok(Value::Integer(QInt::from(1i64))),
        other => Err(EvalError::ArgType { ... })
    }
}
```

### modp(a, p) and mods(a, p) -- Modular Arithmetic
```rust
// For i64 values:
"modp" => {
    expect_args(name, args, 2)?;
    let a = extract_i64(name, args, 0)?;
    let p = extract_i64(name, args, 1)?;
    if p <= 0 {
        return Err(EvalError::Other("modp: modulus must be positive".into()));
    }
    let result = ((a % p) + p) % p;  // ensures non-negative
    Ok(Value::Integer(QInt::from(result)))
}

"mods" => {
    expect_args(name, args, 2)?;
    let a = extract_i64(name, args, 0)?;
    let p = extract_i64(name, args, 1)?;
    if p <= 0 {
        return Err(EvalError::Other("mods: modulus must be positive".into()));
    }
    let r = ((a % p) + p) % p;  // non-negative remainder
    // Symmetric: if r > p/2, subtract p
    if r * 2 > p { Ok(Value::Integer(QInt::from(r - p))) }
    else { Ok(Value::Integer(QInt::from(r))) }
}
```

### type(expr, t) -- Type Checking
```rust
"type" => {
    expect_args(name, args, 2)?;
    let type_name = match &args[1] {
        Value::Symbol(s) => s.as_str(),
        Value::String(s) => s.as_str(),
        other => return Err(EvalError::ArgType { ... }),
    };
    let matches = match type_name {
        "integer" => matches!(&args[0], Value::Integer(_)),
        "rational" => matches!(&args[0], Value::Rational(_)),
        "numeric" => matches!(&args[0], Value::Integer(_) | Value::Rational(_)),
        "series" => matches!(&args[0], Value::Series(_)),
        "list" => matches!(&args[0], Value::List(_)),
        "string" => matches!(&args[0], Value::String(_)),
        "boolean" => matches!(&args[0], Value::Bool(_)),
        "symbol" | "name" => matches!(&args[0], Value::Symbol(_)),
        "procedure" => matches!(&args[0], Value::Procedure(_)),
        "infinity" => matches!(&args[0], Value::Infinity),
        _ => false, // unknown type name
    };
    Ok(Value::Bool(matches))
}
```

### evalb(expr) -- Boolean Evaluation
```rust
"evalb" => {
    expect_args(name, args, 1)?;
    match &args[0] {
        Value::Bool(b) => Ok(Value::Bool(*b)),
        Value::Integer(n) => Ok(Value::Bool(!n.is_zero())),
        other => Err(EvalError::Other(format!(
            "evalb: expected boolean or integer, got {}", other.type_name()
        )))
    }
}
```

### cat(s1, s2, ...) -- String/Name Concatenation
```rust
"cat" => {
    if args.is_empty() {
        return Err(EvalError::WrongArgCount { ... });
    }
    let mut result = String::new();
    for arg in args {
        match arg {
            Value::Symbol(s) => result.push_str(s),
            Value::String(s) => result.push_str(s),
            Value::Integer(n) => result.push_str(&n.0.to_string()),
            Value::Rational(r) => result.push_str(&format!("{}/{}", r.numer(), r.denom())),
            Value::Bool(b) => result.push_str(if *b { "true" } else { "false" }),
            _ => result.push_str(arg.type_name()),
        }
    }
    Ok(Value::Symbol(result))
}
```

## Implementation Details

### Function Count Updates
Current counts that need updating:
- `ALL_FUNCTION_NAMES` in eval.rs (~line 6242): Currently ~105 entries. Add 9 new = 114.
- `FUNC_HELP` in help.rs (~line 174): Currently 103 entries. Add 9 new = 112.
- `canonical_function_names()` in repl.rs (~line 66): Currently 105 entries. Add 9 new = 114.
- Test assertion in help.rs: `assert_eq!(canonical.len(), 103, ...)` -> update to 112.

Note: FUNC_HELP count (103) differs from ALL_FUNCTION_NAMES (105) because some entries like `print` and the `?` help prefix are handled specially.

### Maple Semantics Reference

| Function | Maple Behavior | Our Implementation |
|----------|---------------|-------------------|
| `coeff(f, q, n)` | Extract coefficient of q^n | `fps.coeff(n)`, with truncation check |
| `degree(f, q)` | Highest exponent with nonzero coeff | `qseries::qdegree(fps)` |
| `numer(r)` | Numerator of rational | `QRat::numer()` |
| `denom(r)` | Denominator of rational | `QRat::denom()` |
| `modp(a, p)` | `a mod p`, always non-negative | `((a % p) + p) % p` |
| `mods(a, p)` | Symmetric mod, `[-p/2, p/2)` | `modp` then adjust if > p/2 |
| `type(expr, t)` | Boolean type check | Match type_name against symbol |
| `evalb(expr)` | Force boolean evaluation | Identity on Bool, 0/nonzero for Int |
| `cat(s1, ...)` | Concatenate to name | Stringify each arg, return Symbol |

### Test Plan
Each function needs:
1. **Unit tests in eval.rs** -- `dispatch("funcname", &args, &mut env)` pattern
2. **Help test in help.rs** -- `function_help("funcname")` returns Some
3. **Integration test in cli_integration.rs** -- `run_piped("expr;")` checks output

Estimated new tests: ~30-40 (3-4 per function).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `qdegree(f)` only | Also `degree(f, q)` Maple-style | Phase 54 | Maple scripts work without modification |
| No coeff extraction | `coeff(f, q, n)` available | Phase 54 | Core research workflow enabled |
| No type checking | `type(expr, t)` available | Phase 54 | Debugging and conditional logic in scripts |

## Open Questions

1. **coeff() on FractionalPowerSeries / BivariateSeries**
   - What we know: The success criteria only mention FPS (univariate series)
   - What's unclear: Should coeff work on BivariateSeries or FractionalPowerSeries?
   - Recommendation: Start with FPS/Integer/Rational only. Add bivariate support later if needed. Error on unsupported types.

2. **degree() for exact polynomials (POLYNOMIAL_ORDER)**
   - What we know: Polynomials use POLYNOMIAL_ORDER sentinel (1_000_000_000) as truncation_order
   - What's unclear: Should degree filter by POLYNOMIAL_ORDER?
   - Recommendation: `qdegree()` already works correctly on polynomials since it looks at actual nonzero coefficients, not truncation_order. No special handling needed.

3. **type() extended type names**
   - What we know: Maple has many type names (posint, negint, even, odd, etc.)
   - What's unclear: How many should we support?
   - Recommendation: Support the basic Value variant types plus "numeric" (integer or rational). Extended numeric predicates can be added later.

4. **numer/denom on series**
   - What we know: Success criteria says "they work on rational series terms"
   - What's unclear: Does this mean extracting numer/denom from individual coefficients, or from the series as a whole?
   - Recommendation: For now, handle Integer and Rational directly. For series, this is ambiguous -- error with a helpful message. The success criteria example only shows `numer(3/4)`.

## Sources

### Primary (HIGH confidence)
- `crates/qsym-core/src/series/mod.rs` -- FormalPowerSeries API, coeff() method (line 92)
- `crates/qsym-core/src/number.rs` -- QRat numer()/denom() methods (lines 312-319)
- `crates/qsym-core/src/qseries/utilities.rs` -- qdegree() function (line 71)
- `crates/qsym-cli/src/eval.rs` -- dispatch() function pattern, Value enum (19 variants), existing helpers

### Secondary (MEDIUM confidence)
- Maple documentation for modp/mods semantics (Maple convention for symmetric mod)
- Maple documentation for type() supported type names

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all APIs exist in codebase, verified by reading source
- Architecture: HIGH -- follows exact pattern of 100+ existing dispatch functions
- Pitfalls: HIGH -- derived from reading FPS::coeff() assertions and Rust `%` semantics
- Maple semantics: MEDIUM -- modp/mods/type conventions from training data, not verified against current Maple docs

**Research date:** 2026-02-22
**Valid until:** Indefinite (stable codebase patterns, no external dependencies)
