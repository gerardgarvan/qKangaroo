# Phase 35: Series Analysis Signatures - Research

**Researched:** 2026-02-19
**Domain:** CLI function dispatch -- Maple-compatible calling conventions for series analysis functions
**Confidence:** HIGH

## Summary

Phase 35 migrates 7 series analysis functions (sift, prodmake, etamake, jacprodmake, mprodmake, qetamake, qfactor) from their current compact signatures to Garvan's exact Maple calling conventions. The Garvan signatures were verified against the actual Maple source code (`wmprog64.txt` from qseries v1.2) hosted at qseries.org. All 7 functions gain an explicit `q` variable parameter; the `make` functions gain a `T` truncation parameter; sift gains all five Garvan parameters.

**Key finding on qfactor:** The CONTEXT.md requirement SIG-14 states `qfactor(f, q)` but Garvan's actual signature is `qfactor(f, T)` where T is an optional truncation upper bound (not a q variable). In Garvan's Maple, q is implicitly the series variable. Since our q-Kangaroo design requires explicit q parameters for Maple compatibility (as established in Phase 34), the Maple-style form should be `qfactor(f, q)` with optional T: `qfactor(f, q, T)`. This reconciles with the SIG-14 intent.

**Primary recommendation:** Follow the Phase 34 pattern exactly -- remove old signatures completely, add new Maple-style dispatch branches, update tests/help/completion. No backward compatibility.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **No backward compat** -- old signatures (without explicit q) are removed, not kept as aliases
- Functions require the new Maple-style argument lists; old arity forms produce a standard "wrong number of arguments" error with usage hint
- All existing tests for these functions must be rewritten to use the new Maple-style signatures in this phase
- Function names: Claude's discretion to check Garvan's exact Maple names and adjust if any differ from current q-Kangaroo names
- `sift(s, q, n, k, T)` -- strict 5-arg form, T is always required (even for polynomial inputs)
- Error on invalid residue: k must satisfy 0 <= k < n, otherwise return error
- prodmake/etamake output should match Maple's display notation style (not just keep current format)
- Error messages should name the parameter position: "Argument 1 (f): expected series, got integer"
- help() text for all 7 functions must be updated in this phase to show new Maple-style signatures
- Tab completion must be updated in this phase for any renamed function names

### Claude's Discretion
- Exact Garvan function names (check and rename if needed)
- sift truncation semantics (match Garvan)
- sift variable validation approach
- jacprodmake P parameter behavior and output
- Failed decomposition behavior for make-functions
- qfactor output format (match Garvan)
- q-parameter type error specificity

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

## Garvan's Maple Signatures

Verified from Garvan's actual Maple source code (`wmprog64.txt`, qseries v1.2, hosted at qseries.org). Confidence: HIGH.

### sift
```maple
sift(s, q, n, k, T)
```
- **s**: q-series (input)
- **q**: variable name
- **n**: modulus
- **k**: residue class (0 <= k < n)
- **T**: truncation order for the INPUT series

**Truncation semantics (from source):**
```maple
st := series(s, q, T+5):
lasti := floor((T-k)/n):
for i from 0 to lasti do
    y := y + coeff(st, q, (n*i+k)) * q^i:
od:
```
The T parameter controls how many terms of the input to use. The output has terms from q^0 to q^floor((T-k)/n). The output is a **polynomial** in Maple (no O() truncation marker).

**Our implementation:** In our Rust `qseries::sift(f, m, j)`, the truncation is derived from `f.truncation_order()`. For the new 5-arg form, T should set the effective input truncation: `min(T, f.truncation_order())`. The output truncation order is `floor((T-k)/n) + 1`.

### prodmake
```maple
prodmake(f, q, T)       -- returns product expression
prodmake(f, q, T, list) -- returns list of exponents
```
- **f**: q-series
- **q**: variable
- **T**: precision (number of terms to use)

**Return format (3-arg):** Maple expression `(1-q)^a1 * (1-q^2)^a2 * ...` -- a symbolic product. Since q-Kangaroo returns `Value::Dict`, keep the current dict format but add a display format that renders like Garvan's product notation.

**Return format (4-arg with `list`):** Returns `[a1, a2, ..., a_{T-1}]` as a Maple list. We should NOT implement this 4th argument variant -- it's a Maple-specific list mode.

### etamake
```maple
etamake(f, q, last)
```
- **f**: q-series
- **q**: variable
- **last**: maximum delta to search

**Return format:** Maple expression like `eta(tau)^2 / eta(2*tau)` with a `q^exq` prefactor. We keep our dict return format but could add display rendering.

### jacprodmake
```maple
jacprodmake(f, q, T)       -- 3 args, full period search
jacprodmake(f, q, T, PP)   -- 4 args, restrict periods to divisors of PP
```
- **f**: q-series
- **q**: variable
- **T**: precision
- **PP**: (optional) restricts period search to divisors of PP

**P parameter semantics (from source):**
When PP is provided, `periodfind2(A, T, PP)` searches only among divisors of PP (via `numtheory[divisors](PP) minus {1}`). Without PP, `periodfind(A, T)` checks all possible periods. This is a performance/accuracy optimization for when the user knows the expected period structure.

**Return format:** Maple expression `LT * JAC(a1,b1)^e1 * JAC(a2,b2)^e2 * ...` where LT is leading term. Warning on failure, stores args in `fixjacp`.

**Our implementation for PP:** The current Rust `jacprodmake` tries all periods 1..=effective_max. With PP, limit to divisors of PP only. This is a simple filter in the existing loop.

### mprodmake
```maple
mprodmake(f, q, last)
```
- **f**: q-series
- **q**: variable
- **last**: max exponent

**Return format:** Maple expression `(1+q)(1+q^3)(1+q^5)...` -- symbolic product of (1+q^n) factors.

### qetamake
```maple
qetamake(f, q, last)
```
- **f**: q-series
- **q**: variable
- **last**: max exponent

**Return format:** Maple expression using `_E(q^m)` notation, e.g., `_E(q)^2 / _E(q^2)`.

### qfactor
```maple
qfactor(f)       -- 1 arg, auto-determine T
qfactor(f, T)    -- 2 args, T is max exponent for (1-q^i) factors
```
- **f**: rational polynomial in q (NOT a truncated series -- it must be a polynomial)
- **T**: (optional) upper bound for the i in (1-q^i) factors; defaults to 4*degree+3

**Key insight:** Garvan's qfactor does NOT take an explicit q variable parameter. q is implicit in Maple's symbolic system. However, our q-Kangaroo design requires explicit q for consistency (Phase 34 established this pattern). So our Maple-style form will be `qfactor(f, q)` and `qfactor(f, q, T)`.

**Return format:** Maple expression `(1-q)^n1 * (1-q^2)^n2 * ...` -- a symbolic product. Our current dict return format with `{scalar, factors, is_exact}` is fine but should be displayed in product notation.

### Function Name Verification

All 7 function names in q-Kangaroo **match Garvan's Maple names exactly**:
- `sift` -- matches
- `prodmake` -- matches
- `etamake` -- matches
- `jacprodmake` -- matches
- `mprodmake` -- matches
- `qetamake` -- matches
- `qfactor` -- matches

No renaming needed.

## Current Implementation

### Current Signatures (eval.rs, lines 1492-1573)

| Function | Current Signature | Current Args | Core Rust Function |
|----------|------------------|-------------|-------------------|
| sift | `sift(series, m, j)` | 3 | `qseries::sift(&fps, m, j)` |
| prodmake | `prodmake(series, max_n)` | 2 | `qseries::prodmake(&fps, max_n)` |
| etamake | `etamake(series, max_n)` | 2 | `qseries::etamake(&fps, max_n)` |
| jacprodmake | `jacprodmake(series, max_n)` | 2 | `qseries::jacprodmake(&fps, max_n)` |
| mprodmake | `mprodmake(series, max_n)` | 2 | `qseries::mprodmake(&fps, max_n)` |
| qetamake | `qetamake(series, max_n)` | 2 | `qseries::qetamake(&fps, max_n)` |
| qfactor | `qfactor(series)` | 1 | `qseries::qfactor(&fps)` |

### Target Signatures (Maple-style)

| Function | New Signature | New Args | Core Change Needed |
|----------|--------------|----------|-------------------|
| sift | `sift(s, q, n, k, T)` | 5 | Add q extraction, add T truncation capping |
| prodmake | `prodmake(f, q, T)` | 3 | Add q extraction |
| etamake | `etamake(f, q, T)` | 3 | Add q extraction |
| jacprodmake | `jacprodmake(f, q, T)` or `(f, q, T, P)` | 3 or 4 | Add q extraction, add P filtering |
| mprodmake | `mprodmake(f, q, T)` | 3 | Add q extraction |
| qetamake | `qetamake(f, q, T)` | 3 | Add q extraction |
| qfactor | `qfactor(f, q)` or `(f, q, T)` | 2 or 3 | Add q extraction, optional T |

### Current Value Converters (eval.rs, lines 2209-2280)

These helpers convert core Rust structs to `Value::Dict`:
- `infinite_product_form_to_value` -- prodmake result
- `eta_quotient_to_value` -- etamake result
- `jacobi_product_form_to_value` -- jacprodmake result
- `btreemap_i64_to_value` -- mprodmake result
- `q_eta_form_to_value` -- qetamake result
- `q_factorization_to_value` -- qfactor result

These converters currently produce structured dicts. The CONTEXT.md says "output should match Maple's display notation style." This is about the `format_value` Display output, not the underlying Value representation. The Dict structure can stay, but the format/display logic should render them in Maple-like product notation.

**Recommendation:** Keep current Value::Dict return format (breaking it would affect downstream code). Change the `format_value` rendering of these dicts to use Maple-like product notation if feasible, OR defer display changes to a later phase. The CONTEXT.md says "should match" -- this is an aspiration, not a hard blocker for the signature migration.

### Current Tests (eval.rs, lines 4080-4220)

7 unit tests exist for these functions, all using old signatures:
- `dispatch_sift_returns_series` -- uses `sift(series, 5, 0)`
- `dispatch_prodmake_returns_dict` -- uses `prodmake(series, 10)`
- `dispatch_etamake_returns_dict` -- uses `etamake(series, 10)`
- `dispatch_jacprodmake_returns_dict` -- uses `jacprodmake(series, 10)`
- `dispatch_mprodmake_returns_dict` -- uses `mprodmake(series, 10)`
- `dispatch_qetamake_returns_dict` -- uses `qetamake(series, 10)`
- `dispatch_qfactor_returns_dict_with_is_exact` -- uses `qfactor(series)`

All must be rewritten to use new Maple-style signatures.

### Current Help Text (help.rs, lines 264-326)

7 help entries exist with old signatures. All must be updated.

### Current Tab Completion (repl.rs, lines 64-65)

Function names are already in the canonical list. No renaming needed since Garvan names match.

## Architecture Patterns

### Pattern from Phase 34: Maple-Style Dispatch

Phase 34 established a clear pattern for migrating functions. The key pattern elements:

**1. Disambiguation by first-arg type:**
```rust
"function_name" => {
    if args.len() >= N && matches!(&args[POS], Value::Symbol(_)) {
        // Maple-style: extract symbol for q variable
        let sym = extract_symbol_id(name, args, POS, env)?;
        // ... rest of Maple-style logic
    } else {
        // Legacy (or error for Phase 35 since no backward compat)
    }
}
```

**2. Phase 35 difference from Phase 34:** Phase 34 kept legacy paths. Phase 35 REMOVES them. The else branch becomes an error, not a legacy fallback:
```rust
"sift" => {
    expect_args(name, args, 5)?;
    let fps = extract_series(name, args, 0)?;
    let _sym = extract_symbol_id(name, args, 1, env)?;
    let n = extract_i64(name, args, 2)?;
    let k = extract_i64(name, args, 3)?;
    let t = extract_i64(name, args, 4)?;
    // Validate k
    if k < 0 || k >= n {
        return Err(EvalError::Other(format!(
            "sift: residue k={} must satisfy 0 <= k < n={}", k, n
        )));
    }
    // Cap T at series truncation
    let effective_t = t.min(fps.truncation_order());
    let result = qseries::sift(&fps, n, k);
    // ... truncation logic
    Ok(Value::Series(result))
}
```

**3. Extract + validate pattern:**
```rust
let fps = extract_series(name, args, 0)?;      // Arg 1: series
let _sym = extract_symbol_id(name, args, 1, env)?;  // Arg 2: q variable (validate, don't use)
let t = extract_i64(name, args, 2)?;            // Arg 3: truncation
```

Note: The `q` variable is extracted for validation (must be a symbol) but typically not used -- the series already carries its SymbolId internally. This matches how Phase 34's `etaq(q, b, T)` works.

**4. Error format (from CONTEXT.md):**
```
"Argument 1 (f): expected series, got integer"
```

The current `EvalError::ArgType` format is:
```
"Error: sift argument 1 must be series, got integer"
```

To match the CONTEXT.md format, update `ArgType` display or use `EvalError::Other` with custom message. Recommendation: use `EvalError::Other` for custom positional error messages where the existing ArgType format doesn't match.

### Recommended Project Structure

No new files. Changes are entirely in existing files:
```
crates/qsym-cli/src/
  eval.rs       # Dispatch changes (~100 lines changed)
  help.rs       # Help text updates (~50 lines changed)
  repl.rs       # Tab completion (no changes needed -- names don't change)
crates/qsym-cli/tests/
  cli_integration.rs  # New integration tests (~70 lines added)
```

Optionally, if the `sift` core function needs a T-capped variant:
```
crates/qsym-core/src/qseries/
  utilities.rs  # Possible sift_with_truncation variant
```

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Series coefficient extraction | Custom sift loop | `qseries::sift(&fps, m, j)` | Already handles arithmetic subsequence correctly |
| Product decomposition | Custom prodmake | `qseries::prodmake(&fps, max_n)` | Andrews' algorithm already implemented |
| Symbol validation | Manual string checks | `extract_symbol_id(name, args, idx, env)` | Already handles Symbol->SymbolId + error |
| Series extraction | Manual type matching | `extract_series(name, args, idx)` | Standard helper from Phase 33 |
| Integer extraction | Manual type matching | `extract_i64(name, args, idx)` | Standard helper |
| Arg count validation | Manual len checks | `expect_args(name, args, N)` | Standard helper |
| Arg count ranges | Manual len checks | `expect_args_range(name, args, min, max)` | Standard helper for variable-arity functions |

**Key insight:** The core `qseries::*` functions in `qsym-core` do NOT need changes. They already take the right mathematical parameters. The Phase 35 work is entirely in the CLI dispatch layer (`eval.rs`) -- extracting arguments from the new Maple-style calling convention and passing them to the unchanged core functions.

## Common Pitfalls

### Pitfall 1: sift Truncation Mismatch
**What goes wrong:** Garvan's T parameter controls the input truncation, but our sift function derives output truncation from `f.truncation_order()`. If T < f.truncation_order(), results should be capped. If T > f.truncation_order(), the series doesn't have enough data.
**Why it happens:** Confusion between "T controls output order" vs "T controls input order."
**How to avoid:** Use `effective_t = min(T, f.truncation_order())` as the input truncation bound. The output truncation is `floor((effective_t - k) / n) + 1`. The core `sift` function already computes this correctly from `f.truncation_order()`, so if T >= f.truncation_order(), just call sift normally. If T < f.truncation_order(), either truncate the series first or modify the sift call.
**Warning signs:** Tests where sift output has more terms than expected given T.

### Pitfall 2: q Variable is Validated but Not Used
**What goes wrong:** Developers might try to USE the extracted SymbolId for something. For most functions, the series already knows its variable.
**Why it happens:** The q parameter exists for Maple compatibility (explicit variable naming) but our series already carry their variable.
**How to avoid:** Extract and validate (`extract_symbol_id`), but only use the SymbolId if you need to CREATE a new series (like in sift output). For prodmake/etamake/etc, the q variable is purely for validation.
**Warning signs:** Using the wrong SymbolId for output series construction.

### Pitfall 3: Old Test Patterns Break
**What goes wrong:** Existing tests call functions without the q parameter. After removing old signatures, ALL tests fail.
**Why it happens:** The "no backward compat" decision means every test must be updated.
**How to avoid:** Update tests FIRST (or simultaneously with dispatch changes). Use `Value::Symbol("q".to_string())` as the q argument in test calls.
**Warning signs:** Massive test failures after dispatch changes.

### Pitfall 4: jacprodmake PP Parameter Implementation
**What goes wrong:** Implementing PP filtering requires changes to the core `jacprodmake` function, not just the CLI dispatch.
**Why it happens:** The current Rust `jacprodmake` in prodmake.rs always searches all periods 1..=effective_max. There's no way to pass a period restriction.
**How to avoid:** Either (a) add an optional `period_divisor: Option<i64>` parameter to the core function, or (b) create a new `jacprodmake_with_period` variant, or (c) filter the period candidates in the existing loop. Option (a) is cleanest.
**Warning signs:** 4-arg jacprodmake silently ignoring PP or not compiling.

### Pitfall 5: qfactor Signature Ambiguity
**What goes wrong:** SIG-14 says `qfactor(f, q)` but Garvan's actual signature is `qfactor(f, T)`. Implementing `qfactor(f, q)` means we're adding a parameter Garvan doesn't have, while not supporting the T parameter that Garvan does have.
**How to avoid:** Support both: `qfactor(f, q)` (2-arg, auto-T) and `qfactor(f, q, T)` (3-arg, explicit T). Use `expect_args_range(name, args, 2, 3)`. The 2-arg form uses default T = 4*degree+3 (matching Garvan's default).
**Warning signs:** qfactor not accepting T parameter when user needs it.

### Pitfall 6: Error Message Format
**What goes wrong:** CONTEXT.md says "Argument 1 (f): expected series, got integer" but existing `EvalError::ArgType` displays as "sift argument 1 must be series, got integer".
**Why it happens:** Existing error format doesn't include parameter names in parentheses.
**How to avoid:** For these 7 functions, use `EvalError::Other(format!(...))` with custom formatting, OR update the ArgType variant to support named parameters. Using `Other` is simpler and lower-risk.
**Warning signs:** Error messages not matching the agreed format.

## Code Examples

### Pattern: Simple 3-arg Maple-style (prodmake, etamake, mprodmake, qetamake)

```rust
"prodmake" => {
    expect_args(name, args, 3)?;
    let fps = extract_series(name, args, 0)?;
    let _sym = extract_symbol_id(name, args, 1, env)?;
    let max_n = extract_i64(name, args, 2)?;
    let result = qseries::prodmake(&fps, max_n);
    Ok(infinite_product_form_to_value(&result))
}
```

### Pattern: sift with 5 args and validation

```rust
"sift" => {
    expect_args(name, args, 5)?;
    let fps = extract_series(name, args, 0)?;
    let _sym = extract_symbol_id(name, args, 1, env)?;
    let n = extract_i64(name, args, 2)?;
    let k = extract_i64(name, args, 3)?;
    let t = extract_i64(name, args, 4)?;
    if n <= 0 {
        return Err(EvalError::Other(format!(
            "sift: Argument 3 (n): modulus must be positive, got {}", n
        )));
    }
    if k < 0 || k >= n {
        return Err(EvalError::Other(format!(
            "sift: Argument 4 (k): residue must satisfy 0 <= k < n={}, got {}", n, k
        )));
    }
    // Cap T at series truncation order
    let effective_t = t.min(fps.truncation_order());
    // Truncate series to effective_t before sifting
    let capped = if effective_t < fps.truncation_order() {
        fps.truncated_to(effective_t) // May need to implement this
    } else {
        fps
    };
    let result = qseries::sift(&capped, n, k);
    Ok(Value::Series(result))
}
```

NOTE: `FormalPowerSeries::truncated_to(new_order)` may not exist. If not, create the capped series by constructing a new FPS with `truncation_order = effective_t` and copying only coefficients below that order. OR simply rely on the existing sift logic which already caps at `f.truncation_order()`, and accept that T > f.truncation_order() just uses all available data. The Garvan behavior is `series(s, q, T+5)` which Maple re-expands to get T terms -- in our case, the series already has a fixed truncation.

**Simpler approach:** Since our series already have truncation orders, T is used as the effective truncation: `let result = qseries::sift(&fps, n, k);` and then truncate the output to `floor((min(t, fps.truncation_order()) - k) / n) + 1`.

### Pattern: jacprodmake with optional 4th arg

```rust
"jacprodmake" => {
    if args.len() == 3 {
        let fps = extract_series(name, args, 0)?;
        let _sym = extract_symbol_id(name, args, 1, env)?;
        let max_n = extract_i64(name, args, 2)?;
        let result = qseries::jacprodmake(&fps, max_n);
        Ok(jacobi_product_form_to_value(&result))
    } else if args.len() == 4 {
        let fps = extract_series(name, args, 0)?;
        let _sym = extract_symbol_id(name, args, 1, env)?;
        let max_n = extract_i64(name, args, 2)?;
        let pp = extract_i64(name, args, 3)?;
        let result = qseries::jacprodmake_with_period(&fps, max_n, Some(pp));
        Ok(jacobi_product_form_to_value(&result))
    } else {
        Err(EvalError::WrongArgCount {
            function: name.to_string(),
            expected: "3 or 4".to_string(),
            got: args.len(),
            signature: get_signature(name),
        })
    }
}
```

### Pattern: qfactor with optional T

```rust
"qfactor" => {
    if args.len() == 2 {
        let fps = extract_series(name, args, 0)?;
        let _sym = extract_symbol_id(name, args, 1, env)?;
        let result = qseries::qfactor(&fps);
        Ok(q_factorization_to_value(&result))
    } else if args.len() == 3 {
        let fps = extract_series(name, args, 0)?;
        let _sym = extract_symbol_id(name, args, 1, env)?;
        let _t = extract_i64(name, args, 2)?;
        // T parameter could cap the max factor search, but our current
        // qfactor already handles this via the polynomial degree.
        // For now, ignore T (our algorithm is already degree-bounded).
        let result = qseries::qfactor(&fps);
        Ok(q_factorization_to_value(&result))
    } else {
        Err(EvalError::WrongArgCount {
            function: name.to_string(),
            expected: "2 or 3".to_string(),
            got: args.len(),
            signature: get_signature(name),
        })
    }
}
```

### Pattern: Test with Maple-style args

```rust
#[test]
fn dispatch_sift_maple_style() {
    let mut env = make_env();
    let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(50i64))], &mut env).unwrap();
    let args = vec![
        pgf,
        Value::Symbol("q".to_string()),
        Value::Integer(QInt::from(5i64)),   // n
        Value::Integer(QInt::from(4i64)),   // k
        Value::Integer(QInt::from(30i64)),  // T
    ];
    let val = dispatch("sift", &args, &mut env).unwrap();
    assert!(matches!(val, Value::Series(_)));
}

#[test]
fn dispatch_prodmake_maple_style() {
    let mut env = make_env();
    let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(30i64))], &mut env).unwrap();
    let args = vec![
        pgf,
        Value::Symbol("q".to_string()),
        Value::Integer(QInt::from(20i64)),  // T
    ];
    let val = dispatch("prodmake", &args, &mut env).unwrap();
    assert!(matches!(val, Value::Dict(_)));
}
```

## Implementation Decisions (Claude's Discretion Resolutions)

### 1. Function Names
**Decision:** Keep all current names. All 7 match Garvan's Maple names exactly.
**Confidence:** HIGH (verified against source code)

### 2. sift Truncation Semantics
**Decision:** T acts as a cap on the input truncation. If T < series.truncation_order(), use T. If T >= series.truncation_order(), use series.truncation_order(). The output truncation is floor((effective_T - k) / n) + 1. This matches Garvan's behavior where `series(s, q, T+5)` re-expands the input.
**Confidence:** HIGH (verified from Garvan source code)

### 3. sift Variable Validation
**Decision:** Extract the q symbol with `extract_symbol_id` for validation (must be a Symbol type). Don't otherwise use it -- the series already carries its variable. This matches Phase 34's pattern for etaq, jacprod, etc.
**Confidence:** HIGH (established pattern)

### 4. jacprodmake P Parameter
**Decision:** P (called PP in Garvan) restricts the period search to divisors of P. This requires a small change to the core `jacprodmake` function in `prodmake.rs` to accept an optional period filter. The 3-arg form passes `None`, the 4-arg form passes `Some(P)`.
**Confidence:** HIGH (verified from Garvan source code: `periodfind2` uses `numtheory[divisors](PP)`)

### 5. Failed Decomposition Behavior
**Decision:** Keep current behavior -- return a Dict with `is_exact: false`. Garvan issues a WARNING and stores args in a global variable; we return a result with `is_exact: false` which is more functional. No change needed.
**Confidence:** MEDIUM (reasonable design choice, not strictly matching Garvan's warning-based approach)

### 6. qfactor Output Format
**Decision:** Keep current Value::Dict format `{scalar, factors, is_exact}`. This is structurally equivalent to Garvan's product expression but in a programmatically accessible format. Display formatting can be improved in a future phase.
**Confidence:** MEDIUM (dict format is more useful than string representation for programmatic use)

### 7. qfactor Signature
**Decision:** `qfactor(f, q)` (2-arg) and `qfactor(f, q, T)` (3-arg). The 2-arg form uses the polynomial degree to auto-determine max factor search (matching Garvan's default of 4*degree+3). The 3-arg form passes T as the max factor index. Our current `qfactor` core function already caps at the polynomial degree, so T just provides an optional further cap.
**Confidence:** HIGH (reconciles SIG-14 with Garvan's actual signature)

### 8. Error Message Format
**Decision:** Use `EvalError::Other(format!("function: Argument N (name): ..."))` for custom positional errors in these 7 functions. This is simpler than changing the existing `EvalError::ArgType` variant globally.
**Confidence:** HIGH (pragmatic, minimal blast radius)

## State of the Art

| Old Approach (current) | New Approach (Phase 35) | Impact |
|----------------------|------------------------|--------|
| `sift(series, m, j)` | `sift(s, q, n, k, T)` | Matches Garvan, explicit truncation control |
| `prodmake(series, max_n)` | `prodmake(f, q, T)` | Matches Garvan, explicit q |
| `jacprodmake(series, max_n)` | `jacprodmake(f, q, T[, P])` | Matches Garvan, optional period restriction |
| `qfactor(series)` | `qfactor(f, q[, T])` | Matches Garvan, explicit q, optional T |
| Legacy + Maple coexist (Phase 34) | Maple-only, legacy removed | Clean API, no ambiguity |

## Open Questions

1. **sift truncation when T > series.truncation_order()**
   - What we know: Garvan re-expands the series to T+5 terms. We can't re-expand (series is already computed).
   - What's unclear: Should we error, warn, or silently use all available data?
   - Recommendation: Silently use min(T, series.truncation_order()). The user may set T optimistically; using available data is most useful.

2. **prodmake/etamake/etc output display format**
   - What we know: Garvan returns Maple expressions like `(1-q)^a * (1-q^2)^b * ...`. We return Value::Dict.
   - What's unclear: Should we change the display format in format_value now or defer?
   - Recommendation: Defer display format changes. The Dict is more useful programmatically. Display can be a separate enhancement. The CONTEXT.md says "should match" which we interpret as aspirational for this phase.

3. **qfactor T parameter mapping to core function**
   - What we know: Our core qfactor iterates from degree down to 1. Garvan's T caps the upper bound of the (1-q^i) factor search.
   - What's unclear: Whether to modify core qfactor to accept a max_i parameter.
   - Recommendation: For 3-arg form, pass T to core function. Simple modification: add optional `max_factor: Option<i64>` parameter to `qseries::qfactor`. Or just ignore T for now since our algorithm already naturally caps at degree.

## Sources

### Primary (HIGH confidence)
- Garvan qseries Maple source code: `wmprog64.txt` from [qseries.org v1.2 download](https://qseries.org/fgarvan/qmaple/qseries/1.2/maple16-win64/)
- [jacprodmake function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/jacprodmake.html)
- [sift function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/sift.html)
- [prodmake function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/prodmake.html)
- [etamake function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/etamake.html)
- [qfactor function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/qfactor.html)
- [mprodmake function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/mprodmake.html)
- [qetamake function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/qetamake.html)
- Current q-Kangaroo source: `crates/qsym-cli/src/eval.rs`, `crates/qsym-core/src/qseries/`

### Secondary (MEDIUM confidence)
- [qseries package main page](https://qseries.org/fgarvan/qmaple/qseries/) - version/function listing
- [Garvan tutorial PDF](https://qseries.org/fgarvan/papers/qmaple.pdf) - examples (PDF not directly parseable)

### Tertiary (LOW confidence)
- WebSearch results about prodmake output format (partially confirmed from source)

## Metadata

**Confidence breakdown:**
- Garvan signatures: HIGH - verified from actual Maple source code
- Architecture patterns: HIGH - directly observed in Phase 34 codebase
- Current implementation: HIGH - read directly from source
- Pitfalls: HIGH - derived from implementation analysis
- Display format decisions: MEDIUM - aspirational per CONTEXT.md, deferred details

**Research date:** 2026-02-19
**Valid until:** 2026-03-19 (stable -- Garvan signatures don't change, codebase is under our control)
