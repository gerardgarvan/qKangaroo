# q-Kangaroo

## What This Is

An open-source symbolic computation engine for q-series, purpose-built to replace Frank Garvan's Maple packages (`qseries`, `thetaids`, `ETA`) and extend beyond them. Rust core engine (21,000+ lines) with Python bindings (`q_kangaroo`, 79 DSL functions in 13 groups) and an interactive REPL (`q-kangaroo` zero-dependency standalone executable, 14,000+ lines, 97 functions with Garvan-exact Maple syntax plus a scripting language) for the q-series research community -- freeing researchers from the Maple dependency while providing the same rigor and expanding into mock theta functions, Bailey chains, hypergeometric transformations, machine-assisted identity proving, bivariate series, and scripting that Garvan's packages don't cover.

## Core Value

Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- and every example in Garvan's "q-Product Tutorial" (qmaple.pdf) runs correctly -- so researchers can switch without losing any capability.

## Current State

**v3.0 shipped.** The project provides two access paths for researchers:

**Python API** (`pip install q-kangaroo`):
- 79 DSL functions across 13 groups with research-quality docstrings
- QSession/QExpr/QSeries with Jupyter LaTeX rendering
- 9 comprehensive tutorial notebooks
- Sphinx docs site with audience-aware navigation and function decision guide

**Interactive REPL** (`q-kangaroo` standalone executable):
- Zero-dependency standalone binary — static GMP/MPFR/MPC linking, no DLLs needed
- Maple-exact syntax — researchers copy-paste from Garvan's Maple worksheets
- 97 canonical function names with Garvan-compatible calling conventions
- Scripting language: for-loops, if/elif/else conditionals, procedures with local variables, memoization, early return
- Bivariate series: tripleprod/quinprod/winquist with symbolic z variables, Laurent polynomial display
- Expression operations: series() truncation, expand(), factor() cyclotomic factoring, subs() substitution
- Bare symbol variables, q-monomials as parameters, symbolic labels in output
- Script execution (`q-kangaroo script.qk`), pipe input, `-c` expression mode
- 7 distinct exit codes (sysexits-compatible) with filename:line:col error diagnostics
- Tab completion (97 functions + 18 keywords), 10-category help system, persistent history
- LaTeX output and save-to-file commands

**Documentation:**
- 97-function PDF reference manual (Typst) with formal mathematics and scripting chapter
- Workflow-oriented Maple migration guide with two-column comparison tables
- 9 worked examples with Garvan-canonical signatures, scholarly citations
- CI-compiled PDF included in GitHub release artifacts

**Infrastructure:**
- GitHub Actions CI (Rust + Python tests, Codecov, wheel builds, OIDC PyPI publishing, CLI binary + PDF releases)
- q-Gosper, q-Zeilberger, WZ certificates for machine-proving q-hypergeometric identities
- q-Petkovsek recurrence solver, nonterminating proofs, transformation chain discovery

**Codebase:**
- ~21,000 lines Rust core (`crates/qsym-core/src/`)
- ~14,000 lines CLI (`crates/qsym-cli/src/`)
- 6,320 lines Python API (`crates/qsym-python/src/`)
- ~5,000 lines Typst manual (`manual/`)
- ~47,000 lines documentation (`docs/` including notebooks)
- 879 Rust core tests + 772 CLI tests + 9 Python integration tests
- 133 plans across 46 phases (v1.0-v3.0)

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
- Interactive REPL: Maple-style parser, all 81 functions, tab completion, help system, LaTeX output, save-to-file -- v1.5
- Standalone executables: Windows (.exe + DLLs) and Linux binaries with CI release workflow -- v1.5
- Zero-dependency standalone .exe with static GMP/MPFR/MPC linking -- v1.6
- Script execution (`q-kangaroo script.qk`), pipe input, `-c` mode for non-interactive batch use -- v1.6
- Error hardening: 7 sysexits exit codes, filename:line:col diagnostics, panic translation -- v1.6
- 81-function PDF reference manual with formal math, worked examples, and Maple migration table -- v1.6
- CLI UX polish: --help, --quiet, --verbose, pipe support, read() in REPL -- v1.6
- Maple-exact function signatures: all qseries/thetaids functions callable with Garvan's argument conventions -- v2.0
- Symbolic variables: bare names, q-as-parameter, q-monomial arguments -- v2.0
- 8 new Garvan functions: theta, jac2prod, jac2series, qs2jaccombo, checkmult, checkprod, lqdegree0, findprod -- v2.0
- Symbolic labels in relation discovery output, findcong auto-discover algorithm -- v2.0
- Descending power display, full backward compatibility with v1.x signatures -- v2.0
- 89-function PDF manual with Garvan-canonical signatures, workflow-oriented migration guide -- v2.0
- Scripting language: for-loops, procedures (local vars, memoization, RETURN), if/elif/else conditionals -- v3.0
- Expression operations: series() truncation, expand(), floor(), legendre() -- v3.0
- Polynomial operations: factor() cyclotomic/irreducible, subs() substitution -- v3.0
- Bivariate series: tripleprod/quinprod with symbolic z, winquist with symbolic a,b -- v3.0
- 97-function PDF manual with scripting chapter and 3 qmaple.pdf worked examples -- v3.0

### Active

No active milestone. All planned milestones through v3.0 are complete.

### Future

- Multi-sum creative telescoping (qMultiSum equivalent)
- Batch verification of identity database (Gasper-Rahman Appendix II)
- Human-readable proof output for publication
- Quantum algebra (quantum groups, R-matrices, knot polynomial connections)
- Identity database expansion (~500+ verified identities with citations)
- macOS CI support
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

- **Tech stack**: Rust (core engine + CLI) + Python (user API via PyO3 0.23), maturin for packaging
- **Package name**: `q-kangaroo` on PyPI, `q_kangaroo` for Python import, `q-kangaroo` CLI binary
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
| Hand-written Pratt parser for REPL | No external parser libraries; qsym-cli depends only on qsym-core + rustyline | Good |
| LTO + strip + codegen-units=1 | 4.5MB -> 1.4MB binary size for distribution | Good |
| Bundle MinGW DLLs (not static GMP) | Simpler build; static linking deferred as separate project | Superseded by v1.6 |
| Static GMP/MPFR/MPC linking | Zero-dependency binary; eliminates 5 DLLs from release | Good |
| Hand-written CLI arg parser (no clap) | Zero external dependencies; only 5-6 flags | Good |
| BSD sysexits exit codes | Standard, machine-parseable; matches Unix conventions | Good |
| Custom panic hook | Suppresses raw Rust backtrace for clean error output | Good |
| Typst for PDF manual (not LaTeX) | Faster compilation, cleaner syntax, CI-friendly | Good |
| PDF as standalone release artifact | Not bundled in binary archive; separate download for docs | Good |
| Value::Symbol for bare names | Undefined names become Symbol values, not errors; enables Maple-style expressions | Good |
| q-monomial infrastructure | q^n parsed as special expression, enables aqprod(q^2, q, 5) syntax | Good |
| numbpart canonical (not partition_count) | Matches Maple naming; partition_count kept as alias | Good |
| JacobiProduct value type | Enables JAC(a,b) constructor + arithmetic for theta/Jacobi workflows | Good |
| Descending power display | Matches Maple output conventions; DoubleEndedIterator for FPS | Good |
| zqfactor deferred | Requires bivariate (z,q)-series infrastructure not yet in engine | Deferred |
| Garvan-exact argument positions | Verified against Garvan's Maple source, not just docs | Good |
| Workflow-oriented migration guide | Task-based sections ("Computing eta products") vs alphabetical alias table | Good |
| BivariateSeries as BTreeMap<i64, FPS> | Laurent polynomial in z with FPS coefficients; natural for q-products | Good |
| TrivariateSeries for winquist(a,b) | BTreeMap<(i64,i64), FPS> for two symbolic variables | Good |
| AST interception for subs() | Catches Compare(Eq) before eval so q=1 isn't Bool | Good |
| Cyclotomic trial division high-to-low | Scan from highest n down for correct factor discovery | Good |
| Special-case help for language constructs | Match arms before FUNC_HELP lookup avoids count assertion changes | Good |
| Direct Pochhammer for winquist bivariate | Instead of tripleprod decomposition; simpler, fewer truncation issues | Good |

---
*Last updated: 2026-02-21 after v3.0 milestone shipped*
