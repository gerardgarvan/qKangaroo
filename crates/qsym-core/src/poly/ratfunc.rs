//! Rational function p(x)/q(x) over QRat with auto-simplification.
//!
//! `QRatRationalFunc` maintains invariants: gcd(numer, denom) = 1,
//! denom is monic, and denom is nonzero. These invariants enable
//! direct equality comparison of normalized forms.

use super::gcd::poly_gcd;
use super::QRatPoly;
use crate::number::QRat;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Rational function p(x)/q(x) in lowest terms with monic denominator.
///
/// Invariants:
///   1. gcd(numer, denom) = 1
///   2. denom is monic (leading coefficient = 1)
///   3. denom is nonzero
#[derive(Clone, Debug)]
pub struct QRatRationalFunc {
    pub numer: QRatPoly,
    pub denom: QRatPoly,
}

impl QRatRationalFunc {
    /// Construct a rational function, enforcing all invariants:
    /// 1. Panics if denom is zero.
    /// 2. If numer is zero, returns 0/1.
    /// 3. Divides out GCD to reduce to lowest terms.
    /// 4. Makes denominator monic.
    pub fn new(numer: QRatPoly, denom: QRatPoly) -> Self {
        assert!(
            !denom.is_zero(),
            "Rational function denominator cannot be zero"
        );

        if numer.is_zero() {
            return Self {
                numer: QRatPoly::zero(),
                denom: QRatPoly::one(),
            };
        }

        // Compute GCD and reduce
        let g = poly_gcd(&numer, &denom);
        let n = numer.exact_div(&g);
        let d = denom.exact_div(&g);

        // Make denominator monic
        let lc = d.leading_coeff().unwrap();
        if lc == QRat::one() {
            Self { numer: n, denom: d }
        } else {
            Self {
                numer: n.scalar_div(&lc),
                denom: d.scalar_div(&lc),
            }
        }
    }

    /// Create a rational function from a polynomial: p/1.
    pub fn from_poly(p: QRatPoly) -> Self {
        Self {
            numer: p,
            denom: QRatPoly::one(),
        }
    }

    /// Create a constant rational function: c/1.
    pub fn from_qrat(c: QRat) -> Self {
        Self {
            numer: QRatPoly::constant(c),
            denom: QRatPoly::one(),
        }
    }

    /// The zero rational function: 0/1.
    pub fn zero() -> Self {
        Self {
            numer: QRatPoly::zero(),
            denom: QRatPoly::one(),
        }
    }

    /// The identity rational function: 1/1.
    pub fn one() -> Self {
        Self {
            numer: QRatPoly::one(),
            denom: QRatPoly::one(),
        }
    }

    // ---- Arithmetic ----

    /// Add two rational functions: a/b + c/d = (a*d + b*c) / (b*d).
    pub fn rf_add(&self, other: &Self) -> Self {
        let numer = &(&self.numer * &other.denom) + &(&self.denom * &other.numer);
        let denom = &self.denom * &other.denom;
        Self::new(numer, denom)
    }

    /// Subtract two rational functions: a/b - c/d = (a*d - b*c) / (b*d).
    pub fn rf_sub(&self, other: &Self) -> Self {
        let numer = &(&self.numer * &other.denom) - &(&self.denom * &other.numer);
        let denom = &self.denom * &other.denom;
        Self::new(numer, denom)
    }

    /// Multiply two rational functions with cross-cancellation optimization.
    ///
    /// Instead of multiplying then reducing, cross-cancel first:
    /// (a/b) * (c/d) with g1 = gcd(a,d), g2 = gcd(c,b)
    /// = (a/g1 * c/g2) / (b/g2 * d/g1)
    pub fn rf_mul(&self, other: &Self) -> Self {
        let g1 = poly_gcd(&self.numer, &other.denom);
        let g2 = poly_gcd(&other.numer, &self.denom);
        let n1 = self.numer.exact_div(&g1);
        let d2 = other.denom.exact_div(&g1);
        let n2 = other.numer.exact_div(&g2);
        let d1 = self.denom.exact_div(&g2);
        Self::new(&n1 * &n2, &d1 * &d2)
    }

    /// Divide two rational functions: (a/b) / (c/d) = (a*d) / (b*c).
    ///
    /// Panics if other is zero.
    pub fn rf_div(&self, other: &Self) -> Self {
        assert!(
            !other.is_zero(),
            "Rational function division by zero"
        );
        Self::new(
            &self.numer * &other.denom,
            &self.denom * &other.numer,
        )
    }

    /// Negate a rational function: -(a/b) = (-a)/b.
    ///
    /// No need for `new()` since negation preserves coprimality and monicity.
    pub fn rf_neg(&self) -> Self {
        Self {
            numer: -self.numer.clone(),
            denom: self.denom.clone(),
        }
    }

    // ---- Queries ----

    /// True if this rational function is zero (numer is zero).
    pub fn is_zero(&self) -> bool {
        self.numer.is_zero()
    }

    /// True if this is a polynomial (denominator is 1).
    pub fn is_polynomial(&self) -> bool {
        self.denom.is_one()
    }

    /// Evaluate the rational function at x.
    ///
    /// Returns `None` if denom(x) = 0 (pole).
    pub fn eval(&self, x: &QRat) -> Option<QRat> {
        let d_val = self.denom.eval(x);
        if d_val.is_zero() {
            return None;
        }
        let n_val = self.numer.eval(x);
        Some(&n_val / &d_val)
    }

    // ---- q-shift ----

    /// Compute rf(q_val * x): shift both numerator and denominator.
    pub fn q_shift(&self, q_val: &QRat) -> Self {
        Self::new(self.numer.q_shift(q_val), self.denom.q_shift(q_val))
    }

    /// Compute rf(q_val^j * x) for integer j.
    pub fn q_shift_n(&self, q_val: &QRat, j: i64) -> Self {
        Self::new(
            self.numer.q_shift_n(q_val, j),
            self.denom.q_shift_n(q_val, j),
        )
    }
}

// ---- PartialEq / Eq ----
// Valid because both sides are in canonical form (lowest terms, monic denom).

impl PartialEq for QRatRationalFunc {
    fn eq(&self, other: &Self) -> bool {
        self.numer == other.numer && self.denom == other.denom
    }
}

impl Eq for QRatRationalFunc {}

// ---- Display ----

impl fmt::Display for QRatRationalFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denom.is_one() {
            write!(f, "{}", self.numer)
        } else {
            write!(f, "({}) / ({})", self.numer, self.denom)
        }
    }
}

// ---- std::ops trait implementations ----

// Add: &ref + &ref
impl Add<&QRatRationalFunc> for &QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn add(self, rhs: &QRatRationalFunc) -> QRatRationalFunc {
        self.rf_add(rhs)
    }
}

impl Add for QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn add(self, rhs: QRatRationalFunc) -> QRatRationalFunc {
        self.rf_add(&rhs)
    }
}

impl Add<&QRatRationalFunc> for QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn add(self, rhs: &QRatRationalFunc) -> QRatRationalFunc {
        self.rf_add(rhs)
    }
}

impl Add<QRatRationalFunc> for &QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn add(self, rhs: QRatRationalFunc) -> QRatRationalFunc {
        self.rf_add(&rhs)
    }
}

// Sub: &ref - &ref
impl Sub<&QRatRationalFunc> for &QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn sub(self, rhs: &QRatRationalFunc) -> QRatRationalFunc {
        self.rf_sub(rhs)
    }
}

impl Sub for QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn sub(self, rhs: QRatRationalFunc) -> QRatRationalFunc {
        self.rf_sub(&rhs)
    }
}

impl Sub<&QRatRationalFunc> for QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn sub(self, rhs: &QRatRationalFunc) -> QRatRationalFunc {
        self.rf_sub(rhs)
    }
}

impl Sub<QRatRationalFunc> for &QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn sub(self, rhs: QRatRationalFunc) -> QRatRationalFunc {
        self.rf_sub(&rhs)
    }
}

// Mul: &ref * &ref
impl Mul<&QRatRationalFunc> for &QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn mul(self, rhs: &QRatRationalFunc) -> QRatRationalFunc {
        self.rf_mul(rhs)
    }
}

impl Mul for QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn mul(self, rhs: QRatRationalFunc) -> QRatRationalFunc {
        self.rf_mul(&rhs)
    }
}

impl Mul<&QRatRationalFunc> for QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn mul(self, rhs: &QRatRationalFunc) -> QRatRationalFunc {
        self.rf_mul(rhs)
    }
}

impl Mul<QRatRationalFunc> for &QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn mul(self, rhs: QRatRationalFunc) -> QRatRationalFunc {
        self.rf_mul(&rhs)
    }
}

// Div: &ref / &ref
impl Div<&QRatRationalFunc> for &QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn div(self, rhs: &QRatRationalFunc) -> QRatRationalFunc {
        self.rf_div(rhs)
    }
}

impl Div for QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn div(self, rhs: QRatRationalFunc) -> QRatRationalFunc {
        self.rf_div(&rhs)
    }
}

impl Div<&QRatRationalFunc> for QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn div(self, rhs: &QRatRationalFunc) -> QRatRationalFunc {
        self.rf_div(rhs)
    }
}

impl Div<QRatRationalFunc> for &QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn div(self, rhs: QRatRationalFunc) -> QRatRationalFunc {
        self.rf_div(&rhs)
    }
}

// Neg
impl Neg for &QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn neg(self) -> QRatRationalFunc {
        self.rf_neg()
    }
}

impl Neg for QRatRationalFunc {
    type Output = QRatRationalFunc;
    fn neg(self) -> QRatRationalFunc {
        (&self).rf_neg()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Helper: build (x - r) ----
    fn x_minus(r: i64) -> QRatPoly {
        QRatPoly::from_vec(vec![QRat::from((-r, 1i64)), QRat::one()])
    }

    // ---- Construction and auto-reduction tests ----

    #[test]
    fn test_auto_reduce_x2_minus_1_over_x_minus_1() {
        // (x^2 - 1) / (x - 1) = (x + 1) / 1
        let numer = QRatPoly::from_i64_coeffs(&[-1, 0, 1]); // x^2 - 1
        let denom = x_minus(1); // x - 1
        let rf = QRatRationalFunc::new(numer, denom);
        assert_eq!(rf.numer, QRatPoly::from_i64_coeffs(&[1, 1])); // x + 1
        assert_eq!(rf.denom, QRatPoly::one());
        assert!(rf.is_polynomial());
    }

    #[test]
    fn test_auto_reduce_monic_denom() {
        // (2x) / (2x + 2) should simplify to x / (x + 1) with monic denom
        let numer = QRatPoly::from_i64_coeffs(&[0, 2]); // 2x
        let denom = QRatPoly::from_i64_coeffs(&[2, 2]); // 2x + 2
        let rf = QRatRationalFunc::new(numer, denom);
        assert_eq!(rf.numer, QRatPoly::from_i64_coeffs(&[0, 1])); // x
        assert_eq!(rf.denom, QRatPoly::from_i64_coeffs(&[1, 1])); // x + 1
    }

    #[test]
    fn test_auto_reduce_common_scalar() {
        // (6x^2 + 3x) / (3x) = (2x + 1) / 1
        let numer = QRatPoly::from_i64_coeffs(&[0, 3, 6]); // 6x^2 + 3x
        let denom = QRatPoly::from_i64_coeffs(&[0, 3]); // 3x
        let rf = QRatRationalFunc::new(numer, denom);
        assert_eq!(rf.numer, QRatPoly::from_i64_coeffs(&[1, 2])); // 2x + 1
        assert_eq!(rf.denom, QRatPoly::one());
    }

    #[test]
    fn test_zero_numerator() {
        // 0 / (x + 1) = 0/1
        let rf = QRatRationalFunc::new(QRatPoly::zero(), QRatPoly::from_i64_coeffs(&[1, 1]));
        assert!(rf.is_zero());
        assert_eq!(rf.numer, QRatPoly::zero());
        assert_eq!(rf.denom, QRatPoly::one());
    }

    #[test]
    #[should_panic(expected = "Rational function denominator cannot be zero")]
    fn test_zero_denominator_panics() {
        let _ = QRatRationalFunc::new(QRatPoly::one(), QRatPoly::zero());
    }

    // ---- Convenience constructors ----

    #[test]
    fn test_from_poly() {
        let p = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let rf = QRatRationalFunc::from_poly(p.clone());
        assert_eq!(rf.numer, p);
        assert!(rf.is_polynomial());
    }

    #[test]
    fn test_from_qrat() {
        let rf = QRatRationalFunc::from_qrat(QRat::from((3, 7)));
        assert_eq!(rf.numer, QRatPoly::constant(QRat::from((3, 7))));
        assert!(rf.is_polynomial());
    }

    #[test]
    fn test_zero_and_one() {
        assert!(QRatRationalFunc::zero().is_zero());
        let one = QRatRationalFunc::one();
        assert!(!one.is_zero());
        assert!(one.is_polynomial());
        assert_eq!(one.numer, QRatPoly::one());
    }

    // ---- Arithmetic tests ----

    #[test]
    fn test_add_1_over_x_plus_1_over_xp1() {
        // 1/x + 1/(x+1) = (2x+1) / (x^2 + x)
        let a = QRatRationalFunc::new(QRatPoly::one(), QRatPoly::x());
        let b = QRatRationalFunc::new(QRatPoly::one(), QRatPoly::from_i64_coeffs(&[1, 1]));
        let sum = &a + &b;
        assert_eq!(sum.numer, QRatPoly::from_i64_coeffs(&[1, 2])); // 2x + 1
        assert_eq!(sum.denom, QRatPoly::from_i64_coeffs(&[0, 1, 1])); // x^2 + x
    }

    #[test]
    fn test_mul_poly_times_poly() {
        // (x+1)/1 * (x-1)/1 = (x^2 - 1)/1
        let a = QRatRationalFunc::from_poly(QRatPoly::from_i64_coeffs(&[1, 1]));
        let b = QRatRationalFunc::from_poly(QRatPoly::from_i64_coeffs(&[-1, 1]));
        let product = &a * &b;
        assert_eq!(product.numer, QRatPoly::from_i64_coeffs(&[-1, 0, 1]));
        assert!(product.is_polynomial());
    }

    #[test]
    fn test_mul_cross_cancellation() {
        // ((x+1)/(x-1)) * ((x-1)/(x+2)) = (x+1)/(x+2)
        let a = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[1, 1]),  // x + 1
            QRatPoly::from_i64_coeffs(&[-1, 1]), // x - 1
        );
        let b = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[-1, 1]), // x - 1
            QRatPoly::from_i64_coeffs(&[2, 1]),  // x + 2
        );
        let product = &a * &b;
        assert_eq!(product.numer, QRatPoly::from_i64_coeffs(&[1, 1])); // x + 1
        assert_eq!(product.denom, QRatPoly::from_i64_coeffs(&[2, 1])); // x + 2
    }

    #[test]
    fn test_div_reciprocal() {
        // (1/x) / (1/(x+1)) = (x+1)/x
        let a = QRatRationalFunc::new(QRatPoly::one(), QRatPoly::x());
        let b = QRatRationalFunc::new(QRatPoly::one(), QRatPoly::from_i64_coeffs(&[1, 1]));
        let result = &a / &b;
        assert_eq!(result.numer, QRatPoly::from_i64_coeffs(&[1, 1])); // x + 1
        assert_eq!(result.denom, QRatPoly::x()); // x
    }

    #[test]
    fn test_neg() {
        // -(x/(x+1)) = (-x)/(x+1)
        let rf = QRatRationalFunc::new(
            QRatPoly::x(),
            QRatPoly::from_i64_coeffs(&[1, 1]),
        );
        let neg = -&rf;
        assert_eq!(neg.numer, QRatPoly::from_i64_coeffs(&[0, -1])); // -x
        assert_eq!(neg.denom, QRatPoly::from_i64_coeffs(&[1, 1])); // x + 1
    }

    #[test]
    fn test_sub_basic() {
        // 1/x - 1/(x+1) = 1/(x^2 + x)
        let a = QRatRationalFunc::new(QRatPoly::one(), QRatPoly::x());
        let b = QRatRationalFunc::new(QRatPoly::one(), QRatPoly::from_i64_coeffs(&[1, 1]));
        let diff = &a - &b;
        assert_eq!(diff.numer, QRatPoly::one()); // 1
        assert_eq!(diff.denom, QRatPoly::from_i64_coeffs(&[0, 1, 1])); // x^2 + x
    }

    // ---- Query tests ----

    #[test]
    fn test_is_zero() {
        assert!(QRatRationalFunc::zero().is_zero());
        assert!(!QRatRationalFunc::one().is_zero());
    }

    #[test]
    fn test_is_polynomial() {
        assert!(QRatRationalFunc::from_poly(QRatPoly::from_i64_coeffs(&[1, 1])).is_polynomial());
        let rf = QRatRationalFunc::new(QRatPoly::x(), QRatPoly::from_i64_coeffs(&[1, 1]));
        assert!(!rf.is_polynomial());
    }

    #[test]
    fn test_eval_regular_point() {
        // (x + 1) / x at x = 2 => 3/2
        let rf = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[1, 1]),
            QRatPoly::x(),
        );
        let result = rf.eval(&QRat::from((2, 1)));
        assert_eq!(result, Some(QRat::from((3, 2))));
    }

    #[test]
    fn test_eval_at_pole() {
        // x / (x - 1) at x = 1 => pole (None)
        let rf = QRatRationalFunc::new(
            QRatPoly::x(),
            x_minus(1),
        );
        assert_eq!(rf.eval(&QRat::from((1, 1))), None);
    }

    // ---- q-shift tests ----

    #[test]
    fn test_q_shift_both() {
        // q_shift should shift both numer and denom
        let rf = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[1, 1]), // x + 1
            QRatPoly::from_i64_coeffs(&[-1, 1]), // x - 1
        );
        let q = QRat::from((2, 1));
        let shifted = rf.q_shift(&q);

        // numer shifts: 1 + x -> 1 + 2x
        // denom shifts: -1 + x -> -1 + 2x
        // After auto-reduction (coprime), should give (1 + 2x) / (-1 + 2x)
        // Make monic: denom lc = 2, so divide both by 2:
        // numer = (1/2 + x), denom = (-1/2 + x)
        let expected_numer = QRatPoly::from_vec(vec![QRat::from((1, 2)), QRat::one()]);
        let expected_denom = QRatPoly::from_vec(vec![QRat::from((-1, 2)), QRat::one()]);
        assert_eq!(shifted.numer, expected_numer);
        assert_eq!(shifted.denom, expected_denom);
    }

    #[test]
    fn test_q_shift_n_roundtrip() {
        // rf.q_shift_n(q, 2).q_shift_n(q, -2) == rf
        let rf = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[1, 2, 3]), // 3x^2 + 2x + 1
            QRatPoly::from_i64_coeffs(&[1, 1]),     // x + 1
        );
        let q = QRat::from((3, 2));
        let roundtrip = rf.q_shift_n(&q, 2).q_shift_n(&q, -2);
        assert_eq!(roundtrip, rf);
    }

    // ---- Equality tests ----

    #[test]
    fn test_equality_same_construction() {
        let a = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[-1, 0, 1]), // x^2 - 1
            QRatPoly::from_i64_coeffs(&[-1, 1]),     // x - 1
        );
        let b = QRatRationalFunc::from_poly(QRatPoly::from_i64_coeffs(&[1, 1])); // x + 1
        assert_eq!(a, b, "(x^2-1)/(x-1) should equal (x+1)/1");
    }

    #[test]
    fn test_inequality() {
        let a = QRatRationalFunc::new(QRatPoly::x(), QRatPoly::from_i64_coeffs(&[1, 1]));
        let b = QRatRationalFunc::new(QRatPoly::x(), QRatPoly::from_i64_coeffs(&[2, 1]));
        assert_ne!(a, b);
    }

    // ---- Display tests ----

    #[test]
    fn test_display_polynomial() {
        let rf = QRatRationalFunc::from_poly(QRatPoly::from_i64_coeffs(&[1, 1]));
        assert_eq!(format!("{}", rf), "x + 1");
    }

    #[test]
    fn test_display_rational() {
        let rf = QRatRationalFunc::new(QRatPoly::x(), QRatPoly::from_i64_coeffs(&[1, 1]));
        assert_eq!(format!("{}", rf), "(x) / (x + 1)");
    }

    // ---- Trait operator tests ----

    #[test]
    fn test_ops_by_value() {
        let a = QRatRationalFunc::from_poly(QRatPoly::from_i64_coeffs(&[1, 1]));
        let b = QRatRationalFunc::from_poly(QRatPoly::from_i64_coeffs(&[-1, 1]));
        // by-value mul
        let product = a * b;
        assert_eq!(product.numer, QRatPoly::from_i64_coeffs(&[-1, 0, 1]));
        assert!(product.is_polynomial());
    }

    #[test]
    fn test_add_sub_inverse() {
        // a + b - b = a
        let a = QRatRationalFunc::new(QRatPoly::one(), QRatPoly::x());
        let b = QRatRationalFunc::new(QRatPoly::one(), QRatPoly::from_i64_coeffs(&[1, 1]));
        let result = &(&a + &b) - &b;
        assert_eq!(result, a);
    }

    #[test]
    fn test_mul_div_inverse() {
        // a * b / b = a
        let a = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[1, 1]),
            QRatPoly::from_i64_coeffs(&[2, 1]),
        );
        let b = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[-1, 1]),
            QRatPoly::from_i64_coeffs(&[3, 1]),
        );
        let result = &(&a * &b) / &b;
        assert_eq!(result, a);
    }

    #[test]
    fn test_neg_double() {
        // -(-a) = a
        let a = QRatRationalFunc::new(
            QRatPoly::from_i64_coeffs(&[1, 2]),
            QRatPoly::from_i64_coeffs(&[3, 1]),
        );
        let double_neg = -(-a.clone());
        assert_eq!(double_neg, a);
    }
}
