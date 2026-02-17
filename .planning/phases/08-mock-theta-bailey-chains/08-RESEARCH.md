# Phase 8: Mock Theta & Bailey Chains - Research

**Researched:** 2026-02-14
**Domain:** Mock theta functions, Appell-Lerch sums, Zwegers completions, Bailey pair/chain/lemma machinery (Rust, computational q-series algebra)
**Confidence:** HIGH

## Summary

Phase 8 implements four distinct but interconnected mathematical subsystems: (1) mock theta functions -- Ramanujan's 17+ classical functions of orders 3, 5, and 7, each defined as explicit q-series that can be computed term-by-term into FormalPowerSeries; (2) Appell-Lerch sums -- a unifying framework that expresses all classical mock theta functions in terms of a single building-block function m(x, q, z); (3) Zwegers completions -- the non-holomorphic correction that transforms mock theta functions into harmonic Maass forms; and (4) Bailey pair/chain machinery -- an algebraic engine for generating and verifying q-series identities through iterative pair transformation.

The key architectural insight is that mock theta functions are simply q-series defined by specific summation formulas with q-Pochhammer denominators -- the existing `aqprod`, `QMonomial`, and `FormalPowerSeries` infrastructure handles all computational evaluation. Each mock theta function is a named function that constructs an FPS by accumulating terms of the form q^{f(n)} / (product of q-Pochhammer factors). The Appell-Lerch sum m(x, q, z) is a two-parameter q-series that provides a canonical representation; expressing mock theta functions in terms of it gives structural insight but the FPS computation is independent. Zwegers completions involve a non-holomorphic correction term using the complementary error function erfc -- since the project works with exact rational q-series (not floating-point modular form computations), the completion is best represented symbolically or verified via known relations rather than computed numerically. The Bailey pair machinery is purely algebraic: pairs (alpha_n, beta_n) are stored as closures/data, and the Bailey lemma transforms one pair into another, enabling chain iteration to discover new identities.

The implementation strategy splits naturally into three plans: (1) mock theta functions + Appell-Lerch sums (PART-04 through PART-08), (2) Bailey pair database + lemma application (PART-09, PART-10), and (3) automated discovery + Zwegers completions + Python API (PART-06, PART-11).

**Primary recommendation:** Create `crates/qsym-core/src/qseries/mock_theta.rs` for mock theta function evaluation (all orders), `crates/qsym-core/src/qseries/appell_lerch.rs` for the Appell-Lerch sum m(x,q,z) and universal mock theta functions g2/g3, and `crates/qsym-core/src/qseries/bailey.rs` for Bailey pairs, lemma, chain, and discovery. Zwegers completions go in `appell_lerch.rs` as a symbolic representation (the non-holomorphic part is transcendental and cannot be computed as exact rational FPS). All modules build on existing `FormalPowerSeries`, `QMonomial`, `aqprod`, and `arithmetic::*` infrastructure with zero new crate dependencies.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rug` | 1.28 | QRat exact arithmetic (GMP-backed) | Already in use; all coefficients are exact rationals |
| `FormalPowerSeries` | internal | Sparse series representation | Phase 2 foundation; all mock theta evaluations produce FPS |
| `aqprod` | internal | q-Pochhammer symbol computation | Phase 3 foundation; denominators of mock theta functions are q-Pochhammer products |
| `QMonomial` | internal | Parameter representation: c*q^m | Phase 3; used for Appell-Lerch sum parameters |
| `PochhammerOrder` | internal | Finite/Infinite order enum | Phase 3; used in product evaluations |
| `arithmetic::*` | internal | FPS add, mul, invert, shift | Phase 2; needed for series accumulation |
| `eval_phi` / `eval_psi` | internal | Hypergeometric series evaluation | Phase 6; some mock theta functions have hypergeometric representations |
| `BTreeMap<i64, QRat>` | stdlib | Sparse FPS storage | Underlying FPS representation |
| `serde` | 1 | TOML serialization for Bailey pair database | Already a dependency for identity database |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `etaq` | internal | Eta quotient computation | Verification of mock theta + theta function relations |
| `theta2/3/4` | internal | Jacobi theta functions | Verification via known identities |
| `findlincombo` / `findhom` | internal | Relation discovery | Bailey pair automated discovery uses relation-finding |
| `IdentityDatabase` | internal | TOML identity storage | Pattern for Bailey pair database storage |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Direct term-by-term FPS accumulation | Hypergeometric eval_phi | Most mock theta functions have non-standard denominators (e.g., (1+q^2)(1+q^4)...) that don't fit the _r phi_s template directly. Direct accumulation is simpler and more general. |
| Symbolic Zwegers completion | Numerical floating-point completion | The non-holomorphic part involves erfc (error function) and is inherently transcendental. Since the project uses exact QRat arithmetic, symbolic representation + verification via known identities is the right approach. |
| Closure-based Bailey pairs | Struct-based pairs | Closures would be more flexible but harder to serialize. Use a BaileyPair struct with enum variants for known pair types (each variant stores its parameters and has an `eval(n)` method). |

**Dependencies to add:** None. Phase 8 uses only libraries already in Cargo.toml.

## Architecture Patterns

### Recommended Project Structure

```
crates/qsym-core/src/qseries/
  mock_theta.rs          # Mock theta functions (all orders)
  appell_lerch.rs        # Appell-Lerch sums, universal mock theta, Zwegers
  bailey.rs              # Bailey pairs, lemma, chain, discovery
  mod.rs                 # Updated with new pub mod + pub use

crates/qsym-core/tests/
  qseries_mock_theta_tests.rs    # Mock theta function tests
  qseries_appell_lerch_tests.rs  # Appell-Lerch sum tests
  qseries_bailey_tests.rs        # Bailey pair/chain tests

crates/qsym-python/src/
  dsl.rs                 # Extended with Group 10: Mock Theta & Bailey
```

### Pattern 1: Mock Theta Function Evaluation (Term-by-Term Accumulation)

**What:** Each mock theta function is computed by accumulating terms q^{f(n)} / (product of specific factors), similar to eval_phi but with non-standard denominators.

**When to use:** All PART-04/PART-05 mock theta function evaluations.

**Example:**
```rust
/// Third-order mock theta function f(q).
///
/// f(q) = sum_{n=0}^{inf} q^{n^2} / (-q;q)_n^2
///
/// OEIS A000025: 1, 1, -2, 3, -3, 3, -5, 7, -6, 6, -10, 12, ...
pub fn mock_theta_f3(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // Accumulate denominator product incrementally
    let mut denom = FormalPowerSeries::one(variable, truncation_order);

    for n in 0..truncation_order {
        let q_exp = n * n;
        if q_exp >= truncation_order {
            break;
        }
        // term = q^{n^2} / denom
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let denom_inv = arithmetic::invert(&denom);
        let term = arithmetic::mul(&numer, &denom_inv);
        result = arithmetic::add(&result, &term);

        // Update denom: denom *= (1 + q^{n+1})^2
        // (-q;q)_{n+1} = (-q;q)_n * (1 + q^{n+1})
        // So (-q;q)_n^2 step: multiply by (1+q^{n+1})^2
        let factor = one_plus_q_m(n + 1, variable, truncation_order);
        denom = arithmetic::mul(&denom, &factor);
        denom = arithmetic::mul(&denom, &factor);
    }
    result
}
```

**Key insight:** The denominator product is built incrementally (multiply by one more factor each iteration), not recomputed from scratch. This gives O(N^2) total work (N terms, each requiring O(N) FPS operations) rather than O(N^3).

### Pattern 2: Appell-Lerch Sum m(x, q, z)

**What:** The Appell-Lerch sum is a canonical building block for all mock theta functions. Following Hickerson-Mortenson notation:

```
m(x, q, z) = (1/j(z,q)) * sum_{r in Z} (-1)^r * q^{r(r-1)/2} * z^r / (1 - x*q^r*z)
```

where j(z, q) = (z;q)_inf * (q/z;q)_inf * (q;q)_inf is the Jacobi theta function.

**When to use:** PART-07 (Appell-Lerch sums) and PART-08 (universal mock theta function).

**Implementation note:** Since m(x,q,z) involves two parameters x and z beyond the base q, and our FPS is single-variable in q, x and z must be specialized to specific QMonomial values (x = q^a, z = q^b) before evaluation. The sum converges when computed for fixed x, z as a Laurent series in q.

### Pattern 3: Bailey Pair Storage and Lemma Application

**What:** A Bailey pair relative to parameter `a` is (alpha_n, beta_n) satisfying:
```
beta_n = sum_{j=0}^{n} alpha_j / [(q;q)_{n-j} * (aq;q)_{n+j}]
```

The Bailey lemma produces a new pair (alpha'_n, beta'_n) from an existing one:
```
alpha'_n = [(b;q)_n * (c;q)_n * (aq/bc)^n] / [(aq/b;q)_n * (aq/c;q)_n] * alpha_n
beta'_n = sum_{k=0}^{n} [(b;q)_k * (c;q)_k * (aq/bc)_{n-k} * (aq/bc)^k] / [(q;q)_{n-k}] * beta_k
```

**When to use:** PART-09, PART-10, PART-11.

**Example:**
```rust
/// A Bailey pair relative to parameter `a`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaileyPair {
    /// Name/identifier for this pair
    pub name: String,
    /// The parameter `a` (as QMonomial)
    pub a: BaileyParam,
    /// Type classification
    pub pair_type: BaileyPairType,
    /// Tags for search
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BaileyPairType {
    /// Rogers-Ramanujan: alpha_n = (a;q)_n*(1-aq^{2n})*(-1)^n*q^{n(3n-1)/2}*a^n / [(q;q)_n*(1-a)]
    /// beta_n = 1/(q;q)_n
    RogersRamanujan,
    /// Unit pair: alpha_0 = 1, alpha_n = 0 for n > 0; beta_n = 1/[(q;q)_n*(aq;q)_n]
    Unit,
    /// Conjugate pair from q-binomial theorem
    QBinomial { z: QRat },
    /// Custom pair with explicit coefficient tables
    Tabulated { alphas: Vec<QRat>, betas: Vec<QRat> },
    // ... more canonical types
}

impl BaileyPair {
    /// Evaluate alpha_n for this pair at index n.
    pub fn alpha(&self, n: i64, variable: SymbolId, truncation_order: i64) -> QRat { ... }
    /// Evaluate beta_n for this pair at index n.
    pub fn beta(&self, n: i64, variable: SymbolId, truncation_order: i64) -> QRat { ... }
}
```

### Pattern 4: Bailey Chain Iteration

**What:** Iteratively apply the Bailey lemma with specific parameter choices (b, c) to generate chains of pairs. Key specializations:
- b, c -> infinity: alpha'_n = q^{n^2+n} * alpha_n (simplest chain step)
- b = q*sqrt(a), c = -q*sqrt(a): well-poised chain

**When to use:** PART-10 (chain iteration) and PART-11 (automated discovery).

### Anti-Patterns to Avoid

- **Computing erfc numerically for Zwegers completions:** The project uses exact QRat arithmetic. The non-holomorphic completion involves the complementary error function, which is transcendental. Represent completions symbolically or verify known relations rather than attempting numerical computation.
- **Implementing mock theta functions as hypergeometric series via eval_phi:** While some mock theta functions have hypergeometric representations (e.g., f(q) = sum q^{n^2}/(-q;q)_n^2 involves a specialized 2phi1-like structure), the denominators are non-standard (involving (-q;q)_n, (q;q^2)_n, etc.). Direct term-by-term accumulation is cleaner than trying to shoehorn them into the _r phi_s framework.
- **Building Bailey pair database as generic functions:** Bailey pairs should be typed enum variants (RogersRamanujan, Unit, QBinomial, etc.) rather than arbitrary closures. This enables serialization, search, and display.
- **Recomputing q-Pochhammer products from scratch each iteration:** Always maintain running products incrementally. For f(q), the denominator (-q;q)_n^2 should be updated by multiplying by (1+q^{n+1})^2, not recomputed as aqprod(&neg_q, variable, Finite(n+1), trunc) at each step.

## Mathematical Definitions

### PART-04: Third-Order Mock Theta Functions

Ramanujan's seven third-order mock theta functions:

```
f(q)   = sum_{n>=0} q^{n^2} / (-q;q)_n^2
         OEIS A000025: 1, 1, -2, 3, -3, 3, -5, 7, -6, 6, -10, 12, ...

phi(q)  = sum_{n>=0} q^{n^2} / (-q^2;q^2)_n
          OEIS A053250: 1, 1, 0, -1, 1, 1, -1, -1, 0, 2, ...

psi(q)  = sum_{n>=1} q^{n^2} / (q;q^2)_n
          OEIS A053251: 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 5, ...

chi(q)  = sum_{n>=0} q^{n^2} / prod_{k=1}^{n} (1 - q^k + q^{2k})
          OEIS A053252: 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, -1, ...

omega(q) = sum_{n>=0} q^{2n(n+1)} / (q;q^2)_{n+1}^2
           OEIS A053253

nu(q)   = sum_{n>=0} q^{n(n+1)} / (-q;q^2)_{n+1}
          OEIS A053254

rho(q)  = sum_{n>=0} q^{2n(n+1)} / prod_{k=0}^{n} (1 + q^{2k+1} + q^{4k+2})
          OEIS A053255
```

**Implementation notes:**
- f(q): denominator (-q;q)_n^2 means (1+q)(1+q^2)...(1+q^n) squared. Use aqprod with a = QMonomial::new(-QRat::one(), 1) (i.e., -q), then square.
- phi(q): denominator (-q^2;q^2)_n means (1+q^2)(1+q^4)...(1+q^{2n}). Use aqprod with a = QMonomial::new(-QRat::one(), 2), q -> q^2 steps.
- psi(q): denominator (q;q^2)_n = (1-q)(1-q^3)...(1-q^{2n-1}). Starts at n=1.
- chi(q): denominator factors (1-q^k+q^{2k}) are NOT q-Pochhammer products. These must be built as custom FPS factors. The factor 1 - q^k + q^{2k} = (1 - zeta*q^k)(1 - zeta_bar*q^k) where zeta = e^{2pi*i/3}, but in exact arithmetic, compute as a 3-term polynomial in q^k.
- omega(q), nu(q): involve q^2 base Pochhammer symbols.
- rho(q): like chi, has non-Pochhammer factors (1+q^{2k+1}+q^{4k+2}).

### PART-05: Fifth and Seventh Order Mock Theta Functions

**Fifth order (10 functions):**
```
f0(q) = sum_{n>=0} q^{n^2} / (-q;q)_n
        OEIS A053256: 1, 1, -1, 1, 0, 0, -1, 1, 0, 1, -2, ...

f1(q) = sum_{n>=0} q^{n^2+n} / (-q;q)_n
        OEIS A053257

F0(q) = sum_{n>=0} q^{2n^2} / (q;q^2)_n
        OEIS A053258

F1(q) = sum_{n>=0} q^{2n^2+2n} / (q;q^2)_{n+1}
        OEIS A053259

phi0(q) = sum_{n>=0} (-q;q^2)_n * q^{n^2}
          OEIS A053260

phi1(q) = sum_{n>=0} (-q;q^2)_n * q^{(n+1)^2}

psi0(q) = sum_{n>=0} (-1;q)_n * q^{n(n+1)/2}
          (Note: 1 + 2*psi0(q) = ...)

psi1(q) = sum_{n>=0} (-q;q)_n * q^{n(n+1)/2}

chi0(q) = 2*F0(q) - phi0(-q)

chi1(q) = 2*F1(q) + q^{-1}*phi1(-q)
```

**Seventh order (3 functions):**
```
F0(q) = sum_{n>=0} q^{n^2} / (q^{n+1};q)_n
        OEIS A053263 (if available)

F1(q) = sum_{n>=0} q^{n^2} / (q^n;q)_n
        OEIS A053264 (if available)

F2(q) = sum_{n>=0} q^{n^2+n} / (q^{n+1};q)_{n+1}
```

**Implementation notes:**
- f0, f1: single (-q;q)_n denominator (not squared like third-order f).
- F0, F1: (q;q^2)_n denominators, same pattern as third-order psi.
- phi0, phi1: (-q;q^2)_n appears in NUMERATOR (not denominator), multiplied by q^{power}. These are finite product * monomial terms.
- psi0: (-1;q)_n = (1-1)(1-q)...(1-q^{n-1}) = 0 for n >= 1... Actually (-1;q)_n = (1+1)(1+q)(1+q^2)...(1+q^{n-1}) = 2(-q;q)_{n-1} via the convention that (-1;q)_n = prod_{k=0}^{n-1}(1-(-1)*q^k) = prod_{k=0}^{n-1}(1+q^k). Need to verify the precise convention used.
- Seventh-order: (q^{n+1};q)_n = (1-q^{n+1})(1-q^{n+2})...(1-q^{2n}). This is a shifted q-Pochhammer with varying base. Use aqprod with a = q^{n+1}, order n.

### PART-07: Appell-Lerch Sums

The Appell-Lerch sum in Hickerson-Mortenson notation:
```
m(x, q, z) = (1 / j(z;q)) * sum_{r in Z} (-1)^r * q^{r(r-1)/2} * z^r / (1 - x*q^r*z)
```
where j(z;q) = (z;q)_inf * (q/z;q)_inf * (q;q)_inf is the Jacobi theta function (essentially theta_1 up to normalization).

**Three functional equations (Zwegers/Hickerson):**
1. m(x, q, z*q) relates to m(x, q, z) via a shift
2. m(x, q, 1/z) relates to m(x, q, z) (reflection)
3. m(x*q, q, z) relates to m(x, q, z) (shift in x)

**Implementation approach:** For evaluation as FPS in q, fix x = q^a and z = q^b for specific integer powers a, b. The bilateral sum becomes:
```
m(q^a, q, q^b) = (1/j(q^b;q)) * sum_{r in Z} (-1)^r * q^{r(r-1)/2 + br} / (1 - q^{a+r+b})
```
The 1/(1 - q^k) terms expand as geometric series in q, contributing to the FPS.

### PART-08: Universal Mock Theta Functions

```
g2(x, q) = x^{-1} * (-q; q)_inf * sum_{n>=0} q^{n(n+1)/2} * (-q;q)_n / (x;q)_{n+1} * (q/x;q)_{n+1}

g3(x, q) = sum_{n>=0} q^{n(n+1)/2} / (x;q)_{n+1} * (q/x;q)_{n+1}
```

Gordon and McIntosh showed:
- Odd-order mock theta functions are specializations of g3(x, q)
- Even-order mock theta functions are specializations of g2(x, q)

**Implementation:** Evaluate g2 and g3 as FPS for specific x = q^a values.

### PART-06: Zwegers Completions

Zwegers' key construction: define the completed function
```
hat_mu(u, v; tau) = mu(u, v; tau) + (i/2) * R(u - v; tau)
```
where R is a non-holomorphic correction involving the complementary error function:
```
R(z; tau) = sum_{n in Z + 1/2} {sgn(n) - E((n + Im(z)/y)*sqrt(2y))} * (-1)^{n-1/2} * q^{-n^2/2} * e^{-2*pi*i*n*z}
```
with E(z) = 2*integral_0^z e^{-pi*t^2} dt (error function) and y = Im(tau).

**Implementation approach:** Since the project computes exact q-series (not numerical modular forms), the Zwegers completion is best handled as:
1. A symbolic `ZwegersCompletion` struct recording which mock theta function and its completion parameters
2. Verification of known completion identities (e.g., that completed forms satisfy modular transformation properties) by checking q-series relations to the required truncation order
3. NOT attempting to compute erfc numerically -- this would require floating-point arithmetic outside the project's exact-rational paradigm

### PART-09: Bailey Pair Database

Standard canonical Bailey pairs (relative to parameter a):

**Unit pair:**
```
alpha_0 = 1, alpha_n = 0 (n > 0)
beta_n = 1 / [(q;q)_n * (aq;q)_n]
```

**Rogers-Ramanujan pair (DLMF 17.12.6):**
```
alpha_n = (a;q)_n * (1 - a*q^{2n}) * (-1)^n * q^{n(3n-1)/2} * a^n / [(q;q)_n * (1-a)]
beta_n = 1 / (q;q)_n
```

**q-Binomial pair (from bilateral Bailey theory):**
```
alpha_n = (-1)^n * z^n * q^{n(n-1)/2}
beta_n = (z;q)_n * (q/z;q)_n / (q;q)_{2n}
```

**Conjugate pairs from Slater's list:** 130+ pairs catalogued by Slater (1952), indexed by type. Implementation should start with the most important 10-15 pairs and allow extension.

### PART-10: Bailey Lemma and Chain Iteration

**Bailey lemma (DLMF 17.12.5):** Given Bailey pair (alpha, beta) relative to a, and parameters b, c:
```
alpha'_n = [(b;q)_n * (c;q)_n * (aq/(bc))^n] / [(aq/b;q)_n * (aq/c;q)_n] * alpha_n
beta'_n = [1/((aq/b;q)_n * (aq/c;q)_n)] * sum_{k=0}^{n} [(b;q)_k * (c;q)_k * (aq/(bc);q)_{n-k} * (aq/(bc))^k / (q;q)_{n-k}] * beta_k
```

**Weak Bailey lemma:** When summing:
```
sum_{n>=0} q^{n^2} * a^n * beta_n = [1/(aq;q)_inf] * sum_{n>=0} q^{n^2} * a^n * alpha_n
```

**Key specializations for chain iteration:**
- b, c -> infinity (a = q): alpha'_n = q^{n^2+n} * alpha_n; beta'_n = sum_{k=0}^n q^{k^2+k}/(q;q)_{n-k} * beta_k
- b, c -> 0 (a = q): alpha'_n = q^{-n^2-n} * alpha_n (inverse direction)

**Implementation:** A `bailey_lemma(pair, b, c)` function that returns a new BaileyPair, with specialized fast paths for the b,c->infinity and b,c->0 cases.

### PART-11: Automated Bailey Pair Discovery

Given a conjectured identity (as two FPS that should be equal), attempt to:
1. Express one side as sum_{n>=0} (some weight) * beta_n for some sequence beta_n
2. Search the Bailey pair database for matching beta_n
3. If found, apply the weak Bailey lemma to verify: check that the other side matches sum * alpha_n
4. If direct match fails, try applying 1-2 Bailey lemma iterations to known pairs and re-check

This is essentially a search problem over the pair database + chain depth. Use `findlincombo` from Phase 4 to check if coefficient sequences match known pair evaluations.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| q-Pochhammer products in denominators | Custom product loops | `aqprod` with appropriate QMonomial + PochhammerOrder | Already handles finite/infinite, positive/negative order, arbitrary base |
| FPS arithmetic (add, mul, invert) | Manual coefficient manipulation | `arithmetic::add/mul/invert` | Truncation-aware, sparse-optimized, thoroughly tested |
| Bailey pair coefficient verification | Custom Gaussian elimination | `findlincombo` / `findhom` from Phase 4 | Exact rational linear algebra already implemented |
| TOML serialization for Bailey pairs | Custom parser | `serde` with `#[derive(Serialize, Deserialize)]` | Pattern established by Phase 7 identity database |
| FPS equality checking | Manual coefficient comparison | Compare via `arithmetic::add(a, &arithmetic::negate(b))` and check all-zero | Already handles sparse representation correctly |
| Complementary error function | Numerical erfc implementation | Symbolic representation only | Exact rational arithmetic cannot represent erfc; verify completion relations via q-series instead |

**Key insight:** The existing infrastructure (aqprod, FPS arithmetic, linear algebra) handles 90% of the computational work. Mock theta functions are just named q-series with specific term formulas. Bailey pairs are stored data + algebraic transformations. The hard part is getting the mathematical formulas right, not the computation.

## Common Pitfalls

### Pitfall 1: q-Pochhammer Base Confusion
**What goes wrong:** Mock theta functions use various bases: (a;q)_n, (a;q^2)_n, (-q;q)_n, (-q;q^2)_n. Confusing the base (q vs q^2) produces wrong series.
**Why it happens:** The standard `aqprod` uses base q (step size 1). For base q^2 you need step size 2, which means using `etaq`-style generators or manual product construction.
**How to avoid:** Create explicit helper functions like `aqprod_q2` that handle base-q^2 products: (a;q^2)_n = prod_{k=0}^{n-1}(1 - a*q^{2k}). This is equivalent to aqprod with a monomial having doubled power steps.
**Warning signs:** First few coefficients match but diverge at higher orders.

### Pitfall 2: Non-Pochhammer Denominators (chi, rho)
**What goes wrong:** chi(q) has factors (1 - q^k + q^{2k}) and rho(q) has factors (1 + q^{2k+1} + q^{4k+2}) -- these are NOT q-Pochhammer products.
**Why it happens:** Trying to express everything through aqprod. These are cyclotomic-type factors.
**How to avoid:** Build these as explicit 3-term FPS: create a helper `one_minus_q_plus_q2(k, variable, trunc)` that returns 1 - q^k + q^{2k} as an FPS.
**Warning signs:** Attempt to factor into q-Pochhammer products fails or gives wrong results.

### Pitfall 3: Incremental vs Recomputed Denominators
**What goes wrong:** Recomputing the full denominator product aqprod(a, variable, Finite(n), trunc) at each step n gives O(N^3) complexity instead of O(N^2).
**Why it happens:** Not maintaining a running product that gets multiplied by one more factor each iteration.
**How to avoid:** Always maintain the running denominator product and update it incrementally.
**Warning signs:** Tests pass but computation is 10-100x slower than expected for truncation_order > 50.

### Pitfall 4: Bailey Lemma beta'_n Summation Off-by-One
**What goes wrong:** The summation in beta'_n = sum_{k=0}^{n} [...] beta_k has subtle index dependencies in the q-Pochhammer factors.
**Why it happens:** (aq/bc;q)_{n-k} and (b;q)_k share the index space but shift differently.
**How to avoid:** Test the Bailey lemma with the unit pair first (alpha_0=1, rest=0, giving a trivially verifiable result), then test with the Rogers-Ramanujan pair, comparing output against published tables.
**Warning signs:** Chain produces pairs that don't satisfy the Bailey pair relation when verified.

### Pitfall 5: Bilateral vs Unilateral Sums
**What goes wrong:** The Appell-Lerch sum m(x,q,z) is a bilateral sum (r from -inf to +inf). Truncating only the positive side gives wrong results.
**Why it happens:** Forgetting the negative part, which contributes non-trivially.
**How to avoid:** Use the same positive/negative split strategy as eval_psi: compute positive part (r >= 0) and negative part (r < 0) separately, then add. Bound the negative part by tracking when q^{r(r-1)/2} exceeds truncation order.
**Warning signs:** Appell-Lerch sums don't satisfy their functional equations when tested.

### Pitfall 6: Fifth-Order psi0 Convention Ambiguity
**What goes wrong:** Different references define psi0 with different normalizations. Some give "1 + 2*psi0(q) = ..." while others give psi0 directly.
**Why it happens:** Multiple conventions in the literature.
**How to avoid:** Always verify against OEIS coefficients for the first 10-15 terms. Use OEIS sequence numbers as ground truth.
**Warning signs:** Coefficients off by factor of 2 or shifted by a constant.

## Code Examples

### Mock Theta Function f(q) - Third Order

```rust
// Verified pattern: term-by-term with incremental denominator
pub fn mock_theta_f3(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // (-q;q)_0^2 = 1
    let mut denom_sq = FormalPowerSeries::one(variable, truncation_order);

    for n in 0.. {
        let q_exp = n * n;
        if q_exp >= truncation_order { break; }

        // term_n = q^{n^2} / (-q;q)_n^2
        let numer = FormalPowerSeries::monomial(variable, QRat::one(), q_exp, truncation_order);
        let term = arithmetic::mul(&numer, &arithmetic::invert(&denom_sq));
        result = arithmetic::add(&result, &term);

        // Update: (-q;q)_{n+1}^2 = (-q;q)_n^2 * (1+q^{n+1})^2
        let factor = {
            let m = n + 1;
            if m < truncation_order {
                let mut f = FormalPowerSeries::one(variable, truncation_order);
                f.set_coeff(m, QRat::one()); // 1 + q^m
                f
            } else {
                FormalPowerSeries::one(variable, truncation_order)
            }
        };
        denom_sq = arithmetic::mul(&denom_sq, &factor);
        denom_sq = arithmetic::mul(&denom_sq, &factor);
    }
    result
}
```

### Bailey Lemma Application

```rust
/// Apply the Bailey lemma: given (alpha, beta) relative to a,
/// produce (alpha', beta') relative to a with parameters b, c.
pub fn bailey_lemma(
    pair: &BaileyPair,
    a: &QMonomial,
    b: &QMonomial,
    c: &QMonomial,
    max_n: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> BaileyPairTabulated {
    let aq_over_bc = a.mul(&QMonomial::q()).div(&b.mul(c));
    let mut new_alphas = Vec::new();
    let mut new_betas = Vec::new();

    for n in 0..=max_n {
        // alpha'_n = [(b;q)_n * (c;q)_n * (aq/bc)^n] / [(aq/b;q)_n * (aq/c;q)_n] * alpha_n
        let b_poch = aqprod_coeff(b, n);  // (b;q)_n evaluated as rational
        let c_poch = aqprod_coeff(c, n);
        let aq_b = a.mul(&QMonomial::q()).div(b);
        let aq_c = a.mul(&QMonomial::q()).div(c);
        let aq_b_poch = aqprod_coeff(&aq_b, n);
        let aq_c_poch = aqprod_coeff(&aq_c, n);
        let ratio = aq_over_bc.power * n; // q^{...} power contribution
        let old_alpha = pair.alpha(n);
        // Combine all factors...
        let new_alpha = /* ... combine all the above ... */;
        new_alphas.push(new_alpha);

        // beta'_n = [1/(aq/b;q)_n*(aq/c;q)_n] * sum_{k=0}^{n} [...]
        let mut beta_sum = QRat::zero();
        for k in 0..=n {
            let bk = aqprod_coeff(b, k);
            let ck = aqprod_coeff(c, k);
            let bc_shift = aqprod_coeff(&aq_over_bc, n - k);
            let q_nk = aqprod_coeff(&QMonomial::q(), n - k); // (q;q)_{n-k}
            let old_beta = pair.beta(k);
            // accumulate...
        }
        new_betas.push(beta_sum);
    }

    BaileyPairTabulated { alphas: new_alphas, betas: new_betas }
}
```

### Appell-Lerch Sum (Specialized)

```rust
/// Compute m(q^a_pow, q, q^z_pow) as FPS, following Hickerson-Mortenson.
///
/// m(x, q, z) = (1/j(z;q)) * sum_{r in Z} (-1)^r * q^{r(r-1)/2} * z^r / (1 - x*q^r*z)
pub fn appell_lerch_m(
    a_pow: i64,      // x = q^{a_pow}
    z_pow: i64,      // z = q^{z_pow}
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    // j(z;q) = (z;q)_inf * (q/z;q)_inf * (q;q)_inf
    let z_mono = QMonomial::q_power(z_pow);
    let qz_inv = QMonomial::q_power(1 - z_pow);
    let q_mono = QMonomial::q_power(1);

    let j_z = arithmetic::mul(
        &arithmetic::mul(
            &aqprod(&z_mono, variable, PochhammerOrder::Infinite, truncation_order),
            &aqprod(&qz_inv, variable, PochhammerOrder::Infinite, truncation_order),
        ),
        &aqprod(&q_mono, variable, PochhammerOrder::Infinite, truncation_order),
    );
    let j_z_inv = arithmetic::invert(&j_z);

    // Bilateral sum: positive part (r >= 0) + negative part (r < 0)
    let mut bilateral_sum = FormalPowerSeries::zero(variable, truncation_order);

    // Positive r
    for r in 0.. {
        let q_exp = r * (r - 1) / 2 + z_pow * r;
        if q_exp >= truncation_order && r > 0 { break; }
        let sign = if r % 2 == 0 { QRat::one() } else { -QRat::one() };
        // 1/(1 - q^{a_pow + r + z_pow}) = geometric series
        let denom_pow = a_pow + r + z_pow;
        let geom = geometric_series(denom_pow, variable, truncation_order);
        let term = arithmetic::mul(
            &FormalPowerSeries::monomial(variable, sign, q_exp, truncation_order),
            &geom,
        );
        bilateral_sum = arithmetic::add(&bilateral_sum, &term);
    }

    // Negative r
    for r in 1.. {
        let neg_r = -(r as i64);
        let q_exp = neg_r * (neg_r - 1) / 2 + z_pow * neg_r;
        if q_exp >= truncation_order { break; }
        // Similar to positive but with r -> -r
        // ...
    }

    arithmetic::mul(&bilateral_sum, &j_z_inv)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Mock theta functions as isolated curiosities | Unified via Appell-Lerch sums (Hickerson-Mortenson 2014) | 2014 | All classical mock theta are specializations of m(x,q,z) |
| No modular interpretation | Zwegers completions (2002) | 2002 | Mock theta = holomorphic part of harmonic Maass form |
| Individual Bailey pair proofs | Systematic Bailey chain/lattice (Andrews 1984, Warnaar 2001) | 1984-2001 | Automated identity generation |
| Unilateral Bailey pairs only | Bilateral Bailey pairs (Lovejoy, recent work 2025) | 2025 | Extended to bilateral series, new Rogers-Ramanujan identities |
| Manual identity verification | Computational verification via q-expansion | ongoing | Verify to O(q^N) then prove via modular methods |

**Deprecated/outdated:**
- Watson's individual transformations for specific mock theta functions: superseded by the unified Appell-Lerch framework
- Treating mock theta functions as purely formal series without modular context: Zwegers showed the deep connection to harmonic Maass forms

## Plan Structure Recommendation

### Plan 08-01: Mock Theta Functions (PART-04, PART-05)
- All 20 classical mock theta functions (7 third-order, 10 fifth-order, 3 seventh-order)
- Helper functions for non-standard denominators
- OEIS coefficient verification for every function
- Estimated: ~15 tasks

### Plan 08-02: Appell-Lerch Sums & Bailey Machinery (PART-07, PART-08, PART-09, PART-10)
- Appell-Lerch sum m(x, q, z) evaluation
- Universal mock theta functions g2, g3
- Functional equation verification
- Bailey pair database (struct, canonical pairs, TOML storage)
- Bailey lemma implementation
- Bailey chain iteration
- Weak Bailey lemma
- Estimated: ~18 tasks

### Plan 08-03: Zwegers Completions, Discovery & Python API (PART-06, PART-11)
- Zwegers completion symbolic representation
- Completion relation verification
- Automated Bailey pair discovery
- Python DSL bindings for all new functions
- Integration tests
- Estimated: ~12 tasks

## Open Questions

1. **Base-q^2 Pochhammer helper**
   - What we know: Several mock theta functions use (a;q^2)_n products. The current aqprod assumes base q with step 1.
   - What's unclear: Should we generalize aqprod to accept a step parameter, or create a separate helper?
   - Recommendation: Create a thin wrapper `aqprod_base(a, variable, n, base_step, trunc)` that internally constructs factors with doubled exponent increments. Keep aqprod's API unchanged.

2. **Bailey pair count for database**
   - What we know: Slater catalogued 130+ pairs. Not all are needed for the success criteria.
   - What's unclear: Exactly which pairs are needed for the automated discovery requirement (PART-11).
   - Recommendation: Start with 8-10 canonical pairs (unit, Rogers-Ramanujan, q-binomial, 5 from Slater that generate the most identities). The database is extensible.

3. **Zwegers completion depth**
   - What we know: The non-holomorphic part involves erfc, which is transcendental.
   - What's unclear: How deeply the success criterion "Zwegers' completions transform mock theta functions into harmonic Maass forms" should be implemented given exact-arithmetic constraints.
   - Recommendation: Implement symbolic representation + verify known completion relations via q-series coefficient comparison (e.g., check that the holomorphic part of the completed form has the right modular properties when tested against theta function identities). Do NOT implement numerical erfc.

4. **Two-variable series for Appell-Lerch**
   - What we know: m(x,q,z) naturally lives in two variables. Our FPS is single-variable.
   - What's unclear: Whether this is a fundamental limitation.
   - Recommendation: Always specialize x, z to powers of q (x = q^a, z = q^b) before evaluation. This covers all classical mock theta function representations and the success criteria. True two-variable series would be future work.

## Sources

### Primary (HIGH confidence)
- [DLMF 17.12](https://dlmf.nist.gov/17.12) - Bailey pairs definition, Bailey lemma, canonical Rogers-Ramanujan pair
- [Wolfram MathWorld: Mock Theta Function](https://mathworld.wolfram.com/MockThetaFunction.html) - Complete definitions of all 20 classical mock theta functions (third, fifth, seventh order)
- [OEIS A000025](https://oeis.org/A000025) - Third-order f(q) coefficients: 1, 1, -2, 3, -3, 3, -5, 7, ...
- [OEIS A053250](https://oeis.org/A053250) - Third-order phi(q) coefficients
- [OEIS A053251](https://oeis.org/A053251) - Third-order psi(q) coefficients
- [OEIS A053252](https://oeis.org/A053252) - Third-order chi(q) coefficients
- [OEIS A053256](https://oeis.org/A053256) - Fifth-order f0(q) coefficients
- [arXiv:2509.21230](https://arxiv.org/html/2509.21230) - Bailey pairs and quantum q-series identities (2025) - complete Bailey lemma formulas, 5 canonical pairs, chain iteration

### Secondary (MEDIUM confidence)
- [Hickerson-Mortenson, Proc. London Math. Soc. 2014](https://londmathsoc.onlinelibrary.wiley.com/doi/abs/10.1112/plms/pdu007) - Hecke-type double sums, Appell-Lerch sums, mock theta functions
- [arXiv:2501.12211](https://arxiv.org/abs/2501.12211) - Bilateral Bailey pairs (2025), bilateral Bailey lemma formula
- [Garvan: Universal Mock Theta Functions](https://qseries.org/fgarvan/papers/umtf.pdf) - g2, g3 definitions and Hecke-Rogers identities
- [Zagier Bourbaki Seminar](https://people.mpim-bonn.mpg.de/zagier/files/aster/326/fulltext.pdf) - Ramanujan's mock theta functions and applications, Zwegers completion overview
- [Griffin-Ono-Rolen](http://www.mi.uni-koeln.de/~mgriffin/PDFs/rama_mock.pdf) - Ramanujan's mock theta functions survey

### Tertiary (LOW confidence - needs validation during implementation)
- Exact formulas for phi0, phi1, psi0, psi1 of fifth order: multiple conventions exist in the literature. Must verify against OEIS during implementation.
- Seventh-order OEIS sequence numbers: not confirmed from search results.
- Specific Slater pair indices needed for automated discovery: to be determined during Plan 08-02 implementation.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All computation uses existing infrastructure (aqprod, FPS, arithmetic)
- Mock theta function definitions: HIGH - Verified against OEIS and MathWorld
- Bailey pair/lemma formulas: HIGH - Verified via DLMF 17.12 and multiple recent papers
- Appell-Lerch sum formulas: MEDIUM - Multiple sources agree on notation, but exact evaluation strategy (specializing x,z to q-powers) needs implementation validation
- Zwegers completions: MEDIUM - Mathematical theory is well-established, but implementation within exact-rational framework requires creative approach
- Architecture patterns: HIGH - Follows established patterns from Phases 6 and 7

**Research date:** 2026-02-14
**Valid until:** 2026-03-14 (stable mathematical domain, formulas don't change)
