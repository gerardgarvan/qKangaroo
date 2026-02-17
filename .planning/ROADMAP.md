# Roadmap: q-Kangaroo

## Milestones

- v1.0 Core Engine - Phases 1-8 (shipped 2026-02-14)
- v1.1 Polish & Publish - Phases 9-12 (shipped 2026-02-15)
- v1.2 Algorithmic Identity Proving - Phases 13-17 (shipped 2026-02-16)
- v1.3 Documentation & Vignettes - Phases 18-21 (in progress)

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

### v1.3 Documentation & Vignettes

- [x] Phase 18: Docstring Enrichment (4/4 plans) -- 2026-02-16
- [x] Phase 19: Vignette Expansion (3/3 plans) -- 2026-02-16
- [x] Phase 20: New Vignettes & Migration Guide (3/3 plans) -- 2026-02-16
- [ ] Phase 21: Sphinx Site Polish

#### Phase 18: Docstring Enrichment

**Goal:** Upgrade all 79 function docstrings to research-quality with realistic mathematical examples, cross-references, and mathematical notes.

**Requirements:** DOC-01, DOC-02, DOC-03

**Plans:** 4 plans

Plans:
- [ ] 18-01-PLAN.md -- Enrich Groups 1-4 (Pochhammer, Named Products, Theta, Partitions)
- [ ] 18-02-PLAN.md -- Enrich Groups 5-7 (Utilities/Prodmake, Relation Discovery exact, Relation Discovery modular)
- [ ] 18-03-PLAN.md -- Enrich Groups 8-10 (Hypergeometric, Identity Proving, Mock Theta/Appell-Lerch/Bailey)
- [ ] 18-04-PLAN.md -- Enrich Groups 11-13 (q-Gosper, Algorithmic Summation, Extensions) + final validation

**Scope:**
- Enrich all 79 function examples in dsl.rs with meaningful mathematical use cases (not toy inputs)
- Add "See Also" cross-references between related functions across all 13 groups
- Add "Notes" sections with mathematical background where needed (theta, mock theta, Bailey, hypergeometric functions)
- Update .pyi type stubs to match enriched docstrings

**Dependencies:** None (can start immediately)

#### Phase 19: Vignette Expansion

**Goal:** Expand existing 5 notebooks from introductory demos to comprehensive tutorials covering all relevant functions in each topic area.

**Requirements:** DOC-04, DOC-05, DOC-06, DOC-07, DOC-08

**Plans:** 3 plans

Plans:
- [ ] 19-01-PLAN.md -- Expand partition_congruences.ipynb (rank/crank, prodmake, Dyson) and theta_identities.ipynb (triple product, quintuple, Winquist)
- [ ] 19-02-PLAN.md -- Expand hypergeometric_summation.ipynb (all summation formulas, all Heine transforms, bilateral psi)
- [ ] 19-03-PLAN.md -- Expand mock_theta_functions.ipynb (Appell-Lerch, g2/g3, orders) and bailey_chains.ipynb (multi-step chains, R-R, discover)

**Scope:**
- partition_congruences.ipynb: Add rank/crank, prodmake analysis, Dyson's conjecture
- theta_identities.ipynb: Add Jacobi triple product, quintuple product, Winquist, relationships
- hypergeometric_summation.ipynb: All 6 summation formulas, all Heine transforms, Sears/Watson
- mock_theta_functions.ipynb: Appell-Lerch connection, g2/g3, order comparisons
- bailey_chains.ipynb: Multi-step chains, Rogers-Ramanujan from unit pair, bailey_discover

**Dependencies:** Phase 18 (enriched docstrings referenced from notebooks)

#### Phase 20: New Vignettes & Migration Guide

**Goal:** Create 4 new notebooks covering gaps: newcomer onboarding, series analysis workflow, identity proving workflow, and Maple migration.

**Requirements:** DOC-09, DOC-10, DOC-11, DOC-12, DOC-13

**Plans:** 3 plans

Plans:
- [ ] 20-01-PLAN.md -- Create getting_started.ipynb (newcomer onboarding) and series_analysis.ipynb (analysis pipeline)
- [ ] 20-02-PLAN.md -- Create identity_proving.ipynb (algorithmic proving workflow)
- [ ] 20-03-PLAN.md -- Create maple_migration.ipynb (13-group migration guide) and update index.rst

**Scope:**
- getting_started.ipynb: Zero to first identity for newcomers (install -> session -> q-Pochhammer -> products -> identity)
- series_analysis.ipynb: prodmake -> etamake -> sift -> findlincombo/findhom/findpoly -> findcong pipeline
- identity_proving.ipynb: q-Zeilberger -> WZ certificate -> verification -> q-Petkovsek -> prove_nonterminating
- maple_migration.ipynb: Side-by-side Maple->Python for all 13 function groups, 30+ common operations

**Dependencies:** Phase 18 (docstrings ready for notebook cross-references)

#### Phase 21: Sphinx Site Polish

**Goal:** Polish the Sphinx documentation site with improved navigation, cross-linking, and a decision guide for function selection.

**Requirements:** DOC-14, DOC-15, DOC-16

**Scope:**
- Rewrite index.rst with audience-aware navigation ("New to q-series?", "Switching from Maple?", "Looking for a specific function?")
- Add cross-links from each API page to relevant vignettes
- Create function_guide.rst: "Which function should I use?" decision page organized by task type
- Update examples/index.rst with descriptions and audience tags for all 9 notebooks

**Dependencies:** Phases 19-20 (all notebooks must exist before cross-linking)

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
| 21. Sphinx Site Polish | v1.3 | 0/? | Planned | - |
