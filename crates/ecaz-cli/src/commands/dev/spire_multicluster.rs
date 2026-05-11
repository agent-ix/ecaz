use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use super::support::{
    find_pgrx_install, repo_root, resolve_pgrx_home, run_status, DEFAULT_PG_MAJOR,
};

#[derive(Subcommand, Debug)]
pub enum SpireMulticlusterCommand {
    /// Run the PG18 one-coordinator/two-remote transport-overlap fixture.
    TransportOverlapPg18(TransportOverlapPg18Args),
    /// Run a PG18 Stage E fault-matrix fixture case.
    FaultPg18(StageEFaultPg18Args),
}

impl SpireMulticlusterCommand {
    pub async fn run(self, _database: &str) -> Result<()> {
        match self {
            SpireMulticlusterCommand::TransportOverlapPg18(args) => {
                run_transport_overlap_pg18(args).await
            }
            SpireMulticlusterCommand::FaultPg18(args) => run_stage_e_fault_pg18(args).await,
        }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StageEFaultCase {
    EpochMismatch,
    FingerprintMismatch,
    MissingOrReindexedRemoteIndex,
    RemoteBackendTermination,
    RemoteStatementTimeout,
    VersionSkew,
    SimulatedNetworkPartition,
}

impl StageEFaultCase {
    fn as_matrix_key(self) -> &'static str {
        match self {
            StageEFaultCase::EpochMismatch => "epoch_mismatch",
            StageEFaultCase::FingerprintMismatch => "fingerprint_mismatch",
            StageEFaultCase::MissingOrReindexedRemoteIndex => "missing_or_reindexed_remote_index",
            StageEFaultCase::RemoteBackendTermination => "remote_backend_termination",
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
            StageEFaultCase::RemoteBackendTermination | StageEFaultCase::RemoteStatementTimeout => {
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
        "epoch_mismatch" | "epoch-mismatch" => Ok(StageEFaultCase::EpochMismatch),
        "fingerprint_mismatch" | "fingerprint-mismatch" => {
            Ok(StageEFaultCase::FingerprintMismatch)
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
        "simulated_network_partition" | "simulated-network-partition" => {
            Ok(StageEFaultCase::SimulatedNetworkPartition)
        }
        other => Err(format!(
            "unsupported Stage E fault case {other:?}; supported: epoch_mismatch, fingerprint_mismatch, missing_or_reindexed_remote_index, remote_backend_termination, remote_statement_timeout, simulated_network_partition, version_skew"
        )),
    }
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
