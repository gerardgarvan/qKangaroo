---
phase: 36-relation-discovery-signatures
plan: 01
subsystem: qsym-core/qseries/relations
tags: [findcong, garvan-algorithm, monomial-generators, trial-factoring]
dependency_graph:
  requires: []
  provides: [generate_monomials-pub, generate_nonhom_monomials, findcong_garvan, trial_factor]
  affects: [36-02, 36-03]
tech_stack:
  added: []
  patterns: [GCD-based-congruence-discovery, trial-division-factoring]
key_files:
  created: []
  modified:
    - crates/qsym-core/src/qseries/relations.rs
    - crates/qsym-core/src/qseries/mod.rs
decisions:
  - findcong_garvan uses rug::Integer GCD with abs_ref for sign safety
  - trial_factor returns Vec<(i64, u32)> for simple prime-power iteration
  - generate_nonhom_monomials delegates to generate_monomials for each degree level
  - Test truncation uses t=99 with partition_gf(100) to respect O(q^100) bound
metrics:
  duration: 4min
  completed: 2026-02-19T22:18:28Z
  tasks: 2
  tests_added: 6
  total_core_tests: 281
---

# Phase 36 Plan 01: Pub Monomial Generators + findcong_garvan Summary

Public monomial generators and Garvan's auto-scan findcong algorithm with GCD-based trial factoring for automatic congruence discovery.

## What Changed

### Task 1: Make monomial generators pub and add findcong_garvan
- Made `generate_monomials` public (was private) for CLI polynomial output formatting
- Added `generate_nonhom_monomials(k, max_degree)` for non-homogeneous monomial enumeration
- Added `trial_factor(n)` for prime-power factoring via trial division up to sqrt(n)
- Added `findcong_garvan(f, t, lm, xset)` implementing Garvan's algorithm:
  - Auto-scans moduli 2..=lm (default: floor(sqrt(t)))
  - Extracts coefficient subsequences f(m*n + r) for each (m, r) pair
  - Computes GCD of nonzero coefficients using rug::Integer
  - Factors GCD via trial_factor to discover all prime-power divisors
  - Filters results against user-supplied exclusion set (xset)
- Updated mod.rs re-exports: `generate_monomials`, `generate_nonhom_monomials`, `findcong_garvan`

### Task 2: Unit tests for findcong_garvan and trial_factor
- `test_trial_factor_basic`: 7 assertions covering edge cases (0, 1) and composites (12, 25, 100)
- `test_generate_monomials_pub`: verifies public access, correct count and content for k=2,3
- `test_generate_nonhom_monomials`: verifies degree 0..max aggregation (3 and 6 monomials)
- `test_findcong_garvan_partition_congruences`: discovers Ramanujan's p(5n+4) mod 5 and p(7n+5) mod 7
- `test_findcong_garvan_with_lm`: confirms lm=5 excludes modulus 7 results
- `test_findcong_garvan_with_xset`: confirms xset={5} excludes p(5n+4) mod 5 but keeps p(7n+5) mod 7

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Partition GF truncation boundary**
- **Found during:** Task 2
- **Issue:** `partition_gf(100)` produces series with truncation_order=100, meaning coeff(100) panics. Tests used t=100 which accessed index 100.
- **Fix:** Changed test truncation parameter from t=100 to t=99 to stay within valid coefficient range.
- **Files modified:** crates/qsym-core/src/qseries/relations.rs
- **Commit:** e7e3f63

**2. [Rule 3 - Blocking] Type inference ambiguity with rug::Complex**
- **Found during:** Task 2
- **Issue:** `assert_eq!(trial_factor(1), vec![])` failed to compile because rug implements `PartialEq<Complex> for (i64, u32)`, creating type ambiguity for empty vecs.
- **Fix:** Added explicit type annotation `let empty: Vec<(i64, u32)> = vec![];` for empty vec comparisons.
- **Files modified:** crates/qsym-core/src/qseries/relations.rs
- **Commit:** e7e3f63

**3. [Rule 3 - Blocking] Format string brace escaping**
- **Found during:** Task 2
- **Issue:** `"xset={5}"` in assert message was interpreted as format positional arg `{5}`.
- **Fix:** Escaped braces as `"xset={{5}}"`.
- **Files modified:** crates/qsym-core/src/qseries/relations.rs
- **Commit:** e7e3f63

## Commits

| # | Hash | Message |
|---|------|---------|
| 1 | 4916c17 | feat(36-01): make monomial generators pub and add findcong_garvan |
| 2 | e7e3f63 | test(36-01): add unit tests for findcong_garvan and trial_factor |

## Verification

- `cargo test -p qsym-core --lib` -- 281 passed, 0 failed (275 existing + 6 new)
- `generate_monomials` and `generate_nonhom_monomials` are pub and re-exported from `qseries`
- `findcong_garvan` correctly discovers Ramanujan's partition congruences from partition_gf
- `trial_factor` correctly handles primes, composites, and edge cases

## Self-Check: PASSED

- [x] crates/qsym-core/src/qseries/relations.rs exists
- [x] crates/qsym-core/src/qseries/mod.rs exists
- [x] .planning/phases/36-relation-discovery-signatures/36-01-SUMMARY.md exists
- [x] Commit 4916c17 found (feat: pub monomial generators + findcong_garvan)
- [x] Commit e7e3f63 found (test: 6 unit tests)
