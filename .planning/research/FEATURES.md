# Feature Research: PyPI Release Readiness

**Domain:** Python mathematics research library (symbolic q-series computation)
**Researched:** 2026-02-14
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| PyPI package with sdist + wheel | Standard distribution format for Python libraries | LOW | maturin builds both automatically. Requires manylinux2014+ for Linux compatibility (Rust 1.64+ requires glibc 2.17) |
| Installation via `pip install q-kangaroo` | How researchers install Python libraries | LOW | Already works with maturin. Test on PyPI Test first. |
| Comprehensive pyproject.toml metadata | Classifiers, keywords, description for PyPI discovery | LOW | Need classifiers: "Topic :: Scientific/Engineering :: Mathematics", Python versions, license (BSD like sympy/mpmath) |
| README with quickstart example | First thing users see on PyPI/GitHub | LOW | Must include: install command, 5-line code example, link to docs |
| LICENSE file | Required for academic/research use | LOW | Already exists (check if it's in sdist) |
| API reference documentation | Users need to know what functions exist and their signatures | MEDIUM | Sphinx autodoc with NumPy-style docstrings (standard for scientific Python) |
| Type hints in signatures | Expected by modern Python users, enables IDE autocomplete | LOW | PyO3 0.23 supports type stubs generation. Critical for UX in Jupyter/VS Code |
| Jupyter notebook LaTeX rendering | Math libraries MUST display beautifully in notebooks | LOW | Already implemented via `__repr_latex__()` — table stakes for math libraries |
| Installation verification example | Users need to confirm install worked | LOW | README should include `python -c "import q_kangaroo; print(q_kangaroo.__version__)"` |
| Error messages with context | When operations fail, users need to know why | MEDIUM | Current: Rust panics propagate as Python exceptions. Need custom exception types for common errors (e.g., InvalidParameterError, ConvergenceError) |
| Example gallery/tutorials | Users learn by example, not just API docs | MEDIUM | Sphinx-Gallery to auto-generate from .py files with narrative comments. Standard for matplotlib, scikit-learn |
| GitHub repository metadata | Repository is the second place users look (after PyPI) | LOW | Description, topics, link to docs in "About" section |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| CITATION.cff + Zenodo DOI | Academic users need to cite software properly. GitHub displays citation automatically | LOW | Standard format. Zenodo auto-publishes from GitHub releases. DOI badge in README (sympy, pandas do this) |
| Multi-version Python wheel (abi3) | Single wheel works for Python 3.9-3.14+, smaller downloads | MEDIUM | PyO3 abi3-py39 feature flag. Requires testing across versions. Reduces wheel count from 5+ to 1 per platform |
| Automated CI badges | Shows project health: tests passing, coverage %, latest version | LOW | GitHub Actions with pytest-coverage-comment. Badges for build status, coverage, PyPI version, downloads |
| Performance comparison docs | "q-Kangaroo is 50x faster than Maple for X" — validates switch from Garvan's tools | HIGH | Requires benchmarking harness, Maple equivalent implementations. Strong differentiator for adoption |
| Interactive Binder links | Try library without installing (click link → live Jupyter notebook in cloud) | MEDIUM | Sphinx-Gallery integrates automatically. Free via mybinder.org. Very popular for scientific tutorials |
| Versioned documentation on ReadTheDocs | Users can reference docs for version they have installed | MEDIUM | ReadTheDocs builds automatically from GitHub tags. Free for open source. Handles version selector automatically |
| Mathematical notation in docstrings | API docs render formulas (not just code) — critical for q-series | MEDIUM | Sphinx math extension (supports LaTeX). NumPy-style docstrings with `.. math::` directives. Makes docs match papers |
| Export to LaTeX string | Generate publication-ready equations from symbolic expressions | LOW | Already implemented internally for `__repr_latex__()`. Expose as `.to_latex()` method |
| Reproducibility lock file example | Academic users need exact versions for paper replication | LOW | Provide `requirements-lock.txt` generated via `pip freeze` or `uv pip compile` in examples/ |
| Rich error context in Jupyter | Errors show the problematic expression in LaTeX, not just stack trace | HIGH | Requires custom exception classes with `_repr_latex_()`. Very polished UX, rare in math libraries |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Automatic simplification | "Make everything simplify automatically like Mathematica" | Unpredictable performance (can hang), breaks symbolic equality, loses canonical forms | Keep explicit `.simplify()` as in Phase 2. Document when to use it. Predictability > magic |
| GUI/Desktop application | "Can you make a Mathematica-like interface?" | Huge scope creep, maintenance burden, not how researchers work in 2026 | Jupyter notebooks ARE the GUI. Invest in notebook UX (LaTeX rendering, error messages) |
| Support Python 2.7 | "Some legacy code still uses Python 2" | Python 2 EOL was 2020. PyO3 doesn't support it. Rust toolchain doesn't support it | Minimum Python 3.9 (matches PyO3 abi3 options, sympy's minimum) |
| Numerical approximation fallback | "If symbolic fails, return float" | Breaks type expectations, hides failures, loses exactness guarantee | Provide explicit `.n()` or `.evalf()` methods when numerical is intentional. Never implicit |
| Homebrew formula / conda-forge / apt packages | "Make it easier to install than pip" | Fragmented maintenance, platform-specific bugs, delays releases | pip install works everywhere. Document system dependencies (GMP) in README troubleshooting section |
| Windows binary without MinGW | "Can you make it work without installing extra tools?" | GMP requires MinGW on Windows. Bundling is complex and fragile | Document `os.add_dll_directory()` workaround clearly. Consider static linking in future (high effort) |

## Feature Dependencies

```
[PyPI Package]
    └──requires──> [pyproject.toml metadata]
    └──requires──> [LICENSE file]
    └──requires──> [README with quickstart]

[API Documentation]
    └──requires──> [NumPy-style docstrings in code]
    └──requires──> [Sphinx setup]
    └──enhances──> [Type hints] (auto-extracted via sphinx-autodoc-typehints)

[Example Gallery]
    └──requires──> [API Documentation Sphinx setup]
    └──requires──> [Sphinx-Gallery extension]

[Versioned Docs on ReadTheDocs]
    └──requires──> [API Documentation]
    └──requires──> [GitHub repository with tags/releases]

[CITATION.cff + Zenodo]
    └──requires──> [GitHub repository]
    └──requires──> [First GitHub release]
    └──enhances──> [Academic adoption]

[CI Badges]
    └──requires──> [GitHub Actions workflows]
    └──requires──> [pytest test suite] (already exists)
    └──enhances──> [README]

[Interactive Binder]
    └──requires──> [Example Gallery or notebooks/ directory]
    └──requires──> [requirements.txt for Binder environment]

[Multi-version abi3 wheel]
    └──requires──> [PyO3 abi3 feature flag]
    └──conflicts──> [Python-version-specific APIs]
```

### Dependency Notes

- **API Documentation requires NumPy-style docstrings:** Sphinx autodoc extracts from docstrings. Retrofit existing code (73 DSL functions).
- **Example Gallery requires Sphinx:** Sphinx-Gallery is an extension, not standalone. Must set up Sphinx first.
- **Versioned Docs requires GitHub releases:** ReadTheDocs builds from Git tags. Establishes v0.1.0 release workflow.
- **Interactive Binder enhances Example Gallery:** Sphinx-Gallery auto-generates Binder links if configured. Synergistic features.
- **Type hints enhance API docs:** sphinx-autodoc-typehints extracts types from annotations, reducing docstring duplication.
- **CITATION.cff enhances academic adoption:** Researchers need to cite dependencies. Makes attribution frictionless.
- **CI badges enhance README:** Visual proof of quality. Increases trust for new users.
- **Multi-version abi3 conflicts with version-specific APIs:** Can't use Python 3.14-only features if supporting 3.9+.

## MVP Definition

### Launch With (v0.1.0)

Minimum viable PyPI release — what's needed to be credible as a research library.

- [x] PyPI package (sdist + manylinux wheel) — Already works via maturin, need CI to automate
- [x] Complete pyproject.toml metadata (classifiers, keywords, description, URLs) — Currently minimal
- [x] README with installation + quickstart (< 10 lines of code) — Needs writing
- [x] LICENSE file in sdist — Verify inclusion
- [x] Basic API documentation (Sphinx with autodoc) — No docs currently
- [x] NumPy-style docstrings for all 73 DSL functions — Current docstrings are minimal
- [x] Type hints (`.pyi` stub file from PyO3) — PyO3 can generate
- [x] GitHub repository metadata (description, topics, website link) — Currently bare
- [x] Installation verification instructions in README — Not present
- [x] CI badge in README (tests passing) — No badges currently

**Why essential:** A library on PyPI without docs is unusable. NumPy-style docstrings are table stakes for scientific Python. Type hints enable IDE autocomplete (critical UX). README is the first impression.

### Add After Validation (v0.2.0)

Features to add once core is working and users start trying it.

- [ ] Example gallery (Sphinx-Gallery with 5-10 .py examples) — Trigger: Users ask "how do I...?" repeatedly
- [ ] CITATION.cff + Zenodo DOI — Trigger: First paper using library is submitted
- [ ] Coverage badge + pytest-coverage-comment — Trigger: Setting up CI for v0.1.0
- [ ] Multi-version abi3 wheel — Trigger: Users request Python 3.14 support but we're not ready to drop 3.9
- [ ] Versioned docs on ReadTheDocs — Trigger: v0.2.0 release creates need for version selector
- [ ] Custom exception types (InvalidParameterError, etc.) — Trigger: Users report confusing error messages
- [ ] Mathematical notation in docstrings (Sphinx math extension) — Trigger: Users reference papers and can't match API to formulas
- [ ] Performance comparison docs (vs Maple) — Trigger: Users ask "is this faster than Garvan's tools?"

**Why defer:** Example gallery requires effort to write good examples. CITATION.cff is free but only matters when people cite. Versioned docs only matter with multiple versions. Performance comparisons require Maple setup (not trivial).

### Future Consideration (v1.0+)

Features to defer until library is established.

- [ ] Interactive Binder links — Trigger: Example gallery exists + users want to try without installing
- [ ] Export to LaTeX string API — Trigger: Users want to copy-paste into papers
- [ ] Rich error context in Jupyter — Trigger: Multiple user complaints about error UX
- [ ] Reproducibility lock file examples — Trigger: Paper replication requests
- [ ] Static GMP linking on Windows — Trigger: Windows users struggle with DLL setup

**Why defer:** Binder requires examples first. LaTeX export is nice-to-have (can copy from Jupyter cell output). Rich errors are polish. Lock files are niche (advanced users know `pip freeze`). Static linking is hard, wait for pain threshold.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| PyPI package with wheels | HIGH | LOW (maturin handles) | P1 |
| Complete pyproject.toml | HIGH | LOW (fill template) | P1 |
| README quickstart | HIGH | LOW (write example) | P1 |
| NumPy-style docstrings | HIGH | MEDIUM (73 functions) | P1 |
| Sphinx API docs | HIGH | MEDIUM (setup + autodoc) | P1 |
| Type hints (.pyi stubs) | HIGH | LOW (PyO3 generates) | P1 |
| GitHub repo metadata | MEDIUM | LOW (fill fields) | P1 |
| CI badge | MEDIUM | LOW (GitHub Actions) | P1 |
| Example gallery | HIGH | MEDIUM (write examples) | P2 |
| CITATION.cff + Zenodo | HIGH (for academics) | LOW (template) | P2 |
| Coverage badge | LOW | LOW (GitHub Action) | P2 |
| Multi-version abi3 wheel | MEDIUM | MEDIUM (testing matrix) | P2 |
| Versioned ReadTheDocs | MEDIUM | LOW (auto-build) | P2 |
| Custom exception types | MEDIUM | MEDIUM (refactor errors) | P2 |
| Math notation in docstrings | MEDIUM | MEDIUM (retrofit docs) | P2 |
| Performance comparisons | MEDIUM | HIGH (Maple setup + benchmarks) | P2 |
| Interactive Binder | LOW | LOW (config file) | P3 |
| LaTeX export API | LOW | LOW (expose existing) | P3 |
| Rich Jupyter errors | LOW | HIGH (custom exception display) | P3 |
| Reproducibility lock files | LOW | LOW (generate + document) | P3 |
| Static GMP linking | MEDIUM | HIGH (build complexity) | P3 |

**Priority key:**
- P1: Must have for v0.1.0 (credible PyPI release)
- P2: Should have for v0.2.0 (polished research library)
- P3: Nice to have for v1.0+ (mature ecosystem)

## Competitor Feature Analysis

Comparing q-Kangaroo packaging/docs to established scientific Python libraries:

| Feature | SymPy | mpmath | Our Approach (q-Kangaroo) |
|---------|-------|--------|---------------------------|
| PyPI metadata | Extensive classifiers (OS Independent, CPython, PyPy, Scientific/Engineering/Mathematics/Physics) | BSD license, comprehensive topics, project links (homepage, docs, source, tracker) | Follow SymPy model: BSD, Scientific/Engineering :: Mathematics, Python 3.9-3.13+, link to docs/repo |
| Documentation | Sphinx on docs.sympy.org, versioned, extensive tutorials, API reference | Sphinx on mpmath.org, comprehensive API docs, examples, math rendering | Sphinx with autodoc, ReadTheDocs hosting, NumPy-style docstrings, math extension for LaTeX formulas |
| Example structure | Tutorial sections, example problems, thematic organization | "Basics" section, function demonstrations, use-case driven | Sphinx-Gallery with narrative examples (partition theory, theta functions, mock theta) |
| Citation | No CITATION.cff visible, but widely cited | No CITATION.cff, author-based attribution | CITATION.cff + Zenodo DOI (newer standard, best practice for 2026) |
| Badges | PyPI version, Gitter, Zenodo DOI, downloads, GitHub issues, NumFocus | Build status, code coverage, Zenodo DOI | Build status, coverage, PyPI version (same as mpmath, simpler than SymPy's extensive set) |
| Type hints | Gradually adding (large legacy codebase) | Limited (pre-dates type hint era) | Full type hints from day 1 (PyO3 stubs + manual annotations) — modern advantage |
| Installation | Pure Python, pip only, simple | Pure Python, pip only, simple | Rust extension, maturin-based, need manylinux wheels + document GMP dependency clearly |
| Error messages | Custom exception hierarchy (SymPyError base class) | Standard Python exceptions | Start with Python exceptions, add custom hierarchy in v0.2.0 when error patterns emerge |
| Jupyter integration | LaTeX rendering via `_repr_latex_()`, pretty printing | Text-based output, mpmath.pretty for better display | LaTeX rendering (already implemented), ensure compatibility with IPython.display |
| Performance docs | Not emphasized (symbolic focus) | Precision vs. speed tradeoffs documented | MUST document vs. Maple (competitor is Garvan's tools, not other Python libs) |

### Key Insights from Competitor Analysis:

1. **NumPy-style docstrings are universal** in scientific Python (sympy, mpmath, scipy, numpy)
2. **Sphinx + ReadTheDocs is the standard** (free, automated, versioned)
3. **CITATION.cff is modern best practice** (newer than sympy/mpmath, but becoming standard — pandas uses it)
4. **Type hints are competitive advantage** (established libraries have legacy code, we can start clean)
5. **Manylinux compliance is critical** for Rust extensions (unlike pure Python sympy/mpmath)
6. **Performance matters for our domain** (researchers switching from Maple need speed justification)
7. **Jupyter LaTeX rendering is non-negotiable** for math libraries (already implemented)

## Sources

**PyPI Packaging & Metadata:**
- [Python Packaging User Guide - Overview](https://packaging.python.org/en/latest/overview/)
- [Writing pyproject.toml](https://packaging.python.org/en/latest/guides/writing-pyproject-toml/)
- [PEP 621 – Storing project metadata in pyproject.toml](https://peps.python.org/pep-0621/)
- [SymPy on PyPI](https://pypi.org/project/sympy/)
- [mpmath on PyPI](https://pypi.org/project/mpmath/)

**Maturin & Rust-Python Packaging:**
- [Maturin User Guide - Distribution](https://www.maturin.rs/distribution.html)
- [PyO3 Building and Distribution](https://pyo3.rs/v0.28.0/building-and-distribution.html)
- [Maturin Tutorial](https://www.maturin.rs/tutorial.html)

**Documentation:**
- [Sphinx Documentation Professional Standards](https://wbarillon.medium.com/sphinx-documentation-with-professional-standards-25e5683cb38b)
- [Sphinx and Markdown for Research Software](https://coderefinery.github.io/documentation/sphinx/)
- [NumPy-Style Docstring Format](https://numpydoc.readthedocs.io/en/latest/format.html)
- [ReadTheDocs Sphinx Deployment](https://docs.readthedocs.com/platform/stable/intro/sphinx.html)
- [Sphinx-Gallery Documentation](https://sphinx-gallery.github.io/stable/index.html)

**Academic Citation & Reproducibility:**
- [CITATION.cff File - Zenodo](https://help.zenodo.org/docs/github/describe-software/citation-file/)
- [Citation File Format](https://citation-file-format.github.io/)
- [pyOpenSci - How to Add Citation to Code](https://www.pyopensci.org/lessons/package-share-code/publish-share-code/cite-code.html)
- [pip Repeatable Installs](https://pip.pypa.io/en/stable/topics/repeatable-installs/)
- [PEP 665 – File format for reproducibility](https://peps.python.org/pep-0665/)

**CI/CD & Badges:**
- [Pytest Coverage Comment GitHub Action](https://github.com/marketplace/actions/pytest-coverage-comment)
- [Coverage Badge Action](https://github.com/marketplace/actions/coverage-badge)
- [Making a Coverage Badge - Ned Batchelder](https://nedbatchelder.com/blog/202209/making_a_coverage_badge)

**Jupyter & LaTeX Rendering:**
- [How to Render LaTeX in Jupyter Using IPython](https://medium.com/@idelossantosruiz/how-to-render-latex-output-in-jupyter-notebook-and-other-python-environments-using-ipython-4e1484431e21)
- [LaTeX - Mathematical Python](https://patrickwalls.github.io/mathematicalpython/jupyter/latex/)

**Error Handling & UX:**
- [Python Errors Done Right](https://medium.com/swlh/python-errors-done-right-faa1bfa85d02)
- [Error Handling Strategies in Python](https://llego.dev/posts/error-handling-strategies-best-practices-python/)
- [Google Python Style Guide](https://google.github.io/styleguide/pyguide.html)

**Performance & Benchmarking:**
- [Python Performance Benchmark Suite](https://pyperformance.readthedocs.io/)
- [Python Math Benchmarks](https://www.simonwenkel.com/notes/programming_languages/python/python-math-benchmarks.html)

---
*Feature research for: PyPI release readiness (packaging, documentation, CI, UX polish)*
*Researched: 2026-02-14*
*Confidence: HIGH (verified with official docs, SymPy/mpmath analysis, maturin/PyO3 guides)*
