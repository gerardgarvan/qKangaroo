//! Wrapper types around `rug::Integer` and `rug::Rational` with guaranteed `Hash` implementations.
//!
//! These newtypes ensure hash-consing compatibility: `a == b` implies `hash(a) == hash(b)`.

use rug::integer::Order;
use rug::ops::Pow;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Arbitrary-precision integer wrapper around `rug::Integer`.
///
/// Provides `Hash` via canonical digit representation, ensuring
/// the hash-consing invariant holds for the expression arena.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct QInt(pub rug::Integer);

impl Hash for QInt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the canonical byte representation (most significant first)
        let digits = self.0.to_digits::<u8>(Order::Msf);
        digits.hash(state);
        // Hash the sign to distinguish positive from negative
        self.0.cmp0().hash(state);
    }
}

impl fmt::Display for QInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --- Convenience constructors ---

impl From<i64> for QInt {
    fn from(val: i64) -> Self {
        QInt(rug::Integer::from(val))
    }
}

impl From<i32> for QInt {
    fn from(val: i32) -> Self {
        QInt(rug::Integer::from(val))
    }
}

impl From<u64> for QInt {
    fn from(val: u64) -> Self {
        QInt(rug::Integer::from(val))
    }
}

impl From<u32> for QInt {
    fn from(val: u32) -> Self {
        QInt(rug::Integer::from(val))
    }
}

impl From<rug::Integer> for QInt {
    fn from(val: rug::Integer) -> Self {
        QInt(val)
    }
}

// --- Arithmetic operations ---

impl Add for QInt {
    type Output = QInt;
    fn add(self, rhs: QInt) -> QInt {
        QInt(rug::Integer::from(&self.0 + &rhs.0))
    }
}

impl<'a> Add<&'a QInt> for &'a QInt {
    type Output = QInt;
    fn add(self, rhs: &'a QInt) -> QInt {
        QInt(rug::Integer::from(&self.0 + &rhs.0))
    }
}

impl Sub for QInt {
    type Output = QInt;
    fn sub(self, rhs: QInt) -> QInt {
        QInt(rug::Integer::from(&self.0 - &rhs.0))
    }
}

impl<'a> Sub<&'a QInt> for &'a QInt {
    type Output = QInt;
    fn sub(self, rhs: &'a QInt) -> QInt {
        QInt(rug::Integer::from(&self.0 - &rhs.0))
    }
}

impl Mul for QInt {
    type Output = QInt;
    fn mul(self, rhs: QInt) -> QInt {
        QInt(rug::Integer::from(&self.0 * &rhs.0))
    }
}

impl<'a> Mul<&'a QInt> for &'a QInt {
    type Output = QInt;
    fn mul(self, rhs: &'a QInt) -> QInt {
        QInt(rug::Integer::from(&self.0 * &rhs.0))
    }
}

impl Div for QInt {
    type Output = QInt;
    /// Integer (truncating) division. Panics if divisor is zero.
    fn div(self, rhs: QInt) -> QInt {
        assert!(
            rhs.0.cmp0() != Ordering::Equal,
            "QInt division by zero"
        );
        QInt(rug::Integer::from(&self.0 / &rhs.0))
    }
}

impl<'a> Div<&'a QInt> for &'a QInt {
    type Output = QInt;
    fn div(self, rhs: &'a QInt) -> QInt {
        assert!(
            rhs.0.cmp0() != Ordering::Equal,
            "QInt division by zero"
        );
        QInt(rug::Integer::from(&self.0 / &rhs.0))
    }
}

impl Neg for QInt {
    type Output = QInt;
    fn neg(self) -> QInt {
        QInt(rug::Integer::from(-&self.0))
    }
}

impl QInt {
    /// Zero constant.
    pub fn zero() -> Self {
        QInt(rug::Integer::from(0))
    }

    /// One constant.
    pub fn one() -> Self {
        QInt(rug::Integer::from(1))
    }

    /// Check if this integer is zero.
    pub fn is_zero(&self) -> bool {
        self.0.cmp0() == Ordering::Equal
    }

    /// Raise to a u32 power.
    pub fn pow_u32(&self, exp: u32) -> Self {
        QInt(rug::Integer::from(Pow::pow(&self.0, exp)))
    }
}

/// Arbitrary-precision rational number wrapper around `rug::Rational`.
///
/// `rug::Rational` automatically reduces to lowest terms on construction.
/// Provides `Hash` via canonical representation of numerator and denominator.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct QRat(pub rug::Rational);

impl Hash for QRat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash numerator and denominator separately using canonical byte representation
        let numer = self.0.numer();
        let denom = self.0.denom();
        numer.to_digits::<u8>(Order::Msf).hash(state);
        numer.cmp0().hash(state);
        denom.to_digits::<u8>(Order::Msf).hash(state);
        denom.cmp0().hash(state);
    }
}

impl fmt::Display for QRat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --- Convenience constructors ---

impl From<(i64, i64)> for QRat {
    fn from((num, den): (i64, i64)) -> Self {
        QRat(rug::Rational::from((
            rug::Integer::from(num),
            rug::Integer::from(den),
        )))
    }
}

impl From<(i32, i32)> for QRat {
    fn from((num, den): (i32, i32)) -> Self {
        QRat(rug::Rational::from((
            rug::Integer::from(num),
            rug::Integer::from(den),
        )))
    }
}

impl From<rug::Rational> for QRat {
    fn from(val: rug::Rational) -> Self {
        QRat(val)
    }
}

impl From<QInt> for QRat {
    fn from(val: QInt) -> Self {
        QRat(rug::Rational::from(val.0))
    }
}

// --- Arithmetic operations ---

impl Add for QRat {
    type Output = QRat;
    fn add(self, rhs: QRat) -> QRat {
        QRat(rug::Rational::from(&self.0 + &rhs.0))
    }
}

impl<'a> Add<&'a QRat> for &'a QRat {
    type Output = QRat;
    fn add(self, rhs: &'a QRat) -> QRat {
        QRat(rug::Rational::from(&self.0 + &rhs.0))
    }
}

impl Sub for QRat {
    type Output = QRat;
    fn sub(self, rhs: QRat) -> QRat {
        QRat(rug::Rational::from(&self.0 - &rhs.0))
    }
}

impl<'a> Sub<&'a QRat> for &'a QRat {
    type Output = QRat;
    fn sub(self, rhs: &'a QRat) -> QRat {
        QRat(rug::Rational::from(&self.0 - &rhs.0))
    }
}

impl Mul for QRat {
    type Output = QRat;
    fn mul(self, rhs: QRat) -> QRat {
        QRat(rug::Rational::from(&self.0 * &rhs.0))
    }
}

impl<'a> Mul<&'a QRat> for &'a QRat {
    type Output = QRat;
    fn mul(self, rhs: &'a QRat) -> QRat {
        QRat(rug::Rational::from(&self.0 * &rhs.0))
    }
}

impl Div for QRat {
    type Output = QRat;
    /// Rational division. Panics if divisor is zero.
    fn div(self, rhs: QRat) -> QRat {
        assert!(
            rhs.0.cmp0() != Ordering::Equal,
            "QRat division by zero"
        );
        QRat(rug::Rational::from(&self.0 / &rhs.0))
    }
}

impl<'a> Div<&'a QRat> for &'a QRat {
    type Output = QRat;
    fn div(self, rhs: &'a QRat) -> QRat {
        assert!(
            rhs.0.cmp0() != Ordering::Equal,
            "QRat division by zero"
        );
        QRat(rug::Rational::from(&self.0 / &rhs.0))
    }
}

impl Neg for QRat {
    type Output = QRat;
    fn neg(self) -> QRat {
        QRat(rug::Rational::from(-&self.0))
    }
}

impl QRat {
    /// Zero constant.
    pub fn zero() -> Self {
        QRat(rug::Rational::from(0))
    }

    /// One constant.
    pub fn one() -> Self {
        QRat(rug::Rational::from(1))
    }

    /// Check if this rational is zero.
    pub fn is_zero(&self) -> bool {
        self.0.cmp0() == Ordering::Equal
    }

    /// Get the numerator.
    pub fn numer(&self) -> &rug::Integer {
        self.0.numer()
    }

    /// Get the denominator.
    pub fn denom(&self) -> &rug::Integer {
        self.0.denom()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    fn hash_of<T: Hash>(val: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn qint_hash_invariant() {
        // a == b implies hash(a) == hash(b)
        let a = QInt::from(42i64);
        let b = QInt::from(42i64);
        assert_eq!(a, b);
        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn qint_hash_zero() {
        let a = QInt::from(0i64);
        let b = QInt::from(0i64);
        assert_eq!(a, b);
        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn qint_hash_negative() {
        let pos = QInt::from(42i64);
        let neg = QInt::from(-42i64);
        assert_ne!(pos, neg);
        // Hashes SHOULD differ (not guaranteed but extremely likely)
        assert_ne!(hash_of(&pos), hash_of(&neg));
    }

    #[test]
    fn qrat_hash_invariant() {
        let a = QRat::from((3i64, 4i64));
        let b = QRat::from((3i64, 4i64));
        assert_eq!(a, b);
        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn qrat_auto_reduces() {
        // 6/4 should reduce to 3/2
        let a = QRat::from((6i64, 4i64));
        let b = QRat::from((3i64, 2i64));
        assert_eq!(a, b);
        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn qint_arithmetic() {
        let a = QInt::from(10i64);
        let b = QInt::from(3i64);
        assert_eq!(a.clone() + b.clone(), QInt::from(13i64));
        assert_eq!(a.clone() - b.clone(), QInt::from(7i64));
        assert_eq!(a.clone() * b.clone(), QInt::from(30i64));
        assert_eq!(-a, QInt::from(-10i64));
    }

    #[test]
    fn qrat_arithmetic() {
        let half = QRat::from((1i64, 2i64));
        let third = QRat::from((1i64, 3i64));
        let expected_sum = QRat::from((5i64, 6i64));
        assert_eq!(half.clone() + third.clone(), expected_sum);
    }
}
