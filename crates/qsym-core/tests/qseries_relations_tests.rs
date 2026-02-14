//! Comprehensive tests for relation discovery functions.
//!
//! Tests verify:
//! - findlincombo discovers known linear combinations
//! - findlincombo returns None when no relation exists
//! - findhom discovers homogeneous polynomial relations among series
//! - findpoly discovers two-variable polynomial relations
//! - PolynomialRelation has correct degree fields

use qsym_core::number::QRat;
use qsym_core::qseries::{
    findlincombo, findhom, findpoly, theta3, theta4,
    findcong, findnonhom, findhomcombo, findnonhomcombo, partition_gf,
};
use qsym_core::series::{FormalPowerSeries, arithmetic};
use qsym_core::series::generator::InfiniteProductGenerator;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;

// ===========================================================================
// Helpers
// ===========================================================================

/// Create a QRat from an integer.
fn qi(n: i64) -> QRat {
    QRat::from((n, 1i64))
}

/// Create a SymbolId for "q".
fn q_var() -> SymbolId {
    let mut arena = ExprArena::new();
    arena.symbols_mut().intern("q")
}

/// Build an FPS from a list of (exponent, coefficient) pairs.
fn fps_from_pairs(variable: SymbolId, pairs: &[(i64, i64)], trunc: i64) -> FormalPowerSeries {
    let mut fps = FormalPowerSeries::zero(variable, trunc);
    for &(exp, coeff) in pairs {
        if exp < trunc {
            fps.set_coeff(exp, qi(coeff));
        }
    }
    fps
}

/// Compute theta2(q)^4 directly as a q-series using the product formula.
///
/// theta2(q) = 2*q^{1/4} * prod_{n>=1}(1-q^{2n})(1+q^{2n})^2
///
/// theta2(q)^4 = 16*q * prod_{n>=1}(1-q^{2n})^4 * (1+q^{2n})^8
fn theta2_fourth_power(variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let num_factors = (truncation_order + 1) / 2 + 1;

    // Factor 1: prod_{n>=1}(1-q^{2n})^4
    // Build (q^2;q^2)_inf first, then raise to 4th power
    let initial1 = FormalPowerSeries::one(variable, truncation_order);
    let mut ipg1 = InfiniteProductGenerator::new(
        initial1,
        1,
        Box::new(move |n, var, trunc| {
            let mut factor = FormalPowerSeries::one(var, trunc);
            factor.set_coeff(2 * n, -QRat::one());
            factor
        }),
    );
    ipg1.ensure_order(num_factors);
    let q2q2_inf = ipg1.into_series();
    let q2q2_inf_4 = {
        let sq = arithmetic::mul(&q2q2_inf, &q2q2_inf);
        arithmetic::mul(&sq, &sq)
    };

    // Factor 2: prod_{n>=1}(1+q^{2n})^8
    let initial2 = FormalPowerSeries::one(variable, truncation_order);
    let mut ipg2 = InfiniteProductGenerator::new(
        initial2,
        1,
        Box::new(move |n, var, trunc| {
            let mut factor = FormalPowerSeries::one(var, trunc);
            factor.set_coeff(2 * n, QRat::one());
            factor
        }),
    );
    ipg2.ensure_order(num_factors);
    let plus_q2n = ipg2.into_series();
    let plus_q2n_2 = arithmetic::mul(&plus_q2n, &plus_q2n);
    let plus_q2n_4 = arithmetic::mul(&plus_q2n_2, &plus_q2n_2);
    let plus_q2n_8 = arithmetic::mul(&plus_q2n_4, &plus_q2n_4);

    // theta2^4 = 16*q * factor1 * factor2
    let product = arithmetic::mul(&q2q2_inf_4, &plus_q2n_8);
    let prefactor = FormalPowerSeries::monomial(
        variable,
        QRat::from((16, 1i64)),
        1,
        truncation_order,
    );
    arithmetic::mul(&prefactor, &product)
}

// ===========================================================================
// findlincombo tests
// ===========================================================================

#[test]
fn test_findlincombo_exact() {
    // f1 = 1 + q + q^2, f2 = 1 + 2q + 3q^2
    // f3 = 3*f1 + 7*f2 = 10 + 17q + 24q^2
    let q = q_var();
    let trunc = 30;

    let f1 = fps_from_pairs(q, &[(0, 1), (1, 1), (2, 1)], trunc);
    let f2 = fps_from_pairs(q, &[(0, 1), (1, 2), (2, 3)], trunc);
    let f3 = arithmetic::add(
        &arithmetic::scalar_mul(&qi(3), &f1),
        &arithmetic::scalar_mul(&qi(7), &f2),
    );

    let result = findlincombo(&f3, &[&f1, &f2], 0);
    assert!(result.is_some(), "Should find a linear combination");

    let coeffs = result.unwrap();
    assert_eq!(coeffs.len(), 2);
    assert_eq!(coeffs[0], qi(3), "Coefficient of f1 should be 3");
    assert_eq!(coeffs[1], qi(7), "Coefficient of f2 should be 7");
}

#[test]
fn test_findlincombo_no_relation() {
    // Three linearly independent series: f3 is not a linear combo of f1, f2.
    // Use enough nonzero coefficients and topshift to make the system overdetermined.
    let q = q_var();
    let trunc = 30;

    let f1 = fps_from_pairs(q, &[(0, 1), (1, 1), (2, 0), (3, 0), (4, 0), (5, 1)], trunc);
    let f2 = fps_from_pairs(q, &[(0, 0), (1, 0), (2, 1), (3, 0), (4, 1), (5, 0)], trunc);
    let f3 = fps_from_pairs(q, &[(0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1)], trunc);

    // Use topshift=10 for overdetermined system
    let result = findlincombo(&f3, &[&f1, &f2], 10);
    assert!(result.is_none(), "Should not find a linear combination");
}

#[test]
fn test_findlincombo_single_basis() {
    // findlincombo(2*f, [f]) should return [2]
    let q = q_var();
    let trunc = 30;

    let f = fps_from_pairs(q, &[(0, 1), (1, 3), (2, 5), (3, 7)], trunc);
    let f_times_2 = arithmetic::scalar_mul(&qi(2), &f);

    let result = findlincombo(&f_times_2, &[&f], 0);
    assert!(result.is_some(), "Should find single-basis linear combination");

    let coeffs = result.unwrap();
    assert_eq!(coeffs.len(), 1);
    assert_eq!(coeffs[0], qi(2), "Coefficient should be 2");
}

#[test]
fn test_findlincombo_fractional_coefficients() {
    // f = (1/3)*g1 + (5/7)*g2
    let q = q_var();
    let trunc = 30;

    let g1 = fps_from_pairs(q, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)], trunc);
    let g2 = fps_from_pairs(q, &[(0, 2), (1, 1), (2, 4), (3, 3), (4, 6)], trunc);

    let c1 = QRat::from((1, 3i64));
    let c2 = QRat::from((5, 7i64));
    let f = arithmetic::add(
        &arithmetic::scalar_mul(&c1, &g1),
        &arithmetic::scalar_mul(&c2, &g2),
    );

    let result = findlincombo(&f, &[&g1, &g2], 0);
    assert!(result.is_some(), "Should find fractional linear combination");

    let coeffs = result.unwrap();
    assert_eq!(coeffs.len(), 2);
    assert_eq!(coeffs[0], QRat::from((1, 3i64)), "Coefficient of g1 should be 1/3");
    assert_eq!(coeffs[1], QRat::from((5, 7i64)), "Coefficient of g2 should be 5/7");
}

// ===========================================================================
// findhom tests
// ===========================================================================

#[test]
fn test_findhom_trivial_degree1() {
    // f and g = 2*f: the degree-1 monomials are [f, g], and f - (1/2)*g = 0
    let q = q_var();
    let trunc = 30;

    let f = fps_from_pairs(q, &[(0, 1), (1, 3), (2, 5), (3, 7), (4, 11)], trunc);
    let g = arithmetic::scalar_mul(&qi(2), &f);

    let relations = findhom(&[&f, &g], 1, 0);
    assert!(!relations.is_empty(), "Should find a degree-1 relation");

    // Monomial ordering from generate_monomials(2, 1): [(0,1), (1,0)]
    // So column 0 = g (series[1]^1), column 1 = f (series[0]^1)
    // The relation v[0]*g + v[1]*f = 0.
    // Since g = 2*f, we need v[0]*2f + v[1]*f = 0, i.e., 2*v[0] + v[1] = 0.
    // Null space gives v = [-1/2, 1] (or scalar multiple).
    let v = &relations[0];
    assert_eq!(v.len(), 2, "Should have 2 monomial coefficients");
    assert!(
        !v[0].is_zero() && !v[1].is_zero(),
        "Both components should be nonzero"
    );

    // Verify the relation holds: v[0]*g + v[1]*f should be zero
    let check = arithmetic::add(
        &arithmetic::scalar_mul(&v[0], &g),
        &arithmetic::scalar_mul(&v[1], &f),
    );
    assert!(check.is_zero(), "Relation v[0]*g + v[1]*f should equal zero");
}

#[test]
fn test_findhom_jacobi_theta() {
    // Jacobi identity: theta3(q)^4 = theta2(q)^4 + theta4(q)^4
    // Equivalently: theta2(q)^4 + theta4(q)^4 - theta3(q)^4 = 0
    //
    // We compute theta2^4 directly as a q-series, along with theta3 and theta4,
    // then use findhom to discover the relation among these three fourth powers.
    let q = q_var();
    let trunc = 50;

    // Compute the three fourth powers as q-series
    let t2_4 = theta2_fourth_power(q, trunc);
    let t3 = theta3(q, trunc);
    let t3_4 = {
        let sq = arithmetic::mul(&t3, &t3);
        arithmetic::mul(&sq, &sq)
    };
    let t4 = theta4(q, trunc);
    let t4_4 = {
        let sq = arithmetic::mul(&t4, &t4);
        arithmetic::mul(&sq, &sq)
    };

    // Use findhom with degree 1 on the fourth powers themselves:
    // We're looking for c1*t2_4 + c2*t3_4 + c3*t4_4 = 0
    // This is "degree 1" in the three input series (which are already fourth powers)
    let relations = findhom(&[&t2_4, &t3_4, &t4_4], 1, 0);
    assert!(
        !relations.is_empty(),
        "Should find the Jacobi theta identity"
    );

    let v = &relations[0];
    assert_eq!(v.len(), 3, "Should have 3 components");

    // Verify the relation: the null space vector should be proportional to [1, -1, 1]
    // (theta2^4 - theta3^4 + theta4^4 = 0)
    // Normalize by v[0] (which should be nonzero for this relation)
    assert!(!v[0].is_zero(), "First component should be nonzero");

    let scale = QRat::one() / v[0].clone();
    let normalized: Vec<QRat> = v.iter().map(|c| c.clone() * scale.clone()).collect();

    assert_eq!(normalized[0], qi(1), "Coefficient of theta2^4 should be 1");
    assert_eq!(normalized[1], qi(-1), "Coefficient of theta3^4 should be -1");
    assert_eq!(normalized[2], qi(1), "Coefficient of theta4^4 should be 1");
}

#[test]
fn test_findhom_no_relation() {
    // Two "random" independent series should have no degree-1 homogeneous relation
    let q = q_var();
    let trunc = 30;

    let f = fps_from_pairs(q, &[(0, 1), (1, 1), (2, 1)], trunc);
    let g = fps_from_pairs(q, &[(0, 1), (1, 2), (2, 5), (3, 3)], trunc);

    let relations = findhom(&[&f, &g], 1, 0);
    assert!(relations.is_empty(), "Should not find a degree-1 relation between independent series");
}

// ===========================================================================
// findpoly tests
// ===========================================================================

#[test]
fn test_findpoly_linear() {
    // Construct x, y where x = y + 1: P(x,y) = x - y - 1 = 0
    let q = q_var();
    let trunc = 30;

    let y = fps_from_pairs(q, &[(1, 1), (2, 2), (3, 3), (4, 4)], trunc);
    let x = arithmetic::add(&y, &FormalPowerSeries::one(q, trunc));

    let result = findpoly(&x, &y, 1, 1, 0);
    assert!(result.is_some(), "Should find linear polynomial relation");

    let rel = result.unwrap();
    assert_eq!(rel.deg_x, 1);
    assert_eq!(rel.deg_y, 1);

    // Verify the relation P(x,y) = 0:
    // coefficients[i][j] = coefficient of x^i * y^j
    // P(x,y) = c[0][0] + c[0][1]*y + c[1][0]*x + c[1][1]*x*y
    // We expect P proportional to x - y - 1 = 0, i.e., c[1][0]*x + c[0][1]*y + c[0][0]
    // with c[1][0] = 1, c[0][1] = -1, c[0][0] = -1, c[1][1] = 0 (up to scaling)

    // Normalize by the x coefficient (c[1][0])
    let x_coeff = &rel.coefficients[1][0];
    assert!(!x_coeff.is_zero(), "x coefficient should be nonzero");

    let scale = QRat::one() / x_coeff.clone();
    let c00 = rel.coefficients[0][0].clone() * scale.clone();
    let c01 = rel.coefficients[0][1].clone() * scale.clone();
    let c10 = rel.coefficients[1][0].clone() * scale.clone();
    let c11 = rel.coefficients[1][1].clone() * scale.clone();

    assert_eq!(c10, qi(1), "Coefficient of x should be 1");
    assert_eq!(c01, qi(-1), "Coefficient of y should be -1");
    assert_eq!(c00, qi(-1), "Constant term should be -1");
    assert_eq!(c11, qi(0), "Coefficient of x*y should be 0");
}

#[test]
fn test_findpoly_quadratic() {
    // Construct x, y where x^2 + y = 0 (i.e., y = -x^2)
    let q = q_var();
    let trunc = 30;

    let x = fps_from_pairs(q, &[(0, 1), (1, 1), (2, 1)], trunc);
    let x_squared = arithmetic::mul(&x, &x);
    let y = arithmetic::negate(&x_squared);

    let result = findpoly(&x, &y, 2, 1, 0);
    assert!(result.is_some(), "Should find quadratic polynomial relation");

    let rel = result.unwrap();
    assert_eq!(rel.deg_x, 2);
    assert_eq!(rel.deg_y, 1);

    // P(x,y) = x^2 + y = 0
    // coefficients[2][0] = 1 (x^2 term), coefficients[0][1] = 1 (y term)
    // All other coefficients should be 0

    // Find a nonzero coefficient to normalize
    let y_coeff = &rel.coefficients[0][1];
    assert!(!y_coeff.is_zero(), "y coefficient should be nonzero");

    let scale = QRat::one() / y_coeff.clone();

    // Check x^2 coefficient
    let c20 = rel.coefficients[2][0].clone() * scale.clone();
    let c01 = rel.coefficients[0][1].clone() * scale.clone();

    assert_eq!(c20, qi(1), "Coefficient of x^2 should equal coefficient of y");
    assert_eq!(c01, qi(1), "Coefficient of y should be 1 (normalized)");

    // All other coefficients should be 0
    assert_eq!(rel.coefficients[0][0].clone() * scale.clone(), qi(0), "Constant should be 0");
    assert_eq!(rel.coefficients[1][0].clone() * scale.clone(), qi(0), "x coefficient should be 0");
    assert_eq!(rel.coefficients[1][1].clone() * scale.clone(), qi(0), "x*y coefficient should be 0");
    assert_eq!(rel.coefficients[2][1].clone() * scale.clone(), qi(0), "x^2*y coefficient should be 0");
}

#[test]
fn test_findpoly_no_relation() {
    // Two algebraically independent series: no polynomial relation at low degree.
    // Use series with enough distinct coefficients and topshift for overdetermination.
    let q = q_var();
    let trunc = 50;

    // x = 1 + q + 2q^2 + 3q^3 + 5q^4 + 8q^5 + ... (Fibonacci-like)
    let mut x = FormalPowerSeries::zero(q, trunc);
    let mut fib = vec![0i64; trunc as usize];
    fib[0] = 1;
    fib[1] = 1;
    for i in 2..trunc as usize {
        fib[i] = fib[i - 1] + fib[i - 2];
    }
    for i in 0..trunc as usize {
        x.set_coeff(i as i64, qi(fib[i]));
    }

    // y = 1 + 3q + 7q^2 + 13q^3 + ... (prime-gap-like)
    let primes = [1, 3, 7, 13, 19, 29, 37, 43, 53, 61, 71, 79, 89, 97, 103, 109, 127, 131, 139, 149,
                  157, 163, 173, 181, 191, 197, 211, 223, 227, 233, 239, 251, 257, 263, 269, 277,
                  281, 293, 307, 311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373];
    let mut y = FormalPowerSeries::zero(q, trunc);
    for (i, &p) in primes.iter().enumerate() {
        if (i as i64) < trunc {
            y.set_coeff(i as i64, qi(p));
        }
    }

    // No degree-1 polynomial relation should exist with overdetermined system
    let result = findpoly(&x, &y, 1, 1, 20);
    assert!(result.is_none(), "Should not find a linear relation between algebraically independent series");
}

// ===========================================================================
// findcong tests
// ===========================================================================

#[test]
fn test_findcong_ramanujan_mod5() {
    // Ramanujan's congruence: p(5n+4) = 0 (mod 5)
    let q = q_var();
    let pgf = partition_gf(q, 200);
    let congs = findcong(&pgf, &[5]);

    // Should discover at minimum the congruence for residue 4, divisor 5
    let has_mod5 = congs.iter().any(|c| {
        c.modulus_m == 5 && c.residue_b == 4 && c.divisor_r == 5
    });
    assert!(
        has_mod5,
        "Should discover Ramanujan's congruence p(5n+4) = 0 mod 5. Found: {:?}",
        congs
    );
}

#[test]
fn test_findcong_ramanujan_mod7() {
    // Ramanujan's congruence: p(7n+5) = 0 (mod 7)
    let q = q_var();
    let pgf = partition_gf(q, 200);
    let congs = findcong(&pgf, &[7]);

    let has_mod7 = congs.iter().any(|c| {
        c.modulus_m == 7 && c.residue_b == 5 && c.divisor_r == 7
    });
    assert!(
        has_mod7,
        "Should discover Ramanujan's congruence p(7n+5) = 0 mod 7. Found: {:?}",
        congs
    );
}

#[test]
fn test_findcong_ramanujan_mod11() {
    // Ramanujan's congruence: p(11n+6) = 0 (mod 11)
    let q = q_var();
    let pgf = partition_gf(q, 200);
    let congs = findcong(&pgf, &[11]);

    let has_mod11 = congs.iter().any(|c| {
        c.modulus_m == 11 && c.residue_b == 6 && c.divisor_r == 11
    });
    assert!(
        has_mod11,
        "Should discover Ramanujan's congruence p(11n+6) = 0 mod 11. Found: {:?}",
        congs
    );
}

// ===========================================================================
// findnonhom tests
// ===========================================================================

#[test]
fn test_findnonhom_affine_relation() {
    // Create f, g where g = 2*f + 3 (affine, not homogeneous)
    // findnonhom([f, g], 1, 0) should find the relation.
    let q = q_var();
    let trunc = 50;

    let f = fps_from_pairs(q, &[(0, 1), (1, 2), (2, 3), (3, 5), (4, 7)], trunc);
    let three = FormalPowerSeries::monomial(q, qi(3), 0, trunc); // constant 3
    let g = arithmetic::add(&arithmetic::scalar_mul(&qi(2), &f), &three);

    // findnonhom should discover the relation g - 2*f - 3 = 0
    let relations = findnonhom(&[&f, &g], 1, 0);
    assert!(
        !relations.is_empty(),
        "Should find a non-homogeneous affine relation"
    );

    // Verify: the relation involves f, g, and the constant term
    // Degree-0 and degree-1 monomials for 2 variables:
    // degree 0: [(0,0)] -> constant 1
    // degree 1: [(0,1), (1,0)] -> g, f
    // So columns are: [const, g, f]
    // Relation: 3*const + (-1)*g + 2*f = 0 (since g = 2f + 3)
    // Null space vector should be proportional to [3, -1, 2]
    let v = &relations[0];
    assert_eq!(v.len(), 3, "Should have 3 components (const, g, f)");

    // Verify: v[0]*1 + v[1]*g + v[2]*f = 0
    let check = arithmetic::add(
        &arithmetic::add(
            &FormalPowerSeries::monomial(q, v[0].clone(), 0, trunc),
            &arithmetic::scalar_mul(&v[1], &g),
        ),
        &arithmetic::scalar_mul(&v[2], &f),
    );
    assert!(check.is_zero(), "Affine relation should hold: v[0]*1 + v[1]*g + v[2]*f = 0");
}

// ===========================================================================
// findhomcombo tests
// ===========================================================================

#[test]
fn test_findhomcombo_quadratic() {
    // Create f = g1^2 + g2^2
    // findhomcombo(f, [g1, g2], 2, 0) should recover coefficients for g1^2, g1*g2, g2^2
    // i.e., [1, 0, 1]
    let q = q_var();
    let trunc = 50;

    let g1 = fps_from_pairs(q, &[(0, 1), (1, 2), (2, 3), (3, 1), (4, 2)], trunc);
    let g2 = fps_from_pairs(q, &[(0, 1), (1, 1), (2, 4), (3, 2), (4, 3)], trunc);

    let g1_sq = arithmetic::mul(&g1, &g1);
    let g2_sq = arithmetic::mul(&g2, &g2);
    let f = arithmetic::add(&g1_sq, &g2_sq);

    let result = findhomcombo(&f, &[&g1, &g2], 2, 0);
    assert!(result.is_some(), "Should express f as homogeneous degree-2 combination");

    let coeffs = result.unwrap();
    // Degree-2 monomials in 2 variables from generate_monomials(2, 2):
    // [(0,2), (1,1), (2,0)] -> g2^2, g1*g2, g1^2
    assert_eq!(coeffs.len(), 3, "Should have 3 monomial coefficients");

    // f = 1*g1^2 + 0*g1*g2 + 1*g2^2
    // Monomials: [g2^2, g1*g2, g1^2] -> coefficients should be [1, 0, 1]
    assert_eq!(coeffs[0], qi(1), "Coefficient of g2^2 should be 1");
    assert_eq!(coeffs[1], qi(0), "Coefficient of g1*g2 should be 0");
    assert_eq!(coeffs[2], qi(1), "Coefficient of g1^2 should be 1");
}

// ===========================================================================
// findnonhomcombo tests
// ===========================================================================

#[test]
fn test_findnonhomcombo_affine() {
    // Create f = g1 + g2 + 5 (affine combination, not homogeneous)
    // findnonhomcombo(f, [g1, g2], 1, 0) should discover it
    let q = q_var();
    let trunc = 50;

    let g1 = fps_from_pairs(q, &[(0, 1), (1, 3), (2, 7), (3, 2), (4, 5)], trunc);
    let g2 = fps_from_pairs(q, &[(0, 2), (1, 1), (2, 4), (3, 6), (4, 1)], trunc);

    let five = FormalPowerSeries::monomial(q, qi(5), 0, trunc);
    let f = arithmetic::add(&arithmetic::add(&g1, &g2), &five);

    let result = findnonhomcombo(&f, &[&g1, &g2], 1, 0);
    assert!(result.is_some(), "Should express f as non-homogeneous degree-1 combination");

    let coeffs = result.unwrap();
    // Degree 0..=1 monomials in 2 variables:
    // degree 0: [(0,0)] -> constant 1
    // degree 1: [(0,1), (1,0)] -> g2, g1
    // So: [const=1, g2, g1] -> coefficients [5, 1, 1]
    assert_eq!(coeffs.len(), 3, "Should have 3 monomial coefficients (const, g2, g1)");
    assert_eq!(coeffs[0], qi(5), "Constant coefficient should be 5");
    assert_eq!(coeffs[1], qi(1), "Coefficient of g2 should be 1");
    assert_eq!(coeffs[2], qi(1), "Coefficient of g1 should be 1");
}
