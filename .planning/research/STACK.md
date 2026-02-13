# Stack Research

**Domain:** Symbolic computation engine for q-series (Rust core + Python API)
**Researched:** 2026-02-13
**Confidence:** MEDIUM-HIGH (core libraries well-verified; some integration patterns rely on training data)

---

## Recommended Stack

### Core Language & Toolchain

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (stable) | 1.85+ | Core computation engine | Required by `rug` 1.28; memory safety without GC is critical for long-running symbolic computations; zero-cost abstractions for expression tree traversal |
| Python | 3.11-3.14 | User-facing API, Jupyter integration | Widest scientific computing ecosystem; python-flint 0.8.0 supports 3.11-3.14; free-threaded Python (3.13t+) enables true parallelism with PyO3 0.28 |

### Arbitrary Precision Arithmetic (PRIMARY -- Rust side)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `rug` | 1.28.1 | Primary BigInt/BigRational/BigFloat via GMP/MPFR/MPC bindings | Fastest arbitrary precision in Rust. Wraps GMP 6.3.0, MPFR 4.2.2, MPC 1.3.1. Benchmarks show rug outperforms malachite and num-bigint on most operations (especially multiplication, GCD, division). For a CAS replacing Maple, GMP-level performance is non-negotiable. Requires Rust 1.85+. **Confidence: HIGH** |
| `gmp-mpfr-sys` | ~1.6 | Low-level FFI to GMP/MPFR/MPC | Underlying dependency of `rug`. Provides raw C bindings when you need direct GMP access for performance-critical mock theta computations. Ships GMP 6.3.0 source and can compile from source or use system libs. Supports MSVC on Windows. **Confidence: HIGH** |
| `num-bigint` | 0.4.6 | Fallback pure-Rust BigInt for no-GMP environments | Keep as optional feature flag for users who cannot install GMP (e.g., WASM targets). 2-10x slower than rug for large numbers but zero C dependencies. Requires rustc 1.60+. **Confidence: HIGH** |
| `num-rational` | 0.4.x | Pure-Rust BigRational (pairs with num-bigint) | Same fallback rationale. Use `Ratio<BigInt>` type. **Confidence: HIGH** |

**Architecture decision:** Use `rug` as the default arithmetic backend behind a trait abstraction (`QNumber` trait). Implement for both `rug::Integer`/`rug::Rational` and `num::BigInt`/`num::BigRational`. Feature-flag the GMP backend (`default = ["gmp"]`). This lets users without C toolchains still build the library, while production use gets GMP speed.

### Expression Representation (Rust side)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `hashconsing` | latest (0.5.x) | Hash-consed expression DAG with structural sharing | Implements Filiatre-Conchon hash consing. Guarantees structural identity via UID comparison (O(1) equality). Based on `Arc` with weak-reference deduplication tables. Critical for q-series where subexpression sharing (e.g., `(q; q)_inf` appearing in many terms) dominates. Recent arxiv paper (2509.20534) confirms 3-100x downstream speedups from hash consing in symbolic computation. **Confidence: MEDIUM** (library is maintained but low download count; may need custom fork or from-scratch implementation using same principles) |
| `bumpalo` | 3.19.0 | Arena allocator for temporary expression trees during rewriting | Phase-oriented allocation for rewrite passes. Allocate all intermediate expressions in a bump arena, extract result, then drop entire arena. Avoids per-node allocation overhead during saturation. Well-maintained, 130M+ downloads. **Confidence: HIGH** |
| Custom `ExprPool` | N/A | Index-based expression storage (arena + hash table) | Build a custom expression pool where nodes are stored in a `Vec<ExprNode>` and referenced by `ExprId` (a `u32` index). Combine with hash-consing: before inserting a new node, check a `HashMap<ExprNode, ExprId>` for structural duplicates. This is the pattern used by egg internally and by Symbolica. Avoids Rust borrow-checker pain with recursive tree structures. **Confidence: HIGH** (well-established pattern in Rust CAS implementations) |

### Rewrite / Simplification Engine (Rust side)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `egg` | 0.11.0 | E-graph equality saturation for expression simplification | The standard Rust library for equality saturation. `define_language!` macro for custom expression types, `rewrite!` macro for pattern-based rules, `Runner` for saturation, `Extractor` with custom cost functions. 30x faster than traditional approaches. Well-documented with tutorials. Used by Herbie (numerical accuracy), Cranelift (compiler), and research projects. **Confidence: HIGH** |
| `egglog` | 2.0.0 | Next-gen equality saturation with Datalog integration | Successor to `egg`. Faster and more general. Adds Datalog-style rules, incremental execution, lattice-based reasoning. Better for complex multi-step rewriting where you need relational queries over the e-graph (e.g., "find all eta-quotients equivalent to this q-series modulo some modular relation"). However, API is less stable and documentation is thinner than `egg`. **Confidence: MEDIUM** |

**Architecture decision:** Start with `egg` 0.11.0 for the rewrite engine. It is battle-tested, well-documented, and the `define_language!` + `rewrite!` macros make it straightforward to encode q-series identities as rewrite rules. Plan migration path to `egglog` once the core identity set is established -- egglog's Datalog integration will be valuable for modular form relations that are naturally expressed as relational queries. Wrap the e-graph behind a `Simplifier` trait so the backend can be swapped.

### Python Bindings

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `pyo3` | 0.28.0 | Rust-to-Python FFI bindings | The only serious option for Rust/Python interop. Supports CPython 3.7+, PyPy 7.3+, GraalPy 24.0+. v0.28 adds PEP 489 multi-phase module init, free-threaded Python support (opt-out), `#[pyclass]` improvements. Requires Rust 1.83+. **Confidence: HIGH** |
| `maturin` | 1.11.5 | Build tool for PyO3 packages | Builds and publishes Rust+PyO3 crates as Python wheels. Handles cross-platform wheel building, sdist generation, PyPI upload. `maturin develop` for rapid iteration. Use with `uv` for fast venv management. **Confidence: HIGH** |

### Python-Side Libraries

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| `sympy` | 1.14.x | Symbolic math interop, LaTeX printing, series expansion verification | SymPy is the Python symbolic math standard. Use for: (1) `sympy.printing.latex()` to generate LaTeX from expressions, (2) verification/testing against SymPy's own simplification, (3) interop with existing mathematical software. Do NOT use as computation backend -- too slow for production q-series. **Confidence: HIGH** |
| `python-flint` | 0.8.0 | Fast polynomial/number-theory operations from Python | Wraps FLINT 3.3.1 + MPFR 4.2.2. Provides `fmpz`, `fmpq`, `fmpz_poly`, `arb`, `acb` types from Python. Use for: (1) cross-validation of Rust results, (2) users who want FLINT-speed polynomial ops without going through our Rust core, (3) potential integration as alternative backend for specific operations. Ships binary wheels for CPython 3.11-3.14. **Confidence: HIGH** |
| `IPython.display` | (bundled) | LaTeX rendering in Jupyter | Built into IPython/Jupyter. `display(Math(latex_string))` renders via MathJax. Zero additional dependencies for Jupyter users. Our Python objects should implement `_repr_latex_()` for automatic rendering. **Confidence: HIGH** |
| `numpy` | 2.x | Numerical evaluation of formal power series | Standard numerical array library. Use for vectorized coefficient extraction, numerical evaluation of truncated series. Interop with PyO3 via `numpy` crate or `pyo3-numpy`. **Confidence: HIGH** |

### Development & Testing Tools

| Tool | Version | Purpose | Notes |
|------|---------|---------|-------|
| `cargo-nextest` | latest | Fast Rust test runner | Parallel test execution, better output than `cargo test` |
| `criterion` | 0.5.x | Rust benchmarking | Statistically rigorous benchmarks for arithmetic and rewriting performance |
| `proptest` / `quickcheck` | latest | Property-based testing | Critical for CAS correctness: generate random expressions, verify identities hold |
| `pytest` | 8.x | Python test runner | For Python API tests |
| `hypothesis` | latest | Python property-based testing | Generate random q-series expressions, verify Rust/Python agreement |
| `rayon` | 1.11.0 | Data parallelism in Rust | `par_iter()` for parallel rewrite rule application, parallel coefficient computation |
| `serde` + `serde_json` | 1.0.x | Serialization of expressions | Serialize expression trees for caching, debugging, IPC. All expression types should derive `Serialize`/`Deserialize`. |
| `tracing` | 0.1.x | Structured logging | Instrument rewrite engine, track simplification steps, profile performance |

### FLINT Integration (Rust side -- for polynomial arithmetic)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `flint3-sys` | 3.3.1 | Raw FFI bindings to FLINT C library | Provides access to FLINT's world-class polynomial arithmetic (`fmpz_poly`, `fmpq_poly`), modular arithmetic (`nmod_poly`), and number theory functions. Since FLINT 3 merged Arb, also gives access to ball arithmetic (`arb`, `acb`) for rigorous error-bounded computation. Compiles FLINT from source (best performance). **Confidence: MEDIUM** (crate is maintained but thin Rust wrapper; you will write safe Rust wrappers on top) |

**Architecture decision:** Do NOT try to reimplement polynomial arithmetic from scratch. FLINT is the gold standard for number-theory polynomials and is used by SageMath, SymPy (as optional backend), and Mathematica. Use `flint3-sys` for raw bindings and build safe Rust wrappers for the specific FLINT functions needed: `fmpz_poly` (integer polynomials), `fmpq_poly` (rational polynomials), `arb_poly` (ball arithmetic polynomials for rigorous power series truncation).

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Arbitrary precision | `rug` (GMP bindings) | `malachite` 0.9.1 (pure Rust) | Malachite is impressive pure Rust but 1.5-3x slower than rug/GMP for most operations due to lack of inline assembly. For a CAS replacing Maple, we need GMP-level speed. Malachite also LGPL-licensed which may complicate downstream use. |
| Arbitrary precision | `rug` (GMP bindings) | `dashu` 0.4.x (pure Rust, no_std) | Similar to malachite: pure Rust is nice but slower. `no_std` support irrelevant for our use case. Less mature than malachite. |
| Arbitrary precision | `rug` (GMP bindings) | `num-bigint` 0.4.6 (pure Rust) | Significantly slower (5-10x for large numbers). Only use as fallback for no-GMP environments. Algorithms are simpler (e.g., O(n^1.5) string conversion vs malachite's O(n log^2 n)). |
| E-graphs | `egg` 0.11.0 | `egglog` 2.0.0 | egglog is the future but API less stable, documentation thinner. Start with egg, migrate later. |
| E-graphs | `egg` | Custom rewrite engine | Rolling your own is tempting but equality saturation is hard to get right. egg has years of research behind it. |
| Expression trees | Custom `ExprPool` | `hashconsing` crate alone | hashconsing provides the interning but not the full expression pool. Combine hashconsing principles with index-based arena for best of both worlds. |
| Python bindings | `pyo3` 0.28 | `cpython` crate | cpython crate is effectively abandoned. PyO3 is the only maintained option. |
| Python bindings | `pyo3` 0.28 | `cffi` via maturin | cffi is C-level FFI, loses all the ergonomic #[pyclass]/#[pymethods] benefits. Only use if targeting non-CPython interpreters that lack PyO3 support. |
| Build tool | `maturin` | `setuptools-rust` | setuptools-rust works but maturin is simpler, faster, and purpose-built for PyO3. |
| CAS foundation | Build custom engine | Use Symbolica as foundation | Symbolica is source-available, NOT open source. Commercial license required for academic/professional use ($6,600/year institutional). Cannot fork or redistribute. Defeats the "open source replacement for Garvan's Maple packages" goal. |
| CAS foundation | Build custom engine | Wrap SymPy from Rust | SymPy is pure Python and orders of magnitude too slow for production q-series computation. Good for verification, not for the engine. |
| CAS foundation | Build custom engine | Use SageMath | SageMath is monolithic, GPL-licensed, and hard to embed. Good as external verification tool but wrong architecture for a library. |
| Polynomial arithmetic | `flint3-sys` (FLINT bindings) | Reimplement in pure Rust | FLINT has 20+ years of optimized polynomial algorithms. Reimplementing even basic polynomial multiplication competitively would take months. Use FLINT via FFI. |
| Polynomial arithmetic | `flint3-sys` | `rug` polynomial support | GMP does not have polynomial types. MPFR is floating-point only. FLINT is purpose-built for exact polynomial/number-theory computation. |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Symbolica as foundation | Source-available, not open source. Commercial license required for any non-hobbyist use. Cannot redistribute. Fundamentally incompatible with open-source project goals. | Build custom engine using egg + rug + flint3-sys. Study Symbolica's architecture (it's readable on GitHub) for design inspiration only. |
| SymPy as computation engine | 100-1000x slower than FLINT for polynomial operations. Python overhead makes it unsuitable as a computation backend. | Use SymPy for LaTeX printing, verification, and as user-facing interop layer only. Core computation in Rust. |
| `num-bigint` as primary arithmetic | 5-10x slower than GMP for large integers. O(n^2) algorithms for operations that GMP does in O(n log n). | Use `rug` (GMP) as primary, `num-bigint` as optional pure-Rust fallback behind feature flag. |
| `cpython` crate | Effectively abandoned. Last meaningful update years ago. | `pyo3` 0.28.0 -- actively maintained, large community. |
| Rolling custom e-graph implementation | Equality saturation is a research-grade algorithm. Getting congruence closure, rebuilding, and extraction right is months of work. | `egg` 0.11.0 -- battle-tested, used in production compilers and research. |
| Maple/proprietary CAS as dependency | Defeats the entire purpose of the project (replacing proprietary Maple dependency). | Build equivalent functionality in Rust, verify against Maple results during testing. |
| `f64` for any exact computation | IEEE 754 floating-point cannot represent exact rational coefficients. Silent precision loss will produce wrong mathematical results. | `rug::Rational` for exact arithmetic. Only use floats for numerical evaluation (clearly labeled approximate). |

---

## Stack Patterns by Variant

**If targeting maximum performance (production research use):**
- Enable `gmp` feature (default): uses `rug` + `gmp-mpfr-sys` + `flint3-sys`
- Requires C compiler and GMP/MPFR/FLINT installed (or compiled from source by build scripts)
- This is the expected configuration for researchers replacing Garvan's Maple packages

**If targeting easy installation / WASM / no C toolchain:**
- Disable `gmp` feature: falls back to `num-bigint` + `num-rational`
- No FLINT integration (polynomial operations will be slower, pure-Rust fallback)
- Suitable for teaching, lightweight exploration, web demos

**If targeting Jupyter notebook users:**
- Python package via `maturin` + `pyo3`
- `_repr_latex_()` on all Python expression objects for automatic MathJax rendering
- Integration with SymPy's pretty-printing for familiar output

**If targeting SageMath integration:**
- Expose as standard Python package installable in SageMath's Python environment
- Provide conversion functions between our types and SageMath's power series / modular form types
- This is a Phase 3+ concern, not MVP

---

## Version Compatibility Matrix

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| `rug` 1.28.1 | `gmp-mpfr-sys` ~1.6, Rust 1.85+ | rug pins gmp-mpfr-sys minor version |
| `gmp-mpfr-sys` 1.6.x | GMP 6.3.0, MPFR 4.2.2, MPC 1.3.1 | Compiles from bundled source by default |
| `flint3-sys` 3.3.1 | FLINT 3.3.x, GMP (system or compiled) | Always compiles FLINT from source; can share GMP with gmp-mpfr-sys via system libs |
| `pyo3` 0.28.0 | Rust 1.83+, Python 3.7+, PyPy 7.3+ | MSRV bumped to 1.83 in v0.28 |
| `maturin` 1.11.5 | `pyo3` 0.28.x, Python 3.8+ | Handles wheel building for all platforms |
| `egg` 0.11.0 | Rust stable (1.70+ estimated) | No known incompatibilities |
| `python-flint` 0.8.0 | FLINT 3.0-3.3, CPython 3.11-3.14 | Binary wheels available; no build from source needed on Python side |
| `sympy` 1.14.x | Python 3.9+ | Pure Python, no compatibility issues |
| `numpy` 2.x | Python 3.10+ | NumPy 2.0 has breaking C API changes; use `pyo3-numpy` compatible version |

**Minimum Rust version for full stack: 1.85.0** (driven by `rug` 1.28.1)
**Minimum Python version for full stack: 3.11** (driven by `python-flint` 0.8.0 wheel availability)

---

## Installation

### Rust Dependencies (Cargo.toml)

```toml
[dependencies]
# Arbitrary precision arithmetic (GMP backend -- default)
rug = { version = "1.28", optional = true, features = ["rational", "float", "serde"] }
gmp-mpfr-sys = { version = "~1.6", optional = true }

# Arbitrary precision arithmetic (pure Rust fallback)
num-bigint = { version = "0.4", optional = true }
num-rational = { version = "0.4", optional = true }
num-traits = "0.2"

# Expression rewriting (e-graphs)
egg = "0.11"

# Hash consing for expression deduplication
hashconsing = "0.5"

# FLINT bindings for polynomial arithmetic
flint3-sys = { version = "3.3", optional = true }

# Python bindings
pyo3 = { version = "0.28", features = ["extension-module"], optional = true }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Parallelism
rayon = "1.11"

# Logging / tracing
tracing = "0.1"

# Arena allocation
bumpalo = { version = "3.19", features = ["collections"] }

[features]
default = ["gmp", "flint"]
gmp = ["dep:rug", "dep:gmp-mpfr-sys"]
flint = ["dep:flint3-sys"]
pure-rust = ["dep:num-bigint", "dep:num-rational"]
python = ["dep:pyo3"]

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.0"
```

### Python Dependencies (pyproject.toml)

```toml
[build-system]
requires = ["maturin>=1.11,<2.0"]
build-backend = "maturin"

[project]
name = "qsymbolic"
requires-python = ">=3.11"
dependencies = [
    "sympy>=1.13",
    "numpy>=1.26",
]

[project.optional-dependencies]
flint = ["python-flint>=0.8.0"]
dev = [
    "pytest>=8.0",
    "hypothesis>=6.0",
    "jupyter>=1.0",
    "ipython>=8.0",
]

[tool.maturin]
features = ["python"]
```

### Build Commands

```bash
# Rust library (with GMP + FLINT)
cargo build --release

# Rust library (pure Rust, no C deps)
cargo build --release --no-default-features --features pure-rust

# Python package (development)
maturin develop --release

# Python package (wheel for distribution)
maturin build --release

# Run tests
cargo nextest run
pytest tests/

# Run benchmarks
cargo bench
```

---

## Existing CAS Landscape: Leverage vs Build

### What to LEVERAGE (use as dependencies or for interop)

| System | How to Leverage | Why |
|--------|----------------|-----|
| **GMP** (via `rug`) | Primary integer/rational arithmetic backend | 30+ years of optimization, inline assembly, fastest arbitrary precision on every platform |
| **FLINT** (via `flint3-sys`) | Polynomial arithmetic, number theory functions | Gold standard for exact polynomial computation. Used by SageMath and SymPy internally. Since FLINT 3, includes Arb for ball arithmetic. |
| **egg** | Equality saturation engine | Research-grade rewrite engine. Encode q-series identities as rewrite rules. |
| **SymPy** (Python side) | LaTeX generation, verification, user-familiar API patterns | SymPy's `latex()` printer is excellent. Users know SymPy's API conventions. |
| **python-flint** (Python side) | Cross-validation, alternative polynomial backend | Provides direct FLINT access from Python for users who want it |

### What to BUILD from scratch

| Component | Why Build It | Complexity |
|-----------|-------------|------------|
| **Expression type system** | q-series has domain-specific node types (q-Pochhammer symbols, eta functions, theta functions, mock theta functions) that no existing CAS represents natively | Medium |
| **Hash-consed expression pool** | Needs tight integration with our specific expression types and the e-graph rewrite engine | Medium |
| **q-series identity database** | Domain-specific: Garvan's identities, Ramanujan's 40 identities, modular equation catalog | High (mathematical, not engineering) |
| **Formal power series engine** | Lazy truncated power series with exact rational coefficients. Existing Rust implementations don't exist; SymPy's is too slow; SageMath's can't be extracted. | High |
| **Modular form arithmetic** | Eta-quotient manipulation, modular equation solving, valence formula implementation | High (mathematical) |
| **Python API layer** | PyO3 bindings with Pythonic API design, `_repr_latex_()`, operator overloading | Medium |
| **Safe Rust wrappers for FLINT** | `flint3-sys` gives raw C bindings; we need safe, ergonomic Rust types on top | Medium |

### What to IGNORE (do not build or integrate)

| System | Why Ignore |
|--------|-----------|
| **Symbolica** | Source-available, not open source. Commercial license. Study for design inspiration only. |
| **Wolfram/Mathematica** | Proprietary. Cannot integrate. Use for manual verification only. |
| **Maple** | The thing we are replacing. Use Garvan's published identities (papers), not the Maple implementation. |
| **Maxima/FriCAS** | Lisp-based, hard to call from Rust, and their q-series support is minimal. |
| **SageMath** | Monolithic GPL system. Cannot embed. Use as external verification. SageMath's power series use PARI/FLINT internally -- we integrate FLINT directly instead. |

---

## Rust Ecosystem Maturity Assessment for Symbolic Math

| Area | Maturity | Assessment |
|------|----------|------------|
| Arbitrary precision integers | **Mature** | rug/GMP is production-grade. malachite and num provide pure-Rust alternatives. |
| Arbitrary precision rationals | **Mature** | rug::Rational wraps GMP's mpq. Fully functional. |
| Polynomial arithmetic | **Immature in Rust, mature via FFI** | No good pure-Rust polynomial library for exact computation. FLINT via `flint3-sys` fills the gap. |
| Expression trees / DAGs | **Patterns exist, no dominant library** | Use index-based arena pattern (well-documented). `hashconsing` crate exists but is niche. Build custom. |
| E-graph rewriting | **Mature** | egg 0.11 is best-in-class. egglog 2.0 is the future. Rust leads the e-graph ecosystem. |
| Pattern matching on expressions | **Mature via egg** | egg's `Pattern` and `Searcher`/`Applier` traits handle this. |
| Formal power series | **Does not exist** | Must build from scratch. No Rust library for lazy formal power series with exact coefficients. |
| Python bindings | **Mature** | PyO3 0.28 + maturin 1.11 is the gold standard. |
| Serialization | **Mature** | serde is the Rust standard. Expression trees serialize naturally. |
| Parallelism | **Mature** | rayon provides effortless data parallelism. |

**Overall assessment:** The Rust ecosystem provides excellent foundations (arithmetic, e-graphs, Python bindings, parallelism) but has significant gaps in domain-specific symbolic math (formal power series, polynomial algebra, expression tree libraries). The strategy is: leverage mature Rust crates where they exist, use FFI to C libraries (GMP, FLINT) for number-theory-specific operations, and build domain-specific components (expression types, power series, q-series identities) from scratch.

---

## Sources

### Verified (HIGH confidence)
- [rug 1.28.1 docs](https://docs.rs/rug/latest/rug/) -- version, GMP dependency, features
- [egg 0.11.0 docs](https://docs.rs/egg/latest/egg/) -- version, API, tutorials
- [egglog 2.0.0 docs](https://docs.rs/egglog/latest/egglog/) -- version, successor to egg
- [PyO3 0.28.0 docs](https://docs.rs/pyo3/latest/pyo3/) -- version, Python/Rust version requirements
- [maturin 1.11.5 on PyPI](https://pypi.org/project/maturin/) -- version, release date
- [python-flint 0.8.0 on PyPI](https://pypi.org/project/python-flint/) -- version, FLINT 3.3.1, CPython 3.11-3.14
- [num-bigint 0.4.6 docs](https://docs.rs/num-bigint/latest/num_bigint/) -- version, features
- [bumpalo 3.19.0 docs](https://docs.rs/crate/bumpalo/latest) -- version, features
- [flint3-sys 3.3.1 docs](https://docs.rs/flint3-sys/latest/flint3_sys/) -- version, FLINT types exposed
- [gmp-mpfr-sys docs](https://docs.rs/gmp-mpfr-sys) -- GMP 6.3.0, MPFR 4.2.2, MPC 1.3.1
- [malachite 0.9.1 docs](https://docs.rs/malachite/latest/malachite/) -- version, LGPL license, sub-crates

### Verified (MEDIUM confidence)
- [Malachite performance page](https://www.malachite.rs/performance/) -- benchmarks vs rug, num
- [hashconsing crate docs](https://docs.rs/hashconsing) -- Filiatre-Conchon implementation
- [Symbolica pricing/license](https://symbolica.io/license/) -- source-available, commercial license required
- [Symbolica 1.0 release post](https://symbolica.io/posts/stable_release/) -- Numerica/Graphica MIT crates
- [E-Graphs in Rust (Stephen Diehl)](https://www.stephendiehl.com/posts/egraphs/) -- architecture patterns
- [Hash consing for symbolic computation (arxiv 2509.20534)](https://arxiv.org/html/2509.20534v2) -- performance data, architecture

### Research references
- [Garvan q-series Maple package tutorial](https://qseries.org/fgarvan/papers/qmaple.pdf)
- [Garvan ETA package manual](https://qseries.org/fgarvan/qmaple/ETA/tutorial/maple-eta-manual.pdf)
- [Garvan thetaids package](https://qseries.org/fgarvan/qmaple/thetaids/)
- [SymPy formal power series docs](https://docs.sympy.org/latest/modules/series/formal.html)
- [SageMath power series docs](https://doc.sagemath.org/html/en/reference/power_series/sage/rings/power_series_ring_element.html)
- [FLINT documentation](https://flintlib.org/doc/introduction_calcium.html)
- [SymPy + FLINT integration plans (Oscar Benjamin)](https://oscarbenjamin.github.io/blog/czi/post1.html)

---
*Stack research for: Q-Symbolic (q-series symbolic computation engine)*
*Researched: 2026-02-13*
