Partition Functions
===================

Integer partitions are a central topic in combinatorics and number theory.
The partition function $p(n)$ counts the number of ways to write $n$ as a
sum of positive integers. q-Kangaroo provides both direct computation of
$p(n)$ and generating functions for various partition statistics including
the Dyson rank and Andrews--Garvan crank.

.. autofunction:: q_kangaroo.partition_count

.. autofunction:: q_kangaroo.partition_gf

.. autofunction:: q_kangaroo.distinct_parts_gf

.. autofunction:: q_kangaroo.odd_parts_gf

.. autofunction:: q_kangaroo.bounded_parts_gf

.. autofunction:: q_kangaroo.rank_gf

.. autofunction:: q_kangaroo.crank_gf

.. seealso::

   :doc:`/examples/partition_congruences` -- Ramanujan's congruences, rank, crank, and Dyson's conjecture

   :doc:`/examples/getting_started` -- First steps with partition generating functions

   :doc:`/examples/maple_migration` -- Maple partition function equivalents
