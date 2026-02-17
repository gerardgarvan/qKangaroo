# Phase 10: PyPI Packaging & Metadata - Research

**Researched:** 2026-02-14
**Domain:** Python packaging (maturin/PyO3), PyPI metadata, type stubs, academic citation
**Confidence:** HIGH

## Summary

This phase prepares the `q-kangaroo` package for PyPI distribution. The project already has a functional maturin-based build producing a `.pyd` native module. The main work items are: (1) completing pyproject.toml metadata, (2) enabling ABI3 stable ABI for cross-version wheel compatibility, (3) bundling MinGW DLLs into the Windows wheel so users do not need MinGW installed, (4) creating `.pyi` type stub files for IDE autocomplete, (5) adding LICENSE and CITATION.cff files, and (6) verifying the wheel installs cleanly on fresh virtualenvs.

The critical technical challenge is DLL bundling. The current build dynamically links against 5 MinGW/GMP DLLs (`libgmp-10.dll`, `libmpfr-6.dll`, `libmpc-3.dll`, `libgcc_s_seh-1.dll`, `libwinpthread-1.dll`). These must be placed alongside the `.pyd` in the wheel, and the existing `os.add_dll_directory()` mechanism in `__init__.py` must be updated to find them from within the installed package rather than from a hardcoded MinGW path.

**Primary recommendation:** Use maturin's `[tool.maturin] include` with `format = "wheel"` to bundle the 5 MinGW DLLs into the wheel alongside the native module, update `__init__.py` DLL loading to use package-relative paths, enable `abi3-py39` in Cargo.toml, and hand-write `.pyi` stubs (not auto-generated) for maximum quality.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| maturin | >=1.0,<2.0 | Build backend for PyO3 wheels | Official PyO3 build tool, handles wheel creation and ABI3 tags |
| PyO3 | 0.23 | Rust-Python bindings | Already in use; add `abi3-py39` feature for stable ABI |
| pip | any | Installation tool | Standard Python package installer |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| twine | latest | PyPI upload validation | `twine check` to verify wheel metadata before upload |
| mypy/pyright | latest | Stub validation | `stubtest` to verify .pyi stubs match runtime module |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-written .pyi stubs | pyo3-stub-gen crate | pyo3-stub-gen is semi-automated but requires proc macro annotations on all 73 functions; hand-written stubs give full control over docstrings and types |
| Dynamic DLL linking + bundling | Static GMP linking | Static `.a` libraries exist but gmp-mpfr-sys's `use-system-libs` feature delegates to pkg-config/system linker which defaults to dynamic; forcing static requires build script changes and is fragile |

**Installation (dev tools):**
```bash
pip install maturin twine mypy
```

## Architecture Patterns

### Recommended Project Structure (After Phase 10)
```
crates/qsym-python/
  Cargo.toml                    # Add abi3-py39 feature
  pyproject.toml                # Complete PEP 621 metadata
  python/
    q_kangaroo/
      __init__.py               # Updated DLL loading (package-relative)
      _q_kangaroo.pyd           # Native module (built by maturin)
      py.typed                  # PEP 561 marker (empty file)
      __init__.pyi              # Type stubs for __init__.py re-exports
      _q_kangaroo.pyi           # Type stubs for native module
      libgmp-10.dll             # Bundled (Windows wheel only)
      libmpfr-6.dll             # Bundled (Windows wheel only)
      libmpc-3.dll              # Bundled (Windows wheel only)
      libgcc_s_seh-1.dll        # Bundled (Windows wheel only)
      libwinpthread-1.dll       # Bundled (Windows wheel only)
  tests/
    test_integration.py
LICENSE                         # MIT license (project root)
CITATION.cff                    # CFF 1.2.0 (project root)
```

### Pattern 1: DLL Bundling via Maturin Include
**What:** Copy MinGW DLLs into the Python package directory and configure maturin to include them in wheels.
**When to use:** Always for Windows wheels that depend on non-system shared libraries.
**Example pyproject.toml:**
```toml
[tool.maturin]
features = ["pyo3/extension-module", "pyo3/abi3-py39"]
python-source = "python"
module-name = "q_kangaroo._q_kangaroo"
include = [
    { path = "python/q_kangaroo/libgmp-10.dll", format = "wheel" },
    { path = "python/q_kangaroo/libmpfr-6.dll", format = "wheel" },
    { path = "python/q_kangaroo/libmpc-3.dll", format = "wheel" },
    { path = "python/q_kangaroo/libgcc_s_seh-1.dll", format = "wheel" },
    { path = "python/q_kangaroo/libwinpthread-1.dll", format = "wheel" },
]
```

**Alternative approach (simpler):** Since this is a mixed Python/Rust project with `python-source = "python"`, maturin automatically includes all non-gitignored files in the Python source directory. If the DLLs are placed in `python/q_kangaroo/`, they will be included automatically without explicit `include` directives. However, they MUST be `.gitignore`d to avoid committing binaries, which means they would NOT be auto-included. Therefore, use the explicit `include` approach, or use a pre-build script to copy them.

**Recommended DLL bundling workflow:**
1. Before `maturin build`, copy the 5 DLLs from `C:/mingw64-gcc/mingw64/bin/` into `python/q_kangaroo/`
2. Add the DLL filenames to `.gitignore` to avoid committing binaries
3. Use `include = [{ path = "python/q_kangaroo/*.dll", format = "wheel" }]` in pyproject.toml
4. Update `__init__.py` to use package-relative DLL directory

### Pattern 2: Package-Relative DLL Loading
**What:** Update `__init__.py` to load DLLs from the installed package directory rather than a hardcoded path.
**When to use:** When DLLs are bundled inside the wheel.
**Example:**
```python
import os
import sys

if sys.platform == "win32":
    # First try bundled DLLs (installed via pip)
    _pkg_dir = os.path.dirname(os.path.abspath(__file__))
    if os.path.isfile(os.path.join(_pkg_dir, "libgmp-10.dll")):
        os.add_dll_directory(_pkg_dir)
    else:
        # Fallback: development environment with MinGW in PATH
        _mingw_dir = os.environ.get("MINGW_BIN", r"C:\mingw64-gcc\mingw64\bin")
        if os.path.isdir(_mingw_dir):
            os.add_dll_directory(_mingw_dir)
```

### Pattern 3: ABI3 Configuration
**What:** Enable Python Stable ABI (PEP 384) so one wheel works across Python 3.9-3.14+.
**When to use:** Always for PyPI distribution of extension modules.
**Cargo.toml changes:**
```toml
[dependencies]
pyo3 = { version = "0.23", features = ["extension-module", "abi3-py39"] }
```
**Environment variables for build:**
```bash
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
```
**Resulting wheel filename pattern:** `q_kangaroo-0.1.0-cp39-abi3-win_amd64.whl`

### Pattern 4: PEP 561 Type Stubs
**What:** `.pyi` stub files + `py.typed` marker for IDE autocomplete.
**When to use:** All published packages with native extensions.
**Key files:**
- `py.typed` - empty marker file (must be in package directory)
- `_q_kangaroo.pyi` - stubs for the native module's classes and functions
- `__init__.pyi` - stubs matching `__init__.py` re-exports

### Anti-Patterns to Avoid
- **Hardcoded DLL paths in shipped packages:** The current `__init__.py` has `r"C:\mingw64-gcc\mingw64\bin"` as default. This MUST NOT be the primary lookup path in installed packages.
- **Forgetting py.typed marker:** Without this file, type checkers ignore `.pyi` stubs even if they exist.
- **Using pyo3-stub-gen for this project:** It requires adding proc macro annotations to all 73+ functions and dealing with complex return types (PyObject for dicts, Option types). Hand-written stubs will be more accurate and faster to create.
- **Committing DLLs to git:** DLLs should be copied at build time, not stored in the repository.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Wheel creation | Custom packaging scripts | `maturin build --release` | Handles ABI3 tags, platform tags, metadata injection |
| ABI3 compatibility | Manual limited API restrictions | PyO3's `abi3-py39` feature | Compiler-enforced API restriction |
| License inclusion in wheel | Manual file copying | `license-files` in pyproject.toml | PEP 639 compliance, maturin auto-bundles |
| Wheel metadata validation | Manual inspection | `twine check dist/*.whl` | Validates all required metadata fields |
| Type stub validation | Manual review | `python -m mypy.stubtest q_kangaroo` | Checks stubs match runtime module |
| CITATION.cff validation | Manual YAML review | `cffconvert --validate` | Validates CFF 1.2.0 schema compliance |

**Key insight:** Maturin handles nearly everything (wheel creation, platform tags, ABI3 naming, metadata embedding, license inclusion). The main manual work is: writing complete pyproject.toml metadata, creating .pyi stubs, and the DLL bundling pre-build step.

## Common Pitfalls

### Pitfall 1: DLL Not Found on Windows After pip install
**What goes wrong:** `ImportError: DLL load failed while importing _q_kangaroo`
**Why it happens:** The `.pyd` depends on `libgmp-10.dll` etc., which are in MinGW/bin on the dev machine but nowhere on the user's machine.
**How to avoid:** Bundle all 5 DLLs into the wheel alongside the `.pyd`, and call `os.add_dll_directory()` pointing to the package directory before importing the native module.
**Warning signs:** Works in dev environment but fails in fresh virtualenv.

**Full DLL dependency chain verified via objdump:**
- `_q_kangaroo.pyd` -> `libgmp-10.dll`, `libmpfr-6.dll`, `libmpc-3.dll`, `python3.dll`, system DLLs
- `libmpfr-6.dll` -> `libgcc_s_seh-1.dll`, `libgmp-10.dll`
- `libmpc-3.dll` -> `libgmp-10.dll`, `libmpfr-6.dll`
- `libgcc_s_seh-1.dll` -> `libwinpthread-1.dll`
- `libgmp-10.dll` -> only KERNEL32.dll, msvcrt.dll (system)

### Pitfall 2: ABI3 Feature Not Propagating
**What goes wrong:** Wheel filename shows `cp314-cp314-win_amd64` instead of `cp39-abi3-win_amd64`.
**Why it happens:** `abi3-py39` feature not added to pyo3 dependency in Cargo.toml, or `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` not set when building with Python 3.14.
**How to avoid:** Add the feature to Cargo.toml AND set the env var. Verify wheel filename after build.
**Warning signs:** Wheel filename contains specific Python version instead of `abi3`.

### Pitfall 3: maturin Features Conflict
**What goes wrong:** Build fails because features are specified in both Cargo.toml and pyproject.toml.
**Why it happens:** pyproject.toml `[tool.maturin] features` overrides or conflicts with Cargo.toml dependency features.
**How to avoid:** Put `abi3-py39` in Cargo.toml's pyo3 dependency features (where it belongs). Use `[tool.maturin] features` only for workspace-level features if needed.
**Warning signs:** Unexpected feature resolution during `maturin build`.

### Pitfall 4: Type Stubs Not Found by IDE
**What goes wrong:** IDE shows no autocomplete despite .pyi files existing.
**Why it happens:** Missing `py.typed` marker file, or stubs placed in wrong directory, or stubs not included in wheel.
**How to avoid:** Place `py.typed` (empty) and `.pyi` files in `python/q_kangaroo/` directory. Verify they appear in the installed package.
**Warning signs:** `python -c "import q_kangaroo; print(q_kangaroo.__file__)"` shows the package path -- check that `py.typed` and `*.pyi` exist there after install.

### Pitfall 5: License Not Included in Wheel
**What goes wrong:** `pip show q-kangaroo` shows no license, or license file missing from distribution.
**Why it happens:** LICENSE file exists at project root but pyproject.toml doesn't reference it, or maturin doesn't find it.
**How to avoid:** Use `license = {file = "LICENSE"}` or `license = "MIT"` (SPDX expression per PEP 639) in pyproject.toml. Maturin will auto-include referenced license files.
**Warning signs:** Check wheel contents with `unzip -l dist/*.whl | grep -i license`.

### Pitfall 6: `requires-python` Mismatch with ABI3
**What goes wrong:** pip refuses to install the wheel on Python 3.9 because metadata says `>=3.14`.
**Why it happens:** `requires-python` defaults to the build Python version if not explicitly set.
**How to avoid:** Explicitly set `requires-python = ">=3.9"` in pyproject.toml (already done).
**Warning signs:** Check metadata with `pip show q-kangaroo` after install.

## Code Examples

### Complete pyproject.toml
```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "q-kangaroo"
version = "0.1.0"
description = "Symbolic computation engine for q-series, partition functions, and modular forms"
readme = "README.md"
requires-python = ">=3.9"
license = "MIT"
keywords = [
    "q-series", "partitions", "modular-forms", "symbolic-computation",
    "number-theory", "theta-functions", "mock-theta", "hypergeometric",
    "bailey-chains", "eta-quotients"
]
authors = [
    { name = "Author Name", email = "author@example.com" }
]
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Science/Research",
    "License :: OSI Approved :: MIT License",
    "Operating System :: Microsoft :: Windows",
    "Operating System :: POSIX :: Linux",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Programming Language :: Python :: 3.13",
    "Programming Language :: Python :: 3.14",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Rust",
    "Topic :: Scientific/Engineering :: Mathematics",
]

[project.urls]
Homepage = "https://github.com/OWNER/q-kangaroo"
Repository = "https://github.com/OWNER/q-kangaroo"
"Bug Tracker" = "https://github.com/OWNER/q-kangaroo/issues"
Documentation = "https://github.com/OWNER/q-kangaroo#readme"

[tool.maturin]
features = ["pyo3/extension-module", "pyo3/abi3-py39"]
python-source = "python"
module-name = "q_kangaroo._q_kangaroo"
include = [
    { path = "python/q_kangaroo/*.dll", format = "wheel" },
]
```

### Cargo.toml with ABI3
```toml
[package]
name = "qsym-python"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[lib]
name = "_q_kangaroo"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.23", features = ["extension-module", "abi3-py39"] }
qsym-core = { path = "../qsym-core" }
```

Note: The `abi3-py39` feature should be specified in EITHER Cargo.toml OR pyproject.toml `[tool.maturin] features`, NOT both. Specifying it in `[tool.maturin] features` as `"pyo3/abi3-py39"` passes the feature through cargo's workspace feature mechanism. The recommended approach is to put it in pyproject.toml's `[tool.maturin] features` since that keeps Cargo.toml clean and the packaging concern in the packaging config. If using the pyproject.toml approach, the Cargo.toml dependency stays as-is: `pyo3 = { version = "0.23", features = ["extension-module"] }`.

### Updated __init__.py DLL Loading
```python
import os
import sys

if sys.platform == "win32":
    _pkg_dir = os.path.dirname(os.path.abspath(__file__))
    # Prefer bundled DLLs from the installed wheel
    if os.path.isfile(os.path.join(_pkg_dir, "libgmp-10.dll")):
        os.add_dll_directory(_pkg_dir)
    else:
        # Fallback for development: MinGW in PATH or env var
        _mingw_dir = os.environ.get("MINGW_BIN", r"C:\mingw64-gcc\mingw64\bin")
        if os.path.isdir(_mingw_dir):
            os.add_dll_directory(_mingw_dir)
```

### CITATION.cff Template
```yaml
cff-version: 1.2.0
message: "If you use this software, please cite it using these metadata."
title: "q-Kangaroo"
abstract: "Open-source symbolic computation engine for q-series, partition functions, and modular forms, replacing proprietary Maple packages."
type: software
authors:
  - family-names: "LastName"
    given-names: "FirstName"
license: MIT
version: 0.1.0
date-released: "2026-02-14"
repository-code: "https://github.com/OWNER/q-kangaroo"
keywords:
  - q-series
  - partitions
  - modular forms
  - symbolic computation
  - number theory
```

### py.typed Marker
An empty file at `python/q_kangaroo/py.typed`. No content needed.

### Example _q_kangaroo.pyi (Abbreviated)
```python
"""Type stubs for the q-Kangaroo native module."""

from fractions import Fraction
from typing import Optional

class QSession:
    """A symbolic computation session owning an expression arena."""
    def __init__(self) -> None: ...
    def symbol(self, name: str) -> QExpr: ...
    def symbols(self, names: str) -> list[QExpr]: ...
    def integer(self, val: int) -> QExpr: ...
    def rational(self, num: int, den: int) -> QExpr: ...
    def infinity(self) -> QExpr: ...
    def stats(self) -> tuple[int, int]: ...
    def generate(self, func_name: str, params: list[int], truncation_order: int) -> QSeries: ...
    def batch_generate(self, func_name: str, param_grid: list[list[int]], truncation_order: int) -> list[tuple[list[int], QSeries]]: ...

class QExpr:
    """A handle to a symbolic expression within a QSession."""
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def _repr_latex_(self) -> str: ...
    def latex(self) -> str: ...
    def __add__(self, other: QExpr) -> QExpr: ...
    def __radd__(self, other: QExpr) -> QExpr: ...
    def __mul__(self, other: QExpr) -> QExpr: ...
    def __rmul__(self, other: QExpr) -> QExpr: ...
    def __neg__(self) -> QExpr: ...
    def __sub__(self, other: QExpr) -> QExpr: ...
    def __pow__(self, exp: QExpr, modulo: object = ...) -> QExpr: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def simplify(self) -> QExpr: ...
    def variant(self) -> str: ...

class QSeries:
    """A q-series (formal power series) with sparse rational coefficients."""
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __getitem__(self, key: int) -> Fraction: ...
    def __len__(self) -> int: ...
    def truncation_order(self) -> int: ...
    def min_order(self) -> Optional[int]: ...
    def is_zero(self) -> bool: ...
    def coeffs(self) -> list[tuple[int, Fraction]]: ...
    def to_dict(self) -> dict[int, Fraction]: ...
    def degree(self) -> Optional[int]: ...
    def low_degree(self) -> Optional[int]: ...
    def __add__(self, other: QSeries) -> QSeries: ...
    def __mul__(self, other: QSeries) -> QSeries: ...
    def __neg__(self) -> QSeries: ...
    def __sub__(self, other: QSeries) -> QSeries: ...
    def invert(self) -> QSeries: ...
    def sift(self, m: int, j: int) -> QSeries: ...

def version() -> str: ...

# Group 1: Pochhammer and q-Binomial
def aqprod(session: QSession, coeff_num: int, coeff_den: int, power: int, n: Optional[int], order: int) -> QSeries: ...
def qbin(session: QSession, n: int, k: int, order: int) -> QSeries: ...

# ... (73 total DSL functions with full signatures)
```

### Build Script for Windows Wheel
```bash
#!/bin/bash
# Build a Windows wheel with bundled DLLs
set -e

export PATH="/c/mingw64-gcc/mingw64/bin:/c/cygwin64/bin:/c/Users/Owner/.cargo/bin:$PATH"
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1

# Copy DLLs to package directory for bundling
cp /c/mingw64-gcc/mingw64/bin/libgmp-10.dll python/q_kangaroo/
cp /c/mingw64-gcc/mingw64/bin/libmpfr-6.dll python/q_kangaroo/
cp /c/mingw64-gcc/mingw64/bin/libmpc-3.dll python/q_kangaroo/
cp /c/mingw64-gcc/mingw64/bin/libgcc_s_seh-1.dll python/q_kangaroo/
cp /c/mingw64-gcc/mingw64/bin/libwinpthread-1.dll python/q_kangaroo/

# Build the wheel
maturin build --release

# Verify wheel filename contains abi3
ls target/wheels/q_kangaroo-*-cp39-abi3-*.whl

# Clean up DLLs from source tree
rm python/q_kangaroo/*.dll
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `license = {text = "MIT"}` | `license = "MIT"` (SPDX expression) | PEP 639, 2024 | Simpler, standard license declaration |
| Manual license file inclusion | `license-files` in pyproject.toml | PEP 639, maturin 1.x | Auto-included in .dist-info |
| Per-Python-version wheels | ABI3 stable ABI wheels | PEP 384, PyO3 0.15+ | Single wheel for Python 3.9-3.14+ |
| Manual .pyi writing | pyo3-stub-gen / PyO3 introspection | 2024-2025 | Still experimental; hand-written stubs recommended for complex APIs |
| `setup.py` / `setup.cfg` | `pyproject.toml` (PEP 621) | 2023+ | All metadata in one file |

**Deprecated/outdated:**
- `setup.py`: Replaced by pyproject.toml for metadata; maturin uses pyproject.toml exclusively
- `license = {text = "..."}`: PEP 639 replaces with SPDX expression string
- PyO3 experimental-inspect feature: Still experimental, not ready for production stubs

## Open Questions

1. **License choice**
   - What we know: MEMORY.md and prior context do not specify a license
   - What's unclear: Which license the user wants (MIT, Apache-2.0, GPL, etc.)
   - Recommendation: Default to MIT in the template; user should confirm before publishing. The pyproject.toml template above uses MIT as placeholder.

2. **Author information**
   - What we know: Needed for pyproject.toml `authors` field and CITATION.cff
   - What's unclear: User's name, email, ORCID (for academic citation)
   - Recommendation: Use placeholder in templates; user fills in before PyPI upload.

3. **GitHub repository URL**
   - What we know: Needed for pyproject.toml `project.urls` and CITATION.cff `repository-code`
   - What's unclear: Whether repo is public yet, actual URL
   - Recommendation: Use placeholder URL; easily changed later.

4. **Linux wheel building and testing**
   - What we know: PKG-01 requires Linux installation. Build environment is Windows/Cygwin.
   - What's unclear: Whether to cross-compile, use CI, or defer Linux wheel to a separate workflow.
   - Recommendation: For this phase, verify the metadata and structure are correct. The actual Linux wheel build would require a Linux environment (or CI like GitHub Actions with manylinux containers). The success criterion says "locally built wheel" -- on Windows we can verify Windows wheel; Linux verification may need to be deferred to CI setup or tested in WSL if available.

5. **Static vs dynamic GMP linking**
   - What we know: Static libraries (`libgmp.a`, `libmpfr.a`, `libmpc.a`) exist. Dynamic linking produces 5 DLL dependencies.
   - What's unclear: Whether `gmp-mpfr-sys` with `use-system-libs` can be forced to static-link via environment variables or build script changes.
   - Recommendation: Use DLL bundling (known to work) rather than investigating static linking (uncertain, may require upstream changes). DLL bundling adds ~3MB to wheel size but eliminates the unknown.

6. **README.md for PyPI long_description**
   - What we know: pyproject.toml can reference `readme = "README.md"` for PyPI page content.
   - What's unclear: Whether a README.md exists or needs to be created.
   - Recommendation: Create a minimal README.md or skip the `readme` field initially. A README is nice-to-have for PyPI but not in the phase requirements.

## Sources

### Primary (HIGH confidence)
- **objdump analysis** of `_q_kangaroo.cp314-win_amd64.pyd` - Verified exact DLL dependency chain (5 DLLs)
- **Project source code** - Read all .rs source files, pyproject.toml, Cargo.toml, __init__.py
- [Maturin User Guide - Configuration](https://www.maturin.rs/config) - `include`, `features`, `python-source` options
- [Maturin User Guide - Project Layout](https://www.maturin.rs/project_layout.html) - Mixed project layout, auto-inclusion rules
- [Maturin User Guide - Metadata](https://www.maturin.rs/metadata.html) - PEP 621 metadata fields
- [PyO3 User Guide - Type Stubs](https://pyo3.rs/main/type-stub.html) - Stub generation approach
- [PyO3 User Guide - Typing Hints](https://pyo3.rs/main/python-typing-hints.html) - .pyi conventions for PyO3
- [GitHub Docs - CITATION files](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-citation-files) - CFF format and GitHub integration
- [Citation File Format schema guide](https://github.com/citation-file-format/citation-file-format/blob/main/schema-guide.md) - CFF 1.2.0 required/optional fields

### Secondary (MEDIUM confidence)
- [Python Packaging Guide - pyproject.toml](https://packaging.python.org/en/latest/guides/writing-pyproject-toml/) - PEP 621 reference
- [PEP 639](https://peps.python.org/pep-0639/) - License metadata specification
- [PyPI Classifiers](https://pypi.org/classifiers/) - Valid trove classifier strings
- [Maturin User Guide - Distribution](https://www.maturin.rs/distribution.html) - Shared library bundling, auditwheel

### Tertiary (LOW confidence)
- pyo3-stub-gen assessment based on GitHub README - not tested with this project's specific API patterns
- Static linking feasibility - based on general gmp-mpfr-sys docs, not tested

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - maturin/PyO3 are well-documented, version-pinned, already in use
- Architecture (DLL bundling): HIGH - DLL dependency chain verified via objdump; maturin include mechanism documented
- Architecture (ABI3): HIGH - PyO3 abi3-py39 feature is well-documented; PYO3_USE_ABI3_FORWARD_COMPATIBILITY already used in project
- Architecture (type stubs): HIGH - PEP 561 is stable; hand-written stubs follow established patterns
- Pitfalls: HIGH - DLL loading issues are the #1 reported problem in PyO3/maturin issue trackers
- pyproject.toml metadata: HIGH - PEP 621 is stable and maturin supports it fully
- CITATION.cff: HIGH - CFF 1.2.0 is stable; GitHub integration is documented

**Research date:** 2026-02-14
**Valid until:** 2026-04-14 (90 days - packaging standards are stable)
