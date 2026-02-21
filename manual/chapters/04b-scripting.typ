// 04b-scripting.typ -- Scripting Language (v3.0)
#import "../template.typ": *

= Scripting Language
#index[scripting language]

q-Kangaroo v3.0 adds control-flow statements, procedure definitions,
expression operations, and symbolic product support. This chapter
documents the complete scripting language in workflow order: variables and
loops, conditionals, procedures, expression and polynomial operations,
and symbolic products.

== For Loops
#index[for loops]
#index-main[for]

Syntax: `for var from start to end [by step] do body od`

The default step is 1. A negative step counts down. Body statements are
separated by `;` (print) or `:` (suppress). The loop variable persists
after the loop ends (Maple behaviour).

#repl-block("q> s := 0: for k from 1 to 5 do s := s + k: od: s
15")

#repl("for k from 10 to 1 by -2 do k od", "2")

=== Worked Example: Pentagonal Number Series
#index[pentagonal number theorem]

Euler's pentagonal theorem states that
$(q; q)_oo = sum_(n = -oo)^oo (-1)^n q^(n(3n-1)\/2)$.
We verify this by computing the partial sum for $n in [-5, 5]$ and
comparing with `aqprod`.

_cf.~Garvan, "A q-Product Tutorial for a q-Series Maple Package"
(1998), Section~4._

#repl-block("q> s := 0:
q> for n from -5 to 5 do
q>   s := s + (-1)^n * q^(n*(3*n-1)/2):
q> od:
q> series(s, q, 20)
-q^15 - q^12 + q^7 + q^5 - q^2 - q + 1 + O(q^20)")

#repl("aqprod(q, q, infinity, 20)",
  "-q^15 - q^12 + q^7 + q^5 - q^2 - q + 1 + O(q^20)")

The two results agree, confirming the identity to $O(q^20)$.

== Boolean and Comparison Operators
#index[boolean operators]
#index[comparison operators]

=== Comparison Operators

#table(
  columns: (auto, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header([*Operator*], [*Meaning*]),
  [`=`], [Equal],
  [`<>`], [Not equal],
  [`<`], [Less than],
  [`>`], [Greater than],
  [`<=`], [Less than or equal],
  [`>=`], [Greater than or equal],
)

Comparisons return `true` or `false`.

#repl("3 > 2", "true")

=== Boolean Operators

#table(
  columns: (auto, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header([*Operator*], [*Meaning*]),
  [`not`], [Logical negation (binds tightest)],
  [`and`], [Logical conjunction (short-circuit)],
  [`or`], [Logical disjunction (short-circuit)],
)

Precedence: `not` > `and` > `or`. Both `and` and `or` use short-circuit
evaluation.

#repl("2 > 1 and 3 > 2", "true")

#repl("not (1 = 2)", "true")

== If/Elif/Else Conditionals
#index[conditionals]
#index-main[if]

Syntax: `if cond then body [elif cond then body] [else body] fi`

Multiple `elif` branches are allowed. The `else` branch is optional.

#repl("if 3 > 2 then 1 else 0 fi", "1")

#repl-block("q> n := 7:
q> if n < 5 then 0 elif n < 10 then 1 else 2 fi
1")

== Procedures
#index[procedures]
#index-main[proc]

Syntax: `name := proc(params) [local vars;] [option remember;] body; end`

- *Local variables* are scoped to the procedure and do not affect the
  global environment.
- *`RETURN(value)`* exits the procedure immediately with the given value.
- *`option remember`* enables memoization: results are cached for
  repeated calls with the same arguments.

#repl("double := proc(x) 2*x; end: double(21)", "42")

#repl-block("q> abs := proc(x)
q>   if x < 0 then RETURN(-x) fi;
q>   x;
q> end:
q> abs(-5)
5")

=== Worked Example: Memoized Partition Recurrence
#index[partition recurrence]
#index[memoization]

Euler's recurrence computes $p(n)$ using generalised pentagonal numbers.
This procedure uses `option remember` for efficient recursion.

_cf.~Garvan, "A q-Product Tutorial for a q-Series Maple Package"
(1998), Section~7._

#repl-block("q> prec := proc(n)
q>   local k, s, p1, p2;
q>   option remember;
q>   if n < 0 then RETURN(0) fi;
q>   if n = 0 then RETURN(1) fi;
q>   s := 0:
q>   for k from 1 to n do
q>     p1 := floor(k*(3*k-1)/2);
q>     p2 := floor(k*(3*k+1)/2);
q>     if p1 > n then RETURN(s) fi;
q>     s := s + (-1)^(k+1) * (prec(n - p1) + prec(n - p2)):
q>   od:
q>   s;
q> end:
q> prec(20)
627")

This matches `numbpart(20)`, confirming correctness. The example
demonstrates: `proc`, `local`, `option remember`, `RETURN`, `for`, `if`,
and recursion.

== Expression Operations
#index[expression operations]

#func-entry(
  name: "series",
  signature: "series(expr, q, T)",
  description: [
    Truncate a $q$-series to $O(q^T)$. Accepts Series, JacobiProduct,
    Integer, or Rational values. Uses `min(T, original_order)` so it
    never extends beyond computed data.
    #index[series truncation]
  ],
  params: (
    ([expr], [Series/JacobiProduct/Integer/Rational], [The expression to truncate]),
    ([q], [Variable], [The series variable]),
    ([T], [Integer], [New truncation order]),
  ),
  examples: (
    ("f := aqprod(q, q, infinity, 50): series(f, q, 10)",
     "q^7 + q^5 - q^2 - q + 1 + O(q^10)"),
  ),
  related: ("expand",),
)

#func-entry(
  name: "expand",
  signature: "expand(expr) or expand(JP, q, T)",
  description: [
    Expand a JacobiProduct into a $q$-series. The one-argument form uses
    the current default truncation order. The three-argument form
    specifies the variable and truncation explicitly.
    #index[expand]
  ],
  params: (
    ([expr], [JacobiProduct/Series], [The expression to expand]),
    ([q], [Variable], [The series variable (3-argument form)]),
    ([T], [Integer], [Truncation order (3-argument form)]),
  ),
  examples: (
    ("expand(JAC(1,5) * JAC(4,5), q, 20)",
     "-3*q^19 + 2*q^17 - ... + q^5 - q^4 - q + 1 + O(q^20)"),
  ),
  related: ("series", "JAC", "jac2series"),
)

== Polynomial Operations
#index[polynomial operations]

#func-entry(
  name: "factor",
  signature: "factor(poly)",
  description: [
    Factor a polynomial in $q$ into irreducible (cyclotomic) factors over
    the integers. The input must be an exact polynomial (not a truncated
    series).
    #index[polynomial factoring]
    #index[cyclotomic factoring]
  ],
  params: (
    ([poly], [Polynomial], [An exact polynomial in $q$]),
  ),
  examples: (
    ("factor(q^4 - 1)", "(q-1)(q+1)(q^2+1)"),
  ),
  related: ("subs", "qfactor"),
)

#func-entry(
  name: "subs",
  signature: "subs(q=value, expr)",
  description: [
    Substitute a value for the variable $q$ in a series or polynomial.
    Returns the evaluated result. If the variable name does not match,
    the expression is returned unchanged.
    #index[substitution]
  ],
  params: (
    ([q=value], [Assignment], [Variable and replacement value]),
    ([expr], [Series/Polynomial], [Expression to evaluate]),
  ),
  examples: (
    ("f := 1 + q + q^2: subs(q=2, f)", "7"),
  ),
  related: ("factor", "series"),
)

== Number Theory Functions
#index[number theory]

#func-entry(
  name: "floor",
  signature: "floor(x)",
  description: [
    Return the greatest integer not exceeding $x$.
    #index[floor function]
  ],
  params: (
    ([x], [Integer/Rational], [A number]),
  ),
  examples: (
    ("floor(7/3)", "2"),
  ),
  related: ("legendre",),
)

#func-entry(
  name: "legendre",
  signature: "legendre(m, p)",
  description: [
    Compute the Legendre symbol $(m\/p)$ for odd prime $p$. Returns $-1$,
    $0$, or $1$. Also available as `L(m, p)`.
    #index[Legendre symbol]
  ],
  params: (
    ([m], [Integer], [Numerator]),
    ([p], [Integer], [An odd prime $>= 3$]),
  ),
  examples: (
    ("legendre(2, 7)", "1"),
  ),
  related: ("floor",),
)

== Symbolic Products
#index[symbolic products]
#index[bivariate series]

When the first argument to `tripleprod`, `quinprod`, or `winquist` is a
_symbol_ (an undefined variable such as `z`) rather than a $q$-monomial,
the function returns a bivariate Laurent polynomial: a polynomial in
the symbolic variable with $q$-series coefficients.

#repl("tripleprod(z, q, 10)",
  "q^6*z^4 - q^3*z^3 + q*z^2 - z + 1 + O(q^10) - q*z^(-1) + q^3*z^(-2) - q^6*z^(-3) + O(q^10)")

#repl-block("q> quinprod(z, q, 10)
q^7*z^6 - q^5*z^5 + q^2*z^3 - q*z^2 + 1 + O(q^10) - z^(-1) + q*z^(-3) - q^2*z^(-4) + q^5*z^(-6) - q^7*z^(-7) + O(q^10)")

The `winquist` function produces a trivariate result (Laurent polynomial
in two symbolic variables `a`, `b` with $q$-series coefficients) when
both base arguments are symbols.

=== Worked Example: Jacobi Triple Product Identity
#index[Jacobi triple product identity]

The Jacobi triple product identity states:

$ product_(k >= 1) (1 - q^k)(1 - z q^k)(1 - z^(-1) q^(k-1))
  = sum_(n = -oo)^oo (-1)^n z^n q^(n(n-1)\/2) $

We verify this by computing `tripleprod(z, q, 10)` and checking
that the $z^n$ coefficient of the result is $(-1)^n q^(n(n-1)\/2)$.

_cf.~Garvan, "A q-Product Tutorial for a q-Series Maple Package"
(1998), Section~2._

#repl("tripleprod(z, q, 10)",
  "q^6*z^4 - q^3*z^3 + q*z^2 - z + 1 + O(q^10) - q*z^(-1) + q^3*z^(-2) - q^6*z^(-3) + O(q^10)")

Reading off the coefficients: $z^0 = 1$, $z^1 = -1 dot q^0 = -1$
(but displayed as $-z$, matching $(-1)^1 q^(0)$), $z^2 = q^1$
(matching $(-1)^2 q^1$), $z^(-1) = -q^1$ (matching
$(-1)^(-1) q^((-1)(-2)\/2) = -q$), and so on.
Each coefficient confirms $(-1)^n q^(n(n-1)\/2)$.
