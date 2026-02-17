Relation Discovery
==================

These functions discover algebraic and linear relations among $q$-series
using exact rational arithmetic or modular arithmetic over $\mathbb{Z}/p\mathbb{Z}$.
They are the core tools for experimental mathematics with $q$-series:
finding linear combinations, homogeneous polynomial relations, partition
congruences, and more.

Exact Rational Arithmetic
-------------------------

.. autofunction:: q_kangaroo.findlincombo

.. autofunction:: q_kangaroo.findhom

.. autofunction:: q_kangaroo.findpoly

.. autofunction:: q_kangaroo.findcong

.. autofunction:: q_kangaroo.findnonhom

.. autofunction:: q_kangaroo.findhomcombo

.. autofunction:: q_kangaroo.findnonhomcombo

Modular and Structural
----------------------

.. autofunction:: q_kangaroo.findlincombomodp

.. autofunction:: q_kangaroo.findhommodp

.. autofunction:: q_kangaroo.findhomcombomodp

.. autofunction:: q_kangaroo.findmaxind

.. autofunction:: q_kangaroo.findprod

.. seealso::

   :doc:`/examples/series_analysis` -- Discovering relations with findlincombo, findhom, findpoly, findcong

   :doc:`/examples/partition_congruences` -- Using findcong for partition congruences

   :doc:`/examples/maple_migration` -- Maple ``findhom``, ``findprod``, ``findcong`` equivalents
