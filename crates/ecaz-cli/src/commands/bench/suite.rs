//! `ecaz bench suite` — configured benchmark suite runner.
//!
//! Suites are JSON plans that expand into ordinary `ecaz` commands. The runner
//! keeps the expansion visible in a manifest, then optionally executes each
//! selected step in sequence.

use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, Context, ContextCompat, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::process::Command;

use crate::profiles::{self, IndexProfile};
use crate::psql::ConnectionOptions;

#[derive(Args, Debug)]
pub struct SuiteArgs {
    #[command(subcommand)]
    command: Option<SuiteCommand>,

    /// JSON suite configuration file. Legacy alias for `bench suite run --dry-run`.
    #[arg(long)]
    config: Option<PathBuf>,

    /// Print expanded commands without executing suite steps. Legacy alias for
    /// `bench suite run --dry-run`.
    #[arg(long)]
    dry_run: bool,

    /// Expand only steps with this name. Repeatable. Legacy alias for
    /// `bench suite run --only`.
    #[arg(long = "only")]
    only: Vec<String>,

    /// Write the suite manifest to this path. Legacy alias for
    /// `bench suite run --manifest-output`.
    #[arg(long)]
    manifest_output: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum SuiteCommand {
    /// Execute or dry-run a configured benchmark suite.
    Run(RunArgs),
    /// Validate suite shape and required input files before a long run.
    Audit(AuditArgs),
    /// Summarize completion state from a suite manifest.
    Status(StatusArgs),
    /// Emit a minimal markdown report from a suite manifest.
    Report(ReportArgs),
}

#[derive(Args, Debug)]
struct RunArgs {
    /// JSON suite configuration file.
    #[arg(long)]
    config: PathBuf,

    /// Print expanded commands without executing suite steps.
    #[arg(long)]
    dry_run: bool,

    /// Execute remaining selected steps after a failure.
    #[arg(long)]
    continue_on_error: bool,

    /// Run only steps with this name. Repeatable.
    #[arg(long = "only")]
    only: Vec<String>,

    /// Run only steps with this tag. Repeatable.
    #[arg(long = "only-tag")]
    only_tag: Vec<String>,

    /// Reuse successful step records from an earlier manifest.
    #[arg(long)]
    resume_from: Option<PathBuf>,

    /// Write normalized result rows. Defaults to `<artifact_dir>/results.jsonl`
    /// when the config has `artifact_dir`.
    #[arg(long)]
    results_output: Option<PathBuf>,

    /// Override the suite config artifact directory for generated logs,
    /// manifests, and results.
    #[arg(long)]
    artifact_dir: Option<PathBuf>,

    /// Write the suite manifest to this path. Defaults to
    /// `<artifact_dir>/suite-manifest.json` when the config has `artifact_dir`.
    #[arg(long)]
    manifest_output: Option<PathBuf>,

    /// Permit latency/recall suite steps against a debug-built backend.
    #[arg(long)]
    allow_debug_backend: bool,
}

#[derive(Args, Debug)]
struct AuditArgs {
    /// JSON suite configuration file.
    #[arg(long)]
    config: PathBuf,
}

#[derive(Args, Debug)]
struct StatusArgs {
    /// Suite manifest produced by `ecaz bench suite run`.
    #[arg(long)]
    manifest: PathBuf,
}

#[derive(Args, Debug)]
struct ReportArgs {
    /// Suite manifest produced by `ecaz bench suite run`.
    #[arg(long)]
    manifest: PathBuf,

    /// Write normalized result rows parsed from manifest artifacts.
    #[arg(long)]
    results_output: Option<PathBuf>,
}

#[derive(Debug)]
struct SuiteRunOptions {
    config: PathBuf,
    dry_run: bool,
    continue_on_error: bool,
    only: Vec<String>,
    only_tag: Vec<String>,
    resume_from: Option<PathBuf>,
    results_output: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    manifest_output: Option<PathBuf>,
    allow_debug_backend: bool,
}

#[derive(Debug, Deserialize)]
struct SuiteConfig {
    name: String,
    schema_version: u32,
    #[serde(default)]
    artifact_dir: Option<PathBuf>,
    #[serde(default)]
    defaults: SuiteDefaults,
    #[serde(default)]
    thresholds: Vec<ThresholdConfig>,
    steps: Vec<SuiteStep>,
}

#[derive(Debug, Clone, Deserialize)]
struct ThresholdConfig {
    name: String,
    step: String,
    metric: String,
    #[serde(default)]
    filters: BTreeMap<String, String>,
    field: String,
    op: ThresholdOp,
    value: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ThresholdOp {
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
}

#[derive(Debug, Default, Deserialize)]
struct SuiteDefaults {
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    bits: Option<i32>,
    #[serde(default)]
    seed: Option<i64>,
    #[serde(default)]
    queries_limit: Option<usize>,
    #[serde(default)]
    iterations: Option<usize>,
    #[serde(default)]
    force_index: Option<bool>,
    #[serde(default)]
    sample_backend_memory: Option<bool>,
    #[serde(default)]
    memory_sample_interval_ms: Option<u64>,
    #[serde(default)]
    pg: Option<u16>,
    #[serde(default)]
    socket_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum SuiteStep {
    CorpusFetch(CorpusFetchStep),
    CorpusPrepare(CorpusPrepareStep),
    Load(LoadStep),
    Recall(RecallStep),
    CrossAm(CrossAmStep),
    Latency(LatencyStep),
    SpireLocalMultinode(SpireLocalMultinodeStep),
    SpirePipeline(SpirePipelineStep),
    Storage(StorageStep),
    Explain(ExplainStep),
    SidecarRerank(SidecarRerankStep),
    Comparator(ComparatorStep),
    Raw(RawStep),
}

#[derive(Debug, Deserialize)]
struct CorpusFetchStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    dataset: String,
    output_dir: PathBuf,
    #[serde(default)]
    revision: Option<String>,
    #[serde(default)]
    force: bool,
}

#[derive(Debug, Deserialize)]
struct CorpusPrepareStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    profile: String,
    parquet: PathBuf,
    output_dir: PathBuf,
    #[serde(default)]
    id_column: Option<String>,
    #[serde(default)]
    vector_column: Option<String>,
    #[serde(default)]
    dim: Option<usize>,
    #[serde(default)]
    source_dataset: Option<String>,
    #[serde(default)]
    chunk_rows: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct LoadStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    pgoptions: Option<String>,
    #[serde(default)]
    capture_parallel_workers: bool,
    prefix: String,
    #[serde(default)]
    corpus_file: Option<PathBuf>,
    #[serde(default)]
    queries_file: Option<PathBuf>,
    #[serde(default)]
    manifest_file: Option<PathBuf>,
    #[serde(default)]
    allow_manifest_mismatch: bool,
    #[serde(default)]
    chunked: bool,
    #[serde(default)]
    dim: Option<usize>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    bits: Option<i32>,
    #[serde(default)]
    seed: Option<i64>,
    #[serde(default)]
    m: Vec<i32>,
    #[serde(default)]
    ef_construction: Option<i32>,
    #[serde(default)]
    storage_format: Option<String>,
    #[serde(default)]
    index_name: Option<String>,
    #[serde(default)]
    table_reloptions: Vec<String>,
    #[serde(default)]
    reloptions: Vec<String>,
    #[serde(default)]
    log_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct RecallStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    prefix: String,
    k: usize,
    sweep: Vec<i32>,
    #[serde(default)]
    rerank_width: Option<i32>,
    #[serde(default)]
    adaptive_nprobe: Option<bool>,
    #[serde(default)]
    adaptive_nprobe_score_gap_micros: Option<i32>,
    #[serde(default)]
    adaptive_nprobe_score_margin_ratio_bps: Option<i32>,
    #[serde(default)]
    ivf_scratch_soa_batch_decode: Option<bool>,
    #[serde(default)]
    queries_limit: Option<usize>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    bits: Option<i32>,
    #[serde(default)]
    seed: Option<i64>,
    #[serde(default)]
    force_index: Option<bool>,
    #[serde(default)]
    session_gucs: Vec<String>,
    #[serde(default)]
    truth_cache_file: Option<PathBuf>,
    #[serde(default)]
    truth_cache_dir: Option<PathBuf>,
    #[serde(default)]
    truth_corpus_file: Option<PathBuf>,
    #[serde(default)]
    log_output: Option<PathBuf>,
    #[serde(default)]
    predictions_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct CrossAmStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    inputs: Vec<String>,
    #[serde(default)]
    k: Option<usize>,
    #[serde(default)]
    log_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LatencyStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    pgoptions: Option<String>,
    prefix: String,
    sweep: Vec<i32>,
    #[serde(default)]
    k: Option<usize>,
    #[serde(default)]
    concurrency: Option<usize>,
    #[serde(default)]
    iterations: Option<usize>,
    #[serde(default)]
    rerank_width: Option<i32>,
    #[serde(default)]
    adaptive_nprobe: Option<bool>,
    #[serde(default)]
    adaptive_nprobe_score_gap_micros: Option<i32>,
    #[serde(default)]
    adaptive_nprobe_score_margin_ratio_bps: Option<i32>,
    #[serde(default)]
    ivf_scratch_soa_batch_decode: Option<bool>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    bits: Option<i32>,
    #[serde(default)]
    seed: Option<i64>,
    #[serde(default)]
    force_index: Option<bool>,
    #[serde(default)]
    sample_backend_memory: Option<bool>,
    #[serde(default)]
    cache_state: Option<String>,
    #[serde(default)]
    session_gucs: Vec<String>,
    #[serde(default)]
    task87_candidate_batch_counters: Option<bool>,
    #[serde(default)]
    memory_sample_interval_ms: Option<u64>,
    #[serde(default)]
    log_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct SpireLocalMultinodeStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    pgoptions: Option<String>,
    #[serde(default)]
    artifact_dir: Option<PathBuf>,
    #[serde(default)]
    run_dir: Option<PathBuf>,
    #[serde(default)]
    smoke_log: Option<PathBuf>,
    #[serde(default)]
    pg: Option<u16>,
    #[serde(default)]
    pgbin: Option<PathBuf>,
    #[serde(default)]
    coord_port: Option<u16>,
    #[serde(default)]
    remote1_port: Option<u16>,
    #[serde(default)]
    remote2_port: Option<u16>,
    #[serde(default)]
    remote3_port: Option<u16>,
    #[serde(default)]
    run_id: Option<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    prepared_prefix: Option<String>,
    #[serde(default)]
    prepared_dir: Option<PathBuf>,
    #[serde(default)]
    storage_format: Option<String>,
    #[serde(default)]
    coord_index: Option<String>,
    #[serde(default)]
    remote_index: Option<String>,
    #[serde(default)]
    reloptions: Vec<String>,
    #[serde(default)]
    coord_reloptions: Vec<String>,
    #[serde(default)]
    remote_reloptions: Vec<String>,
    #[serde(default)]
    bench_top_k: Option<u16>,
    #[serde(default)]
    bench_queries_limit: Option<usize>,
    #[serde(default)]
    bench_sweep: Option<String>,
    #[serde(default)]
    bench_rowcap_sweep: Option<String>,
    #[serde(default)]
    skip_bench_rowcap: bool,
    #[serde(default)]
    bench_truth_corpus_file: Option<PathBuf>,
    #[serde(default)]
    bench_query_metric_projection_columns: Vec<String>,
    #[serde(default)]
    bench_session_gucs: Vec<String>,
    #[serde(default)]
    bench_production_read_variants: Vec<String>,
    #[serde(default)]
    skip_bench_suite: bool,
    #[serde(default)]
    skip_fault_drills: bool,
    #[serde(default)]
    skip_install: bool,
}

#[derive(Debug, Deserialize)]
struct SpirePipelineStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    pgoptions: Option<String>,
    prefix: String,
    #[serde(default)]
    index: Option<String>,
    #[serde(default)]
    queries_limit: Option<usize>,
    sweep: Vec<i32>,
    #[serde(default)]
    rerank_width: Option<i32>,
    #[serde(default)]
    max_candidate_rows: Option<i32>,
    #[serde(default)]
    max_routed_candidate_rows: Option<i32>,
    #[serde(default)]
    adaptive_nprobe: Option<bool>,
    #[serde(default)]
    adaptive_nprobe_score_gap_micros: Option<i32>,
    #[serde(default)]
    include_remote: Option<bool>,
    #[serde(default)]
    require_remote_placements: Option<bool>,
    #[serde(default)]
    include_local_store_overlap: Option<bool>,
    #[serde(default)]
    remote_selected_pids: Vec<i64>,
    #[serde(default)]
    remote_requested_epoch: Option<i64>,
    #[serde(default)]
    top_k: Option<i32>,
    #[serde(default)]
    consistency_mode: Option<String>,
    #[serde(default)]
    remote_tuple_transport: Option<String>,
    #[serde(default)]
    include_cost_snapshot: Option<bool>,
    #[serde(default)]
    cost_routing_dimension_scale: Option<f64>,
    #[serde(default)]
    cost_leaf_dimension_scale: Option<f64>,
    #[serde(default)]
    cost_index_page_scale: Option<f64>,
    #[serde(default)]
    cost_local_store_page_fanout_scale: Option<f64>,
    #[serde(default)]
    cost_storage_scoring_multiplier: Option<f64>,
    #[serde(default)]
    cost_rerank_multiplier: Option<f64>,
    #[serde(default)]
    include_query_metrics: Option<bool>,
    #[serde(default)]
    include_recall: Option<bool>,
    #[serde(default)]
    truth_corpus_file: Option<PathBuf>,
    #[serde(default)]
    truth_cache_file: Option<PathBuf>,
    #[serde(default)]
    leaf_block_rank_output: Option<PathBuf>,
    #[serde(default)]
    target_block_rank_output: Option<PathBuf>,
    #[serde(default)]
    target_candidate_rank_output: Option<PathBuf>,
    #[serde(default)]
    miss_attribution_output: Option<PathBuf>,
    #[serde(default)]
    leaf_block_rank_local_sequence_offset: Option<i64>,
    #[serde(default)]
    include_production_read_profile: Option<bool>,
    #[serde(default)]
    production_read_only: Option<bool>,
    #[serde(default)]
    production_read_timeline_no_payload: Option<bool>,
    #[serde(default)]
    query_metric_k: Option<usize>,
    #[serde(default)]
    query_metric_projection_columns: Vec<String>,
    #[serde(default)]
    session_gucs: Vec<String>,
    #[serde(default)]
    task87_candidate_batch_counters: Option<bool>,
    #[serde(default)]
    log_output: Option<PathBuf>,
    #[serde(default)]
    funnel_output: Option<PathBuf>,
    #[serde(default)]
    stage_containment_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct StorageStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    prefix: String,
    #[serde(default)]
    log_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct ExplainStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    prefix: String,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    index_name: Option<String>,
    #[serde(default)]
    query_table: Option<String>,
    #[serde(default)]
    corpus_table: Option<String>,
    nprobe: i32,
    rerank_width: i32,
    #[serde(default)]
    ivf_scratch_soa_batch_decode: Option<bool>,
    #[serde(default)]
    session_gucs: Vec<String>,
    #[serde(default)]
    pg: Option<u16>,
    #[serde(default)]
    db: Option<String>,
    #[serde(default)]
    socket_dir: Option<PathBuf>,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    sql_file: Option<PathBuf>,
    #[serde(default)]
    log_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct SidecarRerankStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    prefix: String,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    k: Option<usize>,
    #[serde(default)]
    candidate_k: Option<usize>,
    #[serde(default)]
    final_rerank_k: Option<usize>,
    #[serde(default)]
    concurrency: Option<usize>,
    sweep: Vec<i32>,
    #[serde(default)]
    queries_limit: Option<usize>,
    #[serde(default)]
    warmup_queries: Option<usize>,
    #[serde(default)]
    bits: Option<i32>,
    #[serde(default)]
    seed: Option<i64>,
    #[serde(default)]
    variants: Vec<String>,
    #[serde(default)]
    read_modes: Vec<String>,
    #[serde(default)]
    rebuild_sidecar_table: bool,
    #[serde(default)]
    force_index: Option<bool>,
    #[serde(default)]
    allow_unsafe_index_shape: bool,
    #[serde(default)]
    log_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct ComparatorStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    engine: String,
    prefix: String,
    #[serde(default)]
    k: Option<usize>,
    sweep: Vec<i32>,
    #[serde(default)]
    queries_limit: Option<usize>,
    #[serde(default)]
    lists: Option<i32>,
    #[serde(default)]
    m: Option<i32>,
    #[serde(default)]
    ef_construction: Option<i32>,
    #[serde(default)]
    num_neighbors: Option<i32>,
    #[serde(default)]
    build_search_list_size: Option<i32>,
    #[serde(default)]
    max_alpha: Option<f32>,
    #[serde(default)]
    storage_layout: Option<String>,
    #[serde(default)]
    maintenance_work_mem: Option<String>,
    #[serde(default)]
    rebuild: bool,
    #[serde(default)]
    log_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct RawStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    args: Vec<String>,
    #[serde(default)]
    expected_artifacts: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SuiteManifest {
    suite: String,
    schema_version: u32,
    config: String,
    config_sha256: String,
    dry_run: bool,
    generated_at_unix_ms: u128,
    connection: ManifestConnection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    backend: Option<BackendPreflight>,
    steps: Vec<StepRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    threshold_results: Vec<ThresholdResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestConnection {
    database: String,
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackendPreflight {
    build_profile: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StepRecord {
    name: String,
    kind: String,
    command: Vec<String>,
    selected: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    quant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    isa: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kernel_status: Option<KernelCellStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pgoptions: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    expected_artifacts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    status: Option<StepStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    started_at_unix_ms: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    finished_at_unix_ms: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parallel_workers_before: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parallel_workers_after: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parallel_workers_delta: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum KernelCellStatus {
    Valid,
    MissingKernel,
    StructurallyAbsent,
    InvalidConfig,
    /// Task 103/104: the cell's kernel was retired on measured grounds; the
    /// step still executes so the matrix re-confirms the disposition.
    Retired,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThresholdResult {
    name: String,
    step: String,
    metric: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    filters: BTreeMap<String, String>,
    field: String,
    op: ThresholdOp,
    expected: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    actual: Option<f64>,
    passed: bool,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum StepStatus {
    DryRun,
    Pending,
    Skipped,
    Succeeded,
    Failed,
}

pub async fn run(conn: &ConnectionOptions, args: SuiteArgs) -> Result<()> {
    match args.command {
        Some(SuiteCommand::Run(run_args)) => run_suite(conn, run_args.into()).await,
        Some(SuiteCommand::Audit(audit_args)) => audit_suite(&audit_args.config).await,
        Some(SuiteCommand::Status(status_args)) => status_manifest(&status_args.manifest).await,
        Some(SuiteCommand::Report(report_args)) => {
            report_manifest(&report_args.manifest, report_args.results_output.as_deref()).await
        }
        None => {
            let config = args.config.context(
                "missing --config; use `ecaz bench suite run --config <path>` or the legacy `ecaz bench suite --config <path> --dry-run` alias",
            )?;
            if !args.dry_run {
                bail!(
                    "legacy `ecaz bench suite --config` only supports --dry-run; use `ecaz bench suite run --config {}` to execute",
                    config.display()
                );
            }
            run_suite(
                conn,
                SuiteRunOptions {
                    config,
                    dry_run: true,
                    continue_on_error: false,
                    only: args.only,
                    only_tag: Vec::new(),
                    resume_from: None,
                    results_output: None,
                    artifact_dir: None,
                    manifest_output: args.manifest_output,
                    allow_debug_backend: false,
                },
            )
            .await
        }
    }
}

impl From<RunArgs> for SuiteRunOptions {
    fn from(args: RunArgs) -> Self {
        Self {
            config: args.config,
            dry_run: args.dry_run,
            continue_on_error: args.continue_on_error,
            only: args.only,
            only_tag: args.only_tag,
            resume_from: args.resume_from,
            results_output: args.results_output,
            artifact_dir: args.artifact_dir,
            manifest_output: args.manifest_output,
            allow_debug_backend: args.allow_debug_backend,
        }
    }
}

async fn run_suite(conn: &ConnectionOptions, args: SuiteRunOptions) -> Result<()> {
    let (raw, mut config) = load_config(&args.config).await?;
    if let Some(artifact_dir) = &args.artifact_dir {
        config.artifact_dir = Some(artifact_dir.clone());
    }
    apply_default_artifact_logs(&mut config);
    apply_artifact_dir_templates(&mut config);
    validate_config(&config)?;

    let mut manifest = build_manifest(conn, &args, &raw, &config)?;
    if let Some(resume_from) = &args.resume_from {
        apply_resume(&mut manifest, resume_from).await?;
    }
    if !args.dry_run && manifest_has_release_guarded_steps(&manifest) {
        let preflight = preflight_backend(conn).await?;
        manifest.backend = Some(preflight.clone());
        write_manifest_if_requested(&args, &config, &manifest).await?;
        if preflight.build_profile != "release" && !args.allow_debug_backend {
            bail!(
                "suite selected latency/recall steps but backend build profile is {:?}; \
                 reinstall a release backend or pass --allow-debug-backend",
                preflight.build_profile
            );
        }
    }
    write_manifest_if_requested(&args, &config, &manifest).await?;

    if args.dry_run {
        for record in &manifest.steps {
            if record.selected && !record.command.is_empty() {
                crate::ecaz_println!(
                    "[suite:{}] {} -> {}",
                    config.name,
                    record.name,
                    shell_join_with_pgoptions(&record.command, record.pgoptions.as_deref())
                );
            } else if record.selected {
                crate::ecaz_println!(
                    "[suite:{}] {} -> kernel_status={}",
                    config.name,
                    record.name,
                    record
                        .kernel_status
                        .map(kernel_status_label)
                        .unwrap_or("skipped")
                );
            }
        }
        return Ok(());
    }

    let exe = std::env::current_exe().context("resolving current ecaz executable")?;
    for idx in 0..manifest.steps.len() {
        if !manifest.steps[idx].selected
            || matches!(manifest.steps[idx].status, Some(StepStatus::Skipped))
        {
            continue;
        }
        if matches!(manifest.steps[idx].status, Some(StepStatus::Succeeded)) {
            crate::ecaz_println!(
                "[suite:{}] {} already succeeded in resume manifest",
                config.name,
                manifest.steps[idx].name
            );
            continue;
        }
        prepare_step(&config.steps[idx], &config.defaults).await?;
        let command = manifest.steps[idx].command.clone();
        crate::ecaz_println!(
            "[suite:{}] {} -> {}",
            config.name,
            manifest.steps[idx].name,
            shell_join_with_pgoptions(&command, manifest.steps[idx].pgoptions.as_deref())
        );
        manifest.steps[idx].status = Some(StepStatus::Pending);
        manifest.steps[idx].started_at_unix_ms = Some(now_ms());
        write_manifest_if_requested(&args, &config, &manifest).await?;

        let capture_parallel_workers = matches!(
            &config.steps[idx],
            SuiteStep::Load(step) if step.capture_parallel_workers
        );
        let started = Instant::now();
        let pgoptions = manifest.steps[idx].pgoptions.clone();
        let status = spawn_step(&exe, &command, conn, pgoptions.as_deref())
            .await
            .wrap_err_with(|| {
                format!(
                    "running suite step {:?}: {}",
                    manifest.steps[idx].name,
                    shell_join(&command)
                )
            })?;
        manifest.steps[idx].finished_at_unix_ms = Some(now_ms());
        manifest.steps[idx].duration_ms = Some(started.elapsed().as_millis());
        manifest.steps[idx].exit_code = status.code();
        manifest.steps[idx].status = Some(if status.success() {
            StepStatus::Succeeded
        } else {
            StepStatus::Failed
        });
        if capture_parallel_workers && status.success() {
            let workers_launched =
                capture_parallel_workers_from_load_artifacts(&manifest.steps[idx]).await?;
            manifest.steps[idx].parallel_workers_before = Some(0);
            manifest.steps[idx].parallel_workers_after = Some(workers_launched);
            manifest.steps[idx].parallel_workers_delta = Some(workers_launched);
        }
        write_manifest_if_requested(&args, &config, &manifest).await?;

        if !status.success() && !args.continue_on_error {
            bail!(
                "suite step {:?} failed with {}; rerun with --continue-on-error to keep going",
                manifest.steps[idx].name,
                format_exit_status(status)
            );
        }
    }

    let rows = write_results_if_requested(&args, &config, &manifest).await?;
    let selected_steps = selected_step_names(&manifest);
    manifest.threshold_results =
        evaluate_thresholds_for_steps(&config.thresholds, &rows, &selected_steps);
    write_manifest_if_requested(&args, &config, &manifest).await?;
    let failures = manifest
        .threshold_results
        .iter()
        .filter(|result| !result.passed)
        .count();
    if failures > 0 {
        bail!("suite thresholds failed: {failures}");
    }
    Ok(())
}

async fn audit_suite(config_path: &Path) -> Result<()> {
    let (_raw, mut config) = load_config(config_path).await?;
    apply_default_artifact_logs(&mut config);
    let mut findings = Vec::new();
    let mut produced = HashSet::new();
    if let Err(err) = validate_config(&config) {
        findings.push(err.to_string());
    }
    for step in &config.steps {
        for input in step.input_paths() {
            if produced.contains(&input) {
                continue;
            }
            if tokio::fs::metadata(&input).await.is_err() {
                findings.push(format!(
                    "step {:?} references missing input {}",
                    step.name(),
                    input.display()
                ));
            }
        }
        produced.extend(step.produced_paths());
        if step.expected_artifacts().is_empty() {
            findings.push(format!(
                "step {:?} does not declare an artifact path",
                step.name()
            ));
        }
    }

    if findings.is_empty() {
        crate::ecaz_println!(
            "[suite:{}] audit passed: {} steps",
            config.name,
            config.steps.len()
        );
        Ok(())
    } else {
        for finding in &findings {
            crate::ecaz_eprintln!("[suite:{}] audit: {finding}", config.name);
        }
        bail!("suite audit found {} issue(s)", findings.len())
    }
}

async fn status_manifest(path: &Path) -> Result<()> {
    let manifest = load_manifest(path).await?;
    let summary = summarize_manifest(&manifest).await;
    crate::ecaz_println!(
        "[suite:{}] completed={} failed={} skipped={} dry_run={} missing_artifacts={} stale={}",
        manifest.suite,
        summary.completed,
        summary.failed,
        summary.skipped,
        summary.dry_run,
        summary.missing_artifacts,
        summary.stale
    );
    for step in &manifest.steps {
        let status = step.status.unwrap_or(if step.selected {
            StepStatus::Pending
        } else {
            StepStatus::Skipped
        });
        crate::ecaz_println!(
            "{:<12} {:<36} {}",
            format!("{status:?}"),
            step.name,
            shell_join_with_pgoptions(&step.command, step.pgoptions.as_deref())
        );
    }
    Ok(())
}

async fn report_manifest(path: &Path, results_output: Option<&Path>) -> Result<()> {
    let manifest = load_manifest(path).await?;
    let summary = summarize_manifest(&manifest).await;
    let rows = extract_result_rows(&manifest).await?;
    crate::ecaz_println!("# Suite Report: {}", manifest.suite);
    crate::ecaz_println!("");
    crate::ecaz_println!("- config: `{}`", manifest.config);
    crate::ecaz_println!("- config_sha256: `{}`", manifest.config_sha256);
    crate::ecaz_println!("- dry_run: `{}`", manifest.dry_run);
    crate::ecaz_println!(
        "- steps: completed {}, failed {}, skipped {}, dry-run {}, missing artifacts {}, stale {}",
        summary.completed,
        summary.failed,
        summary.skipped,
        summary.dry_run,
        summary.missing_artifacts,
        summary.stale
    );
    crate::ecaz_println!("");
    crate::ecaz_println!("| Step | Kind | Status | Duration ms | Artifacts |");
    crate::ecaz_println!("| --- | --- | --- | ---: | --- |");
    for step in &manifest.steps {
        let status = step.status.unwrap_or(if step.selected {
            StepStatus::Pending
        } else {
            StepStatus::Skipped
        });
        crate::ecaz_println!(
            "| {} | {} | {:?} | {} | {} |",
            step.name,
            step.kind,
            status,
            step.duration_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".into()),
            if step.expected_artifacts.is_empty() {
                "-".into()
            } else {
                step.expected_artifacts
                    .iter()
                    .map(|path| format!("`{path}`"))
                    .collect::<Vec<_>>()
                    .join("<br>")
            }
        );
    }
    if !rows.is_empty() {
        crate::ecaz_println!("");
        crate::ecaz_println!("## Parsed Results");
        crate::ecaz_println!("");
        crate::ecaz_println!("| Step | Kind | Metric | Values |");
        crate::ecaz_println!("| --- | --- | --- | --- |");
        for row in &rows {
            crate::ecaz_println!(
                "| {} | {} | {} | {} |",
                row.step,
                row.kind,
                row.metric,
                format_metric_values(&row.values)
            );
        }
        if let Some(pooling_gate) = render_spire_pooling_gate_section(&rows) {
            crate::ecaz_println!("");
            crate::ecaz_println!("{pooling_gate}");
        }
    }
    if !manifest.threshold_results.is_empty() {
        crate::ecaz_println!("");
        crate::ecaz_println!("## Thresholds");
        crate::ecaz_println!("");
        crate::ecaz_println!("| Name | Status | Actual | Expected |");
        crate::ecaz_println!("| --- | --- | ---: | ---: |");
        for result in &manifest.threshold_results {
            crate::ecaz_println!(
                "| {} | {} | {} | {:?} {} |",
                result.name,
                if result.passed { "pass" } else { "fail" },
                result
                    .actual
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".into()),
                result.op,
                result.expected
            );
        }
    }
    if let Some(path) = results_output {
        write_results_jsonl(path, &rows).await?;
        crate::ecaz_eprintln!("wrote {}", path.display());
    }
    Ok(())
}

async fn load_config(path: &Path) -> Result<(String, SuiteConfig)> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .wrap_err_with(|| format!("reading {}", path.display()))?;
    let config: SuiteConfig =
        serde_json::from_str(&raw).wrap_err_with(|| format!("parsing {}", path.display()))?;
    Ok((raw, config))
}

fn apply_default_artifact_logs(config: &mut SuiteConfig) {
    let Some(artifact_dir) = config.artifact_dir.clone() else {
        return;
    };
    for step in &mut config.steps {
        let log_path =
            |name: &str| artifact_dir.join(format!("{}.log", artifact_safe_step_name(name)));
        match step {
            SuiteStep::Load(step) if step.log_file.is_none() => {
                step.log_file = Some(log_path(&step.name));
            }
            SuiteStep::Recall(step) if step.log_output.is_none() => {
                step.log_output = Some(log_path(&step.name));
            }
            SuiteStep::CrossAm(step) if step.log_output.is_none() => {
                step.log_output = Some(log_path(&step.name));
            }
            SuiteStep::Latency(step) if step.log_output.is_none() => {
                step.log_output = Some(log_path(&step.name));
            }
            SuiteStep::SpireLocalMultinode(step) => {
                if step.artifact_dir.is_none() {
                    step.artifact_dir =
                        Some(artifact_dir.join(artifact_safe_step_name(&step.name)));
                }
                if step.smoke_log.is_none() {
                    step.smoke_log = step
                        .artifact_dir
                        .as_ref()
                        .map(|dir| dir.join("local-multinode.log"));
                }
            }
            SuiteStep::SpirePipeline(step) if step.log_output.is_none() => {
                step.log_output = Some(log_path(&step.name));
            }
            SuiteStep::Storage(step) if step.log_file.is_none() => {
                step.log_file = Some(log_path(&step.name));
            }
            SuiteStep::Explain(step) => {
                let safe_name = artifact_safe_step_name(&step.name);
                if step.sql_file.is_none() {
                    step.sql_file = Some(artifact_dir.join(format!("{safe_name}.sql")));
                }
                if step.log_output.is_none() {
                    step.log_output = Some(artifact_dir.join(format!("{safe_name}.log")));
                }
            }
            SuiteStep::SidecarRerank(step) if step.log_output.is_none() => {
                step.log_output = Some(log_path(&step.name));
            }
            SuiteStep::Comparator(step) if step.log_output.is_none() => {
                step.log_output = Some(log_path(&step.name));
            }
            _ => {}
        }
    }
}

fn artifact_safe_step_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn apply_artifact_dir_templates(config: &mut SuiteConfig) {
    let Some(artifact_dir) = config.artifact_dir.as_ref() else {
        return;
    };
    let artifact_dir = artifact_dir.display().to_string();
    for step in &mut config.steps {
        match step {
            SuiteStep::Load(step) => {
                rewrite_artifact_dir_path(&mut step.log_file, &artifact_dir);
            }
            SuiteStep::Recall(step) => {
                rewrite_artifact_dir_path(&mut step.truth_cache_file, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.truth_cache_dir, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.truth_corpus_file, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.log_output, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.predictions_output, &artifact_dir);
            }
            SuiteStep::CrossAm(step) => {
                rewrite_artifact_dir_path(&mut step.log_output, &artifact_dir);
            }
            SuiteStep::Latency(step) => {
                rewrite_artifact_dir_path(&mut step.log_output, &artifact_dir);
            }
            SuiteStep::SpireLocalMultinode(step) => {
                rewrite_artifact_dir_path(&mut step.artifact_dir, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.run_dir, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.smoke_log, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.pgbin, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.prepared_dir, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.bench_truth_corpus_file, &artifact_dir);
            }
            SuiteStep::SpirePipeline(step) => {
                rewrite_artifact_dir_path(&mut step.truth_corpus_file, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.truth_cache_file, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.leaf_block_rank_output, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.target_block_rank_output, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.target_candidate_rank_output, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.miss_attribution_output, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.log_output, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.funnel_output, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.stage_containment_output, &artifact_dir);
            }
            SuiteStep::Storage(step) => {
                rewrite_artifact_dir_path(&mut step.log_file, &artifact_dir);
            }
            SuiteStep::Explain(step) => {
                rewrite_artifact_dir_path(&mut step.sql_file, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.log_output, &artifact_dir);
            }
            SuiteStep::SidecarRerank(step) => {
                rewrite_artifact_dir_path(&mut step.log_output, &artifact_dir);
            }
            SuiteStep::Comparator(step) => {
                rewrite_artifact_dir_path(&mut step.log_output, &artifact_dir);
            }
            SuiteStep::Raw(step) => {
                for arg in &mut step.args {
                    *arg = arg.replace("${artifact_dir}", &artifact_dir);
                }
                for artifact in &mut step.expected_artifacts {
                    rewrite_artifact_dir_pathbuf(artifact, &artifact_dir);
                }
            }
            SuiteStep::CorpusFetch(_) | SuiteStep::CorpusPrepare(_) => {}
        }
    }
}

fn rewrite_artifact_dir_path(path: &mut Option<PathBuf>, artifact_dir: &str) {
    if let Some(path) = path {
        rewrite_artifact_dir_pathbuf(path, artifact_dir);
    }
}

fn rewrite_artifact_dir_pathbuf(path: &mut PathBuf, artifact_dir: &str) {
    if let Some(raw) = path.to_str() {
        *path = PathBuf::from(raw.replace("${artifact_dir}", artifact_dir));
    }
}

async fn load_manifest(path: &Path) -> Result<SuiteManifest> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .wrap_err_with(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).wrap_err_with(|| format!("parsing {}", path.display()))
}

fn build_manifest(
    conn: &ConnectionOptions,
    args: &SuiteRunOptions,
    raw: &str,
    config: &SuiteConfig,
) -> Result<SuiteManifest> {
    let mut manifest = SuiteManifest {
        suite: config.name.clone(),
        schema_version: config.schema_version,
        config: args.config.display().to_string(),
        config_sha256: sha256_hex(raw.as_bytes()),
        dry_run: args.dry_run,
        generated_at_unix_ms: now_ms(),
        connection: ManifestConnection {
            database: conn.database.clone(),
            host: conn.host.clone(),
            port: conn.port,
            user: conn.user.clone(),
            password_configured: conn.password.is_some(),
        },
        backend: None,
        steps: Vec::with_capacity(config.steps.len()),
        threshold_results: Vec::new(),
    };

    for step in &config.steps {
        let selected = step_selected(step, args);
        let kernel_status = step_kernel_status(step.tags())?;
        let runnable = selected && kernel_cell_is_runnable(kernel_status);
        let command = if runnable {
            child_command_args(conn, step.expand(&config.defaults, conn)?)
        } else {
            Vec::new()
        };
        manifest.steps.push(StepRecord {
            name: step.name().to_string(),
            kind: step.kind().to_string(),
            command,
            selected,
            quant: step_quant(step.tags()),
            isa: step_isa(step.tags()),
            kernel_status,
            pgoptions: step.pgoptions().map(ToOwned::to_owned),
            tags: step.tags().to_vec(),
            expected_artifacts: step
                .expected_artifacts()
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            status: Some(if selected {
                if !runnable {
                    StepStatus::Skipped
                } else if args.dry_run {
                    StepStatus::DryRun
                } else {
                    StepStatus::Pending
                }
            } else {
                StepStatus::Skipped
            }),
            started_at_unix_ms: None,
            finished_at_unix_ms: None,
            duration_ms: None,
            exit_code: None,
            parallel_workers_before: None,
            parallel_workers_after: None,
            parallel_workers_delta: None,
        });
    }
    Ok(manifest)
}

fn step_selected(step: &SuiteStep, args: &SuiteRunOptions) -> bool {
    let name_matches = args.only.is_empty() || args.only.iter().any(|only| only == step.name());
    let tag_matches = args.only_tag.is_empty()
        || args
            .only_tag
            .iter()
            .any(|only| step.tags().iter().any(|tag| tag == only));
    name_matches && tag_matches
}

async fn apply_resume(manifest: &mut SuiteManifest, resume_from: &Path) -> Result<()> {
    let previous = load_manifest(resume_from).await?;
    if previous.config_sha256 != manifest.config_sha256 {
        bail!(
            "resume manifest config hash {} does not match current config hash {}",
            previous.config_sha256,
            manifest.config_sha256
        );
    }
    let previous_by_name: HashMap<_, _> = previous
        .steps
        .into_iter()
        .map(|step| (step.name.clone(), step))
        .collect();
    for step in &mut manifest.steps {
        if !step.selected {
            continue;
        }
        if let Some(previous) = previous_by_name.get(&step.name) {
            if matches!(previous.status, Some(StepStatus::Succeeded)) {
                if previous.command != step.command {
                    bail!(
                        "resume step {:?} command differs from current expanded command",
                        step.name
                    );
                }
                step.status = previous.status;
                step.started_at_unix_ms = previous.started_at_unix_ms;
                step.finished_at_unix_ms = previous.finished_at_unix_ms;
                step.duration_ms = previous.duration_ms;
                step.exit_code = previous.exit_code;
            }
        }
    }
    Ok(())
}

async fn write_manifest_if_requested(
    args: &SuiteRunOptions,
    config: &SuiteConfig,
    manifest: &SuiteManifest,
) -> Result<()> {
    if let Some(path) = manifest_path(args, config) {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        let body = serde_json::to_string_pretty(manifest)?;
        tokio::fs::write(&path, format!("{body}\n"))
            .await
            .wrap_err_with(|| format!("writing {}", path.display()))?;
        crate::ecaz_eprintln!("[suite:{}] wrote {}", config.name, path.display());
    }
    Ok(())
}

async fn write_results_if_requested(
    args: &SuiteRunOptions,
    config: &SuiteConfig,
    manifest: &SuiteManifest,
) -> Result<Vec<ResultRow>> {
    let rows = extract_result_rows(manifest).await?;
    let path = args.results_output.clone().or_else(|| {
        config
            .artifact_dir
            .as_ref()
            .map(|dir| dir.join("results.jsonl"))
    });
    if let Some(path) = path {
        write_results_jsonl(&path, &rows).await?;
        crate::ecaz_eprintln!("[suite:{}] wrote {}", config.name, path.display());
    }
    Ok(rows)
}

#[derive(Debug, Serialize)]
struct ResultRow {
    suite: String,
    step: String,
    kind: String,
    metric: String,
    artifact: String,
    values: BTreeMap<String, String>,
}

async fn extract_result_rows(manifest: &SuiteManifest) -> Result<Vec<ResultRow>> {
    let mut rows = Vec::new();
    for step in &manifest.steps {
        if let Some(row) = kernel_cell_result_row(manifest, step) {
            rows.push(row);
        }
        if !matches!(step.status, Some(StepStatus::Succeeded)) {
            continue;
        }
        if let Some(row) = parallel_worker_result_row(manifest, step) {
            rows.push(row);
        }
        for artifact in &step.expected_artifacts {
            let path = Path::new(artifact);
            let Ok(raw) = tokio::fs::read_to_string(path).await else {
                continue;
            };
            rows.extend(parse_result_rows(manifest, step, artifact, &raw));
        }
    }
    Ok(rows)
}

fn parallel_worker_result_row(manifest: &SuiteManifest, step: &StepRecord) -> Option<ResultRow> {
    let before = step.parallel_workers_before?;
    let after = step.parallel_workers_after?;
    let delta = step.parallel_workers_delta?;
    Some(ResultRow {
        suite: manifest.suite.clone(),
        step: step.name.clone(),
        kind: step.kind.clone(),
        metric: "parallel_workers".into(),
        artifact: "suite-manifest".into(),
        values: add_result_context(
            manifest,
            step,
            BTreeMap::from([
                ("before".into(), before.to_string()),
                ("after".into(), after.to_string()),
                ("delta".into(), delta.to_string()),
            ]),
        ),
    })
}

fn kernel_cell_result_row(manifest: &SuiteManifest, step: &StepRecord) -> Option<ResultRow> {
    let status = step.kernel_status?;
    if status == KernelCellStatus::Valid {
        return None;
    }
    Some(ResultRow {
        suite: manifest.suite.clone(),
        step: step.name.clone(),
        kind: step.kind.clone(),
        metric: "kernel_cell".into(),
        artifact: "suite-manifest".into(),
        values: add_result_context(
            manifest,
            step,
            BTreeMap::from([("kernel_status".into(), kernel_status_label(status).into())]),
        ),
    })
}

async fn capture_parallel_workers_from_load_artifacts(step: &StepRecord) -> Result<i64> {
    for artifact in &step.expected_artifacts {
        let path = Path::new(artifact);
        let Ok(raw) = tokio::fs::read_to_string(path).await else {
            continue;
        };
        if let Some(workers_launched) = parse_parallel_workers_from_load_artifact(&raw) {
            return Ok(workers_launched);
        }
    }
    bail!(
        "load step {:?} requested capture_parallel_workers but no supported build timing row with worker count was found in expected artifacts",
        step.name
    )
}

fn parse_parallel_workers_from_load_artifact(raw: &str) -> Option<i64> {
    raw.lines().find_map(|line| {
        if let Some(values) = parse_ec_ivf_build_timing_line(line) {
            return values.get("workers_launched")?.parse::<i64>().ok();
        }
        if let Some(values) = parse_ec_diskann_build_timing_line(line) {
            return values
                .get("parallel_effective_workers")?
                .parse::<i64>()
                .ok();
        }
        None
    })
}

fn parse_result_rows(
    manifest: &SuiteManifest,
    step: &StepRecord,
    artifact: &str,
    raw: &str,
) -> Vec<ResultRow> {
    match step.kind.as_str() {
        "recall" | "latency" | "sidecar-rerank" | "spire-pipeline" => {
            let mut rows: Vec<ResultRow> = parse_table_rows(raw)
                .into_iter()
                .map(|values| ResultRow {
                    suite: manifest.suite.clone(),
                    step: step.name.clone(),
                    kind: step.kind.clone(),
                    metric: step.kind.clone(),
                    artifact: artifact.into(),
                    values: add_result_context(manifest, step, values),
                })
                .collect();
            if step.kind == "latency" {
                rows.extend(
                    parse_block_kernel_counter_rows(raw)
                        .into_iter()
                        .map(|values| ResultRow {
                            suite: manifest.suite.clone(),
                            step: step.name.clone(),
                            kind: step.kind.clone(),
                            metric: "block_kernel_counters".into(),
                            artifact: artifact.into(),
                            values: add_result_context(manifest, step, values),
                        }),
                );
            }
            rows
        }
        "cross-am" => parse_table_rows(raw)
            .into_iter()
            .map(|values| ResultRow {
                suite: manifest.suite.clone(),
                step: step.name.clone(),
                kind: step.kind.clone(),
                metric: "cross_am".into(),
                artifact: artifact.into(),
                values: add_result_context(manifest, step, values),
            })
            .collect(),
        "storage" => parse_storage_rows(raw)
            .into_iter()
            .map(|(metric, values)| ResultRow {
                suite: manifest.suite.clone(),
                step: step.name.clone(),
                kind: step.kind.clone(),
                metric,
                artifact: artifact.into(),
                values: add_result_context(manifest, step, values),
            })
            .collect(),
        "load" => parse_load_rows(raw)
            .into_iter()
            .map(|(metric, values)| ResultRow {
                suite: manifest.suite.clone(),
                step: step.name.clone(),
                kind: step.kind.clone(),
                metric,
                artifact: artifact.into(),
                values: add_result_context(manifest, step, values),
            })
            .collect(),
        "comparator" => {
            let mut rows: Vec<ResultRow> = parse_comparator_table_rows(raw)
                .into_iter()
                .map(|values| ResultRow {
                    suite: manifest.suite.clone(),
                    step: step.name.clone(),
                    kind: step.kind.clone(),
                    metric: "comparator".into(),
                    artifact: artifact.into(),
                    values: add_result_context(manifest, step, values),
                })
                .collect();
            rows.extend(
                parse_comparator_summary_rows(raw)
                    .into_iter()
                    .map(|(metric, values)| ResultRow {
                        suite: manifest.suite.clone(),
                        step: step.name.clone(),
                        kind: step.kind.clone(),
                        metric,
                        artifact: artifact.into(),
                        values: add_result_context(manifest, step, values),
                    }),
            );
            rows
        }
        "explain" => parse_explain_rows(raw)
            .into_iter()
            .map(|(metric, values)| ResultRow {
                suite: manifest.suite.clone(),
                step: step.name.clone(),
                kind: step.kind.clone(),
                metric,
                artifact: artifact.into(),
                values: add_result_context(manifest, step, values),
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn add_result_context(
    manifest: &SuiteManifest,
    step: &StepRecord,
    mut values: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    insert_if_absent(
        &mut values,
        "suite_database",
        Some(manifest.connection.database.as_str()),
    );
    insert_if_absent(
        &mut values,
        "suite_host",
        manifest.connection.host.as_deref().or(Some("local_socket")),
    );
    insert_if_absent(
        &mut values,
        "suite_port",
        manifest
            .connection
            .port
            .map(|port| port.to_string())
            .as_deref(),
    );
    insert_if_absent(
        &mut values,
        "socket_dir",
        command_flag_value(&step.command, "--socket-dir").as_deref(),
    );
    insert_if_absent(
        &mut values,
        "prefix",
        command_flag_value(&step.command, "--prefix").as_deref(),
    );
    insert_if_absent(
        &mut values,
        "profile",
        command_flag_value(&step.command, "--profile").as_deref(),
    );
    insert_if_absent(&mut values, "quant", step.quant.as_deref());
    insert_if_absent(&mut values, "isa", step.isa.as_deref());
    let kernel_status = step.kernel_status.map(kernel_status_label);
    insert_if_absent(&mut values, "kernel_status", kernel_status);
    insert_if_absent(
        &mut values,
        "storage_format",
        command_flag_value(&step.command, "--storage-format")
            .or_else(|| known_tag_value(&step.tags, &["rabitq", "pq_fastscan", "turboquant"]))
            .as_deref(),
    );
    insert_if_absent(
        &mut values,
        "cache_state",
        command_flag_value(&step.command, "--cache-state")
            .or_else(|| known_tag_value(&step.tags, &["post_recall_warm", "warm", "cold"]))
            .as_deref(),
    );
    values
}

fn insert_if_absent(values: &mut BTreeMap<String, String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        values
            .entry(key.to_owned())
            .or_insert_with(|| value.to_owned());
    }
}

fn step_quant(tags: &[String]) -> Option<String> {
    tag_value(tags, "quant=")
        .or_else(|| known_tag_value(tags, &["rabitq", "pq_fastscan", "turboquant"]))
}

fn step_isa(tags: &[String]) -> Option<String> {
    tag_value(tags, "isa=")
}

fn step_kernel_status(tags: &[String]) -> Result<Option<KernelCellStatus>> {
    tag_value(tags, "kernel_status=")
        .map(|value| parse_kernel_status(&value))
        .transpose()
}

fn tag_value(tags: &[String], prefix: &str) -> Option<String> {
    tags.iter()
        .find_map(|tag| tag.strip_prefix(prefix).map(ToOwned::to_owned))
}

fn parse_kernel_status(value: &str) -> Result<KernelCellStatus> {
    match value {
        "valid" => Ok(KernelCellStatus::Valid),
        "missing_kernel" => Ok(KernelCellStatus::MissingKernel),
        "structurally_absent" => Ok(KernelCellStatus::StructurallyAbsent),
        "invalid_config" => Ok(KernelCellStatus::InvalidConfig),
        "retired" => Ok(KernelCellStatus::Retired),
        other => bail!(
            "kernel_status tag must be one of valid, missing_kernel, structurally_absent, invalid_config, retired; got {other:?}"
        ),
    }
}

fn kernel_cell_is_runnable(status: Option<KernelCellStatus>) -> bool {
    matches!(
        status,
        None | Some(KernelCellStatus::Valid) | Some(KernelCellStatus::Retired)
    )
}

fn kernel_status_label(status: KernelCellStatus) -> &'static str {
    match status {
        KernelCellStatus::Valid => "valid",
        KernelCellStatus::MissingKernel => "missing_kernel",
        KernelCellStatus::StructurallyAbsent => "structurally_absent",
        KernelCellStatus::InvalidConfig => "invalid_config",
        KernelCellStatus::Retired => "retired",
    }
}

fn command_flag_value(command: &[String], flag: &str) -> Option<String> {
    command
        .windows(2)
        .find(|window| window.first().map(String::as_str) == Some(flag))
        .and_then(|window| window.get(1))
        .cloned()
}

fn known_tag_value(tags: &[String], known: &[&str]) -> Option<String> {
    known
        .iter()
        .find(|candidate| tags.iter().any(|tag| tag == **candidate))
        .map(|candidate| (*candidate).to_owned())
}

fn parse_table_rows(raw: &str) -> Vec<BTreeMap<String, String>> {
    let mut header: Option<Vec<String>> = None;
    let mut rows = Vec::new();
    for line in raw.lines() {
        let Some(cells) = table_cells(line) else {
            if is_table_boundary_line(line) {
                continue;
            }
            header = None;
            continue;
        };
        if cells.is_empty() || cells.iter().any(|cell| cell.chars().all(|ch| ch == '═')) {
            continue;
        }
        if header.as_ref().map(|h| h.len()) != Some(cells.len()) {
            header = Some(cells);
            continue;
        }
        if header.as_deref() == Some(cells.as_slice()) {
            continue;
        }

        if let Some(header) = &header {
            rows.push(
                header
                    .iter()
                    .cloned()
                    .zip(cells.into_iter())
                    .collect::<BTreeMap<_, _>>(),
            );
        }
    }
    rows
}

fn parse_block_kernel_counter_rows(raw: &str) -> Vec<BTreeMap<String, String>> {
    raw.lines()
        .filter_map(|line| {
            line.trim_start()
                .strip_prefix("[block-kernel-counters] ")
                .and_then(parse_space_key_values)
        })
        .collect()
}

fn parse_space_key_values(rest: &str) -> Option<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for part in rest.split_whitespace() {
        let (key, value) = part.split_once('=')?;
        values.insert(key.to_owned(), value.to_owned());
    }
    (!values.is_empty()).then_some(values)
}

fn parse_storage_rows(raw: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut rows = Vec::new();
    for table_row in parse_table_rows(raw) {
        if let (Some(field), Some(value)) = (table_row.get("field"), table_row.get("value")) {
            let mut values = BTreeMap::from([
                ("field".into(), field.clone()),
                ("value".into(), value.clone()),
            ]);
            if let Some(bytes) = parse_byte_value(value) {
                values.insert("value_bytes".into(), format!("{bytes:.0}"));
            }
            rows.push(("storage_field".into(), values));
        } else if table_row.contains_key("index") {
            let mut values = table_row;
            if let Some(bytes) = values.get("size").and_then(|value| parse_byte_value(value)) {
                values.insert("size_bytes".into(), format!("{bytes:.0}"));
            }
            if let Some(bytes) = values
                .get("per row")
                .and_then(|value| parse_byte_value(value))
            {
                values.insert("per_row_bytes".into(), format!("{bytes:.1}"));
            }
            rows.push(("storage_index".into(), values));
        }
    }
    rows
}

fn parse_byte_value(value: &str) -> Option<f64> {
    let mut parts = value.split_whitespace();
    let amount = parts.next()?.parse::<f64>().ok()?;
    let unit = parts.next().unwrap_or("B");
    let multiplier = match unit {
        "B" => 1.0,
        "KiB" => 1024.0,
        "MiB" => 1024.0 * 1024.0,
        "GiB" => 1024.0 * 1024.0 * 1024.0,
        "TiB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some(amount * multiplier)
}

fn parse_comparator_table_rows(raw: &str) -> Vec<BTreeMap<String, String>> {
    parse_table_rows(raw)
        .into_iter()
        .filter(|row| row.contains_key("engine") && row.contains_key("recall@k"))
        .collect()
}

fn parse_comparator_summary_rows(raw: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut rows = Vec::new();
    for line in raw.lines() {
        if let Some((name, seconds)) = parse_comparator_timed_line(line, "built ") {
            rows.push((
                "comparator_build".into(),
                BTreeMap::from([("subject".into(), name), ("seconds".into(), seconds)]),
            ));
        } else if let Some((name, bytes)) = parse_comparator_size_line(line) {
            rows.push((
                "comparator_index_size".into(),
                BTreeMap::from([("subject".into(), name), ("bytes".into(), bytes)]),
            ));
        }
    }
    rows
}

fn parse_comparator_timed_line(line: &str, prefix: &str) -> Option<(String, String)> {
    let rest = line
        .trim_start()
        .strip_prefix("[comparator] ")?
        .strip_prefix(prefix)?;
    let (name, duration) = rest.rsplit_once(" in ")?;
    Some((name.trim().into(), duration_seconds(duration.trim())?))
}

fn parse_comparator_size_line(line: &str) -> Option<(String, String)> {
    let rest = line.trim_start().strip_prefix("[comparator] ")?;
    let (name, bytes) = rest.rsplit_once(" pg_relation_size=")?;
    let bytes = bytes.strip_suffix(" bytes")?.trim();
    bytes.parse::<u64>().ok()?;
    Some((name.trim().into(), bytes.into()))
}

fn parse_load_rows(raw: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut rows = Vec::new();
    for line in raw.lines() {
        if let Some((name, seconds)) = parse_timed_loader_line(line, "copied corpus table ") {
            rows.push((
                "load_timing".into(),
                timed_values("copy_corpus", &name, seconds),
            ));
        } else if let Some((name, seconds)) = parse_timed_loader_line(line, "encoded corpus table ")
        {
            rows.push((
                "load_timing".into(),
                timed_values("encode_corpus", &name, seconds),
            ));
        } else if let Some((name, seconds)) = parse_timed_loader_line(line, "copied queries table ")
        {
            rows.push((
                "load_timing".into(),
                timed_values("copy_queries", &name, seconds),
            ));
        } else if let Some((name, seconds)) = parse_timed_loader_line(line, "built ") {
            rows.push((
                "load_timing".into(),
                timed_values("build_index", &name, seconds),
            ));
        } else if let Some(values) = parse_ec_ivf_build_timing_line(line) {
            rows.push(("ec_ivf_build_timing".into(), values));
        } else if let Some(values) = parse_ec_diskann_build_timing_line(line) {
            rows.push(("ec_diskann_build_timing".into(), values));
        } else if let Some((name, seconds)) = parse_timed_loader_line(line, "completed prefix ") {
            rows.push(("load_timing".into(), timed_values("total", &name, seconds)));
        }
    }
    rows
}

fn parse_ec_ivf_build_timing_line(line: &str) -> Option<BTreeMap<String, String>> {
    let rest = line
        .trim_start()
        .strip_prefix("[loader] ec_ivf build timing: ")?;
    parse_integer_key_values(rest)
}

fn parse_ec_diskann_build_timing_line(line: &str) -> Option<BTreeMap<String, String>> {
    let rest = line
        .trim_start()
        .strip_prefix("[loader] ")?
        .strip_prefix("ec_diskann_ambuild_timing ")?;
    parse_integer_key_values(rest)
}

fn parse_integer_key_values(rest: &str) -> Option<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for part in rest.split_whitespace() {
        let (key, value) = part.split_once('=')?;
        if value.parse::<i64>().is_err() {
            continue;
        }
        values.insert(key.to_owned(), value.to_owned());
    }
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

fn parse_explain_rows(raw: &str) -> Vec<(String, BTreeMap<String, String>)> {
    parse_table_rows(raw)
        .into_iter()
        .filter(|row| row.contains_key("modeled_total_cost"))
        .map(|row| ("planner_cost".into(), row))
        .collect()
}

fn parse_timed_loader_line(line: &str, prefix: &str) -> Option<(String, String)> {
    let rest = line
        .trim_start()
        .strip_prefix("[loader] ")?
        .strip_prefix(prefix)?;
    let (name, duration) = rest.rsplit_once(" in ")?;
    Some((name.trim().into(), duration_seconds(duration.trim())?))
}

fn timed_values(phase: &str, subject: &str, seconds: String) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("phase".into(), phase.into()),
        ("subject".into(), subject.into()),
        ("seconds".into(), seconds),
    ])
}

fn duration_seconds(value: &str) -> Option<String> {
    let value = value.trim();
    let split_at = value
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .unwrap_or(value.len());
    let amount = value[..split_at].parse::<f64>().ok()?;
    let unit = value[split_at..].trim();
    let seconds = match unit {
        "ms" => amount / 1000.0,
        "" | "s" => amount,
        "m" | "min" => amount * 60.0,
        _ => return None,
    };
    Some(format!("{seconds:.6}"))
}

fn is_table_boundary_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && trimmed.chars().all(|ch| {
            matches!(
                ch,
                '┌' | '┬'
                    | '┐'
                    | '╞'
                    | '╪'
                    | '╡'
                    | '├'
                    | '┼'
                    | '┤'
                    | '└'
                    | '┴'
                    | '┘'
                    | '─'
                    | '═'
                    | '╌'
                    | '+'
                    | '-'
                    | ' '
            )
        })
}

fn table_cells(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if trimmed.starts_with('│') {
        return Some(
            trimmed
                .trim_matches('│')
                .split('┆')
                .flat_map(|part| part.split('│'))
                .map(|cell| cell.trim().to_string())
                .filter(|cell| !cell.is_empty())
                .collect(),
        );
    }
    if !trimmed.contains('|') {
        return None;
    }
    if trimmed
        .chars()
        .all(|ch| ch == '-' || ch == '+' || ch.is_whitespace())
    {
        return None;
    }
    Some(
        trimmed
            .split('|')
            .map(|cell| cell.trim().to_string())
            .filter(|cell| !cell.is_empty())
            .collect(),
    )
}

async fn write_results_jsonl(path: &Path, rows: &[ResultRow]) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    let mut body = String::new();
    for row in rows {
        body.push_str(&serde_json::to_string(row)?);
        body.push('\n');
    }
    tokio::fs::write(path, body)
        .await
        .wrap_err_with(|| format!("writing {}", path.display()))
}

fn format_metric_values(values: &BTreeMap<String, String>) -> String {
    values
        .iter()
        .map(|(key, value)| format!("`{key}={value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Debug, Clone)]
struct SpirePoolingGateRow {
    step: String,
    nprobe: String,
    connect_p50_ms: f64,
    connect_p95_ms: f64,
    latency_p95_ms: Option<f64>,
}

impl SpirePoolingGateRow {
    fn triggered_by_p50(&self) -> bool {
        self.connect_p50_ms >= 1.0
    }

    fn connect_latency_ratio(&self) -> Option<f64> {
        let latency_p95_ms = self.latency_p95_ms?;
        if latency_p95_ms <= 0.0 {
            return None;
        }
        Some(self.connect_p95_ms / latency_p95_ms)
    }

    fn triggered_by_ratio(&self) -> bool {
        self.connect_latency_ratio()
            .map(|ratio| ratio >= 0.15)
            .unwrap_or(false)
    }

    fn decision(&self) -> &'static str {
        if self.triggered_by_p50() || self.triggered_by_ratio() {
            "pooling_candidate"
        } else if self.latency_p95_ms.is_some() {
            "pooling_not_justified"
        } else {
            "missing_latency_p95"
        }
    }
}

fn render_spire_pooling_gate_section(rows: &[ResultRow]) -> Option<String> {
    let gate_rows = spire_pooling_gate_rows(rows);
    if gate_rows.is_empty() {
        return None;
    }

    let mut body = String::from("## SPIRE Connection Pooling Gate\n\n");
    body.push_str("| Step | nprobe | connect_p50_ms | connect_p95_ms | latency_p95_ms | connect_p95/read_p95 | decision |\n");
    body.push_str("| --- | ---: | ---: | ---: | ---: | ---: | --- |\n");
    for row in gate_rows {
        let latency = row
            .latency_p95_ms
            .map(|value| format!("{value:.3}"))
            .unwrap_or_else(|| "-".into());
        let ratio = row
            .connect_latency_ratio()
            .map(|value| format!("{value:.4}"))
            .unwrap_or_else(|| "-".into());
        body.push_str(&format!(
            "| {} | {} | {:.3} | {:.3} | {} | {} | {} |\n",
            row.step,
            row.nprobe,
            row.connect_p50_ms,
            row.connect_p95_ms,
            latency,
            ratio,
            row.decision()
        ));
    }
    Some(body.trim_end().to_owned())
}

fn spire_pooling_gate_rows(rows: &[ResultRow]) -> Vec<SpirePoolingGateRow> {
    let mut latency_by_step_nprobe = BTreeMap::<(String, String), f64>::new();
    for row in rows {
        if row.kind != "spire-pipeline" {
            continue;
        }
        let Some(nprobe) = row.values.get("nprobe") else {
            continue;
        };
        let Some(latency_p95_ms) = row
            .values
            .get("latency_p95")
            .and_then(|value| parse_duration_ms(value))
        else {
            continue;
        };
        latency_by_step_nprobe.insert((row.step.clone(), nprobe.clone()), latency_p95_ms);
    }

    rows.iter()
        .filter(|row| row.kind == "spire-pipeline")
        .filter_map(|row| {
            let nprobe = row.values.get("nprobe")?.clone();
            let connect_p50_ms = row
                .values
                .get("connect_p50")
                .and_then(|value| parse_duration_ms(value))?;
            let connect_p95_ms = row
                .values
                .get("connect_p95")
                .and_then(|value| parse_duration_ms(value))?;
            let latency_p95_ms = latency_by_step_nprobe
                .get(&(row.step.clone(), nprobe.clone()))
                .copied();
            Some(SpirePoolingGateRow {
                step: row.step.clone(),
                nprobe,
                connect_p50_ms,
                connect_p95_ms,
                latency_p95_ms,
            })
        })
        .collect()
}

fn parse_duration_ms(value: &str) -> Option<f64> {
    let value = value.trim();
    let split_at = value
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .unwrap_or(value.len());
    let amount = value[..split_at].parse::<f64>().ok()?;
    let unit = value[split_at..].trim();
    match unit {
        "ms" => Some(amount),
        "" => Some(amount),
        "s" => Some(amount * 1000.0),
        "m" | "min" => Some(amount * 60_000.0),
        _ => None,
    }
}

fn selected_step_names(manifest: &SuiteManifest) -> HashSet<&str> {
    manifest
        .steps
        .iter()
        .filter(|step| step.selected)
        .map(|step| step.name.as_str())
        .collect()
}

#[cfg(test)]
fn evaluate_thresholds(thresholds: &[ThresholdConfig], rows: &[ResultRow]) -> Vec<ThresholdResult> {
    let selected_steps: HashSet<&str> = thresholds
        .iter()
        .map(|threshold| threshold.step.as_str())
        .collect();
    evaluate_thresholds_for_steps(thresholds, rows, &selected_steps)
}

fn evaluate_thresholds_for_steps(
    thresholds: &[ThresholdConfig],
    rows: &[ResultRow],
    selected_steps: &HashSet<&str>,
) -> Vec<ThresholdResult> {
    thresholds
        .iter()
        .filter(|threshold| selected_steps.contains(threshold.step.as_str()))
        .map(|threshold| evaluate_threshold(threshold, rows))
        .collect()
}

fn evaluate_threshold(threshold: &ThresholdConfig, rows: &[ResultRow]) -> ThresholdResult {
    let actual = rows
        .iter()
        .filter(|row| row.step == threshold.step && row.metric == threshold.metric)
        .filter(|row| {
            threshold.filters.iter().all(|(key, value)| {
                row.values
                    .get(key)
                    .map(|actual| actual == value)
                    .unwrap_or(false)
            })
        })
        .filter_map(|row| row.values.get(&threshold.field))
        .filter_map(|value| parse_numeric_prefix(value))
        .next();
    let passed = actual
        .map(|actual| compare_threshold(actual, threshold.op, threshold.value))
        .unwrap_or(false);
    ThresholdResult {
        name: threshold.name.clone(),
        step: threshold.step.clone(),
        metric: threshold.metric.clone(),
        filters: threshold.filters.clone(),
        field: threshold.field.clone(),
        op: threshold.op,
        expected: threshold.value,
        actual,
        passed,
        message: match actual {
            Some(actual) => format!(
                "{} {} {:?} {} -> {}",
                threshold.field, actual, threshold.op, threshold.value, passed
            ),
            None => format!(
                "no result row for step={}, metric={}, filters={:?}, field={}",
                threshold.step, threshold.metric, threshold.filters, threshold.field
            ),
        },
    }
}

fn compare_threshold(actual: f64, op: ThresholdOp, expected: f64) -> bool {
    match op {
        ThresholdOp::Gt => actual > expected,
        ThresholdOp::Gte => actual >= expected,
        ThresholdOp::Lt => actual < expected,
        ThresholdOp::Lte => actual <= expected,
        ThresholdOp::Eq => (actual - expected).abs() < f64::EPSILON,
    }
}

fn parse_numeric_prefix(value: &str) -> Option<f64> {
    let value = value.trim();
    let split_at = value
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.' || ch == '-'))
        .unwrap_or(value.len());
    value[..split_at].parse().ok()
}

fn validate_config(config: &SuiteConfig) -> Result<()> {
    if config.schema_version != 1 {
        bail!(
            "unsupported suite schema_version {}; supported: 1",
            config.schema_version
        );
    }
    if config.steps.is_empty() {
        bail!("suite {:?} has no steps", config.name);
    }
    validate_profile_name("suite defaults profile", config.defaults.profile.as_deref())?;
    let mut names = HashSet::new();
    for step in &config.steps {
        if !names.insert(step.name()) {
            bail!("duplicate suite step name {:?}", step.name());
        }
        step.validate()?;
    }
    Ok(())
}

impl SuiteStep {
    fn name(&self) -> &str {
        match self {
            SuiteStep::CorpusFetch(step) => &step.name,
            SuiteStep::CorpusPrepare(step) => &step.name,
            SuiteStep::Load(step) => &step.name,
            SuiteStep::Recall(step) => &step.name,
            SuiteStep::CrossAm(step) => &step.name,
            SuiteStep::Latency(step) => &step.name,
            SuiteStep::SpireLocalMultinode(step) => &step.name,
            SuiteStep::SpirePipeline(step) => &step.name,
            SuiteStep::Storage(step) => &step.name,
            SuiteStep::Explain(step) => &step.name,
            SuiteStep::SidecarRerank(step) => &step.name,
            SuiteStep::Comparator(step) => &step.name,
            SuiteStep::Raw(step) => &step.name,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            SuiteStep::CorpusFetch(_) => "corpus-fetch",
            SuiteStep::CorpusPrepare(_) => "corpus-prepare",
            SuiteStep::Load(_) => "load",
            SuiteStep::Recall(_) => "recall",
            SuiteStep::CrossAm(_) => "cross-am",
            SuiteStep::Latency(_) => "latency",
            SuiteStep::SpireLocalMultinode(_) => "spire-local-multinode",
            SuiteStep::SpirePipeline(_) => "spire-pipeline",
            SuiteStep::Storage(_) => "storage",
            SuiteStep::Explain(_) => "explain",
            SuiteStep::SidecarRerank(_) => "sidecar-rerank",
            SuiteStep::Comparator(_) => "comparator",
            SuiteStep::Raw(_) => "raw",
        }
    }

    fn tags(&self) -> &[String] {
        match self {
            SuiteStep::CorpusFetch(step) => &step.tags,
            SuiteStep::CorpusPrepare(step) => &step.tags,
            SuiteStep::Load(step) => &step.tags,
            SuiteStep::Recall(step) => &step.tags,
            SuiteStep::CrossAm(step) => &step.tags,
            SuiteStep::Latency(step) => &step.tags,
            SuiteStep::SpireLocalMultinode(step) => &step.tags,
            SuiteStep::SpirePipeline(step) => &step.tags,
            SuiteStep::Storage(step) => &step.tags,
            SuiteStep::Explain(step) => &step.tags,
            SuiteStep::SidecarRerank(step) => &step.tags,
            SuiteStep::Comparator(step) => &step.tags,
            SuiteStep::Raw(step) => &step.tags,
        }
    }

    fn pgoptions(&self) -> Option<&str> {
        match self {
            SuiteStep::Load(step) => step.pgoptions.as_deref(),
            SuiteStep::Latency(step) => step.pgoptions.as_deref(),
            SuiteStep::SpireLocalMultinode(step) => step.pgoptions.as_deref(),
            SuiteStep::SpirePipeline(step) => step.pgoptions.as_deref(),
            _ => None,
        }
    }

    fn validate(&self) -> Result<()> {
        match self {
            SuiteStep::CorpusPrepare(step) => {
                if step.dim == Some(0) {
                    bail!("corpus-prepare step {:?} must set dim >= 1", step.name)
                }
                if step.chunk_rows == Some(0) {
                    bail!(
                        "corpus-prepare step {:?} must set chunk_rows >= 1",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::Load(step) => {
                validate_profile_name("load profile", step.profile.as_deref())?;
                if step.corpus_file.is_none()
                    && step.queries_file.is_none()
                    && step.manifest_file.is_none()
                {
                    bail!(
                        "load step {:?} must include corpus/queries files or a manifest_file",
                        step.name
                    )
                }
                if step.chunked && (step.corpus_file.is_some() || step.queries_file.is_some()) {
                    bail!(
                        "load step {:?} cannot mix chunked manifest loading with corpus/queries files",
                        step.name
                    )
                }
                if step.chunked && step.manifest_file.is_none() {
                    bail!(
                        "load step {:?} requires manifest_file when chunked=true",
                        step.name
                    )
                }
                if !step.chunked && (step.corpus_file.is_none() || step.queries_file.is_none()) {
                    bail!(
                        "load step {:?} requires corpus_file and queries_file unless chunked=true",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::Recall(step) => {
                validate_profile_name("recall profile", step.profile.as_deref())?;
                if step.sweep.is_empty() {
                    bail!(
                        "recall step {:?} must include at least one sweep value",
                        step.name
                    )
                }
                if step.truth_cache_file.is_some() && step.truth_cache_dir.is_some() {
                    bail!(
                        "recall step {:?} cannot set both truth_cache_file and truth_cache_dir",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::CrossAm(step) => {
                if step.inputs.len() < 2 {
                    bail!(
                        "cross-am step {:?} must include at least two input entries",
                        step.name
                    )
                }
                for input in &step.inputs {
                    let Some((label, path)) = input.split_once('=') else {
                        bail!(
                            "cross-am step {:?} input {:?} must use label=path",
                            step.name,
                            input
                        );
                    };
                    if label.trim().is_empty() || path.trim().is_empty() {
                        bail!(
                            "cross-am step {:?} input {:?} must include non-empty label and path",
                            step.name,
                            input
                        );
                    }
                }
                if step.k == Some(0) {
                    bail!("cross-am step {:?} must set k >= 1", step.name)
                }
                if step.log_output.is_none() {
                    bail!(
                        "cross-am step {:?} must set log_output or suite artifact_dir",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::Latency(step) => {
                validate_profile_name("latency profile", step.profile.as_deref())?;
                if step.sweep.is_empty() {
                    bail!(
                        "latency step {:?} must include at least one sweep value",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::SpireLocalMultinode(step) => {
                if step.pg.unwrap_or(18) != 18 {
                    bail!(
                        "spire-local-multinode step {:?} requires pg=18, got {}",
                        step.name,
                        step.pg.unwrap_or(18)
                    )
                }
                if let Some(tier) = step.tier.as_deref() {
                    if !matches!(tier, "correctness" | "representative") {
                        bail!(
                            "spire-local-multinode step {:?} tier {:?} must be correctness or representative",
                            step.name,
                            tier
                        )
                    }
                }
                validate_optional_nonempty(
                    "spire-local-multinode storage_format",
                    step.storage_format.as_deref(),
                )?;
                validate_optional_nonempty(
                    "spire-local-multinode coord_index",
                    step.coord_index.as_deref(),
                )?;
                validate_optional_nonempty(
                    "spire-local-multinode remote_index",
                    step.remote_index.as_deref(),
                )?;
                validate_reloption_list("spire-local-multinode reloptions", &step.reloptions)?;
                validate_reloption_list(
                    "spire-local-multinode coord_reloptions",
                    &step.coord_reloptions,
                )?;
                validate_reloption_list(
                    "spire-local-multinode remote_reloptions",
                    &step.remote_reloptions,
                )?;
                if step.bench_top_k == Some(0) {
                    bail!(
                        "spire-local-multinode step {:?} must set bench_top_k >= 1",
                        step.name
                    )
                }
                if step.bench_queries_limit == Some(0) {
                    bail!(
                        "spire-local-multinode step {:?} must set bench_queries_limit >= 1",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::SpirePipeline(step) => {
                if step.sweep.is_empty() {
                    bail!(
                        "spire-pipeline step {:?} must include at least one sweep value",
                        step.name
                    )
                }
                if step.queries_limit == Some(0) {
                    bail!(
                        "spire-pipeline step {:?} must set queries_limit >= 1",
                        step.name
                    )
                }
                if step.top_k.map(|value| value < 0).unwrap_or(false) {
                    bail!("spire-pipeline step {:?} must set top_k >= 0", step.name)
                }
                if step.query_metric_k == Some(0) {
                    bail!(
                        "spire-pipeline step {:?} must set query_metric_k >= 1",
                        step.name
                    )
                }
                if step
                    .remote_requested_epoch
                    .map(|epoch| epoch <= 0)
                    .unwrap_or(false)
                {
                    bail!(
                        "spire-pipeline step {:?} must set remote_requested_epoch > 0",
                        step.name
                    )
                }
                if let Some(mode) = step.remote_tuple_transport.as_deref() {
                    validate_spire_remote_tuple_transport(mode)?;
                }
                Ok(())
            }
            SuiteStep::Explain(step) => {
                validate_profile_name("explain profile", step.profile.as_deref())?;
                if step.sql_file.is_none() || step.log_output.is_none() {
                    bail!(
                        "explain step {:?} must set sql_file/log_output or suite artifact_dir",
                        step.name
                    )
                }
                let profile = profiles::resolve(step.profile.as_deref().unwrap_or("ec_ivf"))
                    .unwrap_or(&profiles::EC_IVF);
                super::validate_ivf_scratch_soa_batch_decode(
                    profile,
                    step.ivf_scratch_soa_batch_decode.unwrap_or(false),
                )
            }
            SuiteStep::SidecarRerank(step) => {
                validate_profile_name("sidecar-rerank profile", step.profile.as_deref())?;
                if step.sweep.is_empty() {
                    bail!(
                        "sidecar-rerank step {:?} must include at least one sweep value",
                        step.name
                    )
                }
                if step.k == Some(0) {
                    bail!("sidecar-rerank step {:?} must set k >= 1", step.name)
                }
                if step.candidate_k == Some(0) {
                    bail!(
                        "sidecar-rerank step {:?} must set candidate_k >= 1",
                        step.name
                    )
                }
                if step.final_rerank_k == Some(0) {
                    bail!(
                        "sidecar-rerank step {:?} must set final_rerank_k >= 1",
                        step.name
                    )
                }
                if step.concurrency == Some(0) {
                    bail!(
                        "sidecar-rerank step {:?} must set concurrency >= 1",
                        step.name
                    )
                }
                Ok(())
            }
            SuiteStep::Comparator(step) => {
                if step.sweep.is_empty() {
                    bail!("comparator step {:?} must include a sweep", step.name)
                }
                if !matches!(
                    step.engine.as_str(),
                    "vchord" | "pgvector-hnsw" | "pgvector-ivfflat" | "pgvectorscale"
                ) {
                    bail!(
                        "comparator step {:?} engine {:?} must be one of vchord, pgvector-hnsw, pgvector-ivfflat, pgvectorscale",
                        step.name,
                        step.engine
                    )
                }
                Ok(())
            }
            SuiteStep::Raw(step) if step.args.is_empty() => {
                bail!("raw step {:?} must include args", step.name)
            }
            _ => Ok(()),
        }
    }

    fn expand(&self, defaults: &SuiteDefaults, conn: &ConnectionOptions) -> Result<Vec<String>> {
        match self {
            SuiteStep::CorpusFetch(step) => Ok(expand_corpus_fetch(step)),
            SuiteStep::CorpusPrepare(step) => Ok(expand_corpus_prepare(step)),
            SuiteStep::Load(step) => Ok(expand_load(step, defaults)),
            SuiteStep::Recall(step) => Ok(expand_recall(step, defaults)),
            SuiteStep::CrossAm(step) => Ok(expand_cross_am(step)),
            SuiteStep::Latency(step) => Ok(expand_latency(step, defaults)),
            SuiteStep::SpireLocalMultinode(step) => {
                Ok(expand_spire_local_multinode(step, defaults))
            }
            SuiteStep::SpirePipeline(step) => Ok(expand_spire_pipeline(step, defaults)),
            SuiteStep::Storage(step) => Ok(expand_storage(step)),
            SuiteStep::Explain(step) => Ok(expand_explain(step, defaults, conn)),
            SuiteStep::SidecarRerank(step) => Ok(expand_sidecar_rerank(step, defaults)),
            SuiteStep::Comparator(step) => Ok(expand_comparator(step, defaults)),
            SuiteStep::Raw(step) => Ok(step.args.clone()),
        }
    }

    fn expected_artifacts(&self) -> Vec<PathBuf> {
        match self {
            SuiteStep::CorpusFetch(step) => vec![step.output_dir.join("ecaz_fetch_manifest.json")],
            SuiteStep::CorpusPrepare(step) => {
                let manifest = step
                    .output_dir
                    .join(format!("{}_manifest.json", step.profile));
                if step.chunk_rows.is_some() {
                    vec![manifest]
                } else {
                    vec![
                        step.output_dir.join(format!("{}_corpus.tsv", step.profile)),
                        step.output_dir
                            .join(format!("{}_queries.tsv", step.profile)),
                        manifest,
                    ]
                }
            }
            SuiteStep::Load(step) => step.log_file.iter().cloned().collect(),
            SuiteStep::Recall(step) => step
                .log_output
                .iter()
                .chain(step.predictions_output.iter())
                .cloned()
                .collect(),
            SuiteStep::CrossAm(step) => step.log_output.iter().cloned().collect(),
            SuiteStep::Latency(step) => step.log_output.iter().cloned().collect(),
            SuiteStep::SpireLocalMultinode(step) => {
                let mut artifacts: Vec<PathBuf> = step.smoke_log.iter().cloned().collect();
                if let Some(run_dir) = &step.run_dir {
                    artifacts.push(run_dir.join("topology.local.json"));
                } else if let Some(run_id) = &step.run_id {
                    artifacts.push(PathBuf::from(format!(
                        "target/spire-local-multinode-{run_id}/topology.local.json"
                    )));
                }
                if let Some(artifact_dir) = &step.artifact_dir {
                    if !step.skip_bench_suite {
                        artifacts.extend([
                            artifact_dir.join("bench-suite/suite-manifest.json"),
                            artifact_dir.join("bench-suite/results.jsonl"),
                        ]);
                    }
                }
                artifacts
            }
            SuiteStep::SpirePipeline(step) => step
                .log_output
                .iter()
                .chain(step.funnel_output.iter())
                .chain(step.stage_containment_output.iter())
                .chain(step.leaf_block_rank_output.iter())
                .chain(step.target_block_rank_output.iter())
                .chain(step.target_candidate_rank_output.iter())
                .chain(step.miss_attribution_output.iter())
                .cloned()
                .collect(),
            SuiteStep::Storage(step) => step.log_file.iter().cloned().collect(),
            SuiteStep::Explain(step) => step
                .sql_file
                .iter()
                .chain(step.log_output.iter())
                .cloned()
                .collect(),
            SuiteStep::SidecarRerank(step) => step.log_output.iter().cloned().collect(),
            SuiteStep::Comparator(step) => step.log_output.iter().cloned().collect(),
            SuiteStep::Raw(step) => step.expected_artifacts.clone(),
        }
    }

    fn input_paths(&self) -> Vec<PathBuf> {
        match self {
            SuiteStep::CorpusPrepare(step) => vec![step.parquet.clone()],
            SuiteStep::Load(step) => {
                let mut paths = Vec::new();
                if let Some(path) = &step.corpus_file {
                    paths.push(path.clone());
                }
                if let Some(path) = &step.queries_file {
                    paths.push(path.clone());
                }
                if let Some(path) = &step.manifest_file {
                    paths.push(path.clone());
                }
                paths
            }
            SuiteStep::CrossAm(step) => step
                .inputs
                .iter()
                .filter_map(|input| input.split_once('=').map(|(_, path)| PathBuf::from(path)))
                .collect(),
            SuiteStep::SpireLocalMultinode(step) => {
                let mut paths = Vec::new();
                if let Some(prepared_dir) = &step.prepared_dir {
                    paths.push(prepared_dir.clone());
                }
                if let Some(path) = &step.bench_truth_corpus_file {
                    paths.push(path.clone());
                }
                paths
            }
            _ => Vec::new(),
        }
    }

    fn produced_paths(&self) -> Vec<PathBuf> {
        match self {
            SuiteStep::CorpusFetch(step) => vec![
                step.output_dir.clone(),
                step.output_dir.join("data"),
                step.output_dir.join("ecaz_fetch_manifest.json"),
            ],
            SuiteStep::CorpusPrepare(step) => {
                let mut paths = vec![
                    step.output_dir.clone(),
                    step.output_dir
                        .join(format!("{}_manifest.json", step.profile)),
                ];
                if step.chunk_rows.is_some() {
                    paths.push(step.output_dir.join(format!("{}_corpus", step.profile)));
                    paths.push(step.output_dir.join(format!("{}_queries", step.profile)));
                } else {
                    paths.push(step.output_dir.join(format!("{}_corpus.tsv", step.profile)));
                    paths.push(
                        step.output_dir
                            .join(format!("{}_queries.tsv", step.profile)),
                    );
                }
                paths
            }
            SuiteStep::Explain(step) => step.sql_file.iter().cloned().collect(),
            SuiteStep::SpireLocalMultinode(step) => step
                .artifact_dir
                .iter()
                .chain(step.run_dir.iter())
                .cloned()
                .collect(),
            SuiteStep::Recall(step) => step
                .log_output
                .iter()
                .chain(step.predictions_output.iter())
                .cloned()
                .collect(),
            _ => Vec::new(),
        }
    }
}

fn validate_profile_name(label: &str, profile_name: Option<&str>) -> Result<()> {
    if let Some(profile_name) = profile_name {
        if profiles::resolve(profile_name).is_none() {
            bail!(
                "{label} {:?} is not registered; known profiles: {}",
                profile_name,
                profiles::names().join(", ")
            );
        }
    }
    Ok(())
}

fn validate_optional_nonempty(label: &str, value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        if value.trim().is_empty() {
            bail!("{label} must not be empty");
        }
    }
    Ok(())
}

fn validate_reloption_list(label: &str, reloptions: &[String]) -> Result<()> {
    for reloption in reloptions {
        if reloption.trim().is_empty() {
            bail!("{label} must not include empty reloptions");
        }
        if reloption.contains(';') {
            bail!("{label} item {:?} must not contain ';'", reloption);
        }
    }
    Ok(())
}

async fn prepare_step(step: &SuiteStep, defaults: &SuiteDefaults) -> Result<()> {
    if let SuiteStep::Explain(step) = step {
        let sql_file = step
            .sql_file
            .as_ref()
            .context("explain step missing sql_file after defaults")?;
        if let Some(parent) = sql_file.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        tokio::fs::write(sql_file, explain_sql(step, defaults))
            .await
            .wrap_err_with(|| format!("writing {}", sql_file.display()))?;
    }
    Ok(())
}

async fn spawn_step(
    exe: &Path,
    args: &[String],
    conn: &ConnectionOptions,
    pgoptions: Option<&str>,
) -> Result<ExitStatus> {
    let mut command = Command::new(exe);
    command.args(args);
    if let Some(password) = &conn.password {
        command.env("PGPASSWORD", password);
    }
    if let Some(pgoptions) = pgoptions {
        command.env("PGOPTIONS", pgoptions);
    }
    let output = command
        .output()
        .await
        .wrap_err_with(|| format!("spawning {}", exe.display()))?;
    std::io::stdout()
        .write_all(&output.stdout)
        .wrap_err("replaying step stdout")?;
    std::io::stdout()
        .write_all(&output.stderr)
        .wrap_err("replaying step stderr")?;
    Ok(output.status)
}

fn child_command_args(conn: &ConnectionOptions, mut step_args: Vec<String>) -> Vec<String> {
    let mut args = Vec::new();
    push_arg(&mut args, "--database", &conn.database);
    if let Some(host) = &conn.host {
        push_arg(&mut args, "--host", host);
    }
    if let Some(port) = conn.port {
        push_arg(&mut args, "--port", &port.to_string());
    }
    if let Some(user) = &conn.user {
        push_arg(&mut args, "--user", user);
    }
    args.append(&mut step_args);
    args
}

fn expand_load(step: &LoadStep, defaults: &SuiteDefaults) -> Vec<String> {
    let mut args = Vec::new();
    push_opt_path(&mut args, "--log-file", step.log_file.as_deref());
    args.extend(["corpus".into(), "load".into()]);
    push_arg(&mut args, "--prefix", &step.prefix);
    push_arg(
        &mut args,
        "--profile",
        &profile(defaults, step.profile.as_deref()),
    );
    push_opt_path(&mut args, "--corpus-file", step.corpus_file.as_deref());
    push_opt_path(&mut args, "--queries-file", step.queries_file.as_deref());
    push_opt_path(&mut args, "--manifest-file", step.manifest_file.as_deref());
    if step.allow_manifest_mismatch {
        args.push("--allow-manifest-mismatch".into());
    }
    if step.chunked {
        args.push("--chunked".into());
    }
    if let Some(dim) = step.dim {
        push_arg(&mut args, "--dim", &dim.to_string());
    }
    push_arg(&mut args, "--bits", &bits(defaults, step.bits).to_string());
    push_arg(&mut args, "--seed", &seed(defaults, step.seed).to_string());
    if !step.m.is_empty() {
        push_arg(&mut args, "--m", &join_i32(&step.m));
    }
    if let Some(ef_construction) = step.ef_construction {
        push_arg(&mut args, "--ef-construction", &ef_construction.to_string());
    }
    if let Some(storage_format) = step.storage_format.as_deref() {
        push_arg(&mut args, "--storage-format", storage_format);
    }
    if let Some(index_name) = step.index_name.as_deref() {
        push_arg(&mut args, "--index-name", index_name);
    }
    for reloption in &step.table_reloptions {
        push_arg(&mut args, "--table-reloption", reloption);
    }
    for reloption in &step.reloptions {
        push_arg(&mut args, "--reloption", reloption);
    }
    args
}

fn expand_corpus_fetch(step: &CorpusFetchStep) -> Vec<String> {
    let mut args = vec!["corpus".into(), "fetch".into()];
    push_arg(&mut args, "--dataset", &step.dataset);
    push_arg_path(&mut args, "--output-dir", &step.output_dir);
    if let Some(revision) = step.revision.as_deref() {
        push_arg(&mut args, "--revision", revision);
    }
    if step.force {
        args.push("--force".into());
    }
    args
}

fn expand_corpus_prepare(step: &CorpusPrepareStep) -> Vec<String> {
    let mut args = vec!["corpus".into(), "prepare".into()];
    push_arg(&mut args, "--profile", &step.profile);
    push_arg_path(&mut args, "--parquet", &step.parquet);
    push_arg_path(&mut args, "--output-dir", &step.output_dir);
    if let Some(id_column) = step.id_column.as_deref() {
        push_arg(&mut args, "--id-column", id_column);
    }
    if let Some(vector_column) = step.vector_column.as_deref() {
        push_arg(&mut args, "--vector-column", vector_column);
    }
    if let Some(dim) = step.dim {
        push_arg(&mut args, "--dim", &dim.to_string());
    }
    if let Some(source_dataset) = step.source_dataset.as_deref() {
        push_arg(&mut args, "--source-dataset", source_dataset);
    }
    if let Some(chunk_rows) = step.chunk_rows {
        push_arg(&mut args, "--chunk-rows", &chunk_rows.to_string());
    }
    args
}

fn expand_recall(step: &RecallStep, defaults: &SuiteDefaults) -> Vec<String> {
    let mut args = vec!["bench".into(), "recall".into()];
    push_arg(&mut args, "--prefix", &step.prefix);
    push_arg(
        &mut args,
        "--profile",
        &profile(defaults, step.profile.as_deref()),
    );
    push_arg(&mut args, "--k", &step.k.to_string());
    push_arg(&mut args, "--sweep", &join_i32(&step.sweep));
    if let Some(width) = step.rerank_width {
        push_arg(&mut args, "--rerank-width", &width.to_string());
    }
    if step.adaptive_nprobe.unwrap_or(false) {
        args.push("--adaptive-nprobe".into());
    }
    if let Some(score_gap_micros) = step.adaptive_nprobe_score_gap_micros {
        push_arg(
            &mut args,
            "--adaptive-nprobe-score-gap-micros",
            &score_gap_micros.to_string(),
        );
    }
    if let Some(score_margin_ratio_bps) = step.adaptive_nprobe_score_margin_ratio_bps {
        push_arg(
            &mut args,
            "--adaptive-nprobe-score-margin-ratio-bps",
            &score_margin_ratio_bps.to_string(),
        );
    }
    if step.ivf_scratch_soa_batch_decode.unwrap_or(false) {
        args.push("--ivf-scratch-soa-batch-decode".into());
    }
    if let Some(limit) = step.queries_limit.or(defaults.queries_limit) {
        push_arg(&mut args, "--queries-limit", &limit.to_string());
    }
    push_arg(&mut args, "--bits", &bits(defaults, step.bits).to_string());
    push_arg(&mut args, "--seed", &seed(defaults, step.seed).to_string());
    if step.force_index.or(defaults.force_index).unwrap_or(false) {
        args.push("--force-index".into());
    }
    for guc in &step.session_gucs {
        push_arg(&mut args, "--session-guc", guc);
    }
    push_opt_path(
        &mut args,
        "--truth-cache-file",
        step.truth_cache_file.as_deref(),
    );
    push_opt_path(
        &mut args,
        "--truth-cache-dir",
        step.truth_cache_dir.as_deref(),
    );
    push_opt_path(
        &mut args,
        "--truth-corpus-file",
        step.truth_corpus_file.as_deref(),
    );
    push_opt_path(&mut args, "--log-output", step.log_output.as_deref());
    push_opt_path(
        &mut args,
        "--predictions-output",
        step.predictions_output.as_deref(),
    );
    args
}

fn expand_cross_am(step: &CrossAmStep) -> Vec<String> {
    let mut args = vec!["bench".into(), "cross-am".into()];
    for input in &step.inputs {
        push_arg(&mut args, "--input", input);
    }
    if let Some(k) = step.k {
        push_arg(&mut args, "--k", &k.to_string());
    }
    if let Some(log_output) = &step.log_output {
        push_arg_path(&mut args, "--log-output", log_output);
    }
    args
}

fn expand_latency(step: &LatencyStep, defaults: &SuiteDefaults) -> Vec<String> {
    let mut args = vec!["bench".into(), "latency".into()];
    push_arg(&mut args, "--prefix", &step.prefix);
    push_arg(
        &mut args,
        "--profile",
        &profile(defaults, step.profile.as_deref()),
    );
    push_arg(&mut args, "--k", &step.k.unwrap_or(10).to_string());
    push_arg(
        &mut args,
        "--concurrency",
        &step.concurrency.unwrap_or(1).to_string(),
    );
    push_arg(
        &mut args,
        "--iterations",
        &step
            .iterations
            .or(defaults.iterations)
            .unwrap_or(1000)
            .to_string(),
    );
    push_arg(&mut args, "--sweep", &join_i32(&step.sweep));
    if let Some(width) = step.rerank_width {
        push_arg(&mut args, "--rerank-width", &width.to_string());
    }
    if step.adaptive_nprobe.unwrap_or(false) {
        args.push("--adaptive-nprobe".into());
    }
    if let Some(score_gap_micros) = step.adaptive_nprobe_score_gap_micros {
        push_arg(
            &mut args,
            "--adaptive-nprobe-score-gap-micros",
            &score_gap_micros.to_string(),
        );
    }
    if let Some(score_margin_ratio_bps) = step.adaptive_nprobe_score_margin_ratio_bps {
        push_arg(
            &mut args,
            "--adaptive-nprobe-score-margin-ratio-bps",
            &score_margin_ratio_bps.to_string(),
        );
    }
    if step.ivf_scratch_soa_batch_decode.unwrap_or(false) {
        args.push("--ivf-scratch-soa-batch-decode".into());
    }
    push_arg(&mut args, "--bits", &bits(defaults, step.bits).to_string());
    push_arg(&mut args, "--seed", &seed(defaults, step.seed).to_string());
    if step.force_index.or(defaults.force_index).unwrap_or(false) {
        args.push("--force-index".into());
    }
    if step
        .sample_backend_memory
        .or(defaults.sample_backend_memory)
        .unwrap_or(false)
    {
        args.push("--sample-backend-memory".into());
    }
    if let Some(cache_state) = &step.cache_state {
        push_arg(&mut args, "--cache-state", cache_state);
    }
    for guc in &step.session_gucs {
        push_arg(&mut args, "--session-guc", guc);
    }
    if step.task87_candidate_batch_counters.unwrap_or(false) {
        args.push("--task87-candidate-batch-counters".into());
    }
    push_arg(
        &mut args,
        "--memory-sample-interval-ms",
        &step
            .memory_sample_interval_ms
            .or(defaults.memory_sample_interval_ms)
            .unwrap_or(25)
            .to_string(),
    );
    push_opt_path(&mut args, "--log-output", step.log_output.as_deref());
    args
}

fn expand_spire_local_multinode(
    step: &SpireLocalMultinodeStep,
    defaults: &SuiteDefaults,
) -> Vec<String> {
    let mut args = vec![
        "dev".into(),
        "spire-multicluster".into(),
        "local-multinode-pg18".into(),
    ];
    push_arg(
        &mut args,
        "--pg",
        &step.pg.or(defaults.pg).unwrap_or(18).to_string(),
    );
    push_opt_path(&mut args, "--pgbin", step.pgbin.as_deref());
    push_opt_path(&mut args, "--artifact-dir", step.artifact_dir.as_deref());
    push_opt_path(&mut args, "--run-dir", step.run_dir.as_deref());
    push_opt_path(&mut args, "--smoke-log", step.smoke_log.as_deref());
    push_opt_u16(&mut args, "--coord-port", step.coord_port);
    push_opt_u16(&mut args, "--remote1-port", step.remote1_port);
    push_opt_u16(&mut args, "--remote2-port", step.remote2_port);
    push_opt_u16(&mut args, "--remote3-port", step.remote3_port);
    push_opt_arg(&mut args, "--run-id", step.run_id.as_deref());
    push_opt_arg(&mut args, "--tier", step.tier.as_deref());
    push_opt_arg(&mut args, "--prefix", step.prefix.as_deref());
    push_opt_arg(
        &mut args,
        "--prepared-prefix",
        step.prepared_prefix.as_deref(),
    );
    push_opt_path(&mut args, "--prepared-dir", step.prepared_dir.as_deref());
    push_opt_arg(
        &mut args,
        "--storage-format",
        step.storage_format.as_deref(),
    );
    push_opt_arg(&mut args, "--coord-index", step.coord_index.as_deref());
    push_opt_arg(&mut args, "--remote-index", step.remote_index.as_deref());
    for reloption in &step.reloptions {
        push_arg(&mut args, "--reloption", reloption);
    }
    for reloption in &step.coord_reloptions {
        push_arg(&mut args, "--coord-reloption", reloption);
    }
    for reloption in &step.remote_reloptions {
        push_arg(&mut args, "--remote-reloption", reloption);
    }
    push_opt_u16(&mut args, "--bench-top-k", step.bench_top_k);
    push_opt_usize(&mut args, "--bench-queries-limit", step.bench_queries_limit);
    push_opt_arg(&mut args, "--bench-sweep", step.bench_sweep.as_deref());
    push_opt_arg(
        &mut args,
        "--bench-rowcap-sweep",
        step.bench_rowcap_sweep.as_deref(),
    );
    if step.skip_bench_rowcap {
        args.push("--skip-bench-rowcap".into());
    }
    push_opt_path(
        &mut args,
        "--bench-truth-corpus-file",
        step.bench_truth_corpus_file.as_deref(),
    );
    if !step.bench_query_metric_projection_columns.is_empty() {
        push_arg(
            &mut args,
            "--bench-query-metric-projection-columns",
            &step.bench_query_metric_projection_columns.join(","),
        );
    }
    for guc in &step.bench_session_gucs {
        push_arg(&mut args, "--bench-session-guc", guc);
    }
    for variant in &step.bench_production_read_variants {
        push_arg(&mut args, "--bench-production-read-variant", variant);
    }
    if step.skip_bench_suite {
        args.push("--skip-bench-suite".into());
    }
    if step.skip_fault_drills {
        args.push("--skip-fault-drills".into());
    }
    if step.skip_install {
        args.push("--skip-install".into());
    }
    args
}

fn expand_spire_pipeline(step: &SpirePipelineStep, defaults: &SuiteDefaults) -> Vec<String> {
    let mut args = vec!["bench".into(), "spire-pipeline".into()];
    push_arg(&mut args, "--prefix", &step.prefix);
    push_opt_arg(&mut args, "--index", step.index.as_deref());
    push_arg(
        &mut args,
        "--queries-limit",
        &step
            .queries_limit
            .or(defaults.queries_limit)
            .unwrap_or(1)
            .to_string(),
    );
    push_arg(&mut args, "--sweep", &join_i32(&step.sweep));
    if let Some(width) = step.rerank_width {
        push_arg(&mut args, "--rerank-width", &width.to_string());
    }
    if let Some(max_candidate_rows) = step.max_candidate_rows {
        push_arg(
            &mut args,
            "--max-candidate-rows",
            &max_candidate_rows.to_string(),
        );
    }
    if let Some(max_routed_candidate_rows) = step.max_routed_candidate_rows {
        push_arg(
            &mut args,
            "--max-routed-candidate-rows",
            &max_routed_candidate_rows.to_string(),
        );
    }
    if step.adaptive_nprobe.unwrap_or(false) {
        args.push("--adaptive-nprobe".into());
    }
    if let Some(score_gap_micros) = step.adaptive_nprobe_score_gap_micros {
        push_arg(
            &mut args,
            "--adaptive-nprobe-score-gap-micros",
            &score_gap_micros.to_string(),
        );
    }
    if step.include_remote.unwrap_or(false) {
        args.push("--include-remote".into());
    }
    if step.require_remote_placements.unwrap_or(false) {
        args.push("--require-remote-placements".into());
    }
    if step.include_local_store_overlap.unwrap_or(false) {
        args.push("--include-local-store-overlap".into());
    }
    if !step.remote_selected_pids.is_empty() {
        push_arg(
            &mut args,
            "--remote-selected-pids",
            &join_i64(&step.remote_selected_pids),
        );
    }
    if let Some(epoch) = step.remote_requested_epoch {
        push_arg(&mut args, "--remote-requested-epoch", &epoch.to_string());
    }
    if let Some(top_k) = step.top_k {
        push_arg(&mut args, "--top-k", &top_k.to_string());
    }
    if let Some(consistency_mode) = step.consistency_mode.as_deref() {
        push_arg(&mut args, "--consistency-mode", consistency_mode);
    }
    if let Some(mode) = step.remote_tuple_transport.as_deref() {
        push_arg(&mut args, "--remote-tuple-transport", mode);
    }
    if step.include_cost_snapshot.unwrap_or(false) {
        args.push("--include-cost-snapshot".into());
    }
    push_opt_f64(
        &mut args,
        "--cost-routing-dimension-scale",
        step.cost_routing_dimension_scale,
    );
    push_opt_f64(
        &mut args,
        "--cost-leaf-dimension-scale",
        step.cost_leaf_dimension_scale,
    );
    push_opt_f64(
        &mut args,
        "--cost-index-page-scale",
        step.cost_index_page_scale,
    );
    push_opt_f64(
        &mut args,
        "--cost-local-store-page-fanout-scale",
        step.cost_local_store_page_fanout_scale,
    );
    push_opt_f64(
        &mut args,
        "--cost-storage-scoring-multiplier",
        step.cost_storage_scoring_multiplier,
    );
    push_opt_f64(
        &mut args,
        "--cost-rerank-multiplier",
        step.cost_rerank_multiplier,
    );
    if step.include_query_metrics.unwrap_or(false) {
        args.push("--include-query-metrics".into());
    }
    if step.include_recall.unwrap_or(false) {
        args.push("--include-recall".into());
    }
    push_opt_path(
        &mut args,
        "--truth-corpus-file",
        step.truth_corpus_file.as_deref(),
    );
    push_opt_path(
        &mut args,
        "--truth-cache-file",
        step.truth_cache_file.as_deref(),
    );
    push_opt_path(
        &mut args,
        "--leaf-block-rank-output",
        step.leaf_block_rank_output.as_deref(),
    );
    push_opt_path(
        &mut args,
        "--target-block-rank-output",
        step.target_block_rank_output.as_deref(),
    );
    push_opt_path(
        &mut args,
        "--target-candidate-rank-output",
        step.target_candidate_rank_output.as_deref(),
    );
    push_opt_path(
        &mut args,
        "--miss-attribution-output",
        step.miss_attribution_output.as_deref(),
    );
    if let Some(offset) = step.leaf_block_rank_local_sequence_offset {
        push_arg(
            &mut args,
            "--leaf-block-rank-local-sequence-offset",
            &offset.to_string(),
        );
    }
    if step.include_production_read_profile.unwrap_or(false) {
        args.push("--include-production-read-profile".into());
    }
    if step.production_read_only.unwrap_or(false) {
        args.push("--production-read-only".into());
    }
    if step.production_read_timeline_no_payload.unwrap_or(false) {
        args.push("--production-read-timeline-no-payload".into());
    }
    if let Some(k) = step.query_metric_k {
        push_arg(&mut args, "--query-metric-k", &k.to_string());
    }
    if !step.query_metric_projection_columns.is_empty() {
        push_arg(
            &mut args,
            "--query-metric-projection-columns",
            &step.query_metric_projection_columns.join(","),
        );
    }
    for guc in &step.session_gucs {
        push_arg(&mut args, "--session-guc", guc);
    }
    if step.task87_candidate_batch_counters.unwrap_or(false) {
        args.push("--task87-candidate-batch-counters".into());
    }
    push_opt_path(&mut args, "--log-output", step.log_output.as_deref());
    push_opt_path(&mut args, "--funnel-output", step.funnel_output.as_deref());
    push_opt_path(
        &mut args,
        "--stage-containment-output",
        step.stage_containment_output.as_deref(),
    );
    args
}

fn expand_storage(step: &StorageStep) -> Vec<String> {
    let mut args = Vec::new();
    push_opt_path(&mut args, "--log-file", step.log_file.as_deref());
    args.extend(["bench".into(), "storage".into()]);
    push_arg(&mut args, "--prefix", &step.prefix);
    args
}

fn expand_explain(
    step: &ExplainStep,
    defaults: &SuiteDefaults,
    conn: &ConnectionOptions,
) -> Vec<String> {
    let mut args = vec!["dev".into(), "sql".into()];
    push_arg(
        &mut args,
        "--pg",
        &step.pg.or(defaults.pg).unwrap_or(18).to_string(),
    );
    push_arg(
        &mut args,
        "--db",
        step.db.as_deref().unwrap_or(&conn.database),
    );
    push_opt_path(
        &mut args,
        "--socket-dir",
        step.socket_dir
            .as_deref()
            .or(defaults.socket_dir.as_deref())
            .or(conn.host.as_deref().map(Path::new)),
    );
    if let Some(port) = step.port.or(conn.port) {
        push_arg(&mut args, "--port", &port.to_string());
    }
    args.push("--raw".into());
    if let Some(sql_file) = &step.sql_file {
        push_arg_path(&mut args, "--file", sql_file);
    }
    if let Some(log_output) = &step.log_output {
        push_arg_path(&mut args, "--log-output", log_output);
    }
    args
}

fn expand_sidecar_rerank(step: &SidecarRerankStep, defaults: &SuiteDefaults) -> Vec<String> {
    let mut args = vec!["bench".into(), "sidecar-rerank".into()];
    push_arg(&mut args, "--prefix", &step.prefix);
    push_arg(
        &mut args,
        "--profile",
        &profile(defaults, step.profile.as_deref()),
    );
    push_arg(&mut args, "--k", &step.k.unwrap_or(10).to_string());
    push_arg(
        &mut args,
        "--candidate-k",
        &step.candidate_k.unwrap_or(50).to_string(),
    );
    if let Some(final_rerank_k) = step.final_rerank_k {
        push_arg(&mut args, "--final-rerank-k", &final_rerank_k.to_string());
    }
    if let Some(concurrency) = step.concurrency {
        push_arg(&mut args, "--concurrency", &concurrency.to_string());
    }
    push_arg(&mut args, "--sweep", &join_i32(&step.sweep));
    if let Some(limit) = step.queries_limit.or(defaults.queries_limit) {
        push_arg(&mut args, "--queries-limit", &limit.to_string());
    }
    if let Some(warmup_queries) = step.warmup_queries {
        push_arg(&mut args, "--warmup-queries", &warmup_queries.to_string());
    }
    push_arg(&mut args, "--bits", &bits(defaults, step.bits).to_string());
    push_arg(&mut args, "--seed", &seed(defaults, step.seed).to_string());
    for variant in &step.variants {
        push_arg(&mut args, "--variant", variant);
    }
    for read_mode in &step.read_modes {
        push_arg(&mut args, "--read-mode", read_mode);
    }
    if step.rebuild_sidecar_table {
        args.push("--rebuild-sidecar-table".into());
    }
    if step.force_index.or(defaults.force_index).unwrap_or(false) {
        args.push("--force-index".into());
    }
    if step.allow_unsafe_index_shape {
        args.push("--allow-unsafe-index-shape".into());
    }
    push_opt_path(&mut args, "--log-output", step.log_output.as_deref());
    args
}

fn expand_comparator(step: &ComparatorStep, defaults: &SuiteDefaults) -> Vec<String> {
    let mut args = Vec::new();
    args.extend(["bench".into(), "comparator".into()]);
    push_arg(&mut args, "--engine", &step.engine);
    push_arg(&mut args, "--prefix", &step.prefix);
    push_arg(&mut args, "--k", &step.k.unwrap_or(10).to_string());
    push_arg(&mut args, "--sweep", &join_i32(&step.sweep));
    if let Some(lists) = step.lists {
        push_arg(&mut args, "--lists", &lists.to_string());
    }
    if let Some(m) = step.m {
        push_arg(&mut args, "--m", &m.to_string());
    }
    if let Some(ef_construction) = step.ef_construction {
        push_arg(&mut args, "--ef-construction", &ef_construction.to_string());
    }
    if let Some(num_neighbors) = step.num_neighbors {
        push_arg(&mut args, "--num-neighbors", &num_neighbors.to_string());
    }
    if let Some(build_search_list_size) = step.build_search_list_size {
        push_arg(
            &mut args,
            "--build-search-list-size",
            &build_search_list_size.to_string(),
        );
    }
    if let Some(max_alpha) = step.max_alpha {
        push_arg(&mut args, "--max-alpha", &max_alpha.to_string());
    }
    if let Some(storage_layout) = step.storage_layout.as_deref() {
        push_arg(&mut args, "--storage-layout", storage_layout);
    }
    if let Some(memory) = step.maintenance_work_mem.as_deref() {
        push_arg(&mut args, "--maintenance-work-mem", memory);
    }
    if let Some(limit) = step.queries_limit.or(defaults.queries_limit) {
        push_arg(&mut args, "--queries-limit", &limit.to_string());
    }
    push_opt_path(&mut args, "--log-output", step.log_output.as_deref());
    if step.rebuild {
        args.push("--rebuild".into());
    }
    args
}

fn explain_sql(step: &ExplainStep, defaults: &SuiteDefaults) -> String {
    let corpus_table = step
        .corpus_table
        .clone()
        .unwrap_or_else(|| format!("{}_corpus", step.prefix));
    let query_table = step
        .query_table
        .clone()
        .unwrap_or_else(|| format!("{}_queries", step.prefix));
    let index = step
        .index_name
        .clone()
        .unwrap_or_else(|| format!("{}_idx", step.prefix));
    let profile = explain_step_profile(step, defaults);
    let scan_guc = profile.ef_search_guc.unwrap_or("ec_ivf.nprobe");
    let rerank_guc = rerank_width_guc(profile);
    let use_scratch_soa =
        profile.name == "ec_ivf" && step.ivf_scratch_soa_batch_decode.unwrap_or(false);
    let session_set_sql = explain_session_guc_set_sql(&step.session_gucs);
    let session_reset_sql = explain_session_guc_reset_sql(&step.session_gucs);
    let set_scratch_soa_sql = if use_scratch_soa {
        "SET ec_ivf.scratch_soa_batch_decode = on;\n".to_owned()
    } else {
        String::new()
    };
    let current_scratch_soa_sql = if use_scratch_soa {
        "current_setting('ec_ivf.scratch_soa_batch_decode') AS scratch_soa_batch_decode,\n           "
            .to_owned()
    } else {
        String::new()
    };
    let reset_scratch_soa_sql = if use_scratch_soa {
        "RESET ec_ivf.scratch_soa_batch_decode;\n".to_owned()
    } else {
        String::new()
    };
    let set_rerank_sql = rerank_guc
        .map(|guc| {
            format!(
                "SET {guc} = {rerank_width};\n",
                rerank_width = step.rerank_width
            )
        })
        .unwrap_or_default();
    let current_rerank_sql = rerank_guc
        .map(|guc| format!("current_setting('{guc}') AS rerank_width,\n           "))
        .unwrap_or_default();
    let reset_rerank_sql = rerank_guc
        .map(|guc| format!("RESET {guc};\n"))
        .unwrap_or_default();
    let cost_snapshot_sql = cost_snapshot_function(profile)
        .map(|function| {
            format!(
                "SELECT *\n\
                 FROM {function}('{index}'::regclass);\n\n"
            )
        })
        .unwrap_or_default();
    let cost_tuning_snapshot_sql = cost_tuning_snapshot_function(profile)
        .map(|function| {
            format!(
                "SELECT *\n\
                 FROM {function}('{index}'::regclass);\n\n"
            )
        })
        .unwrap_or_default();
    format!(
        "\\pset pager off\n\
         \\timing on\n\n\
         SET enable_seqscan = off;\n\
         SET {scan_guc} = {nprobe};\n\
         {session_set_sql}\
         {set_scratch_soa_sql}\
         {set_rerank_sql}\n\
         SELECT\n\
           current_setting('server_version') AS server_version,\n\
           current_setting('{scan_guc}') AS sweep_value,\n\
           {current_scratch_soa_sql}\
           {current_rerank_sql}'{profile_name}' AS profile;\n\n\
         SELECT\n\
           '{index}' AS index_name,\n\
           pg_relation_size('{index}'::regclass) AS index_bytes,\n\
           pg_size_pretty(pg_relation_size('{index}'::regclass)) AS index_size;\n\n\
         {cost_snapshot_sql}\
         {cost_tuning_snapshot_sql}\
         EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)\n\
         SELECT id\n\
         FROM {corpus_table}\n\
         ORDER BY embedding <#> (\n\
           SELECT source\n\
           FROM {query_table}\n\
           ORDER BY id\n\
           LIMIT 1\n\
         )::real[]\n\
         LIMIT 10;\n\n\
         RESET enable_seqscan;\n\
         RESET {scan_guc};\n\
         {session_reset_sql}\
         {reset_scratch_soa_sql}\
         {reset_rerank_sql}",
        nprobe = step.nprobe,
        scan_guc = scan_guc,
        session_set_sql = session_set_sql,
        set_scratch_soa_sql = set_scratch_soa_sql,
        set_rerank_sql = set_rerank_sql,
        current_scratch_soa_sql = current_scratch_soa_sql,
        current_rerank_sql = current_rerank_sql,
        profile_name = profile.name,
        index = index,
        cost_snapshot_sql = cost_snapshot_sql,
        cost_tuning_snapshot_sql = cost_tuning_snapshot_sql,
        corpus_table = corpus_table,
        query_table = query_table,
        reset_scratch_soa_sql = reset_scratch_soa_sql,
        session_reset_sql = session_reset_sql,
        reset_rerank_sql = reset_rerank_sql
    )
}

fn explain_session_guc_set_sql(session_gucs: &[String]) -> String {
    session_gucs
        .iter()
        .filter_map(|guc| guc.split_once('='))
        .map(|(name, value)| format!("SET {name} = {value};\n"))
        .collect()
}

fn explain_session_guc_reset_sql(session_gucs: &[String]) -> String {
    session_gucs
        .iter()
        .filter_map(|guc| guc.split_once('='))
        .map(|(name, _)| format!("RESET {name};\n"))
        .collect()
}

fn explain_step_profile<'a>(
    step: &'a ExplainStep,
    defaults: &'a SuiteDefaults,
) -> &'static IndexProfile {
    let profile_name = step
        .profile
        .as_deref()
        .or(defaults.profile.as_deref())
        .unwrap_or("ec_ivf");
    profiles::resolve(profile_name).unwrap_or(&profiles::EC_IVF)
}

fn rerank_width_guc(profile: &IndexProfile) -> Option<&'static str> {
    match profile.name {
        "ec_ivf" => Some("ec_ivf.rerank_width"),
        "ec_spire" => Some("ec_spire.rerank_width"),
        _ => None,
    }
}

fn cost_snapshot_function(profile: &IndexProfile) -> Option<&'static str> {
    match profile.name {
        "ec_hnsw" => Some("ec_hnsw_index_cost_snapshot"),
        "ec_ivf" => Some("ec_ivf_index_cost_snapshot"),
        "ec_diskann" => Some("ec_diskann_index_cost_snapshot"),
        "ec_spire" => Some("ec_spire_index_cost_snapshot"),
        _ => None,
    }
}

fn cost_tuning_snapshot_function(profile: &IndexProfile) -> Option<&'static str> {
    match profile.name {
        "ec_spire" => Some("ec_spire_index_cost_tuning_snapshot"),
        _ => None,
    }
}

fn manifest_path(args: &SuiteRunOptions, config: &SuiteConfig) -> Option<PathBuf> {
    args.manifest_output.clone().or_else(|| {
        config
            .artifact_dir
            .as_ref()
            .map(|dir| dir.join("suite-manifest.json"))
    })
}

fn profile(defaults: &SuiteDefaults, step_profile: Option<&str>) -> String {
    step_profile
        .or(defaults.profile.as_deref())
        .unwrap_or("ec_hnsw")
        .to_string()
}

fn bits(defaults: &SuiteDefaults, step_bits: Option<i32>) -> i32 {
    step_bits.or(defaults.bits).unwrap_or(4)
}

fn seed(defaults: &SuiteDefaults, step_seed: Option<i64>) -> i64 {
    step_seed.or(defaults.seed).unwrap_or(42)
}

fn join_i32(values: &[i32]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn join_i64(values: &[i64]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn push_arg(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.into());
    args.push(value.into());
}

fn push_opt_arg(args: &mut Vec<String>, flag: &str, value: Option<&str>) {
    if let Some(value) = value {
        push_arg(args, flag, value);
    }
}

fn push_arg_path(args: &mut Vec<String>, flag: &str, value: &Path) {
    push_arg(args, flag, &value.display().to_string());
}

fn push_opt_path(args: &mut Vec<String>, flag: &str, value: Option<&Path>) {
    if let Some(value) = value {
        push_arg_path(args, flag, value);
    }
}

fn push_opt_u16(args: &mut Vec<String>, flag: &str, value: Option<u16>) {
    if let Some(value) = value {
        push_arg(args, flag, &value.to_string());
    }
}

fn push_opt_usize(args: &mut Vec<String>, flag: &str, value: Option<usize>) {
    if let Some(value) = value {
        push_arg(args, flag, &value.to_string());
    }
}

fn push_opt_f64(args: &mut Vec<String>, flag: &str, value: Option<f64>) {
    if let Some(value) = value {
        push_arg(args, flag, &value.to_string());
    }
}

fn validate_spire_remote_tuple_transport(value: &str) -> Result<()> {
    match value {
        "auto" | "json_tuple_payload_v1" | "pg_binary_attr_v1" => Ok(()),
        other => bail!(
            "unsupported spire-pipeline remote_tuple_transport {:?}; supported: auto, json_tuple_payload_v1, pg_binary_attr_v1",
            other
        ),
    }
}

fn manifest_has_release_guarded_steps(manifest: &SuiteManifest) -> bool {
    manifest.steps.iter().any(|step| {
        step.selected
            && matches!(step.status, Some(StepStatus::Pending))
            && matches!(step.kind.as_str(), "latency" | "recall")
    })
}

async fn preflight_backend(conn: &ConnectionOptions) -> Result<BackendPreflight> {
    let client = crate::psql::connect(conn).await?;
    let build_profile = query_backend_build_profile(&client).await?;
    let backend_path = derive_local_pgrx_backend_path(&client).await?;
    let sha256 = match &backend_path {
        Some(path) => Some(sha256_file_hex(path).await?),
        None => None,
    };
    Ok(BackendPreflight {
        build_profile,
        sha256,
        path: backend_path.map(|path| path.display().to_string()),
    })
}

async fn query_backend_build_profile(client: &tokio_postgres::Client) -> Result<String> {
    let row = client
        .query_one("SELECT ecaz_build_profile()", &[])
        .await
        .context("querying ecaz_build_profile(); reinstall/update the extension if missing")?;
    Ok(row.get::<_, String>(0))
}

async fn derive_local_pgrx_backend_path(
    client: &tokio_postgres::Client,
) -> Result<Option<PathBuf>> {
    let row = client
        .query_one("SHOW data_directory", &[])
        .await
        .context("querying data_directory for backend sha256 preflight")?;
    let data_dir = PathBuf::from(row.get::<_, String>(0));
    let Some(data_name) = data_dir.file_name().and_then(|name| name.to_str()) else {
        return Ok(None);
    };
    let Some(pg_major) = data_name.strip_prefix("data-") else {
        return Ok(None);
    };
    let Some(pgrx_home) = data_dir.parent() else {
        return Ok(None);
    };
    let mut candidates = Vec::new();
    for entry in std::fs::read_dir(pgrx_home)
        .wrap_err_with(|| format!("reading pgrx home {}", pgrx_home.display()))?
    {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if file_name.starts_with(&format!("{pg_major}.")) {
            candidates.push(entry.path().join("pgrx-install/lib/postgresql/ecaz.so"));
        }
    }
    candidates.sort();
    Ok(candidates.into_iter().rev().find(|path| path.is_file()))
}

async fn sha256_file_hex(path: &Path) -> Result<String> {
    let bytes = tokio::fs::read(path)
        .await
        .wrap_err_with(|| format!("reading backend {}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn shell_join(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg.chars().all(|c| {
                c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':' | '=')
            }) {
                arg.clone()
            } else {
                format!("{arg:?}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_join_with_pgoptions(args: &[String], pgoptions: Option<&str>) -> String {
    let command = shell_join(args);
    match pgoptions {
        Some(pgoptions) if !pgoptions.is_empty() => {
            format!(
                "PGOPTIONS={} {command}",
                shell_join(&[pgoptions.to_string()])
            )
        }
        _ => command,
    }
}

fn format_exit_status(status: ExitStatus) -> String {
    status
        .code()
        .map(|code| format!("exit code {code}"))
        .unwrap_or_else(|| "signal termination".into())
}

#[derive(Default)]
struct ManifestSummary {
    completed: usize,
    failed: usize,
    skipped: usize,
    dry_run: usize,
    missing_artifacts: usize,
    stale: usize,
}

async fn summarize_manifest(manifest: &SuiteManifest) -> ManifestSummary {
    let mut summary = ManifestSummary::default();
    for step in &manifest.steps {
        match step.status.unwrap_or(if step.selected {
            StepStatus::Pending
        } else {
            StepStatus::Skipped
        }) {
            StepStatus::Succeeded => summary.completed += 1,
            StepStatus::Failed => summary.failed += 1,
            StepStatus::Skipped => summary.skipped += 1,
            StepStatus::DryRun => summary.dry_run += 1,
            StepStatus::Pending => summary.stale += 1,
        }
        if step.selected
            && matches!(step.status, Some(StepStatus::Succeeded))
            && has_missing_artifact(step).await
        {
            summary.missing_artifacts += 1;
        }
    }
    summary
}

async fn has_missing_artifact(step: &StepRecord) -> bool {
    for artifact in &step.expected_artifacts {
        if tokio::fs::metadata(artifact).await.is_err() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct SuiteOnly {
        #[command(flatten)]
        args: SuiteArgs,
    }

    fn conn() -> ConnectionOptions {
        ConnectionOptions {
            database: "postgres".into(),
            host: Some("/tmp/pg".into()),
            port: Some(28818),
            user: None,
            password: Some("secret".into()),
        }
    }

    #[test]
    fn shell_join_with_pgoptions_renders_environment_prefix() {
        let command = vec![
            "--database".into(),
            "postgres".into(),
            "bench".into(),
            "spire-pipeline".into(),
        ];

        let rendered = shell_join_with_pgoptions(
            &command,
            Some("-c ec_spire.remote_search_connection_pool_size=0"),
        );

        assert!(
            rendered.starts_with("PGOPTIONS=\"-c ec_spire.remote_search_connection_pool_size=0\" ")
        );
        assert!(rendered.contains("bench spire-pipeline"));
    }

    #[test]
    fn parses_nested_run_command() {
        let cli = SuiteOnly::try_parse_from([
            "suite",
            "run",
            "--config",
            "suite.json",
            "--dry-run",
            "--only",
            "r10",
            "--only-tag",
            "recall",
            "--resume-from",
            "old-manifest.json",
            "--results-output",
            "results.jsonl",
            "--artifact-dir",
            "artifacts/current",
        ])
        .expect("suite parses");
        match cli.args.command {
            Some(SuiteCommand::Run(args)) => {
                assert_eq!(args.config, PathBuf::from("suite.json"));
                assert!(args.dry_run);
                assert_eq!(args.only, vec!["r10"]);
                assert_eq!(args.only_tag, vec!["recall"]);
                assert_eq!(args.resume_from, Some(PathBuf::from("old-manifest.json")));
                assert_eq!(args.results_output, Some(PathBuf::from("results.jsonl")));
                assert_eq!(args.artifact_dir, Some(PathBuf::from("artifacts/current")));
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn artifact_dir_templates_rewrite_raw_step_paths() {
        let mut config = SuiteConfig {
            name: "current".into(),
            schema_version: 1,
            artifact_dir: Some("artifacts/current".into()),
            defaults: SuiteDefaults::default(),
            thresholds: Vec::new(),
            steps: vec![SuiteStep::Raw(RawStep {
                name: "precheck".into(),
                tags: Vec::new(),
                args: vec![
                    "dev".into(),
                    "sql".into(),
                    "--log-output".into(),
                    "${artifact_dir}/precheck.log".into(),
                ],
                expected_artifacts: vec!["${artifact_dir}/precheck.log".into()],
            })],
        };

        apply_artifact_dir_templates(&mut config);

        let SuiteStep::Raw(step) = &config.steps[0] else {
            panic!("expected raw step");
        };
        assert_eq!(step.args[3], "artifacts/current/precheck.log");
        assert_eq!(
            step.expected_artifacts,
            vec![PathBuf::from("artifacts/current/precheck.log")]
        );
    }

    #[test]
    fn artifact_dir_templates_rewrite_load_step_paths() {
        let mut config = SuiteConfig {
            name: "current".into(),
            schema_version: 1,
            artifact_dir: Some("artifacts/current".into()),
            defaults: SuiteDefaults::default(),
            thresholds: Vec::new(),
            steps: vec![SuiteStep::Load(LoadStep {
                name: "load".into(),
                tags: Vec::new(),
                pgoptions: None,
                capture_parallel_workers: false,
                prefix: "surface".into(),
                corpus_file: None,
                queries_file: None,
                manifest_file: None,
                allow_manifest_mismatch: false,
                chunked: false,
                dim: None,
                profile: Some("ec_ivf".into()),
                bits: None,
                seed: None,
                m: Vec::new(),
                ef_construction: None,
                storage_format: None,
                index_name: None,
                table_reloptions: Vec::new(),
                reloptions: Vec::new(),
                log_file: Some("${artifact_dir}/load.log".into()),
            })],
        };

        apply_artifact_dir_templates(&mut config);

        let SuiteStep::Load(step) = &config.steps[0] else {
            panic!("expected load step");
        };
        assert_eq!(
            step.log_file,
            Some(PathBuf::from("artifacts/current/load.log"))
        );
    }

    #[test]
    fn spire_local_multinode_step_expands_local_four_instance_lane() {
        let raw = r#"{
          "name": "local-multinode",
          "schema_version": 1,
          "artifact_dir": "artifacts/task121",
          "defaults": {"pg": 18},
          "steps": [{
            "kind": "spire-local-multinode",
            "name": "local-gate",
            "tags": ["task121", "local-multinode"],
            "pgoptions": "-c ec_spire.leaf_block_rows=64",
            "run_id": "task121",
            "coord_port": 39800,
            "remote1_port": 39801,
            "remote2_port": 39802,
            "remote3_port": 39803,
            "tier": "correctness",
            "storage_format": "turboquant",
            "coord_index": "task121_coord_idx",
            "remote_index": "task121_remote_idx",
            "reloptions": ["nlists=128", "top_graph_enabled=1"],
            "coord_reloptions": ["training_sample_rows=10000"],
            "remote_reloptions": ["boundary_replica_count=1"],
            "bench_top_k": 6,
            "bench_queries_limit": 1,
            "bench_sweep": "3",
            "bench_query_metric_projection_columns": ["id", "source"],
            "bench_session_gucs": [
              "ec_spire.max_remote_payload_bytes_per_row=16384",
              "ec_spire.pre_materialization_prune=off"
            ],
            "bench_production_read_variants": [
              "name=source-prune-on;projection=id,source;guc=ec_spire.pre_materialization_prune=on",
              "name=id-prune-off;projection=id;guc=ec_spire.pre_materialization_prune=off",
              "name=global-preheap-on;timeline_payload=none;guc=ec_spire.remote_search_global_pre_heap_merge=on"
            ],
            "skip_bench_suite": true,
            "skip_fault_drills": true,
            "skip_install": true
          }]
        }"#;
        let mut config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        apply_default_artifact_logs(&mut config);
        apply_artifact_dir_templates(&mut config);
        validate_config(&config).expect("suite validates");

        let args = SuiteRunOptions {
            config: "suite.json".into(),
            dry_run: true,
            continue_on_error: false,
            only: Vec::new(),
            only_tag: Vec::new(),
            resume_from: None,
            results_output: None,
            artifact_dir: None,
            manifest_output: None,
            allow_debug_backend: false,
        };
        let manifest = build_manifest(&conn(), &args, raw, &config).expect("manifest builds");
        let step = &manifest.steps[0];

        assert_eq!(step.kind, "spire-local-multinode");
        assert_eq!(
            step.pgoptions.as_deref(),
            Some("-c ec_spire.leaf_block_rows=64")
        );
        assert!(step
            .command
            .windows(3)
            .any(|w| w == ["dev", "spire-multicluster", "local-multinode-pg18"]));
        assert!(step
            .command
            .windows(2)
            .any(|w| w == ["--coord-port", "39800"]));
        assert!(step
            .command
            .windows(2)
            .any(|w| w == ["--remote1-port", "39801"]));
        assert!(step
            .command
            .windows(2)
            .any(|w| w == ["--remote2-port", "39802"]));
        assert!(step
            .command
            .windows(2)
            .any(|w| w == ["--remote3-port", "39803"]));
        assert!(step
            .command
            .windows(2)
            .any(|w| w == ["--storage-format", "turboquant"]));
        assert!(step
            .command
            .windows(2)
            .any(|w| w == ["--coord-index", "task121_coord_idx"]));
        assert!(step
            .command
            .windows(2)
            .any(|w| w == ["--remote-index", "task121_remote_idx"]));
        assert!(step
            .command
            .windows(2)
            .any(|w| w == ["--reloption", "nlists=128"]));
        assert!(step
            .command
            .windows(2)
            .any(|w| w == ["--coord-reloption", "training_sample_rows=10000"]));
        assert!(step
            .command
            .windows(2)
            .any(|w| w == ["--remote-reloption", "boundary_replica_count=1"]));
        assert!(step
            .command
            .windows(2)
            .any(|w| w == ["--bench-query-metric-projection-columns", "id,source"]));
        assert!(step.command.windows(2).any(|w| w
            == [
                "--bench-session-guc",
                "ec_spire.max_remote_payload_bytes_per_row=16384"
            ]));
        assert!(step.command.windows(2).any(|w| w
            == [
                "--bench-session-guc",
                "ec_spire.pre_materialization_prune=off"
            ]));
        assert!(step.command.windows(2).any(|w| w == [
            "--bench-production-read-variant",
            "name=source-prune-on;projection=id,source;guc=ec_spire.pre_materialization_prune=on"
        ]));
        assert!(step.command.windows(2).any(|w| w
            == [
                "--bench-production-read-variant",
                "name=id-prune-off;projection=id;guc=ec_spire.pre_materialization_prune=off"
            ]));
        assert!(step.command.windows(2).any(|w| w
            == [
                "--bench-production-read-variant",
                "name=global-preheap-on;timeline_payload=none;guc=ec_spire.remote_search_global_pre_heap_merge=on"
            ]));
        assert!(step.command.contains(&"--skip-bench-suite".into()));
        assert!(step.command.contains(&"--skip-fault-drills".into()));
        assert!(step
            .expected_artifacts
            .iter()
            .any(|path| path.ends_with("local-multinode.log")));
        assert!(
            step.expected_artifacts
                .iter()
                .any(|path| path
                    .ends_with("target/spire-local-multinode-task121/topology.local.json"))
        );
        assert!(!step
            .expected_artifacts
            .iter()
            .any(|path| path.ends_with("bench-suite/results.jsonl")));
    }

    #[test]
    fn spire_local_multinode_step_tracks_bench_artifacts_when_enabled() {
        let raw = r#"{
          "name": "local-multinode",
          "schema_version": 1,
          "artifact_dir": "artifacts/task121",
          "defaults": {"pg": 18},
          "steps": [{
            "kind": "spire-local-multinode",
            "name": "local-gate",
            "run_dir": "target/task121-local-run",
            "coord_port": 39800,
            "remote1_port": 39801,
            "remote2_port": 39802,
            "remote3_port": 39803,
            "tier": "correctness",
            "skip_bench_rowcap": true,
            "skip_fault_drills": true,
            "skip_install": true
          }]
        }"#;
        let mut config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        apply_default_artifact_logs(&mut config);
        apply_artifact_dir_templates(&mut config);
        validate_config(&config).expect("suite validates");

        let args = SuiteRunOptions {
            config: "suite.json".into(),
            dry_run: true,
            continue_on_error: false,
            only: Vec::new(),
            only_tag: Vec::new(),
            resume_from: None,
            results_output: None,
            artifact_dir: None,
            manifest_output: None,
            allow_debug_backend: false,
        };
        let manifest = build_manifest(&conn(), &args, raw, &config).expect("manifest builds");
        let step = &manifest.steps[0];

        assert!(step.command.contains(&"--skip-bench-rowcap".into()));
        assert!(step
            .expected_artifacts
            .iter()
            .any(|path| path.ends_with("target/task121-local-run/topology.local.json")));
        assert!(step
            .expected_artifacts
            .iter()
            .any(|path| path
                .ends_with("artifacts/task121/local-gate/bench-suite/suite-manifest.json")));
        assert!(step
            .expected_artifacts
            .iter()
            .any(|path| path.ends_with("artifacts/task121/local-gate/bench-suite/results.jsonl")));
    }

    #[test]
    fn spire_local_multinode_step_rejects_semicolon_reloptions() {
        let raw = r#"{
          "name": "local-multinode",
          "schema_version": 1,
          "defaults": {"pg": 18},
          "steps": [{
            "kind": "spire-local-multinode",
            "name": "local-gate",
            "coord_port": 39800,
            "remote1_port": 39801,
            "remote2_port": 39802,
            "remote3_port": 39803,
            "reloptions": ["nlists=128;top_graph_enabled=1"]
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        let err = validate_config(&config).expect_err("reloption with semicolon is rejected");

        assert!(err.to_string().contains("must not contain ';'"));
    }

    #[test]
    fn artifact_dir_templates_rewrite_spire_pipeline_paths() {
        let mut config = SuiteConfig {
            name: "current".into(),
            schema_version: 1,
            artifact_dir: Some("artifacts/current".into()),
            defaults: SuiteDefaults::default(),
            thresholds: Vec::new(),
            steps: vec![SuiteStep::SpirePipeline(SpirePipelineStep {
                name: "profile".into(),
                tags: Vec::new(),
                pgoptions: None,
                prefix: "surface".into(),
                index: None,
                queries_limit: None,
                sweep: vec![8, 16],
                rerank_width: None,
                max_candidate_rows: None,
                max_routed_candidate_rows: None,
                adaptive_nprobe: None,
                adaptive_nprobe_score_gap_micros: None,
                include_remote: None,
                require_remote_placements: None,
                include_local_store_overlap: None,
                remote_selected_pids: Vec::new(),
                remote_requested_epoch: None,
                top_k: None,
                consistency_mode: None,
                remote_tuple_transport: None,
                include_cost_snapshot: None,
                cost_routing_dimension_scale: None,
                cost_leaf_dimension_scale: None,
                cost_index_page_scale: None,
                cost_local_store_page_fanout_scale: None,
                cost_storage_scoring_multiplier: None,
                cost_rerank_multiplier: None,
                include_query_metrics: None,
                include_recall: None,
                truth_corpus_file: Some("${artifact_dir}/truth/corpus.tsv".into()),
                truth_cache_file: Some("${artifact_dir}/truth/cache.json".into()),
                leaf_block_rank_output: Some(
                    "${artifact_dir}/profile-leaf-block-rank.jsonl".into(),
                ),
                target_block_rank_output: Some(
                    "${artifact_dir}/profile-target-block-rank.jsonl".into(),
                ),
                target_candidate_rank_output: Some(
                    "${artifact_dir}/profile-target-candidate-rank.jsonl".into(),
                ),
                miss_attribution_output: Some("${artifact_dir}/profile-misses.jsonl".into()),
                leaf_block_rank_local_sequence_offset: None,
                include_production_read_profile: None,
                production_read_only: None,
                production_read_timeline_no_payload: None,
                query_metric_k: None,
                query_metric_projection_columns: Vec::new(),
                session_gucs: Vec::new(),
                task87_candidate_batch_counters: None,
                log_output: Some("${artifact_dir}/profile.log".into()),
                funnel_output: Some("${artifact_dir}/profile-funnel.jsonl".into()),
                stage_containment_output: Some("${artifact_dir}/profile-stage.jsonl".into()),
            })],
        };

        apply_artifact_dir_templates(&mut config);

        let SuiteStep::SpirePipeline(step) = &config.steps[0] else {
            panic!("expected spire-pipeline step");
        };
        assert_eq!(
            step.truth_corpus_file.as_deref(),
            Some(Path::new("artifacts/current/truth/corpus.tsv"))
        );
        assert_eq!(
            step.truth_cache_file.as_deref(),
            Some(Path::new("artifacts/current/truth/cache.json"))
        );
        assert_eq!(
            config.steps[0].expected_artifacts(),
            vec![
                PathBuf::from("artifacts/current/profile.log"),
                PathBuf::from("artifacts/current/profile-funnel.jsonl"),
                PathBuf::from("artifacts/current/profile-stage.jsonl"),
                PathBuf::from("artifacts/current/profile-leaf-block-rank.jsonl"),
                PathBuf::from("artifacts/current/profile-target-block-rank.jsonl"),
                PathBuf::from("artifacts/current/profile-target-candidate-rank.jsonl"),
                PathBuf::from("artifacts/current/profile-misses.jsonl"),
            ]
        );
        let conn = ConnectionOptions {
            database: "postgres".into(),
            host: None,
            port: None,
            user: None,
            password: None,
        };
        let args = config.steps[0]
            .expand(&config.defaults, &conn)
            .expect("spire-pipeline expansion should succeed");
        assert!(args.windows(2).any(|w| w
            == [
                "--target-candidate-rank-output",
                "artifacts/current/profile-target-candidate-rank.jsonl",
            ]));
        assert!(args.windows(2).any(|w| w
            == [
                "--stage-containment-output",
                "artifacts/current/profile-stage.jsonl"
            ]));
    }

    #[test]
    fn parses_legacy_dry_run_alias() {
        let cli = SuiteOnly::try_parse_from([
            "suite",
            "--config",
            "suite.json",
            "--dry-run",
            "--manifest-output",
            "manifest.json",
        ])
        .expect("suite parses");
        assert!(cli.args.command.is_none());
        assert_eq!(cli.args.config, Some(PathBuf::from("suite.json")));
        assert!(cli.args.dry_run);
        assert_eq!(
            cli.args.manifest_output,
            Some(PathBuf::from("manifest.json"))
        );
    }

    #[test]
    fn parses_minimal_suite_config() {
        let cfg: SuiteConfig = serde_json::from_str(
            r#"{
              "name": "smoke",
              "schema_version": 1,
              "steps": [
                {
                  "kind": "recall",
                  "name": "r10",
                  "prefix": "p",
                  "k": 10,
                  "sweep": [48]
                }
              ]
            }"#,
        )
        .unwrap();
        assert_eq!(cfg.name, "smoke");
        assert_eq!(cfg.steps.len(), 1);
        assert_eq!(cfg.steps[0].name(), "r10");
        validate_config(&cfg).unwrap();
    }

    #[test]
    fn rejects_duplicate_step_names() {
        let cfg: SuiteConfig = serde_json::from_str(
            r#"{
              "name": "smoke",
              "schema_version": 1,
              "steps": [
                {"kind": "storage", "name": "same", "prefix": "p"},
                {"kind": "storage", "name": "same", "prefix": "p"}
              ]
            }"#,
        )
        .unwrap();
        assert!(validate_config(&cfg)
            .unwrap_err()
            .to_string()
            .contains("duplicate suite step name"));
    }

    #[test]
    fn rejects_unknown_profile_names() {
        let cfg: SuiteConfig = serde_json::from_str(
            r#"{
              "name": "smoke",
              "schema_version": 1,
              "defaults": {"profile": "missing_am"},
              "steps": [
                {"kind": "storage", "name": "storage", "prefix": "p"}
              ]
            }"#,
        )
        .unwrap();

        assert!(validate_config(&cfg)
            .unwrap_err()
            .to_string()
            .contains("known profiles"));
    }

    #[test]
    fn expands_recall_with_defaults() {
        let defaults = SuiteDefaults {
            profile: Some("ec_ivf".into()),
            queries_limit: Some(100),
            force_index: Some(true),
            ..SuiteDefaults::default()
        };
        let step = RecallStep {
            name: "recall".into(),
            tags: vec!["sweep".into()],
            prefix: "surface".into(),
            k: 10,
            sweep: vec![48, 96],
            rerank_width: Some(500),
            adaptive_nprobe: Some(true),
            adaptive_nprobe_score_gap_micros: Some(1000),
            adaptive_nprobe_score_margin_ratio_bps: Some(2500),
            ivf_scratch_soa_batch_decode: Some(true),
            queries_limit: None,
            profile: None,
            bits: None,
            seed: None,
            force_index: None,
            session_gucs: vec!["ec_ivf.scratch_soa_batch_decode=on".into()],
            truth_cache_file: Some("truth.json".into()),
            truth_cache_dir: None,
            truth_corpus_file: Some("corpus.tsv".into()),
            log_output: Some("recall.log".into()),
            predictions_output: Some("predictions.json".into()),
        };
        let args = expand_recall(&step, &defaults);
        assert!(args.windows(2).any(|w| w == ["--profile", "ec_ivf"]));
        assert!(args.windows(2).any(|w| w == ["--queries-limit", "100"]));
        assert!(args.contains(&"--force-index".into()));
        assert!(args.windows(2).any(|w| w == ["--sweep", "48,96"]));
        assert!(args.contains(&"--adaptive-nprobe".into()));
        assert!(args
            .windows(2)
            .any(|w| w == ["--session-guc", "ec_ivf.scratch_soa_batch_decode=on"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--truth-corpus-file", "corpus.tsv"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--adaptive-nprobe-score-gap-micros", "1000"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--adaptive-nprobe-score-margin-ratio-bps", "2500"]));
        assert!(args.contains(&"--ivf-scratch-soa-batch-decode".into()));
        assert!(args
            .windows(2)
            .any(|w| w == ["--predictions-output", "predictions.json"]));
    }

    #[test]
    fn expands_comparator_with_vchord_engine_and_lists() {
        let defaults = SuiteDefaults {
            queries_limit: Some(200),
            ..SuiteDefaults::default()
        };
        let step = ComparatorStep {
            name: "comparator-vchord".into(),
            tags: vec!["comparator".into()],
            engine: "vchord".into(),
            prefix: "real_100k".into(),
            k: Some(10),
            sweep: vec![1, 4, 16, 64],
            queries_limit: None,
            lists: Some(320),
            m: None,
            ef_construction: None,
            num_neighbors: None,
            build_search_list_size: None,
            max_alpha: None,
            storage_layout: None,
            maintenance_work_mem: Some("4GB".into()),
            rebuild: true,
            log_output: Some("comparator-vchord.log".into()),
        };
        let args = expand_comparator(&step, &defaults);
        assert_eq!(&args[0], "bench");
        assert_eq!(&args[1], "comparator");
        assert!(args.windows(2).any(|w| w == ["--engine", "vchord"]));
        assert!(args.windows(2).any(|w| w == ["--prefix", "real_100k"]));
        assert!(args.windows(2).any(|w| w == ["--sweep", "1,4,16,64"]));
        assert!(args.windows(2).any(|w| w == ["--lists", "320"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--maintenance-work-mem", "4GB"]));
        assert!(args.windows(2).any(|w| w == ["--queries-limit", "200"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--log-output", "comparator-vchord.log"]));
        assert!(args.contains(&"--rebuild".into()));
        // No ecaz side: there must be no --profile in the comparator argv.
        assert!(!args.iter().any(|a| a == "--profile"));
    }

    #[test]
    fn parses_comparator_table_and_summary_rows() {
        let raw = "[comparator] built real_100k_corpus_vchord_idx in 12.34s\n\
                   [comparator] real_100k_corpus_vchord_idx pg_relation_size=4096 bytes\n\
                   ┌────────┬───────┬──────────┬────────┬──────┬──────┬──────┬──────┐\n\
                   │ engine ┆ sweep ┆ recall@k ┆ ndcg@k ┆ p50  ┆ p95  ┆ p99  ┆ mean │\n\
                   ╞════════╪═══════╪══════════╪════════╪══════╪══════╪══════╪══════╡\n\
                   │ vchord[probes=16] ┆ 16 ┆ 0.9000 ┆ 0.8500 ┆ 1.00 ms ┆ 2.00 ms ┆ 3.00 ms ┆ 1.50 ms │\n\
                   └────────┴───────┴──────────┴────────┴──────┴──────┴──────┴──────┘\n";
        let table_rows = parse_comparator_table_rows(raw);
        assert_eq!(table_rows.len(), 1);
        assert_eq!(
            table_rows[0].get("engine").map(String::as_str),
            Some("vchord[probes=16]")
        );
        assert_eq!(
            table_rows[0].get("recall@k").map(String::as_str),
            Some("0.9000")
        );
        let summary = parse_comparator_summary_rows(raw);
        assert!(summary
            .iter()
            .any(|(metric, v)| metric == "comparator_build"
                && v.get("subject").map(String::as_str) == Some("real_100k_corpus_vchord_idx")));
        assert!(summary
            .iter()
            .any(|(metric, v)| metric == "comparator_index_size"
                && v.get("bytes").map(String::as_str) == Some("4096")));
    }

    #[test]
    fn expands_cross_am_with_inputs_and_log_output() {
        let step = CrossAmStep {
            name: "consistency".into(),
            tags: vec!["cross-am".into()],
            inputs: vec![
                "hnsw=target/hnsw-predictions.json".into(),
                "diskann=target/diskann-predictions.json".into(),
            ],
            k: Some(10),
            log_output: Some("target/cross-am.log".into()),
        };

        let args = expand_cross_am(&step);

        assert_eq!(args[..2], ["bench", "cross-am"]);
        assert!(args
            .windows(2)
            .any(|w| w == ["--input", "hnsw=target/hnsw-predictions.json"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--input", "diskann=target/diskann-predictions.json"]));
        assert!(args.windows(2).any(|w| w == ["--k", "10"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--log-output", "target/cross-am.log"]));
    }

    #[test]
    fn validates_cross_am_inputs_are_labeled_paths() {
        let cfg: SuiteConfig = serde_json::from_str(
            r#"{
              "name": "cross-am",
              "schema_version": 1,
              "steps": [
                {
                  "kind": "cross-am",
                  "name": "bad",
                  "inputs": ["hnsw=target/hnsw.json", "target/diskann.json"],
                  "log_output": "target/cross-am.log"
                }
              ]
            }"#,
        )
        .unwrap();

        assert!(validate_config(&cfg)
            .unwrap_err()
            .to_string()
            .contains("must use label=path"));
    }

    #[test]
    fn expands_sidecar_rerank_with_variants() {
        let defaults = SuiteDefaults {
            profile: Some("ec_ivf".into()),
            queries_limit: Some(200),
            force_index: Some(true),
            ..SuiteDefaults::default()
        };
        let step = SidecarRerankStep {
            name: "sidecar".into(),
            tags: vec!["sidecar".into()],
            prefix: "surface".into(),
            profile: None,
            k: Some(10),
            candidate_k: Some(50),
            final_rerank_k: Some(25),
            concurrency: Some(4),
            sweep: vec![64, 128],
            queries_limit: None,
            warmup_queries: Some(25),
            bits: None,
            seed: None,
            variants: vec!["f32".into(), "f16".into(), "rabitq8".into()],
            read_modes: vec!["free".into(), "random-id".into(), "tid-sorted".into()],
            rebuild_sidecar_table: true,
            force_index: None,
            allow_unsafe_index_shape: false,
            log_output: Some("sidecar.log".into()),
        };
        let args = expand_sidecar_rerank(&step, &defaults);
        assert!(args.windows(2).any(|w| w == ["--profile", "ec_ivf"]));
        assert!(args.windows(2).any(|w| w == ["--candidate-k", "50"]));
        assert!(args.windows(2).any(|w| w == ["--final-rerank-k", "25"]));
        assert!(args.windows(2).any(|w| w == ["--concurrency", "4"]));
        assert!(args.windows(2).any(|w| w == ["--sweep", "64,128"]));
        assert!(args.windows(2).any(|w| w == ["--warmup-queries", "25"]));
        assert!(args.windows(2).any(|w| w == ["--variant", "f32"]));
        assert!(args.windows(2).any(|w| w == ["--variant", "f16"]));
        assert!(args.windows(2).any(|w| w == ["--variant", "rabitq8"]));
        assert!(args.windows(2).any(|w| w == ["--read-mode", "free"]));
        assert!(args.windows(2).any(|w| w == ["--read-mode", "random-id"]));
        assert!(args.windows(2).any(|w| w == ["--read-mode", "tid-sorted"]));
        assert!(args.contains(&"--rebuild-sidecar-table".into()));
        assert!(args.contains(&"--force-index".into()));
        assert!(args
            .windows(2)
            .any(|w| w == ["--log-output", "sidecar.log"]));
    }

    #[test]
    fn expands_chunked_load_without_corpus_query_paths() {
        let defaults = SuiteDefaults {
            profile: Some("ec_ivf".into()),
            bits: Some(4),
            seed: Some(42),
            ..SuiteDefaults::default()
        };
        let step = LoadStep {
            name: "load".into(),
            tags: vec!["load".into()],
            pgoptions: None,
            capture_parallel_workers: false,
            prefix: "surface".into(),
            corpus_file: None,
            queries_file: None,
            manifest_file: Some("stage/anchor_manifest.json".into()),
            allow_manifest_mismatch: false,
            chunked: true,
            dim: None,
            profile: Some("ec_ivf".into()),
            bits: None,
            seed: None,
            m: Vec::new(),
            ef_construction: None,
            storage_format: Some("rabitq".into()),
            index_name: Some("surface_rabitq_idx".into()),
            table_reloptions: Vec::new(),
            reloptions: vec!["nlists=1024".into()],
            log_file: Some("load.log".into()),
        };
        let args = expand_load(&step, &defaults);
        assert!(args.contains(&"--chunked".into()));
        assert!(args.windows(2).any(|w| w == ["--storage-format", "rabitq"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--index-name", "surface_rabitq_idx"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--manifest-file", "stage/anchor_manifest.json"]));
        assert!(!args.iter().any(|arg| arg == "--corpus-file"));
        assert!(!args.iter().any(|arg| arg == "--queries-file"));
    }

    #[test]
    fn load_step_pgoptions_flow_into_manifest_record() {
        let config = SuiteConfig {
            name: "pgoptions-load".into(),
            schema_version: 1,
            artifact_dir: None,
            defaults: SuiteDefaults::default(),
            thresholds: Vec::new(),
            steps: vec![SuiteStep::Load(LoadStep {
                name: "load".into(),
                tags: vec!["load".into()],
                pgoptions: Some("-c max_parallel_maintenance_workers=4".into()),
                capture_parallel_workers: true,
                prefix: "surface".into(),
                corpus_file: None,
                queries_file: None,
                manifest_file: None,
                allow_manifest_mismatch: false,
                chunked: false,
                dim: None,
                profile: Some("ec_ivf".into()),
                bits: None,
                seed: None,
                m: Vec::new(),
                ef_construction: None,
                storage_format: None,
                index_name: None,
                table_reloptions: vec!["parallel_workers=4".into()],
                reloptions: vec!["nlists=128".into()],
                log_file: Some("load.log".into()),
            })],
        };
        let args = SuiteRunOptions {
            config: "suite.json".into(),
            dry_run: true,
            continue_on_error: false,
            only: Vec::new(),
            only_tag: Vec::new(),
            resume_from: None,
            results_output: None,
            artifact_dir: None,
            manifest_output: None,
            allow_debug_backend: false,
        };

        let manifest =
            build_manifest(&conn(), &args, "{}", &config).expect("manifest should build");

        assert_eq!(
            manifest.steps[0].pgoptions.as_deref(),
            Some("-c max_parallel_maintenance_workers=4")
        );
        assert!(manifest.steps[0]
            .command
            .windows(2)
            .any(|w| w == ["--reloption", "nlists=128"]));
        assert!(!manifest.steps[0]
            .command
            .windows(2)
            .any(|w| w == ["--reloption", "parallel_workers=4"]));
        assert!(manifest.steps[0]
            .command
            .windows(2)
            .any(|w| w == ["--table-reloption", "parallel_workers=4"]));
        assert!(matches!(
            &config.steps[0],
            SuiteStep::Load(step) if step.capture_parallel_workers
        ));
    }

    #[test]
    fn quant_axis_tags_flow_into_manifest_and_missing_kernel_marker() {
        let config = SuiteConfig {
            name: "kernel-axis".into(),
            schema_version: 1,
            artifact_dir: None,
            defaults: SuiteDefaults::default(),
            thresholds: Vec::new(),
            steps: vec![
                SuiteStep::Raw(RawStep {
                    name: "lut32-populated".into(),
                    tags: vec![
                        "quant=turboquant".into(),
                        "isa=scalar".into(),
                        "kernel_status=valid".into(),
                    ],
                    args: vec![
                        "bench".into(),
                        "latency".into(),
                        "--prefix".into(),
                        "p".into(),
                    ],
                    expected_artifacts: Vec::new(),
                }),
                SuiteStep::Raw(RawStep {
                    name: "rabitq-sve2-missing".into(),
                    tags: vec![
                        "quant=rabitq".into(),
                        "isa=sve2".into(),
                        "kernel_status=missing_kernel".into(),
                    ],
                    args: vec![
                        "bench".into(),
                        "latency".into(),
                        "--prefix".into(),
                        "p".into(),
                    ],
                    expected_artifacts: Vec::new(),
                }),
            ],
        };
        let args = SuiteRunOptions {
            config: "suite.json".into(),
            dry_run: true,
            continue_on_error: false,
            only: Vec::new(),
            only_tag: Vec::new(),
            resume_from: None,
            results_output: None,
            artifact_dir: None,
            manifest_output: None,
            allow_debug_backend: false,
        };

        let manifest =
            build_manifest(&conn(), &args, "{}", &config).expect("manifest should build");

        let valid = &manifest.steps[0];
        assert_eq!(valid.quant.as_deref(), Some("turboquant"));
        assert_eq!(valid.isa.as_deref(), Some("scalar"));
        assert_eq!(valid.kernel_status, Some(KernelCellStatus::Valid));
        assert!(matches!(valid.status, Some(StepStatus::DryRun)));
        assert!(!valid.command.is_empty());

        let missing = &manifest.steps[1];
        assert_eq!(missing.quant.as_deref(), Some("rabitq"));
        assert_eq!(missing.isa.as_deref(), Some("sve2"));
        assert_eq!(missing.kernel_status, Some(KernelCellStatus::MissingKernel));
        assert!(matches!(missing.status, Some(StepStatus::Skipped)));
        assert!(missing.command.is_empty());

        let row = kernel_cell_result_row(&manifest, missing).expect("marker row");
        assert_eq!(row.metric, "kernel_cell");
        assert_eq!(row.artifact, "suite-manifest");
        assert_eq!(
            row.values.get("kernel_status").map(String::as_str),
            Some("missing_kernel")
        );
        assert_eq!(row.values.get("quant").map(String::as_str), Some("rabitq"));
        assert_eq!(row.values.get("isa").map(String::as_str), Some("sve2"));
    }

    #[test]
    fn retired_kernel_cells_execute_and_emit_marker_row() {
        let config = SuiteConfig {
            name: "kernel-axis-retired".into(),
            schema_version: 1,
            artifact_dir: None,
            defaults: SuiteDefaults::default(),
            thresholds: Vec::new(),
            steps: vec![SuiteStep::Raw(RawStep {
                name: "tiled_lut-retired-confirmation".into(),
                tags: vec![
                    "quant=turboquant".into(),
                    "isa=neon".into(),
                    "kernel_status=retired".into(),
                ],
                args: vec![
                    "bench".into(),
                    "latency".into(),
                    "--prefix".into(),
                    "p".into(),
                ],
                expected_artifacts: Vec::new(),
            })],
        };
        let args = SuiteRunOptions {
            config: "suite.json".into(),
            dry_run: true,
            continue_on_error: false,
            only: Vec::new(),
            only_tag: Vec::new(),
            resume_from: None,
            results_output: None,
            artifact_dir: None,
            manifest_output: None,
            allow_debug_backend: false,
        };

        let manifest =
            build_manifest(&conn(), &args, "{}", &config).expect("manifest should build");

        let retired = &manifest.steps[0];
        assert_eq!(retired.kernel_status, Some(KernelCellStatus::Retired));
        assert!(matches!(retired.status, Some(StepStatus::DryRun)));
        assert!(!retired.command.is_empty());

        let row = kernel_cell_result_row(&manifest, retired).expect("marker row");
        assert_eq!(
            row.values.get("kernel_status").map(String::as_str),
            Some("retired")
        );
    }

    #[test]
    fn quant_axis_rejects_unknown_kernel_status_marker() {
        let config = SuiteConfig {
            name: "kernel-axis".into(),
            schema_version: 1,
            artifact_dir: None,
            defaults: SuiteDefaults::default(),
            thresholds: Vec::new(),
            steps: vec![SuiteStep::Raw(RawStep {
                name: "bad-status".into(),
                tags: vec!["kernel_status=not_real".into()],
                args: vec!["bench".into(), "latency".into()],
                expected_artifacts: Vec::new(),
            })],
        };
        let args = SuiteRunOptions {
            config: "suite.json".into(),
            dry_run: true,
            continue_on_error: false,
            only: Vec::new(),
            only_tag: Vec::new(),
            resume_from: None,
            results_output: None,
            artifact_dir: None,
            manifest_output: None,
            allow_debug_backend: false,
        };

        assert!(build_manifest(&conn(), &args, "{}", &config)
            .unwrap_err()
            .to_string()
            .contains("kernel_status tag"));
    }

    #[test]
    fn parses_task92_quant_axis_smoke_config() {
        let raw = include_str!("../../../suites/task92-quant-axis-smoke.json");
        let mut config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        apply_default_artifact_logs(&mut config);
        apply_artifact_dir_templates(&mut config);
        validate_config(&config).expect("suite validates");

        let args = SuiteRunOptions {
            config: "task92-quant-axis-smoke.json".into(),
            dry_run: true,
            continue_on_error: false,
            only: Vec::new(),
            only_tag: Vec::new(),
            resume_from: None,
            results_output: None,
            artifact_dir: None,
            manifest_output: None,
            allow_debug_backend: false,
        };
        let manifest = build_manifest(&conn(), &args, raw, &config).expect("manifest should build");

        let populated = &manifest.steps[0];
        assert_eq!(
            populated.name,
            "latency-spire-turboquant-lut32-scalar-populated"
        );
        assert_eq!(populated.quant.as_deref(), Some("turboquant"));
        assert_eq!(populated.isa.as_deref(), Some("scalar"));
        assert_eq!(populated.kernel_status, Some(KernelCellStatus::Valid));
        assert!(matches!(populated.status, Some(StepStatus::DryRun)));
        assert!(!populated.command.is_empty());

        let missing = &manifest.steps[1];
        assert_eq!(missing.name, "latency-spire-rabitq-sve2-missing");
        assert_eq!(missing.quant.as_deref(), Some("rabitq"));
        assert_eq!(missing.isa.as_deref(), Some("sve2"));
        assert_eq!(missing.kernel_status, Some(KernelCellStatus::MissingKernel));
        assert!(matches!(missing.status, Some(StepStatus::Skipped)));
        assert!(missing.command.is_empty());
        assert!(kernel_cell_result_row(&manifest, missing).is_some());
    }

    #[test]
    fn parses_task92_offpath_calibration_config() {
        let raw = include_str!("../../../suites/task92-offpath-calibration.json");
        let mut config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        apply_default_artifact_logs(&mut config);
        apply_artifact_dir_templates(&mut config);
        validate_config(&config).expect("suite validates");

        let args = SuiteRunOptions {
            config: "task92-offpath-calibration.json".into(),
            dry_run: true,
            continue_on_error: false,
            only: Vec::new(),
            only_tag: Vec::new(),
            resume_from: None,
            results_output: None,
            artifact_dir: None,
            manifest_output: None,
            allow_debug_backend: false,
        };
        let manifest = build_manifest(&conn(), &args, raw, &config).expect("manifest should build");

        assert_eq!(manifest.steps.len(), 2);
        let kernel_on = &manifest.steps[0];
        assert_eq!(kernel_on.name, "latency-spire-turboquant-lut32-kernel-on");
        assert_eq!(kernel_on.quant.as_deref(), Some("turboquant"));
        assert_eq!(kernel_on.isa.as_deref(), Some("scalar"));
        assert_eq!(kernel_on.kernel_status, Some(KernelCellStatus::Valid));
        assert!(matches!(kernel_on.status, Some(StepStatus::DryRun)));
        assert!(kernel_on
            .command
            .windows(2)
            .any(|w| w == ["--cache-state", "task92_offpath_kernel_on"]));

        let kernel_off = &manifest.steps[1];
        assert_eq!(kernel_off.name, "latency-spire-turboquant-lut32-kernel-off");
        assert_eq!(kernel_off.quant.as_deref(), Some("turboquant"));
        assert_eq!(kernel_off.isa.as_deref(), Some("scalar"));
        assert_eq!(kernel_off.kernel_status, Some(KernelCellStatus::Valid));
        assert!(matches!(kernel_off.status, Some(StepStatus::DryRun)));
        assert!(kernel_off
            .command
            .windows(2)
            .any(|w| { w == ["--session-guc", "ec_spire.candidate_batch_scoring=off",] }));
        assert!(kernel_off
            .command
            .contains(&"--task87-candidate-batch-counters".to_owned()));
    }

    #[test]
    fn parses_parallel_workers_from_loader_timing_artifact() {
        let raw = "[loader] copied corpus table task71_real10k_w4 in 0.123s\n\
                   [loader] ec_ivf build timing: requested_workers=4 workers_launched=4 heap_tuples=10000 index_tuples=10000\n";
        assert_eq!(parse_parallel_workers_from_load_artifact(raw), Some(4));
        let diskann_raw = "[loader] ec_diskann_ambuild_timing index=task65b_w4_idx phase=complete heap_tuples=10000 scanned_tuples=10000 unique_tuples=10000 data_pages=610 heap_scan_ms=2 source_ref_ms=0 training_ms=1 sidecar_setup_ms=0 payload_derivation_ms=10 build_persist_ms=1234 core_medoid_ms=5 core_graph_ms=1100 core_persist_ms=20 parallel_requested_workers=4 parallel_effective_workers=4 parallel_batch_size=16 parallel_flush_rate=0 parallel_rayon_scaffold=true parallel_epochs=625 parallel_proposal_ms=900 parallel_reducer_ms=200 parallel_same_epoch_candidate_reads=12 parallel_total_candidate_reads=5000 overflow_ms=0 codebook_ms=0 write_pages_ms=4 metadata_ms=1 flush_total_ms=5 total_ms=1400\n";
        assert_eq!(
            parse_parallel_workers_from_load_artifact(diskann_raw),
            Some(4)
        );
        assert_eq!(
            parse_parallel_workers_from_load_artifact("[loader] copied corpus table x in 0.123s\n"),
            None
        );
    }

    #[test]
    fn parallel_worker_counter_emits_result_row() {
        let manifest = SuiteManifest {
            suite: "suite".into(),
            schema_version: 1,
            config: "suite.json".into(),
            config_sha256: "abc".into(),
            dry_run: false,
            generated_at_unix_ms: 0,
            connection: ManifestConnection {
                database: "tqvector_bench".into(),
                host: Some("/tmp/pg".into()),
                port: Some(28818),
                user: None,
                password_configured: false,
            },
            backend: None,
            steps: vec![StepRecord {
                name: "load-real10k-w4".into(),
                kind: "load".into(),
                command: vec![
                    "--database".into(),
                    "tqvector_bench".into(),
                    "corpus".into(),
                    "load".into(),
                    "--prefix".into(),
                    "task71_real10k_w4".into(),
                ],
                selected: true,
                quant: None,
                isa: None,
                kernel_status: None,
                pgoptions: Some(
                    "-c max_parallel_maintenance_workers=4 -c max_parallel_workers=4".into(),
                ),
                tags: vec!["real10k".into(), "workers4".into()],
                expected_artifacts: Vec::new(),
                status: Some(StepStatus::Succeeded),
                started_at_unix_ms: Some(1),
                finished_at_unix_ms: Some(2),
                duration_ms: Some(1),
                exit_code: Some(0),
                parallel_workers_before: Some(10),
                parallel_workers_after: Some(14),
                parallel_workers_delta: Some(4),
            }],
            threshold_results: Vec::new(),
        };

        let row = parallel_worker_result_row(&manifest, &manifest.steps[0])
            .expect("counter row should be emitted");

        assert_eq!(row.metric, "parallel_workers");
        assert_eq!(row.artifact, "suite-manifest");
        assert_eq!(row.values.get("before").map(String::as_str), Some("10"));
        assert_eq!(row.values.get("after").map(String::as_str), Some("14"));
        assert_eq!(row.values.get("delta").map(String::as_str), Some("4"));
        assert_eq!(
            row.values.get("prefix").map(String::as_str),
            Some("task71_real10k_w4")
        );
    }

    #[test]
    fn load_step_exposes_pgoptions_for_manifest_and_spawn() {
        let step = SuiteStep::Load(LoadStep {
            name: "load".into(),
            tags: vec!["load".into()],
            pgoptions: Some("-c ec_spire.leaf_block_rows=16".into()),
            capture_parallel_workers: false,
            prefix: "surface".into(),
            corpus_file: None,
            queries_file: None,
            manifest_file: Some("stage/anchor_manifest.json".into()),
            allow_manifest_mismatch: false,
            chunked: true,
            dim: None,
            profile: Some("ec_spire".into()),
            bits: None,
            seed: None,
            m: Vec::new(),
            ef_construction: None,
            storage_format: Some("rabitq".into()),
            index_name: None,
            table_reloptions: Vec::new(),
            reloptions: Vec::new(),
            log_file: Some("load.log".into()),
        });

        assert_eq!(step.pgoptions(), Some("-c ec_spire.leaf_block_rows=16"));
    }

    #[test]
    fn expands_latency_with_cache_state_label() {
        let step = LatencyStep {
            name: "latency".into(),
            tags: vec!["latency".into()],
            pgoptions: None,
            prefix: "surface".into(),
            sweep: vec![64, 128],
            k: None,
            concurrency: None,
            iterations: Some(10),
            rerank_width: None,
            adaptive_nprobe: None,
            adaptive_nprobe_score_gap_micros: None,
            adaptive_nprobe_score_margin_ratio_bps: None,
            ivf_scratch_soa_batch_decode: None,
            profile: Some("ec_diskann".into()),
            bits: None,
            seed: None,
            force_index: None,
            sample_backend_memory: None,
            cache_state: Some("post_recall_warm".into()),
            session_gucs: vec!["ec_diskann.scan_profile_notice=on".into()],
            task87_candidate_batch_counters: Some(true),
            memory_sample_interval_ms: None,
            log_output: Some("latency.log".into()),
        };
        let args = expand_latency(&step, &SuiteDefaults::default());
        assert!(args
            .windows(2)
            .any(|w| w == ["--cache-state", "post_recall_warm"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--session-guc", "ec_diskann.scan_profile_notice=on"]));
        assert!(args.contains(&"--task87-candidate-batch-counters".into()));
    }

    #[test]
    fn expands_spire_pipeline_with_production_profile() {
        let defaults = SuiteDefaults {
            queries_limit: Some(5),
            ..SuiteDefaults::default()
        };
        let step = SpirePipelineStep {
            name: "spire-profile".into(),
            tags: vec!["spire".into(), "profile".into()],
            pgoptions: None,
            prefix: "aws_spire".into(),
            index: Some("aws_spire_idx".into()),
            queries_limit: None,
            sweep: vec![3, 6],
            rerank_width: Some(0),
            max_candidate_rows: Some(1000),
            max_routed_candidate_rows: Some(26_000),
            adaptive_nprobe: Some(true),
            adaptive_nprobe_score_gap_micros: Some(500),
            include_remote: Some(true),
            require_remote_placements: Some(true),
            include_local_store_overlap: Some(false),
            remote_selected_pids: vec![10, 11],
            remote_requested_epoch: Some(1),
            top_k: Some(10),
            consistency_mode: Some("strict".into()),
            remote_tuple_transport: Some("pg_binary_attr_v1".into()),
            include_cost_snapshot: Some(true),
            cost_routing_dimension_scale: Some(0.02),
            cost_leaf_dimension_scale: None,
            cost_index_page_scale: None,
            cost_local_store_page_fanout_scale: None,
            cost_storage_scoring_multiplier: None,
            cost_rerank_multiplier: None,
            include_query_metrics: Some(true),
            include_recall: Some(true),
            truth_corpus_file: Some("truth-corpus.tsv".into()),
            truth_cache_file: Some("truth-cache.json".into()),
            leaf_block_rank_output: None,
            target_block_rank_output: Some("target-block-rank.jsonl".into()),
            target_candidate_rank_output: Some("target-candidate-rank.jsonl".into()),
            miss_attribution_output: Some("miss-attribution.jsonl".into()),
            leaf_block_rank_local_sequence_offset: None,
            include_production_read_profile: Some(true),
            production_read_only: Some(true),
            production_read_timeline_no_payload: Some(true),
            query_metric_k: Some(10),
            query_metric_projection_columns: vec!["title".into()],
            session_gucs: vec!["ec_spire.candidate_batch_scoring=off".into()],
            task87_candidate_batch_counters: Some(true),
            log_output: Some("spire-profile.log".into()),
            funnel_output: None,
            stage_containment_output: Some("stage-containment.jsonl".into()),
        };

        let args = expand_spire_pipeline(&step, &defaults);
        assert_eq!(args[..2], ["bench", "spire-pipeline"]);
        assert!(args.windows(2).any(|w| w == ["--prefix", "aws_spire"]));
        assert!(args.windows(2).any(|w| w == ["--index", "aws_spire_idx"]));
        assert!(args.windows(2).any(|w| w == ["--queries-limit", "5"]));
        assert!(args.windows(2).any(|w| w == ["--sweep", "3,6"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--max-routed-candidate-rows", "26000"]));
        assert!(args.contains(&"--include-remote".into()));
        assert!(args.contains(&"--require-remote-placements".into()));
        assert!(args.contains(&"--include-production-read-profile".into()));
        assert!(args.contains(&"--production-read-only".into()));
        assert!(args.contains(&"--production-read-timeline-no-payload".into()));
        assert!(args
            .windows(2)
            .any(|w| w == ["--remote-selected-pids", "10,11"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--remote-tuple-transport", "pg_binary_attr_v1"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--query-metric-projection-columns", "title"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--session-guc", "ec_spire.candidate_batch_scoring=off"]));
        assert!(args.contains(&"--task87-candidate-batch-counters".into()));
        assert!(args
            .windows(2)
            .any(|w| w == ["--truth-corpus-file", "truth-corpus.tsv"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--truth-cache-file", "truth-cache.json"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--target-block-rank-output", "target-block-rank.jsonl"]));
        assert!(args.windows(2).any(|w| w
            == [
                "--target-candidate-rank-output",
                "target-candidate-rank.jsonl"
            ]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--miss-attribution-output", "miss-attribution.jsonl"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--stage-containment-output", "stage-containment.jsonl"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["--log-output", "spire-profile.log"]));
    }

    #[test]
    fn artifact_dir_supplies_default_spire_pipeline_log_output() {
        let mut config = SuiteConfig {
            name: "aws-suite".into(),
            schema_version: 1,
            artifact_dir: Some("artifacts/aws".into()),
            defaults: SuiteDefaults::default(),
            thresholds: Vec::new(),
            steps: vec![SuiteStep::SpirePipeline(SpirePipelineStep {
                name: "profile k10".into(),
                tags: Vec::new(),
                pgoptions: None,
                prefix: "aws_spire".into(),
                index: None,
                queries_limit: Some(10),
                sweep: vec![8, 16],
                rerank_width: None,
                max_candidate_rows: None,
                max_routed_candidate_rows: None,
                adaptive_nprobe: None,
                adaptive_nprobe_score_gap_micros: None,
                include_remote: Some(true),
                require_remote_placements: Some(true),
                include_local_store_overlap: None,
                remote_selected_pids: Vec::new(),
                remote_requested_epoch: None,
                top_k: Some(10),
                consistency_mode: None,
                remote_tuple_transport: None,
                include_cost_snapshot: None,
                cost_routing_dimension_scale: None,
                cost_leaf_dimension_scale: None,
                cost_index_page_scale: None,
                cost_local_store_page_fanout_scale: None,
                cost_storage_scoring_multiplier: None,
                cost_rerank_multiplier: None,
                include_query_metrics: Some(true),
                include_recall: Some(true),
                truth_corpus_file: None,
                truth_cache_file: None,
                leaf_block_rank_output: None,
                target_block_rank_output: None,
                target_candidate_rank_output: None,
                miss_attribution_output: None,
                leaf_block_rank_local_sequence_offset: None,
                include_production_read_profile: Some(true),
                production_read_only: Some(true),
                production_read_timeline_no_payload: None,
                query_metric_k: Some(10),
                query_metric_projection_columns: Vec::new(),
                session_gucs: Vec::new(),
                task87_candidate_batch_counters: None,
                log_output: None,
                funnel_output: None,
                stage_containment_output: None,
            })],
        };

        apply_default_artifact_logs(&mut config);
        let SuiteStep::SpirePipeline(step) = &config.steps[0] else {
            panic!("expected spire-pipeline step");
        };
        assert_eq!(
            step.log_output.as_deref(),
            Some(Path::new("artifacts/aws/profile_k10.log"))
        );
        assert_eq!(
            config.steps[0].expected_artifacts(),
            vec![PathBuf::from("artifacts/aws/profile_k10.log")]
        );
        let conn = ConnectionOptions {
            database: "postgres".into(),
            host: None,
            port: None,
            user: None,
            password: None,
        };
        let args = config.steps[0]
            .expand(&config.defaults, &conn)
            .expect("spire-pipeline expansion should succeed");
        assert!(args
            .windows(2)
            .any(|w| { w == ["--log-output", "artifacts/aws/profile_k10.log",] }));
    }

    #[test]
    fn spire_pooling_gate_reports_not_justified_and_candidates() {
        let rows = vec![
            ResultRow {
                suite: "suite".into(),
                step: "profile".into(),
                kind: "spire-pipeline".into(),
                metric: "spire-pipeline".into(),
                artifact: "profile.log".into(),
                values: BTreeMap::from([
                    ("nprobe".into(), "8".into()),
                    ("latency_p95".into(), "10.000 ms".into()),
                ]),
            },
            ResultRow {
                suite: "suite".into(),
                step: "profile".into(),
                kind: "spire-pipeline".into(),
                metric: "spire-pipeline".into(),
                artifact: "profile.log".into(),
                values: BTreeMap::from([
                    ("nprobe".into(), "8".into()),
                    ("connect_p50".into(), "0.500 ms".into()),
                    ("connect_p95".into(), "1.000 ms".into()),
                ]),
            },
            ResultRow {
                suite: "suite".into(),
                step: "profile".into(),
                kind: "spire-pipeline".into(),
                metric: "spire-pipeline".into(),
                artifact: "profile.log".into(),
                values: BTreeMap::from([
                    ("nprobe".into(), "16".into()),
                    ("latency_p95".into(), "10.000 ms".into()),
                ]),
            },
            ResultRow {
                suite: "suite".into(),
                step: "profile".into(),
                kind: "spire-pipeline".into(),
                metric: "spire-pipeline".into(),
                artifact: "profile.log".into(),
                values: BTreeMap::from([
                    ("nprobe".into(), "16".into()),
                    ("connect_p50".into(), "1.200 ms".into()),
                    ("connect_p95".into(), "1.600 ms".into()),
                ]),
            },
        ];

        let gate_rows = spire_pooling_gate_rows(&rows);
        assert_eq!(gate_rows.len(), 2);
        assert_eq!(gate_rows[0].decision(), "pooling_not_justified");
        assert_eq!(gate_rows[1].decision(), "pooling_candidate");
        let report =
            render_spire_pooling_gate_section(&rows).expect("pooling gate section should render");
        assert!(report.contains("connect_p95/read_p95"));
        assert!(report.contains("pooling_not_justified"));
        assert!(report.contains("pooling_candidate"));
    }

    #[test]
    fn parses_fetch_prepare_suite_config() {
        let cfg: SuiteConfig = serde_json::from_str(
            r#"{
              "name": "scale",
              "schema_version": 1,
              "steps": [
                {
                  "kind": "corpus-fetch",
                  "name": "fetch",
                  "dataset": "dbpedia-openai3-large-1536-1m",
                  "output_dir": "data/fetch"
                },
                {
                  "kind": "corpus-prepare",
                  "name": "prepare",
                  "profile": "ec_real_ann_benchmarks_anchor",
                  "parquet": "data/fetch/data",
                  "output_dir": "data/staged",
                  "chunk_rows": 25000
                },
                {
                  "kind": "load",
                  "name": "load",
                  "prefix": "profile_real1m",
                  "manifest_file": "data/staged/ec_real_ann_benchmarks_anchor_manifest.json",
                  "chunked": true
                }
              ]
            }"#,
        )
        .unwrap();
        validate_config(&cfg).unwrap();
        assert_eq!(cfg.steps[0].kind(), "corpus-fetch");
        assert_eq!(cfg.steps[1].kind(), "corpus-prepare");
        assert_eq!(cfg.steps[2].kind(), "load");
    }

    #[test]
    fn step_selection_requires_name_and_tag_matches() {
        let step = SuiteStep::Recall(RecallStep {
            name: "recall".into(),
            tags: vec!["recall".into(), "sweep".into()],
            prefix: "surface".into(),
            k: 10,
            sweep: vec![48],
            rerank_width: None,
            adaptive_nprobe: None,
            adaptive_nprobe_score_gap_micros: None,
            adaptive_nprobe_score_margin_ratio_bps: None,
            ivf_scratch_soa_batch_decode: None,
            queries_limit: None,
            profile: None,
            bits: None,
            seed: None,
            force_index: None,
            session_gucs: Vec::new(),
            truth_cache_file: None,
            truth_cache_dir: None,
            truth_corpus_file: None,
            log_output: None,
            predictions_output: None,
        });
        let args = SuiteRunOptions {
            config: "suite.json".into(),
            dry_run: true,
            continue_on_error: false,
            only: vec!["recall".into()],
            only_tag: vec!["sweep".into()],
            resume_from: None,
            results_output: None,
            artifact_dir: None,
            manifest_output: None,
            allow_debug_backend: false,
        };
        assert!(step_selected(&step, &args));

        let args = SuiteRunOptions {
            only_tag: vec!["latency".into()],
            ..args
        };
        assert!(!step_selected(&step, &args));
    }

    #[test]
    fn parses_recall_result_table() {
        let rows = parse_table_rows(
            "┌────────┬─────────┬───────────────┬──────────┬─────────────────┬──────────────────┬────────────┬────────────┬────────────┬────────┬─────────────┐\n\
             │ nprobe ┆ queries ┆ recall_trials ┆ recall@k ┆ recall_ci95_low ┆ recall_ci95_high ┆ recall_p10 ┆ recall_p50 ┆ recall_p90 ┆ ndcg@k ┆ mean q-time │\n\
             ╞════════╪═════════╪═══════════════╪══════════╪═════════════════╪══════════════════╪════════════╪════════════╪════════════╪════════╪═════════════╡\n\
             │ 96     ┆ 50      ┆ 500           ┆ 0.9980   ┆ 0.9889          ┆ 0.9996           ┆ 1.0000     ┆ 1.0000     ┆ 1.0000     ┆ 0.9997 ┆ 11.00 ms    │\n\
             └────────┴─────────┴───────────────┴──────────┴─────────────────┴──────────────────┴────────────┴────────────┴────────────┴────────┴─────────────┘\n",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("nprobe").map(String::as_str), Some("96"));
        assert_eq!(rows[0].get("recall@k").map(String::as_str), Some("0.9980"));
        assert_eq!(
            rows[0].get("recall_ci95_low").map(String::as_str),
            Some("0.9889")
        );
        assert_eq!(
            rows[0].get("recall_p50").map(String::as_str),
            Some("1.0000")
        );
    }

    #[test]
    fn parse_table_rows_resets_headers_between_same_width_tables() {
        let rows = parse_table_rows(
            "First table\n\
             ┌───┬───┐\n\
             │ a ┆ b │\n\
             ╞═══╪═══╡\n\
             │ 1 ┆ 2 │\n\
             └───┴───┘\n\
             \n\
             Second table\n\
             ┌───┬───┐\n\
             │ c ┆ d │\n\
             ╞═══╪═══╡\n\
             │ 3 ┆ 4 │\n\
             └───┴───┘\n",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("a").map(String::as_str), Some("1"));
        assert_eq!(rows[0].get("b").map(String::as_str), Some("2"));
        assert_eq!(rows[1].get("c").map(String::as_str), Some("3"));
        assert_eq!(rows[1].get("d").map(String::as_str), Some("4"));
        assert!(!rows[1].contains_key("a"));
    }

    #[test]
    fn result_rows_include_suite_context_fields() {
        let manifest = SuiteManifest {
            suite: "task60".into(),
            schema_version: 1,
            config: "suite.json".into(),
            config_sha256: "hash".into(),
            dry_run: false,
            generated_at_unix_ms: 0,
            connection: ManifestConnection {
                database: "tqvector_bench".into(),
                host: None,
                port: Some(28818),
                user: None,
                password_configured: false,
            },
            backend: None,
            steps: Vec::new(),
            threshold_results: Vec::new(),
        };
        let step = StepRecord {
            name: "recall-100k-diskann-rabitq".into(),
            kind: "recall".into(),
            command: vec![
                "--database".into(),
                "tqvector_bench".into(),
                "bench".into(),
                "recall".into(),
                "--prefix".into(),
                "task60_real_100k_diskann_rabitq".into(),
                "--profile".into(),
                "ec_diskann".into(),
                "--socket-dir".into(),
                "/var/run/postgresql".into(),
            ],
            selected: true,
            quant: Some("rabitq".into()),
            isa: None,
            kernel_status: None,
            pgoptions: None,
            tags: vec!["recall".into(), "rabitq".into(), "task60".into()],
            expected_artifacts: vec!["recall.log".into()],
            status: Some(StepStatus::Succeeded),
            started_at_unix_ms: None,
            finished_at_unix_ms: None,
            duration_ms: None,
            exit_code: None,
            parallel_workers_before: None,
            parallel_workers_after: None,
            parallel_workers_delta: None,
        };
        let rows = parse_result_rows(
            &manifest,
            &step,
            "recall.log",
            "┌────────┬─────────┬───────────────┬──────────┐\n\
             │ nprobe ┆ queries ┆ recall_trials ┆ recall@k │\n\
             ╞════════╪═════════╪═══════════════╪══════════╡\n\
             │ 800    ┆ 200     ┆ 2000          ┆ 0.9910   │\n\
             └────────┴─────────┴───────────────┴──────────┘\n",
        );

        assert_eq!(rows.len(), 1);
        let values = &rows[0].values;
        assert_eq!(
            values.get("storage_format").map(String::as_str),
            Some("rabitq")
        );
        assert_eq!(
            values.get("prefix").map(String::as_str),
            Some("task60_real_100k_diskann_rabitq")
        );
        assert_eq!(
            values.get("profile").map(String::as_str),
            Some("ec_diskann")
        );
        assert_eq!(
            values.get("suite_database").map(String::as_str),
            Some("tqvector_bench")
        );
        assert_eq!(
            values.get("suite_host").map(String::as_str),
            Some("local_socket")
        );
        assert_eq!(values.get("suite_port").map(String::as_str), Some("28818"));
        assert_eq!(
            values.get("socket_dir").map(String::as_str),
            Some("/var/run/postgresql")
        );
    }

    #[test]
    fn latency_result_rows_include_block_kernel_counter_lines() {
        let manifest = SuiteManifest {
            suite: "task94".into(),
            schema_version: 1,
            config: "suite.json".into(),
            config_sha256: "hash".into(),
            dry_run: false,
            generated_at_unix_ms: 0,
            connection: ManifestConnection {
                database: "tqvector_bench".into(),
                host: None,
                port: Some(28818),
                user: None,
                password_configured: false,
            },
            backend: None,
            steps: Vec::new(),
            threshold_results: Vec::new(),
        };
        let step = StepRecord {
            name: "latency-pqfastscan-grouped-pq".into(),
            kind: "latency".into(),
            command: vec![
                "bench".into(),
                "latency".into(),
                "--prefix".into(),
                "task94_real10k_pqfastscan".into(),
                "--profile".into(),
                "ec_ivf".into(),
                "--socket-dir".into(),
                "/var/run/postgresql".into(),
            ],
            selected: true,
            quant: Some("grouped_pq".into()),
            isa: Some("sve2".into()),
            kernel_status: None,
            pgoptions: None,
            tags: vec![
                "pq_fastscan".into(),
                "quant=grouped_pq".into(),
                "isa=sve2".into(),
            ],
            expected_artifacts: vec!["latency.log".into()],
            status: Some(StepStatus::Succeeded),
            started_at_unix_ms: None,
            finished_at_unix_ms: None,
            duration_ms: None,
            exit_code: None,
            parallel_workers_before: None,
            parallel_workers_after: None,
            parallel_workers_delta: None,
        };
        let rows = parse_result_rows(
            &manifest,
            &step,
            "latency.log",
            "┌────────┬─────────┬────────────┐\n\
             │ nprobe ┆ queries ┆ latency_ms │\n\
             ╞════════╪═════════╪════════════╡\n\
             │ 8      ┆ 20      ┆ 1.2500     │\n\
             └────────┴─────────┴────────────┘\n\
             [block-kernel-counters] command=latency label=nprobe=8 surface=ivf quant=grouped_pq isa=sve2 flushes=2 candidates=39 elapsed_nanos=1500000 elapsed_ms=1.500000 kernel_flushes=1 kernel_candidates=32 kernel_elapsed_nanos=1100000 kernel_elapsed_ms=1.100000 scalar_flushes=1 scalar_candidates=7 scalar_elapsed_nanos=400000 scalar_elapsed_ms=0.400000\n\
             [task87-counters] command=latency label=nprobe=8 surface=ivf flushes=2 candidates=39 elapsed_nanos=1500000 elapsed_ms=1.500000 lut32_flushes=1 lut32_candidates=32\n",
        );

        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|row| row.metric == "latency"));
        let counters = rows
            .iter()
            .find(|row| row.metric == "block_kernel_counters")
            .expect("direct block-kernel counter row should be extracted");
        assert_eq!(counters.kind, "latency");
        assert_eq!(counters.artifact, "latency.log");
        assert_eq!(
            counters.values.get("surface").map(String::as_str),
            Some("ivf")
        );
        assert_eq!(
            counters.values.get("quant").map(String::as_str),
            Some("grouped_pq")
        );
        assert_eq!(counters.values.get("isa").map(String::as_str), Some("sve2"));
        assert_eq!(
            counters.values.get("kernel_candidates").map(String::as_str),
            Some("32")
        );
        assert_eq!(
            counters.values.get("scalar_candidates").map(String::as_str),
            Some("7")
        );
        assert_eq!(
            counters.values.get("profile").map(String::as_str),
            Some("ec_ivf")
        );
        assert_eq!(
            counters.values.get("storage_format").map(String::as_str),
            Some("pq_fastscan")
        );
    }

    #[test]
    fn parses_cross_am_result_table_for_thresholds() {
        let rows = parse_table_rows(
            "┌──────────────┬─────────┬───┬───────────┬───────────────┐\n\
             │ pair         ┆ queries ┆ k ┆ jaccard@k ┆ kendall_tau@k │\n\
             ╞══════════════╪═════════╪═══╪═══════════╪═══════════════╡\n\
             │ hnsw~diskann ┆ 2       ┆ 3 ┆ 0.7500    ┆ -0.3333       │\n\
             └──────────────┴─────────┴───┴───────────┴───────────────┘\n",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get("pair").map(String::as_str),
            Some("hnsw~diskann")
        );

        let threshold = ThresholdConfig {
            name: "jaccard-floor".into(),
            step: "consistency".into(),
            metric: "cross_am".into(),
            filters: BTreeMap::from([("pair".into(), "hnsw~diskann".into())]),
            field: "jaccard@k".into(),
            op: ThresholdOp::Gte,
            value: 0.7,
        };
        let result = evaluate_threshold(
            &threshold,
            &[ResultRow {
                suite: "suite".into(),
                step: "consistency".into(),
                kind: "cross-am".into(),
                metric: "cross_am".into(),
                artifact: "cross-am.log".into(),
                values: rows[0].clone(),
            }],
        );
        assert!(result.passed);
        assert_eq!(result.actual, Some(0.7500));
    }

    #[test]
    fn parses_loader_timing_rows() {
        let rows = parse_load_rows(
            "[loader] copied corpus table p_corpus in 15.02s\n\
             [loader] copied queries table p_queries in 183.48ms\n\
             [loader] completed prefix p in 45.76s\n",
        );
        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows[0].1.get("phase").map(String::as_str),
            Some("copy_corpus")
        );
        assert_eq!(
            rows[1].1.get("seconds").map(String::as_str),
            Some("0.183480")
        );
    }

    #[test]
    fn parses_ec_ivf_build_timing_rows() {
        let rows = parse_load_rows(
            "[loader] ec_ivf build timing: requested_workers=2 workers_launched=2 heap_tuples=10000 index_tuples=10000 heap_ingest_us=100 parallel_begin_us=200 parallel_drain_us=300 parallel_sort_push_us=400 parallel_worker_tuple_buffer_capacity=16384 parallel_worker_tuple_buffer_struct_bytes=1572864\n",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "ec_ivf_build_timing");
        assert_eq!(
            rows[0].1.get("workers_launched").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            rows[0]
                .1
                .get("parallel_worker_tuple_buffer_capacity")
                .map(String::as_str),
            Some("16384")
        );
    }

    #[test]
    fn parses_ec_diskann_build_timing_rows() {
        let rows = parse_load_rows(
            "[loader] ec_diskann_ambuild_timing index=task65b_w4_idx phase=complete heap_tuples=10000 scanned_tuples=10000 unique_tuples=10000 data_pages=610 heap_scan_ms=2 source_ref_ms=0 training_ms=1 sidecar_setup_ms=0 payload_derivation_ms=10 build_persist_ms=1234 core_medoid_ms=5 core_graph_ms=1100 core_persist_ms=20 parallel_requested_workers=4 parallel_effective_workers=4 parallel_batch_size=16 parallel_flush_rate=0 parallel_rayon_scaffold=true parallel_epochs=625 parallel_proposal_ms=900 parallel_reducer_ms=200 parallel_same_epoch_candidate_reads=12 parallel_total_candidate_reads=5000 overflow_ms=0 codebook_ms=0 write_pages_ms=4 metadata_ms=1 flush_total_ms=5 total_ms=1400\n",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "ec_diskann_build_timing");
        assert_eq!(
            rows[0]
                .1
                .get("parallel_effective_workers")
                .map(String::as_str),
            Some("4")
        );
        assert_eq!(
            rows[0].1.get("parallel_batch_size").map(String::as_str),
            Some("16")
        );
        assert_eq!(
            rows[0].1.get("parallel_reducer_ms").map(String::as_str),
            Some("200")
        );
        assert_eq!(
            rows[0].1.get("parallel_rayon_scaffold"),
            None,
            "boolean timing fields are intentionally ignored by numeric result parsing"
        );
    }

    #[test]
    fn parses_storage_rows_with_raw_byte_fields() {
        let rows = parse_storage_rows(
            "┌────────┬──────────┐\n\
             │ field  ┆ value    │\n\
             ╞════════╪══════════╡\n\
             │ total  ┆ 1.5 MiB  │\n\
             └────────┴──────────┘\n\
             ┌────────┬───────────────┬──────────┬────────────┬──────────┬─────────┐\n\
             │ index  ┆ access method ┆ profile  ┆ reloptions ┆ size     ┆ per row │\n\
             ╞════════╪═══════════════╪══════════╪════════════╪══════════╪═════════╡\n\
             │ ix     ┆ ec_diskann    ┆ diskann  ┆ {}         ┆ 13.0 MiB ┆ 494.0 B │\n\
             └────────┴───────────────┴──────────┴────────────┴──────────┴─────────┘\n",
        );

        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].1.get("value_bytes").map(String::as_str),
            Some("1572864")
        );
        assert_eq!(
            rows[1].1.get("size_bytes").map(String::as_str),
            Some("13631488")
        );
        assert_eq!(
            rows[1].1.get("per_row_bytes").map(String::as_str),
            Some("494.0")
        );
    }

    #[test]
    fn parses_explain_planner_cost_rows() {
        let rows = parse_explain_rows(
            "┌──────────────────────┬──────────────────────┬────────────────────┐\n\
             │ planner_scan_enabled ┆ modeled_startup_cost ┆ modeled_total_cost │\n\
             ╞══════════════════════╪══════════════════════╪════════════════════╡\n\
             │ t                    ┆ 12.5                 ┆ 37.25              │\n\
             └──────────────────────┴──────────────────────┴────────────────────┘\n",
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "planner_cost");
        assert_eq!(
            rows[0].1.get("modeled_total_cost").map(String::as_str),
            Some("37.25")
        );
    }

    #[test]
    fn parses_explain_planner_cost_rows_from_psql_aligned_output() {
        let rows = parse_explain_rows(
            " planner_scan_enabled | modeled_startup_cost | modeled_total_cost\n\
             ----------------------+----------------------+--------------------\n\
             t                    | 5.0128               | 22.4224\n\
             (1 row)\n",
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "planner_cost");
        assert_eq!(
            rows[0].1.get("modeled_total_cost").map(String::as_str),
            Some("22.4224")
        );
    }

    #[test]
    fn evaluates_thresholds_against_result_rows() {
        let rows = vec![ResultRow {
            suite: "suite".into(),
            step: "recall".into(),
            kind: "recall".into(),
            metric: "recall".into(),
            artifact: "recall.log".into(),
            values: BTreeMap::from([("recall@k".into(), "0.9980".into())]),
        }];
        let thresholds = vec![ThresholdConfig {
            name: "recall-floor".into(),
            step: "recall".into(),
            metric: "recall".into(),
            filters: BTreeMap::new(),
            field: "recall@k".into(),
            op: ThresholdOp::Gte,
            value: 0.995,
        }];
        let results = evaluate_thresholds(&thresholds, &rows);
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert_eq!(results[0].actual, Some(0.9980));
    }

    #[test]
    fn threshold_filters_select_matching_sweep_row() {
        let rows = vec![
            ResultRow {
                suite: "suite".into(),
                step: "recall".into(),
                kind: "recall".into(),
                metric: "recall".into(),
                artifact: "recall.log".into(),
                values: BTreeMap::from([
                    ("nprobe".into(), "48".into()),
                    ("recall@k".into(), "0.9820".into()),
                ]),
            },
            ResultRow {
                suite: "suite".into(),
                step: "recall".into(),
                kind: "recall".into(),
                metric: "recall".into(),
                artifact: "recall.log".into(),
                values: BTreeMap::from([
                    ("nprobe".into(), "96".into()),
                    ("recall@k".into(), "0.9980".into()),
                ]),
            },
        ];
        let threshold = ThresholdConfig {
            name: "recall-p96-floor".into(),
            step: "recall".into(),
            metric: "recall".into(),
            filters: BTreeMap::from([("nprobe".into(), "96".into())]),
            field: "recall@k".into(),
            op: ThresholdOp::Gte,
            value: 0.995,
        };
        let result = evaluate_threshold(&threshold, &rows);
        assert!(result.passed);
        assert_eq!(result.actual, Some(0.9980));
    }

    #[test]
    fn skips_thresholds_for_unselected_steps() {
        let rows = vec![ResultRow {
            suite: "suite".into(),
            step: "selected".into(),
            kind: "recall".into(),
            metric: "recall".into(),
            artifact: "selected.log".into(),
            values: BTreeMap::from([("recall@k".into(), "0.9980".into())]),
        }];
        let thresholds = vec![
            ThresholdConfig {
                name: "selected-floor".into(),
                step: "selected".into(),
                metric: "recall".into(),
                filters: BTreeMap::new(),
                field: "recall@k".into(),
                op: ThresholdOp::Gte,
                value: 0.995,
            },
            ThresholdConfig {
                name: "unselected-floor".into(),
                step: "unselected".into(),
                metric: "recall".into(),
                filters: BTreeMap::new(),
                field: "recall@k".into(),
                op: ThresholdOp::Gte,
                value: 0.995,
            },
        ];
        let selected_steps = HashSet::from(["selected"]);
        let results = evaluate_thresholds_for_steps(&thresholds, &rows, &selected_steps);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "selected-floor");
        assert!(results[0].passed);
    }

    #[test]
    fn prefixes_child_commands_with_connection_flags() {
        let args = child_command_args(&conn(), vec!["bench".into(), "storage".into()]);
        assert!(args.windows(2).any(|w| w == ["--database", "postgres"]));
        assert!(args.windows(2).any(|w| w == ["--host", "/tmp/pg"]));
        assert!(args.windows(2).any(|w| w == ["--port", "28818"]));
        assert!(!args.contains(&"--password".into()));
        assert!(args.ends_with(&["bench".into(), "storage".into()]));
    }

    #[test]
    fn expands_explain_with_connection_defaults() {
        let defaults = SuiteDefaults::default();
        let step = ExplainStep {
            name: "explain".into(),
            tags: Vec::new(),
            prefix: "pfx".into(),
            profile: None,
            index_name: None,
            query_table: None,
            corpus_table: None,
            nprobe: 96,
            rerank_width: 1000,
            ivf_scratch_soa_batch_decode: None,
            session_gucs: Vec::new(),
            pg: None,
            db: None,
            socket_dir: None,
            port: None,
            sql_file: Some("explain.sql".into()),
            log_output: Some("explain.log".into()),
        };
        let args = expand_explain(&step, &defaults, &conn());
        assert!(args.windows(2).any(|w| w == ["--db", "postgres"]));
        assert!(args.windows(2).any(|w| w == ["--socket-dir", "/tmp/pg"]));
        assert!(args.windows(2).any(|w| w == ["--port", "28818"]));
        assert!(args.windows(2).any(|w| w == ["--file", "explain.sql"]));
    }

    #[test]
    fn explain_sql_uses_suite_fields() {
        let step = ExplainStep {
            name: "explain".into(),
            tags: Vec::new(),
            prefix: "pfx".into(),
            profile: None,
            index_name: None,
            query_table: None,
            corpus_table: None,
            nprobe: 96,
            rerank_width: 1000,
            ivf_scratch_soa_batch_decode: None,
            session_gucs: Vec::new(),
            pg: None,
            db: None,
            socket_dir: None,
            port: None,
            sql_file: Some("explain.sql".into()),
            log_output: Some("explain.log".into()),
        };
        let sql = explain_sql(&step, &SuiteDefaults::default());
        assert!(sql.contains("SET ec_ivf.nprobe = 96;"));
        assert!(sql.contains("SET ec_ivf.rerank_width = 1000;"));
        assert!(sql.contains("FROM ec_ivf_index_cost_snapshot('pfx_idx'::regclass);"));
        assert!(sql.contains("FROM pfx_corpus"));
        assert!(sql.contains("FROM pfx_queries"));
        assert!(sql.contains("'pfx_idx'::regclass"));
    }

    #[test]
    fn explain_sql_can_enable_ivf_scratch_soa() {
        let step = ExplainStep {
            name: "explain".into(),
            tags: Vec::new(),
            prefix: "pfx".into(),
            profile: None,
            index_name: None,
            query_table: None,
            corpus_table: None,
            nprobe: 96,
            rerank_width: 1000,
            ivf_scratch_soa_batch_decode: Some(true),
            session_gucs: Vec::new(),
            pg: None,
            db: None,
            socket_dir: None,
            port: None,
            sql_file: Some("explain.sql".into()),
            log_output: Some("explain.log".into()),
        };
        let sql = explain_sql(&step, &SuiteDefaults::default());

        assert!(sql.contains("SET ec_ivf.scratch_soa_batch_decode = on;"));
        assert!(sql.contains(
            "current_setting('ec_ivf.scratch_soa_batch_decode') AS scratch_soa_batch_decode"
        ));
        assert!(sql.contains("RESET ec_ivf.scratch_soa_batch_decode;"));
    }

    #[test]
    fn explain_sql_applies_session_gucs() {
        let step = ExplainStep {
            name: "explain".into(),
            tags: Vec::new(),
            prefix: "pfx".into(),
            profile: None,
            index_name: None,
            query_table: None,
            corpus_table: None,
            nprobe: 96,
            rerank_width: 1000,
            ivf_scratch_soa_batch_decode: Some(true),
            session_gucs: vec!["ec_ivf.dense_posting_coalescing=off".into()],
            pg: None,
            db: None,
            socket_dir: None,
            port: None,
            sql_file: Some("explain.sql".into()),
            log_output: Some("explain.log".into()),
        };
        let sql = explain_sql(&step, &SuiteDefaults::default());

        assert!(sql.contains("SET ec_ivf.dense_posting_coalescing = off;"));
        assert!(sql.contains("RESET ec_ivf.dense_posting_coalescing;"));
    }

    #[test]
    fn explain_sql_uses_spire_profile_gucs_and_cost_snapshot() {
        let step = ExplainStep {
            name: "explain".into(),
            tags: Vec::new(),
            prefix: "spire_pfx".into(),
            profile: Some("ec_spire".into()),
            index_name: None,
            query_table: None,
            corpus_table: None,
            nprobe: 32,
            rerank_width: 500,
            ivf_scratch_soa_batch_decode: None,
            session_gucs: Vec::new(),
            pg: None,
            db: None,
            socket_dir: None,
            port: None,
            sql_file: Some("explain.sql".into()),
            log_output: Some("explain.log".into()),
        };
        let sql = explain_sql(&step, &SuiteDefaults::default());

        assert!(sql.contains("SET ec_spire.nprobe = 32;"));
        assert!(sql.contains("SET ec_spire.rerank_width = 500;"));
        assert!(sql.contains("FROM ec_spire_index_cost_snapshot('spire_pfx_idx'::regclass);"));
        assert!(
            sql.contains("FROM ec_spire_index_cost_tuning_snapshot('spire_pfx_idx'::regclass);")
        );
        assert!(sql.contains("'ec_spire' AS profile"));
        assert!(sql.contains("RESET ec_spire.nprobe;"));
        assert!(sql.contains("RESET ec_spire.rerank_width;"));
    }

    #[test]
    fn explain_sql_uses_diskann_profile_guc_and_cost_snapshot() {
        let step = ExplainStep {
            name: "explain".into(),
            tags: Vec::new(),
            prefix: "diskann_pfx".into(),
            profile: Some("ec_diskann".into()),
            index_name: None,
            query_table: None,
            corpus_table: None,
            nprobe: 200,
            rerank_width: -1,
            ivf_scratch_soa_batch_decode: None,
            session_gucs: Vec::new(),
            pg: None,
            db: None,
            socket_dir: None,
            port: None,
            sql_file: Some("explain.sql".into()),
            log_output: Some("explain.log".into()),
        };
        let sql = explain_sql(&step, &SuiteDefaults::default());

        assert!(sql.contains("SET ec_diskann.list_size = 200;"));
        assert!(sql.contains("FROM ec_diskann_index_cost_snapshot('diskann_pfx_idx'::regclass);"));
        assert!(sql.contains("'ec_diskann' AS profile"));
        assert!(sql.contains("RESET ec_diskann.list_size;"));
    }
}
