// 08-series-analysis.typ -- Series Analysis function reference
#import "../template.typ": *

= Series Analysis
#index[series analysis]

The series analysis functions provide tools for dissecting, factoring, and
reverse-engineering power series. Given an unknown or computed series, these
functions help researchers discover its structure: Is it an eta quotient? A
Jacobi product? A $(1+q^n)$ product? What are its extremal degrees? Are its
coefficients multiplicative? Is it a "nice" formal product?

These are the "detective tools" of q-series research. While the product and
partition functions _construct_ series from known definitions, the series
analysis functions work in the opposite direction -- they take a series and
_decompose_ it into recognizable forms.

All functions that accept a series and perform structural analysis now take
an explicit $q$ variable argument and a truncation bound $T$ following
Garvan's Maple calling conventions.

== Function Reference

#func-entry(
  name: "sift",
  signature: "sift(s, q, n, k, T)",
  description: [
    Extract the arithmetic subsequence of coefficients from a power series.
    Given a series $f(q) = sum a_i q^i$, the call `sift(f, q, n, k, T)` returns
    a new series whose $i$-th coefficient is the $(n i + k)$-th coefficient
    of the input, using terms up to $q^T$. The variable $q$ is passed explicitly
    following Garvan's calling convention. This is the fundamental tool for
    studying partition congruences and arithmetic properties of $q$-series
    coefficients.
    #index[sift]
    #index[arithmetic subsequence]
    #index[partition congruences]
  ],
  math-def: [
    Given $f(q) = sum_(i >= 0) a_i q^i$, the sifted series is:

    $ "sift"(f, q, n, k, T) = sum_(i >= 0) a_(n i + k) q^i $

    This extracts every $n$-th coefficient, starting from position $k$,
    using terms of $f$ up to $q^T$.
  ],
  params: (
    ([s], [Series], [The input power series to sift]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([n], [Integer], [The modulus (step size); must be a positive integer]),
    ([k], [Integer], [The residue (starting offset); must satisfy $0 <= k < n$]),
    ([T], [Integer], [Truncation bound: input is used up to $q^T$]),
  ),
  examples: (
    ("f := partition_gf(50): sift(f, q, 5, 4, 50)",
     "... + 4565*q^5 + 1575*q^4 + 490*q^3 + 135*q^2 + 30*q + 5 + O(q^10)"),
  ),
  edge-cases: (
    [$n$ must be a positive integer.],
    [$k$ must satisfy $0 <= k < n$.],
    [$T$ controls truncation: the input series is used up to $q^T$.],
    [The output series has approximately $floor((T - k) \/ n)$ terms.],
    [The example demonstrates Ramanujan's congruence: all coefficients of `sift(f, q, 5, 4, 50)` are divisible by 5, since $p(5n+4) equiv 0 space (mod 5)$.],
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
  related: ("lqdegree", "lqdegree0", "qfactor"),
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
  related: ("qdegree", "lqdegree0", "qfactor"),
)

#func-entry(
  name: "qfactor",
  signature: "qfactor(f, q) or qfactor(f, q, T)",
  description: [
    Factor a polynomial series into $(1 - q^i)$ factors by top-down division.
    Returns a dictionary mapping each factor $(1 - q^i)$ to its multiplicity,
    along with a scalar coefficient and an `is_exact` flag indicating whether
    the factorization is complete. The variable $q$ is now passed explicitly.
    The optional $T$ parameter limits the maximum factor index to search.
    #index[qfactor]
    #index[factorization]
    #index[cyclotomic factorization]
  ],
  params: (
    ([f], [Series], [The input series to factor (should be a polynomial for exact results)]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([T], [Integer], [Optional: maximum factor index to search]),
  ),
  examples: (
    ("f := aqprod(q, q, 5): qfactor(f, q)",
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
  signature: "prodmake(f, q, T)",
  description: [
    Find the infinite product representation of a series via the logarithmic
    derivative method. Returns exponents $a_n$ such that the series equals
    $product_(n >= 1) (1 - q^n)^(a_n)$ (up to the truncation bound $T$).
    The variable $q$ is passed explicitly following Garvan's calling convention.
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
    ([f], [Series], [The input power series to decompose]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([T], [Integer], [Maximum index $n$ to compute exponents for]),
  ),
  examples: (
    ("f := partition_gf(50): prodmake(f, q, 20)",
     "{exponents: {1: -1, 2: -1, 3: -1, ..., 20: -1}, terms_used: 20}"),
  ),
  edge-cases: (
    [$T$ should be significantly less than the truncation order of the input series.],
    [All exponents for `partition_gf` are $-1$ (since $1\/(q;q)_oo = product 1\/(1-q^n)$), confirming the product form.],
    [The method is numerical: it matches coefficients, not a symbolic proof. Use `prove_eta_id` for rigorous verification.],
  ),
  related: ("etamake", "jacprodmake", "mprodmake", "qetamake", "qfactor"),
)

#func-entry(
  name: "etamake",
  signature: "etamake(f, q, T)",
  description: [
    Find an eta-quotient representation of the series via Mobius inversion.
    An eta quotient is a product of Dedekind eta functions
    $product_d eta(d tau)^(r_d)$. The function returns the divisor-grouped
    exponents $r_d$ and any $q$-shift. The variable $q$ and maximum delta $T$
    are passed explicitly following Garvan's calling convention.
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
    ([f], [Series], [The input power series to decompose]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([T], [Integer], [Maximum delta to search for eta factors]),
  ),
  examples: (
    ("f := partition_gf(50): etamake(f, q, 10)",
     "{factors: {1: -1}, q_shift: 0, is_exact: true}"),
  ),
  edge-cases: (
    [The result `{1: -1}` means $eta(tau)^(-1) = 1\/eta(tau)$, confirming that `partition_gf` $= q^(-1\/24) \/ eta(tau)$.],
    [$T$ should be less than half the truncation order for reliable results.],
    [Not every series has an eta-quotient form; the result is a best-effort approximation.],
  ),
  related: ("etaq", "prodmake", "qetamake", "prove_eta_id"),
)

#func-entry(
  name: "jacprodmake",
  signature: "jacprodmake(f, q, T) or jacprodmake(f, q, T, P)",
  description: [
    Find a Jacobi product representation of the series with automatic period
    search and residue grouping. Returns the period, grouped residue exponents,
    a scalar, and an `is_exact` flag indicating whether the product matches
    the input series exactly. The variable $q$ and search bound $T$ are passed
    explicitly. The optional $P$ parameter restricts the period search to
    divisors of $P$.
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
    ([f], [Series], [The input power series to decompose]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([T], [Integer], [Maximum period to search]),
    ([P], [Integer], [Optional: restrict period search to divisors of $P$]),
  ),
  examples: (
    ("f := jacprod(1, 5, q, 30): jacprodmake(f, q, 10)",
     "{factors: {(1,5): 1}, scalar: 1, is_exact: true}"),
  ),
  edge-cases: (
    [`is_exact: true` means the Jacobi product reproduces the input series exactly (within truncation).],
    [`is_exact: false` means the best approximation was found but does not match exactly.],
    [More input terms (higher truncation order) improve the reliability of the decomposition.],
    [Optional $P$ parameter is useful when the period is known or suspected.],
  ),
  related: ("jacprod", "prodmake", "etamake", "qs2jaccombo"),
)

#func-entry(
  name: "mprodmake",
  signature: "mprodmake(f, q, T)",
  description: [
    Find a $(1 + q^n)$ product representation by iterative extraction.
    Returns exponents $b_n$ such that the series equals
    $product_(n >= 1) (1 + q^n)^(b_n)$. Internally converts between
    $(1+q^n)$ and $(1-q^(2n))\/(1-q^n)$ representations. The variable $q$
    and maximum exponent $T$ are passed explicitly following Garvan's
    calling convention.
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
    ([f], [Series], [The input power series to decompose]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([T], [Integer], [Maximum index $n$ to compute exponents for]),
  ),
  examples: (
    ("f := distinct_parts_gf(50): mprodmake(f, q, 10)",
     "{1: 1, 2: 1, 3: 1, 4: 1, 5: 1, 6: 1, 7: 1, 8: 1, 9: 1, 10: 1}"),
  ),
  edge-cases: (
    [All exponents for `distinct_parts_gf` are $1$, confirming $(-q;q)_oo = product (1 + q^k)$.],
    [$T$ should be significantly less than the truncation order.],
    [Series that are not $(1+q^n)$ products will still produce exponents, but they represent only an approximation.],
  ),
  related: ("distinct_parts_gf", "prodmake", "etamake"),
)

#func-entry(
  name: "qetamake",
  signature: "qetamake(f, q, T)",
  description: [
    Find a combined eta/q-Pochhammer product representation. This extends
    `etamake` by also searching for additional $q$-Pochhammer factors beyond
    pure eta quotients. Returns eta factors and any $q$-shift. The variable $q$
    and maximum index $T$ are passed explicitly following Garvan's calling
    convention.
    #index[qetamake]
  ],
  params: (
    ([f], [Series], [The input power series to decompose]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([T], [Integer], [Maximum index to search]),
  ),
  examples: (
    ("f := partition_gf(50): qetamake(f, q, 10)",
     "{factors: {1: -1}, q_shift: 0, is_exact: true}"),
  ),
  edge-cases: (
    [For pure eta quotients, the output matches `etamake` (possibly with a different $q$-shift convention).],
    [For series with non-eta factors, `qetamake` may find a representation where `etamake` cannot.],
    [$T$ should be less than half the truncation order for reliable results.],
  ),
  related: ("etamake", "prodmake", "etaq", "aqprod"),
)

#func-entry(
  name: "lqdegree0",
  signature: "lqdegree0(f)",
  description: [
    Return the lowest power of $q$ with a nonzero coefficient in a formal
    power series. This is a Garvan compatibility alias for `lqdegree`,
    provided so that Maple scripts using `lqdegree0` work without modification.
    #index[lqdegree0]
    #index[Garvan compatibility]
  ],
  math-def: [
    Given $f(q) = sum_(n >= n_0) a_n q^n$ with $a_(n_0) != 0$, returns:

    $ "lqdegree0"(f) = min{n : a_n != 0} $

    This is the $q$-adic valuation of $f$.
  ],
  params: (
    ([f], [Series], [The input formal power series]),
  ),
  examples: (
    ("f := partition_gf(20): lqdegree0(f)",
     "0"),
    ("f := theta2(20): lqdegree0(f)",
     "1"),
  ),
  edge-cases: (
    [Equivalent to `lqdegree` for FPS inputs.],
    [For the zero series, returns 0.],
    [Named `lqdegree0` to match Garvan's Maple function name exactly.],
  ),
  related: ("lqdegree", "qdegree"),
)

#func-entry(
  name: "checkmult",
  signature: "checkmult(QS, T) or checkmult(QS, T, 'yes')",
  description: [
    Test whether the coefficients of a $q$-series are multiplicative.
    Checks all coprime pairs $(m, n)$ with $2 <= m, n <= T\/2$ and $m n <= T$.
    Prints `MULTIPLICATIVE` or `NOT MULTIPLICATIVE` at the first failing pair
    $(m, n)$. Returns 1 if multiplicative, 0 otherwise. The optional `'yes'`
    argument causes all failing pairs to be printed instead of stopping at the
    first failure.
    #index[checkmult]
    #index[multiplicative function]
    #index[Dirichlet series]
  ],
  math-def: [
    A sequence ${a_n}$ is _multiplicative_ if $a_1 = 1$ and

    $ a_(m n) = a_m dot a_n quad "whenever" gcd(m, n) = 1. $

    The function checks all coprime pairs $(m, n)$ with $2 <= m, n <= T\/2$
    and $m n <= T$. This is a numerical test -- it does not constitute a proof
    of multiplicativity for all $n$.
  ],
  params: (
    ([QS], [Series], [The input $q$-series to test]),
    ([T], [Integer], [Upper bound: coprime pairs tested up to $T$]),
    (['yes'], [String], [Optional: print all failing pairs instead of stopping at first]),
  ),
  examples: (
    ("f := partition_gf(50): checkmult(f, 30)",
     "NOT MULTIPLICATIVE at (2, 3)\n0"),
  ),
  edge-cases: (
    [Only checks coprime pairs -- does not constitute a proof for all $n$.],
    [Requires $a_1 = 1$ for the sequence to be multiplicative.],
    [The `'yes'` argument must be a string literal in single quotes.],
    [Useful for testing whether a $q$-series might be a Dirichlet $L$-function coefficient sequence.],
  ),
  related: ("checkprod", "sift", "findcong"),
)

#func-entry(
  name: "checkprod",
  signature: "checkprod(f, M, Q)",
  description: [
    Check whether a series $f$ is a "nice" formal product. Uses `prodmake` to
    decompose $f$ into $product (1-q^n)^(a_n)$, then checks whether all
    $|a_n| <= M$. Returns `[valuation, 1]` if the product is "nice" (all
    exponents bounded by $M$), `[valuation, max_exp]` if not nice, or
    `[[valuation, c0], -1]` if the leading coefficient is non-integer.
    #index[checkprod]
    #index[formal product]
    #index[nice product]
  ],
  math-def: [
    Given $f(q)$, compute the product decomposition
    $f(q) = product_(n >= 1) (1 - q^n)^(a_n)$. The series is a "nice"
    product if $|a_n| <= M$ for all $n$ up to the truncation order $Q$.
  ],
  params: (
    ([f], [Series], [The input series to test]),
    ([M], [Integer], [Maximum absolute exponent threshold]),
    ([Q], [Integer], [Truncation order for the product decomposition]),
  ),
  examples: (
    ("f := etaq(q, 1, 30): checkprod(f, 10, 30)",
     "[0, 1]"),
  ),
  edge-cases: (
    [Return value `[v, 1]` means nice product with $q$-adic valuation $v$.],
    [Return value `[v, m]` with $m > 1$ means not nice: $m$ is the maximum absolute exponent found.],
    [Return value `[[v, c0], -1]` means the leading coefficient $c_0$ is not $plus.minus 1$.],
    [$Q$ should match or exceed the truncation order of the input series.],
  ),
  related: ("checkmult", "prodmake", "findprod"),
)
