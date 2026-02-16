---
phase: 13-polynomial-infrastructure
verified: 2026-02-15T20:00:00Z
status: passed
score: 5/5 must-haves verified
---

# Phase 13: Polynomial Infrastructure Verification Report

**Phase Goal:** Exact polynomial and rational function arithmetic over QRat is available as a foundation for algorithmic identity proving
**Verified:** 2026-02-15
**Status:** PASSED
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can construct QRatPoly from coefficients, perform add/sub/mul/div/rem, and get exact QRat results | VERIFIED | `QRatPoly` struct with `from_vec`, `from_i64_coeffs`, `monomial`, `linear`, `x()`, `constant()` constructors. Add/Sub/Mul/Neg traits (4 variants each). `div_rem`, `exact_div`, `pseudo_rem` all implemented. Horner `eval` returns exact QRat. 35 arithmetic tests + 10 constructor tests pass. |
| 2 | Polynomial GCD of two polynomials with rational coefficients returns the correct monic GCD without coefficient explosion | VERIFIED | `poly_gcd` in `gcd.rs` implements subresultant PRS with content extraction and primitive part reduction. Returns `make_monic()`. Degree-10 test with 3-digit coefficients confirms no explosion (coefficients stay under 15 digits). 8 GCD unit tests + integration tests pass. |
| 3 | Polynomial resultant correctly identifies when two polynomials share a common root | VERIFIED | `poly_resultant` in `gcd.rs` uses Euclidean algorithm over Q[x] with recursive formula. Returns zero iff shared root exists. Tests verify: `resultant((x-2)(x-3), (x-1)(x-2)) = 0`, `resultant(x^2+1, x^2-1) != 0`, known values for linear pairs. 7 resultant tests pass. |
| 4 | q-shift evaluation p(q^j * x) returns correct polynomial for arbitrary integer j | VERIFIED | `q_shift` and `q_shift_n` methods on `QRatPoly`. Evaluation identity `p.q_shift(q).eval(x) == p.eval(q*x)` verified for multiple test cases. Double shift equals `q_shift_n(2)`. Negative shift round-trip verified. `q_shift_n(q, 0)` returns original. 8 q-shift unit tests + 2 integration tests pass. |
| 5 | Rational functions (quotient of two polynomials) support arithmetic and automatic GCD-based simplification | VERIFIED | `QRatRationalFunc` in `ratfunc.rs` with invariants: lowest terms, monic denominator, nonzero denominator. `new()` calls `poly_gcd` for auto-reduction. Add/Sub/Mul/Div/Neg via both named methods and std::ops traits. Cross-cancellation in `rf_mul`. `q_shift`/`q_shift_n` shift both parts. `(x^2-1)/(x-1)` auto-reduces to `(x+1)/1`. 30 ratfunc unit tests + 6 integration tests pass. |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/qsym-core/src/poly/mod.rs` | QRatPoly struct, constructors, queries, content/primitive_part/make_monic, eval, Display, PartialEq/Eq, q_shift, q_shift_n | VERIFIED | 1168 lines. Full implementation with 37 unit tests + 17 integration tests. |
| `crates/qsym-core/src/poly/arithmetic.rs` | Add, Sub, Mul, Neg traits, scalar_mul, scalar_div, div_rem, exact_div, pseudo_rem | VERIFIED | 582 lines. All 4 operator variants per trait. 33 unit tests including ring axioms. |
| `crates/qsym-core/src/poly/gcd.rs` | poly_gcd (subresultant PRS), poly_resultant | VERIFIED | 401 lines. Subresultant PRS with content extraction. Euclidean resultant over Q[x]. 15 tests. |
| `crates/qsym-core/src/poly/ratfunc.rs` | QRatRationalFunc struct, auto-simplification, arithmetic, q_shift, Display, PartialEq | VERIFIED | 648 lines. Full implementation with cross-cancellation optimization. 30 tests. |
| `crates/qsym-core/src/lib.rs` | `pub mod poly` and re-exports | VERIFIED | Line 10: `pub mod poly;`, Line 21: `pub use poly::{QRatPoly, QRatRationalFunc, poly_gcd, poly_resultant};` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `poly/arithmetic.rs` | `poly/mod.rs` | `use super::QRatPoly` | WIRED | Line 6 of arithmetic.rs |
| `poly/mod.rs` | `number.rs` | `use crate::number::QRat` | WIRED | Line 13 of mod.rs |
| `lib.rs` | `poly/mod.rs` | `pub mod poly` | WIRED | Line 10 of lib.rs |
| `poly/gcd.rs` | `poly/mod.rs` | `use super::QRatPoly` | WIRED | Line 8 of gcd.rs |
| `poly/gcd.rs` | `poly/arithmetic.rs` | `pseudo_rem, scalar_div methods` | WIRED | Lines 90, 97, 134, 146 in gcd.rs |
| `poly/ratfunc.rs` | `poly/gcd.rs` | `poly_gcd for auto-simplification` | WIRED | Lines 7, 45, 115, 116 in ratfunc.rs |
| `poly/ratfunc.rs` | `poly/mod.rs` | `use super::QRatPoly` | WIRED | Line 8 of ratfunc.rs |
| `poly/ratfunc.rs` | `poly/arithmetic.rs` | `exact_div, Mul operations` | WIRED | Lines 46, 47, 117-121, etc. |

### Requirements Coverage

| Requirement | Status | Blocking Issue |
|-------------|--------|----------------|
| POLY-01: Dense univariate polynomial type with add/sub/mul/div/rem | SATISFIED | None |
| POLY-02: Polynomial GCD via subresultant PRS | SATISFIED | None |
| POLY-03: Polynomial resultant computation | SATISFIED | None |
| POLY-04: q-shift operations | SATISFIED | None |
| POLY-05: Rational function type with arithmetic and simplification | SATISFIED | None |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | - | - | - | No anti-patterns found in any poly module file |

### Human Verification Required

No items require human verification. All operations are pure mathematical computations verified by exact equality assertions in 144 automated tests. There are no visual, real-time, or external service dependencies.

### Test Summary

- **144 new poly module tests** (target was 40+)
- **722 total crate tests** (up from 578 before Phase 13)
- **0 failures** across the entire test suite
- **0 regressions** in existing tests

### Gaps Summary

No gaps found. All 5 success criteria are fully satisfied:

1. QRatPoly construction, arithmetic, div/rem, and exact evaluation all work correctly.
2. Subresultant PRS GCD returns monic results without coefficient explosion (verified on degree-10 polynomials with 3-digit coefficients).
3. Resultant correctly returns zero when polynomials share a common root, nonzero otherwise.
4. q-shift evaluation identity `p.q_shift(q).eval(x) == p.eval(q*x)` holds for all tested values, including negative shift round-trips.
5. QRatRationalFunc auto-reduces on construction, supports full arithmetic with cross-cancellation, and satisfies field axioms (verified by integration tests).

---

_Verified: 2026-02-15_
_Verifier: Claude (gsd-verifier)_
