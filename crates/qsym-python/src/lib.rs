use pyo3::prelude::*;

/// Return the version string, exercising GMP linkage by creating and
/// dropping an ExprArena (which internally allocates rug/GMP values).
#[pyfunction]
fn version() -> &'static str {
    // Create and immediately drop an ExprArena to prove that qsym-core
    // (and its GMP dependency via rug) links and works at runtime.
    let arena = qsym_core::ExprArena::new();
    drop(arena);
    "0.1.0"
}

/// The native Python module entry point.
#[pymodule]
fn _qsymbolic(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
