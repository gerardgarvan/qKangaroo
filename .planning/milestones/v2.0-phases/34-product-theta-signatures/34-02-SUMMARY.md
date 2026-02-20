---
phase: 34-product-theta-signatures
plan: 02
subsystem: qsym-cli/eval+help+repl
tags: [maple-compat, numbpart, alias-reversal, help-text, integration-tests]
dependency_graph:
  requires: [34-01]
  provides: [numbpart-canonical, maple-help-signatures, phase-34-complete]
  affects: [eval.rs, help.rs, repl.rs, cli_integration.rs]
tech_stack:
  added: []
  patterns: [alias-reversal, help-redirect, bounded-partition-count]
key_files:
  modified:
    - crates/qsym-cli/src/eval.rs
    - crates/qsym-cli/src/help.rs
    - crates/qsym-cli/src/repl.rs
    - crates/qsym-cli/tests/cli_integration.rs
decisions:
  - "numbpart is now canonical name, partition_count resolves to it via alias"
  - "numbpart(n,m) uses bounded_parts_gf(m, sym, n+1) and extracts coefficient of q^n"
  - "help(partition_count) redirects to numbpart via lookup rewrite in function_help()"
  - "Piped help tests replaced with additional -c flag tests since help commands only work in interactive REPL"
metrics:
  duration: 6min
  completed: 2026-02-19T20:00:14Z
  tasks: 2
  files: 4
  tests_added: 24
  total_tests: 378 unit + 90 integration
---

# Phase 34 Plan 02: Numbpart Canonical + Help + Integration Tests Summary

Reversed numbpart/partition_count alias direction, added numbpart(n,m) bounded form, updated all 7 product function help entries to Maple-style signatures, and added 18 integration tests.

## What Was Done

### Task 1: Reverse numbpart alias + add numbpart(n,m) (79d5a33)

Six changes in eval.rs:

- **Dispatch match arm:** Renamed `"partition_count"` to `"numbpart"` and added 2-arg form `numbpart(n, m)` that computes partitions of n with largest part at most m using `bounded_parts_gf(m, sym, n+1)` and extracting the coefficient of q^n.
- **resolve_alias:** Reversed from `numbpart -> partition_count` to `partition_count -> numbpart`.
- **ALL_FUNCTION_NAMES:** Replaced `"partition_count"` with `"numbpart"`.
- **ALL_ALIAS_NAMES:** Replaced `"numbpart"` with `"partition_count"`.
- **get_signature:** Changed `"partition_count" => "(n)"` to `"numbpart" => "(n) or (n, m)"`.
- **Tests:** Updated resolve_alias tests, renamed dispatch tests to use numbpart, added `dispatch_numbpart_100` (190569292), `dispatch_numbpart_bounded` (5,3)=5, `dispatch_partition_count_alias` verifying alias resolution.

### Task 2: Help text, tab completion, integration tests (55000e6)

Three files updated:

- **help.rs:** Updated general_help() Partitions section to show `numbpart` instead of `partition_count`. Replaced all 7 product FUNC_HELP entries with Maple-style signatures (aqprod(a,q,n), qbin(q,m,n), etaq(q,delta,T), jacprod(a,b,q,T), tripleprod(z,q,T), quinprod(z,q,T), winquist(a,b,q,T)). Replaced `partition_count` entry with `numbpart` including 2-arg signature. Added `partition_count -> numbpart` redirect in `function_help()`. Updated 5 tests.
- **repl.rs:** Replaced `"partition_count"` with `"numbpart"` in `canonical_function_names()`. Updated `complete_no_maple_aliases` test, added `complete_numbpart_canonical` test.
- **cli_integration.rs:** Added 18 new integration tests covering: 8 Maple-style forms (jacprod 4-arg, tripleprod 3-arg, quinprod 3-arg, winquist 4-arg, qbin Garvan 3-arg, qbin 4-arg, etaq multi-delta, etaq single delta), 4 numbpart tests (primary, small, bounded, zero, bounded-zero-max), 2 alias backward-compat tests, 4 legacy form backward-compat tests.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Piped help tests cannot work in piped mode**
- **Found during:** Task 2 (integration tests)
- **Issue:** Plan specified `run_piped("help(numbpart)\n")` and `run_piped("help(partition_count)\n")` but piped stdin uses `script::execute_source` which does not process REPL commands like `help`. The `help` command only works in the interactive REPL loop. Exit code 65 (EX_DATAERR) was returned.
- **Fix:** Replaced piped help tests with `numbpart_zero` and `numbpart_bounded_zero_max` integration tests using `-c` flag. Help system is thoroughly tested by unit tests in help.rs (including the new `function_help_partition_count_redirects_to_numbpart` test).
- **Files modified:** crates/qsym-cli/tests/cli_integration.rs
- **Commit:** 55000e6

## Test Results

- 378 unit tests passing (374 existing + 4 new)
- 90 integration tests passing (72 existing + 18 new)
- Zero regressions

## Self-Check: PASSED

- [x] crates/qsym-cli/src/eval.rs exists and contains "numbpart"
- [x] crates/qsym-cli/src/help.rs exists and contains "numbpart"
- [x] crates/qsym-cli/src/repl.rs exists and contains "numbpart"
- [x] crates/qsym-cli/tests/cli_integration.rs exists and contains "numbpart"
- [x] Commit 79d5a33 exists
- [x] Commit 55000e6 exists
- [x] 34-02-SUMMARY.md exists
