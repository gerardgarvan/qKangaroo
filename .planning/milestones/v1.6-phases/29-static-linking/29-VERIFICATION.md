---
phase: 29-static-linking
verified: 2026-02-18T20:30:00Z
status: passed
score: 7/7 must-haves verified
re_verification: false
must_haves:
  truths:
    - "The q-kangaroo.exe binary has zero non-system DLL dependencies (no libgmp, libmpfr, libmpc, libgcc, libwinpthread)"
    - "GMP/MPFR/MPC are compiled from bundled C source by gmp-mpfr-sys build.rs, not linked from system libraries"
    - "The .cargo/config.toml no longer points to pre-installed GMP paths"
    - "CI builds GMP/MPFR/MPC from bundled source without pre-installed system libraries"
    - "The Windows release archive contains exactly one file: q-kangaroo.exe (no DLLs)"
    - "The Linux release archive contains exactly one file: q-kangaroo"
    - "CI verifies the built binaries have no non-system shared library dependencies"
---

# Phase 29: Static Linking Verification Report

**Phase Goal:** Users download a single executable file with zero external dependencies
**Verified:** 2026-02-18T20:30:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | q-kangaroo.exe has zero non-system DLL dependencies | VERIFIED | `objdump -p` shows only KERNEL32.dll, USER32.dll, ntdll.dll, ADVAPI32.dll, bcryptprimitives.dll, api-ms-win-crt-*.dll, api-ms-win-core-synch-l1-2-0.dll. No libgmp/libmpfr/libmpc/libgcc/libwinpthread. |
| 2 | GMP/MPFR/MPC compiled from bundled C source | VERIFIED | `crates/qsym-core/Cargo.toml` line 10: `gmp-mpfr-sys = "1.6"` (no `use-system-libs` feature). Without this feature, gmp-mpfr-sys builds from bundled source. |
| 3 | .cargo/config.toml has no system GMP paths | VERIFIED | File is empty (no `[env]` section, no LIBRARY_PATH, C_INCLUDE_PATH, PKG_CONFIG_PATH, CFLAGS, or LDFLAGS). |
| 4 | CI builds from bundled GMP source (no pre-installed libs) | VERIFIED | `cli-release.yml`: Linux has no `apt-get install` for GMP; Windows MSYS2 installs only `diffutils m4 make mingw-w64-x86_64-gcc` (build tools only, no gmp/mpfr/mpc packages). |
| 5 | Windows release archive contains single exe (no DLLs) | VERIFIED | `cli-release.yml` line 104: `7z a q-kangaroo-windows-x86_64.zip q-kangaroo.exe`. No DLL copy steps. No `.dll` references in entire workflow. |
| 6 | Linux release archive contains single binary | VERIFIED | `cli-release.yml` line 51: `tar czf q-kangaroo-linux-x86_64.tar.gz q-kangaroo`. Single file. |
| 7 | CI verifies no non-system shared library deps | VERIFIED | Linux: `ldd` + `grep -q "libgmp\|libmpfr\|libmpc"` with `exit 1` on match (lines 40-46). Windows: `objdump -p` + `grep -qi "libgmp\|libmpfr\|libmpc\|libgcc\|libwinpthread"` with `exit 1` on match (lines 92-98). |

**Score:** 7/7 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/qsym-core/Cargo.toml` | `gmp-mpfr-sys = "1.6"` (no use-system-libs) | VERIFIED | Line 10: `gmp-mpfr-sys = "1.6"`. Grep for `use-system-libs` returns no matches. |
| `.cargo/config.toml` | Empty or no system GMP paths | VERIFIED | File exists but is empty (1 line). No LIBRARY_PATH, C_INCLUDE_PATH, etc. |
| `.github/workflows/cli-release.yml` | CI with static builds and single-file archives | VERIFIED | 129 lines. Build tools only (no pre-built GMP). MSYS2 shell for Windows build. Dependency verification steps. Single-file packaging. |
| `target/release/q-kangaroo.exe` | Locally built static binary | VERIFIED | 1,780,224 bytes (1.78MB). Runs: `q-kangaroo 0.1.0`. Zero non-system DLL deps per objdump. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `crates/qsym-core/Cargo.toml` | gmp-mpfr-sys build.rs | Feature flag removal triggers bundled build | WIRED | `gmp-mpfr-sys = "1.6"` (no features) confirmed; binary has static GMP per objdump |
| `cli-release.yml` (build-windows) | gmp-mpfr-sys build.rs | MSYS2 shell provides sh/m4/make/gcc | WIRED | `shell: msys2 {0}` on lines 84, 89; `install: diffutils m4 make mingw-w64-x86_64-gcc` on line 71 |
| `cli-release.yml` (build-linux) | gmp-mpfr-sys build.rs | Ubuntu pre-installed gcc/m4/make/diffutils | WIRED | `cargo build --release -p qsym-cli` on line 31; no apt-get for GMP |
| `cli-release.yml` (build-windows) | dist/q-kangaroo-windows-x86_64.zip | Single exe packaging | WIRED | `7z a q-kangaroo-windows-x86_64.zip q-kangaroo.exe` on line 104; no DLL copy steps anywhere |

### Requirements Coverage

| Requirement | Status | Blocking Issue |
|-------------|--------|----------------|
| BUILD-01: Binary has zero DLL dependencies | SATISFIED | None -- objdump confirms zero non-system DLL deps |
| BUILD-02: CI builds from bundled GMP source | SATISFIED | None -- no pre-installed GMP packages in CI workflow |
| BUILD-03: Release archive contains only executable | SATISFIED | None -- single-file archives for both Windows and Linux |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | - | - | - | No anti-patterns found in any modified files |

No TODO, FIXME, HACK, PLACEHOLDER, or stub patterns found in any of the three modified files.

### Human Verification Required

### 1. CI Workflow Execution

**Test:** Push a tag (e.g., `v0.1.1`) to trigger the `cli-release.yml` workflow on GitHub Actions.
**Expected:** Both Linux and Windows builds succeed. Dependency verification steps pass. Release archives contain single files. GitHub Release is created with two archives.
**Why human:** CI workflow has not been executed yet (only local verification). MSYS2 shell behavior, runner tool availability, and cache paths can only be validated by running the actual workflow.

### 2. Clean Machine Binary Execution

**Test:** Download the Windows release zip on a machine that has never had GMP/MPFR/MPC installed. Extract and run `q-kangaroo.exe --version`.
**Expected:** Prints version and exits successfully with no "DLL not found" errors.
**Why human:** While objdump confirms no non-system DLL imports, runtime behavior on a truly clean machine is the definitive test.

### Gaps Summary

No gaps found. All seven must-have truths are verified against the actual codebase:

1. **Local binary:** The `target/release/q-kangaroo.exe` is 1.78MB and runs successfully. `objdump -p` confirms it depends only on Windows system DLLs (KERNEL32, USER32, ntdll, ADVAPI32, bcryptprimitives, api-ms-win-crt-*). No libgmp, libmpfr, libmpc, libgcc, or libwinpthread.

2. **Cargo.toml:** The `use-system-libs` feature is fully removed. `gmp-mpfr-sys = "1.6"` triggers the bundled source build path.

3. **.cargo/config.toml:** Completely empty -- no stale environment variables pointing to system GMP.

4. **CI workflow:** Fully rewritten for static builds. Both platforms install only build tools. Both have automated dependency verification gates. Both produce single-file archives.

5. **Commits:** Three feature commits verified: `5a46f41` (Cargo.toml + config.toml), `155ef40` (Linux CI), `70c59d3` (Windows CI).

The two human verification items (CI execution and clean machine test) are standard deployment-level checks that cannot be automated in this verification pass, but all code-level prerequisites are fully in place.

---

_Verified: 2026-02-18T20:30:00Z_
_Verifier: Claude (gsd-verifier)_
