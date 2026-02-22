# Requirements: q-Kangaroo v5.0

**Defined:** 2026-02-21
**Core Value:** Maximum Maple compatibility — researchers can run their Maple code in q-Kangaroo with minimal modification

## v5.0 Requirements

### Bug Fixes

- [ ] **BUG-01**: Division by exact polynomial (POLYNOMIAL_ORDER sentinel) completes in bounded time instead of hanging — `1/aqprod(q,q,5)` and `q^n/aqprod(q,q,n)` inside for-loops work correctly

### Language Features

- [ ] **LANG-01**: `while...do...od` loops execute with boolean/comparison conditions
- [ ] **LANG-02**: List literals `[a, b, c]` as first-class values with display and indexing `L[i]`
- [ ] **LANG-03**: Unicode operator resilience — `∧`→`^`, `·`→`*`, `−`→`-`, `×`→`*` accepted by parser
- [ ] **LANG-04**: `print(expr)` displays intermediate results during loops/procedures

### Series & Polynomial Functions

- [ ] **SERIES-01**: `coeff(f, q, n)` extracts the coefficient of `q^n` from a series
- [ ] **SERIES-02**: `degree(f, q)` returns the degree of a polynomial/series
- [ ] **SERIES-03**: `numer(f)` and `denom(f)` extract numerator/denominator of rational expressions

### Summation & Iteration Functions

- [ ] **ITER-01**: `add(expr, i=a..b)` computes symbolic summation (Maple-style)
- [ ] **ITER-02**: `mul(expr, i=a..b)` computes symbolic product (Maple-style)
- [ ] **ITER-03**: `seq(expr, i=a..b)` generates a list/sequence

### List & Expression Functions

- [ ] **LIST-01**: `nops(expr)` returns the number of operands/elements
- [ ] **LIST-02**: `op(i, expr)` extracts the i-th operand/element
- [ ] **LIST-03**: `map(f, list)` applies function f to each element
- [ ] **LIST-04**: `sort(list)` sorts list elements

### Utility Functions

- [ ] **UTIL-01**: `modp(a, p)` and `mods(a, p)` for modular arithmetic
- [ ] **UTIL-02**: `type(expr, t)` checks expression type (integer, series, list, etc.)
- [ ] **UTIL-03**: `evalb(expr)` evaluates boolean expression
- [ ] **UTIL-04**: `cat(s1, s2, ...)` concatenates strings/names

### Documentation

- [ ] **DOC-01**: Help entries and tab completion for all new functions/keywords
- [ ] **DOC-02**: PDF manual chapter documenting v5.0 additions

## Out of Scope

| Feature | Reason |
|---------|--------|
| zqfactor (bivariate factoring) | Requires fundamental bivariate polynomial infrastructure |
| Full Maple parser compatibility | Goal is q-series research code, not arbitrary Maple programs |
| GUI / IDE integration | CLI and library only |
| convert() family | Too broad; specific conversions added as needed |
| assume() / assumptions | Complex symbolic engine feature beyond scope |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| BUG-01 | Phase 52 | Pending |
| LANG-01 | Phase 52 | Pending |
| LANG-02 | Phase 53 | Pending |
| LANG-03 | Phase 52 | Pending |
| LANG-04 | Phase 52 | Pending |
| SERIES-01 | Phase 54 | Pending |
| SERIES-02 | Phase 54 | Pending |
| SERIES-03 | Phase 54 | Pending |
| ITER-01 | Phase 55 | Pending |
| ITER-02 | Phase 55 | Pending |
| ITER-03 | Phase 55 | Pending |
| LIST-01 | Phase 53 | Pending |
| LIST-02 | Phase 53 | Pending |
| LIST-03 | Phase 53 | Pending |
| LIST-04 | Phase 53 | Pending |
| UTIL-01 | Phase 54 | Pending |
| UTIL-02 | Phase 54 | Pending |
| UTIL-03 | Phase 54 | Pending |
| UTIL-04 | Phase 54 | Pending |
| DOC-01 | Phase 56 | Pending |
| DOC-02 | Phase 56 | Pending |

**Coverage:**
- v5.0 requirements: 21 total
- Mapped to phases: 21
- Unmapped: 0

---
*Requirements defined: 2026-02-21*
