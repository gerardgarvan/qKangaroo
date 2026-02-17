# q-Kangaroo

## What This Is

An open-source symbolic computation engine for q-series, purpose-built to replace Frank Garvan's Maple packages (`qseries`, `thetaids`, `ETA`) and extend beyond them. Rust core engine (18,600+ lines) with Python bindings (`q_kangaroo`, 79 DSL functions in 13 groups) for the q-series research community -- freeing researchers from the Maple dependency while providing the same rigor and expanding into mock theta functions, Bailey chains, hypergeometric transformations, and machine-assisted identity proving that Garvan's packages don't cover.

## Core Value

Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.

## Current State

**v1.4 shipped.** The project now has complete installation documentation and verification tooling:
- `pip install q-kangaroo` works (Linux manylinux2014 + Windows MinGW wheels)
- `import q_kangaroo` provides 79 DSL functions across 13 groups
- All 79 functions have research-quality docstrings with verified examples, DLMF references, cross-references
- 9 comprehensive tutorial notebooks (partition congruences, theta identities, hypergeometric summation, mock theta, Bailey chains, getting started, series analysis, identity proving, Maple migration)
- Sphinx docs site with audience-aware navigation, function decision guide, API-to-notebook cross-linking
- GitHub Actions CI (Rust + Python tests, Codecov coverage, wheel builds, OIDC PyPI publishing)
- QExpr and QSeries render LaTeX in Jupyter notebooks
- q-Gosper, q-Zeilberger, WZ certificates for machine-proving q-hypergeometric identities
- q-Petkovsek recurrence solver, nonterminating proofs, transformation chain discovery

**Codebase:**
- 18,686 lines Rust core (`crates/qsym-core/src/`)
- 4,866 lines Python API (`crates/qsym-python/src/`)
- ~47,000 lines documentation (`docs/` including notebooks)
- 836 Rust tests, 9 Python integration tests
- 70 plans across 23 phases (v1.0 + v1.1 + v1.2 + v1.3 + v1.4)
- INSTALL.md (236 lines), check_install.py (238 lines)

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
- Polynomial infrastructure: QRatPoly, GCD, resultant, rational functions -- v1.2
- q-Gosper algorithm for indefinite q-hypergeometric summation -- v1.2
- q-Zeilberger creative telescoping with WZ proof certificates -- v1.2
- q-Petkovsek recurrence solver with Pochhammer closed-form output -- v1.2
- Chen-Hou-Mu nonterminating identity proofs -- v1.2
- BFS transformation chain discovery over Heine/Sears/Watson catalog -- v1.2
- Python API for all v1.2 algorithms (6 new DSL functions, Sphinx docs) -- v1.2
- Docstring enrichment: All 79 functions upgraded to research-quality with verified examples and DLMF references -- v1.3
- Maple migration guide: Side-by-side translation for all 13 function groups (35+ operations) -- v1.3
- Revised core vignettes: 5 notebooks expanded from demos to comprehensive tutorials -- v1.3
- New vignettes: Getting Started, Series Analysis, Identity Proving, Maple Migration notebooks -- v1.3
- Sphinx site polish: Audience-aware landing page, function decision guide, API-to-notebook cross-links -- v1.3
- Bulletproof installation instructions for pip-install users and build-from-source contributors -- v1.4
- INSTALL.md at repo root + integrated Sphinx installation.rst -- v1.4
- Build verification script (check_install.py) with end-user and --dev modes -- v1.4
- README.md and Sphinx landing page cross-reference installation guide -- v1.4

### Active

(No active requirements -- all milestones complete)

### Future

- Multi-sum creative telescoping (qMultiSum equivalent)
- Batch verification of identity database (Gasper-Rahman Appendix II)
- Human-readable proof output for publication
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
- General holonomic functions framework -- q-hypergeometric algorithms are sufficient
- q-Integration (Jackson q-integral) -- different algorithmic domain
- Automatic conjecture generation from data -- existing findlincombo/findhom/findpoly handle relation discovery

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
| Dense polynomial storage | Vec<QRat> ascending-degree simpler than sparse for algorithm needs | Good |
| Subresultant PRS for GCD | Prevents intermediate coefficient explosion vs naive Euclidean | Good |
| Direct term-value creative telescoping | Avoids polynomial key equation evaluation; handles terminating series | Good |
| BFS for transformation chains | Shortest-path guarantee; DFS could miss shorter paths | Good |
| Closure-from-template for prove_nonterminating | Declarative Python params, Rust builds closures; avoids FFI closure crossing | Good |

---
*Last updated: 2026-02-17 after v1.4 milestone complete*
