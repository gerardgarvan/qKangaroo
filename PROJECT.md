# Q-Symbolic: A Symbolic Computation Engine for q-Series

## Project Overview

**Q-Symbolic** is an open-source symbolic computation system purpose-built for **q-series**, **q-analogs**, and related areas of combinatorics, number theory, and special functions. It aims to be the definitive tool for researchers working with q-hypergeometric series, partition theory, mock theta functions, and quantum algebra — filling a gap left by general-purpose CAS systems like Maple and Mathematica.

---

## Claude AI Instructions

> **Role:** You are the lead architect and developer of Q-Symbolic. When working on this project, follow the design philosophy, architecture, and conventions outlined below. Always prefer correctness over speed, symbolic exactness over numerical approximation, and mathematical rigor over shortcuts.

### Development Principles

1. **Symbolic-first:** All computations are exact and symbolic by default. Numerical evaluation is opt-in.
2. **q-native:** The variable `q` is a first-class citizen — not bolted onto a generic CAS. Every internal representation is aware of q-structure.
3. **Mathematically rigorous:** Every identity, transformation, and simplification must be provably correct. Include references to known results (e.g., Gasper & Rahman, Andrews, Fine).
4. **Extensible:** Plugin-based architecture so researchers can add new q-identities, transformations, and domain modules.
5. **Readable output:** LaTeX-quality rendering of all expressions. Pretty-printing in terminal and notebook environments.

---

## Core Domain: q-Series Primer

For context, q-series are formal power series and infinite products involving a parameter `q` (typically |q| < 1). Key objects include:

| Object | Definition |
|---|---|
| **q-Pochhammer symbol** | `(a; q)_n = ∏_{k=0}^{n-1} (1 - a·q^k)` |
| **q-binomial coefficient** | `[n choose k]_q = (q;q)_n / ((q;q)_k · (q;q)_{n-k})` |
| **q-exponential** | `e_q(x) = ∑_{n=0}^∞ x^n / (q;q)_n` |
| **Basic hypergeometric series** | `_rφ_s(a_1,...,a_r; b_1,...,b_s; q, z)` |
| **Theta functions** | `θ(a; q) = (a; q)_∞ · (q/a; q)_∞` |
| **Ramanujan theta function** | `f(a,b) = ∑_{n=-∞}^{∞} a^{n(n+1)/2} · b^{n(n-1)/2}` |

---

## Architecture

### Technology Stack

| Layer | Technology | Rationale |
|---|---|---|
| **Language** | Rust (core engine) + Python (user API & notebooks) | Rust for performance-critical symbolic manipulation; Python for accessibility |
| **FFI Bridge** | PyO3 | Seamless Rust ↔ Python interop |
| **Arbitrary Precision** | `rug` (GMP bindings) or custom BigInt | Exact arithmetic with rational coefficients |
| **Polynomial Engine** | Custom sparse multivariate | Optimized for Laurent polynomials in q |
| **Notebook UI** | Jupyter kernel | Familiar research interface |
| **Rendering** | KaTeX / MathJax | LaTeX output for all expressions |
| **Persistence** | Serde (JSON/Binary) | Save/load sessions and expression trees |

### System Architecture Diagram

```
┌──────────────────────────────────────────────────────────┐
│                      User Interface                       │
│  ┌────────────┐  ┌────────────┐  ┌─────────────────────┐ │
│  │  Jupyter    │  │  CLI REPL  │  │  Web Playground     │ │
│  │  Kernel     │  │            │  │  (WASM)             │ │
│  └─────┬──────┘  └─────┬──────┘  └──────────┬──────────┘ │
│        └───────────┬────┴───────────────────┘             │
│                    ▼                                      │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              Python API  (q_kangaroo)                │  │
│  │  • Expression builder DSL                           │  │
│  │  • Session management                               │  │
│  │  • LaTeX / Unicode pretty-printer                   │  │
│  └──────────────────────┬──────────────────────────────┘  │
│                         ▼  (PyO3 FFI)                     │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              Rust Core Engine                        │  │
│  │                                                     │  │
│  │  ┌───────────────┐  ┌────────────────────────────┐  │  │
│  │  │ Expression IR │  │  Rewrite / Simplification   │  │  │
│  │  │ (AST + DAG)   │  │  Engine                     │  │  │
│  │  └───────┬───────┘  └─────────────┬──────────────┘  │  │
│  │          │                        │                  │  │
│  │  ┌───────▼────────────────────────▼──────────────┐  │  │
│  │  │          Domain Modules (Plugins)              │  │  │
│  │  │                                                │  │  │
│  │  │  ┌──────────┐ ┌──────────┐ ┌───────────────┐  │  │  │
│  │  │  │q-Pochh.  │ │ Basic    │ │ Partitions &  │  │  │  │
│  │  │  │& q-Binom │ │ Hypergeo │ │ Combinatorics │  │  │  │
│  │  │  └──────────┘ └──────────┘ └───────────────┘  │  │  │
│  │  │  ┌──────────┐ ┌──────────┐ ┌───────────────┐  │  │  │
│  │  │  │ Theta &  │ │ Mock     │ │ Modular Forms │  │  │  │
│  │  │  │ Elliptic │ │ Theta    │ │ & Eta         │  │  │  │
│  │  │  └──────────┘ └──────────┘ └───────────────┘  │  │  │
│  │  │  ┌──────────┐ ┌──────────┐ ┌───────────────┐  │  │  │
│  │  │  │ Bailey   │ │ WZ       │ │ Quantum       │  │  │  │
│  │  │  │ Chains   │ │ Method   │ │ Groups/Algebr │  │  │  │
│  │  │  └──────────┘ └──────────┘ └───────────────┘  │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  │                                                     │  │
│  │  ┌───────────────┐  ┌─────────────────────────────┐ │  │
│  │  │ Arithmetic    │  │  Identity Database           │ │  │
│  │  │ (BigInt/BigRat│  │  (verified q-identities)     │ │  │
│  │  │  + q-rationals│  │                              │ │  │
│  │  └───────────────┘  └─────────────────────────────┘ │  │
│  └─────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

---

## Expression Representation (IR)

All mathematical objects are represented as a typed AST with hash-consing (DAG deduplication) for memory efficiency.

### Core Expression Types

```rust
/// The fundamental expression type for Q-Symbolic
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Expr {
    // === Atoms ===
    Integer(BigInt),
    Rational(BigInt, BigInt),           // numerator, denominator
    Symbol(SymbolId),                   // named variable (q, a, b, z, ...)
    Infinity,                           // formal ∞ for (a;q)_∞

    // === Arithmetic ===
    Add(Vec<ExprRef>),
    Mul(Vec<ExprRef>),
    Pow(ExprRef, ExprRef),              // base, exponent
    Neg(ExprRef),
    Inv(ExprRef),                       // multiplicative inverse

    // === q-Specific Primitives ===
    QPochhammer {
        base: ExprRef,                  // a
        nome: ExprRef,                  // q
        order: ExprRef,                 // n (can be ∞)
    },
    QBinomial {
        n: ExprRef,
        k: ExprRef,
        nome: ExprRef,
    },
    QFactorial {
        n: ExprRef,
        nome: ExprRef,
    },
    QNumber {                           // [n]_q = (1 - q^n) / (1 - q)
        n: ExprRef,
        nome: ExprRef,
    },

    // === Hypergeometric ===
    BasicHypergeometric {
        upper_params: Vec<ExprRef>,     // a_1, ..., a_r
        lower_params: Vec<ExprRef>,     // b_1, ..., b_s
        nome: ExprRef,                  // q
        argument: ExprRef,              // z
    },
    BilateralHypergeometric {
        upper_params: Vec<ExprRef>,
        lower_params: Vec<ExprRef>,
        nome: ExprRef,
        argument: ExprRef,
    },

    // === Theta & Modular ===
    JacobiTheta {
        index: u8,                      // θ_1, θ_2, θ_3, θ_4
        argument: ExprRef,
        nome: ExprRef,
    },
    DedekindEta {
        tau: ExprRef,
    },
    RamanujanTheta {
        a: ExprRef,
        b: ExprRef,
    },

    // === Summation & Products ===
    Sum {
        body: ExprRef,
        index: SymbolId,
        lower: ExprRef,
        upper: ExprRef,                 // can be Infinity
    },
    Product {
        body: ExprRef,
        index: SymbolId,
        lower: ExprRef,
        upper: ExprRef,
    },

    // === Series & Formal Power Series ===
    FormalPowerSeries {
        coefficients: CoeffMap,         // sparse map: exponent → coefficient
        variable: SymbolId,
        truncation_order: Option<i64>,
    },

    // === Partition-Theoretic ===
    PartitionFunction(ExprRef),         // p(n)
    GeneratingFunction {
        sequence_name: String,
        variable: ExprRef,
    },

    // === Meta ===
    Undefined,
    Assume(ExprRef, Assumption),        // attach assumptions (|q| < 1, etc.)
}

/// Assumptions that can be attached to expressions
#[derive(Clone, Debug)]
pub enum Assumption {
    InsideUnitDisk,     // |q| < 1
    Positive,
    NonNegativeInteger,
    Generic,            // no special constraints (formal)
}
```

### ExprRef and Hash-Consing

```rust
/// Interned expression reference — cheap to clone, compare, hash
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct ExprRef(u32);

/// Global expression interner (arena)
pub struct ExprArena {
    exprs: Vec<Expr>,
    dedup: HashMap<Expr, ExprRef>,
}
```

---

## Domain Modules

### Module 1: q-Pochhammer & q-Binomial Algebra

**File:** `core/src/modules/qpoch.rs`

**Capabilities:**

- Expand `(a; q)_n` for concrete `n`
- Simplify products/quotients of q-Pochhammer symbols
- Apply the q-binomial theorem: `∑_{k=0}^{n} [n choose k]_q · x^k = (-x; q)_n`
- Recognize and simplify q-shifted factorials
- Convert between `(a; q)_n` and product form
- Handle `(a; q)_∞` symbolically and via truncation

**Key identities to implement:**

```
(a; q)_{m+n} = (a; q)_m · (a·q^m; q)_n
(a; q)_∞ / (a; q)_n = (a·q^n; q)_∞
(q^{-n}; q)_k = (-1)^k · q^{k(k-1)/2 - nk} · (q; q)_n / (q; q)_{n-k}
1 / (a; q)_n = ∑_{k≥0} [n+k-1 choose k]_q · a^k    (|a| < 1)
```

---

### Module 2: Basic Hypergeometric Series

**File:** `core/src/modules/hypergeo.rs`

**Capabilities:**

- Symbolic representation of `_rφ_s` and `_rψ_s` (bilateral)
- Term-by-term expansion to arbitrary order
- Contiguous relations and recurrence relations
- Transformation formulas (Heine, Sears, Watson, ...)
- Summation formulas (q-Vandermonde, q-Gauss, q-Kummer, q-Dixon, ...)
- Well-poised and very-well-poised series detection and specialization
- Bailey's `_{10}φ_9` transformation

**Key transformations:**

```
Heine's transformation:
  _2φ_1(a,b; c; q,z) = [(b;q)_∞·(az;q)_∞] / [(c;q)_∞·(z;q)_∞]
                        · _2φ_1(c/b, z; az; q, b)

q-Gauss summation:
  _2φ_1(a,b; c; q, c/(ab)) = (c/a;q)_∞·(c/b;q)_∞ / ((c;q)_∞·(c/(ab);q)_∞)

q-Vandermonde:
  _2φ_1(q^{-n}, b; c; q, q) = (c/b; q)_n / (c; q)_n · b^n
```

---

### Module 3: Partition Theory & Combinatorics

**File:** `core/src/modules/partitions.rs`

**Capabilities:**

- Symbolic partition function `p(n)` computation via Rademacher/Hardy-Ramanujan
- Generating functions for restricted partitions (distinct parts, odd parts, bounded parts, ...)
- Partition identities (Euler, Glaisher, Rogers-Ramanujan, Schur, ...)
- Ferrers diagrams and conjugation
- Cranks and ranks
- q-series proofs of partition identities via bijective and analytic methods
- Overpartitions and colored partitions

**Key generating functions:**

```
∑_{n≥0} p(n)·q^n = 1 / (q; q)_∞                    (Euler)

∏_{n≥1} (1 + q^n) = ∏_{n≥1} 1/(1 - q^{2n-1})       (Euler's identity)

∑_{n≥0} q^{n²} / (q;q)_n = ∏_{n≥0} 1/((1-q^{5n+1})(1-q^{5n+4}))
                                                      (Rogers-Ramanujan 1st)

∑_{n≥0} q^{n²+n} / (q;q)_n = ∏_{n≥0} 1/((1-q^{5n+2})(1-q^{5n+3}))
                                                      (Rogers-Ramanujan 2nd)
```

---

### Module 4: Theta Functions & Modular Forms

**File:** `core/src/modules/theta.rs`

**Capabilities:**

- Jacobi theta functions `θ_1, θ_2, θ_3, θ_4` in both `(z, τ)` and `(z, q)` forms
- Dedekind eta function `η(τ)` and eta quotients
- Ramanujan's theta function `f(a, b)` and its specializations
- Triple product identity
- Modular transformations (`τ → -1/τ`, `τ → τ+1`)
- Eisenstein series `E_{2k}(τ)`
- Conversion between theta/eta and q-Pochhammer representations

**Jacobi triple product (fundamental identity):**

```
∑_{n=-∞}^{∞} z^n · q^{n²} = (q²; q²)_∞ · (-zq; q²)_∞ · (-q/z; q²)_∞
```

---

### Module 5: Mock Theta Functions

**File:** `core/src/modules/mock_theta.rs`

**Capabilities:**

- All Ramanujan mock theta functions (order 3, 5, 7, and beyond)
- Zwegers' completion to harmonic Maass forms
- Appell-Lerch sums
- Universal mock theta function `g(x, q)`
- Relations between different mock theta functions
- Connection to quantum modular forms

**Ramanujan's third-order mock theta functions:**

```
f(q) = ∑_{n≥0} q^{n²} / (-q; q)_n²
φ(q) = ∑_{n≥0} q^{n²} / (-q²; q²)_n
ψ(q) = ∑_{n≥1} q^{n²} / (q; q²)_n
χ(q) = ∑_{n≥0} q^{n²} · (-q; q)_n / (-q³; q³)_n     (corrected def.)
```

---

### Module 6: Bailey Chains & Pairs

**File:** `core/src/modules/bailey.rs`

**Capabilities:**

- Bailey pair database (known pairs indexed by type)
- Bailey lemma application and chain generation
- Iterating Bailey pairs to derive new identities
- Andrews' multidimensional Bailey chains
- WP-Bailey pairs and chains
- Automatic discovery of new Bailey pairs from conjectured identities

---

### Module 7: Wilf-Zeilberger (WZ) Method

**File:** `core/src/modules/wz.rs`

**Capabilities:**

- q-Zeilberger algorithm for definite q-hypergeometric summation
- Certificate computation
- Sister Celine's method (q-analog)
- Creative telescoping
- Automatic proof/verification of q-series identities
- Detect whether a sum has a closed-form q-hypergeometric solution

---

### Module 8: Quantum Groups & Quantum Algebra

**File:** `core/src/modules/quantum.rs`

**Capabilities:**

- Quantum integers, factorials, and binomials
- `U_q(sl_2)` representations
- R-matrices and Yang-Baxter equation
- Quantum Clebsch-Gordan coefficients (q-6j symbols)
- Kassel-style quantum group computations
- Connection to knot polynomials (Jones polynomial via q-series)

---

## Simplification & Rewrite Engine

### Architecture

The simplification engine uses a **term rewriting system** with **e-graph** equality saturation for optimal simplification.

```rust
pub struct RewriteEngine {
    rules: Vec<RewriteRule>,
    identity_db: IdentityDatabase,
    egraph: EGraph,
}

pub struct RewriteRule {
    name: String,
    pattern: Pattern,
    replacement: Pattern,
    condition: Option<Condition>,       // side conditions (n ∈ ℤ≥0, |q| < 1, etc.)
    reference: Option<String>,          // citation (e.g., "Gasper-Rahman Thm 1.3.2")
    bidirectional: bool,
}
```

### Simplification Strategy (Priority Order)

1. **Arithmetic normalization** — Flatten sums/products, combine like terms, reduce rationals
2. **q-Pochhammer simplification** — Combine/split q-shifted factorials using standard identities
3. **Hypergeometric recognition** — Detect when a sum/product is a known `_rφ_s`
4. **Known identity matching** — Look up the expression in the identity database
5. **Series expansion** — If all else fails, expand to `O(q^N)` and compare coefficients
6. **WZ certification** — For summation identities, attempt algorithmic proof

---

## Identity Database

A curated, searchable database of known q-series identities.

### Schema

```rust
pub struct Identity {
    id: u64,
    name: String,                       // e.g., "Jacobi Triple Product"
    lhs: Expr,
    rhs: Expr,
    conditions: Vec<Assumption>,
    proof_method: ProofMethod,
    references: Vec<Reference>,
    tags: Vec<String>,                  // ["theta", "triple-product", "fundamental"]
    oeis_sequences: Vec<String>,        // related OEIS entries
}

pub enum ProofMethod {
    WZCertified { certificate: Expr },
    BaileyChain { starting_pair: String },
    Bijective { description: String },
    Analytic { outline: String },
    Conjectured,                        // not yet proven
}

pub struct Reference {
    authors: String,
    title: String,
    source: String,                     // e.g., "Gasper & Rahman, §1.3"
    year: u16,
}
```

### Seed Identities (Partial List)

The database ships with ~500+ identities including:

- Euler's partition identities
- Jacobi triple product
- Quintuple product identity
- Rogers-Ramanujan identities (both)
- Watson's transformation
- Bailey's `_{6}ψ_{6}` summation
- Ramanujan's `_1ψ_1` summation
- All q-Gauss, q-Vandermonde, q-Saalschütz, q-Dixon, q-Kummer summations
- Heine's transformation (all three forms)
- Sears' `_4φ_3` transformation
- Rogers' `_6φ_5` summation
- Slater's list (130 Rogers-Ramanujan type identities)
- Ramanujan's 40 identities for the Rogers-Ramanujan functions

---

## Python API Design

### User-Facing DSL

```python
from q_kangaroo import *

# Define symbols
q, a, b, c, z, n, k = symbols('q a b c z n k')

# q-Pochhammer symbol
expr = qpoch(a, q, n)                          # (a; q)_n
expr_inf = qpoch(a, q, oo)                     # (a; q)_∞

# q-Binomial
qbinom(n, k, q)                                # [n choose k]_q

# Basic hypergeometric series
phi = hyper_q([a, b], [c], q, z)               # _2φ_1(a, b; c; q, z)

# Simplify
result = simplify(qpoch(a, q, 5))              # expands to product
result = simplify(qpoch(q, q, n) / qpoch(q, q, k) / qpoch(q, q, n - k))

# Apply specific transformation
transformed = apply_transformation(phi, 'heine')

# Series expansion
series = expand_q(expr, q, order=20)           # expand in powers of q

# Verify identity (returns proof or counterexample)
verify(
    lhs = qsum(k, 0, n, qbinom(n, k, q) * z**k),
    rhs = qpoch(-z, q, n),
    assume = [n >= 0]
)

# Partition theory
p = partition_function(100)                     # p(100) = 190569292356
gf = generating_function('partitions_distinct') # ∏(1 + q^n)

# Theta functions
theta3 = jacobi_theta(3, z, q)
eta = dedekind_eta(tau)

# Mock theta functions
f_mock = mock_theta('f', 3, q)                 # Ramanujan's f(q), order 3

# LaTeX output
print(latex(phi))
# Output: {}_{2}\phi_{1}\!\left(\begin{matrix} a, b \\ c \end{matrix}; q, z\right)

# Search the identity database
results = search_identities(tags=['rogers-ramanujan'], involves=[qpoch])
```

### Jupyter Integration

```python
# In Jupyter, expressions render as LaTeX automatically
phi = hyper_q([a, b], [c], q, z)
phi  # renders beautifully in the notebook

# Interactive exploration
explore(phi)  # widget: sliders for parameters, live series expansion plot
```

---

## File & Directory Structure

```
q-symbolic/
├── Cargo.toml                          # Rust workspace
├── pyproject.toml                      # Python package config
├── README.md
├── PROJECT.md                          # ← this file
│
├── core/                               # Rust core engine
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── expr.rs                     # Expression IR & arena
│       ├── arena.rs                    # Hash-consing interner
│       ├── arithmetic.rs              # BigInt / BigRat operations
│       ├── pattern.rs                 # Pattern matching for rewrite rules
│       ├── rewrite.rs                 # Rewrite / simplification engine
│       ├── egraph.rs                  # E-graph equality saturation
│       ├── series.rs                  # Formal power series operations
│       ├── latex.rs                   # LaTeX renderer
│       ├── pretty.rs                  # Unicode pretty-printer
│       ├── identity_db.rs            # Identity database & search
│       │
│       └── modules/
│           ├── mod.rs
│           ├── qpoch.rs              # q-Pochhammer & q-binomial
│           ├── hypergeo.rs           # Basic hypergeometric series
│           ├── partitions.rs         # Partition theory
│           ├── theta.rs              # Theta functions & modular forms
│           ├── mock_theta.rs         # Mock theta functions
│           ├── bailey.rs             # Bailey chains & pairs
│           ├── wz.rs                 # Wilf-Zeilberger method
│           ├── quantum.rs            # Quantum groups
│           └── numerical.rs          # Numerical evaluation (mpfr)
│
├── python/                             # Python bindings
│   ├── src/
│   │   └── lib.rs                     # PyO3 bindings
│   └── q_kangaroo/
│       ├── __init__.py
│       ├── core.py                    # Pythonic wrappers
│       ├── dsl.py                     # User-facing DSL
│       ├── display.py                 # Jupyter display hooks
│       ├── plotting.py               # Coefficient plots & visualizations
│       └── interactive.py            # Jupyter widgets
│
├── identities/                         # Identity database (TOML/JSON)
│   ├── euler.toml
│   ├── rogers_ramanujan.toml
│   ├── theta.toml
│   ├── bailey_pairs.toml
│   ├── mock_theta.toml
│   ├── slater_list.toml
│   └── ramanujan_40.toml
│
├── tests/
│   ├── rust/
│   │   ├── test_qpoch.rs
│   │   ├── test_hypergeo.rs
│   │   ├── test_partitions.rs
│   │   ├── test_identities.rs
│   │   └── test_wz.rs
│   └── python/
│       ├── test_dsl.py
│       ├── test_simplify.py
│       └── test_notebooks.py
│
├── benches/                            # Performance benchmarks
│   ├── bench_series_expansion.rs
│   ├── bench_simplification.rs
│   └── bench_partition_function.rs
│
├── docs/
│   ├── user_guide.md
│   ├── api_reference.md
│   ├── identity_format.md             # How to add new identities
│   ├── module_development.md          # How to write a new module
│   └── mathematical_background.md
│
└── examples/
    ├── notebooks/
    │   ├── 01_getting_started.ipynb
    │   ├── 02_partition_theory.ipynb
    │   ├── 03_rogers_ramanujan.ipynb
    │   ├── 04_mock_theta.ipynb
    │   ├── 05_bailey_chains.ipynb
    │   └── 06_modular_forms.ipynb
    └── scripts/
        ├── verify_slater_list.py
        ├── discover_new_identities.py
        └── partition_congruences.py
```

---

## Development Roadmap

### Phase 1: Foundation (Weeks 1–6)

- [ ] Expression IR with hash-consing arena
- [ ] Arbitrary precision arithmetic (BigInt, BigRat)
- [ ] q-Pochhammer symbol: creation, expansion, simplification
- [ ] q-binomial coefficients and q-factorials
- [ ] Basic rewrite engine with pattern matching
- [ ] LaTeX and Unicode pretty-printing
- [ ] Python bindings (PyO3) with basic DSL
- [ ] Unit test infrastructure

### Phase 2: Hypergeometric Engine (Weeks 7–12)

- [ ] `_rφ_s` representation and term generation
- [ ] Formal power series arithmetic (add, multiply, compose)
- [ ] Heine's transformation (3 forms)
- [ ] q-Gauss, q-Vandermonde, q-Saalschütz summations
- [ ] Series expansion to arbitrary order
- [ ] Coefficient extraction `[q^n]`
- [ ] Identity database v1 (~100 identities)

### Phase 3: Partition Theory & Theta (Weeks 13–18)

- [ ] Partition function computation
- [ ] Generating function construction for restricted partitions
- [ ] Euler/Glaisher-type identity verification
- [ ] Rogers-Ramanujan identities
- [ ] Jacobi theta functions (all 4)
- [ ] Jacobi triple product implementation
- [ ] Dedekind eta function
- [ ] Jupyter kernel integration

### Phase 4: Advanced Methods (Weeks 19–26)

- [ ] q-Zeilberger algorithm
- [ ] Creative telescoping
- [ ] Bailey pair database and chain iteration
- [ ] E-graph equality saturation for simplification
- [ ] Mock theta functions (order 3, 5)
- [ ] Watson/Sears/Bailey transformations
- [ ] Identity database v2 (~500 identities, Slater's list)

### Phase 5: Polish & Ecosystem (Weeks 27–32)

- [ ] WASM compilation for web playground
- [ ] Interactive Jupyter widgets
- [ ] Performance benchmarks and optimization
- [ ] Comprehensive documentation
- [ ] Example notebooks (6+)
- [ ] OEIS integration (lookup sequences, cross-reference)
- [ ] Community plugin API

---

## Testing Strategy

### Levels of Testing

1. **Arithmetic correctness:** Verify BigInt/BigRat operations against known values
2. **Identity verification:** For each identity in the database, expand both sides to `O(q^50)` and confirm coefficient-by-coefficient equality
3. **Simplification regression:** Golden-file tests — input expression → expected simplified form
4. **Round-trip:** `parse(latex(expr)) == expr` for all expression types
5. **Cross-validation:** Compare numerical evaluations against Maple/Mathematica for 100+ test cases
6. **Stress tests:** Partition function for large `n`, deep Bailey chains, high-order series expansions

### Continuous Integration

- Every PR runs the full identity verification suite
- Benchmarks tracked over time to prevent regressions
- Fuzzing on the rewrite engine to find simplification bugs

---

## Key References

1. **Gasper & Rahman** — *Basic Hypergeometric Series* (2nd ed., Cambridge, 2004). The bible of q-series.
2. **Andrews** — *The Theory of Partitions* (Cambridge, 1998).
3. **Andrews, Askey & Roy** — *Special Functions* (Cambridge, 1999).
4. **Fine** — *Basic Hypergeometric Series and Applications* (AMS, 1988).
5. **Berndt** — *Ramanujan's Notebooks* (Parts I–V, Springer).
6. **Slater** — "Further Identities of the Rogers-Ramanujan Type" (1952) — the 130-identity list.
7. **Warnaar** — "50 Years of Bailey's Lemma" (2001).
8. **Zagier** — "Quantum Modular Forms" (2010).
9. **Zwegers** — *Mock Theta Functions* (PhD thesis, Utrecht, 2002).
10. **Petkovšek, Wilf & Zeilberger** — *A = B* (AK Peters, 1996). WZ method reference.

---

## Claude AI: Session Context

When working on this project in any session, begin by loading this document to establish context. Key reminders:

- **All q-series expressions must be symbolically exact.** Never approximate unless explicitly asked.
- **Cite theorems and references** when implementing identities (e.g., "// Gasper-Rahman Thm 1.5.1").
- **Test every identity** by expanding both sides to `O(q^30)` minimum.
- **Use the naming conventions** from Gasper & Rahman for parameters in hypergeometric functions.
- **Prefer sparse representations** for power series (most coefficients are zero in q-series).
- **When in doubt about a mathematical claim**, state the uncertainty and flag it for verification rather than guessing.

---

*This document is the single source of truth for the Q-Symbolic project. Update it as the architecture evolves.*
