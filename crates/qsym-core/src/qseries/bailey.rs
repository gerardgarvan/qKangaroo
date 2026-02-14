//! Bailey pairs, Bailey lemma, chain iteration, and weak Bailey lemma.
//!
//! A Bailey pair relative to parameter `a` is a pair of sequences (alpha_n, beta_n)
//! satisfying the Bailey pair relation:
//!
//!   beta_n = sum_{j=0}^{n} alpha_j / [(q;q)_{n-j} * (aq;q)_{n+j}]
//!
//! The Bailey lemma transforms one pair into another, and iterated application
//! (Bailey chains) produces infinite families of q-series identities.
//!
//! The weak Bailey lemma states:
//!   sum_{n>=0} q^{n^2} * a^n * beta_n = [1/(aq;q)_inf] * sum_{n>=0} q^{n^2} * a^n * alpha_n
//!
//! References: DLMF 17.12, Andrews (1984), Warnaar (2001).

use crate::number::QRat;
use crate::series::{FormalPowerSeries, arithmetic};
use crate::symbol::SymbolId;
use super::{QMonomial, PochhammerOrder, aqprod};

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Classification of Bailey pair types.
#[derive(Clone, Debug)]
pub enum BaileyPairType {
    /// Unit pair: alpha_0 = 1, alpha_n = 0 for n > 0;
    /// beta_n = 1/[(q;q)_n * (aq;q)_n].
    Unit,

    /// Rogers-Ramanujan pair (DLMF 17.12.6):
    /// alpha_n = (a;q)_n * (1 - a*q^{2n}) * (-1)^n * q^{n(3n-1)/2} * a^n / [(q;q)_n * (1-a)]
    /// beta_n = 1/(q;q)_n.
    RogersRamanujan,

    /// q-Binomial pair with parameter z:
    /// alpha_n = (-1)^n * z^n * q^{n(n-1)/2}
    /// beta_n involves (z;q)_n and (q;q)_n factors.
    QBinomial { z: QRat },

    /// Explicit coefficient tables (result of lemma application).
    /// Each entry is an FPS (not just a QRat), since Bailey pair terms
    /// can involve q-power contributions.
    Tabulated {
        alphas: Vec<FormalPowerSeries>,
        betas: Vec<FormalPowerSeries>,
    },
}

/// A Bailey pair with metadata for database storage.
#[derive(Clone, Debug)]
pub struct BaileyPair {
    /// Name/identifier for this pair.
    pub name: String,
    /// Type classification and evaluation data.
    pub pair_type: BaileyPairType,
    /// Tags for search (e.g., "canonical", "derived").
    pub tags: Vec<String>,
}

impl BaileyPair {
    /// Evaluate the n-th alpha term as an FPS.
    ///
    /// The parameter `a` is passed at evaluation time (not stored in the pair),
    /// since most classical pairs work for general `a`.
    pub fn alpha_term(
        &self,
        n: i64,
        a: &QMonomial,
        variable: SymbolId,
        truncation_order: i64,
    ) -> FormalPowerSeries {
        match &self.pair_type {
            BaileyPairType::Unit => {
                if n == 0 {
                    FormalPowerSeries::one(variable, truncation_order)
                } else {
                    FormalPowerSeries::zero(variable, truncation_order)
                }
            }
            BaileyPairType::RogersRamanujan => {
                rr_alpha_term(n, a, variable, truncation_order)
            }
            BaileyPairType::QBinomial { z } => {
                qbinom_alpha_term(n, z, variable, truncation_order)
            }
            BaileyPairType::Tabulated { alphas, .. } => {
                let idx = n as usize;
                if idx < alphas.len() {
                    alphas[idx].clone()
                } else {
                    FormalPowerSeries::zero(variable, truncation_order)
                }
            }
        }
    }

    /// Evaluate the n-th beta term as an FPS.
    pub fn beta_term(
        &self,
        n: i64,
        a: &QMonomial,
        variable: SymbolId,
        truncation_order: i64,
    ) -> FormalPowerSeries {
        match &self.pair_type {
            BaileyPairType::Unit => {
                unit_beta_term(n, a, variable, truncation_order)
            }
            BaileyPairType::RogersRamanujan => {
                rr_beta_term(n, variable, truncation_order)
            }
            BaileyPairType::QBinomial { z } => {
                qbinom_beta_term(n, a, z, variable, truncation_order)
            }
            BaileyPairType::Tabulated { betas, .. } => {
                let idx = n as usize;
                if idx < betas.len() {
                    betas[idx].clone()
                } else {
                    FormalPowerSeries::zero(variable, truncation_order)
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Pair evaluation helpers
// ---------------------------------------------------------------------------

/// Unit pair beta_n = 1/[(q;q)_n * (aq;q)_n].
fn unit_beta_term(
    n: i64,
    a: &QMonomial,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let q_q_n = aqprod(&QMonomial::q(), variable, PochhammerOrder::Finite(n), truncation_order);
    let aq = a.mul(&QMonomial::q());
    let aq_q_n = aqprod(&aq, variable, PochhammerOrder::Finite(n), truncation_order);
    let denom = arithmetic::mul(&q_q_n, &aq_q_n);
    arithmetic::invert(&denom)
}

/// Rogers-Ramanujan alpha_n:
/// alpha_n = (a;q)_n * (1 - a*q^{2n}) * (-1)^n * q^{n(3n-1)/2} * a^n / [(q;q)_n * (1-a)]
///
/// Special cases:
/// - alpha_0 = 1 (all factors collapse).
/// - a = 1 (removable singularity): use limit form
///   alpha_n = (1+q^n) * (-1)^n * q^{n(3n-1)/2} for n >= 1.
fn rr_alpha_term(
    n: i64,
    a: &QMonomial,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    if n == 0 {
        // alpha_0 = 1 for all a.
        return FormalPowerSeries::one(variable, truncation_order);
    }

    // Special case: a = 1 (coeff=1, power=0). The formula has a removable singularity.
    // Limit form: alpha_n = (1+q^n) * (-1)^n * q^{n(3n-1)/2}
    // Derivation: (a;q)_n/(1-a) -> (q;q)_{n-1} as a->1, and
    // (1-q^{2n}) / (q;q)_n * (q;q)_{n-1} = (1-q^{2n})/(1-q^n) = (1+q^n).
    if a.coeff == QRat::one() && a.power == 0 {
        let sign: QRat = if n % 2 == 0 { QRat::one() } else { -QRat::one() };
        let q_exp = n * (3 * n - 1) / 2;
        // (1 + q^n) * sign * q^{q_exp}
        let term1 = FormalPowerSeries::monomial(variable, sign.clone(), q_exp, truncation_order);
        let term2 = FormalPowerSeries::monomial(variable, sign, q_exp + n, truncation_order);
        return arithmetic::add(&term1, &term2);
    }

    // (a;q)_n
    let a_poch_n = aqprod(a, variable, PochhammerOrder::Finite(n), truncation_order);

    // (1 - a*q^{2n}) as FPS
    let aq2n = a.mul(&QMonomial::q_power(2 * n));
    let one_minus_aq2n = {
        let mut f = FormalPowerSeries::one(variable, truncation_order);
        if aq2n.power == 0 {
            f.set_coeff(0, QRat::one() - aq2n.coeff.clone());
        } else if aq2n.power < truncation_order {
            f.set_coeff(aq2n.power, -aq2n.coeff.clone());
        }
        f
    };

    // (-1)^n * q^{n(3n-1)/2} * a^n
    // a^n means coeff^n * q^{power*n}
    let sign: QRat = if n % 2 == 0 { QRat::one() } else { -QRat::one() };
    let q_exp = n * (3 * n - 1) / 2 + a.power * n;
    let a_coeff_n = qrat_pow(&a.coeff, n);
    let scalar = sign * a_coeff_n;
    let q_factor = FormalPowerSeries::monomial(variable, scalar, q_exp, truncation_order);

    // (q;q)_n
    let q_q_n = aqprod(&QMonomial::q(), variable, PochhammerOrder::Finite(n), truncation_order);

    // (1 - a) as FPS
    let one_minus_a = {
        let mut f = FormalPowerSeries::one(variable, truncation_order);
        if a.power == 0 {
            f.set_coeff(0, QRat::one() - a.coeff.clone());
        } else if a.power < truncation_order {
            f.set_coeff(a.power, -a.coeff.clone());
        }
        f
    };

    // numerator = (a;q)_n * (1 - a*q^{2n}) * q_factor
    let numer = arithmetic::mul(&arithmetic::mul(&a_poch_n, &one_minus_aq2n), &q_factor);

    // denominator = (q;q)_n * (1 - a)
    let denom = arithmetic::mul(&q_q_n, &one_minus_a);

    // alpha_n = numer / denom
    arithmetic::mul(&numer, &arithmetic::invert(&denom))
}

/// Rogers-Ramanujan beta_n = 1/(q;q)_n.
fn rr_beta_term(
    n: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let q_q_n = aqprod(&QMonomial::q(), variable, PochhammerOrder::Finite(n), truncation_order);
    arithmetic::invert(&q_q_n)
}

/// q-Binomial alpha_n = (-1)^n * z^n * q^{n(n-1)/2}.
fn qbinom_alpha_term(
    n: i64,
    z: &QRat,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    let sign: QRat = if n % 2 == 0 { QRat::one() } else { -QRat::one() };
    let z_n = qrat_pow(z, n);
    let q_exp = n * (n - 1) / 2;
    let coeff = sign * z_n;
    FormalPowerSeries::monomial(variable, coeff, q_exp, truncation_order)
}

/// q-Binomial beta_n = 1/[(q;q)_n * (aq;q)_n] for the q-binomial pair.
///
/// For the q-binomial pair relative to a with parameter z, a simplified form is:
/// beta_n = 1/(q;q)_n when z = 0 (degenerate),
/// but in general the pair definition requires the relation to hold.
///
/// For the standard q-binomial conjugate pair:
/// beta_n = 1/[(q;q)_n] * product involving z.
///
/// We use the general relation-derived form: since alpha_n = (-1)^n z^n q^{n(n-1)/2},
/// and the pair must satisfy beta_n = sum_{j=0}^{n} alpha_j / [(q;q)_{n-j} * (aq;q)_{n+j}],
/// we compute beta directly from the defining relation for correctness.
fn qbinom_beta_term(
    n: i64,
    a: &QMonomial,
    z: &QRat,
    variable: SymbolId,
    truncation_order: i64,
) -> FormalPowerSeries {
    // Compute beta_n from the defining relation:
    // beta_n = sum_{j=0}^{n} alpha_j / [(q;q)_{n-j} * (aq;q)_{n+j}]
    let aq = a.mul(&QMonomial::q());
    let mut result = FormalPowerSeries::zero(variable, truncation_order);

    for j in 0..=n {
        let alpha_j = qbinom_alpha_term(j, z, variable, truncation_order);
        let q_q_nj = aqprod(&QMonomial::q(), variable, PochhammerOrder::Finite(n - j), truncation_order);
        let aq_q_npj = aqprod(&aq, variable, PochhammerOrder::Finite(n + j), truncation_order);
        let denom = arithmetic::mul(&q_q_nj, &aq_q_npj);
        let term = arithmetic::mul(&alpha_j, &arithmetic::invert(&denom));
        result = arithmetic::add(&result, &term);
    }

    result
}

/// Helper: compute r^n for QRat r and i64 n >= 0.
fn qrat_pow(r: &QRat, n: i64) -> QRat {
    assert!(n >= 0, "qrat_pow requires n >= 0");
    if n == 0 {
        return QRat::one();
    }
    let mut result = r.clone();
    for _ in 1..n {
        result = result * r.clone();
    }
    result
}

// ---------------------------------------------------------------------------
// Bailey pair verification
// ---------------------------------------------------------------------------

/// Verify the Bailey pair relation for indices 0..=max_n:
///
///   beta_n = sum_{j=0}^{n} alpha_j / [(q;q)_{n-j} * (aq;q)_{n+j}]
///
/// Returns true if all indices match (as FPS, to truncation_order).
pub fn verify_bailey_pair(
    pair: &BaileyPair,
    a: &QMonomial,
    max_n: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> bool {
    let aq = a.mul(&QMonomial::q());

    for n in 0..=max_n {
        let beta_n = pair.beta_term(n, a, variable, truncation_order);

        // Compute the sum: sum_{j=0}^{n} alpha_j / [(q;q)_{n-j} * (aq;q)_{n+j}]
        let mut relation_sum = FormalPowerSeries::zero(variable, truncation_order);

        for j in 0..=n {
            let alpha_j = pair.alpha_term(j, a, variable, truncation_order);
            let q_q_nj = aqprod(
                &QMonomial::q(),
                variable,
                PochhammerOrder::Finite(n - j),
                truncation_order,
            );
            let aq_q_npj = aqprod(
                &aq,
                variable,
                PochhammerOrder::Finite(n + j),
                truncation_order,
            );
            let denom = arithmetic::mul(&q_q_nj, &aq_q_npj);
            let term = arithmetic::mul(&alpha_j, &arithmetic::invert(&denom));
            relation_sum = arithmetic::add(&relation_sum, &term);
        }

        // Compare beta_n with the computed sum
        let diff = arithmetic::sub(&beta_n, &relation_sum);
        if !diff.is_zero() {
            return false;
        }
    }

    true
}

// ---------------------------------------------------------------------------
// Bailey lemma
// ---------------------------------------------------------------------------

/// Apply the Bailey lemma to transform (alpha, beta) into (alpha', beta').
///
/// Given Bailey pair relative to `a`, with parameters `b`, `c`:
///
/// alpha'_n = [(b;q)_n * (c;q)_n * (aq/(bc))^n] / [(aq/b;q)_n * (aq/c;q)_n] * alpha_n
///
/// beta'_n = [1/((aq/b;q)_n * (aq/c;q)_n)] * sum_{k=0}^{n}
///           [(b;q)_k * (c;q)_k * (aq/(bc);q)_{n-k} * (aq/(bc))^k / (q;q)_{n-k}] * beta_k
///
/// Returns a new BaileyPair of Tabulated type.
pub fn bailey_lemma(
    pair: &BaileyPair,
    a: &QMonomial,
    b: &QMonomial,
    c: &QMonomial,
    max_n: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> BaileyPair {
    let aq = a.mul(&QMonomial::q());
    let bc = b.mul(c);
    let aq_over_bc = aq.div(&bc);
    let aq_over_b = aq.div(b);
    let aq_over_c = aq.div(c);

    let mut new_alphas = Vec::new();
    let mut new_betas = Vec::new();

    for n in 0..=max_n {
        // alpha'_n computation
        let b_poch_n = aqprod(b, variable, PochhammerOrder::Finite(n), truncation_order);
        let c_poch_n = aqprod(c, variable, PochhammerOrder::Finite(n), truncation_order);
        let aq_b_poch_n = aqprod(&aq_over_b, variable, PochhammerOrder::Finite(n), truncation_order);
        let aq_c_poch_n = aqprod(&aq_over_c, variable, PochhammerOrder::Finite(n), truncation_order);

        // (aq/(bc))^n as FPS monomial
        let aq_bc_pow_coeff = qrat_pow(&aq_over_bc.coeff, n);
        let aq_bc_pow_exp = aq_over_bc.power * n;
        let aq_bc_pow_n = FormalPowerSeries::monomial(
            variable,
            aq_bc_pow_coeff,
            aq_bc_pow_exp,
            truncation_order,
        );

        let old_alpha_n = pair.alpha_term(n, a, variable, truncation_order);

        let numer = arithmetic::mul(
            &arithmetic::mul(&b_poch_n, &c_poch_n),
            &arithmetic::mul(&aq_bc_pow_n, &old_alpha_n),
        );
        let denom = arithmetic::mul(&aq_b_poch_n, &aq_c_poch_n);
        let alpha_prime_n = arithmetic::mul(&numer, &arithmetic::invert(&denom));
        new_alphas.push(alpha_prime_n);

        // beta'_n computation
        let aq_b_poch_n_outer = aqprod(&aq_over_b, variable, PochhammerOrder::Finite(n), truncation_order);
        let aq_c_poch_n_outer = aqprod(&aq_over_c, variable, PochhammerOrder::Finite(n), truncation_order);
        let outer_denom = arithmetic::mul(&aq_b_poch_n_outer, &aq_c_poch_n_outer);
        let outer_inv = arithmetic::invert(&outer_denom);

        let mut inner_sum = FormalPowerSeries::zero(variable, truncation_order);
        for k in 0..=n {
            let b_poch_k = aqprod(b, variable, PochhammerOrder::Finite(k), truncation_order);
            let c_poch_k = aqprod(c, variable, PochhammerOrder::Finite(k), truncation_order);
            let aq_bc_poch_nk = aqprod(
                &aq_over_bc,
                variable,
                PochhammerOrder::Finite(n - k),
                truncation_order,
            );
            let q_q_nk = aqprod(
                &QMonomial::q(),
                variable,
                PochhammerOrder::Finite(n - k),
                truncation_order,
            );

            // (aq/(bc))^k
            let aq_bc_pow_k_coeff = qrat_pow(&aq_over_bc.coeff, k);
            let aq_bc_pow_k_exp = aq_over_bc.power * k;
            let aq_bc_pow_k = FormalPowerSeries::monomial(
                variable,
                aq_bc_pow_k_coeff,
                aq_bc_pow_k_exp,
                truncation_order,
            );

            let old_beta_k = pair.beta_term(k, a, variable, truncation_order);

            let term_numer = arithmetic::mul(
                &arithmetic::mul(&b_poch_k, &c_poch_k),
                &arithmetic::mul(
                    &arithmetic::mul(&aq_bc_poch_nk, &aq_bc_pow_k),
                    &old_beta_k,
                ),
            );
            let term = arithmetic::mul(&term_numer, &arithmetic::invert(&q_q_nk));
            inner_sum = arithmetic::add(&inner_sum, &term);
        }

        let beta_prime_n = arithmetic::mul(&outer_inv, &inner_sum);
        new_betas.push(beta_prime_n);
    }

    BaileyPair {
        name: format!("lemma({}, b={:?}, c={:?})", pair.name, b, c),
        pair_type: BaileyPairType::Tabulated {
            alphas: new_alphas,
            betas: new_betas,
        },
        tags: vec!["derived".into()],
    }
}

// ---------------------------------------------------------------------------
// Bailey chain
// ---------------------------------------------------------------------------

/// Apply the Bailey lemma `depth` times with the same parameters b, c.
///
/// Returns the chain of pairs: [original, after 1 application, after 2, ...].
/// The chain has length `depth + 1`.
pub fn bailey_chain(
    pair: &BaileyPair,
    a: &QMonomial,
    b: &QMonomial,
    c: &QMonomial,
    depth: usize,
    max_n: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> Vec<BaileyPair> {
    let mut chain = Vec::with_capacity(depth + 1);
    chain.push(pair.clone());

    let mut current = pair.clone();
    for _ in 0..depth {
        let next = bailey_lemma(&current, a, b, c, max_n, variable, truncation_order);
        chain.push(next.clone());
        current = next;
    }

    chain
}

// ---------------------------------------------------------------------------
// Weak Bailey lemma
// ---------------------------------------------------------------------------

/// Compute both sides of the weak Bailey lemma identity:
///
///   LHS = sum_{n>=0} q^{n^2} * a^n * beta_n
///   RHS = [1/(aq;q)_inf] * sum_{n>=0} q^{n^2} * a^n * alpha_n
///
/// Returns (LHS, RHS). The caller can verify equality.
pub fn weak_bailey_lemma(
    pair: &BaileyPair,
    a: &QMonomial,
    max_n: i64,
    variable: SymbolId,
    truncation_order: i64,
) -> (FormalPowerSeries, FormalPowerSeries) {
    // LHS: sum_{n=0}^{max_n} q^{n^2} * a^n * beta_term(n)
    let mut lhs = FormalPowerSeries::zero(variable, truncation_order);
    for n in 0..=max_n {
        let q_exp = n * n + a.power * n;
        if q_exp >= truncation_order {
            break;
        }
        let a_coeff_n = qrat_pow(&a.coeff, n);
        let weight = FormalPowerSeries::monomial(variable, a_coeff_n, q_exp, truncation_order);
        let beta_n = pair.beta_term(n, a, variable, truncation_order);
        let term = arithmetic::mul(&weight, &beta_n);
        lhs = arithmetic::add(&lhs, &term);
    }

    // RHS: [1/(aq;q)_inf] * sum_{n=0}^{max_n} q^{n^2} * a^n * alpha_term(n)
    let aq = a.mul(&QMonomial::q());
    let aq_inf = aqprod(&aq, variable, PochhammerOrder::Infinite, truncation_order);
    let aq_inf_inv = arithmetic::invert(&aq_inf);

    let mut alpha_sum = FormalPowerSeries::zero(variable, truncation_order);
    for n in 0..=max_n {
        let q_exp = n * n + a.power * n;
        if q_exp >= truncation_order {
            break;
        }
        let a_coeff_n = qrat_pow(&a.coeff, n);
        let weight = FormalPowerSeries::monomial(variable, a_coeff_n, q_exp, truncation_order);
        let alpha_n = pair.alpha_term(n, a, variable, truncation_order);
        let term = arithmetic::mul(&weight, &alpha_n);
        alpha_sum = arithmetic::add(&alpha_sum, &term);
    }

    let rhs = arithmetic::mul(&aq_inf_inv, &alpha_sum);

    (lhs, rhs)
}

// ---------------------------------------------------------------------------
// Bailey Database
// ---------------------------------------------------------------------------

/// A database of Bailey pairs with search capabilities.
#[derive(Clone, Debug)]
pub struct BaileyDatabase {
    pairs: Vec<BaileyPair>,
}

impl BaileyDatabase {
    /// Create a new database with the default canonical pairs.
    pub fn new() -> Self {
        let mut db = BaileyDatabase { pairs: Vec::new() };

        db.pairs.push(BaileyPair {
            name: "unit".into(),
            pair_type: BaileyPairType::Unit,
            tags: vec!["canonical".into(), "unit".into()],
        });

        db.pairs.push(BaileyPair {
            name: "rogers-ramanujan".into(),
            pair_type: BaileyPairType::RogersRamanujan,
            tags: vec!["canonical".into(), "rogers-ramanujan".into()],
        });

        db.pairs.push(BaileyPair {
            name: "q-binomial(z=1)".into(),
            pair_type: BaileyPairType::QBinomial { z: QRat::one() },
            tags: vec!["canonical".into(), "q-binomial".into()],
        });

        db
    }

    /// Add a pair to the database.
    pub fn add(&mut self, pair: BaileyPair) {
        self.pairs.push(pair);
    }

    /// Search by tag (case-insensitive).
    pub fn search_by_tag(&self, tag: &str) -> Vec<&BaileyPair> {
        let tag_lower = tag.to_lowercase();
        self.pairs
            .iter()
            .filter(|p| p.tags.iter().any(|t| t.to_lowercase() == tag_lower))
            .collect()
    }

    /// Search by name (case-insensitive substring match).
    pub fn search_by_name(&self, name: &str) -> Vec<&BaileyPair> {
        let name_lower = name.to_lowercase();
        self.pairs
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&name_lower))
            .collect()
    }

    /// Get all pairs.
    pub fn all_pairs(&self) -> &[BaileyPair] {
        &self.pairs
    }

    /// Number of pairs in the database.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }
}

// ---------------------------------------------------------------------------
// Discovery
// ---------------------------------------------------------------------------

/// Result of automated Bailey pair discovery.
#[derive(Clone, Debug)]
pub struct DiscoveryResult {
    /// Whether a matching Bailey pair was found.
    pub found: bool,
    /// Name of the matching pair (if found).
    pub pair_name: Option<String>,
    /// Chain depth at which the match was found (0 = direct match).
    pub chain_depth: usize,
    /// The matching pair (if found).
    pub matching_pair: Option<BaileyPair>,
    /// Description of how the identity was verified.
    pub verification: String,
}

/// Compare two FPS for equality by checking if their difference is zero.
fn fps_equal(a: &FormalPowerSeries, b: &FormalPowerSeries) -> bool {
    let diff = arithmetic::sub(a, b);
    diff.is_zero()
}

/// Automated Bailey pair discovery: search the pair database (and chain
/// iterations) to verify whether an identity LHS == RHS can be explained
/// by the weak Bailey lemma applied to some known pair.
///
/// Algorithm:
/// 1. If LHS == RHS directly, return trivially verified.
/// 2. For each pair in the database, compute the weak Bailey lemma.
///    If the resulting (wbl_lhs, wbl_rhs) satisfy wbl_lhs == wbl_rhs AND
///    lhs == wbl_lhs, the identity is verified via that pair.
/// 3. If no direct match, try chain iteration up to `max_chain_depth`.
///    For each depth d and each pair, apply `bailey_chain` with default
///    parameters b = (1/2)*q, c = (1/3)*q, then compute the weak Bailey
///    lemma for the last pair in the chain and check for a match.
/// 4. If no match at any depth, return found=false.
pub fn bailey_discover(
    lhs: &FormalPowerSeries,
    rhs: &FormalPowerSeries,
    db: &BaileyDatabase,
    a: &QMonomial,
    max_chain_depth: usize,
    variable: SymbolId,
    truncation_order: i64,
) -> DiscoveryResult {
    // Step 1: Trivial equality
    if fps_equal(lhs, rhs) {
        return DiscoveryResult {
            found: true,
            pair_name: None,
            chain_depth: 0,
            matching_pair: None,
            verification: "direct equality".into(),
        };
    }

    // Compute max_n from truncation_order (n^2 < trunc => n < sqrt(trunc))
    let max_n = {
        let mut n = 0i64;
        while (n + 1) * (n + 1) < truncation_order {
            n += 1;
        }
        n
    };

    // Step 2: Direct match via weak Bailey lemma
    for pair in db.all_pairs() {
        let (wbl_lhs, wbl_rhs) = weak_bailey_lemma(pair, a, max_n, variable, truncation_order);
        if fps_equal(&wbl_lhs, &wbl_rhs) && fps_equal(lhs, &wbl_lhs) {
            return DiscoveryResult {
                found: true,
                pair_name: Some(pair.name.clone()),
                chain_depth: 0,
                matching_pair: Some(pair.clone()),
                verification: format!(
                    "weak Bailey lemma with pair '{}' at chain depth 0",
                    pair.name
                ),
            };
        }
    }

    // Step 3: Chain iteration
    // Default chain parameters: b = (1/2)*q, c = (1/3)*q
    // These avoid vanishing Pochhammer products for general a.
    let b = QMonomial::new(QRat::from((1i64, 2i64)), 1);
    let c = QMonomial::new(QRat::from((1i64, 3i64)), 1);

    for depth in 1..=max_chain_depth {
        for pair in db.all_pairs() {
            let chain = bailey_chain(pair, a, &b, &c, depth, max_n, variable, truncation_order);
            // The last pair in the chain is the one we test
            if let Some(derived) = chain.last() {
                let (wbl_lhs, wbl_rhs) =
                    weak_bailey_lemma(derived, a, max_n, variable, truncation_order);
                if fps_equal(&wbl_lhs, &wbl_rhs) && fps_equal(lhs, &wbl_lhs) {
                    return DiscoveryResult {
                        found: true,
                        pair_name: Some(pair.name.clone()),
                        chain_depth: depth,
                        matching_pair: Some(derived.clone()),
                        verification: format!(
                            "weak Bailey lemma with pair '{}' at chain depth {}",
                            pair.name, depth
                        ),
                    };
                }
            }
        }
    }

    // Step 4: No match found
    DiscoveryResult {
        found: false,
        pair_name: None,
        chain_depth: 0,
        matching_pair: None,
        verification: "no matching pair found in database".into(),
    }
}
