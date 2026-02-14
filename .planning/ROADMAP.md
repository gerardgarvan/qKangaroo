# Roadmap: Q-Symbolic

## Overview

Q-Symbolic delivers an open-source symbolic computation engine for q-series, replacing Frank Garvan's proprietary Maple packages (qseries, thetaids, ETA) with a Rust core and Python API. The roadmap follows a strict foundation-to-parity-to-extensions sequence: first build the expression IR and series engine, then achieve function-by-function parity with Garvan's qseries package, expose it to Python, and finally add hypergeometric series, identity proving, and mock theta/Bailey chain capabilities that go beyond Garvan.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Expression Foundation** - Hash-consed expression arena, arbitrary precision arithmetic, and rendering
- [x] **Phase 2: Simplification & Series Engine** - Phased rewrite engine and formal power series with sparse representation
- [x] **Phase 3: Core q-Series & Partitions** - q-Pochhammer, named products, theta functions, and partition functions
- [ ] **Phase 4: Series Analysis** - Series-to-product conversion, factoring, utilities, and relation discovery
- [ ] **Phase 5: Python API** - PyO3 bindings, session management, DSL, and batch computation
- [ ] **Phase 6: Hypergeometric Series** - Basic hypergeometric representation, summation formulas, and transformations
- [ ] **Phase 7: Identity Proving** - JAC/ETA models, cusp computation, automatic proving, and identity database
- [ ] **Phase 8: Mock Theta & Bailey Chains** - Mock theta functions, Zwegers completions, Appell-Lerch sums, Bailey machinery

## Phase Details

### Phase 1: Expression Foundation
**Goal**: Researchers can create, store, and display arbitrary q-series expressions with exact arithmetic
**Depends on**: Nothing (first phase)
**Requirements**: CORE-01, CORE-02, CORE-03, CORE-08, CORE-09
**Success Criteria** (what must be TRUE):
  1. Expressions with q-Pochhammer, theta, eta, and hypergeometric nodes can be constructed and stored in the arena with O(1) structural equality via hash consing
  2. Arithmetic on BigInt and BigRat values produces exact results matching GMP reference output for edge cases (zero, negative, large values)
  3. Every expression type renders to valid LaTeX that compiles without errors
  4. Every expression type renders to readable Unicode for terminal display
  5. Structurally identical expressions always resolve to the same ExprRef (hash-consing deduplication is verifiable)
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md -- Workspace scaffold, Expr enum, ExprArena with hash-consing, symbol registry, canonical ordering
- [x] 01-02-PLAN.md -- TDD: BigInt/BigRat arithmetic edge case verification
- [x] 01-03-PLAN.md -- LaTeX and Unicode rendering for all expression types

### Phase 2: Simplification & Series Engine
**Goal**: Expressions can be simplified via rewrite rules and expanded into formal power series with correct truncated arithmetic
**Depends on**: Phase 1
**Requirements**: CORE-04, CORE-05, CORE-06, CORE-07
**Success Criteria** (what must be TRUE):
  1. The simplification engine applies rewrite rules in phased order and terminates on all inputs (no infinite loops on adversarial expressions)
  2. Formal power series with sparse representation correctly stores and retrieves coefficients, and tracks truncation order explicitly
  3. Multiplying two O(q^N) series produces the correct result truncated to O(q^N) without creating O(q^2N) intermediates
  4. Lazy generators for infinite products yield correct coefficients on demand up to any requested truncation order
  5. Series addition, multiplication, and coefficient extraction match hand-computed results for known q-series identities (e.g., Euler's identity, Jacobi triple product)
**Plans**: 3 plans

Plans:
- [x] 02-01-PLAN.md -- TDD: FormalPowerSeries data structure and series arithmetic (add, mul, invert, shift)
- [x] 02-02-PLAN.md -- Phased simplification engine with 4 rule phases (normalize, cancel, collect, simplify)
- [x] 02-03-PLAN.md -- Lazy infinite product generators with Euler function and partition function verification

### Phase 3: Core q-Series & Partitions
**Goal**: Researchers can compute q-Pochhammer symbols, named products, theta functions, and partition functions matching Garvan's qseries output
**Depends on**: Phase 2
**Requirements**: QSER-01, QSER-02, QSER-03, QSER-04, QSER-05, QSER-06, QSER-07, QSER-08, PART-01, PART-02, PART-03
**Success Criteria** (what must be TRUE):
  1. aqprod(a, q, n) matches Maple output for n in {-5, -1, 0, 1, 2, 5, infinity} and a in {0, 1, q, q^2, generic symbol}
  2. All named products (etaq, jacprod, tripleprod, quinprod, winquist) expand to series matching Garvan's output to O(q^50)
  3. theta2(q), theta3(q), theta4(q) produce correct series expansions verified against known identities (e.g., Jacobi theta relations)
  4. p(n) returns correct partition counts for n = 0..200 matching OEIS A000041, and restricted partition generating functions (distinct parts, odd parts, bounded) produce correct series
  5. Rank and crank generating functions produce correct series matching published tables
**Plans**: 4 plans

Plans:
- [x] 03-01-PLAN.md -- QMonomial, PochhammerOrder, aqprod (q-Pochhammer symbol), and qbin (q-binomial coefficient)
- [x] 03-02-PLAN.md -- Named products: etaq, jacprod, tripleprod, quinprod, winquist
- [x] 03-03-PLAN.md -- Theta functions: theta2, theta3, theta4 with product representations
- [x] 03-04-PLAN.md -- Partition functions (p(n) pentagonal recurrence, restricted GFs) and rank/crank

### Phase 4: Series Analysis
**Goal**: Researchers can convert between series and product representations, factor q-series, and discover algebraic relations -- completing qseries package parity
**Depends on**: Phase 3
**Requirements**: QSER-09, QSER-10, QSER-11, QSER-12, QSER-13, QSER-14, QSER-15, QSER-16, QSER-17, QSER-18, QSER-19
**Success Criteria** (what must be TRUE):
  1. prodmake(f, q, n) recovers the product form of known infinite products (e.g., Euler product, Jacobi triple product) from their series expansions, matching Garvan's output
  2. etamake and jacprodmake correctly express series as eta-quotients and Jacobi products respectively, matching Garvan's results
  3. qfactor correctly factors q-series polynomials, and sift(f, q, m, j) correctly extracts arithmetic subsequences
  4. findlincombo, findhom, and findpoly discover known relations (e.g., Rogers-Ramanujan as linear combination) when given sufficient terms
  5. The full relation discovery suite (findcong and all 12 functions) runs without error and produces results matching Garvan's examples from the qseries documentation
**Plans**: 6 plans

Plans:
- [ ] 04-01-PLAN.md -- Andrews' algorithm (prodmake) with number theory helpers (mobius, divisors)
- [ ] 04-02-PLAN.md -- q-polynomial factoring (qfactor) and utilities (sift, qdegree, lqdegree)
- [ ] 04-03-PLAN.md -- Rational linear algebra (Gaussian elimination, null space over QRat and Z/pZ)
- [ ] 04-04-PLAN.md -- Series-to-product post-processing (etamake, jacprodmake, mprodmake, qetamake)
- [ ] 04-05-PLAN.md -- Core relation discovery (findlincombo, findhom, findpoly)
- [ ] 04-06-PLAN.md -- Full relation discovery suite (findcong + 8 functions, modp variants)

### Phase 5: Python API
**Goal**: Researchers can use Q-Symbolic from Python with natural syntax, LaTeX display, and batch computation for systematic searches
**Depends on**: Phase 4
**Requirements**: PYTH-01, PYTH-02, PYTH-03, PYTH-04, PYTH-05
**Success Criteria** (what must be TRUE):
  1. Python users can create q-series expressions using natural DSL syntax (e.g., qpoch(a, q, n), theta3(q)) and all Phase 3-4 functions are callable from Python
  2. QSession correctly manages arena ownership across Python's garbage collection -- no memory leaks after creating and discarding thousands of expressions
  3. Expressions display as rendered LaTeX in Jupyter notebooks via _repr_latex_() and as Unicode in Python REPL
  4. Batch computation mode can run systematic parameter searches (e.g., scanning q-Pochhammer products over parameter grids) and return results as Python collections
  5. A Garvan tutorial example (e.g., finding a q-series identity) can be replicated end-to-end in a Python script using Q-Symbolic
**Plans**: TBD

Plans:
- [ ] 05-01: TBD
- [ ] 05-02: TBD

### Phase 6: Hypergeometric Series
**Goal**: Researchers can construct, evaluate, and transform basic hypergeometric series using classical summation and transformation formulas
**Depends on**: Phase 3
**Requirements**: HYPR-01, HYPR-02, HYPR-03, HYPR-04, HYPR-05, HYPR-06, HYPR-07, HYPR-08, HYPR-09, HYPR-10
**Success Criteria** (what must be TRUE):
  1. _r-phi-s and _r-psi-s series can be constructed with arbitrary parameters and evaluated term-by-term to any truncation order, with results matching Gasper-Rahman examples
  2. q-Gauss, q-Vandermonde, and q-Saalschutz summation formulas are automatically applied when applicable and produce correct closed-form results
  3. q-Kummer and q-Dixon summation formulas produce correct results matching published tables
  4. Heine's transformation (all 3 forms), Sears' 4-phi-3, Watson's, and Bailey's transformations correctly convert between hypergeometric representations, verified by expanding both sides to O(q^50)
  5. Researchers can verify a hypergeometric identity by constructing both sides and confirming series agreement to arbitrary precision
**Plans**: TBD

Plans:
- [ ] 06-01: TBD
- [ ] 06-02: TBD
- [ ] 06-03: TBD

### Phase 7: Identity Proving
**Goal**: Researchers can prove q-series identities automatically using the valence formula method, matching thetaids and ETA package capabilities
**Depends on**: Phase 4, Phase 6
**Requirements**: IDPR-01, IDPR-02, IDPR-03, IDPR-04, IDPR-05, IDPR-06, IDPR-07, IDPR-08
**Success Criteria** (what must be TRUE):
  1. JAC and ETA symbolic representations correctly model Jacobi products and eta quotients as structured data, with conversion between representations
  2. Cusp computation (cuspmake1, getacuspord suite) produces correct cusp sets and orders matching Garvan's thetaids output for standard modular groups
  3. provemodfuncid correctly proves known modular function identities via the valence formula, returning a proof certificate or an explicit counterexample
  4. The ETA identity pipeline verifies eta-quotient identities end-to-end, matching results from Garvan's ETA package examples
  5. The identity database contains searchable verified identities (TOML format) that can be looked up by tags, involved functions, and structural patterns
**Plans**: TBD

Plans:
- [ ] 07-01: TBD
- [ ] 07-02: TBD
- [ ] 07-03: TBD

### Phase 8: Mock Theta & Bailey Chains
**Goal**: Researchers can work with mock theta functions, Zwegers completions, Appell-Lerch sums, and systematically generate new identities via Bailey chain machinery
**Depends on**: Phase 6, Phase 7
**Requirements**: PART-04, PART-05, PART-06, PART-07, PART-08, PART-09, PART-10, PART-11
**Success Criteria** (what must be TRUE):
  1. Ramanujan's third-order mock theta functions (f, phi, psi, chi) and fifth/seventh-order functions produce correct series expansions matching published tables
  2. Zwegers' completions transform mock theta functions into harmonic Maass forms, and the universal mock theta function g(x, q) matches known evaluations
  3. Appell-Lerch sums compute correctly and satisfy known functional equations
  4. The Bailey pair database stores known pairs by type, and the Bailey lemma produces correct new pairs from existing ones via chain iteration
  5. Automated Bailey pair discovery, given a conjectured identity, can verify or refute it by searching the pair database and applying the lemma

**Plans**: TBD

Plans:
- [ ] 08-01: TBD
- [ ] 08-02: TBD
- [ ] 08-03: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8

| Phase | Plans Complete | Status | Completed |
|-------|---------------|--------|-----------|
| 1. Expression Foundation | 3/3 | Complete | 2026-02-13 |
| 2. Simplification & Series Engine | 3/3 | Complete | 2026-02-13 |
| 3. Core q-Series & Partitions | 4/4 | Complete | 2026-02-13 |
| 4. Series Analysis | 0/6 | Not started | - |
| 5. Python API | 0/TBD | Not started | - |
| 6. Hypergeometric Series | 0/TBD | Not started | - |
| 7. Identity Proving | 0/TBD | Not started | - |
| 8. Mock Theta & Bailey Chains | 0/TBD | Not started | - |
