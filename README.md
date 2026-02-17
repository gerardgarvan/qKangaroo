# q-Kangaroo

[![codecov](https://codecov.io/gh/OWNER/q-kangaroo/graph/badge.svg)](https://codecov.io/gh/OWNER/q-kangaroo)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A symbolic computation engine for q-series, partition functions, and modular forms.
Built as a high-performance Rust core with Python bindings via PyO3, q-Kangaroo
provides exact arithmetic over the rationals for research in combinatorics and
number theory.

## Installation

```bash
pip install q-kangaroo
```

**Requirements:** Python 3.9+. GMP library is bundled in pre-built wheels.

For build-from-source instructions, platform-specific guides, and troubleshooting, see [INSTALL.md](INSTALL.md).

## Quick Start

```python
from q_kangaroo import QSession, etaq, partition_count, theta3

# Count integer partitions
for n in range(6):
    print(f"p({n}) = {partition_count(n)}")
# p(0) = 1, p(1) = 1, p(2) = 2, p(3) = 3, p(4) = 5, p(5) = 7

# Compute Euler's function (q;q)_inf as a power series
s = QSession()
euler = etaq(s, 1, 1, 20)
print(euler)  # 1 - q - q^2 + q^5 + q^7 - q^12 - q^15 + O(q^20)

# Jacobi theta function
t3 = theta3(s, 20)
print(t3)  # 1 + 2*q + 2*q^4 + 2*q^9 + 2*q^16 + O(q^20)
```

## Verification

```bash
python -c "from q_kangaroo import partition_count; assert partition_count(50) == 204226; print('q-Kangaroo is working!')"
```

## Features

- **q-Pochhammer symbols** -- finite, infinite, and bilateral products with exact rational coefficients
- **Theta functions** -- Jacobi theta functions (theta2, theta3, theta4) as formal power series
- **Partition functions** -- partition counting via pentagonal recurrence, rank and crank generating functions
- **Hypergeometric series** -- basic hypergeometric (phi) and bilateral (psi) series with summation formulas (q-Gauss, q-Vandermonde, q-Saalschutz, q-Kummer, q-Dixon) and transformations (Heine, Sears, Watson)
- **Mock theta functions** -- 20 classical mock theta functions (third, fifth, and seventh order)
- **Appell-Lerch sums** -- bilateral sums and universal mock theta functions (g2, g3)
- **Bailey chains** -- Bailey pairs, lemma, chain, weak lemma, and automated discovery
- **Identity proving** -- eta-quotient identity verification via the valence formula, identity database search
- **Series analysis** -- product analysis (prodmake, etamake, jacprodmake), sifting, polynomial factoring
- **Relation discovery** -- linear combinations, homogeneous/non-homogeneous polynomial relations, congruence detection, modular arithmetic
- **Symbolic expressions** -- hash-consed expression arena with simplification engine

## Jupyter Support

QExpr and QSeries objects render as LaTeX automatically in Jupyter notebooks
via `_repr_latex_()`. No additional configuration is needed -- simply display
an expression or series in a notebook cell and it renders with proper
mathematical typesetting.

```python
from q_kangaroo import QSession, etaq

s = QSession()
euler = etaq(s, 1, 1, 20)
euler  # renders as: 1 - q - q^{2} + q^{5} + q^{7} - q^{12} - q^{15} + O(q^{20})
```

## License

MIT -- see [LICENSE](LICENSE) for details.
