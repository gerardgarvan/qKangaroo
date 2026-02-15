"""Type stubs for the q-Kangaroo package (q_kangaroo).

Re-exports all symbols from the native module _q_kangaroo,
plus the convenience ``symbols`` helper and ``__version__``.
"""

from fractions import Fraction
from typing import Optional, Union, overload

# Classes
from q_kangaroo._q_kangaroo import QSession as QSession
from q_kangaroo._q_kangaroo import QExpr as QExpr
from q_kangaroo._q_kangaroo import QSeries as QSeries

# Group 1: Pochhammer and q-Binomial
from q_kangaroo._q_kangaroo import aqprod as aqprod
from q_kangaroo._q_kangaroo import qbin as qbin

# Group 2: Named Products
from q_kangaroo._q_kangaroo import etaq as etaq
from q_kangaroo._q_kangaroo import jacprod as jacprod
from q_kangaroo._q_kangaroo import tripleprod as tripleprod
from q_kangaroo._q_kangaroo import quinprod as quinprod
from q_kangaroo._q_kangaroo import winquist as winquist

# Group 3: Theta Functions
from q_kangaroo._q_kangaroo import theta2 as theta2
from q_kangaroo._q_kangaroo import theta3 as theta3
from q_kangaroo._q_kangaroo import theta4 as theta4

# Group 4: Partition Functions
from q_kangaroo._q_kangaroo import partition_count as partition_count
from q_kangaroo._q_kangaroo import partition_gf as partition_gf
from q_kangaroo._q_kangaroo import distinct_parts_gf as distinct_parts_gf
from q_kangaroo._q_kangaroo import odd_parts_gf as odd_parts_gf
from q_kangaroo._q_kangaroo import bounded_parts_gf as bounded_parts_gf
from q_kangaroo._q_kangaroo import rank_gf as rank_gf
from q_kangaroo._q_kangaroo import crank_gf as crank_gf

# Group 5: Factoring, Utilities, Prodmake
from q_kangaroo._q_kangaroo import qfactor as qfactor
from q_kangaroo._q_kangaroo import sift as sift
from q_kangaroo._q_kangaroo import qdegree as qdegree
from q_kangaroo._q_kangaroo import lqdegree as lqdegree
from q_kangaroo._q_kangaroo import prodmake as prodmake
from q_kangaroo._q_kangaroo import etamake as etamake
from q_kangaroo._q_kangaroo import jacprodmake as jacprodmake
from q_kangaroo._q_kangaroo import mprodmake as mprodmake
from q_kangaroo._q_kangaroo import qetamake as qetamake

# Group 6: Relation Discovery (exact rational)
from q_kangaroo._q_kangaroo import findlincombo as findlincombo
from q_kangaroo._q_kangaroo import findhom as findhom
from q_kangaroo._q_kangaroo import findpoly as findpoly
from q_kangaroo._q_kangaroo import findcong as findcong
from q_kangaroo._q_kangaroo import findnonhom as findnonhom
from q_kangaroo._q_kangaroo import findhomcombo as findhomcombo
from q_kangaroo._q_kangaroo import findnonhomcombo as findnonhomcombo

# Group 7: Relation Discovery (modular and structural)
from q_kangaroo._q_kangaroo import findlincombomodp as findlincombomodp
from q_kangaroo._q_kangaroo import findhommodp as findhommodp
from q_kangaroo._q_kangaroo import findhomcombomodp as findhomcombomodp
from q_kangaroo._q_kangaroo import findmaxind as findmaxind
from q_kangaroo._q_kangaroo import findprod as findprod

# Group 8: Hypergeometric Series
from q_kangaroo._q_kangaroo import phi as phi
from q_kangaroo._q_kangaroo import psi as psi
from q_kangaroo._q_kangaroo import try_summation as try_summation
from q_kangaroo._q_kangaroo import heine1 as heine1
from q_kangaroo._q_kangaroo import heine2 as heine2
from q_kangaroo._q_kangaroo import heine3 as heine3

# Group 9: Identity Proving
from q_kangaroo._q_kangaroo import prove_eta_id as prove_eta_id
from q_kangaroo._q_kangaroo import search_identities as search_identities

# Group 10: Mock Theta, Appell-Lerch, Bailey
from q_kangaroo._q_kangaroo import mock_theta_f3 as mock_theta_f3
from q_kangaroo._q_kangaroo import mock_theta_phi3 as mock_theta_phi3
from q_kangaroo._q_kangaroo import mock_theta_psi3 as mock_theta_psi3
from q_kangaroo._q_kangaroo import mock_theta_chi3 as mock_theta_chi3
from q_kangaroo._q_kangaroo import mock_theta_omega3 as mock_theta_omega3
from q_kangaroo._q_kangaroo import mock_theta_nu3 as mock_theta_nu3
from q_kangaroo._q_kangaroo import mock_theta_rho3 as mock_theta_rho3
from q_kangaroo._q_kangaroo import mock_theta_f0_5 as mock_theta_f0_5
from q_kangaroo._q_kangaroo import mock_theta_f1_5 as mock_theta_f1_5
from q_kangaroo._q_kangaroo import mock_theta_cap_f0_5 as mock_theta_cap_f0_5
from q_kangaroo._q_kangaroo import mock_theta_cap_f1_5 as mock_theta_cap_f1_5
from q_kangaroo._q_kangaroo import mock_theta_phi0_5 as mock_theta_phi0_5
from q_kangaroo._q_kangaroo import mock_theta_phi1_5 as mock_theta_phi1_5
from q_kangaroo._q_kangaroo import mock_theta_psi0_5 as mock_theta_psi0_5
from q_kangaroo._q_kangaroo import mock_theta_psi1_5 as mock_theta_psi1_5
from q_kangaroo._q_kangaroo import mock_theta_chi0_5 as mock_theta_chi0_5
from q_kangaroo._q_kangaroo import mock_theta_chi1_5 as mock_theta_chi1_5
from q_kangaroo._q_kangaroo import mock_theta_cap_f0_7 as mock_theta_cap_f0_7
from q_kangaroo._q_kangaroo import mock_theta_cap_f1_7 as mock_theta_cap_f1_7
from q_kangaroo._q_kangaroo import mock_theta_cap_f2_7 as mock_theta_cap_f2_7
from q_kangaroo._q_kangaroo import appell_lerch_m as appell_lerch_m
from q_kangaroo._q_kangaroo import universal_mock_theta_g2 as universal_mock_theta_g2
from q_kangaroo._q_kangaroo import universal_mock_theta_g3 as universal_mock_theta_g3
from q_kangaroo._q_kangaroo import bailey_weak_lemma as bailey_weak_lemma
from q_kangaroo._q_kangaroo import bailey_apply_lemma as bailey_apply_lemma
from q_kangaroo._q_kangaroo import bailey_chain as bailey_chain
from q_kangaroo._q_kangaroo import bailey_discover as bailey_discover

# Version
__version__: str

# Convenience helper
@overload
def symbols(names: str, session: Optional[QSession] = None) -> QExpr: ...
@overload
def symbols(names: str, session: Optional[QSession] = None) -> tuple[QExpr, ...]: ...
def symbols(names: str, session: Optional[QSession] = None) -> Union[QExpr, tuple[QExpr, ...]]:
    """Create symbols from a whitespace-separated string.

    Returns a single QExpr for one name, or a tuple of QExpr for multiple names.
    """
    ...

__all__: list[str]
