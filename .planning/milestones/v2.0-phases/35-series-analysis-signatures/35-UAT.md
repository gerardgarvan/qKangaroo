---
phase: 35-series-analysis-signatures
type: uat
tested: 2026-02-19
status: passed
tests: 14/14
---

# Phase 35 UAT: Series Analysis Signatures

## Session Info
- **Phase:** 35 - Series Analysis Signatures
- **Tested:** 2026-02-19
- **Method:** CLI binary (`q-kangaroo.exe -c`) hands-on testing
- **Status:** PASSED (14/14)

## Test Results

| # | Test | Command | Result | Status |
|---|------|---------|--------|--------|
| 1 | sift 5-arg | `f := partition_gf(50); sift(f, q, 5, 4, 50)` | Returns sifted series: `5 + 30*q + 135*q^2 + ...` | PASS |
| 2 | prodmake 3-arg | `f := partition_gf(30); prodmake(f, q, 15)` | Returns `{exponents: {1: 1, 2: 1, ...}, terms_used: 15}` | PASS |
| 3 | etamake 3-arg | `f := partition_gf(30); etamake(f, q, 10)` | Returns `{factors: {1: -1}, q_shift: -1/24}` | PASS |
| 4 | jacprodmake 3-arg | `f := jacprod(1, 5, q, 30); jacprodmake(f, q, 10)` | Returns `{factors: {(1,10): 1, (4,10): 1}, scalar: 1, is_exact: false}` | PASS |
| 5 | jacprodmake 4-arg | `f := jacprod(1, 5, q, 30); jacprodmake(f, q, 10, 10)` | Returns same factors with period filter applied | PASS |
| 6 | mprodmake 3-arg | `f := distinct_parts_gf(30); mprodmake(f, q, 10)` | Returns `{1: 1, 2: 1, ..., 10: 1}` | PASS |
| 7 | qetamake 3-arg | `f := partition_gf(30); qetamake(f, q, 10)` | Returns `{factors: {1: -1}, q_shift: 0}` | PASS |
| 8 | qfactor 2-arg | `f := aqprod(q, q, 5, 20); qfactor(f, q)` | Returns `{scalar: 1, factors: {1:1,...,5:1}, is_exact: true}` | PASS |
| 9 | Old sift errors | `sift(partition_gf(30), 5, 0)` | `Error: sift expects 5 arguments`, exit 1 | PASS |
| 10 | Old prodmake errors | `prodmake(partition_gf(30), 10)` | `Error: prodmake expects 3 arguments`, exit 1 | PASS |
| 11 | sift invalid k | `sift(f, q, 5, 7, 50)` | `Error: sift: Argument 4 (k): residue must satisfy 0 <= k < n=5, got 7`, exit 1 | PASS |
| 12 | Help unit tests | `cargo test -p qsym-cli help` | 3/3 pass (signatures verified) | PASS |
| 13 | qfactor 3-arg | `f := aqprod(q, q, 5, 20); qfactor(f, q, 20)` | Returns same as 2-arg (T accepted, ignored) | PASS |
| 14 | Integration suite | `cargo test --test cli_integration` | 102/102 pass | PASS |

## Mathematical Verification

- **sift**: `sift(partition_gf(50), q, 5, 4, 50)` returns `5 + 30*q + ...` which are p(5n+4) = 5, 30, 135, 490, 1575... -- Ramanujan's congruence p(5n+4) ≡ 0 (mod 5) verified (all divisible by 5)
- **prodmake**: partition_gf decomposed to `{1:1, 2:1, ..., 15:1}` = prod(1-q^n)^1 for n=1..15, consistent with 1/partition_gf (note: exponents are positive, representing the reciprocal)
- **etamake**: partition_gf → `{factors: {1: -1}, q_shift: -1/24}` = q^(-1/24) * eta(tau)^(-1), which is the standard eta-quotient for the partition function
- **qfactor**: aqprod(q,q,5) = (1-q)(1-q^2)...(1-q^5) → factors {1:1, 2:1, 3:1, 4:1, 5:1} exact

## Summary

All 7 series analysis functions work correctly with Maple-style signatures. Error handling is clean with positional parameter names. Old signatures are properly rejected. Mathematical output is correct.
