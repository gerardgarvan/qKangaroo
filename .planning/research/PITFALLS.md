# Domain Pitfalls: Algorithmic q-Hypergeometric Identity Proving

**Domain:** q-Gosper, q-Zeilberger, creative telescoping, WZ certificates in exact arithmetic
**Researched:** 2026-02-15
**Confidence:** HIGH (algorithmic structure), MEDIUM (performance characteristics), LOW (Rust-specific polynomial ring libraries)

---

## Critical Pitfalls

Mistakes that cause rewrites or major architectural issues.

---

### Pitfall 1: Missing Polynomial Ring Infrastructure (The Representation Gap)

**What goes wrong:**
q-Gosper and q-Zeilberger operate on **polynomials and rational functions in q^n** (the q-shift variable), not on FPS coefficients or QMonomials. The algorithms require:
- Univariate polynomials in `q^k` (for q-Gosper's p, q, r decomposition)
- Laurent polynomials in `q^k` (for the q-GFF step)
- Rational functions `p(q^k)/r(q^k)` (for the term ratio)
- Polynomial arithmetic: GCD, resultant, factorization, degree computation
- Solving polynomial equations (finding integer roots of resultants)

q-Kangaroo currently has **none of these**. `QMonomial` is `coeff * q^power` (a single term), `FormalPowerSeries` is a truncated power series in q, and `QRat` is a single rational number. There is no polynomial-in-q^n type.

**Why it happens:**
The existing system was designed for *evaluating* q-series to finite precision, not for *symbolic manipulation* of the summand's ratio as a function of the summation index. These are fundamentally different operations: FPS works in `Q[[q]]` (power series ring), while Gosper/Zeilberger work in `Q(q^n)[q^k]` or `Q(q^n, q^k)` (polynomial/rational function rings in the shift variable).

**Consequences:**
Without a proper polynomial ring type, you cannot:
1. Factor the term ratio into coprime polynomials (the p, q, r decomposition in Gosper)
2. Compute the q-greatest factorial factorization (qGFF)
3. Solve the key recurrence `a(n)*x(n+1) - b(n-1)*x(n) = c(n)` for polynomial x
4. Compute resultants for the dispersion set
5. Build the Zeilberger linear system with polynomial coefficients

Attempting to shoehorn FPS arithmetic into these roles will produce wrong results or require O(N^2) workarounds that defeat the purpose of having an algorithmic prover.

**Prevention:**
Build a dedicated `QPolynomial` type (univariate polynomial in a formal variable, with QRat coefficients) BEFORE implementing q-Gosper. This type needs:
```rust
struct QPolynomial {
    // Sparse representation: degree -> coefficient
    coeffs: BTreeMap<i64, QRat>,  // i64 for Laurent polynomials
}

// Required operations:
impl QPolynomial {
    fn degree(&self) -> Option<i64>;
    fn leading_coeff(&self) -> QRat;
    fn gcd(a: &Self, b: &Self) -> Self;  // Euclidean algorithm
    fn resultant(a: &Self, b: &Self) -> QRat;
    fn evaluate(&self, x: &QRat) -> QRat;
    fn shift(&self, delta: i64) -> Self;  // p(x) -> p(x + delta)
    fn q_shift(&self, q_pow: i64) -> Self;  // p(q^k) -> p(q^{k+1})
    fn div_rem(a: &Self, b: &Self) -> (Self, Self);
    fn integer_roots(&self) -> Vec<i64>;  // for dispersion
}
```

Also need `QRationalFunction` = `QPolynomial / QPolynomial` with automatic GCD cancellation.

**Detection:**
If you find yourself converting between FPS and "polynomial-like" operations, or using truncated FPS where exact polynomial arithmetic is needed, you have hit this gap.

**Phase to address:** First phase (foundation). Everything else depends on this.

**Confidence:** HIGH -- this is a fundamental type system requirement visible from the algorithm descriptions in Koornwinder (1993) and Paule-Riese.

---

### Pitfall 2: Coefficient Explosion in Exact Rational Arithmetic

**What goes wrong:**
During q-Gosper and q-Zeilberger, intermediate QRat coefficients grow **exponentially** in numerator/denominator size. This is called "intermediate expression swell" in computer algebra. Key explosion points:

1. **Polynomial GCD via Euclidean algorithm:** Each step of the Euclidean algorithm over Q[x] can square the bit-size of coefficients. A degree-d GCD computation can produce intermediates with O(2^d) digit counts.

2. **Resultant computation:** The resultant of two polynomials of degree d has coefficients that are determinants of d x d matrices of the input coefficients. For q-Pochhammer ratios, these get large fast.

3. **Linear system solving in Zeilberger:** The qZeil implementation by Paule-Riese found that **95% of runtime** was spent solving the polynomial system, reduced to 30-40% only after aggressive preprocessing to extract constant factors.

4. **q-GFF factorization:** The greatest factorial factorization requires iterated GCD computations with q-shifted polynomials, each amplifying coefficient size.

5. **WZ certificate rational functions:** The certificate R(n,k) = G(n,k)/F(n,k) often has large polynomial numerators and denominators even when the identity itself has small coefficients.

**Why it happens:**
Exact rational arithmetic (QRat via GMP/rug) never loses precision, but the *price* is that intermediate computations accumulate large numerators and denominators that are only cancelled at the end. Unlike floating-point where information is silently lost, exact arithmetic faithfully preserves every factor, leading to memory and time blowup.

**Consequences:**
- Memory exhaustion on moderate-size inputs (10+ parameter identities)
- Quadratic or worse time growth in what should be linear-time operations
- GMP allocation overhead dominates actual computation
- Users experience "hangs" with no progress indication

**Prevention:**

1. **Normalize aggressively:** After EVERY polynomial arithmetic operation, divide all coefficients by their GCD. For QRat coefficients, call `rug::Rational::canonicalize()` (which q-Kangaroo's QRat should already do, but verify in hot loops).

2. **Use modular arithmetic for GCD:** Instead of Euclidean GCD over Q[x], use the modular approach:
   - Compute GCD modulo several primes p_1, ..., p_k
   - Reconstruct via Chinese Remainder Theorem + rational reconstruction
   - This avoids intermediate coefficient swell entirely

3. **Factor out content early:** Before any GCD/resultant, extract the "content" (GCD of all coefficients) from each polynomial. This is the preprocessing that cut qZeil runtime from 95% to 30-40%.

4. **Bound computation depth:** Set configurable limits on polynomial degree, coefficient bit-size, and computation steps. Return "inconclusive" rather than hanging.

5. **Consider lazy/deferred evaluation:** For WZ certificates, store the certificate in factored form (as products of shifted Pochhammer symbols) rather than expanding to a single rational function.

**Detection:**
- QRat denominators exceeding 1000 digits in intermediate steps
- Single GCD call taking >1 second
- Polynomial multiplication producing coefficients 10x larger than inputs

**Phase to address:** Polynomial ring implementation phase (must be built into `QPolynomial` from the start, not retrofitted).

**Confidence:** HIGH -- this is the most documented pitfall in all Zeilberger implementations. Paule-Riese explicitly describe the 95% -> 30-40% runtime improvement.

**Sources:**
- [Paule-Riese qZeil implementation](https://www3.risc.jku.at/publications/download/risc_117/Paule_Riese.pdf)
- [Polynomial GCD coefficient swell](https://en.wikipedia.org/wiki/Polynomial_greatest_common_divisor)
- [Efficient rational creative telescoping](https://www.sciencedirect.com/science/article/abs/pii/S0747717121000535)

---

### Pitfall 3: q-Greatest Factorial Factorization (qGFF) is Not Standard Factorization

**What goes wrong:**
The critical Step 2 of q-Gosper requires decomposing the term ratio `r(k) = t_{k+1}/t_k` into coprime polynomials `a(k), b(k), c(k)` such that:
- `r(k) = a(k)/b(k) * c(k+1)/c(k)` (ordinary Gosper)
- In q-case: `r(q^k) = a(q^k)/b(q^k) * c(q^{k+1})/c(q^k)` with `gcd(a(q^k), b(q^{k+h})) = 1` for all non-negative integers h

This "q-dispersion" condition is NOT the same as ordinary coprimality. Two polynomials can be coprime in the usual sense but have GCD after q-shifting. Computing the full dispersion set requires:
1. Computing `resultant_x(a(x), b(q^j * x))` as a polynomial in `q^j`
2. Finding all non-negative integer values of j where the resultant vanishes
3. Iteratively dividing out common factors at each shift

**Why it happens:**
Developers familiar with ordinary polynomial factorization assume `gcd(a, b) = 1` is sufficient. But in the q-shift world, `a(q^k)` and `b(q^{k+3})` might share a factor even though `a(x)` and `b(x)` are coprime. The dispersion set captures all such "hidden" common factors.

**Consequences:**
- Wrong Gosper decomposition -> wrong or missed solutions
- Algorithm claims "no closed form" when one exists
- Silently incorrect WZ certificates that fail verification

**Prevention:**
1. Implement q-dispersion as a separate, well-tested function
2. Use resultant-based computation (not iterated trial GCD which is slower and error-prone)
3. Test with known examples where ordinary GCD misses the q-shifted common factor
4. The dispersion set is always finite (bounded by degree), so enumerate explicitly

**Detection:**
- WZ certificate verification fails (the telescoping equation doesn't hold)
- Known summable identities return "no closed form"
- Disagreement between algorithmic proof and FPS numerical verification

**Phase to address:** q-Gosper phase (core algorithm step).

**Confidence:** HIGH -- this is the mathematical core of the algorithm, described rigorously in Koornwinder (1993).

**Sources:**
- [Koornwinder 1993: rigorous description of q-Zeilberger](https://staff.fnwi.uva.nl/t.h.koornwinder/art/1993/zeilbalgo.pdf)
- [Fast polynomial dispersion computation](https://dl.acm.org/doi/pdf/10.1145/190347.190413)
- [Greatest factorial factorization and symbolic summation](https://www.sciencedirect.com/science/article/pii/S0747717185710498)

---

### Pitfall 4: Zeilberger's Incremental Order Search Can Hang

**What goes wrong:**
The q-Zeilberger algorithm works by trying to find a recurrence of order J = 1, then J = 2, then J = 3, etc. For each order J, it runs q-Gosper on a parametric sum involving J unknowns. If the minimum recurrence order is large (say J = 8), the algorithm must fail for J = 1 through 7 before finding the answer at J = 8.

Each failed attempt at order J involves:
- Setting up a polynomial system with J+1 unknowns
- Running q-Gosper (which itself involves GFF + degree bound + polynomial solving)
- Verifying that no polynomial solution exists (negative result, which is just as expensive as positive)

**Why it happens:**
There is no cheap a priori bound on the recurrence order for a given identity. Known theoretical bounds exist but are often very loose (e.g., the product of all parameter magnitudes). The practical strategy is incremental search starting from J = 1.

**Consequences:**
- For identities with high-order recurrences, the algorithm runs q-Gosper many times before succeeding
- Each unsuccessful q-Gosper call at order J is wasted computation (but necessary to prove minimality)
- No progress indication -- the algorithm appears hung between J = 1 and the true order
- Combined with coefficient explosion (Pitfall 2), each failed attempt gets slower

**Prevention:**

1. **Set a maximum recurrence order:** Default to J_max = 5 or 6. Most standard q-hypergeometric identities have recurrence orders 1-3. If J_max is exceeded, return "inconclusive" with a suggestion to increase the bound.

2. **Use FPS heuristic first:** Before running Zeilberger, check if the identity holds numerically to O(q^100) using the existing eval_phi. If it doesn't hold, don't even try the algorithmic proof. If it does hold, use the FPS check to estimate the recurrence order by looking at the coefficient recurrence.

3. **Parallel/incremental coefficient extraction:** Paule-Riese's preprocessing optimization should be applied: for each J, pre-extract constant factors from the polynomial system before solving.

4. **Cache q-Gosper subresults:** When moving from order J to J+1, some of the polynomial factorizations and GFF computations can be reused.

**Detection:**
- Algorithm takes >10 seconds on a "simple-looking" identity (likely high recurrence order)
- Order 1 and 2 attempts complete quickly, then order 3+ slows dramatically

**Phase to address:** q-Zeilberger phase (architecture decision at the start).

**Confidence:** HIGH -- this is inherent to the algorithm design, documented in all implementations.

**Sources:**
- [Zeilberger's algorithm (MathWorld)](https://mathworld.wolfram.com/ZeilbergersAlgorithm.html)
- [Creative telescoping bounds](https://www.researchgate.net/publication/278798184_The_ABC_of_Creative_Telescoping_---_Algorithms_Bounds_Complexity)

---

### Pitfall 5: WZ Certificate Boundary Term Errors

**What goes wrong:**
A WZ pair (F, G) satisfies `F(n+1, k) - F(n, k) = G(n, k+1) - G(n, k)`. Summing over k telescopes the RHS, but **only if the boundary terms vanish**: `lim_{k -> +inf} G(n, k) - lim_{k -> -inf} G(n, k) = 0`. For terminating sums with finite bounds [a, b], the condition is `G(n, b+1) = G(n, a)` (often both zero).

Three common boundary errors:
1. **Forgetting to check boundary terms at all** -- the certificate equation alone is necessary but NOT sufficient
2. **Off-by-one in summation bounds** -- checking G(n, b) instead of G(n, b+1), or G(n, a-1) instead of G(n, a)
3. **Non-terminating sums where boundary terms don't vanish** -- the certificate exists algebraically but the identity is false because the boundary contribution is nonzero

**Why it happens:**
The WZ method's elegance makes it tempting to verify only the local equation `F(n+1,k) - F(n,k) = G(n,k+1) - G(n,k)` (which is a purely algebraic check) and skip the analytic boundary check. This is especially tempting in exact arithmetic where the algebraic check is clean.

**Consequences:**
- "Proved" identities that are actually false
- Subtle errors that only manifest for specific parameter values
- Loss of trust in the entire proving engine

**Prevention:**

1. **Always verify boundary terms explicitly.** For a terminating sum from k=0 to k=N, check that `G(n, N+1) = 0` and `G(n, 0) = 0` (or whatever the appropriate boundary values are).

2. **For the q-case specifically:** termination comes from `(q^{-N}; q)_k = 0` for k > N. Verify that the certificate `R(n, k) * F(n, k)` vanishes at the boundary. This requires evaluating the certificate at specific k values, not just checking the recurrence.

3. **Separate "certificate found" from "identity proved":** The API should have distinct return types:
```rust
enum WZResult {
    Proved { certificate: ..., boundary_check: BoundaryVerification },
    CertificateFoundButBoundaryFails { certificate: ..., failing_boundary: ... },
    NoCertificateFound,
}
```

4. **Use FPS cross-check:** After algorithmic proof, always verify the first N coefficients via direct FPS computation (q-Kangaroo already has this capability via eval_phi). This catches boundary errors.

**Detection:**
- Certificate passes algebraic check but FPS verification disagrees
- Identity "proved" for parameters where it's known to be false
- Different results for terminating vs. non-terminating versions of the same identity

**Phase to address:** WZ certificate phase (verification logic).

**Confidence:** HIGH -- this is a well-known mathematical subtlety emphasized in A=B (Petkovsek-Wilf-Zeilberger).

**Sources:**
- [Wilf-Zeilberger pair (Wikipedia)](https://en.wikipedia.org/wiki/Wilf%E2%80%93Zeilberger_pair)
- [WZ Method (Maple Help)](https://www.maplesoft.com/support/help/maple/view.aspx?path=SumTools/Hypergeometric/WZMethod)

---

## Moderate Pitfalls

---

### Pitfall 6: QMonomial Cannot Represent q-Shift Polynomials

**What goes wrong:**
The existing `QMonomial { coeff: QRat, power: i64 }` represents a single term `c * q^m`. But q-Gosper's step functions and Zeilberger's operator polynomials require **polynomials in q^n** like `3*q^{2n} - 5*q^n + 2` (a polynomial in the variable `q^n`, NOT a power series in q).

Developers may try to reuse QMonomial or Vec<QMonomial> to represent these, but QMonomial has no concept of a "variable" -- it's always a concrete power of q. The distinction between `q^3` (a specific monomial) and `q^{3n}` (a function of n) is fundamental to the algorithms.

**Prevention:**
QPolynomial (from Pitfall 1) should explicitly track which variable it's a polynomial in. The q-Gosper algorithm operates on polynomials in `X = q^k` where k is the summation index. When these polynomials are evaluated at a specific k, they produce QRat values. But symbolically, they must remain unevaluated polynomials.

Do NOT attempt to "evaluate at large k and interpolate" -- this is both slower and less reliable than working symbolically.

**Phase to address:** Polynomial ring phase (same as Pitfall 1).

**Confidence:** HIGH -- direct inspection of q-Kangaroo's QMonomial type confirms the gap.

---

### Pitfall 7: Confusing q-Shift with Ordinary Shift

**What goes wrong:**
In ordinary Gosper/Zeilberger, the shift operator is `E_n: f(n) -> f(n+1)`. In the q-case, the shift operator is `E_q: f(q^n) -> f(q^{n+1})`, which is multiplication by q. This means:
- "Shifting a polynomial by 1" means replacing `q^n` by `q^{n+1} = q * q^n`, NOT `q^n + 1`
- The "q-falling factorial" is `(a; q)_n = prod_{k=0}^{n-1} (1 - a*q^k)`, where each factor involves MULTIPLICATION by q, not addition
- The degree in the q-case counts powers of `q^n`, and shifting up by 1 multiplies each monomial by the appropriate power of q

The entire factorization theory (GFF -> qGFF) changes because "consecutive integer roots" in the ordinary case become "consecutive q-power roots" in the q-case.

**Why it happens:**
The literature often presents q-Gosper as "almost unchanged" from ordinary Gosper (Koornwinder's words), which is true at the *structural* level but misleading at the *implementation* level. Every polynomial operation needs its q-analogue.

**Prevention:**
1. Implement q-shift as a first-class operation: `q_shift(p, j)` replaces `X` by `q^j * X` in polynomial p(X)
2. Write parallel test suites: for every ordinary-Gosper test case, have the q-analogue
3. Never use `p.evaluate(x + 1)` when you mean `p.evaluate(q * x)` -- these are different operations

**Phase to address:** q-Gosper implementation phase.

**Confidence:** HIGH -- this is the core mathematical distinction.

---

### Pitfall 8: Linear System Solving Bottleneck in q-Zeilberger

**What goes wrong:**
The final step of both q-Gosper (finding polynomial x(n)) and q-Zeilberger (finding recurrence coefficients) requires solving a linear system over Q. The system has:
- Variables: coefficients of the unknown polynomial (degree d gives d+1 unknowns)
- Equations: matching coefficients of q^k for each power (degree of a, b, c polynomials gives the count)

For q-Zeilberger at recurrence order J with parameter polynomials of degree D, the system size is approximately (J+1)*D x (J+1)*D. With exact QRat arithmetic and no modular tricks, Gaussian elimination on this system has complexity O(n^3 * B) where B is the bit-size of coefficients.

q-Kangaroo's existing `rational_null_space` in `linalg.rs` does naive RREF with QRat. This works for the relation discovery functions (findlincombo, etc.) where matrices are moderate-sized and coefficients are small. For Zeilberger's systems, the matrices can be larger AND the coefficients are polynomials in q (so each "entry" is itself a polynomial, not a single rational number).

**Why it happens:**
The relation discovery functions build matrices from FPS coefficients (small integers/rationals). Zeilberger's algorithm builds matrices from polynomial coefficients (which can have large numerators from Pochhammer products).

**Prevention:**
1. **Pre-simplify:** Before building the linear system, normalize all polynomials (extract content, cancel common factors). This is the Paule-Riese optimization.
2. **Consider fraction-free Gaussian elimination:** Instead of pivoting with exact division (which creates large denominators), use the Bareiss algorithm which keeps everything as integers until the end.
3. **Size the system before solving:** If the matrix exceeds a threshold (e.g., 50x50 with coefficients > 100 digits), warn and offer to switch to modular verification.
4. **Reuse linalg.rs infrastructure but add a "polynomial coefficient" mode.**

**Phase to address:** q-Zeilberger implementation phase.

**Confidence:** MEDIUM -- the bottleneck is well-documented (Paule-Riese), but the exact thresholds depend on implementation quality and GMP performance.

---

### Pitfall 9: Non-Terminating Input Detection

**What goes wrong:**
q-Gosper and q-Zeilberger are designed for **terminating** basic hypergeometric sums (where some upper parameter is `q^{-N}`). When applied to non-terminating sums:
- q-Gosper may loop indefinitely in the dispersion computation (unbounded shift search)
- q-Zeilberger's incremental order search never terminates (no finite recurrence exists for some non-terminating sums)
- The algorithms may produce formally correct recurrences that require additional convergence analysis

The existing `HypergeometricSeries::termination_order()` correctly detects termination, but:
1. The caller might not check it before invoking the prover
2. Some series terminate for specific parameter values but not in general (parametric termination)
3. Balanced/very-well-poised non-terminating sums DO have recurrences but require different algorithmic paths

**Prevention:**
1. **Gate all algorithmic proving functions** behind a termination check:
```rust
pub fn q_zeilberger(series: &HypergeometricSeries, ...) -> Result<Recurrence, ProofError> {
    if series.termination_order().is_none() {
        return Err(ProofError::NonTerminating);
    }
    // ...
}
```
2. For non-terminating sums, offer FPS-based verification (already exists) as fallback
3. Consider adding a separate `q_zeilberger_nonterminating` for well-poised cases with explicit convergence conditions
4. Add timeout/step-count limits to all algorithm loops

**Phase to address:** q-Gosper and q-Zeilberger phases (input validation).

**Confidence:** HIGH -- the termination requirement is mathematically necessary and the existing type already supports detection.

---

### Pitfall 10: Degree Bound Computation Returns Negative (Algorithm Says "No Solution")

**What goes wrong:**
In Gosper's algorithm Step 3, after decomposing the ratio into `a(n), b(n), c(n)`, you must find a polynomial `f(n)` satisfying `a(n)*f(n+1) - b(n-1)*f(n) = c(n)`. The degree of f is bounded by:
- `deg(f) = max(deg(a), deg(b)) - deg(c)` (when leading coefficients don't cancel)
- Or `deg(f) = deg(c) - max(deg(a), deg(b)) + [correction]` (alternative formulation depending on convention)

When the degree bound is **negative**, no polynomial solution exists, meaning the term is not Gosper-summable at this order. This is the algorithm's way of saying "try higher order" (in Zeilberger's outer loop) or "no closed form exists" (in standalone Gosper).

The pitfall: developers treat negative degree bound as an error condition (panic/crash) instead of a valid "no solution" result.

**Prevention:**
1. Return `Option<QPolynomial>` from the Gosper solving step (None = negative degree bound)
2. In the q-case, the degree bound computation is more subtle because "degree" is in terms of `q^k`, not k. Be careful with the degree convention.
3. Test with known non-summable terms (e.g., `1/(q;q)_k` has no q-hypergeometric antidifference)

**Phase to address:** q-Gosper implementation phase.

**Confidence:** HIGH -- this is Step 3 of the algorithm, described in Koepf's textbook.

---

### Pitfall 11: Integration with Existing Identity Proving (Eta-Quotient vs. Algorithmic)

**What goes wrong:**
q-Kangaroo already has `prove_eta_identity` which proves identities via the modular function valence formula. The new algorithmic prover (q-Zeilberger/WZ) is a DIFFERENT proof technique for a DIFFERENT class of identities. Confusion arises in several ways:

1. **Overlapping domains:** Some identities can be proved by BOTH methods (e.g., Rogers-Ramanujan can be expressed as eta-quotients AND as q-hypergeometric sums). If both provers exist, which takes precedence?

2. **Incompatible input formats:** Eta-quotient prover takes `EtaIdentity` (list of eta-quotient terms). Algorithmic prover takes `HypergeometricSeries`. The same mathematical identity looks completely different in each format.

3. **Different result types:** `ProofResult` enum (Proved/NotModular/NegativeOrder/CounterExample) is specific to eta-quotient proving. The algorithmic prover needs its own result type (recurrence found, certificate computed, etc.).

4. **User confusion:** "prove this identity" could mean either method. A unified dispatch is needed.

**Prevention:**
1. **Separate modules:** Put algorithmic proving in a new submodule (e.g., `qseries/algorithmic/` or `qseries/zeilberger/`) alongside the existing `qseries/identity/`
2. **Separate result types:** Create `AlgorithmicProofResult` distinct from `ProofResult`
3. **Unified dispatcher (later phase):** Eventually add a `prove_identity` function that:
   - Tries eta-quotient method if input is expressible as eta-quotients
   - Tries q-Zeilberger if input is q-hypergeometric
   - Falls back to FPS numerical check
4. **Do NOT modify the existing identity module** to accommodate the new one. They are parallel approaches.

**Phase to address:** Architecture phase (module organization), then implementation phases.

**Confidence:** HIGH -- direct inspection of the existing codebase confirms the integration concern.

---

### Pitfall 12: FPS Truncation Order Inadequacy for Verification

**What goes wrong:**
After finding a recurrence or WZ certificate algorithmically, the standard practice is to verify the first N terms via direct FPS computation. But what truncation order N is sufficient?

For eta-quotient identities, the Sturm bound gives an exact answer. For algorithmic proofs, the verification needs to:
1. Check the recurrence initial conditions (typically 1-3 terms)
2. Verify the certificate equation at enough points

The pitfall is using the FPS truncation_order from the existing session (which might be small, like 20) when the recurrence has high order (needing initial conditions at n=0,...,J) or the certificate has poles that only appear beyond q^20.

**Prevention:**
1. Compute the required verification depth from the recurrence order and polynomial degrees
2. Automatically increase truncation_order for verification if needed
3. Document that algorithmic proofs require higher truncation orders than simple series evaluation

**Phase to address:** Verification phase (after core algorithms are working).

**Confidence:** MEDIUM -- the issue is real but the solution is straightforward.

---

## Minor Pitfalls

---

### Pitfall 13: Bilateral Series (psi) Need Separate Algorithmic Treatment

**What goes wrong:**
The existing `eval_psi` handles bilateral series `_r psi_s` by summing positive and negative index parts separately. But q-Zeilberger for bilateral sums requires a DIFFERENT algorithm variant (not just running unilateral q-Zeilberger on each half). The bilateral q-Gosper algorithm has additional subtleties around the "Abel Lemma" approach.

**Prevention:**
Phase bilateral algorithmic proving separately from unilateral. Start with unilateral (phi) only. Most standard identities use phi.

**Confidence:** MEDIUM -- based on Chen-Hou-Mu (2005) work on the Abel Lemma and q-Gosper.

**Source:** [The Abel Lemma and the q-Gosper Algorithm](https://cfc.nankai.edu.cn/_upload/article/files/c6/e1/a2c52bf04b1896f59003b5993582/f50459f6-ce28-4bb2-96c6-9cdb38f67111.pdf)

---

### Pitfall 14: Parametric Identities vs. Concrete Identities

**What goes wrong:**
q-Zeilberger proves identities that hold for all values of a parameter (e.g., "for all n >= 0, sum_k ... = ..."). But q-Kangaroo's FPS verification works at concrete truncation orders. The gap:
- Algorithmic proof gives a recurrence valid for all n
- FPS verification checks finitely many terms
- If the recurrence has a non-hypergeometric solution for some exceptional n, the FPS check might miss it

**Prevention:**
Clearly distinguish between "recurrence proved" (universal statement) and "verified to O(q^N)" (existential check). Both together give strong evidence; either alone is incomplete.

**Confidence:** MEDIUM.

---

### Pitfall 15: Python API Ergonomics for Proofs

**What goes wrong:**
The existing Python DSL uses tuple interfaces for hypergeometric parameters:
```python
phi([(1, 0), (2, -1)], [(3, 1)], (1, 1), 20)  # upper, lower, z, truncation
```

For the algorithmic prover, users need to:
1. Specify the identity to prove (not just evaluate a series)
2. Understand what a recurrence or WZ certificate means
3. Inspect proof objects (recurrence coefficients, certificate rational function)

Exposing raw polynomial ring operations through PyO3 is complex and error-prone. The Python API needs a high-level "prove this identity" function, not low-level access to q-Gosper internals.

**Prevention:**
1. Design the Python API top-down: what do mathematicians want to type?
```python
result = s.prove_sum(phi_params, expected_closed_form)
result = s.find_recurrence(phi_params)
result = s.wz_certificate(F_func, G_func)
```
2. Return structured result objects, not raw data
3. Defer polynomial ring exposure to advanced API (or not at all)

**Phase to address:** Python API phase (after Rust core works).

**Confidence:** MEDIUM -- ergonomics are important but not blocking.

---

### Pitfall 16: Multivariate Extensions (Future-Proofing Trap)

**What goes wrong:**
Developers anticipate needing multivariate creative telescoping (for double/triple sums) and over-engineer the polynomial ring infrastructure to support multiple variables. Multivariate polynomial GCD is dramatically harder than univariate:
- Univariate GCD: O(d^2) via Euclidean, O(d log^2 d) via fast methods
- Multivariate GCD: O(d^n * ...) with complex algorithms (Zippel, EZ-GCD, etc.)

For q-Kangaroo's use case (single-sum q-hypergeometric identities), univariate polynomials in one q-shift variable suffice for all of q-Gosper, q-Zeilberger, and WZ certificates.

**Prevention:**
Build `QPolynomial` as **univariate only**. If multivariate is ever needed, it's a separate type with separate algorithms. Do not make QPolynomial generic over "number of variables" -- this adds complexity for a use case that may never materialize.

**Confidence:** HIGH -- YAGNI principle. All target algorithms are single-sum.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Polynomial ring foundation | Pitfall 1 (representation gap), Pitfall 2 (coefficient explosion) | Build QPolynomial + QRationalFunction with modular GCD from day one |
| q-Gosper algorithm | Pitfall 3 (qGFF), Pitfall 7 (q-shift confusion), Pitfall 10 (negative degree) | Follow Koornwinder 1993 precisely; test with known A=B examples |
| q-Zeilberger algorithm | Pitfall 4 (order search hang), Pitfall 8 (linear system), Pitfall 9 (non-terminating) | Max order bound, preprocessing, termination gate |
| WZ certificates | Pitfall 5 (boundary terms), Pitfall 12 (truncation) | Always verify boundaries; FPS cross-check mandatory |
| Integration with existing code | Pitfall 6 (QMonomial mismatch), Pitfall 11 (dual provers) | Separate modules, separate types, unified dispatch later |
| Python API | Pitfall 15 (ergonomics) | Top-down API design; hide polynomial internals |

---

## Summary: The Three Hardest Problems

1. **Building the polynomial ring** (Pitfalls 1, 2, 6, 16): The existing type system has no polynomial-in-q^n type. This is the biggest infrastructure gap. Get it right (with coefficient management) before writing any algorithm code.

2. **Coefficient management** (Pitfalls 2, 8): Every operation in exact arithmetic risks coefficient explosion. The Paule-Riese lesson (95% -> 30-40% with preprocessing) must be built in from the start, not added after performance problems appear.

3. **Boundary/verification correctness** (Pitfalls 5, 12, 14): Algorithmic proofs have subtle correctness conditions (boundary terms, initial conditions, termination requirements) that are easy to skip and hard to debug. Always cross-check with FPS.

---

## Sources

### Primary References
- [Koornwinder 1993: On Zeilberger's Algorithm and its q-Analogue](https://staff.fnwi.uva.nl/t.h.koornwinder/art/1993/zeilbalgo.pdf) -- rigorous algorithm description
- [Paule-Riese: qZeil implementation](https://www3.risc.jku.at/publications/download/risc_117/Paule_Riese.pdf) -- implementation lessons, preprocessing optimization
- [Koepf: Hypergeometric Summation (Springer)](https://link.springer.com/book/10.1007/978-1-4471-6464-7) -- textbook covering all algorithms
- [A=B (Petkovsek-Wilf-Zeilberger)](https://www.amazon.com/B-Marko-Petkovsek/dp/1568810636) -- foundational reference

### Algorithm-Specific
- [Greatest factorial factorization](https://www.sciencedirect.com/science/article/pii/S0747717185710498)
- [Fast polynomial dispersion](https://dl.acm.org/doi/pdf/10.1145/190347.190413)
- [Efficient rational creative telescoping](https://www.sciencedirect.com/science/article/abs/pii/S0747717121000535)
- [Creative telescoping ABC (thesis)](https://theses.hal.science/tel-01069831/document)

### Implementation References
- [hipergeo (Maxima implementation)](https://github.com/cassiopagnoncelli/hipergeo)
- [AeqB-sage (Sage implementation)](https://github.com/benyoung/AeqB-sage/blob/master/gosper.sage)
- [REDUCE zeilberger package](http://www.reduce-algebra.com/manual/manualse197.html)
- [Ore polynomials in Sage](http://www.algebra.uni-linz.ac.at/people/mkauers/publications/kauers14b.pdf)

### Coefficient Management
- [Polynomial GCD coefficient swell](https://en.wikipedia.org/wiki/Polynomial_greatest_common_divisor)
- [Modular GCD algorithms](https://users.cs.duke.edu/~elk27/bibliography/99/KaMo99.pdf)
- [Sparse multivariate GCD](https://www.cecm.sfu.ca/CAG/papers/MahsaCASC23.pdf)
