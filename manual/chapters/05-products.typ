// 05-products.typ -- Products function reference

= Products
#index[products]
#index[infinite products]

Infinite products lie at the heart of q-series theory. The $q$-Pochhammer
symbol $(a; q)_n$ is the fundamental building block from which nearly every
other q-series object -- Dedekind eta functions, Jacobi theta functions,
partition generating functions, and the Rogers--Ramanujan products -- is
constructed. q-Kangaroo provides seven product functions covering the
$q$-Pochhammer symbol, $q$-binomial coefficients, Dedekind eta quotients,
and several classical named products.

All product functions return truncated formal power series in $q$. The
`order` parameter controls the truncation: terms of degree $>= "order"$
are discarded and represented as $O(q^"order")$.

== Function Reference

#func-entry(
  name: "aqprod",
  signature: "aqprod(coeff_num, coeff_den, power, n_or_infinity, order)",
  description: [
    Compute the $q$-Pochhammer product $(a; q)_n$ where
    $a = ("coeff_num" \/ "coeff_den") dot q^"power"$.
    When `n_or_infinity` is the keyword `infinity`, computes the infinite product
    $(a; q)_oo$. This is the most fundamental building block in q-series theory:
    every other product function in q-Kangaroo is ultimately expressed in terms of
    `aqprod`.
    #index[q-Pochhammer symbol]
    #index[Euler function]
  ],
  math-def: [
    $ (a; q)_n = product_(k=0)^(n-1) (1 - a q^k) $

    For the infinite product:

    $ (a; q)_oo = product_(k=0)^oo (1 - a q^k) $

    With the parametrization $a = (c_1 \/ c_2) q^p$, the call
    `aqprod(c1, c2, p, n, order)` computes $(c_1\/c_2 dot q^p; q)_n$ truncated
    to $O(q^"order")$.
  ],
  params: (
    ([coeff_num], [Integer], [Numerator of the coefficient $a$]),
    ([coeff_den], [Integer], [Denominator of the coefficient $a$ (must be nonzero)]),
    ([power], [Integer], [Power of $q$ in the base: $a = (c_1\/c_2) q^"power"$]),
    ([n_or_infinity], [Integer or `infinity`], [Number of factors, or `infinity` for the infinite product]),
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("aqprod(1, 1, 1, infinity, 10)",
     "1 - q - q^2 + q^5 + q^7 + O(q^10)"),
    ("aqprod(1, 1, 1, 3, 20)",
     "1 - q - q^2 + q^4 + q^5 - q^6 + O(q^20)"),
    ("aqprod(1, 2, 1, infinity, 10)",
     "1 - 1/2*q - 1/2*q^2 - 1/4*q^3 - 1/4*q^4 - 1/8*q^6 + 1/8*q^7 + 1/8*q^9 + O(q^10)"),
  ),
  edge-cases: (
    [`coeff_den` must be nonzero; division by zero produces an error.],
    [`n_or_infinity` must be a non-negative integer or the keyword `infinity`.],
    [`order` must be a positive integer.],
    [The Euler function $(q; q)_oo$ is obtained by `aqprod(1, 1, 1, infinity, order)`.],
  ),
  related: ("etaq", "tripleprod", "jacprod", "quinprod"),
)

#func-entry(
  name: "qbin",
  signature: "qbin(n, k, order)",
  description: [
    Compute the $q$-binomial coefficient (Gaussian binomial coefficient)
    $binom(n, k)_q$. The result is a polynomial in $q$ of degree $k(n-k)$.
    The $q$-binomial coefficient is the $q$-analog of the ordinary binomial
    coefficient and arises as the generating function for partitions into at
    most $k$ parts, each at most $n - k$.
    #index[q-binomial coefficient]
    #index[Gaussian binomial]
  ],
  math-def: [
    $ binom(n, k)_q = frac((q; q)_n, (q; q)_k (q; q)_(n-k)) $

    This is a polynomial in $q$ of degree $k(n-k)$ with non-negative integer
    coefficients. At $q = 1$ it reduces to the ordinary binomial coefficient
    $binom(n, k)$.
  ],
  params: (
    ([n], [Integer], [Upper index (must be non-negative)]),
    ([k], [Integer], [Lower index (must satisfy $0 <= k <= n$)]),
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("qbin(4, 2, 20)",
     "1 + q + 2*q^2 + q^3 + q^4 + O(q^20)"),
    ("qbin(5, 0, 20)",
     "1 + O(q^20)"),
    ("qbin(5, 3, 20)",
     "1 + q + 2*q^2 + 2*q^3 + 2*q^4 + q^5 + q^6 + O(q^20)"),
  ),
  edge-cases: (
    [$k$ must satisfy $0 <= k <= n$; otherwise an error is produced.],
    [The result is a polynomial of degree $k(n-k)$, so terms beyond that degree are zero.],
    [`qbin(n, 0, order)` and `qbin(n, n, order)` both return $1$.],
  ),
  related: ("aqprod", "partition_gf", "bounded_parts_gf"),
)

#func-entry(
  name: "etaq",
  signature: "etaq(b, t, order)",
  description: [
    Compute the generalized Dedekind eta quotient. The parameter $b$ is the base
    (controlling the spacing of factors in the product) and $t$ is the exponent.
    This function computes the product form of the Dedekind eta function raised to
    the power $t$, including the leading $q$-shift $q^(b t\/24)$.
    #index[Dedekind eta function]
    #index[eta quotient]
    #index[modular forms]
  ],
  math-def: [
    $ eta_b^t = q^(b t \/ 24) product_(k=1)^oo (1 - q^(b k))^t $

    The classical Dedekind eta function is $eta(tau) = q^(1\/24) product_(k=1)^oo (1 - q^k)$,
    obtained by `etaq(1, 1, order)`. Setting $t = -1$ gives the reciprocal, which
    equals the partition generating function (up to a $q$-shift).
  ],
  params: (
    ([b], [Integer], [Base: the product runs over $q^(b k)$ for $k >= 1$. Must be positive.]),
    ([t], [Integer], [Exponent: the power to which the eta function is raised. Must be positive.]),
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("etaq(1, 1, 10)",
     "1 - q - q^2 + q^5 + q^7 + O(q^10)"),
    ("etaq(2, 1, 10)",
     "1 - q^2 - q^3 - q^4 + q^7 + q^8 + q^9 + O(q^10)"),
    ("etaq(1, 24, 10)",
     "1 - q + O(q^10)"),
  ),
  edge-cases: (
    [$b$ must be a positive integer.],
    [$t$ must be a positive integer. For negative exponents (eta quotients), use `aqprod` directly or compose products.],
    [The $q$-shift $q^(b t\/24)$ is included automatically as a rational power.],
    [`etaq(1, 1, order)` reproduces the Euler function $product(1 - q^k)$, matching `aqprod(1, 1, 1, infinity, order)`.],
  ),
  related: ("aqprod", "etamake", "prove_eta_id", "qetamake"),
)

#func-entry(
  name: "jacprod",
  signature: "jacprod(a, b, order)",
  description: [
    Compute the Jacobi triple product $J(a, b)$. This product appears throughout
    the theory of theta functions, modular forms, and partition identities. It is
    the product side of the celebrated Jacobi triple product identity.
    #index[Jacobi triple product]
  ],
  math-def: [
    $ J(a, b) = product_(k >= 1) (1 - q^(b k))(1 - q^(b k - a))(1 - q^(b(k-1) + a)) $

    Equivalently, this equals the Jacobi theta function
    $sum_(n = -oo)^oo (-1)^n q^(b binom(n,2) + a n)$ by the Jacobi triple product
    identity.
  ],
  params: (
    ([a], [Integer], [Residue parameter (the "shift" in the product)]),
    ([b], [Integer], [Period parameter (spacing between factors)]),
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("jacprod(1, 2, 10)",
     "1 - 2*q + 2*q^4 - 2*q^9 + O(q^10)"),
    ("jacprod(1, 3, 15)",
     "1 - q - q^2 + q^5 + q^7 - q^12 + O(q^15)"),
    ("jacprod(1, 5, 15)",
     "1 - q - q^4 + q^7 + q^13 + O(q^15)"),
  ),
  edge-cases: (
    [$b$ must be a positive integer.],
    [$a$ must satisfy $0 < a < b$ for a well-defined product.],
    [`jacprod(1, 2, order)` produces the theta function $theta_4(q)$.],
  ),
  related: ("tripleprod", "jacprodmake", "theta2", "theta3", "theta4"),
)

#func-entry(
  name: "tripleprod",
  signature: "tripleprod(coeff_num, coeff_den, power, order)",
  description: [
    Compute the Jacobi triple product in its $(a; q)$-factored form, where
    $a = ("coeff_num" \/ "coeff_den") dot q^"power"$. This is the product
    of three infinite $q$-Pochhammer symbols and appears in many partition
    and theta function identities.
    #index[triple product]
  ],
  math-def: [
    $ "tripleprod"(a) = (a; q)_oo dot (q\/a; q)_oo dot (q; q)_oo $

    where $a = (c_1\/c_2) dot q^p$ and the three factors are evaluated via
    `aqprod`. Note that if $a = q^k$ for $k >= 0$, the first factor $(a; q)_oo$
    contains the term $(1 - q^k dot q^k) dots.h.c$ and the overall product is
    well-defined but may evaluate to zero if $a$ is a power of $q$ (since
    $(q^k; q)_oo = 0$ when $k$ is a non-negative integer due to the factor
    $(1 - q^k dot q^0) = 0$ at $k=0$, etc.).
  ],
  params: (
    ([coeff_num], [Integer], [Numerator of the coefficient $a$]),
    ([coeff_den], [Integer], [Denominator of the coefficient $a$ (must be nonzero)]),
    ([power], [Integer], [Power of $q$ in $a = (c_1\/c_2) q^"power"$]),
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("tripleprod(1, 1, 1, 10)",
     "O(q^10)"),
    ("jacprod(1, 2, 10)",
     "1 - 2*q + 2*q^4 - 2*q^9 + O(q^10)"),
  ),
  edge-cases: (
    [`coeff_den` must be nonzero.],
    [When $a = q^k$ for a non-negative integer $k$, the factor $(q\/a; q)_oo = (q^(1-k); q)_oo$ vanishes (containing the term $1 - q^0 = 0$ when $k >= 1$, or $1 - 1 = 0$ when $k = 0$). In particular, `tripleprod(1, 1, 1, order)` yields $0$. Use `jacprod` for non-trivial triple products.],
    [The triple product is related to `jacprod` by a change of parametrization.],
  ),
  related: ("aqprod", "jacprod", "quinprod", "theta4"),
)

#func-entry(
  name: "quinprod",
  signature: "quinprod(coeff_num, coeff_den, power, order)",
  description: [
    Compute the quintuple product identity expansion. The quintuple product is a
    product of five infinite $q$-Pochhammer symbols and appears in advanced
    partition identities and modular form theory.
    #index[quintuple product]
  ],
  math-def: [
    $ "quinprod"(a) = (a; q)_oo (q\/a; q)_oo (a^2; q^2)_oo (q^2\/a^2; q^2)_oo (q; q)_oo $

    where $a = (c_1\/c_2) dot q^p$. This contains the triple product
    $(a; q)_oo (q\/a; q)_oo (q; q)_oo$ as a factor, with two additional
    terms involving $a^2$ and $q^2$.
  ],
  params: (
    ([coeff_num], [Integer], [Numerator of the coefficient $a$]),
    ([coeff_den], [Integer], [Denominator of the coefficient $a$ (must be nonzero)]),
    ([power], [Integer], [Power of $q$ in $a = (c_1\/c_2) q^"power"$]),
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("quinprod(1, 1, 1, 10)",
     "1 - q - q^2 + q^5 + q^7 + O(q^10)"),
  ),
  edge-cases: (
    [`coeff_den` must be nonzero.],
    [Like `tripleprod`, evaluates to zero when $a$ is a non-negative integer power of $q$ (due to vanishing $q$-Pochhammer factors).],
    [Shares the Euler-function factor $(q; q)_oo$ with `tripleprod`.],
  ),
  related: ("tripleprod", "aqprod", "jacprod"),
)

#func-entry(
  name: "winquist",
  signature: "winquist(a_cn, a_cd, a_p, b_cn, b_cd, b_p, order)",
  description: [
    Compute the Winquist product with two base parameters
    $a = (a_"cn" \/ a_"cd") dot q^(a_p)$ and $b = (b_"cn" \/ b_"cd") dot q^(b_p)$.
    The Winquist product is a product of 10 theta-type factors used primarily in
    partition congruence proofs, particularly for Ramanujan's congruence
    $p(11n + 6) equiv 0 pmod(11)$.
    #index[Winquist product]
    #index[partition congruences]
  ],
  math-def: [
    The Winquist product is defined as a product of 10 modified theta functions.
    With $a$ and $b$ as above, it takes the form:

    $ W(a, b) = product_"10 factors" "theta-type"(a, b, q) $

    Each factor is of the form $(x; q)_oo$ for various combinations of $a$, $b$,
    $q\/a$, $q\/b$, $a b$, $q\/(a b)$, $a\/b$, and $q b\/a$.
  ],
  params: (
    ([a_cn], [Integer], [Numerator of coefficient $a$]),
    ([a_cd], [Integer], [Denominator of coefficient $a$ (must be nonzero)]),
    ([a_p], [Integer], [Power of $q$ in $a = (a_"cn"\/a_"cd") q^(a_p)$]),
    ([b_cn], [Integer], [Numerator of coefficient $b$]),
    ([b_cd], [Integer], [Denominator of coefficient $b$ (must be nonzero)]),
    ([b_p], [Integer], [Power of $q$ in $b = (b_"cn"\/b_"cd") q^(b_p)$]),
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("winquist(1, 1, 1, 1, 1, 2, 10)",
     "O(q^10)"),
  ),
  edge-cases: (
    [Both `a_cd` and `b_cd` must be nonzero.],
    [Used primarily in partition congruence proofs; typically called with carefully chosen parameters derived from modular arithmetic.],
    [Takes 7 parameters: two $(c_n, c_d, p)$ triples for $a$ and $b$, plus the truncation order.],
  ),
  related: ("aqprod", "tripleprod", "prove_eta_id"),
)
