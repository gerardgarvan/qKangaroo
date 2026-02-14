"""Q-Symbolic: Symbolic computation for q-series.

QSession provides two modes for generating q-series:

1. Individual DSL functions (e.g., etaq, partition_gf, aqprod, ...) for one-off
   computation of specific series.

2. QSession.batch_generate() / QSession.generate() for systematic parameter scans.
   These support GENERATOR functions only:
     aqprod, etaq, jacprod, tripleprod, quinprod, theta2, theta3, theta4,
     partition_gf, distinct_parts_gf, odd_parts_gf, bounded_parts_gf,
     rank_gf, crank_gf, qbin

   For analysis (prodmake, qfactor) and relation discovery (findlincombo, etc.),
   use the individual DSL functions directly.
"""

import os
import sys

# On Windows, Python 3.8+ requires explicit DLL directory registration
# for native extensions that depend on shared libraries (e.g., GMP via MinGW).
if sys.platform == "win32":
    _mingw_dir = os.environ.get("MINGW_BIN", r"C:\mingw64-gcc\mingw64\bin")
    if os.path.isdir(_mingw_dir):
        os.add_dll_directory(_mingw_dir)

from qsymbolic._qsymbolic import QSession, QExpr, QSeries, version

# DSL functions: q-Pochhammer, q-binomial, named products, theta, partitions,
# rank/crank, factoring, utilities, prodmake, and relation discovery.
from qsymbolic._qsymbolic import (
    # Group 1: Pochhammer and q-Binomial
    aqprod, qbin,
    # Group 2: Named Products
    etaq, jacprod, tripleprod, quinprod, winquist,
    # Group 3: Theta Functions
    theta2, theta3, theta4,
    # Group 4: Partition Functions
    partition_count, partition_gf, distinct_parts_gf, odd_parts_gf,
    bounded_parts_gf, rank_gf, crank_gf,
    # Group 5: Factoring, Utilities, Prodmake
    qfactor, sift, qdegree, lqdegree,
    prodmake, etamake, jacprodmake, mprodmake, qetamake,
    # Group 6: Relation Discovery (exact rational)
    findlincombo, findhom, findpoly, findcong, findnonhom,
    findhomcombo, findnonhomcombo,
    # Group 7: Relation Discovery (modular and structural)
    findlincombomodp, findhommodp, findhomcombomodp, findmaxind, findprod,
    # Group 8: Hypergeometric Series
    phi, psi, try_summation, heine1, heine2, heine3,
    # Group 9: Identity Proving
    prove_eta_id, search_identities,
    # Group 10: Mock Theta, Appell-Lerch, Bailey
    mock_theta_f3, mock_theta_phi3, mock_theta_psi3, mock_theta_chi3,
    mock_theta_omega3, mock_theta_nu3, mock_theta_rho3,
    mock_theta_f0_5, mock_theta_f1_5, mock_theta_cap_f0_5, mock_theta_cap_f1_5,
    mock_theta_phi0_5, mock_theta_phi1_5, mock_theta_psi0_5, mock_theta_psi1_5,
    mock_theta_chi0_5, mock_theta_chi1_5,
    mock_theta_cap_f0_7, mock_theta_cap_f1_7, mock_theta_cap_f2_7,
    appell_lerch_m, universal_mock_theta_g2, universal_mock_theta_g3,
    bailey_weak_lemma, bailey_apply_lemma, bailey_chain, bailey_discover,
)

__version__ = version()


def symbols(names: str, session=None):
    """Create symbols from a whitespace-separated string.

    If no session is provided, creates a new one.
    Returns a tuple of QExpr for multiple names, or a single QExpr for one name.

    Examples:
        q, a = symbols("q a")
        x = symbols("x")
    """
    if session is None:
        session = QSession()
    result = session.symbols(names)
    if len(result) == 1:
        return result[0]
    return tuple(result)


__all__ = [
    # Classes
    "QSession", "QExpr", "QSeries",
    # Helper
    "symbols",
    # Version
    "__version__",
    # Group 1: Pochhammer and q-Binomial
    "aqprod", "qbin",
    # Group 2: Named Products
    "etaq", "jacprod", "tripleprod", "quinprod", "winquist",
    # Group 3: Theta Functions
    "theta2", "theta3", "theta4",
    # Group 4: Partition Functions
    "partition_count", "partition_gf", "distinct_parts_gf", "odd_parts_gf",
    "bounded_parts_gf", "rank_gf", "crank_gf",
    # Group 5: Factoring, Utilities, Prodmake
    "qfactor", "sift", "qdegree", "lqdegree",
    "prodmake", "etamake", "jacprodmake", "mprodmake", "qetamake",
    # Group 6: Relation Discovery (exact rational)
    "findlincombo", "findhom", "findpoly", "findcong", "findnonhom",
    "findhomcombo", "findnonhomcombo",
    # Group 7: Relation Discovery (modular and structural)
    "findlincombomodp", "findhommodp", "findhomcombomodp", "findmaxind", "findprod",
    # Group 8: Hypergeometric Series
    "phi", "psi", "try_summation", "heine1", "heine2", "heine3",
    # Group 9: Identity Proving
    "prove_eta_id", "search_identities",
    # Group 10: Mock Theta, Appell-Lerch, Bailey
    "mock_theta_f3", "mock_theta_phi3", "mock_theta_psi3", "mock_theta_chi3",
    "mock_theta_omega3", "mock_theta_nu3", "mock_theta_rho3",
    "mock_theta_f0_5", "mock_theta_f1_5", "mock_theta_cap_f0_5", "mock_theta_cap_f1_5",
    "mock_theta_phi0_5", "mock_theta_phi1_5", "mock_theta_psi0_5", "mock_theta_psi1_5",
    "mock_theta_chi0_5", "mock_theta_chi1_5",
    "mock_theta_cap_f0_7", "mock_theta_cap_f1_7", "mock_theta_cap_f2_7",
    "appell_lerch_m", "universal_mock_theta_g2", "universal_mock_theta_g3",
    "bailey_weak_lemma", "bailey_apply_lemma", "bailey_chain_fn", "bailey_discover_fn",
]
