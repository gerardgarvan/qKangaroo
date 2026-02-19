// 10-hypergeometric.typ -- Basic Hypergeometric Series function reference
#import "../template.typ": *

= Basic Hypergeometric Series
#index[basic hypergeometric series]

The basic hypergeometric series ${}_r phi_s$ and the bilateral basic
hypergeometric series ${}_r psi_s$ are fundamental objects in q-series
theory. The general ${}_r phi_s$ is defined by
$ attach(, tl: r) phi_s
  mat(a_1, a_2, ..., a_r;
      b_1, b_2, ..., b_s;
      q, z)
  = sum_(n=0)^oo
    frac(
      (a_1 ; q)_n (a_2 ; q)_n dots.c (a_r ; q)_n,
      (b_1 ; q)_n (b_2 ; q)_n dots.c (b_s ; q)_n (q ; q)_n
    )
    [(-1)^n q^(binom(n, 2))]^(1 + s - r) z^n $
where $(a ; q)_n = product_(k=0)^(n-1)(1 - a q^k)$ is the q-Pochhammer
symbol. When $r <= s + 1$ and $|z| < 1$ the series converges absolutely.
The factor $[(-1)^n q^(binom(n,2))]^(1+s-r)$ reduces to $1$ for the
most common case $r = s + 1$ ("balanced" series).

In q-Kangaroo, the upper and lower parameters are each specified as lists
of `(num, den, pow)` triples, where each triple represents the parameter
$(n"um" slash d"en") dot q^("pow")$. The argument $z$ is similarly encoded
as `z_num`, `z_den`, `z_pow` representing $z = (z_"num" slash z_"den") dot q^(z_"pow")$.

#index[q-Pochhammer symbol]

#func-entry(
  name: "phi",
  signature: "phi(upper_list, lower_list, z_num, z_den, z_pow, order)",
  description: [
    Evaluate the basic hypergeometric series
    ${}_r phi_s ("upper" ; "lower" ; q, z)$
    where $r$ and $s$ are determined by the lengths of the parameter lists.
    Each parameter is a `(num, den, pow)` triple encoding
    $(n"um" slash d"en") dot q^("pow")$.
  ],
  math-def: [
    $ attach(, tl: r) phi_s
      mat(a_1, ..., a_r; b_1, ..., b_s; q, z)
      = sum_(n=0)^oo
        frac(
          product_(j=1)^r (a_j ; q)_n,
          product_(j=1)^s (b_j ; q)_n dot (q ; q)_n
        )
        [(-1)^n q^(binom(n,2))]^(1+s-r) z^n $
  ],
  params: (
    ([upper\_list], [List of (Int, Int, Int)], [Upper parameters as `(num, den, pow)` triples]),
    ([lower\_list], [List of (Int, Int, Int)], [Lower parameters as `(num, den, pow)` triples]),
    ([z\_num], [Integer], [Numerator of $z$]),
    ([z\_den], [Integer], [Denominator of $z$]),
    ([z\_pow], [Integer], [Power of $q$ in $z$]),
    ([order], [Integer], [Truncation order]),
  ),
  examples: (
    ("phi([(1,1,1)], [(1,1,2)], 1, 1, 1, 10)",
     "1 + ... + O(q^10)"),
    ("phi([], [], 1, 1, 0, 10)",
     "1/(q;q)_inf truncated to O(q^10)"),
  ),
  edge-cases: (
    [Either `upper_list` or `lower_list` may be empty (yielding a ${}_0 phi_s$ or ${}_r phi_0$).],
    [Division by zero occurs if a lower parameter equals $q^k$ for some $0 <= k < n$, causing $(b_j ; q)_n = 0$.],
    [The series terminates if an upper parameter equals $q^(-m)$ for a non-negative integer $m$.],
  ),
  related: ("psi", "try_summation", "heine1", "heine2", "heine3"),
)

#func-entry(
  name: "psi",
  signature: "psi(upper_list, lower_list, z_num, z_den, z_pow, order)",
  description: [
    Evaluate the bilateral basic hypergeometric series
    ${}_r psi_s ("upper" ; "lower" ; q, z)$,
    which sums over all integers $n in ZZ$ (both positive and negative indices).
    The bilateral series is defined by
    $ attach(, tl: r) psi_s
      mat(a_1, ..., a_r; b_1, ..., b_s; q, z)
      = sum_(n = -oo)^oo
        frac(
          product_(j=1)^r (a_j ; q)_n,
          product_(j=1)^s (b_j ; q)_n
        )
        z^n $
    where for negative $n$, the q-Pochhammer symbols are defined via
    $(a ; q)_(-n) = 1 slash (a q^(-n) ; q)_n$.
    #index[bilateral series]
  ],
  math-def: [
    $ attach(, tl: r) psi_s
      mat(a_1, ..., a_r; b_1, ..., b_s; q, z)
      = sum_(n = -oo)^oo
        frac(product_(j=1)^r (a_j ; q)_n,
             product_(j=1)^s (b_j ; q)_n) z^n $
  ],
  params: (
    ([upper\_list], [List of (Int, Int, Int)], [Upper parameters as `(num, den, pow)` triples]),
    ([lower\_list], [List of (Int, Int, Int)], [Lower parameters as `(num, den, pow)` triples]),
    ([z\_num], [Integer], [Numerator of $z$]),
    ([z\_den], [Integer], [Denominator of $z$]),
    ([z\_pow], [Integer], [Power of $q$ in $z$]),
    ([order], [Integer], [Truncation order]),
  ),
  examples: (
    ("psi([(1,1,1)], [(1,1,2)], 1, 1, 1, 10)",
     "bilateral sum truncated to order 10"),
  ),
  edge-cases: (
    [The bilateral sum includes negative powers of $q$ if $z$ has non-positive $q$-power.],
    [Convergence requires $|b_1 dots.c b_s slash (a_1 dots.c a_r)| < |z| < 1$ when $r = s$.],
  ),
  related: ("phi", "try_summation"),
)

#func-entry(
  name: "try_summation",
  signature: "try_summation(upper_list, lower_list, z_num, z_den, z_pow, order)",
  description: [
    Attempt to find a closed-form summation for a basic hypergeometric series
    by matching against the five classical summation formulas:
    #index[q-Gauss sum]
    #index[q-Vandermonde]
    #index[q-Saalschutz]
    #index[summation formula]

    + *q-Gauss sum* -- evaluates ${}_2 phi_1 (a, b ; c ; q, c slash (a b))$
    + *q-Vandermonde (q-Chu--Vandermonde)* -- evaluates a terminating ${}_2 phi_1$
    + *q-Saalschutz (q-Pfaff--Saalschutz)* -- evaluates a balanced terminating ${}_3 phi_2$
    + *q-Kummer* -- evaluates certain ${}_2 phi_1$ at $z = -1$
    + *q-Dixon* -- evaluates a very-well-poised ${}_4 phi_3$

    Returns a closed-form product expression if a formula applies, or `null`
    if no known summation matches.
  ],
  params: (
    ([upper\_list], [List of (Int, Int, Int)], [Upper parameters as `(num, den, pow)` triples]),
    ([lower\_list], [List of (Int, Int, Int)], [Lower parameters as `(num, den, pow)` triples]),
    ([z\_num], [Integer], [Numerator of $z$]),
    ([z\_den], [Integer], [Denominator of $z$]),
    ([z\_pow], [Integer], [Power of $q$ in $z$]),
    ([order], [Integer], [Truncation order for verification]),
  ),
  examples: (
    ("try_summation([(1,1,1), (1,1,2)], [(1,1,3)], 1, 1, 1, 10)",
     "closed-form product (or null if no formula applies)"),
  ),
  edge-cases: (
    [Returns `null` if no classical summation formula matches -- this does not mean no closed form exists.],
    [The order parameter is used to verify the candidate closed form against the series.],
  ),
  related: ("phi", "q_gosper"),
)

== Heine Transformations
#index[Heine transformation]

Heine's three classical transformations relate different ${}_2 phi_1$ series.
Given a ${}_2 phi_1 (a, b ; c ; q, z)$, each transformation rewrites it as
a product of infinite q-Pochhammer quotients times a new ${}_2 phi_1$ with
different parameters. These transformations are fundamental tools for
simplifying and evaluating basic hypergeometric series.

All three functions share the same signature pattern: they accept the same
six parameters as `phi` and return the transformed series. The input must
be a valid ${}_2 phi_1$ (exactly 2 upper parameters and 1 lower parameter).

#func-entry(
  name: "heine1",
  signature: "heine1(upper_list, lower_list, z_num, z_den, z_pow, order)",
  description: [
    Apply Heine's first transformation to a ${}_2 phi_1$ series.
    Transforms ${}_2 phi_1 (a, b ; c ; q, z)$ into a q-Pochhammer product
    prefactor times another ${}_2 phi_1$ with rearranged parameters:
    $ attach(, tl: 2) phi_1 (a, b ; c ; q, z)
      = frac((b ; q)_oo (a z ; q)_oo, (c ; q)_oo (z ; q)_oo)
        dot attach(, tl: 2) phi_1 (c slash b, z ; a z ; q, b) $
  ],
  params: (
    ([upper\_list], [List of (Int, Int, Int)], [Two upper parameters `(a, b)` as triples]),
    ([lower\_list], [List of (Int, Int, Int)], [One lower parameter `(c)` as a triple]),
    ([z\_num], [Integer], [Numerator of $z$]),
    ([z\_den], [Integer], [Denominator of $z$]),
    ([z\_pow], [Integer], [Power of $q$ in $z$]),
    ([order], [Integer], [Truncation order]),
  ),
  examples: (
    ("heine1([(1,1,1), (1,1,2)], [(1,1,3)], 1, 1, 1, 10)",
     "(prefactor, transformed_series)"),
  ),
  edge-cases: (
    [Requires exactly 2 upper and 1 lower parameter (a ${}_2 phi_1$).],
    [The prefactor involves infinite products that must converge within the truncation order.],
  ),
  related: ("heine2", "heine3", "phi"),
)

#func-entry(
  name: "heine2",
  signature: "heine2(upper_list, lower_list, z_num, z_den, z_pow, order)",
  description: [
    Apply Heine's second transformation to a ${}_2 phi_1$ series.
    A different rearrangement of the ${}_2 phi_1$ parameters:
    $ attach(, tl: 2) phi_1 (a, b ; c ; q, z)
      = frac((a b z slash c ; q)_oo, (z ; q)_oo)
        dot attach(, tl: 2) phi_1 (c slash a, c slash b ; c ; q, a b z slash c) $
  ],
  params: (
    ([upper\_list], [List of (Int, Int, Int)], [Two upper parameters as triples]),
    ([lower\_list], [List of (Int, Int, Int)], [One lower parameter as a triple]),
    ([z\_num], [Integer], [Numerator of $z$]),
    ([z\_den], [Integer], [Denominator of $z$]),
    ([z\_pow], [Integer], [Power of $q$ in $z$]),
    ([order], [Integer], [Truncation order]),
  ),
  examples: (
    ("heine2([(1,1,1), (1,1,2)], [(1,1,3)], 1, 1, 1, 10)",
     "(prefactor, transformed_series)"),
  ),
  edge-cases: (
    [Requires exactly 2 upper and 1 lower parameter.],
    [The transformed $z$-argument is $a b z slash c$, which may change convergence behavior.],
  ),
  related: ("heine1", "heine3", "phi"),
)

#func-entry(
  name: "heine3",
  signature: "heine3(upper_list, lower_list, z_num, z_den, z_pow, order)",
  description: [
    Apply Heine's third transformation to a ${}_2 phi_1$ series.
    Transforms into a ratio of infinite products times a ${}_2 phi_1$
    with yet another parameter arrangement:
    $ attach(, tl: 2) phi_1 (a, b ; c ; q, z)
      = frac((a z ; q)_oo (b z ; q)_oo, (c ; q)_oo (z ; q)_oo)
        dot frac((c ; q)_oo, (a b z slash c ; q)_oo)
        dot attach(, tl: 2) phi_1 (a b z slash c, z ; b z ; q, c slash(a z)) $
  ],
  params: (
    ([upper\_list], [List of (Int, Int, Int)], [Two upper parameters as triples]),
    ([lower\_list], [List of (Int, Int, Int)], [One lower parameter as a triple]),
    ([z\_num], [Integer], [Numerator of $z$]),
    ([z\_den], [Integer], [Denominator of $z$]),
    ([z\_pow], [Integer], [Power of $q$ in $z$]),
    ([order], [Integer], [Truncation order]),
  ),
  examples: (
    ("heine3([(1,1,1), (1,1,2)], [(1,1,3)], 1, 1, 1, 10)",
     "(prefactor, transformed_series)"),
  ),
  edge-cases: (
    [Requires exactly 2 upper and 1 lower parameter.],
    [The prefactor involves multiple infinite products and their quotients.],
  ),
  related: ("heine1", "heine2", "phi"),
)

== Advanced Transformations
#index[Sears transformation]
#index[Watson transformation]

Beyond the classical Heine transformations, two important higher-order
transformations are available for balanced and very-well-poised series.

#func-entry(
  name: "sears_transform",
  signature: "sears_transform(upper_list, lower_list, z_num, z_den, z_pow, order)",
  description: [
    Apply Sears' transformation to a balanced terminating ${}_4 phi_3$ series.
    Transforms one balanced ${}_4 phi_3$ into an equivalent ${}_4 phi_3$ with
    rearranged parameters. A series ${}_4 phi_3$ is _balanced_ when
    $q a_1 a_2 a_3 a_4 = b_1 b_2 b_3 z$ and one of the upper parameters is
    $q^(-n)$ (terminating condition).
  ],
  params: (
    ([upper\_list], [List of (Int, Int, Int)], [Four upper parameters as triples]),
    ([lower\_list], [List of (Int, Int, Int)], [Three lower parameters as triples]),
    ([z\_num], [Integer], [Numerator of $z$]),
    ([z\_den], [Integer], [Denominator of $z$]),
    ([z\_pow], [Integer], [Power of $q$ in $z$]),
    ([order], [Integer], [Truncation order]),
  ),
  examples: (
    ("sears_transform([(1,1,0),(1,1,1),(1,1,2),(1,1,-3)], [(1,1,3),(1,1,4),(1,1,5)], 1, 1, 0, 10)",
     "(prefactor, transformed_4_phi_3)"),
  ),
  edge-cases: (
    [The series must satisfy the balanced condition; behavior is undefined otherwise.],
    [Requires exactly 4 upper and 3 lower parameters.],
  ),
  related: ("watson_transform", "phi", "find_transformation_chain"),
)

#func-entry(
  name: "watson_transform",
  signature: "watson_transform(upper_list, lower_list, z_num, z_den, z_pow, order)",
  description: [
    Apply Watson's transformation to reduce a very-well-poised ${}_8 phi_7$
    to a balanced ${}_4 phi_3$ via the Watson--Whipple identity. A series is
    _very-well-poised_ when the upper parameters satisfy certain symmetry
    conditions involving $a_1^(1 slash 2)$ and $-a_1^(1 slash 2)$.

    This is one of the deepest classical transformations and is the basis for
    many partition and Rogers--Ramanujan type identities.
  ],
  params: (
    ([upper\_list], [List of (Int, Int, Int)], [Eight upper parameters as triples]),
    ([lower\_list], [List of (Int, Int, Int)], [Seven lower parameters as triples]),
    ([z\_num], [Integer], [Numerator of $z$]),
    ([z\_den], [Integer], [Denominator of $z$]),
    ([z\_pow], [Integer], [Power of $q$ in $z$]),
    ([order], [Integer], [Truncation order]),
  ),
  examples: (
    ("watson_transform(upper, lower, 1, 1, 0, 10)",
     "(prefactor, reduced_4_phi_3)"),
  ),
  edge-cases: (
    [The series must satisfy the very-well-poised condition.],
    [Requires exactly 8 upper and 7 lower parameters.],
  ),
  related: ("sears_transform", "phi", "find_transformation_chain"),
)

#func-entry(
  name: "find_transformation_chain",
  signature: "find_transformation_chain(src_upper, src_lower, src_z_n, src_z_d, src_z_p, tgt_upper, tgt_lower, tgt_z_n, tgt_z_d, tgt_z_p, max_depth, order)",
  description: [
    Search for a chain of Heine, Sears, and Watson transformations connecting
    a source hypergeometric series to a target, using breadth-first search
    (BFS) up to the specified maximum depth. At each step, all applicable
    transformations are tried, and the resulting series is compared against
    the target up to the truncation order. Returns the list of transformation
    steps if a path is found, or an empty list if no chain of length
    $<= "max_depth"$ connects the two series.
  ],
  params: (
    ([src\_upper], [List of (Int, Int, Int)], [Source series upper parameters]),
    ([src\_lower], [List of (Int, Int, Int)], [Source series lower parameters]),
    ([src\_z\_n], [Integer], [Source $z$ numerator]),
    ([src\_z\_d], [Integer], [Source $z$ denominator]),
    ([src\_z\_p], [Integer], [Source $z$ power of $q$]),
    ([tgt\_upper], [List of (Int, Int, Int)], [Target series upper parameters]),
    ([tgt\_lower], [List of (Int, Int, Int)], [Target series lower parameters]),
    ([tgt\_z\_n], [Integer], [Target $z$ numerator]),
    ([tgt\_z\_d], [Integer], [Target $z$ denominator]),
    ([tgt\_z\_p], [Integer], [Target $z$ power of $q$]),
    ([max\_depth], [Integer], [Maximum number of transformation steps]),
    ([order], [Integer], [Truncation order for series comparison]),
  ),
  examples: (
    ("find_transformation_chain([(1,1,1),(1,1,2)], [(1,1,3)], 1,1,1, [(1,1,2),(1,1,1)], [(1,1,3)], 1,1,1, 3, 10)",
     "list of transformation steps (or empty if no path found)"),
  ),
  edge-cases: (
    [Returns an empty list if no chain of length $<= "max_depth"$ connects the source to the target.],
    [Search time grows exponentially with `max_depth` since each step may branch into multiple transformations.],
    [The order parameter controls how many coefficients are compared -- higher values give more confidence but cost more.],
  ),
  related: ("heine1", "heine2", "heine3", "sears_transform", "watson_transform"),
)
