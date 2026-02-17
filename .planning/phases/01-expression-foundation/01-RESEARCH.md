# Phase 1: Expression Foundation - Research

**Researched:** 2026-02-13
**Domain:** Hash-consed expression arena, arbitrary precision arithmetic (GMP), LaTeX/Unicode rendering
**Confidence:** HIGH (core technologies well-documented; patterns verified from multiple sources)

## Summary

Phase 1 builds the absolute foundation of Q-Symbolic: the expression arena with hash-consing, the Expr enum with q-series-specific node types, arbitrary precision arithmetic via GMP, and two rendering backends (LaTeX and Unicode). Every subsequent phase depends on getting these right.

The key architectural insight is that this phase builds a **data model**, not a computation engine. No simplification, no series expansion, no pattern matching -- just the ability to construct, store, deduplicate, and display expressions. Keeping this scope tight is critical: Pitfall P9 (over-engineering the IR) and Pitfall P7 (scope creep to general CAS) are the biggest risks.

**Primary recommendation:** Build the ExprArena + Expr enum first, get hash-consing working with tests, then layer on rug arithmetic, then rendering. Three natural plan boundaries.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rug` | 1.28.1 | BigInt and BigRational via GMP/MPFR | Fastest arbitrary precision in Rust. Wraps GMP 6.3.0. Required for exact q-series coefficient arithmetic. rug::Rational implements Hash, Eq, Ord, Clone, Display, Debug, serde traits. Requires Rust 1.85+. **Confidence: HIGH** |
| `gmp-mpfr-sys` | ~1.6.8 | Low-level GMP FFI (transitive dep of rug) | Ships GMP 6.3.0 source, builds from source. Cygwin is an explicitly supported build target. **Confidence: HIGH** |
| `smallvec` | 1.x (stable) | Inline small vectors for Expr children | Avoids heap allocation for common cases (e.g., hypergeometric params with r,s <= 4). Well-established crate (Servo project). 2.0 is alpha -- use stable 1.x. **Confidence: HIGH** |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `fxhash` or `ahash` | latest | Fast non-cryptographic hash for dedup table | The arena's HashMap<Expr, ExprRef> is the hottest data structure. Default SipHash is unnecessarily secure for this use case. FxHash or AHash provide 2-5x faster hashing. Use `HashMap` from `rustc_hash` (fxhash) or `ahash`. **Confidence: HIGH** |
| `serde` | 1.0.x | Serialization of Expr, ExprArena | Derive Serialize/Deserialize on Expr enum for testing, debugging, and future save/load. Phase 1 uses it for snapshot testing. **Confidence: HIGH** |
| `fmtastic` | 0.2.2 | Unicode subscript/superscript formatting | Provides `Subscript(n)` and `Superscript(n)` for terminal display of expressions like (a;q)_n. Small, no_std-friendly. **Confidence: MEDIUM** (low download count but does exactly what we need; could also hand-roll with a unicode lookup table) |
| `proptest` | 1.x | Property-based testing | Generate random Expr trees, verify hash-consing invariants (identical structure -> identical ExprRef). Critical for correctness. **Confidence: HIGH** |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `rug` (GMP) | `num-bigint` 0.4.6 | Pure Rust, no C deps, but 5-10x slower for large numbers. Use only as fallback behind feature flag. |
| `rug` (GMP) | `malachite` 0.9.1 | Pure Rust, competitive with GMP for some ops, but LGPL license and 1.5-3x slower overall. |
| Custom arena | `hashconsing` 0.5.x crate | Provides HConsed<T> with Arc-based interning and uid equality. But uses Arc (unnecessary overhead when arena owns everything), global consign tables (conflicts with arena-scoped design), and weak refs (complex). Custom ~50 LOC arena is simpler and faster. Study hashconsing for design validation only. |
| Custom arena | `typed-arena` + separate HashMap | typed-arena provides allocation but not interning. You'd still need the HashMap dedup layer. Rolling our own Vec+HashMap is cleaner and gives full control. |
| `fxhash` | `ahash` | Both are fast non-crypto hashes. ahash uses AES-NI when available (fastest on modern x86). fxhash is simpler, used by rustc. Either works. |
| `fmtastic` | Hand-rolled unicode table | fmtastic is tiny; hand-rolling is ~20 lines for the subscript/superscript chars we need. Either approach works. |

**Installation (Cargo.toml for Phase 1):**

```toml
[package]
name = "qsym-core"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
# Arbitrary precision arithmetic (GMP backend)
rug = { version = "1.28", features = ["rational", "serde"] }

# Inline small vectors for Expr children
smallvec = { version = "1", features = ["const_generics", "serde"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }

# Fast hashing for the dedup table
rustc-hash = "2"  # provides FxHashMap

[dev-dependencies]
proptest = "1"
serde_json = "1"
```

---

## Architecture Patterns

### Recommended Project Structure (Phase 1 scope)

```
crates/
  qsym-core/
    Cargo.toml
    src/
      lib.rs           # pub mod declarations, re-exports
      expr.rs          # Expr enum, ExprRef newtype
      arena.rs         # ExprArena (Vec + HashMap hash-consing)
      symbol.rs        # SymbolId, SymbolRegistry (append-only name table)
      number.rs        # Wrapper types around rug::Integer, rug::Rational
      canonical.rs     # Canonical ordering for n-ary Add/Mul children
      render/
        mod.rs         # RenderFormat trait
        latex.rs       # LaTeX rendering (impl Display-like for LaTeX)
        unicode.rs     # Unicode terminal rendering (impl fmt::Display)
    tests/
      arena_tests.rs   # Hash-consing invariant tests
      number_tests.rs  # BigInt/BigRat edge case tests
      render_tests.rs  # LaTeX + Unicode snapshot tests
```

### Pattern 1: Arena-Based Hash Consing (Core Pattern)

**What:** All expressions are stored in a single Vec<Expr>. A dedup HashMap ensures structural identity. ExprRef is a Copy u32 index.

**When to use:** Every expression construction operation goes through the arena.

**Critical details:**

```rust
use rustc_hash::FxHashMap;

/// 32-bit index into the arena. Copy, cheap, no lifetimes.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ExprRef(u32);

/// Owns all expression memory. One per computation session.
pub struct ExprArena {
    nodes: Vec<Expr>,
    dedup: FxHashMap<Expr, ExprRef>,
    symbols: SymbolRegistry,
}

impl ExprArena {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            dedup: FxHashMap::default(),
            symbols: SymbolRegistry::new(),
        }
    }

    /// The core operation: intern an expression.
    /// Returns existing ref if structurally identical expression exists.
    pub fn intern(&mut self, expr: Expr) -> ExprRef {
        if let Some(&existing) = self.dedup.get(&expr) {
            return existing;
        }
        let id = ExprRef(self.nodes.len() as u32);
        self.nodes.push(expr.clone());
        self.dedup.insert(expr, id);
        id
    }

    /// O(1) lookup by ref.
    pub fn get(&self, r: ExprRef) -> &Expr {
        &self.nodes[r.0 as usize]
    }

    /// Structural equality is identity equality thanks to hash-consing.
    /// Two ExprRefs are equal iff they point to structurally identical expressions.
    /// This is O(1) -- just compare u32 values.
}
```

**Source:** Pattern from matklad's interner (https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html), rustc memory management (https://rustc-dev-guide.rust-lang.org/memory.html), and Zhu et al. hash-consing paper (arXiv:2509.20534).

### Pattern 2: N-ary Add/Mul with Canonical Sorting

**What:** Addition and multiplication are n-ary (not binary). Children are stored in a SmallVec/Vec sorted by a canonical ordering. This ensures that `a + b` and `b + a` produce identical arena entries.

**When to use:** Every time an Add or Mul expression is constructed.

**The canonical ordering must be a total order on ExprRef:**
- Since ExprRef is just u32, we can use numeric ordering as the canonical sort.
- BUT: this means the ordering depends on insertion order into the arena, which may vary between runs.
- For deterministic output, define a structural ordering on Expr: atoms < compounds, then by variant discriminant, then recursively by children.
- Trade-off: structural ordering is more expensive (recursive) but produces consistent output. Numeric ordering is O(1) but non-deterministic across sessions.

**Recommendation:** Use ExprRef numeric ordering (u32) for hash-consing dedup. Use structural ordering for display/rendering. The numeric ordering is sufficient for the hash-consing invariant (same children in same order = same hash) because Add/Mul children are always sorted before interning.

```rust
/// Canonicalize children before constructing Add/Mul.
pub fn make_add(arena: &mut ExprArena, mut children: Vec<ExprRef>) -> ExprRef {
    children.sort();  // u32 ordering
    children.dedup(); // remove exact duplicates (same ref = same expr)
    match children.len() {
        0 => arena.intern(Expr::Integer(Integer::ZERO)),
        1 => children[0],
        _ => arena.intern(Expr::Add(children)),
    }
}
```

**Source:** Symbolica documentation (https://symbolica.io/docs/guide/expressions.html) confirms n-ary sorted commutative operators as standard CAS pattern. SymPy uses the same approach.

### Pattern 3: Number Wrappers for Hash-Consing Compatibility

**What:** rug::Integer implements Hash (confirmed: rug::Rational does implement Hash per docs.rs). However, we should wrap rug types in newtypes for two reasons: (1) future-proofing against backend changes, (2) controlling Hash/Eq semantics precisely.

**CRITICAL VERIFICATION NEEDED:** Whether rug::Integer implements std::hash::Hash. The docs.rs page for rug::Rational explicitly lists Hash. For Integer, it was not conclusively confirmed in docs during this research. If Integer does NOT implement Hash, we need a wrapper that hashes the byte representation.

**Workaround if needed:**

```rust
/// Wrapper that guarantees Hash for BigInt values.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct QInt(pub rug::Integer);

impl std::hash::Hash for QInt {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the canonical byte representation
        // rug::Integer provides to_digits() for this
        let digits = self.0.to_digits::<u8>(rug::integer::Order::Msf);
        digits.hash(state);
        // Also hash the sign
        self.0.cmp0().hash(state);
    }
}
```

**Recommendation:** First try using rug::Integer directly in the Expr enum. If the compiler rejects it for missing Hash, wrap it. Test this immediately in Plan 01-01.

### Pattern 4: Expr Enum Design (Minimal for Phase 1)

**What:** The Expr enum for Phase 1 should contain ONLY what's needed for this phase's success criteria. Not all future variants.

```rust
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Expr {
    // --- Atoms ---
    Integer(QInt),                      // arbitrary precision integer
    Rational(QRat),                     // exact rational (num/denom)
    Symbol(SymbolId),                   // interned variable name
    Infinity,                           // formal infinity for (a;q)_inf

    // --- Arithmetic (n-ary, canonically sorted) ---
    Add(Vec<ExprRef>),                  // sum of terms, sorted
    Mul(Vec<ExprRef>),                  // product of factors, sorted
    Neg(ExprRef),                       // unary negation
    Pow(ExprRef, ExprRef),              // base^exponent

    // --- q-Specific Primitives (all needed for success criterion 1) ---
    QPochhammer {
        base: ExprRef,                  // a in (a;q)_n
        nome: ExprRef,                  // q
        order: ExprRef,                 // n (Integer, Symbol, or Infinity)
    },
    JacobiTheta {
        index: u8,                      // 1-4 (theta_1 through theta_4)
        nome: ExprRef,                  // q
    },
    DedekindEta(ExprRef),              // eta(tau), argument is tau
    BasicHypergeometric {
        upper: SmallVec<[ExprRef; 4]>,  // a_1..a_r
        lower: SmallVec<[ExprRef; 4]>,  // b_1..b_s
        nome: ExprRef,                  // q
        argument: ExprRef,              // z
    },

    // --- Placeholder for undefined/error ---
    Undefined,
}
```

**Design decisions within Expr:**

1. **Include Neg variant (diverging from ARCHITECTURE.md).** The architecture research said "no Neg, use Mul([-1, x])." For Phase 1 this is premature optimization -- it makes rendering harder (must detect Mul([-1, x]) pattern for display) and construction more complex. Keep Neg for now; simplification in Phase 2 can normalize to Mul if desired.

2. **No QBinomial variant yet.** q-binomial is defined as a ratio of q-Pochhammer symbols: [n,k]_q = (q;q)_n / ((q;q)_k * (q;q)_{n-k}). It can be represented as a compound expression. Add a dedicated variant in Phase 3 if pattern matching needs it.

3. **SmallVec for hypergeometric params.** Most basic hypergeometric series are _2phi_1 or _3phi_2. SmallVec<[ExprRef; 4]> avoids heap for the common case.

4. **JacobiTheta uses index u8 (1-4), not separate variants.** Fewer variants, same expressiveness.

5. **Infinity is an atom, not a special case of Integer.** Clean separation: infinity is not a number, it is a formal bound in (a;q)_inf.

### Pattern 5: SymbolId Registry (Append-Only Interning)

**What:** Symbols (variable names like q, a, b, tau) are interned into a registry that maps names to SymbolId (u32 index). This avoids string comparison in expression equality.

```rust
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct SymbolId(u32);

pub struct SymbolRegistry {
    names: Vec<String>,
    lookup: FxHashMap<String, SymbolId>,
}

impl SymbolRegistry {
    pub fn intern(&mut self, name: &str) -> SymbolId {
        if let Some(&id) = self.lookup.get(name) {
            return id;
        }
        let id = SymbolId(self.names.len() as u32);
        self.names.push(name.to_owned());
        self.lookup.insert(name.to_owned(), id);
        id
    }

    pub fn name(&self, id: SymbolId) -> &str {
        &self.names[id.0 as usize]
    }
}
```

**Key:** SymbolRegistry lives inside ExprArena (or alongside it in a Session). It is append-only -- symbols are never removed. This ensures SymbolId is always valid.

### Anti-Patterns to Avoid

- **Anti-pattern: Using Box<Expr> for recursive types.** Use ExprRef (u32 index into arena) instead. Box creates heap allocations per node, prevents hash-consing, and complicates lifetime management. ExprRef is Copy, 4 bytes, cache-friendly.

- **Anti-pattern: Global static intern table.** The hashconsing crate's `consign!` macro creates a global static. This prevents multiple arenas, makes testing harder, and is a memory leak vector. Use arena-scoped dedup tables.

- **Anti-pattern: Mutable expression nodes.** Once interned, an Expr must never be mutated. Mutation breaks hash-consing invariants (other ExprRefs pointing to the same node would see unexpected changes). All "modification" is done by constructing new expressions.

- **Anti-pattern: Deriving PartialEq on Expr and using it for equality checks between expressions.** After hash-consing, use `ExprRef == ExprRef` (u32 comparison, O(1)). Only use Expr-level PartialEq for the dedup HashMap during interning. Comparing full Expr structs is O(n) and defeats the purpose.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Arbitrary precision integers | Custom bignum implementation | `rug::Integer` (GMP) | GMP has 30+ years of optimization, inline assembly. Even partial reimplementation would take months and be slower. |
| Arbitrary precision rationals | Custom rational type | `rug::Rational` (GMP mpq) | Auto-canonicalization (always in lowest terms), GCD during construction, overflow-safe. |
| Fast hashing for dedup table | Custom hash function | `FxHashMap` from `rustc-hash` | Used by the Rust compiler itself. 2-5x faster than SipHash for small keys. |
| Unicode subscript/superscript | Manual char lookup table | `fmtastic::Subscript`/`Superscript` or a ~20-line lookup | Trivial either way, but don't spend time on this. |

**Key insight:** Phase 1 has exactly one hand-rolled component: the ExprArena (Vec + HashMap interner). Everything else uses existing libraries. The arena is ~50-80 lines of code and is the core architectural differentiator.

---

## Common Pitfalls

### Pitfall 1: rug::Integer Hash Compatibility

**What goes wrong:** You put rug::Integer directly into the Expr enum, derive Hash on Expr, and the compiler rejects it because Integer doesn't implement Hash. Or worse: it compiles but two equal Integers hash differently because the hash implementation is inconsistent.

**Why it happens:** GMP's mpz_t has internal allocation and capacity fields that are not mathematically meaningful but may affect byte representation.

**How to avoid:**
- Test immediately: create a minimal Rust file with `#[derive(Hash)] struct Foo(rug::Integer)`. If it compiles, rug::Integer implements Hash. If not, wrap it.
- If wrapping: hash the canonical digit representation via `to_digits()` method, which rug::Integer provides.
- Test the invariant: `a == b` implies `hash(a) == hash(b)` for various integer values.

**Warning signs:** Compiler errors on `#[derive(Hash)]` for Expr. Or hash-consing tests failing (identical integers getting different ExprRefs).

### Pitfall 2: Canonical Ordering Inconsistency

**What goes wrong:** Add(vec![ref_a, ref_b]) and Add(vec![ref_b, ref_a]) produce different ExprRefs because children are not sorted before interning. This means `a + b != b + a` at the ExprRef level, completely breaking hash-consing for commutative operations.

**Why it happens:** Forgetting to sort children before calling arena.intern(). Or sorting by one criterion in one place and a different criterion elsewhere.

**How to avoid:**
- All Add/Mul construction goes through helper functions (make_add, make_mul) that sort children.
- Never construct Add/Mul Expr variants directly -- always through the helper.
- Property-based test: for random children lists, shuffling the list and constructing Add should always produce the same ExprRef.

**Warning signs:** Tests pass with ordered inputs but fail with unordered. Expression count in arena grows unexpectedly (duplicates).

### Pitfall 3: GMP Build on Windows/Cygwin

**What goes wrong:** gmp-mpfr-sys fails to build on the development platform (Cygwin). The build script tries to compile GMP from source and encounters path issues, missing tools, or wrong compiler flags.

**Why it happens:** gmp-mpfr-sys compiles GMP from bundled C source using autotools. This can fail if: (1) path contains spaces, (2) wrong toolchain (needs GCC, not MSVC), (3) missing m4 or make.

**How to avoid:**
- Cygwin IS an explicitly supported build target for GMP (confirmed in GMP docs).
- Ensure the project path has no spaces.
- Ensure Cygwin has: gcc, make, m4 installed (`apt-cyg install gcc-core make m4` or equivalent).
- Alternatively: use `use-system-libs` feature to link against pre-installed GMP if available.
- First task in Plan 01-01: verify `cargo build` succeeds with rug before writing any domain code.

**Warning signs:** Build failures mentioning "configure", "cc", or "autotools".

### Pitfall 4: Over-Engineering the Expr Enum

**What goes wrong:** You add 20 variants to Expr for every possible future expression type, including things Phase 1 doesn't need (Sum, Product with bounds, FPS reference, etc.). Each variant increases the enum size, adds match arms to every function, and creates untested code paths.

**Why it happens:** Looking at the ARCHITECTURE.md from project research and trying to build the "final" Expr from the start. Architecture research shows the eventual target, not the Phase 1 scope.

**How to avoid:**
- Start with the minimal set from this research (see Pattern 4 above): atoms, arithmetic ops, and the 4 q-specific nodes required by success criterion 1.
- The Expr enum will grow in Phase 2 (FPS), Phase 3 (Sum/Product with bounds), etc. That is expected and correct.
- Count your variants. Phase 1 should have ~12-14 variants, not 20+.

**Warning signs:** Expr enum has variants with no tests. match arms with `todo!()` or `unimplemented!()` in rendering code.

### Pitfall 5: LaTeX Rendering Edge Cases

**What goes wrong:** LaTeX output compiles for simple cases but breaks for: negative exponents (needs braces), nested fractions, multi-character subscripts, or products-of-products. The success criterion says "valid LaTeX that compiles without errors" -- partial coverage is not enough.

**Why it happens:** LaTeX has quirky rules: `q^{-1}` needs braces but `q^2` doesn't (but `q^{10}` does). Nested `\frac{}{}` can overflow page width. Missing `\left(` / `\right)` on sized delimiters.

**How to avoid:**
- Build a test suite of LaTeX strings. For EACH Expr variant, construct a representative expression, render to LaTeX, and verify it compiles (or at least parses as valid LaTeX).
- Use consistent brace policy: always brace subscripts and superscripts (e.g., `q^{2}` even though `q^2` works). This eliminates a class of edge-case bugs.
- Test with nested expressions: (a;q)_{n} where n is itself a sum, or a product of q-Pochhammer symbols.

**Warning signs:** LaTeX output that looks right in the source but fails to compile with pdflatex.

---

## Code Examples

### Example 1: Creating and Interning Expressions

```rust
let mut arena = ExprArena::new();

// Intern atoms
let q = arena.intern_symbol("q");
let a = arena.intern_symbol("a");
let five = arena.intern(Expr::Integer(QInt::from(5)));
let inf = arena.intern(Expr::Infinity);

// Intern a q-Pochhammer symbol: (a; q)_5
let qpoch_finite = arena.intern(Expr::QPochhammer {
    base: a,
    nome: q,
    order: five,
});

// Intern (a; q)_inf
let qpoch_inf = arena.intern(Expr::QPochhammer {
    base: a,
    nome: q,
    order: inf,
});

// Hash-consing dedup: constructing the same expression returns the same ref
let qpoch_finite_2 = arena.intern(Expr::QPochhammer {
    base: a,
    nome: q,
    order: five,
});
assert_eq!(qpoch_finite, qpoch_finite_2);  // O(1) comparison: same u32
```

### Example 2: BigInt/BigRat Arithmetic

```rust
use rug::{Integer, Rational};

// Basic integer arithmetic
let a = Integer::from(123456789);
let b = Integer::from(987654321);
let product = Integer::from(&a * &b);  // Exact, arbitrary precision
assert_eq!(product.to_string(), "121932631112635269");

// Rational arithmetic
let half = Rational::from((1, 2));
let third = Rational::from((1, 3));
let sum = Rational::from(&half + &third);
assert_eq!(sum, Rational::from((5, 6)));  // Auto-reduced to lowest terms

// Edge cases to test
let zero = Integer::from(0);
let neg = Integer::from(-42);
let big = Integer::from_str_radix(
    "123456789012345678901234567890", 10
).unwrap();
```

### Example 3: LaTeX Rendering

```rust
// Expected LaTeX output for each expression type:

// Integer(42)               -> "42"
// Integer(-7)               -> "-7"
// Rational(3/4)             -> "\\frac{3}{4}"
// Symbol("q")               -> "q"
// Symbol("alpha")           -> "\\alpha"  (recognized Greek letters)
// Infinity                  -> "\\infty"
// Add([a, b])               -> "a + b"
// Mul([a, b])               -> "a \\cdot b"  (or "ab" for symbols)
// Neg(x)                    -> "-x"
// Pow(q, n)                 -> "q^{n}"
// QPochhammer{a, q, n}      -> "(a ; q)_{n}"
// QPochhammer{a, q, inf}    -> "(a ; q)_{\\infty}"
// JacobiTheta{2, q}         -> "\\theta_{2}(q)"
// DedekindEta(tau)           -> "\\eta(\\tau)"
// BasicHypergeometric        -> "{}_{r}\\phi_{s}\\!\\left(\\begin{matrix} a_1, \\ldots, a_r \\\\ b_1, \\ldots, b_s \\end{matrix} ; q, z\\right)"
```

### Example 4: Unicode Terminal Rendering

```rust
// Expected Unicode output for each expression type:

// Integer(42)               -> "42"
// Rational(3/4)             -> "3/4"
// Symbol("q")               -> "q"
// Infinity                  -> "\u{221e}"  (infinity symbol)
// Add([a, b])               -> "a + b"
// Mul([a, b])               -> "a*b"
// Neg(x)                    -> "-x"
// Pow(q, 2)                 -> "q\u{00b2}"  (using superscript 2)
// Pow(q, n)                 -> "q^n"  (fallback for non-numeric exponents)
// QPochhammer{a, q, n}      -> "(a;q)\u{2099}"  (subscript n where possible)
// QPochhammer{a, q, 5}      -> "(a;q)\u{2085}"  (subscript 5)
// QPochhammer{a, q, inf}    -> "(a;q)\u{221e}"   (or "(a;q)_inf")
// JacobiTheta{2, q}         -> "\u{03b8}\u{2082}(q)"  (theta with subscript 2)
// DedekindEta(tau)           -> "\u{03b7}(\u{03c4})"  (eta(tau))
// BasicHypergeometric        -> "_r\u{03c6}_s(a1,...,ar; b1,...,bs; q, z)"
```

**Unicode characters used:**
- Greek: theta (\u{03b8}), eta (\u{03b7}), tau (\u{03c4}), phi (\u{03c6}), alpha (\u{03b1}), etc.
- Subscripts: \u{2080}-\u{2089} for digits 0-9, \u{2099} for n
- Superscripts: \u{2070}, \u{00b9}, \u{00b2}, \u{00b3}, \u{2074}-\u{2079}
- Infinity: \u{221e}
- Product: \u{220f}

**Limitation:** Unicode subscripts/superscripts only cover digits and a few letters (n, i, etc.). For complex sub/superscript expressions, fall back to `_expr` or `^expr` ASCII notation.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `hashconsing` crate with Arc + global table | Custom arena (Vec + HashMap) with u32 indices | ~2023-2024 (pattern popularized by egg, cranelift) | Better cache locality, no Arc overhead, arena-scoped memory control |
| `num-bigint` as go-to arbitrary precision | `rug` (GMP bindings) or `malachite` for performance-sensitive code | rug 1.28+ (2025) requires Rust 1.85+ | 5-10x faster for large numbers |
| Binary Add(left, right) | N-ary Add(Vec<ExprRef>) with canonical sorting | Standard since Symbolica, SymPy, Maple | Better hash-consing hit rate, simpler pattern matching |
| `SipHash` (Rust default hasher) | `FxHash` or `AHash` for non-crypto HashMap | Ongoing best practice | 2-5x faster for small keys like u32/u64 |

**Deprecated/outdated:**
- `hashconsing::consign!` macro for global intern tables -- replaced by arena-scoped patterns
- Binary tree expressions (left + right) for commutative ops -- replaced by n-ary with sorted children
- `num-bigint` as primary arithmetic backend for CAS work -- too slow, use rug/GMP

---

## Open Questions

1. **Does rug::Integer implement std::hash::Hash?**
   - What we know: rug::Rational is confirmed to implement Hash (docs.rs shows it). Integer likely does too (it would be unusual for Rational to have Hash without Integer having it, since Rational contains two Integers).
   - What's unclear: Could not get definitive confirmation from docs or source.
   - Recommendation: Test immediately in Plan 01-01. If Integer does not implement Hash, implement a wrapper (QInt) that hashes via `to_digits()`. This is ~10 lines of code.

2. **Neg variant vs Mul([-1, x]) normalization**
   - What we know: SymPy and Symbolica both normalize negation to multiplication by -1. The ARCHITECTURE.md from project research recommends this.
   - What's unclear: Whether doing this in Phase 1 is beneficial or premature. It affects rendering (must detect the pattern) and construction (more complex).
   - Recommendation: Keep Neg as a separate variant in Phase 1 for simplicity. Phase 2's simplification engine can normalize if needed. The Expr enum can drop Neg later without breaking external APIs.

3. **Should SymbolRegistry be inside ExprArena or alongside it?**
   - What we know: Symbols are referenced by SymbolId in Expr, so the registry must be accessible when rendering.
   - What's unclear: Ownership model. If SymbolRegistry is inside ExprArena, the arena's &mut self prevents borrowing symbols while also interning.
   - Recommendation: Put SymbolRegistry inside ExprArena but provide a `symbols(&self) -> &SymbolRegistry` accessor. Intern symbols before interning expressions that use them. Alternative: keep them separate in a Session struct (which is the Phase 5 pattern anyway).

4. **Structural ordering vs insertion-order for canonical sorting**
   - What we know: ExprRef is u32, so numeric ordering is trivial. But numeric ordering depends on arena insertion order, which may vary.
   - What's unclear: Whether this non-determinism matters in practice. Two runs constructing the same expressions in the same order will produce identical arenas.
   - Recommendation: Use ExprRef numeric ordering for now. It is deterministic within a session. Cross-session determinism (for reproducible hashing, serialization) can be addressed later with a stable structural ordering if needed.

5. **LaTeX compilation testing: automated or manual?**
   - What we know: Success criterion 3 says "valid LaTeX that compiles without errors."
   - What's unclear: Whether to automate LaTeX compilation in CI (requires a TeX installation) or use structural validation (check for balanced braces, valid commands).
   - Recommendation: Structural validation in unit tests (balanced braces, no dangling backslashes, recognized commands). Manual spot-check with actual LaTeX compilation. Full CI LaTeX compilation is nice-to-have but not blocking.

---

## Rendering Strategy Detail

### LaTeX Rendering

LaTeX rendering is string generation: traverse the expression tree, produce a LaTeX string.

**Implementation approach:** Implement a `to_latex(&self, arena: &ExprArena) -> String` method on ExprRef (or a free function). Recursive traversal, each Expr variant maps to a LaTeX fragment.

**Key formatting rules:**

| Expression | LaTeX | Notes |
|------------|-------|-------|
| Integer n (n >= 0) | `n` | Direct string conversion |
| Integer n (n < 0) | `n` or `(-n)` in context | Negative in sum: `-n`, negative standalone: `(-n)` |
| Rational p/q | `\frac{p}{q}` | Always use \frac, never p/q |
| Symbol "q" | `q` | Single-letter symbols: bare |
| Symbol "alpha" | `\alpha` | Greek letter detection and mapping |
| Symbol "x_1" | `x_{1}` | Subscript detection |
| Infinity | `\infty` | |
| Add(terms) | `t_1 + t_2 + ... + t_n` | Detect negative terms for minus sign |
| Mul(factors) | Context-dependent | Numbers: `2 \cdot x`, symbols: `ab`, with powers: `a^{2} b` |
| Neg(x) | `-x` or `-(expr)` | Parenthesize compound expressions |
| Pow(b, e) | `b^{e}` | Always brace exponent. Parenthesize base if compound. |
| QPochhammer(a, q, n) | `(a ; q)_{n}` | Standard notation from DLMF 17.2 |
| JacobiTheta(i, q) | `\theta_{i}(q)` | |
| DedekindEta(tau) | `\eta(\tau)` | |
| BasicHypergeometric | Matrix notation | `{}_{r}\phi_{s}\!\left(\begin{matrix}...\end{matrix};q,z\right)` |

**Parenthesization strategy:** Use a precedence system:
- Atoms (Integer, Symbol, Infinity): no parens needed
- Pow: parenthesize base if it's Add or Mul
- Mul: parenthesize Add children
- Add: no parenthesization of children (except Neg pattern)
- q-Specific: parenthesization built into notation (q-Pochhammer always has parens)

### Unicode Rendering

Unicode rendering is for terminal/REPL display. More constrained than LaTeX.

**Implementation approach:** Implement `fmt::Display` for a wrapper type `DisplayExpr<'a>(&'a ExprArena, ExprRef)` so it integrates with Rust's formatting system.

```rust
pub struct DisplayExpr<'a> {
    arena: &'a ExprArena,
    expr: ExprRef,
}

impl<'a> fmt::Display for DisplayExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.arena.get(self.expr) {
            Expr::QPochhammer { base, nome, order } => {
                write!(f, "({};{})",
                    DisplayExpr { arena: self.arena, expr: *base },
                    DisplayExpr { arena: self.arena, expr: *nome })?;
                // Try to render order as subscript
                render_subscript(f, self.arena, *order)
            }
            // ... other variants
        }
    }
}
```

**Unicode subscript/superscript limitations:**
- Digit subscripts/superscripts: full coverage (0-9)
- Letter subscripts: only a, e, h, i, j, k, l, m, n, o, p, r, s, t, u, v, x
- Letter superscripts: only i, n and a few others
- For expressions that cannot be rendered as Unicode sub/superscripts, fall back to `_expr` or `^(expr)` ASCII notation

---

## BigInt/BigRat Edge Cases to Test

These edge cases are required by success criterion 2 ("matching GMP reference output for edge cases").

### Integer Edge Cases

| Operation | Inputs | Expected | Why It Matters |
|-----------|--------|----------|----------------|
| Addition | 0 + 0 | 0 | Zero identity |
| Addition | MAX_I64 + 1 | Correct big value | Overflow from machine word |
| Multiplication | -1 * x | -x | Sign handling |
| Multiplication | 0 * anything | 0 | Zero annihilation |
| Division | 7 / 3 | 2 (truncated) or Rational(7,3) | Integer vs rational semantics |
| Division | x / 0 | Error/undefined | Must not panic silently |
| Power | 0^0 | 1 (convention) | Mathematical convention |
| Power | (-2)^3 | -8 | Negative base, odd exponent |
| Power | 2^64 | 18446744073709551616 | Exceeds u64 |
| Comparison | -0 == 0 | true | GMP normalizes -0 to 0 |

### Rational Edge Cases

| Operation | Inputs | Expected | Why It Matters |
|-----------|--------|----------|----------------|
| Construction | (6, 4) | 3/2 | Auto-reduction |
| Construction | (-3, -5) | 3/5 | Double negative |
| Construction | (0, n) | 0/1 | Zero numerator |
| Construction | (n, 0) | Error | Zero denominator |
| Addition | 1/3 + 1/6 | 1/2 | Common denominator + reduction |
| Subtraction | 1/2 - 1/2 | 0 | Exact cancellation |
| Multiplication | (1/3) * (3/1) | 1 | Multiplicative inverse |
| Large values | p(100)/p(200) | exact | No precision loss |

---

## Sources

### Primary (HIGH confidence)
- [rug 1.28.1 docs](https://docs.rs/rug/latest/rug/) -- rug API, version, features, Rational traits
- [rug crate on crates.io](https://crates.io/crates/rug) -- version 1.28.1, requires Rust 1.85+
- [gmp-mpfr-sys docs](https://docs.rs/gmp-mpfr-sys) -- GMP 6.3.0, MPFR 4.2.2, build requirements, Cygwin support
- [hashconsing crate docs](https://docs.rs/hashconsing) -- HConsed API, uid equality, consign pattern
- [matklad's Fast Simple Rust Interner](https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html) -- Vec+HashMap interner pattern
- [rustc memory management](https://rustc-dev-guide.rust-lang.org/memory.html) -- Arena + interning in the Rust compiler
- [DLMF 17.2: q-Hypergeometric Notation](https://dlmf.nist.gov/17.2) -- Standard LaTeX notation for q-Pochhammer, basic hypergeometric
- [q-Pochhammer Symbol (Wikipedia)](https://en.wikipedia.org/wiki/Q-Pochhammer_symbol) -- Notation, edge cases, conventions

### Secondary (MEDIUM confidence)
- [Symbolica Expressions Guide](https://symbolica.io/docs/guide/expressions.html) -- N-ary sorted operators, canonical ordering
- [Hash consing paper (arXiv:2509.20534)](https://arxiv.org/html/2509.20534v2) -- 3-100x speedups, memory model
- [fmtastic crate](https://lib.rs/crates/fmtastic) -- Unicode subscript/superscript formatting, v0.2.2
- [Arenas in Rust (Manish Goregaokar)](https://manishearth.github.io/blog/2021/03/15/arenas-in-rust/) -- Arena allocation patterns
- [display_as crate](https://docs.rs/display-as) -- Format-parameterized Display trait (LaTeX, HTML, Math)

### Tertiary (LOW confidence -- needs validation)
- Whether rug::Integer implements std::hash::Hash -- could not confirm definitively from web sources. Likely yes (Rational does, and Rational contains Integers). **Must verify in code.**
- Exact Unicode subscript coverage for mathematical letters -- based on Unicode standard knowledge, not verified against specific terminal rendering.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- rug, smallvec, fxhash are well-documented and widely used
- Architecture (arena + hash-consing): HIGH -- pattern verified from multiple authoritative sources (rustc, egg, hash-consing paper)
- Rendering: MEDIUM-HIGH -- LaTeX notation is standard (DLMF), Unicode approach is straightforward but has edge cases
- Pitfalls: HIGH -- identified from project research + CAS literature + Rust ecosystem knowledge

**Research date:** 2026-02-13
**Valid until:** 2026-03-13 (stable domain; rug and core patterns unlikely to change)
