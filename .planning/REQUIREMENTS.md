# Requirements: q-Kangaroo

**Defined:** 2026-02-13 (v1.0), updated 2026-02-14 (v1.1)
**Core Value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output — so researchers can switch without losing any capability.

## v1.0 Requirements (Complete)

All 62 v1.0 requirements shipped and validated. See MILESTONES.md for details.

## v1.1 Requirements

Requirements for Polish & Publish milestone. Each maps to roadmap phases.

### Package Rename

- [ ] **REN-01**: Python package imports as `q_kangaroo` (not `qsymbolic`)
- [ ] **REN-02**: PyPI package name is `q-kangaroo`
- [ ] **REN-03**: Rust cdylib module compiles as `_q_kangaroo`
- [ ] **REN-04**: All existing 578 Rust tests pass after rename
- [ ] **REN-05**: All existing 9 Python integration tests pass with new import name

### PyPI Packaging

- [ ] **PKG-01**: User can install with `pip install q-kangaroo` on Linux
- [ ] **PKG-02**: User can install with `pip install q-kangaroo` on Windows
- [ ] **PKG-03**: pyproject.toml has complete metadata (classifiers, keywords, URLs, description)
- [ ] **PKG-04**: ABI3 wheels support Python 3.9 through 3.14+
- [ ] **PKG-05**: LICENSE file included in source distribution
- [ ] **PKG-06**: Type stubs (.pyi) provide IDE autocomplete for all 73 functions
- [ ] **PKG-07**: CITATION.cff enables academic citation via GitHub and Zenodo

### CI/CD

- [ ] **CI-01**: GitHub Actions runs Rust tests on every push/PR
- [ ] **CI-02**: GitHub Actions runs Python integration tests on every push/PR
- [ ] **CI-03**: CI builds wheels on Linux (manylinux2014)
- [ ] **CI-04**: CI builds wheels on Windows (MinGW/GMP)
- [ ] **CI-05**: CI reports test coverage with badge in README
- [ ] **CI-06**: Release workflow publishes to PyPI on version tags
- [ ] **CI-07**: Trusted publishing via OIDC (no stored API tokens)

### Documentation

- [ ] **DOC-01**: README has installation instructions, quickstart example, and verification command
- [ ] **DOC-02**: Sphinx documentation site deployed on GitHub Pages
- [ ] **DOC-03**: API reference documents all 73 DSL functions with parameters, return types, and examples
- [ ] **DOC-04**: Getting-started guide walks through basic q-series computation
- [ ] **DOC-05**: Example gallery with 5+ narrative examples (partition congruences, theta identities, hypergeometric summation, mock theta, Bailey chains)
- [ ] **DOC-06**: NumPy-style docstrings on all 73 Python functions
- [ ] **DOC-07**: Mathematical notation (LaTeX) renders correctly in documentation

### UX Polish

- [ ] **UX-01**: QExpr and QSeries render LaTeX in Jupyter notebooks via `_repr_latex_()`
- [ ] **UX-02**: QExpr and QSeries have clear, readable `__repr__` for terminal use
- [ ] **UX-03**: Error messages include function name, expected argument types, and helpful suggestion
- [ ] **UX-04**: Functions have sensible defaults (e.g., default truncation order, default session)
- [ ] **UX-05**: API follows Pythonic conventions (snake_case, keyword arguments where appropriate)

## v2 Requirements

Deferred to future milestones. Tracked but not in current roadmap.

### Extended Platform Support

- **PLAT-01**: CI builds wheels on macOS (x86_64 and aarch64)
- **PLAT-02**: Static GMP linking on Windows (eliminate DLL dependency)
- **PLAT-03**: conda-forge recipe for conda install

### Extended Documentation

- **EDOC-01**: Interactive Binder links for try-without-installing
- **EDOC-02**: Performance benchmarks vs Maple
- **EDOC-03**: Versioned documentation (multiple doc versions on site)

### Extended UX

- **EUX-01**: Rich error rendering in Jupyter (LaTeX-formatted errors)
- **EUX-02**: Tab completion for common function patterns
- **EUX-03**: Progress bars for long computations

### Extended Features

- **FEAT-01**: WZ method (algorithmic identity proving)
- **FEAT-02**: Quantum algebra (quantum groups, R-matrices)
- **FEAT-03**: Identity database expansion to 500+ identities

## Out of Scope

| Feature | Reason |
|---------|--------|
| GUI / desktop application | Library-only distribution |
| Web playground / WASM | Defer until core is polished |
| Mobile app | Not relevant for research audience |
| Paid hosting / SaaS | Open-source project |
| PyO3 0.28 upgrade | Risk of API breakage; stay on 0.23 for v1.1 stability |
| macOS CI | Deferred to v2 to reduce scope |
| General-purpose CAS features | Focused q-series tool, not a Maple replacement for calculus |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| (populated by roadmap) | | |

**Coverage:**
- v1.1 requirements: 26 total
- Mapped to phases: 0
- Unmapped: 26

---
*Requirements defined: 2026-02-13 (v1.0)*
*Last updated: 2026-02-14 after v1.1 milestone definition*
