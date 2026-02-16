use pyo3::prelude::*;

mod convert;
mod dsl;
mod expr;
mod series;
mod session;

/// Return the version string, exercising GMP linkage by creating and
/// dropping an ExprArena (which internally allocates rug/GMP values).
#[pyfunction]
fn version() -> &'static str {
    // Create and immediately drop an ExprArena to prove that qsym-core
    // (and its GMP dependency via rug) links and works at runtime.
    let arena = qsym_core::ExprArena::new();
    drop(arena);
    "0.1.0"
}

/// The native q-Kangaroo Python module entry point.
#[pymodule]
fn _q_kangaroo(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_class::<session::QSession>()?;
    m.add_class::<expr::QExpr>()?;
    m.add_class::<series::QSeries>()?;

    // Group 1: Pochhammer and q-Binomial
    m.add_function(wrap_pyfunction!(dsl::aqprod, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::qbin, m)?)?;

    // Group 2: Named Products
    m.add_function(wrap_pyfunction!(dsl::etaq, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::jacprod, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::tripleprod, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::quinprod, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::winquist, m)?)?;

    // Group 3: Theta Functions
    m.add_function(wrap_pyfunction!(dsl::theta2, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::theta3, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::theta4, m)?)?;

    // Group 4: Partition Functions
    m.add_function(wrap_pyfunction!(dsl::partition_count, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::partition_gf, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::distinct_parts_gf, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::odd_parts_gf, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::bounded_parts_gf, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::rank_gf, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::crank_gf, m)?)?;

    // Group 5: Factoring, Utilities, and Prodmake
    m.add_function(wrap_pyfunction!(dsl::qfactor, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::sift_fn, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::qdegree, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::lqdegree, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::prodmake, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::etamake, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::jacprodmake, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mprodmake, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::qetamake, m)?)?;

    // Group 6: Relation Discovery (exact rational)
    m.add_function(wrap_pyfunction!(dsl::findlincombo, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::findhom, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::findpoly, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::findcong, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::findnonhom, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::findhomcombo, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::findnonhomcombo, m)?)?;

    // Group 7: Relation Discovery (modular and structural)
    m.add_function(wrap_pyfunction!(dsl::findlincombomodp, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::findhommodp, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::findhomcombomodp, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::findmaxind, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::findprod, m)?)?;

    // Group 8: Hypergeometric Series
    m.add_function(wrap_pyfunction!(dsl::phi, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::psi, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::try_summation, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::heine1, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::heine2, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::heine3, m)?)?;

    // Group 9: Identity Proving
    m.add_function(wrap_pyfunction!(dsl::prove_eta_id, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::search_identities, m)?)?;

    // Group 10: Mock Theta, Appell-Lerch & Bailey
    // 10a: Mock theta functions (20)
    m.add_function(wrap_pyfunction!(dsl::mock_theta_f3, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_phi3, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_psi3, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_chi3, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_omega3, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_nu3, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_rho3, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_f0_5, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_f1_5, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_cap_f0_5, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_cap_f1_5, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_phi0_5, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_phi1_5, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_psi0_5, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_psi1_5, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_chi0_5, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_chi1_5, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_cap_f0_7, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_cap_f1_7, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::mock_theta_cap_f2_7, m)?)?;
    // 10b: Appell-Lerch & universal mock theta (3)
    m.add_function(wrap_pyfunction!(dsl::appell_lerch_m, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::universal_mock_theta_g2, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::universal_mock_theta_g3, m)?)?;
    // 10c: Bailey machinery (4)
    m.add_function(wrap_pyfunction!(dsl::bailey_weak_lemma, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::bailey_apply_lemma, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::bailey_chain_fn, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::bailey_discover_fn, m)?)?;

    // Group 11: q-Gosper Algorithm
    m.add_function(wrap_pyfunction!(dsl::q_gosper_fn, m)?)?;

    // Group 12: Algorithmic Summation
    m.add_function(wrap_pyfunction!(dsl::q_zeilberger_fn, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::verify_wz_fn, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::q_petkovsek_fn, m)?)?;

    // Group 13: Identity Proving Extensions
    m.add_function(wrap_pyfunction!(dsl::prove_nonterminating_fn, m)?)?;
    m.add_function(wrap_pyfunction!(dsl::find_transformation_chain_fn, m)?)?;

    Ok(())
}
