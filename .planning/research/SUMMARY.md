# Project Research Summary

**Project:** q-Kangaroo v1.1 (PyPI Release & Documentation)
**Domain:** PyPI packaging, CI/CD, and documentation for Rust+PyO3 symbolic computation library
**Researched:** 2026-02-14
**Confidence:** HIGH

## Executive Summary

The q-Kangaroo project is an established Rust-based symbolic computation engine with 8 completed phases (578+ tests) that now needs professional release infrastructure to reach its target audience—mathematics researchers transitioning from Frank Garvan's Maple-based tools. The recommended approach follows the modern PyO3/maturin stack with GitHub Actions CI/CD, Sphinx documentation on GitHub Pages, and PyPI trusted publishing.

The path forward requires: (1) renaming the Python package from "qsymbolic" to "q-kangaroo" to align with branding, (2) building manylinux-compatible wheels for Linux and native MinGW wheels for Windows with proper GMP dependency handling, (3) establishing comprehensive documentation infrastructure with NumPy-style docstrings and Sphinx autodoc, and (4) implementing multi-platform CI with proper build caching and testing matrices.

Key risks center on GMP bundling for Windows wheels (no automatic solution exists), module renaming pitfalls (triple-name coordination required), and ensuring ABI3 forward compatibility for Python 3.9-3.14+. Mitigation strategies include explicit DLL bundling documentation for Windows, systematic verification of all rename touchpoints, and proper PyO3 feature flags with forward compatibility environment variables.

## Key Findings

### Recommended Stack

Research confirmed that the PyO3/maturin ecosystem has matured into the industry standard for Rust-Python libraries as of 2026. The stack prioritizes minimal configuration, broad compatibility, and proven tooling.

**Core technologies:**
- **maturin 1.12.0**: PyO3-optimized build and publish tool with automatic wheel naming, manylinux support, and built-in cross-compilation — updated Feb 2026 with Python 3.14 support
- **PyO3 0.28.1**: Rust-Python bindings with abi3-py09 for cross-version wheels (single wheel supports Python 3.9-3.14+) — current project uses 0.23, upgrade path clear
- **Sphinx + ReadTheDocs**: Standard documentation stack for scientific Python (sympy, mpmath pattern) with autodoc for API reference, math rendering for formulas, and versioned hosting
- **pytest**: Testing standard for Python (replacing unittest) with pytest-cov for coverage reporting
- **GitHub Actions**: Native CI/CD with maturin-action for wheel builds, platform matrix support (Linux/macOS/Windows), and free tier sufficient for academic projects

**Critical configuration patterns:**
- ABI3 with explicit minimum version: `pyo3 = { version = "0.28", features = ["abi3-py09"] }` produces single wheel per platform
- Maturin mixed layout: Rust cdylib (`_q_kangaroo`) + Python package (`q_kangaroo/`) for natural import ergonomics
- Trusted publishing via OIDC: Token-less PyPI upload from GitHub Actions (superior security to API tokens)
- sccache over target/ caching: 50% faster CI builds with concurrent cache fetch vs multi-GB target/ blobs

### Expected Features

Analysis of scientific Python libraries (SymPy, mpmath) and PyPI packaging standards reveals clear feature tiers.

**Must have (table stakes):**
- PyPI package with sdist + manylinux wheel — researchers expect `pip install q-kangaroo` to just work
- Complete pyproject.toml metadata with classifiers for discoverability (Topic :: Scientific/Engineering :: Mathematics)
- NumPy-style docstrings for all 73 DSL functions — scientific Python standard, enables Sphinx autodoc
- Type hints via .pyi stubs — modern Python expectation, enables IDE autocomplete in Jupyter/VSCode
- README with installation verification and 5-line quickstart example
- Jupyter LaTeX rendering via `_repr_latex_()` — already implemented, non-negotiable for math libraries
- API reference documentation (Sphinx) — users need to discover what functions exist

**Should have (competitive):**
- CITATION.cff + Zenodo DOI — academic users need proper attribution, GitHub displays citation automatically
- Multi-version abi3 wheels — single wheel for Python 3.9-3.14+ reduces PyPI footprint and download size
- CI badges (build status, coverage) — visual proof of project health
- Example gallery with Sphinx-Gallery — researchers learn by example (partition congruences, theta identities)
- Versioned documentation on ReadTheDocs — users reference docs matching their installed version
- Mathematical notation in docstrings — LaTeX formulas in API docs match paper notation

**Defer (v2+):**
- Interactive Binder links (try without installing) — requires example gallery first
- Performance comparison docs vs Maple — high value but needs benchmarking infrastructure
- Rich error context in Jupyter (LaTeX-rendered errors) — polish feature, wait for user feedback
- Static GMP linking on Windows — technically complex, defer until Windows users hit pain threshold

### Architecture Approach

The recommended architecture follows proven PyO3/maturin patterns with careful handling of GMP native dependencies and multi-platform wheel distribution.

**Major components:**
1. **Package Structure**: Maturin mixed layout with `python-source = "python"`, Rust module `_q_kangaroo` (private), Python package `q_kangaroo` (public), PyPI name `q-kangaroo` (hyphenated per PEP 423)
2. **CI/CD Pipeline**: Separate workflows for testing (CI.yml), documentation (docs.yml), and release (release.yml) with independent triggers — tests on all pushes, docs on main only, publish on version tags
3. **Build Matrix**: GitHub Actions matrix with platform-specific GMP installation (apt on Linux, brew on macOS, MSYS2 on Windows), maturin-action with sccache for 50% build speedup, parallel wheel builds with artifact upload/download pattern
4. **Documentation**: Sphinx at workspace root with autodoc extracting from built extension, myst-parser for markdown support, peaceiris actions for GitHub Pages deployment, NumPy-style docstrings as single source of truth
5. **Wheel Distribution**: manylinux2014+ for Linux (glibc 2.17+ required by Rust 1.64+), x86_64-pc-windows-gnu native build for Windows (matches existing MinGW setup), PyPI trusted publishing via OIDC (no long-lived tokens)

**Key patterns:**
- **Job dependencies with artifacts**: Build jobs run in parallel per platform, publish job downloads merged artifacts and uploads to PyPI sequentially — minimizes OIDC token exposure
- **Maturin mixed layout**: Enables pure Python convenience wrappers around Rust core, natural import ergonomics (`from q_kangaroo import aqprod`)
- **Sphinx autodoc for PyO3**: Requires building extension before doc generation (`maturin develop`), extracts docstrings via introspection, no separate Rust API docs needed for Python-only library
- **Platform-specific GMP handling**: Linux uses apt libgmp-dev with automatic wheel repair, Windows uses MSYS2 mingw-w64 packages with documented DLL directory setup, macOS deferred to future phases

### Critical Pitfalls

Research identified 8 high-impact pitfalls with proven mitigation strategies from PyO3/maturin community.

1. **Module Name Triple-Mismatch** — Renaming requires coordinating `[lib] name` in Cargo.toml, `module-name` in pyproject.toml, and `#[pymodule]` decorator. Mismatch causes "ImportError: dynamic module does not define module export function" with no obvious indication which piece is wrong. Prevention: Systematic grep verification plus test matrix (develop install, wheel install, pytest).

2. **ABI3 Feature Without Minimum Version** — Bare `abi3` feature creates version-specific wheels (cp314-cp314) instead of cross-version (cp09-abi3). Prevention: Always use `abi3-py09` not bare `abi3`, verify wheel filename pattern.

3. **GMP Bundling Failures on Windows** — Maturin cannot automatically bundle GMP DLLs into Windows wheels (no patchelf equivalent). Users get "DLL load failed" errors. Prevention: Short-term document GMP requirement with install instructions, long-term include DLLs directly in wheel via manual packaging.

4. **Auditwheel Repair with Read-Only Libraries** — Docker images with read-only GMP cause patchelf permission errors during wheel repair. Prevention: Use maturin 0.13+ (includes fix) or add `chmod -R u+w target/` before build.

5. **Python 3.13+ Maximum Version Error** — PyO3 rejects Python 3.13+ unless `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` environment variable is set. Prevention: Set in CI workflows and document for local development.

## Implications for Roadmap

Based on research findings, the milestone naturally divides into 4 sequential phases with clear dependencies and deliverables.

### Phase 1: Package Rename & Structure
**Rationale:** Renaming is a breaking change that affects all subsequent work. Must complete first to avoid cascading updates across CI configs, documentation, and metadata.

**Delivers:**
- `q-kangaroo` PyPI package name (hyphenated)
- `q_kangaroo` Python module (underscored)
- `_q_kangaroo` Rust cdylib (prefixed)
- Verified import in all contexts (develop, wheel, pytest)

**Addresses:** Pitfall #1 (triple-mismatch), establishes foundation for packaging work

**Avoids:** Late-stage rename requiring changes across CI workflows, documentation, and published wheels

### Phase 2: PyPI Packaging & Metadata
**Rationale:** Package metadata and wheel configuration must be correct before any test uploads. ABI3 configuration enables forward compatibility testing in subsequent phases.

**Delivers:**
- Complete pyproject.toml with classifiers, keywords, URLs
- ABI3 wheels (single wheel per platform for Python 3.9-3.14+)
- LICENSE file verified in sdist
- Platform-specific GMP handling strategy documented

**Uses:** maturin 1.12.0 (STACK.md), PyO3 abi3-py09 feature

**Implements:** Maturin mixed layout pattern (ARCHITECTURE.md)

**Addresses:** Pitfalls #2 (ABI3 feature), #3 (GMP bundling), Feature requirements (pyproject.toml metadata, type hints)

### Phase 3: Multi-Platform CI/CD
**Rationale:** CI infrastructure must validate wheels work on target platforms before writing extensive documentation. Build caching optimizations prevent CI from becoming bottleneck.

**Delivers:**
- GitHub Actions workflows (CI, docs, release)
- Linux wheel builds (manylinux2014, auditwheel verified)
- Windows wheel builds (MinGW, GMP dependency tested)
- sccache configuration for 50% build speedup
- Trusted publishing to PyPI Test configured

**Implements:** Job dependencies with artifacts pattern, platform matrix builds

**Addresses:** Pitfalls #4 (auditwheel), #5 (Python 3.13+), #8 (cache bloat), Feature requirements (CI badges)

**Avoids:** Manual wheel building, credential exposure, non-reproducible releases

### Phase 4: Documentation & Examples
**Rationale:** Documentation requires built extension (maturin develop) and finalized API. NumPy-style docstrings enable both Sphinx autodoc and IDE autocomplete.

**Delivers:**
- NumPy-style docstrings for all 73 DSL functions
- Sphinx documentation with autodoc and math rendering
- GitHub Pages deployment via docs workflow
- README with installation, quickstart, and verification
- Example gallery with 5-10 narrative .py files
- CITATION.cff + Zenodo DOI setup

**Implements:** Sphinx autodoc for PyO3 pattern (ARCHITECTURE.md)

**Addresses:** Feature requirements (API docs, examples, citation), UX pitfalls (generic import errors, missing version info)

### Phase Ordering Rationale

- **Rename before packaging**: Avoids updating package metadata twice, prevents confusion with multiple package names in flight
- **Packaging before CI**: CI needs correct pyproject.toml and ABI3 config to build proper wheels, testing bad configuration wastes CI time
- **CI before documentation**: Documentation workflow depends on build infrastructure (maturin develop, dependencies), want stable CI before investing in docs
- **Documentation last**: Docstrings are user-facing polish, deferrable until package is installable and tested on target platforms

### Research Flags

**Phases likely needing deeper research during planning:**
- **Phase 2 (Packaging):** Windows GMP DLL bundling strategy needs technical validation — research manual DLL inclusion vs static linking vs conda-forge fallback
- **Phase 4 (Documentation):** pyo3-stub-gen workflow for type hints needs testing — verify Sphinx can read generated .pyi files, IDE autocomplete works

**Phases with standard patterns (skip research-phase):**
- **Phase 1 (Rename):** Straightforward find-replace with verification checklist, no novel patterns
- **Phase 3 (CI):** Well-documented GitHub Actions patterns, maturin-action handles complexity

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Official maturin/PyO3 docs, recent version updates (maturin 1.12.0 Feb 2026), verified community adoption |
| Features | HIGH | Direct analysis of SymPy/mpmath (established scientific Python libraries), PyPI packaging guide (official), NumPy-style docstring standard |
| Architecture | HIGH | Proven maturin-action patterns, trusted publishing documented by PyPI/GitHub, Sphinx widely deployed for Rust-Python projects |
| Pitfalls | HIGH | All 8 pitfalls sourced from official PyO3/maturin GitHub issues with verified solutions, community blog posts with benchmarks |

**Overall confidence:** HIGH

### Gaps to Address

- **Windows GMP bundling**: Research identified the problem (no automatic bundling) and short-term workaround (documentation), but long-term solution (static linking vs manual DLL inclusion) needs technical validation during Phase 2 planning. Defer decision until testing shows user pain level.

- **PyO3 0.23 → 0.28.1 upgrade path**: Current project uses PyO3 0.23, research recommends 0.28.1 (adds PEP 489 multi-phase init, free-threaded Python support, MSRV 1.83). Verify no breaking changes for existing Session/Expr/Series APIs during Phase 2. Stack research shows feature flags are compatible.

- **macOS wheel builds**: Architecture research includes macOS patterns (brew install gmp, x86_64/aarch64 targets) but deferred to post-v1.1 per project scope. Gap acknowledged, not blocking for initial PyPI release targeting Linux/Windows users.

## Sources

### Primary (HIGH confidence)
- [Maturin 1.12.0 on PyPI](https://pypi.org/project/maturin/) — version verification, release date Feb 14 2026
- [Maturin Distribution Guide](https://www.maturin.rs/distribution.html) — PyPI publishing, manylinux, wheel repair, cross-compilation
- [PyO3 Building & Distribution](https://pyo3.rs/v0.28.0/building-and-distribution.html) — ABI3 configuration, maturin integration, feature flags
- [PyO3 Error Handling](https://pyo3.rs/v0.22.5/function/error-handling) — PyResult, exception mapping patterns
- [Python Packaging Guide - pyproject.toml](https://packaging.python.org/en/latest/guides/writing-pyproject-toml/) — classifiers, metadata specification
- [PyPI Trusted Publishers](https://docs.pypi.org/trusted-publishers/) — OIDC configuration, GitHub Actions workflow setup
- [Sphinx Documentation](https://www.sphinx-doc.org/) — configuration, autodoc extension, math rendering
- [NumPy-Style Docstrings](https://numpydoc.readthedocs.io/en/latest/format.html) — scientific Python documentation standard
- [GitHub Actions - Building Rust](https://docs.github.com/en/actions/use-cases-and-examples/building-and-testing/building-and-testing-rust) — CI patterns, matrix builds, caching

### Secondary (MEDIUM confidence)
- [PyO3/maturin #1960](https://github.com/PyO3/maturin/issues/1960) — Python 3.13 forward compatibility issue
- [PyO3/maturin #2909](https://github.com/PyO3/maturin/issues/2909) — editable install .so missing after upgrade
- [PyO3/maturin #1292](https://github.com/PyO3/maturin/pull/1292) — auditwheel repair fix for read-only libraries
- [sccache in GitHub Actions](https://depot.dev/blog/sccache-in-github-actions) — performance comparison vs cargo cache
- [SymPy on PyPI](https://pypi.org/project/sympy/) — metadata analysis for scientific library patterns
- [mpmath on PyPI](https://pypi.org/project/mpmath/) — packaging patterns for math libraries
- [Sphinx-Gallery](https://sphinx-gallery.github.io/) — example gallery generation from .py files

### Tertiary (LOW confidence, needs validation)
- [pyo3-stub-gen](https://github.com/Jij-Inc/pyo3-stub-gen) — automated .pyi generation from PyO3, community tool not official PyO3
- Static GMP linking on Windows — mentioned in community discussions but no authoritative guide found, needs experimentation

---
*Research completed: 2026-02-14*
*Ready for roadmap: yes*
