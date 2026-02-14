//! Conversion helpers between rug arbitrary-precision types and Python objects.
//!
//! Strategy: Convert via string representation, which is reliable for
//! arbitrary precision values that may exceed i64/f64 range.

use pyo3::prelude::*;
use qsym_core::number::{QInt, QRat};

/// Convert a rug Integer (wrapped in QInt) to a Python int.
///
/// Uses string conversion for arbitrary precision: rug -> string -> Python int().
pub fn qint_to_python<'py>(py: Python<'py>, val: &QInt) -> PyResult<Bound<'py, PyAny>> {
    let s = val.0.to_string();
    let builtins = py.import("builtins")?;
    builtins.getattr("int")?.call1((s,))
}

/// Convert a rug Rational (wrapped in QRat) to a Python fractions.Fraction.
///
/// Uses string conversion of numerator and denominator for arbitrary precision.
pub fn qrat_to_python<'py>(py: Python<'py>, val: &QRat) -> PyResult<Bound<'py, PyAny>> {
    let numer_s = val.0.numer().to_string();
    let denom_s = val.0.denom().to_string();
    let fractions = py.import("fractions")?;
    let fraction_cls = fractions.getattr("Fraction")?;
    let numer = py.import("builtins")?.getattr("int")?.call1((numer_s,))?;
    let denom = py.import("builtins")?.getattr("int")?.call1((denom_s,))?;
    fraction_cls.call1((numer, denom))
}

/// Extract an i64 from a Python int.
///
/// Used for function parameters that expect bounded integers.
pub fn python_int_to_i64(obj: &Bound<'_, PyAny>) -> PyResult<i64> {
    obj.extract::<i64>()
}
