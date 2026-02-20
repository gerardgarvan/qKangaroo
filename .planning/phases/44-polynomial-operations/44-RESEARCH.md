# Phase 44: Polynomial Operations - Research

**Researched:** 2026-02-20
**Domain:** Polynomial factoring over Q[x] + expression substitution in q-series CLI
**Confidence:** HIGH

## Summary

Phase 44 adds two user-facing functions to the q-Kangaroo CLI: `factor(poly)` for factoring q-polynomials into irreducible factors over the rationals, and `subs(var=val, expr)` for substituting values into expressions. Both build on substantial existing infrastructure.

The core polynomial module (`crates/qsym-core/src/poly/`) already has `QRatPoly` with full arithmetic (add, sub, mul, div_rem, exact_div), GCD via subresultant PRS, content/primitive-part extraction, and Horner evaluation. The series module has sparse `FormalPowerSeries` (BTreeMap<i64, QRat>). The existing `qfactor` function in `qseries/factoring.rs` decomposes into `(1-q^i)^m` form but does NOT perform true irreducible factoring. The new `factor()` must produce actual irreducible factors like `(1-q)(1+q)(1-q+q^2)(1+q+q^2)` from `1-q^6`.

The `subs` function requires special AST handling because `subs(q=1, expr)` contains `q=1` which the parser currently treats as a comparison (CompOp::Eq), evaluating to Bool before dispatch. The solution is to intercept `subs` calls in the FuncCall branch of `eval_expr`, extracting the variable name and substitution value from the Compare AST node before evaluating args.

**Primary recommendation:** Implement cyclotomic trial division for `factor()` (computing Phi_n(q) dynamically, dividing out from largest n down), and handle `subs` via AST-level interception in eval_expr's FuncCall branch.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| POLY-01 | User can call `factor(poly)` to factor a polynomial in q into cyclotomic and irreducible factors over the rationals | Existing `QRatPoly` has all needed arithmetic (div_rem, exact_div, GCD). Need: FPS-to-QRatPoly conversion, cyclotomic polynomial generation, trial division algorithm, result formatting. Cyclotomic trial division is the standard approach for q-polynomials. |
| POLY-02 | User can call `subs(var=val, expr)` to substitute a value for a variable in an expression | Need AST-level interception in eval_expr FuncCall branch to extract variable name from Compare node. For `subs(q=rational, series)` evaluate via Horner. For `subs(q=q^k, series)` perform exponent scaling on FPS. Both operations are straightforward on the existing BTreeMap representation. |
</phase_requirements>

## Standard Stack

### Core (already in codebase)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `QRatPoly` | in-tree | Dense univariate polynomial over Q | Has div_rem, exact_div, GCD, eval, content, primitive_part |
| `FormalPowerSeries` | in-tree | Sparse q-series | BTreeMap<i64, QRat> with truncation; source data for factor/subs |
| `rug` | (existing dep) | Arbitrary-precision integers/rationals | Already used for QInt/QRat |

### New modules needed
| Module | Purpose | Location |
|--------|---------|----------|
| `poly/factor.rs` | Irreducible factoring over Q[x] | `crates/qsym-core/src/poly/factor.rs` |
| `poly/cyclotomic.rs` | Cyclotomic polynomial computation | `crates/qsym-core/src/poly/cyclotomic.rs` |

No new external dependencies are needed.

## Architecture Patterns

### Recommended Module Structure
```
crates/qsym-core/src/poly/
  mod.rs           # QRatPoly (existing)
  arithmetic.rs    # +, -, *, div_rem (existing)
  gcd.rs           # poly_gcd, resultant (existing)
  ratfunc.rs       # QRatRationalFunc (existing)
  cyclotomic.rs    # NEW: cyclotomic_poly(n), divisors()
  factor.rs        # NEW: factor_over_q(), Factorization result type

crates/qsym-cli/src/
  eval.rs          # MODIFY: add factor/subs dispatch, subs AST interception
  help.rs          # MODIFY: add help entries
  repl.rs          # MODIFY: add to completion list
```

### Pattern 1: FPS-to-QRatPoly Conversion
**What:** Convert a FormalPowerSeries (that is an exact polynomial, POLYNOMIAL_ORDER sentinel) to a QRatPoly for factoring.
**When to use:** Before calling factor algorithms.
**Example:**
```rust
// Convert sparse BTreeMap FPS to dense QRatPoly
fn fps_to_qratpoly(fps: &FormalPowerSeries) -> QRatPoly {
    let degree = fps.iter().last().map(|(&k, _)| k).unwrap_or(0);
    let mut coeffs = vec![QRat::zero(); (degree + 1) as usize];
    for (&k, v) in fps.iter() {
        if k >= 0 && (k as usize) < coeffs.len() {
            coeffs[k as usize] = v.clone();
        }
    }
    QRatPoly::from_vec(coeffs)
}
```

### Pattern 2: Cyclotomic Trial Division
**What:** Factor by dividing out cyclotomic polynomials Phi_n(q) from largest n down to 1.
**When to use:** For `factor()` -- the primary use case is q-polynomials which are typically products of cyclotomic factors.
**Why largest-first:** Because Phi_1(q) = q-1 divides many cyclotomic polys when considered as evaluation. Working from largest n prevents false matches.
**Example:**
```rust
// The nth cyclotomic polynomial via Mobius function / inclusion-exclusion
// Phi_n(x) = prod_{d | n} (x^d - 1)^{mu(n/d)}
fn cyclotomic_poly(n: usize) -> QRatPoly {
    // Start with x^n - 1
    // Divide by Phi_d for each proper divisor d of n
    let mut result = x_n_minus_1(n);
    for d in proper_divisors(n) {
        let phi_d = cyclotomic_poly(d);  // recursive, memoizable
        result = result.exact_div(&phi_d);
    }
    result
}

// Factor by trial division of cyclotomic polynomials
fn factor_cyclotomic(poly: &QRatPoly) -> Vec<(QRatPoly, usize)> {
    let mut remaining = poly.primitive_part();
    let deg = remaining.degree().unwrap_or(0);
    let mut factors = Vec::new();

    for n in (1..=deg).rev() {
        let phi_n = cyclotomic_poly(n);
        let phi_deg = phi_n.degree().unwrap_or(0);
        while remaining.degree().unwrap_or(0) >= phi_deg {
            let (q, r) = remaining.div_rem(&phi_n);
            if r.is_zero() {
                *factors.entry(n).or_insert(0) += 1;
                remaining = q;
            } else {
                break;
            }
        }
    }
    factors
}
```

### Pattern 3: AST-Level Interception for subs()
**What:** Handle `subs(q=expr, target)` by inspecting the raw AST before evaluating args.
**When to use:** Required because `q=1` would evaluate to `Bool(true)` if evaluated normally.
**Example:**
```rust
// In eval_expr, FuncCall branch, BEFORE evaluating args:
AstNode::FuncCall { name, args } if name == "subs" => {
    // args[0] should be AstNode::Compare { op: CompOp::Eq, lhs, rhs }
    // Extract variable name from lhs, evaluate rhs for substitution value
    // args[1] is the target expression -- evaluate normally
    if args.len() != 2 {
        return Err(wrong_arg_count("subs", "2", args.len()));
    }
    match &args[0] {
        AstNode::Compare { op: CompOp::Eq, lhs, rhs } => {
            let var_name = match lhs.as_ref() {
                AstNode::Variable(name) => name.clone(),
                _ => return Err(/* error: expected variable on LHS of = */),
            };
            let sub_value = eval_expr(rhs, env)?;
            let target = eval_expr(&args[1], env)?;
            perform_substitution(var_name, sub_value, target, env)
        }
        _ => return Err(/* error: expected var=value as first argument */),
    }
}
```

### Pattern 4: Substitution Implementation
**What:** Three substitution modes based on what is substituted.
**When to use:** Inside the `subs` dispatch.

```rust
// Mode 1: subs(q=rational, series) -> evaluate polynomial at rational point
// Result: Value::Rational or Value::Integer
fn subs_rational(fps: &FormalPowerSeries, val: &QRat) -> QRat {
    let mut result = QRat::zero();
    let mut q_power_cache: BTreeMap<i64, QRat> = BTreeMap::new();
    for (&k, coeff) in fps.iter() {
        let q_k = q_power_cache.entry(k).or_insert_with(|| val.pow(k));
        result = result + coeff * q_k;
    }
    result
}

// Mode 2: subs(q=q^k, series) -> exponent scaling (multiply all exponents by k)
// Result: Value::Series (new FPS with scaled exponents)
fn subs_power_scale(fps: &FormalPowerSeries, k: i64) -> FormalPowerSeries {
    let mut new_coeffs = BTreeMap::new();
    let new_trunc = if fps.truncation_order() == POLYNOMIAL_ORDER {
        POLYNOMIAL_ORDER
    } else {
        fps.truncation_order() * k
    };
    for (&exp, val) in fps.iter() {
        new_coeffs.insert(exp * k, val.clone());
    }
    FormalPowerSeries::from_coeffs(fps.variable(), new_coeffs, new_trunc)
}

// Mode 3: subs(q=integer, series) -> same as mode 1 but integer result
```

### Anti-Patterns to Avoid
- **Do NOT evaluate subs args normally:** The `q=1` syntax MUST be intercepted at the AST level, not dispatched through the standard FuncCall path. If args are evaluated first, `q=1` becomes `Bool(true)`.
- **Do NOT use general polynomial factoring (LLL/Berlekamp-Zassenhaus):** Overkill for q-series polynomials. Cyclotomic trial division handles > 95% of use cases. Report any non-cyclotomic remainder as an irreducible factor.
- **Do NOT convert series with O(q^T) truncation to QRatPoly for factoring:** Only exact polynomials (POLYNOMIAL_ORDER sentinel) should be factored. Truncated series are not polynomials.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Polynomial GCD | Custom Euclidean | Existing `poly_gcd()` | Subresultant PRS already handles coefficient explosion |
| Polynomial division | Long division from scratch | Existing `QRatPoly::div_rem()` | Already correct, tested to degree 10+ |
| Arbitrary precision | Custom bignum | Existing `rug`/GMP | Battle-tested, already a dependency |
| Cyclotomic polynomial values | Hardcoded table for large n | Recursive computation via divisors | Table approach doesn't scale; recursive approach with memoization is O(sum euler_phi) |

**Key insight:** The factoring algorithm itself is novel code, but ALL the polynomial arithmetic primitives it needs already exist in the `poly/` module. The implementation is primarily composition of existing operations.

## Common Pitfalls

### Pitfall 1: Forgetting POLYNOMIAL_ORDER Sentinel
**What goes wrong:** Trying to factor a truncated series (has O(q^T) term) as if it were an exact polynomial.
**Why it happens:** Both exact polynomials and truncated series are stored as FormalPowerSeries. The only difference is `truncation_order` -- exact polys use `POLYNOMIAL_ORDER` (1 billion).
**How to avoid:** Check `fps.truncation_order() == POLYNOMIAL_ORDER` before allowing factoring. If not polynomial, return error "cannot factor truncated series -- use series() to get exact polynomial first."
**Warning signs:** Wrong factors, nonsensical results.

### Pitfall 2: Negative Exponents in FPS
**What goes wrong:** FormalPowerSeries can have negative exponent keys (e.g., q^(-1)). QRatPoly cannot.
**Why it happens:** Series from eta quotients or products can have negative powers.
**How to avoid:** Before converting FPS to QRatPoly, check that min_order >= 0. If negative exponents exist, multiply by q^{|min|} first, factor, then adjust results.
**Warning signs:** Index out of bounds, incorrect factorization.

### Pitfall 3: subs(q=q^2) Truncation Order
**What goes wrong:** When scaling exponents by k, the truncation order must also scale. If original series has O(q^20) and we substitute q -> q^2, the result should have O(q^40).
**Why it happens:** Easy to forget truncation propagation.
**How to avoid:** Multiply truncation_order by k when substituting q -> q^k. UNLESS it's a POLYNOMIAL_ORDER sentinel, in which case preserve the sentinel.
**Warning signs:** Premature truncation, missing high-order terms.

### Pitfall 4: Content Factor in Factorization
**What goes wrong:** Forgetting to extract the content (rational scalar) before factoring.
**Why it happens:** QRatPoly can have rational coefficients. Cyclotomic polynomials are monic with integer coefficients. If the polynomial has content != 1, trial division fails.
**How to avoid:** Always extract content first: `content = poly.content(); primitive = poly.primitive_part();`. Factor the primitive part. Report content as scalar prefactor.
**Warning signs:** "Not fully factored" when polynomial should factor completely.

### Pitfall 5: Cyclotomic Order Too Large
**What goes wrong:** For a degree-d polynomial, we need to check cyclotomic polynomials Phi_n for n up to d. But Phi_n has degree euler_phi(n) which can be much smaller than n.
**Why it happens:** Confusion between the cyclotomic index n and its degree phi(n).
**How to avoid:** Iterate n from degree down to 1. For each n, compute phi_n only if euler_phi(n) <= remaining degree. Skip if Phi_n's degree exceeds the remaining polynomial's degree.
**Warning signs:** Unnecessary computation, but not correctness issues.

### Pitfall 6: subs AST Interception Placement
**What goes wrong:** If `subs` interception is placed after user-defined procedure lookup, a user variable named `subs` would shadow the built-in.
**Why it happens:** The eval_expr FuncCall branch first checks for user procedures, then dispatches builtins.
**How to avoid:** Place the `subs` interception BEFORE the user-procedure check, or after the procedure check but before arg evaluation (i.e., add it as a special case alongside RETURN).
**Warning signs:** User-defined variable named `subs` breaks the builtin.

## Code Examples

### FPS to QRatPoly Conversion
```rust
// Source: Composition of existing FPS.iter() and QRatPoly::from_vec()
fn fps_to_qratpoly(fps: &FormalPowerSeries) -> Result<QRatPoly, String> {
    // Only factor exact polynomials
    if fps.truncation_order() != POLYNOMIAL_ORDER {
        return Err("cannot factor truncated series".to_string());
    }
    if fps.is_zero() {
        return Err("cannot factor zero polynomial".to_string());
    }
    // Check for negative exponents
    if let Some(&min_k) = fps.coefficients.keys().next() {
        if min_k < 0 {
            return Err(format!("polynomial has negative exponent q^{}", min_k));
        }
    }
    let degree = *fps.coefficients.keys().last().unwrap() as usize;
    let mut coeffs = vec![QRat::zero(); degree + 1];
    for (&k, v) in fps.iter() {
        coeffs[k as usize] = v.clone();
    }
    Ok(QRatPoly::from_vec(coeffs))
}
```

### Cyclotomic Polynomial via Recursive Division
```rust
// Phi_1(x) = x - 1
// Phi_2(x) = x + 1
// Phi_3(x) = x^2 + x + 1
// Phi_6(x) = x^2 - x + 1
// General: Phi_n(x) = (x^n - 1) / prod_{d | n, d < n} Phi_d(x)
fn cyclotomic_poly(n: usize) -> QRatPoly {
    // x^n - 1
    let mut xn_minus_1 = QRatPoly::from_i64_coeffs(
        &std::iter::once(-1)
            .chain(std::iter::repeat(0).take(n - 1))
            .chain(std::iter::once(1))
            .collect::<Vec<_>>()
    );
    // Divide by Phi_d for each proper divisor d of n
    for d in 1..n {
        if n % d == 0 {
            let phi_d = cyclotomic_poly(d);
            xn_minus_1 = xn_minus_1.exact_div(&phi_d);
        }
    }
    xn_minus_1
}
```

### Factor Result Type
```rust
/// Result of factoring a polynomial over Q[x].
pub struct Factorization {
    /// Scalar factor (content of the original polynomial).
    pub scalar: QRat,
    /// Irreducible factors with multiplicities, sorted by degree then lex.
    pub factors: Vec<(QRatPoly, usize)>,
}

impl Factorization {
    /// Format as display string: "scalar * (factor1)^e1 * (factor2)^e2 * ..."
    /// Uses 'q' as the variable name.
    pub fn display(&self, var_name: &str) -> String {
        // Each factor displayed as polynomial with var_name replacing 'x'
    }
}
```

### Display Formatting for factor() Output
```rust
// The success criterion requires output like:
// (1-q)(1+q)(1-q+q^2)(1+q+q^2)
// This means factors should be displayed in ascending degree order,
// with each factor shown as a parenthesized polynomial in q.
// Multiplicities > 1 shown as ^n suffix.
```

### Substitution: q=rational (Evaluate)
```rust
// For subs(q=1, f): sum all coefficients
// For subs(q=r, f): Horner-like evaluation
fn evaluate_fps_at_rational(fps: &FormalPowerSeries, val: &QRat) -> QRat {
    let mut result = QRat::zero();
    for (&k, coeff) in fps.iter() {
        // val^k * coeff
        let power = val.pow(k as u32);  // use repeated squaring
        result = &result + &(coeff * &power);
    }
    result
}
```

### Substitution: q=q^k (Exponent Scaling)
```rust
// For subs(q=q^2, f): multiply all exponents by 2
fn scale_exponents(fps: &FormalPowerSeries, k: i64) -> FormalPowerSeries {
    let new_trunc = if fps.truncation_order() >= POLYNOMIAL_ORDER {
        POLYNOMIAL_ORDER
    } else {
        fps.truncation_order() * k.abs()
    };
    let mut new_coeffs = BTreeMap::new();
    for (&exp, val) in fps.iter() {
        new_coeffs.insert(exp * k, val.clone());
    }
    FormalPowerSeries::from_coeffs(fps.variable(), new_coeffs, new_trunc)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `qfactor` (1-q^i decomposition) | `factor` (true irreducible Q[x] factoring) | Phase 44 | Users get standard polynomial factorization, not just q-shift decomposition |
| No substitution | `subs(q=val, expr)` | Phase 44 | Enables evaluation at points and exponent transformations |

**Not deprecated:** `qfactor` remains useful for its specific (1-q^i) decomposition purpose. `factor` is a different, complementary operation.

## Open Questions

1. **Factoring non-cyclotomic remainders**
   - What we know: Cyclotomic trial division handles 95%+ of q-series polynomial use cases. Some polynomials may have non-cyclotomic irreducible factors.
   - What's unclear: Do we need a general-purpose irreducible factoring algorithm (Kronecker/Zassenhaus) for the remaining cases?
   - Recommendation: Report non-cyclotomic remainder as a single "irreducible" factor. Add a TODO for general factoring if users need it. The success criteria only require cyclotomic factoring.

2. **subs with non-q variables**
   - What we know: The signature is `subs(var=val, expr)`. In principle, var could be any symbol, not just q.
   - What's unclear: Do we need to support substituting arbitrary symbols, or just q?
   - Recommendation: Support `subs(q=val, expr)` where the variable name must match the series variable. Error if mismatched. This covers all use cases in the success criteria.

3. **factor() output format as Value**
   - What we know: Success criteria show factored form like `(1-q)(1+q)(1-q+q^2)(1+q+q^2)`.
   - What's unclear: Should `factor()` return a display string, a structured Dict, or a new Value variant?
   - Recommendation: Return a `Value::String` with the factored form for display purposes. Also print the factored form. The existing `qfactor` returns `Value::Dict` -- `factor` could do either, but a string is simpler for the stated requirements.

4. **Detecting if FPS is a polynomial**
   - What we know: POLYNOMIAL_ORDER sentinel marks exact polys. But some series computed with explicit orders (e.g., `qbin(q, 2, 4)`) produce exact polynomials with POLYNOMIAL_ORDER.
   - What's unclear: Are there edge cases where a naturally-finite polynomial doesn't have POLYNOMIAL_ORDER?
   - Recommendation: Check for POLYNOMIAL_ORDER sentinel. If not set but the series has very few terms, allow factoring anyway with a warning.

## Sources

### Primary (HIGH confidence)
- `crates/qsym-core/src/poly/mod.rs` -- QRatPoly: constructors, content, primitive_part, eval, q_shift
- `crates/qsym-core/src/poly/arithmetic.rs` -- QRatPoly add/sub/mul/div_rem/exact_div
- `crates/qsym-core/src/poly/gcd.rs` -- poly_gcd (subresultant PRS), poly_resultant
- `crates/qsym-core/src/series/mod.rs` -- FormalPowerSeries: BTreeMap, truncation_order, iter()
- `crates/qsym-core/src/qseries/factoring.rs` -- existing qfactor: (1-q^i) decomposition
- `crates/qsym-cli/src/eval.rs` -- dispatch function, eval_expr FuncCall handling, POLYNOMIAL_ORDER
- `crates/qsym-cli/src/ast.rs` -- AstNode::Compare with CompOp::Eq (relevant to subs parsing)
- `crates/qsym-cli/src/repl.rs` -- canonical_function_names() for tab completion
- `crates/qsym-cli/src/help.rs` -- general_help() and FUNC_HELP entries

### Secondary (MEDIUM confidence)
- [Cyclotomic polynomial - Wikipedia](https://en.wikipedia.org/wiki/Cyclotomic_polynomial) -- Phi_n(x) = (x^n-1) / prod Phi_d(x) for proper divisors d|n
- [Factorization of polynomials - Wikipedia](https://en.wikipedia.org/wiki/Factorization_of_polynomials) -- Overview of Kronecker, Berlekamp-Zassenhaus, LLL approaches

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all code is in-tree, read directly
- Architecture: HIGH - all patterns derived from existing codebase patterns
- Pitfalls: HIGH - derived from understanding the actual data structures (FPS vs QRatPoly, POLYNOMIAL_ORDER sentinel, AST Compare nodes)
- Factoring algorithm: HIGH for cyclotomic trial division, MEDIUM for edge cases (non-cyclotomic remainder handling)
- subs implementation: HIGH - the AST interception pattern is the only viable approach given the parser design

**Research date:** 2026-02-20
**Valid until:** 2026-03-20 (stable -- all code is in-tree, no external dependency changes)
