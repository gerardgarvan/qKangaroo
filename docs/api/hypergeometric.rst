Hypergeometric Series
=====================

Basic hypergeometric series ${}_r\phi_s$ (DLMF 17.4.1) and bilateral
series ${}_r\psi_s$ are computed as formal power series. Parameters are
specified as ``(coeff_num, coeff_den, power)`` tuples representing the
monomial $\frac{c_n}{c_d} \cdot q^p$. The ``try_summation`` function
attempts known closed-form summation formulas ($q$-Gauss, $q$-Vandermonde,
$q$-Saalschutz, etc.), and the Heine transformations convert between
equivalent ${}_2\phi_1$ representations.

.. autofunction:: q_kangaroo.phi

.. autofunction:: q_kangaroo.psi

.. autofunction:: q_kangaroo.try_summation

.. autofunction:: q_kangaroo.heine1

.. autofunction:: q_kangaroo.heine2

.. autofunction:: q_kangaroo.heine3

.. seealso::

   :doc:`/examples/hypergeometric_summation` -- All summation formulas, Heine transforms, Sears, and Watson

   :doc:`/examples/identity_proving` -- Using hypergeometric identities in proof workflows

   :doc:`/examples/maple_migration` -- Maple ``qphihyper``, ``qpsihyper`` equivalents
