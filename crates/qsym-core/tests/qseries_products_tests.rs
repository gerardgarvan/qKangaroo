//! Comprehensive tests for named infinite product functions:
//! etaq, jacprod, tripleprod, quinprod, winquist.
//!
//! Tests verify:
//! - etaq matches euler function generator for step 1
//! - etaq with step 2 produces correct even-exponent product
//! - jacprod(1,2) produces theta4 coefficients
//! - tripleprod verified against bilateral series sum
//! - quinprod basic verification
//! - winquist basic verification
//! - etaq^2 builds (q;q)_inf^2 correctly

use qsym_core::number::QRat;
use qsym_core::symbol::SymbolId;
use qsym_core::ExprArena;
use qsym_core::series::generator::euler_function_generator;
use qsym_core::series::arithmetic;
use qsym_core::qseries::{QMonomial, etaq, jacprod, tripleprod, quinprod, winquist};

/// Helper: create a SymbolId for "q".
fn q_var() -> SymbolId {
    let mut arena = ExprArena::new();
    arena.symbols_mut().intern("q")
}

/// Helper: create QRat from i64.
fn qrat(n: i64) -> QRat {
    QRat::from((n, 1i64))
}

// ===========================================================================
// 1. etaq tests
// ===========================================================================

/// etaq(1, 1, q, 30) should match euler_function_generator to O(q^30).
/// (q; q)_inf = prod_{n>=0}(1 - q^{1+n}) = prod_{k>=1}(1 - q^k)
#[test]
fn etaq_1_1_is_euler() {
    let q = q_var();
    let trunc = 30;

    let eta = etaq(1, 1, q, trunc);

    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler = euler_gen.into_series();

    for k in 0..trunc {
        assert_eq!(
            eta.coeff(k), euler.coeff(k),
            "etaq(1,1) vs euler: mismatch at q^{}", k
        );
    }
}

/// etaq(2, 2, q, 30) = (q^2; q^2)_inf = prod_{n>=1}(1 - q^{2n}).
/// At odd exponents, all coefficients should be zero.
/// Known coefficients from the pentagonal number theorem applied with q -> q^2:
/// The generalized pentagonal numbers for step 2 are: 2, 10, 4, 14, 12, 22, 26, 34, ...
/// Actually, (q^2;q^2)_inf coefficients at even exponents match (q;q)_inf pattern scaled.
#[test]
fn etaq_2_2_even_only() {
    let q = q_var();
    let trunc = 30;

    let eta22 = etaq(2, 2, q, trunc);

    // All odd-exponent coefficients must be zero
    for k in (1..trunc).step_by(2) {
        assert_eq!(
            eta22.coeff(k), QRat::zero(),
            "etaq(2,2) at odd exponent q^{} should be 0", k
        );
    }

    // Verify: (q^2;q^2)_inf at even exponents = (q;q)_inf at half exponents
    // i.e., coeff of q^{2k} in (q^2;q^2)_inf = coeff of q^k in (q;q)_inf
    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler = euler_gen.into_series();

    for k in 0..trunc / 2 {
        assert_eq!(
            eta22.coeff(2 * k), euler.coeff(k),
            "etaq(2,2) at q^{} vs euler at q^{}: mismatch", 2 * k, k
        );
    }
}

/// etaq(1, 3, q, 30) = (q; q^3)_inf = prod_{n>=0}(1 - q^{1+3n})
/// = (1-q)(1-q^4)(1-q^7)(1-q^10)...
/// First few coefficients: 1, -1, 0, 0, -1, 1, 0, 0, -1, 1, 1, 0, -1, ...
#[test]
fn etaq_1_3_first_terms() {
    let q = q_var();
    let trunc = 20;

    let f = etaq(1, 3, q, trunc);

    // (1-q)(1-q^4)(1-q^7)(1-q^10)(1-q^13)(1-q^16)(1-q^19)
    // Manually verify first few: (1-q) = 1-q
    // * (1-q^4) = 1 - q - q^4 + q^5
    // * (1-q^7) = 1 - q - q^4 + q^5 - q^7 + q^8 + q^11 - q^12
    // * ...
    // Just verify constant term and coefficient of q^1
    assert_eq!(f.coeff(0), qrat(1), "constant term = 1");
    assert_eq!(f.coeff(1), qrat(-1), "coeff(1) = -1");
    // After (1-q)(1-q^4): coeff(2) = 0, coeff(3) = 0
    assert_eq!(f.coeff(2), QRat::zero(), "coeff(2) = 0");
    assert_eq!(f.coeff(3), QRat::zero(), "coeff(3) = 0");
    assert_eq!(f.coeff(4), qrat(-1), "coeff(4) = -1");
    assert_eq!(f.coeff(5), qrat(1), "coeff(5) = 1");
}

/// etaq with b=0 should return zero series.
#[test]
fn etaq_b_zero_is_zero() {
    let q = q_var();
    let f = etaq(0, 1, q, 20);
    assert!(f.is_zero(), "etaq(0, 1) should be zero since first factor is (1-1)=0");
}

// ===========================================================================
// 2. jacprod tests
// ===========================================================================

/// jacprod(1, 2, q, T) = (q;q^2)_inf * (q;q^2)_inf * (q^2;q^2)_inf
/// This equals the Jacobi theta function theta4(q):
///   theta4(q) = 1 - 2q + 2q^4 - 2q^9 + 2q^16 - 2q^25 + ...
///             = sum_{n=-inf}^{inf} (-1)^n q^{n^2}
///
/// Nonzero coefficients only at perfect squares, with value (-1)^n * 2 for n != 0,
/// and value 1 for n=0.
#[test]
fn jacprod_1_2_is_theta4() {
    let q = q_var();
    let trunc = 50;

    let jp = jacprod(1, 2, q, trunc);

    // Build expected theta4 coefficients
    let mut expected = vec![QRat::zero(); trunc as usize];
    // For n = 0, 1, 2, 3, ... while n^2 < trunc:
    // coeff at n^2 += (-1)^n
    // For n = -1, -2, -3, ... (same squares)
    // coeff at n^2 += (-1)^{|n|}
    // Net: coeff at 0 = 1, coeff at k^2 (k>0) = 2*(-1)^k
    expected[0] = qrat(1);
    let mut n: i64 = 1;
    while n * n < trunc {
        let sign = if n % 2 == 0 { 2 } else { -2 };
        expected[(n * n) as usize] = qrat(sign);
        n += 1;
    }

    for k in 0..trunc {
        assert_eq!(
            jp.coeff(k), expected[k as usize],
            "jacprod(1,2) [theta4] mismatch at q^{}: got {}, expected {}", k, jp.coeff(k), expected[k as usize]
        );
    }
}

/// jacprod(1, 5, q, 30) should be consistent with etaq definition:
/// JAC(1,5) = etaq(1,5) * etaq(4,5) * etaq(5,5)
/// This is a basic consistency check.
#[test]
fn jacprod_1_5_consistency() {
    let q = q_var();
    let trunc = 30;

    let jp = jacprod(1, 5, q, trunc);

    // Build manually from etaq
    let p1 = etaq(1, 5, q, trunc);
    let p2 = etaq(4, 5, q, trunc);
    let p3 = etaq(5, 5, q, trunc);
    let temp = arithmetic::mul(&p1, &p2);
    let manual = arithmetic::mul(&temp, &p3);

    for k in 0..trunc {
        assert_eq!(
            jp.coeff(k), manual.coeff(k),
            "jacprod(1,5) vs manual: mismatch at q^{}", k
        );
    }
}

// ===========================================================================
// 3. tripleprod tests
// ===========================================================================

/// tripleprod(z=-q, q, T): Jacobi triple product at z = -q.
///
/// tripleprod(z, q, T) = prod_{n>=1}(1-q^n) * prod_{n>=0}(1-z*q^n) * prod_{n>=1}(1-q^n/z)
///
/// For z = -q (c=-1, m=1):
/// Factor 2: prod_{n>=0}(1-(-1)*q^{1+n}) = prod_{n>=0}(1+q^{n+1}) = prod_{k>=1}(1+q^k)
/// Factor 3: prod_{n>=1}(1-q^n/(-q)) = prod_{n>=1}(1-(-1)*q^{n-1}) = prod_{n>=1}(1+q^{n-1})
///         = (1+1) * prod_{n>=2}(1+q^{n-1}) = 2 * prod_{k>=1}(1+q^k)
///
/// So tripleprod(-q, q, T) = (q;q)_inf * prod_{k>=1}(1+q^k) * 2 * prod_{k>=1}(1+q^k)
///                         = 2 * (q;q)_inf * [prod_{k>=1}(1+q^k)]^2
///
/// Now (q;q)_inf * prod_{k>=1}(1+q^k) = (q;q)_inf * (-q;q)_inf / (1+q^0 part)
/// Actually (-q;q)_inf starts at k=0: prod_{k>=0}(1+q^k) = 2 * prod_{k>=1}(1+q^k)
/// So prod_{k>=1}(1+q^k) = (-q;q)_inf / 2 where (-q;q)_inf = prod_{k>=0}(1-(-1)*q^k)
/// Wait, that's prod_{k>=0}(1+q^k) which has first factor (1+1)=2. Yes.
///
/// Alternative: use the Jacobi triple product identity directly:
/// tripleprod(z) = sum_{n=-inf}^{inf} (-1)^n z^n q^{n(n-1)/2}
/// For z = -q (c=-1, m=1): z^n = (-1)^n * q^n
/// term(n) = (-1)^n * (-1)^n * q^n * q^{n(n-1)/2} = q^{n + n(n-1)/2} = q^{n(n+1)/2}
///
/// So tripleprod(-q, q, T) = sum_{n=-inf}^{inf} q^{n(n+1)/2}
///
/// Triangular numbers n(n+1)/2 for n = ..., -3, -2, -1, 0, 1, 2, 3, ...:
///   n=-3: 3, n=-2: 1, n=-1: 0, n=0: 0, n=1: 1, n=2: 3, n=3: 6, n=4: 10, ...
///
/// So the series = sum of q^{triangular numbers}, with multiplicity from both
/// positive and negative n reaching the same value.
/// t(0)=0 (from n=-1 and n=0), t(1)=1 (from n=-2 and n=1), t(3)=3 (from n=-3 and n=2),
/// t(6)=6 (from n=3 and n=-4), etc.
///
/// Coefficients: at exponent 0: 2 (n=-1 and n=0), at exponent 1: 2 (n=-2 and n=1),
/// at exponent 3: 2, at exponent 6: 2, at exponent 10: 2, at exponent 15: 2, ...
/// At non-triangular numbers: 0.
#[test]
fn tripleprod_minus_q_bilateral_series() {
    let q = q_var();
    let trunc = 30;

    // z = -q means coeff = -1, power = 1
    let z = QMonomial::new(-QRat::one(), 1);
    let tp = tripleprod(&z, q, trunc);

    // Build the expected bilateral sum: sum_{n} q^{n(n+1)/2}
    // for all n such that n(n+1)/2 >= 0 and n(n+1)/2 < trunc
    let mut expected = vec![QRat::zero(); trunc as usize];
    for n in -100..=100i64 {
        let t = n * (n + 1) / 2;
        if t >= 0 && t < trunc {
            expected[t as usize] = expected[t as usize].clone() + QRat::one();
        }
    }

    for k in 0..trunc {
        assert_eq!(
            tp.coeff(k), expected[k as usize],
            "tripleprod(-q): mismatch at q^{}: got {}, expected {}", k, tp.coeff(k), expected[k as usize]
        );
    }
}

/// tripleprod with z=q (c=1, m=1) should give zero because factor 3 has
/// first term (1 - q^0) = 0.
#[test]
fn tripleprod_z_equals_q_is_zero() {
    let q = q_var();
    let z = QMonomial::q_power(1); // z = q (c=1, m=1)
    let tp = tripleprod(&z, q, 20);
    assert!(tp.is_zero(), "tripleprod(q) should be zero: factor3 starts with (1-q^0)=0");
}

/// tripleprod with z = -1 (c=-1, m=0).
///
/// By Jacobi triple product identity:
/// tripleprod(z) = sum_{n=-inf}^{inf} (-1)^n z^n q^{n(n-1)/2}
/// For z = -1: term(n) = (-1)^n * (-1)^n * q^{n(n-1)/2} = q^{n(n-1)/2}
///
/// n(n-1)/2 for n = ..., -2, -1, 0, 1, 2, 3, ...:
///   n=-2: 3, n=-1: 1, n=0: 0, n=1: 0, n=2: 1, n=3: 3, n=4: 6, ...
///
/// Same triangular numbers! coeff at 0: 2, at 1: 2, at 3: 2, at 6: 2, ...
///
/// Wait but z=-1 with c=-1 m=0: factor 2 starts with (1-(-1)*q^0) = (1+1) = 2.
/// So the product won't be zero. Good.
#[test]
fn tripleprod_minus_one() {
    let q = q_var();
    let trunc = 30;

    let z = QMonomial::new(-QRat::one(), 0); // z = -1
    let tp = tripleprod(&z, q, trunc);

    // Expected: sum_{n} q^{n(n-1)/2} for all integer n
    let mut expected = vec![QRat::zero(); trunc as usize];
    for n in -100..=100i64 {
        let t = n * (n - 1) / 2;
        if t >= 0 && t < trunc {
            expected[t as usize] = expected[t as usize].clone() + QRat::one();
        }
    }

    for k in 0..trunc {
        assert_eq!(
            tp.coeff(k), expected[k as usize],
            "tripleprod(-1): mismatch at q^{}: got {}, expected {}", k, tp.coeff(k), expected[k as usize]
        );
    }
}

// ===========================================================================
// 4. quinprod tests
// ===========================================================================

/// quinprod(z=q, q, T): quintuple product at z=q (c=1, m=1).
///
/// The quintuple product identity states:
/// quinprod(z, q) = sum_{n=-inf}^{inf} (z^{3n} - z^{3n+1}) * q^{n(3n-1)/2}  (Watson form)
///
/// But there are several equivalent forms. For z=q:
/// quinprod(q, q, T) = prod_{n>=1}(1-q^n)(1-q*q^n)(1-q^{-1}*q^{n-1})(1-q^2*q^{2n-1})(1-q^{-2}*q^{2n-1})
/// = prod_{n>=1}(1-q^n)(1-q^{n+1})(1-q^{n-2})(1-q^{2n+1})(1-q^{2n-3})
///
/// Factor 3 at n=1: (1-q^{-1}). Exponent -1 < 0, so this factor contributes 1 (since we ignore
/// negative exponents in FPS). Wait, actually let me re-check.
///
/// Factor 3: prod_{n>=1}(1 - (1/c)*q^{n-1-m}) for c=1, m=1:
/// = prod_{n>=1}(1 - q^{n-2})
/// n=1: (1-q^{-1}) -- negative exponent, ignored by FPS (factor = 1)
/// n=2: (1-q^0) = 0! The entire product vanishes!
///
/// Actually wait: in our qpochhammer_inf_generator, factor k=0 has offset = -m = -1,
/// so exponent = -1+0 = -1 (negative, treated as coefficient 0 in FPS = identity factor).
/// Factor k=1: exponent = -1+1 = 0, so we get (1-1*q^0) = (1-1) = 0.
///
/// Hmm, so quinprod(q, q, T) = 0 because one of the sub-products vanishes.
/// Let's pick a different z for testing.
///
/// For z = q^2 (c=1, m=2):
/// Factor 3: prod_{n>=1}(1 - q^{n-3})
/// n=1: exp=-2 (skip), n=2: exp=-1 (skip), n=3: exp=0 => (1-1)=0. Still zero!
///
/// Try z = QMonomial::new(QRat::from((1, 2i64)), 0) i.e., z = 1/2:
/// Factor 3: prod_{n>=1}(1 - 2*q^{n-1}). At n=1: (1-2) = -1, not zero. Good.
/// Factor 2: prod_{n>=1}(1 - (1/2)*q^{n}). offset=1, step=1, all factors nonzero for q.
///
/// Let's verify quinprod(1/2, q, T) against direct computation.
///
/// Actually, let's use a simpler test: verify the quintuple product identity
/// algebraically. The quintuple product identity says:
///   quinprod(z, q) = sum_{n=-inf}^{inf} q^{n(3n+1)/2} * (z^{3n} - z^{3n+1})
/// (Foata-Han form, one of many equivalent forms.)
///
/// For a concrete z with rational coefficients, we can build the sum side and compare.
///
/// Let's use z = QMonomial::new(-QRat::one(), 0) (z = -1):
/// Factor 1: (q;q)_inf
/// Factor 2: prod_{n>=1}(1+q^n) -- (offset=1, coeff=-1 so (1-(-1)*q^n)=(1+q^n))
/// Factor 3: prod_{n>=1}(1+q^{n-1}) -- n=1: (1+1)=2, n=2: (1+q), etc.
///           = 2 * prod_{n>=2}(1+q^{n-1}) = 2 * prod_{k>=1}(1+q^k)
/// Factor 4: prod_{n>=1}(1 - q^{2n-1}) -- (c^2=1, base = 2*0+1=1, step=2)
///           = (q;q^2)_inf starting at 1
/// Factor 5: prod_{n>=1}(1 - q^{2n-1}) -- same as factor 4! (inv_c^2 = 1, base = 1-0 = 1)
///
/// So quinprod(-1, q, T) = (q;q)_inf * prod_{k>=1}(1+q^k) * 2*prod_{k>=1}(1+q^k) * [(q;q^2)_inf]^2
///
/// This is getting complex. Let me just verify against the sum form for z=-1.
///
/// Sum form: sum_{n=-inf}^{inf} q^{n(3n+1)/2} * ((-1)^{3n} - (-1)^{3n+1})
/// = sum_{n} q^{n(3n+1)/2} * ((-1)^n - (-1)^{n+1})
/// = sum_{n} q^{n(3n+1)/2} * (-1)^n * (1 - (-1))
/// = sum_{n} q^{n(3n+1)/2} * (-1)^n * 2
/// = 2 * sum_{n} (-1)^n * q^{n(3n+1)/2}
#[test]
fn quinprod_z_minus_one() {
    let q = q_var();
    let trunc = 30;

    let z = QMonomial::new(-QRat::one(), 0); // z = -1
    let qp = quinprod(&z, q, trunc);

    // Build the bilateral sum: 2 * sum_{n} (-1)^n * q^{n(3n+1)/2}
    let mut expected = vec![QRat::zero(); trunc as usize];
    for n in -100..=100i64 {
        let exp = n * (3 * n + 1) / 2;
        if exp >= 0 && exp < trunc {
            let sign = if n % 2 == 0 { 1 } else { -1 };
            let contribution = qrat(2 * sign);
            expected[exp as usize] = expected[exp as usize].clone() + contribution;
        }
    }

    for k in 0..trunc {
        assert_eq!(
            qp.coeff(k), expected[k as usize],
            "quinprod(-1): mismatch at q^{}: got {}, expected {}", k, qp.coeff(k), expected[k as usize]
        );
    }
}

// ===========================================================================
// 5. winquist tests
// ===========================================================================

/// winquist basic test with a = q^2, b = q^3.
///
/// Winquist's identity is:
/// winquist(a, b, q, T) = (q;q)_inf^2 * prod_{n>=0}[(1-a*q^n)(1-a^{-1}*q^{n+1})(1-b*q^n)(1-b^{-1}*q^{n+1})
///                         *(1-ab*q^n)(1-(ab)^{-1}*q^{n+2})(1-a*b^{-1}*q^n)(1-a^{-1}*b*q^{n+1})]
///
/// For a = q^2 (c=1, m=2), b = q^3 (c=1, m=3):
/// Factor (ac=1, ap=2): prod_{n>=0}(1-q^{2+n}) = (q^2;q)_inf
/// Factor (inv_ac=1, 1-ap=-1): prod_{n>=0}(1-q^{-1+n}). First factor n=0: (1-q^{-1}), exponent -1 (skipped).
///   n=1: (1-q^0) = 0! Product vanishes.
///
/// This means winquist(q^2, q^3, q, T) = 0. Not a great test.
///
/// Let's try a = QMonomial::new(-QRat::one(), 1) (a = -q), b = QMonomial::new(-QRat::one(), 2) (b = -q^2):
/// Factor 1: (ac=-1, ap=1): prod(1-(-1)*q^{1+n}) = prod(1+q^{n+1}) for n>=0
/// Factor 2: (inv_ac=-1, 1-ap=0): prod(1-(-1)*q^{0+n}) = prod(1+q^n). First factor: (1+1)=2. Not zero.
/// Factor 3: (bc=-1, bp=2): prod(1+q^{2+n})
/// Factor 4: (inv_bc=-1, 1-bp=-1): prod(1+q^{n-1}). First factor: (1+q^{-1}) -- exponent -1 ignored, = 1+0 = 1.
/// Factor 5: (ac*bc=1, ap+bp=3): prod(1-q^{3+n}) = (q^3;q)_inf
/// Factor 6: (inv_ac*inv_bc=1, 2-ap-bp=-1): prod(1-q^{n-1}). First factor (1-q^{-1}) = 1 (neg exp skipped). n=1: (1-q^0)=0! Vanishes!
///
/// Hmm, winquist with integer-only QMonomials is tricky because the inverses often hit zero.
/// The function is designed for generic rational z-values.
///
/// Let's use a = QMonomial::new(QRat::from((1,3i64)), 0) (a = 1/3), b = QMonomial::new(QRat::from((1,5i64)), 0) (b = 1/5):
/// None of the factors should vanish since inverses (3, 5, 15, 5/3, 3/5) are all != 1,
/// and offsets are: 0, 1, 0, 1, 0, 2, 0, 1.
/// Wait: ac=1/3, offset=0: factor is prod(1-(1/3)*q^n). First factor (1-1/3)=2/3, not zero. Good.
/// inv_ac=3, offset=1: factor prod(1-3*q^{1+n}). All factors nonzero for small q. Good.
/// bc=1/5, offset=0: (1-1/5)=4/5. Good.
/// inv_bc=5, offset=1: (1-5*q). Good.
/// ac*bc=1/15, offset=0: (1-1/15)=14/15. Good.
/// inv_ac*inv_bc=15, offset=2: (1-15*q^2). Good.
/// ac*inv_bc=5/3, offset=0: (1-5/3) = -2/3. Good.
/// inv_ac*bc=3/5, offset=1: (1-3/5*q). Good.
///
/// All factors have nonzero first terms. Good. Now we can test that the result is nonzero
/// and that constant term is correct.
///
/// Constant term: coeff(0) of each infinite product factor is:
/// euler^2: coeff(0) = 1
/// Factor 1: (1-1/3) = 2/3, then (1-1/3*q)(1-1/3*q^2)... constant terms are all from n=0 factor
/// Actually, for a q-Pochhammer (c*q^offset; q)_inf, the constant term is:
///   If offset > 0: first factor (n=0) has exponent offset > 0, so factor's constant term = 1
///   If offset = 0: first factor is (1 - c), constant term = (1-c)
///   If offset < 0: first factor has neg exponent, constant term = 1 (since neg exp gets ignored)
///
/// For our factors:
/// (1/3, 0): constant = 1-1/3 = 2/3
/// (3, 1): constant = 1 (offset > 0)
/// (1/5, 0): constant = 1-1/5 = 4/5
/// (5, 1): constant = 1
/// (1/15, 0): constant = 1-1/15 = 14/15
/// (15, 2): constant = 1
/// (5/3, 0): constant = 1-5/3 = -2/3
/// (3/5, 1): constant = 1
///
/// Overall constant term = 1 * (2/3) * 1 * (4/5) * 1 * (14/15) * 1 * (-2/3) * 1
/// = (2/3) * (4/5) * (14/15) * (-2/3)
/// = (2*4*14*(-2)) / (3*5*15*3) = -224/675
#[test]
fn winquist_rational_constant_term() {
    let q = q_var();
    let trunc = 20;

    let a = QMonomial::new(QRat::from((1i64, 3i64)), 0);
    let b = QMonomial::new(QRat::from((1i64, 5i64)), 0);

    let w = winquist(&a, &b, q, trunc);

    // Verify constant term
    let expected_const = QRat::from((-224i64, 675i64));
    assert_eq!(
        w.coeff(0), expected_const,
        "winquist(1/3, 1/5) constant term: got {}, expected {}", w.coeff(0), expected_const
    );

    // The series should be nonzero (at least constant term is nonzero)
    assert!(!w.is_zero(), "winquist result should be nonzero");
}

/// Winquist is symmetric under a <-> a^{-1} in a certain sense.
/// We verify that the function produces consistent results when
/// both a and b are set to the same value (tests the ab, a/b factors).
#[test]
fn winquist_a_equals_b() {
    let q = q_var();
    let trunc = 20;

    let a = QMonomial::new(QRat::from((1i64, 2i64)), 0);
    let b = QMonomial::new(QRat::from((1i64, 2i64)), 0);

    let w = winquist(&a, &b, q, trunc);

    // When a = b = 1/2:
    // Factor (ac=1/2, 0): constant = 1/2
    // Factor (inv_ac=2, 1): constant = 1
    // Factor (bc=1/2, 0): constant = 1/2
    // Factor (inv_bc=2, 1): constant = 1
    // Factor (ac*bc=1/4, 0): constant = 3/4
    // Factor (inv_ac*inv_bc=4, 2): constant = 1
    // Factor (ac*inv_bc=1, 0): ZERO! coeff=1, offset=0 => vanishes!
    //
    // Wait: ac * inv_bc = (1/2) * 2 = 1, and offset = ap - bp = 0 - 0 = 0.
    // So this factor is (1*q^0; q)_inf = prod_{n>=0}(1-q^n) which has first factor (1-1)=0.
    // The entire winquist product vanishes!
    assert!(w.is_zero(), "winquist(1/2, 1/2) should be zero: a*b^{{-1}} factor vanishes");
}

// ===========================================================================
// 6. etaq composes to build (q;q)_inf^2
// ===========================================================================

/// etaq(1,1) squared matches (q;q)_inf^2 = euler^2.
/// Verify first 50 coefficients.
#[test]
fn etaq_squared_is_euler_squared() {
    let q = q_var();
    let trunc = 50;

    let eta = etaq(1, 1, q, trunc);
    let eta_sq = arithmetic::mul(&eta, &eta);

    let mut euler_gen = euler_function_generator(q, trunc);
    euler_gen.ensure_order(trunc);
    let euler = euler_gen.into_series();
    let euler_sq = arithmetic::mul(&euler, &euler);

    for k in 0..trunc {
        assert_eq!(
            eta_sq.coeff(k), euler_sq.coeff(k),
            "etaq(1,1)^2 vs euler^2: mismatch at q^{}", k
        );
    }
}
