// 06-partitions.typ -- Partitions function reference

= Partitions
#index[partitions]
#index[partition generating function]

An integer partition of $n$ is a way of writing $n$ as a sum of positive
integers, where order does not matter. For example, the 7 partitions of 5
are: $5$, $4+1$, $3+2$, $3+1+1$, $2+2+1$, $2+1+1+1$, $1+1+1+1+1$.
The number of partitions of $n$ is denoted $p(n)$.

Partition generating functions encode the sequence ${p(n)}$ (and its
variants) as formal power series in $q$. These generating functions are
intimately connected to infinite products via Euler's identity:

$ sum_(n >= 0) p(n) q^n = product_(k >= 1) frac(1, 1 - q^k) $

q-Kangaroo provides seven partition functions: one that returns a single
integer $p(n)$, and six that return generating functions as formal power
series in $q$, including generating functions for distinct parts, odd parts,
bounded parts, rank, and crank statistics.

== Function Reference

#func-entry(
  name: "partition_count",
  signature: "partition_count(n)",
  description: [
    Compute the number of partitions $p(n)$ of the non-negative integer $n$.
    Unlike the other partition functions, this returns a single integer rather
    than a power series. Internally, it uses the pentagonal number recurrence
    for efficient computation.
    #index[partition count]
    #index[pentagonal number theorem]
  ],
  math-def: [
    $ p(n) = #[number of partitions of $n$] $

    Computed via the recurrence derived from Euler's pentagonal number theorem:

    $ p(n) = sum_(k != 0) (-1)^(k+1) p(n - k(3k-1)\/2) $

    with $p(0) = 1$ and $p(n) = 0$ for $n < 0$.
  ],
  params: (
    ([n], [Integer], [The non-negative integer to partition]),
  ),
  examples: (
    ("partition_count(5)",
     "7"),
    ("partition_count(10)",
     "42"),
    ("partition_count(100)",
     "190569292"),
    ("partition_count(200)",
     "3972999029388"),
  ),
  edge-cases: (
    [$n$ must be a non-negative integer.],
    [Returns an integer, not a series. To get the generating function, use `partition_gf`.],
    [Uses exact arbitrary-precision integer arithmetic, so large values of $n$ are supported.],
  ),
  related: ("partition_gf", "distinct_parts_gf", "odd_parts_gf"),
)

#func-entry(
  name: "partition_gf",
  signature: "partition_gf(order)",
  description: [
    Compute the partition generating function $sum_(n >= 0) p(n) q^n$ as a
    formal power series truncated to the given order. The coefficient of $q^n$
    in the result is the number of partitions of $n$.
    #index[partition generating function]
  ],
  math-def: [
    $ sum_(n >= 0) p(n) q^n = frac(1, (q; q)_oo) = product_(k >= 1) frac(1, 1 - q^k) $

    This is the reciprocal of the Euler function $(q; q)_oo$.
  ],
  params: (
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("partition_gf(10)",
     "1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + 11*q^6 + 15*q^7 + 22*q^8 + 30*q^9 + O(q^10)"),
  ),
  edge-cases: (
    [`order` must be a positive integer.],
    [The series begins $1 + q + 2q^2 + 3q^3 + 5q^4 + dots.h.c$, matching the well-known partition numbers.],
    [At `order = 1`, returns just `1 + O(q)`.],
  ),
  related: ("partition_count", "distinct_parts_gf", "odd_parts_gf", "bounded_parts_gf", "aqprod"),
)

#func-entry(
  name: "distinct_parts_gf",
  signature: "distinct_parts_gf(order)",
  description: [
    Compute the generating function for partitions into distinct parts. The
    coefficient of $q^n$ counts the number of partitions of $n$ in which all
    parts are different.
    #index[distinct parts]
  ],
  math-def: [
    $ (-q; q)_oo = product_(k >= 1) (1 + q^k) $

    Each factor $(1 + q^k)$ says: include part $k$ at most once.
  ],
  params: (
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("distinct_parts_gf(10)",
     "1 + q + q^2 + 2*q^3 + 2*q^4 + 3*q^5 + 4*q^6 + 5*q^7 + 6*q^8 + 8*q^9 + O(q^10)"),
  ),
  edge-cases: (
    [`order` must be a positive integer.],
    [By Euler's theorem, this equals `odd_parts_gf(order)` -- partitions into distinct parts are equinumerous with partitions into odd parts.],
  ),
  related: ("odd_parts_gf", "partition_gf", "mprodmake"),
)

#func-entry(
  name: "odd_parts_gf",
  signature: "odd_parts_gf(order)",
  description: [
    Compute the generating function for partitions into odd parts. The
    coefficient of $q^n$ counts the number of partitions of $n$ in which every
    part is odd (1, 3, 5, 7, ...).
    #index[odd parts]
    #index[Euler's theorem]
  ],
  math-def: [
    $ frac(1, (q; q^2)_oo) = product_(k >= 0) frac(1, 1 - q^(2k+1)) $

    By Euler's partition theorem, this equals the distinct-parts generating
    function: $(-q; q)_oo$.
  ],
  params: (
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("odd_parts_gf(10)",
     "1 + q + q^2 + 2*q^3 + 2*q^4 + 3*q^5 + 4*q^6 + 5*q^7 + 6*q^8 + 8*q^9 + O(q^10)"),
  ),
  edge-cases: (
    [`order` must be a positive integer.],
    [The output is identical to `distinct_parts_gf(order)`, confirming Euler's theorem computationally.],
  ),
  related: ("distinct_parts_gf", "partition_gf", "bounded_parts_gf"),
)

#func-entry(
  name: "bounded_parts_gf",
  signature: "bounded_parts_gf(max_part, order)",
  description: [
    Compute the generating function for partitions whose largest part is at
    most `max_part`. The coefficient of $q^n$ counts partitions of $n$ with
    all parts $<= m$.
    #index[bounded partitions]
  ],
  math-def: [
    $ product_(k=1)^m frac(1, 1 - q^k) $

    where $m$ is `max_part`. This is a finite product (a rational function of
    $q$), unlike the infinite-product partition generating function.
  ],
  params: (
    ([max_part], [Integer], [Maximum allowed part size (must be positive)]),
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("bounded_parts_gf(3, 10)",
     "1 + q + 2*q^2 + 3*q^3 + 4*q^4 + 5*q^5 + 7*q^6 + 8*q^7 + 10*q^8 + 12*q^9 + O(q^10)"),
    ("bounded_parts_gf(3, 15)",
     "1 + q + 2*q^2 + 3*q^3 + 4*q^4 + 5*q^5 + 7*q^6 + 8*q^7 + 10*q^8 + 12*q^9 + 14*q^10 + 16*q^11 + 19*q^12 + 21*q^13 + 24*q^14 + O(q^15)"),
  ),
  edge-cases: (
    [`max_part` must be a positive integer.],
    [As `max_part` increases, the result approaches `partition_gf(order)`.],
    [`bounded_parts_gf(1, order)` returns the all-ones series $1 + q + q^2 + dots.h.c + O(q^"order")$.],
  ),
  related: ("partition_gf", "qbin", "aqprod"),
)

#func-entry(
  name: "rank_gf",
  signature: "rank_gf(z_num, z_den, order)",
  description: [
    Compute the rank generating function $R(z; q)$ where $z = z_"num" \/ z_"den"$.
    The rank of a partition is defined as (largest part) $-$ (number of parts).
    The coefficient of $z^m q^n$ in $R(z; q)$ counts partitions of $n$ with rank
    $m$. At $z = 1$, the rank generating function reduces to the ordinary
    partition generating function.
    #index[partition rank]
    #index[rank generating function]
    #index[Dyson's rank]
  ],
  math-def: [
    $ R(z; q) = sum_(n >= 0) sum_m N(m, n) z^m q^n $

    where $N(m, n)$ counts partitions of $n$ with rank $m$. The rank was
    conjectured by Dyson (1944) to explain Ramanujan's partition congruences
    modulo 5 and 7.
  ],
  params: (
    ([z_num], [Integer], [Numerator of the variable $z$]),
    ([z_den], [Integer], [Denominator of the variable $z$ (must be nonzero)]),
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("rank_gf(1, 1, 10)",
     "1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + 11*q^6 + 15*q^7 + 22*q^8 + 30*q^9 + O(q^10)"),
  ),
  edge-cases: (
    [`z_den` must be nonzero.],
    [At $z = 1$ (i.e., `rank_gf(1, 1, order)`), the result equals `partition_gf(order)`.],
    [The rank generating function is useful for studying Ramanujan-type congruences.],
  ),
  related: ("crank_gf", "partition_gf", "sift", "findcong"),
)

#func-entry(
  name: "crank_gf",
  signature: "crank_gf(z_num, z_den, order)",
  description: [
    Compute the crank generating function $C(z; q)$ where $z = z_"num" \/ z_"den"$.
    The crank of a partition was introduced by Andrews and Garvan (1988) to explain
    Ramanujan's partition congruence modulo 11. Like the rank generating function,
    at $z = 1$ it reduces to the partition generating function.
    #index[partition crank]
    #index[crank generating function]
    #index[Andrews-Garvan crank]
  ],
  math-def: [
    $ C(z; q) = product_(n >= 1) frac((1 - q^n)^2, (1 - z q^n)(1 - q^n \/ z)) $

    At $z = 1$, the numerator and denominator factors cancel, giving
    $1 \/ (q; q)_oo$.
  ],
  params: (
    ([z_num], [Integer], [Numerator of the variable $z$]),
    ([z_den], [Integer], [Denominator of the variable $z$ (must be nonzero)]),
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("crank_gf(1, 1, 10)",
     "1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + 11*q^6 + 15*q^7 + 22*q^8 + 30*q^9 + O(q^10)"),
  ),
  edge-cases: (
    [`z_den` must be nonzero.],
    [At $z = 1$ (i.e., `crank_gf(1, 1, order)`), the result equals `partition_gf(order)`.],
    [The crank statistic explains all three of Ramanujan's congruences $p(5n+4) equiv 0 pmod(5)$, $p(7n+5) equiv 0 pmod(7)$, and $p(11n+6) equiv 0 pmod(11)$.],
  ),
  related: ("rank_gf", "partition_gf", "sift", "findcong"),
)
