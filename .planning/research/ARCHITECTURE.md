# Architecture Research: Q-Symbolic Computation Engine

**Domain:** Symbolic computation engine for q-series mathematics
**Researched:** 2026-02-13
**Confidence:** MEDIUM-HIGH (strong evidence from CAS literature and Rust ecosystem; some areas require validation during implementation)

---

## Executive Summary

This document synthesizes how mature CAS systems represent and manipulate symbolic expressions, and translates those lessons into a concrete architecture for Q-Symbolic -- a Rust-core, Python-API symbolic computation engine specialized for q-series. The architecture draws from Maple's DAG-with-simplification-table approach (the system Q-Symbolic replaces), SymPy's immutable tree model, Symbolica's linear compressed representation, and the egg/egglog equality saturation framework. The recommendations are opinionated: where multiple valid approaches exist, this document picks one and explains why.

---

## System Overview

```
+------------------------------------------------------------------+
|                        Python Layer                               |
|  +---------------+  +---------------+  +----------------------+  |
|  | qsymbolic     |  | LaTeX /       |  | Session              |  |
|  | DSL & API     |  | Display       |  | Management           |  |
|  +-------+-------+  +-------+-------+  +----------+-----------+  |
|          |                  |                      |              |
+----------+------------------+----------------------+--------------+
           |     PyO3 FFI boundary (GIL-released)    |
+----------+------------------+----------------------+--------------+
|                        Rust Core Engine                           |
|                                                                   |
|  +-------------------------------------------------------------+ |
|  |                    Expression Layer                           | |
|  |  +-------------+  +---------------+  +--------------------+  | |
|  |  | ExprArena   |  | Expr enum     |  | Symbol Registry    |  | |
|  |  | (hash-cons) |  | (node types)  |  | (append-only)      |  | |
|  |  +------+------+  +-------+-------+  +---------+----------+  | |
|  |         |                 |                     |             | |
|  +---------+-----------------+---------------------+-------------+ |
|            |                 |                     |               |
|  +---------+-----------------+---------------------+-------------+ |
|  |                 Rewrite / Simplification Layer                | |
|  |  +-------------+  +---------------+  +--------------------+  | |
|  |  | Pattern     |  | Rule Engine   |  | E-graph            |  | |
|  |  | Matcher     |  | (phased)      |  | (equality sat.)    |  | |
|  |  +-------------+  +---------------+  +--------------------+  | |
|  +--------------------------------------------------------------+ |
|            |                 |                     |               |
|  +---------+-----------------+---------------------+-------------+ |
|  |                    Domain Module Layer                         | |
|  |  +----------+ +----------+ +--------+ +--------+ +--------+  | |
|  |  |q-Pochh.  | |Hypergeo  | |Theta / | |Mock    | |Bailey/ |  | |
|  |  |& q-Binom | |_r phi _s | |Eta     | |Theta   | |WZ      |  | |
|  |  +----------+ +----------+ +--------+ +--------+ +--------+  | |
|  +--------------------------------------------------------------+ |
|            |                 |                     |               |
|  +---------+-----------------+---------------------+-------------+ |
|  |                   Foundation Layer                             | |
|  |  +-------------+  +---------------+  +--------------------+  | |
|  |  | Arithmetic  |  | Formal Power  |  | Identity           |  | |
|  |  | (rug/GMP)   |  | Series        |  | Database           |  | |
|  |  +-------------+  +---------------+  +--------------------+  | |
|  +--------------------------------------------------------------+ |
+-------------------------------------------------------------------+
```

---

## Component Responsibilities

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| **ExprArena** | Hash-consing interner: ensures structural dedup, O(1) equality, owns all expression memory | Every component that creates or inspects expressions |
| **Expr enum** | Node type definitions (atoms, arithmetic ops, q-specific primitives, summation/product) | ExprArena (storage), Pattern Matcher (inspection), Domain Modules (construction) |
| **Symbol Registry** | Append-only global name table mapping SymbolId -> name + attributes | ExprArena, Python layer (symbol creation) |
| **Pattern Matcher** | Structural pattern matching with wildcards, conditions, level restrictions | Rule Engine (match phase), Domain Modules (identity detection) |
| **Rule Engine** | Phased simplification: applies rewrite rules in priority order, manages rule sets per domain | Pattern Matcher, E-graph (for hard cases), Domain Modules (rule providers) |
| **E-graph** | Equality saturation for discovering non-obvious equivalences among q-series expressions | Rule Engine (as backend for phase 6 deep simplification), Identity Database |
| **Domain Modules** | q-Pochhammer, hypergeometric, theta, mock theta, Bailey, WZ -- each provides rules + operations | Rule Engine (register rules), ExprArena (construct expressions), Arithmetic (compute), FPS (expand) |
| **Arithmetic** | BigInt/BigRat via `rug` crate (GMP), with C FFI escape hatch for hot paths | ExprArena (numeric leaf nodes), FPS (coefficient arithmetic), Domain Modules |
| **Formal Power Series** | Sparse truncated Laurent series with lazy coefficient computation | Domain Modules (series expansion), Arithmetic (coefficient ops), ExprArena (FPS nodes) |
| **Identity Database** | Searchable collection of verified q-series identities with citations and proof methods | Rule Engine (lookup), Domain Modules (populate), Python layer (search API) |
| **Python API** | PyO3 bindings exposing DSL, display, session management | Rust core (all components via FFI boundary) |

---

## Architectural Decisions

### Decision 1: Expression Representation -- Hash-Consed Arena (DAG)

**Recommendation:** Use a custom arena-based hash-consing interner. Expressions are stored in a `Vec<Expr>` with a `HashMap<Expr, ExprRef>` dedup table. `ExprRef` is a 32-bit index (not a pointer).

**Why this over alternatives:**

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| **Plain AST (SymPy-style)** | Simple, natural tree traversal | Duplicates subexpressions, O(n) equality, expression swell in q-series | Reject |
| **Hash-consed DAG (Maple-style)** | O(1) equality, no duplication, cache-friendly arena | Algorithms must be DAG-aware, not just tree-aware | **Use this** |
| **Linear compressed (Symbolica-style)** | 8x smaller than Mathematica, cache-optimal | Complex to implement, harder to extend with new node types | Too exotic for v1 |
| **E-graph as primary representation** | Native equivalence reasoning | Not designed as primary storage, extraction overhead | Use as secondary |

**Confidence:** HIGH. Hash-consing is the established pattern for CAS engines:
- Maple uses a "simplification table" ensuring unique instance representation with pointer-based equality (verified via official Maple Programming Guide Appendix)
- JuliaSymbolics integrated hash-consing in Sep 2025, achieving 3.2x speedup and 2x memory reduction (Zhu et al., arXiv:2509.20534)
- The `hashconsing` Rust crate provides a production-ready foundation based on Filiatre & Conchon's type-safe approach
- Rustc itself uses arena allocation + interning for its IR (verified via rustc-dev-guide)

**Critical design details:**

```rust
/// 32-bit index into the arena. Copy, cheap to pass around.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExprRef(u32);

/// The arena owns all expression memory. Single instance per computation context.
pub struct ExprArena {
    /// Dense storage -- cache-friendly sequential access
    nodes: Vec<Expr>,
    /// Dedup table: hash(Expr) -> ExprRef for O(1) lookup on construction
    dedup: HashMap<Expr, ExprRef>,
    /// Reverse: ExprRef -> &Expr is just nodes[ref.0]
}

impl ExprArena {
    /// The core operation: intern an expression. Returns existing ref if
    /// structurally identical expression already exists.
    pub fn intern(&mut self, expr: Expr) -> ExprRef {
        if let Some(&existing) = self.dedup.get(&expr) {
            return existing;
        }
        let id = ExprRef(self.nodes.len() as u32);
        self.nodes.push(expr.clone());
        self.dedup.insert(expr, id);
        id
    }

    pub fn get(&self, r: ExprRef) -> &Expr { &self.nodes[r.0 as usize] }
}
```

**Why u32 index, not Arc/pointer:**
- 4 billion expressions is more than enough for any single session
- u32 is Copy -- no reference counting overhead
- Cache-friendly: sequential IDs often mean sequential memory access
- Trivially serializable (important for save/load sessions)
- No lifetime parameter pollution through the entire codebase

**Why NOT use the `hashconsing` crate directly:**
- Its `HConsed<T>` uses `Arc` internally -- unnecessary overhead when we own the arena
- Its `uid` system conflicts with our u32 index approach
- Rolling our own is ~50 lines and gives full control over memory layout
- But study its API for design inspiration, especially the invariant that "structurally identical real terms have equal uids"

### Decision 2: Expr Node Design -- N-ary Ops with Sorted Children

**Recommendation:** Use n-ary Add and Mul (not binary), with children sorted in canonical order during construction. This matches how Symbolica, SymPy, and Maple all work.

```rust
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Expr {
    // --- Atoms ---
    Integer(BigInt),                    // arbitrary precision via rug
    Rational(BigRat),                   // exact fractions
    Symbol(SymbolId),                   // interned name reference
    Infinity,                           // formal infinity for (a;q)_inf

    // --- Arithmetic (n-ary, canonically sorted) ---
    Add(Vec<ExprRef>),                  // summands, sorted
    Mul(Vec<ExprRef>),                  // factors, sorted
    Pow(ExprRef, ExprRef),              // base^exponent

    // --- q-Specific Primitives ---
    QPochhammer {
        base: ExprRef,                  // a in (a;q)_n
        nome: ExprRef,                  // q
        order: ExprRef,                 // n (can be Infinity)
    },
    QBinomial {
        n: ExprRef,
        k: ExprRef,
        nome: ExprRef,
    },
    BasicHypergeometric {
        upper: SmallVec<[ExprRef; 4]>,  // a_1..a_r (usually r <= 4)
        lower: SmallVec<[ExprRef; 4]>,  // b_1..b_s
        nome: ExprRef,
        argument: ExprRef,
    },

    // --- Theta & Modular ---
    JacobiTheta(u8, ExprRef, ExprRef),  // index, argument, nome
    DedekindEta(ExprRef),               // tau
    RamanujanTheta(ExprRef, ExprRef),   // a, b

    // --- Summation / Product ---
    Sum { body: ExprRef, var: SymbolId, lo: ExprRef, hi: ExprRef },
    Product { body: ExprRef, var: SymbolId, lo: ExprRef, hi: ExprRef },

    // --- Formal Power Series (opaque, lives in separate storage) ---
    FPS(FPSRef),

    // --- Meta ---
    Undefined,
}
```

**Key design choices within Expr:**

1. **No `Neg` or `Inv` variants.** Negation is `Mul([-1, x])`. Inverse is `Pow(x, -1)`. This reduces pattern matching cases and is how SymPy and Symbolica both work. Canonicalization handles the rest.

2. **`SmallVec<[ExprRef; 4]>` for hypergeometric params.** Most basic hypergeometric series are `_2phi_1` or `_3phi_2`. SmallVec avoids heap allocation for the common case while supporting arbitrary r,s.

3. **`FPSRef` for formal power series.** FPS data is bulky (coefficient maps). Store it in a separate `FPSArena` and reference by index, keeping the Expr enum small. This is critical: the Expr enum should fit in ~64 bytes max for cache efficiency.

4. **Sorted children in Add/Mul.** Canonical ordering guarantees that `x + y` and `y + x` produce identical arena entries, which is essential for hash-consing to work. Use a total ordering on ExprRef (which is just u32 comparison) after first sorting by expression structure.

**Confidence:** HIGH. This is how every mature CAS works -- SymPy, Maple, and Symbolica all use n-ary sorted commutative operators.

### Decision 3: Rewrite Engine -- Phased Rule Application (NOT Pure E-graph)

**Recommendation:** Use a phased term-rewriting system as the primary simplification engine. Integrate e-graph equality saturation as an optional deep-simplification pass for hard cases, NOT as the default simplification strategy.

**Why phased rewriting over pure equality saturation:**

1. **Predictability.** Researchers need to understand what the simplifier does. Phased rewriting ("first normalize arithmetic, then combine q-Pochhammers, then recognize hypergeometric forms") is transparent. E-graph extraction can produce surprising results.

2. **Performance.** Most q-series simplifications are straightforward: combine like q-Pochhammer terms, apply known summation formulas. These don't need the heavy machinery of e-graph saturation. Bottom-up rule application is O(n) average case.

3. **Rule ordering is natural in q-series.** The simplification priority from PROJECT.md maps directly to phases:
   - Phase 1: Arithmetic normalization (flatten, combine, reduce)
   - Phase 2: q-Pochhammer algebra (split/combine/cancel)
   - Phase 3: Hypergeometric recognition (detect _r phi _s patterns)
   - Phase 4: Known identity lookup (database matching)
   - Phase 5: Series expansion comparison (expand both sides)
   - Phase 6: E-graph saturation (when all else fails, explore equivalence space)

4. **E-graphs shine for discovery, not routine simplification.** When a researcher asks "are these two complicated expressions equal?", e-graph saturation can explore the equivalence space and find non-obvious paths. That is phase 6, not phase 1.

**Architecture:**

```rust
pub struct SimplificationEngine {
    /// Phases run in order. Each phase has a set of rules.
    phases: Vec<SimplificationPhase>,
    /// Optional e-graph backend for deep equivalence checking
    egraph: Option<EGraphBackend>,
}

pub struct SimplificationPhase {
    name: &'static str,
    rules: Vec<RewriteRule>,
    /// How many times to iterate this phase before moving on
    max_iterations: usize,
    /// Whether to run bottom-up or top-down
    strategy: TraversalStrategy,
}

pub enum TraversalStrategy {
    BottomUp,   // simplify children first, then parent (default, most phases)
    TopDown,    // try to match at root first (useful for expansion)
}

pub struct RewriteRule {
    name: &'static str,
    pattern: Pattern,
    replacement: PatternOrFn,
    condition: Option<Condition>,
    reference: Option<&'static str>,  // e.g. "Gasper-Rahman Thm 1.3.2"
}
```

**Confidence:** HIGH for phased rewriting as primary. MEDIUM for e-graph integration timing -- the egg library is well-suited (5000 lines of Rust, battle-tested), but integrating it with a custom expression representation requires adapter work. Consider egglog (Datalog + equality saturation) as the e-graph backend since it provides typechecked rules and incremental execution.

### Decision 4: Formal Power Series -- Sparse Map with Lazy Coefficient Computation

**Recommendation:** Represent formal power series as sparse coefficient maps (`BTreeMap<i64, Coefficient>`) with a truncation order and optional lazy generator function.

```rust
pub struct FormalPowerSeries {
    /// Sparse coefficients: exponent -> value. BTreeMap for ordered iteration.
    coefficients: BTreeMap<i64, BigRat>,
    /// Variable this series is in (usually q)
    variable: SymbolId,
    /// Known precision: coefficients are exact up to this order
    known_order: i64,
    /// Maximum order to compute (truncation bound)
    truncation: i64,
    /// Optional lazy generator: given current state, compute next coefficients
    generator: Option<Box<dyn FnMut(&mut FormalPowerSeries)>>,
}
```

**Why sparse over dense:**
- q-series are inherently sparse. The partition generating function `1/(q;q)_inf` has many zero coefficients in intermediate computations
- Rogers-Ramanujan type products have coefficients supported on arithmetic progressions
- Dense representation wastes memory for Laurent series with large gaps
- `BTreeMap<i64, BigRat>` gives ordered iteration (important for series printing and truncation) plus O(log n) access

**Why `BTreeMap` over `HashMap`:**
- Ordered iteration is needed constantly (printing, truncation, coefficient extraction)
- For typical series sizes (100-1000 terms), BTreeMap's O(log n) vs HashMap's O(1) is negligible
- BTreeMap avoids hash computation overhead on BigRat (which can be expensive)

**Lazy evaluation strategy:**
- For infinite products like `(a;q)_inf`, store a generator function that computes coefficients on demand
- When arithmetic operations need coefficient [q^n] and it hasn't been computed yet, invoke the generator up to order n
- This avoids computing thousands of coefficients when only the first 20 are needed
- Aligns with Axiom's stream-based approach and SageMath's lazy series

**Confidence:** HIGH for sparse representation (standard in q-series CAS). MEDIUM for the lazy generator approach (correct in principle, but the Rust ownership model makes closures-over-mutable-state tricky -- may need `Rc<RefCell<>>` or a channel-based design).

### Decision 5: Domain Module Architecture -- Trait-Based with Static Registration

**Recommendation:** Use a trait-based module system where each domain module implements a `DomainModule` trait, with static registration at compile time (not runtime dynamic loading).

```rust
/// Every domain module implements this trait
pub trait DomainModule {
    /// Human-readable name
    fn name(&self) -> &'static str;

    /// Rewrite rules this module provides, organized by phase
    fn rules(&self) -> Vec<(SimplificationPhase, Vec<RewriteRule>)>;

    /// Custom operations this module provides (e.g., expand_qpoch, heine_transform)
    fn operations(&self) -> Vec<(&'static str, Box<dyn Operation>)>;

    /// Identities this module contributes to the database
    fn identities(&self) -> Vec<Identity>;

    /// Given an expression, can this module recognize/simplify it?
    /// Returns None if not applicable, Some(simplified) if it can help.
    fn try_simplify(&self, expr: ExprRef, arena: &ExprArena) -> Option<ExprRef>;
}

/// Register modules at compile time via inventory or ctor crate
inventory::submit! { QPochhammerModule::new() }
inventory::submit! { HypergeometricModule::new() }
inventory::submit! { ThetaModule::new() }
// etc.
```

**Why static registration over dynamic plugins:**
- Q-Symbolic's modules are known at compile time -- we're not building a marketplace
- Static dispatch is faster than dynamic dispatch (no vtable indirection in hot loops)
- Rust's type system catches module interface errors at compile time
- The `inventory` crate provides a clean pattern for auto-registration without manual wiring
- Dynamic loading (via `libloading`) adds complexity with minimal benefit for a math library

**Why trait-based over registry-based:**
- Traits enforce the contract: every module MUST provide rules, operations, and identities
- Trait objects (`Box<dyn DomainModule>`) still allow heterogeneous collections when needed
- Each module is a separate Rust module (file), promoting clean boundaries
- Modules can depend on each other through the arena (e.g., theta module constructs q-Pochhammer expressions)

**Module dependency graph:**

```
Arithmetic (rug/GMP)
    |
    v
ExprArena + Expr
    |
    v
Pattern Matcher
    |
    v
q-Pochhammer Module  <-- foundational, everything depends on this
    |         |
    v         v
Hypergeometric    Theta/Eta Module
    |                |
    v                v
Partitions     Mock Theta
    |                |
    v                v
Bailey Chains    WZ Method
```

**Confidence:** HIGH. Trait-based modular architecture is idiomatic Rust. The `inventory` crate pattern is used by multiple production Rust projects.

### Decision 6: FFI Architecture -- PyO3 with Opaque Handles

**Recommendation:** Expose opaque handles (wrappers around `ExprRef`) to Python, NOT full expression trees. Python never sees or manipulates the Rust expression tree directly. All computation happens in Rust; Python is the orchestration and display layer.

**Project layout (mixed Rust/Python with maturin):**

```
q-symbolic/
+-- Cargo.toml              # workspace root
+-- pyproject.toml           # maturin build config
|
+-- crates/
|   +-- qsym-core/          # Pure Rust core engine
|   |   +-- Cargo.toml
|   |   +-- src/
|   |       +-- lib.rs
|   |       +-- expr.rs
|   |       +-- arena.rs
|   |       +-- rewrite.rs
|   |       +-- fps.rs
|   |       +-- modules/
|   |           +-- mod.rs
|   |           +-- qpoch.rs
|   |           +-- hypergeo.rs
|   |           +-- theta.rs
|   |           +-- ...
|   |
|   +-- qsym-gmp/           # Direct C GMP FFI for hot paths
|   |   +-- Cargo.toml
|   |   +-- src/lib.rs
|   |   +-- gmp_bindings.c  # Optional hand-tuned C for mock theta
|   |
|   +-- qsym-python/        # PyO3 bindings crate
|       +-- Cargo.toml      # depends on qsym-core
|       +-- src/lib.rs      # #[pymodule] definitions
|
+-- python/
|   +-- qsymbolic/          # Pure Python layer
|       +-- __init__.py
|       +-- _core.pyi       # Type stubs for Rust bindings
|       +-- dsl.py          # User-facing expression builder
|       +-- display.py      # LaTeX / Unicode rendering
|       +-- session.py      # Computation context management
|
+-- identities/             # Identity database (TOML files)
+-- tests/
+-- benches/
```

**PyO3 boundary design:**

```rust
// In qsym-python/src/lib.rs

/// Opaque handle exposed to Python. Python cannot peek inside.
#[pyclass]
pub struct QExpr {
    inner: ExprRef,
    /// Reference to the session's arena (shared via Arc)
    session: Arc<Mutex<Session>>,
}

#[pymethods]
impl QExpr {
    /// All arithmetic goes through Rust
    fn __add__(&self, other: &QExpr) -> PyResult<QExpr> {
        let mut session = self.session.lock().unwrap();
        let result = session.arena.intern(Expr::Add(vec![self.inner, other.inner]));
        // Simplify immediately (arithmetic normalization phase only)
        let simplified = session.engine.quick_simplify(result, &mut session.arena);
        Ok(QExpr { inner: simplified, session: self.session.clone() })
    }

    /// Long computations release the GIL
    fn simplify(&self, py: Python<'_>) -> PyResult<QExpr> {
        let session = self.session.clone();
        let expr = self.inner;
        // Release GIL for potentially expensive Rust computation
        let result = py.allow_threads(|| {
            let mut s = session.lock().unwrap();
            s.engine.full_simplify(expr, &mut s.arena)
        });
        Ok(QExpr { inner: result, session: self.session.clone() })
    }

    fn to_latex(&self) -> String {
        let session = self.session.lock().unwrap();
        latex::render(self.inner, &session.arena)
    }
}
```

**Key FFI principles:**

1. **Release GIL for all non-trivial Rust work.** Use `py.allow_threads()` for simplification, series expansion, identity verification. This lets Python threads (and Jupyter's event loop) run while Rust computes.

2. **Minimize boundary crossings.** Don't expose low-level operations (individual tree traversal steps). Expose high-level operations (simplify, expand, verify, search). Each Python call should do substantial Rust work.

3. **Opaque handles, not transparent trees.** Python gets `QExpr` objects that are black-box references. If Python needs to inspect structure, provide methods like `args()`, `head()`, `is_add()` -- don't serialize the whole tree.

4. **Session object manages lifetime.** A `Session` (or `Context`) owns the `ExprArena`, `SimplificationEngine`, and `IdentityDatabase`. Python's `QExpr` objects hold `Arc<Mutex<Session>>` references. This prevents lifetime issues and makes the API Pythonic.

5. **C GMP escape hatch.** The `qsym-gmp` crate provides direct C FFI for hot mock theta paths. It links against `gmp-mpfr-sys` at the low level, bypassing `rug`'s abstractions when profiling shows they're the bottleneck. This is a targeted optimization, not the default path.

**Confidence:** HIGH. PyO3 + maturin is the standard Rust-Python binding approach. The opaque handle pattern is used by Symbolica, polars, and other Rust-backed Python libraries.

### Decision 7: Arithmetic Foundation -- rug (GMP) with Escape Hatch

**Recommendation:** Use the `rug` crate for arbitrary precision arithmetic (BigInt, BigRat, BigFloat). Reserve direct `gmp-mpfr-sys` FFI calls for profiled hot paths only.

**Why rug over alternatives:**

| Library | Performance | Purity | API | Verdict |
|---------|-------------|--------|-----|---------|
| `rug` (GMP wrapper) | Fastest for large numbers | C dependency (GMP) | Excellent, Rustic | **Use this** |
| `num-bigint` | 2-10x slower than GMP | Pure Rust | Good | Too slow for q-series |
| `malachite` | Competitive with GMP | Pure Rust | Complex | Consider as fallback if GMP causes build issues |
| `ibig` | Good for moderate sizes | Pure Rust | Minimal | Not sufficient for arbitrary precision q-series |

**Evidence:** Benchmark comparisons consistently show rug/GMP 2-10x faster than num-bigint for large-precision arithmetic (verified via multiple benchmark repos). For q-series where coefficients can grow to thousands of digits (e.g., partition function p(1000)), this performance gap is critical.

**The escape hatch:**

```rust
// In qsym-gmp/src/lib.rs
// Direct GMP calls for inner loops where rug's abstraction overhead matters

extern "C" {
    fn mpz_mul(rop: *mut mpz_struct, op1: *const mpz_struct, op2: *const mpz_struct);
    // ... other GMP functions
}

/// Batch multiply a vector of BigInts -- avoids rug's per-operation overhead
/// for the innermost loops of mock theta function evaluation
pub fn batch_multiply_gmp(values: &[rug::Integer]) -> rug::Integer {
    // Access raw mpz_t pointers, operate directly, return result
    // Only use when profiling shows this matters
    unimplemented!()
}
```

**Confidence:** HIGH for rug as default. MEDIUM for the escape hatch need -- profile first, then optimize. The rug -> gmp-mpfr-sys path is well-documented since rug exposes its inner GMP structures.

---

## Data Flow

### Expression Construction Flow

```
Python: qpoch(a, q, 5)
    |
    v (PyO3 call)
Rust: Session.lock() -> arena.intern(Symbol("a")) -> ExprRef(0)
                      -> arena.intern(Symbol("q")) -> ExprRef(1)
                      -> arena.intern(Integer(5))   -> ExprRef(2)
                      -> arena.intern(QPochhammer { base: 0, nome: 1, order: 2 })
                      -> ExprRef(3)  // deduplicated if already exists
    |
    v (quick simplify -- arithmetic normalization only)
    ExprRef(3)  // no simplification needed for construction
    |
    v (wrap in opaque handle)
Python: QExpr(inner=3, session=Arc<Session>)
```

### Simplification Flow

```
Python: expr.simplify()
    |
    v (release GIL, enter Rust)
SimplificationEngine::full_simplify(expr, arena)
    |
    v Phase 1: Arithmetic normalization
    Bottom-up traversal: flatten Add/Mul, combine numeric terms, reduce Rational
    |
    v Phase 2: q-Pochhammer algebra
    Match patterns: (a;q)_m * (aq^m;q)_n -> (a;q)_{m+n}
    Match patterns: (a;q)_inf / (a;q)_n -> (aq^n;q)_inf
    |
    v Phase 3: Hypergeometric recognition
    Detect if sum matches _r phi _s structure
    |
    v Phase 4: Known identity lookup
    Hash expression, search identity database
    |
    v Phase 5 (if requested): Series expansion comparison
    Expand to O(q^N), compare coefficient-by-coefficient
    |
    v Phase 6 (if requested): E-graph deep search
    Build e-graph, saturate with q-series rules, extract simplest
    |
    v Return simplified ExprRef
Python: QExpr(inner=simplified_ref)
```

### Series Expansion Flow

```
Request: expand_q(expr, q, order=20)
    |
    v
Dispatch based on Expr variant:
    QPochhammer { base, nome, order: Infinity } ->
        Lazy FPS generator:
            (a;q)_inf = prod_{k=0}^{inf} (1 - a*q^k)
            Compute partial products up to O(q^20)
            Store in FormalPowerSeries { coefficients: BTreeMap, ... }
    |
    QPochhammer { base, nome, order: Integer(n) } ->
        Direct expansion: multiply n factors (1 - a*q^k)
        Each factor is a binomial in q, multiply sequentially
    |
    Add/Mul/Pow ->
        Recursively expand children, then combine FPS
        FPS * FPS: convolve coefficients
        FPS + FPS: add corresponding coefficients
    |
    v
Return FPS truncated to O(q^20)
```

---

## Recommended Project Structure

```
q-symbolic/
+-- Cargo.toml                  # [workspace] with members
+-- pyproject.toml              # maturin config, tool.maturin.python-source = "python"
+-- rust-toolchain.toml         # pin Rust version
|
+-- crates/
|   +-- qsym-core/             # THE core library -- no Python dependency
|   |   +-- Cargo.toml         # depends on: rug, smallvec, inventory
|   |   +-- src/
|   |       +-- lib.rs          # public API re-exports
|   |       +-- expr.rs         # Expr enum definition
|   |       +-- arena.rs        # ExprArena (hash-consing interner)
|   |       +-- symbol.rs       # SymbolId, SymbolRegistry
|   |       +-- canonical.rs    # Canonicalization rules (sort, flatten, normalize)
|   |       +-- pattern.rs      # Pattern type, wildcard matching
|   |       +-- rewrite.rs      # RewriteRule, SimplificationEngine, phases
|   |       +-- egraph.rs       # E-graph adapter (wraps egg or egglog)
|   |       +-- fps.rs          # FormalPowerSeries, FPSArena, lazy generators
|   |       +-- arithmetic.rs   # BigInt/BigRat wrappers, coefficient operations
|   |       +-- latex.rs        # LaTeX rendering
|   |       +-- pretty.rs       # Unicode terminal rendering
|   |       +-- identity.rs     # Identity struct, IdentityDatabase, search
|   |       +-- session.rs      # Session context (owns arena, engine, DB)
|   |       +-- modules/
|   |           +-- mod.rs      # DomainModule trait, module registry
|   |           +-- qpoch.rs    # q-Pochhammer, q-binomial, q-factorial
|   |           +-- hypergeo.rs # _r phi _s, transformations, summations
|   |           +-- theta.rs    # Jacobi theta, Dedekind eta, Ramanujan theta
|   |           +-- partitions.rs # Partition function, generating functions
|   |           +-- mock_theta.rs # Mock theta functions, Appell-Lerch
|   |           +-- bailey.rs   # Bailey pairs, chains, lemma
|   |           +-- wz.rs       # q-Zeilberger, creative telescoping
|   |
|   +-- qsym-gmp/              # Direct C GMP escape hatch (optional)
|   |   +-- Cargo.toml         # depends on: gmp-mpfr-sys
|   |   +-- src/lib.rs         # Raw GMP operations for hot paths
|   |   +-- build.rs           # Link GMP
|   |
|   +-- qsym-python/           # PyO3 bindings
|       +-- Cargo.toml         # depends on: qsym-core, pyo3
|       +-- src/
|           +-- lib.rs          # #[pymodule] qsymbolic._core
|           +-- expr.rs         # QExpr pyclass wrapper
|           +-- session.rs      # QSession pyclass
|           +-- display.rs      # _repr_latex_ hooks
|           +-- convert.rs      # Python <-> Rust type conversions
|
+-- python/
|   +-- qsymbolic/
|       +-- __init__.py         # from qsymbolic._core import ...
|       +-- _core.pyi           # Type stubs for IDE support
|       +-- dsl.py              # symbols(), qpoch(), hyper_q(), etc.
|       +-- display.py          # Jupyter _repr_latex_, Unicode
|       +-- session.py          # QSession Python wrapper
|
+-- identities/                 # TOML identity files
|   +-- euler.toml
|   +-- rogers_ramanujan.toml
|   +-- theta.toml
|   +-- mock_theta.toml
|   +-- slater_list.toml
|   +-- bailey_pairs.toml
|   +-- summation_formulas.toml
|
+-- tests/
|   +-- rust/                   # Cargo test modules
|   +-- python/                 # pytest test files
|   +-- identities/            # Identity verification test suite
|
+-- benches/                    # Criterion benchmarks
+-- examples/
    +-- notebooks/
    +-- scripts/
```

### Structure Rationale

- **`crates/` workspace:** Separates concerns cleanly. `qsym-core` has zero Python dependency, making it testable and usable as a pure Rust library. `qsym-python` is a thin binding layer. `qsym-gmp` is an opt-in performance crate.
- **`python/qsymbolic/`** is the user-facing Python package. It re-exports from `_core` (the Rust bindings) and adds pure Python conveniences (DSL, display).
- **`identities/`** is data, not code. TOML files are human-readable and researcher-editable. Loaded at `Session` initialization.
- **Separate `canonical.rs` from `rewrite.rs`:** Canonicalization (sorting, flattening) happens at construction time in the arena. Rewriting happens later on request. These are different concerns.

---

## Architectural Patterns

### Pattern 1: Arena-Threaded Computation

**What:** All functions that create or modify expressions take `&mut ExprArena` as a parameter. Expressions are never created in isolation.

**When to use:** Everywhere in the core engine.

**Trade-offs:** Requires threading the arena through call stacks. Can be verbose. But eliminates the possibility of dangling references or orphaned expressions.

```rust
fn simplify_add(terms: &[ExprRef], arena: &mut ExprArena) -> ExprRef {
    let mut numeric_sum = BigRat::zero();
    let mut symbolic_terms = Vec::new();

    for &term in terms {
        match arena.get(term) {
            Expr::Rational(r) => numeric_sum += r,
            _ => symbolic_terms.push(term),
        }
    }

    if !numeric_sum.is_zero() {
        symbolic_terms.push(arena.intern(Expr::Rational(numeric_sum)));
    }

    match symbolic_terms.len() {
        0 => arena.intern(Expr::Rational(BigRat::zero())),
        1 => symbolic_terms[0],
        _ => {
            symbolic_terms.sort(); // canonical ordering
            arena.intern(Expr::Add(symbolic_terms))
        }
    }
}
```

### Pattern 2: Phased Simplification with Early Exit

**What:** Run simplification phases in priority order. After each phase, check if the expression changed. If it did, restart from phase 1 (since earlier phases may now apply). Cap total iterations to prevent infinite loops.

**When to use:** The `full_simplify` entry point.

```rust
fn full_simplify(expr: ExprRef, arena: &mut ExprArena, engine: &SimplificationEngine) -> ExprRef {
    let mut current = expr;
    let mut total_iters = 0;
    const MAX_ITERS: usize = 100;

    'outer: while total_iters < MAX_ITERS {
        for phase in &engine.phases {
            let result = phase.apply(current, arena);
            if result != current {  // O(1) check thanks to hash-consing!
                current = result;
                total_iters += 1;
                continue 'outer;  // restart from phase 1
            }
        }
        break; // no phase changed anything -- we're done
    }
    current
}
```

**Key insight:** The `result != current` check is O(1) because hash-consed expressions have identical ExprRef iff structurally identical. This is the payoff of hash-consing.

### Pattern 3: Module-Provided Rules with Phase Tags

**What:** Each domain module provides rewrite rules tagged with which simplification phase they belong to. The engine collects all rules from all modules and organizes them into phases.

**When to use:** System initialization.

```rust
fn build_engine(modules: &[Box<dyn DomainModule>]) -> SimplificationEngine {
    let mut phase_rules: BTreeMap<usize, Vec<RewriteRule>> = BTreeMap::new();

    for module in modules {
        for (phase, rules) in module.rules() {
            phase_rules.entry(phase.priority).or_default().extend(rules);
        }
    }

    SimplificationEngine {
        phases: phase_rules.into_iter().map(|(pri, rules)| {
            SimplificationPhase { priority: pri, rules, .. }
        }).collect(),
        egraph: None, // initialized lazily on first deep-simplify call
    }
}
```

---

## Anti-Patterns

### Anti-Pattern 1: Exposing Raw Expression Trees to Python

**What people do:** Serialize the entire Rust expression tree to Python dicts/lists, let Python manipulate it, serialize back.

**Why it's wrong:** Massive FFI overhead. Breaks hash-consing invariants. Python GC and Rust arena disagree about lifetimes. Expression manipulation in Python is 100-1000x slower than Rust.

**Do this instead:** Opaque handles (`QExpr` wrapping `ExprRef`). Python orchestrates; Rust computes. Provide Rust-side inspection methods (`args()`, `head()`, `is_qpoch()`) that return Python-friendly values.

### Anti-Pattern 2: One Giant Simplify Function

**What people do:** A single `simplify()` function with a massive match statement handling all cases.

**Why it's wrong:** Unmaintainable. Cannot add new simplification rules without modifying core code. No way to control simplification order or disable specific transformations.

**Do this instead:** Phased engine with module-provided rules. Each module owns its rules. The engine orchestrates.

### Anti-Pattern 3: Eager Full Expansion

**What people do:** When asked to simplify `(a;q)_inf * (b;q)_inf`, immediately expand both to O(q^1000) and multiply coefficient-by-coefficient.

**Why it's wrong:** Destroys symbolic structure. Many simplifications (like `(a;q)_inf / (a;q)_n = (aq^n;q)_inf`) can be done symbolically without expanding. Expansion should be a last resort, not the first step.

**Do this instead:** Keep expressions in symbolic form as long as possible. Expansion is phase 5 of simplification, used only when symbolic methods fail or when the user explicitly requests it.

### Anti-Pattern 4: Mutable Expression Nodes

**What people do:** Modify expression nodes in-place during simplification.

**Why it's wrong:** Breaks hash-consing invariants (other references to the same node see unexpected changes). Makes multi-threaded simplification impossible. Makes caching unreliable.

**Do this instead:** Expressions are immutable once interned. Simplification creates new expressions in the arena. The old expression continues to exist (and will be deduplicated away if no longer referenced, or reused if the simplification produces the same result).

### Anti-Pattern 5: GIL-Held Long Computations

**What people do:** Call Rust functions from Python without releasing the GIL, blocking all Python threads during computation.

**Why it's wrong:** Jupyter notebook becomes unresponsive. No way to interrupt long computations from Python.

**Do this instead:** `py.allow_threads()` for anything that might take more than a few milliseconds. Structure Session access so the lock is held only during the Rust computation, not during Python pre/post processing.

---

## Scaling Considerations

These are not "users" in the web-app sense. Scaling concerns for a CAS are about expression complexity and coefficient size.

| Concern | Small (100 terms) | Medium (10K terms) | Large (1M+ terms) |
|---------|-------------------|--------------------|--------------------|
| **Expression arena memory** | Kilobytes, trivial | Megabytes, fine with u32 refs | Potential issue -- consider arena GC or session partitioning |
| **FPS multiplication** | Direct convolution O(n^2) | FFT-based O(n log n) needed | FFT essential, consider FLINT integration |
| **q-Pochhammer expansion** | Sequential multiplication | Parallel factor groups | Batch GMP operations via escape hatch |
| **E-graph saturation** | Milliseconds | Seconds, may need saturation limits | Likely impractical -- use phased rewriting |
| **Identity database search** | Linear scan fine | Index by expression hash/fingerprint | Bloom filter pre-check + hash index |

### Scaling Priorities

1. **First bottleneck: FPS arithmetic.** Multiplying two long series via convolution dominates most q-series computations. Start with naive O(n^2) convolution, benchmark, then add FFT path when N > ~256 terms. The rug/GMP integers already handle large-coefficient multiplication efficiently.

2. **Second bottleneck: Pattern matching in simplification.** As the rule set grows (500+ identities), naive linear scanning of all rules becomes expensive. Index rules by the root node type of their pattern (Add rules, Mul rules, QPochhammer rules, etc.) to narrow candidates.

3. **Third bottleneck: Arena memory for deep computations.** Bailey chain iteration and WZ certificates can generate large intermediate expression DAGs. Implement "session GC" that identifies unreachable expressions and compacts the arena. Not needed for v1 -- add when profiling shows memory pressure.

---

## Integration Points

### External Libraries

| Library | Integration Pattern | Notes |
|---------|---------------------|-------|
| `rug` (GMP/MPFR) | Cargo dependency in qsym-core | Provides `Integer`, `Rational`, `Float`. Build requires GMP C library. |
| `gmp-mpfr-sys` | Cargo dependency in qsym-gmp | Low-level GMP FFI for escape hatch. Used sparingly. |
| `egg` or `egglog` | Cargo dependency in qsym-core (optional feature) | E-graph equality saturation. Behind `feature = "egraph"` gate initially. |
| `smallvec` | Cargo dependency in qsym-core | Inline small vectors for hypergeometric params (avoids heap for common cases). |
| `inventory` | Cargo dependency in qsym-core | Static registration of domain modules. |
| `pyo3` | Cargo dependency in qsym-python | Rust-Python bindings. Use `abi3-py39` for broad compatibility. |
| `maturin` | Build tool (not a dependency) | Builds the Python wheel from Rust. |
| `serde` + `toml` | Cargo dependency in qsym-core | Identity database loading from TOML files. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| **qsym-core <-> qsym-python** | Pub API of qsym-core | qsym-python depends on qsym-core. Keep qsym-core's public API stable. |
| **qsym-core <-> qsym-gmp** | Trait-based (GmpAccelerator trait) | qsym-core defines the trait, qsym-gmp implements it. qsym-core works without qsym-gmp via feature flag. |
| **ExprArena <-> Domain Modules** | &mut ExprArena passed as parameter | Modules create expressions through the arena. They never store their own expression memory. |
| **SimplificationEngine <-> DomainModule** | DomainModule trait methods | Engine calls `try_simplify()` on each applicable module during its phase. |
| **Session <-> Everything** | Session owns Arena, Engine, DB | Single point of coordination. Python holds Arc<Mutex<Session>>. |

---

## Build Order and Critical Path

### Dependency Chain (What Must Be Built First)

```
Phase 0: Project scaffold
    Cargo workspace, pyproject.toml, CI, basic test infrastructure
    |
    v
Phase 1: Expression foundation (CRITICAL PATH)
    ExprArena + Expr enum + SymbolRegistry + canonical ordering
    |   These are the foundation -- nothing else works without them
    |
    +-> Arithmetic layer (rug integration, BigInt/BigRat in Expr)
    |
    +-> Pattern matcher (depends on Expr + Arena)
    |
    v
Phase 2: Simplification engine
    RewriteRule + SimplificationPhase + bottom-up traversal
    |   Depends on: Pattern matcher, ExprArena
    |
    +-> LaTeX / pretty-print (depends on Expr, can be parallel)
    |
    v
Phase 3: First domain module -- q-Pochhammer
    qpoch.rs: expansion, simplification rules, recurrence relations
    |   This is the most important module -- everything uses (a;q)_n
    |
    +-> Formal Power Series (depends on Arithmetic, ExprArena)
    |       FPS is needed for series expansion of q-Pochhammer
    |
    v
Phase 4: PyO3 bindings (can start as soon as Phase 1 is done)
    QExpr, QSession, basic Python DSL
    |   Iteratively expand as more Rust API stabilizes
    |
    v
Phase 5: More domain modules (can proceed in parallel)
    Hypergeometric -> Theta/Eta -> Partitions -> Mock Theta -> Bailey -> WZ
    |   Each adds rules to the engine + operations to the API
    |
    v
Phase 6: Advanced features
    E-graph integration, C GMP escape hatch, identity database v2
    |   These are optimizations and extensions, not foundations
```

### Critical Architectural Decisions That Must Be Made Early

| Decision | When | Why Early | Risk If Deferred |
|----------|------|-----------|-----------------|
| **ExprRef size and arena design** | Phase 1 | Every component depends on how expressions are referenced | Changing from u32 to u64 or from index to pointer later requires touching every file |
| **Expr enum variants** | Phase 1 | Adding variants later is fine, but the core set (atoms, arithmetic) must be right | Changing Add from Vec to BTreeSet, or adding Neg back, cascades everywhere |
| **Canonical ordering** | Phase 1 | Hash-consing correctness depends on consistent canonical forms | Bugs here cause "equal expressions not equal" which is catastrophic for a CAS |
| **Session/Context ownership model** | Phase 1 | Determines how Arena lifetime interacts with Python | Switching from Arc<Mutex<Session>> to lifetime-based borrows is a rewrite |
| **n-ary vs binary Add/Mul** | Phase 1 | Affects every pattern match and simplification rule | Converting between representations is painful |
| **FPS storage model** | Phase 3 | Whether FPS is inline in Expr or separate arena affects memory layout | Can be deferred until Phase 3, but the Expr enum must have the variant ready |

---

## Lessons from Existing CAS Architectures

### From Maple (the system being replaced)

- **Simplification table is essential.** Maple's core insight: every simplified expression lives in a global table, enabling pointer-based equality. Our ExprArena with hash-consing is exactly this pattern.
- **Small integer optimization matters.** Maple stores integers -10^9 to 10^9 directly in pointer bits. Consider storing small integers (that fit in i32) inline in ExprRef rather than heap-allocating BigInt for them.
- **DAG-aware algorithms are non-negotiable.** Maple's documentation explicitly warns that algorithms designed for trees must be adapted for DAGs. Our simplification traversal must handle shared subexpressions correctly (visit each unique node once, not once per reference).

### From SymPy (the most-used open-source CAS)

- **Immutability is the right default.** SymPy's `__new__` pattern for immutable objects has proven correct over 15+ years. Our arena-interned expressions are inherently immutable.
- **`func(*args)` reconstruction invariant.** SymPy guarantees `obj == obj.func(*obj.args)`. Our equivalent: `arena.intern(arena.get(r).clone()) == r`. This invariant should be tested in CI.
- **Sympify-style coercion is needed.** SymPy's `sympify()` converts Python objects to symbolic expressions. We need similar coercion in the Python DSL layer (integer literals -> QExpr, string parsing -> QExpr).

### From Symbolica (closest Rust CAS)

- **Linear compressed representation is fast but complex.** Symbolica's 8x compression over Mathematica is impressive but hard to replicate and extend. Start with the simpler arena approach; optimize representation later if profiling shows memory is the bottleneck.
- **Namespace-based symbol management prevents collisions.** Good idea for Q-Symbolic since researchers may use common symbol names (a, b, q) across different contexts.
- **Pattern matching on n-ary ops needs unordered matching.** Symbolica's approach of matching through symmetry (x+z matches x+y+z) is essential for q-series where expressions are long sums and products.

### From JuliaSymbolics/Hash-Consing Paper (2025)

- **Hash-consing benefits compound.** The 3.2x compute and 2x memory gains compound with expression complexity. For q-series with deeply nested q-Pochhammer products, the benefits should be even larger.
- **Weak references prevent memory leaks in long sessions.** JuliaSymbolics uses WeakValueDict. In Rust, we don't need this since our arena owns everything. But we do need a "GC" strategy for long-running sessions -- periodically rebuild the arena keeping only reachable expressions.
- **Hash-consing + e-graphs is the future.** The paper's "future work" section proposes combining hash-consing (structural sharing) with e-graphs (logical equivalence). This is exactly our architecture: arena for structure, optional e-graph for equivalence.

### From Garvan's qseries Package (the functionality being replaced)

- **Core operations:** `prodmake` (series to product), `etamake` (to eta-quotient), `jacprodmake` (to Jacobi product), `jac2series` (product to series). These conversion operations between representations are the bread and butter of q-series computation.
- **The q-product ↔ q-series round-trip is fundamental.** The ability to go from an infinite product to its series expansion and back is the single most important capability. Architect the FPS module and the q-Pochhammer module to make this round-trip clean.
- **Andrews' algorithm is central.** Garvan's `prodmake` implements Andrews' algorithm for converting q-series to infinite products. This should be a first-class operation in the hypergeometric module.

---

## Sources

### Expression Representation
- [Efficient Symbolic Computation via Hash Consing](https://arxiv.org/html/2509.20534) - Zhu (2025), JuliaSymbolics hash-consing implementation -- HIGH confidence
- [SymPy Core System Architecture](https://deepwiki.com/sympy/sympy/2-core-system) - DeepWiki analysis of SymPy internals -- HIGH confidence
- [Maple Programming Guide Appendix](https://www.maplesoft.com/support/help/Maple/view.aspx?path=ProgrammingGuide/Appendix1) - Official Maple DAG + simplification table documentation -- HIGH confidence
- [Symbolica Expressions](https://symbolica.io/docs/expressions.html) - Official Symbolica documentation -- HIGH confidence

### Rewrite Engines
- [egg: Fast and Extensible Equality Saturation](https://egraphs-good.github.io/egg/egg/) - Official egg documentation -- HIGH confidence
- [egglog: Better Together](https://arxiv.org/abs/2304.04332) - Willsey et al., Datalog + equality saturation -- HIGH confidence
- [Symbolica Pattern Matching](https://symbolica.io/docs/pattern_matching.html) - Official Symbolica docs -- HIGH confidence
- [Algebraic simplification: bottom-up algorithms](https://www.sciencedirect.com/science/article/pii/030439759090078V) - Fernandez-Camacho & Steyaert -- MEDIUM confidence (1990 paper, foundational)

### Arithmetic
- [rug crate documentation](https://docs.rs/rug/latest/rug/index.html) - Official rug docs -- HIGH confidence
- [gmp-mpfr-sys](https://docs.rs/gmp-mpfr-sys) - Official GMP Rust bindings -- HIGH confidence

### Hash-Consing in Rust
- [hashconsing crate](https://docs.rs/hashconsing) - Filiatre & Conchon inspired Rust implementation -- HIGH confidence
- [hash_cons crate](https://lib.rs/crates/hash_cons) - Alternative Rust hash-consing library -- MEDIUM confidence
- [rustc memory management](https://rustc-dev-guide.rust-lang.org/memory.html) - Arena + interning patterns in the Rust compiler -- HIGH confidence

### FFI / Python Bindings
- [PyO3 User Guide](https://pyo3.rs/) - Official PyO3 documentation -- HIGH confidence
- [Maturin Project Layout](https://www.maturin.rs/project_layout.html) - Official maturin docs -- HIGH confidence
- [PyO3 Architecture](https://github.com/PyO3/pyo3/blob/main/Architecture.md) - Official PyO3 internals -- HIGH confidence

### Formal Power Series
- [series Rust crate](https://docs.rs/series/latest/series/) - Truncated Laurent series in Rust -- MEDIUM confidence
- [SageMath Lazy Series](https://doc.sagemath.org/html/en/reference/power_series/sage/rings/lazy_series.html) - SageMath lazy FPS documentation -- HIGH confidence
- [Lazy Multiplication of Formal Power Series](https://www.researchgate.net/publication/2538881_Lazy_Multiplication_of_Formal_Power_Series) - van der Hoeven -- MEDIUM confidence

### Domain-Specific (q-series)
- [Garvan q-product tutorial](https://qseries.org/fgarvan/papers/qmaple.pdf) - Garvan's qseries Maple package paper -- HIGH confidence
- [qseries Package v1.3](https://qseries.org/fgarvan/qmaple/qseries/) - Official qseries package page -- HIGH confidence
- [QPochhammer Wolfram Documentation](https://reference.wolfram.com/language/ref/QPochhammer.html) - Mathematica's QPochhammer implementation -- HIGH confidence

---

*Architecture research for: Q-Symbolic computation engine*
*Researched: 2026-02-13*
