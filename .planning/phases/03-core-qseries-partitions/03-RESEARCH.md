# Phase 3: Core q-Series & Partitions - Research

**Researched:** 2026-02-13
**Domain:** q-Pochhammer symbols, named infinite products, Jacobi theta functions, partition functions, rank/crank (Rust, computational number theory)
**Confidence:** HIGH

## Summary

Phase 3 builds the core q-series function library on top of the Phase 2 formal power series (FPS) engine and infinite product generators. The phase implements 11 requirements (QSER-01 through QSER-08, PART-01 through PART-03) covering: the general q-Pochhammer symbol `aqprod(a, q, n)` for positive, negative, and infinite n; the q-binomial coefficient `qbin(n, k, q)`; five named infinite products (etaq, jacprod, tripleprod, quinprod, winquist); three Jacobi theta functions (theta2, theta3, theta4); the partition function p(n); restricted partition generating functions; and partition rank/crank generating functions.

The key architectural insight is that nearly all functions in this phase reduce to compositions of the existing `InfiniteProductGenerator` and `FormalPowerSeries` arithmetic from Phase 2. The q-Pochhammer symbol for finite n is a finite product; for infinite n, it uses `qpochhammer_inf_generator`. Named products (jacprod, tripleprod, quinprod, winquist) are defined as products of q-Pochhammer symbols. Theta functions are defined via their product representations using q-Pochhammer symbols. The partition function p(n) already works via `1/(q;q)_inf` inversion (verified in Phase 2 tests). Restricted partitions use analogous product generators. The rank and crank generating functions involve summation of rational functions of q-series.

This phase adds NO new crate dependencies. All computation operates on the existing `FormalPowerSeries` with `BTreeMap<i64, QRat>` sparse storage and the `InfiniteProductGenerator` lazy expansion framework. The main implementation work is building a clean API module (`qseries`) that exposes Garvan-compatible function signatures, implementing the finite/negative q-Pochhammer cases, and writing thorough tests verified against OEIS data and known identities.

**Primary recommendation:** Create a new `crates/qsym-core/src/qseries/` module tree that provides all Phase 3 functions as public APIs, composing them from existing FPS arithmetic and infinite product generators. Use the product representations (not series summations) for all named products and theta functions, as these integrate naturally with the existing `InfiniteProductGenerator`.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rug` | 1.28 | QRat coefficient arithmetic (GMP-backed exact rationals) | Already in use; non-negotiable for exact q-series coefficients |
| `BTreeMap<i64, QRat>` | stdlib | Sparse FPS coefficient storage | Already in use from Phase 2; ordered iteration, O(log n) access |
| `FormalPowerSeries` | internal | Series representation and truncated arithmetic | Phase 2 foundation; all Phase 3 functions produce FPS output |
| `InfiniteProductGenerator` | internal | Lazy infinite product expansion | Phase 2 foundation; all named products and theta functions build on this |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `proptest` | 1 | Property-based testing for algebraic identities | Testing that product identities hold, that rank/crank satisfy known congruences |
| `smallvec` | 1 | Inline storage in Expr variants | Already a dependency; used by BasicHypergeometric variant |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Product representations for theta | Series summation for theta | Products compose with existing InfiniteProductGenerator; series summation would require a new summation framework. Products also serve as verification targets. |
| Direct finite product for aqprod(a,q,n) with n>0 | InfiniteProductGenerator with early stop | Finite products with small n are simple enough that a direct loop is cleaner. InfiniteProductGenerator is overkill for n=5. |
| Computing p(n) via series inversion | Pentagonal number recurrence for p(n) | Series inversion already works (verified in Phase 2). The recurrence p(n) = sum (-1)^{k+1} p(n - k(3k-1)/2) is O(n sqrt(n)) and avoids computing the full series, but for generating functions (needed for restricted partitions), the series approach is essential. Implement BOTH: series inversion for generating functions, recurrence for individual p(n) values. |

**Dependencies to add:** None. Phase 3 uses only libraries already in Cargo.toml.

## Architecture Patterns

### Recommended Module Structure

```
crates/qsym-core/src/
  lib.rs              # add: pub mod qseries;
  qseries/
    mod.rs            # Re-exports, module-level docs
    pochhammer.rs     # aqprod(a, q, n) -- finite, negative, infinite
    qbinomial.rs      # qbin(n, k, q) -- q-binomial coefficient
    products.rs       # etaq, jacprod, tripleprod, quinprod, winquist
    theta.rs          # theta2(q), theta3(q), theta4(q)
    partitions.rs     # p(n), restricted partition generating functions
    rank_crank.rs     # Rank and crank generating functions
```

### Pattern 1: Function Returns FormalPowerSeries

**What:** Every Phase 3 function takes parameters and a truncation order, and returns a `FormalPowerSeries`. This is consistent with how Garvan's Maple package works (every function returns a series expansion to O(q^T)).

**When to use:** All Phase 3 public API functions.

**Example:**

```rust
/// Compute (a; q)_n as a formal power series to O(q^T).
///
/// Garvan compatibility: aqprod(a, q, n) with T as truncation order.
///
/// - n > 0: finite product (1-a)(1-aq)...(1-aq^{n-1})
/// - n = 0: returns 1
/// - n < 0: 1 / [(aq^n; q)_{-n}]
/// - n = infinity: uses InfiniteProductGenerator
pub fn aqprod(
    a: &QRat,
    variable: SymbolId,
    n: PochhammerOrder,
    truncation_order: i64,
) -> FormalPowerSeries {
    match n {
        PochhammerOrder::Finite(0) => FormalPowerSeries::one(variable, truncation_order),
        PochhammerOrder::Finite(k) if k > 0 => aqprod_finite_positive(a, variable, k, truncation_order),
        PochhammerOrder::Finite(k) => aqprod_finite_negative(a, variable, k, truncation_order),
        PochhammerOrder::Infinite => aqprod_infinite(a, variable, truncation_order),
    }
}
```

### Pattern 2: Named Products as q-Pochhammer Compositions

**What:** Each named product (jacprod, tripleprod, quinprod, winquist) is defined as a product of q-Pochhammer symbols. Implementation multiplies the corresponding FPS results.

**When to use:** All named product functions.

**Example:**

```rust
/// Jacobi product: (a; q)_inf * (q/a; q)_inf * (q; q)_inf
///
/// JAC(a, q) = prod_{n>=0} (1-aq^n)(1-q^{n+1}/a)(1-q^{n+1})
///
/// In Garvan's notation: JAC(a, b, inf) where a and b are the
/// exponents in q^a and q^b.
pub fn jacprod(
    a: &QRat,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    // (a; q)_inf
    let p1 = aqprod(a, variable, PochhammerOrder::Infinite, truncation_order);
    // (q/a; q)_inf
    let q_over_a = QRat::one() / a.clone();
    let p2 = aqprod(&q_over_a, variable, PochhammerOrder::Infinite, truncation_order);
    // (q; q)_inf -- Euler function
    let p3 = aqprod(&QRat::one(), variable, PochhammerOrder::Infinite, truncation_order);

    let temp = arithmetic::mul(&p1, &p2);
    arithmetic::mul(&temp, &p3)
}
```

### Pattern 3: Enum for q-Pochhammer Order

**What:** The order parameter for aqprod can be a finite integer or infinity. Use an enum rather than a magic sentinel value.

**When to use:** aqprod and any function that takes a q-Pochhammer order.

**Example:**

```rust
/// The order of a q-Pochhammer symbol: finite integer or infinity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PochhammerOrder {
    Finite(i64),
    Infinite,
}
```

### Pattern 4: Test Against Known Mathematical Identities

**What:** Every function is tested by verifying known mathematical identities hold to a specified truncation order. This catches both implementation bugs and numerical issues.

**When to use:** All Phase 3 tests.

**Example:**

```rust
#[test]
fn jacobi_triple_product_identity() {
    // tripleprod(a, q) should equal sum_{n=-inf}^{inf} (-1)^n * a^n * q^{n(n-1)/2}
    // Verify by comparing product expansion with direct summation
    // to O(q^50)
}

#[test]
fn theta_product_identity() {
    // theta3(q)^4 = 1 + 8*sum_{n=1}^{inf} n*q^n / (1+(-q)^n)
    // This is Jacobi's four-square theorem
}
```

### Anti-Patterns to Avoid

- **Symbolic q-Pochhammer with symbolic 'a':** Phase 3 computes series expansions where `a` is a concrete QRat, not a symbolic expression. Do not try to expand `(x; q)_inf` where x is a symbolic variable -- that requires Phase 6+ hypergeometric machinery. Phase 3's `aqprod` takes `a: &QRat`.

- **Using series summation when products are available:** For theta functions, the product representation `theta3(q) = (q^2; q^2)_inf * (-q; q^2)_inf^2` is much better than summing `q^{n^2}` because it composes with existing InfiniteProductGenerator. The summation approach requires computing perfect squares and handling bilateral sums.

- **Computing p(n) for large n via full series:** For a single value p(200), don't compute a 201-term series. Use the pentagonal number recurrence. Reserve series computation for when the full generating function is needed.

- **Treating theta functions as requiring fractional powers:** In the standard q-series convention (as in Garvan), theta2 involves `q^{1/4}` prefactors. However, with the substitution `q -> q^2` in the nome, theta2 can be expressed purely in integer powers. The implementation must handle this carefully -- see the theta functions section below.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Infinite product expansion | Custom coefficient computation | `InfiniteProductGenerator` from Phase 2 | Already handles lazy expansion, truncation, incremental reuse |
| Series multiplication with truncation | Manual convolution loops | `arithmetic::mul()` from Phase 2 | Correctly handles truncation, sparse representation, zero cleanup |
| Series inversion 1/f(q) | Ad-hoc coefficient formula | `arithmetic::invert()` from Phase 2 | Handles the recurrence correctly with truncation |
| Exact rational arithmetic | Floating point or custom rationals | `QRat` (rug::Rational wrapper) | GMP-backed, exact, already hash-consing compatible |
| Partition function p(n) generating function | Custom expansion | `arithmetic::invert(&euler_function)` | Already verified in Phase 2 tests |

**Key insight:** Phase 3 is primarily a _composition_ phase. The hard computational work (series arithmetic, lazy product expansion) was done in Phase 2. Phase 3 provides the mathematical definitions and user-facing API.

## Mathematical Definitions

### QSER-01: q-Pochhammer Symbol -- aqprod(a, q, n)

**Positive finite n:**
```
(a; q)_n = prod_{k=0}^{n-1} (1 - a*q^k)
```
This is a polynomial in q of degree at most n*(n-1)/2 (when a = q^m for some m).

Special cases:
- `(a; q)_0 = 1`
- `(a; q)_1 = 1 - a`
- `(0; q)_n = 1` for all n
- `(1; q)_n = 0` for n >= 1 (first factor is 1-1=0)

**Negative finite n:**
```
(a; q)_{-n} = 1 / (a*q^{-n}; q)_n = 1 / prod_{k=0}^{n-1} (1 - a*q^{k-n})
            = prod_{k=1}^{n} 1/(1 - a/q^k)
```

Alternative form (useful for implementation):
```
(a; q)_{-n} = (-q/a)^n * q^{n(n-1)/2} / (q/a; q)_n
```

The practical approach: compute `(a*q^{-n}; q)_n` as a finite product, then invert the resulting FPS.

**Infinite n:**
```
(a; q)_inf = prod_{k=0}^{inf} (1 - a*q^k)
```
Use `qpochhammer_inf_generator(a, 0, variable, truncation_order)` from Phase 2 (with offset=0 for the standard form starting at k=0).

Note: The existing `qpochhammer_inf_generator` uses signature `(a, offset, variable, truncation_order)` where the product is `prod_{k=0}^{inf} (1 - a*q^{offset+k})`. For standard `(a; q)_inf`, use offset=0. For `(q; q)_inf`, use `a=1, offset=1` (equivalent to `euler_function_generator`).

**Test values for verification:**

| a | n | Result (as series to O(q^10)) |
|---|---|------|
| generic a | 0 | 1 |
| generic a | 1 | 1 - a |
| generic a | 2 | (1-a)(1-aq) = 1 - a - aq + a^2*q |
| 0 | any | 1 |
| 1 | 1 | 0 |
| q | 2 | (1-q)(1-q^2) = 1 - q - q^2 + q^3 |
| q^2 | 3 | (1-q^2)(1-q^3)(1-q^4) |
| 1 | inf | (q;q)_inf (Euler function -- verified in Phase 2) |
| q | inf | (q;q)_inf (same, just shifted start) |
| -1 | inf | (-1;q)_inf = prod(1+q^k) for k>=0 ... but first factor is (1-(-1))=2, careful! |
| generic a | -1 | 1/(1 - a/q) |
| generic a | -5 | 1/[(a/q^5; q)_5] |

**Important for negative n with a as QRat:** When a is a concrete rational number, `(a; q)_{-n}` involves terms like `a/q^k` which have negative q-powers. The resulting series may have a Laurent series (negative exponents). The FPS already supports negative exponent keys in `BTreeMap<i64, QRat>`, but we need to be careful about the truncation semantics. For `a` as a pure rational (no q-dependence), the finite product `(a*q^{-n}; q)_n` produces a polynomial in q with possibly negative powers.

**CRITICAL DESIGN DECISION -- Handling symbolic 'a' parameter:**
In Garvan's Maple package, `a` can be symbolic (containing q). For Phase 3, we restrict `a` to the form `c * q^m` where `c` is a QRat and `m` is an integer. This covers the important cases (a = q, a = q^2, a = -q, etc.) while keeping the FPS framework (which maps exponents to coefficients). A fully symbolic `a` would require multivariate series or symbolic expression manipulation, which is Phase 6+ territory.

Represent this as:
```rust
/// A q-monomial: c * q^m, used as the 'a' parameter in aqprod.
pub struct QMonomial {
    pub coeff: QRat,  // c
    pub power: i64,   // m (the exponent of q)
}
```

### QSER-02: q-Binomial Coefficient -- qbin(n, k, q)

**Definition:**
```
[n choose k]_q = (q; q)_n / [(q; q)_k * (q; q)_{n-k}]
               = prod_{i=0}^{k-1} (1 - q^{n-i}) / (1 - q^{i+1})
```

This is always a polynomial in q (the Gaussian polynomial). It has degree k*(n-k).

**Properties:**
- `[n choose 0]_q = 1`
- `[n choose n]_q = 1`
- `[n choose 1]_q = 1 + q + q^2 + ... + q^{n-1}`
- `[n choose k]_q = [n choose n-k]_q` (symmetry)
- At q=1: `[n choose k]_1 = C(n,k)` (ordinary binomial)

**Implementation:** Compute as the ratio of finite products. Since this is a polynomial (not an infinite series), compute it exactly. Can use the iterative formula to avoid large intermediate products:
```
[n choose k]_q = prod_{i=1}^{k} (1 - q^{n-k+i}) / (1 - q^i)
```

**Test values:**
- `[4 choose 2]_q = 1 + q + 2q^2 + q^3 + q^4`
- `[5 choose 2]_q = 1 + q + 2q^2 + 2q^3 + 2q^4 + q^5 + q^6`

### QSER-03: etaq(b, t, q) -- Eta-Quotient Building Block

**Definition in Garvan's package:**
```
etaq(b, t, q, T) = prod_{n=0}^{inf} (1 - q^{b+tn})
                 = (q^b; q^t)_inf
```
where b and t are positive integers. This expands as a series to O(q^T).

This is a q-Pochhammer symbol with specific integer parameters: `(q^b; q^t)_inf`.

**Implementation:** Direct delegation to aqprod with `a = q^b` as a QMonomial:
```rust
pub fn etaq(b: i64, t: i64, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    // etaq(b, t, q) = (q^b; q^t)_inf = prod_{n>=0} (1 - q^{b + t*n})
    // This is a q-Pochhammer with modified nome step
    // Build using InfiniteProductGenerator with factor k = (1 - q^{b+tk})
    // ...
}
```

Note: `etaq` uses a different q-step (q^t instead of q), so it cannot directly use `qpochhammer_inf_generator` which assumes step q^1. We need a generalized generator or construct factors manually.

**Generalized generator approach:**
```rust
// Factor k: (1 - q^{b + t*k})
InfiniteProductGenerator::new(
    FormalPowerSeries::one(var, trunc),
    0, // start at k=0
    Box::new(move |k, var, trunc| {
        let exp = b + t * k;
        let mut factor = FormalPowerSeries::one(var, trunc);
        if exp < trunc {
            factor.set_coeff(exp, -QRat::one());
        }
        factor
    }),
)
```

The generator naturally terminates when `b + t*k >= trunc`.

**Relationship to Dedekind eta:**
```
eta(tau) = q^{1/24} * prod_{n=1}^{inf} (1 - q^n) = q^{1/24} * etaq(1, 1, q)
```
(where q = e^{2*pi*i*tau}). The q^{1/24} factor is the "eta prefactor" not captured by etaq alone.

### QSER-04: jacprod(a, q) -- Jacobi Product

**Definition:**
In Garvan's notation, `JAC(a, b, inf)` where 0 < a < b:
```
JAC(a, b, inf) = (q^a; q^b)_inf * (q^{b-a}; q^b)_inf * (q^b; q^b)_inf
```

The function `jacprod` takes integer parameters and computes this triple product.

**As a triple product of etaq calls:**
```
jacprod(a, b, q, T) = etaq(a, b, q, T) * etaq(b-a, b, q, T) * etaq(b, b, q, T)
```

**Implementation:** Three calls to `etaq`, multiplied together.

### QSER-05: tripleprod(a, q) -- Jacobi Triple Product

**Definition (Jacobi Triple Product Identity):**
```
tripleprod(z, q, T) = (q; q)_inf * (z; q)_inf * (q/z; q)_inf
                    = sum_{n=-inf}^{inf} (-1)^n * z^n * q^{n(n-1)/2}
```

More precisely, using the product form:
```
prod_{n=1}^{inf} (1 - q^n)(1 - z*q^{n-1})(1 - q^n/z)
```

Wait, the standard form is:
```
prod_{n=1}^{inf} (1 - q^n)(1 + z*q^n)(1 + z^{-1}*q^{n-1})
```
which equals `sum_{n=-inf}^{inf} z^n * q^{n(n+1)/2}` (note: conventions vary).

**Garvan's convention:** In the qseries package, `tripleprod(z, q, T)` computes:
```
tripleprod(z, q, T) = prod_{n>=1} (1 - q^n) * prod_{n>=0} (1 - z*q^n) * prod_{n>=1} (1 - q^n/z)
```

For z as a QMonomial `c*q^m`, this breaks into three infinite products, each computed via a generator.

**Special case z=1:** Gives `theta3(q)` (verified in Phase 2 tests with z=1 variant of Jacobi triple product).

### QSER-06: quinprod(z, q) -- Quintuple Product

**Definition:**
```
quinprod(z, q, T) = prod_{n>=1} (1-q^n)(1-z*q^n)(1-z^{-1}*q^{n-1})(1-z^2*q^{2n-1})(1-z^{-2}*q^{2n-1})
```

Equals the series:
```
sum_{m=-inf}^{inf} (z^{3m} - z^{-3m-1}) * q^{m(3m+1)/2}
```

This is five infinite product factors. For z as a QMonomial, each factor maps to a q-Pochhammer generator.

### QSER-07: winquist(a, b, q) -- Winquist Product

**Definition (Winquist's Identity):**
```
winquist(a, b, q, T) = prod_{n>=1} (1-q^n) *
    [f(a,q)*g(b,q) - f(-a,q)*g(-b,q)]
```

where f and g involve theta-function-like products. The precise form (from the literature):

Winquist's identity states:
```
prod_{n>=1} (1-q^n)^2 * prod_{n>=0}(1-aq^n)(1-a^{-1}q^{n+1})(1-bq^n)(1-b^{-1}q^{n+1})(1-abq^n)(1-a^{-1}b^{-1}q^{n+1})(1-ab^{-1}q^n)(1-a^{-1}bq^{n+1})
```

This equals a specific theta-function expression. It involves 10 infinite products.

Garvan's `winquist(a, b, q, T)` takes two parameters a and b (as QMonomials) and produces the series expansion to O(q^T).

**Implementation:** Build each infinite product factor as a generator, multiply them sequentially.

### QSER-08: Theta Functions -- theta2(q), theta3(q), theta4(q)

**Definitions (at z=0, using nome q):**

**theta3(q):**
```
theta3(q) = sum_{n=-inf}^{inf} q^{n^2} = 1 + 2q + 2q^4 + 2q^9 + 2q^{16} + ...
```
Product form:
```
theta3(q) = prod_{n>=1} (1 - q^{2n})(1 + q^{2n-1})^2
          = (q^2; q^2)_inf * (-q; q^2)_inf^2
```

**theta4(q):**
```
theta4(q) = sum_{n=-inf}^{inf} (-1)^n * q^{n^2} = 1 - 2q + 2q^4 - 2q^9 + 2q^{16} - ...
```
Product form:
```
theta4(q) = prod_{n>=1} (1 - q^{2n})(1 - q^{2n-1})^2
          = (q^2; q^2)_inf * (q; q^2)_inf^2
```

**theta2(q):**
```
theta2(q) = 2*q^{1/4} * sum_{n=0}^{inf} q^{n(n+1)} = 2*q^{1/4} * (1 + q^2 + q^6 + q^{12} + ...)
```
Product form:
```
theta2(q) = 2*q^{1/4} * prod_{n>=1} (1 - q^{2n})(1 + q^{2n})^2
```

**CRITICAL: The q^{1/4} issue in theta2.**

The standard theta2(q) involves `q^{1/4}`, which is not a rational power of q. In Garvan's package, this is handled by working with `q^{1/4}` as a formal symbol, or by substituting `q -> q^4` so that `q^{1/4} -> q`.

For our FPS framework (which uses integer exponents only), two approaches:

**Approach A: Series in q, with fractional exponents.** Allow the BTreeMap to store keys that represent fractional exponents (multiply all exponents by 4). This changes the semantics of the FPS.

**Approach B: Provide theta2_q4(q) which is theta2 evaluated at q^4.** Then `theta2_q4(q) = 2*q * prod_{n>=1}(1-q^{8n})(1+q^{8n})^2`. All exponents are integers.

**Approach C (recommended): Use the Garvan convention.** In Garvan's qseries package, `theta2(q,T)` returns the expansion of `theta2(q)` as a series in `q^{1/4}`, where the variable is understood to be `q^{1/4}`. The returned series has integer powers of `q^{1/4}`. This is equivalent to computing with an auxiliary variable.

**Practical recommendation:** Follow Garvan's convention. For theta3 and theta4, the series are in integer powers of q (no issue). For theta2, document that the returned series is in `q^{1/4}` or alternatively provide a version that takes the fourth-root substitution. Given that our BTreeMap supports arbitrary integer exponents including negatives, we can multiply all exponents by 4 and work in a "4x-scaled" mode where the FPS represents `f(q^{1/4})` but stores exponents as if the variable were `q^{1/4}`.

**Simplest approach for Phase 3:** Compute theta2 using the product form with the substitution. Return a FPS where the "variable" represents `q^{1/4}`:
```
theta2 series: 2*X + 2*X^5 + 2*X^9 + ...  where X = q^{1/4}
```
with coefficients at exponents 1, 5, 9, 13, ... (these are 4n+1 for n=0,1,2,..., corresponding to `q^{(2n+1)^2/4}`).

Document clearly that theta2 returns series in `q^{1/4}`.

**Test values:**

theta3 to O(q^50): nonzero at perfect squares 0, 1, 4, 9, 16, 25, 36, 49 with coefficients 1, 2, 2, 2, 2, 2, 2, 2.

theta4 to O(q^50): nonzero at perfect squares with alternating signs: coeff(0)=1, coeff(1)=-2, coeff(4)=2, coeff(9)=-2, coeff(16)=2, coeff(25)=-2, coeff(36)=2, coeff(49)=-2.

**Theta identity for verification:**
```
theta3(q)^2 = theta4(q)^2 + theta2(q)^2
```
(Jacobi's identity relating the three theta functions -- after appropriate normalization).

### PART-01: Partition Function p(n)

**Already substantially implemented in Phase 2.** The generating function `1/(q;q)_inf` is computed by inverting the Euler function, and p(n) values are verified to p(20) in Phase 2 tests.

Phase 3 needs to:
1. Provide a clean `partition_count(n)` API that extracts p(n) for a single value
2. Extend verification to p(200) matching OEIS A000041
3. Implement the pentagonal number recurrence for efficient single-value computation:
```
p(n) = sum_{k=1}^{...} (-1)^{k+1} [p(n - k(3k-1)/2) + p(n - k(3k+1)/2)]
```
This is O(n * sqrt(n)) per value, much faster than computing the full series to O(q^{n+1}).

**Key p(n) values for extended verification (OEIS A000041):**
```
p(50)  = 204226
p(100) = 190569292
p(200) = 3972999029388
```

### PART-02: Restricted Partition Generating Functions

**Distinct parts:**
```
Q(q) = prod_{k=1}^{inf} (1 + q^k) = (-q; q)_inf
```
where Q(n) = number of partitions of n into distinct parts.

Alternatively: `prod_{k=1}^{inf} 1/(1 - q^{2k-1})` (partitions into odd parts -- Euler's theorem).

**OEIS A000009** (distinct parts): 1, 1, 1, 2, 2, 3, 4, 5, 6, 8, 10, 12, 15, 18, 22, 27, 32, 38, 46, 54, 64

**Odd parts generating function:**
```
prod_{k=0}^{inf} 1/(1 - q^{2k+1})
```
This equals the distinct parts generating function (Euler's theorem).

**Bounded parts -- at most m parts:**
```
prod_{k=1}^{m} 1/(1 - q^k)
```

**Parts bounded by N:**
```
prod_{k=1}^{N} 1/(1 - q^k)
```

**Implementation:** Each of these is a product of simple factors, using `InfiniteProductGenerator` or finite product loops followed by series inversion.

### PART-03: Rank and Crank

**Rank generating function:**
```
R(z, q) = 1 + sum_{n=1}^{inf} q^{n^2} / prod_{k=1}^{n} (1 - z*q^k)(1 - q^k/z)
```
where R(z,q) = sum_{m,n} N(m,n) * z^m * q^n and N(m,n) is the number of partitions of n with rank m.

At z=1: `R(1, q) = sum p(n) q^n = 1/(q;q)_inf`.

At z=-1: `R(-1, q)` gives the generating function for the rank difference N(even rank, n) - N(odd rank, n).

**Crank generating function:**
```
C(z, q) = prod_{n=1}^{inf} (1-q^n) / [(1-z*q^n)(1-q^n/z)]
        = (q; q)_inf / [(z*q; q)_inf * (q/z; q)_inf]
```

At z=1: `C(1, q) = 1/(q;q)_inf` (same as partition generating function -- the 1/0 at n=0 cancels).

**Implementation considerations for rank:** The rank generating function involves a sum over n of `q^{n^2} / (zq; q)_n / (q/z; q)_n`. For each n, we compute the partial product in the denominator and accumulate. This is NOT a simple infinite product -- it's a sum of rational functions. Implementation:

```rust
pub fn rank_generating_function(
    z: &QRat,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::one(variable, truncation_order); // n=0 term

    // For n = 1, 2, ..., while n^2 < truncation_order
    let mut n: i64 = 1;
    while n * n < truncation_order {
        // Compute q^{n^2} / (zq;q)_n / (q/z;q)_n
        let numerator = FormalPowerSeries::monomial(variable, QRat::one(), n*n, truncation_order);
        let denom1 = aqprod_finite_positive(&(z.clone()), variable, n, truncation_order);
        // ... shift appropriately for (z*q; q)_n vs (z; q)_n
        let denom2 = aqprod_finite_positive(&(QRat::one() / z.clone()), variable, n, truncation_order);
        let inv_denom = arithmetic::invert(&arithmetic::mul(&denom1, &denom2));
        let term = arithmetic::mul(&numerator, &inv_denom);
        result = arithmetic::add(&result, &term);
        n += 1;
    }
    result
}
```

**Crank implementation:** Simpler -- it's a ratio of infinite products.

```rust
pub fn crank_generating_function(
    z: &QRat,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    // (q;q)_inf / [(zq;q)_inf * (q/z;q)_inf]
    let euler = aqprod(&QMonomial::new(QRat::one(), 1), variable, PochhammerOrder::Infinite, truncation_order);
    let denom1 = aqprod(&QMonomial::new(z.clone(), 1), variable, PochhammerOrder::Infinite, truncation_order);
    let denom2 = aqprod(&QMonomial::new(QRat::one() / z.clone(), 1), variable, PochhammerOrder::Infinite, truncation_order);
    let denom = arithmetic::mul(&denom1, &denom2);
    arithmetic::mul(&euler, &arithmetic::invert(&denom))
}
```

## Common Pitfalls

### Pitfall 1: q-Pochhammer with a=1 and Positive Finite n

**What goes wrong:** `aqprod(1, q, n)` with n >= 1 should be 0 because the first factor is `(1 - 1) = 0`. But if the implementation computes the product iteratively and multiplies FPS objects, it might miss this and produce a nonzero result due to truncation artifacts.
**Why it happens:** The first factor `(1 - a*q^0) = (1 - 1) = 0` when a=1. The entire product is 0.
**How to avoid:** Check for zero factors early. If any factor in the finite product is identically zero (its constant term is zero AND it's a monomial), short-circuit to the zero series.
**Warning signs:** `aqprod(1, q, 5)` returns a nonzero series.

### Pitfall 2: Negative q-Pochhammer Order with Pole at q=0

**What goes wrong:** `(a; q)_{-n}` involves `1/(1 - a/q^k)` terms. If a is not a power of q, this creates Laurent series with negative powers. The FPS might not handle this correctly if the implementation assumes nonnegative exponents.
**Why it happens:** The BTreeMap<i64, QRat> supports negative keys, but arithmetic operations must handle them correctly through the truncation logic.
**How to avoid:** Our BTreeMap already supports i64 keys (negative exponents). Test with negative-exponent cases explicitly. The `shift()` function already handles negative shifts.
**Warning signs:** Negative exponent coefficients are silently dropped; tests with negative n produce wrong results.

### Pitfall 3: Theta2 Fractional Exponents

**What goes wrong:** theta2(q) = 2*q^{1/4} + 2*q^{9/4} + ... has non-integer exponents. Trying to store this in a BTreeMap<i64, QRat> with integer keys loses information.
**Why it happens:** The standard theta2 definition involves q^{1/4}.
**How to avoid:** Either (a) work with q^{1/4} as the series variable (multiply all exponents by 4), or (b) provide theta2 in the "q^4 substitution" form. Document the convention clearly.
**Warning signs:** theta2 series has missing or wrong coefficients; theta identity `theta2^2 + theta4^2 = theta3^2` fails.

### Pitfall 4: InfiniteProductGenerator Reuse vs Fresh Construction

**What goes wrong:** If two functions both need `(q;q)_inf`, one might try to share a generator. But generators are stateful (they own the partial product). Sharing requires careful management.
**Why it happens:** Efficiency optimization gone wrong.
**How to avoid:** For Phase 3, create fresh generators for each product computation. The cost of re-expanding `(q;q)_inf` is O(N^2) for N terms, which is acceptable for truncation orders up to a few hundred. Caching/sharing is a Phase 4+ optimization.
**Warning signs:** One function's result is corrupted after another function modifies a shared generator.

### Pitfall 5: Truncation Order Mismatch in Product Chains

**What goes wrong:** When computing `f * g * h`, if f, g, h have different truncation orders, the result has `min(trunc_f, trunc_g, trunc_h)`. If you're not careful, creating generators with different truncation orders leads to surprising precision loss.
**Why it happens:** Each generator is created with a fixed truncation order. If one is created with trunc=50 and another with trunc=100, their product is only valid to O(q^50).
**How to avoid:** Always pass the same truncation order to all generators within a single computation. Use a single `truncation_order` parameter that flows through to all sub-computations.
**Warning signs:** Results are correct to O(q^50) but wrong beyond that, even though the user requested O(q^100).

### Pitfall 6: Winquist Product Performance

**What goes wrong:** Winquist's product involves 10 infinite product factors. Multiplying 10 series of O(q^N) naively is O(N^2) per multiplication, so O(10 * N^2) total. For N=200, this is significant.
**Why it happens:** Each multiplication produces a series with up to N nonzero terms, and multiplying two N-term series is O(N^2).
**How to avoid:** This is expected behavior for Phase 3 -- the O(N^2) cost per multiplication is inherent to the algorithm. For N up to a few hundred, it completes in seconds. Optimization (FFT-based multiplication, caching) is deferred to Phase 4+.
**Warning signs:** winquist(a, b, q, 1000) takes minutes. This is a performance concern, not a correctness issue.

### Pitfall 7: Rank Generating Function Convergence

**What goes wrong:** The rank generating function is a sum over n of `q^{n^2} / (product)`. The sum effectively terminates when `n^2 >= truncation_order`. But the denominator `(zq;q)_n * (q/z;q)_n` for large n is a large polynomial being inverted, which is expensive.
**Why it happens:** Each term in the rank sum requires inverting a degree-n polynomial, which is O(T) where T is the truncation order. Total cost is O(sqrt(T) * T).
**How to avoid:** This is inherent complexity. For T=200, sqrt(200)~14, so ~14 terms, each requiring a T-term inversion. Acceptable for Phase 3.
**Warning signs:** Rank generating function is much slower than other functions. This is expected.

## Code Examples

### q-Pochhammer Finite Positive

```rust
/// Compute (a; q)_n for n > 0 where a = coeff * q^power.
fn aqprod_finite_positive(
    a: &QMonomial,
    variable: SymbolId,
    n: i64,
    truncation_order: i64,
) -> FormalPowerSeries {
    // Special case: if a.coeff is 1 and a.power is 0, first factor is (1-1)=0
    if a.coeff == QRat::one() && a.power == 0 && n > 0 {
        return FormalPowerSeries::zero(variable, truncation_order);
    }

    let mut result = FormalPowerSeries::one(variable, truncation_order);
    for k in 0..n {
        // Factor k: (1 - a * q^k) = (1 - coeff * q^{power + k})
        let exp = a.power + k;
        let mut factor = FormalPowerSeries::one(variable, truncation_order);
        if exp >= 0 && exp < truncation_order {
            factor.set_coeff(exp, -a.coeff.clone());
        } else if exp < 0 {
            // Negative exponent: this factor has a term at q^exp
            // BTreeMap supports negative keys
            factor.set_coeff(exp, -a.coeff.clone());
        }
        result = arithmetic::mul(&result, &factor);
    }
    result
}
```

### Theta3 via Product Representation

```rust
/// theta3(q) = (q^2; q^2)_inf * (-q; q^2)_inf^2
///
/// = prod_{n>=1} (1 - q^{2n}) * prod_{n>=0} (1 + q^{2n+1})^2
pub fn theta3(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    // Factor 1: (q^2; q^2)_inf = prod_{n>=1} (1 - q^{2n})
    let mut gen1 = InfiniteProductGenerator::new(
        FormalPowerSeries::one(variable, truncation_order),
        1, // start at n=1
        Box::new(move |n, var, trunc| {
            let mut f = FormalPowerSeries::one(var, trunc);
            let exp = 2 * n;
            if exp < trunc {
                f.set_coeff(exp, -QRat::one());
            }
            f
        }),
    );
    gen1.ensure_order(truncation_order);

    // Factor 2: (-q; q^2)_inf = prod_{n>=0} (1 + q^{2n+1})
    let mut gen2 = InfiniteProductGenerator::new(
        FormalPowerSeries::one(variable, truncation_order),
        0, // start at n=0
        Box::new(move |n, var, trunc| {
            let mut f = FormalPowerSeries::one(var, trunc);
            let exp = 2 * n + 1;
            if exp < trunc {
                f.set_coeff(exp, QRat::one()); // +1 not -1 (because of the minus in -q)
            }
            f
        }),
    );
    gen2.ensure_order(truncation_order);

    // theta3 = gen1 * gen2^2
    let p1 = gen1.into_series();
    let p2 = gen2.into_series();
    let p2_sq = arithmetic::mul(&p2, &p2);
    arithmetic::mul(&p1, &p2_sq)
}
```

### Pentagonal Number Recurrence for p(n)

```rust
/// Compute p(n) using the pentagonal number recurrence.
/// O(n * sqrt(n)) time, O(n) space.
pub fn partition_count(n: i64) -> QRat {
    if n < 0 { return QRat::zero(); }
    if n == 0 { return QRat::one(); }

    // Build table from 0 to n
    let n = n as usize;
    let mut table = vec![QRat::zero(); n + 1];
    table[0] = QRat::one();

    for i in 1..=n {
        let mut sum = QRat::zero();
        let mut k: i64 = 1;
        loop {
            // Generalized pentagonal numbers: k(3k-1)/2 and k(3k+1)/2
            let pent1 = (k * (3 * k - 1) / 2) as usize;
            let pent2 = (k * (3 * k + 1) / 2) as usize;
            let sign = if k % 2 == 1 { QRat::one() } else { -QRat::one() };

            if pent1 > i { break; }
            sum = sum + sign.clone() * table[i - pent1].clone();

            if pent2 <= i {
                sum = sum + sign * table[i - pent2].clone();
            }
            k += 1;
        }
        table[i] = sum;
    }

    table[n].clone()
}
```

### Test: Verify theta3^2 = 1 + 4*sum_{n>=0} [q^{4n+1}/(1-q^{4n+1}) - q^{4n+3}/(1-q^{4n+3})]

```rust
#[test]
fn theta3_squared_formula() {
    let q = q_var();
    let trunc = 50;
    let t3 = theta3(q, trunc);
    let t3_sq = arithmetic::mul(&t3, &t3);

    // theta3(q)^2 = sum_{n=-inf}^{inf} r_2(n) * q^n
    // where r_2(n) is the number of representations of n as a sum of 2 squares
    // r_2(0)=1, r_2(1)=4, r_2(2)=4, r_2(3)=0, r_2(4)=4, r_2(5)=8, ...
    assert_eq!(t3_sq.coeff(0), qrat(1));
    assert_eq!(t3_sq.coeff(1), qrat(4));
    assert_eq!(t3_sq.coeff(2), qrat(4));
    assert_eq!(t3_sq.coeff(3), qrat(0));
    assert_eq!(t3_sq.coeff(4), qrat(4));
    assert_eq!(t3_sq.coeff(5), qrat(8));
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Ad-hoc product construction (Phase 2 tests) | Unified qseries module with named functions | Phase 3 | Clean API, Garvan-compatible function names |
| Only euler_function_generator and qpochhammer_inf_generator | General etaq-style generators with arbitrary step | Phase 3 | Supports all named products |
| p(n) only via series inversion | Both series inversion AND pentagonal recurrence | Phase 3 | Efficient single-value computation for large n |
| Theta functions tested ad-hoc in Phase 2 | Proper theta2/theta3/theta4 functions | Phase 3 | Ready for identity verification |

**Deprecated/outdated:**
- The Phase 2 test for `jacobi_triple_product_z1_theta3` manually constructs the products. Phase 3 replaces this with a proper `theta3()` function.
- The Phase 2 `euler_function_generator` remains but is subsumed by the more general `etaq(1, 1, q, T)`.

## Open Questions

1. **theta2 exponent convention**
   - What we know: theta2 involves q^{1/4}, which is a fractional exponent. Our FPS uses integer exponents.
   - What's unclear: Should we (a) multiply all exponents by 4 and work in q^{1/4} mode, (b) provide a separate theta2_rescaled function, or (c) introduce a scaling parameter to the FPS?
   - Recommendation: For Phase 3, use approach (a): compute theta2 as a series in X = q^{1/4}, with exponents 1, 5, 9, 13, ... (the "4n+1" exponents). Document this clearly. The FPS variable is conceptually q^{1/4} for theta2 only. This matches Garvan's convention where theta2 returns a series in q^{1/4}. For theta identity verification (`theta2^2 + theta4^2 = theta3^2`), rescale theta4 and theta3 to work in the q^{1/4} variable too.

2. **QMonomial representation for 'a' parameter**
   - What we know: Many functions need `a = c * q^m` as a parameter. A struct `QMonomial { coeff: QRat, power: i64 }` captures this.
   - What's unclear: Is QMonomial sufficient for all Phase 3 use cases, or do we need more general symbolic a?
   - Recommendation: QMonomial is sufficient for Phase 3. All named products and theta functions use parameters of this form. Fully symbolic parameters are Phase 6+ (hypergeometric series).

3. **Performance of winquist with 10 product factors**
   - What we know: Winquist involves multiplying 10 infinite product series. Each multiplication is O(T^2).
   - What's unclear: For T=200 or T=500, is this acceptable performance?
   - Recommendation: Acceptable for Phase 3. T=200 with 10 multiplications is ~10 * 200^2 * (cost per coefficient op) = ~400K rational arithmetic ops. GMP handles this in under a second. Profile and optimize in Phase 4 if needed.

4. **Rank generating function z parameter**
   - What we know: The rank GF R(z,q) is bivariate. For a fixed z (as QRat), it becomes a single-variable FPS in q.
   - What's unclear: How to handle z as a formal variable (not a number) for symbolic rank computations.
   - Recommendation: Phase 3 evaluates R(z,q) at specific z values (z = 1, z = -1, z = zeta_k for primitive roots). Symbolic z is Phase 4+ (relation discovery).

## Test Data

### Partition Function p(n) (OEIS A000041) -- Extended

```
p(0)=1, p(1)=1, p(2)=2, p(3)=3, p(4)=5, p(5)=7,
p(6)=11, p(7)=15, p(8)=22, p(9)=30, p(10)=42,
p(11)=56, p(12)=77, p(13)=101, p(14)=135, p(15)=176,
p(16)=231, p(17)=297, p(18)=385, p(19)=490, p(20)=627,
p(30)=5604, p(40)=37338, p(50)=204226,
p(100)=190569292, p(200)=3972999029388
```

### Distinct Parts Q(n) (OEIS A000009)

```
Q(0)=1, Q(1)=1, Q(2)=1, Q(3)=2, Q(4)=2, Q(5)=3,
Q(6)=4, Q(7)=5, Q(8)=6, Q(9)=8, Q(10)=10,
Q(11)=12, Q(12)=15, Q(13)=18, Q(14)=22, Q(15)=27,
Q(16)=32, Q(17)=38, Q(18)=46, Q(19)=54, Q(20)=64
```

### theta3(q) Coefficients

Nonzero only at perfect squares: coeff(0)=1, coeff(n^2)=2 for n >= 1.
```
theta3 = 1 + 2q + 2q^4 + 2q^9 + 2q^16 + 2q^25 + 2q^36 + 2q^49 + ...
```

### theta4(q) Coefficients

Nonzero only at perfect squares with alternating signs:
```
theta4 = 1 - 2q + 2q^4 - 2q^9 + 2q^16 - 2q^25 + 2q^36 - 2q^49 + ...
```
coeff(0)=1, coeff(n^2) = 2*(-1)^n for n >= 1.

### q-Binomial [4 choose 2]_q

```
[4 choose 2]_q = 1 + q + 2*q^2 + q^3 + q^4
```

### q-Binomial [5 choose 2]_q

```
[5 choose 2]_q = 1 + q + 2*q^2 + 2*q^3 + 2*q^4 + q^5 + q^6
```

### Sum of Two Squares r_2(n) (OEIS A004018) -- theta3^2 coefficients

```
r_2(0)=1, r_2(1)=4, r_2(2)=4, r_2(3)=0, r_2(4)=4,
r_2(5)=8, r_2(6)=0, r_2(7)=0, r_2(8)=4, r_2(9)=4,
r_2(10)=8, r_2(11)=0, r_2(12)=0, r_2(13)=8, r_2(14)=0,
r_2(15)=0, r_2(16)=4, r_2(17)=8, r_2(18)=4, r_2(19)=0,
r_2(20)=8
```

### Crank Moment Verification

At z=-1, the crank generating function gives:
```
C(-1, q) = (q;q)_inf / [(-q;q)_inf * (-1;q)_inf]
```
The first factor of (-1;q)_inf is (1-(-1))=2, so this simplifies to known results.

Ramanujan's congruences (verified via crank):
- p(5n+4) = 0 (mod 5) -- crank sorts partitions into 5 residue classes
- p(7n+5) = 0 (mod 7) -- crank sorts partitions into 7 residue classes
- p(11n+6) = 0 (mod 11) -- this uses Winquist's identity

## Sources

### Primary (HIGH confidence)
- [OEIS A000041](https://oeis.org/A000041) -- Partition function values, definitive source
- [OEIS A000009](https://oeis.org/A000009) -- Distinct parts partition function
- [OEIS A004018](https://oeis.org/A004018) -- r_2(n) sum of two squares representation counts
- Existing Phase 2 codebase (generator.rs, arithmetic.rs, mod.rs) -- Verified by 253 passing tests
- Phase 2 RESEARCH.md -- Architecture decisions and patterns

### Secondary (MEDIUM confidence)
- [Garvan qseries package function list](https://qseries.org/fgarvan/qmaple/qseries/1.2/maple16-win64/) -- Verified complete function list: aqprod, etaq, jacprod, tripleprod, quinprod, winquist, theta2, theta3, theta4, qbin, etc.
- [Garvan q-product tutorial (arXiv:math/9812092)](https://arxiv.org/abs/math/9812092) -- Mathematical definitions and algorithms
- [Wikipedia: q-Pochhammer symbol](https://en.wikipedia.org/wiki/Q-Pochhammer_symbol) -- Standard definitions
- [Wikipedia: Jacobi triple product](https://en.wikipedia.org/wiki/Jacobi_triple_product) -- Triple product identity
- [Wikipedia: Crank of a partition](https://en.wikipedia.org/wiki/Crank_of_a_partition) -- Andrews-Garvan crank definition
- [Wikipedia: Rank of a partition](https://en.wikipedia.org/wiki/Rank_of_a_partition) -- Dyson rank definition
- [Wolfram MathWorld: Quintuple Product Identity](https://mathworld.wolfram.com/QuintupleProductIdentity.html) -- Explicit formula
- [Grokipedia: q-Pochhammer symbol](https://grokipedia.com/page/Q-Pochhammer_symbol) -- Negative order formula verified
- [Grokipedia: Theta function](https://grokipedia.com/page/Theta_function) -- Series and product forms

### Tertiary (LOW confidence)
- Winquist's identity explicit formula -- Confirmed it involves 10 product factors from multiple sources, but exact Garvan calling convention (2-parameter a,b form) needs validation against actual Maple output during implementation.
- theta2 in Garvan's convention -- The exact handling of q^{1/4} in Garvan's package needs validation. The approach described here (series in q^{1/4}) is based on standard mathematical convention but should be verified against Garvan's actual output.
- Rank generating function formula -- Standard from literature but the denominator product indexing (starting at k=1 vs k=0) needs careful verification during implementation.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- No new dependencies; everything composes from Phase 2 infrastructure
- Architecture (module structure): HIGH -- Natural decomposition by function type; follows Garvan's organization
- Mathematical definitions: HIGH for theta3/theta4, q-Pochhammer, q-binomial, partitions; MEDIUM for theta2 (exponent convention), quinprod (formula complexity), winquist (exact Garvan API)
- Pitfalls: HIGH -- Well-understood computational number theory issues
- Code examples: MEDIUM -- Based on established algorithms adapted to our specific FPS API; will need refinement during implementation
- Test data: HIGH -- OEIS values are definitive; theta function coefficients are textbook

**Research date:** 2026-02-13
**Valid until:** 2026-03-15 (stable mathematical domain; no fast-moving dependencies)
