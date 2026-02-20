# v2.0 Requirements: Maple Compatibility

## Symbolic Variables

- [x] **SYM-01**: Bare variable names (f, x, a, etc.) evaluate to a Symbol value without requiring `:=` declaration -- undefined names do not error
- [x] **SYM-02**: `q` is recognized as the built-in expansion variable and can be passed as a function parameter (e.g., `etaq(q, 1, 20)`)
- [x] **SYM-03**: Monomial expressions like `q^2`, `q^3` can be used as function arguments where Garvan accepts q-monomials
- [x] **SYM-04**: Variables assigned via `:=` continue to work as before; assigned values take precedence over bare symbol evaluation

## Function Signature Compatibility

### Products & Theta (Group 1)

- [x] **SIG-01**: `aqprod(a, q, n)` accepts Garvan's 3-arg signature where `a` is a q-monomial, `q` is the variable, `n` is the order -- finite products return exact polynomials, infinite products require truncation context
- [x] **SIG-02**: `etaq(q, a, T)` matches Garvan's signature -- `q` is the variable, `a` is a positive integer (delta), `T` is truncation order
- [x] **SIG-03**: `jacprod(a, b, q, T)` matches Garvan's 4-arg signature -- `a`, `b` positive integers, `q` variable, `T` truncation
- [x] **SIG-04**: `tripleprod(a, q_power, T)` matches Garvan's signature with q-monomial first argument
- [x] **SIG-05**: `quinprod(a, q_power, T)` matches Garvan's signature with q-monomial first argument
- [x] **SIG-06**: `winquist(a, b, q, T)` matches Garvan's signature
- [x] **SIG-07**: `qbin(n, k, q, T)` matches Garvan's signature with explicit `q` and `T`

### Series Analysis (Group 2)

- [x] **SIG-08**: `sift(s, q, n, k, T)` matches Garvan's 5-arg signature -- `s` is series, `q` variable, `n` modulus, `k` residue, `T` truncation
- [x] **SIG-09**: `prodmake(f, q, T)` matches Garvan's 3-arg signature
- [x] **SIG-10**: `etamake(f, q, T)` matches Garvan's 3-arg signature
- [x] **SIG-11**: `jacprodmake(f, q, T)` and `jacprodmake(f, q, T, P)` match Garvan's 3-arg and 4-arg signatures
- [x] **SIG-12**: `mprodmake(f, q, T)` matches Garvan's 3-arg signature
- [x] **SIG-13**: `qetamake(f, q, T)` matches Garvan's 3-arg signature
- [x] **SIG-14**: `qfactor(f, q)` matches Garvan's signature with explicit `q`

### Relation Discovery (Group 3)

- [x] **SIG-15**: `findlincombo(f, L, SL, q, topshift)` matches Garvan's 5-arg signature including symbolic label list `SL`
- [x] **SIG-16**: `findhomcombo(f, L, q, n, topshift)` matches Garvan's actual signature (no SL -- verified from Maple source)
- [x] **SIG-17**: `findnonhomcombo(f, L, q, n, topshift)` matches Garvan's actual signature (no SL -- verified from Maple source)
- [x] **SIG-18**: `findlincombomodp(f, L, SL, p, q, topshift)` matches Garvan's signature (p before q -- verified from Maple source)
- [x] **SIG-19**: `findhomcombomodp(f, L, p, q, n, topshift)` matches Garvan's actual signature (no SL, p before q)
- [x] **SIG-20**: `findhom(L, q, n, topshift)` matches Garvan's signature
- [x] **SIG-21**: `findnonhom(L, q, n, topshift)` matches Garvan's signature
- [x] **SIG-22**: `findhommodp(L, p, q, n, topshift)` matches Garvan's signature (p before q -- verified from Maple source)
- [x] **SIG-23**: `findmaxind(L, T)` matches Garvan's signature (2 args, no q -- verified from Maple docs)
- [x] **SIG-24**: `findpoly(x, y, q, dx, dy, [check])` matches Garvan's signature (optional check, not topshift)
- [x] **SIG-25**: `findcong(QS, T)` and `findcong(QS, T, LM)` and `findcong(QS, T, LM, XSET)` match Garvan's overloaded signatures

### Partition Functions

- [x] **SIG-26**: `numbpart(n)` is the primary name for partition counting (matching Maple), with `partition_count` as alias

## New Functions

- [x] **NEW-01**: `theta(z, q, T)` -- general theta function returning sum(z^i * q^(i^2), i=-T..T) with numeric, q-monomial, and symbol z handling
- [x] **NEW-02**: `jac2prod(JP, q, T)` -- convert Jacobi product expression to explicit product notation, prints and returns FPS
- [x] **NEW-03**: `jac2series(JP, q, T)` -- convert Jacobi product expression to q-series, prints and returns FPS
- [x] **NEW-04**: `qs2jaccombo(f, q, T)` -- decompose q-series into linear combination of Jacobi products via jacprodmake + findlincombo
- [x] **NEW-05**: `checkmult(QS, T)` and `checkmult(QS, T, 'yes')` -- check if q-series coefficients are multiplicative; optional 3rd arg prints all failures instead of stopping at first
- [x] **NEW-06**: `checkprod(f, M, Q)` -- check if q-series is a "nice" formal product using prodmake; M is max absolute exponent threshold, Q is truncation order; returns [a, 1] for nice product, [a, max_exp] otherwise
- [x] **NEW-07**: `lqdegree0(qexp)` -- lowest q-degree of a single monomial term (1 arg, FPS only); Garvan-compatible alias for lqdegree
- [ ] **NEW-08**: `zqfactor(F, z, q, N)` and `zqfactor(F, z, q, N, buglim)` -- factor bivariate (z,q)-series into product of (1 - c*z^i*q^j) factors via greedy iterative algorithm; buglim defaults to 1000
- [x] **NEW-09**: `findprod(FL, T, M, Q)` -- exhaustive search over integer coefficient vectors |c_i| <= T for linear combinations of FL that are "nice products" per checkprod(combo, M, Q); returns list of [valuation, coefficient_vector] pairs

## Output & Display

- [x] **OUT-01**: Relation discovery functions print results using symbolic labels (SL for findlincombo/findlincombomodp, X[i] auto-labels for others) matching Maple's output format
- [x] **OUT-02**: `findcong` output format matches Garvan's `[B, A, R]` triple format
- [x] **OUT-03**: Series display uses Maple-style polynomial ordering when appropriate

## Documentation

- [ ] **DOC-01**: PDF reference manual updated -- all function signatures reflect Maple-compatible calling conventions, new functions documented with formal math and examples
- [ ] **DOC-02**: REPL help system updated -- per-function help text reflects new signatures, new functions included in help categories
- [ ] **DOC-03**: Tab completion updated -- new functions included, `numbpart` added as primary name
- [ ] **DOC-04**: Python API docstrings updated to reflect any signature changes propagated to the Python layer
- [ ] **DOC-05**: Maple migration guide updated -- side-by-side examples now show identical syntax (no translation needed for qseries functions)
- [ ] **DOC-06**: README quick start examples updated if any function signatures changed

## Backward Compatibility

- [x] **COMPAT-01**: Existing v1.x function signatures continue to work as aliases (no breaking changes for users of current calling conventions)
- [x] **COMPAT-02**: All existing tests pass with no regressions

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SYM-01 | Phase 33 | Complete |
| SYM-02 | Phase 33 | Complete |
| SYM-03 | Phase 33 | Complete |
| SYM-04 | Phase 33 | Complete |
| SIG-01 | Phase 34 | Complete |
| SIG-02 | Phase 34 | Complete |
| SIG-03 | Phase 34 | Complete |
| SIG-04 | Phase 34 | Complete |
| SIG-05 | Phase 34 | Complete |
| SIG-06 | Phase 34 | Complete |
| SIG-07 | Phase 34 | Complete |
| SIG-08 | Phase 35 | Complete |
| SIG-09 | Phase 35 | Complete |
| SIG-10 | Phase 35 | Complete |
| SIG-11 | Phase 35 | Complete |
| SIG-12 | Phase 35 | Complete |
| SIG-13 | Phase 35 | Complete |
| SIG-14 | Phase 35 | Complete |
| SIG-15 | Phase 36 | Complete |
| SIG-16 | Phase 36 | Complete |
| SIG-17 | Phase 36 | Complete |
| SIG-18 | Phase 36 | Complete |
| SIG-19 | Phase 36 | Complete |
| SIG-20 | Phase 36 | Complete |
| SIG-21 | Phase 36 | Complete |
| SIG-22 | Phase 36 | Complete |
| SIG-23 | Phase 36 | Complete |
| SIG-24 | Phase 36 | Complete |
| SIG-25 | Phase 36 | Complete |
| SIG-26 | Phase 34 | Complete |
| NEW-01 | Phase 37 | Complete |
| NEW-02 | Phase 37 | Complete |
| NEW-03 | Phase 37 | Complete |
| NEW-04 | Phase 37 | Complete |
| NEW-05 | Phase 38 | Complete |
| NEW-06 | Phase 38 | Complete |
| NEW-07 | Phase 38 | Complete |
| NEW-08 | Deferred | Pending (requires bivariate infrastructure) |
| NEW-09 | Phase 38 | Complete |
| OUT-01 | Phase 36 | Complete |
| OUT-02 | Phase 36 | Complete |
| OUT-03 | Phase 39 | Complete |
| DOC-01 | Phase 40 | Pending |
| DOC-02 | Phase 40 | Pending |
| DOC-03 | Phase 40 | Pending |
| DOC-04 | Phase 40 | Pending |
| DOC-05 | Phase 40 | Pending |
| DOC-06 | Phase 40 | Pending |
| COMPAT-01 | Phase 39 | Complete |
| COMPAT-02 | Phase 39 | Complete |
