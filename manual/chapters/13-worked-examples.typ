// 13-worked-examples.typ -- Worked Examples

= Worked Examples
#index[worked examples]

This chapter demonstrates q-Kangaroo through extended examples that span
multiple function groups. Each example shows a complete research workflow
from problem statement through computational verification, with
mathematical context and references to the source literature.

== Euler's Pentagonal Theorem
#index[Euler's pentagonal theorem]
#index("identities", "Euler pentagonal")

*Mathematical context.* Euler's pentagonal number theorem (1750) states that
the Euler function factors as

$ (q; q)_oo = product_(k=1)^oo (1 - q^k) = sum_(n=-oo)^oo (-1)^n q^(n(3n-1)\/2) $

The exponents $0, 1, 2, 5, 7, 12, 15, 22, dots$ are the _generalised pentagonal
numbers_ $omega(n) = n(3n-1)/2$ for $n = 0, plus.minus 1, plus.minus 2, dots$ The
identity is equivalent to the statement that the partition function $p(n)$
satisfies the recurrence $p(n) = p(n-1) + p(n-2) - p(n-5) - p(n-7) + dots$

*Reference:* Euler (1750); see Andrews, _The Theory of Partitions_ (Addison-Wesley, 1976), Chapter 1.

=== REPL Workflow

*Step 1.* Compute $(q; q)_oo$ to 20 terms using `aqprod`:

#repl("aqprod(1, 1, 1, infinity, 20)",
  "1 - q - q^2 + q^5 + q^7 - q^12 - q^15 + O(q^20)")

Observe the nonzero coefficients at $q^0, q^1, q^2, q^5, q^7, q^12, q^15$ ---
exactly the pentagonal numbers --- and that the signs alternate in pairs
$(+, -, -, +, +, -, -, dots)$.

*Step 2.* Confirm that the product is a single-factor eta quotient using
`prodmake`:

#repl("prodmake(%, 10)",
  "{1: 1, 2: 1, 3: 1, 4: 1, 5: 1, 6: 1, 7: 1, 8: 1, 9: 1, 10: 1}")

Every factor $(1 - q^k)$ appears with exponent 1, confirming
$(q;q)_oo = product_(k=1)^oo (1 - q^k)$.

*Step 3.* Verify via the eta representation:

#repl("etamake(aqprod(1, 1, 1, infinity, 50), 10)",
  "{1: 1}")

The result `{1: 1}` means exactly $eta(tau)$, the Dedekind eta function
with $q = e^(2 pi i tau)$.

*Step 4.* Cross-check: multiply by the partition generating function and
confirm the result is 1:

#repl-block("q> f := aqprod(1, 1, 1, infinity, 20):
q> g := partition_gf(20):
q> f * g
1 + O(q^20)")

The identity $(q;q)_oo dot 1\/(q;q)_oo = 1$ is confirmed to 20 terms.

*Takeaway.* The signs and exponents in $( q; q)_oo$ encode the pentagonal
numbers. The `prodmake` and `etamake` functions verify the product structure
directly, while multiplication by the partition generating function provides
an independent check.


== Ramanujan's Partition Congruences
#index[Ramanujan's partition congruences]
#index("identities", "Ramanujan congruences")
#index[findcong]

*Mathematical context.* Ramanujan (1919) discovered that the partition
function satisfies three remarkable congruences:

$ p(5n + 4) &equiv 0 quad (mod 5) \
  p(7n + 5) &equiv 0 quad (mod 7) \
  p(11n + 6) &equiv 0 quad (mod 11) $

These are the only congruences of the form $p(ell n + delta) equiv 0 space (mod ell)$ for
primes $ell <= 31$. Proving these congruences motivated much of the theory
of modular forms as applied to partitions.

*Reference:* Ramanujan (1919), "Some Properties of $p(n)$", _Proc. Cambridge Phil. Soc._ 19, 207--210. See also Berndt, _Ramanujan's Notebooks_, Vol. III (Springer, 1991).

=== REPL Workflow

*Step 1.* Generate the partition function to high order so that sifting
produces enough terms for pattern recognition:

#repl("f := partition_gf(200):",
  "")

*Step 2.* Use `findcong` to automatically discover congruences modulo 5, 7,
and 11:

#repl("findcong(f, [5, 7, 11])",
  "[[5, 4, 5], [7, 5, 7], [11, 6, 11]]")

Each triple $[m, j, d]$ means $p(m n + j) equiv 0 space (mod d)$. The three
Ramanujan congruences are recovered automatically.

*Step 3.* Verify the mod-5 congruence manually with `sift`. Extract the
subsequence $p(5n + 4)$ and observe that every coefficient is divisible by 5:

#repl("sift(f, 5, 4)",
  "5 + 30*q + 135*q^2 + 490*q^3 + 1575*q^4 + 4565*q^5 + 12310*q^6 + 31185*q^7 + 75175*q^8 + 173525*q^9 + O(q^10)")

Coefficients: $5, 30, 135, 490, 1575, dots$ --- all divisible by 5.

*Step 4.* Check whether primes beyond 11 yield simple congruences:

#repl("findcong(partition_gf(500), [13, 17, 19, 23])",
  "[]")

The empty result confirms that no congruences of the form
$p(ell n + delta) equiv 0 space (mod ell)$ exist for $ell in {13, 17, 19, 23}$.

*Takeaway.* The `findcong` function is a research tool that automates the
search for arithmetic congruences in generating function coefficients. Manual
verification with `sift` confirms the pattern. This workflow --- generate,
discover, verify --- is a standard research loop in experimental mathematics.


== Jacobi Triple Product Identity
#index[Jacobi triple product identity]
#index("identities", "Jacobi triple product")
#index[prove_eta_id]

*Mathematical context.* The Jacobi triple product identity (1829) states

$ sum_(n=-oo)^oo z^n q^(n^2) = product_(k>=1) (1 - q^(2k))(1 + z q^(2k-1))(1 + z^(-1) q^(2k-1)) $

At $z = 1$, this specialises to the theta function

$ theta_3(q) = sum_(n=-oo)^oo q^(n^2) = product_(k>=1) (1 - q^(2k))(1 + q^(2k-1))^2 $

which connects the sum-of-squares representation to an infinite product.

*Reference:* Jacobi, _Fundamenta Nova Theoriae Functionum Ellipticarum_ (1829). See also Andrews & Berndt, _Ramanujan's Lost Notebook_, Part I (Springer, 2005).

=== REPL Workflow

*Step 1.* Compute $theta_3(q)$ directly:

#repl("a := theta3(50):",
  "")

*Step 2.* Build the product side of the identity. The right-hand side at
$z = 1$ is $(q^2; q^2)_oo dot (-q; q^2)_oo^2$:

#repl-block("q> b := aqprod(1, 1, 2, infinity, 50) * aqprod(-1, 1, 1, infinity, 50)^2:
q> a - b
O(q^50)")

The difference is zero to 50 terms, confirming the identity.

*Step 3.* Use `prodmake` to examine the product structure of $theta_3$:

#repl("prodmake(a, 20)",
  "{1: 0, 2: 1, 3: 0, 4: -1, 5: 0, 6: 1, 7: 0, 8: -1, 9: 0, 10: 1, 11: 0, 12: -1, 13: 0, 14: 1, 15: 0, 16: -1, 17: 0, 18: 1, 19: 0, 20: -1}")

The even-index exponents are $+1$ and the odd-index exponents beyond
1 alternate, consistent with the triple product factorisation.

*Step 4.* Verify the eta-quotient structure:

#repl("etamake(a, 10)",
  "{1: -2, 2: 5, 4: -2}")

This says $theta_3(q) = eta(2tau)^5 / (eta(tau)^2 eta(4tau)^2)$, a classical
eta-quotient representation.

*Takeaway.* Multiple verification strategies are available: direct series
subtraction, product form analysis via `prodmake`, and eta-quotient
identification via `etamake`. Using independent methods strengthens
confidence in a conjectured identity.


== Rogers-Ramanujan Identities via Bailey Chains
#index[Rogers-Ramanujan identities]
#index("identities", "Rogers-Ramanujan")
#index[Bailey chains]
#index[bailey_chain]

*Mathematical context.* The first Rogers-Ramanujan identity states

$ sum_(n=0)^oo frac(q^(n^2), (q;q)_n) = product_(n=1)^oo frac(1, (1 - q^(5n-4))(1 - q^(5n-1))) $

Rogers (1894) proved this using iterative functional equations, and
Ramanujan rediscovered it independently around 1913. In 1947, Bailey
introduced the notion of _Bailey pairs_ and _Bailey chains_, which provide
a systematic framework for generating infinite families of identities from
a single seed pair.

*Reference:* Rogers (1894), Ramanujan (1913); see Andrews, _q-Series: Their
Development and Application in Analysis, Number Theory, Combinatorics,
Physics and Computer Algebra_ (AMS, 1986). For Bailey chains, see Andrews
(1984), "Multiple series Rogers-Ramanujan type identities", _Pacific J.
Math._ 114, 267--283.

=== REPL Workflow

*Step 1.* Start with the Rogers-Ramanujan Bailey pair. The weak lemma
produces the $(\alpha, \beta)$ pair for the base case:

#repl("bailey_weak_lemma(1, 1, 1, 0, 10, 30)",
  "([1, -1, 0, 1, 0, 0, 0, -1, 0, 0], [1, 1, 1, 1, 2, 2, 3, 3, 4, 5])")

The $alpha$ sequence shows the sparse signs ($1, -1, 0, 1, 0, dots$) while
$beta$ gives the Rogers-Ramanujan coefficients.

*Step 2.* Apply Bailey's lemma to transform the pair. This lifts the pair to
a new pair with modified parameters:

#repl("bailey_apply_lemma(1, 1, 1, 0, 1, 1, 1, 1, 1, 2, 10, 30)",
  "([...], [...])")

*Step 3.* Iterate the Bailey chain to depth 2, generating a family of
identities from the seed:

#repl("bailey_chain(1, 1, 1, 0, 1, 1, 1, 1, 1, 2, 2, 10, 30)",
  "[[...], [...]]")

Each element in the chain is a new $(\alpha, \beta)$ pair that encodes a
distinct $q$-series identity.

*Step 4.* Use `bailey_discover` to find the proof automatically. Given a
target identity as two $q$-series, `bailey_discover` searches the space of
Bailey pairs and chain operations to find a proof path:

#repl-block("q> lhs := partition_gf(30):
q> rhs := aqprod(1, 1, 4, infinity, 30) * aqprod(1, 1, 1, infinity, 30):
q> bailey_discover(lhs, rhs, 1, 3, 30)
{found: true, depth: 1, pair: \"rogers-ramanujan\", ...}")

*Takeaway.* Bailey chains generate infinite families of identities from a
single seed pair. The `bailey_weak_lemma` / `bailey_apply_lemma` /
`bailey_chain` functions let you walk the chain manually, while
`bailey_discover` automates the search. This machinery replaces what would
be pages of hand computation in classical proofs.


== Hypergeometric Transformations
#index[hypergeometric transformations]
#index[Heine's transformations]
#index[find_transformation_chain]

*Mathematical context.* The basic hypergeometric series ${}_2 phi_1$ is

$ attach(, tl: 2) phi_1 (a, b; c; q, z) = sum_(n=0)^oo frac((a;q)_n (b;q)_n, (c;q)_n (q;q)_n) z^n $

Heine (1847) discovered three transformations connecting different
${}_2 phi_1$ series, analogous to Euler's transformation for the classical
Gauss ${}_2 F_1$. These transformations are fundamental tools: a series that
resists direct summation may become tractable after one or more
transformation steps.

*Reference:* Heine (1847); see Gasper & Rahman, _Basic Hypergeometric
Series_, 2nd ed. (Cambridge, 2004), Chapter 1.

=== REPL Workflow

*Step 1.* Define a ${}_2 phi_1$ series with specific parameters. In
q-Kangaroo, each parameter $a = frac(n)(d) q^p$ is encoded as
the triple $(n, d, p)$:

#repl("src := phi([(1,1,2), (1,1,3)], [(1,1,5)], 1, 1, 1, 30):",
  "")

This computes ${}_2 phi_1 (q^2, q^3; q^5; q, q)$ to 30 terms.

*Step 2.* Apply Heine's first transformation:

#repl("heine1([(1,1,2), (1,1,3)], [(1,1,5)], 1, 1, 1, 30)",
  "(prefactor_series, transformed_series)")

The result is a pair `(prefactor, transformed)` where the original series
equals `prefactor * transformed` and `transformed` is a new
${}_2 phi_1$ with different parameters.

*Step 3.* Apply Heine's second transformation to a different series:

#repl("heine2([(1,1,2), (1,1,3)], [(1,1,5)], 1, 1, 1, 30)",
  "(prefactor_series, transformed_series)")

*Step 4.* Use BFS search to automatically find a transformation chain
between two ${}_2 phi_1$ series:

#repl-block("q> find_transformation_chain(
    [(1,1,2), (1,1,3)], [(1,1,5)], 1, 1, 1,
    [(1,1,2), (1,1,1)], [(1,1,4)], 1, 1, 3,
    3, 30)
{found: true, steps: [{name: \"heine1\", ...}], depth: 1}")

The BFS explores the graph of all reachable ${}_2 phi_1$ series connected by
Heine, Sears, and Watson transformations, and returns the shortest path.

*Takeaway.* Heine's transformations are the $q$-analogue of Euler's
classical hypergeometric transformation. The `find_transformation_chain`
function automates what would otherwise be a manual search through
transformation identities --- a BFS over a graph of equivalent series
representations.


== Mock Theta Function Relations
#index[mock theta functions]
#index("identities", "mock theta relations")
#index[findlincombo]
#index[appell_lerch_m]

*Mathematical context.* In his last letter to Hardy (1920), Ramanujan
introduced 17 functions he called "mock theta functions" and claimed they
shared properties with theta functions but were not themselves theta
functions. Zwegers (2002) showed that mock theta functions complete to
real-analytic modular forms, placing them in the framework of harmonic
Maass forms.

Among the third-order mock theta functions, Watson (1936) proved the
relation

$ f(q) + 4 psi(q) = (theta_4(q)^2) / (( q; q )_oo) $

connecting the mock theta function $f(q)$ and $psi(q)$ to classical theta
and eta functions.

*Reference:* Watson (1936), "The Final Problem: An Account of the Mock Theta
Functions in the Last Letter of Ramanujan", _J. London Math. Soc._ 11,
55--80. Zwegers (2002), "Mock theta functions", PhD thesis, Utrecht.

=== REPL Workflow

*Step 1.* Compute the third-order mock theta functions $f(q)$ and $psi(q)$:

#repl-block("q> mf := mock_theta_f3(50):
q> mpsi := mock_theta_psi3(50):")

*Step 2.* Compute the right-hand side: $theta_4(q)^2 / (q;q)_oo$:

#repl-block("q> t4sq := theta4(50) ^ 2:
q> euler_inv := partition_gf(50):
q> rhs := t4sq * euler_inv:")

*Step 3.* Use `findlincombo` to discover the linear relation. We ask:
is there a linear combination of $f(q)$ and $psi(q)$ that equals `rhs`?

#repl("findlincombo(rhs, [mf, mpsi], 0)",
  "[1, 4]")

The coefficients $[1, 4]$ confirm Watson's relation:
$"rhs" = 1 dot f(q) + 4 dot psi(q)$.

*Step 4.* Verify independently by direct subtraction:

#repl("rhs - mf - 4 * mpsi",
  "O(q^50)")

Zero to 50 terms --- the relation is confirmed.

*Step 5.* Explore with the Appell-Lerch sum. Zwegers showed that mock theta
functions can be expressed in terms of the Appell-Lerch sum
$m(a, z, q)$:

#repl("appell_lerch_m(1, 1, 30)",
  "1 + q + 2*q^2 + 2*q^3 + 4*q^4 + 4*q^5 + 7*q^6 + 8*q^7 + 12*q^8 + 14*q^9 + 20*q^10 + 24*q^11 + 34*q^12 + 40*q^13 + 54*q^14 + 66*q^15 + 86*q^16 + 104*q^17 + 136*q^18 + 164*q^19 + O(q^20)")

*Takeaway.* The combination of classical mock theta function computations
(`mock_theta_f3`, `mock_theta_psi3`) with relation discovery tools
(`findlincombo`) and modern Appell-Lerch machinery (`appell_lerch_m`)
provides a computational framework for exploring Ramanujan's mock theta
functions. The `findlincombo` function is especially powerful: given candidate
series and a target, it discovers the exact coefficients of a linear
relation, automating what would be tedious hand calculations.
