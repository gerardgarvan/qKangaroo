# Architecture Patterns: Algorithmic q-Hypergeometric Identity Proving

**Domain:** Algorithmic summation and identity proving for q-hypergeometric series
**Researched:** 2026-02-15
**Confidence:** HIGH (algorithms well-established in literature; integration points verified against codebase)

## Executive Summary

This document describes the architecture for integrating q-Gosper, q-Zeilberger (creative telescoping), and WZ certificate algorithms into the existing q-Kangaroo symbolic computation engine. The key architectural challenge is that these algorithms operate on **rational functions of q^k** (univariate polynomials in an indeterminate x = q^k, with rational function arithmetic), which is fundamentally different from the existing FormalPowerSeries (truncated power series in q). New data structures for univariate polynomial rings over QRat and rational function fields are required, along with a bridge layer that converts between the existing HypergeometricSeries representation and the algorithm input format.

## Recommended Architecture

### System Overview

```
                    USER INPUT
                        |
                        v
          +--------------------------+
          | HypergeometricSeries     |  (existing: upper/lower QMonomial arrays)
          +--------------------------+
                        |
                        v
          +--------------------------+
          | Term Ratio Extraction    |  (NEW: compute t_{k+1}/t_k as RatFunc)
          | (bridge layer)           |
          +--------------------------+
                        |
                  +-----+-----+
                  |           |
                  v           v
   +----------------+   +-------------------+
   | q-Gosper       |   | q-Zeilberger      |
   | (indefinite    |   | (creative         |
   |  summation)    |   |  telescoping)     |
   +-------+--------+   +--------+----------+
           |                      |
           v                      v
   +----------------+   +-------------------+
   | Antidifference  |   | Recurrence +      |
   | (RatFunc cert.) |   | WZ Certificate   |
   +-------+--------+   +--------+----------+
           |                      |
           v                      v
   +--------------------------------------+
   | ProofResult / RecurrenceResult       |  (NEW result types)
   +--------------------------------------+
           |
           v
   +--------------------------------------+
   | FPS Verification                     |  (existing: eval_phi + compare)
   +--------------------------------------+
```

### Core New Data Structures

These are the fundamental types needed. They do NOT exist in the current codebase and must be built from scratch.

#### 1. QPolynomial: Univariate Polynomial over QRat

```rust
/// Univariate polynomial over QRat.
///
/// Represents p(x) = sum_i c_i * x^i where x = q^k.
/// Stored as a dense or sparse coefficient vector.
/// These are NOT power series -- they are finite-degree polynomials
/// with exact rational coefficients.
///
/// Operations needed: add, sub, mul, div_rem (Euclidean division),
/// gcd, resultant, evaluate, shift (p(x) -> p(q*x) for q-shift),
/// degree, leading_coefficient, content, primitive_part.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QPolynomial {
    /// Coefficients indexed by degree: coeffs[i] = coefficient of x^i.
    /// Trailing zeros stripped. Empty = zero polynomial.
    coeffs: Vec<QRat>,
}
```

**Why build our own instead of using a crate:** The `polynomial-ring` crate (AGPL-3.0) is license-incompatible. `symbolica` is powerful but heavyweight and has its own agenda. The operations needed (GCD, resultant, q-shift) are well-defined algorithms over QRat, and QRat already wraps rug::Rational with arbitrary precision. Building a focused ~500-line module is cleaner than adding a large dependency for 5 operations.

**Key operations with their algorithmic roles:**

| Operation | Algorithm | Used By |
|-----------|-----------|---------|
| `mul` | Schoolbook or Karatsuba | All algorithms |
| `div_rem` | Euclidean polynomial division | GCD, factoring |
| `gcd` | Euclidean GCD over Q[x] | Gosper splitting, coprimality |
| `resultant` | Subresultant or Sylvester matrix | Dispersion computation |
| `q_shift(q_val)` | p(x) -> p(q_val * x) | q-Gosper/q-Zeilberger core |
| `evaluate(x)` | Horner's method | Verification |
| `content` / `primitive_part` | GCD of coefficients | Normalization |

#### 2. QRatFunc: Rational Function over QRat

```rust
/// Rational function over QRat: p(x)/q(x) in lowest terms.
///
/// Invariant: gcd(numer, denom) = 1, denom is monic (leading coeff = 1).
/// This represents the term ratio t_{k+1}/t_k of a q-hypergeometric summand
/// as a rational function of x = q^k.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QRatFunc {
    /// Numerator polynomial.
    pub numer: QPolynomial,
    /// Denominator polynomial (monic, nonzero).
    pub denom: QPolynomial,
}
```

**Operations needed:** add, sub, mul, div, q-shift (apply q-shift to both numer and denom), partial fraction decomposition.

#### 3. QRecurrence: Linear Recurrence with Polynomial Coefficients

```rust
/// A linear recurrence relation with polynomial coefficients.
///
/// Represents: sum_{i=0}^{J} c_i(n) * S(n+i) = 0
/// where c_i are polynomials in n (for classical case) or
/// c_i(q^n) are polynomials in q^n (for q-case).
///
/// Produced by q-Zeilberger as the "telescoper."
#[derive(Clone, Debug)]
pub struct QRecurrence {
    /// Coefficients c_0(x), c_1(x), ..., c_J(x) where x = q^n.
    /// The order J = coefficients.len() - 1.
    pub coefficients: Vec<QPolynomial>,
}
```

#### 4. GosperDecomposition: The p, a, b Triple

```rust
/// Result of Gosper's polynomial splitting step.
///
/// Given the term ratio r(x) = t_{k+1}/t_k written as P(x)/Q(x),
/// the splitting produces polynomials a(x), b(x), c(x) such that:
///   P(x)/Q(x) = [a(x) / b(x)] * [c(q*x) / c(x)]
/// with the coprimality condition:
///   gcd(a(x), b(q^j * x)) = 1 for all j >= 1.
///
/// In the q-case, the shift is multiplicative: x -> q*x (not additive x -> x+1).
#[derive(Clone, Debug)]
pub struct GosperDecomposition {
    /// Polynomial a(x) in the numerator.
    pub a: QPolynomial,
    /// Polynomial b(x) in the denominator.
    pub b: QPolynomial,
    /// Polynomial c(x) (the "fudge factor").
    pub c: QPolynomial,
}
```

#### 5. WZCertificate: The Proof Certificate

```rust
/// A WZ (Wilf-Zeilberger) proof certificate.
///
/// For an identity sum_k F(n,k) = rhs(n), the certificate R(n,k)
/// is a rational function such that G(n,k) = R(n,k) * F(n,k) satisfies:
///   F(n+1,k) - F(n,k) = G(n,k+1) - G(n,k)
///
/// Summing over k telescopes the RHS to zero, proving the identity.
/// The certificate is a bivariate rational function in (q^n, q^k).
///
/// For the q-case, R is a rational function of (q^n, q^k),
/// represented as a rational function in q^k whose coefficients
/// are rational functions of q^n.
#[derive(Clone, Debug)]
pub struct WZCertificate {
    /// The rational function R(n,k) as a function of x=q^k,
    /// with coefficients that are rational functions of y=q^n.
    ///
    /// For simplicity in the initial implementation, we store
    /// R as a pair of bivariate polynomials (numer, denom) in (q^n, q^k).
    /// A more refined representation can come later.
    pub certificate: QRatFunc,
    /// The recurrence order (how many n-shifts appear).
    pub order: usize,
}
```

### Module Structure

```
crates/qsym-core/src/
  qseries/
    polynomial.rs        (NEW: QPolynomial, QRatFunc)
    gosper.rs             (NEW: q-Gosper algorithm)
    zeilberger.rs         (NEW: q-Zeilberger / creative telescoping)
    wz.rs                 (NEW: WZ certificate computation + verification)
    hypergeometric.rs     (EXISTING: add bridge methods)
    mod.rs                (MODIFY: add pub mod + pub use)
```

### Component Boundaries and Data Flow

| Component | Responsibility | Inputs | Outputs |
|-----------|---------------|--------|---------|
| `polynomial.rs` | Polynomial ring Q[x], rational function field Q(x), GCD, resultant, q-shift | QRat coefficients | QPolynomial, QRatFunc |
| `gosper.rs` | q-Gosper indefinite summation: splitting step + polynomial equation solving | QRatFunc (term ratio) | `Option<QRatFunc>` (antidifference ratio) or None |
| `zeilberger.rs` | Creative telescoping: iterate q-Gosper over trial recurrence orders | HypergeometricSeries, order bound | QRecurrence + QRatFunc certificate |
| `wz.rs` | WZ pair construction, certificate verification, identity proof packaging | QRecurrence + certificate | WZCertificate, ProofResult |
| Bridge on `hypergeometric.rs` | Convert HypergeometricSeries to term ratio QRatFunc | HypergeometricSeries | QRatFunc |

## Detailed Algorithm Architecture

### Algorithm 1: q-Gosper (Indefinite Summation)

**Problem:** Given a q-hypergeometric term t_k, find a q-hypergeometric term s_k such that t_k = s_{k+1} - s_k (the q-antidifference).

**Input:** The term ratio r(x) = t_{k+1}/t_k as a QRatFunc where x = q^k.

**Steps:**

1. **Ratio computation** (bridge layer):
   For r phi_s(a_1,...,a_r; b_1,...,b_s; q, z), the general term is:
   ```
   t_k = [(a_1;q)_k * ... * (a_r;q)_k] / [(q;q)_k * (b_1;q)_k * ... * (b_s;q)_k]
         * [(-1)^k * q^{k(k-1)/2}]^{1+s-r} * z^k
   ```
   The ratio t_{k+1}/t_k is:
   ```
   r(x) = [(1-a_1*x)(1-a_2*x)...(1-a_r*x)] / [(1-q*x)(1-b_1*x)...(1-b_s*x)]
          * [(-1) * x]^{1+s-r} * z
   ```
   where x = q^k. Each factor (1 - a_i * q^k) becomes a linear polynomial (1 - a_i * x) in x.

   **This is the critical bridge:** HypergeometricSeries parameters map directly to linear factors in x = q^k.

2. **q-Coprime splitting** (Gosper decomposition):
   Write r(x) = P(x)/Q(x) with P, Q coprime polynomials.
   Find a(x), b(x), c(x) such that:
   - P(x)/Q(x) = [a(x)/b(x)] * [c(q*x)/c(x)]
   - gcd(a(x), b(q^j * x)) = 1 for all integers j >= 1

   **q-Dispersion computation:** The dispersion of polynomials P(x) and Q(x) under q-shift is the set of positive integers j such that gcd(P(x), Q(q^j * x)) is nontrivial. Computed via:
   - For each root alpha of P and root beta of Q, check if alpha = beta * q^j for some positive integer j.
   - Equivalently: compute resultant R(y) = Res_x(P(x), Q(y*x)) and find its roots that are powers of q.
   - For polynomials with rational coefficients in the q-setting, this reduces to checking if q^j * beta/alpha = 1, i.e., if alpha/beta is a power of q.

   The splitting iteratively removes common q-shifted factors:
   ```
   For j in dispersion_set (descending):
       g = gcd(a(x), b(q^j * x))
       a = a / g
       b = b / g(q^{-j} * x)   // back-shift
       c = c * prod_{i=1}^{j} g(q^{-i} * x)
   ```

3. **Key equation** (polynomial equation solving):
   Find polynomial f(x) satisfying:
   ```
   a(x) * f(q*x) - b(x/q) * f(x) = c(x)
   ```
   (q-analogue of Gosper's key equation a(n)*f(n+1) - b(n-1)*f(n) = c(n))

   Degree bound: deg(f) <= max(deg(a), deg(b)) when deg(a) != deg(b), otherwise determined by leading coefficient comparison.

   This is a **linear system**: expand f(x) = f_0 + f_1*x + ... + f_d*x^d, substitute into the equation, and match coefficients. The resulting system of linear equations over QRat can be solved by the existing RREF in `linalg.rs`.

4. **Result:** If f exists, the antidifference ratio is s_k/t_k = (b(x/q) * f(x)) / c(x), a rational function of q^k.

**Output:** `Option<QRatFunc>` -- Some(ratio) if summable, None if not Gosper-summable.

### Algorithm 2: q-Zeilberger (Creative Telescoping)

**Problem:** Given a terminating q-hypergeometric sum S(n) = sum_{k=0}^{N} F(n,k), find a linear recurrence for S(n) with polynomial coefficients.

**Input:** HypergeometricSeries whose parameters depend on n (one upper parameter is q^{-n}).

**Core idea:** Find a "telescoper" L and a "certificate" G such that:
```
L * F(n,k) = G(n, k+1) - G(n, k)
```
where L = c_0(q^n) + c_1(q^n) * E_n + ... + c_J(q^n) * E_n^J is a linear difference operator in n, and E_n is the shift operator E_n[f(n)] = f(n+1).

Summing both sides over k, the RHS telescopes to boundary terms (which vanish for terminating sums), giving:
```
c_0(q^n) * S(n) + c_1(q^n) * S(n+1) + ... + c_J(q^n) * S(n+J) = 0
```

**Steps:**

1. **For J = 1, 2, 3, ...** (increasing recurrence order):

2. **Construct the modified summand:**
   ```
   H_J(n, k) = sum_{i=0}^{J} c_i * F(n+i, k)
   ```
   where c_i are unknown polynomial coefficients.

   The ratio H_J(n, k+1) / H_J(n, k) is a rational function of q^k (since F is q-hypergeometric in k), BUT its coefficients depend on the unknown c_i.

3. **Apply q-Gosper to H_J:**
   The key insight is that H_J is q-hypergeometric in k if and only if the c_i satisfy certain polynomial constraints. The q-Gosper algorithm applied to H_J either succeeds (finding the certificate) or fails.

   In practice: compute the term ratio of F(n,k) in k, then form the combined ratio for the ansatz sum_{i=0}^{J} c_i * F(n+i,k). Apply Gosper's splitting and key equation. The c_i appear as parameters in the key equation's linear system.

4. **Solve the extended system:**
   The key equation produces a system of linear equations in both the c_i (recurrence coefficients) and the f_j (polynomial antidifference coefficients). If this system has a nontrivial solution, we have found a recurrence of order J.

5. **Extract recurrence + certificate:**
   The c_i give QRecurrence. The Gosper solution f(x) gives the WZ certificate.

**Output:** `(QRecurrence, QRatFunc)` -- the recurrence and the rational certificate.

**Architectural note:** q-Zeilberger calls q-Gosper as a subroutine. This is the fundamental dependency that determines build order.

### Algorithm 3: WZ Certificate Verification

**Problem:** Given a conjectured identity and a WZ certificate R(n,k), verify the identity.

**Steps:**

1. **Normalize:** Given sum_k F(n,k) = rhs(n), set F_norm(n,k) = F(n,k)/rhs(n).

2. **Construct G:** G(n,k) = R(n,k) * F_norm(n,k).

3. **Verify the WZ equation:**
   ```
   F_norm(n+1,k) - F_norm(n,k) = G(n,k+1) - G(n,k)
   ```
   This can be verified either:
   - **Symbolically:** Substitute and simplify (requires rational function arithmetic).
   - **Numerically via FPS:** Expand both sides as FPS in q to sufficient order and compare coefficients. This leverages existing eval_phi infrastructure.

4. **Check boundary conditions:** Verify that the telescoping sum converges (boundary terms vanish).

**Architectural choice:** Use FPS verification (existing infrastructure) for robustness. Symbolic verification is a stretch goal.

### Bridge: HypergeometricSeries to Term Ratio

This is the crucial integration point. The existing `HypergeometricSeries` stores parameters as `Vec<QMonomial>`, and each QMonomial is `(coeff: QRat, power: i64)` representing `coeff * q^power`.

The bridge conversion:

```rust
/// Convert a HypergeometricSeries into the term ratio t_{k+1}/t_k
/// as a rational function of x = q^k.
///
/// For _r phi_s (a_1,...,a_r; b_1,...,b_s; q, z):
///   ratio(x) = prod_i (1 - a_i.coeff * q^{a_i.power} * x)
///            / [(1 - q*x) * prod_j (1 - b_j.coeff * q^{b_j.power} * x)]
///            * [(-1) * x]^{1+s-r} * z.coeff * q^{z.power}
///
/// Each QMonomial a_i = c_i * q^{m_i} produces a linear factor (1 - c_i * q^{m_i} * x).
/// In the polynomial representation, the coefficient of x in this factor is -c_i * q^{m_i}.
///
/// IMPORTANT: The q^{m_i} factor means this is NOT purely a polynomial in x.
/// We have two options:
///   (a) Work in Q(q)[x] -- polynomials in x with coefficients in Q(q).
///   (b) Specialize: for terminating series with q^{-n} parameter,
///       set q to a concrete value or keep q symbolic.
///
/// For the q-Zeilberger algorithm, we need option (a): the coefficients
/// of the polynomial in x = q^k contain q^{m_i} terms, but these are
/// CONSTANTS (not depending on k). So the polynomial ring is Q[x]
/// where the QRat coefficients absorb the q^{power} factors as numerical values.
///
/// HOWEVER: when parameters depend on n (like q^{-n}), we need the
/// parameter to remain symbolic. This means we actually need polynomials
/// in TWO variables: x = q^k and y = q^n, or equivalently,
/// rational functions in x with coefficients that are rational functions of y.
pub fn term_ratio(series: &HypergeometricSeries) -> QRatFunc {
    // Build numerator: prod_i (1 - a_i * x)
    // Build denominator: (1 - q*x) * prod_j (1 - b_j * x)
    // Multiply by extra factor and z
    // ...
}
```

**The two-variable problem:** For q-Zeilberger, the summand F(n,k) depends on both n and k. The term ratio in k is a rational function of x = q^k, but its coefficients depend on q^n. For terminating series where one parameter is q^{-n}, we face a design choice:

1. **Concrete n approach:** Specialize to specific integer values of n, run q-Gosper for each, and find the recurrence by interpolation. Simpler but limited.

2. **Symbolic n approach:** Work in Q(y)[x] -- polynomials in x = q^k whose coefficients are rational functions of y = q^n. More general but requires two-level polynomial/rational function arithmetic.

**Recommendation:** Start with the **concrete n approach** for the first implementation. For each trial recurrence order J, specialize n to enough integer values (J+1 values suffice) to determine the recurrence coefficients. This avoids the complexity of bivariate polynomial arithmetic while still being algorithmically correct for all terminating q-hypergeometric sums. The symbolic approach can be added later as an optimization.

## Patterns to Follow

### Pattern 1: Result Types with Proof Certificates

**What:** Every algorithm produces a result type that includes both the answer and enough data to verify it independently.

**When:** All proving algorithms.

**Rationale:** Matches the existing pattern in `prove.rs` (ProofResult with cusp_orders, sturm_bound) and `hypergeometric.rs` (SummationResult, TransformationResult).

```rust
/// Result of attempting q-Gosper indefinite summation.
pub enum GosperResult {
    /// Found antidifference: t_k = s_{k+1} - s_k.
    /// The certificate is the ratio s_k/t_k as a rational function of q^k.
    Summable {
        /// The antidifference ratio s_k/t_k.
        certificate: QRatFunc,
        /// Gosper decomposition (a, b, c) used.
        decomposition: GosperDecomposition,
        /// The polynomial f(x) from the key equation.
        key_polynomial: QPolynomial,
    },
    /// No q-hypergeometric antidifference exists.
    NotSummable,
}

/// Result of q-Zeilberger creative telescoping.
pub enum ZeilbergerResult {
    /// Found recurrence of the given order.
    Recurrence {
        /// The recurrence relation: sum_i c_i(q^n) * S(n+i) = 0.
        recurrence: QRecurrence,
        /// The rational certificate for the telescoping.
        certificate: QRatFunc,
        /// Recurrence order J (= recurrence.coefficients.len() - 1).
        order: usize,
    },
    /// No recurrence found up to the given order bound.
    NotFound {
        max_order_tried: usize,
    },
}

/// Result of WZ proof verification.
pub enum WZProofResult {
    /// Identity proved via WZ method.
    Proved {
        /// The WZ certificate R(n,k).
        certificate: WZCertificate,
        /// The recurrence satisfied by the sum.
        recurrence: QRecurrence,
        /// Number of FPS terms verified.
        verification_terms: i64,
    },
    /// Certificate verification failed at a specific point.
    Failed {
        reason: String,
    },
}
```

### Pattern 2: FPS Verification as Safety Net

**What:** After any algebraic computation, verify the result by expanding both sides as FormalPowerSeries and comparing coefficients.

**When:** After q-Gosper finds an antidifference, after q-Zeilberger finds a recurrence.

**Rationale:** The existing codebase uses this pattern extensively (verify_transformation, prove_by_expansion). It catches implementation bugs in the algebraic algorithms without requiring a second implementation.

```rust
/// Verify a Gosper result by FPS expansion.
///
/// Checks that s_{k+1} - s_k = t_k for enough terms.
fn verify_gosper_result(
    series: &HypergeometricSeries,
    result: &GosperResult,
    variable: SymbolId,
    truncation_order: i64,
) -> bool {
    // Expand the original series and the reconstructed antidifference
    // Compare coefficients
}
```

### Pattern 3: Layered Algorithm Composition

**What:** q-Zeilberger calls q-Gosper as a subroutine. WZ calls q-Zeilberger. Each layer is independently testable.

**When:** Always -- this is the fundamental algorithm dependency structure.

**Rationale:** Classical algorithm design from Petkovsek-Wilf-Zeilberger "A=B":
- q-Gosper: indefinite summation (base case)
- q-Zeilberger: definite summation = q-Gosper + creative telescoping
- WZ: proof = q-Zeilberger + certificate extraction + verification

```
QPolynomial (ring operations)
    |
    v
QRatFunc (field operations)
    |
    v
q-Gosper (uses QPolynomial GCD, resultant, linear algebra)
    |
    v
q-Zeilberger (calls q-Gosper as subroutine)
    |
    v
WZ Certificate (calls q-Zeilberger, verifies via FPS)
```

### Pattern 4: q-Shift as First-Class Operation

**What:** The q-shift operation x -> q*x on polynomials is the q-analogue of the classical shift n -> n+1. It must be a fundamental, efficient operation on QPolynomial.

**When:** All q-Gosper and q-Zeilberger computations.

```rust
impl QPolynomial {
    /// Apply q-shift: p(x) -> p(q_val * x).
    ///
    /// If p(x) = sum_i c_i x^i, then p(q_val * x) = sum_i c_i * q_val^i * x^i.
    /// This is O(n) where n = degree.
    pub fn q_shift(&self, q_val: &QRat) -> QPolynomial {
        let mut result = self.clone();
        let mut q_power = QRat::one();
        for i in 0..result.coeffs.len() {
            result.coeffs[i] = &result.coeffs[i] * &q_power;
            q_power = &q_power * q_val;
        }
        result
    }

    /// Apply q-shift j times: p(x) -> p(q^j * x).
    pub fn q_shift_n(&self, q_val: &QRat, j: i64) -> QPolynomial {
        if j == 0 { return self.clone(); }
        let q_j = qrat_pow(q_val, j); // q^j
        self.q_shift(&q_j)
    }
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Bivariate Polynomials Too Early

**What:** Building a full bivariate polynomial ring Q[x,y] (x=q^k, y=q^n) before the univariate case works.

**Why bad:** Bivariate polynomial GCD is significantly more complex (sparse interpolation, modular algorithms). The concrete-n approach for q-Zeilberger sidesteps this entirely and is sufficient for all terminating q-hypergeometric identities.

**Instead:** Build QPolynomial (univariate) first. Use concrete-n specialization for q-Zeilberger. Add bivariate support later only if needed for non-terminating or multi-sum cases.

### Anti-Pattern 2: Reusing FormalPowerSeries for Polynomial Operations

**What:** Trying to use the existing FormalPowerSeries type (BTreeMap<i64, QRat> with truncation) for the polynomial ring operations needed by Gosper/Zeilberger.

**Why bad:** FPS has truncation semantics (operations discard terms beyond truncation_order). Polynomial GCD, resultant, and exact division require exact polynomial arithmetic without any truncation. The conceptual mismatch would cause subtle bugs.

**Instead:** Build a dedicated QPolynomial type with exact arithmetic. Use FPS only for verification (expanding results to check correctness).

### Anti-Pattern 3: Symbolic q in Polynomials

**What:** Keeping q as a symbolic indeterminate in the polynomial coefficients, making the coefficient ring Q(q) instead of Q.

**Why bad:** For concrete q-series computations, q is the formal variable of the power series (it has no specific numerical value, but it is "the" variable). The q^{power} in QMonomial parameters are concrete rational multiples when we fix q. For Gosper's algorithm, we need the ratio to be a rational function of x = q^k with NUMERICAL coefficients. The powers q^{m_i} from QMonomial parameters become part of those numerical coefficients.

**However:** The q^{m_i} factors from QMonomial.power are actually powers of the formal variable q. When we write the term ratio as a polynomial in x = q^k, the coefficient of x in the factor (1 - c_i * q^{m_i} * x) includes the "number" c_i * q^{m_i}. But q^{m_i} is not a rational number -- it is a formal power of q.

**Resolution:** This is the key design tension. For the **concrete-n approach**, we can:
- Fix n to a specific integer, so q^{-n} becomes q^{-5} (say).
- All parameters are then QMonomials with specific powers.
- But they are still formal powers of q, not rationals.

The correct approach: work in the polynomial ring Q[x] where the coefficients encode the q-power information through **tagging**. Each coefficient is a QMonomial (rational * q^power), not just a QRat. So the polynomial ring is really over QMonomial arithmetic, or equivalently, Laurent polynomials in q with rational coefficients.

**Revised recommendation:** QPolynomial coefficients should be QMonomial (not QRat), so that q-power information flows through polynomial arithmetic correctly. The GCD and resultant algorithms work the same way -- they just operate over the ring of QMonomials. Since QMonomial has mul, div, is_zero, and equality, it satisfies ring axioms.

Wait -- QMonomial is NOT a ring. Addition of QMonomials with different powers yields a two-term expression, not another QMonomial. This means QMonomial coefficients would break polynomial addition.

**Final resolution:** The coefficients must be FormalPowerSeries (or at least, sparse Laurent polynomials in q). But this brings back truncation issues.

**Actually, the clean solution:** For terminating q-hypergeometric series with specific integer parameters, evaluate the term ratio as an explicit rational function by substituting q^k -> x and treating all q^{m} as abstract generators. In practice, for the concrete-n approach:

1. Fix n to a specific integer.
2. The HypergeometricSeries parameters become concrete QMonomials.
3. Compute the term ratio as a product of factors (1 - a_i * q^k) where a_i are now concrete QMonomials.
4. Evaluate the sum directly (which we already can do with eval_phi) for several values of n.
5. Use the existing findlincombo / RREF to determine the recurrence coefficients.

This is the "Sister Celine / Fasenmyer" approach and avoids the need for polynomial ring arithmetic entirely for the first implementation.

**For the FULL q-Gosper/q-Zeilberger implementation:** The polynomial coefficients should live in Q[q, q^{-1}] -- Laurent polynomials in q with rational coefficients. But for our FPS-based system, this is equivalent to FormalPowerSeries with an extended domain allowing negative powers. We can represent this as BTreeMap<i64, QRat> without truncation (exact polynomial, not truncated series).

### Anti-Pattern 4: Monolithic Algorithm Implementation

**What:** Implementing q-Gosper, q-Zeilberger, and WZ all at once in a single module.

**Why bad:** Hard to test, hard to debug. The algorithms have clear dependency ordering.

**Instead:** Build bottom-up: polynomial.rs first, test it thoroughly. Then gosper.rs using polynomial.rs. Then zeilberger.rs using gosper.rs. Then wz.rs using everything.

## Refined Data Structure Design

After working through the anti-patterns, here is the refined recommendation:

### QExactPoly: Exact Polynomial (No Truncation)

```rust
/// Exact sparse polynomial: sum_i c_i * x^i, no truncation.
///
/// Unlike FormalPowerSeries, this has NO truncation_order.
/// All coefficients are exact. Used for Gosper/Zeilberger
/// polynomial arithmetic where truncation would be incorrect.
///
/// Stored sparsely (BTreeMap) since many polynomials in the
/// algorithms are products of linear factors and thus sparse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QExactPoly {
    /// Maps degree -> coefficient. Missing = 0. No zero entries stored.
    coeffs: BTreeMap<i64, QRat>,
}
```

This is essentially a FormalPowerSeries without the truncation_order field and without the variable field. By stripping those two fields, we get exact polynomial semantics. The operations (add, sub, mul, div_rem, gcd, resultant) are standard polynomial ring operations.

**Why sparse:** The polynomials arising in q-Gosper are typically products of linear factors (1 - alpha_i * x), so they have degree r or s but many zero interior coefficients in the monomial basis. BTreeMap gives O(k log k) multiplication where k = number of nonzero terms.

**Why not reuse FPS:** FPS has truncation semantics baked into every operation (mul checks ka + kb < trunc). Even if we set truncation_order to i64::MAX, the semantics are wrong for division with remainder, GCD, and resultant.

### QExactRatFunc: Rational Function (Exact)

```rust
/// Rational function p(x)/q(x) in lowest terms.
///
/// Invariant: gcd(numer, denom) = 1. denom leading coefficient is 1.
#[derive(Clone, Debug)]
pub struct QExactRatFunc {
    pub numer: QExactPoly,
    pub denom: QExactPoly,
}
```

### Dealing with q-Powers in Coefficients

For the bridge from HypergeometricSeries to QExactPoly, each QMonomial parameter a_i = c_i * q^{m_i} produces a linear factor:

```
(1 - a_i * q^k) = (1 - c_i * q^{m_i + k})
```

As a polynomial in x = q^k, this is (1 - c_i * q^{m_i} * x). The coefficient of x is -c_i * q^{m_i}.

**For the concrete-n approach (recommended first):** We do not need this polynomial representation at all. We simply:
1. For each trial n value, evaluate sum_k F(n,k) using eval_phi.
2. Use RREF to find recurrence coefficients from the evaluated sums.

**For the full algebraic approach (later):** The coefficient -c_i * q^{m_i} can be encoded as a QRat IF we assign a specific "numerical" value to q. But q is formal. The resolution is to track q-powers separately:

```rust
/// A coefficient in the q-shifted polynomial ring.
/// Represents c * q^m where c is rational and m is an integer.
/// This is exactly QMonomial.
///
/// For polynomial arithmetic:
/// - Addition: (c1*q^m1) + (c2*q^m2) is NOT a single QCoeff unless m1==m2.
///   This means QExactPoly coefficients need to be QRat, not QCoeff.
///
/// Resolution: Introduce the polynomial ring Q[q][x] where polynomials
/// in x have coefficients that are polynomials in q. But for our use case,
/// each coefficient in x involves at most one q-power (since it comes from
/// a single linear factor). So we can use QExactPoly with QRat coefficients
/// IF we normalize by factoring out q-powers.
```

**Practical approach:** Factor the term ratio into:
```
ratio(x) = (q-power scalar) * (polynomial in x with QRat coefficients)
```
where the q-power scalar is a QMonomial. The polynomial part has QRat coefficients obtained by dividing out the common q-power. This works because each linear factor (1 - c*q^m * x) has a QRat "constant" coefficient (the 1) and a QRat-times-q^m coefficient of x.

Actually, let me reconsider. The term ratio for _2phi_1(a, b; c; q, z) is:
```
ratio = [(1 - a*q^k)(1 - b*q^k)] / [(1 - q^{k+1})(1 - c*q^k)] * z
```

Setting x = q^k:
```
ratio(x) = [(1 - a*x)(1 - b*x)] / [(1 - q*x)(1 - c*x)] * z
```

If a, b, c are QMonomials (say a = a_c * q^{a_p}), then (1 - a*x) = (1 - a_c * q^{a_p} * x).

For polynomial operations, the coefficient of x in this factor is `-a_c * q^{a_p}`. This involves q^{a_p}, which is not a rational number.

**Two correct approaches:**

(A) **Substitution approach:** Set q to a specific algebraic number (e.g., q = 2, or q = a prime). Run the polynomial algorithms over Q. Verify results at the end using FPS. This is numerically robust but loses the formal q structure.

(B) **Laurent polynomial coefficient approach:** The coefficients of x are elements of Q[q, q^{-1}] (Laurent polynomials in q). Each coefficient is a finite sum of terms c_j * q^j. Polynomial operations (add, mul, gcd) work over this coefficient ring. This is the "correct" algebraic approach.

For approach (B), we already have the infrastructure: Q[q, q^{-1}] is exactly a FormalPowerSeries with no truncation, or equivalently, a BTreeMap<i64, QRat> (same structure as QExactPoly). So the coefficient ring for our polynomial in x is itself a QExactPoly (in q).

**Recommended implementation strategy:**

Phase 1: Use approach (A) with q specialized to a large prime p (say p = 10007). Run polynomial algorithms over Q. Verify via FPS. This is the concrete approach and works for all terminating series.

Phase 2 (optional): Implement approach (B) with Q[q, q^{-1}][x] for full symbolic q-Gosper.

## Recommended Build Order

Based on dependency analysis:

### Phase 1: Foundation (QExactPoly + QExactRatFunc)

**New file:** `polynomial.rs`

**Contents:**
- QExactPoly: BTreeMap<i64, QRat> with no truncation
  - Constructors: zero, one, monomial, from_coeffs, linear(a, b) for a + b*x
  - Arithmetic: add, sub, mul, neg, scalar_mul
  - Division: div_rem (Euclidean division), exact_div (panics on remainder)
  - Properties: degree, leading_coeff, is_zero, evaluate
  - q-shift: shift(q_val) maps p(x) -> p(q_val * x)
  - GCD: Euclidean GCD (polynomial GCD over Q[x])
  - Resultant: Sylvester matrix determinant or subresultant chain
  - Content and primitive part
- QExactRatFunc: numer/denom pair in lowest terms
  - Constructors: from_poly, from_numer_denom (auto-cancels GCD)
  - Arithmetic: add, sub, mul, div, neg
  - q-shift: shift both numer and denom

**Test targets:** All ring axioms, GCD correctness, resultant properties, q-shift composition.

**Lines estimate:** ~600-800 lines.

### Phase 2: q-Gosper

**New file:** `gosper.rs`

**Contents:**
- Bridge: HypergeometricSeries -> QExactRatFunc (term ratio)
  - Each QMonomial parameter -> linear factor in x
  - Handle the extra factor [(-1)^k * q^{k(k-1)/2}]^{1+s-r}
- q-Dispersion: compute the set of positive integers j where gcd(P(x), Q(q^j * x)) is nontrivial
- Gosper splitting: decompose ratio into (a, b, c) with coprimality
- Key equation: solve a(x)*f(q*x) - b(x/q)*f(x) = c(x) for polynomial f
  - Use existing linalg RREF for the linear system
- GosperResult enum
- FPS verification of results

**Dependencies:** polynomial.rs, linalg.rs (existing)

**Lines estimate:** ~500-700 lines.

### Phase 3: q-Zeilberger (Creative Telescoping)

**New file:** `zeilberger.rs`

**Contents:**
- **Concrete-n approach (primary):**
  - For trial order J = 1, 2, ..., max_order:
    - For n = 0, 1, ..., J+extra_rows:
      - Evaluate S(n) = sum_k F(n,k) using eval_phi
      - Evaluate S(n+1), ..., S(n+J) similarly
    - Build coefficient matrix [S(n), S(n+1), ..., S(n+J)] for each n
    - Use RREF to find null space (= recurrence coefficients)
  - Verify by checking additional values of n
- **Full algebraic approach (optional/later):**
  - Iterate Gosper over combined summand with parametric c_i
  - Requires bivariate rational functions
- QRecurrence type
- ZeilbergerResult enum
- FPS verification

**Dependencies:** hypergeometric.rs (eval_phi), linalg.rs, gosper.rs (for algebraic approach)

**Lines estimate (concrete approach):** ~300-400 lines.

### Phase 4: WZ Certificates and Proof Integration

**New file:** `wz.rs`

**Contents:**
- WZCertificate: extracted from Zeilberger's certificate output
- verify_wz_certificate: check F(n+1,k) - F(n,k) = G(n,k+1) - G(n,k) via FPS
- prove_hypergeometric_identity: top-level proving function
  - Tries try_all_summations (existing) first
  - Falls back to q-Zeilberger + WZ
  - Produces unified ProofResult
- Integration with existing ProofResult pattern from identity/prove.rs

**Dependencies:** zeilberger.rs, hypergeometric.rs (eval_phi, try_all_summations)

**Lines estimate:** ~300-400 lines.

### Phase 5: Python DSL Integration

**Modified file:** `crates/qsym-python/src/dsl.rs`

**New functions:**
- `q_gosper(series, variable, truncation_order)` -> GosperResult
- `q_zeilberger(series, variable, truncation_order, max_order)` -> ZeilbergerResult
- `wz_prove(identity, variable, truncation_order)` -> WZProofResult

**Dependencies:** All above modules.

## Integration Points with Existing Architecture

### 1. HypergeometricSeries (hypergeometric.rs)

**Connection:** Bridge layer converts HypergeometricSeries to term ratio for q-Gosper. The existing upper/lower QMonomial arrays directly produce linear factors.

**Modification needed:** Add a method `fn term_ratio_at_n(&self, n: i64) -> QExactRatFunc` that specializes n and produces the term ratio.

### 2. eval_phi / eval_psi (hypergeometric.rs)

**Connection:** Used by concrete-n approach in q-Zeilberger to evaluate S(n) for specific n values. Used by FPS verification in all algorithms.

**No modification needed.** Called as-is.

### 3. linalg.rs (RREF, null space)

**Connection:** q-Gosper's key equation and q-Zeilberger's recurrence finding both reduce to linear systems over QRat, solvable by existing rational_null_space.

**No modification needed.** The existing RREF over QRat handles all cases.

### 4. identity/prove.rs (ProofResult)

**Connection:** WZ proofs should integrate with the existing ProofResult pattern, possibly extending it or creating a parallel HypergeometricProofResult.

**Modification needed:** Consider adding a new variant to ProofResult or creating a separate proof result type for hypergeometric identities (as opposed to eta-quotient identities).

### 5. try_all_summations (hypergeometric.rs)

**Connection:** q-Gosper subsumes the pattern-matching summation formulas. The workflow should be: try known formulas first (fast), then fall back to algorithmic methods (slower but more general).

**No modification needed.** Called as a pre-check before running the full algorithm.

### 6. QMonomial (mod.rs)

**Connection:** QMonomial arithmetic (mul, div, is_q_neg_power) is used in the bridge layer to extract parameters for the polynomial representation.

**Possible extension:** Add `fn to_poly_coeff(&self) -> (QRat, i64)` that separates coefficient and q-power for polynomial construction.

## Scalability Considerations

| Concern | Small (deg <10) | Medium (deg 10-50) | Large (deg >50) |
|---------|-----------------|--------------------|--------------------|
| QExactPoly GCD | Euclidean, fast | Euclidean, still fast | Consider subresultant |
| q-Gosper splitting | <10 dispersion checks | 10-50 GCD calls | Optimize dispersion via resultant |
| q-Zeilberger (concrete-n) | J=1-2, few evals | J=3-5, moderate evals | J>5, many large FPS evals |
| FPS verification | O(T) trivial | O(T) fine | O(T) memory for T>10000 |

The concrete-n approach for q-Zeilberger scales well because each evaluation is an independent FPS computation (already optimized in eval_phi). The bottleneck is the number of n values needed, which is J+1 where J is the recurrence order. For most q-hypergeometric identities in practice, J <= 3.

## Sources

- [Koepf, "Hypergeometric Summation" (Springer Universitext)](https://link.springer.com/book/10.1007/978-1-4471-6464-7) -- Comprehensive reference for Gosper, Zeilberger, and q-analogues
- [Petkovsek, Wilf, Zeilberger, "A=B"](https://www.taylorfrancis.com/books/mono/10.1201/9781439864500/) -- Foundational text on WZ theory
- [Paule, Riese, "A Mathematica q-Analogue of Zeilberger's Algorithm"](https://www3.risc.jku.at/publications/download/risc_117/Paule_Riese.pdf) -- q-Zeilberger via algebraic approach
- [Koopf, Boing, "Algorithms for q-Hypergeometric Summation"](https://www.sciencedirect.com/science/article/pii/S074771719890339X) -- qsum package with q-Gosper, q-Zeilberger, q-Petkovsek
- [MathWorld: q-Zeilberger Algorithm](https://mathworld.wolfram.com/q-ZeilbergerAlgorithm.html)
- [MathWorld: Wilf-Zeilberger Pair](https://mathworld.wolfram.com/Wilf-ZeilbergerPair.html)
- [NIST implementations page](https://math.nist.gov/opsf/projects/zeil.html) -- Overview of available implementations
- [Chen et al., unified reduction 2025](https://arxiv.org/abs/2501.03837) -- Recent unified framework for hypergeometric and q-hypergeometric creative telescoping
- [Symbolica CAS (Rust)](https://symbolica.io/) -- Reference for Rust polynomial ring implementation patterns
- [polynomial-ring crate](https://crates.io/crates/polynomial-ring) -- Rust polynomial with resultant (AGPL, not usable directly)
