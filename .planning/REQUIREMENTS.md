# Requirements: Q-Symbolic

**Defined:** 2026-02-13
**Core Value:** Every function in Garvan's Maple packages works correctly in Q-Symbolic, producing matching output -- so researchers can switch without losing any capability.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Core Engine

- [ ] **CORE-01**: Expression IR with hash-consing arena (ExprArena, u32 ExprRef, Expr enum with q-specific nodes)
- [ ] **CORE-02**: Arbitrary precision integer arithmetic via GMP (rug crate -- BigInt)
- [ ] **CORE-03**: Arbitrary precision rational arithmetic via GMP (rug crate -- BigRat)
- [ ] **CORE-04**: Phased rewrite/simplification engine with pattern matching (6-phase strategy)
- [ ] **CORE-05**: Formal power series with sparse representation (BTreeMap, truncation tracking)
- [ ] **CORE-06**: Lazy generators for infinite product expansion
- [ ] **CORE-07**: Series arithmetic (add, multiply, truncate, coefficient extraction)
- [ ] **CORE-08**: LaTeX rendering of all expression types
- [ ] **CORE-09**: Unicode pretty-printing for terminal output

### q-Series Functions (qseries parity)

- [ ] **QSER-01**: q-Pochhammer symbol -- aqprod(a, q, n) for finite and infinite n
- [ ] **QSER-02**: q-binomial coefficient -- qbin(n, k, q)
- [ ] **QSER-03**: Named product -- etaq(b, t, q) (eta-quotient building block)
- [ ] **QSER-04**: Named product -- jacprod(a, q) (Jacobi product)
- [ ] **QSER-05**: Named product -- tripleprod(a, q) (Jacobi triple product)
- [ ] **QSER-06**: Named product -- quinprod(a, q) (quintuple product)
- [ ] **QSER-07**: Named product -- winquist(a, b, q) (Winquist product)
- [ ] **QSER-08**: Theta functions -- theta2(q), theta3(q), theta4(q)
- [ ] **QSER-09**: Series-to-product conversion -- prodmake(f, q, n) via Andrews' algorithm
- [ ] **QSER-10**: Series-to-product conversion -- etamake(f, q, n)
- [ ] **QSER-11**: Series-to-product conversion -- jacprodmake(f, q, n)
- [ ] **QSER-12**: Series-to-product conversion -- mprodmake, qetamake
- [ ] **QSER-13**: Series factoring -- qfactor(f, q), zqfactor
- [ ] **QSER-14**: Series utilities -- sift(f, q, m, j) for extracting subsequences
- [ ] **QSER-15**: Series utilities -- qdegree, lqdegree (degree bounds)
- [ ] **QSER-16**: Relation discovery -- findlincombo (find linear combinations)
- [ ] **QSER-17**: Relation discovery -- findhom (find homogeneous relations)
- [ ] **QSER-18**: Relation discovery -- findpoly (find polynomial relations)
- [ ] **QSER-19**: Relation discovery -- findcong and full suite (12 functions total)

### Hypergeometric Series

- [ ] **HYPR-01**: Basic hypergeometric series _rφ_s representation and term-by-term evaluation
- [ ] **HYPR-02**: Bilateral hypergeometric series _rψ_s representation
- [ ] **HYPR-03**: q-Gauss summation formula
- [ ] **HYPR-04**: q-Vandermonde summation formula
- [ ] **HYPR-05**: q-Saalschutz summation formula
- [ ] **HYPR-06**: q-Kummer and q-Dixon summation formulas
- [ ] **HYPR-07**: Heine's transformation (all 3 forms)
- [ ] **HYPR-08**: Sears' _4φ_3 transformation
- [ ] **HYPR-09**: Watson's transformation
- [ ] **HYPR-10**: Bailey's transformation

### Identity Proving (thetaids + ETA parity)

- [ ] **IDPR-01**: JAC symbolic representation model (Jacobi products as data structure)
- [ ] **IDPR-02**: ETA symbolic representation model (eta quotients as data structure)
- [ ] **IDPR-03**: Cusp computation -- cuspmake1, getacuspord suite
- [ ] **IDPR-04**: Order computation at cusps for modular functions
- [ ] **IDPR-05**: provemodfuncid -- automatic identity proving via valence formula
- [ ] **IDPR-06**: ETA package identity pipeline (verify eta-quotient identities)
- [ ] **IDPR-07**: Identity database -- searchable collection of verified identities with citations (TOML format)
- [ ] **IDPR-08**: Identity lookup by tags, involved functions, and structural patterns

### Partitions & Combinatorics

- [ ] **PART-01**: Partition function p(n) computation for arbitrary n
- [ ] **PART-02**: Generating functions for restricted partitions (distinct parts, odd parts, bounded)
- [ ] **PART-03**: Rank and crank computation
- [ ] **PART-04**: Mock theta functions -- Ramanujan's third-order (f, phi, psi, chi)
- [ ] **PART-05**: Mock theta functions -- fifth and seventh order
- [ ] **PART-06**: Zwegers' completions to harmonic Maass forms
- [ ] **PART-07**: Appell-Lerch sums
- [ ] **PART-08**: Universal mock theta function g(x, q)
- [ ] **PART-09**: Bailey pair database (indexed by type)
- [ ] **PART-10**: Bailey lemma application and chain iteration
- [ ] **PART-11**: Automated discovery of new Bailey pairs from conjectured identities

### Python API

- [ ] **PYTH-01**: PyO3 bindings with QExpr opaque handles wrapping ExprRef
- [ ] **PYTH-02**: QSession managing Arc<Mutex<Session>> for arena ownership
- [ ] **PYTH-03**: Python DSL -- symbols(), qpoch(), hyper_q(), theta(), etc.
- [ ] **PYTH-04**: LaTeX rendering via `_repr_latex_()` for notebook display
- [ ] **PYTH-05**: Batch computation mode for systematic searches and pipelines

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Jupyter & Interactive

- **JUPT-01**: Jupyter kernel with auto-rendering of expressions
- **JUPT-02**: Interactive exploration widgets (parameter sliders, live series plots)
- **JUPT-03**: NumPy integration for coefficient arrays

### Advanced Methods

- **ADVN-01**: WZ method (q-Zeilberger algorithm for definite q-hypergeometric summation)
- **ADVN-02**: Creative telescoping
- **ADVN-03**: Sister Celine's method (q-analog)

### Quantum Algebra

- **QALG-01**: Quantum integers, factorials, and binomials
- **QALG-02**: U_q(sl_2) representations
- **QALG-03**: R-matrices and Yang-Baxter equation
- **QALG-04**: Connection to knot polynomials (Jones polynomial via q-series)

### Ecosystem

- **ECOS-01**: WASM compilation for web playground
- **ECOS-02**: OEIS integration (lookup sequences, cross-reference)
- **ECOS-03**: Community plugin API for extending identity database
- **ECOS-04**: Comprehensive documentation and example notebooks

## Out of Scope

| Feature | Reason |
|---------|--------|
| GUI / desktop application | CLI and library only -- researchers use scripts and notebooks |
| Numerical-only computation mode | Symbolic-first always; numerical evaluation opt-in |
| General-purpose CAS features | Focused q-series tool, not a Maple/Mathematica replacement for calculus/linear algebra |
| Mobile app | Not a use case for symbolic math research |
| Real-time collaboration | Single-user research tool |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 1 | Pending |
| CORE-02 | Phase 1 | Pending |
| CORE-03 | Phase 1 | Pending |
| CORE-04 | Phase 2 | Pending |
| CORE-05 | Phase 2 | Pending |
| CORE-06 | Phase 2 | Pending |
| CORE-07 | Phase 2 | Pending |
| CORE-08 | Phase 1 | Pending |
| CORE-09 | Phase 1 | Pending |
| QSER-01 | Phase 3 | Pending |
| QSER-02 | Phase 3 | Pending |
| QSER-03 | Phase 3 | Pending |
| QSER-04 | Phase 3 | Pending |
| QSER-05 | Phase 3 | Pending |
| QSER-06 | Phase 3 | Pending |
| QSER-07 | Phase 3 | Pending |
| QSER-08 | Phase 3 | Pending |
| QSER-09 | Phase 4 | Pending |
| QSER-10 | Phase 4 | Pending |
| QSER-11 | Phase 4 | Pending |
| QSER-12 | Phase 4 | Pending |
| QSER-13 | Phase 4 | Pending |
| QSER-14 | Phase 4 | Pending |
| QSER-15 | Phase 4 | Pending |
| QSER-16 | Phase 4 | Pending |
| QSER-17 | Phase 4 | Pending |
| QSER-18 | Phase 4 | Pending |
| QSER-19 | Phase 4 | Pending |
| HYPR-01 | Phase 6 | Pending |
| HYPR-02 | Phase 6 | Pending |
| HYPR-03 | Phase 6 | Pending |
| HYPR-04 | Phase 6 | Pending |
| HYPR-05 | Phase 6 | Pending |
| HYPR-06 | Phase 6 | Pending |
| HYPR-07 | Phase 6 | Pending |
| HYPR-08 | Phase 6 | Pending |
| HYPR-09 | Phase 6 | Pending |
| HYPR-10 | Phase 6 | Pending |
| IDPR-01 | Phase 7 | Pending |
| IDPR-02 | Phase 7 | Pending |
| IDPR-03 | Phase 7 | Pending |
| IDPR-04 | Phase 7 | Pending |
| IDPR-05 | Phase 7 | Pending |
| IDPR-06 | Phase 7 | Pending |
| IDPR-07 | Phase 7 | Pending |
| IDPR-08 | Phase 7 | Pending |
| PART-01 | Phase 3 | Pending |
| PART-02 | Phase 3 | Pending |
| PART-03 | Phase 3 | Pending |
| PART-04 | Phase 8 | Pending |
| PART-05 | Phase 8 | Pending |
| PART-06 | Phase 8 | Pending |
| PART-07 | Phase 8 | Pending |
| PART-08 | Phase 8 | Pending |
| PART-09 | Phase 8 | Pending |
| PART-10 | Phase 8 | Pending |
| PART-11 | Phase 8 | Pending |
| PYTH-01 | Phase 5 | Pending |
| PYTH-02 | Phase 5 | Pending |
| PYTH-03 | Phase 5 | Pending |
| PYTH-04 | Phase 5 | Pending |
| PYTH-05 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 62 total
- Mapped to phases: 62
- Unmapped: 0

---
*Requirements defined: 2026-02-13*
*Last updated: 2026-02-13 after roadmap creation*
