//! Manifest verification for local-file corpus fixtures.
//!
//! A "manifest" is a sibling `<basename>_manifest.json` next to a
//! `<basename>_corpus.tsv` / `<basename>_queries.tsv` pair. It pins dataset
//! identity (sha256, row count, first/last id) so a reviewer running the
//! same fixture on a different machine can prove they staged exactly the
//! same bytes the author did.
//!
//! The structure mirrors the Python loader's `_verify_manifest` one-for-one.
//! All comparisons are intentionally mechanical — no heuristics, no "mostly
//! matches" — so a corrupted fixture fails fast with a diff-style message
//! instead of quietly producing bogus recall numbers.
//!
//! The module is pure logic: it takes already-computed `VectorFileStats`
//! and a parsed `serde_json::Value`. The caller is responsible for reading
//! the manifest from disk and inspecting the TSV files. This keeps the
//! module testable without touching the filesystem.
//!
//! # Portability fields
//!
//! `source_parquet_basename` and `source_parquet_shard_basenames` are
//! *optional* additive fields that pin dataset identity independent of
//! the local filesystem. When present they must be portable basenames
//! (no `/` or `\`) — the absolute `source_parquet` field, which does
//! contain a local path, is deliberately ignored here because it isn't
//! portable across machines.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::tsv::VectorFileStats;

const CORPUS_SUFFIX: &str = "_corpus.tsv";
const QUERIES_SUFFIX: &str = "_queries.tsv";
const MANIFEST_SUFFIX: &str = "_manifest.json";
pub const EXPECTED_MANIFEST_VERSION: i64 = 1;

/// If the corpus/queries pair follows the `<basename>_{corpus,queries}.tsv`
/// convention with a common `<basename>`, return the sibling manifest path.
/// Otherwise return `None` — the caller falls back to "no manifest" unless
/// `--manifest-file` was passed explicitly.
pub fn derive_manifest_path(corpus_file: &Path, queries_file: &Path) -> Option<PathBuf> {
    let corpus = corpus_file.to_str()?;
    let queries = queries_file.to_str()?;
    let corpus_base = corpus.strip_suffix(CORPUS_SUFFIX)?;
    let queries_base = queries.strip_suffix(QUERIES_SUFFIX)?;
    if corpus_base != queries_base {
        return None;
    }
    Some(PathBuf::from(format!("{corpus_base}{MANIFEST_SUFFIX}")))
}

/// One mismatch between the manifest and the on-disk fixture. The CLI
/// collects every problem before reporting so a reviewer sees the full
/// diff instead of fixing them one at a time.
#[derive(Debug, PartialEq, Eq)]
pub struct Problem(pub String);

/// Compare `manifest` against the header fields the loader knows (the
/// prefix and dimension it was asked to load) plus the inspected
/// `VectorFileStats` for each TSV file.
///
/// Returns the full list of problems. An empty vec means the manifest
/// matches; the caller decides whether a non-empty list is fatal based on
/// `--allow-manifest-mismatch`.
pub fn verify(
    manifest: &Value,
    prefix: &str,
    corpus_path: &Path,
    queries_path: &Path,
    dim: usize,
    corpus_stats: &VectorFileStats,
    queries_stats: &VectorFileStats,
) -> Vec<Problem> {
    let mut problems = Vec::new();

    // Header fields: manifest_version, prefix, dimension.
    match manifest.get("manifest_version").and_then(Value::as_i64) {
        Some(v) if v == EXPECTED_MANIFEST_VERSION => {}
        other => problems.push(Problem(format!(
            "manifest_version={} (expected {EXPECTED_MANIFEST_VERSION})",
            json_repr_or_null(manifest.get("manifest_version"), other.is_none())
        ))),
    }
    match manifest.get("prefix").and_then(Value::as_str) {
        Some(v) if v == prefix => {}
        _ => problems.push(Problem(format!(
            "prefix={} (expected {prefix:?})",
            json_repr(manifest.get("prefix"))
        ))),
    }
    match manifest.get("dimension").and_then(Value::as_u64) {
        Some(v) if v as usize == dim => {}
        _ => problems.push(Problem(format!(
            "dimension={} (expected {dim})",
            json_repr(manifest.get("dimension"))
        ))),
    }

    // Optional portability fields: must be string / list-of-strings if present,
    // and basenames must not contain path separators.
    if let Some(value) = manifest.get("source_parquet_basename") {
        match value.as_str() {
            Some(s) if !s.contains('/') && !s.contains('\\') => {}
            Some(s) => problems.push(Problem(format!(
                "source_parquet_basename={s:?} (expected portable basename, not a path)"
            ))),
            None => problems.push(Problem(format!(
                "source_parquet_basename={value} (expected string)"
            ))),
        }
    }
    if let Some(value) = manifest.get("source_parquet_shard_basenames") {
        match value.as_array() {
            Some(arr) if arr.iter().all(|v| v.is_string()) => {
                for shard in arr {
                    let s = shard.as_str().expect("checked by all()");
                    if s.contains('/') || s.contains('\\') {
                        problems.push(Problem(format!(
                            "source_parquet_shard_basenames entry {s:?} (expected portable basename, not a path)"
                        )));
                        break;
                    }
                }
            }
            _ => problems.push(Problem(format!(
                "source_parquet_shard_basenames={value} (expected list of strings)"
            ))),
        }
    }

    // Per-file sections: file basename, rows, sha256, first_id, last_id.
    for (label, path, stats) in [
        ("corpus", corpus_path, corpus_stats),
        ("queries", queries_path, queries_stats),
    ] {
        let section = manifest
            .get(label)
            .cloned()
            .unwrap_or(Value::Object(Default::default()));
        let on_disk_basename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if let Some(expected) = section.get("file").and_then(Value::as_str) {
            if expected != on_disk_basename {
                problems.push(Problem(format!(
                    "{label}.file={expected:?} (expected {on_disk_basename:?})"
                )));
            }
        }
        match section.get("rows").and_then(Value::as_u64) {
            Some(v) if v as usize == stats.rows => {}
            _ => problems.push(Problem(format!(
                "{label}.rows={} (expected {})",
                json_repr(section.get("rows")),
                stats.rows
            ))),
        }
        match section.get("sha256").and_then(Value::as_str) {
            Some(v) if v == stats.sha256_hex => {}
            _ => problems.push(Problem(format!(
                "{label}.sha256={} (expected {:?})",
                json_repr(section.get("sha256")),
                stats.sha256_hex
            ))),
        }
        match (
            section.get("first_id").and_then(Value::as_i64),
            stats.first_id,
        ) {
            (Some(v), Some(want)) if v == want => {}
            (None, None) => {}
            _ => problems.push(Problem(format!(
                "{label}.first_id={} (expected {})",
                json_repr(section.get("first_id")),
                json_repr_opt(stats.first_id)
            ))),
        }
        match (
            section.get("last_id").and_then(Value::as_i64),
            stats.last_id,
        ) {
            (Some(v), Some(want)) if v == want => {}
            (None, None) => {}
            _ => problems.push(Problem(format!(
                "{label}.last_id={} (expected {})",
                json_repr(section.get("last_id")),
                json_repr_opt(stats.last_id)
            ))),
        }
    }

    problems
}

fn json_repr(value: Option<&Value>) -> String {
    match value {
        None => "null".into(),
        Some(v) => v.to_string(),
    }
}

fn json_repr_or_null(value: Option<&Value>, _is_missing: bool) -> String {
    json_repr(value)
}

fn json_repr_opt<T: std::fmt::Display>(value: Option<T>) -> String {
    match value {
        None => "null".into(),
        Some(v) => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    fn stats(rows: usize, sha: &str, first: Option<i64>, last: Option<i64>) -> VectorFileStats {
        VectorFileStats {
            rows,
            sha256_hex: sha.to_owned(),
            first_id: first,
            last_id: last,
        }
    }

    fn good_manifest() -> Value {
        json!({
            "manifest_version": 1,
            "prefix": "dbpedia_10k",
            "dimension": 1536,
            "corpus": {
                "file": "dbpedia_10k_corpus.tsv",
                "rows": 10_000,
                "sha256": "c".repeat(64),
                "first_id": 0,
                "last_id": 9_999,
            },
            "queries": {
                "file": "dbpedia_10k_queries.tsv",
                "rows": 200,
                "sha256": "q".repeat(64),
                "first_id": 0,
                "last_id": 199,
            }
        })
    }

    fn good_stats() -> (VectorFileStats, VectorFileStats) {
        (
            stats(10_000, &"c".repeat(64), Some(0), Some(9_999)),
            stats(200, &"q".repeat(64), Some(0), Some(199)),
        )
    }

    #[test]
    fn derive_returns_sibling_when_basenames_match() {
        let got = derive_manifest_path(
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
        );
        assert_eq!(got, Some(PathBuf::from("/data/dbpedia_10k_manifest.json")));
    }

    #[test]
    fn derive_returns_none_for_mismatched_basenames() {
        let got = derive_manifest_path(
            Path::new("/data/alpha_corpus.tsv"),
            Path::new("/data/beta_queries.tsv"),
        );
        assert_eq!(got, None);
    }

    #[test]
    fn derive_returns_none_when_suffixes_dont_match() {
        assert_eq!(
            derive_manifest_path(
                Path::new("/data/foo.tsv"),
                Path::new("/data/foo_queries.tsv"),
            ),
            None
        );
    }

    #[test]
    fn verify_passes_on_matching_manifest() {
        let (cs, qs) = good_stats();
        let problems = verify(
            &good_manifest(),
            "dbpedia_10k",
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
            1536,
            &cs,
            &qs,
        );
        assert!(problems.is_empty(), "unexpected problems: {problems:?}");
    }

    #[test]
    fn verify_flags_wrong_manifest_version() {
        let mut m = good_manifest();
        m["manifest_version"] = json!(2);
        let (cs, qs) = good_stats();
        let p = verify(
            &m,
            "dbpedia_10k",
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
            1536,
            &cs,
            &qs,
        );
        assert!(p.iter().any(|Problem(s)| s.contains("manifest_version")));
    }

    #[test]
    fn verify_flags_wrong_prefix_and_dim() {
        let (cs, qs) = good_stats();
        let p = verify(
            &good_manifest(),
            "other_prefix",
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
            768,
            &cs,
            &qs,
        );
        assert!(p.iter().any(|Problem(s)| s.contains("prefix=")));
        assert!(p.iter().any(|Problem(s)| s.contains("dimension=")));
    }

    #[test]
    fn verify_flags_sha_row_count_and_id_drift() {
        let (_cs, qs) = good_stats();
        let cs = stats(9_999, &"d".repeat(64), Some(1), Some(9_998));
        let p = verify(
            &good_manifest(),
            "dbpedia_10k",
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
            1536,
            &cs,
            &qs,
        );
        assert!(p.iter().any(|Problem(s)| s.contains("corpus.rows=")));
        assert!(p.iter().any(|Problem(s)| s.contains("corpus.sha256=")));
        assert!(p.iter().any(|Problem(s)| s.contains("corpus.first_id=")));
        assert!(p.iter().any(|Problem(s)| s.contains("corpus.last_id=")));
    }

    #[test]
    fn verify_flags_wrong_file_basename() {
        let mut m = good_manifest();
        m["corpus"]["file"] = json!("wrong_name_corpus.tsv");
        let (cs, qs) = good_stats();
        let p = verify(
            &m,
            "dbpedia_10k",
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
            1536,
            &cs,
            &qs,
        );
        assert!(p.iter().any(|Problem(s)| s.contains("corpus.file=")));
    }

    #[test]
    fn verify_accepts_missing_optional_portability_fields() {
        // `source_parquet_*` fields are additive; absent = no problem.
        let (cs, qs) = good_stats();
        assert!(verify(
            &good_manifest(),
            "dbpedia_10k",
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
            1536,
            &cs,
            &qs,
        )
        .is_empty());
    }

    #[test]
    fn verify_flags_non_portable_basename_with_slash() {
        let mut m = good_manifest();
        m["source_parquet_basename"] = json!("subdir/file.parquet");
        let (cs, qs) = good_stats();
        let p = verify(
            &m,
            "dbpedia_10k",
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
            1536,
            &cs,
            &qs,
        );
        assert!(p
            .iter()
            .any(|Problem(s)| s.contains("source_parquet_basename")));
    }

    #[test]
    fn verify_flags_non_string_basename() {
        let mut m = good_manifest();
        m["source_parquet_basename"] = json!(42);
        let (cs, qs) = good_stats();
        let p = verify(
            &m,
            "dbpedia_10k",
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
            1536,
            &cs,
            &qs,
        );
        assert!(p.iter().any(|Problem(s)| s.contains("expected string")));
    }

    #[test]
    fn verify_flags_shard_list_with_non_strings() {
        let mut m = good_manifest();
        m["source_parquet_shard_basenames"] = json!(["ok.parquet", 3]);
        let (cs, qs) = good_stats();
        let p = verify(
            &m,
            "dbpedia_10k",
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
            1536,
            &cs,
            &qs,
        );
        assert!(p
            .iter()
            .any(|Problem(s)| s.contains("expected list of strings")));
    }

    #[test]
    fn verify_flags_shard_list_with_path_separator() {
        let mut m = good_manifest();
        m["source_parquet_shard_basenames"] = json!(["ok.parquet", "dir/nested.parquet"]);
        let (cs, qs) = good_stats();
        let p = verify(
            &m,
            "dbpedia_10k",
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
            1536,
            &cs,
            &qs,
        );
        assert!(p
            .iter()
            .any(|Problem(s)| s.contains("source_parquet_shard_basenames entry")));
    }

    #[test]
    fn verify_accepts_shard_list_of_portable_basenames() {
        let mut m = good_manifest();
        m["source_parquet_shard_basenames"] = json!(["a.parquet", "b.parquet"]);
        let (cs, qs) = good_stats();
        assert!(verify(
            &m,
            "dbpedia_10k",
            Path::new("/data/dbpedia_10k_corpus.tsv"),
            Path::new("/data/dbpedia_10k_queries.tsv"),
            1536,
            &cs,
            &qs,
        )
        .is_empty());
    }
}
