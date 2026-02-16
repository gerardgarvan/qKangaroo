# Requirements: q-Kangaroo

**Defined:** 2026-02-15
**Core Value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.

## v1.2 Requirements

Requirements for algorithmic q-hypergeometric identity proving. Each maps to roadmap phases.

### Polynomial Infrastructure

- [ ] **POLY-01**: Dense univariate polynomial type (QRatPoly) with add, sub, mul, div, rem over QRat coefficients
- [ ] **POLY-02**: Polynomial GCD via subresultant PRS with content extraction to prevent coefficient explosion
- [ ] **POLY-03**: Polynomial resultant computation
- [ ] **POLY-04**: q-shift operations: evaluate p(q*x), p(q^j*x) for polynomial p
- [ ] **POLY-05**: Rational function type (QRatRationalFunc) with arithmetic and simplification

### q-Gosper Algorithm

- [ ] **GOSP-01**: q-hypergeometric term ratio extraction from HypergeometricSeries as rational function of q^k
- [ ] **GOSP-02**: q-dispersion computation via resultant (find all j where gcd(a(x), b(q^j*x)) != 1)
- [ ] **GOSP-03**: q-Greatest Factorial Factorization (qGFF) decomposition into sigma, tau, p
- [ ] **GOSP-04**: Key equation solver (find polynomial f satisfying sigma(x)*f(qx) - tau(x)*f(x) = p(x))
- [ ] **GOSP-05**: Complete q-Gosper algorithm returning Summable(antidifference) or NotSummable

### q-Zeilberger & WZ Certificates

- [ ] **ZEIL-01**: Creative telescoping loop trying recurrence orders d = 1, 2, ... up to configurable max_order
- [ ] **ZEIL-02**: Recurrence output with polynomial coefficients c_0(q^n), ..., c_d(q^n)
- [ ] **ZEIL-03**: WZ proof certificate extraction from Zeilberger output
- [ ] **ZEIL-04**: Independent WZ certificate verification (user-supplied certificates accepted)
- [ ] **ZEIL-05**: FPS cross-verification of recurrence against numerical expansion

### Recurrence Solving

- [ ] **SOLV-01**: q-Petkovsek algorithm finds q-hypergeometric solutions of linear recurrences with polynomial coefficients
- [ ] **SOLV-02**: Closed-form output as product of q-Pochhammer symbols and q-powers

### Nonterminating Proofs

- [ ] **NTPR-01**: Parameter specialization method (Chen-Hou-Mu): replace parameter with x*q^n, apply q-Zeilberger to terminating version
- [ ] **NTPR-02**: Initial condition verification via FPS comparison for nonterminating identities

### Transformation Discovery

- [ ] **TRNS-01**: BFS/DFS search over Heine/Sears/Watson/Bailey transformation chains between two HypergeometricSeries
- [ ] **TRNS-02**: Return transformation sequence with intermediate forms, or "no chain found within depth"

### Python API & Documentation

- [ ] **API-01**: Python DSL functions for q_gosper, q_zeilberger, verify_wz, q_petkovsek
- [ ] **API-02**: Python DSL functions for prove_nonterminating and find_transformation_chain
- [ ] **API-03**: NumPy-style docstrings with LaTeX notation on all new functions
- [ ] **API-04**: Sphinx API reference pages for new functions integrated into existing docs site

## Future Requirements

Deferred to future milestones.

### Multi-Sum Creative Telescoping
- **MSUM-01**: Extend q-Zeilberger to handle multiple nested sums (qMultiSum equivalent)

### Batch Verification
- **BATCH-01**: Systematic verification of identity database (e.g., Gasper-Rahman Appendix II) with certificates

### Human-Readable Proof Output
- **PROOF-01**: Step-by-step proof narrative with LaTeX-renderable steps for publication

## Out of Scope

| Feature | Reason |
|---------|--------|
| General holonomic functions framework | Massively increases scope; q-hypergeometric algorithms are sufficient and faster for our domain |
| q-Integration (Jackson q-integral) | Different algorithmic domain; not needed for identity proving |
| Numerical-only verification as proof | Not a mathematical proof; FPS comparison already exists for sanity checks |
| General CAS pattern matching | Domain-specific structured types are faster and simpler than general symbolic rewriting |
| Automatic conjecture generation from data | Different problem domain; existing findlincombo/findhom/findpoly handle relation discovery |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| POLY-01..05 | Phase 13 | Pending |
| GOSP-01..05 | Phase 14 | Pending |
| ZEIL-01..05 | Phase 15 | Pending |
| SOLV-01..02 | Phase 16 | Pending |
| NTPR-01..02 | Phase 16 | Pending |
| TRNS-01..02 | Phase 16 | Pending |
| API-01..04 | Phase 17 | Pending |

**Coverage:**
- v1.2 requirements: 24 total
- Mapped to phases: 24
- Unmapped: 0

---
*Requirements defined: 2026-02-15*
*Last updated: 2026-02-15 after initial definition*
