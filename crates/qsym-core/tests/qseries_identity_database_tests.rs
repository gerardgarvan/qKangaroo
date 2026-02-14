//! Tests for identity database TOML loading, searching, and round-trip serialization.

use qsym_core::qseries::identity::IdentityDatabase;

const CLASSICAL_TOML: &str = include_str!("../../../data/identities/classical_identities.toml");

#[test]
fn load_classical_identities() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).expect("Should parse TOML");
    assert!(db.len() >= 10, "Should have at least 10 identities, got {}", db.len());
}

#[test]
fn search_by_tag_classical() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let results = db.search_by_tag("classical");
    assert!(results.len() >= 5, "Should find multiple classical identities, got {}", results.len());
}

#[test]
fn search_by_tag_ramanujan() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let results = db.search_by_tag("ramanujan");
    assert!(results.len() >= 2, "Should find Ramanujan identities, got {}", results.len());
}

#[test]
fn search_by_tag_nonexistent() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let results = db.search_by_tag("nonexistent-tag-xyz");
    assert!(results.is_empty());
}

#[test]
fn search_by_function_eta() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let results = db.search_by_function("eta");
    assert!(results.len() >= 5, "Should find multiple eta-related identities, got {}", results.len());
}

#[test]
fn search_by_function_theta() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let results = db.search_by_function("theta");
    assert!(results.len() >= 2, "Should find theta-related identities, got {}", results.len());
}

#[test]
fn get_by_id() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let entry = db.get("euler-pentagonal");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().name, "Euler's Pentagonal Number Theorem");
}

#[test]
fn get_by_id_nonexistent() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let entry = db.get("does-not-exist");
    assert!(entry.is_none());
}

#[test]
fn search_by_pattern() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let results = db.search_by_pattern("partition");
    assert!(results.len() >= 2, "Should find partition-related identities, got {}", results.len());
}

#[test]
fn search_by_pattern_name() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let results = db.search_by_pattern("Pentagonal");
    assert_eq!(results.len(), 1, "Should find Euler's pentagonal by name");
    assert_eq!(results[0].id, "euler-pentagonal");
}

#[test]
fn round_trip_toml() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let toml_str = db.to_toml().expect("Should serialize to TOML");
    let db2 = IdentityDatabase::load_from_toml(&toml_str).expect("Should re-parse");
    assert_eq!(db.len(), db2.len());
    // Check a specific entry survived round-trip
    let e1 = db.get("euler-pentagonal").unwrap();
    let e2 = db2.get("euler-pentagonal").unwrap();
    assert_eq!(e1.name, e2.name);
    assert_eq!(e1.tags, e2.tags);
}

#[test]
fn round_trip_preserves_all_fields() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let toml_str = db.to_toml().expect("Should serialize to TOML");
    let db2 = IdentityDatabase::load_from_toml(&toml_str).expect("Should re-parse");

    // Check Ramanujan delta with eta_quotient LHS
    let e1 = db.get("ramanujan-delta").unwrap();
    let e2 = db2.get("ramanujan-delta").unwrap();
    assert_eq!(e1.id, e2.id);
    assert_eq!(e1.name, e2.name);
    assert_eq!(e1.functions, e2.functions);
    assert_eq!(e1.lhs.expr_type, e2.lhs.expr_type);
    assert_eq!(e1.lhs.level, e2.lhs.level);
    assert_eq!(e1.lhs.factors, e2.lhs.factors);
    assert_eq!(e1.citation.as_ref().unwrap().author, e2.citation.as_ref().unwrap().author);
    assert_eq!(e1.citation.as_ref().unwrap().year, e2.citation.as_ref().unwrap().year);
}

#[test]
fn identity_entry_to_eta_expression() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let entry = db.get("ramanujan-delta").unwrap();
    let eta = entry.lhs_as_eta();
    assert!(eta.is_some(), "Delta function LHS should convert to EtaExpression");
    let eta = eta.unwrap();
    assert_eq!(eta.level, 1);
    assert_eq!(*eta.factors.get(&1).unwrap(), 24);
}

#[test]
fn non_eta_entry_conversion_returns_none() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let entry = db.get("jacobi-triple-product").unwrap();
    // This has type "q_series", not "eta_quotient"
    let eta = entry.lhs_as_eta();
    assert!(eta.is_none(), "q_series type should not convert to EtaExpression");
}

#[test]
fn euler_pentagonal_eta_conversion() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let entry = db.get("euler-pentagonal").unwrap();
    let eta = entry.lhs_as_eta();
    assert!(eta.is_some());
    let eta = eta.unwrap();
    assert_eq!(eta.level, 1);
    assert_eq!(*eta.factors.get(&1).unwrap(), 1);
}

#[test]
fn database_new_and_add() {
    let mut db = IdentityDatabase::new();
    assert!(db.is_empty());
    assert_eq!(db.len(), 0);

    // Parse from TOML and add entries manually
    let source = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let entry = source.get("euler-pentagonal").unwrap().clone();
    db.add(entry);
    assert_eq!(db.len(), 1);
    assert!(!db.is_empty());
}

#[test]
fn search_case_insensitive() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let upper = db.search_by_tag("CLASSICAL");
    let lower = db.search_by_tag("classical");
    let mixed = db.search_by_tag("Classical");
    assert_eq!(upper.len(), lower.len());
    assert_eq!(upper.len(), mixed.len());
}

#[test]
fn search_by_function_jac() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let results = db.search_by_function("jac");
    assert!(results.len() >= 1, "Should find at least one jac identity, got {}", results.len());
}

#[test]
fn all_entries_have_required_fields() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    for entry in db.entries() {
        assert!(!entry.id.is_empty(), "Entry id must not be empty");
        assert!(!entry.name.is_empty(), "Entry name must not be empty");
        assert!(!entry.tags.is_empty(), "Entry tags must not be empty for {}", entry.id);
        assert!(!entry.functions.is_empty(), "Entry functions must not be empty for {}", entry.id);
        assert!(!entry.lhs.expr_type.is_empty(), "LHS expr_type must not be empty for {}", entry.id);
        assert!(!entry.rhs.expr_type.is_empty(), "RHS expr_type must not be empty for {}", entry.id);
    }
}

#[test]
fn entries_have_unique_ids() {
    let db = IdentityDatabase::load_from_toml(CLASSICAL_TOML).unwrap();
    let mut ids: Vec<&str> = db.entries().iter().map(|e| e.id.as_str()).collect();
    let original_len = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), original_len, "All identity IDs should be unique");
}
