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
use super::utilities::sift;

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

/// A congruence discovered by [`findcong`].
///
/// Represents the statement that f(modulus_m * n + residue_b) = 0 (mod divisor_r)
/// for all n in the tested range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Congruence {
    /// The modulus A in f(A*n + B).
    pub modulus_m: i64,
    /// The residue B in f(A*n + B).
    pub residue_b: i64,
    /// The divisor R such that f(A*n + B) = 0 mod R for all tested n.
    pub divisor_r: i64,
}

/// Discover congruences among the coefficients of a formal power series.
///
/// For each modulus m in `moduli`, for each residue j in 0..m, extracts the
/// subsequence f(m*n + j) using [`sift`] and checks whether all coefficients
/// are divisible by some small prime or by m itself.
///
/// This is the key tool for automated discovery of partition congruences.
/// For example, `findcong(&partition_gf, &[5])` discovers Ramanujan's
/// famous congruence p(5n+4) = 0 (mod 5).
///
/// # Arguments
///
/// - `f`: the input series whose coefficients are tested for congruences
/// - `moduli`: list of moduli to test
///
/// # Returns
///
/// All discovered congruences, one per (modulus, residue, divisor) triple.
pub fn findcong(f: &FormalPowerSeries, moduli: &[i64]) -> Vec<Congruence> {
    let test_primes: &[i64] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31];

    let mut results = Vec::new();

    for &m in moduli {
        assert!(m > 0, "findcong modulus must be positive, got {}", m);
        for j in 0..m {
            let sub = sift(f, m, j);

            // Collect all nonzero coefficients from the subsequence
            let nonzero_coeffs: Vec<QRat> = sub
                .iter()
                .map(|(_, c)| c.clone())
                .filter(|c| !c.is_zero())
                .collect();

            if nonzero_coeffs.is_empty() {
                // All zero: trivially divisible by any R, skip (not interesting)
                continue;
            }

            // Build a list of candidate divisors: test_primes + m itself (if not already included)
            let mut candidates: Vec<i64> = test_primes.to_vec();
            if !candidates.contains(&m) {
                candidates.push(m);
            }

            for &r in &candidates {
                if r <= 1 {
                    continue;
                }

                // Check if ALL nonzero coefficients are divisible by r.
                // Each coefficient is a QRat; for integer-coefficient series,
                // we check if numerator is divisible by r (denominator should be 1).
                let r_int = rug::Integer::from(r);
                let all_div = nonzero_coeffs.iter().all(|c| {
                    // Coefficient must be an integer (denominator = 1) for congruence testing
                    let one = rug::Integer::from(1);
                    if c.denom() != &one {
                        return false;
                    }
                    c.numer().is_divisible(&r_int)
                });

                if all_div {
                    results.push(Congruence {
                        modulus_m: m,
                        residue_b: j,
                        divisor_r: r,
                    });
                }
            }
        }
    }

    results
}

/// Find all non-homogeneous polynomial relations of degree <= d among the given series.
///
/// Like [`findhom`] but generates all monomials of degree 0, 1, ..., d (not just
/// exactly d). This allows discovering affine and mixed-degree relations.
///
/// The constant monomial (all exponents 0, representing the constant series 1)
/// is included as the first candidate.
///
/// # Arguments
///
/// - `series`: the series to find relations among
/// - `degree`: the maximum total degree
/// - `topshift`: extra rows for overdetermination
///
/// # Returns
///
/// A vector of relation vectors, one entry per monomial across all degrees 0..=d.
pub fn findnonhom(
    series: &[&FormalPowerSeries],
    degree: i64,
    topshift: i64,
) -> Vec<Vec<QRat>> {
    let k = series.len();
    if k == 0 || degree < 0 {
        return Vec::new();
    }

    // Generate all monomials of degree 0, 1, ..., degree and concatenate
    let mut all_monomials: Vec<Vec<i64>> = Vec::new();
    for d in 0..=degree {
        let monos = generate_monomials(k, d);
        all_monomials.extend(monos);
    }

    let num_monomials = all_monomials.len();
    if num_monomials == 0 {
        return Vec::new();
    }

    // Compute the FPS for each monomial
    let monomial_series: Vec<FormalPowerSeries> = all_monomials
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

/// Express a target series as a homogeneous degree-d combination of basis series.
///
/// Generates all degree-d monomials in `basis`, prepends `f` to the candidate list,
/// builds a coefficient matrix, and finds a null space vector with nonzero f-component.
/// Returns the combination coefficients for each monomial.
///
/// # Arguments
///
/// - `f`: the target series to express
/// - `basis`: the basis series
/// - `degree`: the total degree of the combination
/// - `topshift`: extra rows for overdetermination
///
/// # Returns
///
/// `Some(coefficients)` where `f = sum_i coefficients[i] * monomial_i`, with monomials
/// in the order returned by `generate_monomials(basis.len(), degree)`. Returns `None`
/// if no such expression exists.
pub fn findhomcombo(
    f: &FormalPowerSeries,
    basis: &[&FormalPowerSeries],
    degree: i64,
    topshift: i64,
) -> Option<Vec<QRat>> {
    let k = basis.len();
    if k == 0 || degree < 0 {
        return None;
    }

    // Generate all degree-d monomials in basis
    let monomials = generate_monomials(k, degree);
    let num_monomials = monomials.len();

    if num_monomials == 0 {
        return None;
    }

    // Compute the FPS for each monomial
    let monomial_series: Vec<FormalPowerSeries> = monomials
        .iter()
        .map(|exps| compute_monomial_series(basis, exps))
        .collect();

    // Build candidates list: [f, monomial_0, monomial_1, ...]
    let mut candidates: Vec<&FormalPowerSeries> = Vec::with_capacity(num_monomials + 1);
    candidates.push(f);
    for ms in &monomial_series {
        candidates.push(ms);
    }

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

    // Look for a null space vector with nonzero first component (for f)
    for v in &null_space {
        if !v[0].is_zero() {
            // Normalize so first component = 1, then negate remaining
            let scale = QRat::one() / v[0].clone();
            let coefficients: Vec<QRat> = v[1..]
                .iter()
                .map(|c| -(c.clone() * scale.clone()))
                .collect();
            return Some(coefficients);
        }
    }

    None
}

/// Express a target series as a non-homogeneous degree <= d combination of basis series.
///
/// Like [`findhomcombo`] but uses monomials of degree 0, 1, ..., d instead of
/// exactly degree d. This allows discovering affine and mixed-degree expressions.
///
/// # Arguments
///
/// - `f`: the target series to express
/// - `basis`: the basis series
/// - `degree`: the maximum total degree
/// - `topshift`: extra rows for overdetermination
///
/// # Returns
///
/// `Some(coefficients)` where `f = sum_i coefficients[i] * monomial_i`, with monomials
/// covering all degrees 0..=d. Returns `None` if no such expression exists.
pub fn findnonhomcombo(
    f: &FormalPowerSeries,
    basis: &[&FormalPowerSeries],
    degree: i64,
    topshift: i64,
) -> Option<Vec<QRat>> {
    let k = basis.len();
    if k == 0 || degree < 0 {
        return None;
    }

    // Generate all monomials of degree 0, 1, ..., degree
    let mut all_monomials: Vec<Vec<i64>> = Vec::new();
    for d in 0..=degree {
        let monos = generate_monomials(k, d);
        all_monomials.extend(monos);
    }

    let num_monomials = all_monomials.len();
    if num_monomials == 0 {
        return None;
    }

    // Compute the FPS for each monomial
    let monomial_series: Vec<FormalPowerSeries> = all_monomials
        .iter()
        .map(|exps| compute_monomial_series(basis, exps))
        .collect();

    // Build candidates list: [f, monomial_0, monomial_1, ...]
    let mut candidates: Vec<&FormalPowerSeries> = Vec::with_capacity(num_monomials + 1);
    candidates.push(f);
    for ms in &monomial_series {
        candidates.push(ms);
    }

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

    // Look for a null space vector with nonzero first component (for f)
    for v in &null_space {
        if !v[0].is_zero() {
            // Normalize so first component = 1, then negate remaining
            let scale = QRat::one() / v[0].clone();
            let coefficients: Vec<QRat> = v[1..]
                .iter()
                .map(|c| -(c.clone() * scale.clone()))
                .collect();
            return Some(coefficients);
        }
    }

    None
}
