//! `ecaz bench suite` — configured benchmark suite runner.
//!
//! Suites are JSON plans that expand into ordinary `ecaz` commands. The runner
//! keeps the expansion visible in a manifest, then optionally executes each
//! selected step in sequence.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::Write,
    path::{Path, PathBuf},
    process::ExitStatus,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, eyre, Context, ContextCompat, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::process::Command;

use crate::{
    profiles::{self, IndexProfile},
    psql::ConnectionOptions,
};

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
    /// Reconnect each latency worker after this many timed queries. Zero
    /// preserves the historical single-backend run.
    #[serde(default)]
    worker_batch_size: Option<usize>,
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
    DistannLocalMultinode(DistannLocalMultinodeStep),
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
    #[serde(default)]
    sample_backend_memory: Option<bool>,
    #[serde(default)]
    memory_sample_interval_ms: Option<u64>,
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
    concurrency_sweep: Vec<usize>,
    #[serde(default)]
    iterations: Option<usize>,
    /// Reconnect each latency worker after this many timed queries. Zero
    /// preserves the historical single-backend run.
    #[serde(default)]
    worker_batch_size: Option<usize>,
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
    ivf_stage_counters: Option<bool>,
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
    load_session_gucs: Vec<String>,
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

/// FR-082/NFR-020 distann multinode fixture step: drives
/// `ecaz dev distann-multicluster local-multinode-pg18`, which loads source rows
/// only on the coordinator, builds one physically sharded generation across N
/// real PG18 instances, validates Ready/Published topology on every owner, and
/// exercises cross-process serving. The historical replicated-serving control
/// has a separate explicit dev subcommand and is not topology evidence.
fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DistannBenchmarkSeedVariant {
    name: String,
    seed_strategy: String,
    head_search_width: u32,
    head_seed_count: u32,
    neighbor_score_mode: String,
    /// Task 184 benchmark-only global ranked-slot payload batch size. Zero
    /// preserves eager materialization.
    #[serde(default)]
    materialization_batch_size: u32,
    /// Task 193 benchmark-only generation/projection owner SPI-plan cache arm.
    #[serde(default)]
    owner_payload_plan_cache: Option<bool>,
    /// Task 194 per-variant beam width for fixed-work A/B attribution.
    #[serde(default)]
    beam_width: Option<u32>,
    /// Task 194 per-variant hop-round cap for fixed-work A/B attribution.
    #[serde(default)]
    hop_rounds: Option<u32>,
    /// Task 198 benchmark-only coordinator traversal replica arm.
    #[serde(default)]
    traversal_replica: bool,
    /// Task 218 MAT-21 benchmark-only typed/binary row locator arm.
    #[serde(default)]
    typed_locator: bool,
    /// Task 220 MAT-16 benchmark-only packed owner payload arm.
    #[serde(default)]
    packed_payload: bool,
    /// Task 221 MAT-22 benchmark-only expanded owner row locator arm.
    #[serde(default)]
    expanded_locator: bool,
    /// Task 222 benchmark-only all-column/projected payload A/B. Existing
    /// configs default to the production projected path.
    #[serde(default = "default_true")]
    payload_projection: bool,
    /// NFR-022 pre-registration for a decision-bearing arm. Repeated
    /// registrations use the same id across the 10k/50k/100k steps.
    #[serde(default)]
    nfr_021: Option<DistannNfr021Registration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DistannMetricsMode {
    Benchmark,
    FullMetrics,
}

impl DistannMetricsMode {
    fn label(self) -> &'static str {
        match self {
            Self::Benchmark => "benchmark",
            Self::FullMetrics => "full_metrics",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DistannDecisionRole {
    Control,
    Candidate,
    Context,
}

impl DistannDecisionRole {
    fn label(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Candidate => "candidate",
            Self::Context => "context",
        }
    }

    fn is_decision_bearing(self) -> bool {
        matches!(self, Self::Control | Self::Candidate)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DistannNfr021Admissibility {
    Conforming,
    Nonconforming,
}

impl DistannNfr021Admissibility {
    fn label(self) -> &'static str {
        match self {
            Self::Conforming => "conforming",
            Self::Nonconforming => "nonconforming",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DistannNfr021Registration {
    /// Stable arm identity shared by the scale-specific suite steps.
    id: String,
    role: DistannDecisionRole,
    admissibility: DistannNfr021Admissibility,
    /// Concise pre-measurement basis for the verdict.
    rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DistannNfr021ManifestRegistration {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    variant: Option<String>,
    id: String,
    role: DistannDecisionRole,
    admissibility: DistannNfr021Admissibility,
    rationale: String,
}

#[derive(Debug, Deserialize)]
struct DistannLocalMultinodeStep {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    pgoptions: Option<String>,
    #[serde(default)]
    artifact_dir: Option<PathBuf>,
    #[serde(default)]
    run_dir: Option<PathBuf>,
    /// Reuse a stopped, provenance-matched distann fixture. Rebuild remains
    /// the default when this opt-in is absent.
    #[serde(default)]
    reuse_fixture: bool,
    /// Packet-local provenance directory used to attest a reused fixture.
    #[serde(default)]
    reuse_provenance_dir: Option<PathBuf>,
    #[serde(default)]
    log_file: Option<PathBuf>,
    /// Keep only the compact summary as durable evidence. Raw fixture and
    /// per-arm logs may be pruned after results.jsonl has been materialized.
    #[serde(default)]
    compact_artifacts: bool,
    /// Intentional diagnostic override for a unanimous non-release extension.
    /// Production benchmark suites must leave this false.
    #[serde(default)]
    allow_debug_extension: bool,
    /// Route extension-owned coordinator-to-owner traffic through the
    /// fixture's verify-full mutual-TLS identity.
    #[serde(default)]
    secure_remote_transport: bool,
    /// Task 236 diagnostic-only TLS, mTLS, handshake-fault, and secret-
    /// rotation matrix.
    #[serde(default)]
    tls_security_matrix: bool,
    #[serde(default)]
    pg: Option<u16>,
    #[serde(default)]
    pgbin: Option<PathBuf>,
    #[serde(default)]
    nodes: Option<u32>,
    #[serde(default)]
    coordinator_outside_roster: bool,
    #[serde(default)]
    physical_benchmark: bool,
    #[serde(default)]
    benchmark_iterations: Option<u32>,
    /// Concurrent latency levels for the physical benchmark throughput curve.
    #[serde(default)]
    benchmark_concurrency_sweep: Vec<usize>,
    #[serde(default)]
    benchmark_warmup_iterations: Option<u32>,
    #[serde(default)]
    benchmark_parity_queries: Option<u32>,
    /// Per-scale shipped-default heldout deficit used by Task 167's
    /// baseline-relative regression gate. Must be paired with the physical
    /// sample SD; omit both to record a baseline observation.
    #[serde(default)]
    task167_heldout_baseline_deficit: Option<f64>,
    #[serde(default)]
    task167_heldout_physical_sample_sd: Option<f64>,
    #[serde(default)]
    benchmark_backend_batch_size: Option<u32>,
    /// Run the Task 200 repeated coverage-call RSS regression in one backend
    /// transaction. The fixture is reused when `reuse_fixture` is true.
    #[serde(default)]
    coverage_memory_regression_iterations: Option<u32>,
    /// Task 185 feature-only physical seed gateway/basin provenance.
    #[serde(default)]
    gateway_trace: bool,
    /// Task 227 bounded per-query traversal/frontier/exact-result trace over
    /// the configured evaluation query slice.
    #[serde(default)]
    query_trace: bool,
    /// Task 227 persisted-graph structure and seed-reachability diagnostic for
    /// physical owners plus the monolithic control.
    #[serde(default)]
    graph_diagnostic: bool,
    /// Task 227 truth-joined, query-level residual classification.
    #[serde(default)]
    residual_attribution: bool,
    /// Task 185 feature-only isolated attribution for every returned seed
    /// position. This is intentionally more expensive than gateway_trace.
    #[serde(default)]
    gateway_isolated_trace: bool,
    /// Maximum returned seed positions to isolate per training query.
    #[serde(default)]
    gateway_isolated_seed_limit: Option<u32>,
    /// Task 185 bounded arbitrary persisted-head candidate attribution.
    #[serde(default)]
    gateway_head_candidate_trace: bool,
    /// 1-based persisted-head positions to trace when enabled.
    #[serde(default)]
    gateway_head_candidate_positions: Vec<u32>,
    #[serde(default)]
    coverage_memory_regression_max_slope_kb_per_s: Option<f64>,
    #[serde(default)]
    coverage_memory_regression_max_delta_kb: Option<f64>,
    /// Record a timestamped /proc RSS/HWM series for each latency backend.
    #[serde(default)]
    sample_backend_memory: bool,
    /// Milliseconds between backend RSS/HWM samples.
    #[serde(default)]
    memory_sample_interval_ms: Option<u64>,
    /// Task 172 instrumentation contract. Benchmark mode is the lean gate
    /// surface; full_metrics enables attribution counters and memory sampling.
    /// Legacy configs without this field are labeled from their heavy
    /// instrumentation flags.
    #[serde(default)]
    metrics_mode: Option<DistannMetricsMode>,
    #[serde(default)]
    distann_stage_counters: bool,
    /// Task 224 owner heap/TOAST locality attribution projection. Four suite
    /// steps reuse one fixture to cover the registered shapes.
    #[serde(default)]
    owner_payload_shape: Option<String>,
    #[serde(default)]
    skip_owner_locality_profile: bool,
    #[serde(default)]
    owner_fast_real_array_send: bool,
    #[serde(default)]
    stage_counter_only: bool,
    #[serde(default)]
    skip_recall: bool,
    #[serde(default)]
    skip_single_control: bool,
    #[serde(default)]
    skip_single_benchmark: bool,
    /// Skip the expensive concurrent insert/query drill after the benchmark
    /// matrix when a dedicated concurrency gate is run separately.
    #[serde(default)]
    skip_concurrency_drill: bool,
    /// Preserve physical row count for a subsequent reuse-fixture step.
    #[serde(default)]
    skip_routed_delete_vacuum_drill: bool,
    #[serde(default)]
    materialization_correctness: bool,
    /// Run the Task 199 armed LD_PRELOAD ENOSPC replica-build drill.
    #[serde(default)]
    traversal_replica_enospc_drill: bool,
    #[serde(default)]
    base_port: Option<u16>,
    #[serde(default)]
    rows: Option<usize>,
    #[serde(default)]
    dim: Option<usize>,
    #[serde(default)]
    graph_degree: Option<u32>,
    #[serde(default)]
    build_shards: Option<u32>,
    #[serde(default)]
    head_construction: Option<String>,
    #[serde(default)]
    head_index_cap: Option<u32>,
    #[serde(default)]
    head_sampling_rate: Option<f64>,
    #[serde(default)]
    head_cap_floor: Option<u32>,
    #[serde(default)]
    head_cap_ceiling: Option<u32>,
    #[serde(default)]
    beam_width: Option<u32>,
    /// FR-081 retained candidate heap size L applied to benchmark query arms.
    #[serde(default)]
    candidate_heap_limit: Option<u32>,
    #[serde(default)]
    sharded_head: bool,
    #[serde(default)]
    head_replica_count: Option<u32>,
    #[serde(default)]
    gateway_copy_capacity: Option<u32>,
    #[serde(default)]
    crown_capacity: Option<u32>,
    #[serde(default)]
    crown_width_pruning: bool,
    #[serde(default)]
    fused_head_hop: bool,
    #[serde(default)]
    local_head: bool,
    #[serde(default)]
    hop_rounds: Option<u32>,
    #[serde(default)]
    seed_strategy: Option<String>,
    #[serde(default)]
    head_search_width: Option<u32>,
    #[serde(default)]
    head_seed_count: Option<u32>,
    #[serde(default)]
    neighbor_score_mode: Option<String>,
    #[serde(default)]
    head_policy: Option<String>,
    #[serde(default)]
    production_head_policy: Option<String>,
    #[serde(default)]
    training_query_path: Option<PathBuf>,
    /// Seed-search arms measured against one immutable physical generation.
    /// This avoids rebuilding identical 100k storage for every attribution
    /// setting while preserving one result row per named arm.
    #[serde(default)]
    benchmark_seed_variants: Vec<DistannBenchmarkSeedVariant>,
    /// NFR-022 pre-registration for the singular benchmark arm. Variant
    /// matrices register each arm on DistannBenchmarkSeedVariant instead.
    #[serde(default)]
    nfr_021: Option<DistannNfr021Registration>,
    /// Task 217 same-generation proof pair. The two named runtime arms must
    /// emit byte-identical prediction files while sharing one epoch.
    #[serde(default)]
    same_generation_recall_pair: Option<String>,
    #[serde(default)]
    queries: Option<u32>,
    /// Zero-based row offset into the staged query TSV. The fixture loads only
    /// `queries` rows starting here and records the exact slice digest.
    #[serde(default)]
    query_offset: Option<u32>,
    #[serde(default)]
    top_k: Option<u32>,
    #[serde(default)]
    skip_fault_drills: bool,
    #[serde(default)]
    drop_extension_cleanup_drill: bool,
    /// Load a real staged corpus (`{staged_dir}/{corpus_prefix}_*.tsv`) instead
    /// of the synthetic deterministic corpus. Makes this a real-corpus
    /// distributed quality lane (Task 172).
    #[serde(default)]
    corpus_prefix: Option<String>,
    #[serde(default)]
    staged_dir: Option<PathBuf>,
    /// Session GUCs applied to physical benchmark child commands.
    #[serde(default)]
    bench_session_gucs: Vec<String>,
}

impl DistannLocalMultinodeStep {
    fn effective_metrics_mode(&self) -> DistannMetricsMode {
        self.metrics_mode.unwrap_or({
            if self.distann_stage_counters
                || self.owner_payload_shape.is_some()
                || self.stage_counter_only
                || self.sample_backend_memory
            {
                DistannMetricsMode::FullMetrics
            } else {
                DistannMetricsMode::Benchmark
            }
        })
    }

    fn nfr_021_manifest_registrations(&self) -> Vec<DistannNfr021ManifestRegistration> {
        if self.benchmark_seed_variants.is_empty() {
            return self
                .nfr_021
                .iter()
                .map(|registration| DistannNfr021ManifestRegistration {
                    variant: None,
                    id: registration.id.clone(),
                    role: registration.role,
                    admissibility: registration.admissibility,
                    rationale: registration.rationale.clone(),
                })
                .collect();
        }
        self.benchmark_seed_variants
            .iter()
            .filter_map(|variant| {
                variant
                    .nfr_021
                    .as_ref()
                    .map(|registration| DistannNfr021ManifestRegistration {
                        variant: Some(variant.name.clone()),
                        id: registration.id.clone(),
                        role: registration.role,
                        admissibility: registration.admissibility,
                        rationale: registration.rationale.clone(),
                    })
            })
            .collect()
    }
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
    #[serde(default)]
    result_identity_output: Option<PathBuf>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    runner_git_commit: Option<String>,
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
    git_commit: Option<String>,
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
    nfr_021_registrations: Vec<DistannNfr021ManifestRegistration>,
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

    let mut rows = extract_result_rows(&manifest).await?;
    enrich_distann_result_rows(&manifest, &mut rows)?;
    if let Some(path) = args.results_output.clone().or_else(|| {
        config
            .artifact_dir
            .as_ref()
            .map(|dir| dir.join("results.jsonl"))
    }) {
        write_results_jsonl(&path, &rows).await?;
        crate::ecaz_eprintln!("[suite:{}] wrote {}", config.name, path.display());
    }
    assert_distann_nfr_021_registrations(&rows)?;
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
    let mut rows = extract_result_rows(&manifest).await?;
    enrich_distann_result_rows(&manifest, &mut rows)?;
    crate::ecaz_println!("# Suite Report: {}", manifest.suite);
    crate::ecaz_println!("");
    crate::ecaz_println!("- config: `{}`", manifest.config);
    crate::ecaz_println!("- config_sha256: `{}`", manifest.config_sha256);
    crate::ecaz_println!("- dry_run: `{}`", manifest.dry_run);
    if let Some(runner_git_commit) = &manifest.runner_git_commit {
        crate::ecaz_println!("- runner_git_commit: `{runner_git_commit}`");
    }
    if let Some(backend) = &manifest.backend {
        crate::ecaz_println!(
            "- backend: profile `{}`, git `{}`, sha256 `{}`",
            backend.build_profile,
            backend.git_commit.as_deref().unwrap_or("unknown"),
            backend.sha256.as_deref().unwrap_or("unknown")
        );
    }
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
            SuiteStep::DistannLocalMultinode(step) => {
                if step.artifact_dir.is_none() {
                    step.artifact_dir =
                        Some(artifact_dir.join(artifact_safe_step_name(&step.name)));
                }
                if step.log_file.is_none() {
                    step.log_file = step
                        .artifact_dir
                        .as_ref()
                        .map(|dir| dir.join("distann-local-multinode.log"));
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
            SuiteStep::DistannLocalMultinode(step) => {
                rewrite_artifact_dir_path(&mut step.artifact_dir, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.run_dir, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.log_file, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.pgbin, &artifact_dir);
                rewrite_artifact_dir_path(&mut step.reuse_provenance_dir, &artifact_dir);
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
                rewrite_artifact_dir_path(&mut step.result_identity_output, &artifact_dir);
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
        // Capture before write_manifest_if_requested creates or updates any
        // packet-local outputs. The build-time stamp remains the fallback when
        // the runner is invoked outside a Git checkout.
        runner_git_commit: Some(capture_runner_git_descriptor()),
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
        let mut tags = step.tags().to_vec();
        if let SuiteStep::DistannLocalMultinode(distann) = step {
            tags.retain(|tag| !tag.starts_with("metrics_mode="));
            tags.push(format!(
                "metrics_mode={}",
                distann.effective_metrics_mode().label()
            ));
        }
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
            tags,
            nfr_021_registrations: match step {
                SuiteStep::DistannLocalMultinode(distann) => {
                    distann.nfr_021_manifest_registrations()
                }
                _ => Vec::new(),
            },
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

fn capture_runner_git_descriptor() -> String {
    let head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|head| !head.is_empty());
    let dirty = std::process::Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| !output.stdout.is_empty());
    match (head, dirty) {
        (Some(head), Some(true)) => format!("{head}-dirty"),
        (Some(head), Some(false)) => head,
        _ => env!("ECAZ_GIT_SHA").to_owned(),
    }
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

#[derive(Debug, Clone, Serialize)]
struct ResultRow {
    suite: String,
    step: String,
    kind: String,
    metric: String,
    artifact: String,
    values: BTreeMap<String, String>,
}

fn enrich_distann_result_rows(manifest: &SuiteManifest, rows: &mut Vec<ResultRow>) -> Result<()> {
    assert_distann_storage_ratio_rows(manifest, rows)?;
    let growth_rows = distann_storage_growth_rows(rows);
    rows.extend(growth_rows);
    let nfr_021_rows = distann_nfr_021_conformance_rows(manifest, rows);
    rows.extend(nfr_021_rows);
    Ok(())
}

fn assert_distann_storage_ratio_rows(manifest: &SuiteManifest, rows: &[ResultRow]) -> Result<()> {
    for step in manifest.steps.iter().filter(|step| {
        step.selected
            && step.kind == "distann-local-multinode"
            && matches!(step.status, Some(StepStatus::Succeeded))
            && step.command.iter().any(|arg| arg == "--physical-benchmark")
    }) {
        let storage_keys = rows
            .iter()
            .filter(|row| row.step == step.name && row.metric == "physical_benchmark_storage")
            .filter_map(storage_identity_key)
            .collect::<HashSet<_>>();
        let ratio_keys = rows
            .iter()
            .filter(|row| row.step == step.name && row.metric == "physical_benchmark_storage_ratio")
            .filter_map(storage_identity_key)
            .collect::<HashSet<_>>();
        if storage_keys.is_empty() {
            bail!(
                "distann physical benchmark step {:?} is missing physical_benchmark_storage",
                step.name
            );
        }
        let missing = storage_keys
            .difference(&ratio_keys)
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            bail!(
                "distann physical benchmark step {:?} is missing physical_benchmark_storage_ratio for {:?}",
                step.name,
                missing
            );
        }
    }
    Ok(())
}

fn storage_identity_key(row: &ResultRow) -> Option<(String, String, String)> {
    Some((
        row.values.get("scale")?.clone(),
        row.values.get("variant")?.clone(),
        row.values.get("arm")?.clone(),
    ))
}

fn distann_storage_growth_rows(rows: &[ResultRow]) -> Vec<ResultRow> {
    let mut by_node: HashMap<(String, String, String), HashMap<String, (String, f64)>> =
        HashMap::new();
    for row in rows
        .iter()
        .filter(|row| row.metric == "physical_benchmark_storage_node")
    {
        let (Some(variant), Some(arm), Some(node), Some(scale), Some(bytes)) = (
            row.values.get("variant"),
            row.values.get("arm"),
            row.values.get("node"),
            row.values.get("scale"),
            row.values
                .get("total_resident_bytes")
                .and_then(|value| value.parse::<f64>().ok()),
        ) else {
            continue;
        };
        by_node
            .entry((variant.clone(), arm.clone(), node.clone()))
            .or_default()
            .insert(scale.clone(), (row.step.clone(), bytes));
    }

    by_node
        .into_iter()
        .filter_map(|((variant, arm, node), scales)| {
            let (low_step, low) = scales.get("10k")?.clone();
            let (high_step, high) = scales.get("100k")?.clone();
            if low <= 0.0 {
                return None;
            }
            let ratio = high / low;
            Some(ResultRow {
                suite: rows.first()?.suite.clone(),
                step: "suite-storage-growth".into(),
                kind: "storage-growth".into(),
                metric: "physical_benchmark_storage_growth".into(),
                artifact: "suite-derived".into(),
                values: BTreeMap::from([
                    ("scale_low".into(), "10k".into()),
                    ("scale_high".into(), "100k".into()),
                    ("variant".into(), variant),
                    ("arm".into(), arm),
                    ("node".into(), node),
                    ("low_step".into(), low_step),
                    ("high_step".into(), high_step),
                    ("low_total_resident_bytes".into(), format_bytes(low)),
                    ("high_total_resident_bytes".into(), format_bytes(high)),
                    ("growth_ratio".into(), format!("{ratio:.6}")),
                    (
                        "judgement".into(),
                        "reported_not_threshold_fixed_roster".into(),
                    ),
                ]),
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DistannNfr021Actual {
    Conforming,
    Nonconforming,
    Unavailable,
}

impl DistannNfr021Actual {
    fn label(self) -> &'static str {
        match self {
            Self::Conforming => "conforming",
            Self::Nonconforming => "nonconforming",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Default)]
struct DistannNfr021Evidence {
    scales: HashSet<String>,
    topology_scales: HashSet<String>,
    bytes_per_owned_record: HashMap<(String, String), f64>,
    raw_graph_side_bytes: HashMap<(String, String), f64>,
    owner_nodes: HashSet<String>,
    head_capacities: HashSet<String>,
    max_non_owned_records: u64,
    max_orphan_vectors: u64,
    max_unsharded_derived_bytes: u64,
    missing_owned_record_counts: usize,
    /// NFR-021 clause 2/3: coordinator-resident structures that are not
    /// sharded, keyed by relation name. Task 210 P2 closes the only entries
    /// this set is allowed to hold; anything else is a hard violation.
    coordinator_resident_unsharded: BTreeMap<String, u64>,
}

/// Structures known to be coordinator-resident and unsharded, with the phase
/// that removes them. They are reported on every conformance row until that
/// phase lands, and any relation NOT on this list is a hard violation rather
/// than a known gap. Delete an entry when its phase ships — a reappearance
/// then fails the suite instead of being absorbed.
/// Task 210 closed the last owned distribution gap (005 review round 2): the
/// membership-only head persists zero sample/graph rows, so any
/// coordinator-resident unsharded relation reporting non-zero bytes is now a
/// hard violation — a reappearance fails the suite rather than re-entering an
/// allowlist.
const NFR_021_KNOWN_DISTRIBUTION_GAPS: [(&str, &str); 0] = [];

fn distann_nfr_021_conformance_rows(
    manifest: &SuiteManifest,
    rows: &[ResultRow],
) -> Vec<ResultRow> {
    let mut registrations: BTreeMap<String, DistannNfr021ManifestRegistration> = BTreeMap::new();
    let mut evidence: HashMap<String, DistannNfr021Evidence> = HashMap::new();

    for step in manifest.steps.iter().filter(|step| {
        step.selected
            && step.kind == "distann-local-multinode"
            && matches!(step.status, Some(StepStatus::Succeeded))
    }) {
        for registration in &step.nfr_021_registrations {
            registrations
                .entry(registration.id.clone())
                .or_insert_with(|| registration.clone());
        }
    }

    // A registration identifies a variant, not a single corpus scale. Allow
    // one declaration (normally on the first scale) to collect its matching
    // rows from every successful physical step in the matrix.
    for step in manifest.steps.iter().filter(|step| {
        step.selected
            && step.kind == "distann-local-multinode"
            && matches!(step.status, Some(StepStatus::Succeeded))
    }) {
        for registration in registrations.values() {
            let arm_evidence = evidence.entry(registration.id.clone()).or_default();
            collect_distann_nfr_021_step_evidence(step, registration, rows, arm_evidence);
        }
    }

    registrations
        .into_iter()
        .map(|(id, registration)| {
            let arm_evidence = evidence.remove(&id).unwrap_or_default();
            distann_nfr_021_result_row(manifest, id, registration, arm_evidence)
        })
        .collect()
}

fn collect_distann_nfr_021_step_evidence(
    step: &StepRecord,
    registration: &DistannNfr021ManifestRegistration,
    rows: &[ResultRow],
    evidence: &mut DistannNfr021Evidence,
) {
    let step_rows = rows.iter().filter(|row| row.step == step.name);
    let storage_rows = step_rows
        .clone()
        .filter(|row| row.metric == "physical_benchmark_storage_node")
        .filter(|row| nfr_021_row_matches_variant(row, registration))
        .collect::<Vec<_>>();
    let Some(scale) = storage_rows
        .iter()
        .find_map(|row| row.values.get("scale").cloned())
    else {
        return;
    };
    evidence.scales.insert(scale.clone());

    let topology_by_node = step_rows
        .clone()
        .filter(|row| row.metric == "physical_topology")
        .filter(|row| {
            row.values
                .get("phase")
                .is_some_and(|phase| phase == "published")
        })
        .filter_map(|row| {
            let node = row.values.get("node")?.clone();
            let records = row.values.get("records")?.parse::<u64>().ok()?;
            Some((node, (records, row)))
        })
        .collect::<HashMap<_, _>>();
    if !topology_by_node.is_empty() {
        evidence.topology_scales.insert(scale.clone());
    }
    for (_, row) in topology_by_node.values() {
        evidence.max_non_owned_records = evidence.max_non_owned_records.max(
            row.values
                .get("non_owned")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(u64::MAX),
        );
        evidence.max_orphan_vectors = evidence.max_orphan_vectors.max(
            row.values
                .get("orphans")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(u64::MAX),
        );
    }

    for row in storage_rows {
        if let Some(capacity) = row.values.get("head_index_cap") {
            evidence.head_capacities.insert(capacity.clone());
        }
        if !matches!(row.values.get("node_role"), Some(role) if role == "owner") {
            continue;
        }
        let (Some(node), Some(graph_side_bytes)) = (
            row.values.get("node"),
            row.values
                .get("graph_side_bytes")
                .and_then(|value| value.parse::<f64>().ok()),
        ) else {
            evidence.missing_owned_record_counts += 1;
            continue;
        };
        evidence.owner_nodes.insert(node.clone());
        evidence
            .raw_graph_side_bytes
            .insert((scale.clone(), node.clone()), graph_side_bytes);
        let Some((owned_records, _)) = topology_by_node.get(node) else {
            evidence.missing_owned_record_counts += 1;
            continue;
        };
        if *owned_records == 0 {
            evidence.missing_owned_record_counts += 1;
            continue;
        }
        evidence.bytes_per_owned_record.insert(
            (scale.clone(), node.clone()),
            graph_side_bytes / *owned_records as f64,
        );
    }

    for row in step_rows
        .filter(|row| row.metric == "physical_benchmark_storage_relation")
        .filter(|row| nfr_021_row_matches_variant(row, registration))
    {
        // `bounded` structures are NFR-021-permitted by size argument;
        // `control` rows are control-plane metadata (digests, counts, the
        // membership-only head's bounded id blob — roster-like state). Neither
        // is corpus-derived coordinator state, so neither feeds the
        // derived-bytes hard violation.
        if row
            .values
            .get("nfr_021_class")
            .is_some_and(|class| class == "bounded" || class == "control")
        {
            continue;
        }
        let bytes = row
            .values
            .get("relation_bytes")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(u64::MAX);
        // A coordinator-resident unsharded structure is a distribution gap, not
        // a derived-relation violation: it is reported by name on every
        // conformance row and checked against the known-gap list below.
        if row
            .values
            .get("nfr_021_class")
            .is_some_and(|class| class == "coordinator_resident_unsharded")
        {
            let relation = row
                .values
                .get("relation")
                .cloned()
                .unwrap_or_else(|| "unnamed".to_owned());
            let entry = evidence
                .coordinator_resident_unsharded
                .entry(relation)
                .or_default();
            *entry = (*entry).max(bytes);
            continue;
        }
        evidence.max_unsharded_derived_bytes = evidence.max_unsharded_derived_bytes.max(bytes);
    }
}

fn nfr_021_row_matches_variant(
    row: &ResultRow,
    registration: &DistannNfr021ManifestRegistration,
) -> bool {
    if row.values.get("arm").is_some_and(|arm| arm != "physical") {
        return false;
    }
    match &registration.variant {
        None => true,
        Some(variant) => row
            .values
            .get("variant")
            .is_some_and(|row_variant| row_variant == variant),
    }
}

fn distann_nfr_021_result_row(
    manifest: &SuiteManifest,
    id: String,
    registration: DistannNfr021ManifestRegistration,
    evidence: DistannNfr021Evidence,
) -> ResultRow {
    const REQUIRED_SCALES: [&str; 3] = ["10k", "50k", "100k"];
    const NORMALIZED_GROWTH_THRESHOLD: f64 = 2.0;

    let mut normalized_ratios = Vec::new();
    let mut raw_ratios = Vec::new();
    let owner_matrix_complete = !evidence.owner_nodes.is_empty()
        && evidence.owner_nodes.iter().all(|node| {
            if !REQUIRED_SCALES.iter().all(|scale| {
                evidence
                    .bytes_per_owned_record
                    .contains_key(&(scale.to_string(), node.clone()))
            }) {
                return false;
            }
            let low_key = ("10k".to_owned(), node.clone());
            let high_key = ("100k".to_owned(), node.clone());
            match (
                evidence.bytes_per_owned_record.get(&low_key),
                evidence.bytes_per_owned_record.get(&high_key),
            ) {
                (Some(low), Some(high)) if *low > 0.0 => {
                    normalized_ratios.push(high / low);
                    if let (Some(raw_low), Some(raw_high)) = (
                        evidence.raw_graph_side_bytes.get(&low_key),
                        evidence.raw_graph_side_bytes.get(&high_key),
                    ) {
                        if *raw_low > 0.0 {
                            raw_ratios.push(raw_high / raw_low);
                        }
                    }
                    true
                }
                _ => false,
            }
        });
    let scales_complete = REQUIRED_SCALES
        .iter()
        .all(|scale| evidence.scales.contains(*scale));
    let topology_complete = REQUIRED_SCALES
        .iter()
        .all(|scale| evidence.topology_scales.contains(*scale));
    let normalized_growth_max = normalized_ratios.into_iter().reduce(f64::max);
    let raw_growth_max = raw_ratios.into_iter().reduce(f64::max);
    let head_capacity_constant = evidence.head_capacities.len() == 1;
    let evidence_complete = scales_complete
        && topology_complete
        && owner_matrix_complete
        && evidence.missing_owned_record_counts == 0
        && head_capacity_constant
        && normalized_growth_max.is_some();
    // A coordinator-resident unsharded relation on the known list is an owned,
    // dated gap: reported loudly on every row, not silently absorbed and not
    // used to fail unrelated lanes. Anything off the list is a hard violation.
    let known_gap_relations = NFR_021_KNOWN_DISTRIBUTION_GAPS
        .iter()
        .map(|(relation, _)| *relation)
        .collect::<HashSet<_>>();
    let unexpected_coordinator_resident = evidence
        .coordinator_resident_unsharded
        .iter()
        .any(|(relation, bytes)| *bytes > 0 && !known_gap_relations.contains(relation.as_str()));
    let outstanding_gap = evidence
        .coordinator_resident_unsharded
        .iter()
        .filter(|(_, bytes)| **bytes > 0)
        .map(|(relation, bytes)| {
            let owner = NFR_021_KNOWN_DISTRIBUTION_GAPS
                .iter()
                .find(|(known, _)| *known == relation.as_str())
                .map_or("unowned", |(_, owner)| *owner);
            format!("{relation}:{bytes}:{owner}")
        })
        .collect::<Vec<_>>();
    let coordinator_resident_unsharded_bytes = evidence
        .coordinator_resident_unsharded
        .values()
        .copied()
        .sum::<u64>();
    let hard_violation = evidence.max_non_owned_records > 0
        || evidence.max_orphan_vectors > 0
        || evidence.max_unsharded_derived_bytes > 0
        || unexpected_coordinator_resident
        || evidence.head_capacities.len() > 1
        || normalized_growth_max.is_some_and(|ratio| ratio > NORMALIZED_GROWTH_THRESHOLD);
    let actual = if hard_violation {
        DistannNfr021Actual::Nonconforming
    } else if evidence_complete {
        DistannNfr021Actual::Conforming
    } else {
        DistannNfr021Actual::Unavailable
    };
    let preregistration_matches = match actual {
        DistannNfr021Actual::Conforming => {
            registration.admissibility == DistannNfr021Admissibility::Conforming
        }
        DistannNfr021Actual::Nonconforming => {
            registration.admissibility == DistannNfr021Admissibility::Nonconforming
        }
        DistannNfr021Actual::Unavailable => false,
    };
    let mut values = BTreeMap::from([
        ("nfr_021_id".into(), id),
        ("nfr_021_role".into(), registration.role.label().into()),
        (
            "nfr_021_preregistered_admissibility".into(),
            registration.admissibility.label().into(),
        ),
        ("actual_admissibility".into(), actual.label().into()),
        ("evidence_complete".into(), evidence_complete.to_string()),
        (
            "preregistration_matches".into(),
            preregistration_matches.to_string(),
        ),
        (
            "decision_eligible".into(),
            (actual == DistannNfr021Actual::Conforming).to_string(),
        ),
        (
            "normalized_growth_threshold".into(),
            format!("{NORMALIZED_GROWTH_THRESHOLD:.1}"),
        ),
        (
            "max_non_owned_records".into(),
            evidence.max_non_owned_records.to_string(),
        ),
        (
            "max_orphan_vectors".into(),
            evidence.max_orphan_vectors.to_string(),
        ),
        (
            "max_unsharded_derived_bytes".into(),
            evidence.max_unsharded_derived_bytes.to_string(),
        ),
        (
            "head_capacity_constant".into(),
            head_capacity_constant.to_string(),
        ),
        (
            "missing_owned_record_counts".into(),
            evidence.missing_owned_record_counts.to_string(),
        ),
        (
            "coordinator_resident_unsharded_bytes".into(),
            coordinator_resident_unsharded_bytes.to_string(),
        ),
        (
            "outstanding_distribution_gap".into(),
            if outstanding_gap.is_empty() {
                "none".to_owned()
            } else {
                outstanding_gap.join(",")
            },
        ),
    ]);
    if let Some(ratio) = normalized_growth_max {
        values.insert(
            "normalized_bytes_per_owned_record_growth_max".into(),
            format!("{ratio:.6}"),
        );
    }
    if let Some(ratio) = raw_growth_max {
        values.insert(
            "raw_fixed_roster_graph_side_growth_max".into(),
            format!("{ratio:.6}"),
        );
    }
    let mut scales = evidence.scales.into_iter().collect::<Vec<_>>();
    scales.sort();
    values.insert("scales".into(), scales.join(","));

    ResultRow {
        suite: manifest.suite.clone(),
        step: "suite-nfr-021".into(),
        kind: "distann-conformance".into(),
        metric: "physical_benchmark_nfr_021_conformance".into(),
        artifact: "suite-derived".into(),
        values,
    }
}

fn assert_distann_nfr_021_registrations(rows: &[ResultRow]) -> Result<()> {
    // The conformance row is a cross-scale assertion.  A resumed or
    // step-scoped suite run may intentionally contain only one or two scales;
    // defer the assertion until the result row covers the complete matrix so
    // that `--only`/`--resume-from` can make progress without weakening the
    // final-gate check.
    let complete_matrix_present = rows.iter().any(|row| {
        row.metric == "physical_benchmark_nfr_021_conformance"
            && row.values.get("scales").is_some_and(|scales| {
                ["10k", "50k", "100k"]
                    .iter()
                    .all(|scale| scales.split(',').any(|value| value == *scale))
            })
    });
    if !complete_matrix_present {
        return Ok(());
    }
    let failures = rows
        .iter()
        .filter(|row| row.metric == "physical_benchmark_nfr_021_conformance")
        .filter(|row| {
            !row.values
                .get("preregistration_matches")
                .is_some_and(|value| value == "true")
        })
        .map(|row| {
            format!(
                "{}: preregistered={} actual={}",
                row.values
                    .get("nfr_021_id")
                    .map(String::as_str)
                    .unwrap_or("unknown"),
                row.values
                    .get("nfr_021_preregistered_admissibility")
                    .map(String::as_str)
                    .unwrap_or("missing"),
                row.values
                    .get("actual_admissibility")
                    .map(String::as_str)
                    .unwrap_or("missing")
            )
        })
        .collect::<Vec<_>>();
    if !failures.is_empty() {
        bail!(
            "NFR-021 conformance evidence did not match pre-registration: {}",
            failures.join("; ")
        )
    }
    Ok(())
}

fn format_bytes(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

async fn extract_result_rows(manifest: &SuiteManifest) -> Result<Vec<ResultRow>> {
    let mut rows = Vec::new();
    for step in &manifest.steps {
        if let Some(row) = kernel_cell_result_row(manifest, step) {
            rows.push(row);
        }
        let succeeded = matches!(step.status, Some(StepStatus::Succeeded));
        let failed = matches!(step.status, Some(StepStatus::Failed));
        if !succeeded && !failed {
            continue;
        }
        if succeeded {
            if let Some(row) = parallel_worker_result_row(manifest, step) {
                rows.push(row);
            }
        }
        let artifacts = result_artifacts_for_step(step);
        for artifact in &artifacts {
            let path = Path::new(artifact);
            let Ok(raw) = tokio::fs::read_to_string(path).await else {
                continue;
            };
            rows.extend(parse_result_rows(manifest, step, artifact, &raw));
        }
    }
    Ok(rows)
}

fn result_artifacts_for_step(step: &StepRecord) -> Vec<String> {
    let mut artifacts = step.expected_artifacts.clone();
    // A hard-gated multinode child can fail before it writes its compact
    // summary. Its primary --log-file still contains the emitted failing
    // metric and integrity checkpoints, so retain those structured rows
    // instead of producing an empty results.jsonl for the failed step.
    if matches!(step.status, Some(StepStatus::Failed)) && step.kind == "distann-local-multinode" {
        if let Some(log_file) = command_flag_value(&step.command, "--log-file") {
            if !artifacts.contains(&log_file) {
                artifacts.push(log_file);
            }
        }
    }
    artifacts
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
        "distann-local-multinode" => parse_distann_multinode_rows(raw)
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
        "raw" => parse_raw_result_rows(raw)
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
    insert_if_absent(
        &mut values,
        "metrics_mode",
        tag_value(&step.tags, "metrics_mode=").as_deref(),
    );
    if let Some(registration) = matching_nfr_021_registration(step, &values) {
        insert_if_absent(&mut values, "nfr_021_id", Some(&registration.id));
        insert_if_absent(&mut values, "nfr_021_role", Some(registration.role.label()));
        insert_if_absent(
            &mut values,
            "nfr_021_preregistered_admissibility",
            Some(registration.admissibility.label()),
        );
    }
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

fn matching_nfr_021_registration<'a>(
    step: &'a StepRecord,
    values: &BTreeMap<String, String>,
) -> Option<&'a DistannNfr021ManifestRegistration> {
    if values.get("arm").is_some_and(|arm| arm != "physical") {
        return None;
    }
    if let Some(variant) = values.get("variant") {
        return step.nfr_021_registrations.iter().find(|registration| {
            registration
                .variant
                .as_deref()
                .is_some_and(|name| name == variant)
                || (registration.variant.is_none() && step.nfr_021_registrations.len() == 1)
        });
    }
    (step.nfr_021_registrations.len() == 1).then(|| &step.nfr_021_registrations[0])
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

/// Normalize decision-grade output from a raw suite step without teaching the
/// runner about the command that produced it. The emitting command owns the
/// metric name and whitespace-delimited key/value fields; the suite owns
/// provenance, JSONL persistence, and threshold evaluation.
fn parse_raw_result_rows(raw: &str) -> Vec<(String, BTreeMap<String, String>)> {
    const PREFIX: &str = "[suite-result] ";
    raw.lines()
        .filter_map(|line| {
            let body = line.trim().strip_prefix(PREFIX)?;
            let (metric, fields) = body.split_once(char::is_whitespace)?;
            if metric.is_empty()
                || !metric
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
            {
                return None;
            }
            parse_space_key_values(fields.trim()).map(|values| (metric.to_owned(), values))
        })
        .collect()
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
        } else if let Some(rest) = line.trim_start().strip_prefix("[loader] build_memory ") {
            if let Some(values) = parse_space_key_values(rest) {
                rows.push(("build_memory".into(), values));
            }
        } else if let Some(rest) = line
            .trim_start()
            .strip_prefix("[postgres notice] ec_distann sharded build: ")
        {
            if let Some(values) = parse_space_key_values(rest) {
                rows.push(("distann_shard_build".into(), values));
            }
        } else if let Some((name, seconds)) = parse_timed_loader_line(line, "completed prefix ") {
            rows.push(("load_timing".into(), timed_values("total", &name, seconds)));
        }
    }
    rows
}

/// Parse the `ecaz dev distann-multicluster` fixture log emitted by a
/// `distann-local-multinode` suite step into structured result rows. The
/// fixture prints `[distann-multicluster] ...` lines; these shapes carry
/// decision-grade signal (027-P1 — the empty-`results.jsonl` fix):
///
/// - `release_profile_preflight status=passed ...` — the unanimous extension
///   provenance gate emitted and flushed before expensive fixture setup.
/// - `RECALL_RESULT n_queries=.. identical=.. mismatched_ids=..` — the
///   byte-identical single-vs-multi top-k distinct-recall identity. Emits a
///   `distinct_recall_identity` row with an `identity_ok` threshold
///   (`mismatched_ids == 0`).
/// - `suite_recall_gate single=.. multi=.. delta=.. pass=..` — the
///   `ecaz bench recall` single-vs-multi gate. Emits a `suite_recall_gate` row.
/// - any `<drill> pass=<bool>` line (qual, FR-082, fault drills, concurrency,
///   retention, AC-5, disjoint, recovery) — emits a `drill_outcome` row with
///   the drill label and pass flag, so every asserted fixture arm traces to a
///   result row.
fn parse_distann_multinode_rows(raw: &str) -> Vec<(String, BTreeMap<String, String>)> {
    const PREFIX: &str = "[distann-multicluster] ";
    let mut rows = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("[postgres notice] ec_distann_scan_round ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_scan_round".into(), values));
            }
            continue;
        }
        let body = if let Some(idx) = line.find(PREFIX) {
            &line[idx + PREFIX.len()..]
        } else if let Some(idx) = line.find("physical_benchmark_post_insert_exact_recall ") {
            // A hard-gate error repeats the exact-recall metric after the
            // color-eyre context but without the fixture prefix.
            &line[idx..]
        } else {
            continue;
        };
        // Failure diagnostics can wrap the original fixture line in a
        // color-eyre ANSI span. The metric itself precedes the first escape;
        // discard the presentation suffix so its final key/value remains
        // parseable (for example `pass=false`).
        let body = body.split('\u{1b}').next().unwrap_or(body).trim();
        // Physical benchmark child stderr is appended to the fixture summary
        // with the fixture prefix, so notices arrive as
        // `[distann-multicluster] [postgres notice] ...`. Keep accepting the
        // unwrapped form above for direct parser callers, but parse the
        // durable summary shape here as well.
        if let Some(rest) = body.strip_prefix("[postgres notice] ec_distann_scan_round ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_scan_round".into(), values));
            }
            continue;
        }
        if let Some(rest) = body.strip_prefix("release_profile_preflight ") {
            if let Some(mut values) = parse_space_key_values(rest.trim()) {
                let passed = values.get("status").is_some_and(|value| value == "passed")
                    && values.get("unanimous").is_some_and(|value| value == "true");
                values.insert("pass_numeric".into(), if passed { "1" } else { "0" }.into());
                rows.push(("multinode_release_preflight".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("RECALL_RESULT") {
            if let Some(mut values) = parse_space_key_values(rest.trim()) {
                let identity_ok = values
                    .get("mismatched_ids")
                    .map(|m| m == "0")
                    .unwrap_or(false);
                values.insert("identity_ok".into(), identity_ok.to_string());
                rows.push(("distinct_recall_identity".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("suite_recall_gate ") {
            // Only the measured form carries `single=`; SKIPPED/INCONCLUSIVE do not.
            if let Some(values) = parse_space_key_values(rest.trim()) {
                if values.contains_key("single") {
                    rows.push(("suite_recall_gate".into(), values));
                }
            }
        } else if let Some(rest) = body.strip_prefix("storage_summation ") {
            // Task 172 AC-3 / NFR-018 cluster storage summation.
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("storage_summation".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("storage_node ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("storage_node".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_topology ") {
            if let Some(mut values) = parse_space_key_values(rest.trim()) {
                let topology_ok = values
                    .get("state")
                    .is_some_and(|state| state == "Ready" || state == "Published")
                    && values.get("non_owned").is_some_and(|value| value == "0")
                    && values.get("orphans").is_some_and(|value| value == "0");
                values.insert("topology_ok".into(), topology_ok.to_string());
                rows.push(("physical_topology".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_recall ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_recall".into(), values));
            }
        } else if let Some(rest) =
            body.strip_prefix("physical_benchmark_recall_instrument_calibration ")
        {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push((
                    "physical_benchmark_recall_instrument_calibration".into(),
                    values,
                ));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_post_insert_exact_recall ")
        {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_post_insert_exact_recall".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_insert_throughput_ab ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_insert_throughput_ab".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_append_when_room_ab ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_append_when_room_ab".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_backlink_strategy_ab ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_backlink_strategy_ab".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_insert_work ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_insert_work".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_paired_recall ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_paired_recall".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_provenance ") {
            if let Some(mut values) = parse_space_key_values(rest.trim()) {
                if let Some(unanimous) = values.get("unanimous").map(|value| value == "true") {
                    values.insert(
                        "unanimous_numeric".into(),
                        if unanimous { "1" } else { "0" }.into(),
                    );
                }
                rows.push(("physical_benchmark_provenance".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_generation ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_generation".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_same_generation_recall ") {
            if let Some(mut values) = parse_space_key_values(rest.trim()) {
                let identical = values
                    .get("byte_identical")
                    .is_some_and(|value| value == "true");
                values.insert(
                    "byte_identical_numeric".into(),
                    if identical { "1" } else { "0" }.into(),
                );
                rows.push(("physical_benchmark_same_generation_recall".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_latency ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_latency".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_stage ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_stage".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_materialization_work ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_materialization_work".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_storage ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_storage".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_storage_node ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_storage_node".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_storage_relation ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_storage_relation".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_storage_ratio ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_storage_ratio".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_traversal_replica ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_traversal_replica".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_traversal_replica_cache ")
        {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_traversal_replica_cache".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_head ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_head".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_head_membership ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_head_membership".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_head_policy ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_head_policy".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_build ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_build".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_landmark ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_landmark".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_coverage ") {
            if let Some(values) = parse_space_key_values(rest.trim()) {
                rows.push(("physical_benchmark_coverage".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_benchmark_engagement ") {
            if let Some(mut values) = parse_space_key_values(rest.trim()) {
                if let Some(pass) = values.get("pass").map(|value| value == "true") {
                    values.insert("pass_numeric".into(), if pass { "1" } else { "0" }.into());
                }
                rows.push(("physical_benchmark_engagement".into(), values));
            }
        } else if let Some(rest) = body.strip_prefix("physical_materialization_correctness ") {
            if let Some(mut values) = parse_space_key_values(rest.trim()) {
                let pass = values.get("pass").is_some_and(|value| value == "true");
                values.insert("pass_numeric".into(), if pass { "1" } else { "0" }.into());
                rows.push(("physical_materialization_correctness".into(), values));
            }
        } else if let Some(pass_idx) = body.find(" pass=") {
            // A generic drill-outcome line: `<label> pass=<bool|skipped>`.
            // Preserve an explicit skip and its reason without turning it into
            // either a passing or failing measured drill.
            let label = body[..pass_idx].trim();
            let outcome = &body[pass_idx + 1..];
            if let Some(mut values) = parse_space_key_values(outcome) {
                let pass_token = values.get("pass").cloned().unwrap_or_default();
                if !matches!(pass_token.as_str(), "true" | "false" | "skipped") || label.is_empty()
                {
                    continue;
                }
                values.insert("drill".into(), sanitize_drill_label(label));
                if pass_token != "skipped" {
                    values.insert(
                        "pass_numeric".into(),
                        if pass_token == "true" { "1" } else { "0" }.into(),
                    );
                }
                rows.push(("drill_outcome".into(), values));
            }
        }
    }
    rows
}

/// Collapse a free-text drill label (which may carry interior spaces from the
/// fixture, e.g. `recovery RECALL_RESULT ...`) into a compact identifier.
fn sanitize_drill_label(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
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
    let mut nfr_021_ids: HashMap<String, (DistannDecisionRole, DistannNfr021Admissibility)> =
        HashMap::new();
    for step in &config.steps {
        if !names.insert(step.name()) {
            bail!("duplicate suite step name {:?}", step.name());
        }
        step.validate()?;
        if let SuiteStep::DistannLocalMultinode(step) = step {
            for registration in step.nfr_021_manifest_registrations() {
                if let Some((role, admissibility)) = nfr_021_ids.get(&registration.id) {
                    if *role != registration.role || *admissibility != registration.admissibility {
                        bail!(
                            "NFR-021 registration id {:?} changes role or admissibility across suite steps",
                            registration.id
                        );
                    }
                } else {
                    nfr_021_ids.insert(
                        registration.id,
                        (registration.role, registration.admissibility),
                    );
                }
            }
        }
    }
    Ok(())
}

fn validate_nfr_021_registration(
    step_name: &str,
    variant: Option<&str>,
    registration: &DistannNfr021Registration,
) -> Result<()> {
    let subject = variant
        .map(|variant| format!(" variant {variant:?}"))
        .unwrap_or_default();
    if registration.id.is_empty()
        || !registration
            .id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        bail!(
            "distann-local-multinode step {step_name:?}{subject} NFR-021 id must be a non-empty ASCII identifier"
        )
    }
    if registration.rationale.trim().is_empty() {
        bail!(
            "distann-local-multinode step {step_name:?}{subject} NFR-021 registration requires a rationale"
        )
    }
    if registration.role.is_decision_bearing()
        && registration.admissibility == DistannNfr021Admissibility::Nonconforming
    {
        bail!(
            "distann-local-multinode step {step_name:?}{subject} cannot use an NFR-021-nonconforming {} arm for a decision",
            registration.role.label()
        )
    }
    Ok(())
}

fn validate_reused_fixture_drills(
    step_name: &str,
    reuse_fixture: bool,
    traversal_replica_enospc_drill: bool,
    drop_extension_cleanup_drill: bool,
    materialization_correctness: bool,
) -> Result<()> {
    if reuse_fixture
        && (traversal_replica_enospc_drill
            || drop_extension_cleanup_drill
            || materialization_correctness)
    {
        bail!(
            "distann-local-multinode step {step_name:?} reuse_fixture cannot combine with fixture-mutating drills"
        )
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
            SuiteStep::DistannLocalMultinode(step) => &step.name,
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
            SuiteStep::DistannLocalMultinode(_) => "distann-local-multinode",
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
            SuiteStep::DistannLocalMultinode(step) => &step.tags,
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
            SuiteStep::DistannLocalMultinode(step) => step.pgoptions.as_deref(),
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
                if step.sample_backend_memory.unwrap_or(false)
                    && step.memory_sample_interval_ms == Some(0)
                {
                    bail!(
                        "load step {:?} must set memory_sample_interval_ms >= 1",
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
                if step.concurrency == Some(0) || step.concurrency_sweep.contains(&0) {
                    bail!(
                        "latency step {:?} concurrency values must all be >= 1",
                        step.name
                    )
                }
                if step.concurrency_sweep.iter().collect::<HashSet<_>>().len()
                    != step.concurrency_sweep.len()
                {
                    bail!(
                        "latency step {:?} concurrency_sweep values must be unique",
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
                validate_session_guc_list(
                    "spire-local-multinode load_session_gucs",
                    &step.load_session_gucs,
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
            SuiteStep::DistannLocalMultinode(step) => {
                if step.pg.unwrap_or(18) != 18 {
                    bail!(
                        "distann-local-multinode step {:?} requires pg=18, got {}",
                        step.name,
                        step.pg.unwrap_or(18)
                    )
                }
                if step.nodes == Some(0) {
                    bail!(
                        "distann-local-multinode step {:?} must set nodes >= 1",
                        step.name
                    )
                }
                if step.secure_remote_transport && step.reuse_fixture {
                    bail!(
                        "distann-local-multinode step {:?} cannot combine secure_remote_transport with reuse_fixture",
                        step.name
                    )
                }
                if step.tls_security_matrix
                    && (!step.secure_remote_transport
                        || step.nodes.unwrap_or(3) < 3
                        || step.coordinator_outside_roster
                        || !step.allow_debug_extension)
                {
                    bail!(
                        "distann-local-multinode step {:?} TLS security matrix requires secure_remote_transport, at least three owner nodes, an in-roster coordinator, and allow_debug_extension",
                        step.name
                    )
                }
                if step.tls_security_matrix && (step.physical_benchmark || step.reuse_fixture) {
                    bail!(
                        "distann-local-multinode step {:?} cannot combine tls_security_matrix with physical_benchmark or reuse_fixture",
                        step.name
                    )
                }
                if step.build_shards.is_some_and(|value| value > 4096) {
                    bail!(
                        "distann-local-multinode step {:?} must set build_shards in 0..=4096",
                        step.name
                    )
                }
                if step
                    .head_construction
                    .as_deref()
                    .is_some_and(|value| !matches!(value, "stitched_bfs" | "partition_union"))
                {
                    bail!(
                        "distann-local-multinode step {:?} head_construction must be stitched_bfs or partition_union",
                        step.name
                    )
                }
                if step.top_k == Some(0) {
                    bail!(
                        "distann-local-multinode step {:?} must set top_k >= 1",
                        step.name
                    )
                }
                if step.queries == Some(0) {
                    bail!(
                        "distann-local-multinode step {:?} must set queries >= 1",
                        step.name
                    )
                }
                if step.query_offset.unwrap_or(0) > 0 && step.corpus_prefix.is_none() {
                    bail!(
                        "distann-local-multinode step {:?} query_offset requires corpus_prefix",
                        step.name
                    )
                }
                if let (Some(offset), Some(queries)) = (step.query_offset, step.queries) {
                    offset.checked_add(queries).ok_or_else(|| {
                        eyre!(
                            "distann-local-multinode step {:?} query_offset + queries overflows u32",
                            step.name
                        )
                    })?;
                }
                if step
                    .head_index_cap
                    .is_some_and(|value| !(16..=1_048_576).contains(&value))
                {
                    bail!(
                        "distann-local-multinode step {:?} must set head_index_cap in 16..=1048576",
                        step.name
                    )
                }
                if step.head_sampling_rate.is_none()
                    && (step.head_cap_floor.is_some() || step.head_cap_ceiling.is_some())
                {
                    bail!(
                        "distann-local-multinode step {:?} head bounds require head_sampling_rate",
                        step.name
                    )
                }
                if let Some(rate) = step.head_sampling_rate {
                    if !rate.is_finite() || rate < 0.0 {
                        bail!(
                            "distann-local-multinode step {:?} head_sampling_rate must be finite and non-negative",
                            step.name
                        )
                    }
                    let floor = step.head_cap_floor.unwrap_or(4096);
                    let ceiling = step.head_cap_ceiling.unwrap_or(1_048_576);
                    if !(16..=1_048_576).contains(&floor)
                        || !(16..=1_048_576).contains(&ceiling)
                        || floor > ceiling
                    {
                        bail!(
                            "distann-local-multinode step {:?} head bounds must satisfy 16 <= floor <= ceiling <= 1048576",
                            step.name
                        )
                    }
                }
                if step.crown_capacity.is_some_and(|value| value > 1_048_576) {
                    bail!(
                        "distann-local-multinode step {:?} crown_capacity must be in 0..=1048576",
                        step.name
                    )
                }
                if (step.crown_width_pruning || step.fused_head_hop)
                    && step.crown_capacity.unwrap_or(0) == 0
                {
                    bail!(
                        "distann-local-multinode step {:?} crown features require crown_capacity >= 1",
                        step.name
                    )
                }
                if (step.crown_width_pruning || step.fused_head_hop) && !step.physical_benchmark {
                    bail!(
                        "distann-local-multinode step {:?} crown features require physical_benchmark",
                        step.name
                    )
                }
                if step
                    .beam_width
                    .is_some_and(|value| !(1..=256).contains(&value))
                {
                    bail!(
                        "distann-local-multinode step {:?} must set beam_width in 1..=256",
                        step.name
                    )
                }
                if step
                    .hop_rounds
                    .is_some_and(|value| !(1..=256).contains(&value))
                {
                    bail!(
                        "distann-local-multinode step {:?} must set hop_rounds in 1..=256",
                        step.name
                    )
                }
                if let Some(strategy) = step.seed_strategy.as_deref() {
                    if !matches!(
                        strategy,
                        "persisted_head" | "head_sample_exact" | "head_hierarchy" | "owner_scan"
                    ) {
                        bail!(
                            "distann-local-multinode step {:?} seed_strategy must be persisted_head, head_sample_exact, head_hierarchy, or owner_scan",
                            step.name
                        )
                    }
                    if !step.physical_benchmark {
                        bail!(
                            "distann-local-multinode step {:?} seed_strategy requires physical_benchmark",
                            step.name
                        )
                    }
                }
                if step
                    .head_search_width
                    .is_some_and(|value| !(1..=4096).contains(&value))
                {
                    bail!(
                        "distann-local-multinode step {:?} must set head_search_width in 1..=4096",
                        step.name
                    )
                }
                if step
                    .head_seed_count
                    .is_some_and(|value| !(1..=4096).contains(&value))
                {
                    bail!(
                        "distann-local-multinode step {:?} must set head_seed_count in 1..=4096",
                        step.name
                    )
                }
                if let Some(mode) = step.neighbor_score_mode.as_deref() {
                    if !matches!(mode, "rabitq" | "exact_neighbor") {
                        bail!(
                            "distann-local-multinode step {:?} neighbor_score_mode must be rabitq or exact_neighbor",
                            step.name
                        )
                    }
                    if !step.physical_benchmark {
                        bail!(
                            "distann-local-multinode step {:?} neighbor_score_mode requires physical_benchmark",
                            step.name
                        )
                    }
                }
                if let Some(policy) = step.head_policy.as_deref() {
                    if !matches!(
                        policy,
                        "current_sample"
                            | "geometry_landmarks"
                            | "graph_landmarks"
                            | "training_landmarks"
                            | "training_region_balanced"
                            | "training_query_facility"
                    ) {
                        bail!(
                            "distann-local-multinode step {:?} has invalid head_policy {:?}",
                            step.name,
                            policy
                        )
                    }
                    if policy.starts_with("training_") && step.training_query_path.is_none() {
                        bail!(
                            "distann-local-multinode step {:?} training head policy requires training_query_path",
                            step.name
                        )
                    }
                }
                if let Some(policy) = step.production_head_policy.as_deref() {
                    if !matches!(policy, "current_sample_graph" | "training_landmarks_exact") {
                        bail!(
                            "distann-local-multinode step {:?} has invalid production_head_policy {:?}",
                            step.name,
                            policy
                        )
                    }
                    if policy == "training_landmarks_exact" {
                        if step.training_query_path.is_none() {
                            bail!(
                                "distann-local-multinode step {:?} training_landmarks_exact requires training_query_path",
                                step.name
                            )
                        }
                        if step.head_index_cap.unwrap_or(4096) != 4096 {
                            bail!(
                                "distann-local-multinode step {:?} training_landmarks_exact requires head_index_cap 4096",
                                step.name
                            )
                        }
                    }
                }
                if step.head_policy.is_some() && step.production_head_policy.is_some() {
                    bail!(
                        "distann-local-multinode step {:?} cannot combine head_policy and production_head_policy",
                        step.name
                    )
                }
                let training_path_expected = step
                    .head_policy
                    .as_deref()
                    .is_some_and(|policy| policy.starts_with("training_"))
                    || step.production_head_policy.as_deref() == Some("training_landmarks_exact");
                if step.training_query_path.is_some() != training_path_expected {
                    bail!(
                        "distann-local-multinode step {:?} training_query_path is required exactly for a training head policy",
                        step.name
                    )
                }
                if !step.benchmark_seed_variants.is_empty() {
                    if !step.physical_benchmark {
                        bail!(
                            "distann-local-multinode step {:?} benchmark_seed_variants requires physical_benchmark",
                            step.name
                        )
                    }
                    if step.seed_strategy.is_some()
                        || step.head_search_width.is_some()
                        || step.head_seed_count.is_some()
                        || step.neighbor_score_mode.is_some()
                    {
                        bail!(
                            "distann-local-multinode step {:?} benchmark_seed_variants cannot be combined with singular seed controls",
                            step.name
                        )
                    }
                    let mut variant_names = std::collections::BTreeSet::new();
                    for variant in &step.benchmark_seed_variants {
                        if variant.name.is_empty()
                            || !variant.name.bytes().all(|byte| {
                                byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
                            })
                            || !variant_names.insert(variant.name.as_str())
                        {
                            bail!(
                                "distann-local-multinode step {:?} benchmark seed variant names must be unique ASCII identifiers",
                                step.name
                            )
                        }
                        if !matches!(
                            variant.seed_strategy.as_str(),
                            "persisted_head"
                                | "head_sample_exact"
                                | "head_hierarchy"
                                | "owner_scan"
                        ) {
                            bail!(
                                "distann-local-multinode step {:?} benchmark seed variant {:?} has an invalid strategy",
                                step.name,
                                variant.name
                            )
                        }
                        if !(1..=4096).contains(&variant.head_search_width)
                            || !(1..=4096).contains(&variant.head_seed_count)
                        {
                            bail!(
                                "distann-local-multinode step {:?} benchmark seed variant {:?} widths must be in 1..=4096",
                                step.name,
                                variant.name
                            )
                        }
                        if variant.materialization_batch_size > 4096 {
                            bail!(
                                "distann-local-multinode step {:?} benchmark seed variant {:?} materialization_batch_size must be in 0..=4096",
                                step.name,
                                variant.name
                            )
                        }
                        if variant
                            .beam_width
                            .is_some_and(|value| !(1..=256).contains(&value))
                        {
                            bail!(
                                "distann-local-multinode step {:?} benchmark seed variant {:?} beam_width must be in 1..=256",
                                step.name,
                                variant.name
                            )
                        }
                        if variant
                            .hop_rounds
                            .is_some_and(|value| !(1..=256).contains(&value))
                        {
                            bail!(
                                "distann-local-multinode step {:?} benchmark seed variant {:?} hop_rounds must be in 1..=256",
                                step.name,
                                variant.name
                            )
                        }
                        if !matches!(
                            variant.neighbor_score_mode.as_str(),
                            "rabitq" | "exact_neighbor"
                        ) {
                            bail!(
                                "distann-local-multinode step {:?} benchmark seed variant {:?} has an invalid neighbor score mode",
                                step.name,
                                variant.name
                            )
                        }
                        if variant.traversal_replica
                            && variant.neighbor_score_mode == "exact_neighbor"
                        {
                            bail!(
                                "distann-local-multinode step {:?} benchmark seed variant {:?} cannot combine traversal_replica with exact_neighbor",
                                step.name,
                                variant.name
                            )
                        }
                        if let Some(registration) = &variant.nfr_021 {
                            validate_nfr_021_registration(
                                &step.name,
                                Some(&variant.name),
                                registration,
                            )?;
                            // NFR-021 clause 4 / NFR-022 (Task 210 P1): the
                            // FR-084 traversal replica serves traversal from a
                            // coordinator-resident copy of every owner's graph,
                            // so an arm that enables it can only ever be
                            // context, and can only ever be registered
                            // nonconforming. Rejected before measurement.
                            if variant.traversal_replica {
                                if registration.admissibility
                                    != DistannNfr021Admissibility::Nonconforming
                                {
                                    bail!(
                                        "distann-local-multinode step {:?} benchmark seed variant {:?} enables the FR-084 traversal replica and must be NFR-021-registered as nonconforming",
                                        step.name,
                                        variant.name
                                    )
                                }
                                if registration.role.is_decision_bearing() {
                                    bail!(
                                        "distann-local-multinode step {:?} benchmark seed variant {:?} cannot use the FR-084 traversal replica as a decision-bearing {} arm",
                                        step.name,
                                        variant.name,
                                        registration.role.label()
                                    )
                                }
                            }
                        }
                    }
                    let registered_variants = step
                        .benchmark_seed_variants
                        .iter()
                        .filter(|variant| variant.nfr_021.is_some())
                        .count();
                    if registered_variants != 0
                        && registered_variants != step.benchmark_seed_variants.len()
                    {
                        bail!(
                            "distann-local-multinode step {:?} must NFR-021-register every benchmark_seed_variant when any variant is registered",
                            step.name
                        )
                    }
                }
                if let Some(pair) = step.same_generation_recall_pair.as_deref() {
                    if !step.physical_benchmark || step.benchmark_seed_variants.is_empty() {
                        bail!(
                            "distann-local-multinode step {:?} same_generation_recall_pair requires physical_benchmark and benchmark_seed_variants",
                            step.name
                        )
                    }
                    let (control, candidate) = pair.split_once(',').ok_or_else(|| {
                        eyre!(
                            "distann-local-multinode step {:?} same_generation_recall_pair must be CONTROL,CANDIDATE",
                            step.name
                        )
                    })?;
                    if control.is_empty() || candidate.is_empty() || control == candidate {
                        bail!(
                            "distann-local-multinode step {:?} same_generation_recall_pair must name two distinct variants",
                            step.name
                        )
                    }
                    let names = step
                        .benchmark_seed_variants
                        .iter()
                        .map(|variant| variant.name.as_str())
                        .collect::<HashSet<_>>();
                    if !names.contains(control) || !names.contains(candidate) {
                        bail!(
                            "distann-local-multinode step {:?} same_generation_recall_pair names variants not present in benchmark_seed_variants",
                            step.name
                        )
                    }
                }
                if step.nfr_021.is_some() && !step.benchmark_seed_variants.is_empty() {
                    bail!(
                        "distann-local-multinode step {:?} singular nfr_021 cannot be combined with benchmark_seed_variants",
                        step.name
                    )
                }
                if let Some(registration) = &step.nfr_021 {
                    validate_nfr_021_registration(&step.name, None, registration)?;
                }
                if (step.nfr_021.is_some()
                    || step
                        .benchmark_seed_variants
                        .iter()
                        .any(|variant| variant.nfr_021.is_some()))
                    && !step.physical_benchmark
                {
                    bail!(
                        "distann-local-multinode step {:?} NFR-021 registration requires physical_benchmark",
                        step.name
                    )
                }
                if step.benchmark_iterations == Some(0) {
                    bail!(
                        "distann-local-multinode step {:?} must set benchmark_iterations >= 1",
                        step.name
                    )
                }
                match (
                    step.task167_heldout_baseline_deficit,
                    step.task167_heldout_physical_sample_sd,
                ) {
                    (None, None) => {}
                    (Some(baseline), Some(sample_sd)) => {
                        if !step.physical_benchmark {
                            bail!(
                                "distann-local-multinode step {:?} Task 167 heldout regression gate requires physical_benchmark",
                                step.name
                            )
                        }
                        if !baseline.is_finite() || baseline < 0.0 {
                            bail!(
                                "distann-local-multinode step {:?} Task 167 heldout baseline deficit must be finite and non-negative",
                                step.name
                            )
                        }
                        if !sample_sd.is_finite() || sample_sd < 0.0 {
                            bail!(
                                "distann-local-multinode step {:?} Task 167 heldout physical sample SD must be finite and non-negative",
                                step.name
                            )
                        }
                    }
                    _ => bail!(
                        "distann-local-multinode step {:?} Task 167 heldout regression gate requires both baseline deficit and physical sample SD",
                        step.name
                    ),
                }
                if step
                    .benchmark_concurrency_sweep
                    .iter()
                    .any(|value| *value == 0)
                {
                    bail!(
                        "distann-local-multinode step {:?} benchmark_concurrency_sweep values must all be at least 1",
                        step.name
                    )
                }
                if step
                    .benchmark_concurrency_sweep
                    .iter()
                    .collect::<HashSet<_>>()
                    .len()
                    != step.benchmark_concurrency_sweep.len()
                {
                    bail!(
                        "distann-local-multinode step {:?} benchmark_concurrency_sweep values must be unique",
                        step.name
                    )
                }
                if !step.benchmark_concurrency_sweep.is_empty() && !step.physical_benchmark {
                    bail!(
                        "distann-local-multinode step {:?} benchmark_concurrency_sweep requires physical_benchmark",
                        step.name
                    )
                }
                if step.metrics_mode.is_some() && !step.physical_benchmark {
                    bail!(
                        "distann-local-multinode step {:?} metrics_mode requires physical_benchmark",
                        step.name
                    )
                }
                if step.metrics_mode == Some(DistannMetricsMode::Benchmark)
                    && (step.distann_stage_counters
                        || step.owner_payload_shape.is_some()
                        || step.stage_counter_only
                        || step.sample_backend_memory)
                {
                    bail!(
                        "distann-local-multinode step {:?} benchmark metrics_mode cannot enable full-metrics instrumentation",
                        step.name
                    )
                }
                if (step.sample_backend_memory
                    || step.metrics_mode == Some(DistannMetricsMode::FullMetrics))
                    && step.memory_sample_interval_ms == Some(0)
                {
                    bail!(
                        "distann-local-multinode step {:?} must set memory_sample_interval_ms >= 1",
                        step.name
                    )
                }
                if step.distann_stage_counters && !step.physical_benchmark {
                    bail!(
                        "distann-local-multinode step {:?} distann_stage_counters requires physical_benchmark",
                        step.name
                    )
                }
                if step.owner_payload_shape.is_some()
                    && (!step.physical_benchmark
                        || !step.distann_stage_counters
                        || step.effective_metrics_mode() != DistannMetricsMode::FullMetrics)
                {
                    bail!(
                        "distann-local-multinode step {:?} owner_payload_shape requires physical_benchmark, distann_stage_counters, and full_metrics",
                        step.name
                    )
                }
                if step.owner_payload_shape.as_deref().is_some_and(|shape| {
                    !matches!(
                        shape,
                        "id-only" | "narrow-scalar" | "vector-bearing" | "toasted"
                    )
                }) {
                    bail!(
                        "distann-local-multinode step {:?} owner_payload_shape must be id-only, narrow-scalar, vector-bearing, or toasted",
                        step.name
                    )
                }
                if step.skip_owner_locality_profile && step.owner_payload_shape.is_none() {
                    bail!(
                        "distann-local-multinode step {:?} skip_owner_locality_profile requires owner_payload_shape",
                        step.name
                    )
                }
                if step.owner_fast_real_array_send
                    && (step.owner_payload_shape.as_deref() != Some("vector-bearing")
                        || !step.skip_owner_locality_profile)
                {
                    bail!(
                        "distann-local-multinode step {:?} owner_fast_real_array_send requires vector-bearing owner_payload_shape and skip_owner_locality_profile",
                        step.name
                    )
                }
                if step.gateway_trace && !step.physical_benchmark {
                    bail!(
                        "distann-local-multinode step {:?} gateway_trace requires physical_benchmark",
                        step.name
                    )
                }
                if step.query_trace && !step.physical_benchmark {
                    bail!(
                        "distann-local-multinode step {:?} query_trace requires physical_benchmark",
                        step.name
                    )
                }
                if step.graph_diagnostic && !step.physical_benchmark {
                    bail!(
                        "distann-local-multinode step {:?} graph_diagnostic requires physical_benchmark",
                        step.name
                    )
                }
                if step.graph_diagnostic && step.skip_single_control {
                    bail!(
                        "distann-local-multinode step {:?} graph_diagnostic requires the monolithic control",
                        step.name
                    )
                }
                if step.residual_attribution && (!step.query_trace || !step.graph_diagnostic) {
                    bail!(
                        "distann-local-multinode step {:?} residual_attribution requires query_trace and graph_diagnostic",
                        step.name
                    )
                }
                if step.residual_attribution {
                    if step.skip_recall || step.stage_counter_only {
                        bail!(
                            "distann-local-multinode step {:?} residual_attribution requires per-variant recall predictions",
                            step.name
                        )
                    }
                    if step.corpus_prefix.as_deref() != Some("ec_real_100k")
                        || step.coordinator_outside_roster
                        || step.queries.unwrap_or(200) != 200
                        || !matches!(step.query_offset.unwrap_or(0), 0 | 200)
                        || step.top_k.unwrap_or(10) != 10
                        || step.head_index_cap.unwrap_or(4096) != 4096
                        || step.candidate_heap_limit.unwrap_or(32) != 32
                    {
                        bail!(
                            "distann-local-multinode step {:?} residual_attribution requires the frozen 100k, q200, offset 0/200, k10, head4096, L32 contract",
                            step.name
                        )
                    }
                    let required = [
                        ("prod-bw4-rabitq", "persisted_head", 4, "rabitq"),
                        ("task226-bw8-rabitq", "persisted_head", 8, "rabitq"),
                        (
                            "prod-bw4-exact-neighbor",
                            "persisted_head",
                            4,
                            "exact_neighbor",
                        ),
                        ("owner-bw4-rabitq", "owner_scan", 4, "rabitq"),
                        (
                            "owner-bw4-exact-neighbor",
                            "owner_scan",
                            4,
                            "exact_neighbor",
                        ),
                    ];
                    for (name, strategy, beam, score) in required {
                        let variant = step
                            .benchmark_seed_variants
                            .iter()
                            .find(|variant| variant.name == name)
                            .ok_or_else(|| {
                                eyre!(
                                    "distann-local-multinode step {:?} residual_attribution is missing registered variant {name}",
                                    step.name
                                )
                            })?;
                        if variant.seed_strategy != strategy
                            || variant.beam_width.unwrap_or(step.beam_width.unwrap_or(4)) != beam
                            || variant.hop_rounds.unwrap_or(step.hop_rounds.unwrap_or(100)) != 100
                            || variant.neighbor_score_mode != score
                            || variant.head_search_width != 32
                            || variant.head_seed_count != 32
                            || variant.materialization_batch_size != 10
                            || variant.owner_payload_plan_cache.unwrap_or(false)
                            || variant.traversal_replica
                            || variant.typed_locator
                            || variant.packed_payload
                            || variant.expanded_locator
                        {
                            bail!(
                                "distann-local-multinode step {:?} residual_attribution variant {name} violates the frozen registered shape",
                                step.name
                            )
                        }
                    }
                }
                if step.gateway_isolated_trace && !step.physical_benchmark {
                    bail!(
                        "distann-local-multinode step {:?} gateway_isolated_trace requires physical_benchmark",
                        step.name
                    )
                }
                if step.gateway_head_candidate_trace && !step.physical_benchmark {
                    bail!(
                        "distann-local-multinode step {:?} gateway_head_candidate_trace requires physical_benchmark",
                        step.name
                    )
                }
                if step.gateway_trace && step.training_query_path.is_none() {
                    bail!(
                        "distann-local-multinode step {:?} gateway_trace requires training_query_path for disjoint attribution",
                        step.name
                    )
                }
                if step.gateway_isolated_trace && step.training_query_path.is_none() {
                    bail!(
                        "distann-local-multinode step {:?} gateway_isolated_trace requires training_query_path for disjoint attribution",
                        step.name
                    )
                }
                if step.gateway_head_candidate_trace && step.training_query_path.is_none() {
                    bail!(
                        "distann-local-multinode step {:?} gateway_head_candidate_trace requires training_query_path for disjoint attribution",
                        step.name
                    )
                }
                if step.gateway_head_candidate_trace
                    && step.gateway_head_candidate_positions.is_empty()
                {
                    bail!(
                        "distann-local-multinode step {:?} gateway_head_candidate_trace requires gateway_head_candidate_positions",
                        step.name
                    )
                }
                if step
                    .gateway_head_candidate_positions
                    .iter()
                    .any(|position| !(1..=4096).contains(position))
                {
                    bail!(
                        "distann-local-multinode step {:?} gateway_head_candidate_positions must be in 1..=4096",
                        step.name
                    )
                }
                if !step.gateway_head_candidate_trace
                    && !step.gateway_head_candidate_positions.is_empty()
                {
                    bail!(
                        "distann-local-multinode step {:?} gateway_head_candidate_positions requires gateway_head_candidate_trace",
                        step.name
                    )
                }
                if let Some(limit) = step.gateway_isolated_seed_limit {
                    if !(1..=4096).contains(&limit) {
                        bail!(
                            "distann-local-multinode step {:?} gateway_isolated_seed_limit must be in 1..=4096",
                            step.name
                        )
                    }
                    if !step.gateway_isolated_trace {
                        bail!(
                            "distann-local-multinode step {:?} gateway_isolated_seed_limit requires gateway_isolated_trace",
                            step.name
                        )
                    }
                }
                if step.stage_counter_only
                    && (!step.physical_benchmark || !step.distann_stage_counters)
                {
                    bail!(
                        "distann-local-multinode step {:?} stage_counter_only requires physical_benchmark and distann_stage_counters",
                        step.name
                    )
                }
                if step.stage_counter_only && step.materialization_correctness {
                    bail!(
                        "distann-local-multinode step {:?} stage_counter_only cannot combine with materialization_correctness",
                        step.name
                    )
                }
                validate_reused_fixture_drills(
                    &step.name,
                    step.reuse_fixture,
                    step.traversal_replica_enospc_drill,
                    step.drop_extension_cleanup_drill,
                    step.materialization_correctness,
                )?;
                if step.skip_routed_delete_vacuum_drill && !step.physical_benchmark {
                    bail!(
                        "distann-local-multinode step {:?} skip_routed_delete_vacuum_drill requires physical_benchmark",
                        step.name
                    )
                }
                if step.materialization_correctness {
                    if !step.physical_benchmark {
                        bail!(
                            "distann-local-multinode step {:?} materialization_correctness requires physical_benchmark",
                            step.name
                        )
                    }
                    let effective_beam_width = |variant: &DistannBenchmarkSeedVariant| {
                        variant.beam_width.or(step.beam_width).unwrap_or(4)
                    };
                    let effective_hop_rounds = |variant: &DistannBenchmarkSeedVariant| {
                        variant.hop_rounds.or(step.hop_rounds).unwrap_or(100)
                    };
                    let same_search =
                        |left: &DistannBenchmarkSeedVariant,
                         right: &DistannBenchmarkSeedVariant| {
                            left.seed_strategy == right.seed_strategy
                                && left.head_search_width == right.head_search_width
                                && left.head_seed_count == right.head_seed_count
                                && left.neighbor_score_mode == right.neighbor_score_mode
                                && effective_beam_width(left) == effective_beam_width(right)
                                && effective_hop_rounds(left) == effective_hop_rounds(right)
                                && left.traversal_replica == right.traversal_replica
                        };
                    let has_plan_pair = step.benchmark_seed_variants.iter().any(|control| {
                        control.owner_payload_plan_cache != Some(true)
                            && step.benchmark_seed_variants.iter().any(|candidate| {
                                candidate.owner_payload_plan_cache == Some(true)
                                    && candidate.materialization_batch_size
                                        == control.materialization_batch_size
                                    && candidate.typed_locator == control.typed_locator
                                    && candidate.packed_payload == control.packed_payload
                                    && candidate.expanded_locator == control.expanded_locator
                                    && candidate.payload_projection == control.payload_projection
                                    && same_search(control, candidate)
                            })
                    });
                    let has_batch_pair = step.benchmark_seed_variants.iter().any(|control| {
                        control.materialization_batch_size == 0
                            && step.benchmark_seed_variants.iter().any(|candidate| {
                                candidate.materialization_batch_size == 10
                                    && candidate.owner_payload_plan_cache
                                        == control.owner_payload_plan_cache
                                    && candidate.typed_locator == control.typed_locator
                                    && candidate.packed_payload == control.packed_payload
                                    && candidate.expanded_locator == control.expanded_locator
                                    && candidate.payload_projection == control.payload_projection
                                    && same_search(control, candidate)
                            })
                    });
                    let has_traversal_pair = step.benchmark_seed_variants.iter().any(|control| {
                        !control.traversal_replica
                            && step.benchmark_seed_variants.iter().any(|candidate| {
                                candidate.traversal_replica
                                    && candidate.materialization_batch_size
                                        == control.materialization_batch_size
                                    && candidate.owner_payload_plan_cache
                                        == control.owner_payload_plan_cache
                                    && candidate.typed_locator == control.typed_locator
                                    && candidate.packed_payload == control.packed_payload
                                    && candidate.expanded_locator == control.expanded_locator
                                    && candidate.payload_projection == control.payload_projection
                                    && candidate.seed_strategy == control.seed_strategy
                                    && candidate.head_search_width == control.head_search_width
                                    && candidate.head_seed_count == control.head_seed_count
                                    && candidate.neighbor_score_mode == control.neighbor_score_mode
                                    && effective_beam_width(candidate)
                                        == effective_beam_width(control)
                                    && effective_hop_rounds(candidate)
                                        == effective_hop_rounds(control)
                            })
                    });
                    let has_packed_payload_pair =
                        step.benchmark_seed_variants.iter().any(|control| {
                            !control.packed_payload
                                && step.benchmark_seed_variants.iter().any(|candidate| {
                                    candidate.packed_payload
                                        && candidate.materialization_batch_size
                                            == control.materialization_batch_size
                                        && candidate.owner_payload_plan_cache
                                            == control.owner_payload_plan_cache
                                        && candidate.typed_locator == control.typed_locator
                                        && candidate.expanded_locator == control.expanded_locator
                                        && candidate.payload_projection
                                            == control.payload_projection
                                        && candidate.traversal_replica == control.traversal_replica
                                        && same_search(control, candidate)
                                })
                        });
                    let has_expanded_locator_pair =
                        step.benchmark_seed_variants.iter().any(|control| {
                            !control.expanded_locator
                                && step.benchmark_seed_variants.iter().any(|candidate| {
                                    candidate.expanded_locator
                                        && candidate.materialization_batch_size
                                            == control.materialization_batch_size
                                        && candidate.owner_payload_plan_cache
                                            == control.owner_payload_plan_cache
                                        && candidate.typed_locator == control.typed_locator
                                        && candidate.packed_payload == control.packed_payload
                                        && candidate.traversal_replica == control.traversal_replica
                                        && same_search(control, candidate)
                                })
                        });
                    let has_payload_projection_pair =
                        step.benchmark_seed_variants.iter().any(|control| {
                            !control.payload_projection
                                && step.benchmark_seed_variants.iter().any(|candidate| {
                                    candidate.payload_projection
                                        && candidate.materialization_batch_size
                                            == control.materialization_batch_size
                                        && candidate.owner_payload_plan_cache
                                            == control.owner_payload_plan_cache
                                        && candidate.typed_locator == control.typed_locator
                                        && candidate.packed_payload == control.packed_payload
                                        && candidate.expanded_locator == control.expanded_locator
                                        && candidate.traversal_replica == control.traversal_replica
                                        && same_search(control, candidate)
                                })
                        });
                    if !has_plan_pair
                        && !has_batch_pair
                        && !has_traversal_pair
                        && !has_packed_payload_pair
                        && !has_expanded_locator_pair
                        && !has_payload_projection_pair
                    {
                        bail!(
                            "distann-local-multinode step {:?} materialization_correctness requires an isolated owner-plan, eager/lazy10, owner/replica, packed-payload, expanded-locator, or payload-projection pair",
                            step.name
                        )
                    }
                    if step.coordinator_outside_roster {
                        bail!(
                            "distann-local-multinode step {:?} materialization_correctness requires coordinator owner zero",
                            step.name
                        )
                    }
                }
                if step.physical_benchmark && step.corpus_prefix.is_none() {
                    bail!(
                        "distann-local-multinode step {:?} physical_benchmark requires corpus_prefix",
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
            SuiteStep::DistannLocalMultinode(step) => {
                Ok(expand_distann_local_multinode(step, defaults))
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
            SuiteStep::DistannLocalMultinode(step) => {
                if step.tls_security_matrix {
                    let mut artifacts: Vec<PathBuf> = step.log_file.iter().cloned().collect();
                    if let Some(dir) = &step.artifact_dir {
                        artifacts.push(dir.join("task236-tls-security-matrix.log"));
                    }
                    return artifacts;
                }
                if step.compact_artifacts {
                    return step
                        .artifact_dir
                        .iter()
                        .flat_map(|dir| {
                            let mut artifacts = vec![dir.join("distann-multinode-summary.log")];
                            if step.physical_benchmark {
                                artifacts.push(dir.join("physical-head-membership.json"));
                            }
                            if step.gateway_trace {
                                let variants = if step.benchmark_seed_variants.is_empty() {
                                    vec!["production".to_owned()]
                                } else {
                                    step.benchmark_seed_variants
                                        .iter()
                                        .map(|variant| variant.name.clone())
                                        .collect::<Vec<_>>()
                                };
                                artifacts.extend(variants.into_iter().map(|variant| {
                                    dir.join(format!("physical-{variant}-gateway-trace.json"))
                                }));
                            }
                            if step.query_trace {
                                let variants = if step.benchmark_seed_variants.is_empty() {
                                    vec!["production".to_owned()]
                                } else {
                                    step.benchmark_seed_variants
                                        .iter()
                                        .map(|variant| variant.name.clone())
                                        .collect::<Vec<_>>()
                                };
                                artifacts.extend(variants.into_iter().map(|variant| {
                                    dir.join(format!("physical-{variant}-query-trace.json"))
                                }));
                            }
                            if step.graph_diagnostic {
                                artifacts.push(dir.join("physical-graph-diagnostic.json"));
                            }
                            if step.residual_attribution {
                                artifacts.push(dir.join("physical-residual-attribution.jsonl"));
                                artifacts.push(dir.join("physical-residual-query-features.jsonl"));
                                artifacts
                                    .push(dir.join("physical-residual-attribution-summary.json"));
                            }
                            if step.gateway_isolated_trace {
                                let variants = if step.benchmark_seed_variants.is_empty() {
                                    vec!["production".to_owned()]
                                } else {
                                    step.benchmark_seed_variants
                                        .iter()
                                        .map(|variant| variant.name.clone())
                                        .collect::<Vec<_>>()
                                };
                                artifacts.extend(variants.into_iter().map(|variant| {
                                    dir.join(format!(
                                        "physical-{variant}-gateway-isolated-trace.json"
                                    ))
                                }));
                            }
                            if step.gateway_head_candidate_trace {
                                let variants = if step.benchmark_seed_variants.is_empty() {
                                    vec!["production".to_owned()]
                                } else {
                                    step.benchmark_seed_variants
                                        .iter()
                                        .map(|variant| variant.name.clone())
                                        .collect::<Vec<_>>()
                                };
                                artifacts.extend(variants.into_iter().map(|variant| {
                                    dir.join(format!(
                                        "physical-{variant}-gateway-head-candidate-trace.json"
                                    ))
                                }));
                            }
                            artifacts
                        })
                        .collect();
                }
                let mut artifacts: Vec<PathBuf> = step.log_file.iter().cloned().collect();
                if let Some(dir) = &step.artifact_dir {
                    artifacts.push(dir.join("distann-multinode-summary.log"));
                    if step.physical_benchmark {
                        artifacts.extend([
                            dir.join("physical-recall.log"),
                            dir.join("physical-latency.log"),
                            dir.join("single-recall.log"),
                            dir.join("single-latency.log"),
                        ]);
                    }
                    if step.gateway_trace {
                        let variants = if step.benchmark_seed_variants.is_empty() {
                            vec!["production".to_owned()]
                        } else {
                            step.benchmark_seed_variants
                                .iter()
                                .map(|variant| variant.name.clone())
                                .collect::<Vec<_>>()
                        };
                        artifacts.extend(variants.into_iter().map(|variant| {
                            dir.join(format!("physical-{variant}-gateway-trace.json"))
                        }));
                    }
                    if step.query_trace {
                        let variants = if step.benchmark_seed_variants.is_empty() {
                            vec!["production".to_owned()]
                        } else {
                            step.benchmark_seed_variants
                                .iter()
                                .map(|variant| variant.name.clone())
                                .collect::<Vec<_>>()
                        };
                        artifacts.extend(variants.into_iter().map(|variant| {
                            dir.join(format!("physical-{variant}-query-trace.json"))
                        }));
                    }
                    if step.gateway_isolated_trace {
                        let variants = if step.benchmark_seed_variants.is_empty() {
                            vec!["production".to_owned()]
                        } else {
                            step.benchmark_seed_variants
                                .iter()
                                .map(|variant| variant.name.clone())
                                .collect::<Vec<_>>()
                        };
                        artifacts.extend(variants.into_iter().map(|variant| {
                            dir.join(format!("physical-{variant}-gateway-isolated-trace.json"))
                        }));
                    }
                    if step.gateway_head_candidate_trace {
                        let variants = if step.benchmark_seed_variants.is_empty() {
                            vec!["production".to_owned()]
                        } else {
                            step.benchmark_seed_variants
                                .iter()
                                .map(|variant| variant.name.clone())
                                .collect::<Vec<_>>()
                        };
                        artifacts.extend(variants.into_iter().map(|variant| {
                            dir.join(format!(
                                "physical-{variant}-gateway-head-candidate-trace.json"
                            ))
                        }));
                    }
                    if step.graph_diagnostic {
                        artifacts.push(dir.join("physical-graph-diagnostic.json"));
                    }
                    if step.residual_attribution {
                        artifacts.push(dir.join("physical-residual-attribution.jsonl"));
                        artifacts.push(dir.join("physical-residual-query-features.jsonl"));
                        artifacts.push(dir.join("physical-residual-attribution-summary.json"));
                    }
                }
                artifacts
            }
            SuiteStep::SpireLocalMultinode(step) => {
                let mut artifacts: Vec<PathBuf> = step.smoke_log.iter().cloned().collect();
                if let Some(run_dir) = &step.run_dir {
                    artifacts.push(run_dir.join("topology.local.json"));
                } else if let Some(run_id) = &step.run_id {
                    artifacts.push(
                        crate::commands::dev::default_cluster_root()
                            .join(format!("spire-local-multinode-{run_id}"))
                            .join("topology.local.json"),
                    );
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
                .chain(step.result_identity_output.iter())
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

fn validate_session_guc_list(label: &str, gucs: &[String]) -> Result<()> {
    for guc in gucs {
        if guc.trim().is_empty() {
            bail!("{label} must not include empty GUCs");
        }
        if guc.contains(';') {
            bail!("{label} item {:?} must not contain ';'", guc);
        }
        if !guc.contains('=') {
            bail!("{label} item {:?} must use name=value syntax", guc);
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
    if step
        .sample_backend_memory
        .or(defaults.sample_backend_memory)
        .unwrap_or(false)
    {
        args.push("--sample-backend-memory".into());
        push_arg(
            &mut args,
            "--memory-sample-interval-ms",
            &step
                .memory_sample_interval_ms
                .or(defaults.memory_sample_interval_ms)
                .unwrap_or(25)
                .to_string(),
        );
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
    if !step.concurrency_sweep.is_empty() {
        push_arg(
            &mut args,
            "--concurrency-sweep",
            &join_usize(&step.concurrency_sweep),
        );
    }
    push_arg(
        &mut args,
        "--iterations",
        &step
            .iterations
            .or(defaults.iterations)
            .unwrap_or(1000)
            .to_string(),
    );
    push_opt_arg(
        &mut args,
        "--worker-batch-size",
        step.worker_batch_size
            .or(defaults.worker_batch_size)
            .map(|v| v.to_string())
            .as_deref(),
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
    if step.ivf_stage_counters.unwrap_or(false) {
        args.push("--ivf-stage-counters".into());
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
    for guc in &step.load_session_gucs {
        push_arg(&mut args, "--load-session-guc", guc);
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

fn expand_distann_local_multinode(
    step: &DistannLocalMultinodeStep,
    defaults: &SuiteDefaults,
) -> Vec<String> {
    let explicit_full_metrics = step.metrics_mode == Some(DistannMetricsMode::FullMetrics);
    let mut args = vec![
        "dev".into(),
        "distann-multicluster".into(),
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
    push_opt_path(
        &mut args,
        "--reuse-provenance-dir",
        step.reuse_provenance_dir.as_deref(),
    );
    push_opt_path(&mut args, "--log-file", step.log_file.as_deref());
    push_opt_arg(
        &mut args,
        "--nodes",
        step.nodes.map(|v| v.to_string()).as_deref(),
    );
    if step.coordinator_outside_roster {
        args.push("--coordinator-outside-roster".into());
    }
    if step.allow_debug_extension {
        args.push("--allow-debug-extension".into());
    }
    if step.secure_remote_transport {
        args.push("--secure-remote-transport".into());
    }
    if step.tls_security_matrix {
        args.push("--tls-security-matrix".into());
    }
    if step.physical_benchmark {
        args.push("--physical-benchmark".into());
    }
    if step.gateway_trace {
        args.push("--gateway-trace".into());
    }
    if step.query_trace {
        args.push("--query-trace".into());
    }
    if step.graph_diagnostic {
        args.push("--graph-diagnostic".into());
    }
    if step.residual_attribution {
        args.push("--residual-attribution".into());
    }
    if step.gateway_isolated_trace {
        args.push("--gateway-isolated-trace".into());
    }
    if step.gateway_head_candidate_trace {
        args.push("--gateway-head-candidate-trace".into());
    }
    if !step.gateway_head_candidate_positions.is_empty() {
        args.push("--gateway-head-candidate-positions".into());
        args.push(
            step.gateway_head_candidate_positions
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    push_opt_arg(
        &mut args,
        "--gateway-isolated-seed-limit",
        step.gateway_isolated_seed_limit
            .map(|v| v.to_string())
            .as_deref(),
    );
    if step.distann_stage_counters || explicit_full_metrics {
        args.push("--distann-stage-counters".into());
    }
    if let Some(shape) = step.owner_payload_shape.as_deref() {
        args.extend(["--owner-payload-shape".into(), shape.to_owned()]);
    }
    if step.skip_owner_locality_profile {
        args.push("--skip-owner-locality-profile".into());
    }
    if step.owner_fast_real_array_send {
        args.push("--owner-fast-real-array-send".into());
    }
    if step.stage_counter_only {
        args.push("--stage-counter-only".into());
    }
    if step.skip_recall {
        args.push("--skip-recall".into());
    }
    if step.skip_single_control {
        args.push("--skip-single-control".into());
    }
    if step.skip_single_benchmark {
        args.push("--skip-single-benchmark".into());
    }
    if step.skip_concurrency_drill {
        args.push("--skip-concurrency-drill".into());
    }
    if step.skip_routed_delete_vacuum_drill {
        args.push("--skip-routed-delete-vacuum-drill".into());
    }
    if step.materialization_correctness {
        args.push("--materialization-correctness".into());
    }
    if step.traversal_replica_enospc_drill {
        args.push("--traversal-replica-enospc-drill".into());
    }
    push_opt_arg(
        &mut args,
        "--benchmark-iterations",
        step.benchmark_iterations.map(|v| v.to_string()).as_deref(),
    );
    if !step.benchmark_concurrency_sweep.is_empty() {
        push_arg(
            &mut args,
            "--benchmark-concurrency-sweep",
            &join_usize(&step.benchmark_concurrency_sweep),
        );
    }
    push_opt_arg(
        &mut args,
        "--benchmark-warmup-iterations",
        step.benchmark_warmup_iterations
            .map(|v| v.to_string())
            .as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--benchmark-parity-queries",
        step.benchmark_parity_queries
            .map(|v| v.to_string())
            .as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--task167-heldout-baseline-deficit",
        step.task167_heldout_baseline_deficit
            .map(|v| v.to_string())
            .as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--task167-heldout-physical-sample-sd",
        step.task167_heldout_physical_sample_sd
            .map(|v| v.to_string())
            .as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--benchmark-backend-batch-size",
        step.benchmark_backend_batch_size
            .map(|v| v.to_string())
            .as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--coverage-memory-regression-iterations",
        step.coverage_memory_regression_iterations
            .map(|v| v.to_string())
            .as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--coverage-memory-regression-max-slope-kb-per-s",
        step.coverage_memory_regression_max_slope_kb_per_s
            .map(|v| v.to_string())
            .as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--coverage-memory-regression-max-delta-kb",
        step.coverage_memory_regression_max_delta_kb
            .map(|v| v.to_string())
            .as_deref(),
    );
    if step.sample_backend_memory || explicit_full_metrics {
        args.push("--sample-backend-memory".into());
        push_arg(
            &mut args,
            "--memory-sample-interval-ms",
            &step.memory_sample_interval_ms.unwrap_or(25).to_string(),
        );
    }
    push_opt_u16(&mut args, "--base-port", step.base_port);
    push_opt_arg(
        &mut args,
        "--rows",
        step.rows.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--dim",
        step.dim.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--graph-degree",
        step.graph_degree.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--build-shards",
        step.build_shards.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--head-construction",
        step.head_construction.as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--head-index-cap",
        step.head_index_cap.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--head-sampling-rate",
        step.head_sampling_rate.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--head-cap-floor",
        step.head_cap_floor.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--head-cap-ceiling",
        step.head_cap_ceiling.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--beam-width",
        step.beam_width.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--candidate-heap-limit",
        step.candidate_heap_limit.map(|v| v.to_string()).as_deref(),
    );
    if step.sharded_head {
        args.push("--sharded-head".into());
    }
    push_opt_arg(
        &mut args,
        "--head-replica-count",
        step.head_replica_count.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--gateway-copy-capacity",
        step.gateway_copy_capacity.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--crown-capacity",
        step.crown_capacity.map(|v| v.to_string()).as_deref(),
    );
    if step.crown_width_pruning {
        args.push("--crown-width-pruning".into());
    }
    if step.fused_head_hop {
        args.push("--fused-head-hop".into());
    }
    if step.local_head {
        args.push("--local-head".into());
    }
    push_opt_arg(
        &mut args,
        "--hop-rounds",
        step.hop_rounds.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(&mut args, "--seed-strategy", step.seed_strategy.as_deref());
    push_opt_arg(
        &mut args,
        "--head-search-width",
        step.head_search_width.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--head-seed-count",
        step.head_seed_count.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--neighbor-score-mode",
        step.neighbor_score_mode.as_deref(),
    );
    push_opt_arg(&mut args, "--head-policy", step.head_policy.as_deref());
    push_opt_arg(
        &mut args,
        "--production-head-policy",
        step.production_head_policy.as_deref(),
    );
    push_opt_path(
        &mut args,
        "--training-query-path",
        step.training_query_path.as_deref(),
    );
    if step.reuse_fixture {
        args.push("--reuse-fixture".into());
    }
    for variant in &step.benchmark_seed_variants {
        let encoded = format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            variant.name,
            variant.seed_strategy,
            variant.head_search_width,
            variant.head_seed_count,
            variant.neighbor_score_mode,
            variant.materialization_batch_size,
            if variant.owner_payload_plan_cache.unwrap_or(false) {
                "on"
            } else {
                "off"
            },
            variant.beam_width.or(step.beam_width).unwrap_or(4),
            variant.hop_rounds.or(step.hop_rounds).unwrap_or(100),
            if variant.traversal_replica {
                "on"
            } else {
                "off"
            },
            if variant.typed_locator { "on" } else { "off" },
            if variant.packed_payload { "on" } else { "off" },
            if variant.expanded_locator {
                "on"
            } else {
                "off"
            },
            if variant.payload_projection {
                "on"
            } else {
                "off"
            },
        );
        push_arg(&mut args, "--benchmark-seed-variant", &encoded);
    }
    push_opt_arg(
        &mut args,
        "--same-generation-recall-pair",
        step.same_generation_recall_pair.as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--queries",
        step.queries.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--query-offset",
        step.query_offset.map(|v| v.to_string()).as_deref(),
    );
    push_opt_arg(
        &mut args,
        "--top-k",
        step.top_k.map(|v| v.to_string()).as_deref(),
    );
    if step.skip_fault_drills {
        args.push("--skip-fault-drills".into());
    }
    if step.drop_extension_cleanup_drill {
        args.push("--drop-extension-cleanup-drill".into());
    }
    push_opt_arg(&mut args, "--corpus-prefix", step.corpus_prefix.as_deref());
    push_opt_path(&mut args, "--staged-dir", step.staged_dir.as_deref());
    for guc in &step.bench_session_gucs {
        push_arg(&mut args, "--bench-session-guc", guc);
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
    push_opt_path(
        &mut args,
        "--result-identity-output",
        step.result_identity_output.as_deref(),
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

fn join_usize(values: &[usize]) -> String {
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
    let git_commit = query_backend_git_sha(&client).await;
    let backend_path = derive_local_pgrx_backend_path(&client).await?;
    let sha256 = match &backend_path {
        Some(path) => Some(sha256_file_hex(path).await?),
        None => None,
    };
    Ok(BackendPreflight {
        build_profile,
        git_commit,
        sha256,
        path: backend_path.map(|path| path.display().to_string()),
    })
}

/// Tolerates extensions predating `ecaz_build_git_sha()`; provenance is then
/// recorded as absent rather than failing the preflight.
async fn query_backend_git_sha(client: &tokio_postgres::Client) -> Option<String> {
    client
        .query_one("SELECT ecaz_build_git_sha()", &[])
        .await
        .ok()
        .map(|row| row.get::<_, String>(0))
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
    use clap::Parser;

    use super::*;

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
    fn raw_suite_result_rows_are_structured() {
        let raw = "\
psql header noise\n\
[suite-result] dml_gate_latency lane=control trial=1 us_per_statement=3.125\n\
[suite-result] dml_gate_latency lane=installed trial=1 us_per_statement=17.750\n\
[suite-result] invalid/metric lane=ignored\n\
[suite-result] missing_fields\n";
        let rows = parse_raw_result_rows(raw);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "dml_gate_latency");
        assert_eq!(rows[0].1.get("lane").map(String::as_str), Some("control"));
        assert_eq!(
            rows[1].1.get("us_per_statement").map(String::as_str),
            Some("17.750")
        );
    }

    #[test]
    fn distann_multinode_rows_parse_recall_identity_gate_and_drills() {
        // 027-P1: the `distann-local-multinode` step used to emit an empty
        // results.jsonl because parse_result_rows had no arm. This pins the
        // three decision-grade shapes the fixture emits.
        let raw = "\
[distann-multicluster] node 1 loaded + indexed\n\
[distann-multicluster] release_profile_preflight status=passed nodes=3 unanimous=true extension_git_sha=abc123 extension_build_profile=release debug_override=false\n\
[distann-multicluster] RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0\n\
[distann-multicluster] suite_recall_gate single=0.9950 multi=0.9950 delta=0.0000 pass=true\n\
[distann-multicluster] qual_correctness mismatched_ids=0 pass=true\n\
[distann-multicluster] fault_drill remote_statement_timeout pass=true\n\
[distann-multicluster] suite_recall_gate=SKIPPED(no exe)\n";
        let rows = parse_distann_multinode_rows(raw);

        let preflight = rows
            .iter()
            .find(|(m, _)| m == "multinode_release_preflight")
            .expect("release preflight row");
        assert_eq!(
            preflight
                .1
                .get("extension_build_profile")
                .map(String::as_str),
            Some("release")
        );
        assert_eq!(
            preflight.1.get("pass_numeric").map(String::as_str),
            Some("1")
        );

        let identity = rows
            .iter()
            .find(|(m, _)| m == "distinct_recall_identity")
            .expect("recall identity row");
        assert_eq!(identity.1.get("n_queries").map(String::as_str), Some("50"));
        assert_eq!(
            identity.1.get("mismatched_ids").map(String::as_str),
            Some("0")
        );
        assert_eq!(
            identity.1.get("identity_ok").map(String::as_str),
            Some("true")
        );

        let gate = rows
            .iter()
            .find(|(m, _)| m == "suite_recall_gate")
            .expect("suite recall gate row");
        assert_eq!(gate.1.get("single").map(String::as_str), Some("0.9950"));
        assert_eq!(gate.1.get("multi").map(String::as_str), Some("0.9950"));
        assert_eq!(gate.1.get("pass").map(String::as_str), Some("true"));

        // The SKIPPED gate line (no `single=`) must NOT produce a gate row.
        assert_eq!(
            rows.iter()
                .filter(|(m, _)| m == "suite_recall_gate")
                .count(),
            1,
            "only the measured gate line yields a row"
        );

        let drills: Vec<&String> = rows
            .iter()
            .filter(|(m, _)| m == "drill_outcome")
            .filter_map(|(_, v)| v.get("drill"))
            .collect();
        assert!(
            drills
                .iter()
                .any(|d| d.contains("fault_drill_remote_statement_timeout")),
            "fault drill outcome captured: {drills:?}"
        );
        assert!(
            drills.iter().any(|d| d.contains("qual_correctness")),
            "qual drill outcome captured: {drills:?}"
        );
    }

    #[test]
    fn distann_multinode_recall_mismatch_sets_identity_not_ok() {
        let raw =
            "[distann-multicluster] RECALL_RESULT n_queries=50 identical=48 mismatched_ids=4\n";
        let rows = parse_distann_multinode_rows(raw);
        let identity = rows
            .iter()
            .find(|(m, _)| m == "distinct_recall_identity")
            .expect("recall identity row");
        assert_eq!(
            identity.1.get("identity_ok").map(String::as_str),
            Some("false"),
            "a nonzero mismatch fails the identity threshold"
        );
    }

    #[test]
    fn distann_task167_quality_and_insert_metrics_are_structured() {
        let raw = "\
[distann-multicluster] physical_benchmark_recall_instrument_calibration scale=50k ordinary_distinct_recall=0.954500 exact_scorer_distinct_recall=0.954500 absolute_delta=0.000000 pass=true\n\
[distann-multicluster] physical_benchmark_insert_throughput_ab scale=50k physical_insert_mode=shipped_default_established_tie_priority physical_rows_per_second=0.224 control_rows_per_second=2.000 pass=true\n\
[distann-multicluster] physical_benchmark_append_when_room_ab scale=50k append_enabled_over_disabled=1.003392 pass=true\n\
[distann-multicluster] physical_benchmark_backlink_strategy_ab scale=50k robust_prune_all_over_shipped=1.003392 pass=true\n\
[distann-multicluster] physical_benchmark_insert_work scale=50k metric=backlink_amendments inserts=160 value=5120 pass=true\n\
   0: \u{1b}[91mTask 167 failed: physical_benchmark_post_insert_exact_recall scale=50k population=heldout physical_distinct_recall=0.848722 fresh_distinct_recall=0.857333 physical_minus_fresh=-0.008611 allowed_deficit=0.007000 quality_gate_pass=false pass=false\u{1b}[0m\n";
        let rows = parse_distann_multinode_rows(raw);

        for metric in [
            "physical_benchmark_recall_instrument_calibration",
            "physical_benchmark_insert_throughput_ab",
            "physical_benchmark_append_when_room_ab",
            "physical_benchmark_backlink_strategy_ab",
            "physical_benchmark_insert_work",
            "physical_benchmark_post_insert_exact_recall",
        ] {
            assert!(
                rows.iter().any(|(candidate, _)| candidate == metric),
                "missing Task 167 metric {metric}: {rows:?}"
            );
        }
        let exact = rows
            .iter()
            .find(|(metric, _)| metric == "physical_benchmark_post_insert_exact_recall")
            .expect("exact-recall row");
        assert_eq!(
            exact.1.get("quality_gate_pass").map(String::as_str),
            Some("false")
        );
        assert_eq!(exact.1.get("pass").map(String::as_str), Some("false"));
    }

    #[test]
    fn distann_same_generation_attestations_are_structured() {
        let raw = "[distann-multicluster] physical_benchmark_generation scale=100k variant=control arm=physical generation_identity=abcd generation_identity_kind=epoch_fingerprint build_shared=true same_generation=true\n[distann-multicluster] physical_benchmark_same_generation_recall scale=100k control=control candidate=candidate byte_identical=true\n";
        let rows = parse_distann_multinode_rows(raw);
        assert_eq!(rows[0].0, "physical_benchmark_generation");
        assert_eq!(
            rows[0].1.get("generation_identity").map(String::as_str),
            Some("abcd")
        );
        assert_eq!(rows[1].0, "physical_benchmark_same_generation_recall");
        assert_eq!(
            rows[1].1.get("byte_identical_numeric").map(String::as_str),
            Some("1")
        );
    }

    #[test]
    fn distann_physical_paired_recall_is_structured() {
        let raw = "[distann-multicluster] physical_benchmark_paired_recall scale=100k control=bw4-control candidate=bw8-candidate query_rows=200 trials=2000 candidate_wins=7 control_wins=0 ties=193 candidate_minus_control_mean=0.006500 paired_bootstrap_ci95_low=0.002000 paired_bootstrap_ci95_high=0.012500\n";
        let rows = parse_distann_multinode_rows(raw);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "physical_benchmark_paired_recall");
        assert_eq!(
            rows[0].1.get("candidate_wins").map(String::as_str),
            Some("7")
        );
        assert_eq!(
            rows[0]
                .1
                .get("paired_bootstrap_ci95_high")
                .map(String::as_str),
            Some("0.012500")
        );
    }

    #[test]
    fn distann_physical_provenance_is_structured() {
        let raw = "[distann-multicluster] physical_benchmark_provenance scale=10k extension_git_sha=0123456789abcdef extension_build_profile=release nodes=3 unanimous=true\n";
        let rows = parse_distann_multinode_rows(raw);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "physical_benchmark_provenance");
        assert_eq!(
            rows[0].1.get("extension_git_sha").map(String::as_str),
            Some("0123456789abcdef")
        );
        assert_eq!(
            rows[0].1.get("extension_build_profile").map(String::as_str),
            Some("release")
        );
        assert_eq!(rows[0].1.get("unanimous").map(String::as_str), Some("true"));
        assert_eq!(
            rows[0].1.get("unanimous_numeric").map(String::as_str),
            Some("1")
        );
    }

    #[test]
    fn distann_physical_scan_round_notice_is_structured_from_fixture_summary() {
        let raw = "[distann-multicluster] [postgres notice] ec_distann_scan_round round=0 requested_nodes=64 expanded_nodes=unmeasured transport_wait_ns=1239324 straggler_spread_ns=1021885 request_bytes=512 response_bytes=112190\n";
        let rows = parse_distann_multinode_rows(raw);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "physical_benchmark_scan_round");
        assert_eq!(
            rows[0].1.get("transport_wait_ns").map(String::as_str),
            Some("1239324")
        );
        assert_eq!(
            rows[0].1.get("response_bytes").map(String::as_str),
            Some("112190")
        );
    }

    #[test]
    fn distann_physical_topology_and_gate_are_structured() {
        let raw = "[distann-multicluster] physical_topology phase=published node=2 state=Published records=33 rows=33 non_owned=0 orphans=0 graph_bytes=65536 row_bytes=16384 directory_bytes=16384 control_bytes=8192\n\
[distann-multicluster] physical_topology_gate pass=true owners=3 remote_verified=3 source_rows=90\n\
[distann-multicluster] physical_publish_fault participant_down_partial pass=true decision=Pending registration=Decided active_count=0 local_state=Ready remote_acked_state=Published unavailable_node=3\n\
[distann-multicluster] physical_publish_fault post_ack_pre_pointer pass=true decision=Pending registration=Decided active_count=0 owner_states=Ready,Published,Published\n\
[distann-multicluster] physical_publish_fault idempotent_recovery pass=true decision=Applied registration=Published active_count=1 owner_states=Published,Published,Published\n\
[distann-multicluster] physical_benchmark_recall scale=10k arm=physical seed_strategy=persisted_head queries=10 trials=100 recall=1.0000 mean_ms=10727.91\n\
[distann-multicluster] physical_benchmark_latency scale=10k arm=physical seed_strategy=persisted_head count=5 mean_ms=10744.10 p50_ms=10664.70 p95_ms=11065.20 p99_ms=11125.80 max_ms=11141.00 concurrency=1 cache=warm\n\
[distann-multicluster] physical_benchmark_head scale=10k head_index_cap=4096 head_search_width=32 head_seed_count=32 seed_strategy=persisted_head neighbor_score_mode=rabitq sample_count=4096 head_sample_bytes=25231360 head_graph_bytes=540672 head_cache_estimated_bytes=25772032\n\
[distann-multicluster] physical_benchmark_head_membership scale=10k head_construction=partition_union sample_count=4096 ids_sha256=abcd artifact=artifacts/physical-head-membership.json\n\
[distann-multicluster] physical_benchmark_head_policy scale=10k policy=training_landmarks_exact scoring_mode=exact_landmark_scan training_queries=200 training_query_digest=aaaa head_index_cap=4096 returned_seed_count=32 sample_count=4096 head_sample_digest=bbbb\n";
        let rows = parse_distann_multinode_rows(raw);
        let topology = rows
            .iter()
            .find(|(metric, _)| metric == "physical_topology")
            .expect("topology row");
        assert_eq!(
            topology.1.get("topology_ok").map(String::as_str),
            Some("true")
        );
        assert!(rows.iter().any(|(metric, values)| {
            metric == "drill_outcome"
                && values.get("drill").map(String::as_str) == Some("physical_topology_gate")
                && values.get("pass").map(String::as_str) == Some("true")
        }));
        for drill in [
            "physical_publish_fault_participant_down_partial",
            "physical_publish_fault_post_ack_pre_pointer",
            "physical_publish_fault_idempotent_recovery",
        ] {
            assert!(rows.iter().any(|(metric, values)| {
                metric == "drill_outcome"
                    && values.get("drill").map(String::as_str) == Some(drill)
                    && values.get("pass").map(String::as_str) == Some("true")
            }));
        }
        assert!(rows.iter().any(|(metric, values)| {
            metric == "physical_benchmark_recall"
                && values.get("arm").map(String::as_str) == Some("physical")
                && values.get("seed_strategy").map(String::as_str) == Some("persisted_head")
                && values.get("recall").map(String::as_str) == Some("1.0000")
        }));
        assert!(rows.iter().any(|(metric, values)| {
            metric == "physical_benchmark_latency"
                && values.get("seed_strategy").map(String::as_str) == Some("persisted_head")
                && values.get("p95_ms").map(String::as_str) == Some("11065.20")
        }));
        assert!(rows.iter().any(|(metric, values)| {
            metric == "physical_benchmark_head"
                && values.get("sample_count").map(String::as_str) == Some("4096")
                && values.get("head_search_width").map(String::as_str) == Some("32")
        }));
        assert!(rows.iter().any(|(metric, values)| {
            metric == "physical_benchmark_head_membership"
                && values.get("head_construction").map(String::as_str) == Some("partition_union")
                && values.get("sample_count").map(String::as_str) == Some("4096")
        }));
        assert!(rows.iter().any(|(metric, values)| {
            metric == "physical_benchmark_head_policy"
                && values.get("policy").map(String::as_str) == Some("training_landmarks_exact")
                && values.get("scoring_mode").map(String::as_str) == Some("exact_landmark_scan")
                && values.get("training_queries").map(String::as_str) == Some("200")
        }));
    }

    #[test]
    fn distann_skipped_drill_is_structured_without_claiming_pass() {
        let raw = "[distann-multicluster] physical_routed_delete_vacuum pass=skipped reason=skip_routed_delete_vacuum_drill\n";
        let rows = parse_distann_multinode_rows(raw);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "drill_outcome");
        assert_eq!(
            rows[0].1.get("drill").map(String::as_str),
            Some("physical_routed_delete_vacuum")
        );
        assert_eq!(rows[0].1.get("pass").map(String::as_str), Some("skipped"));
        assert_eq!(
            rows[0].1.get("reason").map(String::as_str),
            Some("skip_routed_delete_vacuum_drill")
        );
        assert!(!rows[0].1.contains_key("pass_numeric"));
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
                sample_backend_memory: None,
                memory_sample_interval_ms: None,
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
            "load_session_gucs": ["ec_spire.leaf_block_rows=64"],
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
            .any(|w| w == ["--load-session-guc", "ec_spire.leaf_block_rows=64"]));
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
        assert!(step.expected_artifacts.iter().any(|path| path
            .ends_with("spire-local-multinode-task121/topology.local.json")
            && !path.starts_with("target")));
        assert!(!step
            .expected_artifacts
            .iter()
            .any(|path| path.ends_with("bench-suite/results.jsonl")));
    }

    #[test]
    fn distann_local_multinode_step_expands_head_index_cap() {
        let raw = r#"{
          "name": "distann-head-cap",
          "schema_version": 1,
          "defaults": {"pg": 18},
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "cap-256",
            "nodes": 3,
            "head_index_cap": 256,
            "build_shards": 4,
            "beam_width": 16,
            "hop_rounds": 25,
            "seed_strategy": "head_sample_exact",
            "head_search_width": 128,
            "head_seed_count": 64,
            "physical_benchmark": true,
            "compact_artifacts": true,
            "allow_debug_extension": true,
            "traversal_replica_enospc_drill": true,
            "artifact_dir": "artifacts/cap-256",
            "benchmark_warmup_iterations": 7,
            "drop_extension_cleanup_drill": true,
            "corpus_prefix": "ec_real_10k"
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");

        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command
            .windows(2)
            .any(|window| window == ["--head-index-cap", "256"]));
        assert!(command
            .windows(2)
            .any(|window| window == ["--build-shards", "4"]));
        assert!(command
            .windows(2)
            .any(|window| window == ["--beam-width", "16"]));
        assert!(command
            .windows(2)
            .any(|window| window == ["--hop-rounds", "25"]));
        assert!(command
            .windows(2)
            .any(|window| window == ["--seed-strategy", "head_sample_exact"]));
        assert!(command
            .windows(2)
            .any(|window| window == ["--head-search-width", "128"]));
        assert!(command
            .windows(2)
            .any(|window| window == ["--head-seed-count", "64"]));
        assert!(command
            .windows(2)
            .any(|window| window == ["--benchmark-warmup-iterations", "7"]));
        assert!(command.contains(&"--drop-extension-cleanup-drill".into()));
        assert!(command.contains(&"--allow-debug-extension".into()));
        assert!(command.contains(&"--traversal-replica-enospc-drill".into()));
        assert_eq!(
            config.steps[0].expected_artifacts(),
            vec![PathBuf::from(
                "artifacts/cap-256/distann-multinode-summary.log"
            )]
        );
    }

    #[test]
    fn distann_task167_heldout_regression_gate_is_step_local_and_paired() {
        let raw = r#"{
          "name": "task167-heldout-gate",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "physical-50k",
            "physical_benchmark": true,
            "corpus_prefix": "ec_real_50k",
            "task167_heldout_baseline_deficit": 0.008611,
            "task167_heldout_physical_sample_sd": 0.000224
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");
        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command
            .windows(2)
            .any(|window| { window == ["--task167-heldout-baseline-deficit", "0.008611"] }));
        assert!(command
            .windows(2)
            .any(|window| { window == ["--task167-heldout-physical-sample-sd", "0.000224"] }));

        let missing_sd = raw.replace(
            ",\n            \"task167_heldout_physical_sample_sd\": 0.000224",
            "",
        );
        let config: SuiteConfig = serde_json::from_str(&missing_sd).expect("suite parses");
        let error = validate_config(&config).expect_err("unpaired baseline must fail");
        assert!(error.to_string().contains("requires both baseline deficit"));
    }

    #[test]
    fn distann_local_multinode_expands_secure_remote_transport() {
        let raw = r#"{
          "name": "distann-secure-transport",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "tls",
            "secure_remote_transport": true,
            "tls_security_matrix": true,
            "allow_debug_extension": true,
            "nodes": 3,
            "artifact_dir": "artifacts/tls"
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");
        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command.contains(&"--secure-remote-transport".into()));
        assert!(command.contains(&"--tls-security-matrix".into()));
        assert_eq!(
            config.steps[0].expected_artifacts(),
            vec![PathBuf::from(
                "artifacts/tls/task236-tls-security-matrix.log"
            )]
        );
    }

    #[test]
    fn distann_local_multinode_labels_and_expands_metrics_modes() {
        let raw = r#"{
          "name": "distann-metrics-modes",
          "schema_version": 1,
          "steps": [
            {
              "kind": "distann-local-multinode",
              "name": "benchmark",
              "physical_benchmark": true,
              "metrics_mode": "benchmark",
              "corpus_prefix": "ec_real_10k"
            },
            {
              "kind": "distann-local-multinode",
              "name": "full",
              "physical_benchmark": true,
              "metrics_mode": "full_metrics",
              "corpus_prefix": "ec_real_10k"
            },
            {
              "kind": "distann-local-multinode",
              "name": "legacy-full",
              "physical_benchmark": true,
              "distann_stage_counters": true,
              "corpus_prefix": "ec_real_10k"
            }
          ]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
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

        let benchmark = &manifest.steps[0];
        assert!(!benchmark
            .command
            .contains(&"--distann-stage-counters".into()));
        assert!(!benchmark
            .command
            .contains(&"--sample-backend-memory".into()));
        assert!(benchmark
            .tags
            .contains(&"metrics_mode=benchmark".to_owned()));
        assert_eq!(
            add_result_context(&manifest, benchmark, BTreeMap::new())
                .get("metrics_mode")
                .map(String::as_str),
            Some("benchmark")
        );

        let full = &manifest.steps[1];
        assert!(full.command.contains(&"--distann-stage-counters".into()));
        assert!(full.command.contains(&"--sample-backend-memory".into()));
        assert!(full.tags.contains(&"metrics_mode=full_metrics".to_owned()));
        assert_eq!(
            add_result_context(&manifest, full, BTreeMap::new())
                .get("metrics_mode")
                .map(String::as_str),
            Some("full_metrics")
        );

        let legacy_full = &manifest.steps[2];
        assert!(legacy_full
            .command
            .contains(&"--distann-stage-counters".into()));
        assert!(!legacy_full
            .command
            .contains(&"--sample-backend-memory".into()));
        assert!(legacy_full
            .tags
            .contains(&"metrics_mode=full_metrics".to_owned()));
        assert_eq!(
            add_result_context(&manifest, legacy_full, BTreeMap::new())
                .get("metrics_mode")
                .map(String::as_str),
            Some("full_metrics")
        );
    }

    #[test]
    fn distann_nfr_021_preregistration_is_manifested_and_labels_arm_rows() {
        let raw = r#"{
          "name": "distann-nfr-021",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "candidate-10k",
            "physical_benchmark": true,
            "corpus_prefix": "ec_real_10k",
            "nfr_021": {
              "id": "owner-candidate",
              "role": "candidate",
              "admissibility": "conforming",
              "rationale": "physical owner generation; no derived O(N) relation"
            }
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
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

        assert_eq!(step.nfr_021_registrations.len(), 1);
        assert_eq!(step.nfr_021_registrations[0].id, "owner-candidate");
        let values = add_result_context(
            &manifest,
            step,
            BTreeMap::from([
                ("arm".into(), "physical".into()),
                ("variant".into(), "physical".into()),
            ]),
        );
        assert_eq!(
            values.get("nfr_021_role").map(String::as_str),
            Some("candidate")
        );
        assert_eq!(
            values
                .get("nfr_021_preregistered_admissibility")
                .map(String::as_str),
            Some("conforming")
        );
    }

    #[test]
    fn distann_nfr_021_rejects_nonconforming_decision_arm_before_measurement() {
        let raw = r#"{
          "name": "distann-nfr-021-invalid",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "candidate-10k",
            "physical_benchmark": true,
            "corpus_prefix": "ec_real_10k",
            "nfr_021": {
              "id": "replica-candidate",
              "role": "candidate",
              "admissibility": "nonconforming",
              "rationale": "coordinator traversal replica"
            }
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        let error = validate_config(&config).expect_err("invalid candidate must be rejected");
        assert!(error
            .to_string()
            .contains("cannot use an NFR-021-nonconforming candidate arm"));
    }

    #[test]
    fn distann_traversal_replica_arm_cannot_be_a_decision_arm_or_claim_conformance() {
        let decision_arm = r#"{
          "name": "distann-replica-decision",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "replica-10k",
            "physical_benchmark": true,
            "corpus_prefix": "ec_real_10k",
            "benchmark_seed_variants": [{
              "name": "replica",
              "seed_strategy": "persisted_head",
              "head_search_width": 32,
              "head_seed_count": 32,
              "neighbor_score_mode": "rabitq",
              "traversal_replica": true,
              "nfr_021": {
                "id": "replica-candidate",
                "role": "candidate",
                "admissibility": "nonconforming",
                "rationale": "FR-084 accelerator"
              }
            }]
          }]
        }"#;
        // Honestly registered as nonconforming, it is still rejected as a
        // decision arm — this one is caught by the general NFR-022 rule.
        let config: SuiteConfig = serde_json::from_str(decision_arm).expect("suite parses");
        let error = validate_config(&config).expect_err("replica decision arm must be rejected");
        assert!(
            error.to_string().contains("for a decision"),
            "unexpected error: {error}"
        );

        // Registered as conforming, it is rejected for the claim itself,
        // whatever role it takes: a coordinator-resident full-graph copy is
        // never NFR-021-conforming (Task 210 P1).
        for role in ["candidate", "context"] {
            let claims_conformance = decision_arm
                .replace("\"role\": \"candidate\"", &format!("\"role\": \"{role}\""))
                .replace(
                    "\"admissibility\": \"nonconforming\"",
                    "\"admissibility\": \"conforming\"",
                );
            let config: SuiteConfig =
                serde_json::from_str(&claims_conformance).expect("suite parses");
            let error =
                validate_config(&config).expect_err("replica arm cannot be registered conforming");
            assert!(
                error
                    .to_string()
                    .contains("must be NFR-021-registered as nonconforming"),
                "unexpected error for role {role}: {error}"
            );
        }
    }

    #[test]
    fn distann_nfr_021_normalizes_fixed_roster_owner_growth() {
        let manifest = SuiteManifest {
            suite: "nfr-021".into(),
            schema_version: 1,
            config: "suite.json".into(),
            config_sha256: "hash".into(),
            dry_run: false,
            generated_at_unix_ms: 0,
            runner_git_commit: None,
            connection: ManifestConnection {
                database: "tqvector_bench".into(),
                host: None,
                port: None,
                user: None,
                password_configured: false,
            },
            backend: None,
            steps: Vec::new(),
            threshold_results: Vec::new(),
        };
        let registration = DistannNfr021ManifestRegistration {
            variant: None,
            id: "owner-control".into(),
            role: DistannDecisionRole::Control,
            admissibility: DistannNfr021Admissibility::Conforming,
            rationale: "physical owner generation".into(),
        };
        let mut evidence = DistannNfr021Evidence {
            scales: ["10k", "50k", "100k"]
                .into_iter()
                .map(ToOwned::to_owned)
                .collect(),
            topology_scales: ["10k", "50k", "100k"]
                .into_iter()
                .map(ToOwned::to_owned)
                .collect(),
            owner_nodes: ["1"].into_iter().map(ToOwned::to_owned).collect(),
            head_capacities: ["4096"].into_iter().map(ToOwned::to_owned).collect(),
            ..DistannNfr021Evidence::default()
        };
        evidence
            .bytes_per_owned_record
            .insert(("10k".into(), "1".into()), 7_600.0);
        evidence
            .bytes_per_owned_record
            .insert(("50k".into(), "1".into()), 8_050.0);
        evidence
            .bytes_per_owned_record
            .insert(("100k".into(), "1".into()), 8_200.0);
        evidence
            .raw_graph_side_bytes
            .insert(("10k".into(), "1".into()), 25_706_496.0);
        evidence
            .raw_graph_side_bytes
            .insert(("100k".into(), "1".into()), 277_372_928.0);

        let row =
            distann_nfr_021_result_row(&manifest, "owner-control".into(), registration, evidence);

        assert_eq!(
            row.values.get("actual_admissibility").map(String::as_str),
            Some("conforming")
        );
        assert_eq!(
            row.values
                .get("preregistration_matches")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(
            row.values
                .get("normalized_bytes_per_owned_record_growth_max")
                .map(String::as_str),
            Some("1.078947")
        );
        assert_eq!(
            row.values
                .get("raw_fixed_roster_graph_side_growth_max")
                .map(String::as_str),
            Some("10.789994")
        );
    }

    #[test]
    fn distann_nfr_021_classifies_unsharded_derived_relation_as_nonconforming() {
        let manifest = SuiteManifest {
            suite: "nfr-021".into(),
            schema_version: 1,
            config: "suite.json".into(),
            config_sha256: "hash".into(),
            dry_run: false,
            generated_at_unix_ms: 0,
            runner_git_commit: None,
            connection: ManifestConnection {
                database: "tqvector_bench".into(),
                host: None,
                port: None,
                user: None,
                password_configured: false,
            },
            backend: None,
            steps: Vec::new(),
            threshold_results: Vec::new(),
        };
        let registration = DistannNfr021ManifestRegistration {
            variant: Some("replica".into()),
            id: "replica-context".into(),
            role: DistannDecisionRole::Context,
            admissibility: DistannNfr021Admissibility::Nonconforming,
            rationale: "known FR-084 negative fixture".into(),
        };
        let evidence = DistannNfr021Evidence {
            max_unsharded_derived_bytes: 1_659_518_976,
            ..DistannNfr021Evidence::default()
        };

        let row =
            distann_nfr_021_result_row(&manifest, "replica-context".into(), registration, evidence);

        assert_eq!(
            row.values.get("actual_admissibility").map(String::as_str),
            Some("nonconforming")
        );
        assert_eq!(
            row.values
                .get("preregistration_matches")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(
            row.values
                .get("max_unsharded_derived_bytes")
                .map(String::as_str),
            Some("1659518976")
        );
    }

    fn nfr_021_test_manifest(suite: &str) -> SuiteManifest {
        SuiteManifest {
            suite: suite.into(),
            schema_version: 1,
            config: "suite.json".into(),
            config_sha256: "hash".into(),
            dry_run: false,
            generated_at_unix_ms: 0,
            runner_git_commit: None,
            connection: ManifestConnection {
                database: "tqvector_bench".into(),
                host: None,
                port: None,
                user: None,
                password_configured: false,
            },
            backend: None,
            steps: Vec::new(),
            threshold_results: Vec::new(),
        }
    }

    fn nfr_021_complete_owner_evidence() -> DistannNfr021Evidence {
        let mut evidence = DistannNfr021Evidence {
            scales: ["10k", "50k", "100k"]
                .into_iter()
                .map(ToOwned::to_owned)
                .collect(),
            topology_scales: ["10k", "50k", "100k"]
                .into_iter()
                .map(ToOwned::to_owned)
                .collect(),
            owner_nodes: ["1"].into_iter().map(ToOwned::to_owned).collect(),
            head_capacities: ["4096"].into_iter().map(ToOwned::to_owned).collect(),
            ..DistannNfr021Evidence::default()
        };
        for (scale, bytes) in [("10k", 7_600.0), ("50k", 8_050.0), ("100k", 8_200.0)] {
            evidence
                .bytes_per_owned_record
                .insert((scale.into(), "1".into()), bytes);
        }
        evidence
    }

    fn nfr_021_owner_registration() -> DistannNfr021ManifestRegistration {
        DistannNfr021ManifestRegistration {
            variant: None,
            id: "owner-control".into(),
            role: DistannDecisionRole::Control,
            admissibility: DistannNfr021Admissibility::Conforming,
            rationale: "physical owner generation".into(),
        }
    }

    /// 005 review round 2 closed the last owned gap: the membership-only head
    /// persists zero sample/graph rows, the allowlist is empty, and a
    /// reappearing coordinator-resident head relation is a hard violation —
    /// while relations reported at zero bytes (the emitter still itemises
    /// them) do not fail anything.
    #[test]
    fn distann_nfr_021_hard_fails_a_reappearing_head_gap_and_accepts_zero_byte_relations() {
        let manifest = nfr_021_test_manifest("nfr-021-closed-gap");
        let mut evidence = nfr_021_complete_owner_evidence();
        evidence
            .coordinator_resident_unsharded
            .insert("ec_distann_generation_head_sample".into(), 0);
        evidence
            .coordinator_resident_unsharded
            .insert("ec_distann_generation_head_graph".into(), 0);
        let row = distann_nfr_021_result_row(
            &manifest,
            "owner-control".into(),
            nfr_021_owner_registration(),
            evidence,
        );
        assert_eq!(
            row.values.get("actual_admissibility").map(String::as_str),
            Some("conforming")
        );
        assert_eq!(
            row.values
                .get("coordinator_resident_unsharded_bytes")
                .map(String::as_str),
            Some("0")
        );
        assert_eq!(
            row.values
                .get("outstanding_distribution_gap")
                .map(String::as_str),
            Some("none")
        );

        let manifest = nfr_021_test_manifest("nfr-021-reappeared-gap");
        let mut evidence = nfr_021_complete_owner_evidence();
        evidence
            .coordinator_resident_unsharded
            .insert("ec_distann_generation_head_sample".into(), 25_280_512);
        let row = distann_nfr_021_result_row(
            &manifest,
            "owner-control".into(),
            nfr_021_owner_registration(),
            evidence,
        );
        assert_eq!(
            row.values.get("actual_admissibility").map(String::as_str),
            Some("nonconforming"),
            "a reappearing head gap is a hard violation, not an allowlist entry"
        );
        assert_eq!(
            row.values
                .get("outstanding_distribution_gap")
                .map(String::as_str),
            Some("ec_distann_generation_head_sample:25280512:unowned")
        );
    }

    #[test]
    fn distann_nfr_021_fails_on_a_coordinator_resident_relation_that_is_not_a_known_gap() {
        let manifest = nfr_021_test_manifest("nfr-021-new-gap");
        let mut evidence = nfr_021_complete_owner_evidence();
        evidence
            .coordinator_resident_unsharded
            .insert("some_new_coordinator_cache".into(), 4_096);

        let row = distann_nfr_021_result_row(
            &manifest,
            "owner-control".into(),
            nfr_021_owner_registration(),
            evidence,
        );

        assert_eq!(
            row.values.get("actual_admissibility").map(String::as_str),
            Some("nonconforming")
        );
        assert_eq!(
            row.values.get("decision_eligible").map(String::as_str),
            Some("false")
        );
        assert_eq!(
            row.values
                .get("outstanding_distribution_gap")
                .map(String::as_str),
            Some("some_new_coordinator_cache:4096:unowned")
        );
    }

    #[test]
    fn distann_nfr_021_head_gap_clears_when_the_head_is_sharded() {
        let manifest = nfr_021_test_manifest("nfr-021-gap-closed");

        let row = distann_nfr_021_result_row(
            &manifest,
            "owner-control".into(),
            nfr_021_owner_registration(),
            nfr_021_complete_owner_evidence(),
        );

        assert_eq!(
            row.values
                .get("outstanding_distribution_gap")
                .map(String::as_str),
            Some("none")
        );
        assert_eq!(
            row.values
                .get("coordinator_resident_unsharded_bytes")
                .map(String::as_str),
            Some("0")
        );
    }

    #[test]
    fn distann_storage_ratio_is_mandatory_for_physical_steps() {
        let manifest = SuiteManifest {
            suite: "storage-required".into(),
            schema_version: 1,
            config: "suite.json".into(),
            config_sha256: "hash".into(),
            dry_run: false,
            generated_at_unix_ms: 0,
            runner_git_commit: None,
            connection: ManifestConnection {
                database: "tqvector_bench".into(),
                host: None,
                port: None,
                user: None,
                password_configured: false,
            },
            backend: None,
            steps: vec![StepRecord {
                name: "physical-10k".into(),
                kind: "distann-local-multinode".into(),
                command: vec!["--physical-benchmark".into()],
                selected: true,
                quant: None,
                isa: None,
                kernel_status: None,
                pgoptions: None,
                tags: Vec::new(),
                nfr_021_registrations: Vec::new(),
                expected_artifacts: Vec::new(),
                status: Some(StepStatus::Succeeded),
                started_at_unix_ms: None,
                finished_at_unix_ms: None,
                duration_ms: None,
                exit_code: Some(0),
                parallel_workers_before: None,
                parallel_workers_after: None,
                parallel_workers_delta: None,
            }],
            threshold_results: Vec::new(),
        };
        let storage = ResultRow {
            suite: manifest.suite.clone(),
            step: "physical-10k".into(),
            kind: "distann-local-multinode".into(),
            metric: "physical_benchmark_storage".into(),
            artifact: "summary.log".into(),
            values: BTreeMap::from([
                ("scale".into(), "10k".into()),
                ("variant".into(), "control".into()),
                ("arm".into(), "physical".into()),
            ]),
        };
        assert!(assert_distann_storage_ratio_rows(&manifest, &[storage.clone()]).is_err());

        let ratio = ResultRow {
            metric: "physical_benchmark_storage_ratio".into(),
            ..storage.clone()
        };
        assert!(assert_distann_storage_ratio_rows(&manifest, &[storage, ratio]).is_ok());
    }

    #[test]
    fn distann_benchmark_metrics_mode_rejects_heavy_instrumentation() {
        let raw = r#"{
          "name": "distann-metrics-conflict",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "benchmark",
            "physical_benchmark": true,
            "metrics_mode": "benchmark",
            "distann_stage_counters": true,
            "corpus_prefix": "ec_real_10k"
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        let error = validate_config(&config).expect_err("conflict must fail");
        assert!(error
            .to_string()
            .contains("benchmark metrics_mode cannot enable full-metrics instrumentation"));
    }

    #[test]
    fn distann_local_multinode_expands_task183_stage_profile() {
        let raw = r#"{
          "name": "distann-stage-profile",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "profile-100k",
            "physical_benchmark": true,
            "distann_stage_counters": true,
            "corpus_prefix": "ec_real_100k"
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");
        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command.contains(&"--distann-stage-counters".into()));

        let rows = parse_distann_multinode_rows(
            "[distann-multicluster] physical_benchmark_stage scale=100k variant=control arm=physical stage=head_score scans=50 samples=50 elapsed_ns=100000000 mean_ms=2.0\n\
[distann-multicluster] physical_benchmark_materialization_work scale=100k variant=control arm=physical metric=remote_candidates_requested scans=50 value=500 mean_per_scan=10.0\n",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "physical_benchmark_stage");
        assert_eq!(
            rows[0].1.get("stage").map(String::as_str),
            Some("head_score")
        );
        assert_eq!(rows[1].0, "physical_benchmark_materialization_work");
        assert_eq!(
            rows[1].1.get("metric").map(String::as_str),
            Some("remote_candidates_requested")
        );
    }

    #[test]
    fn task224_owner_payload_locality_is_suite_addressable_and_structured() {
        let raw = r#"{
          "name": "task224-owner-locality",
          "schema_version": 1,
          "artifact_dir": "reviews/task-224/002-locality-attribution/artifacts/run",
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "locality-100k",
            "artifact_dir": "${artifact_dir}/toasted",
            "reuse_fixture": true,
            "reuse_provenance_dir": "${artifact_dir}/id-only",
            "physical_benchmark": true,
            "distann_stage_counters": true,
            "stage_counter_only": true,
            "skip_routed_delete_vacuum_drill": true,
            "owner_payload_shape": "toasted",
            "corpus_prefix": "ec_real_100k"
          }]
        }"#;
        let mut config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");
        apply_artifact_dir_templates(&mut config);
        let SuiteStep::DistannLocalMultinode(step) = &config.steps[0] else {
            panic!("expected DistANN local multinode step");
        };
        assert_eq!(
            step.artifact_dir.as_deref(),
            Some(Path::new(
                "reviews/task-224/002-locality-attribution/artifacts/run/toasted"
            ))
        );
        assert_eq!(
            step.reuse_provenance_dir.as_deref(),
            Some(Path::new(
                "reviews/task-224/002-locality-attribution/artifacts/run/id-only"
            ))
        );
        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command.contains(&"--distann-stage-counters".into()));
        assert!(command.contains(&"--stage-counter-only".into()));
        assert!(command.contains(&"--skip-routed-delete-vacuum-drill".into()));
        assert!(command
            .windows(2)
            .any(|window| { window == ["--owner-payload-shape", "toasted"] }));

        let rows = parse_distann_multinode_rows(
            "[distann-multicluster] physical_benchmark_stage scale=100k variant=control payload_shape=toasted arm=physical stage=materialize_owner_binary_send_work scans=50 samples=50 elapsed_ns=100000000 mean_ms=2.0\n\
[distann-multicluster] physical_benchmark_materialization_work scale=100k variant=control payload_shape=toasted arm=physical metric=owner_external_toast_values scans=50 value=500 mean_per_scan=10.0\n",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].1.get("payload_shape").map(String::as_str),
            Some("toasted")
        );
        assert_eq!(
            rows[1].1.get("metric").map(String::as_str),
            Some("owner_external_toast_values")
        );
    }

    #[test]
    fn task224_mat26_candidate_requires_unprofiled_vector_projection() {
        let raw = r#"{
          "name": "task224-mat26",
          "schema_version": 1,
          "artifact_dir": "reviews/task-224/003-isolated-candidate/artifacts/run",
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "candidate-100k",
            "artifact_dir": "${artifact_dir}/candidate",
            "physical_benchmark": true,
            "distann_stage_counters": true,
            "stage_counter_only": true,
            "owner_payload_shape": "vector-bearing",
            "skip_owner_locality_profile": true,
            "owner_fast_real_array_send": true,
            "corpus_prefix": "ec_real_100k"
          }]
        }"#;
        let mut config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");
        apply_artifact_dir_templates(&mut config);
        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command.contains(&"--skip-owner-locality-profile".into()));
        assert!(command.contains(&"--owner-fast-real-array-send".into()));

        let invalid = raw.replace(
            "\"skip_owner_locality_profile\": true",
            "\"skip_owner_locality_profile\": false",
        );
        let invalid: SuiteConfig = serde_json::from_str(&invalid).expect("invalid suite parses");
        assert!(validate_config(&invalid).is_err());
    }

    #[test]
    fn task224_mat26_preregistered_suite_validates_as_same_generation_ab() {
        let raw = include_str!("../../../suites/task224-mat26-fast-real-array-100k.json");
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");
        assert_eq!(config.steps.len(), 4);
        let SuiteStep::DistannLocalMultinode(control) = &config.steps[0] else {
            panic!("expected control DistANN step");
        };
        let SuiteStep::DistannLocalMultinode(candidate) = &config.steps[1] else {
            panic!("expected candidate DistANN step");
        };
        assert!(!control.reuse_fixture);
        assert!(candidate.reuse_fixture);
        assert_eq!(control.run_dir, candidate.run_dir);
        assert!(!control.allow_debug_extension);
        assert!(!candidate.allow_debug_extension);
        assert!(control.skip_owner_locality_profile);
        assert!(candidate.skip_owner_locality_profile);
        assert!(!control.owner_fast_real_array_send);
        assert!(candidate.owner_fast_real_array_send);
        assert!(!control.materialization_correctness);
        assert!(!candidate.materialization_correctness);
        assert_eq!(control.benchmark_seed_variants.len(), 2);
        assert_eq!(candidate.benchmark_seed_variants.len(), 2);
        for (left, right) in control
            .benchmark_seed_variants
            .iter()
            .zip(&candidate.benchmark_seed_variants)
        {
            assert_eq!(left.name, right.name);
            assert_eq!(
                left.materialization_batch_size,
                right.materialization_batch_size
            );
        }
        let SuiteStep::DistannLocalMultinode(control_repeat) = &config.steps[2] else {
            panic!("expected repeated control DistANN step");
        };
        let SuiteStep::DistannLocalMultinode(profiled_control) = &config.steps[3] else {
            panic!("expected profiled control DistANN step");
        };
        assert!(control_repeat.reuse_fixture);
        assert!(control_repeat.stage_counter_only);
        assert!(control_repeat.skip_owner_locality_profile);
        assert!(!control_repeat.owner_fast_real_array_send);
        assert!(!control_repeat.allow_debug_extension);
        assert!(profiled_control.reuse_fixture);
        assert!(profiled_control.stage_counter_only);
        assert!(!profiled_control.skip_owner_locality_profile);
        assert!(!profiled_control.owner_fast_real_array_send);
        assert!(!profiled_control.allow_debug_extension);
        assert_eq!(control_repeat.run_dir, control.run_dir);
        assert_eq!(profiled_control.run_dir, control.run_dir);
        assert!(config.steps.iter().all(|step| {
            let SuiteStep::DistannLocalMultinode(step) = step else {
                return false;
            };
            !step.materialization_correctness
        }));
    }

    #[test]
    fn task224_mat26_semantics_use_two_isolated_nonreuse_fixtures() {
        let raw = include_str!("../../../suites/task224-mat26-semantics-10k.json");
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");
        assert_eq!(config.steps.len(), 2);
        let SuiteStep::DistannLocalMultinode(control) = &config.steps[0] else {
            panic!("expected semantic control DistANN step");
        };
        let SuiteStep::DistannLocalMultinode(candidate) = &config.steps[1] else {
            panic!("expected semantic candidate DistANN step");
        };
        for step in [control, candidate] {
            assert!(!step.reuse_fixture);
            assert!(step.materialization_correctness);
            assert!(step.skip_recall);
            assert_eq!(step.benchmark_iterations, Some(1));
            assert!(!step.allow_debug_extension);
        }
        assert_ne!(control.run_dir, candidate.run_dir);
        assert!(!control.owner_fast_real_array_send);
        assert!(candidate.owner_fast_real_array_send);
    }

    #[test]
    fn reused_fixture_rejects_every_fixture_mutating_drill() {
        assert!(validate_reused_fixture_drills("ok", true, false, false, false).is_ok());
        for (enospc, drop_cleanup, correctness) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let error =
                validate_reused_fixture_drills("reused", true, enospc, drop_cleanup, correctness)
                    .expect_err("reused fixtures must reject mutating drills");
            assert!(error
                .to_string()
                .contains("reuse_fixture cannot combine with fixture-mutating drills"));
        }
    }

    #[test]
    fn distann_local_multinode_expands_seed_variants_on_one_fixture() {
        let raw = r#"{
          "name": "distann-seed-screen",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "screen-100k",
            "physical_benchmark": true,
            "corpus_prefix": "ec_real_100k",
            "benchmark_seed_variants": [
              {
                "name": "persisted-w32-s32",
                "seed_strategy": "persisted_head",
                "head_search_width": 32,
                "head_seed_count": 32,
                "neighbor_score_mode": "rabitq",
                "materialization_batch_size": 0
              },
              {
                "name": "owner-oracle",
                "seed_strategy": "owner_scan",
                "head_search_width": 32,
                "head_seed_count": 32,
                "neighbor_score_mode": "rabitq",
                "materialization_batch_size": 10
              }
            ]
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");

        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command.windows(2).any(|window| {
            window
                == [
                    "--benchmark-seed-variant",
                    "persisted-w32-s32:persisted_head:32:32:rabitq:0:off:4:100:off",
                ]
        }));
        assert!(command.windows(2).any(|window| {
            window
                == [
                    "--benchmark-seed-variant",
                    "owner-oracle:owner_scan:32:32:rabitq:10:off:4:100:off",
                ]
        }));
    }

    #[test]
    fn distann_local_multinode_expands_staged_query_slice() {
        let raw = r#"{
          "name": "distann-query-slice",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "diagnostic-rows-201-400",
            "physical_benchmark": true,
            "artifact_dir": "reviews/task-227/003-query-trace/artifacts/run",
            "corpus_prefix": "ec_real_100k",
            "queries": 200,
            "query_offset": 200,
            "reuse_fixture": true,
            "reuse_provenance_dir": "reviews/task-227/003-query-trace/artifacts/prior",
            "query_trace": true
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");
        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command
            .windows(2)
            .any(|window| window == ["--queries", "200"]));
        assert!(command
            .windows(2)
            .any(|window| window == ["--query-offset", "200"]));
        assert!(command.iter().any(|argument| argument == "--reuse-fixture"));
        assert!(command.windows(2).any(|window| {
            window
                == [
                    "--reuse-provenance-dir",
                    "reviews/task-227/003-query-trace/artifacts/prior",
                ]
        }));
        assert!(command.iter().any(|argument| argument == "--query-trace"));
        let artifacts = config.steps[0].expected_artifacts();
        assert!(artifacts
            .iter()
            .any(|artifact| { artifact.ends_with("physical-production-query-trace.json") }));
    }

    #[test]
    fn distann_query_slice_requires_real_corpus() {
        let raw = r#"{
          "name": "distann-query-slice-invalid",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "synthetic-offset",
            "query_offset": 200
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        let error = validate_config(&config).expect_err("offset needs staged corpus");
        assert!(error
            .to_string()
            .contains("query_offset requires corpus_prefix"));
    }

    #[test]
    fn distann_query_trace_requires_physical_benchmark() {
        let raw = r#"{
          "name": "distann-query-trace-invalid",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "synthetic-trace",
            "query_trace": true
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        let error = validate_config(&config).expect_err("trace needs physical benchmark");
        assert!(error
            .to_string()
            .contains("query_trace requires physical_benchmark"));
    }

    #[test]
    fn distann_graph_diagnostic_is_suite_addressable() {
        let raw = r#"{
          "name": "distann-graph-diagnostic",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "graph-100k",
            "physical_benchmark": true,
            "corpus_prefix": "ec_real_100k",
            "artifact_dir": "reviews/task-227/004-graph-diagnostics/artifacts/run",
            "graph_diagnostic": true
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");
        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command
            .iter()
            .any(|argument| argument == "--graph-diagnostic"));
        assert!(config.steps[0]
            .expected_artifacts()
            .iter()
            .any(|artifact| artifact.ends_with("physical-graph-diagnostic.json")));
    }

    #[test]
    fn distann_graph_diagnostic_requires_monolithic_control() {
        let raw = r#"{
          "name": "distann-graph-diagnostic-invalid",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "graph-without-control",
            "physical_benchmark": true,
            "graph_diagnostic": true,
            "skip_single_control": true
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        let error = validate_config(&config).expect_err("graph diagnostic needs control");
        assert!(error
            .to_string()
            .contains("graph_diagnostic requires the monolithic control"));
    }

    #[test]
    fn distann_residual_attribution_is_frozen_and_suite_addressable() {
        let raw = r#"{
          "name": "distann-residual-attribution",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "diagnostic-100k",
            "physical_benchmark": true,
            "corpus_prefix": "ec_real_100k",
            "queries": 200,
            "query_offset": 200,
            "top_k": 10,
            "head_index_cap": 4096,
            "candidate_heap_limit": 32,
            "query_trace": true,
            "graph_diagnostic": true,
            "residual_attribution": true,
            "artifact_dir": "reviews/task-227/005-query-level-attribution/artifacts/run",
            "benchmark_seed_variants": [
              {"name":"prod-bw4-rabitq","seed_strategy":"persisted_head","head_search_width":32,"head_seed_count":32,"neighbor_score_mode":"rabitq","materialization_batch_size":10,"beam_width":4,"hop_rounds":100},
              {"name":"task226-bw8-rabitq","seed_strategy":"persisted_head","head_search_width":32,"head_seed_count":32,"neighbor_score_mode":"rabitq","materialization_batch_size":10,"beam_width":8,"hop_rounds":100},
              {"name":"prod-bw4-exact-neighbor","seed_strategy":"persisted_head","head_search_width":32,"head_seed_count":32,"neighbor_score_mode":"exact_neighbor","materialization_batch_size":10,"beam_width":4,"hop_rounds":100},
              {"name":"owner-bw4-rabitq","seed_strategy":"owner_scan","head_search_width":32,"head_seed_count":32,"neighbor_score_mode":"rabitq","materialization_batch_size":10,"beam_width":4,"hop_rounds":100},
              {"name":"owner-bw4-exact-neighbor","seed_strategy":"owner_scan","head_search_width":32,"head_seed_count":32,"neighbor_score_mode":"exact_neighbor","materialization_batch_size":10,"beam_width":4,"hop_rounds":100}
            ]
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("frozen attribution suite validates");
        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command
            .iter()
            .any(|argument| argument == "--residual-attribution"));
        let artifacts = config.steps[0].expected_artifacts();
        assert!(artifacts
            .iter()
            .any(|artifact| artifact.ends_with("physical-residual-attribution.jsonl")));
        assert!(artifacts
            .iter()
            .any(|artifact| artifact.ends_with("physical-residual-query-features.jsonl")));
        assert!(artifacts
            .iter()
            .any(|artifact| { artifact.ends_with("physical-residual-attribution-summary.json") }));
    }

    #[test]
    fn distann_residual_attribution_requires_trace_and_graph() {
        let raw = r#"{
          "name": "distann-residual-attribution-invalid",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "missing-prerequisites",
            "physical_benchmark": true,
            "residual_attribution": true
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        let error = validate_config(&config).expect_err("attribution needs trace and graph");
        assert!(error
            .to_string()
            .contains("residual_attribution requires query_trace and graph_diagnostic"));
    }

    #[test]
    fn distann_materialization_correctness_is_suite_addressable_and_structured() {
        let raw = r#"{
          "name": "distann-materialization-correctness",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "correctness-10k",
            "physical_benchmark": true,
            "materialization_correctness": true,
            "corpus_prefix": "ec_real_10k",
            "benchmark_seed_variants": [
              {
                "name": "eager",
                "seed_strategy": "persisted_head",
                "head_search_width": 32,
                "head_seed_count": 32,
                "neighbor_score_mode": "rabitq",
                "materialization_batch_size": 0
              },
              {
                "name": "lazy10",
                "seed_strategy": "persisted_head",
                "head_search_width": 32,
                "head_seed_count": 32,
                "neighbor_score_mode": "rabitq",
                "materialization_batch_size": 10
              }
            ]
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");
        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command.contains(&"--materialization-correctness".into()));

        let rows = parse_distann_multinode_rows(
            "[distann-multicluster] physical_materialization_correctness scale=10k scenario=null_payload pass=true rows=10 eager_digest=aaaa candidate_digest=aaaa null_ok=true toast_ok=true\n",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "physical_materialization_correctness");
        assert_eq!(
            rows[0].1.get("scenario").map(String::as_str),
            Some("null_payload")
        );
        assert_eq!(rows[0].1.get("pass_numeric").map(String::as_str), Some("1"));
    }

    #[test]
    fn distann_owner_plan_and_fixed_work_variants_are_suite_addressable() {
        let raw = r#"{
          "name": "distann-owner-plan-fixed-work",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "candidate-100k",
            "physical_benchmark": true,
            "materialization_correctness": true,
            "corpus_prefix": "ec_real_100k",
            "beam_width": 4,
            "hop_rounds": 100,
            "benchmark_seed_variants": [
              {
                "name": "plan-off",
                "seed_strategy": "persisted_head",
                "head_search_width": 32,
                "head_seed_count": 32,
                "neighbor_score_mode": "rabitq",
                "materialization_batch_size": 10,
                "owner_payload_plan_cache": false,
                "beam_width": 4,
                "hop_rounds": 100
              },
              {
                "name": "plan-on",
                "seed_strategy": "persisted_head",
                "head_search_width": 32,
                "head_seed_count": 32,
                "neighbor_score_mode": "rabitq",
                "materialization_batch_size": 10,
                "owner_payload_plan_cache": true,
                "beam_width": 8,
                "hop_rounds": 50
              }
            ]
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        assert!(
            validate_config(&config).is_err(),
            "correctness pair must isolate plan cache"
        );

        let raw = raw
            .replace("\"beam_width\": 8", "\"beam_width\": 4")
            .replace("\"hop_rounds\": 50", "\"hop_rounds\": 100");
        let config: SuiteConfig = serde_json::from_str(&raw).expect("suite parses");
        validate_config(&config).expect("isolated plan pair validates");
        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command.windows(2).any(|window| {
            window
                == [
                    "--benchmark-seed-variant",
                    "plan-off:persisted_head:32:32:rabitq:10:off:4:100:off",
                ]
        }));
        assert!(command.windows(2).any(|window| {
            window
                == [
                    "--benchmark-seed-variant",
                    "plan-on:persisted_head:32:32:rabitq:10:on:4:100:off",
                ]
        }));
    }

    #[test]
    fn distann_variant_rejects_removed_owner_validation_selector() {
        let raw = r#"{
          "name": "production",
          "seed_strategy": "persisted_head",
          "head_search_width": 32,
          "head_seed_count": 32,
          "neighbor_score_mode": "rabitq",
          "materialization_batch_size": 10,
          "owner_validation_cache": true
        }"#;
        assert!(serde_json::from_str::<DistannBenchmarkSeedVariant>(raw).is_err());
    }

    #[test]
    fn distann_variants_normalize_effective_search_shape() {
        let raw = r#"{
          "name": "production",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "production-100k",
            "physical_benchmark": true,
            "corpus_prefix": "ec_real_100k",
            "beam_width": 4,
            "hop_rounds": 100,
            "benchmark_seed_variants": [{
              "name": "production",
              "seed_strategy": "persisted_head",
              "head_search_width": 32,
              "head_seed_count": 32,
              "neighbor_score_mode": "rabitq",
              "materialization_batch_size": 10
            }]
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");

        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command.windows(2).any(|window| {
            window
                == [
                    "--benchmark-seed-variant",
                    "production:persisted_head:32:32:rabitq:10:off:4:100:off",
                ]
        }));
    }

    #[test]
    fn distann_traversal_pair_with_implicit_search_shape_stays_pairable() {
        let raw = r#"{
          "name": "replica-default-shape",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "candidate-10k",
            "physical_benchmark": true,
            "materialization_correctness": true,
            "corpus_prefix": "ec_real_10k",
            "benchmark_seed_variants": [
              {
                "name": "owner",
                "seed_strategy": "persisted_head",
                "head_search_width": 32,
                "head_seed_count": 32,
                "neighbor_score_mode": "rabitq",
                "materialization_batch_size": 10
              },
              {
                "name": "replica",
                "seed_strategy": "persisted_head",
                "head_search_width": 32,
                "head_seed_count": 32,
                "neighbor_score_mode": "rabitq",
                "materialization_batch_size": 10,
                "traversal_replica": true
              }
            ]
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("effective defaults form a valid pair");
        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        for expected in [
            "owner:persisted_head:32:32:rabitq:10:off:4:100:off",
            "replica:persisted_head:32:32:rabitq:10:off:4:100:on",
        ] {
            assert!(command
                .windows(2)
                .any(|window| { window == ["--benchmark-seed-variant", expected] }));
        }
    }

    #[test]
    fn distann_local_multinode_expands_production_trained_head() {
        let raw = r#"{
          "name": "distann-production-head",
          "schema_version": 1,
          "steps": [{
            "kind": "distann-local-multinode",
            "name": "trained-100k",
            "physical_benchmark": true,
            "corpus_prefix": "ec_real_100k",
            "head_index_cap": 4096,
            "production_head_policy": "training_landmarks_exact",
            "training_query_path": "/staged/ec_real_100k_queries.tsv"
          }]
        }"#;
        let config: SuiteConfig = serde_json::from_str(raw).expect("suite parses");
        validate_config(&config).expect("suite validates");

        let command = config.steps[0]
            .expand(&config.defaults, &conn())
            .expect("step expands");
        assert!(command
            .windows(2)
            .any(|window| { window == ["--production-head-policy", "training_landmarks_exact"] }));
        assert!(command.windows(2).any(|window| {
            window == ["--training-query-path", "/staged/ec_real_100k_queries.tsv"]
        }));
        assert!(!command.iter().any(|argument| argument == "--head-policy"));
    }

    #[test]
    fn distann_local_multinode_expands_task183_training_policies() {
        for policy in ["training_region_balanced", "training_query_facility"] {
            let raw = format!(
                r#"{{
                  "name": "distann-task183-head",
                  "schema_version": 1,
                  "steps": [{{
                    "kind": "distann-local-multinode",
                    "name": "{policy}-100k",
                    "physical_benchmark": true,
                    "corpus_prefix": "ec_real_100k",
                    "head_index_cap": 4096,
                    "head_policy": "{policy}",
                    "training_query_path": "/staged/ec_real_100k_queries.tsv"
                  }}]
                }}"#
            );
            let config: SuiteConfig = serde_json::from_str(&raw).expect("suite parses");
            validate_config(&config).expect("suite validates");
            let command = config.steps[0]
                .expand(&config.defaults, &conn())
                .expect("step expands");
            assert!(command
                .windows(2)
                .any(|window| { window == ["--head-policy", policy] }));
            assert!(command.windows(2).any(|window| {
                window == ["--training-query-path", "/staged/ec_real_100k_queries.tsv"]
            }));
        }
    }

    #[test]
    fn distann_drop_extension_cleanup_is_structured() {
        let raw = "[distann-multicluster] physical_drop_extension_cleanup pass=true node=2 ready_before=1 published_before=1 hidden_before=8 hidden_after=0 extension_after=0 post_drop_dml_rows=1\n";
        let rows = parse_distann_multinode_rows(raw);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "drill_outcome");
        assert_eq!(
            rows[0].1.get("drill").map(String::as_str),
            Some("physical_drop_extension_cleanup")
        );
        assert_eq!(rows[0].1.get("pass").map(String::as_str), Some("true"));
    }

    #[test]
    fn distann_local_multinode_rejects_out_of_range_head_index_cap() {
        let step: SuiteStep = serde_json::from_str(
            r#"{
              "kind": "distann-local-multinode",
              "name": "invalid-cap",
              "head_index_cap": 15
            }"#,
        )
        .expect("step parses");

        assert!(step.validate().is_err());
    }

    #[test]
    fn distann_local_multinode_rejects_out_of_range_search_shape() {
        for field in [
            r#""beam_width": 257"#,
            r#""hop_rounds": 257"#,
            r#""head_search_width": 4097"#,
            r#""head_seed_count": 0"#,
            r#""seed_strategy": "unknown""#,
        ] {
            let step: SuiteStep = serde_json::from_str(&format!(
                r#"{{
                  "kind": "distann-local-multinode",
                  "name": "invalid-search-shape",
                  {field}
                }}"#
            ))
            .expect("step parses");
            assert!(step.validate().is_err());
        }
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
                result_identity_output: Some("${artifact_dir}/profile-identity.jsonl".into()),
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
                PathBuf::from("artifacts/current/profile-identity.jsonl"),
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
        assert!(args.windows(2).any(|w| w
            == [
                "--result-identity-output",
                "artifacts/current/profile-identity.jsonl"
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
            sample_backend_memory: Some(true),
            memory_sample_interval_ms: Some(10),
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
        assert!(args.contains(&"--sample-backend-memory".into()));
        assert!(args
            .windows(2)
            .any(|w| w == ["--memory-sample-interval-ms", "10"]));
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
                sample_backend_memory: None,
                memory_sample_interval_ms: None,
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
    fn failed_distann_step_retains_primary_log_for_result_extraction() {
        let step = StepRecord {
            name: "physical-50k".into(),
            kind: "distann-local-multinode".into(),
            command: vec![
                "dev".into(),
                "distann-multicluster".into(),
                "local-multinode-pg18".into(),
                "--log-file".into(),
                "artifacts/physical-50k/distann-local-multinode.log".into(),
            ],
            selected: true,
            quant: None,
            isa: None,
            kernel_status: None,
            pgoptions: None,
            tags: Vec::new(),
            nfr_021_registrations: Vec::new(),
            expected_artifacts: vec!["artifacts/physical-50k/distann-multinode-summary.log".into()],
            status: Some(StepStatus::Failed),
            started_at_unix_ms: Some(1),
            finished_at_unix_ms: Some(2),
            duration_ms: Some(1),
            exit_code: Some(1),
            parallel_workers_before: None,
            parallel_workers_after: None,
            parallel_workers_delta: None,
        };

        assert_eq!(
            result_artifacts_for_step(&step),
            vec![
                "artifacts/physical-50k/distann-multinode-summary.log",
                "artifacts/physical-50k/distann-local-multinode.log",
            ]
        );
    }

    #[test]
    fn parallel_worker_counter_emits_result_row() {
        let manifest = SuiteManifest {
            suite: "suite".into(),
            schema_version: 1,
            config: "suite.json".into(),
            config_sha256: "abc".into(),
            runner_git_commit: None,
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
                nfr_021_registrations: Vec::new(),
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
            sample_backend_memory: None,
            memory_sample_interval_ms: None,
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
            concurrency_sweep: vec![1, 2, 4, 8, 16],
            iterations: Some(10),
            worker_batch_size: Some(5),
            rerank_width: None,
            adaptive_nprobe: None,
            adaptive_nprobe_score_gap_micros: None,
            adaptive_nprobe_score_margin_ratio_bps: None,
            ivf_scratch_soa_batch_decode: None,
            ivf_stage_counters: None,
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
            .any(|w| w == ["--concurrency-sweep", "1,2,4,8,16"]));
        assert!(args.windows(2).any(|w| w == ["--worker-batch-size", "5"]));
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
            result_identity_output: Some("identity.jsonl".into()),
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
            .any(|w| w == ["--result-identity-output", "identity.jsonl"]));
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
                result_identity_output: None,
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
            runner_git_commit: None,
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
            nfr_021_registrations: Vec::new(),
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
            runner_git_commit: None,
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
            nfr_021_registrations: Vec::new(),
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
    fn parses_loader_build_memory_rows() {
        let rows = parse_load_rows(
            "[loader] build_memory index=d8_idx backend_pid=42 rss_before_kb=100 hwm_before_kb=120 rss_peak_kb=900 hwm_peak_kb=950 samples=31 sample_interval_ms=25\n",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "build_memory");
        assert_eq!(rows[0].1.get("index").map(String::as_str), Some("d8_idx"));
        assert_eq!(
            rows[0].1.get("hwm_peak_kb").map(String::as_str),
            Some("950")
        );
    }

    #[test]
    fn parses_distann_shard_build_notice() {
        let rows = parse_load_rows(
            "[postgres notice] ec_distann sharded build: shards=4 duplication_factor=1.2 max_shard_size=4000 stitch_edges_before_prune=10 stitch_edges_after_prune=9 stitch_peak_union_len=64 shard_output_spill_bytes=2000 stitch_peak_cursor_bytes=34000 stitch_peak_group_bytes=1000 stitch_peak_retained_bytes=35000 build_peak_completion_bytes=1500 reachability_repairs=0\n",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "distann_shard_build");
        assert_eq!(
            rows[0]
                .1
                .get("build_peak_completion_bytes")
                .map(String::as_str),
            Some("1500")
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
