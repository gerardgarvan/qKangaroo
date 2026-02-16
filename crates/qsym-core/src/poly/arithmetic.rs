//! Arithmetic operations for QRatPoly.
//!
//! Provides Add, Sub, Mul, Neg trait implementations (by value and by reference),
//! scalar operations, and polynomial division (div_rem, exact_div, pseudo_rem).

use super::QRatPoly;
use crate::number::QRat;
use std::ops::{Add, Mul, Neg, Sub};

// ---- Add ----

impl Add<&QRatPoly> for &QRatPoly {
    type Output = QRatPoly;
    fn add(self, rhs: &QRatPoly) -> QRatPoly {
        let len = self.coeffs().len().max(rhs.coeffs().len());
        let mut coeffs = Vec::with_capacity(len);
        for i in 0..len {
            coeffs.push(&self.coeff(i) + &rhs.coeff(i));
        }
        QRatPoly::from_vec(coeffs)
    }
}

impl Add for QRatPoly {
    type Output = QRatPoly;
    fn add(self, rhs: QRatPoly) -> QRatPoly {
        &self + &rhs
    }
}

impl Add<&QRatPoly> for QRatPoly {
    type Output = QRatPoly;
    fn add(self, rhs: &QRatPoly) -> QRatPoly {
        &self + rhs
    }
}

impl Add<QRatPoly> for &QRatPoly {
    type Output = QRatPoly;
    fn add(self, rhs: QRatPoly) -> QRatPoly {
        self + &rhs
    }
}

// ---- Sub ----

impl Sub<&QRatPoly> for &QRatPoly {
    type Output = QRatPoly;
    fn sub(self, rhs: &QRatPoly) -> QRatPoly {
        let len = self.coeffs().len().max(rhs.coeffs().len());
        let mut coeffs = Vec::with_capacity(len);
        for i in 0..len {
            coeffs.push(&self.coeff(i) - &rhs.coeff(i));
        }
        QRatPoly::from_vec(coeffs)
    }
}

impl Sub for QRatPoly {
    type Output = QRatPoly;
    fn sub(self, rhs: QRatPoly) -> QRatPoly {
        &self - &rhs
    }
}

impl Sub<&QRatPoly> for QRatPoly {
    type Output = QRatPoly;
    fn sub(self, rhs: &QRatPoly) -> QRatPoly {
        &self - rhs
    }
}

impl Sub<QRatPoly> for &QRatPoly {
    type Output = QRatPoly;
    fn sub(self, rhs: QRatPoly) -> QRatPoly {
        self - &rhs
    }
}

// ---- Neg ----

impl Neg for &QRatPoly {
    type Output = QRatPoly;
    fn neg(self) -> QRatPoly {
        let coeffs: Vec<QRat> = self.coeffs().iter().map(|c| -c.clone()).collect();
        QRatPoly::from_vec(coeffs)
    }
}

impl Neg for QRatPoly {
    type Output = QRatPoly;
    fn neg(self) -> QRatPoly {
        -&self
    }
}

// ---- Mul ----

impl Mul<&QRatPoly> for &QRatPoly {
    type Output = QRatPoly;
    fn mul(self, rhs: &QRatPoly) -> QRatPoly {
        if self.is_zero() || rhs.is_zero() {
            return QRatPoly::zero();
        }
        let a = self.coeffs();
        let b = rhs.coeffs();
        let result_len = a.len() + b.len() - 1;
        let mut coeffs = vec![QRat::zero(); result_len];
        for (i, ai) in a.iter().enumerate() {
            if ai.is_zero() {
                continue;
            }
            for (j, bj) in b.iter().enumerate() {
                if bj.is_zero() {
                    continue;
                }
                let product = ai * bj;
                coeffs[i + j] = &coeffs[i + j] + &product;
            }
        }
        QRatPoly::from_vec(coeffs)
    }
}

impl Mul for QRatPoly {
    type Output = QRatPoly;
    fn mul(self, rhs: QRatPoly) -> QRatPoly {
        &self * &rhs
    }
}

impl Mul<&QRatPoly> for QRatPoly {
    type Output = QRatPoly;
    fn mul(self, rhs: &QRatPoly) -> QRatPoly {
        &self * rhs
    }
}

impl Mul<QRatPoly> for &QRatPoly {
    type Output = QRatPoly;
    fn mul(self, rhs: QRatPoly) -> QRatPoly {
        self * &rhs
    }
}

// ---- Division ----

impl QRatPoly {
    /// Euclidean polynomial division: returns (quotient, remainder) such that
    /// `self = quotient * divisor + remainder` and `deg(remainder) < deg(divisor)`.
    ///
    /// Panics if divisor is zero.
    pub fn div_rem(&self, divisor: &QRatPoly) -> (QRatPoly, QRatPoly) {
        assert!(!divisor.is_zero(), "QRatPoly::div_rem: division by zero");

        let d_deg = match divisor.degree() {
            Some(d) => d,
            None => unreachable!(), // already checked is_zero
        };
        let s_deg = match self.degree() {
            Some(d) => d,
            None => return (QRatPoly::zero(), QRatPoly::zero()),
        };

        if s_deg < d_deg {
            return (QRatPoly::zero(), self.clone());
        }

        let lc_d = divisor.leading_coeff().unwrap();
        let mut remainder: Vec<QRat> = self.coeffs().to_vec();
        let q_len = s_deg - d_deg + 1;
        let mut quotient = vec![QRat::zero(); q_len];

        for i in (0..q_len).rev() {
            let idx = i + d_deg;
            if idx >= remainder.len() {
                continue;
            }
            let q_coeff = &remainder[idx] / &lc_d;
            if q_coeff.is_zero() {
                continue;
            }
            quotient[i] = q_coeff.clone();
            // Subtract divisor * q_coeff shifted by i
            for (j, dj) in divisor.coeffs().iter().enumerate() {
                let term = dj * &q_coeff;
                remainder[i + j] = &remainder[i + j] - &term;
            }
        }

        (QRatPoly::from_vec(quotient), QRatPoly::from_vec(remainder))
    }

    /// Exact division: returns `self / divisor` and panics if there is a nonzero remainder.
    pub fn exact_div(&self, divisor: &QRatPoly) -> QRatPoly {
        let (q, r) = self.div_rem(divisor);
        assert!(r.is_zero(), "exact_div: nonzero remainder");
        q
    }

    /// Pseudo-remainder: computes `lc(other)^delta * self mod other` where
    /// `delta = deg(self) - deg(other) + 1`.
    ///
    /// This avoids fractions when working with integer-coefficient polynomials.
    pub fn pseudo_rem(&self, other: &QRatPoly) -> QRatPoly {
        if self.is_zero() {
            return QRatPoly::zero();
        }
        let s_deg = match self.degree() {
            Some(d) => d,
            None => return QRatPoly::zero(),
        };
        let o_deg = match other.degree() {
            Some(d) => d,
            None => panic!("QRatPoly::pseudo_rem: division by zero"),
        };
        if s_deg < o_deg {
            return self.clone();
        }

        let delta = s_deg - o_deg + 1;
        let lc_other = other.leading_coeff().unwrap();

        // Scale self by lc(other)^delta
        let mut scale = QRat::one();
        for _ in 0..delta {
            scale = &scale * &lc_other;
        }
        let scaled = self.scalar_mul(&scale);

        let (_q, r) = scaled.div_rem(other);
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Add tests ----

    #[test]
    fn test_add_basic() {
        // (x^2 + 1) + (2x + 3) = x^2 + 2x + 4
        let a = QRatPoly::from_i64_coeffs(&[1, 0, 1]);
        let b = QRatPoly::from_i64_coeffs(&[3, 2]);
        let sum = &a + &b;
        assert_eq!(sum, QRatPoly::from_i64_coeffs(&[4, 2, 1]));
    }

    #[test]
    fn test_add_cancellation() {
        // (x^2 + x) + (-x^2 + 1) = x + 1
        let a = QRatPoly::from_i64_coeffs(&[0, 1, 1]);
        let b = QRatPoly::from_i64_coeffs(&[1, 0, -1]);
        let sum = &a + &b;
        assert_eq!(sum, QRatPoly::from_i64_coeffs(&[1, 1]));
    }

    #[test]
    fn test_add_zero() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let z = QRatPoly::zero();
        assert_eq!(&a + &z, a.clone());
        assert_eq!(&z + &a, a);
    }

    // ---- Sub tests ----

    #[test]
    fn test_sub_to_zero() {
        // (x^2 + x + 1) - (x^2 + x + 1) = 0
        let a = QRatPoly::from_i64_coeffs(&[1, 1, 1]);
        let result = &a - &a;
        assert!(result.is_zero());
    }

    #[test]
    fn test_sub_basic() {
        // (x^2 + 2x + 3) - (x + 1) = x^2 + x + 2
        let a = QRatPoly::from_i64_coeffs(&[3, 2, 1]);
        let b = QRatPoly::from_i64_coeffs(&[1, 1]);
        let diff = &a - &b;
        assert_eq!(diff, QRatPoly::from_i64_coeffs(&[2, 1, 1]));
    }

    // ---- Mul tests ----

    #[test]
    fn test_mul_difference_of_squares() {
        // (x + 1)(x - 1) = x^2 - 1
        let a = QRatPoly::from_i64_coeffs(&[1, 1]);
        let b = QRatPoly::from_i64_coeffs(&[-1, 1]);
        let product = &a * &b;
        assert_eq!(product, QRatPoly::from_i64_coeffs(&[-1, 0, 1]));
    }

    #[test]
    fn test_mul_perfect_square() {
        // (x + 1)^2 = x^2 + 2x + 1
        let a = QRatPoly::from_i64_coeffs(&[1, 1]);
        let product = &a * &a;
        assert_eq!(product, QRatPoly::from_i64_coeffs(&[1, 2, 1]));
    }

    #[test]
    fn test_mul_by_zero() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let z = QRatPoly::zero();
        assert!((a * z).is_zero());
    }

    #[test]
    fn test_mul_by_one() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let one = QRatPoly::one();
        assert_eq!(&a * &one, a);
    }

    // ---- Neg tests ----

    #[test]
    fn test_neg() {
        // -(x + 1) = -x - 1
        let a = QRatPoly::from_i64_coeffs(&[1, 1]);
        let neg_a = -&a;
        assert_eq!(neg_a, QRatPoly::from_i64_coeffs(&[-1, -1]));
    }

    #[test]
    fn test_neg_zero() {
        assert!((-QRatPoly::zero()).is_zero());
    }

    // ---- Scalar operations tests ----

    #[test]
    fn test_scalar_mul() {
        // 3 * (x^2 + 1) = 3x^2 + 3
        let p = QRatPoly::from_i64_coeffs(&[1, 0, 1]);
        let result = p.scalar_mul(&QRat::from((3, 1)));
        assert_eq!(result, QRatPoly::from_i64_coeffs(&[3, 0, 3]));
    }

    #[test]
    fn test_scalar_mul_by_zero() {
        let p = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        assert!(p.scalar_mul(&QRat::zero()).is_zero());
    }

    #[test]
    fn test_scalar_div() {
        // (6x^2 + 4x + 2) / 2 = 3x^2 + 2x + 1
        let p = QRatPoly::from_i64_coeffs(&[2, 4, 6]);
        let result = p.scalar_div(&QRat::from((2, 1)));
        assert_eq!(result, QRatPoly::from_i64_coeffs(&[1, 2, 3]));
    }

    // ---- div_rem tests ----

    #[test]
    fn test_div_rem_exact() {
        // (x^2 - 1) / (x - 1) = (x + 1, 0)
        let a = QRatPoly::from_i64_coeffs(&[-1, 0, 1]);
        let b = QRatPoly::from_i64_coeffs(&[-1, 1]);
        let (q, r) = a.div_rem(&b);
        assert_eq!(q, QRatPoly::from_i64_coeffs(&[1, 1]));
        assert!(r.is_zero());
    }

    #[test]
    fn test_div_rem_with_remainder() {
        // x^2 / (x - 1) = (x + 1, 1)
        let a = QRatPoly::from_i64_coeffs(&[0, 0, 1]);
        let b = QRatPoly::from_i64_coeffs(&[-1, 1]);
        let (q, r) = a.div_rem(&b);
        assert_eq!(q, QRatPoly::from_i64_coeffs(&[1, 1]));
        assert_eq!(r, QRatPoly::from_i64_coeffs(&[1]));
    }

    #[test]
    fn test_div_rem_smaller_dividend() {
        // x / (x^2 + 1) = (0, x)
        let a = QRatPoly::from_i64_coeffs(&[0, 1]);
        let b = QRatPoly::from_i64_coeffs(&[1, 0, 1]);
        let (q, r) = a.div_rem(&b);
        assert!(q.is_zero());
        assert_eq!(r, QRatPoly::from_i64_coeffs(&[0, 1]));
    }

    #[test]
    fn test_div_rem_constant_divisor() {
        // (6x^2 + 4) / 2 = (3x^2 + 2, 0)
        let a = QRatPoly::from_i64_coeffs(&[4, 0, 6]);
        let b = QRatPoly::from_i64_coeffs(&[2]);
        let (q, r) = a.div_rem(&b);
        assert_eq!(q, QRatPoly::from_i64_coeffs(&[2, 0, 3]));
        assert!(r.is_zero());
    }

    #[test]
    fn test_div_rem_zero_dividend() {
        let a = QRatPoly::zero();
        let b = QRatPoly::from_i64_coeffs(&[1, 1]);
        let (q, r) = a.div_rem(&b);
        assert!(q.is_zero());
        assert!(r.is_zero());
    }

    #[test]
    fn test_div_rem_identity() {
        // For any a, b (b nonzero): a = q*b + r
        let a = QRatPoly::from_i64_coeffs(&[3, -2, 0, 1, 5]); // 5x^4 + x^3 - 2x + 3
        let b = QRatPoly::from_i64_coeffs(&[1, 0, 1]); // x^2 + 1
        let (q, r) = a.div_rem(&b);
        let reconstructed = &(&q * &b) + &r;
        assert_eq!(reconstructed, a);
    }

    // ---- exact_div tests ----

    #[test]
    fn test_exact_div() {
        // (x^2 - 1) / (x - 1) = x + 1
        let a = QRatPoly::from_i64_coeffs(&[-1, 0, 1]);
        let b = QRatPoly::from_i64_coeffs(&[-1, 1]);
        let q = a.exact_div(&b);
        assert_eq!(q, QRatPoly::from_i64_coeffs(&[1, 1]));
    }

    #[test]
    #[should_panic(expected = "exact_div: nonzero remainder")]
    fn test_exact_div_panics_on_remainder() {
        let a = QRatPoly::from_i64_coeffs(&[0, 0, 1]); // x^2
        let b = QRatPoly::from_i64_coeffs(&[-1, 1]); // x - 1
        let _ = a.exact_div(&b); // x^2 / (x-1) has remainder 1
    }

    // ---- pseudo_rem tests ----

    #[test]
    fn test_pseudo_rem() {
        // a = 3x^3 + x + 2, b = x^2 + x + 1
        // lc(b) = 1, delta = 3 - 2 + 1 = 2
        // lc(b)^2 * a = a, so pseudo_rem = remainder of a / b
        let a = QRatPoly::from_i64_coeffs(&[2, 1, 0, 3]);
        let b = QRatPoly::from_i64_coeffs(&[1, 1, 1]);
        let prem = a.pseudo_rem(&b);
        // a = 3x^3 + x + 2 = (3x - 3)(x^2 + x + 1) + (x + 5)
        // Verify via div_rem directly
        let (_q, r) = a.div_rem(&b);
        assert_eq!(prem, r);
    }

    #[test]
    fn test_pseudo_rem_nonunit_lc() {
        // a = x^2, b = 2x + 1
        // lc(b) = 2, delta = 2 - 1 + 1 = 2
        // pseudo_rem = 2^2 * x^2 mod (2x+1) = 4x^2 mod (2x+1)
        let a = QRatPoly::from_i64_coeffs(&[0, 0, 1]);
        let b = QRatPoly::from_i64_coeffs(&[1, 2]);
        let prem = a.pseudo_rem(&b);

        // 4x^2 / (2x+1):
        // 4x^2 = (2x - 1)(2x + 1) + 1
        // So pseudo_rem = 1
        assert_eq!(prem, QRatPoly::from_i64_coeffs(&[1]));
    }

    #[test]
    fn test_pseudo_rem_zero_dividend() {
        let a = QRatPoly::zero();
        let b = QRatPoly::from_i64_coeffs(&[1, 1]);
        assert!(a.pseudo_rem(&b).is_zero());
    }

    #[test]
    fn test_pseudo_rem_smaller_dividend() {
        // deg(a) < deg(b) => return a
        let a = QRatPoly::from_i64_coeffs(&[1, 1]);
        let b = QRatPoly::from_i64_coeffs(&[1, 0, 1]);
        assert_eq!(a.pseudo_rem(&b), a);
    }

    // ---- Ring axiom tests ----

    #[test]
    fn test_add_commutative() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let b = QRatPoly::from_i64_coeffs(&[4, 5]);
        assert_eq!(&a + &b, &b + &a);
    }

    #[test]
    fn test_add_associative() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2]);
        let b = QRatPoly::from_i64_coeffs(&[3, 0, 1]);
        let c = QRatPoly::from_i64_coeffs(&[-1, 1]);
        assert_eq!(&(&a + &b) + &c, &a + &(&b + &c));
    }

    #[test]
    fn test_mul_commutative() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let b = QRatPoly::from_i64_coeffs(&[4, 5]);
        assert_eq!(&a * &b, &b * &a);
    }

    #[test]
    fn test_mul_associative() {
        let a = QRatPoly::from_i64_coeffs(&[1, 1]);
        let b = QRatPoly::from_i64_coeffs(&[1, -1]);
        let c = QRatPoly::from_i64_coeffs(&[2, 1]);
        assert_eq!(&(&a * &b) * &c, &a * &(&b * &c));
    }

    #[test]
    fn test_distributive() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2]);
        let b = QRatPoly::from_i64_coeffs(&[3, 1]);
        let c = QRatPoly::from_i64_coeffs(&[-1, 0, 1]);
        // a * (b + c) = a*b + a*c
        let lhs = &a * &(&b + &c);
        let rhs = &(&a * &b) + &(&a * &c);
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn test_additive_identity() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let z = QRatPoly::zero();
        assert_eq!(&a + &z, a);
    }

    #[test]
    fn test_additive_inverse() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let neg_a = -&a;
        assert!((&a + &neg_a).is_zero());
    }

    #[test]
    fn test_multiplicative_identity() {
        let a = QRatPoly::from_i64_coeffs(&[1, 2, 3]);
        let one = QRatPoly::one();
        assert_eq!(&a * &one, a);
    }

    // ---- By-value variant tests ----

    #[test]
    fn test_add_by_value() {
        let a = QRatPoly::from_i64_coeffs(&[1, 1]);
        let b = QRatPoly::from_i64_coeffs(&[2, 2]);
        let sum = a + b;
        assert_eq!(sum, QRatPoly::from_i64_coeffs(&[3, 3]));
    }

    #[test]
    fn test_sub_by_value() {
        let a = QRatPoly::from_i64_coeffs(&[3, 3]);
        let b = QRatPoly::from_i64_coeffs(&[1, 1]);
        let diff = a - b;
        assert_eq!(diff, QRatPoly::from_i64_coeffs(&[2, 2]));
    }

    #[test]
    fn test_mul_by_value() {
        let a = QRatPoly::from_i64_coeffs(&[1, 1]);
        let b = QRatPoly::from_i64_coeffs(&[1, 1]);
        let product = a * b;
        assert_eq!(product, QRatPoly::from_i64_coeffs(&[1, 2, 1]));
    }

    #[test]
    fn test_neg_by_value() {
        let a = QRatPoly::from_i64_coeffs(&[1, -2]);
        let neg_a = -a;
        assert_eq!(neg_a, QRatPoly::from_i64_coeffs(&[-1, 2]));
    }
}
