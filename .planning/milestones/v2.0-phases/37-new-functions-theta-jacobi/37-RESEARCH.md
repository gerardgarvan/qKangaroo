# Phase 37: New Functions - Theta & Jacobi - Research

**Researched:** 2026-02-19
**Domain:** New CLI value type (JacobiProduct) + 4 new functions (theta, jac2prod, jac2series, qs2jaccombo) + arithmetic dispatch extensions
**Confidence:** HIGH

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Jacobi product input format
- `JAC(a,b)` is a function call that returns a `JacobiProduct` value representing (q^a; q^b)_inf
- JacobiProduct is a standalone value type: assignable to variables, printable, combinable with `*`, `/`, and `^`
- Full algebra supported: `JAC(1,5) * JAC(2,5)`, `JAC(1,5) / JAC(2,5)`, `JAC(1,5)^3` all work
- Integer exponents (positive and negative) supported for power notation
- JP expressions are products/quotients of JAC factors with integer exponents
- `jac2prod` and `jac2series` strictly require a JacobiProduct value -- passing a non-JP value errors with "expected Jacobi product expression (use JAC(a,b))"

#### theta(z, q, T) behavior
- `theta(z, q, T)` computes sum(z^i * q^(i^2), i=-T..T)
- If z is a numeric value (integer, rational): substitute and return univariate q-series
- If z is a q-monomial (e.g., q^2): auto-substitute z=q^k and return univariate q-series (e.g., sum(q^(2i + i^2)))
- If z is a bare symbol (unassigned): print a warning message that z must be numeric or a q-monomial, do NOT error -- just warn
- General form only -- classical theta2/3/4 already exist as separate functions, no variants needed
- No artificial T limit -- user chose T, user gets the result

#### Conversion output formats
- All three conversion functions both **print** human-readable output AND **return** the value (consistent with find* functions from Phase 36)
- `jac2prod(JP, q, T)`: print product notation string like `(1-q)(1-q^2)(1+q^3)...`, return the FPS value
- `jac2series(JP, q, T)`: display behavior at Claude's discretion (standard series display likely best)
- `qs2jaccombo(f, q, T)`: print JAC expression formula like `2*JAC(1,5)*JAC(2,5) + 3*JAC(3,5)`

#### Error handling
- `JAC(a,b)` validates: b must be a positive integer, a must be an integer. Error with clear message on invalid args
- `qs2jaccombo` when no decomposition found: print "No Jacobi product decomposition found", return input series unchanged
- `jac2prod`/`jac2series` require strict JacobiProduct input -- clear error for wrong type

### Claude's Discretion
- Exact `jac2series` display format (standard series display vs. something specialized)
- Internal representation of JacobiProduct (Vec of (a, b, exponent) factors, or similar)
- Algorithm choice for qs2jaccombo decomposition
- How JacobiProduct values display when printed standalone (e.g., "JAC(1,5)*JAC(2,5)^(-1)")

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| NEW-01 | `theta(z, q, T)` -- general theta function returning sum(z^i * q^(i^2), i=-T..T) | Direct summation algorithm using FPS arithmetic; z-handling via extract_monomial_from_arg pattern; see Architecture Pattern 1 |
| NEW-02 | `jac2prod(JP, q, T)` -- convert Jacobi product expression to q-product form | Expand JacobiProduct to explicit FPS using etaq(); print finite product notation; see Architecture Pattern 3 |
| NEW-03 | `jac2series(JP, q, T)` -- convert Jacobi product expression to q-series | Same expansion as jac2prod but display as standard series; see Architecture Pattern 4 |
| NEW-04 | `qs2jaccombo(f, q, T)` -- convert sum of q-series to sum of jacprods | Use jacprodmake decomposition followed by findlincombo over JAC basis; see Architecture Pattern 5 |

</phase_requirements>

## Summary

This phase adds a new `Value::JacobiProduct` variant to the CLI evaluator's Value enum and implements four new functions that operate on Jacobi product expressions. The central design challenge is creating a first-class JacobiProduct value type that supports algebraic operations (multiply, divide, power) through the existing `eval_mul`, `eval_div`, `eval_pow` dispatch, then providing conversion functions between this symbolic representation and concrete q-series.

The implementation naturally splits into two layers: (1) the JacobiProduct type and its algebraic operations in eval.rs, including the `JAC(a,b)` constructor function; and (2) four function dispatch entries (`theta`, `jac2prod`, `jac2series`, `qs2jaccombo`) that use existing qsym-core infrastructure (etaq, arithmetic, jacprodmake, findlincombo).

The existing codebase provides all necessary building blocks: `etaq(a, b, variable, T)` computes (q^a; q^b)_inf as an FPS, `arithmetic::{mul, invert, add, scalar_mul}` handle series algebra, `jacprodmake` decomposes a series into JAC factors, and `findlincombo` solves the linear algebra for expressing a target as a linear combination of basis series. No new qsym-core functions are needed.

**Primary recommendation:** Add `Value::JacobiProduct(Vec<(i64, i64, i64)>)` to represent products of (q^a;q^b)_inf^exp factors, implement arithmetic via JacobiProduct-specific match arms in eval_mul/eval_div/eval_pow, then implement the four functions entirely in eval.rs dispatch using existing core library routines.

## Standard Stack

### Core (existing, no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| qsym-core etaq | current | Compute (q^a;q^b)_inf as FPS | Already implements the exact product JAC(a,b) needs |
| qsym-core arithmetic | current | Series mul, invert, add, scalar_mul, negate | All JacobiProduct-to-series conversions are products of etaq results |
| qsym-core jacprodmake | current | Decompose FPS into JAC(a,b) factors | Directly provides the JAC decomposition qs2jaccombo needs |
| qsym-core findlincombo | current | Find linear combination over basis | Enables qs2jaccombo to express target as sum of JAC products |
| qsym-core FormalPowerSeries | current | Sparse BTreeMap<i64, QRat> series | theta() builds its result as an FPS |

### Supporting (existing patterns)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| format_value (format.rs) | current | Display JacobiProduct values | Needs new match arm for Value::JacobiProduct |
| format_latex (format.rs) | current | LaTeX rendering | Needs new match arm for JacobiProduct |
| help.rs | current | Per-function help entries | Need 5 new entries (JAC, theta, jac2prod, jac2series, qs2jaccombo) |

### No Alternatives Needed
This is purely internal CLI feature work. All building blocks exist in qsym-core. No external dependencies needed.

## Architecture Patterns

### Critical Distinction: JAC(a,b) vs existing jacprod(a,b)

**IMPORTANT**: The existing `jacprod(a, b, q, T)` function computes the Jacobi **triple** product:
```
jacprod(a,b) = (q^a; q^b)_inf * (q^{b-a}; q^b)_inf * (q^b; q^b)_inf
```

The NEW `JAC(a,b)` constructor represents a **single** q-Pochhammer factor:
```
JAC(a,b) = (q^a; q^b)_inf = prod_{k>=0} (1 - q^{a + b*k})
```

This is computed by `etaq(a, b, variable, T)` -- NOT by the existing `jacprod()`. The `etaq` function already computes exactly `prod_{k>=0}(1 - q^{b + t*k})`, so `JAC(a,b) = etaq(a, b, sym, T)`.

### Recommended Project Structure (changes to existing files)
```
crates/qsym-cli/src/
  eval.rs       # Value::JacobiProduct variant, JAC/theta/jac2prod/jac2series/qs2jaccombo dispatch,
                # arithmetic extensions in eval_mul/eval_div/eval_pow
  format.rs     # Display for JacobiProduct values
  help.rs       # Help entries for 5 new functions
```

No changes needed to qsym-core -- all building blocks exist.

### Pattern 1: Value::JacobiProduct Internal Representation

**What:** A new Value variant representing a product of (q^a;q^b)_inf factors with integer exponents.
**Recommendation:** Use `Vec<(i64, i64, i64)>` where each tuple is `(a, b, exponent)`.
**Canonical form:** Sort by (b, a) to ensure deterministic display and comparison. Merge factors with same (a,b) by summing exponents. Remove factors with exponent 0.

```rust
// In Value enum:
/// Jacobi product expression: product of (q^a;q^b)_inf^exp factors.
/// Each triple is (a, b, exponent) where JAC(a,b) = (q^a;q^b)_inf.
/// Maintained in sorted canonical form: sorted by (b, a), merged, no zero exponents.
JacobiProduct(Vec<(i64, i64, i64)>),

// Constructor helper:
fn normalize_jacobi_product(mut factors: Vec<(i64, i64, i64)>) -> Vec<(i64, i64, i64)> {
    // Sort by (b, a) for canonical ordering
    factors.sort_by_key(|&(a, b, _)| (b, a));
    // Merge factors with same (a, b)
    let mut merged: Vec<(i64, i64, i64)> = Vec::new();
    for (a, b, exp) in factors {
        if let Some(last) = merged.last_mut() {
            if last.0 == a && last.1 == b {
                last.2 += exp;
                continue;
            }
        }
        merged.push((a, b, exp));
    }
    // Remove zero exponents
    merged.retain(|&(_, _, exp)| exp != 0);
    merged
}
```

### Pattern 2: JacobiProduct Arithmetic in eval_mul/eval_div/eval_pow

**What:** Extend the existing arithmetic dispatch to handle JacobiProduct operands.
**When to use:** When users write `JAC(1,5) * JAC(2,5)`, `JP / JAC(3,5)`, `JP^3`.

```rust
// In eval_mul:
(Value::JacobiProduct(a), Value::JacobiProduct(b)) => {
    let mut combined = a.clone();
    combined.extend_from_slice(b);
    Ok(Value::JacobiProduct(normalize_jacobi_product(combined)))
}

// In eval_div:
(Value::JacobiProduct(a), Value::JacobiProduct(b)) => {
    let mut combined = a.clone();
    for &(a_val, b_val, exp) in b {
        combined.push((a_val, b_val, -exp));
    }
    Ok(Value::JacobiProduct(normalize_jacobi_product(combined)))
}

// In eval_pow:
(Value::JacobiProduct(factors), Value::Integer(n)) => {
    let exp = n.0.to_i64().ok_or_else(|| EvalError::Other("exponent too large".into()))?;
    let scaled: Vec<_> = factors.iter().map(|&(a, b, e)| (a, b, e * exp)).collect();
    Ok(Value::JacobiProduct(normalize_jacobi_product(scaled)))
}
```

### Pattern 3: JacobiProduct to FPS Expansion

**What:** Convert a JacobiProduct value to a FormalPowerSeries by expanding each factor.
**Used by:** `jac2prod` and `jac2series` functions.

```rust
fn jacobi_product_to_fps(
    factors: &[(i64, i64, i64)],
    sym: SymbolId,
    order: i64,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::one(sym, order);
    for &(a, b, exp) in factors {
        // (q^a; q^b)_inf = etaq(a, b, sym, order)
        let factor_fps = qseries::etaq(a, b, sym, order);
        if exp > 0 {
            for _ in 0..exp {
                result = arithmetic::mul(&result, &factor_fps);
            }
        } else if exp < 0 {
            let inv = arithmetic::invert(&factor_fps);
            for _ in 0..(-exp) {
                result = arithmetic::mul(&result, &inv);
            }
        }
    }
    result
}
```

### Pattern 4: theta(z, q, T) Implementation

**What:** Direct summation of sum(z^i * q^(i^2), i=-T..T).
**z handling:** Three cases based on Value type of first argument.

```rust
// Case 1: z is numeric (Integer or Rational) -> substitute z=c, return sum(c^i * q^(i^2))
// Case 2: z is a q-monomial (Series with one term c*q^k) -> substitute z=c*q^k, return sum(c^i * q^(k*i + i^2))
// Case 3: z is a bare Symbol -> print warning, return Value::None or similar

// For cases 1 and 2:
fn compute_theta_series(
    z_coeff: &QRat,   // coefficient of z-monomial (or the numeric value)
    z_power: i64,      // power of q in z-monomial (0 for pure numeric)
    sym: SymbolId,
    t_range: i64,
    truncation_order: i64,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(sym, truncation_order);
    for i in -t_range..=t_range {
        // exponent of q: z_power * i + i * i
        let q_exp = z_power * i + i * i;
        if q_exp >= truncation_order {
            continue;
        }
        // coefficient: z_coeff^i
        let coeff = qrat_pow(z_coeff, i);
        result.set_coeff(q_exp, result.coeff_or_zero(q_exp) + coeff);
    }
    result
}
```

**Note on negative q exponents:** When z_power * i + i^2 is negative, the term has a negative q-exponent. The FPS supports negative exponents (BTreeMap<i64, QRat>), so this works naturally. However, theta series typically stay in non-negative territory for reasonable inputs.

**Note on coeff_or_zero:** The existing `coeff()` panics if k >= truncation_order but works for all k < truncation_order. For accumulating into existing coefficients, directly manipulate the BTreeMap or use a helper.

### Pattern 5: qs2jaccombo Algorithm

**What:** Decompose a q-series f into a linear combination of Jacobi products.
**Algorithm (two-phase approach):**

Phase A -- Generate candidate JAC basis:
1. Run `jacprodmake(f, T)` to get the JacobiProductForm with factors (a,b)->exp
2. From the factors, extract all (a,b) pairs that appear
3. For each (a,b) pair, compute the FPS via `etaq(a, b, sym, T)`
4. Build a basis of individual JAC(a,b) series

Phase B -- Find linear combination:
1. Use `findlincombo(f, basis, topshift)` to express f as sum of basis elements
2. If found, format and print the result as `c1*JAC(a1,b1) + c2*JAC(a2,b2) + ...`
3. Return the String representation

**Alternative simpler approach:** Use `jacprodmake` directly -- it already returns `JacobiProductForm { factors: BTreeMap<(i64,i64), i64>, scalar, is_exact }`. If `is_exact`, the series IS a single Jacobi product (not a sum). For sums, we need the linear combination approach.

**Practical note:** Garvan's `qs2jaccombo` likely uses a similar approach: expand each candidate JAC product to a series, then use linear algebra to find coefficients. The existing `findlincombo` is the right tool.

**Detailed algorithm:**
1. First try direct `jacprodmake(f, T)` -- if `is_exact`, return that directly as a single product
2. If not exact, generate candidate JAC products by trying small (a,b) values systematically (e.g., all b=2..max_b, a=1..b-1)
3. Expand each candidate to FPS via `etaq`
4. Use `findlincombo(f, &candidates, topshift)` to find the decomposition
5. The topshift can be derived from T (e.g., T/2 or similar)

### Pattern 6: Print-and-Return (from Phase 36)

**What:** Functions both print human-readable output AND return the value.
**Established pattern from find* functions:**

```rust
// Example from findlincombo:
match qseries::findlincombo(&target, &refs, topshift) {
    Some(coeffs) => {
        let s = format_linear_combo(&coeffs, &labels);
        println!("{}", s);
        Ok(Value::String(s))
    }
    None => {
        println!("NOT A LINEAR COMBO.");
        Ok(Value::None)
    }
}
```

Apply same pattern:
- `jac2prod`: print explicit product notation, return `Value::Series(fps)`
- `jac2series`: print series (standard format), return `Value::Series(fps)`
- `qs2jaccombo`: print JAC formula string, return `Value::String(formula)` or `Value::None`

### Pattern 7: JacobiProduct Display Format

**Recommendation for standalone display:**
```
JAC(1,5)                        -- single factor
JAC(1,5)*JAC(2,5)               -- product
JAC(1,5)*JAC(2,5)^(-1)          -- quotient (negative exponent)
JAC(1,5)^3*JAC(2,5)^(-2)        -- general case
```

Rules:
- Exponent 1: omit `^1`
- Exponent -1: show `^(-1)` (not just `-1`)
- Exponent 0: factor already removed by normalization
- Multiple factors: join with `*`
- Empty product (all exponents canceled): display as `1`

### Anti-Patterns to Avoid
- **Computing JAC(a,b) via jacprod():** The existing `jacprod(a,b,sym,T)` computes the Jacobi TRIPLE product, NOT (q^a;q^b)_inf. Use `etaq(a, b, sym, T)` instead.
- **Allowing non-integer exponents:** User decision locks JacobiProduct to integer exponents only. Do not support rational exponents.
- **Missing normalization:** Every JacobiProduct operation MUST normalize (sort, merge, remove zeros) to maintain canonical form. Otherwise display and comparison break.
- **Forgetting to add JacobiProduct to ALL match arms:** Every place that matches on Value variants (type_name, format_value, format_latex, eval_add, eval_sub, eval_mul, eval_div, eval_pow, and any other exhaustive matches) MUST handle JacobiProduct.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Computing (q^a;q^b)_inf | Custom product loop | `qseries::etaq(a, b, sym, T)` | Already handles all edge cases, uses InfiniteProductGenerator |
| Series multiplication | Manual convolution | `arithmetic::mul` | Handles truncation, sparse optimization |
| Series inversion | Newton iteration | `arithmetic::invert` | Already implements O(T^2) coefficient extraction |
| JAC decomposition of a series | Custom factoring | `qseries::jacprodmake` | Implements Andrews' algorithm + period search |
| Linear combination over basis | Custom RREF | `qseries::findlincombo` | Uses rational null space, handles topshift |
| Product notation formatting | Manual string building | Adapt `format_linear_combo` pattern | Established pattern for coefficient-label formatting |

**Key insight:** Every mathematical operation needed for this phase is already implemented in qsym-core. The phase is primarily about CLI plumbing (new Value variant, dispatch, display) rather than new algorithms.

## Common Pitfalls

### Pitfall 1: JAC(a,b) vs jacprod(a,b) confusion
**What goes wrong:** Using `qseries::jacprod(a, b, sym, T)` to compute JAC(a,b), getting the Jacobi triple product instead of (q^a;q^b)_inf.
**Why it happens:** Names are similar; the existing function has a misleading name for this context.
**How to avoid:** Use `qseries::etaq(a, b, sym, T)` for JAC(a,b). Document this clearly in comments.
**Warning signs:** Test JAC(1,5) and compare with (q;q^5)_inf -- if they don't match, wrong function was called.

### Pitfall 2: Exhaustive match coverage
**What goes wrong:** Adding JacobiProduct to Value enum but missing match arms elsewhere, causing compiler errors or `_ =>` fallthrough that silently drops values.
**Why it happens:** Value is matched in ~15+ places across eval.rs and format.rs.
**How to avoid:** Grep for `Value::Symbol` (the most recently added variant) to find all match sites.
**Warning signs:** Compiler warnings about non-exhaustive patterns; JacobiProduct values displaying as unexpected strings.

### Pitfall 3: JacobiProduct + Series arithmetic
**What goes wrong:** User writes `JAC(1,5) + etaq(q, 1, 20)` or `JAC(1,5) + 1` -- what happens?
**Why it happens:** JacobiProduct is a symbolic type; adding it to a Series doesn't make algebraic sense unless you expand it first.
**How to avoid:** JacobiProduct only supports `*`, `/`, `^` with other JacobiProducts (and ^ with Integer). For add/sub, return a TypeError: "cannot add JacobiProduct and series -- use jac2series() to expand first".
**Warning signs:** Silent wrong results or panics when mixing types.

### Pitfall 4: etaq validation for JAC
**What goes wrong:** `JAC(0, 5)` or `JAC(-1, 5)` -- etaq returns zero series for b <= 0, but a can be anything.
**Why it happens:** etaq(b, t, ...) uses b as the starting exponent and t as the step. For JAC(a, b), we call etaq(a, b, ...) where a is start and b is step.
**How to avoid:** Validate in JAC: b must be positive integer, a must be integer. Note: a=0 means (1; q^b)_inf = 0 (since first factor is 1-1=0). Decide if a=0 should error or be allowed.
**Warning signs:** JAC(0,5) returning 0 silently when user expected an error.

### Pitfall 5: theta() with negative q-exponents
**What goes wrong:** For large T and small z_power, terms z^i * q^(i^2) for negative i can produce large negative exponents like q^(-400).
**Why it happens:** sum(z^i * q^(i^2), i=-T..T) includes negative i values.
**How to avoid:** When z is a monomial c*q^k, the exponent is k*i + i^2 = i*(k+i). For i=-T, this is -T*(k-T). If k < T, this can be very negative. Filter terms where q-exponent < 0 (or allow negative exponents if FPS supports them). Most practical: only include terms where the q-exponent is >= 0 (since we're working with formal power series in q, negative powers are typically not meaningful in this context).
**Warning signs:** Series with lots of negative-exponent terms that confuse the display.

### Pitfall 6: qs2jaccombo candidate basis size
**What goes wrong:** Trying all possible JAC(a,b) for a=1..T, b=2..T creates O(T^2) candidates, each requiring etaq expansion.
**Why it happens:** Without intelligent candidate selection, the search space explodes.
**How to avoid:** First run jacprodmake to identify which periods b are relevant. Only generate candidates with those specific b values. Limit candidate count to at most ~50-100 products.
**Warning signs:** qs2jaccombo taking >10 seconds for moderate T values.

## Code Examples

### Adding a new Value variant (established pattern)
```rust
// Source: crates/qsym-cli/src/eval.rs (existing pattern from Value::Symbol)
// Step 1: Add variant to Value enum
pub enum Value {
    // ... existing variants ...
    /// Jacobi product expression: product of (q^a;q^b)_inf^exp factors.
    JacobiProduct(Vec<(i64, i64, i64)>),
}

// Step 2: Add type_name
pub fn type_name(&self) -> &'static str {
    match self {
        // ... existing ...
        Value::JacobiProduct(_) => "jacobi_product",
    }
}
```

### JAC(a,b) constructor dispatch
```rust
// Source: crates/qsym-cli/src/eval.rs dispatch() function
"jac" | "JAC" => {
    expect_args(name, args, 2)?;
    let a = extract_i64(name, args, 0)?;
    let b = extract_i64(name, args, 1)?;
    if b <= 0 {
        return Err(EvalError::Other(format!(
            "JAC: second argument (b) must be a positive integer, got {}", b
        )));
    }
    // Note: a can be any integer, but a=0 gives (1;q^b)=0 which is degenerate
    // We allow it but users should know JAC(0,b) is zero
    Ok(Value::JacobiProduct(vec![(a, b, 1)]))
}
```

### theta(z, q, T) dispatch
```rust
"theta" => {
    expect_args(name, args, 3)?;
    let sym = extract_symbol_id(name, args, 1, env)?;
    let t_range = extract_i64(name, args, 2)?;

    match &args[0] {
        Value::Integer(_) | Value::Rational(_) => {
            // Numeric z: sum(z^i * q^(i^2), i=-T..T)
            let z_val = extract_qrat(name, args, 0)?;
            let mut result = FormalPowerSeries::zero(sym, t_range);
            for i in -t_range..=t_range {
                let q_exp = i * i;
                if q_exp >= t_range { continue; }
                let z_pow_i = qrat_pow(&z_val, i);
                let old = result.coeff(q_exp);
                result.set_coeff(q_exp, old + z_pow_i);
            }
            Ok(Value::Series(result))
        }
        Value::Series(fps) => {
            // q-monomial: extract c*q^k, compute sum(c^i * q^(k*i + i^2))
            let mono = extract_monomial_from_arg(name, args, 0)?;
            let mut result = FormalPowerSeries::zero(sym, t_range);
            for i in -t_range..=t_range {
                let q_exp = mono.power * i + i * i;
                if q_exp < 0 || q_exp >= t_range { continue; }
                let coeff_i = qrat_pow(&mono.coefficient, i);
                let old = result.coeff(q_exp);
                result.set_coeff(q_exp, old + coeff_i);
            }
            Ok(Value::Series(result))
        }
        Value::Symbol(name_str) => {
            println!("Warning: theta(z, q, T) requires z to be numeric or a q-monomial; '{}' is an unassigned symbol", name_str);
            Ok(Value::None)
        }
        _ => Err(EvalError::ArgType { ... })
    }
}
```

### jac2prod product notation formatting
```rust
fn format_jacobi_product_expansion(
    factors: &[(i64, i64, i64)],
    sym_name: &str,
    order: i64,
) -> String {
    // For each factor (a, b, exp), expand as finite product up to order:
    // (q^a; q^b)_inf = (1-q^a)(1-q^{a+b})(1-q^{a+2b})...
    // For exp > 0: multiply those factors
    // For exp < 0: divide by those factors (show in denominator)
    let mut parts = Vec::new();
    for &(a, b, exp) in factors {
        let abs_exp = exp.unsigned_abs();
        for _ in 0..abs_exp {
            let mut factor_parts = Vec::new();
            let mut k = a;
            while k < order && k > 0 {
                factor_parts.push(format!("(1-{}^{})", sym_name, k));
                k += b;
            }
            if exp > 0 {
                parts.push(factor_parts.join(""));
            } else {
                parts.push(format!("1/{}", factor_parts.join("")));
            }
        }
    }
    parts.join("*")
}
```

### Helper: rational power
```rust
/// Compute r^n for integer n (positive, negative, or zero).
fn qrat_pow(r: &QRat, n: i64) -> QRat {
    if n == 0 {
        return QRat::one();
    }
    let base = if n < 0 {
        QRat::one() / r.clone()
    } else {
        r.clone()
    };
    let abs_n = n.unsigned_abs();
    let mut result = QRat::one();
    for _ in 0..abs_n {
        result = result * base.clone();
    }
    result
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| jacprod returns triple product FPS | JAC(a,b) returns symbolic JacobiProduct value | Phase 37 | Users can build symbolic JP expressions before expanding |
| No general theta(z,q,T) | theta(z,q,T) = sum(z^i*q^(i^2)) | Phase 37 | Enables research workflows with parametric theta |
| jacprodmake returns Dict | qs2jaccombo returns JAC formula string | Phase 37 | Human-readable output consistent with find* pattern |

**Key distinction from existing infrastructure:**
- `jacprodmake(f, q, T)` decomposes f into a **single** JAC product (multiplicative)
- `qs2jaccombo(f, q, T)` decomposes f into a **sum** of JAC products (additive combination) -- this is the new capability

## Open Questions

1. **theta() truncation semantics**
   - What we know: T is used as both the sum range (-T..T) and the truncation order for the resulting FPS
   - What's unclear: Should T be the sum range and truncation order simultaneously, or should they be independent?
   - Recommendation: Use T for both (matches the pattern of other functions like theta2/3/4 which take a single order parameter). The user's T choice determines both.

2. **JAC(0,b) behavior**
   - What we know: etaq(0, b, ...) returns the zero series since first factor is (1-q^0) = (1-1) = 0
   - What's unclear: Should JAC(0,b) error or return a degenerate JacobiProduct?
   - Recommendation: Allow it at the JacobiProduct level (it's a valid symbolic expression), but jac2series/jac2prod will correctly return the zero series. Add a note in help text.

3. **qs2jaccombo candidate generation strategy**
   - What we know: Need a set of candidate JAC products to test as basis vectors
   - What's unclear: Optimal strategy for choosing which (a,b) pairs to try
   - Recommendation: Use jacprodmake first to identify the dominant period b, then try all a=1..b-1 for that period. If that fails, try periods up to sqrt(T). Limit total candidates to ~100.

4. **JacobiProduct in add/sub operations**
   - What we know: Only *, /, ^ are algebraically meaningful for symbolic JP expressions
   - What's unclear: Should JP + JP be an error or silently expand?
   - Recommendation: Return TypeError for add/sub with JacobiProduct. Users should use jac2series() first. This is cleaner than silent expansion.

## Sources

### Primary (HIGH confidence)
- **crates/qsym-core/src/qseries/products.rs** -- etaq and jacprod implementations, verified that etaq(a,b,sym,T) computes (q^a;q^b)_inf
- **crates/qsym-core/src/qseries/prodmake.rs** -- jacprodmake algorithm, JacobiProductForm struct with BTreeMap<(i64,i64), i64> factors
- **crates/qsym-cli/src/eval.rs** -- Value enum (11 variants), dispatch() pattern, eval_mul/eval_div/eval_pow implementations, helper functions
- **crates/qsym-cli/src/format.rs** -- format_value pattern for all Value variants
- **crates/qsym-core/src/qseries/relations.rs** -- findlincombo implementation for linear combination discovery
- **crates/qsym-core/src/series/arithmetic.rs** -- mul, add, invert, scalar_mul, negate functions

### Secondary (MEDIUM confidence)
- [Garvan q-product tutorial](https://qseries.org/fgarvan/papers/qmaple.pdf) -- Defines JAC(a,b) notation and jacprodmake algorithm (PDF not parseable, but referenced by name in codebase)
- [Garvan auto-theta paper](https://qseries.org/fgarvan/papers/auto-theta.pdf) -- qs2jaccombo and jac2series function descriptions

### Tertiary (LOW confidence)
- qs2jaccombo detailed algorithm: inferred from mathematical requirements and existing findlincombo infrastructure. Garvan's exact implementation could not be extracted from PDFs. The approach of "expand candidates to FPS, use linear algebra" is mathematically standard.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all building blocks verified in codebase
- Architecture (Value::JacobiProduct): HIGH - follows established Value::Symbol pattern exactly
- Architecture (theta): HIGH - straightforward sum, similar to existing theta2/3/4 but parametric
- Architecture (jac2prod/jac2series): HIGH - just etaq expansion + formatting
- Architecture (qs2jaccombo): MEDIUM - algorithm inferred from mathematical principles and existing tools, not verified against Garvan's exact implementation
- Pitfalls: HIGH - verified by reading actual code for each edge case

**Research date:** 2026-02-19
**Valid until:** 2026-03-19 (stable -- this is internal feature work on a stable codebase)
