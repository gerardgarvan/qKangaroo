# Q-Symbolic

## What This Is

An open-source symbolic computation engine for q-series, purpose-built to replace Frank Garvan's Maple packages (`qseries`, `thetaids`, `ETA`) and extend beyond them. Rust core engine with Python bindings for the q-series research community — freeing researchers from the Maple dependency while providing the same rigor and expanding into areas Garvan's packages don't cover.

## Core Value

Every function in Garvan's Maple packages works correctly in Q-Symbolic, producing matching output — so researchers can switch without losing any capability.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

(None yet — ship to validate)

### Active

- [ ] Full parity with Garvan's `qseries` package (q-Pochhammer, q-binomials, basic hypergeometric series, series expansion, coefficient extraction)
- [ ] Full parity with Garvan's `thetaids` package (theta function identities, proving tools)
- [ ] Full parity with Garvan's `ETA` package (Dedekind eta, eta quotients, modular forms tools)
- [ ] Mock theta functions (Zwegers completions, Appell-Lerch sums, quantum modular forms)
- [ ] Algorithmic identity proving (WZ method, creative telescoping, q-Zeilberger)
- [ ] Bailey chains and pairs (database, chain iteration, automatic discovery)
- [ ] Quantum algebra (quantum groups, R-matrices, knot polynomial connections)
- [ ] Identity database (~500+ verified q-series identities with citations)
- [ ] Python library API for both interactive exploration and batch computation pipelines
- [ ] LaTeX and Unicode pretty-printing of all expressions

### Out of Scope

- Jupyter kernel / interactive widgets — library-first, notebook integration deferred
- Web playground / WASM — defer until core is solid
- GUI / desktop application — CLI and library only
- Numerical-only computation — symbolic-first always, numerical evaluation is opt-in

## Context

**The gap:** Frank Garvan's Maple packages (`qseries`, `thetaids`, `ETA`) are the de facto standard tools for q-series researchers. They work well, but they require a Maple license — a proprietary dependency that limits accessibility and reproducibility. No open-source equivalent exists with comparable coverage.

**Garvan's packages cover:**
- `qseries`: q-Pochhammer symbols, q-binomial coefficients, basic hypergeometric series (`_rφ_s`), series expansion, coefficient extraction, partition functions, generating functions, Heine/q-Gauss/q-Vandermonde transformations and summations
- `thetaids`: Jacobi theta function identities, tools for proving theta function relations
- `ETA`: Dedekind eta function, eta quotients, modular equations, connections to modular forms

**Extensions beyond Garvan:**
- Mock theta functions — Ramanujan's original functions plus Zwegers' completions to harmonic Maass forms, Appell-Lerch sums, universal mock theta function
- Wilf-Zeilberger method — algorithmic proof/verification of q-series identities, creative telescoping
- Bailey chains — systematic generation of new identities from known Bailey pairs
- Quantum algebra — quantum groups, quantum integers, connections to knot theory
- Identity database — curated, searchable collection of ~500+ verified identities with citations and proof methods

**Performance note:** Some mock theta function calculations may benefit from direct C GMP calls rather than going through Rust's `rug` bindings. Architecture should allow C FFI escape hatches for the hottest computation paths.

**Verification strategy:** Parity is achieved when every public function in Garvan's packages has a Q-Symbolic equivalent AND produces matching output on the same inputs. Cross-validate against Maple results.

## Constraints

- **Tech stack**: Rust (core engine) + Python (user API via PyO3), with C GMP escape hatch for performance-critical mock theta paths
- **Correctness**: All computations symbolically exact by default. Every identity must be verifiable by expanding both sides to O(q^50) minimum
- **Build order**: Full Garvan parity before extensions — qseries first, then thetaids, then ETA, then new capabilities
- **Interface**: Library-first — clean importable Python API for scripting and pipelines. Jupyter/notebook support deferred
- **References**: Every implemented identity must cite its source (Gasper & Rahman, Andrews, Fine, etc.)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + Python (not pure Python) | Performance-critical symbolic manipulation needs systems language; Python for accessibility | — Pending |
| C GMP escape hatch for mock theta | Some computations too hot for rug bindings abstraction overhead | — Pending |
| Full Garvan parity before extensions | Ensures the tool is immediately useful as a Maple replacement before adding new capabilities | — Pending |
| Library-first, not Jupyter-first | Researchers need scriptable pipelines; notebook UX can layer on later | — Pending |
| Symbolic-first, numerical opt-in | Matches researcher expectations — exact results by default, approximation only when asked | — Pending |

---
*Last updated: 2026-02-13 after initialization*
