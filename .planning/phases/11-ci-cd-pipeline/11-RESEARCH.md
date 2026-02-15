# Phase 11: CI/CD Pipeline - Research

**Researched:** 2026-02-15
**Domain:** GitHub Actions CI/CD, maturin wheel builds, PyPI trusted publishing, Rust code coverage
**Confidence:** HIGH

## Summary

This phase creates GitHub Actions workflows for automated testing, wheel building, and PyPI publishing for the q-kangaroo project -- a Rust+PyO3 native Python extension using maturin as its build backend. The project has a workspace layout with pyproject.toml in `crates/qsym-python/` and depends on GMP (via `gmp-mpfr-sys` with `use-system-libs`), which is the primary CI complexity driver.

The recommended approach uses three workflow files: (1) a CI workflow for testing on every push/PR, (2) a build workflow for wheel and sdist creation, and (3) a release workflow triggered by version tags that publishes to PyPI via OIDC trusted publishing. The maturin-action (`PyO3/maturin-action@v1`) handles wheel builds natively for both Linux (manylinux2014 container with `yum install -y gmp-devel` via `before-script-linux`) and Windows (MSYS2 setup with `mingw-w64-x86_64-gmp` + GNU Rust toolchain). Coverage uses `cargo-tarpaulin` on Linux with Codecov upload and badge.

**Primary recommendation:** Use `PyO3/maturin-action@v1` with `working-directory: crates/qsym-python` for wheel builds, `msys2/setup-msys2@v2` for Windows GMP setup, `cargo-tarpaulin` for coverage on Linux, and `pypa/gh-action-pypi-publish@release/v1` for OIDC trusted publishing.

## Standard Stack

### Core

| Tool/Action | Version | Purpose | Why Standard |
|-------------|---------|---------|--------------|
| `PyO3/maturin-action` | `@v1` (pin to specific like `@v1.49.4`) | Build manylinux/Windows wheels + sdist | Official maturin GitHub Action, handles manylinux containers, cross-compilation |
| `pypa/gh-action-pypi-publish` | `@release/v1` | Upload wheels/sdist to PyPI via OIDC | Official PyPA action, recommended by PyPI docs |
| `msys2/setup-msys2` | `@v2` | Install MinGW GMP on Windows runners | Standard way to get MSYS2/MinGW packages in GitHub Actions |
| `actions/upload-artifact` | `@v4` | Pass wheels between build and release jobs | Official GitHub artifact action |
| `actions/download-artifact` | `@v4` | Retrieve wheels in release job | Official GitHub artifact action |
| `codecov/codecov-action` | `@v5` | Upload coverage reports to Codecov | Industry standard, free for public repos |
| `cargo-tarpaulin` | latest (0.31+) | Rust code coverage generation | Most popular Rust coverage tool, works on stable |
| `actions/checkout` | `@v4` | Repository checkout | Standard |
| `dtolnay/rust-toolchain` | `@stable` | Install Rust toolchain | De facto standard Rust toolchain action |
| `actions/setup-python` | `@v5` | Install Python for test jobs | Standard Python setup action |

### Supporting

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| `sccache` | via maturin-action `sccache: true` | Cargo build caching | Speed up repeated builds (optional) |
| `actions/cache` | `@v4` | Cache cargo registry/target | Speed up Rust test jobs |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| cargo-tarpaulin | cargo-llvm-cov | llvm-cov is more accurate but tarpaulin works on stable and is simpler; tarpaulin is Linux-only which is fine since coverage only needs one platform |
| Codecov | Coveralls | Both free for open source; Codecov has better Rust ecosystem support and simpler setup |
| maturin-action | cibuildwheel | cibuildwheel is for pure Python/C extensions; maturin-action is purpose-built for maturin/PyO3 |

## Architecture Patterns

### Recommended Workflow File Structure

```
.github/
  workflows/
    ci.yml          # Runs on every push/PR: Rust tests + Python integration tests + coverage
    build.yml       # Runs on every push/PR: wheel builds (Linux + Windows) + sdist
    release.yml     # Runs on version tags: download artifacts, publish to PyPI via OIDC
```

**Rationale for three files:** Separating concerns allows: (a) fast feedback on tests without waiting for wheel builds, (b) wheel builds to verify packaging without triggering releases, (c) release job to require explicit version tag trigger and use separate `id-token: write` permission scope.

**Alternative (simpler):** Two files -- `ci.yml` for tests+coverage, `release.yml` for build+publish (only on tags). This reduces complexity but means wheels are not built/tested on every PR.

**Recommended approach:** Two files. `ci.yml` handles testing and coverage on every push/PR. `release.yml` handles wheel builds and PyPI publishing on version tags. Wheel builds are slow (~10 min with GMP compilation) so running them on every push is wasteful. The release workflow includes its own build step before publishing.

### ci.yml Structure

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:
permissions:
  contents: read
jobs:
  rust-tests:        # cargo test --workspace (Linux)
  python-tests:      # maturin develop + pytest (Linux)
  coverage:          # cargo-tarpaulin + codecov upload (Linux)
```

### release.yml Structure

```yaml
name: Release
on:
  push:
    tags: ['v*']
permissions:
  contents: read
jobs:
  linux-wheels:      # manylinux2014 x86_64 wheel
  windows-wheels:    # win_amd64 GNU wheel
  sdist:             # source distribution
  publish:           # download all artifacts, upload to PyPI via OIDC
    needs: [linux-wheels, windows-wheels, sdist]
    permissions:
      id-token: write
    environment: pypi
```

### Key Layout Consideration: Subdirectory Project

The pyproject.toml lives at `crates/qsym-python/pyproject.toml`, not the repo root. This requires:

1. **maturin-action**: Use `working-directory: crates/qsym-python` input
2. **maturin build args**: Use `--manifest-path ../../Cargo.toml` if maturin cannot auto-detect the workspace root (but typically maturin follows the Cargo.toml in the working directory, which already references the workspace)
3. **sdist**: Must include the workspace Cargo.lock from the repo root. The pyproject.toml's `[tool.maturin]` does not set `manifest-path`, so maturin will look for Cargo.toml in the same directory as pyproject.toml (`crates/qsym-python/Cargo.toml`). The workspace root Cargo.lock is two levels up. Maturin handles this: it detects the workspace and includes the workspace Cargo.lock in sdist automatically since PR #1362.
4. **pytest**: Run from `crates/qsym-python/` with `pytest tests/ -v`

### Anti-Patterns to Avoid

- **Running wheel builds on every push/PR**: GMP compilation in manylinux container is slow (~5-10 min). Reserve for tags or keep as a separate optional workflow.
- **Using MSVC toolchain for Windows builds**: The project depends on `gmp-mpfr-sys` with `use-system-libs`, which does not support MSVC. Must use `x86_64-pc-windows-gnu`.
- **Scoping `id-token: write` globally**: Only the PyPI publish job needs this permission. Set it at job level, not workflow level.
- **Building from source (no `use-system-libs`)**: GMP compilation from source in CI takes much longer and is fragile. Use pre-built system packages.
- **Putting secrets in workflow files**: OIDC trusted publishing eliminates API tokens entirely.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Manylinux wheel compliance | Custom Docker + auditwheel | `maturin-action` with `manylinux: 2014` | Handles container selection, auditwheel, platform tags automatically |
| PyPI authentication | API token management | OIDC trusted publishing | No secrets to rotate, short-lived tokens, industry best practice |
| Windows MinGW setup | Manual PATH/env manipulation | `msys2/setup-msys2@v2` with `install: mingw-w64-x86_64-gmp mingw-w64-x86_64-gcc` | Handles MSYS2 installation, caching, package management |
| Coverage report aggregation | Custom scripts | Codecov service + `codecov-action@v5` | Badge generation, PR comments, trend tracking |
| Artifact passing between jobs | Manual upload/download | `actions/upload-artifact@v4` + `actions/download-artifact@v4` | Standard pattern, handles cleanup, merging |
| Rust toolchain installation | Manual rustup | `dtolnay/rust-toolchain@stable` | Handles caching, components, targets |

## Common Pitfalls

### Pitfall 1: GMP Not Available in Manylinux Container

**What goes wrong:** Maturin builds inside a manylinux Docker container that does not have GMP development headers installed. Build fails with `gmp.h: No such file or directory`.
**Why it happens:** The manylinux2014 container image is CentOS 7-based and ships minimal packages.
**How to avoid:** Use `before-script-linux` input on maturin-action:
```yaml
before-script-linux: |
  yum install -y gmp-devel
```
**Warning signs:** Build error mentioning `gmp.h`, `libgmp`, or `gmp-mpfr-sys` build failure.

### Pitfall 2: Windows Build Uses Wrong Rust Toolchain

**What goes wrong:** Windows runner defaults to `x86_64-pc-windows-msvc` toolchain. GMP does not compile with MSVC, causing build failure.
**Why it happens:** GitHub Actions Windows runners have MSVC Rust toolchain by default.
**How to avoid:** Explicitly install and select the GNU toolchain:
```yaml
- uses: dtolnay/rust-toolchain@stable
  with:
    targets: x86_64-pc-windows-gnu
```
And set `CARGO_BUILD_TARGET=x86_64-pc-windows-gnu` or pass `--target x86_64-pc-windows-gnu` to maturin.
**Warning signs:** Link errors mentioning MSVC, `link.exe`, or `gmp-mpfr-sys` MSVC rejection.

### Pitfall 3: Windows Wheel Platform Tag Compatibility

**What goes wrong:** Concern that `x86_64-pc-windows-gnu` compiled wheels won't install on standard Windows Python (which is MSVC-built).
**Why it actually works:** Maturin produces `win_amd64` platform-tagged wheels regardless of whether the Rust toolchain is GNU or MSVC. The ABI3 stable ABI ensures compatibility. The MinGW C runtime (`msvcrt.dll` / `ucrt`) is present on all Windows 10+ systems. GNU-compiled PyO3 extensions load correctly into MSVC Python because they link to the Python stable ABI DLL (`python3.dll`), not version-specific import libraries.
**Remaining risk:** GMP DLL (`libgmp-10.dll`) must be bundled in the wheel. The existing `include = [{ path = "python/q_kangaroo/*.dll", format = "wheel" }]` in pyproject.toml handles this, but CI must copy the DLL before building.

### Pitfall 4: Missing GMP DLL in Windows Wheel

**What goes wrong:** Wheel builds but `import q_kangaroo` fails at install time because `libgmp-10.dll` is not found.
**Why it happens:** The `gmp-mpfr-sys` crate links to GMP dynamically with `use-system-libs`. The DLL is on the CI PATH but not bundled into the wheel.
**How to avoid:** After MSYS2 setup, copy the DLL into the Python package directory before maturin build:
```bash
cp /mingw64/bin/libgmp-10.dll crates/qsym-python/python/q_kangaroo/
```
The `include = [{ path = "python/q_kangaroo/*.dll", format = "wheel" }]` in pyproject.toml then bundles it.
**Warning signs:** ImportError or OSError about missing DLL at `pip install` time.

### Pitfall 5: PYO3_USE_ABI3_FORWARD_COMPATIBILITY Not Set

**What goes wrong:** Build fails or produces version-specific wheel instead of ABI3 wheel when building against Python 3.14 (which is newer than PyO3's current minimum).
**Why it happens:** PyO3 0.23 may not yet support the Python version on the CI runner without this env var.
**How to avoid:** Set `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` in the build environment.
**Warning signs:** PyO3 build error about unsupported Python version, or wheel filename containing `cp314` instead of `abi3`.

### Pitfall 6: OIDC Publishing Not Configured on PyPI

**What goes wrong:** Release workflow runs but `pypa/gh-action-pypi-publish` fails with authentication error.
**Why it happens:** PyPI trusted publisher must be configured on the PyPI project settings page BEFORE the first OIDC publish attempt.
**How to avoid:** Before first release, go to https://pypi.org/manage/project/q-kangaroo/settings/publishing/ and add a trusted publisher specifying: repository owner/name, workflow filename (`release.yml`), and environment name (`pypi`).
**Warning signs:** 403 or authentication failure in the publish step.

### Pitfall 7: Sdist Cannot Build Due to Missing Workspace Context

**What goes wrong:** `maturin sdist` from `crates/qsym-python/` fails because it cannot find the workspace Cargo.lock or the `qsym-core` dependency.
**Why it happens:** Sdist must include the full workspace context. Maturin's sdist includes workspace members and Cargo.lock automatically, but checkout must be from the repo root.
**How to avoid:** Always checkout the full repository. Use `working-directory` in maturin-action rather than checking out a subdirectory. Maturin handles workspace detection from the member crate's Cargo.toml.

### Pitfall 8: cargo-tarpaulin Only Works on Linux

**What goes wrong:** Attempting to run coverage on Windows fails.
**Why it happens:** cargo-tarpaulin uses ptrace-based instrumentation that only works on Linux.
**How to avoid:** Run coverage exclusively in the Linux job. This is sufficient -- code coverage does not need to be measured on every platform.

### Pitfall 9: upload-artifact v4 Name Collisions

**What goes wrong:** Multiple matrix jobs try to upload to the same artifact name, causing failures.
**Why it happens:** `actions/upload-artifact@v4` requires unique artifact names (unlike v3).
**How to avoid:** Use distinct artifact names per platform (e.g., `wheels-linux`, `wheels-windows`, `wheels-sdist`) and then use `download-artifact@v4` with `merge-multiple: true` or `pattern: wheels-*` to combine them.

## Code Examples

### Example 1: ci.yml -- Rust Tests + Python Tests + Coverage

```yaml
# Source: Synthesized from official docs for maturin-action, cargo-tarpaulin, codecov-action
name: CI

on:
  push:
    branches: [main]
  pull_request:

permissions:
  contents: read

env:
  CARGO_TERM_COLOR: always

jobs:
  rust-tests:
    name: Rust Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      - name: Install GMP
        run: sudo apt-get install -y libgmp-dev
      - name: Run tests
        run: cargo test --workspace

  python-tests:
    name: Python Integration Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - uses: dtolnay/rust-toolchain@stable
      - name: Install GMP
        run: sudo apt-get install -y libgmp-dev
      - name: Install maturin and build
        run: |
          pip install maturin pytest
          cd crates/qsym-python
          PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --release
      - name: Run Python tests
        run: |
          cd crates/qsym-python
          pytest tests/ -v

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install GMP
        run: sudo apt-get install -y libgmp-dev
      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --workspace --out xml --output-dir coverage/
      - name: Upload to Codecov
        uses: codecov/codecov-action@v5
        with:
          files: coverage/cobertura.xml
          fail_ci_if_error: false
          token: ${{ secrets.CODECOV_TOKEN }}
```

### Example 2: release.yml -- Build Wheels + Publish to PyPI

```yaml
# Source: Synthesized from maturin-action docs, PyPI trusted publishing docs
name: Release

on:
  push:
    tags: ['v*']

permissions:
  contents: read

jobs:
  linux-wheels:
    name: Build Linux Wheels
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: PyO3/maturin-action@v1
        with:
          command: build
          args: --release --locked
          working-directory: crates/qsym-python
          manylinux: 2014
          before-script-linux: |
            yum install -y gmp-devel
        env:
          PYO3_USE_ABI3_FORWARD_COMPATIBILITY: '1'
      - uses: actions/upload-artifact@v4
        with:
          name: wheels-linux
          path: crates/qsym-python/target/wheels/*.whl

  windows-wheels:
    name: Build Windows Wheels
    runs-on: windows-latest
    defaults:
      run:
        shell: msys2 {0}
    steps:
      - run: git config --global core.autocrlf input
        shell: bash
      - uses: actions/checkout@v4
      - uses: msys2/setup-msys2@v2
        with:
          msystem: MINGW64
          update: true
          install: >-
            mingw-w64-x86_64-gcc
            mingw-w64-x86_64-gmp
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-gnu
      - uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - name: Copy GMP DLL for bundling
        run: cp /mingw64/bin/libgmp-10.dll crates/qsym-python/python/q_kangaroo/
      - name: Build wheel
        run: |
          pip install maturin
          cd crates/qsym-python
          maturin build --release --target x86_64-pc-windows-gnu
        env:
          PYO3_USE_ABI3_FORWARD_COMPATIBILITY: '1'
          LIBRARY_PATH: /mingw64/lib
          C_INCLUDE_PATH: /mingw64/include
      - uses: actions/upload-artifact@v4
        with:
          name: wheels-windows
          path: crates/qsym-python/target/wheels/*.whl

  sdist:
    name: Build Source Distribution
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: PyO3/maturin-action@v1
        with:
          command: sdist
          working-directory: crates/qsym-python
      - uses: actions/upload-artifact@v4
        with:
          name: wheels-sdist
          path: crates/qsym-python/target/wheels/*.tar.gz

  publish:
    name: Publish to PyPI
    needs: [linux-wheels, windows-wheels, sdist]
    runs-on: ubuntu-latest
    environment:
      name: pypi
      url: https://pypi.org/p/q-kangaroo
    permissions:
      id-token: write
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: wheels-*
          merge-multiple: true
          path: dist/
      - uses: pypa/gh-action-pypi-publish@release/v1
```

### Example 3: Coverage Badge in README

```markdown
[![codecov](https://codecov.io/gh/OWNER/q-kangaroo/graph/badge.svg)](https://codecov.io/gh/OWNER/q-kangaroo)
```

### Example 4: Windows maturin-action Alternative (without MSYS2 shell default)

If using maturin-action on Windows rather than raw maturin:

```yaml
  windows-wheels:
    name: Build Windows Wheels
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: msys2/setup-msys2@v2
        with:
          msystem: MINGW64
          update: true
          install: >-
            mingw-w64-x86_64-gcc
            mingw-w64-x86_64-gmp
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-gnu
      - name: Copy GMP DLL
        shell: msys2 {0}
        run: cp /mingw64/bin/libgmp-10.dll crates/qsym-python/python/q_kangaroo/
      - name: Add MSYS2 to PATH
        shell: bash
        run: echo "D:/a/_temp/msys64/mingw64/bin" >> $GITHUB_PATH
      - uses: PyO3/maturin-action@v1
        with:
          command: build
          args: --release --target x86_64-pc-windows-gnu
          working-directory: crates/qsym-python
        env:
          PYO3_USE_ABI3_FORWARD_COMPATIBILITY: '1'
      - uses: actions/upload-artifact@v4
        with:
          name: wheels-windows
          path: crates/qsym-python/target/wheels/*.whl
```

Note: The MSYS2 bin path on GitHub Actions runners is typically `D:/a/_temp/msys64/mingw64/bin`, but this may vary. Use `${{ steps.msys2.outputs.msys2-location }}/mingw64/bin` for reliability.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| PyPI API tokens in secrets | OIDC Trusted Publishing | 2023 | No stored secrets, short-lived tokens, better security |
| actions/upload-artifact@v3 (multi-upload to same name) | @v4 (unique names + merge) | 2024 | Must use distinct artifact names per job |
| actions-rs/toolchain (archived) | dtolnay/rust-toolchain | 2023 | actions-rs is unmaintained; dtolnay's is the standard |
| Manual maturin install | PyO3/maturin-action@v1 | Ongoing | Handles containers, sccache, cross-compilation |
| codecov-action@v4 | @v5 | Late 2024 | Token optional for public repos, new Wrapper backend |
| grcov (nightly required) | cargo-tarpaulin (stable) | Ongoing | Tarpaulin works on stable Rust, simpler setup |

**Deprecated/outdated:**
- `actions-rs/*` actions: Archived, unmaintained. Use `dtolnay/rust-toolchain` instead.
- `actions/upload-artifact@v3`: Deprecated. v4 requires unique artifact names.
- PyPI API tokens for CI: Still work but OIDC is the recommended path.

## Open Questions

1. **Windows wheel target path in maturin-action**
   - What we know: `maturin-action` with `working-directory` sets the CWD for maturin. Wheel output goes to `target/wheels/` relative to the workspace root or the crate directory.
   - What's unclear: Whether `target/wheels/` ends up at `crates/qsym-python/target/wheels/` or the workspace root `target/wheels/`. This affects the `upload-artifact` path.
   - Recommendation: Test both paths in initial CI setup. The upload step can use a glob pattern like `**/target/wheels/*.whl` as a fallback.

2. **MSYS2 PATH integration with maturin-action on Windows**
   - What we know: maturin-action runs maturin in its own step. MSYS2 MinGW libraries need to be on the linker search path.
   - What's unclear: Whether maturin-action inherits the MSYS2 PATH additions or needs explicit `LIBRARY_PATH` / `C_INCLUDE_PATH` env vars.
   - Recommendation: Start with adding MSYS2 mingw64/bin to `GITHUB_PATH` and setting `LIBRARY_PATH`/`C_INCLUDE_PATH`. Fall back to running maturin directly in the MSYS2 shell if maturin-action does not pick up the paths.

3. **GMP DLL additional dependencies**
   - What we know: `libgmp-10.dll` is the primary GMP library. On the local dev machine, it is the only required MinGW DLL.
   - What's unclear: Whether `libgmp-10.dll` has transitive DLL dependencies (like `libgcc_s_seh-1.dll`, `libwinpthread-1.dll`) that also need bundling.
   - Recommendation: After building the Windows wheel in CI, run `ldd` or `objdump -x` on the `.pyd` file to identify all DLL dependencies. Bundle any MinGW runtime DLLs not present on stock Windows.

4. **Codecov token for public vs private repository**
   - What we know: Codecov v5 allows tokenless upload for public repos with the right org setting. Private repos require a token.
   - What's unclear: Whether the q-kangaroo repo is public or private.
   - Recommendation: Always configure `CODECOV_TOKEN` as a repository secret for reliability. It works for both public and private repos.

## Sources

### Primary (HIGH confidence)
- [PyO3/maturin-action](https://github.com/PyO3/maturin-action) -- Action inputs, manylinux configuration, before-script-linux, working-directory
- [pypa/gh-action-pypi-publish](https://github.com/pypa/gh-action-pypi-publish) -- OIDC trusted publishing workflow, permissions, environment config
- [PyPI Trusted Publishers docs](https://docs.pypi.org/trusted-publishers/using-a-publisher/) -- Setup instructions for OIDC, required permissions
- [msys2/setup-msys2](https://github.com/msys2/setup-msys2) -- Windows MinGW package installation, MINGW64 environment setup
- [codecov/codecov-action](https://github.com/codecov/codecov-action) -- v5 usage, token requirements, badge setup
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) -- Rust coverage tool, Linux-only limitation
- [Maturin User Guide: Distribution](https://www.maturin.rs/distribution) -- Manylinux, sdist, compatibility flags
- [Maturin User Guide: Configuration](https://www.maturin.rs/config.html) -- pyproject.toml [tool.maturin] options

### Secondary (MEDIUM confidence)
- [Maturin CI source (ci.rs)](https://github.com/PyO3/maturin/blob/main/src/ci.rs) -- Generated workflow structure reference
- [PyO3 Building and Distribution Guide](https://pyo3.rs/v0.23.5/building-and-distribution.html) -- ABI3, Windows GNU target compatibility
- [gmp-mpfr-sys docs](https://docs.rs/gmp-mpfr-sys/latest/gmp_mpfr_sys/) -- use-system-libs feature, Windows build requirements
- [GitHub Actions upload-artifact v4 docs](https://github.com/actions/upload-artifact) -- Unique names requirement, merge feature

### Tertiary (LOW confidence)
- [Rust code coverage blog post](https://eipi.xyz/blog/rust-code-coverage-with-github-workflows/) -- Tarpaulin + Codecov workflow patterns (blog, 2024)
- [Python Discuss: Wheel platform tags for Windows](https://discuss.python.org/t/wheel-platform-tag-for-windows/9025) -- win_amd64 tag applies to both GNU and MSVC builds

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- All tools are official/well-documented GitHub Actions with active maintenance
- Architecture (workflow structure): HIGH -- Standard patterns from maturin generate-ci and PyPI docs
- GMP in manylinux container: HIGH -- `yum install gmp-devel` in CentOS 7 (manylinux2014) is well-tested
- Windows MinGW wheel build: MEDIUM -- MSYS2 + GNU Rust + maturin is viable but path integration needs validation
- GMP DLL bundling in Windows wheel: MEDIUM -- DLL copy approach is correct but transitive dependencies need CI-time verification
- OIDC trusted publishing: HIGH -- Well-documented by PyPI, straightforward setup
- Coverage + badge: HIGH -- cargo-tarpaulin + Codecov is a proven combination

**Research date:** 2026-02-15
**Valid until:** 2026-04-15 (90 days -- CI tooling is stable; pin action versions)
