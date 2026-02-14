//! Rational and modular linear algebra for q-series relation discovery.
//!
//! Provides Gaussian elimination over Q (exact arithmetic via QRat) and over Z/pZ,
//! plus null space computation and coefficient matrix building from formal power series.
//!
//! These routines form the shared foundation for all relation discovery functions:
//! `findlincombo`, `findhom`, `findpoly`, `findcong`, etc.

use crate::number::QRat;
use crate::series::FormalPowerSeries;

/// Compute the null space (kernel) of a matrix over Q using exact rational arithmetic.
///
/// Input: `matrix` is an m x n matrix (m rows, n columns) given as a slice of row vectors.
/// Returns: a list of basis vectors for ker(A), where A * v = 0 for each returned v.
///
/// Algorithm: Row-reduce to RREF (Reduced Row Echelon Form) with partial pivoting,
/// then extract free variables and build the null space basis.
///
/// Edge cases:
/// - Empty matrix or matrix with 0 columns: returns empty
/// - Full-rank matrix: returns empty (trivial kernel)
/// - Zero matrix: returns standard basis (all columns free)
pub fn rational_null_space(matrix: &[Vec<QRat>]) -> Vec<Vec<QRat>> {
    if matrix.is_empty() {
        return Vec::new();
    }
    let m = matrix.len();
    let n = matrix[0].len();
    if n == 0 {
        return Vec::new();
    }

    // Copy matrix for in-place reduction
    let mut a: Vec<Vec<QRat>> = matrix.to_vec();

    // Track which column is the pivot for each row (or None)
    let mut pivot_cols: Vec<usize> = Vec::new();
    let mut pivot_row = 0;

    // Forward elimination with partial pivoting -> RREF
    for col in 0..n {
        if pivot_row >= m {
            break;
        }

        // Find a row with nonzero entry in this column (at or below pivot_row)
        let mut found = None;
        for row in pivot_row..m {
            if !a[row][col].is_zero() {
                found = Some(row);
                break;
            }
        }

        let some_row = match found {
            Some(r) => r,
            None => continue, // This column is free
        };

        // Swap to pivot position
        if some_row != pivot_row {
            a.swap(some_row, pivot_row);
        }

        // Scale pivot row to make pivot = 1
        let pivot_val = a[pivot_row][col].clone();
        for j in 0..n {
            let val = a[pivot_row][j].clone();
            a[pivot_row][j] = &val / &pivot_val;
        }

        // Eliminate all other entries in this column
        for row in 0..m {
            if row == pivot_row {
                continue;
            }
            if a[row][col].is_zero() {
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

    // Free variables: columns NOT in pivot_cols
    let pivot_set: std::collections::HashSet<usize> = pivot_cols.iter().copied().collect();
    let free_cols: Vec<usize> = (0..n).filter(|c| !pivot_set.contains(c)).collect();

    if free_cols.is_empty() {
        return Vec::new(); // Full rank, trivial kernel
    }

    // Build a map from pivot column -> row index in RREF
    let mut pivot_col_to_row: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (row_idx, &col) in pivot_cols.iter().enumerate() {
        pivot_col_to_row.insert(col, row_idx);
    }

    // Build null space basis: for each free column fc, create a basis vector
    let mut basis = Vec::new();
    for &fc in &free_cols {
        let mut v = vec![QRat::zero(); n];
        v[fc] = QRat::one();

        // For each pivot column, the corresponding entry is -a[row][fc]
        for &pc in &pivot_cols {
            let row = pivot_col_to_row[&pc];
            v[pc] = -a[row][fc].clone();
        }

        basis.push(v);
    }

    basis
}

/// Build a coefficient matrix from candidate formal power series.
///
/// Each column corresponds to a candidate series, each row to a coefficient index.
/// Row i corresponds to exponent `start_order + i`.
/// Entry [i][j] = candidates[j].coeff(start_order + i).
///
/// Panics if any candidate series has insufficient truncation order
/// (i.e., start_order + num_rows > truncation_order).
pub fn build_coefficient_matrix(
    candidates: &[&FormalPowerSeries],
    start_order: i64,
    num_rows: usize,
) -> Vec<Vec<QRat>> {
    // Validate truncation orders
    let min_required = start_order + num_rows as i64;
    for (j, fps) in candidates.iter().enumerate() {
        assert!(
            fps.truncation_order() >= min_required,
            "Candidate series {} has truncation_order {} but need at least {} (start_order={}, num_rows={})",
            j,
            fps.truncation_order(),
            min_required,
            start_order,
            num_rows
        );
    }

    let mut matrix = Vec::with_capacity(num_rows);
    for i in 0..num_rows {
        let exp = start_order + i as i64;
        let row: Vec<QRat> = candidates.iter().map(|fps| fps.coeff(exp)).collect();
        matrix.push(row);
    }

    matrix
}

/// Compute the null space (kernel) of a matrix over Z/pZ.
///
/// Input: `matrix` is an m x n matrix over Z/pZ (entries assumed to be in [0, p)).
/// `p` must be a prime.
/// Returns: a list of basis vectors for ker(A) mod p.
///
/// Algorithm: Same RREF approach as `rational_null_space`, but with all arithmetic mod p.
pub fn modular_null_space(matrix: &[Vec<i64>], p: i64) -> Vec<Vec<i64>> {
    if matrix.is_empty() {
        return Vec::new();
    }
    let m = matrix.len();
    let n = matrix[0].len();
    if n == 0 {
        return Vec::new();
    }

    // Copy and normalize to [0, p)
    let mut a: Vec<Vec<i64>> = matrix
        .iter()
        .map(|row| row.iter().map(|&x| ((x % p) + p) % p).collect())
        .collect();

    let mut pivot_cols: Vec<usize> = Vec::new();
    let mut pivot_row = 0;

    for col in 0..n {
        if pivot_row >= m {
            break;
        }

        // Find nonzero entry
        let mut found = None;
        for row in pivot_row..m {
            if a[row][col] != 0 {
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

        // Scale pivot row: multiply by modular inverse of pivot
        let pivot_val = a[pivot_row][col];
        let inv = mod_inv(pivot_val, p);
        for j in 0..n {
            a[pivot_row][j] = (a[pivot_row][j] * inv) % p;
        }

        // Eliminate all other entries in this column
        for row in 0..m {
            if row == pivot_row {
                continue;
            }
            if a[row][col] == 0 {
                continue;
            }
            let factor = a[row][col];
            for j in 0..n {
                a[row][j] = ((a[row][j] - factor * a[pivot_row][j]) % p + p) % p;
            }
        }

        pivot_cols.push(col);
        pivot_row += 1;
    }

    // Free columns
    let pivot_set: std::collections::HashSet<usize> = pivot_cols.iter().copied().collect();
    let free_cols: Vec<usize> = (0..n).filter(|c| !pivot_set.contains(c)).collect();

    if free_cols.is_empty() {
        return Vec::new();
    }

    let mut pivot_col_to_row: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (row_idx, &col) in pivot_cols.iter().enumerate() {
        pivot_col_to_row.insert(col, row_idx);
    }

    let mut basis = Vec::new();
    for &fc in &free_cols {
        let mut v = vec![0i64; n];
        v[fc] = 1;

        for &pc in &pivot_cols {
            let row = pivot_col_to_row[&pc];
            // v[pc] = -a[row][fc] mod p
            v[pc] = ((-(a[row][fc])) % p + p) % p;
        }

        basis.push(v);
    }

    basis
}

/// Compute the modular inverse of `a` modulo `p` using Fermat's little theorem.
///
/// Since p is prime, a^{p-1} = 1 (mod p), so a^{-1} = a^{p-2} (mod p).
/// Panics if a = 0 (mod p).
fn mod_inv(a: i64, p: i64) -> i64 {
    let a = ((a % p) + p) % p;
    assert!(a != 0, "Cannot invert zero modulo {}", p);
    mod_pow(a, p - 2, p)
}

/// Modular exponentiation: compute base^exp mod modulus using fast exponentiation.
///
/// Uses the square-and-multiply algorithm. All intermediate results stay in range.
pub fn mod_pow(mut base: i64, mut exp: i64, modulus: i64) -> i64 {
    assert!(modulus > 0, "Modulus must be positive");
    if modulus == 1 {
        return 0;
    }
    let mut result: i64 = 1;
    base = ((base % modulus) + modulus) % modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = mod_mul(result, base, modulus);
        }
        exp >>= 1;
        base = mod_mul(base, base, modulus);
    }
    result
}

/// Safe modular multiplication that avoids overflow by using i128 intermediates.
fn mod_mul(a: i64, b: i64, modulus: i64) -> i64 {
    ((a as i128 * b as i128) % modulus as i128) as i64
}
