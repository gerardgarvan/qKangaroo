# q-Kangaroo

## What This Is

An open-source symbolic computation engine for q-series, purpose-built to replace Frank Garvan's Maple packages (`qseries`, `thetaids`, `ETA`) and extend beyond them. Rust core engine with Python bindings (`q_kangaroo`) for the q-series research community — freeing researchers from the Maple dependency while providing the same rigor and expanding into areas Garvan's packages don't cover.

## Core Value

Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output — so researchers can switch without losing any capability.

## Current Milestone: v1.1 Polish & Publish

**Goal:** Make q-Kangaroo release-ready — publishable on PyPI with documentation, CI, and polished UX.

**Target features:**
- Rename package to q-kangaroo (PyPI) / q_kangaroo (import)
- PyPI packaging with wheels — `pip install q-kangaroo` just works
- Documentation site on GitHub Pages (API reference, getting-started, examples)
- GitHub Actions CI for Linux + Windows with test coverage
- UX polish: Jupyter rendering, API ergonomics, clear error messages

## Requirements

### Validated

<!-- Shipped and confirmed in v1.0. -->

- Garvan `qseries` parity: q-Pochhammer, q-binomials, hypergeometric series, partitions, relation discovery — v1.0 Phase 1-5
- Garvan `thetaids` parity: theta functions, Jacobi products, identity proving via valence formula — v1.0 Phase 3,7
- Garvan `ETA` parity: Dedekind eta, eta quotients, cusp computation, modular forms tools — v1.0 Phase 7
- Mock theta functions (20 classical), Appell-Lerch sums, Bailey chains/discovery — v1.0 Phase 8
- Hypergeometric series: eval_phi/psi, summation/transformation formulas — v1.0 Phase 6
- Python API: 73 DSL functions, QSession/QExpr/QSeries, batch generation — v1.0 Phase 5,8
- LaTeX and Unicode pretty-printing — v1.0 Phase 1

### Active

- [ ] PyPI package: `pip install q-kangaroo` with pre-built wheels
- [ ] Rename: `qsymbolic` → `q_kangaroo` throughout codebase
- [ ] Documentation site on GitHub Pages
- [ ] API reference with all 73 functions documented
- [ ] Getting-started guide with worked examples
- [ ] GitHub Actions CI (Linux + Windows)
- [ ] Test coverage reporting
- [ ] Jupyter rich rendering (LaTeX in notebooks)
- [ ] API ergonomics (sensible defaults, fewer required args)
- [ ] Clear error messages with helpful suggestions

### Future

- Algorithmic identity proving (WZ method, creative telescoping, q-Zeilberger)
- Quantum algebra (quantum groups, R-matrices, knot polynomial connections)
- Identity database expansion (~500+ verified identities with citations)
- macOS CI support
- Web playground / WASM
- Jupyter kernel / interactive widgets

### Out of Scope

- GUI / desktop application — CLI and library only
- Numerical-only computation — symbolic-first always
- Mobile app
- Paid hosting / SaaS

## Context

**v1.0 shipped:** Core engine complete with 578 Rust tests, 9 Python integration tests, 73 DSL functions across 10 groups. Full Garvan parity achieved plus extensions (mock theta, hypergeometric, Bailey chains). UAT verified all 8 phases (47/47 tests passed).

**What's needed now:** The engine works but isn't accessible. No PyPI package, no docs site, current module name is internal (`qsymbolic`). Researchers can't discover or install it without building from source.

**Verification strategy:** Package is release-ready when: `pip install q-kangaroo` works on fresh Linux/Windows, `import q_kangaroo` succeeds, docs site is live, CI is green.

## Constraints

- **Tech stack**: Rust (core engine) + Python (user API via PyO3), maturin for packaging
- **Package name**: `q-kangaroo` on PyPI, `q_kangaroo` for Python import
- **Platforms**: Linux + Windows (MinGW/GMP) for CI; macOS deferred
- **Correctness**: All existing 578 tests must continue passing through rename
- **Build**: Windows requires MinGW GCC + pre-built GMP at `C:/mingw64-gcc/mingw64/`

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + Python (not pure Python) | Performance-critical symbolic manipulation needs systems language; Python for accessibility | Good |
| Full Garvan parity before extensions | Ensures the tool is immediately useful as a Maple replacement before adding new capabilities | Good |
| Library-first, not Jupyter-first | Researchers need scriptable pipelines; notebook UX can layer on later | Good |
| Symbolic-first, numerical opt-in | Matches researcher expectations — exact results by default, approximation only when asked | Good |
| Package name: q-kangaroo | User-chosen name for public release | -- Pending |
| GitHub Pages for docs | Free, auto-deploy, good for project documentation | -- Pending |

---
*Last updated: 2026-02-14 after v1.1 milestone start*
