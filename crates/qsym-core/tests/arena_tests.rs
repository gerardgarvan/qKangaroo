//! Hash-consing invariant tests for ExprArena and canonical ordering.
//!
//! These tests verify the core correctness properties:
//! - Structurally identical expressions get the same ExprRef.
//! - Different expressions get different ExprRefs.
//! - Canonical ordering ensures commutativity of Add/Mul at the ExprRef level.
//! - All Expr variants round-trip through the arena.

use qsym_core::canonical::{
    make_add, make_basic_hypergeometric, make_dedekind_eta, make_jacobi_theta, make_mul,
    make_neg, make_pow, make_qpochhammer,
};
use qsym_core::{Expr, ExprArena, ExprRef};
use smallvec::smallvec;

// ============================================================
// Hash-consing deduplication tests
// ============================================================

#[test]
fn intern_same_integer_returns_same_ref() {
    let mut arena = ExprArena::new();
    let a = arena.intern_int(42);
    let b = arena.intern_int(42);
    assert_eq!(a, b);
}

#[test]
fn intern_same_symbol_returns_same_ref() {
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("q");
    let b = arena.intern_symbol("q");
    assert_eq!(a, b);
}

#[test]
fn intern_same_qpochhammer_returns_same_ref() {
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("a");
    let q = arena.intern_symbol("q");
    let n = arena.intern_int(5);

    let qp1 = make_qpochhammer(&mut arena, a, q, n);
    let qp2 = make_qpochhammer(&mut arena, a, q, n);
    assert_eq!(qp1, qp2);
}

#[test]
fn different_integers_get_different_refs() {
    let mut arena = ExprArena::new();
    let a = arena.intern_int(1);
    let b = arena.intern_int(2);
    assert_ne!(a, b);
}

#[test]
fn different_symbols_get_different_refs() {
    let mut arena = ExprArena::new();
    let q = arena.intern_symbol("q");
    let a = arena.intern_symbol("a");
    assert_ne!(q, a);
}

// ============================================================
// Canonical ordering (commutativity) tests
// ============================================================

#[test]
fn add_commutativity() {
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("a");
    let b = arena.intern_symbol("b");

    let sum1 = make_add(&mut arena, vec![a, b]);
    let sum2 = make_add(&mut arena, vec![b, a]);
    assert_eq!(sum1, sum2, "a + b must equal b + a");
}

#[test]
fn mul_commutativity() {
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("a");
    let b = arena.intern_symbol("b");

    let prod1 = make_mul(&mut arena, vec![a, b]);
    let prod2 = make_mul(&mut arena, vec![b, a]);
    assert_eq!(prod1, prod2, "a * b must equal b * a");
}

#[test]
fn add_commutativity_three_children() {
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("a");
    let b = arena.intern_symbol("b");
    let c = arena.intern_symbol("c");

    let sum1 = make_add(&mut arena, vec![a, b, c]);
    let sum2 = make_add(&mut arena, vec![c, a, b]);
    let sum3 = make_add(&mut arena, vec![b, c, a]);
    assert_eq!(sum1, sum2);
    assert_eq!(sum2, sum3);
}

#[test]
fn mul_commutativity_three_children() {
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("a");
    let b = arena.intern_symbol("b");
    let c = arena.intern_symbol("c");

    let prod1 = make_mul(&mut arena, vec![a, b, c]);
    let prod2 = make_mul(&mut arena, vec![c, a, b]);
    let prod3 = make_mul(&mut arena, vec![b, c, a]);
    assert_eq!(prod1, prod2);
    assert_eq!(prod2, prod3);
}

// ============================================================
// Identity element tests
// ============================================================

#[test]
fn add_zero_children_returns_zero() {
    let mut arena = ExprArena::new();
    let result = make_add(&mut arena, vec![]);
    let zero = arena.intern_int(0);
    assert_eq!(result, zero);
}

#[test]
fn add_one_child_returns_child() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let result = make_add(&mut arena, vec![x]);
    assert_eq!(result, x, "Add of single child should unwrap");
}

#[test]
fn mul_zero_children_returns_one() {
    let mut arena = ExprArena::new();
    let result = make_mul(&mut arena, vec![]);
    let one = arena.intern_int(1);
    assert_eq!(result, one);
}

#[test]
fn mul_one_child_returns_child() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let result = make_mul(&mut arena, vec![x]);
    assert_eq!(result, x, "Mul of single child should unwrap");
}

// ============================================================
// Arena growth (no spurious duplicates, no missed dedup)
// ============================================================

#[test]
fn arena_len_grows_correctly() {
    let mut arena = ExprArena::new();
    assert_eq!(arena.len(), 0);

    arena.intern_int(1);
    assert_eq!(arena.len(), 1);

    arena.intern_int(2);
    assert_eq!(arena.len(), 2);

    // Duplicate -- should NOT increase len
    arena.intern_int(1);
    assert_eq!(arena.len(), 2);

    arena.intern_symbol("q");
    assert_eq!(arena.len(), 3);

    // Duplicate symbol -- should NOT increase len
    arena.intern_symbol("q");
    assert_eq!(arena.len(), 3);
}

#[test]
fn arena_no_spurious_growth_from_dedup() {
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("a");
    let b = arena.intern_symbol("b");
    let len_before = arena.len();

    // Creating the same Add both ways should add only 1 entry
    let _sum1 = make_add(&mut arena, vec![a, b]);
    assert_eq!(arena.len(), len_before + 1);

    let _sum2 = make_add(&mut arena, vec![b, a]);
    assert_eq!(
        arena.len(),
        len_before + 1,
        "Reversed Add should be dedup'd"
    );
}

// ============================================================
// Complex nested expression tests
// ============================================================

#[test]
fn nested_qpochhammer_dedup() {
    // Construct (a;q)_inf + (a;q)_5
    // Both share the same a and q sub-expressions
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("a");
    let q = arena.intern_symbol("q");
    let five = arena.intern_int(5);
    let inf = arena.intern(Expr::Infinity);

    let qp_finite = make_qpochhammer(&mut arena, a, q, five);
    let qp_inf = make_qpochhammer(&mut arena, a, q, inf);

    // They should be different (different order)
    assert_ne!(qp_finite, qp_inf);

    // But interning the same finite one again should dedup
    let qp_finite_2 = make_qpochhammer(&mut arena, a, q, five);
    assert_eq!(qp_finite, qp_finite_2);

    // The sum should be a single Add node
    let sum = make_add(&mut arena, vec![qp_finite, qp_inf]);
    let sum2 = make_add(&mut arena, vec![qp_inf, qp_finite]);
    assert_eq!(sum, sum2, "QPochhammer sum should be commutative");
}

// ============================================================
// All Expr variant round-trip tests
// ============================================================

#[test]
fn roundtrip_integer() {
    let mut arena = ExprArena::new();
    let r = arena.intern_int(42);
    match arena.get(r) {
        Expr::Integer(n) => assert_eq!(n.0, 42),
        other => panic!("Expected Integer, got {:?}", other),
    }
}

#[test]
fn roundtrip_rational() {
    let mut arena = ExprArena::new();
    let r = arena.intern_rat(3, 4);
    match arena.get(r) {
        Expr::Rational(q) => {
            assert_eq!(*q.numer(), 3);
            assert_eq!(*q.denom(), 4);
        }
        other => panic!("Expected Rational, got {:?}", other),
    }
}

#[test]
fn roundtrip_symbol() {
    let mut arena = ExprArena::new();
    let r = arena.intern_symbol("tau");
    match arena.get(r) {
        Expr::Symbol(id) => assert_eq!(arena.symbols().name(*id), "tau"),
        other => panic!("Expected Symbol, got {:?}", other),
    }
}

#[test]
fn roundtrip_infinity() {
    let mut arena = ExprArena::new();
    let r = arena.intern(Expr::Infinity);
    assert_eq!(arena.get(r), &Expr::Infinity);
}

#[test]
fn roundtrip_undefined() {
    let mut arena = ExprArena::new();
    let r = arena.intern(Expr::Undefined);
    assert_eq!(arena.get(r), &Expr::Undefined);
}

#[test]
fn roundtrip_neg() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let neg_x = make_neg(&mut arena, x);
    match arena.get(neg_x) {
        Expr::Neg(inner) => assert_eq!(*inner, x),
        other => panic!("Expected Neg, got {:?}", other),
    }
}

#[test]
fn roundtrip_pow() {
    let mut arena = ExprArena::new();
    let base = arena.intern_symbol("q");
    let exp = arena.intern_int(2);
    let pow = make_pow(&mut arena, base, exp);
    match arena.get(pow) {
        Expr::Pow(b, e) => {
            assert_eq!(*b, base);
            assert_eq!(*e, exp);
        }
        other => panic!("Expected Pow, got {:?}", other),
    }
}

#[test]
fn roundtrip_jacobi_theta() {
    let mut arena = ExprArena::new();
    let q = arena.intern_symbol("q");
    let theta = make_jacobi_theta(&mut arena, 3, q);
    match arena.get(theta) {
        Expr::JacobiTheta { index, nome } => {
            assert_eq!(*index, 3);
            assert_eq!(*nome, q);
        }
        other => panic!("Expected JacobiTheta, got {:?}", other),
    }
}

#[test]
fn roundtrip_dedekind_eta() {
    let mut arena = ExprArena::new();
    let tau = arena.intern_symbol("tau");
    let eta = make_dedekind_eta(&mut arena, tau);
    match arena.get(eta) {
        Expr::DedekindEta(inner) => assert_eq!(*inner, tau),
        other => panic!("Expected DedekindEta, got {:?}", other),
    }
}

#[test]
fn roundtrip_basic_hypergeometric() {
    let mut arena = ExprArena::new();
    let a1 = arena.intern_symbol("a1");
    let a2 = arena.intern_symbol("a2");
    let b1 = arena.intern_symbol("b1");
    let q = arena.intern_symbol("q");
    let z = arena.intern_symbol("z");

    let hyper = make_basic_hypergeometric(
        &mut arena,
        smallvec![a1, a2],
        smallvec![b1],
        q,
        z,
    );
    match arena.get(hyper) {
        Expr::BasicHypergeometric {
            upper,
            lower,
            nome,
            argument,
        } => {
            assert_eq!(upper.len(), 2);
            assert_eq!(upper[0], a1);
            assert_eq!(upper[1], a2);
            assert_eq!(lower.len(), 1);
            assert_eq!(lower[0], b1);
            assert_eq!(*nome, q);
            assert_eq!(*argument, z);
        }
        other => panic!("Expected BasicHypergeometric, got {:?}", other),
    }
}

#[test]
fn roundtrip_add() {
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("a");
    let b = arena.intern_symbol("b");
    let sum = make_add(&mut arena, vec![a, b]);
    match arena.get(sum) {
        Expr::Add(children) => {
            assert_eq!(children.len(), 2);
            // Children are sorted, so order depends on ExprRef values
        }
        other => panic!("Expected Add, got {:?}", other),
    }
}

#[test]
fn roundtrip_mul() {
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("a");
    let b = arena.intern_symbol("b");
    let prod = make_mul(&mut arena, vec![a, b]);
    match arena.get(prod) {
        Expr::Mul(children) => {
            assert_eq!(children.len(), 2);
        }
        other => panic!("Expected Mul, got {:?}", other),
    }
}

// ============================================================
// Add/Mul deduplication of identical children
// ============================================================

#[test]
fn add_dedup_removes_duplicate_refs() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    // make_add with [x, x] should dedup to just x (single child -> unwrap)
    let result = make_add(&mut arena, vec![x, x]);
    assert_eq!(result, x, "Add with duplicate refs should dedup to single child");
}

#[test]
fn mul_dedup_removes_duplicate_refs() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let result = make_mul(&mut arena, vec![x, x]);
    assert_eq!(result, x, "Mul with duplicate refs should dedup to single child");
}

// ============================================================
// proptest: random permutation commutativity
// ============================================================

mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Strategy to generate a permutation of indices 0..n
    fn permutation(n: usize) -> impl Strategy<Value = Vec<usize>> {
        Just((0..n).collect::<Vec<_>>()).prop_shuffle()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn add_any_permutation_same_ref(perm in permutation(5)) {
            let mut arena = ExprArena::new();
            // Create 5 distinct symbols
            let syms: Vec<ExprRef> = ["a", "b", "c", "d", "e"]
                .iter()
                .map(|s| arena.intern_symbol(s))
                .collect();

            // Canonical (sorted) order
            let canonical = make_add(&mut arena, syms.clone());

            // Permuted order
            let permuted_children: Vec<ExprRef> = perm.iter().map(|&i| syms[i]).collect();
            let permuted = make_add(&mut arena, permuted_children);

            prop_assert_eq!(canonical, permuted,
                "Add with permutation {:?} should produce same ExprRef", perm);
        }

        #[test]
        fn mul_any_permutation_same_ref(perm in permutation(5)) {
            let mut arena = ExprArena::new();
            let syms: Vec<ExprRef> = ["a", "b", "c", "d", "e"]
                .iter()
                .map(|s| arena.intern_symbol(s))
                .collect();

            let canonical = make_mul(&mut arena, syms.clone());
            let permuted_children: Vec<ExprRef> = perm.iter().map(|&i| syms[i]).collect();
            let permuted = make_mul(&mut arena, permuted_children);

            prop_assert_eq!(canonical, permuted,
                "Mul with permutation {:?} should produce same ExprRef", perm);
        }

        #[test]
        fn intern_int_idempotent(val in -1000i64..1000) {
            let mut arena = ExprArena::new();
            let a = arena.intern_int(val);
            let b = arena.intern_int(val);
            prop_assert_eq!(a, b, "Interning same integer {} twice should give same ref", val);
        }
    }
}
