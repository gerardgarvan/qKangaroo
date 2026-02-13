# Phase 2: Simplification & Series Engine - Research

**Researched:** 2026-02-13
**Domain:** Phased term rewriting, formal power series arithmetic, lazy coefficient generation (Rust)
**Confidence:** HIGH

## Summary

Phase 2 builds two co-dependent subsystems on top of the Phase 1 expression foundation: (1) a phased simplification/rewrite engine that applies rules in priority order with guaranteed termination, and (2) a formal power series (FPS) engine with sparse representation, truncated arithmetic, and lazy coefficient generation for infinite products. These are co-dependent because simplification rules need series expansion to verify results, and series multiplication needs simplification to keep intermediates small.

The research confirms that a **custom phased rewriter** (not egg/equality saturation) is the correct choice for Phase 2. Egg's `define_language!` macro requires all expression types to be defined as a single enum with `[Id; N]` children arrays, which is incompatible with our existing `ExprArena` + `ExprRef` hash-consing architecture. Egg also uses its own internal e-graph storage, creating a dual-storage problem. The prior research already recommended phased rewriting over pure e-graph for Phase 2, and this research confirms that decision with concrete implementation evidence. Egg integration should be deferred to a future phase (Phase 6+ for deep equivalence checking) where it adds genuine value beyond what phased rewriting provides.

For the FPS engine, `BTreeMap<i64, QRat>` is the correct sparse representation. The key insight is that truncation must be enforced **during** multiplication (not after), producing O(N) storage per series where N is the truncation order, rather than O(N^2) intermediate swell. Lazy generators for infinite products use `Box<dyn FnMut(i64, &BTreeMap<i64, QRat>) -> QRat>` closures that compute coefficients on demand. The existing `series` Rust crate uses dense `Vec` storage with a starting power and cutoff, making it unsuitable for q-series work where series are inherently sparse.

**Primary recommendation:** Build a custom 6-phase bottom-up rewrite engine operating directly on ExprArena/ExprRef, paired with a from-scratch sparse FPS engine using BTreeMap with explicit truncation order tracking. Do not integrate egg in this phase.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rug` | 1.28 | QRat coefficient arithmetic in FPS | Already in use from Phase 1; GMP-backed exact rational arithmetic is non-negotiable for q-series coefficients |
| `rustc-hash` | 2 | FxHashMap for pattern match caches | Already in use; O(1) lookup for memoizing simplification results |
| `smallvec` | 1 | Inline storage for small rule match results | Already in use; avoids heap allocation for common 1-3 element match results |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `proptest` | 1 | Property-based testing for series arithmetic invariants | Testing that series operations satisfy mathematical identities (commutativity, associativity, distributivity) |
| `serde` | 1.0 | Serialization of FPS for debugging/caching | Already a dependency; derive Serialize/Deserialize on FormalPowerSeries |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom phased rewriter | `egg` 0.11 (equality saturation) | Egg has its own internal expression storage (RecExpr), incompatible with our ExprArena. Egg's Language trait requires `[Id; N]` fixed-arity children, but our Add/Mul are n-ary Vec<ExprRef>. Integration would require maintaining dual representations. Phased rewriting is simpler, predictable, and sufficient for Phase 2 simplification needs. Defer egg to Phase 6+ for deep equivalence discovery. |
| Custom phased rewriter | `term_rewriting` crate | Low-level TRS library. Provides parsing and rule application but no arena integration, no hash-consing awareness. Would still need extensive custom code. Better to build directly on our arena. |
| Custom FPS (BTreeMap) | `series` crate | Series crate uses dense Vec storage with starting power offset. Not sparse -- wasteful for q-series like `1 + q^1000`. No lazy generation. No QRat/BigRat support out of the box. Unsuitable for our domain. |
| BTreeMap<i64, QRat> | HashMap<i64, QRat> | BTreeMap gives ordered iteration (needed for display, truncation, coefficient extraction in order) at O(log n) per access. For typical series sizes (100-1000 nonzero terms), the difference is negligible. BTreeMap avoids hashing QRat (expensive for large rationals). |
| BTreeMap<i64, QRat> | Vec<(i64, QRat)> sorted | Vec would give better cache locality but O(n) insertion. BTreeMap's O(log n) insertion matters during multiplication where we insert results incrementally. Vec is better for read-only iteration but worse for incremental construction. |

**Dependencies to add to Cargo.toml:**

No new dependencies required. Phase 2 uses only libraries already in Cargo.toml from Phase 1.

## Architecture Patterns

### Recommended Module Structure

```
crates/qsym-core/src/
  lib.rs              # add: pub mod simplify; pub mod series;
  simplify/
    mod.rs            # SimplificationEngine, SimplificationPhase, TraversalStrategy
    rule.rs           # RewriteRule, Pattern, PatternNode, Condition
    pattern.rs        # Pattern matching on ExprRef against Pattern trees
    phases/
      mod.rs          # Phase registry, phase ordering
      normalize.rs    # Phase 1: arithmetic normalization
      cancel.rs       # Phase 2: cancellation (0*x -> 0, x+0 -> x)
      collect.rs      # Phase 3: collect like terms (2*x + 3*x -> 5*x)
      simplify_arith.rs # Phase 4: arithmetic simplification (Neg(Neg(x)) -> x)
      expand.rs       # Phase 5: controlled expansion
      verify.rs       # Phase 6: verification/fixpoint check
  series/
    mod.rs            # FormalPowerSeries, pub API
    arithmetic.rs     # add, mul, sub, negate, truncate, shift
    generator.rs      # LazyGenerator trait, infinite product generators
    display.rs        # Display/Debug for FPS, "1 + q + q^2 + O(q^3)" format
```

### Pattern 1: Bottom-Up Phased Simplification with Fixed-Point Detection

**What:** The simplification engine traverses expressions bottom-up (children first, then parent), applies rules from the current phase, and uses hash-consing for O(1) change detection. When a phase produces changes, restart from phase 1. Cap total iterations.

**When to use:** Every call to `simplify()`.

**Example:**

```rust
// Source: Architecture research + standard TRS pattern
pub struct SimplificationEngine {
    phases: Vec<SimplificationPhase>,
    max_total_iterations: usize,  // default: 100
}

pub struct SimplificationPhase {
    pub name: &'static str,
    pub rules: Vec<RewriteRule>,
    pub max_phase_iterations: usize,  // default: 10
    pub strategy: TraversalStrategy,
}

pub enum TraversalStrategy {
    BottomUp,   // simplify children first (default, most phases)
    TopDown,    // try root first (expansion phase)
}

impl SimplificationEngine {
    pub fn simplify(&self, expr: ExprRef, arena: &mut ExprArena) -> ExprRef {
        let mut current = expr;
        let mut total_iters = 0;

        'outer: while total_iters < self.max_total_iterations {
            for phase in &self.phases {
                let result = self.apply_phase(phase, current, arena);
                if result != current {
                    // O(1) change detection via ExprRef equality (hash-consing!)
                    current = result;
                    total_iters += 1;
                    continue 'outer; // restart from phase 1
                }
            }
            break; // fixpoint reached -- no phase changed anything
        }
        current
    }

    fn apply_phase(
        &self,
        phase: &SimplificationPhase,
        expr: ExprRef,
        arena: &mut ExprArena,
    ) -> ExprRef {
        match phase.strategy {
            TraversalStrategy::BottomUp => self.bottom_up(phase, expr, arena),
            TraversalStrategy::TopDown => self.top_down(phase, expr, arena),
        }
    }

    fn bottom_up(
        &self,
        phase: &SimplificationPhase,
        expr: ExprRef,
        arena: &mut ExprArena,
    ) -> ExprRef {
        // First, recursively simplify all children
        let simplified_children = self.simplify_children(phase, expr, arena);
        // Then try rules at this node
        self.try_rules(&phase.rules, simplified_children, arena)
    }
}
```

### Pattern 2: RewriteRule with Pattern Matching on ExprRef

**What:** Rules are defined as pattern -> replacement pairs. Patterns are lightweight trees of PatternNode that match against ExprRef values in the arena. Pattern variables capture sub-expressions.

**When to use:** Defining simplification rules.

**Example:**

```rust
// Source: Standard TRS pattern adapted for arena architecture
pub enum PatternNode {
    /// Matches any expression, captures it under the given name
    Variable(String),
    /// Matches a specific integer value
    Integer(i64),
    /// Matches a specific expression variant with child patterns
    Add(Vec<PatternNode>),
    Mul(Vec<PatternNode>),
    Neg(Box<PatternNode>),
    Pow(Box<PatternNode>, Box<PatternNode>),
    // ... other variants
}

pub struct RewriteRule {
    pub name: &'static str,
    pub pattern: PatternNode,
    pub replacement: Replacement,
    pub condition: Option<Box<dyn Fn(&Bindings, &ExprArena) -> bool>>,
}

pub enum Replacement {
    /// Replace with a pattern template (substituting captured variables)
    Pattern(PatternNode),
    /// Replace via custom function (for complex rewrites)
    Function(Box<dyn Fn(&Bindings, &mut ExprArena) -> ExprRef>),
}

/// Captured pattern variable bindings
pub type Bindings = FxHashMap<String, ExprRef>;

// Example rules:
fn arithmetic_normalization_rules() -> Vec<RewriteRule> {
    vec![
        // x + 0 -> x
        RewriteRule {
            name: "add-zero",
            pattern: PatternNode::Add(vec![
                PatternNode::Variable("x".into()),
                PatternNode::Integer(0),
            ]),
            replacement: Replacement::Pattern(PatternNode::Variable("x".into())),
            condition: None,
        },
        // 0 * x -> 0
        RewriteRule {
            name: "mul-zero",
            pattern: PatternNode::Mul(vec![
                PatternNode::Integer(0),
                PatternNode::Variable("x".into()),
            ]),
            replacement: Replacement::Pattern(PatternNode::Integer(0)),
            condition: None,
        },
        // Neg(Neg(x)) -> x
        RewriteRule {
            name: "double-negation",
            pattern: PatternNode::Neg(Box::new(
                PatternNode::Neg(Box::new(PatternNode::Variable("x".into())))
            )),
            replacement: Replacement::Pattern(PatternNode::Variable("x".into())),
            condition: None,
        },
    ]
}
```

### Pattern 3: FormalPowerSeries with Sparse BTreeMap and Explicit Truncation

**What:** FPS stores nonzero coefficients in a BTreeMap<i64, QRat> with an explicit truncation order. The invariant is: all coefficients with exponent >= truncation_order are unknown/not computed. Coefficients not in the map are implicitly zero.

**When to use:** All series arithmetic.

**Example:**

```rust
use std::collections::BTreeMap;
use crate::number::QRat;
use crate::symbol::SymbolId;

/// A formal power series in a single variable with sparse rational coefficients.
///
/// Represents f(q) = sum_{k=min_order}^{truncation_order-1} c_k * q^k + O(q^truncation_order)
///
/// Invariants:
/// - All keys in `coefficients` are < `truncation_order`
/// - Missing keys have coefficient 0
/// - No key maps to QRat::zero() (enforce on insertion)
/// - `truncation_order` is always tracked explicitly
#[derive(Clone, Debug)]
pub struct FormalPowerSeries {
    /// Sparse coefficients: exponent -> nonzero coefficient value
    coefficients: BTreeMap<i64, QRat>,
    /// Variable this series is in (usually "q")
    variable: SymbolId,
    /// Coefficients are exact for exponents < truncation_order
    /// Everything at or above this order is unknown
    truncation_order: i64,
}

impl FormalPowerSeries {
    /// Create the zero series: 0 + O(q^N)
    pub fn zero(variable: SymbolId, truncation_order: i64) -> Self {
        Self {
            coefficients: BTreeMap::new(),
            variable,
            truncation_order,
        }
    }

    /// Create a monomial: c * q^k + O(q^N)
    pub fn monomial(variable: SymbolId, coeff: QRat, power: i64, truncation_order: i64) -> Self {
        let mut fps = Self::zero(variable, truncation_order);
        if !coeff.is_zero() && power < truncation_order {
            fps.coefficients.insert(power, coeff);
        }
        fps
    }

    /// Get coefficient of q^k. Returns QRat::zero() for missing entries.
    /// Panics if k >= truncation_order (coefficient is unknown).
    pub fn coeff(&self, k: i64) -> QRat {
        assert!(k < self.truncation_order,
            "Cannot access coefficient at q^{k}: series only known to O(q^{trunc})",
            k = k, trunc = self.truncation_order);
        self.coefficients.get(&k).cloned().unwrap_or_else(QRat::zero)
    }

    /// Set coefficient of q^k. Removes entry if value is zero.
    pub fn set_coeff(&mut self, k: i64, value: QRat) {
        if k >= self.truncation_order {
            return; // beyond truncation, ignore
        }
        if value.is_zero() {
            self.coefficients.remove(&k);
        } else {
            self.coefficients.insert(k, value);
        }
    }

    /// Number of nonzero coefficients
    pub fn num_nonzero(&self) -> usize {
        self.coefficients.len()
    }

    /// Lowest power with nonzero coefficient, or None if zero series
    pub fn min_order(&self) -> Option<i64> {
        self.coefficients.keys().next().copied()
    }

    pub fn truncation_order(&self) -> i64 {
        self.truncation_order
    }
}
```

### Pattern 4: Truncated Series Multiplication (Critical for Performance)

**What:** Multiply two truncated series without creating O(N^2) intermediates. Only compute and store terms with exponent < min(truncation_order_a, truncation_order_b).

**When to use:** Every series multiplication.

**Example:**

```rust
/// Multiply two formal power series, truncating to O(q^N) where
/// N = min(a.truncation_order, b.truncation_order).
///
/// Time: O(|a| * |b|) where |x| is the number of nonzero coefficients
/// Space: O(N) where N is the truncation order (NOT O(|a| * |b|))
pub fn mul(a: &FormalPowerSeries, b: &FormalPowerSeries) -> FormalPowerSeries {
    assert_eq!(a.variable, b.variable, "Cannot multiply series in different variables");
    let trunc = a.truncation_order.min(b.truncation_order);

    let mut result = FormalPowerSeries::zero(a.variable, trunc);

    for (&ka, ca) in &a.coefficients {
        // Early termination: if ka already >= trunc, skip
        if ka >= trunc {
            continue;
        }
        for (&kb, cb) in &b.coefficients {
            let k = ka + kb;
            // CRITICAL: skip terms at or above truncation order
            if k >= trunc {
                // Since b.coefficients is sorted (BTreeMap), all remaining
                // kb values are larger, so k will only grow. Break inner loop.
                break;
            }
            // Accumulate: result[k] += ca * cb
            let product = ca * cb;  // QRat * QRat
            let entry = result.coefficients.entry(k).or_insert_with(QRat::zero);
            *entry = &*entry + &product;
        }
    }

    // Clean up any zero entries that resulted from cancellation
    result.coefficients.retain(|_, v| !v.is_zero());
    result
}
```

### Pattern 5: Lazy Generator for Infinite Products

**What:** Infinite products like (q;q)_inf = prod_{k=1}^{inf}(1-q^k) are computed lazily by multiplying one factor at a time, truncating at each step. A generator struct holds the current partial product and how many factors have been included.

**When to use:** Expanding q-Pochhammer symbols with infinite order, eta functions, Jacobi theta functions.

**Example:**

```rust
/// Lazily generates coefficients of an infinite product by
/// multiplying one factor at a time.
///
/// For (a;q)_inf = prod_{k=0}^{inf} (1 - a*q^k):
/// - Start with partial_product = 1 + O(q^N)
/// - Factor k=0: multiply by (1 - a)
/// - Factor k=1: multiply by (1 - a*q)
/// - Factor k=2: multiply by (1 - a*q^2)
/// - ... continue until factor doesn't affect terms below O(q^N)
pub struct InfiniteProductGenerator {
    /// Current partial product (truncated series)
    partial_product: FormalPowerSeries,
    /// How many factors have been multiplied in
    factors_included: i64,
    /// The function that generates the k-th factor as a series
    /// For (a;q)_inf, factor_k = 1 - a*q^k = series with coeff[0]=1, coeff[k]=-a
    factor_fn: Box<dyn FnMut(i64, SymbolId, i64) -> FormalPowerSeries>,
}

impl InfiniteProductGenerator {
    /// Ensure the partial product includes enough factors to be correct
    /// up to O(q^N). For (1-q^k) factors, we need at most N factors
    /// since factor k only affects terms at q^k and above.
    pub fn ensure_order(&mut self, target_order: i64) {
        while self.factors_included < target_order {
            let factor = (self.factor_fn)(
                self.factors_included,
                self.partial_product.variable,
                target_order,
            );
            self.partial_product = mul(&self.partial_product, &factor);
            self.factors_included += 1;
        }
    }

    /// Get the current series (valid up to however many factors included)
    pub fn series(&self) -> &FormalPowerSeries {
        &self.partial_product
    }
}

/// Create a generator for (q;q)_inf = prod_{k=1}^{inf} (1 - q^k)
/// This is the Euler function, whose coefficients encode the pentagonal
/// number theorem: only exponents k(3k-1)/2 have nonzero coefficients.
pub fn euler_function_generator(
    q_var: SymbolId,
    truncation_order: i64,
) -> InfiniteProductGenerator {
    let initial = FormalPowerSeries::monomial(
        q_var,
        QRat::one(),
        0,
        truncation_order,
    );

    InfiniteProductGenerator {
        partial_product: initial,
        factors_included: 1, // start at k=1 for (q;q)_inf
        factor_fn: Box::new(move |k, var, trunc| {
            // Factor = (1 - q^k)
            let mut factor = FormalPowerSeries::zero(var, trunc);
            factor.set_coeff(0, QRat::one());
            if k < trunc {
                factor.set_coeff(k, -QRat::one());
            }
            factor
        }),
    }
}
```

### Anti-Patterns to Avoid

- **Using egg for Phase 2 simplification:** Egg manages its own e-graph storage with `RecExpr` and `Id`. Integrating it with our ExprArena requires constant translation between two representations, is error-prone, and provides no benefit for the straightforward simplification rules needed in Phase 2. Save egg for Phase 6+ when we need deep equivalence discovery.

- **Dense coefficient arrays for FPS:** The `series` crate and similar libraries use `Vec<C>` indexed from a starting power. This wastes memory for sparse q-series (e.g., Euler function has nonzero coefficients only at generalized pentagonal numbers: 0, 1, 2, 5, 7, 12, 15, 22, 26, ...). BTreeMap stores only nonzero terms.

- **Post-hoc truncation in multiplication:** Computing the full convolution of two O(q^N) series creates O(q^2N) intermediates, then throwing away the top half. This doubles memory usage and wastes computation. Always truncate DURING multiplication by checking `ka + kb < trunc` before computing each term.

- **Bidirectional rewrite rules:** Mathematical identities are bidirectional (a = b means both a -> b and b -> a). Encoding both directions as simplification rules creates infinite loops. Only include the direction that reduces complexity (measured by a well-founded ordering on expressions). Transformations (bidirectional) are user-invoked operations, not automatic simplifications.

- **Storing zero coefficients in BTreeMap:** After series subtraction or cancellation, some entries may become zero. Always `.retain(|_, v| !v.is_zero())` after operations to maintain the sparsity invariant.

- **Mutable FPS during multiplication:** Do not modify either input series during multiplication. Create a fresh result FPS. This preserves hash-consing safety if series are later referenced by multiple expressions.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Arbitrary precision rational arithmetic | Custom BigRat | `rug::Rational` (QRat wrapper) | GMP's mpq is battle-tested for decades. QRat already wraps it with Hash support. Rational arithmetic in FPS coefficients is the hot path -- needs GMP speed. |
| Hash map for expression dedup | Custom hash table | `FxHashMap` from `rustc-hash` | Already in use for ExprArena dedup. FxHashMap is optimized for small keys (u32/u64) which is exactly our use case. |
| Ordered map for coefficients | Custom sorted vec | `BTreeMap<i64, QRat>` from stdlib | Standard library BTreeMap is well-optimized for Rust. O(log n) insert/lookup, ordered iteration, range queries. No reason to build custom. |
| Property-based testing | Custom random expr generators | `proptest` | Already a dev-dependency. Proptest handles shrinking, reproducibility, and strategy composition. Essential for testing algebraic properties of series arithmetic. |

**Key insight:** Phase 2 should add NO new dependencies. Everything needed (BTreeMap, QRat, ExprArena, FxHashMap) is already available from Phase 1 or stdlib. The complexity is in the algorithms and correctness, not in finding libraries.

## Common Pitfalls

### Pitfall 1: Rewrite Non-Termination

**What goes wrong:** Rule A rewrites f(x) to g(x), rule B rewrites g(x) back to f(x). Simplification loops forever.
**Why it happens:** Treating mathematical identities as bidirectional rules. Adding rules without checking the combined set for termination.
**How to avoid:** Define a strict complexity measure on expressions (e.g., weighted term count: atoms=1, Add/Mul=1+sum(children), Neg=1+child, Pow=2+base+exp). Only allow simplification rules where the RHS has strictly lower complexity than the LHS. Test with adversarial expressions: deeply nested Neg(Neg(Neg(...))), Add of many identical terms, Pow(Pow(x, 2), 2).
**Warning signs:** Tests hanging; memory growing during simplification; debug traces showing the same expression repeatedly.

### Pitfall 2: O(N^2) Intermediate Swell in Series Multiplication

**What goes wrong:** Multiplying two series truncated to O(q^N) creates an intermediate with up to N^2 terms before truncation. For N=10000, this means 100M terms temporarily.
**Why it happens:** Computing the full convolution before truncating. Natural to write but catastrophically wasteful.
**How to avoid:** Check `ka + kb < truncation_order` BEFORE computing each product term. Break the inner loop early (since BTreeMap iterates in order, once `ka + kb >= trunc`, all subsequent `kb` values are even larger). This bounds the result to at most N nonzero terms.
**Warning signs:** Multiplying two O(q^1000) series takes seconds instead of milliseconds. Memory usage much larger than 2 * sizeof(input).

### Pitfall 3: Losing Truncation Order Through Operations

**What goes wrong:** After adding two series, the result's truncation order is wrong. E.g., O(q^10) + O(q^20) should be O(q^10) (we only know the sum to the lower precision), but implementation sets it to O(q^20).
**Why it happens:** Not carefully tracking truncation through every operation.
**How to avoid:** For every binary operation, the result's truncation order is `min(a.truncation_order, b.truncation_order)`. For unary operations (negate, scalar multiply), truncation order is preserved. For exponentiation/composition, truncation analysis is more complex. Always assert truncation order in tests.
**Warning signs:** Tests pass with matching truncation orders but fail when mixing different orders. Silent wrong answers when extracting "high" coefficients.

### Pitfall 4: Pattern Matching on N-ary Add/Mul

**What goes wrong:** A rule for "x + 0 -> x" only matches `Add([x, 0])` (binary add). But our Add is n-ary: `Add([a, b, 0, c])`. The rule doesn't fire.
**Why it happens:** Pattern matching on n-ary operators requires subset matching, not exact structural matching. The pattern `Add([?x, 0])` should match any Add containing 0, extracting the remaining terms.
**How to avoid:** Implement pattern matching for Add/Mul as "does the pattern set appear as a subset of the children set?" When matched, the replacement receives both the matched children and the unmatched remainder. E.g., `Add([?x, 0])` on `Add([a, b, 0, c])` binds `?x` to... well, this is ambiguous. Better: special "absorb zero" rules operate directly on the children Vec, not through pattern matching. Use Rust match on the Expr enum for arithmetic normalization rules, reserve pattern matching for structural rules.
**Warning signs:** Simple rules like "add zero" don't fire on expressions with more than 2 terms.

### Pitfall 5: Coefficient Cancellation Creating False Sparsity

**What goes wrong:** Two series that should sum to zero instead produce a BTreeMap with entries mapping to QRat::zero(). These phantom zero entries make the series appear non-sparse, waste memory, and cause wrong results in `is_zero()` checks and `num_nonzero()` counts.
**Why it happens:** After `entry += value`, the entry might be zero but is not removed from the map.
**How to avoid:** After every operation that modifies coefficients, either (a) check if the result is zero and remove it immediately, or (b) do a bulk `retain(|_, v| !v.is_zero())` pass at the end. Option (a) is cleaner for addition; option (b) is necessary for multiplication where intermediate accumulation happens.
**Warning signs:** `num_nonzero()` returns a positive number for the zero series. Series display shows "+ 0*q^5" terms.

### Pitfall 6: Simplification Invalidating ExprRef Cache

**What goes wrong:** You cache simplification results in a `HashMap<ExprRef, ExprRef>`. But the simplification of an expression depends on the arena state (which expressions exist). If the arena grows (new expressions interned during simplification), previously cached results may reference stale ExprRef values -- or worse, the same ExprRef now points to a different expression because the arena was rebuilt.
**Why it happens:** ExprArena is append-only, so ExprRef values are stable. But if you implement arena compaction/GC (not planned for Phase 2), this breaks.
**How to avoid:** Since ExprArena is append-only, ExprRef values are stable for the lifetime of the arena. Caching `HashMap<ExprRef, ExprRef>` is safe as long as the arena is never compacted. Document this invariant. Clear the cache if the arena is ever rebuilt (future concern, not Phase 2).
**Warning signs:** None in Phase 2 (arena is append-only). Would manifest as wrong simplification results if arena compaction is ever added.

## Code Examples

Verified patterns from official sources and domain knowledge:

### Series Addition (Truncated)

```rust
/// Add two formal power series, truncating to min precision.
/// Time: O(|a| + |b|), Space: O(|a| + |b|)
pub fn add(a: &FormalPowerSeries, b: &FormalPowerSeries) -> FormalPowerSeries {
    assert_eq!(a.variable, b.variable);
    let trunc = a.truncation_order.min(b.truncation_order);
    let mut result = FormalPowerSeries::zero(a.variable, trunc);

    // Merge coefficients from both series
    for (&k, v) in &a.coefficients {
        if k < trunc {
            result.set_coeff(k, v.clone());
        }
    }
    for (&k, v) in &b.coefficients {
        if k < trunc {
            let existing = result.coeff(k);
            let sum = &existing + v;
            result.set_coeff(k, sum);
        }
    }
    result
}
```

### Scalar Multiplication

```rust
/// Multiply a formal power series by a scalar (QRat).
pub fn scalar_mul(s: &QRat, a: &FormalPowerSeries) -> FormalPowerSeries {
    if s.is_zero() {
        return FormalPowerSeries::zero(a.variable, a.truncation_order);
    }
    let mut result = FormalPowerSeries::zero(a.variable, a.truncation_order);
    for (&k, v) in &a.coefficients {
        let product = s * v;
        if !product.is_zero() {
            result.coefficients.insert(k, product);
        }
    }
    result
}
```

### Negation

```rust
/// Negate a formal power series: -f(q)
pub fn negate(a: &FormalPowerSeries) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(a.variable, a.truncation_order);
    for (&k, v) in &a.coefficients {
        result.coefficients.insert(k, -v.clone());
    }
    result
}
```

### Coefficient Extraction

```rust
/// Extract coefficient of q^k from a formal power series.
/// Returns QRat::zero() if the coefficient is zero.
/// Panics if k >= truncation_order (coefficient unknown).
pub fn extract_coeff(fps: &FormalPowerSeries, k: i64) -> QRat {
    fps.coeff(k)
}
```

### Bottom-Up Expression Traversal on Arena DAG

```rust
/// Recursively simplify children of an expression, returning a new ExprRef
/// with simplified children. Uses hash-consing to detect unchanged results.
fn simplify_children(
    phase: &SimplificationPhase,
    expr: ExprRef,
    arena: &mut ExprArena,
) -> ExprRef {
    match arena.get(expr).clone() {
        Expr::Add(children) => {
            let new_children: Vec<ExprRef> = children.iter()
                .map(|&c| bottom_up(phase, c, arena))
                .collect();
            if new_children == children {
                expr // no change, return original ExprRef
            } else {
                // Re-canonicalize (sort children) and intern
                make_add(arena, new_children)
            }
        }
        Expr::Mul(children) => {
            let new_children: Vec<ExprRef> = children.iter()
                .map(|&c| bottom_up(phase, c, arena))
                .collect();
            if new_children == children {
                expr
            } else {
                make_mul(arena, new_children)
            }
        }
        Expr::Neg(child) => {
            let new_child = bottom_up(phase, child, arena);
            if new_child == child {
                expr
            } else {
                make_neg(arena, new_child)
            }
        }
        Expr::Pow(base, exp) => {
            let new_base = bottom_up(phase, base, arena);
            let new_exp = bottom_up(phase, exp, arena);
            if new_base == base && new_exp == exp {
                expr
            } else {
                make_pow(arena, new_base, new_exp)
            }
        }
        // Atoms and q-specific nodes: no children to simplify (for Phase 2)
        _ => expr,
    }
}
```

### Test: Euler Function via Pentagonal Number Theorem

```rust
/// Verify that prod_{k=1}^{N} (1-q^k) matches the pentagonal number theorem:
/// (q;q)_inf = sum_{k=-inf}^{inf} (-1)^k * q^{k(3k-1)/2}
///
/// The nonzero coefficients are:
///   q^0: +1, q^1: -1, q^2: -1, q^5: +1, q^7: +1,
///   q^12: -1, q^15: -1, q^22: +1, q^26: +1, ...
#[test]
fn euler_function_pentagonal() {
    // ... set up arena, q_var, truncation_order = 30 ...
    let euler = euler_function_generator(q_var, 30);
    euler.ensure_order(30);
    let fps = euler.series();

    // Verify against known pentagonal number expansion
    assert_eq!(fps.coeff(0), QRat::one());
    assert_eq!(fps.coeff(1), -QRat::one());
    assert_eq!(fps.coeff(2), -QRat::one());
    assert_eq!(fps.coeff(3), QRat::zero());
    assert_eq!(fps.coeff(4), QRat::zero());
    assert_eq!(fps.coeff(5), QRat::one());
    assert_eq!(fps.coeff(6), QRat::zero());
    assert_eq!(fps.coeff(7), QRat::one());
    // ... continue for all generalized pentagonal numbers up to 30
}
```

### Test: Partition Function Recurrence

```rust
/// Verify p(n) = 1/(q;q)_inf matches OEIS A000041.
///
/// Known values: p(0)=1, p(1)=1, p(2)=2, p(3)=3, p(4)=5, p(5)=7,
///   p(6)=11, p(7)=15, p(8)=22, p(9)=30, p(10)=42
#[test]
fn partition_generating_function() {
    // Compute 1/(q;q)_inf by inverting the Euler function
    // Or: directly compute via the recurrence
    //   p(n) = p(n-1) + p(n-2) - p(n-5) - p(n-7) + p(n-12) + ...
    let expected = vec![1, 1, 2, 3, 5, 7, 11, 15, 22, 30, 42];
    for (n, &p_n) in expected.iter().enumerate() {
        assert_eq!(partition_count(n as i64), p_n);
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Global hash-consing with `Arc` (hashconsing crate) | Index-based arena hash-consing (ExprArena with Vec + FxHashMap) | Already in Phase 1 | No lifetime issues, no Arc overhead, O(1) equality via u32 comparison |
| egg/equality saturation for all simplification | Phased rewriting for routine simplification, egg reserved for deep equivalence | Prior research decision | Predictable, fast simplification for common cases; egg's overhead only paid when needed |
| Dense Vec for power series coefficients | Sparse BTreeMap<i64, QRat> | Standard for q-series | Memory proportional to nonzero terms, not maximum exponent |
| Post-hoc truncation of series multiplication | Truncation during multiplication (break inner loop) | Standard optimization | Prevents O(N^2) memory blowup |

**Deprecated/outdated:**
- The `series` crate's dense Vec representation is unsuitable for sparse q-series. Do not use it.
- The `hashconsing` crate's `Arc`-based approach was considered in earlier research but rejected in favor of our index-based arena.
- The `term_rewriting` crate is too low-level and lacks arena/hash-consing awareness.

## Open Questions

1. **Series Division / Multiplicative Inverse**
   - What we know: 1/f(q) can be computed by the recurrence c[n] = (-1/f[0]) * sum_{k=1}^{n} f[k]*c[n-k], provided f[0] != 0. This is O(N^2) for N terms.
   - What's unclear: Should we implement division as a primitive, or compose it from multiplication + inversion? For Phase 2, inversion is needed for partition function (1/(q;q)_inf). The recurrence approach is well-known but needs careful implementation with truncation tracking.
   - Recommendation: Implement as a separate `invert()` function using the recurrence. Compose division as `a * invert(b)`. This is simpler and each piece is independently testable.

2. **Pattern Matching Expressiveness for N-ary Operators**
   - What we know: Simple patterns (match specific children) work for binary/fixed-arity operators. N-ary Add/Mul require subset matching or "rest" patterns.
   - What's unclear: How much pattern matching infrastructure is needed for Phase 2? The 6 simplification phases primarily need arithmetic normalization (combine numeric terms, flatten nested Add/Mul, eliminate identity elements), not deep structural matching.
   - Recommendation: For Phase 2, implement rules as Rust functions that operate directly on the Expr enum (match on `Expr::Add(children)` and iterate), not through a generic pattern matching engine. Save the generic pattern matcher for Phase 3+ when domain-specific rules (q-Pochhammer algebra) need it.

3. **When to Trigger Simplification**
   - What we know: Two strategies: (a) simplify eagerly on every `intern()` call, (b) simplify lazily on explicit `simplify()` calls.
   - What's unclear: Eager simplification ensures canonical form at all times but slows construction. Lazy simplification is faster for construction but means unsimplified expressions accumulate.
   - Recommendation: Keep construction fast (no simplification during `intern()`). Provide explicit `simplify(expr, arena) -> ExprRef` that users/downstream phases call when needed. This matches how Maple and SymPy work -- construction is cheap, simplification is opt-in.

4. **FPS Variable Identity**
   - What we know: Each FPS has a `variable: SymbolId`. Operations on series in different variables should error.
   - What's unclear: Should we support multivariate FPS (series in q with coefficients that are series in z)? Phase 2 scope is univariate.
   - Recommendation: Phase 2 is strictly univariate. Assert `a.variable == b.variable` on every binary operation. Multivariate support is a Phase 3+ concern (needed for bilateral series).

## Simplification Phases Detail

The 6-phase simplification strategy adapted for our specific Expr enum and Phase 2 scope:

### Phase 1: Normalize (arithmetic flattening)
- Flatten nested Add: `Add([a, Add([b, c])])` -> `Add([a, b, c])`
- Flatten nested Mul: `Mul([a, Mul([b, c])])` -> `Mul([a, b, c])`
- Combine numeric constants in Add: `Add([3, 5, x])` -> `Add([8, x])`
- Combine numeric constants in Mul: `Mul([3, 5, x])` -> `Mul([15, x])`
- Re-canonicalize children order after modifications

### Phase 2: Cancel (identity/annihilator elimination)
- `Add([..., 0, ...])` -> remove the zero
- `Mul([..., 1, ...])` -> remove the one
- `Mul([..., 0, ...])` -> `Integer(0)`
- `Pow(x, 0)` -> `Integer(1)` (for x != 0, x != Undefined)
- `Pow(x, 1)` -> `x`
- `Pow(1, n)` -> `Integer(1)`
- `Neg(Integer(0))` -> `Integer(0)`

### Phase 3: Collect (combine like terms)
- `Add([..., x, ..., x, ...])` -> `Add([..., Mul([2, x]), ...])`
- `Add([..., Mul([3, x]), ..., Mul([5, x]), ...])` -> `Add([..., Mul([8, x]), ...])`
- `Mul([..., x, ..., x, ...])` -> `Mul([..., Pow(x, 2), ...])`

### Phase 4: Simplify (algebraic simplifications)
- `Neg(Neg(x))` -> `x`
- `Neg(Integer(n))` -> `Integer(-n)`
- `Neg(Rational(r))` -> `Rational(-r)`
- `Pow(Pow(x, a), b)` -> `Pow(x, Mul([a, b]))` (when a, b are integers)
- `Mul([..., Neg(x), ...])` -> `Neg(Mul([..., x, ...]))` (bubble negation out)

### Phase 5: Expand (controlled expansion -- off by default in Phase 2)
- Reserved for future: distribute Mul over Add, expand Pow(Add, n)
- Not implemented in Phase 2 (keep expressions factored)

### Phase 6: Verify (fixpoint check)
- Compare result with input: if unchanged, simplification is complete
- Log a warning if max iterations reached (indicates possible non-termination)

## Test Data for Verification

### Euler Function (q;q)_inf Coefficients (OEIS A010815)

Exponents with nonzero coefficients and their values:
```
q^0:  +1
q^1:  -1
q^2:  -1
q^5:  +1
q^7:  +1
q^12: -1
q^15: -1
q^22: +1
q^26: +1
q^35: -1
q^40: -1
q^51: +1
q^57: +1
```
All other coefficients (q^3, q^4, q^6, q^8, ...) are 0.

Generalized pentagonal numbers: k(3k-1)/2 for k = 0, 1, -1, 2, -2, 3, -3, ...
= 0, 1, 2, 5, 7, 12, 15, 22, 26, 35, 40, 51, 57, ...

Signs: +, -, -, +, +, -, -, +, +, -, -, +, +, ... (pairs of same sign, alternating)

### Partition Function p(n) (OEIS A000041)

```
p(0)=1, p(1)=1, p(2)=2, p(3)=3, p(4)=5, p(5)=7,
p(6)=11, p(7)=15, p(8)=22, p(9)=30, p(10)=42,
p(11)=56, p(12)=77, p(13)=101, p(14)=135, p(15)=176,
p(16)=231, p(17)=297, p(18)=385, p(19)=490, p(20)=627
```

These are the coefficients of 1/(q;q)_inf (the inverse of the Euler function).

### Simple Test: (1-q) * (1+q) = 1-q^2

```
a = 1 - q         (coefficients: {0: 1, 1: -1})
b = 1 + q         (coefficients: {0: 1, 1: 1})
a * b = 1 - q^2   (coefficients: {0: 1, 2: -1})
```

### Simple Test: (1-q)^2 = 1 - 2q + q^2

```
a = 1 - q         (coefficients: {0: 1, 1: -1})
a * a = 1 - 2q + q^2  (coefficients: {0: 1, 1: -2, 2: 1})
```

## Sources

### Primary (HIGH confidence)
- [BTreeMap documentation](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) -- Rust stdlib, ordered map semantics and performance
- [OEIS A000041](https://oeis.org/A000041) -- Partition function values, verified
- [OEIS A010815](https://oeis.org/A010815) -- Euler function coefficients (pentagonal number theorem)
- [Pentagonal Number Theorem - Wikipedia](https://en.wikipedia.org/wiki/Pentagonal_number_theorem) -- Mathematical definitions and expansion
- [Jacobi Triple Product - Wikipedia](https://en.wikipedia.org/wiki/Jacobi_triple_product) -- Identity verification test data
- [egg documentation](https://egraphs-good.github.io/egg/egg/) -- Language trait, define_language! limitations, RecExpr representation
- [series crate docs](https://docs.rs/series/latest/series/) -- Dense Vec representation confirmed unsuitable

### Secondary (MEDIUM confidence)
- [term_rewriting crate](https://docs.rs/term_rewriting) -- First-order TRS in Rust, evaluated and rejected for our use case
- [E-Graphs in Rust - Stephen Diehl](https://www.stephendiehl.com/posts/egraphs/) -- Architecture patterns for e-graph integration
- [Practical recursion schemes in Rust - Tweag](https://www.tweag.io/blog/2025-04-10-rust-recursion-schemes/) -- Bottom-up traversal patterns
- [Graph & Tree Traversals in Rust - Sachan Ganesh](https://sachanganesh.com/programming/graph-tree-traversals-in-rust/) -- Arena-based traversal patterns
- [Garvan q-product tutorial](https://arxiv.org/abs/math/9812092) -- Andrews' algorithm, series-product conversion

### Tertiary (LOW confidence)
- Series division recurrence algorithm -- Based on standard textbook knowledge (Knuth TAOCP), not verified against specific Rust implementation. Implementation details may need adjustment during coding.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- No new dependencies; all Phase 1 libraries sufficient
- Architecture (simplification engine): HIGH -- Phased rewriting is textbook TRS; arena-based bottom-up traversal is well-documented in Rust compiler internals
- Architecture (FPS engine): HIGH -- BTreeMap sparse representation is standard; truncated multiplication algorithm is well-known
- Pitfalls: HIGH -- Expression swell, rewrite non-termination, and truncation tracking are well-documented CAS problems from prior research
- Code examples: MEDIUM -- Patterns based on established algorithms but not extracted from a specific verified Rust implementation; will need refinement during coding
- Test data: HIGH -- Euler function and partition values verified against OEIS

**Research date:** 2026-02-13
**Valid until:** 2026-03-15 (stable domain; no fast-moving dependencies)
