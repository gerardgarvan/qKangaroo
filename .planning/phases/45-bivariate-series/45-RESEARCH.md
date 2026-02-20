# Phase 45: Bivariate Series - Research

**Researched:** 2026-02-20
**Domain:** Bivariate q-series computation (Laurent polynomials in z with FPS coefficients)
**Confidence:** HIGH

## Summary

This phase extends tripleprod, quinprod, and winquist to handle symbolic z variables (i.e., when the first argument is a `Value::Symbol` instead of a q-monomial). Currently these functions require z to be a `QMonomial` (a concrete `c * q^m`) and produce a single `FormalPowerSeries` in q. When z is symbolic, the output is a **Laurent polynomial in z whose coefficients are FormalPowerSeries in q**.

The mathematical foundations are well-established: the Jacobi triple product identity gives `prod_{n>=1}(1-q^n)(1+zq^n)(1+z^{-1}q^{n-1}) = sum_{n=-inf}^{inf} z^n * q^{n(n+1)/2}`, and similar closed-form sum representations exist for the quintuple product and Winquist's identity. These sum forms directly yield the z-exponent-to-q-coefficient mapping needed for the data structure.

The implementation requires: (1) a new `Value::BivariateSeries` variant in the CLI, (2) a `BivariateSeries` data structure (a `BTreeMap<i64, FormalPowerSeries>` mapping z-exponents to q-series coefficients), (3) arithmetic operations on bivariate series, (4) display formatting, and (5) detection of symbolic z in the dispatch logic for tripleprod/quinprod/winquist.

**Primary recommendation:** Add a `BivariateSeries` struct to qsym-core (new file `crates/qsym-core/src/series/bivariate.rs`), a `Value::BivariateSeries` variant to the CLI, and extend the three product function dispatchers to detect `Value::Symbol` arguments and produce bivariate results via the known sum formulas.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| BIVAR-01 | tripleprod(z, q, T) with symbolic z produces Laurent polynomial in z with q-series coefficients | Jacobi triple product sum formula: sum_{n=-inf}^{inf} z^n * q^{n(n+1)/2}. Direct term-by-term computation. |
| BIVAR-02 | quinprod(z, q, T) with symbolic z produces Laurent polynomial in z with q-series coefficients | Quintuple product sum formula: sum_{m=-inf}^{inf} (z^{3m} - z^{-3m-1}) * q^{m(3m+1)/2}. Direct computation. |
| BIVAR-03 | winquist(a, b, q, T) with symbolic a, b produces multivariate series | Winquist involves two symbolic variables; needs multivariate Laurent polynomial or product-of-bivariate approach. See Open Questions. |
| BIVAR-04 | Arithmetic (add, subtract, multiply, negate) on bivariate series values | Coefficient-wise FPS arithmetic for add/sub/negate; convolution over z-exponents for multiply. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| BTreeMap<i64, FormalPowerSeries> | std | Sparse Laurent polynomial mapping z^k -> q-series coefficient | Already used for FPS coefficients; natural extension to bivariate |
| qsym_core::series::FormalPowerSeries | existing | Each coefficient of z^k is an FPS in q | Reuses all existing FPS arithmetic |
| qsym_core::series::arithmetic | existing | FPS add/sub/mul/negate/scalar_mul for coefficient operations | All bivariate arithmetic delegates to existing FPS ops |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| qsym_core::number::QRat | existing | Rational coefficients within FPS | Unchanged from current |
| qsym_core::symbol::SymbolId | existing | Variable identification (q, z, a, b) | Bivariate series stores variable names for display |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| BTreeMap<i64, FPS> in qsym-core | HashMap<i64, FPS> | BTreeMap gives sorted iteration for display; matches FPS pattern |
| New struct in qsym-core | Struct only in qsym-cli | Core struct enables future Python API bivariate support; better separation |
| Two-variable FPS (BTreeMap<(i64,i64), QRat>) | BTreeMap<i64, FPS> | Nested structure preserves truncation semantics per-variable; cleaner arithmetic |

## Architecture Patterns

### Recommended Project Structure
```
crates/qsym-core/src/series/
  mod.rs           # Add `pub mod bivariate;`
  bivariate.rs     # NEW: BivariateSeries struct + arithmetic

crates/qsym-cli/src/
  eval.rs          # Add Value::BivariateSeries variant; extend tripleprod/quinprod/winquist dispatch
  format.rs        # Add bivariate display formatting
  help.rs          # Update help text for tripleprod/quinprod/winquist
```

### Pattern 1: BivariateSeries Data Structure

**What:** A sparse Laurent polynomial in an "outer" variable (z) where each coefficient is a FormalPowerSeries in an "inner" variable (q).

**When to use:** Whenever a product function receives a symbolic variable instead of a q-monomial.

**Structure:**
```rust
// In crates/qsym-core/src/series/bivariate.rs

use std::collections::BTreeMap;
use crate::number::QRat;
use crate::symbol::SymbolId;
use super::FormalPowerSeries;

/// A bivariate series: Laurent polynomial in `outer_variable` with
/// FormalPowerSeries coefficients in the inner variable (usually q).
///
/// Represents: sum_k z^k * f_k(q) where f_k(q) are formal power series.
///
/// Invariants:
/// - No key maps to a zero FPS (enforce on insertion)
/// - All FPS coefficients share the same inner variable and truncation order
#[derive(Clone, Debug)]
pub struct BivariateSeries {
    /// Outer variable name (e.g., "z" for tripleprod, could also be "a" or "b")
    pub outer_variable: String,
    /// Sparse mapping: z-exponent -> FPS coefficient in q
    pub terms: BTreeMap<i64, FormalPowerSeries>,
    /// Inner variable (usually q's SymbolId)
    pub inner_variable: SymbolId,
    /// Truncation order for the inner (q) series
    pub truncation_order: i64,
}
```

### Pattern 2: Sum-Form Computation for Symbolic Products

**What:** Instead of computing infinite products (which requires z to be a q-power), use the known sum-form identities to directly build bivariate series.

**When to use:** When tripleprod/quinprod/winquist receive a Value::Symbol as the z argument.

**Jacobi Triple Product (BIVAR-01):**
```
tripleprod(z, q, T) = sum_{n=-inf}^{inf} z^n * q^{n(n+1)/2}
```
Each term contributes coefficient 1 at z^n with q-exponent n(n+1)/2. We only include terms where n(n+1)/2 < T.

The range of n: solve n(n+1)/2 < T, so roughly |n| < sqrt(2T). Iterate n from -N to N.

**Quintuple Product (BIVAR-02):**
```
quinprod(z, q, T) = sum_{m=-inf}^{inf} (z^{3m} - z^{-3m-1}) * q^{m(3m+1)/2}
```
Each term contributes:
- +1 at z^{3m} with q-exponent m(3m+1)/2
- -1 at z^{-3m-1} with q-exponent m(3m+1)/2

Range: |m| bounded by m(3m+1)/2 < T, roughly |m| < sqrt(2T/3).

**Winquist Product (BIVAR-03):**
```
winquist(a, b, q, T) = sum over (r,s) of sign * a^r * b^s * q^{exponent(r,s)}
```
Winquist's identity involves two outer variables. This needs either:
(a) A trivariate series (BTreeMap<(i64,i64), FPS>) mapping (a-exp, b-exp) -> q-series, or
(b) A nested bivariate: Laurent poly in a with BivariateSeries-in-b coefficients.

Option (a) is simpler and more practical: a `MultivariateSeries` with a `BTreeMap<Vec<i64>, FPS>` or `BTreeMap<(i64,i64), FPS>`.

### Pattern 3: Value Variant Integration

**What:** Add `Value::BivariateSeries(BivariateSeries)` to the Value enum.

**When to use:** Return type when symbolic z is detected in product functions.

**Detection logic in dispatch:**
```rust
"tripleprod" => {
    if args.len() == 3 {
        match &args[0] {
            Value::Symbol(var_name) => {
                // Symbolic z -> produce BivariateSeries
                let sym = extract_symbol_id(name, args, 1, env)?;
                let order = extract_i64(name, args, 2)?;
                let result = compute_tripleprod_bivariate(var_name, sym, order);
                Ok(Value::BivariateSeries(result))
            }
            Value::Series(_) => {
                // q-monomial z -> existing FPS path (unchanged)
                let monomial = extract_monomial_from_arg(name, args, 0)?;
                let sym = extract_symbol_id(name, args, 1, env)?;
                let order = extract_i64(name, args, 2)?;
                let result = qseries::tripleprod(&monomial, sym, order);
                Ok(Value::Series(result))
            }
            _ => { /* legacy path or error */ }
        }
    }
}
```

### Pattern 4: Display Format

**What:** Show bivariate series as a Laurent polynomial in z with q-series coefficients in parentheses.

**Format examples:**
```
tripleprod(z, q, 5):
  z^2*q^3 + z*q + 1 + z^(-1)*q + z^(-2)*q^3 + O(q^5)

Alternative (Garvan-like, coefficients grouped by z-power, descending):
  (q^3 + O(q^5))*z^2 + (q + O(q^5))*z + (1 + O(q^5)) + (q + O(q^5))*z^(-1) + (q^3 + O(q^5))*z^(-2)
```

The recommended display groups by z-power in descending order, showing each q-series coefficient inline:
```
(coeff_k)*z^k + (coeff_{k-1})*z^{k-1} + ... + (coeff_0) + ... + (coeff_{-j})*z^{-j}
```
Where each `coeff_k` is displayed as an FPS. For simple coefficients (single q-monomial), compress: `q^3*z^2` instead of `(q^3 + O(q^5))*z^2`.

### Anti-Patterns to Avoid
- **Storing bivariate as flat BTreeMap<(i64,i64), QRat>:** Loses truncation order semantics for the q-variable, makes arithmetic harder
- **Trying to use existing FPS for bivariate:** FPS is fundamentally single-variable; forcing two variables into one structure creates confusion
- **Computing products symbolically:** The product-form computation requires numerical z; always use the sum-form identity for symbolic z
- **Modifying existing tripleprod/quinprod in qsym-core:** Keep the core functions unchanged; add new bivariate computation functions alongside them

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| FPS coefficient arithmetic | Custom rational arithmetic for bivariate | Existing `arithmetic::add/sub/mul/negate/scalar_mul` | Already correct, tested with 863 core tests |
| Laurent polynomial z-exponent tracking | Manual array indexing | BTreeMap<i64, FPS> with negative keys | BTreeMap naturally handles negative exponents |
| Sum-form truncation bounds | Ad-hoc bounds | Solve n(n+1)/2 < T analytically: n_max = floor((-1 + sqrt(1+8T))/2) | Off-by-one errors are common in manual bounds |
| Variable name display | Hardcoded "z" | Store outer_variable as String in struct | Users might use z, a, b, w, etc. |

**Key insight:** The bivariate structure is simple (sparse map of FPS values). All the hard arithmetic is already implemented in the FPS module. The new code is primarily: (a) sum-form evaluation loops, (b) display formatting, and (c) dispatch routing.

## Common Pitfalls

### Pitfall 1: Incorrect Sum Bounds
**What goes wrong:** Missing terms because the iteration range for n is too narrow, or including terms beyond truncation.
**Why it happens:** The sum is infinite; we must truncate based on when q-exponents exceed T.
**How to avoid:** For tripleprod, n(n+1)/2 < T means |n| < (-1 + sqrt(1 + 8T))/2. Use `((1.0 + (1.0 + 8.0 * T as f64).sqrt()) / 2.0).ceil() as i64` for the bound, then filter terms where q-exponent >= T.
**Warning signs:** Missing terms at boundaries; asymmetric results when the identity should be symmetric.

### Pitfall 2: Variable Name Collision
**What goes wrong:** User calls `tripleprod(q, q, 10)` where the first `q` is Value::Symbol("q") -- the outer variable has the same name as the inner variable.
**Why it happens:** Currently `q` is special (the session variable). But a user might type bare `q` expecting it to be the symbol.
**How to avoid:** Detect this case: if the outer variable name equals the inner variable name, it's the existing monomial-path (q^1 case), not a bivariate case. The current `extract_monomial_from_arg` already handles `Value::Symbol` as q^1, so this path is already correct. Only trigger bivariate when the symbol name differs from the q-variable.
**Warning signs:** `tripleprod(q, q, 10)` should continue to produce a univariate FPS, not a bivariate series.

### Pitfall 3: Inconsistent Truncation Orders in Bivariate Terms
**What goes wrong:** Different z-exponent coefficients end up with different truncation orders after arithmetic.
**Why it happens:** If two bivariate series are added and their FPS coefficients have different truncation orders.
**How to avoid:** All FPS coefficients in a BivariateSeries should share the same truncation order. When doing arithmetic, use min(T1, T2) across all terms and re-truncate.
**Warning signs:** Display showing different O(q^N) for different z-powers.

### Pitfall 4: Forgetting to Remove Zero FPS Entries
**What goes wrong:** After arithmetic (especially subtraction), some z-exponent entries become the zero series but remain in the BTreeMap.
**Why it happens:** FPS subtraction may cancel all terms.
**How to avoid:** After every arithmetic operation, filter out entries where the FPS `is_zero()`.
**Warning signs:** Display showing unnecessary `0*z^k` terms.

### Pitfall 5: Winquist Two-Variable Complexity
**What goes wrong:** Attempting to represent winquist(a, b, q, T) as a single BivariateSeries.
**Why it happens:** Winquist has TWO symbolic variables, not one.
**How to avoid:** Either (a) use a different data structure for two outer variables, or (b) represent as a nested structure. The simplest approach for Phase 45 is option (a): a dedicated `MultivariateSeries` with `BTreeMap<(i64, i64), FPS>` for the (a-exp, b-exp) -> q-coefficient mapping. Alternatively, limit Winquist to the case where only one of a, b is symbolic, reducing to BivariateSeries.
**Warning signs:** Compile errors trying to fit two-variable output into single-variable BivariateSeries.

### Pitfall 6: Arithmetic Between Bivariate Series with Different Outer Variables
**What goes wrong:** User tries `tripleprod(z, q, 10) + tripleprod(w, q, 10)` -- addition of series in different outer variables.
**Why it happens:** The BivariateSeries terms are keyed by different variables.
**How to avoid:** Assert outer variables match in arithmetic, or return a TypeError. This is analogous to how FPS arithmetic asserts same inner variable.
**Warning signs:** Silently incorrect results from adding z-powers and w-powers.

## Code Examples

### Example 1: BivariateSeries Construction (tripleprod)
```rust
// Jacobi triple product sum form:
// tripleprod(z, q, T) = sum_{n} z^n * q^{n(n+1)/2}
// where we include all n such that n(n+1)/2 < T

fn compute_tripleprod_bivariate(
    outer_var: &str,
    inner_var: SymbolId,
    truncation_order: i64,
) -> BivariateSeries {
    let mut terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();

    // Compute bound: n(n+1)/2 < T
    // For n >= 0: n < (-1 + sqrt(1 + 8T)) / 2
    // For n < 0: |n|(|n|-1)/2 < T (different formula due to n(n+1)/2 behavior)
    let bound = ((-1.0 + (1.0 + 8.0 * truncation_order as f64).sqrt()) / 2.0).ceil() as i64 + 1;

    for n in -bound..=bound {
        let q_exp = n * (n + 1) / 2;
        if q_exp < 0 || q_exp >= truncation_order {
            continue;
        }
        // Add contribution: z^n * q^{n(n+1)/2}
        let coeff_fps = terms.entry(n).or_insert_with(||
            FormalPowerSeries::zero(inner_var, truncation_order)
        );
        let old = coeff_fps.coeff(q_exp);
        coeff_fps.set_coeff(q_exp, old + QRat::one());
    }

    // Remove zero entries
    terms.retain(|_, fps| !fps.is_zero());

    BivariateSeries {
        outer_variable: outer_var.to_string(),
        terms,
        inner_variable: inner_var,
        truncation_order,
    }
}
```

### Example 2: BivariateSeries Arithmetic (add)
```rust
pub fn bivariate_add(a: &BivariateSeries, b: &BivariateSeries) -> BivariateSeries {
    assert_eq!(a.outer_variable, b.outer_variable);
    assert_eq!(a.inner_variable, b.inner_variable);
    let trunc = a.truncation_order.min(b.truncation_order);

    let mut result_terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();

    // Copy terms from a
    for (&z_exp, fps) in &a.terms {
        result_terms.insert(z_exp, fps.clone());
    }

    // Add terms from b
    for (&z_exp, fps_b) in &b.terms {
        let entry = result_terms.entry(z_exp).or_insert_with(||
            FormalPowerSeries::zero(a.inner_variable, trunc)
        );
        *entry = arithmetic::add(entry, fps_b);
    }

    // Remove zero entries
    result_terms.retain(|_, fps| !fps.is_zero());

    BivariateSeries {
        outer_variable: a.outer_variable.clone(),
        terms: result_terms,
        inner_variable: a.inner_variable,
        truncation_order: trunc,
    }
}
```

### Example 3: Quintuple Product Bivariate
```rust
// quinprod(z, q, T) = sum_{m} (z^{3m} - z^{-3m-1}) * q^{m(3m+1)/2}
fn compute_quinprod_bivariate(
    outer_var: &str,
    inner_var: SymbolId,
    truncation_order: i64,
) -> BivariateSeries {
    let mut terms: BTreeMap<i64, FormalPowerSeries> = BTreeMap::new();

    let bound = ((-1.0 + (1.0 + 24.0 * truncation_order as f64).sqrt()) / 6.0).ceil() as i64 + 1;

    for m in -bound..=bound {
        let q_exp = m * (3 * m + 1) / 2;
        if q_exp < 0 || q_exp >= truncation_order {
            continue;
        }

        // +1 contribution at z^{3m}
        let z_exp_pos = 3 * m;
        let entry = terms.entry(z_exp_pos).or_insert_with(||
            FormalPowerSeries::zero(inner_var, truncation_order)
        );
        let old = entry.coeff(q_exp);
        entry.set_coeff(q_exp, old + QRat::one());

        // -1 contribution at z^{-3m-1}
        let z_exp_neg = -3 * m - 1;
        let entry2 = terms.entry(z_exp_neg).or_insert_with(||
            FormalPowerSeries::zero(inner_var, truncation_order)
        );
        let old2 = entry2.coeff(q_exp);
        entry2.set_coeff(q_exp, old2 - QRat::one());
    }

    terms.retain(|_, fps| !fps.is_zero());

    BivariateSeries {
        outer_variable: outer_var.to_string(),
        terms,
        inner_variable: inner_var,
        truncation_order,
    }
}
```

### Example 4: Display Format
```rust
// Display: group by z-power descending
// "(q^3 + O(q^5))*z^2 + (q + O(q^5))*z + (1 + O(q^5)) + ..."
fn format_bivariate(bv: &BivariateSeries, symbols: &SymbolRegistry) -> String {
    let z = &bv.outer_variable;
    let mut parts = Vec::new();

    for (&z_exp, fps) in bv.terms.iter().rev() {
        let fps_str = format_series(fps, symbols);

        if z_exp == 0 {
            // Constant in z: just the q-series
            parts.push(fps_str);
        } else {
            let z_part = if z_exp == 1 {
                z.clone()
            } else if z_exp == -1 {
                format!("{}^(-1)", z)
            } else if z_exp < 0 {
                format!("{}^({})", z, z_exp)
            } else {
                format!("{}^{}", z, z_exp)
            };

            // If FPS is a single monomial, simplify display
            if fps.num_nonzero() == 1 {
                let (&k, c) = fps.iter().next().unwrap();
                // Format as coeff*q^k*z^exp or just q^k*z^exp if coeff==1
                parts.push(format!("{}*{}", format_monomial(c, k, symbols), z_part));
            } else {
                parts.push(format!("({})*{}", fps_str, z_part));
            }
        }
    }

    if parts.is_empty() {
        format!("O({}^{})", symbols.name(bv.inner_variable), bv.truncation_order)
    } else {
        parts.join(" + ") // Needs sign handling for negatives
    }
}
```

### Example 5: Value Enum Extension
```rust
pub enum Value {
    // ... existing variants ...
    /// Bivariate series: Laurent polynomial in outer variable with q-series coefficients.
    BivariateSeries(BivariateSeries),
}
```

### Example 6: Dispatch Detection
```rust
// In the tripleprod dispatch, the key change:
// Currently: Value::Symbol -> extract_monomial_from_arg treats it as q^1
// New: If Symbol name != inner variable name, produce bivariate
"tripleprod" => {
    if args.len() == 3 {
        let is_symbolic = match &args[0] {
            Value::Symbol(name) => {
                // Check if this symbol is the q-variable or a different variable
                match &args[1] {
                    Value::Symbol(q_name) => name != q_name,
                    _ => false,
                }
            }
            _ => false,
        };

        if is_symbolic {
            let outer_name = match &args[0] { Value::Symbol(s) => s.clone(), _ => unreachable!() };
            let sym = extract_symbol_id(name, args, 1, env)?;
            let order = extract_i64(name, args, 2)?;
            let result = compute_tripleprod_bivariate(&outer_name, sym, order);
            Ok(Value::BivariateSeries(result))
        } else {
            // Existing path: extract monomial
            let monomial = extract_monomial_from_arg(name, args, 0)?;
            let sym = extract_symbol_id(name, args, 1, env)?;
            let order = extract_i64(name, args, 2)?;
            let result = qseries::tripleprod(&monomial, sym, order);
            Ok(Value::Series(result))
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Symbolic z rejected | Symbolic z -> bivariate output | Phase 45 | Users can explore product identities with symbolic parameters |
| tripleprod only numeric z | tripleprod detects Symbol, routes to sum-form | Phase 45 | Full Maple compatibility for symbolic usage |
| No bivariate Value type | Value::BivariateSeries variant | Phase 45 | Enables bivariate arithmetic in REPL |

**Current state of the codebase:**
- tripleprod/quinprod: `extract_monomial_from_arg` converts `Value::Symbol` to `QMonomial::new(QRat::one(), 1)` (treats symbol as q^1). This is the numeric path.
- winquist: Similarly extracts monomials from both a and b arguments.
- No bivariate data type exists anywhere in the codebase.
- The Value enum has 13 variants; adding BivariateSeries would make 14.

## Mathematical Reference

### Jacobi Triple Product (tripleprod)
```
prod_{n>=1} (1-q^n)(1+zq^n)(1+z^{-1}q^{n-1}) = sum_{n=-inf}^{inf} z^n * q^{n(n+1)/2}
```
Note: The conventional form uses `(q; q)_inf * (-zq; q)_inf * (-z^{-1}; q)_inf`. The Garvan convention `(z; q)_inf * (q/z; q)_inf * (q;q)_inf` differs by sign/shift. The existing code uses the Garvan convention, so the sum form should match. For the Garvan convention:
```
tripleprod(z, q, T) = (q;q)_inf * (z;q)_inf * (q/z;q)_inf
= sum_{n=-inf}^{inf} (-1)^n * z^n * q^{n(n-1)/2}
```
This uses the Jacobi triple product in the form:
```
sum_{n=-inf}^{inf} (-1)^n * x^n * q^{n(n-1)/2} = (x;q)_inf * (q/x;q)_inf * (q;q)_inf
```

**Verification needed:** The exact sign convention and q-exponent formula. The sum involves `(-1)^n * z^n * q^{n(n-1)/2}`. For T=5:
- n=0: +1 (q^0, z^0)
- n=1: -z*q^0 = -z
- n=-1: -z^{-1}*q^1 = -z^{-1}*q
- n=2: +z^2*q^1 = z^2*q
- n=-2: +z^{-2}*q^3
- n=3: -z^3*q^3
- n=-3: -z^{-3}*q^6 (excluded, >= T=5)

Sanity check: setting z = q^m should recover the existing tripleprod. Substituting z = q^m: sum_n (-1)^n * q^{mn + n(n-1)/2}. This matches the known result.

### Quintuple Product (quinprod)
```
prod_{n>=1} (1-q^n)(1-zq^n)(1-z^{-1}q^{n-1})(1-z^2q^{2n-1})(1-z^{-2}q^{2n-1})
= sum_{m=-inf}^{inf} (z^{3m} - z^{-3m-1}) * q^{m(3m+1)/2}
```

### Winquist's Identity
Winquist's identity is a two-variable identity:
```
(q;q)_inf^2 * prod of 8 Pochhammer factors involving a, b
= sum_{r,s} epsilon(r,s) * a^r * b^s * q^{f(r,s)}
```
The exact sum form involves a double sum. The standard form from Winquist (1969):
```
prod_{n>=1} (1-q^n)^2 * (1-aq^n)(1-a^{-1}q^{n-1})(1-bq^n)(1-b^{-1}q^{n-1})
  * (1-abq^n)(1-a^{-1}b^{-1}q^{n-1})(1-ab^{-1}q^{n-1})(1-a^{-1}bq^n)
= sum_{m,n} epsilon(m,n) * a^{...} * b^{...} * q^{...}
```
The double-sum form involves pentagonal-number-like exponents. The exact formula requires careful verification.

## Open Questions

1. **Winquist two-variable representation**
   - What we know: Winquist involves TWO symbolic variables (a, b). The BivariateSeries struct handles ONE outer variable.
   - What's unclear: The exact double-sum form of Winquist's identity. Whether to use BTreeMap<(i64,i64), FPS> or nested BivariateSeries.
   - Recommendation: For Phase 45, implement a simple approach: a struct with `BTreeMap<(i64, i64), FormalPowerSeries>` mapping `(a_exp, b_exp) -> q_coeff`. Display as `coeff*a^r*b^s + ...`. If the double-sum is too complex to verify, defer Winquist bivariate to a follow-up and focus on tripleprod + quinprod first.

2. **Exact sign convention for tripleprod sum form**
   - What we know: The Garvan convention is `(z;q)_inf * (q/z;q)_inf * (q;q)_inf`. The Jacobi triple product identity gives `sum_n (-1)^n * z^n * q^{n(n-1)/2}`.
   - What's unclear: Need to verify this matches the existing numeric implementation at specific z values.
   - Recommendation: Implement, then validate by comparing `tripleprod_bivariate("z", q, 20)` evaluated at z=q^m against existing `tripleprod(q^m, q, 20)` for several values of m.

3. **Bivariate * Bivariate multiplication**
   - What we know: Multiplying two bivariate series requires convolving over z-exponents, where each "multiplication" is an FPS multiply.
   - What's unclear: Performance for large series (O(Z^2 * Q^2) where Z is number of z-terms and Q is truncation order).
   - Recommendation: Implement naively first. For typical uses (T <= 100), this will be fast enough.

4. **Arithmetic between Value::BivariateSeries and Value::Series**
   - What we know: A user might want to multiply a bivariate series by a q-series scalar.
   - What's unclear: Whether to auto-promote Series to BivariateSeries or handle as scalar multiplication.
   - Recommendation: Handle Series as a bivariate with only a z^0 term (or equivalently, scalar-multiply each FPS coefficient). Similarly, Integer/Rational multiply each coefficient.

## Sources

### Primary (HIGH confidence)
- Existing codebase: `crates/qsym-core/src/qseries/products.rs` -- current tripleprod/quinprod/winquist implementations
- Existing codebase: `crates/qsym-core/src/series/mod.rs` -- FormalPowerSeries structure
- Existing codebase: `crates/qsym-cli/src/eval.rs` -- Value enum, dispatch logic, arithmetic operators
- Existing codebase: `crates/qsym-cli/src/format.rs` -- display formatting patterns

### Secondary (MEDIUM confidence)
- [MathWorld: Jacobi Triple Product](https://mathworld.wolfram.com/JacobiTripleProduct.html) -- sum form: sum_{n=-inf}^{inf} x^n * q^{n(n+1)/2}
- [MathWorld: Quintuple Product Identity](https://mathworld.wolfram.com/QuintupleProductIdentity.html) -- sum form: sum_m (z^{3m} - z^{-3m-1}) * q^{m(3m+1)/2}
- [Garvan q-series tutorial](https://qseries.org/fgarvan/papers/qmaple.pdf) -- reference for naming conventions and Maple behavior

### Tertiary (LOW confidence)
- Winquist double-sum formula -- needs verification from primary mathematical reference. The sign pattern and exponent formula require careful derivation or lookup in Winquist (1969) or subsequent papers.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - BTreeMap<i64, FPS> is the natural data structure; no external dependencies needed
- Architecture: HIGH - Pattern follows existing Value/dispatch/format architecture exactly
- Pitfalls: HIGH - Based on direct codebase analysis of existing patterns
- Mathematical formulas (tripleprod, quinprod): MEDIUM - Standard identities but sign conventions need verification against Garvan's specific convention
- Mathematical formulas (winquist): LOW - Double-sum form not fully verified

**Research date:** 2026-02-20
**Valid until:** 2026-03-20 (stable mathematical domain; codebase patterns unlikely to change)
