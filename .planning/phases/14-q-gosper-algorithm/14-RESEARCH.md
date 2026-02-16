# Phase 14: q-Gosper Algorithm - Research

**Researched:** 2026-02-16
**Domain:** q-hypergeometric indefinite summation / computer algebra
**Confidence:** HIGH

## Summary

The q-Gosper algorithm is the q-analogue of Gosper's classical algorithm for indefinite hypergeometric summation. Given a q-hypergeometric term t_k (where t_{k+1}/t_k is a rational function of q^k), it decides whether there exists a q-hypergeometric antidifference s_k satisfying t_k = s_{k+1} - s_k, and produces it when one exists. The algorithm is the foundational subroutine for q-Zeilberger's algorithm (Phase 15).

The algorithm operates entirely in the polynomial domain Q(q)[x] where x represents q^k. It decomposes the term ratio into a "Gosper normal form" using q-dispersion (computed via resultant) and q-Greatest Factorial Factorization (qGFF), then solves a polynomial "key equation." All required infrastructure -- polynomial arithmetic, GCD, resultant, and q-shift -- already exists from Phase 13.

**Primary recommendation:** Implement as a new module `crates/qsym-core/src/qseries/gosper.rs` with five public functions mapping directly to GOSP-01 through GOSP-05, reusing Phase 13's QRatPoly and QRatRationalFunc types throughout.

## Standard Stack

### Core (existing -- no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| QRatPoly | Phase 13 | Dense polynomial arithmetic over Q | Already built, exact match for q-Gosper polynomials |
| poly_gcd | Phase 13 | Subresultant PRS GCD | Needed for q-dispersion decomposition loop |
| poly_resultant | Phase 13 | Euclidean resultant | Needed for q-dispersion set computation |
| q_shift / q_shift_n | Phase 13 | p(x) -> p(q^j * x) | Core operation in q-Gosper normal form |
| QRatRationalFunc | Phase 13 | Auto-simplifying rational functions | Term ratio representation |
| HypergeometricSeries | Phase 6 | Input type for q-Gosper | Term ratio extraction from params |
| QMonomial | Phase 3 | c * q^m representation | Hypergeometric parameters |
| FormalPowerSeries | Phase 2 | FPS for verification | Cross-check antidifferences by expansion |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rug | (existing) | Arbitrary-precision QRat | All polynomial coefficient arithmetic |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Dense QRatPoly | Sparse poly | Dense is better for low-degree polys typical in Gosper (deg 1-20); sparse would add complexity for no gain |
| Resultant for q-dispersion | Direct irreducible factorization | Resultant is simpler to implement and we already have it; factorization would be faster for large polys but unnecessary here |

## Architecture Patterns

### Recommended Project Structure
```
crates/qsym-core/src/qseries/
    gosper.rs          # NEW: q-Gosper algorithm (all 5 requirements)
    hypergeometric.rs  # EXISTING: HypergeometricSeries, QMonomial
    mod.rs             # UPDATE: pub mod gosper; pub use gosper::*;
```

### Pattern 1: Term Ratio Extraction (GOSP-01)

**What:** Given a `HypergeometricSeries` with upper params a_1,...,a_r, lower params b_1,...,b_s, and argument z, extract the term ratio t_{k+1}/t_k as a rational function of x = q^k.

**Mathematical basis:** For r_phi_s, the k-th term ratio is:

```
t_{k+1}/t_k = [prod_i (1 - a_i * x)] / [(1 - x) * prod_j (1 - b_j * x)]
              * (-1)^{1+s-r} * x^{1+s-r} * z
```

where each a_i = a_i.coeff * q^{a_i.power} is absorbed: if a_i = c_i * q^{m_i}, then the factor (a_i; q)_k contributes (1 - c_i * q^{m_i} * q^k) = (1 - c_i * q^{m_i} * x) to the ratio at step k. But this is NOT a polynomial in x -- it's a monomial c_i * q^{m_i} * x.

**Key insight:** The ratio t_{k+1}/t_k for a q-hypergeometric term is a rational function of x = q^k. Each Pochhammer factor (a;q)_k contributes the ratio factor (1 - a*q^k)/(1 - a*q^{k+1}) from step k to k+1... wait, more carefully:

For a terminating _r phi_s with upper param q^{-N} (so N is the termination order), the k-th term is:
```
t_k = [(a_1;q)_k * ... * (a_r;q)_k] / [(q;q)_k * (b_1;q)_k * ... * (b_s;q)_k]
      * [(-1)^k * q^{k(k-1)/2}]^{1+s-r} * z^k
```

The ratio t_{k+1}/t_k is then:
```
t_{k+1}/t_k = [prod_i (1 - a_i * q^k)] / [(1 - q^{k+1}) * prod_j (1 - b_j * q^k)]
              * (-1)^{1+s-r} * q^{k*(1+s-r)} * z
```

Setting x = q^k, this becomes a rational function in x:
```
r(x) = [prod_i (1 - a_i * x)] * [(-1)^{1+s-r} * z * x^{1+s-r}]
       / [(1 - q*x) * prod_j (1 - b_j * x)]
```

where a_i and b_j are the QMonomial parameter values (c * q^m as elements of Q(q)).

**Important subtlety:** The parameters a_i = c_i * q^{m_i} are elements of Q(q), not of Q. For the polynomial algorithm, we need to work in Q(q)[x]. Since q is a fixed indeterminate in our system, we can treat each coefficient c_i * q^{m_i} as a rational number by specializing q to a specific value, OR we can work symbolically.

**Recommended approach:** Since our QRatPoly works over Q, and q-Gosper needs polynomials over Q(q), we have two options:
1. **Parametric q:** Keep q symbolic and work with coefficients in Q(q) -- requires extending QRatPoly or using a different representation.
2. **Specialized q:** Pick a specific rational q value and work in Q[x] -- simpler but only works for specific q.

For the q-Gosper algorithm as used in computer algebra (Paule-Riese, Koornwinder, Koepf-Boeing), q is treated as an indeterminate throughout. The polynomials a(x), b(x) have coefficients that are rational functions of q.

**Critical design decision:** Our QRatPoly coefficients are QRat (rational numbers), not rational functions of q. For the q-Gosper algorithm, each "coefficient" in a(x) involves powers of q. Since we ultimately evaluate at specific q (our FPS variable), the cleanest approach is:

**Use QRatPoly with a fixed rational q parameter threaded through all operations.** The q-Gosper algorithm receives a concrete rational value for q (which could be any non-zero rational, but in practice is a formal parameter). Since our HypergeometricSeries parameters are QMonomial = {coeff: QRat, power: i64}, we can construct the ratio polynomials by treating q as a parameter:

- Factor (1 - a_i * x) where a_i = coeff * q^power becomes (1 - coeff * q_val^power * x), which is a linear polynomial in x over Q when q_val is a rational number.

BUT: q-Gosper is supposed to work symbolically for all q, not for specific q values. The correct approach is:

**Approach: Treat q as a symbolic parameter.** The polynomials in the Gosper algorithm have coefficients that are polynomials/rational functions of q. Rather than building a full Q(q)[x] infrastructure, we can:

1. Absorb q-powers into QRat coefficients by choosing q as a specific prime or transcendental-like value. This is the approach used by many CAS implementations (choosing q large enough that no accidental cancellations occur).

2. Better: Since our HypergeometricSeries already has params as QMonomial = {coeff: QRat, power: i64}, we can build polynomials in x whose coefficients track both a QRat scalar and a q-power. This is exactly what QMonomial already does.

**Final recommended approach:** Thread a concrete QRat `q_val` parameter throughout the algorithm. The algorithm operates on QRatPoly in x, where coefficients are concrete rationals derived from evaluating q-power expressions at `q_val`. The algorithm's correctness is independent of the specific q value (as long as q is not a root of unity), so we use a parameter q_val: &QRat.

This matches the pattern used in eval_phi: the FPS variable represents q, and concrete rational arithmetic is performed.

**When to use:** Extract term ratio from HypergeometricSeries before running the Gosper normal form decomposition.

**Example:**
```rust
/// Result of q-Gosper algorithm.
pub enum QGosperResult {
    /// The sum is q-Gosper summable; antidifference certificate provided.
    Summable {
        /// The rational function f(x)/g(x) such that the antidifference
        /// is (b(x-1) * f(x)) / (c(x)) * t_k, where x = q^k.
        certificate: QRatRationalFunc,
    },
    /// The sum is not q-Gosper summable.
    NotSummable,
}

/// Extract the term ratio t_{k+1}/t_k as a rational function of x = q^k.
///
/// Given a HypergeometricSeries, constructs polynomials a(x), b(x) such that
/// t_{k+1}/t_k = a(x) / b(x) where x = q^k.
///
/// The numerator is: prod_i (1 - a_i*x) * extra_sign * z * x^extra_power
/// The denominator is: (1 - q*x) * prod_j (1 - b_j*x)
pub fn extract_term_ratio(
    series: &HypergeometricSeries,
    q_val: &QRat,
) -> QRatRationalFunc {
    // Build numerator: product of (1 - a_i_eval * x) for each upper param
    // where a_i_eval = a_i.coeff * q_val^{a_i.power}
    let mut numer = QRatPoly::one();
    for a in &series.upper {
        let a_eval = eval_qmonomial(a, q_val);
        // (1 - a_eval * x) = linear poly with coeffs [1, -a_eval]
        let factor = QRatPoly::from_vec(vec![QRat::one(), -a_eval]);
        numer = &numer * &factor;
    }

    // Build denominator: (1 - q*x) * product of (1 - b_j_eval * x)
    let mut denom = QRatPoly::from_vec(vec![QRat::one(), -q_val.clone()]);
    for b in &series.lower {
        let b_eval = eval_qmonomial(b, q_val);
        let factor = QRatPoly::from_vec(vec![QRat::one(), -b_eval]);
        denom = &denom * &factor;
    }

    // Extra factor: (-1)^{1+s-r} * z_eval * x^{1+s-r}
    // This is a monomial in x that multiplies the numerator
    let extra = 1 + series.s() as i64 - series.r() as i64;
    let z_eval = eval_qmonomial(&series.argument, q_val);
    let sign = if extra % 2 == 0 { QRat::one() } else { -QRat::one() };
    let extra_coeff = &sign * &z_eval;

    if extra >= 0 {
        let extra_monomial = QRatPoly::monomial(extra_coeff, extra as usize);
        numer = &numer * &extra_monomial;
    } else {
        // Negative extra: x^{-|extra|} goes to denominator
        let denom_extra = QRatPoly::monomial(QRat::one(), (-extra) as usize);
        denom = &denom * &denom_extra;
        numer = numer.scalar_mul(&extra_coeff);
    }

    QRatRationalFunc::new(numer, denom)
}
```

### Pattern 2: q-Dispersion (GOSP-02)

**What:** Given polynomials a(x) and b(x) in Q[x], find all non-negative integers j such that gcd(a(x), b(q^j * x)) is non-trivial.

**Mathematical basis:** The q-dispersion set is:
```
Disp_q(a, b) = { j in Z_>=0 : gcd(a(x), b(q^j * x)) != 1 }
```

This is the q-analogue of the classical dispersion set where the additive shift n -> n+j is replaced by the multiplicative q-shift x -> q^j * x.

**Computation via resultant:**
```
R(j) = Res_x(a(x), b(q^j * x))
```

The key property: R(j) = 0 if and only if gcd(a(x), b(q^j * x)) is non-trivial.

But since j is an integer parameter (not a polynomial variable), we cannot directly compute R as a polynomial in j. Instead, we use the following approach:

**Algorithm for q-dispersion computation:**

1. Let deg(a) = m, deg(b) = n.
2. The maximum possible dispersion value is bounded: if a(x) has a root alpha and b(q^j * x) has a root beta, then alpha = q^j * beta, so j = log_q(alpha/beta). Since we work over Q, the roots might be algebraic, but the key observation is:

   **Bound on j:** For polynomials a(x) and b(x) of degrees m and n, the dispersion set is finite. Specifically, if a(x) = prod(x - alpha_i) and b(x) = prod(x - beta_j), then gcd(a(x), b(q^j*x)) is nontrivial iff some alpha_i = q^j * beta_k, i.e., iff alpha_i/beta_k = q^j for some i,k. Since there are only m*n such ratios, the dispersion set has at most m*n elements.

   **Practical bound:** Compute Res_x(a(x), b(y*x)) as a polynomial in y. Then find which y = q^j (for j = 0, 1, 2, ...) are roots. The maximum j to check is bounded by:
   - If a(x) and b(x) have integer/rational coefficients and we're evaluating q as a rational, then compute Res_x(a(x), b(y*x)) as a polynomial R(y) in y.
   - Then find j >= 0 such that R(q^j) = 0.
   - Since R(y) has degree at most m*n, there are at most m*n roots, so at most m*n values of j.
   - Alternatively, iterate j = 0, 1, 2, ... up to some bound and check gcd(a(x), b(q^j * x)) != 1.

**Recommended practical approach:**
Since we have a concrete q_val: &QRat, iterate j from 0 upward:
1. For each j, compute b_shifted = b.q_shift_n(q_val, j).
2. Compute g = poly_gcd(&a, &b_shifted).
3. If g is not constant (degree >= 1), add j to the dispersion set.
4. Stop when j exceeds the degree bound: j_max = degree(a) * degree(b) is a safe upper bound because the resultant R(y) has degree <= m*n, so it has at most m*n roots.

**More refined bound:** We can compute the resultant R(y) = Res_x(a(x), b(y*x)) as a polynomial in y by treating y as a parameter. Since b(y*x) has coefficients that are polynomials in y (specifically, the coefficient of x^i in b(y*x) is b_i * y^i), the resultant is a polynomial in y of degree at most m*n. Then R(q_val^j) = 0 gives us the dispersion set. But this requires a two-variable resultant computation which is more complex.

**Simplest correct approach:** Iterate j = 0, 1, ..., J_max where J_max = deg(a)*deg(b), computing gcd(a(x), b(q^j*x)) for each j. This is O(J_max * GCD_cost) which is very feasible for typical Gosper inputs (degrees 1-10).

```rust
/// Compute the q-dispersion set of two polynomials.
///
/// Returns the sorted set of non-negative integers j such that
/// gcd(a(x), b(q^j * x)) is non-trivial.
pub fn q_dispersion(
    a: &QRatPoly,
    b: &QRatPoly,
    q_val: &QRat,
) -> Vec<i64> {
    if a.is_zero() || b.is_zero() {
        return vec![];
    }
    let deg_a = a.degree().unwrap_or(0);
    let deg_b = b.degree().unwrap_or(0);
    let j_max = (deg_a * deg_b) as i64;

    let mut result = Vec::new();
    for j in 0..=j_max {
        let b_shifted = b.q_shift_n(q_val, j);
        let g = poly_gcd(a, &b_shifted);
        if !g.is_one() && !g.is_zero() {
            result.push(j);
        }
    }
    result
}
```

### Pattern 3: q-Greatest Factorial Factorization / Gosper Normal Form (GOSP-03)

**What:** Decompose the term ratio r(x) = a(x)/b(x) into the Gosper normal form:
```
r(x) = sigma(x) * c(q*x) / (tau(x) * c(x))
```
where gcd(sigma(x), tau(q^j * x)) = 1 for all j >= 1.

This is equivalent to finding polynomials sigma(x), tau(x), c(x) such that:
- r(x) = sigma(x)/tau(x) * c(q*x)/c(x)
- sigma and tau are "q-coprime": no common q-shifted roots

**Algorithm (q-analogue of Gosper Step 2):**

Given r(x) = a(x)/b(x) (reduced, gcd(a,b) = 1):

1. Initialize: sigma = a, tau = b, c = 1.
2. Compute the q-dispersion set S = {j >= 1 : gcd(sigma(x), tau(q^j * x)) != 1}.
3. While S is non-empty:
   a. Take j_max = max(S).
   b. Compute g = gcd(sigma(x), tau(q^{j_max} * x)).
   c. Update sigma: sigma = sigma / g.
   d. Update tau: tau = tau / g(q^{-j_max} * x).
      (i.e., divide tau by the "unshifted" version of g)
   e. Update c: c = c * prod_{i=0}^{j_max - 1} g(q^{-i} * x).
      (multiply c by the product of g shifted back by 0, 1, ..., j_max-1 steps)
   f. Recompute S with updated sigma, tau.

After this loop: gcd(sigma(x), tau(q^j * x)) = 1 for all j >= 1.

**Verification:** r(x) = sigma(x)/tau(x) * c(q*x)/c(x) should equal the original a(x)/b(x).

```rust
/// Gosper normal form decomposition.
///
/// Given a rational function r(x) = numer/denom (assumed coprime),
/// produces (sigma, tau, c) such that:
///   r(x) = sigma(x)/tau(x) * c(q*x)/c(x)
/// and gcd(sigma(x), tau(q^j*x)) = 1 for all j >= 1.
pub struct GosperNormalForm {
    pub sigma: QRatPoly,
    pub tau: QRatPoly,
    pub c: QRatPoly,
}

pub fn gosper_normal_form(
    numer: &QRatPoly,
    denom: &QRatPoly,
    q_val: &QRat,
) -> GosperNormalForm {
    // ... decomposition algorithm
}
```

### Pattern 4: Key Equation Solver (GOSP-04)

**What:** Given sigma(x), tau(x), c(x) from the Gosper normal form, find a polynomial f(x) satisfying:
```
sigma(x) * f(q*x) - tau(x) * f(x) = c(x)
```

This is the q-analogue of the classical Gosper key equation a(n)*f(n+1) - b(n-1)*f(n) = c(n).

**Degree bound:** Let d = deg(sigma), e = deg(tau), m = deg(c).

The degree of f is bounded by:
```
deg(f) <= m - max(d, e)
```

with the following refinement:
- If d != e: deg(f) = m - max(d, e).
- If d == e: Let sigma = s_d * x^d + ... and tau = t_e * x^e + ...
  The leading terms of sigma(x)*f(q*x) and tau(x)*f(x) must cancel.
  sigma(x)*f(q*x) has leading term s_d * lc(f) * q^{deg(f)} * x^{d+deg(f)}.
  tau(x)*f(x) has leading term t_e * lc(f) * x^{e+deg(f)}.
  For cancellation: s_d * q^{deg(f)} = t_e, so deg(f) = log_q(t_e/s_d).
  If log_q(t_e/s_d) is not a non-negative integer, try deg(f) = m - d (or m - d + 1).
  If no valid degree exists, the equation has no polynomial solution.

**Solution method:** Use undetermined coefficients.
1. Assume f(x) = f_0 + f_1*x + ... + f_D*x^D with unknown f_i.
2. Substitute into the key equation.
3. Expand and collect powers of x.
4. Equate coefficients to get a linear system over Q.
5. Solve for f_0, ..., f_D.

If the system has no solution, the series is not q-Gosper summable.

```rust
/// Solve the q-Gosper key equation:
///   sigma(x) * f(q*x) - tau(x) * f(x) = c(x)
///
/// Returns Some(f) if a polynomial solution exists, None otherwise.
pub fn solve_key_equation(
    sigma: &QRatPoly,
    tau: &QRatPoly,
    c: &QRatPoly,
    q_val: &QRat,
) -> Option<QRatPoly> {
    // Compute degree bound
    // Set up linear system
    // Solve via Gaussian elimination
}
```

### Pattern 5: Complete q-Gosper Algorithm (GOSP-05)

**What:** Combine all steps into the complete algorithm:

1. **Extract term ratio** (GOSP-01): t_{k+1}/t_k = a(x)/b(x).
2. **Reduce to coprime:** Ensure gcd(a,b) = 1 (QRatRationalFunc already does this).
3. **Gosper normal form** (GOSP-02 + GOSP-03): Decompose a/b into sigma/tau * c(qx)/c(x).
4. **Solve key equation** (GOSP-04): Find polynomial f satisfying sigma*f(qx) - tau*f(x) = c(x).
5. **Return result** (GOSP-05):
   - If f exists: Summable. The antidifference is s_k = (tau(x) * f(x) / c(x)) * t_k where x = q^k.
   - If no f: NotSummable.

**Verification:** For summable cases, verify that s_{k+1} - s_k = t_k by FPS expansion.

### Anti-Patterns to Avoid

- **Working symbolically in q when concrete q suffices:** The algorithm needs concrete q values to evaluate q-shifts on polynomials. Don't try to build a symbolic Q(q)[x] ring -- use QRatPoly with a fixed q_val parameter.

- **Unbounded dispersion search:** Always bound the j-search in q-dispersion by deg(a)*deg(b). Never loop without a bound.

- **Ignoring the q = root of unity case:** The algorithm assumes q is not a root of unity. For the key equation degree bound, q^j must be distinct for different j. Add a precondition check.

- **Building a general linear system solver from scratch:** Reuse the existing linalg module's RREF over QRat from Phase 4.

- **Forgetting to verify the decomposition:** Always verify that sigma(x)/tau(x) * c(qx)/c(x) equals the original ratio after the normal form decomposition.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Polynomial GCD | Custom GCD | `poly_gcd` from Phase 13 | Subresultant PRS already handles coefficient explosion |
| Polynomial resultant | Custom resultant | `poly_resultant` from Phase 13 | Euclidean algorithm over Q already correct |
| q-shift of polynomials | Manual coefficient scaling | `QRatPoly::q_shift_n` | Already O(n) and tested |
| Rational function simplification | Manual GCD and cancel | `QRatRationalFunc::new` | Auto-reduces via poly_gcd on construction |
| Linear system solving | Custom Gaussian elimination | `rational_null_space` / RREF from linalg.rs | Already handles exact QRat arithmetic |
| QMonomial evaluation | Manual power computation | Helper function | Just coeff * q_val^power |

**Key insight:** Phase 13 was specifically designed as infrastructure for this phase. Every polynomial operation needed by q-Gosper already exists. The implementation is pure algorithmic composition.

## Common Pitfalls

### Pitfall 1: q-Dispersion Bound Correctness
**What goes wrong:** The dispersion bound j_max = deg(a)*deg(b) could be too small in theory if the polynomial coefficients themselves depend on q in a way that creates additional roots.
**Why it happens:** When coefficients of a(x) and b(x) involve powers of q_val, the shifted polynomial b(q^j*x) can have coefficients that accidentally create new common factors.
**How to avoid:** Use the resultant-based approach as a secondary check. For the iterative approach, verify after the loop that no further dispersions exist by checking j_max + 1.
**Warning signs:** Normal form verification fails (sigma/tau * c(qx)/c(x) != original ratio).

### Pitfall 2: Key Equation Degree Bound Edge Cases
**What goes wrong:** When deg(sigma) == deg(tau) and the leading coefficient ratio lc(tau)/lc(sigma) is q^j for some non-negative integer j, there are two candidate degrees for f, and the wrong one may be tried first.
**Why it happens:** The degree bound formula has a case split that is easy to get wrong.
**How to avoid:** Try both candidate degrees (the "obvious" bound and the "cancellation" bound) if the first fails. The key equation system will simply be inconsistent if the degree is wrong.
**Warning signs:** A summable series is incorrectly reported as not summable.

### Pitfall 3: The c polynomial accumulation in the normal form loop
**What goes wrong:** The c polynomial must accumulate products of shifted GCD factors across multiple iterations. Getting the shift directions wrong corrupts the decomposition.
**Why it happens:** In the loop, when dispersion j is found and GCD g is extracted, c must be multiplied by g(q^0*x) * g(q^{-1}*x) * ... * g(q^{-(j-1)}*x). The negative shifts and the indexing are error-prone.
**How to avoid:** Verify the decomposition after each iteration: sigma(x)/tau(x) * c(qx)/c(x) should equal the original ratio at every step.
**Warning signs:** The ratio reconstruction check fails.

### Pitfall 4: Extra Factor in Term Ratio for Non-Balanced Series
**What goes wrong:** When 1 + s - r != 0, the extra factor (-1)^{1+s-r} * q^{k*(1+s-r)} * z contributes a monomial x^{1+s-r} to the term ratio. If 1+s-r < 0, this goes to the denominator.
**Why it happens:** Treating the extra factor as always part of the numerator.
**How to avoid:** Handle the sign of 1+s-r explicitly, putting the monomial in the correct polynomial.
**Warning signs:** Term ratio doesn't match eval_phi's ratio when verified numerically.

### Pitfall 5: Verification Against FPS
**What goes wrong:** The antidifference formula involves rational functions of q^k, which are hard to convert back to FPS directly.
**Why it happens:** The Gosper certificate is a rational function of x = q^k, not an FPS.
**How to avoid:** For verification, evaluate the antidifference at k = 0, 1, ..., N and check that s_{k+1} - s_k = t_k for each k. Use the concrete q_val to compute numerically.
**Warning signs:** Verification fails on known-summable test cases.

## Code Examples

### QMonomial Evaluation Helper
```rust
/// Evaluate a QMonomial at a specific q value.
/// QMonomial { coeff: c, power: m } -> c * q_val^m
fn eval_qmonomial(mono: &QMonomial, q_val: &QRat) -> QRat {
    if mono.power == 0 {
        return mono.coeff.clone();
    }
    let q_pow = qrat_pow_signed(q_val, mono.power);
    &mono.coeff * &q_pow
}
```

### Term Ratio Construction
```rust
/// Build the numerator polynomial of the term ratio.
///
/// For _r phi_s with upper params a_1,...,a_r and argument z:
/// numer(x) = prod_i(1 - a_i_eval * x) * sign * z_eval * x^{max(0, extra)}
fn build_term_ratio_numer(
    series: &HypergeometricSeries,
    q_val: &QRat,
) -> QRatPoly {
    let mut numer = QRatPoly::one();
    for a in &series.upper {
        let a_eval = eval_qmonomial(a, q_val);
        let factor = QRatPoly::linear(QRat::one(), -a_eval);
        numer = &numer * &factor;
    }
    // Handle extra factor...
    numer
}
```

### q-Dispersion Computation
```rust
/// Compute q-dispersion set: all j >= 0 where gcd(a(x), b(q^j*x)) != 1.
pub fn q_dispersion(a: &QRatPoly, b: &QRatPoly, q_val: &QRat) -> Vec<i64> {
    let deg_a = a.degree().unwrap_or(0);
    let deg_b = b.degree().unwrap_or(0);
    let j_max = (deg_a * deg_b) as i64;

    (0..=j_max)
        .filter(|&j| {
            let b_shifted = b.q_shift_n(q_val, j);
            let g = poly_gcd(a, &b_shifted);
            g.degree().unwrap_or(0) >= 1
        })
        .collect()
}
```

### Gosper Normal Form Decomposition
```rust
pub fn gosper_normal_form(
    a: &QRatPoly,
    b: &QRatPoly,
    q_val: &QRat,
) -> GosperNormalForm {
    let mut sigma = a.clone();
    let mut tau = b.clone();
    let mut c = QRatPoly::one();

    loop {
        // Find q-dispersion set for j >= 1
        let disp = q_dispersion_positive(&sigma, &tau, q_val);
        if disp.is_empty() {
            break;
        }
        let j = *disp.last().unwrap(); // largest dispersion

        // g = gcd(sigma(x), tau(q^j * x))
        let tau_shifted = tau.q_shift_n(q_val, j);
        let g = poly_gcd(&sigma, &tau_shifted);

        // Update sigma, tau, c
        sigma = sigma.exact_div(&g);
        let g_unshifted = g.q_shift_n(q_val, -j);
        tau = tau.exact_div(&g_unshifted);

        // c *= prod_{i=0}^{j-1} g(q^{-i} * x)
        for i in 0..j {
            let g_shifted = g.q_shift_n(q_val, -i);
            c = &c * &g_shifted;
        }
    }

    GosperNormalForm { sigma, tau, c }
}
```

### Key Equation Solving
```rust
/// Solve sigma(x)*f(qx) - tau(x)*f(x) = c(x) for polynomial f(x).
pub fn solve_key_equation(
    sigma: &QRatPoly,
    tau: &QRatPoly,
    c_poly: &QRatPoly,
    q_val: &QRat,
) -> Option<QRatPoly> {
    let d_sigma = sigma.degree()?;
    let d_tau = tau.degree()?;
    let d_c = c_poly.degree()?;

    // Compute candidate degree for f
    let deg_f = compute_degree_bound(sigma, tau, c_poly, q_val)?;

    // Set up undetermined coefficients: f = f_0 + f_1*x + ... + f_D*x^D
    // Substitute into equation and collect coefficients of x^i
    // This gives a (d_c + 1) x (deg_f + 1) linear system

    // Build the matrix and solve
    let n_unknowns = deg_f + 1;
    let n_equations = d_c + 1;

    // ... build coefficient matrix ...
    // ... solve via RREF ...

    // If solution exists, construct f from coefficients
    Some(QRatPoly::from_vec(solution_coeffs))
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual summation formulas | Algorithmic Gosper/q-Gosper | 1978 / 1993-1997 | Mechanized indefinite summation |
| Symbolic q in Q(q)[x] | Concrete q_val in Q[x] | Standard CAS practice | Simpler implementation, same results |
| Full qGFF decomposition | Iterative GCD-based normal form | Paule-Riese 1997 | Conceptually cleaner, same result |

**Key references:**
- Gosper (1978): Original hypergeometric summation algorithm
- Koornwinder (1993): Rigorous q-Gosper description
- Paule-Riese (1997): qGFF-based approach, Mathematica implementation
- Koepf-Boeing (1998): Maple implementation in qsum package
- Petkovsek-Wilf-Zeilberger "A=B" (1996): Textbook treatment

## Open Questions

1. **q_val Parameter Choice**
   - What we know: The algorithm works for any non-root-of-unity q. Using a concrete q_val (like 2/3) produces correct results.
   - What's unclear: Whether we should require users to pass q_val or default to something.
   - Recommendation: Accept q_val as a required parameter. For testing, use q_val = 2 or 3 (small integers that are not roots of unity and won't cause accidental cancellations).

2. **Degree Bound Tightness**
   - What we know: The degree bound deg(f) <= deg(c) - max(deg(sigma), deg(tau)) is correct in the non-degenerate case.
   - What's unclear: The exact handling when deg(sigma) == deg(tau) and the leading coefficient ratio is a q-power.
   - Recommendation: Implement the two-case formula and fall back to trying deg_f + 1 if the first attempt fails (this is standard practice in CAS implementations).

3. **Dispersion Bound for Large Degrees**
   - What we know: j_max = deg(a)*deg(b) is a correct upper bound.
   - What's unclear: Whether a tighter bound exists for our typical inputs (linear factors from Pochhammer symbols).
   - Recommendation: Use the simple bound. For typical q-hypergeometric inputs, degrees are small (2-10) so the bound is 4-100, which is fast.

4. **Integration with Existing try_all_summations**
   - What we know: Phase 6 has try_all_summations that tries pattern-matching summation formulas.
   - What's unclear: Whether q-Gosper should be added to that function or remain separate.
   - Recommendation: Keep separate. q-Gosper is algorithmic (works on any input), while try_all_summations is pattern-based (works on specific forms). They serve different purposes.

5. **Antidifference Representation**
   - What we know: The antidifference is s_k = (tau(x)*f(x)/c(x)) * t_k where x = q^k.
   - What's unclear: The best way to represent this in Rust (as a QRatRationalFunc, as an FPS, or as a structured type).
   - Recommendation: Return a structured GosperCertificate containing the rational function and the original series reference. Let the caller decide how to evaluate it.

## Sources

### Primary (HIGH confidence)
- Phase 13 codebase: `crates/qsym-core/src/poly/` -- QRatPoly, poly_gcd, poly_resultant, q_shift verified by reading source
- Phase 6 codebase: `crates/qsym-core/src/qseries/hypergeometric.rs` -- HypergeometricSeries, eval_phi verified by reading source
- [AeqB-sage Gosper implementation](https://github.com/benyoung/AeqB-sage/blob/master/gosper.sage) -- Confirmed algorithm structure: step2 (dispersion decomposition), step3 (key equation with degree bound)
- [MathWorld - Gosper's Algorithm](https://mathworld.wolfram.com/GospersAlgorithm.html) -- Confirmed 4-step structure: ratio, decompose, key equation, assemble

### Secondary (MEDIUM confidence)
- [Paule-Riese 1997 (RISC)](https://www3.risc.jku.at/publications/download/risc_117/Paule_Riese.pdf) -- PDF confirmed to exist; content about qGFF-based q-Gosper algorithm verified by web search descriptions
- [Koornwinder 1993](https://staff.fnwi.uva.nl/t.h.koornwinder/art/1993/zeilbalgo.pdf) -- Rigorous q-Gosper description confirmed by web search
- [Koepf - Hypergeometric Summation](https://link.springer.com/book/10.1007/978-1-4471-6464-7) -- Chapter 5 confirmed to cover Gosper + q-Gosper
- [Algorithms for q-hypergeometric summation (Koepf-Boeing 1998)](https://www.sciencedirect.com/science/article/pii/S074771719890339X) -- Maple qgosper implementation

### Tertiary (LOW confidence)
- Degree bound formula for the case deg(sigma)==deg(tau): reconstructed from training data on A=B book and Koepf. Needs validation during implementation with test cases.
- q-dispersion bound j_max = deg(a)*deg(b): standard result but not verified from 2026 source. Very likely correct based on classical dispersion theory.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All required infrastructure exists in Phase 13 and Phase 6, verified by reading source code
- Architecture: HIGH - The q-Gosper algorithm is well-established (Koornwinder 1993, Paule-Riese 1997); the pattern of decompose-then-solve is universally used
- Term ratio extraction: HIGH - Directly follows from the definition of _r phi_s, verified against eval_phi source
- q-Dispersion: MEDIUM-HIGH - Algorithm is standard (iterate j, check GCD); bound is from classical theory
- Normal form decomposition: MEDIUM - Algorithm reconstructed from multiple sources; the c accumulation step needs careful implementation
- Key equation degree bound: MEDIUM - Two-case formula reconstructed from training data; needs validation with test cases
- Pitfalls: HIGH - Common failure modes identified from algorithm structure

**Research date:** 2026-02-16
**Valid until:** 2026-04-16 (stable mathematical algorithms, no version churn)
