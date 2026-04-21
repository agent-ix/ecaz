//! `ecaz corpus prepare` — parquet → canonical TSV + manifest.
//!
//! Ports `scripts/qdrant_dbpedia_to_tsv.py`. Given a parquet release (file
//! or directory of `*.parquet`) and a named subset profile, picks the
//! reproducible first-N / next-M rows under ascending lexicographic id
//! order and emits `<prefix>_{corpus,queries}.tsv` + `<prefix>_manifest.json`.
//!
//! # Two-pass selection
//!
//! Pass 1 streams the id column and maintains a size-K max-heap of the K
//! smallest ids seen (K = corpus_rows + query_rows). The result is the
//! canonical sorted-id prefix without materializing the full dataset.
//!
//! Pass 2 streams id + vector columns and materializes only the selected
//! rows into an `id -> Vec<f32>` map.
//!
//! # Purity boundary
//!
//! Profile table, column resolution, canonical formatters, selection
//! splitting, manifest JSON construction, and the streaming heap update
//! are pure functions with unit tests. Parquet I/O is a thin shell on top.

use arrow_array::{
    cast::AsArray, types::Float32Type, types::Float64Type, Array, LargeStringArray, StringArray,
};
use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ProjectionMask;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub const DEFAULT_SOURCE_DATASET: &str =
    "Qdrant dbpedia-entities-openai3-text-embedding-3-large-1536-1M";
pub const DEFAULT_DIM: usize = 1536;
pub const ID_COLUMN_CANDIDATES: &[&str] = &["id", "_id"];
pub const VECTOR_COLUMN_CANDIDATES: &[&str] = &[
    "embedding",
    "vector",
    "values",
    "openai",
    "text_embedding",
    "text-embedding-3-large-1536-embedding",
];

/// A named subset-selection recipe, matching the Python PROFILES table
/// one-to-one so regenerated fixtures stay byte-identical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubsetProfile {
    pub prefix: &'static str,
    pub corpus_rows: usize,
    pub query_rows: usize,
}

impl SubsetProfile {
    pub fn query_start(&self) -> usize {
        self.corpus_rows
    }
    pub fn needed_rows(&self) -> usize {
        self.corpus_rows + self.query_rows
    }
}

pub const PROFILES: &[SubsetProfile] = &[
    SubsetProfile {
        prefix: "ec_hnsw_real_50k",
        corpus_rows: 50_000,
        query_rows: 1_000,
    },
    SubsetProfile {
        prefix: "ec_hnsw_real_10k",
        corpus_rows: 10_000,
        query_rows: 200,
    },
    SubsetProfile {
        prefix: "ec_hnsw_real_ann_benchmarks_anchor",
        corpus_rows: 990_000,
        query_rows: 10_000,
    },
];

pub fn resolve_profile(name: &str) -> Option<&'static SubsetProfile> {
    PROFILES.iter().find(|p| p.prefix == name)
}

#[derive(Args, Debug)]
pub struct PrepareArgs {
    /// Canonical subset profile to emit (one of the PROFILES entries).
    #[arg(long)]
    pub profile: String,
    /// Path to the parquet file or directory containing `*.parquet` shards.
    #[arg(long)]
    pub parquet: PathBuf,
    /// Directory to write the TSVs and manifest into (created if missing).
    #[arg(long)]
    pub output_dir: PathBuf,
    /// Override the id column name (auto-inferred from ID_COLUMN_CANDIDATES).
    #[arg(long)]
    pub id_column: Option<String>,
    /// Override the vector column name (auto-inferred from VECTOR_COLUMN_CANDIDATES).
    #[arg(long)]
    pub vector_column: Option<String>,
    /// Expected embedding dimensionality.
    #[arg(long, default_value_t = DEFAULT_DIM)]
    pub dim: usize,
    /// Human-readable dataset label recorded in the manifest.
    #[arg(long, default_value_t = DEFAULT_SOURCE_DATASET.to_string())]
    pub source_dataset: String,
}

pub async fn run(_database: &str, args: PrepareArgs) -> Result<()> {
    let profile = resolve_profile(&args.profile).ok_or_else(|| {
        let names: Vec<&str> = PROFILES.iter().map(|p| p.prefix).collect();
        eyre!(
            "unknown profile {:?}; try {}",
            args.profile,
            names.join(", ")
        )
    })?;
    if args.dim == 0 {
        return Err(eyre!("--dim must be >= 1"));
    }
    std::fs::create_dir_all(&args.output_dir)
        .wrap_err_with(|| format!("creating {}", args.output_dir.display()))?;

    let parquet_files = resolve_parquet_files(&args.parquet)?;
    let schema_names = read_schema_names(&parquet_files[0])?;
    let id_column = resolve_id_column(&schema_names, args.id_column.as_deref())?;
    let vector_column = resolve_vector_column(&schema_names, args.vector_column.as_deref())?;

    eprintln!(
        "[prepare] pass 1: scanning {} shard(s) for sorted-id prefix (K={})",
        parquet_files.len(),
        profile.needed_rows()
    );
    let sorted_ids = load_sorted_ids(&parquet_files, &id_column, profile.needed_rows())?;
    let (corpus_source_ids, query_source_ids) = split_sorted_ids(&sorted_ids, profile);

    eprintln!(
        "[prepare] pass 2: materializing {} selected vectors",
        corpus_source_ids.len() + query_source_ids.len()
    );
    let mut selected: HashSet<String> =
        HashSet::with_capacity(corpus_source_ids.len() + query_source_ids.len());
    selected.extend(corpus_source_ids.iter().cloned());
    selected.extend(query_source_ids.iter().cloned());
    let rows_by_id = load_selected_rows(
        &parquet_files,
        &id_column,
        &vector_column,
        &selected,
        args.dim,
    )?;

    let corpus_path = args.output_dir.join(format!("{}_corpus.tsv", profile.prefix));
    let queries_path = args.output_dir.join(format!("{}_queries.tsv", profile.prefix));
    let manifest_path = args
        .output_dir
        .join(format!("{}_manifest.json", profile.prefix));

    let corpus_entries: Vec<(i64, String)> = corpus_source_ids
        .iter()
        .enumerate()
        .map(|(i, s)| (i as i64, s.clone()))
        .collect();
    let query_entries: Vec<(i64, String)> = query_source_ids
        .iter()
        .enumerate()
        .map(|(i, s)| ((profile.query_start() + i) as i64, s.clone()))
        .collect();
    let corpus_manifest = write_tsv(&corpus_path, &corpus_entries, &rows_by_id)?;
    let query_manifest = write_tsv(&queries_path, &query_entries, &rows_by_id)?;

    let source_parquet_abs = std::fs::canonicalize(&args.parquet)
        .unwrap_or_else(|_| args.parquet.clone())
        .to_string_lossy()
        .into_owned();
    let shard_basenames: Vec<String> = {
        let mut v: Vec<String> = parquet_files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    };
    let manifest = build_manifest_json(
        profile,
        &source_parquet_abs,
        source_parquet_basename(&args.parquet),
        &shard_basenames,
        &args.source_dataset,
        &id_column,
        &vector_column,
        args.dim,
        &corpus_manifest,
        &query_manifest,
        &chrono::Utc::now().to_rfc3339(),
    );
    let mut handle = File::create(&manifest_path)
        .wrap_err_with(|| format!("creating {}", manifest_path.display()))?;
    serde_json::to_writer_pretty(&mut handle, &manifest)?;
    handle.write_all(b"\n")?;

    eprintln!("[prepare] wrote {}", corpus_path.display());
    eprintln!("[prepare] wrote {}", queries_path.display());
    eprintln!("[prepare] wrote {}", manifest_path.display());
    eprintln!(
        "[prepare] profile={} corpus_rows={} query_rows={} sort_key='{} ascending lexicographic'",
        profile.prefix, profile.corpus_rows, profile.query_rows, id_column
    );
    Ok(())
}

/// Enumerate the input path into an ordered list of parquet files. Accepts
/// either a single `*.parquet` file or a directory of shards.
pub fn resolve_parquet_files(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if path.is_dir() {
        let mut files: Vec<PathBuf> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("parquet"))
            .collect();
        files.sort();
        if !files.is_empty() {
            return Ok(files);
        }
    }
    Err(eyre!("no parquet files found at {}", path.display()))
}

fn read_schema_names(file: &Path) -> Result<Vec<String>> {
    let f = File::open(file).wrap_err_with(|| format!("opening {}", file.display()))?;
    let builder =
        ParquetRecordBatchReaderBuilder::try_new(f).wrap_err("reading parquet metadata")?;
    Ok(builder
        .schema()
        .fields()
        .iter()
        .map(|f| f.name().clone())
        .collect())
}

/// Pick the vector column: caller override, or the unique match against
/// `VECTOR_COLUMN_CANDIDATES`, or the unique non-id column as a fallback.
pub fn resolve_vector_column(schema_names: &[String], requested: Option<&str>) -> Result<String> {
    if let Some(name) = requested {
        if !schema_names.iter().any(|n| n == name) {
            return Err(eyre!(
                "vector column {name:?} not found in parquet schema {schema_names:?}"
            ));
        }
        return Ok(name.to_owned());
    }
    let matches: Vec<String> = VECTOR_COLUMN_CANDIDATES
        .iter()
        .filter(|c| schema_names.iter().any(|n| n == *c))
        .map(|s| s.to_string())
        .collect();
    match matches.len() {
        1 => Ok(matches.into_iter().next().unwrap()),
        0 => {
            let fallback: Vec<String> = schema_names
                .iter()
                .filter(|n| !ID_COLUMN_CANDIDATES.contains(&n.as_str()))
                .cloned()
                .collect();
            if fallback.len() == 1 {
                Ok(fallback.into_iter().next().unwrap())
            } else {
                Err(eyre!(
                    "could not infer vector column from schema {schema_names:?}; \
                     pass --vector-column explicitly"
                ))
            }
        }
        _ => Err(eyre!(
            "multiple plausible vector columns found {matches:?}; pass --vector-column explicitly"
        )),
    }
}

/// Pick the id column: caller override, or the unique match against
/// `ID_COLUMN_CANDIDATES`.
pub fn resolve_id_column(schema_names: &[String], requested: Option<&str>) -> Result<String> {
    if let Some(name) = requested {
        if !schema_names.iter().any(|n| n == name) {
            return Err(eyre!(
                "id column {name:?} not found in parquet schema {schema_names:?}"
            ));
        }
        return Ok(name.to_owned());
    }
    let matches: Vec<String> = ID_COLUMN_CANDIDATES
        .iter()
        .filter(|c| schema_names.iter().any(|n| n == *c))
        .map(|s| s.to_string())
        .collect();
    match matches.len() {
        1 => Ok(matches.into_iter().next().unwrap()),
        0 => Err(eyre!(
            "could not infer id column from schema {schema_names:?}; \
             pass --id-column explicitly"
        )),
        _ => Err(eyre!(
            "multiple plausible id columns found {matches:?}; pass --id-column explicitly"
        )),
    }
}

/// Update a size-K max-heap of candidate ids with one new id. Returns the
/// heap after update. Exposed as a pure function so the selection
/// invariant (K smallest ids seen so far) can be unit-tested without
/// opening a parquet file.
pub fn push_smallest(heap: &mut BinaryHeap<String>, id: String, k: usize) {
    if heap.len() < k {
        heap.push(id);
        return;
    }
    if let Some(top) = heap.peek() {
        if id < *top {
            heap.pop();
            heap.push(id);
        }
    }
}

fn load_sorted_ids(
    parquet_files: &[PathBuf],
    id_column: &str,
    needed_rows: usize,
) -> Result<Vec<String>> {
    let mut heap: BinaryHeap<String> = BinaryHeap::with_capacity(needed_rows);
    let mut total_rows: usize = 0;
    for file in parquet_files {
        let f = File::open(file)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(f)?;
        let schema = builder.schema();
        let col_idx = schema
            .index_of(id_column)
            .map_err(|_| eyre!("id column {id_column:?} missing from shard {}", file.display()))?;
        let mask = ProjectionMask::roots(builder.parquet_schema(), [col_idx]);
        let reader = builder.with_projection(mask).with_batch_size(16_384).build()?;
        for batch in reader {
            let batch = batch?;
            let array = batch.column(0);
            for i in 0..array.len() {
                total_rows += 1;
                let id = read_string_at(array, i)
                    .ok_or_else(|| eyre!("id column {id_column:?} contains a null at row {i}"))?;
                push_smallest(&mut heap, id, needed_rows);
            }
        }
    }
    if total_rows < needed_rows {
        return Err(eyre!(
            "parquet only has {total_rows} rows, but {needed_rows} are required"
        ));
    }
    let mut sorted: Vec<String> = heap.into_sorted_vec();
    let before = sorted.len();
    sorted.dedup();
    if sorted.len() != before {
        return Err(eyre!(
            "duplicate ids detected within the selected canonical prefix"
        ));
    }
    Ok(sorted)
}

fn load_selected_rows(
    parquet_files: &[PathBuf],
    id_column: &str,
    vector_column: &str,
    selected: &HashSet<String>,
    dim: usize,
) -> Result<HashMap<String, Vec<f32>>> {
    let mut rows: HashMap<String, Vec<f32>> = HashMap::with_capacity(selected.len());
    for file in parquet_files {
        if rows.len() == selected.len() {
            break;
        }
        let f = File::open(file)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(f)?;
        let schema = builder.schema();
        let id_idx = schema.index_of(id_column).map_err(|_| {
            eyre!("id column {id_column:?} missing from shard {}", file.display())
        })?;
        let vec_idx = schema.index_of(vector_column).map_err(|_| {
            eyre!(
                "vector column {vector_column:?} missing from shard {}",
                file.display()
            )
        })?;
        let mask = ProjectionMask::roots(builder.parquet_schema(), [id_idx, vec_idx]);
        let reader = builder.with_projection(mask).with_batch_size(4_096).build()?;
        for batch in reader {
            let batch = batch?;
            let ids = batch.column(0);
            let vecs = batch.column(1);
            for i in 0..ids.len() {
                let id = match read_string_at(ids, i) {
                    Some(s) => s,
                    None => continue,
                };
                if !selected.contains(&id) {
                    continue;
                }
                if rows.contains_key(&id) {
                    return Err(eyre!(
                        "duplicate selected id {id} encountered during parquet scan"
                    ));
                }
                let v = read_vector_at(vecs, i, dim, vector_column, &id)?;
                rows.insert(id, v);
                if rows.len() == selected.len() {
                    return Ok(rows);
                }
            }
        }
    }
    if rows.len() != selected.len() {
        let missing = selected.len() - rows.len();
        return Err(eyre!(
            "failed to recover {missing} selected ids from parquet scan"
        ));
    }
    Ok(rows)
}

fn read_string_at(array: &dyn Array, idx: usize) -> Option<String> {
    if array.is_null(idx) {
        return None;
    }
    if let Some(a) = array.as_any().downcast_ref::<StringArray>() {
        return Some(a.value(idx).to_owned());
    }
    if let Some(a) = array.as_any().downcast_ref::<LargeStringArray>() {
        return Some(a.value(idx).to_owned());
    }
    // Fallback: coerce via debug string (last resort; Python coerces non-string
    // ids the same way).
    Some(format!("{:?}", array))
}

fn read_vector_at(
    array: &dyn Array,
    idx: usize,
    dim: usize,
    vector_column: &str,
    id: &str,
) -> Result<Vec<f32>> {
    if array.is_null(idx) {
        return Err(eyre!("row id {id}: vector column {vector_column:?} is null"));
    }
    if let Some(list) = array.as_list_opt::<i32>() {
        return list_to_vec(list.value(idx).as_ref(), dim, vector_column, id);
    }
    if let Some(list) = array.as_list_opt::<i64>() {
        return list_to_vec(list.value(idx).as_ref(), dim, vector_column, id);
    }
    if let Some(fixed) = array.as_fixed_size_list_opt() {
        return list_to_vec(fixed.value(idx).as_ref(), dim, vector_column, id);
    }
    Err(eyre!(
        "row id {id}: unsupported vector column type {:?}",
        array.data_type()
    ))
}

fn list_to_vec(inner: &dyn Array, dim: usize, vector_column: &str, id: &str) -> Result<Vec<f32>> {
    let v: Vec<f32> = if let Some(a) = inner.as_primitive_opt::<Float32Type>() {
        a.values().to_vec()
    } else if let Some(a) = inner.as_primitive_opt::<Float64Type>() {
        a.values().iter().map(|x| *x as f32).collect()
    } else {
        return Err(eyre!(
            "row id {id}: vector column {vector_column:?} has element type {:?}, expected Float32/Float64",
            inner.data_type()
        ));
    };
    if v.len() != dim {
        return Err(eyre!(
            "row id {id}: expected dim {dim} in column {vector_column:?}, got {}",
            v.len()
        ));
    }
    Ok(v)
}

/// Render a float in canonical form (up to 9 significant digits, no
/// trailing garbage). Matches Python's `format(float(v), ".9g")`.
pub fn canonical_float(value: f64) -> Result<String> {
    if !value.is_finite() {
        return Err(eyre!(
            "non-finite value {value} is not allowed in embeddings"
        ));
    }
    // Rust's `{:.9}` is fixed-precision, not shortest-roundtrip-g. Re-emit
    // using Python-compatible "%.9g": 9 significant digits, no trailing
    // zeros, scientific only when the magnitude demands it.
    Ok(format_g9(value))
}

fn format_g9(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    // Python's %g drops trailing zeros and the decimal point when not needed;
    // switches to scientific notation outside 1e-4..1e17 range.
    let abs = value.abs();
    let exp = abs.log10().floor() as i32;
    let use_sci = exp < -4 || exp >= 9;
    let raw = if use_sci {
        format!("{value:.8e}")
    } else {
        let digits_after = (8 - exp).max(0) as usize;
        format!("{value:.*}", digits_after)
    };
    trim_trailing_zeros(&raw)
}

fn trim_trailing_zeros(s: &str) -> String {
    // Handle scientific-notation suffix separately so we only trim the
    // mantissa's trailing zeros.
    if let Some((mantissa, exp)) = s.split_once('e') {
        let trimmed = trim_fraction_zeros(mantissa);
        // Python's %g prints the exponent as `e+05` / `e-05` with zero-padded
        // two-digit minimum. Our input from `{:.8e}` is already `e0` / `e-1`
        // style; re-normalize to Python style.
        let exp_i: i32 = exp.parse().unwrap_or(0);
        let sign = if exp_i < 0 { '-' } else { '+' };
        format!("{trimmed}e{sign}{:02}", exp_i.unsigned_abs())
    } else {
        trim_fraction_zeros(s)
    }
}

fn trim_fraction_zeros(s: &str) -> String {
    if let Some(dot) = s.find('.') {
        let mut end = s.len();
        while end > dot + 1 && s.as_bytes()[end - 1] == b'0' {
            end -= 1;
        }
        if s.as_bytes()[end - 1] == b'.' {
            end -= 1;
        }
        s[..end].to_owned()
    } else {
        s.to_owned()
    }
}

/// Render a vector as `[v1,v2,...]` with canonical floats (no spaces).
pub fn canonical_json_array(values: &[f32]) -> Result<String> {
    let mut out = String::with_capacity(values.len() * 10 + 2);
    out.push('[');
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&canonical_float(*v as f64)?);
    }
    out.push(']');
    Ok(out)
}

/// Split the sorted-id prefix into `(corpus_ids, query_ids)` using the
/// profile's `corpus_rows` / `query_start` boundaries.
pub fn split_sorted_ids<'a>(
    sorted_ids: &'a [String],
    profile: &SubsetProfile,
) -> (Vec<String>, Vec<String>) {
    let corpus = sorted_ids[..profile.corpus_rows].to_vec();
    let query = sorted_ids[profile.query_start()..profile.query_start() + profile.query_rows]
        .to_vec();
    (corpus, query)
}

/// Per-file entries recorded in the manifest (file / rows / sha256 /
/// first_id / last_id / first_source_id / last_source_id). Matches the
/// Python `FileManifest` dataclass layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileManifest {
    pub file: String,
    pub rows: usize,
    pub sha256: String,
    pub first_id: Option<i64>,
    pub last_id: Option<i64>,
    pub first_source_id: Option<String>,
    pub last_source_id: Option<String>,
}

/// Write the TSV and return a `FileManifest` describing what landed.
/// Kept as a free function so the sha256 and first/last bookkeeping are
/// unit-testable without touching parquet.
pub fn write_tsv(
    path: &Path,
    entries: &[(i64, String)],
    rows_by_id: &HashMap<String, Vec<f32>>,
) -> Result<FileManifest> {
    let mut hasher = Sha256::new();
    let mut first_id: Option<i64> = None;
    let mut last_id: Option<i64> = None;
    let mut first_src: Option<String> = None;
    let mut last_src: Option<String> = None;
    let file =
        File::create(path).wrap_err_with(|| format!("creating {}", path.display()))?;
    let mut w = BufWriter::new(file);
    for (row_id, source_id) in entries {
        let v = rows_by_id
            .get(source_id)
            .ok_or_else(|| eyre!("row {row_id}: source id {source_id:?} not in materialized set"))?;
        let line = format!("{row_id}\t{}\n", canonical_json_array(v)?);
        if first_id.is_none() {
            first_id = Some(*row_id);
            first_src = Some(source_id.clone());
        }
        last_id = Some(*row_id);
        last_src = Some(source_id.clone());
        hasher.update(line.as_bytes());
        w.write_all(line.as_bytes())?;
    }
    w.flush()?;
    Ok(FileManifest {
        file: path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        rows: entries.len(),
        sha256: hex::encode(hasher.finalize()),
        first_id,
        last_id,
        first_source_id: first_src,
        last_source_id: last_src,
    })
}

/// Basename of the user-provided parquet path. Mirrors the Python helper:
/// for a trailing-slash directory input `/a/b/`, returns `b` (not `""`).
pub fn source_parquet_basename(path: &Path) -> String {
    let s = path.to_string_lossy();
    let trimmed = s.trim_end_matches('/');
    Path::new(trimmed)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| s.into_owned())
}

#[allow(clippy::too_many_arguments)]
pub fn build_manifest_json(
    profile: &SubsetProfile,
    source_parquet_abs: &str,
    source_parquet_basename: String,
    shard_basenames: &[String],
    source_dataset: &str,
    id_column: &str,
    vector_column: &str,
    dim: usize,
    corpus: &FileManifest,
    queries: &FileManifest,
    generated_at_utc: &str,
) -> Value {
    json!({
        "manifest_version": 1,
        "prefix": profile.prefix,
        "source_dataset": source_dataset,
        "source_parquet": source_parquet_abs,
        "source_parquet_basename": source_parquet_basename,
        "source_parquet_shard_basenames": shard_basenames,
        "id_column": id_column,
        "vector_column": vector_column,
        "dimension": dim,
        "selection_rule": {
            "sort_key": format!("{id_column} ascending lexicographic"),
            "corpus_start": 0,
            "corpus_rows": profile.corpus_rows,
            "query_start": profile.query_start(),
            "query_rows": profile.query_rows,
            "output_id_mode": "global_sorted_row_index",
        },
        "corpus": file_manifest_json(corpus),
        "queries": file_manifest_json(queries),
        "generated_at_utc": generated_at_utc,
        "generated_by": "ecaz corpus prepare",
    })
}

fn file_manifest_json(m: &FileManifest) -> Value {
    json!({
        "file": m.file,
        "rows": m.rows,
        "sha256": m.sha256,
        "first_id": m.first_id,
        "last_id": m.last_id,
        "first_source_id": m.first_source_id,
        "last_source_id": m.last_source_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- profile table ---

    #[test]
    fn profile_query_start_equals_corpus_rows() {
        let p = resolve_profile("ec_hnsw_real_50k").unwrap();
        assert_eq!(p.query_start(), 50_000);
        assert_eq!(p.needed_rows(), 51_000);
    }

    #[test]
    fn resolve_profile_unknown_name_is_none() {
        assert!(resolve_profile("nope").is_none());
    }

    #[test]
    fn anchor_profile_matches_documented_split() {
        let p = resolve_profile("ec_hnsw_real_ann_benchmarks_anchor").unwrap();
        assert_eq!(p.corpus_rows, 990_000);
        assert_eq!(p.query_rows, 10_000);
        assert_eq!(p.needed_rows(), 1_000_000);
    }

    // --- column resolution ---

    #[test]
    fn resolve_id_column_picks_unique_candidate() {
        let schema = vec!["id".to_owned(), "embedding".to_owned()];
        assert_eq!(resolve_id_column(&schema, None).unwrap(), "id");
    }

    #[test]
    fn resolve_id_column_respects_override() {
        let schema = vec!["id".to_owned(), "alt".to_owned()];
        assert_eq!(resolve_id_column(&schema, Some("alt")).unwrap(), "alt");
    }

    #[test]
    fn resolve_id_column_rejects_override_missing_from_schema() {
        let schema = vec!["id".to_owned()];
        assert!(resolve_id_column(&schema, Some("nope")).is_err());
    }

    #[test]
    fn resolve_id_column_rejects_ambiguous_candidates() {
        let schema = vec!["id".to_owned(), "_id".to_owned()];
        let err = resolve_id_column(&schema, None).unwrap_err().to_string();
        assert!(err.contains("multiple"), "got {err}");
    }

    #[test]
    fn resolve_vector_column_picks_unique_candidate() {
        let schema = vec!["id".to_owned(), "embedding".to_owned()];
        assert_eq!(
            resolve_vector_column(&schema, None).unwrap(),
            "embedding"
        );
    }

    #[test]
    fn resolve_vector_column_falls_back_to_unique_non_id_column() {
        let schema = vec!["id".to_owned(), "my_weird_name".to_owned()];
        assert_eq!(
            resolve_vector_column(&schema, None).unwrap(),
            "my_weird_name"
        );
    }

    #[test]
    fn resolve_vector_column_rejects_ambiguous_schema() {
        let schema = vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
        assert!(resolve_vector_column(&schema, None).is_err());
    }

    // --- canonical float / json array ---

    #[test]
    fn canonical_float_zero_is_bare_zero() {
        assert_eq!(canonical_float(0.0).unwrap(), "0");
    }

    #[test]
    fn canonical_float_rejects_nan_and_inf() {
        assert!(canonical_float(f64::NAN).is_err());
        assert!(canonical_float(f64::INFINITY).is_err());
        assert!(canonical_float(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn canonical_float_strips_trailing_zeros() {
        assert_eq!(canonical_float(1.5).unwrap(), "1.5");
        assert_eq!(canonical_float(2.0).unwrap(), "2");
    }

    #[test]
    fn canonical_float_keeps_nine_significant_digits() {
        let got = canonical_float(0.123_456_789_123).unwrap();
        // 9 significant digits of 0.123456789123 → 0.123456789
        assert_eq!(got, "0.123456789");
    }

    #[test]
    fn canonical_json_array_joins_with_commas_and_no_spaces() {
        let arr = canonical_json_array(&[0.5, 1.0, -0.25]).unwrap();
        assert_eq!(arr, "[0.5,1,-0.25]");
    }

    // --- split_sorted_ids ---

    #[test]
    fn split_sorted_ids_carves_contiguous_prefixes() {
        let ids: Vec<String> = (0..12).map(|i| format!("id{i:02}")).collect();
        let p = SubsetProfile {
            prefix: "t",
            corpus_rows: 10,
            query_rows: 2,
        };
        let (c, q) = split_sorted_ids(&ids, &p);
        assert_eq!(c.len(), 10);
        assert_eq!(c[0], "id00");
        assert_eq!(c[9], "id09");
        assert_eq!(q, vec!["id10".to_owned(), "id11".to_owned()]);
    }

    // --- push_smallest ---

    #[test]
    fn push_smallest_tracks_k_smallest_across_unsorted_stream() {
        let mut heap = BinaryHeap::new();
        let stream = ["m", "a", "z", "b", "e", "c", "y", "d"];
        for s in stream {
            push_smallest(&mut heap, s.to_owned(), 3);
        }
        let mut result: Vec<String> = heap.into_sorted_vec();
        result.sort();
        assert_eq!(result, vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]);
    }

    #[test]
    fn push_smallest_leaves_heap_unchanged_when_new_id_is_larger() {
        let mut heap = BinaryHeap::new();
        push_smallest(&mut heap, "a".to_owned(), 2);
        push_smallest(&mut heap, "b".to_owned(), 2);
        push_smallest(&mut heap, "z".to_owned(), 2);
        let sorted: Vec<String> = heap.into_sorted_vec();
        assert_eq!(sorted, vec!["a".to_owned(), "b".to_owned()]);
    }

    // --- write_tsv ---

    #[test]
    fn write_tsv_records_sha256_first_last_ids_and_row_count() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("t.tsv");
        let mut rows_by_id = HashMap::new();
        rows_by_id.insert("src_a".to_owned(), vec![0.5_f32, 0.5]);
        rows_by_id.insert("src_b".to_owned(), vec![1.0_f32, -1.0]);
        let entries = vec![(0, "src_a".to_owned()), (1, "src_b".to_owned())];
        let m = write_tsv(&path, &entries, &rows_by_id).unwrap();
        assert_eq!(m.rows, 2);
        assert_eq!(m.first_id, Some(0));
        assert_eq!(m.last_id, Some(1));
        assert_eq!(m.first_source_id.as_deref(), Some("src_a"));
        assert_eq!(m.last_source_id.as_deref(), Some("src_b"));
        assert_eq!(m.sha256.len(), 64);
        // File content must match the sha recorded.
        let bytes = std::fs::read(&path).unwrap();
        let expected = {
            let mut h = Sha256::new();
            h.update(&bytes);
            hex::encode(h.finalize())
        };
        assert_eq!(m.sha256, expected);
    }

    #[test]
    fn write_tsv_on_empty_entries_returns_none_first_last() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("empty.tsv");
        let rows_by_id: HashMap<String, Vec<f32>> = HashMap::new();
        let m = write_tsv(&path, &[], &rows_by_id).unwrap();
        assert_eq!(m.rows, 0);
        assert!(m.first_id.is_none());
        assert!(m.last_id.is_none());
    }

    // --- source_parquet_basename ---

    #[test]
    fn source_parquet_basename_handles_trailing_slash_directory() {
        assert_eq!(
            source_parquet_basename(Path::new("/tmp/foo/")),
            "foo"
        );
        assert_eq!(
            source_parquet_basename(Path::new("/tmp/foo.parquet")),
            "foo.parquet"
        );
    }

    // --- build_manifest_json ---

    #[test]
    fn manifest_json_has_version_one_and_portable_fields() {
        let profile = SubsetProfile {
            prefix: "t",
            corpus_rows: 5,
            query_rows: 2,
        };
        let corpus = FileManifest {
            file: "t_corpus.tsv".into(),
            rows: 5,
            sha256: "a".repeat(64),
            first_id: Some(0),
            last_id: Some(4),
            first_source_id: Some("s0".into()),
            last_source_id: Some("s4".into()),
        };
        let queries = FileManifest {
            file: "t_queries.tsv".into(),
            rows: 2,
            sha256: "b".repeat(64),
            first_id: Some(5),
            last_id: Some(6),
            first_source_id: Some("s5".into()),
            last_source_id: Some("s6".into()),
        };
        let v = build_manifest_json(
            &profile,
            "/abs/path/foo.parquet",
            "foo.parquet".into(),
            &["foo.parquet".to_owned()],
            "test-dataset",
            "id",
            "embedding",
            4,
            &corpus,
            &queries,
            "2026-04-20T00:00:00+00:00",
        );
        assert_eq!(v["manifest_version"], 1);
        assert_eq!(v["prefix"], "t");
        assert_eq!(v["source_parquet_basename"], "foo.parquet");
        assert_eq!(v["selection_rule"]["corpus_rows"], 5);
        assert_eq!(v["selection_rule"]["query_start"], 5);
        assert_eq!(v["selection_rule"]["sort_key"], "id ascending lexicographic");
        assert_eq!(v["corpus"]["rows"], 5);
        assert_eq!(v["queries"]["first_source_id"], "s5");
    }
}
