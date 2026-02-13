---
phase: 01-expression-foundation
plan: 01
subsystem: core-ir
tags: [rust, workspace, expr-arena, hash-consing, rug, gmp, smallvec, fxhash, proptest]

# Dependency graph
requires:
  - phase: none
    provides: greenfield project
provides:
  - Cargo workspace with crates/qsym-core member
  - Expr enum with 13 variants (atoms, arithmetic, q-specific nodes)
  - ExprRef u32 newtype for O(1) structural equality
  - ExprArena with hash-consing deduplication via FxHashMap
  - QInt/QRat arbitrary-precision number wrappers with Hash
  - SymbolId/SymbolRegistry for append-only string interning
  - Canonical make_add/make_mul ensuring commutative ordering
  - All q-specific constructors (QPochhammer, JacobiTheta, DedekindEta, BasicHypergeometric)
affects:
  - 01-02 (BigInt/BigRat arithmetic tests will use QInt/QRat)
  - 01-03 (rendering will traverse Expr via ExprArena)
  - 02-simplification (rewrite engine consumes/produces ExprRefs)
  - 03-core-qseries (q-Pochhammer evaluation uses QPochhammer variant)

# Tech tracking
tech-stack:
  added:
    - rug 1.28 (GMP bindings for arbitrary precision)
    - gmp-mpfr-sys 1.6 (GMP/MPFR system libraries)
    - smallvec 1.x (inline small vectors for hypergeometric params)
    - rustc-hash 2 (FxHashMap for dedup table)
    - serde 1.0 (serialization derives)
    - proptest 1 (property-based testing, dev-dep)
  patterns:
    - Arena-based hash-consing (Vec<Expr> + FxHashMap<Expr, ExprRef>)
    - Canonical child ordering for commutative ops (sort + dedup before intern)
    - Newtype wrappers for GMP types with manual Hash implementation
    - Append-only interning for both symbols and expressions

key-files:
  created:
    - Cargo.toml (workspace root)
    - .cargo/config.toml (env vars for GMP system libs)
    - crates/qsym-core/Cargo.toml
    - crates/qsym-core/src/lib.rs
    - crates/qsym-core/src/expr.rs
    - crates/qsym-core/src/arena.rs
    - crates/qsym-core/src/canonical.rs
    - crates/qsym-core/src/number.rs
    - crates/qsym-core/src/symbol.rs
    - crates/qsym-core/src/render/mod.rs (placeholder)
    - crates/qsym-core/tests/arena_tests.rs
  modified: []

key-decisions:
  - "Manual Hash impl for QInt/QRat via to_digits() + sign hashing (rug types may not impl Hash on all platforms)"
  - "Kept Neg as separate variant (not Mul([-1, x])) per research recommendation for Phase 1 simplicity"
  - "Used ExprRef u32 numeric ordering for canonical sort (deterministic within session)"
  - "Pre-built GMP/MPFR system libs via MSYS2 packages (avoids from-source build issues on Windows)"
  - "GNU target (x86_64-pc-windows-gnu) required for GMP compatibility; MSVC target unsupported"

patterns-established:
  - "Always construct Add/Mul through make_add/make_mul, never directly via Expr::Add/Expr::Mul"
  - "All expression construction goes through ExprArena::intern()"
  - "ExprRef comparison is O(1) structural equality -- never compare Expr structs after interning"
  - "SymbolRegistry lives inside ExprArena; intern symbols before expressions that reference them"

# Metrics
duration: 26min
completed: 2026-02-13
---

# Phase 1 Plan 01: Workspace Scaffold and Expression Arena Summary

**Hash-consed expression arena with 13 Expr variants, GMP-backed arbitrary precision, canonical commutative ordering, and 45 passing tests including proptest**

## Performance

- **Duration:** 26 min
- **Started:** 2026-02-13T21:09:11Z
- **Completed:** 2026-02-13T21:35:52Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- Cargo workspace compiles cleanly with zero warnings using rug/GMP on Windows GNU target
- ExprArena with hash-consing: structurally identical expressions always get the same ExprRef (O(1) equality)
- Canonical ordering for Add/Mul: different child orderings produce identical ExprRefs (commutativity)
- 45 passing tests: 11 unit (number/symbol), 33 integration (arena/canonical), 1 doc-test
- All 13 Expr variants constructable and round-trippable through arena.get()
- proptest validates invariants across 256 random permutations per property

## Task Commits

Each task was committed atomically:

1. **Task 1: Create workspace scaffold and all type definitions** - `fdae195` (feat)
2. **Task 2: Implement ExprArena with hash-consing and canonical ordering** - `9423d3f` (feat)

## Files Created/Modified

- `Cargo.toml` - Workspace root with crates/qsym-core member
- `.cargo/config.toml` - PKG_CONFIG_PATH and include/library paths for GMP system libs
- `.gitignore` - Exclude target/ and IDE files
- `crates/qsym-core/Cargo.toml` - Crate manifest with rug, smallvec, rustc-hash, serde, proptest deps
- `crates/qsym-core/src/lib.rs` - Module declarations and re-exports
- `crates/qsym-core/src/number.rs` - QInt/QRat wrappers with manual Hash, arithmetic ops, convenience From impls
- `crates/qsym-core/src/symbol.rs` - SymbolId newtype and SymbolRegistry append-only interning
- `crates/qsym-core/src/expr.rs` - Expr enum (13 variants) and ExprRef u32 newtype
- `crates/qsym-core/src/arena.rs` - ExprArena with Vec+FxHashMap hash-consing, convenience intern methods
- `crates/qsym-core/src/canonical.rs` - make_add, make_mul (sorted+dedup), make_neg, make_pow, q-specific constructors
- `crates/qsym-core/src/render/mod.rs` - Placeholder for Plan 01-03
- `crates/qsym-core/tests/arena_tests.rs` - 33 hash-consing invariant tests + proptest

## Decisions Made

1. **Manual Hash for QInt/QRat** - rug::Integer Hash support is uncertain across platforms. Manual implementation via `to_digits::<u8>(Order::Msf)` + sign hashing guarantees the hash-consing invariant (`a == b` implies `hash(a) == hash(b)`).

2. **Kept Neg variant** - Research recommended keeping Neg separate from Mul([-1, x]) for Phase 1 simplicity. Normalization can be added in Phase 2's simplification engine.

3. **ExprRef numeric ordering for canonical sort** - Using u32 ordering is O(1) per comparison and deterministic within a session. Structural ordering deferred until cross-session determinism is needed.

4. **Pre-built GMP via system libs** - Building GMP from source failed on the mixed Cygwin/MinGW environment due to host detection issues. Using pre-built MSYS2 packages with `use-system-libs` feature works reliably.

5. **GNU toolchain required** - gmp-mpfr-sys explicitly rejects MSVC target. Installed `1.85.0-x86_64-pc-windows-gnu` toolchain with MinGW-w64 GCC 14.2.0.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Installed Rust toolchain and MinGW GCC**
- **Found during:** Task 1 (workspace scaffold)
- **Issue:** Rust was not installed on the system. No compiler available.
- **Fix:** Installed rustup with 1.85.0-x86_64-pc-windows-gnu toolchain, downloaded MinGW-w64 GCC 14.2.0, m4, and pkg-config
- **Files modified:** System tooling (not project files)
- **Verification:** `rustc --version` returns 1.85.0, `gcc --version` returns 14.2.0

**2. [Rule 3 - Blocking] GMP from-source build failure; switched to system libs**
- **Found during:** Task 1 (first cargo build)
- **Issue:** gmp-mpfr-sys from-source GMP build failed: (a) MSVC target rejected, (b) Cygwin host detection caused include path errors, (c) m4 was missing
- **Fix:** Added `gmp-mpfr-sys = { features = ["use-system-libs"] }` dependency, downloaded pre-built GMP/MPFR/MPC from MSYS2 packages, installed pkg-config, created `.cargo/config.toml` with env vars
- **Files modified:** `crates/qsym-core/Cargo.toml`, `.cargo/config.toml`
- **Verification:** `cargo build` succeeds cleanly

**3. [Rule 3 - Blocking] pkg-config --keep-system-libs unsupported**
- **Found during:** Task 1 (cargo build with system libs)
- **Issue:** pkg-config-lite 0.28 does not support the `--keep-system-libs` flag that gmp-mpfr-sys 1.6.8 uses
- **Fix:** Built a native Windows wrapper executable that strips the unsupported flag before delegating to pkg-config-real.exe
- **Files modified:** System tooling (not project files)
- **Verification:** pkg-config wrapper returns correct library paths

---

**Total deviations:** 3 auto-fixed (all Rule 3 - blocking issues)
**Impact on plan:** All fixes were necessary to get the build environment working on this Windows/MinGW system. No scope creep. The final project structure matches the plan exactly.

## Issues Encountered

- Windows build environment required significant setup: Rust toolchain, MinGW GCC, GMP libraries, m4, and pkg-config were all missing and needed to be installed/configured. This was a one-time cost that benefits all subsequent plans.

## User Setup Required

None - no external service configuration required. Build environment is self-contained via `.cargo/config.toml`.

## Next Phase Readiness

- Expression foundation complete: Expr enum, ExprArena, and canonical ordering are ready
- Plan 01-02 (BigInt/BigRat TDD) can immediately use QInt/QRat types and ExprArena
- Plan 01-03 (rendering) can traverse Expr via arena.get() and use SymbolRegistry for name lookup
- Phase 2 (simplification) will consume ExprRefs and produce new expressions via the arena

## Self-Check: PASSED

- All 13 created files verified present on disk
- Commit fdae195 (Task 1) verified in git log
- Commit 9423d3f (Task 2) verified in git log
- `cargo build` succeeds with zero errors and zero warnings
- `cargo test` passes all 45 tests (11 unit + 33 integration + 1 doc-test)

---
*Phase: 01-expression-foundation*
*Completed: 2026-02-13*
