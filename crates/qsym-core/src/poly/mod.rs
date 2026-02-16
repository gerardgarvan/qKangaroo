//! Dense univariate polynomial over QRat (arbitrary-precision rationals).
//!
//! `QRatPoly` stores coefficients in ascending degree order as a `Vec<QRat>`.
//! Invariant: the vector is either empty (zero polynomial) or the last element is nonzero.

pub mod arithmetic;
pub mod gcd;

pub use gcd::{poly_gcd, poly_resultant};

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
}
