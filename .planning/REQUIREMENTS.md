# Requirements: q-Kangaroo v4.0

**Defined:** 2026-02-21
**Core Value:** Every executable example in Garvan's qmaple.pdf runs correctly in the q-Kangaroo CLI without modification

## v4.0 Requirements

### Bug Fixes

- [ ] **FIX-01**: `aqprod(q,q,n)` in 3-arg mode computes the full finite polynomial `(q;q)_n` instead of truncating to O(q^n)
- [ ] **FIX-02**: `theta2(q,T)`, `theta3(q,T)`, `theta4(q,T)` accept Garvan's 2-arg form (variable + truncation order)
- [ ] **FIX-03**: `qfactor` displays results in q-product form `(1-q^a)(1-q^b)...` instead of raw struct
- [ ] **FIX-04**: `etamake` displays results in eta(k*tau) notation instead of raw struct
- [ ] **FIX-05**: `qfactor(f,T)` accepts Garvan's 2-arg signature (f + upper bound T) in addition to `qfactor(f,q,T)`

### Language Features

- [ ] **LANG-01**: User can use `"` (ditto operator) to reference the last computed result: `etamake(",q,100)`
- [ ] **LANG-02**: User can define lambda functions with arrow operator: `F := q -> theta3(q,500)/theta3(q^5,100)`
- [ ] **LANG-03**: User can call `min(a,b,c)` to compute the minimum of integer/rational arguments
- [ ] **LANG-04**: User can use fractional q-powers: `q^(1/4)`, `q^(1/3)`, `theta2(q,100)/q^(1/4)`
- [ ] **LANG-05**: User can write `option remember` before `local` in procedures (either order accepted)

### New Functions

- [ ] **FUNC-01**: User can call `jac2series(jacexpr, T)` to convert a Jacobi-type product into its theta-series expansion
- [ ] **FUNC-02**: User can call `radsimp(expr)` to simplify rational expressions involving series quotients
- [ ] **FUNC-03**: User can call `quinprod(z,q,prodid)` and `quinprod(z,q,seriesid)` to display the quintuple product identity in product and series forms
- [ ] **FUNC-04**: User can use `subs(X[1]=val1, X[2]=val2, ..., expr)` with indexed variables for multi-substitution (as output by findnonhom)

## Future Requirements

### Deferred from v3.0

- **BIVAR-07**: `zqfactor(f)` factors bivariate rational functions in (z, q) as q-products
- **MAPLE-02**: While-loops: `while cond do stmts od`
- **MAPLE-03**: `nops(list)`, `op(i, list)` list manipulation functions

## Out of Scope

| Feature | Reason |
|---------|--------|
| Full Maple language compatibility | Focused on qmaple.pdf tutorial examples, not general CAS |
| `RootOf` algebraic numbers | Only used in Exercise 4 hint, not in executable examples |
| Interactive debugger | Not needed for tutorial parity |
| `with(numtheory)` | Only L(m,p) needed, already implemented as legendre/L |
| `global` variable declarations | findnonhom already handles X variables internally |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FIX-01 | Phase 48 | Pending |
| FIX-02 | Phase 48 | Pending |
| FIX-03 | Phase 49 | Pending |
| FIX-04 | Phase 49 | Pending |
| FIX-05 | Phase 48 | Pending |
| LANG-01 | Phase 47 | Complete |
| LANG-02 | Phase 47 | Complete |
| LANG-03 | Phase 48 | Pending |
| LANG-04 | Phase 47 | Complete |
| LANG-05 | Phase 47 | Complete |
| FUNC-01 | Phase 50 | Pending |
| FUNC-02 | Phase 50 | Pending |
| FUNC-03 | Phase 50 | Pending |
| FUNC-04 | Phase 50 | Pending |

**Coverage:**
- v4.0 requirements: 14 total
- Mapped to phases: 14
- Unmapped: 0

---
*Requirements defined: 2026-02-21*
*Last updated: 2026-02-21 after roadmap creation*
