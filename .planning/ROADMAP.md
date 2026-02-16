# Roadmap: q-Kangaroo

## Milestones

- v1.0 Core Engine - Phases 1-8 (shipped 2026-02-14)
- v1.1 Polish & Publish - Phases 9-12 (shipped 2026-02-15)
- v1.2 Algorithmic Identity Proving - Phases 13-17 (in progress)

## Phases

<details>
<summary>v1.0 Core Engine (Phases 1-8) - SHIPPED 2026-02-14</summary>

- [x] Phase 1: Expression Foundation (3/3 plans) -- 2026-02-13
- [x] Phase 2: Simplification & Series Engine (3/3 plans) -- 2026-02-13
- [x] Phase 3: Core q-Series & Partitions (4/4 plans) -- 2026-02-13
- [x] Phase 4: Series Analysis (7/7 plans) -- 2026-02-13
- [x] Phase 5: Python API (4/4 plans) -- 2026-02-13
- [x] Phase 6: Hypergeometric Series (4/4 plans) -- 2026-02-14
- [x] Phase 7: Identity Proving (4/4 plans) -- 2026-02-14
- [x] Phase 8: Mock Theta & Bailey Chains (4/4 plans) -- 2026-02-14

See `.planning/milestones/v1.0-MILESTONE-AUDIT.md` for details.

</details>

<details>
<summary>v1.1 Polish & Publish (Phases 9-12) - SHIPPED 2026-02-15</summary>

- [x] Phase 9: Package Rename & Structure (2/2 plans) -- 2026-02-14
- [x] Phase 10: PyPI Packaging & Metadata (2/2 plans) -- 2026-02-14
- [x] Phase 11: CI/CD Pipeline (2/2 plans) -- 2026-02-15
- [x] Phase 12: Documentation & UX Polish (4/4 plans) -- 2026-02-15

See `.planning/milestones/v1.1-ROADMAP.md` for details.

</details>

### v1.2 Algorithmic Identity Proving

**Milestone Goal:** Machine-prove q-hypergeometric identities using q-Gosper, q-Zeilberger, creative telescoping, and WZ certificates -- completing the transition from Garvan's Maple toolbox to a self-contained proving engine.

- [x] **Phase 13: Polynomial Infrastructure** - Exact polynomial and rational function arithmetic over QRat
- [ ] **Phase 14: q-Gosper Algorithm** - Indefinite q-hypergeometric summation via Gosper's algorithm
- [ ] **Phase 15: q-Zeilberger & WZ Certificates** - Creative telescoping and WZ proof certificates for definite sums
- [ ] **Phase 16: Extensions** - Recurrence solving, nonterminating proofs, transformation discovery
- [ ] **Phase 17: Python API & Documentation** - DSL functions, docstrings, and Sphinx pages for all new algorithms

## Phase Details

### Phase 13: Polynomial Infrastructure
**Goal**: Exact polynomial and rational function arithmetic over QRat is available as a foundation for algorithmic identity proving
**Depends on**: Nothing (builds on existing QRat/QInt from v1.0)
**Requirements**: POLY-01, POLY-02, POLY-03, POLY-04, POLY-05
**Success Criteria** (what must be TRUE):
  1. User can construct QRatPoly from coefficients, perform add/sub/mul/div/rem, and get exact QRat results
  2. Polynomial GCD of two polynomials with rational coefficients returns the correct monic GCD without coefficient explosion
  3. Polynomial resultant correctly identifies when two polynomials share a common root
  4. q-shift evaluation p(q^j * x) returns correct polynomial for arbitrary integer j
  5. Rational functions (quotient of two polynomials) support arithmetic and automatic GCD-based simplification
**Plans**: 3 plans

Plans:
- [x] 13-01-PLAN.md -- QRatPoly type with constructors, arithmetic, div/rem, content, eval, Display
- [x] 13-02-PLAN.md -- Subresultant PRS GCD, resultant, q-shift operations
- [x] 13-03-PLAN.md -- QRatRationalFunc with auto-simplification, arithmetic, integration tests

### Phase 14: q-Gosper Algorithm
**Goal**: Users can determine whether a q-hypergeometric sum has a closed-form antidifference, and obtain it when one exists
**Depends on**: Phase 13 (polynomial arithmetic)
**Requirements**: GOSP-01, GOSP-02, GOSP-03, GOSP-04, GOSP-05
**Success Criteria** (what must be TRUE):
  1. Given a HypergeometricSeries, the term ratio t(k+1)/t(k) is correctly extracted as a rational function of q^k
  2. q-dispersion computation correctly finds all integer shifts j where gcd(a(x), b(q^j * x)) is nontrivial
  3. qGFF decomposition produces sigma, tau, p factors satisfying the Gosper normal form constraints
  4. The complete q-Gosper algorithm returns Summable(antidifference) for known summable series (e.g., q-Vandermonde) and NotSummable for non-summable ones
**Plans**: TBD

Plans:
- [ ] 14-01: TBD
- [ ] 14-02: TBD

### Phase 15: q-Zeilberger & WZ Certificates
**Goal**: Users can prove q-hypergeometric identities by obtaining recurrences via creative telescoping and verifying them with WZ certificates
**Depends on**: Phase 14 (q-Gosper as subroutine)
**Requirements**: ZEIL-01, ZEIL-02, ZEIL-03, ZEIL-04, ZEIL-05
**Success Criteria** (what must be TRUE):
  1. Creative telescoping finds a recurrence for the q-Vandermonde sum at order d=1
  2. The recurrence output provides polynomial coefficients c_0(q^n), ..., c_d(q^n) that the user can inspect
  3. A WZ proof certificate is extracted and independently verified against the recurrence
  4. User-supplied WZ certificates are accepted and verified (not just internally generated ones)
  5. FPS cross-verification confirms the recurrence matches numerical series expansion to a given order
**Plans**: TBD

Plans:
- [ ] 15-01: TBD
- [ ] 15-02: TBD

### Phase 16: Extensions
**Goal**: Users can solve recurrences for closed forms, prove nonterminating identities, and discover transformation chains between hypergeometric series
**Depends on**: Phase 14 (q-Gosper for nonterminating proofs), Phase 15 (q-Zeilberger recurrences for solving)
**Requirements**: SOLV-01, SOLV-02, NTPR-01, NTPR-02, TRNS-01, TRNS-02
**Success Criteria** (what must be TRUE):
  1. q-Petkovsek finds q-hypergeometric solutions of recurrences produced by q-Zeilberger
  2. Closed-form output is expressed as products of q-Pochhammer symbols and q-powers
  3. Nonterminating identities are proved by parameter specialization (Chen-Hou-Mu method) reducing to terminating q-Zeilberger problems
  4. Transformation chain search finds known paths (e.g., Heine transform sequences) between two hypergeometric series within a configurable depth bound
**Plans**: TBD

Plans:
- [ ] 16-01: TBD
- [ ] 16-02: TBD
- [ ] 16-03: TBD

### Phase 17: Python API & Documentation
**Goal**: All v1.2 algorithms are accessible from Python with the same quality of documentation as existing functions
**Depends on**: Phases 13-16 (stable Rust interfaces)
**Requirements**: API-01, API-02, API-03, API-04
**Success Criteria** (what must be TRUE):
  1. Python functions q_gosper, q_zeilberger, verify_wz, q_petkovsek work from `import q_kangaroo`
  2. Python functions prove_nonterminating and find_transformation_chain work from `import q_kangaroo`
  3. All new functions have NumPy-style docstrings with LaTeX mathematical notation
  4. Sphinx API reference pages for the new functions are integrated into the existing documentation site
**Plans**: TBD

Plans:
- [ ] 17-01: TBD
- [ ] 17-02: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Expression Foundation | v1.0 | 3/3 | Complete | 2026-02-13 |
| 2. Simplification & Series Engine | v1.0 | 3/3 | Complete | 2026-02-13 |
| 3. Core q-Series & Partitions | v1.0 | 4/4 | Complete | 2026-02-13 |
| 4. Series Analysis | v1.0 | 7/7 | Complete | 2026-02-13 |
| 5. Python API | v1.0 | 4/4 | Complete | 2026-02-13 |
| 6. Hypergeometric Series | v1.0 | 4/4 | Complete | 2026-02-14 |
| 7. Identity Proving | v1.0 | 4/4 | Complete | 2026-02-14 |
| 8. Mock Theta & Bailey Chains | v1.0 | 4/4 | Complete | 2026-02-14 |
| 9. Package Rename & Structure | v1.1 | 2/2 | Complete | 2026-02-14 |
| 10. PyPI Packaging & Metadata | v1.1 | 2/2 | Complete | 2026-02-14 |
| 11. CI/CD Pipeline | v1.1 | 2/2 | Complete | 2026-02-15 |
| 12. Documentation & UX Polish | v1.1 | 4/4 | Complete | 2026-02-15 |
| 13. Polynomial Infrastructure | v1.2 | 3/3 | Complete | 2026-02-15 |
| 14. q-Gosper Algorithm | v1.2 | 0/TBD | Not started | - |
| 15. q-Zeilberger & WZ Certificates | v1.2 | 0/TBD | Not started | - |
| 16. Extensions | v1.2 | 0/TBD | Not started | - |
| 17. Python API & Documentation | v1.2 | 0/TBD | Not started | - |
