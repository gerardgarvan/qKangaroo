---
phase: 29-static-linking
plan: 01
subsystem: build
tags: [gmp-mpfr-sys, static-linking, cargo, mingw, zero-dll]

# Dependency graph
requires:
  - phase: 28-dll-bundling
    provides: "Release binary with bundled DLL workflow"
provides:
  - "Static GMP/MPFR/MPC linking via gmp-mpfr-sys bundled source build"
  - "Zero non-system DLL dependencies in q-kangaroo.exe"
  - "Clean .cargo/config.toml (no system GMP paths)"
affects: [29-02-ci-static-build, future-releases]

# Tech tracking
tech-stack:
  added: []
  patterns: ["gmp-mpfr-sys default mode (no use-system-libs) for static GMP/MPFR/MPC"]

key-files:
  created: []
  modified:
    - "crates/qsym-core/Cargo.toml"
    - ".cargo/config.toml"

key-decisions:
  - "Remove use-system-libs feature entirely rather than making it conditional"
  - "Clear .cargo/config.toml completely rather than commenting out env vars"
  - "Pre-build static GMP/MPFR/MPC with --host=x86_64-w64-mingw32 and populate gmp-mpfr-sys cache for local Cygwin builds"

patterns-established:
  - "gmp-mpfr-sys cache at AppData/Local/gmp-mpfr-sys/1.6/x86_64-pc-windows-gnu/1.6.8/ for local dev builds"

requirements-completed: [BUILD-01]

# Metrics
duration: 100min
completed: 2026-02-18
---

# Phase 29 Plan 01: Static GMP/MPFR/MPC Linking Summary

**Removed use-system-libs from gmp-mpfr-sys, built static GMP 6.3.0/MPFR 4.2.2/MPC 1.3.1 from bundled C source, producing a zero-DLL q-kangaroo.exe binary (1.78MB)**

## Performance

- **Duration:** 100 min (mostly debugging Cygwin+MinGW build environment)
- **Started:** 2026-02-18T18:24:09Z
- **Completed:** 2026-02-18T20:04:37Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Removed `use-system-libs` feature from gmp-mpfr-sys in qsym-core/Cargo.toml
- Cleared `.cargo/config.toml` of all system GMP paths (PKG_CONFIG_PATH, C_INCLUDE_PATH, LIBRARY_PATH, CFLAGS, LDFLAGS)
- Built GMP 6.3.0, MPFR 4.2.2, MPC 1.3.1 from bundled C source as static `.a` archives
- Verified release binary has zero non-system DLL dependencies via `objdump -p`
- Binary size increased from ~1.4MB to 1.78MB (380KB for static GMP/MPFR/MPC code)

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove use-system-libs and clear system GMP paths** - `5a46f41` (feat)
2. **Task 2: Build locally and verify static linking** - No commit (verification-only task, no file changes)

## Files Created/Modified
- `crates/qsym-core/Cargo.toml` - Changed `gmp-mpfr-sys = { version = "1.6", features = ["use-system-libs"] }` to `gmp-mpfr-sys = "1.6"`
- `.cargo/config.toml` - Cleared entire `[env]` section (was pointing to pre-built GMP at C:/mingw64-gcc/mingw64/)

## Decisions Made
- **Remove feature entirely:** Removed `use-system-libs` unconditionally rather than keeping it as a dev convenience feature, because the static build from source is now the only supported mode
- **Clear config.toml completely:** Emptied `.cargo/config.toml` rather than commenting out the env vars, to ensure no accidental dynamic linking from stale system library paths
- **Pre-build GMP for Cygwin cache:** Manually built GMP/MPFR/MPC with `--host=x86_64-w64-mingw32` and populated the gmp-mpfr-sys cache, because the default build-from-source path doesn't work in Cygwin (see Deviations section)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed gmp-mpfr-sys build failure on Cygwin**
- **Found during:** Task 2 (build verification)
- **Issue:** gmp-mpfr-sys build.rs invokes `sh -c configure` which under Cygwin detects `zen2-pc-cygwin` as the build system. This causes two fatal problems: (1) Cygwin symlinks created by configure are unreadable by MinGW gcc (`../gmp-mparam.h: Invalid argument`), and (2) Cygwin libtool calls `lib` (MSVC archiver) instead of `ar`
- **Fix:** Manually built GMP/MPFR/MPC from bundled source with `--host=x86_64-w64-mingw32` (replacing Cygwin symlinks with copies, fixing \r in MPC config.h). Placed resulting static libraries in gmp-mpfr-sys cache at `C:\Users\Owner\AppData\Local\gmp-mpfr-sys\1.6\x86_64-pc-windows-gnu\1.6.8\`
- **Files modified:** None (cache files are outside the repo)
- **Verification:** `cargo build --release` found cached libraries and skipped source build; `objdump -p` confirmed zero non-system DLL deps
- **Note:** This issue is specific to the local Cygwin+MinGW dev environment. CI builds (MSYS2 shell) will not have this problem.

**2. [Rule 3 - Blocking] Fixed m4 DLL dependency issue**
- **Found during:** Task 2 (build verification)
- **Issue:** MinGW m4 at `C:/mingw64-gcc/mingw64/bin/m4.exe` requires `msys-2.0.dll` and `msys-iconv-2.dll` which were not in PATH
- **Fix:** Copied MSYS DLLs from Git for Windows (`C:\Program Files\Git\usr\bin\`) to `C:\mingw64-gcc\mingw64\bin\`
- **Files modified:** None (system tool fix, outside repo)
- **Verification:** `m4 --version` works with MinGW in PATH

---

**Total deviations:** 2 auto-fixed (both blocking)
**Impact on plan:** Both fixes were necessary for the local build environment only. The CI workflow (Phase 29 Plan 02) uses MSYS2 which does not have these issues. No scope creep.

## Issues Encountered
- First build attempt failed with Cygwin symlink incompatibility (see Deviation 1)
- Second attempt (with ln shim) failed with libtool `lib` command not found (same root cause: Cygwin vs MinGW mismatch)
- Third approach: manual GMP/MPFR/MPC build with correct `--host` flag succeeded, then populated gmp-mpfr-sys cache
- MPC configure generated `config.h` with `\r` inside a string literal due to Cygwin/Windows line ending mismatch; fixed with `sed -i 's/\r//'`
- gmp-mpfr-sys cache directory structure required investigation: files go at `{LocalAppData}/gmp-mpfr-sys/{version_prefix}/{target}/{version_prefix}.{patch}/`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Static linking verified locally; ready for CI workflow update (29-02)
- Binary at 1.78MB with static GMP/MPFR/MPC (acceptable size increase)
- The `.cargo/config.toml` is empty; local dev builds now use gmp-mpfr-sys cache instead of system libraries

---
*Phase: 29-static-linking*
*Completed: 2026-02-18*
