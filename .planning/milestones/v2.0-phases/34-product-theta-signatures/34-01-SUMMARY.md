---
phase: 34-product-theta-signatures
plan: 01
subsystem: qsym-cli/eval
tags: [maple-compat, dispatch, qbin, etaq, jacprod, tripleprod, quinprod, winquist]
dependency_graph:
  requires: [33-03]
  provides: [maple-style-product-dispatch]
  affects: [eval.rs]
tech_stack:
  added: []
  patterns: [multi-form-dispatch, monomial-extraction, list-argument-validation]
key_files:
  modified:
    - crates/qsym-cli/src/eval.rs
decisions:
  - "jacprod Maple-style uses JAC(a,b)/JAC(b,3b) per Garvan source, distinct from legacy JAC(a,b)"
  - "qbin Garvan form uses tight truncation order then re-wraps with POLYNOMIAL_ORDER sentinel to avoid O(q^1B) computation"
  - "etaq multi-delta validates non-empty list and positive deltas using EvalError::Other"
  - "Used arithmetic::invert + mul for series division (no arithmetic::div exists)"
metrics:
  duration: 12min
  completed: 2026-02-19T19:51:12Z
  tasks: 2
  files: 1
  tests_added: 9
  total_tests: 374 unit + 72 integration
---

# Phase 34 Plan 01: Product & Theta Maple-Style Dispatch Summary

Maple-style dispatch paths for 6 product/theta functions using first-arg type detection and arg count disambiguation, with etaq multi-delta list support.

## What Was Done

### Task 1: jacprod, tripleprod, quinprod, winquist (f1882be)

Added Maple-style if/else branches above legacy paths for four functions:

- **jacprod(a, b, q, T)**: 4-arg form with Symbol at position 2. Computes `JAC(a,b) / JAC(b,3b)` per Garvan's exact formula using `arithmetic::invert` + `arithmetic::mul`. Legacy 3-arg `jacprod(a, b, order)` unchanged.
- **tripleprod(z, q, T)**: 3-arg form with Series/Symbol first arg. Extracts monomial via `extract_monomial_from_arg`, dispatches to `qseries::tripleprod`. Legacy 4-arg form unchanged.
- **quinprod(z, q, T)**: Same pattern as tripleprod. 3-arg Maple form, 4-arg legacy.
- **winquist(a, b, q, T)**: 4-arg form with Symbol at position 2. Extracts two monomials for a and b. Legacy 7-arg form unchanged.

Updated `get_signature` for all four functions to show both forms.

### Task 2: qbin and etaq multi-delta (0d8f31f)

- **qbin(q, m, n)**: Garvan's 3-arg form with Symbol first. Uses tight truncation order `m*(n-m)+1` then re-wraps result with `POLYNOMIAL_ORDER` sentinel via `FormalPowerSeries::from_coeffs` for exact polynomial display.
- **qbin(n, k, q, T)**: 4-arg form with Symbol at position 2 and explicit truncation.
- **etaq(q, [deltas], T)**: Multi-delta list form. Detects `Value::List` at position 1, validates non-empty list and positive deltas, multiplies individual `qseries::etaq(d, 1, sym, order)` calls.
- Updated `get_signature` for qbin and etaq.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] arithmetic::div does not exist**
- **Found during:** Task 1 (jacprod Maple dispatch)
- **Issue:** Plan referenced `arithmetic::div` but only `arithmetic::invert` and `arithmetic::mul` exist
- **Fix:** Used `arithmetic::invert(&jac_b3b)` followed by `arithmetic::mul(&jac_ab, &inv_b3b)`
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Commit:** f1882be

**2. [Rule 1 - Bug] POLYNOMIAL_ORDER sentinel causes qbin to hang**
- **Found during:** Task 2 (qbin Garvan dispatch)
- **Issue:** Passing `POLYNOMIAL_ORDER = 1_000_000_000` as truncation order to `qseries::qbin` caused the invert operation to attempt computing 1 billion coefficients
- **Fix:** Computed tight upper bound `m*(n-m)+1` for actual polynomial degree, computed qbin with tight order, then re-wrapped coefficients with POLYNOMIAL_ORDER sentinel using `FormalPowerSeries::from_coeffs`
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Commit:** 0d8f31f

**3. [Rule 3 - Blocking] EvalError::Generic does not exist**
- **Found during:** Task 2 (etaq multi-delta)
- **Issue:** Plan referenced `EvalError::Generic` but the enum has `EvalError::Other(String)` instead
- **Fix:** Used `EvalError::Other(format!(...))` for validation errors
- **Files modified:** crates/qsym-cli/src/eval.rs
- **Commit:** 0d8f31f

## Test Results

- 374 unit tests passing (369 existing + 5 new in Task 1 + 5 new in Task 2, minus 1 shared helper = 374)
- 72 integration tests passing
- Zero regressions

## Self-Check: PASSED

- [x] crates/qsym-cli/src/eval.rs exists
- [x] Commit f1882be exists
- [x] Commit 0d8f31f exists
- [x] 34-01-SUMMARY.md exists
