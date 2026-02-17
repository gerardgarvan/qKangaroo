Example Gallery
===============

Tutorials and guides demonstrating q-Kangaroo's features with mathematical context.

**New to q-series?** Start with :doc:`getting_started`.

**Switching from Maple?** See the :doc:`maple_migration` guide.

Tutorials
---------

Step-by-step walkthroughs that build skills progressively.

**Getting Started**
   Zero to first identity. Install q-Kangaroo, create a session, compute
   q-Pochhammer symbols, build products, and prove an identity.
   *No prerequisites.*

**Series Analysis**
   The analysis pipeline: prodmake, etamake, sift, and relation discovery
   (findlincombo, findhom, findpoly, findcong). Learn to reverse-engineer
   any q-series into its product form and discover algebraic relations.
   *Requires: basic q-series familiarity.*

**Identity Proving**
   Algorithmic proof techniques: q-Zeilberger, WZ certificates, q-Petkovsek,
   and nonterminating identities. Use creative telescoping and recurrence
   solving to prove hypergeometric identities.
   *Requires: hypergeometric series background.*

.. toctree::
   :maxdepth: 1
   :caption: Tutorials

   getting_started
   series_analysis
   identity_proving

Topic Guides
------------

Deep dives into specific areas of q-series theory, each with worked examples
and mathematical context.

**Partition Congruences**
   Ramanujan's congruences, rank and crank generating functions, Dyson's
   conjecture, and prodmake analysis of partition series.
   *For: number theory researchers.*

**Theta Identities**
   Jacobi triple product, quintuple product, Winquist formula, and theta
   function relationships. Verify classical identities computationally.
   *For: modular forms background.*

**Hypergeometric Summation**
   All 6 summation formulas (q-Gauss, q-Vandermonde, q-Saalschutz, q-Kummer,
   q-Dixon), Heine transforms, Sears' balanced 4phi3, and Watson's 8phi7
   reduction.
   *For: special functions researchers.*

**Mock Theta Functions**
   Ramanujan's 20 mock theta functions across three orders, Appell-Lerch
   connection, and the universal mock theta functions g2 and g3.
   *For: mock modular forms background.*

**Bailey Chains**
   Bailey pairs, lemma, multi-step chains, Rogers-Ramanujan identities from
   the unit pair, and bailey_discover for automatic pair matching.
   *For: combinatorial identity researchers.*

.. toctree::
   :maxdepth: 1
   :caption: Topic Guides

   partition_congruences
   theta_identities
   hypergeometric_summation
   mock_theta_functions
   bailey_chains

Reference
---------

**Maple Migration**
   Side-by-side translation of all 13 function groups from Garvan's Maple
   packages (qseries, thetaids, prodmake, etc.). Each function shows the
   Maple call and the equivalent q-Kangaroo Python call.
   *For: researchers migrating existing Maple code.*

.. toctree::
   :maxdepth: 1
   :caption: Reference

   maple_migration
