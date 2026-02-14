# Stack Research

**Domain:** PyPI packaging, documentation, CI/CD, and UX for Rust+PyO3 q-series computation library
**Researched:** 2026-02-14
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **maturin** | 1.12.0 | Build & publish PyO3 wheels to PyPI | Industry standard for PyO3 projects. Minimal config, automatic wheel naming, built-in cross-compilation support. Updated Feb 14, 2026 with Python 3.14 support. |
| **PyO3** | 0.28.1 | Rust-Python bindings | Already in use (0.23). v0.28.1 adds PEP 489 multi-phase init, free-threaded Python support, MSRV 1.83. Stable ABI3 for forward compatibility. |
| **mdBook** | 0.5.2 | Static documentation site | Rust-native documentation generator. Standard for Rust ecosystem. Fast, lightweight, sensible defaults. Preferred over MkDocs for Rust projects. |
| **pytest** | 8.x | Python testing framework | Accepted standard for Python testing in 2026, replacing unittest. Easy syntax, strong features, excellent CI/CD integration. |
| **GitHub Actions** | N/A (SaaS) | CI/CD pipeline | Native GitHub integration. Matrix builds for Rust+Python+multi-platform. Free for public repos. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **pytest-cov** | 7.0.0+ | Coverage reporting | Always. Generates HTML/XML/terminal reports. Supports branch coverage, xdist parallelization. Integrate into CI for automated thresholds. |
| **maturin-action** | v3.x | GitHub Action for maturin | Always for CI. Handles cross-compilation, automatic platform detection, PyPI upload with trusted publishing. |
| **peaceiris/actions-mdbook** | v2.x | GitHub Action for mdBook | Always for docs CI. Installs mdBook on Linux/macOS/Windows runners. |
| **peaceiris/actions-gh-pages** | v4.x | Deploy to GitHub Pages | Always for docs deployment. Handles static file upload, CNAME, custom domains. |
| **mdbook-mermaid** | 0.14.0+ | Diagram support for mdBook | Optional. Adds mermaid.js diagrams to documentation. Useful for architecture diagrams. |
| **mdbook-katex** | 0.10.0+ | Math rendering for mdBook | Optional but recommended. Renders LaTeX math in docs. Critical for q-series formulas. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| **cargo audit** | Rust dependency security scanning | Run in CI. Checks against RustSec Advisory Database. 2026 best practice: scan deployed binaries, not just Cargo.lock. |
| **cargo deny** | License/dependency policy enforcement | Optional but recommended. Gates duplicate dependencies, enforces license policy, blocks unwanted sources. |
| **uv** | Fast Python package installer/publisher | Maturin docs recommend `uv publish` for PyPI upload (faster than twine). |

## Installation

```bash
# Maturin (for local development)
pip install maturin==1.12.0

# Python dev dependencies (pyproject.toml)
pip install pytest pytest-cov

# mdBook (for local docs builds)
cargo install mdbook --version 0.5.2

# mdBook plugins (optional)
cargo install mdbook-mermaid mdbook-katex

# Rust security tools (optional)
cargo install cargo-audit cargo-deny
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| **maturin** | setuptools-rust | Never for new PyO3 projects. Maturin has better defaults, less config, native PyO3 support. |
| **mdBook** | MkDocs (Python) | If team is Python-only and needs extensive customization. MkDocs has larger plugin ecosystem but requires Python dependency. For Rust projects, mdBook is the no-brainer choice. |
| **mdBook** | Sphinx (Python) | If you need API doc generation from docstrings. Sphinx excels at reference docs. mdBook is better for narrative documentation. |
| **GitHub Actions** | Travis CI / CircleCI | If you need specialized hardware (GPUs, ARM servers). GitHub Actions covers 95% of use cases and has better GitHub integration. |
| **pytest** | unittest | Never for new projects. pytest is the 2026 standard. unittest is legacy. |
| **pytest-cov** | coverage.py (direct) | Never when using pytest. pytest-cov provides better integration, automatic .coverage handling, default reporting. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| **setuptools-rust** | Opinionated but manual config. Maturin is PyO3-optimized. | **maturin** — designed for PyO3, minimal config |
| **twine** (for upload) | Slower than modern tools. | **uv publish** — recommended by maturin docs |
| **Gitbook** | Deprecated/commercial pivot. | **mdBook** — active, open-source, Rust-native |
| **unittest** | Legacy Python testing. Verbose syntax. | **pytest** — modern standard, better DX |
| **Manual wheel builds** | Error-prone, no auditwheel/delocate automation. | **maturin + maturin-action** — handles platform tags, manylinux compliance |

## Stack Patterns by Variant

**For PyPI wheel building (Linux):**
- Use `maturin-action` with `manylinux: auto` or explicit version (e.g., `2014`, `2_28`)
- Set `--compatibility pypi` flag to enforce PyPI pre-upload checks
- Use Docker containers for manylinux compliance (automatic via maturin-action)
- Install GMP: `apt-get update && apt-get install -y libgmp-dev` in workflow

**For PyPI wheel building (Windows):**
- Use `maturin-action` on `windows-latest` runner
- Install GMP via MSYS2: `msys2/setup-msys2@v2` with `mingw-w64-ucrt-x86_64-gmp` package
- Target: `x86_64-pc-windows-gnu` (matches existing build)
- UCRT64 is 2026 MSYS2 default (replaces MINGW64)

**For PyPI wheel building (macOS — deferred):**
- Use `maturin-action` on `macos-latest` runner
- GMP: `brew install gmp`
- Target: `x86_64-apple-darwin` (Intel) and `aarch64-apple-darwin` (Apple Silicon)
- Deferred until Phase 9+

**For documentation:**
- Use mdBook with mdbook-katex (critical for LaTeX math)
- Optional: mdbook-mermaid for architecture diagrams
- Deploy to GitHub Pages via peaceiris actions
- Build in CI, deploy on push to main

**For CI matrix:**
- Rust: `stable` (MSRV 1.83 from PyO3 0.28)
- Python: `3.9, 3.10, 3.11, 3.12, 3.13, 3.14` (ABI3 means single wheel supports all)
- Platforms: `ubuntu-latest` (Linux), `windows-latest` (Windows)
- Don't test all Python versions on all platforms (use ABI3 wheel + single test per platform)

**For Jupyter integration:**
- Implement `_repr_html_()` and `_repr_latex_()` methods on QExpr/QSeries
- Return strings (HTML or LaTeX markup)
- No return value (None) is treated as method doesn't exist
- Use existing LaTeX rendering from Phase 1-8

**For error handling (Python exceptions from Rust):**
- Use `PyResult<T>` (alias for `Result<T, PyErr>`)
- Implement `From<CustomError> for PyErr` for custom Rust errors
- PyO3 auto-converts via `?` operator
- Map to appropriate Python exception types (ValueError, TypeError, RuntimeError)
- Use `pyo3::exceptions` module for standard exceptions

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| **PyO3 0.28.1** | maturin 1.12.0+ | maturin 1.12.0 updated max Python to 3.14 (matches PyO3 0.28) |
| **PyO3 0.28.1** | Rust 1.83+ | MSRV bumped from 1.63 (PyO3 0.23) to 1.83 (PyO3 0.28) |
| **PyO3 ABI3** | Python 3.9-3.14 | Use `abi3-py09` feature for minimum Python 3.9 support |
| **maturin 1.12.0** | Python 3.7+ | Tool requirement (not extension minimum) |
| **pytest 8.x** | Python 3.8+ | Drop Python 3.7 support |
| **mdBook 0.5.2** | Rust 1.74+ | MSRV for mdBook itself |
| **GitHub Actions ubuntu-latest** | Currently Ubuntu 22.04 LTS | libgmp-dev available via apt |
| **GitHub Actions windows-latest** | Currently Windows Server 2022 | MSYS2 provides GMP via mingw-w64 packages |

## Configuration Details

### pyproject.toml (maturin)

```toml
[build-system]
requires = ["maturin>=1.12,<2.0"]
build-backend = "maturin"

[project]
name = "q-kangaroo"
# Metadata merged from Cargo.toml, pyproject.toml takes precedence
dependencies = []  # Runtime dependencies (none for pure Rust lib)
requires-python = ">=3.9"  # Matches abi3-py09
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Science/Research",
    "Topic :: Scientific/Engineering :: Mathematics",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Rust",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Programming Language :: Python :: 3.13",
    "Programming Language :: Python :: 3.14",
]

[project.urls]
Homepage = "https://github.com/<user>/Kangaroo"
Documentation = "https://<user>.github.io/Kangaroo"
Repository = "https://github.com/<user>/Kangaroo"
"Bug Tracker" = "https://github.com/<user>/Kangaroo/issues"

[tool.maturin]
python-source = "python"  # If Python code in python/ dir
features = ["pyo3/abi3-py09"]  # Enable ABI3 with Python 3.9 minimum
compatibility = "linux"  # Or "manylinux2014", set per-platform in CI
strip = true  # Strip debug symbols from release builds
```

### Cargo.toml (PyO3 + ABI3)

```toml
[dependencies]
pyo3 = { version = "0.28.1", features = ["abi3-py09"] }

# For forward compatibility with unreleased Python versions
# Set environment variable: PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
```

### book.toml (mdBook + plugins)

```toml
[book]
title = "q-Kangaroo Documentation"
authors = ["<author>"]
language = "en"
src = "docs/src"
build-dir = "docs/book"

[preprocessor.mermaid]
command = "mdbook-mermaid"

[preprocessor.katex]
# LaTeX math rendering (critical for q-series formulas)

[output.html]
mathjax-support = false  # Use KaTeX instead (faster)
git-repository-url = "https://github.com/<user>/Kangaroo"
edit-url-template = "https://github.com/<user>/Kangaroo/edit/main/{path}"
```

### GitHub Actions (.github/workflows/ci.yml)

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
        python-version: ['3.9', '3.14']  # Min and max for quick check
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - name: Install GMP (Ubuntu)
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install -y libgmp-dev

      - name: Install GMP (Windows)
        if: runner.os == 'Windows'
        uses: msys2/setup-msys2@v2
        with:
          msystem: UCRT64
          update: true
          install: mingw-w64-ucrt-x86_64-gmp

      - uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}

      - uses: dtolnay/rust-toolchain@stable

      - name: Install test dependencies
        run: pip install pytest pytest-cov maturin

      - name: Build extension
        run: maturin develop --release

      - name: Run Rust tests
        run: cargo test --all-features

      - name: Run Python tests
        run: pytest tests/ --cov=q_kangaroo --cov-report=xml --cov-report=html

      - name: Upload coverage
        uses: codecov/codecov-action@v4  # Optional: requires Codecov account
        if: matrix.os == 'ubuntu-latest' && matrix.python-version == '3.14'

  build-wheels:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            manylinux: auto
          - target: x86_64-pc-windows-gnu
            os: windows-latest
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - uses: PyO3/maturin-action@v1
        with:
          target: ${{ matrix.target }}
          manylinux: ${{ matrix.manylinux }}
          args: --release --locked --compatibility pypi
          sccache: true  # Cache Rust compilation

      - uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.target }}
          path: target/wheels/*.whl

  publish:
    if: github.event_name == 'push' && startsWith(github.ref, 'refs/tags/')
    needs: [test, build-wheels]
    runs-on: ubuntu-latest
    permissions:
      id-token: write  # For PyPI trusted publishing
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: wheels-*
          merge-multiple: true
          path: dist/

      - uses: pypa/gh-action-pypi-publish@release/v1
        # Uses OIDC trusted publishing (no API token needed)
```

### GitHub Actions (.github/workflows/docs.yml)

```yaml
name: Documentation

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    permissions:
      contents: write  # For GitHub Pages deployment
    steps:
      - uses: actions/checkout@v4

      - uses: peaceiris/actions-mdbook@v2
        with:
          mdbook-version: '0.5.2'

      - name: Install mdBook plugins
        run: |
          cargo install mdbook-mermaid mdbook-katex

      - name: Build book
        run: mdbook build

      - uses: peaceiris/actions-gh-pages@v4
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/book
          cname: docs.example.com  # Optional: custom domain
```

## Jupyter Integration Pattern

```python
# In crates/qsym-python/src/expr.rs (PyO3)

#[pymethods]
impl QExpr {
    fn _repr_html_(&self) -> PyResult<String> {
        // Use existing to_html() or similar
        Ok(format!("<div class='qexpr'>{}</div>", self.to_latex()?))
    }

    fn _repr_latex_(&self) -> PyResult<String> {
        // Use existing to_latex() from Phase 1-8
        Ok(format!("${}$", self.to_latex()?))
    }
}
```

## Error Handling Pattern

```rust
// In crates/qsym-core/src/error.rs

use pyo3::PyErr;
use pyo3::exceptions::{PyValueError, PyRuntimeError};

#[derive(Debug)]
pub enum QSymError {
    DivisionByZero,
    InvalidSeries(String),
    // ... other errors
}

impl std::fmt::Display for QSymError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            QSymError::DivisionByZero => write!(f, "Division by zero"),
            QSymError::InvalidSeries(msg) => write!(f, "Invalid series: {}", msg),
        }
    }
}

impl From<QSymError> for PyErr {
    fn from(err: QSymError) -> PyErr {
        match err {
            QSymError::DivisionByZero => PyValueError::new_err(err.to_string()),
            QSymError::InvalidSeries(_) => PyValueError::new_err(err.to_string()),
        }
    }
}

// Usage in PyO3 functions:
#[pyfunction]
fn some_function() -> PyResult<f64> {
    let result = compute()?;  // QSymError auto-converts to PyErr via From trait
    Ok(result)
}
```

## Sources

**HIGH CONFIDENCE:**
- [maturin 1.12.0 on PyPI](https://pypi.org/project/maturin/) — version, release date (Feb 14, 2026)
- [Maturin Distribution Guide](https://www.maturin.rs/distribution.html) — PyPI publishing, cross-compilation, manylinux
- [maturin-action README](https://github.com/PyO3/maturin-action) — CI configuration, platform support
- [PyO3 0.28.0 Building & Distribution](https://pyo3.rs/v0.28.0/building-and-distribution) — ABI3, maturin config
- [PyO3 Error Handling](https://pyo3.rs/v0.22.5/function/error-handling) — PyResult, exception mapping
- [IPython Integration Guide](https://ipython.readthedocs.io/en/stable/config/integrating.html) — _repr_html_, _repr_latex_
- [pytest Best Practices](https://docs.pytest.org/en/stable/explanation/goodpractices.html) — project structure, fixtures
- [mdBook Documentation](https://rust-lang.github.io/mdBook/) — installation, configuration
- [MSYS2 CI Setup](https://www.msys2.org/docs/ci/) — GitHub Actions integration
- [Python Packaging Guide: pyproject.toml](https://packaging.python.org/en/latest/guides/writing-pyproject-toml/) — classifiers, metadata

**MEDIUM CONFIDENCE:**
- [PyO3 Releases](https://github.com/pyo3/pyo3/releases) — 0.28.1 version (inferred from search results)
- [mdBook Releases](https://github.com/rust-lang/mdbook/releases) — 0.5.2 version
- [pytest-cov Documentation](https://pytest-cov.readthedocs.io/en/latest/reporting.html) — coverage features
- [cargo audit on RustSec](https://rustsec.org/) — security scanning
- [Rust Security Guide 2026](https://sherlock.xyz/post/rust-security-auditing-guide-2026) — 2026 best practices

---
*Stack research for: PyPI packaging + docs + CI for Rust+PyO3 q-series library*
*Researched: 2026-02-14*
