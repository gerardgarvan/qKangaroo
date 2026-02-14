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
]
