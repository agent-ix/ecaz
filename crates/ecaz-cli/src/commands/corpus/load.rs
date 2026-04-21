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

    fn opt(k: &str, v: &str) -> (String, String) {
        (k.to_owned(), v.to_owned())
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
}
