q-Kangaroo Documentation
========================

**q-Kangaroo** is an open-source symbolic computation engine for q-series,
built as a modern replacement for Garvan's classical Maple packages.

It provides exact rational arithmetic over formal power series, covering
q-Pochhammer symbols, theta functions, partition generating functions,
hypergeometric series, mock theta functions, Bailey chains, and identity
proving -- all accessible from Python via a clean API.

Getting Started
---------------

.. important:: **First time installing?**

   Follow the step-by-step installation guide for pip install, building from
   source, and troubleshooting common issues.

   :doc:`installation`

.. tip:: **New to q-series?**

   Start with our step-by-step tutorial that takes you from installation to
   your first identity. No prerequisites required.

   :doc:`examples/getting_started`

.. note:: **Switching from Maple?**

   Side-by-side translation guide for all 13 function groups in Garvan's
   packages. Migrate your existing code with minimal effort.

   :doc:`examples/maple_migration`

.. seealso:: **Looking for a specific function?**

   Use our decision guide to find the right function for your task, or
   browse the full API reference (79 functions in 13 groups).

   - :doc:`function_guide` -- Which function should I use?
   - :doc:`api/index` -- Full API reference

What's Inside
-------------

- **q-Pochhammer symbols & products** -- 10 functions for building q-series from scratch (aqprod, etaq, jacprod, tripleprod, quinprod, winquist, and more)
- **Partition functions** -- 7 functions for counting, generating, and analyzing partitions (partition_count, rank_gf, crank_gf, restricted parts)
- **Series analysis & relations** -- 18 functions for reverse-engineering series into products, finding linear combinations, and discovering congruences
- **Hypergeometric summation** -- 14 functions covering phi/psi evaluation, 5 summation formulas, Heine/Sears/Watson transformations
- **Mock theta, Bailey, & identity proving** -- 30 functions for Ramanujan's mock theta functions, Appell-Lerch sums, Bailey chains, q-Zeilberger, and WZ certificates

.. toctree::
   :maxdepth: 2
   :caption: User Guide

   installation
   quickstart
   mathematical_notation

.. toctree::
   :maxdepth: 2
   :caption: Guides

   function_guide

.. toctree::
   :maxdepth: 2
   :caption: API Reference

   api/index

.. toctree::
   :maxdepth: 2
   :caption: Examples

   examples/index

Indices and tables
==================

* :ref:`genindex`
* :ref:`search`
