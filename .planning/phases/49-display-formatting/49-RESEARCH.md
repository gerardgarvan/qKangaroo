# Phase 49: Display Formatting - Research

**Researched:** 2026-02-21
**Domain:** CLI display formatting for structured mathematical results
**Confidence:** HIGH

## Summary

Phase 49 requires replacing the raw `Value::Dict` display format for `qfactor` and `etamake` results with human-readable mathematical notation. Currently both functions return `Value::Dict` which renders as `{scalar: 1, factors: {1: 1, 2: 1}, is_exact: true}`. The goal is to display these as product-form notation like `(1-q)(1-q^2)(1-q^3)` for qfactor and `eta(tau)^(-2) * eta(2*tau)^5` for etamake.

The best approach is to introduce two new `Value` variants -- `Value::QProduct` and `Value::EtaQuotient` -- following the precedent set by `Value::JacobiProduct`. This is cleaner than modifying Dict display because: (1) it follows the existing codebase pattern where structured mathematical objects get their own variants, (2) it keeps `format_value` and `format_latex` dispatch clean via match arms, (3) it avoids fragile heuristic detection of "is this Dict a qfactor result?", and (4) it preserves the data for programmatic access. The conversion helpers `q_factorization_to_value` and `eta_quotient_to_value` simply change their return types.

**Primary recommendation:** Add `Value::QProduct` and `Value::EtaQuotient` variants to the Value enum, with dedicated format functions in `format.rs` for both plain-text and LaTeX output.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| FIX-03 | `qfactor` displays results in q-product form `(1-q^a)(1-q^b)...` instead of raw struct | New `Value::QProduct` variant with `format_qproduct()` and `format_qproduct_latex()` functions |
| FIX-04 | `etamake` displays results in eta(k*tau) notation instead of raw struct | New `Value::EtaQuotient` variant with `format_eta_quotient()` and `format_eta_quotient_latex()` functions |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| qsym-cli | local | CLI evaluator with Value enum, format.rs | The only display path |
| qsym-core | local | QFactorization, EtaQuotient structs | Source data types |

### Supporting
No new external dependencies. This is purely internal formatting work.

## Architecture Patterns

### Recommended Approach: New Value Variants

The codebase already has a precedent for structured mathematical display types:

- `Value::JacobiProduct(Vec<(i64, i64, i64)>)` -- added for `jacprodmake` results, displays as `JAC(1,5)*JAC(2,5)` instead of raw dict

Follow this pattern exactly for qfactor and etamake.

### Data Flow

```
qseries::qfactor() -> QFactorization -> q_factorization_to_value() -> Value::QProduct
                                                                          |
                                                              format_value() -> product string
                                                              format_latex() -> LaTeX string

qseries::etamake() -> EtaQuotient -> eta_quotient_to_value() -> Value::EtaQuotient
                                                                     |
                                                          format_value() -> eta string
                                                          format_latex() -> LaTeX string
```

### Files to Modify

```
crates/qsym-cli/src/
  eval.rs          # Add Value::QProduct, Value::EtaQuotient variants
                   # Update type_name(), q_factorization_to_value(), eta_quotient_to_value()
                   # Update arithmetic match arms (add/sub/mul/div helpful errors)
                   # Update tests that match on Value::Dict for these results
  format.rs        # Add format_qproduct(), format_qproduct_latex()
                   # Add format_eta_quotient(), format_eta_quotient_latex()
                   # Add match arms in format_value() and format_latex()
  help.rs          # Update example_output for qfactor and etamake
```

### Value::QProduct Design

```rust
/// Q-product factorization: scalar * prod (1-q^i)^{mult_i}
/// Stores factors as BTreeMap<i64, i64> (i -> multiplicity).
QProduct {
    /// Maps i -> multiplicity in prod (1-q^i)^{mult}
    factors: BTreeMap<i64, i64>,
    /// Scalar prefactor
    scalar: QRat,
    /// Whether factorization is exact
    is_exact: bool,
},
```

**Plain-text format:** `(1-q)(1-q^2)(1-q^3)^2` (ascending order by i)
- Scalar != 1: prefix with scalar, e.g. `3*(1-q)(1-q^2)`
- Multiplicity > 1: append exponent `(1-q^2)^3`
- Multiplicity < 0: append negative exponent `(1-q^2)^(-1)`
- Multiplicity == 1: no exponent `(1-q^2)`
- i == 1: `(1-q)` not `(1-q^1)`
- No factors: just the scalar (e.g. `1`)
- Not exact: append ` (approx)` marker

**LaTeX format:** `(1-q)(1-q^{2})(1-q^{3})^{2}`

### Value::EtaQuotient Design

```rust
/// Eta-quotient: prod eta(d*tau)^{r_d} * q^{q_shift}
EtaQuotient {
    /// Maps d -> r_d where result is prod eta(d*tau)^{r_d}
    factors: BTreeMap<i64, i64>,
    /// q-shift prefactor: sum_d r_d * d / 24
    q_shift: QRat,
},
```

**Plain-text format:** `eta(tau)^(-1)` or `eta(tau)^(-2) * eta(2*tau)^5 * eta(4*tau)^(-2)`
- d == 1: `eta(tau)` not `eta(1*tau)`
- d > 1: `eta(2*tau)`, `eta(3*tau)`, etc.
- Exponent == 1: `eta(tau)` not `eta(tau)^1`
- Exponent != 1: `eta(tau)^(-2)`, `eta(2*tau)^5`
- Multiple factors: joined by ` * `
- q_shift != 0: prepend `q^{shift} * ` (or `q^(a/b) * ` for rationals)
- No factors: display q_shift only, or `1` if both are trivial

**LaTeX format:** `\eta(\tau)^{-1}` or `\eta(\tau)^{-2} \cdot \eta(2\tau)^{5} \cdot \eta(4\tau)^{-2}`

### Pattern: Conversion Helper Changes

Current:
```rust
fn q_factorization_to_value(qf: &qseries::QFactorization) -> Value {
    let mut factor_entries: Vec<(String, Value)> = Vec::new();
    for (&i, &mult) in &qf.factors {
        factor_entries.push((i.to_string(), Value::Integer(QInt::from(mult))));
    }
    Value::Dict(vec![
        ("scalar".to_string(), Value::Rational(qf.scalar.clone())),
        ("factors".to_string(), Value::Dict(factor_entries)),
        ("is_exact".to_string(), Value::Bool(qf.is_exact)),
    ])
}
```

New:
```rust
fn q_factorization_to_value(qf: &qseries::QFactorization) -> Value {
    Value::QProduct {
        factors: qf.factors.clone(),
        scalar: qf.scalar.clone(),
        is_exact: qf.is_exact,
    }
}
```

Same pattern for `eta_quotient_to_value`.

### Anti-Patterns to Avoid

- **Heuristic Dict detection:** Do NOT try to detect "is this Dict a qfactor result?" by checking keys. This is fragile and error-prone.
- **Modifying Dict display globally:** Do NOT change how generic Dict values are rendered. Other functions (q_gosper, q_zeilberger, etc.) legitimately use Dict.
- **Losing data access:** The new variants must still allow programmatic field access if needed. The struct fields are public, so `match` on the variant works.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Exponent display | Custom string manipulation | Follow `format_z_power()` pattern in format.rs | Consistent with existing exponent formatting |
| Sign handling | Ad-hoc negative checks | Follow `format_series()` pattern for QRat sign detection | `QRat.0.cmp0()` is the standard way |

## Common Pitfalls

### Pitfall 1: Breaking Existing Tests
**What goes wrong:** Existing tests match on `Value::Dict` for qfactor/etamake results. Changing the return type breaks them.
**Why it happens:** Tests like `dispatch_qfactor_returns_dict_with_is_exact`, `dispatch_etamake_returns_dict`, `integration_qfactor_qbin` all pattern-match on `Value::Dict`.
**How to avoid:** Update all test assertions to match on the new variant types. Search for "qfactor" and "etamake" in eval.rs tests.
**Warning signs:** `cargo test` failures with "expected Dict, got ..." patterns.

### Pitfall 2: Missing Arithmetic Match Arms
**What goes wrong:** If someone tries `qfactor(f,q) + 1`, they get an unhelpful "cannot add" error without guidance.
**Why it happens:** New Value variants need match arms in `eval_binary_op` (add, sub, mul, div).
**How to avoid:** Add catch-all arms with helpful error messages, following the JacobiProduct pattern: "cannot add qproduct and integer -- result is a factorization, not a series".
**Warning signs:** Compile warnings about non-exhaustive match.

### Pitfall 3: Forgetting the LaTeX Path
**What goes wrong:** `\latex` command shows `\{scalar: ...\}` instead of nice notation.
**Why it happens:** `format_latex()` in format.rs needs match arms for the new variants.
**How to avoid:** Always update both `format_value()` AND `format_latex()` for new variants.

### Pitfall 4: Help Example Output Mismatch
**What goes wrong:** Help shows old format `{scalar: 1, factors: {...}}` while REPL shows new format.
**Why it happens:** `help.rs` has hardcoded `example_output` strings.
**How to avoid:** Update `example_output` in help.rs for `qfactor`, `etamake`, and possibly `qetamake`.

### Pitfall 5: Scalar Display Edge Cases
**What goes wrong:** Scalar = 1 shows `1*(1-q)` instead of `(1-q)`, or scalar = -1 shows `-1*(1-q)` instead of `-(1-q)`.
**How to avoid:** Special-case scalar == 1 (omit), scalar == -1 (prefix `-`), and other values (show as fraction if needed).

### Pitfall 6: Empty Factors Map
**What goes wrong:** Zero factors with scalar=5 should show `5`, not `5*` or empty product.
**How to avoid:** Check `factors.is_empty()` and handle the pure-scalar case.

## Code Examples

### Format function for QProduct (plain text)

```rust
/// Format a q-product factorization as human-readable string.
/// Example: "(1-q)(1-q^2)(1-q^3)^2" or "3*(1-q)(1-q^2)"
fn format_qproduct(factors: &BTreeMap<i64, i64>, scalar: &QRat, is_exact: bool) -> String {
    let mut out = String::new();

    // Handle scalar
    let scalar_is_one = scalar.0.numer().cmp0() != Ordering::Equal
        && *scalar.0.numer() == *scalar.0.denom();
    let scalar_is_neg_one = {
        let neg = -scalar.clone();
        neg.0.numer().cmp0() != Ordering::Equal
            && *neg.0.numer() == *neg.0.denom()
    };

    if factors.is_empty() {
        return format!("{}", scalar);
    }

    if scalar_is_neg_one {
        out.push('-');
    } else if !scalar_is_one {
        let _ = write!(out, "{}*", scalar);
    }

    // Factors in ascending order
    for (&i, &mult) in factors {
        if i == 1 {
            out.push_str("(1-q)");
        } else {
            let _ = write!(out, "(1-q^{})", i);
        }
        if mult != 1 {
            if mult < 0 {
                let _ = write!(out, "^({})", mult);
            } else {
                let _ = write!(out, "^{}", mult);
            }
        }
    }

    if !is_exact {
        out.push_str(" (approx)");
    }

    out
}
```

### Format function for EtaQuotient (plain text)

```rust
/// Format an eta-quotient as human-readable string.
/// Example: "eta(tau)^(-1)" or "q^(1/24) * eta(tau)^(-2) * eta(2*tau)^5"
fn format_eta_quotient(factors: &BTreeMap<i64, i64>, q_shift: &QRat) -> String {
    let mut parts = Vec::new();

    // q-shift prefix
    if !q_shift.is_zero() {
        if *q_shift.0.denom() == *rug::Integer::ONE {
            let n = q_shift.0.numer();
            if *n == *rug::Integer::ONE {
                parts.push("q".to_string());
            } else {
                parts.push(format!("q^{}", n));
            }
        } else {
            parts.push(format!("q^({}/{})", q_shift.0.numer(), q_shift.0.denom()));
        }
    }

    // Eta factors in ascending d order
    for (&d, &r_d) in factors {
        let eta_arg = if d == 1 {
            "tau".to_string()
        } else {
            format!("{}*tau", d)
        };

        if r_d == 1 {
            parts.push(format!("eta({})", eta_arg));
        } else {
            parts.push(format!("eta({})^({})", eta_arg, r_d));
        }
    }

    if parts.is_empty() {
        "1".to_string()
    } else {
        parts.join(" * ")
    }
}
```

### Updating type_name()

```rust
Value::QProduct { .. } => "qproduct",
Value::EtaQuotient { .. } => "eta_quotient",
```

### Arithmetic Error Pattern (following JacobiProduct)

```rust
// In eval_binary_op, add/sub/mul/div match:
(Value::QProduct { .. }, _) | (_, Value::QProduct { .. }) => {
    Err(EvalError::Other(format!(
        "cannot {} {} and {} -- qfactor result is a factorization, not a series",
        op_name, left.type_name(), right.type_name()
    )))
}
(Value::EtaQuotient { .. }, _) | (_, Value::EtaQuotient { .. }) => {
    Err(EvalError::Other(format!(
        "cannot {} {} and {} -- etamake result is an eta-quotient, not a series",
        op_name, left.type_name(), right.type_name()
    )))
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| All analysis results as Value::Dict | JacobiProduct got its own variant (v2.0) | Phase 33-40 | Precedent for this pattern |
| Raw dict display for qfactor/etamake | Will become formatted mathematical notation | Phase 49 | FIX-03, FIX-04 |

## Open Questions

1. **Should `qetamake` also get formatted display?**
   - What we know: `qetamake` returns a `QEtaForm` (similar structure to EtaQuotient: factors + q_shift). It currently returns `Value::Dict`.
   - What's unclear: The roadmap only mentions FIX-03 and FIX-04 (qfactor and etamake). But qetamake has nearly identical structure.
   - Recommendation: The planner should scope only to qfactor and etamake per requirements. A future phase could extend to qetamake if desired.

2. **Should `prodmake` also get formatted display?**
   - What we know: `prodmake` returns an `InfiniteProductForm` with exponents (BTreeMap<i64, QRat>). It could display as `prod (1-q^n)^{a_n}`.
   - What's unclear: Not in scope for FIX-03/FIX-04.
   - Recommendation: Out of scope for Phase 49.

3. **Should the new variants support subscript/field access?**
   - What we know: Currently there's no subscript operator for Dict values in the evaluator.
   - Recommendation: No. If users need the raw data, they can use the existing Dict-returning functions or we add accessor functions later.

## Sources

### Primary (HIGH confidence)
- `crates/qsym-cli/src/eval.rs` -- Value enum (line 63), QFactorization conversion (line 5130), EtaQuotient conversion (line 5081), arithmetic ops, tests
- `crates/qsym-cli/src/format.rs` -- format_value() (line 34), format_latex() (line 803), JacobiProduct formatting (line 62), all display functions
- `crates/qsym-core/src/qseries/factoring.rs` -- QFactorization struct (line 17): `factors: BTreeMap<i64, i64>`, `scalar: QRat`, `is_exact: bool`
- `crates/qsym-core/src/qseries/prodmake.rs` -- EtaQuotient struct (line 232): `factors: BTreeMap<i64, i64>`, `q_shift: QRat`
- `crates/qsym-cli/src/help.rs` -- example_output for qfactor (line 329), etamake (line 343)
- `.planning/REQUIREMENTS.md` -- FIX-03 (line 12), FIX-04 (line 13)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all code is local, fully inspected
- Architecture: HIGH -- follows established JacobiProduct precedent exactly
- Pitfalls: HIGH -- identified from direct code inspection of tests, match arms, and format paths

**Research date:** 2026-02-21
**Valid until:** Stable (internal codebase, no external dependency changes expected)
