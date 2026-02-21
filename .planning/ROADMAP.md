# Roadmap: q-Kangaroo

## Milestones

- v1.0 Core Engine - Phases 1-8 (shipped 2026-02-14)
- v1.1 Polish & Publish - Phases 9-12 (shipped 2026-02-15)
- v1.2 Algorithmic Identity Proving - Phases 13-17 (shipped 2026-02-16)
- v1.3 Documentation & Vignettes - Phases 18-21 (shipped 2026-02-16)
- v1.4 Installation & Build Guide - Phases 22-23 (shipped 2026-02-17)
- v1.5 Interactive REPL - Phases 24-28 (shipped 2026-02-18)
- v1.6 CLI Hardening & Manual - Phases 29-32 (shipped 2026-02-18)
- v2.0 Maple Compatibility - Phases 33-40 (shipped 2026-02-20)
- **v3.0 Scripting & Bivariate Series - Phases 41-46 (in progress)**

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

<details>
<summary>v2.0 Maple Compatibility (Phases 33-40) - SHIPPED 2026-02-20</summary>

- [x] Phase 33: Symbolic Variable Foundation (3/3 plans) -- 2026-02-19
- [x] Phase 34: Product & Theta Signatures (2/2 plans) -- 2026-02-19
- [x] Phase 35: Series Analysis Signatures (2/2 plans) -- 2026-02-19
- [x] Phase 36: Relation Discovery Signatures (3/3 plans) -- 2026-02-19
- [x] Phase 37: New Functions - Theta & Jacobi (2/2 plans) -- 2026-02-19
- [x] Phase 38: New Functions - Analysis & Discovery (2/2 plans) -- 2026-02-19
- [x] Phase 39: Output & Compatibility (2/2 plans) -- 2026-02-19
- [x] Phase 40: Documentation (5/5 plans) -- 2026-02-20

See `.planning/milestones/v2.0-ROADMAP.md` for details.

</details>

### v3.0 Scripting & Bivariate Series (In Progress)

**Milestone Goal:** Every example in Garvan's "q-Product Tutorial" (qmaple.pdf) runs correctly in q-Kangaroo -- adding scripting language features (for-loops, procedures, conditionals), expression operations (series truncation, expansion, polynomial factoring, substitution), and bivariate (z,q)-series support for tripleprod/quinprod/winquist with symbolic z.

- [ ] **Phase 41: Control Flow Parsing** - Parser and AST support for for-loops, if/elif/else conditionals, boolean/comparison operators
- [ ] **Phase 42: Procedures & Evaluation** - Procedure definitions, local variables, RETURN, option remember, and control flow evaluation
- [ ] **Phase 43: Expression Operations** - series() truncation, expand(), runtime q-exponent arithmetic, floor(), legendre()
- [ ] **Phase 44: Polynomial Operations** - factor() cyclotomic/irreducible factoring, subs() variable substitution
- [ ] **Phase 45: Bivariate Series** - New Laurent-in-z-with-FPS-coefficients data type, tripleprod/quinprod/winquist with symbolic z, bivariate arithmetic
- [ ] **Phase 46: Documentation** - Manual chapter on scripting, help entries for new syntax, worked qmaple.pdf example reproductions

## Phase Details

### Phase 41: Control Flow Parsing
**Goal**: Users can write for-loops and if/elif/else conditionals that parse correctly into AST nodes with boolean and comparison operators
**Depends on**: Phase 40 (v2.0 complete)
**Requirements**: SCRIPT-01, SCRIPT-06, SCRIPT-07
**Success Criteria** (what must be TRUE):
  1. User can type `for n from 1 to 5 do print(n) od` and it parses without error into a ForLoop AST node with variable name, start/end bounds, and body statements
  2. User can type `if x > 0 then A elif x = 0 then B else C fi` and it parses into a conditional AST node with condition, then-branch, elif-branches, and else-branch
  3. User can use all six comparison operators (`=`, `<>`, `<`, `>`, `<=`, `>=`) and three boolean operators (`and`, `or`, `not`) in expressions, with correct precedence (not binds tighter than and, and binds tighter than or)
  4. For-loop and if/else blocks can contain multiple semicolon-separated statements in their bodies
**Plans**: 2 plans
Plans:
- [ ] 41-01-PLAN.md -- Tokens, AST types, lexer extensions, and parser comparison/boolean operator support
- [ ] 41-02-PLAN.md -- For-loop and if/elif/else parsing with statement sequences and REPL multiline detection

### Phase 42: Procedures & Evaluation
**Goal**: Users can define and call named procedures with local variables, early return, and memoization, and all control flow (for, if) evaluates correctly
**Depends on**: Phase 41
**Requirements**: SCRIPT-02, SCRIPT-03, SCRIPT-04, SCRIPT-05
**Success Criteria** (what must be TRUE):
  1. User can define `f := proc(n) local k; k := n*n; k; end` and call `f(5)` to get 25, with local variable `k` not leaking into global scope
  2. User can use `RETURN(value)` inside a procedure to exit early and produce a value
  3. User can add `option remember` to a procedure and observe that repeated calls with the same arguments return cached results
  4. For-loops evaluate correctly: `for n from 1 to 5 do n^2 od` iterates with the loop variable properly scoped
  5. If/elif/else/fi conditionals evaluate correctly: only the matching branch executes, boolean operators short-circuit
**Plans**: 2 plans
Plans:
- [ ] 42-01-PLAN.md -- Control flow evaluation (compare, bool, for, if) and RETURN/EarlyReturn support
- [ ] 42-02-PLAN.md -- Procedure parsing, definition, calling, local scoping, memoization, format, and REPL multiline

### Phase 43: Expression Operations
**Goal**: Users can truncate series, expand products, use runtime arithmetic in q-exponents, and compute floor/legendre
**Depends on**: Phase 42
**Requirements**: SERIES-01, SERIES-02, SERIES-03, UTIL-01, UTIL-02
**Success Criteria** (what must be TRUE):
  1. User can call `series(aqprod(q,q,50), q, 10)` and get a series truncated to O(q^10), regardless of the original truncation order
  2. User can call `expand(aqprod(q,q,5) * aqprod(q^2,q^2,5))` and get the product expanded into a single polynomial/series form
  3. User can write `for n from 0 to 4 do aqprod(q^(n*n), q, 20) od` and the `q^(n*n)` evaluates correctly for each integer n, including expressions like `q^(k*(3*k+1)/2)` where k is a loop variable
  4. User can call `floor(7/3)` to get 2 and `floor(-7/3)` to get -3 (standard mathematical floor)
  5. User can call `legendre(2, 5)` to get -1, matching the Legendre symbol (2/5)
**Plans**: 2 plans
Plans:
- [ ] 43-01-PLAN.md -- eval_pow Rational exponent arms, floor(), legendre(), L alias, help entries
- [ ] 43-02-PLAN.md -- series() truncation, expand() product expansion, help entries

### Phase 44: Polynomial Operations
**Goal**: Users can factor polynomials in q and substitute values into expressions
**Depends on**: Phase 43
**Requirements**: POLY-01, POLY-02
**Success Criteria** (what must be TRUE):
  1. User can call `factor(1 - q^6)` and get a factored form showing cyclotomic factors like `(1-q)(1+q)(1-q+q^2)(1+q+q^2)`
  2. User can call `factor()` on a polynomial produced by series computation and get meaningful irreducible factors over the rationals
  3. User can call `subs(q=1, series_expr)` to evaluate a series at q=1 (getting a rational number), and `subs(q=q^2, expr)` to transform q-exponents
**Plans**: 2 plans
Plans:
- [ ] 44-01-PLAN.md -- Core cyclotomic/factor modules, CLI factor() dispatch, help, completion
- [ ] 44-02-PLAN.md -- subs() AST interception, substitution logic, help, completion

### Phase 45: Bivariate Series
**Goal**: Users can compute tripleprod, quinprod, and winquist with symbolic z variables, getting Laurent polynomials in z with q-series coefficients, and perform arithmetic on these bivariate values
**Depends on**: Phase 41 (control flow for testing), Phase 43 (expression operations)
**Requirements**: BIVAR-01, BIVAR-02, BIVAR-03, BIVAR-04
**Success Criteria** (what must be TRUE):
  1. User can call `tripleprod(z, q, 10)` with z as a symbolic variable and get output displaying a Laurent polynomial in z where each coefficient is a q-series truncated to O(q^10)
  2. User can call `quinprod(z, q, 10)` with symbolic z and get the quintuple product as a bivariate Laurent polynomial matching Garvan's output format
  3. User can call `winquist(a, b, q, 10)` with symbolic a, b and get the Winquist product as a bivariate expression in a, b with q-series coefficients
  4. User can add, subtract, multiply, and negate bivariate series values and get correct bivariate results
**Plans**: 4 plans
Plans:
- [x] 45-01-PLAN.md -- BivariateSeries core struct, arithmetic, Value variant, display formatting
- [x] 45-02-PLAN.md -- tripleprod/quinprod bivariate dispatch via sum-form identities, help updates
- [x] 45-03-PLAN.md -- winquist one-symbolic bivariate dispatch via direct Pochhammer factors, help updates
- [ ] 45-04-PLAN.md -- Gap closure: winquist two-symbolic via TrivariateSeries, cross-validation

### Phase 46: Documentation
**Goal**: All new v3.0 features are documented in the PDF manual, help system, and worked examples reproducing Garvan's tutorial
**Depends on**: Phases 41-45
**Requirements**: DOC-01, DOC-02, DOC-03
**Success Criteria** (what must be TRUE):
  1. PDF manual contains a "Scripting Language" chapter documenting for-loop syntax, if/elif/else/fi syntax, proc/end definitions, local variables, option remember, RETURN, and boolean/comparison operators with runnable examples
  2. User can type `help for`, `help proc`, `help if`, `help series`, `help factor`, `help subs` in the REPL and get syntax documentation with examples
  3. Worked examples section includes at least 3 reproductions of key examples from Garvan's qmaple.pdf tutorial, demonstrating for-loops with series computation, procedure definitions with memoization, and bivariate product identities
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 41 -> 42 -> 43 -> 44 -> 45 -> 46

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
| 37. New Functions - Theta & Jacobi | v2.0 | 2/2 | Complete | 2026-02-19 |
| 38. New Functions - Analysis & Discovery | v2.0 | 2/2 | Complete | 2026-02-19 |
| 39. Output & Compatibility | v2.0 | 2/2 | Complete | 2026-02-19 |
| 40. Documentation | v2.0 | 5/5 | Complete | 2026-02-20 |
| 41. Control Flow Parsing | v3.0 | 2/2 | Complete | 2026-02-20 |
| 42. Procedures & Evaluation | v3.0 | 2/2 | Complete | 2026-02-20 |
| 43. Expression Operations | v3.0 | 2/2 | Complete | 2026-02-20 |
| 44. Polynomial Operations | v3.0 | 0/2 | Not started | - |
| 45. Bivariate Series | v3.0 | 3/4 | Gap closure | - |
| 46. Documentation | v3.0 | 0/TBD | Not started | - |
