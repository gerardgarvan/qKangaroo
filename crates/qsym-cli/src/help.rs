//! Help system for the q-Kangaroo REPL.
//!
//! Provides two public functions:
//! - [`general_help`]: grouped listing of all 81 functions + session commands.
//! - [`function_help`]: per-function signature, description, and example.

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
  partition_count    - number of partitions p(n)
  partition_gf       - partition generating function 1/(q;q)_inf
  distinct_parts_gf  - distinct parts generating function (-q;q)_inf
  odd_parts_gf       - odd parts generating function
  bounded_parts_gf   - parts <= max_part generating function
  rank_gf            - rank generating function R(z;q)
  crank_gf           - crank generating function C(z;q)

Theta Functions:
  theta2   - Jacobi theta_2(q)
  theta3   - Jacobi theta_3(q)
  theta4   - Jacobi theta_4(q)

Series Analysis:
  sift           - extract arithmetic subsequence: coeff of q^(mj+r)
  qdegree        - highest power of q with nonzero coefficient
  lqdegree       - lowest power of q with nonzero coefficient
  qfactor        - factor series into (1-q^i) factors
  prodmake       - find infinite product form via log derivative
  etamake        - find eta quotient form
  jacprodmake    - find Jacobi product form
  mprodmake      - find (1+q^n) product form
  qetamake       - combined eta/q-Pochhammer product form

Relations:
  findlincombo       - find linear combination of candidates matching target
  findhomcombo       - find homogeneous polynomial combo matching target
  findnonhomcombo    - find nonhomogeneous polynomial combo matching target
  findlincombomodp   - linear combination mod prime p
  findhomcombomodp   - homogeneous polynomial combo mod prime p
  findhom            - find homogeneous relation among series
  findnonhom         - find nonhomogeneous relation among series
  findhommodp        - homogeneous relation mod prime p
  findmaxind         - find maximally independent subset of series
  findprod           - find product identity among series
  findcong           - find partition congruences
  findpoly           - find polynomial relation between two series

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

Commands:
  help [function]   - show this help or help for a specific function
  set precision N   - set default truncation order (currently: 20)
  clear             - reset all variables, %, and precision
  quit / exit       - exit the REPL (also Ctrl-D)
  latex [expr]      - show LaTeX for last result or expression (coming soon)
  save filename     - save result to file (coming soon)",
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

/// All 81 function help entries.
const FUNC_HELP: &[FuncHelp] = &[
    // -----------------------------------------------------------------------
    // Group 1: Products (7)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "aqprod",
        signature: "aqprod(coeff_num, coeff_den, power, n_or_infinity, order)",
        description: "Compute the q-Pochhammer product (a;q)_n where a = (coeff_num/coeff_den)*q^power.\n  When n is 'infinity', computes the infinite product (a;q)_inf.",
        example: "q> aqprod(1, 1, 1, infinity, 10)",
        example_output: "1 - q - q^2 + q^5 + q^7 + O(q^10)",
    },
    FuncHelp {
        name: "qbin",
        signature: "qbin(n, k, order)",
        description: "Compute the q-binomial coefficient (Gaussian binomial) [n choose k]_q.\n  Returns a polynomial in q of degree k*(n-k).",
        example: "q> qbin(4, 2, 20)",
        example_output: "1 + q + 2*q^2 + q^3 + q^4 + O(q^20)",
    },
    FuncHelp {
        name: "etaq",
        signature: "etaq(b, t, order)",
        description: "Compute the generalized Dedekind eta quotient q^(b*t/24) * prod_{k>=1} (1 - q^(b*k))^t.\n  The parameter b is the base and t is the exponent.",
        example: "q> etaq(1, 1, 10)",
        example_output: "q^(1/24) * (1 - q - q^2 + q^5 + q^7 + ...)",
    },
    FuncHelp {
        name: "jacprod",
        signature: "jacprod(a, b, order)",
        description: "Compute the Jacobi triple product J(a,b) = prod_{k>=1} (1-q^(b*k))(1-q^(b*k-a))(1-q^(b*(k-1)+a)).\n  A fundamental building block for theta functions and partition identities.",
        example: "q> jacprod(1, 2, 10)",
        example_output: "1 - q - q^3 + q^6 + O(q^10)",
    },
    FuncHelp {
        name: "tripleprod",
        signature: "tripleprod(coeff_num, coeff_den, power, order)",
        description: "Compute the Jacobi triple product (a;q)_inf * (q/a;q)_inf * (q;q)_inf\n  where a = (coeff_num/coeff_den)*q^power.",
        example: "q> tripleprod(1, 1, 1, 10)",
        example_output: "1 - 2*q + 2*q^4 - 2*q^9 + O(q^10)",
    },
    FuncHelp {
        name: "quinprod",
        signature: "quinprod(coeff_num, coeff_den, power, order)",
        description: "Compute the quintuple product identity expansion.\n  The quintuple product is (a;q)_inf * (q/a;q)_inf * (a^2;q^2)_inf * (q^2/a^2;q^2)_inf * (q;q)_inf.",
        example: "q> quinprod(1, 1, 1, 10)",
        example_output: "1 - q - q^2 + q^5 + q^7 + O(q^10)",
    },
    FuncHelp {
        name: "winquist",
        signature: "winquist(a_cn, a_cd, a_p, b_cn, b_cd, b_p, order)",
        description: "Compute the Winquist product with parameters a = (a_cn/a_cd)*q^a_p and b = (b_cn/b_cd)*q^b_p.\n  A product of 10 theta-type factors used in partition congruence proofs.",
        example: "q> winquist(1, 1, 1, 1, 1, 2, 10)",
        example_output: "(series in q truncated to order 10)",
    },

    // -----------------------------------------------------------------------
    // Group 2: Partitions (7)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "partition_count",
        signature: "partition_count(n)",
        description: "Compute the number of partitions p(n) using the pentagonal number recurrence.\n  Returns an integer, not a series.",
        example: "q> partition_count(100)",
        example_output: "190569292",
    },
    FuncHelp {
        name: "partition_gf",
        signature: "partition_gf(order)",
        description: "Compute the partition generating function 1/(q;q)_inf = sum_{n>=0} p(n)*q^n.\n  The coefficient of q^n is the number of partitions of n.",
        example: "q> partition_gf(10)",
        example_output: "1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + 11*q^6 + 15*q^7 + 22*q^8 + 30*q^9 + O(q^10)",
    },
    FuncHelp {
        name: "distinct_parts_gf",
        signature: "distinct_parts_gf(order)",
        description: "Compute the generating function for partitions into distinct parts, (-q;q)_inf.\n  Coefficient of q^n counts partitions of n with all parts distinct.",
        example: "q> distinct_parts_gf(10)",
        example_output: "1 + q + q^2 + 2*q^3 + 2*q^4 + 3*q^5 + 4*q^6 + 5*q^7 + 6*q^8 + 8*q^9 + O(q^10)",
    },
    FuncHelp {
        name: "odd_parts_gf",
        signature: "odd_parts_gf(order)",
        description: "Compute the generating function for partitions into odd parts.\n  Equal to 1/((q;q^2)_inf) by Euler's theorem.",
        example: "q> odd_parts_gf(10)",
        example_output: "1 + q + q^2 + 2*q^3 + 2*q^4 + 3*q^5 + 4*q^6 + 5*q^7 + 6*q^8 + 8*q^9 + O(q^10)",
    },
    FuncHelp {
        name: "bounded_parts_gf",
        signature: "bounded_parts_gf(max_part, order)",
        description: "Compute the generating function for partitions where all parts are <= max_part.\n  Equal to 1/prod_{k=1}^{max_part} (1-q^k).",
        example: "q> bounded_parts_gf(3, 10)",
        example_output: "1 + q + q^2 + 2*q^3 + 2*q^4 + 3*q^5 + 4*q^6 + 4*q^7 + 5*q^8 + 6*q^9 + O(q^10)",
    },
    FuncHelp {
        name: "rank_gf",
        signature: "rank_gf(z_num, z_den, order)",
        description: "Compute the rank generating function R(z;q) where z = z_num/z_den.\n  At z=1 this reduces to the partition generating function.",
        example: "q> rank_gf(1, 1, 10)",
        example_output: "1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + ... + O(q^10)",
    },
    FuncHelp {
        name: "crank_gf",
        signature: "crank_gf(z_num, z_den, order)",
        description: "Compute the crank generating function C(z;q) where z = z_num/z_den.\n  At z=1 this reduces to the partition generating function.",
        example: "q> crank_gf(1, 1, 10)",
        example_output: "1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + ... + O(q^10)",
    },

    // -----------------------------------------------------------------------
    // Group 3: Theta (3)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "theta2",
        signature: "theta2(order)",
        description: "Compute Jacobi theta function theta_2(q) = 2*q^(1/4) * sum_{n>=0} q^(n(n+1)).\n  Uses the q^(1/4) convention.",
        example: "q> theta2(10)",
        example_output: "2*q^(1/4) + 2*q^(9/4) + O(q^10)",
    },
    FuncHelp {
        name: "theta3",
        signature: "theta3(order)",
        description: "Compute Jacobi theta function theta_3(q) = 1 + 2*sum_{n>=1} q^(n^2).\n  The generating function for sums of squares.",
        example: "q> theta3(10)",
        example_output: "1 + 2*q + 2*q^4 + 2*q^9 + O(q^10)",
    },
    FuncHelp {
        name: "theta4",
        signature: "theta4(order)",
        description: "Compute Jacobi theta function theta_4(q) = 1 + 2*sum_{n>=1} (-1)^n * q^(n^2).\n  Related to theta_3 by q -> -q.",
        example: "q> theta4(10)",
        example_output: "1 - 2*q + 2*q^4 - 2*q^9 + O(q^10)",
    },

    // -----------------------------------------------------------------------
    // Group 4: Series Analysis (9)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "sift",
        signature: "sift(series, m, j)",
        description: "Extract the arithmetic subsequence of coefficients: returns a new series whose\n  n-th coefficient is the (m*n+j)-th coefficient of the input series.",
        example: "q> sift(partition_gf(50), 5, 4)",
        example_output: "1 + q + 2*q^2 + ... (coefficients of q^(5n+4) from partition_gf)",
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
        name: "qfactor",
        signature: "qfactor(series)",
        description: "Factor a polynomial series into (1-q^i) factors by top-down division.\n  Returns a dictionary mapping each factor (1-q^i) to its multiplicity.",
        example: "q> qfactor(aqprod(1, 1, 1, 5, 20))",
        example_output: "{(1-q): 1, (1-q^2): 1, (1-q^3): 1, (1-q^4): 1, (1-q^5): 1}",
    },
    FuncHelp {
        name: "prodmake",
        signature: "prodmake(series, max_n)",
        description: "Find the infinite product representation of a series via the log derivative method.\n  Returns exponents a_n such that series = prod_{n>=1} (1-q^n)^{a_n}.",
        example: "q> prodmake(partition_gf(50), 20)",
        example_output: "{1: -1, 2: -1, 3: -1, ...}",
    },
    FuncHelp {
        name: "etamake",
        signature: "etamake(series, max_n)",
        description: "Find an eta-quotient representation of the series via Mobius inversion.\n  Returns divisor-grouped exponents for eta(d*tau) factors.",
        example: "q> etamake(partition_gf(50), 10)",
        example_output: "{1: -1} (meaning 1/eta(tau))",
    },
    FuncHelp {
        name: "jacprodmake",
        signature: "jacprodmake(series, max_n)",
        description: "Find a Jacobi product representation with period search and residue grouping.\n  Includes an is_exact flag indicating whether the product matches exactly.",
        example: "q> jacprodmake(theta3(50), 10)",
        example_output: "{period: 2, residues: {...}, is_exact: true}",
    },
    FuncHelp {
        name: "mprodmake",
        signature: "mprodmake(series, max_n)",
        description: "Find a (1+q^n) product representation by iterative extraction.\n  Converts (1+q^n) = (1-q^(2n))/(1-q^n) factors.",
        example: "q> mprodmake(distinct_parts_gf(50), 10)",
        example_output: "{1: 1, 2: 1, 3: 1, ...}",
    },
    FuncHelp {
        name: "qetamake",
        signature: "qetamake(series, max_n)",
        description: "Find a combined eta/q-Pochhammer product representation.\n  Extends etamake with additional q-Pochhammer factors.",
        example: "q> qetamake(partition_gf(50), 10)",
        example_output: "{eta_factors: {...}, qpoch_factors: {...}}",
    },

    // -----------------------------------------------------------------------
    // Group 5: Relations (12)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "findlincombo",
        signature: "findlincombo(target, [candidates], topshift)",
        description: "Find rational coefficients c_i such that target = sum c_i * candidates[i].\n  Uses exact arithmetic over Q via RREF.",
        example: "q> findlincombo(partition_gf(30), [distinct_parts_gf(30), odd_parts_gf(30)], 0)",
        example_output: "[1] (if partition_gf equals odd_parts_gf)",
    },
    FuncHelp {
        name: "findhomcombo",
        signature: "findhomcombo(target, [candidates], degree, topshift)",
        description: "Find a homogeneous polynomial combination of given degree matching the target.\n  Generates all monomials of degree d in the candidate series.",
        example: "q> findhomcombo(target, [f, g, h], 2, 0)",
        example_output: "polynomial combination coefficients",
    },
    FuncHelp {
        name: "findnonhomcombo",
        signature: "findnonhomcombo(target, [candidates], degree, topshift)",
        description: "Find a nonhomogeneous polynomial combination up to given degree matching the target.\n  Includes all monomials from degree 0 through degree d.",
        example: "q> findnonhomcombo(target, [f, g], 3, 0)",
        example_output: "polynomial combination coefficients",
    },
    FuncHelp {
        name: "findlincombomodp",
        signature: "findlincombomodp(target, [candidates], p, topshift)",
        description: "Find a linear combination matching the target, with arithmetic mod prime p.\n  Uses Fermat inverse for modular division.",
        example: "q> findlincombomodp(target, [f, g], 7, 0)",
        example_output: "coefficients in Z/pZ",
    },
    FuncHelp {
        name: "findhomcombomodp",
        signature: "findhomcombomodp(target, [candidates], p, degree, topshift)",
        description: "Find a homogeneous polynomial combination mod prime p.\n  Combines monomial generation with modular arithmetic.",
        example: "q> findhomcombomodp(target, [f, g], 5, 2, 0)",
        example_output: "polynomial combination coefficients mod 5",
    },
    FuncHelp {
        name: "findhom",
        signature: "findhom([series], degree, topshift)",
        description: "Find a homogeneous polynomial relation of given degree among the series.\n  Returns the null space of the coefficient matrix.",
        example: "q> findhom([theta3(50)^2, theta2(50)^2, theta4(50)^2], 1, 0)",
        example_output: "relation coefficients (if one exists)",
    },
    FuncHelp {
        name: "findnonhom",
        signature: "findnonhom([series], degree, topshift)",
        description: "Find a nonhomogeneous relation of given degree among the series.\n  Includes constant and lower-degree terms.",
        example: "q> findnonhom([f, g, h], 2, 0)",
        example_output: "relation coefficients",
    },
    FuncHelp {
        name: "findhommodp",
        signature: "findhommodp([series], p, degree, topshift)",
        description: "Find a homogeneous relation among series with arithmetic mod prime p.\n  Useful when exact rational arithmetic is too expensive.",
        example: "q> findhommodp([f, g, h], 7, 2, 0)",
        example_output: "relation coefficients mod 7",
    },
    FuncHelp {
        name: "findmaxind",
        signature: "findmaxind([series], topshift)",
        description: "Find a maximally independent subset of the given series via Gaussian elimination.\n  Returns the indices of the pivot columns.",
        example: "q> findmaxind([f, g, h, f+g], 0)",
        example_output: "[0, 1, 2]",
    },
    FuncHelp {
        name: "findprod",
        signature: "findprod([series], max_coeff, max_exp)",
        description: "Find a product identity among series by brute-force search over exponent combinations.\n  Tests prod series[i]^{e_i} for integer exponents in [-max_coeff, max_coeff].",
        example: "q> findprod([etaq(1,1,30), etaq(2,1,30)], 3, 2)",
        example_output: "exponent vector (if product identity exists)",
    },
    FuncHelp {
        name: "findcong",
        signature: "findcong(series, [moduli])",
        description: "Find partition-type congruences by checking whether sifted subsequences\n  vanish modulo each specified modulus.",
        example: "q> findcong(partition_gf(200), [5, 7, 11])",
        example_output: "list of (modulus, residue, prime) triples",
    },
    FuncHelp {
        name: "findpoly",
        signature: "findpoly(x, y, deg_x, deg_y, topshift)",
        description: "Find a polynomial relation P(x, y) = 0 between two series x and y.\n  Searches for P of degree deg_x in x and deg_y in y.",
        example: "q> findpoly(theta3(50)^4, theta2(50)^4, 2, 2, 0)",
        example_output: "polynomial coefficients (if relation exists)",
    },

    // -----------------------------------------------------------------------
    // Group 6: Hypergeometric (9)
    // -----------------------------------------------------------------------
    FuncHelp {
        name: "phi",
        signature: "phi(upper_list, lower_list, z_num, z_den, z_pow, order)",
        description: "Evaluate the basic hypergeometric series r_phi_s(upper; lower; q, z).\n  Parameters are lists of (num, den, pow) triples for upper and lower.",
        example: "q> phi([(1,1,1)], [(1,1,2)], 1, 1, 1, 10)",
        example_output: "1 + ... + O(q^10)",
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
        example_output: "1 + q - 2*q^2 + 3*q^3 + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_phi3",
        signature: "mock_theta_phi3(order)",
        description: "Compute Ramanujan's third-order mock theta function phi(q).\n  phi(q) = sum_{n>=0} q^(n^2) / (-q^2;q^2)_n.",
        example: "q> mock_theta_phi3(10)",
        example_output: "1 + q + q^2 + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_psi3",
        signature: "mock_theta_psi3(order)",
        description: "Compute Ramanujan's third-order mock theta function psi(q).\n  psi(q) = sum_{n>=1} q^(n^2) / (q;q^2)_n.",
        example: "q> mock_theta_psi3(10)",
        example_output: "q + q^2 + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_chi3",
        signature: "mock_theta_chi3(order)",
        description: "Compute Ramanujan's third-order mock theta function chi(q).\n  chi(q) = sum_{n>=0} q^(n^2) * (-q;q)_n / prod (1-q^k+q^(2k)).",
        example: "q> mock_theta_chi3(10)",
        example_output: "1 + q + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_omega3",
        signature: "mock_theta_omega3(order)",
        description: "Compute Ramanujan's third-order mock theta function omega(q).\n  omega(q) = sum_{n>=0} q^(2n(n+1)) / (q;q^2)_{n+1}^2.",
        example: "q> mock_theta_omega3(10)",
        example_output: "1 + 2*q^2 + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_nu3",
        signature: "mock_theta_nu3(order)",
        description: "Compute Ramanujan's third-order mock theta function nu(q).\n  nu(q) = sum_{n>=0} (-1)^n * q^(n(n+1)) / (-q;q^2)_{n+1}.",
        example: "q> mock_theta_nu3(10)",
        example_output: "1 - q + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_rho3",
        signature: "mock_theta_rho3(order)",
        description: "Compute the third-order mock theta function rho(q).\n  rho(q) = sum_{n>=0} q^(2n(n+1)) / prod (1+q^m+q^(2m)).",
        example: "q> mock_theta_rho3(10)",
        example_output: "1 + q^2 + ... + O(q^10)",
    },
    // Fifth-order mock theta (10)
    FuncHelp {
        name: "mock_theta_f0_5",
        signature: "mock_theta_f0_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function f_0(q).\n  f_0(q) = sum_{n>=0} q^(n^2) / (-q;q)_n.",
        example: "q> mock_theta_f0_5(10)",
        example_output: "1 + q + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_f1_5",
        signature: "mock_theta_f1_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function f_1(q).\n  f_1(q) = sum_{n>=1} q^(n^2) / (q;q)_n.",
        example: "q> mock_theta_f1_5(10)",
        example_output: "q + q^2 + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_cap_f0_5",
        signature: "mock_theta_cap_f0_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function F_0(q).\n  F_0(q) = sum_{n>=0} q^(2n^2) / (q;q^2)_n.",
        example: "q> mock_theta_cap_f0_5(10)",
        example_output: "1 + q^2 + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_cap_f1_5",
        signature: "mock_theta_cap_f1_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function F_1(q).\n  F_1(q) = sum_{n>=1} q^(2n^2-2n+1) / (q;q^2)_n.",
        example: "q> mock_theta_cap_f1_5(10)",
        example_output: "q + q^3 + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_phi0_5",
        signature: "mock_theta_phi0_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function phi_0(q).\n  phi_0(q) = sum_{n>=0} q^(n^2) * (-q;q^2)_n.",
        example: "q> mock_theta_phi0_5(10)",
        example_output: "1 + q + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_phi1_5",
        signature: "mock_theta_phi1_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function phi_1(q).\n  phi_1(q) = sum_{n>=0} q^((n+1)^2) * (-q;q^2)_n.",
        example: "q> mock_theta_phi1_5(10)",
        example_output: "q + q^2 + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_psi0_5",
        signature: "mock_theta_psi0_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function psi_0(q).\n  psi_0(q) = sum_{n>=0} q^((n+1)(n+2)/2) * (-q;q)_n.",
        example: "q> mock_theta_psi0_5(10)",
        example_output: "q + q^2 + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_psi1_5",
        signature: "mock_theta_psi1_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function psi_1(q).\n  psi_1(q) = sum_{n>=1} q^(n(n+1)/2) * (-q;q)_{n-1}.",
        example: "q> mock_theta_psi1_5(10)",
        example_output: "q + q^2 + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_chi0_5",
        signature: "mock_theta_chi0_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function chi_0(q).\n  chi_0(q) = sum_{n>=0} q^n * (-q;q)_{n-1} / (q^(n+1);q)_n. Uses q -> -q composition.",
        example: "q> mock_theta_chi0_5(10)",
        example_output: "1 + q + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_chi1_5",
        signature: "mock_theta_chi1_5(order)",
        description: "Compute Ramanujan's fifth-order mock theta function chi_1(q).\n  chi_1(q) = sum_{n>=0} q^n * (-q;q)_n / (q^(n+1);q)_n. Uses q -> -q composition.",
        example: "q> mock_theta_chi1_5(10)",
        example_output: "q + q^2 + ... + O(q^10)",
    },
    // Seventh-order mock theta (3)
    FuncHelp {
        name: "mock_theta_cap_f0_7",
        signature: "mock_theta_cap_f0_7(order)",
        description: "Compute the seventh-order mock theta function F_0(q).\n  F_0(q) = sum_{n>=0} q^(n^2) / (q^(n+1);q)_n.",
        example: "q> mock_theta_cap_f0_7(10)",
        example_output: "1 + q + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_cap_f1_7",
        signature: "mock_theta_cap_f1_7(order)",
        description: "Compute the seventh-order mock theta function F_1(q).\n  F_1(q) = sum_{n>=1} q^(n^2) / (q^n;q)_n.",
        example: "q> mock_theta_cap_f1_7(10)",
        example_output: "q + q^2 + ... + O(q^10)",
    },
    FuncHelp {
        name: "mock_theta_cap_f2_7",
        signature: "mock_theta_cap_f2_7(order)",
        description: "Compute the seventh-order mock theta function F_2(q).\n  F_2(q) = sum_{n>=0} q^(n(n+1)) / (q^(n+1);q)_{n+1}.",
        example: "q> mock_theta_cap_f2_7(10)",
        example_output: "1 + q + ... + O(q^10)",
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
];

/// Return per-function help for the given name, or `None` if unrecognized.
///
/// Only canonical function names are matched -- Maple aliases return `None`.
pub fn function_help(name: &str) -> Option<String> {
    FUNC_HELP.iter().find(|h| h.name == name).map(|h| {
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
            "Series Analysis:",
            "Relations:",
            "Hypergeometric:",
            "Mock Theta & Bailey:",
            "Identity Proving:",
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
        let aliases = [
            "numbpart",
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
                "general_help should not contain Maple alias: {}",
                alias
            );
        }
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
        // Maple aliases should NOT have help entries
        assert!(function_help("numbpart").is_none());
        assert!(function_help("qphihyper").is_none());
        assert!(function_help("qgosper").is_none());
    }

    #[test]
    fn every_canonical_function_has_help_entry() {
        let canonical: Vec<&str> = vec![
            "aqprod", "qbin", "etaq", "jacprod", "tripleprod", "quinprod", "winquist",
            "partition_count", "partition_gf", "distinct_parts_gf", "odd_parts_gf",
            "bounded_parts_gf", "rank_gf", "crank_gf",
            "theta2", "theta3", "theta4",
            "sift", "qdegree", "lqdegree", "qfactor",
            "prodmake", "etamake", "jacprodmake", "mprodmake", "qetamake",
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
        ];
        assert_eq!(canonical.len(), 81, "test list should have 81 entries");

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
            81,
            "FUNC_HELP should have exactly 81 entries, got {}",
            FUNC_HELP.len()
        );
    }
}
