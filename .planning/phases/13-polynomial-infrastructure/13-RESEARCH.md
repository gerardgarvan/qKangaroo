# Phase 13: Polynomial Infrastructure - Research

**Researched:** 2026-02-15
**Domain:** Dense univariate polynomial arithmetic over QRat, rational function type, subresultant GCD
**Confidence:** HIGH

## Summary

Phase 13 builds the polynomial and rational function infrastructure that q-Gosper (Phase 14) and q-Zeilberger (Phase 15) require. The existing codebase has no polynomial type -- `FormalPowerSeries` has truncation semantics incompatible with exact polynomial arithmetic, and `QMonomial` is a single term, not a polynomial. A new `QRatPoly` type (dense `Vec<QRat>`, ascending degree) and `QRatRationalFunc` type (numerator/denominator pair in lowest terms) must be built from scratch.

The critical algorithmic component is polynomial GCD via the subresultant PRS (Polynomial Remainder Sequence) algorithm, which prevents the coefficient explosion that plagues naive Euclidean GCD over Q[x]. Content extraction (factoring out the GCD of all coefficients) before GCD operations is mandatory -- Paule and Riese documented that this optimization alone reduced q-Zeilberger runtime from 95% to 30-40%. The resultant computation falls out naturally as a byproduct of the subresultant chain.

**Primary recommendation:** Build a focused ~700-900 line `poly` module under `crates/qsym-core/src/poly/` with `QRatPoly` (dense, ascending coefficients) and `QRatRationalFunc` (auto-simplifying on construction). Use subresultant PRS for GCD from day one. Do NOT use naive Euclidean GCD. Do NOT reuse `FormalPowerSeries`.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rug | 1.28 (existing) | QRat coefficients (GMP-backed arbitrary precision) | Already the coefficient ring for the entire engine. No adapter needed. |
| No new dependency | N/A | Polynomial arithmetic built in-house | All external Rust polynomial crates are either AGPL-licensed, floating-point only, or heavyweight with no rug interop. See milestone research for full analysis. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| linalg.rs (existing) | internal | Solving polynomial equations by undetermined coefficients | When q-Gosper/q-Zeilberger need to solve key equations (Phase 14+) |
| proptest | 1 (existing dev-dep) | Property-based testing of polynomial ring axioms | Test polynomial arithmetic commutativity, associativity, GCD correctness |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| In-house QRatPoly | feanor-math (MIT) | feanor docs say "operations with polynomials over infinite rings are currently very slow"; has own bigint, no rug interop, heavyweight trait system |
| In-house QRatPoly | polynomial-ring (AGPL) | Excellent API but AGPL license is incompatible with project MIT license |
| In-house QRatPoly | algebraics (LGPL) | Has subresultant GCD but LGPL complicates static linking in Rust; marginal benefit for ~700 lines of focused code |
| Dense Vec | Sparse BTreeMap | Gosper/Zeilberger polynomials are low-degree (5-30) and typically dense-filled. Vec is simpler, faster, and avoids BTreeMap allocation overhead for this size range. |

**Installation:**
```bash
# No new dependencies needed
```

## Architecture Patterns

### Recommended Project Structure

```
crates/qsym-core/src/
  poly/
    mod.rs              # QRatPoly struct, constructors, Display, basic queries
    arithmetic.rs       # Add, Sub, Mul, Neg, scalar_mul, div_rem
    gcd.rs              # content, primitive_part, subresultant_gcd, resultant
    ratfunc.rs          # QRatRationalFunc with auto-simplification
  lib.rs                # Add: pub mod poly
```

This follows the existing pattern of `series/` (mod.rs, arithmetic.rs, display.rs, generator.rs) and `qseries/identity/` (7 submodules). The `poly/` module is a peer of `series/` and `qseries/`, not nested under `qseries/`, because polynomial arithmetic is a general mathematical primitive used by multiple downstream modules.

### Pattern 1: Dense Polynomial with Trailing-Zero Stripping

**What:** `QRatPoly` stores coefficients as `Vec<QRat>` in ascending degree order. The last element is always nonzero (the leading coefficient). An empty Vec represents the zero polynomial. Every mutating operation must strip trailing zeros.

**When to use:** All polynomial operations. This is the canonical representation.

**Example:**
```rust
/// Dense univariate polynomial over QRat.
///
/// p(x) = coeffs[0] + coeffs[1]*x + coeffs[2]*x^2 + ... + coeffs[d]*x^d
///
/// Invariant: coeffs is empty (zero poly) OR coeffs.last() is nonzero.
/// This means degree = coeffs.len() - 1 for nonzero polynomials.
#[derive(Clone, Debug)]
pub struct QRatPoly {
    coeffs: Vec<QRat>,
}

impl QRatPoly {
    /// Strip trailing zero coefficients to maintain invariant.
    fn normalize(&mut self) {
        while self.coeffs.last().map_or(false, |c| c.is_zero()) {
            self.coeffs.pop();
        }
    }

    pub fn degree(&self) -> Option<usize> {
        if self.coeffs.is_empty() {
            None  // Zero polynomial has no degree
        } else {
            Some(self.coeffs.len() - 1)
        }
    }

    pub fn leading_coeff(&self) -> Option<QRat> {
        self.coeffs.last().cloned()
    }

    pub fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }
}
```

### Pattern 2: Subresultant PRS for GCD (NOT Euclidean)

**What:** Polynomial GCD uses the subresultant PRS algorithm, which controls intermediate coefficient growth to at most polynomial (vs exponential for naive Euclidean). The algorithm tracks scaling factors `psi` and `beta` that allow exact integer division at each step, preventing coefficient explosion.

**When to use:** Always for GCD computation. Never use naive Euclidean GCD over Q[x].

**Algorithm (from Brown 1978, verified against SymPy euclidtools.py):**

```rust
/// Compute GCD of two polynomials using subresultant PRS.
///
/// Algorithm: Brown's subresultant PRS (1978).
///
/// For polynomials over Q[x], an alternative is:
///   1. Clear denominators to get integer polynomials
///   2. Extract content (GCD of integer coefficients)
///   3. Run subresultant PRS on primitive parts
///   4. Make result monic
///
/// The subresultant PRS avoids the coefficient explosion of
/// naive Euclidean GCD by tracking scaling factors.
pub fn poly_gcd(a: &QRatPoly, b: &QRatPoly) -> QRatPoly {
    // Handle trivial cases
    if a.is_zero() { return b.make_monic(); }
    if b.is_zero() { return a.make_monic(); }

    // Ensure deg(f) >= deg(g)
    let (f, g) = if a.degree() >= b.degree() {
        (a.clone(), b.clone())
    } else {
        (b.clone(), a.clone())
    };

    // Extract content and work with primitive parts
    let cont_f = f.content();
    let cont_g = g.content();
    let cont_gcd = qrat_gcd(&cont_f, &cont_g);

    let f = f.primitive_part();
    let g = g.primitive_part();

    // Subresultant PRS loop
    let mut f = f;
    let mut g = g;
    let mut h;

    let d = f.degree().unwrap() - g.degree().unwrap(); // delta
    let lc_g = g.leading_coeff().unwrap();

    // Initial scaling: psi = -1, beta = (-1)^(d+1)
    let mut psi = QRat::from((-1i64, 1i64));
    let neg_one = QRat::from((-1i64, 1i64));

    loop {
        // h = pseudo_remainder(f, g)
        h = f.pseudo_rem(&g);

        if h.is_zero() {
            // g is the GCD (up to content)
            let result = g.primitive_part();
            return result.scalar_mul(&cont_gcd).make_monic();
        }

        let deg_f = f.degree().unwrap();
        let deg_g = g.degree().unwrap();
        let delta = deg_f - deg_g;

        // Compute beta = -lc(f) * psi^delta
        let lc_f = f.leading_coeff().unwrap();
        let neg_lc_f = -lc_f.clone();
        let psi_delta = qrat_pow(&psi, delta as u32);
        let beta = neg_lc_f * psi_delta;

        // h = h / beta (exact division)
        h = h.scalar_div(&beta);

        // Update psi: psi = (-lc_f)^delta / psi^(delta-1)
        if delta > 1 {
            let neg_lc_f_delta = qrat_pow(&neg_lc_f, delta as u32);
            let psi_delta_minus_1 = qrat_pow(&psi, (delta - 1) as u32);
            psi = neg_lc_f_delta / psi_delta_minus_1;
        } else {
            // delta == 1: psi = -lc_f
            psi = -lc_f;
        }

        // Shift: f <- g, g <- h
        f = g;
        g = h;
    }
}
```

### Pattern 3: Rational Function with Auto-Simplification on Construction

**What:** `QRatRationalFunc` always stores numerator/denominator in lowest terms with a monic (leading coefficient = 1) denominator. Every constructor and arithmetic operation enforces this invariant.

**When to use:** All rational function operations.

```rust
/// Rational function p(x)/q(x) in lowest terms.
///
/// Invariants:
///   1. gcd(numer, denom) = 1
///   2. denom is monic (leading_coeff = 1)
///   3. denom is nonzero
pub struct QRatRationalFunc {
    pub numer: QRatPoly,
    pub denom: QRatPoly,
}

impl QRatRationalFunc {
    /// Construct from numerator and denominator, auto-reducing to lowest terms.
    pub fn new(numer: QRatPoly, denom: QRatPoly) -> Self {
        assert!(!denom.is_zero(), "Rational function denominator cannot be zero");

        let g = poly_gcd(&numer, &denom);
        let mut n = numer.exact_div(&g);
        let mut d = denom.exact_div(&g);

        // Make denominator monic
        let lc = d.leading_coeff().unwrap();
        if lc != QRat::one() {
            n = n.scalar_div(&lc);
            d = d.scalar_div(&lc);
        }

        Self { numer: n, denom: d }
    }
}
```

### Pattern 4: q-Shift as O(n) Coefficient Scaling

**What:** The q-shift operation `p(x) -> p(q_val * x)` is a simple coefficient-wise scaling: if `p(x) = sum c_i * x^i`, then `p(q_val * x) = sum c_i * q_val^i * x^i`. This is O(degree), not O(degree^2).

**When to use:** All q-Gosper and q-Zeilberger computations use this as a core primitive.

```rust
impl QRatPoly {
    /// Apply q-shift: p(x) -> p(q_val * x).
    ///
    /// Each coefficient c_i is multiplied by q_val^i.
    /// This is O(degree) -- no polynomial multiplication needed.
    pub fn q_shift(&self, q_val: &QRat) -> QRatPoly {
        if self.is_zero() || q_val == &QRat::one() {
            return self.clone();
        }
        let mut result = Vec::with_capacity(self.coeffs.len());
        let mut q_power = QRat::one();
        for c in &self.coeffs {
            result.push(c.clone() * q_power.clone());
            q_power = q_power * q_val.clone();
        }
        QRatPoly::from_vec(result)
    }

    /// Apply q-shift j times: p(x) -> p(q^j * x).
    pub fn q_shift_n(&self, q_val: &QRat, j: i64) -> QRatPoly {
        if j == 0 { return self.clone(); }
        let q_j = qrat_pow_signed(q_val, j);
        self.q_shift(&q_j)
    }
}
```

### Anti-Patterns to Avoid

- **Reusing FormalPowerSeries for polynomial arithmetic:** FPS has truncation semantics (`truncation_order`), a `variable: SymbolId` field, and truncated multiplication. Polynomial GCD, resultant, and exact division require exact arithmetic without truncation. The conceptual mismatch would cause subtle bugs. Build QRatPoly independently.

- **Naive Euclidean GCD over Q[x]:** Each step can square the bit-size of coefficients. A degree-20 GCD can produce intermediates with thousands of digits. Subresultant PRS keeps coefficient growth polynomial, not exponential.

- **Skipping content extraction:** Content extraction (factoring out GCD of all coefficients) before GCD is the single most impactful optimization. Without it, the Paule-Riese team found 95% of q-Zeilberger runtime was spent in coefficient management.

- **Making QRatPoly generic over coefficient ring:** YAGNI. We only need Q[x]. A generic `Polynomial<R: Ring>` adds trait complexity with no benefit for this project. If needed later, generalize then.

- **Sparse BTreeMap representation:** For degree 5-30 polynomials arising in Gosper/Zeilberger, dense Vec is simpler and faster. The existing FPS uses BTreeMap because power series can have thousands of terms with gaps. Polynomials in this domain are low-degree and typically dense.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Arbitrary-precision rational arithmetic | Custom bigrat | QRat (rug/GMP) | Already the project's coefficient type. GMP is the gold standard. |
| Linear system solving | Custom Gaussian elimination | `rational_null_space` in linalg.rs | Already implemented, tested, and used by 12+ relation discovery functions. |
| Polynomial evaluation | Naive sum of powers | Horner's method | O(n) multiplications vs O(n^2). Horner is the standard and trivial to implement. |
| Hash for QRatPoly | Custom hash function | Derive or manual from coeffs | Follow QRat's pattern (hash canonical byte representation). |

**Key insight:** The polynomial infrastructure is the ONLY new code needed. Everything else (coefficient arithmetic, linear algebra, hash-consing patterns) already exists in the codebase.

## Common Pitfalls

### Pitfall 1: Coefficient Explosion in Intermediate GCD Steps

**What goes wrong:** Naive Euclidean GCD over Q[x] can produce intermediate polynomials with coefficient bit-sizes that grow exponentially. A degree-d GCD computation can produce intermediates with O(2^d) digit counts in numerators/denominators.

**Why it happens:** Each pseudo-division step multiplies all coefficients by the leading coefficient of the divisor, raised to a power. Without proper scaling (subresultant technique), these factors accumulate exponentially.

**How to avoid:** Use subresultant PRS algorithm, which tracks scaling factors (psi, beta) that allow exact integer division at each step, controlling growth to polynomial. Additionally, extract content (GCD of all coefficients) from both polynomials before GCD computation.

**Warning signs:** QRat denominators exceeding 1000 digits in intermediate steps. Single GCD call taking >1 second. Polynomial multiplication producing coefficients 10x larger than inputs.

### Pitfall 2: Trailing Zero Invariant Violations

**What goes wrong:** After arithmetic operations (especially subtraction), the result polynomial may have trailing zero coefficients. If these are not stripped, `degree()` returns wrong values, `leading_coeff()` returns zero, and all downstream algorithms (GCD, division) break silently.

**Why it happens:** Subtraction of polynomials with equal leading terms produces a polynomial whose highest-degree coefficient cancels to zero. Without normalization, the Vec still has that zero entry.

**How to avoid:** Call `normalize()` (strip trailing zeros) after EVERY arithmetic operation that can produce cancellation: `sub`, `add`, `div_rem`, `pseudo_rem`, `scalar_mul` (if scalar is zero). Make `normalize()` private and call it in all constructors and mutating operations.

**Warning signs:** `degree()` returning `Some(d)` but `coeff(d)` returning zero. GCD returning non-monic results. Division quotients with wrong degrees.

### Pitfall 3: Division by Zero in Pseudo-Remainder / Exact Division

**What goes wrong:** `pseudo_rem` multiplies the dividend by `lc(divisor)^(deg_diff + 1)` before division. If the divisor is zero or constant with special properties, the multiplication can overflow or the division can fail. `exact_div` (used after GCD to divide out common factors) panics if the division is not exact.

**Why it happens:** Edge cases: zero divisor, constant divisor, divisor with higher degree than dividend.

**How to avoid:** Guard all division functions with degree/zero checks. `exact_div` should assert remainder is zero with a descriptive panic message. `div_rem` should return (zero, dividend) when divisor degree exceeds dividend degree.

**Warning signs:** Panics in polynomial division. Non-zero remainders from `exact_div`. Incorrect quotient degrees.

### Pitfall 4: Content of Zero Polynomial

**What goes wrong:** The "content" of a polynomial is the GCD of its coefficients. For the zero polynomial, content is undefined (or conventionally zero). Calling `content()` on a zero polynomial and then trying to divide by it causes a panic.

**Why it happens:** GCD and primitive_part functions assume nonzero input.

**How to avoid:** Define `content(zero) = QRat::zero()` and `primitive_part(zero) = zero`. Guard the division in `primitive_part` against zero content.

### Pitfall 5: Monic Normalization for Rational Function Denominator

**What goes wrong:** When constructing `QRatRationalFunc`, the denominator must be made monic (leading coefficient 1). If you forget to scale the numerator by the same factor, the rational function's value changes.

**Why it happens:** Making the denominator monic requires dividing both numerator and denominator by the denominator's leading coefficient.

**How to avoid:** Always scale both numer and denom together. Encapsulate this in the `QRatRationalFunc::new()` constructor so it cannot be bypassed.

## Code Examples

Verified patterns from the existing codebase and algorithm references.

### Constructors

```rust
// Follows the pattern of QRat::from((num, den)) constructors
impl QRatPoly {
    pub fn zero() -> Self {
        Self { coeffs: vec![] }
    }

    pub fn one() -> Self {
        Self { coeffs: vec![QRat::one()] }
    }

    pub fn constant(c: QRat) -> Self {
        if c.is_zero() { Self::zero() } else { Self { coeffs: vec![c] } }
    }

    /// The indeterminate x: 0 + 1*x
    pub fn x() -> Self {
        Self { coeffs: vec![QRat::zero(), QRat::one()] }
    }

    /// Monomial c * x^d
    pub fn monomial(c: QRat, deg: usize) -> Self {
        if c.is_zero() { return Self::zero(); }
        let mut coeffs = vec![QRat::zero(); deg + 1];
        coeffs[deg] = c;
        Self { coeffs }
    }

    /// From coefficient vector (ascending degree). Strips trailing zeros.
    pub fn from_vec(mut coeffs: Vec<QRat>) -> Self {
        // Strip trailing zeros
        while coeffs.last().map_or(false, |c| c.is_zero()) {
            coeffs.pop();
        }
        Self { coeffs }
    }

    /// Linear polynomial a + b*x
    pub fn linear(a: QRat, b: QRat) -> Self {
        Self::from_vec(vec![a, b])
    }
}
```

### Polynomial Division (div_rem)

```rust
/// Euclidean division: self = quotient * divisor + remainder
/// with deg(remainder) < deg(divisor).
///
/// Returns (quotient, remainder).
/// Panics if divisor is zero.
pub fn div_rem(&self, divisor: &QRatPoly) -> (QRatPoly, QRatPoly) {
    assert!(!divisor.is_zero(), "Polynomial division by zero");

    if self.degree() < divisor.degree() {
        return (QRatPoly::zero(), self.clone());
    }

    let mut remainder = self.coeffs.clone();
    let div_deg = divisor.degree().unwrap();
    let div_lc = divisor.leading_coeff().unwrap();
    let self_deg = self.degree().unwrap();

    let quot_len = self_deg - div_deg + 1;
    let mut quotient = vec![QRat::zero(); quot_len];

    for i in (0..quot_len).rev() {
        let idx = i + div_deg;
        if remainder[idx].is_zero() { continue; }

        let q_coeff = remainder[idx].clone() / div_lc.clone();
        quotient[i] = q_coeff.clone();

        for j in 0..=div_deg {
            let sub = q_coeff.clone() * divisor.coeffs[j].clone();
            let pos = i + j;
            remainder[pos] = remainder[pos].clone() - sub;
        }
    }

    (QRatPoly::from_vec(quotient), QRatPoly::from_vec(remainder))
}
```

### Pseudo-Remainder (for subresultant PRS)

```rust
/// Pseudo-remainder: lc(b)^(deg(a) - deg(b) + 1) * a mod b.
///
/// Returns a polynomial whose coefficients are integers (no fractions introduced)
/// when the inputs have integer coefficients.
pub fn pseudo_rem(&self, other: &QRatPoly) -> QRatPoly {
    assert!(!other.is_zero(), "Pseudo-remainder with zero divisor");

    let deg_a = match self.degree() { Some(d) => d, None => return QRatPoly::zero() };
    let deg_b = match other.degree() { Some(d) => d, None => panic!("zero divisor") };

    if deg_a < deg_b {
        return self.clone();
    }

    let delta = deg_a - deg_b + 1;
    let lc_b = other.leading_coeff().unwrap();
    let scale = qrat_pow(&lc_b, delta as u32);

    // scaled_self = lc(b)^delta * self
    let scaled = self.scalar_mul(&scale);

    // Now divide scaled by other -- remainder is the pseudo-remainder
    let (_, rem) = scaled.div_rem(other);
    rem
}
```

### Content and Primitive Part

```rust
impl QRatPoly {
    /// Content: the GCD of all coefficients (as a positive QRat).
    /// For a polynomial over Q, this means: gcd(all numerators) / lcm(all denominators).
    pub fn content(&self) -> QRat {
        if self.is_zero() { return QRat::zero(); }

        // For QRat coefficients: first clear denominators,
        // then GCD of resulting integer numerators,
        // divided by LCM of original denominators.
        let mut numer_gcd = rug::Integer::from(0);
        let mut denom_lcm = rug::Integer::from(1);

        for c in &self.coeffs {
            if !c.is_zero() {
                numer_gcd = numer_gcd.gcd(c.numer());
                denom_lcm = denom_lcm.lcm(c.denom());
            }
        }

        if numer_gcd == 0 { return QRat::zero(); }

        QRat::from(rug::Rational::from((numer_gcd, denom_lcm)))
    }

    /// Primitive part: self / content(self).
    pub fn primitive_part(&self) -> QRatPoly {
        let c = self.content();
        if c.is_zero() { return QRatPoly::zero(); }
        self.scalar_div(&c)
    }

    /// Make monic: divide by leading coefficient.
    pub fn make_monic(&self) -> QRatPoly {
        match self.leading_coeff() {
            None => QRatPoly::zero(),
            Some(lc) => self.scalar_div(&lc),
        }
    }
}
```

### Horner Evaluation

```rust
impl QRatPoly {
    /// Evaluate at a rational point using Horner's method.
    /// p(x) = c_0 + x*(c_1 + x*(c_2 + ... + x*c_d))
    pub fn eval(&self, x: &QRat) -> QRat {
        if self.is_zero() { return QRat::zero(); }

        let mut result = QRat::zero();
        for c in self.coeffs.iter().rev() {
            result = result * x.clone() + c.clone();
        }
        result
    }
}
```

### Rational Function Arithmetic

```rust
impl QRatRationalFunc {
    /// Add: a/b + c/d = (a*d + b*c) / (b*d), reduced
    pub fn add(&self, other: &QRatRationalFunc) -> QRatRationalFunc {
        let numer = &self.numer * &other.denom + &self.denom * &other.numer;
        let denom = &self.denom * &other.denom;
        QRatRationalFunc::new(numer, denom) // auto-reduces
    }

    /// Multiply: (a/b) * (c/d) = (a*c) / (b*d), reduced
    pub fn mul(&self, other: &QRatRationalFunc) -> QRatRationalFunc {
        // Cross-cancel before multiplying to keep sizes small
        let g1 = poly_gcd(&self.numer, &other.denom);
        let g2 = poly_gcd(&other.numer, &self.denom);

        let n1 = self.numer.exact_div(&g1);
        let d2 = other.denom.exact_div(&g1);
        let n2 = other.numer.exact_div(&g2);
        let d1 = self.denom.exact_div(&g2);

        let numer = &n1 * &n2;
        let denom = &d1 * &d2;
        QRatRationalFunc::new(numer, denom)
    }
}
```

### Display (LaTeX-compatible)

```rust
// Follows the existing DLMF 17.2 notation pattern from hypergeometric.rs
impl fmt::Display for QRatPoly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() { return write!(f, "0"); }
        let mut first = true;
        for (i, c) in self.coeffs.iter().enumerate() {
            if c.is_zero() { continue; }
            if !first && !is_negative(c) { write!(f, " + ")?; }
            else if !first { write!(f, " - ")?; }
            // ... format coefficient * x^i
        }
        Ok(())
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Euclidean GCD over Q[x] | Subresultant PRS | 1970s (Brown/Traub 1978) | Avoids exponential coefficient growth |
| Sylvester matrix for resultant | Extract from subresultant chain | 1970s | O(d^2) vs O(d^3) and shares code with GCD |
| Dense polynomial * all sizes | Dense for small, sparse for large | Ongoing | For degree <50 (our case), dense is faster |
| Generic ring polynomial types | Domain-specific QRatPoly | Project decision | Eliminates trait overhead, simplifies code |

**Deprecated/outdated:**
- Naive Euclidean GCD over Q[x]: causes coefficient explosion. Never use for exact rational arithmetic.
- Sylvester matrix determinant for resultant: wasteful when you already need GCD (subresultant gives both).

## Detailed Design Decisions

### 1. QRatPoly API Surface

The complete API needed for Phase 13 (and anticipated by Phases 14-15):

**Construction:**
- `zero()`, `one()`, `constant(c)`, `x()`, `monomial(c, deg)`, `linear(a, b)`, `from_vec(coeffs)`, `from_coeffs(&[(i64, QRat)])` (sparse input)

**Queries:**
- `degree() -> Option<usize>`, `leading_coeff() -> Option<QRat>`, `coeff(i) -> QRat`, `is_zero()`, `is_constant()`, `is_one()`

**Arithmetic:**
- `Add, Sub, Mul, Neg` traits (by value and by reference)
- `scalar_mul(&QRat)`, `scalar_div(&QRat)` (panics on zero)
- `div_rem(&QRatPoly) -> (QRatPoly, QRatPoly)` -- Euclidean division
- `exact_div(&QRatPoly) -> QRatPoly` -- panics if remainder nonzero
- `pseudo_rem(&QRatPoly) -> QRatPoly` -- for subresultant PRS

**GCD/Resultant:**
- `poly_gcd(a, b) -> QRatPoly` -- subresultant PRS, returns monic GCD
- `poly_resultant(a, b) -> QRat` -- via subresultant chain
- `content() -> QRat`, `primitive_part() -> QRatPoly`, `make_monic() -> QRatPoly`

**Evaluation/Shift:**
- `eval(&QRat) -> QRat` -- Horner's method
- `q_shift(&QRat) -> QRatPoly` -- p(x) -> p(q*x)
- `q_shift_n(&QRat, i64) -> QRatPoly` -- p(x) -> p(q^j * x)

**Comparison/Display:**
- `PartialEq, Eq` (coefficient-wise)
- `Display` (human-readable format)

### 2. QRatRationalFunc API Surface

**Construction:**
- `new(numer, denom)` -- auto-reduces to lowest terms, makes denom monic
- `from_poly(p)` -- creates p/1
- `from_qrat(c)` -- creates c/1 (constant rational function)

**Arithmetic:**
- `Add, Sub, Mul, Div, Neg` traits
- Cross-cancel optimization in multiplication

**Queries:**
- `is_zero()`, `is_polynomial()`, `eval(&QRat) -> Option<QRat>` (None if denom vanishes)

**q-Shift:**
- `q_shift(&QRat) -> QRatRationalFunc` -- shift both numer and denom

### 3. q-Shift Semantics

For this phase, `q` in `q_shift` is a **concrete QRat value**, not a formal variable. This means:

- `p(x).q_shift(&q_val)` computes `p(q_val * x)` by scaling coefficient `c_i` to `c_i * q_val^i`.
- `p(x).q_shift_n(&q_val, j)` computes `p(q_val^j * x)`.
- The caller is responsible for choosing what `q_val` represents. In the concrete-n approach for q-Zeilberger (Phase 15), `q_val` is a specific rational number substituted for `q`.

For Phase 14 (q-Gosper), the q-shift is the key operation in the coprime splitting step: checking `gcd(a(x), b(q^j * x))` for various `j`. The `q_val` is the same concrete rational value used throughout the computation.

### 4. Subresultant PRS Algorithm Details

The algorithm maintains three sequences: the polynomial remainder sequence `R_i`, the scaling factors `psi_i`, and the degree differences `delta_i`.

**Initialization:**
- `R_0 = f` (higher degree), `R_1 = g` (lower degree)
- `delta_0 = deg(f) - deg(g)`
- `psi_0 = -1` (as QRat)
- `beta_0 = (-1)^(delta_0 + 1)`

**Loop (i = 1, 2, ...):**
1. `h = prem(R_{i-1}, R_i)` -- pseudo-remainder
2. If `h = 0`, then `R_i` is the GCD (up to content)
3. Divide: `R_{i+1} = h / beta_i` (exact division in the coefficient ring)
4. Update delta: `delta_i = deg(R_{i-1}) - deg(R_i)`
5. Update psi: if `delta > 1`: `psi = (-lc(R_{i-1}))^delta / psi^(delta-1)`; if `delta = 1`: `psi = -lc(R_{i-1})`
6. Update beta: `beta = -lc(R_{i-1}) * psi^delta`

**Resultant extraction:** The resultant is related to the final subresultant. When the last nonzero remainder has degree 0 (is a constant), the resultant can be computed from the accumulated scaling factors. Specifically, if the subresultant chain terminates at a constant `c`, the resultant is `c` (up to sign corrections based on the degrees).

### 5. Resultant via Subresultant Chain

The polynomial resultant `Res(a, b)` can be computed as a byproduct of the subresultant PRS:

```rust
/// Compute resultant of two polynomials.
///
/// The resultant is zero iff the polynomials share a common root.
/// Computed via the subresultant chain (same computation as GCD).
pub fn poly_resultant(a: &QRatPoly, b: &QRatPoly) -> QRat {
    // Run subresultant PRS, tracking the accumulated scalar
    // The resultant = last subresultant of degree 0
    // (with appropriate sign correction)
    // ...
}
```

The resultant is used in Phase 14 for q-dispersion computation: finding all positive integers `j` such that `gcd(a(x), b(q^j * x))` is nontrivial.

### 6. Where to Put the Module

The `poly/` module goes at the **crate root level** (`crates/qsym-core/src/poly/`), as a peer of `series/`, `qseries/`, `simplify/`, etc. This is because:

1. Polynomial arithmetic is a general mathematical primitive, not specific to q-series
2. Both `qseries/gosper.rs` (Phase 14) and `qseries/zeilberger.rs` (Phase 15) will depend on it
3. It follows the existing pattern where `series/` provides FPS and `qseries/` uses it

Add to `lib.rs`:
```rust
pub mod poly;
```

### 7. Integration with Existing Types

| Existing Type | Integration | How |
|--------------|-------------|-----|
| `QRat` | QRatPoly coefficients | Direct use, no adapter |
| `QInt` | Degree bounds, exponents | Used for integer degree comparisons |
| `QMonomial` | Bridge to q-Gosper | `QMonomial.coeff` becomes a QRatPoly coefficient; `QMonomial.power` tracks q-power offset (Phase 14) |
| `FormalPowerSeries` | Verification only | Convert QRatPoly to FPS for numerical checking (Phase 14+) |
| `rational_null_space` | Solving polynomial equations | Used by q-Gosper key equation solver (Phase 14) |

## Test Strategy

### Ring Axiom Tests

```rust
// Property-based tests with proptest
// Commutativity: a + b == b + a, a * b == b * a
// Associativity: (a + b) + c == a + (b + c)
// Distributivity: a * (b + c) == a*b + a*c
// Identity: a + 0 == a, a * 1 == a
// Inverse: a + (-a) == 0
```

### GCD Correctness Tests

| Test Case | a(x) | b(x) | Expected GCD |
|-----------|-------|-------|--------------|
| Coprime linears | (x - 1) | (x - 2) | 1 |
| Common factor | (x-1)(x-2) | (x-1)(x-3) | (x-1) |
| One divides other | (x-1)^2 | (x-1) | (x-1) |
| Same polynomial | (x-1)(x+1) | (x-1)(x+1) | (x-1)(x+1) |
| With zero | 0 | p(x) | p(x) (monic) |
| Large coefficients | p with 100-digit QRat coeffs | q similarly | Verify no explosion |
| Content extraction | 6x^2 + 4x + 2 | content = 2 | primitive_part = 3x^2 + 2x + 1 |

### Resultant Tests

| Test Case | a(x) | b(x) | Expected Resultant |
|-----------|-------|-------|-------------------|
| Common root | (x-1)(x-2) | (x-1)(x-3) | 0 (share root x=1) |
| No common root | (x-1) | (x-2) | nonzero |
| Classical: x^2-1, x-1 | x^2-1 | x-1 | 0 |

### q-Shift Tests

| Test Case | p(x) | q_val | Expected |
|-----------|-------|-------|----------|
| Identity shift | x^2 + x + 1 | 1 | x^2 + x + 1 |
| Simple shift | x^2 + x + 1 | 2 | 4x^2 + 2x + 1 |
| Double shift | p | q | p.q_shift(q).q_shift(q) == p.q_shift_n(q, 2) |
| Evaluation identity | p.q_shift(q).eval(x) | any | p.eval(q*x) |

### Rational Function Tests

| Test Case | Construction | Expected |
|-----------|-------------|----------|
| Auto-reduce | (x^2-1)/(x-1) | (x+1)/1 |
| Addition | 1/x + 1/(x+1) | (2x+1)/(x(x+1)) |
| Monic denom | (2x)/(2x+2) | x/(x+1) |

### Division Tests

| Test Case | Dividend | Divisor | Quotient | Remainder |
|-----------|----------|---------|----------|-----------|
| Exact | x^2-1 | x-1 | x+1 | 0 |
| With remainder | x^2 | x-1 | x+1 | 1 |
| Higher degree divisor | x | x^2+1 | 0 | x |
| Constant divisor | 6x^2+4 | 2 | 3x^2+2 | 0 |

## Open Questions

1. **PartialEq for QRatRationalFunc**
   - What we know: Two rational functions are equal iff they agree as functions, which for lowest-terms representation means numer1 == numer2 AND denom1 == denom2.
   - What's unclear: Should we implement `Eq` and `Hash` for QRatRationalFunc? It would be useful for caching but requires careful normalization.
   - Recommendation: Implement `PartialEq` and `Eq` via coefficient comparison of normalized forms. Defer `Hash` unless needed.

2. **Display format for QRatPoly**
   - What we know: Need human-readable output. Options: "3x^2 + 2x + 1" vs "3*x^2 + 2*x + 1" vs LaTeX.
   - What's unclear: Should we support multiple display formats (plain vs LaTeX)?
   - Recommendation: Implement `Display` for plain text ("3x^2 + 2x + 1") initially. Add LaTeX rendering in Phase 14+ if needed for proof output.

3. **Should poly_gcd always return monic?**
   - What we know: Over Q[x], the GCD is unique up to scalar multiples. Convention is to return monic (leading coeff = 1).
   - What's unclear: Whether making it monic introduces unnecessary division by the leading coefficient.
   - Recommendation: Yes, always return monic. The leading coefficient division is O(degree) and eliminates ambiguity. This matches the convention used by all reference implementations (SymPy, Maple, Mathematica).

## Sources

### Primary (HIGH confidence)
- Existing codebase: `number.rs` (QRat/QInt), `linalg.rs` (rational_null_space), `series/mod.rs` (FormalPowerSeries), `qseries/mod.rs` (QMonomial), `qseries/factoring.rs` (qfactor polynomial division pattern)
- [SymPy euclidtools.py](https://github.com/sympy/sympy/blob/master/sympy/polys/euclidtools.py) - Reference implementation of subresultant PRS (`dmp_inner_subresultants`)
- [SymPy subresultants_qq_zz.py](https://github.com/sympy/sympy/blob/master/sympy/polys/subresultants_qq_zz.py) - Modified subresultant PRS over Q and Z
- [Brown, "The Subresultant PRS Algorithm" (1978)](https://iiif.library.cmu.edu/file/Traub_box00027_fld00059_bdl0001_doc0001/Traub_box00027_fld00059_bdl0001_doc0001.pdf) - Original subresultant PRS paper
- [Paule & Riese, qZeil (1997)](https://www3.risc.jku.at/publications/download/risc_117/Paule_Riese.pdf) - Content extraction optimization: 95% -> 30-40% runtime
- Milestone research: `.planning/research/STACK.md`, `ARCHITECTURE.md`, `PITFALLS.md`, `FEATURES.md`

### Secondary (MEDIUM confidence)
- [Wikipedia: Polynomial greatest common divisor](https://en.wikipedia.org/wiki/Polynomial_greatest_common_divisor) - Overview of PRS variants and coefficient growth
- [Geddes/Czapor/Labahn, "Algorithms for Computer Algebra"](https://link.springer.com/book/10.1007/b102438) - Textbook treatment of subresultant PRS
- [AFP Subresultants formalization](https://www.isa-afp.org/browser_info/current/AFP/Subresultants/document.pdf) - Isabelle/HOL formalization confirming algorithm correctness

### Tertiary (LOW confidence)
- Various web search results on polynomial GCD algorithms and rational function normalization (used for cross-verification only)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - No new dependencies; QRat already exists and is well-tested with 578+ Rust tests
- Architecture: HIGH - Dense Vec representation is standard for low-degree polynomials; subresultant PRS is the textbook algorithm
- Pitfalls: HIGH - Coefficient explosion is the #1 documented issue in all q-Zeilberger implementations; trailing zero bugs are universal in polynomial implementations
- Code examples: MEDIUM-HIGH - Based on algorithm descriptions and SymPy reference, not tested in Rust yet

**Research date:** 2026-02-15
**Valid until:** 2026-06-15 (stable algorithms, no dependency drift risk)
