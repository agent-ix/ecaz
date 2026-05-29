//! `ecaz bench sidecar-rerank` — IVF/RaBitQ sidecar upper-bound study.
//!
//! This is intentionally a measurement harness, not an index feature. It asks
//! an isolated `ec_ivf`/RaBitQ `rerank=off` index for an approximate candidate
//! frontier, then locally reranks only those candidate ids with f32, f16, or
//! bits=8 RaBitQ sidecar representations.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use bytes::Bytes;
use clap::{Args, ValueEnum};
use color_eyre::eyre::{bail, eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use futures::{
    stream::{self, StreamExt, TryStreamExt},
    SinkExt,
};
use half::f16;
use ndarray::{s, Array2};
use tokio_postgres::Client;

use ecaz::bench_api::{ProdQuantizer, Quantizer, RaBitQQuantizer};

use crate::profiles::{self, IndexProfile};
use crate::psql::{self, ConnectionOptions};

use super::recall::{
    brute_force_top_k, fetch_sources_public, map_indices_to_ids, ndcg_at_k_from_sources,
    recall_summary_at_k,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum SidecarVariant {
    F32,
    F16,
    Rabitq8,
    Rabitq8ls,
    Rabitq8c3,
    Rabitq8c4,
}

impl SidecarVariant {
    fn label(self) -> &'static str {
        match self {
            Self::F32 => "f32",
            Self::F16 => "f16",
            Self::Rabitq8 => "rabitq8",
            Self::Rabitq8ls => "rabitq8ls",
            Self::Rabitq8c3 => "rabitq8c3",
            Self::Rabitq8c4 => "rabitq8c4",
        }
    }

    fn rabitq_clip(self) -> Option<f32> {
        match self {
            Self::Rabitq8 | Self::Rabitq8ls => Some(2.0),
            Self::Rabitq8c3 => Some(3.0),
            Self::Rabitq8c4 => Some(4.0),
            Self::F32 | Self::F16 => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum SidecarReadMode {
    Free,
    RandomId,
    TidSorted,
}

impl SidecarReadMode {
    fn label(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::RandomId => "random-id",
            Self::TidSorted => "tid-sorted",
        }
    }

    fn uses_db(self) -> bool {
        !matches!(self, Self::Free)
    }
}

#[derive(Args, Debug)]
pub struct SidecarRerankArgs {
    /// Prefix identifying the isolated IVF/RaBitQ corpus.
    #[arg(long)]
    pub prefix: String,
    /// Access-method profile. Currently intended for ec_ivf only.
    #[arg(long, default_value = "ec_ivf")]
    pub profile: String,
    /// k for recall@k / NDCG@k.
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    /// Candidate frontier size to fetch from the rerank=off IVF index.
    #[arg(long, default_value_t = 50)]
    pub candidate_k: usize,
    /// Concurrent sidecar DB fetch/score tasks per variant/read-mode.
    #[arg(long, default_value_t = 1)]
    pub concurrency: usize,
    /// Sweep values for the profile tuning GUC.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// Cap the query set.
    #[arg(long)]
    pub queries_limit: Option<usize>,
    /// Run this many untimed queries before collecting timed candidate and sidecar metrics.
    #[arg(long, default_value_t = 0)]
    pub warmup_queries: usize,
    /// Quantization bits used when encoding query vectors at scan time.
    #[arg(long, default_value_t = 4)]
    pub bits: i32,
    /// Quantizer seed.
    #[arg(long, default_value_t = 42)]
    pub seed: i64,
    /// Sidecar variants to evaluate. Omit to run f32, f16, and rabitq8.
    #[arg(long, value_enum)]
    pub variant: Vec<SidecarVariant>,
    /// Sidecar read modes to evaluate. Omit to run the free-I/O upper bound.
    #[arg(long, value_enum)]
    pub read_mode: Vec<SidecarReadMode>,
    /// Drop and repopulate fixed-width sidecar tables before real-I/O modes.
    #[arg(long)]
    pub rebuild_sidecar_table: bool,
    /// Force benchmark queries onto the ordered ANN path.
    #[arg(long)]
    pub force_index: bool,
    /// Permit non-isolated or non-rerank=off indexes. Intended only for debugging.
    #[arg(long)]
    pub allow_unsafe_index_shape: bool,
    /// Write the final table to this path in addition to stdout.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
}

pub async fn run(conn: &ConnectionOptions, args: SidecarRerankArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if args.k == 0 {
        bail!("--k must be >= 1");
    }
    if args.candidate_k < args.k {
        bail!("--candidate-k must be >= --k");
    }
    if args.concurrency == 0 {
        bail!("--concurrency must be >= 1");
    }
    let profile = profiles::resolve(&args.profile).ok_or_else(|| {
        eyre!(
            "unknown profile {:?}; try {}",
            args.profile,
            profiles::names().join(", ")
        )
    })?;
    if profile.name != "ec_ivf" && !args.allow_unsafe_index_shape {
        bail!("sidecar-rerank is currently scoped to --profile ec_ivf");
    }
    let scan_guc = profile
        .ef_search_guc
        .ok_or_else(|| eyre!("profile {:?} has no tuning GUC", profile.name))?;
    let sweep = if args.sweep.is_empty() {
        profile.default_sweep.to_vec()
    } else {
        args.sweep.clone()
    };
    if sweep.is_empty() {
        bail!("--sweep is required for profile {:?}", profile.name);
    }

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let client = psql::connect(conn).await?;
    validate_index_shape(
        &client,
        &corpus_table,
        profile,
        args.allow_unsafe_index_shape,
    )
    .await?;
    if args.force_index {
        psql::prefer_ordered_ann_path(&client).await?;
    }

    eprintln!("[sidecar-rerank] fetching queries from {queries_table} ...");
    let (_query_ids, queries) =
        fetch_sources_public(&client, &queries_table, args.queries_limit).await?;
    if queries.nrows() == 0 {
        bail!("queries table {queries_table} is empty");
    }
    eprintln!("[sidecar-rerank] fetching corpus from {corpus_table} ...");
    let (corpus_ids, corpus) = fetch_sources_public(&client, &corpus_table, None).await?;
    validate_corpus_and_queries(&corpus_table, &corpus, &queries)?;

    eprintln!(
        "[sidecar-rerank] computing exact truth: {} queries vs {} corpus rows ...",
        queries.nrows(),
        corpus.nrows()
    );
    let truth_gt = brute_force_top_k(&corpus, &queries, args.k);
    let truth_ids = map_indices_to_ids(&truth_gt.indices, &corpus_ids);
    let id_to_pos: HashMap<i64, usize> = corpus_ids
        .iter()
        .enumerate()
        .map(|(pos, id)| (*id, pos))
        .collect();

    let variants = if args.variant.is_empty() {
        vec![
            SidecarVariant::F32,
            SidecarVariant::F16,
            SidecarVariant::Rabitq8,
        ]
    } else {
        args.variant.clone()
    };
    let read_modes = if args.read_mode.is_empty() {
        vec![SidecarReadMode::Free]
    } else {
        args.read_mode.clone()
    };
    let knn_sql = super::recall::build_knn_sql(profile, &corpus_table);
    let stmt = client
        .prepare(&knn_sql)
        .await
        .wrap_err("preparing sidecar-rerank candidate statement")?;

    let warmup_queries = if args.warmup_queries > 0 {
        let warmup_count = args.warmup_queries.min(queries.nrows());
        eprintln!(
            "[sidecar-rerank] warming candidate path with {warmup_count} untimed queries ..."
        );
        Some(queries.slice(s![0..warmup_count, ..]).to_owned())
    } else {
        None
    };

    let mut warmup_candidate_runs = Vec::new();
    if let Some(warmup_queries) = warmup_queries.as_ref() {
        for value in &sweep {
            client
                .batch_execute(&format!("SET {scan_guc} = {value}"))
                .await
                .wrap_err_with(|| format!("SET {scan_guc} = {value}"))?;
            let warmup_run = collect_candidates(
                &client,
                &stmt,
                profile,
                warmup_queries,
                args.bits,
                args.seed,
                args.candidate_k,
            )
            .await?;
            warmup_candidate_runs.push((*value, warmup_run));
        }
    }

    let mut candidate_runs = Vec::with_capacity(sweep.len());
    for value in &sweep {
        client
            .batch_execute(&format!("SET {scan_guc} = {value}"))
            .await
            .wrap_err_with(|| format!("SET {scan_guc} = {value}"))?;
        let candidate_run = collect_candidates(
            &client,
            &stmt,
            profile,
            &queries,
            args.bits,
            args.seed,
            args.candidate_k,
        )
        .await?;
        candidate_runs.push((*value, candidate_run));
    }
    client
        .batch_execute(&format!("RESET {scan_guc}"))
        .await
        .wrap_err_with(|| format!("RESET {scan_guc}"))?;

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        profile.sweep_axis_label(),
        "variant",
        "read_mode",
        "queries",
        "candidate_k",
        "concurrency",
        "recall@k",
        "recall_p10",
        "recall_p50",
        "recall_p90",
        "ndcg@k",
        "candidate_sql_p50",
        "sidecar_io_p50",
        "sidecar_score_p50",
        "sidecar_p50",
        "total_bound_p50",
        "candidate_sql_p95",
        "sidecar_io_p95",
        "sidecar_score_p95",
        "sidecar_p95",
        "total_bound_p95",
        "candidate_sql_p99",
        "sidecar_io_p99",
        "sidecar_score_p99",
        "sidecar_p99",
        "total_bound_p99",
        "sidecar_bytes_per_vector",
        "sidecar_size",
    ]);

    let uses_db_sidecars = read_modes.iter().any(|mode| mode.uses_db());
    for variant in variants {
        let sidecar = build_sidecar(variant, &corpus, args.seed as u64)?;
        if uses_db_sidecars {
            ensure_sidecar_tables(
                &client,
                &args.prefix,
                std::slice::from_ref(&sidecar),
                &corpus_ids,
                &corpus,
                args.rebuild_sidecar_table,
            )
            .await?;
        }
        if let Some(warmup_queries) = warmup_queries.as_ref() {
            eprintln!(
                "[sidecar-rerank] warming sidecar path for {} with {} untimed queries ...",
                sidecar.variant.label(),
                warmup_queries.nrows()
            );
            for (_, warmup_candidate_run) in &warmup_candidate_runs {
                for read_mode in &read_modes {
                    match read_mode {
                        SidecarReadMode::Free => {
                            let _ = rerank_with_sidecar(
                                &sidecar,
                                &warmup_candidate_run.ids,
                                &id_to_pos,
                                &corpus,
                                warmup_queries,
                            )?;
                        }
                        SidecarReadMode::RandomId | SidecarReadMode::TidSorted => {
                            let _ = rerank_with_sidecar_db(
                                &client,
                                &sidecar,
                                *read_mode,
                                &sidecar_table_name(&args.prefix, sidecar.variant),
                                &warmup_candidate_run.ids,
                                warmup_queries,
                                args.concurrency,
                            )
                            .await?;
                        }
                    }
                }
            }
        }
        for (value, candidate_run) in &candidate_runs {
            for read_mode in &read_modes {
                let reranked = match read_mode {
                    SidecarReadMode::Free => rerank_with_sidecar(
                        &sidecar,
                        &candidate_run.ids,
                        &id_to_pos,
                        &corpus,
                        &queries,
                    )?,
                    SidecarReadMode::RandomId | SidecarReadMode::TidSorted => {
                        rerank_with_sidecar_db(
                            &client,
                            &sidecar,
                            *read_mode,
                            &sidecar_table_name(&args.prefix, sidecar.variant),
                            &candidate_run.ids,
                            &queries,
                            args.concurrency,
                        )
                        .await?
                    }
                };
                let recall = recall_summary_at_k(&truth_ids, &reranked.predictions, args.k);
                let ndcg = ndcg_at_k_from_sources(
                    &truth_gt.scores,
                    &reranked.predictions,
                    &corpus_ids,
                    &corpus,
                    &queries,
                    args.k,
                );
                let total_ns: Vec<u128> = candidate_run
                    .elapsed_ns
                    .iter()
                    .zip(reranked.elapsed_ns.iter())
                    .map(|(candidate, sidecar)| candidate + sidecar)
                    .collect();
                let candidate_summary = summarize_ns(&candidate_run.elapsed_ns);
                let sidecar_io_summary = summarize_ns(&reranked.io_elapsed_ns);
                let sidecar_score_summary = summarize_ns(&reranked.score_elapsed_ns);
                let sidecar_summary = summarize_ns(&reranked.elapsed_ns);
                let total_summary = summarize_ns(&total_ns);
                table.add_row(vec![
                    Cell::new(value),
                    Cell::new(sidecar.variant.label()),
                    Cell::new(read_mode.label()),
                    Cell::new(recall.queries),
                    Cell::new(args.candidate_k),
                    Cell::new(args.concurrency),
                    Cell::new(format!("{:.4}", recall.recall)),
                    Cell::new(format!("{:.4}", recall.p10)),
                    Cell::new(format!("{:.4}", recall.p50)),
                    Cell::new(format!("{:.4}", recall.p90)),
                    Cell::new(format!("{:.4}", ndcg)),
                    Cell::new(format_ms(candidate_summary.p50_ms)),
                    Cell::new(format_ms(sidecar_io_summary.p50_ms)),
                    Cell::new(format_ms(sidecar_score_summary.p50_ms)),
                    Cell::new(format_ms(sidecar_summary.p50_ms)),
                    Cell::new(format_ms(total_summary.p50_ms)),
                    Cell::new(format_ms(candidate_summary.p95_ms)),
                    Cell::new(format_ms(sidecar_io_summary.p95_ms)),
                    Cell::new(format_ms(sidecar_score_summary.p95_ms)),
                    Cell::new(format_ms(sidecar_summary.p95_ms)),
                    Cell::new(format_ms(total_summary.p95_ms)),
                    Cell::new(format_ms(candidate_summary.p99_ms)),
                    Cell::new(format_ms(sidecar_io_summary.p99_ms)),
                    Cell::new(format_ms(sidecar_score_summary.p99_ms)),
                    Cell::new(format_ms(sidecar_summary.p99_ms)),
                    Cell::new(format_ms(total_summary.p99_ms)),
                    Cell::new(sidecar.bytes_per_vector),
                    Cell::new(format_bytes(sidecar.total_bytes(corpus.nrows()))),
                ]);
            }
        }
    }

    let output = table.to_string();
    println!("{output}");
    if let Some(path) = args.log_output {
        write_log(&path, &output).await?;
    }
    Ok(())
}

async fn validate_index_shape(
    client: &Client,
    corpus_table: &str,
    profile: &IndexProfile,
    allow_unsafe: bool,
) -> Result<()> {
    let rows = client
        .query(
            "SELECT i.relname, COALESCE(i.reloptions, ARRAY[]::text[])
             FROM pg_class t
             JOIN pg_index ix ON ix.indrelid = t.oid
             JOIN pg_class i  ON i.oid = ix.indexrelid
             JOIN pg_am pam ON pam.oid = i.relam
             WHERE t.relname = $1
               AND pam.amname = $2
             ORDER BY i.relname",
            &[&corpus_table, &profile.access_method],
        )
        .await
        .wrap_err_with(|| format!("checking {corpus_table} {} indexes", profile.access_method))?;
    if rows.is_empty() {
        bail!(
            "{} on {:?}",
            super::missing_am_error(profile, profile.access_method),
            corpus_table
        );
    }
    let off_indexes = rows
        .iter()
        .filter(|row| {
            let reloptions: Vec<String> = row.get(1);
            reloptions.iter().any(|opt| opt == "rerank=off")
        })
        .count();
    if allow_unsafe {
        return Ok(());
    }
    if rows.len() != 1 || off_indexes != 1 {
        bail!(
            "sidecar-rerank requires one isolated {} index on {corpus_table} with rerank=off; found {} index(es), {} rerank=off",
            profile.access_method,
            rows.len(),
            off_indexes
        );
    }
    Ok(())
}

fn validate_corpus_and_queries(
    table: &str,
    corpus: &Array2<f32>,
    queries: &Array2<f32>,
) -> Result<()> {
    if corpus.nrows() == 0 {
        bail!("corpus table {table} is empty");
    }
    if corpus.ncols() == 0 || corpus.ncols() != queries.ncols() {
        bail!(
            "{table}: corpus dim {} does not match query dim {}",
            corpus.ncols(),
            queries.ncols()
        );
    }
    Ok(())
}

struct CandidateRun {
    ids: Vec<Vec<i64>>,
    elapsed_ns: Vec<u128>,
}

async fn collect_candidates(
    client: &Client,
    stmt: &tokio_postgres::Statement,
    profile: &IndexProfile,
    queries: &Array2<f32>,
    bits: i32,
    seed: i64,
    candidate_k: usize,
) -> Result<CandidateRun> {
    let mut ids = Vec::with_capacity(queries.nrows());
    let mut elapsed_ns = Vec::with_capacity(queries.nrows());
    for q in 0..queries.nrows() {
        let query_vec: Vec<f32> = queries.row(q).to_vec();
        let started = Instant::now();
        let rows = if profile.encode_scan_query {
            client
                .query(stmt, &[&query_vec, &bits, &seed, &(candidate_k as i64)])
                .await
        } else {
            client
                .query(stmt, &[&query_vec, &(candidate_k as i64)])
                .await
        }
        .wrap_err("executing sidecar-rerank candidate query")?;
        elapsed_ns.push(started.elapsed().as_nanos());
        ids.push(rows.iter().map(|row| row.get::<_, i64>(0)).collect());
    }
    Ok(CandidateRun { ids, elapsed_ns })
}

struct Sidecar {
    variant: SidecarVariant,
    bytes_per_vector: usize,
    storage: SidecarStorage,
}

impl Sidecar {
    fn total_bytes(&self, rows: usize) -> usize {
        self.bytes_per_vector.saturating_mul(rows)
    }
}

enum SidecarStorage {
    F32,
    F16(Vec<Vec<f16>>),
    Rabitq8 {
        quantizer: Arc<RaBitQQuantizer>,
        codes: Vec<Box<[u8]>>,
    },
}

fn sidecar_table_name(prefix: &str, variant: SidecarVariant) -> String {
    format!("{prefix}_sidecar_{}", variant.label())
}

async fn ensure_sidecar_tables(
    client: &Client,
    prefix: &str,
    sidecars: &[Sidecar],
    corpus_ids: &[i64],
    corpus: &Array2<f32>,
    rebuild: bool,
) -> Result<()> {
    for sidecar in sidecars {
        let table = sidecar_table_name(prefix, sidecar.variant);
        profiles::validate_ident(&table)
            .wrap_err_with(|| format!("invalid sidecar table name {table:?}"))?;
        if rebuild {
            eprintln!("[sidecar-rerank] dropping {table} before rebuild ...");
            client
                .batch_execute(&format!("DROP TABLE IF EXISTS {table}"))
                .await
                .wrap_err_with(|| format!("dropping sidecar table {table}"))?;
        }
        client
            .batch_execute(&format!(
                "CREATE UNLOGGED TABLE IF NOT EXISTS {table} (
                    id bigint PRIMARY KEY,
                    payload bytea NOT NULL CHECK (octet_length(payload) = {})
                ) WITH (fillfactor = 100);
                 ALTER TABLE {table} ALTER COLUMN payload SET STORAGE PLAIN",
                sidecar.bytes_per_vector
            ))
            .await
            .wrap_err_with(|| format!("creating sidecar table {table}"))?;

        let count: i64 = client
            .query_one(&format!("SELECT count(*)::bigint FROM {table}"), &[])
            .await
            .wrap_err_with(|| format!("counting sidecar table {table}"))?
            .get(0);
        if count == corpus.nrows() as i64 {
            eprintln!(
                "[sidecar-rerank] {table} already has {} rows; keeping existing table",
                count
            );
            continue;
        }
        eprintln!(
            "[sidecar-rerank] populating {table}: existing rows {}, target rows {} ...",
            count,
            corpus.nrows()
        );
        client
            .batch_execute(&format!("BEGIN; TRUNCATE {table}"))
            .await
            .wrap_err_with(|| format!("starting sidecar table load for {table}"))?;
        if let Err(err) = copy_sidecar_rows(client, &table, sidecar, corpus_ids, corpus).await {
            let _ = client.batch_execute("ROLLBACK").await;
            return Err(err).wrap_err_with(|| format!("copying sidecar rows into {table}"));
        }
        client
            .batch_execute(&format!("COMMIT; ANALYZE {table}"))
            .await
            .wrap_err_with(|| format!("finishing sidecar table load for {table}"))?;
    }
    Ok(())
}

async fn copy_sidecar_rows(
    client: &Client,
    table: &str,
    sidecar: &Sidecar,
    corpus_ids: &[i64],
    corpus: &Array2<f32>,
) -> Result<()> {
    const COPY_CHUNK_TARGET: usize = 8 * 1024 * 1024;

    let sink = client
        .copy_in::<_, Bytes>(&format!("COPY {table} (id, payload) FROM STDIN BINARY"))
        .await
        .wrap_err_with(|| format!("starting binary COPY for {table}"))?;
    futures::pin_mut!(sink);

    let mut chunk = Vec::with_capacity(COPY_CHUNK_TARGET + sidecar.bytes_per_vector + 32);
    chunk.extend_from_slice(b"PGCOPY\n\xff\r\n\0");
    chunk.extend_from_slice(&0_i32.to_be_bytes()); // flags
    chunk.extend_from_slice(&0_i32.to_be_bytes()); // header extension length

    for (pos, id) in corpus_ids.iter().enumerate() {
        let payload = sidecar_payload_bytes(sidecar, corpus, pos)?;
        append_copy_row(&mut chunk, *id, &payload);
        if chunk.len() >= COPY_CHUNK_TARGET {
            let full = std::mem::replace(
                &mut chunk,
                Vec::with_capacity(COPY_CHUNK_TARGET + sidecar.bytes_per_vector + 32),
            );
            sink.send(Bytes::from(full))
                .await
                .wrap_err_with(|| format!("sending binary COPY chunk for {table}"))?;
        }
    }

    chunk.extend_from_slice(&(-1_i16).to_be_bytes());
    sink.send(Bytes::from(chunk))
        .await
        .wrap_err_with(|| format!("sending final binary COPY chunk for {table}"))?;
    sink.finish()
        .await
        .wrap_err_with(|| format!("finishing binary COPY for {table}"))?;
    Ok(())
}

fn append_copy_row(out: &mut Vec<u8>, id: i64, payload: &[u8]) {
    out.extend_from_slice(&2_i16.to_be_bytes());
    out.extend_from_slice(&8_i32.to_be_bytes());
    out.extend_from_slice(&id.to_be_bytes());
    out.extend_from_slice(&(payload.len() as i32).to_be_bytes());
    out.extend_from_slice(payload);
}

fn sidecar_payload_bytes(sidecar: &Sidecar, corpus: &Array2<f32>, pos: usize) -> Result<Vec<u8>> {
    match &sidecar.storage {
        SidecarStorage::F32 => Ok(corpus
            .row(pos)
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()),
        SidecarStorage::F16(encoded) => Ok(encoded[pos]
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()),
        SidecarStorage::Rabitq8 { codes, .. } => Ok(codes[pos].to_vec()),
    }
}

fn build_sidecar(variant: SidecarVariant, corpus: &Array2<f32>, seed: u64) -> Result<Sidecar> {
    match variant {
        SidecarVariant::F32 => Ok(Sidecar {
            variant,
            bytes_per_vector: corpus.ncols() * std::mem::size_of::<f32>(),
            storage: SidecarStorage::F32,
        }),
        SidecarVariant::F16 => {
            let encoded = corpus
                .rows()
                .into_iter()
                .map(|row| row.iter().map(|value| f16::from_f32(*value)).collect())
                .collect();
            Ok(Sidecar {
                variant,
                bytes_per_vector: corpus.ncols() * std::mem::size_of::<f16>(),
                storage: SidecarStorage::F16(encoded),
            })
        }
        SidecarVariant::Rabitq8
        | SidecarVariant::Rabitq8ls
        | SidecarVariant::Rabitq8c3
        | SidecarVariant::Rabitq8c4 => {
            let prod = ProdQuantizer::cached(corpus.ncols(), 4, seed);
            let clip = variant.rabitq_clip().expect("RaBitQ variant has a clip");
            let quantizer = Arc::new(
                RaBitQQuantizer::with_srht_bits_clip(corpus.ncols(), prod, 8, clip).map_err(
                    |err| eyre!("building bits=8 RaBitQ sidecar with clip {clip}: {err}"),
                )?,
            );
            let bytes_per_vector = <RaBitQQuantizer as Quantizer>::code_len(quantizer.as_ref());
            let codes = corpus
                .rows()
                .into_iter()
                .map(|row| {
                    let values: Vec<f32> = row.to_vec();
                    <RaBitQQuantizer as Quantizer>::encode_code(quantizer.as_ref(), &values)
                })
                .collect();
            Ok(Sidecar {
                variant,
                bytes_per_vector,
                storage: SidecarStorage::Rabitq8 { quantizer, codes },
            })
        }
    }
}

struct RerankRun {
    predictions: Vec<Vec<i64>>,
    io_elapsed_ns: Vec<u128>,
    score_elapsed_ns: Vec<u128>,
    elapsed_ns: Vec<u128>,
}

fn rerank_with_sidecar(
    sidecar: &Sidecar,
    candidates: &[Vec<i64>],
    id_to_pos: &HashMap<i64, usize>,
    corpus: &Array2<f32>,
    queries: &Array2<f32>,
) -> Result<RerankRun> {
    let mut predictions = Vec::with_capacity(candidates.len());
    let mut elapsed_ns = Vec::with_capacity(candidates.len());
    for (q, ids) in candidates.iter().enumerate() {
        let query = queries.row(q).to_vec();
        let started = Instant::now();
        let mut scored = Vec::with_capacity(ids.len());
        match &sidecar.storage {
            SidecarStorage::F32 => {
                for id in ids {
                    let pos = *id_to_pos.get(id).ok_or_else(|| {
                        eyre!("candidate id {id} not present in corpus source map")
                    })?;
                    let score = dot_f32(&query, corpus, pos);
                    scored.push((*id, score));
                }
            }
            SidecarStorage::F16(encoded) => {
                for id in ids {
                    let pos = *id_to_pos.get(id).ok_or_else(|| {
                        eyre!("candidate id {id} not present in corpus source map")
                    })?;
                    scored.push((*id, dot_f16(&query, &encoded[pos])));
                }
            }
            SidecarStorage::Rabitq8 { quantizer, codes } => {
                let prepared = quantizer.prepare_estimator(&query);
                if matches!(
                    sidecar.variant,
                    SidecarVariant::Rabitq8 | SidecarVariant::Rabitq8c3 | SidecarVariant::Rabitq8c4
                ) {
                    let mut slab = Vec::with_capacity(ids.len() * sidecar.bytes_per_vector);
                    for id in ids {
                        let pos = *id_to_pos.get(id).ok_or_else(|| {
                            eyre!("candidate id {id} not present in corpus source map")
                        })?;
                        slab.extend_from_slice(&codes[pos]);
                    }
                    let mut scores = Vec::new();
                    prepared
                        .estimate_ip_batch(&slab, sidecar.bytes_per_vector, &mut scores)
                        .map_err(|err| {
                            eyre!("{} batch scoring failed: {err}", sidecar.variant.label())
                        })?;
                    for (id, score) in ids.iter().copied().zip(scores) {
                        scored.push((id, score));
                    }
                } else {
                    for id in ids {
                        let pos = *id_to_pos.get(id).ok_or_else(|| {
                            eyre!("candidate id {id} not present in corpus source map")
                        })?;
                        let score = rabitq_sidecar_score(sidecar.variant, &prepared, &codes[pos]);
                        scored.push((*id, score));
                    }
                }
            }
        }
        scored.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        predictions.push(scored.into_iter().map(|(id, _)| id).collect());
        let elapsed = started.elapsed().as_nanos();
        elapsed_ns.push(elapsed);
    }
    let io_elapsed_ns = vec![0; elapsed_ns.len()];
    let score_elapsed_ns = elapsed_ns.clone();
    Ok(RerankRun {
        predictions,
        io_elapsed_ns,
        score_elapsed_ns,
        elapsed_ns,
    })
}

async fn rerank_with_sidecar_db(
    client: &Client,
    sidecar: &Sidecar,
    read_mode: SidecarReadMode,
    table: &str,
    candidates: &[Vec<i64>],
    queries: &Array2<f32>,
    concurrency: usize,
) -> Result<RerankRun> {
    let random_stmt = if matches!(read_mode, SidecarReadMode::RandomId) {
        Some(
            client
                .prepare(&format!("SELECT payload FROM {table} WHERE id = $1"))
                .await
                .wrap_err_with(|| format!("preparing random-id sidecar fetch for {table}"))?,
        )
    } else {
        None
    };
    let tid_sorted_stmt = if matches!(read_mode, SidecarReadMode::TidSorted) {
        Some(
            client
                .prepare(&format!(
                    "SELECT id, payload FROM {table}
                     WHERE id = ANY($1::bigint[])
                     ORDER BY ctid"
                ))
                .await
                .wrap_err_with(|| format!("preparing tid-sorted sidecar fetch for {table}"))?,
        )
    } else {
        None
    };

    let results: Vec<_> = stream::iter(candidates.iter().enumerate())
        .map(|(q, ids)| {
            let query = queries.row(q).to_vec();
            rerank_one_sidecar_db_query(
                client,
                sidecar,
                read_mode,
                table,
                random_stmt.as_ref(),
                tid_sorted_stmt.as_ref(),
                ids,
                query,
            )
        })
        .buffered(concurrency)
        .try_collect()
        .await?;

    let mut predictions = Vec::with_capacity(results.len());
    let mut io_elapsed_ns = Vec::with_capacity(results.len());
    let mut score_elapsed_ns = Vec::with_capacity(results.len());
    let mut elapsed_ns = Vec::with_capacity(results.len());
    for result in results {
        predictions.push(result.prediction);
        io_elapsed_ns.push(result.io_elapsed_ns);
        score_elapsed_ns.push(result.score_elapsed_ns);
        elapsed_ns.push(result.elapsed_ns);
    }

    Ok(RerankRun {
        predictions,
        io_elapsed_ns,
        score_elapsed_ns,
        elapsed_ns,
    })
}

struct RerankQueryResult {
    prediction: Vec<i64>,
    io_elapsed_ns: u128,
    score_elapsed_ns: u128,
    elapsed_ns: u128,
}

async fn rerank_one_sidecar_db_query(
    client: &Client,
    sidecar: &Sidecar,
    read_mode: SidecarReadMode,
    table: &str,
    random_stmt: Option<&tokio_postgres::Statement>,
    tid_sorted_stmt: Option<&tokio_postgres::Statement>,
    ids: &[i64],
    query: Vec<f32>,
) -> Result<RerankQueryResult> {
    let total_started = Instant::now();
    let io_started = Instant::now();
    let fetched = match read_mode {
        SidecarReadMode::Free => unreachable!("free sidecar mode does not use DB fetch"),
        SidecarReadMode::RandomId => {
            let stmt = random_stmt.expect("random statement prepared");
            let mut fetched = Vec::with_capacity(ids.len());
            for id in ids {
                let row = client
                    .query_opt(stmt, &[id])
                    .await
                    .wrap_err_with(|| format!("fetching sidecar payload id {id} from {table}"))?
                    .ok_or_else(|| eyre!("sidecar table {table} missing id {id}"))?;
                fetched.push((*id, row.get::<_, Vec<u8>>(0)));
            }
            fetched
        }
        SidecarReadMode::TidSorted => {
            let stmt = tid_sorted_stmt.expect("tid-sorted statement prepared");
            client
                .query(stmt, &[&ids])
                .await
                .wrap_err_with(|| format!("fetching tid-sorted sidecar payloads from {table}"))?
                .into_iter()
                .map(|row| (row.get::<_, i64>(0), row.get::<_, Vec<u8>>(1)))
                .collect()
        }
    };
    let io_elapsed_ns = io_started.elapsed().as_nanos();
    if fetched.len() != ids.len() {
        bail!(
            "sidecar table {table} returned {} rows for {} candidate ids",
            fetched.len(),
            ids.len()
        );
    }

    let score_started = Instant::now();
    let mut scored = score_sidecar_payloads(sidecar, &query, &fetched)?;
    scored.sort_unstable_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    let score_elapsed_ns = score_started.elapsed().as_nanos();
    Ok(RerankQueryResult {
        prediction: scored.into_iter().map(|(id, _)| id).collect(),
        io_elapsed_ns,
        score_elapsed_ns,
        elapsed_ns: total_started.elapsed().as_nanos(),
    })
}

fn score_sidecar_payloads(
    sidecar: &Sidecar,
    query: &[f32],
    payloads: &[(i64, Vec<u8>)],
) -> Result<Vec<(i64, f32)>> {
    let mut scored = Vec::with_capacity(payloads.len());
    match &sidecar.storage {
        SidecarStorage::F32 => {
            for (id, payload) in payloads {
                scored.push((*id, dot_f32_bytes(query, payload)?));
            }
        }
        SidecarStorage::F16(_) => {
            for (id, payload) in payloads {
                scored.push((*id, dot_f16_bytes(query, payload)?));
            }
        }
        SidecarStorage::Rabitq8 { quantizer, .. } => {
            let prepared = quantizer.prepare_estimator(query);
            if matches!(
                sidecar.variant,
                SidecarVariant::Rabitq8 | SidecarVariant::Rabitq8c3 | SidecarVariant::Rabitq8c4
            ) {
                let mut slab = Vec::with_capacity(payloads.len() * sidecar.bytes_per_vector);
                for (id, payload) in payloads {
                    if payload.len() != sidecar.bytes_per_vector {
                        bail!(
                            "{} sidecar payload for id {id} has {} bytes, expected {}",
                            sidecar.variant.label(),
                            payload.len(),
                            sidecar.bytes_per_vector
                        );
                    }
                    slab.extend_from_slice(payload);
                }
                let mut scores = Vec::new();
                prepared
                    .estimate_ip_batch(&slab, sidecar.bytes_per_vector, &mut scores)
                    .map_err(|err| {
                        eyre!("{} batch scoring failed: {err}", sidecar.variant.label())
                    })?;
                for ((id, _), score) in payloads.iter().zip(scores) {
                    scored.push((*id, score));
                }
            } else {
                for (id, payload) in payloads {
                    if payload.len() != sidecar.bytes_per_vector {
                        bail!(
                            "{} sidecar payload for id {id} has {} bytes, expected {}",
                            sidecar.variant.label(),
                            payload.len(),
                            sidecar.bytes_per_vector
                        );
                    }
                    let score = rabitq_sidecar_score(sidecar.variant, &prepared, payload);
                    scored.push((*id, score));
                }
            }
        }
    }
    Ok(scored)
}

fn rabitq_sidecar_score(
    variant: SidecarVariant,
    prepared: &ecaz::bench_api::PreparedEstimator,
    code: &[u8],
) -> f32 {
    match variant {
        SidecarVariant::Rabitq8 | SidecarVariant::Rabitq8c3 | SidecarVariant::Rabitq8c4 => {
            prepared.estimate_ip(code).estimate
        }
        SidecarVariant::Rabitq8ls => prepared.estimate_ip_least_squares_scalar_only(code),
        SidecarVariant::F32 | SidecarVariant::F16 => unreachable!("not a RaBitQ sidecar variant"),
    }
}

fn dot_f32(query: &[f32], corpus: &Array2<f32>, pos: usize) -> f32 {
    query
        .iter()
        .zip(corpus.row(pos).iter())
        .map(|(left, right)| *left * *right)
        .sum()
}

fn dot_f16(query: &[f32], source: &[f16]) -> f32 {
    query
        .iter()
        .zip(source.iter())
        .map(|(left, right)| *left * right.to_f32())
        .sum()
}

fn dot_f32_bytes(query: &[f32], payload: &[u8]) -> Result<f32> {
    let expected = query.len() * std::mem::size_of::<f32>();
    if payload.len() != expected {
        bail!(
            "f32 sidecar payload has {} bytes, expected {}",
            payload.len(),
            expected
        );
    }
    let mut sum = 0.0;
    for (left, bytes) in query.iter().zip(payload.chunks_exact(4)) {
        let right = f32::from_le_bytes(bytes.try_into().expect("validated f32 chunk"));
        sum += *left * right;
    }
    Ok(sum)
}

fn dot_f16_bytes(query: &[f32], payload: &[u8]) -> Result<f32> {
    let expected = query.len() * std::mem::size_of::<f16>();
    if payload.len() != expected {
        bail!(
            "f16 sidecar payload has {} bytes, expected {}",
            payload.len(),
            expected
        );
    }
    let mut sum = 0.0;
    for (left, bytes) in query.iter().zip(payload.chunks_exact(2)) {
        let right = f16::from_le_bytes(bytes.try_into().expect("validated f16 chunk"));
        sum += *left * right.to_f32();
    }
    Ok(sum)
}

#[derive(Clone, Copy)]
struct LatencySummary {
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
}

fn summarize_ns(values: &[u128]) -> LatencySummary {
    if values.is_empty() {
        return LatencySummary {
            p50_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
        };
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    LatencySummary {
        p50_ms: ns_to_ms(percentile_u128(&sorted, 0.50)),
        p95_ms: ns_to_ms(percentile_u128(&sorted, 0.95)),
        p99_ms: ns_to_ms(percentile_u128(&sorted, 0.99)),
    }
}

fn percentile_u128(sorted: &[u128], p: f64) -> u128 {
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn ns_to_ms(ns: u128) -> f64 {
    ns as f64 / 1_000_000.0
}

fn format_ms(value: f64) -> String {
    format!("{value:.3} ms")
}

fn format_bytes(bytes: usize) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.2} GiB", b / GIB)
    } else if b >= MIB {
        format!("{:.2} MiB", b / MIB)
    } else if b >= KIB {
        format!("{:.2} KiB", b / KIB)
    } else {
        format!("{bytes} B")
    }
}

async fn write_log(path: &Path, output: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    tokio::fs::write(path, format!("{output}\n"))
        .await
        .wrap_err_with(|| format!("writing {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_f32_bytes_matches_plain_dot() {
        let query = [1.0, -2.0, 0.5];
        let source = [0.25_f32, 0.75, -4.0];
        let payload: Vec<u8> = source
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();

        let score = dot_f32_bytes(&query, &payload).unwrap();
        assert_eq!(score, -3.25);
    }

    #[test]
    fn dot_f16_bytes_matches_half_precision_dot() {
        let query = [1.0, -2.0, 0.5];
        let source = [
            f16::from_f32(0.25),
            f16::from_f32(0.75),
            f16::from_f32(-4.0),
        ];
        let payload: Vec<u8> = source
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();

        let score = dot_f16_bytes(&query, &payload).unwrap();
        assert_eq!(score, -3.25);
    }

    #[test]
    fn sidecar_read_mode_labels_distinguish_db_modes() {
        assert_eq!(SidecarReadMode::Free.label(), "free");
        assert!(!SidecarReadMode::Free.uses_db());
        assert_eq!(SidecarReadMode::RandomId.label(), "random-id");
        assert!(SidecarReadMode::RandomId.uses_db());
        assert_eq!(SidecarReadMode::TidSorted.label(), "tid-sorted");
        assert!(SidecarReadMode::TidSorted.uses_db());
    }
}
