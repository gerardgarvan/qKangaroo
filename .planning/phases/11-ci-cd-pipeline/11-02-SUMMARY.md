---
phase: 11-ci-cd-pipeline
plan: 02
subsystem: infra
tags: [github-actions, release, maturin, manylinux, msys2, pypi, oidc, wheels]

requires:
  - phase: 11-ci-cd-pipeline
    plan: 01
    provides: CI workflow with Rust tests, Python tests, and coverage
  - phase: 10-pypi-packaging-metadata
    provides: pyproject.toml with maturin config, ABI3 settings, DLL bundling
provides:
  - Release workflow with Linux wheels, Windows wheels, sdist, and OIDC PyPI publishing
affects: [12-documentation]

tech-stack:
  added: [maturin-action, manylinux2014, msys2-setup, pypi-oidc-trusted-publishing]
  patterns: [tag-triggered-release, oidc-publish-no-tokens, dll-bundling-in-ci]

key-files:
  created:
    - .github/workflows/release.yml

key-decisions:
  - "Used OIDC trusted publishing (id-token: write) instead of stored PyPI API tokens"
  - "Used LIBRARY_PATH and C_INCLUDE_PATH env vars for MSYS2 GMP linkage on Windows"
  - "Bundled libgmp-10.dll, libgcc_s_seh-1.dll, libwinpthread-1.dll for Windows wheels (not libmpfr/libmpc)"
  - "Used hardcoded D:/a/_temp/msys64/mingw64/bin path for MSYS2 on GitHub Actions runners"
  - "Set git core.autocrlf input BEFORE checkout to prevent Windows line ending issues"

patterns-established:
  - "Release trigger: push tags v* only (no branch push, no PR)"
  - "Artifact naming: wheels-linux, wheels-windows, wheels-sdist with merge-multiple download"
  - "Windows CI GMP: msys2/setup-msys2@v2 with MINGW64 and DLL copy into package directory"
  - "OIDC publish: job-level permissions (id-token: write), environment: pypi, no secrets"

duration: 1min
completed: 2026-02-15
---

# Phase 11 Plan 02: Release Workflow Summary

**Tag-triggered release workflow building manylinux2014 Linux wheels, MinGW Windows wheels with bundled GMP DLLs, and sdist -- publishing to PyPI via OIDC trusted publishing with zero stored tokens**

## Performance

- **Duration:** 1 min
- **Started:** 2026-02-15T18:44:46Z
- **Completed:** 2026-02-15T18:45:32Z
- **Tasks:** 2 (1 auto + 1 non-blocking checkpoint auto-approved)
- **Files created:** 1

## Accomplishments
- Release workflow with four jobs triggered by version tags (v*)
- Linux wheels via maturin-action with manylinux2014 container and GMP (yum install gmp-devel)
- Windows wheels via MSYS2 MinGW64 with GNU Rust target and bundled DLLs (libgmp-10, libgcc_s_seh-1, libwinpthread-1)
- Source distribution (sdist) built alongside wheels
- OIDC trusted publishing to PyPI with no stored API tokens (id-token: write at job level)
- Artifact download with merge-multiple pattern for unified dist/ directory

## Task Commits

Each task was committed atomically:

1. **Task 1: Create release workflow with all four jobs** - `c9ccd85` (feat)
2. **Task 2: Verify workflow files** - auto-approved (non-blocking checkpoint in YOLO mode)

## Files Created/Modified
- `.github/workflows/release.yml` - Release workflow with linux-wheels, windows-wheels, sdist, and publish jobs

## Decisions Made
- Used OIDC trusted publishing (id-token: write at job level, environment: pypi) instead of stored PyPI API tokens -- more secure, no secret rotation needed
- Bundled 3 MinGW DLLs (libgmp-10, libgcc_s_seh-1, libwinpthread-1) -- these are the runtime deps; libmpfr and libmpc were included in local builds but are not needed by the .pyd
- Set `git config --global core.autocrlf input` BEFORE checkout step to prevent Windows line ending corruption
- Used hardcoded MSYS2 path (D:/a/_temp/msys64/mingw64/) as msys2/setup-msys2 does not expose a location output
- Set LIBRARY_PATH and C_INCLUDE_PATH for the maturin build step to find MSYS2 GMP headers and libraries

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

Before the release workflow can successfully publish, the user must complete these external configuration steps:

### PyPI Trusted Publisher (Required for CI-07)
1. **Register package on PyPI** (if not already): https://pypi.org/manage/projects/
2. **Configure trusted publisher** at https://pypi.org/manage/project/q-kangaroo/settings/publishing/
   - Owner: `<github-username>`
   - Repository: `q-kangaroo`
   - Workflow name: `release.yml`
   - Environment name: `pypi`

### GitHub Repository Environment (Required for OIDC)
1. Go to Repository Settings > Environments > New environment
2. Create environment named `pypi`
3. No secrets needed -- OIDC handles authentication

### Codecov Token (Referenced by Plan 01)
1. Sign in at https://app.codecov.io/ with GitHub
2. Add the q-kangaroo repository
3. Copy the Upload Token
4. Add as `CODECOV_TOKEN` secret in GitHub repository settings

### First Release
1. Push all code to GitHub (owner/q-kangaroo)
2. Tag a release: `git tag v0.1.0 && git push origin v0.1.0`
3. Watch the Actions tab for the release workflow run
4. Verify wheels appear on PyPI after successful publish

## Next Phase Readiness
- Both CI workflows complete (ci.yml + release.yml) -- all 7 CI requirements (CI-01 through CI-07) covered
- Phase 11 (CI/CD Pipeline) is fully complete
- Phase 12 (Documentation & UX) can proceed

## Self-Check: PASSED

- FOUND: .github/workflows/release.yml
- FOUND: 11-02-SUMMARY.md
- FOUND: commit c9ccd85

---
*Phase: 11-ci-cd-pipeline*
*Completed: 2026-02-15*
