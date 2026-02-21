# Requirements: q-Kangaroo v3.0

**Defined:** 2026-02-20
**Core Value:** Every example in Garvan's "q-Product Tutorial" (qmaple.pdf) runs correctly in q-Kangaroo

## v3.0 Requirements

### Scripting Language

- [ ] **SCRIPT-01**: User can write for-loops: `for n from 1 to 8 do stmts od`
- [ ] **SCRIPT-02**: User can define named procedures: `f := proc(args) local x,k; stmts; end`
- [ ] **SCRIPT-03**: User can use `option remember` in procedures for memoization
- [ ] **SCRIPT-04**: User can use `RETURN(value)` to exit a procedure early
- [ ] **SCRIPT-05**: User can use `local x, y` declarations for procedure-scoped variables
- [ ] **SCRIPT-06**: User can use if/elif/else/fi conditionals in procedures and at top level
- [ ] **SCRIPT-07**: User can use boolean operators (`and`, `or`, `not`) and comparison operators (`=`, `<>`, `<`, `>`, `<=`, `>=`) in conditions

### Series & Expression Operations

- [ ] **SERIES-01**: User can call `series(expr, q, T)` to truncate a computed expression to O(q^T)
- [ ] **SERIES-02**: User can call `expand(expr)` to expand products into polynomial form
- [ ] **SERIES-03**: User can use runtime integer arithmetic in q-exponents: `q^(n*n)`, `q^(k*(3*k+1)/2)` where n, k are loop variables

### Polynomial Operations

- [ ] **POLY-01**: User can call `factor(poly)` to factor a polynomial in q into cyclotomic and irreducible factors over the rationals
- [ ] **POLY-02**: User can call `subs(var=val, expr)` to substitute a value for a variable in an expression

### Bivariate Series

- [ ] **BIVAR-01**: User can call `tripleprod(z, q, T)` where z is a symbolic variable and get a Laurent polynomial in z with q-series coefficients
- [ ] **BIVAR-02**: User can call `quinprod(z, q, T)` where z is a symbolic variable and get a Laurent polynomial in z with q-series coefficients
- [ ] **BIVAR-03**: User can call `winquist(a, b, q, T)` where a, b are symbolic variables and get a multivariate series
- [ ] **BIVAR-04**: User can perform arithmetic (add, subtract, multiply, negate) on bivariate series values

### Utility Functions

- [ ] **UTIL-01**: User can call `floor(x)` to compute the floor of a rational number
- [ ] **UTIL-02**: User can call `legendre(m, p)` (or `L(m, p)`) to compute the Legendre symbol (m/p)

### Documentation

- [ ] **DOC-01**: PDF manual includes a chapter on the scripting language (for, proc, if, series, factor, subs)
- [ ] **DOC-02**: Help system (`help for`, `help proc`, `help if`) documents new scripting syntax
- [ ] **DOC-03**: Worked examples section includes reproductions of key examples from Garvan's qmaple.pdf tutorial

## Future Requirements

### Bivariate Extensions

- **BIVAR-05**: `quinprod(z, q, prodid)` displays the quintuple product identity in symbolic product form
- **BIVAR-06**: `quinprod(z, q, seriesid)` displays the quintuple product identity in symbolic series form
- **BIVAR-07**: `zqfactor(f)` factors bivariate rational functions in (z, q) as q-products

### Additional Maple Parity

- **MAPLE-01**: `radsimp(expr)` rational simplification for expressions involving series quotients
- **MAPLE-02**: While-loops: `while cond do stmts od`
- **MAPLE-03**: `nops(list)`, `op(i, list)` list manipulation functions

## Out of Scope

| Feature | Reason |
|---------|--------|
| Full Maple language compatibility | Focused on qseries tutorial examples, not general CAS |
| Symbolic differentiation/integration | Not used in qmaple.pdf examples |
| Arbitrary-precision floats | Symbolic-first; exact rational arithmetic only |
| String manipulation | Not needed for q-series computation |
| File I/O beyond read() | Script execution via read() already exists |
| Module/package system | Single-file scripts sufficient for tutorial examples |
| Interactive debugging (breakpoints) | Not needed for tutorial parity |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SCRIPT-01 | Phase 41 | Complete |
| SCRIPT-02 | Phase 42 | Complete |
| SCRIPT-03 | Phase 42 | Complete |
| SCRIPT-04 | Phase 42 | Complete |
| SCRIPT-05 | Phase 42 | Complete |
| SCRIPT-06 | Phase 41 | Complete |
| SCRIPT-07 | Phase 41 | Complete |
| SERIES-01 | Phase 43 | Complete |
| SERIES-02 | Phase 43 | Complete |
| SERIES-03 | Phase 43 | Complete |
| POLY-01 | Phase 44 | Complete |
| POLY-02 | Phase 44 | Complete |
| BIVAR-01 | Phase 45 | Complete |
| BIVAR-02 | Phase 45 | Complete |
| BIVAR-03 | Phase 45 | Complete |
| BIVAR-04 | Phase 45 | Complete |
| UTIL-01 | Phase 43 | Complete |
| UTIL-02 | Phase 43 | Complete |
| DOC-01 | Phase 46 | Complete |
| DOC-02 | Phase 46 | Complete |
| DOC-03 | Phase 46 | Complete |

**Coverage:**
- v3.0 requirements: 21 total
- Mapped to phases: 21
- Unmapped: 0

---
*Requirements defined: 2026-02-20*
*Last updated: 2026-02-20 after roadmap creation*
