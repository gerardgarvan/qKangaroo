---
phase: 38-new-functions-analysis-discovery
verified: 2026-02-20T01:10:49Z
status: gaps_found
score: 11/12 must-haves verified
gaps:
  - truth: "REQUIREMENTS.md tracking entries reflect completed status for NEW-05, NEW-06, NEW-07, NEW-09"
    status: failed
    reason: "REQUIREMENTS.md still shows unchecked [ ] and Pending for all four requirement IDs. The code implements them correctly but the tracking document was not updated."
    artifacts:
      - path: ".planning/REQUIREMENTS.md"
        issue: "Lines 56-60: NEW-05/06/07/09 all have unchecked [ ] prefix. Lines 120-124: tracking table shows Pending for all four."
    missing:
      - "Change [ ] to [x] for NEW-05 in .planning/REQUIREMENTS.md"
      - "Change [ ] to [x] for NEW-06 in .planning/REQUIREMENTS.md"
      - "Change [ ] to [x] for NEW-07 in .planning/REQUIREMENTS.md"
      - "Change [ ] to [x] for NEW-09 in .planning/REQUIREMENTS.md"
      - "Change Pending to Complete for NEW-05, NEW-06, NEW-07, NEW-09 in the tracking table"
---

# Phase 38: New Functions - Analysis & Discovery - Verification Report

**Phase Goal:** Four analysis/discovery functions are available (checkmult, checkprod, lqdegree0, findprod), completing the univariate Garvan function inventory
**Verified:** 2026-02-20T01:10:49Z
**Status:** gaps_found
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | lqdegree0(f) returns the minimum key in an FPS BTreeMap (lowest q-degree) | VERIFIED | eval.rs lines 2054-2062: dispatch arm calls fps.min_order(), returns Value::Integer or Value::None. Unit test dispatch_lqdegree0_returns_min_order passes. |
| 2 | checkmult(f, T) prints MULTIPLICATIVE or NOT MULTIPLICATIVE with first failing (m,n) and returns 1 or 0 | VERIFIED | eval.rs lines 2158-2190: 2-or-3 arg dispatch, inner loop uses gcd_i64 coprimality check, prints result, returns QInt 0 or 1. Integration test cli_checkmult_not_multiplicative passes. |
| 3 | checkmult(f, T, yes) prints ALL failing (m,n) pairs and returns 1 or 0 | VERIFIED | eval.rs line 2163: print_all flag set when args[2] is String(yes). Integration test cli_checkmult_with_yes asserts multiple NOT MULTIPLICATIVE occurrences. |
| 4 | checkprod(f, M, Q) silently returns [a, 1] for nice products, [a, max_exp] otherwise, or [[a, c0], -1] for non-integer leading coeff | VERIFIED | eval.rs lines 2192-2199: dispatches to checkprod_impl. checkprod_impl (lines 1563-1612) implements all three return paths. Integration test cli_checkprod_eta_nice passes. |
| 5 | findprod(FL, T, M, Q) silently returns list of [valuation, coeff_vector] pairs for primitive coefficient vectors yielding nice products | VERIFIED | eval.rs lines 2445-2492: 4-arg dispatch using odometer iteration, gcd_i64 primitive filter, checkprod_impl inner call. Integration test cli_findprod_garvan_4arg passes. |
| 6 | Old 3-arg findprod dispatch is replaced by new 4-arg Garvan version | VERIFIED | eval.rs line 2447: expect_args(name, args, 4). Unit test dispatch_findprod_old_3arg_errors and integration test cli_findprod_old_3arg_errors both confirm 3-arg call returns an error. |
| 7 | help(checkmult) shows signature, description, and example with 2-arg and 3-arg forms | VERIFIED | help.rs lines 344-349: FuncHelp entry with name=checkmult, signature includes optional yes arg, description and example present. every_canonical_function_has_help_entry passes. |
| 8 | help(checkprod) shows signature with (f, M, Q) and describes silent [a, code] return | VERIFIED | help.rs lines 350-356: FuncHelp entry with signature checkprod(f, M, Q), description documents all three return formats. |
| 9 | help(lqdegree0) shows signature with (f) and describes FPS-only behavior | VERIFIED | help.rs lines 294-300: FuncHelp entry with signature lqdegree0(f), description mentions FPS and Garvan compatibility. |
| 10 | help(findprod) shows updated 4-arg (FL, T, M, Q) signature, not old 3-arg | VERIFIED | help.rs lines 424-430: FuncHelp entry with signature findprod(FL, T, M, Q). Canonical count tests at 89 pass. |
| 11 | General help listing includes checkmult, checkprod, lqdegree0 in appropriate categories | VERIFIED | help.rs lines 53-61: Series Analysis section includes lqdegree0, checkmult, checkprod with descriptions. general_help tests pass. |
| 12 | REQUIREMENTS.md tracking entries reflect completed status for NEW-05, NEW-06, NEW-07, NEW-09 | FAILED | .planning/REQUIREMENTS.md lines 56-60 still show unchecked [ ]. Tracking table lines 120-124 still show Pending for all four IDs. NEW-08 correctly remains deferred. |

**Score:** 11/12 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|--------|
| crates/qsym-cli/src/eval.rs | All 4 dispatch arms + gcd_i64 + checkprod_impl + is_nice_checkprod_result + increment_coeffs + get_signature + ALL_FUNCTION_NAMES entries | VERIFIED | Dispatch arms at lines 2054, 2158, 2192, 2445. Helpers at lines 1553, 1563, 1614, 1629. get_signature entries at 3639-3641, 3658. ALL_FUNCTION_NAMES at 3751-3752. 6 unit tests in cfg(test). |
| crates/qsym-cli/src/help.rs | FuncHelp entries for checkmult, checkprod, lqdegree0; updated findprod entry; general_help listing | VERIFIED | Entries at lines 294-300, 344-349, 350-356, 424-430. General help at lines 53-61, 73. FUNC_HELP.len()==89, canonical count==89, both tests pass. |
| crates/qsym-cli/tests/cli_integration.rs | Integration tests for all 4 Phase 38 functions | VERIFIED | 6 tests at lines 1580-1653: lqdegree0, checkmult x2, checkprod, findprod 4-arg, findprod 3-arg rejection. All 6 pass. |
| .planning/REQUIREMENTS.md | NEW-05/06/07/09 marked [x] and Complete | STUB | Checkbox remains unchecked [ ]. Tracking table shows Pending. No functional impact on goal. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|--------|
| eval.rs checkprod dispatch (line 2192) | checkprod_impl helper (line 1563) | direct function call | WIRED | checkprod_impl(&fps, m_threshold, q_order) at line 2198 confirmed in source |
| eval.rs findprod dispatch (line 2445) | checkprod_impl helper (line 1563) | called in inner loop | WIRED | checkprod_impl(&combo, m_threshold, q_order) at line 2478 confirmed in source |
| eval.rs findprod dispatch (line 2445) | arithmetic::scalar_mul + arithmetic::add | forms linear combination | WIRED | Lines 2473-2474: arithmetic::scalar_mul and arithmetic::add calls confirmed in source |
| help.rs FUNC_HELP array | help.rs canonical test list | every_canonical_function_has_help_entry test | WIRED | Test at line 935 includes lqdegree0, checkmult, checkprod in canonical vec; asserts FUNC_HELP.len()==89; test passes |
| cli_integration.rs | qsym-cli binary | run() helper with -c flag | WIRED | All 6 integration tests use run(&["-c", ...]) pattern; all 6 pass with exit code 0 |

### Requirements Coverage

| Requirement | Status | Blocking Issue |
|-------------|--------|----------------|
| NEW-05: checkmult(QS, T) and checkmult(QS, T, yes) | SATISFIED in code; NOT UPDATED in tracking | REQUIREMENTS.md not updated |
| NEW-06: checkprod(f, M, Q) | SATISFIED in code; NOT UPDATED in tracking | REQUIREMENTS.md not updated |
| NEW-07: lqdegree0(qexp) | SATISFIED in code; NOT UPDATED in tracking | REQUIREMENTS.md not updated. Fractional exponent scope deferred per RESEARCH.md. |
| NEW-08: zqfactor | CORRECTLY DEFERRED | Marked Deferred in REQUIREMENTS.md - correct |
| NEW-09: findprod(FL, T, M, Q) | SATISFIED in code; NOT UPDATED in tracking | REQUIREMENTS.md not updated |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| crates/qsym-cli/src/eval.rs | 3254 | fn polynomial_relation_to_value never used (pre-existing compiler warning) | Info | Pre-existing issue unrelated to Phase 38; no functional impact |

No stub returns, empty implementations, TODO/FIXME comments, or placeholder patterns found in Phase 38 code artifacts.

### Human Verification Required

None. All Phase 38 behaviors are programmatically verifiable via dispatch return values and stdout capture in integration tests.

### Gaps Summary

One gap: REQUIREMENTS.md was not updated after implementation. This is a documentation tracking gap only. All four functions (checkmult, checkprod, lqdegree0, findprod) are fully implemented, wired, tested, and accessible via CLI.

The fix is a 5-line change: flip four unchecked boxes to checked in the requirements list, and change four Pending entries to Complete in the tracking table.

Root cause: Both SUMMARY files document requirements-completed: [NEW-05, NEW-06, NEW-07, NEW-09] but neither updated the source REQUIREMENTS.md document.

---

## Test Results Summary

- Unit tests (qsym-cli lib): **418 passed, 0 failed**
- Integration tests (cli_integration): **131 passed, 0 failed**
- Phase 38 unit tests (6): dispatch_lqdegree0_returns_min_order, dispatch_checkmult_partition_not_multiplicative, dispatch_checkmult_with_yes_prints_all, dispatch_checkprod_eta_nice_product, dispatch_findprod_garvan_4arg, dispatch_findprod_old_3arg_errors -- **all pass**
- Phase 38 integration tests (6): cli_lqdegree0_partition_gf, cli_checkmult_not_multiplicative, cli_checkmult_with_yes, cli_checkprod_eta_nice, cli_findprod_garvan_4arg, cli_findprod_old_3arg_errors -- **all pass**
- Help tests: every_canonical_function_has_help_entry (89 entries), func_help_count_matches_canonical (89) -- **both pass**
- function_count_verification: count >= 78 -- **passes**

---

_Verified: 2026-02-20T01:10:49Z_
_Verifier: Claude (gsd-verifier)_
