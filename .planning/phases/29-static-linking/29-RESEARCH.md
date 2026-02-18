# Phase 29: Static Linking - Research

**Researched:** 2026-02-18
**Domain:** Static linking of GMP/MPFR/MPC via gmp-mpfr-sys, CI workflow changes
**Confidence:** HIGH

## Summary

Phase 29 eliminates all non-system DLL dependencies from the q-Kangaroo Windows binary and all non-standard shared library dependencies from the Linux binary. The core change is a single line: removing the `use-system-libs` feature from the `gmp-mpfr-sys` dependency in `crates/qsym-core/Cargo.toml`. Without this feature, gmp-mpfr-sys builds GMP 6.3.0, MPFR 4.2.2, and MPC 1.3.1 from bundled C source code and produces static `.a` libraries that get linked into the final binary.

Empirical verification of the current release binary (via `objdump -p`) confirms that the only non-system DLL dependency is `libgmp-10.dll`. Contrary to the milestone research note, `libgcc_s_seh-1.dll` and `libwinpthread-1.dll` are NOT runtime dependencies of the current binary -- Rust's windows-gnu toolchain statically links libgcc_eh and libpthread by default. The CI workflow bundles these DLLs unnecessarily. After static GMP linking, the Windows binary will depend only on standard Windows system DLLs (KERNEL32, USER32, bcryptprimitives, ntdll, ADVAPI32, and UCRT api-ms-win-crt-* DLLs).

The CI workflow changes are significant but well-understood. The Windows build must switch from pre-built GMP packages to MSYS2 build tools (diffutils, m4, make, mingw-w64-x86_64-gcc) and run `cargo build` from the MSYS2 shell (because gmp-mpfr-sys's build.rs invokes `sh -c configure` and `make`). The Linux build drops `libgmp-dev libmpfr-dev libmpc-dev` and needs only gcc, m4, make, diffutils (all pre-installed on ubuntu-latest). The `.cargo/config.toml` environment variables pointing to the system GMP location must be removed or made conditional. The DLL-copying and zip-a-directory packaging steps are eliminated entirely.

**Primary recommendation:** Remove `features = ["use-system-libs"]` from gmp-mpfr-sys in Cargo.toml, update CI to install build tools instead of pre-built GMP packages, run Windows cargo build from MSYS2 shell, eliminate DLL bundling, and update `.cargo/config.toml` to remove system library paths.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| BUILD-01 | Binary has zero DLL dependencies (static GMP/MPFR/MPC linking) | Remove `use-system-libs` feature from gmp-mpfr-sys; builds GMP/MPFR/MPC from bundled source as static `.a` libraries; verified via docs.rs build.rs source |
| BUILD-02 | CI workflow builds from bundled GMP source (no pre-installed system libs) | Replace `libgmp-dev` (Linux) and `mingw-w64-x86_64-gmp` (Windows) with build tools only; gmp-mpfr-sys build.rs handles configure/make automatically |
| BUILD-03 | Release archive contains only the executable (no DLL files) | Eliminate DLL copy steps and directory packaging; archive contains single file (q-kangaroo.exe or q-kangaroo) |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| gmp-mpfr-sys | 1.6.8 | FFI bindings + bundled GMP/MPFR/MPC source | Already a dependency; removing `use-system-libs` triggers static build from bundled source |
| rug | 1.28.1 | High-level Rust API for GMP integers/rationals | Already a dependency; no changes needed |

### Supporting (CI)
| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| msys2/setup-msys2@v2 | v2 | Provides MSYS2 MinGW64 environment on Windows CI | Windows build job only |
| actions/cache@v4 | v4 | Caches cargo registry, target dir, AND gmp-mpfr-sys cache dir | Both Linux and Windows builds |

### Bundled C Library Versions (from gmp-mpfr-sys 1.6.8)
| Library | Version | Notes |
|---------|---------|-------|
| GMP | 6.3.0 | Built from source by gmp-mpfr-sys build.rs |
| MPFR | 4.2.2 | Built from source by gmp-mpfr-sys build.rs |
| MPC | 1.3.1 | Built from source by gmp-mpfr-sys build.rs |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| gmp-mpfr-sys bundled build | Manual vendored GMP source + custom build.rs | Far more complex, gmp-mpfr-sys already handles this perfectly |
| MSYS2 shell for cargo | Regular bash shell with MSYS2 tools in PATH | Risky -- gmp-mpfr-sys build.rs calls `sh -c configure` which needs MSYS2's sh for Windows path handling |

## Architecture Patterns

### Change Inventory

The changes for this phase are entirely in build configuration, not application code:

```
Files to modify:
  crates/qsym-core/Cargo.toml          # Remove use-system-libs feature
  .cargo/config.toml                    # Remove system library paths (or make conditional)
  .github/workflows/cli-release.yml     # Major rewrite of both build jobs

Files unchanged:
  All Rust source files (src/**/*.rs)   # Zero code changes
  Cargo.lock                            # Will auto-update on next build
```

### Pattern 1: gmp-mpfr-sys Static Build
**What:** When `use-system-libs` is NOT enabled (the default), gmp-mpfr-sys's build.rs:
1. Extracts bundled GMP/MPFR/MPC C source from the crate
2. Runs `sh -c "../gmp-src/configure ..."` using relative paths
3. Runs `make -j{N}` to compile
4. Produces `.a` static libraries in a cache directory
5. Emits `cargo:rustc-link-lib=static=gmp` (etc.) so Rust links statically
6. Caches the compiled libraries per-version in a platform-specific directory

**When to use:** Always for distribution binaries (zero external dependencies)

### Pattern 2: CI Windows Build from MSYS2 Shell
**What:** The `cargo build` command must be run from within the MSYS2 MinGW64 shell (using `shell: msys2 {0}`) so that gmp-mpfr-sys's build.rs can find `sh`, `make`, `m4`, `gcc`, and `diff` in PATH.

**Why:** The build.rs invokes `Command::new("sh")` and `Command::new("make")`. On Windows, these must resolve to MSYS2 tools. Running from a regular `bash` shell with only `mingw64/bin` in PATH is insufficient because `m4`, `make`, and `diff` live in MSYS2's `/usr/bin/`, not in `mingw64/bin`.

**Example CI step:**
```yaml
- name: Build release binary
  shell: msys2 {0}
  run: |
    export PATH="/mingw64/bin:$PATH"
    cargo build --release -p qsym-cli --target x86_64-pc-windows-gnu
```

### Pattern 3: CI Linux Build (Simplified)
**What:** On ubuntu-latest, gcc, make, m4, and diffutils are all pre-installed. No `apt-get install` of GMP libraries is needed.

**Example CI step:**
```yaml
- name: Build release binary
  run: cargo build --release -p qsym-cli
```

### Pattern 4: Single-File Release Archive
**What:** The Windows release archive changes from a zip containing a directory with 6 files (exe + 5 DLLs) to a zip containing a single exe file.

```yaml
# OLD: directory with DLLs
mkdir -p dist/q-kangaroo-windows
cp target/.../q-kangaroo.exe dist/q-kangaroo-windows/
cp /mingw64/bin/libgmp-10.dll dist/q-kangaroo-windows/
# ... 4 more DLL copies
cd dist && 7z a q-kangaroo-windows-x86_64.zip q-kangaroo-windows/

# NEW: single exe
mkdir -p dist
cp target/x86_64-pc-windows-gnu/release/q-kangaroo.exe dist/
cd dist && 7z a q-kangaroo-windows-x86_64.zip q-kangaroo.exe
```

### Anti-Patterns to Avoid
- **Running cargo from bash shell on Windows CI:** The gmp-mpfr-sys build.rs needs MSYS2's `sh` and build tools. Use `shell: msys2 {0}` for the cargo build step.
- **Keeping LIBRARY_PATH/C_INCLUDE_PATH pointing to system GMP:** These environment variables will cause the build to find system GMP headers and potentially link dynamically. They must be removed or unset.
- **Hardcoding the MSYS2 install path:** Use `${{ steps.msys2.outputs.msys2-location }}` or the known default `D:/a/_temp/msys64` but prefer the output variable for forward compatibility.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| GMP/MPFR/MPC compilation | Custom build.rs with configure/make | gmp-mpfr-sys default mode (no use-system-libs) | gmp-mpfr-sys handles cross-platform configure/make, caching, path quoting, and version pinning |
| DLL dependency checking | Manual ldd/objdump scripts | `objdump -p binary \| grep "DLL Name"` as a CI verification step | Simple one-liner, no tooling needed |
| Static linking flags | Custom RUSTFLAGS for libgcc | Nothing -- Rust already statically links libgcc_eh and libpthread on windows-gnu by default | Empirically verified: current binary has no libgcc or libwinpthread DLL dependencies |

**Key insight:** The entire static linking transformation is achieved by removing one Cargo feature flag. The gmp-mpfr-sys crate handles all the complexity of building GMP/MPFR/MPC from source, caching, and emitting the correct linker flags.

## Common Pitfalls

### Pitfall 1: Build Path Contains Spaces
**What goes wrong:** GMP's configure script fails silently or with cryptic errors when the build directory path contains spaces.
**Why it happens:** Autotools/libtool do not properly quote paths with spaces in many places.
**How to avoid:** Ensure the CI checkout path (`D:/a/Kangaroo/Kangaroo`) and all cache directories have no spaces. GitHub Actions default paths are space-free.
**Warning signs:** Configure step fails with "No such file or directory" errors on path fragments.

### Pitfall 2: Stale .cargo/config.toml Causing Dynamic Linking
**What goes wrong:** Even after removing `use-system-libs`, the build finds system GMP via LIBRARY_PATH/C_INCLUDE_PATH in `.cargo/config.toml` and links dynamically.
**Why it happens:** `.cargo/config.toml` currently sets `LIBRARY_PATH = "C:/mingw64-gcc/mingw64/lib"` etc., which makes the linker find dynamic `.dll.a` import libraries.
**How to avoid:** Remove the `[env]` section from `.cargo/config.toml` entirely, or make it conditional on a feature/profile. For the local dev environment, these paths can be set as shell environment variables instead.
**Warning signs:** `objdump -p` on the built binary still shows `libgmp-10.dll`.

### Pitfall 3: Missing Build Tools in CI
**What goes wrong:** Build fails with "m4: not found" or "make: not found" during GMP configure/make.
**Why it happens:** The MSYS2 install step does not include the required build tools.
**How to avoid:**
- Windows: Install `diffutils m4 make mingw-w64-x86_64-gcc` in MSYS2 (note: NOT the `gmp/mpfr/mpc` packages -- those are the pre-built libraries we're removing)
- Linux: gcc, m4, make, diffutils are all pre-installed on ubuntu-latest (Ubuntu 24.04)
**Warning signs:** Build.rs error mentioning missing tools.

### Pitfall 4: MSYS2 PATH Not Including /usr/bin for Build Tools
**What goes wrong:** `cargo build` from a non-MSYS2 shell cannot find `m4`, `make`, or `diff` even though MSYS2 is installed.
**Why it happens:** Adding only `mingw64/bin` to PATH (as the current workflow does) provides `gcc` but not `m4`, `make`, or `diff` which live in MSYS2's `/usr/bin/`.
**How to avoid:** Run `cargo build` from `shell: msys2 {0}` which sets up the full PATH including both `/mingw64/bin` and `/usr/bin`. Alternatively, add both paths to GITHUB_PATH, but the MSYS2 shell approach is cleaner and documented.
**Warning signs:** "sh: m4: command not found" during configure.

### Pitfall 5: First Build Takes 2-5 Minutes
**What goes wrong:** CI builds take noticeably longer on first run.
**Why it happens:** Compiling GMP/MPFR/MPC from C source is inherently slow (configure + make for 3 libraries).
**How to avoid:** Cache the gmp-mpfr-sys cache directory (`$HOME/.cache/gmp-mpfr-sys` on Linux, `$LOCALAPPDATA/gmp-mpfr-sys` on Windows). Alternatively, use `GMP_MPFR_SYS_CACHE` to set a custom cache location that's easier to cache in CI. Include this path in the `actions/cache` step.
**Warning signs:** Build step takes 3+ minutes when caches are cold.

### Pitfall 6: Cargo Needs Rust Toolchain in MSYS2 Shell PATH
**What goes wrong:** Running `cargo build` from `shell: msys2 {0}` fails with "cargo: command not found".
**Why it happens:** The MSYS2 shell has its own PATH that does not inherit the full Windows PATH by default (path-type: minimal).
**How to avoid:** Either add the cargo/rustup bin directory to PATH within the MSYS2 shell step, or configure `path-type: inherit` on the setup-msys2 action.
**Warning signs:** "cargo: command not found" or "rustc: command not found" when using `shell: msys2 {0}`.

### Pitfall 7: Local Dev Environment Breakage
**What goes wrong:** After removing `use-system-libs` and `.cargo/config.toml` env vars, the local build fails because Cygwin lacks m4.
**Why it happens:** The local dev environment uses Cygwin's tools, not MSYS2. Cygwin has `make` and `diff` but not `m4`.
**How to avoid:** Either install m4 in Cygwin (`apt-cyg install m4`), or ensure the MinGW m4 works (currently broken due to missing DLL), or document that local builds require MSYS2 tools in PATH.
**Warning signs:** "m4: not found" when building locally.

## Code Examples

### Cargo.toml Change (crates/qsym-core/Cargo.toml)
```toml
# BEFORE (dynamic linking against system GMP):
gmp-mpfr-sys = { version = "1.6", features = ["use-system-libs"] }

# AFTER (static linking from bundled source):
gmp-mpfr-sys = "1.6"
```
Source: [gmp-mpfr-sys docs](https://docs.rs/gmp-mpfr-sys/1.6.8/gmp_mpfr_sys/) -- "Using this feature, the system libraries for GMP... will be used instead of building them from source."

### .cargo/config.toml Change
```toml
# BEFORE:
[env]
PKG_CONFIG_PATH = "C:/mingw64-gcc/mingw64/lib/pkgconfig"
C_INCLUDE_PATH = "C:/mingw64-gcc/mingw64/include"
LIBRARY_PATH = "C:/mingw64-gcc/mingw64/lib"
CFLAGS = "-IC:/mingw64-gcc/mingw64/include"
LDFLAGS = "-LC:/mingw64-gcc/mingw64/lib"

# AFTER (empty or removed -- build from source needs no system paths):
# File can be emptied or deleted entirely.
# If keeping for other purposes, remove all GMP-related env vars.
```

### CI Windows Build Job (new)
```yaml
build-windows:
  name: Build Windows Binary
  runs-on: windows-latest
  steps:
    - name: Configure line endings
      shell: bash
      run: git config --global core.autocrlf input
    - uses: actions/checkout@v4
    - name: Setup MSYS2 with build tools
      id: msys2
      uses: msys2/setup-msys2@v2
      with:
        msystem: MINGW64
        update: false
        install: diffutils m4 make mingw-w64-x86_64-gcc
    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: x86_64-pc-windows-gnu
    - uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: windows-release-${{ hashFiles('**/Cargo.lock') }}
    - name: Build release binary
      shell: msys2 {0}
      run: |
        export PATH="$PATH:/c/Users/runneradmin/.cargo/bin"
        cargo build --release -p qsym-cli --target x86_64-pc-windows-gnu
    - name: Verify no DLL dependencies
      shell: msys2 {0}
      run: |
        objdump -p target/x86_64-pc-windows-gnu/release/q-kangaroo.exe | grep "DLL Name"
        # Should show only Windows system DLLs, no libgmp/libmpfr/libmpc/libgcc/libwinpthread
    - name: Package Windows binary
      shell: bash
      run: |
        mkdir -p dist
        cp target/x86_64-pc-windows-gnu/release/q-kangaroo.exe dist/
        cd dist && 7z a q-kangaroo-windows-x86_64.zip q-kangaroo.exe
    - uses: actions/upload-artifact@v4
      with:
        name: binary-windows
        path: dist/q-kangaroo-windows-x86_64.zip
```

### CI Linux Build Job (simplified)
```yaml
build-linux:
  name: Build Linux Binary
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          ~/.cache/gmp-mpfr-sys
          target
        key: linux-release-${{ hashFiles('**/Cargo.lock') }}
    # No apt-get install needed -- gcc, m4, make, diffutils are pre-installed
    - name: Build release binary
      run: cargo build --release -p qsym-cli
    - name: Verify binary runs
      run: |
        chmod +x target/release/q-kangaroo
        target/release/q-kangaroo --version
    - name: Verify no GMP shared library dependency
      run: |
        ldd target/release/q-kangaroo | grep -v "libgmp" || true
        ! ldd target/release/q-kangaroo | grep "libgmp"
    - name: Package Linux binary
      run: |
        mkdir -p dist
        cp target/release/q-kangaroo dist/
        cd dist && tar czf q-kangaroo-linux-x86_64.tar.gz q-kangaroo
    - uses: actions/upload-artifact@v4
      with:
        name: binary-linux
        path: dist/q-kangaroo-linux-x86_64.tar.gz
```

### DLL Dependency Verification (local)
```bash
# On Windows (Cygwin/MinGW):
objdump -p target/release/q-kangaroo.exe | grep "DLL Name"
# Expected: only system DLLs (KERNEL32, USER32, ntdll, ADVAPI32, bcryptprimitives, api-ms-win-crt-*)
# Must NOT show: libgmp-10.dll, libmpfr-6.dll, libmpc-3.dll, libgcc_s_seh-1.dll, libwinpthread-1.dll

# On Linux:
ldd target/release/q-kangaroo
# Expected: only system libs (libc, libm, libpthread, libdl, ld-linux)
# Must NOT show: libgmp.so, libmpfr.so, libmpc.so
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `use-system-libs` + bundled DLLs | Remove feature, build from source, static link | gmp-mpfr-sys has always supported this | Eliminates all non-system DLL deps |
| Install `libgmp-dev` on Linux CI | Pre-installed gcc/m4/make/diffutils sufficient | Ubuntu runners have had these tools for years | Simpler CI, no apt-get needed |
| 6-file Windows release archive | 1-file release archive (just .exe) | This phase | Simpler distribution |

**Key version info:**
- gmp-mpfr-sys 1.6.8 bundles GMP 6.3.0, MPFR 4.2.2, MPC 1.3.1
- Requires rustc 1.65.0+ (project uses 1.85.0, well above minimum)
- Build tools needed: gcc, m4, make, diffutils, sh

## Open Questions

1. **Exact cargo binary path in MSYS2 shell on GitHub Actions**
   - What we know: `dtolnay/rust-toolchain@stable` installs to `C:\Users\runneradmin\.cargo\bin`
   - What's unclear: Whether this path is automatically in the MSYS2 shell's PATH with `path-type: minimal`, or needs explicit addition
   - Recommendation: Add explicit `export PATH="$PATH:/c/Users/runneradmin/.cargo/bin"` in the MSYS2 shell step, or use `path-type: inherit`

2. **Binary size after static linking**
   - What we know: Current binary with dynamic GMP is 1.4 MB (with LTO + strip)
   - What's unclear: Static GMP/MPFR/MPC will increase binary size (estimated 2-4 MB range)
   - Recommendation: Accept the size increase -- a single portable binary is worth a few extra MB

3. **Local development workflow after removing .cargo/config.toml**
   - What we know: Local dev currently depends on `.cargo/config.toml` pointing to pre-built GMP at `C:/mingw64-gcc/mingw64/`
   - What's unclear: Whether the local environment has all tools needed for building from source (m4 is currently broken in MinGW)
   - Recommendation: Fix local m4 (install Cygwin m4 or fix MinGW m4 DLL issue), or keep `use-system-libs` as a developer convenience behind a cargo feature flag

4. **Caching gmp-mpfr-sys build artifacts in CI**
   - What we know: gmp-mpfr-sys caches compiled libraries in platform-specific directories (`$HOME/.cache/gmp-mpfr-sys` on Linux, `$LOCALAPPDATA\gmp-mpfr-sys` on Windows)
   - What's unclear: Whether `actions/cache` with the `target` directory already captures these, or if the cache dir needs separate inclusion
   - Recommendation: Explicitly add the gmp-mpfr-sys cache directories to the `actions/cache` paths. On Windows in MSYS2, `$LOCALAPPDATA` maps to `C:\Users\runneradmin\AppData\Local`.

## Sources

### Primary (HIGH confidence)
- [gmp-mpfr-sys 1.6.8 official documentation](https://docs.rs/gmp-mpfr-sys/1.6.8/gmp_mpfr_sys/) -- feature flags, build requirements, bundled versions, caching behavior
- [gmp-mpfr-sys build.rs source](https://docs.rs/crate/gmp-mpfr-sys/1.6.8/source/build.rs) -- confirmed uses `sh -c configure`, `make`, relative paths, `cargo:rustc-link-lib=static=gmp`
- Local binary inspection: `objdump -p target/release/q-kangaroo.exe` -- confirmed only `libgmp-10.dll` is non-system DLL dep; libgcc/libwinpthread are NOT dependencies
- [v1.6 milestone SUMMARY.md](../../research/SUMMARY.md) -- pre-research confirming approach

### Secondary (MEDIUM confidence)
- [Rust windows-gnu libgcc linking issue #89919](https://github.com/rust-lang/rust/issues/89919) -- confirms Rust statically links libgcc_eh and libpthread by default on windows-gnu
- [msys2/setup-msys2 GitHub Action](https://github.com/msys2/setup-msys2) -- path-type option, msys2-location output, shell usage
- [GitHub Actions Ubuntu 24.04 runner image](https://github.com/actions/runner-images) -- gcc, m4, make pre-installed
- [gmp-mpfr-sys GitLab (tspiteri)](https://tspiteri.gitlab.io/gmp-mpfr-sys/) -- Windows MSYS2 instructions, path restrictions, caching

### Tertiary (LOW confidence)
- Build time estimates (2-5 minutes for cold GMP source build) -- from milestone research, not empirically measured on CI runners

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- single crate change, verified in official docs and build.rs source
- Architecture: HIGH -- CI workflow patterns verified against existing workflow + MSYS2 docs
- Pitfalls: HIGH -- path issues, tool availability, and shell requirements all verified through multiple sources and local testing

**Research date:** 2026-02-18
**Valid until:** 2026-03-18 (stable domain -- gmp-mpfr-sys rarely changes build behavior)
