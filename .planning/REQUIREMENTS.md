# Requirements: v1.3 Documentation & Vignettes

**Goal:** Every function in q-Kangaroo is well documented with research-quality examples and narrative vignettes showing how to use them. Both Maple-switching researchers and newcomers to q-series can learn the library effectively.

## Docstring Enrichment

- [ ] **DOC-01**: Enrich all 79 function docstrings with realistic mathematical examples (not just toy inputs), including expected output and mathematical context
- [ ] **DOC-02**: Add "See Also" cross-references between related functions in docstrings (e.g., etaq ↔ prodmake, partition_gf ↔ sift ↔ findcong)
- [ ] **DOC-03**: Add "Notes" sections with mathematical background to functions that need it (e.g., theta functions, mock theta functions, Bailey pairs)

## Revised Core Vignettes

- [ ] **DOC-04**: Expand partition_congruences.ipynb — add rank/crank generating functions, Dyson's conjecture verification, prodmake analysis of partition generating functions
- [ ] **DOC-05**: Expand theta_identities.ipynb — add Jacobi triple product derivation, quintuple product, Winquist's identity, theta function relationships
- [ ] **DOC-06**: Expand hypergeometric_summation.ipynb — demonstrate all 6 summation formulas (q-Gauss, q-Vandermonde, q-Saalschutz, q-Kummer, q-Dixon), all Heine transforms, Sears/Watson transforms
- [ ] **DOC-07**: Expand mock_theta_functions.ipynb — connect to Appell-Lerch sums, demonstrate g2/g3 universal functions, add third/fifth/seventh order comparisons
- [ ] **DOC-08**: Expand bailey_chains.ipynb — show multi-step chain construction, derive Rogers-Ramanujan identities from unit pair, demonstrate bailey_discover

## New Vignettes

- [ ] **DOC-09**: Create Getting Started tutorial notebook — zero to first identity for newcomers: install, session, basic q-Pochhammer, first product, first identity verification
- [ ] **DOC-10**: Create Series Analysis workflow notebook — prodmake → etamake → jacprodmake pipeline, sift for subsequences, findlincombo/findhom/findpoly for relation discovery, findcong for congruences
- [ ] **DOC-11**: Create Identity Proving workflow notebook — q-Zeilberger recurrence, WZ certificate extraction and verification, q-Petkovsek closed forms, prove_nonterminating for infinite series, find_transformation_chain

## Maple Migration Guide

- [ ] **DOC-12**: Create maple_migration.ipynb — side-by-side mapping of Garvan's qseries/thetaids/ETA functions to q_kangaroo equivalents with code comparison
- [ ] **DOC-13**: Cover all 13 function groups in migration guide, with Maple→Python translation for at least 30 common operations

## Sphinx Site Polish

- [ ] **DOC-14**: Rewrite Sphinx landing page (index.rst) with structured overview, audience pointers ("New to q-series?", "Switching from Maple?"), and clear navigation paths
- [ ] **DOC-15**: Add cross-links from API reference pages to relevant vignettes (e.g., partition functions → partition_congruences notebook)
- [ ] **DOC-16**: Create "Which function should I use?" decision guide page with function selection flowcharts by task type

## Traceability

| Requirement | Area | Phase |
|-------------|------|-------|
| DOC-01 | Docstrings | Phase 18 |
| DOC-02 | Docstrings | Phase 18 |
| DOC-03 | Docstrings | Phase 18 |
| DOC-04 | Vignette revision | Phase 19 |
| DOC-05 | Vignette revision | Phase 19 |
| DOC-06 | Vignette revision | Phase 19 |
| DOC-07 | Vignette revision | Phase 19 |
| DOC-08 | Vignette revision | Phase 19 |
| DOC-09 | New vignette | Phase 20 |
| DOC-10 | New vignette | Phase 20 |
| DOC-11 | New vignette | Phase 20 |
| DOC-12 | Maple migration | Phase 20 |
| DOC-13 | Maple migration | Phase 20 |
| DOC-14 | Sphinx polish | Phase 21 |
| DOC-15 | Sphinx polish | Phase 21 |
| DOC-16 | Sphinx polish | Phase 21 |

**Coverage:** 16 requirements across 5 areas
