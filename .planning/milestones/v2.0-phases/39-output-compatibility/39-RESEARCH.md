# Phase 39: Output & Compatibility - Research

**Researched:** 2026-02-19
**Domain:** Series display formatting + backward compatibility verification
**Confidence:** HIGH

## Summary

Phase 39 implements descending-power polynomial display and verifies all v1.x signatures remain working after the Phases 33-38 Maple compatibility changes. The work is straightforward refactoring with systematic test updates rather than algorithmic challenges.

The codebase already has clear separation: `format_series()` and `fps_to_latex()` in `format.rs` handle all series display, and `FormalPowerSeries::iter()` returns a BTreeMap iterator in ascending order. We reverse iteration order, update ~10-15 test assertions checking output strings, and add dedicated backward-compatibility integration tests for all 24+ functions with dual signatures.

**Primary recommendation:** Reverse iteration in both format functions using `.rev()`, update all test assertions proactively before the change, add new `backward_compat` integration test group verifying old signatures work.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Polynomial Display Ordering
- **Default ordering: descending (Maple-style)** -- highest power first for all series output
- Both plain text and LaTeX output use descending ordering
- No user toggle -- one default, always descending
- Applies to both infinite series (with O(q^T)) and exact polynomials
- POLYNOMIAL_ORDER sentinel behavior for truncation markers: Claude's discretion

#### Old Signature Handling
- **No deprecation warnings** -- old v1.x signatures work silently, both forms are first-class forever
- Replaced signatures (e.g., old 3-arg findprod): Claude decides whether to give helpful error or standard arg-count error
- **Verify ALL functions** that changed signatures: etaq, aqprod, jacprod, tripleprod, quinprod, winquist, qbin, sift, prodmake, etamake, jacprodmake, mprodmake, qetamake, qfactor, and all find* functions
- Old-style tests that no longer match: update to new Garvan-compatible signatures

#### Test Expectation Updates
- **Update all assertions** that check output ordering -- fix every test to match new descending format
- **Fix proactively** -- review and update integration tests checking output ordering BEFORE making the display change
- **Update roadmap success criteria** to current test counts (281 core + 549 CLI), not outdated v1.6 counts
- **Add dedicated backward-compat test group** -- new "backward_compat" section in cli_integration.rs testing all old-style function calls work correctly

### Claude's Discretion
- POLYNOMIAL_ORDER sentinel behavior for truncation marker display
- Whether replaced signatures get helpful migration messages or standard arg-count errors
- Test organization details within the backward_compat group

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| OUT-03 | Series display uses Maple-style polynomial ordering when appropriate | BTreeMap::iter() returns ascending, use .rev() to reverse. format_series() and fps_to_latex() are the only display functions. |
| COMPAT-01 | Existing v1.x function signatures continue to work as aliases (no breaking changes) | All 24+ changed functions already have dual dispatch branches in eval.rs (legacy branch preserves old signatures). Need integration tests. |
| COMPAT-02 | All existing tests pass with no regressions | Current test suite checks output format strings. Must update assertions before changing display code. Systematic grep for "assert.*q^" patterns. |

</phase_requirements>

## Standard Stack

### Core Libraries (Already in Use)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| std::collections::BTreeMap | stdlib | Stores FPS coefficients in sorted order | BTreeMap::iter() guarantees ascending key order; .rev() gives descending |
| format! / write! macros | stdlib | String building for series display | Zero-allocation string formatting for term construction |

### No New Dependencies Required

Phase 39 is pure refactoring of existing code. All functionality uses existing Rust standard library.

## Architecture Patterns

### Current Display Architecture

```
User → format_value() → format_series() → String
                      → fps_to_latex() → String

FormalPowerSeries {
    coefficients: BTreeMap<i64, QRat>,  // Sorted by exponent
    variable: SymbolId,
    truncation_order: i64,
}

impl FormalPowerSeries {
    pub fn iter(&self) -> impl Iterator<Item = (&i64, &QRat)> {
        self.coefficients.iter()  // Ascending order guaranteed
    }
}
```

Location: `crates/qsym-cli/src/format.rs`

### Pattern 1: Reversing BTreeMap Iteration

**What:** BTreeMap provides `.iter().rev()` to reverse iteration order
**When to use:** When displaying polynomials in descending power order (Maple convention)

**Example:**
```rust
// Current (ascending): 1 + 2*q + q^2
for (&k, c) in fps.iter() {
    format_term(&mut out, k, c, var);
}

// New (descending): q^2 + 2*q + 1
for (&k, c) in fps.iter().rev() {
    format_term(&mut out, k, c, var);
}
```

**Edge case:** First term logic must handle sign correctly regardless of iteration order.

### Pattern 2: Dual Signature Dispatch (Already Implemented)

**What:** Functions detect argument types/counts to route to Garvan vs. legacy implementation
**When to use:** Every function changed in Phases 33-38

**Current structure in eval.rs:**
```rust
"aqprod" => {
    if !args.is_empty() && matches!(&args[0], Value::Series(_) | Value::Symbol(_)) {
        // Garvan: aqprod(q^2, q, n[, order])
        // ...
    } else {
        // Legacy: aqprod(coeff_num, coeff_den, power, n_or_infinity, order)
        // ...
    }
}
```

**All 24+ functions have this structure:**
- etaq (line 1745)
- aqprod (line 1657)
- jacprod (line 1787)
- tripleprod, quinprod, winquist, qbin (lines 1800-1900)
- sift (line ~2100)
- prodmake, etamake, jacprodmake, mprodmake, qetamake, qfactor (lines 2200-2500)
- 11 find* functions (lines 2600-3400)

**Verification strategy:** Integration tests call both forms, assert identical results.

### Pattern 3: Test Assertion Updates

**What:** Replace ascending-order assertions with descending-order
**When to use:** Before changing display code (fail-first development)

**Example transformations:**
```rust
// Old assertion
assert!(stdout.contains("1 + q + 2*q^2 + q^3 + q^4"));

// New assertion (descending)
assert!(stdout.contains("q^4 + q^3 + 2*q^2 + q + 1"));
```

**Systematic approach:**
1. Grep for all `assert.*q^` patterns in test files
2. Manually inspect each, determine expected descending form
3. Update assertion before changing display code
4. Verify tests fail with old display, pass with new display

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Reverse iteration | Custom descending Vec | BTreeMap::iter().rev() | Built-in, zero-cost, guarantees sorted order |
| String formatting | Manual char-by-char | format! / write! macros | Compiler-optimized, handles all edge cases |
| Backward compat detection | Runtime flags/config | Type-based dispatch | Zero runtime cost, compile-time checked |

**Key insight:** BTreeMap provides ordering guarantees that make reversing trivial. Don't collect into Vec, don't sort manually.

## Common Pitfalls

### Pitfall 1: Off-by-One in "First Term" Logic

**What goes wrong:** When reversing iteration, the first term displayed is now the highest power, but sign logic assumes lowest power first.

**Why it happens:** Current code:
```rust
let mut first = true;
for (&k, c) in fps.iter() {
    if first {
        if is_negative { out.push('-'); }
        // ... no leading "+"
        first = false;
    } else {
        // ... print " + " or " - "
    }
}
```

After reversing, the "first" term is now q^N, not constant, but sign logic is identical.

**How to avoid:** Test with series containing negative leading coefficients (e.g., -q^3 + 2*q - 1). Verify no double-negative or missing signs.

**Warning signs:** Test failures with messages like "assertion failed: -q^2 + 1" when expecting "q^2 - 1" (sign handling).

### Pitfall 2: Truncation Order Placement

**What goes wrong:** After reversing, O(q^T) should still appear at the end (right side) of the output, not at the beginning.

**Why it happens:** Truncation logic is outside the loop:
```rust
for (&k, c) in fps.iter().rev() { ... }
if !is_polynomial {
    write!(out, " + O({}^{})", var, trunc);
}
```

This is correct -- order doesn't affect truncation placement. But developer might assume reversal changes where O(...) appears.

**How to avoid:** Verify O(q^T) is always appended after the loop, regardless of iteration direction.

**Warning signs:** Tests expecting "q^2 + q + 1 + O(q^5)" fail because output is "O(q^5) + q^2 + q + 1".

### Pitfall 3: Forgetting LaTeX Formatting

**What goes wrong:** Update `format_series()` for plain text but forget `fps_to_latex()` uses separate code path.

**Why it happens:** Two independent functions format series:
- `format_series()` at line 125 (plain text)
- `fps_to_latex()` at line 267 (LaTeX)

Both iterate over coefficients and must be updated identically.

**How to avoid:** Apply `.rev()` to both `fps.iter()` calls (line 132, line 271). Update both `latex_term()` calls for first-term logic if needed.

**Warning signs:** Plain text output is descending, but LaTeX output (`latex` command in REPL) is still ascending.

### Pitfall 4: Backward-Compat False Positives

**What goes wrong:** Integration test calls legacy signature but gets a different error, test passes incorrectly.

**Why it happens:** Test checks "did it run without error" but doesn't verify correctness:
```rust
let (code, stdout, _) = run(&["-c", "etaq(1, 20)"]);
assert_eq!(code, 0);  // Passes, but result might be wrong
```

**How to avoid:** Backward-compat tests must assert output correctness, not just success:
```rust
let (code, stdout, _) = run(&["-c", "etaq(1, 20)"]);
assert_eq!(code, 0);
assert!(stdout.contains("O(q^20)"));  // Verify truncation
// Could also assert specific coefficient if known
```

**Warning signs:** All backward-compat tests pass, but manual REPL testing shows wrong results.

### Pitfall 5: Missing Test Coverage for Replaced Signatures

**What goes wrong:** A function like `findprod` had signature changes in Phase 38, but no test verifies the old 3-arg form still works.

**Why it happens:** Phase 38 changed `findprod` from 3-arg to 4-arg Garvan signature. Old 3-arg form might not have legacy branch implemented.

**How to avoid:** Systematically verify EVERY function in the "changed signatures" list has:
1. Dual dispatch branch in eval.rs
2. Integration test for old signature
3. Integration test for new signature

**Warning signs:** grep "findprod.*=>" shows dispatch code, but no legacy branch exists for 3-arg form.

## Code Examples

Verified patterns from codebase:

### Descending Iteration Pattern

```rust
// Source: crates/qsym-cli/src/format.rs (line 125, modified)
fn format_series(fps: &FormalPowerSeries, symbols: &SymbolRegistry) -> String {
    let var = symbols.name(fps.variable());
    let trunc = fps.truncation_order();
    let is_polynomial = trunc >= POLYNOMIAL_ORDER;
    let mut first = true;
    let mut out = String::new();

    // CHANGE: Add .rev() here
    for (&k, c) in fps.iter().rev() {
        let is_negative = c.0.cmp0() == Ordering::Less;
        let abs_c = if is_negative { -c.clone() } else { c.clone() };

        // First term logic (handles sign correctly regardless of power)
        if first {
            if is_negative { out.push('-'); }
            // ... format coefficient and variable
            first = false;
        } else {
            // Subsequent terms (always with explicit sign)
            if is_negative {
                write!(out, " - ");
            } else {
                write!(out, " + ");
            }
            // ... format coefficient and variable
        }
    }

    // Truncation marker appended AFTER loop (unchanged)
    if !is_polynomial {
        if first {
            write!(out, "O({}^{})", var, trunc);
        } else {
            write!(out, " + O({}^{})", var, trunc);
        }
    } else if first {
        out.push('0');
    }

    out
}
```

### LaTeX Descending Iteration

```rust
// Source: crates/qsym-cli/src/format.rs (line 267, modified)
fn fps_to_latex(fps: &FormalPowerSeries, symbols: &SymbolRegistry) -> String {
    let var = symbols.name(fps.variable());
    let trunc = fps.truncation_order();
    let is_polynomial = trunc >= POLYNOMIAL_ORDER;
    let terms: Vec<(&i64, &QRat)> = fps.iter().collect();
    let total = terms.len();

    if total == 0 {
        if is_polynomial { return "0".to_string(); }
        return format!("O({}^{{{}}})", var, trunc);
    }

    // Determine which terms to show (ellipsis logic)
    let (show_first, show_last, ellipsis) = if total > 20 {
        (15, 2, true)
    } else {
        (total, 0, false)
    };

    let mut result = String::new();

    // CHANGE: Reverse iteration for first group
    for (i, (k, c)) in terms.iter().rev().take(show_first).enumerate() {
        latex_term(&mut result, i == 0, **k, c, var);
    }

    // Ellipsis and last terms (also reversed)
    if ellipsis {
        write!(result, " + \\cdots");
        let start = total - show_last;
        for (k, c) in terms[start..].iter().rev() {
            latex_term(&mut result, false, **k, c, var);
        }
    }

    // Truncation order (unchanged)
    if !is_polynomial {
        write!(result, " + O({}^{{{}}})", var, trunc);
    }

    result
}
```

**Note:** LaTeX ellipsis logic is more complex -- may need adjustment to show "highest 15 terms + ... + lowest 2 terms" rather than reversed chunks.

### Backward-Compat Integration Test Pattern

```rust
// Source: crates/qsym-cli/tests/cli_integration.rs (new section)
// ===========================================================================
// Backward Compatibility (COMPAT-01, COMPAT-02)
// ===========================================================================

#[test]
fn backward_compat_etaq_legacy_signature() {
    // Old v1.x: etaq(b, t, order) -- no explicit q
    let (code, stdout, stderr) = run(&["-c", "etaq(1, 1, 20)"]);
    assert_eq!(code, 0, "legacy etaq(1, 1, 20) should succeed. stderr: {}", stderr);
    assert!(stdout.contains("O(q^20)"), "should have truncation at 20");
}

#[test]
fn backward_compat_aqprod_legacy_signature() {
    // Old v1.x: aqprod(cn, cd, power, n_or_infinity, order)
    let (code, stdout, stderr) = run(&["-c", "aqprod(1, 1, 2, 5, 20)"]);
    assert_eq!(code, 0, "legacy aqprod(1,1,2,5,20) should succeed. stderr: {}", stderr);
    assert!(stdout.contains("O(q^20)"), "should have truncation");
}

#[test]
fn backward_compat_qbin_legacy_signature() {
    // Old v1.x: qbin(n, k, order)
    let (code, stdout, stderr) = run(&["-c", "qbin(4, 2, 20)"]);
    assert_eq!(code, 0, "legacy qbin(4,2,20) should succeed. stderr: {}", stderr);
    // qbin(4,2) is exact polynomial, no O(...)
    assert!(!stdout.contains("O(q^"), "exact polynomial should not have truncation");
}

// ... repeat for all 24+ changed functions
```

### Test Assertion Update Pattern

```rust
// Before (ascending order)
#[test]
fn format_polynomial_no_truncation() {
    let (reg, sym_q) = q_reg();
    let mut coeffs = std::collections::BTreeMap::new();
    coeffs.insert(0, QRat::one());
    coeffs.insert(1, QRat::from((2i64, 1i64)));
    coeffs.insert(2, QRat::one());
    let fps = FormalPowerSeries::from_coeffs(sym_q, coeffs, POLYNOMIAL_ORDER);
    let val = Value::Series(fps);
    let result = format_value(&val, &reg);
    assert_eq!(result, "1 + 2*q + q^2");  // OLD: ascending
}

// After (descending order)
#[test]
fn format_polynomial_no_truncation() {
    let (reg, sym_q) = q_reg();
    let mut coeffs = std::collections::BTreeMap::new();
    coeffs.insert(0, QRat::one());
    coeffs.insert(1, QRat::from((2i64, 1i64)));
    coeffs.insert(2, QRat::one());
    let fps = FormalPowerSeries::from_coeffs(sym_q, coeffs, POLYNOMIAL_ORDER);
    let val = Value::Series(fps);
    let result = format_value(&val, &reg);
    assert_eq!(result, "q^2 + 2*q + 1");  // NEW: descending
}
```

## State of the Art

### Current Output (Ascending)

```
> qbin(4, 2, q, 10)
1 + q + 2*q^2 + q^3 + q^4

> etaq(q, 1, 20)
q^(1/24) - q^(25/24) - q^(49/24) + q^(121/24) + O(q^20)
```

### Target Output (Descending - Maple Style)

```
> qbin(4, 2, q, 10)
q^4 + q^3 + 2*q^2 + q + 1

> etaq(q, 1, 20)
q^(121/24) - q^(49/24) - q^(25/24) + q^(1/24) + O(q^20)
```

### Implementation Timeline

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Ascending polynomial display | Descending (Maple-style) | Phase 39 | All test assertions updated, LaTeX output also descending |
| Single signature per function | Dual dispatch (Garvan + legacy) | Phases 33-38 | Old signatures preserved forever, no breaking changes |
| Implicit q in all functions | Explicit q parameter | Phases 34-38 | Maple compatibility, but legacy forms still work |

**Deprecated/outdated:**
- None -- this phase preserves all old functionality while adding Maple compatibility

## Open Questions

1. **LaTeX Ellipsis Ordering**
   - What we know: Current code shows "first 15 + ... + last 2" in ascending order
   - What's unclear: Should reversed display show "highest 15 + ... + lowest 2" or "lowest 15 + ... + highest 2"?
   - Recommendation: Match Maple's behavior if documented; otherwise show highest terms first (most significant information first)

2. **POLYNOMIAL_ORDER Display Behavior**
   - What we know: POLYNOMIAL_ORDER (1 billion) sentinel suppresses O(...) truncation marker
   - What's unclear: Should exact polynomials always omit O(...), or should there be a way to show "q^2 + 1 + O(q^1000000000)"?
   - Recommendation: Always suppress for POLYNOMIAL_ORDER -- users expect "q^2 + 1" for exact polynomials, not "q^2 + 1 + O(q^1000000000)"

3. **Test Count Verification**
   - What we know: Verified via `cargo test`: 281 core + 418 CLI unit + 131 CLI integration = 830 total tests
   - What's unclear: N/A -- counts confirmed
   - Recommendation: Update roadmap success criteria to "281 core + 418 CLI unit + 131 CLI integration (830 total)" for accuracy

## Sources

### Primary (HIGH confidence)

- **Codebase inspection**: `crates/qsym-cli/src/format.rs` lines 125-196 (format_series), 267-362 (fps_to_latex)
- **Codebase inspection**: `crates/qsym-cli/src/eval.rs` lines 1650-3600 (function dispatch with dual signatures)
- **Codebase inspection**: `crates/qsym-core/src/series/mod.rs` lines 1-156 (FormalPowerSeries structure)
- **Codebase inspection**: `crates/qsym-cli/tests/cli_integration.rs` (integration test patterns)
- **Official Rust docs**: std::collections::BTreeMap iter() guarantees ascending order, rev() reverses

### Secondary (MEDIUM confidence)

- **CONTEXT.md decisions**: User confirmed descending order, no toggle, no deprecation warnings
- **REQUIREMENTS.md**: OUT-03, COMPAT-01, COMPAT-02 requirement text

### Tertiary (LOW confidence)

- None -- all findings verified with codebase inspection or official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - stdlib only, no new dependencies
- Architecture: HIGH - codebase inspection shows clear patterns
- Pitfalls: HIGH - common test update and iteration reversal issues well-understood

**Research date:** 2026-02-19
**Valid until:** 2026-03-21 (30 days for stable codebase)
