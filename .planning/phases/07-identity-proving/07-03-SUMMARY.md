---
phase: 07-identity-proving
plan: "03"
subsystem: qseries
tags: [eta-quotient, valence-formula, modular-function, identity-proving, newman-conditions, sturm-bound]

# Dependency graph
requires:
  - phase: 07-01
    provides: "EtaExpression with check_modularity, to_series, and fps_pow"
  - phase: 07-02
    provides: "cuspmake, eta_order_at_cusp, cusp_width, total_order, Cusp struct"
provides:
  - "ProofResult enum (Proved, NotModular, NegativeOrder, CounterExample)"
  - "EtaIdentity struct for two-sided and multi-term identities"
  - "prove_eta_identity: full valence formula proving pipeline"
  - "sturm_bound computation for Gamma_0(N)"
  - "prove_by_expansion fallback for general identities"
affects: [07-04, phase-8]

# Tech tracking
tech-stack:
  added: []
  patterns: [valence-formula-pipeline, structural-vs-expansion-proof]

key-files:
  created:
    - crates/qsym-core/src/qseries/identity/prove.rs
    - crates/qsym-core/tests/qseries_identity_prove_tests.rs
  modified:
    - crates/qsym-core/src/qseries/identity/mod.rs
    - crates/qsym-core/src/qseries/mod.rs

key-decisions:
  - "Empty combined factors (LHS=RHS) short-circuit to Proved without q-expansion to avoid non-integer q-shift panics"
  - "Two-tier proving: structural valence formula for 2-term unit-coefficient identities, q-expansion fallback for all others"
  - "Sturm bound used for weight > 0; weight-0 with non-negative cusp orders requires only constant term check"

patterns-established:
  - "Valence formula pipeline: Newman check -> cusp enumeration -> order computation -> q-expansion verification"
  - "ProofResult as exhaustive enum capturing all failure modes for downstream reporting"

# Metrics
duration: 5min
completed: 2026-02-14
---

# Phase 7 Plan 3: Proving Engine via Valence Formula Summary

**prove_eta_identity implements Garvan's provemodfuncGAMMA0id: Newman check, cusp orders, valence formula, and q-expansion verification with ProofResult capturing all four proof outcomes**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-14T17:30:51Z
- **Completed:** 2026-02-14T17:35:53Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- ProofResult enum with Proved/NotModular/NegativeOrder/CounterExample variants for exhaustive proof outcome reporting
- EtaIdentity struct supporting both two-sided (LHS=RHS) and multi-term identities
- Full valence formula pipeline: Newman modularity check -> cusp enumeration -> order-at-cusp computation -> Sturm bound -> q-expansion verification
- 11 integration tests covering all proof outcomes, trivial/non-trivial cases, and expansion fallback paths

## Task Commits

Each task was committed atomically:

1. **Task 1: ProofResult, EtaIdentity, and prove_eta_identity implementation** - `b722fba` (feat)
2. **Task 2: Integration tests proving known identities and catching false ones** - `7d9cf28` (test)

## Files Created/Modified
- `crates/qsym-core/src/qseries/identity/prove.rs` - Proving engine: ProofResult, EtaIdentity, prove_eta_identity, sturm_bound, prove_by_expansion
- `crates/qsym-core/tests/qseries_identity_prove_tests.rs` - 11 integration tests for all proof outcomes and edge cases
- `crates/qsym-core/src/qseries/identity/mod.rs` - Added prove module declaration and re-exports
- `crates/qsym-core/src/qseries/mod.rs` - Extended identity re-exports with ProofResult, EtaIdentity, prove_eta_identity

## Decisions Made
- Empty combined factors (LHS=RHS with identical eta quotients) return Proved immediately without q-expansion, avoiding panics from non-integer q-shifts in individual terms
- Two-tier proving strategy: structural valence formula for standard 2-term identities with +1/-1 coefficients; q-expansion fallback for multi-term or general-coefficient identities
- Sturm bound computed for general weight but weight-0 with non-negative cusp orders only checks constant term (with 5-term safety margin)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added early return for empty combined factors**
- **Found during:** Task 2 (prove_trivial_identity test)
- **Issue:** When LHS = RHS, combined factors are empty; prove_single_eta_quotient then tried q-expansion of original terms which had non-integer q-shifts (e.g., eta(tau)^2 * eta(5tau)^2 has q-shift 1/2), causing a panic in to_series
- **Fix:** Added early return at top of prove_single_eta_quotient when combined.factors.is_empty(), returning Proved with zero cusp orders at all cusps
- **Files modified:** crates/qsym-core/src/qseries/identity/prove.rs
- **Verification:** prove_trivial_identity test passes; all 11 tests pass
- **Committed in:** 7d9cf28 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Essential fix for correctness -- trivial identities would panic without it. No scope creep.

## Issues Encountered
None beyond the auto-fixed bug above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Proving engine complete: prove_eta_identity can verify eta-quotient identities end-to-end
- Ready for Plan 07-04 (Python API integration for identity proving)
- All 452 tests pass across the full qsym-core crate with zero regressions

## Self-Check: PASSED

- All 5 key files exist on disk
- Commit b722fba (Task 1) found in git log
- Commit 7d9cf28 (Task 2) found in git log
- All 452 tests pass (0 failures, 0 regressions)

---
*Phase: 07-identity-proving*
*Completed: 2026-02-14*
