# Feature Research: Q-Symbolic (q-Series Symbolic Computation)

**Domain:** Symbolic mathematics -- q-series, basic hypergeometric functions, partition theory, modular forms
**Researched:** 2026-02-13
**Confidence:** MEDIUM-HIGH (Garvan function inventory verified against official qseries.org; competitor analysis from official docs; extension features from academic literature)

---

## Feature Landscape

### Table Stakes: Garvan Parity (Must Have)

These features replicate Frank Garvan's Maple packages. Without full parity, researchers cannot migrate from Maple. The q-series research community will not adopt a tool that drops capabilities they already have.

---

#### TS-1: Core qseries Package (41 functions)

The qseries package v1.3 is the foundation. Every function below must be replicated.

##### TS-1.1: q-Pochhammer / q-Product Primitives

| Function | What It Does | Complexity | Dependencies |
|----------|-------------|------------|--------------|
| `aqprod(a,q,n)` | Computes finite q-Pochhammer symbol (a;q)_n = (1-a)(1-aq)...(1-aq^{n-1}) | LOW | None -- foundational primitive |
| `qbin(n,k,q)` | q-binomial coefficient (Gaussian polynomial) [n choose k]_q | LOW | `aqprod` |
| `etaq(g,k,T)` | Dedekind eta product expansion to O(q^T) | MEDIUM | Series truncation engine |
| `jacprod(a,b,q,T)` | Jacobi-type infinite product JAC(a,b) expansion to O(q^T) | MEDIUM | Series truncation engine |
| `tripleprod(z,q,T)` | q-series expansion of Jacobi triple product identity | MEDIUM | Series engine |
| `quinprod(z,q,T)` | Watson's quintuple product identity expansion | MEDIUM | Series engine |
| `winquist(z,q,T)` | Winquist's product identity expansion | MEDIUM | Series engine |

**Confidence:** HIGH -- function names and signatures verified from qseries.org/fgarvan/qmaple/qseries/functions/

##### TS-1.2: Series-to-Product Conversion (Core Differentiator of Garvan's Package)

This is the central capability that makes Garvan's package valuable: algorithmically converting between q-series and q-product representations.

| Function | What It Does | Complexity | Dependencies |
|----------|-------------|------------|--------------|
| `prodmake(f,q,T)` | Convert q-series into infinite product form (Andrews' algorithm) | HIGH | Series engine, pattern matching |
| `etamake(f,q,T)` | Convert q-series into eta-product form | HIGH | `prodmake`, eta function knowledge |
| `jacprodmake(f,q,T)` | Convert q-series into theta function product JAC(a,b) form | HIGH | `prodmake`, theta function knowledge |
| `jacprodmake(f,q,T,P)` | Variant: restrict search to theta-products where b divides P | HIGH | `jacprodmake` |
| `qetamake(f,q,T)` | Variant of etamake for q-eta function output | HIGH | `etamake` |
| `mprodmake(f,q,T)` | Convert q-series into product of form (1+q^n1)*(1+q^n2)*... | HIGH | `prodmake` |
| `jac2prod(expr)` | Convert theta function products back to q-product form | MEDIUM | JAC representation |
| `jac2series(expr,q,T)` | Convert theta function products to q-series expansion | MEDIUM | JAC representation, series engine |
| `qfactor(f,q)` | Factor rational function into finite q-product (1-q^i) terms | HIGH | Polynomial factoring over q |
| `zqfactor(f,z,q)` | Extended factorization for (z,q)-series into (z,q)-products | HIGH | `qfactor`, bivariate algebra |

**Confidence:** HIGH -- verified from official function list and jacprodmake documentation page

##### TS-1.3: Algebraic Relation Discovery

These functions find relationships between q-series -- critical for research exploration.

| Function | What It Does | Complexity | Dependencies |
|----------|-------------|------------|--------------|
| `findlincombo(f,list,q,T)` | Express q-series f as linear combination of given list | MEDIUM | Linear algebra over series coefficients |
| `findlincombomodp(f,list,q,T,p)` | Linear combination modulo prime p | MEDIUM | `findlincombo`, modular arithmetic |
| `findhom(list,q,T,n)` | Find potential homogeneous relations of degree n among q-series | HIGH | Linear algebra, polynomial enumeration |
| `findhomcombo(f,list,q,T,n)` | Express f as homogeneous polynomial of degree n in given series | HIGH | `findhom` |
| `findhomcombomodp(f,list,q,T,n,p)` | Homogeneous combination modulo p | HIGH | `findhomcombo`, modular arithmetic |
| `findhommodp(list,q,T,n,p)` | Homogeneous relations modulo p | HIGH | `findhom`, modular arithmetic |
| `findnonhom(list,q,T,n)` | Find non-homogeneous polynomial relations | HIGH | `findhom` variant |
| `findnonhomcombo(f,list,q,T,n)` | Express f as non-homogeneous polynomial of degree n | HIGH | `findnonhom` |
| `findpoly(f,g,q,T,n)` | Find polynomial relation between two q-series | MEDIUM | Polynomial search |
| `findprod(list,q,T)` | Find products that are linear combinations of given q-series | HIGH | `findlincombo`, product enumeration |
| `findmaxind(list,q,T)` | Find maximal independent subset of q-series list | MEDIUM | Linear algebra |
| `findcong(f,q,T,m)` | Find linear congruences in q-series coefficients | MEDIUM | Coefficient extraction, modular arithmetic |

**Confidence:** HIGH -- verified from official function list

##### TS-1.4: Series Utilities

| Function | What It Does | Complexity | Dependencies |
|----------|-------------|------------|--------------|
| `sift(f,q,m,r,T)` | Extract terms where exponent is congruent to r mod m | LOW | Series engine |
| `qdegree(f,q)` | Compute degree in q of a q-polynomial | LOW | Series engine |
| `lqdegree(f,q)` | Lowest degree in q of a q-polynomial | LOW | Series engine |
| `lqdegree0(f,q)` | Lowest degree in q of a q-monomial | LOW | Series engine |
| `qs2jaccombo(f,q,T)` | Convert sum of q-series to sum of Jacobi products | HIGH | `jacprodmake`, relation finding |
| `checkprod(f,q,T)` | Check whether q-series is a "nice" product | MEDIUM | `prodmake` |
| `checkmult(f,q,T)` | Check whether coefficients of q-series are multiplicative | MEDIUM | Coefficient extraction |

**Confidence:** HIGH -- verified from official function list

##### TS-1.5: Theta Functions

| Function | What It Does | Complexity | Dependencies |
|----------|-------------|------------|--------------|
| `theta(z,q,T)` | General Jacobi theta function theta(z,q) | MEDIUM | Series engine |
| `theta2(q,T)` | Jacobi theta function theta_2(q) | MEDIUM | Series engine |
| `theta3(q,T)` | Jacobi theta function theta_3(q) | MEDIUM | Series engine |
| `theta4(q,T)` | Jacobi theta function theta_4(q) | MEDIUM | Series engine |

**Confidence:** HIGH -- verified from official function list

---

#### TS-2: thetaids Package (~55 functions)

The thetaids package v1.0 proves theta-function identities automatically using the valence formula for modular functions. This is a key research tool.

##### TS-2.1: Identity Proving Core

| Function | What It Does | Complexity | Dependencies |
|----------|-------------|------------|--------------|
| `provemodfuncid(identity,N)` | Automatically prove a theta-function identity using valence formula | VERY HIGH | Cusp computation, order computation, series verification |
| `provemodfuncidBATCH(list)` | Batch-prove multiple theta-function identities | HIGH | `provemodfuncid` |
| `processjacid(identity)` | Process a Jacobi product identity for verification | HIGH | JAC representation, cusp analysis |
| `Gamma1ModFunc(expr,N)` | Verify if generalized eta-quotient is modular function on Gamma_1(N) | HIGH | Modular form theory |

**Confidence:** HIGH -- verified from thetaids functions-v1p0.html

##### TS-2.2: Eta/Theta Notation Conversion

| Function | What It Does | Complexity | Dependencies |
|----------|-------------|------------|--------------|
| `eeta(n)` | Symbolic eta(n*tau) expression | LOW | Symbolic representation |
| `egeta(n,g)` | Symbolic generalized eta_[n,g](tau) expression | LOW | Symbolic representation |
| `eta2geta(expr)` | Convert ETA notation to generalized eta notation | MEDIUM | Notation engine |
| `eta2jac(expr)` | Convert ETA notation to JAC notation | MEDIUM | Notation engine |
| `_etan(n)` | Symbolic eta(n*tau) | LOW | Symbolic representation |
| `_ETAn(n)` | Express eta(n*tau) in JAC terms | MEDIUM | JAC conversion |
| `_etanm(n,m)` | Symbolic generalized eta_[n,m](tau) | LOW | Symbolic representation |
| `_ETAnm(n,m)` | Express generalized eta in JAC terms | MEDIUM | JAC conversion |
| `jac2eprod(expr)` | Transform theta-function quotients to eta-products | MEDIUM | Product conversion |
| `jac2getaprod(expr)` | Transform theta quotients using generalized eta notation | MEDIUM | `jac2eprod` |
| `jac2getacombo(expr)` | Convert linear combo of JAC products to eta-product combos | MEDIUM | `jac2getaprod` |
| `mixedjac2jac(expr)` | Convert quotient sums to common base | MEDIUM | Base normalization |
| `ejac(expr)` | Support function for jac2eprod conversion | LOW | Internal |
| `getalist2jacprod(list)` | Convert geta-list to jacprod format | LOW | Format conversion |
| `GETAP2getalist(expr)` | Transform generalized eta-product quotients to geta-lists | LOW | Format conversion |
| `gterm2jacprod(list)` | Transform [b,a,c] lists to jacprod form | LOW | Format conversion |
| `jacnormalid(expr)` | Normalize jacprod sums by dividing by lowest q-power term | LOW | Normalization |
| `JACP2jaclist(expr)` | Convert jacprod quotients into jac-lists | LOW | Format conversion |

**Confidence:** HIGH -- verified from thetaids functions page

##### TS-2.3: Cusp and Order Computation

| Function | What It Does | Complexity | Dependencies |
|----------|-------------|------------|--------------|
| `cuspmake1(N)` | Compute set of inequivalent cusps for Gamma_1(N) | HIGH | Modular group theory |
| `CUSPSANDWIDMAKE1(N)` | List cusps with their widths for Gamma_1(N) | HIGH | `cuspmake1` |
| `cuspwid1(cusp,N)` | Compute cusp width in Gamma_1(N) | MEDIUM | Modular group theory |
| `cuspequiv1(c1,c2,N)` | Test Gamma_1(N)-equivalence of two cusps | MEDIUM | Modular group theory |
| `cuspsetinequiv1(set,N)` | Test inequivalence among set of cusps | MEDIUM | `cuspequiv1` |
| `numcuspequiv1(N)` | Count inequivalent cusps for Gamma_1(N) | MEDIUM | `cuspmake1` |
| `getacuspord(expr,cusp)` | Determine order of generalized eta-product at a cusp | HIGH | Order theory |
| `getaprodcuspORDS(expr,N)` | Calculate orders at each cusp for generalized eta-products | HIGH | `getacuspord` |
| `getaprodcuspord(expr,cusp)` | Compute invariant order at cusps | HIGH | `getacuspord` |
| `mintotORDS(expr)` | Establish lower bounds for order sums | MEDIUM | Order computation |
| `Bord(expr,cusp)` | Calculate theta-function order at cusps | HIGH | Order theory |
| `v0(expr)` | Calculate 2*ORD(G,0) | MEDIUM | Order computation |
| `vinf(expr)` | Calculate 2*ORD(G,infinity) | MEDIUM | Order computation |
| `printJACIDORDStable(identity)` | Display order tables for jacprod identities | LOW | `getaprodcuspORDS` |

**Confidence:** HIGH -- verified from thetaids functions page

##### TS-2.4: Utility Functions

| Function | What It Does | Complexity | Dependencies |
|----------|-------------|------------|--------------|
| `jacbase(expr)` | Determine q-base of a jacprod | LOW | JAC representation |
| `jcombobase(expr)` | Find q-base of jacprod sums | LOW | JAC representation |
| `qjacdegree(expr)` | Determine q-degree of jac terms | LOW | JAC representation |
| `lqdegree(expr)` / `lqdegree0(expr)` | Lowest q-degree variants | LOW | Series engine |
| `rmcofjac(expr)` | Remove constant coefficients from jac terms | LOW | JAC manipulation |
| `rmcofnotqjac(expr)` | Remove non-constant q-coefficients | LOW | JAC manipulation |
| `Acmake(c)` | Compute set A_c | MEDIUM | Set theory |
| `Scmake(c)` | Compute set S_c | MEDIUM | Set theory |
| `DivCheck(expr)` | Verify specific divisor conditions | MEDIUM | Divisibility |
| `phiset(N)` | Generate integers relatively prime to N | LOW | Number theory |
| `newxy(x,y,N)` | Calculate new x,y modulo N | LOW | Modular arithmetic |
| `QP2(x)` | Evaluate {x}^2 - {x} + 1/6 (Bernoulli-related) | LOW | Arithmetic |
| `fpart(x)` | Extract fractional part | LOW | Arithmetic |
| `JACCOF` | Constant value 1 | LOW | None |

**Confidence:** HIGH -- verified from thetaids functions page

---

#### TS-3: ETA Package (v0.3b)

The ETA package proves eta-product identities using the valence formula. This is critical for researchers working with Dedekind eta functions and modular forms.

| Function Category | What It Provides | Complexity | Dependencies |
|-------------------|-----------------|------------|--------------|
| Eta-product identity framework | Rewrite identities in terms of generalized eta-functions | HIGH | Symbolic eta representation |
| Modular function verification | Check that each term is a modular function on Gamma_1(N) | VERY HIGH | Modular group theory, cusp theory |
| Valence formula application | Use valence formula to determine verification depth (power of q needed) | HIGH | Cusp orders, modular forms theory |
| Identity verification engine | Prove identity by carrying out series verification to required depth | HIGH | Series engine, coefficient comparison |
| Order computation at cusps | Functions for computing orders and invariant orders at cusps | HIGH | Cusp theory |

**Note:** The ETA package PDF manual was not parseable. The function list is inferred from the tutorial abstract (arXiv:1907.09130) and cross-reference with thetaids (which shares the eta-function infrastructure). Some overlap with thetaids cusp functions is expected. Confidence: MEDIUM -- abstract verified, individual functions LOW.

---

#### TS-4: Satellite Packages

These additional Garvan packages extend the core for specific research domains.

| Package | Version | Core Functions | Complexity | Dependencies |
|---------|---------|---------------|------------|--------------|
| **Rank** | (2020) | `N(m,n)` -- number of partitions of n with rank m; `N(r,m,n)` -- rank congruent to r mod m | MEDIUM | Partition enumeration, qseries |
| **Crank** | (2020) | `M(m,n)` -- number of partitions of n with crank m; `M(r,m,n)` -- crank congruent to r mod m | MEDIUM | Partition enumeration, qseries |
| **SPT-Crank** | (2012) | `NS(m,n)` -- weighted vector partition count with crank m; `NS(r,m,n)` -- spt-crank congruent to r mod m | HIGH | Partition enumeration, rank/crank theory |
| **T-Core** | v0.2 (2023) | t-core of partition; t-quotient of partition; t-core crank | MEDIUM | Partition representation |
| **RamaRobinsIDs** | v0.2 (2018) | Find and prove theta-function identities of Ramanujan-Robins type | VERY HIGH | qseries + thetaids |
| **MODFORMS** | v0.2 | Computing with quasi-modular forms of level 1 | HIGH | Modular forms theory |

**Confidence:** HIGH for existence and purpose, MEDIUM for internal function details (pages confirm capabilities but don't list all internal functions)

---

#### TS-5: Implicit Capabilities (Not Named Functions, But Required Infrastructure)

These are not named Garvan functions but are capabilities the q-series research workflow requires.

| Capability | Why Required | Complexity | Notes |
|------------|-------------|------------|-------|
| Truncated power series arithmetic | Every computation truncates at O(q^T) | MEDIUM | Foundation of everything; add, multiply, divide series to given order |
| Exact rational arithmetic | Coefficients are often rational numbers, must be exact | MEDIUM | BigInt + BigRational required |
| Symbolic expression representation | JAC(a,b), ETA, generalized eta, theta must be symbolic objects | HIGH | Core data model |
| LaTeX output | Researchers need to paste results into papers | LOW | Formatting layer |
| Coefficient extraction | Get n-th coefficient of a q-series | LOW | Fundamental operation |
| Series substitution | Substitute q -> q^k, compose series | MEDIUM | Required for sifting, base changes |

**Confidence:** HIGH -- these are implied by every tutorial and usage pattern

---

### Differentiators: Extensions Beyond Garvan (Competitive Advantage)

These features do not exist in Garvan's packages but would make Q-Symbolic the go-to tool for modern q-series research.

---

#### D-1: Basic Hypergeometric Series Engine (_r phi_s and _r psi_s)

Garvan's package does NOT include a general basic hypergeometric series evaluator. This is a significant gap.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| `qhyper_phi(a_list, b_list, q, z, T)` | Evaluate unilateral _r phi_s basic hypergeometric series symbolically to O(q^T) | HIGH | General engine for phi series |
| `qhyper_psi(a_list, b_list, q, z, T)` | Evaluate bilateral _r psi_s basic hypergeometric series | HIGH | Bilateral requires sum from -inf to inf |
| Well-poised / very-well-poised detection | Automatically detect and exploit special structure | MEDIUM | Optimization of evaluation |
| Terminating vs non-terminating handling | Handle both cases correctly | MEDIUM | Terminating series are finite sums |

**Why valuable:** Mathematica has QHypergeometricPFQ; mpmath has qhyper. No Garvan equivalent. Researchers currently switch tools to evaluate these. Building this keeps them in one environment.

**Confidence:** HIGH that Garvan lacks this; HIGH that Mathematica/mpmath have it

---

#### D-2: Classical Summation Formula Verification and Application

Named summation theorems that evaluate basic hypergeometric series in closed form.

| Formula | What It Does | Complexity | Notes |
|---------|-------------|------------|-------|
| q-Gauss sum | 2phi1(a,b;c;q,c/(ab)) in closed form | MEDIUM | Requires q-Pochhammer ratio |
| q-Vandermonde (q-Chu-Vandermonde) | Terminating 2phi1(q^{-n},a;c;q,q) | MEDIUM | Finite sum identity |
| q-Saalschutz (q-Pfaff-Saalschutz) | Terminating balanced 3phi2 | MEDIUM | Jackson's theorem |
| q-Kummer | 2phi1 with specific parameters | MEDIUM | Requires q-Gauss |
| q-Dixon | Sum for well-poised 3phi2 | MEDIUM | Requires q-Vandermonde |
| q-Dougall | Summation for very-well-poised series | HIGH | Advanced identity |
| Rogers-Ramanujan identities | Specific product = series identities | MEDIUM | Iconic q-series identities |

**Why valuable:** These are the bread-and-butter identities researchers use daily. Having them as verified, callable operations (not just manually applied formulas) accelerates research.

**Confidence:** HIGH -- these are well-documented classical results from Gasper and Rahman's "Basic Hypergeometric Series"

---

#### D-3: Transformation Formulas

Named transformation theorems that convert one basic hypergeometric series into another.

| Formula | What It Does | Complexity | Notes |
|---------|-------------|------------|-------|
| Heine's transformations (3 forms) | Transform 2phi1 into different 2phi1 | MEDIUM | Three classical forms |
| Sears' transformation | Transform terminating balanced 4phi3 | HIGH | Multi-parameter |
| Watson's transformation | Transform 8phi7 to 4phi3 | HIGH | Very-well-poised to balanced |
| Bailey's transformation | Transform 10phi9 | VERY HIGH | Advanced |
| Jackson's transformation | Transform terminating 8phi7 | HIGH | Advanced |
| Euler's transformations (q-analog) | Basic 2phi1 -> 2phi1 | MEDIUM | Foundational |

**Why valuable:** Researchers spend significant time applying these by hand. Automated application (and verification) saves hours per identity.

**Confidence:** HIGH -- standard material from Gasper-Rahman textbook

---

#### D-4: Mock Theta Functions

Ramanujan's mock theta functions and Zwegers' harmonic weak Maass form framework.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| All 17 original Ramanujan mock theta functions | Series expansions, coefficient extraction | MEDIUM | Well-defined series |
| Additional mock thetas from lost notebook | Extended catalog | MEDIUM | ~20 additional functions |
| Mock modular form framework (Zwegers) | Represent mock thetas as holomorphic parts of harmonic Maass forms | VERY HIGH | Modern theoretical framework |
| Shadow computation | Compute the "shadow" (non-holomorphic correction) | VERY HIGH | Requires real-analytic modular forms |

**Why valuable:** Mock theta functions are one of the hottest areas in modern number theory. No existing CAS has dedicated symbolic support. This would be genuinely unique.

**Confidence:** MEDIUM -- the mathematical theory is well-established, but implementation complexity is uncertain; shadow computation may require numerical methods

---

#### D-5: Bailey Chain / Bailey Pair Machinery

Automated Bailey chain iteration for generating q-series identities.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Bailey pair database | Catalog of known Bailey pairs (alpha_n, beta_n) | MEDIUM | Data entry + verification |
| Bailey lemma application | Given a Bailey pair, produce a new one | HIGH | Iterative transformation |
| Bailey chain iteration | Apply Bailey lemma repeatedly to generate identity chains | HIGH | Recursive application |
| WP-Bailey pairs | Well-poised Bailey pair extensions (Andrews-Berkovich) | VERY HIGH | Generalization |
| Identity generation from chains | Automatically produce new q-series identities | VERY HIGH | Combines chain + series evaluation |

**Why valuable:** Bailey chains are a primary tool for discovering new identities. Garvan uses them in research papers but has no package automating them. The q-Gosper algorithm (Koornwinder, Paule-Riese) connects to this.

**Confidence:** MEDIUM -- well-established mathematics; no existing packaged tool found; implementation complexity uncertain

---

#### D-6: WZ Method (Wilf-Zeilberger) for q-Identities

Automated proving/discovery of q-hypergeometric identities via the WZ method.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| q-Zeilberger algorithm | Find recurrences for definite q-hypergeometric sums | VERY HIGH | Core algorithm from A=B book |
| q-Gosper algorithm | Find closed-form q-hypergeometric antidifferences | VERY HIGH | Required by q-Zeilberger |
| WZ certificate computation | Find WZ proof certificate for identity | VERY HIGH | Certification |
| Creative telescoping | General framework for sum/integral evaluation | VERY HIGH | Advanced algorithmic framework |

**Why valuable:** Zeilberger's EKHAD packages exist only in Maple/Mathematica. A modern implementation would serve the community. These algorithms can prove identities that the valence formula approach cannot (non-modular identities).

**Confidence:** MEDIUM -- algorithms well-documented in "A=B" (Petkovsek, Wilf, Zeilberger); Zeilberger has Maple implementations; reimplementation is feasible but complex

---

#### D-7: Performance via Rust Core

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Parallel series multiplication | Exploit multi-core for large T computations | HIGH | Rust concurrency |
| SIMD-accelerated coefficient arithmetic | Fast inner loops for series operations | HIGH | Platform-specific |
| Lazy/streaming series evaluation | Only compute coefficients as needed | MEDIUM | Iterator-based design |
| Batch computation pipeline mode | Process thousands of series without Python overhead | MEDIUM | CLI + batch API |

**Why valuable:** Garvan's Maple packages are single-threaded and slow for large computations. Researchers doing systematic searches (e.g., "find all partition congruences mod p for p < 1000") hit Maple's performance wall. Rust core would be 10-100x faster.

**Confidence:** HIGH -- Rust performance advantages are well-established; the specific speedup for q-series arithmetic is plausible based on BigInt benchmark literature

---

#### D-8: Modular Forms Integration

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Modular form spaces | Compute spaces M_k(Gamma_0(N)) and M_k(Gamma_1(N)) | VERY HIGH | Extends Garvan's MODFORMS |
| Hecke operators | Apply T_n to modular forms | VERY HIGH | Requires modular form representation |
| Sturm bound computation | Determine how many q-expansion terms suffice for equality | MEDIUM | Number-theoretic computation |
| Eisenstein series | Standard basis elements for modular form spaces | MEDIUM | Well-known series |

**Why valuable:** SageMath already does this well. This is a differentiator only if the Rust core makes it significantly faster. Consider deferring to v2+ unless performance is the primary selling point.

**Confidence:** MEDIUM -- SageMath covers this territory well; differentiation would come from performance, not capability

---

### Anti-Features (Do NOT Build)

| Anti-Feature | Why Requested | Why Problematic | Alternative |
|--------------|---------------|-----------------|-------------|
| Numerical floating-point evaluation | "I want to plot q-series" | Destroys exact arithmetic advantage; mpmath/numpy already do this. Mixing numerical and symbolic creates subtle bugs. | Expose coefficient extraction; let users feed to matplotlib/numpy for plotting. Provide optional float conversion as explicit lossy operation. |
| GUI / Interactive notebook | "I want a Maple-like worksheet" | Enormous engineering effort orthogonal to math engine. Jupyter already exists. | Provide first-class Jupyter integration via Python bindings. Let Jupyter be the GUI. |
| General-purpose CAS | "Add calculus, linear algebra, etc." | Scope explosion. SymPy/SageMath/Mathematica already exist. | Interop with SymPy for anything outside q-series domain. Focus exclusively on q-series/partition/modular forms. |
| Maple compatibility layer | "Read existing .mw worksheets" | Maple's language is proprietary and complex. Parsing it is a rabbit hole. | Provide migration guide with function-name mapping table. Let researchers translate their scripts. |
| Real-time collaborative editing | "Multiple researchers editing same session" | Enormous infrastructure complexity for minimal value in a math research tool. | File-based workflow with version control (git). |
| Numerical modular form computation | "Compute L-functions numerically" | Specialized domain (LMFDB, PARI/GP already do this). Numerical computation is antithetical to the symbolic mission. | Provide export to PARI/GP format for numerical work. |
| Automated theorem proving (general) | "Prove any q-series identity" | Undecidable in general. Garvan's approach (valence formula) works for modular function identities specifically. | Implement specific proof strategies (valence formula, WZ method, Bailey chains) that are known to terminate for well-defined classes. |

---

## Feature Dependencies

```
[Truncated Power Series Arithmetic]
    |
    +---> [q-Pochhammer (aqprod)]
    |         |
    |         +---> [q-Binomial (qbin)]
    |         +---> [Jacobi Triple Product (tripleprod)]
    |         +---> [Quintuple Product (quinprod)]
    |         +---> [Winquist Product (winquist)]
    |         +---> [Basic Hypergeometric Series (D-1)]
    |                   |
    |                   +---> [Summation Formulas (D-2)]
    |                   +---> [Transformation Formulas (D-3)]
    |
    +---> [Eta Product (etaq)]
    |         |
    |         +---> [etamake -- series-to-eta conversion]
    |         +---> [ETA Package identity proving]
    |
    +---> [Theta Functions (theta, theta2, theta3, theta4)]
    |         |
    |         +---> [jacprod -- JAC representation]
    |                   |
    |                   +---> [jacprodmake -- series-to-JAC conversion]
    |                   +---> [jac2prod, jac2series -- JAC-to-series]
    |                   +---> [thetaids identity proving]
    |
    +---> [Coefficient Extraction / sift]
    |         |
    |         +---> [findcong -- congruence discovery]
    |         +---> [Rank/Crank/SPT packages]
    |
    +---> [Series-to-Product Conversion (prodmake)]
              |
              +---> [etamake, jacprodmake, mprodmake, qetamake]
              +---> [qfactor, zqfactor]

[Symbolic Expression Model (JAC, ETA, generalized eta)]
    |
    +---> [Notation Conversions (eta2jac, jac2eprod, etc.)]
    |
    +---> [Cusp/Order Computation (cuspmake1, getacuspord, etc.)]
    |         |
    |         +---> [Modular Function Verification (Gamma1ModFunc)]
    |         +---> [Valence Formula Application]
    |         +---> [provemodfuncid -- Identity Proving]
    |
    +---> [Algebraic Relation Discovery (findlincombo, findhom, etc.)]

[Bailey Pair Database (D-5)]
    |
    +---> [Bailey Lemma Application]
              |
              +---> [Bailey Chain Iteration]
                        |
                        +---> [Identity Generation]

[q-Gosper Algorithm (D-6)]
    |
    +---> [q-Zeilberger Algorithm]
              |
              +---> [WZ Certificate Computation]
```

### Dependency Notes

- **Everything requires truncated power series arithmetic:** This is the absolute foundation. No function works without it.
- **q-Pochhammer is the second foundation:** Nearly all named products and series depend on it.
- **Series-to-product conversion depends on pattern matching:** `prodmake` is algorithmically complex (Andrews' algorithm) and everything in the conversion family depends on it.
- **Identity proving depends on cusp theory:** The `provemodfuncid` function requires the full cusp/order machinery from thetaids.
- **Algebraic relation discovery is independent of identity proving:** The `find*` functions use linear algebra, not modular forms theory.
- **Differentiators D-1 through D-3 depend only on q-Pochhammer:** Basic hypergeometric series, summation formulas, and transformation formulas are built on q-Pochhammer products and series arithmetic.
- **Differentiators D-5 and D-6 are largely independent:** Bailey chains and WZ method are separate algorithmic frameworks. They can be built in parallel.
- **Mock theta functions (D-4) depend on basic hypergeometric series:** Most mock thetas are defined as specific q-hypergeometric sums.

---

## MVP Definition

### Launch With (v1.0) -- Garvan qseries Core Parity

Minimum viable product -- what's needed for researchers to start using Q-Symbolic for basic q-series work.

- [ ] Truncated power series arithmetic (add, multiply, divide, compose) with exact rational coefficients
- [ ] q-Pochhammer symbol (`aqprod`) and q-binomial (`qbin`)
- [ ] All named products: `etaq`, `jacprod`, `tripleprod`, `quinprod`, `winquist`
- [ ] Theta functions: `theta`, `theta2`, `theta3`, `theta4`
- [ ] Series-to-product conversion: `prodmake`, `etamake`, `jacprodmake`, `mprodmake`, `qetamake`
- [ ] Factoring: `qfactor`, `zqfactor`
- [ ] Series utilities: `sift`, `qdegree`, `lqdegree`, coefficient extraction
- [ ] Algebraic relation discovery: `findlincombo`, `findhom`, `findpoly`, `findcong` (full suite)
- [ ] Check functions: `checkprod`, `checkmult`
- [ ] Python bindings with ergonomic API
- [ ] LaTeX output for all expressions

**Why this set:** These 41 functions constitute the qseries package and represent what researchers use daily. They do not require modular forms theory -- just series arithmetic, product representations, and linear algebra.

### Add After Core Validation (v1.x) -- Identity Proving + Partitions

Features to add once the core series engine is battle-tested.

- [ ] JAC/ETA symbolic representation model
- [ ] Full thetaids notation conversion suite (eta2jac, jac2eprod, etc.)
- [ ] Cusp and order computation (cuspmake1, getacuspord, etc.)
- [ ] `provemodfuncid` and `provemodfuncidBATCH` -- automatic identity proving
- [ ] ETA package identity proving pipeline
- [ ] Rank package: `N(m,n)`, `N(r,m,n)`
- [ ] Crank package: `M(m,n)`, `M(r,m,n)`
- [ ] SPT-Crank package: `NS(m,n)`, `NS(r,m,n)`
- [ ] T-Core package: t-core, t-quotient, t-core crank
- [ ] Basic hypergeometric series engine (D-1) -- _r phi_s evaluation
- [ ] Classical summation formulas (D-2) -- q-Gauss, q-Vandermonde, q-Saalschutz

**Trigger for adding:** Core series engine is correct and performant; early adopter feedback validates API design; researchers confirm they can replicate their existing Maple workflows.

### Future Consideration (v2+) -- Competitive Differentiation

Features to defer until parity is solid and the user base is established.

- [ ] Transformation formulas (D-3) -- Heine, Sears, Watson, Bailey, Jackson
- [ ] Mock theta functions (D-4) -- all Ramanujan mock thetas, Zwegers framework
- [ ] Bailey chain machinery (D-5) -- pair database, lemma application, chain iteration
- [ ] WZ method (D-6) -- q-Gosper, q-Zeilberger, creative telescoping
- [ ] RamaRobinsIDs equivalent -- Ramanujan-Robins identity search
- [ ] MODFORMS equivalent -- quasi-modular forms
- [ ] Full modular forms spaces (D-8) -- M_k(Gamma_0(N)), Hecke operators
- [ ] Performance optimizations (D-7) -- parallel multiplication, SIMD, streaming

**Why defer:** These require significant mathematical sophistication to implement correctly. Getting them wrong is worse than not having them. The core parity features are what enable adoption; the extensions are what sustain competitive advantage over time.

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Truncated power series arithmetic | HIGH | MEDIUM | P1 |
| q-Pochhammer / q-binomial | HIGH | LOW | P1 |
| Named products (eta, jac, triple, quintuple, Winquist) | HIGH | MEDIUM | P1 |
| Theta functions (theta, theta2, theta3, theta4) | HIGH | LOW | P1 |
| prodmake (series-to-product conversion) | HIGH | HIGH | P1 |
| etamake / jacprodmake / mprodmake | HIGH | HIGH | P1 |
| qfactor / zqfactor | HIGH | HIGH | P1 |
| sift / qdegree / coefficient extraction | HIGH | LOW | P1 |
| find* relation discovery suite | HIGH | MEDIUM | P1 |
| checkprod / checkmult | MEDIUM | LOW | P1 |
| Python bindings | HIGH | MEDIUM | P1 |
| LaTeX output | MEDIUM | LOW | P1 |
| JAC/ETA symbolic model | HIGH | MEDIUM | P2 |
| thetaids notation conversions | HIGH | MEDIUM | P2 |
| Cusp/order computation | HIGH | HIGH | P2 |
| provemodfuncid (identity proving) | HIGH | VERY HIGH | P2 |
| ETA package identity pipeline | HIGH | VERY HIGH | P2 |
| Rank/Crank/SPT/T-Core packages | MEDIUM | MEDIUM | P2 |
| Basic hypergeometric _r phi_s engine | HIGH | HIGH | P2 |
| Classical summation formulas | HIGH | MEDIUM | P2 |
| Transformation formulas | MEDIUM | HIGH | P3 |
| Mock theta functions | MEDIUM | HIGH | P3 |
| Bailey chain machinery | MEDIUM | VERY HIGH | P3 |
| WZ method (q-Zeilberger) | MEDIUM | VERY HIGH | P3 |
| Modular forms spaces | LOW | VERY HIGH | P3 |
| Performance optimizations | MEDIUM | HIGH | P3 |

**Priority key:**
- P1: Must have for launch -- researchers cannot switch without these
- P2: Should have -- full Garvan parity + first differentiators
- P3: Nice to have -- competitive differentiation, attract new users

---

## Competitor Feature Analysis

| Feature | Garvan (Maple) | Mathematica | SageMath | mpmath/SymPy | Q-Symbolic Target |
|---------|---------------|-------------|----------|-------------|-------------------|
| q-Pochhammer symbol | `aqprod` (finite only) | `QPochhammer` (finite + infinite, symbolic + numeric) | Via mpmath only | `qp` (numeric) | Full symbolic + arbitrary precision |
| q-Binomial | `qbin` | `QBinomial` | Not dedicated | Not available | `qbin` with symbolic q |
| q-Factorial | Not available | `QFactorial` | Via mpmath | `qfac` (numeric) | Symbolic q-factorial |
| q-Gamma | Not available | `QGamma` | Via mpmath | `qgamma` (numeric) | Symbolic q-gamma |
| Basic hypergeometric _r phi_s | **Not available** | `QHypergeometricPFQ` (full) | **Not available** | `qhyper` (numeric, limited) | Full symbolic evaluation |
| Series-to-product conversion | `prodmake`, `etamake`, `jacprodmake` (best in class) | **Not available** | **Not available** | **Not available** | Full Garvan parity |
| Algebraic relation discovery | `findlincombo`, `findhom`, `findpoly`, etc. (best in class) | **Not available** | **Not available** | **Not available** | Full Garvan parity |
| Theta function identity proving | `provemodfuncid` (best in class) | **Not available** | **Not available** | **Not available** | Full thetaids parity |
| Eta-product identities | ETA package (best in class) | **Not available** | `EtaProduct`, `EtaGroup` (partial) | **Not available** | Full ETA parity + performance |
| Eta q-expansion | `etaq` | `DedekindEta` (analytic) | `EtaProduct.q_expansion()` | **Not available** | Full symbolic expansion |
| Modular forms spaces | MODFORMS (level 1 only) | Partial | **Best in class** (full theory) | **Not available** | Defer to SageMath or v2+ |
| Partition rank/crank | Rank, Crank, SPT-Crank packages | **Not available** | `Partitions` (combinatorial, not rank/crank) | **Not available** | Full parity |
| Elliptic theta functions | `theta`, `theta2`, `theta3`, `theta4` | `EllipticTheta` (4 types, full) | Via mpmath | **Not available** | Full symbolic |
| Named products (triple, quintuple, Winquist) | All three | **Not available** | **Not available** | **Not available** | All three |
| WZ method | Via Zeilberger's EKHAD (Maple) | Via Zeilberger's packages | **Not available** | **Not available** | Future (P3) |
| Bailey chains | **Not available** | **Not available** | **Not available** | **Not available** | Future (P3) -- unique |
| Mock theta functions | **Not available** | **Not available** | **Not available** | **Not available** | Future (P3) -- unique |
| Performance (large T) | Slow (Maple interpreter) | Fast (Wolfram kernel) | Medium (Python/Cython) | Medium (C backend) | **Fastest** (Rust core) |
| Open source | No (Maple proprietary) | No (Mathematica proprietary) | Yes (GPL) | Yes (BSD) | Yes |
| Cost | Maple license (~$2400/yr academic) | Mathematica license (~$170/yr academic) | Free | Free | Free |

### Key Gaps in the Ecosystem

1. **No tool combines Garvan's conversion/discovery capabilities with general basic hypergeometric series.** Garvan excels at series-to-product conversion and relation finding. Mathematica excels at evaluating q-hypergeometric functions. Nobody does both.

2. **No tool automates Bailey chains.** This is entirely manual work across all platforms.

3. **No tool has dedicated mock theta function support.** Researchers define these ad hoc in whatever CAS they use.

4. **Garvan's packages require Maple.** Maple is expensive and declining in popularity. A free, open-source replacement removes a significant barrier to entry.

5. **SageMath's eta-product support is weaker than Garvan's.** SageMath has `EtaProduct` and `EtaGroup` for modular curves X_0(N) but lacks the full identity-proving pipeline.

6. **No tool is fast for large-scale systematic computation.** All existing tools are interpreted languages. A Rust core would enable searches over parameter spaces that are currently infeasible.

---

## Sources

### Primary (HIGH confidence)
- [Garvan qseries v1.3 function list](https://qseries.org/fgarvan/qmaple/qseries/functions/) -- official function documentation
- [Garvan thetaids v1.0 function list](https://qseries.org/fgarvan/qmaple/thetaids/functions-v1p0.html) -- official function documentation
- [Garvan qseries package main page](https://qseries.org/fgarvan/qmaple/qseries/) -- package overview and capabilities
- [Garvan jacprodmake documentation](https://qseries.org/fgarvan/qmaple/qseries/functions/jacprodmake.html) -- detailed function reference
- [Wolfram Language q Functions guide](https://reference.wolfram.com/language/guide/QFunctions.html) -- Mathematica q-series capabilities
- [mpmath q-functions documentation](https://mpmath.org/doc/current/functions/qfunctions.html) -- Python q-function capabilities
- [SageMath EtaProducts documentation](https://doc.sagemath.org/html/en/reference/modfrm/sage/modular/etaproducts.html) -- SageMath eta-product support

### Secondary (MEDIUM confidence)
- [Garvan q-product tutorial (arXiv:math/9812092)](https://arxiv.org/abs/math/9812092) -- package tutorial paper
- [Garvan ETA tutorial (arXiv:1907.09130)](https://arxiv.org/abs/1907.09130) -- ETA package tutorial
- [Garvan auto-theta paper (arXiv:1807.08051)](https://arxiv.org/abs/1807.08051) -- thetaids/ramarobinsids methodology
- [Garvan ETA package page](https://qseries.org/fgarvan/qmaple/ETA/) -- ETA package overview
- [Garvan Rank package](https://qseries.org/fgarvan/qmaple/rank/) -- Rank package overview
- [Garvan Crank package](https://qseries.org/fgarvan/qmaple/crank/) -- Crank package overview
- [Garvan SPT-Crank package](https://qseries.org/fgarvan/qmaple/sptcrank/) -- SPT-Crank package overview
- [Garvan T-Core package](https://qseries.org/fgarvan/qmaple/tcore/) -- T-Core package overview
- [Garvan RamaRobinsIDs package](https://qseries.org/fgarvan/qmaple/ramarobinsids/) -- RamaRobinsIDs overview
- [Garvan MODFORMS package](https://qseries.org/fgarvan/qmaple/modforms/) -- MODFORMS overview
- [Wolfram QPochhammer documentation](https://reference.wolfram.com/language/ref/QPochhammer.html)
- [Wolfram QHypergeometricPFQ documentation](https://reference.wolfram.com/language/ref/QHypergeometricPFQ.html)
- [Wolfram QBinomial documentation](https://reference.wolfram.com/language/ref/QBinomial.html.en)
- [Wolfram DedekindEta documentation](https://reference.wolfram.com/language/ref/DedekindEta.html)

### Tertiary (LOW confidence -- needs validation)
- ETA package internal function list -- could not parse PDF; function inventory inferred from tutorial abstract and cross-reference
- MODFORMS package function list -- could not access; only description available
- Bailey chain implementation complexity estimates -- based on academic literature, not implementation experience
- WZ method implementation complexity -- based on "A=B" book descriptions, not implementation experience
- Mock theta function implementation scope -- based on Wikipedia/MathWorld descriptions

---
*Feature research for: Q-Symbolic (q-series symbolic computation)*
*Researched: 2026-02-13*
