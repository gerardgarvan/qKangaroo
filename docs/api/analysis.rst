Series Analysis
===============

Tools for analyzing and reverse-engineering $q$-series. Andrews' ``prodmake``
algorithm recovers infinite product exponents via log-derivative and Mobius
inversion. The ``etamake`` and ``jacprodmake`` functions express series in
standard product forms. The ``qfactor`` function factors $q$-polynomials,
and ``sift`` extracts arithmetic subsequences.

.. autofunction:: q_kangaroo.qfactor

.. autofunction:: q_kangaroo.sift

.. autofunction:: q_kangaroo.qdegree

.. autofunction:: q_kangaroo.lqdegree

.. autofunction:: q_kangaroo.prodmake

.. autofunction:: q_kangaroo.etamake

.. autofunction:: q_kangaroo.jacprodmake

.. autofunction:: q_kangaroo.mprodmake

.. autofunction:: q_kangaroo.qetamake

.. seealso::

   :doc:`/examples/series_analysis` -- Complete prodmake/etamake/sift/relation-discovery pipeline

   :doc:`/examples/partition_congruences` -- Using sift and prodmake for congruence analysis

   :doc:`/examples/maple_migration` -- Maple ``prodmake``, ``etamake``, ``sift`` equivalents
