Mathematical Notation
=====================

This page maps q-Kangaroo function names to their standard mathematical
notation. All functions operate on formal power series with exact
rational coefficients, truncated at a specified order $N$.

$q$-Pochhammer Symbol
---------------------

The $q$-Pochhammer symbol $(a;q)_n$ is the fundamental object:

.. math::

   (a;q)_n = \prod_{k=0}^{n-1} (1 - a\,q^k)

For $n = \infty$:

.. math::

   (a;q)_\infty = \prod_{k=0}^{\infty} (1 - a\,q^k)

In q-Kangaroo, ``aqprod(session, coeff_num, coeff_den, power, n, order)``
computes $(a;q)_n$ where $a = \frac{c_n}{c_d} \cdot q^p$. Pass ``n=None``
for the infinite product.

$q$-Binomial Coefficient
------------------------

The $q$-binomial (Gaussian) coefficient:

.. math::

   \binom{n}{k}_q = \frac{(q;q)_n}{(q;q)_k\,(q;q)_{n-k}}

Computed by ``qbin(session, n, k, order)``.

Named Products
--------------

.. list-table::
   :header-rows: 1
   :widths: 20 55 25

   * - Function
     - Mathematical Form
     - Reference
   * - ``etaq(s,b,t,N)``
     - $(q^b; q^t)_\infty$
     - Generalized eta
   * - ``jacprod``
     - $\text{JAC}(a,b) = (q^a;q^b)(q^{b-a};q^b)(q^b;q^b)$
     - Jacobi product
   * - ``tripleprod``
     - $(z;q)(q/z;q)(q;q)$
     - Jacobi triple
   * - ``quinprod``
     - $(z;q)(q/z;q)(z^2;q^2)(q^2/z^2;q^2)(q;q)(q^2;q^2)$
     - Quintuple product
   * - ``winquist``
     - Winquist's 10-factor product
     - Winquist (1969)

The Dedekind eta function is $\eta(\tau) = q^{1/24}(q;q)_\infty$ where
$q = e^{2\pi i\tau}$. Since q-Kangaroo works with integer exponents,
``etaq(s, 1, 1, N)`` gives $(q;q)_\infty$ without the $q^{1/24}$ prefactor.

Theta Functions
---------------

The Jacobi theta functions:

.. math::

   \theta_2(q) &= 2q^{1/4} \sum_{n=0}^{\infty} q^{n(n+1)} \\
   \theta_3(q) &= \sum_{n=-\infty}^{\infty} q^{n^2} = 1 + 2\sum_{n=1}^{\infty} q^{n^2} \\
   \theta_4(q) &= \sum_{n=-\infty}^{\infty} (-1)^n q^{n^2}

Computed by ``theta2``, ``theta3``, ``theta4``. Note that ``theta2``
uses the $q^{1/4}$ convention: the FPS exponents represent powers of
$X = q^{1/4}$.

Partition Functions
-------------------

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Function
     - Generating Function
   * - ``partition_gf``
     - $\sum_{n=0}^{\infty} p(n)\,q^n = \frac{1}{(q;q)_\infty}$
   * - ``distinct_parts_gf``
     - $\prod_{k=1}^{\infty}(1+q^k) = (-q;q)_\infty$
   * - ``odd_parts_gf``
     - $\prod_{k=0}^{\infty}\frac{1}{1-q^{2k+1}}$
   * - ``bounded_parts_gf``
     - $\frac{1}{(q;q)_m}$ (parts $\le m$)
   * - ``rank_gf``
     - $R(z,q)$: Dyson rank generating function
   * - ``crank_gf``
     - $C(z,q)$: Andrews--Garvan crank generating function

``partition_count(n)`` computes $p(n)$ directly via the pentagonal number
recurrence, without constructing a generating function.

Hypergeometric Series
---------------------

Basic hypergeometric series (DLMF 17.4.1):

.. math::

   {}_r\phi_s\!\left[\begin{matrix}a_1,\ldots,a_r\\b_1,\ldots,b_s\end{matrix};\,q,\,z\right]
   = \sum_{n=0}^{\infty} \frac{(a_1;q)_n \cdots (a_r;q)_n}{(b_1;q)_n \cdots (b_s;q)_n}\,
   \frac{z^n}{(q;q)_n}\,(-1)^{(1+s-r)n}\,q^{(1+s-r)\binom{n}{2}}

Computed by ``phi(session, upper, lower, z_num, z_den, z_pow, order)``
where ``upper`` and ``lower`` are lists of ``(coeff_num, coeff_den, power)``
tuples specifying the parameter monomials.

Bilateral series ${}_r\psi_s$ are computed by ``psi`` with the same
interface.

Series Analysis
---------------

- ``prodmake(series, max_n)``: Andrews' algorithm -- recover infinite product
  exponents $a_n$ in $\prod(1-q^n)^{a_n}$ via log-derivative and Mobius inversion.
- ``etamake(series, max_n)``: Express as eta-quotient $\prod \eta(d\tau)^{e_d}$.
- ``jacprodmake(series, max_n)``: Express as Jacobi product form.
- ``mprodmake(series, max_n)``: Express as product of $(1+q^n)$ factors.
- ``qfactor(series)``: Factor a $q$-polynomial into $(1-q^i)$ components.
- ``sift(series, m, j)``: Extract the arithmetic subsequence $f(m\cdot i + j)$.

Truncation Model
----------------

All series are computed to a specified truncation order $N$: the series is
known exactly for exponents $< N$, and represented as $f(q) + O(q^N)$.
Internally, coefficients are stored in a sparse ``BTreeMap<i64, QRat>``
mapping exponent to exact rational coefficient. Multiplication truncates
during computation (not post-hoc), giving $O(N)$ cost for sparse series.
