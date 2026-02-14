//! QSeries: Python-facing wrapper for FormalPowerSeries.
//!
//! QSeries owns a `FormalPowerSeries` directly (not behind the session Mutex).
//! FPS is a standalone computation result, not an arena expression -- there is
//! no need for GC-safe Arc reference counting here.

use pyo3::prelude::*;

use qsym_core::series::{FormalPowerSeries, arithmetic};
use qsym_core::qseries;

use crate::convert::qrat_to_python;

/// A q-series (formal power series) with sparse rational coefficients.
///
/// Supports coefficient access via `series[k]` (returns a Python Fraction),
/// arithmetic operations (+, *, -, unary -), and string display in the REPL.
///
/// ```python
/// s = QSession()
/// e = etaq(s, 1, 1, 20)
/// print(e)          # '1 - q + q^2 + ...'
/// print(e[0])       # Fraction(1, 1)
/// print(len(e))     # number of nonzero coefficients
/// ```
#[pyclass(frozen)]
pub struct QSeries {
    pub(crate) fps: FormalPowerSeries,
}

#[pymethods]
impl QSeries {
    /// Unicode string representation for the REPL.
    fn __repr__(&self) -> String {
        format!("{}", self.fps)
    }

    /// String representation (same as __repr__).
    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// Get the coefficient at power `key`, returned as a Python Fraction.
    ///
    /// Raises IndexError if key >= truncation_order.
    fn __getitem__(&self, py: Python<'_>, key: i64) -> PyResult<PyObject> {
        if key >= self.fps.truncation_order() {
            return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                "coefficient at q^{} is unknown (series truncated at O(q^{}))",
                key,
                self.fps.truncation_order()
            )));
        }
        let c = self.fps.coeff(key);
        let obj = qrat_to_python(py, &c)?;
        Ok(obj.into())
    }

    /// Number of nonzero coefficients stored.
    fn __len__(&self) -> usize {
        self.fps.num_nonzero()
    }

    /// The truncation order N: series is known exactly for exponents < N.
    fn truncation_order(&self) -> i64 {
        self.fps.truncation_order()
    }

    /// Lowest power with nonzero coefficient, or None if zero series.
    fn min_order(&self) -> Option<i64> {
        self.fps.min_order()
    }

    /// True if all coefficients are zero (the zero series).
    fn is_zero(&self) -> bool {
        self.fps.is_zero()
    }

    /// Iterate over nonzero coefficients, returning list of (power, Fraction) tuples.
    fn coeffs(&self, py: Python<'_>) -> PyResult<Vec<(i64, PyObject)>> {
        let mut result = Vec::new();
        for (&k, v) in self.fps.iter() {
            let frac = qrat_to_python(py, v)?;
            result.push((k, frac.into()));
        }
        Ok(result)
    }

    /// Return a Python dict mapping power -> Fraction for nonzero coefficients.
    fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = pyo3::types::PyDict::new(py);
        for (&k, v) in self.fps.iter() {
            let frac = qrat_to_python(py, v)?;
            dict.set_item(k, frac)?;
        }
        Ok(dict.into())
    }

    /// Highest nonzero exponent (degree), or None if zero series.
    fn degree(&self) -> Option<i64> {
        qseries::qdegree(&self.fps)
    }

    /// Lowest nonzero exponent (low degree / valuation), or None if zero series.
    fn low_degree(&self) -> Option<i64> {
        qseries::lqdegree(&self.fps)
    }

    // ---- Arithmetic ----

    /// Addition: self + other
    fn __add__(&self, other: &QSeries) -> QSeries {
        QSeries {
            fps: arithmetic::add(&self.fps, &other.fps),
        }
    }

    /// Multiplication: self * other
    fn __mul__(&self, other: &QSeries) -> QSeries {
        QSeries {
            fps: arithmetic::mul(&self.fps, &other.fps),
        }
    }

    /// Unary negation: -self
    fn __neg__(&self) -> QSeries {
        QSeries {
            fps: arithmetic::negate(&self.fps),
        }
    }

    /// Subtraction: self - other
    fn __sub__(&self, other: &QSeries) -> QSeries {
        let neg_other = arithmetic::negate(&other.fps);
        QSeries {
            fps: arithmetic::add(&self.fps, &neg_other),
        }
    }

    /// Multiplicative inverse: 1 / self
    fn invert(&self) -> QSeries {
        QSeries {
            fps: arithmetic::invert(&self.fps),
        }
    }

    /// Extract arithmetic subsequence: g[i] = self[m*i + j].
    fn sift(&self, m: i64, j: i64) -> QSeries {
        QSeries {
            fps: qseries::sift(&self.fps, m, j),
        }
    }
}
