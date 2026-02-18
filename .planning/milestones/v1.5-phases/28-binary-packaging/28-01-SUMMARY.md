---
phase: 28-binary-packaging
plan: 01
subsystem: infra
tags: [ci, github-actions, lto, release, binary, dll-bundling]

# Dependency graph
requires:
  - phase: 27-output-commands
    provides: complete CLI binary (q-kangaroo.exe with REPL, parser, evaluator)
provides:
  - Release profile optimization (LTO, strip, codegen-units=1) in workspace Cargo.toml
  - cli-release.yml workflow for automated binary builds on version tags
  - --version flag on CLI binary
  - Windows binary bundled with 5 MinGW DLLs in zip
  - Linux binary packaged as tar.gz
  - GitHub Release creation with auto-generated release notes
affects: []

# Tech tracking
tech-stack:
  added: [softprops/action-gh-release@v2, msys2/setup-msys2@v2]
  patterns: [release-profile-optimization, dll-bundling-in-ci]

key-files:
  created:
    - .github/workflows/cli-release.yml
  modified:
    - Cargo.toml
    - crates/qsym-cli/src/main.rs

key-decisions:
  - "LTO + strip + codegen-units=1 reduces binary from ~4.5MB to ~1.4MB"
  - "Bundle 5 MinGW DLLs on Windows (no static GMP linking)"
  - "Separate cli-release.yml from existing release.yml (Python wheels)"
  - "--version flag for CI build verification"

patterns-established:
  - "Release profile: LTO + strip + codegen-units=1 for optimized binaries"
  - "DLL bundling: copy MinGW runtime DLLs alongside .exe in zip"

requirements-completed: [BIN-01, BIN-02]

# Metrics
duration: 3min
completed: 2026-02-18
---

# Phase 28 Plan 1: Release Build Configuration and CI Workflow Summary

**Release-optimized CLI binary (1.4MB with LTO+strip) with GitHub Actions workflow building Windows+Linux binaries on version tags**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-18T05:27:23Z
- **Completed:** 2026-02-18T05:30:00Z
- **Tasks:** 4
- **Files modified:** 3

## Accomplishments
- Release profile optimization reduces binary size from ~4.5MB to ~1.4MB via LTO, strip, codegen-units=1
- cli-release.yml workflow builds Linux (tar.gz) and Windows (zip with 5 DLLs) binaries on v* tags
- --version flag enables CI verification of built binaries
- All 294 existing CLI tests pass with no regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Add release profile to workspace Cargo.toml** - `b8e3f83` (feat)
2. **Task 2: Create cli-release.yml CI workflow** - `c9badc3` (feat)
3. **Task 3: Add --version flag to CLI** - `034b929` (feat)
4. **Task 4: Build verification** - no code changes (verification only)

## Files Created/Modified
- `Cargo.toml` - Added [profile.release] section with LTO, strip, codegen-units=1
- `.github/workflows/cli-release.yml` - New CI workflow: build-linux, build-windows, create-release jobs
- `crates/qsym-cli/src/main.rs` - Added --version / -V flag handling before REPL entry

## Decisions Made
- LTO + strip + codegen-units=1 chosen for optimal binary size/performance tradeoff (1.4MB vs ~4.5MB)
- Bundle 5 MinGW DLLs on Windows rather than static linking (libgmp-10, libmpfr-6, libmpc-3, libgcc_s_seh-1, libwinpthread-1)
- Separate cli-release.yml from existing release.yml to keep Python wheel and CLI binary pipelines independent
- --version flag uses simple args check (no clap dependency) for minimal overhead

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required. The workflow triggers automatically on v* tags and workflow_dispatch.

## Next Phase Readiness

Phase 28 is the final phase. The project is complete:
- v1.0-v1.4: Core engine, Python API, packaging, docs, polynomial infrastructure (Phases 1-22)
- v1.5: Interactive REPL with parser, evaluator, function dispatch, session management, output commands, and binary packaging (Phases 23-28)
- All 79 plans across 28 phases shipped

## Self-Check: PASSED

All files verified present. All 3 task commits verified in git log.

---
*Phase: 28-binary-packaging*
*Completed: 2026-02-18*
