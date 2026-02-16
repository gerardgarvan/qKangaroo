# q-Kangaroo

## What This Is

An open-source symbolic computation engine for q-series, purpose-built to replace Frank Garvan's Maple packages (`qseries`, `thetaids`, `ETA`) and extend beyond them. Rust core engine (11,300+ lines) with Python bindings (`q_kangaroo`, 73 DSL functions) for the q-series research community -- freeing researchers from the Maple dependency while providing the same rigor and expanding into mock theta functions, Bailey chains, and hypergeometric transformations that Garvan's packages don't cover.

## Core Value

Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.

## Current State

**v1.1 shipped.** The project is release-ready:
- `pip install q-kangaroo` works (Linux manylinux2014 + Windows MinGW wheels)
- `import q_kangaroo` provides 73 DSL functions across 10 groups
- Sphinx documentation site with API reference, getting-started guide, 5 example notebooks
- GitHub Actions CI (Rust + Python tests, Codecov coverage, wheel builds, OIDC PyPI publishing)
- QExpr and QSeries render LaTeX in Jupyter notebooks
- All 73 functions have NumPy-style docstrings with mathematical notation

**Codebase:**
- 11,331 lines Rust core (`crates/qsym-core/src/`)
- 3,881 lines Python API (`crates/qsym-python/src/`)
- 5,862 lines documentation (`docs/`)
- 578 Rust tests, 9 Python integration tests
- 44 plans across 12 phases (v1.0 + v1.1)

## Requirements

### Validated

- Garvan `qseries` parity: q-Pochhammer, q-binomials, hypergeometric series, partitions, relation discovery -- v1.0
- Garvan `thetaids` parity: theta functions, Jacobi products, identity proving via valence formula -- v1.0
- Garvan `ETA` parity: Dedekind eta, eta quotients, cusp computation, modular forms tools -- v1.0
- Mock theta functions (20 classical), Appell-Lerch sums, Bailey chains/discovery -- v1.0
- Hypergeometric series: eval_phi/psi, 6 summation formulas, Heine/Sears/Watson/Bailey transforms -- v1.0
- Python API: 73 DSL functions, QSession/QExpr/QSeries, batch generation -- v1.0
- LaTeX and Unicode pretty-printing -- v1.0
- Package rename: qsymbolic -> q_kangaroo (Python), _q_kangaroo (native) -- v1.1
- PyPI packaging: ABI3 wheels, type stubs, LICENSE, CITATION.cff -- v1.1
- CI/CD: GitHub Actions CI + release workflow with OIDC PyPI publishing -- v1.1
- Documentation: Sphinx site, 73 NumPy-style docstrings, 5 example notebooks -- v1.1
- UX polish: Jupyter LaTeX rendering, get_default_session(), error messages -- v1.1

### Active

**v1.2 — Algorithmic Identity Proving:**
- q-Gosper's algorithm for indefinite q-hypergeometric summation
- q-Zeilberger's algorithm for definite q-hypergeometric sum recurrences
- Creative telescoping for multi-sum q-hypergeometric identities
- WZ proof certificates for identity verification
- Full Python API with DSL functions, docstrings, Sphinx docs for all new algorithms

### Future

- Quantum algebra (quantum groups, R-matrices, knot polynomial connections)
- Identity database expansion (~500+ verified identities with citations)
- macOS CI support
- Static GMP linking on Windows (eliminate DLL dependency)
- conda-forge recipe
- Web playground / WASM
- Versioned documentation

### Out of Scope

- GUI / desktop application -- CLI and library only
- Numerical-only computation -- symbolic-first always
- Mobile app
- Paid hosting / SaaS
- General-purpose CAS features -- focused q-series tool

## Constraints

- **Tech stack**: Rust (core engine) + Python (user API via PyO3 0.23), maturin for packaging
- **Package name**: `q-kangaroo` on PyPI, `q_kangaroo` for Python import
- **Platforms**: Linux + Windows (MinGW/GMP) for CI; macOS deferred
- **Build**: Windows requires MinGW GCC + pre-built GMP at `C:/mingw64-gcc/mingw64/`
- **ABI**: Stable ABI (abi3) for cross-version Python support (3.9+)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + Python (not pure Python) | Performance-critical symbolic manipulation needs systems language; Python for accessibility | Good |
| Full Garvan parity before extensions | Ensures the tool is immediately useful as a Maple replacement | Good |
| Library-first, not Jupyter-first | Researchers need scriptable pipelines; notebook UX layers on later | Good |
| Symbolic-first, numerical opt-in | Matches researcher expectations -- exact results by default | Good |
| Package name: q-kangaroo | User-chosen name for public release | Good |
| GitHub Pages for docs | Free, auto-deploy, Sphinx + Furo theme | Good |
| ABI3 via maturin features | Avoids Cargo.toml feature conflicts | Good |
| OIDC trusted publishing | Zero stored tokens, most secure PyPI publishing method | Good |
| Explicit session parameter | Better for reproducibility; get_default_session() for convenience | Good |
| Pre-executed notebooks | Avoids fragile CI notebook execution; nbsphinx_execute="never" | Good |

---
*Last updated: 2026-02-15 after v1.2 milestone started*
