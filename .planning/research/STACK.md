# Technology Stack: Algorithmic q-Hypergeometric Identity Proving

**Project:** q-Kangaroo -- q-Zeilberger, Creative Telescoping, q-Gosper, WZ Certificates
**Researched:** 2026-02-15
**Overall confidence:** HIGH (algorithms well-documented in literature; Rust ecosystem assessed)

## Executive Assessment

The q-Zeilberger/creative telescoping/q-Gosper/WZ family of algorithms requires a fundamentally different computational substrate than what q-Kangaroo currently has. The existing engine operates on **formal power series** (FPS) -- truncated numerical expansions in q. The new algorithms operate on **symbolic polynomials and rational functions** in variables like q^n and q^k, requiring polynomial GCD, resultant computation, polynomial equation solving, and shift/q-shift operator algebra.

**The core question is: external polynomial crate or build from scratch?**

**Recommendation: Build a purpose-built polynomial module within qsym-core.** The algorithms require a narrow but deep set of polynomial operations over QRat coefficients. No existing MIT-licensed Rust crate provides exactly this. Building in-house gives control over coefficient representation (reusing existing QRat/QInt), avoids license contamination, and allows optimization for the specific access patterns of Gosper/Zeilberger.

## Recommended Stack

### No New External Dependencies

The existing dependency set (rug, smallvec, rustc-hash, serde) is sufficient. All new capabilities are built as pure Rust modules within qsym-core, leveraging the existing QRat exact arithmetic.

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| rug | 1.28 (existing) | Arbitrary-precision QRat coefficients for polynomials | Already in use. GMP-backed exact arithmetic is the correct coefficient domain for Gosper/Zeilberger. No change needed. |
| No new crate | N/A | Polynomial arithmetic | Build in-house. See rationale below. |

### New Internal Modules (within qsym-core)

| Module | Purpose | Key Types |
|--------|---------|-----------|
| `poly/` | Univariate polynomial arithmetic over QRat | `QRatPoly` -- dense polynomial with QRat coefficients |
| `poly/gcd.rs` | Polynomial GCD via subresultant PRS | `gcd()`, `extended_gcd()`, `resultant()` |
| `poly/ops.rs` | Arithmetic, division, composition, evaluation | `div_rem()`, `pseudo_div()`, `compose()`, `eval()`, `shift()` |
| `qseries/gosper.rs` | q-Gosper's algorithm | `QGosperResult`, `gosper_normal()`, `q_gosper()` |
| `qseries/zeilberger.rs` | q-Zeilberger / creative telescoping | `Recurrence`, `Certificate`, `q_zeilberger()` |
| `qseries/wz.rs` | WZ proof certificates | `WZPair`, `wz_certificate()`, `verify_wz()` |

### Core Data Structures Needed

#### 1. QRatPoly -- Univariate Polynomial over QRat

```rust
/// Dense univariate polynomial with QRat coefficients.
/// Represents p(x) = c_0 + c_1*x + c_2*x^2 + ... + c_d*x^d
/// where x is an abstract indeterminate (n, q^n, q^k, etc.).
///
/// Dense representation because Gosper/Zeilberger polynomials are
/// typically low-degree (< 50) with most coefficients nonzero.
pub struct QRatPoly {
    /// Coefficients in ascending degree order. Last element is nonzero (leading coeff).
    /// Empty vec represents the zero polynomial.
    coeffs: Vec<QRat>,
}
```

**Why dense, not sparse:** The polynomials arising in Gosper/Zeilberger are typically low-degree (degree bounded by the number of Pochhammer parameters). Sparse representation adds overhead for polynomials of degree 5-20 where most coefficients are nonzero. Dense is simpler, faster for these sizes, and well-suited to Euclidean GCD.

**Why not BTreeMap like FPS:** FPS can have thousands of terms with most being zero (sparse in nature). Gosper polynomials are qualitatively different -- low degree, dense fill.

#### 2. QRatRationalFunc -- Rational Function over QRat

```rust
/// Rational function p(x)/q(x) over QRat, always in reduced form.
/// Represents elements of the fraction field Q(q)[x] or Q(q^n, q^k).
pub struct QRatRationalFunc {
    numer: QRatPoly,
    denom: QRatPoly,  // Invariant: monic, gcd(numer, denom) = 1
}
```

**Why needed:** The q-Gosper algorithm works with the ratio t(k+1)/t(k) which is a rational function in q^k. The WZ certificate R(n,k) is a rational function. Creative telescoping produces recurrence coefficients that are rational functions in q^n.

#### 3. Recurrence -- Linear Recurrence with Polynomial Coefficients

```rust
/// Linear recurrence: c_0(n)*f(n) + c_1(n)*f(n+1) + ... + c_J(n)*f(n+J) = 0
/// where each c_i is a polynomial in q^n (represented as QRatPoly).
pub struct Recurrence {
    /// Coefficients c_0, c_1, ..., c_J as polynomials in q^n.
    coefficients: Vec<QRatPoly>,
    /// The order J of the recurrence.
    order: usize,
}
```

**Why needed:** This is the primary output of q-Zeilberger. Given a definite sum S(n) = sum_k F(n,k), q-Zeilberger produces a recurrence satisfied by S(n).

#### 4. WZPair -- Wilf-Zeilberger Proof Certificate

```rust
/// A WZ pair (F, G) where F(n+1,k) - F(n,k) = G(n,k+1) - G(n,k).
/// G(n,k) = R(n,k) * F(n,k) where R is the rational certificate.
pub struct WZPair {
    /// The original summand F(n,k) as a q-hypergeometric description.
    summand: HypergeometricSeries,  // existing type
    /// The proof certificate R(n,k) as a rational function.
    certificate: QRatRationalFunc,
    /// The recurrence satisfied by sum_k F(n,k).
    recurrence: Recurrence,
}
```

## Required Polynomial Operations

These are the specific operations needed by the algorithms, derived from analyzing Gosper/Zeilberger as described in Petkovsek-Wilf-Zeilberger "A=B" and Koornwinder's rigorous q-analogue description.

### For q-Gosper's Algorithm

| Operation | Why Needed | Complexity |
|-----------|------------|------------|
| Polynomial arithmetic (+, -, *, scalar) | Basic manipulation of p, q, r polynomials | O(d) to O(d^2) |
| `div_rem(a, b)` -> (quotient, remainder) | Euclidean division in GCD computation | O(d^2) |
| `gcd(a, b)` via subresultant PRS | Core of Gosper normal form: find gcd(a(n), b(n+h)) | O(d^2) |
| `resultant(a, b)` | Compute dispersion: Res_n(a(n), b(n+h)) to find integer h values where gcd is nontrivial | O(d^3) |
| `shift(p, h)` -> p(x+h) | Replace x by x+h in polynomial (Taylor shift) | O(d^2) |
| `compose(p, q)` -> p(q(x)) | Substitute one polynomial into another | O(d^2 * e) |
| `degree()`, `leading_coeff()` | Degree bounds for solution polynomial in key equation | O(1) |
| `eval(p, r)` -> QRat | Evaluate polynomial at rational point | O(d) |
| `monic(p)` -> p/lc(p) | Normalize polynomials in Gosper decomposition | O(d) |
| Solve `a(k)*x(k+1) - b(k-1)*x(k) = c(k)` for polynomial x | The "key equation" of Gosper's algorithm; solve by undetermined coefficients | O(d^2) |

### For q-Zeilberger's Algorithm (Creative Telescoping)

| Operation | Why Needed |
|-----------|------------|
| All of q-Gosper (used as subroutine) | q-Zeilberger calls q-Gosper iteratively for increasing recurrence orders |
| Polynomial in q^n with QRat coefficients | Recurrence coefficients are polynomials in q^n |
| Linear system solving over QRat | Determine recurrence coefficients via undetermined coefficients |
| Rational function arithmetic | Certificate manipulation |

### For WZ Certificates

| Operation | Why Needed |
|-----------|------------|
| Rational function construction | Certificate R(n,k) = G(n,k)/F(n,k) |
| Rational function simplification (GCD cancel) | Keep certificate in reduced form |
| Verification: check F(n+1,k) - F(n,k) = G(n,k+1) - G(n,k) | Prove correctness of certificate |

## Why Not External Crates

### polynomial-ring (v0.5.1)

**License: AGPL-3.0-or-later.** This is a copyleft license incompatible with q-Kangaroo's MIT license. Using it would require relicensing the entire project under AGPL. **Rejected.**

Despite having excellent API coverage (resultant, pseudo-division, square-free), the license makes it unusable.

### ring-algorithm (v0.8.0)

**License: AGPL-3.0-or-later.** Same license problem as polynomial-ring (same author). **Rejected.**

### feanor-math (v3.5.13)

**License: MIT.** Compatible. Has polynomial GCD, resultant, factoring, linear algebra, Euclidean algorithm.

**However, rejected for these reasons:**

1. **Performance warning:** Documentation explicitly states "operations with polynomials over infinite rings (integers, rationals, number fields) are currently very slow." This is exactly our use case.
2. **Heavyweight dependency:** Large crate (3.5.13 with 98 releases) with its own bigint implementation. We already have rug/GMP for exact arithmetic. Adding feanor-math means two arbitrary-precision integer implementations in the same binary.
3. **Trait complexity:** The RingBase/RingStore trait system adds abstraction overhead. We need polynomials over QRat, period. Generic ring abstraction is unnecessary complexity.
4. **No rug interop:** feanor-math uses its own `RustBigintRing` or optional MPIR bindings. It does not natively work with rug::Rational. An adapter layer would be needed, adding allocation overhead on every coefficient operation.
5. **Edition 2024:** Matches our Rust edition, but the dependency is still too heavy for what we need.

### Symbolica

**License: Source-available, not open-source.** Free for hobbyists; requires commercial license for employment use. **Rejected** for an open-source project.

### rust-poly (v0.4.3)

**License: MIT.** Compatible. But only supports floating-point polynomials. No exact rational arithmetic. **Rejected.**

### algebraics

**License: LGPL-2.1-or-later.** Has polynomial GCD via subresultant PRS. LGPL is more permissive than AGPL but still requires dynamic linking or LGPL relicensing for static linking in Rust. **Marginal; prefer building in-house for simplicity.**

### Build In-House (Recommended)

**Rationale:**

1. **Narrow scope:** We need ~500-800 lines of polynomial code, not a general-purpose computer algebra system. The operations are: arithmetic, div_rem, GCD (subresultant PRS), resultant, shift, compose, eval, and solving linear polynomial equations by undetermined coefficients.
2. **Native QRat integration:** Polynomials use the same QRat type already used everywhere in qsym-core. No adapter layer, no extra allocations, no trait gymnastics.
3. **MIT license:** No licensing concerns.
4. **Optimizable:** Can be tuned for the exact polynomial sizes (degree 5-30) and access patterns (many GCDs of shifted polynomials) that Gosper/Zeilberger produce.
5. **Existing infrastructure:** The project already has exact rational arithmetic (QRat), linear algebra (rational_null_space, RREF), and the mathematical domain knowledge. Adding polynomial arithmetic is a natural extension.
6. **Precedent:** The project already implements all its own q-series algorithms (aqprod, eval_phi, prodmake, etc.) rather than depending on external math libraries. This is consistent with the project's philosophy.

## Existing Infrastructure Reuse

| Existing Component | How It's Reused |
|--------------------|-----------------|
| `QRat` (number.rs) | Polynomial coefficients. All polynomial arithmetic uses QRat. |
| `QInt` (number.rs) | Integer degree bounds, exponents in shift operations. |
| `rational_null_space` (linalg.rs) | Solving the Gosper key equation by undetermined coefficients: set up linear system, find null space. |
| `build_coefficient_matrix` (linalg.rs) | Matrix construction for the undetermined coefficients approach. |
| `HypergeometricSeries` (hypergeometric.rs) | Input to q-Zeilberger: the summand F(n,k) is described by its hypergeometric parameters. |
| `QMonomial` (mod.rs) | Parameters of hypergeometric series; used to compute the term ratio t(k+1)/t(k). |
| `eval_phi` / `eval_psi` (hypergeometric.rs) | Numerical verification of recurrences and certificates against FPS expansions. |
| `FormalPowerSeries` (series/) | Verification: expand both sides of an identity to O(q^N) and compare. |

## Integration Points

### HypergeometricSeries -> q-Gosper/q-Zeilberger

The term ratio t(k+1)/t(k) for a q-hypergeometric term is a rational function in q^k. The existing `HypergeometricSeries` stores upper/lower parameters as `Vec<QMonomial>`. The ratio is:

```
t(k+1)/t(k) = prod_i(1 - a_i*q^k) / [prod_j(1 - b_j*q^k) * (1 - q^{k+1})] * extra_factor * z
```

This ratio must be converted from a "product of linear factors in q^k" form into a `QRatRationalFunc` (polynomial quotient). This is the bridge between the existing hypergeometric infrastructure and the new polynomial algorithms.

### FPS Verification

After q-Zeilberger produces a recurrence for S(n) = sum_k F(n,k), verification works by:
1. Computing S(n) as an FPS for several values of n (using existing eval_phi)
2. Checking the recurrence holds numerically to O(q^N)

This uses existing FPS arithmetic with no changes.

### Python API Extension

New DSL functions in crates/qsym-python/src/dsl.rs:
- `q_gosper(series, variable)` -> indefinite sum or "not q-hypergeometric"
- `q_zeilberger(series, variable)` -> Recurrence + Certificate
- `wz_prove(identity)` -> WZPair (certificate + verification)
- `verify_recurrence(series, recurrence, order)` -> bool

These follow the same pattern as existing DSL functions (73 currently).

## Polynomial Module Specification

### Required API (minimum viable for Gosper/Zeilberger)

```rust
// --- Construction ---
QRatPoly::zero() -> QRatPoly
QRatPoly::one() -> QRatPoly
QRatPoly::constant(c: QRat) -> QRatPoly
QRatPoly::monomial(c: QRat, deg: usize) -> QRatPoly
QRatPoly::from_coeffs(coeffs: Vec<QRat>) -> QRatPoly
QRatPoly::x() -> QRatPoly  // the indeterminate

// --- Queries ---
QRatPoly::degree(&self) -> Option<usize>  // None for zero poly
QRatPoly::leading_coeff(&self) -> QRat
QRatPoly::coeff(&self, i: usize) -> QRat
QRatPoly::is_zero(&self) -> bool
QRatPoly::is_constant(&self) -> bool

// --- Arithmetic ---
impl Add, Sub, Mul, Neg for QRatPoly
QRatPoly::scalar_mul(&self, s: &QRat) -> QRatPoly
QRatPoly::monic(&self) -> QRatPoly  // divide by leading coefficient

// --- Division ---
QRatPoly::div_rem(&self, other: &QRatPoly) -> (QRatPoly, QRatPoly)
QRatPoly::pseudo_div(&self, other: &QRatPoly) -> (QRatPoly, QRatPoly, QRat)

// --- GCD and Resultant ---
pub fn poly_gcd(a: &QRatPoly, b: &QRatPoly) -> QRatPoly  // subresultant PRS
pub fn poly_extended_gcd(a: &QRatPoly, b: &QRatPoly) -> (QRatPoly, QRatPoly, QRatPoly)
pub fn poly_resultant(a: &QRatPoly, b: &QRatPoly) -> QRat

// --- Manipulation ---
QRatPoly::eval(&self, x: &QRat) -> QRat  // Horner's method
QRatPoly::shift(&self, h: &QRat) -> QRatPoly  // p(x) -> p(x + h)
QRatPoly::compose(&self, other: &QRatPoly) -> QRatPoly  // p(q(x))
QRatPoly::derivative(&self) -> QRatPoly
QRatPoly::integer_roots(&self) -> Vec<QRat>  // find rational roots (for dispersion)

// --- Rational Functions ---
QRatRationalFunc::new(numer: QRatPoly, denom: QRatPoly) -> QRatRationalFunc  // auto-reduces
impl Add, Sub, Mul, Div, Neg for QRatRationalFunc
QRatRationalFunc::eval(&self, x: &QRat) -> Option<QRat>
```

### Implementation Effort Estimate

| Component | Estimated Lines | Difficulty |
|-----------|----------------|------------|
| QRatPoly (struct + arithmetic) | 200-250 | Low -- straightforward dense polynomial ops |
| div_rem + pseudo_div | 80-100 | Low -- standard Euclidean division |
| poly_gcd (subresultant PRS) | 100-150 | Medium -- need careful handling of content/primitive part |
| poly_resultant | 50-80 | Medium -- derived from subresultant chain |
| shift, compose, derivative | 80-100 | Low -- well-known algorithms |
| integer_roots (rational root theorem) | 40-60 | Low -- test divisors of constant/leading |
| QRatRationalFunc | 100-150 | Low-Medium -- arithmetic with auto-reduction |
| **Total polynomial infrastructure** | **650-890** | |

This is comparable to the existing hypergeometric.rs (1383 lines) or bailey.rs (747 lines) -- well within the project's established scale.

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Polynomial arithmetic | Build in-house (QRatPoly) | feanor-math (MIT) | Slow for rationals (documented), heavyweight dep, no rug interop, unnecessary abstraction |
| Polynomial arithmetic | Build in-house | polynomial-ring | AGPL license -- incompatible with MIT project |
| Polynomial GCD | Subresultant PRS | Euclidean GCD | Euclidean GCD over Q causes coefficient explosion. Subresultant PRS controls growth. |
| Coefficient type | QRat (existing) | num::Rational64 | 64-bit overflow for Gosper polynomials with large q-Pochhammer parameters. Need arbitrary precision. |
| Poly representation | Dense Vec<QRat> | Sparse BTreeMap<usize, QRat> | Gosper/Zeilberger polynomials are low-degree, mostly dense. Vec is simpler and faster. |
| Resultant | Via subresultant chain | Sylvester matrix determinant | Subresultant is more numerically stable and shares code with GCD. |

## Sources

**HIGH CONFIDENCE (algorithms):**
- Petkovsek, Wilf, Zeilberger, "A=B" (1996) -- definitive reference for Gosper/Zeilberger/WZ algorithms
- [Koornwinder, "On Zeilberger's algorithm and its q-analogue"](https://staff.fnwi.uva.nl/t.h.koornwinder/art/1993/zeilbalgo.pdf) -- rigorous q-analogue description
- [Paule & Riese, "A Mathematica q-analogue of Zeilberger's Algorithm"](https://www3.risc.jku.at/publications/download/risc_117/Paule_Riese.pdf) -- qGFF approach
- [SymPy gosper.py source](https://www.aidoczh.com/sympy/_modules/sympy/concrete/gosper.html) -- reference implementation of Gosper's algorithm
- [AeqB-sage repository](https://github.com/benyoung/AeqB-sage) -- Sage reference implementation
- [Zeilberger, "The Method of Creative Telescoping" (1991)](https://sites.math.rutgers.edu/~zeilberg/mamarimY/creativeT.pdf) -- original creative telescoping paper

**HIGH CONFIDENCE (Rust ecosystem):**
- [polynomial-ring v0.5.1](https://lib.rs/crates/polynomial-ring) -- AGPL-3.0, has resultant/pseudo-division (license incompatible)
- [feanor-math v3.5.13](https://lib.rs/crates/feanor-math) -- MIT, has polynomial GCD/resultant/factoring (slow for rationals per docs)
- [ring-algorithm v0.8.0](https://lib.rs/crates/ring-algorithm) -- AGPL-3.0 (license incompatible)
- [Symbolica](https://symbolica.io/license/) -- source-available, not open-source (commercial license required)
- [algebraics crate](https://docs.rs/algebraics/latest/algebraics/) -- LGPL-2.1, has subresultant GCD
- [rust-poly v0.4.3](https://lib.rs/crates/rust-poly) -- MIT, floating-point only (no exact arithmetic)

**MEDIUM CONFIDENCE:**
- [arXiv:2501.03837 -- Unified reduction for creative telescoping (2025)](https://arxiv.org/abs/2501.03837) -- recent algorithmic advances
- [qZeil package documentation](https://www3.risc.jku.at/research/combinat/software/ergosum/RISC/qZeil.html) -- Mathematica implementation reference
- [Hipergeo for Maxima](https://github.com/cassiopagnoncelli/hipergeo) -- Maxima implementation including WZ certificates

---
*Stack research for: q-Zeilberger, creative telescoping, q-Gosper, WZ proof certificates*
*Researched: 2026-02-15*
