# Roadmap: q-Kangaroo

## Milestones

- v1.0 Core Engine - Phases 1-8 (shipped 2026-02-14)
- v1.1 Polish & Publish - Phases 9-12 (shipped 2026-02-15)
- v1.2 Algorithmic Identity Proving - Phases 13-17 (shipped 2026-02-16)
- v1.3 Documentation & Vignettes - Phases 18-21 (shipped 2026-02-16)
- v1.4 Installation & Build Guide - Phases 22-23 (shipped 2026-02-17)
- v1.5 Interactive REPL - Phases 24-28 (shipped 2026-02-18)
- v1.6 CLI Hardening & Manual - Phases 29-32 (shipped 2026-02-18)
- v2.0 Maple Compatibility - Phases 33-40 (in progress)

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

<details>
<summary>v1.2 Algorithmic Identity Proving (Phases 13-17) - SHIPPED 2026-02-16</summary>

- [x] Phase 13: Polynomial Infrastructure (3/3 plans) -- 2026-02-15
- [x] Phase 14: q-Gosper Algorithm (3/3 plans) -- 2026-02-16
- [x] Phase 15: q-Zeilberger & WZ Certificates (3/3 plans) -- 2026-02-16
- [x] Phase 16: Extensions (3/3 plans) -- 2026-02-16
- [x] Phase 17: Python API & Documentation (2/2 plans) -- 2026-02-16

See `.planning/milestones/v1.2-ROADMAP.md` for details.

</details>

<details>
<summary>v1.3 Documentation & Vignettes (Phases 18-21) - SHIPPED 2026-02-16</summary>

- [x] Phase 18: Docstring Enrichment (4/4 plans) -- 2026-02-16
- [x] Phase 19: Vignette Expansion (3/3 plans) -- 2026-02-16
- [x] Phase 20: New Vignettes & Migration Guide (3/3 plans) -- 2026-02-16
- [x] Phase 21: Sphinx Site Polish (2/2 plans) -- 2026-02-16

See `.planning/milestones/v1.3-ROADMAP.md` for details.

</details>

<details>
<summary>v1.4 Installation & Build Guide (Phases 22-23) - SHIPPED 2026-02-17</summary>

- [x] Phase 22: Installation Documentation (2/2 plans) -- 2026-02-17
- [x] Phase 23: Verification & Cross-References (2/2 plans) -- 2026-02-17

See `.planning/milestones/v1.4-ROADMAP.md` for details.

</details>

<details>
<summary>v1.5 Interactive REPL (Phases 24-28) - SHIPPED 2026-02-18</summary>

- [x] Phase 24: Parser & AST (2/2 plans) -- 2026-02-17
- [x] Phase 25: Evaluator & Function Dispatch (3/3 plans) -- 2026-02-18
- [x] Phase 26: REPL Shell & Session (2/2 plans) -- 2026-02-18
- [x] Phase 27: Output Commands & Polish (1/1 plan) -- 2026-02-18
- [x] Phase 28: Binary Packaging (1/1 plan) -- 2026-02-18

See `.planning/milestones/v1.5-ROADMAP.md` for details.

</details>

<details>
<summary>v1.6 CLI Hardening & Manual (Phases 29-32) - SHIPPED 2026-02-18</summary>

- [x] Phase 29: Static Linking (2/2 plans) -- 2026-02-18
- [x] Phase 30: Script Execution & CLI Flags (3/3 plans) -- 2026-02-18
- [x] Phase 31: Error Hardening & Exit Codes (2/2 plans) -- 2026-02-18
- [x] Phase 32: PDF Reference Manual (6/6 plans) -- 2026-02-18

See `.planning/milestones/v1.6-ROADMAP.md` for details.

</details>

### v2.0 Maple Compatibility (Phases 33-40)

**Milestone Goal:** Every qseries/thetaids function can be called with Garvan's exact Maple syntax -- researchers copy-paste from Maple worksheets and get correct results.

- [x] **Phase 33: Symbolic Variable Foundation** - Parser and evaluator support bare symbols, q-as-parameter, and q-monomial arguments -- 2026-02-19
- [x] **Phase 34: Product & Theta Signatures** - Product/theta functions accept Garvan's exact calling conventions -- 2026-02-19
- [x] **Phase 35: Series Analysis Signatures** - Series analysis functions accept Garvan's exact calling conventions -- 2026-02-19
- [x] **Phase 36: Relation Discovery Signatures** - All find* functions accept Garvan's signatures with symbolic labels -- 2026-02-19
- [ ] **Phase 37: New Functions - Theta & Jacobi** - theta, jac2prod, jac2series, qs2jaccombo implemented
- [ ] **Phase 38: New Functions - Analysis & Discovery** - checkmult, checkprod, lqdegree0, zqfactor, findprod implemented
- [ ] **Phase 39: Output & Compatibility** - Maple-style display, backward compat verified, all tests green
- [ ] **Phase 40: Documentation** - Manual, help, tab completion, migration guide all updated

## Phase Details

### Phase 33: Symbolic Variable Foundation
**Goal**: Users can type bare variable names, pass `q` as a function argument, and use q-monomials like `q^2` as parameters -- the prerequisite for all Maple-compatible signatures
**Depends on**: Phase 32 (existing CLI infrastructure)
**Requirements**: SYM-01, SYM-02, SYM-03, SYM-04
**Success Criteria** (what must be TRUE):
  1. Typing an undefined name like `f` at the REPL returns a Symbol value (no error)
  2. `etaq(q, 1, 20)` works -- `q` is accepted as a function parameter and the result is the correct q-series
  3. `aqprod(q^2, q, 5)` works -- q-monomial `q^2` is accepted as a function argument
  4. `x := 42` followed by `x` returns 42 (assignment still takes precedence over symbol fallback)
**Plans:** 3 plans
Plans:
- [x] 33-01-PLAN.md -- Value::Symbol variant, Token::Q/AstNode::Q removal, symbol fallback, test updates
- [x] 33-02-PLAN.md -- Symbol arithmetic (pow/mul/add/sub), polynomial display, variable-aware formatting
- [x] 33-03-PLAN.md -- Function dispatch with symbol args, single-quote lexer, restart/anames/unassign, integration tests

### Phase 34: Product & Theta Signatures
**Goal**: All product and theta functions accept Garvan's exact argument lists so researchers can call them identically to Maple
**Depends on**: Phase 33
**Requirements**: SIG-01, SIG-02, SIG-03, SIG-04, SIG-05, SIG-06, SIG-07, SIG-26
**Success Criteria** (what must be TRUE):
  1. `aqprod(q^2, q, 5)` returns the same polynomial as Garvan's `aqprod(q^2, q, 5)` in Maple
  2. `etaq(q, 3, 20)` returns the eta-quotient series matching Garvan output to 20 terms
  3. `jacprod(1, 5, q, 30)` and `qbin(4, 2, q, 10)` return correct results with explicit q and T arguments
  4. `numbpart(100)` returns 190569292 (primary name matches Maple, `partition_count` remains as alias)
  5. `tripleprod`, `quinprod`, and `winquist` all accept Garvan's exact argument forms
**Plans:** 2 plans
Plans:
- [x] 34-01-PLAN.md -- Maple-style dispatch for jacprod, tripleprod, quinprod, winquist, qbin, etaq multi-delta
- [x] 34-02-PLAN.md -- numbpart alias reversal, help text updates, tab completion, integration tests

### Phase 35: Series Analysis Signatures
**Goal**: Series analysis functions accept Garvan's calling conventions so sifting, product-make, and factoring workflows match Maple exactly
**Depends on**: Phase 34
**Requirements**: SIG-08, SIG-09, SIG-10, SIG-11, SIG-12, SIG-13, SIG-14
**Success Criteria** (what must be TRUE):
  1. `sift(s, q, 5, 2, 30)` extracts the correct residue-2-mod-5 subseries with explicit q and T
  2. `prodmake(f, q, 30)` and `etamake(f, q, 30)` decompose a series into product/eta forms with Garvan's signatures
  3. `jacprodmake(f, q, 30)` (3-arg) and `jacprodmake(f, q, 30, P)` (4-arg) both work
  4. `qfactor(f, q)` factors a q-series with explicit q parameter
**Plans:** 2 plans
Plans:
- [x] 35-01-PLAN.md -- Core jacprodmake period filter, Maple-style dispatch for all 7 functions, unit tests
- [x] 35-02-PLAN.md -- Help text updates for all 7 functions, CLI integration tests

### Phase 36: Relation Discovery Signatures
**Goal**: All relation-finding functions accept Garvan's signatures including symbolic label lists, and output uses those labels in results
**Depends on**: Phase 33, Phase 34
**Requirements**: SIG-15, SIG-16, SIG-17, SIG-18, SIG-19, SIG-20, SIG-21, SIG-22, SIG-23, SIG-24, SIG-25, OUT-01, OUT-02
**Success Criteria** (what must be TRUE):
  1. `findlincombo(f, [e1,e2], [F1,F2], q, 0)` finds the linear combination and prints result using symbolic labels F1, F2 (e.g., "12*F1 + 13*F2")
  2. `findhomcombo`, `findnonhomcombo`, `findlincombomodp`, `findhomcombomodp` all accept the SL label list in their Garvan position
  3. `findhom`, `findnonhom`, `findhommodp`, `findmaxind`, `findpoly` accept Garvan's exact argument lists with explicit q
  4. `findcong(QS, T)` outputs results in Garvan's `[B, A, R]` triple format
  5. `findcong(QS, T, LM)` and `findcong(QS, T, LM, XSET)` overloaded forms work
**Plans:** 3 plans
Plans:
- [x] 36-01-PLAN.md -- Core: pub monomial generators, Garvan findcong algorithm with GCD factoring
- [x] 36-02-PLAN.md -- Dispatch: all 11 functions with Garvan signatures, formatting helpers, unit tests
- [x] 36-03-PLAN.md -- Help text updates for all functions, CLI integration tests

### Phase 37: New Functions - Theta & Jacobi
**Goal**: The four theta/Jacobi conversion functions are available, enabling workflows that convert between theta, Jacobi product, and q-series representations
**Depends on**: Phase 34
**Requirements**: NEW-01, NEW-02, NEW-03, NEW-04
**Success Criteria** (what must be TRUE):
  1. `theta(z, q, 20)` returns the general theta series sum(z^i * q^(i^2), i=-20..20) as a formal power series in z and q
  2. `jac2prod(JP, q, 30)` converts a Jacobi product expression to explicit q-product form
  3. `jac2series(JP, q, 30)` converts a Jacobi product expression to a truncated q-series
  4. `qs2jaccombo(f, q, 30)` decomposes a q-series into a linear combination of Jacobi products
**Plans**: TBD

### Phase 38: New Functions - Analysis & Discovery
**Goal**: Five new analysis/discovery functions are available, completing the Garvan function inventory
**Depends on**: Phase 35
**Requirements**: NEW-05, NEW-06, NEW-07, NEW-08, NEW-09
**Success Criteria** (what must be TRUE):
  1. `checkmult(f, q, 30)` correctly reports whether the coefficients of a q-series are multiplicative
  2. `checkprod(f, q, 30)` validates whether a q-series represents a well-formed product and reports the result
  3. `lqdegree0(f, q)` returns the lowest q-degree (distinct from existing `lqdegree` which works on series values)
  4. `zqfactor(f, z, q)` factors a bivariate (z,q)-series into (z,q)-product form
  5. `findprod(L, q, maxcoeff, maxexp)` searches for a product identity matching the given series list
**Plans**: TBD

### Phase 39: Output & Compatibility
**Goal**: Series display matches Maple conventions and all existing v1.x calling conventions still work as aliases
**Depends on**: Phase 34, Phase 35, Phase 36, Phase 37, Phase 38
**Requirements**: OUT-03, COMPAT-01, COMPAT-02
**Success Criteria** (what must be TRUE):
  1. Series output uses Maple-style polynomial ordering (descending powers) when appropriate
  2. Every v1.x function signature (e.g., `etaq(1, 20)` without explicit q) continues to work and returns the same result
  3. The full existing test suite (836 core + 294 CLI tests) passes with zero regressions
**Plans**: TBD

### Phase 40: Documentation
**Goal**: All documentation reflects the new Maple-compatible signatures so users can learn the system from any entry point
**Depends on**: Phase 39
**Requirements**: DOC-01, DOC-02, DOC-03, DOC-04, DOC-05, DOC-06
**Success Criteria** (what must be TRUE):
  1. PDF reference manual documents every new and changed function with Garvan's exact signatures, formal math definitions, and worked examples
  2. REPL `help(function_name)` shows updated signatures and examples for all changed functions; new functions appear in help categories
  3. Tab completion includes all new function names (theta, jac2prod, jac2series, qs2jaccombo, checkmult, checkprod, lqdegree0, zqfactor, findprod, numbpart)
  4. Maple migration guide shows side-by-side examples where q-Kangaroo syntax is now identical to Maple (no translation needed)
  5. Python API docstrings and README quick-start reflect any signature changes
**Plans**: TBD

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
| 14. q-Gosper Algorithm | v1.2 | 3/3 | Complete | 2026-02-16 |
| 15. q-Zeilberger & WZ Certificates | v1.2 | 3/3 | Complete | 2026-02-16 |
| 16. Extensions | v1.2 | 3/3 | Complete | 2026-02-16 |
| 17. Python API & Documentation | v1.2 | 2/2 | Complete | 2026-02-16 |
| 18. Docstring Enrichment | v1.3 | 4/4 | Complete | 2026-02-16 |
| 19. Vignette Expansion | v1.3 | 3/3 | Complete | 2026-02-16 |
| 20. New Vignettes & Migration Guide | v1.3 | 3/3 | Complete | 2026-02-16 |
| 21. Sphinx Site Polish | v1.3 | 2/2 | Complete | 2026-02-16 |
| 22. Installation Documentation | v1.4 | 2/2 | Complete | 2026-02-17 |
| 23. Verification & Cross-References | v1.4 | 2/2 | Complete | 2026-02-17 |
| 24. Parser & AST | v1.5 | 2/2 | Complete | 2026-02-17 |
| 25. Evaluator & Function Dispatch | v1.5 | 3/3 | Complete | 2026-02-18 |
| 26. REPL Shell & Session | v1.5 | 2/2 | Complete | 2026-02-18 |
| 27. Output Commands & Polish | v1.5 | 1/1 | Complete | 2026-02-18 |
| 28. Binary Packaging | v1.5 | 1/1 | Complete | 2026-02-18 |
| 29. Static Linking | v1.6 | 2/2 | Complete | 2026-02-18 |
| 30. Script Execution & CLI Flags | v1.6 | 3/3 | Complete | 2026-02-18 |
| 31. Error Hardening & Exit Codes | v1.6 | 2/2 | Complete | 2026-02-18 |
| 32. PDF Reference Manual | v1.6 | 6/6 | Complete | 2026-02-18 |
| 33. Symbolic Variable Foundation | v2.0 | 3/3 | Complete | 2026-02-19 |
| 34. Product & Theta Signatures | v2.0 | 2/2 | Complete | 2026-02-19 |
| 35. Series Analysis Signatures | v2.0 | 2/2 | Complete | 2026-02-19 |
| 36. Relation Discovery Signatures | v2.0 | 3/3 | Complete | 2026-02-19 |
| 37. New Functions - Theta & Jacobi | v2.0 | 0/TBD | Not started | - |
| 38. New Functions - Analysis & Discovery | v2.0 | 0/TBD | Not started | - |
| 39. Output & Compatibility | v2.0 | 0/TBD | Not started | - |
| 40. Documentation | v2.0 | 0/TBD | Not started | - |
