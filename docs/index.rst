q-Kangaroo Documentation
========================

**q-Kangaroo** is an open-source symbolic computation engine for q-series,
built as a modern replacement for Garvan's classical Maple packages.

It provides exact rational arithmetic over formal power series, covering
q-Pochhammer symbols, theta functions, partition generating functions,
hypergeometric series, mock theta functions, Bailey chains, and identity
proving -- all accessible from Python via a clean API.

Key features:

- **79 functions** organized in 13 functional groups
- **Exact arithmetic** using GMP-backed rationals (no floating-point)
- **Sparse representation** via BTreeMap with explicit truncation
- **LaTeX rendering** for Jupyter notebooks ($q$-series display as math)
- **NumPy-style docstrings** with mathematical notation on every function

.. toctree::
   :maxdepth: 2
   :caption: User Guide

   installation
   quickstart
   mathematical_notation

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
