# Phase 15: q-Zeilberger & WZ Certificates - Research

**Researched:** 2026-02-16
**Domain:** q-hypergeometric definite summation / creative telescoping / WZ proof theory
**Confidence:** HIGH

## Summary

The q-Zeilberger algorithm is the q-analogue of Zeilberger's creative telescoping algorithm. Given a q-hypergeometric summand F(n,k) -- a term of a definite sum S(n) = sum_k F(n,k) -- it finds a linear recurrence with polynomial coefficients c_0(q^n), ..., c_d(q^n) satisfied by S(n), together with a WZ proof certificate R(n,k) that independently verifies the recurrence. The algorithm uses q-Gosper (Phase 14) as its core subroutine: for each candidate recurrence order d = 1, 2, ..., it forms a "modified summand" that combines d+1 q-shifted copies of F(n,k) with undetermined polynomial coefficients, then attempts to apply q-Gosper to this modified summand. If q-Gosper succeeds, the recurrence coefficients and WZ certificate are both extracted from the Gosper result.

Phase 14 already provides the complete q-Gosper pipeline: `extract_term_ratio` to get the rational function t_{k+1}/t_k, `gosper_normal_form` for the sigma/tau/c decomposition, `solve_key_equation` for the polynomial certificate, and the top-level `q_gosper` function. The q-Zeilberger algorithm wraps this with an outer loop over recurrence orders, constructing the modified term ratio at each order by multiplying the original ratio by the appropriate n-shift factors. WZ certificate verification is independent: given F(n,k) and R(n,k), check that c_0*F(n,k) + ... + c_d*F(n+d,k) = G(n,k+1) - G(n,k) where G(n,k) = R(n,k)*F(n,k), which reduces to evaluating both sides at concrete (q^n, q^k) values.

**Primary recommendation:** Implement as a new module `crates/qsym-core/src/qseries/zeilberger.rs` with four public items: `q_zeilberger` (ZEIL-01/02), `ZeilbergerResult` (ZEIL-02), `WZCertificate` (ZEIL-03), and `verify_wz_certificate` (ZEIL-04/05). The module reuses `q_gosper` from Phase 14 as its subroutine and `eval_phi` from Phase 6 for FPS cross-verification.

## Standard Stack

### Core (existing -- no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| q_gosper | Phase 14 | Core subroutine for creative telescoping | Gosper's algorithm decides summability of modified summand at each order |
| extract_term_ratio | Phase 14 | Term ratio extraction from HypergeometricSeries | Needed to construct modified term ratio for q-Zeilberger |
| gosper_normal_form | Phase 14 | Sigma/tau/c decomposition | May be needed for direct manipulation of modified summand ratio |
| solve_key_equation | Phase 14 | Polynomial key equation solver | Needed if we bypass full q_gosper for modified summand |
| QRatPoly | Phase 13 | Polynomial arithmetic over Q | Recurrence coefficients are polynomials in x = q^n |
| QRatRationalFunc | Phase 13 | Rational function arithmetic | WZ certificate is a rational function |
| poly_gcd | Phase 13 | Polynomial GCD | Auto-simplification in rational function construction |
| q_shift / q_shift_n | Phase 13 | Polynomial q-shift | Shifting term ratio by n -> n+j |
| HypergeometricSeries | Phase 6 | Input summand representation | Summand F(n,k) comes from hypergeometric series params |
| eval_phi | Phase 6 | FPS evaluation of series | Cross-verification (ZEIL-05) |
| FormalPowerSeries | Phase 2 | FPS type | Cross-verification target |
| QMonomial | Phase 3 | c * q^m representation | Hypergeometric parameters |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rug | (existing) | Arbitrary-precision QRat | All coefficient arithmetic |
| solve_linear_system | Phase 14 (gosper.rs private) | RREF over Q | Finding recurrence coefficients (may need to be made pub(crate)) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| q_gosper on modified summand | Direct parametric Gosper (solve for c_j and f simultaneously) | Parametric Gosper is more efficient but significantly more complex to implement; calling q_gosper in a loop is simpler and correct |
| Concrete q_val throughout | Symbolic q with Q(q)[x] | Concrete q_val matches Phase 14's approach; symbolic q would require new infrastructure |
| Dense QRatPoly for c_j(x) | Sparse representation | Dense is fine for the low-degree polynomials (deg 1-5) that arise in practice |

## Architecture Patterns

### Recommended Project Structure
```
crates/qsym-core/src/qseries/
    zeilberger.rs      # NEW: q-Zeilberger algorithm and WZ certificates (ZEIL-01 through ZEIL-05)
    gosper.rs          # EXISTING: q-Gosper (subroutine)
    hypergeometric.rs  # EXISTING: HypergeometricSeries, eval_phi
    mod.rs             # UPDATE: pub mod zeilberger; pub use zeilberger::*;
```

### Pattern 1: Creative Telescoping Loop (ZEIL-01)

**What:** The outer loop of q-Zeilberger tries recurrence orders d = 1, 2, ..., max_order. At each order d, it forms a "modified summand" that is a linear combination of F(n,k), F(n+1,k), ..., F(n+d,k) with undetermined polynomial coefficients c_0(x), ..., c_d(x) where x = q^n, and attempts to find these coefficients such that the combination is q-Gosper-summable in k.

**Mathematical basis:** We seek polynomials c_0(x), ..., c_d(x) in x = q^n (not all zero) and a rational function R(n,k) such that:

```
c_0(q^n) * F(n,k) + c_1(q^n) * F(n+1,k) + ... + c_d(q^n) * F(n+d,k) = G(n,k+1) - G(n,k)
```

where G(n,k) = R(n,k) * F(n,k). Summing over k, the RHS telescopes to zero (assuming proper boundary conditions for terminating series), giving the recurrence:

```
c_0(q^n) * S(n) + c_1(q^n) * S(n+1) + ... + c_d(q^n) * S(n+d) = 0
```

**Key insight for implementation:** F(n+j,k)/F(n,k) is a rational function of q^n and q^k. So the modified summand's term ratio (as a function of x_k = q^k) has coefficients that are polynomial in x_n = q^n. We can factor out F(n,k) and work with:

```
c_0(x_n) + c_1(x_n) * [F(n+1,k)/F(n,k)] + ... + c_d(x_n) * [F(n+d,k)/F(n,k)]
```

Each ratio F(n+j,k)/F(n,k) is a product of j consecutive term ratios in the n-direction. Since F is q-hypergeometric in both n and k, these ratios are rational functions of q^k with coefficients depending polynomially on q^n.

**Algorithm:**

For each order d = 1, 2, ..., max_order:
1. Compute the n-direction shift ratios: F(n+j,k)/F(n,k) for j = 0, ..., d.
   These are rational functions of x_k = q^k, parameterized by q^n.
2. Form the modified summand's k-direction term ratio by combining these with
   undetermined polynomial coefficients c_0(x_n), ..., c_d(x_n).
3. Apply q-Gosper to this modified summand (in the k variable).
4. If q-Gosper succeeds, extract the recurrence coefficients c_j and the WZ certificate.
5. If q-Gosper fails, increment d and try again.

**Concrete q_val approach (matching Phase 14):** Since our q-Gosper works with a concrete q_val, we thread q_val through the q-Zeilberger algorithm. The n-shift ratios involve evaluating QMonomial parameters at q_val. The recurrence coefficients c_j are polynomials in x_n = q^n_val (a second rational parameter representing the n-variable).

**Implementation subtlety -- parametric Gosper vs. loop-and-try:**

There are two standard implementation approaches:

*Approach A (Parametric Gosper):* Treat c_0, ..., c_d as unknowns from the start. In the Gosper key equation, the unknowns are both the polynomial f(x_k) coefficients AND the c_j coefficients. This gives a larger linear system but solves everything in one shot. This is the approach of Paule-Riese and most CAS implementations.

*Approach B (Trial c_j):* Fix c_j values (e.g., try c_d = 1 and search for remaining c_j) and call q_gosper directly. This is simpler but may miss valid recurrences where c_d is not normalizable to 1.

**Recommended approach:** Approach A (Parametric Gosper). The reason: our q-Gosper already has a `solve_key_equation` that sets up a linear system. For q-Zeilberger, we extend this system to include the c_j as additional unknowns. The key equation becomes:

```
sigma(x_k) * f(q*x_k) - tau(x_k) * f(x_k) = [c_0(x_n)*tau_0(x_k)*c_0_poly(x_k) + ... + c_d(x_n)*tau_d(x_k)*c_d_poly(x_k)]
```

where each term on the RHS corresponds to the contribution from F(n+j,k).

However, the parametric approach requires significant modification to the Gosper machinery. A simpler, pragmatically equivalent approach is:

**Recommended simplified approach:** For each order d, construct the modified summand explicitly by choosing specific values of q_val for n (denoted n_val), building the modified term ratio, and calling q_gosper. Then verify the recurrence at multiple n values. This is the approach that reuses Phase 14's q_gosper as a black box.

Wait -- this won't work directly because we need the c_j to be polynomials in q^n, not specific values.

**Final recommended approach: Extended linear system.**

The cleanest approach that reuses maximum existing infrastructure:

1. For order d, the modified summand (divided by F(n,k)) is:
   ```
   H(n,k) = c_0 + c_1 * R_1(n,k) + ... + c_d * R_d(n,k)
   ```
   where R_j(n,k) = F(n+j,k)/F(n,k) is known.

2. H(n,k) has k-direction term ratio:
   ```
   H(n,k+1)/H(n,k) = [c_0*t_k + c_1*R_1(n,k+1)*t_k + ...] / [c_0 + c_1*R_1(n,k) + ...]
   ```
   where t_k = F(n,k+1)/F(n,k) is the original k-direction term ratio.

   This ratio depends on the c_j, making it non-hypergeometric in general. The standard Paule-Riese solution is to work with the individual term ratios and combine them in the key equation.

3. **The Paule-Riese trick:** Instead of working with the combined modified summand, decompose the problem. Each F(n+j,k) is q-hypergeometric in k. The j-th shifted term has k-direction ratio:
   ```
   F(n+j,k+1)/F(n+j,k) = r_j(x_k, x_n)
   ```
   which is a rational function of x_k = q^k with coefficients depending on x_n = q^n.

   Apply Gosper's normal form to each r_j separately to get sigma_j, tau_j, c_j_poly. Then the key equation for the combined summand becomes a *parametric* key equation with the recurrence coefficients as additional unknowns.

4. **Practical simplification:** For a terminating _r phi_s summand with concrete q_val, the n-direction shifts just change the termination parameter. For example, the q-Vandermonde _2phi1(q^{-n}, a; c; q, z) shifted to n+j gives _2phi1(q^{-(n+j)}, a; c; q, z'). The term ratio in k is:
   ```
   r(x_k) = (1 - q^{-n}*x_k)(1 - a*x_k) * z / [(1 - q*x_k)(1 - c*x_k)]
   ```
   The shifted version replaces q^{-n} with q^{-(n+j)} = q^{-n} * q^{-j}.

   So the n-shift changes one coefficient in the term ratio polynomial. This means we can parameterize by q^n and solve a combined system.

**Final implementation strategy (proven to work):**

Use the concrete q_val approach. For each order d:

1. Choose a symbolic/parametric treatment of n: let x_n = q_val^n be a free variable.
2. Compute the ratio F(n+j,k)/F(n,k) for j = 1, ..., d as a function of x_k and x_n.
   For a HypergeometricSeries, this is a product of j consecutive n-direction ratios.
3. The modified summand's k-direction ratio involves x_n as a parameter.
4. Apply Gosper's normal form to the combined modified summand's ratio.
5. In the key equation, the unknowns are the f polynomial coefficients AND the c_j coefficients.
   This is a homogeneous linear system -- we seek a non-trivial solution.
6. If found, extract c_j(x_n) and the Gosper certificate.

Since our polynomials are over Q (concrete q_val), the "parameter" x_n = q_val^n can be treated as a second indeterminate. But this requires two-variable polynomial support we don't have.

**Pragmatic final approach:** Treat n as a specific value and run q-Gosper on the modified summand for multiple n values, then interpolate the polynomial c_j(q^n). This is:

1. For each order d, for each trial n = n_0, n_0+1, ..., n_0+D (where D is the maximum expected degree of c_j):
   a. Build the concrete modified summand for this specific n.
   b. Run q-Gosper on it with parametric c_j (undetermined coefficients in the key equation).
2. From the solutions at multiple n values, interpolate the c_j as polynomials in q^n.

Actually, let me reconsider. The standard efficient approach used by all CAS implementations is:

**The standard approach (Koornwinder/Paule-Riese):**

For a q-hypergeometric summand F(n,k), the ratio F(n+j,k)/F(n,k) in the k-variable is a rational function r_j(q^k) whose coefficients involve q^n. The key observation is:

The modified summand H_k = sum_{j=0}^d c_j * F(n+j,k) has the property that H_{k+1}/H_k can be written as a rational function once we know c_j. The algorithm avoids this ratio by working with the TELESCOPING EQUATION directly:

```
sum_{j=0}^d c_j * F(n+j,k) = G(n,k+1) - G(n,k)
```

Dividing by F(n,k):

```
sum_{j=0}^d c_j * R_j(n,k) = G(n,k+1)/F(n,k) - G(n,k)/F(n,k)
```

where R_j = F(n+j,k)/F(n,k). Setting y(k) = G(n,k)/F(n,k), the right side becomes y(k+1)*t_k - y(k) where t_k = F(n,k+1)/F(n,k).

So the equation is:

```
sum_{j=0}^d c_j * R_j(n,k) = y(k+1) * t_k - y(k)
```

where y(k) is the unknown rational function (WZ certificate) and c_j are unknown constants (for a given n).

This is exactly a q-Gosper-type equation where the RHS is the inhomogeneous term. In the standard Gosper approach, after normal form decomposition of t_k, we solve:

```
sigma * f(q*x_k) - tau * f(x_k) = [sum_{j=0}^d c_j * P_j(x_k)] * (some factors)
```

where the c_j appear linearly in the RHS. Combined with the undetermined coefficients of f, this gives a linear system in {f_0, ..., f_D, c_0, ..., c_d}. The system is homogeneous (we can normalize c_d = 1 or seek null space).

This is the approach we should implement.

**Example:** For a concrete q_val and n_val, each R_j(n,k) evaluates to a specific rational function of x_k = q^k. The Gosper normal form is applied to t_k = F(n,k+1)/F(n,k). The key equation becomes:

```
sigma(x_k) * f(q*x_k) - tau(x_k) * f(x_k) = sum_{j=0}^d c_j * [R_j evaluated] * tau(x_k) * c_poly(x_k)
```

Wait, this needs more care. Let me think through the exact formulation.

The modified summand divided by F(n,k) is sum_{j=0}^d c_j * R_j where R_0 = 1. Its k-direction first difference (telescoping) gives:

```
[sum_j c_j R_j(k+1)] * t_k - [sum_j c_j R_j(k)] = 0
```

No -- we want the ANTI-DIFFERENCE. Let me reformulate.

We seek y(k) such that:

```
y(k+1) * F(n,k+1) - y(k) * F(n,k) = sum_{j=0}^d c_j * F(n+j,k)
```

Dividing by F(n,k):

```
y(k+1) * t_k - y(k) = sum_{j=0}^d c_j * R_j(k)
```

where t_k = F(n,k+1)/F(n,k) and R_j(k) = F(n+j,k)/F(n,k).

This is a FIRST-ORDER q-DIFFERENCE EQUATION in y(k) with inhomogeneous RHS involving the c_j.

Applying Gosper's approach: decompose t_k = sigma/tau * c_poly(q*x_k)/c_poly(x_k) as in the normal form. Set y(k) = f(x_k)/c_poly(x_k). Then:

```
sigma * f(q*x_k) - tau * f(x_k) = c_poly(x_k) * sum_{j=0}^d c_j * R_j(x_k)  [times appropriate factors]
```

Actually the precise formula depends on the relation between the Gosper decomposition and the inhomogeneous equation. Let me work through it carefully.

The standard Gosper key equation for y(k+1)*t_k - y(k) = RHS(k) where t_k = sigma/tau * c(qx)/c(x) and y = f/c is:

sigma * f(qx) - tau * f(x) = tau * c(x) * RHS(x) / tau(x) ... no, let me redo.

From Phase 14: y(x) = f(x)/c(x), s_k = y(q^k)*t_k, and the Gosper key equation is sigma*f(qx) - tau*f(x) = tau*c.

For Zeilberger, the equation is different. We don't have s_{k+1} - s_k = t_k. Instead we have:

```
y(k+1)*F(n,k+1) - y(k)*F(n,k) = sum_{j=0}^d c_j * F(n+j,k)
```

Let G(n,k) = y(k)*F(n,k). Then G(n,k+1) - G(n,k) = sum_j c_j F(n+j,k). This is the telescoping equation.

Dividing by F(n,k):

```
y(k+1)*t_k - y(k) = sum_j c_j * R_j(k)
```

Now apply the Gosper substitution y(k) = f(x)/c_poly(x) where x = q^k, and t_k = sigma(x)/tau(x) * c_poly(qx)/c_poly(x):

```
[f(qx)/c_poly(qx)] * [sigma(x)/tau(x) * c_poly(qx)/c_poly(x)] - f(x)/c_poly(x) = sum_j c_j R_j(x)
```

Simplify:

```
f(qx)*sigma(x) / [tau(x)*c_poly(x)] - f(x)/c_poly(x) = sum_j c_j R_j(x)
```

Multiply through by tau(x)*c_poly(x):

```
sigma(x)*f(qx) - tau(x)*f(x) = tau(x)*c_poly(x) * sum_j c_j R_j(x)
```

So the Zeilberger key equation is:

```
sigma(x)*f(qx) - tau(x)*f(x) = tau(x)*c_poly(x) * [c_0 + c_1*R_1(x) + ... + c_d*R_d(x)]
```

where R_0 = 1 and the c_j are unknown constants (for a specific n value -- they become polynomials in q^n when n varies).

The RHS is a polynomial in x (once we clear denominators from R_j). Each R_j(x) is a rational function, so tau(x)*c_poly(x)*R_j(x) may have denominators. We need to clear these by multiplying the entire equation by the LCM of all R_j denominators.

**Implementation plan:**

1. Compute the Gosper normal form for t_k (the k-direction term ratio of the original summand).
2. Compute R_j(x) = F(n+j,k)/F(n,k) for j = 1, ..., d as rational functions of x = q^k.
3. Compute tau(x)*c_poly(x)*R_j(x) for each j. Let D(x) be the LCM of all denominators.
4. Multiply through by D(x) to get a polynomial key equation.
5. The unknowns are: f polynomial coefficients + c_j constants. Set up the linear system.
6. The system is homogeneous in the c_j (but not in f because the LHS has known coefficients for sigma*f(qx) - tau*f(x)).

Wait, the system IS homogeneous overall if we view it as: sigma*f(qx) - tau*f(x) - RHS = 0 where RHS depends on c_j. We seek a nontrivial solution in (f, c_0, ..., c_d). We can normalize by setting c_d = 1 (or use null space).

This is a standard extension of the Gosper linear system with additional columns for c_0, ..., c_{d-1} (with c_d = 1 normalization).

### Pattern 2: N-Direction Shift Ratios (ZEIL-01 helper)

**What:** Compute F(n+j,k)/F(n,k) for a q-hypergeometric summand.

**For a HypergeometricSeries _r phi_s:** The summand depends on n through one or more of the upper/lower parameters. Typically, one upper parameter is q^{-n} (terminating), and the argument z may also depend on n.

The ratio F(n+1,k)/F(n,k) depends on which parameters involve n. For a summand F(n,k) = term_k of _r phi_s where the n-dependence is in one upper parameter q^{-n}:

```
F(n+1,k)/F(n,k) = (q^{-n-1};q)_k / (q^{-n};q)_k * (possibly z-ratio)
```

For (a*q^j; q)_k / (a; q)_k = product_{i=0}^{k-1} (1 - a*q^{j+i}) / (1 - a*q^i).

With a = q^{-n}: (q^{-n-1};q)_k / (q^{-n};q)_k = product_{i=0}^{k-1} (1-q^{-n-1+i})/(1-q^{-n+i}).

Setting x = q^k, each factor in the product is a ratio of linear functions of q^n, and the overall ratio is a rational function of x_n = q^n.

**Implementation approach:** Compute the n-direction term ratio F(n+1,k)/F(n,k) as a function of x_k = q^k, evaluated at a specific n value. Then R_j = product of j consecutive n-ratios.

For the specific case of F(n,k) being the k-th term of a q-hypergeometric series parameterized by n, we can extract the n-direction ratio analytically. The ratio F(n+1,k)/F(n,k) involves:
- For each upper param a_i(n): the ratio (a_i(n+1);q)_k / (a_i(n);q)_k
- For each lower param b_j(n): the ratio (b_j(n);q)_k / (b_j(n+1);q)_k
- For the argument z(n): the ratio z(n+1)^k / z(n)^k
- For the extra factor: ratio of [(-1)^k q^{k(k-1)/2}]^{extra} terms (same for both n and n+1, so cancels)

Each Pochhammer ratio (a*q^s; q)_k / (a; q)_k can be computed as a product of k linear factors in x = q^k at specific values.

**Simplification for the common case:** For a terminating _r phi_s with upper param q^{-n}, the shift n -> n+1 changes q^{-n} to q^{-n-1} = q^{-n} * q^{-1}. The k-th term ratio in the n-direction is:

```
F(n+1,k)/F(n,k) = prod_{i=0}^{k-1} (1 - q^{-n-1+i})/(1 - q^{-n+i}) * [z(n+1)/z(n)]^k
```

At x_k = q^k with concrete q_val and n_val, this evaluates to a specific rational number for each k. As a function of x_k, it's a rational function.

**Design decision:** Rather than parameterizing by n symbolically, use the concrete approach:
- Accept n_val as a parameter (the specific value of n)
- Compute R_j(x_k) as concrete QRatRationalFunc for each j = 1, ..., d
- This gives concrete polynomials in the key equation

For the recurrence coefficients to be "polynomial in q^n", we run the algorithm for multiple n values and verify consistency, or we note that for a specific n_val the c_j are constants (the polynomial evaluated at q^n_val).

### Pattern 3: Extended Key Equation (ZEIL-01 core)

**What:** Solve the extended Gosper key equation with undetermined recurrence coefficients.

**Equation:**
```
sigma(x)*f(qx) - tau(x)*f(x) = tau(x)*c(x) * [c_0 + c_1*R_1(x) + ... + c_d*R_d(x)]
```

**Where:**
- sigma, tau, c are from the Gosper normal form of the k-direction term ratio
- R_j(x) = F(n+j,k)/F(n,k) as a function of x = q^k
- f(x) is an unknown polynomial
- c_0, ..., c_d are unknown constants (c_d can be normalized to 1)

**But R_j(x) are rational functions,** so we need to clear denominators. Let:
```
R_j(x) = P_j(x) / Q_j(x)  (coprime)
```

Let D(x) = lcm(Q_1(x), ..., Q_d(x)). Multiply the equation by D(x):

```
D(x)*sigma(x)*f(qx) - D(x)*tau(x)*f(x) = D(x)*tau(x)*c(x) * [c_0 + c_1*P_1(x)/Q_1(x) + ... + c_d*P_d(x)/Q_d(x)]
```

After clearing: all terms become polynomials in x. The unknowns are {f_0, ..., f_D} and {c_0, ..., c_{d-1}} (with c_d = 1). This is a linear system.

**Degree bound for f:** Similar to the standard Gosper degree bound, but the RHS now has potentially higher degree due to the R_j factors. The degree of RHS is at most deg(D) + deg(tau) + deg(c) + max_j(deg(P_j) - deg(Q_j) + deg(D)).

**Practical concern:** The R_j denominators may introduce significant degree inflation. For typical q-hypergeometric inputs, the n-shift ratios are products of linear factors, so degrees stay manageable.

### Pattern 4: WZ Certificate Extraction (ZEIL-03)

**What:** Once the extended key equation is solved, extract the WZ certificate.

**The WZ certificate R(n,k) is defined as:**
```
R(n,k) = G(n,k) / F(n,k) = y(k) = f(x_k) / c_poly(x_k)
```

where f is the polynomial solution and c_poly is from the Gosper normal form. The certificate is a rational function of x_k = q^k (and also depends on the specific n value).

**Output format:** Return R(n,k) as a QRatRationalFunc in x = q^k. For the user, this means they can evaluate R at specific (n, k) values.

### Pattern 5: WZ Certificate Verification (ZEIL-04)

**What:** Given F(n,k), recurrence coefficients c_0, ..., c_d, and a WZ certificate R(n,k), verify that:

```
sum_{j=0}^d c_j * F(n+j,k) = G(n,k+1) - G(n,k)
```

where G(n,k) = R(n,k) * F(n,k).

**Verification by evaluation:** For each (n, k) in a grid:
1. Compute LHS = sum_j c_j * F(n+j, k) by evaluating the hypergeometric terms.
2. Compute G(n,k) = R(n,k) * F(n,k) and G(n,k+1) = R(n,k+1) * F(n,k+1).
3. Compute RHS = G(n,k+1) - G(n,k).
4. Check LHS == RHS (exact rational arithmetic).

**User-supplied certificates (ZEIL-04):** Accept R as a QRatRationalFunc and verify it against the recurrence. This is purely a verification step -- no need to re-run the algorithm.

### Pattern 6: FPS Cross-Verification (ZEIL-05)

**What:** Verify that the recurrence c_0*S(n) + ... + c_d*S(n+d) = 0 holds by computing S(n) via FPS expansion and checking.

**Approach:** Use eval_phi to compute the FPS of the sum S(n) for several n values, then check that the linear combination of these FPS values satisfies the recurrence.

For a terminating sum S(n) = sum_k F(n,k), eval_phi gives an exact FPS. Evaluate at specific n values (n = 0, 1, ..., N) and check c_0*S(n) + ... + c_d*S(n+d) = 0 for each n.

### Anti-Patterns to Avoid

- **Treating n as symbolic throughout:** Our infrastructure works with concrete rationals. Use concrete n_val and verify consistency, not symbolic Q(q^n)[x].

- **Building a separate q-Gosper variant for Zeilberger:** Reuse the existing q_gosper infrastructure. The only extension needed is the parametric key equation with c_j unknowns.

- **Ignoring the degree blow-up from R_j denominators:** The n-shift ratios may have large denominators. Always compute the LCM and clear denominators before setting up the linear system.

- **Assuming c_d = 1 always works:** Sometimes the leading coefficient c_d vanishes for certain n values. Use null-space computation rather than simple normalization.

- **Forgetting boundary conditions:** The telescoping argument requires G(n, 0) - G(n, -inf) = 0 for terminating series. For terminating sums, this is automatic since F(n, k) = 0 for k > termination_order.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Indefinite q-summation of modified summand | New summation algorithm | `q_gosper` / `solve_key_equation` from Phase 14 | Already correct and tested with 45 tests |
| Term ratio extraction | Manual ratio computation | `extract_term_ratio` from Phase 14 | Handles all r_phi_s cases including extra factors |
| Polynomial GCD in key equation setup | Custom GCD | `poly_gcd` from Phase 13 | Subresultant PRS handles coefficient growth |
| Rational function simplification | Manual cancel | `QRatRationalFunc::new` from Phase 13 | Auto-reduces on construction |
| FPS evaluation for cross-verification | Manual term accumulation | `eval_phi` from Phase 6 | Handles truncation, extra factors, termination |
| Linear system solving | New RREF solver | `solve_linear_system` from gosper.rs (make pub(crate)) | Already handles overdetermined/underdetermined cases |
| q-shift of polynomials | Manual coefficient scaling | `QRatPoly::q_shift_n` from Phase 13 | O(n) and correct |

**Key insight:** The q-Zeilberger algorithm is architecturally an outer loop around q-Gosper's key equation solver, with the extension of additional unknowns (the c_j coefficients) in the linear system. Almost no new mathematical machinery is needed beyond what Phases 13-14 provide.

## Common Pitfalls

### Pitfall 1: N-Direction Ratio Computation
**What goes wrong:** Computing F(n+j,k)/F(n,k) incorrectly, especially when multiple parameters depend on n or when the argument z depends on n.
**Why it happens:** For the q-Vandermonde _2phi1(q^{-n}, a; c; q, cq^n/a), both the upper param q^{-n} AND the argument z = cq^n/a depend on n. Shifting n -> n+1 changes both.
**How to avoid:** Compute the ratio by explicitly constructing the HypergeometricSeries for n+j and taking the term-by-term ratio, rather than trying to derive the ratio analytically.
**Warning signs:** WZ verification fails for specific test cases.

### Pitfall 2: Degree Bound for Extended Key Equation
**What goes wrong:** The degree bound for f in the extended key equation is too tight, missing valid solutions.
**Why it happens:** The RHS has higher degree than in the standard Gosper case because of the R_j factors. The bound from standard Gosper may be insufficient.
**How to avoid:** Compute the degree of the cleared-denominator RHS carefully. Try multiple degree candidates as in Phase 14's compute_degree_candidates. Fall back to deg+1 if first attempt fails.
**Warning signs:** q-Vandermonde (which should succeed at d=1) returns no recurrence.

### Pitfall 3: Homogeneous System Solution
**What goes wrong:** The extended linear system with c_j unknowns is homogeneous -- we need a non-trivial solution, not the trivial zero solution.
**Why it happens:** Setting c_d = 1 changes the system to inhomogeneous, but if c_d should actually be 0 (the recurrence has lower effective order), this normalization fails.
**How to avoid:** Compute the null space of the coefficient matrix. If the null space has dimension >= 1, extract a non-trivial solution. Normalize afterward (e.g., make c_d the first nonzero c_j from the right).
**Warning signs:** The algorithm claims no recurrence exists when one should.

### Pitfall 4: WZ Certificate Poles
**What goes wrong:** The WZ certificate R(n,k) = f(q^k)/c(q^k) has poles at specific k values where c(q^k) = 0.
**Why it happens:** The Gosper normal form introduces a c polynomial whose roots at q-shifts of k create poles in the certificate.
**How to avoid:** In verification, check for poles and handle them (the identity should still hold by continuity/cancellation). The verification grid should avoid pole locations.
**Warning signs:** Division-by-zero panic during certificate evaluation.

### Pitfall 5: Termination Order vs. Recurrence Validity
**What goes wrong:** The recurrence c_0*S(n) + ... + c_d*S(n+d) = 0 is derived for a specific n_val but may not hold for all n.
**Why it happens:** If the c_j are specific to one n value (constant, not polynomial in q^n), the recurrence may be an artifact.
**How to avoid:** Verify the recurrence at multiple n values. Alternatively, verify via FPS cross-check (ZEIL-05) at several n.
**Warning signs:** Recurrence holds at n_val but fails at n_val + 1.

## Code Examples

### Data Structures

```rust
/// Result of the q-Zeilberger algorithm.
#[derive(Clone, Debug)]
pub struct ZeilbergerResult {
    /// The recurrence order d.
    pub order: usize,
    /// The recurrence coefficients c_0, ..., c_d as concrete QRat values.
    /// These are the values at the specific n used in the computation.
    /// c_j is the coefficient of S(n+j).
    pub coefficients: Vec<QRat>,
    /// The WZ proof certificate: a rational function R(x) of x = q^k.
    /// G(n,k) = R(q^k) * F(n,k) is the antidifference companion.
    pub certificate: QRatRationalFunc,
}

/// Result enum for q-Zeilberger.
#[derive(Clone, Debug)]
pub enum QZeilbergerResult {
    /// A recurrence was found.
    Recurrence(ZeilbergerResult),
    /// No recurrence found up to the given max_order.
    NoRecurrence,
}
```

### N-Direction Shift Ratio

```rust
/// Compute the n-direction shift ratio F(n+j,k)/F(n,k) for a q-hypergeometric summand.
///
/// Given a HypergeometricSeries parameterized by n (through its parameters),
/// computes the ratio as a rational function of x = q^k.
///
/// The n-dependence is captured by specifying which parameters change with n
/// and how they change.
fn n_shift_ratio(
    series: &HypergeometricSeries,
    j: i64,                    // shift amount (n -> n+j)
    n_val: i64,                // current n value
    q_val: &QRat,              // concrete q
) -> QRatRationalFunc {
    // For each parameter that depends on n (e.g., q^{-n} in upper params),
    // compute the ratio (a(n+j);q)_k / (a(n);q)_k as a function of x_k.
    // This is a product of k factors, each a linear function of x_k.
    // ...
}
```

### Creative Telescoping Main Loop

```rust
/// Run the q-Zeilberger algorithm.
///
/// Finds a linear recurrence c_0*S(n) + c_1*S(n+1) + ... + c_d*S(n+d) = 0
/// for the sum S(n) = sum_k F(n,k), together with a WZ proof certificate.
///
/// The algorithm tries orders d = 1, 2, ..., max_order, calling an extended
/// version of q-Gosper's key equation solver at each order.
pub fn q_zeilberger(
    series: &HypergeometricSeries,
    n_val: i64,                // specific n value for computation
    q_val: &QRat,              // concrete q parameter
    max_order: usize,          // maximum recurrence order to try
) -> QZeilbergerResult {
    for d in 1..=max_order {
        // 1. Compute n-shift ratios R_1(x), ..., R_d(x)
        let ratios: Vec<QRatRationalFunc> = (1..=d as i64)
            .map(|j| n_shift_ratio(series, j, n_val, q_val))
            .collect();

        // 2. Compute Gosper normal form of k-direction term ratio
        let term_ratio = extract_term_ratio(series_at_n, q_val);
        let gnf = gosper_normal_form(&term_ratio.numer, &term_ratio.denom, q_val);

        // 3. Set up extended key equation with c_j unknowns
        // 4. Solve the extended linear system
        // 5. If solution found, extract coefficients and certificate
        // 6. Return result
    }
    QZeilbergerResult::NoRecurrence
}
```

### WZ Certificate Verification

```rust
/// Verify a WZ certificate independently.
///
/// Checks that:
///   c_0*F(n,k) + ... + c_d*F(n+d,k) = G(n,k+1) - G(n,k)
/// where G(n,k) = R(q^k) * F(n,k), for k = 0, 1, ..., max_k.
///
/// The verification uses exact rational arithmetic and is independent
/// of how the certificate was obtained.
pub fn verify_wz_certificate(
    series: &HypergeometricSeries,
    n_val: i64,
    coefficients: &[QRat],
    certificate: &QRatRationalFunc,
    q_val: &QRat,
    max_k: usize,
) -> bool {
    // For each k = 0, ..., max_k:
    // 1. Compute F(n+j, k) for j = 0, ..., d
    // 2. Compute LHS = sum_j c_j * F(n+j, k)
    // 3. Compute G(n,k) = R(q^k) * F(n,k) and G(n,k+1) = R(q^{k+1}) * F(n,k+1)
    // 4. Compute RHS = G(n,k+1) - G(n,k)
    // 5. Check LHS == RHS
    // ...
}
```

### FPS Cross-Verification

```rust
/// Verify recurrence by FPS expansion.
///
/// Computes S(n) for several n values using eval_phi and checks that
/// c_0*S(n) + ... + c_d*S(n+d) = 0.
pub fn verify_recurrence_fps(
    series_builder: &dyn Fn(i64) -> HypergeometricSeries,
    coefficients: &[QRat],
    n_start: i64,
    n_count: usize,
    variable: SymbolId,
    truncation_order: i64,
    q_val: &QRat,
) -> bool {
    let d = coefficients.len() - 1;
    for i in 0..n_count {
        let n = n_start + i as i64;
        let mut sum = FormalPowerSeries::zero(variable, truncation_order);
        for (j, c_j) in coefficients.iter().enumerate() {
            let s_nj = eval_phi(&series_builder(n + j as i64), variable, truncation_order);
            // sum += c_j * s_nj
            // ...
        }
        if !sum.is_zero() {
            return false;
        }
    }
    true
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual identity proofs | Algorithmic Zeilberger/WZ | 1990-1992 | Mechanized definite summation proofs |
| Symbolic q throughout | Concrete q_val with verification | Standard CAS practice | Simpler implementation |
| Separate certificate extraction | Certificate as Gosper byproduct | Paule-Riese 1997 | Certificate comes free from key equation |
| Verify by symbolic algebra | Verify by evaluation at grid points | Practical implementations | More robust, catches numerical issues |

**Key references:**
- Zeilberger (1990): "A fast algorithm for proving terminating hypergeometric identities"
- Zeilberger (1991): "The method of creative telescoping"
- Wilf-Zeilberger (1992): "An algorithmic proof theory for hypergeometric identities"
- Koornwinder (1993): "On Zeilberger's algorithm and its q-analogue: a rigorous description"
- Paule-Riese (1997): "A Mathematica q-analogue of Zeilberger's algorithm"
- Koepf (2014): "Hypergeometric Summation" (Springer), Chapters 6-7
- Petkovsek-Wilf-Zeilberger (1996): "A=B" (A.K. Peters), Chapters 6-7

## Open Questions

1. **Parametric vs. Evaluation-Based c_j Computation**
   - What we know: For a specific n_val, the c_j are concrete rationals. For the recurrence to hold for all n, the c_j should be polynomials in q^n.
   - What's unclear: Whether computing at a single n_val suffices (and the polynomial structure is guaranteed by theory) or whether we need multiple n values.
   - Recommendation: Theory guarantees that if the algorithm succeeds for one generic n_val, the c_j ARE polynomials in q^n. Use a single n_val (chosen to avoid special cases like n=0) and verify at additional n values via ZEIL-05.

2. **solve_linear_system Accessibility**
   - What we know: The solve_linear_system function in gosper.rs is private. The extended key equation needs a similar solver with additional columns.
   - What's unclear: Whether to make the existing one pub(crate) or write a new one.
   - Recommendation: Make solve_linear_system pub(crate) in gosper.rs and reuse it, OR duplicate the solver in zeilberger.rs (it's ~100 lines of RREF). Duplicating is simpler and avoids changing Phase 14.

3. **N-Direction Ratio Representation**
   - What we know: F(n+j,k)/F(n,k) is a rational function of x_k = q^k for each specific n.
   - What's unclear: The cleanest way to compute this from a HypergeometricSeries.
   - Recommendation: Build the series for n+j (by adjusting the parameters) and compute the ratio of term ratios. For a series with upper param q^{-n}: at n_val, the param is q^{-n_val}; at n_val+j, it's q^{-(n_val+j)}. The ratio of their k-th terms gives R_j.

4. **Max Degree Bound for f in Extended System**
   - What we know: Standard Gosper uses deg(c_poly) - max(deg(sigma), deg(tau)) as the degree bound for f.
   - What's unclear: How the R_j denominators affect this bound after clearing.
   - Recommendation: After clearing denominators, compute the max degree of the RHS and use that as the bound. Try multiple candidates (as in Phase 14's compute_degree_candidates).

5. **Python API**
   - What we know: Phase 17 covers Python API for all v1.2 algorithms.
   - What's unclear: Whether Phase 15 should include basic Python bindings.
   - Recommendation: Phase 15 focuses on Rust-only. Python API deferred to Phase 17.

## Recommended Plan Structure

Based on the requirements and dependencies:

**Plan 15-01: N-Direction Ratios and Creative Telescoping Loop (ZEIL-01)**
- Compute F(n+j,k)/F(n,k) as QRatRationalFunc
- Extended key equation setup with c_j unknowns
- Creative telescoping loop trying d = 1, 2, ..., max_order
- Test: q-Vandermonde finds recurrence at d=1

**Plan 15-02: Recurrence Output and WZ Certificate Extraction (ZEIL-02, ZEIL-03)**
- ZeilbergerResult struct with polynomial coefficients and certificate
- WZ certificate extraction from extended key equation solution
- Test: q-Vandermonde recurrence has inspectable c_0(q^n), c_1(q^n)
- Test: WZ certificate is independently verifiable

**Plan 15-03: WZ Verification and FPS Cross-Check (ZEIL-04, ZEIL-05)**
- verify_wz_certificate function accepting user-supplied certificates
- verify_recurrence_fps function using eval_phi
- Test: User-supplied correct certificate passes
- Test: User-supplied incorrect certificate fails
- Test: FPS cross-verification matches for multiple n values

## Sources

### Primary (HIGH confidence)
- Phase 14 codebase: `crates/qsym-core/src/qseries/gosper.rs` -- Complete q-Gosper implementation verified by reading 1655 lines of source. Key equation solver, normal form, term ratio extraction all available and tested (45 tests).
- Phase 13 codebase: `crates/qsym-core/src/poly/` -- QRatPoly, QRatRationalFunc, poly_gcd, q_shift verified (727+ lines, 144 tests).
- Phase 6 codebase: `crates/qsym-core/src/qseries/hypergeometric.rs` -- HypergeometricSeries, eval_phi, eval_psi verified (1383 lines).
- [Maple WZ Method documentation](https://www.maplesoft.com/support/help/maple/view.aspx?path=SumTools/Hypergeometric/WZMethod) -- Verified WZ pair definition F(n+1,k)-F(n,k) = G(n,k+1)-G(n,k), certificate R(n,k) = G(n,k)/F(n,k).
- [MathWorld - Wilf-Zeilberger Pair](https://mathworld.wolfram.com/Wilf-ZeilbergerPair.html) -- Verified WZ pair definition and proof strategy.
- [AMS Notices - What Is a WZ Pair?](https://www.ams.org/notices/201004/rtx100400508p.pdf) -- Verified certificate verification procedure: substitute R, check recurrence, confirm telescoping.

### Secondary (MEDIUM confidence)
- [Paule-Riese 1997 (RISC)](https://www3.risc.jku.at/publications/download/risc_117/Paule_Riese.pdf) -- q-analogue of Zeilberger based on qGFF approach. PDF exists; algorithm structure confirmed by multiple sources.
- [Koornwinder 1993](https://staff.fnwi.uva.nl/t.h.koornwinder/art/1993/zeilbalgo.pdf) -- Rigorous q-Zeilberger description including adaptations of Zeilberger's 6 steps to q-case.
- [Koepf - Hypergeometric Summation, Chapters 6-7](https://link.springer.com/book/10.1007/978-1-4471-6464-7) -- q-WZ method (Ch 6) and q-Zeilberger algorithm (Ch 7) with Maple implementations.
- [Zeilberger's original paper (1991)](https://sites.math.rutgers.edu/~zeilberg/mamarimY/creativeT.pdf) -- "The method of creative telescoping" defining the creative telescoping equation.
- [Wilf-Zeilberger 1992](https://sites.math.rutgers.edu/~zeilberg/mamarimY/Zeilberger_y1992_p575.pdf) -- Algorithmic proof theory for hypergeometric and q-identities.

### Tertiary (LOW confidence)
- Extended key equation formulation: derived from combining Phase 14's Gosper key equation with the creative telescoping framework. The specific form (with R_j denominators cleared) needs validation during implementation.
- Degree bound for f in extended system: extrapolated from standard Gosper bounds. Needs empirical validation with test cases.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All required infrastructure exists in Phases 13-14, verified by reading source
- Architecture (creative telescoping loop): HIGH - Algorithm well-established (Zeilberger 1990, Koornwinder 1993, Paule-Riese 1997)
- Architecture (extended key equation): MEDIUM - Derivation follows from standard Gosper theory but specific implementation with R_j denominator clearing needs validation
- N-direction ratio computation: MEDIUM - Mathematically straightforward but implementation details depend on HypergeometricSeries parameterization
- WZ verification: HIGH - Purely evaluative (check LHS == RHS at grid points), no algorithmic complexity
- FPS cross-verification: HIGH - Uses existing eval_phi, straightforward

**Research date:** 2026-02-16
**Valid until:** 2026-04-16 (stable mathematical algorithms, no version churn)
