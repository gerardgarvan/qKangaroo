// 14-maple-migration.typ -- Maple Migration Guide
#import "../template.typ": *

= Maple Migration Guide
#index-main[Maple migration]
#index[Maple compatibility]
#index[Garvan packages]

q-Kangaroo v2.0 accepts Frank Garvan's exact Maple calling conventions for
all `qseries` and `thetaids` functions.  Researchers can copy commands
directly from Maple worksheets and run them at the `q>` prompt.  This guide
is organised by workflow --- look up what you want to do, and the table shows
side-by-side syntax.  Where Maple and q-Kangaroo syntax is identical (most
functions), the table confirms it.  Where differences remain, they are
highlighted.

== Overview
#index("Maple migration", "overview")

Most `qseries` functions now use *identical syntax* between Maple and
q-Kangaroo.  Key examples: `aqprod`, `etaq`, `sift`, `prodmake`,
`findlincombo`, `findcong` --- all accept Garvan's exact argument lists.

The remaining differences fall into three categories:

+ *Hypergeometric series* use integer-triple encoding `(num, den, pow)`
  instead of symbolic parameters.
+ *Some function names differ* from Maple, but aliases are accepted
  (`proveid`, `qzeil`, `qgosper`, etc.).
+ *Session conventions* match Maple: `:=` for assignment, `:` to suppress
  output.  No session object is needed at the REPL.


== Computing Eta Products
#index("Maple migration", "eta products")
#index[etaq]
#index[etamake]

Maple and q-Kangaroo use identical syntax for eta-product computation:

#table(
  columns: (1fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*],
  ),
  [`etaq(q, 1, 20)`], [`etaq(q, 1, 20)`],
  [`etaq(q, \[1,2,3\], 20)`], [`etaq(q, \[1,2,3\], 20)`],
  [`etamake(f, q, 10)`], [`etamake(f, q, 10)`],
  [`prodmake(f, q, 10)`], [`prodmake(f, q, 10)`],
)

Syntax is identical.  No translation needed.


== Analysing Series
#index("Maple migration", "series analysis")
#index[sift]
#index[prodmake]

All series-analysis functions accept Garvan's exact signatures:

#table(
  columns: (1fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*],
  ),
  [`sift(f, q, 5, 4, 50)`], [`sift(f, q, 5, 4, 50)`],
  [`prodmake(f, q, 20)`], [`prodmake(f, q, 20)`],
  [`mprodmake(f, q, 20)`], [`mprodmake(f, q, 20)`],
  [`qfactor(f, q)`], [`qfactor(f, q)`],
  [`checkmult(f, 30)`], [`checkmult(f, 30)`],
  [`checkprod(f, 10, 30)`], [`checkprod(f, 10, 30)`],
)

Syntax is identical.


== Finding Congruences
#index("Maple migration", "congruences")
#index[findcong]

The congruence-discovery function accepts Garvan's exact signatures
including optional modulus limits and exclusion sets:

#table(
  columns: (1fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*],
  ),
  [`findcong(QS, T)`], [`findcong(QS, T)`],
  [`findcong(QS, T, LM)`], [`findcong(QS, T, LM)`],
  [`findcong(QS, T, LM, XSET)`], [`findcong(QS, T, LM, XSET)`],
)

*Worked example --- discovering Ramanujan's congruences:*

#repl-block("q> p := partition_gf(200):\nq> findcong(p, 200)\n[4, 5, 5]\n[5, 7, 7]\n[6, 11, 11]")

Output format matches Garvan: `[B, A, R]` triples where
$p(A n + B) equiv 0 space (mod R)$.


== Discovering Relations
#index("Maple migration", "relations")
#index[findlincombo]
#index[findhomcombo]
#index[findprod]
#index[findpoly]

All relation-discovery functions accept Garvan's exact signatures including
symbolic labels:

#table(
  columns: (1fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*],
  ),
  [`findlincombo(f, L, SL, q, 0)`], [`findlincombo(f, L, SL, q, 0)`],
  [`findhomcombo(f, L, q, 2, 0)`], [`findhomcombo(f, L, q, 2, 0)`],
  [`findprod(FL, 2, 10, 30)`], [`findprod(FL, 2, 10, 30)`],
  [`findpoly(x, y, q, 2, 2)`], [`findpoly(x, y, q, 2, 2)`],
  [`findmaxind(L, 0)`], [`findmaxind(L, 0)`],
  [`findhom(L, q, 2, 0)`], [`findhom(L, q, 2, 0)`],
  [`findnonhom(L, q, 2, 0)`], [`findnonhom(L, q, 2, 0)`],
)

All relation functions accept Garvan's exact signatures including symbolic
labels.


== Theta Functions and Jacobi Products
#index("Maple migration", "theta functions")
#index("Maple migration", "Jacobi products")
#index[theta]
#index[JAC]
#index[jac2prod]
#index[jac2series]
#index[qs2jaccombo]

Theta and Jacobi product operations use identical syntax:

#table(
  columns: (1fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*],
  ),
  [`theta(z, q, T)`], [`theta(z, q, T)`],
  [`JAC(1,5) * JAC(4,5)`], [`JAC(1,5) * JAC(4,5)`],
  [`jac2prod(JP, q, T)`], [`jac2prod(JP, q, T)`],
  [`jac2series(JP, q, T)`], [`jac2series(JP, q, T)`],
  [`qs2jaccombo(f, q, T)`], [`qs2jaccombo(f, q, T)`],
)

All Jacobi product operations use identical syntax.


== Product Functions
#index("Maple migration", "product functions")
#index[aqprod]
#index[jacprod]
#index[tripleprod]
#index[quinprod]
#index[winquist]
#index[qbin]

All product-construction functions accept Garvan's calling conventions:

#table(
  columns: (1fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*],
  ),
  [`aqprod(q^2, q, 5)`], [`aqprod(q^2, q, 5)`],
  [`aqprod(q, q, infinity, 20)`], [`aqprod(q, q, infinity, 20)`],
  [`jacprod(1, 5, q, 30)`], [`jacprod(1, 5, q, 30)`],
  [`tripleprod(q^3, q, 20)`], [`tripleprod(q^3, q, 20)`],
  [`quinprod(q^2, q, 20)`], [`quinprod(q^2, q, 20)`],
  [`winquist(q, q^2, q, 10)`], [`winquist(q, q^2, q, 10)`],
  [`qbin(q, 2, 4)`], [`qbin(q, 2, 4)`],
)

Syntax is identical for all product functions.


== Remaining Differences
#index("Maple migration", "remaining differences")

While most syntax is now identical, a few areas still require translation:

=== Hypergeometric Series
#index("Maple migration", "hypergeometric encoding")

This is the main area where syntax differs.  Maple uses symbolic parameters;
q-Kangaroo encodes $q$-monomials as integer triples `(num, den, pow)`
representing $frac("num", "den") dot q^"pow"$.

#table(
  columns: (1fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*],
  ),
  [`qphihyper([a,b],[c],q,z,N)`], [`phi([(n,d,p),...], [...], z_n, z_d, z_p, N)`],
  [`qpsihyper([a,b],[c],q,z,N)`], [`psi([(n,d,p),...], [...], z_n, z_d, z_p, N)`],
  [`qgauss([a,b],[c],q,z,N)`], [`try_summation(upper, lower, z_n, z_d, z_p, N)`],
)

For example, the Maple parameter `q^3` becomes the triple `(1, 1, 3)` and
`1/2*q^5` becomes `(1, 2, 5)`.

=== Function Name Aliases
#index("Maple migration", "aliases")

Some function names differ from Garvan's packages, but the Maple names are
accepted as aliases at the `q>` prompt:

#table(
  columns: (1fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple Name*], [*q-Kangaroo Canonical (alias accepted)*],
  ),
  [`proveid`], [`prove_eta_id` (alias: `proveid`)],
  [`qZeil`], [`q_zeilberger` (alias: `qzeil`)],
  [`qgosper`], [`q_gosper` (alias: `qgosper`)],
  [`rankgf`], [`rank_gf` (alias: `rankgf`)],
  [`crankgf`], [`crank_gf` (alias: `crankgf`)],
  [`numbpart`], [`numbpart` (also: `partition_count`)],
  [`g2`], [`universal_mock_theta_g2` (alias: `g2`)],
  [`g3`], [`universal_mock_theta_g3` (alias: `g3`)],
)

All aliases are case-insensitive: `numbpart(100)` and `NUMBPART(100)` both
work.

=== Partition Counting

Maple's `numbpart(n)` is q-Kangaroo's canonical name.  Both `numbpart(n)`
and `partition_count(n)` are accepted.  The two-argument form
`numbpart(n, m)` counts partitions of $n$ with largest part at most $m$.

=== Session Conventions

q-Kangaroo's REPL uses the same conventions as Maple:

- `:=` for assignment (`f := partition_gf(50)`)
- `:` at end of statement suppresses output
- `%` refers to the last result (like Maple's `%`)


== Quick Reference Card
#index("Maple migration", "quick reference")

The most commonly used functions, at a glance:

#table(
  columns: (2fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Function Call*], [*Status*],
  ),
  [`etaq(q, delta, T)`], [Identical],
  [`aqprod(a, q, n)`], [Identical],
  [`sift(f, q, n, k, T)`], [Identical],
  [`prodmake(f, q, T)`], [Identical],
  [`findcong(QS, T)`], [Identical],
  [`findlincombo(f, L, SL, q, 0)`], [Identical],
  [`findprod(FL, T, M, Q)`], [Identical],
  [`numbpart(n)`], [Identical],
  [`theta(z, q, T)`], [Identical],
  [`JAC(a, b)`], [Identical],
  [`jacprod(a, b, q, T)`], [Identical],
  [`phi(upper, lower, z_n, z_d, z_p, N)`], [Triple encoding],
  [`proveid(...)`], [Alias accepted],
  [`qzeil(...)`], [Alias accepted],
)

Functions marked *Identical* accept exactly the syntax shown in Garvan's
Maple documentation.  Functions marked *Triple encoding* use `(num, den, pow)`
integer triples in place of symbolic $q$-monomials.  Functions marked
*Alias accepted* use a different canonical name but accept the Maple name.
