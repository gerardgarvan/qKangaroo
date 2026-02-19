# Phase 36: Relation Discovery Signatures - Research

**Researched:** 2026-02-19
**Domain:** CLI function dispatch -- Maple-compatible calling conventions for relation discovery functions
**Confidence:** HIGH

## Summary

Phase 36 migrates 11 relation discovery functions from their current compact signatures to Garvan's exact Maple calling conventions. The Garvan signatures were verified against the actual Maple source code (`wmprog64.txt` from qseries v1.2) and the official function reference pages at qseries.org.

**Critical finding:** The requirements document (SIG-15 through SIG-25) contains several discrepancies with Garvan's actual Maple signatures. Specifically: (1) `findhomcombo`, `findnonhomcombo`, and `findhomcombomodp` do NOT have an SL (symbolic label list) parameter in Garvan -- only `findlincombo` and `findlincombomodp` accept SL; (2) the parameter ordering for modp variants has `p` before `q` in Garvan, not `q` before `p` as stated in some requirements; (3) `findcong` in Garvan takes `(QS, T, [LM], [XSET])` where T is a truncation order and it automatically searches all moduli 2..floor(sqrt(T)), which is fundamentally different from our current `(series, [moduli])` implementation; (4) `findpoly` has an optional `check` parameter for verification, not `topshift`; (5) `findmaxind` takes `(XFL, T)` with no explicit `q` parameter -- T is passed to `findhom` internally.

**Primary recommendation:** Follow the Garvan actual signatures faithfully (they are the ground truth), not the requirements as written. The requirements attempted to impose SL on functions that don't have it in Garvan. Functions without SL use auto-generated `X[1], X[2], ...` labels (or `L1, L2, ...` in our notation). The findcong implementation needs significant rework to match Garvan's automatic modulus-scanning behavior. Apply the Phase 35 pattern: remove old signatures completely, update dispatch/help/tests.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- SL parameters are passed as bare symbols using Value::Symbol from Phase 33 (e.g., `[F1, F2, F3]`)
- SL labels must be unique -- error on duplicate labels to prevent confusing output
- findlincombo and similar functions print formatted strings: `12*F1 + 13*F2` -- matches Maple's display-oriented output
- Congruence results displayed as list `[B, A, R]` triples -- matches Garvan exactly
- When multiple congruences found, print each `[B, A, R]` on its own line
- When no combination/relation found: print a message (e.g., "no combination found") and return successfully (exit 0)
- findpoly follows the same pattern: print "no polynomial relation found" message on failure
- For modp variants: validate that p is prime at dispatch time, error on non-prime

### Claude's Discretion
- SL length validation behavior (strict match vs lenient truncate/pad)
- Auto-generated labels for functions without SL (L1, L2... vs numeric indices)
- Modular arithmetic display conventions (match Garvan)
- Homogeneous combo polynomial display (match Garvan)
- Multiple solution handling (match Garvan)
- findcong overload strategy (arg-count dispatch vs optional defaults)
- Short-list-for-degree validation approach

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SIG-15 | `findlincombo(f, L, SL, q, topshift)` matches Garvan's 5-arg signature including SL | Verified from Garvan source: `proc(f,L,SL,q,topshift)`. MATCHES. |
| SIG-16 | `findhomcombo(f, L, SL, q, n, topshift)` matches Garvan's signature with SL | **DISCREPANCY:** Garvan actual: `proc(f,L,q,n,topshift,etaoption)`. NO SL parameter. Uses X[1],X[2],... labels. Implement Garvan's actual signature without SL. |
| SIG-17 | `findnonhomcombo(f, L, SL, q, n, topshift)` matches Garvan's signature with SL | **DISCREPANCY:** Garvan actual: `proc()` with args `(f,L,q,n,[topshift],[etaoption])`. NO SL parameter. Uses X[i] labels. |
| SIG-18 | `findlincombomodp(f, L, SL, q, p, topshift)` matches Garvan's signature with SL and prime p | **ORDERING:** Garvan actual: `proc(f,L,SL,p,q,topshift)`. Has SL but p comes BEFORE q. Use Garvan's ordering. |
| SIG-19 | `findhomcombomodp(f, L, SL, q, p, n, topshift)` matches Garvan's signature | **DISCREPANCY:** Garvan actual: `proc(f,L,p,q,n,topshift,etaoption)`. NO SL parameter. p before q. |
| SIG-20 | `findhom(L, q, n, topshift)` matches Garvan's signature | Verified: `proc(L,q,n,topshift)`. MATCHES. |
| SIG-21 | `findnonhom(L, q, n, topshift)` matches Garvan's signature | Verified: `proc(L,q,n,topshift)`. MATCHES. |
| SIG-22 | `findhommodp(L, q, p, n, topshift)` matches Garvan's signature | **ORDERING:** Garvan actual: `proc(L,p,q,n,topshift)`. p comes BEFORE q. Use Garvan's ordering. |
| SIG-23 | `findmaxind(L, q, topshift)` matches Garvan's signature | **DISCREPANCY:** Garvan docs: `findmaxind(XFL, T)`. Only 2 args: list + integer T (topshift). No q parameter. Not in wmprog64.txt source. |
| SIG-24 | `findpoly(f, g, q, dx, dy, topshift)` matches Garvan's signature | **DISCREPANCY:** Garvan actual: `findpoly(x, y, q, deg1, deg2, [check])`. 6th arg is optional `check` for verification, not `topshift`. Matrix uses fixed +10 buffer. |
| SIG-25 | `findcong(QS, T)` and overloads match Garvan's signatures | **MAJOR REWORK:** Garvan: `findcong(QS, T, [LM], [XSET])`. T=truncation, auto-scans moduli 2..LM, LM defaults to floor(sqrt(T)). Completely different from current `(series, [moduli])`. |
| OUT-01 | Relation discovery functions print results using SL matching Maple's output | Only findlincombo and findlincombomodp use SL. Others use X[i] / auto-generated labels. Output format: `12*F1 + 13*F2` for linear combos; polynomial expressions for hom/nonhom. |
| OUT-02 | findcong output format matches Garvan's [B, A, R] triple format | Garvan prints `[r, M, P^R]` which is `[B, A, R]`. Each triple on its own line. R is a prime power from GCD factorization. |
</phase_requirements>

## Garvan's Actual Maple Signatures (Verified)

Verified from Garvan's actual Maple source code (`wmprog64.txt`, qseries v1.2) and official function reference pages at qseries.org. Confidence: HIGH.

### findlincombo
```maple
qseries[findlincombo]:=proc(f, L, SL, q, topshift)
```
- **f**: target q-series to express as linear combination
- **L**: list of basis q-series
- **SL**: list of symbolic names for printing (e.g., `[F1, F2, F3]`)
- **q**: the q variable
- **topshift**: integer (typically 0), extra rows for overdetermination
- **Returns**: symbolic expression `ANS := add(-goo[k]/goo[nx+1]*SL[k], k=1..nx)`
- **Output**: `12*F1 + 13*F2` format; prints "NOT A LINEAR COMBO." on failure

### findhomcombo
```maple
qseries[findhomcombo]:=proc(f, L, q, n, topshift, etaoption)
```
- **f**: target q-series
- **L**: list of basis q-series
- **q**: the q variable
- **n**: degree of homogeneous polynomial
- **topshift**: integer (typically 0)
- **etaoption**: `yes` or `no` for eta-product form (optional, we skip this)
- **NO SL parameter** -- uses auto-generated `X[1], X[2], ...` labels
- **Returns**: set of polynomial expressions in X[i] variables

### findnonhomcombo
```maple
qseries[findnonhomcombo]:=proc()  -- variable args
```
- Args: `(f, L, q, n, [topshift], [etaoption])`
- Valid arg counts: 4, 5, or 6
- 4 args: `(f, L, q, n)` with topshift=0, etaoption=no
- 5 args: `(f, L, q, n, topshift)` or `(f, L, q, n, etaoption)` (disambiguated by type)
- 6 args: `(f, L, q, n, topshift, etaoption)`
- **NO SL parameter** -- uses X[i] labels

### findlincombomodp
```maple
qseries[findlincombomodp]:=proc(f, L, SL, p, q, topshift)
```
- Same as findlincombo but with p (prime) before q
- **NOTE:** p is BEFORE q in Garvan's signature (not q before p as in requirements)
- **Has SL** -- same label mechanism as findlincombo

### findhomcombomodp
```maple
qseries[findhomcombomodp]:=proc(f, L, p, q, n, topshift, etaoption)
```
- Same as findhomcombo but with p before q
- **NO SL parameter** -- uses X[i] labels
- **NOTE:** p before q

### findhom
```maple
qseries[findhom]:=proc(L, q, n, topshift)
```
- **L**: list of q-series
- **q**: the q variable
- **n**: degree of homogeneous relations
- **topshift**: integer (typically 0)
- **Returns**: set of polynomial relations in X[i] variables

### findnonhom
```maple
qseries[findnonhom]:=proc(L, q, n, topshift)
```
- Same structure as findhom but finds non-homogeneous relations (degree 0..n)

### findhommodp
```maple
qseries[findhommodp]:=proc(L, p, q, n, topshift)
```
- Same as findhom but modular, with p BEFORE q

### findmaxind
```
findmaxind(XFL, T)  -- from docs only, NOT in wmprog64.txt source
```
- **XFL**: list of q-series
- **T**: nonneg integer passed to findhom (essentially topshift)
- **Returns**: `[P, NXFL]` where P is independent subset, NXFL is indices list
- No q parameter in docs

### findpoly
```maple
qseries[findpoly]:=proc()  -- 5 or 6 args
```
- Args: `(x, y, q, deg1, deg2, [check])`
- **check**: optional verification precision (NOT topshift)
- Matrix uses fixed `dim2 := dim1 + 10` buffer (no configurable topshift)
- **Returns**: polynomial in X, Y; prints "NO polynomial relation found." on failure

### findcong
```maple
qseries[findcong]:=proc()  -- 2, 3, or 4 args
```
- Args: `(QS, T, [LM], [XSET])`
- **QS**: q-series (polynomial in q with integer coefficients)
- **T**: truncation upper bound (how many coefficients to examine)
- **LM**: max modulus to search (default: `trunc(sqrt(T))`)
- **XSET**: set of excluded moduli (default: `{}`)
- Automatically searches ALL moduli M from 2 to LM
- For each M, tests all residues r = 0..M-1
- Computes GCD of `{c(Mn+r) : n=0..floor((T-r)/M)}`
- Factors GCD, reports prime power divisors not in XSET
- **Prints**: `[r, M, P^R]` for each congruence found (B=r, A=M, R=P^R)

## Current Implementation (What Needs to Change)

### Current Signatures (eval.rs, lines 1647-1827)

| Function | Current Signature | Current Args | Target Garvan Signature | Target Args |
|----------|------------------|-------------|------------------------|-------------|
| findlincombo | `(target, [candidates], topshift)` | 3 | `(f, L, SL, q, topshift)` | 5 |
| findhomcombo | `(target, [candidates], degree, topshift)` | 4 | `(f, L, q, n, topshift)` | 5 (skip etaoption) |
| findnonhomcombo | `(target, [candidates], degree, topshift)` | 4 | `(f, L, q, n, topshift)` | 5 (skip etaoption) |
| findlincombomodp | `(target, [candidates], p, topshift)` | 4 | `(f, L, SL, p, q, topshift)` | 6 |
| findhomcombomodp | `(target, [candidates], p, degree, topshift)` | 5 | `(f, L, p, q, n, topshift)` | 6 (skip etaoption) |
| findhom | `([series], degree, topshift)` | 3 | `(L, q, n, topshift)` | 4 |
| findnonhom | `([series], degree, topshift)` | 3 | `(L, q, n, topshift)` | 4 |
| findhommodp | `([series], p, degree, topshift)` | 4 | `(L, p, q, n, topshift)` | 5 |
| findmaxind | `([series], topshift)` | 2 | `(L, topshift)` | 2 (no q in Garvan docs) |
| findpoly | `(x, y, deg_x, deg_y, topshift)` | 5 | `(x, y, q, deg1, deg2, [check])` | 5 or 6 |
| findcong | `(series, [moduli])` | 2 | `(QS, T, [LM], [XSET])` | 2-4 |

### Key Changes Required

**Group 1: Add SL + q parameter (2 functions)**
- findlincombo: 3 args -> 5 args. Add SL (list of symbols) and q. Change return to formatted string.
- findlincombomodp: 4 args -> 6 args. Add SL, reorder p before q. Change return to formatted string.

**Group 2: Add q parameter only, no SL (5 functions)**
- findhomcombo: 4 args -> 5 args. Add q, no SL (use X[i] labels). Return set of polynomial expressions.
- findnonhomcombo: 4 args -> 5 args. Add q, no SL. Return set of polynomial expressions.
- findhomcombomodp: 5 args -> 6 args. Add q, reorder p before q. Return set of polynomial expressions mod p.
- findhom: 3 args -> 4 args. Add q. Return set of polynomial relations.
- findnonhom: 3 args -> 4 args. Add q. Return set of polynomial relations.

**Group 3: Add q + reorder (1 function)**
- findhommodp: 4 args -> 5 args. Add q, keep p before q. Return set of modular polynomial relations.

**Group 4: findpoly changes (1 function)**
- findpoly: 5 args -> 5-6 args. Add q, replace topshift with optional check. Output formatted polynomial.

**Group 5: findmaxind minimal change (1 function)**
- findmaxind: Keep 2 args `(L, T)`. No q per Garvan docs. BUT return format changes to `[P, NXFL]` pair.

**Group 6: findcong major rework (1 function)**
- findcong: Complete signature change. New: `(QS, T, [LM], [XSET])`. Auto-scan moduli 2..LM. Factor GCD to find prime power divisors. Output `[B, A, R]` triples.

## Architecture Patterns

### Recommended Project Structure

No new files. All changes in existing files:
```
crates/qsym-cli/src/
  eval.rs       # Dispatch changes (~400 lines changed/added)
  help.rs       # Help text updates (~100 lines changed)
  format.rs     # Possibly: polynomial expression formatter helper
crates/qsym-core/src/qseries/
  relations.rs  # findcong needs rework, generate_monomials needs pub visibility
```

### Pattern 1: SL-based Output (findlincombo, findlincombomodp)

**What:** Functions with SL parameter format results as symbolic linear combinations.
**When to use:** Only for findlincombo and findlincombomodp.

```rust
// Extract SL as list of symbol strings
fn extract_symbol_list(name: &str, args: &[Value], index: usize) -> Result<Vec<String>, EvalError> {
    match &args[index] {
        Value::List(items) => {
            let mut labels = Vec::new();
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::Symbol(s) => labels.push(s.clone()),
                    _ => return Err(EvalError::Other(format!(
                        "{}: Argument {} (SL): element {} must be a symbol, got {}",
                        name, index + 1, i + 1, item.type_name()
                    ))),
                }
            }
            Ok(labels)
        }
        _ => Err(EvalError::ArgType { ... }),
    }
}

// Format linear combination with labels
fn format_linear_combo(coeffs: &[QRat], labels: &[String]) -> String {
    let mut parts = Vec::new();
    for (c, label) in coeffs.iter().zip(labels.iter()) {
        if c.is_zero() { continue; }
        let coeff_str = if *c == QRat::from(1) {
            label.clone()
        } else if *c == QRat::from(-1) {
            format!("-{}", label)
        } else {
            format!("{}*{}", c, label)
        };
        parts.push(coeff_str);
    }
    if parts.is_empty() { "0".to_string() }
    else { parts.join(" + ").replace("+ -", "- ") }
}
```

### Pattern 2: X[i]-based Output (findhom, findhomcombo, findnonhom, etc.)

**What:** Functions without SL use auto-generated `X[1], X[2], ...` labels for polynomial expressions.
**When to use:** All relation discovery functions except findlincombo/findlincombomodp.

```rust
// Format monomial expression using X[i] labels
fn format_monomial_term(coeff: &QRat, exponents: &[i64], labels: &[String]) -> String {
    let mut parts = Vec::new();
    for (i, &e) in exponents.iter().enumerate() {
        if e == 0 { continue; }
        if e == 1 {
            parts.push(labels[i].clone());
        } else {
            parts.push(format!("{}^{}", labels[i], e));
        }
    }
    let monomial = if parts.is_empty() { "1".to_string() } else { parts.join("*") };
    // ... combine with coefficient
}

// Generate default labels: X[1], X[2], ..., X[k]
fn default_labels(k: usize) -> Vec<String> {
    (1..=k).map(|i| format!("X[{}]", i)).collect()
}
```

### Pattern 3: findcong Garvan-style (auto-scan moduli)

**What:** findcong scans all moduli from 2 to LM, computing GCD and factoring.
**When to use:** The new findcong implementation.

```rust
// Core algorithm matching Garvan's findcong
pub fn findcong_garvan(
    f: &FormalPowerSeries,
    t: i64,                    // truncation order
    lm: Option<i64>,           // max modulus (default: floor(sqrt(T)))
    xset: &HashSet<i64>,       // excluded moduli
) -> Vec<Congruence> {
    let lm = lm.unwrap_or_else(|| (t as f64).sqrt() as i64);
    let mut results = Vec::new();
    for m in 2..=lm {
        for r in 0..m {
            // Extract subsequence c(m*n + r) for n = 0..floor((t-r)/m)
            let max_n = (t - r) / m;
            let coeffs: Vec<Integer> = (0..=max_n)
                .map(|n| f.coeff(m * n + r).numer().clone())
                .collect();
            // Compute GCD
            let g = gcd_of_list(&coeffs);
            if g <= 1 { continue; }
            // Factor g and report prime power divisors
            for (p, e) in factor(g) {
                let pe = p.pow(e);
                if !xset.contains(&pe) {
                    results.push(Congruence {
                        residue_b: r,
                        modulus_m: m,
                        divisor_r: pe,
                    });
                }
            }
        }
    }
    results
}
```

### Pattern 4: Dispatch with Arg-Count Overloading

**What:** Functions like findcong and findpoly accept variable argument counts.
**When to use:** findcong (2-4 args), findpoly (5-6 args), findnonhomcombo (4-5 args).

```rust
"findcong" => {
    expect_args_range(name, args, 2, 4)?;
    let fps = extract_series(name, args, 0)?;
    let t = extract_i64(name, args, 1)?;
    let lm = if args.len() >= 3 {
        Some(extract_i64(name, args, 2)?)
    } else {
        None
    };
    let xset = if args.len() >= 4 {
        let list = extract_i64_list(name, args, 3)?;
        list.into_iter().collect::<HashSet<i64>>()
    } else {
        HashSet::new()
    };
    let results = findcong_garvan(&fps, t, lm, &xset);
    // Print each [B, A, R] triple
    for c in &results {
        println!("[{}, {}, {}]", c.residue_b, c.modulus_m, c.divisor_r);
    }
    // Return list of triples
    Ok(Value::List(results.iter().map(|c| Value::List(vec![
        Value::Integer(QInt::from(c.residue_b)),
        Value::Integer(QInt::from(c.modulus_m)),
        Value::Integer(QInt::from(c.divisor_r)),
    ])).collect()))
}
```

### Anti-Patterns to Avoid

- **Putting SL on functions that Garvan doesn't have SL:** Only findlincombo and findlincombomodp have SL. Don't add it to findhomcombo, findnonhomcombo, findhomcombomodp.
- **Wrong p/q ordering for modp functions:** Garvan consistently puts p BEFORE q. Don't swap them.
- **Using topshift for findpoly:** Garvan uses a fixed +10 buffer, not configurable topshift. The optional 6th arg is `check` for verification, not topshift.
- **Keeping moduli-list interface for findcong:** The Garvan interface auto-scans. The current `(series, [moduli])` interface is not Maple-compatible.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Linear algebra (null space) | Custom solver | `qseries::findlincombo`, etc. | Core functions already correct |
| Monomial generation | Custom combinatorics | `generate_monomials` (in relations.rs) | Already implemented, needs pub |
| Primality testing | Complex algorithm | Simple trial division for small p | Only used for validation, not performance-critical |
| Symbol extraction | Manual matching | `extract_symbol_id(name, args, idx, env)` | Standard Phase 33 helper |
| GCD computation | Custom GCD | `rug::Integer::gcd` | rug library already available |
| Integer factorization | Sophisticated algorithm | Trial division up to sqrt(n) | findcong GCDs are typically small |

**Key insight:** The core `qseries::*` functions in `qsym-core` are mostly correct and don't need signature changes (except findcong). The Phase 36 work is primarily in the CLI dispatch layer (`eval.rs`) -- extracting arguments from Maple-style calling conventions, formatting output with labels, and passing them to the unchanged core functions. The exception is findcong which needs a fundamentally different algorithm in the core.

## Common Pitfalls

### Pitfall 1: SL Applied to Wrong Functions
**What goes wrong:** Adding SL parameter to findhomcombo/findnonhomcombo/findhomcombomodp because the requirements say so, but Garvan's actual Maple code doesn't have SL for these functions.
**Why it happens:** Requirements document was written with assumptions, not verified against source.
**How to avoid:** Follow verified Garvan signatures. Only findlincombo and findlincombomodp have SL. Others use X[i] auto-labels.
**Warning signs:** Mismatch between our output and Garvan's Maple output.

### Pitfall 2: findcong Algorithm Mismatch
**What goes wrong:** Keeping the current `(series, [moduli])` interface or just adding a T parameter without changing the algorithm.
**Why it happens:** Not realizing Garvan's findcong auto-scans all moduli and uses GCD + factorization, not just fixed-prime checks.
**How to avoid:** Implement the Garvan algorithm: scan M=2..LM, compute GCD of coefficients, factor GCD, report prime power divisors.
**Warning signs:** findcong(partition_gf(200), 200) not finding `[24, 25, 25]` (the 5^2 congruence).

### Pitfall 3: p/q Parameter Ordering
**What goes wrong:** Putting q before p in modp functions, contradicting Garvan's convention.
**Why it happens:** Requirements document has q before p, but Garvan has p before q.
**How to avoid:** Follow Garvan: `findlincombomodp(f, L, SL, p, q, topshift)`, `findhommodp(L, p, q, n, topshift)`, `findhomcombomodp(f, L, p, q, n, topshift)`.
**Warning signs:** Argument-position errors when users copy Maple code.

### Pitfall 4: findpoly topshift vs check
**What goes wrong:** Using the 6th arg as topshift when Garvan uses it as check (verification order).
**Why it happens:** Requirements say topshift; Garvan uses check.
**How to avoid:** The 6th arg is optional `check` -- when provided, verify the found polynomial to O(q^check). Our current code already has topshift in the core function -- we can keep using it internally but expose the optional check parameter at the CLI level. For backward compat with our internal topshift, keep topshift fixed at 0 (or 10 as Garvan does) and add the optional check parameter.
**Warning signs:** findpoly producing different results or missing verification output.

### Pitfall 5: Monomial Ordering Mismatch
**What goes wrong:** The generate_monomials function returns monomials in a specific order, but the output labels don't match user expectations.
**Why it happens:** Garvan and our implementation may use different monomial orderings.
**How to avoid:** Use the same ordering as generate_monomials and document it clearly. The ordering is lexicographic on exponent tuples.
**Warning signs:** Polynomial expressions having coefficients in unexpected positions.

### Pitfall 6: Output Format -- Returning Value vs Printing
**What goes wrong:** Only returning a Value without printing the formatted output, or only printing without returning a useful Value.
**Why it happens:** Garvan's Maple functions both print (diagnostic/result output) and return (symbolic expression).
**How to avoid:** Functions should BOTH print formatted output (matching Garvan's display) AND return a structured Value for programmatic use. The printed output goes to stdout; the return value is what gets assigned to variables.
**Warning signs:** No output visible in REPL when function runs, or output appears but can't be captured in a variable.

### Pitfall 7: generate_monomials Visibility
**What goes wrong:** Can't format polynomial output because `generate_monomials` is private in the core library.
**Why it happens:** The function is `fn` not `pub fn`.
**How to avoid:** Either make `generate_monomials` pub, or re-implement the monomial ordering logic in the CLI (less desirable), or have the core functions return structured results that include the monomial information.
**Warning signs:** Can't map coefficients back to monomial terms for display.

## Implementation Decisions (Claude's Discretion Resolutions)

### 1. SL Length Validation
**Decision:** Strict match -- SL length must equal L length. Error message: "findlincombo: SL has N labels but L has M series". Rationale: Garvan's Maple would error on mismatched list lengths.
**Confidence:** HIGH (natural Maple behavior)

### 2. Auto-Generated Labels for Functions Without SL
**Decision:** Use `X[1], X[2], ..., X[k]` notation, matching Garvan's Maple X variable convention exactly. This is what Garvan uses in findhom, findhomcombo, findnonhom, etc.
**Confidence:** HIGH (verified from Garvan source: `product(X[ii]^d[i,ii], ii=1..D)`)

### 3. Modular Arithmetic Display
**Decision:** For modp variants, coefficients are displayed as plain integers in range [0, p-1], matching Garvan's `modp(ANS, p)` behavior. No explicit "mod p" annotation on each coefficient -- the function name already implies modular arithmetic.
**Confidence:** HIGH (verified from Garvan source)

### 4. Homogeneous Combo Polynomial Display
**Decision:** Display as Garvan does: polynomial expressions in X[i] variables. For example, `3*X[1]^2 - 2*X[1]*X[2] + X[2]^2`. When multiple relations found (null space dimension > 1), return them as a set/list of expressions.
**Confidence:** HIGH (verified from Garvan source)

### 5. Multiple Solution Handling
**Decision:** Return ALL null space vectors as separate polynomial expressions, matching Garvan who returns a set of relations. Print each relation on its own line. This is already what our core functions do (findhom returns `Vec<Vec<QRat>>`).
**Confidence:** HIGH (verified from Garvan source: returns `convert(gg,set)`)

### 6. findcong Overload Strategy
**Decision:** Use arg-count dispatch via `expect_args_range(name, args, 2, 4)`. Parse args positionally based on count. This matches Garvan's `nargs` checks.
**Confidence:** HIGH (directly matches Garvan's pattern)

### 7. Short-List Validation for Degree
**Decision:** For degree-n homogeneous relations among k series, the number of monomials is C(k+n-1, n). If the series list is shorter than needed to form any monomials, return empty result (no error). This is what the core functions already do.
**Confidence:** MEDIUM (reasonable behavior, matches existing code)

### 8. findpoly check Parameter
**Decision:** Implement the optional 6th arg as `check` (verification precision), not topshift. When provided, after finding the polynomial relation, verify it by expanding to O(q^check). Our internal topshift for the matrix is fixed at 10 (matching Garvan's `dim2 := dim1 + 10`). Print "The polynomial is" and the polynomial, followed by verification result if check provided.
**Confidence:** HIGH (verified from Garvan source)

### 9. findmaxind Signature
**Decision:** Keep as `(L, T)` with 2 args matching Garvan docs. T functions as topshift passed to the internal linear independence check. No q parameter per Garvan. Return value changes from list of indices to `[P, NXFL]` pair where P is the independent subset and NXFL is the list of indices.
**Confidence:** MEDIUM (docs only, not in wmprog64.txt source)

### 10. etaoption Parameter
**Decision:** Skip the `etaoption` parameter entirely. Our system doesn't have an eta-product symbolic form converter at the CLI level. Functions that Garvan accepts `etaoption` for (findhomcombo, findnonhomcombo, findhomcombomodp) will use fixed-length signatures without it.
**Confidence:** HIGH (pragmatic -- etaoption is a display convenience, not core math)

## Code Examples

### Example 1: findlincombo with SL labels

```rust
"findlincombo" => {
    // Maple: findlincombo(f, L, SL, q, topshift)
    expect_args(name, args, 5)?;
    let target = extract_series(name, args, 0)?;
    let candidates = extract_series_list(name, args, 1)?;
    let labels = extract_symbol_list(name, args, 2)?;
    let _sym = extract_symbol_id(name, args, 3, env)?;
    let topshift = extract_i64(name, args, 4)?;

    // Validate SL length matches L length
    if labels.len() != candidates.len() {
        return Err(EvalError::Other(format!(
            "findlincombo: SL has {} labels but L has {} series",
            labels.len(), candidates.len()
        )));
    }
    // Validate unique labels
    let mut seen = std::collections::HashSet::new();
    for label in &labels {
        if !seen.insert(label) {
            return Err(EvalError::Other(format!(
                "findlincombo: duplicate label '{}' in SL", label
            )));
        }
    }

    let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
    match qseries::findlincombo(&target, &refs, topshift) {
        Some(coeffs) => {
            let formatted = format_linear_combo(&coeffs, &labels);
            println!("{}", formatted);
            Ok(Value::String(formatted))
        }
        None => {
            println!("NOT A LINEAR COMBO.");
            Ok(Value::None)
        }
    }
}
```

### Example 2: findhom with auto X[i] labels

```rust
"findhom" => {
    // Maple: findhom(L, q, n, topshift)
    expect_args(name, args, 4)?;
    let series_list = extract_series_list(name, args, 0)?;
    let _sym = extract_symbol_id(name, args, 1, env)?;
    let degree = extract_i64(name, args, 2)?;
    let topshift = extract_i64(name, args, 3)?;
    let refs: Vec<&FormalPowerSeries> = series_list.iter().collect();
    let rows = qseries::findhom(&refs, degree, topshift);

    let k = series_list.len();
    let labels = default_labels(k);  // X[1], X[2], ..., X[k]
    let monomials = generate_monomials(k, degree);  // need pub access

    // Format each null space vector as a polynomial expression
    let mut expressions = Vec::new();
    for row in &rows {
        let expr = format_polynomial_relation(row, &monomials, &labels);
        println!("{}", expr);
        expressions.push(Value::String(expr));
    }
    Ok(Value::List(expressions))
}
```

### Example 3: findcong Garvan-style

```rust
"findcong" => {
    // Maple: findcong(QS, T, [LM], [XSET])
    expect_args_range(name, args, 2, 4)?;
    let fps = extract_series(name, args, 0)?;
    let t = extract_i64(name, args, 1)?;
    let lm = if args.len() >= 3 {
        Some(extract_i64(name, args, 2)?)
    } else {
        None  // default: floor(sqrt(T))
    };
    let xset: HashSet<i64> = if args.len() >= 4 {
        extract_i64_list(name, args, 3)?.into_iter().collect()
    } else {
        HashSet::new()
    };

    let results = findcong_garvan(&fps, t, lm, &xset);

    // Print each [B, A, R] triple
    for c in &results {
        println!("[{}, {}, {}]", c.residue_b, c.modulus_m, c.divisor_r);
    }

    // Return as list of [B, A, R] triples
    Ok(Value::List(results.iter().map(|c| Value::List(vec![
        Value::Integer(QInt::from(c.residue_b)),
        Value::Integer(QInt::from(c.modulus_m)),
        Value::Integer(QInt::from(c.divisor_r)),
    ])).collect()))
}
```

### Example 4: findlincombomodp with primality check

```rust
"findlincombomodp" => {
    // Maple: findlincombomodp(f, L, SL, p, q, topshift)
    expect_args(name, args, 6)?;
    let target = extract_series(name, args, 0)?;
    let candidates = extract_series_list(name, args, 1)?;
    let labels = extract_symbol_list(name, args, 2)?;
    let p = extract_i64(name, args, 3)?;
    let _sym = extract_symbol_id(name, args, 4, env)?;
    let topshift = extract_i64(name, args, 5)?;

    // Validate p is prime
    if !is_prime(p) {
        return Err(EvalError::Other(format!(
            "findlincombomodp: Argument 4 (p): {} is not prime", p
        )));
    }
    // ... SL validation as above ...

    let refs: Vec<&FormalPowerSeries> = candidates.iter().collect();
    match qseries::findlincombomodp(&target, &refs, p, topshift) {
        Some(coeffs) => {
            let formatted = format_linear_combo_modp(&coeffs, &labels, p);
            println!("{}", formatted);
            Ok(Value::String(formatted))
        }
        None => {
            println!("NOT A LINEAR COMBO MOD {}.", p);
            Ok(Value::None)
        }
    }
}
```

### Example 5: Primality test helper

```rust
fn is_prime(n: i64) -> bool {
    if n < 2 { return false; }
    if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5i64;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}
```

### Example 6: Test pattern

```rust
#[test]
fn dispatch_findlincombo_maple_style() {
    let mut env = make_env();
    let f1 = dispatch("partition_gf", &[Value::Integer(QInt::from(30i64))], &mut env).unwrap();
    let f2 = dispatch("distinct_parts_gf", &[Value::Integer(QInt::from(30i64))], &mut env).unwrap();
    // findlincombo(f1, [f1, f2], [F1, F2], q, 0)
    // f1 = 1*f1 + 0*f2, so should find [1, 0]
    let args = vec![
        f1.clone(),
        Value::List(vec![f1, f2]),
        Value::List(vec![Value::Symbol("F1".to_string()), Value::Symbol("F2".to_string())]),
        Value::Symbol("q".to_string()),
        Value::Integer(QInt::from(0i64)),
    ];
    let val = dispatch("findlincombo", &args, &mut env).unwrap();
    assert!(matches!(val, Value::String(_)));
}

#[test]
fn dispatch_findcong_garvan_style() {
    let mut env = make_env();
    let pgf = dispatch("partition_gf", &[Value::Integer(QInt::from(200i64))], &mut env).unwrap();
    // findcong(pgf, 200) should find Ramanujan congruences
    let args = vec![
        pgf,
        Value::Integer(QInt::from(200i64)),
    ];
    let val = dispatch("findcong", &args, &mut env).unwrap();
    // Should find [4, 5, 5] (Ramanujan p(5n+4) = 0 mod 5)
    if let Value::List(triples) = val {
        let has_ramanujan_5 = triples.iter().any(|t| {
            if let Value::List(items) = t {
                items.len() == 3
                    && matches!(&items[0], Value::Integer(n) if n.0 == 4)
                    && matches!(&items[1], Value::Integer(n) if n.0 == 5)
                    && matches!(&items[2], Value::Integer(n) if n.0 == 5)
            } else { false }
        });
        assert!(has_ramanujan_5, "Should find Ramanujan's p(5n+4) = 0 mod 5");
    } else {
        panic!("Expected list of triples");
    }
}
```

## State of the Art

| Old Approach (current) | New Approach (Phase 36) | Impact |
|----------------------|------------------------|--------|
| `findlincombo(target, [candidates], topshift)` | `findlincombo(f, L, SL, q, topshift)` | SL labels in output |
| No SL labels in output | `12*F1 + 13*F2` formatted output | Matches Maple display |
| `findcong(series, [moduli])` explicit moduli | `findcong(QS, T, [LM], [XSET])` auto-scan | Matches Garvan exactly |
| findcong returns Dict triples | findcong returns `[B, A, R]` list triples | Matches Garvan format |
| findhom returns raw coefficient vectors | findhom returns polynomial expressions in X[i] | Human-readable output |
| No q parameter in relation functions | All relation functions accept explicit q | Maple compatibility |

## Open Questions

1. **findmaxind implementation source**
   - What we know: The function is documented on qseries.org but NOT in the `wmprog64.txt` source file. Our current implementation works (Gaussian elimination for pivot columns).
   - What's unclear: Whether Garvan's findmaxind has additional behavior we're missing.
   - Recommendation: Keep our current algorithm. Change signature to `(L, T)` matching docs. Return `[P, NXFL]` pair. Accept that this is MEDIUM confidence since we can't verify against source.

2. **findcong GCD factorization for large numbers**
   - What we know: Garvan uses Maple's `ifactors` for GCD factorization. Our system needs to factor the GCD of coefficient subsequences. For partition generating functions, GCDs are typically small.
   - What's unclear: Whether large GCDs might occur in practice, requiring sophisticated factoring.
   - Recommendation: Use trial division for factoring since GCDs from partition-type series are typically small primes or their powers. If needed, rug::Integer has `is_divisible` already.

3. **Output mode: print vs return**
   - What we know: Garvan's Maple functions both print diagnostic info AND return symbolic expressions.
   - What's unclear: Whether our CLI should print to stdout during function evaluation (side effect) or only return formatted strings.
   - Recommendation: Print formatted results to stdout (matching Garvan's behavior -- researchers expect to see output immediately). Also return the formatted string as Value::String for variable assignment.

4. **findnonhomcombo variable arity**
   - What we know: Garvan's findnonhomcombo accepts 4-6 args with complex disambiguation.
   - What's unclear: Whether to support all Garvan's arg counts or simplify to fixed 5 args (f, L, q, n, topshift).
   - Recommendation: Support fixed 5 args only (skip etaoption). This matches the simplified pattern used across all other functions in this phase.

## Sources

### Primary (HIGH confidence)
- Garvan qseries Maple source code: `wmprog64.txt` from [qseries.org v1.2](https://qseries.org/fgarvan/qmaple/qseries/1.2/maple16-win64/) -- verified all function signatures
- [findlincombo function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/findlincombo.html)
- [findhomcombo function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/findhomcombo.html)
- [findnonhomcombo function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/findnonhomcombo.html)
- [findlincombomodp function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/findlincombomodp.html)
- [findhomcombomodp function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/findhomcombomodp.html)
- [findhom function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/findhom.html)
- [findhommodp function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/findhommodp.html)
- [findmaxind function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/findmaxind.html)
- [findpoly function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/findpoly.html)
- [findcong function reference](https://qseries.org/fgarvan/qmaple/qseries/functions/findcong.html)
- Current q-Kangaroo source: `crates/qsym-cli/src/eval.rs`, `crates/qsym-core/src/qseries/relations.rs`

### Secondary (MEDIUM confidence)
- [findmaxind docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findmaxind.html) -- documented but not in source file
- Phase 35 research (`35-RESEARCH.md`) -- established patterns for Maple-style dispatch migration

### Tertiary (LOW confidence)
- findmaxind internal behavior -- not verified against source, only docs

## Metadata

**Confidence breakdown:**
- Garvan signatures (findlincombo, findhom, etc.): HIGH - verified from actual Maple source code
- findcong algorithm: HIGH - verified from Maple source, fundamental rework needed
- findmaxind: MEDIUM - documented but not in source file
- Output formatting: HIGH - verified from Maple source ANS construction
- Architecture patterns: HIGH - follows established Phase 34/35 patterns
- Pitfalls: HIGH - derived from signature discrepancy analysis

**Research date:** 2026-02-19
**Valid until:** 2026-03-19 (stable -- Garvan signatures don't change, codebase is under our control)
