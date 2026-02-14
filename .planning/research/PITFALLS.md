# Pitfalls Research: PyO3/Maturin Packaging, CI, Documentation, and Renaming

**Domain:** Rust+Python library with native dependencies (GMP) using PyO3 0.23 + maturin
**Researched:** 2026-02-14
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Module Name Triple-Mismatch (Rename Breakage)

**What goes wrong:**
When renaming a PyO3/maturin package, three separate identifiers must align: `[lib] name` in Cargo.toml, `module-name` in pyproject.toml, and the `#[pymodule]` decorator name. Misalignment causes "ImportError: dynamic module does not define module export function" with no obvious indication which piece is wrong.

**Why it happens:**
Maturin merges metadata from Cargo.toml and pyproject.toml (pyproject.toml takes precedence), but the Rust macro `#[pymodule]` is evaluated separately during compilation. Developers update 2 of 3 locations and assume it's complete.

**How to avoid:**
For rename from `qsymbolic` to `q_kangaroo`:

1. **Cargo.toml**: `[lib] name = "_q_kangaroo"` (underscore prefix for internal module)
2. **pyproject.toml**: `[tool.maturin] module-name = "q_kangaroo._q_kangaroo"` (package.internal_module)
3. **lib.rs**: `#[pymodule] fn _q_kangaroo(m: &Bound<'_, PyModule>) -> PyResult<()>`
4. **__init__.py**: `from q_kangaroo._q_kangaroo import ...`

All four must use `_q_kangaroo` for the Rust module. The last component of `module-name` MUST match `#[pymodule]` name exactly.

**Warning signs:**
- ImportError mentioning `PyInit_` with wrong module name
- Module imports in interactive Python fail but `maturin develop` succeeds
- IDE shows module as available but runtime import fails

**Phase to address:**
Phase 1: Rename (first step before any other work to avoid cascading changes)

---

### Pitfall 2: ABI3 Feature Flag Without Minimum Version

**What goes wrong:**
Using bare `abi3` feature in PyO3 dependencies creates version-specific wheels (e.g., `cp39-cp39`) instead of cross-version compatible `cp36-abi3` wheels. Users with different Python versions cannot install the wheel despite ABI3 being intended for forward compatibility.

**Why it happens:**
PyO3 allows setting `abi3` without any minimum version and defaults to the current Python interpreter version. Maturin requires an explicit minimum version flag (e.g., `abi3-py09`) to properly name the wheel with the correct platform tag.

**How to avoid:**
In Cargo.toml dependencies:
```toml
pyo3 = { version = "0.23", features = ["extension-module", "abi3-py09"] }
```

NOT:
```toml
pyo3 = { version = "0.23", features = ["extension-module", "abi3"] }
```

Use `abi3-py09` (or whatever your minimum supported version is) to generate `cp09-abi3` wheels that work on Python 3.9+. This project uses Python 3.14 but should target `abi3-py09` for backward compatibility.

**Warning signs:**
- Wheel filename contains `cp314-cp314` instead of `cp09-abi3`
- Users on Python 3.10/3.11/3.12 report "incompatible wheel" errors
- `maturin build` output shows platform-specific tag when expecting `abi3`

**Phase to address:**
Phase 2: Packaging (configure before first test wheel build)

---

### Pitfall 3: GMP Bundling Failures on Windows

**What goes wrong:**
On Windows, maturin cannot automatically bundle GMP DLLs into wheels like it does on Linux (via patchelf). Users installing from wheels get runtime errors: "DLL load failed while importing _q_kangaroo" because GMP is not found in system PATH.

**Why it happens:**
Maturin's wheel repair process uses patchelf on Linux to bundle shared libraries and adjust rpath. Windows has no equivalent automatic bundling—maturin expects system libraries to be in PATH or requires manual DLL packaging. The current project setup uses MinGW GMP at `C:/mingw64-gcc/mingw64/bin`, which end users won't have.

**How to avoid:**
**Short-term (development/local users):**
- Document GMP requirement in README with installation instructions
- Add runtime check in `__init__.py` with helpful error message
- Current `os.add_dll_directory()` approach works but requires MINGW_BIN env var

**Long-term (distribution):**
- Include GMP DLLs directly in wheel (manual packaging via `include` in pyproject.toml)
- Build static linkage version (requires rebuilding GMP with static flags—complex)
- Use cibuildwheel with pre-built GMP and explicit DLL inclusion
- Document Windows-specific installation: "Requires GMP runtime or use conda-forge"

**Warning signs:**
- Wheel builds successfully but `import q_kangaroo` fails immediately on fresh Windows install
- Dependency Walker shows missing `libgmp-10.dll` or similar
- Works in development environment but not in clean virtualenv

**Phase to address:**
Phase 2: Packaging (must decide approach before publishing to PyPI)

---

### Pitfall 4: Auditwheel Repair with Read-Only Libraries

**What goes wrong:**
When building Linux wheels in Docker/CI, if system libraries (like GMP) have read-only permissions (0o555), maturin's wheel repair process fails with permission errors during `patchelf --set-soname` operations. The repaired wheel is incomplete or build fails entirely.

**Why it happens:**
Maturin copies libraries into the wheel and attempts to patch them (adjust SONAME, rpath). If the copied file inherits read-only permissions, patchelf cannot modify it. This commonly occurs in Docker images where system packages install read-only libraries.

**How to avoid:**
In GitHub Actions / CI configuration:
```yaml
- name: Build wheels
  run: |
    # Ensure copied libraries are writable before maturin processes them
    chmod -R u+w target/ || true
    maturin build --release --manylinux 2014
```

Or use maturin 0.13+ which includes fixes for read-only library handling (PR #1292).

**Warning signs:**
- Build succeeds locally but fails in CI with patchelf errors
- Error messages mentioning "Permission denied" during wheel repair
- `auditwheel repair` or maturin's internal repair step fails

**Phase to address:**
Phase 3: CI Setup (test Linux wheel builds early)

---

### Pitfall 5: Python 3.13+ with PyO3 Maximum Version Error

**What goes wrong:**
Building with Python 3.13+ triggers error: "the configured Python interpreter version (3.13) is newer than PyO3's maximum supported version (3.12)". Build halts unless `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` is set, but developers don't know this flag exists.

**Why it happens:**
PyO3 explicitly validates Python version against its tested maximum. Newer Python versions may have ABI changes. The stable ABI (abi3) provides forward compatibility, but PyO3 requires explicit opt-in via environment variable to suppress version checking.

**How to avoid:**
For projects using `abi3-py3X`:

In CI workflows and local development:
```bash
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
```

In GitHub Actions:
```yaml
env:
  PYO3_USE_ABI3_FORWARD_COMPATIBILITY: 1
```

This tells PyO3: "I'm using stable ABI, allow building with newer Python than tested maximum."

**Warning signs:**
- Builds fail on CI runners with Python 3.13+
- Local development works on Python 3.10 but CI fails with newer Python
- Error message explicitly mentions maximum supported version

**Phase to address:**
Phase 3: CI Setup (configure environment variables before first CI run)

---

### Pitfall 6: Editable Install Missing .so After Upgrade

**What goes wrong:**
After upgrading maturin (e.g., 1.8.2 → 1.11.0+), `maturin develop` appears to succeed but the compiled `.so` file is not placed in the Python package directory. Imports fail with "ModuleNotFoundError" despite successful build output.

**Why it happens:**
Maturin changed how editable installs handle mixed Rust/Python projects. With `python-source = "python"` in pyproject.toml, the .so location logic changed. Some versions had regressions (Issue #2909).

**How to avoid:**
**Immediate fix:**
```bash
# Force rebuild and install
maturin develop --release
# Verify .so exists
ls -la python/q_kangaroo/_q_kangaroo*.so
```

**Configuration verification:**
In pyproject.toml:
```toml
[tool.maturin]
python-source = "python"
module-name = "q_kangaroo._q_kangaroo"
```

Ensure directory structure:
```
crates/qsym-python/
  python/
    q_kangaroo/
      __init__.py         # imports from ._q_kangaroo
      _q_kangaroo.*.so    # placed here by maturin develop
```

**For pytest during development:**
Use maturin-import-hook to auto-rebuild:
```python
# conftest.py or top of test file
import maturin_import_hook
maturin_import_hook.install()
```

**Warning signs:**
- `maturin develop` succeeds but Python can't import module
- .so file missing from expected location
- Tests worked before maturin upgrade, fail after

**Phase to address:**
Phase 4: Testing Setup (verify before writing CI test jobs)

---

### Pitfall 7: ABI3 + generate-import-lib Configuration Conflict

**What goes wrong:**
When both `abi3` and `generate-import-lib` features are enabled for cross-compilation to Windows, maturin doesn't generate `PYO3_CONFIG_FILE` for pyo3-build-config. PyO3 then uses whatever Python interpreter it finds in PATH (possibly wrong version/platform), causing build failures or incompatible binaries.

**Why it happens:**
Maturin's configuration generation has an interaction bug between these two features (Issue #2385). `generate-import-lib` is designed to enable Windows cross-compilation without needing Python libraries, but when combined with `abi3`, the config file generation path is skipped.

**How to avoid:**
**For cross-compilation to Windows:**
- Use `abi3-py3X` feature explicitly (not bare `abi3`)
- Manually set `PYO3_CROSS_LIB_DIR` and `PYO3_CROSS_PYTHON_VERSION` when using both features
- Consider using pre-built Windows runners instead of cross-compilation

**For this project (Cygwin/MinGW native build):**
- Not cross-compiling, so `generate-import-lib` not needed
- Use `abi3-py09` only, let maturin detect local Python

**Warning signs:**
- Cross-compilation to Windows produces wheels that import-fail
- Build uses wrong Python version despite configuration
- pyo3-build-config warnings about missing config file

**Phase to address:**
Phase 3: CI Setup (if adding Windows cross-compilation jobs)

---

### Pitfall 8: GitHub Actions Cache Bloat with cargo target/

**What goes wrong:**
Using `actions/cache` to cache `target/` directory in GitHub Actions leads to multi-gigabyte cache entries that take 5+ minutes to restore/save. Cache eviction causes frequent cache misses, and the cache contains artifacts from prior builds that aren't useful for current build, slowing CI dramatically.

**Why it happens:**
The `target/` directory hoards artifacts from all previous builds and grows unbounded. GitHub Actions caches are limited (10GB total across repository), so large caches get evicted frequently. Cargo must wait for entire cache blob to download before starting build, even if most artifacts are irrelevant.

**How to avoid:**
**Use sccache instead of target/ caching:**

```yaml
- uses: PyO3/maturin-action@v1
  with:
    maturin-version: latest
    command: build
    args: --release --manylinux 2014
    sccache: 'true'  # Enable sccache

env:
  RUSTC_WRAPPER: sccache
  SCCACHE_GHA_ENABLED: 'true'
  SCCACHE_CACHE_SIZE: '2G'
```

**Benefits:**
- 50% faster than cargo target caching in tests
- Concurrent fetch (build starts immediately)
- Smaller cache entries (only compiled objects, not all artifacts)
- Built-in eviction strategy

**Warning signs:**
- CI jobs spend 3-5+ minutes on "Restore cache" step
- Cache size grows beyond 2-3GB
- Frequent "cache not found" messages despite recent builds

**Phase to address:**
Phase 3: CI Setup (configure before running many builds)

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skip ABI3, use version-specific builds | Simpler Cargo.toml, one less feature flag | Users need separate wheels per Python version, 5x wheel count | Never for library distribution |
| Document GMP as external dependency (not bundled) | Avoids complex wheel bundling | Windows users must manually install GMP or use conda | Acceptable for academic/research tool with tech-savvy users |
| Use bare `abi3` feature without version | Shorter feature list | Wheels named incorrectly, compatibility broken | Never—always specify minimum version |
| Manually copy .so during development (no maturin develop) | Avoids maturin version issues | Stale .so files cause debugging nightmares, no auto-rebuild | Never—use maturin-import-hook if `develop` breaks |
| Skip wheel repair on Linux | Faster builds, no patchelf needed | Wheels fail on systems without GMP in standard locations | Only for internal use, never PyPI distribution |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| pytest with maturin | Running `pytest` directly without rebuild after Rust changes | Use maturin-import-hook or explicit `maturin develop && pytest` |
| GitHub Actions maturin-action | Not setting `sccache: true`, using actions/cache on target/ | Use maturin-action's built-in sccache parameter |
| PyPI upload | Uploading wheel without testing in clean virtualenv | Test wheel in Docker/fresh env before upload, verify GMP loading |
| Documentation generation | Expecting Sphinx autodoc to work with PyO3 functions | Use pyo3-stub-gen to generate .pyi stubs first, then Sphinx autodoc |
| Cross-platform testing | Only testing on development platform (Windows MinGW) | Test wheels on clean Linux (Docker), macOS (if supporting), Windows MSVC |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Caching entire target/ in CI | 5+ min cache restore/save, frequent cache misses | Use sccache instead of target/ caching | After ~10 builds (cache eviction) |
| No `--release` flag during wheel builds | Wheels work but are 10-100x slower for compute-heavy code | Always use `--release` for distributed wheels | User benchmarks show poor performance |
| Re-importing module in long-running Python process | Each import re-initializes GMP arena, leaking memory | Use single QSession instance, pass session to functions | Programs running >1hr with many imports |
| Rebuilding from scratch in CI without caching | 10+ min Rust builds on every commit | Configure sccache with GitHub Actions cache backend | Every CI run without caching |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Bundling MinGW GMP DLL without verifying source | Supply chain attack if DLL replaced with malicious version | Use checksummed GMP from official MSYS2 repo, verify signatures |
| Accepting arbitrary code in `prove_eta_id` symbolic expressions | Limited (Rust memory-safe) but potential DoS via exponential expansion | Document input limits, add expansion depth checks if exposing via web API |
| Not validating Python version in __init__.py | Crashes or wrong behavior on unsupported Python versions | Check `sys.version_info >= (3, 9)` in __init__.py, raise clear error |
| Exposing internal Rust panic messages to users | Information disclosure about internals | Wrap PyO3 functions with Result, convert panics to Python exceptions with sanitized messages |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Generic "ImportError: DLL load failed" on Windows | User has no idea GMP is missing, tries reinstalling Python | Catch ImportError in __init__.py, raise with "q_kangaroo requires GMP: install via conda-forge or set MINGW_BIN" |
| No version() function or __version__ | Users can't report which version they're using in bug reports | Expose version in both Rust (`version()`) and Python (`__version__`) |
| Docstrings only in Rust, not visible in Python help() | Users run `help(aqprod)` and see cryptic PyO3 signature | Use pyo3-stub-gen or write docstrings in #[pyfunction] doc comments |
| Error messages using Rust terminology (ExprRef, Arena, etc.) | Python users don't understand Rust internals | Convert Rust errors to Python exceptions with domain terminology |
| No install test in README | Users install, can't verify it worked | Add "Quick Test" section: `python -c "import q_kangaroo; print(q_kangaroo.version())"` |

## "Looks Done But Isn't" Checklist

- **Wheel builds successfully:** Often missing verification that it installs in clean environment—test in Docker/fresh virtualenv
- **Tests pass locally:** Often missing CI verification—local .so may be stale, tests passing against old code
- **Rename completed in code:** Often missing updates to pyproject.toml metadata (author, description, URLs still reference old name)
- **Documentation generated:** Often missing .pyi stub files for IDE autocomplete—Sphinx docs exist but no type hints
- **GMP dependency handled:** Often missing runtime check—works on dev machine (GMP in PATH) but fails for users
- **CI green:** Often missing release build test—debug builds pass, release builds fail due to optimization-exposed bugs
- **Version bumped:** Often missing git tag—version number updated but no corresponding tag for release tracking

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Module name mismatch after rename | LOW | 1. Search codebase for old module name 2. Update Cargo.toml, pyproject.toml, lib.rs, __init__.py 3. `maturin develop` 4. Test import |
| Wrong ABI3 feature (bare `abi3`) | LOW | 1. Update Cargo.toml to `abi3-py09` 2. Clean build 3. Verify wheel filename contains `abi3` |
| GMP DLL missing in distributed wheel | MEDIUM | 1. Add manual DLL bundling via pyproject.toml `include` 2. Rebuild wheels 3. Test on clean Windows 4. Republish |
| Auditwheel repair failure in CI | LOW | 1. Add `chmod -R u+w` before maturin build 2. Or upgrade maturin to 0.13+ 3. Re-run CI |
| Editable install not working | LOW | 1. `pip uninstall q_kangaroo` 2. `maturin develop --release` 3. Verify .so location 4. Or use maturin-import-hook |
| GitHub Actions cache too large | MEDIUM | 1. Remove actions/cache steps for target/ 2. Add `sccache: true` to maturin-action 3. Set SCCACHE env vars 4. Initial run slower (no cache), subsequent 50% faster |
| Published wheel incompatible with Python 3.13 | MEDIUM | 1. Set PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 2. Rebuild wheels 3. Test on Python 3.13 4. Republish with updated version |
| Sphinx docs missing PyO3 function signatures | MEDIUM | 1. Install pyo3-stub-gen 2. Generate .pyi files 3. Configure Sphinx to read stubs 4. Rebuild docs |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Module name mismatch | Phase 1: Rename | `python -c "import q_kangaroo; print(q_kangaroo.version())"` succeeds |
| Wrong ABI3 feature | Phase 2: Packaging | Wheel filename contains `cp09-abi3`, not `cp314-cp314` |
| GMP bundling on Windows | Phase 2: Packaging | Test wheel install in clean virtualenv without MINGW_BIN set |
| Auditwheel read-only libraries | Phase 3: CI Setup | Linux wheel build in CI completes without permission errors |
| Python 3.13+ version error | Phase 3: CI Setup | CI job on Python 3.13 succeeds with forward compat flag |
| Editable install .so missing | Phase 4: Testing Setup | `maturin develop && pytest` runs without import errors |
| ABI3 + generate-import-lib conflict | Phase 3: CI Setup (if cross-compiling) | Cross-compiled Windows wheel imports correctly |
| GitHub Actions cache bloat | Phase 3: CI Setup | Cache restore <30s, build time reduced by ~50% with sccache |

## Phase-Specific Deep Dives

### Phase 1: Rename Risks

**Primary risk:** Incomplete rename creates broken imports in some contexts but not others (works locally, fails in CI; works in development install, fails in wheel).

**All locations requiring update:**
1. `Cargo.toml` (2 files: workspace member name, `[lib] name`)
2. `pyproject.toml` ([project] name, module-name)
3. `lib.rs` (#[pymodule] function name)
4. `__init__.py` (import statement, __all__, docstrings)
5. `test_integration.py` (import statements)
6. `README.md` (package name in examples)
7. `.github/workflows/*.yml` (package name in test commands if present)

**Verification strategy:**
```bash
# Grep for old name across entire project
rg "qsymbolic" --type toml --type rust --type python --type markdown

# Test matrix
1. `maturin develop && python -c "import q_kangaroo; print(q_kangaroo.version())"`
2. `maturin build && pip install target/wheels/*.whl && python -c "import q_kangaroo"`
3. `pytest` (after develop install)
```

### Phase 2: Packaging Risks

**Primary risk:** Wheels build successfully but are not installable/usable on target platforms due to:
- Wrong platform tag (no ABI3)
- Missing GMP dependency
- Incorrect manylinux compliance level

**Testing checklist:**
- [ ] Build wheel on dev machine, verify filename pattern: `q_kangaroo-*-cp09-abi3-*.whl`
- [ ] Test wheel in Docker Ubuntu 22.04: `pip install wheel && python -c "import q_kangaroo"`
- [ ] Test wheel in fresh Windows virtualenv without MinGW in PATH
- [ ] Test wheel on Python 3.9, 3.10, 3.11, 3.12, 3.13 (abi3 forward compat)
- [ ] Run `auditwheel show` on Linux wheel, verify manylinux2014 tag
- [ ] Check wheel size—if >50MB, investigate unnecessary bundled files

### Phase 3: CI Setup Risks

**Primary risk:** CI green status gives false confidence—tests pass in CI but wheels produced are broken for end users.

**Critical CI jobs:**
1. **Linux wheel build:** manylinux2014 Docker, sccache enabled, auditwheel verify
2. **Windows wheel build:** Native build with MinGW GMP, test install in separate step
3. **macOS wheel build (if supporting):** Native build, test install
4. **Test matrix:** Install wheel on Python 3.9, 3.10, 3.11, 3.12, 3.13
5. **Import smoke test:** Every platform runs `python -c "import q_kangaroo; q_kangaroo.partition_gf(...)"`

**sccache configuration:**
```yaml
env:
  RUSTC_WRAPPER: sccache
  SCCACHE_GHA_ENABLED: 'true'
  SCCACHE_CACHE_SIZE: '2G'
  PYO3_USE_ABI3_FORWARD_COMPATIBILITY: '1'
```

### Phase 4: Documentation Risks

**Primary risk:** Documentation exists but is unusable—no autocomplete in IDEs, no type hints, docstrings not visible in Python help().

**Documentation components:**
1. **Type stubs (.pyi):** Use pyo3-stub-gen to generate from Rust code
2. **Sphinx docs:** Configure autodoc to read .pyi files
3. **README examples:** Test as part of CI (run examples in doctest or script)
4. **Docstrings in Rust:** Write in `#[pyfunction]` doc comments, extracted by pyo3-stub-gen

**Verification:**
- Open VSCode/PyCharm, type `q_kangaroo.aqprod(` → autocomplete shows parameters
- Run `help(q_kangaroo.aqprod)` → shows full docstring
- Sphinx build produces HTML without warnings
- All README code examples execute successfully

## Sources

### Official Documentation
- [Maturin User Guide - Project Layout](https://www.maturin.rs/project_layout.html) (module naming)
- [Maturin User Guide - Distribution](https://www.maturin.rs/distribution.html) (manylinux, wheel repair)
- [PyO3 Building and Distribution](https://pyo3.rs/v0.28.0/building-and-distribution.html) (ABI3, cross-compilation)
- [PyO3 FAQ and Troubleshooting](https://pyo3.rs/main/faq) (common import errors)

### GitHub Issues and Discussions
- [PyO3/maturin #1960](https://github.com/PyO3/maturin/issues/1960) (Python 3.13 version error)
- [PyO3/maturin #2385](https://github.com/PyO3/maturin/issues/2385) (ABI3 + generate-import-lib bug)
- [PyO3/maturin #400](https://github.com/PyO3/maturin/issues/400) (bare abi3 feature flag issues)
- [PyO3/maturin #2909](https://github.com/PyO3/maturin/issues/2909) (editable install .so missing)
- [PyO3/maturin #742](https://github.com/PyO3/maturin/pull/742) (auditwheel repair implementation)
- [PyO3/maturin #1135](https://github.com/PyO3/maturin/issues/1135) (patchelf pure Rust rewrite)
- [PyO3/maturin #1292](https://github.com/PyO3/maturin/pull/1292) (fix auditwheel with read-only libraries)
- [PyO3/maturin #256](https://github.com/PyO3/maturin/issues/256) (module export function error)
- [PyO3/pyo3 #2330](https://github.com/PyO3/pyo3/discussions/2330) (documentation generation strategies)

### Community Resources
- [Fast Rust Builds with sccache and GitHub Actions](https://depot.dev/blog/sccache-in-github-actions) (sccache vs cargo cache comparison)
- [Optimizing Rust Builds for Faster GitHub Actions Pipelines](https://www.uffizzi.com/blog/optimizing-rust-builds-for-faster-github-actions-pipelines) (CI optimization)
- [Building Portable Native Python Extensions With Rust, PyO3, And Maturin](https://blog.savant-ai.io/building-portable-native-python-extensions-with-rust-pyo3-and-maturin-3c1a1634d324) (manylinux practices)
- [Documenting Native Python Extensions Made With Rust and PyO3](https://blog.savant-ai.io/documenting-native-python-extensions-made-with-rust-and-pyo3-227aff68e481) (documentation workflow)
- [pyo3-stub-gen GitHub](https://github.com/Jij-Inc/pyo3-stub-gen) (stub generation tool)

---

*Pitfalls research for: q-Kangaroo PyO3/maturin packaging and release*
*Researched: 2026-02-14*
*Confidence: HIGH (official docs + verified GitHub issues + community best practices)*
