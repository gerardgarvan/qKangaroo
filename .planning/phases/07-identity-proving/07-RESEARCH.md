# Phase 7: Identity Proving - Research

**Researched:** 2026-02-14
**Domain:** Modular forms, cusp theory, valence formula, eta quotients, Jacobi products
**Confidence:** MEDIUM-HIGH (mathematical theory well-understood; Garvan source code partially inspected)

## Summary

Phase 7 implements automatic identity proving for q-series via the valence formula method, matching Garvan's `thetaids` and `ETA` Maple packages. The core idea: to prove that two q-series expressions are equal, express their difference as a modular function on a congruence subgroup (Gamma_0(N) or Gamma_1(N)), compute the order of vanishing at every cusp, and if all orders are non-negative and the total is bounded, the function must be a constant -- which can be checked by examining the first coefficient of the q-expansion.

The implementation requires three layers: (1) symbolic representation models for JAC (Jacobi products) and ETA (eta quotients) that capture product structure as data rather than expanding to formal power series, (2) number-theoretic algorithms for computing cusps of congruence subgroups and orders of modular functions at those cusps, and (3) the proving engine that orchestrates the valence formula check. A fourth component -- the identity database -- is a TOML-based searchable collection of verified identities.

**Primary recommendation:** Implement in four plans: (1) JAC and ETA symbolic models with conversion, (2) cusp computation and order-at-cusp formulas, (3) the proving engine (provemodfuncid / provemodfuncGAMMA0id), and (4) identity database with Python API bindings.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.x | Serialization for identity database TOML | Already in project (QInt, QRat derive Serialize/Deserialize) |
| toml | 0.8.x | TOML parsing/writing for identity database | De facto Rust TOML crate, 28M+ monthly downloads |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rug | (existing) | GCD, integer arithmetic for cusp computation | All number theory computations |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| TOML for identity DB | JSON or SQLite | TOML is human-readable/editable, matches project constraint; JSON lacks comments; SQLite adds dependency |
| Custom cusp algorithm | num-modular crate | Custom is simple enough (just GCD + Euler phi); external crate adds dependency for trivial benefit |

**Installation:**
```bash
cargo add toml --package qsym-core
# serde already present
```

## Architecture Patterns

### Recommended Project Structure
```
crates/qsym-core/src/qseries/
    identity/
        mod.rs          # Module re-exports
        jac.rs          # JAC symbolic representation
        eta.rs          # ETA symbolic representation
        cusps.rs        # Cusp computation for Gamma_0(N) and Gamma_1(N)
        orders.rs       # Order-at-cusp formulas
        prove.rs        # provemodfuncid, provemodfuncGAMMA0id
        database.rs     # Identity database (TOML load/save/search)
    mod.rs              # Add `pub mod identity;`
data/
    identities/         # TOML identity files
        eta_identities.toml
        theta_identities.toml
        ramanujan_identities.toml
```

### Pattern 1: Symbolic Product Representations (JAC and ETA)

**What:** Structured data types that capture the algebraic form of Jacobi products and eta quotients without expanding to formal power series. These are the inputs to the identity proving pipeline.

**When to use:** Whenever manipulating identities at the structural level (proving, converting between representations, displaying).

**JAC Representation:**
```rust
/// A single Jacobi triple product factor: JAC(a, b)^exponent
/// JAC(a, b) = (q^a; q^b)_inf * (q^{b-a}; q^b)_inf * (q^b; q^b)_inf
/// Constraint: 0 < a < b
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JacFactor {
    pub a: i64,
    pub b: i64,
    pub exponent: i64,
}

/// A symbolic JAC expression: scalar * q^shift * product of JAC(a_i, b_i)^{e_i}
#[derive(Clone, Debug)]
pub struct JacExpression {
    pub scalar: QRat,
    pub q_shift: QRat,        // fractional power of q prefactor
    pub factors: Vec<JacFactor>,
}
```

**ETA Representation:**
```rust
/// An eta quotient: product of eta(delta * tau)^{r_delta}
/// where eta(tau) = q^{1/24} * (q;q)_inf
///
/// The GP (generalized permutation) form from Garvan stores this as
/// a flat list [t1, r1, t2, r2, ...] but we use a BTreeMap for clarity.
#[derive(Clone, Debug)]
pub struct EtaExpression {
    /// Maps divisor delta -> exponent r_delta
    pub factors: BTreeMap<i64, i64>,
    /// The level N (all deltas must divide N)
    pub level: i64,
}

impl EtaExpression {
    /// Weight k = (1/2) * sum of r_delta
    pub fn weight(&self) -> QRat { ... }

    /// q-shift = sum(delta * r_delta) / 24
    pub fn q_shift(&self) -> QRat { ... }

    /// Check Newman's conditions for modularity on Gamma_0(N):
    /// 1. sum(delta * r_delta) = 0 (mod 24)
    /// 2. sum((N/delta) * r_delta) = 0 (mod 24)
    /// 3. product(delta^|r_delta|) is a perfect square
    /// 4. Each delta divides N
    pub fn check_modularity(&self) -> ModularityResult { ... }

    /// Order of vanishing at cusp c/d of Gamma_0(N) (Ligozat formula)
    pub fn order_at_cusp(&self, d: i64) -> QRat { ... }
}
```

### Pattern 2: Cusp Computation

**What:** Algorithms for enumerating inequivalent cusps of Gamma_0(N) and Gamma_1(N).

**Gamma_0(N) cusps (cuspmake):**
```rust
/// Compute inequivalent cusps for Gamma_0(N).
///
/// Algorithm (from Garvan's ETA package):
/// 1. Start with S = {infinity} (represented as 1/0)
/// 2. For each proper divisor c of N (c > 1):
///    a. Let gc = gcd(c, N/c)
///    b. Track seen residue classes mod gc
///    c. For each d in 1..c-1 with gcd(d,c)=1:
///       If d mod gc not yet seen, add d/c to S
/// 3. Return S
///
/// Cusp equivalence for Gamma_0(N):
/// Two cusps a1/c1 and a2/c2 are equivalent iff:
///   c1 = c2 (mod N) and a1 = a2 (mod gcd(c1, N))
///   OR c1 = -c2 (mod N) and a1 = -a2 (mod gcd(c1, N))
///
/// Number of cusps = sum_{d | N} phi(gcd(d, N/d))
pub fn cuspmake(n: i64) -> Vec<Cusp> { ... }
```

**Gamma_1(N) cusps (cuspmake1):**
```rust
/// Compute inequivalent cusps for Gamma_1(N).
///
/// Cusp equivalence for Gamma_1(N):
/// Two cusps u1/v1 and u2/v2 are equivalent iff:
///   v1 = v2 (mod N) and u1 = u2 (mod gcd(v1, N))
///   OR v1 = -v2 (mod N) and u1 = -u2 (mod gcd(v1, N))
///
/// The algorithm iterates over all reduced fractions a/c with
/// c dividing N, 0 <= a < c, gcd(a,c)=1, and groups them by
/// equivalence class, keeping one representative per class.
pub fn cuspmake1(n: i64) -> Vec<Cusp> { ... }
```

### Pattern 3: Order at Cusp (Ligozat Formula)

**What:** The formula for computing the order of vanishing of an eta quotient at a cusp.

**The Ligozat formula for eta quotients on Gamma_0(N):**

For an eta quotient f(tau) = prod_{delta | N} eta(delta * tau)^{r_delta}, the order of vanishing at cusp c/d (where d | N, gcd(c,d) = 1) is:

```
ord_{c/d}(f) = (N / 24) * sum_{delta | N} [ gcd(d, delta)^2 * r_delta / (gcd(d, N/d) * d * delta) ]
```

Key property: the order depends only on d (the denominator of the cusp), not on c (the numerator).

```rust
/// Compute the order of vanishing of an eta quotient at cusp c/d.
///
/// Uses the Ligozat formula:
///   ord = (N/24) * sum_{delta|N} gcd(d, delta)^2 * r_delta / (gcd(d, N/d) * d * delta)
///
/// The order only depends on d, not c.
pub fn eta_order_at_cusp(eta: &EtaExpression, d: i64) -> QRat { ... }
```

**For Garvan's ETA package, the order computation uses two variants:**
- `cuspord(etaprod, cusp)`: invariant order = sum over factors of gcd(t,c)^2 / t * r / 24
- `cuspORD(etaprod, N, cusp)`: weighted order = cuspord * cusp_width, where width = N / gcd(N, c^2)

### Pattern 4: Valence Formula and Proving Engine

**What:** The central theorem enabling automatic proofs.

**Valence Formula (weight 0 case, for modular functions):**
A modular function f on Gamma_0(N) of weight 0 that has no poles on the upper half-plane satisfies:
- The total order of f (sum of orders at all cusps) equals 0.
- If ord_s(f) >= 0 at every cusp s, then f is a constant.
- If additionally ord_s(f) > 0 at some cusp, then f = 0.

**Application to identity proving:**
To prove LHS = RHS:
1. Express both sides as eta quotients (or generalized eta products)
2. Form g = LHS/RHS (or LHS - RHS for additive identities)
3. Check g is a modular function on Gamma_0(N) (weight 0, satisfies Newman conditions)
4. Compute ord_s(g) at every cusp s of Gamma_0(N)
5. If all orders >= 0, then g is a constant
6. The constant is determined by checking the q-expansion to just 1 term (the Sturm bound gives an upper bound on how many terms are needed in general)

**Sturm Bound:**
For modular forms of weight k on Gamma_0(N), two forms are equal if their q-expansions agree up to:
```
B = floor(k * m / 12)
where m = [SL_2(Z) : Gamma_0(N)] = N * prod_{p | N} (1 + 1/p)
```
For weight 0 (modular functions), this simplifies: if all cusp orders are non-negative, check constant term only.

```rust
/// Proof result from the identity proving engine.
#[derive(Clone, Debug)]
pub enum ProofResult {
    /// Identity proved: all cusp orders non-negative, constant verified.
    Proved {
        level: i64,
        cusps: Vec<(Cusp, QRat)>,   // cusp -> order
        sturm_bound: i64,
        verification_terms: i64,
    },
    /// Not a modular function (fails Newman conditions).
    NotModular {
        failed_conditions: Vec<String>,
    },
    /// Cusp order is negative at some cusp (identity may be false).
    NegativeOrder {
        cusp: Cusp,
        order: QRat,
    },
    /// Numerical verification failed.
    CounterExample {
        coefficient_index: i64,
        expected: QRat,
        actual: QRat,
    },
}

/// Prove that an eta-quotient identity holds on Gamma_0(N).
///
/// Algorithm (matching Garvan's provemodfuncGAMMA0id):
/// 1. Parse identity into individual eta-product terms
/// 2. For each term, verify Newman's modularity conditions
/// 3. Compute cusps of Gamma_0(N) via cuspmake
/// 4. Compute order at each cusp for each term
/// 5. Compute minimum total order across all cusps
/// 6. Apply valence formula: if min total order >= 0, identity
///    is a constant; verify by q-expansion
pub fn prove_eta_identity(identity: &EtaIdentity, level: i64) -> ProofResult { ... }
```

### Pattern 5: Identity Database (TOML)

**What:** Searchable collection of verified identities stored as TOML files.

```toml
# Example identity entry
[[identity]]
id = "euler-pentagonal"
name = "Euler's Pentagonal Number Theorem"
tags = ["euler", "pentagonal", "partition", "classical"]
functions = ["eta"]

[identity.lhs]
type = "eta_quotient"
level = 1
factors = { 1 = 1 }  # eta(tau)

[identity.rhs]
type = "q_series"
formula = "sum_{n=-inf}^{inf} (-1)^n q^{n(3n-1)/2}"

[identity.proof]
method = "valence_formula"
level = 1
verified = true

[identity.citation]
author = "Euler"
year = 1750
reference = "De partitione numerorum"
```

### Anti-Patterns to Avoid
- **Expanding to FPS for structural operations:** JAC and ETA models should be manipulated symbolically. Only expand to FPS when doing numerical verification.
- **Conflating invariant order with weighted order:** Garvan's `cuspord` and `cuspORD` are different. The invariant order is the "raw" order; the weighted order multiplies by cusp width. The valence formula uses the weighted order.
- **Assuming cusp order depends on numerator:** For eta quotients on Gamma_0(N), the order at cusp c/d depends only on d. This simplification is critical for efficiency.
- **Skipping Newman condition checks:** Always verify modularity conditions before attempting the valence formula. Skipping this produces meaningless results.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| GCD computation | Custom GCD | rug::Integer::gcd or i64 euclidean | Battle-tested, handles edge cases |
| Euler totient phi(n) | Custom from scratch | Simple formula: n * prod(1 - 1/p) for p | n | Well-known, but implement ourselves (tiny function) |
| TOML parsing | Custom parser | `toml` crate with serde | Standard Rust ecosystem solution |
| Cusp computation | Ad-hoc enumeration | Garvan's cuspmake algorithm (structured) | Proven correct, well-understood |
| Integer factorization | Complex algorithms | Trial division up to sqrt(n) | Sufficient for N < 10000 (typical levels) |

**Key insight:** The mathematical algorithms here are specialized enough that no crate exists for them. We must implement cuspmake, order-at-cusp, and the valence formula ourselves. But we should not re-implement GCD, TOML parsing, or serialization.

## Common Pitfalls

### Pitfall 1: Incorrect Cusp Width Computation
**What goes wrong:** Using the wrong formula for cusp width, leading to incorrect weighted orders.
**Why it happens:** Multiple conventions exist. Garvan uses width = N / gcd(N, c^2) where c is the denominator of the cusp. Other sources define width differently.
**How to avoid:** Follow Garvan's convention exactly. The weighted order (cuspORD) = invariant order (cuspord) * width. Test against known values.
**Warning signs:** Total weighted order across all cusps does not sum to the expected value (should be 0 for a modular function of weight 0).

### Pitfall 2: Forgetting the q-Shift in Eta Quotients
**What goes wrong:** The eta function includes a q^{1/24} factor: eta(delta*tau) = q^{delta/24} * (q^delta; q^delta)_inf. When computing eta quotients, the total q-shift is sum(delta * r_delta) / 24, which must be accounted for.
**Why it happens:** The existing `prodmake` infrastructure works with (1-q^n) products, not with eta functions directly.
**How to avoid:** Track q-shift explicitly in EtaExpression. When converting between ETA and q-series representations, always include the q-shift.
**Warning signs:** Series comparison off by a power of q.

### Pitfall 3: Cusp Enumeration for Gamma_1(N) is More Complex
**What goes wrong:** Using Gamma_0(N) cusp algorithm for Gamma_1(N) or vice versa.
**Why it happens:** Gamma_1(N) has more cusps and different equivalence classes. The thetaids package (generalized eta functions) works on Gamma_1(N), while the ETA package works on Gamma_0(N).
**How to avoid:** Implement both cuspmake (for Gamma_0) and cuspmake1 (for Gamma_1) as separate functions with clearly documented group-theoretic semantics.
**Warning signs:** Number of cusps does not match expected formula.

### Pitfall 4: Newman Condition Integer Overflow
**What goes wrong:** Computing sum(delta * r_delta) or sum((N/delta) * r_delta) mod 24 with large N could overflow i64.
**Why it happens:** Levels can be moderately large (up to a few thousand).
**How to avoid:** Use QRat or i128 for intermediate computations, or reduce mod 24 at each step. For the "perfect square" check on product(delta^|r_delta|), use rug Integer.
**Warning signs:** Modularity check gives wrong answer for large levels.

### Pitfall 5: Generalized Eta Functions vs Standard Eta
**What goes wrong:** Confusing standard Dedekind eta functions with Schoeneberg's generalized eta functions.
**Why it happens:** The thetaids package works with generalized eta functions eta_{delta,g}(tau) which have an additional parameter g. The ETA package works with standard Dedekind eta only.
**How to avoid:** Start with the ETA package (standard Dedekind eta, simpler). Generalized eta support can be added later for thetaids parity. The generalized eta function is: eta_{delta,g}(tau) = q^{delta*P_2(g/delta)/2} * prod_{n>0, n=g mod delta} (1-q^n) * prod_{n>0, n=-g mod delta} (1-q^n), where P_2(x) = {x}^2 - {x} + 1/6 is the second Bernoulli polynomial on the fractional part.
**Warning signs:** JAC-to-eta conversion produces unexpected generalized eta terms.

### Pitfall 6: Identity Database Schema Evolution
**What goes wrong:** TOML schema becomes rigid and cannot accommodate new identity types.
**Why it happens:** Starting with an overly specific schema.
**How to avoid:** Use a flexible schema with `type` discriminators and optional fields. Define clear extension points.
**Warning signs:** Adding a new identity type requires changing every existing entry.

## Code Examples

### Cusp Enumeration for Gamma_0(N)
```rust
/// A cusp represented as a/c with gcd(a,c)=1.
/// Infinity is represented as 1/0.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Cusp {
    pub numer: i64,  // a
    pub denom: i64,  // c (0 for infinity)
}

/// Euler's totient function phi(n).
fn euler_phi(n: i64) -> i64 {
    if n <= 0 { return 0; }
    let mut result = n;
    let mut m = n;
    let mut p = 2i64;
    while p * p <= m {
        if m % p == 0 {
            while m % p == 0 { m /= p; }
            result -= result / p;
        }
        p += 1;
    }
    if m > 1 {
        result -= result / m;
    }
    result
}

/// GCD of two i64 values.
fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Number of cusps of Gamma_0(N).
/// Formula: sum_{d | N} phi(gcd(d, N/d))
fn num_cusps_gamma0(n: i64) -> i64 {
    let mut count = 0;
    let mut d = 1;
    while d * d <= n {
        if n % d == 0 {
            count += euler_phi(gcd(d, n / d));
            if d != n / d {
                count += euler_phi(gcd(n / d, d));
            }
        }
        d += 1;
    }
    count
}

/// Enumerate inequivalent cusps of Gamma_0(N).
/// Based on Garvan's cuspmake algorithm from the ETA package.
fn cuspmake(n: i64) -> Vec<Cusp> {
    let mut cusps = vec![Cusp { numer: 1, denom: 0 }]; // infinity

    // For each divisor c of N, c > 1
    let divs = divisors(n);
    for &c in &divs {
        if c <= 1 { continue; }
        let gc = gcd(c, n / c);
        let mut seen_residues: Vec<i64> = Vec::new();
        for d in 1..c {
            if gcd(d, c) != 1 { continue; }
            let r = d % gc;
            if !seen_residues.contains(&r) {
                seen_residues.push(r);
                cusps.push(Cusp { numer: d, denom: c });
            }
        }
    }
    cusps
}
```

### Order at Cusp (Ligozat Formula)
```rust
/// Compute the order of vanishing of an eta quotient at cusp with denominator d.
///
/// Ligozat formula:
///   ord = (N/24) * sum_{delta|N} gcd(d, delta)^2 * r_delta / (gcd(d, N/d) * d * delta)
///
/// Returns a QRat (may be fractional for intermediate computation, but should
/// be a non-negative integer or half-integer for valid modular forms).
fn eta_order_at_cusp(eta: &EtaExpression, d: i64) -> QRat {
    let n = eta.level;
    let g = gcd(d, n / d);
    let mut sum = QRat::zero();

    for (&delta, &r_delta) in &eta.factors {
        if r_delta == 0 { continue; }
        let gcd_d_delta = gcd(d, delta);
        // Contribution: gcd(d, delta)^2 * r_delta / (gcd(d, N/d) * d * delta)
        let numer = gcd_d_delta * gcd_d_delta * r_delta;
        let denom = g * d * delta;
        sum = sum + QRat::from((numer, denom));
    }

    // Multiply by N/24
    sum * QRat::from((n, 24i64))
}
```

### Newman Modularity Check
```rust
/// Check Newman's conditions for an eta quotient to be modular on Gamma_0(N).
///
/// Returns Ok(()) if all conditions pass, Err with details if any fail.
fn check_newman_conditions(eta: &EtaExpression) -> Result<(), Vec<String>> {
    let n = eta.level;
    let mut errors = Vec::new();

    // Condition 0: All deltas divide N
    for &delta in eta.factors.keys() {
        if n % delta != 0 {
            errors.push(format!("delta={} does not divide N={}", delta, n));
        }
    }

    // Condition 1: sum(delta * r_delta) = 0 (mod 24)
    let sum1: i64 = eta.factors.iter().map(|(&d, &r)| d * r).sum();
    if sum1 % 24 != 0 {
        errors.push(format!("sum(delta*r_delta) = {} not 0 mod 24", sum1));
    }

    // Condition 2: sum((N/delta) * r_delta) = 0 (mod 24)
    let sum2: i64 = eta.factors.iter().map(|(&d, &r)| (n / d) * r).sum();
    if sum2 % 24 != 0 {
        errors.push(format!("sum((N/delta)*r_delta) = {} not 0 mod 24", sum2));
    }

    // Condition 3: product(delta^|r_delta|) is a perfect square
    // (Use rug Integer for this check)

    // Weight check: sum(r_delta) must be even (weight = sum/2 must be integer)
    // For modular functions: weight = 0 => sum(r_delta) = 0
    let weight_sum: i64 = eta.factors.values().sum();
    if weight_sum != 0 {
        errors.push(format!("sum(r_delta) = {} (weight = {}/2, not 0)", weight_sum, weight_sum));
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

### Conversion Between Representations
```rust
/// Convert a JAC expression to an ETA expression.
///
/// JAC(a, b) = (q^a; q^b)_inf * (q^{b-a}; q^b)_inf * (q^b; q^b)_inf
///
/// In eta notation: JAC(a, b) involves generalized eta functions.
/// For the simpler case where JAC factors correspond to standard eta quotients:
///   (q^d; q^d)_inf = eta(d*tau) / q^{d/24}
///
/// A JAC(a,b) where a and b-a are multiples of some d can sometimes be
/// expressed as eta quotients, but in general requires generalized eta functions.
///
/// For Phase 7, we handle the standard Dedekind eta case primarily,
/// and can convert JAC to q-series via the existing jacprod function.
fn jac_to_series(jac: &JacExpression, var: SymbolId, trunc: i64) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::one(var, trunc);
    // Apply q-shift
    // ... then multiply each JAC factor using existing jacprod function
    for factor in &jac.factors {
        let prod = jacprod(factor.a, factor.b, var, trunc);
        let powered = fps_pow(&prod, factor.exponent); // from relations module
        result = arithmetic::mul(&result, &powered);
    }
    result
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hand-verify identities by expanding many terms | Valence formula: finite check suffices | Garvan/Frye 2016-2018 | Automated proving for entire classes |
| Separate tools for eta vs theta | Unified via generalized eta functions | thetaids v1.0 (2016) | JAC/theta identities proved with same engine |
| Manual cusp enumeration | Algorithmic cuspmake/cuspmake1 | Garvan ETA/thetaids packages | Scales to arbitrary level N |

**Deprecated/outdated:**
- None identified. The valence formula method is mathematically timeless. Garvan's packages were last updated May 2023 (thetaids v1.1).

## Key Mathematical Formulas (Reference)

### Number of Cusps
- Gamma_0(N): sum_{d | N} phi(gcd(d, N/d))
- Examples: Gamma_0(1)=1, Gamma_0(2)=2, Gamma_0(6)=4, Gamma_0(12)=6

### Index of Subgroups
- [SL_2(Z) : Gamma_0(N)] = N * prod_{p | N} (1 + 1/p)  (Dedekind psi function)
- [SL_2(Z) : Gamma_1(N)] = N^2 * prod_{p | N} (1 - 1/p^2)

### Sturm Bound
- B(M_k(Gamma_0(N))) = floor(k * [SL_2(Z) : Gamma_0(N)] / 12)
- For weight 0 modular functions with all non-negative cusp orders: check constant term only

### Cusp Width
- Width of cusp c/d on Gamma_0(N) = N / gcd(N, d^2) [Garvan convention]
- (Some sources use different normalization)

### Generalized Eta Function (for thetaids parity, future extension)
- eta_{delta,g}(tau) = q^{delta * P_2(g/delta) / 2} * prod_{n>0, n=g(mod delta)} (1-q^n) * prod_{n>0, n=-g(mod delta)} (1-q^n)
- where P_2(x) = {x}^2 - {x} + 1/6 (second Bernoulli polynomial on fractional part)
- JAC(g, delta, inf) = eta_{delta,g}(tau) in Garvan's notation

### Garvan ETA Package Data Structure
- GP (Generalized Permutation): flat list [t1, r1, t2, r2, ..., tn, rn] where ti are divisors of N and ri are exponents
- gammacheck(GP, N): verifies Newman conditions
- cuspord(GP, cusp): invariant order at cusp
- cuspORD(GP, N, cusp): weighted order = cuspord * width
- cuspORDS(GP, CUSPS, N): orders at all cusps (should sum to 0)

## Test Cases for Verification

### Known Cusp Counts
| N | Cusps of Gamma_0(N) | Count |
|---|---------------------|-------|
| 1 | {inf} | 1 |
| 2 | {inf, 0} | 2 |
| 4 | {inf, 0, 1/2} | 3 |
| 6 | {inf, 0, 1/2, 1/3} | 4 |
| 12 | {inf, 0, 1/2, 1/3, 1/4, 1/6} | 6 |

### Known Eta Quotient Identities
1. **Euler's identity:** eta(tau)^1 = q^{1/24} * prod(1-q^n)
   - Level 1, weight 1/2 -- not a modular function (weight != 0)

2. **Ramanujan's partition congruence p(5n+4) = 0 (mod 5):**
   - Related to eta(5tau)^5 / eta(tau)

3. **Rogers-Ramanujan type identities:**
   - G(q) and H(q) identities provable via thetaids

4. **Simple eta identity:** eta(tau)^24 = Delta(tau) (the modular discriminant)
   - Level 1, weight 12

### Identity Database Test Cases
- Load a TOML file with 5+ identities
- Search by tag "euler" returns matching entries
- Search by function "eta" returns all eta-related identities

## Open Questions

1. **Generalized eta functions scope**
   - What we know: thetaids works with generalized eta_{delta,g} which are needed for JAC(a,b) where a/b is not a simple divisor ratio
   - What's unclear: How much generalized eta support is needed for Phase 7 vs deferring to a future enhancement
   - Recommendation: Implement standard Dedekind eta pipeline first (ETA package parity). Add generalized eta support as a stretch goal or separate sub-plan. The JAC representation can convert to q-series via the existing jacprod function for verification purposes.

2. **Identity database size and sourcing**
   - What we know: Garvan's packages prove hundreds of identities. The DB should be TOML-based and searchable.
   - What's unclear: How many identities to seed initially, and what citation format to use.
   - Recommendation: Start with 10-20 well-known identities (Euler pentagonal, Jacobi triple product, Rogers-Ramanujan, Ramanujan's partition congruences, basic eta-quotient identities). Source citations from Garvan's papers and Gasper-Rahman.

3. **Python API surface area**
   - What we know: Need to expose identity proving and database lookup to Python.
   - What's unclear: Exact API design for the proving pipeline (how verbose should proof certificates be?).
   - Recommendation: Return a structured ProofResult that Python can inspect. Include cusp details for debugging/display.

## Existing Infrastructure to Leverage

The following Phase 4 infrastructure is directly usable:

| Existing Component | Location | How Phase 7 Uses It |
|-------------------|----------|---------------------|
| `prodmake` + `InfiniteProductForm` | `qseries/prodmake.rs` | Convert q-series to product form for analysis |
| `etamake` + `EtaQuotient` | `qseries/prodmake.rs` | Convert q-series to eta-quotient form |
| `jacprodmake` + `JacobiProductForm` | `qseries/prodmake.rs` | Convert q-series to JAC form |
| `mobius`, `divisors` | `qseries/prodmake.rs` (private) | Need to make public or reimplement for cusps |
| `rational_null_space` | `qseries/linalg.rs` | Linear algebra for identity relations |
| `etaq`, `jacprod` | `qseries/products.rs` | Expand eta/JAC to FPS for verification |
| `FormalPowerSeries` | `series/mod.rs` | q-expansion computation and comparison |
| `QRat`, `QInt` | `number.rs` | Exact arithmetic throughout |

**Important:** The `mobius` and `divisors` functions in `prodmake.rs` are currently `fn` (private). Phase 7 needs them for cusp computation. Either make them `pub(crate)` or re-implement in the identity module (they are small).

## Sources

### Primary (HIGH confidence)
- Garvan ETA package source code (`wprog-ETA-06-12-2020-HOMEPC.txt`) -- cuspmake algorithm, cuspord formula, gammacheck conditions, provemodfuncGAMMA0id algorithm
- Ligozat formula from multiple academic papers (Rouse-Webb, Lemke Oliver) -- order of vanishing formula
- Newman's theorem conditions -- verified across multiple sources (Wikipedia Dedekind eta, Grokipedia, arxiv papers)

### Secondary (MEDIUM confidence)
- [Frye & Garvan, "Automatic Proof of Theta-Function Identities" (arXiv:1807.08051)](https://arxiv.org/abs/1807.08051) -- thetaids methodology, generalized eta functions, cuspmake1
- [Garvan, "A tutorial for the MAPLE ETA package" (arXiv:1907.09130)](https://arxiv.org/abs/1907.09130) -- ETA package tutorial
- [LMFDB Sturm bound definition](https://www.lmfdb.org/knowledge/show/cmf.sturm_bound) -- Sturm bound formula
- [SageMath Gamma_0/Gamma_1 documentation](https://doc.sagemath.org/html/en/reference/arithgroup/sage/modular/arithgroup/congroup_gamma0.html) -- cusp equivalence criteria
- [Garvan qseries.org thetaids page](https://qseries.org/fgarvan/qmaple/thetaids/) -- package overview and version info
- [arXiv:2407.05748 (Eta quotient expressions)](https://arxiv.org/html/2407.05748v1) -- Ligozat formula, Newman conditions

### Tertiary (LOW confidence)
- Generalized eta function definition from Schoeneberg: exact formula P_2(g/delta) extracted from search results, needs validation against Garvan's actual implementation
- Cusp counts for Gamma_1(N): numerical values from SageMath docs, formula not explicitly extracted

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- serde/toml are obvious choices, no alternatives needed
- Architecture (JAC/ETA models): HIGH -- Garvan's data structures are well-documented in source code
- Architecture (cuspmake): HIGH -- Algorithm extracted directly from Garvan's ETA source
- Architecture (order at cusp / Ligozat): HIGH -- Formula verified across 4+ independent sources
- Architecture (valence formula / proving): MEDIUM-HIGH -- General approach clear from multiple papers; exact Garvan implementation details partially extracted from source
- Architecture (generalized eta / thetaids): MEDIUM -- Definition found but complex; recommend deferring full implementation
- Identity database TOML schema: MEDIUM -- No existing standard; design based on requirements
- Pitfalls: HIGH -- Well-documented in modular forms literature

**Research date:** 2026-02-14
**Valid until:** 2026-06-14 (stable mathematical domain, algorithms do not change)
