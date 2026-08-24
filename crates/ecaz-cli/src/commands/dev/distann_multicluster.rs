//! `ec_distann` real multi-instance fixture (Task 165 M3, Slice A).
//!
//! The primary lane loads source rows only on the coordinator, creates empty
//! participant shells, and drives the Task 179 physical generation lifecycle.
//! The historical replicated-serving fixture remains available under an
//! explicit control-only subcommand.

use clap::{Args, Subcommand, ValueEnum};
use color_eyre::eyre::{bail, eyre, Context, Result};
use ecaz_fault_injection::ProviderMode;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::{Barrier, Mutex};

use crate::commands::bench::latency::{monitor_backend_memory, rss_slope_kb_per_second};

use super::support::{
    default_cluster_root, find_pgrx_install, repo_root, resolve_pgrx_home, run_status,
};

#[derive(Subcommand, Debug)]
pub enum DistannMulticlusterCommand {
    /// Spin up N real PG18 instances and build one physically sharded epoch.
    #[command(name = "local-multinode-pg18")]
    LocalMultinodePg18(LocalMultinodePg18Args),
    /// Historical replicated-serving control (not physical topology evidence).
    #[command(name = "replicated-serving-control-pg18")]
    ReplicatedServingControlPg18(LocalMultinodePg18Args),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FixtureMode {
    Physical,
    ReplicatedServingControl,
}

#[derive(Args, Debug)]
pub struct LocalMultinodePg18Args {
    #[arg(long, default_value_t = 18)]
    pub pg: u16,
    #[arg(long)]
    pub pgbin: Option<PathBuf>,
    #[arg(long)]
    pub pgrx_home: Option<PathBuf>,
    #[arg(long)]
    pub run_dir: Option<PathBuf>,
    #[arg(long)]
    pub artifact_dir: Option<PathBuf>,
    /// Reuse a stopped, attested physical fixture instead of rebuilding it.
    /// Build-affecting provenance must match; the default remains rebuild.
    #[arg(long, default_value_t = false)]
    pub reuse_fixture: bool,
    /// Packet-local or external directory containing the immutable benchmark
    /// provenance used to attest a reused fixture. When omitted, preserve the
    /// historical sibling-directory lookup.
    #[arg(long)]
    pub reuse_provenance_dir: Option<PathBuf>,
    /// Number of physical owners. Node 1 is also the coordinator unless
    /// `--coordinator-outside-roster` is set.
    #[arg(long, default_value_t = 3)]
    pub nodes: u32,
    /// Start one additional coordinator instance which is not a physical owner.
    #[arg(long, default_value_t = false)]
    pub coordinator_outside_roster: bool,
    /// Run same-data single-instance versus physical recall, latency, and
    /// storage measurements through the standard benchmark commands.
    #[arg(long, default_value_t = false)]
    pub physical_benchmark: bool,
    /// Query iterations per latency arm in physical benchmark mode.
    #[arg(long, default_value_t = 5)]
    pub benchmark_iterations: u32,
    /// Concurrent latency levels for the physical benchmark throughput curve.
    /// When set, this is passed to each distributed and single-index latency
    /// child as `--concurrency-sweep`.
    #[arg(long = "benchmark-concurrency-sweep", value_delimiter = ',')]
    pub benchmark_concurrency_sweep: Vec<usize>,
    /// Untimed queries on each latency worker before physical benchmark
    /// measurement. This warms backend-local head and transport caches.
    #[arg(long, default_value_t = 0)]
    pub benchmark_warmup_iterations: u32,
    /// Held-out query count for post-insert exact-ground-truth quality. An
    /// additional fixed 48 queries cover inserted neighborhoods.
    #[arg(long)]
    pub benchmark_parity_queries: Option<u32>,
    /// Shipped-default heldout deficit for this scale. When paired with
    /// `--task167-heldout-physical-sample-sd`, the heldout row becomes a
    /// baseline-relative regression gate. When both are omitted, the row is
    /// a non-blocking baseline observation.
    #[arg(long)]
    pub task167_heldout_baseline_deficit: Option<f64>,
    /// Sample standard deviation of the shipped-default physical heldout arm
    /// at this scale. The regression band is baseline deficit + 2 * sample SD.
    #[arg(long)]
    pub task167_heldout_physical_sample_sd: Option<f64>,
    /// Reconnect a latency worker after this many timed queries. Zero keeps
    /// one backend for the whole arm; nonzero values bound backend-local
    /// memory during long physical-query diagnostics and replay the untimed
    /// warmup before each fresh backend.
    #[arg(long, default_value_t = 0)]
    pub benchmark_backend_batch_size: u32,
    /// Keep the single physical latency backend in one explicit transaction
    /// for the timed queries. Diagnostic mode for transaction-lifetime memory
    /// retention; requires one backend for the whole arm.
    #[arg(long, default_value_t = false)]
    pub benchmark_hold_transaction: bool,
    /// Run repeated benchmark-only physical seed-coverage calls in one
    /// transaction and fail on a positive RSS slope. This is the executable
    /// regression gate for Task 200.
    #[arg(long)]
    pub coverage_memory_regression_iterations: Option<u32>,
    /// Task 185 benchmark-only per-seed gateway/basin provenance. Writes one
    /// compact JSON trace per physical seed variant under --artifact-dir.
    #[arg(long, default_value_t = false)]
    pub gateway_trace: bool,
    /// Task 227 benchmark-only per-query traversal/frontier/exact-result trace
    /// over the selected evaluation query slice.
    #[arg(long, default_value_t = false)]
    pub query_trace: bool,
    /// Task 185 benchmark-only isolated attribution for each returned seed
    /// position. This is intentionally more expensive than gateway_trace.
    #[arg(long, default_value_t = false)]
    pub gateway_isolated_trace: bool,
    /// Maximum returned seed positions to isolate per training query. Omit to
    /// isolate the complete configured seed list.
    #[arg(long)]
    pub gateway_isolated_seed_limit: Option<u32>,
    /// Task 185 benchmark-only arbitrary persisted-head candidate attribution.
    /// The positions are 1-based and comma-separated; each selected head
    /// member is traced independently on the disjoint training slice.
    #[arg(long, default_value_t = false)]
    pub gateway_head_candidate_trace: bool,
    /// Persisted-head positions to trace when gateway_head_candidate_trace is
    /// enabled. This is explicit to keep the diagnostic bounded.
    #[arg(long, value_delimiter = ',')]
    pub gateway_head_candidate_positions: Vec<u32>,
    /// Maximum allowed RSS slope for the Task 200 coverage regression gate.
    #[arg(long, default_value_t = 100.0)]
    pub coverage_memory_regression_max_slope_kb_per_s: f64,
    /// Maximum RSS peak-to-trough range after the one-call warm-up phase.
    #[arg(long, default_value_t = 4096.0)]
    pub coverage_memory_regression_max_delta_kb: f64,
    /// Sample the latency backend's RSS/HWM series at a fixed interval.
    #[arg(long, default_value_t = false)]
    pub sample_backend_memory: bool,
    /// Milliseconds between backend RSS/HWM samples.
    #[arg(long, default_value_t = 25)]
    pub memory_sample_interval_ms: u64,
    /// Run only the physical latency attribution arms. This skips the
    /// duplicate single-index build and recall matrix when those already
    /// exist in the owning packet.
    #[arg(long, default_value_t = false)]
    pub stage_counter_only: bool,
    /// Skip the recall child for a latency-only memory reproduction.
    #[arg(long, default_value_t = false)]
    pub skip_recall: bool,
    /// Skip building and measuring the single-index control arm.
    #[arg(long, default_value_t = false)]
    pub skip_single_control: bool,
    /// Keep the single-index control available for storage and insert A/B,
    /// but skip its duplicate recall/latency arms. Large physical indexes can
    /// make that redundant control query dominate the matrix wall time.
    #[arg(long, default_value_t = false)]
    pub skip_single_benchmark: bool,
    /// Task 183 benchmark-only per-stage attribution for the physical latency
    /// arm. Requires a measurement extension exposing the stage snapshot API.
    #[arg(long, default_value_t = false)]
    pub distann_stage_counters: bool,
    /// Task 184 suite-driven semantic matrix for eager versus ranked-window
    /// materialization. Requires two otherwise-identical benchmark variants
    /// with batch sizes zero and ten.
    #[arg(long, default_value_t = false)]
    pub materialization_correctness: bool,
    /// Run the Task 199 armed LD_PRELOAD ENOSPC replica-build drill.
    #[arg(long, default_value_t = false)]
    pub traversal_replica_enospc_drill: bool,
    /// First TCP port; node k listens on base_port + (k - 1).
    #[arg(long, default_value_t = 39710)]
    pub base_port: u16,
    /// Deterministic corpus row count (per node; replicated).
    #[arg(long, default_value_t = 2000)]
    pub rows: u32,
    /// Vector dimension.
    #[arg(long, default_value_t = 16)]
    pub dim: u32,
    /// ec_distann graph degree reloption.
    #[arg(long, default_value_t = 32)]
    pub graph_degree: u32,
    /// Number of partition-local Vamana builds used for the physical head
    /// union. Zero selects the extension's automatic policy; one preserves
    /// the monolithic control; values >=2 exercise Task 207 construction.
    #[arg(long, default_value_t = 1)]
    pub build_shards: u32,
    /// Task 207 head construction A/B, independent of the sharded graph.
    #[arg(long, default_value = "stitched_bfs")]
    pub head_construction: String,
    /// Persisted coordinator head-sample cap reloption. Exposed so FR-080
    /// sensitivity matrices can vary the cap through `ecaz bench suite`.
    #[arg(long, default_value_t = 4096)]
    pub head_index_cap: u32,
    /// Task 211 head sizing law rate.
    #[arg(long)]
    pub head_sampling_rate: Option<f64>,
    /// Task 211 head sizing law lower clamp.
    #[arg(long)]
    pub head_cap_floor: Option<u32>,
    /// Task 211 head sizing law upper clamp.
    #[arg(long)]
    pub head_cap_ceiling: Option<u32>,
    /// Session beam width applied to both physical and single benchmark arms.
    #[arg(long)]
    pub beam_width: Option<u32>,
    /// FR-081 retained candidate heap size L applied to benchmark query arms.
    #[arg(long)]
    pub candidate_heap_limit: Option<u32>,
    /// Session hop-round cap applied to both benchmark arms. Together with
    /// beam_width this makes fixed-product BW/H A/B runs suite-addressable.
    #[arg(long)]
    pub hop_rounds: Option<u32>,
    /// Task 210 P2a: build and serve the FR-080 head as roster shards. Sets
    /// `ec_distann.shard_head_storage` before the build so the coordinator
    /// persists landmark ids only, and `ec_distann.sharded_head_search` on the
    /// benchmark arms so every owner searches the landmarks it already holds.
    #[arg(long, default_value_t = false)]
    pub sharded_head: bool,
    /// Task 210 P2b: additional roster nodes that may serve a head shard
    /// (DISTRIBUTEDANN 4.1). Requires --sharded-head.
    #[arg(long)]
    pub head_replica_count: Option<u32>,
    /// Task 210 P3: TRAV-30 bounded gateway copy capacity. Sets
    /// `ec_distann.gateway_copy_capacity` on the physical benchmark arms so
    /// the coordinator caches that many head landmarks' routing payloads.
    #[arg(long)]
    pub gateway_copy_capacity: Option<u32>,
    /// Task 212 bounded deterministic crown-cache capacity on physical scans.
    #[arg(long)]
    pub crown_capacity: Option<u32>,
    /// Task 212 conservative width pruning using the crown cache.
    #[arg(long, default_value_t = false)]
    pub crown_width_pruning: bool,
    /// Task 213 fused head-hop seed expansion using crown-ranked seeds.
    #[arg(long, default_value_t = false)]
    pub fused_head_hop: bool,
    /// Control arm for head-sharding A/Bs now that the sharded head is the
    /// shipped default (fe5822f46): forces the legacy coordinator-local head
    /// by setting `ec_distann.shard_head_storage=off` on the building session
    /// and `ec_distann.sharded_head_search=off` on the physical arms.
    #[arg(long, default_value_t = false, conflicts_with = "sharded_head")]
    pub local_head: bool,
    /// Task 180 benchmark-only physical seed mode. Requires an extension build
    /// with `distann-head-attribution-benchmark` when set.
    #[arg(long)]
    pub seed_strategy: Option<String>,
    /// Task 180 benchmark-only approximate head-search width, independent of
    /// the distributed beam width.
    #[arg(long)]
    pub head_search_width: Option<u32>,
    /// Task 180 benchmark-only number of head candidates returned as seeds.
    #[arg(long)]
    pub head_seed_count: Option<u32>,
    /// Task 180 benchmark-only traversal scoring mode.
    #[arg(long)]
    pub neighbor_score_mode: Option<String>,
    /// Task 181 benchmark-only deterministic landmark construction policy.
    #[arg(long)]
    pub head_policy: Option<String>,
    /// Task 182 production generation policy. Unlike `--head-policy`, this
    /// drives immutable generation metadata and requires no benchmark feature.
    #[arg(long)]
    pub production_head_policy: Option<String>,
    /// Server-readable TSV containing at least 400 held-out queries. Rows
    /// 201-400 are the disjoint Task 181 training slice.
    #[arg(long)]
    pub training_query_path: Option<PathBuf>,
    /// Task 180 benchmark-only seed variants evaluated against one immutable
    /// physical generation. Repeat as
    /// NAME:MODE:SEARCH_WIDTH:SEED_COUNT:NEIGHBOR_SCORE_MODE with an optional
    /// sixth MATERIALIZATION_BATCH_SIZE field (zero preserves eager behavior)
    /// and optional OWNER_VALIDATION_CACHE on/off field for Task 192.
    #[arg(long = "benchmark-seed-variant")]
    pub benchmark_seed_variants: Vec<String>,
    /// Assert byte-identical per-query predictions for two physical runtime
    /// arms that share one immutable generation. The value is
    /// CONTROL_VARIANT,CANDIDATE_VARIANT.
    #[arg(long)]
    pub same_generation_recall_pair: Option<String>,
    /// Query count for the recall comparison.
    #[arg(long, default_value_t = 50)]
    pub queries: u32,
    /// Zero-based row offset into the staged query TSV. The fixture loads only
    /// this exact slice, keeping recall, latency, and trace artifacts aligned
    /// without copying query data into a review packet.
    #[arg(long, default_value_t = 0)]
    pub query_offset: u32,
    /// top-k for the recall comparison.
    #[arg(long, default_value_t = 10)]
    pub top_k: u32,
    /// Keep the instances running after the run (for manual inspection).
    #[arg(long, default_value_t = false)]
    pub keep_running: bool,
    /// Run only the multi-node distinct-recall gates (skip the TC-042 fault
    /// matrix + FR-082 lifecycle drills). Used by the `distann-local-multinode`
    /// suite step to package the scaled distinct_recall evidence without the
    /// (expensive at scale) per-drill re-setups.
    #[arg(long, default_value_t = false)]
    pub skip_fault_drills: bool,
    /// Skip the expensive concurrent insert/query drill after the benchmark
    /// matrix. Used for large-scale measurement arms when the dedicated
    /// bounded concurrency gate is run separately.
    #[arg(long, default_value_t = false)]
    pub skip_concurrency_drill: bool,
    /// Permit a unanimous non-release extension profile for intentional short
    /// diagnostic fixtures. The default benchmark contract requires release.
    #[arg(long, default_value_t = false)]
    pub allow_debug_extension: bool,
    /// Insert bounded candidates until one coordinator-routed remote owner
    /// commit is observed, for protocol validation and review evidence.
    #[arg(long, default_value_t = false)]
    pub remote_insert_probe: bool,
    /// After the physical fixture passes, drop the extension on every owner
    /// and prove AM-owned generation relations are dependency-cleaned and the
    /// preloaded hooks pass through ordinary DML without the extension.
    #[arg(long, default_value_t = false)]
    pub drop_extension_cleanup_drill: bool,
    /// Load a real staged corpus instead of the synthetic deterministic corpus.
    /// When set, each node loads `{staged_dir}/{corpus_prefix}_corpus.tsv` into
    /// `dm` (encoded with the standard `encode_to_ecvector(source, 4, 42)`) and
    /// `{corpus_prefix}_queries.tsv` into `dm_queries`, so the recall gate runs
    /// against real DBpedia vectors + held-out queries (Task 172 real-corpus
    /// distributed quality lane, not a synthetic identity smoke).
    #[arg(long)]
    pub corpus_prefix: Option<String>,
    /// Directory holding the staged corpus TSVs (default: repo
    /// `data/staged-current`). Only used with `--corpus-prefix`.
    #[arg(long)]
    pub staged_dir: Option<PathBuf>,
    /// Additional session GUCs applied to physical benchmark child commands.
    /// This carries packet-local instrumentation such as per-round notices.
    #[arg(long = "bench-session-guc")]
    pub bench_session_gucs: Vec<String>,
    /// Inject one exact-peer provider fault into the first remote owner query.
    #[arg(long, value_enum)]
    pub remote_socket_fault: Option<RemoteSocketFaultArg>,
    /// Per-operation delay for --remote-socket-fault slow.
    #[arg(long, default_value_t = 25)]
    pub remote_socket_fault_latency_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum RemoteSocketFaultArg {
    Reset,
    Slow,
}

impl RemoteSocketFaultArg {
    fn provider_mode(self) -> ProviderMode {
        match self {
            Self::Reset => ProviderMode::SocketReset,
            Self::Slow => ProviderMode::SocketSlow,
        }
    }
}

impl DistannMulticlusterCommand {
    pub async fn run(&self) -> Result<()> {
        match self {
            DistannMulticlusterCommand::LocalMultinodePg18(args) => {
                run_local_multinode_pg18(args, FixtureMode::Physical).await
            }
            DistannMulticlusterCommand::ReplicatedServingControlPg18(args) => {
                run_local_multinode_pg18(args, FixtureMode::ReplicatedServingControl).await
            }
        }
    }
}

struct Node {
    node_id: u32,
    port: u16,
    data_dir: PathBuf,
    log_file: PathBuf,
}

#[derive(Debug)]
struct Task199EnospcFixture {
    tablespace_dir: PathBuf,
    arm_file: PathBuf,
    marker_file: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExtensionProvenance {
    node_id: u32,
    port: u16,
    git_sha: String,
    build_profile: String,
    features: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExtensionPreflight {
    git_sha: String,
    build_profile: String,
    nodes: usize,
    debug_override: bool,
    features: String,
}

fn validate_extension_preflight(
    observed: &[ExtensionProvenance],
    allow_debug_extension: bool,
) -> Result<ExtensionPreflight> {
    let expected = observed
        .first()
        .ok_or_else(|| color_eyre::eyre::eyre!("extension preflight observed no nodes"))?;
    if expected.git_sha.trim().is_empty() {
        bail!(
            "extension preflight returned an empty git SHA on node {} port {}",
            expected.node_id,
            expected.port
        );
    }
    for node in observed.iter().skip(1) {
        if node.git_sha != expected.git_sha
            || node.build_profile != expected.build_profile
            || node.features != expected.features
        {
            bail!(
                "extension provenance mismatch on node {} port {}: expected {}/{}/{}, observed {}/{}/{}",
                node.node_id,
                node.port,
                expected.git_sha,
                expected.build_profile,
                expected.features,
                node.git_sha,
                node.build_profile,
                node.features
            );
        }
    }
    if expected.build_profile != "release" && !allow_debug_extension {
        bail!(
            "extension preflight rejected non-release profile on node {} port {}: observed {}/{}; reinstall a release extension or set the suite step's allow_debug_extension=true for an intentional diagnostic run",
            expected.node_id,
            expected.port,
            expected.git_sha,
            expected.build_profile
        );
    }
    if expected
        .features
        .split(',')
        .any(|feature| feature == "pg-test")
        && !allow_debug_extension
    {
        bail!(
            "extension preflight rejected pg-test feature on node {} port {}: observed {}/{}/{}; install with --no-default-features --features pg18 for production evidence or explicitly allow a diagnostic build",
            expected.node_id,
            expected.port,
            expected.git_sha,
            expected.build_profile,
            expected.features
        );
    }
    Ok(ExtensionPreflight {
        git_sha: expected.git_sha.clone(),
        build_profile: expected.build_profile.clone(),
        nodes: observed.len(),
        debug_override: allow_debug_extension
            && (expected.build_profile != "release"
                || expected
                    .features
                    .split(',')
                    .any(|feature| feature == "pg-test")),
        features: expected.features.clone(),
    })
}

fn validate_query_stage_counter_feature(requested: bool, features: &str) -> Result<()> {
    if requested
        && !features
            .split(',')
            .any(|feature| feature == "distann-head-attribution-benchmark")
    {
        bail!(
            "--distann-stage-counters requires extension feature \
             distann-head-attribution-benchmark; observed features {features}. \
             Task 167 insert-work counters are collected independently"
        );
    }
    Ok(())
}

async fn preflight_fixture_extensions(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    allow_debug_extension: bool,
) -> Result<ExtensionPreflight> {
    let mut observed = Vec::with_capacity(nodes.len());
    for node in nodes {
        run_psql_file(
            psql,
            socket_dir,
            node.port,
            "CREATE EXTENSION IF NOT EXISTS ecaz",
        )
        .await
        .wrap_err_with(|| {
            format!(
                "loading extension on node {} port {}",
                node.node_id, node.port
            )
        })?;
        let (client, connection) =
            tokio_postgres::connect(&conninfo(socket_dir, node.port), tokio_postgres::NoTls)
                .await
                .wrap_err_with(|| {
                    format!(
                        "connecting to node {} port {} for extension preflight",
                        node.node_id, node.port
                    )
                })?;
        let connection_task = tokio::spawn(async move { connection.await });
        let row = client
            .query_one(
                "SELECT ecaz_build_git_sha(), ecaz_build_profile(), ecaz_build_features()",
                &[],
            )
            .await
            .wrap_err_with(|| {
                format!(
                    "querying extension provenance on node {} port {}",
                    node.node_id, node.port
                )
            })?;
        observed.push(ExtensionProvenance {
            node_id: node.node_id,
            port: node.port,
            git_sha: row.get(0),
            build_profile: row.get(1),
            features: row.get(2),
        });
        connection_task.abort();
    }
    let preflight = validate_extension_preflight(&observed, allow_debug_extension)?;
    crate::ecaz_println!(
        "[distann-multicluster] release_profile_preflight status=passed nodes={} unanimous=true extension_git_sha={} extension_build_profile={} extension_features={} debug_override={}",
        preflight.nodes,
        preflight.git_sha,
        preflight.build_profile,
        preflight.features,
        preflight.debug_override
    );
    Ok(preflight)
}

async fn run_local_multinode_pg18(args: &LocalMultinodePg18Args, mode: FixtureMode) -> Result<()> {
    if args.pg != 18 {
        bail!("distann local-multinode requires --pg 18, got {}", args.pg);
    }
    if args.nodes == 0 || (mode == FixtureMode::ReplicatedServingControl && args.nodes < 2) {
        bail!(
            "distann local-multinode has invalid owner count {}",
            args.nodes
        );
    }
    if mode == FixtureMode::ReplicatedServingControl && args.coordinator_outside_roster {
        bail!("replicated-serving-control does not support an outside coordinator");
    }
    if args.remote_socket_fault.is_some() && mode != FixtureMode::Physical {
        bail!("--remote-socket-fault requires the physical fixture");
    }
    if args.remote_socket_fault.is_some() && !args.coordinator_outside_roster && args.nodes < 2 {
        bail!("--remote-socket-fault requires at least one remote owner");
    }
    if args.remote_socket_fault == Some(RemoteSocketFaultArg::Slow)
        && args.remote_socket_fault_latency_ms == 0
    {
        bail!("--remote-socket-fault-latency-ms must be at least 1");
    }
    if args.physical_benchmark && args.corpus_prefix.is_none() {
        bail!("--physical-benchmark requires --corpus-prefix");
    }
    if args.queries == 0 {
        bail!("--queries must be at least 1");
    }
    args.query_offset
        .checked_add(args.queries)
        .ok_or_else(|| eyre!("--query-offset + --queries overflows u32"))?;
    if args.query_offset > 0 && args.corpus_prefix.is_none() {
        bail!("--query-offset requires --corpus-prefix");
    }
    if args.query_offset > 0 && args.reuse_fixture {
        bail!("--query-offset cannot combine with --reuse-fixture");
    }
    if args.distann_stage_counters && !args.physical_benchmark {
        bail!("--distann-stage-counters requires --physical-benchmark");
    }
    if args.gateway_trace && !args.physical_benchmark {
        bail!("--gateway-trace requires --physical-benchmark");
    }
    if args.query_trace && !args.physical_benchmark {
        bail!("--query-trace requires --physical-benchmark");
    }
    if args.gateway_isolated_trace && !args.physical_benchmark {
        bail!("--gateway-isolated-trace requires --physical-benchmark");
    }
    if args.gateway_head_candidate_trace && !args.physical_benchmark {
        bail!("--gateway-head-candidate-trace requires --physical-benchmark");
    }
    if args.gateway_trace && args.training_query_path.is_none() {
        bail!(
            "--gateway-trace requires --training-query-path so attribution uses the disjoint training slice"
        );
    }
    if args.gateway_isolated_trace && args.training_query_path.is_none() {
        bail!(
            "--gateway-isolated-trace requires --training-query-path so attribution uses the disjoint training slice"
        );
    }
    if args.gateway_head_candidate_trace && args.training_query_path.is_none() {
        bail!(
            "--gateway-head-candidate-trace requires --training-query-path so attribution uses the disjoint training slice"
        );
    }
    if args.gateway_head_candidate_trace && args.gateway_head_candidate_positions.is_empty() {
        bail!("--gateway-head-candidate-trace requires --gateway-head-candidate-positions");
    }
    if args
        .gateway_head_candidate_positions
        .iter()
        .any(|position| !(1..=4096).contains(position))
    {
        bail!("--gateway-head-candidate-positions values must be in 1..=4096");
    }
    if !args.gateway_head_candidate_trace && !args.gateway_head_candidate_positions.is_empty() {
        bail!("--gateway-head-candidate-positions requires --gateway-head-candidate-trace");
    }
    if let Some(limit) = args.gateway_isolated_seed_limit {
        if !(1..=4096).contains(&limit) {
            bail!("--gateway-isolated-seed-limit must be in 1..=4096");
        }
        if !args.gateway_isolated_trace {
            bail!("--gateway-isolated-seed-limit requires --gateway-isolated-trace");
        }
    }
    if args.stage_counter_only && (!args.physical_benchmark || !args.distann_stage_counters) {
        bail!("--stage-counter-only requires --physical-benchmark and --distann-stage-counters");
    }
    if args.stage_counter_only && args.materialization_correctness {
        bail!("--stage-counter-only cannot combine with --materialization-correctness");
    }
    if args.materialization_correctness && !args.physical_benchmark {
        bail!("--materialization-correctness requires --physical-benchmark");
    }
    if args.materialization_correctness && args.coordinator_outside_roster {
        bail!("--materialization-correctness requires the coordinator to be physical owner zero");
    }
    if args.benchmark_iterations == 0 {
        bail!("--benchmark-iterations must be at least 1");
    }
    if !args.benchmark_concurrency_sweep.is_empty() {
        if !args.physical_benchmark {
            bail!("--benchmark-concurrency-sweep requires --physical-benchmark");
        }
        if args
            .benchmark_concurrency_sweep
            .iter()
            .any(|value| *value == 0)
        {
            bail!("--benchmark-concurrency-sweep values must all be at least 1");
        }
        let unique = args
            .benchmark_concurrency_sweep
            .iter()
            .collect::<HashSet<_>>();
        if unique.len() != args.benchmark_concurrency_sweep.len() {
            bail!("--benchmark-concurrency-sweep values must be unique");
        }
    }
    if args.benchmark_hold_transaction && args.benchmark_backend_batch_size != 0 {
        bail!("--benchmark-hold-transaction requires --benchmark-backend-batch-size 0");
    }
    if let Some(iterations) = args.coverage_memory_regression_iterations {
        if iterations < 20 {
            bail!("--coverage-memory-regression-iterations must be at least 20");
        }
        if !args.physical_benchmark {
            bail!("--coverage-memory-regression-iterations requires --physical-benchmark");
        }
        if !args
            .coverage_memory_regression_max_slope_kb_per_s
            .is_finite()
            || args.coverage_memory_regression_max_slope_kb_per_s < 0.0
        {
            bail!(
                "--coverage-memory-regression-max-slope-kb-per-s must be finite and non-negative"
            );
        }
        if !args.coverage_memory_regression_max_delta_kb.is_finite()
            || args.coverage_memory_regression_max_delta_kb < 0.0
        {
            bail!("--coverage-memory-regression-max-delta-kb must be finite and non-negative");
        }
    }
    if args.sample_backend_memory && args.memory_sample_interval_ms == 0 {
        bail!("--memory-sample-interval-ms must be at least 1");
    }
    if args.reuse_fixture && mode != FixtureMode::Physical {
        bail!("--reuse-fixture requires the physical fixture");
    }
    if args.reuse_fixture
        && (args.traversal_replica_enospc_drill
            || args.drop_extension_cleanup_drill
            || args.materialization_correctness)
    {
        bail!("--reuse-fixture cannot combine with fixture-mutating drills");
    }
    if !(16..=1_048_576).contains(&args.head_index_cap) {
        bail!("--head-index-cap must be in 16..=1048576");
    }
    if args.build_shards > 4096 {
        bail!("--build-shards must be in 0..=4096");
    }
    if !matches!(
        args.head_construction.as_str(),
        "stitched_bfs" | "partition_union"
    ) {
        bail!("--head-construction must be stitched_bfs or partition_union");
    }
    if args.head_sampling_rate.is_none()
        && (args.head_cap_floor.is_some() || args.head_cap_ceiling.is_some())
    {
        bail!("--head-cap-floor/--head-cap-ceiling require --head-sampling-rate");
    }
    if let Some(rate) = args.head_sampling_rate {
        if !rate.is_finite() || rate < 0.0 {
            bail!("--head-sampling-rate must be finite and non-negative");
        }
        let floor = args.head_cap_floor.unwrap_or(4096);
        let ceiling = args.head_cap_ceiling.unwrap_or(1_048_576);
        if !(16..=1_048_576).contains(&floor) || !(16..=1_048_576).contains(&ceiling) {
            bail!("--head-cap-floor/--head-cap-ceiling must be in 16..=1048576");
        }
        if floor > ceiling {
            bail!("--head-cap-floor must not exceed --head-cap-ceiling");
        }
    }
    if args.crown_capacity.is_some_and(|value| value > 1_048_576) {
        bail!("--crown-capacity must be in 0..=1048576");
    }
    if (args.crown_width_pruning || args.fused_head_hop) && args.crown_capacity.unwrap_or(0) == 0 {
        bail!("--crown-width-pruning/--fused-head-hop require --crown-capacity >= 1");
    }
    if (args.crown_width_pruning || args.fused_head_hop) && !args.physical_benchmark {
        bail!("--crown-width-pruning/--fused-head-hop require --physical-benchmark");
    }
    if args
        .beam_width
        .is_some_and(|value| !(1..=256).contains(&value))
    {
        bail!("--beam-width must be in 1..=256");
    }
    if args
        .candidate_heap_limit
        .is_some_and(|value| !(1..=4096).contains(&value))
    {
        bail!("--candidate-heap-limit must be in 1..=4096");
    }
    if args.candidate_heap_limit.is_some() && !args.physical_benchmark {
        bail!("--candidate-heap-limit requires --physical-benchmark");
    }
    if args
        .hop_rounds
        .is_some_and(|value| !(1..=256).contains(&value))
    {
        bail!("--hop-rounds must be in 1..=256");
    }
    if let Some(strategy) = args.seed_strategy.as_deref() {
        if !matches!(
            strategy,
            "persisted_head" | "head_sample_exact" | "head_hierarchy" | "owner_scan"
        ) {
            bail!("--seed-strategy must be persisted_head, head_sample_exact, head_hierarchy, or owner_scan");
        }
        if !args.physical_benchmark {
            bail!("--seed-strategy requires --physical-benchmark");
        }
    }
    if args
        .head_search_width
        .is_some_and(|value| !(1..=4096).contains(&value))
    {
        bail!("--head-search-width must be in 1..=4096");
    }
    if args
        .head_seed_count
        .is_some_and(|value| !(1..=4096).contains(&value))
    {
        bail!("--head-seed-count must be in 1..=4096");
    }
    if let Some(mode) = args.neighbor_score_mode.as_deref() {
        if !matches!(mode, "rabitq" | "exact_neighbor") {
            bail!("--neighbor-score-mode must be rabitq or exact_neighbor");
        }
        if !args.physical_benchmark {
            bail!("--neighbor-score-mode requires --physical-benchmark");
        }
    }
    if let Some(policy) = args.head_policy.as_deref() {
        if !matches!(
            policy,
            "current_sample"
                | "geometry_landmarks"
                | "graph_landmarks"
                | "training_landmarks"
                | "training_region_balanced"
                | "training_query_facility"
        ) {
            bail!("--head-policy must be current_sample, geometry_landmarks, graph_landmarks, training_landmarks, training_region_balanced, or training_query_facility");
        }
        if !args.physical_benchmark {
            bail!("--head-policy requires --physical-benchmark");
        }
        if policy.starts_with("training_") && args.training_query_path.is_none() {
            bail!("--head-policy training policies require --training-query-path");
        }
    }
    if let Some(policy) = args.production_head_policy.as_deref() {
        if !matches!(policy, "current_sample_graph" | "training_landmarks_exact") {
            bail!(
                "--production-head-policy must be current_sample_graph or training_landmarks_exact"
            );
        }
        if mode != FixtureMode::Physical {
            bail!("--production-head-policy requires the physical fixture");
        }
        if policy == "training_landmarks_exact" {
            if args.training_query_path.is_none() {
                bail!(
                    "--production-head-policy training_landmarks_exact requires --training-query-path"
                );
            }
            if args.head_index_cap != 4096 {
                bail!("training_landmarks_exact requires --head-index-cap 4096");
            }
        }
    }
    if args.head_policy.is_some() && args.production_head_policy.is_some() {
        bail!("--head-policy and --production-head-policy are mutually exclusive");
    }
    let training_path_expected = args
        .head_policy
        .as_deref()
        .is_some_and(|policy| policy.starts_with("training_"))
        || args.production_head_policy.as_deref() == Some("training_landmarks_exact");
    if args.training_query_path.is_some() != training_path_expected {
        bail!(
            "--training-query-path is required exactly for a training benchmark or production head policy"
        );
    }
    if !args.benchmark_seed_variants.is_empty() {
        if !args.physical_benchmark {
            bail!("--benchmark-seed-variant requires --physical-benchmark");
        }
        if args.seed_strategy.is_some()
            || args.head_search_width.is_some()
            || args.head_seed_count.is_some()
            || args.neighbor_score_mode.is_some()
        {
            bail!(
                "--benchmark-seed-variant cannot be combined with singular seed or neighbor-score controls"
            );
        }
        parse_benchmark_seed_variants(&args.benchmark_seed_variants)?;
    }
    if let Some(pair) = args.same_generation_recall_pair.as_deref() {
        let (control, candidate) = pair
            .split_once(',')
            .ok_or_else(|| eyre!("--same-generation-recall-pair must be CONTROL,CANDIDATE"))?;
        if control.is_empty() || candidate.is_empty() || control == candidate {
            bail!("--same-generation-recall-pair must name two distinct variants");
        }
        let names = args
            .benchmark_seed_variants
            .iter()
            .filter_map(|variant| variant.split(':').next())
            .collect::<HashSet<_>>();
        if !names.contains(control) || !names.contains(candidate) {
            bail!(
                "--same-generation-recall-pair variants must be present in --benchmark-seed-variant: {pair}"
            );
        }
    }
    let instance_count = args.nodes + u32::from(args.coordinator_outside_roster);
    let repo_root = repo_root()?;
    let pgbin = match args.pgbin.clone() {
        Some(path) => path,
        None => {
            let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
            find_pgrx_install(args.pg, &pgrx_home)?.bin_dir
        }
    };
    let pg_ctl = pgbin.join("pg_ctl");
    let psql = pgbin.join("psql");

    let run_dir = args
        .run_dir
        .clone()
        .unwrap_or_else(|| default_cluster_root().join("distann-local-multinode"));
    let mut socket_dir = run_dir.join("sockets");
    let mut log_dir = args
        .artifact_dir
        .clone()
        .unwrap_or_else(|| run_dir.join("logs"));
    if run_dir.exists() && !args.reuse_fixture {
        // Best-effort stop of a prior run before wiping.
        for k in 0..instance_count {
            let data_dir = run_dir.join(format!("node{}", k + 1));
            let _ = Command::new(&pg_ctl)
                .arg("-D")
                .arg(&data_dir)
                .arg("-m")
                .arg("immediate")
                .arg("stop")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await;
        }
        fs::remove_dir_all(&run_dir).wrap_err_with(|| format!("clearing {}", run_dir.display()))?;
    } else if args.reuse_fixture && !run_dir.exists() {
        bail!(
            "--reuse-fixture requested but run directory does not exist: {}",
            run_dir.display()
        );
    }
    fs::create_dir_all(&socket_dir)?;
    fs::create_dir_all(&log_dir)?;
    socket_dir = fs::canonicalize(&socket_dir)
        .wrap_err_with(|| format!("canonicalizing {}", socket_dir.display()))?;
    log_dir = fs::canonicalize(&log_dir)
        .wrap_err_with(|| format!("canonicalizing {}", log_dir.display()))?;
    let enospc_fixture = if args.traversal_replica_enospc_drill {
        if mode != FixtureMode::Physical {
            bail!("--traversal-replica-enospc-drill requires the physical fixture");
        }
        if !cfg!(target_os = "linux") {
            bail!("--traversal-replica-enospc-drill requires the Linux LD_PRELOAD provider");
        }
        if ecaz_fault_injection::provider_library_path().is_none() {
            bail!("Task 199 ENOSPC drill has no built fault-provider library");
        }
        let tablespace_dir = run_dir.join("task199-enospc-tablespace");
        fs::create_dir_all(&tablespace_dir)?;
        let tablespace_dir = fs::canonicalize(&tablespace_dir)?;
        let arm_file = fs::canonicalize(&run_dir)?.join("task199-enospc-provider.arm");
        let marker_file = log_dir.join("task199-enospc-provider.marker");
        fs::write(&marker_file, "")?;
        Some(Task199EnospcFixture {
            tablespace_dir,
            arm_file,
            marker_file,
        })
    } else {
        None
    };

    let nodes: Vec<Node> = (0..instance_count)
        .map(|k| Node {
            node_id: k + 1,
            port: args.base_port + k as u16,
            data_dir: run_dir.join(format!("node{}", k + 1)),
            log_file: log_dir.join(format!("node{}-postgres.log", k + 1)),
        })
        .collect();
    let remote_fault_marker = log_dir.join("distann-remote-socket-fault.marker");
    let remote_fault_arm = log_dir.join("distann-remote-socket-fault.arm");
    if args.remote_socket_fault.is_some() {
        let provider = ecaz_fault_injection::provider_library_path()
            .filter(|path| !path.contains("not built"))
            .ok_or_else(|| {
                color_eyre::eyre::eyre!(
                    "--remote-socket-fault requires the Linux LD_PRELOAD provider"
                )
            })?;
        if !Path::new(provider).is_file() {
            bail!("fault provider does not exist at {provider}");
        }
        let _ = fs::remove_file(&remote_fault_marker);
        let _ = fs::remove_file(&remote_fault_arm);
    }

    crate::ecaz_println!("[distann-multicluster] repo={}", repo_root.display());
    crate::ecaz_println!("[distann-multicluster] pgbin={}", pgbin.display());
    crate::ecaz_println!(
        "[distann-multicluster] mode={} owners={} instances={} coordinator_outside_roster={} base_port={} rows={} dim={} graph_degree={} build_shards={} head_index_cap={} query_offset={} queries={}",
        match mode {
            FixtureMode::Physical => "physical",
            FixtureMode::ReplicatedServingControl => "replicated-serving-control",
        },
        args.nodes,
        instance_count,
        args.coordinator_outside_roster,
        args.base_port,
        args.rows,
        args.dim,
        args.graph_degree,
        args.build_shards,
        args.head_index_cap,
        args.query_offset,
        args.queries,
    );

    // initdb + start + extension on every node. Reuse deliberately skips
    // initdb and starts the exact stopped PGDATA trees after provenance is
    // checked below; it never silently rebuilds a mismatched fixture.
    let physical_benchmark_startup_options = if args.physical_benchmark {
        // Large staged physical generations can spend more than ten minutes
        // in the owner-side head search before the coordinator receives the
        // first row.  Set both the extension budget and PostgreSQL's backend
        // default at server start so child benchmark sessions and the
        // backend-created owner sessions inherit the same measurement budget.
        " -c ec_distann.remote_statement_timeout_ms=3600000 -c statement_timeout=3600000"
    } else {
        ""
    };
    for node in &nodes {
        if args.reuse_fixture {
            if !node.data_dir.join("PG_VERSION").is_file() {
                bail!(
                    "--reuse-fixture found no PG_VERSION for node {} at {}",
                    node.node_id,
                    node.data_dir.display()
                );
            }
            continue;
        }
        let mut command = Command::new(&pg_ctl);
        command
            .arg("initdb")
            .arg("-D")
            .arg(&node.data_dir)
            .arg("-o")
            .arg("-A trust -U postgres")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        run_status(command)
            .await
            .wrap_err_with(|| format!("initdb node {}", node.node_id))?;
    }
    for node in &nodes {
        let mut command = Command::new(&pg_ctl);
        command
            .arg("-w")
            .arg("-D")
            .arg(&node.data_dir)
            .arg("-l")
            .arg(&node.log_file)
            .arg("-o")
            .arg(format!(
                "-p {} -c listen_addresses=127.0.0.1 -c unix_socket_directories='' \
                 -c shared_preload_libraries=ecaz -c max_prepared_transactions=32{}",
                node.port, physical_benchmark_startup_options
            ))
            .arg("start")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        for target in &nodes {
            let remote_conninfo = if args.physical_benchmark {
                format!(
                    "{} options=-cstatement_timeout=3600000",
                    conninfo(&socket_dir, target.port)
                )
            } else {
                conninfo(&socket_dir, target.port)
            };
            command.env(
                format!("EC_SPIRE_REMOTE_CONNINFO_DISTANN_NODE_{}", target.node_id),
                remote_conninfo,
            );
        }
        if node.node_id == 1 {
            if let Some(fault) = args.remote_socket_fault {
                let peer = format!("tcp:127.0.0.1:{}", nodes[1].port);
                let marker = remote_fault_marker.display().to_string();
                let arm_file = remote_fault_arm.display().to_string();
                for (name, value) in ecaz_fault_injection::provider_environment(
                    fault.provider_mode(),
                    "",
                    1,
                    (fault == RemoteSocketFaultArg::Slow)
                        .then_some(args.remote_socket_fault_latency_ms),
                    Some(&marker),
                    Some(&arm_file),
                    Some(&peer),
                ) {
                    command.env(name, value);
                }
            }
            if let Some(fixture) = enospc_fixture.as_ref() {
                let path_match = format!("pg_tblspc/|{}", fixture.tablespace_dir.to_string_lossy());
                let environment = ecaz_fault_injection::provider_environment(
                    ecaz_fault_injection::ProviderMode::EnospcWrite,
                    // PostgreSQL opens a tablespace relation through its
                    // PGDATA-relative pg_tblspc/<oid>/... symlink path, while
                    // /proc/self/fd resolves data-write targets through the
                    // symlink to this fixture's canonical directory.
                    &path_match,
                    1,
                    None,
                    Some(&fixture.marker_file.to_string_lossy()),
                    Some(&fixture.arm_file.to_string_lossy()),
                    None,
                );
                for (name, value) in environment {
                    command.env(name, value);
                }
            }
        }
        run_status(command)
            .await
            .wrap_err_with(|| format!("start node {}", node.node_id))?;
    }

    let result = async {
        // Task 197: extension load and release/SHA validation must precede all
        // corpus loading and physical generation construction. ecaz_println!
        // flushes stdout and the optional packet-local mirror on every line.
        let extension_preflight =
            preflight_fixture_extensions(&psql, &socket_dir, &nodes, args.allow_debug_extension)
                .await?;
        validate_query_stage_counter_feature(
            args.distann_stage_counters,
            &extension_preflight.features,
        )?;

        match mode {
            FixtureMode::Physical => {
                if args.reuse_fixture {
                    drive_reused_physical_fixture(
                        args,
                        &pg_ctl,
                        &psql,
                        &socket_dir,
                        &nodes,
                        log_dir.as_path(),
                        &extension_preflight,
                    )
                    .await
                } else {
                    drive_physical_fixture(
                        args,
                        &pg_ctl,
                        &psql,
                        &socket_dir,
                        &nodes,
                        log_dir.as_path(),
                        &extension_preflight,
                        enospc_fixture.as_ref(),
                    )
                    .await
                }
            }
            FixtureMode::ReplicatedServingControl => {
                drive_fixture(args, &pg_ctl, &psql, &socket_dir, &nodes, log_dir.as_path()).await
            }
        }
    }
    .await;

    if !args.keep_running {
        for node in &nodes {
            let _ = Command::new(&pg_ctl)
                .arg("-D")
                .arg(&node.data_dir)
                .arg("-m")
                .arg("fast")
                .arg("stop")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await;
        }
    } else {
        crate::ecaz_println!(
            "[distann-multicluster] instances left running under {}",
            run_dir.display()
        );
    }
    result
}

/// libpq conninfo for a node over the shared socket dir.
fn conninfo(_socket_dir: &Path, port: u16) -> String {
    format!("host=127.0.0.1 port={} dbname=postgres user=postgres", port,)
}

/// The identical, deterministic corpus + index setup run on every node.
/// Build the per-node corpus/index setup SQL: real staged corpus when
/// `--corpus-prefix` is set, else the synthetic deterministic corpus.
fn build_setup_sql(args: &LocalMultinodePg18Args) -> Result<String> {
    match &args.corpus_prefix {
        Some(prefix) => {
            let staged_dir = match &args.staged_dir {
                Some(dir) => dir.clone(),
                None => repo_root()?.join("data/staged-current"),
            };
            // Canonicalize (absolute + symlinks resolved) so the server-side
            // COPY can read the staged file regardless of the backend's cwd.
            let corpus_path =
                std::fs::canonicalize(staged_dir.join(format!("{prefix}_corpus.tsv")))
                    .wrap_err_with(|| format!("resolving staged corpus for prefix {prefix}"))?;
            let queries_path =
                std::fs::canonicalize(staged_dir.join(format!("{prefix}_queries.tsv")))
                    .wrap_err_with(|| format!("resolving staged queries for prefix {prefix}"))?;
            Ok(real_setup_sql(
                &corpus_path,
                &queries_path,
                args.query_offset,
                args.queries,
                args.graph_degree,
                args.head_index_cap,
                args.build_shards,
                &args.head_construction,
                &head_sizing_reloptions(args),
            ))
        }
        None => Ok(setup_sql(args)),
    }
}

/// A SQL `real[]` expression for a drill row's insert vector. In real mode it
/// reuses an existing corpus vector (guaranteed to match the index dimension);
/// in synthetic mode it reproduces the deterministic corpus generator at
/// `args.dim`. A synthetic `args.dim` vector does NOT match a real corpus, so
/// the drill inserts must not synthesize a vector when a real corpus is loaded.
fn insert_vector_expr(args: &LocalMultinodePg18Args, table: &str) -> String {
    if args.corpus_prefix.is_some() {
        format!("(SELECT source FROM {table} ORDER BY id LIMIT 1)")
    } else {
        synthetic_unit_vector_expr("7", args.dim)
    }
}

/// The `ecvector_distann_ip_ops` build/search distance assumes unit vectors.
/// Keep the deterministic synthetic fixture on that same contract so an exact
/// copied source is its own maximum-inner-product candidate.
fn synthetic_unit_vector_expr(row: &str, dimensions: u32) -> String {
    format!(
        "(SELECT array_agg((component / norm)::real ORDER BY d) \
           FROM (SELECT d, component, sqrt(sum(component * component) OVER ()) AS norm \
                   FROM (SELECT d, \
                                (sin(({row}) * 0.017 * (d + 1)) + \
                                 cos(({row}) * 0.0031 * (d + 1)))::double precision AS component \
                           FROM generate_series(0, {dimensions} - 1) AS d) raw) normalized)"
    )
}

/// Execute the synthetic generator in PostgreSQL and verify the contract the
/// distance operator relies on. Keeping this as a live SQL preflight catches
/// expression/type changes that a Rust string-shape assertion cannot.
async fn preflight_synthetic_unit_norm(
    coordinator: &tokio_postgres::Client,
    dimensions: u32,
) -> Result<f64> {
    const SAMPLES: u32 = 32;
    const MAX_ABS_ERROR: f64 = 1.0e-5;
    let vector = synthetic_unit_vector_expr("g", dimensions);
    let sql = format!(
        "SELECT max(abs(sqrt(norm_sq) - 1.0))::double precision
           FROM (
             SELECT g, sum(component::double precision * component::double precision) AS norm_sq
               FROM generate_series(1, {SAMPLES}) AS g
               CROSS JOIN LATERAL unnest({vector}) AS component
              GROUP BY g
           ) norms"
    );
    let max_abs_error = coordinator
        .query_one(&sql, &[])
        .await
        .wrap_err("running PostgreSQL synthetic unit-norm preflight")?
        .get::<_, Option<f64>>(0)
        .ok_or_else(|| eyre!("PostgreSQL synthetic unit-norm preflight returned no samples"))?;
    if !max_abs_error.is_finite() || max_abs_error > MAX_ABS_ERROR {
        bail!(
            "PostgreSQL synthetic unit-norm preflight failed: dimensions={dimensions} samples={SAMPLES} max_abs_error={max_abs_error} tolerance={MAX_ABS_ERROR}"
        );
    }
    Ok(max_abs_error)
}

/// Real staged-corpus load: COPY each 2-column TSV (`id\t[v1,v2,...]`) into a
/// text stage, convert the `[...]` JSON-array literal to a PG `{...}` array,
/// and materialize `dm(source real[], embedding ecvector)` +
/// `dm_queries(source real[])`. The `4, 42` encode params match both the
/// synthetic path and the standard suite load (`encode_to_ecvector(source, 4,
/// 42)`), so this lane is comparable to the single-node suite matrices.
fn real_setup_sql(
    corpus_path: &Path,
    queries_path: &Path,
    query_offset: u32,
    queries_limit: u32,
    gd: u32,
    head_index_cap: u32,
    build_shards: u32,
    head_construction: &str,
    head_sizing: &str,
) -> String {
    // Escape the paths as SQL string literals (double any single quote) so a
    // path containing `'` cannot break out of the COPY ... FROM '<path>' literal
    // (172-P2). Canonical repo paths are unlikely to contain one, but the COPY
    // literal must be robust to it.
    let corpus = corpus_path.display().to_string().replace('\'', "''");
    let queries = queries_path.display().to_string().replace('\'', "''");
    format!(
        "DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'peter') THEN CREATE ROLE peter LOGIN SUPERUSER; END IF; END $$;\n\
         DROP TABLE IF EXISTS dm; DROP TABLE IF EXISTS dm_queries;\n\
         CREATE TABLE dm (id bigint, source real[], embedding ecvector);\n\
         CREATE TEMP TABLE dm_stage (id bigint, vec text);\n\
         COPY dm_stage (id, vec) FROM '{corpus}' WITH (FORMAT text, DELIMITER E'\\t');\n\
         INSERT INTO dm\n\
           SELECT id, translate(vec, '[]', '{{}}')::real[],\n\
                  encode_to_ecvector(translate(vec, '[]', '{{}}')::real[], 4, 42)\n\
           FROM dm_stage ORDER BY id;\n\
         DROP TABLE dm_stage;\n\
         CREATE TABLE dm_queries (id bigint, source real[]);\n\
         CREATE TEMP TABLE dmq_stage (id bigint, vec text);\n\
         COPY dmq_stage (id, vec) FROM '{queries}' WITH (FORMAT text, DELIMITER E'\\t');\n\
         INSERT INTO dm_queries\n\
           SELECT id, translate(vec, '[]', '{{}}')::real[]\n\
           FROM dmq_stage ORDER BY id OFFSET {query_offset} LIMIT {queries_limit};\n\
         DROP TABLE dmq_stage;\n\
         CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops)\n\
           WITH (graph_degree = {gd}, head_index_cap = {head_index_cap},
                 build_shards = {build_shards}, head_construction = '{head_construction}'{head_sizing});\n",
        head_construction = head_construction,
        head_sizing = head_sizing,
    )
}

fn setup_sql(args: &LocalMultinodePg18Args) -> String {
    let head_sizing = head_sizing_reloptions(args);
    format!(
        "DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'peter') THEN CREATE ROLE peter LOGIN SUPERUSER; END IF; END $$;\n\
         DROP TABLE IF EXISTS dm;\n\
         CREATE TABLE dm (id bigint, source real[], embedding ecvector);\n\
         INSERT INTO dm\n\
         SELECT g,\n\
                arr,\n\
                encode_to_ecvector(arr, 4, 42)\n\
         FROM (\n\
           SELECT g,\n\
                  (SELECT array_agg((sin(g * 0.017 * (d + 1)) + cos(g * 0.0031 * (d + 1)))::real)\n\
                     FROM generate_series(0, {dim} - 1) AS d) AS arr\n\
           FROM generate_series(1, {rows}) AS g\n\
         ) s;\n\
         CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops)\n\
           WITH (graph_degree = {gd}, head_index_cap = {head_index_cap},
                 build_shards = {build_shards}, head_construction = '{head_construction}'{head_sizing});\n",
        dim = args.dim,
        rows = args.rows,
        gd = args.graph_degree,
        head_index_cap = args.head_index_cap,
        build_shards = args.build_shards,
        head_construction = args.head_construction,
        head_sizing = head_sizing,
    )
}

/// Build-time reloptions for Task 211. An omitted rate preserves the legacy
/// explicit-cap surface; a supplied rate is persisted in the generation
/// descriptor and attested against the captured row count by the extension.
fn head_sizing_reloptions(args: &LocalMultinodePg18Args) -> String {
    let Some(rate) = args.head_sampling_rate else {
        return String::new();
    };
    format!(
        ", head_sampling_rate = {rate}, head_cap_floor = {}, head_cap_ceiling = {}",
        args.head_cap_floor.unwrap_or(4096),
        args.head_cap_ceiling.unwrap_or(1_048_576),
    )
}

/// The coordinator-side recall comparison: single-node (empty roster) vs the
/// full multi-node roster (CustomScan → owner row shipping), asserting the top-k
/// id sets are identical (distinct_recall delta 0 ⇒ ≥ single − 0.001).
fn recall_sql(roster: &str, queries: u32, top_k: u32, real: bool) -> String {
    // Real lane: held-out queries from dm_queries. Synthetic lane: the first
    // `queries` corpus rows (ids 1..=queries) double as queries, as before.
    let q_source = if real {
        format!("SELECT id AS qid, source AS v FROM dm_queries ORDER BY id LIMIT {queries}")
    } else {
        format!("SELECT id AS qid, source AS v FROM dm WHERE id <= {queries}")
    };
    format!(
        "SET enable_seqscan = off;\n\
         DROP TABLE IF EXISTS q; CREATE TEMP TABLE q AS {q_source};\n\
         SET ec_distann.roster = ''; SET ec_distann.local_node_id = 1; SET ec_distann.epoch = 0;\n\
         DROP TABLE IF EXISTS base; CREATE TEMP TABLE base AS\n\
           SELECT q.qid, r.id FROM q CROSS JOIN LATERAL\n\
             (SELECT id FROM dm ORDER BY embedding <#> q.v LIMIT {top_k}) r;\n\
         SET ec_distann.roster = '{roster}'; SET ec_distann.local_node_id = 1; SET ec_distann.epoch = 1;\n\
         DROP TABLE IF EXISTS two; CREATE TEMP TABLE two AS\n\
           SELECT q.qid, r.id FROM q CROSS JOIN LATERAL\n\
             (SELECT id FROM dm ORDER BY embedding <#> q.v LIMIT {top_k}) r;\n\
         SET ec_distann.roster = '';\n\
         SELECT 'RECALL_RESULT'\n\
           || ' n_queries=' || count(DISTINCT qid)\n\
           || ' identical=' || count(DISTINCT qid) FILTER (WHERE mismatch = 0)\n\
           || ' mismatched_ids=' || coalesce(sum(mismatch), 0)\n\
         FROM (\n\
           SELECT q.qid,\n\
             (SELECT count(*) FROM (SELECT id FROM base WHERE qid=q.qid EXCEPT SELECT id FROM two WHERE qid=q.qid) d)\n\
           + (SELECT count(*) FROM (SELECT id FROM two WHERE qid=q.qid EXCEPT SELECT id FROM base WHERE qid=q.qid) d) AS mismatch\n\
           FROM q\n\
         ) s;\n"
    )
}

fn physical_setup_sql(args: &LocalMultinodePg18Args, coordinator: bool) -> Result<String> {
    let head_sizing = head_sizing_reloptions(args);
    let physical_dim = if let Some(corpus_prefix) = &args.corpus_prefix {
        let staged_dir = args
            .staged_dir
            .clone()
            .unwrap_or(repo_root()?.join("data/staged-current"));
        let manifest_path = staged_dir.join(format!("{corpus_prefix}_manifest.json"));
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(&manifest_path)
                .wrap_err_with(|| format!("reading {}", manifest_path.display()))?,
        )
        .wrap_err_with(|| format!("parsing {}", manifest_path.display()))?;
        manifest
            .get("dimension")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| *value > 0)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!(
                    "staged manifest {} has no valid dimension",
                    manifest_path.display()
                )
            })?
    } else {
        args.dim
    };
    let correctness_column = if args.materialization_correctness {
        ", payload_note text"
    } else {
        ""
    };
    let prefix = format!(
        "DROP TABLE IF EXISTS dm CASCADE;
        DROP TABLE IF EXISTS dm_queries;
        CREATE TABLE dm (
            id bigint, source_id uuid NOT NULL, source real[], embedding ecvector({}){}
        ) WITH (
            autovacuum_enabled = false,
            toast.autovacuum_enabled = false
        );",
        physical_dim, correctness_column
    );
    let load = if !coordinator {
        String::new()
    } else if let Some(corpus_prefix) = &args.corpus_prefix {
        let staged_dir = args
            .staged_dir
            .clone()
            .unwrap_or(repo_root()?.join("data/staged-current"));
        let corpus = std::fs::canonicalize(staged_dir.join(format!("{corpus_prefix}_corpus.tsv")))?
            .display()
            .to_string()
            .replace('\'', "''");
        let queries =
            std::fs::canonicalize(staged_dir.join(format!("{corpus_prefix}_queries.tsv")))?
                .display()
                .to_string()
                .replace('\'', "''");
        format!(
            "CREATE TEMP TABLE dm_stage (id bigint, vec text);
             COPY dm_stage (id, vec) FROM '{corpus}' WITH (FORMAT text, DELIMITER E'\\t');
             INSERT INTO dm (id, source_id, source, embedding)
             SELECT id,
                    (substr(md5(id::text),1,8)||'-'||substr(md5(id::text),9,4)||'-4'||
                     substr(md5(id::text),14,3)||'-8'||substr(md5(id::text),18,3)||'-'||
                     substr(md5(id::text),21,12))::uuid,
                    translate(vec, '[]', '{{}}')::real[],
                    encode_to_ecvector(translate(vec, '[]', '{{}}')::real[], 4, 42)
               FROM dm_stage ORDER BY id;
             CREATE TABLE dm_queries (id bigint, source real[]);
             CREATE TEMP TABLE dmq_stage (id bigint, vec text);
             COPY dmq_stage (id, vec) FROM '{queries}' WITH (FORMAT text, DELIMITER E'\\t');
             INSERT INTO dm_queries
             SELECT id, translate(vec, '[]', '{{}}')::real[]
               FROM dmq_stage ORDER BY id OFFSET {} LIMIT {};",
            args.query_offset, args.queries
        )
    } else {
        let synthetic_vector = synthetic_unit_vector_expr("g", args.dim);
        format!(
            "INSERT INTO dm (id, source_id, source, embedding)
             SELECT g,
                    (substr(md5(g::text),1,8)||'-'||substr(md5(g::text),9,4)||'-4'||
                     substr(md5(g::text),14,3)||'-8'||substr(md5(g::text),18,3)||'-'||
                     substr(md5(g::text),21,12))::uuid,
                    arr, encode_to_ecvector(arr, 4, 42)
               FROM (
                 SELECT g,
                        {synthetic_vector} AS arr
                   FROM generate_series(1, {rows}) AS g
               ) source_rows;",
            rows = args.rows,
        )
    };
    let correctness_fixture = if coordinator && args.materialization_correctness {
        // Keep the benchmark/query vector non-null. A correctness-only payload
        // column provides both genuine NULL datums and forced, uncompressed
        // out-of-line varlena datums in the immutable row tier without changing
        // ordinary fixtures. EXTERNAL + >12 KiB cannot remain inline on an 8 KiB
        // heap page; the DO block asserts every premise before index capture.
        "ALTER TABLE dm ALTER COLUMN payload_note SET STORAGE EXTERNAL;
         UPDATE dm
            SET payload_note = CASE WHEN id % 2 = 0 THEN NULL
                                    ELSE (SELECT string_agg(md5(id::text || ':' || piece::text), '' ORDER BY piece)
                                            FROM generate_series(1, 400) AS piece) END;
         DO $fixture$
         DECLARE external_storage boolean;
         BEGIN
             SELECT attstorage = 'e' INTO external_storage
               FROM pg_attribute
              WHERE attrelid = 'dm'::regclass AND attname = 'payload_note';
             IF NOT COALESCE(external_storage, false)
                OR EXISTS (
                    SELECT 1 FROM dm
                     WHERE payload_note IS NOT NULL
                       AND (octet_length(payload_note) < 12800
                            OR pg_column_compression(payload_note) IS NOT NULL)
                ) THEN
                 RAISE EXCEPTION 'materialization fixture is not forced external, uncompressed, and oversized';
             END IF;
         END
         $fixture$;"
    } else {
        ""
    };
    // Task 210 P2a: the storage half of head sharding is read at build time,
    // so it is set on the building session rather than on the query arms.
    let shard_head_storage = if args.sharded_head {
        "SET ec_distann.shard_head_storage = on;"
    } else if args.local_head {
        "SET ec_distann.shard_head_storage = off;"
    } else {
        ""
    };
    Ok(format!(
        "{prefix}
         {load}
         {correctness_fixture}
         -- The owner-placement proof filters by source_id. Keep that probe
         -- indexed so 50k/100k matrix setup measures the physical access path
         -- rather than repeatedly scanning the wide vector heap.
         CREATE INDEX dm_source_id_probe_idx ON dm (source_id);
         ANALYZE dm;
         {shard_head_storage}
         CREATE INDEX dm_idx ON dm USING ec_distann
             (embedding ecvector_distann_ip_ops) INCLUDE (source_id)
            WITH (distributed_control = true, source_identity = 'include',
                   graph_degree = {}, head_index_cap = {},
                   build_shards = {},
                   head_construction = '{}',
                   neighbor_code_format = 'rabitq'{});",
        args.graph_degree,
        args.head_index_cap,
        args.build_shards,
        args.head_construction,
        head_sizing
    ))
}

#[derive(Debug)]
struct PhysicalTopologyRow {
    node_id: i64,
    state: String,
    records: i64,
    rows: i64,
    non_owned_live: i64,
    non_owned_tombstones: i64,
    orphan_records: i64,
    orphan_rows: i64,
    graph_bytes: i64,
    row_bytes: i64,
    directory_bytes: i64,
    control_bytes: i64,
}

async fn physical_topology(
    psql: &Path,
    socket_dir: &Path,
    node: &Node,
    selector_sql: &str,
) -> Result<PhysicalTopologyRow> {
    let sql = format!(
        "SELECT concat_ws('|', node_id, state, record_count, row_count,
                non_owned_live_count, non_owned_tombstone_count,
                orphan_record_count, orphan_row_count, graph_bytes,
                row_tier_bytes, directory_bytes, control_index_bytes)
           FROM {selector_sql}"
    );
    let raw = capture_psql(psql, socket_dir, node.port, &sql).await?;
    let fields = raw.trim().split('|').collect::<Vec<_>>();
    if fields.len() != 12 {
        bail!(
            "physical topology node {} returned malformed row {:?}",
            node.node_id,
            raw.trim()
        );
    }
    let number = |index: usize, field: &str| -> Result<i64> {
        fields[index]
            .parse::<i64>()
            .wrap_err_with(|| format!("decoding topology {field}"))
    };
    Ok(PhysicalTopologyRow {
        node_id: number(0, "node_id")?,
        state: fields[1].to_owned(),
        records: number(2, "record_count")?,
        rows: number(3, "row_count")?,
        non_owned_live: number(4, "non_owned_live_count")?,
        non_owned_tombstones: number(5, "non_owned_tombstone_count")?,
        orphan_records: number(6, "orphan_record_count")?,
        orphan_rows: number(7, "orphan_row_count")?,
        graph_bytes: number(8, "graph_bytes")?,
        row_bytes: number(9, "row_tier_bytes")?,
        directory_bytes: number(10, "directory_bytes")?,
        control_bytes: number(11, "control_index_bytes")?,
    })
}

fn validate_physical_topology(
    phase: &str,
    topology: &[PhysicalTopologyRow],
    expected_state: &str,
    source_count: i64,
) -> Result<()> {
    if topology.is_empty()
        || topology.iter().any(|row| {
            row.state != expected_state
                || row.records <= 0
                || row.records != row.rows
                || row.non_owned_live != 0
                || row.non_owned_tombstones != 0
                || row.orphan_records != 0
                || row.orphan_rows != 0
        })
        || topology.iter().map(|row| row.records).sum::<i64>() != source_count
    {
        bail!("physical topology {phase} is incomplete or inconsistent: {topology:?}");
    }
    Ok(())
}

fn benchmark_table_rows(raw: &str) -> Vec<Vec<String>> {
    raw.lines()
        .filter(|line| line.contains('┆'))
        .map(|line| {
            line.split(['│', '┆'])
                .map(str::trim)
                .filter(|cell| !cell.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .filter(|cells| {
            cells
                .first()
                .is_some_and(|cell| cell.parse::<u32>().is_ok())
        })
        .collect()
}

fn benchmark_table_row(raw: &str) -> Result<Vec<String>> {
    benchmark_table_rows(raw)
        .into_iter()
        .next()
        .ok_or_else(|| color_eyre::eyre::eyre!("benchmark output has no data row"))
}

fn benchmark_ms(cell: &str) -> Result<f64> {
    cell.trim_end_matches(" ms")
        .trim()
        .parse::<f64>()
        .wrap_err_with(|| format!("decoding benchmark duration {cell:?}"))
}

fn attribution_stage_mean(stage_rows: &[&str], stage: &str) -> Result<f64> {
    let row = stage_rows
        .iter()
        .find(|row| {
            row.split_whitespace()
                .any(|field| field == format!("stage={stage}"))
        })
        .ok_or_else(|| color_eyre::eyre::eyre!("missing attribution stage {stage}"))?;
    let value = row
        .split_whitespace()
        .find_map(|field| field.strip_prefix("mean_ms="))
        .ok_or_else(|| color_eyre::eyre::eyre!("attribution stage {stage} has no mean_ms"))?;
    value
        .parse::<f64>()
        .wrap_err_with(|| format!("parsing mean_ms for attribution stage {stage}"))
}

async fn run_physical_bench_child(args: Vec<String>) -> Result<String> {
    let executable = std::env::current_exe().wrap_err("resolving benchmark executable")?;
    // Session GUCs decide which mechanism an arm measures; a silently absent
    // GUC is how two replica arms ran inert (2026-07-31). Log the exact child
    // argv so packet artifacts can prove what each benchmark was told.
    crate::ecaz_eprintln!(
        "[distann-multicluster] physical_bench_child args={}",
        args.join(" ")
    );
    let output = Command::new(&executable)
        .args(&args)
        .output()
        .await
        .wrap_err_with(|| format!("spawning {}", executable.display()))?;
    if !output.status.success() {
        bail!(
            "physical benchmark child failed with {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut captured = String::from_utf8_lossy(&output.stdout).into_owned();
    // PostgreSQL NOTICE records are delivered on the child connection's
    // stderr by the reporting client. Keep them with the benchmark stdout so
    // the parent can persist structured per-round telemetry in its summary.
    captured.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(captured)
}

fn append_distann_notice_lines(summary: &mut Vec<String>, child_output: &str) {
    summary.extend(
        child_output
            .lines()
            .filter(|line| line.trim_start().starts_with("[postgres notice] "))
            .map(str::to_owned),
    );
}

fn append_materialization_benchmark_guc(
    args: &mut Vec<String>,
    arm: &str,
    materialization_batch_size: u32,
) {
    if arm == "physical" && materialization_batch_size != 10 {
        args.extend([
            "--session-guc".into(),
            format!("ec_distann.benchmark_materialization_batch_size={materialization_batch_size}"),
        ]);
    }
}

fn append_owner_payload_plan_cache_guc(args: &mut Vec<String>, arm: &str, enabled: bool) {
    if arm == "physical" && enabled {
        args.extend([
            "--session-guc".into(),
            format!(
                "ec_distann.benchmark_owner_payload_plan_cache={}",
                if enabled { "on" } else { "off" }
            ),
        ]);
    }
}

fn append_typed_locator_guc(args: &mut Vec<String>, arm: &str, enabled: bool) {
    if arm == "physical" && enabled {
        args.extend([
            "--session-guc".into(),
            "ec_distann.benchmark_typed_locator=on".into(),
        ]);
    }
}

fn append_packed_payload_guc(args: &mut Vec<String>, arm: &str, enabled: bool) {
    if arm == "physical" && enabled {
        args.extend([
            "--session-guc".into(),
            "ec_distann.benchmark_packed_payload=on".into(),
        ]);
    }
}

fn append_expanded_locator_guc(args: &mut Vec<String>, arm: &str, enabled: bool) {
    if arm == "physical" && enabled {
        args.extend([
            "--session-guc".into(),
            "ec_distann.benchmark_expanded_locator=on".into(),
        ]);
    }
}

/// NFR-021 clause 4 (Task 210 P1): the FR-084 traversal replica is off by
/// default in the extension. A replica arm must opt in explicitly, which is
/// also what marks it as a non-conforming accelerator in the emitted rows.
/// Task 210 P2a/P2b: sharded head search and its replica count are session
/// GUCs on the physical arm; the storage half is applied before the build.
fn append_sharded_head_guc(
    args: &mut Vec<String>,
    arm: &str,
    sharded_head: bool,
    local_head: bool,
    head_replica_count: Option<u32>,
) {
    if arm != "physical" {
        return;
    }
    if sharded_head {
        args.extend([
            "--session-guc".into(),
            "ec_distann.sharded_head_search=on".into(),
        ]);
    }
    if local_head {
        args.extend([
            "--session-guc".into(),
            "ec_distann.sharded_head_search=off".into(),
        ]);
    }
    // The replica count is independent of the legacy --sharded-head flag:
    // sharded search is the shipped default now, and gating the GUC on the
    // flag left the default-config replica arm silently inert
    // (head_replica_shards_served=0 AND head_replica_fallbacks=0 in the first
    // gate attempt — routing never consulted replicas at all).
    if let Some(replicas) = head_replica_count {
        args.extend([
            "--session-guc".into(),
            format!("ec_distann.head_replica_count={replicas}"),
        ]);
    }
}

/// Task 210 P3: the TRAV-30 gateway copy capacity is a coordinator session
/// GUC on the physical arm; population happens once per cached epoch at scan
/// open, bounded by this capacity.
fn append_gateway_copy_guc(args: &mut Vec<String>, arm: &str, capacity: Option<u32>) {
    if arm != "physical" {
        return;
    }
    if let Some(capacity) = capacity {
        args.extend([
            "--session-guc".into(),
            format!("ec_distann.gateway_copy_capacity={capacity}"),
        ]);
    }
}

fn append_crown_gucs(
    args: &mut Vec<String>,
    arm: &str,
    capacity: Option<u32>,
    width_pruning: bool,
    fused_head_hop: bool,
) {
    if arm != "physical" {
        return;
    }
    if let Some(capacity) = capacity {
        args.extend([
            "--session-guc".into(),
            format!("ec_distann.crown_capacity={capacity}"),
        ]);
    }
    if width_pruning {
        args.extend([
            "--session-guc".into(),
            "ec_distann.crown_width_pruning=on".into(),
        ]);
    }
    if fused_head_hop {
        args.extend([
            "--session-guc".into(),
            "ec_distann.fused_head_hop=on".into(),
        ]);
    }
}

fn crown_counter(line: &str, name: &str) -> Option<i64> {
    line.split_whitespace()
        .find_map(|field| field.strip_prefix(&format!("{name}=")))
        .and_then(|value| value.parse().ok())
}

fn validate_crown_activation(
    args: &LocalMultinodePg18Args,
    stats_seen: bool,
    crown_seeds_served: i64,
    _crown_width_pruned_shards: i64,
    crown_width_pruning_activations: i64,
    fused_head_hops: i64,
    fused_first_round_requested_ids: i64,
) -> Result<()> {
    if args.crown_capacity.is_none() {
        return Ok(());
    }
    if !stats_seen {
        bail!("crown-enabled physical arm did not report ec_distann crown counters");
    }
    if args.crown_width_pruning && crown_width_pruning_activations <= 0 {
        bail!("crown-width arm reported zero crown_width_pruning_activations");
    }
    if args.fused_head_hop && crown_seeds_served <= 0 {
        bail!("fused crown arm served zero crown seeds");
    }
    if args.fused_head_hop && fused_head_hops <= 0 {
        bail!("fused-head-hop arm reported zero fused_head_hops");
    }
    if args.fused_head_hop && fused_first_round_requested_ids <= 0 {
        bail!("fused-head-hop arm reported zero fused first-round requested ids");
    }
    Ok(())
}

fn append_nonconforming_replica_guc(args: &mut Vec<String>, arm: &str, enabled: bool) {
    if arm == "physical" && enabled {
        args.extend([
            "--session-guc".into(),
            "ec_distann.allow_nonconforming_replica=on".into(),
        ]);
    }
}

#[derive(Clone, Debug)]
struct BenchmarkSeedVariant {
    name: String,
    strategy: String,
    head_search_width: u32,
    head_seed_count: u32,
    neighbor_score_mode: String,
    materialization_batch_size: u32,
    owner_payload_plan_cache: bool,
    beam_width: Option<u32>,
    hop_rounds: Option<u32>,
    traversal_replica: bool,
    typed_locator: bool,
    packed_payload: bool,
    expanded_locator: bool,
}

#[derive(Debug, Deserialize)]
struct PairedPredictionFile {
    query_ids: Vec<i64>,
    rows: Vec<PairedPredictionSweep>,
}

#[derive(Debug, Deserialize)]
struct PairedPredictionSweep {
    sweep_value: i32,
    predictions: Vec<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
struct PairedTruthCache {
    truth: PairedTruthSet,
}

#[derive(Debug, Deserialize)]
struct PairedTruthSet {
    ids: Vec<Vec<i64>>,
}

fn paired_recall_line(
    scale: &str,
    control_path: &Path,
    candidate_path: &Path,
    truth_path: &Path,
    k: usize,
) -> Result<String> {
    let control: PairedPredictionFile = serde_json::from_slice(&fs::read(control_path)?)
        .wrap_err_with(|| {
            format!(
                "reading paired control predictions {}",
                control_path.display()
            )
        })?;
    let candidate: PairedPredictionFile = serde_json::from_slice(&fs::read(candidate_path)?)
        .wrap_err_with(|| {
            format!(
                "reading paired candidate predictions {}",
                candidate_path.display()
            )
        })?;
    let truth: PairedTruthCache = serde_json::from_slice(&fs::read(truth_path)?)
        .wrap_err_with(|| format!("reading paired truth cache {}", truth_path.display()))?;
    let control_sweep = control
        .rows
        .iter()
        .find(|row| row.sweep_value == 32)
        .ok_or_else(|| color_eyre::eyre::eyre!("paired control predictions have no sweep 32"))?;
    let candidate_sweep = candidate
        .rows
        .iter()
        .find(|row| row.sweep_value == 32)
        .ok_or_else(|| color_eyre::eyre::eyre!("paired candidate predictions have no sweep 32"))?;
    let query_count = truth.truth.ids.len();
    if control.query_ids != candidate.query_ids
        || control.query_ids.len() != query_count
        || control_sweep.predictions.len() != query_count
        || candidate_sweep.predictions.len() != query_count
        || k == 0
    {
        bail!(
            "paired recall inputs are not aligned: control_queries={} candidate_queries={} truth_queries={} control_rows={} candidate_rows={} k={k}",
            control.query_ids.len(),
            candidate.query_ids.len(),
            query_count,
            control_sweep.predictions.len(),
            candidate_sweep.predictions.len(),
        );
    }

    let mut deltas = Vec::with_capacity(query_count);
    let mut candidate_wins = 0usize;
    let mut control_wins = 0usize;
    let mut ties = 0usize;
    for query_index in 0..query_count {
        let truth_ids = &truth.truth.ids[query_index];
        let query_recall = |predictions: &[i64]| {
            predictions
                .iter()
                .take(k)
                .filter(|id| truth_ids.contains(id))
                .count() as f64
                / k as f64
        };
        let control_recall = query_recall(&control_sweep.predictions[query_index]);
        let candidate_recall = query_recall(&candidate_sweep.predictions[query_index]);
        let delta = candidate_recall - control_recall;
        if delta > 0.0 {
            candidate_wins += 1;
        } else if delta < 0.0 {
            control_wins += 1;
        } else {
            ties += 1;
        }
        deltas.push(delta);
    }

    let mean_delta = deltas.iter().sum::<f64>() / query_count as f64;
    let mut bootstrap = Vec::with_capacity(10_000);
    let mut state = 0x9e3779b97f4a7c15_u64;
    for _ in 0..10_000 {
        let mut sum = 0.0;
        for _ in 0..query_count {
            state ^= state << 7;
            state ^= state >> 9;
            state ^= state << 8;
            sum += deltas[(state as usize) % query_count];
        }
        bootstrap.push(sum / query_count as f64);
    }
    bootstrap.sort_by(f64::total_cmp);
    let low = bootstrap[250];
    let high = bootstrap[9_749];
    Ok(format!(
        "physical_benchmark_paired_recall scale={scale} control=bw4-control candidate=bw8-candidate query_rows={query_count} trials={} candidate_wins={candidate_wins} control_wins={control_wins} ties={ties} candidate_minus_control_mean={mean_delta:.6} paired_bootstrap_ci95_low={low:.6} paired_bootstrap_ci95_high={high:.6}",
        query_count * k
    ))
}

fn task167_pre_insert_recall_calibration_line(
    scale: &str,
    predictions_path: &Path,
    truth_path: &Path,
    ordinary_distinct_recall: f64,
    k: usize,
) -> Result<String> {
    let predictions: PairedPredictionFile = serde_json::from_slice(&fs::read(predictions_path)?)
        .wrap_err_with(|| {
            format!(
                "reading Task 167 pre-insert predictions {}",
                predictions_path.display()
            )
        })?;
    let truth: PairedTruthCache = serde_json::from_slice(&fs::read(truth_path)?)
        .wrap_err_with(|| format!("reading Task 167 truth cache {}", truth_path.display()))?;
    let sweep = predictions
        .rows
        .iter()
        .find(|row| row.sweep_value == 32)
        .ok_or_else(|| eyre!("Task 167 pre-insert predictions have no sweep 32"))?;
    if k == 0
        || truth.truth.ids.is_empty()
        || predictions.query_ids.len() != truth.truth.ids.len()
        || sweep.predictions.len() != truth.truth.ids.len()
    {
        bail!(
            "Task 167 pre-insert calibration inputs are not aligned: query_ids={} predictions={} truth={} k={k}",
            predictions.query_ids.len(),
            sweep.predictions.len(),
            truth.truth.ids.len(),
        );
    }
    for (query_index, (truth, predicted)) in
        truth.truth.ids.iter().zip(&sweep.predictions).enumerate()
    {
        let distinct_truth = truth.iter().take(k).copied().collect::<HashSet<_>>();
        if truth.len() != k || distinct_truth.len() != k || predicted.len() != k {
            bail!(
                "Task 167 pre-insert calibration query {query_index} is not a full distinct recall@{k} row: truth={} distinct_truth={} predictions={}",
                truth.len(),
                distinct_truth.len(),
                predicted.len(),
            );
        }
    }
    let exact_scorer_recall = truth
        .truth
        .ids
        .iter()
        .zip(&sweep.predictions)
        .map(|(truth, predicted)| task167_distinct_recall(truth, predicted).0)
        .sum::<f64>()
        / truth.truth.ids.len() as f64;
    // The ordinary bench table is rendered to four decimal places before it
    // is parsed here. Half a displayed unit is therefore the strictest useful
    // cross-instrument comparison tolerance.
    const DISPLAY_TOLERANCE: f64 = 0.000_05;
    let absolute_delta = (ordinary_distinct_recall - exact_scorer_recall).abs();
    let pass = absolute_delta <= DISPLAY_TOLERANCE + f64::EPSILON;
    if !pass {
        bail!(
            "Task 167 pre-insert recall instruments disagree: ordinary={ordinary_distinct_recall:.6} exact_scorer={exact_scorer_recall:.6} delta={absolute_delta:.6} tolerance={DISPLAY_TOLERANCE:.6}"
        );
    }
    Ok(format!(
        "physical_benchmark_recall_instrument_calibration scale={scale} phase=pre_incremental_insert graph_state=same predictions=same truth=same queries={} top_k={k} ordinary_distinct_recall={ordinary_distinct_recall:.6} exact_scorer_distinct_recall={exact_scorer_recall:.6} absolute_delta={absolute_delta:.6} tolerance={DISPLAY_TOLERANCE:.6} pass=true",
        truth.truth.ids.len(),
    ))
}

fn parse_benchmark_seed_variants(values: &[String]) -> Result<Vec<BenchmarkSeedVariant>> {
    let mut names = std::collections::BTreeSet::new();
    values
        .iter()
        .map(|value| {
            let fields = value.split(':').collect::<Vec<_>>();
            if !(5..=13).contains(&fields.len()) {
                bail!(
                    "benchmark seed variant must be NAME:MODE:SEARCH_WIDTH:SEED_COUNT:NEIGHBOR_SCORE_MODE[:MATERIALIZATION_BATCH_SIZE[:OWNER_PAYLOAD_PLAN_CACHE[:BEAM_WIDTH[:HOP_ROUNDS[:TRAVERSAL_REPLICA[:TYPED_LOCATOR[:PACKED_PAYLOAD[:EXPANDED_LOCATOR]]]]]]]], got {value:?}"
                );
            }
            let name = fields[0];
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
            {
                bail!("benchmark seed variant name must be an ASCII identifier, got {name:?}");
            }
            if !names.insert(name.to_owned()) {
                bail!("duplicate benchmark seed variant name {name:?}");
            }
            let strategy = fields[1];
            if !matches!(
                strategy,
                "persisted_head" | "head_sample_exact" | "head_hierarchy" | "owner_scan"
            ) {
                bail!(
                    "benchmark seed variant mode must be persisted_head, head_sample_exact, head_hierarchy, or owner_scan, got {strategy:?}"
                );
            }
            let head_search_width = fields[2]
                .parse::<u32>()
                .wrap_err_with(|| format!("parsing search width in benchmark seed variant {value:?}"))?;
            let head_seed_count = fields[3]
                .parse::<u32>()
                .wrap_err_with(|| format!("parsing seed count in benchmark seed variant {value:?}"))?;
            if !(1..=4096).contains(&head_search_width)
                || !(1..=4096).contains(&head_seed_count)
            {
                bail!(
                    "benchmark seed variant widths must be in 1..=4096, got {value:?}"
                );
            }
            let neighbor_score_mode = fields[4];
            if !matches!(neighbor_score_mode, "rabitq" | "exact_neighbor") {
                bail!(
                    "benchmark seed variant neighbor score mode must be rabitq or exact_neighbor, got {neighbor_score_mode:?}"
                );
            }
            // Omitted variant controls inherit the normal production lazy-10
            // path. Zero remains an explicit eager arm for Task 184-style
            // materialization comparisons.
            let materialization_batch_size = fields
                .get(5)
                .map(|field| {
                    field.parse::<u32>().wrap_err_with(|| {
                        format!(
                            "parsing materialization batch size in benchmark seed variant {value:?}"
                        )
                    })
                })
                .transpose()?
                .unwrap_or(10);
            if materialization_batch_size > 4096 {
                bail!(
                    "benchmark seed variant materialization batch size must be in 0..=4096, got {value:?}"
                );
            }
            let owner_payload_plan_cache = fields
                .get(6)
                .map(|field| match *field {
                    "on" => Ok(true),
                    "off" => Ok(false),
                    _ => bail!(
                        "benchmark seed variant owner payload plan cache must be on or off, got {value:?}"
                    ),
                })
                .transpose()?
                .unwrap_or(false);
            let beam_width = fields
                .get(7)
                .map(|field| {
                    field.parse::<u32>().wrap_err_with(|| {
                        format!("parsing beam width in benchmark seed variant {value:?}")
                    })
                })
                .transpose()?;
            if beam_width.is_some_and(|width| !(1..=256).contains(&width)) {
                bail!(
                    "benchmark seed variant beam width must be in 1..=256, got {value:?}"
                );
            }
            let hop_rounds = fields
                .get(8)
                .map(|field| {
                    field.parse::<u32>().wrap_err_with(|| {
                        format!("parsing hop rounds in benchmark seed variant {value:?}")
                    })
                })
                .transpose()?;
            if hop_rounds.is_some_and(|rounds| !(1..=256).contains(&rounds)) {
                bail!(
                    "benchmark seed variant hop rounds must be in 1..=256, got {value:?}"
                );
            }
            let traversal_replica = fields
                .get(9)
                .map(|field| match *field {
                    "on" => Ok(true),
                    "off" => Ok(false),
                    _ => bail!(
                        "benchmark seed variant traversal replica must be on or off, got {value:?}"
                    ),
                })
                .transpose()?
                .unwrap_or(false);
            if traversal_replica && neighbor_score_mode == "exact_neighbor" {
                bail!(
                    "benchmark seed variant cannot combine traversal replica with exact_neighbor, got {value:?}"
                );
            }
            let typed_locator = fields
                .get(10)
                .map(|field| match *field {
                    "on" => Ok(true),
                    "off" => Ok(false),
                    _ => bail!(
                        "benchmark seed variant typed locator must be on or off, got {value:?}"
                    ),
                })
                .transpose()?
                .unwrap_or(false);
            let packed_payload = fields
                .get(11)
                .map(|field| match *field {
                    "on" => Ok(true),
                    "off" => Ok(false),
                    _ => bail!(
                        "benchmark seed variant packed payload must be on or off, got {value:?}"
                    ),
                })
                .transpose()?
                .unwrap_or(false);
            let expanded_locator = fields
                .get(12)
                .map(|field| match *field {
                    "on" => Ok(true),
                    "off" => Ok(false),
                    _ => bail!(
                        "benchmark seed variant expanded locator must be on or off, got {value:?}"
                    ),
                })
                .transpose()?
                .unwrap_or(false);
            Ok(BenchmarkSeedVariant {
                name: name.to_owned(),
                strategy: strategy.to_owned(),
                head_search_width,
                head_seed_count,
                neighbor_score_mode: neighbor_score_mode.to_owned(),
                materialization_batch_size,
                owner_payload_plan_cache,
                beam_width,
                hop_rounds,
                traversal_replica,
                typed_locator,
                packed_payload,
                expanded_locator,
            })
        })
        .collect()
}

fn register_same_seed_digest(
    digests: &mut std::collections::HashMap<(String, u32, u32), (String, String)>,
    variant: &BenchmarkSeedVariant,
    seed_id_digest: &str,
) -> Result<Option<String>> {
    let key = (
        variant.strategy.clone(),
        variant.head_search_width,
        variant.head_seed_count,
    );
    if let Some((prior_variant, prior_digest)) = digests.get(&key) {
        if prior_digest != seed_id_digest {
            bail!(
                "same-seed attribution failed: variants {} and {} selected different seed IDs ({} != {})",
                prior_variant,
                variant.name,
                prior_digest,
                seed_id_digest
            );
        }
        return Ok(Some(prior_variant.clone()));
    }
    digests.insert(key, (variant.name.clone(), seed_id_digest.to_owned()));
    Ok(None)
}

async fn materialization_result_json(
    coordinator: &tokio_postgres::Client,
    variant: &BenchmarkSeedVariant,
    sql: &str,
    has_attribution_hooks: bool,
) -> Result<String> {
    let reset_sql = if has_attribution_hooks {
        "SELECT ec_distann_stage_scoring_reset();"
    } else {
        ""
    };
    coordinator
        .batch_execute(&format!(
            "{reset_sql} SET enable_seqscan = off; {}",
            materialization_variant_settings_sql(variant),
        ))
        .await?;
    coordinator
        .query_one(sql, &[])
        .await
        .wrap_err_with(|| {
            format!(
                "running materialization semantic query for variant {}",
                variant.name
            )
        })?
        .try_get::<_, String>(0)
        .wrap_err("decoding materialization semantic result")
}

fn materialization_variant_settings_sql(variant: &BenchmarkSeedVariant) -> String {
    let mut settings = Vec::new();
    if variant.materialization_batch_size != 10 {
        settings.push(format!(
            "SET ec_distann.benchmark_materialization_batch_size = {}",
            variant.materialization_batch_size
        ));
    }
    if variant.owner_payload_plan_cache {
        settings.push("SET ec_distann.benchmark_owner_payload_plan_cache = on".to_owned());
    }
    if settings.is_empty() {
        String::new()
    } else {
        format!("{};", settings.join("; "))
    }
}

fn materialization_semantic_sql(
    corpus: &str,
    queries: &str,
    predicate: &str,
    limit: u32,
    query_offset: u32,
) -> String {
    format!(
        "WITH query_vector AS (
             SELECT source FROM {queries} ORDER BY id OFFSET {query_offset} LIMIT 1
         )
         SELECT COALESCE(jsonb_agg(to_jsonb(result) ORDER BY result.distance)::text, '[]')
           FROM (
             SELECT id,
                    source_id::text AS source_id,
                    payload_note IS NULL AS payload_null,
                    CASE WHEN payload_note IS NULL THEN NULL ELSE md5(payload_note) END AS payload_digest,
                    CASE WHEN payload_note IS NULL THEN NULL ELSE octet_length(payload_note) END AS payload_octets,
                    CASE WHEN payload_note IS NULL THEN NULL ELSE pg_column_compression(payload_note) END AS payload_compression,
                    (SELECT attstorage::text FROM pg_attribute WHERE attrelid = '{corpus}'::regclass AND attname = 'payload_note') AS payload_storage,
                    embedding <#> (SELECT source FROM query_vector) AS distance
               FROM {corpus}
              WHERE {predicate}
              ORDER BY embedding <#> (SELECT source FROM query_vector)
              LIMIT {limit}
           ) result"
    )
}

async fn task198_replica_semantic_result(
    coordinator: &tokio_postgres::Client,
    corpus: &str,
    queries: &str,
    fail_batch: i32,
    query_offset: u32,
    result_limit: u32,
) -> Result<String> {
    let has_fault_hooks = coordinator
        .query_one(
            "SELECT to_regprocedure('ec_distann_stage_scoring_reset()') IS NOT NULL
                    AND current_setting(
                        'ec_distann.benchmark_traversal_replica_fail_batch', true
                    ) IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    if fail_batch >= 0 && !has_fault_hooks {
        bail!("traversal-replica fault injection is absent from the normal release build");
    }
    let benchmark_settings = if has_fault_hooks {
        format!(
            "SELECT ec_distann_stage_scoring_reset();
             SET ec_distann.benchmark_seed_mode = 'persisted_head';
             SET ec_distann.benchmark_head_search_width = 32;
             SET ec_distann.benchmark_head_seed_count = 32;
             SET ec_distann.benchmark_exact_neighbor = off;
             SET ec_distann.benchmark_materialization_batch_size = 10;
             SET ec_distann.benchmark_owner_payload_plan_cache = off;
             SET ec_distann.benchmark_traversal_replica_fail_batch = {fail_batch};
             SET ec_distann.allow_nonconforming_replica = on;"
        )
    } else {
        // The replica is off by default (NFR-021 clause 4, Task 210 P1); the
        // semantic drill exists to exercise it, so it opts in explicitly.
        "SET ec_distann.allow_nonconforming_replica = on;".to_owned()
    };
    coordinator
        .batch_execute(&format!(
            "{benchmark_settings}
             SET enable_seqscan = off;
             SET ec_distann.beam_width = 4;
             SET ec_distann.hop_rounds = 100;"
        ))
        .await?;
    let has_payload = coordinator
        .query_one(
            "SELECT EXISTS (
                 SELECT 1 FROM pg_attribute
                  WHERE attrelid = $1::text::regclass
                    AND attname = 'payload_note'
                    AND NOT attisdropped
             )",
            &[&corpus],
        )
        .await?
        .get::<_, bool>(0);
    let sql = if has_payload {
        materialization_semantic_sql(corpus, queries, "TRUE", result_limit, query_offset)
    } else {
        format!(
            "WITH query_vector AS (
                 SELECT source FROM {queries} ORDER BY id OFFSET {query_offset} LIMIT 1
             )
             SELECT COALESCE(
                        jsonb_agg(to_jsonb(result) ORDER BY result.distance, result.id)::text,
                        '[]'
                    )
               FROM (
                 SELECT id, source_id::text AS source_id,
                        embedding <#> (SELECT source FROM query_vector) AS distance
                   FROM {corpus}
                  ORDER BY embedding <#> (SELECT source FROM query_vector)
                  LIMIT {result_limit}
               ) result"
        )
    };
    coordinator
        .query_one(&sql, &[])
        .await?
        .try_get::<_, String>(0)
        .wrap_err("decoding Task 198 semantic result")
}

fn task199_real_insert_sql(corpus: &str) -> String {
    format!(
        "WITH next_row AS (
             SELECT coalesce(max(id), 0) + 1 AS id FROM {corpus}
         ), seed AS (
             SELECT source, embedding FROM {corpus} ORDER BY id LIMIT 1
         )
         INSERT INTO {corpus} (id, source_id, source, embedding)
         SELECT next_row.id,
                (substr(md5(next_row.id::text),1,8)||'-'||
                 substr(md5(next_row.id::text),9,4)||'-4'||
                 substr(md5(next_row.id::text),14,3)||'-8'||
                 substr(md5(next_row.id::text),18,3)||'-'||
                 substr(md5(next_row.id::text),21,12))::uuid,
                seed.source, seed.embedding
           FROM next_row CROSS JOIN seed
         RETURNING id"
    )
}

async fn task199_real_delete_invalidation_drill(
    coordinator: &tokio_postgres::Client,
    corpus: &str,
) -> Result<String> {
    let target_id = coordinator
        .query_one(&format!("SELECT min(id)::bigint FROM {corpus}"), &[])
        .await?
        .get::<_, i64>(0);
    let invalidation = coordinator
        .execute(
            &format!("DELETE FROM {corpus} WHERE id = $1"),
            &[&target_id],
        )
        .await
        .expect_err("real DELETE must invalidate the Ready traversal replica");
    let retryable = invalidation
        .code()
        .is_some_and(|code| code.code() == "40001")
        && invalidation
            .as_db_error()
            .is_some_and(|error| error.message().contains("EC_REPLICA_INVALIDATED"));
    let row_preserved = coordinator
        .query_one(
            &format!("SELECT count(*) = 1 FROM {corpus} WHERE id = $1"),
            &[&target_id],
        )
        .await?
        .get::<_, bool>(0);
    let stale = coordinator
        .query_one(
            "SELECT count(*) = 1
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Stale'",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    if !retryable || !row_preserved || !stale {
        bail!(
            "Task 199 real DELETE invalidation failed: retryable={retryable} \
             row_preserved={row_preserved} stale={stale}"
        );
    }
    Ok(format!(
        "scenario=real_delete_durable_invalidation pass=true target_id={target_id} \
         first_sqlstate=40001 token=EC_REPLICA_INVALIDATED state=Stale deleted_rows=0"
    ))
}

async fn task199_crash_after_control_commit_drill(
    coordinator: &tokio_postgres::Client,
    coordinator_port: u16,
    corpus: &str,
    queries: &str,
    owner_baseline: &str,
) -> Result<String> {
    let inserted_id = coordinator
        .query_one(
            &format!("SELECT coalesce(max(id), 0) + 1 FROM {corpus}"),
            &[],
        )
        .await?
        .get::<_, i64>(0);
    let (mutator, mutator_connection) = task199_connect(coordinator_port).await?;
    let mutator_pid = mutator
        .query_one("SELECT pg_backend_pid()", &[])
        .await?
        .get::<_, i32>(0);
    mutator
        .batch_execute("SET ec_distann.debug_crash_after_replica_control_commit = on")
        .await?;
    let crash = mutator
        .query_one(&task199_real_insert_sql(corpus), &[])
        .await
        .expect_err("injected post-control-commit crash must terminate the mutating backend");
    let terminated = crash.code().is_some_and(|code| code.code() == "57P01") || crash.is_closed();
    let _ = mutator_connection.await;

    let state = coordinator
        .query_one(
            "SELECT state
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              ORDER BY build_started_at DESC LIMIT 1",
            &[],
        )
        .await?
        .get::<_, String>(0);
    let inserted_count = coordinator
        .query_one(
            &format!("SELECT count(*)::bigint FROM {corpus} WHERE id = $1"),
            &[&inserted_id],
        )
        .await?
        .get::<_, i64>(0);
    let (fresh, fresh_connection) = task199_connect(coordinator_port).await?;
    let owner_after_crash =
        task198_replica_semantic_result(&fresh, corpus, queries, -1, 0, 20).await?;
    fresh_connection.abort();
    let owner_fallback = owner_after_crash == owner_baseline;
    if !terminated || state != "Stale" || inserted_count != 0 || !owner_fallback {
        bail!(
            "Task 199 post-control-commit crash failed: pid={mutator_pid} \
             terminated={terminated} state={state} inserted={inserted_count} \
             owner_fallback={owner_fallback} error={crash}"
        );
    }
    Ok(format!(
        "scenario=crash_after_control_commit pass=true backend_pid={mutator_pid} \
         backend_terminated=true state=Stale inserted_rows=0 \
         fresh_backend_owner_fallback_identity=true"
    ))
}

async fn task199_enospc_replica_build_drill(
    coordinator: &tokio_postgres::Client,
    scale: &str,
    corpus: &str,
    queries: &str,
    owner_baseline: &str,
    fixture: &Task199EnospcFixture,
    lines: &mut Vec<String>,
) -> Result<()> {
    async fn verify_failure(
        coordinator: &tokio_postgres::Client,
        corpus: &str,
        queries: &str,
        owner_baseline: &str,
        fixture: &Task199EnospcFixture,
        marker_start: usize,
        phase: &str,
        error: &tokio_postgres::Error,
    ) -> Result<(usize, String)> {
        let sqlstate = error.code().map(|code| code.code()).unwrap_or("none");
        if sqlstate != "53100" {
            bail!(
                "Task 199 {phase} ENOSPC build returned SQLSTATE {sqlstate}, expected 53100: {error}"
            );
        }
        let marker = fs::read_to_string(&fixture.marker_file)?;
        let expected_target = fixture.tablespace_dir.to_string_lossy();
        let provider_faults = marker
            .lines()
            .skip(marker_start)
            .filter(|line| {
                let expected_operation = match phase {
                    "create" => {
                        line.contains("op=open")
                            || line.contains("op=open64")
                            || line.contains("op=openat")
                            || line.contains("op=openat2")
                    }
                    "data" => {
                        line.contains("op=write")
                            || line.contains("op=pwrite")
                            || line.contains("op=pwrite64")
                            || line.contains("op=pwritev")
                            || line.contains("op=fsync")
                            || line.contains("op=fdatasync")
                    }
                    _ => false,
                };
                line.contains("fault=1")
                    && line.contains("mode=enospc-write")
                    && expected_operation
                    && line.contains("errno=28")
                    && match phase {
                        "create" => line.contains("pg_tblspc/"),
                        "data" => line.contains(expected_target.as_ref()),
                        _ => false,
                    }
            })
            .collect::<Vec<_>>();
        let residue = coordinator
            .query_one(
                "SELECT
                     (SELECT count(*)::bigint
                        FROM ec_distann_traversal_replica_status(
                            'dm_idx'::regclass
                        )),
                     (SELECT count(*)::bigint
                        FROM pg_catalog.pg_class
                       WHERE relnamespace = 'public'::regnamespace
                         AND relname ~ '^_ecdz_replica(_dir)?_')",
                &[],
            )
            .await?;
        let catalog_residue = residue.get::<_, i64>(0);
        let relation_residue = residue.get::<_, i64>(1);
        let cluster_healthy = coordinator
            .query_one("SELECT 1::bigint", &[])
            .await?
            .get::<_, i64>(0)
            == 1;
        let owner_after_failure =
            task198_replica_semantic_result(coordinator, corpus, queries, -1, 0, 20).await?;
        let owner_fallback = owner_after_failure == owner_baseline;
        if provider_faults.is_empty()
            || catalog_residue != 0
            || relation_residue != 0
            || !cluster_healthy
            || !owner_fallback
        {
            bail!(
                "Task 199 {phase} ENOSPC cleanup failed: sqlstate={sqlstate} \
                 provider_faults={} catalog_residue={catalog_residue} \
                 relation_residue={relation_residue} cluster_healthy={cluster_healthy} \
                 owner_fallback={owner_fallback} error={error}",
                provider_faults.len()
            );
        }
        Ok((provider_faults.len(), provider_faults[0].to_owned()))
    }

    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the pre-ENOSPC replica");
    }
    let original_tablespace = coordinator
        .query_one(
            "SELECT CASE
                        WHEN c.reltablespace = 0 THEN 'pg_default'
                        ELSE t.spcname
                    END
               FROM pg_catalog.pg_class c
               LEFT JOIN pg_catalog.pg_tablespace t
                 ON t.oid = c.reltablespace
              WHERE c.oid = 'dm_idx'::regclass::oid",
            &[],
        )
        .await?
        .get::<_, String>(0);
    let tablespace_path = fixture.tablespace_dir.to_string_lossy().replace('\'', "''");
    coordinator
        .batch_execute(&format!(
            "CREATE TABLESPACE task199_replica_enospc LOCATION '{tablespace_path}'"
        ))
        .await
        .wrap_err("creating the Task 199 ENOSPC replica tablespace")?;
    coordinator
        .batch_execute("ALTER INDEX dm_idx SET TABLESPACE task199_replica_enospc")
        .await
        .wrap_err("preparing the Task 199 ENOSPC replica tablespace")?;

    let create_marker_start = fs::read_to_string(&fixture.marker_file)?.lines().count();
    fs::write(&fixture.arm_file, "create\n")?;
    let injected = coordinator
        .query_one(
            "SELECT ec_distann_build_traversal_replica('dm_idx'::regclass)",
            &[],
        )
        .await;
    let disarm = fs::remove_file(&fixture.arm_file);
    disarm.wrap_err("disarming the Task 199 ENOSPC provider")?;
    let create_error =
        injected.expect_err("create-armed replica build must fail under injected ENOSPC");
    let (create_faults, create_marker) = verify_failure(
        coordinator,
        corpus,
        queries,
        owner_baseline,
        fixture,
        create_marker_start,
        "create",
        &create_error,
    )
    .await?;
    let create_recovery_digest =
        build_and_attest_traversal_replica(coordinator, scale, lines).await?;
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} \
         scenario=enospc_create_cleanup pass=true sqlstate=53100 \
         provider_fault_events={create_faults} errno=28 \
         eligible_partial_images=0 catalog_residue=0 relation_residue=0 \
         cluster_healthy=true owner_fallback_identity=true \
         recovery_build_state=Ready recovery_digest={create_recovery_digest}"
    ));
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} \
         scenario=enospc_provider_fault_marker phase=create {create_marker}"
    ));

    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the create-ENOSPC recovery replica");
    }
    let data_marker_start = fs::read_to_string(&fixture.marker_file)?.lines().count();
    fs::write(&fixture.arm_file, "data\n")?;
    let injected = coordinator
        .query_one(
            "SELECT ec_distann_build_traversal_replica('dm_idx'::regclass)",
            &[],
        )
        .await;
    let disarm = fs::remove_file(&fixture.arm_file);
    disarm.wrap_err("disarming the Task 199 mid-copy ENOSPC provider")?;
    let data_error =
        injected.expect_err("data-armed replica build must fail under injected ENOSPC");
    let (data_faults, data_marker) = verify_failure(
        coordinator,
        corpus,
        queries,
        owner_baseline,
        fixture,
        data_marker_start,
        "data",
        &data_error,
    )
    .await?;

    let original_tablespace = original_tablespace.replace('"', "\"\"");
    coordinator
        .batch_execute(&format!(
            "ALTER INDEX dm_idx SET TABLESPACE \"{original_tablespace}\""
        ))
        .await
        .wrap_err("restoring the Task 199 index tablespace after ENOSPC drills")?;
    coordinator
        .batch_execute("DROP TABLESPACE task199_replica_enospc")
        .await
        .wrap_err("dropping the Task 199 ENOSPC tablespace")?;
    let data_recovery_digest =
        build_and_attest_traversal_replica(coordinator, scale, lines).await?;
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} \
         scenario=enospc_midcopy_cleanup pass=true sqlstate=53100 \
         provider_fault_events={data_faults} errno=28 \
         eligible_partial_images=0 catalog_residue=0 relation_residue=0 \
         cluster_healthy=true owner_fallback_identity=true \
         tablespace_restored=true recovery_build_state=Ready \
         recovery_digest={data_recovery_digest}"
    ));
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} \
         scenario=enospc_provider_fault_marker phase=data {data_marker}"
    ));
    Ok(())
}

async fn task199_physical_graph_digest(coordinator: &tokio_postgres::Client) -> Result<String> {
    Ok(coordinator
        .query_one(
            "SELECT encode(graph_digest, 'hex')
               FROM ec_distann_epoch_topology(
                   'dm_idx'::regclass,
                   (
                       SELECT epoch_fingerprint
                         FROM ec_distann_active_epoch
                        WHERE index_oid = 'dm_idx'::regclass::oid
                   )
               )",
            &[],
        )
        .await?
        .get::<_, String>(0))
}

async fn task199_participant_tombstone_invalidation_drill(
    coordinator: &tokio_postgres::Client,
) -> Result<String> {
    let replica_relation = coordinator
        .query_one(
            "SELECT replica_relid::regclass::text
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Ready'",
            &[],
        )
        .await?
        .get::<_, String>(0);
    let vec_id = coordinator
        .query_one(
            &format!(
                "SELECT vec_id
                   FROM {replica_relation}
                  WHERE owner_ordinal = 0
                  ORDER BY vec_id
                  LIMIT 1"
            ),
            &[],
        )
        .await?
        .get::<_, i64>(0);
    let before_digest = task199_physical_graph_digest(coordinator).await?;
    let invalidation = coordinator
        .query_one(
            "SELECT ec_distann_apply_record_writes(
                 'dm_idx'::regclass,
                 (
                     SELECT epoch_fingerprint
                       FROM ec_distann_active_epoch
                      WHERE index_oid = 'dm_idx'::regclass::oid
                 ),
                 ARRAY[$1]::bigint[]
             )",
            &[&vec_id],
        )
        .await
        .expect_err("participant tombstone endpoint must invalidate the Ready replica");
    let retryable = invalidation
        .code()
        .is_some_and(|code| code.code() == "40001")
        && invalidation
            .as_db_error()
            .is_some_and(|error| error.message().contains("EC_REPLICA_INVALIDATED"));
    let after_digest = task199_physical_graph_digest(coordinator).await?;
    let generation_unchanged = after_digest == before_digest;
    let stale = coordinator
        .query_one(
            "SELECT count(*) = 1
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Stale'",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    if !retryable || !generation_unchanged || !stale {
        bail!(
            "Task 199 participant tombstone invalidation failed: retryable={retryable} \
             generation_unchanged={generation_unchanged} stale={stale}"
        );
    }
    Ok(format!(
        "scenario=participant_tombstone_durable_invalidation pass=true vec_id={vec_id} \
         owner_ordinal=0 first_sqlstate=40001 token=EC_REPLICA_INVALIDATED \
         state=Stale generation_unchanged=true tombstoned_rows=0"
    ))
}

async fn build_and_attest_traversal_replica(
    coordinator: &tokio_postgres::Client,
    scale: &str,
    lines: &mut Vec<String>,
) -> Result<String> {
    coordinator
        .query_one(
            "SELECT ec_distann_traversal_replica_control_preflight('dm_idx'::regclass)",
            &[],
        )
        .await
        .wrap_err("preflighting traversal-replica control connection")?;
    let replica_started = Instant::now();
    let replica = coordinator
        .query_one(
            "SELECT encode(ec_distann_build_traversal_replica('dm_idx'::regclass), 'hex')",
            &[],
        )
        .await
        .wrap_err("building coordinator traversal replica")?
        .get::<_, String>(0);
    let status = coordinator
        .query_one(
            "SELECT state, owner_count, expected_record_count,
                    copied_record_count, copied_bytes, peak_copy_batch_bytes,
                    relation_bytes, coalesce(wal_bytes, 0),
                    encode(content_digest, 'hex'), state_reason,
                    coalesce(build_duration_ms, 0), active_pins, reclaim_eligible
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Ready'
              ORDER BY ready_at DESC
              LIMIT 1",
            &[],
        )
        .await
        .wrap_err("attesting coordinator traversal replica")?;
    let attested_digest = status.get::<_, String>(8);
    if status.get::<_, String>(0) != "Ready"
        || status.get::<_, String>(9) != "ready"
        || status.get::<_, i64>(11) != 0
        || status.get::<_, bool>(12)
        || attested_digest != replica
    {
        bail!("traversal replica build did not attest one matching unpinned Ready image");
    }
    lines.push(format!(
        "physical_benchmark_traversal_replica scale={scale} state=Ready owners={} expected_records={} copied_records={} copied_bytes={} peak_copy_batch_bytes={} relation_bytes={} wal_bytes={} build_ms={} catalog_build_ms={} active_pins=0 reclaim_eligible=false content_digest={replica}",
        status.get::<_, i32>(1),
        status.get::<_, i64>(2),
        status.get::<_, i64>(3),
        status.get::<_, i64>(4),
        status.get::<_, i64>(5),
        status.get::<_, i64>(6),
        status.get::<_, i64>(7),
        replica_started.elapsed().as_millis(),
        status.get::<_, i64>(10),
    ));
    let replay = coordinator
        .query_one(
            "SELECT encode(ec_distann_build_traversal_replica('dm_idx'::regclass), 'hex')",
            &[],
        )
        .await
        .wrap_err("replaying coordinator traversal replica build")?
        .get::<_, String>(0);
    if replay != replica {
        bail!("idempotent traversal replica build returned a different digest");
    }
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} scenario=idempotent_build_replay pass=true content_digest={replica}"
    ));
    Ok(replica)
}

async fn task199_no_replica_insert_throughput(
    coordinator: &tokio_postgres::Client,
    scale: &str,
    source_table: &str,
    lines: &mut Vec<String>,
) -> Result<()> {
    const TRIALS: usize = 5;
    const ROWS_PER_TRIAL: u64 = 2_000;
    let ready_count = coordinator
        .query_one(
            "SELECT count(*)::bigint
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Ready'",
            &[],
        )
        .await?
        .get::<_, i64>(0);
    if ready_count != 0 {
        bail!("Task 199 no-replica insert benchmark found a Ready replica");
    }

    let mut elapsed_ns = Vec::with_capacity(TRIALS);
    for trial in 0..TRIALS {
        coordinator
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS task199_no_replica_insert_probe;
                 CREATE TABLE task199_no_replica_insert_probe (
                     id bigint, source real[], embedding ecvector
                 );
                 INSERT INTO task199_no_replica_insert_probe
                 SELECT id, source, embedding
                   FROM {source_table}
                  ORDER BY id
                  LIMIT 128;
                 CREATE INDEX task199_no_replica_insert_probe_idx
                    ON task199_no_replica_insert_probe
                 USING ec_distann (embedding ecvector_distann_ip_ops)
                  WITH (graph_degree = 32, head_index_cap = 128,
                        neighbor_code_format = 'rabitq');"
            ))
            .await?;
        let started = Instant::now();
        let inserted = coordinator
            .execute(
                &format!(
                    "INSERT INTO task199_no_replica_insert_probe
                     SELECT (1000000000 + {} * {ROWS_PER_TRIAL}
                             + row_number() OVER (ORDER BY id))::bigint,
                            source, embedding
                       FROM {source_table}
                      ORDER BY id
                      LIMIT {ROWS_PER_TRIAL}",
                    trial
                ),
                &[],
            )
            .await?;
        let nanos = started.elapsed().as_nanos();
        if inserted != ROWS_PER_TRIAL {
            bail!(
                "Task 199 no-replica insert trial {trial} inserted {inserted}, expected {ROWS_PER_TRIAL}"
            );
        }
        elapsed_ns.push(nanos);
    }
    coordinator
        .batch_execute("DROP TABLE task199_no_replica_insert_probe")
        .await?;
    elapsed_ns.sort_unstable();
    let median_ns = elapsed_ns[TRIALS / 2];
    let rows_per_second = ROWS_PER_TRIAL as f64 * 1_000_000_000.0 / median_ns as f64;
    lines.push(format!(
        "physical_benchmark_no_replica_insert scale={scale} pass=true \
         ready_replica_absent=true trials={TRIALS} rows_per_trial={ROWS_PER_TRIAL} \
         median_ns={median_ns} rows_per_second={rows_per_second:.3}"
    ));
    Ok(())
}

/// Task 167 insert-throughput and backlink-strategy A/B. The candidate gives
/// established neighbors priority over a proposed backlink on exact-distance
/// ties; it is measured against the same-generation local control, then quality
/// is gated before the rejected append-when-room control mutates the disposable
/// physical fixture. Every trial uses a disjoint ID range, so no arm includes
/// cleanup or tombstone work from another arm.
const TASK167_AB_TRIALS: usize = 5;
const TASK167_AB_ROWS_PER_TRIAL: usize = 32;
const TASK167_AB_SAMPLE_ROWS: usize = TASK167_AB_TRIALS * TASK167_AB_ROWS_PER_TRIAL;

#[derive(Debug, Clone, Copy)]
struct Task167InsertMeasurement {
    rows_per_second: f64,
    inserted_rows: usize,
}

#[derive(Debug, Clone, Copy)]
struct Task167DefaultInsertBaseline {
    measurement: Task167InsertMeasurement,
    backlink_amendments: i64,
    backlink_no_room: i64,
}

fn task167_insert_trial_items(
    trial: usize,
    rows_per_trial: usize,
) -> impl Iterator<Item = (usize, usize)> {
    (0..rows_per_trial).map(move |ordinal| (ordinal, trial * rows_per_trial + ordinal))
}

async fn measure_task167_insert_arm(
    coordinator: &tokio_postgres::Client,
    table: &str,
    physical_corpus: &str,
    physical: bool,
    id_base: i64,
    trials: usize,
    rows_per_trial: usize,
) -> Result<Task167InsertMeasurement> {
    let mut trial_rows_per_second = Vec::with_capacity(trials);
    let mut inserted_rows = 0;
    for trial in 0..trials {
        let started = Instant::now();
        for (ordinal, source_offset) in task167_insert_trial_items(trial, rows_per_trial) {
            let id = id_base + trial as i64 * rows_per_trial as i64 + ordinal as i64;
            let sql = if physical {
                format!(
                    "INSERT INTO {table} (id, source_id, source, embedding) \
                     SELECT {id}, (substr(md5({id}::text),1,8)||'-'||substr(md5({id}::text),9,4)||'-4'||\
                            substr(md5({id}::text),14,3)||'-8'||substr(md5({id}::text),18,3)||'-'||\
                            substr(md5({id}::text),21,12))::uuid, source, \
                            encode_to_ecvector(source, 4, 42) \
                       FROM {physical_corpus} ORDER BY id OFFSET {source_offset} LIMIT 1"
                )
            } else {
                format!(
                    "INSERT INTO {table} (id, source, embedding) \
                     SELECT {id}, source, encode_to_ecvector(source, 4, 42) \
                       FROM {physical_corpus} ORDER BY id OFFSET {source_offset} LIMIT 1"
                )
            };
            let inserted = coordinator.execute(&sql, &[]).await?;
            if inserted != 1 {
                bail!(
                    "Task 167 {table} trial {trial} inserted {inserted} rows for ordinal {ordinal}"
                );
            }
            inserted_rows += usize::try_from(inserted).unwrap_or(usize::MAX);
        }
        let elapsed_ns = started.elapsed().as_nanos().max(1) as f64;
        trial_rows_per_second.push(rows_per_trial as f64 * 1_000_000_000.0 / elapsed_ns);
    }
    trial_rows_per_second.sort_by(f64::total_cmp);
    Ok(Task167InsertMeasurement {
        rows_per_second: trial_rows_per_second[trials / 2],
        inserted_rows,
    })
}

async fn task167_default_insert_throughput(
    coordinator: &tokio_postgres::Client,
    scale: &str,
    physical_corpus: &str,
    single_corpus: &str,
    graph_degree: u32,
    lines: &mut Vec<String>,
) -> Result<Task167DefaultInsertBaseline> {
    let single = measure_task167_insert_arm(
        coordinator,
        single_corpus,
        physical_corpus,
        false,
        1_000_000,
        TASK167_AB_TRIALS,
        TASK167_AB_ROWS_PER_TRIAL,
    )
    .await?;
    coordinator
        .batch_execute("RESET ec_distann.debug_disable_append_when_room")
        .await
        .wrap_err("selecting the shipped robust-prune insert strategy")?;
    coordinator
        .batch_execute("SELECT ec_distann_insert_work_reset()")
        .await
        .wrap_err("resetting Task 167 physical insert-work counters")?;
    let shipped = measure_task167_insert_arm(
        coordinator,
        physical_corpus,
        physical_corpus,
        true,
        2_000_000,
        TASK167_AB_TRIALS,
        TASK167_AB_ROWS_PER_TRIAL,
    )
    .await?;
    let shipped_work = coordinator
        .query(
            "SELECT metric, inserts, value, mean_per_insert
               FROM ec_distann_insert_work_snapshot()
              ORDER BY metric",
            &[],
        )
        .await
        .wrap_err("reading shipped-default Task 167 insert-work counters")?;
    coordinator
        .batch_execute("RESET ec_distann.debug_disable_append_when_room")
        .await
        .wrap_err("restoring the shipped robust-prune strategy")?;
    if single.inserted_rows != TASK167_AB_SAMPLE_ROWS
        || shipped.inserted_rows != TASK167_AB_SAMPLE_ROWS
    {
        bail!(
            "Task 167 default insert measurement executed unexpected statement counts: single={} shipped={} preregistered={TASK167_AB_SAMPLE_ROWS}",
            single.inserted_rows,
            shipped.inserted_rows,
        );
    }
    let ratio = shipped.rows_per_second / single.rows_per_second.max(f64::EPSILON);
    lines.push(format!(
        "physical_benchmark_insert_throughput_ab scale={scale} physical_table={physical_corpus} control_table={single_corpus} trials={TASK167_AB_TRIALS} rows_per_trial={TASK167_AB_ROWS_PER_TRIAL} sample_rows={} workload=single_row_insert physical_insert_mode=shipped_default_established_tie_priority physical_rows_per_second={:.3} control_rows_per_second={:.3} physical_over_control={ratio:.6} pass=true",
        shipped.inserted_rows,
        shipped.rows_per_second,
        single.rows_per_second,
    ));
    let rows = shipped_work;
    let expected_inserts = shipped.inserted_rows as i64;
    if rows.len() != 8 {
        bail!(
            "Task 167 physical insert-work snapshot returned {} rows, expected 8",
            rows.len()
        );
    }
    let mut values = HashMap::new();
    for row in rows {
        let metric = row.get::<_, String>(0);
        let inserts = row.get::<_, i64>(1);
        let value = row.get::<_, i64>(2);
        let mean = row.get::<_, f64>(3);
        if inserts != expected_inserts {
            bail!(
                "Task 167 physical insert-work metric {metric} counted {inserts} attempts, expected {expected_inserts}"
            );
        }
        values.insert(metric.clone(), (value, mean));
        lines.push(format!(
            "physical_benchmark_insert_work scale={scale} insert_mode=shipped_default_established_tie_priority metric={metric} inserts={inserts} value={value} mean_per_insert={mean:.6} graph_degree={graph_degree} counter_scope=coordinator_backend remote_owner_work_included=false pass=true"
        ));
    }
    let bound = i64::from(graph_degree) * expected_inserts;
    for metric in ["forward_neighbors_selected", "backlink_amendments"] {
        let (value, _) = values
            .get(metric)
            .copied()
            .ok_or_else(|| eyre!("insert-work snapshot omitted {metric}"))?;
        if value > bound {
            bail!("Task 167 {metric} exceeded graph-degree bound: value={value} bound={bound}");
        }
    }
    Ok(Task167DefaultInsertBaseline {
        measurement: shipped,
        backlink_amendments: values
            .get("backlink_amendments")
            .map(|(value, _)| *value)
            .unwrap_or_default(),
        backlink_no_room: values
            .get("backlink_no_room")
            .map(|(value, _)| *value)
            .unwrap_or_default(),
    })
}

async fn task167_append_when_room_diagnostic(
    coordinator: &tokio_postgres::Client,
    scale: &str,
    physical_corpus: &str,
    baseline: Task167DefaultInsertBaseline,
    lines: &mut Vec<String>,
) -> Result<()> {
    coordinator
        .batch_execute("SET ec_distann.debug_disable_append_when_room = off")
        .await
        .wrap_err("enable append-when-room A/B candidate")?;
    coordinator
        .batch_execute("SELECT ec_distann_insert_work_reset()")
        .await
        .wrap_err("reset append-when-room candidate counters")?;
    let append_when_room = measure_task167_insert_arm(
        coordinator,
        physical_corpus,
        physical_corpus,
        true,
        3_000_000,
        TASK167_AB_TRIALS,
        TASK167_AB_ROWS_PER_TRIAL,
    )
    .await?;
    let append_work = coordinator
        .query(
            "SELECT metric, value FROM ec_distann_insert_work_snapshot() ORDER BY metric",
            &[],
        )
        .await?
        .into_iter()
        .map(|row| (row.get::<_, String>(0), row.get::<_, i64>(1)))
        .collect::<HashMap<_, _>>();
    coordinator
        .batch_execute("RESET ec_distann.debug_disable_append_when_room")
        .await
        .wrap_err("restoring the shipped robust-prune strategy")?;
    if append_when_room.inserted_rows != TASK167_AB_SAMPLE_ROWS {
        bail!(
            "Task 167 append-when-room diagnostic inserted {} rows, expected {TASK167_AB_SAMPLE_ROWS}",
            append_when_room.inserted_rows,
        );
    }
    let append_ratio =
        append_when_room.rows_per_second / baseline.measurement.rows_per_second.max(f64::EPSILON);
    let append_amendments = append_work
        .get("backlink_amendments")
        .copied()
        .unwrap_or_default();
    let append_candidate_faster =
        append_ratio >= 1.0 && append_amendments <= baseline.backlink_amendments;
    lines.push(format!(
        "physical_benchmark_backlink_strategy_ab scale={scale} trials={TASK167_AB_TRIALS} rows_per_trial={TASK167_AB_ROWS_PER_TRIAL} sample_rows={} shipped_default_established_tie_rows_per_second={:.3} append_when_room_rows_per_second={:.3} append_when_room_over_shipped_default={append_ratio:.6} shipped_default_backlink_amendments={} append_when_room_backlink_amendments={append_amendments} shipped_default_backlink_no_room={} append_when_room_backlink_no_room={} counter_scope=coordinator_backend remote_owner_work_included=false comparison=sequential_same_fixture measurement_order=after_shipped_default_quality_gate control_graph_mutation_excluded_from_quality_gate=true control_faster={append_candidate_faster} pass=true",
        append_when_room.inserted_rows,
        baseline.measurement.rows_per_second,
        append_when_room.rows_per_second,
        baseline.backlink_amendments,
        baseline.backlink_no_room,
        append_work.get("backlink_no_room").copied().unwrap_or_default(),
    ));
    Ok(())
}

const TASK167_INSERTED_QUALITY_QUERIES: usize = 48;
// Packet 045 preregistered this AC-4 inserted-neighborhood band from five
// isolated 10k production repeats. Heldout is deliberately not represented by
// a cross-scale constant: its regression band is supplied per suite step from
// the shipped-default baseline at that scale.
const TASK167_INSERTED_QUALITY_ALLOWED_DEFICIT: f64 = 0.015;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Task167HeldoutRegressionGate {
    baseline_deficit: f64,
    physical_sample_sd: f64,
    allowed_deficit: f64,
}

fn task167_heldout_regression_gate(
    baseline_deficit: Option<f64>,
    physical_sample_sd: Option<f64>,
) -> Result<Option<Task167HeldoutRegressionGate>> {
    let (baseline_deficit, physical_sample_sd) = match (baseline_deficit, physical_sample_sd) {
        (None, None) => return Ok(None),
        (Some(baseline_deficit), Some(physical_sample_sd)) => {
            (baseline_deficit, physical_sample_sd)
        }
        _ => bail!(
            "Task 167 heldout regression gate requires both the per-scale baseline deficit and physical sample SD"
        ),
    };
    if !baseline_deficit.is_finite() || baseline_deficit < 0.0 {
        bail!("Task 167 heldout baseline deficit must be finite and non-negative");
    }
    if !physical_sample_sd.is_finite() || physical_sample_sd < 0.0 {
        bail!("Task 167 heldout physical sample SD must be finite and non-negative");
    }
    let allowed_deficit = baseline_deficit + 2.0 * physical_sample_sd;
    if !allowed_deficit.is_finite() {
        bail!("Task 167 heldout regression band overflowed");
    }
    Ok(Some(Task167HeldoutRegressionGate {
        baseline_deficit,
        physical_sample_sd,
        allowed_deficit,
    }))
}

fn task167_search_guc_sql(
    args: &LocalMultinodePg18Args,
    production: &BenchmarkSeedVariant,
    beam_width: u32,
    candidate_heap_limit: u32,
    hop_rounds: u32,
) -> Result<String> {
    let beam_width = production.beam_width.unwrap_or(beam_width);
    let hop_rounds = production.hop_rounds.unwrap_or(hop_rounds);
    let mut sql = format!(
        "SET enable_seqscan = off;\
         SET ec_distann.beam_width = {beam_width};\
         SET ec_distann.candidate_heap_limit = {candidate_heap_limit};\
         SET ec_distann.hop_rounds = {hop_rounds};\
         SET ec_distann.top_k = {};",
        args.top_k,
    );
    // Match the ordinary recall child's precedence: general caller-supplied
    // assignments follow the base budgets, then the named variant controls
    // below win if the same setting was supplied twice.
    for assignment in &args.bench_session_gucs {
        let (name, value) = assignment.split_once('=').ok_or_else(|| {
            eyre!("Task 167 benchmark session GUC must be NAME=VALUE: {assignment:?}")
        })?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'.')
            || value.is_empty()
        {
            bail!("Task 167 benchmark session GUC is not a safe NAME=VALUE assignment: {assignment:?}");
        }
        sql.push_str(&format!(" SET {name} = {value};"));
    }
    let strategy = production.strategy.replace('\'', "''");
    sql.push_str(&format!(
        " SET ec_distann.benchmark_seed_mode = '{strategy}';\
          SET ec_distann.benchmark_head_search_width = {};\
          SET ec_distann.benchmark_head_seed_count = {};\
          SET ec_distann.benchmark_exact_neighbor = {};\
          SET ec_distann.benchmark_materialization_batch_size = {};\
          SET ec_distann.benchmark_owner_payload_plan_cache = {};\
          SET ec_distann.benchmark_typed_locator = {};\
          SET ec_distann.benchmark_packed_payload = {};\
          SET ec_distann.benchmark_expanded_locator = {};\
          SET ec_distann.allow_nonconforming_replica = {};\
          SET ec_distann.sharded_head_search = {};\
          SET ec_distann.head_replica_count = {};\
          SET ec_distann.gateway_copy_capacity = {};\
          SET ec_distann.crown_capacity = {};\
          SET ec_distann.crown_width_pruning = {};\
          SET ec_distann.fused_head_hop = {};",
        production.head_search_width,
        production.head_seed_count,
        if production.neighbor_score_mode == "exact_neighbor" {
            "on"
        } else {
            "off"
        },
        production.materialization_batch_size,
        if production.owner_payload_plan_cache {
            "on"
        } else {
            "off"
        },
        if production.typed_locator {
            "on"
        } else {
            "off"
        },
        if production.packed_payload {
            "on"
        } else {
            "off"
        },
        if production.expanded_locator {
            "on"
        } else {
            "off"
        },
        if production.traversal_replica {
            "on"
        } else {
            "off"
        },
        if args.local_head { "off" } else { "on" },
        args.head_replica_count.unwrap_or(0),
        args.gateway_copy_capacity.unwrap_or(0),
        args.crown_capacity.unwrap_or(0),
        if args.crown_width_pruning {
            "on"
        } else {
            "off"
        },
        if args.fused_head_hop { "on" } else { "off" },
    ));
    Ok(sql)
}

/// Compare the post-insert physical generation and a reloption-matched fresh
/// rebuild against the same brute-force fp32 ground truth. Pairwise ANN set
/// overlap is deliberately not a correctness metric: independently built
/// graphs may return different valid approximations. Source fingerprints
/// collapse exact duplicate rows, and each query divides by its own number of
/// distinct exact-truth keys so duplicate ties cannot lower the metric ceiling.
/// The inserted-neighborhood population retains the AC-4 non-inferiority band
/// preregistered in packet 045. Heldout is a scale-specific regression detector:
/// a suite step either supplies its shipped-default baseline and physical-arm
/// sample SD, or records a non-blocking baseline observation. Both population
/// rows are returned even when an applied gate fails so the caller can write the
/// packet summary before exiting nonzero. Missing rows, wrong plans, malformed
/// truth, and other measurement-integrity failures still fail immediately.
async fn task167_post_insert_exact_recall(
    coordinator: &tokio_postgres::Client,
    scale: &str,
    physical_corpus: &str,
    roster: &str,
    graph_degree: u32,
    head_index_cap: u32,
    build_shards: u32,
    head_construction: &str,
    head_sizing: &str,
    query_count: u32,
    truth_corpus_path: &Path,
    truth_queries_path: &Path,
    search_guc_sql: &str,
    graph_phase: &str,
    inserted_id_bases: &[i64],
    heldout_baseline_deficit: Option<f64>,
    heldout_physical_sample_sd: Option<f64>,
) -> Result<Vec<String>> {
    if inserted_id_bases.is_empty() {
        bail!("Task 167 exact-recall gate requires at least one inserted ID range");
    }
    let heldout_count = usize::try_from(query_count).unwrap_or(usize::MAX);
    if heldout_count <= TASK167_INSERTED_QUALITY_QUERIES {
        bail!(
            "Task 167 exact-recall quality sample requires held-out queries to dominate: inserted={} heldout={heldout_count}",
            TASK167_INSERTED_QUALITY_QUERIES,
        );
    }
    let query_count = heldout_count + TASK167_INSERTED_QUALITY_QUERIES;
    let heldout_gate =
        task167_heldout_regression_gate(heldout_baseline_deficit, heldout_physical_sample_sd)?;

    let fresh_table = format!("task167_fresh_{scale}");
    let fresh_index = format!("{fresh_table}_idx");
    coordinator
        .batch_execute(&format!(
            "SET ec_distann.roster = '';
             DROP TABLE IF EXISTS {fresh_table} CASCADE;
             CREATE TABLE {fresh_table} AS
               SELECT id, source, embedding FROM {physical_corpus};
             CREATE INDEX {fresh_index} ON {fresh_table}
               USING ec_distann (embedding ecvector_distann_ip_ops)
               WITH (distributed_control = false, graph_degree = {graph_degree}, head_index_cap = {head_index_cap},
                     build_shards = {build_shards}, head_construction = '{head_construction}',
                     neighbor_code_format = 'rabitq'{head_sizing});
             ANALYZE {fresh_table};"
        ))
        .await
        .wrap_err("building Task 167 post-insert fresh rebuild")?;

    let (mut corpus_ids, base_corpus) =
        crate::commands::bench::recall::load_sources_tsv_file(truth_corpus_path)
            .wrap_err("loading Task 167 exact-truth corpus")?;
    if !corpus_ids.windows(2).all(|ids| ids[0] < ids[1]) {
        bail!(
            "Task 167 exact-truth corpus IDs must be strictly increasing to match INSERT ... ORDER BY id offsets"
        );
    }
    if base_corpus.nrows() < TASK167_AB_SAMPLE_ROWS
        || base_corpus.nrows() < TASK167_INSERTED_QUALITY_QUERIES
    {
        bail!(
            "Task 167 exact-truth corpus has {} rows, expected at least {}",
            base_corpus.nrows(),
            TASK167_AB_SAMPLE_ROWS.max(TASK167_INSERTED_QUALITY_QUERIES),
        );
    }
    let (_, heldout_queries) =
        crate::commands::bench::recall::load_sources_tsv_file(truth_queries_path)
            .wrap_err("loading Task 167 held-out exact-recall queries")?;
    if heldout_queries.nrows() < heldout_count || heldout_queries.ncols() != base_corpus.ncols() {
        bail!(
            "Task 167 held-out query shape is {}x{}, expected at least {}x{}",
            heldout_queries.nrows(),
            heldout_queries.ncols(),
            heldout_count,
            base_corpus.ncols(),
        );
    }

    let dimension = base_corpus.ncols();
    let mut corpus_values = base_corpus
        .as_slice()
        .ok_or_else(|| eyre!("Task 167 base corpus is not contiguous"))?
        .to_vec();
    for id_base in inserted_id_bases {
        for source_offset in 0..TASK167_AB_SAMPLE_ROWS {
            corpus_ids.push(*id_base + source_offset as i64);
            corpus_values.extend(base_corpus.row(source_offset).iter().copied());
        }
    }
    let exact_corpus =
        ndarray::Array2::from_shape_vec((corpus_ids.len(), dimension), corpus_values)?;

    let mut query_populations = Vec::with_capacity(query_count);
    let mut query_values = Vec::with_capacity(query_count * dimension);
    for source_offset in 0..TASK167_INSERTED_QUALITY_QUERIES {
        query_populations.push("inserted_neighborhood");
        query_values.extend(base_corpus.row(source_offset).iter().copied());
    }
    for query_offset in 0..heldout_count {
        query_populations.push("heldout");
        query_values.extend(heldout_queries.row(query_offset).iter().copied());
    }
    let exact_queries = ndarray::Array2::from_shape_vec((query_count, dimension), query_values)?;

    let truth =
        crate::commands::bench::recall::brute_force_top_k(&exact_corpus, &exact_queries, 10);
    let truth_indices = truth.indices;
    let corpus_keys = exact_corpus
        .outer_iter()
        .map(|source| task167_source_fingerprint(source.as_slice().expect("contiguous row")))
        .collect::<Vec<_>>();
    let key_by_id = corpus_ids
        .iter()
        .copied()
        .zip(corpus_keys.iter().copied())
        .collect::<HashMap<_, _>>();

    let epoch = coordinator
        .query_one(
            "SELECT epoch
               FROM ec_distann_active_epoch
              WHERE index_oid = 'public.dm_idx'::regclass::oid",
            &[],
        )
        .await
        .wrap_err("reading active epoch for Task 167 fresh-rebuild parity")?
        .get::<_, i64>(0);
    let roster = roster.replace('\'', "''");
    let physical_predictions = task167_ann_predictions(
        coordinator,
        physical_corpus,
        &exact_queries,
        &format!(
            "{search_guc_sql} SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch={epoch};"
        ),
        "EcDistannDistributedScan",
    )
    .await
    .wrap_err("running Task 167 incremental physical exact-recall arm")?;
    let fresh_predictions = task167_ann_predictions(
        coordinator,
        &fresh_table,
        &exact_queries,
        &format!(
            "{search_guc_sql} SET ec_distann.roster=''; SET ec_distann.local_node_id=1; SET ec_distann.epoch=0;"
        ),
        "Index Scan",
    )
    .await
    .wrap_err("running Task 167 fresh-rebuild exact-recall arm")?;

    let mut lines = Vec::new();
    for population in ["inserted_neighborhood", "heldout"] {
        let summary = task167_exact_recall_summary(
            population,
            &query_populations,
            &truth_indices,
            &physical_predictions,
            &fresh_predictions,
            &corpus_keys,
            &key_by_id,
        )?;
        let (allowed_deficit, baseline_deficit, physical_sample_sd, gate_mode, gate_source) =
            match population {
                "inserted_neighborhood" => (
                    Some(TASK167_INSERTED_QUALITY_ALLOWED_DEFICIT),
                    None,
                    None,
                    "ac4_absolute",
                    "packet_045_inserted_neighborhood_band",
                ),
                "heldout" => match heldout_gate {
                    Some(gate) => (
                        Some(gate.allowed_deficit),
                        Some(gate.baseline_deficit),
                        Some(gate.physical_sample_sd),
                        "baseline_relative",
                        "suite_step_per_scale_baseline_plus_2sd",
                    ),
                    None => (
                        None,
                        None,
                        None,
                        "baseline_recording",
                        "suite_step_baseline_observation",
                    ),
                },
                _ => bail!("Task 167 exact-recall gate has no policy for {population}"),
            };
        let quality_gate_pass = allowed_deficit.map(|allowed_deficit| {
            task167_exact_recall_within_allowed_deficit(
                summary.physical_recall,
                summary.fresh_recall,
                allowed_deficit,
            )
        });
        let step_pass = quality_gate_pass.unwrap_or(true);
        let allowed_deficit = allowed_deficit
            .map(|value| format!("{value:.6}"))
            .unwrap_or_else(|| "not_applicable".to_owned());
        let baseline_deficit = baseline_deficit
            .map(|value| format!("{value:.6}"))
            .unwrap_or_else(|| "not_recorded".to_owned());
        let physical_sample_sd = physical_sample_sd
            .map(|value| format!("{value:.6}"))
            .unwrap_or_else(|| "not_recorded".to_owned());
        let quality_gate_pass = quality_gate_pass
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not_applied".to_owned());
        let disposition = if population == "heldout" && heldout_gate.is_none() {
            "disclosed_baseline_characteristic"
        } else {
            "gate_applied"
        };
        let line = format!(
            "physical_benchmark_post_insert_exact_recall scale={scale} phase={graph_phase} population={population} queries={} top_k=10 truth=brute_force_fp32 denominator=per_query_distinct_exact_source_fingerprints truth_slots={} truth_distinct_keys={} truth_duplicate_slots={} physical_distinct_recall={:.6} fresh_distinct_recall={:.6} physical_minus_fresh={:.6} baseline_deficit={baseline_deficit} physical_sample_sd={physical_sample_sd} allowed_deficit={allowed_deficit} quality_gate_mode={gate_mode} quality_gate_source={gate_source} quality_gate_applied={} quality_gate_pass={quality_gate_pass} disposition={disposition} fresh_reloptions_matched=true heldout_query_set_matches_ordinary=true heldout_queries_dominate=true search_gucs_pinned=true diagnostic_control_mutation_excluded=true excluded_backlink_strategy=append_when_room measurement_complete=true pass={step_pass}",
            summary.queries,
            summary.truth_slots,
            summary.truth_distinct_keys,
            summary.truth_duplicate_slots,
            summary.physical_recall,
            summary.fresh_recall,
            summary.physical_recall - summary.fresh_recall,
            quality_gate_pass != "not_applied",
        );
        lines.push(line);
    }

    coordinator
        .batch_execute("SET ec_distann.roster = ''")
        .await
        .ok();
    coordinator
        .batch_execute(&format!("DROP TABLE {fresh_table} CASCADE"))
        .await
        .wrap_err("dropping Task 167 post-insert fresh rebuild")?;
    Ok(lines)
}

fn task167_quality_gate_failure(lines: &[String]) -> Option<String> {
    let failed = lines
        .iter()
        .filter(|line| {
            line.starts_with("physical_benchmark_post_insert_exact_recall ")
                && line.contains(" quality_gate_pass=false")
        })
        .cloned()
        .collect::<Vec<_>>();
    (!failed.is_empty()).then(|| {
        format!(
            "Task 167 post-insert exact recall failed its calibrated quality gate: {}",
            failed.join("; ")
        )
    })
}

fn enforce_task167_quality_gate(lines: &[String]) -> Result<()> {
    if let Some(failure) = task167_quality_gate_failure(lines) {
        bail!(failure);
    }
    Ok(())
}

async fn task167_ann_predictions(
    coordinator: &tokio_postgres::Client,
    table: &str,
    queries: &ndarray::Array2<f32>,
    setup_sql: &str,
    required_plan_node: &str,
) -> Result<Vec<Vec<i64>>> {
    coordinator.batch_execute(setup_sql).await?;
    let mut predictions = Vec::with_capacity(queries.nrows());
    for (query_index, query) in queries.outer_iter().enumerate() {
        let literal = task167_real_array_literal(
            query
                .as_slice()
                .ok_or_else(|| eyre!("Task 167 query row is not contiguous"))?,
        )?;
        let sql = format!("SELECT id FROM {table} ORDER BY embedding <#> {literal} LIMIT 10");
        if query_index == 0 {
            let plan = coordinator
                .query(&format!("EXPLAIN (FORMAT TEXT, COSTS OFF) {sql}"), &[])
                .await?
                .into_iter()
                .map(|row| row.get::<_, String>(0))
                .collect::<Vec<_>>()
                .join("\n");
            if !plan.contains(required_plan_node) {
                bail!(
                    "Task 167 exact-recall arm for {table} expected plan node {required_plan_node}: {plan}"
                );
            }
        }
        let ids = coordinator
            .query(&sql, &[])
            .await?
            .into_iter()
            .map(|row| row.get::<_, i64>(0))
            .collect::<Vec<_>>();
        if ids.len() != 10 {
            bail!(
                "Task 167 exact-recall arm for {table} query {query_index} returned {} rows, expected 10",
                ids.len(),
            );
        }
        predictions.push(ids);
    }
    Ok(predictions)
}

fn task167_real_array_literal(values: &[f32]) -> Result<String> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        bail!("Task 167 exact-recall query contains empty or non-finite source data");
    }
    Ok(format!(
        "ARRAY[{}]::real[]",
        values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn task167_source_fingerprint(source: &[f32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for value in source {
        hasher.update(value.to_bits().to_le_bytes());
    }
    hasher.finalize().into()
}

fn task167_distinct_recall<K>(truth: &[K], predicted: &[K]) -> (f64, usize, usize)
where
    K: Copy + Eq + std::hash::Hash,
{
    let truth_slots = truth.len();
    let truth = truth.iter().copied().collect::<HashSet<_>>();
    let predicted = predicted.iter().copied().collect::<HashSet<_>>();
    let distinct = truth.len();
    let duplicate_slots = truth_slots.saturating_sub(distinct);
    if distinct == 0 {
        return (0.0, 0, duplicate_slots);
    }
    let hits = truth.intersection(&predicted).count();
    (hits as f64 / distinct as f64, distinct, duplicate_slots)
}

#[derive(Debug)]
struct Task167ExactRecallSummary {
    queries: usize,
    truth_slots: usize,
    truth_distinct_keys: usize,
    truth_duplicate_slots: usize,
    physical_recall: f64,
    fresh_recall: f64,
}

fn task167_exact_recall_summary(
    population: &str,
    populations: &[&str],
    truth_indices: &[Vec<usize>],
    physical_predictions: &[Vec<i64>],
    fresh_predictions: &[Vec<i64>],
    corpus_keys: &[[u8; 32]],
    key_by_id: &HashMap<i64, [u8; 32]>,
) -> Result<Task167ExactRecallSummary> {
    let query_count = populations.len();
    if truth_indices.len() != query_count
        || physical_predictions.len() != query_count
        || fresh_predictions.len() != query_count
    {
        bail!(
            "Task 167 exact-recall input length mismatch: populations={query_count} truth={} physical={} fresh={}",
            truth_indices.len(),
            physical_predictions.len(),
            fresh_predictions.len(),
        );
    }
    let mut queries = 0;
    let mut truth_slots = 0;
    let mut truth_distinct_keys = 0;
    let mut physical_recall = 0.0;
    let mut fresh_recall = 0.0;
    for query_index in 0..populations.len() {
        if populations[query_index] != population {
            continue;
        }
        let truth_keys = truth_indices[query_index]
            .iter()
            .map(|index| {
                corpus_keys.get(*index).copied().ok_or_else(|| {
                    eyre!("Task 167 exact truth returned out-of-range corpus row {index}")
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let physical_keys = physical_predictions[query_index]
            .iter()
            .map(|id| {
                key_by_id
                    .get(id)
                    .copied()
                    .ok_or_else(|| eyre!("Task 167 physical prediction returned unknown id {id}"))
            })
            .collect::<Result<Vec<_>>>()?;
        let fresh_keys = fresh_predictions[query_index]
            .iter()
            .map(|id| {
                key_by_id
                    .get(id)
                    .copied()
                    .ok_or_else(|| eyre!("Task 167 fresh prediction returned unknown id {id}"))
            })
            .collect::<Result<Vec<_>>>()?;
        let (physical, distinct, _) = task167_distinct_recall(&truth_keys, &physical_keys);
        let (fresh, fresh_distinct, _) = task167_distinct_recall(&truth_keys, &fresh_keys);
        if fresh_distinct != distinct {
            bail!("Task 167 exact-recall denominator drifted between arms");
        }
        queries += 1;
        truth_slots += truth_keys.len();
        truth_distinct_keys += distinct;
        physical_recall += physical;
        fresh_recall += fresh;
    }
    if queries == 0 {
        bail!("Task 167 exact-recall population {population} has no queries");
    }
    Ok(Task167ExactRecallSummary {
        queries,
        truth_slots,
        truth_distinct_keys,
        truth_duplicate_slots: truth_slots.saturating_sub(truth_distinct_keys),
        physical_recall: physical_recall / queries as f64,
        fresh_recall: fresh_recall / queries as f64,
    })
}

fn task167_exact_recall_within_allowed_deficit(
    physical_recall: f64,
    fresh_recall: f64,
    allowed_deficit: f64,
) -> bool {
    fresh_recall - physical_recall <= allowed_deficit + f64::EPSILON
}

async fn retire_and_reclaim_traversal_replica(
    coordinator: &tokio_postgres::Client,
) -> Result<(bool, bool)> {
    coordinator
        .execute(
            "SELECT ec_distann_retire_traversal_replica('dm_idx'::regclass)",
            &[],
        )
        .await?;
    let retired = coordinator
        .query_one(
            "SELECT count(*) = 1
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Retiring' AND state_reason = 'explicit_retire'",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    let reclaimed = coordinator
        .query_one(
            "SELECT ec_distann_reclaim_traversal_replica('dm_idx'::regclass)",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    Ok((retired, reclaimed))
}

async fn task199_connect(
    coordinator_port: u16,
) -> Result<(tokio_postgres::Client, tokio::task::JoinHandle<()>)> {
    let (client, connection) = tokio_postgres::connect(
        &conninfo(Path::new(""), coordinator_port),
        tokio_postgres::NoTls,
    )
    .await
    .wrap_err("connecting Task 199 auxiliary coordinator session")?;
    let task = tokio::spawn(async move {
        let _ = connection.await;
    });
    Ok((client, task))
}

fn task199_decode_ordered_rows(rows: Vec<tokio_postgres::Row>) -> Vec<(i64, String, u32)> {
    rows.into_iter()
        .map(|row| {
            (
                row.get::<_, i64>(0),
                row.get::<_, String>(1),
                row.get::<_, f32>(2).to_bits(),
            )
        })
        .collect()
}

fn task199_ordered_scan_sql(corpus: &str, queries: &str, limit: u32) -> String {
    format!(
        "WITH query_vector AS (
             SELECT source AS query_embedding
               FROM {queries} ORDER BY id LIMIT 1
         )
         SELECT id, source_id::text,
                embedding <#> (SELECT query_embedding FROM query_vector) AS distance
           FROM {corpus}
          ORDER BY embedding <#> (SELECT query_embedding FROM query_vector)
          LIMIT {limit}"
    )
}

async fn task199_concurrent_build_mutation_drill(
    coordinator: &tokio_postgres::Client,
    coordinator_port: u16,
    corpus: &str,
) -> Result<(String, String)> {
    let (builder, builder_connection) = task199_connect(coordinator_port).await?;
    let builder_pid = builder
        .query_one("SELECT pg_backend_pid()", &[])
        .await?
        .get::<_, i32>(0);
    let build = tokio::spawn(async move {
        builder
            .query_one(
                "SELECT encode(
                     ec_distann_build_traversal_replica('dm_idx'::regclass),
                     'hex'
                 )",
                &[],
            )
            .await
            .map(|row| row.get::<_, String>(0))
    });
    let mut build_fence_observed = false;
    for _ in 0..100 {
        build_fence_observed = coordinator
            .query_one(
                "SELECT EXISTS (
                     SELECT 1 FROM pg_catalog.pg_locks
                      WHERE pid = $1
                        AND relation = 'dm_idx'::regclass::oid
                        AND mode = 'ShareRowExclusiveLock'
                        AND granted
                 )",
                &[&builder_pid],
            )
            .await?
            .get::<_, bool>(0);
        if build_fence_observed {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    if !build_fence_observed {
        let _ = build.await;
        builder_connection.abort();
        bail!("Task 199 could not observe the traversal-replica build fence");
    }

    let (mutator, mutator_connection) = task199_connect(coordinator_port).await?;
    let mutator_pid = mutator
        .query_one("SELECT pg_backend_pid()", &[])
        .await?
        .get::<_, i32>(0);
    let inserted_id = mutator
        .query_one(
            &format!("SELECT coalesce(max(id), 0) + 1 FROM {corpus}"),
            &[],
        )
        .await?
        .get::<_, i64>(0);
    let mutation_sql = task199_real_insert_sql(corpus);
    let mutation = tokio::spawn(async move { mutator.query_one(&mutation_sql, &[]).await });
    let mut mutation_waited = false;
    for _ in 0..100 {
        mutation_waited = coordinator
            .query_one(
                "SELECT EXISTS (
                     SELECT 1 FROM pg_catalog.pg_locks
                      WHERE pid = $1
                        AND relation = 'dm_idx'::regclass::oid
                        AND mode = 'RowExclusiveLock'
                        AND NOT granted
                 )",
                &[&mutator_pid],
            )
            .await?
            .get::<_, bool>(0);
        if mutation_waited {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let digest = build
        .await
        .wrap_err("joining Task 199 concurrent replica build")?
        .wrap_err("building traversal replica during the mutation-fence drill")?;
    builder_connection.abort();
    let mutation_error = mutation
        .await
        .wrap_err("joining the blocked Task 199 mutation")?
        .expect_err("blocked mutation must invalidate the newly Ready replica");
    let mutation_fenced = mutation_error
        .code()
        .is_some_and(|code| code.code() == "40001")
        && mutation_error
            .as_db_error()
            .is_some_and(|error| error.message().contains("EC_REPLICA_INVALIDATED"));
    mutator_connection.abort();
    let inserted_count = coordinator
        .query_one(
            &format!("SELECT count(*)::bigint FROM {corpus} WHERE id = $1"),
            &[&inserted_id],
        )
        .await?
        .get::<_, i64>(0);
    let stale = coordinator
        .query_one(
            "SELECT count(*) = 1
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Stale'",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    if !mutation_waited || !mutation_fenced || inserted_count != 0 || !stale {
        bail!(
            "Task 199 build/mutation fence failed: observed={build_fence_observed} \
             mutation_waited={mutation_waited} mutation_fenced={mutation_fenced} \
             inserted={inserted_count} stale={stale}"
        );
    }
    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the blocking-fence replica");
    }
    let rebuilt_digest = coordinator
        .query_one(
            "SELECT encode(
                 ec_distann_build_traversal_replica('dm_idx'::regclass),
                 'hex'
             )",
            &[],
        )
        .await?
        .get::<_, String>(0);
    if rebuilt_digest != digest {
        bail!("Task 199 blocking-fence rebuild returned a different content digest");
    }
    Ok((
        rebuilt_digest,
        format!(
            "scenario=blocking_build_mutation_fenced pass=true build_lock=ShareRowExclusiveLock waiting_lock=RowExclusiveLock mutation_sqlstate=40001 token=EC_REPLICA_INVALIDATED inserted_rows=0 state_after_error=Stale rebuilt=true"
        ),
    ))
}

async fn task199_inflight_scan_invalidation_drill(
    coordinator: &tokio_postgres::Client,
    coordinator_port: u16,
    corpus: &str,
    queries: &str,
    owner_baseline: &str,
) -> Result<String> {
    let scan_sql = task199_ordered_scan_sql(corpus, queries, 40);
    let (scan, scan_connection) = task199_connect(coordinator_port).await?;
    scan.batch_execute(
        "SET enable_seqscan = off;
         SET ec_distann.beam_width = 4;
         SET ec_distann.hop_rounds = 100;",
    )
    .await?;
    let baseline = task199_decode_ordered_rows(scan.query(&scan_sql, &[]).await?);
    scan.batch_execute(&format!(
        "BEGIN ISOLATION LEVEL READ COMMITTED;
         DECLARE task199_replica_cursor NO SCROLL CURSOR FOR {scan_sql}"
    ))
    .await?;
    let mut cursor_rows = task199_decode_ordered_rows(
        scan.query("FETCH FORWARD 10 FROM task199_replica_cursor", &[])
            .await?,
    );
    let active_pins = coordinator
        .query_one(
            "SELECT active_pins
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Ready'",
            &[],
        )
        .await?
        .get::<_, i64>(0);

    let (mutator, mutator_connection) = task199_connect(coordinator_port).await?;
    let inserted_id = mutator
        .query_one(
            &format!("SELECT coalesce(max(id), 0) + 1 FROM {corpus}"),
            &[],
        )
        .await?
        .get::<_, i64>(0);
    let invalidation = mutator
        .query_one(&task199_real_insert_sql(corpus), &[])
        .await
        .expect_err("in-flight scan mutation must invalidate the Ready replica");
    let first_retryable = invalidation
        .code()
        .is_some_and(|code| code.code() == "40001")
        && invalidation
            .as_db_error()
            .is_some_and(|error| error.message().contains("EC_REPLICA_INVALIDATED"));
    mutator_connection.abort();

    cursor_rows.extend(task199_decode_ordered_rows(
        scan.query("FETCH FORWARD ALL FROM task199_replica_cursor", &[])
            .await?,
    ));
    scan.batch_execute("COMMIT").await?;
    scan_connection.abort();
    let state = coordinator
        .query_one(
            "SELECT state
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              ORDER BY build_started_at DESC LIMIT 1",
            &[],
        )
        .await?
        .get::<_, String>(0);
    let inserted_count = coordinator
        .query_one(
            &format!("SELECT count(*)::bigint FROM {corpus} WHERE id = $1"),
            &[&inserted_id],
        )
        .await?
        .get::<_, i64>(0);
    let owner_after =
        task198_replica_semantic_result(coordinator, corpus, queries, -1, 0, 20).await?;
    let pass = active_pins > 0
        && first_retryable
        && state == "Stale"
        && inserted_count == 0
        && cursor_rows == baseline
        && owner_after == owner_baseline;
    if !pass {
        bail!(
            "Task 199 in-flight scan invalidation failed: pins={active_pins} \
             retryable={first_retryable} state={state} inserted={inserted_count} \
             cursor_identity={} owner_identity={}",
            cursor_rows == baseline,
            owner_after == owner_baseline,
        );
    }
    Ok(format!(
        "scenario=inflight_scan_invalidation pass=true active_pins={active_pins} cursor_rows={} first_sqlstate=40001 state=Stale inserted_rows=0 owner_fallback_identity=true",
        cursor_rows.len()
    ))
}

async fn task199_auth_failure_recovery_drill(
    coordinator: &tokio_postgres::Client,
    coordinator_port: u16,
    corpus: &str,
    queries: &str,
    owner_baseline: &str,
) -> Result<String> {
    const MISSING_PASSWORD_FILE: &str = "/task199-intentionally-missing-replica-control-password";
    coordinator
        .batch_execute(&format!(
            "ALTER SYSTEM SET ec_distann.replica_control_password_file = \
             '{MISSING_PASSWORD_FILE}'"
        ))
        .await?;
    coordinator
        .query_one("SELECT pg_reload_conf()", &[])
        .await?;
    let mut reloaded = false;
    for _ in 0..100 {
        let setting = coordinator
            .query_one(
                "SELECT current_setting(
                     'ec_distann.replica_control_password_file', true
                 )",
                &[],
            )
            .await?
            .get::<_, Option<String>>(0);
        if setting.as_deref() == Some(MISSING_PASSWORD_FILE) {
            reloaded = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let inserted_id = coordinator
        .query_one(
            &format!("SELECT coalesce(max(id), 0) + 1 FROM {corpus}"),
            &[],
        )
        .await?
        .get::<_, i64>(0);
    let auth_error = coordinator
        .query_one(&task199_real_insert_sql(corpus), &[])
        .await
        .expect_err("broken replica control authentication must fail closed");
    let auth_failed_closed = auth_error.as_db_error().is_some_and(|error| {
        error.message().contains("EC_REPLICA_CONTROL")
            && error.message().contains("password-file metadata failed")
    });
    let ready_after_failure = coordinator
        .query_one(
            "SELECT count(*) = 1
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Ready'",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    let replica_relation = coordinator
        .query_one(
            "SELECT format('%I.%I', namespace.nspname, relation.relname)
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass) status
               JOIN pg_class relation ON relation.oid = status.replica_relid
               JOIN pg_namespace namespace ON namespace.oid = relation.relnamespace
              WHERE status.state = 'Ready'",
            &[],
        )
        .await?
        .get::<_, String>(0);
    let (locker, locker_connection) = task199_connect(coordinator_port).await?;
    locker
        .batch_execute(&format!(
            "BEGIN;
             LOCK TABLE {replica_relation} IN ACCESS EXCLUSIVE MODE"
        ))
        .await?;
    coordinator.batch_execute("BEGIN READ ONLY").await?;
    let owner_fallback =
        task198_replica_semantic_result(coordinator, corpus, queries, -1, 0, 20).await;
    let _ = coordinator.batch_execute("ROLLBACK").await;
    locker.batch_execute("ROLLBACK").await?;
    locker_connection.abort();
    let double_demotion_fallback = owner_fallback? == owner_baseline
        && coordinator
            .query_one(
                "SELECT count(*) = 1
                   FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
                  WHERE state = 'Ready'",
                &[],
            )
            .await?
            .get::<_, bool>(0);
    coordinator
        .batch_execute("ALTER SYSTEM RESET ec_distann.replica_control_password_file")
        .await?;
    coordinator
        .query_one("SELECT pg_reload_conf()", &[])
        .await?;
    coordinator
        .batch_execute(&format!("TRUNCATE TABLE {replica_relation}"))
        .await?;
    let suppressed_fallback =
        task198_replica_semantic_result(coordinator, corpus, queries, -1, 0, 20).await?;
    let suppressed_repeat_control_attempt = suppressed_fallback == owner_baseline
        && coordinator
            .query_one(
                "SELECT count(*) = 1
                   FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
                  WHERE state = 'Ready'",
                &[],
            )
            .await?
            .get::<_, bool>(0);
    let recovered = coordinator
        .query_one(
            "SELECT ec_distann_recover_traversal_replica_invalidation(
                 'dm_idx'::regclass
             )",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    let state = coordinator
        .query_one(
            "SELECT state
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              ORDER BY build_started_at DESC LIMIT 1",
            &[],
        )
        .await?
        .get::<_, String>(0);
    let inserted_count = coordinator
        .query_one(
            &format!("SELECT count(*)::bigint FROM {corpus} WHERE id = $1"),
            &[&inserted_id],
        )
        .await?
        .get::<_, i64>(0);
    if !reloaded
        || !auth_failed_closed
        || !ready_after_failure
        || !double_demotion_fallback
        || !suppressed_repeat_control_attempt
        || !recovered
        || state != "Stale"
        || inserted_count != 0
    {
        bail!(
            "Task 199 authentication recovery failed: reloaded={reloaded} \
             failed_closed={auth_failed_closed} ready={ready_after_failure} \
             double_demotion_fallback={double_demotion_fallback} \
             suppressed_repeat={suppressed_repeat_control_attempt} \
             recovered={recovered} state={state} inserted={inserted_count}"
        );
    }
    Ok(
        "scenario=control_auth_failure_recovery pass=true failed_closed=true \
         state_after_failure=Ready double_demotion_failure_owner_fallback=true \
         read_only=true backend_build_suppression=true \
         repeated_control_attempt=false operator_recovery=true \
         state_after_recovery=Stale inserted_rows=0"
            .to_owned(),
    )
}

async fn task199_relation_lock_fallback_drill(
    coordinator: &tokio_postgres::Client,
    coordinator_port: u16,
    corpus: &str,
    queries: &str,
    owner_baseline: &str,
) -> Result<String> {
    let replica_relation = coordinator
        .query_one(
            "SELECT format('%I.%I', namespace.nspname, relation.relname)
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass) status
               JOIN pg_class relation ON relation.oid = status.replica_relid
               JOIN pg_namespace namespace ON namespace.oid = relation.relnamespace
              WHERE status.state = 'Ready'",
            &[],
        )
        .await?
        .get::<_, String>(0);
    let (locker, locker_connection) = task199_connect(coordinator_port).await?;
    locker
        .batch_execute(&format!(
            "BEGIN;
             LOCK TABLE {replica_relation} IN ACCESS EXCLUSIVE MODE"
        ))
        .await?;
    let fallback = task198_replica_semantic_result(coordinator, corpus, queries, -1, 0, 20).await?;
    locker.batch_execute("ROLLBACK").await?;
    locker_connection.abort();
    let status = coordinator
        .query_one(
            "SELECT state, state_reason
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              ORDER BY build_started_at DESC LIMIT 1",
            &[],
        )
        .await?;
    let state = status.get::<_, String>(0);
    let reason = status.get::<_, String>(1);
    let pass = fallback == owner_baseline
        && state == "Stale"
        && reason.contains("EC_REPLICA_RELATION_RACE");
    if !pass {
        bail!(
            "Task 199 relation-lock fallback failed: identity={} state={state} reason={reason}",
            fallback == owner_baseline
        );
    }
    Ok(
        "scenario=relation_lock_race_fallback pass=true nonblocking=true \
         owner_identity=true state=Stale token=EC_REPLICA_RELATION_RACE"
            .to_owned(),
    )
}

async fn task199_queued_ddl_lock_drill(
    coordinator: &tokio_postgres::Client,
    coordinator_port: u16,
) -> Result<String> {
    let (holder, holder_connection) = task199_connect(coordinator_port).await?;
    holder
        .batch_execute(
            "BEGIN;
             SELECT encode(
                 ec_distann_build_traversal_replica('dm_idx'::regclass),
                 'hex'
             );",
        )
        .await?;
    let (dropper, dropper_connection) = task199_connect(coordinator_port).await?;
    let dropper_pid = dropper
        .query_one("SELECT pg_backend_pid()", &[])
        .await?
        .get::<_, i32>(0);
    let cancel = dropper.cancel_token();
    let drop_task = tokio::spawn(async move { dropper.batch_execute("DROP INDEX dm_idx").await });
    let mut queued = false;
    for _ in 0..100 {
        queued = coordinator
            .query_one(
                "SELECT EXISTS (
                     SELECT 1 FROM pg_catalog.pg_locks
                      WHERE pid = $1
                        AND relation = 'dm_idx'::regclass::oid
                        AND mode = 'AccessExclusiveLock'
                        AND NOT granted
                 )",
                &[&dropper_pid],
            )
            .await?
            .get::<_, bool>(0);
        if queued {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    if !queued {
        cancel.cancel_query(tokio_postgres::NoTls).await?;
        let _ = drop_task.await;
        holder.batch_execute("ROLLBACK").await?;
        holder_connection.abort();
        dropper_connection.abort();
        bail!("Task 199 could not queue AccessExclusiveLock behind the holder");
    }
    let started = Instant::now();
    let guard_result = tokio::time::timeout(
        Duration::from_secs(2),
        holder.query_one(
            "SELECT ec_distann_guard_traversal_replica_mutation(
                 'dm_idx'::regclass
             )",
            &[],
        ),
    )
    .await;
    let bounded = match guard_result {
        Ok(Err(error)) => {
            error.code().is_some_and(|code| code.code() == "40001")
                && error
                    .as_db_error()
                    .is_some_and(|db| db.message().contains("EC_REPLICA_INVALIDATED"))
        }
        _ => false,
    };
    cancel.cancel_query(tokio_postgres::NoTls).await?;
    let _ = drop_task.await;
    holder.batch_execute("ROLLBACK").await?;
    holder_connection.abort();
    dropper_connection.abort();
    let index_survived = coordinator
        .query_one("SELECT to_regclass('dm_idx') IS NOT NULL", &[])
        .await?
        .get::<_, bool>(0);
    let stale = coordinator
        .query_one(
            "SELECT count(*) = 1
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Stale'",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    if !bounded || !index_survived || !stale {
        bail!(
            "Task 199 queued-DDL control transaction failed: bounded={bounded} \
             index_survived={index_survived} stale={stale}"
        );
    }
    Ok(format!(
        "scenario=queued_ddl_invalidation pass=true queued_access_exclusive=true \
         control_elapsed_ms={} first_sqlstate=40001 index_survived=true state=Stale",
        started.elapsed().as_millis()
    ))
}

async fn task199_epoch_turnover_drill(
    coordinator: &tokio_postgres::Client,
    scale: &str,
    corpus: &str,
    queries: &str,
    owner_baseline: &str,
    training_query_path: Option<&Path>,
    lines: &mut Vec<String>,
) -> Result<()> {
    let replica_oids = coordinator
        .query_one(
            "SELECT replica_relid::oid, directory_relid::oid
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Ready'",
            &[],
        )
        .await?;
    let replica_relid = replica_oids.get::<_, u32>(0);
    let directory_relid = replica_oids.get::<_, u32>(1);
    let current_epoch = coordinator
        .query_one(
            "SELECT epoch
               FROM ec_distann_active_epoch
              WHERE index_oid = 'dm_idx'::regclass::oid",
            &[],
        )
        .await?
        .get::<_, i64>(0);
    let successor_epoch = current_epoch + 1;
    let successor_build = "73737373-7373-4373-8373-737373737373";
    coordinator
        .batch_execute(&format!(
            "SELECT ec_distann_begin_epoch_build(
                 'dm_idx'::regclass, {successor_epoch},
                 '{successor_build}'::uuid
             )"
        ))
        .await?;
    if let Some(path) = training_query_path {
        let path = std::fs::canonicalize(path)?
            .display()
            .to_string()
            .replace('\'', "''");
        coordinator
            .batch_execute(&format!(
                "CREATE TEMP TABLE ec_distann_task199_training_stage (
                     load_ordinal bigserial, source_id bigint, vec text
                 );
                 COPY ec_distann_task199_training_stage (source_id, vec)
                   FROM '{path}' WITH (FORMAT text, DELIMITER E'\\t');
                 CREATE TEMP TABLE ec_distann_task199_training_queries AS
                 SELECT (load_ordinal - 200)::bigint AS training_ordinal,
                        translate(vec, '[]', '{{}}')::real[] AS vector
                   FROM ec_distann_task199_training_stage
                  WHERE load_ordinal BETWEEN 201 AND 400
                  ORDER BY load_ordinal;
                 DROP TABLE ec_distann_task199_training_stage;
                 SELECT ec_distann_build_epoch_with_training(
                     'dm_idx'::regclass, {successor_epoch},
                     '{successor_build}'::uuid,
                     'ec_distann_task199_training_queries'::regclass
                 );
                 DROP TABLE ec_distann_task199_training_queries;"
            ))
            .await?;
    } else {
        coordinator
            .batch_execute(&format!(
                "SELECT ec_distann_build_epoch(
                     'dm_idx'::regclass, {successor_epoch},
                     '{successor_build}'::uuid
                 )"
            ))
            .await?;
    }
    coordinator
        .batch_execute(&format!(
            "SELECT ec_distann_decide_epoch_publish(
                 'dm_idx'::regclass, '{successor_build}'::uuid
             );"
        ))
        .await?;
    coordinator
        .batch_execute(&format!(
            "SELECT ec_distann_recover_epoch_publish(
                 'dm_idx'::regclass, '{successor_build}'::uuid
             );"
        ))
        .await?;
    let retiring = coordinator
        .query_one(
            "SELECT state, state_reason
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              ORDER BY build_started_at DESC LIMIT 1",
            &[],
        )
        .await?;
    let retiring_state = retiring.get::<_, String>(0);
    let retiring_reason = retiring.get::<_, String>(1);
    let successor_owner =
        task198_replica_semantic_result(coordinator, corpus, queries, -1, 0, 20).await?;
    let reclaimed = coordinator
        .query_one(
            "SELECT ec_distann_reclaim_traversal_replica('dm_idx'::regclass)",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    let relation_residue = coordinator
        .query_one(
            "SELECT count(*)::bigint
               FROM pg_catalog.pg_class
              WHERE oid IN ($1::oid, $2::oid)",
            &[&replica_relid, &directory_relid],
        )
        .await?
        .get::<_, i64>(0);
    coordinator
        .batch_execute(&format!(
            "SELECT ec_distann_recover_epoch_publish(
                 'dm_idx'::regclass, '{successor_build}'::uuid
             )"
        ))
        .await?;
    let active_successor = coordinator
        .query_one(
            "SELECT build_id::text = $1
               FROM ec_distann_active_epoch
              WHERE index_oid = 'dm_idx'::regclass::oid",
            &[&successor_build],
        )
        .await?
        .get::<_, bool>(0);
    if retiring_state != "Retiring"
        || retiring_reason != "epoch_superseded"
        || successor_owner != owner_baseline
        || !reclaimed
        || relation_residue != 0
        || !active_successor
    {
        bail!(
            "Task 199 epoch turnover failed: state={retiring_state} \
             reason={retiring_reason} owner_identity={} reclaimed={reclaimed} \
             relation_residue={relation_residue} active_successor={active_successor}",
            successor_owner == owner_baseline
        );
    }
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} \
         scenario=epoch_turnover_retire_reclaim pass=true state=Retiring \
         reason=epoch_superseded owner_identity=true reclaimed=true \
         relation_residue=0 successor_epoch={successor_epoch}"
    ));
    build_and_attest_traversal_replica(coordinator, scale, lines).await?;
    Ok(())
}

async fn run_task199_replica_lifecycle_drills(
    coordinator: &tokio_postgres::Client,
    coordinator_port: u16,
    scale: &str,
    corpus: &str,
    queries: &str,
    content_digest: &str,
    owner_baseline: &str,
    training_query_path: Option<&Path>,
    enospc_fixture: Option<&Task199EnospcFixture>,
) -> Result<Vec<String>> {
    let replica = task198_replica_semantic_result(coordinator, corpus, queries, -1, 0, 20).await?;
    if replica != owner_baseline {
        bail!("Task 199 Ready replica changed ordered semantic results");
    }
    let ordered_result_digest = hex::encode(Sha256::digest(owner_baseline.as_bytes()));
    let mut lines = vec![format!(
        "physical_benchmark_traversal_replica_fault scale={scale} scenario=normal_ready_semantic_identity pass=true content_digest={content_digest} ordered_result_digest={ordered_result_digest}"
    )];

    for isolation in ["READ UNCOMMITTED", "REPEATABLE READ", "SERIALIZABLE"] {
        coordinator
            .batch_execute(&format!("BEGIN ISOLATION LEVEL {isolation} READ ONLY"))
            .await?;
        let isolated =
            task198_replica_semantic_result(coordinator, corpus, queries, -1, 0, 20).await;
        let _ = coordinator.batch_execute("ROLLBACK").await;
        if isolated? != owner_baseline {
            bail!("Task 199 {isolation} owner fallback changed ordered results");
        }
    }
    let isolation_pass = coordinator
        .query_one(
            "SELECT count(*) = 1
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Ready'",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    if !isolation_pass {
        bail!("Task 199 stronger-isolation owner fallback demoted the Ready replica");
    }
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} scenario=isolation_read_semantics pass=true read_uncommitted=true repeatable_read_owner_fallback=true serializable_owner_fallback=true read_only=true ordered_identity=true state=Ready"
    ));

    coordinator
        .batch_execute(
            "CREATE TEMP TABLE IF NOT EXISTS task199_isolation_xid_probe (
                 marker integer NOT NULL
             ) ON COMMIT PRESERVE ROWS",
        )
        .await?;
    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the pre-isolation replica");
    }
    for isolation in ["REPEATABLE READ", "SERIALIZABLE"] {
        let inserted_id = coordinator
            .query_one(
                &format!("SELECT coalesce(max(id), 0) + 1 FROM {corpus}"),
                &[],
            )
            .await?
            .get::<_, i64>(0);
        coordinator
            .batch_execute(&format!("BEGIN ISOLATION LEVEL {isolation}"))
            .await?;
        coordinator
            .execute(
                "INSERT INTO task199_isolation_xid_probe (marker) VALUES (1)",
                &[],
            )
            .await?;
        let (builder, builder_connection) = task199_connect(coordinator_port).await?;
        build_and_attest_traversal_replica(&builder, scale, &mut lines).await?;
        builder_connection.abort();
        let mutation_error = coordinator
            .query_one(&task199_real_insert_sql(corpus), &[])
            .await
            .expect_err(
                "stale-snapshot stronger-isolation mutation must invalidate the Ready replica",
            );
        let fenced = mutation_error
            .code()
            .is_some_and(|code| code.code() == "40001")
            && mutation_error
                .as_db_error()
                .is_some_and(|error| error.message().contains("EC_REPLICA_INVALIDATED"));
        let _ = coordinator.batch_execute("ROLLBACK").await;
        let posture = coordinator
            .query_one(
                &format!(
                    "SELECT
                         (SELECT count(*) = 1
                            FROM ec_distann_traversal_replica_status(
                                'dm_idx'::regclass
                            )
                           WHERE state = 'Stale')
                         AND NOT EXISTS (
                             SELECT 1 FROM {corpus} WHERE id = $1
                         )"
                ),
                &[&inserted_id],
            )
            .await?
            .get::<_, bool>(0);
        if !fenced || !posture {
            bail!("Task 199 {isolation} mutation did not fence against Ready");
        }
        let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
        if !retired || !reclaimed {
            bail!("Task 199 could not reclaim the {isolation} invalidation replica");
        }
    }
    build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?;
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} scenario=stronger_isolation_mutation_fenced pass=true repeatable_read=true serializable=true stale_snapshot=true xid_assigned_before_ready=true ready_committed_after_snapshot=true sqlstate=40001 token=EC_REPLICA_INVALIDATED rebuilt_between_cases=true inserted_rows=0"
    ));

    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} {}",
        task199_crash_after_control_commit_drill(
            coordinator,
            coordinator_port,
            corpus,
            queries,
            owner_baseline,
        )
        .await?
    ));
    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the post-control-commit crash replica");
    }
    build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?;

    let has_fault_hooks = coordinator
        .query_one(
            "SELECT to_regprocedure('ec_distann_stage_scoring_reset()') IS NOT NULL
                    AND current_setting(
                        'ec_distann.benchmark_traversal_replica_fail_batch', true
                    ) IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    let replacement_digest = if has_fault_hooks {
        let mut exercised_offset = None;
        let mut fallback_count = 0;
        for query_offset in 0..32 {
            let expected =
                task198_replica_semantic_result(coordinator, corpus, queries, -1, query_offset, 64)
                    .await?;
            let restarted =
                task198_replica_semantic_result(coordinator, corpus, queries, 1, query_offset, 64)
                    .await?;
            if restarted != expected {
                bail!(
                "Task 199 mid-replica failure did not fully restart to identical owner results \
                 for query offset {query_offset}"
            );
            }
            let row = coordinator
                .query_one(
                    "SELECT value
                   FROM ec_distann_materialization_work_snapshot()
                  WHERE metric = 'replica_fallbacks'",
                    &[],
                )
                .await
                .wrap_err("reading replica_fallbacks after the injected mid-scan failure")?;
            fallback_count = row
                .try_get::<_, i64>(0)
                .wrap_err("decoding replica_fallbacks after the injected mid-scan failure")?;
            if fallback_count == 1 {
                exercised_offset = Some(query_offset);
                break;
            }
            if fallback_count > 1 {
                bail!("Task 199 mid-scan drill recorded more than one fallback");
            }
        }
        let exercised_offset = exercised_offset.ok_or_else(|| {
            color_eyre::eyre::eyre!("Task 199 could not exercise a second replica expansion batch")
        })?;
        let midscan_status = coordinator
            .query_one(
                "SELECT state, state_reason, last_error
                   FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
                  ORDER BY build_started_at DESC LIMIT 1",
                &[],
            )
            .await?;
        if midscan_status.get::<_, String>(0) != "Stale"
            || !midscan_status
                .get::<_, String>(1)
                .contains("replica traversal failed")
            || !midscan_status
                .get::<_, String>(2)
                .contains("injected traversal replica failure")
        {
            bail!("Task 199 mid-scan failure was not durably diagnosed and demoted");
        }
        lines.push(format!(
            "physical_benchmark_traversal_replica_fault scale={scale} scenario=mid_scan_full_restart pass=true fallback_count={fallback_count} query_offset={exercised_offset} state=Stale diagnosed=true"
        ));

        let (retired_after_fault, reclaimed_after_fault) =
            retire_and_reclaim_traversal_replica(coordinator).await?;
        if !retired_after_fault || !reclaimed_after_fault {
            bail!("Task 199 could not reclaim the mid-scan failed replica");
        }
        build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?
    } else {
        lines.push(format!(
            "physical_benchmark_traversal_replica_feature_isolation scale={scale} normal_release=true fault_hooks_absent=true selector_absent=true"
        ));
        content_digest.to_owned()
    };

    let replica_relation = coordinator
        .query_one(
            "SELECT format('%I.%I', namespace.nspname, relation.relname)
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass) status
               JOIN pg_class relation ON relation.oid = status.replica_relid
               JOIN pg_namespace namespace ON namespace.oid = relation.relnamespace
              WHERE status.state = 'Ready'
              ORDER BY status.ready_at DESC LIMIT 1",
            &[],
        )
        .await?
        .get::<_, String>(0);
    coordinator
        .batch_execute(&format!("TRUNCATE TABLE {replica_relation}"))
        .await
        .wrap_err("truncating Task 199 replica for corruption fallback drill")?;
    let corrupt_fallback =
        task198_replica_semantic_result(coordinator, corpus, queries, -1, 0, 20).await?;
    if corrupt_fallback != owner_baseline {
        bail!("Task 199 corrupt replica did not fall back to identical owner results");
    }
    let corrupt_status = coordinator
        .query_one(
            "SELECT state, state_reason, last_error
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              ORDER BY build_started_at DESC LIMIT 1",
            &[],
        )
        .await?;
    if corrupt_status.get::<_, String>(0) != "Stale"
        || !corrupt_status
            .get::<_, String>(2)
            .contains("replica traversal failed")
    {
        bail!("Task 199 corrupt Ready image was not diagnosed and demoted");
    }
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} scenario=corrupt_partial_image_fallback pass=true state=Stale diagnosed=true replacement_digest={replacement_digest}"
    ));

    let (retired_after_corruption, reclaimed_after_corruption) =
        retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired_after_corruption || !reclaimed_after_corruption {
        bail!("Task 199 could not reclaim the corrupt replica");
    }
    build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?;

    let inserted_id = coordinator
        .query_one(
            &format!("SELECT coalesce(max(id), 0) + 1 FROM {corpus}"),
            &[],
        )
        .await?
        .get::<_, i64>(0);
    let insert_sql = task199_real_insert_sql(corpus);
    let first_error = coordinator
        .query_one(&insert_sql, &[])
        .await
        .expect_err("real INSERT must fail the first attempt while replica is Ready");
    let first_retryable = first_error
        .code()
        .is_some_and(|code| code.code() == "40001")
        && first_error
            .as_db_error()
            .is_some_and(|error| error.message().contains("EC_REPLICA_INVALIDATED"));
    let state_after_error = coordinator
        .query_one(
            "SELECT state
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              ORDER BY build_started_at DESC LIMIT 1",
            &[],
        )
        .await?
        .get::<_, String>(0);
    // Task 167 still owns distributed incremental insert. Task 199 must prove
    // that its one-time invalidation composes with the existing explicit
    // fail-closed posture: the retry reaches ordinary distributed-control DML
    // handling, does not return a second 40001, and does not mutate source or
    // owner state.
    let retry_error = coordinator.query_one(&insert_sql, &[]).await.expect_err(
        "distributed-control INSERT retry must retain the Task 167 fail-closed posture",
    );
    let retry_failed_closed = retry_error.as_db_error().is_some_and(|error| {
        error
            .message()
            .contains("EC_GENERATION_MISSING: ec_distann distributed-control inserts")
    });
    let inserted_count = coordinator
        .query_one(
            &format!("SELECT count(*)::bigint FROM {corpus} WHERE id = $1"),
            &[&inserted_id],
        )
        .await?
        .get::<_, i64>(0);
    let rebuild_error = coordinator
        .query_one(
            "SELECT ec_distann_build_traversal_replica('dm_idx'::regclass)",
            &[],
        )
        .await
        .expect_err("same-generation Stale rebuild must name the recovery sequence");
    let rebuild_guidance = rebuild_error.as_db_error().is_some_and(|error| {
        error
            .message()
            .contains("ec_distann_retire_traversal_replica")
            && error
                .message()
                .contains("ec_distann_reclaim_traversal_replica")
    });
    let mutation_pass = first_retryable
        && state_after_error == "Stale"
        && retry_failed_closed
        && inserted_count == 0
        && rebuild_guidance;
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} scenario=real_insert_durable_invalidation pass={mutation_pass} first_sqlstate={} token=EC_REPLICA_INVALIDATED state_after_error={state_after_error} retry_token=EC_GENERATION_MISSING retry_inserted_rows={inserted_count} rebuild_guidance={rebuild_guidance}",
        first_error.code().map(|code| code.code()).unwrap_or("none"),
    ));
    if !mutation_pass {
        bail!("Task 199 real INSERT invalidation drill failed");
    }

    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    let replay = coordinator
        .query_one(
            "SELECT ec_distann_reclaim_traversal_replica('dm_idx'::regclass)",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    let residue = coordinator
        .query_one(
            "SELECT count(*)::bigint
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)",
            &[],
        )
        .await?
        .get::<_, i64>(0);
    let removed_fallback =
        task198_replica_semantic_result(coordinator, corpus, queries, -1, 0, 20).await?;
    let reclaim_pass =
        retired && reclaimed && !replay && residue == 0 && removed_fallback == owner_baseline;
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} scenario=retire_reclaim_and_removed_fallback pass={reclaim_pass} retired={retired} reclaimed={reclaimed} replay={replay} catalog_residue={residue} owner_fallback_identity={}",
        removed_fallback == owner_baseline,
    ));
    if !reclaim_pass {
        bail!("Task 199 retirement/reclaim drill failed");
    }

    let hot_updated = coordinator
        .execute(
            &format!(
                "UPDATE {corpus}
                    SET source = source
                  WHERE id = (SELECT max(id) FROM {corpus})"
            ),
            &[],
        )
        .await?;
    if hot_updated != 1 {
        bail!("Task 199 could not create one pre-build dead heap tuple for VACUUM");
    }
    build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?;
    coordinator
        .batch_execute(&format!("VACUUM (INDEX_CLEANUP ON) {corpus}"))
        .await
        .wrap_err("vacuuming the Task 199 source with a Ready replica")?;
    let vacuum_status = coordinator
        .query_one(
            "SELECT state, state_reason
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              ORDER BY build_started_at DESC LIMIT 1",
            &[],
        )
        .await?;
    let vacuum_state = vacuum_status.get::<_, String>(0);
    let vacuum_reason = vacuum_status.get::<_, String>(1);
    if vacuum_state != "Stale" || vacuum_reason != "vacuum" {
        bail!("Task 199 VACUUM disposition failed: state={vacuum_state} reason={vacuum_reason}");
    }
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} scenario=vacuum_demotes_and_continues pass=true hot_updated_rows=1 state=Stale reason=vacuum"
    ));
    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the VACUUM-demoted replica");
    }

    build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?;
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} {}",
        task199_real_delete_invalidation_drill(coordinator, corpus).await?
    ));
    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the real DELETE invalidation replica");
    }

    build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?;
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} {}",
        task199_participant_tombstone_invalidation_drill(coordinator).await?
    ));
    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the participant tombstone invalidation replica");
    }

    let (concurrent_digest, concurrent_line) =
        task199_concurrent_build_mutation_drill(coordinator, coordinator_port, corpus).await?;
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} {concurrent_line} content_digest={concurrent_digest}"
    ));
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} {}",
        task199_inflight_scan_invalidation_drill(
            coordinator,
            coordinator_port,
            corpus,
            queries,
            owner_baseline,
        )
        .await?
    ));
    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the in-flight invalidation replica");
    }

    build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?;
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} {}",
        task199_auth_failure_recovery_drill(
            coordinator,
            coordinator_port,
            corpus,
            queries,
            owner_baseline,
        )
        .await?
    ));
    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the authentication-recovery replica");
    }

    build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?;
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} {}",
        task199_relation_lock_fallback_drill(
            coordinator,
            coordinator_port,
            corpus,
            queries,
            owner_baseline,
        )
        .await?
    ));
    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the relation-lock fallback replica");
    }

    build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?;
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} {}",
        task199_queued_ddl_lock_drill(coordinator, coordinator_port).await?
    ));
    let (retired, reclaimed) = retire_and_reclaim_traversal_replica(coordinator).await?;
    if !retired || !reclaimed {
        bail!("Task 199 could not reclaim the queued-DDL replica");
    }

    build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?;
    task199_epoch_turnover_drill(
        coordinator,
        scale,
        corpus,
        queries,
        owner_baseline,
        training_query_path,
        &mut lines,
    )
    .await?;
    let persisted_before =
        task198_replica_semantic_result(coordinator, corpus, queries, -1, 0, 20).await?;
    let (reconnected, reconnected_task) = task199_connect(coordinator_port).await?;
    let persisted_after =
        task198_replica_semantic_result(&reconnected, corpus, queries, -1, 0, 20).await?;
    let ready_after_reconnect = reconnected
        .query_one(
            "SELECT count(*) = 1
               FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
              WHERE state = 'Ready'",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    reconnected_task.abort();
    if persisted_before != persisted_after || !ready_after_reconnect {
        bail!("Task 199 replica did not survive a fresh coordinator backend");
    }
    lines.push(format!(
        "physical_benchmark_traversal_replica_fault scale={scale} \
         scenario=coordinator_backend_reconnect pass=true ordered_identity=true state=Ready"
    ));
    if let Some(fixture) = enospc_fixture {
        task199_enospc_replica_build_drill(
            coordinator,
            scale,
            corpus,
            queries,
            owner_baseline,
            fixture,
            &mut lines,
        )
        .await?;
    }
    Ok(lines)
}

async fn compare_materialization_scenario(
    coordinator: &tokio_postgres::Client,
    control: &BenchmarkSeedVariant,
    candidate: &BenchmarkSeedVariant,
    scale: &str,
    scenario: &str,
    sql: &str,
    require_null: bool,
    require_toast: bool,
    expected_rows: usize,
    qualified: bool,
    has_attribution_hooks: bool,
) -> Result<String> {
    let control_json =
        materialization_result_json(coordinator, control, sql, has_attribution_hooks).await?;
    let candidate_json =
        materialization_result_json(coordinator, candidate, sql, has_attribution_hooks).await?;
    let eager_value: serde_json::Value = serde_json::from_str(&control_json)?;
    let candidate_value: serde_json::Value = serde_json::from_str(&candidate_json)?;
    let rows = eager_value
        .as_array()
        .ok_or_else(|| color_eyre::eyre::eyre!("materialization result is not a JSON array"))?
        .len();
    let null_ok = !require_null
        || eager_value
            .as_array()
            .is_some_and(|values| values.iter().all(|value| value["payload_null"] == true));
    let external_toast_ok = !require_toast
        || eager_value.as_array().is_some_and(|values| {
            values.iter().all(|value| {
                value["payload_octets"]
                    .as_u64()
                    .is_some_and(|octets| octets >= 12_800)
                    && value["payload_compression"].is_null()
                    && value["payload_storage"] == "e"
            })
        });
    let mut remote_requested = 0_i64;
    let mut duplicate_requested = 0_i64;
    let mut local_consumed = 0_i64;
    if has_attribution_hooks {
        let work = coordinator
            .query(
                "SELECT metric, value FROM ec_distann_materialization_work_snapshot()
                  WHERE metric IN ('remote_candidates_requested', 'duplicate_remote_candidates_requested', 'executor_local_rows_consumed')",
                &[],
            )
            .await?;
        for row in work {
            match row.get::<_, String>(0).as_str() {
                "remote_candidates_requested" => remote_requested = row.get(1),
                "duplicate_remote_candidates_requested" => duplicate_requested = row.get(1),
                "executor_local_rows_consumed" => local_consumed = row.get(1),
                _ => {}
            }
        }
    }
    let configured_top_k = coordinator
        .query_one("SELECT current_setting('ec_distann.top_k')::bigint", &[])
        .await?
        .get::<_, i64>(0);
    let initial_bar = configured_top_k.max(expected_rows as i64);
    let deepening_cap = initial_bar.saturating_mul(64).max(1024);
    let payload_read_bound = if qualified {
        deepening_cap
    } else {
        ((expected_rows as i64 + 9) / 10) * 10
    };
    let payload_reads = remote_requested.saturating_add(local_consumed);
    let attribution_pass =
        !has_attribution_hooks || (duplicate_requested == 0 && payload_reads <= payload_read_bound);
    let pass = eager_value == candidate_value
        && rows == expected_rows
        && null_ok
        && external_toast_ok
        && attribution_pass;
    if !pass {
        bail!(
            "materialization correctness scenario {scenario} failed: rows={rows}/{expected_rows} identity={} null_ok={null_ok} external_toast_ok={external_toast_ok} remote_requested={remote_requested} local_consumed={local_consumed} payload_reads={payload_reads}/{payload_read_bound} duplicate_requested={duplicate_requested}",
            eager_value == candidate_value
        );
    }
    Ok(format!(
        "physical_materialization_correctness scale={scale} scenario={scenario} pass=true rows={rows} eager_digest={} candidate_digest={} null_ok={null_ok} external_toast_ok={external_toast_ok} attribution_available={has_attribution_hooks} remote_requested={remote_requested} local_consumed={local_consumed} payload_reads={payload_reads} payload_read_bound={payload_read_bound} deepening_cap={deepening_cap} duplicate_requested={duplicate_requested}",
        hex::encode(Sha256::digest(control_json.as_bytes())),
        hex::encode(Sha256::digest(candidate_json.as_bytes())),
    ))
}

async fn run_materialization_correctness(
    coordinator: &tokio_postgres::Client,
    pg_ctl: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    seed_variants: &[BenchmarkSeedVariant],
    scale: &str,
    corpus: &str,
    queries: &str,
) -> Result<Vec<String>> {
    if nodes.len() < 2 {
        bail!("materialization correctness requires at least two physical owners");
    }
    let has_attribution_hooks = coordinator
        .query_one(
            "SELECT to_regprocedure('ec_distann_stage_scoring_reset()') IS NOT NULL
                    AND to_regprocedure(
                        'ec_distann_materialization_work_snapshot()'
                    ) IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    let same_search = |left: &BenchmarkSeedVariant, right: &BenchmarkSeedVariant| {
        left.strategy == right.strategy
            && left.head_search_width == right.head_search_width
            && left.head_seed_count == right.head_seed_count
            && left.neighbor_score_mode == right.neighbor_score_mode
            && left.beam_width == right.beam_width
            && left.hop_rounds == right.hop_rounds
    };
    let plan_pair = seed_variants
        .iter()
        .filter(|variant| !variant.owner_payload_plan_cache)
        .find_map(|control| {
            seed_variants
                .iter()
                .find(|candidate| {
                    candidate.owner_payload_plan_cache
                        && candidate.materialization_batch_size
                            == control.materialization_batch_size
                        && candidate.traversal_replica == control.traversal_replica
                        && same_search(control, candidate)
                })
                .map(|candidate| (control, candidate))
        });
    let batch_pair = seed_variants
        .iter()
        .filter(|variant| variant.materialization_batch_size == 0)
        .find_map(|control| {
            seed_variants
                .iter()
                .find(|candidate| {
                    candidate.materialization_batch_size == 10
                        && candidate.owner_payload_plan_cache == control.owner_payload_plan_cache
                        && candidate.traversal_replica == control.traversal_replica
                        && same_search(control, candidate)
                })
                .map(|candidate| (control, candidate))
        });
    let traversal_pair = seed_variants
        .iter()
        .filter(|variant| !variant.traversal_replica)
        .find_map(|control| {
            seed_variants
                .iter()
                .find(|candidate| {
                    candidate.traversal_replica
                        && candidate.materialization_batch_size
                            == control.materialization_batch_size
                        && candidate.owner_payload_plan_cache == control.owner_payload_plan_cache
                        && same_search(control, candidate)
                })
                .map(|candidate| (control, candidate))
        });
    let packed_pair = seed_variants
        .iter()
        .filter(|variant| !variant.packed_payload)
        .find_map(|control| {
            seed_variants
                .iter()
                .find(|candidate| {
                    candidate.packed_payload
                        && candidate.materialization_batch_size
                            == control.materialization_batch_size
                        && candidate.owner_payload_plan_cache == control.owner_payload_plan_cache
                        && candidate.traversal_replica == control.traversal_replica
                        && candidate.typed_locator == control.typed_locator
                        && same_search(control, candidate)
                })
                .map(|candidate| (control, candidate))
        });
    let expanded_pair = seed_variants
        .iter()
        .filter(|variant| !variant.expanded_locator)
        .find_map(|control| {
            seed_variants
                .iter()
                .find(|candidate| {
                    candidate.expanded_locator
                        && candidate.materialization_batch_size
                            == control.materialization_batch_size
                        && candidate.owner_payload_plan_cache == control.owner_payload_plan_cache
                        && candidate.typed_locator == control.typed_locator
                        && candidate.packed_payload == control.packed_payload
                        && candidate.traversal_replica == control.traversal_replica
                        && same_search(control, candidate)
                })
                .map(|candidate| (control, candidate))
        });
    let (control, candidate) = plan_pair
        .or(batch_pair)
        .or(traversal_pair)
        .or(packed_pair)
        .or(expanded_pair)
        .ok_or_else(|| {
            color_eyre::eyre::eyre!(
                "materialization correctness requires an isolated owner-plan, eager/lazy10, owner/replica, or packed-payload pair"
            )
        })?;

    coordinator
        .batch_execute(&format!(
            "SET ec_distann.benchmark_seed_mode = '{}';
             SET ec_distann.benchmark_head_search_width = {};
             SET ec_distann.benchmark_head_seed_count = {};
             SET ec_distann.benchmark_exact_neighbor = {};",
            control.strategy.replace('\'', "''"),
            control.head_search_width,
            control.head_seed_count,
            if control.neighbor_score_mode == "exact_neighbor" {
                "on"
            } else {
                "off"
            },
        ))
        .await?;

    coordinator
        .batch_execute(&materialization_variant_settings_sql(control))
        .await?;
    let ranked_ids = coordinator
        .query(
            &format!(
                "SELECT id FROM {corpus}
                  ORDER BY embedding <#> (SELECT source FROM {queries} ORDER BY id LIMIT 1)
                  LIMIT 64"
            ),
            &[],
        )
        .await?;
    if ranked_ids.len() < 50 {
        bail!(
            "materialization correctness expected at least 50 ranked IDs, got {}",
            ranked_ids.len()
        );
    }
    let ranked_ids = ranked_ids
        .iter()
        .map(|row| row.get::<_, i64>(0).to_string())
        .collect::<Vec<_>>();
    let exclude_first = ranked_ids[..10].join(",");
    let exclude_multiple = ranked_ids[..40].join(",");

    let mut lines = Vec::new();
    for (scenario, predicate, limit, require_null, require_toast, qualified) in [
        (
            "fewer_than_window",
            "TRUE".to_owned(),
            5,
            false,
            false,
            false,
        ),
        (
            "exactly_one_window",
            "TRUE".to_owned(),
            10,
            false,
            false,
            false,
        ),
        (
            "more_than_window",
            "TRUE".to_owned(),
            15,
            false,
            false,
            false,
        ),
        (
            "reject_first_window",
            format!("id NOT IN ({exclude_first})"),
            10,
            false,
            false,
            true,
        ),
        (
            "reject_multiple_windows",
            format!("id NOT IN ({exclude_multiple})"),
            10,
            false,
            false,
            true,
        ),
        (
            "null_payload",
            "payload_note IS NULL".to_owned(),
            10,
            true,
            false,
            true,
        ),
        (
            "toasted_projection_qual",
            "payload_note IS NOT NULL AND id % 3 = 1".to_owned(),
            10,
            false,
            true,
            true,
        ),
    ] {
        let sql = materialization_semantic_sql(corpus, queries, &predicate, limit, 0);
        lines.push(
            compare_materialization_scenario(
                coordinator,
                control,
                candidate,
                scale,
                scenario,
                &sql,
                require_null,
                require_toast,
                limit as usize,
                qualified,
                has_attribution_hooks,
            )
            .await?,
        );
    }

    if !has_attribution_hooks {
        lines.push(format!(
            "physical_materialization_feature_isolation scale={scale} normal_release=true attribution_hooks_absent=true semantic_scenarios=7"
        ));
        return Ok(lines);
    }

    let mut mixed = None;
    for query_offset in 0..10 {
        coordinator
            .batch_execute(&format!(
                "SELECT ec_distann_stage_scoring_reset(); {}",
                materialization_variant_settings_sql(candidate),
            ))
            .await?;
        let mixed_sql = materialization_semantic_sql(corpus, queries, "TRUE", 10, query_offset);
        let _ = coordinator.query_one(&mixed_sql, &[]).await?;
        let work = coordinator
            .query(
                "SELECT metric, value FROM ec_distann_materialization_work_snapshot()
                  WHERE metric IN ('executor_remote_rows_consumed', 'executor_local_rows_consumed',
                                   'duplicate_remote_candidates_requested')",
                &[],
            )
            .await?;
        let mut remote = 0_i64;
        let mut local = 0_i64;
        let mut duplicate_requested = 0_i64;
        for row in work {
            match row.get::<_, String>(0).as_str() {
                "executor_remote_rows_consumed" => remote = row.get(1),
                "executor_local_rows_consumed" => local = row.get(1),
                "duplicate_remote_candidates_requested" => duplicate_requested = row.get(1),
                _ => {}
            }
        }
        if remote > 0 && local > 0 && remote + local == 10 && duplicate_requested == 0 {
            mixed = Some((query_offset, remote, local, duplicate_requested));
            break;
        }
    }
    let (mixed_query_offset, remote, local, duplicate_requested) = mixed.ok_or_else(|| {
        color_eyre::eyre::eyre!("no mixed local/remote top-10 in first 10 queries")
    })?;
    lines.push(format!(
        "physical_materialization_correctness scale={scale} scenario=mixed_local_remote pass=true rows=10 query_offset={mixed_query_offset} remote_consumed={remote} local_consumed={local} duplicate_requested={duplicate_requested}"
    ));

    coordinator
        .batch_execute(&format!(
            "SELECT ec_distann_stage_scoring_reset();
             {}
             BEGIN;
             DECLARE task184_materialization_cursor NO SCROLL CURSOR FOR
             SELECT id, source_id, source, payload_note
               FROM {corpus}
              ORDER BY embedding <#> (SELECT source FROM {queries} ORDER BY id OFFSET {mixed_query_offset} LIMIT 1)
              LIMIT 40;",
            materialization_variant_settings_sql(candidate),
        ))
        .await?;
    let first_rows = coordinator
        .query("FETCH FORWARD 10 FROM task184_materialization_cursor", &[])
        .await?;
    let requested = coordinator
        .query_one(
            "SELECT value FROM ec_distann_materialization_work_snapshot()
              WHERE metric = 'remote_candidates_requested'",
            &[],
        )
        .await?
        .get::<_, i64>(0);
    if first_rows.len() != 10 || requested == 0 {
        coordinator.batch_execute("ROLLBACK").await?;
        bail!(
            "post-first-batch failure drill did not complete a remote first batch: rows={} requested={requested}",
            first_rows.len()
        );
    }

    let mut stopped = Vec::new();
    for node in nodes.iter().skip(1) {
        let mut stop = Command::new(pg_ctl);
        stop.arg("-w")
            .arg("-D")
            .arg(&node.data_dir)
            .arg("-m")
            .arg("immediate")
            .arg("stop")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        run_status(stop)
            .await
            .wrap_err_with(|| format!("stopping remote owner {} for Task 184", node.node_id))?;
        stopped.push(node);
    }
    let later_error = coordinator
        .query("FETCH FORWARD ALL FROM task184_materialization_cursor", &[])
        .await
        .err();
    let _ = coordinator.batch_execute("ROLLBACK").await;
    for node in stopped {
        restart_physical_node(pg_ctl, socket_dir, node, nodes).await?;
    }
    let duplicate_requested = coordinator
        .query_one(
            "SELECT value FROM ec_distann_materialization_work_snapshot()
              WHERE metric = 'duplicate_remote_candidates_requested'",
            &[],
        )
        .await?
        .get::<_, i64>(0);
    if duplicate_requested != 0 {
        bail!(
            "post-first-batch remote-owner outage re-requested {duplicate_requested} remote payloads"
        );
    }
    let Some(later_error) = later_error else {
        bail!("post-first-batch remote-owner outage returned a complete prefix without error");
    };
    lines.push(format!(
        "physical_materialization_correctness scale={scale} scenario=post_first_batch_remote_failure pass=true first_rows={} first_remote_requested={requested} duplicate_requested={duplicate_requested} error_digest={}",
        first_rows.len(),
        hex::encode(Sha256::digest(later_error.to_string().as_bytes())),
    ));
    Ok(lines)
}

async fn run_coverage_memory_regression(
    coordinator: &tokio_postgres::Client,
    physical_queries: &str,
    scale: &str,
    iterations: u32,
    max_slope_kb_per_s: f64,
    max_delta_kb: f64,
    sample_interval_ms: u64,
    log_dir: &Path,
) -> Result<String> {
    const WARMUP_INVOCATIONS: u32 = 5;
    const SETTLE_INVOCATIONS: u32 = 1;
    const WARMUP_SETTLE_MS: u64 = 1_000;
    let pid = coordinator
        .query_one("SELECT pg_backend_pid()", &[])
        .await?
        .get::<_, i32>(0);
    let stop = Arc::new(AtomicBool::new(false));
    let series = Arc::new(Mutex::new(Vec::new()));

    let coverage_sql = |call_count: u32| {
        format!(
            "SELECT count(*)::bigint
           FROM generate_series(1, {call_count}) AS calls(repeat_no)
          CROSS JOIN LATERAL (
              SELECT count(*)::bigint AS query_count
                FROM {physical_queries}
          ) query_cardinality
          CROSS JOIN LATERAL (
              SELECT source
                FROM {physical_queries}
               ORDER BY id
               LIMIT 1
              OFFSET ((calls.repeat_no - 1) % query_cardinality.query_count)
          ) query_row
          CROSS JOIN LATERAL ec_distann_physical_seed_coverage_benchmark(
              'dm_idx'::regclass, query_row.source, 32, 32) coverage"
        )
    };

    let result = async {
        coordinator.batch_execute("BEGIN").await?;
        // The first few coverage calls acquire the bounded working set needed
        // by the scan. Discard samples collected during them so the regression
        // statistic covers the stable post-warm-up segment rather than cold
        // page/cache acquisition.
        coordinator
            .query_one(&coverage_sql(WARMUP_INVOCATIONS), &[])
            .await?;
        let peak = Arc::new(Mutex::new(
            crate::commands::bench::latency::MemorySample::default(),
        ));
        let monitor = tokio::spawn(monitor_backend_memory(
            pid,
            sample_interval_ms,
            Arc::clone(&stop),
            peak,
            Arc::clone(&series),
            "coverage-regression".to_owned(),
            None,
        ));
        coordinator
            .query_one(&coverage_sql(SETTLE_INVOCATIONS), &[])
            .await?;
        tokio::time::sleep(Duration::from_millis(WARMUP_SETTLE_MS)).await;
        series.lock().await.clear();
        let rows = match coordinator.query_one(&coverage_sql(iterations), &[]).await {
            Ok(row) => row.get::<_, i64>(0),
            Err(error) => {
                stop.store(true, Ordering::Relaxed);
                let _ = monitor.await;
                let _ = coordinator.batch_execute("ROLLBACK").await;
                return Err(error.into());
            }
        };
        // Stop sampling before COMMIT resets TopTransactionContext; the
        // statistic must describe the held-transaction query window only.
        stop.store(true, Ordering::Relaxed);
        monitor
            .await
            .map_err(|error| color_eyre::eyre::eyre!("RSS monitor task failed: {error}"))??;
        coordinator.batch_execute("COMMIT").await?;
        Ok::<i64, color_eyre::Report>(rows)
    }
    .await;

    let points = series.lock().await.clone();
    let rows = result.wrap_err("running Task 200 coverage memory regression")?;
    if rows < i64::from(iterations) {
        bail!(
            "Task 200 coverage memory regression executed {rows} rows, expected at least {iterations}"
        );
    }
    let trim_samples = usize::try_from((1_000 / sample_interval_ms.max(1)).max(1)).unwrap_or(1);
    if points.len() <= trim_samples.saturating_mul(2).saturating_add(1) {
        bail!(
            "Task 200 coverage memory regression collected too few samples ({}) after edge trimming",
            points.len()
        );
    }
    // The first and last sampler ticks can race working-set reacquisition and
    // query completion respectively. Trim one settle-second at each edge;
    // the interior remains the held-transaction measurement window.
    let stable_points = &points[trim_samples..points.len() - trim_samples];
    let slope = rss_slope_kb_per_second(stable_points).ok_or_else(|| {
        color_eyre::eyre::eyre!(
            "Task 200 coverage memory regression collected fewer than two RSS samples"
        )
    })?;
    let first = stable_points
        .first()
        .map(|point| point.rss_kb)
        .unwrap_or_default();
    let last = stable_points
        .last()
        .map(|point| point.rss_kb)
        .unwrap_or_default();
    let mut rss_values = stable_points
        .iter()
        .map(|point| point.rss_kb)
        .collect::<Vec<_>>();
    rss_values.sort_unstable();
    let percentile_trim = (rss_values.len() / 100).max(1);
    let lower = rss_values[percentile_trim];
    let upper = rss_values[rss_values.len() - percentile_trim - 1];
    let delta = upper.saturating_sub(lower);
    let series_path = log_dir.join("coverage-memory-regression.series.log");
    let series_text = points
        .iter()
        .map(|point| {
            format!(
                "[backend-memory] pid={} elapsed_ms={} rss_kb={} hwm_kb={}",
                point.pid, point.elapsed_ms, point.rss_kb, point.hwm_kb
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&series_path, format!("{series_text}\n"))
        .wrap_err_with(|| format!("writing {}", series_path.display()))?;
    let pass = slope <= max_slope_kb_per_s && (delta as f64) <= max_delta_kb;
    let line = format!(
        "physical_benchmark_memory_regression scale={scale} warmup_invocations={} warmup_settle_ms={WARMUP_SETTLE_MS} trimmed_edge_samples={trim_samples} percentile_trim_percent=1 coverage_invocations={iterations} rows_returned={rows} samples={} stable_samples={} rss_first_kb={first} rss_last_kb={last} rss_p01_kb={lower} rss_p99_kb={upper} rss_p01_to_p99_kb={delta} max_delta_kb={max_delta_kb:.2} rss_slope_kb_per_s={slope:.2} max_slope_kb_per_s={max_slope_kb_per_s:.2} series={} pass={pass}",
        WARMUP_INVOCATIONS + SETTLE_INVOCATIONS,
        points.len(),
        stable_points.len(),
        series_path.display(),
    );
    if !pass {
        bail!("Task 200 coverage memory regression FAILED: {line}");
    }
    Ok(line)
}

fn query_slice_sha256(bytes: &[u8], offset: u32, limit: u32) -> Result<String> {
    let offset = usize::try_from(offset).wrap_err("query offset exceeds usize")?;
    let limit = usize::try_from(limit).wrap_err("query limit exceeds usize")?;
    let mut hasher = Sha256::new();
    let mut selected = 0usize;
    for line in bytes
        .split_inclusive(|byte| *byte == b'\n')
        .skip(offset)
        .take(limit)
    {
        hasher.update(line);
        selected += 1;
    }
    if selected != limit {
        bail!("query slice is short: offset={offset} requested={limit} selected={selected}");
    }
    Ok(hex::encode(hasher.finalize()))
}

async fn run_physical_benchmarks(
    args: &LocalMultinodePg18Args,
    coordinator: &tokio_postgres::Client,
    coordinator_port: u16,
    pg_ctl: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    published: &[PhysicalTopologyRow],
    log_dir: &Path,
    build_ms: u128,
    publish_ms: u128,
    extension_preflight: &ExtensionPreflight,
    enospc_fixture: Option<&Task199EnospcFixture>,
) -> Result<Vec<String>> {
    let beam_width = args.beam_width.unwrap_or(4);
    let candidate_heap_limit = args
        .candidate_heap_limit
        .unwrap_or(32)
        .max(beam_width)
        .max(args.top_k);
    let hop_rounds = args.hop_rounds.unwrap_or(100);
    // Task 210 P2b: replica routing is gated on an attested population, so a
    // replica arm must distribute the shard copies before benchmarking —
    // otherwise every request clamps to its owner and the arm measures
    // nothing (the head_replica_fallbacks=96 outcome of the first P2 run).
    if let Some(replicas) = args.head_replica_count.filter(|count| *count > 0) {
        let placed = coordinator
            .query_one(
                "SELECT ec_distann_populate_head_replicas('dm_idx'::regclass, $1::integer)",
                &[&i32::try_from(replicas).unwrap_or(i32::MAX)],
            )
            .await
            .wrap_err("populating head shard replicas")?
            .get::<_, i64>(0);
        crate::ecaz_eprintln!(
            "[distann-multicluster] physical_head_replicas populated replica_count={replicas} placed={placed}"
        );
    }
    let production_head_width = (beam_width * 2).max(32);
    let seed_variants = if args.benchmark_seed_variants.is_empty() {
        vec![BenchmarkSeedVariant {
            name: "production".to_owned(),
            strategy: args
                .seed_strategy
                .clone()
                .unwrap_or_else(|| "persisted_head".to_owned()),
            head_search_width: args.head_search_width.unwrap_or(production_head_width),
            head_seed_count: args.head_seed_count.unwrap_or(production_head_width),
            neighbor_score_mode: args
                .neighbor_score_mode
                .clone()
                .unwrap_or_else(|| "rabitq".to_owned()),
            materialization_batch_size: 10,
            owner_payload_plan_cache: false,
            beam_width: None,
            hop_rounds: None,
            traversal_replica: false,
            typed_locator: false,
            packed_payload: false,
            expanded_locator: false,
        }]
    } else {
        parse_benchmark_seed_variants(&args.benchmark_seed_variants)?
    };
    let explicit_seed_controls = args.seed_strategy.is_some()
        || args.head_search_width.is_some()
        || args.head_seed_count.is_some()
        || args.neighbor_score_mode.is_some()
        || seed_variants.iter().any(|variant| {
            variant.strategy != "persisted_head"
                || variant.head_search_width != production_head_width
                || variant.head_seed_count != production_head_width
                || variant.neighbor_score_mode != "rabitq"
                || variant.materialization_batch_size != 10
                || variant.owner_payload_plan_cache
        });
    let corpus_prefix = args
        .corpus_prefix
        .as_deref()
        .ok_or_else(|| color_eyre::eyre::eyre!("physical benchmark requires corpus_prefix"))?;
    if !corpus_prefix
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        bail!("physical benchmark corpus prefix is not a SQL identifier");
    }
    let scale = corpus_prefix
        .strip_prefix("ec_real_")
        .unwrap_or(corpus_prefix);
    let expected_sha = &extension_preflight.git_sha;
    let expected_profile = &extension_preflight.build_profile;
    let requested_head_policy = args
        .production_head_policy
        .as_deref()
        .or(args.head_policy.as_deref())
        .unwrap_or("current_sample");
    let production_head_attestation = if let Some(production_policy) =
        args.production_head_policy.as_deref()
    {
        let policy = coordinator
            .query_one(
                "SELECT head_policy, scoring_mode, training_query_count,
                        encode(training_query_digest, 'hex'), head_index_cap,
                        returned_seed_count, sample_count,
                        encode(head_sample_digest, 'hex')
                   FROM ec_distann_active_head_policy('dm_idx'::regclass)",
                &[],
            )
            .await
            .wrap_err("attesting Task 182 active production head policy")?;
        let attested_policy = policy.get::<_, String>(0);
        if attested_policy != production_policy {
            bail!(
                "production head-policy attestation mismatch: requested {production_policy}, got {attested_policy}"
            );
        }
        let construction = coordinator
            .query_one(
                "SELECT head_construction, marker_attested
                   FROM ec_distann_active_head_construction('dm_idx'::regclass)",
                &[],
            )
            .await
            .wrap_err("attesting Task 207 active physical head construction")?;
        Some(format!(
            "physical_benchmark_head_policy scale={scale} policy={} scoring_mode={} head_construction={} head_construction_marker_attested={} training_queries={} training_query_digest={} head_index_cap={} returned_seed_count={} sample_count={} head_sample_digest={}",
            attested_policy,
            policy.get::<_, String>(1),
            construction.get::<_, String>(0),
            construction.get::<_, bool>(1),
            policy.get::<_, i32>(2),
            policy.get::<_, String>(3),
            policy.get::<_, i32>(4),
            policy.get::<_, i32>(5),
            policy.get::<_, i32>(6),
            policy.get::<_, String>(7),
        ))
    } else {
        None
    };
    let has_head_policy_provenance = coordinator
        .query_one(
            "SELECT to_regprocedure('ec_distann_physical_head_policy()') IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    if (args.gateway_trace || args.gateway_isolated_trace || args.gateway_head_candidate_trace)
        && !coordinator
            .query_one(
                "SELECT to_regprocedure('ec_distann_physical_seed_gateway_trace_benchmark(regclass,real[],integer)') IS NOT NULL AND to_regprocedure('ec_distann_physical_seed_isolated_gateway_trace_benchmark(regclass,real[],integer,integer)') IS NOT NULL AND to_regprocedure('ec_distann_physical_head_candidate_trace_benchmark(regclass,real[],integer,integer)') IS NOT NULL",
                &[],
            )
            .await?
            .get::<_, bool>(0)
    {
        bail!(
            "gateway attribution requires an extension built with distann-head-attribution-benchmark"
        );
    }
    if args.query_trace
        && !coordinator
            .query_one(
                "SELECT to_regprocedure('ec_distann_physical_query_trace_benchmark(regclass,real[],integer)') IS NOT NULL",
                &[],
            )
            .await?
            .get::<_, bool>(0)
    {
        bail!(
            "query trace requires an extension built with distann-head-attribution-benchmark"
        );
    }
    if args.head_policy.is_some() && !has_head_policy_provenance {
        bail!("Task 181 head policy requires extension head-policy provenance helper");
    }
    let benchmark_head_policy = if has_head_policy_provenance {
        coordinator
            .query_one("SELECT ec_distann_physical_head_policy()", &[])
            .await?
            .get::<_, String>(0)
    } else {
        "current_sample".to_owned()
    };
    if args.production_head_policy.is_none()
        && !args.reuse_fixture
        && benchmark_head_policy != requested_head_policy
    {
        bail!(
            "physical head-policy attestation mismatch: requested {requested_head_policy}, got {benchmark_head_policy}"
        );
    }
    let attested_head_policy = if args.reuse_fixture {
        benchmark_head_policy.clone()
    } else {
        args.production_head_policy
            .clone()
            .unwrap_or(benchmark_head_policy)
    };
    let physical_prefix = format!("task179_physical_{scale}");
    let single_prefix = format!("task179_single_{scale}");
    let physical_corpus = format!("{physical_prefix}_corpus");
    let physical_queries = format!("{physical_prefix}_queries");
    let single_corpus = format!("{single_prefix}_corpus");
    let single_queries = format!("{single_prefix}_queries");
    let single_index = format!("{single_prefix}_idx");
    if !args.reuse_fixture {
        coordinator
            .batch_execute(&format!(
                "RESET enable_seqscan;
                 ALTER TABLE dm RENAME TO {physical_corpus};
                 ALTER TABLE dm_queries RENAME TO {physical_queries};"
            ))
            .await?;
    } else {
        coordinator.batch_execute("RESET enable_seqscan").await?;
    }

    let single_build_ms = if args.stage_counter_only || args.skip_single_control {
        0
    } else {
        let single_started = Instant::now();
        let head_sizing = head_sizing_reloptions(args);
        coordinator
            .batch_execute(&format!(
                "CREATE TABLE {single_corpus} AS
                     SELECT id, source, embedding FROM {physical_corpus};
                 CREATE TABLE {single_queries} AS SELECT * FROM {physical_queries};
                 CREATE INDEX {single_index} ON {single_corpus}
                     USING ec_distann (embedding ecvector_distann_ip_ops)
                     WITH (graph_degree = {}, head_index_cap = {},
                           neighbor_code_format = 'rabitq'{});",
                args.graph_degree, args.head_index_cap, head_sizing
            ))
            .await?;
        single_started.elapsed().as_millis()
    };
    let has_seed_strategy_provenance = coordinator
        .query_one(
            "SELECT to_regprocedure('ec_distann_physical_seed_strategy()') IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    let has_neighbor_score_provenance = coordinator
        .query_one(
            "SELECT to_regprocedure('ec_distann_physical_neighbor_score_mode()') IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    let has_seed_id_digest = coordinator
        .query_one(
            "SELECT to_regprocedure('ec_distann_physical_seed_id_digest(regclass,real[])') IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    if explicit_seed_controls && (!has_seed_strategy_provenance || !has_neighbor_score_provenance) {
        bail!("Task 180 controls require extension seed and neighbor-score provenance helpers");
    }
    let requires_same_seed_attribution = seed_variants.iter().enumerate().any(|(index, left)| {
        seed_variants.iter().skip(index + 1).any(|right| {
            left.strategy == right.strategy
                && left.head_search_width == right.head_search_width
                && left.head_seed_count == right.head_seed_count
                && left.neighbor_score_mode != right.neighbor_score_mode
        })
    });
    if requires_same_seed_attribution && !has_seed_id_digest {
        bail!("same-seed neighbor-score attribution requires extension seed-ID digest helper");
    }

    let staged_dir = args
        .staged_dir
        .clone()
        .unwrap_or(repo_root()?.join("data/staged-current"));
    let truth_corpus =
        std::fs::canonicalize(staged_dir.join(format!("{corpus_prefix}_corpus.tsv")))?;
    let truth_queries =
        std::fs::canonicalize(staged_dir.join(format!("{corpus_prefix}_queries.tsv")))?;
    let query_bytes = std::fs::read(&truth_queries)?;
    let query_sha256 = hex::encode(Sha256::digest(&query_bytes));
    let query_slice_sha256 = query_slice_sha256(&query_bytes, args.query_offset, args.queries)?;
    let query_start = args.query_offset + 1;
    let query_end = args.query_offset + args.queries;
    let truth_cache = nodes[0]
        .data_dir
        .parent()
        .unwrap_or(nodes[0].data_dir.as_path())
        .join(format!("{corpus_prefix}-truth.json"));
    let common = vec![
        "--database".to_owned(),
        "postgres".to_owned(),
        "--host".to_owned(),
        "127.0.0.1".to_owned(),
        "--port".to_owned(),
        coordinator_port.to_string(),
        "--user".to_owned(),
        "postgres".to_owned(),
    ];
    let mut lines = vec![format!(
        "physical_benchmark_provenance scale={scale} extension_git_sha={expected_sha} extension_build_profile={expected_profile} nodes={} unanimous=true stage_counter_only={} query_offset={} query_rows={} query_slice_sha256={query_slice_sha256}",
        extension_preflight.nodes,
        args.stage_counter_only,
        args.query_offset,
        args.queries,
    )];
    let mut task167_quality_gate_failed = false;
    if let Some(attestation) = production_head_attestation {
        lines.push(attestation);
    }
    let training_provenance = if let Some(path) = args.training_query_path.as_deref() {
        let bytes = std::fs::read(std::fs::canonicalize(path)?)?;
        let contents = String::from_utf8(bytes.clone())?;
        let slice = contents
            .lines()
            .skip(200)
            .take(200)
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "training_prefix=rows_201_400 training_queries=200 training_file_sha256={} training_slice_sha256={}",
            hex::encode(Sha256::digest(bytes)),
            hex::encode(Sha256::digest(slice.as_bytes()))
        )
    } else {
        "training_prefix=none training_queries=0 training_file_sha256=none training_slice_sha256=none".to_owned()
    };
    lines.push(format!(
        "physical_benchmark_landmark scale={scale} head_policy={attested_head_policy} evaluation_prefix=rows_{query_start}_{query_end} evaluation_query_offset={} evaluation_queries={} evaluation_query_sha256={query_sha256} evaluation_slice_sha256={query_slice_sha256} {training_provenance} deterministic=true sample_cap={} construction_ms={build_ms}",
        args.query_offset,
        args.queries,
        args.head_index_cap,
    ));
    let gateway_trace_queries =
        if args.gateway_trace || args.gateway_isolated_trace || args.gateway_head_candidate_trace {
            let training_path = std::fs::canonicalize(
                args.training_query_path
                    .as_deref()
                    .expect("gateway trace validation requires a training query path"),
            )?;
            let training_path = training_path.display().to_string().replace('\'', "''");
            coordinator
                .batch_execute(&format!(
                    "CREATE TEMP TABLE ec_distann_task185_gateway_training_stage (
                     load_ordinal bigserial, source_id bigint, vec text
                 );
                 COPY ec_distann_task185_gateway_training_stage (source_id, vec)
                   FROM '{training_path}' WITH (FORMAT text, DELIMITER E'\\t');
                 CREATE TEMP TABLE ec_distann_task185_gateway_training_queries AS
                 SELECT (load_ordinal - 200)::bigint AS id,
                        translate(vec, '[]', '{{}}')::real[] AS source
                   FROM ec_distann_task185_gateway_training_stage
                  WHERE load_ordinal BETWEEN 201 AND 400
                  ORDER BY load_ordinal;
                 DROP TABLE ec_distann_task185_gateway_training_stage;"
                ))
                .await
                .wrap_err("staging Task 185 disjoint gateway-training queries")?;
            Some("ec_distann_task185_gateway_training_queries")
        } else {
            None
        };
    if let Some(iterations) = args.coverage_memory_regression_iterations {
        lines.push(
            run_coverage_memory_regression(
                coordinator,
                &physical_queries,
                &scale,
                iterations,
                args.coverage_memory_regression_max_slope_kb_per_s,
                args.coverage_memory_regression_max_delta_kb,
                args.memory_sample_interval_ms,
                log_dir,
            )
            .await?,
        );
    }
    if args.head_policy.is_some() && !args.stage_counter_only && !args.skip_recall {
        let coverage = coordinator
            .query_one(
                &format!(
                    "WITH coverage AS (
                         SELECT q.id, c.* FROM {physical_queries} q
                         CROSS JOIN LATERAL ec_distann_physical_seed_coverage_benchmark(
                             'dm_idx'::regclass, q.source, 32, 32) c
                         ORDER BY q.id LIMIT 200
                     ), regions AS (
                         SELECT query_region, count(*) AS queries,
                                count(*) FILTER (WHERE zero_owner_represented) AS zero_queries
                         FROM coverage GROUP BY query_region
                     )
                     SELECT jsonb_build_object(
                         'queries', (SELECT count(*) FROM coverage),
                         'owner_membership_rate', (SELECT sum(owner_in_head)::double precision / NULLIF(sum(owner_seed_count), 0) FROM coverage),
                         'bounded_overlap_rate', (SELECT sum(bounded_owner_overlap)::double precision / NULLIF(sum(owner_seed_count), 0) FROM coverage),
                         'exact_overlap_rate', (SELECT sum(exact_owner_overlap)::double precision / NULLIF(sum(owner_seed_count), 0) FROM coverage),
                         'zero_fraction', (SELECT count(*) FILTER (WHERE zero_owner_represented)::double precision / NULLIF(count(*), 0) FROM coverage),
                         'mean_best_score_gap', (SELECT avg(best_score_gap) FROM coverage),
                         'p50_best_score_gap', (SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY best_score_gap) FROM coverage),
                         'p95_best_score_gap', (SELECT percentile_cont(0.95) WITHIN GROUP (ORDER BY best_score_gap) FROM coverage),
                         'represented_query_regions', (SELECT count(*) FROM regions),
                         'region_histogram', (SELECT jsonb_object_agg(query_region, jsonb_build_object('queries', queries, 'zero', zero_queries)) FROM regions)
                     )::text"
                ),
                &[],
            )
            .await
            .wrap_err("collecting Task 181 seed coverage diagnostics")?
            .get::<_, String>(0);
        lines.push(format!(
            "physical_benchmark_coverage scale={scale} head_policy={attested_head_policy} coverage_json={}",
            coverage.replace(' ', "")
        ));
    }
    let mut benchmark_arms = Vec::with_capacity(seed_variants.len() + 1);
    let traversal_replica_requested = seed_variants
        .iter()
        .any(|variant| variant.traversal_replica);
    let mut traversal_replica_digest = None;
    let mut traversal_owner_baseline = None;
    if traversal_replica_requested {
        let unavailable = nodes
            .last()
            .ok_or_else(|| color_eyre::eyre::eyre!("Task 198 fixture has no owner"))?;
        let mut stop = Command::new(pg_ctl);
        stop.arg("-w")
            .arg("-D")
            .arg(&unavailable.data_dir)
            .arg("-m")
            .arg("immediate")
            .arg("stop")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        run_status(stop)
            .await
            .wrap_err("stopping one owner for Task 198 partial-build drill")?;
        let outage_error = coordinator
            .query_one(
                "SELECT ec_distann_build_traversal_replica('dm_idx'::regclass)",
                &[],
            )
            .await
            .err();
        restart_physical_node(pg_ctl, socket_dir, unavailable, nodes).await?;
        // The intentionally killed owner also closes any warm scan connection
        // held by this coordinator backend. Exercise the documented bounded
        // reconnect path now, before later semantic comparisons could turn the
        // drill's expected outage into an unrelated one-shot failure.
        let recovery_first = task198_replica_semantic_result(
            coordinator,
            &physical_corpus,
            &physical_queries,
            -1,
            0,
            1,
        )
        .await;
        let transport_retried = recovery_first
            .as_ref()
            .is_err_and(|error| error.to_string().contains("connection closed"));
        if transport_retried {
            task198_replica_semantic_result(
                coordinator,
                &physical_corpus,
                &physical_queries,
                -1,
                0,
                1,
            )
            .await
            .wrap_err("recovering physical transport after owner restart")?;
        } else {
            recovery_first.wrap_err("probing physical transport after owner restart")?;
        }
        lines.push(format!(
            "physical_benchmark_traversal_replica_fault scale={scale} scenario=owner_restart_transport_recovery pass=true retried={transport_retried}"
        ));
        let residue = coordinator
            .query_one(
                "SELECT count(*)::bigint
                   FROM ec_distann_traversal_replica_status('dm_idx'::regclass)",
                &[],
            )
            .await?
            .get::<_, i64>(0);
        let outage_pass = outage_error.is_some() && residue == 0;
        lines.push(format!(
            "physical_benchmark_traversal_replica_fault scale={scale} scenario=owner_outage_partial_build pass={outage_pass} owner={} error={} catalog_residue={residue}",
            unavailable.node_id,
            outage_error
                .as_ref()
                .and_then(tokio_postgres::Error::as_db_error)
                .map(|error| error.message().replace(' ', "_"))
                .unwrap_or_else(|| "none".to_owned()),
        ));
        if !outage_pass {
            bail!("Task 198 owner-outage build drill left residue or did not fail");
        }
        task199_no_replica_insert_throughput(coordinator, scale, &physical_corpus, &mut lines)
            .await?;
    }
    let mut crown_storage = std::collections::HashMap::<String, (i64, i64, i64, i64)>::new();
    let mut same_seed_digests =
        std::collections::HashMap::<(String, u32, u32), (String, String)>::new();
    for variant in &seed_variants {
        let variant_beam_width = variant.beam_width.unwrap_or(beam_width);
        let variant_hop_rounds = variant.hop_rounds.unwrap_or(hop_rounds);
        let seed_label = if args.fused_head_hop {
            format!("{}+crown_fused", variant.strategy)
        } else if args.crown_width_pruning {
            format!("{}+crown_width_pruned", variant.strategy)
        } else {
            variant.strategy.clone()
        };
        if explicit_seed_controls {
            coordinator
                .batch_execute(&format!(
                    "SET ec_distann.benchmark_seed_mode = '{}';\n\
                     SET ec_distann.benchmark_head_search_width = {};\n\
                     SET ec_distann.benchmark_head_seed_count = {};\n\
                     SET ec_distann.benchmark_exact_neighbor = {};",
                    variant.strategy.replace('\'', "''"),
                    variant.head_search_width,
                    variant.head_seed_count,
                    if variant.neighbor_score_mode == "exact_neighbor" {
                        "on"
                    } else {
                        "off"
                    },
                ))
                .await
                .wrap_err("configuring Task 180 benchmark seed controls")?;
        }
        let attested_strategy = if has_seed_strategy_provenance {
            coordinator
                .query_one("SELECT ec_distann_physical_seed_strategy()", &[])
                .await?
                .get::<_, String>(0)
        } else {
            // Historical physical-generation commits predate the provenance
            // helper. Keep production benchmark configs runnable without
            // assigning a strategy label the extension cannot attest.
            "pre-provenance".to_owned()
        };
        if explicit_seed_controls && attested_strategy != variant.strategy {
            bail!(
                "physical seed strategy attestation mismatch for variant {}: requested {}, got {}",
                variant.name,
                variant.strategy,
                attested_strategy
            );
        }
        let attested_neighbor_score = if has_neighbor_score_provenance {
            coordinator
                .query_one("SELECT ec_distann_physical_neighbor_score_mode()", &[])
                .await?
                .get::<_, String>(0)
        } else {
            "rabitq".to_owned()
        };
        if explicit_seed_controls && attested_neighbor_score != variant.neighbor_score_mode {
            bail!(
                "physical neighbor-score attestation mismatch for variant {}: requested {}, got {}",
                variant.name,
                variant.neighbor_score_mode,
                attested_neighbor_score
            );
        }
        // The digest probe must execute under the same crown/fused session
        // settings as the benchmark children. Otherwise it only observes the
        // coordinator connection's default path and falsely reports same-seed
        // provenance for every arm.
        coordinator
            .batch_execute(&format!(
                "SET ec_distann.crown_capacity = {};\n\
                 SET ec_distann.crown_width_pruning = {};\n\
                 SET ec_distann.fused_head_hop = {};",
                args.crown_capacity.unwrap_or(0),
                if args.crown_width_pruning {
                    "on"
                } else {
                    "off"
                },
                if args.fused_head_hop { "on" } else { "off" },
            ))
            .await
            .wrap_err("configuring coordinator crown provenance settings")?;
        if has_seed_id_digest {
            let digest_rows = coordinator
                .query(
                    &format!(
                        "SELECT q.id::text,
                                encode(ec_distann_physical_seed_id_digest(
                                    'dm_idx'::regclass, q.source), 'hex')
                           FROM {physical_queries} q
                          ORDER BY q.id
                          LIMIT {}",
                        args.queries
                    ),
                    &[],
                )
                .await
                .wrap_err_with(|| {
                    format!("collecting seed-ID digest for variant {}", variant.name)
                })?;
            if digest_rows.len() != args.queries as usize {
                bail!(
                    "seed-ID digest variant {} expected {} queries, found {}",
                    variant.name,
                    args.queries,
                    digest_rows.len()
                );
            }
            let mut hasher = Sha256::new();
            hasher.update(b"ec_distann_seed_id_matrix_v1\0");
            hasher.update(args.queries.to_le_bytes());
            for row in digest_rows {
                let query_id = row.get::<_, String>(0);
                let seed_digest = row.get::<_, String>(1);
                let seed_digest =
                    hex::decode(&seed_digest).wrap_err("decoding extension seed-ID digest")?;
                hasher.update(
                    u32::try_from(query_id.len())
                        .wrap_err("query ID length exceeds u32")?
                        .to_le_bytes(),
                );
                hasher.update(query_id.as_bytes());
                hasher.update(seed_digest);
            }
            let seed_id_digest = hex::encode(hasher.finalize());
            let compared_with =
                register_same_seed_digest(&mut same_seed_digests, variant, &seed_id_digest)?
                    .unwrap_or_else(|| "none".to_owned());
            lines.push(format!(
                "physical_benchmark_seed_digest scale={scale} variant={} seed_strategy={} seed_set_change={} head_search_width={} head_seed_count={} beam_width={variant_beam_width} candidate_heap_limit={candidate_heap_limit} hop_rounds={variant_hop_rounds} neighbor_score_mode={} materialization_batch_size={} owner_payload_plan_cache={} traversal_replica={} queries={} seed_id_digest={} compared_with={} same_seed={}",
                variant.name,
                seed_label,
                args.fused_head_hop || args.crown_width_pruning,
                variant.head_search_width,
                variant.head_seed_count,
                variant.neighbor_score_mode,
                variant.materialization_batch_size,
                variant.owner_payload_plan_cache,
                variant.traversal_replica,
                args.queries,
                seed_id_digest,
                compared_with,
                !(args.fused_head_hop || args.crown_width_pruning),
            ));
        }
        benchmark_arms.push((
            "physical",
            physical_prefix.as_str(),
            variant.name.as_str(),
            attested_strategy,
            variant.head_search_width,
            variant.head_seed_count,
            attested_neighbor_score,
            variant.materialization_batch_size,
            variant.owner_payload_plan_cache,
            variant.traversal_replica,
            variant.typed_locator,
            variant.packed_payload,
            variant.expanded_locator,
            variant_beam_width,
            variant_hop_rounds,
        ));
        lines.push(format!(
            "physical_benchmark_build scale={scale} variant={} seed_strategy={} seed_set_change={} head_index_cap={} head_sampling_rate={:?} head_cap_floor={:?} head_cap_ceiling={:?} head_search_width={} head_seed_count={} crown_capacity={:?} crown_width_pruning={} fused_head_hop={} beam_width={variant_beam_width} candidate_heap_limit={candidate_heap_limit} hop_rounds={variant_hop_rounds} neighbor_score_mode={} materialization_batch_size={} owner_payload_plan_cache={} traversal_replica={} typed_locator={} packed_payload={} expanded_locator={} stored_neighbor_code_format=rabitq build_shared=true physical_ms={build_ms} publish_ms={publish_ms} single_ms={single_build_ms}",
            variant.name,
            seed_label,
            args.fused_head_hop || args.crown_width_pruning,
            args.head_index_cap,
            args.head_sampling_rate,
            args.head_cap_floor,
            args.head_cap_ceiling,
            variant.head_search_width,
            variant.head_seed_count,
            args.crown_capacity,
            args.crown_width_pruning,
            args.fused_head_hop,
            variant.neighbor_score_mode,
            variant.materialization_batch_size,
            variant.owner_payload_plan_cache,
            variant.traversal_replica,
            variant.typed_locator,
            variant.packed_payload,
            variant.expanded_locator,
        ));
    }
    if !args.stage_counter_only && !args.skip_single_control && !args.skip_single_benchmark {
        benchmark_arms.push((
            "single",
            single_prefix.as_str(),
            "single",
            "single_index".to_owned(),
            production_head_width,
            production_head_width,
            "rabitq".to_owned(),
            0,
            false,
            false,
            false,
            false,
            false,
            beam_width,
            hop_rounds,
        ));
    }
    // A traversal_replica=false arm is measured while no Ready image exists.
    // Only after every owner/single control has completed do we build Ready and
    // run the traversal_replica=true arms. This makes the production lifecycle,
    // not a scan-path selector GUC, the A/B boundary.
    benchmark_arms.sort_by_key(|arm| arm.9);
    let mut prediction_paths = std::collections::BTreeMap::<String, PathBuf>::new();
    let mut physical_distinct_recall = std::collections::BTreeMap::<String, f64>::new();
    let mut same_generation_identity: Option<String> = None;

    for (
        arm,
        prefix,
        variant,
        seed_strategy,
        head_search_width,
        head_seed_count,
        neighbor_score_mode,
        materialization_batch_size,
        owner_payload_plan_cache,
        traversal_replica,
        typed_locator,
        packed_payload,
        expanded_locator,
        arm_beam_width,
        arm_hop_rounds,
    ) in benchmark_arms
    {
        let seed_label = if arm == "physical" && args.fused_head_hop {
            format!("{seed_strategy}+crown_fused")
        } else if arm == "physical" && args.crown_width_pruning {
            format!("{seed_strategy}+crown_width_pruned")
        } else {
            seed_strategy.clone()
        };
        if arm == "physical" {
            // The active epoch is immutable for the duration of this lane.
            // Read it for every arm anyway: this makes a fixture replacement,
            // accidental rebuild, or candidate-induced publication fail closed
            // instead of silently invalidating the A/B comparison.
            let generation_identity = coordinator
                .query_one(
                    "SELECT encode(epoch_fingerprint, 'hex')
                       FROM ec_distann_active_epoch
                      WHERE index_oid = 'public.dm_idx'::regclass::oid",
                    &[],
                )
                .await
                .wrap_err("attesting same-generation epoch identity")?
                .get::<_, String>(0);
            let same_generation = same_generation_identity
                .as_deref()
                .is_none_or(|identity| identity == generation_identity);
            if !same_generation {
                bail!(
                    "same-generation lane observed epoch identity change for arm {variant}: expected {}, got {}",
                    same_generation_identity.as_deref().unwrap_or("none"),
                    generation_identity
                );
            }
            same_generation_identity = Some(generation_identity.clone());
            lines.push(format!(
                "physical_benchmark_generation scale={scale} variant={variant} arm=physical generation_identity={generation_identity} generation_identity_kind=epoch_fingerprint build_shared=true same_generation={same_generation}"
            ));
        }
        if traversal_replica && traversal_replica_digest.is_none() {
            traversal_owner_baseline = Some(
                task198_replica_semantic_result(
                    coordinator,
                    &physical_corpus,
                    &physical_queries,
                    -1,
                    0,
                    20,
                )
                .await?,
            );
            traversal_replica_digest =
                Some(build_and_attest_traversal_replica(coordinator, scale, &mut lines).await?);
        }
        if !args.stage_counter_only && !args.skip_recall {
            let recall_log = log_dir.join(format!("{arm}-{variant}-recall.log"));
            let predictions_output = log_dir.join(format!("{arm}-{variant}-predictions.json"));
            let mut recall_args = common.clone();
            recall_args.extend([
                "bench".into(),
                "recall".into(),
                "--prefix".into(),
                prefix.to_owned(),
                "--profile".into(),
                "ec_distann".into(),
                "--k".into(),
                args.top_k.to_string(),
                "--sweep".into(),
                "32".into(),
                "--queries-limit".into(),
                args.queries.to_string(),
                "--force-index".into(),
                "--truth-cache-file".into(),
                truth_cache.display().to_string(),
                "--log-output".into(),
                recall_log.display().to_string(),
                "--predictions-output".into(),
                predictions_output.display().to_string(),
                "--session-guc".into(),
                format!("ec_distann.beam_width={arm_beam_width}"),
                "--session-guc".into(),
                format!("ec_distann.hop_rounds={arm_hop_rounds}"),
                "--session-guc".into(),
                format!("ec_distann.candidate_heap_limit={candidate_heap_limit}"),
            ]);
            if arm == "physical" {
                for guc in &args.bench_session_gucs {
                    recall_args.extend(["--session-guc".into(), guc.clone()]);
                }
            }
            if arm == "physical" {
                recall_args.extend([
                    "--truth-corpus-file".into(),
                    truth_corpus.display().to_string(),
                    "--report-distann-crown-stats".into(),
                ]);
                if explicit_seed_controls {
                    recall_args.extend([
                        "--session-guc".into(),
                        format!("ec_distann.benchmark_seed_mode={seed_strategy}"),
                        "--session-guc".into(),
                        format!("ec_distann.benchmark_head_search_width={head_search_width}"),
                        "--session-guc".into(),
                        format!("ec_distann.benchmark_head_seed_count={head_seed_count}"),
                        "--session-guc".into(),
                        format!(
                            "ec_distann.benchmark_exact_neighbor={}",
                            if neighbor_score_mode == "exact_neighbor" {
                                "on"
                            } else {
                                "off"
                            }
                        ),
                    ]);
                }
                append_materialization_benchmark_guc(
                    &mut recall_args,
                    arm,
                    materialization_batch_size,
                );
                append_owner_payload_plan_cache_guc(
                    &mut recall_args,
                    arm,
                    owner_payload_plan_cache,
                );
                append_typed_locator_guc(&mut recall_args, arm, typed_locator);
                append_packed_payload_guc(&mut recall_args, arm, packed_payload);
                append_expanded_locator_guc(&mut recall_args, arm, expanded_locator);
                append_nonconforming_replica_guc(&mut recall_args, arm, traversal_replica);
                append_sharded_head_guc(
                    &mut recall_args,
                    arm,
                    args.sharded_head,
                    args.local_head,
                    args.head_replica_count,
                );
                append_gateway_copy_guc(&mut recall_args, arm, args.gateway_copy_capacity);
                append_crown_gucs(
                    &mut recall_args,
                    arm,
                    args.crown_capacity,
                    args.crown_width_pruning,
                    args.fused_head_hop,
                );
            }
            let recall = run_physical_bench_child(recall_args).await?;
            append_distann_notice_lines(&mut lines, &recall);
            if arm == "physical" {
                let mut crown_stats_seen = false;
                let mut reported_crown_capacity = 0_i64;
                let mut reported_crown_entries = 0_i64;
                let mut reported_crown_resident_bytes = 0_i64;
                let mut reported_crown_resident_bytes_bound = 0_i64;
                let mut crown_seeds_served = 0_i64;
                let mut crown_width_pruned_shards = 0_i64;
                let mut crown_width_pruning_activations = 0_i64;
                let mut fused_head_hops = 0_i64;
                let mut fused_first_round_requested_ids = 0_i64;
                for stats in recall
                    .lines()
                    .filter_map(|line| line.strip_prefix("[distann-crown-stats] "))
                {
                    crown_stats_seen = true;
                    reported_crown_capacity = crown_counter(stats, "capacity")
                        .ok_or_else(|| eyre!("crown stats omitted capacity"))?;
                    reported_crown_entries = crown_counter(stats, "entries")
                        .ok_or_else(|| eyre!("crown stats omitted entries"))?;
                    reported_crown_resident_bytes = crown_counter(stats, "resident_bytes")
                        .ok_or_else(|| eyre!("crown stats omitted resident_bytes"))?;
                    reported_crown_resident_bytes_bound =
                        crown_counter(stats, "resident_bytes_bound")
                            .ok_or_else(|| eyre!("crown stats omitted resident_bytes_bound"))?;
                    crown_seeds_served = crown_counter(stats, "crown_seeds_served")
                        .ok_or_else(|| eyre!("crown stats omitted crown_seeds_served"))?;
                    crown_width_pruned_shards = crown_counter(stats, "crown_width_pruned_shards")
                        .ok_or_else(|| {
                        eyre!("crown stats omitted crown_width_pruned_shards")
                    })?;
                    crown_width_pruning_activations =
                        crown_counter(stats, "crown_width_pruning_activations").ok_or_else(
                            || eyre!("crown stats omitted crown_width_pruning_activations"),
                        )?;
                    fused_head_hops = crown_counter(stats, "fused_head_hops")
                        .ok_or_else(|| eyre!("crown stats omitted fused_head_hops"))?;
                    fused_first_round_requested_ids =
                        crown_counter(stats, "fused_first_round_requested_ids").ok_or_else(
                            || eyre!("crown stats omitted fused_first_round_requested_ids"),
                        )?;
                    lines.push(format!(
                        "physical_benchmark_crown_stats scale={scale} variant={variant} arm={arm} {stats}"
                    ));
                }
                validate_crown_activation(
                    args,
                    crown_stats_seen,
                    crown_seeds_served,
                    crown_width_pruned_shards,
                    crown_width_pruning_activations,
                    fused_head_hops,
                    fused_first_round_requested_ids,
                )?;
                crown_storage.insert(
                    variant.to_owned(),
                    (
                        reported_crown_capacity,
                        reported_crown_entries,
                        reported_crown_resident_bytes,
                        reported_crown_resident_bytes_bound,
                    ),
                );
            }
            let row = benchmark_table_row(&recall)?;
            let membership_recall = row[3].parse::<f64>()?;
            let distinct_recall = row[12].parse::<f64>()?;
            let distinct_recall_ci95_low = row[13].parse::<f64>()?;
            let distinct_recall_ci95_high = row[14].parse::<f64>()?;
            let mean_ms = benchmark_ms(&row[11])?;
            if arm == "physical" {
                prediction_paths.insert(variant.to_owned(), predictions_output);
                physical_distinct_recall.insert(variant.to_owned(), distinct_recall);
            }
            if arm == "physical" && args.gateway_trace {
                let seed_strategy_sql = seed_strategy.replace('\'', "''");
                coordinator
                    .batch_execute(&format!(
                        "SET ec_distann.beam_width = {arm_beam_width};\n\
                         SET ec_distann.hop_rounds = {arm_hop_rounds};\n\
                         SET ec_distann.candidate_heap_limit = {candidate_heap_limit};\n\
                         SET ec_distann.benchmark_seed_mode = '{seed_strategy_sql}';\n\
                         SET ec_distann.benchmark_head_search_width = {head_search_width};\n\
                         SET ec_distann.benchmark_head_seed_count = {head_seed_count};\n\
                         SET ec_distann.benchmark_exact_neighbor = {};\n\
                         SET ec_distann.sharded_head_search = {};",
                        if neighbor_score_mode == "exact_neighbor" {
                            "on"
                        } else {
                            "off"
                        },
                        if args.sharded_head { "on" } else { "off" },
                    ))
                    .await
                    .wrap_err("configuring coordinator for Task 185 gateway trace")?;
                let trace_json = coordinator
                    .query_one(
                        &format!(
                            "WITH traces AS (
                                 SELECT q.id::bigint AS query_id, t.*
                                   FROM {} q
                                  CROSS JOIN LATERAL ec_distann_physical_seed_gateway_trace_benchmark(
                                      'dm_idx'::regclass, q.source, {}) t
                                  ORDER BY q.id
                                  LIMIT {}
                             )
                             SELECT jsonb_build_object(
                                 'queries', count(*),
                                 'traces', COALESCE(jsonb_agg(
                                     jsonb_build_object(
                                         'query_id', query_id,
                                         'seed_ids', seed_ids,
                                         'seed_expanded_counts', seed_expanded_counts,
                                         'seed_hit_counts', seed_hit_counts,
                                         'hit_ids', hit_ids,
                                         'hit_origin_masks', hit_origin_masks,
                                         'expanded_unique', expanded_unique,
                                         'expanded_overlap', expanded_overlap,
                                         'records_expanded', records_expanded,
                                         'rounds_executed', rounds_executed
                                     ) ORDER BY query_id
                                 ), '[]'::jsonb)
                             )::text
                               FROM traces",
                            gateway_trace_queries
                                .as_deref()
                                .expect("gateway trace training relation"),
                            args.top_k,
                            args.queries
                        ),
                        &[],
                    )
                    .await
                    .wrap_err("collecting Task 185 gateway traces")?
                    .get::<_, String>(0);
                let trace_path = log_dir.join(format!("{arm}-{variant}-gateway-trace.json"));
                fs::write(&trace_path, &trace_json).wrap_err_with(|| {
                    format!("writing Task 185 gateway trace {}", trace_path.display())
                })?;
                lines.push(format!(
                    "physical_benchmark_gateway_trace scale={scale} variant={variant} arm={arm} query_prefix=rows_201_400 queries={} top_k={} beam_width={arm_beam_width} candidate_heap_limit={candidate_heap_limit} hop_rounds={arm_hop_rounds} seed_strategy={seed_strategy} head_search_width={head_search_width} head_seed_count={head_seed_count} neighbor_score_mode={neighbor_score_mode} output={}",
                    args.queries,
                    args.top_k,
                    trace_path.display()
                ));
            }
            if arm == "physical" && args.query_trace {
                let seed_strategy_sql = seed_strategy.replace('\'', "''");
                coordinator
                    .batch_execute(&format!(
                        "SET ec_distann.beam_width = {arm_beam_width};\n\
                         SET ec_distann.hop_rounds = {arm_hop_rounds};\n\
                         SET ec_distann.candidate_heap_limit = {candidate_heap_limit};\n\
                         SET ec_distann.benchmark_seed_mode = '{seed_strategy_sql}';\n\
                         SET ec_distann.benchmark_head_search_width = {head_search_width};\n\
                         SET ec_distann.benchmark_head_seed_count = {head_seed_count};\n\
                         SET ec_distann.benchmark_exact_neighbor = {};\n\
                         SET ec_distann.sharded_head_search = {};",
                        if neighbor_score_mode == "exact_neighbor" {
                            "on"
                        } else {
                            "off"
                        },
                        if args.sharded_head { "on" } else { "off" },
                    ))
                    .await
                    .wrap_err("configuring coordinator for Task 227 query trace")?;
                let trace_json = coordinator
                    .query_one(
                        &format!(
                            "WITH traces AS (
                                 SELECT q.id::bigint AS query_id,
                                        ec_distann_physical_query_trace_benchmark(
                                            'dm_idx'::regclass, q.source, {}
                                        ) AS trace
                                   FROM dm_queries q
                                  ORDER BY q.id
                                  LIMIT {}
                             )
                             SELECT jsonb_build_object(
                                 'schema', 'ec_distann_query_trace_file_v1',
                                 'query_prefix', 'rows_{}_{}',
                                 'query_offset', {},
                                 'queries', count(*),
                                 'query_file_sha256', '{}',
                                 'query_slice_sha256', '{}',
                                 'traces', COALESCE(jsonb_agg(
                                     jsonb_build_object(
                                         'query_id', query_id,
                                         'trace', trace
                                     ) ORDER BY query_id
                                 ), '[]'::jsonb)
                             )::text
                               FROM traces",
                            args.top_k,
                            args.queries,
                            query_start,
                            query_end,
                            args.query_offset,
                            query_sha256,
                            query_slice_sha256,
                        ),
                        &[],
                    )
                    .await
                    .wrap_err("collecting Task 227 query traces")?
                    .get::<_, String>(0);
                let trace_path = log_dir.join(format!("{arm}-{variant}-query-trace.json"));
                fs::write(&trace_path, &trace_json).wrap_err_with(|| {
                    format!("writing Task 227 query trace {}", trace_path.display())
                })?;
                lines.push(format!(
                    "physical_benchmark_query_trace scale={scale} variant={variant} arm={arm} query_prefix=rows_{query_start}_{query_end} query_offset={} queries={} top_k={} beam_width={arm_beam_width} candidate_heap_limit={candidate_heap_limit} hop_rounds={arm_hop_rounds} seed_strategy={seed_strategy} head_search_width={head_search_width} head_seed_count={head_seed_count} neighbor_score_mode={neighbor_score_mode} query_file_sha256={query_sha256} query_slice_sha256={query_slice_sha256} output={}",
                    args.query_offset,
                    args.queries,
                    args.top_k,
                    trace_path.display()
                ));
            }
            if arm == "physical" && args.gateway_isolated_trace {
                let isolated_seed_count =
                    args.gateway_isolated_seed_limit.unwrap_or(head_seed_count);
                if isolated_seed_count == 0 || isolated_seed_count > head_seed_count {
                    bail!(
                        "--gateway-isolated-seed-limit {} exceeds arm {} head_seed_count {}",
                        isolated_seed_count,
                        variant,
                        head_seed_count
                    );
                }
                coordinator
                    .batch_execute(&format!(
                        "SET ec_distann.beam_width = {arm_beam_width};\n\
                         SET ec_distann.hop_rounds = {arm_hop_rounds};\n\
                         SET ec_distann.candidate_heap_limit = {candidate_heap_limit};\n\
                         SET ec_distann.benchmark_seed_mode = '{}';\n\
                         SET ec_distann.benchmark_head_search_width = {head_search_width};\n\
                         SET ec_distann.benchmark_head_seed_count = {head_seed_count};\n\
                         SET ec_distann.benchmark_exact_neighbor = {};\n\
                         SET ec_distann.sharded_head_search = {};",
                        seed_strategy.replace('\'', "''"),
                        if neighbor_score_mode == "exact_neighbor" {
                            "on"
                        } else {
                            "off"
                        },
                        if args.sharded_head { "on" } else { "off" },
                    ))
                    .await
                    .wrap_err("configuring coordinator for Task 185 isolated gateway trace")?;
                let trace_json = coordinator
                    .query_one(
                        &format!(
                            "WITH traces AS (
                                 SELECT q.id::bigint AS query_id,
                                        positions.position::integer AS seed_position,
                                        t.*
                                   FROM {} q
                                  CROSS JOIN LATERAL generate_series(1, {}) AS positions(position)
                                  CROSS JOIN LATERAL ec_distann_physical_seed_isolated_gateway_trace_benchmark(
                                      'dm_idx'::regclass, q.source, {}, positions.position::integer) t
                                  ORDER BY q.id, positions.position
                                  LIMIT {}
                             )
                             SELECT jsonb_build_object(
                                 'queries', count(DISTINCT query_id),
                                 'seed_positions', count(*),
                                 'traces', COALESCE(jsonb_agg(
                                     jsonb_build_object(
                                         'query_id', query_id,
                                         'seed_position', seed_position,
                                         'seed_ids', seed_ids,
                                         'seed_expanded_counts', seed_expanded_counts,
                                         'seed_hit_counts', seed_hit_counts,
                                         'hit_ids', hit_ids,
                                         'hit_origin_masks', hit_origin_masks,
                                         'expanded_unique', expanded_unique,
                                         'expanded_overlap', expanded_overlap,
                                         'records_expanded', records_expanded,
                                         'rounds_executed', rounds_executed
                                     ) ORDER BY query_id, seed_position
                                 ), '[]'::jsonb)
                             )::text
                               FROM traces",
                            gateway_trace_queries
                                .as_deref()
                                .expect("isolated gateway trace training relation"),
                            isolated_seed_count,
                            args.top_k,
                            args.queries.saturating_mul(isolated_seed_count),
                        ),
                        &[],
                    )
                    .await
                    .wrap_err("collecting Task 185 isolated gateway traces")?
                    .get::<_, String>(0);
                let trace_path =
                    log_dir.join(format!("{arm}-{variant}-gateway-isolated-trace.json"));
                fs::write(&trace_path, &trace_json).wrap_err_with(|| {
                    format!(
                        "writing Task 185 isolated gateway trace {}",
                        trace_path.display()
                    )
                })?;
                lines.push(format!(
                    "physical_benchmark_gateway_isolated_trace scale={scale} variant={variant} arm={arm} query_prefix=rows_201_400 queries={} seed_positions={} top_k={} beam_width={arm_beam_width} candidate_heap_limit={candidate_heap_limit} hop_rounds={arm_hop_rounds} seed_strategy={seed_strategy} head_search_width={head_search_width} head_seed_count={head_seed_count} neighbor_score_mode={neighbor_score_mode} output={}",
                    args.queries,
                    args.queries.saturating_mul(isolated_seed_count),
                    args.top_k,
                    trace_path.display()
                ));
            }
            if arm == "physical" && args.gateway_head_candidate_trace {
                let candidate_positions = args
                    .gateway_head_candidate_positions
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                coordinator
                    .batch_execute(&format!(
                        "SET ec_distann.beam_width = {arm_beam_width};\n\
                         SET ec_distann.hop_rounds = {arm_hop_rounds};\n\
                         SET ec_distann.candidate_heap_limit = {candidate_heap_limit};\n\
                         SET ec_distann.benchmark_seed_mode = '{}';\n\
                         SET ec_distann.benchmark_head_search_width = {head_search_width};\n\
                         SET ec_distann.benchmark_head_seed_count = {head_seed_count};\n\
                         SET ec_distann.benchmark_exact_neighbor = {};\n\
                         SET ec_distann.sharded_head_search = {};",
                        seed_strategy.replace('\'', "''"),
                        if neighbor_score_mode == "exact_neighbor" {
                            "on"
                        } else {
                            "off"
                        },
                        if args.sharded_head { "on" } else { "off" },
                    ))
                    .await
                    .wrap_err("configuring coordinator for Task 185 arbitrary-head trace")?;
                let trace_json = coordinator
                    .query_one(
                        &format!(
                            "WITH traces AS (
                                 SELECT q.id::bigint AS query_id,
                                        positions.position::integer AS candidate_position,
                                        t.*
                                   FROM {} q
                                  CROSS JOIN LATERAL unnest(ARRAY[{}]::integer[]) AS positions(position)
                                  CROSS JOIN LATERAL ec_distann_physical_head_candidate_trace_benchmark(
                                      'dm_idx'::regclass, q.source, {}, positions.position::integer) t
                                  ORDER BY q.id, positions.position
                                  LIMIT {}
                             )
                             SELECT jsonb_build_object(
                                 'queries', count(DISTINCT query_id),
                                 'candidate_positions', count(*),
                                 'traces', COALESCE(jsonb_agg(
                                     jsonb_build_object(
                                         'query_id', query_id,
                                         'candidate_position', candidate_position,
                                         'seed_ids', seed_ids,
                                         'seed_expanded_counts', seed_expanded_counts,
                                         'seed_hit_counts', seed_hit_counts,
                                         'hit_ids', hit_ids,
                                         'hit_origin_masks', hit_origin_masks,
                                         'expanded_unique', expanded_unique,
                                         'expanded_overlap', expanded_overlap,
                                         'records_expanded', records_expanded,
                                         'rounds_executed', rounds_executed
                                     ) ORDER BY query_id, candidate_position
                                 ), '[]'::jsonb)
                             )::text
                               FROM traces",
                            gateway_trace_queries
                                .as_deref()
                                .expect("arbitrary-head trace training relation"),
                            candidate_positions,
                            args.top_k,
                            args.queries
                                .saturating_mul(args.gateway_head_candidate_positions.len() as u32),
                        ),
                        &[],
                    )
                    .await
                    .wrap_err("collecting Task 185 arbitrary-head traces")?
                    .get::<_, String>(0);
                let trace_path =
                    log_dir.join(format!("{arm}-{variant}-gateway-head-candidate-trace.json"));
                fs::write(&trace_path, &trace_json).wrap_err_with(|| {
                    format!(
                        "writing Task 185 arbitrary-head trace {}",
                        trace_path.display()
                    )
                })?;
                lines.push(format!(
                    "physical_benchmark_gateway_head_candidate_trace scale={scale} variant={variant} arm={arm} query_prefix=rows_201_400 queries={} candidate_positions={candidate_positions} top_k={} beam_width={arm_beam_width} candidate_heap_limit={candidate_heap_limit} hop_rounds={arm_hop_rounds} seed_strategy={seed_strategy} head_search_width={head_search_width} head_seed_count={head_seed_count} neighbor_score_mode={neighbor_score_mode} output={}",
                    args.queries,
                    args.top_k,
                    trace_path.display()
                ));
            }
            lines.push(format!(
                "physical_benchmark_recall scale={scale} variant={variant} head_index_cap={} head_sampling_rate={:?} head_cap_floor={:?} head_cap_ceiling={:?} crown_capacity={:?} crown_width_pruning={} fused_head_hop={} head_search_width={head_search_width} head_seed_count={head_seed_count} beam_width={arm_beam_width} candidate_heap_limit={candidate_heap_limit} hop_rounds={arm_hop_rounds} neighbor_score_mode={neighbor_score_mode} materialization_batch_size={materialization_batch_size} owner_payload_plan_cache={owner_payload_plan_cache} traversal_replica={traversal_replica} typed_locator={typed_locator} packed_payload={packed_payload} expanded_locator={expanded_locator} arm={arm} seed_strategy={seed_label} seed_set_change={} queries={} trials={} recall={membership_recall:.4} membership_recall={membership_recall:.4} distinct_recall={distinct_recall:.4} distinct_recall_ci95_low={distinct_recall_ci95_low:.4} distinct_recall_ci95_high={distinct_recall_ci95_high:.4} mean_ms={mean_ms:.2}",
                args.head_index_cap, args.head_sampling_rate, args.head_cap_floor,
                args.head_cap_ceiling, args.crown_capacity, args.crown_width_pruning,
                args.fused_head_hop, args.fused_head_hop || args.crown_width_pruning, row[1], row[2]
            ));
        }

        let latency_log = log_dir.join(format!("{arm}-{variant}-latency.log"));
        let mut latency_args = common.clone();
        latency_args.extend([
            "bench".into(),
            "latency".into(),
            "--prefix".into(),
            prefix.to_owned(),
            "--profile".into(),
            "ec_distann".into(),
            "--k".into(),
            args.top_k.to_string(),
            "--sweep".into(),
            "32".into(),
            "--iterations".into(),
            args.benchmark_iterations.to_string(),
            "--warmup-iterations".into(),
            args.benchmark_warmup_iterations.to_string(),
            "--concurrency".into(),
            "1".into(),
            "--force-index".into(),
            "--cache-state".into(),
            "warm".into(),
            "--log-output".into(),
            latency_log.display().to_string(),
            "--session-guc".into(),
            format!("ec_distann.beam_width={arm_beam_width}"),
            "--session-guc".into(),
            format!("ec_distann.hop_rounds={arm_hop_rounds}"),
            "--session-guc".into(),
            format!("ec_distann.candidate_heap_limit={candidate_heap_limit}"),
        ]);
        if !args.benchmark_concurrency_sweep.is_empty() {
            latency_args.extend([
                "--concurrency-sweep".into(),
                args.benchmark_concurrency_sweep
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ]);
        }
        if arm == "physical" {
            for guc in &args.bench_session_gucs {
                latency_args.extend(["--session-guc".into(), guc.clone()]);
            }
        }
        if args.benchmark_backend_batch_size > 0 {
            latency_args.extend([
                "--worker-batch-size".into(),
                args.benchmark_backend_batch_size.to_string(),
            ]);
        }
        if arm == "physical" && args.benchmark_hold_transaction {
            latency_args.push("--hold-transaction".into());
        }
        if arm == "physical" {
            latency_args.push("--report-distann-crown-stats".into());
        }
        if arm == "physical" && args.sample_backend_memory {
            latency_args.extend([
                "--sample-backend-memory".into(),
                "--memory-sample-interval-ms".into(),
                args.memory_sample_interval_ms.to_string(),
                "--memory-series-output".into(),
                latency_log
                    .with_extension("memory-series.log")
                    .display()
                    .to_string(),
            ]);
        }
        if arm == "physical" && explicit_seed_controls {
            latency_args.extend([
                "--session-guc".into(),
                format!("ec_distann.benchmark_seed_mode={seed_strategy}"),
                "--session-guc".into(),
                format!("ec_distann.benchmark_head_search_width={head_search_width}"),
                "--session-guc".into(),
                format!("ec_distann.benchmark_head_seed_count={head_seed_count}"),
                "--session-guc".into(),
                format!(
                    "ec_distann.benchmark_exact_neighbor={}",
                    if neighbor_score_mode == "exact_neighbor" {
                        "on"
                    } else {
                        "off"
                    }
                ),
            ]);
        }
        append_materialization_benchmark_guc(&mut latency_args, arm, materialization_batch_size);
        append_owner_payload_plan_cache_guc(&mut latency_args, arm, owner_payload_plan_cache);
        append_typed_locator_guc(&mut latency_args, arm, typed_locator);
        append_packed_payload_guc(&mut latency_args, arm, packed_payload);
        append_expanded_locator_guc(&mut latency_args, arm, expanded_locator);
        append_nonconforming_replica_guc(&mut latency_args, arm, traversal_replica);
        append_sharded_head_guc(
            &mut latency_args,
            arm,
            args.sharded_head,
            args.local_head,
            args.head_replica_count,
        );
        append_gateway_copy_guc(&mut latency_args, arm, args.gateway_copy_capacity);
        append_crown_gucs(
            &mut latency_args,
            arm,
            args.crown_capacity,
            args.crown_width_pruning,
            args.fused_head_hop,
        );
        if arm == "physical" && args.distann_stage_counters {
            latency_args.push("--distann-stage-counters".into());
        }
        let latency = run_physical_bench_child(latency_args).await?;
        append_distann_notice_lines(&mut lines, &latency);
        if arm == "physical" {
            let mut crown_stats_seen = false;
            let mut reported_crown_capacity = 0_i64;
            let mut reported_crown_entries = 0_i64;
            let mut reported_crown_resident_bytes = 0_i64;
            let mut reported_crown_resident_bytes_bound = 0_i64;
            let mut crown_seeds_served = 0_i64;
            let mut crown_width_pruned_shards = 0_i64;
            let mut crown_width_pruning_activations = 0_i64;
            let mut fused_head_hops = 0_i64;
            let mut fused_first_round_requested_ids = 0_i64;
            for stats in latency
                .lines()
                .filter_map(|line| line.strip_prefix("[distann-crown-stats] "))
            {
                crown_stats_seen = true;
                reported_crown_capacity = crown_counter(stats, "capacity")
                    .ok_or_else(|| eyre!("crown stats omitted capacity"))?;
                reported_crown_entries = crown_counter(stats, "entries")
                    .ok_or_else(|| eyre!("crown stats omitted entries"))?;
                reported_crown_resident_bytes = crown_counter(stats, "resident_bytes")
                    .ok_or_else(|| eyre!("crown stats omitted resident_bytes"))?;
                reported_crown_resident_bytes_bound = crown_counter(stats, "resident_bytes_bound")
                    .ok_or_else(|| eyre!("crown stats omitted resident_bytes_bound"))?;
                crown_seeds_served = crown_counter(stats, "crown_seeds_served")
                    .ok_or_else(|| eyre!("crown stats omitted crown_seeds_served"))?;
                crown_width_pruned_shards = crown_counter(stats, "crown_width_pruned_shards")
                    .ok_or_else(|| eyre!("crown stats omitted crown_width_pruned_shards"))?;
                crown_width_pruning_activations =
                    crown_counter(stats, "crown_width_pruning_activations").ok_or_else(|| {
                        eyre!("crown stats omitted crown_width_pruning_activations")
                    })?;
                fused_head_hops = crown_counter(stats, "fused_head_hops")
                    .ok_or_else(|| eyre!("crown stats omitted fused_head_hops"))?;
                fused_first_round_requested_ids =
                    crown_counter(stats, "fused_first_round_requested_ids").ok_or_else(|| {
                        eyre!("crown stats omitted fused_first_round_requested_ids")
                    })?;
                lines.push(format!(
                    "physical_benchmark_crown_stats scale={scale} variant={variant} arm={arm} {stats}"
                ));
            }
            validate_crown_activation(
                args,
                crown_stats_seen,
                crown_seeds_served,
                crown_width_pruned_shards,
                crown_width_pruning_activations,
                fused_head_hops,
                fused_first_round_requested_ids,
            )?;
            crown_storage.insert(
                variant.to_owned(),
                (
                    reported_crown_capacity,
                    reported_crown_entries,
                    reported_crown_resident_bytes,
                    reported_crown_resident_bytes_bound,
                ),
            );
        }
        let rows = benchmark_table_rows(&latency);
        let expected_rows = args.benchmark_concurrency_sweep.len().max(1);
        if rows.len() != expected_rows {
            bail!(
                "physical latency returned {} rows for arm {arm:?}, expected {expected_rows}",
                rows.len()
            );
        }
        for row in rows {
            let concurrency = row
                .get(11)
                .map(|value| value.parse::<usize>())
                .transpose()
                .wrap_err("decoding physical latency concurrency")?
                .unwrap_or(1);
            let wall_ms = row.get(12).map(|value| benchmark_ms(value)).transpose()?;
            let qps = row
                .get(13)
                .map(|value| value.parse::<f64>())
                .transpose()
                .wrap_err("decoding physical latency qps")?;
            if !args.benchmark_concurrency_sweep.is_empty() && (wall_ms.is_none() || qps.is_none())
            {
                bail!("physical latency concurrency sweep row lacks wall_ms/qps for arm {arm:?}");
            }
            let wall_ms_label = wall_ms
                .map(|value| format!("{value:.2}"))
                .unwrap_or_else(|| "NA".to_owned());
            let qps_label = qps
                .map(|value| format!("{value:.3}"))
                .unwrap_or_else(|| "NA".to_owned());
            lines.push(format!(
                "physical_benchmark_latency scale={scale} variant={variant} head_index_cap={} head_sampling_rate={:?} head_cap_floor={:?} head_cap_ceiling={:?} crown_capacity={:?} crown_width_pruning={} fused_head_hop={} head_search_width={head_search_width} head_seed_count={head_seed_count} beam_width={arm_beam_width} candidate_heap_limit={candidate_heap_limit} hop_rounds={arm_hop_rounds} neighbor_score_mode={neighbor_score_mode} materialization_batch_size={materialization_batch_size} owner_payload_plan_cache={owner_payload_plan_cache} traversal_replica={traversal_replica} typed_locator={typed_locator} packed_payload={packed_payload} expanded_locator={expanded_locator} arm={arm} seed_strategy={seed_label} seed_set_change={} count={} mean_ms={:.2} p50_ms={:.2} p95_ms={:.2} p99_ms={:.2} max_ms={:.2} concurrency={concurrency} wall_ms={wall_ms_label} qps={qps_label} cache=warm warmup_iterations={} worker_batch_size={} hold_transaction={}",
                args.head_index_cap,
                args.head_sampling_rate,
                args.head_cap_floor,
                args.head_cap_ceiling,
                args.crown_capacity,
                args.crown_width_pruning,
                args.fused_head_hop,
                args.fused_head_hop || args.crown_width_pruning,
                row[1],
                benchmark_ms(&row[2])?,
                benchmark_ms(&row[5])?,
                benchmark_ms(&row[6])?,
                benchmark_ms(&row[7])?,
                benchmark_ms(&row[8])?,
                args.benchmark_warmup_iterations,
                args.benchmark_backend_batch_size,
                args.benchmark_hold_transaction && arm == "physical",
            ));
        }
        if arm == "physical" && args.distann_stage_counters {
            let stage_rows = latency
                .lines()
                .filter_map(|line| line.strip_prefix("[distann-stage-counters] "))
                .collect::<Vec<_>>();
            let expected_counter_groups = args.benchmark_concurrency_sweep.len().max(1);
            if stage_rows.len() != 37 * expected_counter_groups {
                bail!(
                    "physical latency attribution expected {} ec_distann stage rows ({} concurrency groups), got {}",
                    37 * expected_counter_groups,
                    expected_counter_groups,
                    stage_rows.len()
                );
            }
            // Reconcile the first concurrency group; all groups are retained
            // in the packet output below for the latency sweep evidence.
            let attribution_stage_rows = &stage_rows[..37];
            let remote_expand = attribution_stage_mean(attribution_stage_rows, "remote_expand")?;
            let remote_components = [
                "traversal_connection_ready",
                "traversal_request_encode",
                "traversal_owner_service",
                "traversal_transport_wait",
                "traversal_coordinator_receive_decode",
            ]
            .iter()
            .map(|stage| attribution_stage_mean(attribution_stage_rows, stage))
            .sum::<Result<f64>>()?;
            let remote_error =
                (remote_components - remote_expand).abs() / remote_expand.max(f64::EPSILON);
            let traversal_total =
                attribution_stage_mean(attribution_stage_rows, "traversal_total")?;
            let traversal_component_names: &[&str] = if traversal_replica {
                &[
                    "replica_graph_vector_read",
                    "replica_score",
                    "traversal_frontier_insert",
                ]
            } else {
                &[
                    "local_expand",
                    "remote_expand",
                    "traversal_coordinator_partition",
                    "traversal_coordinator_decode",
                    "traversal_frontier_insert",
                ]
            };
            let traversal_components = traversal_component_names
                .iter()
                .map(|stage| attribution_stage_mean(attribution_stage_rows, stage))
                .sum::<Result<f64>>()?;
            let traversal_error =
                (traversal_components - traversal_total).abs() / traversal_total.max(f64::EPSILON);
            let reconciliation_pass = remote_error <= 0.05 && traversal_error <= 0.10;
            lines.push(format!(
                "physical_benchmark_traversal_reconciliation scale={scale} variant={variant} traversal_replica={traversal_replica} arm={arm} remote_expand_ms={remote_expand:.6} remote_components_ms={remote_components:.6} remote_relative_error={remote_error:.6} remote_tolerance=0.05 traversal_total_ms={traversal_total:.6} traversal_components_ms={traversal_components:.6} traversal_relative_error={traversal_error:.6} traversal_tolerance=0.10 pass={reconciliation_pass}"
            ));
            if !reconciliation_pass {
                bail!(
                    "physical traversal attribution failed reconciliation: remote relative error {remote_error:.4}, traversal relative error {traversal_error:.4}"
                );
            }
            for stage in stage_rows {
                lines.push(format!(
                    "physical_benchmark_stage scale={scale} variant={variant} beam_width={arm_beam_width} candidate_heap_limit={candidate_heap_limit} hop_rounds={arm_hop_rounds} materialization_batch_size={materialization_batch_size} owner_payload_plan_cache={owner_payload_plan_cache} traversal_replica={traversal_replica} arm={arm} seed_strategy={seed_strategy} {stage}"
                ));
            }
            let work_rows = latency
                .lines()
                .filter_map(|line| line.strip_prefix("[distann-materialization-work] "))
                .collect::<Vec<_>>();
            // The extension exposes 33 server-side work metrics
            // (DistannMaterializationWork::ALL). The bench child appends one
            // client_result_rows metric so the measured result-consumption
            // boundary is represented in the same stream. Keep this in step
            // with the enum: adding a counter without updating it fails every
            // physical latency step.
            if work_rows.len() != 34 * expected_counter_groups {
                bail!(
                    "physical latency attribution expected {} ec_distann attribution-work rows ({} concurrency groups), got {}",
                    34 * expected_counter_groups,
                    expected_counter_groups,
                    work_rows.len()
                );
            }
            for work in work_rows {
                lines.push(format!(
                    "physical_benchmark_materialization_work scale={scale} variant={variant} beam_width={arm_beam_width} candidate_heap_limit={candidate_heap_limit} hop_rounds={arm_hop_rounds} materialization_batch_size={materialization_batch_size} owner_payload_plan_cache={owner_payload_plan_cache} traversal_replica={traversal_replica} arm={arm} seed_strategy={seed_strategy} {work}"
                ));
            }
        }
    }

    if let Some(pair) = args.same_generation_recall_pair.as_deref() {
        let (control, candidate) = pair
            .split_once(',')
            .ok_or_else(|| eyre!("same-generation recall pair must be CONTROL,CANDIDATE"))?;
        let control_path = prediction_paths.get(control).ok_or_else(|| {
            eyre!("same-generation recall control variant {control:?} produced no predictions")
        })?;
        let candidate_path = prediction_paths.get(candidate).ok_or_else(|| {
            eyre!("same-generation recall candidate variant {candidate:?} produced no predictions")
        })?;
        let control_bytes =
            std::fs::read(control_path).wrap_err("reading same-generation control predictions")?;
        let candidate_bytes = std::fs::read(candidate_path)
            .wrap_err("reading same-generation candidate predictions")?;
        let byte_identical = control_bytes == candidate_bytes;
        lines.push(format!(
            "physical_benchmark_same_generation_recall scale={scale} control={control} candidate={candidate} control_predictions={} candidate_predictions={} byte_identical={byte_identical}",
            control_path.display(),
            candidate_path.display(),
        ));
        if !byte_identical {
            bail!(
                "same-generation recall identity failed for {control} vs {candidate}: prediction files differ"
            );
        }
    }

    if let (Some(control), Some(candidate)) = (
        prediction_paths.get("bw4-control"),
        prediction_paths.get("bw8-candidate"),
    ) {
        lines.push(paired_recall_line(
            scale,
            control,
            candidate,
            &truth_cache,
            args.top_k as usize,
        )?);
    }

    if let Some(content_digest) = traversal_replica_digest.as_deref() {
        lines.extend(
            run_task199_replica_lifecycle_drills(
                coordinator,
                coordinator_port,
                scale,
                &physical_corpus,
                &physical_queries,
                content_digest,
                traversal_owner_baseline.as_deref().ok_or_else(|| {
                    color_eyre::eyre::eyre!("traversal replica lifecycle has no owner baseline")
                })?,
                args.training_query_path.as_deref(),
                enospc_fixture,
            )
            .await?,
        );
    }

    let sizes = if args.stage_counter_only || args.skip_single_control {
        coordinator
            .query_one(
                &format!(
                    "SELECT 0::bigint, 0::bigint,
                            pg_total_relation_size('{physical_corpus}'::regclass)::bigint"
                ),
                &[],
            )
            .await?
    } else {
        coordinator
            .query_one(
                &format!(
                    "SELECT pg_total_relation_size('{single_index}'::regclass)::bigint,
                            pg_total_relation_size('{single_corpus}'::regclass)::bigint,
                            pg_total_relation_size('{physical_corpus}'::regclass)::bigint"
                ),
                &[],
            )
            .await?
    };
    let single_index_bytes = sizes.get::<_, i64>(0);
    let single_source_bytes = sizes.get::<_, i64>(1);
    let coordinator_source_bytes = sizes.get::<_, i64>(2);
    let raw_vector = coordinator
        .query_one(
            &format!(
                "SELECT count(source)::bigint,
                        coalesce((SELECT array_length(source, 1)
                                    FROM {physical_corpus}
                                   LIMIT 1), 0)::bigint
                   FROM {physical_corpus}"
            ),
            &[],
        )
        .await
        .wrap_err("measuring physical raw vector bytes")?;
    let raw_vector_rows = raw_vector.get::<_, i64>(0);
    let raw_vector_dim = raw_vector.get::<_, i64>(1);
    let raw_vector_bytes = raw_vector_rows
        .saturating_mul(raw_vector_dim)
        .saturating_mul(4);
    if raw_vector_bytes <= 0 {
        bail!(
            "physical storage audit has no positive raw vector denominator: rows={raw_vector_rows} dim={raw_vector_dim}"
        );
    }
    let head = coordinator
        .query_one(
            "SELECT state.sample_count::bigint,
                    encode(state.head_sample_digest, 'hex'),
                    COALESCE((SELECT sum(pg_column_size(sample.vector) + pg_column_size(sample.vec_id))::bigint
                                FROM ec_distann_generation_head_sample sample
                               WHERE sample.index_oid = state.index_oid
                                 AND sample.logical_index_uuid = state.logical_index_uuid
                                 AND sample.build_id = state.build_id), 0),
                    COALESCE((SELECT sum(pg_column_size(sample.neighbors))::bigint
                                FROM ec_distann_generation_head_sample sample
                               WHERE sample.index_oid = state.index_oid
                                 AND sample.logical_index_uuid = state.logical_index_uuid
                                 AND sample.build_id = state.build_id), 0),
                    pg_column_size(state)::bigint
               FROM ec_distann_active_epoch active
               JOIN ec_distann_generation_head_state state
                 USING (index_oid, logical_index_uuid, build_id)
              WHERE active.index_oid = 'dm_idx'::regclass",
            &[],
        )
        .await
        .wrap_err("measuring persisted coordinator head")?;
    let head_sample_count = head.get::<_, i64>(0);
    let head_sample_digest = head.get::<_, String>(1);
    let head_sample_bytes = head.get::<_, i64>(2);
    // The state row is bounded control metadata (digests, counts, and under
    // membership-only storage the bounded id list) — itemised separately as
    // control state, not folded into the corpus-shaped head relations
    // (005 review round 2: the zero-byte gate is about topology/vector rows).
    let head_graph_bytes = head.get::<_, i64>(3);
    let head_state_bytes = head.get::<_, i64>(4);
    let head_cache_estimated_bytes = head_sample_bytes + head_graph_bytes;
    let head_membership = coordinator
        .query_one(
            "SELECT state.head_construction,
                    state.membership,
                    COALESCE((SELECT array_agg(sample.vec_id ORDER BY sample.sample_ordinal)
                                FROM ec_distann_generation_head_sample sample
                               WHERE sample.index_oid = state.index_oid
                                 AND sample.logical_index_uuid = state.logical_index_uuid
                                 AND sample.build_id = state.build_id), ARRAY[]::bigint[])
               FROM ec_distann_active_epoch active
               JOIN ec_distann_generation_head_state state
                 USING (index_oid, logical_index_uuid, build_id)
              WHERE active.index_oid = 'dm_idx'::regclass",
            &[],
        )
        .await
        .wrap_err("reading persisted head membership")?;
    let head_construction = match head_membership.get::<_, i16>(0) {
        0 => "stitched_bfs",
        1 => "partition_union",
        other => bail!("invalid persisted head construction marker {other}"),
    };
    let mut head_ids = head_membership.get::<_, Vec<i64>>(2);
    if head_ids.is_empty() {
        if let Some(blob) = head_membership.get::<_, Option<Vec<u8>>>(1) {
            if blob.len() < 4 {
                bail!("persisted head membership blob is truncated");
            }
            let count = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
            if count != head_sample_count as usize || blob.len() != 4 + count * 8 {
                bail!(
                    "persisted head membership blob has count={count} bytes={} expected_count={head_sample_count}",
                    blob.len()
                );
            }
            head_ids = blob[4..]
                .chunks_exact(8)
                .map(|chunk| i64::from_le_bytes(chunk.try_into().unwrap()))
                .collect();
        }
    }
    if head_ids.len() != head_sample_count as usize {
        bail!(
            "persisted head membership count={} does not match sample_count={head_sample_count}",
            head_ids.len()
        );
    }
    let logical_id_by_vec_id = coordinator
        .query(
            &format!("SELECT id, source_id::text FROM {physical_corpus}"),
            &[],
        )
        .await
        .wrap_err("reading logical ids for persisted head membership")?
        .into_iter()
        .map(|row| {
            let logical_id = row.get::<_, i64>(0);
            let source_id = row.get::<_, String>(1);
            let identity = source_identity_uuid_bytes(&source_id)?;
            Ok((distann_vec_id_from_source_identity(&identity), logical_id))
        })
        .collect::<Result<HashMap<_, _>>>()?;
    let logical_head_ids = head_ids
        .iter()
        .map(|vec_id| {
            logical_id_by_vec_id.get(vec_id).copied().ok_or_else(|| {
                eyre!("persisted head vec_id {vec_id} has no logical source-id mapping")
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut membership_hasher = Sha256::new();
    for id in &head_ids {
        membership_hasher.update(id.to_le_bytes());
    }
    let head_membership_path = log_dir.join("physical-head-membership.json");
    let head_membership_json = serde_json::json!({
        "scale": scale,
        "head_construction": head_construction,
        "sample_count": head_ids.len(),
        "head_sample_digest": head_sample_digest,
        "ids_sha256": hex::encode(membership_hasher.finalize()),
        "vec_ids": head_ids,
        "logical_ids": logical_head_ids,
    });
    fs::write(
        &head_membership_path,
        serde_json::to_vec_pretty(&head_membership_json)?,
    )
    .wrap_err_with(|| format!("writing {}", head_membership_path.display()))?;
    let remote_owners = if args.coordinator_outside_roster {
        nodes.len()
    } else {
        nodes.len().saturating_sub(1)
    };
    for variant in &seed_variants {
        let variant_beam_width = variant.beam_width.unwrap_or(beam_width);
        let variant_hop_rounds = variant.hop_rounds.unwrap_or(hop_rounds);
        let shared = format!(
            "variant={} seed_strategy={} head_index_cap={} head_search_width={} head_seed_count={} beam_width={variant_beam_width} candidate_heap_limit={candidate_heap_limit} hop_rounds={variant_hop_rounds} neighbor_score_mode={} materialization_batch_size={} owner_payload_plan_cache={} traversal_replica={}",
            variant.name,
            variant.strategy,
            args.head_index_cap,
            variant.head_search_width,
            variant.head_seed_count,
            variant.neighbor_score_mode,
            variant.materialization_batch_size,
            variant.owner_payload_plan_cache,
            variant.traversal_replica,
        );
        // NFR-018/NFR-021 storage is deliberately measured inside the arm
        // loop.  The owner generation rows are immutable across variants, but
        // derived relations (notably the optional traversal replica) are not;
        // replaying one pre-loop scalar would make the arm comparison
        // unmeasurable.
        let owner_graph_side_bytes = published
            .iter()
            .map(|row| row.graph_bytes + row.directory_bytes + row.control_bytes)
            .sum::<i64>();
        let owner_row_tier_bytes = published.iter().map(|row| row.row_bytes).sum::<i64>();
        let owner_total_bytes = owner_graph_side_bytes + owner_row_tier_bytes;
        let max_owner_graph_side_bytes = published
            .iter()
            .map(|row| row.graph_bytes + row.directory_bytes + row.control_bytes)
            .max()
            .unwrap_or(0);
        let physical_generation_bytes = published
            .iter()
            .map(|row| row.graph_bytes + row.row_bytes + row.directory_bytes + row.control_bytes)
            .sum::<i64>();
        let control_index_bytes = published.iter().map(|row| row.control_bytes).sum::<i64>();
        let mut derived_relation_bytes = 0_i64;
        if variant.traversal_replica {
            let replica = coordinator
                .query_one(
                    "SELECT relation_bytes::bigint, coalesce(wal_bytes, 0)::bigint,
                            copied_bytes::bigint, coalesce(build_duration_ms, 0)::bigint,
                            replica_relid::oid::bigint
                       FROM ec_distann_traversal_replica_status('dm_idx'::regclass)
                      WHERE state = 'Ready'
                      ORDER BY ready_at DESC
                      LIMIT 1",
                    &[],
                )
                .await
                .wrap_err("measuring per-arm traversal replica storage")?;
            let relation_bytes = replica.get::<_, i64>(0);
            let replica_relid = replica.get::<_, i64>(4);
            derived_relation_bytes = relation_bytes;
            lines.push(format!(
                "physical_benchmark_storage_relation scale={scale} {shared} arm=physical node=coordinator node_role=coordinator relation=physical_benchmark_traversal_replica relation_oid={replica_relid} relation_bytes={relation_bytes} wal_bytes={} copied_bytes={} build_ms={} storage_derived=true",
                replica.get::<_, i64>(1),
                replica.get::<_, i64>(2),
                replica.get::<_, i64>(3),
            ));
            let cache = coordinator
                .query_one(
                    "SELECT coalesce(io.heap_blks_read, 0)::bigint,
                            coalesce(io.heap_blks_hit, 0)::bigint,
                            pg_relation_size(status.replica_relid)::bigint
                       FROM ec_distann_traversal_replica_status('dm_idx'::regclass) status
                       LEFT JOIN pg_statio_all_tables io
                         ON io.relid = status.replica_relid
                      WHERE status.state = 'Ready'
                      ORDER BY status.ready_at DESC
                      LIMIT 1",
                    &[],
                )
                .await
                .wrap_err("measuring per-arm traversal replica cache residency")?;
            lines.push(format!(
                "physical_benchmark_traversal_replica_cache scale={scale} {shared} arm=physical heap_blocks_read={} heap_blocks_hit={} heap_bytes={} cache_residency_proxy=pg_statio",
                cache.get::<_, i64>(0),
                cache.get::<_, i64>(1),
                cache.get::<_, i64>(2),
            ));
        }
        // NFR-021 clause 2/3: the coordinator's own index-derived state is
        // itemised per relation, not reported as zero. The head sample and its
        // Vamana graph are coordinator-resident and unsharded; they are
        // generation-scoped and therefore identical across arms, which is
        // recorded as `arm_invariant=true` rather than implied by reprinting a
        // pre-loop scalar (the Task 204 defect).
        lines.push(format!(
            "physical_benchmark_storage_relation scale={scale} {shared} arm=physical node=coordinator node_role=coordinator relation=ec_distann_generation_head_sample relation_bytes={head_sample_bytes} storage_derived=false arm_invariant=true nfr_021_class=coordinator_resident_unsharded",
        ));
        lines.push(format!(
            "physical_benchmark_storage_relation scale={scale} {shared} arm=physical node=coordinator node_role=coordinator relation=ec_distann_generation_head_graph relation_bytes={head_graph_bytes} storage_derived=false arm_invariant=true nfr_021_class=coordinator_resident_unsharded",
        ));
        lines.push(format!(
            "physical_benchmark_storage_relation scale={scale} {shared} arm=physical node=coordinator node_role=coordinator relation=ec_distann_generation_head_state relation_bytes={head_state_bytes} storage_derived=false arm_invariant=true nfr_021_class=control",
        ));
        let (crown_capacity, crown_entries, crown_resident_bytes, crown_resident_bytes_bound) =
            crown_storage
                .get(&variant.name)
                .copied()
                .unwrap_or_default();
        if crown_resident_bytes > crown_resident_bytes_bound {
            bail!(
                "crown resident bytes exceed bound for variant {}: {} > {}",
                variant.name,
                crown_resident_bytes,
                crown_resident_bytes_bound
            );
        }
        lines.push(format!(
            "physical_benchmark_storage_relation scale={scale} {shared} arm=physical node=coordinator node_role=coordinator relation=ec_distann_crown_cache relation_bytes={crown_resident_bytes} crown_capacity={crown_capacity} crown_entries={crown_entries} crown_resident_bytes={crown_resident_bytes} crown_resident_bytes_bound={crown_resident_bytes_bound} storage_derived=false nfr_021_class=bounded_codes_only within_capacity_bound=true",
        ));
        let coordinator_head_bytes = head_sample_bytes + head_graph_bytes;
        let coordinator_total_resident_bytes =
            derived_relation_bytes + coordinator_head_bytes + crown_resident_bytes;
        let cluster_graph_side_bytes = owner_graph_side_bytes + derived_relation_bytes;
        let max_single_node_graph_side_bytes =
            max_owner_graph_side_bytes.max(derived_relation_bytes);
        let cluster_index_space_amplification =
            cluster_graph_side_bytes as f64 / raw_vector_bytes as f64;
        lines.push(format!(
            "physical_benchmark_storage_ratio scale={scale} {shared} arm=physical raw_vector_rows={raw_vector_rows} raw_vector_dim={raw_vector_dim} raw_vector_bytes={raw_vector_bytes} cluster_graph_side_bytes={cluster_graph_side_bytes} cluster_index_space_amplification={cluster_index_space_amplification:.6} max_single_node_graph_side_bytes={max_single_node_graph_side_bytes} max_single_node_growth_reference=100k_div_10k",
        ));
        for row in published {
            let graph_side_bytes = row.graph_bytes + row.directory_bytes + row.control_bytes;
            lines.push(format!(
                "physical_benchmark_storage_node scale={scale} {shared} arm=physical node={} node_role=owner graph_bytes={} directory_bytes={} control_bytes={} graph_side_bytes={graph_side_bytes} row_tier_bytes={} total_resident_bytes={} derived_relation_bytes=0",
                row.node_id,
                row.graph_bytes,
                row.directory_bytes,
                row.control_bytes,
                row.row_bytes,
                graph_side_bytes + row.row_bytes,
            ));
        }
        lines.push(format!(
            "physical_benchmark_storage_node scale={scale} {shared} arm=physical node=coordinator node_role=coordinator graph_bytes=0 directory_bytes=0 control_bytes=0 graph_side_bytes={derived_relation_bytes} row_tier_bytes=0 head_sample_bytes={head_sample_bytes} head_graph_bytes={head_graph_bytes} crown_resident_bytes={crown_resident_bytes} coordinator_resident_unsharded_bytes={coordinator_head_bytes} total_resident_bytes={coordinator_total_resident_bytes} derived_relation_bytes={derived_relation_bytes} relations_itemised=true",
        ));
        lines.push(format!(
            "physical_benchmark_storage scale={scale} {shared} arm=physical stored_neighbor_code_format=rabitq storage_shared=false owners={} physical_generation_bytes={physical_generation_bytes} owner_graph_side_bytes={owner_graph_side_bytes} owner_row_tier_bytes={owner_row_tier_bytes} owner_total_bytes={owner_total_bytes} derived_relation_bytes={derived_relation_bytes} cluster_graph_side_bytes={cluster_graph_side_bytes} max_single_node_graph_side_bytes={max_single_node_graph_side_bytes} control_index_bytes={control_index_bytes} coordinator_source_bytes={coordinator_source_bytes} single_index_bytes={single_index_bytes} single_source_bytes={single_source_bytes} raw_vector_bytes={raw_vector_bytes} cluster_index_space_amplification={cluster_index_space_amplification:.6}",
            published.len(),
        ));
        lines.push(format!(
            "physical_benchmark_head scale={scale} {shared} arm=physical stored_neighbor_code_format=rabitq storage_shared=true sample_count={head_sample_count} head_sample_digest={head_sample_digest} head_sample_bytes={head_sample_bytes} head_graph_bytes={head_graph_bytes} head_cache_estimated_bytes={head_cache_estimated_bytes}"
        ));
        lines.push(format!(
            "physical_benchmark_head_membership scale={scale} {shared} head_construction={head_construction} sample_count={} ids_sha256={} artifact={}",
            head_ids.len(),
            head_membership_json["ids_sha256"].as_str().unwrap_or_default(),
            head_membership_path.display(),
        ));
        lines.push(format!(
            "physical_benchmark_engagement scale={scale} {shared} arm=physical remote_owners={remote_owners} materialize_probes={remote_owners} pass={}",
            remote_owners > 0
        ));
    }
    if args.stage_counter_only || args.skip_single_control {
        lines.push(format!(
            "physical_benchmark_insert_throughput_ab scale={scale} pass=false reason=single_control_skipped"
        ));
    } else {
        let production = seed_variants
            .iter()
            .find(|variant| variant.name == "production")
            .ok_or_else(|| {
                eyre!(
                    "Task 167 exact-recall closeout requires a physical seed variant named production"
                )
            })?;
        if !args.skip_recall {
            let production_predictions = prediction_paths.get("production").ok_or_else(|| {
                eyre!("Task 167 production recall arm produced no prediction artifact")
            })?;
            let ordinary_distinct_recall = physical_distinct_recall
                .get("production")
                .copied()
                .ok_or_else(|| eyre!("Task 167 production recall arm produced no recall score"))?;
            lines.push(task167_pre_insert_recall_calibration_line(
                scale,
                production_predictions,
                &truth_cache,
                ordinary_distinct_recall,
                args.top_k as usize,
            )?);
        }
        let search_guc_sql = task167_search_guc_sql(
            args,
            production,
            beam_width,
            candidate_heap_limit,
            hop_rounds,
        )?;
        let default_insert_baseline = task167_default_insert_throughput(
            coordinator,
            scale,
            &physical_corpus,
            &single_corpus,
            args.graph_degree,
            &mut lines,
        )
        .await?;
        let roster = nodes
            .iter()
            .map(|node| {
                format!(
                    "{}@{} options=-cstatement_timeout=3600000",
                    node.node_id,
                    conninfo(socket_dir, node.port)
                )
            })
            .collect::<Vec<_>>()
            .join(";");
        let parity_query_count = args.benchmark_parity_queries.unwrap_or(args.queries);
        let exact_recall_lines = task167_post_insert_exact_recall(
            coordinator,
            scale,
            &physical_corpus,
            &roster,
            args.graph_degree,
            args.head_index_cap,
            args.build_shards,
            &args.head_construction,
            &head_sizing_reloptions(args),
            parity_query_count,
            &truth_corpus,
            &truth_queries,
            &search_guc_sql,
            "post_160_shipped_default_established_tie_priority_inserts",
            &[2_000_000_i64],
            args.task167_heldout_baseline_deficit,
            args.task167_heldout_physical_sample_sd,
        )
        .await?;
        task167_quality_gate_failed = task167_quality_gate_failure(&exact_recall_lines).is_some();
        lines.extend(exact_recall_lines);
        if task167_quality_gate_failed {
            lines.push(format!(
                "physical_benchmark_backlink_strategy_ab scale={scale} pass=skipped reason=candidate_default_quality_gate_failed control_mutation_excluded=true"
            ));
        } else {
            task167_append_when_room_diagnostic(
                coordinator,
                scale,
                &physical_corpus,
                default_insert_baseline,
                &mut lines,
            )
            .await?;
        }
    }
    if task167_quality_gate_failed && args.materialization_correctness {
        lines.push(format!(
            "physical_benchmark_materialization_correctness scale={scale} pass=skipped reason=candidate_default_quality_gate_failed"
        ));
    } else if args.materialization_correctness {
        lines.extend(
            run_materialization_correctness(
                coordinator,
                pg_ctl,
                socket_dir,
                nodes,
                &seed_variants,
                scale,
                &physical_corpus,
                &physical_queries,
            )
            .await?,
        );
    }
    for line in &mut lines {
        line.push_str(&format!(
            " corpus_prefix={corpus_prefix} query_sha256={query_sha256} query_offset={} query_slice_sha256={query_slice_sha256} extension_git_sha={expected_sha} extension_build_profile={expected_profile}",
            args.query_offset
        ));
    }
    Ok(lines)
}

fn benchmark_log_value(line: &str, key: &str) -> Option<String> {
    line.split_whitespace()
        .find_map(|field| field.strip_prefix(&format!("{key}=")))
        .map(ToOwned::to_owned)
}

fn distann_vec_id_from_source_identity(identity: &[u8; 16]) -> i64 {
    let low = u64::from_le_bytes(identity[..8].try_into().expect("identity low bytes"));
    let high = u64::from_le_bytes(identity[8..].try_into().expect("identity high bytes"));
    let mut value = low ^ 0x6469_7374_616e_6e01;
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^= value >> 33;
    value = value.wrapping_add(high);
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^= value >> 33;
    value as i64
}

fn source_identity_uuid_bytes(value: &str) -> Result<[u8; 16]> {
    let compact = value.replace('-', "");
    let bytes = hex::decode(compact).wrap_err("decoding source identity UUID")?;
    bytes
        .try_into()
        .map_err(|_| eyre!("source identity UUID has invalid length"))
}

fn benchmark_log_line<'a>(contents: &'a str, prefix: &str) -> Option<&'a str> {
    contents.lines().find(|line| line.contains(prefix))
}

async fn validate_reused_physical_fixture(
    args: &LocalMultinodePg18Args,
    socket_dir: &Path,
    nodes: &[Node],
    log_dir: &Path,
    extension_preflight: &ExtensionPreflight,
) -> Result<(String, i64)> {
    let corpus_prefix = args
        .corpus_prefix
        .as_deref()
        .ok_or_else(|| color_eyre::eyre::eyre!("fixture reuse requires corpus_prefix"))?;
    let scale = corpus_prefix
        .strip_prefix("ec_real_")
        .unwrap_or(corpus_prefix);
    let prior_log_dir = args.reuse_provenance_dir.clone().or_else(|| {
        log_dir
            .parent()
            .map(|parent| parent.join("counters-off-100k"))
    });
    let provenance_dirs = prior_log_dir
        .into_iter()
        .chain(std::iter::once(log_dir.to_owned()));
    let contents = provenance_dirs
        .filter_map(|directory| {
            let summary_path = directory.join("distann-multinode-summary.log");
            let log_path = directory.join("distann-local-multinode.log");
            fs::read_to_string(summary_path)
                .ok()
                .or_else(|| fs::read_to_string(log_path).ok())
        })
        .next()
        .ok_or_else(|| {
            color_eyre::eyre::eyre!(
                "--reuse-fixture requires an attested benchmark log under {}",
                log_dir.display()
            )
        })?;
    let provenance = benchmark_log_line(&contents, "physical_benchmark_provenance")
        .ok_or_else(|| color_eyre::eyre::eyre!("reuse log has no physical provenance line"))?;
    let build = benchmark_log_line(&contents, "physical_benchmark_build")
        .ok_or_else(|| color_eyre::eyre::eyre!("reuse log has no physical build line"))?;
    let expected_sha = &extension_preflight.git_sha;
    let expected_profile = &extension_preflight.build_profile;
    let expected_query_sha = {
        let staged_dir = args
            .staged_dir
            .clone()
            .unwrap_or(repo_root()?.join("data/staged-current"));
        let query_path = fs::canonicalize(staged_dir.join(format!("{corpus_prefix}_queries.tsv")))?;
        hex::encode(Sha256::digest(fs::read(query_path)?))
    };
    let expected_source_count = {
        let staged_dir = args
            .staged_dir
            .clone()
            .unwrap_or(repo_root()?.join("data/staged-current"));
        let corpus_path = fs::canonicalize(staged_dir.join(format!("{corpus_prefix}_corpus.tsv")))?;
        fs::read_to_string(corpus_path)?.lines().count() as i64
    };
    let beam_width = args.beam_width.unwrap_or(4);
    let expected_seed = args.seed_strategy.as_deref().unwrap_or("head_sample_exact");
    let expected_head_width = args.head_search_width.unwrap_or((beam_width * 2).max(32));
    let expected_head_count = args.head_seed_count.unwrap_or(expected_head_width);
    let expected_neighbor = args.neighbor_score_mode.as_deref().unwrap_or("rabitq");
    let expected_head_cap = args.head_index_cap.to_string();
    let expected_head_width = expected_head_width.to_string();
    let expected_head_count = expected_head_count.to_string();
    let checks = [
        ("scale", scale, benchmark_log_value(provenance, "scale")),
        (
            "corpus_prefix",
            corpus_prefix,
            benchmark_log_value(provenance, "corpus_prefix"),
        ),
        (
            "query_sha256",
            expected_query_sha.as_str(),
            benchmark_log_value(provenance, "query_sha256"),
        ),
        (
            "extension_git_sha",
            expected_sha.as_str(),
            benchmark_log_value(provenance, "extension_git_sha"),
        ),
        (
            "extension_build_profile",
            expected_profile.as_str(),
            benchmark_log_value(provenance, "extension_build_profile"),
        ),
        (
            "head_index_cap",
            expected_head_cap.as_str(),
            benchmark_log_value(build, "head_index_cap"),
        ),
        (
            "seed_strategy",
            expected_seed,
            benchmark_log_value(build, "seed_strategy"),
        ),
        (
            "head_search_width",
            expected_head_width.as_str(),
            benchmark_log_value(build, "head_search_width"),
        ),
        (
            "head_seed_count",
            expected_head_count.as_str(),
            benchmark_log_value(build, "head_seed_count"),
        ),
        (
            "neighbor_score_mode",
            expected_neighbor,
            benchmark_log_value(build, "neighbor_score_mode"),
        ),
        (
            "stored_neighbor_code_format",
            "rabitq",
            benchmark_log_value(build, "stored_neighbor_code_format"),
        ),
    ];
    for (key, expected, actual) in checks {
        let actual = actual.ok_or_else(|| {
            color_eyre::eyre::eyre!("reuse provenance is missing required field {key}")
        })?;
        if actual != expected {
            bail!(
                "--reuse-fixture provenance mismatch for {key}: requested {expected}, existing {actual}"
            );
        }
    }
    let (coordinator, connection) =
        tokio_postgres::connect(&conninfo(socket_dir, nodes[0].port), tokio_postgres::NoTls)
            .await
            .wrap_err("connecting to reused physical fixture")?;
    let connection_task = tokio::spawn(async move { connection.await });
    let reloptions = coordinator
        .query_one(
            "SELECT coalesce(array_to_string(reloptions, ','), '')
               FROM pg_class WHERE oid = 'public.dm_idx'::regclass",
            &[],
        )
        .await?
        .get::<_, String>(0);
    let options = reloptions.replace(' ', "");
    if !options
        .split(',')
        .any(|option| option == format!("graph_degree={}", args.graph_degree))
    {
        connection_task.abort();
        bail!(
            "--reuse-fixture graph_degree mismatch: requested {}, existing reloptions={reloptions}",
            args.graph_degree
        );
    }
    if !options.contains("neighbor_code_format=rabitq") {
        connection_task.abort();
        bail!("--reuse-fixture codec mismatch: existing index is not rabitq");
    }
    let physical_corpus = format!("task179_physical_{scale}_corpus");
    let source_count = coordinator
        .query_one(
            &format!("SELECT count(*)::bigint FROM {physical_corpus}"),
            &[],
        )
        .await?
        .get::<_, i64>(0);
    connection_task.abort();
    if source_count != expected_source_count {
        bail!(
            "--reuse-fixture row-count mismatch: corpus expects {}, existing {}",
            expected_source_count,
            source_count
        );
    }
    Ok((scale.to_owned(), source_count))
}

async fn drive_reused_physical_fixture(
    args: &LocalMultinodePg18Args,
    pg_ctl: &Path,
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    log_dir: &Path,
    extension_preflight: &ExtensionPreflight,
) -> Result<()> {
    let (scale, source_count) =
        validate_reused_physical_fixture(args, socket_dir, nodes, log_dir, extension_preflight)
            .await?;
    let coordinator_conninfo = conninfo(socket_dir, nodes[0].port);
    let (coordinator, connection) =
        tokio_postgres::connect(&coordinator_conninfo, tokio_postgres::NoTls)
            .await
            .wrap_err("connecting persistent reused physical coordinator session")?;
    let connection_task = tokio::spawn(async move { connection.await });
    if args.physical_benchmark {
        coordinator
            .batch_execute("SET ec_distann.remote_statement_timeout_ms = 3600000")
            .await
            .wrap_err("setting large-scale physical benchmark remote timeout")?;
    }
    let fingerprint = coordinator
        .query_one(
            "SELECT encode(epoch_fingerprint, 'hex')
               FROM ec_distann_active_epoch
              WHERE index_oid = 'public.dm_idx'::regclass::oid",
            &[],
        )
        .await?
        .get::<_, String>(0);
    let selector = format!(
        "ec_distann_epoch_topology('public.dm_idx'::regclass, decode('{fingerprint}', 'hex'))"
    );
    let owners = if args.coordinator_outside_roster {
        &nodes[1..]
    } else {
        nodes
    };
    let mut published = Vec::with_capacity(owners.len());
    for node in owners {
        published.push(physical_topology(psql, socket_dir, node, &selector).await?);
    }
    validate_physical_topology("reused", &published, "Published", source_count)?;
    crate::ecaz_println!(
        "[distann-multicluster] fixture_decision action=reuse run_dir={} scale={} source_rows={} extension_git_sha={} extension_build_profile={}",
        nodes[0].data_dir.parent().unwrap_or(nodes[0].data_dir.as_path()).display(),
        scale,
        source_count,
        extension_preflight.git_sha,
        extension_preflight.build_profile
    );
    let benchmark_lines = if args.physical_benchmark {
        run_physical_benchmarks(
            args,
            &coordinator,
            nodes[0].port,
            pg_ctl,
            socket_dir,
            owners,
            &published,
            log_dir,
            0,
            0,
            extension_preflight,
            None,
        )
        .await?
    } else {
        Vec::new()
    };
    for line in &benchmark_lines {
        crate::ecaz_println!("[distann-multicluster] {line}");
    }
    drop(coordinator);
    connection_task.abort();
    let mut summary = format!(
        "physical_fixture_decision action=reuse scale={scale} source_rows={source_count}\n"
    );
    for row in &published {
        summary.push_str(&format!(
            "[distann-multicluster] physical_topology phase=reused node={} state={} records={} rows={} non_owned={} orphans={} graph_bytes={} row_bytes={} directory_bytes={} control_bytes={}\n",
            row.node_id,
            row.state,
            row.records,
            row.rows,
            row.non_owned_live + row.non_owned_tombstones,
            row.orphan_records + row.orphan_rows,
            row.graph_bytes,
            row.row_bytes,
            row.directory_bytes,
            row.control_bytes,
        ));
    }
    for line in &benchmark_lines {
        summary.push_str(&format!("[distann-multicluster] {line}\n"));
    }
    fs::write(log_dir.join("distann-multinode-summary.log"), summary)?;
    enforce_task167_quality_gate(&benchmark_lines)
}

async fn drive_physical_fixture(
    args: &LocalMultinodePg18Args,
    pg_ctl: &Path,
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    log_dir: &Path,
    extension_preflight: &ExtensionPreflight,
    enospc_fixture: Option<&Task199EnospcFixture>,
) -> Result<()> {
    crate::ecaz_println!(
        "[distann-multicluster] physical_setup_start rows={} nodes={}",
        args.rows,
        nodes.len()
    );
    for (ordinal, node) in nodes.iter().enumerate() {
        let setup = physical_setup_sql(args, ordinal == 0)?;
        run_psql_file(psql, socket_dir, node.port, &setup)
            .await
            .wrap_err_with(|| format!("physical shell setup on node {}", node.node_id))?;
        run_psql_file(
            psql,
            socket_dir,
            node.port,
            &format!(
                "SELECT ec_distann_configure_participant_identity(
                    'public.dm_idx'::regclass, 'physical/node-{}')",
                node.node_id
            ),
        )
        .await?;
    }
    let source_count = capture_psql(psql, socket_dir, nodes[0].port, "SELECT count(*) FROM dm")
        .await?
        .trim()
        .parse::<i64>()?;
    if args.corpus_prefix.is_some() {
        let query_count = capture_psql(
            psql,
            socket_dir,
            nodes[0].port,
            "SELECT count(*) FROM dm_queries",
        )
        .await?
        .trim()
        .parse::<u32>()?;
        if query_count != args.queries {
            bail!(
                "staged query slice is short: offset={} requested={} loaded={query_count}",
                args.query_offset,
                args.queries
            );
        }
    }

    let coordinator_conninfo = conninfo(socket_dir, nodes[0].port);
    let (coordinator, connection) =
        tokio_postgres::connect(&coordinator_conninfo, tokio_postgres::NoTls)
            .await
            .wrap_err("connecting persistent physical coordinator session")?;
    let connection_task = tokio::spawn(async move { connection.await });
    if args.physical_benchmark {
        coordinator
            .batch_execute("SET ec_distann.remote_statement_timeout_ms = 3600000")
            .await
            .wrap_err("setting large-scale physical benchmark remote timeout")?;
    }
    let synthetic_norm_error = preflight_synthetic_unit_norm(&coordinator, args.dim).await?;
    crate::ecaz_println!(
        "[distann-multicluster] physical_synthetic_unit_norm samples=32 dimensions={} max_abs_error={synthetic_norm_error:.9} tolerance=0.000010000 pass=true",
        args.dim,
    );
    let owners = if args.coordinator_outside_roster {
        &nodes[1..]
    } else {
        nodes
    };
    for (ordinal, node) in owners.iter().enumerate() {
        coordinator
            .batch_execute(&format!(
                "SELECT ec_distann_register_node_descriptor(
                    'public.dm_idx'::regclass, {ordinal}, {}, 'physical/node-{}',
                    'DISTANN_NODE_{}', 'public.dm_idx', {})",
                node.node_id,
                node.node_id,
                node.node_id,
                !args.coordinator_outside_roster && ordinal == 0
            ))
            .await
            .wrap_err_with(|| format!("registering physical node {}", node.node_id))?;
    }
    if args.production_head_policy.as_deref() == Some("training_landmarks_exact") {
        let training_path = std::fs::canonicalize(
            args.training_query_path
                .as_deref()
                .expect("validated trained policy has a query path"),
        )?;
        let training_path = training_path.display().to_string().replace('\'', "''");
        coordinator
            .batch_execute(&format!(
                "CREATE TEMP TABLE ec_distann_training_stage (
                     load_ordinal bigserial, source_id bigint, vec text
                 );
                 COPY ec_distann_training_stage (source_id, vec)
                   FROM '{training_path}' WITH (FORMAT text, DELIMITER E'\\t');
                 CREATE TEMP TABLE ec_distann_training_queries AS
                 SELECT (load_ordinal - 200)::bigint AS training_ordinal,
                        translate(vec, '[]', '{{}}')::real[] AS vector
                   FROM ec_distann_training_stage
                  WHERE load_ordinal BETWEEN 201 AND 400
                  ORDER BY load_ordinal;
                 DROP TABLE ec_distann_training_stage;"
            ))
            .await
            .wrap_err("staging Task 182 production training relation")?;
    }
    let physical_started = Instant::now();
    let build_id = "71717171-7171-4171-8171-717171717171";
    if let Some(policy) = args.head_policy.as_deref() {
        let training = match args.training_query_path.as_deref() {
            Some(path) => format!(
                "SET ec_distann.benchmark_training_query_path = '{}';",
                std::fs::canonicalize(path)?
                    .display()
                    .to_string()
                    .replace('\'', "''")
            ),
            None => "RESET ec_distann.benchmark_training_query_path;".to_owned(),
        };
        coordinator
            .batch_execute(&format!(
                "SET ec_distann.benchmark_head_policy = '{}'; {training}",
                policy.replace('\'', "''")
            ))
            .await
            .wrap_err("configuring Task 181 benchmark head builder")?;
    }
    if args.sharded_head {
        // Task 210 P2a: the head is persisted by T2 inside ec_distann_build_epoch,
        // not by CREATE INDEX, so the membership-only GUC has to be set on the
        // session that runs the build. Setting it at index creation is a silent
        // no-op -- the first A/B measured identical coordinator bytes because of
        // exactly that.
        coordinator
            .batch_execute("SET ec_distann.shard_head_storage = on;")
            .await
            .wrap_err("enabling Task 210 membership-only head storage")?;
    }
    if args.local_head {
        coordinator
            .batch_execute("SET ec_distann.shard_head_storage = off;")
            .await
            .wrap_err("forcing the legacy coordinator-local head control")?;
    }
    coordinator
        .batch_execute(&format!(
            "SELECT ec_distann_begin_epoch_build('public.dm_idx'::regclass, 1, '{build_id}'::uuid)"
        ))
        .await?;
    let build_sql = if args.production_head_policy.as_deref() == Some("training_landmarks_exact") {
        format!(
            "SELECT ec_distann_build_epoch_with_training(
                 'public.dm_idx'::regclass, 1, '{build_id}'::uuid,
                 'ec_distann_training_queries'::regclass);
             DROP TABLE ec_distann_training_queries;"
        )
    } else {
        format!("SELECT ec_distann_build_epoch('public.dm_idx'::regclass, 1, '{build_id}'::uuid)")
    };
    coordinator.batch_execute(&build_sql).await?;
    let physical_build_ms = physical_started.elapsed().as_millis();

    let ready_selector =
        format!("ec_distann_generation_topology('public.dm_idx'::regclass, '{build_id}'::uuid)");
    let mut ready = Vec::with_capacity(owners.len());
    for node in owners {
        let row = physical_topology(psql, socket_dir, node, &ready_selector).await?;
        crate::ecaz_println!(
            "[distann-multicluster] physical_topology phase=ready node={} state={} records={} rows={} non_owned={} orphans={} graph_bytes={} row_bytes={} directory_bytes={} control_bytes={}",
            row.node_id,
            row.state,
            row.records,
            row.rows,
            row.non_owned_live + row.non_owned_tombstones,
            row.orphan_records + row.orphan_rows,
            row.graph_bytes,
            row.row_bytes,
            row.directory_bytes,
            row.control_bytes,
        );
        ready.push(row);
    }
    validate_physical_topology("ready", &ready, "Ready", source_count)?;

    coordinator
        .batch_execute(&format!(
            "SELECT ec_distann_decide_epoch_publish('public.dm_idx'::regclass, '{build_id}'::uuid)"
        ))
        .await?;
    let publish_fault_lines = if !args.skip_fault_drills
        && !args.physical_benchmark
        && !args.coordinator_outside_roster
        && owners.len() >= 3
    {
        physical_publish_fault_drills(
            &coordinator,
            pg_ctl,
            psql,
            socket_dir,
            nodes,
            owners,
            build_id,
        )
        .await?
    } else {
        coordinator
            .batch_execute(&format!(
                "SELECT ec_distann_recover_epoch_publish('public.dm_idx'::regclass, '{build_id}'::uuid)"
            ))
            .await?;
        Vec::new()
    };
    for line in &publish_fault_lines {
        crate::ecaz_println!("[distann-multicluster] {line}");
    }
    let physical_publish_ms = physical_started.elapsed().as_millis();
    let fingerprint = coordinator
        .query_one(
            "SELECT encode(epoch_fingerprint, 'hex') FROM ec_distann_active_epoch
              WHERE index_oid = 'public.dm_idx'::regclass::oid",
            &[],
        )
        .await?
        .get::<_, String>(0);
    let published_selector = format!(
        "ec_distann_epoch_topology('public.dm_idx'::regclass, decode('{fingerprint}', 'hex'))"
    );
    let mut published = Vec::with_capacity(owners.len());
    for node in owners {
        let row = physical_topology(psql, socket_dir, node, &published_selector).await?;
        crate::ecaz_println!(
            "[distann-multicluster] physical_topology phase=published node={} state={} records={} rows={} non_owned={} orphans={} graph_bytes={} row_bytes={} directory_bytes={} control_bytes={}",
            row.node_id,
            row.state,
            row.records,
            row.rows,
            row.non_owned_live + row.non_owned_tombstones,
            row.orphan_records + row.orphan_rows,
            row.graph_bytes,
            row.row_bytes,
            row.directory_bytes,
            row.control_bytes,
        );
        published.push(row);
    }
    validate_physical_topology("published", &published, "Published", source_count)?;

    let query_limit = i64::from(args.top_k.max(1));
    coordinator
        .batch_execute("SET enable_seqscan = off")
        .await?;
    let served = coordinator
        .query_one(
            &format!(
                "SELECT count(*) FROM (
                     SELECT source_id FROM dm
                      ORDER BY embedding <#> (SELECT source FROM dm ORDER BY id LIMIT 1)
                      LIMIT {query_limit}
                 ) served"
            ),
            &[],
        )
        .await?
        .get::<_, i64>(0);
    let serving_ok = served == query_limit.min(source_count);
    crate::ecaz_println!(
        "[distann-multicluster] physical_serving pass={} rows={} owners={} source_rows={}",
        serving_ok,
        served,
        owners.len(),
        source_count
    );
    if !serving_ok {
        bail!("physical serving returned {served} rows, expected {query_limit}");
    }
    if args.remote_insert_probe {
        let remote_insert_ok = physical_remote_insert_probe(
            psql,
            socket_dir,
            nodes[0].port,
            args,
            nodes,
            owners.len(),
        )
        .await?;
        crate::ecaz_println!(
            "[distann-multicluster] physical_remote_insert_probe pass={remote_insert_ok}"
        );
        if !remote_insert_ok {
            bail!("physical remote insert probe did not commit on a remote owner");
        }
    }
    let mut remote_verified = 0_usize;
    let mut remote_fault_lines = Vec::new();
    let remote_owners = if args.coordinator_outside_roster {
        owners
    } else {
        &owners[1..]
    };
    for node in remote_owners {
        let row_relation = capture_psql(
            psql,
            socket_dir,
            node.port,
            &format!(
                "SELECT row_tier_relid::regclass::text FROM ec_distann_generation
                  WHERE index_oid = 'public.dm_idx'::regclass::oid
                    AND epoch_fingerprint = decode('{fingerprint}', 'hex')"
            ),
        )
        .await?;
        let row_relation = row_relation.trim();
        if row_relation.is_empty()
            || !row_relation
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'"'))
        {
            bail!("remote owner {} returned unsafe row relation", node.node_id);
        }
        let sample = capture_psql(
            psql,
            socket_dir,
            node.port,
            &format!(
                "SELECT source_id::text || '|' || source::text
                   FROM {row_relation} ORDER BY source_id LIMIT 1"
            ),
        )
        .await?;
        let (source_id, vector) = sample
            .trim()
            .split_once('|')
            .ok_or_else(|| color_eyre::eyre::eyre!("remote owner sample is malformed"))?;
        if source_id.len() != 36
            || !source_id.bytes().enumerate().all(|(index, byte)| {
                if matches!(index, 8 | 13 | 18 | 23) {
                    byte == b'-'
                } else {
                    byte.is_ascii_hexdigit()
                }
            })
            || !vector.starts_with('{')
            || !vector.ends_with('}')
            || !vector.bytes().all(|byte| {
                byte.is_ascii_digit()
                    || matches!(byte, b'{' | b'}' | b',' | b'.' | b'-' | b'+' | b'e' | b'E')
            })
        {
            bail!(
                "remote owner {} returned malformed identity/vector",
                node.node_id
            );
        }
        // The ec_distann distributed planner intentionally requires a constant
        // query vector. Extended-protocol parameters remain Params during path
        // creation and therefore exercise the local AM path instead. `vector`
        // has already passed the strict numeric-array allowlist above, so use a
        // literal here to prove the same CustomScan shape as production literal
        // and benchmark queries.
        let owner_query = format!(
            "SELECT source_id FROM dm
              ORDER BY embedding <#> '{vector}'::real[]
              LIMIT {source_count}"
        );
        let owner_plan = coordinator
            .query(
                &format!("EXPLAIN (FORMAT TEXT, COSTS OFF) {owner_query}"),
                &[],
            )
            .await?
            .into_iter()
            .map(|row| row.get::<_, String>(0))
            .collect::<Vec<_>>()
            .join("\n");
        if !owner_plan.contains("EcDistannDistributedScan") {
            bail!(
                "remote-owner proof did not plan EcDistannDistributedScan for owner {}: {}",
                node.node_id,
                owner_plan
            );
        }
        if node.node_id == 2 {
            if let Some(fault) = args.remote_socket_fault {
                let line = run_remote_socket_fault_probe(
                    &coordinator,
                    &owner_query,
                    fault,
                    args.remote_socket_fault_latency_ms,
                    &log_dir.join("distann-remote-socket-fault.arm"),
                    &log_dir.join("distann-remote-socket-fault.marker"),
                    node.port,
                    source_id,
                )
                .await?;
                crate::ecaz_println!("[distann-multicluster] {line}");
                remote_fault_lines.push(line);
            }
        }
        let pinned_owner_query = format!(
            "SELECT source_id::text FROM dm
              WHERE source_id = '{source_id}'::uuid
              ORDER BY embedding <#> '{vector}'::real[]
              LIMIT 1"
        );
        let pinned_probe_output =
            capture_psql_allow_error(psql, socket_dir, nodes[0].port, &pinned_owner_query).await;
        let pinned_probe_source_id = pinned_probe_output
            .lines()
            .map(str::trim)
            .find(|candidate| *candidate == source_id);
        // This probe is diagnostic only: zero_rows and returned_sample are
        // both expected under ANN post-filter semantics because the equality
        // qual is applied after the distributed top-k. The owner_exact_probe
        // below is the authoritative placement proof; record both outcomes
        // so the packet does not mistake the probe for a serving assertion.
        let pinned_probe_status = if pinned_probe_source_id.is_some() {
            "returned_sample"
        } else if pinned_probe_output
            .lines()
            .all(|line| line.trim().is_empty())
        {
            "zero_rows"
        } else {
            "error_or_unexpected_row"
        };
        let owner_exact_probe = capture_psql_allow_error(
            psql,
            socket_dir,
            node.port,
            &format!(
                "SELECT source_id::text FROM {row_relation} \
                  WHERE source_id = '{source_id}'::uuid LIMIT 1;"
            ),
        )
        .await;
        let owner_exact_source_id = owner_exact_probe
            .lines()
            .map(str::trim)
            .find(|candidate| *candidate == source_id);
        if owner_exact_source_id.is_none() {
            bail!(
                "remote owner exact pinned probe did not return sampled source_id for node {}; sampled_source_id={source_id}; output={}",
                node.node_id,
                compact_capture_error(&owner_exact_probe),
            );
        }
        let materialized_rows = coordinator
            .query(
                &format!("SELECT source_id::text FROM ({owner_query}) q"),
                &[],
            )
            .await?;
        let mut materialized_source_id = None;
        for row in &materialized_rows {
            let candidate_source_id = row.get::<_, String>(0);
            if !candidate_source_id
                .bytes()
                .enumerate()
                .all(|(index, byte)| {
                    if matches!(index, 8 | 13 | 18 | 23) {
                        byte == b'-'
                    } else {
                        byte.is_ascii_hexdigit()
                    }
                })
            {
                continue;
            }
            let owner_match = capture_psql_allow_error(
                psql,
                socket_dir,
                node.port,
                &format!(
                    "SELECT count(*) FROM {row_relation} \
                      WHERE source_id = '{candidate_source_id}'::uuid;"
                ),
            )
            .await;
            if owner_match
                .lines()
                .find_map(|line| line.trim().parse::<i64>().ok())
                == Some(1)
            {
                materialized_source_id = Some(candidate_source_id);
                break;
            }
        }
        let Some(materialized_source_id) = materialized_source_id else {
            bail!(
                "remote owner materialization returned no row belonging to owner node {} among {} rows; sampled_source_id={source_id}; owner_query={owner_query}",
                node.node_id,
                materialized_rows.len(),
            );
        };
        let owner_served =
            owner_exact_source_id == Some(source_id) && !materialized_source_id.is_empty();
        crate::ecaz_println!(
            "[distann-multicluster] physical_remote_owner node={} custom_scan=true pass={} expected_source_id={} materialized_source_id={} pinned_probe={} pinned_probe_output={} owner_exact_probe={}",
            node.node_id,
            owner_served,
            source_id,
            materialized_source_id,
            pinned_probe_status,
            compact_capture_error(&pinned_probe_output),
            compact_capture_error(&owner_exact_probe)
        );
        if !owner_served {
            bail!(
                "coordinator did not materialize selected row from remote owner {}",
                node.node_id
            );
        }
        remote_verified += 1;
    }
    let physical_mid_insert_ok = mid_insert_drill(psql, socket_dir, nodes[0].port, args).await;
    crate::ecaz_println!(
        "[distann-multicluster] physical_mid_insert_failure pass={physical_mid_insert_ok}"
    );
    if !physical_mid_insert_ok {
        bail!("physical TC-043 mid-insert drill failed");
    }
    let benchmark_lines = if args.physical_benchmark {
        run_physical_benchmarks(
            args,
            &coordinator,
            nodes[0].port,
            pg_ctl,
            socket_dir,
            owners,
            &published,
            log_dir,
            physical_build_ms,
            physical_publish_ms,
            extension_preflight,
            enospc_fixture,
        )
        .await?
    } else {
        Vec::new()
    };
    for line in &benchmark_lines {
        crate::ecaz_println!("[distann-multicluster] {line}");
    }
    if let Some(failure) = task167_quality_gate_failure(&benchmark_lines) {
        let mut summary = format!(
            "physical_fixture owners={} coordinator_outside_roster={} source_rows={} quality_gate_pass=false\n",
            owners.len(),
            args.coordinator_outside_roster,
            source_count,
        );
        for (phase, rows) in [("ready", &ready), ("published", &published)] {
            for row in rows {
                summary.push_str(&format!(
                    "[distann-multicluster] physical_topology phase={phase} node={} state={} records={} rows={} non_owned={} orphans={} graph_bytes={} row_bytes={} directory_bytes={} control_bytes={}\n",
                    row.node_id,
                    row.state,
                    row.records,
                    row.rows,
                    row.non_owned_live + row.non_owned_tombstones,
                    row.orphan_records + row.orphan_rows,
                    row.graph_bytes,
                    row.row_bytes,
                    row.directory_bytes,
                    row.control_bytes,
                ));
            }
        }
        for line in &publish_fault_lines {
            summary.push_str(&format!("[distann-multicluster] {line}\n"));
        }
        for line in &benchmark_lines {
            summary.push_str(&format!("[distann-multicluster] {line}\n"));
        }
        summary.push_str(
            "[distann-multicluster] physical_quality_gate pass=false control_mutation_excluded=true post_gate_drills=skipped\n",
        );
        let summary_path = log_dir.join("distann-multinode-summary.log");
        fs::write(&summary_path, summary)
            .wrap_err_with(|| format!("writing {}", summary_path.display()))?;
        crate::ecaz_println!(
            "[distann-multicluster] failed-gate summary written to {}",
            summary_path.display()
        );
        drop(coordinator);
        connection_task.abort();
        bail!(failure);
    }
    let concurrency_table = if args.physical_benchmark {
        let corpus_prefix = args
            .corpus_prefix
            .as_deref()
            .ok_or_else(|| color_eyre::eyre::eyre!("physical benchmark requires corpus_prefix"))?;
        let scale = corpus_prefix
            .strip_prefix("ec_real_")
            .unwrap_or(corpus_prefix);
        format!("task179_physical_{scale}_corpus")
    } else {
        "dm".to_owned()
    };
    // The 50k/100k physical head search can legitimately exceed PostgreSQL's
    // default ten-minute statement timeout while the owner scans the larger
    // staged generation. Keep the benchmark workload unchanged, but give the
    // remote owner sessions the same bounded one-hour measurement allowance;
    // this is required in the conninfo because PGOPTIONS on the coordinator
    // does not propagate through the backend-created owner connections.
    let fixture_roster = nodes
        .iter()
        .map(|node| {
            format!(
                "{}@{} options=-cstatement_timeout=3600000",
                node.node_id,
                conninfo(socket_dir, node.port)
            )
        })
        .collect::<Vec<_>>()
        .join(";");
    let physical_concurrency_ok = if args.skip_concurrency_drill {
        crate::ecaz_println!(
            "[distann-multicluster] physical_concurrent_insert_query pass=skipped reason=skip_concurrency_drill"
        );
        true
    } else {
        physical_concurrency_drill(
            psql,
            socket_dir,
            nodes[0].port,
            args,
            nodes,
            &fixture_roster,
            &concurrency_table,
            &fingerprint,
        )
        .await?
    };
    // This diagnostic is reverse-edge coverage: `found` contains inserted
    // vec_ids discovered in other nodes' neighbour lists. It is not the
    // number of forward edges selected by inserted nodes; the controlled
    // two-writer target assertion below is the separate backlink invariant.
    crate::ecaz_println!(
        "[distann-multicluster] physical_concurrent_insert_query pass={physical_concurrency_ok}"
    );
    if !physical_concurrency_ok {
        bail!("physical TC-043 concurrent insert/query drill failed");
    }
    let physical_delete_vacuum_ok = physical_routed_delete_vacuum_drill(
        psql,
        socket_dir,
        nodes[0].port,
        nodes,
        args,
        &fixture_roster,
        &concurrency_table,
        &fingerprint,
    )
    .await?;
    if !physical_delete_vacuum_ok {
        bail!("physical routed DELETE + VACUUM drill failed");
    }
    if args.drop_extension_cleanup_drill {
        let unpublished_build_id = "72727272-7272-4272-8272-727272727272";
        coordinator
            .batch_execute(&format!(
                "SELECT ec_distann_begin_epoch_build('public.dm_idx'::regclass, 2, '{unpublished_build_id}'::uuid)"
            ))
            .await?;
        coordinator
            .batch_execute(&format!(
                "SELECT ec_distann_build_epoch('public.dm_idx'::regclass, 2, '{unpublished_build_id}'::uuid)"
            ))
            .await?;
    }
    drop(coordinator);
    connection_task.abort();
    let drop_extension_lines = if args.drop_extension_cleanup_drill {
        physical_drop_extension_cleanup_drill(psql, socket_dir, nodes).await?
    } else {
        Vec::new()
    };
    for line in &drop_extension_lines {
        crate::ecaz_println!("[distann-multicluster] {line}");
    }
    crate::ecaz_println!(
        "[distann-multicluster] physical_topology_gate pass=true owners={} remote_verified={} source_rows={}",
        owners.len(),
        remote_verified,
        source_count
    );
    let mut summary = format!(
        "physical_fixture owners={} coordinator_outside_roster={} source_rows={}\n",
        owners.len(),
        args.coordinator_outside_roster,
        source_count
    );
    for (phase, rows) in [("ready", &ready), ("published", &published)] {
        for row in rows {
            summary.push_str(&format!(
                "[distann-multicluster] physical_topology phase={phase} node={} state={} records={} rows={} non_owned={} orphans={} graph_bytes={} row_bytes={} directory_bytes={} control_bytes={}\n",
                row.node_id,
                row.state,
                row.records,
                row.rows,
                row.non_owned_live + row.non_owned_tombstones,
                row.orphan_records + row.orphan_rows,
                row.graph_bytes,
                row.row_bytes,
                row.directory_bytes,
                row.control_bytes,
            ));
        }
    }
    summary.push_str(&format!(
        "[distann-multicluster] physical_serving pass=true rows={served} owners={} source_rows={source_count}\n",
        owners.len()
    ));
    for line in &publish_fault_lines {
        summary.push_str(&format!("[distann-multicluster] {line}\n"));
    }
    for line in &benchmark_lines {
        summary.push_str(&format!("[distann-multicluster] {line}\n"));
    }
    summary.push_str(&format!(
        "[distann-multicluster] physical_mid_insert_failure pass={physical_mid_insert_ok}\n"
    ));
    summary.push_str(&format!(
        "[distann-multicluster] physical_concurrent_insert_query pass={physical_concurrency_ok}\n"
    ));
    summary.push_str(&format!(
        "[distann-multicluster] physical_routed_delete_vacuum pass={physical_delete_vacuum_ok}\n"
    ));
    for line in &drop_extension_lines {
        summary.push_str(&format!("[distann-multicluster] {line}\n"));
    }
    for line in &remote_fault_lines {
        summary.push_str(&format!("[distann-multicluster] {line}\n"));
    }
    summary.push_str(&format!(
        "[distann-multicluster] physical_topology_gate pass=true owners={} remote_verified={remote_verified} source_rows={source_count}\n",
        owners.len()
    ));
    let summary_path = log_dir.join("distann-multinode-summary.log");
    fs::write(&summary_path, summary)
        .wrap_err_with(|| format!("writing {}", summary_path.display()))?;
    crate::ecaz_println!(
        "[distann-multicluster] summary written to {}",
        summary_path.display()
    );
    Ok(())
}

async fn run_remote_socket_fault_probe(
    coordinator: &tokio_postgres::Client,
    owner_query: &str,
    fault: RemoteSocketFaultArg,
    latency_ms: u64,
    arm_file: &Path,
    marker: &Path,
    peer_port: u16,
    expected_source_id: &str,
) -> Result<String> {
    let probe_sql = format!("SELECT source_id::text FROM ({owner_query}) q");
    let baseline_started = Instant::now();
    let baseline = coordinator
        .query_opt(&probe_sql, &[])
        .await
        .wrap_err("running disarmed DistANN remote socket baseline")?;
    let baseline_source_id = baseline
        .map(|row| row.get::<_, String>(0))
        .ok_or_else(|| eyre!("disarmed DistANN remote socket baseline returned no row"))?;
    if baseline_source_id != expected_source_id {
        bail!(
            "disarmed DistANN remote socket baseline returned {baseline_source_id}, expected {expected_source_id}"
        );
    }
    let baseline_ms = baseline_started.elapsed().as_millis();

    fs::write(arm_file, "").wrap_err_with(|| format!("arming {}", arm_file.display()))?;
    let fault_started = Instant::now();
    let outcome = coordinator.query_opt(&probe_sql, &[]).await;
    let fault_ms = fault_started.elapsed().as_millis();
    fs::remove_file(arm_file).wrap_err_with(|| format!("disarming {}", arm_file.display()))?;

    match fault {
        RemoteSocketFaultArg::Reset if outcome.is_ok() => {
            bail!("armed DistANN socket-reset query unexpectedly succeeded")
        }
        RemoteSocketFaultArg::Reset => {}
        RemoteSocketFaultArg::Slow => {
            let fault_source_id = outcome
                .wrap_err("armed DistANN socket-slow query failed")?
                .map(|row| row.get::<_, String>(0))
                .ok_or_else(|| eyre!("armed DistANN socket-slow query returned no row"))?;
            if fault_source_id != expected_source_id {
                bail!(
                    "armed DistANN socket-slow query returned {fault_source_id}, expected {expected_source_id}"
                );
            }
            let required_fault_ms = baseline_ms.saturating_add(u128::from(latency_ms));
            if fault_ms < required_fault_ms {
                bail!(
                    "armed DistANN socket-slow query took {fault_ms} ms versus {baseline_ms} ms baseline, below required baseline-plus-latency {required_fault_ms} ms"
                );
            }
        }
    }

    let marker_content =
        fs::read_to_string(marker).wrap_err_with(|| format!("reading {}", marker.display()))?;
    let expected_mode = fault.provider_mode().as_str();
    let expected_target = format!("target=tcp:127.0.0.1:{peer_port}");
    if !marker_content.lines().any(|line| {
        line.contains("fault=1")
            && line.contains(&format!("mode={expected_mode}"))
            && line.contains(&expected_target)
    }) {
        bail!(
            "DistANN remote socket marker has no exact-peer fault event for {expected_mode} {expected_target}"
        );
    }
    let recovered_source_id = coordinator
        .query_opt(&probe_sql, &[])
        .await
        .wrap_err("running disarmed DistANN remote socket recovery query")?
        .map(|row| row.get::<_, String>(0))
        .ok_or_else(|| eyre!("disarmed DistANN remote socket recovery returned no row"))?;
    if recovered_source_id != expected_source_id {
        bail!(
            "disarmed DistANN remote socket recovery returned {recovered_source_id}, expected {expected_source_id}"
        );
    }
    Ok(format!(
        "remote_socket_fault mode={} peer=tcp:127.0.0.1:{} baseline_ms={} fault_ms={} expected_source_id={} recovered_source_id={} fault_event=true disarmed=true recovery=true",
        expected_mode,
        peer_port,
        baseline_ms,
        fault_ms,
        expected_source_id,
        recovered_source_id
    ))
}

async fn physical_drop_extension_cleanup_drill(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
) -> Result<Vec<String>> {
    let mut lines = Vec::with_capacity(nodes.len());
    for node in nodes {
        let hidden_before = capture_psql(
            psql,
            socket_dir,
            node.port,
            "SELECT count(*) FROM pg_class WHERE relname ~ '^_ecdz_'",
        )
        .await?
        .trim()
        .parse::<i64>()?;
        let states_before = capture_psql(
            psql,
            socket_dir,
            node.port,
            "SELECT
                 count(*) FILTER (WHERE state = 'Ready')::text || '|' ||
                 count(*) FILTER (WHERE state = 'Published')::text
               FROM ec_distann_generation
              WHERE index_oid = 'public.dm_idx'::regclass::oid",
        )
        .await?;
        let state_counts = states_before
            .trim()
            .split('|')
            .map(str::parse::<i64>)
            .collect::<Result<Vec<_>, _>>()?;
        let ready_before = state_counts.first().copied().unwrap_or(0);
        let published_before = state_counts.get(1).copied().unwrap_or(0);
        if hidden_before < 6 || ready_before < 1 || published_before < 1 {
            bail!(
                "physical DROP EXTENSION drill precondition failed on node {}: hidden={hidden_before} Ready={ready_before} Published={published_before}",
                node.node_id,
            );
        }
        run_psql_file(
            psql,
            socket_dir,
            node.port,
            "DROP EXTENSION ecaz CASCADE;
             CREATE TABLE ecaz_drop_extension_probe(id integer PRIMARY KEY);
             INSERT INTO ecaz_drop_extension_probe VALUES (1);",
        )
        .await
        .wrap_err_with(|| format!("dropping ecaz on physical node {}", node.node_id))?;
        let after = capture_psql(
            psql,
            socket_dir,
            node.port,
            "SELECT
                 (SELECT count(*) FROM pg_extension WHERE extname = 'ecaz')::text || '|' ||
                 (SELECT count(*) FROM pg_class WHERE relname ~ '^_ecdz_')::text || '|' ||
                 (SELECT count(*) FROM ecaz_drop_extension_probe)::text",
        )
        .await?;
        let fields = after.trim().split('|').collect::<Vec<_>>();
        if fields.as_slice() != ["0", "0", "1"] {
            bail!(
                "physical DROP EXTENSION cleanup failed on node {}: expected 0|0|1, got {}",
                node.node_id,
                after.trim()
            );
        }
        lines.push(format!(
            "physical_drop_extension_cleanup pass=true node={} ready_before={ready_before} published_before={published_before} hidden_before={hidden_before} hidden_after=0 extension_after=0 post_drop_dml_rows=1",
            node.node_id
        ));
    }
    Ok(lines)
}

async fn physical_publish_fault_drills(
    coordinator: &tokio_postgres::Client,
    pg_ctl: &Path,
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    owners: &[Node],
    build_id: &str,
) -> Result<Vec<String>> {
    let recover_sql = format!(
        "SELECT ec_distann_recover_epoch_publish('public.dm_idx'::regclass, '{build_id}'::uuid)"
    );
    coordinator
        .batch_execute("SET ec_distann.debug_fail_recover_after_publish_ack = on")
        .await?;

    // Real partial-ack window: local publication remains transactional on the
    // coordinator, owner 2 commits its remote acknowledgement, and the last
    // owner crashes before it can acknowledge. The coordinator must retain the
    // durable Pending/Decided decision with no active pointer.
    let unavailable = owners
        .last()
        .ok_or_else(|| color_eyre::eyre::eyre!("physical fault drill has no owner"))?;
    let mut stop = Command::new(pg_ctl);
    stop.arg("-w")
        .arg("-D")
        .arg(&unavailable.data_dir)
        .arg("-m")
        .arg("immediate")
        .arg("stop")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    run_status(stop)
        .await
        .wrap_err("crashing the last physical owner for the publish drill")?;

    let down_error = coordinator.batch_execute(&recover_sql).await.err();
    let (decision, registration, active_count) =
        physical_publish_coordinator_state(coordinator, build_id).await?;
    let local_state = physical_generation_state(psql, socket_dir, &owners[0], build_id).await?;
    let acked_state = physical_generation_state(psql, socket_dir, &owners[1], build_id).await?;
    let participant_down_pass = down_error.is_some()
        && decision == "Pending"
        && registration == "Decided"
        && active_count == 0
        && local_state == "Ready"
        && acked_state == "Published";
    let mut lines = vec![format!(
        "physical_publish_fault participant_down_partial pass={participant_down_pass} decision={decision} registration={registration} active_count={active_count} local_state={local_state} remote_acked_state={acked_state} unavailable_node={}",
        unavailable.node_id
    )];
    if !participant_down_pass {
        bail!(
            "physical participant-down publish drill failed: {}",
            lines.last().expect("fault line")
        );
    }

    restart_physical_node(pg_ctl, socket_dir, unavailable, nodes).await?;

    // With every owner reachable, the debug hook fails after all participant
    // acknowledgements but before the coordinator active-pointer swap. Remote
    // acknowledgements persist; the local publish rolls back with T4a.
    let injected_error = coordinator.batch_execute(&recover_sql).await.err();
    let (decision, registration, active_count) =
        physical_publish_coordinator_state(coordinator, build_id).await?;
    let mut states = Vec::with_capacity(owners.len());
    for owner in owners {
        states.push(physical_generation_state(psql, socket_dir, owner, build_id).await?);
    }
    let post_ack_pass = injected_error
        .as_ref()
        .and_then(tokio_postgres::Error::as_db_error)
        .is_some_and(|error| error.message().contains("EC_FAULT_INJECTED"))
        && decision == "Pending"
        && registration == "Decided"
        && active_count == 0
        && states.first().is_some_and(|state| state == "Ready")
        && states.iter().skip(1).all(|state| state == "Published");
    lines.push(format!(
        "physical_publish_fault post_ack_pre_pointer pass={post_ack_pass} decision={decision} registration={registration} active_count={active_count} owner_states={}",
        states.join(",")
    ));
    if !post_ack_pass {
        bail!(
            "physical post-ack/pre-pointer publish drill failed: {}",
            lines.last().expect("fault line")
        );
    }

    coordinator
        .batch_execute("SET ec_distann.debug_fail_recover_after_publish_ack = off")
        .await?;
    coordinator.batch_execute(&recover_sql).await?;
    let (decision, registration, active_count) =
        physical_publish_coordinator_state(coordinator, build_id).await?;
    let mut recovered_states = Vec::with_capacity(owners.len());
    for owner in owners {
        recovered_states.push(physical_generation_state(psql, socket_dir, owner, build_id).await?);
    }
    let recovery_pass = decision == "Applied"
        && registration == "Published"
        && active_count == 1
        && recovered_states.iter().all(|state| state == "Published");
    lines.push(format!(
        "physical_publish_fault idempotent_recovery pass={recovery_pass} decision={decision} registration={registration} active_count={active_count} owner_states={}",
        recovered_states.join(",")
    ));
    if !recovery_pass {
        bail!(
            "physical idempotent publish recovery failed: {}",
            lines.last().expect("fault line")
        );
    }
    Ok(lines)
}

async fn physical_publish_coordinator_state(
    coordinator: &tokio_postgres::Client,
    build_id: &str,
) -> Result<(String, String, i64)> {
    let row = coordinator
        .query_one(
            "SELECT
                 COALESCE((SELECT decision_state FROM ec_distann_publish_decision
                            WHERE build_id = $1::text::uuid), 'Missing'),
                 COALESCE((SELECT state FROM ec_distann_build_registration
                            WHERE build_id = $1::text::uuid), 'Missing'),
                 (SELECT count(*) FROM ec_distann_active_epoch
                   WHERE build_id = $1::text::uuid)",
            &[&build_id],
        )
        .await?;
    Ok((row.get(0), row.get(1), row.get(2)))
}

async fn physical_generation_state(
    psql: &Path,
    socket_dir: &Path,
    node: &Node,
    build_id: &str,
) -> Result<String> {
    let state = capture_psql(
        psql,
        socket_dir,
        node.port,
        &format!(
            "SELECT state FROM ec_distann_generation
              WHERE index_oid = 'public.dm_idx'::regclass::oid
                AND build_id = '{build_id}'::uuid"
        ),
    )
    .await?;
    Ok(state.trim().to_owned())
}

async fn restart_physical_node(
    pg_ctl: &Path,
    socket_dir: &Path,
    node: &Node,
    nodes: &[Node],
) -> Result<()> {
    let mut restart = Command::new(pg_ctl);
    restart
        .arg("-w")
        .arg("-D")
        .arg(&node.data_dir)
        .arg("-l")
        .arg(&node.log_file)
        .arg("-o")
        .arg(format!(
            "-p {} -c listen_addresses=127.0.0.1 -c unix_socket_directories='' \
             -c shared_preload_libraries=ecaz -c max_prepared_transactions=32",
            node.port
        ))
        .arg("start")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    for target in nodes {
        restart.env(
            format!("EC_SPIRE_REMOTE_CONNINFO_DISTANN_NODE_{}", target.node_id),
            conninfo(socket_dir, target.port),
        );
    }
    run_status(restart)
        .await
        .wrap_err_with(|| format!("restarting physical owner {}", node.node_id))
}

async fn drive_fixture(
    args: &LocalMultinodePg18Args,
    pg_ctl: &Path,
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    log_dir: &Path,
) -> Result<()> {
    // Replicated deterministic corpus + index on every node.
    let setup = build_setup_sql(args)?;
    for node in nodes {
        run_psql_file(psql, socket_dir, node.port, &setup)
            .await
            .wrap_err_with(|| format!("corpus/index setup on node {}", node.node_id))?;
        crate::ecaz_println!(
            "[distann-multicluster] node {} loaded + indexed",
            node.node_id
        );
    }

    // Coordinator roster: every node by socket conninfo, in node-id order.
    let roster = nodes
        .iter()
        .map(|node| format!("{}@{}", node.node_id, conninfo(socket_dir, node.port)))
        .collect::<Vec<_>>()
        .join(";");

    // Distinct-recall gate on the coordinator (node 1).
    let coord_port = nodes[0].port;
    let recall = recall_sql(
        &roster,
        args.queries,
        args.top_k,
        args.corpus_prefix.is_some(),
    );
    let out = capture_psql(psql, socket_dir, coord_port, &recall)
        .await
        .wrap_err("running the multi-node recall comparison")?;
    let result_line = out
        .lines()
        .find(|line| line.contains("RECALL_RESULT"))
        .unwrap_or("RECALL_RESULT <none>")
        .trim()
        .to_owned();
    crate::ecaz_println!("[distann-multicluster] {result_line}");

    // Suite-driven recall gate (006-P1 letter): `ecaz bench recall` against the
    // coordinator single-node vs multi-node, distinct_recall(multi) >=
    // distinct_recall(single) - 0.001. Run here — before the mutating drills —
    // so benchgate_corpus is byte-identical across nodes (consistent vec_ids).
    // The byte-identical top-k gate above is strictly stronger.
    let suite_line = suite_recall_gate(psql, socket_dir, nodes, &roster, args).await;
    crate::ecaz_println!("[distann-multicluster] {suite_line}");
    // 019-P1: a genuine recall regression fails the run (SKIPPED/INCONCLUSIVE are
    // environment issues, not gate failures).
    if suite_line.contains("pass=false") {
        bail!("suite recall gate FAILED: {suite_line}");
    }

    // Qual correctness (011/020-P1): a WHERE predicate on a NON-projected column
    // plus LIMIT. Multi-node must match single-node exactly — this exercises
    // shipping the qual column (source) for remote rows and over-fetching so the
    // LIMIT applies after the qual. Runs early, on the clean/consistent corpus.
    let (qual_line, qual_ok) =
        qual_correctness_drill(psql, socket_dir, coord_port, &roster, args).await;
    crate::ecaz_println!("[distann-multicluster] {qual_line}");

    // FR-082 published-epoch read consumption: reads must source the epoch from
    // the persisted manifest (`active_epoch`), not the session GUC. Proven by a
    // coordinator-only publish (breaks scans via fingerprint mismatch — only
    // possible if reads consume active_epoch) vs a coordinated all-node publish
    // (swaps the epoch; scans match the baseline again).
    let (fr082_line, fr082_ok) =
        fr082_published_epoch_drill(psql, socket_dir, nodes, &roster, args).await;
    crate::ecaz_println!("[distann-multicluster] {fr082_line}");

    // Cluster storage summation (Task 172 AC-3 / NFR-018): per-node and summed
    // index+heap bytes across every participant, plus the replicated-index space
    // amplification vs raw f32 vectors. Runs in both modes so the suite always
    // captures it.
    let storage_lines = storage_summation(psql, socket_dir, nodes, args).await;
    for line in &storage_lines {
        crate::ecaz_println!("[distann-multicluster] {line}");
    }

    // Recall-only mode (the `distann-local-multinode` suite step): the multi-node
    // distinct-recall gates above are the scaled evidence; skip the (expensive at
    // scale) TC-042 fault matrix + FR-082 lifecycle drills, which are proven
    // scale-independently at the fixture default size.
    if args.skip_fault_drills {
        // 172-P1: report MEASURED corpus metadata, not the synthetic args
        // defaults. In real mode args.rows/args.dim are meaningless (16/2000);
        // the true rows/dim come from the storage summation over the loaded data.
        let (measured_rows, measured_dim) = storage_lines
            .iter()
            .find(|l| l.starts_with("storage_summation"))
            .map(|l| {
                let field = |k: &str| {
                    l.split_whitespace()
                        .find_map(|t| t.strip_prefix(k))
                        .unwrap_or("?")
                        .to_owned()
                };
                (field("corpus_rows="), field("dim="))
            })
            .unwrap_or_else(|| (args.rows.to_string(), args.dim.to_string()));
        let corpus_label = match &args.corpus_prefix {
            Some(prefix) => format!("corpus=real staged prefix={prefix}"),
            None => "corpus=synthetic".to_owned(),
        };
        let mut summary = format!(
            "distann-multinode fixture (recall-only)\n{corpus_label}\nnodes={}\nrows={measured_rows}\ndim={measured_dim}\ngraph_degree={}\nhead_index_cap={}\nqueries={}\ntop_k={}\nroster={}\n{}\n",
            args.nodes,
            args.graph_degree,
            args.head_index_cap,
            args.queries,
            args.top_k,
            roster,
            result_line
        );
        summary.push_str(&format!("{qual_line}\n"));
        summary.push_str(&format!("{fr082_line}\n"));
        summary.push_str(&format!("{suite_line}\n"));
        for line in &storage_lines {
            summary.push_str(&format!("{line}\n"));
        }
        let summary_path = log_dir.join("distann-multinode-summary.log");
        fs::write(&summary_path, &summary)
            .wrap_err_with(|| format!("writing {}", summary_path.display()))?;
        crate::ecaz_println!(
            "[distann-multicluster] summary written to {}",
            summary_path.display()
        );
        if !result_line.contains("mismatched_ids=0") {
            bail!("multi-node distinct-recall gate FAILED: {result_line}");
        }
        if !qual_ok {
            bail!("qual correctness FAILED: {qual_line}");
        }
        if !fr082_ok {
            bail!("FR-082 published-epoch read consumption FAILED: {fr082_line}");
        }
        crate::ecaz_println!(
            "[distann-multicluster] GATE PASS (recall-only): multi-node distinct-recall identical to single-node"
        );
        return Ok(());
    }

    // TC-042 fault matrix (NFR-020): each fault must make the multi-node query
    // ERROR (fail closed) — never a silent wrong or partial result — and a
    // post-recovery query must match the baseline (no false reject).
    let mut drills: Vec<(String, bool)> = Vec::new();
    let last = nodes.last().unwrap();
    let single_query = format!(
        "SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {};",
        args.top_k
    );

    // 1. simulated_network_partition: one owner at a dead port ⇒ connect error.
    {
        let dead_roster = roster_with_port_override(nodes, socket_dir, last.node_id, 1);
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{dead_roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push((
            "simulated_network_partition".to_owned(),
            query_errored(&out),
        ));
    }

    // 2. epoch_bump_no_false_reject: a bare epoch-number bump must NOT reject —
    // the FR-082 fingerprint is content-based and the coordinator propagates its
    // epoch to owners, so both sides agree and the query returns its result.
    {
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=999999; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        // Pass = no false reject: no error AND a result row came back.
        drills.push((
            "epoch_bump_no_false_reject".to_owned(),
            !query_errored(&out) && out.contains('\n'),
        ));
    }

    // 3. remote_content_divergence (real epoch/fingerprint mismatch): rebuild an
    // owner's index with a different graph_degree so its content fingerprint no
    // longer matches the coordinator's ⇒ the owner rejects the epoch (error).
    // Diverge DOWNWARD (graph_degree - 8): a larger degree can overflow the
    // ec_distann node-record page budget on high-dim corpora (a 1536-dim
    // co-placed vector + graph_degree neighbor codes must fit one 8 KB page), so
    // graph_degree + 8 fails ambuild at real scale. A smaller degree is always
    // within the budget the base build already satisfied and still changes the
    // content fingerprint.
    let divergent_degree = args.graph_degree.saturating_sub(8).max(4);
    {
        run_psql_file(
            psql,
            socket_dir,
            last.port,
            &format!(
                "DROP INDEX dm_idx; CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {divergent_degree});",
            ),
        )
        .await
        .wrap_err("diverging remote index content for the drill")?;
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("remote_content_divergence".to_owned(), query_errored(&out)));
        run_psql_file(
            psql,
            socket_dir,
            last.port,
            &format!(
                "DROP INDEX dm_idx; CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {});",
                args.graph_degree
            ),
        )
        .await
        .wrap_err("restoring remote index content after the drill")?;
    }

    // 3. missing_or_reindexed_remote_index: drop the index on an owner ⇒ error.
    {
        run_psql_file(psql, socket_dir, last.port, "DROP INDEX dm_idx;")
            .await
            .wrap_err("dropping remote index for the drill")?;
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push((
            "missing_or_reindexed_remote_index".to_owned(),
            query_errored(&out),
        ));
        // Rebuild for recovery.
        run_psql_file(
            psql,
            socket_dir,
            last.port,
            &format!(
                "CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {});",
                args.graph_degree
            ),
        )
        .await
        .wrap_err("rebuilding remote index after the drill")?;
    }

    // 4. remote_backend_termination / instance down: stop an owner ⇒ error.
    {
        let _ = Command::new(pg_ctl)
            .arg("-D")
            .arg(&last.data_dir)
            .arg("-m")
            .arg("fast")
            .arg("stop")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("remote_backend_termination".to_owned(), query_errored(&out)));
        // Restart for recovery.
        let mut restart = Command::new(pg_ctl);
        restart
            .arg("-w")
            .arg("-D")
            .arg(&last.data_dir)
            .arg("-l")
            .arg(&last.log_file)
            .arg("-o")
            .arg(format!(
                "-p {} -k {} -c listen_addresses=''",
                last.port,
                socket_dir.display()
            ))
            .arg("start")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        run_status(restart)
            .await
            .wrap_err("restarting owner after the drill")?;
    }

    // 6. placement_drift: coordinator local_node_id absent from the roster ⇒ no
    // local node ⇒ error (a placement disagreement is never a silent miss).
    {
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=99; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("placement_drift".to_owned(), query_errored(&out)));
    }

    // 7. remote_statement_timeout: inject `statement_timeout=1` (1 ms) into one
    // owner's conninfo ⇒ its expand statement is cancelled server-side ⇒ the
    // coordinator surfaces the remote error rather than a partial result.
    {
        let timeout_roster = roster_with_conninfo_suffix(
            nodes,
            socket_dir,
            last.node_id,
            "options=-cstatement_timeout=1",
        );
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{timeout_roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("remote_statement_timeout".to_owned(), query_errored(&out)));
    }

    // 7b. hop_round_failure_mid_beam: force the search past round 0 (a high top_k
    // bar prevents the round-0 convergence early-exit) and inject a failure at the
    // start of hop round 1 via `ec_distann.debug_fail_hop_round`. A mid-beam round
    // failure must discard the partial beam and ERROR — never surface round 0's
    // partial frontier as a complete result.
    {
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
             SET ec_distann.hop_rounds=4; SET ec_distann.top_k=200; SET ec_distann.beam_width=8; \
             SET ec_distann.debug_fail_hop_round=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        // Fail closed AND specifically the injected mid-beam (round 1) failure.
        let mid_beam = query_errored(&out) && out.contains("round 1");
        crate::ecaz_println!(
            "[distann-multicluster] hop_round_failure_mid_beam DIAG errored={} mid_beam={mid_beam}",
            query_errored(&out)
        );
        drills.push(("hop_round_failure_mid_beam".to_owned(), mid_beam));
    }

    // 7c. missing_node_record (FR-079 case c): force the local expander to report
    // an owned record as absent from its directory. The scan must raise the
    // structural fault, never silently under-return.
    {
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
             SET ec_distann.debug_missing_node_record=true; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        let pass = query_errored(&out) && out.contains("missing node record");
        crate::ecaz_println!(
            "[distann-multicluster] missing_node_record DIAG errored={} tagged={pass}",
            query_errored(&out)
        );
        drills.push(("missing_node_record".to_owned(), pass));
    }

    // 8. missing_heap_row_co_placement_drift (also the partial mid-delete case):
    // remove only an owned record's heap row on its owner, leaving the index
    // record ⇒ the owner's exact rerank fails `[EC_VECTOR_MISSING]` ⇒ error. The
    // correct in-epoch delete is a monotonic tombstone via FR-083's
    // `ec_distann_apply_record_writes` (which keeps the frozen vector, per
    // FR-082-AC-5); this drill proves the *drift* hazard fails closed rather than
    // silently dropping the row. The drill self-recovers by re-running setup on
    // the owner.
    {
        let drift_ok =
            co_placement_drift_drill(psql, socket_dir, coord_port, &roster, nodes, args).await;
        drills.push(("missing_heap_row_co_placement_drift".to_owned(), drift_ok));
    }

    // 8a. mid-delete / lost-tombstone-write (NFR-020): a tombstone write that
    // errors after the WAL-logged flag flip. The monotonic tombstone stays set
    // (PG does not undo index-page writes on abort), so the record is deleted and
    // STAYS deleted — the caller sees an error but the row never resurrects.
    {
        let mid_delete_ok = mid_delete_drill(psql, socket_dir, coord_port, args).await;
        drills.push((
            "mid_delete_lost_tombstone_no_resurrect".to_owned(),
            mid_delete_ok,
        ));
    }

    // 8b. mid-insert failure (FR-083 fold path, TC-043): a graph insert that fails
    // after staging pages but before publishing metadata must roll back cleanly —
    // no partial record visible. Runs on an isolated table so shared `dm` is
    // untouched.
    {
        let mid_insert_ok = mid_insert_drill(psql, socket_dir, coord_port, args).await;
        drills.push(("mid_insert_failure_rolls_back".to_owned(), mid_insert_ok));
    }

    // 7. concurrency (FR-082-AC-4): run many multi-node scans concurrently with a
    // background inserter mutating the coordinator's table. Every scan must
    // complete (return only expanded records; never a torn/half-applied read that
    // errors). A single failing session fails the drill.
    let concurrency_ok =
        concurrency_drill(psql, socket_dir, coord_port, nodes, &roster, args).await?;
    crate::ecaz_println!(
        "[distann-multicluster] concurrency_scan_insert_epochswap pass={concurrency_ok}"
    );

    // 7b. live retention gate (FR-082-AC-3): a scan held open (AccessShareLock)
    // must block retire; once it drains, retire succeeds.
    let retention_ok = retention_gate_drill(psql, socket_dir, coord_port, args).await;
    crate::ecaz_println!("[distann-multicluster] live_retention_gate pass={retention_ok}");

    // 7c. AC-5 frozen vec_id→vector: a live record's exact-rerank result must be
    // byte-identical after real delete+VACUUM+reinsert TID churn on every node
    // (the AM's ambulkdelete tombstones deleted records so they are never
    // reranked, and a live record's heap TID is never reclaimed → its vector is
    // frozen without a separate tier, under D10).
    let frozen_ok = frozen_vector_drill(psql, socket_dir, coord_port, &roster, nodes, args).await;
    crate::ecaz_println!(
        "[distann-multicluster] ac5_frozen_vector_after_vacuum_reuse pass={frozen_ok}"
    );

    // 8. recovery / no-false-reject: after all faults clear, the full-roster
    // query must match the single-node baseline again.
    let recovery = capture_psql(
        psql,
        socket_dir,
        coord_port,
        &recall_sql(
            &roster,
            args.queries,
            args.top_k,
            args.corpus_prefix.is_some(),
        ),
    )
    .await
    .wrap_err("running the post-recovery recall comparison")?;
    let recovery_line = recovery
        .lines()
        .find(|line| line.contains("RECALL_RESULT"))
        .unwrap_or("RECALL_RESULT <none>")
        .trim()
        .to_owned();
    let recovered = recovery_line.contains("mismatched_ids=0");

    for (name, fail_closed) in &drills {
        crate::ecaz_println!("[distann-multicluster] fault_drill {name} pass={fail_closed}");
    }
    crate::ecaz_println!("[distann-multicluster] recovery {recovery_line} recovered={recovered}");

    // Disjoint-shard demonstration (destructive — prunes to owned shards; runs
    // last, after the replicated-corpus recovery check).
    let (disjoint_line, disjoint_ok) =
        disjoint_shard_drill(psql, socket_dir, nodes, &roster, args).await;
    crate::ecaz_println!("[distann-multicluster] {disjoint_line}");

    // Persist the evidence.
    let mut summary = format!(
        "distann-multinode fixture\nnodes={}\nrows={}\ndim={}\ngraph_degree={}\nqueries={}\ntop_k={}\nroster={}\n{}\n",
        args.nodes, args.rows, args.dim, args.graph_degree, args.queries, args.top_k, roster, result_line
    );
    for (name, fail_closed) in &drills {
        summary.push_str(&format!("fault_drill {name} pass={fail_closed}\n"));
    }
    summary.push_str(&format!(
        "concurrency_scan_insert_epochswap pass={concurrency_ok}\n"
    ));
    summary.push_str(&format!("{qual_line}\n"));
    summary.push_str(&format!("{fr082_line}\n"));
    summary.push_str(&format!("live_retention_gate pass={retention_ok}\n"));
    summary.push_str(&format!(
        "ac5_frozen_vector_after_vacuum_reuse pass={frozen_ok}\n"
    ));
    summary.push_str(&format!("{suite_line}\n"));
    summary.push_str(&format!("recovery {recovery_line} recovered={recovered}\n"));
    summary.push_str(&format!("{disjoint_line}\n"));
    let summary_path = log_dir.join("distann-multinode-summary.log");
    fs::write(&summary_path, &summary)
        .wrap_err_with(|| format!("writing {}", summary_path.display()))?;
    crate::ecaz_println!(
        "[distann-multicluster] summary written to {}",
        summary_path.display()
    );

    if !result_line.contains("mismatched_ids=0") {
        bail!("multi-node distinct-recall gate FAILED: {result_line}");
    }
    if !concurrency_ok {
        bail!("concurrency drill FAILED: a scan errored under concurrent insert load");
    }
    if !qual_ok {
        bail!("qual correctness FAILED: multi-node WHERE+LIMIT result differs from single-node");
    }
    if !fr082_ok {
        bail!("FR-082 published-epoch read consumption FAILED: {fr082_line}");
    }
    if !retention_ok {
        bail!("live retention gate FAILED: retire not gated by an in-flight scan, or blocked after drain");
    }
    if !frozen_ok {
        bail!("AC-5 FAILED: a live record's rerank changed after delete+VACUUM+reinsert TID churn");
    }
    let all_fail_closed = drills.iter().all(|(_, ok)| *ok);
    if !all_fail_closed {
        let failed: Vec<&str> = drills
            .iter()
            .filter(|(_, ok)| !*ok)
            .map(|(name, _)| name.as_str())
            .collect();
        bail!("TC-042 fault matrix FAILED (not fail-closed): {failed:?}");
    }
    if !recovered {
        bail!("recovery FAILED: post-fault query did not match baseline: {recovery_line}");
    }
    if !disjoint_ok {
        bail!("disjoint-shard FAILED: multi-node result changed after pruning to owned shards");
    }
    crate::ecaz_println!(
        "[distann-multicluster] GATE PASS: recall identical; {} faults fail-closed; recovery clean",
        drills.len()
    );
    Ok(())
}

fn roster_with_port_override(
    nodes: &[Node],
    socket_dir: &Path,
    override_node_id: u32,
    override_port: u16,
) -> String {
    nodes
        .iter()
        .map(|node| {
            let port = if node.node_id == override_node_id {
                override_port
            } else {
                node.port
            };
            format!("{}@{}", node.node_id, conninfo(socket_dir, port))
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// A roster where `override_node_id`'s conninfo carries an extra libpq keyword
/// (space-separated, matching the `host=… port=…` conninfo shape). Used to inject
/// `options=-cstatement_timeout=1` into a single owner for the
/// remote_statement_timeout fault drill.
fn roster_with_conninfo_suffix(
    nodes: &[Node],
    socket_dir: &Path,
    override_node_id: u32,
    suffix: &str,
) -> String {
    nodes
        .iter()
        .map(|node| {
            let base = conninfo(socket_dir, node.port);
            if node.node_id == override_node_id {
                format!("{}@{} {}", node.node_id, base, suffix)
            } else {
                format!("{}@{}", node.node_id, base)
            }
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// NFR-020 co-placement drift / missing-heap-row (and partial mid-delete) drill.
///
/// In the replicated topology every node holds every heap row, so a serving node
/// reranks from its OWN heap copy — deleting a row on a single owner is masked by
/// the other replicas (proven: a single-node delete still returned the row with
/// no error). Genuine cluster-wide co-placement drift is: the index record
/// survives on every node but its co-placed heap row is gone everywhere. This
/// drill deletes a record's heap row on ALL nodes (leaving the index record),
/// then runs a query anchored on that record's own vector. Every serving node's
/// exact rerank must fetch the (now invisible) heap tuple and fail
/// `[EC_VECTOR_MISSING]`, so the multi-node query ERRORs (fail closed) rather than
/// silently dropping or mis-ranking the true top-1. Recovery re-runs the
/// deterministic setup on every node (identical vec_ids), so the post-fault recall
/// baseline still matches.
async fn co_placement_drift_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster: &str,
    nodes: &[Node],
    args: &LocalMultinodePg18Args,
) -> bool {
    // Exercise both ownership arms: a coordinator-owned record (local MVCC skip →
    // correct-complete) and a remote-owned record (remote rerank → structural
    // fault). Both must satisfy the NFR-020 disjunction.
    let coord = co_placement_drift_case(psql, socket_dir, coord_port, roster, nodes, args, 0).await;
    let remote = co_placement_drift_case(
        psql,
        socket_dir,
        coord_port,
        roster,
        nodes,
        args,
        args.nodes - 1,
    )
    .await;
    coord && remote
}

/// FR-082 published-epoch read-consumption drill. Reads must source the scan
/// epoch from the persisted manifest (`active_epoch`), not the session GUC — so a
/// `publish` actually changes what queries see. Proven in three steps against the
/// replicated `dm`:
///
///   A. baseline multi-node scan at the built-in published epoch (1) succeeds;
///   B. publish epoch 2 on the COORDINATOR ONLY ⇒ its fingerprint no longer
///      matches the owners' (still epoch 1) ⇒ the scan ERRORS. This can only
///      happen if reads consume `active_epoch` (the GUC is unchanged throughout);
///   C. publish epoch 2 on EVERY node ⇒ the epoch swaps atomically and the scan
///      succeeds again with the same top-k as the baseline.
///
/// Restores epoch 1 on all nodes so later drills see the default state. Returns
/// (summary, pass).
async fn fr082_published_epoch_drill(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> (String, bool) {
    let coord_port = nodes[0].port;
    // Note: no `ec_distann.epoch` is set — reads must ignore the GUC and use the
    // published manifest epoch.
    let scan = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; \
         SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {};",
        args.top_k
    );
    let ids = |out: &str| -> Vec<i64> {
        let mut v: Vec<i64> = out.lines().filter_map(|l| l.trim().parse().ok()).collect();
        v.sort_unstable();
        v
    };
    let publish = |port: u16, epoch: i64| {
        let sql = format!("SELECT ec_distann_publish_epoch('dm_idx'::regclass, {epoch});");
        async move { run_psql_file(psql, socket_dir, port, &sql).await }
    };

    // A. baseline at the built-in published epoch.
    let base_out = capture_psql_allow_error(psql, socket_dir, coord_port, &scan).await;
    let base_ids = ids(&base_out);
    let base_ok = !query_errored(&base_out) && !base_ids.is_empty();

    // B. coordinator-only publish of epoch 2 ⇒ mismatch ⇒ scan errors.
    let _ = publish(coord_port, 2).await;
    let skew_out = capture_psql_allow_error(psql, socket_dir, coord_port, &scan).await;
    let skew_errored = query_errored(&skew_out);

    // C. publish epoch 2 on every node ⇒ swap ⇒ scan matches the baseline.
    for node in nodes {
        let _ = publish(node.port, 2).await;
    }
    let swap_out = capture_psql_allow_error(psql, socket_dir, coord_port, &scan).await;
    let swap_ids = ids(&swap_out);
    let swap_ok = !query_errored(&swap_out) && swap_ids == base_ids;

    // Restore epoch 1 on every node for the later drills.
    for node in nodes {
        let _ = publish(node.port, 1).await;
    }

    let pass = base_ok && skew_errored && swap_ok;
    (
        format!(
            "fr082_published_epoch base_ok={base_ok} coord_only_publish_errored={skew_errored} \
             all_publish_swap_ok={swap_ok} pass={pass}"
        ),
        pass,
    )
}

/// NFR-020 mid-delete / lost-tombstone-write drill: attempt a tombstone write via
/// the FR-083 owner endpoint (`ec_distann_apply_record_writes`) with
/// `ec_distann.debug_fail_tombstone_write` on — the endpoint WAL-logs the flag
/// flip, then errors.
///
/// NFR-020 requires that a lost remote tombstone write "must error, never
/// silently resurrect the row." The tombstone flag is a MONOTONIC set (dml.rs),
/// and PostgreSQL does not physically undo WAL-logged index-page changes on a
/// transaction abort — so the flag stays set: the record is deleted and STAYS
/// deleted (the safe, non-resurrecting direction), while the caller still sees an
/// error. This drill asserts exactly that: the write errors AND the record is
/// tombstoned and remains tombstoned across re-reads (monotonic, no resurrection)
/// AND an ANN scan excludes it. Runs on an isolated table so `dm` is untouched.
/// Returns true iff errored AND tombstoned-and-stable AND excluded from scans.
async fn mid_delete_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    args: &LocalMultinodePg18Args,
) -> bool {
    let single = "SELECT set_config('ec_distann.roster','',false); SET ec_distann.local_node_id=1; SET ec_distann.epoch=0;";
    let dim = args.dim;
    let gvec = format!(
        "encode_to_ecvector((SELECT array_agg((sin(g * 0.017 * (d + 1)) + cos(g * 0.0031 * (d + 1)))::real) \
           FROM generate_series(0, {dim} - 1) AS d), 4, 42)"
    );
    let setup = format!(
        "DROP TABLE IF EXISTS md; CREATE TABLE md (id bigint, embedding ecvector); \
         INSERT INTO md SELECT g, {gvec} FROM generate_series(1, 500) AS g; \
         CREATE INDEX md_idx ON md USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {gd});",
        gd = args.graph_degree,
    );
    if run_psql_file(psql, socket_dir, coord_port, &setup)
        .await
        .is_err()
    {
        return false;
    }
    // Discover a live owned vec_id + its id (to check scan exclusion).
    let discover = format!(
        "{single} SELECT d.vec_id || '|' || t.id \
           FROM ec_distann_list_directory('md_idx'::regclass) d \
           JOIN md t ON t.ctid = ('(' || d.heap_block || ',' || d.heap_offset || ')')::tid \
          WHERE NOT d.is_tombstone ORDER BY t.id LIMIT 1;"
    );
    let discovered = capture_psql_allow_error(psql, socket_dir, coord_port, &discover).await;
    let Some((vec_id, id)) = discovered
        .lines()
        .find_map(|l| l.trim().split_once('|'))
        .filter(|(v, i)| v.parse::<i64>().is_ok() && i.parse::<i64>().is_ok())
        .map(|(v, i)| (v.to_owned(), i.to_owned()))
    else {
        crate::ecaz_println!(
            "[distann-multicluster] mid_delete: no live vec_id discovered (skipped)"
        );
        let _ = run_psql_file(psql, socket_dir, coord_port, "DROP TABLE IF EXISTS md;").await;
        return false;
    };
    // Attempt the tombstone write with injection: must error.
    let attempt = format!(
        "{single} SET ec_distann.debug_fail_tombstone_write=true; \
         SELECT ec_distann_apply_record_writes('md_idx'::regclass, ec_distann_epoch_fingerprint('md_idx'::regclass), ARRAY[{vec_id}]::bigint[]);"
    );
    let attempt_out = capture_psql_allow_error(psql, socket_dir, coord_port, &attempt).await;
    let errored = query_errored(&attempt_out);
    // Re-read is_tombstone twice: monotonic ⇒ tombstoned and stable (no resurrection).
    let tomb = format!(
        "{single} SELECT is_tombstone FROM ec_distann_list_directory('md_idx'::regclass) WHERE vec_id={vec_id};"
    );
    let t1 = capture_psql_allow_error(psql, socket_dir, coord_port, &tomb).await;
    let t2 = capture_psql_allow_error(psql, socket_dir, coord_port, &tomb).await;
    let tombstoned = |o: &str| o.lines().any(|l| l.trim() == "t");
    let stable_tombstoned = tombstoned(&t1) && tombstoned(&t2);
    // And the ANN scan excludes the now-tombstoned record (deleted, not resurrected).
    let scan = format!(
        "{single} SET enable_seqscan=off; \
         SELECT id FROM md ORDER BY embedding <#> (SELECT embedding FROM md WHERE id={id}) LIMIT 10;"
    );
    let scan_out = capture_psql_allow_error(psql, socket_dir, coord_port, &scan).await;
    let excluded = !scan_out.lines().any(|l| l.trim() == id);
    let pass = errored && stable_tombstoned && excluded;
    crate::ecaz_println!(
        "[distann-multicluster] mid_delete_lost_tombstone DIAG vec_id={vec_id} id={id} errored={errored} \
         stable_tombstoned={stable_tombstoned} excluded_from_scan={excluded} pass={pass}"
    );
    let _ = run_psql_file(psql, socket_dir, coord_port, "DROP TABLE IF EXISTS md;").await;
    pass
}

/// FR-083 mid-insert failure drill (TC-043), on an isolated one-owner physical
/// generation so the shared `dm` fixture remains untouched. The injected
/// failure occurs after the physical graph append (and after any owner-side
/// prepared write) but before backlink publication. The source row count and
/// published physical record count must remain unchanged after the abort.
async fn mid_insert_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    args: &LocalMultinodePg18Args,
) -> bool {
    let dim = args.dim;
    let setup_source = synthetic_unit_vector_expr("g", dim);
    let setup = format!(
        "DROP TABLE IF EXISTS mi CASCADE; \
         CREATE TABLE mi (id bigint, source_id uuid NOT NULL, source real[], embedding ecvector({dim})); \
         INSERT INTO mi \
         SELECT g, (substr(md5(g::text),1,8)||'-'||substr(md5(g::text),9,4)||'-4'||\
                    substr(md5(g::text),14,3)||'-8'||substr(md5(g::text),18,3)||'-'||\
                    substr(md5(g::text),21,12))::uuid, arr, encode_to_ecvector(arr, 4, 42) \
           FROM (SELECT g, {setup_source} AS arr \
                   FROM generate_series(1, 500) g) rows; \
         CREATE INDEX mi_idx ON mi USING ec_distann (embedding ecvector_distann_ip_ops)\
           INCLUDE (source_id) WITH (distributed_control = true, source_identity = 'include', graph_degree = {gd}); \
         SELECT ec_distann_configure_participant_identity('mi_idx'::regclass, 'mi/node-1'); \
         SELECT ec_distann_register_node_descriptor('mi_idx'::regclass, 0, 1, 'mi/node-1',\
                'DISTANN_NODE_1', 'public.mi_idx', true);",
        gd = args.graph_degree,
    );
    if run_psql_file(psql, socket_dir, coord_port, &setup)
        .await
        .is_err()
    {
        return false;
    }
    for lifecycle_sql in [
        "SELECT ec_distann_begin_epoch_build('mi_idx'::regclass, 1, '81818181-8181-4181-8181-818181818181'::uuid); \
         SELECT ec_distann_build_epoch('mi_idx'::regclass, 1, '81818181-8181-4181-8181-818181818181'::uuid);",
        "SELECT ec_distann_decide_epoch_publish('mi_idx'::regclass, '81818181-8181-4181-8181-818181818181'::uuid);",
        "SELECT ec_distann_recover_epoch_publish('mi_idx'::regclass, '81818181-8181-4181-8181-818181818181'::uuid);",
    ] {
        if run_psql_file(psql, socket_dir, coord_port, lifecycle_sql)
            .await
            .is_err()
        {
            let _ = run_psql_file(psql, socket_dir, coord_port, "DROP TABLE IF EXISTS mi CASCADE;").await;
            return false;
        }
    }
    let topology_sql =
        "SELECT count(*) FROM mi; SELECT record_count FROM ec_distann_epoch_topology(\
                        'mi_idx'::regclass, (SELECT epoch_fingerprint FROM ec_distann_active_epoch \
                        WHERE index_oid='mi_idx'::regclass::oid));";
    let before = capture_psql_allow_error(psql, socket_dir, coord_port, topology_sql).await;
    let before_values = before
        .lines()
        .filter_map(|line| line.trim().parse::<i64>().ok())
        .collect::<Vec<_>>();
    if before_values.len() < 2 {
        crate::ecaz_eprintln!(
            "[distann-multicluster] mid_insert_failure DIAG before_probe_output={:?}",
            before
        );
        let _ = run_psql_file(
            psql,
            socket_dir,
            coord_port,
            "DROP TABLE IF EXISTS mi CASCADE;",
        )
        .await;
        return false;
    }
    let failed_source = synthetic_unit_vector_expr("501", dim);
    let insert = format!(
        "SET ec_distann.debug_fail_insert=true; INSERT INTO mi VALUES (501,\
         '00000000-0000-4000-8000-000000000501',\
         {failed_source}, encode_to_ecvector({failed_source}, 4, 42));",
    );
    let insert_out = capture_psql_allow_error(psql, socket_dir, coord_port, &insert).await;
    let insert_errored = query_errored(&insert_out);
    let after = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        "RESET ec_distann.debug_fail_insert; SELECT count(*) FROM mi; SELECT record_count FROM ec_distann_epoch_topology(\
         'mi_idx'::regclass, (SELECT epoch_fingerprint FROM ec_distann_active_epoch \
         WHERE index_oid='mi_idx'::regclass::oid));",
    )
    .await;
    let after_values = after
        .lines()
        .filter_map(|line| line.trim().parse::<i64>().ok())
        .collect::<Vec<_>>();
    let consistent = after_values.len() >= 2 && after_values[..2] == before_values[..2];
    let pass = insert_errored && consistent;
    crate::ecaz_println!(
        "[distann-multicluster] mid_insert_failure DIAG physical_insert_errored={insert_errored} \
         before_rows={} after_rows={} before_records={} after_records={} consistent={consistent} pass={pass}",
        before_values[0],
        after_values.first().copied().unwrap_or(-1),
        before_values[1],
        after_values.get(1).copied().unwrap_or(-1),
    );
    // Exercise the committed UPDATE contract on the same isolated published
    // generation. The index AM must preserve the source-derived vec_id while
    // appending a complete replacement graph/row pair and retiring only the
    // prior graph version. Resolve the owner-local relation names from the
    // generation catalog instead of guessing generated identifiers.
    let source_id_output = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        "SELECT source_id::text FROM mi WHERE id=1;",
    )
    .await;
    let source_id = source_id_output
        .lines()
        .map(str::trim)
        .find(|line| line.len() == 36 && line.bytes().filter(|byte| *byte == b'-').count() == 4)
        .unwrap_or("");
    let relation_output = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        "SELECT graph_store_relid::regclass::text || '|' || row_tier_relid::regclass::text \
           FROM ec_distann_generation \
          WHERE index_oid='mi_idx'::regclass::oid AND state='Published' \
          ORDER BY epoch DESC LIMIT 1;",
    )
    .await;
    let relation_names = relation_output
        .lines()
        .map(str::trim)
        .find_map(|line| line.split_once('|'));
    let safe_relation = |relation: &str| {
        !relation.is_empty()
            && relation
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'"'))
    };
    let update_probe = if let Some(identity) = source_identity_uuid_bytes(source_id).ok() {
        if let Some((graph_relation, row_relation)) = relation_names {
            if safe_relation(graph_relation) && safe_relation(row_relation) {
                let vec_id = distann_vec_id_from_source_identity(&identity);
                let version_probe = || {
                    format!(
                        "SELECT vec_id::text || '|' || count(*)::text || '|' || \
                                count(*) FILTER (WHERE g.is_current)::text \
                           FROM {graph_relation} g \
                          WHERE vec_id = {vec_id} \
                          GROUP BY vec_id;"
                    )
                };
                let before_update =
                    capture_psql_allow_error(psql, socket_dir, coord_port, &version_probe()).await;
                let update_source = synthetic_unit_vector_expr("1001", dim);
                let update_sql = format!(
                    "UPDATE mi SET source={update_source}, \
                     embedding=encode_to_ecvector({update_source}, 4, 42) WHERE id=1;"
                );
                let update_output =
                    capture_psql_allow_error(psql, socket_dir, coord_port, &update_sql).await;
                let after_update =
                    capture_psql_allow_error(psql, socket_dir, coord_port, &version_probe()).await;
                let decode_version = |output: &str| {
                    output.lines().find_map(|line| {
                        let fields = line.trim().split('|').collect::<Vec<_>>();
                        if fields.len() == 3 {
                            Some((
                                fields[0].parse::<i64>().ok()?,
                                fields[1].parse::<i64>().ok()?,
                                fields[2].parse::<i64>().ok()?,
                            ))
                        } else {
                            None
                        }
                    })
                };
                let before_version = decode_version(&before_update);
                let after_version = decode_version(&after_update);
                let update_ok = !query_errored(&update_output);
                let stable_replacement = matches!(
                    (before_version, after_version),
                    (Some((before_vec_id, 1, 1)), Some((after_vec_id, 2, 1)))
                        if before_vec_id == after_vec_id
                );
                crate::ecaz_println!(
                    "[distann-multicluster] physical_update_replacement DIAG update_ok={update_ok} \
                     stable_vec_id={stable_replacement} before={before_version:?} after={after_version:?} pass={}",
                    update_ok && stable_replacement
                );
                update_ok && stable_replacement
            } else {
                crate::ecaz_eprintln!(
                    "[distann-multicluster] physical_update_replacement DIAG unsafe relation names: {:?}",
                    relation_names
                );
                false
            }
        } else {
            crate::ecaz_eprintln!(
                "[distann-multicluster] physical_update_replacement DIAG generation relation lookup failed: {}",
                compact_capture_error(&relation_output)
            );
            false
        }
    } else {
        crate::ecaz_eprintln!(
            "[distann-multicluster] physical_update_replacement DIAG source identity lookup failed: {}",
            compact_capture_error(&source_id_output)
        );
        false
    };
    let _ = run_psql_file(
        psql,
        socket_dir,
        coord_port,
        "DROP TABLE IF EXISTS mi CASCADE;",
    )
    .await;
    pass && update_probe
}

/// FR-083 concurrent insert/query drill (TC-043), against the published
/// physical generation used by the real fixture. Readers must continue to
/// return a complete top-k cardinality while coordinator-routed inserts append
/// complete source/row-tier records and owner graph records concurrently.
async fn physical_concurrency_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    args: &LocalMultinodePg18Args,
    nodes: &[Node],
    roster: &str,
    table: &str,
    epoch_fingerprint: &str,
) -> Result<bool> {
    const SCANNERS: usize = 4;
    const WRITERS: usize = 2;
    const ITERATIONS: usize = 12;
    let roster = task167_retry_attribution_roster(roster)?.replace('\'', "''");
    let query_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.debug_retry_attribution=on; \
         SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; \
         SELECT count(*) FROM (SELECT source_id FROM {table} \
          ORDER BY embedding <#> (SELECT source FROM {table} ORDER BY id LIMIT 1) \
          LIMIT {}) rows;",
        args.top_k
    );
    let insert_vector = insert_vector_expr(args, table);
    let source_rows_output = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT count(*) FROM {table}"),
    )
    .await;
    let source_rows = source_rows_output
        .lines()
        .find_map(|line| line.trim().parse::<i64>().ok())
        .unwrap_or(0);
    let expected_count = i64::from(args.top_k).min(source_rows);
    if expected_count == 0 {
        crate::ecaz_println!(
            "[distann-multicluster] physical_concurrent_insert_query DIAG role=source_rows table={table} output={}",
            compact_capture_error(&source_rows_output)
        );
        return Ok(false);
    }
    let base_rows = args.rows;
    let owner_nodes = if args.coordinator_outside_roster {
        &nodes[1..]
    } else {
        nodes
    };
    let dimension_output = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT array_length(source, 1) FROM {table} LIMIT 1;"),
    )
    .await;
    let Some(dimension) = dimension_output
        .lines()
        .find_map(|line| line.trim().parse::<usize>().ok())
    else {
        crate::ecaz_println!(
            "[distann-multicluster] physical_concurrent_insert_query DIAG role=shared_target reason=source_dimension_missing output={}",
            compact_capture_error(&dimension_output)
        );
        return Ok(false);
    };
    let zero_query = (0..dimension).map(|_| "0").collect::<Vec<_>>().join(",");
    let seed_id = 899_999_i64;
    let seed_insert_sql = format!(
        "SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; \
         WITH row_data AS (SELECT {seed_id}::bigint AS id, {insert_vector}::real[] AS source) \
         INSERT INTO {table} (id, source_id, source, embedding) \
         SELECT id, (substr(md5(id::text),1,8)||'-'||substr(md5(id::text),9,4)||'-4'||\
                substr(md5(id::text),14,3)||'-8'||substr(md5(id::text),18,3)||'-'||\
                substr(md5(id::text),21,12))::uuid, source, \
                encode_to_ecvector(source, 4, 42) FROM row_data;"
    );
    let seed_output = run_capture(psql, socket_dir, coord_port, &seed_insert_sql).await;
    if !seed_output.status_ok {
        crate::ecaz_println!(
            "[distann-multicluster] physical_concurrent_insert_query DIAG role=shared_target reason=seed_insert_failed stderr={}",
            compact_capture_error(&seed_output.stderr)
        );
        return Ok(false);
    }
    let seed_mapping_output = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!(
            "SELECT m.vec_id::text FROM {table} d \
               JOIN ec_distann_physical_source_map m ON m.source_tid = d.ctid \
              WHERE m.index_oid = 'dm_idx'::regclass::oid AND d.id = {seed_id};"
        ),
    )
    .await;
    let Some(seed_signed_id) = seed_mapping_output
        .lines()
        .map(str::trim)
        .find_map(|line| line.parse::<i64>().ok())
    else {
        crate::ecaz_println!(
            "[distann-multicluster] physical_concurrent_insert_query DIAG role=shared_target reason=seed_mapping_failed output={}",
            compact_capture_error(&seed_mapping_output)
        );
        return Ok(false);
    };
    let mut seed_neighbors = HashSet::new();
    for owner_node in owner_nodes {
        let seed_graph_sql = format!(
            "SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id={}; \
             SELECT vec_id::text || '|' || COALESCE(array_to_string(neighbor_vec_ids, ','), '') \
               FROM ec_distann_expand_physical_nodes(\
                    'public.dm_idx'::regclass, decode('{epoch_fingerprint}', 'hex'), \
                    ARRAY[{zero_query}]::real[], ARRAY[{seed_signed_id}]::bigint[], NULL, NULL);",
            owner_node.node_id,
        );
        let seed_graph_output =
            capture_psql_allow_error(psql, socket_dir, owner_node.port, &seed_graph_sql).await;
        if let Some((_, neighbors)) = seed_graph_output
            .lines()
            .find_map(|line| line.trim().split_once('|'))
        {
            seed_neighbors.extend(
                neighbors
                    .split(',')
                    .filter_map(|value| value.parse::<i64>().ok())
                    .map(|value| u64::from_le_bytes(value.to_le_bytes())),
            );
            break;
        }
    }
    let mut shared_target_candidates = Vec::new();
    for owner_node in owner_nodes {
        let relation_output = capture_psql_allow_error(
            psql,
            socket_dir,
            owner_node.port,
            "SELECT graph_store_relid::regclass::text \
               FROM ec_distann_generation \
              WHERE index_oid='dm_idx'::regclass::oid AND state='Published' \
              ORDER BY epoch DESC LIMIT 1;",
        )
        .await;
        let Some(graph_relation) = relation_output.lines().map(str::trim).find(|line| {
            !line.is_empty()
                && line
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'"'))
        }) else {
            continue;
        };
        let graph_sql = format!(
            "SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id={}; \
             SELECT vec_id::text || '|' || COALESCE(array_to_string(neighbor_vec_ids, ','), '') \
               FROM ec_distann_expand_physical_nodes(\
                    'public.dm_idx'::regclass, \
                    decode('{epoch_fingerprint}', 'hex'), \
                    ARRAY[{zero_query}]::real[], \
                    (SELECT COALESCE(array_agg(vec_id), ARRAY[]::bigint[]) \
                       FROM {graph_relation} WHERE is_current), NULL, NULL);",
            owner_node.node_id,
        );
        let graph_output =
            capture_psql_allow_error(psql, socket_dir, owner_node.port, &graph_sql).await;
        // Select a target with exactly two spare slots for the two concurrent
        // writers. This directly exercises append-when-room; before and after
        // the fill wave must both be exactly at the configured degree.
        let full_degree = args.graph_degree as usize;
        for target in graph_output.lines().filter_map(|line| {
            let (vec_id, neighbors) = line.trim().split_once('|')?;
            let neighbor_count = neighbors
                .split(',')
                .filter(|value| !value.is_empty())
                .count();
            let signed_id = vec_id.parse::<i64>().ok()?;
            if neighbor_count.saturating_add(WRITERS) != full_degree {
                return None;
            }
            Some((
                owner_node.node_id,
                u64::from_le_bytes(signed_id.to_le_bytes()),
                signed_id,
                neighbor_count,
            ))
        }) {
            shared_target_candidates.push(target);
        }
    }
    let candidate_signed_ids = shared_target_candidates
        .iter()
        .map(|(_, _, signed_id, _)| signed_id.to_string())
        .collect::<Vec<_>>();
    if candidate_signed_ids.is_empty() {
        crate::ecaz_println!(
            "[distann-multicluster] physical_concurrent_insert_query DIAG role=shared_target reason=no_target_with_two_spare_slots graph_degree={}",
            args.graph_degree
        );
        return Ok(false);
    }
    let seeded_target_signed_id = shared_target_candidates
        .iter()
        .find(|(_, vec_id, _, _)| seed_neighbors.contains(vec_id))
        .map(|(_, _, signed_id, _)| *signed_id)
        .or_else(|| {
            shared_target_candidates
                .first()
                .map(|(_, _, signed_id, _)| *signed_id)
        });
    crate::ecaz_println!(
        "[distann-multicluster] physical_concurrent_insert_query DIAG role=shared_target seed_vec_id={} seed_neighbor_count={} seeded_target_signed_id={:?}",
        seed_signed_id,
        seed_neighbors.len(),
        seeded_target_signed_id
    );
    let Some(nearest_target_signed_id) = seeded_target_signed_id else {
        crate::ecaz_println!(
            "[distann-multicluster] physical_concurrent_insert_query DIAG role=shared_target reason=no_full_target"
        );
        return Ok(false);
    };
    let Some((
        shared_target_owner,
        shared_target_vec_id,
        shared_target_signed_id,
        shared_target_initial_neighbor_count,
    )) = shared_target_candidates
        .into_iter()
        .find(|(_, _, signed_id, _)| *signed_id == nearest_target_signed_id)
    else {
        crate::ecaz_println!(
            "[distann-multicluster] physical_concurrent_insert_query DIAG role=shared_target reason=nearest_target_owner_lookup_failed signed_vec_id={nearest_target_signed_id}"
        );
        return Ok(false);
    };
    let source_output = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!(
            "SELECT d.source::text FROM {table} d \
               JOIN ec_distann_physical_source_map m ON m.source_tid = d.ctid \
              WHERE m.index_oid = 'dm_idx'::regclass::oid \
                AND m.vec_id = {shared_target_signed_id};"
        ),
    )
    .await;
    let Some(source) = source_output.lines().map(str::trim).find(|line| {
        line.starts_with('{')
            && line.ends_with('}')
            && line.bytes().all(|byte| {
                byte.is_ascii_digit()
                    || matches!(byte, b'{' | b'}' | b',' | b'.' | b'-' | b'+' | b'e' | b'E')
            })
    }) else {
        crate::ecaz_println!(
            "[distann-multicluster] physical_concurrent_insert_query DIAG role=shared_target reason=source_lookup_failed output={}",
            compact_capture_error(&source_output)
        );
        return Ok(false);
    };
    let shared_target_source = source.to_owned();
    crate::ecaz_println!(
        "[distann-multicluster] physical_concurrent_insert_query DIAG role=shared_target owner={} vec_id={} signed_vec_id={} source={}",
        shared_target_owner,
        shared_target_vec_id,
        shared_target_signed_id,
        shared_target_source
    );
    let mut counter_reset_ok = true;
    let mut counter_reset_outputs = Vec::new();
    for owner in owner_nodes {
        let output = capture_psql_allow_error(
            psql,
            socket_dir,
            owner.port,
            "CREATE UNLOGGED TABLE IF NOT EXISTS public.ec_distann_retry_attribution (\
                 backend_pid integer NOT NULL,\
                 node_id integer NOT NULL,\
                 served_epoch bigint NOT NULL,\
                 missing_vec_id bigint NOT NULL,\
                 recorded_at timestamptz NOT NULL DEFAULT clock_timestamp()\
             );\
             TRUNCATE public.ec_distann_retry_attribution;\
             SELECT ec_distann_stage_scoring_reset();",
        )
        .await;
        counter_reset_ok &= !output.contains("ERROR");
        counter_reset_outputs.push((owner.node_id, compact_capture_error(&output)));
    }
    if !counter_reset_ok {
        crate::ecaz_println!(
            "[distann-multicluster] physical_concurrent_insert_query DIAG role=frontier_retry_counter reason=owner_reset_failed outputs={counter_reset_outputs:?}"
        );
        return Ok(false);
    }
    // The attribution relation is created by fixture setup before this reset;
    // it is intentionally not touched while the 2PC wave is running.
    let retry_snapshot_sql = "SELECT count(*) FROM public.ec_distann_retry_attribution;";
    let parse_retry_count = |output: &str| {
        output
            .lines()
            .find_map(|line| line.trim().parse::<u64>().ok())
    };
    let inserted_ids = (0..WRITERS)
        .flat_map(|writer| {
            (0..ITERATIONS).map(move |iteration| {
                900_000_i64 + base_rows as i64 + (writer * ITERATIONS + iteration) as i64
            })
        })
        .collect::<HashSet<_>>();

    let mut tasks = Vec::with_capacity(SCANNERS + WRITERS);
    let start_barrier = Arc::new(Barrier::new(SCANNERS + WRITERS));
    let active_writers = Arc::new(AtomicUsize::new(WRITERS));
    for _ in 0..SCANNERS {
        let psql = psql.to_path_buf();
        let socket_dir = socket_dir.to_path_buf();
        let query_sql = query_sql.clone();
        let start_barrier = Arc::clone(&start_barrier);
        let active_writers = Arc::clone(&active_writers);
        tasks.push(tokio::spawn(async move {
            start_barrier.wait().await;
            let max_iterations = ITERATIONS * 16;
            for iteration in 0..max_iterations {
                if iteration >= ITERATIONS && active_writers.load(Ordering::Acquire) == 0 {
                    return true;
                }
                let output = run_capture(&psql, &socket_dir, coord_port, &query_sql).await;
                if !output.status_ok {
                    crate::ecaz_println!(
                        "[distann-multicluster] physical_concurrent_insert_query DIAG role=scanner iteration={iteration} stderr={}",
                        compact_capture_error(&output.stderr)
                    );
                    return false;
                }
                let count = output
                    .stdout
                    .lines()
                    .find_map(|line| line.trim().parse::<i64>().ok());
                if count != Some(expected_count) {
                    crate::ecaz_println!(
                        "[distann-multicluster] physical_concurrent_insert_query DIAG role=scanner iteration={iteration} count={count:?} expected={expected_count} stdout={}",
                        compact_capture_error(&output.stdout)
                    );
                    return false;
                }
            }
            crate::ecaz_println!(
                "[distann-multicluster] physical_concurrent_insert_query DIAG role=scanner reason=writers_exceeded_scan_budget active_writers={}",
                active_writers.load(Ordering::Acquire)
            );
            false
        }));
    }

    for writer in 0..WRITERS {
        let psql_insert = psql.to_path_buf();
        let socket_dir_insert = socket_dir.to_path_buf();
        let insert_roster = roster.clone();
        let insert_table = table.to_owned();
        let insert_vector = insert_vector.clone();
        let shared_target_source = shared_target_source.clone();
        let start_barrier = Arc::clone(&start_barrier);
        let active_writers = Arc::clone(&active_writers);
        tasks.push(tokio::spawn(async move {
            start_barrier.wait().await;
            let mut writer_ok = true;
            for iteration in 0..ITERATIONS {
                let id = 900_000_i64
                    + base_rows as i64
                    + (writer * ITERATIONS + iteration) as i64;
                // Force the first insert from each writer through the shared
                // target. Later rows keep the ordinary deterministic source
                // used by the concurrent wave.
                let source_expr = if iteration == 0 {
                    format!("'{shared_target_source}'::real[]")
                } else {
                    insert_vector.clone()
                };
                let writer_owner = writer + 2;
                let insert_sql = format!(
                    "SET ec_distann.debug_retry_attribution=on; \
                     SET ec_distann.roster='{insert_roster}'; SET ec_distann.local_node_id={writer_owner}; \
                     WITH row_data AS (SELECT {id}::bigint AS id, {source_expr}::real[] AS source) \
                     INSERT INTO {insert_table} (id, source_id, source, embedding) \
                     SELECT id, (substr(md5(id::text),1,8)||'-'||substr(md5(id::text),9,4)||'-4'||\
                            substr(md5(id::text),14,3)||'-8'||substr(md5(id::text),18,3)||'-'||\
                            substr(md5(id::text),21,12))::uuid, source, \
                            encode_to_ecvector(source, 4, 42) FROM row_data;"
                );
                let output =
                    run_capture(&psql_insert, &socket_dir_insert, coord_port, &insert_sql).await;
                if !output.status_ok {
                    crate::ecaz_println!(
                        "[distann-multicluster] physical_concurrent_insert_query DIAG role=writer writer={writer} iteration={iteration} id={id} stderr={}",
                        compact_capture_error(&output.stderr)
                    );
                    writer_ok = false;
                    break;
                }
            }
            active_writers.fetch_sub(1, Ordering::Release);
            writer_ok
        }));
    }

    let mut pass = true;
    for task in tasks {
        match task.await {
            Ok(result) => pass &= result,
            Err(error) => {
                crate::ecaz_println!(
                    "[distann-multicluster] physical concurrency task panicked: {error}"
                );
                pass = false;
            }
        }
    }
    let mut recent_intent_rows_by_owner = Vec::new();
    for owner in owner_nodes {
        let output = capture_psql_allow_error(
            psql,
            socket_dir,
            owner.port,
            "SELECT node_id::text || '|' || served_epoch::text || '|' || intent_state || '|' || COALESCE(tracked_vec_id::text, 'NULL') \
               FROM ec_distann_remote_prepared_xact_intent \
              WHERE updated_at >= clock_timestamp() - interval '30 seconds' \
                AND intent_state IN ('prepare_requested', 'prepare_acked', 'commit_intended', 'commit_local') \
              ORDER BY updated_at DESC LIMIT 24;",
        )
        .await;
        recent_intent_rows_by_owner.push((owner.node_id, compact_capture_error(&output)));
    }
    crate::ecaz_println!(
        "[distann-multicluster] physical_concurrent_insert_query DIAG role=recent_intents by_owner={recent_intent_rows_by_owner:?}"
    );
    let mut natural_retries_by_owner = Vec::new();
    for owner in owner_nodes {
        let output =
            capture_psql_allow_error(psql, socket_dir, owner.port, retry_snapshot_sql).await;
        natural_retries_by_owner.push((
            owner.node_id,
            parse_retry_count(&output),
            compact_capture_error(&output),
        ));
    }
    let natural_retries = natural_retries_by_owner
        .iter()
        .map(|(_, count, _)| *count)
        .sum::<Option<u64>>();
    let target_port = owner_nodes
        .iter()
        .find(|node| node.node_id == shared_target_owner)
        .map(|node| node.port)
        .unwrap_or(coord_port);
    let target_graph_sql = format!(
        "SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id={shared_target_owner}; \
         SELECT vec_id::text || '|' || COALESCE(array_to_string(neighbor_vec_ids, ','), '') \
           FROM ec_distann_expand_physical_nodes(\
                'public.dm_idx'::regclass, \
                decode('{epoch_fingerprint}', 'hex'), \
                ARRAY[{zero_query}]::real[], \
                ARRAY[{shared_target_signed_id}]::bigint[], NULL, NULL);"
    );
    let before_saturation_output =
        capture_psql_allow_error(psql, socket_dir, target_port, &target_graph_sql).await;
    let before_saturation_count = before_saturation_output
        .lines()
        .find_map(|line| line.trim().split_once('|'))
        .map(|(_, neighbors)| {
            neighbors
                .split(',')
                .filter(|value| !value.is_empty())
                .count()
        });
    let saturation_needed = before_saturation_count
        .map(|count| (args.graph_degree as usize).saturating_sub(count))
        .unwrap_or(0);
    // Fill the target to the configured degree before the second wave. Keep
    // this range disjoint from the first writer wave: those ids occupy
    // base_rows + 0..WRITERS*ITERATIONS.
    let saturation_rows = (0..saturation_needed)
        .map(|offset| {
            let id = 900_000_i64
                + base_rows as i64
                + (WRITERS * ITERATIONS) as i64
                + 1_000
                + offset as i64;
            // Use the target's exact source vector so every fill row is a
            // deterministic candidate backlink. Arbitrary corpus vectors can
            // be valid inserts without being selected into this target's
            // neighbour list, leaving the saturation precondition unmet.
            let source_expr = format!("'{shared_target_source}'::real[]");
            (id, source_expr)
        })
        .collect::<Vec<_>>();
    let saturation_barrier = Arc::new(Barrier::new(saturation_rows.len().max(1)));
    let mut saturation_tasks = Vec::new();
    for (id, saturation_source_expr) in saturation_rows {
        let psql_insert = psql.to_path_buf();
        let socket_dir_insert = socket_dir.to_path_buf();
        let insert_roster = roster.clone();
        let insert_table = table.to_owned();
        let insert_source_expr = saturation_source_expr;
        let saturation_barrier = Arc::clone(&saturation_barrier);
        saturation_tasks.push(tokio::spawn(async move {
            saturation_barrier.wait().await;
            let insert_sql = format!(
                "SET ec_distann.roster='{insert_roster}'; SET ec_distann.local_node_id=1; \
                 WITH row_data AS (SELECT {id}::bigint AS id, {insert_source_expr}::real[] AS source) \
                 INSERT INTO {insert_table} (id, source_id, source, embedding) \
                 SELECT id, (substr(md5(id::text),1,8)||'-'||substr(md5(id::text),9,4)||'-4'||\
                        substr(md5(id::text),14,3)||'-8'||substr(md5(id::text),18,3)||'-'||\
                        substr(md5(id::text),21,12))::uuid, source, \
                        encode_to_ecvector(source, 4, 42) FROM row_data;"
            );
            run_capture(&psql_insert, &socket_dir_insert, coord_port, &insert_sql)
                .await
                .status_ok
        }));
    }
    let mut saturation_inserts_ok = true;
    for task in saturation_tasks {
        saturation_inserts_ok &= task.await.unwrap_or(false);
    }
    let final_output =
        capture_psql_allow_error(psql, socket_dir, target_port, &target_graph_sql).await;
    let final_neighbor_count = final_output
        .lines()
        .find_map(|line| line.trim().split_once('|'))
        .map(|(_, neighbors)| {
            neighbors
                .split(',')
                .filter(|value| !value.is_empty())
                .count()
        });
    let saturation_pass = before_saturation_count == Some(args.graph_degree as usize)
        && saturation_inserts_ok
        && final_neighbor_count == Some(args.graph_degree as usize);
    crate::ecaz_println!(
        "[distann-multicluster] physical_concurrent_insert_query DIAG role=saturated_target owner={} vec_id={} initial_neighbors={} before_neighbors={:?} final_neighbors={:?} inserts_ok={} pass={}",
        shared_target_owner,
        shared_target_vec_id,
        shared_target_initial_neighbor_count,
        before_saturation_count,
        final_neighbor_count,
        saturation_inserts_ok,
        saturation_pass
    );
    pass &= saturation_pass;
    let mut steady_reset_ok = true;
    let mut steady_reset_outputs = Vec::new();
    for owner in owner_nodes {
        let output = capture_psql_allow_error(
            psql,
            socket_dir,
            owner.port,
            "TRUNCATE public.ec_distann_retry_attribution; \
             SELECT ec_distann_stage_scoring_reset();",
        )
        .await;
        steady_reset_ok &= !output.contains("ERROR");
        steady_reset_outputs.push((owner.node_id, compact_capture_error(&output)));
    }
    let steady_query = run_capture(psql, socket_dir, coord_port, &query_sql).await;
    let mut steady_retries_by_owner = Vec::new();
    for owner in owner_nodes {
        let output =
            capture_psql_allow_error(psql, socket_dir, owner.port, retry_snapshot_sql).await;
        steady_retries_by_owner.push((
            owner.node_id,
            parse_retry_count(&output),
            compact_capture_error(&output),
        ));
    }
    let steady_retries = steady_retries_by_owner
        .iter()
        .map(|(_, count, _)| *count)
        .sum::<Option<u64>>();
    let retry_counter_ok = counter_reset_ok
        && steady_reset_ok
        && steady_query.status_ok
        && natural_retries.is_some_and(|count| count > 0)
        && steady_retries == Some(0);
    crate::ecaz_println!(
        "[distann-multicluster] physical_concurrent_insert_query DIAG role=frontier_retry_counter retry_source=natural_2pc_wave forced_retry_probe=false natural_retries={natural_retries:?} natural_by_owner={natural_retries_by_owner:?} steady_retries={steady_retries:?} steady_by_owner={steady_retries_by_owner:?} pass={retry_counter_ok} reset_outputs={counter_reset_outputs:?} steady_reset_outputs={steady_reset_outputs:?}"
    );
    let (inserted_vec_ids, inserted_vec_ids_by_id) = {
        let first_id = 900_000_i64 + base_rows as i64;
        let last_id = first_id + (WRITERS * ITERATIONS) as i64;
        let mapping_output = capture_psql_allow_error(
            psql,
            socket_dir,
            coord_port,
            &format!(
                "SELECT d.id::text || '|' || m.vec_id::text \
                   FROM {table} d \
                   JOIN ec_distann_physical_source_map m ON m.source_tid = d.ctid \
                  WHERE m.index_oid = 'dm_idx'::regclass::oid \
                    AND d.id >= {first_id} AND d.id < {last_id} \
                  ORDER BY d.id;"
            ),
        )
        .await;
        let mapped = mapping_output
            .lines()
            .filter_map(|line| {
                let (id, vec_id) = line.trim().split_once('|')?;
                let id = id.parse::<i64>().ok()?;
                let vec_id = vec_id.parse::<i64>().ok()?;
                Some((id, u64::from_le_bytes(vec_id.to_le_bytes())))
            })
            .collect::<HashMap<_, _>>();
        if mapped.len() != inserted_ids.len() {
            crate::ecaz_println!(
                "[distann-multicluster] physical_concurrent_insert_query DIAG role=back_edge_check reason=source_map_vec_id_mapping_incomplete expected={} actual={} output={}",
                inserted_ids.len(),
                mapped.len(),
                compact_capture_error(&mapping_output)
            );
            pass = false;
        }
        let inserted_vec_ids = mapped
            .iter()
            .filter(|(id, _)| inserted_ids.contains(id))
            .map(|(_, vec_id)| *vec_id)
            .collect::<HashSet<_>>();
        (inserted_vec_ids, mapped)
    };
    let mut reverse_edge_coverage_count = 0;
    let forward_neighbor_check = if !inserted_vec_ids.is_empty() {
        'check: {
            let graph_vec_ids = inserted_vec_ids
                .iter()
                .map(|vec_id| i64::from_le_bytes(vec_id.to_le_bytes()).to_string())
                .collect::<Vec<_>>()
                .join(",");
            let mut found = HashSet::new();
            let mut check_ok = true;
            let mut current_graph_rows = 0usize;
            for owner_node in owner_nodes {
                let relation_output = capture_psql_allow_error(
                    psql,
                    socket_dir,
                    owner_node.port,
                    "SELECT graph_store_relid::regclass::text \
                   FROM ec_distann_generation \
                  WHERE index_oid='dm_idx'::regclass::oid AND state='Published' \
                  ORDER BY epoch DESC LIMIT 1;",
                )
                .await;
                let Some(graph_relation) = relation_output.lines().map(str::trim).find(|line| {
                    !line.is_empty()
                        && line.bytes().all(|byte| {
                            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'"')
                        })
                }) else {
                    crate::ecaz_println!(
                    "[distann-multicluster] physical_concurrent_insert_query DIAG role=back_edge_check node={} reason=published_graph_relation_missing output={}",
                    owner_node.node_id,
                    compact_capture_error(&relation_output)
                );
                    check_ok = false;
                    continue;
                };
                let graph_count_output = capture_psql_allow_error(
                    psql,
                    socket_dir,
                    owner_node.port,
                    &format!(
                        "SELECT count(*) FROM {graph_relation} \
                           WHERE is_current \
                             AND vec_id = ANY(ARRAY[{graph_vec_ids}]::bigint[]);"
                    ),
                )
                .await;
                let Some(graph_count) = graph_count_output
                    .lines()
                    .find_map(|line| line.trim().parse::<usize>().ok())
                else {
                    crate::ecaz_println!(
                    "[distann-multicluster] physical_concurrent_insert_query DIAG role=back_edge_check node={} reason=inserted_graph_rows_lookup_failed output={}",
                    owner_node.node_id,
                    compact_capture_error(&graph_count_output)
                );
                    check_ok = false;
                    continue;
                };
                current_graph_rows += graph_count;
                let graph_sql = format!(
                "SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id={}; \
                 SELECT vec_id::text || '|' || COALESCE(array_to_string(neighbor_vec_ids, ','), '') \
                   FROM ec_distann_expand_physical_nodes(\
                        'public.dm_idx'::regclass, \
                        decode('{epoch_fingerprint}', 'hex'), \
                        ARRAY[{zero_query}]::real[], \
                        (SELECT COALESCE(array_agg(vec_id), ARRAY[]::bigint[]) \
                           FROM {graph_relation} WHERE is_current), NULL, NULL);",
                owner_node.node_id,
            );
                let graph_output =
                    capture_psql_allow_error(psql, socket_dir, owner_node.port, &graph_sql).await;
                if graph_output.starts_with("psql:") || graph_output.contains("ERROR") {
                    crate::ecaz_println!(
                    "[distann-multicluster] physical_concurrent_insert_query DIAG role=back_edge_check node={} reason=graph_expansion_failed output={}",
                    owner_node.node_id,
                    compact_capture_error(&graph_output)
                );
                    continue;
                }
                for line in graph_output.lines() {
                    let Some((_vec_id, neighbors)) = line.trim().split_once('|') else {
                        continue;
                    };
                    for neighbor in neighbors
                        .split(',')
                        .filter_map(|value| value.parse::<i64>().ok())
                    {
                        let neighbor = u64::from_le_bytes(neighbor.to_le_bytes());
                        if inserted_vec_ids.contains(&neighbor) {
                            found.insert(neighbor);
                        }
                    }
                }
            }
            reverse_edge_coverage_count = found.len();
            if current_graph_rows != inserted_vec_ids.len() {
                crate::ecaz_println!(
                "[distann-multicluster] physical_concurrent_insert_query DIAG role=back_edge_check inserted_graph_rows={} expected_graph_rows={}",
                current_graph_rows,
                inserted_vec_ids.len()
                );
            }
            let shared_ids = [
                900_000_i64 + base_rows as i64,
                900_000_i64 + base_rows as i64 + ITERATIONS as i64,
            ];
            let Some(shared_vec_ids) = shared_ids
                .iter()
                .map(|id| inserted_vec_ids_by_id.get(id).copied())
                .collect::<Option<Vec<_>>>()
            else {
                crate::ecaz_println!(
                    "[distann-multicluster] physical_concurrent_insert_query DIAG role=shared_target reason=controlled_insert_mapping_missing"
                );
                break 'check false;
            };
            let target_port = owner_nodes
                .iter()
                .find(|node| node.node_id == shared_target_owner)
                .map(|node| node.port)
                .unwrap_or(coord_port);
            let target_graph_sql = format!(
                "SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id={shared_target_owner}; \
                 SELECT vec_id::text || '|' || COALESCE(array_to_string(neighbor_vec_ids, ','), '') \
                   FROM ec_distann_expand_physical_nodes(\
                        'public.dm_idx'::regclass, \
                        decode('{epoch_fingerprint}', 'hex'), \
                        ARRAY[{zero_query}]::real[], \
                        ARRAY[{shared_target_signed_id}]::bigint[], NULL, NULL);"
            );
            let target_output =
                capture_psql_allow_error(psql, socket_dir, target_port, &target_graph_sql).await;
            let target_neighbors = target_output
                .lines()
                .find_map(|line| line.trim().split_once('|'))
                .map(|(_, neighbors)| {
                    neighbors
                        .split(',')
                        .filter_map(|value| value.parse::<i64>().ok())
                        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default();
            let shared_target_ok = shared_vec_ids
                .iter()
                .all(|vec_id| target_neighbors.contains(vec_id));
            crate::ecaz_println!(
                "[distann-multicluster] physical_concurrent_insert_query DIAG role=shared_target owner={} target_vec_id={} inserted_vec_ids={:?} target_neighbors={:?} pass={}",
                shared_target_owner,
                shared_target_vec_id,
                shared_vec_ids,
                target_neighbors,
                shared_target_ok
            );
            // The broad count proves every committed insert has one current
            // graph record on its placement owner. The controlled target is
            // the lost-update invariant: two writers insert near-duplicates
            // while sharing one target, and both backlinks must survive.
            break 'check check_ok
                && current_graph_rows == inserted_vec_ids.len()
                && shared_target_ok;
        }
    } else {
        false
    };
    // Reverse-edge coverage: `found` contains inserted vec_ids discovered in
    // other nodes' neighbour lists. This is not forward-edge selection by
    // the inserted nodes; the controlled target assertion separately proves
    // the two writer backlinks.
    crate::ecaz_println!(
        "[distann-multicluster] physical_concurrent_insert_query DIAG scanners={SCANNERS} writers={WRITERS} iterations={ITERATIONS} expected_count={expected_count} reverse_edge_coverage={reverse_edge_coverage_count}/{} shared_target_vec_id={} shared_target_owner={} back_edge_check={forward_neighbor_check} pass={pass}",
        inserted_vec_ids.len(),
        shared_target_vec_id,
        shared_target_owner
    );
    Ok(pass && forward_neighbor_check && retry_counter_ok)
}

fn task167_retry_attribution_roster(roster: &str) -> Result<String> {
    roster
        .split(';')
        .map(|entry| {
            let (conninfo, options) = entry.rsplit_once(" options=").ok_or_else(|| {
                eyre!("Task 167 retry-attribution roster entry has no options: {entry}")
            })?;
            if options.is_empty()
                || options
                    .bytes()
                    .any(|byte| byte.is_ascii_whitespace() || matches!(byte, b'\'' | b'"'))
            {
                bail!("Task 167 retry-attribution roster has unsafe options: {options}");
            }
            Ok(format!(
                "{conninfo} options='{options} -cec_distann.debug_retry_attribution=on'"
            ))
        })
        .collect::<Result<Vec<_>>>()
        .map(|entries| entries.join(";"))
}

/// Commit one coordinator-routed physical insert against a non-local owner.
/// The empty roster is deliberate: it exercises the published participant
/// binding fallback used when the coordinator session has no operator roster
/// GUC installed. Candidates that hash to the coordinator are retained as
/// harmless local inserts while searching for a remote owner.
async fn physical_remote_insert_probe(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    args: &LocalMultinodePg18Args,
    nodes: &[Node],
    owner_count: usize,
) -> Result<bool> {
    let vector = insert_vector_expr(args, "dm");
    for candidate in 0..32_u32 {
        let id = 910_000_i64 + i64::from(candidate);
        let source_id = format!(
            "11111111-1111-4{:03x}-8{:03x}-{:012x}",
            candidate,
            candidate,
            u64::from(candidate) + 1
        );
        let insert_sql = format!(
            "SET ec_distann.roster=''; SET ec_distann.local_node_id=1; \
             WITH row_data AS (SELECT {id}::bigint AS id, '{source_id}'::uuid AS source_id, \
                                      {vector}::real[] AS source) \
             INSERT INTO dm (id, source_id, source, embedding) \
             SELECT id, source_id, source, encode_to_ecvector(source, 4, 42) FROM row_data; \
             SELECT m.vec_id::text || '|' || ec_distann_owning_node(m.vec_id, {owner_count}, 1)::text \
               FROM ec_distann_physical_source_map m \
               JOIN dm d ON d.ctid = m.source_tid \
              WHERE m.index_oid = 'dm_idx'::regclass::oid AND d.source_id = '{source_id}'::uuid;"
        );
        let output = run_capture(psql, socket_dir, coord_port, &insert_sql).await;
        if !output.status_ok {
            crate::ecaz_println!(
                "[distann-multicluster] physical_remote_insert_probe candidate={candidate} status=false stderr={}",
                compact_capture_error(&output.stderr)
            );
            return Ok(false);
        }
        let Some((vec_id_text, owner_text)) = output
            .stdout
            .lines()
            .find_map(|line| line.trim().split_once('|'))
        else {
            crate::ecaz_println!(
                "[distann-multicluster] physical_remote_insert_probe candidate={candidate} status=false reason=owner_not_found stdout={}",
                compact_capture_error(&output.stdout)
            );
            return Ok(false);
        };
        let vec_id = vec_id_text.parse::<i64>()?;
        let owner = owner_text.parse::<usize>()?;
        let remote = if args.coordinator_outside_roster {
            owner < owner_count
        } else {
            owner > 0 && owner < owner_count
        };
        crate::ecaz_println!(
            "[distann-multicluster] physical_remote_insert_probe candidate={candidate} committed=true owner={owner} remote={remote}"
        );
        if remote {
            let owner_node_index = if args.coordinator_outside_roster {
                owner + 1
            } else {
                owner
            };
            let Some(owner_node) = nodes.get(owner_node_index) else {
                return Ok(false);
            };
            let relation_output = capture_psql_allow_error(
                psql,
                socket_dir,
                owner_node.port,
                "SELECT graph_store_relid::regclass::text \
                   FROM ec_distann_generation \
                  WHERE index_oid='dm_idx'::regclass::oid AND state='Published' \
                  ORDER BY epoch DESC LIMIT 1;",
            )
            .await;
            let Some(graph_relation) = relation_output.lines().map(str::trim).find(|line| {
                !line.is_empty()
                    && line.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'"')
                    })
            }) else {
                return Ok(false);
            };
            let owner_check = capture_psql_allow_error(
                psql,
                socket_dir,
                owner_node.port,
                &format!(
                    "SELECT count(*) FROM {graph_relation} \
                      WHERE vec_id = {vec_id} AND is_current;"
                ),
            )
            .await;
            let owner_count = owner_check
                .lines()
                .find_map(|line| line.trim().parse::<i64>().ok());
            if owner_count != Some(1) {
                crate::ecaz_println!(
                    "[distann-multicluster] physical_remote_insert_probe candidate={candidate} owner_graph_check=false vec_id={vec_id} output={}",
                    compact_capture_error(&owner_check)
                );
                return Ok(false);
            }
            return Ok(true);
        }
    }
    Ok(false)
}

/// Exercise the committed physical DELETE path through PostgreSQL VACUUM. The
/// selected row is deliberately owned by a remote participant so this covers
/// the routed tombstone endpoint and verifies the owner retained the graph
/// record as a tombstone after the source heap tuple was reclaimed.
async fn physical_routed_delete_vacuum_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    nodes: &[Node],
    args: &LocalMultinodePg18Args,
    roster: &str,
    table: &str,
    epoch_fingerprint: &str,
) -> Result<bool> {
    let owner_count = if args.coordinator_outside_roster {
        args.nodes.saturating_sub(1)
    } else {
        args.nodes
    };
    let discovered = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!(
            "SET ec_distann.roster = '{}'; SET ec_distann.local_node_id=1; \
             SELECT t.id || '|' || m.vec_id::text || '|' || \
                    ec_distann_owning_node(m.vec_id, {owner_count}, 1)::text \
               FROM {table} t \
             JOIN ec_distann_physical_source_map m ON m.source_tid = t.ctid \
              WHERE m.index_oid = 'dm_idx'::regclass::oid \
                AND ec_distann_owning_node(m.vec_id, {owner_count}, 1) > 0 \
              ORDER BY t.id LIMIT 1;",
            roster.replace('\'', "''")
        ),
    )
    .await;
    let Some((id_text, rest)) = discovered
        .lines()
        .find_map(|line| line.trim().split_once('|'))
    else {
        crate::ecaz_println!(
            "[distann-multicluster] physical_routed_delete_vacuum pass=false reason=no_live_source_row"
        );
        return Ok(false);
    };
    let Some((vec_id_text, owner_text)) = rest.split_once('|') else {
        return Ok(false);
    };
    let id = id_text.trim().parse::<i64>()?;
    let vec_id = vec_id_text.trim().parse::<i64>()?;
    let owner_ordinal = owner_text.trim().parse::<usize>()?;
    let owner_node_index = if args.coordinator_outside_roster {
        owner_ordinal + 1
    } else {
        owner_ordinal
    };
    let Some(owner_node) = nodes.get(owner_node_index) else {
        return Ok(false);
    };
    if owner_node.port == coord_port {
        return Ok(false);
    }
    let dimension_output = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT array_length(source, 1) FROM {table} LIMIT 1;"),
    )
    .await;
    let Some(dimension) = dimension_output
        .lines()
        .find_map(|line| line.trim().parse::<usize>().ok())
    else {
        crate::ecaz_println!(
            "[distann-multicluster] physical_routed_delete_vacuum pass=false reason=source_dimension_missing output={}",
            compact_capture_error(&dimension_output)
        );
        return Ok(false);
    };
    let deleted = run_capture(
        psql,
        socket_dir,
        coord_port,
        &format!("DELETE FROM {table} WHERE id = {id};"),
    )
    .await;
    let vacuum = run_capture(
        psql,
        socket_dir,
        coord_port,
        &format!("VACUUM (INDEX_CLEANUP ON) {table};"),
    )
    .await;
    let tombstone = capture_psql_allow_error(
        psql,
        socket_dir,
        owner_node.port,
        &format!(
            "SELECT is_tombstone \
               FROM ec_distann_expand_physical_nodes(\
                    'public.dm_idx'::regclass, decode('{epoch_fingerprint}', 'hex'), \
                    ARRAY[{zero_query}]::real[], ARRAY[{vec_id}]::bigint[], NULL, NULL);",
            zero_query = (0..dimension).map(|_| "0").collect::<Vec<_>>().join(","),
        ),
    )
    .await;
    let tombstoned = tombstone.lines().any(|line| line.trim() == "t");
    let pass = deleted.status_ok && vacuum.status_ok && tombstoned;
    crate::ecaz_println!(
        "[distann-multicluster] physical_routed_delete_vacuum pass={pass} id={id} vec_id={vec_id} owner={} delete_ok={} vacuum_ok={} owner_tombstone={tombstoned} delete_stderr={} vacuum_stderr={} owner_output={}",
        owner_node.node_id,
        deleted.status_ok,
        vacuum.status_ok,
        compact_capture_error(&deleted.stderr),
        compact_capture_error(&vacuum.stderr),
        compact_capture_error(&tombstone),
    );
    Ok(pass)
}

/// One co-placement-drift case: pick a live record owned by `owner_idx`, delete
/// its heap row on EVERY node (index record survives ⇒ cluster-wide dangling
/// record / missing co-placed vector), and assert the NFR-020 disjunction — the
/// multinode scan SHALL either raise an error OR return a correct complete result
/// (equal to a single-node scan over the same deleted corpus, target excluded),
/// never a partial/stale result presented as complete.
async fn co_placement_drift_case(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster: &str,
    nodes: &[Node],
    args: &LocalMultinodePg18Args,
    owner_idx: u32,
) -> bool {
    let discover = format!(
        "SELECT t.id || '|' || t.source::text \
           FROM ec_distann_list_directory('dm_idx'::regclass) d \
           JOIN dm t ON t.ctid = ('(' || d.heap_block || ',' || d.heap_offset || ')')::tid \
          WHERE NOT d.is_tombstone \
            AND ec_distann_owning_node(d.vec_id, {n}, 1) = {owner_idx} \
          ORDER BY t.id LIMIT 1;",
        n = args.nodes,
    );
    let discovered = capture_psql_allow_error(psql, socket_dir, coord_port, &discover).await;
    let Some((id_text, source_text)) = discovered
        .lines()
        .find_map(|l| l.trim().split_once('|'))
        .filter(|(id, src)| id.parse::<i64>().is_ok() && src.starts_with('{'))
    else {
        crate::ecaz_println!(
            "[distann-multicluster] co_placement_drift[owner={owner_idx}]: no record discovered (skipped)"
        );
        return false;
    };
    let target_id: i64 = id_text.trim().parse().unwrap();

    for node in nodes {
        if run_psql_file(
            psql,
            socket_dir,
            node.port,
            &format!("DELETE FROM dm WHERE id = {target_id};"),
        )
        .await
        .is_err()
        {
            return false;
        }
    }

    let anchor = format!("encode_to_ecvector('{source_text}'::real[], 4, 42)");
    let multi_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT id FROM dm ORDER BY embedding <#> {anchor} LIMIT {k};",
        k = args.top_k,
    );
    let single_sql = format!(
        "SET enable_seqscan=off; SELECT set_config('ec_distann.roster','',false); SET ec_distann.local_node_id=1; SET ec_distann.epoch=0; \
         SELECT id FROM dm ORDER BY embedding <#> {anchor} LIMIT {k};",
        k = args.top_k,
    );
    let multi_out = capture_psql_allow_error(psql, socket_dir, coord_port, &multi_sql).await;
    let single_out = capture_psql_allow_error(psql, socket_dir, coord_port, &single_sql).await;
    let errored = query_errored(&multi_out);

    let ids = |out: &str| -> Vec<i64> {
        let mut v: Vec<i64> = out.lines().filter_map(|l| l.trim().parse().ok()).collect();
        v.sort_unstable();
        v
    };
    let (multi_ids, single_ids) = (ids(&multi_out), ids(&single_out));
    let target_excluded = !multi_ids.contains(&target_id) && !single_ids.contains(&target_id);
    let correct_complete = !errored && multi_ids == single_ids && target_excluded;
    let pass = errored || correct_complete;
    let arm = if errored { "error" } else { "correct_complete" };
    crate::ecaz_println!(
        "[distann-multicluster] co_placement_drift[owner={owner_idx}] target_id={target_id} arm={arm} \
         multi_n={} single_n={} pass={pass}",
        multi_ids.len(),
        single_ids.len(),
    );

    // Recovery: restore the corpus on every node. Must use the SAME setup as the
    // initial build (real staged corpus when --corpus-prefix is set), else the
    // node is rebuilt with the synthetic dim-16 corpus and the post-recovery
    // real-query recall comparison fails with a dimension mismatch.
    if let Ok(setup) = build_setup_sql(args) {
        for node in nodes {
            let _ = run_psql_file(psql, socket_dir, node.port, &setup).await;
        }
    }
    pass
}

/// A drill query satisfies NFR-020's fail-closed arm if it raised an ERROR
/// rather than returning a (possibly wrong/partial) result.
fn query_errored(output: &str) -> bool {
    output.contains("ERROR")
        || output.contains("EC_INTERNAL")
        || output.contains("could not connect")
}

/// FR-082-AC-4 concurrency drill: `scanners` concurrent multi-node scan loops on
/// the coordinator, plus a background inserter mutating the table, all at once.
/// Returns true iff every session completed without error (each scan drew only
/// from expanded records — a torn/half-applied read would surface as an error).
async fn concurrency_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    nodes: &[Node],
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> Result<bool> {
    let scanners = 4;
    let iters = 12;
    let scan_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT count(*) FROM (SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {}) t;",
        args.top_k
    );
    // Insert vector: a real corpus vector (correct dimension) in real mode, else
    // a synthetic vector matching the corpus generator. Synthetic args.dim does
    // NOT match a real corpus dimension, so a synthetic vector would fail the
    // aminsert dimension check against the real index.
    let arr = insert_vector_expr(args, "dm");

    let mut tasks = Vec::new();
    for _ in 0..scanners {
        let (psql, socket_dir, sql) = (
            psql.to_path_buf(),
            socket_dir.to_path_buf(),
            scan_sql.clone(),
        );
        tasks.push(tokio::spawn(async move {
            for _ in 0..iters {
                let out = run_capture(&psql, &socket_dir, coord_port, &sql).await;
                if !out.status_ok {
                    // A scan that races an in-progress coordinated epoch swap may
                    // fail-closed with an epoch mismatch (FR-082-AC-2, one epoch per
                    // scan) — that is a correct outcome, not corruption. Any OTHER
                    // error (torn read, crash, wrong-result path) fails the drill.
                    let stderr = out.stderr.to_lowercase();
                    if stderr.contains("epoch") && stderr.contains("mismatch") {
                        continue;
                    }
                    return Err(out.stderr);
                }
            }
            Ok(())
        }));
    }
    // Background inserter: unique ids well above the corpus range.
    {
        let (psql, socket_dir, arr) = (psql.to_path_buf(), socket_dir.to_path_buf(), arr.clone());
        let base_rows = args.rows;
        tasks.push(tokio::spawn(async move {
            for i in 0..iters {
                let sql = format!(
                    "INSERT INTO dm SELECT {}, {arr}, encode_to_ecvector({arr}, 4, 42);",
                    900_000 + base_rows as i64 + i
                );
                let out = run_capture(&psql, &socket_dir, coord_port, &sql).await;
                if !out.status_ok {
                    return Err(out.stderr);
                }
            }
            Ok(())
        }));
    }
    // Epoch-swap-under-load (FR-082-AC-1 / one-epoch-per-scan): perform COORDINATED
    // epoch publishes across EVERY node while scans run. Publishing on all nodes
    // keeps the cluster at a single consistent epoch (all-1 or all-2) so each
    // in-flight scan returns wholly from one published epoch — a scan that races a
    // swap surfaces a retriable epoch mismatch and restarts under the refreshed
    // epoch (FR-082-AC-2), never a torn result. The metadata-page publish write
    // must not corrupt concurrent scans reading the metadata; end back at epoch 1.
    {
        let (psql, socket_dir) = (psql.to_path_buf(), socket_dir.to_path_buf());
        let ports: Vec<u16> = nodes.iter().map(|n| n.port).collect();
        tasks.push(tokio::spawn(async move {
            for i in 0..iters {
                let epoch = if i % 2 == 0 { 2 } else { 1 };
                let sql =
                    format!("SELECT ec_distann_publish_epoch('dm_idx'::regclass::oid, {epoch});");
                for &port in &ports {
                    let out = run_capture(&psql, &socket_dir, port, &sql).await;
                    if !out.status_ok {
                        return Err(out.stderr);
                    }
                }
            }
            for &port in &ports {
                let _ = run_capture(
                    &psql,
                    &socket_dir,
                    port,
                    "SELECT ec_distann_publish_epoch('dm_idx'::regclass::oid, 1);",
                )
                .await;
            }
            Ok(())
        }));
    }

    let mut ok = true;
    for task in tasks {
        match task.await {
            Ok(Ok(())) => {}
            Ok(Err(stderr)) => {
                crate::ecaz_println!("[distann-multicluster] concurrency session error: {stderr}");
                ok = false;
            }
            Err(join_err) => {
                crate::ecaz_println!(
                    "[distann-multicluster] concurrency task panicked: {join_err}"
                );
                ok = false;
            }
        }
    }
    Ok(ok)
}

/// True disjoint-shard demonstration: prune each node's replicated corpus to only
/// the heap rows it OWNS (`owning_node`), then prove the multi-node top-k result
/// signature is byte-identical to the pre-prune (replicated) result — i.e. the
/// distributed read is correct with genuinely disjoint per-node storage, not a
/// full replica. Returns a report line; fatal if the signature changes.
async fn disjoint_shard_drill(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> (String, bool) {
    let coord_port = nodes[0].port;
    // Operate on benchgate_corpus (a clean, cross-node-consistent copy the suite
    // gate created before the mutating drills). Save the query vectors first so
    // they survive pruning of non-owned coordinator rows.
    let setup = run_capture(
        psql,
        socket_dir,
        coord_port,
        &format!(
            "DROP TABLE IF EXISTS dj_queries; \
             CREATE TABLE dj_queries AS SELECT id AS qid, source AS v FROM benchgate_corpus WHERE id <= {};",
            args.queries
        ),
    )
    .await;
    if !setup.status_ok {
        return (
            "disjoint_shard=SKIPPED(no benchgate_corpus)".to_owned(),
            false,
        );
    }
    // Signature over (id, EXACT DISTANCE) per query in a canonical (dist, id)
    // order (021-P2): includes the distance — not just the id set — so a
    // distance/recall change is caught, while the canonical order makes it
    // deterministic (equal-distance tie order, which the scan does not guarantee
    // and which is not a recall property, does not spuriously fail the drill).
    let sig_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT md5(string_agg(qid || ':' || id || ':' || dist, ',' ORDER BY qid, dist, id)) FROM ( \
           SELECT q.qid, r.id, (r.embedding <#> q.v)::float8 AS dist FROM dj_queries q \
           CROSS JOIN LATERAL ( \
             SELECT id, embedding FROM benchgate_corpus ORDER BY embedding <#> q.v LIMIT {k}) r) t;",
        k = args.top_k
    );
    let sig = |out: String| {
        out.lines()
            .map(str::trim)
            .find(|l| l.len() == 32)
            .unwrap_or("")
            .to_owned()
    };
    let before = sig(capture_psql_allow_error(psql, socket_dir, coord_port, &sig_sql).await);

    // Prune each node to its owned shard: delete the heap rows for vec_ids this
    // node does not own, then VACUUM (ambulkdelete tombstones their records).
    let n = args.nodes;
    let mut row_report = Vec::new();
    for node in nodes {
        let owner_idx = node.node_id - 1; // placement index = roster position
        let before_rows = capture_psql_allow_error(
            psql,
            socket_dir,
            node.port,
            "SELECT count(*) FROM benchgate_corpus;",
        )
        .await
        .lines()
        .find_map(|l| l.trim().parse::<i64>().ok())
        .unwrap_or(-1);
        let del = run_capture(
            psql,
            socket_dir,
            node.port,
            &format!(
                "DELETE FROM benchgate_corpus WHERE ctid IN (\
                   SELECT ('(' || heap_block || ',' || heap_offset || ')')::tid \
                     FROM ec_distann_list_directory('benchgate_corpus_idx'::regclass::oid) \
                    WHERE NOT is_tombstone AND ec_distann_owning_node(vec_id, {n}, 1) <> {owner_idx});"
            ),
        )
        .await;
        let vac = run_capture(psql, socket_dir, node.port, "VACUUM benchgate_corpus;").await;
        if !del.status_ok || !vac.status_ok {
            return ("disjoint_shard=SKIPPED(prune failed)".to_owned(), false);
        }
        let after_rows = capture_psql_allow_error(
            psql,
            socket_dir,
            node.port,
            "SELECT count(*) FROM benchgate_corpus;",
        )
        .await
        .lines()
        .find_map(|l| l.trim().parse::<i64>().ok())
        .unwrap_or(-1);
        row_report.push(format!("n{}:{}->{}", node.node_id, before_rows, after_rows));
    }

    let after = sig(capture_psql_allow_error(psql, socket_dir, coord_port, &sig_sql).await);
    let identical = !before.is_empty() && before == after;
    (
        format!(
            "disjoint_shard identical_after_prune={identical} per_node_rows[{}]",
            row_report.join(" ")
        ),
        identical,
    )
}

/// Suite-driven recall gate (006-P1 letter): reuse the fixture corpus as a
/// `benchgate_*` bench-format corpus and run `ecaz bench recall` against the
/// coordinator single-node vs multi-node, asserting recall(multi) >= recall(single)
/// - 0.001. Best-effort/non-fatal: the byte-identical top-k gate is the hard one.
/// Task 172 AC-3 / NFR-018 cluster storage summation. Queries each node for its
/// `dm_idx` index bytes and `dm` heap bytes, sums across the cluster, and
/// computes the replicated-index space amplification vs raw f32 vectors
/// (`cluster_index_bytes / (rows * dim * 4)`). Emits one `storage_node` line per
/// node plus a `storage_summation` summary line (both parsed into result rows).
async fn storage_summation(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    _args: &LocalMultinodePg18Args,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cluster_index = 0i64;
    let mut cluster_heap = 0i64;
    let mut coord_index = 0i64;
    let mut rows = 0i64;
    let mut dim = 0i64;
    for (k, node) in nodes.iter().enumerate() {
        // index bytes | heap bytes | row count | vector dimension
        let sql = "SELECT pg_total_relation_size('dm_idx') || ' ' || \
                   pg_total_relation_size('dm') || ' ' || \
                   (SELECT count(*) FROM dm) || ' ' || \
                   coalesce((SELECT array_length(source, 1) FROM dm LIMIT 1), 0);";
        let out = capture_psql_allow_error(psql, socket_dir, node.port, sql).await;
        let vals: Vec<i64> = out
            .lines()
            .find(|l| {
                l.split_whitespace().count() == 4
                    && l.split_whitespace().all(|f| f.parse::<i64>().is_ok())
            })
            .map(|l| {
                l.split_whitespace()
                    .filter_map(|f| f.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        if vals.len() != 4 {
            lines.push(format!(
                "storage_node node={} index_bytes=0 heap_bytes=0 rows=0 (parse failed)",
                node.node_id
            ));
            continue;
        }
        let (idx, heap, n, d) = (vals[0], vals[1], vals[2], vals[3]);
        if k == 0 {
            coord_index = idx;
            rows = n;
            dim = d;
        }
        cluster_index += idx;
        cluster_heap += heap;
        lines.push(format!(
            "storage_node node={} index_bytes={idx} heap_bytes={heap} rows={n} dim={d}",
            node.node_id
        ));
    }
    let raw_vector_bytes = rows * dim * 4;
    let amp = if raw_vector_bytes > 0 {
        cluster_index as f64 / raw_vector_bytes as f64
    } else {
        0.0
    };
    lines.push(format!(
        "storage_summation nodes={} coord_index_bytes={coord_index} cluster_index_bytes={cluster_index} \
         cluster_heap_bytes={cluster_heap} corpus_rows={rows} dim={dim} raw_vector_bytes={raw_vector_bytes} \
         cluster_index_space_amplification={amp:.4}",
        nodes.len()
    ));
    lines
}

async fn suite_recall_gate(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> String {
    let gd = args.graph_degree;
    for node in nodes {
        let sql = format!(
            "DROP TABLE IF EXISTS benchgate_corpus; \
             CREATE TABLE benchgate_corpus AS SELECT * FROM dm; \
             CREATE INDEX benchgate_corpus_idx ON benchgate_corpus \
               USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {gd});"
        );
        if !run_capture(psql, socket_dir, node.port, &sql)
            .await
            .status_ok
        {
            return "suite_recall_gate=SKIPPED(benchgate setup failed)".to_owned();
        }
    }
    let coord_port = nodes[0].port;
    // Real lane: held-out queries from dm_queries. Synthetic lane: first 50
    // corpus rows (as before).
    let benchgate_queries_sql = if args.corpus_prefix.is_some() {
        format!(
            "DROP TABLE IF EXISTS benchgate_queries; CREATE TABLE benchgate_queries AS \
             SELECT id, source FROM dm_queries ORDER BY id LIMIT {};",
            args.queries
        )
    } else {
        "DROP TABLE IF EXISTS benchgate_queries; CREATE TABLE benchgate_queries AS SELECT id, source FROM dm WHERE id <= 50;".to_owned()
    };
    let _ = run_capture(psql, socket_dir, coord_port, &benchgate_queries_sql).await;
    let ecaz = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return "suite_recall_gate=SKIPPED(no exe)".to_owned(),
    };
    let single = run_bench_recall(&ecaz, socket_dir, coord_port, "").await;
    let multi = run_bench_recall(&ecaz, socket_dir, coord_port, roster).await;
    match (single, multi) {
        (Some(s), Some(m)) => {
            let pass = m >= s - 0.001;
            format!(
                "suite_recall_gate single={s:.4} multi={m:.4} delta={:.4} pass={pass}",
                m - s
            )
        }
        _ => "suite_recall_gate=INCONCLUSIVE(recall parse/connect failed)".to_owned(),
    }
}

/// Invoke `ecaz bench recall` against the coordinator with the given roster
/// session-GUC; parse recall@k from the comfy-table (a single-sweep row).
async fn run_bench_recall(
    ecaz: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster_val: &str,
) -> Option<f64> {
    let mut cmd = Command::new(ecaz);
    cmd.arg("--database")
        .arg("postgres")
        .arg("--host")
        .arg(socket_dir)
        .arg("--port")
        .arg(coord_port.to_string())
        .arg("bench")
        .arg("recall")
        .arg("--prefix")
        .arg("benchgate")
        .arg("--profile")
        .arg("ec_distann")
        .arg("--k")
        .arg("10")
        .arg("--sweep")
        .arg("32")
        .arg("--force-index");
    // Single-node = default (empty) roster; the GUC parser rejects an empty value,
    // so only set it for the multi-node arm.
    if !roster_val.is_empty() {
        cmd.arg("--session-guc")
            .arg(format!("ec_distann.roster={roster_val}"))
            .arg("--session-guc")
            .arg("ec_distann.local_node_id=1")
            .arg("--session-guc")
            .arg("ec_distann.epoch=1");
    }
    let out = cmd.output().await.ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let errtext = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        crate::ecaz_println!(
            "[distann-multicluster] bench recall (roster={:?}) exit={:?} stderr={}",
            !roster_val.is_empty(),
            out.status.code(),
            errtext
                .lines()
                .rev()
                .take(3)
                .collect::<Vec<_>>()
                .join(" | ")
        );
    }
    // comfy-table data row (columns: top_k/sweep, queries, recall_trials,
    // recall@k, ...): `│ 32 ┆ 50 ┆ 500 ┆ 0.5040 ┆ ...`. The left border is '│',
    // inner columns are separated by '┆'; recall@k is field index 4.
    for line in text.lines() {
        let fields: Vec<&str> = line.split(['│', '┆']).map(str::trim).collect();
        if fields.len() > 4 && fields[1].parse::<i64>().is_ok() {
            if let Ok(recall) = fields[4].parse::<f64>() {
                return Some(recall);
            }
        }
    }
    None
}

/// FR-082-AC-3 live gate: hold a single-node index scan open (AccessShareLock on
/// dm_idx) in a background transaction, and assert `ec_distann_retire_epoch` is
/// gated while it is in flight, then succeeds once it drains.
async fn retention_gate_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    args: &LocalMultinodePg18Args,
) -> bool {
    let idx = "'dm_idx'::regclass::oid";
    // Background holder: an ec_distann index scan held open ~3s via a cursor.
    let hold_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.roster=''; SET ec_distann.local_node_id=1; \
         BEGIN; \
         DECLARE c CURSOR FOR SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {}; \
         FETCH 1 FROM c; SELECT pg_sleep(3); COMMIT;",
        args.top_k
    );
    let holder = {
        let (psql, socket_dir) = (psql.to_path_buf(), socket_dir.to_path_buf());
        tokio::spawn(async move { run_capture(&psql, &socket_dir, coord_port, &hold_sql).await })
    };
    // Let the scan acquire its AccessShareLock.
    tokio::time::sleep(std::time::Duration::from_millis(900)).await;

    let gated_out = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT ec_distann_retire_epoch({idx})"),
    )
    .await;
    let gated = gated_out.contains("retention gate");

    let _ = holder.await;

    let drained_out = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT ec_distann_retire_epoch({idx})"),
    )
    .await;
    let succeeded_after_drain = !drained_out.contains("ERROR");

    // Restore a Published epoch for any downstream steps.
    let _ = run_capture(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT ec_distann_publish_epoch({idx}, 1)"),
    )
    .await;

    gated && succeeded_after_drain
}

/// FR-082-AC-5: a live record's exact-rerank result must be byte-identical after
/// real delete+VACUUM+reinsert TID churn on every node. Deleted records are
/// tombstoned by the AM's ambulkdelete (never reranked); a live record's heap TID
/// is never reclaimed, so its co-placed vector is frozen without a separate tier.
async fn frozen_vector_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster: &str,
    nodes: &[Node],
    args: &LocalMultinodePg18Args,
) -> bool {
    // Probe: row 1's multi-node top-1 (id:distance), byte-exact.
    let probe = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT id || ':' || (embedding <#> (SELECT source FROM dm WHERE id=1))::float8 \
           FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT 1;"
    );
    let baseline = capture_psql_allow_error(psql, socket_dir, coord_port, &probe).await;
    let baseline = baseline
        .lines()
        .find(|l| l.contains(':'))
        .unwrap_or("")
        .trim()
        .to_owned();
    if baseline.is_empty() {
        crate::ecaz_println!("[distann-multicluster] ac5 baseline probe empty");
        return false;
    }

    // Delete a mid range on every node, then VACUUM (triggers ambulkdelete →
    // tombstone + heap reclaim), freeing those TIDs for reuse.
    let lo = args.rows / 4;
    let hi = lo + 150;
    for node in nodes {
        let del = run_capture(
            psql,
            socket_dir,
            node.port,
            &format!("DELETE FROM dm WHERE id BETWEEN {lo} AND {hi};"),
        )
        .await;
        let vac = run_capture(psql, socket_dir, node.port, "VACUUM dm;").await;
        if !del.status_ok || !vac.status_ok {
            crate::ecaz_println!(
                "[distann-multicluster] ac5 delete/vacuum failed on node {}",
                node.node_id
            );
            return false;
        }
    }
    // Reinsert new rows on every node (may reuse the reclaimed TIDs). Use a
    // real corpus vector (correct dimension) in real mode; the reinserted vector
    // content is irrelevant to the frozen-vector check (which probes a
    // pre-existing live record), only its dimension must match the index.
    let arr = insert_vector_expr(args, "dm");
    for node in nodes {
        let ins = run_capture(
            psql,
            socket_dir,
            node.port,
            &format!(
                "INSERT INTO dm SELECT g, {arr}, encode_to_ecvector({arr}, 4, 42) FROM generate_series({lo}, {hi}) AS g;"
            ),
        )
        .await;
        if !ins.status_ok {
            crate::ecaz_println!(
                "[distann-multicluster] ac5 reinsert failed on node {}",
                node.node_id
            );
            return false;
        }
    }

    // Re-probe: row 1 (never touched) must rerank byte-identically.
    let after = capture_psql_allow_error(psql, socket_dir, coord_port, &probe).await;
    if after.contains("EC_VECTOR_MISSING") || after.contains("ERROR") {
        crate::ecaz_println!("[distann-multicluster] ac5 post-churn probe errored: {after}");
        return false;
    }
    let after = after
        .lines()
        .find(|l| l.contains(':'))
        .unwrap_or("")
        .trim()
        .to_owned();
    baseline == after
}

/// 011/020-P1: a WHERE qual on a NON-projected column (`source`) plus LIMIT.
/// Multi-node (CustomScan) must return exactly the single-node result — proving
/// the qual column is shipped for remote rows and the LIMIT applies after the
/// qual (over-fetch), not before. Returns (report line, pass).
async fn qual_correctness_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> (String, bool) {
    // Order by the id=1 vector; filter on source[1] > 0 (source is NOT selected).
    let query = format!(
        "SELECT id FROM dm WHERE source[1] > 0 \
           ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {}",
        args.top_k
    );
    let sql = format!(
        "SET enable_seqscan=off; \
         DROP TABLE IF EXISTS qc_s; DROP TABLE IF EXISTS qc_m; \
         SELECT set_config('ec_distann.roster', '', false); SET ec_distann.local_node_id=1; SET ec_distann.epoch=0; \
         CREATE TEMP TABLE qc_s AS {query}; \
         SELECT set_config('ec_distann.roster', '{roster}', false); SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         CREATE TEMP TABLE qc_m AS {query}; \
         SELECT set_config('ec_distann.roster', '', false); \
         SELECT (SELECT count(*) FROM qc_s) || ' ' || (SELECT count(*) FROM qc_m) || ' ' || \
           ((SELECT count(*) FROM (SELECT id FROM qc_s EXCEPT SELECT id FROM qc_m) x) \
          + (SELECT count(*) FROM (SELECT id FROM qc_m EXCEPT SELECT id FROM qc_s) x));"
    );
    let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
    let parsed: Vec<i64> = out
        .lines()
        .find(|l| {
            l.split_whitespace().count() == 3
                && l.split_whitespace().all(|f| f.parse::<i64>().is_ok())
        })
        .map(|l| {
            l.split_whitespace()
                .filter_map(|f| f.parse().ok())
                .collect()
        })
        .unwrap_or_default();
    if parsed.len() != 3 {
        return (
            format!(
                "qual_correctness=INCONCLUSIVE({})",
                out.lines().last().unwrap_or("").trim()
            ),
            false,
        );
    }
    let (s_n, m_n, mismatch) = (parsed[0], parsed[1], parsed[2]);
    // Pass = same count and zero id mismatch (single==multi under the qual+LIMIT).
    let pass = s_n == m_n && mismatch == 0;
    (
        format!("qual_correctness single_n={s_n} multi_n={m_n} mismatch={mismatch} pass={pass}"),
        pass,
    )
}

struct CaptureOut {
    status_ok: bool,
    stdout: String,
    stderr: String,
}

async fn run_capture(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> CaptureOut {
    let mut command = psql_base(psql, socket_dir, port);
    command
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-tAc")
        .arg(sql);
    match command.output().await {
        Ok(output) => CaptureOut {
            status_ok: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        Err(error) => CaptureOut {
            status_ok: false,
            stdout: String::new(),
            stderr: format!("spawn error: {error}"),
        },
    }
}

fn psql_base(psql: &Path, _socket_dir: &Path, port: u16) -> Command {
    let mut command = Command::new(psql);
    command
        .arg("-h")
        .arg("127.0.0.1")
        .arg("-p")
        .arg(port.to_string())
        .arg("-U")
        .arg("postgres")
        .arg("-d")
        .arg("postgres");
    command
}

fn compact_capture_error(output: &str) -> String {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" | ")
        .replace('\n', " ")
}

async fn run_psql_file(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> Result<()> {
    let mut command = psql_base(psql, socket_dir, port);
    command
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-c")
        .arg(sql)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    run_status(command).await
}

async fn capture_psql(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> Result<String> {
    let mut command = psql_base(psql, socket_dir, port);
    command
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-tAc")
        .arg(sql);
    let output = command.output().await.wrap_err("spawning psql")?;
    if !output.status.success() {
        bail!("psql failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

async fn capture_psql_allow_error(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> String {
    let mut command = psql_base(psql, socket_dir, port);
    command.arg("-tAc").arg(sql);
    match command.output().await {
        Ok(output) => {
            let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
            combined.push_str(&String::from_utf8_lossy(&output.stderr));
            combined
        }
        Err(error) => format!("psql spawn error: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestLocalMultinodeArgs {
        #[command(flatten)]
        args: LocalMultinodePg18Args,
    }

    #[test]
    fn query_slice_digest_preserves_exact_selected_line_bytes() {
        let digest = query_slice_sha256(b"a\nb\nc\nd\n", 1, 2).expect("slice hashes");
        assert_eq!(
            digest,
            "bb9ead4c391dab4c05bd498dafac47a54f8b212625f2124a911202cc6ea61d27"
        );
    }

    #[test]
    fn query_slice_digest_rejects_short_slice() {
        let error = query_slice_sha256(b"a\nb\n", 1, 2).expect_err("slice is short");
        assert!(error.to_string().contains("selected=1"));
    }

    #[test]
    fn real_setup_sql_loads_only_the_requested_query_slice() {
        let sql = real_setup_sql(
            Path::new("/staged/corpus.tsv"),
            Path::new("/staged/queries.tsv"),
            200,
            200,
            32,
            4096,
            1,
            "stitched_bfs",
            "",
        );
        assert!(sql.contains("ORDER BY id OFFSET 200 LIMIT 200"));
    }

    fn provenance(
        node_id: u32,
        port: u16,
        sha: &str,
        profile: &str,
        features: &str,
    ) -> ExtensionProvenance {
        ExtensionProvenance {
            node_id,
            port,
            git_sha: sha.to_owned(),
            build_profile: profile.to_owned(),
            features: features.to_owned(),
        }
    }

    #[test]
    fn extension_preflight_accepts_unanimous_release_nodes() {
        let observed = [
            provenance(1, 39710, "abc123", "release", "pg18"),
            provenance(2, 39711, "abc123", "release", "pg18"),
            provenance(3, 39712, "abc123", "release", "pg18"),
        ];
        let preflight = validate_extension_preflight(&observed, false).unwrap();
        assert_eq!(preflight.git_sha, "abc123");
        assert_eq!(preflight.build_profile, "release");
        assert_eq!(preflight.nodes, 3);
        assert!(!preflight.debug_override);
    }

    #[test]
    fn task167_ab_work_items_match_the_preregistered_sample_size() {
        let work = (0..TASK167_AB_TRIALS)
            .flat_map(|trial| task167_insert_trial_items(trial, TASK167_AB_ROWS_PER_TRIAL))
            .collect::<Vec<_>>();
        assert_eq!(work.len(), TASK167_AB_SAMPLE_ROWS);
        assert_eq!(work.first(), Some(&(0, 0)));
        assert_eq!(work.last(), Some(&(TASK167_AB_ROWS_PER_TRIAL - 1, 159)));
        assert_eq!(
            work.iter()
                .map(|(_, source_offset)| *source_offset)
                .collect::<HashSet<_>>()
                .len(),
            TASK167_AB_SAMPLE_ROWS,
        );
    }

    #[test]
    fn task167_synthetic_vector_sql_has_stable_component_order() {
        let expression = synthetic_unit_vector_expr("g", 4);
        assert!(expression.contains("sqrt(sum(component * component) OVER ())"));
        assert!(expression.contains("generate_series(0, 4 - 1)"));
        assert!(expression.contains("(g) * 0.017"));
        assert!(expression.contains("array_agg((component / norm)::real ORDER BY d)"));
    }

    #[test]
    fn task167_production_insert_work_does_not_require_query_stage_feature() {
        validate_query_stage_counter_feature(false, "pg18").unwrap();
    }

    #[test]
    fn task167_query_stage_counters_reject_missing_benchmark_feature() {
        let error = validate_query_stage_counter_feature(true, "pg18").unwrap_err();
        let rendered = error.to_string();
        assert!(rendered.contains("--distann-stage-counters"));
        assert!(rendered.contains("distann-head-attribution-benchmark"));
        assert!(rendered.contains("insert-work counters are collected independently"));
    }

    #[test]
    fn task167_query_stage_counters_accept_benchmark_feature() {
        validate_query_stage_counter_feature(true, "distann-head-attribution-benchmark,pg18")
            .unwrap();
    }

    #[test]
    fn task167_retry_attribution_is_explicitly_enabled_in_remote_options() {
        let roster = "1@host=127.0.0.1 port=39710 options=-cstatement_timeout=3600000;2@host=127.0.0.1 port=39711 options=-cstatement_timeout=3600000";
        let attributed = task167_retry_attribution_roster(roster).unwrap();
        assert_eq!(
            attributed,
            "1@host=127.0.0.1 port=39710 options='-cstatement_timeout=3600000 -cec_distann.debug_retry_attribution=on';2@host=127.0.0.1 port=39711 options='-cstatement_timeout=3600000 -cec_distann.debug_retry_attribution=on'"
        );
    }

    #[test]
    fn task167_distinct_exact_recall_uses_the_distinct_truth_denominator() {
        let truth = ["same", "same", "same", "b", "c", "d", "e", "f", "g", "h"];
        let predicted = ["same", "b", "c", "d", "e", "f", "g", "h", "x", "y"];
        let (recall, distinct, duplicate_slots) = task167_distinct_recall(&truth, &predicted);
        assert_eq!(distinct, 8);
        assert_eq!(duplicate_slots, 2);
        assert!((recall - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn task167_exact_recall_pins_the_production_search_operating_point() {
        let mut args = TestLocalMultinodeArgs::parse_from(["test"]).args;
        args.top_k = 10;
        args.bench_session_gucs = vec!["ec_distann.remote_statement_timeout_ms=12345".to_owned()];
        let production = BenchmarkSeedVariant {
            name: "production".to_owned(),
            strategy: "persisted_head".to_owned(),
            head_search_width: 32,
            head_seed_count: 32,
            neighbor_score_mode: "rabitq".to_owned(),
            materialization_batch_size: 10,
            owner_payload_plan_cache: false,
            beam_width: None,
            hop_rounds: None,
            traversal_replica: false,
            typed_locator: false,
            packed_payload: false,
            expanded_locator: false,
        };
        let sql = task167_search_guc_sql(&args, &production, 4, 32, 100).unwrap();
        for pinned in [
            "ec_distann.beam_width = 4",
            "ec_distann.candidate_heap_limit = 32",
            "ec_distann.hop_rounds = 100",
            "ec_distann.top_k = 10",
            "ec_distann.remote_statement_timeout_ms = 12345",
            "ec_distann.benchmark_seed_mode = 'persisted_head'",
            "ec_distann.benchmark_head_search_width = 32",
            "ec_distann.benchmark_head_seed_count = 32",
            "ec_distann.benchmark_materialization_batch_size = 10",
            "ec_distann.sharded_head_search = on",
            "ec_distann.crown_capacity = 0",
        ] {
            assert!(sql.contains(pinned), "missing pinned GUC: {pinned}");
        }
    }

    #[test]
    fn task167_heldout_regression_gate_is_per_scale_and_baseline_relative() {
        assert_eq!(task167_heldout_regression_gate(None, None).unwrap(), None);
        let gate = task167_heldout_regression_gate(Some(0.008611), Some(0.000224))
            .unwrap()
            .expect("configured gate");
        assert_eq!(gate.baseline_deficit, 0.008611);
        assert_eq!(gate.physical_sample_sd, 0.000224);
        assert!((gate.allowed_deficit - 0.009059).abs() < f64::EPSILON);

        assert!(task167_heldout_regression_gate(Some(0.008611), None).is_err());
        assert!(task167_heldout_regression_gate(None, Some(0.000224)).is_err());
        assert!(task167_heldout_regression_gate(Some(-0.1), Some(0.0)).is_err());
        assert!(task167_heldout_regression_gate(Some(0.1), Some(f64::NAN)).is_err());
    }

    #[test]
    fn task167_exact_recall_enforces_the_calibrated_band() {
        assert!(task167_exact_recall_within_allowed_deficit(
            0.992, 0.999, 0.007
        ));
        assert!(task167_exact_recall_within_allowed_deficit(
            1.0, 0.999, 0.007
        ));
        assert!(!task167_exact_recall_within_allowed_deficit(
            0.991, 0.999, 0.007
        ));
    }

    #[test]
    fn task167_quality_gate_failure_retains_the_failed_population_row() {
        let lines = vec![
            "physical_benchmark_post_insert_exact_recall population=inserted_neighborhood quality_gate_pass=true pass=true".to_owned(),
            "physical_benchmark_post_insert_exact_recall population=heldout physical_distinct_recall=0.848722 fresh_distinct_recall=0.857333 quality_gate_pass=false pass=false".to_owned(),
            "physical_benchmark_backlink_strategy_ab pass=skipped reason=candidate_default_quality_gate_failed control_mutation_excluded=true".to_owned(),
        ];
        let failure = task167_quality_gate_failure(&lines).expect("heldout gate must fail");
        assert!(failure.contains("population=heldout"));
        assert!(failure.contains("physical_distinct_recall=0.848722"));
        assert!(!failure.contains("population=inserted_neighborhood"));
        assert!(enforce_task167_quality_gate(&lines).is_err());
    }

    #[test]
    fn task167_quality_gate_accepts_two_passing_population_rows() {
        let lines = vec![
            "physical_benchmark_post_insert_exact_recall population=inserted_neighborhood quality_gate_pass=true pass=true".to_owned(),
            "physical_benchmark_post_insert_exact_recall population=heldout quality_gate_applied=false quality_gate_pass=not_applied disposition=disclosed_baseline_characteristic pass=true".to_owned(),
        ];
        assert!(task167_quality_gate_failure(&lines).is_none());
        enforce_task167_quality_gate(&lines).unwrap();
    }

    #[test]
    fn extension_preflight_rejects_debug_without_override() {
        let error = validate_extension_preflight(
            &[
                provenance(1, 39710, "abc123", "debug", "pg18"),
                provenance(2, 39711, "abc123", "debug", "pg18"),
            ],
            false,
        )
        .unwrap_err();
        let rendered = error.to_string();
        assert!(rendered.contains("node 1 port 39710"));
        assert!(rendered.contains("observed abc123/debug"));
        assert!(rendered.contains("allow_debug_extension=true"));
    }

    #[test]
    fn extension_preflight_rejects_pg_test_feature_without_override() {
        let error = validate_extension_preflight(
            &[
                provenance(
                    1,
                    39710,
                    "abc123",
                    "release",
                    "distann-head-attribution-benchmark,pg-test,pg18",
                ),
                provenance(
                    2,
                    39711,
                    "abc123",
                    "release",
                    "distann-head-attribution-benchmark,pg-test,pg18",
                ),
            ],
            false,
        )
        .unwrap_err();
        let rendered = error.to_string();
        assert!(rendered.contains("pg-test"));
        assert!(rendered.contains("--no-default-features --features pg18"));
    }

    #[test]
    fn extension_preflight_rejects_mixed_node_provenance_even_with_override() {
        let error = validate_extension_preflight(
            &[
                provenance(1, 39710, "abc123", "debug", "pg18"),
                provenance(2, 39711, "abc123", "release", "pg18"),
            ],
            true,
        )
        .unwrap_err();
        let rendered = error.to_string();
        assert!(rendered.contains("node 2 port 39711"));
        assert!(rendered.contains("expected abc123/debug"));
        assert!(rendered.contains("observed abc123/release"));
    }

    #[test]
    fn extension_preflight_allows_unanimous_debug_with_explicit_override() {
        let observed = [
            provenance(1, 39710, "abc123", "debug", "pg18"),
            provenance(2, 39711, "abc123", "debug", "pg18"),
        ];
        let preflight = validate_extension_preflight(&observed, true).unwrap();
        assert_eq!(preflight.build_profile, "debug");
        assert!(preflight.debug_override);
    }

    fn seed_variant(name: &str, neighbor_score_mode: &str) -> BenchmarkSeedVariant {
        BenchmarkSeedVariant {
            name: name.to_owned(),
            strategy: "persisted_head".to_owned(),
            head_search_width: 32,
            head_seed_count: 32,
            neighbor_score_mode: neighbor_score_mode.to_owned(),
            materialization_batch_size: 0,
            owner_payload_plan_cache: false,
            beam_width: None,
            hop_rounds: None,
            traversal_replica: false,
            typed_locator: false,
            packed_payload: false,
            expanded_locator: false,
        }
    }

    #[test]
    fn traversal_reconciliation_reads_exact_stage_labels() {
        let rows = [
            "command=latency stage=remote_expand mean_ms=6.25",
            "command=latency stage=traversal_owner_service mean_ms=2.00",
        ];
        assert_eq!(
            attribution_stage_mean(&rows, "remote_expand").unwrap(),
            6.25
        );
        assert_eq!(
            attribution_stage_mean(&rows, "traversal_owner_service").unwrap(),
            2.0
        );
        assert!(attribution_stage_mean(&rows, "remote").is_err());
    }

    #[test]
    fn materialization_variant_parser_is_backward_compatible() {
        let variants = parse_benchmark_seed_variants(&[
            "eager:persisted_head:32:32:rabitq".to_owned(),
            "lazy:persisted_head:32:32:rabitq:10".to_owned(),
        ])
        .expect("variants parse");
        assert_eq!(variants[0].materialization_batch_size, 10);
        assert_eq!(variants[1].materialization_batch_size, 10);
        let eager =
            parse_benchmark_seed_variants(&["eager:persisted_head:32:32:rabitq:0".to_owned()])
                .expect("explicit eager variant parses");
        assert_eq!(eager[0].materialization_batch_size, 0);
    }

    #[test]
    fn production_schema_cache_has_no_variant_setting() {
        let variant = seed_variant("production", "rabitq");
        assert!(!materialization_variant_settings_sql(&variant)
            .contains("benchmark_owner_validation_cache"));
    }

    #[test]
    fn owner_plan_and_fixed_work_variant_controls_are_explicit() {
        let variants = parse_benchmark_seed_variants(&[
            "plan-off:persisted_head:32:32:rabitq:10:off:4:100".to_owned(),
            "plan-on:persisted_head:32:32:rabitq:10:on:8:50:on".to_owned(),
        ])
        .expect("owner plan and traversal variants parse");
        assert!(!variants[0].owner_payload_plan_cache);
        assert!(variants[1].owner_payload_plan_cache);
        assert_eq!(variants[0].beam_width, Some(4));
        assert_eq!(variants[0].hop_rounds, Some(100));
        assert_eq!(variants[1].beam_width, Some(8));
        assert_eq!(variants[1].hop_rounds, Some(50));
        assert!(variants[1].traversal_replica);
    }

    #[test]
    fn eager_materialization_control_is_forwarded_as_explicit_zero() {
        let mut eager_args = Vec::new();
        append_materialization_benchmark_guc(&mut eager_args, "physical", 0);
        assert_eq!(
            eager_args,
            [
                "--session-guc",
                "ec_distann.benchmark_materialization_batch_size=0"
            ]
        );

        let mut single_args = Vec::new();
        append_materialization_benchmark_guc(&mut single_args, "single", 0);
        assert!(single_args.is_empty());
    }

    #[test]
    fn materialization_semantic_sql_projects_null_and_varlena_evidence() {
        let sql = materialization_semantic_sql(
            "physical_corpus",
            "physical_queries",
            "payload_note IS NOT NULL AND id % 3 = 1",
            10,
            0,
        );
        assert!(sql.contains("payload_note IS NULL AS payload_null"));
        assert!(sql.contains("md5(payload_note)"));
        assert!(sql.contains("octet_length(payload_note)"));
        assert!(sql.contains("pg_column_compression(payload_note)"));
        assert!(sql.contains("attstorage::text"));
        assert!(sql.contains("WHERE payload_note IS NOT NULL AND id % 3 = 1"));
        assert!(sql.contains("LIMIT 10"));
    }

    #[test]
    fn same_seed_digest_accepts_equal_neighbor_score_arms() {
        let mut digests = std::collections::HashMap::new();
        assert_eq!(
            register_same_seed_digest(&mut digests, &seed_variant("rabitq", "rabitq"), "aaaa")
                .unwrap(),
            None
        );
        assert_eq!(
            register_same_seed_digest(
                &mut digests,
                &seed_variant("exact", "exact_neighbor"),
                "aaaa"
            )
            .unwrap(),
            Some("rabitq".to_owned())
        );
    }

    #[test]
    fn same_seed_digest_rejects_different_neighbor_score_arms() {
        let mut digests = std::collections::HashMap::new();
        register_same_seed_digest(&mut digests, &seed_variant("rabitq", "rabitq"), "aaaa").unwrap();
        let error = register_same_seed_digest(
            &mut digests,
            &seed_variant("exact", "exact_neighbor"),
            "bbbb",
        )
        .unwrap_err();
        assert!(error.to_string().contains("selected different seed IDs"));
    }
}
