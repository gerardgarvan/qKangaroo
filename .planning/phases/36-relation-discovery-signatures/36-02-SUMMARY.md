---
phase: 36-relation-discovery-signatures
plan: 02
subsystem: qsym-cli/eval
tags: [maple-compat, relation-discovery, dispatch, formatting, garvan-signatures]
dependency_graph:
  requires: [generate_monomials-pub, generate_nonhom_monomials, findcong_garvan]
  provides: [garvan-dispatch-11-functions, format-helpers, sl-validation]
  affects: [36-03]
tech_stack:
  added: []
  patterns: [SL-label-formatting, X[i]-auto-labels, findcong-auto-scan-dispatch]
key_files:
  created: []
  modified:
    - crates/qsym-cli/src/eval.rs
decisions:
  - SL label validation uses strict match (labels.len() == candidates.len())
  - validate_unique_labels uses HashSet for O(n) duplicate detection
  - is_prime uses trial division (6k+-1) for small p validation
  - format_linear_combo handles 1/-1/0 coefficients with label-only display
  - format_polynomial_expr builds monomial strings from exponent tuples
  - format_findpoly_result uses X/Y variable names for bivariate polynomials
  - default_labels generates X[1]..X[k] matching Garvan convention
  - findmaxind returns 1-based indices matching Garvan convention
  - findcong dispatches to findcong_garvan with auto-scan algorithm
  - findpoly uses fixed topshift=10 matching Garvan's dim2:=dim1+10
metrics:
  duration: 5min
  completed: 2026-02-19T22:26:16Z
  tasks: 1
  tests_added: 5
  tests_updated: 5
  total_cli_tests: 385
---

# Phase 36 Plan 02: Garvan-Compatible Relation Discovery Dispatch Summary

Rewrote all 11 relation discovery function dispatch blocks in eval.rs to match Garvan's exact Maple signatures, with formatted output using symbolic labels and comprehensive validation.

## What Changed

### Task 1: Add formatting helpers and rewrite dispatch for all 11 functions

**Part A: 9 helper functions added**
- `extract_symbol_list`: Extract list of Symbol values as Vec<String> for SL parameters
- `validate_unique_labels`: HashSet-based duplicate label detection with descriptive error
- `is_prime`: Trial division primality test (6k+-1 pattern) for modp validation
- `format_linear_combo`: QRat coefficients with symbolic labels ("12*F1 + 13*F2")
- `format_linear_combo_modp`: i64 coefficients with symbolic labels for modular output
- `format_polynomial_expr`: QRat coefficients with monomial terms ("3*X[1]^2*X[2]")
- `format_polynomial_expr_modp`: i64 coefficients for modular polynomial output
- `format_findpoly_result`: Bivariate polynomial in X, Y variables
- `default_labels`: Generate X[1]..X[k] labels matching Garvan convention

**Part B: 11 dispatch blocks rewritten**
1. findlincombo: 3-arg -> 5-arg (f, L, SL, q, topshift). Returns Value::String with SL labels.
2. findhomcombo: 4-arg -> 5-arg (f, L, q, n, topshift). No SL, uses X[i] labels.
3. findnonhomcombo: 4-arg -> 5-arg (f, L, q, n, topshift). No SL, uses X[i] labels.
4. findlincombomodp: 4-arg -> 6-arg (f, L, SL, p, q, topshift). Prime validation, p before q.
5. findhomcombomodp: 5-arg -> 6-arg (f, L, p, q, n, topshift). No SL, prime validation.
6. findhom: 3-arg -> 4-arg (L, q, n, topshift). Returns List of String expressions.
7. findnonhom: 3-arg -> 4-arg (L, q, n, topshift). Returns List of String expressions.
8. findhommodp: 4-arg -> 5-arg (L, p, q, n, topshift). Prime validation, p before q.
9. findmaxind: 2-arg (L, T) unchanged. Now returns 1-based indices.
10. findpoly: 5-arg -> 5-6 arg (x, y, q, dx, dy, [check]). Fixed topshift=10.
11. findcong: 2-arg -> 2-4 arg (QS, T, [LM], [XSET]). Uses findcong_garvan auto-scan.

**Part C: Signature strings updated** for all 11 functions in get_signature.

**Part D: HashSet import** added to std::collections.

**Part E: Tests updated**
- 5 existing tests rewritten for new Maple signatures
- 5 new tests: duplicate SL errors, non-prime p errors, findcong with LM, findhomcombo maple-style, findhommodp p-before-q

## Deviations from Plan

None - plan executed exactly as written.

## Commits

| # | Hash | Message |
|---|------|---------|
| 1 | 0baf750 | feat(36-02): rewrite 11 relation discovery dispatch blocks for Garvan Maple compat |

## Verification

- `cargo test -p qsym-cli --lib` -- 385 passed, 0 failed (380 existing + 5 net new)
- All 11 function dispatch blocks match Garvan's verified signatures
- SL-based output uses symbolic labels; non-SL functions use X[i] labels
- findcong uses findcong_garvan auto-scan algorithm and [B, A, R] triple format
- Old calling conventions produce WrongArgCount errors (no backward compat per Phase 35 decision)
- Duplicate SL labels produce descriptive errors
- Non-prime p in modp functions produces descriptive errors
- No-solution cases print messages and return Value::None (not errors)

## Self-Check: PASSED

- [x] crates/qsym-cli/src/eval.rs exists
- [x] Commit 0baf750 found
- [x] 385 tests pass
