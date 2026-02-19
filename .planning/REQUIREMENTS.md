# v2.0 Requirements: Maple Compatibility

## Symbolic Variables

- [ ] **SYM-01**: Bare variable names (f, x, a, etc.) evaluate to a Symbol value without requiring `:=` declaration — undefined names do not error
- [ ] **SYM-02**: `q` is recognized as the built-in expansion variable and can be passed as a function parameter (e.g., `etaq(q, 1, 20)`)
- [ ] **SYM-03**: Monomial expressions like `q^2`, `q^3` can be used as function arguments where Garvan accepts q-monomials
- [ ] **SYM-04**: Variables assigned via `:=` continue to work as before; assigned values take precedence over bare symbol evaluation

## Function Signature Compatibility

### Products & Theta (Group 1)

- [ ] **SIG-01**: `aqprod(a, q, n)` accepts Garvan's 3-arg signature where `a` is a q-monomial, `q` is the variable, `n` is the order — finite products return exact polynomials, infinite products require truncation context
- [ ] **SIG-02**: `etaq(q, a, T)` matches Garvan's signature — `q` is the variable, `a` is a positive integer (delta), `T` is truncation order
- [ ] **SIG-03**: `jacprod(a, b, q, T)` matches Garvan's 4-arg signature — `a`, `b` positive integers, `q` variable, `T` truncation
- [ ] **SIG-04**: `tripleprod(a, q_power, T)` matches Garvan's signature with q-monomial first argument
- [ ] **SIG-05**: `quinprod(a, q_power, T)` matches Garvan's signature with q-monomial first argument
- [ ] **SIG-06**: `winquist(a, b, q, T)` matches Garvan's signature
- [ ] **SIG-07**: `qbin(n, k, q, T)` matches Garvan's signature with explicit `q` and `T`

### Series Analysis (Group 2)

- [ ] **SIG-08**: `sift(s, q, n, k, T)` matches Garvan's 5-arg signature — `s` is series, `q` variable, `n` modulus, `k` residue, `T` truncation
- [ ] **SIG-09**: `prodmake(f, q, T)` matches Garvan's 3-arg signature
- [ ] **SIG-10**: `etamake(f, q, T)` matches Garvan's 3-arg signature
- [ ] **SIG-11**: `jacprodmake(f, q, T)` and `jacprodmake(f, q, T, P)` match Garvan's 3-arg and 4-arg signatures
- [ ] **SIG-12**: `mprodmake(f, q, T)` matches Garvan's 3-arg signature
- [ ] **SIG-13**: `qetamake(f, q, T)` matches Garvan's 3-arg signature
- [ ] **SIG-14**: `qfactor(f, q)` matches Garvan's signature with explicit `q`

### Relation Discovery (Group 3)

- [ ] **SIG-15**: `findlincombo(f, L, SL, q, topshift)` matches Garvan's 5-arg signature including symbolic label list `SL`
- [ ] **SIG-16**: `findhomcombo(f, L, SL, q, n, topshift)` matches Garvan's signature with `SL` labels and degree `n`
- [ ] **SIG-17**: `findnonhomcombo(f, L, SL, q, n, topshift)` matches Garvan's signature with `SL` labels
- [ ] **SIG-18**: `findlincombomodp(f, L, SL, q, p, topshift)` matches Garvan's signature with `SL` labels and prime `p`
- [ ] **SIG-19**: `findhomcombomodp(f, L, SL, q, p, n, topshift)` matches Garvan's signature
- [ ] **SIG-20**: `findhom(L, q, n, topshift)` matches Garvan's signature
- [ ] **SIG-21**: `findnonhom(L, q, n, topshift)` matches Garvan's signature
- [ ] **SIG-22**: `findhommodp(L, q, p, n, topshift)` matches Garvan's signature
- [ ] **SIG-23**: `findmaxind(L, q, topshift)` matches Garvan's signature
- [ ] **SIG-24**: `findpoly(f, g, q, dx, dy, topshift)` matches Garvan's signature
- [ ] **SIG-25**: `findcong(QS, T)` and `findcong(QS, T, LM)` and `findcong(QS, T, LM, XSET)` match Garvan's overloaded signatures

### Partition Functions

- [ ] **SIG-26**: `numbpart(n)` is the primary name for partition counting (matching Maple), with `partition_count` as alias

## New Functions

- [ ] **NEW-01**: `theta(z, q, T)` — general theta function returning sum(z^i * q^(i^2), i=-T..T)
- [ ] **NEW-02**: `jac2prod(JP, q, T)` — convert Jacobi product expression to q-product form
- [ ] **NEW-03**: `jac2series(JP, q, T)` — convert Jacobi product expression to q-series
- [ ] **NEW-04**: `qs2jaccombo(f, q, T)` — convert sum of q-series to sum of jacprods
- [ ] **NEW-05**: `checkmult(f, q, T)` — check if q-series coefficients are multiplicative
- [ ] **NEW-06**: `checkprod(f, q, T)` — validate that q-series represents a well-formed product
- [ ] **NEW-07**: `lqdegree0(f, q)` — lowest degree in q (monomial version, distinct from lqdegree)
- [ ] **NEW-08**: `zqfactor(f, z, q)` — factor (z,q)-series into (z,q)-product form
- [ ] **NEW-09**: `findprod(L, q, maxcoeff, maxexp)` — find product identity matching Garvan's signature

## Output & Display

- [ ] **OUT-01**: Relation discovery functions print results using symbolic labels (SL parameter) matching Maple's output format (e.g., "12 F1 + 13 F2")
- [ ] **OUT-02**: `findcong` output format matches Garvan's `[B, A, R]` triple format
- [ ] **OUT-03**: Series display uses Maple-style polynomial ordering when appropriate

## Documentation

- [ ] **DOC-01**: PDF reference manual updated — all function signatures reflect Maple-compatible calling conventions, new functions documented with formal math and examples
- [ ] **DOC-02**: REPL help system updated — per-function help text reflects new signatures, new functions included in help categories
- [ ] **DOC-03**: Tab completion updated — new functions included, `numbpart` added as primary name
- [ ] **DOC-04**: Python API docstrings updated to reflect any signature changes propagated to the Python layer
- [ ] **DOC-05**: Maple migration guide updated — side-by-side examples now show identical syntax (no translation needed for qseries functions)
- [ ] **DOC-06**: README quick start examples updated if any function signatures changed

## Backward Compatibility

- [ ] **COMPAT-01**: Existing v1.x function signatures continue to work as aliases (no breaking changes for users of current calling conventions)
- [ ] **COMPAT-02**: All existing tests pass with no regressions

## Traceability

*Filled by roadmap*

