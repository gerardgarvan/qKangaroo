# q-Kangaroo

[![codecov](https://codecov.io/gh/OWNER/q-kangaroo/graph/badge.svg)](https://codecov.io/gh/OWNER/q-kangaroo)

A symbolic computation engine for q-series, partition functions, and modular forms. Built as a high-performance Rust core with Python bindings via PyO3, q-Kangaroo provides exact arithmetic over the rationals for research in combinatorics and number theory.

## Features

- **q-Pochhammer symbols** -- finite, infinite, and bilateral products with exact rational coefficients
- **Theta functions** -- Jacobi theta functions (theta2, theta3, theta4) as formal power series
- **Partition functions** -- partition counting via pentagonal recurrence, rank and crank generating functions
- **Hypergeometric series** -- basic hypergeometric (phi) and bilateral (psi) series with summation formulas (q-Gauss, q-Vandermonde, q-Saalschutz, q-Kummer, q-Dixon)
- **Mock theta functions** -- 20 classical mock theta functions (third, fifth, and seventh order)
- **Bailey chains** -- Bailey pairs, lemma, chain, weak lemma, and automated discovery
- **Identity proving** -- coefficient comparison, series manipulation, and algebraic verification
- **Series analysis** -- product analysis (prodmake, etamake, jacprodmake), sifting, linear relation finding, modular arithmetic

## Installation

```bash
pip install q-kangaroo
```

> **Note:** Not yet published to PyPI. For development installation, clone the repository and build with maturin:
>
> ```bash
> cd crates/qsym-python
> pip install maturin
> maturin develop --release
> ```

## Quick Example

```python
from q_kangaroo import QSession, partition_count

# Count integer partitions
for n in range(10):
    print(f"p({n}) = {partition_count(n)}")

# Work with q-series
session = QSession(precision=20)
q = session.var("q")
series = session.qpochhammer_inf(q)
print(series)  # Euler's function (q;q)_inf
```

## License

MIT
