// 09-relations.typ -- Relation Discovery function reference
#import "../template.typ": *

= Relation Discovery
#index[relation discovery]

These functions use Gaussian elimination over $QQ$ (or $ZZ slash p ZZ$) to discover
algebraic relations among q-series. They are the primary research tools for
finding new identities. All functions operate on truncated series and compare
coefficients up to the truncation order.

In v2.0, all relation functions now accept Garvan's exact calling conventions:
the series variable $q$ is passed explicitly, symbolic labels (`SL`) can be
provided for readable output, and auto-generated `X[i]` labels are used when
no explicit labels are given. Functions that search for combinations print
their results using these labels.

#index[Gaussian elimination]
#index[RREF]

== Linear Combinations

These functions find explicit coefficient vectors expressing a target series as
a polynomial combination of candidate series.

#func-entry(
  name: "findlincombo",
  signature: "findlincombo(f, L, SL, q, topshift)",
  description: [
    Find rational coefficients $c_i$ such that
    $ f = sum_i c_i dot L[i] $
    using exact arithmetic over $QQ$ via reduced row echelon form (RREF).
    The result is printed using the symbolic labels `SL`. For example, if
    `SL = [D, O]` and the result is `[0, 1]`, it prints `1*O`. Prints
    `"NOT A LINEAR COMBO."` on failure.
    #index[findlincombo]
    #index[linear combination]
  ],
  params: (
    ([f], [Series], [The target series to express as a linear combination]),
    ([L], [List of Series], [List of candidate basis series]),
    ([SL], [List of Symbol], [Symbolic labels for the candidates (bare symbols like `D`, `O`)]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("f := partition_gf(30): findlincombo(f, [distinct_parts_gf(30), odd_parts_gf(30)], [D, O], q, 0)",
     "1*O"),
  ),
  edge-cases: (
    [Prints `"NOT A LINEAR COMBO."` if no linear combination exists within the truncation order.],
    [The `SL` labels must be bare symbols (not strings) and must match the length of `L`.],
    [Coefficients are exact rationals, not floating-point approximations.],
    [The `topshift` parameter skips leading coefficients -- useful when series have known leading terms.],
  ),
  related: ("findhomcombo", "findnonhomcombo", "findlincombomodp"),
)

#func-entry(
  name: "findhomcombo",
  signature: "findhomcombo(f, L, q, n, topshift)",
  description: [
    Express $f$ as a degree-$n$ homogeneous polynomial in the basis series $L$.
    Generates all monomials of exact degree $n$ in the candidate
    series, then applies RREF over $QQ$ to find coefficients. Uses
    auto-generated `X[i]` labels. For example,
    with degree 2 and candidates $L = [f_1, f_2, f_3]$, the search space includes
    $X[1]^2$, $X[1] X[2]$, $X[1] X[3]$, $X[2]^2$, $X[2] X[3]$, $X[3]^2$.
    Prints `"NOT FOUND"` on failure.
  ],
  params: (
    ([f], [Series], [The target series to match]),
    ([L], [List of Series], [Candidate series for building monomials]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([n], [Integer], [Degree of the homogeneous polynomial]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("f := partition_gf(30): findhomcombo(f, [etaq(q, 1, 30), etaq(q, 2, 30)], q, 2, 0)",
     "polynomial expression in X[1], X[2], ..."),
  ),
  edge-cases: (
    [Prints `"NOT FOUND"` if no homogeneous combination of the given degree exists.],
    [The number of monomials grows with $binom(n + d - 1, d)$ where $n$ is the number of candidates.],
    [Uses auto-generated `X[i]` labels (1-based indexing).],
  ),
  related: ("findlincombo", "findnonhomcombo", "findhomcombomodp"),
)

#func-entry(
  name: "findnonhomcombo",
  signature: "findnonhomcombo(f, L, q, n, topshift)",
  description: [
    Express $f$ as a degree-$<= n$ polynomial in the basis series $L$.
    Includes all monomials from degree 0 (constant term) through
    degree $n$ in the candidate series, providing a more flexible search than
    `findhomcombo`. Uses auto-generated `X[i]` labels.
  ],
  params: (
    ([f], [Series], [The target series to match]),
    ([L], [List of Series], [Candidate series for building monomials]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([n], [Integer], [Maximum degree of the polynomial]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("f := partition_gf(30): findnonhomcombo(f, [etaq(q, 1, 30), etaq(q, 2, 30)], q, 3, 0)",
     "polynomial expression in X[1], X[2], ..."),
  ),
  edge-cases: (
    [Prints `"NOT FOUND"` if no polynomial combination up to the given degree exists.],
    [Includes the constant monomial (degree 0), so this subsumes linear search.],
    [Uses auto-generated `X[i]` labels (1-based indexing).],
  ),
  related: ("findlincombo", "findhomcombo"),
)

#func-entry(
  name: "findlincombomodp",
  signature: "findlincombomodp(f, L, SL, p, q, topshift)",
  description: [
    Find $f$ as a linear combination of $L$ with arithmetic performed
    modulo a prime $p$, using symbolic labels `SL` for output. Uses Fermat's
    little theorem for modular inverse ($a^(-1) equiv a^(p-2) mod p$). Note
    that $p$ comes before $q$ in the argument list. This is useful when exact
    rational arithmetic is too expensive -- for instance, when working with
    very long series where rational coefficient growth causes slowdown.
    #index[modular arithmetic]
  ],
  params: (
    ([f], [Series], [The target series]),
    ([L], [List of Series], [Candidate series to combine]),
    ([SL], [List of Symbol], [Symbolic labels for the candidates (bare symbols)]),
    ([p], [Integer], [A prime modulus]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("f := partition_gf(30): findlincombomodp(f, [distinct_parts_gf(30)], [D], 7, q, 0)",
     "linear combination with SL labels, coefficients in Z/7Z"),
  ),
  edge-cases: (
    [The modulus $p$ must be prime for Fermat inverse to work correctly.],
    [Results are only valid modulo $p$ -- a match mod $p$ does not guarantee an exact rational relation.],
    [Note the argument order: $p$ comes before $q$.],
  ),
  related: ("findlincombo", "findhomcombomodp"),
)

#func-entry(
  name: "findhomcombomodp",
  signature: "findhomcombomodp(f, L, p, q, n, topshift)",
  description: [
    Express $f$ as a degree-$n$ homogeneous polynomial in $L$ with
    arithmetic performed modulo a prime $p$. Combines the monomial generation
    of `findhomcombo` with the modular arithmetic of `findlincombomodp`.
    Uses auto-generated `X[i]` labels. Note that $p$ comes before $q$ in the
    argument list.
  ],
  params: (
    ([f], [Series], [The target series]),
    ([L], [List of Series], [Candidate series for building monomials]),
    ([p], [Integer], [A prime modulus]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([n], [Integer], [Degree of the homogeneous polynomial]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("f := partition_gf(30): findhomcombomodp(f, [etaq(q, 1, 30)], 5, q, 2, 0)",
     "polynomial expression in X[1], X[2], ... with coefficients mod p"),
  ),
  edge-cases: (
    [The modulus $p$ must be prime.],
    [Useful for large-scale searches where exact arithmetic is prohibitive.],
    [Note the argument order: $p$ comes before $q$.],
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
  signature: "findhom(L, q, n, topshift)",
  description: [
    Find all degree-$n$ homogeneous polynomial relations among the
    series in list $L$. Constructs all monomials of degree $n$ from
    the input series, builds a coefficient matrix, and computes its null
    space via RREF. A nonzero null-space vector corresponds to a polynomial
    identity among the series. Uses auto-generated `X[i]` labels. The series
    variable $q$ is passed explicitly.
  ],
  params: (
    ([L], [List of Series], [Series among which to find a relation]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([n], [Integer], [Degree of the homogeneous polynomial relation]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findhom([theta3(50)^2, theta2(50)^2, theta4(50)^2], q, 1, 0)",
     "1*X[1] + (-1)*X[2] + (-1)*X[3]"),
  ),
  edge-cases: (
    [Returns the null space, which may contain multiple independent relations.],
    [An empty null space means no relation of the given degree exists.],
    [Uses auto-generated `X[i]` labels (1-based indexing).],
  ),
  related: ("findnonhom", "findhommodp", "findhomcombo"),
)

#func-entry(
  name: "findnonhom",
  signature: "findnonhom(L, q, n, topshift)",
  description: [
    Find all degree-$<= n$ polynomial relations among the series in list $L$.
    Includes constant and lower-degree terms in the monomial basis, so this
    can discover relations like $f^2 + g - 3 = 0$ that `findhom` would miss.
    Uses auto-generated `X[i]` labels. The series variable $q$ is passed
    explicitly.
  ],
  params: (
    ([L], [List of Series], [Series among which to find a relation]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([n], [Integer], [Maximum degree of the polynomial relation]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findnonhom([theta3(50)^2, theta2(50)^2, theta4(50)^2], q, 2, 0)",
     "polynomial relation(s) in X[1], X[2], X[3]"),
  ),
  edge-cases: (
    [The constant term (degree 0) is always included in the monomial basis.],
    [Returns the null space of the augmented coefficient matrix.],
    [Uses auto-generated `X[i]` labels (1-based indexing).],
  ),
  related: ("findhom", "findnonhomcombo"),
)

#func-entry(
  name: "findhommodp",
  signature: "findhommodp(L, p, q, n, topshift)",
  description: [
    Find degree-$n$ homogeneous relations among the series in $L$ with
    arithmetic performed modulo a prime $p$. Useful when exact rational
    arithmetic is too expensive for the size of the series or the degree
    of the relation. Uses auto-generated `X[i]` labels. Note that $p$ comes
    before $q$ in the argument list.
  ],
  params: (
    ([L], [List of Series], [Series among which to find a relation]),
    ([p], [Integer], [A prime modulus]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([n], [Integer], [Degree of the homogeneous polynomial relation]),
    ([topshift], [Integer], [Number of leading coefficients to skip]),
  ),
  examples: (
    ("findhommodp([etaq(q, 1, 30), etaq(q, 2, 30)], 7, q, 2, 0)",
     "relation coefficients mod 7 in X[1], X[2]"),
  ),
  edge-cases: (
    [The modulus $p$ must be prime.],
    [A relation mod $p$ suggests but does not prove an exact relation over $QQ$.],
    [Note the argument order: $p$ comes before $q$.],
  ),
  related: ("findhom", "findlincombomodp"),
)

#func-entry(
  name: "findmaxind",
  signature: "findmaxind(L, T)",
  description: [
    Find a maximally independent subset of the given series via Gaussian
    elimination. Returns the 1-based indices of the pivot columns in the
    coefficient matrix, identifying which series form a basis for the span
    of the full list. This is useful for determining the dimension of a space
    of modular forms or checking whether a new series is linearly independent
    of known ones. Uses $T$ extra rows for the coefficient matrix.
  ],
  params: (
    ([L], [List of Series], [Series to test for independence]),
    ([T], [Integer], [Number of extra rows to use in the coefficient matrix]),
  ),
  examples: (
    ("findmaxind([etaq(q, 1, 20), etaq(q, 2, 20), etaq(q, 1, 20)], 0)",
     "[1, 2]"),
  ),
  edge-cases: (
    [Returns 1-based indices of the pivot columns after RREF (Garvan convention).],
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
  signature: "findprod(FL, T, M, Q)",
  description: [
    Search for linear combinations of the series in list `FL` that yield "nice"
    formal products. Tests all primitive coefficient vectors with entries in
    $[-T, T]$: for each vector $(c_1, dots, c_k)$, computes the linear
    combination $c_1 f_1 + dots + c_k f_k$ and checks whether the result
    is a nice product using `checkprod` with threshold $M$ and truncation $Q$.
    Returns a list of `[valuation, c1, c2, ...]` vectors silently (one per
    nice product found).
    #index[findprod]
    #index[product identity search]
  ],
  params: (
    ([FL], [List of Series], [Series to combine as linear combinations]),
    ([T], [Integer], [Maximum absolute value of combination coefficients]),
    ([M], [Integer], [Maximum absolute product exponent threshold for "nice"]),
    ([Q], [Integer], [Truncation order for product check]),
  ),
  examples: (
    ("e1 := etaq(q, 1, 30); e2 := etaq(q, 2, 30): findprod([e1, e2], 2, 10, 30)",
     "[[0, 1, 0], [0, 0, 1], ...] (coefficient vectors yielding nice products)"),
  ),
  edge-cases: (
    [Returns an empty list if no nice products are found within the search bounds.],
    [Search time grows exponentially with the number of series and $T$.],
    [Only tests _primitive_ coefficient vectors (GCD of entries is 1).],
    [This function has completely different semantics from the legacy `findprod` -- the parameters $T$, $M$, $Q$ replace the old `max_coeff` and `max_exp`.],
  ),
  related: ("checkprod", "findhom", "findpoly"),
)

#func-entry(
  name: "findcong",
  signature: "findcong(QS, T) or findcong(QS, T, LM) or findcong(QS, T, LM, XSET)",
  description: [
    Auto-discover congruences in the coefficients of a $q$-series. Scans all
    moduli $2 dots "LM"$ (default: $floor(sqrt(T))$) and all residue classes,
    checking whether the sifted subsequences vanish modulo small primes.
    Reports `[B, A, R]` triples meaning $p(A n + B) equiv 0 space (mod R)$.
    The optional `XSET` parameter is a list of moduli to exclude from the scan.

    This is the tool for rediscovering Ramanujan's celebrated congruences:
    $p(5n + 4) equiv 0 mod 5$, $p(7n + 5) equiv 0 mod 7$, and
    $p(11n + 6) equiv 0 mod 11$.
    #index[findcong]
    #index[Ramanujan congruences]
    #index[congruence discovery]
  ],
  params: (
    ([QS], [Series], [The generating function to analyze]),
    ([T], [Integer], [Number of terms to scan]),
    ([LM], [Integer], [Optional: maximum modulus to test (default: $floor(sqrt(T))$)]),
    ([XSET], [List of Integer], [Optional: list of moduli to exclude from the scan]),
  ),
  examples: (
    ("p := partition_gf(200): findcong(p, 200)",
     "[4, 5, 5], [5, 7, 7], [6, 11, 11]  (Ramanujan's three congruences)"),
  ),
  edge-cases: (
    [Output format is `[B, A, R]` triples: the coefficient at position $A n + B$ is $equiv 0 mod R$.],
    [The series must have enough terms ($>= T$) for the scan to be meaningful.],
    [Default `LM` is $floor(sqrt(T))$; larger values scan more moduli but take longer.],
    [`XSET` is useful for excluding known moduli to focus on new discoveries.],
    [This function has completely different semantics from the legacy `findcong(series, [moduli])`.],
  ),
  related: ("sift", "findlincombo", "checkmult"),
)

#func-entry(
  name: "findpoly",
  signature: "findpoly(x, y, q, dx, dy) or findpoly(x, y, q, dx, dy, check)",
  description: [
    Find a polynomial relation $P(X, Y) = 0$ between two series $x$ and $y$.
    Searches for a polynomial $P$ of degree at most `dx` in $X$ and
    `dy` in $Y$ whose evaluation at the two series vanishes to the
    truncation order. The series variable $q$ is now passed explicitly.
    The optional `check` argument specifies a verification order: the
    relation is verified to $O(q^"check")$.
    Uses `X` and `Y` as the polynomial variable names in output.
  ],
  params: (
    ([x], [Series], [First series (represented as $X$ in the polynomial)]),
    ([y], [Series], [Second series (represented as $Y$ in the polynomial)]),
    ([q], [Variable], [The series variable (passed explicitly)]),
    ([dx], [Integer], [Maximum degree in $X$]),
    ([dy], [Integer], [Maximum degree in $Y$]),
    ([check], [Integer], [Optional: verification order to check the relation]),
  ),
  examples: (
    ("findpoly(theta3(50)^4, theta2(50)^4 + theta4(50)^4, q, 1, 1)",
     "polynomial in X, Y (e.g., X - Y)"),
  ),
  edge-cases: (
    [Returns `null` if no polynomial relation of the given degree bounds exists.],
    [Higher degree bounds increase both the chance of finding a relation and the computation time.],
    [The optional `check` argument provides additional verification at higher order.],
    [The series variable $q$ must be passed explicitly.],
  ),
  related: ("findhom", "findnonhom", "findprod"),
)
