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

use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tsv::VectorFileStats;

const CORPUS_SUFFIX: &str = "_corpus.tsv";
const QUERIES_SUFFIX: &str = "_queries.tsv";
const MANIFEST_SUFFIX: &str = "_manifest.json";
pub const EXPECTED_MANIFEST_VERSION: i64 = 1;
pub const ARTIFACT_LAYOUT_SINGLE_TSV: &str = "single_tsv";
pub const ARTIFACT_LAYOUT_CHUNKED: &str = "chunked";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkManifest {
    pub path: String,
    pub kind: String,
    pub start_row: i64,
    pub end_row: i64,
    pub rows: usize,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkedFileManifest {
    pub rows: usize,
    pub first_id: Option<i64>,
    pub last_id: Option<i64>,
    pub first_source_id: Option<String>,
    pub last_source_id: Option<String>,
    pub chunks: Vec<ChunkManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkedManifest {
    pub manifest_version: i64,
    pub artifact_layout: String,
    pub prefix: String,
    pub source_dataset: String,
    pub source_parquet: String,
    pub source_parquet_basename: String,
    pub source_parquet_shard_basenames: Vec<String>,
    pub id_column: String,
    pub vector_column: String,
    pub dimension: usize,
    pub chunk_rows: usize,
    pub selection_rule: Value,
    pub corpus: ChunkedFileManifest,
    pub queries: ChunkedFileManifest,
    pub generated_at_utc: String,
    pub generated_by: String,
}

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

pub fn is_chunked_manifest(manifest: &Value) -> bool {
    manifest.get("artifact_layout").and_then(Value::as_str) == Some(ARTIFACT_LAYOUT_CHUNKED)
}

pub fn parse_chunked_manifest(manifest: &Value) -> Result<ChunkedManifest> {
    let parsed: ChunkedManifest = serde_json::from_value(manifest.clone())
        .map_err(|e| eyre!("parsing chunked manifest: {e}"))?;
    validate_chunked_manifest(&parsed)?;
    Ok(parsed)
}

fn validate_chunked_manifest(manifest: &ChunkedManifest) -> Result<()> {
    if manifest.manifest_version != EXPECTED_MANIFEST_VERSION {
        return Err(eyre!(
            "manifest_version={} (expected {EXPECTED_MANIFEST_VERSION})",
            manifest.manifest_version
        ));
    }
    if manifest.artifact_layout != ARTIFACT_LAYOUT_CHUNKED {
        return Err(eyre!(
            "artifact_layout={:?} (expected {:?})",
            manifest.artifact_layout,
            ARTIFACT_LAYOUT_CHUNKED
        ));
    }
    if manifest.chunk_rows == 0 {
        return Err(eyre!("chunk_rows must be >= 1"));
    }
    validate_chunked_section(&manifest.corpus, "corpus")?;
    validate_chunked_section(&manifest.queries, "queries")?;
    Ok(())
}

fn validate_chunked_section(section: &ChunkedFileManifest, label: &str) -> Result<()> {
    let mut total_rows = 0usize;
    let mut expected_start: Option<i64> = None;
    for chunk in &section.chunks {
        if chunk.kind != label {
            return Err(eyre!(
                "{label} chunk {:?} has kind {:?}",
                chunk.path,
                chunk.kind
            ));
        }
        if Path::new(&chunk.path).is_absolute() {
            return Err(eyre!(
                "{label} chunk path {:?} must be relative",
                chunk.path
            ));
        }
        if chunk.rows == 0 {
            return Err(eyre!("{label} chunk {:?} has zero rows", chunk.path));
        }
        let expected_end = chunk.start_row + chunk.rows as i64 - 1;
        if chunk.end_row != expected_end {
            return Err(eyre!(
                "{label} chunk {:?} end_row={} (expected {expected_end})",
                chunk.path,
                chunk.end_row
            ));
        }
        if let Some(next_start) = expected_start {
            if chunk.start_row != next_start {
                return Err(eyre!(
                    "{label} chunk {:?} start_row={} (expected {next_start})",
                    chunk.path,
                    chunk.start_row
                ));
            }
        }
        expected_start = Some(chunk.end_row + 1);
        total_rows += chunk.rows;
    }
    if total_rows != section.rows {
        return Err(eyre!(
            "{label}.rows={} but chunks sum to {total_rows}",
            section.rows
        ));
    }
    if section.rows == 0 {
        if section.first_id.is_some() || section.last_id.is_some() {
            return Err(eyre!("{label} empty section must not set first_id/last_id"));
        }
        return Ok(());
    }
    let first_chunk = section
        .chunks
        .first()
        .ok_or_else(|| eyre!("{label}.rows={} but chunks is empty", section.rows))?;
    let last_chunk = section.chunks.last().unwrap();
    let first_id = section.first_id.unwrap_or(first_chunk.start_row);
    let last_id = section.last_id.unwrap_or(last_chunk.end_row);
    if first_id != first_chunk.start_row {
        return Err(eyre!(
            "{label}.first_id={} (expected {})",
            first_id,
            first_chunk.start_row
        ));
    }
    if last_id != last_chunk.end_row {
        return Err(eyre!(
            "{label}.last_id={} (expected {})",
            last_id,
            last_chunk.end_row
        ));
    }
    Ok(())
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

    #[test]
    fn parse_chunked_manifest_accepts_contiguous_relative_chunks() {
        let manifest = json!({
            "manifest_version": 1,
            "artifact_layout": "chunked",
            "prefix": "dbpedia_1m",
            "source_dataset": "dbpedia",
            "source_parquet": "/tmp/dbpedia",
            "source_parquet_basename": "dbpedia",
            "source_parquet_shard_basenames": ["part-0.parquet"],
            "id_column": "_id",
            "vector_column": "embedding",
            "dimension": 1536,
            "chunk_rows": 2,
            "selection_rule": {},
            "corpus": {
                "rows": 3,
                "first_id": 0,
                "last_id": 2,
                "first_source_id": "a",
                "last_source_id": "c",
                "chunks": [
                    {
                        "path": "corpus/corpus-00000.tsv",
                        "kind": "corpus",
                        "start_row": 0,
                        "end_row": 1,
                        "rows": 2,
                        "byte_length": 10,
                        "sha256": "a"
                    },
                    {
                        "path": "corpus/corpus-00001.tsv",
                        "kind": "corpus",
                        "start_row": 2,
                        "end_row": 2,
                        "rows": 1,
                        "byte_length": 5,
                        "sha256": "b"
                    }
                ]
            },
            "queries": {
                "rows": 1,
                "first_id": 3,
                "last_id": 3,
                "first_source_id": "d",
                "last_source_id": "d",
                "chunks": [
                    {
                        "path": "queries/queries-00000.tsv",
                        "kind": "queries",
                        "start_row": 3,
                        "end_row": 3,
                        "rows": 1,
                        "byte_length": 5,
                        "sha256": "c"
                    }
                ]
            },
            "generated_at_utc": "2026-04-26T00:00:00Z",
            "generated_by": "ecaz corpus prepare"
        });
        let parsed = parse_chunked_manifest(&manifest).unwrap();
        assert_eq!(parsed.chunk_rows, 2);
        assert_eq!(parsed.corpus.chunks.len(), 2);
    }

    #[test]
    fn parse_chunked_manifest_rejects_absolute_chunk_paths() {
        let manifest = json!({
            "manifest_version": 1,
            "artifact_layout": "chunked",
            "prefix": "x",
            "source_dataset": "dbpedia",
            "source_parquet": "/tmp/dbpedia",
            "source_parquet_basename": "dbpedia",
            "source_parquet_shard_basenames": ["part-0.parquet"],
            "id_column": "_id",
            "vector_column": "embedding",
            "dimension": 1536,
            "chunk_rows": 2,
            "selection_rule": {},
            "corpus": {
                "rows": 1,
                "first_id": 0,
                "last_id": 0,
                "first_source_id": "a",
                "last_source_id": "a",
                "chunks": [{
                    "path": "/abs/corpus-00000.tsv",
                    "kind": "corpus",
                    "start_row": 0,
                    "end_row": 0,
                    "rows": 1,
                    "byte_length": 1,
                    "sha256": "a"
                }]
            },
            "queries": {
                "rows": 1,
                "first_id": 1,
                "last_id": 1,
                "first_source_id": "b",
                "last_source_id": "b",
                "chunks": [{
                    "path": "queries/queries-00000.tsv",
                    "kind": "queries",
                    "start_row": 1,
                    "end_row": 1,
                    "rows": 1,
                    "byte_length": 1,
                    "sha256": "b"
                }]
            },
            "generated_at_utc": "2026-04-26T00:00:00Z",
            "generated_by": "ecaz corpus prepare"
        });
        let err = parse_chunked_manifest(&manifest).unwrap_err().to_string();
        assert!(err.contains("must be relative"), "err: {err}");
    }
}
