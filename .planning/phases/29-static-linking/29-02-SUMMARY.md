---
phase: 29-static-linking
plan: 02
subsystem: infra
tags: [github-actions, ci, static-linking, gmp, msys2, single-exe]

# Dependency graph
requires:
  - phase: 29-static-linking
    plan: 01
    provides: "Static GMP linking via gmp-mpfr-sys default mode (no use-system-libs)"
provides:
  - "CI release workflow building from bundled GMP source (no pre-installed libraries)"
  - "Single-file release archives (no DLLs)"
  - "Automated dependency verification in CI (ldd on Linux, objdump on Windows)"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "MSYS2 shell for cargo build on Windows CI (gmp-mpfr-sys build.rs compatibility)"
    - "gmp-mpfr-sys cache directories in actions/cache for faster CI rebuilds"
    - "Post-build dependency verification (ldd/objdump) to prevent regression"

key-files:
  created: []
  modified:
    - ".github/workflows/cli-release.yml"

key-decisions:
  - "Run cargo from MSYS2 shell on Windows CI (not bash with PATH additions) for full build tool compatibility"
  - "Cache gmp-mpfr-sys build artifacts separately from target dir for reliable cross-build caching"
  - "Use objdump DLL Name grep (Windows) and ldd grep (Linux) as CI gates to prevent dependency regressions"

patterns-established:
  - "MSYS2 shell build pattern: shell: msys2 {0} with explicit cargo PATH export"
  - "Dependency verification gates: CI step that fails build if non-system dependencies detected"

requirements-completed: [BUILD-02, BUILD-03]

# Metrics
duration: 2min
completed: 2026-02-18
---

# Phase 29 Plan 02: CI Release Workflow Summary

**CI builds GMP/MPFR/MPC from bundled source with MSYS2 build tools, produces single-file release archives, and verifies no non-system dependencies**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-18T18:24:10Z
- **Completed:** 2026-02-18T18:25:39Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Linux CI build drops apt-get install of libgmp-dev/libmpfr-dev/libmpc-dev; relies on pre-installed gcc/m4/make/diffutils
- Windows CI build replaces pre-built GMP packages with build tools only (diffutils m4 make mingw-w64-x86_64-gcc)
- Windows cargo build runs from MSYS2 shell for gmp-mpfr-sys build.rs compatibility
- Both builds include automated dependency verification (ldd on Linux, objdump on Windows)
- Windows archive contains single q-kangaroo.exe (no DLLs); Linux archive contains single q-kangaroo
- gmp-mpfr-sys cache directories added to actions/cache for faster subsequent builds

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite Linux build job for static GMP** - `155ef40` (feat)
2. **Task 2: Rewrite Windows build job and packaging for static single-exe** - `70c59d3` (feat)

## Files Created/Modified
- `.github/workflows/cli-release.yml` - CI release workflow with static builds, dependency verification, and single-file archives

## Decisions Made
- Run cargo from MSYS2 shell on Windows CI (not bash with PATH additions) for full build tool compatibility -- gmp-mpfr-sys build.rs invokes sh/m4/make which need MSYS2's /usr/bin in PATH
- Cache gmp-mpfr-sys build artifacts in platform-specific directories (~/.cache/gmp-mpfr-sys on Linux, C:/Users/runneradmin/AppData/Local/gmp-mpfr-sys on Windows)
- Use objdump DLL Name grep on Windows and ldd grep on Linux as CI gates to prevent dependency regressions

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- CI workflow is ready for static-linked binaries (requires Plan 01 Cargo.toml changes to be in effect)
- First CI run with static GMP will be slower (~2-5 min for GMP source build) but subsequent runs will use gmp-mpfr-sys cache

## Self-Check: PASSED

- FOUND: .github/workflows/cli-release.yml
- FOUND: .planning/phases/29-static-linking/29-02-SUMMARY.md
- FOUND: commit 155ef40
- FOUND: commit 70c59d3
- OK: No pre-built GMP package references in workflow
- OK: No DLL copy references in workflow
- OK: Linux dependency verification step exists
- OK: Windows dependency verification step exists

---
*Phase: 29-static-linking*
*Completed: 2026-02-18*
