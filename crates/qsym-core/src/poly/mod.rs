//! Dense univariate polynomial over QRat (arbitrary-precision rationals).
//!
//! `QRatPoly` stores coefficients in ascending degree order as a `Vec<QRat>`.
//! Invariant: the vector is either empty (zero polynomial) or the last element is nonzero.

pub mod arithmetic;
pub mod cyclotomic;
pub mod factor;
pub mod gcd;
pub mod ratfunc;

pub use factor::{Factorization, factor_over_q};
pub use gcd::{poly_gcd, poly_resultant};
pub use ratfunc::QRatRationalFunc;

use crate::number::QRat;
use std::cmp::Ordering;
use std::fmt;

/// A dense univariate polynomial with `QRat` coefficients.
///
/// Coefficients are stored in ascending degree order: `coeffs[i]` is the
/// coefficient of x^i. The representation is canonical: the vector is empty
/// for the zero polynomial, and the last element is always nonzero otherwise.
#[derive(Clone, Debug)]
pub struct QRatPoly {
    coeffs: Vec<QRat>,
}

impl QRatPoly {
    // ---- Private helpers ----

    /// Strip trailing zero coefficients to maintain the canonical invariant.
    fn normalize(&mut self) {
        while self
            .coeffs
            .last()
            .map_or(false, |c| c.0.cmp0() == Ordering::Equal)
        {
            self.coeffs.pop();
        }
    }

    // ---- Constructors ----

    /// The zero polynomial.
    pub fn zero() -> Self {
        QRatPoly { coeffs: Vec::new() }
    }

    /// The constant polynomial 1.
    pub fn one() -> Self {
        QRatPoly {
            coeffs: vec![QRat::one()],
        }
    }

    /// A constant polynomial. Returns zero polynomial if `c` is zero.
    pub fn constant(c: QRat) -> Self {
        if c.is_zero() {
            Self::zero()
        } else {
            QRatPoly { coeffs: vec![c] }
        }
    }

    /// The indeterminate x (i.e., 0 + 1*x).
    pub fn x() -> Self {
        QRatPoly {
            coeffs: vec![QRat::zero(), QRat::one()],
        }
    }

    /// The monomial c * x^deg. Returns zero polynomial if `c` is zero.
    pub fn monomial(c: QRat, deg: usize) -> Self {
        if c.is_zero() {
            return Self::zero();
        }
        let mut coeffs = vec![QRat::zero(); deg + 1];
        coeffs[deg] = c;
        QRatPoly { coeffs }
    }

    /// The linear polynomial a + b*x.
    pub fn linear(a: QRat, b: QRat) -> Self {
        Self::from_vec(vec![a, b])
    }

    /// Construct from a vector of coefficients (ascending degree order).
    /// Trailing zeros are stripped.
    pub fn from_vec(coeffs: Vec<QRat>) -> Self {
        let mut p = QRatPoly { coeffs };
        p.normalize();
        p
    }

    /// Convenience constructor from i64 coefficients (ascending degree order).
    pub fn from_i64_coeffs(coeffs: &[i64]) -> Self {
        let v: Vec<QRat> = coeffs.iter().map(|&c| QRat::from((c, 1i64))).collect();
        Self::from_vec(v)
    }

    // ---- Queries ----

    /// Degree of the polynomial, or `None` for the zero polynomial.
    pub fn degree(&self) -> Option<usize> {
        if self.coeffs.is_empty() {
            None
        } else {
            Some(self.coeffs.len() - 1)
        }
    }

    /// Leading coefficient, or `None` for the zero polynomial.
    pub fn leading_coeff(&self) -> Option<QRat> {
        self.coeffs.last().cloned()
    }

    /// Coefficient of x^i. Returns zero if `i` is beyond the polynomial's degree.
    pub fn coeff(&self, i: usize) -> QRat {
        self.coeffs.get(i).cloned().unwrap_or_else(QRat::zero)
    }

    /// True if this is the zero polynomial.
    pub fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }

    /// True if this is a constant polynomial (including zero).
    pub fn is_constant(&self) -> bool {
        self.degree().map_or(true, |d| d == 0)
    }

    /// True if this is the constant polynomial 1.
    pub fn is_one(&self) -> bool {
        self.coeffs.len() == 1 && self.coeffs[0] == QRat::one()
    }

    /// Public read-only access to the coefficient slice.
    pub fn coeffs(&self) -> &[QRat] {
        &self.coeffs
    }

    // ---- Content / Primitive Part / Monic ----

    /// Content of the polynomial: gcd(numerators) / lcm(denominators).
    ///
    /// For the zero polynomial, returns zero.
    pub fn content(&self) -> QRat {
        if self.coeffs.is_empty() {
            return QRat::zero();
        }

        let mut numer_gcd = rug::Integer::from(0);
        let mut denom_lcm = rug::Integer::from(1);

        for c in &self.coeffs {
            let n = c.0.numer().clone().abs();
            let d = c.0.denom().clone();
            numer_gcd = numer_gcd.gcd(&n);
            denom_lcm = denom_lcm.lcm(&d);
        }

        QRat(rug::Rational::from((numer_gcd, denom_lcm)))
    }

    /// Primitive part: self / content. Returns zero for the zero polynomial.
    pub fn primitive_part(&self) -> QRatPoly {
        let cont = self.content();
        if cont.is_zero() {
            return Self::zero();
        }
        self.scalar_div(&cont)
    }

    /// Make monic: divide all coefficients by the leading coefficient.
    /// Returns zero for the zero polynomial.
    pub fn make_monic(&self) -> QRatPoly {
        match self.leading_coeff() {
            None => Self::zero(),
            Some(lc) => self.scalar_div(&lc),
        }
    }

    // ---- Scalar operations (used by content/primitive_part/make_monic) ----

    /// Multiply every coefficient by a scalar.
    pub fn scalar_mul(&self, c: &QRat) -> QRatPoly {
        if c.is_zero() {
            return Self::zero();
        }
        let coeffs: Vec<QRat> = self.coeffs.iter().map(|ci| ci * c).collect();
        QRatPoly::from_vec(coeffs)
    }

    /// Divide every coefficient by a scalar. Panics if `c` is zero.
    pub fn scalar_div(&self, c: &QRat) -> QRatPoly {
        assert!(!c.is_zero(), "QRatPoly::scalar_div: division by zero");
        let coeffs: Vec<QRat> = self.coeffs.iter().map(|ci| ci / c).collect();
        QRatPoly::from_vec(coeffs)
    }

    // ---- Evaluation ----

    /// Evaluate the polynomial at `x` using Horner's method.
    pub fn eval(&self, x: &QRat) -> QRat {
        let mut result = QRat::zero();
        for c in self.coeffs.iter().rev() {
            result = &(&result * x) + c;
        }
        result
    }

    // ---- q-shift operations ----

    /// Compute p(q_val * x): scale coefficient c_i by q_val^i.
    ///
    /// This is an O(n) operation that shifts the polynomial argument by a
    /// multiplicative factor. Used extensively in q-dispersion and q-Gosper.
    pub fn q_shift(&self, q_val: &QRat) -> QRatPoly {
        if self.is_zero() {
            return QRatPoly::zero();
        }
        if *q_val == QRat::one() {
            return self.clone();
        }
        let mut result = Vec::with_capacity(self.coeffs.len());
        let mut q_power = QRat::one();
        for c in &self.coeffs {
            result.push(&q_power * c);
            q_power = &q_power * q_val;
        }
        QRatPoly::from_vec(result)
    }

    /// Compute p(q_val^j * x) for integer j.
    ///
    /// Equivalent to j successive applications of `q_shift` (for positive j)
    /// or shifting by the inverse (for negative j).
    ///
    /// Panics if `q_val` is zero and `j` is negative.
    pub fn q_shift_n(&self, q_val: &QRat, j: i64) -> QRatPoly {
        if j == 0 || self.is_zero() {
            return self.clone();
        }
        let effective_q = qrat_pow_signed(q_val, j);
        self.q_shift(&effective_q)
    }
}

/// Raise a QRat to a signed integer power.
///
/// For negative exponents, computes base^|exp| then inverts.
/// Panics if base is zero and exp is negative.
fn qrat_pow_signed(base: &QRat, exp: i64) -> QRat {
    if exp == 0 {
        return QRat::one();
    }
    if exp > 0 {
        qrat_pow_u32(base, exp as u32)
    } else {
        assert!(!base.is_zero(), "qrat_pow_signed: zero base with negative exponent");
        let positive = qrat_pow_u32(base, (-exp) as u32);
        &QRat::one() / &positive
    }
}

/// Raise a QRat to a u32 power via repeated squaring.
fn qrat_pow_u32(base: &QRat, exp: u32) -> QRat {
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

// ---- PartialEq / Eq ----

impl PartialEq for QRatPoly {
    fn eq(&self, other: &Self) -> bool {
        if self.coeffs.len() != other.coeffs.len() {
            return false;
        }
        self.coeffs
            .iter()
            .zip(other.coeffs.iter())
            .all(|(a, b)| a == b)
    }
}

impl Eq for QRatPoly {}

// ---- Display ----

impl fmt::Display for QRatPoly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }

        let mut first = true;
        // Print from highest degree down
        for i in (0..self.coeffs.len()).rev() {
            let c = &self.coeffs[i];
            if c.is_zero() {
                continue;
            }

            let is_negative = c.0.cmp0() == Ordering::Less;
            let abs_c = QRat(rug::Rational::from(c.0.clone().abs()));
            let is_unit = abs_c == QRat::one();

            if first {
                if is_negative {
                    write!(f, "-")?;
                }
                first = false;
            } else if is_negative {
                write!(f, " - ")?;
            } else {
                write!(f, " + ")?;
            }

            match i {
                0 => {
                    // Constant term: always print the coefficient
                    write!(f, "{}", abs_c)?;
                }
                1 => {
                    // x^1: print "x" or "cx"
                    if !is_unit {
                        write!(f, "{}*", abs_c)?;
                    }
                    write!(f, "x")?;
                }
                _ => {
                    // x^k for k >= 2
                    if !is_unit {
                        write!(f, "{}*", abs_c)?;
                    }
                    write!(f, "x^{}", i)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Constructor tests ----

    #[test]
    fn test_zero() {
        let z = QRatPoly::zero();
        assert!(z.is_zero());
        assert_eq!(z.degree(), None);
        assert_eq!(z.leading_coeff(), None);
    }

    #[test]
    fn test_one() {
        let one = QRatPoly::one();
        assert!(!one.is_zero());
        assert!(one.is_one());
        assert!(one.is_constant());
        assert_eq!(one.degree(), Some(0));
        assert_eq!(one.leading_coeff(), Some(QRat::one()));
    }

    #[test]
    fn test_constant() {
        let c = QRatPoly::constant(QRat::from((3, 1)));
        assert_eq!(c.degree(), Some(0));
        assert!(c.is_constant());
        assert_eq!(c.coeff(0), QRat::from((3, 1)));

        // Zero constant returns zero poly
        let z = QRatPoly::constant(QRat::zero());
        assert!(z.is_zero());
    }

    #[test]
    fn test_x() {
        let x = QRatPoly::x();
        assert_eq!(x.degree(), Some(1));
        assert_eq!(x.coeff(0), QRat::zero());
        assert_eq!(x.coeff(1), QRat::one());
    }

    #[test]
    fn test_monomial() {
        let m = QRatPoly::monomial(QRat::from((5, 1)), 3);
        assert_eq!(m.degree(), Some(3));
        assert_eq!(m.coeff(3), QRat::from((5, 1)));
        assert_eq!(m.coeff(0), QRat::zero());
        assert_eq!(m.coeff(1), QRat::zero());
        assert_eq!(m.coeff(2), QRat::zero());

        // Monomial with zero coeff returns zero poly
        let z = QRatPoly::monomial(QRat::zero(), 5);
        assert!(z.is_zero());
    }

    #[test]
    fn test_linear() {
        let l = QRatPoly::linear(QRat::from((1, 1)), QRat::from((2, 1)));
        assert_eq!(l.degree(), Some(1));
        assert_eq!(l.coeff(0), QRat::from((1, 1)));
        assert_eq!(l.coeff(1), QRat::from((2, 1)));
    }

    #[test]
    fn test_from_vec_strips_trailing_zeros() {
        let p = QRatPoly::from_vec(vec![
            QRat::from((1, 1)),
            QRat::from((2, 1)),
            QRat::zero(),
            QRat::zero(),
        ]);
        assert_eq!(p.degree(), Some(1));
        assert_eq!(p.coeffs().len(), 2);
    }

    #[test]
    fn test_from_i64_coeffs() {
        let p = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        assert_eq!(p.degree(), Some(2));
        assert_eq!(p.coeff(0), QRat::from((1, 1)));
        assert_eq!(p.coeff(1), QRat::from((2, 1)));
        assert_eq!(p.coeff(2), QRat::from((3, 1)));
    }

    // ---- Query tests ----

    #[test]
    fn test_degree_and_leading_coeff() {
        let p = QRatPoly::from_i64_coeffs(&[1, 0, 3]); // 1 + 3x^2
        assert_eq!(p.degree(), Some(2));
        assert_eq!(p.leading_coeff(), Some(QRat::from((3, 1))));
    }

    #[test]
    fn test_coeff_out_of_range() {
        let p = QRatPoly::from_i64_coeffs(&[1, 2]);
        assert_eq!(p.coeff(5), QRat::zero());
    }

    #[test]
    fn test_is_constant() {
        assert!(QRatPoly::zero().is_constant());
        assert!(QRatPoly::one().is_constant());
        assert!(QRatPoly::constant(QRat::from((7, 1))).is_constant());
        assert!(!QRatPoly::x().is_constant());
    }

    // ---- Content / Primitive Part / Monic tests ----

    #[test]
    fn test_content_integer_coeffs() {
        // 6x^2 + 4x + 2 => content = 2
        let p = QRatPoly::from_i64_coeffs(&[2, 4, 6]);
        let cont = p.content();
        assert_eq!(cont, QRat::from((2, 1)));
    }

    #[test]
    fn test_primitive_part() {
        // 6x^2 + 4x + 2 => primitive part = 3x^2 + 2x + 1
        let p = QRatPoly::from_i64_coeffs(&[2, 4, 6]);
        let pp = p.primitive_part();
        assert_eq!(pp, QRatPoly::from_i64_coeffs(&[1, 2, 3]));
    }

    #[test]
    fn test_content_rational_coeffs() {
        // (1/2)x + (1/3) => content = gcd(1,1)/lcm(2,3) = 1/6
        let p = QRatPoly::from_vec(vec![QRat::from((1, 3)), QRat::from((1, 2))]);
        let cont = p.content();
        assert_eq!(cont, QRat::from((1, 6)));
    }

    #[test]
    fn test_content_zero_poly() {
        assert_eq!(QRatPoly::zero().content(), QRat::zero());
    }

    #[test]
    fn test_make_monic() {
        // 2x^2 + 4x + 6 => x^2 + 2x + 3
        let p = QRatPoly::from_i64_coeffs(&[6, 4, 2]);
        let m = p.make_monic();
        assert_eq!(m, QRatPoly::from_i64_coeffs(&[3, 2, 1]));
    }

    #[test]
    fn test_make_monic_zero() {
        assert!(QRatPoly::zero().make_monic().is_zero());
    }

    // ---- Evaluation tests ----

    #[test]
    fn test_eval_quadratic() {
        // x^2 + x + 1 evaluated at x=2 => 4 + 2 + 1 = 7
        let p = QRatPoly::from_i64_coeffs(&[1, 1, 1]);
        let result = p.eval(&QRat::from((2, 1)));
        assert_eq!(result, QRat::from((7, 1)));
    }

    #[test]
    fn test_eval_root() {
        // 3x - 1 evaluated at x=1/3 => 0
        let p = QRatPoly::from_vec(vec![QRat::from((-1, 1)), QRat::from((3, 1))]);
        let result = p.eval(&QRat::from((1, 3)));
        assert_eq!(result, QRat::zero());
    }

    #[test]
    fn test_eval_zero_poly() {
        assert_eq!(QRatPoly::zero().eval(&QRat::from((5, 1))), QRat::zero());
    }

    // ---- Display tests ----

    #[test]
    fn test_display_zero() {
        assert_eq!(format!("{}", QRatPoly::zero()), "0");
    }

    #[test]
    fn test_display_constant() {
        assert_eq!(
            format!("{}", QRatPoly::constant(QRat::from((3, 1)))),
            "3"
        );
    }

    #[test]
    fn test_display_x() {
        assert_eq!(format!("{}", QRatPoly::x()), "x");
    }

    #[test]
    fn test_display_polynomial() {
        // 3x^2 + 2x + 1
        let p = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        assert_eq!(format!("{}", p), "3*x^2 + 2*x + 1");
    }

    #[test]
    fn test_display_negative_coeffs() {
        // x^2 - 2x + 1
        let p = QRatPoly::from_i64_coeffs(&[1, -2, 1]);
        assert_eq!(format!("{}", p), "x^2 - 2*x + 1");
    }

    #[test]
    fn test_display_leading_negative() {
        // -x + 1
        let p = QRatPoly::from_i64_coeffs(&[1, -1]);
        assert_eq!(format!("{}", p), "-x + 1");
    }

    #[test]
    fn test_display_rational_coeffs() {
        // (1/2)x + 1/3
        let p = QRatPoly::from_vec(vec![QRat::from((1, 3)), QRat::from((1, 2))]);
        assert_eq!(format!("{}", p), "1/2*x + 1/3");
    }

    // ---- PartialEq tests ----

    #[test]
    fn test_equality() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let b = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_inequality() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let b = QRatPoly::from_i64_coeffs(&[1, 2, 4]);
        assert_ne!(a, b);
    }

    #[test]
    fn test_equality_different_trailing_zeros() {
        // from_vec strips trailing zeros, so these should be equal
        let a = QRatPoly::from_vec(vec![
            QRat::from((1, 1)),
            QRat::from((2, 1)),
            QRat::zero(),
        ]);
        let b = QRatPoly::from_i64_coeffs(&[1, 2]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_zero_polys_equal() {
        assert_eq!(QRatPoly::zero(), QRatPoly::zero());
        assert_eq!(
            QRatPoly::from_vec(vec![QRat::zero()]),
            QRatPoly::zero()
        );
    }

    // ---- q-shift tests ----

    #[test]
    fn test_q_shift_identity() {
        // Shifting by q=1 should return the same polynomial
        let p = QRatPoly::from_i64_coeffs(&[1, 1, 1]); // x^2 + x + 1
        let shifted = p.q_shift(&QRat::one());
        assert_eq!(shifted, p);
    }

    #[test]
    fn test_q_shift_simple() {
        // (x^2 + x + 1).q_shift(2) = 4x^2 + 2x + 1
        // c_0 * 2^0 + c_1 * 2^1 * x + c_2 * 2^2 * x^2
        // = 1 + 2x + 4x^2
        let p = QRatPoly::from_i64_coeffs(&[1, 1, 1]);
        let shifted = p.q_shift(&QRat::from((2, 1)));
        assert_eq!(shifted, QRatPoly::from_i64_coeffs(&[1, 2, 4]));
    }

    #[test]
    fn test_q_shift_zero_poly() {
        let z = QRatPoly::zero();
        let shifted = z.q_shift(&QRat::from((5, 1)));
        assert!(shifted.is_zero());
    }

    #[test]
    fn test_q_shift_evaluation_identity() {
        // For any p, q, x: p.q_shift(q).eval(x) == p.eval(q*x)
        let p = QRatPoly::from_i64_coeffs(&[3, -2, 1, 5]); // 5x^3 + x^2 - 2x + 3
        let q = QRat::from((3, 2)); // q = 3/2
        let x = QRat::from((7, 3)); // x = 7/3

        let shifted = p.q_shift(&q);
        let lhs = shifted.eval(&x);

        let qx = &q * &x;
        let rhs = p.eval(&qx);

        assert_eq!(lhs, rhs, "p.q_shift(q).eval(x) should equal p.eval(q*x)");
    }

    #[test]
    fn test_q_shift_evaluation_identity_2() {
        // Test with a different polynomial and values
        let p = QRatPoly::from_vec(vec![
            QRat::from((1, 2)),
            QRat::from((-3, 7)),
            QRat::from((5, 1)),
        ]); // 5x^2 - 3/7 x + 1/2
        let q = QRat::from((4, 5));
        let x = QRat::from((-1, 3));

        let shifted = p.q_shift(&q);
        let lhs = shifted.eval(&x);
        let qx = &q * &x;
        let rhs = p.eval(&qx);
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn test_q_shift_double_equals_shift_n_2() {
        // p.q_shift(q).q_shift(q) == p.q_shift_n(q, 2)
        let p = QRatPoly::from_i64_coeffs(&[1, -1, 2, 3]);
        let q = QRat::from((2, 3));

        let double_shift = p.q_shift(&q).q_shift(&q);
        let shift_n_2 = p.q_shift_n(&q, 2);

        assert_eq!(double_shift, shift_n_2);
    }

    #[test]
    fn test_q_shift_n_negative_roundtrip() {
        // p.q_shift_n(q, -1).q_shift(q) == p
        let p = QRatPoly::from_i64_coeffs(&[2, 3, 1]);
        let q = QRat::from((5, 2));

        let shifted_neg = p.q_shift_n(&q, -1);
        let roundtrip = shifted_neg.q_shift(&q);

        assert_eq!(roundtrip, p, "q_shift_n(-1) then q_shift should be identity");
    }

    #[test]
    fn test_q_shift_n_zero_returns_original() {
        let p = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let shifted = p.q_shift_n(&QRat::from((7, 1)), 0);
        assert_eq!(shifted, p);
    }

    #[test]
    fn test_q_shift_n_positive_3() {
        // q_shift_n(q, 3) should equal three successive q_shifts
        let p = QRatPoly::from_i64_coeffs(&[1, 1, 1]);
        let q = QRat::from((2, 1));

        let triple_shift = p.q_shift(&q).q_shift(&q).q_shift(&q);
        let shift_n_3 = p.q_shift_n(&q, 3);

        assert_eq!(triple_shift, shift_n_3);
    }
}

// ---- Integration tests verifying Phase 13 success criteria ----

#[cfg(test)]
mod integration_tests {
    use super::*;
    use super::gcd::{poly_gcd, poly_resultant};
    use super::ratfunc::QRatRationalFunc;

    // Helper: build (x - r)
    fn x_minus(r: i64) -> QRatPoly {
        QRatPoly::from_vec(vec![QRat::from((-r, 1i64)), QRat::one()])
    }

    // Helper: build product of linear factors (x - r1)(x - r2)...
    fn poly_from_roots(roots: &[i64]) -> QRatPoly {
        let mut result = QRatPoly::one();
        for &r in roots {
            result = &result * &x_minus(r);
        }
        result
    }

    // ========================================
    // POLY-01: QRatPoly arithmetic operations
    // ========================================

    #[test]
    fn test_poly01_div_and_mul_roundtrip() {
        // (x^3 - 2x^2 + x) / (x - 1) = x^2 - x
        // (x^2 - x) * (x - 1) + 0 = x^3 - 2x^2 + x
        let a = QRatPoly::from_i64_coeffs(&[0, 1, -2, 1]); // x^3 - 2x^2 + x
        let b = QRatPoly::from_i64_coeffs(&[-1, 1]); // x - 1
        let (q, r) = a.div_rem(&b);
        assert_eq!(q, QRatPoly::from_i64_coeffs(&[0, -1, 1])); // x^2 - x
        assert!(r.is_zero());
        // Verify reconstruction: q * b + r == a
        let reconstructed = &(&q * &b) + &r;
        assert_eq!(reconstructed, a);
    }

    #[test]
    fn test_poly01_full_arithmetic_chain() {
        // Verify: (a + b) * c == a*c + b*c (distributive law with larger polys)
        let a = QRatPoly::from_i64_coeffs(&[1, -3, 2, 1]); // x^3 + 2x^2 - 3x + 1
        let b = QRatPoly::from_i64_coeffs(&[2, 0, -1, 0, 1]); // x^4 - x^2 + 2
        let c = QRatPoly::from_i64_coeffs(&[-1, 2, 3]); // 3x^2 + 2x - 1
        let lhs = &(&a + &b) * &c;
        let rhs = &(&a * &c) + &(&b * &c);
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn test_poly01_exact_div_with_qrat() {
        // (3/2 x^2 + 3x + 3/2) / (x + 1) = (3/2)(x + 1)
        // numerator = (3/2)(x^2 + 2x + 1) = (3/2)(x+1)^2
        let numer = QRatPoly::from_vec(vec![
            QRat::from((3, 2)),
            QRat::from((3, 1)),
            QRat::from((3, 2)),
        ]);
        let denom = QRatPoly::from_i64_coeffs(&[1, 1]); // x + 1
        let q = numer.exact_div(&denom);
        let expected = QRatPoly::from_vec(vec![QRat::from((3, 2)), QRat::from((3, 2))]);
        assert_eq!(q, expected);
    }

    // ========================================
    // POLY-02: GCD of degree-5+ polynomials
    // ========================================

    #[test]
    fn test_poly02_gcd_degree5_known_common() {
        // p = (x^2 + 1)(x^3 - x + 1), q = (x^2 + 1)(x^2 - 3x + 2)
        // gcd(p, q) = x^2 + 1
        let common = QRatPoly::from_i64_coeffs(&[1, 0, 1]); // x^2 + 1
        let p_extra = QRatPoly::from_i64_coeffs(&[1, -1, 0, 1]); // x^3 - x + 1
        let q_extra = QRatPoly::from_i64_coeffs(&[2, -3, 1]); // x^2 - 3x + 2
        let p = &common * &p_extra;
        let q = &common * &q_extra;

        assert_eq!(p.degree(), Some(5));
        assert_eq!(q.degree(), Some(4));

        let g = poly_gcd(&p, &q);
        assert_eq!(g, common, "GCD should be x^2 + 1");
    }

    #[test]
    fn test_poly02_gcd_degree8_no_explosion() {
        // Build degree 8 polys with a degree 4 common factor
        let common = poly_from_roots(&[1, 2, 3, 4]); // degree 4
        let p_extra = poly_from_roots(&[5, 6, 7, 8]); // degree 4
        let q_extra = poly_from_roots(&[9, 10, 11, 12]); // degree 4
        let p = &common * &p_extra;
        let q = &common * &q_extra;
        assert_eq!(p.degree(), Some(8));
        assert_eq!(q.degree(), Some(8));

        let g = poly_gcd(&p, &q);
        assert_eq!(g.degree(), Some(4));
        assert_eq!(g, common.make_monic());
    }

    #[test]
    fn test_poly02_gcd_monic_result() {
        // GCD should always be monic over Q[x]
        let a = QRatPoly::from_i64_coeffs(&[6, 4, 2]); // 2x^2 + 4x + 6
        let b = QRatPoly::from_i64_coeffs(&[3, 2, 1]); // x^2 + 2x + 3
        let g = poly_gcd(&a, &b);
        // a = 2 * b, so gcd = b (monic)
        assert_eq!(g, b);
        assert_eq!(g.leading_coeff(), Some(QRat::one()));
    }

    // ========================================
    // POLY-03: Resultant identifies shared roots
    // ========================================

    #[test]
    fn test_poly03_resultant_shared_root() {
        // resultant(x^2 - 5x + 6, x^2 - 3x + 2) = 0 (share root x=2)
        let a = QRatPoly::from_i64_coeffs(&[6, -5, 1]); // x^2 - 5x + 6 = (x-2)(x-3)
        let b = QRatPoly::from_i64_coeffs(&[2, -3, 1]); // x^2 - 3x + 2 = (x-1)(x-2)
        assert_eq!(poly_resultant(&a, &b), QRat::zero());
    }

    #[test]
    fn test_poly03_resultant_no_common_root() {
        // resultant(x^2 + 1, x^2 - 1) != 0 (no common root over Q)
        let a = QRatPoly::from_i64_coeffs(&[1, 0, 1]); // x^2 + 1
        let b = QRatPoly::from_i64_coeffs(&[-1, 0, 1]); // x^2 - 1
        let r = poly_resultant(&a, &b);
        assert!(!r.is_zero(), "x^2+1 and x^2-1 have no common root");
    }

    #[test]
    fn test_poly03_resultant_value() {
        // res(x - a, x - b) = a - b for linear polynomials
        // res(x - 2, x - 5) = 2 - 5 = -3
        let a = QRatPoly::from_i64_coeffs(&[-2, 1]); // x - 2
        let b = QRatPoly::from_i64_coeffs(&[-5, 1]); // x - 5
        let r = poly_resultant(&a, &b);
        assert_eq!(r, QRat::from((-3i64, 1i64)));
    }

    // ========================================
    // POLY-04: q-shift evaluation identity
    // ========================================

    #[test]
    fn test_poly04_q_shift_eval_identity() {
        // For p = x^3 + 2x + 1, q = 3/7:
        // p.q_shift(q).eval(x) == p.eval(q * x) for several x values
        let p = QRatPoly::from_i64_coeffs(&[1, 2, 0, 1]); // x^3 + 2x + 1
        let q = QRat::from((3, 7));

        let shifted = p.q_shift(&q);

        for x_val in &[
            QRat::from((1, 1)),
            QRat::from((2, 1)),
            QRat::from((-3, 2)),
            QRat::from((7, 5)),
            QRat::from((0, 1)),
        ] {
            let lhs = shifted.eval(x_val);
            let qx = &q * x_val;
            let rhs = p.eval(&qx);
            assert_eq!(lhs, rhs, "q-shift identity failed at x = {}", x_val);
        }
    }

    #[test]
    fn test_poly04_q_shift_n_composition() {
        // q_shift_n(q, a).q_shift_n(q, b) == q_shift_n(q, a+b)
        let p = QRatPoly::from_i64_coeffs(&[1, -2, 3, 1]);
        let q = QRat::from((2, 3));

        let shift_3_then_2 = p.q_shift_n(&q, 3).q_shift_n(&q, 2);
        let shift_5 = p.q_shift_n(&q, 5);
        assert_eq!(shift_3_then_2, shift_5);
    }

    // ========================================
    // POLY-05: Rational function auto-simplification and arithmetic
    // ========================================

    #[test]
    fn test_poly05_ratfunc_simplification() {
        // (x^3 - x) / (x^2 - 1) = x/1 since x^3-x = x(x^2-1)
        let numer = QRatPoly::from_i64_coeffs(&[0, -1, 0, 1]); // x^3 - x
        let denom = QRatPoly::from_i64_coeffs(&[-1, 0, 1]); // x^2 - 1
        let rf = QRatRationalFunc::new(numer, denom);
        assert_eq!(rf.numer, QRatPoly::x());
        assert_eq!(rf.denom, QRatPoly::one());
        assert!(rf.is_polynomial());
    }

    #[test]
    fn test_poly05_ratfunc_addition() {
        // 1/(x-1) + 1/(x+1) = 2x/(x^2-1)
        let a = QRatRationalFunc::new(QRatPoly::one(), x_minus(1));
        let b = QRatRationalFunc::new(QRatPoly::one(), QRatPoly::from_i64_coeffs(&[1, 1]));
        let sum = &a + &b;
        // numerator: 2x, denom: x^2 - 1
        assert_eq!(sum.numer, QRatPoly::from_i64_coeffs(&[0, 2]));
        assert_eq!(sum.denom, QRatPoly::from_i64_coeffs(&[-1, 0, 1]));
    }

    // ========================================
    // Round-trip test: build and decompose rational functions
    // ========================================

    #[test]
    fn test_roundtrip_rf_mul_inverse() {
        // rf = (x+1)(x+2) / ((x+3)(x+4))
        // rf * ((x+3)(x+4) / (x+1)(x+2)) == 1/1
        let n1 = &QRatPoly::from_i64_coeffs(&[1, 1]) * &QRatPoly::from_i64_coeffs(&[2, 1]);
        let d1 = &QRatPoly::from_i64_coeffs(&[3, 1]) * &QRatPoly::from_i64_coeffs(&[4, 1]);
        let rf = QRatRationalFunc::new(n1.clone(), d1.clone());
        let rf_inv = QRatRationalFunc::new(d1, n1);
        let product = &rf * &rf_inv;
        assert_eq!(product, QRatRationalFunc::one());
    }

    #[test]
    fn test_roundtrip_add_sub() {
        // (a/b + c/d) - c/d == a/b
        let a = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[1, 2]),
            QRatPoly::from_i64_coeffs(&[3, 0, 1]),
        );
        let b = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[-1, 1]),
            QRatPoly::from_i64_coeffs(&[1, 1, 1]),
        );
        let roundtrip = &(&a + &b) - &b;
        assert_eq!(roundtrip, a);
    }

    // ========================================
    // Coefficient size test: GCD of degree-10 polynomials
    // ========================================

    #[test]
    fn test_coefficient_size_degree10() {
        // GCD of degree-10 polynomials with 3-digit rational coefficients.
        // Verify subresultant PRS keeps coefficients manageable.

        // Build common factor with large-ish coefficients
        // common = (100x + 123)(200x + 456)(300x + 789) -- degree 3
        let f1 = QRatPoly::from_i64_coeffs(&[123, 100]);
        let f2 = QRatPoly::from_i64_coeffs(&[456, 200]);
        let f3 = QRatPoly::from_i64_coeffs(&[789, 300]);
        let common = &(&f1 * &f2) * &f3;
        assert_eq!(common.degree(), Some(3));

        // p_extra: degree 7 polynomial from integer roots
        let p_extra = poly_from_roots(&[1, 2, 3, 4, 5, 6, 7]);
        // q_extra: degree 7 polynomial from different roots
        let q_extra = poly_from_roots(&[8, 9, 10, 11, 12, 13, 14]);

        let p = &common * &p_extra; // degree 10
        let q = &common * &q_extra; // degree 10
        assert_eq!(p.degree(), Some(10));
        assert_eq!(q.degree(), Some(10));

        let g = poly_gcd(&p, &q);

        // The GCD should be common.make_monic()
        assert_eq!(g.degree(), Some(3), "GCD should have degree 3");
        assert_eq!(g, common.make_monic());

        // Verify coefficient sizes are reasonable (no explosion)
        // The monic GCD coefficients should have numerators/denominators with < 15 digits
        for c in g.coeffs() {
            let numer_digits = c.0.numer().to_string().len();
            let denom_digits = c.0.denom().to_string().len();
            assert!(
                numer_digits <= 15,
                "Numerator too large: {} ({} digits)",
                c.0.numer(),
                numer_digits
            );
            assert!(
                denom_digits <= 15,
                "Denominator too large: {} ({} digits)",
                c.0.denom(),
                denom_digits
            );
        }
    }

    // ========================================
    // Additional cross-module interaction tests
    // ========================================

    #[test]
    fn test_ratfunc_eval_matches_poly_division() {
        // For p/q rational function, eval at x should match numer.eval / denom.eval
        let rf = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[2, 3, 1]), // x^2 + 3x + 2
            QRatPoly::from_i64_coeffs(&[-1, 1]),    // x - 1
        );
        let x = QRat::from((5, 1));
        let result = rf.eval(&x).unwrap();
        let expected = &rf.numer.eval(&x) / &rf.denom.eval(&x);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_gcd_and_ratfunc_consistency() {
        // poly_gcd(a, b) == 1 iff (a/b) rational function stays unreduced
        let a = QRatPoly::from_i64_coeffs(&[1, 1]); // x + 1
        let b = QRatPoly::from_i64_coeffs(&[-1, 1]); // x - 1
        let g = poly_gcd(&a, &b);
        assert!(g.is_one(), "x+1 and x-1 are coprime");

        let rf = QRatRationalFunc::new(a.clone(), b.clone());
        // Since coprime, rf should keep exact numer and denom
        assert_eq!(rf.numer, a);
        assert_eq!(rf.denom, b);
    }

    #[test]
    fn test_resultant_zero_iff_gcd_nontrivial() {
        // Verify: resultant(a,b) == 0 iff gcd(a,b) != 1
        let common = x_minus(3);
        let a = &common * &x_minus(1); // (x-3)(x-1)
        let b = &common * &x_minus(5); // (x-3)(x-5)

        let g = poly_gcd(&a, &b);
        let r = poly_resultant(&a, &b);
        assert!(!g.is_one(), "Should have nontrivial GCD");
        assert!(r.is_zero(), "Resultant should be zero when GCD nontrivial");

        // Now test coprime pair
        let c = QRatPoly::from_i64_coeffs(&[1, 0, 1]); // x^2 + 1
        let d = QRatPoly::from_i64_coeffs(&[1, 1]); // x + 1
        let g2 = poly_gcd(&c, &d);
        let r2 = poly_resultant(&c, &d);
        assert!(g2.is_one(), "Should be coprime");
        assert!(!r2.is_zero(), "Resultant should be nonzero when coprime");
    }

    #[test]
    fn test_q_shift_preserves_gcd_structure() {
        // If gcd(a, b) = g, then shifting all by same q preserves the factorization
        let common = QRatPoly::from_i64_coeffs(&[1, 1]); // x + 1
        let a = &common * &QRatPoly::from_i64_coeffs(&[-1, 1]); // (x+1)(x-1)
        let b = &common * &QRatPoly::from_i64_coeffs(&[2, 1]); // (x+1)(x+2)

        let q = QRat::from((3, 1));
        let a_shifted = a.q_shift(&q);
        let b_shifted = b.q_shift(&q);
        let common_shifted = common.q_shift(&q);

        // The shifted common factor should divide both shifted polys
        let (_, r1) = a_shifted.div_rem(&common_shifted);
        let (_, r2) = b_shifted.div_rem(&common_shifted);
        assert!(r1.is_zero(), "Shifted common factor should divide shifted a");
        assert!(r2.is_zero(), "Shifted common factor should divide shifted b");
    }

    #[test]
    fn test_ratfunc_field_axioms() {
        // Verify field-like axioms for rational functions
        let a = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[1, 1]),
            QRatPoly::from_i64_coeffs(&[-1, 1]),
        );
        let b = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[2, 1]),
            QRatPoly::from_i64_coeffs(&[3, 1]),
        );
        let c = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[-2, 0, 1]),
            QRatPoly::from_i64_coeffs(&[1, 0, 1]),
        );

        // Commutativity of addition
        assert_eq!(&a + &b, &b + &a);

        // Commutativity of multiplication
        assert_eq!(&a * &b, &b * &a);

        // Associativity of multiplication
        assert_eq!(&(&a * &b) * &c, &a * &(&b * &c));

        // Distributivity: a * (b + c) = a*b + a*c
        let lhs = &a * &(&b + &c);
        let rhs = &(&a * &b) + &(&a * &c);
        assert_eq!(lhs, rhs);

        // Multiplicative identity
        let one = QRatRationalFunc::one();
        assert_eq!(&a * &one, a.clone());

        // Additive identity
        let zero = QRatRationalFunc::zero();
        assert_eq!(&a + &zero, a.clone());

        // Additive inverse
        let neg_a = -a.clone();
        assert!((&a + &neg_a).is_zero());

        // Multiplicative inverse (a * (1/a) = 1)
        let a_inv = &QRatRationalFunc::one() / &a;
        let product = &a * &a_inv;
        assert_eq!(product, one);
    }

    #[test]
    fn test_poly_eval_at_multiple_points() {
        // Evaluate a polynomial at several points and verify with known values
        // p(x) = x^4 - 10x^2 + 9 = (x-1)(x+1)(x-3)(x+3)
        let p = poly_from_roots(&[1, -1, 3, -3]);
        assert_eq!(p.eval(&QRat::from((1, 1))), QRat::zero());
        assert_eq!(p.eval(&QRat::from((-1, 1))), QRat::zero());
        assert_eq!(p.eval(&QRat::from((3, 1))), QRat::zero());
        assert_eq!(p.eval(&QRat::from((-3, 1))), QRat::zero());
        assert_eq!(p.eval(&QRat::from((0, 1))), QRat::from((9, 1)));
        // p(2) = 16 - 40 + 9 = -15
        assert_eq!(p.eval(&QRat::from((2, 1))), QRat::from((-15, 1)));
    }

    #[test]
    fn test_content_preserves_polynomial() {
        // content * primitive_part == original (up to sign)
        let p = QRatPoly::from_vec(vec![
            QRat::from((2, 3)),
            QRat::from((4, 3)),
            QRat::from((8, 3)),
        ]);
        let cont = p.content();
        let pp = p.primitive_part();
        let reconstructed = pp.scalar_mul(&cont);
        assert_eq!(reconstructed, p);
    }
}
