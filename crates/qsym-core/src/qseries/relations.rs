//! Relation discovery functions for q-series.
//!
//! Provides the core relation discovery tools from Garvan's qseries package:
//! - [`findlincombo`]: find f as a linear combination of basis series
//! - [`findhom`]: find homogeneous polynomial relations among series
//! - [`findpoly`]: find a two-variable polynomial relation P(x,y) = 0
//! - [`findlincombomodp`]: find linear combination mod a prime p
//! - [`findhommodp`]: find homogeneous relations mod p
//! - [`findhomcombomodp`]: express target as homogeneous combo mod p
//! - [`findmaxind`]: find maximal linearly independent subset
//! - [`findprod`]: search for linear combinations with nice product forms
//!
//! All functions follow the coefficient-matrix + null-space pattern:
//! 1. Build candidate series (monomials in the input series)
//! 2. Extract coefficients into a matrix
//! 3. Compute the rational null space
//! 4. Interpret null space vectors as relations

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use super::linalg::{build_coefficient_matrix, rational_null_space, modular_null_space};
use super::prodmake::prodmake;
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

// ===========================================================================
// Modular arithmetic helpers for modp variants
// ===========================================================================

/// Compute modular inverse via Fermat's little theorem: a^{p-2} mod p.
fn mod_inv_local(a: i64, p: i64) -> i64 {
    let a = ((a % p) + p) % p;
    assert!(a != 0, "Cannot invert zero modulo {}", p);
    mod_pow_local(a, p - 2, p)
}

/// Fast modular exponentiation.
fn mod_pow_local(mut base: i64, mut exp: i64, modulus: i64) -> i64 {
    if modulus == 1 {
        return 0;
    }
    let mut result: i64 = 1;
    base = ((base % modulus) + modulus) % modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = ((result as i128 * base as i128) % modulus as i128) as i64;
        }
        exp >>= 1;
        base = ((base as i128 * base as i128) % modulus as i128) as i64;
    }
    result
}

/// Convert a QRat coefficient to i64 mod p.
///
/// For a/b, computes a * b^{-1} mod p. Returns None if b = 0 mod p.
fn qrat_to_mod_p(c: &QRat, p: i64) -> Option<i64> {
    use rug::Integer;

    let p_int = Integer::from(p);

    let numer = c.numer();
    let denom = c.denom();

    // Check if denominator is divisible by p
    if denom.is_divisible(&p_int) {
        return None;
    }

    // Convert numerator mod p
    let n_mod = {
        let r = Integer::from(numer % &p_int);
        let val = r.to_i64().unwrap_or(0);
        ((val % p) + p) % p
    };

    // Convert denominator mod p
    let d_mod = {
        let r = Integer::from(denom % &p_int);
        let val = r.to_i64().unwrap_or(0);
        ((val % p) + p) % p
    };

    // a * b^{-1} mod p
    let d_inv = mod_inv_local(d_mod, p);
    Some(((n_mod as i128 * d_inv as i128) % p as i128) as i64)
}

/// Build a coefficient matrix over Z/pZ from candidate formal power series.
///
/// Each column is a candidate series, each row a coefficient index.
/// Coefficients are converted to Z/pZ via `qrat_to_mod_p`.
/// Returns None if any coefficient has a denominator divisible by p.
fn build_modp_coefficient_matrix(
    candidates: &[&FormalPowerSeries],
    start_order: i64,
    num_rows: usize,
    p: i64,
) -> Option<Vec<Vec<i64>>> {
    let mut matrix = Vec::with_capacity(num_rows);
    for i in 0..num_rows {
        let exp = start_order + i as i64;
        let mut row = Vec::with_capacity(candidates.len());
        for fps in candidates {
            let c = fps.coeff(exp);
            match qrat_to_mod_p(&c, p) {
                Some(val) => row.push(val),
                None => return None,
            }
        }
        matrix.push(row);
    }
    Some(matrix)
}

// ===========================================================================
// findlincombomodp
// ===========================================================================

/// Find a linear combination of basis series that equals f, working mod p.
///
/// Like [`findlincombo`] but all arithmetic is performed over Z/pZ.
/// Returns coefficients mod p if f can be expressed as a linear combination
/// of the basis series modulo p.
///
/// # Arguments
///
/// - `f`: the target series
/// - `basis`: the basis series
/// - `p`: a prime modulus
/// - `topshift`: extra rows for overdetermination
///
/// # Returns
///
/// `Some(coefficients)` where `f = sum_i coefficients[i] * basis[i] (mod p)`,
/// or `None` if no such combination exists mod p.
pub fn findlincombomodp(
    f: &FormalPowerSeries,
    basis: &[&FormalPowerSeries],
    p: i64,
    topshift: i64,
) -> Option<Vec<i64>> {
    if basis.is_empty() {
        if f.is_zero() {
            return Some(Vec::new());
        }
        return None;
    }

    // Build candidates: [f, basis[0], ..., basis[k-1]]
    let mut candidates: Vec<&FormalPowerSeries> = Vec::with_capacity(basis.len() + 1);
    candidates.push(f);
    candidates.extend_from_slice(basis);

    let num_candidates = candidates.len();

    let start_order = candidates
        .iter()
        .filter_map(|fps| fps.min_order())
        .min()
        .unwrap_or(0)
        .min(0);

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

    // Build modular coefficient matrix
    let matrix = build_modp_coefficient_matrix(&candidates, start_order, num_rows, p)?;
    let null_space = modular_null_space(&matrix, p);

    // Look for a null space vector with nonzero first component
    for v in &null_space {
        if v[0] != 0 {
            // Normalize so first component = 1
            let inv = mod_inv_local(v[0], p);
            // Coefficients are the negatives of the remaining components
            let coefficients: Vec<i64> = v[1..]
                .iter()
                .map(|&c| ((-(c as i128 * inv as i128) % p as i128) + p as i128) as i64 % p)
                .collect();
            return Some(coefficients);
        }
    }

    None
}

// ===========================================================================
// findhommodp
// ===========================================================================

/// Find all homogeneous degree-d polynomial relations among series, working mod p.
///
/// Like [`findhom`] but all arithmetic is performed over Z/pZ.
///
/// # Arguments
///
/// - `series`: the series to find relations among
/// - `p`: a prime modulus
/// - `degree`: the total degree of the homogeneous polynomial
/// - `topshift`: extra rows for overdetermination
///
/// # Returns
///
/// A vector of relation vectors over Z/pZ.
pub fn findhommodp(
    series: &[&FormalPowerSeries],
    p: i64,
    degree: i64,
    topshift: i64,
) -> Vec<Vec<i64>> {
    let k = series.len();
    if k == 0 || degree < 0 {
        return Vec::new();
    }

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

    let start_order = candidates
        .iter()
        .filter_map(|fps| fps.min_order())
        .min()
        .unwrap_or(0)
        .min(0);

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

    match build_modp_coefficient_matrix(&candidates, start_order, num_rows, p) {
        Some(matrix) => modular_null_space(&matrix, p),
        None => Vec::new(),
    }
}

// ===========================================================================
// findhomcombomodp
// ===========================================================================

/// Express a target series as a homogeneous degree-d combination of basis series, mod p.
///
/// Like [`findhomcombo`] but all arithmetic is performed over Z/pZ.
///
/// # Arguments
///
/// - `f`: the target series to express
/// - `basis`: the basis series
/// - `p`: a prime modulus
/// - `degree`: the total degree of the combination
/// - `topshift`: extra rows for overdetermination
///
/// # Returns
///
/// `Some(coefficients)` mod p for the degree-d monomials, or `None`.
pub fn findhomcombomodp(
    f: &FormalPowerSeries,
    basis: &[&FormalPowerSeries],
    p: i64,
    degree: i64,
    topshift: i64,
) -> Option<Vec<i64>> {
    let k = basis.len();
    if k == 0 || degree < 0 {
        return None;
    }

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

    // Build candidates: [f, monomial_0, monomial_1, ...]
    let mut candidates: Vec<&FormalPowerSeries> = Vec::with_capacity(num_monomials + 1);
    candidates.push(f);
    for ms in &monomial_series {
        candidates.push(ms);
    }

    let num_candidates = candidates.len();

    let start_order = candidates
        .iter()
        .filter_map(|fps| fps.min_order())
        .min()
        .unwrap_or(0)
        .min(0);

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

    let matrix = build_modp_coefficient_matrix(&candidates, start_order, num_rows, p)?;
    let null_space = modular_null_space(&matrix, p);

    // Look for a null space vector with nonzero first component (for f)
    for v in &null_space {
        if v[0] != 0 {
            let inv = mod_inv_local(v[0], p);
            let coefficients: Vec<i64> = v[1..]
                .iter()
                .map(|&c| ((-(c as i128 * inv as i128) % p as i128) + p as i128) as i64 % p)
                .collect();
            return Some(coefficients);
        }
    }

    None
}

// ===========================================================================
// findmaxind
// ===========================================================================

/// Find the maximal linearly independent subset of the given series.
///
/// Builds a coefficient matrix with all series as columns, performs
/// Gaussian elimination to find pivot columns, and returns the indices
/// of the independent series.
///
/// # Arguments
///
/// - `series`: the series to analyze
/// - `topshift`: extra rows for overdetermination
///
/// # Returns
///
/// Indices of the maximal linearly independent subset of `series`.
pub fn findmaxind(
    series: &[&FormalPowerSeries],
    topshift: i64,
) -> Vec<usize> {
    let k = series.len();
    if k == 0 {
        return Vec::new();
    }

    let start_order = series
        .iter()
        .filter_map(|fps| fps.min_order())
        .min()
        .unwrap_or(0)
        .min(0);

    let max_trunc = series
        .iter()
        .map(|fps| fps.truncation_order())
        .min()
        .unwrap();

    let available_rows = (max_trunc - start_order) as usize;
    let desired_rows = k + topshift as usize;
    let num_rows = desired_rows.min(available_rows);

    if num_rows == 0 {
        return Vec::new();
    }

    // Build coefficient matrix: each column is a series
    let matrix = build_coefficient_matrix(&series.to_vec(), start_order, num_rows);

    // Perform Gaussian elimination to find pivot columns
    let m = matrix.len();
    let n = matrix[0].len();

    let mut a: Vec<Vec<QRat>> = matrix;
    let mut pivot_cols: Vec<usize> = Vec::new();
    let mut pivot_row = 0;

    for col in 0..n {
        if pivot_row >= m {
            break;
        }

        // Find a row with nonzero entry in this column
        let mut found = None;
        for row in pivot_row..m {
            if !a[row][col].is_zero() {
                found = Some(row);
                break;
            }
        }

        let some_row = match found {
            Some(r) => r,
            None => continue,
        };

        if some_row != pivot_row {
            a.swap(some_row, pivot_row);
        }

        // Scale pivot row
        let pivot_val = a[pivot_row][col].clone();
        for j in 0..n {
            let val = a[pivot_row][j].clone();
            a[pivot_row][j] = &val / &pivot_val;
        }

        // Eliminate other rows
        for row in 0..m {
            if row == pivot_row || a[row][col].is_zero() {
                continue;
            }
            let factor = a[row][col].clone();
            for j in 0..n {
                let sub = &factor * &a[pivot_row][j];
                let val = a[row][j].clone();
                a[row][j] = val - sub;
            }
        }

        pivot_cols.push(col);
        pivot_row += 1;
    }

    // Pivot columns correspond to independent series
    pivot_cols
}

// ===========================================================================
// findprod
// ===========================================================================

/// Search for linear combinations of series that yield nice product forms.
///
/// For each integer linear combination (with coefficients bounded by `max_coeff`),
/// computes the resulting series and uses [`prodmake`] to check if it has a
/// "nice" infinite product form (integer exponents).
///
/// This is fundamentally a brute-force search bounded by `max_coeff` and
/// `max_exp` parameters.
///
/// # Arguments
///
/// - `series`: the input series to combine
/// - `max_coeff`: maximum absolute value of combination coefficients
/// - `max_exp`: maximum exponent to check in prodmake
///
/// # Returns
///
/// A vector of coefficient vectors, where each inner vector gives the integer
/// linear combination coefficients that produce a series with a nice product form.
pub fn findprod(
    series: &[&FormalPowerSeries],
    max_coeff: i64,
    max_exp: i64,
) -> Vec<Vec<i64>> {
    let k = series.len();
    if k == 0 {
        return Vec::new();
    }

    let mut results: Vec<Vec<i64>> = Vec::new();

    // Generate all coefficient vectors with entries in [-max_coeff, max_coeff]
    // Skip the all-zero vector
    let mut coeffs = vec![-(max_coeff); k];

    loop {
        // Skip the all-zero vector
        if coeffs.iter().any(|&c| c != 0) {
            // Compute the linear combination
            let combo = compute_linear_combination(series, &coeffs);

            // Skip if the combination is zero
            if !combo.is_zero() {
                // Use prodmake to check if it has a nice product form
                if has_nice_product_form(&combo, max_exp) {
                    results.push(coeffs.clone());
                }
            }
        }

        // Increment coefficients (odometer-style)
        if !increment_coeffs(&mut coeffs, max_coeff) {
            break;
        }
    }

    results
}

/// Compute a linear combination: sum_i coeffs[i] * series[i].
fn compute_linear_combination(
    series: &[&FormalPowerSeries],
    coeffs: &[i64],
) -> FormalPowerSeries {
    assert_eq!(series.len(), coeffs.len());

    let trunc = series.iter().map(|s| s.truncation_order()).min().unwrap();
    let var = series[0].variable();
    let mut result = FormalPowerSeries::zero(var, trunc);

    for (s, &c) in series.iter().zip(coeffs.iter()) {
        if c == 0 {
            continue;
        }
        let scaled = arithmetic::scalar_mul(&QRat::from((c, 1i64)), s);
        result = arithmetic::add(&result, &scaled);
    }

    result
}

/// Check if a series has a "nice" product form by running prodmake
/// and verifying that all exponents are integers.
fn has_nice_product_form(f: &FormalPowerSeries, max_exp: i64) -> bool {
    let product = prodmake(f, max_exp);

    if product.exponents.is_empty() {
        return false;
    }

    // Check all exponents are integers (denominator = 1)
    let one = rug::Integer::from(1);
    product.exponents.values().all(|exp| exp.denom() == &one)
}

/// Increment coefficient vector in odometer fashion from [-max, ..., -max] to [max, ..., max].
/// Returns false when overflow (all done).
fn increment_coeffs(coeffs: &mut [i64], max_coeff: i64) -> bool {
    for c in coeffs.iter_mut().rev() {
        *c += 1;
        if *c <= max_coeff {
            return true;
        }
        *c = -max_coeff;
    }
    false
}
