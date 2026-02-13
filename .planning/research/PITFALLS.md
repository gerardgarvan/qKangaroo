# Pitfalls Research: Q-Symbolic Computation Engine

**Domain:** Symbolic computation engine for q-series (Rust core + Python API replacing Garvan's Maple packages)
**Researched:** 2026-02-13
**Confidence:** MEDIUM-HIGH (strong on CAS architecture and Rust/PyO3 pitfalls from official docs; moderate on q-series-specific numerics from domain literature; some claims from training data flagged)

---

## Critical Pitfalls

Mistakes that cause rewrites, multi-week delays, or architectural dead ends.

---

### Pitfall 1: Choosing the Wrong Expression Normalization Strategy

**Severity:** PROJECT-KILLER

**What goes wrong:**
You pick a canonical form for expressions early (e.g., always expand to sum-of-products, or always factor) and build your simplification engine around it. Later you discover that q-series identities are most naturally expressed in product form (eta-quotients, Jacobi products), while verification requires expanded series form, and user-facing output needs yet another form. Your canonical form forces constant expensive conversions, or worse, destroys structure that downstream algorithms need. For example, auto-expanding `(q;q)_inf` into its series representation loses the product structure that pattern-matching for eta-quotient identities requires.

**Why it happens:**
General CAS design advice says "pick a canonical form and normalize everything." This works for polynomials over Q but fails for q-series where the same object has fundamentally different useful representations (infinite product, truncated series, eta-quotient, theta function form). Fredrik Johansson's critique of existing CAS systems identifies this as a core design flaw: "most existing computer algebra systems force the user to return to a 'canonical' expression at every stage."

**How to avoid:**
- Use a **multi-representation architecture** where expressions carry their "natural" form plus lazy converters to other forms.
- The internal IR should be a DAG of q-series building blocks (q-Pochhammer symbols, eta functions, theta functions) that can be *projected* into expanded series form on demand, not stored that way.
- Make the canonical form decision per-algorithm, not per-system. Pattern matching operates on product form; coefficient extraction operates on series form; display uses whatever the user specified.
- Garvan's package uses this approach: `prodmake` converts series to products, `jac2series` converts products to series. These are explicit user-invoked conversions, not automatic canonicalization.

**Warning signs:**
- You find yourself writing `to_series()` calls everywhere before doing anything useful.
- Two mathematically identical expressions compare as unequal because they are in different normal forms.
- Performance profiling shows most time in conversion functions, not actual computation.
- Users complain that the system "rearranges" their expressions in unhelpful ways.

**Phase to address:** Phase 1 (Core IR design). This is a foundational decision. Getting it wrong means rewriting the entire expression layer.

**Confidence:** HIGH -- corroborated by multiple CAS design critiques (Johansson, SymPy docs, Cadabra design philosophy) and Garvan's own package design.

---

### Pitfall 2: Infinite Loops and Non-Termination in the Simplification/Rewriting Engine

**Severity:** PROJECT-KILLER

**What goes wrong:**
Your rewrite rules form cycles. Rule A rewrites `f(x)` to `g(x)`, rule B rewrites `g(x)` back to `f(x)`. Or worse: equality saturation with bidirectional rules causes the e-graph to grow without bound. Even a terminating, confluent term rewriting system can cause non-termination in equality saturation -- a rule set like `h(f(x), y) -> h(x, g(y))`, `h(x, g(y)) -> h(x, y)`, `f(x) -> x` generates infinitely many inequivalent terms `g^n(b)` that cannot be merged.

In q-series specifically, identities like the Jacobi triple product or Euler's pentagonal number theorem create natural bidirectional rewrite opportunities that are particularly prone to this.

**Why it happens:**
- Treating mathematical identities as bidirectional rewrite rules without termination analysis.
- Adding new rules to an existing rule set without checking confluence/termination of the combined set.
- Confusing "simplification" (reduce complexity) with "transformation" (rewrite to equivalent form). The former should always terminate; the latter may not.
- Overreliance on timeouts as a termination mechanism rather than structural guarantees.

**How to avoid:**
- Separate the rule system into **directed simplification rules** (always reduce a well-defined complexity measure, guaranteed terminating) and **transformation rules** (bidirectional, user-invoked, bounded by iteration limits).
- For simplification: define a strict total order on expression complexity (term count, depth, or a weighted measure) and only allow rules that decrease this measure. This is the standard approach from term rewriting theory -- termination via a well-founded ordering.
- For exploration/identity discovery: use **depth-bounded equality saturation** (only apply rules when created e-classes don't exceed depth N) or **merge-only saturation** (only apply rules when both sides already exist in the e-graph).
- Implement a hard iteration counter on any rewrite loop. Log when it triggers -- this is a bug signal, not a feature.
- Start with a small, manually-verified rule set. Add rules one at a time with regression tests.

**Warning signs:**
- Tests that hang or time out intermittently.
- Memory usage growing unboundedly during "simplification."
- The same expression appearing in debug traces of rewrite steps.
- Adding a new identity rule breaks previously-working simplifications.

**Phase to address:** Phase 2-3 (Simplification engine design). Must be architecturally prevented, not just patched with timeouts.

**Confidence:** HIGH -- term rewriting termination and confluence are well-studied (Berkeley CS294-260 lecture notes, academic TRS literature). The e-graph non-termination example is from the equality saturation literature.

---

### Pitfall 3: Expression Swell Destroying Performance

**Severity:** PROJECT-KILLER

**What goes wrong:**
Intermediate expressions during computation grow exponentially even when the final result is small. Computing the q-expansion of a product of 10 eta functions to 1000 terms might generate intermediate polynomials with millions of terms that mostly cancel. Multiplying two truncated q-series of length N produces an intermediate result of length N^2 before truncation. Memory usage spikes, GC thrashes, and a computation that should take milliseconds takes minutes or OOMs.

This is THE central performance problem of symbolic computation, identified across every major CAS implementation from Macsyma to SymPy.

**Why it happens:**
- Naive polynomial multiplication without early truncation.
- Expanding products before collecting terms.
- Not exploiting the sparse structure of q-series (most coefficients are zero in typical q-series).
- Hash consing without normalization -- structurally different but equivalent sub-expressions are not shared.

**How to avoid:**
- **Truncated arithmetic everywhere.** Never compute more terms than requested. When multiplying two series truncated to O(q^N), truncate the product to O(q^N) during multiplication, not after. This is O(N^2) instead of O(N^2) memory but with much smaller constants.
- **Lazy series representation.** Represent q-series as iterators/generators that produce coefficients on demand rather than materializing the entire series. This is particularly natural for infinite products like `(q;q)_inf`.
- **Sparse representation for q-series.** Most q-series arising from eta-quotients and partition functions have sparse coefficient arrays. Use a sorted vec of (exponent, coefficient) pairs rather than dense arrays.
- **Monitor intermediate expression size** with hard limits. If an intermediate exceeds 10x the expected output size, something is wrong algorithmically.

**Warning signs:**
- Memory usage grows much faster than the precision parameter.
- Computation time is super-linear in the number of requested terms.
- The system is fast for 100 terms but unusable at 10,000 terms.
- Profiling shows most time in memory allocation, not arithmetic.

**Phase to address:** Phase 1-2 (Core series arithmetic). The truncated arithmetic must be baked into the series multiplication primitives from day one.

**Confidence:** HIGH -- expression swell is universally documented across CAS literature (Wikipedia "Computer algebra" article, SymPy performance docs, hash consing paper).

---

### Pitfall 4: Hash Consing Without Proper Weak References in Rust

**Severity:** CRITICAL (causes memory leaks that worsen over time)

**What goes wrong:**
You implement hash consing (structural sharing of equivalent sub-expressions via an intern table) using strong references in the intern table. Every expression ever created stays alive forever because the intern table holds a reference to it. Memory grows monotonically. In long-running sessions (which researchers will have -- exploring identities for hours), this becomes a memory leak that eventually exhausts RAM.

In Rust specifically, the standard approach of using `Weak<T>` references in the intern table requires careful integration with `Arc<T>` and has subtle correctness issues around resurrection (a weakly-referenced expression gets dropped, then an identical expression is created -- the weak reference is now dangling, and you must detect this and create a fresh entry).

**Why it happens:**
- Rust's ownership model makes weak reference patterns more explicit than in GC'd languages, but also more error-prone.
- The hash consing literature assumes garbage-collected languages where weak hash tables are a standard library feature.
- It's easy to prototype with `HashMap<ExprHash, Arc<Expr>>` (strong refs) and "fix it later," but the memory leak is insidious -- it only manifests under sustained use.

**How to avoid:**
- Use an **arena-based approach** instead of global hash consing. Allocate expressions in an arena (using `bumpalo` or `typed-arena`). When a computation is complete, drop the entire arena. This sidesteps the weak reference problem entirely for batch computations.
- If you need cross-computation sharing (e.g., a persistent identity database), use a **two-tier system**: short-lived computation arenas that are dropped after each operation, and a long-lived "knowledge base" with explicit insertion (not automatic interning).
- If you must use hash consing, use `HashMap<ExprHash, Weak<Expr>>` and periodically sweep dead entries. The `elsa` or `internment` crates provide some patterns, but audit them carefully.
- Benchmark memory usage over sustained sessions (1000+ operations), not just single computations.

**Warning signs:**
- RSS (resident set size) grows monotonically during a session, even when computing small results.
- `HashMap` intern table size grows without bound.
- Profiling shows increasing time in hash table lookups as the table grows.

**Phase to address:** Phase 1 (Memory architecture). Arena vs. hash consing is a foundational choice that affects every expression-handling API.

**Confidence:** HIGH -- arena allocation is well-documented in Rust (bumpalo, typed-arena official docs); hash consing memory issues are documented in the academic literature and Wikipedia.

---

### Pitfall 5: Incorrect q-Pochhammer Edge Cases

**Severity:** CRITICAL (produces silently wrong mathematical results)

**What goes wrong:**
The q-Pochhammer symbol `(a;q)_n` has several edge cases that are easy to get wrong:

1. **Negative n:** `(a;q)_{-n} = 1 / (aq^{-n}; q)_n = 1 / prod_{k=0}^{n-1} (1 - aq^{k-n})`. Getting the index shift wrong produces silently incorrect results.

2. **n = 0:** `(a;q)_0 = 1` (empty product). Forgetting this base case causes division by zero or NaN propagation.

3. **n = infinity:** `(a;q)_inf = prod_{k=0}^{inf} (1 - aq^k)`. This converges only for |q| < 1. But in symbolic computation, you are working with formal power series where q is an indeterminate, not a number. Confusing the analytic convergence condition with the algebraic (formal power series) setting causes either (a) unnecessary convergence checks that reject valid symbolic computations, or (b) missing convergence checks when doing numerical evaluation.

4. **Multi-parameter products:** `(a1, a2, ..., am; q)_n` is shorthand for the product of individual q-Pochhammer symbols. Implementing this as a single loop instead of a product of individual symbols introduces coupling bugs.

5. **The relationship `(a;q)_n = (a;q)_inf / (aq^n; q)_inf`** is valid for computing finite products from infinite ones, but introduces division by zero when `aq^n` is a power of q that makes a factor vanish.

**Why it happens:**
- Copying formulas from references without carefully tracking index conventions (some references use 0-indexed, others 1-indexed products).
- Not distinguishing between formal power series operations and numerical evaluation.
- Testing only with "nice" inputs (positive integer n, |q| < 1, generic a).

**How to avoid:**
- Implement a comprehensive test suite against Maple/Mathematica for `(a;q)_n` with n in {-5, -1, 0, 1, 2, 5, inf} and a in {0, 1, q, q^2, generic symbol}.
- Clearly separate the **formal power series** code path (q is an indeterminate, no convergence concerns, truncate to O(q^N)) from the **numerical evaluation** code path (q is a complex number, convergence matters).
- Use Garvan's package output as the ground truth oracle for every implemented function.
- Document index conventions explicitly in code comments. Pick ONE convention and convert at API boundaries.

**Warning signs:**
- Numerical evaluation of known identities (like Euler's pentagonal number theorem) gives wrong coefficients for certain terms.
- Expressions involving `(q;q)_{-n}` produce panics or unexpected results.
- Your q-binomial coefficient implementation disagrees with Mathematica for edge-case arguments.

**Phase to address:** Phase 2 (Core q-series functions). Every function must have an edge-case test suite before moving to the next function.

**Confidence:** HIGH -- the mathematical definitions are unambiguous (Wikipedia, Wolfram MathWorld, Garvan's papers). The implementation edge cases are well-known in the community.

---

### Pitfall 6: PyO3 GIL Ref Memory Accumulation Across the FFI Boundary

**Severity:** CRITICAL (causes Python-side memory leaks and potential crashes)

**What goes wrong:**
When using PyO3's older GIL Refs API (`&'py PyAny`), Python objects created in a loop don't get freed when the Rust variable goes out of scope -- they persist until the enclosing `GILPool` exits. If a Python user calls a Rust function that internally loops over expressions (e.g., computing 10,000 series coefficients and returning them as a Python list), all intermediate Python objects stay alive for the entire call. This can cause memory to balloon by 10-100x.

Even with the newer `Bound<T>` API, there are subtleties: dropping a `Py<T>` outside a GIL context doesn't immediately deallocate -- it queues the deallocation for the next GIL acquisition, creating delayed cleanup patterns.

**Why it happens:**
- Rust developers naturally expect RAII (drop = dealloc). PyO3's GIL-managed memory doesn't follow this model.
- The GIL Refs API has been deprecated in favor of the Bound API (as of PyO3 0.21+), but many tutorials and examples still use the old API.
- Data marshaling between Rust's strict type system and Python's dynamic types is error-prone (23% serialization failure rate with mixed-type dictionaries, per one study).

**How to avoid:**
- **Use ONLY the Bound<T> API** (not the deprecated GIL Refs). This provides immediate reference counting on drop, eliminating pool-lifetime issues. This is the officially recommended path as of PyO3 0.21+.
- Minimize Python object creation on the Rust side. Return raw Rust types and let PyO3's conversion traits handle the Python object creation at the boundary, not deep in computation loops.
- For bulk data transfer (like coefficient arrays), use **zero-copy mechanisms**: return a NumPy array view into Rust-allocated memory rather than creating N individual Python integers.
- Profile memory under realistic Python usage patterns (Jupyter notebook sessions, loops calling Rust functions thousands of times).
- Pin PyO3 version and read the migration guide for that specific version. The API is evolving rapidly.

**Warning signs:**
- Python process memory grows when calling Rust functions in a loop, even for small results.
- Mysterious segfaults in Python that don't reproduce in pure-Rust tests.
- GIL contention causing Python threads to block unexpectedly.

**Phase to address:** Phase 4-5 (Python bindings). But the Rust API must be designed from Phase 1 to support efficient FFI (return owned values, avoid lifetimes that can't cross the boundary).

**Confidence:** HIGH -- PyO3 official memory management docs explicitly document these issues and the Bound API migration path.

---

## Moderate Pitfalls

Mistakes that cause multi-day delays, significant refactoring, or correctness issues in specific areas.

---

### Pitfall 7: Building a General CAS Instead of a Focused q-Series Tool

**Severity:** HIGH (scope creep kills momentum and delays usable output)

**What goes wrong:**
You start implementing general polynomial arithmetic, then general rational function simplification, then general pattern matching, then a general type system for mathematical objects... and six months later you have a half-built general CAS with no q-series functionality. Meanwhile, Garvan's students are still using Maple because your tool can't compute a single q-Pochhammer symbol.

The research literature is clear: "Most general computer algebra systems have difficulty computing with the polynomials and matrices that arise in actual research, as real problems tend to produce large polynomials and large matrices that the general CA systems cannot handle." Focused tools outperform general ones in their specific domain.

**Why it happens:**
- "We'll need this eventually" thinking. Every q-series function needs polynomial arithmetic, so you build a general polynomial library first. But q-series polynomials have specific structure (sparse, integer coefficients, single variable q) that a general library handles suboptimally.
- Perfectionism. "The architecture should be clean and general" leads to over-abstraction.
- Not having a concrete validation target. Without "match Garvan's output for function X," there's no definition of done.

**How to avoid:**
- **Function-by-function parity with Garvan's package as the Phase 2-3 milestone.** Literally: implement `qpochhammer`, test against Maple output, move to `qbinomial`, test against Maple output, etc. Every function has a concrete "done" definition.
- Only build general infrastructure when TWO OR MORE q-series functions need it. One function is not a pattern; two is.
- Keep a "NOT BUILDING" list: general equation solving, general calculus, general linear algebra, visualization, notebooks. These are explicitly out of scope.
- Time-box infrastructure work. If building a general polynomial library takes more than 1 week, you're over-engineering. q-series polynomials are univariate in q with integer/rational coefficients -- that's a much simpler problem than general multivariate polynomials.

**Warning signs:**
- You've been working for a month and can't compute `(q;q)_10` yet.
- Your type hierarchy has more than 3 levels of abstraction.
- You're debating whether to support multivariate polynomials "for future flexibility."
- The README describes features that aren't q-series-related.

**Phase to address:** Every phase. This is a constant discipline, not a one-time decision. The roadmap itself is the primary defense.

**Confidence:** HIGH -- this is a universal software engineering pitfall, but particularly well-documented in the CAS domain where the history of failed "general-purpose" projects is long.

---

### Pitfall 8: Testing Against One Oracle Only (and Trusting It Blindly)

**Severity:** HIGH (produces false confidence in correctness)

**What goes wrong:**
You test every function against Maple output, and when your result matches Maple, you mark the test as passing. Then a researcher reports that a well-known identity doesn't hold in your system. Investigation reveals that Maple itself had a bug for that particular edge case, and your system faithfully reproduced it.

A study converting NIST Digital Library of Mathematical Functions to Maple and Mathematica found that only 26.7% of expressions were successfully numerically verified by Maple, and 22.6% by Mathematica. Both systems had bugs, timeouts, and errors on significant fractions of standard mathematical identities.

**Why it happens:**
- Treating the reference implementation as ground truth rather than as a helpful-but-fallible oracle.
- Only testing with forward computation (compute, compare) rather than testing mathematical properties (identities, functional equations, special values).

**How to avoid:**
- **Three-oracle strategy:** Test against (1) Garvan's Maple output (primary reference), (2) Mathematica/Wolfram Alpha (independent CAS), and (3) known mathematical identities (the actual ground truth).
- **Property-based testing** for mathematical invariants:
  - `(a;q)_m * (aq^m;q)_n = (a;q)_{m+n}` for all valid m, n.
  - Euler's identity: `sum_{n=0}^{inf} p(n)q^n = prod_{k=1}^{inf} 1/(1-q^k)` to N terms.
  - The Jacobi triple product identity.
  - Symmetry properties of q-binomial coefficients.
- **Numerical cross-validation:** For any symbolic identity, also evaluate both sides numerically at random q values (|q| < 0.9) and check agreement to high precision. Disagreement means either the identity is wrong or the numerical evaluation has bugs.
- **Coefficient-level testing:** For q-series, compare individual coefficients (which are exact integers) rather than truncated series expressions. Integer comparison has no precision issues.

**Warning signs:**
- All your tests pass but a researcher finds a wrong result.
- You have zero tests that check mathematical properties (only input/output comparison tests).
- Your test suite doesn't test any identity that Maple is known to have issues with.

**Phase to address:** Phase 2-3 (alongside function implementation). The test harness is as important as the code it tests.

**Confidence:** HIGH -- the NIST/CAS verification study is published research. Property-based testing is well-established methodology.

---

### Pitfall 9: Over-Engineering the Expression IR Before Understanding the Domain

**Severity:** HIGH (wastes months on abstractions that don't fit)

**What goes wrong:**
You design an elaborate expression IR with a type system for rings, fields, modules, categories, etc. -- then discover that q-series computation mostly operates on:
- Formal power series in one variable (q) with integer or rational coefficients.
- Products of q-Pochhammer symbols with various parameters.
- Eta-quotients (specific products of Dedekind eta functions).

Your general IR has 50 node types when you need 8. The abstraction overhead (trait objects, dynamic dispatch, runtime type checks) costs 10x performance compared to a flat enum with the right variants.

**Why it happens:**
- Reading about how Mathematica, Sage, or SymPy design their expression types and trying to replicate that generality.
- Designing the IR "top-down" from mathematical category theory instead of "bottom-up" from actual q-series computations.
- Benchmarks for Rust show enum dispatch is ~12x faster than `Box<dyn Trait>` dispatch. This matters enormously for expression-heavy computation.

**How to avoid:**
- Start with a **minimal flat enum** for the expression IR:
  ```rust
  enum Expr {
      Integer(BigInt),
      Rational(BigRational),
      Variable,          // q -- there's only one
      QPochhammer { a: Box<Expr>, n: Box<Expr> },
      Series { coeffs: Vec<(i64, BigInt)>, order: i64 },
      Product(Vec<Expr>),
      Sum(Vec<Expr>),
      Power { base: Box<Expr>, exp: Box<Expr> },
  }
  ```
- Add new variants ONLY when a concrete function needs them. After implementing 5-10 Garvan functions, you'll know what the IR actually needs.
- Use Rust enums, not trait objects, for the core IR. Enums give exhaustive match checking (the compiler tells you when you miss a case) and much better performance.
- Plan for ONE refactor of the IR after Phase 2 (when you know the domain). Budget 1-2 weeks for this. It's cheaper than getting it right the first time.

**Warning signs:**
- Your `Expr` enum has more than 15 variants before you've implemented 5 functions.
- You have trait hierarchies for mathematical structures (Ring, Field, etc.) that only have one implementing type.
- Expression construction requires more than 2-3 lines of code for common q-series objects.

**Phase to address:** Phase 1 (initial IR), with planned refactor in Phase 3 (after domain knowledge is established).

**Confidence:** HIGH -- Rust enum vs trait object performance is well-benchmarked. The "start minimal, grow from need" approach is standard for domain-specific compilers/languages.

---

### Pitfall 10: Numerical Stability When Cross-Validating Identities

**Severity:** MODERATE-HIGH (undermines the testing strategy)

**What goes wrong:**
You implement numerical cross-validation (evaluate both sides of a q-series identity at specific q values and compare). But for |q| close to 1, products like `(q;q)_inf` converge extremely slowly. For |q| close to 0, many terms are negligibly small and floating-point cancellation destroys precision. You get spurious test failures that erode trust in the test suite, or spurious passes that hide real bugs.

**Why it happens:**
- The q-Pochhammer infinite product `(a;q)_inf` is analytic in the open unit disk but has a natural boundary at |q| = 1. Near this boundary, convergence degrades dramatically.
- At q = roots of unity, the product may diverge or require special formulas (mock theta functions extend to certain points on the unit circle).
- Floating-point arithmetic with standard f64 precision (15-16 digits) is insufficient for cancellation-heavy expressions.

**How to avoid:**
- Use **arbitrary-precision floating point** (via `rug`/MPFR bindings, or the `arb` library for ball arithmetic) for numerical cross-validation, not f64. Compute to 50+ digits of precision.
- Evaluate at "well-conditioned" q values: |q| in [0.1, 0.5] is the sweet spot. Avoid |q| > 0.9 and |q| < 0.01.
- For cross-validation, prefer **exact integer coefficient comparison** over numerical evaluation whenever possible. The coefficient of q^k in a q-series is an exact integer -- compare those.
- When numerical comparison is necessary, use **interval arithmetic** (compute with error bounds) so you know whether disagreement is real or due to precision loss.

**Warning signs:**
- Numerical tests pass at q=0.5 but fail at q=0.99.
- The same test passes with 50 digits of precision but fails with 15 digits.
- You're constantly adjusting tolerance thresholds in numerical comparison tests.

**Phase to address:** Phase 2-3 (testing infrastructure).

**Confidence:** MEDIUM-HIGH -- convergence behavior of q-Pochhammer products is mathematically well-understood. Specific precision recommendations based on domain knowledge from training data (not independently verified for this exact use case).

---

### Pitfall 11: GMP/Bignum Allocation Overhead for Small Integers

**Severity:** MODERATE-HIGH (causes 10-100x slowdown for common operations)

**What goes wrong:**
q-series coefficients are often small integers (fitting in 64 bits) for the first few hundred terms, then occasionally become very large. If every coefficient is stored as a heap-allocated `BigInt` (which is what `num-bigint` does), you pay allocation overhead on every single arithmetic operation. Since q-series multiplication involves O(N^2) integer multiplications, this overhead dominates for moderate N.

Fredrik Johansson identifies this as a core CAS performance problem: "hardware, compilers, and programming languages are not designed for performant computation with [arbitrary-size integers]" and naive wrapping of GMP's `mpz_t` creates "massive amounts of overhead for small integers."

**Why it happens:**
- Using `num-bigint::BigInt` as the default integer type. It's pure Rust and safe, but allocates on the heap for every value, even small ones.
- Not benchmarking with representative workloads early. The first 100-term tests look fast; the 10,000-term benchmarks reveal the problem.

**How to avoid:**
- Use a **small-integer-optimized bignum** type. `malachite` avoids heap allocation for any integer less than 2^64, and only allocates for larger values. This matches the q-series workload perfectly (most coefficients are small, a few are huge).
- Alternatively, use `rug` (Rust bindings to GMP/MPFR) which uses GMP's internal small-integer optimization.
- Profile with realistic workloads: compute `(q;q)_inf` to 10,000 terms and measure. The coefficient of q^10000 in the partition function is a 100+ digit number, but the coefficient of q^5 is just 7.
- Consider a **hybrid representation**: store coefficients as `i64` until overflow, then promote to bignum. This is more implementation work but maximizes performance for the common case.

**Warning signs:**
- Profiling shows most time in `alloc` and `dealloc`, not in arithmetic.
- Series multiplication is slower than expected from the O(N^2) algorithm complexity.
- Memory usage is much higher than the raw coefficient data would require.

**Phase to address:** Phase 1 (choice of bignum library). Changing the integer type later requires touching every arithmetic operation.

**Confidence:** HIGH -- malachite docs explicitly describe the small-integer optimization; GMP/num-bigint performance comparisons are benchmarked on GitHub.

---

### Pitfall 12: Series Expansion Blowup in Partition and Modular Form Computations

**Severity:** MODERATE-HIGH (makes "interesting" computations infeasible)

**What goes wrong:**
Computing partition-related generating functions requires expanding products like `prod_{k=1}^{N} 1/(1-q^k)`. The naive approach computes this as a sequence of polynomial multiplications, each doubling the potential length. For N = 1000, the intermediate polynomial can have a million terms. Furthermore, the partition function p(n) grows exponentially -- p(1000) ~ 2.4 * 10^31, requiring arbitrary precision storage for each coefficient.

The Dedekind eta function and its quotients require computing q-expansions where Sturm's bound determines the minimum number of terms needed for uniqueness, and this bound can be large for high-level modular forms.

**Why it happens:**
- Not using the recurrence relation. Euler's pentagonal number theorem gives: `p(n) = p(n-1) + p(n-2) - p(n-5) - p(n-7) + ...` which computes p(n) in O(n^{3/2}) time and O(n) space, rather than expanding the full generating function.
- For eta-quotients, not exploiting the fact that the q-expansion coefficients satisfy recurrence relations derived from modular properties.
- Materializing the full coefficient array when only specific coefficients are needed.

**How to avoid:**
- Implement **recurrence-based algorithms** for partition functions and related sequences, not generating function expansion.
- For eta-quotients, use the multiplicative structure: compute the q-expansion of each eta factor separately, then multiply truncated series. This is more efficient than expanding the product formula directly.
- Implement **lazy coefficient computation**: provide an iterator interface that computes coefficients on demand, so computing p(1000) doesn't require storing p(0) through p(999) simultaneously.
- Set and enforce **precision bounds** at every API entry point. Every function should take an `order: usize` parameter that limits computation to O(q^order).

**Warning signs:**
- Computing partitions for n > 500 takes more than a second.
- Memory usage scales quadratically (or worse) with the precision parameter.
- The system OOMs when asked for 100,000 terms of a q-series.

**Phase to address:** Phase 3 (partition functions and modular forms). But the truncated series infrastructure from Phase 1-2 is a prerequisite.

**Confidence:** MEDIUM-HIGH -- the recurrence relation for p(n) is classical mathematics. Performance claims based on algorithmic complexity analysis and domain knowledge.

---

## Minor Pitfalls

Mistakes that cause days of debugging or minor usability issues.

---

### Pitfall 13: Rust Lifetime Annotations Preventing Clean Python API Design

**Severity:** MODERATE

**What goes wrong:**
Your Rust expression types use lifetimes (`Expr<'a>`) because they borrow from an arena. PyO3 cannot expose types with non-`'static` lifetimes to Python. You end up needing a separate "Python-safe" expression type that owns all its data, requiring expensive deep-copy conversions at every FFI boundary.

**How to avoid:**
- Design the Rust API with two tiers from the start: an internal tier using arena references for performance, and an external tier using `Arc<Expr>` (owned, reference-counted) for the Python boundary.
- The `Py<T>` type in PyO3 requires `T: 'static`. Plan for this from day one.
- Consider making the Python-facing types wrappers around indices into a Rust-side arena, rather than owning the expression data directly. This avoids deep copies while maintaining `'static` lifetime.

**Phase to address:** Phase 1 (API design), Phase 4-5 (Python bindings).

---

### Pitfall 14: Forgetting the Formal vs. Analytic Distinction

**Severity:** MODERATE

**What goes wrong:**
Your system conflates two different mathematical settings: (1) formal power series in q (algebraic, no convergence issues, q is an indeterminate), and (2) analytic functions of a complex variable q (convergence matters, |q| < 1 required). A function that works perfectly for formal series produces nonsensical results when someone tries to evaluate it numerically at q = 0.99, or a function that correctly checks convergence rejects valid formal computations.

**How to avoid:**
- Make the distinction **explicit in the type system**: `FormalSeries<R>` vs. `AnalyticFunction`. Operations on formal series never check convergence. Numerical evaluation requires explicit conversion with convergence verification.
- Error messages should distinguish "this doesn't converge at q=X" from "this is not a valid formal power series operation."

**Phase to address:** Phase 1-2 (type design and series arithmetic).

---

### Pitfall 15: Not Handling the q-Pochhammer Multi-Parameter Shorthand Correctly

**Severity:** LOW-MODERATE

**What goes wrong:**
Mathematical papers freely use `(a1, a2, ..., am; q)_n` as shorthand for `(a1;q)_n * (a2;q)_n * ... * (am;q)_n`. If your parser/API doesn't handle this, users must manually expand every multi-parameter symbol, which is tedious and error-prone. Worse, if you handle it but implement it as a single interleaved loop instead of a product of separate computations, cancellation bugs can creep in.

**How to avoid:**
- Support the multi-parameter syntax at the API level but implement it as a product of individual q-Pochhammer calls internally.
- Test that `qpochhammer([a,b], q, n) == qpochhammer(a, q, n) * qpochhammer(b, q, n)` for all test cases.

**Phase to address:** Phase 2 (q-Pochhammer implementation).

---

### Pitfall 16: Display/Serialization Disagreeing with Mathematical Convention

**Severity:** LOW-MODERATE

**What goes wrong:**
Your system displays `1 + -1*q^2 + ...` instead of `1 - q^2 + ...`, or sorts terms by decreasing power instead of increasing power (the convention in q-series), or displays `q^1` instead of `q`. Researchers comparing your output to Maple or published papers get confused and lose trust.

**How to avoid:**
- Study Garvan's Maple output format exactly. Match it term-for-term in the default display mode.
- Increasing power order (constant term first) is the q-series convention. Decreasing power order is the polynomial convention. Default to increasing.
- Suppress coefficients of 1 and -1, suppress exponents of 1, display `- q^k` not `+ (-1)*q^k`.

**Phase to address:** Phase 2 (output formatting), verified by visual comparison tests.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Dense coefficient arrays for series | Simple indexing, easy to implement | O(max_exponent) memory even for sparse series; wastes memory for series like `1 + q^1000` | Never for the general case. OK for benchmarks of dense series only |
| Using `f64` for numerical cross-validation | Fast, no dependencies | False positives/negatives in tests due to precision loss | Never for regression tests. OK for quick manual sanity checks |
| Global mutable intern table for hash consing | Simple implementation, maximum sharing | Thread-safety issues, memory leaks, prevents parallelism | Never. Use arena-scoped or thread-local tables |
| Returning `String` from Rust to Python | Avoids PyO3 complexity | Parsing overhead, lost structure, no programmatic access from Python | Only for display/debugging. Never for computation results |
| Implementing all of Garvan's functions before testing any | Feels productive | Bugs compound; later functions built on broken earlier ones | Never. Test each function before starting the next |
| Using `clone()` everywhere to satisfy the borrow checker | Code compiles | O(n) cloning overhead on expressions, defeats the point of structural sharing | Only for prototyping. Profile and eliminate before Phase 3 |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Dense series representation | Works fine for 100 terms | Use sparse (exponent, coefficient) pairs | > 1000 terms with gaps in exponents |
| Naive polynomial multiplication (schoolbook) | O(n^2) is fine for n < 100 | Implement Karatsuba or FFT-based multiplication | > 5000 terms |
| No truncation during series multiplication | Intermediate results are correct | Truncate to requested precision during multiplication | > 500 terms (intermediate swell) |
| `BigInt` for every coefficient | Correct for all sizes | Use small-int-optimized type (malachite) | > 10000 coefficients (allocation dominates) |
| Single-threaded partition computation | Simple correctness | Design for parallelizable coefficient computation | n > 50000 |
| Recomputing q-Pochhammer products from scratch | Correct results | Cache/memoize common products like (q;q)_inf | When user computes multiple identities in a session |

## Integration Gotchas

Common mistakes when connecting components.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Rust core <-> Python API | Exposing Rust lifetimes through PyO3 | Use `Arc`-based owned types at the boundary; keep lifetimes internal |
| Rust core <-> GMP (via rug) | Linking issues on Windows; C dependency complicates builds | Use malachite (pure Rust) for portability, or provide pre-built wheels |
| Symbolic engine <-> Numerical evaluator | Sharing the same expression type for both | Separate types: `SymbolicExpr` for algebra, `NumericalExpr` for evaluation |
| Series arithmetic <-> Pattern matching | Pattern matching on expanded series (slow) | Match on structural form first, expand to series only for coefficient extraction |
| User input <-> Internal IR | Parsing user expressions into IR loses structure | Preserve user's original representation alongside internal canonical form |

## UX Pitfalls

Common user experience mistakes for mathematical software.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No progress indicator for long computations | User thinks program is hung | Print "Computing to O(q^N)..." with periodic progress |
| Displaying raw Rust debug format | Unreadable output: `Sum([Prod([QPoch...])]) ` | Custom Display impl matching standard mathematical notation |
| Silent truncation without indicating precision | User doesn't know if `1 + q + q^2` is exact or truncated to O(q^3) | Always display the truncation order: `1 + q + q^2 + O(q^3)` |
| Python API requires importing 10 modules | Barrier to entry | Single `from qsymbolic import *` with all common functions |
| Different output from Maple for the same input | Erodes trust even if both are correct | Default to Garvan-compatible output format; offer alternatives via flags |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **q-Pochhammer implementation:** Often missing negative index support -- verify `(a;q)_{-3}` returns the correct reciprocal product
- [ ] **Series multiplication:** Often missing early truncation -- verify that multiplying two O(q^1000) series doesn't create an O(q^2000) intermediate
- [ ] **Eta-quotient conversion:** Often missing the holomorphicity check (Ligozat/Martin conditions) -- verify that invalid eta-quotients are rejected
- [ ] **Identity testing:** Often tests only one direction of a bidirectional identity -- verify both `LHS - RHS = 0` and `RHS - LHS = 0`
- [ ] **Python bindings:** Often missing `__repr__` and `__eq__` -- verify that `print(expr)` shows math notation and `expr1 == expr2` works
- [ ] **Error messages:** Often missing mathematical context -- verify that errors say "q-Pochhammer (a;q)_n requires n integer" not "type error: expected i64"
- [ ] **Convergence conditions:** Often missing boundary case |q| = 1 -- verify that numerical evaluation at q=1 raises a clear error, not NaN
- [ ] **Series precision tracking:** Often loses track of precision through operations -- verify that `O(q^10) * O(q^10) = O(q^10)`, not `O(q^20)` or untracked

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Wrong normalization strategy | HIGH (2-4 weeks) | Introduce a representation trait/enum layer between storage and algorithms; migrate algorithms one at a time |
| Infinite rewrite loops | MEDIUM (1 week) | Add iteration limits immediately; audit and classify rules as simplification vs transformation; remove bidirectional rules |
| Expression swell | MEDIUM (1-2 weeks) | Retrofit truncation into series multiplication; switch to sparse representation; profile to find worst offenders |
| Hash consing memory leak | MEDIUM (1 week) | Switch to arena-based allocation; drop the global intern table; accept some re-computation |
| q-Pochhammer edge case bugs | LOW (days) | Add comprehensive edge-case test suite; fix individual cases; no architectural change needed |
| PyO3 memory issues | MEDIUM (1-2 weeks) | Migrate to Bound API; restructure FFI boundary to minimize Python object creation in Rust |
| Scope creep | HIGH (organizational) | Freeze feature list; cut to MVP; ship what works; resist "just one more feature" |
| Single-oracle testing | LOW-MEDIUM (1 week) | Add Mathematica cross-validation; add property-based identity tests; doesn't require code changes |
| Over-engineered IR | MEDIUM (1-2 weeks) | Flatten to enum; remove unused variants and traits; simplify construction APIs |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Wrong normalization (P1) | Phase 1: Core IR design | Can represent q-Pochhammer, series, AND product form without lossy conversion |
| Infinite rewrite loops (P2) | Phase 2-3: Simplification engine | Every rule set passes termination check; no test timeouts |
| Expression swell (P3) | Phase 1-2: Series arithmetic | 10,000-term multiplication completes in < 1 second with < 100MB memory |
| Hash consing leak (P4) | Phase 1: Memory architecture | RSS stays flat during 10,000-operation stress test |
| q-Pochhammer edge cases (P5) | Phase 2: Core functions | All edge cases (n negative, 0, inf; a = 0, 1, q) have explicit tests |
| PyO3 memory (P6) | Phase 4-5: Python bindings | Python memory stays flat in a loop calling Rust 10,000 times |
| Scope creep (P7) | Every phase | Each phase deliverable is a specific Garvan function set, not infrastructure |
| Single-oracle testing (P8) | Phase 2-3: Test infrastructure | Three independent verification methods for each function |
| Over-engineered IR (P9) | Phase 1, refactored Phase 3 | Expr enum has < 12 variants after Phase 3 |
| Numerical instability (P10) | Phase 2-3: Testing | All numerical tests use arbitrary precision; no tolerance-tuning commits |
| Bignum overhead (P11) | Phase 1: Integer type choice | Benchmark: 10,000-term series ops within 2x of C/GMP baseline |
| Series blowup (P12) | Phase 3: Partition functions | p(10000) computes in < 1 second using recurrence, not generating function |
| Lifetime/FFI conflict (P13) | Phase 1: API design | Rust public API uses only `'static` or owned types |
| Formal vs analytic (P14) | Phase 1-2: Type system | Separate types for formal and numerical operations |
| Multi-parameter shorthand (P15) | Phase 2: q-Pochhammer | Test: multi-param equals product of singles |
| Display conventions (P16) | Phase 2: Output formatting | Visual comparison tests against Garvan's Maple output |

## Sources

### Official Documentation (HIGH confidence)
- [PyO3 Memory Management Guide](https://pyo3.rs/v0.22.5/memory) -- GIL Ref deprecation, Bound API, memory pitfalls
- [PyO3 Migration Guide (v0.22)](https://pyo3.rs/v0.22.2/migration) -- GIL Refs to Bound API migration path
- [SymPy Gotchas and Pitfalls](https://docs.sympy.org/latest/explanation/gotchas.html) -- Expression representation, equality, simplification
- [SymPy Best Practices](https://docs.sympy.org/latest/explanation/best-practices.html) -- Performance, numeric computation
- [egg: Equality Saturation Library](https://egraphs-good.github.io/) -- E-graph design, rewrite termination
- [bumpalo Arena Allocator](https://github.com/fitzgen/bumpalo) -- Rust arena allocation patterns
- [typed-arena](https://lib.rs/crates/typed-arena) -- Arena allocation with destructor support
- [malachite](https://www.malachite.rs/) -- Small-integer-optimized arbitrary precision
- [rug (GMP bindings)](https://docs.rs/rug) -- GMP/MPFR Rust bindings

### Research Papers and Technical Writing (MEDIUM-HIGH confidence)
- [Efficient Symbolic Computation via Hash Consing (2025)](https://arxiv.org/html/2509.20534) -- Hash consing pitfalls, DAG traversal overhead, memory behavior
- [Things I Would Like to See in a CAS - Fredrik Johansson](https://fredrikj.net/blog/2022/04/things-i-would-like-to-see-in-a-computer-algebra-system/) -- CAS design critique, integer overhead, normalization
- [Ensuring Termination of EqSat over a Terminating TRS](https://effect.systems/blog/ta-completion.html) -- Non-termination in equality saturation
- [Term Rewriting (Berkeley CS294-260)](https://inst.eecs.berkeley.edu/~cs294-260/sp24/2024-01-22-term-rewriting) -- Termination, confluence, completion

### Mathematical References (HIGH confidence for definitions, MEDIUM for implementation advice)
- [q-Pochhammer Symbol - Wikipedia](https://en.wikipedia.org/wiki/Q-Pochhammer_symbol) -- Definitions, edge cases, convergence
- [q-Pochhammer Symbol - Wolfram MathWorld](https://mathworld.wolfram.com/q-PochhammerSymbol.html) -- Formulas, special values
- [Garvan's q-series Maple Package (v1.3)](https://qseries.org/fgarvan/qmaple/qseries/) -- Reference implementation
- [Garvan's q-Product Tutorial](https://qseries.org/fgarvan/papers/qmaple.pdf) -- Andrews' algorithm, prodmake, etamake
- [Partition Function - Wikipedia](https://en.wikipedia.org/wiki/Partition_function_(number_theory)) -- Growth rates, recurrence relations
- [Jacobi Triple Product - Wikipedia](https://en.wikipedia.org/wiki/Jacobi_triple_product) -- Domain restrictions, convergence
- [Dedekind Eta Function - Wikipedia](https://en.wikipedia.org/wiki/Dedekind_eta_function) -- Eta-quotient conditions, Sturm's bound

### Community and Ecosystem (MEDIUM confidence)
- [Computer Algebra System - Wikipedia](https://en.wikipedia.org/wiki/Computer_algebra_system) -- Expression swell, CAS history
- [Rust enum vs trait object benchmarks](https://github.com/Britefury/rust-enum-vs-trait-benchmark) -- 12x performance difference
- [enum_dispatch crate](https://docs.rs/enum_dispatch) -- Performance optimization for dispatch patterns
- [Arenas in Rust - Manish Goregaokar](https://manishearth.github.io/blog/2021/03/15/arenas-in-rust/) -- Arena patterns and tradeoffs
- [num-bigint performance vs GMP](https://github.com/rust-num/num/issues/181) -- Benchmark comparisons

---
*Pitfalls research for: Q-Symbolic computation engine (Rust + Python, q-series domain)*
*Researched: 2026-02-13*
