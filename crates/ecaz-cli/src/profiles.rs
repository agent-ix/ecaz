//! Access-method profiles.
//!
//! Each profile captures what the loader, benchmarks, and comparator need to
//! know about an indexable access method: column type, encoder, `USING`
//! clause, operator class, per-scan tuning GUC, and which CLI sweep axis
//! applies. Adding a new access method means adding one entry to `REGISTRY`.
//!
//! # Drift
//!
//! This module hand-mirrors the extension's opclass + reloption surface
//! (`src/am/ec_hnsw/options.rs`, `src/am/ec_diskann/options.rs`). The
//! follow-up plan is to extract shared constants into a `ecaz-core` crate
//! so the CLI and the extension cannot drift. For v1 we accept the small
//! duplication; the constants are few and stable.
//!
//! See the README for the extraction plan.
//!
//! # Multiple corpora, one profile
//!
//! Profiles describe access methods, not corpora. One corpus can be indexed
//! by many profiles when their `embedding_type` matches — today `ec_hnsw`
//! and `ec_diskann` both use `ecvector`, so one `<prefix>_corpus` table
//! supports both.

use regex::Regex;
use std::sync::OnceLock;

/// Immutable description of one access-method profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexProfile {
    /// Short identifier used by `--profile` and index-name prefixes.
    pub name: &'static str,
    /// Postgres `USING <am>` clause value.
    pub access_method: &'static str,
    /// Operator class used in `CREATE INDEX`.
    pub operator_class: &'static str,
    /// Column type used for the indexed expression.
    pub embedding_type: &'static str,
    /// SQL function that encodes `real[]` → `embedding_type`.
    pub encoder_function: &'static str,
    /// Per-scan tuning GUC (None when the AM has no equivalent knob).
    pub ef_search_guc: Option<&'static str>,
    /// If the AM can read raw `real[]` at build time, the column name to
    /// use as the build source; None if the AM only reads the indexed
    /// expression.
    pub build_source_column: Option<&'static str>,
    /// Primary sweep axis exposed by CLI commands.
    pub sweep_axis: SweepAxis,
    /// Reloption keys the CLI knows about. Unknown keys are still accepted
    /// by `--reloption` passthrough — this set is for help text only.
    pub known_reloptions: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SweepAxis {
    /// HNSW's `m` (graph degree per layer). Produces `<prefix>_m{N}_idx` names.
    M,
    /// DiskANN's `list_size` (L_search frontier width at scan time).
    ListSize,
    /// No native sweep axis; callers build a single index named `<prefix>_idx`.
    None,
}

impl IndexProfile {
    pub fn sweep_axis_is_m(&self) -> bool {
        matches!(self.sweep_axis, SweepAxis::M)
    }
}

pub const EC_HNSW: IndexProfile = IndexProfile {
    name: "ec_hnsw",
    access_method: "ec_hnsw",
    operator_class: "ecvector_ip_ops",
    embedding_type: "ecvector",
    encoder_function: "encode_to_ecvector",
    ef_search_guc: Some("ec_hnsw.ef_search"),
    build_source_column: Some("source"),
    sweep_axis: SweepAxis::M,
    known_reloptions: &["m", "ef_construction", "build_source_column", "storage_format"],
};

pub const EC_DISKANN: IndexProfile = IndexProfile {
    name: "ec_diskann",
    access_method: "ec_diskann",
    operator_class: "ecvector_diskann_ip_ops",
    embedding_type: "ecvector",
    encoder_function: "encode_to_ecvector",
    ef_search_guc: Some("ec_diskann.list_size"),
    build_source_column: None,
    sweep_axis: SweepAxis::ListSize,
    known_reloptions: &[
        "graph_degree",
        "build_list_size",
        "list_size",
        "rerank_budget",
        "top_k",
        "alpha",
        "storage_format",
    ],
};

const REGISTRY: &[&IndexProfile] = &[&EC_HNSW, &EC_DISKANN];

pub fn resolve(name: &str) -> Option<&'static IndexProfile> {
    REGISTRY.iter().find(|p| p.name == name).copied()
}

pub fn names() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = REGISTRY.iter().map(|p| p.name).collect();
    v.sort_unstable();
    v
}

/// Validate a SQL identifier the CLI is going to interpolate into DDL
/// (prefix, table name, column name). Anything outside `[a-zA-Z_][a-zA-Z0-9_]*`
/// is rejected so we never need runtime SQL-injection defenses elsewhere.
pub fn validate_ident(name: &str) -> color_eyre::eyre::Result<()> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").expect("static regex"));
    if !re.is_match(name) {
        return Err(color_eyre::eyre::eyre!(
            "{:?} must match [a-zA-Z_][a-zA-Z0-9_]*",
            name
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_known_profiles() {
        assert_eq!(resolve("ec_hnsw").map(|p| p.access_method), Some("ec_hnsw"));
        assert_eq!(
            resolve("ec_diskann").map(|p| p.operator_class),
            Some("ecvector_diskann_ip_ops")
        );
        assert!(resolve("ec_bogus").is_none());
    }

    #[test]
    fn ec_hnsw_supports_m_sweep_and_build_source() {
        let p = &EC_HNSW;
        assert!(p.sweep_axis_is_m());
        assert_eq!(p.build_source_column, Some("source"));
    }

    #[test]
    fn ec_diskann_has_list_size_sweep_and_no_build_source() {
        let p = &EC_DISKANN;
        assert!(!p.sweep_axis_is_m());
        assert_eq!(p.build_source_column, None);
        assert_eq!(p.ef_search_guc, Some("ec_diskann.list_size"));
    }

    #[test]
    fn names_are_sorted_and_complete() {
        assert_eq!(names(), vec!["ec_diskann", "ec_hnsw"]);
    }

    #[test]
    fn validate_ident_accepts_safe_names() {
        assert!(validate_ident("dbpedia_10k").is_ok());
        assert!(validate_ident("_under").is_ok());
        assert!(validate_ident("A1").is_ok());
    }

    #[test]
    fn every_profile_has_nonempty_access_method_and_opclass() {
        for p in REGISTRY {
            assert!(!p.name.is_empty(), "profile with empty name");
            assert!(!p.access_method.is_empty(), "profile {} missing AM", p.name);
            assert!(!p.operator_class.is_empty(), "profile {} missing opclass", p.name);
            assert!(!p.embedding_type.is_empty(), "profile {} missing embedding type", p.name);
            assert!(!p.encoder_function.is_empty(), "profile {} missing encoder", p.name);
        }
    }

    #[test]
    fn registry_has_no_duplicate_names() {
        let mut names: Vec<&str> = REGISTRY.iter().map(|p| p.name).collect();
        names.sort_unstable();
        let unique: std::collections::HashSet<&&str> = names.iter().collect();
        assert_eq!(names.len(), unique.len(), "duplicate profile name in REGISTRY");
    }

    #[test]
    fn hnsw_and_diskann_share_embedding_type_so_one_corpus_serves_both() {
        // README contract: a single <prefix>_corpus table supports both
        // ec_hnsw and ec_diskann indexes without re-encoding. If this ever
        // fails, the multi-corpus story in README.md needs updating.
        assert_eq!(EC_HNSW.embedding_type, EC_DISKANN.embedding_type);
        assert_eq!(EC_HNSW.encoder_function, EC_DISKANN.encoder_function);
    }

    #[test]
    fn validate_ident_rejects_injection_attempts() {
        assert!(validate_ident("foo; DROP TABLE x").is_err());
        assert!(validate_ident("1_leading_digit").is_err());
        assert!(validate_ident("").is_err());
        assert!(validate_ident("with-dash").is_err());
    }
}
