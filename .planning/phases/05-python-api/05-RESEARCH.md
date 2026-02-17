# Phase 5: Python API - Research

**Researched:** 2026-02-13
**Domain:** PyO3 Python bindings for Rust symbolic computation engine with GMP dependency
**Confidence:** MEDIUM (PyO3 patterns well-documented; GMP+PyO3+Windows is underexplored territory)

## Summary

Phase 5 wraps the existing qsym-core Rust crate (arena-based symbolic expression engine with 379 tests, GMP-backed arbitrary precision via rug) in Python bindings using PyO3 and maturin. The core challenge is architectural: ExprArena owns all expression data and ExprRef is a u32 index into it, so the Python layer must manage a shared Session that owns the arena and route all operations through it. The GMP dependency (gmp-mpfr-sys with `use-system-libs`) adds Windows build complexity since GMP must be compiled from source as part of the cdylib build.

The recommended architecture is a `#[pyclass(frozen)]` QSession wrapping `Arc<Mutex<SessionInner>>` where SessionInner holds ExprArena plus a SymbolRegistry. QExpr objects hold both the ExprRef (u32) and a clone of the Arc to keep the session alive. This avoids lifetime issues (PyO3 forbids lifetime parameters on pyclass) and lets Python's GC naturally clean up sessions. The rug types (QInt/QRat) should NOT be exposed to Python directly -- instead, convert to/from Python int and fractions.Fraction at the boundary using num-bigint and manual conversion.

**Primary recommendation:** Use maturin 1.11+ with PyO3 0.23+ in a new `crates/qsym-python/` crate (cdylib) that depends on qsym-core. Wrap all arena operations behind a frozen QSession class with Mutex-protected interior mutability. Convert rug integers to Python ints via byte-level extraction (not via num-bigint -- rug is not compatible with num-bigint traits). Implement `_repr_latex_()` and `__repr__` on QExpr by delegating to the existing Rust render module.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| pyo3 | 0.23+ (target 0.28) | Rust-Python FFI bindings | Official, most-used Rust-Python bridge |
| maturin | 1.11+ | Build system for PyO3 wheels | Official PyO3 companion, handles cdylib+wheel packaging |
| gmp-mpfr-sys | 1.6.8 | GMP C library (bundled build) | Already used by qsym-core via rug; `use-system-libs` feature |
| rug | 1.28 | Arbitrary precision arithmetic | Already used by qsym-core |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| num-bigint | 0.4 | BigInt<->Python int conversion | PyO3 feature `num-bigint` for boundary conversions |
| num-rational | 0.4 | Ratio<->Python Fraction | If exposing rational numbers to Python |
| parking_lot | (optional) | Faster Mutex | If std::sync::Mutex becomes a bottleneck |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| maturin | setuptools-rust | maturin is simpler, purpose-built for PyO3; setuptools-rust offers more customization |
| Arc<Mutex<>> | GIL-protected RefCell | Arc<Mutex> works with free-threaded Python (3.13t+); RefCell does not |
| Manual rug->PyInt | gmpy2 interop | gmpy2 already wraps GMP for Python but adds a dependency and API mismatch |

**Installation (development):**
```bash
pip install maturin
cd crates/qsym-python
maturin develop --release
```

## Architecture Patterns

### Recommended Project Structure
```
crates/
  qsym-core/          # Existing: pure Rust library (lib crate)
    Cargo.toml
    src/
  qsym-python/        # NEW: PyO3 bindings (cdylib crate)
    Cargo.toml         # [lib] crate-type = ["cdylib"]
    pyproject.toml     # maturin build config
    src/
      lib.rs           # #[pymodule] entry point
      session.rs       # QSession pyclass
      expr.rs          # QExpr pyclass
      series.rs        # QSeries pyclass (FormalPowerSeries wrapper)
      dsl.rs           # Python DSL functions (qpoch, theta3, etc.)
      convert.rs       # rug <-> Python type conversions
    python/
      qsymbolic/       # Pure Python layer
        __init__.py    # Re-exports, convenience imports
        display.py     # Jupyter display helpers (optional)
```

### Pattern 1: Frozen Session with Mutex Interior
**What:** The QSession pyclass owns the arena via Arc<Mutex<SessionInner>>. All mutations go through the lock.
**When to use:** Always -- this is the core ownership pattern.
**Example:**
```rust
// Source: PyO3 thread-safety guide + project-specific design
use std::sync::{Arc, Mutex};
use pyo3::prelude::*;
use qsym_core::{ExprArena, ExprRef};

struct SessionInner {
    arena: ExprArena,
    // Default variable for series operations
    q_var: Option<ExprRef>,
}

#[pyclass(frozen)]
struct QSession {
    inner: Arc<Mutex<SessionInner>>,
}

#[pymethods]
impl QSession {
    #[new]
    fn new() -> Self {
        let inner = SessionInner {
            arena: ExprArena::new(),
            q_var: None,
        };
        QSession {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    fn symbol(&self, name: &str) -> QExpr {
        let mut session = self.inner.lock().expect("session lock poisoned");
        let expr_ref = session.arena.intern_symbol(name);
        QExpr {
            session: Arc::clone(&self.inner),
            expr_ref,
        }
    }
}
```

### Pattern 2: QExpr as Opaque Handle with Session Back-Reference
**What:** Each QExpr holds a clone of the session Arc plus its ExprRef (u32). Operations lock the session, perform the operation, and return a new QExpr.
**When to use:** For all expression objects returned to Python.
**Example:**
```rust
#[pyclass(frozen)]
struct QExpr {
    session: Arc<Mutex<SessionInner>>,
    expr_ref: ExprRef,
}

#[pymethods]
impl QExpr {
    fn __repr__(&self) -> String {
        let session = self.session.lock().expect("lock");
        format!("{}", session.arena.display(self.expr_ref))
    }

    fn _repr_latex_(&self) -> String {
        let session = self.session.lock().expect("lock");
        let latex = qsym_core::render::to_latex(&session.arena, self.expr_ref);
        format!("${}$", latex)
    }

    fn __add__(&self, other: &QExpr) -> PyResult<QExpr> {
        let mut session = self.session.lock().expect("lock");
        let sum = qsym_core::canonical::make_add(
            &mut session.arena,
            vec![self.expr_ref, other.expr_ref],
        );
        Ok(QExpr {
            session: Arc::clone(&self.session),
            expr_ref: sum,
        })
    }
}
```

### Pattern 3: DSL Functions as Module-Level Callables
**What:** Free functions like `qpoch()`, `theta3()` that take a session and parameters.
**When to use:** For the user-facing API surface.
**Example:**
```rust
#[pyfunction]
fn qpoch(session: &QSession, a: &QExpr, q: &QExpr, n: &QExpr) -> QExpr {
    let mut sess = session.inner.lock().expect("lock");
    let expr_ref = qsym_core::canonical::make_qpochhammer(
        &mut sess.arena,
        a.expr_ref,
        q.expr_ref,
        n.expr_ref,
    );
    QExpr {
        session: Arc::clone(&session.inner),
        expr_ref,
    }
}
```

### Pattern 4: FormalPowerSeries as QSeries with Mapping Protocol
**What:** Wrap FormalPowerSeries in a pyclass with `__getitem__` for coefficient access.
**When to use:** When returning computed series to Python.
**Example:**
```rust
#[pyclass(frozen, mapping)]
struct QSeries {
    session: Arc<Mutex<SessionInner>>,
    // FPS stored outside session since it's self-contained
    fps: FormalPowerSeries,
}

#[pymethods]
impl QSeries {
    fn __getitem__(&self, key: i64) -> PyResult<PyObject> {
        Python::attach(|py| {
            let coeff = self.fps.coeff(key);
            // Convert QRat to Python Fraction
            qrat_to_python(py, &coeff)
        })
    }

    fn __len__(&self) -> usize {
        self.fps.num_nonzero()
    }

    fn coeffs(&self) -> Vec<(i64, String)> {
        self.fps.iter()
            .map(|(&k, v)| (k, v.to_string()))
            .collect()
    }
}
```

### Anti-Patterns to Avoid
- **Exposing ExprRef directly to Python:** ExprRef is a u32 with no validity checking. Python users must never construct or manipulate raw ExprRef values. Always wrap in QExpr.
- **Holding the Mutex lock across Python callbacks:** This will deadlock when Python code calls back into the session. Always release the lock before returning to Python.
- **Storing rug types in pyclass fields:** rug::Integer and rug::Rational are not Send+Sync in the way PyO3 requires. Convert at the boundary.
- **Using `&mut self` methods on pyclass:** PyO3 requires runtime borrow checking for &mut self. Use `#[pyclass(frozen)]` with Mutex instead.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| BigInt <-> Python int | Custom byte serialization | num-bigint BigInt or manual GMP digit extraction | Edge cases with sign, endianness, zero |
| Rational <-> Fraction | Custom string parsing | Extract numer/denom as BigInts, construct Fraction | fractions.Fraction is exact |
| Wheel building | Custom setup.py | maturin | Handles platform tags, auditwheel, shared lib bundling |
| GIL-safe locking | Custom lock wrappers | pyo3::sync::MutexExt (lock_py_attached) | Avoids deadlock with GC/GIL |
| Python operator overloading | Manual __dunder__ registration | PyO3 #[pymethods] with __add__ etc. | PyO3 handles slot registration automatically |
| Jupyter display | Custom mimetype handling | _repr_latex_() returning "$...$" | IPython display protocol is standardized |

**Key insight:** The entire qsym-core library is already built and tested. The Python layer should be a thin wrapper that converts types at boundaries and delegates all computation to Rust. Do not reimplement any mathematical logic in the Python layer.

## Common Pitfalls

### Pitfall 1: GMP Build Failure on Windows
**What goes wrong:** gmp-mpfr-sys fails to compile GMP from source in the cdylib build because MSYS2/MinGW tools are not in PATH.
**Why it happens:** The qsym-core crate uses `gmp-mpfr-sys` with `use-system-libs` feature, which on Windows is NOT SUPPORTED per the crate docs. The bundled build requires `diffutils`, `m4`, `make`, and `mingw-w64-x86_64-gcc` from MSYS2.
**How to avoid:**
1. Ensure MSYS2 MinGW64 tools are in PATH when building
2. The existing project already builds with the gnu toolchain on Cygwin, so the same environment should work for the cdylib
3. Test the cdylib build early (Task 1) before writing any bindings code
4. Consider switching from `use-system-libs` to bundled build if the current system-libs approach causes issues with cdylib linking
**Warning signs:** Linker errors mentioning `__gmp*` symbols, missing `m4` or `make` during build

### Pitfall 2: Mutex Deadlock with Python GC
**What goes wrong:** Python's garbage collector runs while the session Mutex is held, triggering a `__del__` or `__dealloc__` on a QExpr that tries to acquire the same lock.
**Why it happens:** QExpr holds an Arc to the session. When Python GC collects a QExpr, if deallocation tries to lock the session, deadlock occurs.
**How to avoid:**
1. QExpr's Drop/dealloc should NOT lock the session -- just drop the Arc (reference counting handles cleanup)
2. Use `#[pyclass(frozen)]` so PyO3 never needs to borrow-check
3. If custom dealloc is needed, use `try_lock()` instead of `lock()`
4. For operations that call back into Python, use `Python::detach()` (was `allow_threads`) to release the GIL before locking
**Warning signs:** Hanging tests, especially when many QExpr objects are created and discarded

### Pitfall 3: Session Lifetime vs Python GC
**What goes wrong:** Python user creates a QSession, creates QExpr objects from it, drops the session, then tries to use QExpr -- the session is still alive (Arc keeps it) but user is confused.
**Why it happens:** Arc reference counting means the session lives as long as any QExpr references it.
**How to avoid:** This is actually correct behavior -- document it. The session is automatically kept alive by its expressions. Consider adding a `session.clear()` or `session.stats()` method for debugging.
**Warning signs:** Memory usage growing unboundedly (arena is append-only)

### Pitfall 4: rug Types Not Send+Sync
**What goes wrong:** Compilation error when rug::Integer or rug::Rational is stored directly in a pyclass field.
**Why it happens:** GMP allocates from a custom allocator; the resulting types may not be thread-safe.
**How to avoid:** Never store rug types directly in pyclass. Keep them inside the SessionInner (behind the Mutex). Convert to Python native types (int, Fraction) at the boundary.
**Warning signs:** Compiler error about Send/Sync bounds on pyclass

### Pitfall 5: FormalPowerSeries Ownership
**What goes wrong:** FormalPowerSeries is a standalone struct (BTreeMap<i64, QRat> + SymbolId + i64). Trying to store it inside the session arena doesn't make sense -- it's a computation result, not an expression node.
**Why it happens:** Confusion between symbolic expressions (in the arena) and computed series (standalone values).
**How to avoid:** QSeries wraps FormalPowerSeries directly (owns it). The QSeries also holds an Arc to the session for type conversions and context, but the FPS data is owned by the QSeries object itself.
**Warning signs:** Trying to intern a FormalPowerSeries in the ExprArena

### Pitfall 6: Python Package Name Collision
**What goes wrong:** The Rust cdylib produces a `qsymbolic.pyd` (or `.so`) but Python also needs a `qsymbolic/` package directory for pure Python code.
**Why it happens:** maturin mixed mode requires careful coordination between the native module name and the Python package directory.
**How to avoid:** Use maturin's mixed layout: native module is `_qsymbolic` (underscore prefix), Python package is `qsymbolic/` which imports from `_qsymbolic`.
**Warning signs:** ImportError, module not found, or the pure Python layer not being included in the wheel

## Code Examples

Verified patterns from official sources:

### QSession + QExpr Core Pattern
```rust
// Source: PyO3 thread-safety guide + frozen pyclass pattern
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use std::sync::{Arc, Mutex};
use qsym_core::{ExprArena, ExprRef};
use qsym_core::expr::Expr;
use qsym_core::number::{QInt, QRat};
use qsym_core::canonical;
use qsym_core::render;

struct SessionInner {
    arena: ExprArena,
}

#[pyclass(frozen)]
#[derive(Clone)]
struct QSession {
    inner: Arc<Mutex<SessionInner>>,
}

#[pymethods]
impl QSession {
    #[new]
    fn new() -> Self {
        QSession {
            inner: Arc::new(Mutex::new(SessionInner {
                arena: ExprArena::new(),
            })),
        }
    }

    /// Create a symbolic variable
    fn symbol(&self, name: &str) -> QExpr {
        let mut s = self.inner.lock().unwrap();
        let r = s.arena.intern_symbol(name);
        QExpr { session: Arc::clone(&self.inner), expr_ref: r }
    }

    /// Create multiple symbols at once: symbols("q a b n")
    fn symbols(&self, names: &str) -> Vec<QExpr> {
        let mut s = self.inner.lock().unwrap();
        names.split_whitespace()
            .map(|name| {
                let r = s.arena.intern_symbol(name);
                QExpr { session: Arc::clone(&self.inner), expr_ref: r }
            })
            .collect()
    }

    /// Create an integer expression
    fn integer(&self, val: i64) -> QExpr {
        let mut s = self.inner.lock().unwrap();
        let r = s.arena.intern_int(val);
        QExpr { session: Arc::clone(&self.inner), expr_ref: r }
    }

    /// Create a rational expression
    fn rational(&self, num: i64, den: i64) -> PyResult<QExpr> {
        if den == 0 {
            return Err(PyValueError::new_err("denominator cannot be zero"));
        }
        let mut s = self.inner.lock().unwrap();
        let r = s.arena.intern_rat(num, den);
        Ok(QExpr { session: Arc::clone(&self.inner), expr_ref: r })
    }

    /// Get arena statistics
    fn stats(&self) -> (usize, usize) {
        let s = self.inner.lock().unwrap();
        (s.arena.len(), s.arena.symbols().len())
    }
}
```

### QExpr with Dunder Methods
```rust
// Source: PyO3 class/protocols guide
#[pyclass(frozen)]
#[derive(Clone)]
struct QExpr {
    session: Arc<Mutex<SessionInner>>,
    expr_ref: ExprRef,
}

#[pymethods]
impl QExpr {
    /// Unicode representation for Python REPL
    fn __repr__(&self) -> String {
        let s = self.session.lock().unwrap();
        format!("{}", s.arena.display(self.expr_ref))
    }

    /// LaTeX representation for Jupyter notebooks
    fn _repr_latex_(&self) -> String {
        let s = self.session.lock().unwrap();
        let latex = render::to_latex(&s.arena, self.expr_ref);
        format!("${}$", latex)
    }

    /// String conversion
    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// Addition: expr + other
    fn __add__(&self, other: &QExpr) -> QExpr {
        let mut s = self.session.lock().unwrap();
        let r = canonical::make_add(&mut s.arena, vec![self.expr_ref, other.expr_ref]);
        QExpr { session: Arc::clone(&self.session), expr_ref: r }
    }

    /// Multiplication: expr * other
    fn __mul__(&self, other: &QExpr) -> QExpr {
        let mut s = self.session.lock().unwrap();
        let r = canonical::make_mul(&mut s.arena, vec![self.expr_ref, other.expr_ref]);
        QExpr { session: Arc::clone(&self.session), expr_ref: r }
    }

    /// Negation: -expr
    fn __neg__(&self) -> QExpr {
        let mut s = self.session.lock().unwrap();
        let r = canonical::make_neg(&mut s.arena, self.expr_ref);
        QExpr { session: Arc::clone(&self.session), expr_ref: r }
    }

    /// Power: expr ** other
    fn __pow__(&self, other: &QExpr, _modulo: Option<&Bound<'_, PyAny>>) -> QExpr {
        let mut s = self.session.lock().unwrap();
        let r = canonical::make_pow(&mut s.arena, self.expr_ref, other.expr_ref);
        QExpr { session: Arc::clone(&self.session), expr_ref: r }
    }

    /// Equality: structural (O(1) via hash-consing)
    fn __eq__(&self, other: &QExpr) -> bool {
        self.expr_ref == other.expr_ref
    }

    fn __hash__(&self) -> u64 {
        self.expr_ref.0 as u64
    }

    /// Get LaTeX string without dollar signs
    fn latex(&self) -> String {
        let s = self.session.lock().unwrap();
        render::to_latex(&s.arena, self.expr_ref)
    }

    /// Simplify the expression
    fn simplify(&self) -> QExpr {
        let mut s = self.session.lock().unwrap();
        let engine = qsym_core::simplify::SimplificationEngine::new();
        let r = engine.simplify(self.expr_ref, &mut s.arena);
        QExpr { session: Arc::clone(&self.session), expr_ref: r }
    }
}
```

### Module Definition
```rust
// Source: PyO3 module guide + maturin mixed layout
#[pymodule]
fn _qsymbolic(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<QSession>()?;
    m.add_class::<QExpr>()?;
    m.add_class::<QSeries>()?;
    m.add_function(wrap_pyfunction!(qpoch, m)?)?;
    m.add_function(wrap_pyfunction!(theta3, m)?)?;
    m.add_function(wrap_pyfunction!(theta4, m)?)?;
    // ... more DSL functions
    Ok(())
}
```

### rug Integer to Python int Conversion
```rust
// Source: Manual implementation -- rug does not have PyO3 integration
use pyo3::types::PyInt;

fn qint_to_python<'py>(py: Python<'py>, val: &QInt) -> PyResult<Bound<'py, PyInt>> {
    // Convert via string for correctness (handles arbitrary size)
    let s = val.0.to_string();
    // Parse as Python int -- handles arbitrary precision
    let builtins = py.import("builtins")?;
    let py_int = builtins.getattr("int")?.call1((s,))?;
    Ok(py_int.downcast_into::<PyInt>()?)
}

fn qrat_to_python<'py>(py: Python<'py>, val: &QRat) -> PyResult<PyObject> {
    let fractions = py.import("fractions")?;
    let numer = val.0.numer().to_string();
    let denom = val.0.denom().to_string();
    let fraction = fractions.getattr("Fraction")?.call1((numer, denom))?;
    Ok(fraction.into())
}
```

### Batch Computation Pattern
```rust
// Source: Project-specific design for PYTH-05
#[pymethods]
impl QSession {
    /// Run a batch parameter search over a grid
    /// Returns list of (params, series) tuples
    fn batch_compute(
        &self,
        py: Python<'_>,
        func_name: &str,
        param_grid: Vec<Vec<i64>>,
        truncation_order: i64,
    ) -> PyResult<Vec<(Vec<i64>, QSeries)>> {
        // Release GIL for heavy computation
        py.detach(|| {
            let mut results = Vec::new();
            let mut session = self.inner.lock().unwrap();
            let q_sym = session.arena.symbols_mut().intern("q");

            for params in &param_grid {
                let fps = match func_name {
                    "aqprod" => {
                        let a = QMonomial::q_power(params[0]);
                        let order = PochhammerOrder::Infinite;
                        qsym_core::qseries::aqprod(&a, q_sym, order, truncation_order)
                    }
                    "theta3" => qsym_core::qseries::theta3(q_sym, truncation_order),
                    // ... more functions
                    _ => return Err(PyValueError::new_err("unknown function")),
                };
                results.push((params.clone(), QSeries {
                    session: Arc::clone(&self.inner),
                    fps,
                }));
            }
            Ok(results)
        })
    }
}
```

### Cargo.toml for qsym-python crate
```toml
# Source: maturin tutorial + PyO3 guide
[package]
name = "qsym-python"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[lib]
name = "_qsymbolic"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.23", features = ["extension-module"] }
qsym-core = { path = "../qsym-core" }

# Note: qsym-core brings in rug + gmp-mpfr-sys transitively
```

### pyproject.toml
```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "qsymbolic"
version = "0.1.0"
description = "Symbolic computation engine for q-series"
requires-python = ">=3.9"

[tool.maturin]
# Build the cdylib from the qsym-python crate
manifest-path = "crates/qsym-python/Cargo.toml"
# Mixed Rust/Python layout
python-source = "crates/qsym-python/python"
features = ["pyo3/extension-module"]
```

### Pure Python __init__.py
```python
# crates/qsym-python/python/qsymbolic/__init__.py
"""Q-Symbolic: Symbolic computation for q-series."""

from _qsymbolic import QSession, QExpr, QSeries
from _qsymbolic import qpoch, theta3, theta4

# Convenience: create a default session
def symbols(names: str, session=None):
    """Create symbolic variables. Returns tuple if multiple names."""
    if session is None:
        session = QSession()
    result = session.symbols(names)
    if len(result) == 1:
        return result[0]
    return tuple(result)

__all__ = [
    "QSession", "QExpr", "QSeries",
    "qpoch", "theta3", "theta4",
    "symbols",
]
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `PyObject` type alias | `Py<PyAny>` smart pointer | PyO3 0.21+ | PyObject deprecated; use Py<T> or Bound<'py, T> |
| `Python::with_gil()` | `Python::attach()` | PyO3 0.26 | Rename only; same semantics |
| `Python::allow_threads()` | `Python::detach()` | PyO3 0.26 | Rename only; releases GIL for Rust computation |
| `#[pyclass]` with &mut self | `#[pyclass(frozen)]` + Mutex | PyO3 0.23+ | frozen is recommended for thread safety |
| `gil-refs` feature | Bound<'py, T> API | PyO3 0.23 | gil-refs removed; Bound is the only path |
| Opt-in free-threaded Python | Opt-out free-threaded Python | PyO3 0.28 | Free-threaded support now default |

**Deprecated/outdated:**
- `IntoPy` trait: Replaced by `IntoPyObject` (PyO3 0.23+)
- `PyObject`: Use `Py<PyAny>` directly
- GIL refs (`&PyDict`, `&PyList` etc.): Use `Bound<'py, PyDict>` etc.
- `ToPyObject`: Use `IntoPyObject`

## Open Questions

1. **PyO3 version targeting: 0.23 vs 0.28?**
   - What we know: PyO3 0.28 is latest (2026-02-01) with `__init__` support and free-threaded Python opt-out. PyO3 0.23 is the minimum supporting Bound API and frozen pyclass.
   - What's unclear: Whether 0.28's breaking changes (removed 0.25-0.26 deprecations) cause issues. Whether we need free-threaded Python support.
   - Recommendation: Target PyO3 0.23 initially for maximum compatibility. Upgrade to 0.28 after basic functionality works. The core patterns (frozen pyclass, Mutex) are identical across versions.

2. **gmp-mpfr-sys `use-system-libs` in cdylib context**
   - What we know: qsym-core currently uses `use-system-libs` feature. This feature is documented as "not supported on Windows" by the crate. But the project builds successfully in Cygwin.
   - What's unclear: Whether a cdylib build will successfully link GMP into the shared library. Whether the resulting .pyd will find GMP at runtime.
   - Recommendation: Test cdylib compilation FIRST before writing bindings. If `use-system-libs` fails for cdylib, switch to bundled build (remove the feature flag). The bundled build compiles GMP from source and statically links it, which is actually better for wheel distribution.

3. **Memory management for append-only arena**
   - What we know: ExprArena is append-only (never frees expressions). This is fine for CAS usage but could leak if Python users create many sessions.
   - What's unclear: Whether typical research workflows create enough expressions to matter. Whether we need a `session.compact()` method.
   - Recommendation: Defer. Document that sessions grow monotonically. Users who need to discard old work should create a new QSession. This matches Mathematica/Maple behavior.

4. **Batch computation GIL release**
   - What we know: `Python::detach()` releases the GIL for pure Rust computation. The qseries functions are pure Rust and don't call Python.
   - What's unclear: Whether rug's GMP allocation is thread-safe when the GIL is released. GMP uses a global allocator by default.
   - Recommendation: Release GIL for batch computation but test thoroughly. GMP's default allocator uses malloc which is thread-safe on modern platforms. If issues arise, keep GIL held during computation (simpler, slightly slower for multi-threaded Python).

5. **Python package distribution**
   - What we know: maturin can build wheels. The GMP dependency makes this a platform-specific wheel (not pure Python, not manylinux without bundled GMP).
   - What's unclear: Whether the wheel will work on other Windows machines without MSYS2/Cygwin installed.
   - Recommendation: For Phase 5, focus on `maturin develop` for local use. Wheel distribution to other machines is a Phase 6+ concern. The bundled GMP build (statically linked) would make this possible.

## Sources

### Primary (HIGH confidence)
- [PyO3 main guide - Python classes](https://pyo3.rs/main/class.html) - pyclass, pymethods, frozen, protocols
- [PyO3 thread-safety guide](https://pyo3.rs/v0.25.1/class/thread-safety.html) - Mutex patterns, MutexExt, frozen + interior mutability
- [PyO3 class protocols](https://pyo3.rs/main/class/protocols.html) - __repr__, __add__, __getitem__, mapping protocol
- [PyO3 type conversion tables](https://pyo3.rs/main/conversions/tables.html) - HashMap/BTreeMap->dict, BigInt->int, Ratio->Fraction
- [PyO3 changelog](https://pyo3.rs/main/changelog.html) - Version history 0.23-0.28
- [gmp-mpfr-sys docs](https://docs.rs/gmp-mpfr-sys/latest/gmp_mpfr_sys/index.html) - Version 1.6.8, Windows build requirements, use-system-libs limitations
- [maturin tutorial](https://www.maturin.rs/tutorial.html) - Project setup, pyproject.toml, cdylib config
- [IPython display integration](https://ipython.readthedocs.io/en/stable/config/integrating.html) - _repr_latex_(), _repr_html_(), display protocol

### Secondary (MEDIUM confidence)
- [PyO3 rug feature discussion #3691](https://github.com/PyO3/pyo3/discussions/3691) - rug integration not merged, LGPL concerns, community fork exists
- [maturin issue #2370](https://github.com/PyO3/maturin/issues/2370) - MSYS2 pip install issues
- [Jupyter _repr_mimebundle_ forum thread](https://discourse.jupyter.org/t/how-to-write-a-class-which-naturally-produce-a-latex-output-in-jupyter-notebook/17437) - LaTeX display in notebooks
- [PyO3 numeric types guide](https://pyo3.rs/main/class/numeric.html) - __add__, __mul__ implementation patterns

### Tertiary (LOW confidence)
- PyO3 + rug + GMP coexistence in a single cdylib on Windows -- no verified examples found. This is the highest-risk area and needs early validation.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - PyO3, maturin, and their patterns are well-documented
- Architecture (session/arena pattern): HIGH - Standard PyO3 frozen+Mutex pattern adapted to arena design
- Pitfalls: MEDIUM - Deadlock and GMP linking issues are known patterns but not specifically verified for this exact combo
- GMP+cdylib on Windows: LOW - No prior art found for rug+PyO3+cdylib on Windows/Cygwin; needs early build validation
- Type conversions (rug->Python): MEDIUM - String-based conversion is reliable but may be slow for bulk operations; byte-level conversion is faster but more complex

**Research date:** 2026-02-13
**Valid until:** 2026-03-15 (PyO3 releases roughly monthly; maturin stable)
