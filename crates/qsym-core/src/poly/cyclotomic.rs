//! Cyclotomic polynomial computation.
//!
//! Provides `cyclotomic_poly(n)` which computes the n-th cyclotomic polynomial
//! Phi_n(x) via recursive division: start with x^n - 1, then divide by Phi_d(x)
//! for every proper divisor d of n.

use super::QRatPoly;

/// Compute all divisors of `n` in ascending order.
fn divisors(n: usize) -> Vec<usize> {
    if n == 0 {
        return vec![];
    }
    let mut divs = Vec::new();
    let mut i = 1;
    while i * i <= n {
        if n % i == 0 {
            divs.push(i);
            if i != n / i {
                divs.push(n / i);
            }
        }
        i += 1;
    }
    divs.sort();
    divs
}

/// Construct the polynomial x^n - 1.
fn x_n_minus_1(n: usize) -> QRatPoly {
    let mut coeffs = vec![0i64; n + 1];
    coeffs[0] = -1;
    coeffs[n] = 1;
    QRatPoly::from_i64_coeffs(&coeffs)
}

/// Compute the n-th cyclotomic polynomial Phi_n(x).
///
/// Uses recursive division: Phi_n(x) = (x^n - 1) / prod_{d | n, d < n} Phi_d(x).
///
/// # Panics
///
/// Panics if `n == 0`.
pub fn cyclotomic_poly(n: usize) -> QRatPoly {
    assert!(n > 0, "cyclotomic_poly: n must be positive");

    // Base case: Phi_1(x) = x - 1
    if n == 1 {
        return QRatPoly::from_i64_coeffs(&[-1, 1]);
    }

    let mut result = x_n_minus_1(n);

    // Divide by Phi_d(x) for each proper divisor d of n
    let divs = divisors(n);
    for &d in &divs {
        if d == n {
            continue; // skip n itself
        }
        let phi_d = cyclotomic_poly(d);
        result = result.exact_div(&phi_d);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divisors() {
        assert_eq!(divisors(1), vec![1]);
        assert_eq!(divisors(6), vec![1, 2, 3, 6]);
        assert_eq!(divisors(12), vec![1, 2, 3, 4, 6, 12]);
        assert_eq!(divisors(7), vec![1, 7]);
    }

    #[test]
    fn test_x_n_minus_1() {
        let p = x_n_minus_1(3);
        assert_eq!(p, QRatPoly::from_i64_coeffs(&[-1, 0, 0, 1]));
    }

    #[test]
    fn cyclotomic_1() {
        // Phi_1(x) = x - 1
        let phi = cyclotomic_poly(1);
        assert_eq!(phi, QRatPoly::from_i64_coeffs(&[-1, 1]));
    }

    #[test]
    fn cyclotomic_2() {
        // Phi_2(x) = x + 1
        let phi = cyclotomic_poly(2);
        assert_eq!(phi, QRatPoly::from_i64_coeffs(&[1, 1]));
    }

    #[test]
    fn cyclotomic_3() {
        // Phi_3(x) = x^2 + x + 1
        let phi = cyclotomic_poly(3);
        assert_eq!(phi, QRatPoly::from_i64_coeffs(&[1, 1, 1]));
    }

    #[test]
    fn cyclotomic_4() {
        // Phi_4(x) = x^2 + 1
        let phi = cyclotomic_poly(4);
        assert_eq!(phi, QRatPoly::from_i64_coeffs(&[1, 0, 1]));
    }

    #[test]
    fn cyclotomic_6() {
        // Phi_6(x) = x^2 - x + 1
        let phi = cyclotomic_poly(6);
        assert_eq!(phi, QRatPoly::from_i64_coeffs(&[1, -1, 1]));
    }

    #[test]
    fn cyclotomic_12() {
        // Phi_12(x) = x^4 - x^2 + 1
        let phi = cyclotomic_poly(12);
        assert_eq!(phi, QRatPoly::from_i64_coeffs(&[1, 0, -1, 0, 1]));
    }

    #[test]
    fn cyclotomic_product_equals_x_n_minus_1() {
        // Product of all Phi_d(x) for d | n should equal x^n - 1
        for n in 1..=15 {
            let divs = divisors(n);
            let mut product = QRatPoly::one();
            for &d in &divs {
                product = &product * &cyclotomic_poly(d);
            }
            assert_eq!(
                product,
                x_n_minus_1(n),
                "Product of cyclotomic polys for divisors of {} should equal x^{} - 1",
                n,
                n
            );
        }
    }
}
