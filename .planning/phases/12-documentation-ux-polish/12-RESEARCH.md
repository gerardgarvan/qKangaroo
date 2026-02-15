# Phase 12: Documentation & UX Polish - Research

**Researched:** 2026-02-15
**Domain:** Sphinx documentation, Jupyter rendering, PyO3 docstrings, Python UX
**Confidence:** HIGH

## Summary

Phase 12 covers two interleaved concerns: comprehensive documentation (README, Sphinx site, docstrings, examples) and UX polish (Jupyter LaTeX rendering, terminal repr, error messages, sensible defaults). The project is well-positioned: QExpr already has `_repr_latex_()` (rendering via Rust's `render::to_latex`), PEP 561 type stubs already exist (`.pyi` files), and PyO3's Rust doc comments (`///`) already flow through as Python `__doc__` strings.

The key technical decision is **how to generate API documentation for a native extension with 73+ functions**. Sphinx 8.2.0 (released 2025-02-18) added native support for reading `.pyi` stub files in autodoc, which is directly applicable since the project already ships `_q_kangaroo.pyi`. This means we can use Sphinx autodoc without needing to build the native module during docs CI -- the stubs provide signatures and docstrings can come from the Rust source (exposed at runtime). However, for a docs-only CI build without a compiled `.pyd`, the stubs alone provide signatures but NOT docstrings (those live in the compiled module). The most robust approach is to build the module in the docs CI job (which already works on Ubuntu in `ci.yml`) and use autodoc against the live module.

For QSeries LaTeX rendering, the `_repr_latex_()` method needs to be added to the Rust `QSeries` implementation. The existing `FormalPowerSeries` display format (`1 - q + q^2 + O(q^10)`) provides the text repr; a parallel LaTeX renderer is needed for Jupyter output (`1 - q + q^2 + O(q^{10})`). This is straightforward: iterate BTreeMap entries and emit LaTeX-formatted terms.

**Primary recommendation:** Use Sphinx 8.2+ with `autodoc` + `napoleon` + `mathjax` + `sphinx-math-dollar`, build the native module in docs CI, deploy to GitHub Pages via `peaceiris/actions-gh-pages@v4`. Use Furo theme for clean, modern look. Add `_repr_latex_()` to QSeries in Rust. Write NumPy-style docstrings in Rust `///` comments on all 73 `#[pyfunction]` functions.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Sphinx | >=8.2 | Documentation generator | 8.2+ has native .pyi stub support; industry standard |
| furo | >=2024.8 | Sphinx HTML theme | Clean, responsive, dark mode, recommended by Scientific Python |
| sphinx.ext.autodoc | (built-in) | API reference from live module | Extracts docstrings + signatures from compiled PyO3 module |
| sphinx.ext.napoleon | (built-in) | NumPy-style docstring support | Parses NumPy/Google docstrings into reST |
| sphinx.ext.mathjax | (built-in) | LaTeX math rendering in HTML | Renders $...$ and $$...$$ via MathJax JS |
| sphinx-math-dollar | >=1.2.1 | Dollar-sign math in docstrings/RST | Enables `$\theta_3(q)$` syntax instead of `:math:` |
| sphinx.ext.intersphinx | (built-in) | Cross-reference external docs | Link to Python stdlib, NumPy, etc. |
| sphinx-copybutton | >=0.5 | Copy button on code blocks | UX polish for code examples |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| nbsphinx | >=0.9 | Jupyter notebook rendering in Sphinx | Example gallery (5+ narrative examples) |
| sphinx.ext.autosummary | (built-in) | Summary tables for API reference | Generates per-function pages from module |
| sphinx-autodoc-typehints | >=2.0 | Type hints in documentation | Renders parameter types from stubs |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Furo | pydata-sphinx-theme | PyData has more features (search, version switcher) but heavier; Furo is simpler and faster |
| autodoc | sphinx-autoapi | AutoAPI parses source without importing -- useful when module can't be built, but we CAN build in CI |
| nbsphinx | myst-nb | myst-nb has better caching but nbsphinx has gallery support and simpler setup |
| RST | MyST (markdown) | MyST is more familiar to many devs, but math-heavy docs are cleaner in RST with mathjax |

**Installation (docs build):**
```bash
pip install "sphinx>=8.2" furo sphinx-math-dollar sphinx-copybutton nbsphinx sphinx-autodoc-typehints
```

## Architecture Patterns

### Recommended Documentation Structure
```
docs/
├── conf.py              # Sphinx configuration
├── index.rst            # Landing page with toctree
├── installation.rst     # Installation guide (pip, from source)
├── quickstart.rst       # Getting started tutorial
├── api/
│   ├── index.rst        # API reference overview
│   ├── session.rst      # QSession class docs
│   ├── expr.rst         # QExpr class docs
│   ├── series.rst       # QSeries class docs
│   ├── pochhammer.rst   # Group 1: aqprod, qbin
│   ├── products.rst     # Group 2: etaq, jacprod, etc.
│   ├── theta.rst        # Group 3: theta2, theta3, theta4
│   ├── partitions.rst   # Group 4: partition_count, etc.
│   ├── analysis.rst     # Group 5: qfactor, prodmake, etc.
│   ├── relations.rst    # Group 6-7: findlincombo, etc.
│   ├── hypergeometric.rst # Group 8: phi, psi, etc.
│   ├── identity.rst     # Group 9: prove_eta_id, etc.
│   └── mock_theta.rst   # Group 10: mock theta, Appell-Lerch, Bailey
├── examples/
│   ├── partition_congruences.ipynb  # Ramanujan congruences
│   ├── theta_identities.ipynb      # Jacobi theta function identities
│   ├── hypergeometric_summation.ipynb # q-Gauss, q-Vandermonde
│   ├── mock_theta_functions.ipynb   # Classical mock theta overview
│   └── bailey_chains.ipynb          # Bailey pair discovery
├── mathematical_notation.rst  # LaTeX notation conventions
└── requirements.txt     # Docs build dependencies
```

### Pattern 1: NumPy-Style Docstrings in Rust (PyO3)
**What:** Write NumPy-style docstrings as Rust `///` doc comments on `#[pyfunction]` functions. PyO3 automatically exposes these as Python `__doc__` strings. The `napoleon` extension parses them into structured API docs.
**When to use:** All 73 DSL functions + 3 classes
**Example:**
```rust
// In dsl.rs
/// Compute the generalized eta product: $(q^b; q^t)_\infty$.
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// b : int
///     Base exponent. The product starts at $q^b$.
/// t : int
///     Step size. Successive factors use $q^{b+t}, q^{b+2t}, \ldots$
/// order : int
///     Truncation order $N$: series is computed to $O(q^N)$.
///
/// Returns
/// -------
/// QSeries
///     The formal power series $(q^b; q^t)_\infty = \prod_{k=0}^{\infty}(1 - q^{b+kt})$
///     truncated at $O(q^N)$.
///
/// Examples
/// --------
/// >>> s = QSession()
/// >>> euler = etaq(s, 1, 1, 20)  # (q;q)_inf = Euler function
/// >>> euler[0]
/// Fraction(1, 1)
///
/// Notes
/// -----
/// This is the building block for eta-quotients and modular forms.
/// The Dedekind eta function is $\eta(\tau) = q^{1/24} (q;q)_\infty$
/// where $q = e^{2\pi i \tau}$.
///
/// See Also
/// --------
/// aqprod : General q-Pochhammer symbol with arbitrary base monomial.
/// jacprod : Jacobi triple product built from eta products.
#[pyfunction]
pub fn etaq(session: &QSession, b: i64, t: i64, order: i64) -> QSeries {
    // ...
}
```

### Pattern 2: QSeries LaTeX Rendering
**What:** Add `_repr_latex_()` method to QSeries in Rust, generating LaTeX from FormalPowerSeries coefficients.
**When to use:** Jupyter notebook display
**Example implementation approach:**
```rust
// In series.rs
/// LaTeX representation for Jupyter notebooks.
fn _repr_latex_(&self) -> String {
    let mut parts = Vec::new();
    let var = "q";
    for (&k, c) in &self.fps.coefficients {
        // Format each term as LaTeX: coefficient * q^{k}
        // Handle: negative signs, fractions (\frac{p}{q}), unit coefficients, etc.
    }
    if parts.is_empty() {
        format!("$O({}^{{{}}})$", var, self.fps.truncation_order())
    } else {
        parts.push(format!("O({}^{{{}}})", var, self.fps.truncation_order()));
        format!("${}$", parts.join(""))
    }
}
```

### Pattern 3: API Reference with autodoc
**What:** Use `.. autofunction::` and `.. autoclass::` directives to pull docstrings from the live module.
**When to use:** Each API reference page
**Example:**
```rst
.. api/products.rst

Named Products
==============

.. autofunction:: q_kangaroo.etaq

.. autofunction:: q_kangaroo.jacprod

.. autofunction:: q_kangaroo.tripleprod

.. autofunction:: q_kangaroo.quinprod

.. autofunction:: q_kangaroo.winquist
```

### Pattern 4: Sphinx conf.py for Math-Heavy Scientific Docs
**What:** Configuration optimized for mathematical content with dollar-sign LaTeX.
**When to use:** `docs/conf.py`
**Example:**
```python
# docs/conf.py
project = "q-Kangaroo"
copyright = "2025, Author Name"
author = "Author Name"

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
    "sphinx.ext.napoleon",
    "sphinx.ext.mathjax",
    "sphinx.ext.intersphinx",
    "sphinx_math_dollar",
    "sphinx_copybutton",
    "sphinx_autodoc_typehints",
    "nbsphinx",
]

# Theme
html_theme = "furo"

# Napoleon settings for NumPy-style docstrings
napoleon_google_docstring = False
napoleon_numpy_docstring = True
napoleon_use_param = True
napoleon_use_rtype = True
napoleon_preprocess_types = True

# Math: allow $...$ for inline, $$...$$ for display
mathjax3_config = {
    "tex": {
        "inlineMath": [["\\(", "\\)"]],
        "displayMath": [["\\[", "\\]"]],
    }
}

# Intersphinx
intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
}

# Autodoc
autodoc_member_order = "bysource"
autodoc_typehints = "description"

# nbsphinx
nbsphinx_execute = "never"  # Notebooks are pre-executed
```

### Pattern 5: GitHub Actions Docs Deployment
**What:** CI workflow that builds Sphinx docs and deploys to GitHub Pages.
**When to use:** `.github/workflows/docs.yml`
**Example:**
```yaml
name: Documentation

on:
  push:
    branches: [main]
  pull_request:

permissions:
  contents: write

jobs:
  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - uses: dtolnay/rust-toolchain@stable
      - name: Install GMP
        run: sudo apt-get update && sudo apt-get install -y libgmp-dev
      - name: Build Python extension
        working-directory: crates/qsym-python
        run: |
          pip install maturin
          PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --release
      - name: Install docs dependencies
        run: pip install -r docs/requirements.txt
      - name: Build documentation
        run: sphinx-build -b html docs docs/_build/html -W
      - name: Deploy to GitHub Pages
        if: github.event_name == 'push' && github.ref == 'refs/heads/main'
        uses: peaceiris/actions-gh-pages@v4
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: docs/_build/html
```

### Anti-Patterns to Avoid
- **Writing docstrings in .pyi stubs instead of Rust:** The .pyi files are for type information only. Docstrings belong in Rust `///` comments so they're available at runtime via `help()`. The stubs should NOT duplicate docstrings.
- **Using sphinx-autoapi for a buildable module:** Since CI can build the native module, use autodoc (which gives better docstring extraction) rather than autoapi (which only parses source).
- **Notebooks that execute during Sphinx build:** Set `nbsphinx_execute = "never"` and pre-execute notebooks. Building the native module + executing notebooks adds fragility and build time.
- **Monolithic API reference page:** With 73+ functions, a single page is unreadable. Split by functional group (Groups 1-10).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| LaTeX in HTML docs | Custom JS math rendering | `sphinx.ext.mathjax` + `sphinx-math-dollar` | MathJax handles all LaTeX edge cases, $-sign syntax is natural |
| API docs from source | Manual RST for each function | `sphinx.ext.autodoc` + `napoleon` | 73 functions -- manual docs will drift from code |
| NumPy docstring parsing | Custom docstring parser | `sphinx.ext.napoleon` | Standard, well-tested, handles all NumPy sections |
| Notebook rendering | Custom notebook-to-HTML | `nbsphinx` | Handles cell execution, output rendering, thumbnail galleries |
| GitHub Pages deployment | Custom deployment scripts | `peaceiris/actions-gh-pages@v4` | Battle-tested, handles .nojekyll, caching, custom domains |
| FPS LaTeX rendering | Python-side string formatting | Rust `_repr_latex_()` method | Series can have 100+ terms; Rust is faster and co-located with data |

**Key insight:** The documentation stack is mature and well-integrated. Every component (Sphinx, napoleon, mathjax, nbsphinx, furo) is designed to work together. The only custom code needed is the QSeries `_repr_latex_()` method and the NumPy-style docstrings in Rust.

## Common Pitfalls

### Pitfall 1: Sphinx autodoc can't find the native module
**What goes wrong:** `sphinx-build` fails with `ModuleNotFoundError: No module named '_q_kangaroo'` because the native .pyd/.so isn't built.
**Why it happens:** autodoc imports the module at build time to extract docstrings. If the native extension isn't installed, it fails.
**How to avoid:** In docs CI, build the module with `maturin develop --release` BEFORE running `sphinx-build`. The existing `ci.yml` Python tests job already does this on Ubuntu.
**Warning signs:** Empty function signatures, missing docstrings in rendered API pages.

### Pitfall 2: Dollar signs in docstrings conflicting with sphinx-math-dollar
**What goes wrong:** Literal dollar signs (e.g., in error messages) get misinterpreted as math delimiters.
**Why it happens:** `sphinx-math-dollar` aggressively parses `$...$` as inline math.
**How to avoid:** Escape literal dollar signs with backslash. In Rust docstrings, use `\$` for literal dollar signs. In practice, math docstrings rarely contain literal `$`.
**Warning signs:** Garbled text in rendered docs where `$` appears.

### Pitfall 3: LaTeX in Rust docstrings needs double backslashes
**What goes wrong:** `$\frac{1}{2}$` in a Rust `///` comment becomes `$rac{1}{2}$` in Python because `\f` is consumed.
**Why it happens:** Rust string processing may interpret backslash sequences in some contexts.
**How to avoid:** In Rust raw doc comments (`///`), backslashes are NOT processed -- they pass through verbatim. So `/// $\frac{1}{2}$` works correctly. However, verify this with a test. If issues arise, use `\\frac`.
**Warning signs:** Broken LaTeX rendering in `help()` or Sphinx output.

### Pitfall 4: QSeries _repr_latex_ with large coefficient fractions
**What goes wrong:** A term like `(12345/67890)*q^3` renders as a huge fraction in LaTeX, making the output unreadable.
**Why it happens:** Exact rational arithmetic produces large numerators/denominators.
**How to avoid:** Implement truncation: if the series has more than ~20 non-zero terms, show first 15 + `\cdots` + last 2 + O(q^N). Also, for very large numerator/denominator (> 6 digits), consider a compact display.
**Warning signs:** Jupyter cells with horizontally-scrolling LaTeX output.

### Pitfall 5: Notebooks in docs repo with platform-specific output
**What goes wrong:** Notebooks executed on Windows produce different repr output than Linux, or contain Windows-specific paths.
**Why it happens:** `__repr__` output may vary by platform, and notebook metadata includes kernel info.
**How to avoid:** Pre-execute notebooks on Linux (where docs CI runs), strip output metadata. Use `nbsphinx_execute = "never"` in conf.py.
**Warning signs:** Diff noise in notebook JSON, CI failures from output comparison.

### Pitfall 6: Missing .nojekyll file for GitHub Pages
**What goes wrong:** CSS/JS files in directories starting with `_` (like `_static/`, `_sources/`) are 404'd on GitHub Pages.
**Why it happens:** GitHub Pages uses Jekyll by default, which ignores `_`-prefixed directories.
**How to avoid:** `peaceiris/actions-gh-pages` automatically adds `.nojekyll`. If deploying manually, add an empty `.nojekyll` file to the output root.
**Warning signs:** Docs site loads with no styling.

### Pitfall 7: PyO3 text_signature interaction with docstrings
**What goes wrong:** `help(etaq)` shows a garbled signature because the first line of the docstring contains a signature-like string that CPython misinterprets.
**Why it happens:** CPython's `inspect.signature()` parses `__text_signature__` from the first line of `__doc__`. If the first line of the Rust docstring looks like a signature, it gets consumed.
**How to avoid:** Use `#[pyo3(text_signature = "(session, b, t, order)")]` explicitly on functions where auto-generation produces bad signatures. Or rely on auto-generation from `#[pyo3(signature = (...))]`.
**Warning signs:** `help(func)` shows incorrect or missing parameter names.

## Code Examples

### QSeries LaTeX Rendering Implementation
```rust
// In crates/qsym-python/src/series.rs
// Add to #[pymethods] impl QSeries:

/// LaTeX representation for Jupyter notebooks, wrapped in $...$.
fn _repr_latex_(&self) -> String {
    let var = "q";
    let mut result = String::new();
    let mut first = true;
    let max_terms = 20; // truncate display for readability
    let num_terms = self.fps.coefficients.len();
    let show_ellipsis = num_terms > max_terms;

    for (idx, (&k, c)) in self.fps.coefficients.iter().enumerate() {
        if show_ellipsis && idx >= max_terms - 2 && idx < num_terms - 2 {
            if idx == max_terms - 2 {
                result.push_str(" + \\cdots");
            }
            continue;
        }

        let is_negative = c.0.cmp0() == std::cmp::Ordering::Less;
        let abs_c = if is_negative { -c.clone() } else { c.clone() };
        let abs_is_one = !abs_c.is_zero()
            && *abs_c.0.numer() == *abs_c.0.denom();

        if first {
            if is_negative { result.push('-'); }
        } else {
            if is_negative {
                result.push_str(" - ");
            } else {
                result.push_str(" + ");
            }
        }

        // Format coefficient
        let coeff_str = if abs_is_one && k != 0 {
            String::new() // coefficient is 1, omit
        } else if *abs_c.0.denom() == 1u32 {
            format!("{}", abs_c.0.numer()) // integer
        } else {
            format!("\\frac{{{}}}{{{}}}", abs_c.0.numer(), abs_c.0.denom())
        };

        // Format variable part
        let var_str = match k {
            0 => String::new(),
            1 => var.to_string(),
            _ => format!("{}^{{{}}}", var, k),
        };

        if k == 0 {
            result.push_str(&coeff_str);
        } else if coeff_str.is_empty() {
            result.push_str(&var_str);
        } else {
            result.push_str(&format!("{} {}", coeff_str, var_str));
        }

        first = false;
    }

    if first {
        // Zero series
        result = format!("O({}^{{{}}})", var, self.fps.truncation_order());
    } else {
        result.push_str(&format!(" + O({}^{{{}}})", var, self.fps.truncation_order()));
    }

    format!("${}$", result)
}
```

### NumPy-Style Docstring Template for DSL Functions
```rust
/// Compute the q-binomial (Gaussian) coefficient $[n \choose k]_q$.
///
/// The q-binomial coefficient is a polynomial in $q$ defined by:
///
/// $$[n \choose k]_q = \frac{(q;q)_n}{(q;q)_k (q;q)_{n-k}}$$
///
/// Parameters
/// ----------
/// session : QSession
///     The computation session.
/// n : int
///     Upper index (non-negative integer).
/// k : int
///     Lower index ($0 \le k \le n$).
/// order : int
///     Truncation order $N$: series is computed to $O(q^N)$.
///
/// Returns
/// -------
/// QSeries
///     The q-binomial coefficient as a formal power series.
///
/// Raises
/// ------
/// ValueError
///     If $k < 0$ or $k > n$.
///
/// Examples
/// --------
/// >>> from q_kangaroo import QSession, qbin
/// >>> s = QSession()
/// >>> b = qbin(s, 5, 2, 20)
/// >>> b[0]
/// Fraction(1, 1)
/// >>> b[1]
/// Fraction(1, 1)
///
/// Notes
/// -----
/// The q-binomial satisfies the recurrence:
/// $[n \choose k]_q = [n-1 \choose k-1]_q + q^k [n-1 \choose k]_q$
///
/// See Also
/// --------
/// aqprod : General q-Pochhammer symbol used in the definition.
#[pyfunction]
pub fn qbin(session: &QSession, n: i64, k: i64, order: i64) -> QSeries {
    // ...
}
```

### Error Message Pattern (UX-03)
```rust
// Pattern for clear, helpful error messages
if b <= 0 {
    return Err(pyo3::exceptions::PyValueError::new_err(format!(
        "etaq(): parameter 'b' must be positive, got b={}. \
         The base exponent b defines the starting power q^b in the product (q^b; q^t)_inf.",
        b
    )));
}

if t <= 0 {
    return Err(pyo3::exceptions::PyValueError::new_err(format!(
        "etaq(): parameter 't' must be positive, got t={}. \
         The step t defines the q-spacing: factors are (1-q^b), (1-q^{{b+t}}), (1-q^{{b+2t}}), ...",
        t
    )));
}
```

### Default Session Pattern (UX-04)
The existing `symbols()` function in `__init__.py` already implements the default session pattern:
```python
def symbols(names: str, session=None):
    if session is None:
        session = QSession()
    result = session.symbols(names)
    if len(result) == 1:
        return result[0]
    return tuple(result)
```

For DSL functions requiring a session, we can add Python wrapper functions in `__init__.py` that accept `session=None` and create a default:
```python
# In __init__.py -- convenience wrappers with defaults
def etaq(session_or_b, b_or_t=None, t_or_order=None, order=None, *, session=None):
    """Wrapper that allows calling with or without explicit session."""
    # This is complex. Better approach: keep the existing API but provide
    # a module-level default session for quick interactive use.

_default_session = None

def get_default_session():
    global _default_session
    if _default_session is None:
        _default_session = QSession()
    return _default_session
```

**Recommendation for UX-04:** Rather than wrapping all 73 functions, add a documented `get_default_session()` helper and ensure the `symbols()` convenience function is well-documented. The explicit session parameter is actually GOOD API design for a computation library -- it makes truncation order and state management clear. Document this design choice prominently.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| sphinx-autoapi (source parsing) | Sphinx 8.2+ autodoc .pyi support | Feb 2025 | Can use autodoc with native modules even when module not importable |
| RST-only docstrings | sphinx-math-dollar ($...$) | 2020+ | Natural LaTeX syntax in docstrings |
| sphinx_rtd_theme | Furo / pydata-sphinx-theme | 2021+ | Modern, responsive, dark mode |
| Manual notebook HTML | nbsphinx / myst-nb | 2020+ | Automatic notebook rendering with gallery support |
| PyO3 manual text_signature | PyO3 auto-generated signatures | PyO3 0.20+ | `#[pyo3(signature)]` auto-generates `__text_signature__` |

**Deprecated/outdated:**
- `sphinx_rtd_theme`: Still works but Furo/PyData themes are preferred for new projects
- `sphinx.ext.pngmath`: Replaced by mathjax years ago
- Manual `.rst` API stubs: autodoc + autosummary generates these automatically

## Specific Findings for Key Questions

### Q1: Sphinx autodoc with PyO3/maturin projects
**Answer (HIGH confidence):** Sphinx 8.2.0 (released 2025-02-18) added native support for .pyi stub files in autodoc. The project already ships `_q_kangaroo.pyi` with full function signatures. For BEST results, build the native module in CI and let autodoc import it -- this gives both signatures AND docstrings. The stubs serve as fallback for signature information.
**Source:** [Sphinx 8.2 changelog](https://www.sphinx-doc.org/en/master/changes/8.2.html), [Sphinx issue #7630](https://github.com/sphinx-doc/sphinx/issues/7630)

### Q2: Deploying Sphinx to GitHub Pages
**Answer (HIGH confidence):** Use `peaceiris/actions-gh-pages@v4` in a GitHub Actions workflow. The workflow builds docs and pushes to `gh-pages` branch. Configure GitHub Pages to serve from that branch. This is the standard approach used by the Sphinx project itself.
**Source:** [Sphinx deployment docs](https://www.sphinx-doc.org/en/master/tutorial/deploying.html)

### Q3: _repr_latex_() on PyO3 classes
**Answer (HIGH confidence):** Implement in Rust as a `#[pymethods]` function returning `String`. QExpr already has this -- it calls `render::to_latex()` on the arena expression and wraps in `$...$`. QSeries needs a new implementation that iterates `FormalPowerSeries.coefficients` (BTreeMap) and formats each term as LaTeX. This is purely a formatting function with no FFI complexity.
**Source:** Existing implementation in `crates/qsym-python/src/expr.rs` lines 47-51.

### Q4: Documentation structure for scientific packages
**Answer (HIGH confidence):** Scientific Python packages (SymPy, SciPy, NumPy) use: API reference organized by topic, getting-started guide, example gallery (often Jupyter notebooks), and mathematical notation reference. The Scientific Python Development Guide recommends Furo theme, napoleon extension, and intersphinx.
**Source:** [Scientific Python Development Guide](https://learn.scientific-python.org/development/guides/docs/)

### Q5: Sphinx extensions for math-heavy documentation
**Answer (HIGH confidence):** Use `sphinx.ext.mathjax` (built-in) + `sphinx-math-dollar` (for `$...$` syntax) + `sphinx.ext.napoleon` (NumPy docstrings). SymPy's own `sphinx-math-dollar` extension is battle-tested for math-heavy docs.
**Source:** [sphinx-math-dollar docs](https://www.sympy.org/sphinx-math-dollar/)

### Q6: NumPy-style docstrings on PyO3 functions
**Answer (HIGH confidence):** Write NumPy-style docstrings as Rust `///` doc comments. PyO3 passes these through as `__doc__` strings. Napoleon parses the Parameters/Returns/Examples sections. This is how other PyO3 scientific projects document their APIs. The docstrings support LaTeX via `sphinx-math-dollar` when rendered in Sphinx.
**Source:** [PyO3 function docs](https://pyo3.rs/v0.25.0/function/signature), [Napoleon docs](https://www.sphinx-doc.org/en/master/usage/extensions/napoleon.html)

### Q7: Sensible defaults with PyO3
**Answer (HIGH confidence):** PyO3 supports default parameter values via `#[pyo3(signature = (session, b, t, order=20))]`. For Python-side defaults (like optional session), use wrapper functions in `__init__.py`. The existing `symbols()` helper already demonstrates this pattern.
**Source:** [PyO3 signature docs](https://pyo3.rs/v0.25.0/function/signature)

### Q8: Default session pattern
**Answer (MEDIUM confidence):** Two viable approaches: (1) module-level `get_default_session()` for quick interactive use, or (2) keep explicit session in all functions (current design). Recommendation: keep explicit session as primary API (it's better for reproducibility and truncation control), but add a documented `get_default_session()` convenience for interactive/notebook use. Do NOT make session optional in the core DSL functions -- it would change the API contract for 73 functions.

## Scope Assessment

### What Already Exists (DO NOT recreate)
- QExpr `_repr_latex_()` -- already implemented in Rust (`expr.rs:47-51`)
- QExpr `__repr__()` / `__str__()` -- already implemented (`expr.rs:36-44`)
- QSeries `__repr__()` / `__str__()` -- already implemented via FPS Display (`series.rs:34-41`)
- Type stubs `.pyi` -- already exist with full signatures
- `__init__.py` with `symbols()` helper -- already has default session pattern
- Basic README with installation and example -- exists
- CI workflow (`ci.yml`) with Ubuntu Python test job -- builds native module

### What Needs to Be Built
1. **QSeries `_repr_latex_()`** -- new Rust method in `series.rs`
2. **NumPy-style docstrings** -- rewrite 73 `///` doc comments in Rust `dsl.rs`
3. **Sphinx documentation site** -- `docs/` directory with conf.py, RST files
4. **API reference RST files** -- 10 topic pages using autodoc directives
5. **Example gallery** -- 5 Jupyter notebooks
6. **Getting-started guide** -- narrative RST
7. **Docs CI workflow** -- `.github/workflows/docs.yml`
8. **README expansion** -- fuller installation, quickstart, badges
9. **Error message improvements** -- add function name + suggestions to PyO3 error returns
10. **Sensible defaults** -- add `#[pyo3(signature)]` with defaults where appropriate

### What Needs Minimal Changes
- QExpr rendering -- already works, may need minor polish
- `__init__.py` -- may add `get_default_session()` and minor UX helpers
- `.pyi` stubs -- update docstrings to match new NumPy-style format

## Open Questions

1. **Example notebook execution environment**
   - What we know: Notebooks need to be pre-executed and committed with output
   - What's unclear: Should we execute on the developer's machine or in CI?
   - Recommendation: Execute locally, commit with output. CI only renders (nbsphinx_execute="never").

2. **QSession default truncation order (UX-04)**
   - What we know: Currently QSession has no default truncation -- each function takes `order` explicitly
   - What's unclear: Should we add a session-wide default truncation?
   - Recommendation: Do NOT change the core API. Instead, document `order=20` as a sensible default in examples and add it as default to a few convenience wrappers if needed.

3. **Type stubs docstring duplication**
   - What we know: `.pyi` stubs have brief one-line docstrings. Rust source will have full NumPy-style.
   - What's unclear: Should stubs mirror the full docstrings?
   - Recommendation: Keep stubs minimal (signatures + one-line description). Full docs come from the compiled module via autodoc.

## Sources

### Primary (HIGH confidence)
- Sphinx 8.2 changelog: https://www.sphinx-doc.org/en/master/changes/8.2.html -- .pyi autodoc support confirmed
- PyO3 function signatures: https://pyo3.rs/v0.25.0/function/signature -- text_signature, docstring handling
- Sphinx napoleon: https://www.sphinx-doc.org/en/master/usage/extensions/napoleon.html -- NumPy docstring support
- Sphinx deployment guide: https://www.sphinx-doc.org/en/master/tutorial/deploying.html -- GitHub Pages workflow
- Scientific Python Development Guide: https://learn.scientific-python.org/development/guides/docs/ -- recommended tools

### Secondary (MEDIUM confidence)
- sphinx-math-dollar: https://www.sympy.org/sphinx-math-dollar/ -- dollar-sign math in RST/docstrings
- Furo theme: https://pradyunsg.me/furo/ -- theme features and configuration
- nbsphinx: https://nbsphinx.readthedocs.io/ -- notebook rendering in Sphinx
- peaceiris/actions-gh-pages: https://github.com/peaceiris/actions-gh-pages -- deployment action

### Tertiary (LOW confidence)
- Sphinx issue #13415: https://github.com/sphinx-doc/sphinx/issues/13415 -- potential autodoc/stub interaction bug (may need workaround)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all tools are established, well-documented, actively maintained
- Architecture: HIGH - patterns verified against existing codebase and official docs
- Pitfalls: HIGH - documented from real PyO3/Sphinx integration experience and issue trackers
- QSeries LaTeX: HIGH - modeled directly on existing QExpr implementation

**Research date:** 2026-02-15
**Valid until:** 2026-05-15 (stable domain, 90-day validity)
