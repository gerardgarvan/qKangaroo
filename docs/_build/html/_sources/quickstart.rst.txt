Quick Start
===========

This guide walks through the core features of q-Kangaroo: computing
$q$-series, counting partitions, discovering congruences, and working
with theta functions.

Creating a Session
------------------

Every computation starts with a ``QSession``, which owns the expression
arena and manages memory:

.. code-block:: python

   from q_kangaroo import QSession
   s = QSession()

For quick interactive use, you can use the convenience helper:

.. code-block:: python

   from q_kangaroo import get_default_session
   s = get_default_session()

Computing Euler's Function
--------------------------

Euler's function $(q;q)_\infty = \prod_{k=1}^{\infty}(1-q^k)$ is the
fundamental building block of $q$-series. Compute it with ``etaq``:

.. code-block:: python

   from q_kangaroo import QSession, etaq
   s = QSession()
   euler = etaq(s, 1, 1, 20)
   print(euler)
   # 1 - q - q^2 + q^5 + q^7 - q^12 - q^15 + O(q^20)

The arguments to ``etaq(session, b, t, order)`` define the product
$(q^b; q^t)_\infty$, truncated at $O(q^{\mathtt{order}})$.

Counting Partitions
-------------------

The number of integer partitions $p(n)$ counts the ways to write $n$ as
a sum of positive integers. q-Kangaroo computes this via the pentagonal
number recurrence:

.. code-block:: python

   from q_kangaroo import partition_count
   print(partition_count(5))    # 7
   print(partition_count(10))   # 42
   print(partition_count(50))   # 204226

You can also work with the partition generating function
$\sum_{n=0}^{\infty} p(n)\,q^n = \frac{1}{(q;q)_\infty}$:

.. code-block:: python

   from q_kangaroo import QSession, partition_gf
   s = QSession()
   pgf = partition_gf(s, 20)
   print(pgf)
   # 1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + ... + O(q^20)

Discovering Partition Congruences
---------------------------------

Ramanujan discovered that $p(5n+4) \equiv 0 \pmod{5}$. The ``findcong``
function discovers such congruences automatically:

.. code-block:: python

   from q_kangaroo import QSession, partition_gf, findcong
   s = QSession()
   pgf = partition_gf(s, 200)
   congs = findcong(pgf, [5, 7, 11])
   for c in congs:
       print(c)
   # {'modulus': 5, 'residue': 4, 'cong_mod': 5}
   # {'modulus': 7, 'residue': 5, 'cong_mod': 7}
   # {'modulus': 11, 'residue': 6, 'cong_mod': 11}

This rediscovers Ramanujan's three classical congruences:
$p(5n+4) \equiv 0 \pmod{5}$,
$p(7n+5) \equiv 0 \pmod{7}$, and
$p(11n+6) \equiv 0 \pmod{11}$.

Working with Theta Functions
-----------------------------

The Jacobi theta functions $\theta_2(q)$, $\theta_3(q)$, and
$\theta_4(q)$ are available:

.. code-block:: python

   from q_kangaroo import QSession, theta3
   s = QSession()
   t3 = theta3(s, 20)
   print(t3)
   # 1 + 2*q + 2*q^4 + 2*q^9 + 2*q^16 + O(q^20)

$q$-Pochhammer Symbols
-----------------------

The general $q$-Pochhammer symbol $(a;q)_n$ is computed with ``aqprod``.
The parameters specify the base monomial $a = \frac{c_{\mathrm{num}}}{c_{\mathrm{den}}} \cdot q^p$:

.. code-block:: python

   from q_kangaroo import QSession, aqprod
   s = QSession()
   # (q; q)_5 -- finite Pochhammer (n=5)
   finite = aqprod(s, 1, 1, 1, 5, 20)
   print(finite)

   # (q; q)_inf -- infinite Pochhammer (n=None)
   infinite = aqprod(s, 1, 1, 1, None, 20)
   print(infinite)

Hypergeometric Series
---------------------

Basic hypergeometric series ${}_r\phi_s$ and bilateral series
${}_r\psi_s$ are available. Parameters are specified as tuples
``(coeff_num, coeff_den, power)`` representing the monomial
$\frac{c_n}{c_d} \cdot q^p$:

.. code-block:: python

   from q_kangaroo import QSession, phi, try_summation
   s = QSession()

   # 2phi1(q, q^2; q^3; q, q) -- a basic hypergeometric series
   upper = [(1, 1, 1), (1, 1, 2)]
   lower = [(1, 1, 3)]
   result = phi(s, upper, lower, 1, 1, 1, 20)
   print(result)

LaTeX in Jupyter
----------------

In Jupyter notebooks, ``QSeries`` objects render as LaTeX automatically
via ``_repr_latex_()``. You can also access the LaTeX string directly:

.. code-block:: python

   from q_kangaroo import QSession, etaq
   s = QSession()
   euler = etaq(s, 1, 1, 20)
   print(euler.latex())
   # 1 - q - q^{2} + q^{5} + q^{7} - q^{12} - q^{15} + O(q^{20})

Next Steps
----------

- See the :doc:`api/index` for the complete function reference.
- Read :doc:`mathematical_notation` for the mathematical conventions used.
