// 12-identity-proving.typ -- Identity Proving function reference
#import "../template.typ": *

= Identity Proving
#index[identity proving]

Algorithmic methods for proving $q$-series identities. These functions go
beyond empirical verification --- checking that coefficients of two series
agree to some finite order --- to provide rigorous mathematical proofs. The
eta-quotient prover uses the valence formula for modular forms: if a modular
form of weight $k$ and level $N$ has more zeros (counted with multiplicity at
cusps) than $k N slash 12 dot product_(p | N) (1 + 1 slash p)$, then it is
identically zero. The $q$-Gosper, $q$-Zeilberger, and WZ machinery provides
computer-algebra proofs of hypergeometric summation identities through
creative telescoping and certificate verification.

The `search_identities` function provides a lookup interface into the
built-in identity database, while `prove_nonterminating` extends proving
capabilities to nonterminating hypergeometric identities (available only
through the Python API).

#func-entry(
  name: "prove_eta_id",
  signature: "prove_eta_id(terms_list, level)",
  description: [
    Prove an eta-quotient identity using the valence formula for modular
    forms. The function verifies that a linear combination of eta quotients
    is identically zero by checking that the number of zeros exceeds the
    bound imposed by the valence formula.
  ],
  math-def: [
    Uses the _valence formula_ for modular forms: if $f$ is a modular form of
    weight $k$ for $Gamma_0(N)$ with $"ord"(f) > k N / 12 product_(p | N)
    (1 + 1/p)$, then $f equiv 0$.

    Each term in `terms_list` specifies an eta quotient
    $product eta(d dot tau)^(r_d)$ together with a scalar coefficient.
  ],
  params: (
    ([terms_list], [List], [List of `(eta_args, coefficient)` pairs, where `eta_args` specifies the eta quotient via base-exponent pairs]),
    ([level], [Integer], [Modular group level $N$ for $Gamma_0(N)$]),
  ),
  examples: (
    ("prove_eta_id([([1,1], 1), ([1,-1], -1)], 1)", "true (identity proven) or false"),
    ("prove_eta_id([([1,24], 1), ([2,-24], -1)], 2)", "true"),
  ),
  edge-cases: (
    [The `level` must divide the LCM of all eta bases appearing in the terms.],
    [Works for modular forms of weight 0 (eta quotients whose exponents sum to zero).],
    [Returns `false` if the coefficient check is insufficient to prove the identity, which does not necessarily mean the identity is false.],
    [Large levels increase the valence bound and require more coefficients to be checked.],
  ),
  related: ("etaq", "search_identities"),
)
#index[eta-quotient identity]
#index[valence formula]
#index[modular form]

#func-entry(
  name: "search_identities",
  signature: "search_identities(search_type)",
  description: [
    Search for identities of a given type in the built-in database. Returns
    a list of known identities matching the search criteria. This is useful
    for finding starting points for identity exploration or for checking
    whether a suspected identity is already known.
  ],
  params: (
    ([search_type], [String], [Type of identity to search for: `"theta"`, `"eta"`, `"mock"`, `"bailey"`, or `"product"`]),
  ),
  examples: (
    ("search_identities(\"theta\")", "list of matching theta-function identities"),
    ("search_identities(\"bailey\")", "list of matching Bailey pair identities"),
  ),
  edge-cases: (
    [The `search_type` must be a string (enclosed in double quotes in the REPL).],
    [An unrecognized search type returns an empty list.],
    [The database is curated and does not contain all known identities; absence does not imply non-existence.],
  ),
  related: ("prove_eta_id", "bailey_discover"),
)

#func-entry(
  name: "q_gosper",
  signature: "q_gosper(upper_list, lower_list, z_num, z_den, z_pow, q_num, q_den)",
  description: [
    Apply the $q$-Gosper algorithm for indefinite $q$-hypergeometric summation.
    Given a $q$-hypergeometric term $t_k$ specified by its term ratio
    $t_(k+1) slash t_k$, the algorithm searches for a $q$-rational function
    $y_k$ such that the sum $sum t_k$ has the closed-form antidifference
    $y_k t_k$.
  ],
  math-def: [
    The algorithm finds $y_k$ (a $q$-rational function of $q^k$) such that
    $ sum_(k=0)^n t_k = y_(n+1) t_(n+1) - y_0 t_0 $
    where the term ratio $t_(k+1) slash t_k$ is determined by the upper and
    lower parameter lists. Not all $q$-hypergeometric sums are Gosper-summable;
    the algorithm returns `None` when no $q$-hypergeometric antidifference exists.
  ],
  params: (
    ([upper_list], [List], [Upper parameters as `(num, den, pow)` triples specifying $q^("pow") dot "num"/"den"$]),
    ([lower_list], [List], [Lower parameters as `(num, den, pow)` triples]),
    ([z_num], [Integer], [Numerator of the argument $z$]),
    ([z_den], [Integer], [Denominator of the argument $z$]),
    ([z_pow], [Integer], [Power of $q$ in $z$]),
    ([q_num], [Integer], [Numerator of the base $q$ (usually 1)]),
    ([q_den], [Integer], [Denominator of the base $q$ (usually 1)]),
  ),
  examples: (
    ("q_gosper([(1,1,0)], [(1,1,1)], 1, 1, 0, 1, 1)", "closed-form antidifference (or None)"),
  ),
  edge-cases: (
    [Not all $q$-hypergeometric sums have $q$-hypergeometric antidifferences.],
    [Returns `None` if no Gosper-summable form exists.],
    [The `q_num` and `q_den` parameters allow working with non-standard bases.],
  ),
  related: ("q_zeilberger", "phi", "verify_wz"),
)
#index[q-Gosper algorithm]
#index[indefinite summation]
#index[antidifference]

#func-entry(
  name: "q_zeilberger",
  signature: "q_zeilberger(upper_list, lower_list, z_num, z_den, z_pow, n, q_num, q_den, max_order)",
  description: [
    Apply $q$-Zeilberger's creative telescoping algorithm to find a recurrence
    relation for a definite $q$-hypergeometric sum. Given a summand $F(n, k)$
    specified by the upper/lower parameter lists, the algorithm produces a
    recurrence $sum_(j=0)^J a_j(q^n) S(n+j) = 0$ for the definite sum
    $S(n) = sum_k F(n, k)$.
  ],
  math-def: [
    _Creative telescoping_ finds polynomials $a_0(q^n), ..., a_J(q^n)$ and a
    _WZ certificate_ $R(n, k)$ such that
    $ sum_(j=0)^J a_j(q^n) F(n+j, k) = G(n, k+1) - G(n, k) $
    where $G(n, k) = R(n, k) F(n, k)$. Summing over $k$ yields the recurrence
    for $S(n)$.
  ],
  params: (
    ([upper_list], [List], [Upper parameters as `(num, den, pow)` triples]),
    ([lower_list], [List], [Lower parameters as `(num, den, pow)` triples]),
    ([z_num], [Integer], [Numerator of the argument $z$]),
    ([z_den], [Integer], [Denominator of the argument $z$]),
    ([z_pow], [Integer], [Power of $q$ in $z$]),
    ([n], [Integer], [Evaluation point for the summation index]),
    ([q_num], [Integer], [Numerator of the base $q$]),
    ([q_den], [Integer], [Denominator of the base $q$]),
    ([max_order], [Integer], [Maximum recurrence order to search]),
  ),
  examples: (
    ("q_zeilberger([(1,1,0),(1,1,1)], [(1,1,2)], 1, 1, 0, 5, 1, 1, 3)", "(recurrence_coeffs, certificate)"),
  ),
  edge-cases: (
    [The `max_order` controls the search depth; higher values find higher-order recurrences but take longer.],
    [Returns `None` if no recurrence of order $<= "max_order"$ exists.],
    [The returned certificate can be verified independently using `verify_wz`.],
  ),
  related: ("verify_wz", "q_gosper", "q_petkovsek"),
)
#index[q-Zeilberger algorithm]
#index[creative telescoping]
#index[recurrence relation]

#func-entry(
  name: "verify_wz",
  signature: "verify_wz(upper_list, lower_list, z_num, z_den, z_pow, n, q_num, q_den, max_order, max_k)",
  description: [
    Verify a Wilf--Zeilberger proof certificate. First runs `q_zeilberger` to
    produce the recurrence and certificate, then verifies that the certificate
    satisfies the WZ pair equations. A successful verification constitutes a
    rigorous proof of the underlying summation identity.
  ],
  params: (
    ([upper_list], [List], [Upper parameters as `(num, den, pow)` triples]),
    ([lower_list], [List], [Lower parameters as `(num, den, pow)` triples]),
    ([z_num], [Integer], [Numerator of the argument $z$]),
    ([z_den], [Integer], [Denominator of the argument $z$]),
    ([z_pow], [Integer], [Power of $q$ in $z$]),
    ([n], [Integer], [Evaluation point for the summation index]),
    ([q_num], [Integer], [Numerator of the base $q$]),
    ([q_den], [Integer], [Denominator of the base $q$]),
    ([max_order], [Integer], [Maximum recurrence order to search]),
    ([max_k], [Integer], [Maximum $k$ value for certificate verification]),
  ),
  examples: (
    ("verify_wz([(1,1,0),(1,1,1)], [(1,1,2)], 1, 1, 0, 5, 1, 1, 3, 10)", "true (certificate valid) or false"),
  ),
  edge-cases: (
    [Returns `false` if `q_zeilberger` fails to find a recurrence.],
    [The `max_k` parameter limits the range over which the certificate is checked; larger values give stronger verification.],
    [A `true` result constitutes a rigorous proof, not merely empirical evidence.],
  ),
  related: ("q_zeilberger", "q_gosper"),
)
#index[WZ proof]
#index[Wilf-Zeilberger]

#func-entry(
  name: "q_petkovsek",
  signature: "q_petkovsek(coeff_list, q_num, q_den)",
  description: [
    Solve a $q$-holonomic recurrence using the $q$-Petkovsek algorithm. Given
    a recurrence with polynomial coefficients in $q^n$, the algorithm finds
    all solutions that are $q$-hypergeometric terms --- that is, sequences
    $a_n$ where $a_(n+1) slash a_n$ is a rational function of $q^n$.
  ],
  params: (
    ([coeff_list], [List], [Polynomial coefficients of the recurrence, from lowest to highest order]),
    ([q_num], [Integer], [Numerator of the base $q$]),
    ([q_den], [Integer], [Denominator of the base $q$]),
  ),
  examples: (
    ("q_petkovsek([1, -1, 1], 1, 1)", "list of q-hypergeometric solutions"),
  ),
  edge-cases: (
    [The `coeff_list` must contain at least 2 elements (a nontrivial recurrence).],
    [Returns an empty list if no $q$-hypergeometric solutions exist.],
    [The algorithm finds _all_ $q$-hypergeometric solutions, not just one.],
  ),
  related: ("q_zeilberger", "q_gosper"),
)
#index[q-Petkovsek algorithm]
#index[q-holonomic recurrence]

#func-entry(
  name: "prove_nonterminating",
  signature: "prove_nonterminating(requires Python API)",
  description: [
    Prove a nonterminating hypergeometric identity using symbolic parameter
    manipulation. This function requires closure support for representing
    symbolic parameters, which is available only through the Python API
    (`q_kangaroo` package), not the CLI REPL.
  ],
  params: (),
  examples: (
    ("prove_nonterminating(...)", "Error: prove_nonterminating requires the Python API"),
  ),
  edge-cases: (
    [*This function is NOT available in the CLI.* Calling it from the REPL produces an error message directing the user to the Python API.],
    [Requires the `q_kangaroo` Python package to be installed (via `pip install q_kangaroo`).],
    [Uses closure-based symbolic parameters to manipulate nonterminating series, which cannot be represented in the CLI's expression language.],
  ),
  related: ("phi", "try_summation", "q_gosper"),
)
