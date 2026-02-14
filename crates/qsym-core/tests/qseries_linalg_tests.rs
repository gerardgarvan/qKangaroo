//! Comprehensive tests for rational and modular linear algebra.
//!
//! Tests verify:
//! - rational_null_space on identity, rank-deficient, zero, and edge-case matrices
//! - Every null space vector v satisfies A * v = 0 exactly
//! - build_coefficient_matrix extracts FPS coefficients correctly
//! - modular_null_space on full-rank and singular matrices over Z/pZ

use qsym_core::number::QRat;
use qsym_core::qseries::{rational_null_space, build_coefficient_matrix, modular_null_space};
use qsym_core::series::FormalPowerSeries;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;

// ===========================================================================
// Helpers
// ===========================================================================

/// Create a QRat from a fraction (numerator, denominator).
fn qr(n: i64, d: i64) -> QRat {
    QRat::from((n, d))
}

/// Create a QRat from an integer.
fn qi(n: i64) -> QRat {
    QRat::from((n, 1i64))
}

/// Create a SymbolId for "q".
fn q_var() -> SymbolId {
    let mut arena = ExprArena::new();
    arena.symbols_mut().intern("q")
}

/// Verify that matrix * vector = 0 for a rational matrix and vector.
/// Panics with a descriptive message if any entry of the product is nonzero.
fn verify_null_vector(matrix: &[Vec<QRat>], v: &[QRat], label: &str) {
    for (i, row) in matrix.iter().enumerate() {
        let mut dot = QRat::zero();
        for (j, entry) in row.iter().enumerate() {
            dot = dot + entry.clone() * v[j].clone();
        }
        assert!(
            dot.is_zero(),
            "{}: A*v row {} is {:?}, expected 0 (v = {:?})",
            label,
            i,
            dot,
            v
        );
    }
}

/// Verify that matrix * vector = 0 (mod p) for a modular matrix and vector.
fn verify_modular_null_vector(matrix: &[Vec<i64>], v: &[i64], p: i64, label: &str) {
    for (i, row) in matrix.iter().enumerate() {
        let mut dot: i64 = 0;
        for (j, &entry) in row.iter().enumerate() {
            dot = ((dot + entry * v[j]) % p + p) % p;
        }
        assert!(
            dot == 0,
            "{}: A*v row {} is {} (mod {}), expected 0 (v = {:?})",
            label,
            i,
            dot,
            p,
            v
        );
    }
}

// ===========================================================================
// 1. Null space tests over Q
// ===========================================================================

/// Identity matrix: full rank, null space should be empty.
#[test]
fn test_null_space_identity() {
    let matrix = vec![
        vec![qi(1), qi(0), qi(0)],
        vec![qi(0), qi(1), qi(0)],
        vec![qi(0), qi(0), qi(1)],
    ];
    let ns = rational_null_space(&matrix);
    assert!(ns.is_empty(), "3x3 identity should have trivial kernel");
}

/// Rank-deficient matrix: [[1, 2, 3], [2, 4, 6]] has rank 1.
/// Null space should have dimension 2 and every basis vector should satisfy A*v = 0.
#[test]
fn test_null_space_rank_deficient() {
    let matrix = vec![
        vec![qi(1), qi(2), qi(3)],
        vec![qi(2), qi(4), qi(6)],
    ];
    let ns = rational_null_space(&matrix);
    assert_eq!(ns.len(), 2, "Rank-1 matrix of size 2x3 should have 2-dim null space");

    for (k, v) in ns.iter().enumerate() {
        verify_null_vector(&matrix, v, &format!("rank_deficient basis[{}]", k));
    }
}

/// Matrix with single relation: [[1, 0, 1], [0, 1, 2], [1, 1, 3]].
/// This has rank 2 (row 3 = row 1 + row 2), so null space dimension = 1.
/// The null space should give a vector proportional to [1, 2, -1].
#[test]
fn test_null_space_single_relation() {
    let matrix = vec![
        vec![qi(1), qi(0), qi(1)],
        vec![qi(0), qi(1), qi(2)],
        vec![qi(1), qi(1), qi(3)],
    ];
    let ns = rational_null_space(&matrix);
    assert_eq!(ns.len(), 1, "Rank-2 matrix of size 3x3 should have 1-dim null space");

    let v = &ns[0];
    verify_null_vector(&matrix, v, "single_relation");

    // Verify the null space vector is proportional to [1, 2, -1].
    // The algorithm sets the free variable (col 2) to 1, so v[2] = 1.
    // Then v[0] = -A_rref[0][2] = -1, v[1] = -A_rref[1][2] = -2.
    // So v = [-1, -2, 1], which is proportional to [1, 2, -1] (scaled by -1).
    // We just check A*v=0 which was already done; let's also verify proportionality.
    // Check that v[0]*2 == v[1] (the 1:2 ratio) or v[0]*(-1) == v[2]*1.
    // Safer: check that cross-ratios are consistent.
    assert!(
        !v[2].is_zero(),
        "Free variable column should have nonzero entry"
    );
    // v should satisfy: 1*v[0] + 0*v[1] + 1*v[2] = 0 => v[0] = -v[2]
    let sum = v[0].clone() + v[2].clone();
    assert!(sum.is_zero(), "v[0] + v[2] should be 0");
    // 0*v[0] + 1*v[1] + 2*v[2] = 0 => v[1] = -2*v[2]
    let sum2 = v[1].clone() + qi(2) * v[2].clone();
    assert!(sum2.is_zero(), "v[1] + 2*v[2] should be 0");
}

/// Empty input: should return empty null space.
#[test]
fn test_null_space_empty_matrix() {
    let matrix: Vec<Vec<QRat>> = Vec::new();
    let ns = rational_null_space(&matrix);
    assert!(ns.is_empty(), "Empty matrix should have empty null space");
}

/// Zero matrix: all columns are free, null space = standard basis.
#[test]
fn test_null_space_zero_matrix() {
    let matrix = vec![
        vec![qi(0), qi(0), qi(0)],
        vec![qi(0), qi(0), qi(0)],
    ];
    let ns = rational_null_space(&matrix);
    assert_eq!(ns.len(), 3, "2x3 zero matrix should have 3-dim null space (all cols free)");

    for (k, v) in ns.iter().enumerate() {
        verify_null_vector(&matrix, v, &format!("zero_matrix basis[{}]", k));
    }

    // Check that basis vectors are linearly independent (standard basis e_0, e_1, e_2)
    for (k, v) in ns.iter().enumerate() {
        assert!(
            !v[k].is_zero(),
            "Basis vector {} should have nonzero entry at position {}",
            k,
            k
        );
    }
}

/// Full-rank matrix with rational entries: [[1/2, 1/3], [1, 2/3]].
/// Determinant = (1/2)(2/3) - (1/3)(1) = 1/3 - 1/3 = 0.
/// Wait -- that's singular. Let me use [[1/2, 1/3], [1, 1]] instead.
/// Determinant = 1/2 - 1/3 = 1/6 != 0. Full rank, null space empty.
#[test]
fn test_null_space_rational_entries() {
    let matrix = vec![
        vec![qr(1, 2), qr(1, 3)],
        vec![qi(1), qi(1)],
    ];
    let ns = rational_null_space(&matrix);
    assert!(
        ns.is_empty(),
        "Full-rank 2x2 rational matrix should have empty null space"
    );
}

/// Singular 2x2 rational matrix: [[1/2, 1/3], [1, 2/3]].
/// Det = (1/2)(2/3) - (1/3)(1) = 1/3 - 1/3 = 0, so rank 1.
/// Null space should have dimension 1.
#[test]
fn test_null_space_singular_rational() {
    let matrix = vec![
        vec![qr(1, 2), qr(1, 3)],
        vec![qi(1), qr(2, 3)],
    ];
    let ns = rational_null_space(&matrix);
    assert_eq!(ns.len(), 1, "Singular 2x2 rational matrix should have 1-dim null space");
    verify_null_vector(&matrix, &ns[0], "singular_rational");
}

/// Wide matrix (more columns than rows): [[1, 0, 2, 1], [0, 1, 1, 3]].
/// Rank = 2, n = 4, so null space dimension = 2.
#[test]
fn test_null_space_wide_matrix() {
    let matrix = vec![
        vec![qi(1), qi(0), qi(2), qi(1)],
        vec![qi(0), qi(1), qi(1), qi(3)],
    ];
    let ns = rational_null_space(&matrix);
    assert_eq!(ns.len(), 2, "2x4 rank-2 matrix should have 2-dim null space");

    for (k, v) in ns.iter().enumerate() {
        verify_null_vector(&matrix, v, &format!("wide_matrix basis[{}]", k));
    }
}

// ===========================================================================
// 2. build_coefficient_matrix tests
// ===========================================================================

/// Create two known FPS and verify the coefficient matrix.
/// f1 = 1 + q + q^2 + O(q^5)
/// f2 = 1 + 2q + 3q^2 + O(q^5)
/// Matrix from start_order=0, num_rows=3:
///   [[1, 1], [1, 2], [1, 3]]
#[test]
fn test_build_coeff_matrix() {
    let q = q_var();

    // Build f1 = 1 + q + q^2
    let mut f1 = FormalPowerSeries::zero(q, 5);
    f1.set_coeff(0, qi(1));
    f1.set_coeff(1, qi(1));
    f1.set_coeff(2, qi(1));

    // Build f2 = 1 + 2q + 3q^2
    let mut f2 = FormalPowerSeries::zero(q, 5);
    f2.set_coeff(0, qi(1));
    f2.set_coeff(1, qi(2));
    f2.set_coeff(2, qi(3));

    let mat = build_coefficient_matrix(&[&f1, &f2], 0, 3);

    assert_eq!(mat.len(), 3, "Should have 3 rows");
    assert_eq!(mat[0].len(), 2, "Should have 2 columns");

    // Row 0: exponent 0
    assert_eq!(mat[0][0], qi(1));
    assert_eq!(mat[0][1], qi(1));
    // Row 1: exponent 1
    assert_eq!(mat[1][0], qi(1));
    assert_eq!(mat[1][1], qi(2));
    // Row 2: exponent 2
    assert_eq!(mat[2][0], qi(1));
    assert_eq!(mat[2][1], qi(3));
}

/// Test build_coefficient_matrix with nonzero start_order.
#[test]
fn test_build_coeff_matrix_offset() {
    let q = q_var();

    // Build f = q^2 + 3q^3 + 7q^4 + O(q^10)
    let mut f = FormalPowerSeries::zero(q, 10);
    f.set_coeff(2, qi(1));
    f.set_coeff(3, qi(3));
    f.set_coeff(4, qi(7));

    let mat = build_coefficient_matrix(&[&f], 2, 3);

    assert_eq!(mat.len(), 3);
    assert_eq!(mat[0][0], qi(1));  // coeff of q^2
    assert_eq!(mat[1][0], qi(3));  // coeff of q^3
    assert_eq!(mat[2][0], qi(7));  // coeff of q^4
}

// ===========================================================================
// 3. Modular null space tests
// ===========================================================================

/// Full-rank matrix mod 5: [[1, 2], [3, 4]].
/// Determinant = 4 - 6 = -2 = 3 (mod 5) != 0. Null space should be empty.
#[test]
fn test_modular_null_space_mod5() {
    let matrix = vec![
        vec![1, 2],
        vec![3, 4],
    ];
    let ns = modular_null_space(&matrix, 5);
    assert!(ns.is_empty(), "Full-rank 2x2 mod 5 should have empty null space");
}

/// Singular matrix mod 7: [[1, 2, 3], [2, 4, 6]].
/// Row 2 = 2 * Row 1, so rank 1. Null space dimension = 2.
#[test]
fn test_modular_null_space_singular_mod7() {
    let matrix = vec![
        vec![1, 2, 3],
        vec![2, 4, 6],
    ];
    let ns = modular_null_space(&matrix, 7);
    assert_eq!(ns.len(), 2, "Rank-1 matrix 2x3 mod 7 should have 2-dim null space");

    for (k, v) in ns.iter().enumerate() {
        verify_modular_null_vector(&matrix, v, 7, &format!("mod7_singular basis[{}]", k));
    }
}

/// Identity matrix mod 11: full rank, null space empty.
#[test]
fn test_modular_null_space_identity_mod11() {
    let matrix = vec![
        vec![1, 0, 0],
        vec![0, 1, 0],
        vec![0, 0, 1],
    ];
    let ns = modular_null_space(&matrix, 11);
    assert!(ns.is_empty(), "3x3 identity mod 11 should have empty null space");
}

/// Zero matrix mod 5: all columns free.
#[test]
fn test_modular_null_space_zero_mod5() {
    let matrix = vec![
        vec![0, 0],
        vec![0, 0],
    ];
    let ns = modular_null_space(&matrix, 5);
    assert_eq!(ns.len(), 2, "2x2 zero matrix mod 5 should have 2-dim null space");

    for (k, v) in ns.iter().enumerate() {
        verify_modular_null_vector(&matrix, v, 5, &format!("mod5_zero basis[{}]", k));
    }
}

// ===========================================================================
// 4. Integration: null space of coefficient matrix
// ===========================================================================

/// Build a coefficient matrix from series that have a known linear relation,
/// then verify that rational_null_space finds it.
/// f1 = 1 + q + q^2, f2 = 0 + q + 2q^2, f3 = f1 + f2 = 1 + 2q + 3q^2.
/// The columns are linearly independent pairwise, but f3 = f1 + f2.
/// Relation: f1 + f2 - f3 = 0, so null space should have dimension 1.
#[test]
fn test_null_space_of_coefficient_matrix() {
    let q = q_var();

    // f1 = 1 + q + q^2
    let mut f1 = FormalPowerSeries::zero(q, 10);
    f1.set_coeff(0, qi(1));
    f1.set_coeff(1, qi(1));
    f1.set_coeff(2, qi(1));

    // f2 = 0 + q + 2q^2 (different from f1!)
    let mut f2 = FormalPowerSeries::zero(q, 10);
    f2.set_coeff(1, qi(1));
    f2.set_coeff(2, qi(2));

    // f3 = f1 + f2 = 1 + 2q + 3q^2
    let mut f3 = FormalPowerSeries::zero(q, 10);
    f3.set_coeff(0, qi(1));
    f3.set_coeff(1, qi(2));
    f3.set_coeff(2, qi(3));

    let mat = build_coefficient_matrix(&[&f1, &f2, &f3], 0, 3);
    let ns = rational_null_space(&mat);

    // Matrix columns: [1,1,1], [0,1,2], [1,2,3]. Rank 2, null dim = 1.
    assert_eq!(ns.len(), 1, "Exactly one linear relation among three series");

    let v = &ns[0];
    verify_null_vector(&mat, v, "coefficient_matrix_integration");

    // The relation is f1 + f2 - f3 = 0, so v should be proportional to [1, 1, -1].
    // Verify: v[0]/v[2] = -1 and v[1]/v[2] = -1 (since v[2] is the free variable entry).
    assert!(!v[2].is_zero(), "Free variable entry should be nonzero");
    let ratio_0 = v[0].clone() + v[2].clone();
    let ratio_1 = v[1].clone() + v[2].clone();
    assert!(ratio_0.is_zero(), "v[0] + v[2] should be 0");
    assert!(ratio_1.is_zero(), "v[1] + v[2] should be 0");
}
