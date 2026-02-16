# Phase 16: Extensions - Research

**Researched:** 2026-02-16
**Domain:** Recurrence solving (q-Petkovsek), nonterminating identity proofs (Chen-Hou-Mu), transformation chain discovery
**Confidence:** MEDIUM (algorithms well-documented in literature but no prior Rust implementations to reference)

## Summary

Phase 16 implements three independent algorithmic extensions that build on the q-Gosper (Phase 14) and q-Zeilberger (Phase 15) infrastructure. The three areas are: (1) q-Petkovsek/qHyper algorithm for finding q-hypergeometric closed-form solutions to recurrences produced by q-Zeilberger, (2) Chen-Hou-Mu parameter specialization method for proving nonterminating q-hypergeometric identities by reducing them to terminating cases solvable by q-Zeilberger, and (3) BFS/DFS transformation chain search over the existing transformation catalog (Heine 1/2/3, Sears, Watson) to find paths between two hypergeometric series.

These three components are largely independent -- they can be planned and implemented in parallel. The q-Petkovsek algorithm is the most algorithmically complex, requiring a normal form decomposition for rational functions under q-shift and systematic enumeration of candidate solutions. The Chen-Hou-Mu method is conceptually elegant but implementation must handle subtle edge cases around initial conditions. The transformation chain search is a straightforward graph algorithm over the existing transformation infrastructure.

**Primary recommendation:** Implement as three independent plans: (1) q-Petkovsek with closed-form output (SOLV-01, SOLV-02), (2) Chen-Hou-Mu nonterminating proofs (NTPR-01, NTPR-02), (3) transformation chain BFS (TRNS-01, TRNS-02). The q-Petkovsek plan is the largest; the other two are moderate.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| QRatPoly | Phase 13 | Polynomial arithmetic over Q | Already in codebase, exact arithmetic |
| QRatRationalFunc | Phase 13 | Rational function with auto-reduction | Already in codebase, GCD-based simplification |
| poly_gcd | Phase 13 | Polynomial GCD | Subresultant PRS, no coefficient explosion |
| q_shift / q_shift_n | Phase 13 | Polynomial q-shift operations | O(n) coefficient scaling |
| q_zeilberger | Phase 15 | Creative telescoping | Produces recurrences this phase solves |
| extract_term_ratio | Phase 14 | Term ratio extraction | Needed for solution verification |
| HypergeometricSeries | Phase 6 | Series representation | All transformations operate on this type |
| heine_transform_1/2/3, sears_transform, watson_transform | Phase 6 | Existing transformations | Nodes in transformation graph |
| verify_transformation | Phase 6 | FPS comparison | Verify transformation chain correctness |
| aqprod | Phase 3 | q-Pochhammer product | Closed-form output representation |
| FormalPowerSeries | Phase 2 | Sparse FPS | Numerical verification |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| solve_linear_system (in gosper.rs) | Phase 14 | Gaussian elimination over Q | Polynomial solution finding in q-Petkovsek |
| q_dispersion | Phase 14 | GCD shift detection | Normal form decomposition in q-Petkovsek |
| BTreeMap | std | Ordered map | Closed-form output representation |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Full q-Petkovsek | van Hoeij q-analogue | More efficient but much more complex; Petkovsek is sufficient for our use case |
| BFS chain search | Bidirectional BFS | Only worthwhile at depth > 5; our max depth is configurable and typically small |
| Symbolic parameter specialization | Direct FPS comparison | FPS comparison works but doesn't constitute a proof; Chen-Hou-Mu gives provable recurrence |

## Architecture Patterns

### Recommended Project Structure
```
crates/qsym-core/src/qseries/
    petkovsek.rs        # q-Petkovsek algorithm (SOLV-01, SOLV-02) -- NEW
    nonterminating.rs   # Chen-Hou-Mu nonterminating proofs (NTPR-01, NTPR-02) -- NEW
    hypergeometric.rs   # Add transformation chain search (TRNS-01, TRNS-02) to existing file
    mod.rs              # Re-export new public types/functions
```

### Pattern 1: q-Petkovsek Algorithm (qHyper)

**What:** Given a linear recurrence with constant (rational) coefficients c_0*y(n) + ... + c_d*y(n+d) = 0, find all q-hypergeometric solutions y(n) where y(n+1)/y(n) is a rational function of q^n.

**Algorithm overview (adapted from Abramov-Paule-Petkovsek 1998):**

The q-Petkovsek algorithm solves q-difference equations by decomposing the problem into finding polynomial solutions of an auxiliary equation. The key insight: if y is q-hypergeometric with y(qx)/y(x) = r(x), then r(x) can be decomposed into a "normal form."

**Step 1: Build the q-difference operator.** From recurrence coefficients c_0, ..., c_d and concrete q_val, form the operator L = sum_j c_j * sigma^j where sigma is the q-shift operator (sigma y)(x) = y(qx).

**Step 2: Extract leading and trailing polynomial coefficients.** From the recurrence p_0(x)*y(x) + p_1(x)*y(qx) + ... + p_d(x)*y(q^d*x) = 0, identify p_0(x) = c_0 and p_d(x) = c_d (these are constants in our case since q-Zeilberger produces constant-coefficient recurrences at concrete q).

**Step 3: Candidate enumeration.** A q-hypergeometric solution has the form y(n+1)/y(n) = z * a(q^n) / b(q^n) where z is a constant, a and b are monic polynomials, and a, b satisfy specific coprimality conditions. The candidates for (a, b) come from:
- Factors of the "first coefficient" p_0 (for b-candidates)
- Factors of the "last coefficient" p_d (for a-candidates)
- Combined with all possible q-shift offsets

**Step 4: For each candidate pair (a, b), solve for the polynomial part.** Substitute the candidate ratio into the recurrence and reduce to finding a polynomial solution of a transformed recurrence. Use the existing `solve_linear_system` for this.

**Step 5: Reconstruct the q-hypergeometric solution.** Combine z, a, b, and the polynomial part to express the solution as a product of q-Pochhammer symbols and q-powers.

**Important simplification for our use case:** Since q-Zeilberger at concrete q produces recurrences with *constant* (QRat) coefficients (not polynomial in n), the q-Petkovsek algorithm simplifies significantly. The "leading" and "trailing" coefficients are just c_0 and c_d (rational numbers), and the candidate generation is simpler.

**Example (for concrete q recurrences):**
```rust
// Recurrence from q-Zeilberger: c_0*S(n) + c_1*S(n+1) = 0
// Solution: S(n+1)/S(n) = -c_0/c_1 (constant ratio)
// This is already q-hypergeometric: S(n) = S(0) * (-c_0/c_1)^n
// Express as q-Pochhammer product by factoring the ratio
```

### Pattern 2: Chen-Hou-Mu Nonterminating Proof Method

**What:** Prove a nonterminating identity LHS(x) = RHS(x) by:
1. Replace parameter x with x*q^n to create a "parametrized" version
2. Both sides become functions of n (with x as a free parameter)
3. Apply q-Zeilberger to show both sides satisfy the same recurrence
4. Verify initial conditions match (by FPS comparison at small n values)
5. Conclude LHS = RHS for all x by induction

**When to use:** Nonterminating q-hypergeometric identities (infinite sums, e.g., q-Gauss, q-Kummer, Heine transformations) that cannot be directly proved by q-Zeilberger (which requires terminating sums).

**Algorithm:**
```
prove_nonterminating(lhs_builder, rhs_builder, x_param, q_val, max_order, n_verify):
    1. For n = n_test (e.g., n_test = 5):
       a. Build LHS(n_test) and RHS(n_test) by substituting x -> x*q^n_test
       b. Apply q_zeilberger to LHS(n_test) to find recurrence R_L
       c. Apply q_zeilberger to RHS(n_test) to find recurrence R_R
       d. Check that R_L and R_R have the same order and (proportional) coefficients
    2. Verify initial conditions:
       For j = 0, 1, ..., d (recurrence order):
           Compute LHS(j) and RHS(j) as FPS
           Check they agree to truncation_order
    3. If recurrences match AND initial conditions match:
       Return proof certificate (recurrence + initial condition verification)
    4. Else: return NoProof
```

**Key subtlety:** The parametrized series must be terminating in k (the summation variable) for q-Zeilberger to apply. The substitution x -> x*q^n achieves this: if the original has an upper parameter x, replacing with x*q^n makes it q^{-n}*x*q^n = x (not terminating). The trick is that x is treated as a free parameter and one of the *original* upper parameters must be q^{-n_test} for the termination. So the method works by choosing n_test to make the series terminate.

**Practical implementation:** Rather than symbolic parameter manipulation, we:
1. Take concrete n values
2. Build the series at each n
3. Run q_zeilberger at each n
4. Compare the recurrence coefficients (they should be proportional)
5. Verify initial conditions by direct FPS evaluation

### Pattern 3: Transformation Chain BFS

**What:** Given source and target HypergeometricSeries, search for a sequence of transformations (Heine 1/2/3, Sears, Watson) that transforms source into target (modulo a prefactor).

**Algorithm:**
```
find_transformation_chain(source, target, max_depth, variable, truncation_order):
    1. Build a queue of (series, chain_so_far, prefactor_so_far)
    2. Initialize with (source, [], FPS::one())
    3. BFS loop:
       a. Dequeue next state (current_series, chain, prefactor)
       b. If current_series matches target (via FPS comparison):
          Return TransformationChain { steps: chain, total_prefactor: prefactor }
       c. If len(chain) >= max_depth: skip
       d. For each transformation T in [heine1, heine2, heine3, sears, watson]:
          Try applying T to current_series
          If applicable:
            new_series = T.transformed
            new_prefactor = prefactor * T.prefactor
            new_chain = chain + [T]
            Enqueue (new_series, new_chain, new_prefactor)
    4. Return NoChainFound { depth: max_depth }
```

**Series equality check:** Two HypergeometricSeries are "equal" if they have the same (r,s), the same upper/lower parameter sets (as multisets), and the same argument. We compare QMonomial fields directly. The prefactor is tracked separately.

**Optimization:** Use a visited set to avoid revisiting the same series parameters. Hash or normalize the HypergeometricSeries (sort upper/lower params) for deduplication.

### Anti-Patterns to Avoid
- **Symbolic q-Petkovsek at generic q:** Our recurrences have concrete q values. Don't implement the full symbolic version -- it requires multivariate polynomial GCD and is far more complex.
- **Floating-point verification:** All verification must use exact QRat arithmetic via FPS comparison, never floating-point approximation.
- **Unbounded transformation search:** Always enforce a depth bound on BFS. Without it, the search space is infinite (repeated application of the same transformation yields new series).
- **Reinventing linear algebra:** Reuse the existing `solve_linear_system` from gosper.rs rather than writing a new solver. Extract it to a shared location or duplicate (as zeilberger.rs already does).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Polynomial GCD | Custom GCD | `poly_gcd` from Phase 13 | Subresultant PRS prevents coefficient explosion |
| Linear system solving | New Gaussian elimination | `solve_linear_system` from gosper.rs | Already handles exact QRat, overdetermined systems |
| FPS arithmetic | Custom series multiplication | `arithmetic::mul`, `arithmetic::add` | Phase 2 infrastructure handles truncation correctly |
| q-Pochhammer evaluation | Manual product loops | `aqprod` from Phase 3 | Handles positive/negative/infinite orders, edge cases |
| Series comparison | Coefficient-by-coefficient loops | FPS `PartialEq` | Already implemented with correct truncation semantics |

**Key insight:** Phase 16 is about *composing* existing infrastructure (q-Zeilberger, polynomial arithmetic, q-Pochhammer, transformations) into higher-level algorithms. Very little new low-level machinery is needed.

## Common Pitfalls

### Pitfall 1: Constant vs Polynomial Recurrence Coefficients
**What goes wrong:** Treating q-Zeilberger's constant-coefficient recurrences (at concrete q) as if they were polynomial-coefficient recurrences (symbolic in q^n).
**Why it happens:** The q-Petkovsek literature assumes polynomial coefficients p_i(x). Our q-Zeilberger produces constant QRat coefficients.
**How to avoid:** Design the q-Petkovsek interface for constant-coefficient recurrences. The solution ratio y(n+1)/y(n) is then a QRat constant, not a rational function of q^n. This dramatically simplifies the algorithm.
**Warning signs:** If the q-Petkovsek code needs QRatPoly coefficients in the recurrence, you're overcomplicating it.

### Pitfall 2: Chen-Hou-Mu Termination Assumption
**What goes wrong:** Applying q-Zeilberger to the parametrized series when it's not actually terminating.
**Why it happens:** The substitution x -> x*q^n doesn't automatically make the series terminate. You need an upper parameter that becomes q^{-n_test}.
**How to avoid:** The user must specify which parameter to specialize, OR the function must detect which upper parameter can serve as the termination parameter. Verify that the built series is terminating before calling q_zeilberger.
**Warning signs:** q_zeilberger returns NoRecurrence unexpectedly.

### Pitfall 3: Transformation Graph Cycles
**What goes wrong:** BFS revisits the same series parameters through different transformation paths, causing exponential blowup.
**Why it happens:** Heine transforms are not involutions (applying H1 then H1 again gives a different series), but the graph can still have cycles through longer paths.
**How to avoid:** Maintain a visited set keyed on normalized HypergeometricSeries parameters (sorted upper/lower, canonical QMonomial representation). Skip states already visited.
**Warning signs:** BFS takes excessively long even at small depth bounds.

### Pitfall 4: q-Petkovsek Candidate Explosion
**What goes wrong:** For higher-order recurrences, the number of candidate (a, b) pairs grows combinatorially.
**Why it happens:** Each factor of the leading/trailing coefficient generates candidates, and all q-shift offsets must be considered.
**How to avoid:** For our simplified constant-coefficient case, the candidate space is much smaller. Focus on the ratio -c_0/c_d for order-1 recurrences, and systematic enumeration for order-2+. Bound the search.
**Warning signs:** Algorithm takes more than a few seconds on a recurrence of order <= 3.

### Pitfall 5: Nonterminating Proof Initial Conditions
**What goes wrong:** Recurrences match but initial conditions don't, and the proof is incorrectly accepted.
**Why it happens:** Both sides may satisfy the same recurrence with different initial values.
**How to avoid:** Always verify initial conditions for n = 0, 1, ..., d (where d is the recurrence order) by computing both sides as FPS and comparing.
**Warning signs:** Proof accepted for identities that are known to be false.

### Pitfall 6: Closed-Form Output Representation
**What goes wrong:** The q-Petkovsek solution is correct as a ratio y(n+1)/y(n) but cannot be expressed in terms of q-Pochhammer symbols.
**Why it happens:** Not every rational constant ratio corresponds to a simple q-Pochhammer expression. The factoring step is nontrivial.
**How to avoid:** For constant ratio r = -c_0/c_1, express S(n) = S(0) * r^n. Then try to identify r as a ratio of q-Pochhammer quotients evaluated at specific points. Use the existing `qfactor` infrastructure if the ratio can be expressed as a product of (1-q^i) factors.
**Warning signs:** Closed-form output is just "r^n" without Pochhammer decomposition.

## Code Examples

### Example 1: q-Petkovsek for Order-1 Constant-Coefficient Recurrence
```rust
/// Result of the q-Petkovsek algorithm.
#[derive(Clone, Debug)]
pub struct QPetkovsekResult {
    /// The ratio y(n+1)/y(n) as a QRat (for constant-coefficient recurrences).
    pub ratio: QRat,
    /// Closed-form representation as q-Pochhammer factors.
    pub closed_form: Option<ClosedForm>,
}

#[derive(Clone, Debug)]
pub struct ClosedForm {
    /// Scalar prefactor (e.g., from S(0)).
    pub scalar: QRat,
    /// q-power: the solution includes a factor q^{power * n(n-1)/2} or similar.
    pub q_power_coeff: i64,
    /// Pochhammer numerator factors: (a_i;q)_n
    pub numer_factors: Vec<QMonomial>,
    /// Pochhammer denominator factors: (b_j;q)_n
    pub denom_factors: Vec<QMonomial>,
}

/// Solve a constant-coefficient q-recurrence for q-hypergeometric solutions.
///
/// Given c_0*S(n) + c_1*S(n+1) + ... + c_d*S(n+d) = 0 where c_j are QRat constants,
/// find all q-hypergeometric solutions.
pub fn q_petkovsek(
    coefficients: &[QRat],
    q_val: &QRat,
) -> Vec<QPetkovsekResult> {
    // For order 1: c_0*S(n) + c_1*S(n+1) = 0
    // => S(n+1)/S(n) = -c_0/c_1
    // For order 2+: enumerate candidate ratios...
    todo!()
}
```

### Example 2: Chen-Hou-Mu Nonterminating Proof
```rust
/// Result of a nonterminating identity proof.
#[derive(Clone, Debug)]
pub enum NonterminatingProofResult {
    /// Identity proved: both sides satisfy the same recurrence with matching initial conditions.
    Proved {
        /// The shared recurrence coefficients.
        recurrence: Vec<QRat>,
        /// Number of initial conditions verified.
        initial_conditions_checked: usize,
    },
    /// Proof failed: recurrences differ or initial conditions don't match.
    Failed {
        reason: String,
    },
}

/// Prove a nonterminating identity using the Chen-Hou-Mu method.
///
/// Both `lhs_builder` and `rhs_builder` take an integer n and return an FPS
/// representing the LHS and RHS of the identity at that parameter value.
pub fn prove_nonterminating(
    lhs_series_builder: &dyn Fn(i64) -> HypergeometricSeries,
    rhs_fps_builder: &dyn Fn(i64, SymbolId, i64) -> FormalPowerSeries,
    q_val: &QRat,
    n_test: i64,
    max_order: usize,
    variable: SymbolId,
    truncation_order: i64,
) -> NonterminatingProofResult {
    todo!()
}
```

### Example 3: Transformation Chain Search
```rust
/// A single step in a transformation chain.
#[derive(Clone, Debug)]
pub struct TransformationStep {
    /// Name of the transformation applied.
    pub name: String,
    /// The resulting series after this transformation.
    pub result: TransformationResult,
}

/// Result of a transformation chain search.
#[derive(Clone, Debug)]
pub enum TransformationChainResult {
    /// A chain was found.
    Found {
        /// The sequence of transformations.
        steps: Vec<TransformationStep>,
        /// The cumulative prefactor.
        total_prefactor: FormalPowerSeries,
    },
    /// No chain found within the depth bound.
    NotFound {
        max_depth: usize,
    },
}

/// Search for a transformation chain between two hypergeometric series.
pub fn find_transformation_chain(
    source: &HypergeometricSeries,
    target: &HypergeometricSeries,
    max_depth: usize,
    variable: SymbolId,
    truncation_order: i64,
) -> TransformationChainResult {
    todo!()
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Trial-and-error identity proving | Algorithmic q-Zeilberger + WZ | 1990s (Zeilberger) | Automated proof of terminating identities |
| Manual nonterminating proofs | Chen-Hou-Mu parameter specialization | 2005 | Systematic proof of nonterminating identities |
| Petkovsek's original algorithm | van Hoeij's improvement | ~2005 | Dramatically faster for large recurrences |
| Individual transformation lookup | Systematic chain search | Modern CAS | Discovers multi-step transformation paths |

**Deprecated/outdated:**
- Original Petkovsek algorithm: replaced by van Hoeij's version for efficiency. However, for our use case (small-order recurrences from q-Zeilberger), the original approach is sufficient and simpler to implement.

## Open Questions

1. **Constant vs polynomial coefficients in q-Petkovsek**
   - What we know: Our q-Zeilberger produces constant QRat coefficients at concrete q. The full q-Petkovsek handles polynomial coefficients.
   - What's unclear: Whether we ever need the polynomial-coefficient version. If q-Zeilberger is extended to work symbolically in q, we'd need it.
   - Recommendation: Implement only the constant-coefficient version. It handles all recurrences from our current q-Zeilberger. Document the limitation for future extension.

2. **Closed-form Pochhammer decomposition**
   - What we know: For order-1 recurrences, the ratio is a single QRat constant. For order-2+, solutions are more complex.
   - What's unclear: How to systematically decompose a q-power ratio into q-Pochhammer factors. The existing `qfactor` works on polynomials in q, not on ratio sequences.
   - Recommendation: For order-1, express as scalar * product. For order-2+, return the ratio and attempt Pochhammer decomposition as best-effort. Flag if decomposition fails.

3. **Chen-Hou-Mu: which parameter to specialize?**
   - What we know: The method requires choosing a parameter to replace with x*q^n. For standard identities (q-Gauss, q-Kummer, etc.), this is well-understood.
   - What's unclear: Automatic parameter selection for user-supplied identities.
   - Recommendation: Require the user to specify which parameter to specialize. Provide helper functions for common patterns. Automatic detection can be added later.

4. **Transformation chain: inverse transformations**
   - What we know: Heine/Sears/Watson are currently implemented as forward transformations only.
   - What's unclear: Whether we need inverse transformations to find chains. Heine 3 is an involution (applying twice returns related series), but H1 and H2 are not.
   - Recommendation: Only use forward transformations in the BFS. The three Heine transforms plus Sears plus Watson already form a rich enough graph for depth <= 4 searches. Inverse transformations can be added if needed.

## Sources

### Primary (HIGH confidence)
- Existing codebase: `gosper.rs`, `zeilberger.rs`, `hypergeometric.rs`, `mod.rs`, `poly/mod.rs` -- directly inspected
- ROADMAP.md, REQUIREMENTS.md -- project constraints

### Secondary (MEDIUM confidence)
- [Abramov-Paule-Petkovsek 1998: "q-Hypergeometric solutions of q-difference equations"](https://www3.risc.jku.at/publications/download/risc_2062/Abramov_PP_Petkovsek.pdf) -- qHyper algorithm, normal form decomposition
- [Chen-Hou-Mu 2005: "Nonterminating Basic Hypergeometric Series and the q-Zeilberger Algorithm"](https://arxiv.org/abs/math/0509281) -- parameter specialization method
- [Koepf "Hypergeometric Summation" (Springer 2014)](https://link.springer.com/book/10.1007/978-1-4471-6464-7) -- Chapters on q-Petkovsek and q-Zeilberger algorithms
- [Petkovsek, Wilf, Zeilberger "A=B" (1996)](https://www2.math.upenn.edu/~wilf/AeqB.html) -- Chapter 8: Algorithm Hyper, foundational reference

### Tertiary (LOW confidence)
- [Wikipedia: Petkovsek's algorithm](https://en.wikipedia.org/wiki/Petkov%C5%A1ek's_algorithm) -- overview only, no implementation details
- [Koepf qFPS Maple package](https://www.hypergeometric-summation.org/) -- reference implementation exists in Maple but not accessible for direct comparison

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all libraries already in codebase and well-tested
- Architecture (q-Petkovsek): MEDIUM - algorithm well-documented but implementation details for constant-coefficient simplification are my synthesis, not from a specific source
- Architecture (Chen-Hou-Mu): MEDIUM - method is clear from the paper abstract and confirmed by multiple sources, but specific implementation details (parameter selection, initial condition handling) are inferred
- Architecture (transformation BFS): HIGH - straightforward graph search over existing transformation functions
- Pitfalls: HIGH - based on direct codebase analysis and understanding of the algorithms

**Research date:** 2026-02-16
**Valid until:** 2026-03-16 (stable algorithms, no fast-moving dependencies)
