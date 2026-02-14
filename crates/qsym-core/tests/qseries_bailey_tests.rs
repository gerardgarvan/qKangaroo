//! Integration tests for Bailey pairs, Bailey lemma, chain iteration, and weak Bailey lemma.
//!
//! Tests verify:
//! - Unit pair alpha/beta evaluation
//! - Rogers-Ramanujan pair alpha/beta evaluation
//! - q-Binomial pair alpha evaluation
//! - Bailey pair relation verification for all pair types
//! - Bailey lemma produces valid pairs
//! - Bailey chain iteration produces valid chains
//! - Weak Bailey lemma identity holds
//! - BaileyDatabase search and storage

use qsym_core::number::QRat;
use qsym_core::series::{arithmetic, FormalPowerSeries};
use qsym_core::ExprArena;
use qsym_core::symbol::SymbolId;
use qsym_core::qseries::{
    QMonomial, PochhammerOrder, aqprod,
    BaileyPair, BaileyPairType, BaileyDatabase,
    bailey_lemma, bailey_chain, weak_bailey_lemma, verify_bailey_pair,
};

/// Helper: create a SymbolId for "q".
fn q_var() -> SymbolId {
    let mut arena = ExprArena::new();
    arena.symbols_mut().intern("q")
}

/// Helper: create QRat from i64.
fn qrat(n: i64) -> QRat {
    QRat::from((n, 1i64))
}

/// Helper: create QRat fraction from (num, den).
fn qrat_frac(n: i64, d: i64) -> QRat {
    QRat::from((n, d))
}

// ===========================================================================
// 1. Unit pair tests
// ===========================================================================

/// Unit pair alpha_0 = 1, alpha_n = 0 for n > 0.
#[test]
fn test_unit_pair_alpha() {
    let q = q_var();
    let trunc = 20;
    let a = QMonomial::q(); // a = q

    let pair = BaileyPair {
        name: "unit".into(),
        pair_type: BaileyPairType::Unit,
        tags: vec![],
    };

    let alpha_0 = pair.alpha_term(0, &a, q, trunc);
    assert_eq!(alpha_0, FormalPowerSeries::one(q, trunc));

    let alpha_1 = pair.alpha_term(1, &a, q, trunc);
    assert!(alpha_1.is_zero());

    let alpha_5 = pair.alpha_term(5, &a, q, trunc);
    assert!(alpha_5.is_zero());
}

/// Unit pair beta_n = 1/[(q;q)_n * (aq;q)_n].
/// For a = q: beta_0 = 1, beta_1 = 1/[(1-q)(1-q^2)].
#[test]
fn test_unit_pair_beta() {
    let q = q_var();
    let trunc = 20;
    let a = QMonomial::q(); // a = q

    let pair = BaileyPair {
        name: "unit".into(),
        pair_type: BaileyPairType::Unit,
        tags: vec![],
    };

    // beta_0 = 1/[(q;q)_0 * (q^2;q)_0] = 1/(1*1) = 1
    let beta_0 = pair.beta_term(0, &a, q, trunc);
    assert_eq!(beta_0.coeff(0), QRat::one());

    // beta_1 = 1/[(q;q)_1 * (q^2;q)_1] = 1/[(1-q)(1-q^2)]
    let beta_1 = pair.beta_term(1, &a, q, trunc);
    // (1-q)(1-q^2) = 1 - q - q^2 + q^3
    // 1/[(1-q)(1-q^2)] has expansion: 1 + q + 2q^2 + 2q^3 + 3q^4 + 3q^5 + ...
    assert_eq!(beta_1.coeff(0), QRat::one());
    assert_eq!(beta_1.coeff(1), QRat::one());
    assert_eq!(beta_1.coeff(2), qrat(2));
    assert_eq!(beta_1.coeff(3), qrat(2));
    assert_eq!(beta_1.coeff(4), qrat(3));
}

/// Unit pair satisfies the Bailey pair relation.
#[test]
fn test_unit_pair_relation() {
    let q = q_var();
    let trunc = 20;
    let a = QMonomial::q(); // a = q

    let pair = BaileyPair {
        name: "unit".into(),
        pair_type: BaileyPairType::Unit,
        tags: vec![],
    };

    assert!(verify_bailey_pair(&pair, &a, 3, q, trunc));
}

// ===========================================================================
// 2. Rogers-Ramanujan pair tests
// ===========================================================================

/// R-R pair alpha_0 should be 1.
#[test]
fn test_rr_pair_alpha_0() {
    let q = q_var();
    let trunc = 20;
    let a = QMonomial::q(); // a = q

    let pair = BaileyPair {
        name: "rr".into(),
        pair_type: BaileyPairType::RogersRamanujan,
        tags: vec![],
    };

    let alpha_0 = pair.alpha_term(0, &a, q, trunc);
    // alpha_0 should be 1 (constant FPS)
    assert_eq!(alpha_0.coeff(0), QRat::one());
    // All other coefficients should be zero
    for k in 1..trunc {
        assert_eq!(alpha_0.coeff(k), QRat::zero(), "alpha_0 nonzero at q^{}", k);
    }
}

/// R-R pair alpha_1 for a = q.
/// alpha_1 = (q;q)_1 * (1-q^3) * (-1) * q^{1*(3*1-1)/2} * q^1 / [(q;q)_1 * (1-q)]
///         = (1-q) * (1-q^3) * (-1) * q * q / [(1-q) * (1-q)]
///         = -(1-q^3) * q^2 / (1-q)
///         = -(1+q+q^2) * q^2
///         = -q^2 - q^3 - q^4
#[test]
fn test_rr_pair_alpha_1() {
    let q = q_var();
    let trunc = 20;
    let a = QMonomial::q(); // a = q

    let pair = BaileyPair {
        name: "rr".into(),
        pair_type: BaileyPairType::RogersRamanujan,
        tags: vec![],
    };

    let alpha_1 = pair.alpha_term(1, &a, q, trunc);
    assert_eq!(alpha_1.coeff(0), QRat::zero());
    assert_eq!(alpha_1.coeff(1), QRat::zero());
    assert_eq!(alpha_1.coeff(2), -QRat::one());
    assert_eq!(alpha_1.coeff(3), -QRat::one());
    assert_eq!(alpha_1.coeff(4), -QRat::one());
    for k in 5..10 {
        assert_eq!(alpha_1.coeff(k), QRat::zero(), "alpha_1 nonzero at q^{}", k);
    }
}

/// R-R pair satisfies the Bailey pair relation for a=q.
#[test]
fn test_rr_pair_relation() {
    let q = q_var();
    let trunc = 20;
    let a = QMonomial::q(); // a = q

    let pair = BaileyPair {
        name: "rr".into(),
        pair_type: BaileyPairType::RogersRamanujan,
        tags: vec![],
    };

    assert!(verify_bailey_pair(&pair, &a, 2, q, trunc));
}

// ===========================================================================
// 3. q-Binomial pair tests
// ===========================================================================

/// q-Binomial pair with z=1: alpha_n = (-1)^n * q^{n(n-1)/2}.
/// alpha_0 = 1, alpha_1 = -1 (q^0 = 1, sign = -1), alpha_2 = q (q^1, sign = +1).
#[test]
fn test_qbinomial_pair_alpha() {
    let q = q_var();
    let trunc = 20;

    let pair = BaileyPair {
        name: "qbinom".into(),
        pair_type: BaileyPairType::QBinomial { z: QRat::one() },
        tags: vec![],
    };

    let a = QMonomial::q();

    // alpha_0 = (-1)^0 * 1^0 * q^0 = 1
    let alpha_0 = pair.alpha_term(0, &a, q, trunc);
    assert_eq!(alpha_0.coeff(0), QRat::one());

    // alpha_1 = (-1)^1 * 1^1 * q^{0} = -1
    let alpha_1 = pair.alpha_term(1, &a, q, trunc);
    assert_eq!(alpha_1.coeff(0), -QRat::one());

    // alpha_2 = (-1)^2 * 1^2 * q^{1} = q
    let alpha_2 = pair.alpha_term(2, &a, q, trunc);
    assert_eq!(alpha_2.coeff(0), QRat::zero());
    assert_eq!(alpha_2.coeff(1), QRat::one());

    // alpha_3 = (-1)^3 * 1^3 * q^{3} = -q^3
    let alpha_3 = pair.alpha_term(3, &a, q, trunc);
    assert_eq!(alpha_3.coeff(3), -QRat::one());
}

/// q-Binomial pair satisfies the Bailey pair relation (z=1, a=q).
#[test]
fn test_qbinomial_pair_relation() {
    let q = q_var();
    let trunc = 20;
    let a = QMonomial::q();

    let pair = BaileyPair {
        name: "qbinom".into(),
        pair_type: BaileyPairType::QBinomial { z: QRat::one() },
        tags: vec![],
    };

    // The q-binomial pair is constructed from the defining relation,
    // so verification should pass by construction.
    assert!(verify_bailey_pair(&pair, &a, 2, q, trunc));
}

// ===========================================================================
// 4. Bailey lemma tests
// ===========================================================================

/// Apply Bailey lemma to unit pair.
/// Use a = q^2, b = (1/2)*q, c = (1/3)*q to avoid vanishing Pochhammer products.
/// Verify the resulting pair satisfies the Bailey pair relation.
#[test]
fn test_bailey_lemma_unit_pair() {
    let q = q_var();
    let trunc = 15;
    let a = QMonomial::q_power(2);
    // b, c chosen so that aq/b and aq/c have non-unit coefficients,
    // preventing (aq/b;q)_n and (aq/c;q)_n from vanishing.
    let b = QMonomial::new(qrat_frac(1, 2), 1); // (1/2)*q
    let c = QMonomial::new(qrat_frac(1, 3), 1); // (1/3)*q
    let max_n = 4;

    let unit_pair = BaileyPair {
        name: "unit".into(),
        pair_type: BaileyPairType::Unit,
        tags: vec![],
    };

    let derived = bailey_lemma(&unit_pair, &a, &b, &c, max_n, q, trunc);

    // The derived pair should be Tabulated
    match &derived.pair_type {
        BaileyPairType::Tabulated { alphas, betas } => {
            assert_eq!(alphas.len() as i64, max_n + 1);
            assert_eq!(betas.len() as i64, max_n + 1);
        }
        _ => panic!("Expected Tabulated pair type"),
    }

    // Verify the derived pair satisfies the Bailey pair relation
    assert!(verify_bailey_pair(&derived, &a, max_n, q, trunc));
}

/// Apply Bailey lemma to Rogers-Ramanujan pair, verify output satisfies the relation.
#[test]
fn test_bailey_lemma_preserves_relation() {
    let q = q_var();
    let trunc = 15;
    let a = QMonomial::q_power(2);
    let b = QMonomial::new(qrat_frac(1, 2), 1);
    let c = QMonomial::new(qrat_frac(1, 3), 1);
    let max_n = 3;

    let rr_pair = BaileyPair {
        name: "rr".into(),
        pair_type: BaileyPairType::RogersRamanujan,
        tags: vec![],
    };

    let derived = bailey_lemma(&rr_pair, &a, &b, &c, max_n, q, trunc);

    // Verify the derived pair satisfies the Bailey pair relation
    assert!(verify_bailey_pair(&derived, &a, max_n, q, trunc));
}

// ===========================================================================
// 5. Bailey chain tests
// ===========================================================================

/// Chain of depth 1: [original, lemma(original)]. Length should be 2.
#[test]
fn test_bailey_chain_depth_1() {
    let q = q_var();
    let trunc = 15;
    let a = QMonomial::q_power(2);
    let b = QMonomial::new(qrat_frac(1, 2), 1);
    let c = QMonomial::new(qrat_frac(1, 3), 1);
    let max_n = 3;

    let unit_pair = BaileyPair {
        name: "unit".into(),
        pair_type: BaileyPairType::Unit,
        tags: vec![],
    };

    let chain = bailey_chain(&unit_pair, &a, &b, &c, 1, max_n, q, trunc);
    assert_eq!(chain.len(), 2);

    // First element is the original
    match &chain[0].pair_type {
        BaileyPairType::Unit => {}
        _ => panic!("First chain element should be Unit"),
    }

    // Second element is derived
    match &chain[1].pair_type {
        BaileyPairType::Tabulated { .. } => {}
        _ => panic!("Second chain element should be Tabulated"),
    }
}

/// Chain of depth 2: length 3, each pair satisfies the Bailey pair relation.
#[test]
fn test_bailey_chain_depth_2() {
    let q = q_var();
    let trunc = 15;
    let a = QMonomial::q_power(2);
    let b = QMonomial::new(qrat_frac(1, 2), 1);
    let c = QMonomial::new(qrat_frac(1, 3), 1);
    let max_n = 3;

    let unit_pair = BaileyPair {
        name: "unit".into(),
        pair_type: BaileyPairType::Unit,
        tags: vec![],
    };

    let chain = bailey_chain(&unit_pair, &a, &b, &c, 2, max_n, q, trunc);
    assert_eq!(chain.len(), 3);

    // Each pair in the chain should satisfy the Bailey pair relation
    for (i, pair) in chain.iter().enumerate() {
        assert!(
            verify_bailey_pair(pair, &a, max_n, q, trunc),
            "Chain pair {} does not satisfy Bailey pair relation",
            i
        );
    }
}

// ===========================================================================
// 6. Weak Bailey lemma tests
// ===========================================================================

/// Weak Bailey lemma for unit pair with a=q:
/// LHS = sum q^{n^2+n} * beta_n (only n=0 term has non-trivial alpha)
/// RHS = 1/(q^2;q)_inf * sum q^{n^2+n} * alpha_n = 1/(q^2;q)_inf * q
/// (only n=0 contributes to alpha sum: q^0 * q^0 * 1 = 1, weighted by q^{0+0}=1;
///  actually n=0: q^{0} * q^0 * alpha_0 = 1.)
///
/// The identity should hold: LHS == RHS.
#[test]
fn test_weak_bailey_lemma_unit_pair() {
    let q = q_var();
    let trunc = 15;
    let a = QMonomial::q(); // a = q
    let max_n = 10;

    let pair = BaileyPair {
        name: "unit".into(),
        pair_type: BaileyPairType::Unit,
        tags: vec![],
    };

    let (lhs, rhs) = weak_bailey_lemma(&pair, &a, max_n, q, trunc);

    // LHS and RHS should be equal as FPS
    let diff = arithmetic::sub(&lhs, &rhs);
    assert!(
        diff.is_zero(),
        "Weak Bailey lemma failed for unit pair. LHS-RHS has {} nonzero coefficients",
        diff.num_nonzero()
    );
}

/// Weak Bailey lemma for Rogers-Ramanujan pair with a=q:
/// LHS = sum_{n>=0} q^{n^2+n} * beta_n = sum_{n>=0} q^{n^2+n} / (q;q)_n
/// RHS = [1/(q^2;q)_inf] * sum_{n>=0} q^{n^2+n} * alpha_n
///
/// We verify that LHS == RHS as FPS.
#[test]
fn test_weak_bailey_lemma_rr_pair() {
    let q = q_var();
    let trunc = 15;
    let a = QMonomial::q(); // a = q
    let max_n = 8;

    let pair = BaileyPair {
        name: "rr".into(),
        pair_type: BaileyPairType::RogersRamanujan,
        tags: vec![],
    };

    let (lhs, rhs) = weak_bailey_lemma(&pair, &a, max_n, q, trunc);

    // LHS and RHS should be equal
    let diff = arithmetic::sub(&lhs, &rhs);
    assert!(
        diff.is_zero(),
        "Weak Bailey lemma failed for R-R pair. Difference has {} nonzero coefficients",
        diff.num_nonzero()
    );
}

/// Weak Bailey lemma for Rogers-Ramanujan pair with a=1 (limit form):
/// This gives the first Rogers-Ramanujan identity.
/// LHS = sum_{n>=0} q^{n^2} / (q;q)_n
/// = 1 + q + q^2 + q^3 + 2q^4 + 2q^5 + 3q^6 + ...
/// (OEIS A003114: partitions into parts == 1 or 4 mod 5)
#[test]
fn test_weak_bailey_lemma_rr_pair_a_one() {
    let q = q_var();
    let trunc = 20;
    let a = QMonomial::one(); // a = 1
    let max_n = 10;

    let pair = BaileyPair {
        name: "rr".into(),
        pair_type: BaileyPairType::RogersRamanujan,
        tags: vec![],
    };

    let (lhs, rhs) = weak_bailey_lemma(&pair, &a, max_n, q, trunc);

    // LHS and RHS should be equal
    let diff = arithmetic::sub(&lhs, &rhs);
    assert!(
        diff.is_zero(),
        "Weak Bailey lemma failed for R-R pair (a=1). Difference has {} nonzero coefficients",
        diff.num_nonzero()
    );

    // Verify LHS starts with the known Rogers-Ramanujan coefficients
    assert_eq!(lhs.coeff(0), qrat(1));
    assert_eq!(lhs.coeff(1), qrat(1));
    assert_eq!(lhs.coeff(2), qrat(1));
    assert_eq!(lhs.coeff(3), qrat(1));
    assert_eq!(lhs.coeff(4), qrat(2));
    assert_eq!(lhs.coeff(5), qrat(2));
    assert_eq!(lhs.coeff(6), qrat(3));
}

// ===========================================================================
// 7. Database tests
// ===========================================================================

/// Default database has 3+ pairs (unit, R-R, q-binomial).
#[test]
fn test_database_default_pairs() {
    let db = BaileyDatabase::new();
    assert!(db.len() >= 3, "Expected at least 3 pairs, got {}", db.len());
}

/// Search by tag "canonical" finds pairs.
#[test]
fn test_database_search_by_tag() {
    let db = BaileyDatabase::new();
    let canonical = db.search_by_tag("canonical");
    assert!(canonical.len() >= 3, "Expected at least 3 canonical pairs");
}

/// Search by tag is case-insensitive.
#[test]
fn test_database_search_by_tag_case_insensitive() {
    let db = BaileyDatabase::new();
    let canonical = db.search_by_tag("CANONICAL");
    assert!(canonical.len() >= 3, "Case-insensitive tag search failed");
}

/// Search by name "rogers" finds Rogers-Ramanujan.
#[test]
fn test_database_search_by_name() {
    let db = BaileyDatabase::new();
    let rr = db.search_by_name("rogers");
    assert_eq!(rr.len(), 1, "Expected exactly 1 Rogers-Ramanujan pair");
    assert!(rr[0].name.contains("rogers-ramanujan"));
}

/// Add a custom Tabulated pair and find it.
#[test]
fn test_database_add_pair() {
    let q = q_var();
    let trunc = 10;

    let mut db = BaileyDatabase::new();
    let initial_len = db.len();

    let custom = BaileyPair {
        name: "custom-test".into(),
        pair_type: BaileyPairType::Tabulated {
            alphas: vec![FormalPowerSeries::one(q, trunc)],
            betas: vec![FormalPowerSeries::one(q, trunc)],
        },
        tags: vec!["custom".into()],
    };

    db.add(custom);
    assert_eq!(db.len(), initial_len + 1);

    let found = db.search_by_tag("custom");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "custom-test");

    let found_name = db.search_by_name("custom-test");
    assert_eq!(found_name.len(), 1);
}

/// Database all_pairs returns correct slice.
#[test]
fn test_database_all_pairs() {
    let db = BaileyDatabase::new();
    assert_eq!(db.all_pairs().len(), db.len());
}

// ===========================================================================
// 8. Additional verification tests
// ===========================================================================

/// Verify unit pair relation with a different value of a: a = q^2.
#[test]
fn test_unit_pair_relation_a_q2() {
    let q = q_var();
    let trunc = 15;
    let a = QMonomial::q_power(2); // a = q^2

    let pair = BaileyPair {
        name: "unit".into(),
        pair_type: BaileyPairType::Unit,
        tags: vec![],
    };

    assert!(verify_bailey_pair(&pair, &a, 3, q, trunc));
}

/// R-R pair relation with a = q^2.
#[test]
fn test_rr_pair_relation_a_q2() {
    let q = q_var();
    let trunc = 15;
    let a = QMonomial::q_power(2); // a = q^2

    let pair = BaileyPair {
        name: "rr".into(),
        pair_type: BaileyPairType::RogersRamanujan,
        tags: vec![],
    };

    assert!(verify_bailey_pair(&pair, &a, 2, q, trunc));
}
