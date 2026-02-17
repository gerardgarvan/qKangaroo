Identity Proving
================

Tools for proving eta-quotient identities and searching the identity
database. The ``prove_eta_id`` function uses the valence formula for
modular forms to verify that two eta-quotients are equal, requiring only
a finite number of coefficient checks. The ``search_identities`` function
queries a built-in database of known identities by tag, function name, or
pattern.

.. autofunction:: q_kangaroo.prove_eta_id

.. autofunction:: q_kangaroo.search_identities

.. seealso::

   :doc:`/examples/identity_proving` -- Proving identities with q-Zeilberger and WZ certificates

   :doc:`/examples/theta_identities` -- Eta-quotient identities for theta functions

   :doc:`/examples/maple_migration` -- Maple identity proving equivalents
