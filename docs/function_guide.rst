Which Function Should I Use?
============================

This page organizes q-Kangaroo's 79 functions by **task** -- what you want
to accomplish -- rather than by implementation group. Find your task below,
and the guide points you to the right function with a cross-reference to the
full API documentation.

1. Building q-Series from Scratch
---------------------------------

Start here if you want to construct a specific q-series or product.

- "I want to compute a q-Pochhammer symbol" -- :func:`~q_kangaroo.aqprod` (finite or infinite)
- "I want an eta product" -- :func:`~q_kangaroo.etaq`
- "I want a Jacobi triple product" -- :func:`~q_kangaroo.jacprod`
- "I want a generic triple product" -- :func:`~q_kangaroo.tripleprod`
- "I want a quintuple product" -- :func:`~q_kangaroo.quinprod`
- "I want the Winquist product" -- :func:`~q_kangaroo.winquist`
- "I want a q-binomial coefficient" -- :func:`~q_kangaroo.qbin`
- "I want a theta function" -- :func:`~q_kangaroo.theta2`, :func:`~q_kangaroo.theta3`, :func:`~q_kangaroo.theta4`
- "I want a basic hypergeometric series" -- :func:`~q_kangaroo.phi`
- "I want a bilateral hypergeometric series" -- :func:`~q_kangaroo.psi`

2. Partition Functions
----------------------

Functions for counting partitions, generating partition series, and computing
rank and crank statistics.

- "I want to count partitions p(n)" -- :func:`~q_kangaroo.partition_count`
- "I want a partition generating function" -- :func:`~q_kangaroo.partition_gf`
- "I want distinct parts only" -- :func:`~q_kangaroo.distinct_parts_gf`
- "I want odd parts only" -- :func:`~q_kangaroo.odd_parts_gf`
- "I want bounded parts" -- :func:`~q_kangaroo.bounded_parts_gf`
- "I want the rank generating function" -- :func:`~q_kangaroo.rank_gf`
- "I want the crank generating function" -- :func:`~q_kangaroo.crank_gf`

3. Analyzing a Series (Reverse Engineering)
--------------------------------------------

Given a series, recover its product form, eta-quotient representation, or
extract subsequences.

- "I have a series and want its infinite product form" -- :func:`~q_kangaroo.prodmake`
- "I want the eta-quotient representation" -- :func:`~q_kangaroo.etamake`
- "I want the Jacobi product representation" -- :func:`~q_kangaroo.jacprodmake`
- "I want mixed-sign product form" -- :func:`~q_kangaroo.mprodmake`
- "I want generalized eta-quotient representation" -- :func:`~q_kangaroo.qetamake`
- "I want to extract an arithmetic subsequence" -- :func:`~q_kangaroo.sift`
- "I want to factor a q-polynomial" -- :func:`~q_kangaroo.qfactor`
- "I want the q-degree of a series" -- :func:`~q_kangaroo.qdegree`
- "I want the low q-degree of a series" -- :func:`~q_kangaroo.lqdegree`

4. Finding Relations Between Series
------------------------------------

Discover linear combinations, polynomial relations, congruences, and
independence among collections of q-series.

**Exact rational arithmetic:**

- "I want to express one series as a linear combination of others" -- :func:`~q_kangaroo.findlincombo`
- "I want a homogeneous polynomial relation" -- :func:`~q_kangaroo.findhom`
- "I want a general polynomial relation" -- :func:`~q_kangaroo.findpoly`
- "I want to discover partition congruences" -- :func:`~q_kangaroo.findcong`
- "I want a non-homogeneous relation" -- :func:`~q_kangaroo.findnonhom`
- "I want to combine multiple series in a homogeneous search" -- :func:`~q_kangaroo.findhomcombo`
- "I want to combine multiple series in a non-homogeneous search" -- :func:`~q_kangaroo.findnonhomcombo`

**Modular arithmetic (faster for large series):**

- "I want linear combinations mod p" -- :func:`~q_kangaroo.findlincombomodp`
- "I want homogeneous relations mod p" -- :func:`~q_kangaroo.findhommodp`
- "I want combined homogeneous relations mod p" -- :func:`~q_kangaroo.findhomcombomodp`

**Structural:**

- "I want to find independent series from a set" -- :func:`~q_kangaroo.findmaxind`
- "I want a brute-force product search" -- :func:`~q_kangaroo.findprod`

5. Proving Identities
---------------------

Algorithmic tools for proving q-series identities, from eta-quotient
verification to creative telescoping and WZ certificates.

- "I want to prove an eta-quotient identity" -- :func:`~q_kangaroo.prove_eta_id`
- "I want to search the identity database" -- :func:`~q_kangaroo.search_identities`
- "I want algorithmic summation (q-Gosper)" -- :func:`~q_kangaroo.q_gosper`
- "I want creative telescoping (q-Zeilberger)" -- :func:`~q_kangaroo.q_zeilberger`
- "I want a WZ proof certificate" -- :func:`~q_kangaroo.verify_wz`
- "I want to solve q-hypergeometric recurrences" -- :func:`~q_kangaroo.q_petkovsek`
- "I want to prove a nonterminating identity" -- :func:`~q_kangaroo.prove_nonterminating`
- "I want to find transformation chains" -- :func:`~q_kangaroo.find_transformation_chain`

6. Summation and Transformation
-------------------------------

Closed-form summation of hypergeometric series and parameter transformations.

- "I want to try closed-form summation" -- :func:`~q_kangaroo.try_summation` (tries q-Gauss, q-Vandermonde, q-Saalschutz, q-Kummer, q-Dixon)
- "I want Heine's first transformation of 2phi1" -- :func:`~q_kangaroo.heine1`
- "I want Heine's second transformation of 2phi1" -- :func:`~q_kangaroo.heine2`
- "I want Heine's third transformation of 2phi1" -- :func:`~q_kangaroo.heine3`

.. note::

   Sears' balanced 4phi3 transformation and Watson's 8phi7 to 4phi3 reduction
   are available internally via :func:`~q_kangaroo.try_summation` and
   :func:`~q_kangaroo.find_transformation_chain`, which apply them automatically
   when applicable.

7. Mock Theta Functions and Beyond
----------------------------------

Ramanujan's classical mock theta functions, Appell-Lerch sums, universal
mock theta functions, and the Bailey chain machinery.

**Third-order mock theta functions** (7 functions):

- :func:`~q_kangaroo.mock_theta_f3`, :func:`~q_kangaroo.mock_theta_phi3`, :func:`~q_kangaroo.mock_theta_psi3`, :func:`~q_kangaroo.mock_theta_chi3`, :func:`~q_kangaroo.mock_theta_omega3`, :func:`~q_kangaroo.mock_theta_nu3`, :func:`~q_kangaroo.mock_theta_rho3`

**Fifth-order mock theta functions** (10 functions):

- :func:`~q_kangaroo.mock_theta_f0_5`, :func:`~q_kangaroo.mock_theta_f1_5`, :func:`~q_kangaroo.mock_theta_cap_f0_5`, :func:`~q_kangaroo.mock_theta_cap_f1_5`, :func:`~q_kangaroo.mock_theta_phi0_5`, :func:`~q_kangaroo.mock_theta_phi1_5`, :func:`~q_kangaroo.mock_theta_psi0_5`, :func:`~q_kangaroo.mock_theta_psi1_5`, :func:`~q_kangaroo.mock_theta_chi0_5`, :func:`~q_kangaroo.mock_theta_chi1_5`

**Seventh-order mock theta functions** (3 functions):

- :func:`~q_kangaroo.mock_theta_cap_f0_7`, :func:`~q_kangaroo.mock_theta_cap_f1_7`, :func:`~q_kangaroo.mock_theta_cap_f2_7`

**Appell-Lerch sums and universal mock theta functions:**

- "I want an Appell-Lerch sum" -- :func:`~q_kangaroo.appell_lerch_m`
- "I want universal mock theta g2" -- :func:`~q_kangaroo.universal_mock_theta_g2`
- "I want universal mock theta g3" -- :func:`~q_kangaroo.universal_mock_theta_g3`

**Bailey chain machinery:**

- "I want Bailey's weak lemma" -- :func:`~q_kangaroo.bailey_weak_lemma`
- "I want to apply Bailey's lemma" -- :func:`~q_kangaroo.bailey_apply_lemma`
- "I want a multi-step Bailey chain" -- :func:`~q_kangaroo.bailey_chain`
- "I want to discover Bailey pair relationships" -- :func:`~q_kangaroo.bailey_discover`

Still not sure?
---------------

Browse the complete :doc:`api/index` for all 79 functions organized by
implementation group, or work through the :doc:`examples/getting_started`
tutorial for a hands-on introduction.
