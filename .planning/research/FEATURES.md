# Feature Landscape: Algorithmic q-Hypergeometric Identity Proving

**Domain:** Symbolic q-hypergeometric summation and identity certification
**Researched:** 2026-02-15
**Overall confidence:** MEDIUM-HIGH (algorithms well-established in literature since 1990s; implementation details verified against multiple sources; some internal algorithm steps rely on training data where PDFs were unreadable)

---

## Context: What Already Exists in q-Kangaroo

Before cataloguing new features, here is what the engine already provides (built in Phases 1-8):

| Existing Capability | Module | Relevance to New Work |
|---------------------|--------|----------------------|
| q-Pochhammer (a;q)\_n evaluation | `pochhammer.rs` | Foundation for q-hypergeometric term representation |
| HypergeometricSeries / BilateralHypergeometricSeries structs | `hypergeometric.rs` | Input format for summation algorithms |
| eval\_phi / eval\_psi (FPS evaluation) | `hypergeometric.rs` | Numerical verification of identities |
| 6 summation formulas (q-Gauss, q-Vandermonde, q-Saalschutz, q-Kummer, q-Dixon, Watson) | `hypergeometric.rs` | Pattern-matching proofs; q-Zeilberger subsumes these |
| Heine/Sears/Watson/Bailey transformations | `hypergeometric.rs` | Transform to summable form; composable with new algorithms |
| QMonomial arithmetic (mul, div, try\_sqrt, is\_q\_neg\_power) | `mod.rs` | Critical for ratio analysis in q-Gosper |
| FormalPowerSeries with exact QRat arithmetic | `series/` | Verification backend; certificate checking |
| prodmake / etamake / jacprodmake | `prodmake.rs` | Product recognition after summation |
| findlincombo / findhom / findpoly / findcong | `relations.rs` | Relation discovery; complementary to algorithmic proving |
| Eta-quotient identity proving (valence formula) | `identity/prove.rs` | Modular function proofs; different domain than q-hypergeometric |
| Bailey pairs/chains/discovery | `bailey.rs` | Bailey-based identity construction; feeds into new algorithms |
| Rational null space / RREF over QRat | `linalg.rs` | Needed for polynomial system solving in Gosper/Zeilberger |

**Key insight:** The existing engine excels at *numerical verification* (expand both sides to O(q^T) and compare) and *pattern-matching proofs* (recognize specific summation formulas). What is missing is *algorithmic proving* -- automatically finding recurrences and certificates that constitute a mathematical proof without relying on pattern databases.

---

## Table Stakes

Features that any serious q-hypergeometric identity proving system must have. Missing any of these means the tool cannot compete with Garvan's Maple qseries, Paule/Riese's qZeil, or Koepf's qsum.

### 1. q-Hypergeometric Term Ratio Test

| Aspect | Detail |
|--------|--------|
| **What** | Given a sequence t(k), determine whether t(k+1)/t(k) is a rational function of q^k. If yes, extract the rational function r(q^k) = P(q^k)/Q(q^k). |
| **Why expected** | This is the entry point for ALL q-Gosper/q-Zeilberger algorithms. Every competing tool has this. Without it, you cannot even begin. |
| **Complexity** | LOW |
| **Dependencies** | QMonomial arithmetic (exists), polynomial representation in q^k (new) |
| **Input** | A q-hypergeometric term t(k) specified by its parameters (upper/lower Pochhammer products, argument, extra factors) |
| **Output** | The rational function r(x) = t(k+1)/t(k) where x = q^k, decomposed as numerator/denominator polynomials in x |
| **Notes** | Already partially present: HypergeometricSeries encodes the ratio implicitly via Pochhammer parameters. Need to make it explicit as a polynomial ratio for the algorithms. |

### 2. q-Gosper Algorithm (Indefinite q-Hypergeometric Summation)

| Aspect | Detail |
|--------|--------|
| **What** | Given a q-hypergeometric term t(k), decide whether there exists a q-hypergeometric antidifference g(k) such that t(k) = g(k) - g(k-1), and if so, find g(k). This is a COMPLETE decision procedure: if it fails, no q-hypergeometric antidifference exists. |
| **Why expected** | Fundamental building block. Every competing tool (qZeil, qsum, Koornwinder's Maple package) implements this. It is to q-Zeilberger what Gaussian elimination is to linear algebra -- you cannot do the definite case without the indefinite case. |
| **Complexity** | HIGH |
| **Dependencies** | q-hypergeometric term ratio test, univariate polynomial arithmetic in Q(q)[x], polynomial GCD, resultant computation |
| **Input** | A q-hypergeometric term t(k) (via its ratio r(q^k) = t(k+1)/t(k)) |
| **Output** | Either `Summable(g_k)` where g(k) is the antidifference, or `NotSummable` (proof that no q-hypergeometric antidifference exists) |

**Algorithm steps (from Koornwinder 1993, Paule/Riese 1997):**

1. **Compute ratio:** r(x) = t(k+1)/t(k) as rational function of x = q^k
2. **Polynomial decomposition:** Write r(x) = a(x)/b(x) in lowest terms. Find polynomials p(x), sigma(x), tau(x) such that:
   - a(x) = p(x) * sigma(x)
   - b(x) = p(q*x) * tau(x)  (note the q-shift!)
   - gcd(sigma(x), tau(q^j * x)) = 1 for all j >= 0

   This is the **q-greatest factorial factorization (qGFF)** step. In the q-case, this is actually simpler than the ordinary case: instead of finding rational roots of resultants, one checks multiplicities of factors related by q-shifts. The "q-dispersion" is found by checking when gcd(a(x), b(q^j * x)) is nontrivial for non-negative integers j.

3. **Solve for f:** Find a polynomial f(x) in x = q^k satisfying:
   sigma(x) * f(q*x) - tau(x) * f(x) = p(x)

   This is a q-difference equation for f. An upper bound on deg(f) is computable from deg(sigma), deg(tau), deg(p). Set up f as unknown polynomial with undetermined coefficients, expand, and solve the resulting linear system over Q(q).

4. **Construct antidifference:** If f exists, then g(k) = (f(q^k) / p(q^k)) * t(k).

**Confidence:** HIGH -- algorithm is well-established (Koornwinder 1993, textbook treatment in Koepf "Hypergeometric Summation" Ch. 8-9, A=B Ch. 8).

### 3. q-Zeilberger Algorithm (Definite q-Hypergeometric Summation / Creative Telescoping)

| Aspect | Detail |
|--------|--------|
| **What** | Given a bivariate q-hypergeometric term F(n,k), find a linear recurrence in n satisfied by S(n) = sum\_k F(n,k), together with a proof certificate G(n,k). |
| **Why expected** | This is THE central algorithm for proving q-hypergeometric identities. Every competing tool (qZeil, qsum, qMultiSum, HolonomicFunctions) implements this. It is the algorithm that proves identities like q-Vandermonde, q-Gauss, q-Saalschutz automatically, without needing to know them in advance. |
| **Complexity** | HIGH |
| **Dependencies** | q-Gosper algorithm (used as subroutine), bivariate polynomial arithmetic, linear system solving |
| **Input** | A bivariate q-hypergeometric summand F(n,k) and optionally a conjectured recurrence order |
| **Output** | A `RecurrenceRelation` containing: (1) polynomial coefficients c\_0(n), ..., c\_d(n) such that c\_0(n)*S(n) + c\_1(n)*S(n+1) + ... + c\_d(n)*S(n+d) = 0, and (2) a certificate rational function R(n,k) |

**Algorithm steps (creative telescoping):**

1. **For increasing order d = 1, 2, 3, ...:**
   a. Form the ansatz: c\_0(q^n)*F(n,k) + c\_1(q^n)*F(n+1,k) + ... + c\_d(q^n)*F(n+d,k) = G(n,k+1) - G(n,k) where c\_i are unknown polynomials in q^n and G(n,k) = R(n,k)*F(n,k) for unknown rational R.

   b. Since each F(n+j,k)/F(n,k) is a known rational function of q^n, q^k (it is q-hypergeometric in both variables), the ansatz reduces to: [c\_0 + c\_1*r\_1 + ... + c\_d*r\_d] * F(n,k) = G(n,k+1) - G(n,k).

   c. Apply **q-Gosper's algorithm** to the term inside brackets (viewed as a function of k with parameters involving q^n). If q-Gosper succeeds for some choice of c\_i, we have found our recurrence.

   d. The q-Gosper step produces the certificate G(n,k) if successful.

2. **Termination:** The algorithm terminates for "proper q-hypergeometric" terms (the ratio F(n,k+1)/F(n,k) and F(n+1,k)/F(n,k) are rational in q^n, q^k). Abramov and colleagues characterized exactly when termination is guaranteed.

3. **Verification:** Given the recurrence and certificate, verification is a finite rational identity check -- purely algebraic, requiring no infinite summation.

**Confidence:** HIGH -- algorithm is the q-analogue of the most well-known algorithm in computer algebra for combinatorial identities. Implementations exist in Maple (Koornwinder, Koepf), Mathematica (Paule/Riese qZeil), and REDUCE (Koepf qsum).

### 4. WZ Proof Certificate Extraction and Verification

| Aspect | Detail |
|--------|--------|
| **What** | Given an identity sum\_k F(n,k) = 1 (or a known closed form), find a rational function R(n,k) -- the WZ proof certificate -- such that G(n,k) = R(n,k)*F(n,k) satisfies the WZ equation: F(n+1,k) - F(n,k) = G(n,k+1) - G(n,k). Verification checks this equation holds as a rational function identity. |
| **Why expected** | WZ certificates are the standard format for machine-verifiable proofs of hypergeometric identities. The certificate R(n,k) is typically a small rational function even when the identity is complex. Verification is trivial: substitute and check. This is the "proof" that q-Kangaroo can output. |
| **Complexity** | MEDIUM (extraction is a byproduct of q-Zeilberger; verification is LOW) |
| **Dependencies** | q-Zeilberger algorithm (produces the certificate), rational function arithmetic |
| **Input for extraction** | A q-hypergeometric summand F(n,k) and a conjectured identity S(n) = closed\_form(n) |
| **Input for verification** | F(n,k) and R(n,k), the proposed certificate |
| **Output** | `CertificateResult::Verified` or `CertificateResult::Invalid { counterexample }` |

**Key property:** Verification is independent of how the certificate was found. A user can supply a certificate from a paper and q-Kangaroo verifies it. This separates "proof discovery" from "proof checking."

**Confidence:** HIGH -- WZ pairs are well-defined mathematical objects (Wilf-Zeilberger 1992).

### 5. Recurrence Solving for Closed Forms

| Aspect | Detail |
|--------|--------|
| **What** | Given a linear recurrence c\_0(n)*a(n) + c\_1(n)*a(n+1) + ... + c\_d(n)*a(n+d) = 0 with polynomial coefficients, find q-hypergeometric solutions (solutions where a(n+1)/a(n) is rational in q^n). |
| **Why expected** | q-Zeilberger produces a recurrence for S(n). To get a closed form, you must SOLVE that recurrence. Without this step, q-Zeilberger outputs "S(n) satisfies this recurrence" but not "S(n) = [product formula]." The q-Petkovsek algorithm does exactly this. |
| **Complexity** | HIGH |
| **Dependencies** | Polynomial factoring over Q(q), rational root finding, polynomial arithmetic |
| **Input** | A linear recurrence with polynomial coefficients in q^n |
| **Output** | All q-hypergeometric solutions, or proof that none exist |
| **Notes** | Can defer to Phase 2 of this milestone. Many proofs work without a closed form -- the recurrence + initial values suffice. But for human-readable results, closed forms are strongly desired. |

**Confidence:** MEDIUM -- q-Petkovsek's algorithm is well-described in Koepf's book but implementations are complex; the q-case uses Newton polygon improvements.

---

## Differentiators

Features that would set q-Kangaroo apart from existing tools. Not strictly required, but valuable for adoption and capability.

### 6. Nonterminating Identity Proving via Parameter Specialization

| Aspect | Detail |
|--------|--------|
| **What** | Prove nonterminating q-hypergeometric identities by the Chen-Hou-Mu method: replace a parameter x with x*q^n, apply q-Zeilberger to the resulting terminating sum, verify the recurrence matches both sides, check initial conditions. |
| **Why valuable** | q-Zeilberger directly handles only terminating sums. But most interesting identities (q-Gauss, Heine transformations, Rogers-Ramanujan) are nonterminating. This technique extends q-Zeilberger's reach dramatically. Chen et al. (2008) showed it applies to "almost all nonterminating basic hypergeometric summation formulas in Gasper-Rahman." |
| **Complexity** | MEDIUM (once q-Zeilberger exists, this is a wrapper strategy) |
| **Dependencies** | q-Zeilberger algorithm, FPS evaluation for initial condition checking |
| **Input** | A nonterminating q-hypergeometric identity (LHS = RHS) |
| **Output** | Proof verdict: recurrence matches + initial conditions verified, or failure |

**Confidence:** HIGH -- technique is published (Chen, Hou, Mu 2008; Proceedings of the Edinburgh Mathematical Society) and has been applied to dozens of classical identities.

### 7. Automatic Transformation Chain Discovery

| Aspect | Detail |
|--------|--------|
| **What** | Given two q-hypergeometric series that are equal, automatically find a sequence of known transformations (Heine I/II/III, Sears, Watson, Bailey) that converts one to the other. |
| **Why valuable** | Currently, transformations are applied one at a time manually. An automated search would let the user say "prove that phi\_A = phi\_B" and get back the chain of transformations. This is more powerful than pattern-matching individual transformations. Competing tools do NOT typically offer this -- it would be a genuine differentiator. |
| **Complexity** | MEDIUM-HIGH |
| **Dependencies** | Existing transformation functions, BFS/DFS search over transformation space |
| **Input** | Two HypergeometricSeries that should be equal |
| **Output** | A sequence of transformations, or "no chain found within search depth" |
| **Notes** | The search space is manageable: there are ~8 transformations, and chains rarely exceed depth 3-4. Can prune by checking FPS agreement at each step. |

### 8. Batch Identity Verification with Certificates

| Aspect | Detail |
|--------|--------|
| **What** | Given a database of q-hypergeometric identities (e.g., all identities in Gasper-Rahman Appendix II), systematically prove each one and output certificates. |
| **Why valuable** | Demonstrates the power of the proving engine. Creates a verified identity database. Competing tools have example notebooks with ~500 identities (qZeil) but no systematic verified database available as data. This would be a research contribution, not just a software feature. |
| **Complexity** | MEDIUM (once proving algorithms exist, this is engineering) |
| **Dependencies** | q-Zeilberger, WZ certificates, identity representation format |
| **Input** | Collection of identities in structured format |
| **Output** | For each: proof status, certificate, recurrence, timing |

### 9. Multi-Sum Creative Telescoping (qMultiSum)

| Aspect | Detail |
|--------|--------|
| **What** | Extend q-Zeilberger to handle multiple nested sums: sum\_{k1} sum\_{k2} ... F(n, k1, k2, ...). Find a recurrence in n by telescoping over all summation variables. |
| **Why valuable** | Many partition-theoretic identities involve multiple sums. Riese's qMultiSum package (Mathematica) handles this but is not widely available outside Mathematica. Having this in an open-source Rust engine would be significant. |
| **Complexity** | VERY HIGH |
| **Dependencies** | q-Zeilberger (single sum case), multivariate polynomial arithmetic |
| **Input** | Multivariate q-hypergeometric summand F(n, k1, ..., km) |
| **Output** | Recurrence in n with certificate functions |
| **Notes** | Defer to future milestone. The single-sum case covers the vast majority of use cases. |

### 10. Human-Readable Proof Output

| Aspect | Detail |
|--------|--------|
| **What** | Generate a step-by-step proof narrative: "Step 1: Compute ratio t(k+1)/t(k) = ... Step 2: qGFF decomposition gives sigma=..., tau=..., p=... Step 3: Solving for f gives f(x) = ... Step 4: Certificate R(n,k) = ... Step 5: Verification: LHS - RHS = 0." |
| **Why valuable** | Researchers need to understand and cite proofs, not just get a yes/no answer. A proof they can include in a paper (or at least reference) is much more valuable than a boolean. No competing tool generates publication-ready proof narratives -- they output certificates and recurrences but leave the human to assemble the proof. |
| **Complexity** | MEDIUM |
| **Dependencies** | q-Gosper, q-Zeilberger, LaTeX rendering (exists) |
| **Input** | Any identity that was successfully proved |
| **Output** | Structured proof with LaTeX-renderable steps |

---

## Anti-Features

Features to explicitly NOT build, despite being tempting or requested.

### A1. General Holonomic Functions Framework

| Aspect | Detail |
|--------|--------|
| **What** | Koutschan's HolonomicFunctions approach: represent functions as annihilating ideals in Ore algebras, then use closure properties and creative telescoping over arbitrary D-finite / q-D-finite functions. |
| **Why avoid** | Massively increases scope and complexity. The holonomic approach is more general but q-Kangaroo's domain is specifically q-hypergeometric series, where specialized algorithms (q-Gosper, q-Zeilberger) are faster and simpler. HolonomicFunctions took Koutschan's entire PhD thesis. The overhead of Ore algebras, Groebner bases in non-commutative rings, etc. is not justified for our use case. |
| **What instead** | Implement q-Gosper and q-Zeilberger directly for q-hypergeometric terms. These cover the vast majority of q-series identities in practice. If a user needs holonomic functions, they should use Mathematica. |

### A2. Symbolic Integration / q-Integration

| Aspect | Detail |
|--------|--------|
| **What** | q-analogue of the Risch algorithm for indefinite q-integration (Jackson q-integral). |
| **Why avoid** | Different algorithmic domain. q-Integration is related but separate from q-summation. The user's stated goal is identity PROVING, not integration. Adding q-integration would double the scope without serving the core use case. |
| **What instead** | Focus on summation algorithms. If q-integrals arise, they can often be converted to sums. |

### A3. Numerical / Floating-Point Verification as Proof

| Aspect | Detail |
|--------|--------|
| **What** | Verify identities by evaluating both sides at many numerical points and declaring "proved" if they agree. |
| **Why avoid** | This is NOT a proof. It is a heuristic that can miss edge cases, and it already exists in q-Kangaroo (FPS comparison to O(q^T)). The whole point of algorithmic proving is to produce MATHEMATICAL PROOFS (certificates, recurrences) that are valid for all n, not just checked values. |
| **What instead** | Use FPS comparison for quick sanity checks and counterexample detection, but require algebraic certificates for actual proofs. The existing `eval_phi` + FPS comparison already serves the numerical role. |

### A4. Computer Algebra System (CAS) Generality

| Aspect | Detail |
|--------|--------|
| **What** | Build a general-purpose CAS with pattern matching, rule rewriting, simplification of arbitrary expressions. |
| **Why avoid** | q-Kangaroo is a domain-specific engine, not a CAS. Adding general symbolic manipulation would require years of development (simplification, pattern matching, calculus, etc.) and compete with SymPy/Mathematica/Maple on their home turf. |
| **What instead** | Keep the domain-specific approach: q-hypergeometric terms are represented as structured data (HypergeometricSeries), not general symbolic expressions. Algorithms operate on this structured representation. |

### A5. Automatic Conjecture Generation from Data

| Aspect | Detail |
|--------|--------|
| **What** | Given a sequence of integers, automatically conjecture a q-hypergeometric closed form (like the OEIS but algorithmic). |
| **Why avoid** | Different problem domain (sequence recognition vs. identity proving). Would require integer relation detection (PSLQ/LLL), extensive database of known sequences, etc. The existing `findlincombo`/`findhom`/`findpoly` already handle the "find relations among given series" case. Going further into fully automatic conjecture is a research project, not an engineering task. |
| **What instead** | Strengthen the existing relation discovery tools. Users provide candidate series; q-Kangaroo finds relations among them. |

---

## Feature Dependencies

```
[q-Hypergeometric Term Ratio Test]  (Feature 1)
    |
    v
[Univariate Polynomial Arithmetic in Q[x]]  (new infrastructure)
    |-- polynomial GCD
    |-- polynomial resultant
    |-- polynomial evaluation / interpolation
    |-- q-shift operations on polynomials
    |
    v
[q-Gosper Algorithm]  (Feature 2)
    |-- uses: qGFF (q-Greatest Factorial Factorization)
    |-- uses: polynomial linear system solving
    |-- uses: degree bound computation
    |
    +-------+
    |       |
    v       v
[q-Zeilberger Algorithm]  (Feature 3)     [WZ Certificate Verification]  (Feature 4)
    |-- uses: q-Gosper as subroutine           |-- uses: rational function arithmetic
    |-- uses: bivariate ratio analysis          |-- independent of how certificate was found
    |-- outputs: recurrence + certificate
    |
    +---+---+
    |       |
    v       v
[Recurrence Solving]     [Nonterminating Proofs]  (Feature 6)
(Feature 5)                  |-- uses: q-Zeilberger
    |-- q-Petkovsek          |-- uses: parameter specialization
    |-- finds closed forms   |-- uses: FPS initial condition check
    |
    v
[Transformation Chain Discovery]  (Feature 7)
    |-- uses: existing transformations
    |-- uses: FPS comparison for pruning
    |-- optional, can be independent

[Batch Verification]  (Feature 8)
    |-- uses: q-Zeilberger + WZ certificates
    |-- primarily engineering, not algorithmic

[Human-Readable Proof Output]  (Feature 10)
    |-- uses: all proving algorithms
    |-- uses: LaTeX rendering (exists)
```

### Critical Path

The critical dependency chain is:

1. **Polynomial infrastructure** (GCD, resultant, q-shift) -- everything depends on this
2. **q-Gosper** -- must work before q-Zeilberger can be attempted
3. **q-Zeilberger** -- the main deliverable; enables features 4, 6, 8, 10
4. **WZ verification** -- relatively easy once q-Zeilberger produces certificates

Features 5, 6, 7, 8, 10 are independent of each other and can be built in parallel once the q-Zeilberger core exists.

---

## Competitor Analysis

### Garvan's Maple qseries Package

| Capability | Status | How q-Kangaroo Compares |
|------------|--------|------------------------|
| prodmake (series-to-product) | In Garvan | ALREADY IN q-Kangaroo |
| etamake / jacprodmake | In Garvan | ALREADY IN q-Kangaroo |
| findhom / findlincombo / findpoly | In Garvan | ALREADY IN q-Kangaroo |
| sift (subsequence extraction) | In Garvan | ALREADY IN q-Kangaroo |
| qfactor (polynomial factoring) | In Garvan | ALREADY IN q-Kangaroo |
| Eta-quotient identity proving (valence formula) | In thetaids/ETA packages | ALREADY IN q-Kangaroo |
| q-Gosper / q-Zeilberger | **NOT in Garvan's packages** | NEW -- this is what we are building |
| Bailey chain construction | In various papers | ALREADY IN q-Kangaroo |

**Key insight:** Garvan's packages do NOT include q-Gosper or q-Zeilberger. His approach is modular functions (valence formula) and relation discovery (linear algebra over coefficient matrices), not algorithmic hypergeometric summation. q-Kangaroo already replicates Garvan's capabilities. The new algorithmic proving features would EXCEED Garvan's toolkit.

### Paule/Riese qZeil (Mathematica)

| Capability | Status | How q-Kangaroo Compares |
|------------|--------|------------------------|
| q-Gosper (indefinite summation) | Core feature | TO BE BUILT (Feature 2) |
| q-Zeilberger (definite summation) | Core feature | TO BE BUILT (Feature 3) |
| WZ certificate output | Included | TO BE BUILT (Feature 4) |
| ~500 worked examples | In notebooks | TO BE BUILT as test suite (Feature 8) |
| Closed-form Mathematica output | Included | Rust structs instead (different UX) |
| Multi-sum (qMultiSum) | Separate package by Riese | DEFERRED (Feature 9, future milestone) |
| Nonterminating identities | Limited | PLANNED (Feature 6, via Chen method) |

**Key differentiator vs qZeil:** qZeil is Mathematica-only, proprietary, and its code is not openly maintained. q-Kangaroo would be open-source, callable from Python, and significantly faster (Rust vs. interpreted Mathematica). The qGFF approach from Paule-Riese is well-documented and the algorithm is public.

### Koepf's qsum (Maple)

| Capability | Status | How q-Kangaroo Compares |
|------------|--------|------------------------|
| q-Gosper | Implemented (by Boing) | TO BE BUILT |
| q-Zeilberger | Implemented | TO BE BUILT |
| q-Petkovsek (recurrence solving) | Implemented | TO BE BUILT (Feature 5) |
| q-van-Hoeij (improved recurrence solving) | In qFPS.mpl | DEFERRED |
| Maple worksheet integration | Native | Python/Jupyter integration instead |

**Key differentiator vs qsum:** qsum requires Maple (commercial). The algorithms are well-documented in Koepf's textbook "Hypergeometric Summation" (2014, 2nd ed.), making implementation feasible.

### Koutschan's HolonomicFunctions (Mathematica)

| Capability | Status | How q-Kangaroo Compares |
|------------|--------|------------------------|
| General holonomic creative telescoping | Core feature | ANTI-FEATURE (too general for our scope) |
| Closure properties (sum, product, etc.) | Core feature | Not needed for q-hypergeometric |
| q-difference equations | Supported | We handle q-hypergeometric terms directly |
| Ore algebra framework | Foundation | NOT BUILDING (too complex, not needed) |

**Key insight:** HolonomicFunctions is more powerful but more complex. For pure q-hypergeometric work, q-Gosper + q-Zeilberger is sufficient and faster.

### Schneider's Sigma (Mathematica)

| Capability | Status | How q-Kangaroo Compares |
|------------|--------|------------------------|
| Difference field theory approach | Core feature | Different paradigm; we use direct algorithms |
| Nested multi-sum simplification | Core feature | DEFERRED (Feature 9) |
| Optimal nested depth | Unique capability | Out of scope |

**Key insight:** Sigma uses a fundamentally different approach (difference rings/fields). It is more general for nested sums but not specifically optimized for q-hypergeometric terms. Not a direct competitor for our use case.

---

## MVP Recommendation

### Phase 1: Polynomial Infrastructure + q-Gosper

Build the foundation and the first complete algorithm.

**Prioritize:**
1. Univariate polynomial arithmetic over Q(q)[x] -- GCD, resultant, degree bounds
2. q-shift operations on polynomials (evaluate p(q*x), p(q^j*x))
3. q-dispersion computation (find all j where gcd(a(x), b(q^j*x)) != 1)
4. qGFF decomposition (sigma, tau, p)
5. q-Gosper algorithm (polynomial f solver + antidifference construction)
6. q-hypergeometric term ratio extraction from HypergeometricSeries

**Deliverable:** `q_gosper(term) -> GosperResult::Summable(antidifference) | NotSummable`

### Phase 2: q-Zeilberger + WZ Certificates

The main proving engine.

**Prioritize:**
1. Bivariate q-hypergeometric term representation
2. Creative telescoping loop (try orders d = 1, 2, 3, ...)
3. Certificate extraction and output
4. WZ certificate verification (independent checker)
5. Recurrence output format with polynomial coefficients

**Deliverable:** `q_zeilberger(summand) -> ZeilbergerResult { recurrence, certificate }`

### Phase 3: Closed Forms + Nonterminating Proofs

Complete the proving pipeline.

**Prioritize:**
1. q-Petkovsek algorithm for recurrence solving
2. Nonterminating identity proving (parameter specialization method)
3. Integration with existing summation formulas (verify that q-Zeilberger rediscovers q-Gauss etc.)
4. Transformation chain discovery (search over Heine/Sears/Watson/Bailey)

**Defer:**
- Multi-sum creative telescoping (Feature 9): Scope too large for this milestone
- Human-readable proof output (Feature 10): Can be added incrementally
- Batch verification database (Feature 8): Engineering task, not algorithmic

---

## Input/Output Specifications (Expected User-Facing Behavior)

### q-Gosper

```
Input:  q_gosper(term: QHypergeometricTerm) -> GosperResult
        where QHypergeometricTerm encodes t(k) with ratio t(k+1)/t(k) = rational(q^k)

Output: GosperResult::Summable {
            antidifference: QHypergeometricTerm,  // g(k) such that t(k) = g(k) - g(k-1)
            certificate: RationalFunction,         // f(x)/p(x) where x = q^k
        }
        GosperResult::NotSummable                  // proved: no q-hyp antidifference exists

Example: q_gosper(q^{k^2} * (q;q)_k / (q^2;q^2)_k)
         -> Summable { ... } or NotSummable
```

### q-Zeilberger

```
Input:  q_zeilberger(
            summand: BivarQHypTerm,     // F(n,k) q-hypergeometric in both n and k
            max_order: Option<usize>,   // maximum recurrence order to try (default: 5)
        ) -> ZeilbergerResult

Output: ZeilbergerResult::Found {
            recurrence: QRecurrence {
                coefficients: Vec<Polynomial>,  // c_0(q^n), ..., c_d(q^n)
                order: usize,                   // d
            },
            certificate: RationalFunction,       // R(n,k) such that G = R*F
        }
        ZeilbergerResult::NotFound {
            max_order_tried: usize,
        }

Example: q_zeilberger(F(n,k) = q^{k^2} * [n choose k]_q)
         -> Found { recurrence: S(n+1) = (1+q^{n+1})*S(n), certificate: R = ... }
```

### WZ Verification

```
Input:  verify_wz(
            summand: BivarQHypTerm,         // F(n,k)
            certificate: RationalFunction,   // R(n,k)
        ) -> WZVerification

Output: WZVerification::Valid              // F(n+1,k) - F(n,k) = G(n,k+1) - G(n,k) holds
        WZVerification::Invalid {
            failure_description: String,
        }
```

### Python API (expected)

```python
from q_kangaroo import QSession

s = QSession()

# q-Gosper: indefinite summation
result = s.q_gosper(numerator_params=[(1, 0)], denominator_params=[(1, 1)], argument=(1, 1))
# Returns: { "summable": True, "antidifference": ..., "certificate": ... }
# or:      { "summable": False }

# q-Zeilberger: prove a definite sum identity
result = s.q_zeilberger(
    summand_params={ "n_upper": [...], "n_lower": [...], "k_upper": [...], "k_lower": [...] },
    max_order=5
)
# Returns: { "found": True, "recurrence_coefficients": [...], "certificate": "..." }

# WZ verification
valid = s.verify_wz_certificate(summand_params={...}, certificate="...")
# Returns: True or False
```

---

## Sources

**Algorithm Foundations:**
- [Koornwinder, "On Zeilberger's algorithm and its q-analogue" (1993)](https://staff.fnwi.uva.nl/t.h.koornwinder/art/1993/zeilbalgo.pdf) -- rigorous description of q-Gosper and q-Zeilberger
- [Paule & Riese, "A Mathematica q-Analogue of Zeilberger's Algorithm" (1997)](https://www3.risc.jku.at/publications/download/risc_117/Paule_Riese.pdf) -- qGFF concept, qZeil implementation
- [Koepf, "Hypergeometric Summation" (2nd ed., 2014)](https://link.springer.com/book/10.1007/978-1-4471-6464-7) -- textbook treatment of all algorithms including q-analogues
- [Petkovsek, Wilf, Zeilberger, "A=B" (1996)](https://sites.math.rutgers.edu/~zeilberg/AeqB.pdf) -- foundational reference for Gosper, Zeilberger, WZ theory
- [Wilf & Zeilberger, "An algorithmic proof theory for hypergeometric (ordinary and q) multisum/integral identities" (1992)](https://link.springer.com/article/10.1007/BF02100618) -- WZ proof certificates for q-case

**Nonterminating Extensions:**
- [Chen, Hou, Mu, "Nonterminating Basic Hypergeometric Series and the q-Zeilberger Algorithm" (2008)](https://arxiv.org/abs/math/0509281) -- parameter specialization technique

**Competing Implementations:**
- [qZeil package (Paule/Riese, Mathematica)](https://www3.risc.jku.at/research/combinat/software/ergosum/RISC/qZeil.html) -- ~500 examples
- [qMultiSum package (Riese, Mathematica)](http://www3.risc.jku.at/research/combinat/software/ergosum/RISC/qMultiSum.html) -- multi-sum extension
- [qsum package (Koepf/Boing, Maple)](https://www.hypergeometric-summation.org/) -- Maple implementation
- [HolonomicFunctions (Koutschan, Mathematica)](http://www3.risc.jku.at/research/combinat/software/ergosum/RISC/HolonomicFunctions.html) -- general holonomic approach
- [Sigma package (Schneider, Mathematica)](https://www3.risc.jku.at/research/combinat/software/Sigma/) -- difference field approach
- [Garvan qseries/thetaids/ETA (Maple)](https://qseries.org/fgarvan/qmaple/qseries/) -- relation discovery, not algorithmic summation

**WZ Theory:**
- [AMS Notices, "What is a Wilf-Zeilberger Pair?" (Tefera, 2010)](https://www.ams.org/notices/201004/rtx100400508p.pdf) -- accessible introduction
- [Gosper's Algorithm (Wikipedia-equivalent)](https://handwiki.org/wiki/Gosper's_algorithm) -- polynomial p, q, r decomposition steps

**Termination and Complexity:**
- [Abramov et al., "Applicability of the q-analogue of Zeilberger's algorithm" (2004)](https://ui.adsabs.harvard.edu/abs/arXiv:math%2F0410222) -- when q-Zeilberger terminates
- [Chen et al., "A Unification of Zeilberger's Algorithm and Its q-Analogue" (2025)](https://arxiv.org/abs/2501.03837) -- recent unified treatment

---
*Feature research for: Algorithmic q-hypergeometric identity proving*
*Researched: 2026-02-15*
*Confidence: MEDIUM-HIGH (algorithms well-established in literature; specific implementation details cross-checked against multiple academic sources; some PDF sources unreadable, compensated by multiple corroborating web sources)*
