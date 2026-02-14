use pyo3::prelude::*;

mod convert;
mod dsl;
mod expr;
mod series;
mod session;

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
    m.add_class::<session::QSession>()?;
    m.add_class::<expr::QExpr>()?;
    m.add_class::<series::QSeries>()?;

    // Group 1: Pochhammer and q-Binomial
    m.add_function(wrap_pyfunction!(dsl::aqprod, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::qbin, m)?)?;

    // Group 2: Named Products
    m.add_function(wrap_pyfunction!(dsl::etaq, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::jacprod, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::tripleprod, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::quinprod, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::winquist, m)?)?;

    // Group 3: Theta Functions
    m.add_function(wrap_pyfunction!(dsl::theta2, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::theta3, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::theta4, m)?)?;

    Ok(())
}
