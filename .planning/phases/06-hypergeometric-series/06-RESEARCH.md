# Phase 6: Hypergeometric Series - Research

**Researched:** 2026-02-13
**Domain:** Basic hypergeometric series (_r phi_s, _r psi_s), summation formulas, transformation formulas (Rust, computational q-series algebra)
**Confidence:** HIGH

## Summary

Phase 6 implements the basic hypergeometric series framework on top of the existing q-Pochhammer and FormalPowerSeries infrastructure from Phases 2-3. The phase covers 10 requirements (HYPR-01 through HYPR-10): constructing and evaluating _r phi_s and _r psi_s series term-by-term, implementing 6 classical summation formulas (q-Gauss, q-Vandermonde, q-Saalschutz, q-Kummer, q-Dixon), and implementing 4 transformation families (Heine's 3 forms, Sears' 4phi3, Watson's 8phi7-to-4phi3, and Bailey's 10phi9).

The key architectural insight is that basic hypergeometric series are *defined* in terms of q-Pochhammer symbols, which are already implemented via `aqprod`. The general term of _r phi_s is a ratio of q-Pochhammer symbols multiplied by a power factor. Term-by-term evaluation into a FormalPowerSeries is the core computational operation; summation formulas and transformations are *structural recognition* operations that detect when a given hypergeometric series matches a known closed form or can be rewritten as another hypergeometric series.

The implementation strategy has two levels: (1) a computational layer in `qseries/hypergeometric.rs` that evaluates series into FPS by computing terms, and (2) a structural/symbolic layer that represents `BasicHypergeometric` parameters and applies pattern-matching rules for summation and transformation. Summation formulas return closed-form FPS results (products of q-Pochhammer symbols). Transformations return new `HypergeometricSeries` structs with different parameters. Verification is done by expanding both sides to O(q^N) and comparing FPS coefficients.

This phase adds NO new crate dependencies. All computation builds on the existing `FormalPowerSeries`, `QMonomial`, `PochhammerOrder`, `aqprod`, and `arithmetic::*` functions.

**Primary recommendation:** Create a new `crates/qsym-core/src/qseries/hypergeometric.rs` module containing: (1) `HypergeometricSeries` struct for _r phi_s with parameter lists, (2) `BilateralHypergeometricSeries` struct for _r psi_s, (3) `eval_phi` and `eval_psi` functions for FPS evaluation, (4) summation formula functions that return closed-form FPS, (5) transformation functions that return new series representations. Parameters should be `Vec<QMonomial>` (not symbolic Expr) to stay within the computable FPS framework.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rug` | 1.28 | QRat exact arithmetic (GMP-backed) | Already in use; non-negotiable for exact q-series |
| `FormalPowerSeries` | internal | Sparse series representation | Phase 2 foundation; all evaluations produce FPS |
| `aqprod` | internal | q-Pochhammer symbol computation | Phase 3 foundation; hypergeometric terms are ratios of these |
| `QMonomial` | internal | Parameter representation: c*q^m | Already defined; perfect for hypergeometric parameters |
| `PochhammerOrder` | internal | Finite/Infinite order enum | Already defined; used in summation conditions |
| `arithmetic::*` | internal | FPS add, mul, invert, shift | Phase 2 foundation; needed for closed-form evaluation |
| `smallvec` | 1 | SmallVec in Expr::BasicHypergeometric | Already a dependency; used for upper/lower parameter lists |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `BTreeMap<i64, QRat>` | stdlib | Sparse FPS storage | Underlying FPS representation |
| `InfiniteProductGenerator` | internal | Lazy infinite product expansion | Evaluating closed-form summation results that involve infinite q-Pochhammer products |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| QMonomial parameters | Symbolic ExprRef parameters | QMonomial keeps everything computable as FPS; symbolic params need multivariate series or lazy eval. QMonomial covers all concrete cases (a=q^k, a=c*q^k). Symbolic params are future work. |
| Struct-based series representation | Using Expr::BasicHypergeometric directly | Computation needs mutable state, parameter vectors, and truncation. The Expr variant is for symbolic representation. Keep them separate: struct for computation, Expr for symbolic display/manipulation. |
| Term-by-term summation | Direct product formula for series eval | Products of q-Pochammers would require O(r+s) infinite product generators. Term-by-term is simpler and naturally handles terminating series (when a_i = q^{-n}). |

**Dependencies to add:** None. Phase 6 uses only libraries already in Cargo.toml.

## Mathematical Definitions

### HYPR-01: Basic Hypergeometric Series _r phi_s

**Definition (Gasper-Rahman, eq. 1.2.22; DLMF 17.4.1):**

```
_r phi_s (a_1, ..., a_r ; b_1, ..., b_s ; q, z)
  = sum_{n=0}^{inf} [(a_1;q)_n * ... * (a_r;q)_n] / [(q;q)_n * (b_1;q)_n * ... * (b_s;q)_n]
    * [(-1)^n * q^{n(n-1)/2}]^{1+s-r} * z^n
```

**Key points:**
- The factor `[(-1)^n * q^{n(n-1)/2}]^{1+s-r}` is the Gasper-Rahman convention
- When r = s+1 (the "balanced" case), this factor is 1 (since 1+s-r = 0), giving the simpler:
  ```
  _{s+1} phi_s = sum_{n=0}^{inf} [(a_1;q)_n * ... * (a_{s+1};q)_n] / [(q;q)_n * (b_1;q)_n * ... * (b_s;q)_n] * z^n
  ```
- A series is **terminating** when some a_i = q^{-N} for nonnegative integer N (then (a_i;q)_n = 0 for n > N)
- A series is **balanced** (for r = s+1) when a_1 * ... * a_{s+1} * q = b_1 * ... * b_s * z
- A series is **well-poised** when a_1 * q = a_2 * b_1 = a_3 * b_2 = ... = a_{s+1} * b_s
- A series is **very-well-poised** when it is well-poised and a_2 = q*sqrt(a_1), a_3 = -q*sqrt(a_1)

**Convergence:** For |q| < 1:
- r <= s: converges for all z
- r = s+1: converges for |z| < 1
- r > s+1: diverges unless terminating

**Term-by-term evaluation:** The n-th term ratio is:
```
c_{n+1} / c_n = [(1 - a_1*q^n) * ... * (1 - a_r*q^n)] / [(1 - q^{n+1}) * (1 - b_1*q^n) * ... * (1 - b_s*q^n)]
                * [(-1) * q^n]^{1+s-r} * z
```
This recurrence is efficient for implementation: compute each term from the previous one.

### HYPR-02: Bilateral Hypergeometric Series _r psi_s

**Definition (Gasper-Rahman; DLMF 17.4):**

```
_r psi_s (a_1, ..., a_r ; b_1, ..., b_s ; q, z)
  = sum_{n=-inf}^{inf} [(a_1;q)_n * ... * (a_r;q)_n] / [(b_1;q)_n * ... * (b_s;q)_n]
    * [(-1)^n * q^{n(n-1)/2}]^{s-r} * z^n
```

**Key differences from _r phi_s:**
- Summation runs from -inf to +inf (bilateral)
- No (q;q)_n in the denominator
- The power factor is `[(-1)^n * q^{n(n-1)/2}]^{s-r}` (not 1+s-r)
- Uses `(a;q)_n` for negative n via: `(a;q)_{-m} = (-q/a)^m * q^{m(m-1)/2} / (q/a;q)_m`

**Convergence:** For r = s, converges when |b_1*...*b_s / (a_1*...*a_r)| < |z| < 1.

**Implementation:** Split the sum into n >= 0 and n < 0 parts. For n >= 0, use standard q-Pochhammer. For n < 0, use the negative-order identity to convert (a;q)_{-m} into ratios of positive-order Pochhammer symbols. Both parts contribute to the FPS.

**Ramanujan's 1psi1 summation (important closed form):**
```
_1 psi_1 (a ; b ; q, z) = (q;q)_inf * (b/a;q)_inf * (az;q)_inf * (q/(az);q)_inf
                         / [(b;q)_inf * (q/a;q)_inf * (z;q)_inf * (b/(az);q)_inf]
```
Valid for |b/a| < |z| < 1.

### HYPR-03: q-Gauss Summation (DLMF 17.6.1)

```
_2 phi_1 (a, b ; c ; q, c/(ab))
  = (c/a;q)_inf * (c/b;q)_inf / [(c;q)_inf * (c/(ab);q)_inf]
```

**Conditions:**
- |c/(ab)| < 1 (convergence of the 2phi1)
- c is not a negative integer power of q

**Applicability check:** A 2phi1 series matches when z = c/(a_1 * a_2). When this condition is detected, return the closed-form product of 4 infinite q-Pochhammer symbols.

### HYPR-04: q-Vandermonde (q-Chu-Vandermonde) Summation (DLMF 17.6.2, 17.6.3)

**First form (DLMF 17.6.2):**
```
_2 phi_1 (a, q^{-n} ; c ; q, c*q^n/a)
  = (c/a;q)_n / (c;q)_n
```

**Second form (DLMF 17.6.3):**
```
_2 phi_1 (a, q^{-n} ; c ; q, q)
  = a^n * (c/a;q)_n / (c;q)_n
```

**Conditions:** n is a nonnegative integer (terminating series). This is the q-analog of the Vandermonde identity C(m+n, r) = sum C(m,k)*C(n,r-k).

**Applicability check:** One upper parameter is q^{-n}. Check whether z matches either `c*q^n/a` (first form) or `q` (second form).

### HYPR-05: q-Pfaff-Saalschutz Summation (DLMF 17.7.4)

```
_3 phi_2 (a, b, q^{-n} ; c, a*b*q^{1-n}/c ; q, q)
  = (c/a;q)_n * (c/b;q)_n / [(c;q)_n * (c/(ab);q)_n]
```

**Conditions:**
- n is a nonnegative integer (terminating)
- The series is balanced: the second lower parameter is determined as `a*b*q^{1-n}/c`
- z = q

**Applicability check:** It is a 3phi2 with one upper parameter q^{-n}, z=q, and the lower parameters satisfy the balanced condition.

### HYPR-06: q-Kummer (Bailey-Daum) and q-Dixon Summation

**Bailey-Daum q-Kummer sum (DLMF 17.6.5):**
```
_2 phi_1 (a, b ; a*q/b ; q, -q/b)
  = (-q;q)_inf * (a*q;q^2)_inf * (a*q^2/b^2;q^2)_inf
    / [(-q/b;q)_inf * (a*q/b;q)_inf]
```

**Conditions:** |b| > |q| (equivalently |q/b| < 1).

**Note:** The right-hand side involves q^2-Pochhammer symbols (i.e., (x;q^2)_inf), which are computed by `etaq` with step parameter 2.

**Jackson's q-Dixon sum (DLMF 17.7.6):**
```
_3 phi_2 (q^{-2n}, b, c ; q^{1-2n}/b, q^{1-2n}/c ; q, q^{2-n}/(bc))
  = (b;q)_n * (c;q)_n * (q;q)_{2n} * (bc;q)_{2n}
    / [(q;q)_n * (bc;q)_n * (b;q)_{2n} * (c;q)_{2n}]
```

**Conditions:** n is a nonnegative integer.

### HYPR-07: Heine's Transformation (3 Forms)

**First Heine transformation (Gasper-Rahman 1.4.1):**
```
_2 phi_1 (a, b ; c ; q, z)
  = [(b;q)_inf * (az;q)_inf] / [(c;q)_inf * (z;q)_inf]
    * _2 phi_1 (c/b, z ; az ; q, b)
```
Valid for |z| < 1 and |b| < 1.

**Second Heine transformation (Gasper-Rahman 1.4.2):**
Obtained by applying the first transformation to the result:
```
_2 phi_1 (a, b ; c ; q, z)
  = [(c/b;q)_inf * (bz;q)_inf] / [(c;q)_inf * (z;q)_inf]
    * _2 phi_1 (abz/c, b ; bz ; q, c/b)
```

**Third Heine transformation (Gasper-Rahman 1.4.3):**
The third iterate:
```
_2 phi_1 (a, b ; c ; q, z)
  = [(abz/c;q)_inf] / [(z;q)_inf]
    * _2 phi_1 (c/a, c/b ; c ; q, abz/c)
```

**Implementation:** Each transformation takes a `HypergeometricSeries` (with r=2, s=1) and returns a new `HypergeometricSeries` with transformed parameters plus a scalar prefactor (ratio of infinite q-Pochhammer products). The prefactor is evaluated as FPS, and the transformed series is also evaluated as FPS. The product gives the same result as the original.

### HYPR-08: Sears' 4phi3 Transformation

**Sears-Whipple transformation for terminating balanced _4 phi_3:**
```
_4 phi_3 (q^{-n}, a, b, c ; d, e, f ; q, q) where def = abc*q^{1-n}
  = [(e/a;q)_n * (f/a;q)_n] / [(e;q)_n * (f;q)_n]
    * _4 phi_3 (q^{-n}, a, d/b, d/c ; d, aq^{1-n}/e, aq^{1-n}/f ; q, q)
```

**Conditions:**
- Terminating: one upper parameter is q^{-n}
- Balanced: d*e*f = a*b*c*q^{1-n} (equivalently z=q and the balance condition)

**Implementation:** Detects balanced terminating 4phi3 and applies the parameter transformation.

### HYPR-09: Watson's Transformation

**Watson's transformation (8phi7 to 4phi3):**
```
_8 phi_7 (a, q*sqrt(a), -q*sqrt(a), b, c, d, e, f ;
          sqrt(a), -sqrt(a), aq/b, aq/c, aq/d, aq/e, aq/f ;
          q, a^2*q^2/(bcdef))

  = [(aq;q)_inf * (aq/(de);q)_inf * (aq/(df);q)_inf * (aq/(ef);q)_inf]
    / [(aq/d;q)_inf * (aq/e;q)_inf * (aq/f;q)_inf * (aq/(def);q)_inf]
    * _4 phi_3 (aq/(bc), d, e, f ; aq/b, aq/c, def/a ; q, q)
```

**Conditions:**
- Very-well-poised on the left (a_2 = q*sqrt(a_1), a_3 = -q*sqrt(a_1))
- At least one of d, e, f is q^{-N} (terminating)
- z = a^2*q^2/(bcdef)

**Implementation:** This is a powerful simplification: an 8phi7 reduces to a 4phi3. The left side is very-well-poised (the sqrt(a) and -sqrt(a) parameters are characteristic). Detection checks the very-well-poised structure.

### HYPR-10: Bailey's Transformation

**Bailey's _10 phi_9 transformation (Gasper-Rahman, eq. 2.9.1):**

Bailey's transformation relates two very-well-poised 10phi9 series (or equivalently, transforms a 10phi9 into a product of 4phi3 series). The version from DLMF 17.7.12 for the terminating case:

```
_4 phi_3 (a, aq, b^2*q^{2n}, q^{-2n} ; b, bq, a^2*q^2 ; q^2, q^2)
  = a^n * (-q;q)_n * (b/a;q)_n / [(-aq;q)_n * (b;q)_n]
```

For the general Bailey transformation (the classical 10phi9 form), the parameters relate two very-well-poised series. The key equation is:

```
_10 phi_9 (a, qa^{1/2}, -qa^{1/2}, b, c, d, e, f, a^2*q^{n+2}/(bcde), q^{-n} ;
           a^{1/2}, -a^{1/2}, aq/b, aq/c, aq/d, aq/e, aq/f, bcdeq^{-n-1}/a, aq^{n+1} ;
           q, q)
  = [(aq;q)_n * (aq/(bc);q)_n * (aq/(bd);q)_n * (aq/(cd);q)_n]
    / [(aq/b;q)_n * (aq/c;q)_n * (aq/d;q)_n * (aq/(bcd);q)_n]
    * _10 phi_9 (...)  [with transformed parameters]
```

**Implementation note:** The 10phi9 forms are complex. For Phase 6, implement the DLMF 17.7.12 form (4phi3 with q^2 base) as the primary Bailey formula, and detect the very-well-poised structure for the full 10phi9 if needed. Verification is via series expansion comparison.

## Architecture Patterns

### Recommended Module Structure

```
crates/qsym-core/src/
  qseries/
    mod.rs            # Add: pub mod hypergeometric; pub use exports
    hypergeometric.rs  # All Phase 6 code in one file (or split if >600 lines)
```

If the file grows large, split into:
```
    hypergeometric/
      mod.rs          # Structs, eval, re-exports
      summation.rs    # q-Gauss, q-Vandermonde, q-Saalschutz, q-Kummer, q-Dixon
      transform.rs    # Heine, Sears, Watson, Bailey
```

Test file: `crates/qsym-core/tests/qseries_hypergeometric_tests.rs`

### Pattern 1: HypergeometricSeries Struct

**What:** A lightweight struct holding the parameters of an _r phi_s series, separate from the Expr::BasicHypergeometric symbolic node. This struct is used for computation; the Expr variant is for display/symbolic manipulation.

**When to use:** All computational operations on hypergeometric series.

```rust
/// Parameters of a basic hypergeometric series _r phi_s.
///
/// Represents: _r phi_s (a_1, ..., a_r ; b_1, ..., b_s ; q, z)
/// where each a_i and b_j is a QMonomial (c * q^m).
#[derive(Clone, Debug)]
pub struct HypergeometricSeries {
    /// Upper parameters a_1, ..., a_r
    pub upper: Vec<QMonomial>,
    /// Lower parameters b_1, ..., b_s
    pub lower: Vec<QMonomial>,
    /// The argument z (as QMonomial)
    pub argument: QMonomial,
}

impl HypergeometricSeries {
    pub fn r(&self) -> usize { self.upper.len() }
    pub fn s(&self) -> usize { self.lower.len() }

    /// Check if the series is terminating (some a_i = q^{-n} with n >= 0).
    pub fn termination_order(&self) -> Option<i64> {
        for a in &self.upper {
            // a_i = c * q^m terminates at order n when c=1 and m <= 0
            // because (q^{-n};q)_k = 0 for k > n
            if a.coeff == QRat::one() && a.power <= 0 {
                return Some(-a.power);
            }
        }
        None
    }
}
```

### Pattern 2: Term-by-Term Evaluation via Ratio Recurrence

**What:** Evaluate _r phi_s by computing term ratios rather than recomputing q-Pochhammer products from scratch for each n.

**When to use:** `eval_phi` and `eval_psi`.

```rust
/// Evaluate _r phi_s to O(q^T) as a FormalPowerSeries.
pub fn eval_phi(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let r = series.r();
    let s = series.s();
    let extra_power = (1 + s).saturating_sub(r); // max(0, 1+s-r)

    let mut result = FormalPowerSeries::zero(variable, truncation_order);

    // First term (n=0) is always 1
    let mut term = FormalPowerSeries::one(variable, truncation_order);
    result = arithmetic::add(&result, &term);

    let max_n = if let Some(n) = series.termination_order() {
        n
    } else {
        truncation_order // non-terminating: compute until truncation
    };

    for n in 1..=max_n {
        // Multiply by the n-th ratio factor:
        // ratio = prod_i (1 - a_i * q^{n-1}) / [(1 - q^n) * prod_j (1 - b_j * q^{n-1})]
        //         * [(-1) * q^{n-1}]^{extra_power} * z
        let ratio = compute_term_ratio(series, n, variable, truncation_order);
        term = arithmetic::mul(&term, &ratio);

        if term.is_zero() { break; } // terminating
        result = arithmetic::add(&result, &term);
    }
    result
}
```

### Pattern 3: Summation as Pattern Matching + Closed Form

**What:** Summation formulas check if the series parameters match a known pattern, and if so, return the closed-form result directly as FPS (product of q-Pochhammer symbols), skipping term-by-term evaluation.

**When to use:** All summation formula implementations (HYPR-03 through HYPR-06).

```rust
/// Result of attempting to apply a summation formula.
pub enum SummationResult {
    /// Formula applied; returns the closed-form FPS.
    ClosedForm(FormalPowerSeries),
    /// Formula does not apply (parameters don't match the pattern).
    NotApplicable,
}

/// Try q-Gauss summation on a 2phi1 series.
/// Applies when z = c/(a*b) where a, b are upper and c is lower param.
pub fn try_q_gauss(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> SummationResult {
    if series.r() != 2 || series.s() != 1 {
        return SummationResult::NotApplicable;
    }
    let a = &series.upper[0];
    let b = &series.upper[1];
    let c = &series.lower[0];
    let z = &series.argument;

    // Check: z = c/(a*b)
    // In QMonomial terms: z.coeff = c.coeff / (a.coeff * b.coeff)
    //                     z.power = c.power - a.power - b.power
    let expected_coeff = c.coeff.clone() / (a.coeff.clone() * b.coeff.clone());
    let expected_power = c.power - a.power - b.power;

    if z.coeff != expected_coeff || z.power != expected_power {
        return SummationResult::NotApplicable;
    }

    // Closed form: (c/a;q)_inf * (c/b;q)_inf / [(c;q)_inf * (c/(ab);q)_inf]
    let c_over_a = QMonomial::new(c.coeff.clone() / a.coeff.clone(), c.power - a.power);
    let c_over_b = QMonomial::new(c.coeff.clone() / b.coeff.clone(), c.power - b.power);
    let c_over_ab = QMonomial::new(
        c.coeff.clone() / (a.coeff.clone() * b.coeff.clone()),
        c.power - a.power - b.power,
    );

    let numer1 = aqprod(&c_over_a, variable, PochhammerOrder::Infinite, truncation_order);
    let numer2 = aqprod(&c_over_b, variable, PochhammerOrder::Infinite, truncation_order);
    let denom1 = aqprod(c, variable, PochhammerOrder::Infinite, truncation_order);
    let denom2 = aqprod(&c_over_ab, variable, PochhammerOrder::Infinite, truncation_order);

    let numer = arithmetic::mul(&numer1, &numer2);
    let denom = arithmetic::mul(&denom1, &denom2);
    SummationResult::ClosedForm(arithmetic::mul(&numer, &arithmetic::invert(&denom)))
}
```

### Pattern 4: Transformation Returns New Series + Prefactor

**What:** Transformation formulas return a new `HypergeometricSeries` with different parameters plus an FPS prefactor.

**When to use:** All transformation implementations (HYPR-07 through HYPR-10).

```rust
/// Result of applying a transformation formula.
pub struct TransformationResult {
    /// The scalar/product prefactor, evaluated as FPS.
    pub prefactor: FormalPowerSeries,
    /// The transformed hypergeometric series.
    pub transformed: HypergeometricSeries,
}

/// Verify a transformation by expanding both sides and comparing.
pub fn verify_transformation(
    original: &HypergeometricSeries,
    result: &TransformationResult,
    variable: SymbolId,
    truncation_order: i64,
) -> bool {
    let lhs = eval_phi(original, variable, truncation_order);
    let rhs_series = eval_phi(&result.transformed, variable, truncation_order);
    let rhs = arithmetic::mul(&result.prefactor, &rhs_series);
    lhs == rhs
}
```

### Pattern 5: QMonomial Arithmetic Helpers

**What:** QMonomial needs basic arithmetic (multiply, divide, power) for computing parameter transformations.

**When to use:** Computing transformed parameters like c/a, a*b, a*b*q^{1-n}/c.

```rust
impl QMonomial {
    /// Multiply two QMonomials: (c1*q^p1) * (c2*q^p2) = (c1*c2)*q^{p1+p2}
    pub fn mul(&self, other: &QMonomial) -> QMonomial {
        QMonomial::new(
            self.coeff.clone() * other.coeff.clone(),
            self.power + other.power,
        )
    }

    /// Divide: (c1*q^p1) / (c2*q^p2) = (c1/c2)*q^{p1-p2}
    pub fn div(&self, other: &QMonomial) -> QMonomial {
        QMonomial::new(
            self.coeff.clone() / other.coeff.clone(),
            self.power - other.power,
        )
    }

    /// Check if this is q^{-n} for some n >= 0 (i.e., coeff=1, power <= 0)
    pub fn is_q_neg_power(&self) -> Option<i64> {
        if self.coeff == QRat::one() && self.power <= 0 {
            Some(-self.power)
        } else {
            None
        }
    }
}
```

### Anti-Patterns to Avoid

- **Mixing symbolic and computational layers:** Do NOT try to evaluate Expr::BasicHypergeometric symbolically. Use the HypergeometricSeries struct for computation and Expr for display. They can interconvert but serve different purposes.

- **Computing each term from scratch:** Do NOT compute (a_1;q)_n, (a_2;q)_n, etc. independently for each n. Use the ratio recurrence: `term_n = term_{n-1} * ratio(n)`. This is O(1) per term instead of O(n).

- **Requiring symbolic sqrt(a) for Watson/Bailey:** The Watson and Bailey transformations involve sqrt(a_1). For QMonomial parameters where a_1 = c*q^{2m} (even power), sqrt(a_1) = sqrt(c)*q^m. If c is a perfect square rational, this works. If not, flag as not applicable for QMonomial parameters. Do NOT implement general symbolic square roots.

- **Trying to make summation formulas "automatic":** Do NOT insert a magic auto-simplify step that tries all summation formulas. Instead, provide explicit `try_q_gauss()`, `try_q_vandermonde()`, etc. functions that the user or a driver function can call. An optional `try_all_summations()` convenience function can iterate through them.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| q-Pochhammer symbol computation | Custom product loops | `aqprod()` from Phase 3 | Already handles finite, negative, infinite orders correctly |
| Infinite product evaluation | Manual factor-by-factor | `InfiniteProductGenerator` / `aqprod` with Infinite order | Truncation-aware, handles edge cases |
| Series arithmetic | Custom convolution | `arithmetic::mul`, `arithmetic::add`, `arithmetic::invert` | Correct truncation propagation, sparse optimization |
| Parameter matching for QMonomial | Approximate float comparison | Exact QRat comparison | QRat equality is exact; no floating-point tolerance needed |
| n-th term of q-Pochhammer | Recomputing full product | Ratio recurrence: multiply previous term by `(1 - a*q^{n-1})` | O(1) per term vs O(n) |

**Key insight:** Phase 6 is primarily a *pattern recognition and formula application* phase. The heavy computation (series arithmetic, product expansion) is already implemented. The new work is: (1) the term-by-term evaluation loop, (2) parameter matching for summation conditions, (3) parameter transformation for Heine/Sears/Watson/Bailey.

## Common Pitfalls

### Pitfall 1: Term Ratio Computation with QMonomial

**What goes wrong:** When computing the n-th term ratio, the factors (1 - a_i * q^n) must be evaluated as FPS terms. If a_i = c*q^m, then a_i * q^n = c*q^{m+n}. The factor is `1 - c*q^{m+n}` which is a 2-term binomial. Multiplying by this produces a single FPS multiplication per factor. With r upper and s lower parameters, each term requires r+s+1 FPS multiplications.

**Why it happens:** The temptation is to compute each factor as a QRat coefficient and multiply scalars. But this doesn't work because the factors involve different powers of q.

**How to avoid:** Build each ratio factor as a simple 2-term FPS (1 at q^0 and -c at q^{m+n}) and multiply. Alternatively, compute the full n-th term coefficient directly: the n-th term contributes to q^{n*z.power} (where z = c_z * q^{p_z} is the argument), so track which power of q each term lands on.

**Better approach:** Since all parameters are QMonomials (c*q^m), the n-th term of the hypergeometric series is itself a QMonomial times a product of rational numbers. The coefficient of the n-th term can be computed as a QRat, and it contributes to a specific power of q. This avoids FPS multiplication entirely for term computation.

**Warning signs:** eval_phi is very slow for large truncation orders. If each of N terms requires O(N) FPS operations, total is O(N^2). With direct coefficient computation, it should be O(N).

### Pitfall 2: Bilateral Series Divergence at n -> -inf

**What goes wrong:** The bilateral series _r psi_s sums from -inf to +inf. For n < 0, `(a;q)_{-m}` involves `1/(a*q^{-m};q)_m` which can diverge if `a*q^{-m}` passes through a zero of a denominator factor.

**Why it happens:** Bilateral series have convergence conditions that must be satisfied. Not all parameter combinations give convergent series.

**How to avoid:** For the FPS evaluation, only finitely many negative-n terms contribute below the truncation order (since the power of q grows with |n|). Check convergence conditions before evaluation. For _r psi_r (r=s), convergence requires |b_1*...*b_r / (a_1*...*a_r)| < |z| < 1 -- verify at construction time.

**Warning signs:** Bilateral evaluation produces enormous coefficients or apparent non-convergence.

### Pitfall 3: QMonomial Equality and Normalization

**What goes wrong:** When checking if z = c/(ab) for q-Gauss, the QMonomial comparison must be exact. But QMonomials are not automatically normalized (e.g., `QMonomial::new(QRat::from((2,4)), 3)` and `QMonomial::new(QRat::from((1,2)), 3)` should be equal).

**Why it happens:** QRat from rug is always in lowest terms, so this is actually not a problem for the rational coefficient. But be careful about the zero case: `QMonomial::new(QRat::zero(), 5)` and `QMonomial::new(QRat::zero(), 0)` should both represent 0.

**How to avoid:** QRat handles normalization automatically. For the zero case, add a normalization step: if coeff is zero, set power to 0.

### Pitfall 4: Distinguishing Terminating vs Non-Terminating

**What goes wrong:** Summation formulas like q-Vandermonde and q-Saalschutz require a terminating series (a_i = q^{-n}). If the implementation doesn't detect termination, it tries to sum infinitely many terms.

**Why it happens:** q^{-n} as a QMonomial is `QMonomial::new(QRat::one(), -n)`. Detection requires checking that coeff == 1 and power <= 0.

**How to avoid:** The `termination_order()` method on `HypergeometricSeries` handles this. Always check termination before applying terminating-only formulas.

### Pitfall 5: Watson's sqrt(a) Requirement

**What goes wrong:** Watson's transformation requires parameters q*sqrt(a_1) and -q*sqrt(a_1) in positions a_2, a_3. For QMonomial a_1 = c*q^m, sqrt(a_1) requires sqrt(c)*q^{m/2}. If m is odd or c is not a perfect square rational, this fails.

**Why it happens:** The very-well-poised structure involves square roots.

**How to avoid:** Only attempt Watson's transformation when: (1) a_1.power is even, and (2) a_1.coeff is a perfect square rational. Add a `QRat::is_perfect_square()` helper or `QMonomial::try_sqrt()` that returns None if not possible.

### Pitfall 6: Heine's Transformation Prefactor Precision

**What goes wrong:** Heine's transformation involves a prefactor that is a ratio of infinite products, e.g., `(b;q)_inf * (az;q)_inf / [(c;q)_inf * (z;q)_inf]`. If these are evaluated separately and then divided, precision loss from truncation can compound.

**Why it happens:** Each infinite product is truncated to O(q^T). Division of two truncated series can lose precision.

**How to avoid:** Evaluate both sides of the transformation to the same truncation order and compare. The prefactor is itself a well-defined FPS; compute it via mul + invert as usual. Truncation order propagation (using min of both operands) is already handled by `arithmetic::*`.

### Pitfall 7: Performance of High-Order Hypergeometric Series

**What goes wrong:** Watson's 8phi7 has 8 upper and 7 lower parameters. Each term ratio involves 16 factor multiplications. For truncation order T with N terms, this is O(16*N) factor constructions.

**Why it happens:** High r+s means more work per term.

**How to avoid:** Use the direct coefficient computation approach (not FPS-per-term). Since each QMonomial parameter contributes a known factor to the coefficient, compute the n-th term coefficient as a product of QRats and place it at the correct power of q.

## Code Examples

### Evaluating a 2phi1 Series

```rust
/// Evaluate _2 phi_1 (a, b ; c ; q, z) to O(q^T).
///
/// Since r=2, s=1, we have r = s+1 and the extra factor is
/// [(-1)^n * q^{n(n-1)/2}]^0 = 1. This simplifies to:
///
/// sum_{n=0}^{inf} [(a;q)_n * (b;q)_n / ((q;q)_n * (c;q)_n)] * z^n
pub fn eval_2phi1(
    a: &QMonomial, b: &QMonomial, c: &QMonomial, z: &QMonomial,
    variable: SymbolId, truncation_order: i64,
) -> FormalPowerSeries {
    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    // Track cumulative coefficient and power
    let mut coeff = QRat::one();
    let mut power: i64 = 0;

    for n in 0..truncation_order {
        // Add term: coeff * q^power
        if power < truncation_order && !coeff.is_zero() {
            let current = result.coeff(power);
            result.set_coeff(power, current + coeff.clone());
        }

        // Termination check
        if n >= 0 {
            // Ratio: term_{n+1} / term_n =
            //   (1 - a.coeff * q^{a.power + n}) * (1 - b.coeff * q^{b.power + n})
            //   / [(1 - q^{n+1}) * (1 - c.coeff * q^{c.power + n})]
            //   * z.coeff * q^{z.power}
            //
            // For QMonomial coefficient tracking:
            // The factor (1 - x*q^k) at q^0 contributes (1-x) if k=0, or 1 if k>0.
            // Full treatment requires FPS multiplication per factor.
            //
            // SIMPLIFIED for pure-coefficient case where we track only the
            // leading coefficient contribution:
            // [This example shows the concept; actual implementation handles
            //  the full q-power distribution via FPS]
        }

        // For demonstration, compute ratio as FPS
        let a_factor = one_minus_monomial(a, n, variable, truncation_order);
        let b_factor = one_minus_monomial(b, n, variable, truncation_order);
        let q_factor = one_minus_qn(n + 1, variable, truncation_order);
        let c_factor = one_minus_monomial(c, n, variable, truncation_order);
        let z_mono = FormalPowerSeries::monomial(variable, z.coeff.clone(), z.power, truncation_order);

        // ... multiply term by ratio and accumulate
        break; // placeholder
    }
    result
}

/// Helper: create FPS for (1 - a.coeff * q^{a.power + n})
fn one_minus_monomial(a: &QMonomial, n: i64, variable: SymbolId, trunc: i64) -> FormalPowerSeries {
    let exp = a.power + n;
    let mut f = FormalPowerSeries::one(variable, trunc);
    if exp == 0 {
        f.set_coeff(0, QRat::one() - a.coeff.clone());
    } else if exp > 0 && exp < trunc {
        f.set_coeff(exp, -a.coeff.clone());
    }
    f
}
```

### Efficient Direct Coefficient Computation (Preferred)

```rust
/// Evaluate _r phi_s by computing each term's coefficient directly as QRat.
///
/// For QMonomial parameters, the n-th term of _r phi_s is:
///   T_n = [(a_1;q)_n * ... * (a_r;q)_n] / [(q;q)_n * (b_1;q)_n * ... * (b_s;q)_n]
///         * [(-1)^n * q^{n(n-1)/2}]^{1+s-r} * z^n
///
/// The power of q in T_n is:
///   sum_{k=0}^{n-1} (a_i.power + k) - sum_{k=0}^{n-1} (b_j.power + k) - ...
///   + n * z.power + n(n-1)/2 * (1+s-r)
///
/// The rational coefficient is a product of rational factors from the
/// (1-c*q^m) terms in the Pochhammer symbols.
///
/// This approach avoids FPS arithmetic entirely for term computation.
pub fn eval_phi_direct(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let r = series.r();
    let s = series.s();
    let extra = (1 + s) as i64 - r as i64; // can be negative
    let mut result = FormalPowerSeries::zero(variable, truncation_order);

    // Use ratio recurrence: term_n = term_{n-1} * R(n)
    // Track: (cumulative_coeff, cumulative_power) for each term
    let mut term_coeff = QRat::one();
    let mut term_power: i64 = 0;

    let max_n = series.termination_order().unwrap_or(truncation_order);

    for n in 0..=max_n {
        if term_power >= truncation_order { break; }

        // Accumulate term
        if !term_coeff.is_zero() && term_power < truncation_order {
            let existing = result.coeff(term_power);
            result.set_coeff(term_power, existing + term_coeff.clone());
        }

        if n == max_n { break; }

        // Compute ratio to next term
        // Numerator factors: prod_i (1 - a_i.coeff * q^{a_i.power + n})
        // At the coefficient level for the ratio, we need the contribution
        // from each (1 - c*q^m) factor.
        //
        // Factor (1 - c*q^m) at power m:
        //   if m == 0: contributes (1-c) as scalar
        //   if m > 0: contributes 1 at q^0 and -c at q^m
        //
        // For the ratio recurrence to work as scalar coefficients,
        // we need all q-powers to be tracked. This means the n-th term
        // can contribute to MULTIPLE powers of q through cross-terms.
        //
        // CONCLUSION: Direct scalar ratio only works when all parameters
        // have zero q-power (pure rational parameters, no q-dependence).
        // For general QMonomial parameters, use FPS multiplication.
        //
        // [See eval_phi_fps below for the general implementation]
        break;
    }
    result
}
```

### FPS-Based Term Accumulation (General Case)

```rust
/// Evaluate _r phi_s using FPS multiplication for each term ratio.
///
/// This is the general-purpose implementation that handles all QMonomial
/// parameter combinations correctly.
pub fn eval_phi(
    series: &HypergeometricSeries,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let r = series.r();
    let s = series.s();
    let extra = (1 + s) as i64 - r as i64;

    let mut result = FormalPowerSeries::zero(variable, truncation_order);
    let mut term = FormalPowerSeries::one(variable, truncation_order);

    let max_n = series.termination_order()
        .map(|n| n.min(truncation_order))
        .unwrap_or(truncation_order);

    for n in 0..=max_n {
        // Accumulate current term
        result = arithmetic::add(&result, &term);

        if n == max_n { break; }

        // Build ratio = prod_i(1 - a_i*q^{a_i.power+n})
        //             / [(1 - q^{n+1}) * prod_j(1 - b_j*q^{b_j.power+n})]
        //             * (-1)^extra * q^{n*extra} * z
        let mut ratio = FormalPowerSeries::one(variable, truncation_order);

        // Upper parameter factors (numerator)
        for a in &series.upper {
            let factor = one_minus_monomial(a, n, variable, truncation_order);
            ratio = arithmetic::mul(&ratio, &factor);
        }

        // (q;q)_n denominator: divide by (1 - q^{n+1})
        let q_factor = one_minus_qn(n + 1, variable, truncation_order);
        ratio = arithmetic::mul(&ratio, &arithmetic::invert(&q_factor));

        // Lower parameter factors (denominator)
        for b in &series.lower {
            let factor = one_minus_monomial(b, n, variable, truncation_order);
            ratio = arithmetic::mul(&ratio, &arithmetic::invert(&factor));
        }

        // Extra factor: [(-1)^n * q^{n(n-1)/2}]^{extra}
        if extra != 0 {
            // This contributes a sign and a power of q
            let sign = if extra % 2 == 0 || n % 2 == 0 { QRat::one() } else { -QRat::one() };
            let q_shift = n * (n - 1) / 2 * extra;
            let extra_term = FormalPowerSeries::monomial(variable, sign, q_shift, truncation_order);
            ratio = arithmetic::mul(&ratio, &extra_term);
        }

        // Argument z
        let z_term = FormalPowerSeries::monomial(
            variable, series.argument.coeff.clone(), series.argument.power,
            truncation_order,
        );
        ratio = arithmetic::mul(&ratio, &z_term);

        // Update term
        term = arithmetic::mul(&term, &ratio);
        if term.is_zero() { break; }
    }
    result
}

fn one_minus_qn(n: i64, variable: SymbolId, trunc: i64) -> FormalPowerSeries {
    let mut f = FormalPowerSeries::one(variable, trunc);
    if n < trunc && n > 0 {
        f.set_coeff(n, -QRat::one());
    } else if n == 0 {
        f.set_coeff(0, QRat::zero()); // 1 - q^0 = 0
    }
    f
}
```

**IMPORTANT performance note:** The approach above uses FPS inversion (O(T) per inversion) inside the loop, making it O(N * T) per term, and O(N^2 * T) overall. This is expensive for large T. An optimization is to NOT invert each factor separately, but instead:
1. Accumulate all numerator factors via FPS multiplication
2. Accumulate all denominator factors via FPS multiplication
3. Multiply numerator by invert(denominator) ONCE

Or even better: track the cumulative product as a single FPS and multiply by each binomial factor, dividing at the end. But for Phase 6's initial implementation, correctness first, optimize later.

### Python API Integration

```rust
// In crates/qsym-python/src/dsl.rs:

/// Evaluate basic hypergeometric series _r phi_s.
///
/// upper_params and lower_params are lists of (coeff_num, coeff_den, power) tuples.
/// z is the argument as (coeff_num, coeff_den, power).
#[pyfunction]
pub fn phi(
    session: &QSession,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64, z_den: i64, z_pow: i64,
    order: i64,
) -> QSeries {
    let mut inner = session.inner.lock().unwrap();
    let sym_q = inner.get_or_create_symbol_id("q");

    let upper_params: Vec<QMonomial> = upper.iter()
        .map(|(n, d, p)| QMonomial::new(QRat::from((*n, *d)), *p))
        .collect();
    let lower_params: Vec<QMonomial> = lower.iter()
        .map(|(n, d, p)| QMonomial::new(QRat::from((*n, *d)), *p))
        .collect();
    let z = QMonomial::new(QRat::from((z_num, z_den)), z_pow);

    let series = HypergeometricSeries { upper: upper_params, lower: lower_params, argument: z };
    let fps = qseries::hypergeometric::eval_phi(&series, sym_q, order);
    QSeries { fps }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No hypergeometric support | Expr::BasicHypergeometric variant exists but unused | Phase 1 | Symbolic node ready, needs computational backend |
| Parameters only as QRat | QMonomial (c*q^m) captures q-dependence | Phase 3 | Parameters can be q, q^2, -q, etc. |
| Manual product construction for each identity | Pattern-matching summation formulas | Phase 6 | Automated closed-form recognition |
| No bilateral series | _r psi_s with negative-index Pochhammer | Phase 6 | Enables Ramanujan's 1psi1 and related identities |

**Not yet implemented (future phases):**
- Symbolic parameters (Phase 7+): a_i as general Expr, not just QMonomial
- q-Zeilberger algorithm (v2 ADVN-01): automated summation via creative telescoping
- WZ method (v2 ADVN-02): computer proof of hypergeometric identities

## Test Data and Verification Strategy

### Verification by Expansion (Primary Strategy)

For all formulas, the verification strategy is:
1. Construct the LHS as a hypergeometric series
2. Evaluate LHS to O(q^50) via term-by-term `eval_phi`
3. Compute the RHS via the closed-form/transformation formula
4. Compare FPS coefficients exactly (QRat equality)

This is the "gold standard" test: if both sides agree to O(q^50), the formula is correct (for these specific parameters). Testing with multiple parameter choices gives high confidence.

### Test Cases from Gasper-Rahman

**q-binomial theorem (1phi0):**
```
_1 phi_0 (a ; - ; q, z) = (az;q)_inf / (z;q)_inf
```
Test with a = q^2, z = q: LHS via term-by-term, RHS = (q^3;q)_inf / (q;q)_inf.

**2phi1 with specific parameters:**
```
_2 phi_1 (q, q^2 ; q^3 ; q, q) -- simple balanced case
```
Evaluate to O(q^50) and verify.

**q-Gauss test:**
```
_2 phi_1 (q, q^2 ; q^5 ; q, q^2)
```
Here z = q^2 and c/(ab) = q^5 / (q * q^2) = q^2, so z = c/(ab). Check!
Closed form: (q^4;q)_inf * (q^3;q)_inf / [(q^5;q)_inf * (q^2;q)_inf]

**q-Vandermonde test (first form):**
```
_2 phi_1 (q^3, q^{-5} ; q^2 ; q, q^4/q^3) = _2 phi_1 (q^3, q^{-5} ; q^2 ; q, q)
```
Wait: z = c*q^n/a = q^2 * q^5 / q^3 = q^4. So first form with z = q^4.
Result: (q^{-1};q)_5 / (q^2;q)_5 -- BUT (q^{-1};q)_5 involves negative exponents.

**Simpler q-Vandermonde test:**
```
_2 phi_1 (q^2, q^{-3} ; q^4 ; q, q)  [second form, z=q]
```
Result: (q^2)^3 * (q^2;q)_3 / (q^4;q)_3 = q^6 * (1-q^2)(1-q^3)(1-q^4) / [(1-q^4)(1-q^5)(1-q^6)]
      = q^6 * (1-q^2)(1-q^3) / [(1-q^5)(1-q^6)]

**q-Saalschutz test:**
```
_3 phi_2 (q, q^2, q^{-3} ; q^4, q*q^2*q^{-2}/q^4 ; q, q)
  = _3 phi_2 (q, q^2, q^{-3} ; q^4, q/q^4 ; q, q)
  = _3 phi_2 (q, q^2, q^{-3} ; q^4, q^{-3} ; q, q)
```
Balance check: d*e = a*b*q^{1-n}/c, i.e. q^4 * q^{-3} = q * q^2 * q^{-2} / ... Need to set up carefully.

**Better q-Saalschutz test (n=2):**
```
_3 phi_2 (q, q^2, q^{-2} ; q^3, q^{1-2}*q*q^2/q^3 ; q, q)
  = _3 phi_2 (q, q^2, q^{-2} ; q^3, q^{-1}*q^3/q^3 ; q, q)
  = _3 phi_2 (q, q^2, q^{-2} ; q^3, q^{-1} ; q, q)
```
Balance condition: d*e = a*b*q^{1-n}/c => q^3 * q^{-1} = q*q^2*q^{-1}/q^3... Need careful setup.
Result: (q^2;q)_2 * (q;q)_2 / [(q^3;q)_2 * (q^{-2};q)_2] -- use finite Pochhammer.

**Heine's transformation test:**
```
_2 phi_1 (q, q^2 ; q^3 ; q, q)
```
First Heine: RHS = [(q^2;q)_inf * (q^2;q)_inf] / [(q^3;q)_inf * (q;q)_inf] * _2 phi_1(q, q ; q^2 ; q, q^2)
Both sides should match to O(q^50).

**Ramanujan's 1psi1 test:**
```
_1 psi_1 (q ; q^3 ; q, q)
```
Closed form: (q^2;q)_inf * (q^2;q)_inf * (q;q)_inf * (1;q)_inf / [(q^3;q)_inf * (1;q)_inf * (q;q)_inf * (q^2;q)_inf]
Simplify using cancellation. Compare bilateral sum (truncated) with product form.

### OEIS Sequences for Verification

- **2phi1(q,q;q^2;q,q)** -- this is a well-known q-series; check OEIS for coefficient sequences
- **Sum of q-binomial coefficients** as hypergeometric series
- **(q;q)_inf as _1 phi_0** -- already verified in Phase 2/3, but can also be expressed as _0 phi_0

### Performance Expectations

For truncation order T:
- eval_phi for 2phi1: O(T^2) due to FPS multiplication per term step -- acceptable for T <= 200
- eval_phi for 8phi7 (Watson): O(T^2 * 16) -- acceptable for T <= 100
- Summation formulas: O(T^2) for infinite product evaluation -- fast
- eval_psi bilateral: O(T^2 * sqrt(T)) -- the bilateral sum has ~2*sqrt(T) terms contributing below T

## Open Questions

1. **eval_phi Performance: FPS vs Direct Coefficient**
   - What we know: FPS multiplication per term ratio is correct but O(T^2) per call. For 2phi1 with T=200, this means ~200 FPS multiplications each O(T), giving O(T^2) = O(40000) rational ops. Acceptable.
   - What's unclear: For 8phi7 with T=200, each term step involves ~16 factor multiplications. Is O(200 * 16 * 200) = O(640000) acceptable? Probably yes for GMP rationals.
   - Recommendation: Implement FPS-based evaluation first. Profile. If too slow, implement direct coefficient computation for the balanced case (where extra_power = 0 and all parameters have zero q-power).

2. **Bilateral Series Negative-Index Terms**
   - What we know: (a;q)_{-m} = 1/(a*q^{-m};q)_m. The existing `aqprod` already handles negative orders.
   - What's unclear: How many negative-n terms contribute below truncation order T? The n-th term (n < 0) contributes to q^{n*z.power + ...}, so for z = q the terms go as q^n, requiring only |n| < T terms.
   - Recommendation: For _r psi_s, evaluate n from 0 to T (positive) and from -1 to -T (negative). Sum both parts. The total is at most 2T terms.

3. **Watson's and Bailey's Square Root Requirement**
   - What we know: Very-well-poised series require sqrt(a_1). For QMonomial a_1 = q^{2m}, sqrt = q^m (easy). For a_1 = c*q^{2m} with rational c, need sqrt(c).
   - What's unclear: How common is non-integer sqrt(c) in practice? Research use cases typically have a_1 = q^k.
   - Recommendation: Implement `QMonomial::try_sqrt()` that returns `Some` when c is a perfect rational square and power is even, `None` otherwise. Only apply Watson/Bailey when sqrt succeeds.

4. **Relationship to Expr::BasicHypergeometric**
   - What we know: The Expr variant already exists for symbolic representation. The computational struct is separate.
   - What's unclear: Should we create bidirectional conversion? When should one use Expr vs struct?
   - Recommendation: For Phase 6, provide `to_expr(arena, series) -> ExprRef` that creates the symbolic node, and `from_expr(arena, expr) -> Option<HypergeometricSeries>` that extracts parameters. But computation always goes through the struct. The Expr is for display, rendering, and symbolic simplification (future).

5. **Python API Design**
   - What we know: Upper/lower parameters need to be passed from Python. QMonomial requires (num, den, power) triple.
   - What's unclear: Should we provide a higher-level API like `phi_2_1(session, a, b, c, z, order)` for common cases?
   - Recommendation: Provide both: a general `phi(session, upper, lower, z, order)` and convenience functions `phi_2_1`, `phi_3_2`, etc. for the common cases. Same for summation functions.

## Sources

### Primary (HIGH confidence)
- [DLMF 17.6 -- Summation Formulas](https://dlmf.nist.gov/17.6) -- q-Gauss (17.6.1), q-Chu-Vandermonde (17.6.2, 17.6.3), Bailey-Daum q-Kummer (17.6.5)
- [DLMF 17.7 -- Special Cases](https://dlmf.nist.gov/17.7) -- q-Pfaff-Saalschutz (17.7.4), Jackson's q-Dixon (17.7.6), Bailey's 4phi3 (17.7.12)
- [DLMF 17.4 -- Basic Hypergeometric Functions](https://dlmf.nist.gov/17.4) -- General _r phi_s and _r psi_s definitions
- [Wolfram MathWorld: q-Hypergeometric Function](https://mathworld.wolfram.com/q-HypergeometricFunction.html) -- Definition with Gasper-Rahman convention, Heine's first transformation
- [Wolfram MathWorld: Watson-Whipple Transformation](https://mathworld.wolfram.com/Watson-WhippleTransformation.html) -- Watson's 8phi7-to-4phi3 transformation
- [Wolfram MathWorld: Bailey's Transformation](https://mathworld.wolfram.com/BaileysTransformation.html) -- Classical Bailey transformation (9F8 form)
- Existing codebase: `expr.rs` (BasicHypergeometric variant), `pochhammer.rs` (aqprod), `series/mod.rs` (FPS), `arithmetic.rs`

### Secondary (MEDIUM confidence)
- [Grokipedia: Basic Hypergeometric Series](https://grokipedia.com/page/Basic_hypergeometric_series) -- Definitions of r+1_phi_r, Heine's 2phi1, q-binomial theorem, 1psi1
- [Grokipedia: Bilateral Hypergeometric Series](https://grokipedia.com/page/bilateral_hypergeometric_series) -- _r psi_s definition, convergence, Ramanujan's 1psi1
- Gasper-Rahman equation numbers (1.4.1, 1.4.2, 1.4.3 for Heine; 1.2.22 for general _r phi_s; 2.9.1 for Bailey) -- referenced in multiple sources but not directly verified against the book

### Tertiary (LOW confidence)
- Heine's second and third transformation exact formulas -- reconstructed from the structure of the first transformation plus iteration. Standard in the literature but I did not verify against a primary text. The formulas given above should be verified during implementation by expanding both sides.
- Sears' 4phi3 exact formula -- multiple sources confirm it exists for balanced terminating 4phi3, but the precise parameter convention (which parameter is q^{-n}, how the balance condition is stated) needs verification during implementation.
- Bailey's 10phi9 general form -- the DLMF 17.7.12 form (4phi3 with q^2 base) is verified. The full 10phi9 form cited by Gasper-Rahman eq. 2.9.1 was not directly verified and has LOW confidence on exact parameter placement.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- No new dependencies; everything composes from existing Phase 2-3 infrastructure
- Mathematical definitions (_r phi_s, _r psi_s): HIGH -- DLMF definitions are authoritative
- Summation formulas (q-Gauss, q-Vandermonde, q-Saalschutz): HIGH -- DLMF equations verified
- Summation formulas (q-Kummer, q-Dixon): HIGH -- DLMF 17.6.5 and 17.7.6 verified
- Heine's transformation (first form): HIGH -- Wolfram MathWorld verified
- Heine's transformation (second, third forms): MEDIUM -- reconstructed from first form iteration, standard but not directly verified against primary text
- Sears' 4phi3: MEDIUM -- structure confirmed, exact parameter placement needs implementation verification
- Watson's transformation: HIGH -- Wolfram MathWorld exact formula verified
- Bailey's transformation: MEDIUM -- DLMF 17.7.12 form verified, full 10phi9 form LOW confidence
- Architecture: HIGH -- natural extension of existing patterns
- Pitfalls: HIGH -- well-understood computational number theory issues

**Research date:** 2026-02-13
**Valid until:** 2026-03-15 (stable mathematical domain; no fast-moving dependencies)
