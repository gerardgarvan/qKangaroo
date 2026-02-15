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
