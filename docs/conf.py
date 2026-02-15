"""Sphinx configuration for q-Kangaroo documentation."""

project = "q-Kangaroo"
copyright = "2025, q-Kangaroo Contributors"
author = "q-Kangaroo Contributors"

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

html_theme = "furo"

# Napoleon: NumPy-style docstrings
napoleon_google_docstring = False
napoleon_numpy_docstring = True
napoleon_use_param = True
napoleon_use_rtype = True
napoleon_preprocess_types = True

# MathJax for LaTeX rendering
mathjax3_config = {
    "tex": {
        "inlineMath": [["\\(", "\\)"]],
        "displayMath": [["\\[", "\\]"]],
    }
}

# Intersphinx links to Python stdlib
intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
}

# Autodoc settings
autodoc_member_order = "bysource"
autodoc_typehints = "description"

# nbsphinx: pre-executed notebooks only
nbsphinx_execute = "never"

# Exclude patterns
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]
