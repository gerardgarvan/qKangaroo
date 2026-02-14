//! Relation discovery functions for q-series.
//!
//! Provides the core relation discovery tools from Garvan's qseries package:
//! - [`findlincombo`]: find f as a linear combination of basis series
//! - [`findhom`]: find homogeneous polynomial relations among series
//! - [`findpoly`]: find a two-variable polynomial relation P(x,y) = 0
//!
//! All functions follow the coefficient-matrix + null-space pattern:
//! 1. Build candidate series (monomials in the input series)
//! 2. Extract coefficients into a matrix
//! 3. Compute the rational null space
//! 4. Interpret null space vectors as relations

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use super::linalg::{build_coefficient_matrix, rational_null_space};

/// A polynomial relation P(x, y) = 0 discovered by [`findpoly`].
///
/// The polynomial is represented as a 2D coefficient grid where
/// `coefficients[i][j]` is the coefficient of x^i * y^j.
#[derive(Clone, Debug)]
pub struct PolynomialRelation {
    /// coefficients[i][j] is the coefficient of x^i * y^j in P(x, y).
    pub coefficients: Vec<Vec<QRat>>,
    /// Maximum degree in x.
    pub deg_x: i64,
    /// Maximum degree in y.
    pub deg_y: i64,
}

/// Compute f^n for a formal power series by repeated multiplication.
///
/// - n = 0 returns FPS::one
/// - n = 1 returns a clone
/// - n > 1 uses repeated squaring
/// - n < 0 inverts first, then raises to |n|
fn fps_pow(f: &FormalPowerSeries, n: i64) -> FormalPowerSeries {
    if n == 0 {
        return FormalPowerSeries::one(f.variable(), f.truncation_order());
    }

    let (base, exp) = if n < 0 {
        (arithmetic::invert(f), (-n) as u64)
    } else {
        (f.clone(), n as u64)
    };

    // Repeated squaring
    let mut result = FormalPowerSeries::one(base.variable(), base.truncation_order());
    let mut power = base;
    let mut e = exp;

    while e > 0 {
        if e & 1 == 1 {
            result = arithmetic::mul(&result, &power);
        }
        e >>= 1;
        if e > 0 {
            power = arithmetic::mul(&power, &power);
        }
    }

    result
}

/// Find coefficients c_1, ..., c_k such that f = c_1*basis[0] + ... + c_k*basis[k-1].
///
/// Uses the coefficient-matrix + null-space approach: builds a matrix where columns
/// correspond to [f, basis[0], ..., basis[k-1]] and rows are coefficient values,
/// then finds a null space vector with nonzero first component (corresponding to f).
///
/// # Arguments
///
/// - `f`: the target series to express as a linear combination
/// - `basis`: the basis series
/// - `topshift`: extra rows beyond the number of columns for overdetermined system
///
/// # Returns
///
/// `Some(coefficients)` where `f = sum_i coefficients[i] * basis[i]`, or `None`
/// if f is not a linear combination of the basis.
pub fn findlincombo(
    f: &FormalPowerSeries,
    basis: &[&FormalPowerSeries],
    topshift: i64,
) -> Option<Vec<QRat>> {
    if basis.is_empty() {
        // f must be zero for a trivial "combination"
        if f.is_zero() {
            return Some(Vec::new());
        }
        return None;
    }

    // Build candidates list: [f, basis[0], basis[1], ..., basis[k-1]]
    let mut candidates: Vec<&FormalPowerSeries> = Vec::with_capacity(basis.len() + 1);
    candidates.push(f);
    candidates.extend_from_slice(basis);

    let num_candidates = candidates.len(); // k + 1

    // Determine start_order: minimum of min_order across all candidates
    let start_order = candidates
        .iter()
        .filter_map(|fps| fps.min_order())
        .min()
        .unwrap_or(0)
        .min(0);

    // Determine the max truncation order available
    let max_trunc = candidates
        .iter()
        .map(|fps| fps.truncation_order())
        .min()
        .unwrap();

    // Number of rows: enough to overdetermine the system
    let available_rows = (max_trunc - start_order) as usize;
    let desired_rows = num_candidates + topshift as usize;
    let num_rows = desired_rows.min(available_rows);

    if num_rows == 0 {
        return None;
    }

    // Build coefficient matrix and compute null space
    let matrix = build_coefficient_matrix(&candidates, start_order, num_rows);
    let null_space = rational_null_space(&matrix);

    // Look for a null space vector where the first component (for f) is nonzero
    for v in &null_space {
        if !v[0].is_zero() {
            // Normalize so first component = 1
            let scale = QRat::one() / v[0].clone();
            // Coefficients are the negatives of the remaining components
            let coefficients: Vec<QRat> = v[1..]
                .iter()
                .map(|c| -(c.clone() * scale.clone()))
                .collect();
            return Some(coefficients);
        }
    }

    None
}

/// Generate all k-tuples of non-negative integers that sum to `degree`.
///
/// Returns a vector of exponent tuples, each of length `k`.
fn generate_monomials(k: usize, degree: i64) -> Vec<Vec<i64>> {
    let mut result = Vec::new();
    let mut current = vec![0i64; k];
    generate_monomials_recursive(k, degree, 0, &mut current, &mut result);
    result
}

fn generate_monomials_recursive(
    k: usize,
    remaining: i64,
    pos: usize,
    current: &mut Vec<i64>,
    result: &mut Vec<Vec<i64>>,
) {
    if pos == k - 1 {
        current[pos] = remaining;
        result.push(current.clone());
        return;
    }

    for val in 0..=remaining {
        current[pos] = val;
        generate_monomials_recursive(k, remaining - val, pos + 1, current, result);
    }
}

/// Compute the product series[0]^exponents[0] * series[1]^exponents[1] * ...
fn compute_monomial_series(
    series: &[&FormalPowerSeries],
    exponents: &[i64],
) -> FormalPowerSeries {
    assert_eq!(series.len(), exponents.len());

    // Start with 1
    let trunc = series
        .iter()
        .map(|s| s.truncation_order())
        .min()
        .unwrap_or(1);
    let var = series[0].variable();
    let mut result = FormalPowerSeries::one(var, trunc);

    for (s, &e) in series.iter().zip(exponents.iter()) {
        if e == 0 {
            continue;
        }
        let powered = fps_pow(s, e);
        result = arithmetic::mul(&result, &powered);
    }

    result
}

/// Find all homogeneous degree-d polynomial relations among the given series.
///
/// Generates all monomials of degree `degree` in the input series, computes the
/// corresponding FPS for each monomial, builds a coefficient matrix, and finds the
/// null space. Each null space vector represents a homogeneous polynomial relation.
///
/// # Arguments
///
/// - `series`: the series to find relations among
/// - `degree`: the total degree of the homogeneous polynomial
/// - `topshift`: extra rows for overdetermination
///
/// # Returns
///
/// A vector of relation vectors. Each vector has one entry per monomial
/// (in the order returned by `generate_monomials`). A nonzero vector `v` means
/// `sum_i v[i] * monomial_i = 0`.
///
/// The monomial ordering is: all k-tuples of non-negative integers summing to
/// `degree`, in lexicographic order.
pub fn findhom(
    series: &[&FormalPowerSeries],
    degree: i64,
    topshift: i64,
) -> Vec<Vec<QRat>> {
    let k = series.len();
    if k == 0 || degree < 0 {
        return Vec::new();
    }

    // Generate all monomials of the given degree
    let monomials = generate_monomials(k, degree);
    let num_monomials = monomials.len();

    if num_monomials == 0 {
        return Vec::new();
    }

    // Compute the FPS for each monomial
    let monomial_series: Vec<FormalPowerSeries> = monomials
        .iter()
        .map(|exps| compute_monomial_series(series, exps))
        .collect();

    let candidates: Vec<&FormalPowerSeries> = monomial_series.iter().collect();

    // Determine start_order
    let start_order = candidates
        .iter()
        .filter_map(|fps| fps.min_order())
        .min()
        .unwrap_or(0)
        .min(0);

    // Determine available rows
    let max_trunc = candidates
        .iter()
        .map(|fps| fps.truncation_order())
        .min()
        .unwrap();

    let available_rows = (max_trunc - start_order) as usize;
    let desired_rows = num_monomials + topshift as usize;
    let num_rows = desired_rows.min(available_rows);

    if num_rows == 0 {
        return Vec::new();
    }

    // Build coefficient matrix and compute null space
    let matrix = build_coefficient_matrix(&candidates, start_order, num_rows);
    rational_null_space(&matrix)
}

/// Find a polynomial relation P(x, y) = 0 between two series.
///
/// Searches for a polynomial P with degree at most `deg_x` in x and `deg_y` in y
/// such that P(x, y) = 0 when x and y are substituted with the given series.
///
/// Candidates are all x^i * y^j for 0 <= i <= deg_x, 0 <= j <= deg_y.
///
/// # Arguments
///
/// - `x`: the first series
/// - `y`: the second series
/// - `deg_x`: maximum degree in x
/// - `deg_y`: maximum degree in y
/// - `topshift`: extra rows for overdetermination
///
/// # Returns
///
/// `Some(PolynomialRelation)` if a relation is found, `None` otherwise.
pub fn findpoly(
    x: &FormalPowerSeries,
    y: &FormalPowerSeries,
    deg_x: i64,
    deg_y: i64,
    topshift: i64,
) -> Option<PolynomialRelation> {
    if deg_x < 0 || deg_y < 0 {
        return None;
    }

    // Build candidate series: x^i * y^j for all valid (i, j)
    // Order: (0,0), (0,1), ..., (0,deg_y), (1,0), (1,1), ..., (1,deg_y), ...
    let mut candidate_series: Vec<FormalPowerSeries> = Vec::new();

    // Precompute powers of x and y
    let x_powers: Vec<FormalPowerSeries> = (0..=deg_x)
        .map(|i| fps_pow(x, i))
        .collect();
    let y_powers: Vec<FormalPowerSeries> = (0..=deg_y)
        .map(|j| fps_pow(y, j))
        .collect();

    for i in 0..=deg_x {
        for j in 0..=deg_y {
            let product = arithmetic::mul(&x_powers[i as usize], &y_powers[j as usize]);
            candidate_series.push(product);
        }
    }

    let candidates: Vec<&FormalPowerSeries> = candidate_series.iter().collect();
    let num_candidates = candidates.len();

    // Determine start_order
    let start_order = candidates
        .iter()
        .filter_map(|fps| fps.min_order())
        .min()
        .unwrap_or(0)
        .min(0);

    // Determine available rows
    let max_trunc = candidates
        .iter()
        .map(|fps| fps.truncation_order())
        .min()
        .unwrap();

    let available_rows = (max_trunc - start_order) as usize;
    let desired_rows = num_candidates + topshift as usize;
    let num_rows = desired_rows.min(available_rows);

    if num_rows == 0 {
        return None;
    }

    // Build coefficient matrix and compute null space
    let matrix = build_coefficient_matrix(&candidates, start_order, num_rows);
    let null_space = rational_null_space(&matrix);

    if null_space.is_empty() {
        return None;
    }

    // Take the first null space vector and format as PolynomialRelation
    let v = &null_space[0];

    // Convert flat vector to 2D grid: v[i * (deg_y+1) + j] -> coefficients[i][j]
    let dy = (deg_y + 1) as usize;
    let mut coefficients = Vec::new();
    for i in 0..=deg_x {
        let row: Vec<QRat> = (0..=deg_y)
            .map(|j| v[i as usize * dy + j as usize].clone())
            .collect();
        coefficients.push(row);
    }

    Some(PolynomialRelation {
        coefficients,
        deg_x,
        deg_y,
    })
}
