---
phase: 36-relation-discovery-signatures
plan: 03
subsystem: qsym-cli/help, qsym-cli/tests
tags: [maple-compat, relation-discovery, help-text, integration-tests, garvan-signatures]
dependency_graph:
  requires: [garvan-dispatch-11-functions, format-helpers, sl-validation]
  provides: [help-text-12-functions, integration-tests-13]
  affects: []
tech_stack:
  added: []
  patterns: [temp-script-integration-tests, colon-terminated-multiline-scripts]
key_files:
  created: []
  modified:
    - crates/qsym-cli/src/help.rs
    - crates/qsym-cli/tests/cli_integration.rs
decisions:
  - Help examples use two-line assign-then-call format matching Maple documentation style
  - findcong integration tests use partition_gf(201) with T=200 to avoid boundary access error
  - Script-based integration tests use colon terminators for multi-statement separation
  - findhom test uses theta functions to fourth power (theta2^4 starts at q^1 giving X[2] relation)
metrics:
  duration: 6min
  completed: 2026-02-19T22:34:08Z
  tasks: 2
  tests_added: 13
  tests_updated: 0
  total_cli_tests: 500
---

# Phase 36 Plan 03: Help Text and Integration Tests for Relation Discovery Summary

Updated help entries for 11 relation discovery functions to show Garvan's exact Maple signatures, and added 13 CLI integration tests verifying end-to-end dispatch through the actual binary.

## What Changed

### Task 1: Update help text for all 12 relation discovery functions

**Category listing updated:** The Relations section in general_help() now reflects the Garvan-compatible descriptions (e.g., "find f as linear combination of L using SL labels" instead of generic "find linear combination of candidates matching target").

**11 FUNC_HELP entries rewritten:**
1. findlincombo: `(f, L, SL, q, topshift)` -- SL labels, Garvan 5-arg signature
2. findhomcombo: `(f, L, q, n, topshift)` -- X[i] auto-labels, no SL
3. findnonhomcombo: `(f, L, q, n, topshift)` -- X[i] auto-labels, no SL
4. findlincombomodp: `(f, L, SL, p, q, topshift)` -- SL labels, p before q
5. findhomcombomodp: `(f, L, p, q, n, topshift)` -- X[i] labels, p before q
6. findhom: `(L, q, n, topshift)` -- X[i] labels, polynomial relations
7. findnonhom: `(L, q, n, topshift)` -- X[i] labels, degree-<=n
8. findhommodp: `(L, p, q, n, topshift)` -- p before q
9. findmaxind: `(L, T)` -- 2-arg, 1-based indices
10. findcong: `(QS, T) or (QS, T, LM) or (QS, T, LM, XSET)` -- auto-scan overloads
11. findpoly: `(x, y, q, dx, dy) or (x, y, q, dx, dy, check)` -- optional check

**findprod:** Left unchanged (not part of Phase 36 scope).

**All 18 help-related unit tests pass** (including func_help_count=81 and every_canonical_function_has_help_entry).

### Task 2: Add 13 CLI integration tests

All tests use the established `run()` and `write_temp_script()` helpers, running the actual `q-kangaroo` binary as a subprocess.

**Tests added:**
1. `findlincombo_maple_style` -- SL labels in output (F1 from f=1*f+0*g)
2. `findlincombo_not_found` -- "NOT A LINEAR COMBO" message, exit 0
3. `findlincombo_duplicate_sl_error` -- "duplicate label" error
4. `findlincombo_old_signature_error` -- "expects 5 arguments" for old 3-arg form
5. `findlincombomodp_maple_style` -- p before q with SL labels
6. `findlincombomodp_non_prime_error` -- "not prime" error for p=4
7. `findhom_maple_style` -- X[i] labels in output
8. `findhommodp_p_before_q` -- exit 0 with p before q ordering
9. `findmaxind_two_args` -- [1, 2] indices for 2 independent series
10. `findpoly_maple_style` -- no crash with q parameter
11. `findcong_garvan_auto_scan` -- finds [4, 5, 5] Ramanujan congruence
12. `findcong_with_lm` -- LM=5 limits scan, no mod-7 results
13. `findcong_old_signature_error` -- "must be integer" for old [moduli] arg

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Script statement termination**
- **Found during:** Task 2 (first test run)
- **Issue:** Multi-line temp scripts used bare newlines between statements, but the parser requires `:` or `;` terminators
- **Fix:** Added `:` terminators after each assignment line in all temp scripts
- **Files modified:** crates/qsym-cli/tests/cli_integration.rs
- **Commit:** 64cb4df

**2. [Rule 1 - Bug] findcong boundary access error**
- **Found during:** Task 2 (manual testing)
- **Issue:** `findcong(partition_gf(200), 200)` fails because it tries to access coefficient at q^200 but series is only known to O(q^200)
- **Fix:** Integration tests use `partition_gf(201)` with `findcong(p, 200)` to provide enough coefficients
- **Files modified:** crates/qsym-cli/tests/cli_integration.rs
- **Commit:** 64cb4df

## Commits

| # | Hash | Message |
|---|------|---------|
| 1 | c9d1af5 | feat(36-03): update help text for 11 relation discovery functions |
| 2 | 64cb4df | test(36-03): add 13 integration tests for relation discovery dispatch |

## Verification

- `cargo test -p qsym-cli --lib` -- 385 passed, 0 failed
- `cargo test -p qsym-cli --test cli_integration` -- 115 passed, 0 failed (102 existing + 13 new)
- Total CLI tests: 500 (385 unit + 115 integration)
- help(findlincombo) shows "(f, L, SL, q, topshift)" signature
- help(findcong) shows "(QS, T) or (QS, T, LM) or (QS, T, LM, XSET)" signature
- Integration tests verify actual binary behavior end-to-end
- findcong integration test finds Ramanujan's p(5n+4) = 0 mod 5 as [4, 5, 5]
- Old signatures produce clean error messages (tested for findlincombo and findcong)
- Zero regressions in existing test suite

## Self-Check: PASSED

- [x] crates/qsym-cli/src/help.rs exists and updated
- [x] crates/qsym-cli/tests/cli_integration.rs exists and updated
- [x] Commit c9d1af5 found
- [x] Commit 64cb4df found
- [x] 385 unit tests pass
- [x] 115 integration tests pass
