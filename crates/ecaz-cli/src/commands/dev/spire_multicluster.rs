use clap::{Args, Subcommand, ValueEnum};
use color_eyre::eyre::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
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

    /// Local corpus TSV for exact truth in bench spire-pipeline.
    #[arg(long)]
    bench_truth_corpus_file: Option<PathBuf>,

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
    let pgbin = match args.pgbin {
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
    let pgbin = match args.pgbin {
        Some(path) => path,
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
    let pgbin = match args.pgbin {
        Some(path) => path,
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
    let pgbin = match args.pgbin {
        Some(path) => path,
        None => {
            let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
            find_pgrx_install(args.pg, &pgrx_home)?.bin_dir
        }
    };
    let script = repo_root.join("scripts/run_spire_phase13e_aws_harness_local_pg18.sh");
    if !script.is_file() {
        bail!(
            "SPIRE local multinode fixture script is missing: {}",
            script.display()
        );
    }

    crate::ecaz_println!("[spire-multicluster] repo={}", repo_root.display());
    crate::ecaz_println!("[spire-multicluster] pgbin={}", pgbin.display());
    crate::ecaz_println!("[spire-multicluster] topology=local-multinode");
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
    push_path_arg(&mut command, "--smoke-log", args.smoke_log.as_ref());
    push_u16_arg(&mut command, "--coord-port", args.coord_port);
    push_u16_arg(&mut command, "--remote1-port", args.remote1_port);
    push_u16_arg(&mut command, "--remote2-port", args.remote2_port);
    push_u16_arg(&mut command, "--remote3-port", args.remote3_port);
    if let Some(run_id) = args.run_id {
        command.arg("--run-id").arg(run_id);
    }
    push_string_arg(&mut command, "--tier", args.tier.as_deref());
    push_string_arg(&mut command, "--prefix", args.prefix.as_deref());
    push_string_arg(
        &mut command,
        "--prepared-prefix",
        args.prepared_prefix.as_deref(),
    );
    push_path_arg(&mut command, "--prepared-dir", args.prepared_dir.as_ref());
    set_env_if_some(
        &mut command,
        "SPIRE_AWS_STORAGE_FORMAT",
        args.storage_format.as_deref(),
    );
    set_env_if_some(&mut command, "COORD_INDEX", args.coord_index.as_deref());
    set_env_if_some(&mut command, "REMOTE_INDEX", args.remote_index.as_deref());
    let coord_reloptions = joined_reloptions(&args.reloptions, &args.coord_reloptions);
    let remote_reloptions = joined_reloptions(&args.reloptions, &args.remote_reloptions);
    set_env_if_nonempty(
        &mut command,
        "SPIRE_AWS_COORD_RELOPTIONS",
        &coord_reloptions,
    );
    set_env_if_nonempty(
        &mut command,
        "SPIRE_AWS_REMOTE_RELOPTIONS",
        &remote_reloptions,
    );
    push_u16_arg(&mut command, "--bench-top-k", args.bench_top_k);
    push_usize_arg(
        &mut command,
        "--bench-queries-limit",
        args.bench_queries_limit,
    );
    push_string_arg(&mut command, "--bench-sweep", args.bench_sweep.as_deref());
    push_string_arg(
        &mut command,
        "--bench-rowcap-sweep",
        args.bench_rowcap_sweep.as_deref(),
    );
    push_path_arg(
        &mut command,
        "--bench-truth-corpus-file",
        args.bench_truth_corpus_file.as_ref(),
    );
    if args.skip_bench_suite {
        command.arg("--skip-bench-suite");
    }
    if args.skip_fault_drills {
        command.arg("--skip-fault-drills");
    }
    if args.skip_install {
        command.arg("--skip-install");
    }

    run_status(command)
        .await
        .wrap_err("running SPIRE PG18 local multinode fixture")
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

fn push_usize_arg(command: &mut Command, name: &str, value: Option<usize>) {
    if let Some(value) = value {
        command.arg(name).arg(value.to_string());
    }
}

fn push_string_arg(command: &mut Command, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        command.arg(name).arg(value);
    }
}

fn set_env_if_some(command: &mut Command, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        command.env(name, value);
    }
}

fn set_env_if_nonempty(command: &mut Command, name: &str, value: &str) {
    if !value.is_empty() {
        command.env(name, value);
    }
}

fn joined_reloptions(shared: &[String], specific: &[String]) -> String {
    shared
        .iter()
        .chain(specific.iter())
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(";")
}
