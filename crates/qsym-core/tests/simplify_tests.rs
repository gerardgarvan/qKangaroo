//! Comprehensive tests for the phased simplification engine.
//!
//! Covers all 4 rule phases: normalization, cancellation, collection,
//! arithmetic simplification. Also verifies termination on adversarial
//! inputs and idempotency of the simplify function.

use qsym_core::canonical::*;
use qsym_core::number::{QInt, QRat};
use qsym_core::simplify::{simplify, SimplificationEngine};
use qsym_core::{Expr, ExprArena, ExprRef};

// ===========================================================================
// Normalization tests (Phase 1)
// ===========================================================================

#[test]
fn flatten_nested_add() {
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("a");
    let b = arena.intern_symbol("b");
    let c = arena.intern_symbol("c");

    // Add([a, Add([b, c])])
    let inner = make_add(&mut arena, vec![b, c]);
    let outer = make_add(&mut arena, vec![a, inner]);

    let result = simplify(outer, &mut arena);
    let expected = make_add(&mut arena, vec![a, b, c]);
    assert_eq!(result, expected, "Nested Add should be flattened");
}

#[test]
fn flatten_nested_mul() {
    let mut arena = ExprArena::new();
    let a = arena.intern_symbol("a");
    let b = arena.intern_symbol("b");
    let c = arena.intern_symbol("c");

    let inner = make_mul(&mut arena, vec![b, c]);
    let outer = make_mul(&mut arena, vec![a, inner]);

    let result = simplify(outer, &mut arena);
    let expected = make_mul(&mut arena, vec![a, b, c]);
    assert_eq!(result, expected, "Nested Mul should be flattened");
}

#[test]
fn combine_integer_add() {
    let mut arena = ExprArena::new();
    let three = arena.intern(Expr::Integer(QInt::from(3i64)));
    let five = arena.intern(Expr::Integer(QInt::from(5i64)));
    let x = arena.intern_symbol("x");

    let sum = make_add(&mut arena, vec![three, x, five]);
    let result = simplify(sum, &mut arena);

    // Should be Add([8, x]) or just x + 8 depending on sort
    let eight = arena.intern(Expr::Integer(QInt::from(8i64)));
    let expected = make_add(&mut arena, vec![eight, x]);
    assert_eq!(result, expected, "Integer constants in Add should be combined");
}

#[test]
fn combine_integer_mul() {
    let mut arena = ExprArena::new();
    let three = arena.intern(Expr::Integer(QInt::from(3i64)));
    let five = arena.intern(Expr::Integer(QInt::from(5i64)));
    let x = arena.intern_symbol("x");

    let prod = make_mul(&mut arena, vec![three, x, five]);
    let result = simplify(prod, &mut arena);

    let fifteen = arena.intern(Expr::Integer(QInt::from(15i64)));
    let expected = make_mul(&mut arena, vec![fifteen, x]);
    assert_eq!(
        result, expected,
        "Integer constants in Mul should be combined"
    );
}

#[test]
fn combine_mixed_numeric_add() {
    let mut arena = ExprArena::new();
    let one_int = arena.intern(Expr::Integer(QInt::from(1i64)));
    let half = arena.intern(Expr::Rational(QRat::from((1i64, 2i64))));
    let x = arena.intern_symbol("x");

    let sum = make_add(&mut arena, vec![one_int, half, x]);
    let result = simplify(sum, &mut arena);

    // 1 + 1/2 = 3/2
    let three_halves = arena.intern(Expr::Rational(QRat::from((3i64, 2i64))));
    let expected = make_add(&mut arena, vec![three_halves, x]);
    assert_eq!(
        result, expected,
        "Mixed Integer + Rational should combine to Rational"
    );
}

// ===========================================================================
// Cancellation tests (Phase 2)
// ===========================================================================

#[test]
fn add_zero_left() {
    let mut arena = ExprArena::new();
    let zero = arena.intern(Expr::Integer(QInt::zero()));
    let x = arena.intern_symbol("x");
    let sum = make_add(&mut arena, vec![zero, x]);
    let result = simplify(sum, &mut arena);
    assert_eq!(result, x, "Add([0, x]) should simplify to x");
}

#[test]
fn add_zero_right() {
    let mut arena = ExprArena::new();
    let zero = arena.intern(Expr::Integer(QInt::zero()));
    let x = arena.intern_symbol("x");
    let sum = make_add(&mut arena, vec![x, zero]);
    let result = simplify(sum, &mut arena);
    assert_eq!(result, x, "Add([x, 0]) should simplify to x");
}

#[test]
fn mul_one() {
    let mut arena = ExprArena::new();
    let one = arena.intern(Expr::Integer(QInt::from(1i64)));
    let x = arena.intern_symbol("x");
    let prod = make_mul(&mut arena, vec![one, x]);
    let result = simplify(prod, &mut arena);
    assert_eq!(result, x, "Mul([1, x]) should simplify to x");
}

#[test]
fn mul_zero() {
    let mut arena = ExprArena::new();
    let zero = arena.intern(Expr::Integer(QInt::zero()));
    let x = arena.intern_symbol("x");
    let prod = make_mul(&mut arena, vec![zero, x]);
    let result = simplify(prod, &mut arena);
    let expected_zero = arena.intern(Expr::Integer(QInt::zero()));
    assert_eq!(result, expected_zero, "Mul([0, x]) should simplify to 0");
}

#[test]
fn pow_zero_exp() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let zero = arena.intern(Expr::Integer(QInt::zero()));
    let pow = make_pow(&mut arena, x, zero);
    let result = simplify(pow, &mut arena);
    let one = arena.intern(Expr::Integer(QInt::from(1i64)));
    assert_eq!(result, one, "Pow(x, 0) should simplify to 1");
}

#[test]
fn pow_one_exp() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let one = arena.intern(Expr::Integer(QInt::from(1i64)));
    let pow = make_pow(&mut arena, x, one);
    let result = simplify(pow, &mut arena);
    assert_eq!(result, x, "Pow(x, 1) should simplify to x");
}

#[test]
fn pow_one_base() {
    let mut arena = ExprArena::new();
    let one = arena.intern(Expr::Integer(QInt::from(1i64)));
    let n = arena.intern_symbol("n");
    let pow = make_pow(&mut arena, one, n);
    let result = simplify(pow, &mut arena);
    let expected_one = arena.intern(Expr::Integer(QInt::from(1i64)));
    assert_eq!(result, expected_one, "Pow(1, n) should simplify to 1");
}

// ===========================================================================
// Collection tests (Phase 3)
// ===========================================================================

#[test]
fn collect_duplicate_add() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    // Bypass make_add dedup by constructing Expr::Add directly
    let dup_add = arena.intern(Expr::Add(vec![x, x]));
    let result = simplify(dup_add, &mut arena);

    // Should get Mul([2, x])
    let two = arena.intern(Expr::Integer(QInt::from(2i64)));
    let expected = make_mul(&mut arena, vec![two, x]);
    assert_eq!(result, expected, "x + x should simplify to 2*x");
}

#[test]
fn collect_like_terms_add() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let two = arena.intern(Expr::Integer(QInt::from(2i64)));
    let three = arena.intern(Expr::Integer(QInt::from(3i64)));

    let term_2x = make_mul(&mut arena, vec![two, x]);
    let term_3x = make_mul(&mut arena, vec![three, x]);
    let sum = make_add(&mut arena, vec![term_2x, term_3x]);
    let result = simplify(sum, &mut arena);

    let five = arena.intern(Expr::Integer(QInt::from(5i64)));
    let expected = make_mul(&mut arena, vec![five, x]);
    assert_eq!(result, expected, "2*x + 3*x should simplify to 5*x");
}

#[test]
fn collect_duplicate_mul() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    // Bypass make_mul dedup
    let dup_mul = arena.intern(Expr::Mul(vec![x, x]));
    let result = simplify(dup_mul, &mut arena);

    let two = arena.intern(Expr::Integer(QInt::from(2i64)));
    let expected = make_pow(&mut arena, x, two);
    assert_eq!(result, expected, "x * x should simplify to x^2");
}

#[test]
fn collect_triple_mul() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    // x * x * x (bypass dedup)
    let triple_mul = arena.intern(Expr::Mul(vec![x, x, x]));
    let result = simplify(triple_mul, &mut arena);

    let three = arena.intern(Expr::Integer(QInt::from(3i64)));
    let expected = make_pow(&mut arena, x, three);
    assert_eq!(result, expected, "x * x * x should simplify to x^3");
}

// ===========================================================================
// Arithmetic simplification tests (Phase 4)
// ===========================================================================

#[test]
fn double_negation() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let neg_x = make_neg(&mut arena, x);
    let neg_neg_x = make_neg(&mut arena, neg_x);
    let result = simplify(neg_neg_x, &mut arena);
    assert_eq!(result, x, "Neg(Neg(x)) should simplify to x");
}

#[test]
fn triple_negation() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let neg1 = make_neg(&mut arena, x);
    let neg2 = make_neg(&mut arena, neg1);
    let neg3 = make_neg(&mut arena, neg2);
    let result = simplify(neg3, &mut arena);
    let expected = make_neg(&mut arena, x);
    assert_eq!(result, expected, "Neg(Neg(Neg(x))) should simplify to Neg(x)");
}

#[test]
fn neg_integer() {
    let mut arena = ExprArena::new();
    let five = arena.intern(Expr::Integer(QInt::from(5i64)));
    let neg_five = make_neg(&mut arena, five);
    let result = simplify(neg_five, &mut arena);
    let expected = arena.intern(Expr::Integer(QInt::from(-5i64)));
    assert_eq!(result, expected, "Neg(5) should simplify to -5");
}

#[test]
fn neg_rational() {
    let mut arena = ExprArena::new();
    let three_fourths = arena.intern(Expr::Rational(QRat::from((3i64, 4i64))));
    let neg = make_neg(&mut arena, three_fourths);
    let result = simplify(neg, &mut arena);
    let expected = arena.intern(Expr::Rational(QRat::from((-3i64, 4i64))));
    assert_eq!(
        result, expected,
        "Neg(3/4) should simplify to -3/4"
    );
}

// ===========================================================================
// Compound / integration tests
// ===========================================================================

#[test]
fn nested_flatten_and_cancel() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let y = arena.intern_symbol("y");
    let zero = arena.intern(Expr::Integer(QInt::zero()));

    // Add([Add([0, x]), y]) -> should flatten to Add([0, x, y]) -> cancel 0 -> Add([x, y])
    let inner = make_add(&mut arena, vec![zero, x]);
    // inner should already be just x due to make_add dedup+simplify? No, make_add just sorts and dedups.
    // Actually make_add(vec![0, x]) = Add([0, x]) since 0 != x (different ExprRefs).
    // But wait -- cancel will remove the zero. Let me verify the inner is Add or just x.
    // make_add([zero, x]) with zero = #0, x = #1 -> sorted and deduped -> Add([#0, #1])
    // That's still 2 elements so it creates Add.
    let outer = make_add(&mut arena, vec![inner, y]);
    let result = simplify(outer, &mut arena);
    let expected = make_add(&mut arena, vec![x, y]);
    assert_eq!(
        result, expected,
        "Add([Add([0, x]), y]) should flatten and cancel to Add([x, y])"
    );
}

#[test]
fn deep_negation_chain_even() {
    // 4 levels of negation = even -> identity
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let mut current = x;
    for _ in 0..4 {
        current = make_neg(&mut arena, current);
    }
    let result = simplify(current, &mut arena);
    assert_eq!(result, x, "Neg^4(x) should simplify to x");
}

#[test]
fn complex_simplify() {
    // Mul([1, Add([3, 5, Neg(0)])]) -> Mul([1, Add([3, 5, 0])]) -> Mul([1, 8]) -> 8
    let mut arena = ExprArena::new();
    let zero = arena.intern(Expr::Integer(QInt::zero()));
    let one = arena.intern(Expr::Integer(QInt::from(1i64)));
    let three = arena.intern(Expr::Integer(QInt::from(3i64)));
    let five = arena.intern(Expr::Integer(QInt::from(5i64)));

    let neg_zero = make_neg(&mut arena, zero);
    let sum = make_add(&mut arena, vec![three, five, neg_zero]);
    let prod = make_mul(&mut arena, vec![one, sum]);
    let result = simplify(prod, &mut arena);

    let eight = arena.intern(Expr::Integer(QInt::from(8i64)));
    assert_eq!(result, eight, "Mul([1, Add([3, 5, Neg(0)])]) should simplify to 8");
}

#[test]
fn pow_pow_integer_exp() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let two = arena.intern(Expr::Integer(QInt::from(2i64)));
    let three = arena.intern(Expr::Integer(QInt::from(3i64)));

    let inner_pow = make_pow(&mut arena, x, two);
    let outer_pow = make_pow(&mut arena, inner_pow, three);
    let result = simplify(outer_pow, &mut arena);

    let six = arena.intern(Expr::Integer(QInt::from(6i64)));
    let expected = make_pow(&mut arena, x, six);
    assert_eq!(
        result, expected,
        "Pow(Pow(x, 2), 3) should simplify to Pow(x, 6)"
    );
}

#[test]
fn atoms_unchanged() {
    let mut arena = ExprArena::new();

    // Symbol
    let x = arena.intern_symbol("x");
    assert_eq!(simplify(x, &mut arena), x, "Symbol unchanged");

    // Integer
    let n = arena.intern(Expr::Integer(QInt::from(42i64)));
    assert_eq!(simplify(n, &mut arena), n, "Integer unchanged");

    // Rational
    let r = arena.intern(Expr::Rational(QRat::from((2i64, 3i64))));
    assert_eq!(simplify(r, &mut arena), r, "Rational unchanged");

    // Infinity
    let inf = arena.intern(Expr::Infinity);
    assert_eq!(simplify(inf, &mut arena), inf, "Infinity unchanged");

    // Undefined
    let undef = arena.intern(Expr::Undefined);
    assert_eq!(simplify(undef, &mut arena), undef, "Undefined unchanged");
}

// ===========================================================================
// Termination tests
// ===========================================================================

#[test]
fn termination_identity() {
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let result = simplify(x, &mut arena);
    assert_eq!(result, x, "Simplification of an atom is a fixpoint");
}

#[test]
fn termination_deep_nesting() {
    // Build Neg(Neg(Neg(...Neg(x)...))) 50 levels deep
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");

    let depth = 50;
    let mut current = x;
    for _ in 0..depth {
        current = make_neg(&mut arena, current);
    }

    let engine = SimplificationEngine::with_max_iterations(200);
    let result = engine.simplify(current, &mut arena);

    // 50 is even, so result should be x
    assert_eq!(result, x, "50-deep Neg chain should simplify to x (even depth)");
}

#[test]
fn termination_deep_nesting_odd() {
    // 51 levels of negation (odd) -> Neg(x)
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");

    let depth = 51;
    let mut current = x;
    for _ in 0..depth {
        current = make_neg(&mut arena, current);
    }

    let engine = SimplificationEngine::with_max_iterations(200);
    let result = engine.simplify(current, &mut arena);

    let neg_x = make_neg(&mut arena, x);
    // After simplify_arith, Neg(x) where x is a Symbol stays as Neg(x).
    // But if x is actually a Symbol, Neg(Symbol) doesn't match any further rule.
    assert_eq!(
        result, neg_x,
        "51-deep Neg chain should simplify to Neg(x) (odd depth)"
    );
}

#[test]
fn termination_wide_expression() {
    // Build Add of 100 distinct symbols
    let mut arena = ExprArena::new();
    let symbols: Vec<ExprRef> = (0..100)
        .map(|i| arena.intern_symbol(&format!("x{}", i)))
        .collect();

    let wide_add = make_add(&mut arena, symbols.clone());
    let result = simplify(wide_add, &mut arena);

    // Should return the same expression (no simplification needed)
    // Just verify it terminates
    assert_eq!(
        result, wide_add,
        "Wide Add of distinct symbols should be unchanged"
    );
}

// ===========================================================================
// Idempotency tests
// ===========================================================================

#[test]
fn simplify_is_idempotent() {
    let mut arena = ExprArena::new();

    // Test several expressions
    let x = arena.intern_symbol("x");
    let y = arena.intern_symbol("y");
    let zero = arena.intern(Expr::Integer(QInt::zero()));
    let one = arena.intern(Expr::Integer(QInt::from(1i64)));
    let three = arena.intern(Expr::Integer(QInt::from(3i64)));
    let five = arena.intern(Expr::Integer(QInt::from(5i64)));

    let expressions = vec![
        // Atom
        x,
        // Add with zero
        make_add(&mut arena, vec![zero, x]),
        // Mul with one
        make_mul(&mut arena, vec![one, x]),
        // Double negation
        {
            let neg_x = make_neg(&mut arena, x);
            make_neg(&mut arena, neg_x)
        },
        // Add with numeric constants
        make_add(&mut arena, vec![three, five, x]),
        // Compound: Mul([1, Add([3, 5])])
        {
            let sum = make_add(&mut arena, vec![three, five]);
            make_mul(&mut arena, vec![one, sum])
        },
        // Nested Add
        {
            let inner = make_add(&mut arena, vec![x, y]);
            make_add(&mut arena, vec![inner, three])
        },
    ];

    for expr in expressions {
        let once = simplify(expr, &mut arena);
        let twice = simplify(once, &mut arena);
        assert_eq!(
            once, twice,
            "simplify should be idempotent: simplify(simplify(e)) == simplify(e), \
             expr = {:?}, once = {:?}, twice = {:?}",
            arena.get(expr),
            arena.get(once),
            arena.get(twice)
        );
    }
}

// ===========================================================================
// Additional edge case tests
// ===========================================================================

#[test]
fn neg_zero_cancels() {
    let mut arena = ExprArena::new();
    let zero = arena.intern(Expr::Integer(QInt::zero()));
    let neg_zero = make_neg(&mut arena, zero);
    let result = simplify(neg_zero, &mut arena);
    let expected_zero = arena.intern(Expr::Integer(QInt::zero()));
    assert_eq!(result, expected_zero, "Neg(0) should simplify to 0");
}

#[test]
fn add_all_zeros() {
    let mut arena = ExprArena::new();
    let zero = arena.intern(Expr::Integer(QInt::zero()));
    // Add([0, 0]) -- bypass dedup
    let add = arena.intern(Expr::Add(vec![zero, zero]));
    let result = simplify(add, &mut arena);
    let expected_zero = arena.intern(Expr::Integer(QInt::zero()));
    assert_eq!(result, expected_zero, "Add([0, 0]) should simplify to 0");
}

#[test]
fn mul_all_ones() {
    let mut arena = ExprArena::new();
    let one = arena.intern(Expr::Integer(QInt::from(1i64)));
    // Mul([1, 1]) -- bypass dedup
    let mul = arena.intern(Expr::Mul(vec![one, one]));
    let result = simplify(mul, &mut arena);
    let expected_one = arena.intern(Expr::Integer(QInt::from(1i64)));
    assert_eq!(result, expected_one, "Mul([1, 1]) should simplify to 1");
}

#[test]
fn add_single_numeric_not_combined() {
    // When there's only one numeric in an Add, it should NOT be removed or altered
    let mut arena = ExprArena::new();
    let three = arena.intern(Expr::Integer(QInt::from(3i64)));
    let x = arena.intern_symbol("x");

    let sum = make_add(&mut arena, vec![three, x]);
    let result = simplify(sum, &mut arena);
    // Should remain as Add([3, x]) -- one numeric doesn't need combining
    assert_eq!(result, sum, "Add([3, x]) with single numeric should be unchanged");
}

#[test]
fn mul_single_numeric_not_combined() {
    let mut arena = ExprArena::new();
    let three = arena.intern(Expr::Integer(QInt::from(3i64)));
    let x = arena.intern_symbol("x");

    let prod = make_mul(&mut arena, vec![three, x]);
    let result = simplify(prod, &mut arena);
    assert_eq!(result, prod, "Mul([3, x]) with single numeric should be unchanged");
}

#[test]
fn collect_x_plus_x_via_direct_construction() {
    // Test that x + x (constructed as duplicate in Add) gets collected
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    // Directly intern to bypass make_add dedup
    let add = arena.intern(Expr::Add(vec![x, x]));
    let result = simplify(add, &mut arena);

    let two = arena.intern(Expr::Integer(QInt::from(2i64)));
    let expected = make_mul(&mut arena, vec![two, x]);
    assert_eq!(result, expected, "Direct Add([x, x]) should collect to 2*x");
}

#[test]
fn pow_of_pow_with_non_integer_exponents_unchanged() {
    // Pow(Pow(x, y), z) where y and z are symbols should NOT simplify
    let mut arena = ExprArena::new();
    let x = arena.intern_symbol("x");
    let y = arena.intern_symbol("y");
    let z = arena.intern_symbol("z");

    let inner = make_pow(&mut arena, x, y);
    let outer = make_pow(&mut arena, inner, z);
    let result = simplify(outer, &mut arena);
    assert_eq!(
        result, outer,
        "Pow(Pow(x, y), z) with symbolic exponents should be unchanged"
    );
}
