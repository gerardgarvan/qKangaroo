Mock Theta Functions, Appell--Lerch Sums, and Bailey Chains
===========================================================

This module provides Ramanujan's classical mock theta functions (20 functions
across third, fifth, and seventh orders), Appell--Lerch sums and universal
mock theta functions, and the Bailey chain machinery for discovering and
proving $q$-series identities.

Third-Order Mock Theta Functions
--------------------------------

The seven third-order mock theta functions from Ramanujan's last letter to
Hardy (1920). Each takes a ``QSession`` and truncation order.

.. autofunction:: q_kangaroo.mock_theta_f3

.. autofunction:: q_kangaroo.mock_theta_phi3

.. autofunction:: q_kangaroo.mock_theta_psi3

.. autofunction:: q_kangaroo.mock_theta_chi3

.. autofunction:: q_kangaroo.mock_theta_omega3

.. autofunction:: q_kangaroo.mock_theta_nu3

.. autofunction:: q_kangaroo.mock_theta_rho3

Fifth-Order Mock Theta Functions
--------------------------------

The ten fifth-order mock theta functions, including both the lower-case
and upper-case (capital) variants.

.. autofunction:: q_kangaroo.mock_theta_f0_5

.. autofunction:: q_kangaroo.mock_theta_f1_5

.. autofunction:: q_kangaroo.mock_theta_cap_f0_5

.. autofunction:: q_kangaroo.mock_theta_cap_f1_5

.. autofunction:: q_kangaroo.mock_theta_phi0_5

.. autofunction:: q_kangaroo.mock_theta_phi1_5

.. autofunction:: q_kangaroo.mock_theta_psi0_5

.. autofunction:: q_kangaroo.mock_theta_psi1_5

.. autofunction:: q_kangaroo.mock_theta_chi0_5

.. autofunction:: q_kangaroo.mock_theta_chi1_5

Seventh-Order Mock Theta Functions
----------------------------------

Three seventh-order mock theta functions.

.. autofunction:: q_kangaroo.mock_theta_cap_f0_7

.. autofunction:: q_kangaroo.mock_theta_cap_f1_7

.. autofunction:: q_kangaroo.mock_theta_cap_f2_7

Appell--Lerch Sums
------------------

The Appell--Lerch sum $m(a,q,z)$ and Zwegers' universal mock theta
functions $g_2$ and $g_3$, which provide the modular completion
framework for mock theta functions.

.. autofunction:: q_kangaroo.appell_lerch_m

.. autofunction:: q_kangaroo.universal_mock_theta_g2

.. autofunction:: q_kangaroo.universal_mock_theta_g3

Bailey Chain Machinery
----------------------

The Bailey lemma and Bailey chain provide a systematic method for
generating and proving $q$-series identities. Starting from a base
Bailey pair $(\alpha_n, \beta_n)$, the lemma produces a new pair, and
iterating yields a chain of increasingly complex identities.

.. autofunction:: q_kangaroo.bailey_weak_lemma

.. autofunction:: q_kangaroo.bailey_apply_lemma

.. autofunction:: q_kangaroo.bailey_chain

.. autofunction:: q_kangaroo.bailey_discover
