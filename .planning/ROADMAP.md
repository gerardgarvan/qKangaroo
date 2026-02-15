# Roadmap: q-Kangaroo

## Milestones

- v1.0 Core Engine - Phases 1-8 (shipped 2026-02-14)
- v1.1 Polish & Publish - Phases 9-12 (in progress)

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

<details>
<summary>v1.0 Core Engine (Phases 1-8) - SHIPPED 2026-02-14</summary>

### Phase 1: Expression Foundation
**Goal**: Researchers can create, store, and display arbitrary q-series expressions with exact arithmetic
**Plans**: 3/3 complete

Plans:
- [x] 01-01: Workspace scaffold, Expr enum, ExprArena with hash-consing
- [x] 01-02: BigInt/BigRat arithmetic edge case verification
- [x] 01-03: LaTeX and Unicode rendering for all expression types

### Phase 2: Simplification & Series Engine
**Goal**: Expressions can be simplified via rewrite rules and expanded into formal power series
**Plans**: 3/3 complete

Plans:
- [x] 02-01: FormalPowerSeries data structure and series arithmetic
- [x] 02-02: Phased simplification engine with 4 rule phases
- [x] 02-03: Lazy infinite product generators with Euler function verification

### Phase 3: Core q-Series & Partitions
**Goal**: Researchers can compute q-Pochhammer symbols, named products, theta functions, and partition functions matching Garvan's qseries output
**Plans**: 4/4 complete

Plans:
- [x] 03-01: QMonomial, PochhammerOrder, aqprod, qbin
- [x] 03-02: Named products: etaq, jacprod, tripleprod, quinprod, winquist
- [x] 03-03: Theta functions: theta2, theta3, theta4
- [x] 03-04: Partition functions and rank/crank

### Phase 4: Series Analysis
**Goal**: Researchers can convert between series and product representations, factor q-series, and discover algebraic relations
**Plans**: 7/7 complete

Plans:
- [x] 04-01: Andrews' algorithm (prodmake)
- [x] 04-02: q-polynomial factoring (qfactor) and utilities
- [x] 04-03: Rational linear algebra
- [x] 04-04: Series-to-product post-processing (etamake, jacprodmake, mprodmake)
- [x] 04-05: Core relation discovery (findlincombo, findhom, findpoly)
- [x] 04-06: Congruence discovery and combo variants
- [x] 04-07: Modular relation discovery and search

### Phase 5: Python API
**Goal**: Researchers can use q-Kangaroo from Python with natural syntax, LaTeX display, and batch computation
**Plans**: 4/4 complete

Plans:
- [x] 05-01: Scaffold qsym-python crate, validate cdylib+GMP build
- [x] 05-02: Core QSession + QExpr with operators and rendering
- [x] 05-03: QSeries wrapper and DSL functions
- [x] 05-04: Batch computation and integration test

### Phase 6: Hypergeometric Series
**Goal**: Researchers can construct, evaluate, and transform basic hypergeometric series
**Plans**: 4/4 complete

Plans:
- [x] 06-01: HypergeometricSeries struct, eval_phi, eval_psi
- [x] 06-02: Summation formulas
- [x] 06-03: Heine's transformation and Sears' 4phi3
- [x] 06-04: Watson's and Bailey's transformations, Python bindings

### Phase 7: Identity Proving
**Goal**: Researchers can prove q-series identities automatically using the valence formula method
**Plans**: 4/4 complete

Plans:
- [x] 07-01: JAC and ETA symbolic models
- [x] 07-02: Cusp computation and order-at-cusp formulas
- [x] 07-03: Proving engine: provemodfuncid
- [x] 07-04: Identity database and Python bindings

### Phase 8: Mock Theta & Bailey Chains
**Goal**: Researchers can work with mock theta functions, Zwegers completions, Appell-Lerch sums, and Bailey chain machinery
**Plans**: 4/4 complete

Plans:
- [x] 08-01: All 20 classical mock theta functions
- [x] 08-02: Appell-Lerch sums, universal mock theta g2/g3, Zwegers completion
- [x] 08-03: Bailey pair database, lemma, chain iteration, weak Bailey lemma
- [x] 08-04: Automated Bailey pair discovery and Python bindings

</details>

### v1.1 Polish & Publish (In Progress)

**Milestone Goal:** Make q-Kangaroo release-ready -- publishable on PyPI with documentation, CI, and polished UX so researchers can discover, install, and use it without building from source.

- [x] **Phase 9: Package Rename & Structure** - Rename qsymbolic to q_kangaroo throughout the codebase with zero test regressions
- [x] **Phase 10: PyPI Packaging & Metadata** - Complete package metadata, ABI3 wheels, type stubs, and citation file
- [x] **Phase 11: CI/CD Pipeline** - GitHub Actions for testing, wheel building, coverage, and trusted PyPI publishing
- [ ] **Phase 12: Documentation & UX Polish** - Documentation site, API reference, examples, Jupyter rendering, and API ergonomics

## Phase Details

### Phase 9: Package Rename & Structure
**Goal**: The codebase uses the final public name (q_kangaroo) everywhere, and all existing functionality continues working without regressions
**Depends on**: Phase 8 (v1.0 complete)
**Requirements**: REN-01, REN-02, REN-03, REN-04, REN-05
**Success Criteria** (what must be TRUE):
  1. `import q_kangaroo` succeeds in a Python session after `maturin develop`
  2. All 578 Rust tests pass with `cargo test` after the rename
  3. All 9 Python integration tests pass with the new `q_kangaroo` import name
  4. The compiled shared library is named `_q_kangaroo` (with underscore prefix) and loads correctly
  5. No references to the old name `qsymbolic` remain in source files, configs, or test code
**Plans**: 2 plans

Plans:
- [x] 09-01-PLAN.md -- Atomic rename of all source files, directory, and Rust build verification
- [x] 09-02-PLAN.md -- Python rebuild, integration test verification, and PROJECT.md cleanup

### Phase 10: PyPI Packaging & Metadata
**Goal**: The package is ready for PyPI upload with complete metadata, cross-version wheels, type hints, and academic citation support
**Depends on**: Phase 9
**Requirements**: PKG-01, PKG-02, PKG-03, PKG-04, PKG-05, PKG-06, PKG-07
**Success Criteria** (what must be TRUE):
  1. `pip install q-kangaroo` from a locally built wheel succeeds on a fresh Windows virtualenv and `import q_kangaroo` works
  2. `pip show q-kangaroo` displays complete metadata (author, license, description, classifiers, project URLs, keywords)
  3. A single wheel file supports Python 3.9 through 3.14+ (ABI3 filename pattern: `*-cp39-abi3-*.whl`)
  4. IDE autocomplete shows function signatures and docstrings for all 73 DSL functions (type stubs present and valid)
  5. LICENSE and CITATION.cff files exist
**Plans**: 2 plans

Plans:
- [x] 10-01-PLAN.md -- Metadata, ABI3, LICENSE, CITATION.cff, and DLL loading configuration
- [x] 10-02-PLAN.md -- Type stubs (.pyi), wheel build, and end-to-end installation verification

### Phase 11: CI/CD Pipeline
**Goal**: Every push triggers automated testing and wheel builds, and tagged releases publish to PyPI without manual intervention
**Depends on**: Phase 10
**Requirements**: CI-01, CI-02, CI-03, CI-04, CI-05, CI-06, CI-07
**Success Criteria** (what must be TRUE):
  1. Pushing a commit to any branch triggers Rust tests and Python integration tests, with results visible on the PR
  2. CI produces manylinux2014 wheels for Linux and MinGW wheels for Windows on every push
  3. Test coverage percentage is reported and displayed as a badge in the README
  4. Pushing a version tag (e.g., `v1.1.0`) triggers an automated release that uploads wheels and sdist to PyPI
  5. PyPI publishing uses OIDC trusted publishing (no API tokens stored in repository secrets)
**Plans**: 2 plans

Plans:
- [x] 11-01-PLAN.md -- CI workflow: Rust tests, Python tests, coverage with Codecov badge
- [x] 11-02-PLAN.md -- Release workflow: Linux/Windows wheel builds, sdist, OIDC PyPI publishing

### Phase 12: Documentation & UX Polish
**Goal**: Researchers can discover, learn, and productively use q-Kangaroo through comprehensive documentation, polished Jupyter integration, and Pythonic API conventions
**Depends on**: Phase 11 (needs working CI and installable package)
**Requirements**: DOC-01, DOC-02, DOC-03, DOC-04, DOC-05, DOC-06, DOC-07, UX-01, UX-02, UX-03, UX-04, UX-05
**Success Criteria** (what must be TRUE):
  1. README contains installation instructions (`pip install q-kangaroo`), a working quickstart example, and a verification command that a new user can follow end-to-end
  2. A Sphinx documentation site is live on GitHub Pages with rendered API reference for all 73 functions, a getting-started guide, and an example gallery with at least 5 narrative examples
  3. Every Python function has a NumPy-style docstring with parameters, return type, mathematical notation (LaTeX), and at least one usage example
  4. QExpr and QSeries objects display rendered LaTeX in Jupyter notebooks and readable text in terminal sessions
  5. Functions accept sensible defaults (e.g., default truncation order, optional session), use snake_case with keyword arguments, and produce error messages that name the function and suggest corrections
**Plans**: 4 plans

Plans:
- [ ] 12-01-PLAN.md -- UX polish: QSeries LaTeX rendering, get_default_session(), README expansion
- [ ] 12-02-PLAN.md -- NumPy-style docstrings for all 73 DSL functions + error message improvements
- [ ] 12-03-PLAN.md -- Sphinx documentation site scaffold with API reference and guides
- [ ] 12-04-PLAN.md -- Example notebooks (5 narratives) and docs CI workflow for GitHub Pages

## Progress

**Execution Order:**
Phases execute in numeric order: 9 -> 10 -> 11 -> 12

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
| 12. Documentation & UX Polish | v1.1 | 0/4 | Not started | - |
