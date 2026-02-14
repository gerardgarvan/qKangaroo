//! Identity database: TOML-based searchable collection of verified q-series identities.
//!
//! Provides:
//! - [`IdentityEntry`]: a single identity with metadata, terms, and citation
//! - [`IdentityDatabase`]: collection of identities with search by tag, function, pattern
//! - TOML serialization/deserialization via serde

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use super::eta::EtaExpression;

/// One side of an identity (LHS or RHS), in eta-quotient form.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentitySide {
    /// The type of expression ("eta_quotient", "q_series", "theta", "jac")
    #[serde(rename = "type")]
    pub expr_type: String,
    /// Level N for eta quotients (optional for other types)
    pub level: Option<i64>,
    /// Eta quotient factors: maps delta -> r_delta
    /// Serialized as { "1" = 2, "5" = -3 } in TOML
    pub factors: Option<BTreeMap<String, i64>>,
    /// Free-form formula description (for display / non-eta types)
    pub formula: Option<String>,
}

/// Citation information for an identity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CitationInfo {
    pub author: Option<String>,
    pub year: Option<i64>,
    pub reference: Option<String>,
    pub doi: Option<String>,
}

/// Proof information for an identity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofInfo {
    /// Proof method: "valence_formula", "q_expansion", "bijective", "classical", "definition"
    pub method: Option<String>,
    /// Level used in the proof
    pub level: Option<i64>,
    /// Whether the identity has been verified
    pub verified: Option<bool>,
}

/// A single identity entry in the database.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityEntry {
    /// Unique identifier (e.g., "euler-pentagonal", "jacobi-triple-product")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Tags for categorization and search
    pub tags: Vec<String>,
    /// Functions involved (e.g., ["eta"], ["eta", "theta"], ["jac"])
    pub functions: Vec<String>,
    /// Left-hand side of the identity
    pub lhs: IdentitySide,
    /// Right-hand side of the identity
    pub rhs: IdentitySide,
    /// Proof information
    pub proof: Option<ProofInfo>,
    /// Citation information
    pub citation: Option<CitationInfo>,
}

impl IdentityEntry {
    /// Try to convert LHS to an EtaExpression.
    /// Returns None if the LHS is not of type "eta_quotient" or lacks required fields.
    pub fn lhs_as_eta(&self) -> Option<EtaExpression> {
        side_to_eta(&self.lhs)
    }

    /// Try to convert RHS to an EtaExpression.
    pub fn rhs_as_eta(&self) -> Option<EtaExpression> {
        side_to_eta(&self.rhs)
    }
}

fn side_to_eta(side: &IdentitySide) -> Option<EtaExpression> {
    if side.expr_type != "eta_quotient" {
        return None;
    }
    let level = side.level?;
    let factors_map = side.factors.as_ref()?;
    let mut factors = BTreeMap::new();
    for (k, v) in factors_map {
        let delta: i64 = k.parse().ok()?;
        factors.insert(delta, *v);
    }
    let pairs: Vec<(i64, i64)> = factors.into_iter().collect();
    Some(EtaExpression::from_factors(&pairs, level))
}

/// Wrapper for TOML top-level: contains a list of identity entries.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct IdentityFile {
    identity: Vec<IdentityEntry>,
}

/// Searchable collection of verified identities.
pub struct IdentityDatabase {
    entries: Vec<IdentityEntry>,
}

impl IdentityDatabase {
    /// Create an empty database.
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Load identities from a TOML string.
    pub fn load_from_toml(toml_str: &str) -> Result<Self, String> {
        let file: IdentityFile = toml::from_str(toml_str)
            .map_err(|e| format!("TOML parse error: {}", e))?;
        Ok(Self { entries: file.identity })
    }

    /// Load identities from a file path.
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        Self::load_from_toml(&content)
    }

    /// Add a new entry to the database.
    pub fn add(&mut self, entry: IdentityEntry) {
        self.entries.push(entry);
    }

    /// Number of identities in the database.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the database is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get a reference to all entries.
    pub fn entries(&self) -> &[IdentityEntry] {
        &self.entries
    }

    /// Find an entry by its unique id.
    pub fn get(&self, id: &str) -> Option<&IdentityEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Search for entries containing the given tag (case-insensitive).
    pub fn search_by_tag(&self, tag: &str) -> Vec<&IdentityEntry> {
        let tag_lower = tag.to_lowercase();
        self.entries.iter()
            .filter(|e| e.tags.iter().any(|t| t.to_lowercase() == tag_lower))
            .collect()
    }

    /// Search for entries whose `functions` list contains the given function (case-insensitive).
    pub fn search_by_function(&self, function: &str) -> Vec<&IdentityEntry> {
        let func_lower = function.to_lowercase();
        self.entries.iter()
            .filter(|e| e.functions.iter().any(|f| f.to_lowercase() == func_lower))
            .collect()
    }

    /// Search by pattern: case-insensitive substring match against id, name, tags, functions, and formula fields.
    pub fn search_by_pattern(&self, pattern: &str) -> Vec<&IdentityEntry> {
        let pattern_lower = pattern.to_lowercase();
        self.entries.iter()
            .filter(|e| {
                e.id.to_lowercase().contains(&pattern_lower)
                    || e.name.to_lowercase().contains(&pattern_lower)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&pattern_lower))
                    || e.functions.iter().any(|f| f.to_lowercase().contains(&pattern_lower))
                    || e.lhs.formula.as_ref().map_or(false, |f| f.to_lowercase().contains(&pattern_lower))
                    || e.rhs.formula.as_ref().map_or(false, |f| f.to_lowercase().contains(&pattern_lower))
            })
            .collect()
    }

    /// Serialize the database to a TOML string.
    pub fn to_toml(&self) -> Result<String, String> {
        let file = IdentityFile {
            identity: self.entries.clone(),
        };
        toml::to_string_pretty(&file)
            .map_err(|e| format!("TOML serialization error: {}", e))
    }
}
