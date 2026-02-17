QSeries
=======

A ``QSeries`` represents a formal power series with sparse rational
coefficients, truncated at a specified order $N$. Coefficients are
returned as :class:`fractions.Fraction` objects for exact arithmetic.

``QSeries`` supports arithmetic (``+``, ``-``, ``*``), coefficient access
via ``series[k]``, and LaTeX rendering in Jupyter notebooks via
``_repr_latex_()``.

.. autoclass:: q_kangaroo.QSeries
   :members:
   :undoc-members:

.. seealso::

   :doc:`/examples/getting_started` -- Working with formal power series

   :doc:`/examples/series_analysis` -- Analyzing and reverse-engineering series
