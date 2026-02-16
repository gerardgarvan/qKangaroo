# Phase 17: Python API & Documentation - Research

**Researched:** 2026-02-16
**Domain:** PyO3 Python bindings, Sphinx documentation, type stubs for v1.2 algorithms
**Confidence:** HIGH

## Summary

Phase 17 exposes the v1.2 Rust algorithms (q-Gosper, q-Zeilberger, q-Petkovsek, nonterminating proofs, transformation chains) to Python and documents them. The existing infrastructure is mature (73 DSL functions already wired, Sphinx site with 13 API pages, PEP 561 type stubs), so this phase follows well-established patterns rather than introducing new architecture.

The primary design challenge is `prove_nonterminating`, which in Rust takes closure arguments (`lhs_builder: &dyn Fn(i64) -> HypergeometricSeries` and `rhs_builder: &dyn Fn(i64) -> QRat`). Python cannot pass closures to Rust via PyO3, so the Python DSL function must construct these closures on the Rust side from declarative specifications the user passes as data. The same pattern applies to `verify_recurrence_fps` (takes `series_builder: &dyn Fn(i64) -> HypergeometricSeries`).

**Primary recommendation:** Follow the exact patterns of Groups 8-11 in dsl.rs. For closure-based APIs (prove_nonterminating, verify_recurrence_fps), accept declarative "template" parameters from Python that specify how to build the HypergeometricSeries at each n, then construct the closures Rust-side. Return results as Python dicts with string status keys, matching the prove_eta_id pattern.

## Standard Stack

### Core (all already in use -- no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| PyO3 | 0.23 | Rust-Python bindings | Already used in crates/qsym-python |
| maturin | (existing) | Build system | Already configured |
| Sphinx | (existing) | Documentation site | Furo theme + napoleon + autodoc already working |

### Supporting (no changes needed)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| sphinx_math_dollar | (existing) | LaTeX in docs | Already in conf.py |
| sphinx_autodoc_typehints | (existing) | Type hints in docs | Already in conf.py |
| nbsphinx | (existing) | Notebook rendering | Already in conf.py |

### Alternatives Considered
None -- there is no alternative stack to consider. This phase extends an existing, stable infrastructure.

## Architecture Patterns

### Existing File Structure (extend, do not restructure)
```
crates/qsym-python/
  src/
    dsl.rs               # ADD new functions here (Groups 12-13)
    lib.rs               # ADD wrap_pyfunction! registrations here
    convert.rs           # Existing QRat/QInt converters (reuse)
    series.rs            # QSeries pyclass (reuse)
    session.rs           # QSession pyclass (reuse)
  python/q_kangaroo/
    __init__.py          # ADD imports and __all__ entries
    _q_kangaroo.pyi      # ADD type stubs for new functions + missing q_gosper stub
    __init__.pyi         # ADD re-export stubs for new functions + missing q_gosper stub
docs/
  api/
    index.rst            # ADD new page references to toctree
    summation.rst        # NEW -- q-Gosper, q-Zeilberger, q-Petkovsek, WZ verification
    transformations.rst  # NEW -- prove_nonterminating, find_transformation_chain
  index.rst              # UPDATE function count
  conf.py                # No changes needed
```

### Pattern 1: Simple Result-as-Dict (q_zeilberger, q_petkovsek)
**What:** Rust enum result -> Python dict with string status key
**When to use:** For functions returning algebraic result types (enums with variants)
**Example:** (following prove_eta_id pattern from dsl.rs line 2156)
```rust
// Source: crates/qsym-python/src/dsl.rs (existing prove_eta_id pattern)
#[pyfunction]
pub fn q_zeilberger_fn(py: Python<'_>, ...) -> PyResult<PyObject> {
    // ... compute result ...
    let dict = PyDict::new(py);
    match result {
        QZeilbergerResult::Recurrence(zr) => {
            dict.set_item("found", true)?;
            dict.set_item("order", zr.order)?;
            dict.set_item("coefficients", qrat_vec_to_pylist(py, &zr.coefficients)?)?;
            dict.set_item("certificate", format!("{}", zr.certificate))?;
        }
        QZeilbergerResult::NoRecurrence => {
            dict.set_item("found", false)?;
        }
    }
    Ok(dict.into())
}
```

### Pattern 2: Closure-from-Declarative-Spec (prove_nonterminating)
**What:** Python passes declarative templates; Rust constructs closures
**When to use:** For functions that take `Fn(i64) -> T` closures
**Critical design decision:** The Python user specifies the LHS series as a "template" where certain parameter powers are expressed as functions of n, and the Rust wrapper builds the `Fn(i64) -> HypergeometricSeries` closure internally.
**Example approach:**
```rust
/// Prove a nonterminating q-hypergeometric identity (Chen-Hou-Mu method).
///
/// The LHS is a template hypergeometric series where one upper parameter is q^{-n}.
/// The user specifies:
///   - upper_fixed: list of (num, den, power) for n-independent upper params
///   - upper_n_offset: power offset so the n-dependent param = q^{offset - n}
///   - lower: list of (num, den, power) for lower params
///   - z_base_pow: base argument power; argument = q^{z_base_pow + n}
///   - rhs_params: specification of how to compute the RHS at each n
///     (e.g., list of Pochhammer quotients evaluated at concrete q)
///
/// This constructs the closures internally and calls prove_nonterminating.
#[pyfunction]
#[pyo3(signature = (upper_fixed, upper_n_offset, lower, z_base_pow, rhs_numer_bases, rhs_denom_bases, q_num, q_den, n_test, max_order))]
pub fn prove_nonterminating_fn(py: Python<'_>, ...) -> PyResult<PyObject> {
    // Build lhs_builder closure from template params
    let lhs_builder = move |n: i64| -> HypergeometricSeries {
        let mut upper = upper_fixed_qm.clone();
        upper.push(QMonomial::q_power(upper_n_offset - n));
        HypergeometricSeries { upper, lower: lower_qm.clone(), argument: QMonomial::q_power(z_base_pow + n) }
    };
    // Build rhs_builder closure from Pochhammer spec
    let rhs_builder = move |n: i64| -> QRat { ... };
    // Call core function
    let result = prove_nonterminating(&lhs_builder, &rhs_builder, &q_val, n_test, max_order);
    // Convert to Python dict
}
```

### Pattern 3: HypergeometricSeries Input (q_zeilberger, verify_wz)
**What:** Python specifies the series as (upper, lower, z) tuples, plus n_val and q_val
**When to use:** For functions operating on a single concrete HypergeometricSeries
**Example:**
```rust
#[pyfunction]
#[pyo3(signature = (upper, lower, z_num, z_den, z_pow, n_val, q_num, q_den, max_order))]
pub fn q_zeilberger_fn(py: Python<'_>,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64, z_den: i64, z_pow: i64,
    n_val: i64, q_num: i64, q_den: i64,
    max_order: usize,
) -> PyResult<PyObject> {
    let series = HypergeometricSeries {
        upper: parse_qmonomials(upper),
        lower: parse_qmonomials(lower),
        argument: QMonomial::new(QRat::from((z_num, z_den)), z_pow),
    };
    let q_val = QRat::from((q_num, q_den));
    let (n_indices, n_in_arg) = detect_n_params(&series, n_val, &q_val);
    let result = q_zeilberger(&series, n_val, &q_val, max_order, &n_indices, n_in_arg);
    // ... convert result to dict ...
}
```

### Anti-Patterns to Avoid
- **Exposing Rust closures to Python:** PyO3 cannot pass Python callables as Rust `dyn Fn` trait objects across the FFI boundary (without `pyo3-asyncio` or callback patterns that add complexity). Use declarative specs instead.
- **Exposing QRatRationalFunc as a pyclass:** The certificate from Zeilberger is a rational function over QRatPoly. Rather than wrapping these types, return their string representation (Display) and numerator/denominator strings. This matches how q_gosper_fn already handles it.
- **Adding new Rust modules:** All new Python DSL functions go in the existing `dsl.rs` file as additional `#[pyfunction]` items. No new Rust source files.
- **Breaking naming conventions:** Existing pattern: Rust `q_gosper_fn` -> Python `q_gosper` via `#[pyo3(name = "...")]` or via `__init__.py` alias. Follow the same for `q_zeilberger_fn`, `q_petkovsek_fn`, etc.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| QRat -> Python | Custom conversion | `qrat_to_python()` from convert.rs | Already handles arbitrary precision via string |
| QRat[] -> Python | Manual iteration | `qrat_vec_to_pylist()` from dsl.rs | Already handles Vec<QRat> -> Python list[Fraction] |
| QMonomial parsing | New parser | `parse_qmonomials()` from dsl.rs | Already handles Vec<(i64,i64,i64)> -> Vec<QMonomial> |
| Sphinx page layout | Custom RST | Follow existing .rst files | mock_theta.rst and hypergeometric.rst are templates |
| Type stubs | Auto-generation | Manual stubs matching dsl.rs patterns | _q_kangaroo.pyi has 469 lines of hand-maintained stubs |

## Common Pitfalls

### Pitfall 1: prove_nonterminating Closure Design
**What goes wrong:** Attempting to accept Python callables as Rust closures, or trying to use PyO3 callbacks
**Why it happens:** The Rust API takes `&dyn Fn(i64) -> HypergeometricSeries` which cannot be directly mapped from Python
**How to avoid:** Accept declarative template parameters from Python. The template specifies: (a) which upper parameter index is n-dependent and what its power formula is, (b) how the argument depends on n, (c) how to compute the RHS Pochhammer product at concrete q for each n. Build closures entirely on the Rust side.
**Warning signs:** If the Python function signature includes `Callable` or `lambda`, the design is wrong.

### Pitfall 2: Missing q_gosper Type Stubs
**What goes wrong:** q_gosper was added to __init__.py during Phase 14 UAT but never added to _q_kangaroo.pyi or __init__.pyi
**Why it happens:** The type stubs were written in Phase 10 and q_gosper was added later
**How to avoid:** Add q_gosper_fn stub to _q_kangaroo.pyi and q_gosper re-export stub to __init__.pyi as part of this phase
**Warning signs:** IDE/mypy warnings about unknown function q_gosper

### Pitfall 3: Inconsistent Function Count in Docs
**What goes wrong:** docs/index.rst says "73 functions" and api/index.rst says "73 functions organized in 10 functional groups" but we are adding ~6 new functions
**Why it happens:** The counts were correct at Phase 12 completion
**How to avoid:** Update the function count and group count in index.rst and api/index.rst
**Warning signs:** Documentation claiming wrong function count

### Pitfall 4: verify_recurrence_fps Closure Pattern
**What goes wrong:** Same closure issue as prove_nonterminating
**Why it happens:** verify_recurrence_fps takes `series_builder: &dyn Fn(i64) -> HypergeometricSeries`
**How to avoid:** Same declarative template approach. User provides the series template with n-dependent parts specified as offsets. Or: since this is primarily a verification utility, consider whether it needs a Python binding at all -- the user can verify recurrences by computing S(n) values from Python and checking manually. If exposed, use same template pattern.
**Warning signs:** Same as Pitfall 1

### Pitfall 5: lib.rs Registration Order
**What goes wrong:** Forgetting to register new pyfunction in lib.rs _q_kangaroo module
**Why it happens:** Three files must be updated in sync: dsl.rs (function), lib.rs (registration), __init__.py (re-export)
**How to avoid:** Checklist: for each new function, update (1) dsl.rs, (2) lib.rs wrap_pyfunction!, (3) __init__.py import + __all__, (4) _q_kangaroo.pyi, (5) __init__.pyi

### Pitfall 6: detect_n_params Auto-Detection Limitation
**What goes wrong:** q_zeilberger relies on detect_n_params which is a heuristic; for non-standard series, users need to provide n_param_indices manually
**Why it happens:** The heuristic looks for q^{-n} in upper params; non-standard parameterizations may not match
**How to avoid:** The Python wrapper should auto-detect by default but allow optional `n_param_indices` and `n_is_in_argument` override parameters
**Warning signs:** q_zeilberger returning NoRecurrence for series that should have one

## Code Examples

### Example 1: q_zeilberger Python DSL Function
```rust
// Pattern: Group 8 (phi) + Group 11 (q_gosper_fn) combined
#[pyfunction]
#[pyo3(name = "q_zeilberger", signature = (upper, lower, z_num, z_den, z_pow, n_val, q_num, q_den, max_order, n_param_indices=None, n_is_in_argument=None))]
pub fn q_zeilberger_fn(
    py: Python<'_>,
    upper: Vec<(i64, i64, i64)>,
    lower: Vec<(i64, i64, i64)>,
    z_num: i64, z_den: i64, z_pow: i64,
    n_val: i64,
    q_num: i64, q_den: i64,
    max_order: usize,
    n_param_indices: Option<Vec<usize>>,
    n_is_in_argument: Option<bool>,
) -> PyResult<PyObject> {
    let series = HypergeometricSeries {
        upper: parse_qmonomials(upper),
        lower: parse_qmonomials(lower),
        argument: QMonomial::new(QRat::from((z_num, z_den)), z_pow),
    };
    let q_val = QRat::from((q_num, q_den));

    // Auto-detect or use provided
    let (indices, in_arg) = match (n_param_indices, n_is_in_argument) {
        (Some(idx), Some(flag)) => (idx, flag),
        _ => detect_n_params(&series, n_val, &q_val),
    };

    let result = q_zeilberger(&series, n_val, &q_val, max_order, &indices, in_arg);

    let dict = PyDict::new(py);
    match result {
        QZeilbergerResult::Recurrence(zr) => {
            dict.set_item("found", true)?;
            dict.set_item("order", zr.order)?;
            dict.set_item("coefficients", qrat_vec_to_pylist(py, &zr.coefficients)?)?;
            dict.set_item("certificate", format!("{}", zr.certificate))?;
            dict.set_item("numer", format!("{}", zr.certificate.numer))?;
            dict.set_item("denom", format!("{}", zr.certificate.denom))?;
        }
        QZeilbergerResult::NoRecurrence => {
            dict.set_item("found", false)?;
        }
    }
    Ok(dict.into())
}
```

### Example 2: q_petkovsek Python DSL Function
```rust
#[pyfunction]
#[pyo3(name = "q_petkovsek", signature = (coefficients, q_num, q_den))]
pub fn q_petkovsek_fn(
    py: Python<'_>,
    coefficients: Vec<(i64, i64)>,  // list of (numer, denom) Fraction pairs
    q_num: i64,
    q_den: i64,
) -> PyResult<PyObject> {
    let coeffs: Vec<QRat> = coefficients.iter()
        .map(|(n, d)| QRat::from((*n, *d)))
        .collect();
    let q_val = QRat::from((q_num, q_den));

    let results = q_petkovsek(&coeffs, &q_val);

    let items: Vec<PyObject> = results.iter().map(|r| {
        let dict = PyDict::new(py);
        dict.set_item("ratio", qrat_to_python(py, &r.ratio).unwrap()).unwrap();
        match &r.closed_form {
            Some(cf) => {
                dict.set_item("has_closed_form", true).unwrap();
                dict.set_item("scalar", qrat_to_python(py, &cf.scalar).unwrap()).unwrap();
                dict.set_item("q_power_coeff", cf.q_power_coeff).unwrap();
                // numer/denom factors as lists of (num, den, power) tuples
                let nf: Vec<(String, i64)> = cf.numer_factors.iter()
                    .map(|m| (format!("{}", m.coeff), m.power)).collect();
                dict.set_item("numer_factors", nf).unwrap();
                let df: Vec<(String, i64)> = cf.denom_factors.iter()
                    .map(|m| (format!("{}", m.coeff), m.power)).collect();
                dict.set_item("denom_factors", df).unwrap();
            }
            None => {
                dict.set_item("has_closed_form", false).unwrap();
            }
        }
        dict.into()
    }).collect();
    Ok(PyList::new(py, &items)?.into())
}
```

### Example 3: prove_nonterminating with Template Parameters
```rust
/// Prove nonterminating identity via Chen-Hou-Mu.
///
/// LHS template: _r phi_s (upper_fixed..., q^{n_param_offset - n}; lower...; q, q^{z_pow_offset + n})
/// RHS: product of Pochhammer factors at concrete q.
///
/// rhs_numer_bases/rhs_denom_bases: each is a q-power (int) whose
/// Pochhammer (q^base; q)_n enters the RHS numerator/denominator.
#[pyfunction]
#[pyo3(name = "prove_nonterminating", signature = (
    upper_fixed, n_param_offset, lower, z_pow_offset,
    rhs_numer_bases, rhs_denom_bases,
    q_num, q_den, n_test, max_order
))]
pub fn prove_nonterminating_fn(
    py: Python<'_>,
    upper_fixed: Vec<(i64, i64, i64)>,
    n_param_offset: i64,       // n-dependent upper param = q^{offset - n}
    lower: Vec<(i64, i64, i64)>,
    z_pow_offset: i64,         // argument = q^{offset + n}
    rhs_numer_bases: Vec<i64>, // RHS = prod (q^base;q)_n / prod (q^base;q)_n
    rhs_denom_bases: Vec<i64>,
    q_num: i64, q_den: i64,
    n_test: i64,
    max_order: usize,
) -> PyResult<PyObject> {
    let upper_fixed_qm = parse_qmonomials(upper_fixed);
    let lower_qm = parse_qmonomials(lower);
    let q_val = QRat::from((q_num, q_den));

    let lhs_builder = |n: i64| -> HypergeometricSeries {
        let mut upper = upper_fixed_qm.clone();
        upper.push(QMonomial::q_power(n_param_offset - n));
        HypergeometricSeries {
            upper,
            lower: lower_qm.clone(),
            argument: QMonomial::q_power(z_pow_offset + n),
        }
    };

    let rhs_builder = |n: i64| -> QRat {
        if n == 0 { return QRat::one(); }
        let mut numer = QRat::one();
        for &base in &rhs_numer_bases {
            numer = &numer * &pochhammer_scalar_val(&q_val, base, n);
        }
        let mut denom = QRat::one();
        for &base in &rhs_denom_bases {
            denom = &denom * &pochhammer_scalar_val(&q_val, base, n);
        }
        &numer / &denom
    };

    let result = prove_nonterminating(&lhs_builder, &rhs_builder, &q_val, n_test, max_order);

    let dict = PyDict::new(py);
    match result {
        NonterminatingProofResult::Proved { recurrence_order, recurrence_coefficients, initial_conditions_checked } => {
            dict.set_item("proved", true)?;
            dict.set_item("recurrence_order", recurrence_order)?;
            dict.set_item("coefficients", qrat_vec_to_pylist(py, &recurrence_coefficients)?)?;
            dict.set_item("initial_conditions_checked", initial_conditions_checked)?;
        }
        NonterminatingProofResult::Failed { reason } => {
            dict.set_item("proved", false)?;
            dict.set_item("reason", reason)?;
        }
    }
    Ok(dict.into())
}
```

### Example 4: find_transformation_chain Python DSL Function
```rust
#[pyfunction]
#[pyo3(name = "find_transformation_chain", signature = (session, source_upper, source_lower, source_z_num, source_z_den, source_z_pow, target_upper, target_lower, target_z_num, target_z_den, target_z_pow, max_depth, order))]
pub fn find_transformation_chain_fn(
    py: Python<'_>,
    session: &QSession,
    source_upper: Vec<(i64, i64, i64)>,
    source_lower: Vec<(i64, i64, i64)>,
    source_z_num: i64, source_z_den: i64, source_z_pow: i64,
    target_upper: Vec<(i64, i64, i64)>,
    target_lower: Vec<(i64, i64, i64)>,
    target_z_num: i64, target_z_den: i64, target_z_pow: i64,
    max_depth: usize,
    order: i64,
) -> PyResult<PyObject> {
    let mut inner = session.inner.lock().map_err(|e| ...)?;
    let var = inner.get_or_create_symbol_id("q");
    drop(inner);

    let source = HypergeometricSeries {
        upper: parse_qmonomials(source_upper),
        lower: parse_qmonomials(source_lower),
        argument: QMonomial::new(QRat::from((source_z_num, source_z_den)), source_z_pow),
    };
    let target = HypergeometricSeries {
        upper: parse_qmonomials(target_upper),
        lower: parse_qmonomials(target_lower),
        argument: QMonomial::new(QRat::from((target_z_num, target_z_den)), target_z_pow),
    };

    let result = find_transformation_chain(&source, &target, max_depth, var, order);
    let dict = PyDict::new(py);
    match result {
        TransformationChainResult::Found { steps, total_prefactor } => {
            dict.set_item("found", true)?;
            // Convert steps to list of dicts
            let step_dicts: Vec<PyObject> = steps.iter().map(|s| {
                let d = PyDict::new(py);
                d.set_item("name", &s.name).unwrap();
                d.set_item("prefactor", QSeries { fps: s.step_prefactor.clone() }).unwrap();
                d.into()
            }).collect();
            dict.set_item("steps", PyList::new(py, &step_dicts)?)?;
            dict.set_item("total_prefactor", QSeries { fps: total_prefactor })?;
        }
        TransformationChainResult::NotFound { max_depth } => {
            dict.set_item("found", false)?;
            dict.set_item("max_depth", max_depth)?;
        }
    }
    Ok(dict.into())
}
```

### Example 5: Sphinx RST Page (following existing pattern)
```rst
Algorithmic Summation
=====================

The q-Gosper algorithm performs indefinite q-hypergeometric summation.
The q-Zeilberger algorithm extends this via creative telescoping to find
recurrences for definite sums $S(n) = \sum_k F(n,k)$. The q-Petkovsek
algorithm solves the resulting recurrences for closed-form solutions.

.. autofunction:: q_kangaroo.q_gosper

.. autofunction:: q_kangaroo.q_zeilberger

.. autofunction:: q_kangaroo.verify_wz

.. autofunction:: q_kangaroo.q_petkovsek

.. autofunction:: q_kangaroo.prove_nonterminating

.. autofunction:: q_kangaroo.find_transformation_chain
```

### Example 6: NumPy-style Docstring (following existing pattern)
```rust
/// Run the q-Zeilberger creative telescoping algorithm for definite q-hypergeometric summation.
///
/// Given a q-hypergeometric summand $F(n,k)$ defined by upper/lower parameters and argument,
/// finds a linear recurrence $c_0 S(n) + c_1 S(n+1) + \cdots + c_d S(n+d) = 0$ for the
/// definite sum $S(n) = \sum_k F(n,k)$, along with a WZ proof certificate.
///
/// Parameters
/// ----------
/// upper : list[tuple[int, int, int]]
///     Upper parameters. Each is ``(num, den, power)`` representing $(num/den) \cdot q^{power}$.
/// lower : list[tuple[int, int, int]]
///     Lower parameters, same tuple format.
/// z_num : int
///     Numerator of the argument $z$ coefficient.
/// z_den : int
///     Denominator of the argument $z$ coefficient.
/// z_pow : int
///     Power of $q$ in the argument $z$.
/// n_val : int
///     Concrete value of the summation parameter $n$.
/// q_num : int
///     Numerator of the concrete $q$ value.
/// q_den : int
///     Denominator of the concrete $q$ value.
/// max_order : int
///     Maximum recurrence order to search.
/// n_param_indices : list[int] or None, optional
///     Indices of upper parameters that depend on $n$. Auto-detected if ``None``.
/// n_is_in_argument : bool or None, optional
///     Whether the argument $z$ depends on $n$. Auto-detected if ``None``.
///
/// Returns
/// -------
/// dict
///     If found: ``{"found": True, "order": d, "coefficients": [...], "certificate": "...", "numer": "...", "denom": "..."}``.
///     If not found: ``{"found": False}``.
///
/// Examples
/// --------
/// >>> from q_kangaroo import q_zeilberger
/// >>> # q-Vandermonde: 2phi1(q^{-5}, q^2; q^3; q, q^4) at q=2
/// >>> result = q_zeilberger([(1,1,-5), (1,1,2)], [(1,1,3)], 1, 1, 4, 5, 2, 1, 3)
/// >>> result["found"]
/// True
/// >>> result["order"]
/// 1
///
/// See Also
/// --------
/// q_gosper : Indefinite q-hypergeometric summation.
/// verify_wz : Independent WZ certificate verification.
/// q_petkovsek : Solve recurrences for closed-form solutions.
```

## Inventory of New Functions

### Functions to Add (6 new Python-facing functions)

| # | Python Name | Rust DSL Name | Rust Core Function | Group | Complexity |
|---|-------------|---------------|-------------------|-------|------------|
| 1 | `q_zeilberger` | `q_zeilberger_fn` | `qseries::q_zeilberger` | 12 | Medium (HyperSeries + auto-detect) |
| 2 | `verify_wz` | `verify_wz_fn` | `qseries::verify_wz_certificate` | 12 | Medium (same param pattern as q_zeilberger) |
| 3 | `q_petkovsek` | `q_petkovsek_fn` | `qseries::q_petkovsek` | 12 | Low (takes coefficient list + q) |
| 4 | `prove_nonterminating` | `prove_nonterminating_fn` | `qseries::prove_nonterminating` | 13 | HIGH (closure template design) |
| 5 | `find_transformation_chain` | `find_transformation_chain_fn` | `qseries::find_transformation_chain` | 13 | Medium (two series + session) |

Note: `q_gosper` already exists in `__init__.py` and `dsl.rs` but is MISSING from both `.pyi` stub files. Must be added.

### Files to Modify (checklist per function)

For each function, ALL of these must be updated:
1. `crates/qsym-python/src/dsl.rs` -- add `#[pyfunction]` with docstring
2. `crates/qsym-python/src/lib.rs` -- add `m.add_function(wrap_pyfunction!(dsl::xxx, m)?)?;`
3. `crates/qsym-python/python/q_kangaroo/__init__.py` -- add import + `__all__` entry
4. `crates/qsym-python/python/q_kangaroo/_q_kangaroo.pyi` -- add type stub
5. `crates/qsym-python/python/q_kangaroo/__init__.pyi` -- add re-export stub

Plus documentation:
6. `docs/api/summation.rst` -- new page for all 5 algorithmic functions
7. `docs/api/index.rst` -- add summation to toctree + update counts
8. `docs/index.rst` -- update function count

### Existing Gap: q_gosper Type Stubs

The q_gosper_fn function (from Phase 14 UAT) is registered in lib.rs, has a docstring in dsl.rs, and is re-exported in __init__.py as `q_gosper`. However:
- `_q_kangaroo.pyi` does NOT have a stub for `q_gosper_fn`
- `__init__.pyi` does NOT have a re-export for `q_gosper`
These must be added as part of this phase.

## prove_nonterminating Design Deep-Dive

### The Challenge

The Rust signature is:
```rust
pub fn prove_nonterminating(
    lhs_builder: &dyn Fn(i64) -> HypergeometricSeries,
    rhs_builder: &dyn Fn(i64) -> QRat,
    q_val: &QRat,
    n_test: i64,
    max_order: usize,
) -> NonterminatingProofResult
```

Both `lhs_builder` and `rhs_builder` are closures. PyO3 cannot directly accept Python functions for use as Rust closures in this way without significant complexity (GIL management, Python object lifetimes).

### Recommended Solution: Declarative Template

The standard pattern for nonterminating identities is:
- **LHS:** `_r phi_s(a_1,...,a_{r-1}, q^{-n}; b_1,...,b_s; q, z_n)` where `z_n = c * q^{f(n)}`
- **RHS:** A ratio of Pochhammer products evaluated at concrete q: `prod (q^{a_i};q)_n / prod (q^{b_j};q)_n`

This covers q-Gauss, q-Vandermonde, q-Kummer, and all the standard test cases in the Rust tests.

The Python function signature should be:
```python
def prove_nonterminating(
    upper_fixed: list[tuple[int, int, int]],  # n-independent upper params
    n_param_offset: int,                       # n-dep upper = q^{offset - n}
    lower: list[tuple[int, int, int]],         # lower params
    z_pow_offset: int,                          # z = q^{offset + n}
    rhs_numer_bases: list[int],                # RHS numer: prod (q^base;q)_n
    rhs_denom_bases: list[int],                # RHS denom: prod (q^base;q)_n
    q_num: int, q_den: int,                    # concrete q
    n_test: int,                               # test n value (>= 5 recommended)
    max_order: int,                            # max recurrence order
) -> dict:
    """..."""
```

This is expressive enough for all standard use cases and the Rust side constructs the closures from these parameters.

### Helper Needed

A `pochhammer_scalar_val(q_val, base_power, n)` helper that computes `(q^base; q)_n` at concrete q, matching the pattern in the Rust tests. This is just:
```rust
fn pochhammer_scalar_val(q_val: &QRat, base_power: i64, n: i64) -> QRat {
    if n <= 0 { return QRat::one(); }
    let base = qrat_pow_i64(q_val, base_power);
    let mut result = QRat::one();
    for k in 0..n {
        let qk = qrat_pow_i64(q_val, k);
        let factor = &QRat::one() - &(&base * &qk);
        result = &result * &factor;
    }
    result
}
```

This is a private helper in dsl.rs, not exposed to Python.

## verify_recurrence_fps Exposure Decision

`verify_recurrence_fps` takes `series_builder: &dyn Fn(i64) -> HypergeometricSeries` -- the same closure problem. It is NOT listed in the requirements (API-01 through API-04). Its use case is cross-verification, which can be done from Python by:
1. Running q_zeilberger at multiple n values
2. Comparing the recurrence coefficients

**Recommendation:** Do NOT expose `verify_recurrence_fps` to Python. It is not required and the closure pattern makes it complex. The user can verify recurrences by calling `q_zeilberger` at multiple n values from Python.

Similarly, `detect_n_params` does not need a separate Python binding -- it is called internally by `q_zeilberger_fn`.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 73 DSL functions | ~79 DSL functions (73 + 5 new + 1 stub fix) | Phase 17 | Update all counts |
| 10 function groups | 12 function groups (add Algorithmic Summation, Identity Transformation) | Phase 17 | Update docs |
| 13 API doc pages | 14 API doc pages (add summation.rst) | Phase 17 | Update api/index.rst toctree |

## New Rust Imports Needed in dsl.rs

```rust
use qsym_core::qseries::{
    // Existing:
    self, QMonomial, PochhammerOrder, HypergeometricSeries, SummationResult,
    BaileyDatabase, bailey_lemma, bailey_chain, weak_bailey_lemma, bailey_discover,
    QGosperResult,
    // NEW:
    q_zeilberger, QZeilbergerResult, ZeilbergerResult, detect_n_params,
    verify_wz_certificate,
    q_petkovsek, QPetkovsekResult, ClosedForm,
    prove_nonterminating, NonterminatingProofResult,
    find_transformation_chain, TransformationChainResult, TransformationStep,
};
```

And for `verify_wz_certificate`, we need `QRatRationalFunc` from the poly module:
```rust
use qsym_core::poly::QRatRationalFunc;
```

Note: `QRatRationalFunc` is used in `ZeilbergerResult.certificate` and in `verify_wz_certificate` parameters. For the Python binding, we convert to string (Display), so we do NOT need to make it a pyclass. But we need to import it for the function signatures.

For `verify_wz`, the certificate input from Python would be the numer/denom polynomial strings -- but parsing polynomials from strings is complex. A simpler approach: the Python user gets the certificate from `q_zeilberger` (as strings) and passes it back. We could store the raw `ZeilbergerResult` in a Python-side cache, or more practically, `verify_wz` could accept the Zeilberger result dict and re-run internally. However, the cleanest design is:

**verify_wz_fn takes the same series params as q_zeilberger plus coefficients and runs the internal verify_wz_certificate.** The user workflow is:
1. Call `q_zeilberger(...)` to get `{"found": True, "order": 1, "coefficients": [...], ...}`
2. Call `verify_wz(same_series_params, coefficients, max_k)` to independently verify

For the certificate parameter, since the user cannot construct `QRatRationalFunc` from Python, `verify_wz_fn` should re-run q_zeilberger internally to get the certificate, then verify it. Or alternatively, store the result and pass an opaque handle. The simplest approach: verify_wz_fn runs both q_zeilberger AND verify_wz_certificate internally, taking only the series params + q + n + max_k.

## Open Questions

1. **prove_nonterminating template expressiveness**
   - What we know: The standard Pochhammer-ratio RHS pattern covers q-Gauss, q-Vandermonde, q-Kummer
   - What's unclear: Are there nonterminating identities where the RHS is not a simple Pochhammer ratio? (e.g., Heine transforms have infinite product prefactors)
   - Recommendation: Start with Pochhammer-ratio template. If users need more, a future version could accept the RHS as a Python-side FPS computed for each n.

2. **verify_wz certificate round-trip**
   - What we know: The certificate is a QRatRationalFunc which cannot easily be passed from Python
   - What's unclear: Whether users need to verify externally-supplied certificates or only verify the ones q_zeilberger found
   - Recommendation: Have verify_wz re-derive the certificate internally via q_zeilberger, then verify. The user passes series params + q + n_val + max_order + max_k. This is simpler than certificate serialization.

3. **find_transformation_chain step details**
   - What we know: Each TransformationStep has name, result_series, step_prefactor
   - What's unclear: Whether to expose result_series as QSeries or as (upper, lower, z) tuples
   - Recommendation: Return step_prefactor as QSeries and result_series as a dict with upper/lower/z tuples, since HypergeometricSeries is not a pyclass.

## Sources

### Primary (HIGH confidence)
- `crates/qsym-python/src/dsl.rs` -- 3286 lines, 73 existing DSL functions with full docstrings
- `crates/qsym-python/src/lib.rs` -- 128 lines, module registration pattern
- `crates/qsym-python/python/q_kangaroo/__init__.py` -- 147 lines, re-export pattern
- `crates/qsym-python/python/q_kangaroo/_q_kangaroo.pyi` -- 469 lines, type stub pattern
- `crates/qsym-python/python/q_kangaroo/__init__.pyi` -- 127 lines, re-export stub pattern
- `crates/qsym-core/src/qseries/zeilberger.rs` -- Public API signatures for q_zeilberger, verify_wz_certificate, detect_n_params
- `crates/qsym-core/src/qseries/petkovsek.rs` -- Public API signatures for q_petkovsek, QPetkovsekResult, ClosedForm
- `crates/qsym-core/src/qseries/nonterminating.rs` -- Public API signatures for prove_nonterminating, NonterminatingProofResult
- `crates/qsym-core/src/qseries/hypergeometric.rs` -- TransformationStep, TransformationChainResult, find_transformation_chain
- `docs/conf.py` -- Sphinx configuration (autodoc + napoleon + mathjax)
- `docs/api/*.rst` -- 13 existing API reference pages

### Secondary (MEDIUM confidence)
- Memory/MEMORY.md -- Phase history and architecture decisions

### Tertiary (LOW confidence)
- None -- all findings are based on direct source code examination

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, all patterns established
- Architecture: HIGH -- extending existing patterns, no new modules
- Pitfalls: HIGH -- identified from direct code examination of existing gaps
- prove_nonterminating design: MEDIUM -- the template approach is sound but edge cases (non-Pochhammer RHS) may need future extension

**Research date:** 2026-02-16
**Valid until:** Indefinite (internal codebase research, no external dependency changes)
