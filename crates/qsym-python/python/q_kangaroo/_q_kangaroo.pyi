"""Type stubs for the q-Kangaroo native module (_q_kangaroo).

Auto-generated from Rust source: session.rs, expr.rs, series.rs, dsl.rs, lib.rs.
"""

from fractions import Fraction
from typing import Optional

# ===========================================================================
# Classes
# ===========================================================================

class QSession:
    """A symbolic computation session owning an expression arena."""

    def __init__(self) -> None: ...
    def symbol(self, name: str) -> QExpr:
        """Intern a symbol by name, returning a QExpr handle."""
        ...
    def symbols(self, names: str) -> list[QExpr]:
        """Intern multiple symbols from a whitespace-separated string."""
        ...
    def integer(self, val: int) -> QExpr:
        """Create an integer literal expression."""
        ...
    def rational(self, num: int, den: int) -> QExpr:
        """Create a rational literal expression (num/den)."""
        ...
    def infinity(self) -> QExpr:
        """Create an infinity expression."""
        ...
    def stats(self) -> tuple[int, int]:
        """Return (arena_size, symbol_count) for diagnostics."""
        ...
    def generate(self, func_name: str, params: list[int], truncation_order: int) -> QSeries:
        """Generate a single q-series from a named generator function."""
        ...
    def batch_generate(self, func_name: str, param_grid: list[list[int]], truncation_order: int) -> list[tuple[list[int], QSeries]]:
        """Batch parameter search: generate q-series over a parameter grid."""
        ...

class QExpr:
    """A handle to a symbolic expression within a QSession."""

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def _repr_latex_(self) -> str:
        """LaTeX representation for Jupyter notebooks."""
        ...
    def latex(self) -> str:
        """LaTeX string without dollar-sign wrappers."""
        ...
    def __add__(self, other: QExpr) -> QExpr: ...
    def __radd__(self, other: QExpr) -> QExpr: ...
    def __mul__(self, other: QExpr) -> QExpr: ...
    def __rmul__(self, other: QExpr) -> QExpr: ...
    def __neg__(self) -> QExpr: ...
    def __sub__(self, other: QExpr) -> QExpr: ...
    def __pow__(self, exp: QExpr, modulo: object = ...) -> QExpr: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def simplify(self) -> QExpr:
        """Apply the simplification engine to this expression."""
        ...
    def variant(self) -> str:
        """Return the variant name of the underlying expression."""
        ...

class QSeries:
    """A q-series (formal power series) with sparse rational coefficients."""

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def _repr_latex_(self) -> str:
        """LaTeX representation for Jupyter notebooks."""
        ...
    def latex(self) -> str:
        """LaTeX string without dollar-sign wrappers."""
        ...
    def __getitem__(self, key: int) -> Fraction:
        """Get the coefficient at power key, returned as a Fraction."""
        ...
    def __len__(self) -> int:
        """Number of nonzero coefficients stored."""
        ...
    def truncation_order(self) -> int:
        """The truncation order N: series is known exactly for exponents < N."""
        ...
    def min_order(self) -> Optional[int]:
        """Lowest power with nonzero coefficient, or None if zero series."""
        ...
    def is_zero(self) -> bool:
        """True if all coefficients are zero."""
        ...
    def coeffs(self) -> list[tuple[int, Fraction]]:
        """Iterate over nonzero coefficients as (power, Fraction) tuples."""
        ...
    def to_dict(self) -> dict[int, Fraction]:
        """Return a dict mapping power to Fraction for nonzero coefficients."""
        ...
    def degree(self) -> Optional[int]:
        """Highest nonzero exponent (degree), or None if zero series."""
        ...
    def low_degree(self) -> Optional[int]:
        """Lowest nonzero exponent (valuation), or None if zero series."""
        ...
    def __add__(self, other: QSeries) -> QSeries: ...
    def __mul__(self, other: QSeries) -> QSeries: ...
    def __neg__(self) -> QSeries: ...
    def __sub__(self, other: QSeries) -> QSeries: ...
    def invert(self) -> QSeries:
        """Multiplicative inverse: 1 / self."""
        ...
    def sift(self, m: int, j: int) -> QSeries:
        """Extract arithmetic subsequence: g[i] = self[m*i + j]."""
        ...

# ===========================================================================
# Module-level functions
# ===========================================================================

def version() -> str:
    """Return the q-Kangaroo version string."""
    ...

# ===========================================================================
# Group 1: Pochhammer and q-Binomial
# ===========================================================================

def aqprod(session: QSession, coeff_num: int, coeff_den: int, power: int, n: Optional[int], order: int) -> QSeries:
    """Compute the general q-Pochhammer symbol (a; q)_n as a formal power series."""
    ...

def qbin(session: QSession, n: int, k: int, order: int) -> QSeries:
    """Compute the q-binomial (Gaussian) coefficient [n choose k]_q."""
    ...

# ===========================================================================
# Group 2: Named Products
# ===========================================================================

def etaq(session: QSession, b: int, t: int, order: int) -> QSeries:
    """Compute the generalized eta product: (q^b; q^t)_inf."""
    ...

def jacprod(session: QSession, a: int, b: int, order: int) -> QSeries:
    """Compute the Jacobi triple product JAC(a, b)."""
    ...

def tripleprod(session: QSession, coeff_num: int, coeff_den: int, power: int, order: int) -> QSeries:
    """Compute the Jacobi triple product with monomial parameter z."""
    ...

def quinprod(session: QSession, coeff_num: int, coeff_den: int, power: int, order: int) -> QSeries:
    """Compute the quintuple product with monomial parameter z."""
    ...

def winquist(session: QSession, a_cn: int, a_cd: int, a_p: int, b_cn: int, b_cd: int, b_p: int, order: int) -> QSeries:
    """Compute Winquist's identity product with parameters a and b."""
    ...

# ===========================================================================
# Group 3: Theta Functions
# ===========================================================================

def theta2(session: QSession, order: int) -> QSeries:
    """Compute the Jacobi theta function theta2(q) in q^{1/4} convention."""
    ...

def theta3(session: QSession, order: int) -> QSeries:
    """Compute the Jacobi theta function theta3(q)."""
    ...

def theta4(session: QSession, order: int) -> QSeries:
    """Compute the Jacobi theta function theta4(q)."""
    ...

# ===========================================================================
# Group 4: Partition Functions
# ===========================================================================

def partition_count(n: int) -> int:
    """Compute p(n), the number of partitions of n."""
    ...

def partition_gf(session: QSession, order: int) -> QSeries:
    """Compute the partition generating function: 1/(q;q)_inf."""
    ...

def distinct_parts_gf(session: QSession, order: int) -> QSeries:
    """Generating function for partitions into distinct parts."""
    ...

def odd_parts_gf(session: QSession, order: int) -> QSeries:
    """Generating function for partitions into odd parts."""
    ...

def bounded_parts_gf(session: QSession, max_part: int, order: int) -> QSeries:
    """Generating function for partitions with at most max_part parts."""
    ...

def rank_gf(session: QSession, z_num: int, z_den: int, order: int) -> QSeries:
    """Compute the rank generating function R(z, q)."""
    ...

def crank_gf(session: QSession, z_num: int, z_den: int, order: int) -> QSeries:
    """Compute the crank generating function C(z, q)."""
    ...

# ===========================================================================
# Group 5: Factoring, Utilities, and Prodmake/Post-processing
# ===========================================================================

def qfactor(series: QSeries) -> dict[str, object]:
    """Factor a q-polynomial into (1-q^i) components."""
    ...

def sift(series: QSeries, m: int, j: int) -> QSeries:
    """Extract arithmetic subsequence: g[i] = series[m*i + j]."""
    ...

def qdegree(series: QSeries) -> Optional[int]:
    """Highest nonzero exponent (degree) of a series."""
    ...

def lqdegree(series: QSeries) -> Optional[int]:
    """Lowest nonzero exponent (low degree / valuation) of a series."""
    ...

def prodmake(series: QSeries, max_n: int) -> dict[str, object]:
    """Recover infinite product exponents from series coefficients (Andrews' algorithm)."""
    ...

def etamake(series: QSeries, max_n: int) -> dict[str, object]:
    """Express a series as an eta-quotient."""
    ...

def jacprodmake(series: QSeries, max_n: int) -> dict[str, object]:
    """Express a series as a Jacobi product form."""
    ...

def mprodmake(series: QSeries, max_n: int) -> dict[int, int]:
    """Express a series as a product of (1+q^n) factors."""
    ...

def qetamake(series: QSeries, max_n: int) -> dict[str, object]:
    """Express a series in (q^d;q^d)_inf notation."""
    ...

# ===========================================================================
# Group 6: Relation Discovery (exact rational)
# ===========================================================================

def findlincombo(target: QSeries, candidates: list[QSeries], topshift: int) -> Optional[list[Fraction]]:
    """Find target as a linear combination of candidate series."""
    ...

def findhom(series_list: list[QSeries], degree: int, topshift: int) -> list[list[Fraction]]:
    """Find all homogeneous degree-d polynomial relations among series."""
    ...

def findpoly(x: QSeries, y: QSeries, deg_x: int, deg_y: int, topshift: int) -> Optional[dict[str, object]]:
    """Find a polynomial relation P(x, y) = 0 between two series."""
    ...

def findcong(series: QSeries, moduli: list[int]) -> list[dict[str, int]]:
    """Discover congruences among the coefficients of a series."""
    ...

def findnonhom(series_list: list[QSeries], degree: int, topshift: int) -> list[list[Fraction]]:
    """Find all non-homogeneous polynomial relations of degree <= d among series."""
    ...

def findhomcombo(target: QSeries, candidates: list[QSeries], degree: int, topshift: int) -> Optional[list[Fraction]]:
    """Express target as a homogeneous degree-d combination of basis series."""
    ...

def findnonhomcombo(target: QSeries, candidates: list[QSeries], degree: int, topshift: int) -> Optional[list[Fraction]]:
    """Express target as a non-homogeneous degree <= d combination of basis series."""
    ...

# ===========================================================================
# Group 7: Relation Discovery (modular and structural)
# ===========================================================================

def findlincombomodp(target: QSeries, candidates: list[QSeries], p: int, topshift: int) -> Optional[list[int]]:
    """Find a linear combination mod p."""
    ...

def findhommodp(series_list: list[QSeries], p: int, degree: int, topshift: int) -> list[list[int]]:
    """Find homogeneous degree-d relations mod p."""
    ...

def findhomcombomodp(target: QSeries, candidates: list[QSeries], p: int, degree: int, topshift: int) -> Optional[list[int]]:
    """Express target as a homogeneous degree-d combination mod p."""
    ...

def findmaxind(series_list: list[QSeries], topshift: int) -> list[int]:
    """Find the maximal linearly independent subset of the given series."""
    ...

def findprod(series_list: list[QSeries], max_coeff: int, max_exp: int) -> list[list[int]]:
    """Search for linear combinations of series with nice product forms."""
    ...

# ===========================================================================
# Group 8: Hypergeometric Series
# ===========================================================================

def phi(session: QSession, upper: list[tuple[int, int, int]], lower: list[tuple[int, int, int]], z_num: int, z_den: int, z_pow: int, order: int) -> QSeries:
    """Evaluate a basic hypergeometric series _r phi_s as a formal power series."""
    ...

def psi(session: QSession, upper: list[tuple[int, int, int]], lower: list[tuple[int, int, int]], z_num: int, z_den: int, z_pow: int, order: int) -> QSeries:
    """Evaluate a bilateral hypergeometric series _r psi_s."""
    ...

def try_summation(session: QSession, upper: list[tuple[int, int, int]], lower: list[tuple[int, int, int]], z_num: int, z_den: int, z_pow: int, order: int) -> Optional[QSeries]:
    """Try all summation formulas on a hypergeometric series."""
    ...

def heine1(session: QSession, upper: list[tuple[int, int, int]], lower: list[tuple[int, int, int]], z_num: int, z_den: int, z_pow: int, order: int) -> tuple[QSeries, QSeries]:
    """Apply Heine's first transformation to a 2phi1 series."""
    ...

def heine2(session: QSession, upper: list[tuple[int, int, int]], lower: list[tuple[int, int, int]], z_num: int, z_den: int, z_pow: int, order: int) -> tuple[QSeries, QSeries]:
    """Apply Heine's second transformation to a 2phi1 series."""
    ...

def heine3(session: QSession, upper: list[tuple[int, int, int]], lower: list[tuple[int, int, int]], z_num: int, z_den: int, z_pow: int, order: int) -> tuple[QSeries, QSeries]:
    """Apply Heine's third transformation to a 2phi1 series."""
    ...

# ===========================================================================
# Group 9: Identity Proving and Database
# ===========================================================================

def prove_eta_id(session: QSession, lhs_factors: list[tuple[int, int]], rhs_factors: list[tuple[int, int]], level: int) -> dict[str, object]:
    """Prove an eta-quotient identity via the valence formula."""
    ...

def search_identities(query: str, search_type: str = "pattern", db_path: Optional[str] = None) -> list[dict[str, object]]:
    """Search the identity database by tag, function, or pattern."""
    ...

# ===========================================================================
# Group 10: Mock Theta Functions, Appell-Lerch Sums & Bailey Machinery
# ===========================================================================

# 10a. Mock Theta Functions (20 functions)

# Third-order mock theta functions (7)

def mock_theta_f3(session: QSession, truncation_order: int) -> QSeries:
    """Third-order mock theta function f(q) (Ramanujan)."""
    ...

def mock_theta_phi3(session: QSession, truncation_order: int) -> QSeries:
    """Third-order mock theta function phi(q)."""
    ...

def mock_theta_psi3(session: QSession, truncation_order: int) -> QSeries:
    """Third-order mock theta function psi(q)."""
    ...

def mock_theta_chi3(session: QSession, truncation_order: int) -> QSeries:
    """Third-order mock theta function chi(q)."""
    ...

def mock_theta_omega3(session: QSession, truncation_order: int) -> QSeries:
    """Third-order mock theta function omega(q)."""
    ...

def mock_theta_nu3(session: QSession, truncation_order: int) -> QSeries:
    """Third-order mock theta function nu(q)."""
    ...

def mock_theta_rho3(session: QSession, truncation_order: int) -> QSeries:
    """Third-order mock theta function rho(q)."""
    ...

# Fifth-order mock theta functions (10)

def mock_theta_f0_5(session: QSession, truncation_order: int) -> QSeries:
    """Fifth-order mock theta function f0(q)."""
    ...

def mock_theta_f1_5(session: QSession, truncation_order: int) -> QSeries:
    """Fifth-order mock theta function f1(q)."""
    ...

def mock_theta_cap_f0_5(session: QSession, truncation_order: int) -> QSeries:
    """Fifth-order mock theta function F0(q)."""
    ...

def mock_theta_cap_f1_5(session: QSession, truncation_order: int) -> QSeries:
    """Fifth-order mock theta function F1(q)."""
    ...

def mock_theta_phi0_5(session: QSession, truncation_order: int) -> QSeries:
    """Fifth-order mock theta function phi0(q)."""
    ...

def mock_theta_phi1_5(session: QSession, truncation_order: int) -> QSeries:
    """Fifth-order mock theta function phi1(q)."""
    ...

def mock_theta_psi0_5(session: QSession, truncation_order: int) -> QSeries:
    """Fifth-order mock theta function psi0(q)."""
    ...

def mock_theta_psi1_5(session: QSession, truncation_order: int) -> QSeries:
    """Fifth-order mock theta function psi1(q)."""
    ...

def mock_theta_chi0_5(session: QSession, truncation_order: int) -> QSeries:
    """Fifth-order mock theta function chi0(q)."""
    ...

def mock_theta_chi1_5(session: QSession, truncation_order: int) -> QSeries:
    """Fifth-order mock theta function chi1(q)."""
    ...

# Seventh-order mock theta functions (3)

def mock_theta_cap_f0_7(session: QSession, truncation_order: int) -> QSeries:
    """Seventh-order mock theta function F0(q)."""
    ...

def mock_theta_cap_f1_7(session: QSession, truncation_order: int) -> QSeries:
    """Seventh-order mock theta function F1(q)."""
    ...

def mock_theta_cap_f2_7(session: QSession, truncation_order: int) -> QSeries:
    """Seventh-order mock theta function F2(q)."""
    ...

# 10b. Appell-Lerch Sums and Universal Mock Theta Functions (3 functions)

def appell_lerch_m(session: QSession, a_pow: int, z_pow: int, truncation_order: int) -> QSeries:
    """Compute the Appell-Lerch bilateral sum m(q^a, q, q^z)."""
    ...

def universal_mock_theta_g2(session: QSession, a_pow: int, truncation_order: int) -> QSeries:
    """Compute the universal mock theta function g2(q^a, q)."""
    ...

def universal_mock_theta_g3(session: QSession, a_pow: int, truncation_order: int) -> QSeries:
    """Compute the universal mock theta function g3(q^a, q)."""
    ...

# 10c. Bailey Machinery (4 functions)

def bailey_weak_lemma(session: QSession, pair_name: str, a_num: int, a_den: int, a_pow: int, max_n: int, truncation_order: int) -> tuple[QSeries, QSeries]:
    """Compute both sides of the weak Bailey lemma for a named pair."""
    ...

def bailey_apply_lemma(session: QSession, pair_name: str, a: tuple[int, int, int], b: tuple[int, int, int], c: tuple[int, int, int], max_n: int, truncation_order: int) -> dict[str, object]:
    """Apply the Bailey lemma to transform a named pair with parameters b, c."""
    ...

def bailey_chain(session: QSession, pair_name: str, a: tuple[int, int, int], b: tuple[int, int, int], c: tuple[int, int, int], depth: int, max_n: int, truncation_order: int) -> list[dict[str, object]]:
    """Apply the Bailey lemma iteratively (Bailey chain) to a named pair."""
    ...

def bailey_discover(session: QSession, lhs: QSeries, rhs: QSeries, a: tuple[int, int, int], max_chain_depth: int, truncation_order: int) -> dict[str, object]:
    """Automated Bailey pair discovery from the database."""
    ...
