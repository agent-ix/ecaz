//! `ecaz corpus load` — port of the legacy `scripts/load_real_corpus.py`.
//!
//! See the module-level doc in `super` for the corpus model. This command
//! is the only way new data enters Postgres; everything downstream assumes
//! the `<prefix>_corpus` / `<prefix>_queries` contract it establishes.
//!
//! The flow is idempotent: an existing non-empty corpus/query table is
//! left alone, and an index whose reloptions already match the requested
//! set is kept as-is. This preserves the "load once, rerun forever"
//! discipline that makes the real-corpus benchmarks cheap to iterate.

use bytes::{BufMut, BytesMut};
use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use futures::SinkExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio_postgres::Client;

use crate::manifest;
use crate::profiles::{self, IndexProfile};
use crate::psql;
use crate::reloptions;
use crate::tsv;

const DEFAULT_HNSW_BUILD_SOURCE_COLUMN: &str = "source";
const DEFAULT_HNSW_M_SWEEP: &[i32] = &[8, 16];
/// Flush the COPY sink at roughly this size. Large enough to amortise the
/// async send overhead, small enough that a 10M-row corpus still surfaces
/// progress before finishing.
const COPY_CHUNK_BYTES: usize = 1 << 20;

#[derive(Args, Debug)]
pub struct LoadArgs {
    /// Fixture prefix used for table and index names. Must match
    /// [a-zA-Z_][a-zA-Z0-9_]*.
    #[arg(long)]
    pub prefix: String,

    /// Path to <basename>_corpus.tsv (one `id\t<json_array>` per line).
    #[arg(long)]
    pub corpus_file: PathBuf,

    /// Path to <basename>_queries.tsv (one `id\t<json_array>` per line).
    #[arg(long)]
    pub queries_file: PathBuf,

    /// Vector dimensionality.
    #[arg(long, default_value_t = 1536)]
    pub dim: usize,

    /// Access-method profile (drives embedding type, encoder, opclass).
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,

    /// Quantization bits passed to the profile's encoder.
    #[arg(long, default_value_t = 4)]
    pub bits: i32,

    /// Quantizer seed passed to the profile's encoder.
    #[arg(long, default_value_t = 42)]
    pub seed: i64,

    /// HNSW-only: m values to sweep. Accepts `--m 8,16` or repeated `--m 8 --m 16`.
    #[arg(long, value_delimiter = ',')]
    pub m: Vec<i32>,

    /// HNSW-only: ef_construction passed to CREATE INDEX.
    #[arg(long, default_value_t = 128)]
    pub ef_construction: i32,

    /// Optional storage format (turboquant / pq_fastscan).
    #[arg(long)]
    pub storage_format: Option<String>,

    /// AM-specific reloption passthrough. Repeatable.
    /// Example: `--reloption graph_degree=48 --reloption alpha=1.2`.
    #[arg(long = "reloption", value_parser = crate::reloptions::parse_cli)]
    pub reloptions: Vec<(String, String)>,

    /// Optional manifest file path (auto-discovered when corpus/queries files
    /// follow the `<basename>_{corpus,queries}.tsv` convention).
    #[arg(long)]
    pub manifest_file: Option<PathBuf>,

    /// Continue past manifest verification failures with a warning.
    #[arg(long)]
    pub allow_manifest_mismatch: bool,
}

struct IndexJob {
    name: String,
    reloptions: Vec<(String, String)>,
}

pub async fn run(database: &str, args: LoadArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    let profile = profiles::resolve(&args.profile).ok_or_else(|| {
        eyre!(
            "unknown profile {:?}; try {}",
            args.profile,
            profiles::names().join(", ")
        )
    })?;

    if !profile.sweep_axis_is_m() && !args.m.is_empty() {
        return Err(eyre!(
            "--m is not supported by profile {:?}; use --reloption for AM-specific tunables",
            profile.name
        ));
    }

    let unknown = profile.unknown_reloption_keys(&args.reloptions);
    if !unknown.is_empty() {
        eprintln!(
            "[loader] warning: profile {:?} does not list {} as known reloption{}; \
             passing through verbatim. Known reloptions: {}",
            profile.name,
            unknown.join(", "),
            if unknown.len() == 1 { "" } else { "s" },
            profile.known_reloptions.join(", ")
        );
    }

    let collisions =
        reloption_flag_collisions(profile, &args.reloptions, args.storage_format.as_deref());
    if !collisions.is_empty() {
        let formatted = collisions
            .iter()
            .map(|c| format!("--reloption {}=... conflicts with {}", c.key, c.flag))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(eyre!(
            "{formatted}. Use the native CLI flag or drop the --reloption, not both"
        ));
    }

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let index_prefix = match args.storage_format.as_deref() {
        Some(sf) => format!("{}_{sf}", args.prefix),
        None => args.prefix.clone(),
    };
    let index_jobs = plan_index_jobs(
        profile,
        &index_prefix,
        &args.m,
        args.ef_construction,
        args.storage_format.as_deref(),
        &args.reloptions,
    );

    // Inspect inputs first: row counts drive progress bars and manifest
    // verification, and we want to fail fast on malformed files before we
    // open any transactions.
    eprintln!("[loader] inspecting {}", args.corpus_file.display());
    let corpus_stats = tsv::inspect(&args.corpus_file, args.dim)?;
    eprintln!("[loader] inspecting {}", args.queries_file.display());
    let query_stats = tsv::inspect(&args.queries_file, args.dim)?;

    eprintln!(
        "[loader] corpus: {} rows, sha256={}  queries: {} rows, sha256={}",
        corpus_stats.rows, corpus_stats.sha256_hex, query_stats.rows, query_stats.sha256_hex
    );

    verify_manifest_if_present(
        args.manifest_file.as_deref(),
        &args.corpus_file,
        &args.queries_file,
        &args.prefix,
        args.dim,
        &corpus_stats,
        &query_stats,
        args.allow_manifest_mismatch,
    )?;

    let client = psql::connect(database).await?;

    let corpus_loaded = ensure_corpus_table(
        &client,
        &corpus_table,
        &args.corpus_file,
        args.dim,
        args.bits,
        args.seed,
        profile,
        corpus_stats.rows,
    )
    .await?;
    let queries_loaded = ensure_queries_table(
        &client,
        &queries_table,
        &args.queries_file,
        args.dim,
        query_stats.rows,
    )
    .await?;

    for job in &index_jobs {
        ensure_index(&client, &corpus_table, job, profile).await?;
    }

    print_summary(
        profile,
        &corpus_table,
        corpus_loaded,
        &queries_table,
        queries_loaded,
        &index_jobs,
    );
    Ok(())
}

/// Verify a sibling manifest if one was requested or auto-discovered.
///
/// Three paths:
/// - `--manifest-file` passed: the path must exist, or we fail.
/// - No flag, sibling auto-discovered and present: verify it.
/// - No flag, no sibling on disk: log once, continue without verification.
///
/// When problems are found and `allow_mismatch` is false, bail with the
/// full diff. With `allow_mismatch`, log a warning and continue so a
/// reviewer can poke at an inconsistent fixture without rebuilding it.
fn verify_manifest_if_present(
    explicit: Option<&Path>,
    corpus_file: &Path,
    queries_file: &Path,
    prefix: &str,
    dim: usize,
    corpus_stats: &tsv::VectorFileStats,
    query_stats: &tsv::VectorFileStats,
    allow_mismatch: bool,
) -> Result<()> {
    let derived = manifest::derive_manifest_path(corpus_file, queries_file);
    let (path, explicit_request): (PathBuf, bool) = match (explicit, derived) {
        (Some(p), _) => (p.to_path_buf(), true),
        (None, Some(p)) if p.exists() => (p, false),
        (None, Some(p)) => {
            eprintln!(
                "[loader] no sibling manifest at {}; continuing without verification",
                p.display()
            );
            return Ok(());
        }
        (None, None) => return Ok(()),
    };
    if explicit_request && !path.exists() {
        return Err(eyre!("manifest file {:?} does not exist", path));
    }
    let raw = std::fs::read_to_string(&path)
        .wrap_err_with(|| format!("reading manifest {}", path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .wrap_err_with(|| format!("parsing manifest {}", path.display()))?;
    let problems = manifest::verify(
        &parsed,
        prefix,
        corpus_file,
        queries_file,
        dim,
        corpus_stats,
        query_stats,
    );
    if problems.is_empty() {
        eprintln!("[loader] verified manifest {} for prefix {prefix}", path.display());
        return Ok(());
    }
    let joined = problems
        .iter()
        .map(|p| p.0.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    let msg = format!("manifest verification failed for {}: {joined}", path.display());
    if allow_mismatch {
        eprintln!("[loader] warning: {msg}");
        Ok(())
    } else {
        Err(eyre!(msg))
    }
}

/// Pair describing a `--reloption key=...` that duplicates a native CLI flag.
struct FlagCollision {
    key: &'static str,
    flag: &'static str,
}

/// Reject `--reloption` keys that a native CLI flag already sets. Postgres
/// rejects duplicate reloption keys at `CREATE INDEX`, and even when it
/// doesn't, letting `--reloption` silently override a native flag is worse
/// UX than a clear up-front error pointing at the redundant flag.
fn reloption_flag_collisions(
    profile: &IndexProfile,
    reloptions: &[(String, String)],
    storage_format: Option<&str>,
) -> Vec<FlagCollision> {
    let mut managed: Vec<FlagCollision> = Vec::new();
    if profile.sweep_axis_is_m() {
        managed.push(FlagCollision { key: "m", flag: "--m" });
        managed.push(FlagCollision {
            key: "ef_construction",
            flag: "--ef-construction",
        });
        managed.push(FlagCollision {
            key: "build_source_column",
            flag: "(HNSW built-in)",
        });
    }
    if storage_format.is_some() {
        managed.push(FlagCollision {
            key: "storage_format",
            flag: "--storage-format",
        });
    }
    managed
        .into_iter()
        .filter(|c| reloptions.iter().any(|(k, _)| k == c.key))
        .collect()
}

fn plan_index_jobs(
    profile: &IndexProfile,
    index_prefix: &str,
    m_values: &[i32],
    ef_construction: i32,
    storage_format: Option<&str>,
    extra: &[(String, String)],
) -> Vec<IndexJob> {
    if profile.sweep_axis_is_m() {
        let sweep = dedup_preserve_order(if m_values.is_empty() {
            DEFAULT_HNSW_M_SWEEP.to_vec()
        } else {
            m_values.to_vec()
        });
        sweep
            .into_iter()
            .map(|m| {
                let mut opts: Vec<(String, String)> = vec![
                    ("m".into(), m.to_string()),
                    ("ef_construction".into(), ef_construction.to_string()),
                    (
                        "build_source_column".into(),
                        DEFAULT_HNSW_BUILD_SOURCE_COLUMN.into(),
                    ),
                ];
                if let Some(sf) = storage_format {
                    opts.push(("storage_format".into(), sf.into()));
                }
                opts.extend(extra.iter().cloned());
                IndexJob {
                    name: format!("{index_prefix}_m{m}_idx"),
                    reloptions: opts,
                }
            })
            .collect()
    } else {
        let mut opts: Vec<(String, String)> = extra.to_vec();
        if let Some(sf) = storage_format {
            opts.push(("storage_format".into(), sf.into()));
        }
        vec![IndexJob {
            name: format!("{index_prefix}_idx"),
            reloptions: opts,
        }]
    }
}

fn dedup_preserve_order(values: Vec<i32>) -> Vec<i32> {
    let mut seen = std::collections::HashSet::new();
    values.into_iter().filter(|v| seen.insert(*v)).collect()
}

async fn ensure_corpus_table(
    client: &Client,
    table: &str,
    path: &Path,
    dim: usize,
    bits: i32,
    seed: i64,
    profile: &IndexProfile,
    expected_rows: usize,
) -> Result<usize> {
    if psql::relation_exists(client, table, 'r').await? {
        let existing = psql::row_count(client, table).await? as usize;
        if existing > 0 {
            eprintln!("[loader] {table} already has {existing} rows; skipping reload");
            return Ok(existing);
        }
        eprintln!("[loader] {table} exists but is empty; dropping and reloading");
        client
            .batch_execute(&format!("DROP TABLE IF EXISTS {table} CASCADE"))
            .await?;
    }
    client
        .batch_execute(&format!(
            "CREATE TABLE {table} (
                id        bigint PRIMARY KEY,
                source    real[] NOT NULL,
                embedding {embedding}
            )",
            embedding = profile.embedding_type
        ))
        .await
        .wrap_err_with(|| format!("creating table {table}"))?;

    copy_rows_from_tsv(client, table, path, dim, expected_rows, "corpus").await?;

    eprintln!(
        "[loader] encoding {embedding_type} embeddings via {fn_name}(source, {bits}, {seed}) ...",
        embedding_type = profile.embedding_type,
        fn_name = profile.encoder_function
    );
    client
        .batch_execute(&format!(
            "UPDATE {table} SET embedding = {fn_name}(source, {bits}, {seed})",
            fn_name = profile.encoder_function
        ))
        .await
        .wrap_err_with(|| format!("encoding embeddings for {table}"))?;
    psql::row_count(client, table).await.map(|n| n as usize)
}

async fn ensure_queries_table(
    client: &Client,
    table: &str,
    path: &Path,
    dim: usize,
    expected_rows: usize,
) -> Result<usize> {
    if psql::relation_exists(client, table, 'r').await? {
        let existing = psql::row_count(client, table).await? as usize;
        if existing > 0 {
            eprintln!("[loader] {table} already has {existing} rows; skipping reload");
            return Ok(existing);
        }
        eprintln!("[loader] {table} exists but is empty; dropping and reloading");
        client
            .batch_execute(&format!("DROP TABLE IF EXISTS {table} CASCADE"))
            .await?;
    }
    client
        .batch_execute(&format!(
            "CREATE TABLE {table} (
                id     bigint PRIMARY KEY,
                source real[] NOT NULL
            )"
        ))
        .await
        .wrap_err_with(|| format!("creating table {table}"))?;
    copy_rows_from_tsv(client, table, path, dim, expected_rows, "queries").await?;
    psql::row_count(client, table).await.map(|n| n as usize)
}

async fn copy_rows_from_tsv(
    client: &Client,
    table: &str,
    path: &Path,
    dim: usize,
    expected_rows: usize,
    label: &str,
) -> Result<()> {
    let sink = client
        .copy_in::<_, bytes::Bytes>(&format!(
            "COPY {table} (id, source) FROM STDIN WITH (FORMAT text, DELIMITER E'\\t')"
        ))
        .await
        .wrap_err_with(|| format!("opening COPY stream for {table}"))?;
    futures::pin_mut!(sink);

    let bar = ProgressBar::new(expected_rows as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "[loader] {msg} {wide_bar} {human_pos}/{human_len} ({per_sec}, eta {eta})",
        )
        .unwrap(),
    );
    bar.set_message(format!("loading {label} into {table}"));
    bar.enable_steady_tick(Duration::from_millis(250));

    let mut buf = BytesMut::with_capacity(COPY_CHUNK_BYTES + 4096);
    let mut sent = 0u64;
    for row in tsv::iter_rows(path, dim)? {
        let row = row?;
        use std::io::Write as _;
        let mut w = (&mut buf).writer();
        write!(w, "{}\t", row.id).expect("bytesmut writer is infallible");
        // Reuse the shared array-literal formatter so the COPY payload and
        // any other place we render vectors agree on float repr.
        let lit = tsv::format_real_array_literal(&row.values);
        buf.put_slice(lit.as_bytes());
        buf.put_u8(b'\n');
        sent += 1;
        if buf.len() >= COPY_CHUNK_BYTES {
            sink.send(buf.split().freeze())
                .await
                .wrap_err_with(|| format!("COPY send failed for {table}"))?;
            bar.set_position(sent);
        }
    }
    if !buf.is_empty() {
        sink.send(buf.split().freeze())
            .await
            .wrap_err_with(|| format!("COPY send failed for {table}"))?;
    }
    let finished = sink
        .finish()
        .await
        .wrap_err_with(|| format!("COPY finish failed for {table}"))?;
    bar.finish_with_message(format!(
        "loaded {finished} {label} rows into {table}"
    ));
    Ok(())
}

async fn ensure_index(
    client: &Client,
    corpus_table: &str,
    job: &IndexJob,
    profile: &IndexProfile,
) -> Result<()> {
    let summary = if job.reloptions.is_empty() {
        "<none>".to_owned()
    } else {
        reloptions::normalize_list(&job.reloptions).join(", ")
    };
    if psql::index_exists_with_reloptions(client, &job.name, &job.reloptions).await? {
        eprintln!(
            "[loader] {index} already exists with reloptions=[{summary}]; skipping rebuild",
            index = job.name
        );
        return Ok(());
    }
    eprintln!(
        "[loader] building {index} using {am} (reloptions=[{summary}]) ...",
        index = job.name,
        am = profile.access_method,
    );
    let sql = psql::build_create_index_sql(corpus_table, &job.name, profile, &job.reloptions);
    client
        .batch_execute(&sql)
        .await
        .wrap_err_with(|| format!("building index {}", job.name))?;
    Ok(())
}

fn print_summary(
    profile: &IndexProfile,
    corpus_table: &str,
    corpus_rows: usize,
    queries_table: &str,
    queries_rows: usize,
    jobs: &[IndexJob],
) {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_header(vec!["field", "value"]);
    t.add_row(vec!["profile".into(), Cell::new(profile.name)]);
    t.add_row(vec![
        "corpus".into(),
        Cell::new(format!("{corpus_table} ({corpus_rows} rows)")),
    ]);
    t.add_row(vec![
        "queries".into(),
        Cell::new(format!("{queries_table} ({queries_rows} rows)")),
    ]);
    let indexes = jobs
        .iter()
        .map(|j| {
            let opts = if j.reloptions.is_empty() {
                "<default>".to_owned()
            } else {
                reloptions::normalize_list(&j.reloptions).join(", ")
            };
            format!("{} [{}]", j.name, opts)
        })
        .collect::<Vec<_>>()
        .join("\n");
    t.add_row(vec!["indexes".into(), Cell::new(indexes)]);
    println!("{t}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{EC_DISKANN, EC_HNSW};
    use crate::tsv::VectorFileStats;
    use std::io::Write as _;
    use tempfile::TempDir;

    fn opt(k: &str, v: &str) -> (String, String) {
        (k.to_owned(), v.to_owned())
    }

    fn stats(rows: usize, sha: &str) -> VectorFileStats {
        VectorFileStats {
            rows,
            sha256_hex: sha.to_owned(),
            first_id: Some(0),
            last_id: Some(rows.saturating_sub(1) as i64),
        }
    }

    #[test]
    fn hnsw_plan_defaults_to_8_16_sweep_with_ef_and_build_source() {
        let jobs = plan_index_jobs(&EC_HNSW, "dbpedia_10k", &[], 128, None, &[]);
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].name, "dbpedia_10k_m8_idx");
        assert_eq!(jobs[1].name, "dbpedia_10k_m16_idx");
        assert!(jobs[0].reloptions.contains(&opt("m", "8")));
        assert!(jobs[0].reloptions.contains(&opt("ef_construction", "128")));
        assert!(jobs[0]
            .reloptions
            .contains(&opt("build_source_column", "source")));
    }

    #[test]
    fn hnsw_plan_honors_explicit_m_and_dedup() {
        let jobs = plan_index_jobs(
            &EC_HNSW,
            "foo_pq_fastscan",
            &[8, 16, 8],
            96,
            Some("pq_fastscan"),
            &[],
        );
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].name, "foo_pq_fastscan_m8_idx");
        assert!(jobs[0].reloptions.contains(&opt("ef_construction", "96")));
        assert!(jobs[0].reloptions.contains(&opt("storage_format", "pq_fastscan")));
    }

    #[test]
    fn hnsw_plan_passes_extras_through_and_orders_after_built_ins() {
        let extras = vec![opt("storage_format", "turboquant"), opt("custom", "x")];
        let jobs = plan_index_jobs(&EC_HNSW, "p", &[8], 128, None, &extras);
        // built-ins come first so duplicates from --reloption would override
        let keys: Vec<&str> = jobs[0].reloptions.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(
            keys,
            vec!["m", "ef_construction", "build_source_column", "storage_format", "custom"]
        );
    }

    #[test]
    fn dedup_preserve_order_keeps_first_occurrence() {
        assert_eq!(
            dedup_preserve_order(vec![16, 8, 16, 32, 8]),
            vec![16, 8, 32]
        );
        assert_eq!(dedup_preserve_order(vec![]), Vec::<i32>::new());
        assert_eq!(dedup_preserve_order(vec![8]), vec![8]);
    }

    #[test]
    fn diskann_plan_is_single_index_with_no_hnsw_defaults() {
        let extras = vec![opt("graph_degree", "48")];
        let jobs = plan_index_jobs(&EC_DISKANN, "foo", &[], 128, None, &extras);
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "foo_idx");
        assert!(jobs[0].reloptions.contains(&opt("graph_degree", "48")));
        assert!(!jobs[0].reloptions.iter().any(|(k, _)| k == "m"));
        assert!(!jobs[0]
            .reloptions
            .iter()
            .any(|(k, _)| k == "build_source_column"));
    }

    #[test]
    fn diskann_plan_appends_storage_format_to_extras() {
        let jobs = plan_index_jobs(&EC_DISKANN, "foo_pq_fastscan", &[], 128, Some("pq_fastscan"), &[]);
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "foo_pq_fastscan_idx");
        assert!(jobs[0].reloptions.contains(&opt("storage_format", "pq_fastscan")));
    }

    // --- reloption / CLI flag collisions ---

    #[test]
    fn collision_hnsw_m_reloption_flagged() {
        let opts = vec![opt("m", "32")];
        let c = reloption_flag_collisions(&EC_HNSW, &opts, None);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].key, "m");
    }

    #[test]
    fn collision_hnsw_ef_construction_and_build_source_flagged() {
        let opts = vec![opt("ef_construction", "96"), opt("build_source_column", "x")];
        let c = reloption_flag_collisions(&EC_HNSW, &opts, None);
        let keys: Vec<&str> = c.iter().map(|c| c.key).collect();
        assert!(keys.contains(&"ef_construction"));
        assert!(keys.contains(&"build_source_column"));
    }

    #[test]
    fn collision_storage_format_flagged_only_when_cli_flag_set() {
        let opts = vec![opt("storage_format", "pq_fastscan")];
        assert!(reloption_flag_collisions(&EC_DISKANN, &opts, None).is_empty());
        let c = reloption_flag_collisions(&EC_DISKANN, &opts, Some("turboquant"));
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].key, "storage_format");
    }

    #[test]
    fn collision_diskann_m_reloption_not_flagged() {
        // DiskANN has no --m flag; an `m=` reloption here is just pass-through
        // (and independently flagged as unknown by profile.unknown_reloption_keys).
        let opts = vec![opt("m", "32")];
        assert!(reloption_flag_collisions(&EC_DISKANN, &opts, None).is_empty());
    }

    #[test]
    fn collision_empty_when_no_overlap() {
        let opts = vec![opt("graph_degree", "48"), opt("alpha", "1.2")];
        assert!(reloption_flag_collisions(&EC_DISKANN, &opts, None).is_empty());
        assert!(reloption_flag_collisions(&EC_HNSW, &[], None).is_empty());
    }

    // --- manifest orchestration ---

    fn write(dir: &TempDir, name: &str, body: &str) -> PathBuf {
        let p = dir.path().join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        p
    }

    #[test]
    fn manifest_orchestration_no_derived_no_explicit_is_ok() {
        let td = TempDir::new().unwrap();
        let corpus = write(&td, "odd_name.txt", "");
        let queries = write(&td, "other.txt", "");
        let res = verify_manifest_if_present(
            None,
            &corpus,
            &queries,
            "p",
            4,
            &stats(1, &"a".repeat(64)),
            &stats(1, &"b".repeat(64)),
            false,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn manifest_orchestration_explicit_missing_errs() {
        let td = TempDir::new().unwrap();
        let corpus = write(&td, "x_corpus.tsv", "");
        let queries = write(&td, "x_queries.tsv", "");
        let missing = td.path().join("nope.json");
        let err = verify_manifest_if_present(
            Some(&missing),
            &corpus,
            &queries,
            "x",
            4,
            &stats(1, &"a".repeat(64)),
            &stats(1, &"b".repeat(64)),
            false,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("does not exist"), "err: {err}");
    }

    #[test]
    fn manifest_orchestration_sibling_auto_discovered_verified() {
        let td = TempDir::new().unwrap();
        let corpus = write(&td, "x_corpus.tsv", "");
        let queries = write(&td, "x_queries.tsv", "");
        let manifest_path = td.path().join("x_manifest.json");
        let body = serde_json::json!({
            "manifest_version": 1,
            "prefix": "x",
            "dimension": 4,
            "corpus": {
                "file": "x_corpus.tsv", "rows": 1,
                "sha256": "a".repeat(64), "first_id": 0, "last_id": 0
            },
            "queries": {
                "file": "x_queries.tsv", "rows": 1,
                "sha256": "b".repeat(64), "first_id": 0, "last_id": 0
            }
        })
        .to_string();
        std::fs::write(&manifest_path, body).unwrap();
        verify_manifest_if_present(
            None,
            &corpus,
            &queries,
            "x",
            4,
            &stats(1, &"a".repeat(64)),
            &stats(1, &"b".repeat(64)),
            false,
        )
        .unwrap();
    }

    #[test]
    fn manifest_orchestration_mismatch_errs_unless_allowed() {
        let td = TempDir::new().unwrap();
        let corpus = write(&td, "x_corpus.tsv", "");
        let queries = write(&td, "x_queries.tsv", "");
        let manifest_path = td.path().join("x_manifest.json");
        let body = serde_json::json!({
            "manifest_version": 1, "prefix": "x", "dimension": 4,
            "corpus": { "file": "x_corpus.tsv", "rows": 99,
                        "sha256": "a".repeat(64), "first_id": 0, "last_id": 0 },
            "queries": { "file": "x_queries.tsv", "rows": 1,
                         "sha256": "b".repeat(64), "first_id": 0, "last_id": 0 },
        })
        .to_string();
        std::fs::write(&manifest_path, body).unwrap();

        let strict = verify_manifest_if_present(
            None, &corpus, &queries, "x", 4,
            &stats(1, &"a".repeat(64)), &stats(1, &"b".repeat(64)),
            false,
        );
        assert!(strict.is_err());
        let lenient = verify_manifest_if_present(
            None, &corpus, &queries, "x", 4,
            &stats(1, &"a".repeat(64)), &stats(1, &"b".repeat(64)),
            true,
        );
        assert!(lenient.is_ok());
    }
}
