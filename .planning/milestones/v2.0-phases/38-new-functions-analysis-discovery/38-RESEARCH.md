# Phase 38: New Functions - Analysis & Discovery - Research

**Researched:** 2026-02-19
**Domain:** CLI function dispatch in qsym-cli (eval.rs) + qsym-core algorithm additions
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

1. **Signature Corrections (Verified Against Garvan Source)**
   - `checkmult(QS, T)` or `checkmult(QS, T, 'yes')` — no q arg
   - `checkprod(f, M, Q)` — M is max-exponent threshold, Q is truncation order
   - `lqdegree0(qexp)` — 1 arg only
   - `findprod(FL, T, M, Q)` — 4 args, no q arg (replaces/updates old 3-arg form)

2. **checkmult Behavior**
   - 2-arg: print "MULTIPLICATIVE" or "NOT MULTIPLICATIVE" with first failing (m,n). Return 1 or 0.
   - 3-arg with 'yes': print ALL failing (m,n) pairs. Return 1 or 0.
   - Algorithm: check f(mn) = f(m)*f(n) for all coprime pairs m,n with 2 <= m,n <= T/2 and mn <= T.

3. **checkprod Behavior**
   - Silent return only (no printing).
   - Returns `[a, 1]` for nice product, `[a, max_exp]` for not nice, `[[a, c0], -1]` for non-integer divisible.
   - Return type: Value::List.

4. **lqdegree0**
   - FPS only. Returns minimum key in FPS BTreeMap.
   - Nearly identical to existing lqdegree; added for Garvan compatibility.

5. **findprod Behavior**
   - 4-arg (FL, T, M, Q). T is max |coefficient|, M is max product exponent, Q is truncation order.
   - Primitive vector filtering (gcd = 1 over coefficient vector).
   - Returns list of [valuation, coefficient_vector] pairs.
   - No artificial limits.

6. **zqfactor Deferral** — Out of scope.

7. **Output Patterns**
   - checkmult: prints + returns integer.
   - checkprod, lqdegree0, findprod: silent return only.

### Claude's Discretion

None specified.

### Deferred Ideas (OUT OF SCOPE)

- Bivariate (z,q)-series type and zqfactor implementation
- lqdegree0 support for JacobiProduct inputs
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| NEW-05 | `checkmult(QS, T)` and `checkmult(QS, T, 'yes')` — check if q-series coefficients are multiplicative | Pure CLI logic in eval.rs dispatch; uses fps.coeff() to check f(m)*f(n)==f(mn); optional 3rd string arg 'yes' already parsed as Value::String by existing lexer |
| NEW-06 | `checkprod(f, M, Q)` — check if series is a "nice" formal product | Calls existing prodmake() from qsym-core; normalization + max-exponent check implemented in eval.rs or new qsym-core helper |
| NEW-07 | `lqdegree0(qexp)` — lowest q-degree of a single monomial term (1 arg, FPS only) | Thin wrapper over fps.min_order(); nearly identical to existing lqdegree dispatch at line 1955 |
| NEW-09 | `findprod(FL, T, M, Q)` — exhaustive search over integer coefficient vectors | Replaces old 3-arg findprod; adds checkprod logic, primitive-vector GCD filter, and returns [valuation, coeff_vec] pairs; new qsym-core function `findprod_garvan` |
</phase_requirements>

---

## Summary

Phase 38 adds four analysis/discovery functions to the CLI. All four are pure additions within the established function dispatch pattern in `crates/qsym-cli/src/eval.rs`. No new Value variants are needed. The functions build on existing qsym-core infrastructure: `prodmake` (already dispatched), `FormalPowerSeries::coeff()` / `min_order()`, and `FormalPowerSeries::truncation_order()`.

The critical challenge is `findprod` (NEW-09): the existing `qseries::findprod` in `crates/qsym-core/src/qseries/relations.rs` (line 1381) has the wrong 3-arg signature and wrong return type. It must be replaced with a 4-arg `findprod_garvan` that integrates checkprod logic, uses primitive-vector filtering (gcd=1), and returns `Vec<(i64, Vec<i64>)>` (valuation + coefficient vector pairs). The old 3-arg `findprod` CLI dispatch at eval.rs line 2303 must be updated to match the new Garvan signature.

`checkprod` is the linchpin: it is called by `findprod` internally. Implementing checkprod as a standalone helper function (either in eval.rs or as a qsym-core function) before implementing findprod is the correct dependency order per the CONTEXT.md implementation notes.

**Primary recommendation:** Implement in order: (1) `lqdegree0` and `checkmult` (independent), (2) `checkprod` helper, (3) `findprod` using checkprod internally. All CLI dispatch additions follow the identical pattern in eval.rs.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| qsym-core (internal) | current | `prodmake`, `FormalPowerSeries` methods | All existing analysis functions use this |
| rug (via qsym-core) | already in Cargo.toml | GCD computation via `rug::Integer::gcd` | Already used in relations.rs for integer exponent checks |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::collections::BTreeMap | stdlib | FPS coefficient storage | Accessing min/max keys for lqdegree0 and checkprod valuation |
| std::cmp | stdlib | abs() for max-exponent checks | checkprod's max-exponent threshold |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Implementing checkprod in eval.rs | Implementing in qsym-core | eval.rs approach is simpler for functions that are purely CLI-level wrappers; but since findprod calls checkprod repeatedly in a loop, a qsym-core helper avoids cloning overhead. Either works; eval.rs helper function is fine since findprod will also live in eval.rs dispatch. |
| New `findprod_garvan` function | Modifying existing `findprod` in-place | Adding a new named function preserves the old 3-arg API in case it's used in tests; safer approach is to rename old to `findprod_old` or remove it since the dispatch is updated anyway |

**Installation:** No new dependencies required.

---

## Architecture Patterns

### Recommended File Structure

The established pattern is: all new function dispatch in `eval.rs`, supplementary algorithms in `qsym-core/src/qseries/relations.rs` (or utilities.rs), and public re-exports in `qseries/mod.rs`.

```
crates/qsym-cli/src/eval.rs        -- add 4 dispatch arms + 4 get_signature() entries + 4 ALL_FUNCTION_NAMES entries
crates/qsym-cli/src/help.rs        -- add 4 FuncHelp entries + update general_help() listing
crates/qsym-core/src/qseries/
  relations.rs                      -- replace findprod with new 4-arg findprod_garvan
  utilities.rs                      -- add lqdegree0 (or lqdegree alias)
  mod.rs                            -- update pub use lines
crates/qsym-cli/tests/cli_integration.rs  -- new integration tests
```

### Pattern 1: Standard Function Dispatch (lqdegree0 and checkmult)

**What:** Add a named match arm in `eval.rs`'s `dispatch()` function. Extract args using existing helpers, call business logic, return Value.

**When to use:** For all new functions.

**Example (lqdegree0 — mirrors existing lqdegree at line 1955):**
```rust
"lqdegree0" => {
    // Garvan: lqdegree0(qexp) -- lowest q-degree of a series
    expect_args(name, args, 1)?;
    let fps = extract_series(name, args, 0)?;
    match fps.min_order() {
        Some(d) => Ok(Value::Integer(QInt::from(d))),
        None => Ok(Value::None),
    }
}
```

**Example (checkmult — 2 or 3 args, prints + returns 1/0):**
```rust
"checkmult" => {
    // Garvan: checkmult(QS, T) or checkmult(QS, T, 'yes')
    expect_args_range(name, args, 2, 3)?;
    let fps = extract_series(name, args, 0)?;
    let t = extract_i64(name, args, 1)?;
    let print_all = args.len() == 3 && matches!(&args[2], Value::String(s) if s == "yes");

    // Algorithm: check f(mn) = f(m)*f(n) for coprime pairs
    let mut failures: Vec<(i64, i64)> = Vec::new();
    let half_t = t / 2;
    'outer: for m in 2..=half_t {
        for n in m..=half_t {
            if m * n > t {
                break;
            }
            // igcd(m, n) == 1 check
            if gcd(m, n) != 1 {
                continue;
            }
            let fm = fps.coeff(m);
            let fn_ = fps.coeff(n);
            let fmn = fps.coeff(m * n);
            if fm.clone() * fn_ != fmn {
                failures.push((m, n));
                if !print_all {
                    break 'outer;
                }
            }
        }
    }

    if failures.is_empty() {
        println!("MULTIPLICATIVE");
        Ok(Value::Integer(QInt::from(1i64)))
    } else {
        for (m, n) in &failures {
            println!("NOT MULTIPLICATIVE at ({}, {})", m, n);
        }
        Ok(Value::Integer(QInt::from(0i64)))
    }
}
```

### Pattern 2: checkprod Helper Function

**What:** Implement `checkprod` logic as a standalone function in eval.rs (private to the module) since it is called both from the `"checkprod"` dispatch arm and from the `"findprod"` dispatch arm.

**Algorithm:**
1. Find min_order `a` of `f` (the valuation/leading q-power).
2. Divide `f` by its leading coefficient `c0`. If `c0` denom != 1, return `[[a, c0], -1]`.
3. Run `qseries::prodmake(&normalized, Q)` where Q is the truncation order arg.
4. Find `max_exp = max(exponents.values().map(|x| x.numer().abs()))`.
5. If `max_exp < M`, return `[a, 1]`. Otherwise return `[a, max_exp]`.

```rust
// Private helper called from both checkprod and findprod dispatch
fn checkprod_impl(fps: &FormalPowerSeries, m_threshold: i64, q_order: i64)
    -> Value
{
    // Step 1: Find valuation a
    let a = fps.min_order().unwrap_or(0);

    // Step 2: Get leading coefficient c0
    let c0 = fps.coeff(a);
    let one = rug::Integer::from(1u32);

    // Check integer-divisibility
    if c0.denom() != &one {
        // Non-integer leading coefficient
        return Value::List(vec![
            Value::List(vec![
                Value::Integer(QInt::from(a)),
                Value::Rational(c0),
            ]),
            Value::Integer(QInt::from(-1i64)),
        ]);
    }

    // Step 3: Normalize and run prodmake
    // prodmake internally normalizes (strips q^a and divides by c0)
    let product = qseries::prodmake(fps, q_order);

    // Step 4: Find max |exponent|
    let max_exp = product.exponents.values()
        .map(|rat| {
            // exponent should be integer (denominator = 1)
            rat.numer().to_i64().unwrap_or(i64::MAX).abs()
        })
        .max()
        .unwrap_or(0);

    // Step 5: Return result
    if max_exp < m_threshold {
        Value::List(vec![
            Value::Integer(QInt::from(a)),
            Value::Integer(QInt::from(1i64)),
        ])
    } else {
        Value::List(vec![
            Value::Integer(QInt::from(a)),
            Value::Integer(QInt::from(max_exp)),
        ])
    }
}
```

**Dispatch arm:**
```rust
"checkprod" => {
    // Garvan: checkprod(f, M, Q)
    expect_args(name, args, 3)?;
    let fps = extract_series(name, args, 0)?;
    let m_threshold = extract_i64(name, args, 1)?;
    let q_order = extract_i64(name, args, 2)?;
    Ok(checkprod_impl(&fps, m_threshold, q_order))
}
```

### Pattern 3: findprod (4-arg Garvan version) Replacing Old 3-arg

**What:** Replace the existing 3-arg `findprod` dispatch at line 2303 with the new 4-arg Garvan version. Also replace `qseries::findprod` in relations.rs with a new `findprod_garvan` that:
1. Iterates over coefficient vectors with entries in `[-T, T]`.
2. Skips vectors where `gcd(|c_1|, ..., |c_k|) > 1` (primitive vector filter).
3. Forms the linear combination.
4. Calls checkprod logic with threshold M and truncation Q.
5. If result is `[a, 1]` (nice product), records `[a, c_1, ..., c_k]`.

**Return value:** `Value::List` of `Value::List([Value::Integer(a), Value::Integer(c1), ..., Value::Integer(ck)])`.

```rust
"findprod" => {
    // Garvan: findprod(FL, T, M, Q)
    expect_args(name, args, 4)?;
    let series_list = extract_series_list(name, args, 0)?;
    let max_coeff = extract_i64(name, args, 1)?;
    let m_threshold = extract_i64(name, args, 2)?;
    let q_order = extract_i64(name, args, 3)?;

    let k = series_list.len();
    let mut results: Vec<Value> = Vec::new();

    // Iterate coefficient vectors
    let mut coeffs = vec![-max_coeff; k];
    loop {
        if coeffs.iter().any(|&c| c != 0) {
            // Primitive vector check: gcd of absolute values == 1
            let g = coeffs.iter().fold(0i64, |acc, &c| gcd(acc.abs(), c.abs()));
            if g <= 1 {
                // Form linear combination
                let combo = compute_linear_combo(&series_list, &coeffs, q_order);
                if !combo.is_zero() {
                    let result = checkprod_impl(&combo, m_threshold, q_order);
                    // Nice product if result == [a, 1]
                    if is_nice_checkprod(&result) {
                        let a = extract_valuation(&result);
                        let mut row = vec![Value::Integer(QInt::from(a))];
                        row.extend(coeffs.iter().map(|&c| Value::Integer(QInt::from(c))));
                        results.push(Value::List(row));
                    }
                }
            }
        }
        if !increment_coeffs(&mut coeffs, max_coeff) {
            break;
        }
    }
    Ok(Value::List(results))
}
```

### Pattern 4: GCD Helper for Primitive Vector Check

The `gcd` function is private to `relations.rs` and `eta.rs`. For checkmult and findprod, a simple Euclidean GCD can be inlined in eval.rs or a small private helper added:

```rust
// In eval.rs (private)
fn gcd(a: i64, b: i64) -> i64 {
    let (mut x, mut y) = (a.abs(), b.abs());
    while y != 0 {
        let tmp = y;
        y = x % y;
        x = tmp;
    }
    x
}
```

### Anti-Patterns to Avoid

- **Calling qseries::findprod (old 3-arg) from the new 4-arg dispatch:** The old function does not support primitive-vector filtering or the [valuation, coeff_vec] return format. Do not call it.
- **Implementing checkprod inline inside findprod:** checkprod must also be available as a standalone CLI function. Extract as a shared helper.
- **Using i64::MAX for non-integer exponent detection:** Use `QRat::denom() == 1` check from prodmake results (same pattern as `has_nice_product_form` in relations.rs line 1453).
- **Forgetting the gcd=1 primitive vector filter:** Without it, findprod returns redundant multiples of the same relation.
- **Assuming prodmake handles the truncation:** prodmake caps `max_n` at `f.truncation_order() - 1`. If the user passes Q > f's truncation order, prodmake silently caps it. This is fine behavior.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Log-derivative product decomposition | Custom prodmake | `qseries::prodmake` (prodmake.rs line 134) | Already implemented with Mobius inversion; used by etamake, jacprodmake, mprodmake |
| FPS coefficient access | Custom indexing | `fps.coeff(k)` (series/mod.rs line 90) | Returns QRat::zero() for missing keys automatically |
| FPS leading coefficient | Custom scan | `fps.min_order()` + `fps.coeff(a)` | min_order() wraps BTreeMap::keys().next() |
| Odometer iteration | Custom nested loops | `increment_coeffs` pattern (already in relations.rs line 1459) | Copy this exact pattern |
| Integer check on QRat | Custom comparison | `rat.denom() == &rug::Integer::from(1)` | Same pattern as has_nice_product_form line 1453 |
| Truncation-order management for linear combos | Custom | `arithmetic::add` + `arithmetic::scalar_mul` + min truncation order | Used in `compute_linear_combination` in relations.rs line 1422 |

**Key insight:** The entire algorithmic substrate for checkprod and findprod already exists in prodmake.rs and relations.rs. These functions are coordinators, not algorithms.

---

## Common Pitfalls

### Pitfall 1: Coprimality Check in checkmult
**What goes wrong:** Looping m from 2 to T and n from 2 to T/m yields duplicate pairs (m,n) and (n,m). Garvan's Maple loops n from m to T/m with the igcd check.
**Why it happens:** Naive double loop produces each pair twice.
**How to avoid:** Loop `n` from `m` to `half_t` (not from 2), and check `m*n <= T` as loop guard. Use Garvan's loop structure: `for m in 2..=half_t { for n in m..=half_t { if m*n > t { break; } if gcd(m,n)==1 { ... } } }`.
**Warning signs:** If checkmult prints the same (m,n) pair twice or produces 0 when series is multiplicative.

### Pitfall 2: checkprod Non-Integer Leading Coefficient Case
**What goes wrong:** If `fps.coeff(a)` has denominator > 1, the Garvan behavior is to return `[[a, c0], -1]`. If this special case is forgotten, checkprod crashes or returns wrong value on fractional-coefficient series.
**Why it happens:** prodmake normalizes away the leading coefficient internally, but checkprod must report whether the leading coefficient was integer before prodmake strips it.
**How to avoid:** Check `c0.denom() == &rug::Integer::from(1)` before calling prodmake. Return `[[a, c0], -1]` if denom != 1.
**Warning signs:** checkprod returning wrong type on series like `(1/2) + q + ...`.

### Pitfall 3: findprod Old vs. New Signature Conflict
**What goes wrong:** The existing `findprod` dispatch at eval.rs line 2303 takes 3 args. If the new 4-arg version is added as a separate arm instead of replacing it, the old 3-arg arm still catches calls.
**Why it happens:** Rust match arms are tried in order; having two `"findprod"` arms means the first always wins.
**How to avoid:** Replace the entire existing `"findprod" => { ... }` block at lines 2303-2318. Do not add a new arm alongside it.
**Warning signs:** `findprod(fl, 2, 5, 30)` reports "wrong number of arguments: expected 3, got 4".

### Pitfall 4: findprod Primitive Vector Filter Edge Cases
**What goes wrong:** The `gcd(0, x) = x` convention means the all-zero vector (skipped anyway) and vectors with some zero entries are handled correctly, but the fold starting at `gcd(0, |c1|)` must be verified. `gcd(0, 0) = 0` — only skip when `g == 0` (zero vector) or `g > 1` (non-primitive).
**Why it happens:** Edge case in GCD computation with zero entries.
**How to avoid:** Use `g <= 1` as the pass condition (not `g == 1`), since zero vector has `g = 0` and is already skipped by the `coeffs.iter().any(|&c| c != 0)` guard. Check: if all zeros → skip; if gcd of abs values is 1 → proceed; if gcd > 1 → skip.
**Warning signs:** Missing valid coefficient vectors from findprod output.

### Pitfall 5: checkprod max_exp Extraction from QRat
**What goes wrong:** prodmake returns `BTreeMap<i64, QRat>` exponents. The exponents should be integers (denom=1) for valid series, but if prodmake returns a non-integer exponent, converting to i64 via `to_f64() as i64` (as done in mprodmake at prodmake.rs line 411) truncates incorrectly.
**Why it happens:** Series that don't have a nice product representation yield non-integer exponents.
**How to avoid:** For the max_exp check, use `rat.numer().to_i64().unwrap_or(i64::MAX).abs()` directly (the numerator gives the "integer part" when denom=1, which is the valid case; when denom != 1 the abs value of the numerator is >= M anyway). Alternatively, check for non-integer exponents separately and handle.
**Warning signs:** checkprod returning `[a, 1]` (nice) for series that should not be nice products.

### Pitfall 6: get_signature and ALL_FUNCTION_NAMES Not Updated
**What goes wrong:** New functions aren't tab-completed in the REPL, and `help checkmult` returns a generic error.
**Why it happens:** `get_signature()` (line 3439) and `ALL_FUNCTION_NAMES` (line 3565) must be manually updated.
**How to avoid:** After adding each dispatch arm, immediately add its entry to `get_signature()` and `ALL_FUNCTION_NAMES`. The count test at line 5890 checks `count >= 75`; update it to `count >= 79` (4 new functions added).
**Warning signs:** Tab completion misses the new functions; `help findprod` produces "unknown function" instead of the help text.

### Pitfall 7: Existing findprod Tests Failing
**What goes wrong:** The existing unit test at eval.rs (if any) for the 3-arg findprod will fail after the signature change.
**Why it happens:** The old `findprod([s1,s2], max_coeff, max_exp)` API is replaced by `findprod(fl, T, M, Q)`.
**How to avoid:** Search for any existing tests of `findprod` in eval.rs and cli_integration.rs. Update them to use the new 4-arg signature.
**Warning signs:** `dispatch_findprod_*` tests fail after the change.

---

## Code Examples

Verified patterns from the existing codebase:

### Accessing FPS Coefficients (for checkmult)
```rust
// Source: crates/qsym-core/src/series/mod.rs line 90
// Returns QRat::zero() for any key not in the BTreeMap
let fm: QRat = fps.coeff(m);
let fn_: QRat = fps.coeff(n);
let fmn: QRat = fps.coeff(m * n);
// Comparison: QRat implements PartialEq
let is_multiplicative = (fm.clone() * fn_) == fmn;
```

### Getting FPS Minimum Order (for lqdegree0)
```rust
// Source: crates/qsym-core/src/series/mod.rs line 122
// lqdegree already does exactly this:
// Source: crates/qsym-core/src/qseries/utilities.rs line 83
pub fn lqdegree(f: &FormalPowerSeries) -> Option<i64> {
    f.min_order()
}
// lqdegree0 is identical — just add an alias in eval.rs dispatch
```

### Running prodmake (for checkprod)
```rust
// Source: crates/qsym-core/src/qseries/prodmake.rs line 134
// prodmake normalizes internally: strips q^a prefix, divides by leading coeff
let product: InfiniteProductForm = qseries::prodmake(&fps, q_order);
// product.exponents: BTreeMap<i64, QRat>
// Check integer exponent: (same pattern as relations.rs line 1453)
let one = rug::Integer::from(1u32);
let all_integer = product.exponents.values().all(|exp| exp.denom() == &one);
```

### Optional String Argument Pattern ('yes' for checkmult)
```rust
// Source: lexer.rs — single-quoted strings become Token::StringLit → Value::String
// Source: eval.rs line 978 — AstNode::StringLit(s) => Ok(Value::String(s.clone()))
// Detection pattern:
let print_all = args.len() == 3 && matches!(&args[2], Value::String(s) if s == "yes");
```

### Odometer Coefficient Iteration (for findprod)
```rust
// Source: crates/qsym-core/src/qseries/relations.rs line 1459
fn increment_coeffs(coeffs: &mut [i64], max_coeff: i64) -> bool {
    for c in coeffs.iter_mut().rev() {
        *c += 1;
        if *c <= max_coeff {
            return true;
        }
        *c = -max_coeff;
    }
    false
}
```

### Linear Combination (for findprod)
```rust
// Source: crates/qsym-core/src/qseries/relations.rs line 1422
// compute_linear_combination takes &[&FormalPowerSeries] and &[i64] coefficients
// result truncation = min truncation of all input series
let trunc = series_list.iter().map(|s| s.truncation_order()).min().unwrap();
// Use arithmetic::add + arithmetic::scalar_mul
```

### expect_args_range for Optional Arguments (checkmult)
```rust
// Source: eval.rs line 199
// Used for jacprodmake (3 or 4 args), findcong (2 to 4 args), etc.
expect_args_range(name, args, 2, 3)?;
// Then check args.len() to branch
```

### Printing + Returning 1 or 0 (checkmult)
```rust
// Pattern from findcong (eval.rs line 2336-2348):
// Print diagnostic messages then return the data structure
println!("MULTIPLICATIVE");
Ok(Value::Integer(QInt::from(1i64)))
// or:
println!("NOT MULTIPLICATIVE at ({}, {})", m, n);
Ok(Value::Integer(QInt::from(0i64)))
```

### FuncHelp Entry Pattern
```rust
// Source: help.rs line 121-132, example from lqdegree at line 285
FuncHelp {
    name: "checkmult",
    signature: "checkmult(QS, T) or checkmult(QS, T, 'yes')",
    description: "Test if q-series coefficients are multiplicative: f(mn) = f(m)*f(n) for gcd(m,n)=1.\n  Checks all coprime pairs m,n with 2<=m,n<=T/2 and m*n<=T.\n  Optional 'yes' arg prints ALL failing pairs instead of stopping at first.",
    example: "q> f := partition_gf(100)\nq> checkmult(f, 50)",
    example_output: "NOT MULTIPLICATIVE at (2, 3)\n0",
},
```

---

## State of the Art

| Old Approach | Current Approach | Notes |
|--------------|------------------|-------|
| 3-arg `findprod(FL, max_coeff, max_exp)` at eval.rs line 2303 | 4-arg `findprod(FL, T, M, Q)` Garvan-compatible | Replace the old dispatch arm entirely |
| Old `findprod` in relations.rs (line 1381) returns `Vec<Vec<i64>>` | New version returns `Vec<(i64, Vec<i64>)>` — (valuation, coeffs) | Rename old to `findprod_old` or remove; add new `findprod_garvan` |
| `lqdegree` only (no `lqdegree0`) | Both `lqdegree` and `lqdegree0` | lqdegree0 is identical for FPS; add as a second dispatch arm |

**Deprecated/outdated:**
- Old `findprod` (3-arg, returns just coefficient vectors, no valuation): replaced by Garvan-compatible 4-arg version.
- The comment in help.rs line 401-406 describing findprod as "exponent vector" search: update to match actual Garvan semantics.

---

## Open Questions

1. **Should `checkprod` helper live in eval.rs or qsym-core?**
   - What we know: findprod calls it in a tight loop; qsym-core is more testable independently.
   - What's unclear: Whether the extra clone overhead of passing through Value is acceptable.
   - Recommendation: Implement as a private `fn checkprod_impl(fps: &FormalPowerSeries, m_threshold: i64, q_order: i64) -> Value` in eval.rs. This avoids a new qsym-core public API while keeping the dispatch clean.

2. **What is the exact Garvan format for checkmult failure output?**
   - What we know: Garvan prints "NOT MULTIPLICATIVE" with the failing (m,n) pair.
   - What's unclear: Exact formatting ("NOT MULTIPLICATIVE at (2,3)" vs "checkmult: failure at m=2,n=3").
   - Recommendation: Match Garvan's output pattern: `println!("NOT MULTIPLICATIVE at ({}, {})", m, n);` since Garvan prints "m =..., n =..." style; match Garvan's Maple output convention.

3. **Does `findprod` need to deduplicate results that differ only by sign of all coefficients?**
   - What we know: Garvan collects all primitive vectors. (c1,...,ck) and (-c1,...,-ck) are both primitive.
   - What's unclear: Whether both are returned or just one.
   - Recommendation: Return both — let the user decide. Garvan's Maple does not deduplicate.

---

## Sources

### Primary (HIGH confidence)
- Direct source code inspection: `crates/qsym-core/src/qseries/prodmake.rs` — prodmake API, InfiniteProductForm structure
- Direct source code inspection: `crates/qsym-core/src/qseries/relations.rs` lines 1358-1468 — existing findprod, compute_linear_combination, increment_coeffs, has_nice_product_form
- Direct source code inspection: `crates/qsym-cli/src/eval.rs` lines 1955-1963 (lqdegree), 2303-2318 (old findprod), 3439-3528 (get_signature), 3565-3610 (ALL_FUNCTION_NAMES)
- Direct source code inspection: `crates/qsym-core/src/series/mod.rs` — coeff(), min_order(), truncation_order(), is_zero()
- Direct source code inspection: `crates/qsym-core/src/qseries/utilities.rs` lines 83-85 — lqdegree implementation
- CONTEXT.md decisions (locked) — verified against Garvan Maple source
- `crates/qsym-cli/src/lexer.rs` lines 72-93 — single-quoted string lexing produces Token::StringLit → Value::String

### Secondary (MEDIUM confidence)
- Phase 33-37 pattern analysis from eval.rs integration test file and dispatch arms

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies verified from source code inspection
- Architecture: HIGH — established patterns confirmed by examining 6+ analogous function dispatch implementations
- Pitfalls: HIGH — pitfalls derived from reading actual Rust code that exhibits the problem or its solution
- Algorithm correctness: HIGH for lqdegree0/checkmult (trivial); MEDIUM for checkprod/findprod (Garvan algorithm details may have edge cases not yet exercised)

**Research date:** 2026-02-19
**Valid until:** 2026-03-19 (codebase-internal, stable)
