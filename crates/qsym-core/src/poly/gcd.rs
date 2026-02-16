//! Polynomial GCD (subresultant PRS), resultant, and q-shift operations.
//!
//! `poly_gcd` uses the subresultant Polynomial Remainder Sequence (PRS) algorithm
//! to compute the GCD of two polynomials without coefficient explosion.
//! `poly_resultant` uses the Euclidean algorithm over Q[x] (exact field, no growth issues).
//! `q_shift` and `q_shift_n` shift a polynomial p(x) -> p(q^j * x).

use super::QRatPoly;
use crate::number::QRat;

/// Raise a QRat to a u32 power via repeated squaring.
fn qrat_pow(base: &QRat, exp: u32) -> QRat {
    if exp == 0 {
        return QRat::one();
    }
    if exp == 1 {
        return base.clone();
    }
    let mut result = QRat::one();
    let mut b = base.clone();
    let mut e = exp;
    while e > 0 {
        if e & 1 == 1 {
            result = &result * &b;
        }
        e >>= 1;
        if e > 0 {
            b = &b * &b;
        }
    }
    result
}

/// Compute the GCD of two polynomials using the subresultant PRS algorithm.
///
/// Returns the monic GCD (leading coefficient 1). Over Q\[x\], every pair of
/// polynomials has a unique monic GCD.
///
/// The subresultant PRS prevents coefficient explosion that would make the
/// naive Euclidean algorithm unusable for degree 5-30 polynomials.
pub fn poly_gcd(a: &QRatPoly, b: &QRatPoly) -> QRatPoly {
    // Handle trivial cases
    if a.is_zero() {
        return b.make_monic();
    }
    if b.is_zero() {
        return a.make_monic();
    }
    if a.is_constant() && b.is_constant() {
        return QRatPoly::one();
    }

    // Ensure deg(f) >= deg(g)
    let (f_in, g_in) = {
        let deg_a = a.degree().unwrap();
        let deg_b = b.degree().unwrap();
        if deg_a >= deg_b {
            (a.clone(), b.clone())
        } else {
            (b.clone(), a.clone())
        }
    };

    // Extract content and work with primitive parts
    let f = f_in.primitive_part();
    let g = g_in.primitive_part();

    if f.is_zero() {
        return g.make_monic();
    }
    if g.is_zero() {
        return f.make_monic();
    }

    // Subresultant PRS algorithm
    let mut f = f;
    let mut g = g;

    // Initialize psi = -1
    let mut psi = QRat::from((-1i64, 1i64));

    // First iteration is special
    let delta0 = f.degree().unwrap() - g.degree().unwrap();
    let sign0 = if (delta0 + 1) % 2 == 0 {
        QRat::one()
    } else {
        QRat::from((-1i64, 1i64))
    };

    let h = f.pseudo_rem(&g);
    if h.is_zero() {
        return g.primitive_part().make_monic();
    }

    // First beta = (-1)^(delta+1)
    let beta0 = sign0;
    let h = h.scalar_div(&beta0);

    // Update psi after first iteration
    let lc_f_neg = -(f.leading_coeff().unwrap());
    if delta0 == 1 {
        psi = lc_f_neg.clone();
    } else if delta0 > 1 {
        // psi = (-lc(f))^delta / psi^(delta-1)
        let num = qrat_pow(&lc_f_neg, delta0 as u32);
        let den = qrat_pow(&psi, (delta0 - 1) as u32);
        psi = &num / &den;
    }
    // if delta0 == 0, psi stays as -1 (but delta0 >= 0 since we ensured deg(f)>=deg(g))

    // Shift
    f = g;
    g = h;

    // Subsequent iterations
    loop {
        if g.is_zero() {
            return f.primitive_part().make_monic();
        }
        if g.is_constant() {
            // GCD is 1 (coprime)
            return QRatPoly::one();
        }

        let deg_f = f.degree().unwrap();
        let deg_g = g.degree().unwrap();

        if deg_f < deg_g {
            // Should not happen in normal PRS, but handle gracefully
            return g.primitive_part().make_monic();
        }

        let delta = deg_f - deg_g;
        let h = f.pseudo_rem(&g);

        if h.is_zero() {
            return g.primitive_part().make_monic();
        }

        let lc_f = f.leading_coeff().unwrap();
        let neg_lc_f = -(lc_f.clone());

        // beta = -lc(f) * psi^delta
        let psi_delta = qrat_pow(&psi, delta as u32);
        let beta = &neg_lc_f * &psi_delta;
        let h = h.scalar_div(&beta);

        // Update psi
        if delta == 1 {
            psi = neg_lc_f;
        } else if delta > 1 {
            let num = qrat_pow(&neg_lc_f, delta as u32);
            let den = qrat_pow(&psi, (delta - 1) as u32);
            psi = &num / &den;
        }
        // delta == 0: psi unchanged

        f = g;
        g = h;
    }
}

/// Compute the resultant of two polynomials.
///
/// The resultant is zero if and only if the two polynomials share a common root
/// (over the algebraic closure). Uses the Euclidean algorithm over Q\[x\].
pub fn poly_resultant(a: &QRatPoly, b: &QRatPoly) -> QRat {
    if a.is_zero() || b.is_zero() {
        return QRat::zero();
    }

    let m = match a.degree() {
        Some(d) => d,
        None => return QRat::zero(),
    };
    let n = match b.degree() {
        Some(d) => d,
        None => return QRat::zero(),
    };

    // Base cases: constant polynomials
    if m == 0 {
        return qrat_pow(&a.coeff(0), n as u32);
    }
    if n == 0 {
        return qrat_pow(&b.coeff(0), m as u32);
    }

    // Compute remainder via Euclidean division (over Q, no coefficient explosion)
    let (_, r) = a.div_rem(b);

    if r.is_zero() {
        // a and b share a common factor
        return QRat::zero();
    }

    let k = r.degree().unwrap();

    // res(a, b) = (-1)^(m*n) * lc(b)^(m - k) * res(b, r)
    let sign = if (m * n) % 2 == 1 {
        QRat::from((-1i64, 1i64))
    } else {
        QRat::one()
    };

    let lc_b = b.leading_coeff().unwrap();
    let lc_b_power = qrat_pow(&lc_b, (m - k) as u32);

    let sub_res = poly_resultant(b, &r);

    &(&sign * &lc_b_power) * &sub_res
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Helper to build polynomial from roots: (x - r1)(x - r2)... ----

    fn poly_from_roots(roots: &[i64]) -> QRatPoly {
        let mut result = QRatPoly::one();
        for &r in roots {
            // (x - r)
            let factor = QRatPoly::from_vec(vec![
                QRat::from((-r, 1i64)),
                QRat::one(),
            ]);
            result = &result * &factor;
        }
        result
    }

    // ---- GCD tests ----

    #[test]
    fn test_gcd_coprime_linears() {
        // gcd(x-1, x-2) = 1
        let a = QRatPoly::from_i64_coeffs(&[-1, 1]); // x - 1
        let b = QRatPoly::from_i64_coeffs(&[-2, 1]); // x - 2
        let g = poly_gcd(&a, &b);
        assert!(g.is_one(), "gcd of coprime linears should be 1, got {}", g);
    }

    #[test]
    fn test_gcd_common_factor() {
        // gcd((x-1)(x-2), (x-1)(x-3)) = (x-1)
        let a = poly_from_roots(&[1, 2]); // (x-1)(x-2)
        let b = poly_from_roots(&[1, 3]); // (x-1)(x-3)
        let g = poly_gcd(&a, &b);
        let expected = QRatPoly::from_i64_coeffs(&[-1, 1]); // x - 1 (monic)
        assert_eq!(g, expected, "gcd should be x-1");
    }

    #[test]
    fn test_gcd_one_divides_other() {
        // gcd((x-1)^2, (x-1)) = (x-1)
        let a = poly_from_roots(&[1, 1]); // (x-1)^2
        let b = QRatPoly::from_i64_coeffs(&[-1, 1]); // x - 1
        let g = poly_gcd(&a, &b);
        let expected = QRatPoly::from_i64_coeffs(&[-1, 1]);
        assert_eq!(g, expected);
    }

    #[test]
    fn test_gcd_same_polynomial() {
        // gcd(p, p) = p (monic)
        let p = QRatPoly::from_i64_coeffs(&[6, 4, 2]); // 2x^2 + 4x + 6
        let g = poly_gcd(&p, &p);
        let expected = p.make_monic();
        assert_eq!(g, expected);
    }

    #[test]
    fn test_gcd_zero_cases() {
        let p = QRatPoly::from_i64_coeffs(&[-2, 3, 1]); // x^2 + 3x - 2
        let z = QRatPoly::zero();

        // gcd(0, p) = p.make_monic()
        assert_eq!(poly_gcd(&z, &p), p.make_monic());
        // gcd(p, 0) = p.make_monic()
        assert_eq!(poly_gcd(&p, &z), p.make_monic());
    }

    #[test]
    fn test_gcd_both_constant() {
        // gcd(6, 4) = 1 (monic over Q)
        let a = QRatPoly::constant(QRat::from((6, 1)));
        let b = QRatPoly::constant(QRat::from((4, 1)));
        let g = poly_gcd(&a, &b);
        assert!(g.is_one());
    }

    #[test]
    fn test_gcd_content_extraction() {
        // Polynomials with large rational coefficients
        // a = (2/3)(x-1)(x-2) = (2/3)(x^2 - 3x + 2) = (2/3)x^2 - 2x + 4/3
        let factor_a = poly_from_roots(&[1, 2]);
        let a = factor_a.scalar_mul(&QRat::from((2, 3)));

        // b = (5/7)(x-1)(x-5) = (5/7)(x^2 - 6x + 5)
        let factor_b = poly_from_roots(&[1, 5]);
        let b = factor_b.scalar_mul(&QRat::from((5, 7)));

        let g = poly_gcd(&a, &b);
        // Should be monic (x-1)
        let expected = QRatPoly::from_i64_coeffs(&[-1, 1]);
        assert_eq!(g, expected, "gcd with rational coefficients should find common factor");
    }

    #[test]
    fn test_gcd_degree_10() {
        // Build degree 10 polynomials with a known degree 3 GCD factor
        // common = (x-1)(x-2)(x-3) = x^3 - 6x^2 + 11x - 6
        // a = common * (x-4)(x-5)(x-6)(x-7) (degree 7+3=10... wait, 3+4=7)
        // Let me be precise:
        // common: roots {1,2,3} (degree 3)
        // a_extra: roots {4,5,6,7,8,9,10} (degree 7) => a = common * a_extra (degree 10)
        // b_extra: roots {11,12,13,14,15,16,17} (degree 7) => b = common * b_extra (degree 10)
        let common = poly_from_roots(&[1, 2, 3]);
        let a = &common * &poly_from_roots(&[4, 5, 6, 7, 8, 9, 10]);
        let b = &common * &poly_from_roots(&[11, 12, 13, 14, 15, 16, 17]);

        assert_eq!(a.degree(), Some(10));
        assert_eq!(b.degree(), Some(10));

        let g = poly_gcd(&a, &b);
        // GCD should be the common factor (monic)
        assert_eq!(g.degree(), Some(3), "GCD should have degree 3");
        assert_eq!(g, common.make_monic());
    }

    // ---- Resultant tests ----

    #[test]
    fn test_resultant_common_root() {
        // resultant((x-1)(x-2), (x-1)(x-3)) = 0
        let a = poly_from_roots(&[1, 2]);
        let b = poly_from_roots(&[1, 3]);
        assert_eq!(poly_resultant(&a, &b), QRat::zero());
    }

    #[test]
    fn test_resultant_no_common_root() {
        // resultant(x-1, x-2) != 0
        let a = QRatPoly::from_i64_coeffs(&[-1, 1]);
        let b = QRatPoly::from_i64_coeffs(&[-2, 1]);
        let r = poly_resultant(&a, &b);
        assert!(!r.is_zero(), "coprime linears should have nonzero resultant");
    }

    #[test]
    fn test_resultant_x_squared_minus_one() {
        // resultant(x^2-1, x-1) = 0 (they share root x=1)
        let a = QRatPoly::from_i64_coeffs(&[-1, 0, 1]); // x^2 - 1
        let b = QRatPoly::from_i64_coeffs(&[-1, 1]);     // x - 1
        assert_eq!(poly_resultant(&a, &b), QRat::zero());
    }

    #[test]
    fn test_resultant_no_real_common_root() {
        // resultant(x^2+1, x^2-1) != 0 (roots are {i,-i} vs {1,-1})
        let a = QRatPoly::from_i64_coeffs(&[1, 0, 1]);  // x^2 + 1
        let b = QRatPoly::from_i64_coeffs(&[-1, 0, 1]); // x^2 - 1
        let r = poly_resultant(&a, &b);
        assert!(!r.is_zero(), "should have no common roots");
    }

    #[test]
    fn test_resultant_linear_pair() {
        // resultant(x-a, x-b) = a-b (up to sign, depends on convention)
        // More precisely: res(x-a, x-b) = (-1)^(1*1) * (x-b) evaluated at a = a-b
        // Actually: res(f,g) where f=x-a, g=x-b:
        // res(x-a, x-b) via our formula: m=1, n=1, (x-a) mod (x-b) = b-a (constant)
        // r = b-a, k=0, sign = (-1)^(1*1) = -1, lc(b)^(1-0)=1
        // res = -1 * 1 * res(x-b, b-a)
        // res(x-b, b-a) = (b-a)^1 = b-a (since deg=0 in second arg)
        // So res(x-a, x-b) = -(b-a) = a-b
        let a = QRatPoly::from_i64_coeffs(&[-3, 1]); // x - 3
        let b = QRatPoly::from_i64_coeffs(&[-5, 1]); // x - 5
        let r = poly_resultant(&a, &b);
        // Should be 3-5 = -2 or 5-3=2 depending on convention
        // From the formula: res(x-3, x-5) = 3 - 5 = -2
        assert_eq!(r, QRat::from((-2i64, 1i64)), "res(x-3, x-5) = -2");
    }

    #[test]
    fn test_resultant_constant_case() {
        // resultant(x+1, 3) = 3^1 = 3
        let a = QRatPoly::from_i64_coeffs(&[1, 1]); // x + 1
        let b = QRatPoly::constant(QRat::from((3, 1)));
        let r = poly_resultant(&a, &b);
        assert_eq!(r, QRat::from((3, 1)));
    }

    #[test]
    fn test_resultant_zero_input() {
        let a = QRatPoly::from_i64_coeffs(&[1, 1]);
        assert_eq!(poly_resultant(&a, &QRatPoly::zero()), QRat::zero());
        assert_eq!(poly_resultant(&QRatPoly::zero(), &a), QRat::zero());
    }
}
