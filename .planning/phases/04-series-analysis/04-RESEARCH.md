# Phase 4: Series Analysis - Research

**Researched:** 2026-02-13
**Domain:** q-series analysis -- series-to-product conversion, factoring, subsequence extraction, and algebraic relation discovery
**Confidence:** MEDIUM-HIGH (algorithms well-documented in Garvan's papers; implementation details require some inference)

## Summary

Phase 4 completes qseries package parity by implementing the "analysis" layer: converting series back to product forms, factoring q-polynomials, extracting arithmetic subsequences, and discovering algebraic relations between q-series. This is the inverse of Phase 3 (which built products from parameters); Phase 4 recovers parameters from series data.

The core algorithms fall into four groups: (1) **Series-to-product conversion** using Andrews' algorithm via logarithmic derivative / Mobius inversion (prodmake, etamake, jacprodmake, mprodmake, qetamake), (2) **Series factoring** into (1-q^n) factors (qfactor, zqfactor), (3) **Series utilities** for subsequence extraction and degree measurement (sift, qdegree, lqdegree), and (4) **Relation discovery** via linear algebra -- constructing coefficient matrices and computing null spaces to find linear combinations, homogeneous relations, polynomial relations, and congruences (12 find* functions total).

The phase builds entirely on the existing FormalPowerSeries infrastructure. No new external dependencies are needed -- the linear algebra required (rational Gaussian elimination for null space computation) is small enough to implement directly using the existing QRat type, avoiding external matrix library dependencies.

**Primary recommendation:** Implement in 4 plan groups: (1) series-to-product conversion (prodmake + etamake/jacprodmake/mprodmake/qetamake), (2) factoring + utilities (qfactor, zqfactor, sift, qdegree, lqdegree), (3) core relation discovery (findlincombo, findhom, findpoly with rational Gaussian elimination), (4) full relation discovery suite (findcong, findnonhom, findhomcombo, findnonhomcombo, modp variants, findmaxind, findprod).

## Standard Stack

### Core (no new dependencies needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rug | 1.28 | QRat arbitrary-precision rationals | Already in use; prodmake/relation discovery need exact rational arithmetic |
| (built-in) | - | BTreeMap<i64, QRat> | FormalPowerSeries sparse coefficients -- all Phase 4 functions operate on these |

### Supporting (no new crates)

| Component | Purpose | Rationale |
|-----------|---------|-----------|
| Hand-rolled rational Gaussian elimination | Null space computation for relation discovery | Matrix sizes are small (typically < 100x100). QRat gives exact results. External crates (nalgebra, ndarray) use f64 which loses precision. |
| Mobius function table | Prodmake algorithm needs mu(n) for exponent recovery | Trivial to compute via sieve; not worth a dependency. |
| Divisor enumeration | Prodmake algorithm needs divisors of n | Simple trial division; n < truncation_order (typically < 1000). |

### Why No External Linear Algebra Crate

The relation discovery functions (findlincombo, findhom, findpoly, etc.) need exact rational null space computation. The matrices involved are:
- Small: rows = number of series coefficients used, cols = number of basis monomials
- Exact: Must use QRat, not floating point (relations are exact, not approximate)
- Sparse operations are not needed (matrices are typically dense)

No Rust crate provides Gaussian elimination over arbitrary-precision rationals (rug::Rational). Implementing row reduction + null space extraction over QRat is ~80 lines of code and trivially correct.

## Architecture Patterns

### Recommended Module Structure

```
crates/qsym-core/src/qseries/
    mod.rs              -- add re-exports for new functions
    pochhammer.rs       -- (existing) aqprod
    products.rs         -- (existing) etaq, jacprod, etc.
    theta.rs            -- (existing) theta2, theta3, theta4
    partitions.rs       -- (existing) partition_count, etc.
    rank_crank.rs       -- (existing) rank_gf, crank_gf
    prodmake.rs         -- NEW: prodmake, etamake, jacprodmake, mprodmake, qetamake
    factoring.rs        -- NEW: qfactor, zqfactor
    utilities.rs        -- NEW: sift, qdegree, lqdegree
    relations.rs        -- NEW: findlincombo, findhom, findpoly, findnonhom, etc.
    linalg.rs           -- NEW: rational matrix operations (Gaussian elimination, null space)
```

### Pattern 1: FPS-in, structured-result-out

All Phase 4 functions take `&FormalPowerSeries` inputs and return structured results (not just FPS).

**What:** Functions accept series and return analysis results in domain-specific types.
**When to use:** Every Phase 4 function.

```rust
/// Result of prodmake: the exponents a_n in prod (1-q^n)^{-a_n}
pub struct InfiniteProductForm {
    /// Exponents: maps n -> a_n where product is prod (1-q^n)^{-a_n}
    pub exponents: BTreeMap<i64, QRat>,
    /// How many terms were used to determine this product
    pub terms_used: i64,
}

/// Result of etamake: eta-quotient representation
pub struct EtaQuotient {
    /// Maps delta -> r_delta where result is prod eta(delta*tau)^{r_delta}
    pub factors: BTreeMap<i64, i64>,
    /// Fractional power of q prefactor (e.g., q^{1/24})
    pub q_shift: QRat,
}

/// Result of qfactor: product of (1-q^i) factors
pub struct QFactorization {
    /// Maps i -> multiplicity where product is prod (1-q^i)^{mult}
    pub factors: BTreeMap<i64, i64>,
    /// Scalar prefactor
    pub scalar: QRat,
    /// Whether factorization is exact (remainder is zero)
    pub is_exact: bool,
}
```

### Pattern 2: Coefficient Matrix + Null Space for Relation Discovery

**What:** All relation discovery functions follow the same pattern:
1. Build a list of "candidate" series (monomials in the input series)
2. Extract coefficients into a matrix (rows = coefficients, cols = candidates)
3. Compute the null space of this matrix over Q
4. Interpret null space vectors as relations

**When to use:** findlincombo, findhom, findnonhom, findpoly, findhomcombo, findnonhomcombo, and their modp variants.

```rust
/// Build coefficient matrix: each column is a candidate series,
/// each row is a coefficient index.
fn build_coefficient_matrix(
    candidates: &[FormalPowerSeries],
    start_order: i64,
    num_rows: usize,
) -> Vec<Vec<QRat>> {
    let mut matrix = Vec::new();
    for i in 0..num_rows {
        let row: Vec<QRat> = candidates.iter()
            .map(|s| s.coeff(start_order + i as i64))
            .collect();
        matrix.push(row);
    }
    matrix
}

/// Compute null space of matrix over Q using Gaussian elimination.
/// Returns list of basis vectors for the kernel.
fn rational_null_space(matrix: &[Vec<QRat>]) -> Vec<Vec<QRat>> {
    // Row-reduce to RREF, identify free variables, back-substitute
    // ...
}
```

### Pattern 3: Andrews' Algorithm (prodmake core)

**What:** Given f(q) = 1 + sum b_n q^n, recover a_n such that f(q) = prod (1-q^n)^{-a_n}.

**Algorithm steps:**
1. Compute c_n (the "sigma-like" sums): c_n = n*b_n - sum_{j=1}^{n-1} c_j * b_{n-j}
2. Recover a_n via Mobius inversion: n*a_n = sum_{d|n} mu(n/d) * c_d
3. Output the nonzero a_n values

```rust
/// Andrews' algorithm: recover infinite product exponents from series coefficients.
///
/// Given f(q) = 1 + sum_{n>=1} b_n q^n, finds a_n such that
/// f(q) = prod_{n>=1} (1 - q^n)^{-a_n} to O(q^T).
pub fn prodmake(f: &FormalPowerSeries, max_n: i64) -> InfiniteProductForm {
    // Step 1: Ensure f has constant term 1
    let b0 = f.coeff(0);
    assert!(b0 == QRat::one(), "prodmake: series must have constant term 1");

    // Step 2: Compute c_n values via recurrence
    // c_n = n*b_n - sum_{j=1}^{n-1} c_j * b_{n-j}
    let mut c = BTreeMap::new();
    for n in 1..=max_n {
        let mut val = QRat::from((n, 1i64)) * f.coeff(n);
        for j in 1..n {
            if let Some(cj) = c.get(&j) {
                val = val - cj.clone() * f.coeff(n - j);
            }
        }
        if !val.is_zero() {
            c.insert(n, val);
        }
    }

    // Step 3: Recover a_n via Mobius inversion
    // n * a_n = sum_{d|n} mu(n/d) * c_d
    let mut exponents = BTreeMap::new();
    for n in 1..=max_n {
        let mut sum = QRat::zero();
        for d in divisors(n) {
            if let Some(cd) = c.get(&d) {
                let mu_val = mobius(n / d);
                if mu_val != 0 {
                    sum = sum + QRat::from((mu_val as i64, 1i64)) * cd.clone();
                }
            }
        }
        if !sum.is_zero() {
            let a_n = sum / QRat::from((n, 1i64));
            exponents.insert(n, a_n);
        }
    }

    InfiniteProductForm { exponents, terms_used: max_n }
}
```

### Pattern 4: etamake/jacprodmake as post-processing of prodmake

**What:** etamake and jacprodmake are NOT separate algorithms -- they post-process prodmake results.
- **etamake**: Takes prodmake output (exponents a_n) and groups them into eta-quotient form: prod eta(d*tau)^{r_d}. The key insight is eta(d*tau) = q^{d/24} * (q^d; q^d)_inf, so prodmake exponents at multiples of d correspond to eta(d*tau) factors.
- **jacprodmake**: Takes prodmake output and attempts to express it as a product of JAC(a,b) factors. Searches for period b such that exponents group into JAC patterns.
- **mprodmake**: Converts to products of (1+q^n) factors instead of (1-q^n) factors.
- **qetamake**: Like etamake but outputs in (q^d;q^d)_inf notation (the "q-eta" form) rather than eta(d*tau) modular form notation.

### Anti-Patterns to Avoid

- **Computing product expansions to verify prodmake by re-expanding:** Instead, directly compare the original series coefficients against the product's series expansion. The test should compute the product from the exponents and verify coefficient-by-coefficient agreement.
- **Using floating point for relation discovery:** Relations are exact; floating point introduces false positives and misses true relations. Always use QRat.
- **Building enormous matrices for findpoly:** The number of monomials grows combinatorially. Use degree bounds and truncation order to keep matrices manageable.
- **Separate Gaussian elimination implementations for Q and Z/pZ:** Factor the linear algebra to be generic over a "field" trait, then instantiate for QRat and for modular integers.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Arbitrary precision rationals | Custom rational type | QRat (rug::Rational) | Already have it; GMP is fast and correct |
| Series arithmetic | Custom multiply/add | arithmetic::mul, arithmetic::add | Already implemented and tested |
| Infinite product expansion | Custom product loop | InfiniteProductGenerator | Already implemented |
| Divisor enumeration for n < 1000 | Sieve-based factorization | Simple trial division loop | n is small; overengineering wastes time |

**Key insight:** Phase 4 functions are algorithms that COMPOSE existing FPS operations. The FPS layer is complete; Phase 4 adds analysis logic on top.

## Common Pitfalls

### Pitfall 1: Prodmake with non-unit constant term
**What goes wrong:** If f(0) != 1, the logarithmic derivative formula fails.
**Why it happens:** Many q-series have prefactors (e.g., q^k * prod(...) or scalar * prod(...)).
**How to avoid:** Before running Andrews' algorithm, extract the leading term. If f = c * q^k * g(q) where g(0) = 1, run prodmake on g and prepend the q^k shift and scalar.
**Warning signs:** Assertion failure on constant term, or nonsensical exponents.

### Pitfall 2: Non-integer exponents in prodmake
**What goes wrong:** Andrews' algorithm can produce rational (non-integer) a_n values, meaning the series does not have a clean infinite product form.
**Why it happens:** Not every q-series has an infinite product representation with integer exponents.
**How to avoid:** Check that all recovered a_n are integers. If not, report that the series is not expressible as a simple product (or has fractional exponents as in eta-quotients with q^{r/24} prefactors).
**Warning signs:** QRat exponents with denominator != 1.

### Pitfall 3: Sift parameter confusion (j vs residue class)
**What goes wrong:** Garvan's sift(f, q, m, j) extracts terms where exponent = j (mod m), returning sum a_{m*i+j} * q^i. The indices shift: the output exponent i corresponds to input exponent m*i+j.
**Why it happens:** Off-by-one errors in the index mapping.
**How to avoid:** Be explicit: output coefficient at q^i = input coefficient at q^{m*i+j}. Write tests with known sift results.
**Warning signs:** Wrong output when sifting known series like partition function mod 5.

### Pitfall 4: Null space dimension > 1 in relation discovery
**What goes wrong:** findlincombo/findhom may return multiple relations, or the "topshift" parameter may need adjustment to eliminate spurious relations.
**Why it happens:** When the coefficient matrix doesn't have enough rows (not enough series terms), the null space is larger than expected, yielding spurious relations.
**How to avoid:** Use the `topshift` parameter to add extra rows beyond the minimum, filtering out accidental relations. Garvan's convention: topshift=0 usually works; increase if needed.
**Warning signs:** More relations returned than expected, or relations that fail at higher precision.

### Pitfall 5: qfactor on non-polynomial input
**What goes wrong:** qfactor expects a polynomial in q (finite series), not an infinite series.
**Why it happens:** Confusion between FPS (which always has a truncation order) and actual polynomials.
**How to avoid:** qfactor should work on the polynomial part of the series (coefficients below truncation). The result should be verified by re-expanding and checking agreement.
**Warning signs:** Unexpected remainder, factorization that doesn't round-trip.

### Pitfall 6: Modular arithmetic in findlincombomodp/findhommodp
**What goes wrong:** Working mod p requires converting QRat coefficients to Z/pZ, which loses information.
**Why it happens:** The modp variants reduce the problem to finite field linear algebra.
**How to avoid:** Implement modular reduction carefully: QRat to i64 mod p (only valid when denominator is coprime to p). Use modular inverse for division.
**Warning signs:** Panic on division by zero mod p when denominator is divisible by p.

## Code Examples

### Example 1: prodmake on Euler function

```rust
// Euler function (q;q)_inf = prod (1-q^n)
// As a series: 1 - q - q^2 + q^5 + q^7 - q^12 - q^15 + ...
// prodmake should recover a_n = 1 for all n (since (q;q)_inf = prod (1-q^n)^1)

let euler_series = euler_function_generator(q, 50).into_series();
let result = prodmake(&euler_series, 49);
// result.exponents should be: {1: -1, 2: -1, 3: -1, ...}
// (negative because (q;q)_inf = prod (1-q^n)^{+1}, but prodmake returns
//  exponents for prod (1-q^n)^{-a_n}, so a_n = -1 for all n)
```

### Example 2: sift to extract partition congruences

```rust
// p(n) generating function: 1/(q;q)_inf
// sift(pgf, q, 5, 4) should give the subsequence p(5n+4)
// which is always divisible by 5 (Ramanujan's congruence)

let pgf = partition_gf(q, 200);
let sifted = sift(&pgf, 5, 4);
// sifted.coeff(0) = p(4) = 5
// sifted.coeff(1) = p(9) = 30
// sifted.coeff(2) = p(14) = 135
// All divisible by 5
```

### Example 3: findlincombo

```rust
// Given three series f1, f2, f3 where f3 = 3*f1 + 7*f2,
// findlincombo(f3, &[f1, f2], topshift=0) should return [3, 7]

let coeffs = findlincombo(&f3, &[&f1, &f2], 0);
// coeffs = Some(vec![QRat::from(3), QRat::from(7)])
```

### Example 4: Gaussian elimination for null space

```rust
/// Compute null space of m x n matrix over Q.
/// Returns basis vectors for ker(A).
pub fn rational_null_space(matrix: &[Vec<QRat>]) -> Vec<Vec<QRat>> {
    let m = matrix.len(); // rows
    if m == 0 { return vec![]; }
    let n = matrix[0].len(); // cols

    // Copy matrix for in-place reduction
    let mut a: Vec<Vec<QRat>> = matrix.to_vec();

    // Forward elimination with partial pivoting
    let mut pivot_cols = Vec::new();
    let mut row = 0;
    for col in 0..n {
        // Find pivot in column
        let pivot_row = (row..m).find(|&r| !a[r][col].is_zero());
        if let Some(pr) = pivot_row {
            a.swap(row, pr);
            pivot_cols.push(col);

            // Scale pivot row
            let pivot_val = a[row][col].clone();
            for j in col..n {
                a[row][j] = a[row][j].clone() / pivot_val.clone();
            }

            // Eliminate column in all other rows
            for r in 0..m {
                if r != row && !a[r][col].is_zero() {
                    let factor = a[r][col].clone();
                    for j in col..n {
                        let sub = factor.clone() * a[row][j].clone();
                        a[r][j] = a[r][j].clone() - sub;
                    }
                }
            }
            row += 1;
        }
    }

    // Free variables = columns not in pivot_cols
    let free_cols: Vec<usize> = (0..n)
        .filter(|c| !pivot_cols.contains(c))
        .collect();

    // Build null space basis vectors
    let mut basis = Vec::new();
    for &fc in &free_cols {
        let mut vec = vec![QRat::zero(); n];
        vec[fc] = QRat::one();
        // For each pivot row, set the corresponding entry
        for (i, &pc) in pivot_cols.iter().enumerate() {
            vec[pc] = -a[i][fc].clone();
        }
        basis.push(vec);
    }
    basis
}
```

## Algorithm Details

### Andrews' Algorithm (prodmake) -- Step by Step

**Input:** f(q) = 1 + b_1*q + b_2*q^2 + ... + O(q^T)
**Output:** Exponents a_1, a_2, ..., a_{T-1} such that f(q) = prod_{n=1}^{T-1} (1-q^n)^{-a_n} + O(q^T)

**Mathematical basis:** Taking log of both sides:
log f(q) = -sum_{n=1}^{inf} a_n * log(1 - q^n)
         = sum_{n=1}^{inf} a_n * sum_{k=1}^{inf} q^{nk}/k

Taking q * d/dq of both sides:
q*f'(q)/f(q) = sum_{n=1}^{inf} n*a_n * q^n / (1-q^n)
             = sum_{n=1}^{inf} (sum_{d|n} d*a_d) * q^n

Define c_n = sum_{d|n} d*a_d. Then:
q*f'(q)/f(q) = sum_{n=1}^{inf} c_n * q^n

**Step 1:** Compute c_n from b_n using the recurrence:
- c_1 = b_1
- c_n = n*b_n - sum_{j=1}^{n-1} c_j * b_{n-j}

This comes from equating coefficients of q^n on both sides of:
f(q) * (sum c_n q^n) = q * f'(q)

**Step 2:** Recover a_n from c_n via Mobius inversion:
n*a_n = sum_{d|n} mu(n/d) * c_d

Where mu is the Mobius function.

**Step 3:** Return nonzero a_n values.

### etamake Algorithm

**Input:** f(q) (possibly with q^{r/24} prefactor), max_n
**Output:** EtaQuotient {factors, q_shift}

1. Run prodmake on f (after normalizing constant term to 1)
2. The prodmake exponents a_n tell us f = prod (1-q^n)^{-a_n}
3. Group exponents by GCD pattern: if a_n is nonzero only when d|n for some d, then eta(d*tau)^r_d contributes a_{kd} = r_d for k=1,2,...
4. Use the identity: eta(d*tau) = q^{d/24} * prod_{k=1}^{inf} (1-q^{dk}) = q^{d/24} * (q^d;q^d)_inf
5. The q-shift (prefactor) is sum_d r_d * d/24

### jacprodmake Algorithm

**Input:** f(q), max_n, optional period P
**Output:** Product of JAC(a,b) terms

1. Run prodmake on f
2. Search for period b (try b=1,2,3,... up to P or some bound)
3. For each candidate b, check if exponents group as: a_n depends only on n mod b
4. If grouped: for each residue class r mod b (0 < r < b), compute JAC(r,b) contribution
5. JAC(0,b) = (q^b;q^b)_inf represents the eta-like factor
6. Return the product of JAC factors matching the exponents

### sift Algorithm

**Input:** f(q) = sum a_i q^i + O(q^T), modulus m, residue j
**Output:** g(q) = sum a_{m*i+j} q^i + O(q^{floor((T-j)/m)})

Simply extract every m-th coefficient starting at j:
```
for i in 0.. {
    let src_exp = m * i + j;
    if src_exp >= T { break; }
    g.set_coeff(i, f.coeff(src_exp));
}
```

### qfactor Algorithm

**Input:** f(q) polynomial (finite series)
**Output:** Product of (1-q^i)^{e_i} factors

1. Ensure f(0) = 1 (or normalize)
2. For i = 1, 2, 3, ...:
   - While f is divisible by (1-q^i), divide and increment e_i
   - "Divisible" means: if f(q)/(1-q^i) has only polynomial terms (no remainder)
   - In practice: f = (1-q^i)*g + r. Check r = 0.
3. Continue until f = 1 (fully factored) or no more factors found

### Relation Discovery Core Algorithm

Used by findlincombo, findhom, findpoly, findnonhom:

**Input:** List of candidate series [s_1, ..., s_k], precision T

**Step 1:** Build coefficient matrix A where A[i][j] = coeff of q^i in s_j
- Rows: i from start_order to start_order + num_rows - 1
- Cols: j = 1..k (one per candidate)
- num_rows should exceed k by "topshift" margin

**Step 2:** Compute null space of A over Q (or Z/pZ for modp variants)

**Step 3:** Each null space vector [c_1, ..., c_k] gives a relation:
c_1*s_1 + c_2*s_2 + ... + c_k*s_k = 0

**For findhom(L, n):** candidates are all degree-n monomials in L
**For findnonhom(L, n):** candidates are all monomials of degree <= n in L
**For findpoly(x, y, deg1, deg2):** candidates are x^i * y^j for 0 <= i <= deg1, 0 <= j <= deg2
**For findlincombo(f, L):** candidates are f together with elements of L

## Function Catalog (Complete Phase 4 Scope)

### Group 1: Series-to-Product Conversion (QSER-09, 10, 11, 12)

| Function | Signature | Description |
|----------|-----------|-------------|
| `prodmake` | `(f: &FPS, max_n: i64) -> InfiniteProductForm` | Andrews' algorithm: recover prod (1-q^n)^{-a_n} |
| `etamake` | `(f: &FPS, max_n: i64) -> EtaQuotient` | Express as eta-quotient (modular form notation) |
| `jacprodmake` | `(f: &FPS, max_n: i64) -> JacobiProductForm` | Express as product of JAC(a,b) factors |
| `jacprodmake` (with period) | `(f: &FPS, max_n: i64, period: i64) -> JacobiProductForm` | Restrict JAC search to given period |
| `mprodmake` | `(f: &FPS, max_n: i64) -> MProductForm` | Express as product of (1+q^n) factors |
| `qetamake` | `(f: &FPS, max_n: i64) -> QEtaQuotient` | Express in (q^d;q^d)_inf notation |

### Group 2: Factoring and Utilities (QSER-13, 14, 15)

| Function | Signature | Description |
|----------|-----------|-------------|
| `qfactor` | `(f: &FPS) -> QFactorization` | Factor polynomial into (1-q^i) terms |
| `zqfactor` | `(f: &FPS) -> ZQFactorization` | Factor (z,q)-series into (1-z^i*q^j) terms |
| `sift` | `(f: &FPS, m: i64, j: i64) -> FPS` | Extract subsequence: coeff(m*i+j) -> coeff(i) |
| `qdegree` | `(f: &FPS) -> Option<i64>` | Highest exponent with nonzero coefficient |
| `lqdegree` | `(f: &FPS) -> Option<i64>` | Lowest exponent with nonzero coefficient |

### Group 3: Core Relation Discovery (QSER-16, 17, 18)

| Function | Signature | Description |
|----------|-----------|-------------|
| `findlincombo` | `(f: &FPS, basis: &[&FPS], topshift: i64) -> Option<Vec<QRat>>` | Express f as linear combination of basis |
| `findhom` | `(series: &[&FPS], degree: i64, topshift: i64) -> Vec<Vec<QRat>>` | Find homogeneous degree-n relations |
| `findpoly` | `(x: &FPS, y: &FPS, deg1: i64, deg2: i64) -> Option<Polynomial2D>` | Find polynomial relation between two series |

### Group 4: Full Discovery Suite (QSER-19)

| Function | Signature | Description |
|----------|-----------|-------------|
| `findcong` | `(f: &FPS, max_coeff: i64) -> Vec<Congruence>` | Find partition-type congruences c[An+B] = 0 mod R |
| `findnonhom` | `(series: &[&FPS], degree: i64, topshift: i64) -> Vec<Vec<QRat>>` | Find non-homogeneous degree <= n relations |
| `findhomcombo` | `(f: &FPS, basis: &[&FPS], degree: i64, topshift: i64) -> Option<Vec<QRat>>` | Express f as homogeneous combo |
| `findnonhomcombo` | `(f: &FPS, basis: &[&FPS], degree: i64, topshift: i64) -> Option<Vec<QRat>>` | Express f as non-homogeneous combo |
| `findlincombomodp` | `(f: &FPS, basis: &[&FPS], p: i64, topshift: i64) -> Option<Vec<i64>>` | Linear combo mod p |
| `findhommodp` | `(series: &[&FPS], p: i64, degree: i64, topshift: i64) -> Vec<Vec<i64>>` | Homogeneous relations mod p |
| `findhomcombomodp` | `(f: &FPS, basis: &[&FPS], p: i64, degree: i64, topshift: i64) -> Option<Vec<i64>>` | Homogeneous combo mod p |
| `findmaxind` | `(series: &[&FPS], topshift: i64) -> Vec<usize>` | Find maximal linearly independent subset |
| `findprod` | `(series: &[&FPS], max_coeff: i64, max_exp: i64) -> Vec<Vec<i64>>` | Search linear combos yielding nice products |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual series inspection | Automated prodmake/etamake | Garvan 1998 | Researchers can convert any series to product form automatically |
| Brute-force relation search | Coefficient matrix + null space | Garvan 1998 | Systematic discovery of algebraic relations |
| Integer-only factoring | QRat exact factoring | Always | Handles fractional exponents in eta-quotients |

**Note on Garvan's package versions:**
- v1.1: Original package
- v1.2: Added `ebase` variable for jacprodmake
- v1.3: Added modp variants, fixed sift for negative powers, replaced `taylor` with `series`

## Open Questions

1. **Handling of non-unit leading coefficient in prodmake**
   - What we know: Garvan's prodmake normalizes the input. Andrews' algorithm requires constant term = 1.
   - What's unclear: Exact normalization strategy -- does Garvan strip q^k prefactor and scalar, or does he return an error?
   - Recommendation: Strip q^{min_order} prefactor and scalar 1/f(0), apply prodmake to normalized series, then include prefactor in result. This is mathematically clean.

2. **jacprodmake period search strategy**
   - What we know: jacprodmake searches for period b. With optional P parameter, it restricts to divisors of P.
   - What's unclear: Upper bound on search when P is not given. Garvan says "tries periods" but doesn't specify the bound.
   - Recommendation: Default bound = truncation_order / 2. With P parameter, try only divisors of P. Return the smallest period that works.

3. **zqfactor reliability**
   - What we know: Garvan's documentation says "does not work all the time."
   - What's unclear: When exactly it fails and what the failure mode is.
   - Recommendation: Implement a best-effort version. Return is_exact=false when factorization is incomplete. Document the limitation.

4. **findprod search bounds**
   - What we know: findprod searches over all z-linear combinations with bounded coefficients.
   - What's unclear: How to efficiently prune the search space.
   - Recommendation: Use the max_coeff parameter to bound search. For each combination, use checkprod (a helper that checks if a series has a nice product form via prodmake).

## Sources

### Primary (HIGH confidence)
- [Garvan qseries v1.3 function list](https://qseries.org/fgarvan/qmaple/qseries/) -- Complete function catalog with parameter signatures
- [prodmake docs](https://qseries.org/fgarvan/qmaple/qseries/functions/prodmake.html) -- Signature and examples
- [etamake docs](https://qseries.org/fgarvan/qmaple/qseries/functions/etamake.html) -- Signature and eta-product conversion
- [jacprodmake docs](https://qseries.org/fgarvan/qmaple/qseries/functions/jacprodmake.html) -- Signature and JAC pattern search
- [mprodmake docs](https://qseries.org/fgarvan/qmaple/qseries/functions/mprodmake.html) -- (1+q^n) product form
- [qetamake docs](https://qseries.org/fgarvan/qmaple/qseries/functions/qetamake.html) -- q-eta notation
- [qfactor docs](https://qseries.org/fgarvan/qmaple/qseries/functions/qfactor.html) -- q-polynomial factoring
- [zqfactor docs](https://qseries.org/fgarvan/qmaple/qseries/functions/zqfactor.html) -- (z,q)-series factoring
- [sift docs](https://qseries.org/fgarvan/qmaple/qseries/functions/sift.html) -- Subsequence extraction
- [qdegree docs](https://qseries.org/fgarvan/qmaple/qseries/functions/qdegree.html) -- Degree computation
- [lqdegree docs](https://qseries.org/fgarvan/qmaple/qseries/functions/lqdegree.html) -- Lowest degree computation
- [findlincombo docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findlincombo.html) -- Linear combination discovery
- [findhom docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findhom.html) -- Homogeneous relation discovery
- [findpoly docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findpoly.html) -- Polynomial relation discovery
- [findcong docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findcong.html) -- Congruence discovery
- [findnonhom docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findnonhom.html) -- Non-homogeneous relations
- [findhomcombo docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findhomcombo.html) -- Homogeneous combo
- [findnonhomcombo docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findnonhomcombo.html) -- Non-homogeneous combo
- [findmaxind docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findmaxind.html) -- Maximal independent subset
- [findprod docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findprod.html) -- Product search
- [findhommodp docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findhommodp.html) -- Homogeneous mod p
- [findlincombomodp docs](https://qseries.org/fgarvan/qmaple/qseries/functions/findlincombomodp.html) -- Linear combo mod p

### Secondary (MEDIUM confidence)
- [Garvan 1998 arXiv paper](https://arxiv.org/abs/math/9812092) -- "A q-product tutorial for a q-series MAPLE package" describing Andrews' algorithm
- [Springer publication](https://link.springer.com/chapter/10.1007/978-3-642-56513-7_4) -- Same paper, published version

### Tertiary (LOW confidence)
- Andrews' algorithm logarithmic derivative recurrence: n*b_n = sum c_j * b_{n-j} -- Reconstructed from multiple search results describing the algorithm. The exact formula needs validation against Garvan's paper or Andrews' original work.
- Mobius inversion step for recovering a_n from c_n -- Standard mathematical result but not directly verified from a single authoritative source for this specific application.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- No new dependencies; all on existing FPS/QRat infrastructure
- Architecture: HIGH -- Module structure follows established Phase 3 patterns; algorithm decomposition is clear
- Algorithm (prodmake): MEDIUM -- Core recurrence verified from multiple sources; implementation details inferred
- Algorithm (relation discovery): MEDIUM-HIGH -- Coefficient matrix + null space approach confirmed by Garvan docs; implementation straightforward
- Algorithm (etamake/jacprodmake): MEDIUM -- Post-processing of prodmake; grouping logic inferred from function behavior
- Pitfalls: HIGH -- Based on direct codebase inspection and Garvan documentation notes

**Research date:** 2026-02-13
**Valid until:** 2026-03-15 (stable mathematical algorithms; unlikely to change)
