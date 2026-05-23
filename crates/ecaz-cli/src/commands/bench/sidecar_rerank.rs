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

use clap::{Args, ValueEnum};
use color_eyre::eyre::{bail, eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use half::f16;
use ndarray::Array2;
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
}

impl SidecarVariant {
    fn label(self) -> &'static str {
        match self {
            Self::F32 => "f32",
            Self::F16 => "f16",
            Self::Rabitq8 => "rabitq8",
        }
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
    /// Sweep values for the profile tuning GUC.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// Cap the query set.
    #[arg(long)]
    pub queries_limit: Option<usize>,
    /// Quantization bits used when encoding query vectors at scan time.
    #[arg(long, default_value_t = 4)]
    pub bits: i32,
    /// Quantizer seed.
    #[arg(long, default_value_t = 42)]
    pub seed: i64,
    /// Sidecar variants to evaluate. Omit to run f32, f16, and rabitq8.
    #[arg(long, value_enum)]
    pub variant: Vec<SidecarVariant>,
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
    let sidecars = build_sidecars(&variants, &corpus, args.seed as u64)?;
    let knn_sql = super::recall::build_knn_sql(profile, &corpus_table);
    let stmt = client
        .prepare(&knn_sql)
        .await
        .wrap_err("preparing sidecar-rerank candidate statement")?;

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        profile.sweep_axis_label(),
        "variant",
        "queries",
        "candidate_k",
        "recall@k",
        "recall_p10",
        "recall_p50",
        "recall_p90",
        "ndcg@k",
        "candidate_sql_p50",
        "sidecar_p50",
        "total_bound_p50",
        "candidate_sql_p95",
        "sidecar_p95",
        "total_bound_p95",
        "candidate_sql_p99",
        "sidecar_p99",
        "total_bound_p99",
        "sidecar_bytes_per_vector",
        "sidecar_size",
    ]);

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
        for sidecar in &sidecars {
            let reranked =
                rerank_with_sidecar(sidecar, &candidate_run.ids, &id_to_pos, &corpus, &queries)?;
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
            let sidecar_summary = summarize_ns(&reranked.elapsed_ns);
            let total_summary = summarize_ns(&total_ns);
            table.add_row(vec![
                Cell::new(value),
                Cell::new(sidecar.variant.label()),
                Cell::new(recall.queries),
                Cell::new(args.candidate_k),
                Cell::new(format!("{:.4}", recall.recall)),
                Cell::new(format!("{:.4}", recall.p10)),
                Cell::new(format!("{:.4}", recall.p50)),
                Cell::new(format!("{:.4}", recall.p90)),
                Cell::new(format!("{:.4}", ndcg)),
                Cell::new(format_ms(candidate_summary.p50_ms)),
                Cell::new(format_ms(sidecar_summary.p50_ms)),
                Cell::new(format_ms(total_summary.p50_ms)),
                Cell::new(format_ms(candidate_summary.p95_ms)),
                Cell::new(format_ms(sidecar_summary.p95_ms)),
                Cell::new(format_ms(total_summary.p95_ms)),
                Cell::new(format_ms(candidate_summary.p99_ms)),
                Cell::new(format_ms(sidecar_summary.p99_ms)),
                Cell::new(format_ms(total_summary.p99_ms)),
                Cell::new(sidecar.bytes_per_vector),
                Cell::new(format_bytes(sidecar.total_bytes(corpus.nrows()))),
            ]);
        }
    }
    client
        .batch_execute(&format!("RESET {scan_guc}"))
        .await
        .wrap_err_with(|| format!("RESET {scan_guc}"))?;

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

fn build_sidecars(
    variants: &[SidecarVariant],
    corpus: &Array2<f32>,
    seed: u64,
) -> Result<Vec<Sidecar>> {
    let mut out = Vec::new();
    for variant in variants {
        match variant {
            SidecarVariant::F32 => out.push(Sidecar {
                variant: *variant,
                bytes_per_vector: corpus.ncols() * std::mem::size_of::<f32>(),
                storage: SidecarStorage::F32,
            }),
            SidecarVariant::F16 => {
                let encoded = corpus
                    .rows()
                    .into_iter()
                    .map(|row| row.iter().map(|value| f16::from_f32(*value)).collect())
                    .collect();
                out.push(Sidecar {
                    variant: *variant,
                    bytes_per_vector: corpus.ncols() * std::mem::size_of::<f16>(),
                    storage: SidecarStorage::F16(encoded),
                });
            }
            SidecarVariant::Rabitq8 => {
                let prod = ProdQuantizer::cached(corpus.ncols(), 4, seed);
                let quantizer = Arc::new(
                    RaBitQQuantizer::with_srht_bits(corpus.ncols(), prod, 8)
                        .map_err(|err| eyre!("building bits=8 RaBitQ sidecar: {err}"))?,
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
                out.push(Sidecar {
                    variant: *variant,
                    bytes_per_vector,
                    storage: SidecarStorage::Rabitq8 { quantizer, codes },
                });
            }
        }
    }
    Ok(out)
}

struct RerankRun {
    predictions: Vec<Vec<i64>>,
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
                for id in ids {
                    let pos = *id_to_pos.get(id).ok_or_else(|| {
                        eyre!("candidate id {id} not present in corpus source map")
                    })?;
                    let score = quantizer.estimate_ip(&prepared, &codes[pos]).estimate;
                    scored.push((*id, score));
                }
            }
        }
        scored.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        predictions.push(scored.into_iter().map(|(id, _)| id).collect());
        elapsed_ns.push(started.elapsed().as_nanos());
    }
    Ok(RerankRun {
        predictions,
        elapsed_ns,
    })
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
