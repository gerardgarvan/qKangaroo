// 11-mock-theta-bailey.typ -- Mock Theta Functions and Bailey Chains

= Mock Theta Functions and Bailey Chains
#index[mock theta function]
#index[Bailey chain]
#index[Ramanujan]

Ramanujan introduced mock theta functions in his last letter to Hardy in
January 1920. He listed 17 examples --- seven of third order, ten of fifth
order --- and described them as functions at $q$-series level that "enter into
mathematics as beautifully as the ordinary theta functions." Unlike classical
theta functions, mock theta functions are _not_ modular forms, but they
exhibit a partial modularity that resisted precise characterization for over
80 years. The theory was finally unified by Zwegers (2002), who showed that
every mock theta function is the holomorphic part of a harmonic Maass form,
and that the Appell--Lerch sum provides the natural framework for this
correspondence.

Bailey chains provide a complementary algebraic approach to $q$-series
identities. A Bailey pair relative to $a$ is a pair of sequences
$(alpha_n, beta_n)$ satisfying a linear relation involving $q$-Pochhammer
products. Bailey's lemma transforms one Bailey pair into another, and
iterating this transformation produces a _Bailey chain_ --- an infinite
sequence of increasingly complex identities, including the Rogers--Ramanujan
identities as special cases.

== Third-Order Mock Theta Functions
#index("mock theta function", "third-order")

Ramanujan's original seven third-order mock theta functions from his 1920
letter to Hardy. Each takes a single `order` parameter specifying the
truncation order of the resulting $q$-series.

#func-entry(
  name: "mock_theta_f3",
  signature: "mock_theta_f3(order)",
  description: [
    Compute Ramanujan's third-order mock theta function $f(q)$. This is
    perhaps the most studied of all mock theta functions, appearing in
    Ramanujan's original letter and in numerous subsequent identities.
  ],
  math-def: [
    $ f(q) = sum_(n >= 0) frac(q^(n^2), (-q; q)_n^2) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_f3(10)", "1 + q - 2*q^2 + 3*q^3 + ... + O(q^10)"),
    ("mock_theta_f3(5)", "1 + q - 2*q^2 + 3*q^3 - 3*q^4 + O(q^5)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [At low orders, the series may appear polynomial; the mock theta property manifests in the asymptotic behavior of coefficients.],
  ),
  related: ("mock_theta_phi3", "mock_theta_psi3", "mock_theta_chi3", "mock_theta_omega3", "mock_theta_nu3", "mock_theta_rho3"),
)

#func-entry(
  name: "mock_theta_phi3",
  signature: "mock_theta_phi3(order)",
  description: [
    Compute Ramanujan's third-order mock theta function $phi(q)$.
  ],
  math-def: [
    $ phi(q) = sum_(n >= 0) frac(q^(n^2), (-q^2; q^2)_n) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_phi3(10)", "1 + q + q^2 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
  ),
  related: ("mock_theta_f3", "mock_theta_psi3", "mock_theta_chi3"),
)

#func-entry(
  name: "mock_theta_psi3",
  signature: "mock_theta_psi3(order)",
  description: [
    Compute Ramanujan's third-order mock theta function $psi(q)$. Note that
    the summation starts at $n = 1$, so the constant term is zero.
  ],
  math-def: [
    $ psi(q) = sum_(n >= 1) frac(q^(n^2), (q; q^2)_n) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_psi3(10)", "q + q^2 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [The constant term is always zero because the sum starts at $n = 1$.],
  ),
  related: ("mock_theta_f3", "mock_theta_phi3", "mock_theta_chi3"),
)

#func-entry(
  name: "mock_theta_chi3",
  signature: "mock_theta_chi3(order)",
  description: [
    Compute Ramanujan's third-order mock theta function $chi(q)$.
  ],
  math-def: [
    $ chi(q) = sum_(n >= 0) frac(q^(n^2) (-q; q)_n, product_(k >= 1) (1 - q^k + q^(2k))) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_chi3(10)", "1 + q + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [The denominator product involves cubic roots of unity factors $1 - q^k + q^(2k)$.],
  ),
  related: ("mock_theta_f3", "mock_theta_phi3", "mock_theta_psi3"),
)

#func-entry(
  name: "mock_theta_omega3",
  signature: "mock_theta_omega3(order)",
  description: [
    Compute Ramanujan's third-order mock theta function $omega(q)$.
  ],
  math-def: [
    $ omega(q) = sum_(n >= 0) frac(q^(2n(n+1)), (q; q^2)_(n+1)^2) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_omega3(10)", "1 + 2*q^2 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [The exponent $2n(n+1)$ grows quadratically, so few terms contribute at low orders.],
  ),
  related: ("mock_theta_nu3", "mock_theta_rho3", "mock_theta_f3"),
)

#func-entry(
  name: "mock_theta_nu3",
  signature: "mock_theta_nu3(order)",
  description: [
    Compute Ramanujan's third-order mock theta function $nu(q)$.
  ],
  math-def: [
    $ nu(q) = sum_(n >= 0) frac((-1)^n q^(n(n+1)), (-q; q^2)_(n+1)) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_nu3(10)", "1 - q + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [The alternating sign $(-1)^n$ causes sign changes in the coefficients.],
  ),
  related: ("mock_theta_omega3", "mock_theta_rho3", "mock_theta_f3"),
)

#func-entry(
  name: "mock_theta_rho3",
  signature: "mock_theta_rho3(order)",
  description: [
    Compute the third-order mock theta function $rho(q)$.
  ],
  math-def: [
    $ rho(q) = sum_(n >= 0) frac(q^(2n(n+1)), product_(m=1)^(n+1) (1 + q^m + q^(2m))) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_rho3(10)", "1 + q^2 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [Like $chi(q)$, the denominator involves cubic-root-of-unity factors.],
  ),
  related: ("mock_theta_omega3", "mock_theta_nu3", "mock_theta_chi3"),
)

== Fifth-Order Mock Theta Functions
#index("mock theta function", "fifth-order")

Ramanujan's ten fifth-order mock theta functions, also from his last letter
to Hardy. These are organized into five pairs: $(f_0, f_1)$, $(F_0, F_1)$,
$(phi_0, phi_1)$, $(psi_0, psi_1)$, and $(chi_0, chi_1)$. Each takes a
single `order` parameter.

#func-entry(
  name: "mock_theta_f0_5",
  signature: "mock_theta_f0_5(order)",
  description: [
    Compute Ramanujan's fifth-order mock theta function $f_0(q)$.
  ],
  math-def: [
    $ f_0(q) = sum_(n >= 0) frac(q^(n^2), (-q; q)_n) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_f0_5(10)", "1 + q + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
  ),
  related: ("mock_theta_f1_5", "mock_theta_f3"),
)

#func-entry(
  name: "mock_theta_f1_5",
  signature: "mock_theta_f1_5(order)",
  description: [
    Compute Ramanujan's fifth-order mock theta function $f_1(q)$. Note that
    the summation starts at $n = 1$.
  ],
  math-def: [
    $ f_1(q) = sum_(n >= 1) frac(q^(n^2), (q; q)_n) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_f1_5(10)", "q + q^2 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [The constant term is zero because the sum starts at $n = 1$.],
  ),
  related: ("mock_theta_f0_5", "mock_theta_f3"),
)

#func-entry(
  name: "mock_theta_cap_f0_5",
  signature: "mock_theta_cap_f0_5(order)",
  description: [
    Compute Ramanujan's fifth-order mock theta function $F_0(q)$ (capital $F$).
  ],
  math-def: [
    $ F_0(q) = sum_(n >= 0) frac(q^(2n^2), (q; q^2)_n) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_cap_f0_5(10)", "1 + q^2 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
  ),
  related: ("mock_theta_cap_f1_5", "mock_theta_f0_5"),
)

#func-entry(
  name: "mock_theta_cap_f1_5",
  signature: "mock_theta_cap_f1_5(order)",
  description: [
    Compute Ramanujan's fifth-order mock theta function $F_1(q)$ (capital $F$).
  ],
  math-def: [
    $ F_1(q) = sum_(n >= 1) frac(q^(2n^2 - 2n + 1), (q; q^2)_n) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_cap_f1_5(10)", "q + q^3 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [The exponent $2n^2 - 2n + 1$ ensures the leading term is $q$.],
  ),
  related: ("mock_theta_cap_f0_5", "mock_theta_f1_5"),
)

#func-entry(
  name: "mock_theta_phi0_5",
  signature: "mock_theta_phi0_5(order)",
  description: [
    Compute Ramanujan's fifth-order mock theta function $phi_0(q)$.
  ],
  math-def: [
    $ phi_0(q) = sum_(n >= 0) q^(n^2) (-q; q^2)_n $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_phi0_5(10)", "1 + q + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [Unlike most mock theta functions, the $q$-Pochhammer factor appears in the numerator.],
  ),
  related: ("mock_theta_phi1_5", "mock_theta_phi3"),
)

#func-entry(
  name: "mock_theta_phi1_5",
  signature: "mock_theta_phi1_5(order)",
  description: [
    Compute Ramanujan's fifth-order mock theta function $phi_1(q)$.
  ],
  math-def: [
    $ phi_1(q) = sum_(n >= 0) q^((n+1)^2) (-q; q^2)_n $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_phi1_5(10)", "q + q^2 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
  ),
  related: ("mock_theta_phi0_5", "mock_theta_phi3"),
)

#func-entry(
  name: "mock_theta_psi0_5",
  signature: "mock_theta_psi0_5(order)",
  description: [
    Compute Ramanujan's fifth-order mock theta function $psi_0(q)$.
  ],
  math-def: [
    $ psi_0(q) = sum_(n >= 0) q^((n+1)(n+2) slash 2) (-q; q)_n $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_psi0_5(10)", "q + q^2 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [The exponent $(n+1)(n+2)/2$ produces triangular-number-like powers.],
  ),
  related: ("mock_theta_psi1_5", "mock_theta_psi3"),
)

#func-entry(
  name: "mock_theta_psi1_5",
  signature: "mock_theta_psi1_5(order)",
  description: [
    Compute Ramanujan's fifth-order mock theta function $psi_1(q)$. Note that
    the summation starts at $n = 1$.
  ],
  math-def: [
    $ psi_1(q) = sum_(n >= 1) q^(n(n+1) slash 2) (-q; q)_(n-1) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_psi1_5(10)", "q + q^2 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [The $q$-Pochhammer factor uses index $n - 1$.],
  ),
  related: ("mock_theta_psi0_5", "mock_theta_psi3"),
)

#func-entry(
  name: "mock_theta_chi0_5",
  signature: "mock_theta_chi0_5(order)",
  description: [
    Compute Ramanujan's fifth-order mock theta function $chi_0(q)$. This
    function is computed internally via a $q arrow.r -q$ composition technique.
  ],
  math-def: [
    $ chi_0(q) = sum_(n >= 0) frac(q^n (-q; q)_(n-1), (q^(n+1); q)_n) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_chi0_5(10)", "1 + q + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [Uses the $q arrow.r -q$ composition method internally for numerical stability.],
  ),
  related: ("mock_theta_chi1_5", "mock_theta_chi3"),
)

#func-entry(
  name: "mock_theta_chi1_5",
  signature: "mock_theta_chi1_5(order)",
  description: [
    Compute Ramanujan's fifth-order mock theta function $chi_1(q)$. Like
    $chi_0$, this function uses the $q arrow.r -q$ composition technique.
  ],
  math-def: [
    $ chi_1(q) = sum_(n >= 0) frac(q^n (-q; q)_n, (q^(n+1); q)_n) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_chi1_5(10)", "q + q^2 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [Uses the $q arrow.r -q$ composition method internally for numerical stability.],
  ),
  related: ("mock_theta_chi0_5", "mock_theta_chi3"),
)

== Seventh-Order Mock Theta Functions
#index("mock theta function", "seventh-order")

Three seventh-order mock theta functions, discovered later than Ramanujan's
original third- and fifth-order examples. Each takes a single `order`
parameter.

#func-entry(
  name: "mock_theta_cap_f0_7",
  signature: "mock_theta_cap_f0_7(order)",
  description: [
    Compute the seventh-order mock theta function $F_0(q)$.
  ],
  math-def: [
    $ F_0(q) = sum_(n >= 0) frac(q^(n^2), (q^(n+1); q)_n) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_cap_f0_7(10)", "1 + q + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
  ),
  related: ("mock_theta_cap_f1_7", "mock_theta_cap_f2_7"),
)

#func-entry(
  name: "mock_theta_cap_f1_7",
  signature: "mock_theta_cap_f1_7(order)",
  description: [
    Compute the seventh-order mock theta function $F_1(q)$. Note that the
    summation starts at $n = 1$.
  ],
  math-def: [
    $ F_1(q) = sum_(n >= 1) frac(q^(n^2), (q^n; q)_n) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_cap_f1_7(10)", "q + q^2 + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
    [The constant term is zero because the sum starts at $n = 1$.],
  ),
  related: ("mock_theta_cap_f0_7", "mock_theta_cap_f2_7"),
)

#func-entry(
  name: "mock_theta_cap_f2_7",
  signature: "mock_theta_cap_f2_7(order)",
  description: [
    Compute the seventh-order mock theta function $F_2(q)$.
  ],
  math-def: [
    $ F_2(q) = sum_(n >= 0) frac(q^(n(n+1)), (q^(n+1); q)_(n+1)) $
  ],
  params: (
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("mock_theta_cap_f2_7(10)", "1 + q + ... + O(q^10)"),
  ),
  edge-cases: (
    [The order must be a positive integer.],
  ),
  related: ("mock_theta_cap_f0_7", "mock_theta_cap_f1_7"),
)

== Appell--Lerch Sums
#index[Appell-Lerch sum]
#index[Zwegers]

Zwegers' unifying framework for mock theta functions. The Appell--Lerch sum
$m(a, z, q)$ and the universal mock theta functions $g_2$ and $g_3$ provide
a systematic way to express all classical mock theta functions and to
understand their modular transformation properties.

#func-entry(
  name: "appell_lerch_m",
  signature: "appell_lerch_m(a_pow, z_pow, order)",
  description: [
    Compute the Appell--Lerch sum $m(a, z, q)$ where $a = q^(a_"pow")$ and
    $z = q^(z_"pow")$. This function provides the foundational building block
    for Zwegers' theory of mock theta functions.
  ],
  math-def: [
    $ m(a, z, q) = frac(1, j(z; q)) sum_(r in ZZ) frac((-1)^r z^r q^(r(r-1) slash 2), 1 - a q^r z) $

    where $j(z; q) = (z; q)_infinity (q slash z; q)_infinity (q; q)_infinity$ is the
    Jacobi theta function normalization.
  ],
  params: (
    ([a_pow], [Integer], [Exponent for $a = q^(a_"pow")$]),
    ([z_pow], [Integer], [Exponent for $z = q^(z_"pow")$]),
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("appell_lerch_m(1, 1, 10)", "series in q truncated to order 10"),
    ("appell_lerch_m(2, 1, 15)", "series in q truncated to order 15"),
  ),
  edge-cases: (
    [The parameters `a_pow` and `z_pow` must be integers.],
    [When $z = q^0 = 1$ or $a = q^0 = 1$, the sum may have poles; the implementation handles cancellation carefully.],
    [The Jacobi theta normalization $j(z; q)$ in the denominator is computed as a triple product.],
  ),
  related: ("universal_mock_theta_g2", "universal_mock_theta_g3", "mock_theta_f3"),
)

#func-entry(
  name: "universal_mock_theta_g2",
  signature: "universal_mock_theta_g2(a_pow, order)",
  description: [
    Compute the universal mock theta function $g_2(a; q)$ where $a = q^(a_"pow")$.
    The function $g_2$ relates to the Appell--Lerch sum via algebraic identities
    and provides a second-order universal mock theta function.
  ],
  math-def: [
    $ g_2(a; q) quad "where" a = q^(a_"pow") $
  ],
  params: (
    ([a_pow], [Integer], [Exponent for $a = q^(a_"pow")$]),
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("universal_mock_theta_g2(1, 10)", "series in q truncated to order 10"),
  ),
  edge-cases: (
    [The parameter `a_pow` must be an integer.],
    [Related to the Appell--Lerch sum $m(a, z, q)$ through specialization of parameters.],
  ),
  related: ("universal_mock_theta_g3", "appell_lerch_m"),
)

#func-entry(
  name: "universal_mock_theta_g3",
  signature: "universal_mock_theta_g3(a_pow, order)",
  description: [
    Compute the universal mock theta function $g_3(a; q)$ where $a = q^(a_"pow")$.
    The function $g_3$ relates to the Appell--Lerch sum via algebraic identities
    and provides a third-order universal mock theta function.
  ],
  math-def: [
    $ g_3(a; q) quad "where" a = q^(a_"pow") $
  ],
  params: (
    ([a_pow], [Integer], [Exponent for $a = q^(a_"pow")$]),
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("universal_mock_theta_g3(1, 10)", "series in q truncated to order 10"),
  ),
  edge-cases: (
    [The parameter `a_pow` must be an integer.],
    [Related to the Appell--Lerch sum $m(a, z, q)$ through specialization of parameters.],
  ),
  related: ("universal_mock_theta_g2", "appell_lerch_m"),
)

== Bailey Chains
#index[Bailey pair]
#index[Bailey's lemma]
#index[Rogers-Ramanujan]

A _Bailey pair_ relative to $a$ is a pair of sequences $(alpha_n, beta_n)$
satisfying

$ beta_n = sum_(k=0)^n frac(alpha_k, (q; q)_(n-k) (a q; q)_(n+k)). $

Bailey's lemma produces a new Bailey pair from an existing one by applying a
transformation matrix involving additional parameters $b$ and $c$. Iterating
this process produces a _Bailey chain_: an infinite sequence of pairs, each
yielding a new $q$-series identity. The Rogers--Ramanujan identities emerge
as a special case at depth 1 starting from the unit Bailey pair.

#func-entry(
  name: "bailey_weak_lemma",
  signature: "bailey_weak_lemma(pair_code, a_num, a_den, a_pow, max_n, order)",
  description: [
    Apply the weak form of Bailey's lemma to a known Bailey pair. The weak
    lemma is the simplest form of the Bailey transformation, without the
    additional parameters $b$ and $c$ of the full lemma.
  ],
  params: (
    ([pair_code], [Integer], [Selects the initial Bailey pair: 0 = Unit, 1 = Rogers--Ramanujan, 2 = $q$-Binomial]),
    ([a_num], [Integer], [Numerator of the parameter $a = (a_"num" slash a_"den") dot q^(a_"pow")$]),
    ([a_den], [Integer], [Denominator of the parameter $a$]),
    ([a_pow], [Integer], [Power of $q$ in the parameter $a$]),
    ([max_n], [Integer], [Maximum index for the Bailey pair computation]),
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("bailey_weak_lemma(1, 1, 1, 0, 10, 20)", "(alpha_series, beta_series)"),
    ("bailey_weak_lemma(0, 1, 1, 0, 5, 15)", "(alpha_series, beta_series)"),
  ),
  edge-cases: (
    [The `pair_code` must be 0, 1, or 2.],
    [The denominator `a_den` must be nonzero.],
    [The `max_n` parameter controls how many terms of the Bailey pair are computed.],
  ),
  related: ("bailey_apply_lemma", "bailey_chain", "bailey_discover"),
)

#func-entry(
  name: "bailey_apply_lemma",
  signature: "bailey_apply_lemma(pair_code, a_n, a_d, a_p, b_n, b_d, b_p, c_n, c_d, c_p, max_n, order)",
  description: [
    Apply the full form of Bailey's lemma to a Bailey pair, producing a new
    pair. The parameters $b$ and $c$ control the transformation matrix,
    allowing more general identity generation than the weak lemma.
  ],
  params: (
    ([pair_code], [Integer], [Selects the initial Bailey pair: 0 = Unit, 1 = Rogers--Ramanujan, 2 = $q$-Binomial]),
    ([a_n, a_d, a_p], [Integer], [Parameter $a = (a_n slash a_d) dot q^(a_p)$]),
    ([b_n, b_d, b_p], [Integer], [Parameter $b = (b_n slash b_d) dot q^(b_p)$]),
    ([c_n, c_d, c_p], [Integer], [Parameter $c = (c_n slash c_d) dot q^(c_p)$]),
    ([max_n], [Integer], [Maximum index for the Bailey pair computation]),
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("bailey_apply_lemma(0, 1,1,0, 1,1,1, 1,1,2, 10, 20)", "(new_alpha_series, new_beta_series)"),
  ),
  edge-cases: (
    [All denominators (`a_d`, `b_d`, `c_d`) must be nonzero.],
    [The `pair_code` must be 0, 1, or 2.],
    [The full lemma has 12 parameters; ensure all are provided.],
  ),
  related: ("bailey_weak_lemma", "bailey_chain", "bailey_discover"),
)

#func-entry(
  name: "bailey_chain",
  signature: "bailey_chain(pair_code, a_n, a_d, a_p, b_n, b_d, b_p, c_n, c_d, c_p, depth, max_n, order)",
  description: [
    Iterate the Bailey chain to the specified depth, starting from a known
    Bailey pair. Each iteration applies Bailey's lemma to produce a new pair.
    At depth 1 with the Rogers--Ramanujan pair, this yields the classical
    Rogers--Ramanujan identities.
  ],
  params: (
    ([pair_code], [Integer], [Selects the initial Bailey pair: 0 = Unit, 1 = Rogers--Ramanujan, 2 = $q$-Binomial]),
    ([a_n, a_d, a_p], [Integer], [Parameter $a$ (rational times $q$-power)]),
    ([b_n, b_d, b_p], [Integer], [Parameter $b$ (rational times $q$-power)]),
    ([c_n, c_d, c_p], [Integer], [Parameter $c$ (rational times $q$-power)]),
    ([depth], [Integer], [Number of Bailey lemma iterations to perform]),
    ([max_n], [Integer], [Maximum index for the Bailey pair computation]),
    ([order], [Integer], [Truncation order for the output series]),
  ),
  examples: (
    ("bailey_chain(1, 1,1,0, 1,1,1, 1,1,2, 3, 10, 20)", "(alpha_at_depth, beta_at_depth)"),
  ),
  edge-cases: (
    [The `depth` must be a non-negative integer; depth 0 returns the original pair.],
    [Higher depths produce increasingly complex series; truncation order should be generous.],
    [This function has 13 parameters.],
  ),
  related: ("bailey_weak_lemma", "bailey_apply_lemma", "bailey_discover"),
)

#func-entry(
  name: "bailey_discover",
  signature: "bailey_discover(lhs, rhs, a_num, a_den, a_pow, max_depth, order)",
  description: [
    Discover a Bailey pair that proves the identity `lhs` $=$ `rhs`. The
    algorithm tries multiple strategies in order: trivial equality check,
    database lookup against known Bailey pairs, weak lemma matching, and
    iterative chain depth search up to `max_depth`.
  ],
  params: (
    ([lhs], [Series], [Left-hand side of the identity to prove]),
    ([rhs], [Series], [Right-hand side of the identity to prove]),
    ([a_num], [Integer], [Numerator of the parameter $a$]),
    ([a_den], [Integer], [Denominator of the parameter $a$]),
    ([a_pow], [Integer], [Power of $q$ in the parameter $a$]),
    ([max_depth], [Integer], [Maximum Bailey chain depth to search]),
    ([order], [Integer], [Truncation order for series comparison]),
  ),
  examples: (
    ("bailey_discover(lhs_series, rhs_series, 1, 1, 0, 3, 20)", "proof description (or None if not found)"),
  ),
  edge-cases: (
    [Both `lhs` and `rhs` must be $q$-series (not integers or other types).],
    [Higher `max_depth` values increase search time exponentially.],
    [Returns `None` if no Bailey pair proof is found within the search depth.],
  ),
  related: ("bailey_chain", "bailey_weak_lemma", "bailey_apply_lemma", "prove_eta_id"),
)

// ---------------------------------------------------------------------------
// Index entries for mathematical concepts
// ---------------------------------------------------------------------------
#index[Hardy]
#index("mock theta function", "Ramanujan's letter to Hardy")
