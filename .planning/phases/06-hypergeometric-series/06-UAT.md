---
status: complete
phase: 06-hypergeometric-series
source: 06-01-SUMMARY.md, 06-02-SUMMARY.md, 06-03-SUMMARY.md, 06-04-SUMMARY.md
started: 2026-02-14T13:15:00Z
updated: 2026-02-14T13:25:00Z
---

## Current Test

[testing complete]

## Tests

### 1. eval_phi Evaluation
expected: eval_phi correctly evaluates terminating and non-terminating _r phi_s series. 1phi0 matches q-binomial theorem, 2phi1 matches q-Gauss identity. QMonomial arithmetic works. 18 tests pass.
result: pass

### 2. Summation Formulas
expected: 5 classical summation formulas auto-detect and produce correct closed forms -- q-Gauss, q-Vandermonde (2 forms), q-Saalschutz, q-Kummer, q-Dixon. try_all_summations dispatches correctly. 8 tests pass.
result: pass

### 3. Transformation Formulas
expected: Heine's 3 transformations for 2phi1 verified by expanding both sides. All 3 forms produce equivalent series. Sears' 4phi3 transformation with permutation search. 8 tests pass.
result: pass

### 4. Watson and Bailey Transformations
expected: Watson detects very-well-poised 8phi7, reduces to 4phi3. Bailey produces closed form for DLMF 17.7.12 with q^2 base. Rejection tests for non-matching series. 4 Rust tests pass.
result: pass

### 5. Python Hypergeometric API
expected: phi(), psi(), try_summation(), heine1/2/3() callable from Python. test_hypergeometric_identity_verification passes end-to-end (q-Gauss from Python).
result: issue
reported: "doesn't work, phi not importable from qsymbolic"
severity: major

## Summary

total: 5
passed: 4
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "phi(), psi(), try_summation(), heine1/2/3() callable from Python"
  status: failed
  reason: "User reported: doesn't work, phi not importable from qsymbolic"
  severity: major
  test: 5
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
