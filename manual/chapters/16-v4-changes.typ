// 16-v4-changes.typ -- What's New in v4.0
#import "../template.typ": *

= What's New in v4.0
#index[v4.0]

q-Kangaroo v4.0 achieves full parity with Garvan's _A q-Product Tutorial
for a q-Series Maple Package_ (1998). This chapter documents all language
features, bug fixes, and new functions introduced in v4.0, followed by a
walkthrough that reproduces the tutorial examples from the paper
(hereafter _qmaple.pdf_).

== Language Features

=== Ditto Operator (")
#index[ditto operator]

The double-quote character `"` references the last printed result,
equivalent to `%`. This matches Maple's convention where `"` recalls
the previous output.

#repl-block("q> aqprod(q, q, 5)
-q^15 + q^14 + q^13 - q^10 - q^9 - q^8 + q^7 + q^6 + q^5 - q^2 - q + 1
q> qfactor(\", q)
(1-q)(1-q^2)(1-q^3)(1-q^4)(1-q^5)")

The ditto operator is especially useful in the walkthrough style of
_qmaple.pdf_ where one frequently pipes the result of one computation
into the next.

=== Arrow Operator (->)
#index[arrow operator]
#index[lambda]

The arrow operator `var -> expr` creates an anonymous function, equivalent
to wrapping the body in a `proc`. This is convenient for defining
short helper functions.

_cf.~qmaple.pdf, Section~4.3 (p.~15)._

#repl-block("q> F := q -> theta3(q, 500)/theta3(q^5, 100):
q> series(F(q), q, 20)
-4*q^19 + 2*q^16 + 2*q^14 - 4*q^11 + 2*q^9 + 2*q^6 + 2*q^4 - 4*q - 2 + 2*q + 1 + O(q^20)")

The arrow desugars to `proc(var) expr; end`, so `F := q -> theta3(q, 500)/theta3(q^5, 100)` is exactly equivalent to writing:

#repl-block("q> F := proc(q) theta3(q, 500)/theta3(q^5, 100); end:")

=== Fractional q-Powers
#index[fractional powers]
#index[FractionalPowerSeries]

Expressions involving fractional exponents of $q$ now produce a
`FractionalPowerSeries` with rational exponents. This arises
naturally with $theta_2(q)$ which involves $q^((n+1)^2\/2)$.

_cf.~qmaple.pdf, Section~3.3 (p.~8)._

#repl("theta2(q, 100)/q^(1/4)",
  "2*q^(323/4) + 2*q^(195/4) + 2*q^(99/4) + 2*q^(35/4) + 2*q^(3/4) + O(q^(399/4))")

When all exponents share a common denominator, the result is displayed
with fractional powers. If the exponents simplify to integers, the
result automatically converts back to a regular Series.

=== Procedure Option/Local Reorder
#index[procedure options]

In procedure definitions, the `option remember` and `local` declarations
can now appear in either order. Both of the following are valid:

#repl-block("q> f := proc(n) option remember; local k; k := n^2; k; end:
q> g := proc(n) local k; option remember; k := n^2; k; end:")

This matches Maple's flexibility in declaration ordering.

== Bug Fixes

=== aqprod 3-Argument Form
#index-main[aqprod]

The call `aqprod(a, q, n)` with exactly three arguments now returns the
full exact polynomial $(a; q)_n$ without truncation. Previously this
form could truncate to the default precision.

#repl("aqprod(q, q, 5)",
  "-q^15 + q^14 + q^13 - q^10 - q^9 - q^8 + q^7 + q^6 + q^5 - q^2 - q + 1")

The result is exact (no $O(q^T)$ term), containing all $5 dot (5+1) / 2 = 15$
terms of the product $(q;q)_5 = (1-q)(1-q^2)(1-q^3)(1-q^4)(1-q^5)$.

=== Theta 2-Argument Form
#index[theta functions]

The calls `theta2(q, T)`, `theta3(q, T)`, and `theta4(q, T)` are now
equivalent to `theta2(q, q, T)`, etc. The two-argument shorthand avoids
repeating the variable when $z = q$.

_cf.~qmaple.pdf, Section~3.3 (p.~8--9)._

#repl("theta2(q, 20)", "2*q^9 + 2*q + O(q^20)")

#repl("theta3(q, 20)", "2*q^16 + 2*q^9 + 2*q^4 + 2*q + 1 + O(q^20)")

#repl("theta4(q, 20)", "2*q^16 - 2*q^9 + 2*q^4 - 2*q + 1 + O(q^20)")

=== qfactor 2-Argument Form
#index-main[qfactor]

The call `qfactor(f, q)` is now equivalent to `qfactor(f, q, T)` with
an automatically determined bound. This provides a convenient shorthand
when the default bound suffices.

#repl("qfactor(aqprod(q, q, 5), q)",
  "(1-q)(1-q^2)(1-q^3)(1-q^4)(1-q^5)")

=== qfactor Product-Form Display
#index[QProduct]

The `qfactor` function now displays results in readable product notation
`(1-q)(1-q^2)...` instead of a raw internal structure. The new
`QProduct` value type carries the factored form.

#repl("qfactor(aqprod(q, q, 5), q)",
  "(1-q)(1-q^2)(1-q^3)(1-q^4)(1-q^5)")

=== etamake Eta-Notation Display
#index-main[etamake]
#index[EtaQuotient]

The `etamake` function now displays results using the familiar
$eta(k tau)$ notation instead of a raw dictionary. The new `EtaQuotient`
value type renders eta quotients in mathematical form.

_cf.~qmaple.pdf, Section~3.3 (p.~8--9)._

#repl("etamake(theta3(q, 100), q, 100)",
  "eta(tau)^(-2) * eta(2*tau)^(5) * eta(4*tau)^(-2)")

#repl("etamake(theta4(q, 100), q, 100)",
  "eta(tau)^(2) * eta(2*tau)^(-1)")

These match the well-known identities
$theta_3(q) = eta(2 tau)^5 \/ (eta(tau)^2 eta(4 tau)^2)$ and
$theta_4(q) = eta(tau)^2 \/ eta(2 tau)$.

== New Functions

#func-entry(
  name: "jac2series",
  signature: "jac2series(JP, T) or jac2series(JP, q, T)",
  description: [
    Convert a Jacobi product expression into a $q$-series expansion
    truncated to order $T$. The two-argument form `jac2series(JP, T)`
    infers the variable automatically. Each `JAC(a,b)` factor is
    expanded as the corresponding theta-series.
    #index[jac2series]
  ],
  params: (
    ([JP], [JacobiProduct], [A product/quotient of JAC terms]),
    ([T], [Integer], [Truncation order]),
    ([q], [Variable], [Series variable (optional in 2-argument form)]),
  ),
  examples: (
    ("jac2series(JAC(0,5)/JAC(1,5), 20)",
     "26*q^19 + 23*q^18 + ... + q^2 + q + 1 + O(q^20)"),
  ),
  related: ("JAC", "jac2prod", "jacprodmake"),
)

#func-entry(
  name: "radsimp",
  signature: "radsimp(expr)",
  description: [
    Simplify a rational expression involving series quotients. Currently
    acts as the identity function, returning its input unchanged. Provided
    for Maple compatibility where `radsimp()` is used after divisions.
    #index[radsimp]
  ],
  params: (
    ([expr], [Any], [Expression to simplify]),
  ),
  examples: (
    ("radsimp(theta3(q, 20))",
     "2*q^16 + 2*q^9 + 2*q^4 + 2*q + 1 + O(q^20)"),
  ),
  related: ("series", "expand"),
)

=== quinprod Identity Modes
#index[quinprod identity modes]
#index[prodid]
#index[seriesid]

The `quinprod` function now accepts the symbolic arguments `prodid` and
`seriesid` as the third parameter to display the quintuple product
identity in product form or series form respectively.

_cf.~qmaple.pdf, Section~6.2 (p.~21--22)._

#repl-block("q> quinprod(z, q, prodid)
(-z,q)_inf * (-q/z,q)_inf * (z^2*q,q^2)_inf * (q^2/z^2,q^2)_inf * (q,q)_inf
(-z,q)_inf * (-q/z,q)_inf * (z^2*q,q^2)_inf * (q^2/z^2,q^2)_inf * (q,q)_inf")

#repl-block("q> quinprod(z, q, seriesid)
(-z,q)_inf * (-q/z,q)_inf * (z^2*q,q^2)_inf * (q^2/z^2,q^2)_inf * (q,q)_inf
  = sum(m=-inf..inf, (z^(3*m) - z^(-3*m-1)) * q^(m*(3*m+1)/2))")

With a positive integer $T$, `quinprod` computes the bivariate expansion:

#repl-block("q> quinprod(z, q, 3)
q^2*z^3 - q*z^2 + 1 + O(q^3) - z^(-1) + q*z^(-3) - q^2*z^(-4) + O(q^3)")

== Walkthrough: qmaple.pdf Tutorial
#index[qmaple.pdf tutorial]
#index[Garvan tutorial]

This section reproduces the executable examples from Garvan's
_A q-Product Tutorial for a q-Series Maple Package_ (1998) in the same
order as the paper. Commands are shown in REPL transcript style with
`q>` prompts and actual output from q-Kangaroo.

=== Section 3: Product Conversion (p.~5--11)

==== Section 3.1: prodmake (p.~5--6)
#index[prodmake]

We compute the left side of the first Rogers-Ramanujan identity
$sum_(n >= 0) q^(n^2) \/ (q;q)_n$ and identify its infinite product form.

#repl-block("q> x := 1:
q> for n from 1 to 8 do
q>   x := x + q^(n*n)/aqprod(q,q,n):
q> od:
q> x := series(x, q, 50):
q> prodmake(x, q, 40)
{1: -1, 4: -1, 6: -1, 9: -1, 11: -1, 14: -1, 16: -1, 19: -1, 21: -1, 24: -1, 26: -1, 29: -1, 31: -1, 34: -1, 36: -1, 39: -1}")

The product exponents $-1$ at positions $1, 4, 6, 9, 11, 14, dots$
(i.e., $equiv 1$ or $4 mod 5$) confirm that
$sum_(n >= 0) q^(n^2) \/ (q;q)_n = 1 \/ ((q;q^5)_oo (q^4;q^5)_oo)$.

==== Section 3.2: qfactor (p.~6--8)
#index[qfactor]
#index[T(r,j)]

Following Andrews, we define the recursive function $T(r, N)$ and compute
$T(8,8)$. We then compare `factor` (cyclotomic) with `qfactor`
($q$-product) factorization.

#repl-block("q> T := proc(r,j)
q>   option remember;
q>   local x,k;
q>   x := 0;
q>   if j = 0 or j = 1 then
q>     RETURN((j-1)^2):
q>   else
q>     for k from 1 to floor(j/2) do
q>       x := x - qbin(q, k, r+2*k) * T(r+2*k, j-2*k);
q>     od:
q>     RETURN(expand(x));
q>   fi:
q> end:
q> t8 := T(8, 8);
q^42 + q^41 + 2*q^40 + ... + 2*q^8 + q^7 + q^6")

#repl-block("q> factor(t8)
(q^4-q^3+q^2-q+1)(q^4+1)(q^4+q^3+q^2+q+1)(q^6)(q^6+q^3+1)(q^8+1)(q^10+q^9+q^8+q^7+q^6+q^5+q^4+q^3+q^2+q+1)")

We define the Dixon sum _cf.~qmaple.pdf, Section~3.2 (p.~7--8)_:

#repl-block("q> dixson := proc(a,b,c,q)
q>   local x,k,y;
q>   x := 0:
q>   y := min(a,b,c):
q>   for k from -y to y do
q>     x := x + (-1)^k * q^(k*(3*k+1)/2) *
q>       qbin(q, c+k, b+c) * qbin(q, a+k, c+a) * qbin(q, b+k, a+b);
q>   od:
q>   RETURN(x):
q> end:
q> dx := expand(dixson(5,5,5,q)):")

==== Section 3.3: etamake (p.~8--9)
#index[etamake]

We write the theta functions as eta products. This demonstrates both the
theta 2-argument form and etamake's new eta-notation display.

_cf.~qmaple.pdf, Section~3.3 (p.~8--9)._

#repl-block("q> theta2(q, 20)
2*q^9 + 2*q + O(q^20)
q> etamake(theta2(q, 100), q, 100)
q * eta(8*tau)^(-1) * eta(16*tau)^(2)")

#repl-block("q> theta3(q, 20)
2*q^16 + 2*q^9 + 2*q^4 + 2*q + 1 + O(q^20)
q> etamake(theta3(q, 100), q, 100)
eta(tau)^(-2) * eta(2*tau)^(5) * eta(4*tau)^(-2)")

#repl-block("q> theta4(q, 20)
2*q^16 - 2*q^9 + 2*q^4 - 2*q + 1 + O(q^20)
q> etamake(theta4(q, 100), q, 100)
eta(tau)^(2) * eta(2*tau)^(-1)")

These confirm the classical identities:
$ theta_2(q) &= q dot eta(8 tau)^(-1) dot eta(16 tau)^2 \
  theta_3(q) &= eta(2 tau)^5 / (eta(tau)^2 dot eta(4 tau)^2) \
  theta_4(q) &= eta(tau)^2 / eta(2 tau) $

==== Section 3.4: jacprodmake (p.~10--11)
#index[jacprodmake]
#index[jac2series]

We re-examine the Rogers-Ramanujan identity using `jacprodmake` and the
new `jac2series` 2-argument form.

_cf.~qmaple.pdf, Section~3.4 (p.~10--11)._

#repl-block("q> y := jacprodmake(x, q, 40)
JAC(0,5) / JAC(1,5)
q> jac2series(y, 20)
26*q^19 + 23*q^18 + 19*q^17 + ... + q^2 + q + 1 + O(q^20)")

The Jacobi product $"JAC"(0,5) / "JAC"(1,5)$ represents
$(q^5;q^5)_oo \/ ((q;q^5)_oo dot.op (q^4;q^5)_oo)$, confirming the first
Rogers-Ramanujan identity.

=== Section 4: Relations (p.~12--19)

==== Section 4.1: findhom (p.~12--13)
#index-main[findhom]

We search for homogeneous relations among $theta_3(q)$, $theta_4(q)$,
$theta_3(q^2)$, and $theta_4(q^2)$.

_cf.~qmaple.pdf, Section~4.1 (p.~12--13)._

#repl-block("q> findhom([theta3(q,100), theta4(q,100), theta3(q^2,100), theta4(q^2,100)], q, 2, 0)
-X[4]^2 + X[1]*X[2]
-2*X[3]^2 + X[2]^2 + X[1]^2
[-X[4]^2 + X[1]*X[2], -2*X[3]^2 + X[2]^2 + X[1]^2]")

The first relation $X_1 X_2 = X_4^2$ says
$theta_3(q) theta_4(q) = theta_4(q^2)^2$. The second says
$theta_3(q)^2 + theta_4(q)^2 = 2 theta_3(q^2)^2$, which is Gauss's
parametrization of the arithmetic-geometric mean iteration.

==== Section 4.2: findhomcombo (p.~13--14)
#index-main[findhomcombo]
#index[Eisenstein series]

We define the Eisenstein series $U_(p,k)$ and use `findhomcombo` to
express $U_(5,6)$ as a polynomial in eta products.

_cf.~qmaple.pdf, Section~4.2 (p.~13--14)._

#repl-block("q> UE := proc(q,k,p,trunk)
q>   local x,m,n;
q>   x := 0:
q>   for m from 1 to trunk do
q>     for n from 1 to floor(trunk/m) do
q>       x := x + L(m,p)*n^(k-1)*q^(m*n):
q>     od:
q>   od:
q> end:
q> f := UE(q, 6, 5, 50):
q> B1 := etaq(q,1,50)^5/etaq(q,5,50):
q> B2 := q*etaq(q,5,50)^5/etaq(q,1,50):
q> findhomcombo(f, [B1, B2], q, 3, 0)
335*X[2]^3 + 40*X[1]*X[2]^2 + X[1]^2*X[2]")

This confirms
$U_(5,6) = eta(5 tau)^3 eta(tau)^9 + 40 dot eta(5 tau)^9 eta(tau)^3 + 335 dot eta(5 tau)^(15) / eta(tau)^3$.

==== Section 4.3: findnonhom (p.~15--16)
#index-main[findnonhom]

The `findnonhom` function searches for nonhomogeneous polynomial
relations between $q$-series. This example uses the arrow operator to
define $F(q) = theta_3(q) / theta_3(q^5)$.

_cf.~qmaple.pdf, Section~4.3 (p.~15--16)._

#repl-block("q> F := q -> theta3(q, 500)/theta3(q^5, 100):")

==== Section 4.4: findnonhomcombo (p.~16--17)
#index-main[findnonhomcombo]

The `findnonhomcombo` function expresses a target $q$-series as a
nonhomogeneous polynomial in a list of basis series. The
_qmaple.pdf_ example defines eta quotients $xi$ and $T$ related to
prime $p = 7$.

_cf.~qmaple.pdf, Section~4.4 (p.~16--17)._

#repl-block("q> xi := q^2*etaq(q,49,100)/etaq(q,1,100):
q> T := q*(etaq(q,7,100)/etaq(q,1,100))^4:
q> findnonhomcombo(T^2, [T, xi], q, 7, -15)
X[1]^2")

==== Section 4.5: findpoly (p.~17--18)
#index-main[findpoly]

The `findpoly` function finds a polynomial relation between two given
$q$-series with specified degrees. The _qmaple.pdf_ example involves
theta functions and the cubic modular identity, using `radsimp` for
intermediate simplification.

_cf.~qmaple.pdf, Section~4.5 (p.~17--18)._

#repl("radsimp(theta3(q, 20))",
  "2*q^16 + 2*q^9 + 2*q^4 + 2*q + 1 + O(q^20)")

The `radsimp` function is provided for compatibility with _qmaple.pdf_
examples; it currently acts as the identity function since q-Kangaroo's
series arithmetic already produces simplified results.

=== Section 5: Sifting Coefficients (p.~19--20)
#index[sifting]
#index[sift]
#index[Ramanujan congruence]

We verify Ramanujan's congruence $p(5n+4) equiv 0 (mod 5)$ by sifting
the partition generating function.

_cf.~qmaple.pdf, Section~5 (p.~19--20)._

#repl-block("q> f := partition_gf(100):
q> g := sift(f, q, 5, 4, 99)
92669720*q^18 + 49995925*q^17 + ... + 135*q^2 + 30*q + 5 + O(q^19)")

Every coefficient is divisible by 5, confirming $p(5n+4) equiv 0 (mod 5)$.

#repl("etamake(g, q, 18)",
  "q^(19/24) * eta(tau)^(-6) * eta(5*tau)^(5)")

This identifies the generating function
$sum p(5n+4) q^n = 5 dot eta(5 tau)^5 / eta(tau)^6$,
confirming Ramanujan's identity.

For the distinct partition analog:

_cf.~qmaple.pdf, Section~5 (p.~19--20)._

#repl-block("q> PD := series(etaq(q,2,200)/etaq(q,1,200), q, 200):
q> PD1 := sift(PD, q, 5, 1, 199):
q> etamake(PD1, q, 38)
q^(5/24) * eta(tau)^(-4) * eta(2*tau)^(2) * eta(5*tau)^(3) * eta(10*tau)^(-1)")

=== Section 6: Product Identities (p.~20--25)

==== Section 6.1: The Triple Product Identity (p.~20--21)
#index[triple product identity]
#index[Jacobi triple product]

The Jacobi triple product identity states:
$ product_(k >= 1) (1 - q^k)(1 - z q^(k-1))(1 - z^(-1) q^k)
  = sum_(n = -oo)^oo (-1)^n z^n q^(n(n-1)\/2). $

_cf.~qmaple.pdf, Section~6.1 (p.~20--21)._

#repl("tripleprod(z, q, 10)",
  "q^6*z^4 - q^3*z^3 + q*z^2 - z + 1 + O(q^10) - q*z^(-1) + q^3*z^(-2) - q^6*z^(-3) + O(q^10)")

Reading off the coefficients confirms $(-1)^n q^(n(n-1)\/2)$ for each
power of $z$.

==== Section 6.2: The Quintuple Product Identity (p.~21--23)
#index[quintuple product identity]

The quintuple product identity is displayed in both product and series
form using the `prodid` and `seriesid` modes.

_cf.~qmaple.pdf, Section~6.2 (p.~21--23)._

#repl-block("q> quinprod(z, q, prodid)
(-z,q)_inf * (-q/z,q)_inf * (z^2*q,q^2)_inf * (q^2/z^2,q^2)_inf * (q,q)_inf")

#repl-block("q> quinprod(z, q, seriesid)
(-z,q)_inf * (-q/z,q)_inf * (z^2*q,q^2)_inf * (q^2/z^2,q^2)_inf * (q,q)_inf
  = sum(m=-inf..inf, (z^(3*m) - z^(-3*m-1)) * q^(m*(3*m+1)/2))")

Computing the bivariate expansion to low order:

#repl("quinprod(z, q, 3)",
  "q^2*z^3 - q*z^2 + 1 + O(q^3) - z^(-1) + q*z^(-3) - q^2*z^(-4) + O(q^3)")

As an application, we split Euler's infinite product by residue modulo 5.
By the quintuple product identity with $q arrow.r q^5$ and $z arrow.r q$,
the residue-0 component $E_0$ can be identified as a Jacobi product.

_cf.~qmaple.pdf, Section~6.2 (p.~22--23)._

#repl-block("q> EULER := etaq(q, 1, 500):
q> E0 := sift(EULER, q, 5, 0, 499):
q> jacprodmake(E0, q, 50)
JAC(2,5) * JAC(0,5) / JAC(1,5)")

==== Section 6.3: Winquist's Identity (p.~23--25)
#index[Winquist's identity]

Winquist's identity involves two symbolic variables $a$ and $b$,
producing a trivariate result (Laurent polynomial in $a$, $b$ with
$q$-series coefficients).

_cf.~qmaple.pdf, Section~6.3 (p.~23--25)._

#repl-block("q> series(winquist(a, b, q, 10), q, 10)
... (trivariate Laurent polynomial in a, b with q-coefficients)")

The _qmaple.pdf_ example constructs $Q(k) = product_(n >= 1) (1-q^k)(1-q^(33-k))(1-q^33)$ for $1 < k < 33$ and verifies Winquist's identity by showing that a particular combination $A_0 B_2 - q^2 A_9 B_4$ equals the `winquist` output.

=== Not Yet Supported
#index[unsupported features]

The following examples from _qmaple.pdf_ require features not yet
available in q-Kangaroo:

- *Exercise 4 (p.~9):* `omega := RootOf(z^2 + z + 1 = 0)` -- requires
  algebraic number fields. The function $b(q)$ involving a primitive
  cube root of unity cannot be evaluated.

- *`with(qseries)` / `with(numtheory)`:* Maple session commands for
  loading packages. Not needed in q-Kangaroo where all functions are
  available by default (no-op).

- *`while` loops:* Now supported in v5.0. See the _What's New in v5.0_
  chapter.
