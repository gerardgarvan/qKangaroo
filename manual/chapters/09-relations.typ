// 09-relations.typ -- Relation Discovery function reference

= Relation Discovery
#index[relation discovery]

These functions use Gaussian elimination over $QQ$ (or $ZZ slash p ZZ$) to discover
algebraic relations among q-series. They are the primary research tools for
finding new identities. All functions operate on truncated series and compare
coefficients up to the truncation order. Several functions accept a `topshift`
parameter that controls how many leading coefficients to skip -- useful when series
have known leading terms that should not participate in the relation search.

#index[Gaussian elimination]
#index[RREF]

== Linear Combinations

These functions find explicit coefficient vectors expressing a target series as
a polynomial combination of candidate series.

#func-entry(
  name: "findlincombo",
  signature: "findlincombo(target, [candidates], topshift)",
  description: [
    Find rational coefficients $c_i$ such that
    $ "target" = sum_i c_i dot "candidates"[i]. $
    Uses exact arithmetic over $QQ$ via reduced row echelon form (RREF).
    This is the main workhorse for discovering linear relations among q-series:
    given a target series and a list of candidate series, it determines whether
    the target can be written as a rational linear combination of the candidates,
    and if so returns the coefficients.
  ],
  params: (
    ([target], [Series], [The series to express as a linear combination]),
    ([candidates], [List of Series], [List of candidate series to combine]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findlincombo(partition_gf(30), [distinct_parts_gf(30), odd_parts_gf(30)], 0)",
     "[0, 1]"),
    ("findlincombo(theta3(50)^2, [theta2(50)^2, theta4(50)^2], 0)",
     "null"),
  ),
  edge-cases: (
    [Returns `null` if no linear combination exists within the truncation order.],
    [The candidates list must be non-empty.],
    [Coefficients are exact rationals, not floating-point approximations.],
  ),
  related: ("findhomcombo", "findnonhomcombo", "findlincombomodp"),
)

#func-entry(
  name: "findhomcombo",
  signature: "findhomcombo(target, [candidates], degree, topshift)",
  description: [
    Find a homogeneous polynomial combination of the given degree matching
    the target. Generates all monomials of exact degree $d$ in the candidate
    series, then applies RREF over $QQ$ to find coefficients. For example,
    with degree 2 and candidates $[f, g, h]$, the search space includes
    $f^2$, $f g$, $f h$, $g^2$, $g h$, $h^2$.
  ],
  params: (
    ([target], [Series], [The target series to match]),
    ([candidates], [List of Series], [Candidate series for building monomials]),
    ([degree], [Integer], [Degree of the homogeneous polynomial]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findhomcombo(theta3(50)^4, [theta2(50)^2, theta4(50)^2], 2, 0)",
     "homogeneous degree-2 coefficients"),
  ),
  edge-cases: (
    [Returns `null` if no homogeneous combination of the given degree exists.],
    [The number of monomials grows with $binom(n + d - 1, d)$ where $n$ is the number of candidates.],
  ),
  related: ("findlincombo", "findnonhomcombo", "findhomcombomodp"),
)

#func-entry(
  name: "findnonhomcombo",
  signature: "findnonhomcombo(target, [candidates], degree, topshift)",
  description: [
    Find a nonhomogeneous polynomial combination up to the given degree matching
    the target. Includes all monomials from degree 0 (constant term) through
    degree $d$ in the candidate series, providing a more flexible search than
    `findhomcombo`.
  ],
  params: (
    ([target], [Series], [The target series to match]),
    ([candidates], [List of Series], [Candidate series for building monomials]),
    ([degree], [Integer], [Maximum degree of the polynomial]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findnonhomcombo(partition_gf(30), [etaq(1,1,30), etaq(2,1,30)], 3, 0)",
     "polynomial combination coefficients"),
  ),
  edge-cases: (
    [Returns `null` if no polynomial combination up to the given degree exists.],
    [Includes the constant monomial (degree 0), so this subsumes linear search.],
  ),
  related: ("findlincombo", "findhomcombo"),
)

#func-entry(
  name: "findlincombomodp",
  signature: "findlincombomodp(target, [candidates], p, topshift)",
  description: [
    Find a linear combination matching the target with arithmetic performed
    modulo a prime $p$. Uses Fermat's little theorem for modular inverse
    ($a^(-1) equiv a^(p-2) mod p$). This is useful when exact rational
    arithmetic is too expensive -- for instance, when working with very long
    series where rational coefficient growth causes slowdown.
  ],
  #index[modular arithmetic]
  params: (
    ([target], [Series], [The target series]),
    ([candidates], [List of Series], [Candidate series to combine]),
    ([p], [Integer], [A prime modulus]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findlincombomodp(partition_gf(100), [odd_parts_gf(100)], 7, 0)",
     "coefficients in Z/7Z"),
  ),
  edge-cases: (
    [The modulus $p$ must be prime for Fermat inverse to work correctly.],
    [Results are only valid modulo $p$ -- a match mod $p$ does not guarantee an exact rational relation.],
  ),
  related: ("findlincombo", "findhomcombomodp"),
)

#func-entry(
  name: "findhomcombomodp",
  signature: "findhomcombomodp(target, [candidates], p, degree, topshift)",
  description: [
    Find a homogeneous polynomial combination of the given degree, with
    arithmetic performed modulo a prime $p$. Combines the monomial generation
    of `findhomcombo` with the modular arithmetic of `findlincombomodp`.
  ],
  params: (
    ([target], [Series], [The target series]),
    ([candidates], [List of Series], [Candidate series for building monomials]),
    ([p], [Integer], [A prime modulus]),
    ([degree], [Integer], [Degree of the homogeneous polynomial]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findhomcombomodp(theta3(50)^4, [theta2(50)^2, theta4(50)^2], 5, 2, 0)",
     "polynomial combination coefficients mod 5"),
  ),
  edge-cases: (
    [The modulus $p$ must be prime.],
    [Useful for large-scale searches where exact arithmetic is prohibitive.],
  ),
  related: ("findhomcombo", "findlincombomodp"),
)

== Relation Finding
#index[relation finding]

These functions search for polynomial relations _among_ a list of series
(without a distinguished target). They return the null space of the
coefficient matrix -- that is, any nonzero vector in the kernel represents
a relation.

#func-entry(
  name: "findhom",
  signature: "findhom([series], degree, topshift)",
  description: [
    Find a homogeneous polynomial relation of the given degree among the
    series in the input list. Constructs all monomials of degree $d$ from
    the input series, builds a coefficient matrix, and computes its null
    space via RREF. A nonzero null-space vector corresponds to a polynomial
    identity among the series.
  ],
  params: (
    ([series], [List of Series], [Series among which to find a relation]),
    ([degree], [Integer], [Degree of the homogeneous polynomial relation]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findhom([theta3(50)^2, theta2(50)^2, theta4(50)^2], 1, 0)",
     "[1, -1, -1]"),
  ),
  edge-cases: (
    [Returns the null space, which may contain multiple independent relations.],
    [An empty null space means no relation of the given degree exists.],
  ),
  related: ("findnonhom", "findhommodp", "findhomcombo"),
)

#func-entry(
  name: "findnonhom",
  signature: "findnonhom([series], degree, topshift)",
  description: [
    Find a nonhomogeneous relation of the given degree among the series.
    Includes constant and lower-degree terms in the monomial basis, so this
    can discover relations like $f^2 + g - 3 = 0$ that `findhom` would miss.
  ],
  params: (
    ([series], [List of Series], [Series among which to find a relation]),
    ([degree], [Integer], [Maximum degree of the polynomial relation]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findnonhom([theta3(50)^2, theta2(50)^2, theta4(50)^2], 1, 0)",
     "relation coefficients"),
  ),
  edge-cases: (
    [The constant term (degree 0) is always included in the monomial basis.],
    [Returns the null space of the augmented coefficient matrix.],
  ),
  related: ("findhom", "findnonhomcombo"),
)

#func-entry(
  name: "findhommodp",
  signature: "findhommodp([series], p, degree, topshift)",
  description: [
    Find a homogeneous relation among the series with arithmetic performed
    modulo a prime $p$. Useful when exact rational arithmetic is too
    expensive for the size of the series or the degree of the relation.
  ],
  params: (
    ([series], [List of Series], [Series among which to find a relation]),
    ([p], [Integer], [A prime modulus]),
    ([degree], [Integer], [Degree of the homogeneous polynomial relation]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findhommodp([theta3(50)^2, theta2(50)^2, theta4(50)^2], 7, 1, 0)",
     "relation coefficients mod 7"),
  ),
  edge-cases: (
    [The modulus $p$ must be prime.],
    [A relation mod $p$ suggests but does not prove an exact relation over $QQ$.],
  ),
  related: ("findhom", "findlincombomodp"),
)

#func-entry(
  name: "findmaxind",
  signature: "findmaxind([series], topshift)",
  description: [
    Find a maximally independent subset of the given series via Gaussian
    elimination. Returns the indices of the pivot columns in the coefficient
    matrix, identifying which series form a basis for the span of the full
    list. This is useful for determining the dimension of a space of modular
    forms or checking whether a new series is linearly independent of known
    ones.
  ],
  params: (
    ([series], [List of Series], [Series to test for independence]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findmaxind([partition_gf(30), odd_parts_gf(30), distinct_parts_gf(30)], 0)",
     "[0, 2]"),
  ),
  edge-cases: (
    [Returns indices (0-based) of the pivot columns after RREF.],
    [If all series are independent, returns all indices.],
    [The returned set is not unique -- different orderings may produce different pivot selections.],
  ),
  related: ("findhom", "findlincombo"),
)

== Specialized Searches

These functions perform targeted searches for specific types of identities:
product relations, partition congruences, and polynomial relations between
two series.

#index[product identity]
#index[partition congruences]

#func-entry(
  name: "findprod",
  signature: "findprod([series], max_coeff, max_exp)",
  description: [
    Find a product identity among the given series by brute-force search over
    exponent combinations. Tests
    $ product_i "series"[i]^(e_i) $
    for all integer exponent vectors with $|e_i| <= "max_coeff"$, checking
    whether any combination produces $1 + O(q^N)$.
  ],
  params: (
    ([series], [List of Series], [Series to combine as a product]),
    ([max\_coeff], [Integer], [Maximum absolute value of exponents to test]),
    ([max\_exp], [Integer], [Maximum exponent range]),
  ),
  examples: (
    ("findprod([etaq(1,1,30), etaq(2,1,30)], 3, 2)",
     "exponent vector (if product identity exists)"),
  ),
  edge-cases: (
    [Returns `null` if no product identity is found within the search bounds.],
    [Search time grows exponentially with the number of series and `max_coeff`.],
    [Only detects identities where the product equals $1 + O(q^N)$.],
  ),
  related: ("findhom", "findpoly"),
)

#func-entry(
  name: "findcong",
  signature: "findcong(series, [moduli])",
  description: [
    Find partition-type congruences by checking whether sifted arithmetic
    subsequences of the input series vanish modulo specified moduli. For each
    modulus $m$ and residue $r$, extracts the subsequence of coefficients at
    positions $m n + r$ and checks divisibility. Reports all $(m, r, p)$
    triples where the sifted coefficients are divisible by the prime $p$.

    This is the tool for rediscovering Ramanujan's celebrated congruences:
    $p(5n + 4) equiv 0 mod 5$, $p(7n + 5) equiv 0 mod 7$, and
    $p(11n + 6) equiv 0 mod 11$.
  ],
  #index[Ramanujan congruences]
  #index[congruence]
  params: (
    ([series], [Series], [The generating function to analyze]),
    ([moduli], [List of Integer], [List of moduli to test]),
  ),
  examples: (
    ("findcong(partition_gf(200), [5, 7, 11])",
     "list of (modulus, residue, prime) triples"),
  ),
  edge-cases: (
    [The series must have enough terms for the sifted subsequences to be meaningful.],
    [Only checks divisibility by the moduli in the list, not all primes.],
  ),
  related: ("sift", "findlincombo"),
)

#func-entry(
  name: "findpoly",
  signature: "findpoly(x, y, deg_x, deg_y, topshift)",
  description: [
    Find a polynomial relation $P(x, y) = 0$ between two series $x$ and $y$.
    Searches for a polynomial $P$ of degree at most `deg_x` in $x$ and
    `deg_y` in $y$ whose evaluation at the two series vanishes to the
    truncation order.
  ],
  params: (
    ([x], [Series], [First series]),
    ([y], [Series], [Second series]),
    ([deg\_x], [Integer], [Maximum degree in $x$]),
    ([deg\_y], [Integer], [Maximum degree in $y$]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findpoly(theta3(50)^4, theta2(50)^4 + theta4(50)^4, 1, 1, 0)",
     "polynomial coefficients (if relation exists)"),
  ),
  edge-cases: (
    [Returns `null` if no polynomial relation of the given degree bounds exists.],
    [Higher degree bounds increase both the chance of finding a relation and the computation time.],
  ),
  related: ("findhom", "findnonhom", "findprod"),
)
