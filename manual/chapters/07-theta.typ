// 07-theta.typ -- Theta Functions reference
#import "../template.typ": *

= Theta Functions
#index[theta functions]
#index[Jacobi theta functions]

The Jacobi theta functions $theta_2$, $theta_3$, and $theta_4$ are
fundamental objects in number theory, combinatorics, and mathematical
physics. They arise as generating functions for sums of squares, as
building blocks for modular forms, and in the theory of elliptic functions.

q-Kangaroo implements four theta functions: the general theta series
`theta(z, q, T)` and the three "one-variable" Jacobi theta specializations
$theta_2$, $theta_3$, and $theta_4$ in the $q$-notation convention.
Each specialization is expressible both as a sum over integers and as an
infinite product via the Jacobi triple product identity.

A classical identity connecting all three theta functions is the *Jacobi
identity*:

#index("Jacobi identity")

$ theta_3(q)^4 = theta_2(q)^4 + theta_4(q)^4 $

This identity can be verified computationally in q-Kangaroo by comparing
the power series expansions of both sides.

== Function Reference

#func-entry(
  name: "theta",
  signature: "theta(z, q, T)",
  description: [
    Compute the general theta series
    $sum_(i=-T)^T z^i q^(i^2)$. The parameter $z$ can be numeric (integer or
    rational) or a $q$-monomial like $q^2$. This is the most general theta
    function form, from which the Jacobi specializations $theta_2$, $theta_3$,
    and $theta_4$ can be obtained by choosing appropriate values of $z$.
    #index[theta]
    #index[general theta series]
  ],
  math-def: [
    $ theta(z, q, T) = sum_(i=-T)^T z^i dot q^(i^2) $

    This is a finite sum (not truncated), returning an exact polynomial in $q$
    for numeric $z$, or a truncated series when $z$ is a $q$-monomial.

    When $z = 1$, this approaches $theta_3(q)$ as $T -> oo$. When $z$ is a
    $q$-monomial like $q^k$, the result is a series in $q$ combining the
    powers from $z^i = q^(k i)$ and $q^(i^2)$.
  ],
  params: (
    ([z], [Integer, Rational, or q-monomial], [The base parameter: can be a number or a $q$-monomial like $q^2$]),
    ([q], [Variable], [The series variable]),
    ([T], [Integer], [Summation bound: the sum runs from $i = -T$ to $i = T$]),
  ),
  examples: (
    ("theta(1, q, 5)",
     "2*q^4 + 2*q + 1 + O(q^5)"),
  ),
  edge-cases: (
    [When $z = 1$, the result approaches $theta_3(q)$ as $T -> oo$: `theta(1, q, 5)` gives $1 + 2q + 2q^4 + O(q^5)$.],
    [When $z$ is an unassigned symbol (not a variable with an assigned value), a warning is printed.],
    [$T$ controls the summation range, not the truncation order of the series.],
  ),
  related: ("theta2", "theta3", "theta4"),
)

#func-entry(
  name: "theta2",
  signature: "theta2(order)",
  description: [
    Compute the Jacobi theta function $theta_2(q)$. This function uses the
    standard $q$-convention where the series is expressed in integer powers
    of $q$ (absorbing the classical $q^(1\/4)$ prefactor into the series
    variable).
    #index[theta2]
    #index[sums of squares]
  ],
  math-def: [
    *Sum form:*
    $ theta_2(q) = 2 q^(1\/4) sum_(n >= 0) q^(n(n+1)) $

    In q-Kangaroo's integer-power convention, the series is:
    $ theta_2(q) = 2q + 2q^9 + 2q^25 + dots.h.c $
    (absorbing the $q^(1\/4)$ factor).

    *Product form:*
    $ theta_2(q) = 2 q^(1\/4) product_(k >= 1) (1-q^(2k))(1+q^(2k))^2 $
  ],
  params: (
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("theta2(10)",
     "2*q + 2*q^9 + O(q^10)"),
    ("theta2(50)",
     "2*q + 2*q^9 + 2*q^25 + 2*q^49 + O(q^50)"),
  ),
  edge-cases: (
    [`order` must be a positive integer.],
    [The nonzero terms appear at $q^(n^2)$ for odd $n$ (i.e., $q^1, q^9, q^25, q^49, dots.h.c$) in the integer-power convention.],
    [For small orders, the series may have very few nonzero terms (e.g., only $2q$ for `order` $<= 9$).],
  ),
  related: ("theta", "theta3", "theta4", "jacprod"),
)

#func-entry(
  name: "theta3",
  signature: "theta3(order)",
  description: [
    Compute the Jacobi theta function $theta_3(q)$. This is the generating
    function for representations as sums of squares: the coefficient of $q^n$
    in $theta_3(q)^k$ counts the number of ways to write $n$ as a sum of $k$
    squares (with sign and order).
    #index[theta3]
    #index[sums of squares]
  ],
  math-def: [
    *Sum form:*
    $ theta_3(q) = 1 + 2 sum_(n >= 1) q^(n^2) = sum_(n = -oo)^oo q^(n^2) $

    *Product form (Jacobi triple product):*
    $ theta_3(q) = product_(k >= 1) (1 - q^(2k))(1 + q^(2k-1))^2 $
  ],
  params: (
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("theta3(10)",
     "1 + 2*q + 2*q^4 + 2*q^9 + O(q^10)"),
    ("qdegree(theta3(10))",
     "9"),
    ("lqdegree(theta3(10))",
     "0"),
  ),
  edge-cases: (
    [`order` must be a positive integer.],
    [The nonzero terms appear at perfect squares: $q^0, q^1, q^4, q^9, q^16, q^25, dots.h.c$],
    [$theta_3(q)^2$ is the generating function for sums of two squares (Jacobi's two-square theorem).],
  ),
  related: ("theta", "theta2", "theta4", "jacprod"),
)

#func-entry(
  name: "theta4",
  signature: "theta4(order)",
  description: [
    Compute the Jacobi theta function $theta_4(q)$. This is related to
    $theta_3$ by $theta_4(q) = theta_3(-q)$, i.e., the series obtained by
    the substitution $q -> -q$.
    #index[theta4]
  ],
  math-def: [
    *Sum form:*
    $ theta_4(q) = 1 + 2 sum_(n >= 1) (-1)^n q^(n^2) $

    *Product form:*
    $ theta_4(q) = product_(k >= 1) (1 - q^(2k))(1 - q^(2k-1))^2 $

    Equivalently, $theta_4(q) = J(1, 2) = $ `jacprod(1, 2, q, T)`.
  ],
  params: (
    ([order], [Integer], [Truncation order for the result]),
  ),
  examples: (
    ("theta4(10)",
     "1 - 2*q + 2*q^4 - 2*q^9 + O(q^10)"),
    ("jacprod(1, 2, q, 10)",
     "-2*q^9 + 2*q^4 - 2*q + 1 + O(q^10)"),
  ),
  edge-cases: (
    [`order` must be a positive integer.],
    [Related to `theta3` by $theta_4(q) = theta_3(-q)$: coefficients at odd-index squares are negated.],
    [`jacprod(1, 2, q, T)` produces the same series as `theta4(T)`.],
  ),
  related: ("theta", "theta3", "theta2", "jacprod"),
)

== The Jacobi Identity

#index[Jacobi four-square identity]

The Jacobi identity $theta_3^4 = theta_2^4 + theta_4^4$ can be verified
computationally by comparing the two sides as truncated power series:

#repl-block("q> a := theta3(50)^4:
q> b := theta2(50)^4 + theta4(50)^4:
q> a - b
O(q^50)")

The vanishing of the difference to high order provides strong numerical
evidence for the identity; for a rigorous proof, use `prove_eta_id` on the
corresponding eta-quotient formulation.
