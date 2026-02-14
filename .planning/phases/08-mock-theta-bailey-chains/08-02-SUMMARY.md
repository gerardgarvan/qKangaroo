---
phase: 08-mock-theta-bailey-chains
plan: "02"
subsystem: qseries
tags: [appell-lerch, mock-theta, zwegers, bilateral-sum, g2, g3]

# Dependency graph
requires:
  - phase: 03-core-qseries-partitions
    provides: "QMonomial, PochhammerOrder, aqprod, FormalPowerSeries arithmetic"
provides:
  - "appell_lerch_m: Appell-Lerch bilateral sum m(q^a, q, q^b) as FPS"
  - "appell_lerch_bilateral: raw bilateral sum without j(z;q) normalization"
  - "universal_mock_theta_g3: universal mock theta function g3(q^a, q)"
  - "universal_mock_theta_g2: universal mock theta function g2(q^a, q)"
  - "ZwegersCompletion: symbolic container for mock theta completion data"
affects: [mock-theta, python-api]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Positive-exponent algebraic identity for Laurent-type Pochhammer products"
    - "Extended truncation for negative q-exponent bilateral sum terms"
    - "Symbolic container for transcendental correction (erfc)"

key-files:
  created:
    - "crates/qsym-core/src/qseries/appell_lerch.rs"
    - "crates/qsym-core/tests/qseries_appell_lerch_tests.rs"
  modified:
    - "crates/qsym-core/src/qseries/mod.rs"

key-decisions:
  - "Return raw bilateral sum (not normalized by j(z;q)) since j(q^b;q)=0 for all integer b"
  - "Algebraic identity rewrites (q^{1-a};q)_{n+1} to avoid negative-exponent FPS inversion"
  - "Truncate g3/g2 summation at max_valid_n = a-2 for integer a_pow >= 2"
  - "ZwegersCompletion is symbolic only (erfc is transcendental, not exact rational)"

patterns-established:
  - "Positive-exponent identity: (q^{1-a};q)_{n+1} = (-1)^{n+1} * q^{-S} * prod(1-q^{a-1-k})"
  - "Extended truncation: when q_exp < 0, compute geometric series to trunc - q_exp then trim"

# Metrics
duration: 25min
completed: 2026-02-14
---

# Phase 8 Plan 02: Appell-Lerch Sums Summary

**Bilateral Appell-Lerch sum with positive-exponent algebraic identity for g2/g3, plus symbolic Zwegers completion**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-02-14
- **Completed:** 2026-02-14
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Appell-Lerch bilateral sum correctly evaluates for integer q-power specializations with pole-skipping
- Universal mock theta functions g3 and g2 use algebraic identity to avoid negative-exponent FPS
- ZwegersCompletion provides symbolic container with linear relation verification
- 25 tests covering bilateral sums, g3/g2, Zwegers, truncation consistency, and edge cases
- Full regression suite passes (483 total tests)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create appell_lerch.rs with m(x,q,z), g2, g3, and Zwegers completion** - `078acff` (feat)
2. **Task 2: Tests for Appell-Lerch sums, universal mock theta, and functional equations** - `c2229f8` (test)

**Plan metadata:** (pending final commit)

## Files Created/Modified
- `crates/qsym-core/src/qseries/appell_lerch.rs` - Appell-Lerch bilateral sum, geometric series, g3/g2 with positive-exponent identity, ZwegersCompletion struct (576 lines)
- `crates/qsym-core/tests/qseries_appell_lerch_tests.rs` - 25 tests covering all public API (430 lines)
- `crates/qsym-core/src/qseries/mod.rs` - Module registration and re-exports for appell_lerch

## Decisions Made

1. **Raw bilateral sum instead of normalized m(x,q,z):** The Jacobi theta function j(q^b;q) = (q^b;q)_inf * (q^{1-b};q)_inf * (q;q)_inf vanishes for ALL integer b because one of the three infinite products always has a (1-q^0) = 0 factor. Therefore appell_lerch_m returns the raw bilateral sum (numerator of m), which is useful for identity verification where j(z;q) cancels from both sides.

2. **Positive-exponent algebraic identity for g3/g2:** The denominator (q^{1-a};q)_{n+1} contains factors (1-q^{negative}) which produce Laurent-type FPS that the inversion recurrence cannot handle correctly. Used the identity: (1-q^{-m}) = -(1-q^m)/q^m to algebraically rewrite each term so all denominators have positive-exponent factors only. This produces: term_n = (-1)^{n+1} * q^{(n+1)(a-1)} / [positive-exponent products].

3. **Denominator truncation at max_valid_n:** For integer a_pow >= 2, the Pochhammer product (q^{1-a};q)_{n+1} vanishes at n >= a-1 (the factor (1-q^0) = 0 appears). Summation is limited to n = 0..=a-2 via compute_max_valid_n().

4. **Extended truncation for negative q_exp:** In the bilateral sum, negative r values produce negative q_exp = r(r-1)/2 + z_pow*r. The geometric series 1/(1-q^k) needs to extend to truncation_order - q_exp to ensure all contributions to coefficients [0, trunc) are captured after multiplication.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] j(q^b;q) = 0 for all integer b -- redesigned appell_lerch_m**
- **Found during:** Task 2 (testing)
- **Issue:** Plan called for computing m(x,q,z) = bilateral_sum / j(z;q), but j(q^b;q) = 0 for every integer b, making division undefined
- **Fix:** Changed appell_lerch_m to return the raw bilateral sum without j(z;q) normalization
- **Files modified:** appell_lerch.rs
- **Verification:** All bilateral sum tests pass
- **Committed in:** c2229f8

**2. [Rule 1 - Bug] FPS inversion fails for Laurent-type Pochhammer products**
- **Found during:** Task 2 (testing g3 truncation consistency)
- **Issue:** (q^{1-a};q)_{n+1} has negative-exponent factors (1-q^{negative}), and the FPS inversion recurrence c[n] = (-1/a0) * sum a[k]*c[n-k] only accesses positive indices, ignoring negative-exponent terms entirely
- **Fix:** Algebraically rewrote g3/g2 using the identity (1-q^{-m}) = -(1-q^m)/q^m to produce all-positive-exponent denominators
- **Files modified:** appell_lerch.rs
- **Verification:** g3 truncation consistency test passes (coefficients match between trunc=10 and trunc=20)
- **Committed in:** c2229f8

**3. [Rule 1 - Bug] Bilateral sum truncation consistency for negative q_exp**
- **Found during:** Task 2 (testing truncation consistency)
- **Issue:** At lower truncation, geometric series had fewer terms, so when shifted by negative q_exp, contributions to low-order coefficients were missed
- **Fix:** Extended the effective truncation order by -q_exp when q_exp < 0, then truncated the product back to target
- **Files modified:** appell_lerch.rs
- **Verification:** appell_lerch_truncation_consistency test passes
- **Committed in:** c2229f8

---

**Total deviations:** 3 auto-fixed (3 bugs via Rule 1)
**Impact on plan:** All fixes necessary for mathematical correctness. The plan's approach of normalizing by j(z;q) was fundamentally impossible for integer parameters -- a mathematical constraint, not an implementation error. The positive-exponent identity and extended truncation are essential for correct FPS arithmetic.

## Issues Encountered
- Heavy cancellation in bilateral sums: the alternating sign (-1)^r causes many coefficient cancellations, resulting in sparse series for most parameter choices. This is genuine mathematical behavior, not a bug.
- The g3/g2 functions produce terms starting at q^{(n+1)*(a-1)}, which grows with a_pow. For large a_pow with small truncation, few terms fit in the truncation window. Tests were adjusted accordingly.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Appell-Lerch sums and universal mock theta functions available for future cross-validation with classical mock theta functions (Plan 01)
- ZwegersCompletion provides foundation for any future modular form completion work
- Plan 04 (verification plan) can use these functions for integration testing

## Self-Check: PASSED

- FOUND: crates/qsym-core/src/qseries/appell_lerch.rs (575 lines, min 250)
- FOUND: crates/qsym-core/tests/qseries_appell_lerch_tests.rs (446 lines, min 150)
- FOUND: .planning/phases/08-mock-theta-bailey-chains/08-02-SUMMARY.md
- FOUND: commit 078acff (Task 1 - feat)
- FOUND: commit c2229f8 (Task 2 - test)
- All 483 tests pass (25 new Appell-Lerch + 458 existing)

---
*Phase: 08-mock-theta-bailey-chains*
*Completed: 2026-02-14*
