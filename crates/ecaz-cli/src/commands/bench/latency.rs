//! `ecaz bench latency` — wall-clock p50/p95/p99 for KNN SQL at k.
//!
//! # Flow
//!
//! 1. Connect, validate profile + prefix + tuning GUC.
//! 2. Load `--iterations` query vectors from `<prefix>_queries.source`
//!    (round-robined if iterations > queries).
//! 3. For every requested concurrency level, spawn that many workers; each
//!    pulls from a shared counter and runs the same prepared KNN statement.
//! 4. Merge per-worker duration buffers, emit one comfy-table row per
//!    tuning/concurrency point: count, mean, stddev, min, p50, p95, p99, max,
//!    concurrent wall time, and QPS.
//!
//! # Purity boundary
//!
//! `percentile` and `summarize` are pure functions over `&[Duration]`.
//! The orchestration (`run`) is a thin DB shell on top; live-Postgres
//! coverage lands with the integration suite.

use clap::{Args, ValueEnum};
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::profiles;
use crate::psql::{self, ConnectionOptions};

use super::recall::build_knn_sql;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum DistannPayloadShape {
    IdOnly,
    NarrowScalar,
    VectorBearing,
    Toasted,
}

#[derive(Args, Debug)]
pub struct LatencyArgs {
    /// Prefix identifying the corpus.
    #[arg(long)]
    pub prefix: String,
    /// Access-method profile to measure.
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,
    /// k for KNN ORDER BY ... LIMIT k.
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    /// Number of concurrent worker connections.
    #[arg(long, default_value_t = 1)]
    pub concurrency: usize,
    /// Concurrency levels to measure at every tuning sweep point. Accepts
    /// `--concurrency-sweep 1,2,4` or repeated flags. When present, this
    /// overrides the single `--concurrency` value.
    #[arg(long, value_delimiter = ',')]
    pub concurrency_sweep: Vec<usize>,
    /// Total number of queries to run per tuning/concurrency point.
    #[arg(long, default_value_t = 1000)]
    pub iterations: usize,
    /// Untimed queries to run on each worker connection before measurement.
    /// Use this to populate backend-local caches without contaminating latency
    /// samples; zero preserves the historical behavior.
    #[arg(long, default_value_t = 0)]
    pub warmup_iterations: usize,
    /// Reconnect each worker after this many timed queries. Zero preserves the
    /// historical single-backend run and nonzero values bound backend-local
    /// memory retained by long physical-query diagnostics.
    #[arg(long, default_value_t = 0)]
    pub worker_batch_size: usize,
    /// Keep the single worker backend in one explicit transaction for the
    /// timed queries, then commit before collecting the final memory sample.
    /// This is a diagnostic mode for transaction-lifetime retention.
    #[arg(long, default_value_t = false)]
    pub hold_transaction: bool,
    /// Sweep values for the profile's tuning axis. Accepts `--sweep 100,200`
    /// or repeated `--sweep 100 --sweep 200`.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// IVF-only: session override for heap-f32 rerank frontier width.
    /// Use -1 for the index reloption, 0 for the full probed frontier.
    #[arg(long)]
    pub rerank_width: Option<i32>,
    /// IVF/SPIRE: enable deterministic adaptive nprobe during the sweep.
    #[arg(long)]
    pub adaptive_nprobe: bool,
    /// IVF/SPIRE: score-gap threshold for adaptive nprobe decisions.
    #[arg(long)]
    pub adaptive_nprobe_score_gap_micros: Option<i32>,
    /// IVF-only: score margin-ratio threshold, in basis points, for adaptive nprobe decisions.
    #[arg(long)]
    pub adaptive_nprobe_score_margin_ratio_bps: Option<i32>,
    /// IVF-only: enable experimental posting scratch SoA batch decode.
    #[arg(long)]
    pub ivf_scratch_soa_batch_decode: bool,
    /// Quantization bits used when encoding query vectors (must match loader).
    #[arg(long, default_value_t = 4)]
    pub bits: i32,
    /// Quantizer seed (must match loader).
    #[arg(long, default_value_t = 42)]
    pub seed: i64,
    /// Force benchmark queries onto the index path by disabling sequential scans.
    #[arg(long)]
    pub force_index: bool,
    /// Sample each worker backend's /proc status while the latency sweep runs.
    #[arg(long)]
    pub sample_backend_memory: bool,
    /// Operator-supplied cache-state label recorded with each latency row.
    #[arg(long, default_value = "unspecified")]
    pub cache_state: String,
    /// Extra session GUCs to set on every worker connection, as name=value.
    #[arg(long = "session-guc")]
    pub session_gucs: Vec<String>,
    /// Reset and snapshot Task 87 CandidateBatch scoring counters on each worker connection.
    #[arg(long)]
    pub task87_candidate_batch_counters: bool,
    /// Task 133: reset and snapshot IVF query-stage latency counters on each worker connection.
    #[arg(long)]
    pub ivf_stage_counters: bool,
    /// Task 183: reset and snapshot physical ec_distann query-stage counters on
    /// each worker connection. Requires the benchmark measurement extension.
    #[arg(long)]
    pub distann_stage_counters: bool,
    /// Task 224 owner-payload projection shape. This is benchmark-only and is
    /// accepted only with the ec_distann profile.
    #[arg(long, value_enum)]
    pub distann_payload_shape: Option<DistannPayloadShape>,
    /// Milliseconds between backend RSS/HWM samples when --sample-backend-memory is set.
    #[arg(long, default_value_t = 25)]
    pub memory_sample_interval_ms: u64,
    /// Stream backend RSS/HWM samples to this file while the sweep runs.
    #[arg(long)]
    pub memory_series_output: Option<PathBuf>,
    /// Write the final latency table to this path in addition to stdout.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
    /// Capture production ec_distann crown activation counters per sweep.
    #[arg(long)]
    pub report_distann_crown_stats: bool,
}

fn build_latency_knn_sql(
    profile: &profiles::IndexProfile,
    corpus_table: &str,
    payload_shape: Option<DistannPayloadShape>,
) -> String {
    let Some(payload_shape) = payload_shape else {
        return build_knn_sql(profile, corpus_table);
    };
    debug_assert_eq!(profile.name, "ec_distann");
    let (projection, predicate) = match payload_shape {
        DistannPayloadShape::IdOnly => ("id", ""),
        DistannPayloadShape::NarrowScalar => ("id, source_id", ""),
        DistannPayloadShape::VectorBearing => ("id, source", ""),
        DistannPayloadShape::Toasted => (
            "id, payload_note",
            "WHERE payload_note IS NOT NULL AND id % 3 = 1 ",
        ),
    };
    format!(
        "SELECT {projection} FROM {corpus_table} {predicate}\
         ORDER BY embedding <#> $1::real[] LIMIT $2"
    )
}

pub async fn run(conn: &ConnectionOptions, args: LatencyArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if args.k == 0 || args.iterations == 0 {
        return Err(eyre!("--k and --iterations must both be >= 1"));
    }
    let concurrency_values =
        normalized_concurrency_values(args.concurrency, &args.concurrency_sweep)?;
    if args.memory_sample_interval_ms == 0 {
        return Err(eyre!("--memory-sample-interval-ms must be >= 1"));
    }
    if args.hold_transaction && args.worker_batch_size != 0 {
        return Err(eyre!("--hold-transaction requires --worker-batch-size 0"));
    }
    let profile = profiles::resolve(&args.profile).ok_or_else(|| {
        eyre!(
            "unknown profile {:?}; try {}",
            args.profile,
            profiles::names().join(", ")
        )
    })?;
    if args.distann_stage_counters && profile.name != "ec_distann" {
        return Err(eyre!(
            "--distann-stage-counters is only supported with --profile ec_distann"
        ));
    }
    if args.report_distann_crown_stats && profile.name != "ec_distann" {
        return Err(eyre!(
            "--report-distann-crown-stats is only supported with --profile ec_distann"
        ));
    }
    if args.distann_payload_shape.is_some() && profile.name != "ec_distann" {
        return Err(eyre!(
            "--distann-payload-shape is only supported with --profile ec_distann"
        ));
    }
    let guc = profile
        .ef_search_guc
        .ok_or_else(|| eyre!("profile {:?} has no tuning GUC to sweep", profile.name))?;
    let sweep_values: Vec<i32> = if args.sweep.is_empty() {
        if profile.default_sweep.is_empty() {
            return Err(eyre!(
                "--sweep is required for profile {:?} (no default sweep registered)",
                profile.name
            ));
        }
        eprintln!(
            "[latency] no --sweep provided; using profile default {} values {:?}",
            profile.sweep_axis_label(),
            profile.default_sweep
        );
        profile.default_sweep.to_vec()
    } else {
        args.sweep.clone()
    };
    validate_rerank_width_arg(profile, args.rerank_width)?;
    let adaptive_nprobe_options = super::AdaptiveNprobeBenchOptions {
        enabled: args.adaptive_nprobe,
        score_gap_micros: args.adaptive_nprobe_score_gap_micros,
        score_margin_ratio_bps: args.adaptive_nprobe_score_margin_ratio_bps,
    };
    super::validate_adaptive_nprobe_options(profile, adaptive_nprobe_options)?;
    super::validate_ivf_scratch_soa_batch_decode(profile, args.ivf_scratch_soa_batch_decode)?;
    let mut session_gucs = super::parse_session_gucs(&args.session_gucs)?;
    if args.distann_stage_counters
        && !session_gucs
            .iter()
            .any(|(name, _)| name == "ec_distann.scan_profile_notice")
    {
        // Per-hop Task 206 telemetry is emitted as NOTICE records. Keep the
        // instrumentation opt-in with the existing stage-counter switch.
        session_gucs.push(("ec_distann.scan_profile_notice".to_owned(), "on".to_owned()));
    }

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let sql = build_latency_knn_sql(profile, &corpus_table, args.distann_payload_shape);

    // Pull query vectors once into memory. Iterations > n_queries wraps.
    let bootstrap = psql::connect(conn).await?;
    if psql::index_count_with_am(&bootstrap, &corpus_table, profile.access_method).await? == 0 {
        return Err(eyre!(
            "{} on {:?}",
            super::missing_am_error(profile, profile.access_method),
            corpus_table
        ));
    }
    let rows = bootstrap
        .query(
            &format!("SELECT source FROM {queries_table} ORDER BY id"),
            &[],
        )
        .await
        .wrap_err_with(|| format!("reading {queries_table}"))?;
    if rows.is_empty() {
        return Err(eyre!("{queries_table} is empty"));
    }
    let queries: Arc<Vec<Vec<f32>>> =
        Arc::new(rows.iter().map(|r| r.get::<_, Vec<f32>>(0)).collect());
    drop(bootstrap);

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    let mut header = vec![
        profile.sweep_axis_label(),
        "count",
        "mean",
        "stddev",
        "min",
        "p50",
        "p95",
        "p99",
        "max",
        "cache_state",
        "worker_batch_size",
        "concurrency",
        "wall_ms",
        "qps",
    ];
    if args.sample_backend_memory {
        header.extend(["rss_peak_kb", "hwm_peak_kb", "memory_samples"]);
    }
    table.set_header(header);

    let rerank_width_guc = rerank_width_guc(profile);
    let mut task87_counter_lines = Vec::new();
    let mut ivf_stage_counter_lines = Vec::new();
    let mut distann_stage_counter_lines = Vec::new();
    let mut distann_materialization_work_lines = Vec::new();
    let mut distann_crown_stats_lines = Vec::new();
    let mut backend_memory_lines = Vec::new();
    let memory_series_output = if args.sample_backend_memory {
        if let Some(path) = args.memory_series_output.as_ref() {
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .wrap_err_with(|| format!("creating {}", parent.display()))?;
            }
            Some(Arc::new(Mutex::new(
                tokio::fs::OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(path)
                    .await
                    .wrap_err_with(|| format!("opening {}", path.display()))?,
            )))
        } else {
            None
        }
    } else {
        None
    };
    for value in &sweep_values {
        let tuning_label = super::sweep_value_label(profile, *value);
        for concurrency in &concurrency_values {
            let sweep_label = if args.concurrency_sweep.is_empty() {
                tuning_label.clone()
            } else {
                format!("{tuning_label} concurrency={concurrency}")
            };
            let sweep = run_sweep_point(
                conn,
                profile,
                guc,
                rerank_width_guc,
                sweep_label.clone(),
                *value,
                &sql,
                Arc::clone(&queries),
                *concurrency,
                args.iterations,
                args.warmup_iterations,
                args.worker_batch_size,
                args.hold_transaction,
                profile.encode_scan_query,
                args.force_index,
                args.rerank_width,
                session_gucs.clone(),
                adaptive_nprobe_options,
                args.ivf_scratch_soa_batch_decode,
                args.bits,
                args.seed,
                args.k,
                args.sample_backend_memory,
                args.memory_sample_interval_ms,
                memory_series_output.clone(),
                args.task87_candidate_batch_counters,
                args.ivf_stage_counters,
                args.distann_stage_counters,
                args.report_distann_crown_stats,
            )
            .await?;
            if args.task87_candidate_batch_counters {
                task87_counter_lines.push(super::format_block_kernel_counter_lines(
                    "latency",
                    &sweep_label,
                    &sweep.task87_candidate_batch_counters,
                ));
            }
            if args.ivf_stage_counters {
                ivf_stage_counter_lines.push(super::format_ivf_stage_counter_lines(
                    "latency",
                    &sweep_label,
                    &sweep.ivf_stage_counters,
                ));
            }
            if args.distann_stage_counters {
                distann_stage_counter_lines.push(super::format_distann_stage_counter_lines(
                    "latency",
                    &sweep_label,
                    &sweep.distann_stage_counters,
                ));
                distann_materialization_work_lines.push(
                    super::format_distann_materialization_work_lines(
                        "latency",
                        &sweep_label,
                        &sweep.distann_materialization_work,
                    ),
                );
            }
            if let Some(stats) = sweep.distann_crown_stats {
                distann_crown_stats_lines.push(super::format_distann_crown_stats(
                    "latency",
                    &sweep_label,
                    stats,
                ));
            }
            let stats = summarize(&sweep.durations);
            let mut row = vec![
                Cell::new(value),
                Cell::new(stats.count),
                Cell::new(format_ms(stats.mean)),
                Cell::new(format_ms(stats.stddev)),
                Cell::new(format_ms(stats.min)),
                Cell::new(format_ms(stats.p50)),
                Cell::new(format_ms(stats.p95)),
                Cell::new(format_ms(stats.p99)),
                Cell::new(format_ms(stats.max)),
                Cell::new(&args.cache_state),
                Cell::new(args.worker_batch_size),
                Cell::new(concurrency),
                Cell::new(format_ms(sweep.wall_time)),
                Cell::new(format!(
                    "{:.3}",
                    throughput_qps(stats.count, sweep.wall_time)
                )),
            ];
            if args.sample_backend_memory {
                row.extend([
                    Cell::new(sweep.memory.rss_peak_kb),
                    Cell::new(sweep.memory.hwm_peak_kb),
                    Cell::new(sweep.memory.samples),
                ]);
                backend_memory_lines.extend(
                    sweep
                        .memory_series
                        .iter()
                        .map(|point| format_backend_memory_point(&sweep_label, point)),
                );
            }
            table.add_row(row);
        }
    }
    let mut output = table.to_string();
    if !task87_counter_lines.is_empty() {
        output.push('\n');
        output.push_str(&task87_counter_lines.join("\n"));
    }
    if !ivf_stage_counter_lines.is_empty() {
        output.push('\n');
        output.push_str(&ivf_stage_counter_lines.join("\n"));
    }
    if !distann_stage_counter_lines.is_empty() {
        output.push('\n');
        output.push_str(&distann_stage_counter_lines.join("\n"));
    }
    if !backend_memory_lines.is_empty() {
        output.push('\n');
        output.push_str(&backend_memory_lines.join("\n"));
    }
    if !distann_materialization_work_lines.is_empty() {
        output.push('\n');
        output.push_str(&distann_materialization_work_lines.join("\n"));
    }
    if !distann_crown_stats_lines.is_empty() {
        output.push('\n');
        output.push_str(&distann_crown_stats_lines.join("\n"));
    }
    println!("{output}");
    if let Some(path) = args.log_output {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        tokio::fs::write(&path, format!("{output}\n"))
            .await
            .wrap_err_with(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_sweep_point(
    conn: &ConnectionOptions,
    profile: &'static profiles::IndexProfile,
    guc: &str,
    rerank_width_guc: Option<&str>,
    sweep_label: String,
    value: i32,
    sql: &str,
    queries: Arc<Vec<Vec<f32>>>,
    concurrency: usize,
    iterations: usize,
    warmup_iterations: usize,
    worker_batch_size: usize,
    hold_transaction: bool,
    encode_scan_query: bool,
    force_index: bool,
    rerank_width: Option<i32>,
    session_gucs: Vec<(String, String)>,
    adaptive_nprobe_options: super::AdaptiveNprobeBenchOptions,
    ivf_scratch_soa_batch_decode: bool,
    bits: i32,
    seed: i64,
    k: usize,
    sample_backend_memory: bool,
    memory_sample_interval_ms: u64,
    memory_series_output: Option<Arc<Mutex<tokio::fs::File>>>,
    task87_candidate_batch_counters: bool,
    ivf_stage_counters: bool,
    distann_stage_counters: bool,
    report_distann_crown_stats: bool,
) -> Result<LatencySweepResult> {
    let bar = ProgressBar::new(iterations as u64);
    bar.set_style(
        ProgressStyle::with_template("[latency {msg}] {wide_bar} {pos}/{len} ({per_sec})").unwrap(),
    );
    let msg = match (rerank_width, rerank_width_guc) {
        (Some(rerank_width), Some(rerank_width_guc)) => {
            format!("{sweep_label} {rerank_width_guc}={rerank_width}")
        }
        _ => sweep_label.clone(),
    };
    let msg = super::append_adaptive_nprobe_label(msg, adaptive_nprobe_options);
    let msg = super::append_ivf_scratch_soa_batch_decode_label(msg, ivf_scratch_soa_batch_decode);
    bar.set_message(msg);
    bar.enable_steady_tick(Duration::from_millis(250));
    let bar = Arc::new(bar);

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(concurrency);
    for _ in 0..concurrency {
        let conn = conn.clone();
        let guc = guc.to_owned();
        let rerank_width_guc = rerank_width_guc.map(str::to_owned);
        let session_gucs = session_gucs.clone();
        let sql = sql.to_owned();
        let queries = Arc::clone(&queries);
        let counter = Arc::clone(&counter);
        let bar = Arc::clone(&bar);
        let memory_series_output = memory_series_output.clone();
        let worker_sweep_label = sweep_label.clone();
        handles.push(tokio::spawn(async move {
            worker(
                conn,
                profile,
                guc,
                value,
                sql,
                queries,
                counter,
                iterations,
                warmup_iterations,
                worker_batch_size,
                hold_transaction,
                encode_scan_query,
                force_index,
                rerank_width,
                rerank_width_guc,
                session_gucs,
                adaptive_nprobe_options,
                ivf_scratch_soa_batch_decode,
                bits,
                seed,
                k,
                worker_sweep_label,
                sample_backend_memory,
                memory_sample_interval_ms,
                memory_series_output.clone(),
                task87_candidate_batch_counters,
                ivf_stage_counters,
                distann_stage_counters,
                report_distann_crown_stats,
                bar,
            )
            .await
        }));
    }

    let mut merged: Vec<Duration> = Vec::with_capacity(iterations);
    let mut memory = MemorySample::default();
    let mut memory_series = Vec::new();
    let mut task87_counter_sets = Vec::new();
    let mut ivf_stage_counter_sets = Vec::new();
    let mut distann_stage_counter_sets = Vec::new();
    let mut distann_materialization_work_sets = Vec::new();
    let mut merged_crown_stats: Option<super::DistannCrownStats> = None;
    let mut timed_started_at = None;
    let mut timed_finished_at = None;
    for h in handles {
        let result = h.await.map_err(|e| eyre!("worker panicked: {e}"))??;
        merged.extend(result.durations);
        memory.merge(result.memory);
        memory_series.extend(result.memory_series);
        task87_counter_sets.push(result.task87_candidate_batch_counters);
        ivf_stage_counter_sets.push(result.ivf_stage_counters);
        distann_stage_counter_sets.push(result.distann_stage_counters);
        distann_materialization_work_sets.push(result.distann_materialization_work);
        if let Some(stats) = result.distann_crown_stats {
            merged_crown_stats.get_or_insert_default().add_assign(stats);
        }
        if let Some(started_at) = result.timed_started_at {
            timed_started_at = Some(
                timed_started_at.map_or(started_at, |earliest: Instant| earliest.min(started_at)),
            );
        }
        if let Some(finished_at) = result.timed_finished_at {
            timed_finished_at = Some(
                timed_finished_at.map_or(finished_at, |latest: Instant| latest.max(finished_at)),
            );
        }
    }
    bar.finish_and_clear();
    let wall_time = timed_started_at
        .zip(timed_finished_at)
        .map_or(Duration::ZERO, |(started_at, finished_at)| {
            finished_at.duration_since(started_at)
        });
    Ok(LatencySweepResult {
        durations: merged,
        wall_time,
        memory,
        memory_series,
        task87_candidate_batch_counters: super::merge_block_kernel_counters(task87_counter_sets),
        ivf_stage_counters: super::merge_ivf_stage_counters(ivf_stage_counter_sets),
        distann_stage_counters: super::merge_distann_stage_counters(distann_stage_counter_sets),
        distann_materialization_work: super::merge_distann_materialization_work(
            distann_materialization_work_sets,
        ),
        distann_crown_stats: merged_crown_stats,
    })
}

#[allow(clippy::too_many_arguments)]
async fn worker(
    conn: ConnectionOptions,
    profile: &'static profiles::IndexProfile,
    guc: String,
    value: i32,
    sql: String,
    queries: Arc<Vec<Vec<f32>>>,
    counter: Arc<AtomicUsize>,
    iterations: usize,
    warmup_iterations: usize,
    worker_batch_size: usize,
    hold_transaction: bool,
    encode_scan_query: bool,
    force_index: bool,
    rerank_width: Option<i32>,
    rerank_width_guc: Option<String>,
    session_gucs: Vec<(String, String)>,
    adaptive_nprobe_options: super::AdaptiveNprobeBenchOptions,
    ivf_scratch_soa_batch_decode: bool,
    bits: i32,
    seed: i64,
    k: usize,
    sweep_label: String,
    sample_backend_memory: bool,
    memory_sample_interval_ms: u64,
    memory_series_output: Option<Arc<Mutex<tokio::fs::File>>>,
    task87_candidate_batch_counters: bool,
    ivf_stage_counters: bool,
    distann_stage_counters: bool,
    report_distann_crown_stats: bool,
    bar: Arc<ProgressBar>,
) -> Result<LatencyWorkerResult> {
    let mut durations = Vec::new();
    let mut memory = MemorySample::default();
    let mut memory_series = Vec::new();
    let mut task87_counter_sets = Vec::new();
    let mut ivf_stage_counter_sets = Vec::new();
    let mut distann_stage_counter_sets = Vec::new();
    let mut distann_materialization_work_sets = Vec::new();
    let mut distann_crown_stats: Option<super::DistannCrownStats> = None;
    let mut timed_started_at = None;
    let mut timed_finished_at = None;
    let batch_size = if worker_batch_size == 0 {
        iterations
    } else {
        worker_batch_size
    };
    loop {
        // Each batch gets a fresh backend. This bounds memory retained by the
        // physical query/materialization path while preserving one merged
        // latency and attribution result for the whole worker.
        let (client, stmt) = open_worker_client(
            &conn,
            profile,
            &guc,
            value,
            &sql,
            rerank_width,
            rerank_width_guc.as_deref(),
            &session_gucs,
            adaptive_nprobe_options,
            ivf_scratch_soa_batch_decode,
            force_index,
        )
        .await?;
        let k_i64 = k as i64;
        // Every reconnect starts a fresh backend. Replay the untimed warmup
        // on each batch so the first timed query is never a reconnect/cold
        // sample. With the default batch size of zero this executes once,
        // preserving the historical single-backend behavior.
        for idx in 0..warmup_iterations {
            let q = &queries[idx % queries.len()];
            if encode_scan_query {
                client.query(&stmt, &[q, &bits, &seed, &k_i64]).await?;
            } else {
                client.query(&stmt, &[q, &k_i64]).await?;
            }
        }
        if report_distann_crown_stats {
            super::reset_distann_crown_stats(&client).await?;
        }
        if task87_candidate_batch_counters {
            super::reset_block_kernel_counters(&client).await?;
        }
        if ivf_stage_counters {
            super::reset_ivf_stage_counters(&client).await?;
        }
        if distann_stage_counters {
            super::reset_distann_stage_counters(&client).await?;
        }
        if hold_transaction {
            client.batch_execute("BEGIN").await?;
        }

        let batch_memory = Arc::new(Mutex::new(MemorySample::default()));
        let batch_memory_series = Arc::new(Mutex::new(Vec::new()));
        let stop_memory_monitor = Arc::new(AtomicBool::new(false));
        let memory_monitor = if sample_backend_memory {
            let backend_pid: i32 = client
                .query_one("SELECT pg_backend_pid()", &[])
                .await
                .wrap_err("fetching latency worker backend pid")?
                .get(0);
            Some(tokio::spawn(monitor_backend_memory(
                backend_pid,
                memory_sample_interval_ms,
                Arc::clone(&stop_memory_monitor),
                Arc::clone(&batch_memory),
                Arc::clone(&batch_memory_series),
                sweep_label.clone(),
                memory_series_output.clone(),
            )))
        } else {
            None
        };

        let mut batch_durations = Vec::with_capacity(batch_size);
        let mut batch_result_rows = 0_i64;
        let mut exhausted = false;
        let batch_result: Result<()> = loop {
            if batch_durations.len() >= batch_size {
                break Ok(());
            }
            let idx = counter.fetch_add(1, Ordering::Relaxed);
            if idx >= iterations {
                exhausted = true;
                break Ok(());
            }
            let q = &queries[idx % queries.len()];
            let t0 = Instant::now();
            timed_started_at.get_or_insert(t0);
            let query_result = if encode_scan_query {
                client.query(&stmt, &[q, &bits, &seed, &k_i64]).await
            } else {
                client.query(&stmt, &[q, &k_i64]).await
            };
            let rows = match query_result {
                Ok(rows) => rows,
                Err(err) => break Err(err.into()),
            };
            batch_result_rows =
                batch_result_rows.saturating_add(i64::try_from(rows.len()).unwrap_or(i64::MAX));
            batch_durations.push(t0.elapsed());
            timed_finished_at = Some(Instant::now());
            bar.inc(1);
        };

        if hold_transaction {
            if batch_result.is_ok() {
                client.batch_execute("COMMIT").await?;
            } else {
                let _ = client.batch_execute("ROLLBACK").await;
            }
        }
        stop_memory_monitor.store(true, Ordering::SeqCst);
        if let Some(memory_monitor) = memory_monitor {
            memory_monitor
                .await
                .map_err(|e| eyre!("latency memory monitor task failed: {e}"))??;
        }
        memory.merge(*batch_memory.lock().await);
        memory_series.extend(batch_memory_series.lock().await.iter().cloned());
        batch_result?;

        if report_distann_crown_stats {
            if let Some(stats) = super::snapshot_distann_crown_stats(&client).await? {
                distann_crown_stats
                    .get_or_insert_default()
                    .add_assign(stats);
            }
        }

        task87_counter_sets.push(if task87_candidate_batch_counters {
            super::snapshot_block_kernel_counters(&client).await?
        } else {
            super::BlockKernelCounterSnapshots::default()
        });
        ivf_stage_counter_sets.push(if ivf_stage_counters {
            super::snapshot_ivf_stage_counters(&client).await?
        } else {
            Vec::new()
        });
        distann_stage_counter_sets.push(if distann_stage_counters {
            super::snapshot_distann_stage_counters(&client).await?
        } else {
            Vec::new()
        });
        if distann_stage_counters {
            let mut work = super::snapshot_distann_materialization_work(&client).await?;
            work.push(super::DistannMaterializationWorkSnapshot {
                metric: "client_result_rows".into(),
                scans: i64::try_from(batch_durations.len()).unwrap_or(i64::MAX),
                value: batch_result_rows,
            });
            distann_materialization_work_sets.push(work);
        } else {
            distann_materialization_work_sets.push(Vec::new());
        }
        durations.extend(batch_durations);
        if exhausted {
            break;
        }
    }

    Ok(LatencyWorkerResult {
        durations,
        timed_started_at,
        timed_finished_at,
        memory,
        memory_series,
        task87_candidate_batch_counters: super::merge_block_kernel_counters(task87_counter_sets),
        ivf_stage_counters: super::merge_ivf_stage_counters(ivf_stage_counter_sets),
        distann_stage_counters: super::merge_distann_stage_counters(distann_stage_counter_sets),
        distann_materialization_work: super::merge_distann_materialization_work(
            distann_materialization_work_sets,
        ),
        distann_crown_stats,
    })
}

#[allow(clippy::too_many_arguments)]
async fn open_worker_client(
    conn: &ConnectionOptions,
    profile: &'static profiles::IndexProfile,
    guc: &str,
    value: i32,
    sql: &str,
    rerank_width: Option<i32>,
    rerank_width_guc: Option<&str>,
    session_gucs: &[(String, String)],
    adaptive_nprobe_options: super::AdaptiveNprobeBenchOptions,
    ivf_scratch_soa_batch_decode: bool,
    force_index: bool,
) -> Result<(tokio_postgres::Client, tokio_postgres::Statement)> {
    // NOTICE records carry per-round traversal attribution. The suite captures
    // stderr into the packet-local run log and parses the structured lines.
    let client = psql::connect_reporting_notices(conn).await?;
    psql::prefer_ordered_ann_path(&client).await?;
    client
        .batch_execute(&format!("SET {guc} = {value}"))
        .await?;
    if let (Some(rerank_width), Some(rerank_width_guc)) = (rerank_width, rerank_width_guc) {
        client
            .batch_execute(&format!("SET {rerank_width_guc} = {rerank_width}"))
            .await?;
    }
    super::apply_session_gucs(&client, session_gucs).await?;
    super::apply_adaptive_nprobe_options(&client, profile, adaptive_nprobe_options).await?;
    super::apply_ivf_scratch_soa_batch_decode(&client, profile, ivf_scratch_soa_batch_decode)
        .await?;
    if force_index {
        client.batch_execute("SET enable_seqscan = off").await?;
    }
    let stmt = client.prepare(sql).await?;
    Ok((client, stmt))
}

fn validate_rerank_width_arg(
    profile: &profiles::IndexProfile,
    rerank_width: Option<i32>,
) -> Result<()> {
    let Some(value) = rerank_width else {
        return Ok(());
    };
    if rerank_width_guc(profile).is_none() {
        return Err(eyre!(
            "--rerank-width is only supported with --profile ec_ivf or ec_spire"
        ));
    }
    if value < -1 {
        return Err(eyre!("--rerank-width must be >= -1"));
    }
    Ok(())
}

fn rerank_width_guc(profile: &profiles::IndexProfile) -> Option<&'static str> {
    match profile.name {
        "ec_ivf" => Some("ec_ivf.rerank_width"),
        "ec_spire" => Some("ec_spire.rerank_width"),
        _ => None,
    }
}

/// Fixed-field summary of a latency sample. All durations are in the
/// Duration type so the caller decides how to format; `summarize` never
/// looks at wall time on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencyStats {
    pub count: usize,
    pub mean: Duration,
    pub stddev: Duration,
    pub min: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
}

impl LatencyStats {
    pub const ZERO: Self = Self {
        count: 0,
        mean: Duration::ZERO,
        stddev: Duration::ZERO,
        min: Duration::ZERO,
        p50: Duration::ZERO,
        p95: Duration::ZERO,
        p99: Duration::ZERO,
        max: Duration::ZERO,
    };
}

/// Summarise a sample of latencies. Percentiles use linear interpolation
/// between the two nearest ranks (numpy's default). An empty input returns
/// `LatencyStats::ZERO` — the caller decides whether to render that.
pub fn summarize(durations: &[Duration]) -> LatencyStats {
    if durations.is_empty() {
        return LatencyStats::ZERO;
    }
    let mut sorted_ns: Vec<f64> = durations.iter().map(|d| d.as_nanos() as f64).collect();
    sorted_ns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let count = sorted_ns.len();
    let mean_ns = sorted_ns.iter().sum::<f64>() / count as f64;
    let var = sorted_ns
        .iter()
        .map(|x| {
            let d = x - mean_ns;
            d * d
        })
        .sum::<f64>()
        / count as f64;
    let stddev_ns = var.sqrt();
    LatencyStats {
        count,
        mean: ns_to_duration(mean_ns),
        stddev: ns_to_duration(stddev_ns),
        min: ns_to_duration(sorted_ns[0]),
        p50: ns_to_duration(percentile_sorted(&sorted_ns, 0.50)),
        p95: ns_to_duration(percentile_sorted(&sorted_ns, 0.95)),
        p99: ns_to_duration(percentile_sorted(&sorted_ns, 0.99)),
        max: ns_to_duration(sorted_ns[count - 1]),
    }
}

#[derive(Debug, Default)]
struct LatencySweepResult {
    durations: Vec<Duration>,
    wall_time: Duration,
    memory: MemorySample,
    memory_series: Vec<BackendMemoryPoint>,
    task87_candidate_batch_counters: super::BlockKernelCounterSnapshots,
    ivf_stage_counters: Vec<super::IvfStageCounterSnapshot>,
    distann_stage_counters: Vec<super::DistannStageCounterSnapshot>,
    distann_materialization_work: Vec<super::DistannMaterializationWorkSnapshot>,
    distann_crown_stats: Option<super::DistannCrownStats>,
}

#[derive(Debug, Default)]
struct LatencyWorkerResult {
    durations: Vec<Duration>,
    timed_started_at: Option<Instant>,
    timed_finished_at: Option<Instant>,
    memory: MemorySample,
    memory_series: Vec<BackendMemoryPoint>,
    task87_candidate_batch_counters: super::BlockKernelCounterSnapshots,
    ivf_stage_counters: Vec<super::IvfStageCounterSnapshot>,
    distann_stage_counters: Vec<super::DistannStageCounterSnapshot>,
    distann_materialization_work: Vec<super::DistannMaterializationWorkSnapshot>,
    distann_crown_stats: Option<super::DistannCrownStats>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MemorySample {
    pub(crate) rss_peak_kb: i64,
    pub(crate) hwm_peak_kb: i64,
    pub(crate) samples: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackendMemoryPoint {
    pub(crate) pid: i32,
    pub(crate) elapsed_ms: u128,
    pub(crate) rss_kb: i64,
    pub(crate) hwm_kb: i64,
}

/// Fit RSS against elapsed time. A peak can stay below a fixed threshold while
/// a transaction-scoped leak continues to grow, so regression gates should use
/// this slope instead of a peak-only assertion.
pub(crate) fn rss_slope_kb_per_second(points: &[BackendMemoryPoint]) -> Option<f64> {
    if points.len() < 2 {
        return None;
    }
    let mean_x = points
        .iter()
        .map(|point| point.elapsed_ms as f64 / 1000.0)
        .sum::<f64>()
        / points.len() as f64;
    let mean_y = points.iter().map(|point| point.rss_kb as f64).sum::<f64>() / points.len() as f64;
    let mut covariance = 0.0;
    let mut variance = 0.0;
    for point in points {
        let x = point.elapsed_ms as f64 / 1000.0 - mean_x;
        let y = point.rss_kb as f64 - mean_y;
        covariance += x * y;
        variance += x * x;
    }
    (variance > 0.0).then_some(covariance / variance)
}

impl MemorySample {
    pub(crate) fn merge(&mut self, other: Self) {
        self.rss_peak_kb = self.rss_peak_kb.max(other.rss_peak_kb);
        self.hwm_peak_kb = self.hwm_peak_kb.max(other.hwm_peak_kb);
        self.samples += other.samples;
    }
}

pub(crate) async fn monitor_backend_memory(
    pid: i32,
    sample_interval_ms: u64,
    stop: Arc<AtomicBool>,
    peak: Arc<Mutex<MemorySample>>,
    series: Arc<Mutex<Vec<BackendMemoryPoint>>>,
    sweep: String,
    output: Option<Arc<Mutex<tokio::fs::File>>>,
) -> Result<()> {
    let started = Instant::now();
    while !stop.load(Ordering::Relaxed) {
        if let Some(sample) = read_proc_status_memory(pid).await? {
            let mut peak = peak.lock().await;
            peak.samples += 1;
            peak.rss_peak_kb = peak.rss_peak_kb.max(sample.rss_peak_kb);
            peak.hwm_peak_kb = peak.hwm_peak_kb.max(sample.hwm_peak_kb);
            let point = BackendMemoryPoint {
                pid,
                elapsed_ms: started.elapsed().as_millis(),
                rss_kb: sample.rss_peak_kb,
                hwm_kb: sample.hwm_peak_kb,
            };
            series.lock().await.push(point.clone());
            if let Some(output) = output.as_ref() {
                let mut output = output.lock().await;
                output
                    .write_all(format_backend_memory_point(&sweep, &point).as_bytes())
                    .await
                    .wrap_err("writing backend memory sample")?;
                output
                    .write_all(b"\n")
                    .await
                    .wrap_err("writing backend memory newline")?;
                output
                    .flush()
                    .await
                    .wrap_err("flushing backend memory sample")?;
            }
        }
        tokio::time::sleep(Duration::from_millis(sample_interval_ms)).await;
    }
    Ok(())
}

fn format_backend_memory_point(sweep: &str, point: &BackendMemoryPoint) -> String {
    format!(
        "[backend-memory] sweep={sweep} pid={} elapsed_ms={} rss_kb={} hwm_kb={}",
        point.pid, point.elapsed_ms, point.rss_kb, point.hwm_kb
    )
}

pub(crate) async fn read_proc_status_memory(pid: i32) -> Result<Option<MemorySample>> {
    let path = format!("/proc/{pid}/status");
    let Ok(contents) = tokio::fs::read_to_string(&path).await else {
        return Ok(None);
    };
    let mut sample = MemorySample::default();
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("VmRSS:") {
            sample.rss_peak_kb = parse_status_kb(value)?;
        } else if let Some(value) = line.strip_prefix("VmHWM:") {
            sample.hwm_peak_kb = parse_status_kb(value)?;
        }
    }
    Ok(Some(sample))
}

fn parse_status_kb(value: &str) -> Result<i64> {
    value
        .split_whitespace()
        .next()
        .ok_or_else(|| eyre!("missing /proc status memory value"))?
        .parse::<i64>()
        .wrap_err("parsing /proc status memory value")
}

#[cfg(test)]
mod memory_regression_tests {
    use super::{rss_slope_kb_per_second, BackendMemoryPoint};

    #[test]
    fn rss_slope_is_not_a_peak_check() {
        let points = [
            BackendMemoryPoint {
                pid: 1,
                elapsed_ms: 0,
                rss_kb: 100,
                hwm_kb: 100,
            },
            BackendMemoryPoint {
                pid: 1,
                elapsed_ms: 1000,
                rss_kb: 1100,
                hwm_kb: 1100,
            },
            BackendMemoryPoint {
                pid: 1,
                elapsed_ms: 2000,
                rss_kb: 2100,
                hwm_kb: 2100,
            },
        ];
        assert_eq!(rss_slope_kb_per_second(&points), Some(1000.0));
        assert!(rss_slope_kb_per_second(&points[..1]).is_none());
    }
}

/// Linear-interpolated percentile from a pre-sorted ascending sample.
/// `p` is in [0, 1]; out-of-range values are clamped so a caller passing
/// `0.95` vs `95.0` never produces a panic.
pub fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let p = p.clamp(0.0, 1.0);
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let rank = p * (n - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let frac = rank - lo as f64;
        sorted[lo] + frac * (sorted[hi] - sorted[lo])
    }
}

fn ns_to_duration(ns: f64) -> Duration {
    if !ns.is_finite() || ns < 0.0 {
        return Duration::ZERO;
    }
    Duration::from_nanos(ns.round() as u64)
}

fn format_ms(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms >= 10.0 {
        format!("{ms:.1} ms")
    } else {
        format!("{ms:.2} ms")
    }
}

fn throughput_qps(completed: usize, wall_time: Duration) -> f64 {
    let seconds = wall_time.as_secs_f64();
    if completed == 0 || seconds <= 0.0 {
        0.0
    } else {
        completed as f64 / seconds
    }
}

fn normalized_concurrency_values(single: usize, sweep: &[usize]) -> Result<Vec<usize>> {
    if single == 0 {
        return Err(eyre!("--concurrency must be >= 1"));
    }
    if sweep.is_empty() {
        return Ok(vec![single]);
    }
    if sweep.contains(&0) {
        return Err(eyre!("--concurrency-sweep values must all be >= 1"));
    }
    if sweep
        .iter()
        .enumerate()
        .any(|(index, value)| sweep[..index].contains(value))
    {
        return Err(eyre!("--concurrency-sweep values must be unique"));
    }
    Ok(sweep.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn task224_payload_shapes_preserve_knn_order_and_select_registered_columns() {
        let scalar = build_latency_knn_sql(
            &profiles::EC_DISTANN,
            "physical_corpus",
            Some(DistannPayloadShape::NarrowScalar),
        );
        assert!(scalar.starts_with("SELECT id, source_id FROM physical_corpus"));
        assert!(scalar.contains("ORDER BY embedding <#> $1::real[] LIMIT $2"));

        let vector = build_latency_knn_sql(
            &profiles::EC_DISTANN,
            "physical_corpus",
            Some(DistannPayloadShape::VectorBearing),
        );
        assert!(vector.starts_with("SELECT id, source FROM physical_corpus"));

        let toasted = build_latency_knn_sql(
            &profiles::EC_DISTANN,
            "physical_corpus",
            Some(DistannPayloadShape::Toasted),
        );
        assert!(toasted.starts_with("SELECT id, payload_note FROM physical_corpus"));
        assert!(toasted.contains("payload_note IS NOT NULL AND id % 3 = 1"));
    }

    // --- percentile_sorted ---

    #[test]
    fn percentile_sorted_empty_is_zero() {
        assert_eq!(percentile_sorted(&[], 0.5), 0.0);
    }

    #[test]
    fn percentile_sorted_single_value_is_that_value_for_any_p() {
        assert_eq!(percentile_sorted(&[42.0], 0.0), 42.0);
        assert_eq!(percentile_sorted(&[42.0], 0.5), 42.0);
        assert_eq!(percentile_sorted(&[42.0], 1.0), 42.0);
    }

    #[test]
    fn percentile_sorted_endpoints_hit_extremes() {
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile_sorted(&v, 0.0), 1.0);
        assert_eq!(percentile_sorted(&v, 1.0), 5.0);
    }

    #[test]
    fn percentile_sorted_linear_interpolates_between_ranks() {
        // Matches numpy.percentile([1,2,3,4], [50, 95]) = [2.5, 3.85]
        let v = vec![1.0, 2.0, 3.0, 4.0];
        assert!((percentile_sorted(&v, 0.50) - 2.5).abs() < 1e-9);
        assert!((percentile_sorted(&v, 0.95) - 3.85).abs() < 1e-9);
    }

    #[test]
    fn percentile_sorted_clamps_out_of_range_p() {
        let v = vec![1.0, 2.0, 3.0];
        assert_eq!(percentile_sorted(&v, -0.5), 1.0);
        assert_eq!(percentile_sorted(&v, 95.0), 3.0);
    }

    // --- summarize ---

    #[test]
    fn summarize_empty_returns_zero_stats() {
        assert_eq!(summarize(&[]), LatencyStats::ZERO);
    }

    #[test]
    fn summarize_single_value_has_zero_stddev_and_equal_percentiles() {
        let s = summarize(&[ms(5)]);
        assert_eq!(s.count, 1);
        assert_eq!(s.mean, ms(5));
        assert_eq!(s.stddev, Duration::ZERO);
        assert_eq!(s.min, ms(5));
        assert_eq!(s.max, ms(5));
        assert_eq!(s.p50, ms(5));
        assert_eq!(s.p99, ms(5));
    }

    #[test]
    fn summarize_is_independent_of_input_order() {
        let asc = [ms(1), ms(2), ms(3), ms(4), ms(5)];
        let desc = [ms(5), ms(4), ms(3), ms(2), ms(1)];
        assert_eq!(summarize(&asc), summarize(&desc));
    }

    #[test]
    fn summarize_mean_and_min_max_match_raw_sample() {
        let sample: Vec<Duration> = (1..=100).map(ms).collect();
        let s = summarize(&sample);
        assert_eq!(s.count, 100);
        assert_eq!(s.min, ms(1));
        assert_eq!(s.max, ms(100));
        // Mean of 1..=100 = 50.5 ms — allow slight rounding into whole ns.
        let mean_ms = s.mean.as_secs_f64() * 1000.0;
        assert!((mean_ms - 50.5).abs() < 0.001, "mean={mean_ms}");
    }

    #[test]
    fn summarize_stddev_matches_population_formula() {
        // sample = [1, 2, 3, 4, 5] ms → pop variance = 2.0, stddev = sqrt(2)
        let s = summarize(&[ms(1), ms(2), ms(3), ms(4), ms(5)]);
        let stddev_ms = s.stddev.as_secs_f64() * 1000.0;
        assert!(
            (stddev_ms - (2.0_f64).sqrt()).abs() < 1e-6,
            "stddev={stddev_ms}"
        );
    }

    #[test]
    fn summarize_p50_is_the_median() {
        let s = summarize(&[ms(1), ms(2), ms(3), ms(4), ms(5)]);
        assert_eq!(s.p50, ms(3));
    }

    // --- format_ms / ns_to_duration ---

    #[test]
    fn format_ms_switches_precision_at_10ms_boundary() {
        assert_eq!(format_ms(Duration::from_micros(4_567)), "4.57 ms");
        assert_eq!(format_ms(Duration::from_millis(150)), "150.0 ms");
    }

    #[test]
    fn throughput_uses_concurrent_wall_time_not_summed_query_durations() {
        assert_eq!(throughput_qps(100, Duration::from_secs(2)), 50.0);
        assert_eq!(throughput_qps(0, Duration::from_secs(2)), 0.0);
        assert_eq!(throughput_qps(100, Duration::ZERO), 0.0);
    }

    #[test]
    fn concurrency_sweep_overrides_single_value_and_preserves_order() {
        assert_eq!(
            normalized_concurrency_values(1, &[1, 2, 4, 8, 16]).unwrap(),
            vec![1, 2, 4, 8, 16]
        );
        assert_eq!(normalized_concurrency_values(4, &[]).unwrap(), vec![4]);
        assert!(normalized_concurrency_values(1, &[1, 0]).is_err());
        assert!(normalized_concurrency_values(1, &[1, 2, 1]).is_err());
    }

    #[test]
    fn ns_to_duration_rejects_nan_and_negative() {
        assert_eq!(ns_to_duration(f64::NAN), Duration::ZERO);
        assert_eq!(ns_to_duration(-1.0), Duration::ZERO);
        assert_eq!(ns_to_duration(f64::INFINITY), Duration::ZERO);
    }

    #[test]
    fn memory_sample_merge_keeps_peaks_and_adds_samples() {
        let mut left = MemorySample {
            rss_peak_kb: 10,
            hwm_peak_kb: 30,
            samples: 2,
        };
        left.merge(MemorySample {
            rss_peak_kb: 20,
            hwm_peak_kb: 25,
            samples: 3,
        });

        assert_eq!(
            left,
            MemorySample {
                rss_peak_kb: 20,
                hwm_peak_kb: 30,
                samples: 5,
            }
        );
    }

    #[test]
    fn parse_status_kb_reads_proc_status_value() {
        assert_eq!(parse_status_kb("   12345 kB").unwrap(), 12345);
    }

    #[test]
    fn parse_session_gucs_accepts_qualified_names() {
        let parsed =
            super::super::parse_session_gucs(&["ec_diskann.scan_profile_notice=on".to_owned()])
                .expect("valid guc");
        assert_eq!(
            parsed,
            vec![("ec_diskann.scan_profile_notice".to_owned(), "on".to_owned())]
        );
    }

    #[test]
    fn parse_session_gucs_rejects_malformed_entries() {
        assert!(
            super::super::parse_session_gucs(&["ec_diskann.scan_profile_notice".to_owned()])
                .is_err()
        );
        assert!(
            super::super::parse_session_gucs(&["ec_diskann.scan_profile_notice=".to_owned()])
                .is_err()
        );
        assert!(
            super::super::parse_session_gucs(&["ec_diskann.scan-profile=on".to_owned()]).is_err()
        );
    }
}
