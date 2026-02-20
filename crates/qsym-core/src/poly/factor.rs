//! Polynomial factorization over Q[x] via cyclotomic trial division.
//!
//! Provides `factor_over_q()` which decomposes a polynomial into cyclotomic
//! and irreducible factors over the rationals, and `Factorization` which
//! stores the result with display formatting.

use super::QRatPoly;
use super::cyclotomic::cyclotomic_poly;
use crate::number::QRat;
use std::cmp::Ordering;

/// Result of polynomial factorization over Q[x].
///
/// Represents: `scalar * prod_i (factor_i)^(multiplicity_i)`
pub struct Factorization {
    /// Content factor (rational scalar).
    pub scalar: QRat,
    /// Irreducible factors with multiplicities, sorted by degree ascending.
    pub factors: Vec<(QRatPoly, usize)>,
}

/// Euler's totient function.
fn euler_phi(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let mut result = n;
    let mut m = n;
    let mut p = 2;
    while p * p <= m {
        if m % p == 0 {
            while m % p == 0 {
                m /= p;
            }
            result -= result / p;
        }
        p += 1;
    }
    if m > 1 {
        result -= result / m;
    }
    result
}

/// Factor a polynomial over Q[x] into irreducible factors.
///
/// The algorithm:
/// 1. Extract content (rational scalar) and work with the primitive part.
/// 2. Ensure the leading coefficient is positive (negate scalar if needed).
/// 3. Trial-divide by cyclotomic polynomials Phi_n from the largest possible
///    degree down to 1.
/// 4. Any remaining non-constant quotient is included as-is (assumed irreducible).
///
/// Returns a `Factorization` containing the scalar and sorted factor list.
pub fn factor_over_q(poly: &QRatPoly) -> Factorization {
    // Handle zero polynomial
    if poly.is_zero() {
        return Factorization {
            scalar: QRat::zero(),
            factors: vec![],
        };
    }

    // Handle constant polynomial
    if poly.is_constant() {
        return Factorization {
            scalar: poly.coeff(0),
            factors: vec![],
        };
    }

    // Extract content and get primitive part
    let content = poly.content();
    let mut prim = poly.primitive_part();

    // Ensure leading coefficient is positive
    let mut scalar = content;
    if let Some(lc) = prim.leading_coeff() {
        if lc.0.cmp0() == Ordering::Less {
            scalar = -scalar.clone();
            prim = -&prim;
        }
    }

    let mut remaining = prim;
    let mut factors: Vec<(QRatPoly, usize)> = Vec::new();

    // Trial divide by cyclotomic polynomials, from highest degree down to 1.
    // The degree of Phi_n is euler_phi(n). We need euler_phi(n) <= deg(remaining).
    // The maximum n to try is bounded by the degree of the remaining polynomial.
    // For safety, scan n from degree down to 1.
    let max_n = remaining.degree().unwrap_or(0);

    // We scan from large n down so we find higher-order cyclotomic factors first.
    // This is important because lower cyclotomics divide x^n - 1 alongside
    // higher ones, and we want to find the primitive factors.
    for n in (1..=max_n).rev() {
        let phi_deg = euler_phi(n);
        loop {
            let rem_deg = match remaining.degree() {
                Some(d) => d,
                None => break, // remaining is constant
            };
            if phi_deg > rem_deg {
                break;
            }
            let phi_n = cyclotomic_poly(n);
            let (q, r) = remaining.div_rem(&phi_n);
            if r.is_zero() {
                // Found a factor! Record it.
                // Check if we already have this factor (shouldn't happen on first hit
                // at this n, but handles repeated division for multiplicities).
                if let Some(entry) = factors.iter_mut().find(|(f, _)| *f == phi_n) {
                    entry.1 += 1;
                } else {
                    factors.push((phi_n, 1));
                }
                remaining = q;
            } else {
                break;
            }
        }
        if remaining.is_constant() {
            break;
        }
    }

    // If remaining is non-constant, it's an irreducible factor not captured
    // by cyclotomic trial division.
    if !remaining.is_constant() {
        // Make monic and adjust scalar
        if let Some(lc) = remaining.leading_coeff() {
            if lc != QRat::one() {
                scalar = &scalar * &lc;
                remaining = remaining.make_monic();
            }
        }
        factors.push((remaining, 1));
    } else if !remaining.is_one() {
        // Remaining is a nonzero constant -- fold into scalar
        scalar = &scalar * &remaining.coeff(0);
    }

    // Sort factors: by degree ascending, then by first differing coefficient for tiebreaking
    factors.sort_by(|(a, _), (b, _)| {
        let deg_a = a.degree().unwrap_or(0);
        let deg_b = b.degree().unwrap_or(0);
        match deg_a.cmp(&deg_b) {
            Ordering::Equal => {
                // Compare coefficients from lowest to highest
                let max_d = deg_a.max(deg_b);
                for i in 0..=max_d {
                    let ca = a.coeff(i);
                    let cb = b.coeff(i);
                    let cmp = ca.0.cmp(&cb.0);
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }
                Ordering::Equal
            }
            other => other,
        }
    });

    Factorization { scalar, factors }
}

/// Format a single polynomial with a named variable (e.g., "q") in descending
/// degree order, suitable for parenthesized factor display.
fn format_poly_with_var(poly: &QRatPoly, var: &str) -> String {
    if poly.is_zero() {
        return "0".to_string();
    }

    let mut parts = Vec::new();
    let deg = poly.degree().unwrap_or(0);

    // Iterate from highest degree down
    for i in (0..=deg).rev() {
        let c = poly.coeff(i);
        if c.is_zero() {
            continue;
        }

        let is_negative = c.0.cmp0() == Ordering::Less;
        let abs_c = QRat(rug::Rational::from(c.0.clone().abs()));
        let is_unit = abs_c == QRat::one();

        let sign = if parts.is_empty() {
            if is_negative { "-" } else { "" }
        } else if is_negative {
            "-"
        } else {
            "+"
        };

        let term = match i {
            0 => format!("{}", abs_c),
            1 => {
                if is_unit {
                    var.to_string()
                } else {
                    format!("{}*{}", abs_c, var)
                }
            }
            _ => {
                if is_unit {
                    format!("{}^{}", var, i)
                } else {
                    format!("{}*{}^{}", abs_c, var, i)
                }
            }
        };

        parts.push(format!("{}{}", sign, term));
    }

    parts.join("")
}

impl Factorization {
    /// Format the factorization using the given variable name.
    ///
    /// Each factor is displayed as a parenthesized polynomial in descending
    /// degree order. Multiplicities > 1 are shown as `^N` suffix.
    /// If scalar != 1, it is prepended.
    pub fn display_with_var(&self, var: &str) -> String {
        if self.factors.is_empty() {
            // Constant (or zero)
            return format!("{}", self.scalar);
        }

        let mut result = String::new();

        // Prepend scalar if != 1 and != -1
        let neg_one = QRat::from((-1i64, 1i64));
        if self.scalar == neg_one {
            result.push('-');
        } else if self.scalar != QRat::one() {
            result.push_str(&format!("{}", self.scalar));
        }

        for (factor, mult) in &self.factors {
            let formatted = format_poly_with_var(factor, var);
            result.push('(');
            result.push_str(&formatted);
            result.push(')');
            if *mult > 1 {
                result.push_str(&format!("^{}", mult));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euler_phi() {
        assert_eq!(euler_phi(1), 1);
        assert_eq!(euler_phi(2), 1);
        assert_eq!(euler_phi(3), 2);
        assert_eq!(euler_phi(4), 2);
        assert_eq!(euler_phi(6), 2);
        assert_eq!(euler_phi(12), 4);
    }

    #[test]
    fn factor_x6_minus_1() {
        // x^6 - 1 = Phi_1 * Phi_2 * Phi_3 * Phi_6
        // = (x-1)(x+1)(x^2+x+1)(x^2-x+1)
        let poly = QRatPoly::from_i64_coeffs(&[-1, 0, 0, 0, 0, 0, 1]);
        let f = factor_over_q(&poly);

        assert_eq!(f.scalar, QRat::one());
        assert_eq!(f.factors.len(), 4);

        // Collect factor polynomials
        let factor_polys: Vec<&QRatPoly> = f.factors.iter().map(|(p, _)| p).collect();

        // Should contain Phi_1 = x-1
        assert!(factor_polys.contains(&&QRatPoly::from_i64_coeffs(&[-1, 1])));
        // Should contain Phi_2 = x+1
        assert!(factor_polys.contains(&&QRatPoly::from_i64_coeffs(&[1, 1])));
        // Should contain Phi_3 = x^2+x+1
        assert!(factor_polys.contains(&&QRatPoly::from_i64_coeffs(&[1, 1, 1])));
        // Should contain Phi_6 = x^2-x+1
        assert!(factor_polys.contains(&&QRatPoly::from_i64_coeffs(&[1, -1, 1])));

        // All multiplicities should be 1
        for (_, mult) in &f.factors {
            assert_eq!(*mult, 1);
        }
    }

    #[test]
    fn factor_x4_minus_1() {
        // x^4 - 1 = (x-1)(x+1)(x^2+1)
        let poly = QRatPoly::from_i64_coeffs(&[-1, 0, 0, 0, 1]);
        let f = factor_over_q(&poly);

        assert_eq!(f.scalar, QRat::one());
        assert_eq!(f.factors.len(), 3);

        let factor_polys: Vec<&QRatPoly> = f.factors.iter().map(|(p, _)| p).collect();
        assert!(factor_polys.contains(&&QRatPoly::from_i64_coeffs(&[-1, 1])));
        assert!(factor_polys.contains(&&QRatPoly::from_i64_coeffs(&[1, 1])));
        assert!(factor_polys.contains(&&QRatPoly::from_i64_coeffs(&[1, 0, 1])));
    }

    #[test]
    fn factor_with_content() {
        // 2x^2 - 2 = 2(x-1)(x+1)
        let poly = QRatPoly::from_i64_coeffs(&[-2, 0, 2]);
        let f = factor_over_q(&poly);

        assert_eq!(f.scalar, QRat::from((2i64, 1i64)));
        assert_eq!(f.factors.len(), 2);
    }

    #[test]
    fn factor_single_cyclotomic() {
        // x^2 + x + 1 = Phi_3
        let poly = QRatPoly::from_i64_coeffs(&[1, 1, 1]);
        let f = factor_over_q(&poly);

        assert_eq!(f.scalar, QRat::one());
        assert_eq!(f.factors.len(), 1);
        assert_eq!(f.factors[0].0, QRatPoly::from_i64_coeffs(&[1, 1, 1]));
        assert_eq!(f.factors[0].1, 1);
    }

    #[test]
    fn factor_negative_leading_coeff() {
        // 1 - x = -(x - 1) = -1 * Phi_1
        let poly = QRatPoly::from_i64_coeffs(&[1, -1]);
        let f = factor_over_q(&poly);

        assert_eq!(f.scalar, QRat::from((-1i64, 1i64)));
        assert_eq!(f.factors.len(), 1);
        assert_eq!(f.factors[0].0, QRatPoly::from_i64_coeffs(&[-1, 1]));
    }

    #[test]
    fn factor_zero() {
        let f = factor_over_q(&QRatPoly::zero());
        assert_eq!(f.scalar, QRat::zero());
        assert!(f.factors.is_empty());
    }

    #[test]
    fn factor_constant() {
        let f = factor_over_q(&QRatPoly::from_i64_coeffs(&[5]));
        assert_eq!(f.scalar, QRat::from((5i64, 1i64)));
        assert!(f.factors.is_empty());
    }

    #[test]
    fn display_x6_minus_1() {
        let poly = QRatPoly::from_i64_coeffs(&[-1, 0, 0, 0, 0, 0, 1]);
        let f = factor_over_q(&poly);
        let display = f.display_with_var("q");
        // Should contain all four cyclotomic factors
        assert!(display.contains("("), "display should have parenthesized factors");
        // Verify product is correct by checking all 4 factors are present
        assert!(display.contains("1-q") || display.contains("-1+q") || display.contains("-q+1"),
            "should contain factor (x-1): got {}", display);
    }

    #[test]
    fn display_with_scalar() {
        // 2x^2 - 2 = 2(x-1)(x+1)
        let poly = QRatPoly::from_i64_coeffs(&[-2, 0, 2]);
        let f = factor_over_q(&poly);
        let display = f.display_with_var("q");
        assert!(display.starts_with("2"), "should start with scalar 2: got {}", display);
    }

    #[test]
    fn display_negative_scalar() {
        // 1 - x = -(x-1)
        let poly = QRatPoly::from_i64_coeffs(&[1, -1]);
        let f = factor_over_q(&poly);
        let display = f.display_with_var("q");
        assert!(display.starts_with("-"), "should start with -: got {}", display);
    }

    #[test]
    fn factor_multiplicity() {
        // (x-1)^2 = x^2 - 2x + 1
        let poly = QRatPoly::from_i64_coeffs(&[1, -2, 1]);
        let f = factor_over_q(&poly);

        assert_eq!(f.scalar, QRat::one());
        assert_eq!(f.factors.len(), 1);
        assert_eq!(f.factors[0].0, QRatPoly::from_i64_coeffs(&[-1, 1]));
        assert_eq!(f.factors[0].1, 2);
    }

    #[test]
    fn display_multiplicity() {
        // (x-1)^2
        let poly = QRatPoly::from_i64_coeffs(&[1, -2, 1]);
        let f = factor_over_q(&poly);
        let display = f.display_with_var("q");
        assert!(display.contains("^2"), "should show multiplicity: got {}", display);
    }

    #[test]
    fn factor_roundtrip() {
        // Verify that the product of factors equals the original polynomial
        let poly = QRatPoly::from_i64_coeffs(&[-1, 0, 0, 0, 0, 0, 1]); // x^6 - 1
        let f = factor_over_q(&poly);

        let mut product = QRatPoly::constant(f.scalar.clone());
        for (factor, mult) in &f.factors {
            for _ in 0..*mult {
                product = &product * factor;
            }
        }
        assert_eq!(product, poly, "factorization product should equal original");
    }

    #[test]
    fn format_poly_basic() {
        // x^2 - x + 1
        let p = QRatPoly::from_i64_coeffs(&[1, -1, 1]);
        let s = format_poly_with_var(&p, "q");
        assert_eq!(s, "q^2-q+1");
    }

    #[test]
    fn format_poly_linear() {
        // x - 1
        let p = QRatPoly::from_i64_coeffs(&[-1, 1]);
        let s = format_poly_with_var(&p, "q");
        assert_eq!(s, "q-1");
    }

    #[test]
    fn format_poly_plus_1() {
        // x + 1
        let p = QRatPoly::from_i64_coeffs(&[1, 1]);
        let s = format_poly_with_var(&p, "q");
        assert_eq!(s, "q+1");
    }
}
