// 14-maple-migration.typ -- Maple Migration Quick Reference

= Maple Migration Quick Reference
#index-main[Maple migration]
#index[aliases]

Users familiar with Frank Garvan's Maple packages (`qseries`, `thetaids`,
`ETA`) can use q-Kangaroo as a direct replacement for most operations.
The tables below map Maple function names to their q-Kangaroo equivalents.
q-Kangaroo also accepts most Maple names as aliases at the REPL prompt,
so existing muscle memory transfers immediately.

== Alias Table
#index("Maple migration", "alias table")

The following 17 Maple function names are recognised as aliases in
q-Kangaroo. You can type either name at the `q>` prompt:

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple Name*], [*q-Kangaroo Name*], [*Notes*],
  ),
  [`numbpart`], [`partition_count`], [Integer output (not a series)],
  [`rankgf`], [`rank_gf`], [Dyson rank generating function],
  [`crankgf`], [`crank_gf`], [Andrews-Garvan crank generating function],
  [`qphihyper`], [`phi`], [Basic hypergeometric ${}_r phi_s$],
  [`qpsihyper`], [`psi`], [Bilateral hypergeometric ${}_r psi_s$],
  [`qgauss`], [`try_summation`], [Tries q-Gauss, q-Vandermonde, q-Chu-Vandermonde],
  [`proveid`], [`prove_eta_id`], [Eta-quotient identity prover (valence formula)],
  [`qzeil`], [`q_zeilberger`], [Creative telescoping (shortened alias)],
  [`qzeilberger`], [`q_zeilberger`], [Creative telescoping (alternative spelling)],
  [`qpetkovsek`], [`q_petkovsek`], [q-hypergeometric recurrence solver],
  [`qgosper`], [`q_gosper`], [Indefinite q-hypergeometric summation],
  [`findlincombo_modp`], [`findlincombomodp`], [Underscore removed],
  [`findhom_modp`], [`findhommodp`], [Underscore removed],
  [`findhomcombo_modp`], [`findhomcombomodp`], [Underscore removed],
  [`search_id`], [`search_identities`], [Expanded name],
  [`g2`], [`universal_mock_theta_g2`], [Zwegers $g_2$ short alias],
  [`g3`], [`universal_mock_theta_g3`], [Zwegers $g_3$ short alias],
)

#index("Maple functions", "numbpart")
#index("Maple functions", "rankgf")
#index("Maple functions", "crankgf")
#index("Maple functions", "qphihyper")
#index("Maple functions", "qpsihyper")
#index("Maple functions", "qgauss")
#index("Maple functions", "proveid")
#index("Maple functions", "qzeil")
#index("Maple functions", "qzeilberger")
#index("Maple functions", "qpetkovsek")
#index("Maple functions", "qgosper")
#index("Maple functions", "findlincombo_modp")
#index("Maple functions", "findhom_modp")
#index("Maple functions", "findhomcombo_modp")
#index("Maple functions", "search_id")
#index("Maple functions", "g2")
#index("Maple functions", "g3")

All aliases are case-insensitive. You can type `numbpart(100)` or
`NUMBPART(100)` at the REPL and both resolve to `partition_count(100)`.


== Complete Function Mapping
#index("Maple migration", "complete mapping")

The following table maps every Maple function from Garvan's packages to its
q-Kangaroo equivalent. Functions marked *Extension* have no Maple
counterpart.

=== Group 1: Pochhammer and q-Binomial

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*], [*Notes*],
  ),
  [`aqprod(a, q, n)`], [`aqprod(num, den, pow, n, order)`], [Monomial $a = frac("num", "den") q^"pow"$],
  [`aqprod(a, q, infinity)`], [`aqprod(num, den, pow, infinity, order)`], [$n =$ `infinity` for $(a;q)_oo$],
  [`qbin(n, k, q)`], [`qbin(n, k, order)`], [Gaussian binomial $binom(n, k)_q$],
)

=== Group 2: Named Products

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*], [*Notes*],
  ),
  [`etaq(d, t, q, N)`], [`etaq(b, t, order)`], [$q^(b t\/24) product (1 - q^(b k))^t$],
  [`jacprod(a, b, q, N)`], [`jacprod(a, b, order)`], [Jacobi triple product $J(a, b)$],
  [`tripleprod(z, b, t, q, N)`], [`tripleprod(num, den, pow, order)`], [Triple product with $z$ parameter],
  [`quinprod(z, q, N)`], [`quinprod(num, den, pow, order)`], [Quintuple product],
  [`winquist(a, b, q, N)`], [`winquist(a_n, a_d, a_p, b_n, b_d, b_p, order)`], [Winquist 10-factor product],
)

=== Group 3: Theta Functions

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*], [*Notes*],
  ),
  [`theta2(q, N)`], [`theta2(order)`], [$theta_2(q)$ in $q^(1\/4)$ convention],
  [`theta3(q, N)`], [`theta3(order)`], [$theta_3(q) = sum q^(n^2)$],
  [`theta4(q, N)`], [`theta4(order)`], [$theta_4(q) = sum (-1)^n q^(n^2)$],
)

=== Group 4: Partition Functions

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*], [*Notes*],
  ),
  [`numbpart(n)`], [`partition_count(n)`], [Exact $p(n)$ as integer],
  [`seq(numbpart(n), ...)`], [`partition_gf(order)`], [$1\/(q;q)_oo$ as series],
  [`rankgf(z, q, N)`], [`rank_gf(z_num, z_den, order)`], [Dyson rank GF $R(z;q)$],
  [`crankgf(z, q, N)`], [`crank_gf(z_num, z_den, order)`], [Andrews-Garvan crank GF $C(z;q)$],
  [---], [`distinct_parts_gf(order)`], [*Extension:* $(-q;q)_oo$],
  [---], [`odd_parts_gf(order)`], [*Extension:* $1\/(q;q^2)_oo$],
  [---], [`bounded_parts_gf(max, order)`], [*Extension:* parts $<= "max"$],
)

=== Group 5: Series Analysis

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*], [*Notes*],
  ),
  [`sift(f, m, j)`], [`sift(series, m, j)`], [Extract $a_(m n + j)$ subsequence],
  [---], [`qdegree(series)`], [*Extension:* highest $q$-power],
  [---], [`lqdegree(series)`], [*Extension:* lowest $q$-power],
  [`qfactor(f, q, N)`], [`qfactor(series)`], [Factor into $(1 - q^i)$ factors],
  [`prodmake(f, q, N)`], [`prodmake(series, n)`], [Andrews' product algorithm],
  [`etamake(f, q, N)`], [`etamake(series, n)`], [Eta-quotient form],
  [---], [`jacprodmake(series, n)`], [*Extension:* Jacobi product form],
  [`mprodmake(f, q, N)`], [`mprodmake(series, n)`], [$(1+q^n)$ product form],
  [---], [`qetamake(series, n)`], [*Extension:* combined eta/q-Pochhammer form],
)

=== Group 6: Relation Discovery (Exact)

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*], [*Notes*],
  ),
  [`findlincombo(...)`], [`findlincombo(target, candidates, topshift)`], [$"target" = sum c_i f_i$],
  [`findhom(...)`], [`findhom(series_list, degree, topshift)`], [Homogeneous polynomial relation],
  [`findhomcombo(...)`], [`findhomcombo(target, cands, degree, topshift)`], [Homogeneous combo],
  [`findnonhom(...)`], [`findnonhom(series_list, degree, topshift)`], [Non-homogeneous relation],
  [`findnonhomcombo(...)`], [`findnonhomcombo(target, cands, deg, topshift)`], [Non-homogeneous combo],
  [`findprod(...)`], [`findprod(series_list, max_coeff, max_exp)`], [Product identity search],
  [`findmaxind(...)`], [`findmaxind(series_list, topshift)`], [Maximal independent subset],
  [`findpoly(...)`], [`findpoly(series_list, degree, topshift)`], [Polynomial relation],
)

=== Group 7: Relation Discovery (Modular)

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*], [*Notes*],
  ),
  [`findcong(f, moduli, q, N)`], [`findcong(series, moduli)`], [Discover congruences $a_(m n + j) equiv 0$],
  [`findlincombo_modp(...)`], [`findlincombomodp(target, cands, p, topshift)`], [Linear relation mod $p$],
  [`findhom_modp(...)`], [`findhommodp(series_list, p, degree, topshift)`], [Polynomial mod $p$],
  [`findhomcombo_modp(...)`], [`findhomcombomodp(target, cands, p, deg, topshift)`], [Combo mod $p$],
)

=== Group 8: Hypergeometric

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*], [*Notes*],
  ),
  [`qphihyper([a1,...], [b1,...], q, z, N)`], [`phi(upper, lower, z_n, z_d, z_p, order)`], [${}_r phi_s$ basic hypergeometric],
  [`qpsihyper([a1,...], [b1,...], q, z, N)`], [`psi(upper, lower, z_n, z_d, z_p, order)`], [${}_r psi_s$ bilateral],
  [`qgauss(...)`], [`try_summation(upper, lower, z_n, z_d, z_p, order)`], [Classical summation formulas],
  [`heine1(...)`], [`heine1(upper, lower, z_n, z_d, z_p, order)`], [Heine transform I],
  [`heine2(...)`], [`heine2(upper, lower, z_n, z_d, z_p, order)`], [Heine transform II],
  [`heine3(...)`], [`heine3(upper, lower, z_n, z_d, z_p, order)`], [Heine transform III],
  [---], [`sears_transform(upper, lower, z_n, z_d, z_p, order)`], [*Extension:* Sears balanced ${}_4 phi_3$],
  [---], [`watson_transform(upper, lower, z_n, z_d, z_p, order)`], [*Extension:* Watson ${}_8 phi_7 arrow.r {}_4 phi_3$],
  [---], [`find_transformation_chain(...)`], [*Extension:* BFS transformation search],
)

=== Group 9: Identity Proving

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*], [*Notes*],
  ),
  [`proveid(lhs, rhs, level)`], [`prove_eta_id(lhs_factors, rhs_factors, level)`], [Valence formula proof],
  [---], [`search_identities(query, type)`], [*Extension:* identity database search],
  [`qgosper(...)`], [`q_gosper(upper, lower, z_n, z_d, z_p, q_n, q_d)`], [Indefinite summation],
  [`qZeil(...)`], [`q_zeilberger(upper, lower, z, n, q, max_order)`], [Creative telescoping],
  [---], [`verify_wz(upper, lower, z, n, q, max_order, max_k)`], [*Extension:* WZ certificate verification],
  [`qPetkovsek(...)`], [`q_petkovsek(coefficients, q_num, q_den)`], [Recurrence solver],
  [---], [`prove_nonterminating(...)`], [*Extension:* Chen-Hou-Mu proof],
)

=== Group 10: Mock Theta, Appell-Lerch, and Bailey

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*], [*Notes*],
  ),
  [---], [`mock_theta_f3` ... `mock_theta_rho3`], [*Extension:* 7 third-order mock theta functions],
  [---], [`mock_theta_f0_5` ... `mock_theta_cap_f1_5`], [*Extension:* 10 fifth-order mock theta functions],
  [---], [`mock_theta_cap_f0_7` ... `cap_f2_7`], [*Extension:* 3 seventh-order mock theta functions],
  [---], [`appell_lerch_m(a_pow, z_pow, order)`], [*Extension:* Appell-Lerch sum $m(a, z, q)$],
  [`g2(...)`], [`universal_mock_theta_g2(a_pow, order)`], [Zwegers $g_2(a; q)$],
  [`g3(...)`], [`universal_mock_theta_g3(a_pow, order)`], [Zwegers $g_3(a; q)$],
  [---], [`bailey_weak_lemma(...)`], [*Extension:* apply Bailey's weak lemma],
  [---], [`bailey_apply_lemma(...)`], [*Extension:* apply Bailey's full lemma],
  [---], [`bailey_chain(...)`], [*Extension:* iterate Bailey chain to depth $d$],
  [---], [`bailey_discover(...)`], [*Extension:* discover Bailey pair proof],
)


== Key Differences
#index("Maple migration", "key differences")

- *Parameter encoding.* Maple uses symbolic parameters ($a$, $b$, $z$);
  q-Kangaroo encodes monomials as integer triples `(num, den, pow)`
  representing $frac("num", "den") dot q^"pow"$. For example, the Maple
  parameter `q^3` becomes `(1, 1, 3)` and `1/2*q^5` becomes `(1, 2, 5)`.

- *Series display.* q-Kangaroo uses $q$ as the indeterminate (not $x$).
  Truncation is shown as $O(q^N)$ where $N$ is the truncation order.

- *Assignment.* Both Maple and q-Kangaroo use `:=` for variable assignment.
  In q-Kangaroo, `:` at the end of a statement suppresses output
  (analogous to Maple's terminating colon).

- *No symbolic parameters.* q-Kangaroo works with concrete $q$-series,
  not formal symbols. All parameters are evaluated numerically before
  series construction.

- *No session argument.* In the REPL, functions are called directly
  (`partition_gf(20)`) without a session object. The Python API requires
  a `QSession` first argument, but the CLI does not.
