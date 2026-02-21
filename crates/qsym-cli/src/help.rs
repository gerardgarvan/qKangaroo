//! Help system for the q-Kangaroo REPL.
//!
//! Provides two public functions:
//! - [`general_help`]: grouped listing of all 99 functions + 5 language
//!   constructs + session commands.
//! - [`function_help`]: per-function signature, description, and example.
//!   Also handles `for`, `proc`, `if`, `ditto`, and `lambda` language
//!   constructs via special-case match arms (bypassing FUNC_HELP).

// ---------------------------------------------------------------------------
// General help
// ---------------------------------------------------------------------------

/// Return the general help text: all functions grouped by category plus a
/// Commands section at the bottom.
pub fn general_help() -> String {
    String::from(
        "\
q-Kangaroo Functions
====================

Products:
  aqprod       - q-Pochhammer product (a;q)_n or (a;q)_inf
  qbin         - q-binomial coefficient [n choose k]_q
  etaq         - Dedekind eta quotient q^(b/24) * prod (1-q^(b*k))
  jacprod      - Jacobi triple product J(a,b)
  tripleprod   - triple product (a;q)_inf * (q/a;q)_inf * (q;q)_inf
  quinprod     - quintuple product
  winquist     - Winquist product (6 parameters)

Partitions:
  numbpart           - number of partitions p(n) or p(n,m)
  partition_gf       - partition generating function 1/(q;q)_inf
  distinct_parts_gf  - distinct parts generating function (-q;q)_inf
  odd_parts_gf       - odd parts generating function
  bounded_parts_gf   - parts <= max_part generating function
  rank_gf            - rank generating function R(z;q)
  crank_gf           - crank generating function C(z;q)

Theta Functions:
  theta    - general theta series sum(z^i*q^(i^2), i=-T..T)
  theta2   - Jacobi theta_2(q)
  theta3   - Jacobi theta_3(q)
  theta4   - Jacobi theta_4(q)

Jacobi Products:
  JAC          - Jacobi product factor (q^a;q^b)_inf
  jac2prod     - convert Jacobi product to explicit product form
  jac2series   - convert Jacobi product to q-series
  qs2jaccombo  - decompose q-series into sum of Jacobi products

Expression Operations:
  series         - truncate a series to O(q^T): series(f, q, T)
  expand         - expand products into polynomial/series form

Polynomial Operations:
  factor         - factor a polynomial in q into irreducible factors
  subs           - substitute a value for a variable: subs(q=1, f)

Simplification:
  radsimp        - simplify rational series expression

Series Analysis:
  sift           - extract arithmetic subsequence: sift(s, q, n, k, T)
  qdegree        - highest power of q with nonzero coefficient
  lqdegree       - lowest power of q with nonzero coefficient
  lqdegree0      - lowest q-degree (Garvan compat alias)
  qfactor        - factor polynomial into (1-q^i) factors
  prodmake       - find infinite product form via log derivative
  etamake        - find eta quotient form
  jacprodmake    - find Jacobi product form
  mprodmake      - find (1+q^n) product form
  qetamake       - combined eta/q-Pochhammer product form
  checkmult      - test if coefficients are multiplicative
  checkprod      - test if series is a nice formal product

Relations:
  findlincombo       - find f as linear combination of L using SL labels
  findhomcombo       - find f as degree-n homogeneous polynomial in L
  findnonhomcombo    - find f as degree-<=n polynomial in L
  findlincombomodp   - linear combination of L mod prime p, with SL labels
  findhomcombomodp   - homogeneous polynomial combo mod prime p
  findhom            - find degree-n homogeneous relations among L
  findnonhom         - find degree-<=n polynomial relations among L
  findhommodp        - homogeneous relation mod prime p
  findmaxind         - find maximally independent subset of series
  findprod           - search for product identities in series list
  findcong           - auto-discover congruences in a q-series
  findpoly           - find polynomial relation P(X,Y)=0 between two series

Hypergeometric:
  phi                        - basic hypergeometric r_phi_s series
  psi                        - bilateral hypergeometric r_psi_s series
  try_summation              - attempt closed-form summation
  heine1                     - Heine's first transformation
  heine2                     - Heine's second transformation
  heine3                     - Heine's third transformation
  sears_transform            - Sears' balanced 4_phi_3 transformation
  watson_transform           - Watson's 8_phi_7 to 4_phi_3 reduction
  find_transformation_chain  - BFS search for transformation path

Mock Theta & Bailey:
  mock_theta_f3 .. mock_theta_rho3      - 7 third-order mock theta functions
  mock_theta_f0_5 .. mock_theta_chi1_5  - 10 fifth-order mock theta functions
  mock_theta_cap_f0_7 .. cap_f2_7       - 3 seventh-order mock theta functions
  appell_lerch_m             - Appell-Lerch sum m(a,z,q)
  universal_mock_theta_g2    - universal mock theta g_2(a;q)
  universal_mock_theta_g3    - universal mock theta g_3(a;q)
  bailey_weak_lemma          - apply Bailey's weak lemma to a pair
  bailey_apply_lemma         - apply Bailey's full lemma
  bailey_chain               - iterate Bailey chain to depth d
  bailey_discover            - discover Bailey pair proving an identity

Identity Proving:
  prove_eta_id          - prove eta-quotient identity via valence formula
  search_identities     - search for identities of a given type
  q_gosper              - q-Gosper indefinite summation algorithm
  q_zeilberger          - q-Zeilberger creative telescoping
  verify_wz             - verify WZ proof certificate
  q_petkovsek           - q-Petkovsek recurrence solver
  prove_nonterminating  - nonterminating identity proof (Python API only)

Number Theory:
  floor          - floor of a rational number
  legendre       - Legendre symbol (m/p) for odd prime p
  min            - minimum of integer/rational values
  max            - maximum of integer/rational values

Scripting:
  for            - for-loop: for var from start to end [by step] do body od
  if             - conditional: if cond then body [elif ...] [else body] fi
  proc           - procedure: name := proc(params) body; end
  RETURN         - early return from procedure: RETURN(value)
  ->             - arrow / lambda: F := x -> expr

Commands:
  help [function]   - show this help or help for a specific function
  set precision N   - set default truncation order (currently: 20)
  clear             - reset all variables, %, and precision
  quit / exit       - exit the REPL (also Ctrl-D)
  latex [var]       - show LaTeX for last result or a variable
  save filename     - save last result to a file
  read filename     - load and execute a script file
  \"                - refer to the last printed result (ditto)",
    )
}

// ---------------------------------------------------------------------------
// Per-function help
// ---------------------------------------------------------------------------

/// A single help entry for a function.
struct FuncHelp {
    /// Function name (must match canonical name exactly).
    name: &'static str,
    /// Signature including parameter names.
    signature: &'static str,
    /// Description (1-3 sentences).
    description: &'static str,
    /// Example input line.
    example: &'static str,
    /// Expected output (abbreviated for series).
    example_output: &'static str,
}

/// All 99 function help entries.
const FUNC_HELP: &[FuncHelp] = &[
    // -----------------------------------------------------------------------
    // Group 1: Products (7)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "aqprod",
        signature: "aqprod(a, q, n) or aqprod(a, q, n, T) or aqprod(a, q, infinity, T)",
        description: "Compute the q-Pochhammer product (a;q)_n where a is a q-monomial, q is the variable, and n is a non-negative integer.\n  The 3-arg form aqprod(a, q, n) computes the full exact polynomial without truncation.\n  Use aqprod(a, q, n, T) for explicit truncation to O(q^T).\n  When n is 'infinity', use aqprod(a, q, infinity, T) with explicit truncation T.",
        example: "q> aqprod(q, q, 5)",
        example_output: "-q^15 + q^14 + q^13 - q^10 - q^9 - q^8 + q^7 + q^6 + q^5 - q^2 - q + 1",
    },
    FuncHelp {
        name: "qbin",
        signature: "qbin(q, m, n)",
        description: "Compute the q-binomial coefficient (Gaussian binomial) [n choose m]_q.\n  Returns an exact polynomial in q of degree m*(n-m).",
        example: "q> qbin(q, 2, 4)",
        example_output: "q^4 + q^3 + 2*q^2 + q + 1",
    },
    FuncHelp {
        name: "etaq",
        signature: "etaq(q, delta, T)",
        description: "Compute the Dedekind eta quotient (q^delta; q^delta)_inf truncated to O(q^T).\n  Also accepts a list of deltas: etaq(q, [d1, d2, ...], T) computes the product of individual eta quotients.",
        example: "q> etaq(q, 1, 10)",
        example_output: "q^7 + q^5 - q^2 - q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "jacprod",
        signature: "jacprod(a, b, q, T)",
        description: "Compute the Jacobi product JAC(a,b)/JAC(b,3b) truncated to O(q^T).\n  Parameters: a, b are positive integers, q is the variable, T is truncation order.",
        example: "q> jacprod(1, 5, q, 20)",
        example_output: "... - q^8 - q^7 + q^4 + q^3 + q^2 + q + 1 + O(q^20)",
    },
    FuncHelp {
        name: "tripleprod",
        signature: "tripleprod(z, q, T)",
        description: "Compute the Jacobi triple product (z;q)_inf * (q/z;q)_inf * (q;q)_inf truncated to O(q^T).\n  The first argument z is a q-monomial (like q^3 or -q^2).\n  When z is a symbolic variable (different from q), returns a bivariate Laurent polynomial\n  in z with q-series coefficients, computed via the sum form: sum(-1)^n z^n q^(n(n-1)/2).",
        example: "q> tripleprod(z, q, 10)",
        example_output: "... + z^2*q - z + 1 - z^(-1)*q + z^(-2)*q^3 + ... + O(q^10)",
    },
    FuncHelp {
        name: "quinprod",
        signature: "quinprod(z, q, T) or quinprod(z, q, \"prodid\") or quinprod(z, q, \"seriesid\")",
        description: "Compute the quintuple product expansion truncated to O(q^T).\n  The first argument z is a q-monomial.\n  When z is a symbolic variable (different from q), returns a bivariate Laurent polynomial\n  in z with q-series coefficients, computed via: sum(z^(3m) - z^(-3m-1)) q^(m(3m+1)/2).\n  When the third argument is the string \"prodid\", displays the quintuple product identity\n  in product form. When \"seriesid\", displays the identity in series form.",
        example: "q> quinprod(z, q, 10)",
        example_output: "... + z^3*q^2 - z^2*q + z^0 - z^(-1) - z^(-3)*q + z^(-4)*q^2 + ... + O(q^10)",
    },
    FuncHelp {
        name: "winquist",
        signature: "winquist(a, b, q, T)",
        description: "Compute the Winquist product with q-monomial parameters a, b truncated to O(q^T).\n  A product of 10 theta-type factors used in partition congruence proofs.\n  When one argument is a symbolic variable (different from q), returns a bivariate\n  Laurent polynomial in that variable with q-series coefficients.\n  When both a and b are symbolic variables (different from q), returns a trivariate\n  series: Laurent polynomial in a, b with q-series coefficients.",
        example: "q> winquist(a, b, q, 10)",
        example_output: "(trivariate Laurent polynomial in a, b with q-series coefficients)",
    },

    // -----------------------------------------------------------------------
    // Group 2: Partitions (7)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "numbpart",
        signature: "numbpart(n) or numbpart(n, m)",
        description: "Compute the number of partitions p(n). With two arguments, compute the number of partitions of n with largest part at most m.",
        example: "q> numbpart(100)",
        example_output: "190569292",
    },
    FuncHelp {
        name: "partition_gf",
        signature: "partition_gf(order)",
        description: "Compute the partition generating function 1/(q;q)_inf = sum_{n>=0} p(n)*q^n.\n  The coefficient of q^n is the number of partitions of n.",
        example: "q> partition_gf(10)",
        example_output: "30*q^9 + 22*q^8 + 15*q^7 + 11*q^6 + 7*q^5 + 5*q^4 + 3*q^3 + 2*q^2 + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "distinct_parts_gf",
        signature: "distinct_parts_gf(order)",
        description: "Compute the generating function for partitions into distinct parts, (-q;q)_inf.\n  Coefficient of q^n counts partitions of n with all parts distinct.",
        example: "q> distinct_parts_gf(10)",
        example_output: "8*q^9 + 6*q^8 + 5*q^7 + 4*q^6 + 3*q^5 + 2*q^4 + 2*q^3 + q^2 + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "odd_parts_gf",
        signature: "odd_parts_gf(order)",
        description: "Compute the generating function for partitions into odd parts.\n  Equal to 1/((q;q^2)_inf) by Euler's theorem.",
        example: "q> odd_parts_gf(10)",
        example_output: "8*q^9 + 6*q^8 + 5*q^7 + 4*q^6 + 3*q^5 + 2*q^4 + 2*q^3 + q^2 + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "bounded_parts_gf",
        signature: "bounded_parts_gf(max_part, order)",
        description: "Compute the generating function for partitions where all parts are <= max_part.\n  Equal to 1/prod_{k=1}^{max_part} (1-q^k).",
        example: "q> bounded_parts_gf(3, 10)",
        example_output: "6*q^9 + 5*q^8 + 4*q^7 + 4*q^6 + 3*q^5 + 2*q^4 + 2*q^3 + q^2 + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "rank_gf",
        signature: "rank_gf(z_num, z_den, order)",
        description: "Compute the rank generating function R(z;q) where z = z_num/z_den.\n  At z=1 this reduces to the partition generating function.",
        example: "q> rank_gf(1, 1, 10)",
        example_output: "... + 7*q^5 + 5*q^4 + 3*q^3 + 2*q^2 + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "crank_gf",
        signature: "crank_gf(z_num, z_den, order)",
        description: "Compute the crank generating function C(z;q) where z = z_num/z_den.\n  At z=1 this reduces to the partition generating function.",
        example: "q> crank_gf(1, 1, 10)",
        example_output: "... + 7*q^5 + 5*q^4 + 3*q^3 + 2*q^2 + q + 1 + O(q^10)",
    },

    // -----------------------------------------------------------------------
    // Group 3: Theta (3)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "theta2",
        signature: "theta2(T) or theta2(q, T) or theta2(a, q, T)",
        description: "Compute Jacobi theta function theta_2(q) = 2*q^(1/4) * sum_{n>=0} q^(n(n+1)).\n  Uses the q^(1/4) convention.\n  The 2-arg Garvan form theta2(q, T) specifies the variable explicitly.\n  The 3-arg form theta2(a, q, T) is the generalized Garvan convention.",
        example: "q> theta2(10)",
        example_output: "2*q^(1/4) + 2*q^(9/4) + O(q^10)",
    },
    FuncHelp {
        name: "theta3",
        signature: "theta3(T) or theta3(q, T) or theta3(a, q, T)",
        description: "Compute Jacobi theta function theta_3(q) = 1 + 2*sum_{n>=1} q^(n^2).\n  The generating function for sums of squares.\n  The 2-arg Garvan form theta3(q, T) specifies the variable explicitly.\n  The 3-arg form theta3(a, q, T) is the generalized Garvan convention.",
        example: "q> theta3(10)",
        example_output: "2*q^9 + 2*q^4 + 2*q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "theta4",
        signature: "theta4(T) or theta4(q, T) or theta4(a, q, T)",
        description: "Compute Jacobi theta function theta_4(q) = 1 + 2*sum_{n>=1} (-1)^n * q^(n^2).\n  Related to theta_3 by q -> -q.\n  The 2-arg Garvan form theta4(q, T) specifies the variable explicitly.\n  The 3-arg form theta4(a, q, T) is the generalized Garvan convention.",
        example: "q> theta4(10)",
        example_output: "-2*q^9 + 2*q^4 - 2*q + 1 + O(q^10)",
    },

    // -----------------------------------------------------------------------
    // Group 4: Series Analysis (9)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "sift",
        signature: "sift(s, q, n, k, T)",
        description: "Extract the arithmetic subsequence of coefficients at residue k mod n.\n  Returns a new series whose i-th coefficient is the (n*i+k)-th coefficient of the input.\n  T controls the truncation: the input is used up to q^T.",
        example: "q> f := partition_gf(50)\nq> sift(f, q, 5, 4, 50)",
        example_output: "... + 3*q^3 + 2*q^2 + q + 1 (coefficients of q^(5i+4) from partition_gf, descending)",
    },
    FuncHelp {
        name: "qdegree",
        signature: "qdegree(series)",
        description: "Return the highest power of q with a nonzero coefficient in the series.\n  For truncated series, this is bounded by the truncation order.",
        example: "q> qdegree(theta3(10))",
        example_output: "9",
    },
    FuncHelp {
        name: "lqdegree",
        signature: "lqdegree(series)",
        description: "Return the lowest power of q with a nonzero coefficient in the series.\n  For the zero series, returns 0.",
        example: "q> lqdegree(theta3(10))",
        example_output: "0",
    },
    FuncHelp {
        name: "lqdegree0",
        signature: "lqdegree0(f)",
        description: "Return the lowest power of q with a nonzero coefficient.\n  Equivalent to lqdegree for FPS inputs. Added for Garvan Maple compatibility.",
        example: "q> f := partition_gf(20)\nq> lqdegree0(f)",
        example_output: "0",
    },
    FuncHelp {
        name: "qfactor",
        signature: "qfactor(f, q) or qfactor(f, T) or qfactor(f, q, T)",
        description: "Factor a polynomial series into (1-q^i) factors by top-down division.\n  Returns a q-product factorization displayed as (1-q^i) factors.\n  The 2-arg form qfactor(f, T) uses implicit variable q (Garvan convention).\n  Optional T limits the maximum factor index to search.",
        example: "q> f := aqprod(q, q, 5)\nq> qfactor(f, q)",
        example_output: "(1-q)(1-q^2)(1-q^3)(1-q^4)(1-q^5)",
    },
    FuncHelp {
        name: "prodmake",
        signature: "prodmake(f, q, T)",
        description: "Find the infinite product representation of a series via the log derivative method.\n  Returns exponents a_n such that series = prod_{n>=1} (1-q^n)^{a_n}.\n  T is the maximum exponent to search.",
        example: "q> f := partition_gf(50)\nq> prodmake(f, q, 20)",
        example_output: "{exponents: {1: -1, 2: -1, 3: -1, ...}, terms_used: 20}",
    },
    FuncHelp {
        name: "etamake",
        signature: "etamake(f, q, T)",
        description: "Find an eta-quotient representation of the series via Mobius inversion.\n  Returns an eta-quotient displayed as eta(d*tau) factors.\n  T is the maximum delta to search.",
        example: "q> f := partition_gf(50)\nq> etamake(f, q, 10)",
        example_output: "eta(tau)^(-1)",
    },
    FuncHelp {
        name: "jacprodmake",
        signature: "jacprodmake(f, q, T) or jacprodmake(f, q, T, P)",
        description: "Find a Jacobi product representation with period search and residue grouping.\n  Returns JAC(a,b) factors. Optional P restricts the period search to divisors of P.\n  Includes an is_exact flag indicating whether the product matches exactly.",
        example: "q> f := jacprod(1, 5, q, 30)\nq> jacprodmake(f, q, 10)",
        example_output: "{factors: {(1,5): 1}, scalar: 1, is_exact: true}",
    },
    FuncHelp {
        name: "mprodmake",
        signature: "mprodmake(f, q, T)",
        description: "Find a (1+q^n) product representation by iterative extraction.\n  Returns exponents for each (1+q^n) factor.\n  T is the maximum exponent to search.",
        example: "q> f := distinct_parts_gf(50)\nq> mprodmake(f, q, 10)",
        example_output: "{1: 1, 2: 1, 3: 1, ...}",
    },
    FuncHelp {
        name: "qetamake",
        signature: "qetamake(f, q, T)",
        description: "Find a combined eta/q-Pochhammer product representation.\n  Extends etamake with additional q-Pochhammer factors.\n  T is the maximum exponent to search.",
        example: "q> f := partition_gf(50)\nq> qetamake(f, q, 10)",
        example_output: "{factors: {...}, q_shift: 0, is_exact: true}",
    },
    FuncHelp {
        name: "checkmult",
        signature: "checkmult(QS, T) or checkmult(QS, T, 'yes')",
        description: "Test if q-series coefficients are multiplicative: f(mn) = f(m)*f(n) for gcd(m,n)=1.\n  Checks all coprime pairs m,n with 2<=m,n<=T/2 and m*n<=T.\n  Prints MULTIPLICATIVE or NOT MULTIPLICATIVE at (m,n). Returns 1 or 0.\n  Optional 'yes' arg prints ALL failing pairs instead of stopping at first.",
        example: "q> f := partition_gf(50)\nq> checkmult(f, 30)",
        example_output: "NOT MULTIPLICATIVE at (2, 3)\n0",
    },
    FuncHelp {
        name: "checkprod",
        signature: "checkprod(f, M, Q)",
        description: "Check if series f is a nice formal product.\n  M is max absolute exponent threshold, Q is truncation order.\n  Returns [a, 1] for nice product, [a, max_exp] if not nice,\n  or [[a, c0], -1] if leading coefficient is non-integer.",
        example: "q> f := etaq(1, 1, 30)\nq> checkprod(f, 10, 30)",
        example_output: "[0, 1]",
    },

    // -----------------------------------------------------------------------
    // Group 5: Relations (12)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "findlincombo",
        signature: "findlincombo(f, L, SL, q, topshift)",
        description: "Find f as a linear combination of basis series L, printing result using symbolic labels SL.\n  Uses exact arithmetic over Q via RREF. Prints \"NOT A LINEAR COMBO.\" on failure.",
        example: "q> f := partition_gf(30)\nq> findlincombo(f, [distinct_parts_gf(30), odd_parts_gf(30)], [D, O], q, 0)",
        example_output: "1*O (partition_gf equals odd_parts_gf)",
    },
    FuncHelp {
        name: "findhomcombo",
        signature: "findhomcombo(f, L, q, n, topshift)",
        description: "Express f as a degree-n homogeneous polynomial in basis series L.\n  Uses auto-generated X[i] labels. Returns polynomial expression or \"NOT FOUND\".",
        example: "q> f := partition_gf(30)\nq> findhomcombo(f, [etaq(1, 1, 30), etaq(2, 1, 30)], q, 2, 0)",
        example_output: "polynomial expression in X[1], X[2], ...",
    },
    FuncHelp {
        name: "findnonhomcombo",
        signature: "findnonhomcombo(f, L, q, n, topshift)",
        description: "Express f as a degree-<=n polynomial in basis series L.\n  Uses auto-generated X[i] labels. Includes all monomials from degree 0 through n.",
        example: "q> f := partition_gf(30)\nq> findnonhomcombo(f, [etaq(1, 1, 30), etaq(2, 1, 30)], q, 2, 0)",
        example_output: "polynomial expression in X[1], X[2], ...",
    },
    FuncHelp {
        name: "findlincombomodp",
        signature: "findlincombomodp(f, L, SL, p, q, topshift)",
        description: "Find f as a linear combination of L mod prime p, using symbolic labels SL.\n  Note: p comes before q in the argument list. Uses Fermat inverse for modular division.",
        example: "q> f := partition_gf(30)\nq> findlincombomodp(f, [distinct_parts_gf(30)], [D], 7, q, 0)",
        example_output: "linear combination with SL labels, coefficients in Z/pZ",
    },
    FuncHelp {
        name: "findhomcombomodp",
        signature: "findhomcombomodp(f, L, p, q, n, topshift)",
        description: "Express f as degree-n homogeneous polynomial in L, mod prime p.\n  Uses auto-generated X[i] labels. Note: p comes before q.",
        example: "q> f := partition_gf(30)\nq> findhomcombomodp(f, [etaq(1, 1, 30)], 5, q, 2, 0)",
        example_output: "polynomial expression in X[1], X[2], ... with coefficients mod p",
    },
    FuncHelp {
        name: "findhom",
        signature: "findhom(L, q, n, topshift)",
        description: "Find all degree-n homogeneous polynomial relations among series in L.\n  Uses auto-generated X[i] labels. Returns the null space of the coefficient matrix.",
        example: "q> e1 := etaq(1, 1, 50)\nq> findhom([theta3(50)^2, theta2(50)^2, theta4(50)^2], q, 1, 0)",
        example_output: "polynomial relation(s) in X[1], X[2], X[3]",
    },
    FuncHelp {
        name: "findnonhom",
        signature: "findnonhom(L, q, n, topshift)",
        description: "Find all degree-<=n polynomial relations among series in L.\n  Uses auto-generated X[i] labels. Includes constant and lower-degree terms.",
        example: "q> findnonhom([theta3(50)^2, theta2(50)^2, theta4(50)^2], q, 2, 0)",
        example_output: "polynomial relation(s) in X[1], X[2], X[3]",
    },
    FuncHelp {
        name: "findhommodp",
        signature: "findhommodp(L, p, q, n, topshift)",
        description: "Find degree-n homogeneous relations mod prime p.\n  Uses auto-generated X[i] labels. Note: p comes before q.",
        example: "q> findhommodp([etaq(1, 1, 30), etaq(2, 1, 30)], 7, q, 2, 0)",
        example_output: "polynomial relation(s) with coefficients mod p",
    },
    FuncHelp {
        name: "findmaxind",
        signature: "findmaxind(L, T)",
        description: "Find maximally linearly independent subset of series L, using T extra rows.\n  Returns 1-based indices of the independent series.",
        example: "q> findmaxind([etaq(1, 1, 20), etaq(2, 1, 20), etaq(1, 1, 20)], 0)",
        example_output: "[1, 2]",
    },
    FuncHelp {
        name: "findprod",
        signature: "findprod(FL, T, M, Q)",
        description: "Search for linear combinations of series FL that yield nice products.\n  T is max |coefficient|, M is max product exponent threshold, Q is truncation order.\n  Tests all primitive coefficient vectors with entries in [-T,T].\n  Returns list of [valuation, c1, c2, ...] pairs silently.",
        example: "q> e1 := etaq(1, 1, 30); e2 := etaq(2, 1, 30)\nq> findprod([e1, e2], 2, 10, 30)",
        example_output: "[[0, 1, 0], [0, 0, 1], ...] (coefficient vectors yielding nice products)",
    },
    FuncHelp {
        name: "findcong",
        signature: "findcong(QS, T) or findcong(QS, T, LM) or findcong(QS, T, LM, XSET)",
        description: "Auto-discover congruences in series QS up to T terms.\n  Scans all moduli 2..LM (default: floor(sqrt(T))). Optional XSET excludes specific moduli.\n  Output: [B, A, R] triples where p(A*n+B) = 0 mod R.",
        example: "q> p := partition_gf(200)\nq> findcong(p, 200)",
        example_output: "[4, 5, 5] (Ramanujan's p(5n+4) = 0 mod 5)",
    },
    FuncHelp {
        name: "findpoly",
        signature: "findpoly(x, y, q, dx, dy) or findpoly(x, y, q, dx, dy, check)",
        description: "Find polynomial P(X,Y)=0 with deg(X)<=dx, deg(Y)<=dy.\n  Optional check: verify the relation to O(q^check). Uses X, Y as variable names.",
        example: "q> x := theta3(50)^4\nq> findpoly(x, theta2(50)^4, q, 2, 2)",
        example_output: "polynomial in X, Y (if relation exists)",
    },

    // -----------------------------------------------------------------------
    // Group 6: Hypergeometric (9)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "phi",
        signature: "phi(upper_list, lower_list, z_num, z_den, z_pow, order)",
        description: "Evaluate the basic hypergeometric series r_phi_s(upper; lower; q, z).\n  Parameters are lists of (num, den, pow) triples for upper and lower.",
        example: "q> phi([(1,1,1)], [(1,1,2)], 1, 1, 1, 10)",
        example_output: "... + 1 + O(q^10)",
    },
    FuncHelp {
        name: "psi",
        signature: "psi(upper_list, lower_list, z_num, z_den, z_pow, order)",
        description: "Evaluate the bilateral hypergeometric series r_psi_s(upper; lower; q, z).\n  Sums over all integers (both positive and negative indices).",
        example: "q> psi([(1,1,1)], [(1,1,2)], 1, 1, 1, 10)",
        example_output: "bilateral sum truncated to order 10",
    },
    FuncHelp {
        name: "try_summation",
        signature: "try_summation(upper_list, lower_list, z_num, z_den, z_pow, order)",
        description: "Attempt to find a closed-form summation for a basic hypergeometric series.\n  Tries q-Gauss, q-Vandermonde, q-Saalschutz, q-Kummer, and q-Dixon formulas.",
        example: "q> try_summation([(1,1,1), (1,1,2)], [(1,1,3)], 1, 1, 1, 10)",
        example_output: "closed-form product (or None if no formula applies)",
    },
    FuncHelp {
        name: "heine1",
        signature: "heine1(upper_list, lower_list, z_num, z_den, z_pow, order)",
        description: "Apply Heine's first transformation to a 2_phi_1 series.\n  Transforms 2_phi_1(a,b;c;q,z) into a product times another 2_phi_1.",
        example: "q> heine1([(1,1,1), (1,1,2)], [(1,1,3)], 1, 1, 1, 10)",
        example_output: "(prefactor, transformed_series)",
    },
    FuncHelp {
        name: "heine2",
        signature: "heine2(upper_list, lower_list, z_num, z_den, z_pow, order)",
        description: "Apply Heine's second transformation to a 2_phi_1 series.\n  A different form of Heine's transformation identity.",
        example: "q> heine2([(1,1,1), (1,1,2)], [(1,1,3)], 1, 1, 1, 10)",
        example_output: "(prefactor, transformed_series)",
    },
    FuncHelp {
        name: "heine3",
        signature: "heine3(upper_list, lower_list, z_num, z_den, z_pow, order)",
        description: "Apply Heine's third transformation to a 2_phi_1 series.\n  Transforms into a ratio of infinite products times a 2_phi_1.",
        example: "q> heine3([(1,1,1), (1,1,2)], [(1,1,3)], 1, 1, 1, 10)",
        example_output: "(prefactor, transformed_series)",
    },
    FuncHelp {
        name: "sears_transform",
        signature: "sears_transform(upper_list, lower_list, z_num, z_den, z_pow, order)",
        description: "Apply Sears' transformation to a balanced terminating 4_phi_3 series.\n  Transforms between two equivalent 4_phi_3 expressions.",
        example: "q> sears_transform(upper, lower, 1, 1, 0, 10)",
        example_output: "(prefactor, transformed_4_phi_3)",
    },
    FuncHelp {
        name: "watson_transform",
        signature: "watson_transform(upper_list, lower_list, z_num, z_den, z_pow, order)",
        description: "Apply Watson's transformation to a very-well-poised 8_phi_7 series.\n  Reduces to a balanced 4_phi_3 via the Watson-Whipple identity.",
        example: "q> watson_transform(upper, lower, 1, 1, 0, 10)",
        example_output: "(prefactor, reduced_4_phi_3)",
    },
    FuncHelp {
        name: "find_transformation_chain",
        signature: "find_transformation_chain(src_upper, src_lower, src_z_n, src_z_d, src_z_p, tgt_upper, tgt_lower, tgt_z_n, tgt_z_d, tgt_z_p, max_depth, order)",
        description: "Search for a chain of Heine/Sears/Watson transformations connecting a source\n  hypergeometric series to a target, using breadth-first search up to max_depth.",
        example: "q> find_transformation_chain(src_u, src_l, 1,1,1, tgt_u, tgt_l, 1,1,1, 3, 10)",
        example_output: "list of transformation steps (or empty if no path found)",
    },

    // -----------------------------------------------------------------------
    // Group 7: Mock Theta / Appell-Lerch / Bailey (27)
    // -----------------------------------------------------------------------
    // Third-order mock theta (7)
    FuncHelp {
        name: "mock_theta_f3",
        signature: "mock_theta_f3(order)",
        description: "Compute Ramanujan's third-order mock theta function f(q).\n  f(q) = sum_{n>=0} q^(n^2) / (-q;q)_n^2.",
        example: "q> mock_theta_f3(10)",
        example_output: "... + 3*q^3 - 2*q^2 + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_phi3",
        signature: "mock_theta_phi3(order)",
        description: "Compute Ramanujan's third-order mock theta function phi(q).\n  phi(q) = sum_{n>=0} q^(n^2) / (-q^2;q^2)_n.",
        example: "q> mock_theta_phi3(10)",
        example_output: "... + q^2 + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_psi3",
        signature: "mock_theta_psi3(order)",
        description: "Compute Ramanujan's third-order mock theta function psi(q).\n  psi(q) = sum_{n>=1} q^(n^2) / (q;q^2)_n.",
        example: "q> mock_theta_psi3(10)",
        example_output: "... + q^2 + q + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_chi3",
        signature: "mock_theta_chi3(order)",
        description: "Compute Ramanujan's third-order mock theta function chi(q).\n  chi(q) = sum_{n>=0} q^(n^2) * (-q;q)_n / prod (1-q^k+q^(2k)).",
        example: "q> mock_theta_chi3(10)",
        example_output: "... + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_omega3",
        signature: "mock_theta_omega3(order)",
        description: "Compute Ramanujan's third-order mock theta function omega(q).\n  omega(q) = sum_{n>=0} q^(2n(n+1)) / (q;q^2)_{n+1}^2.",
        example: "q> mock_theta_omega3(10)",
        example_output: "... + 2*q^2 + 1 + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_nu3",
        signature: "mock_theta_nu3(order)",
        description: "Compute Ramanujan's third-order mock theta function nu(q).\n  nu(q) = sum_{n>=0} (-1)^n * q^(n(n+1)) / (-q;q^2)_{n+1}.",
        example: "q> mock_theta_nu3(10)",
        example_output: "... - q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_rho3",
        signature: "mock_theta_rho3(order)",
        description: "Compute the third-order mock theta function rho(q).\n  rho(q) = sum_{n>=0} q^(2n(n+1)) / prod (1+q^m+q^(2m)).",
        example: "q> mock_theta_rho3(10)",
        example_output: "... + q^2 + 1 + O(q^10)",
    },
    // Fifth-order mock theta (10)
    FuncHelp {
        name: "mock_theta_f0_5",
        signature: "mock_theta_f0_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function f_0(q).\n  f_0(q) = sum_{n>=0} q^(n^2) / (-q;q)_n.",
        example: "q> mock_theta_f0_5(10)",
        example_output: "... + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_f1_5",
        signature: "mock_theta_f1_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function f_1(q).\n  f_1(q) = sum_{n>=1} q^(n^2) / (q;q)_n.",
        example: "q> mock_theta_f1_5(10)",
        example_output: "... + q^2 + q + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_cap_f0_5",
        signature: "mock_theta_cap_f0_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function F_0(q).\n  F_0(q) = sum_{n>=0} q^(2n^2) / (q;q^2)_n.",
        example: "q> mock_theta_cap_f0_5(10)",
        example_output: "... + q^2 + 1 + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_cap_f1_5",
        signature: "mock_theta_cap_f1_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function F_1(q).\n  F_1(q) = sum_{n>=1} q^(2n^2-2n+1) / (q;q^2)_n.",
        example: "q> mock_theta_cap_f1_5(10)",
        example_output: "... + q^3 + q + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_phi0_5",
        signature: "mock_theta_phi0_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function phi_0(q).\n  phi_0(q) = sum_{n>=0} q^(n^2) * (-q;q^2)_n.",
        example: "q> mock_theta_phi0_5(10)",
        example_output: "... + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_phi1_5",
        signature: "mock_theta_phi1_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function phi_1(q).\n  phi_1(q) = sum_{n>=0} q^((n+1)^2) * (-q;q^2)_n.",
        example: "q> mock_theta_phi1_5(10)",
        example_output: "... + q^2 + q + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_psi0_5",
        signature: "mock_theta_psi0_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function psi_0(q).\n  psi_0(q) = sum_{n>=0} q^((n+1)(n+2)/2) * (-q;q)_n.",
        example: "q> mock_theta_psi0_5(10)",
        example_output: "... + q^2 + q + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_psi1_5",
        signature: "mock_theta_psi1_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function psi_1(q).\n  psi_1(q) = sum_{n>=1} q^(n(n+1)/2) * (-q;q)_{n-1}.",
        example: "q> mock_theta_psi1_5(10)",
        example_output: "... + q^2 + q + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_chi0_5",
        signature: "mock_theta_chi0_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function chi_0(q).\n  chi_0(q) = sum_{n>=0} q^n * (-q;q)_{n-1} / (q^(n+1);q)_n. Uses q -> -q composition.",
        example: "q> mock_theta_chi0_5(10)",
        example_output: "... + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_chi1_5",
        signature: "mock_theta_chi1_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function chi_1(q).\n  chi_1(q) = sum_{n>=0} q^n * (-q;q)_n / (q^(n+1);q)_n. Uses q -> -q composition.",
        example: "q> mock_theta_chi1_5(10)",
        example_output: "... + q^2 + q + O(q^10)",
    },
    // Seventh-order mock theta (3)
    FuncHelp {
        name: "mock_theta_cap_f0_7",
        signature: "mock_theta_cap_f0_7(order)",
        description: "Compute the seventh-order mock theta function F_0(q).\n  F_0(q) = sum_{n>=0} q^(n^2) / (q^(n+1);q)_n.",
        example: "q> mock_theta_cap_f0_7(10)",
        example_output: "... + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_cap_f1_7",
        signature: "mock_theta_cap_f1_7(order)",
        description: "Compute the seventh-order mock theta function F_1(q).\n  F_1(q) = sum_{n>=1} q^(n^2) / (q^n;q)_n.",
        example: "q> mock_theta_cap_f1_7(10)",
        example_output: "... + q^2 + q + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_cap_f2_7",
        signature: "mock_theta_cap_f2_7(order)",
        description: "Compute the seventh-order mock theta function F_2(q).\n  F_2(q) = sum_{n>=0} q^(n(n+1)) / (q^(n+1);q)_{n+1}.",
        example: "q> mock_theta_cap_f2_7(10)",
        example_output: "... + q + 1 + O(q^10)",
    },
    // Appell-Lerch (3)
    FuncHelp {
        name: "appell_lerch_m",
        signature: "appell_lerch_m(a_pow, z_pow, order)",
        description: "Compute the Appell-Lerch sum m(a, z, q) where a = q^a_pow and z = q^z_pow.\n  m(a,z,q) = (1/j(z;q)) * sum_{r in Z} (-1)^r * z^r * q^(r(r-1)/2) / (1 - a*q^r*z).",
        example: "q> appell_lerch_m(1, 1, 10)",
        example_output: "series in q truncated to order 10",
    },
    FuncHelp {
        name: "universal_mock_theta_g2",
        signature: "universal_mock_theta_g2(a_pow, order)",
        description: "Compute the universal mock theta function g_2(a;q) where a = q^a_pow.\n  g_2 relates to the Appell-Lerch sum via algebraic identities.",
        example: "q> universal_mock_theta_g2(1, 10)",
        example_output: "series in q truncated to order 10",
    },
    FuncHelp {
        name: "universal_mock_theta_g3",
        signature: "universal_mock_theta_g3(a_pow, order)",
        description: "Compute the universal mock theta function g_3(a;q) where a = q^a_pow.\n  g_3 relates to the Appell-Lerch sum via algebraic identities.",
        example: "q> universal_mock_theta_g3(1, 10)",
        example_output: "series in q truncated to order 10",
    },
    // Bailey (4)
    FuncHelp {
        name: "bailey_weak_lemma",
        signature: "bailey_weak_lemma(pair_code, a_num, a_den, a_pow, max_n, order)",
        description: "Apply Bailey's weak lemma to a known Bailey pair. The pair_code selects the pair:\n  0 = Unit, 1 = Rogers-Ramanujan, 2 = q-Binomial.",
        example: "q> bailey_weak_lemma(1, 1, 1, 0, 10, 20)",
        example_output: "(alpha_series, beta_series)",
    },
    FuncHelp {
        name: "bailey_apply_lemma",
        signature: "bailey_apply_lemma(pair_code, a_n, a_d, a_p, b_n, b_d, b_p, c_n, c_d, c_p, max_n, order)",
        description: "Apply Bailey's full lemma to a Bailey pair, producing a new pair.\n  Parameters b and c control the transformation matrix.",
        example: "q> bailey_apply_lemma(0, 1,1,0, 1,1,1, 1,1,2, 10, 20)",
        example_output: "(new_alpha_series, new_beta_series)",
    },
    FuncHelp {
        name: "bailey_chain",
        signature: "bailey_chain(pair_code, a_n, a_d, a_p, b_n, b_d, b_p, c_n, c_d, c_p, depth, max_n, order)",
        description: "Iterate the Bailey chain to the specified depth, starting from a known pair.\n  Each iteration applies Bailey's lemma to produce a new pair.",
        example: "q> bailey_chain(1, 1,1,0, 1,1,1, 1,1,2, 3, 10, 20)",
        example_output: "(alpha_at_depth, beta_at_depth)",
    },
    FuncHelp {
        name: "bailey_discover",
        signature: "bailey_discover(lhs, rhs, a_num, a_den, a_pow, max_depth, order)",
        description: "Discover a Bailey pair that proves lhs = rhs by searching the database,\n  trying trivial equality, weak lemma matching, and chain depth search.",
        example: "q> bailey_discover(lhs_series, rhs_series, 1, 1, 0, 3, 20)",
        example_output: "proof description (or None if not found)",
    },

    // -----------------------------------------------------------------------
    // Group 8: Identity Proving (7)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "prove_eta_id",
        signature: "prove_eta_id(terms_list, level)",
        description: "Prove an eta-quotient identity using the valence formula for modular forms.\n  terms_list is a list of (eta_args, coefficient) pairs; level is the modular group level.",
        example: "q> prove_eta_id([([1,1], 1), ([1,-1], -1)], 1)",
        example_output: "true (identity proven) or false",
    },
    FuncHelp {
        name: "search_identities",
        signature: "search_identities(search_type)",
        description: "Search for identities of a given type in the database.\n  Returns known identities matching the search criteria.",
        example: "q> search_identities(\"theta\")",
        example_output: "list of matching identities",
    },
    FuncHelp {
        name: "q_gosper",
        signature: "q_gosper(upper_list, lower_list, z_num, z_den, z_pow, q_num, q_den)",
        description: "Apply the q-Gosper algorithm for indefinite q-hypergeometric summation.\n  Finds a closed form for sum of t_k as a rational function times t_k.",
        example: "q> q_gosper([(1,1,0)], [(1,1,1)], 1, 1, 0, 1, 1)",
        example_output: "closed-form antidifference (or None)",
    },
    FuncHelp {
        name: "q_zeilberger",
        signature: "q_zeilberger(upper_list, lower_list, z_num, z_den, z_pow, n, q_num, q_den, max_order)",
        description: "Apply q-Zeilberger's creative telescoping to find a recurrence for a definite sum.\n  Produces a recurrence relation in the summation index n.",
        example: "q> q_zeilberger(upper, lower, 1, 1, 0, 5, 1, 1, 3)",
        example_output: "(recurrence_coeffs, certificate)",
    },
    FuncHelp {
        name: "verify_wz",
        signature: "verify_wz(upper_list, lower_list, z_num, z_den, z_pow, n, q_num, q_den, max_order, max_k)",
        description: "Verify a WZ proof certificate. First runs q_zeilberger to produce the certificate,\n  then verifies it satisfies the WZ pair equations.",
        example: "q> verify_wz(upper, lower, 1, 1, 0, 5, 1, 1, 3, 10)",
        example_output: "true (certificate valid) or false",
    },
    FuncHelp {
        name: "q_petkovsek",
        signature: "q_petkovsek(coeff_list, q_num, q_den)",
        description: "Solve a q-holonomic recurrence using the q-Petkovsek algorithm.\n  Finds q-hypergeometric term solutions of the recurrence.",
        example: "q> q_petkovsek([1, -1, 1], 1, 1)",
        example_output: "list of q-hypergeometric solutions",
    },
    FuncHelp {
        name: "prove_nonterminating",
        signature: "prove_nonterminating(requires Python API)",
        description: "Prove a nonterminating hypergeometric identity. This function requires closure\n  support and is only available through the Python API (q_kangaroo package).",
        example: "q> prove_nonterminating(...)",
        example_output: "Error: prove_nonterminating requires the Python API",
    },
    // -----------------------------------------------------------------------
    // Group 11: Jacobi Products & Conversions (5)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "JAC",
        signature: "JAC(a, b)",
        description: "Create a Jacobi product factor (q^a;q^b)_inf. JacobiProduct values support *, /, and ^ operations.\n  Use jac2series() or jac2prod() to expand to a q-series.",
        example: "q> jp := JAC(1,5) * JAC(4,5)",
        example_output: "JAC(1,5)*JAC(4,5)",
    },
    FuncHelp {
        name: "theta",
        signature: "theta(z, q, T)",
        description: "Compute the general theta series sum(z^i * q^(i^2), i=-T..T).\n  z can be numeric (integer/rational) or a q-monomial like q^2.\n  If z is an unassigned symbol, prints a warning.",
        example: "q> theta(1, q, 5)",
        example_output: "2*q^4 + 2*q + 1 + O(q^5)",
    },
    FuncHelp {
        name: "jac2prod",
        signature: "jac2prod(JP, q, T)",
        description: "Convert a Jacobi product expression to explicit product notation.\n  JP must be a JacobiProduct value (created with JAC(a,b)).\n  Prints product notation like (1-q)(1-q^6)(1-q^11)... and returns the expanded q-series.",
        example: "q> jac2prod(JAC(1,5), q, 20)",
        example_output: "(1-q)(1-q^6)(1-q^11)(1-q^16)",
    },
    FuncHelp {
        name: "jac2series",
        signature: "jac2series(JP, T) or jac2series(JP, q, T)",
        description: "Convert a Jacobi product expression to a truncated q-series.\n  JP must be a JacobiProduct value (created with JAC(a,b)).\n  The 2-arg form jac2series(JP, T) uses the default variable q,\n  which is equivalent to jac2series(JP, q, T).\n  Prints and returns the series expansion.",
        example: "q> jac2series(JAC(1,5) * JAC(4,5), q, 20)",
        example_output: "... + q^7 - q^4 - q + 1 + O(q^20)",
    },
    FuncHelp {
        name: "qs2jaccombo",
        signature: "qs2jaccombo(f, q, T)",
        description: "Decompose a q-series into a linear combination of Jacobi products.\n  First tries single-product decomposition via jacprodmake, then tries linear combination.\n  Prints the JAC formula if found, or 'No Jacobi product decomposition found' otherwise.",
        example: "q> f := etaq(q, 1, 30): qs2jaccombo(f, q, 30)",
        example_output: "JAC(1,1)",
    },

    // -----------------------------------------------------------------------
    // Group 13: Expression Operations (2)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "series",
        signature: "series(expr, q, T)",
        description: "Truncate a q-series to O(q^T). If expr is already computed to order N, the result has order min(T, N).\n  Also accepts Jacobi products (converts then truncates) and integers/rationals (wraps as constant series).\n  The q argument is for Maple compatibility and is validated but not used.",
        example: "q> f := aqprod(q, q, infinity, 50): series(f, q, 10)",
        example_output: "-q^7 - q^5 + q^2 + q + 1 + O(q^10)",
    },
    FuncHelp {
        name: "expand",
        signature: "expand(expr) or expand(expr, q, T)",
        description: "Expand a product expression into polynomial or series form.\n  For Series values, returns unchanged. For Jacobi products, converts to q-series.\n  The 1-argument form uses the current precision; the 3-argument form uses explicit truncation order T.",
        example: "q> expand(JAC(1,5) * JAC(4,5), q, 20)",
        example_output: "... + q^7 - q^4 - q + 1 + O(q^20)",
    },

    // -----------------------------------------------------------------------
    // Group 14: Polynomial Operations (2)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "factor",
        signature: "factor(poly)",
        description: "Factor a polynomial in q into irreducible factors over the rationals.\n  The argument must be an exact polynomial (not a truncated series). Use expand() to convert products first.",
        example: "q> factor(1 - q^6)",
        example_output: "(1-q)(1+q)(1-q+q^2)(1+q+q^2)",
    },
    FuncHelp {
        name: "subs",
        signature: "subs(var=val, expr)",
        description: "Substitute a value for a variable in an expression.\n  subs(q=1, f) evaluates the series at q=1 (sums all coefficients).\n  subs(q=q^2, f) transforms exponents (multiplies all exponents by 2).\n  subs(q=r, f) evaluates the polynomial at the rational point r.\n  Also supports indexed variable substitution: subs(X[1]=q, X[2]=q^2, expr)\n  substitutes indexed variables in expressions (useful with findnonhom output).",
        example: "q> subs(q=1, 1 + q + q^2 + q^3)",
        example_output: "4",
    },

    // -----------------------------------------------------------------------
    // Group 12: Number Theory (4)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "floor",
        signature: "floor(x)",
        description: "Compute the floor (greatest integer <= x) of a number.\n  For integers, returns the input unchanged. For rationals, returns the largest integer <= x.\n  Alias: none.",
        example: "q> floor(-7/3)",
        example_output: "-3",
    },
    FuncHelp {
        name: "legendre",
        signature: "legendre(m, p)",
        description: "Compute the Legendre symbol (m/p) where p is an odd prime >= 3.\n  Returns -1, 0, or 1. Uses GMP's optimized algorithm.\n  Alias: L(m, p). Note: p is not checked for primality.",
        example: "q> legendre(2, 5)",
        example_output: "-1",
    },
    FuncHelp {
        name: "min",
        signature: "min(a, b, ...)",
        description: "Return the minimum of one or more integer or rational arguments.\n  Accepts any mix of integers and rationals. Returns the original value type (Integer stays Integer, Rational stays Rational).",
        example: "q> min(3, 1, 4, 1, 5)",
        example_output: "1",
    },
    FuncHelp {
        name: "max",
        signature: "max(a, b, ...)",
        description: "Return the maximum of one or more integer or rational arguments.\n  Accepts any mix of integers and rationals. Returns the original value type (Integer stays Integer, Rational stays Rational).",
        example: "q> max(3, 1, 4, 1, 5)",
        example_output: "5",
    },

    // -----------------------------------------------------------------------
    // Group 13: Simplification (1)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "radsimp",
        signature: "radsimp(expr)",
        description: "Simplify a rational expression involving series quotients.\n  Currently acts as the identity function, returning its input unchanged.\n  Provided for compatibility where radsimp() is used after series division.",
        example: "q> f := theta3(q, 20)^2: radsimp(f)",
        example_output: "4*q^16 + 4*q^9 + 4*q^4 + 4*q + 1 + O(q^20)",
    },

    // -----------------------------------------------------------------------
    // Group 14: Script Loading (1)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "read",
        signature: "read(\"filename.qk\")",
        description: "Load and execute a q-Kangaroo script file.\n  The file is executed as if its contents were typed at the prompt.\n  Variables and procedures defined in the script persist in the session.",
        example: "q> read(\"examples/rr.qk\")",
        example_output: "(loads and executes the script)",
    },
];

/// Return per-function help for the given name, or `None` if unrecognized.
///
/// Canonical function names are matched directly. The alias `partition_count`
/// redirects to `numbpart`.
pub fn function_help(name: &str) -> Option<String> {
    // Language construct help entries (not function calls -- bypass FUNC_HELP)
    match name {
        "for" => return Some(String::from(
            "for var from start to end [by step] do body od\n\n\
             \x20 Execute body repeatedly with var taking values start, start+step, ..., end.\n\
             \x20 Default step is 1. Body statements are separated by ; or :\n\n\
             \x20 Example:\n\
             \x20   q> s := 0: for k from 1 to 5 do s := s + k od: s\n\
             \x20   15\n\n\
             \x20 See also: if, proc"
        )),
        "proc" => return Some(String::from(
            "name := proc(params) [local vars;] [option remember;] body; end\n\n\
             \x20 Define a named procedure. Local variables are scoped to the procedure.\n\
             \x20 Use RETURN(value) for early exit. option remember caches results.\n\n\
             \x20 Example:\n\
             \x20   q> f := proc(n) local k; k := n*n; k; end: f(5)\n\
             \x20   25\n\n\
             \x20 See also: for, if, RETURN"
        )),
        "if" => return Some(String::from(
            "if cond then body [elif cond then body] [else body] fi\n\n\
             \x20 Conditional evaluation. Only the matching branch executes.\n\
             \x20 Boolean operators: and, or, not. Comparisons: =, <>, <, >, <=, >=\n\n\
             \x20 Example:\n\
             \x20   q> if 3 > 2 then 1 else 0 fi\n\
             \x20   1\n\n\
             \x20 See also: for, proc"
        )),
        "ditto" | "\"" => return Some(String::from(
            "\" (ditto operator)\n\n\
             \x20 The double-quote character \" refers to the last printed result.\n\
             \x20 It is equivalent to % but follows the q-series convention.\n\n\
             \x20 Examples:\n\
             \x20   q> aqprod(q, q, 5)\n\
             \x20   -q^15 + q^14 + q^13 - q^10 - q^9 - q^8 + q^7 + q^6 + q^5 - q^2 - q + 1\n\
             \x20   q> etamake(\", q, 20)\n\
             \x20   eta(tau)\n\n\
             \x20 Edge case: \" always refers to the most recently printed value,\n\
             \x20 even when multiple expressions are separated by semicolons.\n\n\
             \x20 See also: %"
        )),
        "lambda" | "arrow" | "->" => return Some(String::from(
            "-> (arrow / lambda operator)\n\n\
             \x20 Define an anonymous function: var -> expression\n\
             \x20 Typically used with := for named functions.\n\n\
             \x20 Examples:\n\
             \x20   q> F := q -> theta3(q, 500) / theta3(q^5, 100)\n\
             \x20   q> series(F(q), q, 20)\n\
             \x20   2*q^15 + 2*q^10 + q^5 + 2*q^4 + 2*q^3 + 2*q + 1 + O(q^20)\n\n\
             \x20   q> G := n -> n*(3*n - 1)/2\n\
             \x20   q> G(5)\n\
             \x20   35\n\n\
             \x20 The arrow desugars into a procedure with one parameter.\n\n\
             \x20 See also: proc"
        )),
        _ => {}
    }

    // Redirect aliases to canonical names
    let lookup = match name {
        "partition_count" => "numbpart",
        "L" => "legendre",
        _ => name,
    };
    FUNC_HELP.iter().find(|h| h.name == lookup).map(|h| {
        format!(
            "{}\n\n  {}\n\n  Example:\n    {}\n    {}",
            h.signature, h.description, h.example, h.example_output
        )
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_help_contains_all_categories() {
        let text = general_help();
        for category in &[
            "Products:",
            "Partitions:",
            "Theta Functions:",
            "Jacobi Products:",
            "Series Analysis:",
            "Relations:",
            "Hypergeometric:",
            "Mock Theta & Bailey:",
            "Identity Proving:",
            "Expression Operations:",
            "Polynomial Operations:",
            "Simplification:",
            "Number Theory:",
            "Scripting:",
        ] {
            assert!(
                text.contains(category),
                "general_help missing category: {}",
                category
            );
        }
    }

    #[test]
    fn general_help_contains_commands_section() {
        let text = general_help();
        assert!(text.contains("Commands:"), "general_help missing Commands section");
    }

    #[test]
    fn general_help_contains_first_and_last_functions() {
        let text = general_help();
        assert!(text.contains("aqprod"), "general_help missing aqprod");
        assert!(
            text.contains("prove_nonterminating"),
            "general_help missing prove_nonterminating"
        );
        assert!(text.contains("prove_eta_id"), "general_help missing prove_eta_id");
    }

    #[test]
    fn general_help_no_maple_aliases() {
        let text = general_help();
        // numbpart is now canonical (it SHOULD appear in general help)
        // partition_count is now the alias (it should NOT appear)
        let aliases = [
            "partition_count",
            "qphihyper",
            "qpsihyper",
            "qgauss",
            "proveid",
            "qzeil",
            "qgosper",
            "rankgf",
            "crankgf",
        ];
        for alias in &aliases {
            assert!(
                !text.contains(alias),
                "general_help should not contain alias: {}",
                alias
            );
        }
        // numbpart should appear since it's now canonical
        assert!(
            text.contains("numbpart"),
            "general_help should contain canonical name numbpart"
        );
    }

    #[test]
    fn function_help_aqprod() {
        let help = function_help("aqprod");
        assert!(help.is_some(), "aqprod should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("aqprod"), "help should contain function name");
        assert!(
            text.contains("q-Pochhammer"),
            "help should contain description"
        );
        assert!(text.contains("Example:"), "help should contain example section");
    }

    #[test]
    fn function_help_theta3() {
        let help = function_help("theta3");
        assert!(help.is_some(), "theta3 should have a help entry");
    }

    #[test]
    fn function_help_nonexistent_returns_none() {
        assert!(function_help("nonexistent").is_none());
    }

    #[test]
    fn function_help_maple_alias_returns_none() {
        // Maple aliases should NOT have help entries (except partition_count which redirects)
        assert!(function_help("qphihyper").is_none());
        assert!(function_help("qgosper").is_none());
    }

    #[test]
    fn function_help_partition_count_redirects_to_numbpart() {
        let help = function_help("partition_count");
        assert!(help.is_some(), "partition_count should redirect to numbpart");
        let text = help.unwrap();
        assert!(text.contains("numbpart"), "help should show numbpart content");
    }

    #[test]
    fn every_canonical_function_has_help_entry() {
        let canonical: Vec<&str> = vec![
            "aqprod", "qbin", "etaq", "jacprod", "tripleprod", "quinprod", "winquist",
            "numbpart", "partition_gf", "distinct_parts_gf", "odd_parts_gf",
            "bounded_parts_gf", "rank_gf", "crank_gf",
            "theta2", "theta3", "theta4",
            "sift", "qdegree", "lqdegree", "lqdegree0", "qfactor",
            "prodmake", "etamake", "jacprodmake", "mprodmake", "qetamake",
            "checkmult", "checkprod",
            "findlincombo", "findhomcombo", "findnonhomcombo",
            "findlincombomodp", "findhomcombomodp",
            "findhom", "findnonhom", "findhommodp",
            "findmaxind", "findprod", "findcong", "findpoly",
            "phi", "psi", "try_summation",
            "heine1", "heine2", "heine3",
            "sears_transform", "watson_transform", "find_transformation_chain",
            "mock_theta_f3", "mock_theta_phi3", "mock_theta_psi3",
            "mock_theta_chi3", "mock_theta_omega3", "mock_theta_nu3", "mock_theta_rho3",
            "mock_theta_f0_5", "mock_theta_f1_5",
            "mock_theta_cap_f0_5", "mock_theta_cap_f1_5",
            "mock_theta_phi0_5", "mock_theta_phi1_5",
            "mock_theta_psi0_5", "mock_theta_psi1_5",
            "mock_theta_chi0_5", "mock_theta_chi1_5",
            "mock_theta_cap_f0_7", "mock_theta_cap_f1_7", "mock_theta_cap_f2_7",
            "appell_lerch_m", "universal_mock_theta_g2", "universal_mock_theta_g3",
            "bailey_weak_lemma", "bailey_apply_lemma", "bailey_chain", "bailey_discover",
            "prove_eta_id", "search_identities",
            "q_gosper", "q_zeilberger", "verify_wz", "q_petkovsek",
            "prove_nonterminating",
            "JAC", "theta", "jac2prod", "jac2series", "qs2jaccombo",
            "series", "expand",
            "factor", "subs",
            "floor", "legendre", "min", "max",
            "radsimp", "read",
        ];
        assert_eq!(canonical.len(), 99, "test list should have 99 entries");

        for name in &canonical {
            assert!(
                function_help(name).is_some(),
                "missing help entry for canonical function: {}",
                name
            );
        }
    }

    #[test]
    fn func_help_count_matches_canonical() {
        assert_eq!(
            FUNC_HELP.len(),
            99,
            "FUNC_HELP should have exactly 99 entries, got {}",
            FUNC_HELP.len()
        );
    }

    #[test]
    fn help_shows_latex_without_coming_soon() {
        let text = general_help();
        assert!(text.contains("latex"), "help should mention latex command");
        assert!(text.contains("save"), "help should mention save command");
        assert!(
            !text.contains("coming soon"),
            "help should not contain 'coming soon'"
        );
    }

    #[test]
    fn general_help_contains_number_theory_category() {
        let text = general_help();
        assert!(text.contains("Number Theory:"), "general_help missing Number Theory category");
        assert!(text.contains("floor"), "general_help missing floor");
        assert!(text.contains("legendre"), "general_help missing legendre");
        assert!(text.contains("min"), "general_help missing min");
        assert!(text.contains("max"), "general_help missing max");
    }

    #[test]
    fn function_help_floor_returns_some() {
        let help = function_help("floor");
        assert!(help.is_some(), "floor should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("floor"), "help should contain function name");
        assert!(text.contains("greatest integer"), "help should contain description");
    }

    #[test]
    fn function_help_legendre_returns_some() {
        let help = function_help("legendre");
        assert!(help.is_some(), "legendre should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("Legendre symbol"), "help should contain description");
    }

    #[test]
    fn function_help_l_redirects_to_legendre() {
        let help = function_help("L");
        assert!(help.is_some(), "L should redirect to legendre");
        let text = help.unwrap();
        assert!(text.contains("Legendre symbol"), "help should show legendre content");
    }

    #[test]
    fn function_help_series_returns_some() {
        let help = function_help("series");
        assert!(help.is_some(), "series should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("series"), "help should contain function name");
        assert!(text.contains("Truncate"), "help should contain description");
        assert!(text.contains("O(q^T)"), "help should mention truncation");
    }

    #[test]
    fn function_help_expand_returns_some() {
        let help = function_help("expand");
        assert!(help.is_some(), "expand should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("expand"), "help should contain function name");
        assert!(text.contains("Expand"), "help should contain description");
    }

    #[test]
    fn general_help_contains_expression_operations_category() {
        let text = general_help();
        assert!(text.contains("Expression Operations:"), "general_help missing Expression Operations category");
        assert!(text.contains("series"), "general_help missing series");
        assert!(text.contains("expand"), "general_help missing expand");
    }

    #[test]
    fn general_help_contains_polynomial_operations_category() {
        let text = general_help();
        assert!(text.contains("Polynomial Operations:"), "general_help missing Polynomial Operations category");
        assert!(text.contains("factor"), "general_help missing factor");
    }

    #[test]
    fn function_help_factor_returns_some() {
        let help = function_help("factor");
        assert!(help.is_some(), "factor should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("factor"), "help should contain function name");
        assert!(text.contains("irreducible"), "help should contain description");
        assert!(text.contains("factor(1 - q^6)"), "help should contain example");
    }

    #[test]
    fn function_help_subs_returns_some() {
        let help = function_help("subs");
        assert!(help.is_some(), "subs should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("Substitute"), "help should contain description");
        assert!(text.contains("subs(q=1"), "help should contain example");
    }

    #[test]
    fn general_help_contains_subs() {
        let text = general_help();
        assert!(text.contains("subs"), "general_help should contain subs");
    }

    #[test]
    fn function_help_tripleprod_mentions_bivariate() {
        let help = function_help("tripleprod");
        assert!(help.is_some(), "tripleprod should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("symbolic") || text.contains("bivariate"),
            "tripleprod help should mention symbolic z or bivariate");
    }

    #[test]
    fn function_help_quinprod_mentions_bivariate() {
        let help = function_help("quinprod");
        assert!(help.is_some(), "quinprod should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("symbolic") || text.contains("bivariate"),
            "quinprod help should mention symbolic z or bivariate");
    }

    #[test]
    fn function_help_winquist_mentions_bivariate_and_trivariate() {
        let help = function_help("winquist");
        assert!(help.is_some(), "winquist should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("symbolic") || text.contains("bivariate"),
            "winquist help should mention symbolic variable or bivariate");
        assert!(text.contains("trivariate") || text.contains("both"),
            "winquist help should mention trivariate or both symbolic support");
    }

    #[test]
    fn function_help_for_returns_some() {
        let help = function_help("for");
        assert!(help.is_some(), "for should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("for var from"), "help should contain syntax");
        assert!(text.contains("See also:"), "help should contain cross-references");
    }

    #[test]
    fn function_help_proc_returns_some() {
        let help = function_help("proc");
        assert!(help.is_some(), "proc should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("proc"), "help should contain 'proc'");
        assert!(text.contains("RETURN"), "help should mention RETURN");
    }

    #[test]
    fn function_help_if_returns_some() {
        let help = function_help("if");
        assert!(help.is_some(), "if should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("if cond then"), "help should contain syntax");
        assert!(text.contains("See also:"), "help should contain cross-references");
    }

    #[test]
    fn general_help_contains_scripting_category() {
        let text = general_help();
        assert!(text.contains("Scripting:"), "general_help missing Scripting category");
        assert!(text.contains("RETURN"), "general_help Scripting should list RETURN");
    }

    #[test]
    fn function_help_ditto_returns_some() {
        let help = function_help("ditto");
        assert!(help.is_some(), "ditto should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("ditto"), "help should contain 'ditto'");
    }

    #[test]
    fn function_help_lambda_returns_some() {
        let help = function_help("lambda");
        assert!(help.is_some(), "lambda should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("arrow"), "help should contain 'arrow'");
    }

    #[test]
    fn function_help_radsimp_returns_some() {
        let help = function_help("radsimp");
        assert!(help.is_some(), "radsimp should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("radsimp"), "help should contain 'radsimp'");
    }

    #[test]
    fn function_help_read_returns_some() {
        let help = function_help("read");
        assert!(help.is_some(), "read should have a help entry");
        let text = help.unwrap();
        assert!(text.contains("read"), "help should contain 'read'");
    }

    #[test]
    fn general_help_contains_radsimp() {
        let text = general_help();
        assert!(text.contains("radsimp"), "general_help should contain radsimp");
    }

    #[test]
    fn general_help_contains_ditto_reference() {
        let text = general_help();
        assert!(text.contains("ditto"), "general_help should reference ditto");
    }
}
