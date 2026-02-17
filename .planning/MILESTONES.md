# Milestones

## v1.0: Core Engine (Complete)

**Shipped:** 2026-02-14
**Phases:** 1-8 (32 plans, 578 Rust tests, 9 Python integration tests)

### What Shipped

| Phase | Capability |
|-------|-----------|
| 1 | Expression foundation — hash-consed arena, arbitrary precision, LaTeX/Unicode rendering |
| 2 | FPS engine, simplification engine, infinite product generators |
| 3 | q-Pochhammer, q-binomials, named products (eta/Jacobi/triple/quintuple/Winquist), theta functions, partitions, rank/crank |
| 4 | prodmake (Andrews' algorithm), qfactor, sift, etamake/jacprodmake/mprodmake/qetamake, relation discovery (12+ functions) |
| 5 | Python API via PyO3 — 38 DSL functions, QSession/QExpr/QSeries, batch generation |
| 6 | Hypergeometric series — eval_phi/psi, 6 summation formulas, Heine/Sears/Watson/Bailey transforms |
| 7 | Identity proving — eta quotients, cusps, valence formula, TOML identity database |
| 8 | Mock theta (20 functions), Appell-Lerch sums, Bailey pairs/lemma/chains/discovery |

### Key Metrics

- 73 Python DSL functions across 10 groups
- Full Garvan `qseries` parity plus extensions
- UAT: 47/47 tests passed across all 8 phases (1 issue found and fixed)

### Key Decisions

See STATE.md Accumulated Context for full decision log.

## v1.1: Polish & Publish (Complete)

**Shipped:** 2026-02-15
**Phases:** 9-12 (10 plans, 578 Rust tests + 9 Python integration tests)

### What Shipped

| Phase | Capability |
|-------|-----------|
| 9 | Package rename: qsymbolic -> q_kangaroo throughout codebase, zero test regressions |
| 10 | PyPI packaging: ABI3 wheels (cp39-abi3), type stubs (.pyi), LICENSE (MIT), CITATION.cff, DLL bundling |
| 11 | CI/CD: GitHub Actions CI (Rust + Python tests + Codecov), release workflow (Linux/Windows wheels + OIDC PyPI publish) |
| 12 | Documentation & UX: Sphinx docs site (Furo theme, 13 API pages, 5 example notebooks), 73 NumPy-style docstrings, QSeries LaTeX rendering, error message improvements |

### Key Accomplishments

- `pip install q-kangaroo` ready with cross-platform wheels (Linux manylinux2014 + Windows MinGW)
- Complete Sphinx documentation site with API reference for all 73 functions and 5 narrative examples
- Every function has NumPy-style docstring with parameters, returns, examples, and LaTeX math
- QExpr and QSeries render LaTeX in Jupyter notebooks via `_repr_latex_()`
- Zero-token PyPI publishing via OIDC trusted publishing
- README with working quickstart example and Codecov coverage badge

### Key Metrics

- 26 v1.1 requirements: 26/26 complete
- 11,331 lines Rust core + 3,881 lines Python API + 5,862 lines docs
- Total project: 44 plans across 12 phases (v1.0 + v1.1)

## v1.2: Algorithmic Identity Proving (Complete)

**Shipped:** 2026-02-16
**Phases:** 13-17 (14 plans, 836 Rust tests + 9 Python integration tests)

### What Shipped

| Phase | Capability |
|-------|-----------|
| 13 | Polynomial infrastructure — QRatPoly (dense), GCD (subresultant PRS), resultant, q-shift, QRatRationalFunc with auto-simplification |
| 14 | q-Gosper algorithm — term ratio extraction, q-dispersion, qGFF decomposition, key equation solver, complete indefinite summation |
| 15 | q-Zeilberger & WZ certificates — creative telescoping, recurrence output, WZ certificate extraction, independent verification, FPS cross-check |
| 16 | Extensions — q-Petkovsek recurrence solver, Chen-Hou-Mu nonterminating proofs, BFS transformation chain discovery |
| 17 | Python API — 6 new DSL functions (q_zeilberger, verify_wz, q_petkovsek, prove_nonterminating, find_transformation_chain + q_gosper fix), Sphinx summation.rst |

### Key Accomplishments

- Machine-prove q-hypergeometric identities end-to-end (q-Zeilberger recurrence + WZ certificate)
- Nonterminating identity proofs via parameter specialization (Chen-Hou-Mu method)
- BFS transformation chain discovery over 5 transforms (Heine 1/2/3, Sears, Watson)
- q-Petkovsek solver finds Pochhammer closed forms for recurrences
- Polynomial infrastructure enabling all algorithms (GCD, resultant, rational functions)
- 79 Python DSL functions in 13 groups with complete Sphinx documentation

### Key Metrics

- 25 v1.2 requirements: 25/25 complete
- 18,686 lines Rust core + 4,866 lines Python API
- +7,355 lines Rust, +985 lines Python over v1.1
- +258 new Rust tests (578 -> 836)
- Total project: 54 plans across 17 phases (v1.0 + v1.1 + v1.2)
- Execution: 14 plans in ~124 min (avg 9 min/plan)

## v1.3: Documentation & Vignettes (Complete)

**Shipped:** 2026-02-16
**Phases:** 18-21 (12 plans)

### What Shipped

| Phase | Capability |
|-------|-----------|
| 18 | Docstring enrichment — all 79 DSL functions upgraded to research-quality with verified examples, DLMF references, cross-references, and mathematical notes |
| 19 | Vignette expansion — 5 existing notebooks expanded from demos to comprehensive tutorials (partition congruences, theta identities, hypergeometric summation, mock theta functions, Bailey chains) |
| 20 | New vignettes — 4 new notebooks created (getting started, series analysis, identity proving, Maple migration with 13-group translation guide) |
| 21 | Sphinx site polish — audience-aware landing page, "Which Function Should I Use?" decision guide (79 functions), cross-links from all 13 API pages to relevant notebooks |

### Key Accomplishments

- Enriched all 79 function docstrings with research-quality examples and mathematical context
- Expanded 5 existing notebooks from introductory demos to comprehensive tutorials
- Created 4 new notebooks covering newcomer onboarding, analysis pipeline, identity proving workflow, and Maple migration
- Built audience-aware Sphinx navigation with function decision guide and full API-to-notebook cross-linking
- 9 total notebooks covering all 13 function groups with pre-computed outputs

### Key Metrics

- 16 v1.3 requirements: 16/16 complete
- 209 files modified, +41,130 lines
- 12 plans in ~70 min (avg 6 min/plan)
- Total project: 66 plans across 21 phases (v1.0 + v1.1 + v1.2 + v1.3)

## v1.4: Installation & Build Guide (Complete)

**Shipped:** 2026-02-17
**Phases:** 22-23 (4 plans)

### What Shipped

| Phase | Capability |
|-------|-----------|
| 22 | Installation documentation -- INSTALL.md (236 lines) with pip install, Linux/Cygwin build-from-source, and 6-entry troubleshooting; installation.rst (329 lines) Sphinx mirror with RST directives |
| 23 | Verification & cross-references -- check_install.py (238 lines) with 4 end-user + 5 --dev checks; README.md and index.rst cross-references to installation guide |

### Key Accomplishments

- Complete INSTALL.md covering pip install, Linux build, Cygwin/Windows build (MinGW GMP + GNU Rust target), and troubleshooting
- Sphinx installation.rst mirror with 31 code-block directives, note/warning/tip admonitions
- check_install.py verification script with colored pass/fail output for 9 checks (4 end-user + 5 --dev)
- README.md and Sphinx landing page cross-reference installation guide from all entry points
- Fixed __init__.py stale import names (5 Phase 16 functions with _fn suffixes)

### Key Metrics

- 9 v1.4 requirements: 9/9 complete
- 17 commits, 20 files modified, +2,041 lines
- 4 plans in ~8 min (avg 2 min/plan)
- Total project: 70 plans across 23 phases (v1.0 + v1.1 + v1.2 + v1.3 + v1.4)
