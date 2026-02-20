// 05-products.typ -- Products function reference
#import "../template.typ": *

= Products
#index[products]
#index[infinite products]

Infinite products lie at the heart of q-series theory. The $q$-Pochhammer
symbol $(a; q)_n$ is the fundamental building block from which nearly every
other q-series object -- Dedekind eta functions, Jacobi theta functions,
partition generating functions, and the Rogers--Ramanujan products -- is
constructed. q-Kangaroo provides twelve product functions covering the
$q$-Pochhammer symbol, $q$-binomial coefficients, Dedekind eta quotients,
several classical named products, and a Jacobi product algebra for symbolic
manipulation and decomposition.

All product functions return truncated formal power series in $q$. The
`T` parameter controls the truncation: terms of degree $>= T$
are discarded and represented as $O(q^T)$. As in Garvan's _qseries_ package,
the first argument to most product functions is a $q$-monomial (an expression
like $q^2$ or $-q$) rather than integer triples.

== Function Reference

#func-entry(
  name: "aqprod",
  signature: "aqprod(a, q, n) or aqprod(a, q, infinity, T)",
  description: [
    Compute the $q$-Pochhammer product $(a; q)_n$ where $a$ is a $q$-monomial
    (e.g., $q^2$, $-q$, or $2 q^3$), $q$ is the series variable, and $n$ is
    the number of factors. As in Garvan's _qseries_ package, the first argument
    is passed directly as a $q$-monomial rather than as an integer triple.
    When $n$ is the keyword `infinity`, use the four-argument form
    `aqprod(a, q, infinity, T)` with explicit truncation order $T$.
    This is the most fundamental building block in q-series theory: every other
    product function in q-Kangaroo is ultimately expressed in terms of `aqprod`.
    #index[q-Pochhammer symbol]
    #index[Euler function]
  ],
  math-def: [
    $ (a; q)_n = product_(k=0)^(n-1) (1 - a q^k) $

    For the infinite product:

    $ (a; q)_oo = product_(k=0)^oo (1 - a q^k) $

    The call `aqprod(a, q, n)` computes $(a; q)_n$ as an exact polynomial
    (truncated to degree $n$ from the monomial), and `aqprod(a, q, infinity, T)`
    computes $(a; q)_oo$ truncated to $O(q^T)$.
  ],
  params: (
    ([a], [q-monomial], [Base element: a $q$-monomial such as $q$, $q^2$, or $-q$]),
    ([q], [Variable], [The series variable]),
    ([n], [Integer or `infinity`], [Number of factors, or `infinity` for the infinite product]),
    ([T], [Integer], [Truncation order (required when $n$ is `infinity`)]),
  ),
  examples: (
    ("aqprod(q^2, q, 5)",
     "-q^4 - q^3 - q^2 + 1 + O(q^5)"),
    ("aqprod(q, q, infinity, 10)",
     "q^7 + q^5 - q^2 - q + 1 + O(q^10)"),
    ("aqprod(q, q, 3)",
     "-q^2 - q + 1 + O(q^3)"),
  ),
  edge-cases: (
    [The finite form `aqprod(a, q, n)` returns an exact polynomial; the truncation order is inferred from $n$ and the monomial degree.],
    [`n` must be a non-negative integer or the keyword `infinity`.],
    [The Euler function $(q; q)_oo$ is obtained by `aqprod(q, q, infinity, T)`.],
  ),
  related: ("etaq", "tripleprod", "jacprod", "quinprod"),
)

#func-entry(
  name: "qbin",
  signature: "qbin(q, m, n)",
  description: [
    Compute the $q$-binomial coefficient (Gaussian binomial coefficient)
    $binom(n, m)_q$. The result is an exact polynomial in $q$ of degree
    $m(n-m)$.
    The $q$-binomial coefficient is the $q$-analog of the ordinary binomial
    coefficient and arises as the generating function for partitions into at
    most $m$ parts, each at most $n - m$.
    #index[q-binomial coefficient]
    #index[Gaussian binomial]
  ],
  math-def: [
    $ binom(n, m)_q = frac((q; q)_n, (q; q)_m (q; q)_(n-m)) $

    This is a polynomial in $q$ of degree $m(n-m)$ with non-negative integer
    coefficients. At $q = 1$ it reduces to the ordinary binomial coefficient
    $binom(n, m)$.
  ],
  params: (
    ([q], [Variable], [The series variable]),
    ([m], [Integer], [Lower index (must satisfy $0 <= m <= n$)]),
    ([n], [Integer], [Upper index (must be non-negative)]),
  ),
  examples: (
    ("qbin(q, 2, 4)",
     "q^4 + q^3 + 2*q^2 + q + 1"),
    ("qbin(q, 0, 5)",
     "1"),
    ("qbin(q, 3, 5)",
     "q^6 + q^5 + 2*q^4 + 2*q^3 + 2*q^2 + q + 1"),
  ),
  edge-cases: (
    [$m$ must satisfy $0 <= m <= n$; otherwise an error is produced.],
    [The result is an exact polynomial of degree $m(n-m)$, so no truncation order is needed.],
    [`qbin(q, 0, n)` and `qbin(q, n, n)` both return $1$.],
  ),
  related: ("aqprod", "partition_gf", "bounded_parts_gf"),
)

#func-entry(
  name: "etaq",
  signature: "etaq(q, delta, T) or etaq(q, [deltas], T)",
  description: [
    Compute the Dedekind eta quotient. The parameter `delta` is the base
    spacing of factors in the product. The function computes the product form
    of the Dedekind eta function, including the leading $q$-shift
    $q^(delta\/24)$, truncated to $O(q^T)$.
    Also accepts a list of deltas: `etaq(q, [d1, d2, ...], T)` computes the
    product of the individual eta quotients for each delta.
    #index[Dedekind eta function]
    #index[eta quotient]
    #index[modular forms]
  ],
  math-def: [
    $ eta_delta = q^(delta \/ 24) product_(k=1)^oo (1 - q^(delta k)) $

    The classical Dedekind eta function is $eta(tau) = q^(1\/24) product_(k=1)^oo (1 - q^k)$,
    obtained by `etaq(q, 1, T)`. Setting the exponent to $-1$ (via reciprocal)
    gives the partition generating function (up to a $q$-shift).

    For the multi-delta form, $"etaq"(q, [d_1, d_2, dots.h.c], T) = product_i eta_(d_i)$.
  ],
  params: (
    ([q], [Variable], [The series variable]),
    ([delta], [Integer or List], [Base spacing (positive integer), or a list of positive integer spacings]),
    ([T], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("etaq(q, 1, 10)",
     "q^7 + q^5 - q^2 - q + 1 + O(q^10)"),
    ("etaq(q, [1,2,3], 20)",
     "-6*q^19 + ... + 1 + O(q^20)"),
  ),
  edge-cases: (
    [`delta` (or each element of the list) must be a positive integer.],
    [The $q$-shift $q^(delta\/24)$ is included automatically as a rational power.],
    [`etaq(q, 1, T)` reproduces the Euler function $product(1 - q^k)$, matching `aqprod(q, q, infinity, T)`.],
  ),
  related: ("aqprod", "etamake", "prove_eta_id", "qetamake"),
)

#func-entry(
  name: "jacprod",
  signature: "jacprod(a, b, q, T)",
  description: [
    Compute the Jacobi triple product $J(a, b)$ truncated to $O(q^T)$.
    This product appears throughout the theory of theta functions, modular
    forms, and partition identities. It is the product side of the celebrated
    Jacobi triple product identity.
    #index[Jacobi triple product]
  ],
  math-def: [
    $ J(a, b) = product_(k >= 1) (1 - q^(b k))(1 - q^(b k - a))(1 - q^(b(k-1) + a)) $

    Equivalently, this equals the Jacobi theta function
    $sum_(n = -oo)^oo (-1)^n q^(b binom(n,2) + a n)$ by the Jacobi triple product
    identity. Internally computed as `JAC(a,b)/JAC(b,3b)`.
  ],
  params: (
    ([a], [Integer], [Residue parameter (the "shift" in the product)]),
    ([b], [Integer], [Period parameter (spacing between factors)]),
    ([q], [Variable], [The series variable]),
    ([T], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("jacprod(1, 5, q, 20)",
     "... + q^7 - q^6 + q^5 - q^4 - q + 1 + O(q^20)"),
    ("jacprod(1, 2, q, 10)",
     "-2*q^9 + 2*q^4 - 2*q + 1 + O(q^10)"),
  ),
  edge-cases: (
    [$b$ must be a positive integer.],
    [$a$ must satisfy $0 < a < b$ for a well-defined product.],
    [`jacprod(1, 2, q, T)` produces the theta function $theta_4(q)$.],
  ),
  related: ("tripleprod", "jacprodmake", "JAC", "theta2", "theta3", "theta4"),
)

#func-entry(
  name: "tripleprod",
  signature: "tripleprod(z, q, T)",
  description: [
    Compute the Jacobi triple product in its $(a; q)$-factored form, where
    $z$ is a $q$-monomial. This is the product of three infinite
    $q$-Pochhammer symbols and appears in many partition and theta function
    identities.
    #index[triple product]
  ],
  math-def: [
    $ "tripleprod"(z, q, T) = (z; q)_oo dot (q\/z; q)_oo dot (q; q)_oo $

    where $z$ is a $q$-monomial such as $q^k$ or $-q$.
    Note that when $z = q^k$ for integer $k >= 0$, the product
    vanishes because one of the $q$-Pochhammer factors contains a zero term.
  ],
  params: (
    ([z], [q-monomial], [Base element: a $q$-monomial such as $-q$ or $q^2$]),
    ([q], [Variable], [The series variable]),
    ([T], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("tripleprod(-1*q, q, 20)",
     "2*q^15 + 2*q^10 + 2*q^6 + 2*q^3 + 2*q + 2 + O(q^20)"),
  ),
  edge-cases: (
    [When $z = q^k$ for a non-negative integer $k$, the product vanishes: `tripleprod(q, q, T)` yields $O(q^T)$. Use `jacprod` for non-trivial triple products.],
    [The triple product is related to `jacprod` by a change of parametrization.],
  ),
  related: ("aqprod", "jacprod", "quinprod", "theta4"),
)

#func-entry(
  name: "quinprod",
  signature: "quinprod(z, q, T)",
  description: [
    Compute the quintuple product identity expansion, where $z$ is a
    $q$-monomial. The quintuple product is a product of five infinite
    $q$-Pochhammer symbols and appears in advanced partition identities and
    modular form theory.
    #index[quintuple product]
  ],
  math-def: [
    $ "quinprod"(z, q, T) = (z; q)_oo (q\/z; q)_oo (z^2; q^2)_oo (q^2\/z^2; q^2)_oo (q; q)_oo $

    where $z$ is a $q$-monomial. This contains the triple product
    $(z; q)_oo (q\/z; q)_oo (q; q)_oo$ as a factor, with two additional
    terms involving $z^2$ and $q^2$.
  ],
  params: (
    ([z], [q-monomial], [Base element: a $q$-monomial such as $-q$ or $q^2$]),
    ([q], [Variable], [The series variable]),
    ([T], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("quinprod(q, q, 20)",
     "O(q^20)"),
  ),
  edge-cases: (
    [Like `tripleprod`, evaluates to zero when $z$ is a non-negative integer power of $q$ (due to vanishing $q$-Pochhammer factors).],
    [Shares the Euler-function factor $(q; q)_oo$ with `tripleprod`.],
  ),
  related: ("tripleprod", "aqprod", "jacprod"),
)

#func-entry(
  name: "winquist",
  signature: "winquist(a, b, q, T)",
  description: [
    Compute the Winquist product with two $q$-monomial parameters $a$ and $b$,
    truncated to $O(q^T)$.
    The Winquist product is a product of 10 theta-type factors used primarily in
    partition congruence proofs, particularly for Ramanujan's congruence
    $p(11n + 6) equiv 0 space (mod 11)$.
    #index[Winquist product]
    #index[partition congruences]
  ],
  math-def: [
    The Winquist product is defined as a product of 10 modified theta functions.
    With $q$-monomials $a$ and $b$ as base parameters, it takes the form:

    $ W(a, b, q) = product_"10 factors" "theta-type"(a, b, q) $

    Each factor is of the form $(x; q)_oo$ for various combinations of $a$, $b$,
    $q\/a$, $q\/b$, $a b$, $q\/(a b)$, $a\/b$, and $q b\/a$.
  ],
  params: (
    ([a], [q-monomial], [First base parameter: a $q$-monomial]),
    ([b], [q-monomial], [Second base parameter: a $q$-monomial]),
    ([q], [Variable], [The series variable]),
    ([T], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("winquist(q, q^2, q, 10)",
     "O(q^10)"),
  ),
  edge-cases: (
    [Used primarily in partition congruence proofs; typically called with carefully chosen parameters derived from modular arithmetic.],
    [Takes 4 parameters in Garvan form: two $q$-monomials for $a$ and $b$, the variable $q$, and truncation order $T$.],
  ),
  related: ("aqprod", "tripleprod", "prove_eta_id"),
)

== Jacobi Product Algebra
#index[Jacobi product algebra]

The Jacobi product algebra provides symbolic manipulation of infinite products
of the form $(q^a; q^b)_oo$. Rather than expanding products immediately into
power series, you can build symbolic Jacobi product expressions using `JAC`,
combine them with multiplication, division, and exponentiation, and then
convert to explicit product notation or $q$-series on demand. The
`qs2jaccombo` function performs the reverse operation: decomposing a $q$-series
into a linear combination of Jacobi products.

These tools are central to Garvan's approach to $q$-series identity discovery,
where product representations reveal the structure underlying combinatorial
identities.

#func-entry(
  name: "JAC",
  signature: "JAC(a, b)",
  description: [
    Create a Jacobi product factor $(q^a; q^b)_oo$. JacobiProduct values
    support `*`, `/`, and `^` operations for building compound product
    expressions. Use `jac2series()` or `jac2prod()` to expand to a $q$-series.
    #index[JAC]
    #index[Jacobi product]
  ],
  math-def: [
    $ "JAC"(a, b) = (q^a; q^b)_oo = product_(k=0)^oo (1 - q^(a + b k)) $

    Compound products like $"JAC"(1,5) dot "JAC"(4,5)$ represent
    $(q; q^5)_oo (q^4; q^5)_oo$, which is one side of the first
    Rogers--Ramanujan identity.
  ],
  params: (
    ([a], [Integer], [Shift parameter: the initial power of $q$ ($a >= 0$)]),
    ([b], [Integer], [Period parameter: the spacing between factors ($b > 0$)]),
  ),
  examples: (
    ("JAC(1, 5) * JAC(4, 5)",
     "JAC(1,5)*JAC(4,5)"),
    ("JAC(1, 5)^2",
     "JAC(1,5)^2"),
  ),
  edge-cases: (
    [$b$ must be a positive integer.],
    [$a = 0$ is allowed (degenerate case handled by `etaq`).],
    [Attempting addition or subtraction of JacobiProduct values produces a helpful error directing the user to `jac2series()`.],
  ),
  related: ("jac2prod", "jac2series", "qs2jaccombo", "jacprod"),
)

#func-entry(
  name: "jac2prod",
  signature: "jac2prod(JP, q, T)",
  description: [
    Convert a Jacobi product expression to explicit product notation and
    return the expanded $q$-series. Prints the product as a sequence of
    $(1 - q^k)$ factors, then returns the $q$-series expansion truncated
    to $O(q^T)$.
    #index[jac2prod]
  ],
  math-def: [
    Given a JacobiProduct value $J$, compute the finite expansion
    $ J = product_(k) (1 - q^(k)) $
    for all $k <= T$, then multiply out to obtain the truncated series.
  ],
  params: (
    ([JP], [JacobiProduct], [A Jacobi product expression (created with `JAC(a,b)` and combined with `*`, `/`, `^`)]),
    ([q], [Variable], [The series variable]),
    ([T], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("jac2prod(JAC(1,5), q, 20)",
     "(1-q)(1-q^6)(1-q^11)(1-q^16)"),
  ),
  edge-cases: (
    [`JP` must be a JacobiProduct value; passing a series or number produces an error.],
    [The function prints the product notation and returns the expanded $q$-series.],
  ),
  related: ("JAC", "jac2series", "qs2jaccombo", "jacprodmake"),
)

#func-entry(
  name: "jac2series",
  signature: "jac2series(JP, q, T)",
  description: [
    Convert a Jacobi product expression to a truncated $q$-series. Prints and
    returns the series expansion of the JacobiProduct value truncated to
    $O(q^T)$.
    #index[jac2series]
  ],
  math-def: [
    Given a JacobiProduct value $J = product_i (q^(a_i); q^(b_i))_oo^(e_i)$,
    expand each factor as a formal power series and multiply, truncating at
    $O(q^T)$.
  ],
  params: (
    ([JP], [JacobiProduct], [A Jacobi product expression (created with `JAC(a,b)` and combined with `*`, `/`, `^`)]),
    ([q], [Variable], [The series variable]),
    ([T], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("jac2series(JAC(1,5) * JAC(4,5), q, 20)",
     "... + q^7 - q^6 + q^5 - q^4 - q + 1 + O(q^20)"),
  ),
  edge-cases: (
    [`JP` must be a JacobiProduct value.],
    [The function both prints and returns the series.],
    [For compound products like `JAC(1,5) * JAC(4,5)`, each factor is expanded and then multiplied together.],
  ),
  related: ("JAC", "jac2prod", "qs2jaccombo", "jacprodmake"),
)

#func-entry(
  name: "qs2jaccombo",
  signature: "qs2jaccombo(f, q, T)",
  description: [
    Decompose a $q$-series into a linear combination of Jacobi products.
    First attempts a single-product decomposition via `jacprodmake`, then
    tries a linear combination over a candidate JAC basis. Prints the JAC
    formula if found, or "No Jacobi product decomposition found" otherwise.
    #index[qs2jaccombo]
    #index[Jacobi product decomposition]
  ],
  math-def: [
    Given a series $f(q)$, find integers $c_i$ and Jacobi products $J_i$ such
    that

    $ f(q) = sum_i c_i J_i $

    where each $J_i$ is a product of $(q^a; q^b)_oo$ factors. The
    decomposition is found by first checking if $f$ matches a single
    product (via `jacprodmake`), then searching for linear combinations
    over candidates.
  ],
  params: (
    ([f], [Series], [The input $q$-series to decompose]),
    ([q], [Variable], [The series variable]),
    ([T], [Integer], [Truncation order for the computation]),
  ),
  examples: (
    ("f := etaq(q, 1, 30): qs2jaccombo(f, q, 30)",
     "JAC(1,3)"),
  ),
  edge-cases: (
    [Returns the JacobiProduct expression if decomposition succeeds.],
    [Not all series have a Jacobi product decomposition; the function reports failure explicitly.],
    [The candidate basis is generated from periods identified by `jacprodmake`.],
  ),
  related: ("JAC", "jac2series", "jac2prod", "jacprodmake"),
)
