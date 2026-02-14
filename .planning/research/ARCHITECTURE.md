# Architecture Research

**Domain:** PyPI packaging, GitHub Pages docs, GitHub Actions CI for Rust+Python mixed workspace
**Researched:** 2026-02-14
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                   GitHub Actions CI                          │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │  Linux   │  │  macOS   │  │ Windows  │  │   Docs   │    │
│  │  Build   │  │  Build   │  │  Build   │  │  Build   │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
│       │            │            │            │              │
│       v            v            v            v              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │           Artifact Upload (wheels + docs)           │    │
│  └────────────────────┬────────────────────────────────┘    │
│                       │                                      │
│                       v                                      │
│  ┌─────────────────────────────────────────────────────┐    │
│  │         Release Job (download artifacts)            │    │
│  └────┬────────────────────────┬─────────────────┬─────┘    │
├───────┴────────────────────────┴─────────────────┴──────────┤
│  PyPI (trusted pub)     GitHub Releases      GitHub Pages   │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                 Project Structure                            │
├─────────────────────────────────────────────────────────────┤
│  Cargo.toml (workspace)                                      │
│  pyproject.toml → crates/qsym-python/                       │
│  .github/workflows/ → CI.yml, docs.yml                      │
│  docs/ → Sphinx conf.py, index.rst                          │
│  crates/                                                     │
│    ├── qsym-core/  (Rust lib crate)                         │
│    └── qsym-python/ (cdylib + Python package)               │
│        ├── Cargo.toml                                        │
│        ├── pyproject.toml                                    │
│        ├── src/lib.rs (PyO3 bindings)                        │
│        └── python/q_kangaroo/ (after rename)                 │
│            ├── __init__.py                                   │
│            └── py.typed                                      │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| **Workspace root** | CI config, docs config, top-level metadata | .github/, docs/, Cargo.toml |
| **crates/qsym-core** | Rust library implementation | Standard lib crate |
| **crates/qsym-python** | Python bindings + mixed package | PyO3 cdylib + maturin mixed layout |
| **GitHub Actions workflows** | Test, build wheels, docs, publish | maturin-action, pytest, Sphinx |
| **PyPI** | Python package distribution | Trusted publishing (OIDC) |
| **GitHub Pages** | Documentation hosting | Sphinx + gh-pages branch |

## Recommended Project Structure

### After Rename (qsymbolic → q_kangaroo)

```
C:\cygwin64\home\Owner\Kangaroo/
├── .github/
│   └── workflows/
│       ├── CI.yml              # Rust+Python tests + wheel builds
│       ├── docs.yml            # Sphinx build + deploy to gh-pages
│       └── release.yml         # PyPI publish (trusted publishing)
├── .cargo/
│   └── config.toml             # GMP environment variables
├── docs/
│   ├── conf.py                 # Sphinx configuration
│   ├── index.rst               # Documentation homepage
│   ├── requirements.txt        # sphinx, sphinx_rtd_theme, myst_parser
│   ├── api/                    # API reference (autodoc)
│   ├── guide/                  # User guides
│   └── _static/                # Custom CSS, images
├── Cargo.toml                  # Workspace definition
├── crates/
│   ├── qsym-core/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── tests/
│   └── qsym-python/
│       ├── Cargo.toml          # name: qsym-python, lib.name: _q_kangaroo
│       ├── pyproject.toml      # [project] name: q-kangaroo
│       ├── src/
│       │   └── lib.rs          # #[pymodule] _q_kangaroo
│       ├── python/
│       │   └── q_kangaroo/     # RENAMED from qsymbolic
│       │       ├── __init__.py # from q_kangaroo._q_kangaroo import *
│       │       └── py.typed    # Type hint marker
│       └── tests/              # Python integration tests (pytest)
└── README.md
```

### Structure Rationale

- **.github/workflows/**: Separate CI (tests+build), docs (Sphinx), release (PyPI) for clarity and independent triggers
- **docs/ at root**: Standard for multi-language projects, accessible to both Rust and Python tooling
- **crates/qsym-python/python/**: Maturin mixed layout with `python-source = "python"` in pyproject.toml
- **Module naming**: PyO3 module `_q_kangaroo` (private), Python package `q_kangaroo` (public), PyPI name `q-kangaroo` (hyphenated per PEP 423)
- **.cargo/config.toml**: Build environment for GMP (existing, keep as-is for Windows MinGW)

## Architectural Patterns

### Pattern 1: Maturin Mixed Layout with PyO3

**What:** Rust cdylib extension (`_q_kangaroo.pyd`) co-located with Python package (`q_kangaroo/`)

**When to use:** Projects with Rust performance-critical code + Python convenience layer

**Trade-offs:**
- PRO: Natural import ergonomics (`from q_kangaroo import symbols`)
- PRO: Pure Python code can be modified without recompilation
- CON: Module name collision risk (mitigated by `_` prefix for Rust module)
- CON: IDE confusion (mitigated by `py.typed` and clear naming)

**Example:**
```toml
# crates/qsym-python/pyproject.toml
[tool.maturin]
python-source = "python"
module-name = "q_kangaroo._q_kangaroo"  # Package.RustModule

[tool.maturin]
features = ["pyo3/extension-module"]
```

```python
# crates/qsym-python/python/q_kangaroo/__init__.py
from q_kangaroo._q_kangaroo import QSession, QExpr, QSeries
```

### Pattern 2: Multi-Platform Wheel Matrix with Maturin

**What:** GitHub Actions matrix builds for Linux (manylinux), macOS, Windows with platform-specific configurations

**When to use:** Publishing to PyPI (required for broad compatibility)

**Trade-offs:**
- PRO: Automated cross-compilation with maturin-action
- PRO: Platform-specific dependency handling (GMP via apt/MSYS2)
- CON: Complex CI configuration
- CON: Build time increases (mitigated by parallel jobs)

**Example:**
```yaml
# .github/workflows/CI.yml
strategy:
  matrix:
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
        manylinux: auto
      - os: macos-latest
        target: x86_64-apple-darwin
      - os: windows-latest
        target: x86_64-pc-windows-gnu

steps:
  - uses: PyO3/maturin-action@v1
    with:
      target: ${{ matrix.target }}
      args: --release --out dist
      manylinux: ${{ matrix.manylinux }}
```

### Pattern 3: Sphinx Autodoc for PyO3 Extensions

**What:** Python docstring extraction from compiled PyO3 extension via introspection

**When to use:** Documenting Rust-implemented Python APIs (no Rust docs needed for Python users)

**Trade-offs:**
- PRO: Single source of truth (docstrings in Rust via `#[pyo3(text_signature)]`)
- PRO: Standard Python documentation tooling (Sphinx, ReadTheDocs)
- CON: Requires building extension before doc generation
- CON: No Rust API docs (acceptable for Python-only library)

**Example:**
```python
# docs/conf.py
extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.napoleon',       # Google/NumPy docstring style
    'sphinx.ext.autosummary',
    'sphinx.ext.viewcode',
]

# Ensure package is importable (built with maturin develop)
import sys
from pathlib import Path
sys.path.insert(0, str(Path('../crates/qsym-python/python').resolve()))
```

### Pattern 4: Trusted Publishing (PyPI OIDC)

**What:** Token-less PyPI publishing via GitHub Actions OpenID Connect

**When to use:** Production releases to PyPI (superior security to API tokens)

**Trade-offs:**
- PRO: No long-lived secrets in repo
- PRO: Automatic token expiry
- PRO: GitHub identity verification
- CON: Requires PyPI project setup (one-time)
- CON: Restricted to specific workflow/environment

**Example:**
```yaml
# .github/workflows/release.yml
permissions:
  id-token: write  # REQUIRED for trusted publishing
  contents: read

- name: Publish to PyPI
  uses: PyO3/maturin-action@v1
  with:
    command: upload
    args: --skip-existing dist/*
    # NO MATURIN_PYPI_TOKEN → uses trusted publishing
```

### Pattern 5: Job Dependencies with Artifacts

**What:** Sequential jobs with data passing via upload-artifact/download-artifact

**When to use:** Separating build (wheels) from publish (PyPI/GitHub Pages) for security

**Trade-offs:**
- PRO: Minimal OIDC token exposure (publish job only)
- PRO: Parallel builds, sequential publish
- PRO: Artifacts available for download/debugging
- CON: Additional workflow complexity
- CON: Artifact storage limits (500 per job, v4)

**Example:**
```yaml
jobs:
  build:
    steps:
      - uses: PyO3/maturin-action@v1
        with:
          command: build
          args: --release --out dist
      - uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}
          path: dist

  publish:
    needs: [build]  # Wait for all build jobs
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: wheels-*
          merge-multiple: true
          path: dist
```

## Data Flow

### Build and Publish Flow

```
[Git Push to main/tag]
    ↓
[Trigger CI.yml]
    ↓
┌───────────────────────────────────┐
│  Parallel: Linux / macOS / Windows│
│  1. Install Rust toolchain         │
│  2. Install GMP (apt/brew/MSYS2)   │
│  3. cargo test (qsym-core)         │
│  4. maturin build (qsym-python)    │
│  5. pytest (Python tests)          │
│  6. Upload wheels artifact         │
└───────┬───────────────────────────┘
        ↓ (needs: [linux, macos, windows])
┌───────────────────────────────────┐
│  Publish Job (if tag vX.Y.Z)      │
│  1. Download all wheel artifacts  │
│  2. maturin upload (OIDC)         │
│  3. Create GitHub release         │
└───────────────────────────────────┘
        ↓
[PyPI: q-kangaroo package]
[GitHub Releases: wheel downloads]
```

### Documentation Flow

```
[Git Push to main]
    ↓
[Trigger docs.yml]
    ↓
┌───────────────────────────────────┐
│  Docs Job                          │
│  1. Install Rust + GMP             │
│  2. maturin develop (build ext)    │
│  3. pip install sphinx deps        │
│  4. sphinx-build docs/ docs/_build│
│  5. Upload to gh-pages branch      │
└───────┬───────────────────────────┘
        ↓
[GitHub Pages: https://username.github.io/Kangaroo/]
```

### Rename Migration Flow

```
[Phase 1: Code Changes]
1. Cargo.toml: lib.name = "_q_kangaroo"
2. pyproject.toml: name = "q-kangaroo"
3. Rename python/qsymbolic/ → python/q_kangaroo/
4. Update imports in __init__.py
5. Update #[pymodule] in lib.rs
6. Update all docs references

[Phase 2: PyPI Setup]
1. Register "q-kangaroo" on PyPI
2. Configure trusted publisher (GitHub Actions)

[Phase 3: First Release]
1. Tag v1.0.0 (semantic versioning)
2. CI builds wheels for all platforms
3. Trusted publishing uploads to PyPI

[Legacy: qsymbolic]
- Publish final version (0.1.0) with deprecation warning
- Point users to q-kangaroo in README/pyproject description
- Keep on PyPI (do not delete per best practices)
```

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| **PyPI** | Trusted publishing (OIDC) | Requires `id-token: write`, workflow/environment configured on PyPI |
| **GitHub Pages** | peaceiris/actions-gh-pages@v3 | Deploys to `gh-pages` branch, requires `contents: write` |
| **GitHub Releases** | softprops/action-gh-release@v1 | Attaches wheels to release, requires `contents: write` |
| **maturin-action** | PyO3/maturin-action@v1 | Handles Rust+Python build, cross-compilation, wheel packaging |
| **ReadTheDocs** | Alternative to GitHub Pages | Requires .readthedocs.yml, auto-builds on push |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| **qsym-core ↔ qsym-python** | Cargo workspace dependency | `qsym-python/Cargo.toml`: `qsym-core = { path = "../qsym-core" }` |
| **Rust (_q_kangaroo) ↔ Python (q_kangaroo)** | PyO3 #[pymodule] → import | Python imports compiled `.pyd`/`.so`, re-exports in `__init__.py` |
| **Build job ↔ Publish job** | GitHub Actions artifacts (v4) | Wheels uploaded, downloaded in publish job with `merge-multiple: true` |
| **CI ↔ Documentation** | Separate workflows | CI tests/builds on all pushes, docs only on main branch |
| **Local dev ↔ CI** | .cargo/config.toml | GMP environment variables needed both locally (Windows) and CI (via setup steps) |

## PyPI Package Configuration

### Required pyproject.toml Changes

```toml
# crates/qsym-python/pyproject.toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "q-kangaroo"  # PyPI package name (hyphenated per PEP 423)
version = "1.0.0"    # Semantic versioning
description = "Symbolic computation engine for q-series"
readme = "README.md"
requires-python = ">=3.9"
license = { text = "MIT" }  # Or appropriate SPDX identifier
authors = [
    { name = "Your Name", email = "your.email@example.com" }
]
keywords = ["q-series", "symbolic", "partitions", "number-theory", "rust"]
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Science/Research",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Programming Language :: Python :: 3.13",
    "Programming Language :: Rust",
    "Topic :: Scientific/Engineering :: Mathematics",
]

[project.urls]
Homepage = "https://github.com/yourusername/Kangaroo"
Documentation = "https://yourusername.github.io/Kangaroo/"
Repository = "https://github.com/yourusername/Kangaroo"
Issues = "https://github.com/yourusername/Kangaroo/issues"

[tool.maturin]
features = ["pyo3/extension-module"]
python-source = "python"
module-name = "q_kangaroo._q_kangaroo"  # Package.RustModule
compatibility = "pypi"  # Enforce PyPI platform compatibility
```

### Required Cargo.toml Changes

```toml
# crates/qsym-python/Cargo.toml
[package]
name = "qsym-python"
version = "1.0.0"  # Keep in sync with pyproject.toml
edition = "2024"
rust-version = "1.85"

[lib]
name = "_q_kangaroo"  # Rust module name (with underscore)
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.23", features = ["extension-module"] }
qsym-core = { path = "../qsym-core", version = "1.0.0" }
```

## CI Workflow Configuration

### Comprehensive CI.yml

```yaml
# .github/workflows/CI.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
  workflow_dispatch:

jobs:
  rust-test:
    name: Rust Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install GMP
        run: sudo apt-get update && sudo apt-get install -y libgmp-dev
      - uses: dtolnay/rust-toolchain@stable
      - name: Cargo test
        run: cargo test --workspace --all-features

  python-build-test:
    name: Python ${{ matrix.python-version }} on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python-version: ['3.9', '3.10', '3.11', '3.12', '3.13']
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            manylinux: auto
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-gnu

    steps:
      - uses: actions/checkout@v4

      # Platform-specific GMP installation
      - name: Install GMP (Linux)
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install -y libgmp-dev

      - name: Install GMP (macOS)
        if: runner.os == 'macOS'
        run: brew install gmp

      - name: Install GMP (Windows)
        if: runner.os == 'Windows'
        uses: msys2/setup-msys2@v2
        with:
          msystem: MINGW64
          install: mingw-w64-x86_64-gcc mingw-w64-x86_64-gmp
          update: true

      - uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}

      - name: Build wheel
        uses: PyO3/maturin-action@v1
        with:
          target: ${{ matrix.target }}
          args: --release --out dist --manifest-path crates/qsym-python/Cargo.toml
          manylinux: ${{ matrix.manylinux }}

      - name: Install and test
        run: |
          pip install q-kangaroo --no-index --find-links dist --force-reinstall
          pip install pytest
          pytest crates/qsym-python/tests/

      - name: Upload wheels
        uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}-py${{ matrix.python-version }}
          path: dist/*.whl
```

### Documentation Workflow (docs.yml)

```yaml
# .github/workflows/docs.yml
name: Documentation

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: write  # Required for gh-pages deployment

jobs:
  build-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install GMP
        run: sudo apt-get update && sudo apt-get install -y libgmp-dev

      - uses: dtolnay/rust-toolchain@stable

      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      # Build extension for autodoc
      - name: Build extension
        run: |
          pip install maturin
          cd crates/qsym-python
          maturin develop

      - name: Install Sphinx dependencies
        run: pip install -r docs/requirements.txt

      - name: Build documentation
        run: sphinx-build -b html docs/ docs/_build/html

      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/_build/html
          force_orphan: true
```

### Release Workflow (release.yml)

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

permissions:
  id-token: write      # REQUIRED for trusted publishing
  contents: write      # For GitHub releases

jobs:
  build:
    name: Build on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            manylinux: auto
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-gnu

    steps:
      - uses: actions/checkout@v4

      - name: Install GMP (Linux)
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install -y libgmp-dev

      - name: Install GMP (macOS)
        if: runner.os == 'macOS'
        run: brew install gmp

      - name: Install GMP (Windows)
        if: runner.os == 'Windows'
        uses: msys2/setup-msys2@v2
        with:
          msystem: MINGW64
          install: mingw-w64-x86_64-gcc mingw-w64-x86_64-gmp

      - uses: PyO3/maturin-action@v1
        with:
          target: ${{ matrix.target }}
          args: --release --out dist --manifest-path crates/qsym-python/Cargo.toml
          manylinux: ${{ matrix.manylinux }}

      - uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}
          path: dist/*.whl

  publish:
    name: Publish to PyPI
    needs: [build]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: wheels-*
          merge-multiple: true
          path: dist

      - name: Publish to PyPI
        uses: PyO3/maturin-action@v1
        with:
          command: upload
          args: --skip-existing dist/*
          # NO MATURIN_PYPI_TOKEN → uses trusted publishing

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: dist/*.whl
          generate_release_notes: true
```

## Sphinx Documentation Configuration

### Recommended docs/ Structure

```
docs/
├── conf.py              # Sphinx configuration
├── index.rst            # Homepage
├── requirements.txt     # Sphinx dependencies
├── installation.rst     # Installation guide
├── quickstart.rst       # Quickstart tutorial
├── api/
│   ├── index.rst        # API reference homepage
│   ├── session.rst      # QSession class
│   ├── expressions.rst  # QExpr class
│   └── series.rst       # QSeries class
├── guide/
│   ├── qseries.rst      # q-Series guide
│   ├── partitions.rst   # Partition functions
│   └── identities.rst   # Identity proving
└── _static/
    └── custom.css       # Custom styling
```

### Minimal conf.py

```python
# docs/conf.py
project = 'q-Kangaroo'
copyright = '2026, Your Name'
author = 'Your Name'

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.autosummary',
    'sphinx.ext.napoleon',      # Google/NumPy docstrings
    'sphinx.ext.viewcode',
    'sphinx.ext.intersphinx',
    'myst_parser',              # Markdown support
]

# Ensure package is importable
import sys
from pathlib import Path
sys.path.insert(0, str(Path('../crates/qsym-python/python').resolve()))

# Theme
html_theme = 'sphinx_rtd_theme'
html_static_path = ['_static']

# Autodoc settings
autodoc_member_order = 'bysource'
autodoc_typehints = 'description'

# Intersphinx (link to Python docs)
intersphinx_mapping = {
    'python': ('https://docs.python.org/3', None),
}

# MyST parser for .md files
source_suffix = {
    '.rst': 'restructuredtext',
    '.md': 'markdown',
}
```

### docs/requirements.txt

```
sphinx>=7.0
sphinx-rtd-theme
myst-parser
```

## Anti-Patterns

### Anti-Pattern 1: Hardcoding Module Names

**What people do:** Directly reference `qsymbolic` in CI workflows, docs, and multiple config files

**Why it's wrong:** Renaming requires changes in 10+ locations, easy to miss references, breaks automation

**Do this instead:** Use `${{ env.MODULE_NAME }}` in workflows, single source of truth in pyproject.toml

```yaml
env:
  MODULE_NAME: q-kangaroo

- name: Install package
  run: pip install ${{ env.MODULE_NAME }} --no-index --find-links dist
```

### Anti-Pattern 2: Manual Wheel Building

**What people do:** Run `maturin build` manually for each platform, upload to PyPI from local machine

**Why it's wrong:** Non-reproducible builds, no audit trail, credential exposure, manual matrix management

**Do this instead:** Use GitHub Actions with maturin-action and trusted publishing (shown in workflows above)

### Anti-Pattern 3: Single Workflow for Everything

**What people do:** Put tests, builds, docs, and publishing in one 500-line workflow

**Why it's wrong:** Slow feedback (docs block tests), difficult debugging, unnecessary compute on PR

**Do this instead:** Separate workflows: CI.yml (tests+build on all pushes), docs.yml (main only), release.yml (tags only)

### Anti-Pattern 4: Deleting Old PyPI Package

**What people do:** Remove `qsymbolic` from PyPI after publishing `q-kangaroo`

**Why it's wrong:** Breaks existing users, split audience, no migration path

**Do this instead:** Keep `qsymbolic` 0.1.0 on PyPI with deprecation warning in README and dependency on `q-kangaroo`:

```toml
# Old qsymbolic pyproject.toml
[project]
name = "qsymbolic"
version = "0.1.0"
dependencies = ["q-kangaroo>=1.0.0"]
description = "DEPRECATED: Use q-kangaroo instead"
```

### Anti-Pattern 5: Building Extension in ReadTheDocs Without Cache

**What people do:** Run `maturin develop` on every ReadTheDocs build without caching Rust artifacts

**Why it's wrong:** 5-10 minute doc builds, exceeds free tier timeout, wastes compute

**Do this instead:** Use GitHub Actions for docs (shown above) or configure ReadTheDocs `.readthedocs.yml` with caching:

```yaml
# .readthedocs.yml (if using ReadTheDocs instead of GitHub Pages)
version: 2
build:
  os: ubuntu-22.04
  tools:
    python: "3.11"
    rust: "1.85"
  jobs:
    pre_build:
      - cargo build --release --manifest-path crates/qsym-python/Cargo.toml
      - pip install maturin
      - cd crates/qsym-python && maturin develop

sphinx:
  configuration: docs/conf.py

python:
  install:
    - requirements: docs/requirements.txt
```

### Anti-Pattern 6: Ignoring py.typed Marker

**What people do:** Omit `py.typed` file in Python package

**Why it's wrong:** Type checkers (mypy, pyright) ignore package, poor IDE experience, no static analysis

**Do this instead:** Add empty `crates/qsym-python/python/q_kangaroo/py.typed` file, committed to repo

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| **0-100 users** | Current architecture sufficient: GitHub Actions free tier (2000 min/month), PyPI free hosting, GitHub Pages free |
| **100-1k users** | Monitor CI minutes (add caching for Rust builds), consider ReadTheDocs (better analytics), add download badges |
| **1k-10k users** | Add CDN for docs (Cloudflare), parallel test matrix (split unit/integration), nightly release channel |
| **10k+ users** | Self-hosted runners for CI (cost optimization), private PyPI mirror (reliability), dedicated docs hosting |

### Scaling Priorities

1. **First bottleneck:** CI build time (Rust compilation)
   - **Fix:** Rust build caching via `Swatinem/rust-cache@v2` action
   - **Fix:** Split test/build jobs (don't rebuild for each Python version)

2. **Second bottleneck:** Wheel upload size/time
   - **Fix:** Use `maturin build --strip` for smaller binaries
   - **Fix:** Parallel uploads with `upload-artifact` (already parallelized by matrix)

3. **Third bottleneck:** Documentation build time
   - **Fix:** Cache Sphinx build artifacts
   - **Fix:** Only rebuild on docs/ or python/ changes (workflow path filters)

## Required Changes Summary

### New Files

| File | Purpose |
|------|---------|
| `.github/workflows/CI.yml` | Rust+Python tests, multi-platform wheel builds |
| `.github/workflows/docs.yml` | Sphinx build + GitHub Pages deploy |
| `.github/workflows/release.yml` | PyPI publish on version tags |
| `docs/conf.py` | Sphinx configuration |
| `docs/index.rst` | Documentation homepage |
| `docs/requirements.txt` | Sphinx dependencies |
| `crates/qsym-python/python/q_kangaroo/py.typed` | Type hints marker |

### Modified Files

| File | Changes |
|------|---------|
| `crates/qsym-python/Cargo.toml` | `lib.name = "_q_kangaroo"`, version sync |
| `crates/qsym-python/pyproject.toml` | Full metadata (name, description, classifiers, urls), `module-name`, `compatibility` |
| `crates/qsym-python/src/lib.rs` | `#[pymodule]` name `_q_kangaroo` |
| `crates/qsym-python/python/qsymbolic/` → `q_kangaroo/` | Directory rename |
| `crates/qsym-python/python/q_kangaroo/__init__.py` | Import from `q_kangaroo._q_kangaroo` |
| `README.md` | Add badges (PyPI version, CI status, docs), installation with `pip install q-kangaroo` |

### PyPI Configuration (One-Time)

1. Register package name `q-kangaroo` on PyPI
2. Configure trusted publisher:
   - Owner: `<your-github-username>`
   - Repository: `Kangaroo`
   - Workflow: `release.yml`
   - Environment: (leave empty or create `pypi` environment)

### GitHub Settings (One-Time)

1. Enable GitHub Pages: Settings → Pages → Source: Deploy from branch `gh-pages`
2. (Optional) Create `pypi` environment for release approvals
3. (Optional) Add branch protection rules for `main`

## Build Order for Implementation

**Phase 1: Rename (Breaking Change)**
1. Update Cargo.toml, pyproject.toml with new names
2. Rename `python/qsymbolic/` → `python/q_kangaroo/`
3. Update imports in `__init__.py` and `lib.rs`
4. Update all existing tests to use new import name
5. Test locally: `maturin develop && pytest`

**Phase 2: PyPI Metadata**
1. Add complete `[project]` table to pyproject.toml
2. Add `py.typed` marker file
3. Add classifiers, keywords, urls
4. Verify with `maturin build --sdist` (check metadata)

**Phase 3: CI/CD**
1. Add `.github/workflows/CI.yml` (tests + builds)
2. Test on PR (will build wheels, run tests)
3. Add `.github/workflows/release.yml` (PyPI publish)
4. Configure trusted publisher on PyPI
5. Create test tag (v0.1.0-alpha), verify build/upload

**Phase 4: Documentation**
1. Create `docs/` structure (conf.py, index.rst, requirements.txt)
2. Add `.github/workflows/docs.yml`
3. Test locally: `sphinx-build docs/ docs/_build/html`
4. Push to main, verify GitHub Pages deployment

**Phase 5: Polish**
1. Add badges to README.md
2. Write installation/quickstart docs
3. Add API reference sections
4. Create migration guide (qsymbolic → q-kangaroo)

## Sources

### PyPI Packaging & Maturin
- [Maturin Distribution Guide](https://www.maturin.rs/distribution.html)
- [Best practices for publishing wheels with GitHub Actions](https://github.com/PyO3/maturin/discussions/1309)
- [maturin-action GitHub Repository](https://github.com/PyO3/maturin-action)
- [Maturin Project Layout](https://www.maturin.rs/project_layout.html)
- [pyproject.toml specification](https://packaging.python.org/en/latest/specifications/pyproject-toml/)

### GitHub Actions CI/CD
- [Building and testing Rust - GitHub Docs](https://docs.github.com/en/actions/tutorials/build-and-test-code/rust)
- [GitHub Actions Job Dependencies](https://www.edwardthomson.com/blog/github_actions_17_dependent_jobs)
- [GitHub Actions Artifacts v4](https://github.com/actions/upload-artifact)
- [msys2/setup-msys2 Action](https://github.com/msys2/setup-msys2)

### Documentation
- [How can I generate Python API Doc from Rust PyO3 code?](https://github.com/PyO3/pyo3/discussions/2330)
- [Deploying Sphinx to GitHub Pages](https://coderefinery.github.io/documentation/gh_workflow/)
- [Python Package Documentation Guide](https://inventivehq.com/blog/python-package-documentation-guide)
- [Sphinx/ ReadTheDocs integration guidelines](https://github.com/PyO3/maturin/issues/371)

### Trusted Publishing
- [Publishing to PyPI with Trusted Publishers](https://docs.pypi.org/trusted-publishers/)
- [Configuring OpenID Connect in PyPI - GitHub Docs](https://docs.github.com/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-pypi)
- [Publish releases to PyPI without a password](https://til.simonwillison.net/pypi/pypi-releases-from-github)

### Package Renaming
- [simonw/pypi-rename (Cookiecutter template)](https://github.com/simonw/pypi-rename)
- [Impacts/advice for changing package name](https://discuss.python.org/t/impacts-advice-for-changing-package-name-in-1-0-release/16138)

---
*Architecture research for: q-Kangaroo PyPI/Docs/CI integration*
*Researched: 2026-02-14*
*Confidence: HIGH - Based on official maturin, PyO3, GitHub Actions, PyPI, and Sphinx documentation*
