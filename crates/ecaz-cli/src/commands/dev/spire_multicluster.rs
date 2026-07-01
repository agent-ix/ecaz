use clap::{Args, Subcommand, ValueEnum};
use color_eyre::eyre::{bail, eyre, Context, Result};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::Command;

use super::support::{
    find_pgrx_install, repo_root, resolve_pgrx_home, run_status, DEFAULT_PG_MAJOR,
};

#[derive(Subcommand, Debug)]
pub enum SpireMulticlusterCommand {
    /// Run the PG18 one-coordinator/one-remote baseline smoke fixture.
    #[command(name = "smoke-pg18")]
    SmokePg18(SmokePg18Args),
    /// Run the PG18 one-coordinator/one-remote CustomScan read fixture.
    #[command(name = "customscan-read-pg18")]
    CustomScanReadPg18(CustomScanReadPg18Args),
    /// Run the PG18 INSERT followed by CustomScan read fixture.
    #[command(name = "insert-read-after-customscan-pg18")]
    InsertReadAfterCustomScanPg18(InsertReadAfterCustomScanPg18Args),
    /// Run the PG18 one-coordinator/two-remote transport-overlap fixture.
    TransportOverlapPg18(TransportOverlapPg18Args),
    /// Run a PG18 Stage E fault-matrix fixture case.
    FaultPg18(StageEFaultPg18Args),
    /// Run a PG18 Stage E lifecycle-matrix fixture case.
    LifecyclePg18(StageELifecyclePg18Args),
    /// Run the PG18 one-coordinator/three-worker local distributed benchmark lane.
    LocalMultinodePg18(LocalMultinodePg18Args),
}

impl SpireMulticlusterCommand {
    pub async fn run(self, _database: &str) -> Result<()> {
        match self {
            SpireMulticlusterCommand::SmokePg18(args) => run_smoke_pg18(args).await,
            SpireMulticlusterCommand::CustomScanReadPg18(args) => {
                run_customscan_read_pg18(args).await
            }
            SpireMulticlusterCommand::InsertReadAfterCustomScanPg18(args) => {
                run_insert_read_after_customscan_pg18(args).await
            }
            SpireMulticlusterCommand::TransportOverlapPg18(args) => {
                run_transport_overlap_pg18(args).await
            }
            SpireMulticlusterCommand::FaultPg18(args) => run_stage_e_fault_pg18(args).await,
            SpireMulticlusterCommand::LifecyclePg18(args) => run_stage_e_lifecycle_pg18(args).await,
            SpireMulticlusterCommand::LocalMultinodePg18(args) => {
                run_local_multinode_pg18(args).await
            }
        }
    }
}

#[derive(Args, Debug)]
pub struct SmokePg18Args {
    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Explicit PostgreSQL bin directory. Defaults to the newest matching pgrx install.
    #[arg(long)]
    pgbin: Option<PathBuf>,

    /// Store fixture and PostgreSQL logs in a review packet artifact directory.
    #[arg(long)]
    artifact_dir: Option<PathBuf>,

    /// Run directory. Defaults to the script-owned target/ path.
    #[arg(long)]
    run_dir: Option<PathBuf>,

    /// Store PostgreSQL logs outside the run directory.
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Tee fixture stdout/stderr to this file.
    #[arg(long)]
    smoke_log: Option<PathBuf>,

    /// Coordinator PostgreSQL port.
    #[arg(long)]
    coord_port: Option<u16>,

    /// Remote PostgreSQL port.
    #[arg(long)]
    remote_port: Option<u16>,

    /// Run id used in the default run directory.
    #[arg(long)]
    run_id: Option<String>,

    /// Skip cargo pgrx install before starting fixture clusters.
    #[arg(long)]
    skip_install: bool,
}

#[derive(Args, Debug)]
pub struct CustomScanReadPg18Args {
    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Explicit PostgreSQL bin directory. Defaults to the newest matching pgrx install.
    #[arg(long)]
    pgbin: Option<PathBuf>,

    /// Store fixture and PostgreSQL logs in a review packet artifact directory.
    #[arg(long)]
    artifact_dir: Option<PathBuf>,

    /// Run directory. Defaults to the script-owned target/ path.
    #[arg(long)]
    run_dir: Option<PathBuf>,

    /// Store PostgreSQL logs outside the run directory.
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Tee fixture stdout/stderr to this file.
    #[arg(long)]
    smoke_log: Option<PathBuf>,

    /// Coordinator PostgreSQL port.
    #[arg(long)]
    coord_port: Option<u16>,

    /// Remote PostgreSQL port.
    #[arg(long)]
    remote_port: Option<u16>,

    /// Run id used in the default run directory.
    #[arg(long)]
    run_id: Option<String>,

    /// Skip cargo pgrx install before starting fixture clusters.
    #[arg(long)]
    skip_install: bool,
}

#[derive(Args, Debug)]
pub struct InsertReadAfterCustomScanPg18Args {
    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Explicit PostgreSQL bin directory. Defaults to the newest matching pgrx install.
    #[arg(long)]
    pgbin: Option<PathBuf>,

    /// Store fixture and PostgreSQL logs in a review packet artifact directory.
    #[arg(long)]
    artifact_dir: Option<PathBuf>,

    /// Run directory. Defaults to the script-owned target/ path.
    #[arg(long)]
    run_dir: Option<PathBuf>,

    /// Store PostgreSQL logs outside the run directory.
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Tee fixture stdout/stderr to this file.
    #[arg(long)]
    smoke_log: Option<PathBuf>,

    /// Coordinator PostgreSQL port.
    #[arg(long)]
    coord_port: Option<u16>,

    /// Remote PostgreSQL port.
    #[arg(long)]
    remote_port: Option<u16>,

    /// Insert path to exercise.
    #[arg(long, value_enum, default_value_t = InsertReadMode::Helper)]
    insert_mode: InsertReadMode,

    /// Run id used in the default run directory.
    #[arg(long)]
    run_id: Option<String>,

    /// Skip cargo pgrx install before starting fixture clusters.
    #[arg(long)]
    skip_install: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum InsertReadMode {
    Helper,
    Trigger,
}

impl InsertReadMode {
    fn as_script_value(self) -> &'static str {
        match self {
            Self::Helper => "helper",
            Self::Trigger => "trigger",
        }
    }
}

impl std::fmt::Display for InsertReadMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_script_value())
    }
}

#[derive(Args, Debug)]
pub struct TransportOverlapPg18Args {
    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Explicit PostgreSQL bin directory. Defaults to the newest matching pgrx install.
    #[arg(long)]
    pgbin: Option<PathBuf>,

    /// Store fixture and PostgreSQL logs in a review packet artifact directory.
    #[arg(long)]
    artifact_dir: Option<PathBuf>,

    /// Run directory. Defaults to the script-owned target/ path.
    #[arg(long)]
    run_dir: Option<PathBuf>,

    /// Store PostgreSQL logs outside the run directory.
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Tee fixture stdout/stderr to this file.
    #[arg(long)]
    smoke_log: Option<PathBuf>,

    /// Coordinator PostgreSQL port.
    #[arg(long)]
    coord_port: Option<u16>,

    /// Fast remote PostgreSQL port.
    #[arg(long)]
    remote_fast_port: Option<u16>,

    /// Slow remote PostgreSQL port.
    #[arg(long)]
    remote_slow_port: Option<u16>,

    /// Run id used in the default run directory.
    #[arg(long)]
    run_id: Option<String>,

    /// Skip cargo pgrx install before starting fixture clusters.
    #[arg(long)]
    skip_install: bool,
}

#[derive(Args, Debug)]
pub struct StageEFaultPg18Args {
    /// Stage E fault matrix case to run.
    #[arg(long, value_parser = parse_stage_e_fault_case)]
    case: StageEFaultCase,

    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Explicit PostgreSQL bin directory. Defaults to the newest matching pgrx install.
    #[arg(long)]
    pgbin: Option<PathBuf>,

    /// Store fixture and PostgreSQL logs in a review packet artifact directory.
    #[arg(long)]
    artifact_dir: Option<PathBuf>,

    /// Run directory. Defaults to the script-owned target/ path.
    #[arg(long)]
    run_dir: Option<PathBuf>,

    /// Store PostgreSQL logs outside the run directory.
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Tee fixture stdout/stderr to this file.
    #[arg(long)]
    smoke_log: Option<PathBuf>,

    /// Coordinator PostgreSQL port.
    #[arg(long)]
    coord_port: Option<u16>,

    /// Ready remote PostgreSQL port.
    #[arg(long)]
    remote_ready_port: Option<u16>,

    /// Run id used in the default run directory.
    #[arg(long)]
    run_id: Option<String>,

    /// Skip cargo pgrx install before starting fixture clusters.
    #[arg(long)]
    skip_install: bool,
}

#[derive(Args, Debug)]
pub struct StageELifecyclePg18Args {
    /// Stage E lifecycle matrix case to run.
    #[arg(long, value_parser = parse_stage_e_lifecycle_case)]
    case: StageELifecycleCase,

    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Explicit PostgreSQL bin directory. Defaults to the newest matching pgrx install.
    #[arg(long)]
    pgbin: Option<PathBuf>,

    /// Store fixture and PostgreSQL logs in a review packet artifact directory.
    #[arg(long)]
    artifact_dir: Option<PathBuf>,

    /// Run directory. Defaults to the script-owned target/ path.
    #[arg(long)]
    run_dir: Option<PathBuf>,

    /// Store PostgreSQL logs outside the run directory.
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Tee fixture stdout/stderr to this file.
    #[arg(long)]
    smoke_log: Option<PathBuf>,

    /// Coordinator PostgreSQL port.
    #[arg(long)]
    coord_port: Option<u16>,

    /// Ready remote PostgreSQL port.
    #[arg(long)]
    remote_ready_port: Option<u16>,

    /// Run id used in the default run directory.
    #[arg(long)]
    run_id: Option<String>,

    /// Skip cargo pgrx install before starting fixture clusters.
    #[arg(long)]
    skip_install: bool,
}

#[derive(Args, Debug)]
pub struct LocalMultinodePg18Args {
    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Explicit PostgreSQL bin directory. Defaults to the newest matching pgrx install.
    #[arg(long)]
    pgbin: Option<PathBuf>,

    /// Store harness, PostgreSQL, SPIRE, and suite logs in a packet artifact directory.
    #[arg(long)]
    artifact_dir: Option<PathBuf>,

    /// Run directory. Defaults to the script-owned target/ path.
    #[arg(long)]
    run_dir: Option<PathBuf>,

    /// Tee harness stdout/stderr to this file.
    #[arg(long)]
    smoke_log: Option<PathBuf>,

    /// Coordinator PostgreSQL port.
    #[arg(long)]
    coord_port: Option<u16>,

    /// First worker PostgreSQL port.
    #[arg(long)]
    remote1_port: Option<u16>,

    /// Second worker PostgreSQL port.
    #[arg(long)]
    remote2_port: Option<u16>,

    /// Third worker PostgreSQL port.
    #[arg(long)]
    remote3_port: Option<u16>,

    /// Run id used in the default run directory.
    #[arg(long)]
    run_id: Option<String>,

    /// Local load tier: correctness or representative.
    #[arg(long)]
    tier: Option<String>,

    /// Corpus prefix for representative/local-real runs.
    #[arg(long)]
    prefix: Option<String>,

    /// Prepared corpus basename prefix for representative tier.
    #[arg(long)]
    prepared_prefix: Option<String>,

    /// Directory containing prepared corpus/query/manifest files.
    #[arg(long)]
    prepared_dir: Option<PathBuf>,

    /// Storage format for coordinator and remote ec_spire indexes.
    #[arg(long)]
    storage_format: Option<String>,

    /// Coordinator index name.
    #[arg(long)]
    coord_index: Option<String>,

    /// Remote index name.
    #[arg(long)]
    remote_index: Option<String>,

    /// Reloption applied to both coordinator and remote ec_spire indexes.
    #[arg(long = "reloption")]
    reloptions: Vec<String>,

    /// Reloption applied only to the coordinator ec_spire index.
    #[arg(long = "coord-reloption")]
    coord_reloptions: Vec<String>,

    /// Reloption applied only to remote ec_spire indexes.
    #[arg(long = "remote-reloption")]
    remote_reloptions: Vec<String>,

    /// Top-k for the packet-local bench suite.
    #[arg(long)]
    bench_top_k: Option<u16>,

    /// Query count for the packet-local bench suite.
    #[arg(long)]
    bench_queries_limit: Option<usize>,

    /// Comma-separated nprobe sweep for the packet-local bench suite.
    #[arg(long)]
    bench_sweep: Option<String>,

    /// Comma-separated nprobe sweep for the rowcap step.
    #[arg(long)]
    bench_rowcap_sweep: Option<String>,

    /// Skip rowcap production-read bench steps.
    #[arg(long)]
    skip_bench_rowcap: bool,

    /// Local corpus TSV for exact truth in bench spire-pipeline.
    #[arg(long)]
    bench_truth_corpus_file: Option<PathBuf>,

    /// Comma-separated payload projection columns for production-read query metrics.
    #[arg(long, value_delimiter = ',')]
    bench_query_metric_projection_columns: Vec<String>,

    /// Extra session GUCs for packet-local production-read bench suite steps, as name=value.
    #[arg(long = "bench-session-guc")]
    bench_session_gucs: Vec<String>,

    /// Extra production-read bench variant as name=<slug>;projection=id,source;guc=name=value.
    #[arg(long = "bench-production-read-variant")]
    bench_production_read_variants: Vec<String>,

    /// Skip the packet-local bench suite step.
    #[arg(long)]
    skip_bench_suite: bool,

    /// Skip correctness-only pooling/fault drills.
    #[arg(long)]
    skip_fault_drills: bool,

    /// Skip cargo pgrx install before starting fixture clusters.
    #[arg(long)]
    skip_install: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StageEFaultCase {
    ConnectionResetMidBatch,
    EpochMismatch,
    FingerprintMismatch,
    LocalCancel,
    LocalStatementTimeout,
    MissingOrReindexedRemoteIndex,
    RemoteBackendTermination,
    RemoteOom,
    RemoteStatementTimeout,
    VersionSkew,
    SimulatedNetworkPartition,
}

impl StageEFaultCase {
    fn as_matrix_key(self) -> &'static str {
        match self {
            StageEFaultCase::ConnectionResetMidBatch => "connection_reset_mid_batch",
            StageEFaultCase::EpochMismatch => "epoch_mismatch",
            StageEFaultCase::FingerprintMismatch => "fingerprint_mismatch",
            StageEFaultCase::LocalCancel => "local_cancel",
            StageEFaultCase::LocalStatementTimeout => "local_statement_timeout",
            StageEFaultCase::MissingOrReindexedRemoteIndex => "missing_or_reindexed_remote_index",
            StageEFaultCase::RemoteBackendTermination => "remote_backend_termination",
            StageEFaultCase::RemoteOom => "remote_oom",
            StageEFaultCase::RemoteStatementTimeout => "remote_statement_timeout",
            StageEFaultCase::VersionSkew => "version_skew",
            StageEFaultCase::SimulatedNetworkPartition => "simulated_network_partition",
        }
    }

    fn script_name(self) -> &'static str {
        match self {
            StageEFaultCase::EpochMismatch | StageEFaultCase::VersionSkew => {
                "scripts/run_spire_multicluster_stage_e_predispatch_fault_pg18.sh"
            }
            StageEFaultCase::FingerprintMismatch
            | StageEFaultCase::MissingOrReindexedRemoteIndex => {
                "scripts/run_spire_multicluster_stage_e_candidate_receive_fault_pg18.sh"
            }
            StageEFaultCase::LocalCancel
            | StageEFaultCase::ConnectionResetMidBatch
            | StageEFaultCase::LocalStatementTimeout
            | StageEFaultCase::RemoteBackendTermination
            | StageEFaultCase::RemoteOom
            | StageEFaultCase::RemoteStatementTimeout => {
                "scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh"
            }
            StageEFaultCase::SimulatedNetworkPartition => {
                "scripts/run_spire_multicluster_stage_e_network_partition_pg18.sh"
            }
        }
    }
}

fn parse_stage_e_fault_case(value: &str) -> std::result::Result<StageEFaultCase, String> {
    match value {
        "connection_reset_mid_batch" | "connection-reset-mid-batch" => {
            Ok(StageEFaultCase::ConnectionResetMidBatch)
        }
        "epoch_mismatch" | "epoch-mismatch" => Ok(StageEFaultCase::EpochMismatch),
        "fingerprint_mismatch" | "fingerprint-mismatch" => {
            Ok(StageEFaultCase::FingerprintMismatch)
        }
        "local_cancel" | "local-cancel" => Ok(StageEFaultCase::LocalCancel),
        "local_statement_timeout" | "local-statement-timeout" => {
            Ok(StageEFaultCase::LocalStatementTimeout)
        }
        "version_skew" | "version-skew" => Ok(StageEFaultCase::VersionSkew),
        "missing_or_reindexed_remote_index" | "missing-or-reindexed-remote-index" => {
            Ok(StageEFaultCase::MissingOrReindexedRemoteIndex)
        }
        "remote_statement_timeout" | "remote-statement-timeout" => {
            Ok(StageEFaultCase::RemoteStatementTimeout)
        }
        "remote_backend_termination" | "remote-backend-termination" => {
            Ok(StageEFaultCase::RemoteBackendTermination)
        }
        "remote_oom" | "remote-oom" => Ok(StageEFaultCase::RemoteOom),
        "simulated_network_partition" | "simulated-network-partition" => {
            Ok(StageEFaultCase::SimulatedNetworkPartition)
        }
        other => Err(format!(
            "unsupported Stage E fault case {other:?}; supported: connection_reset_mid_batch, epoch_mismatch, fingerprint_mismatch, local_cancel, local_statement_timeout, missing_or_reindexed_remote_index, remote_backend_termination, remote_oom, remote_statement_timeout, simulated_network_partition, version_skew"
        )),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StageELifecycleCase {
    CreateIndexConcurrentlyMissingDescriptor,
    CreateIndexConcurrentlyNewDescriptor,
    DropRemoteIndexBeforeFanout,
    DropRemoteIndexInFlight,
    ReindexRemoteIndexBeforeFanout,
    ReindexRemoteIndexInFlight,
}

impl StageELifecycleCase {
    fn as_matrix_key(self) -> &'static str {
        match self {
            StageELifecycleCase::CreateIndexConcurrentlyMissingDescriptor => {
                "create_index_concurrently_missing_descriptor"
            }
            StageELifecycleCase::CreateIndexConcurrentlyNewDescriptor => {
                "create_index_concurrently_new_descriptor"
            }
            StageELifecycleCase::DropRemoteIndexBeforeFanout => "drop_remote_index_before_fanout",
            StageELifecycleCase::DropRemoteIndexInFlight => "drop_remote_index_in_flight",
            StageELifecycleCase::ReindexRemoteIndexBeforeFanout => {
                "reindex_remote_index_before_fanout"
            }
            StageELifecycleCase::ReindexRemoteIndexInFlight => "reindex_remote_index_in_flight",
        }
    }

    fn script_name(self) -> &'static str {
        "scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh"
    }
}

fn parse_stage_e_lifecycle_case(value: &str) -> std::result::Result<StageELifecycleCase, String> {
    match value {
        "create_index_concurrently_missing_descriptor"
        | "create-index-concurrently-missing-descriptor" => {
            Ok(StageELifecycleCase::CreateIndexConcurrentlyMissingDescriptor)
        }
        "create_index_concurrently_new_descriptor"
        | "create-index-concurrently-new-descriptor" => {
            Ok(StageELifecycleCase::CreateIndexConcurrentlyNewDescriptor)
        }
        "drop_remote_index_before_fanout" | "drop-remote-index-before-fanout" => {
            Ok(StageELifecycleCase::DropRemoteIndexBeforeFanout)
        }
        "drop_remote_index_in_flight" | "drop-remote-index-in-flight" => {
            Ok(StageELifecycleCase::DropRemoteIndexInFlight)
        }
        "reindex_remote_index_before_fanout" | "reindex-remote-index-before-fanout" => {
            Ok(StageELifecycleCase::ReindexRemoteIndexBeforeFanout)
        }
        "reindex_remote_index_in_flight" | "reindex-remote-index-in-flight" => {
            Ok(StageELifecycleCase::ReindexRemoteIndexInFlight)
        }
        other => Err(format!(
            "unsupported Stage E lifecycle case {other:?}; supported: create_index_concurrently_missing_descriptor, create_index_concurrently_new_descriptor, drop_remote_index_before_fanout, drop_remote_index_in_flight, reindex_remote_index_before_fanout, reindex_remote_index_in_flight"
        )),
    }
}

async fn run_smoke_pg18(args: SmokePg18Args) -> Result<()> {
    if args.pg != 18 {
        bail!("smoke-pg18 requires --pg 18, got {}", args.pg);
    }
    let repo_root = repo_root()?;
    let pgbin = match args.pgbin.clone() {
        Some(path) => path,
        None => {
            let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
            find_pgrx_install(args.pg, &pgrx_home)?.bin_dir
        }
    };
    let script = repo_root.join("scripts/run_spire_multicluster_pg18_smoke.sh");
    if !script.is_file() {
        bail!(
            "SPIRE PG18 smoke fixture script is missing: {}",
            script.display()
        );
    }

    crate::ecaz_println!("[spire-multicluster] repo={}", repo_root.display());
    crate::ecaz_println!("[spire-multicluster] pgbin={}", pgbin.display());
    if let Some(artifact_dir) = &args.artifact_dir {
        crate::ecaz_println!(
            "[spire-multicluster] artifact_dir={}",
            artifact_dir.display()
        );
    }

    let mut command = Command::new("bash");
    command
        .arg(&script)
        .arg("--pgbin")
        .arg(&pgbin)
        .current_dir(&repo_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    push_path_arg(&mut command, "--artifact-dir", args.artifact_dir.as_ref());
    push_path_arg(&mut command, "--run-dir", args.run_dir.as_ref());
    push_path_arg(&mut command, "--log-dir", args.log_dir.as_ref());
    push_path_arg(&mut command, "--smoke-log", args.smoke_log.as_ref());
    push_u16_arg(&mut command, "--coord-port", args.coord_port);
    push_u16_arg(&mut command, "--remote-port", args.remote_port);
    if let Some(run_id) = args.run_id {
        command.arg("--run-id").arg(run_id);
    }
    if args.skip_install {
        command.arg("--skip-install");
    }

    run_status(command)
        .await
        .wrap_err("running SPIRE PG18 multicluster smoke fixture")
}

async fn run_customscan_read_pg18(args: CustomScanReadPg18Args) -> Result<()> {
    if args.pg != 18 {
        bail!("customscan-read-pg18 requires --pg 18, got {}", args.pg);
    }
    let repo_root = repo_root()?;
    let pgbin = match args.pgbin.as_ref() {
        Some(path) => path.clone(),
        None => {
            let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
            find_pgrx_install(args.pg, &pgrx_home)?.bin_dir
        }
    };
    let script = repo_root.join("scripts/run_spire_multicluster_customscan_read_pg18.sh");
    if !script.is_file() {
        bail!(
            "SPIRE CustomScan read fixture script is missing: {}",
            script.display()
        );
    }

    crate::ecaz_println!("[spire-multicluster] repo={}", repo_root.display());
    crate::ecaz_println!("[spire-multicluster] pgbin={}", pgbin.display());
    if let Some(artifact_dir) = &args.artifact_dir {
        crate::ecaz_println!(
            "[spire-multicluster] artifact_dir={}",
            artifact_dir.display()
        );
    }

    let mut command = Command::new("bash");
    command
        .arg(&script)
        .arg("--pgbin")
        .arg(&pgbin)
        .current_dir(&repo_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    push_path_arg(&mut command, "--artifact-dir", args.artifact_dir.as_ref());
    push_path_arg(&mut command, "--run-dir", args.run_dir.as_ref());
    push_path_arg(&mut command, "--log-dir", args.log_dir.as_ref());
    push_path_arg(&mut command, "--smoke-log", args.smoke_log.as_ref());
    push_u16_arg(&mut command, "--coord-port", args.coord_port);
    push_u16_arg(&mut command, "--remote-port", args.remote_port);
    if let Some(run_id) = args.run_id {
        command.arg("--run-id").arg(run_id);
    }
    if args.skip_install {
        command.arg("--skip-install");
    }

    run_status(command)
        .await
        .wrap_err("running SPIRE PG18 CustomScan read fixture")
}

async fn run_insert_read_after_customscan_pg18(
    args: InsertReadAfterCustomScanPg18Args,
) -> Result<()> {
    if args.pg != 18 {
        bail!(
            "insert-read-after-customscan-pg18 requires --pg 18, got {}",
            args.pg
        );
    }
    let repo_root = repo_root()?;
    let pgbin = match args.pgbin.as_ref() {
        Some(path) => path.clone(),
        None => {
            let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
            find_pgrx_install(args.pg, &pgrx_home)?.bin_dir
        }
    };
    let script =
        repo_root.join("scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh");
    if !script.is_file() {
        bail!(
            "SPIRE INSERT/read-after-CustomScan fixture script is missing: {}",
            script.display()
        );
    }

    crate::ecaz_println!("[spire-multicluster] repo={}", repo_root.display());
    crate::ecaz_println!("[spire-multicluster] pgbin={}", pgbin.display());
    crate::ecaz_println!(
        "[spire-multicluster] insert_mode={}",
        args.insert_mode.as_script_value()
    );
    if let Some(artifact_dir) = &args.artifact_dir {
        crate::ecaz_println!(
            "[spire-multicluster] artifact_dir={}",
            artifact_dir.display()
        );
    }

    let mut command = Command::new("bash");
    command
        .arg(&script)
        .arg("--pgbin")
        .arg(&pgbin)
        .arg("--insert-mode")
        .arg(args.insert_mode.as_script_value())
        .current_dir(&repo_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    push_path_arg(&mut command, "--artifact-dir", args.artifact_dir.as_ref());
    push_path_arg(&mut command, "--run-dir", args.run_dir.as_ref());
    push_path_arg(&mut command, "--log-dir", args.log_dir.as_ref());
    push_path_arg(&mut command, "--smoke-log", args.smoke_log.as_ref());
    push_u16_arg(&mut command, "--coord-port", args.coord_port);
    push_u16_arg(&mut command, "--remote-port", args.remote_port);
    if let Some(run_id) = args.run_id {
        command.arg("--run-id").arg(run_id);
    }
    if args.skip_install {
        command.arg("--skip-install");
    }

    run_status(command)
        .await
        .wrap_err("running SPIRE PG18 INSERT/read-after-CustomScan fixture")
}

async fn run_transport_overlap_pg18(args: TransportOverlapPg18Args) -> Result<()> {
    if args.pg != 18 {
        bail!("transport-overlap-pg18 requires --pg 18, got {}", args.pg);
    }
    let repo_root = repo_root()?;
    let pgbin = match args.pgbin {
        Some(path) => path,
        None => {
            let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
            find_pgrx_install(args.pg, &pgrx_home)?.bin_dir
        }
    };
    let script = repo_root.join("scripts/run_spire_multicluster_transport_overlap_pg18.sh");
    if !script.is_file() {
        bail!(
            "SPIRE multicluster fixture script is missing: {}",
            script.display()
        );
    }

    crate::ecaz_println!("[spire-multicluster] repo={}", repo_root.display());
    crate::ecaz_println!("[spire-multicluster] pgbin={}", pgbin.display());
    if let Some(artifact_dir) = &args.artifact_dir {
        crate::ecaz_println!(
            "[spire-multicluster] artifact_dir={}",
            artifact_dir.display()
        );
    }

    let mut command = Command::new("bash");
    command
        .arg(&script)
        .arg("--pgbin")
        .arg(&pgbin)
        .current_dir(&repo_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    push_path_arg(&mut command, "--artifact-dir", args.artifact_dir.as_ref());
    push_path_arg(&mut command, "--run-dir", args.run_dir.as_ref());
    push_path_arg(&mut command, "--log-dir", args.log_dir.as_ref());
    push_path_arg(&mut command, "--smoke-log", args.smoke_log.as_ref());
    push_u16_arg(&mut command, "--coord-port", args.coord_port);
    push_u16_arg(&mut command, "--remote-fast-port", args.remote_fast_port);
    push_u16_arg(&mut command, "--remote-slow-port", args.remote_slow_port);
    if let Some(run_id) = args.run_id {
        command.arg("--run-id").arg(run_id);
    }
    if args.skip_install {
        command.arg("--skip-install");
    }

    run_status(command)
        .await
        .wrap_err("running SPIRE PG18 multicluster transport-overlap fixture")
}

async fn run_stage_e_fault_pg18(args: StageEFaultPg18Args) -> Result<()> {
    if args.pg != 18 {
        bail!("fault-pg18 requires --pg 18, got {}", args.pg);
    }
    let repo_root = repo_root()?;
    let pgbin = match args.pgbin {
        Some(path) => path,
        None => {
            let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
            find_pgrx_install(args.pg, &pgrx_home)?.bin_dir
        }
    };
    let script = repo_root.join(args.case.script_name());
    if !script.is_file() {
        bail!(
            "SPIRE Stage E fault fixture script is missing: {}",
            script.display()
        );
    }

    crate::ecaz_println!("[spire-multicluster] repo={}", repo_root.display());
    crate::ecaz_println!("[spire-multicluster] pgbin={}", pgbin.display());
    crate::ecaz_println!(
        "[spire-multicluster] fault_case={}",
        args.case.as_matrix_key()
    );
    if let Some(artifact_dir) = &args.artifact_dir {
        crate::ecaz_println!(
            "[spire-multicluster] artifact_dir={}",
            artifact_dir.display()
        );
    }

    let mut command = Command::new("bash");
    command
        .arg(&script)
        .arg("--case")
        .arg(args.case.as_matrix_key())
        .arg("--pgbin")
        .arg(&pgbin)
        .current_dir(&repo_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    push_path_arg(&mut command, "--artifact-dir", args.artifact_dir.as_ref());
    push_path_arg(&mut command, "--run-dir", args.run_dir.as_ref());
    push_path_arg(&mut command, "--log-dir", args.log_dir.as_ref());
    push_path_arg(&mut command, "--smoke-log", args.smoke_log.as_ref());
    push_u16_arg(&mut command, "--coord-port", args.coord_port);
    push_u16_arg(&mut command, "--remote-ready-port", args.remote_ready_port);
    if let Some(run_id) = args.run_id {
        command.arg("--run-id").arg(run_id);
    }
    if args.skip_install {
        command.arg("--skip-install");
    }

    run_status(command)
        .await
        .wrap_err("running SPIRE PG18 Stage E fault fixture")
}

async fn run_stage_e_lifecycle_pg18(args: StageELifecyclePg18Args) -> Result<()> {
    if args.pg != 18 {
        bail!("lifecycle-pg18 requires --pg 18, got {}", args.pg);
    }
    let repo_root = repo_root()?;
    let pgbin = match args.pgbin {
        Some(path) => path,
        None => {
            let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
            find_pgrx_install(args.pg, &pgrx_home)?.bin_dir
        }
    };
    let script = repo_root.join(args.case.script_name());
    if !script.is_file() {
        bail!(
            "SPIRE Stage E lifecycle fixture script is missing: {}",
            script.display()
        );
    }

    crate::ecaz_println!("[spire-multicluster] repo={}", repo_root.display());
    crate::ecaz_println!("[spire-multicluster] pgbin={}", pgbin.display());
    crate::ecaz_println!(
        "[spire-multicluster] lifecycle_case={}",
        args.case.as_matrix_key()
    );
    if let Some(artifact_dir) = &args.artifact_dir {
        crate::ecaz_println!(
            "[spire-multicluster] artifact_dir={}",
            artifact_dir.display()
        );
    }

    let mut command = Command::new("bash");
    command
        .arg(&script)
        .arg("--case")
        .arg(args.case.as_matrix_key())
        .arg("--pgbin")
        .arg(&pgbin)
        .current_dir(&repo_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    push_path_arg(&mut command, "--artifact-dir", args.artifact_dir.as_ref());
    push_path_arg(&mut command, "--run-dir", args.run_dir.as_ref());
    push_path_arg(&mut command, "--log-dir", args.log_dir.as_ref());
    push_path_arg(&mut command, "--smoke-log", args.smoke_log.as_ref());
    push_u16_arg(&mut command, "--coord-port", args.coord_port);
    push_u16_arg(&mut command, "--remote-ready-port", args.remote_ready_port);
    if let Some(run_id) = args.run_id {
        command.arg("--run-id").arg(run_id);
    }
    if args.skip_install {
        command.arg("--skip-install");
    }

    run_status(command)
        .await
        .wrap_err("running SPIRE PG18 Stage E lifecycle fixture")
}

async fn run_local_multinode_pg18(args: LocalMultinodePg18Args) -> Result<()> {
    if args.pg != 18 {
        bail!("local-multinode-pg18 requires --pg 18, got {}", args.pg);
    }
    let repo_root = repo_root()?;
    let pgbin = match args.pgbin.as_ref() {
        Some(path) => path.clone(),
        None => {
            let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
            find_pgrx_install(args.pg, &pgrx_home)?.bin_dir
        }
    };

    run_native_local_multinode_pg18(repo_root, pgbin, args).await
}

#[derive(Debug, Clone)]
struct LocalPgNode {
    node_id: u32,
    name: &'static str,
    port: u16,
    data_dir: PathBuf,
    log_file: PathBuf,
    secret_name: Option<String>,
}

impl LocalPgNode {
    fn conninfo(&self, socket_dir: &Path) -> String {
        format!(
            "host={} port={} dbname=postgres user=ecaz_coord connect_timeout=1",
            socket_dir.display(),
            self.port
        )
    }
}

struct LocalPgCluster {
    pg_ctl: PathBuf,
    psql: PathBuf,
    nodes: Vec<LocalPgNode>,
}

impl LocalPgCluster {
    async fn stop_all(&self) {
        for node in &self.nodes {
            let _ = Command::new(&self.pg_ctl)
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
    }
}

async fn run_native_local_multinode_pg18(
    repo_root: PathBuf,
    pgbin: PathBuf,
    args: LocalMultinodePg18Args,
) -> Result<()> {
    let run_id = args.run_id.unwrap_or_else(default_local_multinode_run_id);
    let coord_port = args.coord_port.unwrap_or(39700);
    let remote1_port = args.remote1_port.unwrap_or(39701);
    let remote2_port = args.remote2_port.unwrap_or(39702);
    let remote3_port = args.remote3_port.unwrap_or(39703);
    let tier = args.tier.unwrap_or_else(|| "correctness".to_owned());
    if !matches!(tier.as_str(), "correctness" | "representative") {
        bail!("local-multinode-pg18 tier {tier:?} must be correctness or representative");
    }
    let storage_format = args.storage_format.unwrap_or_else(|| "rabitq".to_owned());
    let prefix = match args.prefix {
        Some(prefix) => prefix,
        None if tier == "correctness" => "ec_spire_aws_synth_10k".to_owned(),
        None => "ec_spire_aws_repr_1m".to_owned(),
    };
    let coord_index = args.coord_index.unwrap_or_else(|| format!("{prefix}_idx"));
    let remote_index = args
        .remote_index
        .unwrap_or_else(|| format!("{prefix}_remote_idx"));
    let run_dir = args
        .run_dir
        .unwrap_or_else(|| repo_root.join(format!("target/spire-local-multinode-{run_id}")));
    let log_dir = args
        .artifact_dir
        .clone()
        .unwrap_or_else(|| run_dir.join("logs"));
    let smoke_log = args.smoke_log.or_else(|| {
        args.artifact_dir
            .as_ref()
            .map(|artifact_dir| artifact_dir.join("local-multinode.log"))
    });
    let socket_dir = repo_root.join(format!("target/spire-local-multinode-sockets-{run_id}"));
    let topology = run_dir.join("topology.local.json");
    let work_dir = run_dir.join("work");
    let ecaz_bin = env::current_exe().wrap_err("resolving current ecaz executable")?;
    let pg_ctl = pgbin.join("pg_ctl");
    let psql = pgbin.join("psql");

    if run_dir.exists() {
        bail!("run_dir already exists: {}", run_dir.display());
    }
    fs::create_dir_all(&log_dir).wrap_err_with(|| format!("creating {}", log_dir.display()))?;
    fs::create_dir_all(&socket_dir)
        .wrap_err_with(|| format!("creating {}", socket_dir.display()))?;
    fs::create_dir_all(&run_dir).wrap_err_with(|| format!("creating {}", run_dir.display()))?;
    fs::create_dir_all(&work_dir).wrap_err_with(|| format!("creating {}", work_dir.display()))?;

    log_local_multinode(
        smoke_log.as_deref(),
        &format!("run_dir={}", run_dir.display()),
    )?;
    log_local_multinode(smoke_log.as_deref(), &format!("coord_port={coord_port}"))?;
    log_local_multinode(
        smoke_log.as_deref(),
        &format!("remote1_port={remote1_port}"),
    )?;
    log_local_multinode(
        smoke_log.as_deref(),
        &format!("remote2_port={remote2_port}"),
    )?;
    log_local_multinode(
        smoke_log.as_deref(),
        &format!("remote3_port={remote3_port}"),
    )?;
    log_local_multinode(smoke_log.as_deref(), &format!("tier={tier}"))?;
    log_local_multinode(smoke_log.as_deref(), &format!("prefix={prefix}"))?;

    crate::ecaz_println!("[spire-multicluster] repo={}", repo_root.display());
    crate::ecaz_println!("[spire-multicluster] pgbin={}", pgbin.display());
    crate::ecaz_println!("[spire-multicluster] topology=local-multinode-native");

    if !args.skip_install {
        let mut command = Command::new("cargo");
        command
            .current_dir(&repo_root)
            .arg("pgrx")
            .arg("install")
            .arg("--test")
            .arg("--pg-config")
            .arg(pgbin.join("pg_config"))
            .arg("--features")
            .arg("pg18 pg_test")
            .arg("--no-default-features")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        run_status(command)
            .await
            .wrap_err("installing ecaz into local PG18")?;
    }

    let nodes = vec![
        LocalPgNode {
            node_id: 1,
            name: "coord",
            port: coord_port,
            data_dir: run_dir.join("coord"),
            log_file: log_dir.join("coord-postgres.log"),
            secret_name: None,
        },
        LocalPgNode {
            node_id: 2,
            name: "remote1",
            port: remote1_port,
            data_dir: run_dir.join("remote1"),
            log_file: log_dir.join("remote1-postgres.log"),
            secret_name: Some("spire/remote/aws-local/node2".to_owned()),
        },
        LocalPgNode {
            node_id: 3,
            name: "remote2",
            port: remote2_port,
            data_dir: run_dir.join("remote2"),
            log_file: log_dir.join("remote2-postgres.log"),
            secret_name: Some("spire/remote/aws-local/node3".to_owned()),
        },
        LocalPgNode {
            node_id: 4,
            name: "remote3",
            port: remote3_port,
            data_dir: run_dir.join("remote3"),
            log_file: log_dir.join("remote3-postgres.log"),
            secret_name: Some("spire/remote/aws-local/node4".to_owned()),
        },
    ];
    let cluster = LocalPgCluster {
        pg_ctl,
        psql,
        nodes,
    };

    let result = async {
        init_local_pg_nodes(&cluster).await?;
        start_local_pg_nodes(&cluster, &socket_dir).await?;
        setup_local_pg_nodes(&cluster, &socket_dir).await?;
        write_local_topology(&topology, &socket_dir, &cluster.nodes)?;

        let fixture = prepare_local_multinode_fixture(
            &repo_root,
            &ecaz_bin,
            &socket_dir,
            &log_dir,
            &work_dir,
            &topology,
            coord_port,
            &tier,
            &prefix,
            &coord_index,
            &remote_index,
            &storage_format,
            &args.reloptions,
            &args.coord_reloptions,
            &args.remote_reloptions,
            args.prepared_prefix.as_deref(),
            args.prepared_dir.as_deref(),
        )
        .await?;

        run_local_multinode_smoke(
            &repo_root,
            &ecaz_bin,
            &socket_dir,
            coord_port,
            &log_dir,
            &prefix,
            &coord_index,
            fixture.truth_corpus_file.as_deref(),
        )
        .await?;

        if !args.skip_bench_suite {
            run_local_multinode_bench_suite(
                &repo_root,
                &ecaz_bin,
                &socket_dir,
                coord_port,
                &log_dir,
                &prefix,
                &coord_index,
                args.bench_top_k.unwrap_or(10),
                args.bench_queries_limit.unwrap_or(200),
                args.bench_sweep.as_deref().unwrap_or("64,96"),
                args.bench_rowcap_sweep.as_deref().unwrap_or("96"),
                args.skip_bench_rowcap,
                args.bench_truth_corpus_file
                    .as_deref()
                    .or(fixture.truth_corpus_file.as_deref()),
                &args.bench_query_metric_projection_columns,
                &args.bench_session_gucs,
                &args.bench_production_read_variants,
            )
            .await?;
        }

        if !args.skip_fault_drills && tier == "correctness" {
            log_local_multinode(
                smoke_log.as_deref(),
                "fault_drills=skipped_native_lane_smoke_only",
            )?;
        }

        log_local_multinode(smoke_log.as_deref(), "SPIRE local multinode fixture passed")?;
        log_local_multinode(smoke_log.as_deref(), "HARNESS PASSED")?;
        Ok(())
    }
    .await;

    cluster.stop_all().await;
    result
}

struct LocalMultinodeFixture {
    truth_corpus_file: Option<PathBuf>,
}

async fn init_local_pg_nodes(cluster: &LocalPgCluster) -> Result<()> {
    for node in &cluster.nodes {
        let mut command = Command::new(&cluster.pg_ctl);
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
            .wrap_err_with(|| format!("initializing {}", node.name))?;
    }
    Ok(())
}

async fn start_local_pg_nodes(cluster: &LocalPgCluster, socket_dir: &Path) -> Result<()> {
    let conninfo_env = local_conninfo_env(&cluster.nodes, socket_dir)?;
    for node in cluster.nodes.iter().filter(|node| node.node_id != 1) {
        start_one_local_pg_node(cluster, node, socket_dir, &conninfo_env).await?;
    }
    let coord = cluster
        .nodes
        .iter()
        .find(|node| node.node_id == 1)
        .ok_or_else(|| eyre!("missing coordinator node"))?;
    start_one_local_pg_node(cluster, coord, socket_dir, &conninfo_env).await
}

async fn start_one_local_pg_node(
    cluster: &LocalPgCluster,
    node: &LocalPgNode,
    socket_dir: &Path,
    conninfo_env: &[(String, String)],
) -> Result<()> {
    let mut command = Command::new(&cluster.pg_ctl);
    command
        .arg("-w")
        .arg("-D")
        .arg(&node.data_dir)
        .arg("-l")
        .arg(&node.log_file)
        .arg("-o")
        .arg(format!(
            "-p {} -k {} -c listen_addresses=''",
            node.port,
            socket_dir.display()
        ))
        .arg("start")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    for (name, value) in conninfo_env {
        command.env(name, value);
    }
    run_status(command)
        .await
        .wrap_err_with(|| format!("starting {}", node.name))
}

async fn setup_local_pg_nodes(cluster: &LocalPgCluster, socket_dir: &Path) -> Result<()> {
    for node in &cluster.nodes {
        let mut command = psql_command(cluster, socket_dir, node.port);
        command
            .arg("-c")
            .arg("CREATE ROLE ecaz_coord LOGIN SUPERUSER; CREATE EXTENSION ecaz;")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        run_status(command)
            .await
            .wrap_err_with(|| format!("creating role/extension on {}", node.name))?;
    }
    Ok(())
}

fn psql_command(cluster: &LocalPgCluster, socket_dir: &Path, port: u16) -> Command {
    let mut command = Command::new(&cluster.psql);
    command
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-h")
        .arg(socket_dir)
        .arg("-p")
        .arg(port.to_string())
        .arg("-U")
        .arg("postgres")
        .arg("-d")
        .arg("postgres");
    command
}

fn local_conninfo_env(nodes: &[LocalPgNode], socket_dir: &Path) -> Result<Vec<(String, String)>> {
    nodes
        .iter()
        .filter_map(|node| {
            node.secret_name.as_ref().map(|secret| {
                spire_remote_conninfo_provider_lookup_key(secret)
                    .map(|key| (key, node.conninfo(socket_dir)))
            })
        })
        .collect()
}

fn spire_remote_conninfo_provider_lookup_key(secret_name: &str) -> Result<String> {
    if secret_name.is_empty() {
        bail!("remote secret name must be nonempty");
    }
    let mut key = String::from("EC_SPIRE_REMOTE_CONNINFO_");
    for byte in secret_name.bytes() {
        if byte.is_ascii_alphanumeric() {
            key.push(char::from(byte).to_ascii_uppercase());
        } else {
            key.push('_');
        }
    }
    Ok(key)
}

fn write_local_topology(path: &Path, socket_dir: &Path, nodes: &[LocalPgNode]) -> Result<()> {
    let coord = nodes
        .iter()
        .find(|node| node.node_id == 1)
        .ok_or_else(|| eyre!("missing coordinator node"))?;
    let remotes = nodes
        .iter()
        .filter(|node| node.node_id != 1)
        .map(|node| {
            json!({
                "node_id": node.node_id,
                "operator_host": socket_dir,
                "operator_port": node.port,
                "secret_name": node.secret_name.as_deref().unwrap_or_default()
            })
        })
        .collect::<Vec<_>>();
    let topology = json!({
        "coordinator": {
            "node_id": coord.node_id,
            "operator_host": socket_dir,
            "operator_port": coord.port
        },
        "remotes": remotes
    });
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    fs::write(
        path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&topology).expect("topology json should serialize")
        ),
    )
    .wrap_err_with(|| format!("writing {}", path.display()))
}

#[allow(clippy::too_many_arguments)]
async fn prepare_local_multinode_fixture(
    repo_root: &Path,
    ecaz_bin: &Path,
    socket_dir: &Path,
    log_dir: &Path,
    work_dir: &Path,
    topology_path: &Path,
    coord_port: u16,
    tier: &str,
    prefix: &str,
    coord_index: &str,
    remote_index: &str,
    storage_format: &str,
    shared_reloptions: &[String],
    coord_reloptions: &[String],
    remote_reloptions: &[String],
    prepared_prefix: Option<&str>,
    prepared_dir: Option<&Path>,
) -> Result<LocalMultinodeFixture> {
    let corpus_file;
    let queries_file;
    let manifest_file;
    let truth_corpus_file;
    match tier {
        "correctness" => {
            corpus_file = work_dir.join(format!("{prefix}_corpus.tsv"));
            queries_file = work_dir.join(format!("{prefix}_queries.tsv"));
            manifest_file = None;
            run_ecaz_generate(repo_root, ecaz_bin, 10000, &corpus_file).await?;
            run_ecaz_generate(repo_root, ecaz_bin, 100, &queries_file).await?;
            truth_corpus_file = None;
        }
        "representative" => {
            let prepared_prefix = prepared_prefix.unwrap_or("ec_real_100k");
            let prepared_dir = prepared_dir.ok_or_else(|| {
                eyre!("representative local multinode requires --prepared-dir in native mode")
            })?;
            corpus_file = prepared_dir.join(format!("{prepared_prefix}_corpus.tsv"));
            queries_file = prepared_dir.join(format!("{prepared_prefix}_queries.tsv"));
            manifest_file = Some(prepared_dir.join(format!("{prepared_prefix}_manifest.json")));
            for path in [&corpus_file, &queries_file] {
                if !path.is_file() {
                    bail!(
                        "prepared representative file is missing: {}",
                        path.display()
                    );
                }
            }
            truth_corpus_file = Some(corpus_file.clone());
        }
        _ => bail!("unsupported local multinode tier {tier:?}"),
    }

    let coord_reloptions = joined_reloption_args(shared_reloptions, coord_reloptions);
    let remote_reloptions = joined_reloption_args(shared_reloptions, remote_reloptions);
    run_ecaz_corpus_load(
        repo_root,
        ecaz_bin,
        socket_dir,
        coord_port,
        log_dir,
        "coordinator-load.log",
        prefix,
        &corpus_file,
        Some(&queries_file),
        manifest_file.as_ref(),
        storage_format,
        coord_index,
        &coord_reloptions,
        false,
    )
    .await?;

    let plan_file = write_leaf_owned_local_plan(
        repo_root,
        ecaz_bin,
        socket_dir,
        log_dir,
        topology_path,
        coord_port,
        prefix,
        coord_index,
        remote_index,
        &corpus_file,
        storage_format,
        &remote_reloptions,
    )
    .await?;
    load_local_remote_shards(repo_root, ecaz_bin, socket_dir, log_dir, &plan_file).await?;
    register_local_remote_shards(
        repo_root,
        ecaz_bin,
        socket_dir,
        log_dir,
        coord_port,
        coord_index,
        &plan_file,
    )
    .await?;

    Ok(LocalMultinodeFixture { truth_corpus_file })
}

async fn run_ecaz_generate(
    repo_root: &Path,
    ecaz_bin: &Path,
    n: usize,
    output: &Path,
) -> Result<()> {
    let mut command = Command::new(ecaz_bin);
    command
        .current_dir(repo_root)
        .arg("corpus")
        .arg("generate")
        .arg("--n")
        .arg(n.to_string())
        .arg("--dim")
        .arg("1536")
        .arg("--output")
        .arg(output)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    run_status(command)
        .await
        .wrap_err_with(|| format!("generating {}", output.display()))
}

#[allow(clippy::too_many_arguments)]
async fn run_ecaz_corpus_load(
    repo_root: &Path,
    ecaz_bin: &Path,
    socket_dir: &Path,
    port: u16,
    log_dir: &Path,
    log_name: &str,
    prefix: &str,
    corpus_file: &Path,
    queries_file: Option<&PathBuf>,
    manifest_file: Option<&PathBuf>,
    storage_format: &str,
    index_name: &str,
    reloptions: &[String],
    corpus_only: bool,
) -> Result<()> {
    let mut command = Command::new(ecaz_bin);
    command
        .current_dir(repo_root)
        .arg("--database")
        .arg("postgres")
        .arg("--host")
        .arg(socket_dir)
        .arg("--port")
        .arg(port.to_string())
        .arg("--user")
        .arg("ecaz_coord")
        .arg("--log-file")
        .arg(log_dir.join(log_name))
        .arg("corpus")
        .arg("load")
        .arg("--profile")
        .arg("ec_spire")
        .arg("--prefix")
        .arg(prefix)
        .arg("--dim")
        .arg("1536")
        .arg("--bits")
        .arg("4")
        .arg("--seed")
        .arg("42")
        .arg("--corpus-file")
        .arg(corpus_file)
        .arg("--storage-format")
        .arg(storage_format)
        .arg("--index-name")
        .arg(index_name);
    if let Some(queries_file) = queries_file {
        command.arg("--queries-file").arg(queries_file);
    }
    if let Some(manifest_file) = manifest_file {
        command.arg("--manifest-file").arg(manifest_file);
    }
    if corpus_only {
        command.arg("--corpus-only");
    }
    for reloption in reloptions {
        command.arg("--reloption").arg(reloption);
    }
    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    run_status(command)
        .await
        .wrap_err_with(|| format!("loading local multinode corpus prefix {prefix} on port {port}"))
}

#[allow(clippy::too_many_arguments)]
async fn write_leaf_owned_local_plan(
    repo_root: &Path,
    ecaz_bin: &Path,
    socket_dir: &Path,
    log_dir: &Path,
    topology_path: &Path,
    coord_port: u16,
    prefix: &str,
    coord_index: &str,
    remote_index: &str,
    corpus_file: &Path,
    storage_format: &str,
    remote_reloptions: &[String],
) -> Result<PathBuf> {
    let topology_raw = fs::read_to_string(topology_path)
        .wrap_err_with(|| format!("reading {}", topology_path.display()))?;
    let topology: serde_json::Value =
        serde_json::from_str(&topology_raw).wrap_err("parsing local topology")?;
    let remotes = topology
        .get("remotes")
        .and_then(|value| value.as_array())
        .ok_or_else(|| eyre!("topology remotes must be an array"))?;
    let distributed_dir = log_dir.join("distributed-correctness");
    fs::create_dir_all(&distributed_dir)
        .wrap_err_with(|| format!("creating {}", distributed_dir.display()))?;
    let remotes_json = serde_json::to_string(
        &remotes
            .iter()
            .map(|remote| json!({"node_id": remote["node_id"]}))
            .collect::<Vec<_>>(),
    )?;
    let mut remote_plans = Vec::new();
    let mut total_rows = 0usize;
    for remote in remotes {
        let node_id = remote["node_id"]
            .as_u64()
            .ok_or_else(|| eyre!("remote node_id must be integer"))? as u32;
        let remote_port = remote["operator_port"]
            .as_u64()
            .ok_or_else(|| eyre!("remote operator_port must be integer"))?
            as u16;
        let secret_name = remote["secret_name"]
            .as_str()
            .ok_or_else(|| eyre!("remote secret_name must be string"))?;
        let remote_prefix = format!("{prefix}_node_{node_id}");
        let node_dir = distributed_dir.join(format!("node-{node_id}"));
        fs::create_dir_all(&node_dir)
            .wrap_err_with(|| format!("creating {}", node_dir.display()))?;
        let assignments = node_dir.join("coordinator-base-assignments.tsv");
        run_ecaz_dev_sql_to_file(
            repo_root,
            ecaz_bin,
            socket_dir,
            coord_port,
            repo_root.join("scripts/spire-aws/export-coordinator-leaf-base-assignments.sql"),
            &[
                ("coord_index", coord_index.to_owned()),
                ("coord_table", format!("{prefix}_corpus")),
                ("node_id", node_id.to_string()),
                ("remotes_json", remotes_json.clone()),
            ],
            &assignments,
        )
        .await?;
        let remote_corpus = node_dir.join(format!("{remote_prefix}_corpus.tsv"));
        let row_count =
            write_remote_corpus_from_assignments(&assignments, corpus_file, &remote_corpus)?;
        total_rows += row_count;
        remote_plans.push(json!({
            "node_id": node_id,
            "operator_port": remote_port,
            "conninfo_secret_name": secret_name,
            "conninfo_provider_lookup_key": spire_remote_conninfo_provider_lookup_key(secret_name)?,
            "remote_index_regclass": remote_index,
            "remote_prefix": remote_prefix,
            "shard_ids": [node_id - 2],
            "corpus_file": remote_corpus,
            "remote_load_args": remote_load_args(&remote_prefix, &remote_corpus, remote_index, storage_format, remote_reloptions),
            "remote_identity_query_sql": remote_identity_query_sql(remote_index),
            "coordinator_register_descriptor_sql_template": "",
            "row_count": row_count,
            "shard_row_counts": [{"shard_id": node_id - 2, "row_count": row_count}]
        }));
    }
    let plan = json!({
        "version": 1,
        "prefix": prefix,
        "profile": "ec_spire",
        "dimension": 1536,
        "bits": 4,
        "seed": 42,
        "storage_format": storage_format,
        "reloptions": remote_reloptions,
        "coordinator_index_name": coord_index,
        "source_identity_column": "leaf_base_assignment",
        "shard_policy": "coordinator_leaf_assignment_round_robin",
        "shard_count": remotes.len(),
        "total_rows": total_rows,
        "remotes": remote_plans
    });
    let plan_file = distributed_dir.join("distributed-placement-plan.json");
    fs::write(
        &plan_file,
        format!("{}\n", serde_json::to_string_pretty(&plan)?),
    )
    .wrap_err_with(|| format!("writing {}", plan_file.display()))?;
    fs::write(
        log_dir.join("distributed-placement-plan-correctness.path"),
        format!("{}\n", plan_file.display()),
    )
    .wrap_err("writing distributed placement plan pointer")?;
    Ok(plan_file)
}

async fn run_ecaz_dev_sql_to_file(
    repo_root: &Path,
    ecaz_bin: &Path,
    socket_dir: &Path,
    port: u16,
    sql_file: PathBuf,
    sets: &[(&str, String)],
    output_file: &Path,
) -> Result<()> {
    let mut command = Command::new(ecaz_bin);
    command
        .current_dir(repo_root)
        .arg("--database")
        .arg("postgres")
        .arg("--host")
        .arg(socket_dir)
        .arg("--port")
        .arg(port.to_string())
        .arg("--user")
        .arg("ecaz_coord")
        .arg("dev")
        .arg("sql")
        .arg("--file")
        .arg(sql_file);
    for (name, value) in sets {
        command.arg("--set").arg(format!("{name}={value}"));
    }
    let output = command.output().await.wrap_err("running ecaz dev sql")?;
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent).wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    fs::write(output_file, &output.stdout)
        .wrap_err_with(|| format!("writing {}", output_file.display()))?;
    if !output.status.success() {
        bail!(
            "ecaz dev sql failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn write_remote_corpus_from_assignments(
    assignments: &Path,
    corpus_file: &Path,
    output_file: &Path,
) -> Result<usize> {
    let mut wanted = BTreeSet::new();
    for line in BufReader::new(
        fs::File::open(assignments)
            .wrap_err_with(|| format!("opening {}", assignments.display()))?,
    )
    .lines()
    {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let row_id = line
            .split('\t')
            .nth(11)
            .ok_or_else(|| eyre!("assignment row has fewer than 12 fields: {line}"))?;
        wanted.insert(row_id.to_owned());
    }
    let mut out = fs::File::create(output_file)
        .wrap_err_with(|| format!("creating {}", output_file.display()))?;
    let mut written = 0usize;
    for line in BufReader::new(
        fs::File::open(corpus_file)
            .wrap_err_with(|| format!("opening {}", corpus_file.display()))?,
    )
    .lines()
    {
        let line = line?;
        let Some((id, _)) = line.split_once('\t') else {
            continue;
        };
        if wanted.contains(id) {
            writeln!(out, "{line}")?;
            written += 1;
        }
    }
    if written != wanted.len() {
        bail!(
            "remote corpus {} wrote {} rows for {} assigned row ids",
            output_file.display(),
            written,
            wanted.len()
        );
    }
    Ok(written)
}

fn remote_load_args(
    remote_prefix: &str,
    remote_corpus: &Path,
    remote_index: &str,
    storage_format: &str,
    reloptions: &[String],
) -> Vec<String> {
    let mut args = vec![
        "ecaz".to_owned(),
        "corpus".to_owned(),
        "load".to_owned(),
        "--profile".to_owned(),
        "ec_spire".to_owned(),
        "--prefix".to_owned(),
        remote_prefix.to_owned(),
        "--dim".to_owned(),
        "1536".to_owned(),
        "--bits".to_owned(),
        "4".to_owned(),
        "--seed".to_owned(),
        "42".to_owned(),
        "--corpus-file".to_owned(),
        remote_corpus.display().to_string(),
        "--corpus-only".to_owned(),
        "--storage-format".to_owned(),
        storage_format.to_owned(),
        "--index-name".to_owned(),
        remote_index.to_owned(),
    ];
    for reloption in reloptions {
        args.push("--reloption".to_owned());
        args.push(reloption.to_owned());
    }
    args
}

async fn load_local_remote_shards(
    repo_root: &Path,
    ecaz_bin: &Path,
    socket_dir: &Path,
    log_dir: &Path,
    plan_file: &Path,
) -> Result<()> {
    let plan = read_plan_json(plan_file)?;
    for remote in plan["remotes"]
        .as_array()
        .ok_or_else(|| eyre!("plan remotes must be array"))?
    {
        let node_id = remote["node_id"].as_u64().ok_or_else(|| eyre!("node_id"))? as u32;
        let port = remote["operator_port"]
            .as_u64()
            .ok_or_else(|| eyre!("operator_port"))? as u16;
        let remote_prefix = remote["remote_prefix"]
            .as_str()
            .ok_or_else(|| eyre!("remote_prefix"))?;
        let corpus_file = PathBuf::from(
            remote["corpus_file"]
                .as_str()
                .ok_or_else(|| eyre!("corpus_file"))?,
        );
        let remote_index = remote["remote_index_regclass"]
            .as_str()
            .ok_or_else(|| eyre!("remote_index_regclass"))?;
        let storage_format = plan["storage_format"].as_str().unwrap_or("rabitq");
        let reloptions = plan["reloptions"]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        run_ecaz_corpus_load(
            repo_root,
            ecaz_bin,
            socket_dir,
            port,
            log_dir,
            &format!("remote-load-node-{node_id}.log"),
            remote_prefix,
            &corpus_file,
            None,
            None,
            storage_format,
            remote_index,
            &reloptions,
            true,
        )
        .await?;
    }
    Ok(())
}

async fn register_local_remote_shards(
    repo_root: &Path,
    ecaz_bin: &Path,
    socket_dir: &Path,
    log_dir: &Path,
    coord_port: u16,
    coord_index: &str,
    plan_file: &Path,
) -> Result<()> {
    let plan = read_plan_json(plan_file)?;
    let remotes = plan["remotes"]
        .as_array()
        .ok_or_else(|| eyre!("plan remotes must be array"))?;
    let remotes_json = serde_json::to_string(
        &remotes
            .iter()
            .map(|remote| json!({"node_id": remote["node_id"]}))
            .collect::<Vec<_>>(),
    )?;
    let identity_dir = log_dir.join("remote-identities");
    fs::create_dir_all(&identity_dir)
        .wrap_err_with(|| format!("creating {}", identity_dir.display()))?;
    let materialize_template = fs::read_to_string(
        repo_root.join("scripts/spire-aws/materialize-remote-leaf-base-assignments.sql"),
    )
    .wrap_err("reading remote leaf materialization template")?;
    for remote in remotes {
        let node_id = remote["node_id"].as_u64().ok_or_else(|| eyre!("node_id"))? as u32;
        let port = remote["operator_port"]
            .as_u64()
            .ok_or_else(|| eyre!("operator_port"))? as u16;
        let remote_index = remote["remote_index_regclass"]
            .as_str()
            .ok_or_else(|| eyre!("remote_index_regclass"))?;
        let remote_prefix = remote["remote_prefix"]
            .as_str()
            .ok_or_else(|| eyre!("remote_prefix"))?;
        let remote_table = format!("{remote_prefix}_corpus");
        let assignments = log_dir
            .join("distributed-correctness")
            .join(format!("node-{node_id}/coordinator-base-assignments.tsv"));
        let rendered = log_dir
            .join("remote-leaf-materialization")
            .join(format!("node-{node_id}-remote-materialize-rendered.sql"));
        if let Some(parent) = rendered.parent() {
            fs::create_dir_all(parent)
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        fs::write(
            &rendered,
            materialize_template.replace(
                "__ECAZ_ASSIGNMENT_FILE__",
                &assignments.display().to_string().replace('\'', "''"),
            ),
        )
        .wrap_err_with(|| format!("writing {}", rendered.display()))?;
        run_ecaz_dev_sql_logged(
            repo_root,
            ecaz_bin,
            socket_dir,
            port,
            &rendered,
            &[
                ("remote_index", remote_index.to_owned()),
                ("remote_table", remote_table),
            ],
            log_dir.join(format!(
                "remote-leaf-materialization/node-{node_id}-remote-materialize.log"
            )),
        )
        .await?;
        let identity_sql = remote["remote_identity_query_sql"]
            .as_str()
            .ok_or_else(|| eyre!("remote_identity_query_sql"))?;
        run_ecaz_dev_sql_string_to_file(
            repo_root,
            ecaz_bin,
            socket_dir,
            port,
            identity_sql,
            &identity_dir.join(format!("node-{node_id}-identity.json")),
        )
        .await?;
    }
    run_ecaz_dev_sql_logged(
        repo_root,
        ecaz_bin,
        socket_dir,
        coord_port,
        &repo_root.join("scripts/spire-aws/publish-remote-placements.sql"),
        &[
            ("coord_index", coord_index.to_owned()),
            ("remotes_json", remotes_json),
        ],
        log_dir.join("publish-remote-placements.log"),
    )
    .await?;
    let registration_sql = log_dir.join("register-remotes-rendered.sql");
    let mut render = Command::new(ecaz_bin);
    render
        .current_dir(repo_root)
        .arg("corpus")
        .arg("render-spire-registrations")
        .arg("--plan-file")
        .arg(plan_file)
        .arg("--identity-dir")
        .arg(&identity_dir)
        .arg("--output-file")
        .arg(&registration_sql)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    run_status(render)
        .await
        .wrap_err("rendering remote registrations")?;
    run_ecaz_dev_sql_logged(
        repo_root,
        ecaz_bin,
        socket_dir,
        coord_port,
        &registration_sql,
        &[],
        log_dir.join("register-remotes.log"),
    )
    .await
}

fn read_plan_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).wrap_err_with(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).wrap_err_with(|| format!("parsing {}", path.display()))
}

fn remote_identity_query_sql(remote_index_regclass: &str) -> String {
    let quoted = sql_string_literal(remote_index_regclass);
    format!(
        "SELECT jsonb_build_object('remote_index_regclass', {quoted}, 'last_served_epoch', a.active_epoch, 'min_retained_epoch', a.active_epoch, 'extension_version', e.extension_version, 'remote_index_identity_hex', e.profile_fingerprint, 'endpoint_status', e.status, 'tuple_transport_status', e.tuple_transport_status)::text FROM ec_spire_remote_search_endpoint_identity({quoted}::regclass::oid) e CROSS JOIN ec_spire_index_active_snapshot_diagnostics({quoted}::regclass::oid) a"
    )
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

async fn run_ecaz_dev_sql_logged(
    repo_root: &Path,
    ecaz_bin: &Path,
    socket_dir: &Path,
    port: u16,
    sql_file: &Path,
    sets: &[(&str, String)],
    log_output: PathBuf,
) -> Result<()> {
    let mut command = Command::new(ecaz_bin);
    command
        .current_dir(repo_root)
        .arg("--database")
        .arg("postgres")
        .arg("--host")
        .arg(socket_dir)
        .arg("--port")
        .arg(port.to_string())
        .arg("--user")
        .arg("ecaz_coord")
        .arg("dev")
        .arg("sql")
        .arg("--file")
        .arg(sql_file)
        .arg("--log-output")
        .arg(log_output);
    for (name, value) in sets {
        command.arg("--set").arg(format!("{name}={value}"));
    }
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    run_status(command)
        .await
        .wrap_err("running logged ecaz dev sql")
}

async fn run_ecaz_dev_sql_string_to_file(
    repo_root: &Path,
    ecaz_bin: &Path,
    socket_dir: &Path,
    port: u16,
    sql: &str,
    output_file: &Path,
) -> Result<()> {
    let mut command = Command::new(ecaz_bin);
    command
        .current_dir(repo_root)
        .arg("--database")
        .arg("postgres")
        .arg("--host")
        .arg(socket_dir)
        .arg("--port")
        .arg(port.to_string())
        .arg("--user")
        .arg("ecaz_coord")
        .arg("dev")
        .arg("sql")
        .arg("--env")
        .arg("PGOPTIONS=-c client_min_messages=error")
        .arg("--sql")
        .arg(sql);
    let output = command.output().await.wrap_err("running identity SQL")?;
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent).wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    fs::write(output_file, &output.stdout)
        .wrap_err_with(|| format!("writing {}", output_file.display()))?;
    if !output.status.success() {
        bail!(
            "identity SQL failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

async fn run_local_multinode_smoke(
    repo_root: &Path,
    ecaz_bin: &Path,
    socket_dir: &Path,
    coord_port: u16,
    log_dir: &Path,
    prefix: &str,
    coord_index: &str,
    truth_corpus_file: Option<&Path>,
) -> Result<()> {
    run_ecaz_dev_sql_logged(
        repo_root,
        ecaz_bin,
        socket_dir,
        coord_port,
        &repo_root.join("scripts/spire-aws/smoke-customscan-read.sql"),
        &[
            ("prefix", prefix.to_owned()),
            ("index_name", coord_index.to_owned()),
        ],
        log_dir.join("smoke-customscan-read.log"),
    )
    .await?;
    let production_sql = format!(
        "SET enable_seqscan = off; SET enable_indexscan = off; SELECT * FROM ec_spire_remote_search_production_read_profile({coord_index}::regclass, (SELECT source FROM {prefix}_queries ORDER BY id LIMIT 1)::real[], 10)",
        coord_index = sql_string_literal(coord_index)
    );
    run_ecaz_dev_sql_string_logged(
        repo_root,
        ecaz_bin,
        socket_dir,
        coord_port,
        &production_sql,
        log_dir.join("production-read-profile-smoke.log"),
    )
    .await?;
    let mut command = Command::new(ecaz_bin);
    command
        .current_dir(repo_root)
        .arg("--database")
        .arg("postgres")
        .arg("--host")
        .arg(socket_dir)
        .arg("--port")
        .arg(coord_port.to_string())
        .arg("--user")
        .arg("ecaz_coord")
        .arg("bench")
        .arg("spire-pipeline")
        .arg("--prefix")
        .arg(prefix)
        .arg("--queries-limit")
        .arg("5")
        .arg("--sweep")
        .arg("8,16,32")
        .arg("--include-remote")
        .arg("--consistency-mode")
        .arg("epoch")
        .arg("--include-cost-snapshot")
        .arg("--include-query-metrics")
        .arg("--include-recall")
        .arg("--include-production-read-profile")
        .arg("--production-read-only")
        .arg("--remote-tuple-transport")
        .arg("pg_binary_attr_v1")
        .arg("--query-metric-k")
        .arg("10")
        .arg("--log-output")
        .arg(log_dir.join("bench-spire-pipeline-smoke.log"));
    if let Some(truth_corpus_file) = truth_corpus_file {
        command.arg("--truth-corpus-file").arg(truth_corpus_file);
    }
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    run_status(command)
        .await
        .wrap_err("running local multinode pipeline smoke")
}

async fn run_ecaz_dev_sql_string_logged(
    repo_root: &Path,
    ecaz_bin: &Path,
    socket_dir: &Path,
    port: u16,
    sql: &str,
    log_output: PathBuf,
) -> Result<()> {
    let mut command = Command::new(ecaz_bin);
    command
        .current_dir(repo_root)
        .arg("--database")
        .arg("postgres")
        .arg("--host")
        .arg(socket_dir)
        .arg("--port")
        .arg(port.to_string())
        .arg("--user")
        .arg("ecaz_coord")
        .arg("dev")
        .arg("sql")
        .arg("--sql")
        .arg(sql)
        .arg("--log-output")
        .arg(log_output)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    run_status(command).await.wrap_err("running logged SQL")
}

#[allow(clippy::too_many_arguments)]
async fn run_local_multinode_bench_suite(
    repo_root: &Path,
    ecaz_bin: &Path,
    socket_dir: &Path,
    coord_port: u16,
    log_dir: &Path,
    prefix: &str,
    coord_index: &str,
    bench_top_k: u16,
    bench_queries_limit: usize,
    bench_sweep: &str,
    bench_rowcap_sweep: &str,
    skip_bench_rowcap: bool,
    truth_corpus_file: Option<&Path>,
    query_metric_projection_columns: &[String],
    session_gucs: &[String],
    production_read_variants: &[String],
) -> Result<()> {
    let suite_artifact_dir = log_dir.join("bench-suite");
    fs::create_dir_all(&suite_artifact_dir)
        .wrap_err_with(|| format!("creating {}", suite_artifact_dir.display()))?;
    let suite_config = suite_artifact_dir.join("local-real-production-read-suite.json");
    let base_projection_columns = if query_metric_projection_columns.is_empty() {
        vec!["id".to_owned()]
    } else {
        query_metric_projection_columns.to_vec()
    };
    let variants = if production_read_variants.is_empty() {
        vec![BenchProductionReadVariant {
            name: None,
            projection_columns: None,
            session_gucs: Vec::new(),
            timeline_no_payload: false,
        }]
    } else {
        production_read_variants
            .iter()
            .map(|raw| parse_bench_production_read_variant(raw))
            .collect::<Result<Vec<_>>>()?
    };

    let default_sweep = parse_u16_list(bench_sweep)?;
    let rowcap_sweep = parse_u16_list(bench_rowcap_sweep)?;
    let mut steps = vec![json!({
        "kind": "storage",
        "name": "storage-local-coordinator",
        "tags": ["local", "multinode", "storage"],
        "prefix": prefix,
        "log_file": suite_artifact_dir.join("storage.log")
    })];
    for variant in variants {
        let projection_columns = variant
            .projection_columns
            .unwrap_or_else(|| base_projection_columns.clone());
        let mut variant_session_gucs = session_gucs.to_vec();
        variant_session_gucs.extend(variant.session_gucs);
        let name_suffix = variant
            .name
            .map(|name| format!("-{name}"))
            .unwrap_or_default();
        let mut default_step = production_read_step_json(
            prefix,
            coord_index,
            bench_queries_limit,
            &default_sweep,
            bench_top_k,
            &projection_columns,
            &variant_session_gucs,
            &suite_artifact_dir.join(format!("production-read-k10{name_suffix}-default.log")),
            format!("production-read-k10{name_suffix}-default"),
            "default-cap",
            None,
            variant.timeline_no_payload,
        );
        if let Some(truth_corpus_file) = truth_corpus_file {
            default_step["truth_corpus_file"] = json!(truth_corpus_file);
        }
        steps.push(default_step);
        if !skip_bench_rowcap {
            let mut rowcap_step = production_read_step_json(
                prefix,
                coord_index,
                bench_queries_limit,
                &rowcap_sweep,
                bench_top_k,
                &projection_columns,
                &variant_session_gucs,
                &suite_artifact_dir.join(format!("production-read-k10{name_suffix}-rowcap25k.log")),
                format!("production-read-k10{name_suffix}-rowcap25k"),
                "rowcap25k",
                Some(25000),
                variant.timeline_no_payload,
            );
            if let Some(truth_corpus_file) = truth_corpus_file {
                rowcap_step["truth_corpus_file"] = json!(truth_corpus_file);
            }
            steps.push(rowcap_step);
        }
    }

    let suite = json!({
        "name": "local-multinode-production-read",
        "schema_version": 1,
        "artifact_dir": suite_artifact_dir,
        "defaults": {
            "queries_limit": bench_queries_limit,
            "pg": 18
        },
        "steps": steps
    });
    fs::write(
        &suite_config,
        format!("{}\n", serde_json::to_string_pretty(&suite)?),
    )
    .wrap_err_with(|| format!("writing {}", suite_config.display()))?;
    let mut command = Command::new(ecaz_bin);
    command
        .current_dir(repo_root)
        .arg("--database")
        .arg("postgres")
        .arg("--host")
        .arg(socket_dir)
        .arg("--port")
        .arg(coord_port.to_string())
        .arg("--user")
        .arg("ecaz_coord")
        .arg("bench")
        .arg("suite")
        .arg("run")
        .arg("--config")
        .arg(&suite_config)
        .arg("--manifest-output")
        .arg(suite_artifact_dir.join("suite-manifest.json"))
        .arg("--results-output")
        .arg(suite_artifact_dir.join("results.jsonl"))
        .arg("--log-file")
        .arg(suite_artifact_dir.join("suite-run.log"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    run_status(command)
        .await
        .wrap_err("running local multinode nested bench suite")
}

#[allow(clippy::too_many_arguments)]
fn production_read_step_json(
    prefix: &str,
    coord_index: &str,
    queries_limit: usize,
    sweep: &[u16],
    top_k: u16,
    projection_columns: &[String],
    session_gucs: &[String],
    log_output: &Path,
    name: String,
    cap_tag: &str,
    max_routed_candidate_rows: Option<usize>,
    production_read_timeline_no_payload: bool,
) -> Value {
    let mut step = json!({
        "kind": "spire-pipeline",
        "name": name,
        "tags": ["phase5", "local", "multinode", "production-read", cap_tag],
        "prefix": prefix,
        "index": coord_index,
        "queries_limit": queries_limit,
        "sweep": sweep,
        "top_k": top_k,
        "include_remote": true,
        "require_remote_placements": true,
        "include_cost_snapshot": true,
        "include_query_metrics": true,
        "include_recall": true,
        "include_production_read_profile": true,
        "production_read_only": true,
        "query_metric_k": top_k,
        "query_metric_projection_columns": projection_columns,
        "session_gucs": session_gucs,
        "log_output": log_output
    });
    if let Some(max_routed_candidate_rows) = max_routed_candidate_rows {
        step["max_routed_candidate_rows"] = json!(max_routed_candidate_rows);
    }
    if production_read_timeline_no_payload {
        step["production_read_timeline_no_payload"] = json!(true);
    }
    step
}

#[derive(Debug, Eq, PartialEq)]
struct BenchProductionReadVariant {
    name: Option<String>,
    projection_columns: Option<Vec<String>>,
    session_gucs: Vec<String>,
    timeline_no_payload: bool,
}

fn parse_bench_production_read_variant(raw: &str) -> Result<BenchProductionReadVariant> {
    let mut name = None;
    let mut projection_columns = None;
    let mut session_gucs = Vec::new();
    let mut timeline_no_payload = false;
    for part in raw.split(';').filter(|part| !part.trim().is_empty()) {
        let (key, value) = part.split_once('=').ok_or_else(|| {
            eyre!("bench production-read variant part {part:?} must be key=value")
        })?;
        let key = key.trim();
        let value = value.trim();
        match key {
            "name" => {
                validate_bench_variant_name(value)?;
                name = Some(value.to_owned());
            }
            "projection" => {
                let columns = value
                    .split(',')
                    .map(str::trim)
                    .filter(|column| !column.is_empty())
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();
                if columns.is_empty() {
                    bail!("bench production-read variant projection must include a column");
                }
                projection_columns = Some(columns);
            }
            "guc" => {
                if !value.contains('=') {
                    bail!("bench production-read variant guc {value:?} must be name=value");
                }
                session_gucs.push(value.to_owned());
            }
            "timeline_payload" => match value {
                "projection" => timeline_no_payload = false,
                "none" => timeline_no_payload = true,
                other => bail!(
                    "bench production-read variant timeline_payload {other:?} must be projection or none"
                ),
            },
            other => bail!("unsupported bench production-read variant key {other:?}"),
        }
    }
    if name.is_none() {
        bail!("bench production-read variant must include name=<slug>");
    }
    Ok(BenchProductionReadVariant {
        name,
        projection_columns,
        session_gucs,
        timeline_no_payload,
    })
}

fn validate_bench_variant_name(name: &str) -> Result<()> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        bail!("bench production-read variant name {name:?} must use [A-Za-z0-9_-]");
    }
    Ok(())
}

fn parse_u16_list(raw: &str) -> Result<Vec<u16>> {
    raw.split(',')
        .map(|value| {
            value
                .trim()
                .parse::<u16>()
                .wrap_err_with(|| format!("parsing sweep value {value:?}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bench_production_read_variant() {
        let variant = parse_bench_production_read_variant(
            "name=source-prune-on;projection=id,source;guc=ec_spire.pre_materialization_prune=on",
        )
        .expect("variant parses");

        assert_eq!(
            variant,
            BenchProductionReadVariant {
                name: Some("source-prune-on".into()),
                projection_columns: Some(vec!["id".into(), "source".into()]),
                session_gucs: vec!["ec_spire.pre_materialization_prune=on".into()],
                timeline_no_payload: false,
            }
        );

        let no_payload = parse_bench_production_read_variant(
            "name=global-preheap-on;timeline_payload=none;guc=ec_spire.remote_search_global_pre_heap_merge=on",
        )
        .expect("variant parses");

        assert_eq!(
            no_payload,
            BenchProductionReadVariant {
                name: Some("global-preheap-on".into()),
                projection_columns: None,
                session_gucs: vec!["ec_spire.remote_search_global_pre_heap_merge=on".into()],
                timeline_no_payload: true,
            }
        );
    }
}

fn joined_reloption_args(shared: &[String], specific: &[String]) -> Vec<String> {
    shared.iter().chain(specific.iter()).cloned().collect()
}

fn default_local_multinode_run_id() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn log_local_multinode(path: Option<&Path>, line: &str) -> Result<()> {
    crate::ecaz_println!("{line}");
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .wrap_err_with(|| format!("opening {}", path.display()))?;
        writeln!(file, "{line}").wrap_err_with(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

fn push_path_arg(command: &mut Command, name: &str, value: Option<&PathBuf>) {
    if let Some(value) = value {
        command.arg(name).arg(value);
    }
}

fn push_u16_arg(command: &mut Command, name: &str, value: Option<u16>) {
    if let Some(value) = value {
        command.arg(name).arg(value.to_string());
    }
}
