---
phase: 02-simplification-series-engine
plan: 02
subsystem: simplification
tags: [rewriting, term-rewriting, simplification, bottom-up-traversal, fixpoint, hash-consing]

# Dependency graph
requires:
  - phase: 01-expression-foundation
    provides: "ExprArena, Expr enum, ExprRef, canonical constructors (make_add/make_mul/make_neg/make_pow), QInt, QRat"
provides:
  - "SimplificationEngine with 4 phased rule sets and fixpoint detection"
  - "Bottom-up traversal of ExprArena DAG for all 13 Expr variants"
  - "normalize rules: flatten nested Add/Mul, combine numeric constants"
  - "cancel rules: identity elimination, zero annihilation, Pow identities"
  - "collect rules: like-term coefficient collection, power collection in Mul"
  - "simplify_arith rules: double negation, neg of constants, pow-of-pow"
  - "simplify() convenience function for default-settings simplification"
affects: [series-evaluation, q-pochhammer-algebra, partition-functions, identity-proving]

# Tech tracking
tech-stack:
  added: []
  patterns: [phased-rewriting, bottom-up-traversal, fixpoint-detection-via-hash-consing, direct-match-rules]

key-files:
  created:
    - crates/qsym-core/src/simplify/mod.rs
    - crates/qsym-core/src/simplify/rules.rs
    - crates/qsym-core/src/simplify/traverse.rs
    - crates/qsym-core/tests/simplify_tests.rs
  modified:
    - crates/qsym-core/src/lib.rs

key-decisions:
  - "Direct Rust match arms for rules instead of generic pattern matching engine (per research recommendation)"
  - "4 phases (normalize, cancel, collect, simplify_arith) with restart-from-phase-1 on any change"
  - "Max 100 iterations cap for guaranteed termination"
  - "intern_numeric helper promotes Integer when denominator is 1, keeps Rational otherwise"

patterns-established:
  - "Phased rewriting: apply rule phases in priority order, restart on change, cap iterations"
  - "Bottom-up traversal: recurse into children first, apply rule to rebuilt node"
  - "Change detection via ExprRef equality (O(1) hash-consing comparison)"
  - "Clone-before-recurse pattern to avoid borrow conflicts with arena"

# Metrics
duration: 5min
completed: 2026-02-13
---

# Phase 2 Plan 02: Simplification Engine Summary

**Phased simplification engine with 4 rule sets (normalize, cancel, collect, simplify_arith), bottom-up DAG traversal, and fixpoint termination -- 37 tests passing**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-13T22:25:30Z
- **Completed:** 2026-02-13T22:30:37Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- SimplificationEngine with phased rule application and O(1) fixpoint detection via hash-consing
- Bottom-up traversal correctly handles all 13 Expr variants including q-specific nodes
- 4 rule phases covering: flatten, combine constants, cancel identities, collect like terms, double negation, pow-of-pow
- 37 comprehensive tests including termination on adversarial inputs (50-deep Neg, 100-wide Add) and idempotency verification
- All 240 tests passing (166 existing + 33 series + 37 simplify + 3 smoke + 1 doctest)

## Task Commits

Each task was committed atomically:

1. **Task 1: SimplificationEngine with bottom-up traversal and 4 rule phases** - `c82ec51` (feat)
2. **Task 2: Comprehensive simplification tests including termination** - `b8f0737` (test)

## Files Created/Modified
- `crates/qsym-core/src/simplify/mod.rs` - SimplificationEngine with phased rule application, fixpoint loop, convenience simplify() function, 3 smoke tests
- `crates/qsym-core/src/simplify/rules.rs` - 4 rule phase functions (normalize, cancel, collect, simplify_arith) with direct Rust match arms
- `crates/qsym-core/src/simplify/traverse.rs` - bottom_up_apply() for recursive DAG traversal of all 13 Expr variants
- `crates/qsym-core/src/lib.rs` - Added `pub mod simplify;`
- `crates/qsym-core/tests/simplify_tests.rs` - 37 comprehensive tests

## Decisions Made
- Used direct Rust match arms for rules instead of generic pattern matching engine (simpler, handles n-ary operators correctly, per research recommendation)
- 4 phases instead of 6 (expand and verify deferred per plan -- expand is off by default, verify is implicit in fixpoint check)
- intern_numeric() helper auto-promotes to Integer when QRat denominator is 1, avoiding spurious Rational(n/1) in the arena
- extract_add_term only strips single-numeric-factor from Mul for like-term collection (multi-numeric-factor case handled by normalize phase first)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Simplification engine ready for use by downstream phases (series evaluation, q-Pochhammer algebra)
- All 4 rule phases operational with guaranteed termination
- Future expansion: add expand phase (distribute Mul over Add), neg bubbling from Mul, generic pattern matcher for q-specific rules

## Self-Check: PASSED

All 5 files found. Both task commits verified (c82ec51, b8f0737).

---
*Phase: 02-simplification-series-engine*
*Completed: 2026-02-13*
