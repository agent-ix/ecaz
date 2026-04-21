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
//! by many profiles when their `embedding_type` matches — today `ec_hnsw`,
//! `ec_ivf`, and `ec_diskann` all use `ecvector`, so one
//! `<prefix>_corpus` table supports all three.

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
    /// Whether KNN benchmark queries should encode `real[]` into the indexed
    /// embedding type before passing the ORDER BY probe to PostgreSQL.
    pub encode_scan_query: bool,
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
    /// Default sweep values used by `bench recall` / `bench latency` /
    /// `bench overhead` when the operator does not pass `--sweep`. Picked
    /// to cover the recall/latency Pareto frontier roughly evenly for
    /// this access method. Empty when the AM has no `ef_search_guc` to
    /// sweep.
    pub default_sweep: &'static [i32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SweepAxis {
    /// HNSW's `m` (graph degree per layer). Produces `<prefix>_m{N}_idx` names.
    M,
    /// DiskANN's `list_size` (L_search frontier width at scan time).
    ListSize,
    /// No native sweep axis; callers build a single index named `<prefix>_idx`.
    /// Reserved for future AMs without a tuning knob.
    #[allow(dead_code)]
    None,
}

impl IndexProfile {
    pub fn sweep_axis_is_m(&self) -> bool {
        matches!(self.sweep_axis, SweepAxis::M)
    }

    /// Return the subset of `reloptions` keys not in
    /// [`IndexProfile::known_reloptions`]. Used by commands to surface
    /// typos early (`--reloption graph_degre=48`) rather than letting
    /// Postgres reject them at `CREATE INDEX` time. Unknown keys still
    /// pass through verbatim — the caller decides whether to warn or
    /// stop.
    pub fn unknown_reloption_keys<'a>(
        &self,
        reloptions: &'a [(String, String)],
    ) -> Vec<&'a str> {
        reloptions
            .iter()
            .map(|(k, _)| k.as_str())
            .filter(|k| !self.known_reloptions.iter().any(|known| known == k))
            .collect()
    }
}

pub const EC_HNSW: IndexProfile = IndexProfile {
    name: "ec_hnsw",
    access_method: "ec_hnsw",
    operator_class: "ecvector_ip_ops",
    embedding_type: "ecvector",
    encoder_function: "encode_to_ecvector",
    encode_scan_query: true,
    ef_search_guc: Some("ec_hnsw.ef_search"),
    build_source_column: Some("source"),
    sweep_axis: SweepAxis::M,
    known_reloptions: &[
        "m",
        "ef_construction",
        "build_source_column",
        "storage_format",
    ],
    default_sweep: &[40, 64, 100, 128, 160, 200],
};

pub const EC_DISKANN: IndexProfile = IndexProfile {
    name: "ec_diskann",
    access_method: "ec_diskann",
    operator_class: "ecvector_diskann_ip_ops",
    embedding_type: "ecvector",
    encoder_function: "encode_to_ecvector",
    encode_scan_query: true,
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
    default_sweep: &[64, 128, 200, 400, 800],
};

pub const EC_IVF: IndexProfile = IndexProfile {
    name: "ec_ivf",
    access_method: "ec_ivf",
    operator_class: "ecvector_ip_ops",
    embedding_type: "ecvector",
    encoder_function: "encode_to_ecvector",
    encode_scan_query: false,
    ef_search_guc: Some("ec_ivf.nprobe"),
    build_source_column: None,
    sweep_axis: SweepAxis::None,
    known_reloptions: &[
        "nlists",
        "nprobe",
        "rerank_width",
        "training_sample_rows",
        "seed",
        "pq_group_size",
        "storage_format",
        "quantizer",
        "rerank",
    ],
    default_sweep: &[],
};

const REGISTRY: &[&IndexProfile] = &[&EC_HNSW, &EC_DISKANN, &EC_IVF];

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
        assert_eq!(names(), vec!["ec_diskann", "ec_hnsw", "ec_ivf"]);
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
            assert!(
                !p.operator_class.is_empty(),
                "profile {} missing opclass",
                p.name
            );
            assert!(
                !p.embedding_type.is_empty(),
                "profile {} missing embedding type",
                p.name
            );
            assert!(
                !p.encoder_function.is_empty(),
                "profile {} missing encoder",
                p.name
            );
        }
    }

    #[test]
    fn registry_has_no_duplicate_names() {
        let mut names: Vec<&str> = REGISTRY.iter().map(|p| p.name).collect();
        names.sort_unstable();
        let unique: std::collections::HashSet<&&str> = names.iter().collect();
        assert_eq!(
            names.len(),
            unique.len(),
            "duplicate profile name in REGISTRY"
        );
    }

    #[test]
    fn postgres_profiles_share_embedding_type_so_one_corpus_serves_all() {
        // README contract: a single <prefix>_corpus table supports all
        // Postgres AM indexes without re-encoding. If this ever
        // fails, the multi-corpus story in README.md needs updating.
        assert_eq!(EC_HNSW.embedding_type, EC_DISKANN.embedding_type);
        assert_eq!(EC_HNSW.encoder_function, EC_DISKANN.encoder_function);
        assert_eq!(EC_HNSW.embedding_type, EC_IVF.embedding_type);
        assert_eq!(EC_HNSW.encoder_function, EC_IVF.encoder_function);
    }

    #[test]
    fn ec_ivf_profile_uses_nprobe_and_raw_real_scan_query() {
        let p = &EC_IVF;
        assert_eq!(p.access_method, "ec_ivf");
        assert_eq!(p.ef_search_guc, Some("ec_ivf.nprobe"));
        assert_eq!(p.build_source_column, None);
        assert!(!p.encode_scan_query);
    }

    #[test]
    fn every_profile_has_nonempty_default_sweep() {
        // Every registered profile must ship a sensible default sweep so
        // `bench recall/latency/overhead --profile X` without an explicit
        // --sweep works out of the box. Future AMs without a sweep axis
        // will need explicit opt-out plus a CLI update.
        for p in REGISTRY {
            assert!(
                !p.default_sweep.is_empty(),
                "profile {} has empty default_sweep",
                p.name
            );
        }
    }

    #[test]
    fn default_sweep_is_strictly_ascending() {
        // comfy-table rows print in the order we sweep; an unsorted default
        // would confuse the reader scanning for the recall/latency knee.
        for p in REGISTRY {
            let s = p.default_sweep;
            for i in 1..s.len() {
                assert!(
                    s[i] > s[i - 1],
                    "profile {} default_sweep not strictly ascending: {:?}",
                    p.name,
                    s
                );
            }
        }
    }

    #[test]
    fn unknown_reloption_keys_returns_only_keys_outside_known_set() {
        let opts = vec![
            ("graph_degree".to_string(), "48".to_string()),
            ("graph_degre".to_string(), "48".to_string()), // typo
            ("alpha".to_string(), "1.2".to_string()),
            ("rerank_budge".to_string(), "64".to_string()), // typo
        ];
        let unknown = EC_DISKANN.unknown_reloption_keys(&opts);
        assert_eq!(unknown, vec!["graph_degre", "rerank_budge"]);
    }

    #[test]
    fn unknown_reloption_keys_empty_when_all_known() {
        let opts = vec![
            ("m".to_string(), "8".to_string()),
            ("ef_construction".to_string(), "128".to_string()),
        ];
        assert!(EC_HNSW.unknown_reloption_keys(&opts).is_empty());
    }

    #[test]
    fn unknown_reloption_keys_empty_when_no_reloptions() {
        assert!(EC_HNSW.unknown_reloption_keys(&[]).is_empty());
    }

    #[test]
    fn unknown_reloption_keys_is_case_sensitive() {
        // SQL is case-insensitive for identifiers, but pg_class.reloptions
        // stores canonical lowercase — surface GRAPH_DEGREE as unknown so
        // the operator sees a clean warning rather than a silent downcase.
        let opts = vec![("GRAPH_DEGREE".to_string(), "48".to_string())];
        assert_eq!(
            EC_DISKANN.unknown_reloption_keys(&opts),
            vec!["GRAPH_DEGREE"]
        );
    }

    #[test]
    fn validate_ident_rejects_injection_attempts() {
        assert!(validate_ident("foo; DROP TABLE x").is_err());
        assert!(validate_ident("1_leading_digit").is_err());
        assert!(validate_ident("").is_err());
        assert!(validate_ident("with-dash").is_err());
    }
}
