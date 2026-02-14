//! Cusp computation for congruence subgroups Gamma_0(N) and Gamma_1(N).
//!
//! A cusp of a congruence subgroup is an equivalence class of points in
//! P^1(Q) = Q union {infinity} under the action of the subgroup.
//!
//! This module provides:
//! - [`Cusp`]: representation of a cusp as a/c with gcd(a,c)=1
//! - [`cuspmake`]: enumerate cusps of Gamma_0(N) (Garvan's algorithm)
//! - [`cuspmake1`]: enumerate cusps of Gamma_1(N)
//! - [`num_cusps_gamma0`]: count cusps without enumerating

use std::fmt;

/// A cusp represented as the fraction numer/denom with gcd(numer, denom) = 1.
///
/// Infinity is represented as 1/0.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cusp {
    /// Numerator a of the cusp a/c.
    pub numer: i64,
    /// Denominator c of the cusp a/c (0 for infinity).
    pub denom: i64,
}

impl Cusp {
    /// The cusp at infinity, represented as 1/0.
    pub fn infinity() -> Self {
        Cusp { numer: 1, denom: 0 }
    }

    /// Create a cusp a/c, reducing to lowest terms.
    ///
    /// If c == 0, normalizes to 1/0 (infinity).
    /// If c < 0, negates both numerator and denominator.
    pub fn new(a: i64, c: i64) -> Self {
        if c == 0 {
            return Self::infinity();
        }

        let (mut a, mut c) = (a, c);

        // Ensure denominator is positive
        if c < 0 {
            a = -a;
            c = -c;
        }

        // Reduce to lowest terms
        let g = gcd(a.abs(), c.abs());
        if g > 0 {
            a /= g;
            c /= g;
        }

        Cusp { numer: a, denom: c }
    }

    /// Returns true if this cusp is infinity (1/0).
    pub fn is_infinity(&self) -> bool {
        self.denom == 0
    }
}

impl fmt::Display for Cusp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_infinity() {
            write!(f, "inf")
        } else {
            write!(f, "{}/{}", self.numer, self.denom)
        }
    }
}

/// Greatest common divisor of two non-negative integers.
pub(crate) fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Euler's totient function phi(n): count of integers in 1..n coprime to n.
pub(crate) fn euler_phi(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut result = n;
    let mut m = n;
    let mut p = 2i64;
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

/// Number of cusps of Gamma_0(N) = sum_{d | N} phi(gcd(d, N/d)).
pub fn num_cusps_gamma0(n: i64) -> i64 {
    use crate::qseries::prodmake::divisors;
    let mut count = 0i64;
    for d in divisors(n) {
        count += euler_phi(gcd(d, n / d));
    }
    count
}

/// Enumerate inequivalent cusps of Gamma_0(N).
///
/// Algorithm (based on Garvan's ETA package):
/// 1. Start with {infinity} (= 1/0), which represents the cusp class
///    for denominator c = N.
/// 2. For each divisor c of N with 1 <= c < N:
///    a. Compute gc = gcd(c, N/c)
///    b. For each d in 0..c with gcd(d, c) = 1:
///       If d mod gc has not been seen for this c, add d/c as a new cusp
/// 3. Return all collected cusps
///
/// Two cusps d1/c and d2/c are equivalent under Gamma_0(N) iff
/// d1 = d2 (mod gcd(c, N/c)).
///
/// The number of cusps equals sum_{d|N} phi(gcd(d, N/d)).
pub fn cuspmake(n: i64) -> Vec<Cusp> {
    use crate::qseries::prodmake::divisors;

    assert!(n >= 1, "cuspmake: N must be >= 1, got {}", n);

    // Infinity represents the cusp class for denominator c = N.
    let mut cusps = vec![Cusp::infinity()];

    if n == 1 {
        return cusps;
    }

    let divs = divisors(n);
    for &c in &divs {
        // Skip c = N (represented by infinity) and c = 0 (impossible)
        if c >= n {
            continue;
        }
        let gc = gcd(c, n / c);
        let mut seen_residues: Vec<i64> = Vec::new();
        // d ranges from 0 to c-1; for c=1, this is just d=0
        for d in 0..c {
            if gcd(d, c) != 1 {
                continue;
            }
            let r = d % gc;
            if !seen_residues.contains(&r) {
                seen_residues.push(r);
                cusps.push(Cusp::new(d, c));
            }
        }
    }

    debug_assert_eq!(
        cusps.len() as i64,
        num_cusps_gamma0(n),
        "cuspmake({}): got {} cusps, expected {}",
        n,
        cusps.len(),
        num_cusps_gamma0(n)
    );

    cusps
}

/// Enumerate inequivalent cusps of Gamma_1(N).
///
/// Cusp equivalence for Gamma_1(N):
/// Two cusps u1/v1 and u2/v2 (with v1, v2 > 0, gcd(ui,vi)=1) are equivalent iff:
///   v1 = v2 and u1 = u2 (mod gcd(v1, N))
///   OR v1 = v2 and u1 = -u2 (mod gcd(v1, N)) [only when -I is in Gamma_1(N), i.e., N <= 2]
///
/// For N >= 3, -I is NOT in Gamma_1(N), so equivalence is stricter (no +/- folding).
///
/// Algorithm: Infinity represents the c=N class. For each divisor c of N
/// with 1 <= c < N, enumerate reduced fractions d/c with 0 <= d < c,
/// gcd(d,c) = 1, grouping by residue class d mod gcd(c, N).
pub fn cuspmake1(n: i64) -> Vec<Cusp> {
    use crate::qseries::prodmake::divisors;

    assert!(n >= 1, "cuspmake1: N must be >= 1, got {}", n);

    // Infinity represents the cusp class for denominator c = N.
    let mut cusps = vec![Cusp::infinity()];

    if n == 1 {
        return cusps;
    }

    let divs = divisors(n);
    for &c in &divs {
        // Skip c = N (represented by infinity)
        if c >= n {
            continue;
        }
        let gc = gcd(c, n); // Note: gcd(c, N) for Gamma_1, not gcd(c, N/c)
        let mut seen_residues: Vec<i64> = Vec::new();
        for d in 0..c {
            if gcd(d, c) != 1 {
                continue;
            }
            let r = d % gc;
            if n <= 2 {
                // -I in Gamma_1(N) for N <= 2, so +/- equivalence
                let r_neg = if r == 0 { 0 } else { gc - r };
                if !seen_residues.contains(&r) && !seen_residues.contains(&r_neg) {
                    seen_residues.push(r);
                    cusps.push(Cusp::new(d, c));
                }
            } else {
                // N >= 3: no +/- folding
                if !seen_residues.contains(&r) {
                    seen_residues.push(r);
                    cusps.push(Cusp::new(d, c));
                }
            }
        }
    }

    cusps
}
