# Project Research Summary

**Project:** Q-Symbolic (q-Series Symbolic Computation Engine)
**Domain:** Symbolic mathematics - q-series, basic hypergeometric functions, partition theory, modular forms
**Researched:** 2026-02-13
**Confidence:** MEDIUM-HIGH

## Executive Summary

Q-Symbolic is a Rust-core, Python-API symbolic computation engine replacing Frank Garvan's proprietary Maple packages for q-series research. The research confirms that building a focused, domain-specific tool is more viable than extending general CAS systems. The recommended architecture uses hash-consed expression DAGs (Maple-style), equality saturation for rewriting (egg library), and GMP-backed arbitrary precision arithmetic (rug crate), exposed to Python via PyO3 opaque handles.

The critical insight from research is that **representation strategy determines feasibility**. q-series identities require multiple equivalent forms (infinite products, truncated series, eta-quotients, theta functions) and naive canonicalization destroys essential structure. The architecture must support lazy, multi-representation expressions where conversion between forms is explicit and on-demand, not automatic. This differs from general CAS design and is project-critical.

The roadmap should follow strict functional parity with Garvan's qseries package v1.3 (41 functions) before adding extensions. Three key risks: (1) expression swell during series arithmetic (mitigate with truncated multiplication), (2) rewrite engine non-termination (mitigate with phased simplification and strict rule ordering), and (3) scope creep into general CAS territory (mitigate with function-by-function validation against Maple output). The Rust ecosystem provides strong foundations for arithmetic and rewriting but has no existing formal power series library - this is a from-scratch build.

## Key Findings

### Recommended Stack

**Rationale:** Rust core provides 10-100x performance over Maple while maintaining exact arithmetic; Python bindings via PyO3 enable Jupyter integration for the target research audience.

**Core technologies:**

- **Rust 1.85+ stable** - Memory safety without GC is critical for long-running symbolic computations; required by rug 1.28
- **rug 1.28 (GMP/MPFR/MPC bindings)** - Fastest arbitrary precision arithmetic (2-10x faster than pure Rust alternatives); non-negotiable for CAS performance
- **egg 0.11 (equality saturation)** - Battle-tested e-graph library for rewrite engine; 30x faster than traditional approaches; used by Herbie and Cranelift
- **PyO3 0.28 + maturin 1.11** - Rust-Python FFI; enables first-class Jupyter integration; supports free-threaded Python 3.13+
- **flint3-sys 3.3** - World-class polynomial arithmetic via FLINT C library; 20+ years of optimization cannot be replicated in pure Rust
- **Custom ExprArena + hash consing** - Hash-consed DAG representation for O(1) equality and structural sharing; the hashconsing crate exists but requires custom integration

**Critical dependencies:**
- GMP 6.3.0, MPFR 4.2.2, MPC 1.3.1 (bundled via gmp-mpfr-sys)
- FLINT 3.3.1 (compiled from source via flint3-sys)
- Minimum Rust 1.85, minimum Python 3.11

**What NOT to use:**
- Symbolica (source-available, not open source; commercial license incompatible with open-source mission)
- SymPy as computation engine (100-1000x too slow; use only for LaTeX printing and verification)
- num-bigint as primary arithmetic (5-10x slower than GMP; use only as pure-Rust fallback behind feature flag)

### Expected Features

**Table stakes (Garvan qseries v1.3 parity - 41 functions):**

- q-Pochhammer symbol (aqprod), q-binomial (qbin), finite/infinite products
- Named products: etaq, jacprod, tripleprod, quinprod, winquist
- Theta functions: theta, theta2, theta3, theta4
- **Series-to-product conversion** (the core differentiator): prodmake, etamake, jacprodmake, mprodmake, qetamake, qfactor
- Algebraic relation discovery: findlincombo, findhom, findpoly, findcong (full suite of 12 functions)
- Series utilities: sift, qdegree, coefficient extraction
- Python bindings with LaTeX output for Jupyter

**Competitive differentiators (extensions beyond Garvan):**

- Basic hypergeometric series engine (_r phi _s evaluation) - Garvan lacks this; Mathematica has QHypergeometricPFQ
- Classical summation formulas (q-Gauss, q-Vandermonde, q-Saalschutz) - automated application, not manual
- Performance via Rust core - parallel series multiplication, 10-100x faster than Maple for systematic searches
- Mock theta functions (Ramanujan's 17 + Zwegers framework) - no existing CAS has dedicated support
- Bailey chain machinery - automated iteration for identity generation

**Defer to v2+:**

- Identity proving (thetaids + ETA packages) - requires full modular forms theory infrastructure
- Partition rank/crank packages - combinatorial extensions beyond core series engine
- WZ method (q-Zeilberger, creative telescoping) - algorithmically complex, deferred until core is validated
- Full modular forms spaces (M_k(Gamma_0(N)), Hecke operators) - SageMath already covers this well

### Architecture Approach

**Multi-representation expressions with lazy conversion.** The IR must store expressions in their "natural" form (product, series, eta-quotient) with explicit, on-demand conversion between forms. Hash-consed arena storage (u32 index-based ExprRef, not Arc pointers) provides O(1) equality and structural sharing. Phased simplification engine (6 phases: arithmetic normalization -> q-Pochhammer algebra -> hypergeometric recognition -> identity lookup -> series expansion -> e-graph saturation) with equality saturation reserved for hard cases, not routine simplification.

**Major components:**

1. **ExprArena** - Hash-consing interner with Vec<Expr> storage and HashMap dedup table; owns all expression memory; every component that creates expressions takes &mut ExprArena
2. **Expr enum** - N-ary Add/Mul with sorted children, domain-specific nodes (QPochhammer, JacobiTheta, DedekindEta, BasicHypergeometric), opaque FPSRef for bulk series data
3. **SimplificationEngine** - Phased rule application with bottom-up traversal; rules provided by domain modules; optional e-graph backend for phase 6
4. **FormalPowerSeries** - Sparse BTreeMap<i64, BigRat> representation with lazy generators for infinite products; truncation order tracked explicitly
5. **Domain modules** - Trait-based modules (qpoch, hypergeo, theta, partitions, mock_theta, bailey, wz) registered statically via inventory crate; each provides rules + operations + identities
6. **PyO3 boundary** - Opaque QExpr handles wrapping ExprRef; Arc<Mutex<Session>> owns arena; all computation in Rust with GIL released; Python is orchestration/display layer only

**Critical architectural decisions (must be made in Phase 1):**

- ExprRef size (u32 vs u64 vs pointer) - affects every file in codebase
- Canonical ordering strategy for Add/Mul children - correctness of hash consing depends on this
- n-ary vs binary operators - affects every pattern match
- Session/arena ownership model - determines Python FFI ergonomics

### Critical Pitfalls

1. **Choosing wrong expression normalization strategy (PROJECT-KILLER)** - Auto-expanding to canonical form destroys product structure needed for pattern matching. Mitigation: Multi-representation IR where conversions are explicit user/algorithm operations, not automatic. Use separate types for different representations (product form vs series form vs symbolic form).

2. **Infinite loops in rewriting (PROJECT-KILLER)** - Bidirectional rules cause non-termination; even terminating TRS can cause e-graph blowup. Mitigation: Separate directed simplification rules (guaranteed terminating via complexity measure) from transformation rules (user-invoked, iteration-bounded). Use depth-bounded equality saturation, not unlimited.

3. **Expression swell (PROJECT-KILLER)** - Multiplying two O(q^N) series creates O(q^2N) intermediate before truncation; p(1000) expansion generates millions of terms. Mitigation: Truncated arithmetic everywhere (truncate during multiplication, not after); sparse representation; lazy series with generators; recurrence relations for partition functions, not generating function expansion.

4. **Hash consing memory leaks in Rust (CRITICAL)** - Strong references in intern table cause monotonic memory growth. Mitigation: Arena-based allocation (drop entire arena after computation) instead of global hash consing. Two-tier system: short-lived computation arenas + long-lived knowledge base with explicit insertion.

5. **q-Pochhammer edge cases (CRITICAL)** - Negative n, n=0, n=infinity, multi-parameter products all have specific formulas easy to get wrong. Mitigation: Comprehensive test suite against Maple/Mathematica for n in {-5,-1,0,1,2,5,inf} and a in {0,1,q,q^2,generic}; test every function before moving to next.

6. **PyO3 GIL memory accumulation (CRITICAL)** - GIL Refs API causes memory leaks; Python objects stay alive until GILPool exits. Mitigation: Use ONLY Bound<T> API (not deprecated GIL Refs); minimize Python object creation in Rust; zero-copy for bulk data (NumPy array views); profile memory under realistic usage.

## Implications for Roadmap

Based on combined research, the roadmap should follow a strict **foundation -> core parity -> extensions** sequence. The critical path is ExprArena + Expr enum + SimplificationEngine + q-Pochhammer module, as everything depends on these. Parallel research threads are not recommended in Phase 1-2 due to tight architectural coupling.

### Phase 1: Foundation (Expression IR + Arithmetic)

**Rationale:** Every component depends on how expressions are represented and how arithmetic works. Getting this wrong requires a full rewrite. Must be architecturally correct before any domain functions.

**Delivers:**
- ExprArena with hash-consing (u32 ExprRef, Vec<Expr> storage, HashMap dedup)
- Expr enum (atoms, arithmetic, q-specific nodes, FPS placeholder)
- Canonical ordering for Add/Mul children
- BigInt/BigRat via rug (with num-bigint fallback behind feature flag)
- Pattern matcher (structural patterns with wildcards)
- LaTeX + pretty-print rendering

**Addresses:** Pitfall 1 (normalization), Pitfall 4 (hash consing memory), Pitfall 9 (over-engineering)

**Critical decision point:** ExprRef size, canonical ordering, n-ary vs binary operators - these cannot change later

**Duration estimate:** 3-4 weeks

**Research flag:** STANDARD PATTERNS - hash-consed arenas are well-documented in rustc-dev-guide; no additional research needed

### Phase 2: Simplification Engine + Formal Power Series

**Rationale:** q-Pochhammer expansion requires series arithmetic; series multiplication requires simplification to avoid blowup. These are co-dependent.

**Delivers:**
- SimplificationEngine with phased rule application (6 phases)
- Bottom-up traversal with iteration limits
- FormalPowerSeries (sparse BTreeMap, truncation tracking, lazy generators)
- Series arithmetic (add, mul, truncate, coefficient extraction)
- RewriteRule + Pattern infrastructure for domain modules

**Addresses:** Pitfall 2 (rewrite loops), Pitfall 3 (expression swell), Pitfall 8 (testing strategy)

**Uses:** ExprArena from Phase 1, rug arithmetic, egg 0.11 (integration planned, not implemented yet)

**Duration estimate:** 3-4 weeks

**Research flag:** STANDARD PATTERNS - phased rewriting is textbook TRS; FPS algorithms in academic literature

### Phase 3: Core q-Series Functions (qseries Package Parity)

**Rationale:** Function-by-function validation against Garvan's Maple output. This is the MVP - researchers cannot adopt without these 41 functions.

**Delivers:**
- q-Pochhammer module: aqprod, qbin, finite/infinite products
- Named products: etaq, jacprod, tripleprod, quinprod, winquist
- Theta functions: theta, theta2, theta3, theta4
- Series-to-product conversion: prodmake, etamake, jacprodmake (Andrews' algorithm)
- Factoring: qfactor, zqfactor
- Series utilities: sift, qdegree, lqdegree, coefficient extraction
- Algebraic relation discovery: findlincombo, findhom, findpoly suite (12 functions)

**Addresses:** Pitfall 5 (q-Pochhammer edge cases), Pitfall 7 (scope creep), Pitfall 8 (single-oracle testing)

**Testing strategy:** Three-oracle (Maple, Mathematica, mathematical identities); property-based tests for q-Pochhammer algebra; coefficient-level comparison

**Duration estimate:** 6-8 weeks (41 functions, each requires testing before moving to next)

**Research flag:** NEEDS PHASE RESEARCH for prodmake/etamake/jacprodmake - Andrews' algorithm is documented but implementation details sparse

### Phase 4: Python Bindings (Jupyter Integration)

**Rationale:** Once core functions work in Rust, researchers need Python access. This validates the API design before adding more functions.

**Delivers:**
- PyO3 bindings (qsym-python crate)
- QExpr opaque handles wrapping ExprRef
- QSession managing Arc<Mutex<Session>>
- Python DSL (symbols(), qpoch(), hyper_q(), etc.)
- _repr_latex_() for Jupyter rendering
- NumPy integration for coefficient arrays

**Addresses:** Pitfall 6 (PyO3 memory), Pitfall 13 (lifetime/FFI conflict), UX pitfalls (display, progress)

**Uses:** Bound<T> API only (not deprecated GIL Refs); py.allow_threads() for all non-trivial work

**Duration estimate:** 2-3 weeks

**Research flag:** STANDARD PATTERNS - PyO3 + maturin are well-documented

### Phase 5: Differentiators (Beyond Garvan)

**Rationale:** With core parity established and API validated, add features that make Q-Symbolic competitive with Mathematica.

**Delivers:**
- Basic hypergeometric series module (_r phi _s evaluation)
- Classical summation formulas (q-Gauss, q-Vandermonde, q-Saalschutz, q-Dougall)
- Transformation formulas (Heine, Sears, Watson)
- Performance optimizations (parallel series multiplication via rayon)
- Batch computation mode (CLI for systematic searches)

**Addresses:** Feature gap vs Mathematica; performance claims (10-100x faster than Maple)

**Duration estimate:** 4-5 weeks

**Research flag:** NEEDS PHASE RESEARCH for transformation formulas - Gasper-Rahman book has formulas, but verification logic unclear

### Phase 6: Identity Proving (thetaids + ETA Packages)

**Rationale:** Full Garvan parity requires identity proving, but this is algorithmically complex and depends on all prior phases.

**Delivers:**
- JAC/ETA symbolic representation model
- Cusp and order computation (cuspmake1, getacuspord suite)
- provemodfuncid (automatic identity proving via valence formula)
- ETA package identity pipeline
- Identity database (TOML files loaded at session init)

**Addresses:** Remaining Garvan functions (~55 from thetaids, ~20 from ETA)

**Duration estimate:** 6-8 weeks (modular forms theory is mathematically complex)

**Research flag:** NEEDS DEEP RESEARCH - cusp theory, valence formula, Ligozat/Martin conditions require domain expertise; consider expert consultation

### Phase 7: Partitions + Extensions

**Rationale:** After core and identity proving, add combinatorial and advanced features.

**Delivers:**
- Rank/Crank/SPT-Crank packages (partition statistics)
- Mock theta functions (Ramanujan's 17 + Zwegers framework)
- Bailey chain machinery (pair database, lemma application, chain iteration)
- T-Core package

**Duration estimate:** 5-6 weeks

**Research flag:** NEEDS PHASE RESEARCH for mock theta and Bailey chains - academic literature exists but implementation sparse

### Phase Ordering Rationale

- **Phases 1-2 are tightly coupled** - expression representation and simplification must be co-designed; splitting risks architectural mismatch
- **Phase 3 before Phase 4** - Python bindings expose Rust API; API must be stable before exposure
- **Phase 5 before Phase 6** - differentiators provide user value while identity proving infrastructure is built
- **Linear dependency chain** - each phase uses outputs from all prior phases; parallel development risks integration hell

**Critical path:** Phase 1 -> Phase 2 -> Phase 3. If any of these fail or require rework, later phases cannot proceed.

**Validation gates:**
- After Phase 2: 10,000-term series multiplication completes in <1 second with <100MB memory
- After Phase 3: All 41 qseries functions match Maple output for standard test cases
- After Phase 4: Jupyter notebook can replicate Garvan's tutorial examples
- After Phase 5: Systematic search over parameter space runs 10x faster than Maple

### Research Flags

**Phases needing deeper research during planning:**
- **Phase 3 (prodmake/etamake/jacprodmake)** - Andrews' algorithm for series-to-product conversion is documented in papers but needs implementation strategy research
- **Phase 5 (transformation formulas)** - Gasper-Rahman provides formulas but verification/application logic unclear
- **Phase 6 (identity proving)** - Modular forms theory (cusp computation, valence formula, Ligozat/Martin conditions) is mathematically dense; may need expert consultation
- **Phase 7 (mock theta, Bailey chains)** - Academic literature exists but no packaged implementations found; algorithm extraction needed

**Phases with standard patterns (skip research-phase):**
- **Phase 1** - Hash-consed arenas are well-documented in rustc, SymPy, Maple literature
- **Phase 2** - Phased rewriting is textbook TRS; FPS algorithms in academic CS literature
- **Phase 4** - PyO3 + maturin have comprehensive official documentation

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | rug, egg, PyO3, maturin all verified against official docs; version compatibility confirmed |
| Features | HIGH | Garvan function inventory verified against qseries.org; competitor analysis from official Wolfram/SageMath docs |
| Architecture | MEDIUM-HIGH | Hash-consed DAGs are proven (Maple, JuliaSymbolics); phased rewriting is textbook; specific Rust integration patterns (arena + egg + PyO3) require validation during implementation |
| Pitfalls | MEDIUM-HIGH | Expression swell, rewrite loops, hash consing memory are well-documented CAS issues; q-Pochhammer edge cases from domain literature; PyO3 memory issues from official docs; some claims (10-100x speedup, specific precision thresholds) from training data need benchmarking validation |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

**Architectural gaps:**
- **FormalPowerSeries lazy generator ownership** - Rust's ownership model makes closures-over-mutable-state tricky; may need Rc<RefCell<>> or channel-based design. Resolve in Phase 2 prototyping.
- **egg vs egglog decision** - egg 0.11 is stable but egglog 2.0 is the future. Start with egg, plan migration path. Defer decision until Phase 3 (when rule set is established).
- **C GMP escape hatch necessity** - Unknown if rug's abstraction overhead matters for mock theta hot paths. Profile in Phase 7; implement qsym-gmp crate only if profiling shows need.

**Mathematical gaps:**
- **Andrews' algorithm implementation details** - Papers describe the algorithm but not data structures or termination conditions. Needs Phase 3 research or Garvan source code study.
- **Modular forms infrastructure scope** - Unclear if full thetaids parity requires implementing M_k(Gamma_0(N)) spaces or if cusp/order computation suffices. Needs Phase 6 scoping research.
- **Mock theta convergence conditions** - Unclear if symbolic mock theta computation has edge cases analogous to q-Pochhammer. Needs Phase 7 literature review.

**Ecosystem gaps:**
- **No Rust FPS library** - Must build from scratch. SageMath and Axiom provide design patterns but not code. Plan for 2-3 weeks in Phase 2.
- **flint3-sys Rust wrappers** - flint3-sys provides raw C bindings; need safe Rust wrappers for fmpz_poly, fmpq_poly. Budget 1 week in Phase 2 or defer to Phase 5 if polynomial operations not on critical path.

**Validation gaps:**
- **Performance claims** - "10-100x faster than Maple" is plausible based on Rust vs interpreted Maple but needs benchmarking. Add benchmark suite in Phase 3 with Maple baseline comparison.
- **Numerical precision thresholds** - Claim that |q| in [0.1, 0.5] is "sweet spot" for convergence is domain knowledge, not verified. Validate empirically in Phase 2-3 testing.

## Sources

### Primary (HIGH confidence)

**Stack:**
- [rug 1.28.1 docs](https://docs.rs/rug/latest/rug/) - version, GMP dependency, features
- [egg 0.11.0 docs](https://docs.rs/egg/latest/egg/) - version, API, tutorials
- [PyO3 0.28.0 docs](https://docs.rs/pyo3/latest/pyo3/) - version, memory management, Bound API
- [maturin 1.11.5 on PyPI](https://pypi.org/project/maturin/) - version, project layout
- [python-flint 0.8.0 on PyPI](https://pypi.org/project/python-flint/) - FLINT bindings for Python
- [flint3-sys 3.3.1 docs](https://docs.rs/flint3-sys/latest/flint3_sys/) - FLINT C bindings

**Features:**
- [Garvan qseries v1.3 function list](https://qseries.org/fgarvan/qmaple/qseries/functions/) - official function documentation
- [Garvan thetaids v1.0 function list](https://qseries.org/fgarvan/qmaple/thetaids/functions-v1p0.html) - official function documentation
- [Wolfram Language q Functions guide](https://reference.wolfram.com/language/guide/QFunctions.html) - Mathematica capabilities
- [mpmath q-functions documentation](https://mpmath.org/doc/current/functions/qfunctions.html) - Python q-functions
- [SageMath EtaProducts documentation](https://doc.sagemath.org/html/en/reference/modfrm/sage/modular/etaproducts.html) - SageMath eta-products

**Architecture:**
- [Efficient Symbolic Computation via Hash Consing (arxiv:2509.20534)](https://arxiv.org/html/2509.20534) - JuliaSymbolics hash-consing, 3.2x speedup, 2x memory reduction
- [Maple Programming Guide Appendix](https://www.maplesoft.com/support/help/Maple/view.aspx?path=ProgrammingGuide/Appendix1) - DAG + simplification table
- [Symbolica Expressions](https://symbolica.io/docs/expressions.html) - linear compressed representation
- [SymPy Core System Architecture](https://deepwiki.com/sympy/sympy/2-core-system) - immutable tree model
- [rustc memory management](https://rustc-dev-guide.rust-lang.org/memory.html) - arena + interning patterns

**Pitfalls:**
- [PyO3 Memory Management Guide](https://pyo3.rs/v0.22.5/memory) - GIL Refs deprecation, Bound API
- [SymPy Gotchas and Pitfalls](https://docs.sympy.org/latest/explanation/gotchas.html) - expression representation, equality
- [egg: Equality Saturation Library](https://egraphs-good.github.io/) - e-graph design, rewrite termination
- [Term Rewriting (Berkeley CS294-260)](https://inst.eecs.berkeley.edu/~cs294-260/sp24/2024-01-22-term-rewriting) - termination, confluence

### Secondary (MEDIUM confidence)

- [Garvan q-product tutorial (arxiv:math/9812092)](https://arxiv.org/abs/math/9812092) - Andrews' algorithm description
- [Garvan ETA tutorial (arxiv:1907.09130)](https://arxiv.org/abs/1907.09130) - ETA package methodology
- [Garvan auto-theta paper (arxiv:1807.08051)](https://arxiv.org/abs/1807.08051) - thetaids/ramarobinsids
- [Things I Would Like to See in a CAS - Fredrik Johansson](https://fredrikj.net/blog/2022/04/things-i-would-like-to-see-in-a-computer-algebra-system/) - CAS design critique
- [Ensuring Termination of EqSat over a Terminating TRS](https://effect.systems/blog/ta-completion.html) - non-termination in equality saturation
- [Malachite performance page](https://www.malachite.rs/performance/) - benchmarks vs rug, num
- [hashconsing crate docs](https://docs.rs/hashconsing) - Filiatre-Conchon implementation
- [E-Graphs in Rust (Stephen Diehl)](https://www.stephendiehl.com/posts/egraphs/) - architecture patterns

### Tertiary (LOW confidence - needs validation)

- q-Pochhammer convergence thresholds (|q| in [0.1, 0.5]) - domain knowledge from training data, not verified source
- 10-100x Rust vs Maple performance claim - plausible based on interpreted vs compiled, needs benchmarking
- Mock theta function implementation complexity - inferred from literature, not implementation experience
- Bailey chain algorithm details - academic papers describe theory, not implementation

---

**Research completed:** 2026-02-13
**Ready for roadmap:** Yes
**Next step:** Roadmap creation using phase structure and research flags from this summary
