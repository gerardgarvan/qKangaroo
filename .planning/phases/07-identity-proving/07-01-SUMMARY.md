---
phase: 07-identity-proving
plan: "01"
subsystem: qseries
tags: [jacobi-product, eta-quotient, modularity, newman-conditions, identity-proving]

# Dependency graph
requires:
  - phase: 04-series-analysis
    provides: "prodmake, etamake, jacprodmake -- series-to-product conversion and post-processing"
  - phase: 03-core-qseries-partitions
    provides: "etaq, jacprod, FormalPowerSeries, arithmetic -- FPS expansion of products"
provides:
  - "JacFactor and JacExpression structs for symbolic Jacobi product representation"
  - "EtaExpression struct for symbolic eta quotient representation with level, weight, q-shift"
  - "ModularityResult enum and check_modularity() implementing Newman's four conditions"
  - "to_series() conversion from symbolic models to FormalPowerSeries"
  - "from_etaquotient() conversion from prodmake EtaQuotient to identity EtaExpression"
  - "Shared fps_pow helper in identity/mod.rs for repeated-squaring FPS exponentiation"
  - "pub(crate) mobius and divisors in prodmake.rs for cross-module reuse"
affects: [07-02-cusps, 07-03-proving-engine, 07-04-identity-database]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Symbolic model + to_series() expansion pattern for algebraic expressions"]

key-files:
  created:
    - "crates/qsym-core/src/qseries/identity/mod.rs"
    - "crates/qsym-core/src/qseries/identity/jac.rs"
    - "crates/qsym-core/src/qseries/identity/eta.rs"
    - "crates/qsym-core/tests/qseries_identity_jac_eta_tests.rs"
  modified:
    - "crates/qsym-core/src/qseries/mod.rs"
    - "crates/qsym-core/src/qseries/prodmake.rs"

key-decisions:
  - "fps_pow shared in identity/mod.rs as pub(crate) rather than duplicated in jac.rs and eta.rs"
  - "EtaExpression.from_etaquotient computes level as LCM of all deltas (EtaQuotient lacks level field)"
  - "Newman condition 3 uses rug::Integer for perfect square check on prod(delta^|r_delta|)"
  - "to_series panics on fractional q-shift (FPS only supports integer exponents)"

patterns-established:
  - "Symbolic model pattern: struct captures algebraic form as data, to_series() expands to FPS"
  - "Newman modularity check: four conditions checked independently with descriptive error strings"

# Metrics
duration: 4min
completed: 2026-02-14
---

# Phase 7 Plan 1: JAC and ETA Symbolic Models Summary

**JacExpression and EtaExpression symbolic models with Newman modularity checks and FPS conversion via identity/ submodule**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-14T17:09:40Z
- **Completed:** 2026-02-14T17:13:49Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- JacFactor/JacExpression capture Jacobi product structure as data with to_series() expanding to FPS via jacprod
- EtaExpression captures eta quotients as BTreeMap<delta, r_delta> with level, weight(), q_shift(), and check_modularity()
- Newman's four modularity conditions correctly implemented: divisibility, two 24-divisibility sums, perfect square check, weight zero
- 17 integration tests covering all models, conversions, modularity pass/fail cases, and edge cases

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire identity submodule, make mobius/divisors pub(crate), create JacExpression** - `4e55400` (feat)
2. **Task 2: Create EtaExpression with Newman modularity checks** - included in `4e55400` (eta.rs created with Task 1 for module compilation)
3. **Task 3: Integration tests for JAC and ETA symbolic models** - `4d72efb` (test)

## Files Created/Modified
- `crates/qsym-core/src/qseries/identity/mod.rs` - Module re-exports and shared fps_pow helper
- `crates/qsym-core/src/qseries/identity/jac.rs` - JacFactor, JacExpression with to_series()
- `crates/qsym-core/src/qseries/identity/eta.rs` - EtaExpression, ModularityResult, Newman checks, to_series(), from_etaquotient()
- `crates/qsym-core/tests/qseries_identity_jac_eta_tests.rs` - 17 integration tests
- `crates/qsym-core/src/qseries/mod.rs` - Added pub mod identity and re-exports
- `crates/qsym-core/src/qseries/prodmake.rs` - Made mobius and divisors pub(crate)

## Decisions Made
- fps_pow shared in identity/mod.rs as pub(crate) rather than duplicated -- eliminates code duplication across jac.rs and eta.rs
- EtaExpression.from_etaquotient computes level as LCM of all deltas since prodmake EtaQuotient does not store level
- Newman condition 3 (perfect square check) uses rug::Integer::pow and sqrt for exact arithmetic on arbitrarily large products
- to_series() on both types panics if q-shift is fractional, since FPS only supports integer exponents

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing rug::ops::Pow import in eta.rs**
- **Found during:** Task 1 (compilation verification)
- **Issue:** rug::Integer::pow requires Pow trait in scope; compilation failed without it
- **Fix:** Added `use rug::ops::Pow;` and `use crate::series::arithmetic;` imports to eta.rs
- **Files modified:** crates/qsym-core/src/qseries/identity/eta.rs
- **Verification:** cargo build -p qsym-core succeeds
- **Committed in:** 4e55400 (part of Task 1 commit)

**2. [Rule 3 - Blocking] Task 2 merged into Task 1**
- **Found during:** Task 1 (module wiring)
- **Issue:** eta.rs had to exist for identity/mod.rs to compile (it declares `pub mod eta;`). Creating a stub would waste effort since the full implementation was straightforward.
- **Fix:** Created complete eta.rs during Task 1 instead of in a separate Task 2
- **Files modified:** crates/qsym-core/src/qseries/identity/eta.rs
- **Verification:** All 17 tests pass
- **Committed in:** 4e55400

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both auto-fixes necessary for compilation. No scope creep. Task 2 was logically merged into Task 1.

## Issues Encountered
None beyond the import fixes noted above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- identity/ submodule is wired and functional, ready for cusps.rs (Plan 07-02)
- mobius and divisors are pub(crate), available for cusp computation in Plan 07-02
- JacExpression and EtaExpression provide the symbolic inputs for the proving engine (Plan 07-03)

## Self-Check: PASSED

- All 4 created files verified on disk
- Commits 4e55400 and 4d72efb verified in git log
- 17/17 tests pass
- cargo build -p qsym-core succeeds
- No regressions in full test suite

---
*Phase: 07-identity-proving*
*Completed: 2026-02-14*
