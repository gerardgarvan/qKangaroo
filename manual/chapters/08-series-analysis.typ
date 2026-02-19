// 08-series-analysis.typ -- Series Analysis function reference
#import "../template.typ": *

= Series Analysis
#index[series analysis]

The series analysis functions provide tools for dissecting, factoring, and
reverse-engineering power series. Given an unknown or computed series, these
functions help researchers discover its structure: Is it an eta quotient? A
Jacobi product? A $(1+q^n)$ product? What are its extremal degrees?

These are the "detective tools" of q-series research. While the product and
partition functions *construct* series from known definitions, the series
analysis functions work in the opposite direction -- they take a series and
*decompose* it into recognizable forms.

== Function Reference

#func-entry(
  name: "sift",
  signature: "sift(series, m, j)",
  description: [
    Extract the arithmetic subsequence of coefficients from a power series.
    Given a series $f(q) = sum a_n q^n$, the call `sift(f, m, j)` returns
    a new series whose $n$-th coefficient is the $(m n + j)$-th coefficient
    of the input. This is the fundamental tool for studying partition
    congruences and arithmetic properties of $q$-series coefficients.
    #index[sift]
    #index[arithmetic subsequence]
    #index[partition congruences]
  ],
  math-def: [
    Given $f(q) = sum_(n >= 0) a_n q^n$, the sifted series is:

    $ "sift"(f, m, j) = sum_(n >= 0) a_(m n + j) q^n $

    This extracts every $m$-th coefficient, starting from position $j$.
  ],
  params: (
    ([series], [Series], [The input power series to sift]),
    ([m], [Integer], [The modulus (step size); must be a positive integer]),
    ([j], [Integer], [The residue (starting offset); must satisfy $0 <= j < m$]),
  ),
  examples: (
    ("sift(partition_gf(50), 5, 4)",
     "5 + 30*q + 135*q^2 + 490*q^3 + 1575*q^4 + 4565*q^5 + 12310*q^6 + 31185*q^7 + 75175*q^8 + 173525*q^9 + O(q^10)"),
  ),
  edge-cases: (
    [$m$ must be a positive integer.],
    [$j$ must satisfy $0 <= j < m$.],
    [The output series has approximately $floor(("order" - j) \/ m)$ terms from the original.],
    [The example demonstrates Ramanujan's congruence: all coefficients of `sift(partition_gf(N), 5, 4)` are divisible by 5, since $p(5n+4) equiv 0 space (mod 5)$.],
  ),
  related: ("findcong", "partition_gf", "rank_gf"),
)

#func-entry(
  name: "qdegree",
  signature: "qdegree(series)",
  description: [
    Return the highest power of $q$ with a nonzero coefficient in the series.
    For truncated series, this is bounded by one less than the truncation order.
    This is a utility function for quickly inspecting the support of a series.
    #index[qdegree]
    #index[degree of series]
  ],
  params: (
    ([series], [Series], [The input power series]),
  ),
  examples: (
    ("qdegree(theta3(10))",
     "9"),
    ("qdegree(partition_gf(10))",
     "9"),
  ),
  edge-cases: (
    [For the zero series, the behavior depends on the truncation order.],
    [For polynomial series (no truncation), returns the true degree.],
    [Paired with `lqdegree` to determine the full support range.],
  ),
  related: ("lqdegree", "qfactor"),
)

#func-entry(
  name: "lqdegree",
  signature: "lqdegree(series)",
  description: [
    Return the lowest power of $q$ with a nonzero coefficient in the series.
    For series beginning with the constant term 1, this returns 0. For series
    like $q + q^2 + dots.h.c$, this returns 1.
    #index[lqdegree]
    #index[valuation]
  ],
  params: (
    ([series], [Series], [The input power series]),
  ),
  examples: (
    ("lqdegree(theta3(10))",
     "0"),
    ("lqdegree(theta2(10))",
     "1"),
  ),
  edge-cases: (
    [For the zero series, returns 0.],
    [Also known as the $q$-adic valuation of the series.],
    [Paired with `qdegree` to determine the full support range.],
  ),
  related: ("qdegree", "qfactor"),
)

#func-entry(
  name: "qfactor",
  signature: "qfactor(series)",
  description: [
    Factor a polynomial series into $(1 - q^i)$ factors by top-down division.
    Returns a dictionary mapping each factor $(1 - q^i)$ to its multiplicity,
    along with a scalar coefficient and an `is_exact` flag indicating whether
    the factorization is complete.
    #index[qfactor]
    #index[factorization]
    #index[cyclotomic factorization]
  ],
  params: (
    ([series], [Series], [The input series to factor (should be a polynomial for exact results)]),
  ),
  examples: (
    ("qfactor(aqprod(1, 1, 1, 5, 20))",
     "{scalar: 1, factors: {1: 1, 2: 1, 3: 1, 4: 1, 5: 1}, is_exact: true}"),
  ),
  edge-cases: (
    [Works best on polynomial series (finite $q$-Pochhammer products).],
    [For truncated infinite series, the factorization may be approximate (`is_exact: false`).],
    [The scalar field captures any leading constant factor.],
    [The result `{1: 1, 2: 1, 3: 1, 4: 1, 5: 1}` means $(1-q)(1-q^2)(1-q^3)(1-q^4)(1-q^5)$.],
  ),
  related: ("prodmake", "etamake", "aqprod"),
)

#func-entry(
  name: "prodmake",
  signature: "prodmake(series, max_n)",
  description: [
    Find the infinite product representation of a series via the logarithmic
    derivative method. Returns exponents $a_n$ such that the series equals
    $product_(n >= 1) (1 - q^n)^(a_n)$ (up to the truncation order of the
    input series).
    #index[prodmake]
    #index[logarithmic derivative]
    #index[infinite product representation]
  ],
  math-def: [
    Given $f(q)$, find integers $a_n$ such that:

    $ f(q) = product_(n >= 1) (1 - q^n)^(a_n) $

    The algorithm takes the logarithmic derivative $f'\/f$, expands in a
    power series, and reads off the exponents from the resulting
    Dirichlet-type series.
  ],
  params: (
    ([series], [Series], [The input power series to decompose]),
    ([max_n], [Integer], [Maximum index $n$ to compute exponents for]),
  ),
  examples: (
    ("prodmake(partition_gf(50), 20)",
     "{exponents: {1: 1, 2: 1, 3: 1, ..., 20: 1}, terms_used: 20}"),
  ),
  edge-cases: (
    [`max_n` should be significantly less than the truncation order of the input series.],
    [All exponents for `partition_gf` are $1$ (since $1\/(q;q)_oo = product 1\/(1-q^n)$), confirming the product form.],
    [The method is numerical: it matches coefficients, not a symbolic proof. Use `prove_eta_id` for rigorous verification.],
  ),
  related: ("etamake", "jacprodmake", "mprodmake", "qetamake", "qfactor"),
)

#func-entry(
  name: "etamake",
  signature: "etamake(series, max_n)",
  description: [
    Find an eta-quotient representation of the series via Mobius inversion.
    An eta quotient is a product of Dedekind eta functions
    $product_d eta(d tau)^(r_d)$. The function returns the divisor-grouped
    exponents $r_d$ and any $q$-shift.
    #index[etamake]
    #index[eta quotient decomposition]
    #index[Mobius inversion]
  ],
  math-def: [
    Given $f(q)$, find integers $r_d$ and a rational $q$-shift $s$ such that:

    $ f(q) = q^s product_d eta(d tau)^(r_d) $

    where $eta(d tau) = q^(d\/24) product_(k >= 1) (1 - q^(d k))$. The
    exponents are found by applying Mobius inversion to the output of `prodmake`.
  ],
  params: (
    ([series], [Series], [The input power series to decompose]),
    ([max_n], [Integer], [Maximum divisor to search for eta factors]),
  ),
  examples: (
    ("etamake(partition_gf(50), 10)",
     "{factors: {1: -1}, q_shift: -1/24}"),
  ),
  edge-cases: (
    [The result `{1: -1}` means $eta(tau)^(-1) = 1\/eta(tau)$, confirming that `partition_gf` $= q^(-1\/24) \/ eta(tau)$.],
    [`max_n` should be less than half the truncation order for reliable results.],
    [Not every series has an eta-quotient form; the result is a best-effort approximation.],
  ),
  related: ("etaq", "prodmake", "qetamake", "prove_eta_id"),
)

#func-entry(
  name: "jacprodmake",
  signature: "jacprodmake(series, max_n)",
  description: [
    Find a Jacobi product representation of the series with automatic period
    search and residue grouping. Returns the period, grouped residue exponents,
    a scalar, and an `is_exact` flag indicating whether the product matches
    the input series exactly.
    #index[jacprodmake]
    #index[Jacobi product decomposition]
  ],
  math-def: [
    Given $f(q)$, find a period $b$ and residue exponents such that:

    $ f(q) approx product_((a,b) in "residues") J(a, b)^(e_(a,b)) $

    where $J(a, b)$ is the Jacobi triple product. The algorithm searches over
    candidate periods and uses the `prodmake` exponents grouped by residue
    classes modulo $b$.
  ],
  params: (
    ([series], [Series], [The input power series to decompose]),
    ([max_n], [Integer], [Maximum period to search]),
  ),
  examples: (
    ("jacprodmake(theta3(50), 10)",
     "{factors: {(1,8): -2, (2,8): 3, (3,8): -2}, scalar: 1, is_exact: false}"),
  ),
  edge-cases: (
    [`is_exact: true` means the Jacobi product reproduces the input series exactly (within truncation).],
    [`is_exact: false` means the best approximation was found but does not match exactly.],
    [More input terms (higher truncation order) improve the reliability of the decomposition.],
  ),
  related: ("jacprod", "prodmake", "etamake"),
)

#func-entry(
  name: "mprodmake",
  signature: "mprodmake(series, max_n)",
  description: [
    Find a $(1 + q^n)$ product representation by iterative extraction.
    Returns exponents $b_n$ such that the series equals
    $product_(n >= 1) (1 + q^n)^(b_n)$. Internally converts between
    $(1+q^n)$ and $(1-q^(2n))\/(1-q^n)$ representations.
    #index[mprodmake]
    #index[(1+q) product form]
  ],
  math-def: [
    Given $f(q)$, find integers $b_n$ such that:

    $ f(q) = product_(n >= 1) (1 + q^n)^(b_n) $

    Since $(1 + q^n) = (1 - q^(2n))\/(1 - q^n)$, this is a rearrangement of
    the $(1 - q^n)$ product form.
  ],
  params: (
    ([series], [Series], [The input power series to decompose]),
    ([max_n], [Integer], [Maximum index $n$ to compute exponents for]),
  ),
  examples: (
    ("mprodmake(distinct_parts_gf(50), 10)",
     "{1: 1, 2: 1, 3: 1, 4: 1, 5: 1, 6: 1, 7: 1, 8: 1, 9: 1, 10: 1}"),
  ),
  edge-cases: (
    [All exponents for `distinct_parts_gf` are $1$, confirming $(-q;q)_oo = product (1 + q^k)$.],
    [`max_n` should be significantly less than the truncation order.],
    [Series that are not $(1+q^n)$ products will still produce exponents, but they represent only an approximation.],
  ),
  related: ("distinct_parts_gf", "prodmake", "etamake"),
)

#func-entry(
  name: "qetamake",
  signature: "qetamake(series, max_n)",
  description: [
    Find a combined eta/q-Pochhammer product representation. This extends
    `etamake` by also searching for additional $q$-Pochhammer factors beyond
    pure eta quotients. Returns eta factors and any $q$-shift.
    #index[qetamake]
  ],
  params: (
    ([series], [Series], [The input power series to decompose]),
    ([max_n], [Integer], [Maximum index to search]),
  ),
  examples: (
    ("qetamake(partition_gf(50), 10)",
     "{factors: {1: -1}, q_shift: 0}"),
  ),
  edge-cases: (
    [For pure eta quotients, the output matches `etamake` (possibly with a different $q$-shift convention).],
    [For series with non-eta factors, `qetamake` may find a representation where `etamake` cannot.],
    [`max_n` should be less than half the truncation order for reliable results.],
  ),
  related: ("etamake", "prodmake", "etaq", "aqprod"),
)
