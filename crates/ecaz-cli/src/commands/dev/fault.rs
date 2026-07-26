use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
};

use clap::{Args, Subcommand, ValueEnum};
use color_eyre::eyre::{bail, eyre, Context, Result};
use ecaz_fault_injection::{
    all_smoke_cases, leak_probe_sql, optional_leak_probe_sql, required_smoke_cases,
    workload_accumulator_pressure_settings_sql, workload_accumulator_pressure_sql,
    workload_bulk_insert_sql, workload_insert_sql, workload_reindex_sql,
    workload_repeated_scan_sql, workload_resource_setup_sql, workload_scan_sql, workload_setup_sql,
    workload_table_sql, workload_temp_spill_sql, workload_vacuum_full_sql, workload_vacuum_sql,
    DistannCodec, FaultAm, FaultFixture, FaultLane, ProviderMode,
};
use tokio::process::Command;

use super::support::{
    default_pgrx_port, find_pgrx_install, repo_root, resolve_pgrx_home, run_status,
    DEFAULT_PG_MAJOR,
};
use crate::psql::{self, ConnectionOptions};

#[derive(Subcommand, Debug)]
pub enum FaultCommand {
    /// Print the required PG-level fault-injection matrix.
    Plan(PlanArgs),
    /// Print LD_PRELOAD provider environment for postmaster startup.
    ProviderEnv(ProviderEnvArgs),
    /// Restart a local pgrx postmaster with the LD_PRELOAD provider active.
    ProviderRestart(ProviderRestartArgs),
    /// Restart a local pgrx postmaster without the LD_PRELOAD provider.
    ProviderRestore(ProviderRestoreArgs),
    /// Prepare AM-specific live fault fixtures before provider-backed runs.
    Prepare(PrepareArgs),
    /// Run or dry-run one smoke lane.
    Smoke(SmokeArgs),
    /// Prove fault-lane oracles reject deliberate controlled failures.
    MutationControl(MutationControlArgs),
    /// Print a host-independent cgroup-v2/systemd OOM operator plan.
    CgroupPlan(CgroupPlanArgs),
    /// Run isolated PostgreSQL AM workloads under a cgroup-v2 memory limit.
    CgroupSmoke(CgroupSmokeArgs),
    /// Internal worker launched inside the constrained systemd scope.
    #[command(hide = true)]
    CgroupWorker(CgroupWorkerArgs),
    /// Internal recovery probe launched outside the constrained scope.
    #[command(hide = true)]
    CgroupRecover(CgroupRecoverArgs),
}

#[derive(Args, Debug)]
pub struct PlanArgs {
    /// Restrict output to one lane.
    #[arg(long, value_enum)]
    lane: Option<FaultLaneArg>,
    /// Restrict output to one access method.
    #[arg(long, value_enum)]
    am: Option<FaultAmArg>,
    /// Restrict ec_distann output to one neighbor-code codec.
    #[arg(long, value_enum, requires = "am")]
    distann_codec: Option<DistannCodecArg>,
}

#[derive(Args, Debug)]
pub struct ProviderEnvArgs {
    /// Provider fault mode to configure.
    #[arg(long, value_enum)]
    mode: ProviderModeArg,
    /// Substring that must appear in the target path, for example "base/".
    #[arg(long, default_value = "base/")]
    path_match: String,
    /// Exact matched peer, e.g. tcp:127.0.0.1:39711 or unix:/path/.s.PGSQL.39424.
    #[arg(long)]
    peer_match: Option<String>,
    /// Start injecting on the Nth matching provider operation.
    #[arg(long, default_value_t = 1)]
    after: u64,
    /// Latency in milliseconds for slow-disk mode.
    #[arg(long)]
    latency_ms: Option<u64>,
    /// Optional marker file written by every process that loads the provider.
    #[arg(long)]
    marker: Option<String>,
    /// Optional file whose existence arms injection after postmaster startup.
    #[arg(long)]
    arm_file: Option<String>,
}

#[derive(Args, Debug)]
pub struct ProviderRestartArgs {
    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,
    /// Scratch-cluster port. Defaults to the pgrx convention, e.g. 28818 for PG18.
    #[arg(long)]
    port: Option<u16>,
    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,
    /// Provider fault mode to configure.
    #[arg(long, value_enum)]
    mode: ProviderModeArg,
    /// Substring that must appear in the target path, for example "base/".
    #[arg(long, default_value = "base/")]
    path_match: String,
    /// Exact matched peer, e.g. tcp:127.0.0.1:39711 or unix:/path/.s.PGSQL.39424.
    #[arg(long)]
    peer_match: Option<String>,
    /// Start injecting on the Nth matching provider operation.
    #[arg(long, default_value_t = 1)]
    after: u64,
    /// Latency in milliseconds for slow-disk mode.
    #[arg(long)]
    latency_ms: Option<u64>,
    /// Marker file written by every process that loads the provider.
    #[arg(long)]
    marker: Option<PathBuf>,
    /// Optional file whose existence arms injection after postmaster startup.
    #[arg(long)]
    arm_file: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct ProviderRestoreArgs {
    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,
    /// Scratch-cluster port. Defaults to the pgrx convention, e.g. 28818 for PG18.
    #[arg(long)]
    port: Option<u16>,
    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct PrepareArgs {
    /// Rows to load into each per-AM fault fixture.
    #[arg(long, default_value_t = 64)]
    rows: i64,
    /// Restrict preparation to one access method.
    #[arg(long, value_enum)]
    am: Option<FaultAmArg>,
    /// Restrict ec_distann preparation to one neighbor-code codec.
    #[arg(long, value_enum, requires = "am")]
    distann_codec: Option<DistannCodecArg>,
}

#[derive(Args, Debug)]
pub struct SmokeArgs {
    /// Fault lane to run.
    #[arg(long, value_enum)]
    lane: FaultLaneArg,
    /// Print the cases and post-condition probes without connecting to PG.
    #[arg(long)]
    dry_run: bool,
    /// Rows to load into each per-AM fault fixture for live probes.
    #[arg(long, default_value_t = 64)]
    rows: i64,
    /// Restrict the smoke lane to one access method.
    #[arg(long, value_enum)]
    am: Option<FaultAmArg>,
    /// Restrict ec_distann smoke to one neighbor-code codec.
    #[arg(long, value_enum, requires = "am")]
    distann_codec: Option<DistannCodecArg>,
    /// Marker file proving the target postmaster loaded the fault provider.
    #[arg(long)]
    provider_marker: Option<String>,
    /// Reuse already-created AM fixtures instead of preparing them in this process.
    #[arg(long)]
    assume_prepared: bool,
    /// Measured provider-off elapsed time for the same slow-disk workload.
    #[arg(long, requires = "provider_marker")]
    slow_disk_baseline_ms: Option<u128>,
}

#[derive(Args, Debug)]
pub struct MutationControlArgs {
    /// Controlled negative case to run.
    #[arg(long, value_enum, default_value = "all")]
    kind: MutationControlKindArg,
    /// Rows loaded into each per-AM fixture.
    #[arg(long, default_value_t = 64)]
    rows: i64,
    /// Restrict the control to one access method.
    #[arg(long, value_enum)]
    am: Option<FaultAmArg>,
    /// Restrict ec_distann controls to one neighbor-code codec.
    #[arg(long, value_enum, requires = "am")]
    distann_codec: Option<DistannCodecArg>,
}

#[derive(Args, Debug)]
pub struct CgroupPlanArgs {
    /// PostgreSQL major version to constrain.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,
    /// MemoryMax value for the isolated user scope.
    #[arg(long, default_value = "512M")]
    memory_max: String,
    /// Rows in the selected AM workload.
    #[arg(long, default_value_t = 64)]
    rows: i64,
    /// Restrict the plan to one access method.
    #[arg(long, value_enum)]
    am: Option<FaultAmArg>,
    /// Restrict ec_distann planning to one neighbor-code codec.
    #[arg(long, value_enum, requires = "am")]
    distann_codec: Option<DistannCodecArg>,
}

#[derive(Args, Debug)]
pub struct CgroupSmokeArgs {
    /// PostgreSQL major version to constrain.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,
    /// MemoryMax value for each isolated user scope.
    #[arg(long, default_value = "512M")]
    memory_max: String,
    /// Rows loaded before the repeated AM build workload begins.
    #[arg(long, default_value_t = 64)]
    rows: i64,
    /// First port considered for isolated PostgreSQL clusters.
    #[arg(long, default_value_t = 29_680)]
    base_port: u16,
    /// Packet-local or target-local directory for cluster and scope evidence.
    #[arg(long, default_value = "target/fault-cgroup")]
    artifact_dir: PathBuf,
    /// Target-local scratch directory for uncommittable PostgreSQL data.
    #[arg(long, default_value = "target/fault-cgroup-runtime")]
    runtime_dir: PathBuf,
    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,
    /// Restrict the smoke to one access method.
    #[arg(long, value_enum)]
    am: Option<FaultAmArg>,
    /// Restrict ec_distann smoke to one neighbor-code codec.
    #[arg(long, value_enum, requires = "am")]
    distann_codec: Option<DistannCodecArg>,
}

#[derive(Args, Debug)]
pub struct CgroupWorkerArgs {
    #[arg(long)]
    pg: u16,
    #[arg(long)]
    port: u16,
    #[arg(long)]
    rows: i64,
    #[arg(long)]
    artifact_dir: PathBuf,
    #[arg(long)]
    runtime_dir: PathBuf,
    #[arg(long)]
    pgrx_home: PathBuf,
    #[arg(long, value_enum)]
    am: FaultAmArg,
    #[arg(long, value_enum)]
    distann_codec: Option<DistannCodecArg>,
}

#[derive(Args, Debug)]
pub struct CgroupRecoverArgs {
    #[arg(long)]
    pg: u16,
    #[arg(long)]
    port: u16,
    #[arg(long)]
    rows: i64,
    #[arg(long)]
    artifact_dir: PathBuf,
    #[arg(long)]
    runtime_dir: PathBuf,
    #[arg(long)]
    pgrx_home: PathBuf,
    #[arg(long, value_enum)]
    am: FaultAmArg,
    #[arg(long, value_enum)]
    distann_codec: Option<DistannCodecArg>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FaultLaneArg {
    Io,
    Memory,
    Cancel,
    Timeout,
    LockTimeout,
    Resource,
    SlowDisk,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ProviderModeArg {
    EioRead,
    EnospcWrite,
    SlowDisk,
    SocketReset,
    SocketSlow,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FaultAmArg {
    Hnsw,
    Ivf,
    Diskann,
    Spire,
    Distann,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum DistannCodecArg {
    Rabitq,
    Turboquant,
    GroupedPq,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum MutationControlKindArg {
    All,
    CancelUnexpectedPalloc,
    MemoryUnrecoveredPalloc,
}

impl From<ProviderModeArg> for ProviderMode {
    fn from(value: ProviderModeArg) -> Self {
        match value {
            ProviderModeArg::EioRead => ProviderMode::EioRead,
            ProviderModeArg::EnospcWrite => ProviderMode::EnospcWrite,
            ProviderModeArg::SlowDisk => ProviderMode::SlowDisk,
            ProviderModeArg::SocketReset => ProviderMode::SocketReset,
            ProviderModeArg::SocketSlow => ProviderMode::SocketSlow,
        }
    }
}

impl From<FaultAmArg> for FaultAm {
    fn from(value: FaultAmArg) -> Self {
        match value {
            FaultAmArg::Hnsw => FaultAm::Hnsw,
            FaultAmArg::Ivf => FaultAm::Ivf,
            FaultAmArg::Diskann => FaultAm::DiskAnn,
            FaultAmArg::Spire => FaultAm::Spire,
            FaultAmArg::Distann => FaultAm::DistAnn,
        }
    }
}

impl From<DistannCodecArg> for DistannCodec {
    fn from(value: DistannCodecArg) -> Self {
        match value {
            DistannCodecArg::Rabitq => DistannCodec::RaBitQ,
            DistannCodecArg::Turboquant => DistannCodec::TurboQuant,
            DistannCodecArg::GroupedPq => DistannCodec::GroupedPq,
        }
    }
}

impl From<FaultLaneArg> for FaultLane {
    fn from(value: FaultLaneArg) -> Self {
        match value {
            FaultLaneArg::Io => FaultLane::Io,
            FaultLaneArg::Memory => FaultLane::Memory,
            FaultLaneArg::Cancel => FaultLane::Cancel,
            FaultLaneArg::Timeout => FaultLane::Timeout,
            FaultLaneArg::LockTimeout => FaultLane::LockTimeout,
            FaultLaneArg::Resource => FaultLane::Resource,
            FaultLaneArg::SlowDisk => FaultLane::SlowDisk,
        }
    }
}

impl FaultCommand {
    pub async fn run(self, conn: &ConnectionOptions) -> Result<()> {
        match self {
            FaultCommand::Plan(args) => run_plan(args),
            FaultCommand::ProviderEnv(args) => run_provider_env(args),
            FaultCommand::ProviderRestart(args) => run_provider_restart(args).await,
            FaultCommand::ProviderRestore(args) => run_provider_restore(args).await,
            FaultCommand::Prepare(args) => {
                let fixtures = selected_fixtures(args.am, args.distann_codec)?;
                prepare_workloads(conn, args.rows, &fixtures).await
            }
            FaultCommand::Smoke(args) => run_smoke(conn, args).await,
            FaultCommand::MutationControl(args) => run_mutation_control(conn, args).await,
            FaultCommand::CgroupPlan(args) => run_cgroup_plan(args),
            FaultCommand::CgroupSmoke(args) => run_cgroup_smoke(args).await,
            FaultCommand::CgroupWorker(args) => run_cgroup_worker(args).await,
            FaultCommand::CgroupRecover(args) => run_cgroup_recover(args).await,
        }
    }
}

fn run_cgroup_plan(args: CgroupPlanArgs) -> Result<()> {
    if args.rows <= 0 {
        return Err(eyre!("--rows must be >= 1"));
    }
    if args.memory_max.trim().is_empty() {
        return Err(eyre!("--memory-max must be nonempty"));
    }
    let fixtures = selected_fixtures(args.am, args.distann_codec)?;
    let linux = cfg!(target_os = "linux");
    let cgroup_v2 = Path::new("/sys/fs/cgroup/cgroup.controllers").is_file();
    let systemd_run = std::process::Command::new("systemd-run")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    let availability = if linux && cgroup_v2 && systemd_run {
        "host-reachable"
    } else {
        "unavailable"
    };
    crate::ecaz_println!(
        "[fault] cgroup_plan availability={availability} linux={linux} cgroup_v2={cgroup_v2} systemd_run={systemd_run} pg={} memory_max={} rows={}",
        args.pg,
        args.memory_max,
        args.rows
    );
    for fixture in fixtures {
        crate::ecaz_println!(
            "[fault] cgroup_case am={} fixture={} shape=isolated-one-index-per-table launch=\"systemd-run --user --scope -p MemoryMax={} <isolated-pg18-postmaster-and-ecaz-fault-workload>\" expected=\"backend or postmaster OOM; postmaster recovery; clean postconditions\"",
            fixture.as_str(),
            fixture.slug(),
            args.memory_max
        );
    }
    if availability == "unavailable" {
        crate::ecaz_println!(
            "[fault] cgroup_skip reason=\"requires Linux cgroup v2 and a working user systemd-run scope; no direct /sys/fs/cgroup writes\""
        );
    }
    Ok(())
}

async fn run_cgroup_smoke(args: CgroupSmokeArgs) -> Result<()> {
    if args.rows <= 0 {
        bail!("--rows must be >= 1");
    }
    if args.memory_max.trim().is_empty() {
        bail!("--memory-max must be nonempty");
    }
    require_cgroup_smoke_host().await?;

    let fixtures = selected_fixtures(args.am, args.distann_codec)?;
    let repo_root = repo_root()?;
    let requested_artifact_root = resolve_future_path(&args.artifact_dir)?;
    let requested_runtime_root = resolve_future_path(&args.runtime_dir)?;
    validate_cgroup_roots(
        &requested_artifact_root,
        &requested_runtime_root,
        &repo_root,
    )?;
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref())
        .canonicalize()
        .wrap_err("resolving PGRX_HOME for cgroup smoke")?;
    find_pgrx_install(args.pg, &pgrx_home)?;
    fs::create_dir_all(&args.artifact_dir).wrap_err_with(|| {
        format!(
            "creating cgroup artifact directory {}",
            args.artifact_dir.display()
        )
    })?;
    let artifact_root = args
        .artifact_dir
        .canonicalize()
        .wrap_err("resolving cgroup artifact directory")?;
    fs::create_dir_all(&args.runtime_dir).wrap_err_with(|| {
        format!(
            "creating cgroup runtime directory {}",
            args.runtime_dir.display()
        )
    })?;
    let runtime_root = args
        .runtime_dir
        .canonicalize()
        .wrap_err("resolving cgroup runtime directory")?;
    validate_cgroup_roots(&artifact_root, &runtime_root, &repo_root)?;
    let executable = std::env::current_exe()
        .wrap_err("resolving current ecaz executable")?
        .canonicalize()
        .wrap_err("canonicalizing current ecaz executable")?;
    let run_id = format!(
        "run-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .wrap_err("system clock predates Unix epoch")?
            .as_secs(),
        std::process::id()
    );
    let run_artifact_root = artifact_root.join(&run_id);
    let run_runtime_root = runtime_root.join(&run_id);
    fs::create_dir_all(&run_artifact_root)
        .wrap_err_with(|| format!("creating {}", run_artifact_root.display()))?;
    fs::create_dir_all(&run_runtime_root)
        .wrap_err_with(|| format!("creating {}", run_runtime_root.display()))?;

    for (ordinal, fixture) in fixtures.into_iter().enumerate() {
        let port_offset =
            u16::try_from(ordinal).wrap_err("too many cgroup fixtures for port allocation")?;
        let port = args
            .base_port
            .checked_add(port_offset)
            .ok_or_else(|| eyre!("cgroup fixture port overflow"))?;
        let unit = format!(
            "ecaz-fault-{}-{}",
            std::process::id(),
            fixture.slug().replace('_', "-")
        );
        let case_dir = run_artifact_root.join(fixture.slug());
        let case_runtime_dir = run_runtime_root.join(fixture.slug());
        fs::create_dir_all(&case_dir)
            .wrap_err_with(|| format!("creating {}", case_dir.display()))?;
        fs::create_dir_all(&case_runtime_dir)
            .wrap_err_with(|| format!("creating {}", case_runtime_dir.display()))?;

        let mut scope = Command::new("systemd-run");
        scope
            .arg("--user")
            .arg("--scope")
            .arg("--wait")
            .arg("--pipe")
            .arg(format!("--unit={unit}"))
            .arg(format!("--property=MemoryMax={}", args.memory_max))
            .arg("--property=OOMPolicy=kill")
            .arg(&executable)
            .arg("dev")
            .arg("fault")
            .arg("cgroup-worker")
            .arg("--pg")
            .arg(args.pg.to_string())
            .arg("--port")
            .arg(port.to_string())
            .arg("--rows")
            .arg(args.rows.to_string())
            .arg("--artifact-dir")
            .arg(&case_dir)
            .arg("--runtime-dir")
            .arg(&case_runtime_dir)
            .arg("--pgrx-home")
            .arg(&pgrx_home);
        append_fixture_cli_args(&mut scope, fixture);
        let scope_output = scope
            .output()
            .await
            .wrap_err_with(|| format!("launching constrained scope {unit}"))?;

        let scope_name = format!("{unit}.scope");
        let properties = Command::new("systemctl")
            .arg("--user")
            .arg("show")
            .arg(&scope_name)
            .arg("--property=Result")
            .arg("--property=MemoryCurrent")
            .arg("--property=MemoryPeak")
            .arg("--property=MemoryEvents")
            .arg("--no-pager")
            .output()
            .await
            .wrap_err_with(|| format!("reading systemd evidence for {scope_name}"))?;
        if !properties.status.success() {
            bail!(
                "systemctl show {scope_name} failed: {}",
                String::from_utf8_lossy(&properties.stderr)
            );
        }
        let property_text = String::from_utf8_lossy(&properties.stdout);
        let scope_stdout = String::from_utf8_lossy(&scope_output.stdout);
        let scope_stderr = String::from_utf8_lossy(&scope_output.stderr);
        fs::write(
            case_dir.join("scope.log"),
            format!(
                "unit={scope_name}\nfixture={}\nmemory_max={}\nstatus={}\n\n[stdout]\n{}\n[stderr]\n{}\n[systemctl-show]\n{}\n",
                fixture.as_str(),
                args.memory_max,
                scope_output.status,
                scope_stdout,
                scope_stderr,
                property_text
            ),
        )
        .wrap_err_with(|| format!("writing {}/scope.log", case_dir.display()))?;

        if scope_output.status.success() {
            bail!("{scope_name} completed without a cgroup OOM");
        }
        if !property_text
            .lines()
            .any(|line| line.trim() == "Result=oom-kill")
        {
            bail!(
                "{scope_name} failed without Result=oom-kill; inspect {}/scope.log",
                case_dir.display()
            );
        }
        if !scope_stdout.contains("[fault] cgroup_workload_active=true") {
            bail!(
                "{scope_name} OOMed before the AM workload marker; inspect {}/scope.log",
                case_dir.display()
            );
        }

        let mut recover = Command::new(&executable);
        recover
            .arg("dev")
            .arg("fault")
            .arg("cgroup-recover")
            .arg("--pg")
            .arg(args.pg.to_string())
            .arg("--port")
            .arg(port.to_string())
            .arg("--rows")
            .arg(args.rows.to_string())
            .arg("--artifact-dir")
            .arg(&case_dir)
            .arg("--runtime-dir")
            .arg(&case_runtime_dir)
            .arg("--pgrx-home")
            .arg(&pgrx_home);
        append_fixture_cli_args(&mut recover, fixture);
        let recovery_output = recover
            .output()
            .await
            .wrap_err_with(|| format!("running recovery probe for {scope_name}"))?;
        fs::write(
            case_dir.join("recovery.log"),
            format!(
                "status={}\n\n[stdout]\n{}\n[stderr]\n{}\n",
                recovery_output.status,
                String::from_utf8_lossy(&recovery_output.stdout),
                String::from_utf8_lossy(&recovery_output.stderr)
            ),
        )
        .wrap_err_with(|| format!("writing {}/recovery.log", case_dir.display()))?;
        if !recovery_output.status.success() {
            bail!(
                "recovery probe for {scope_name} failed; inspect {}/recovery.log",
                case_dir.display()
            );
        }
        fs::remove_dir_all(&case_runtime_dir).wrap_err_with(|| {
            format!(
                "removing recovered cgroup runtime {}",
                case_runtime_dir.display()
            )
        })?;

        let reset_status = Command::new("systemctl")
            .arg("--user")
            .arg("reset-failed")
            .arg(&scope_name)
            .status()
            .await
            .wrap_err_with(|| format!("resetting failed state for {scope_name}"))?;
        if !reset_status.success() {
            bail!("systemctl reset-failed {scope_name} failed with {reset_status}");
        }
        crate::ecaz_println!(
            "[fault] cgroup_oom am={} unit={} memory_max={} result=oom-kill workload_active=true recovery=pass artifacts={}",
            fixture.as_str(),
            scope_name,
            args.memory_max,
            case_dir.display()
        );
    }
    fs::remove_dir(&run_runtime_root)
        .wrap_err_with(|| format!("removing empty {}", run_runtime_root.display()))?;
    Ok(())
}

async fn require_cgroup_smoke_host() -> Result<()> {
    if !cfg!(target_os = "linux") {
        bail!(
            "cgroup smoke requires Linux; current target is {}",
            std::env::consts::OS
        );
    }
    if !Path::new("/sys/fs/cgroup/cgroup.controllers").is_file() {
        bail!("cgroup smoke requires cgroup v2 at /sys/fs/cgroup/cgroup.controllers");
    }
    let status = Command::new("systemctl")
        .arg("--user")
        .arg("show-environment")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .wrap_err("probing the systemd user manager")?;
    if !status.success() {
        bail!("cgroup smoke requires a reachable systemd user manager");
    }
    Ok(())
}

fn resolve_future_path(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    let mut existing = normalized.as_path();
    let mut missing = Vec::new();
    while !existing.exists() {
        let name = existing
            .file_name()
            .ok_or_else(|| eyre!("could not resolve future path {}", path.display()))?;
        missing.push(name.to_os_string());
        existing = existing
            .parent()
            .ok_or_else(|| eyre!("could not resolve future path {}", path.display()))?;
    }
    let mut resolved = existing
        .canonicalize()
        .wrap_err_with(|| format!("canonicalizing existing ancestor {}", existing.display()))?;
    for name in missing.into_iter().rev() {
        resolved.push(name);
    }
    Ok(resolved)
}

fn validate_cgroup_roots(
    artifact_root: &Path,
    runtime_root: &Path,
    repo_root: &Path,
) -> Result<()> {
    if paths_overlap(artifact_root, runtime_root) {
        bail!(
            "cgroup --artifact-dir and --runtime-dir must be disjoint, got {} and {}",
            artifact_root.display(),
            runtime_root.display()
        );
    }
    for evidence_tree in [repo_root.join("reviews"), repo_root.join("benchmarks")] {
        if runtime_root.starts_with(&evidence_tree) {
            bail!(
                "cgroup --runtime-dir {} must not be inside evidence tree {}",
                runtime_root.display(),
                evidence_tree.display()
            );
        }
    }
    Ok(())
}

fn paths_overlap(lhs: &Path, rhs: &Path) -> bool {
    lhs == rhs || lhs.starts_with(rhs) || rhs.starts_with(lhs)
}

fn append_fixture_cli_args(command: &mut Command, fixture: FaultFixture) {
    let am = match fixture.access_method() {
        FaultAm::Hnsw => "hnsw",
        FaultAm::Ivf => "ivf",
        FaultAm::DiskAnn => "diskann",
        FaultAm::Spire => "spire",
        FaultAm::DistAnn => "distann",
    };
    command.arg("--am").arg(am);
    if let Some(codec) = fixture.codec() {
        command.arg("--distann-codec").arg(codec.as_str());
    }
}

async fn run_cgroup_worker(args: CgroupWorkerArgs) -> Result<()> {
    let fixture = selected_single_fixture(args.am, args.distann_codec)?;
    if args.rows <= 0 {
        bail!("--rows must be >= 1");
    }
    let install = find_pgrx_install(args.pg, &args.pgrx_home)?;
    assert_fault_install_ready(&install)?;
    let data_dir = args.runtime_dir.join("data");
    let socket_dir = args.runtime_dir.join("socket");
    let postgres_log = args.artifact_dir.join("postgres.log");
    if data_dir.join("PG_VERSION").exists() {
        bail!(
            "refusing to reuse cgroup worker data directory {}",
            data_dir.display()
        );
    }
    fs::create_dir_all(&socket_dir)
        .wrap_err_with(|| format!("creating {}", socket_dir.display()))?;
    let mut initdb = Command::new(install.bin_dir.join("initdb"));
    initdb
        .arg("-D")
        .arg(&data_dir)
        .arg("-A")
        .arg("trust")
        .arg("-U")
        .arg("postgres");
    run_status(initdb).await?;

    let pg_ctl = install.bin_dir.join("pg_ctl");
    let mut start = Command::new(&pg_ctl);
    start
        .arg("-D")
        .arg(&data_dir)
        .arg("-l")
        .arg(&postgres_log)
        .arg("-o")
        .arg(format!(
            "-p {} -c listen_addresses='' -c unix_socket_directories={} -c shared_preload_libraries=ecaz",
            args.port,
            socket_dir.display()
        ))
        .arg("-w")
        .arg("start");
    run_status(start).await?;

    let conn = isolated_fault_connection(&socket_dir, args.port);
    let client = connect_isolated_fault(&conn, "cgroup worker").await?;
    client
        .batch_execute("CREATE EXTENSION ecaz;")
        .await
        .wrap_err("creating ecaz in cgroup worker cluster")?;
    client
        .batch_execute(&workload_table_sql(fixture, args.rows))
        .await
        .wrap_err_with(|| format!("preparing {} cgroup table", fixture.as_str()))?;
    client
        .batch_execute(&ecaz_fault_injection::workload_create_index_sql(
            fixture, args.rows,
        ))
        .await
        .wrap_err_with(|| format!("committing initial {} cgroup index", fixture.as_str()))?;

    let create_index = ecaz_fault_injection::workload_create_index_sql(fixture, args.rows);
    let create_index = create_index.replace('\'', "''");
    let index = ecaz_fault_injection::workload_index(fixture);
    let workload_sql = format!(
        "DO $ecaz_cgroup$
         BEGIN
           LOOP
             EXECUTE 'DROP INDEX IF EXISTS {index}';
             EXECUTE '{create_index}';
           END LOOP;
         END
         $ecaz_cgroup$"
    );
    let workload = tokio::spawn(async move {
        client
            .batch_execute(&workload_sql)
            .await
            .map_err(color_eyre::Report::from)
    });
    wait_for_cgroup_workload_active(&conn).await?;
    if workload.is_finished() {
        return workload
            .await
            .wrap_err("joining cgroup AM workload")?
            .and_then(|_| Err(eyre!("cgroup AM workload ended before memory pressure")));
    }
    crate::ecaz_println!(
        "[fault] cgroup_workload_active=true am={} port={} pressure=resident-8MiB-chunks",
        fixture.as_str(),
        args.port
    );

    let mut resident_chunks: Vec<Vec<u8>> = Vec::new();
    loop {
        let mut chunk = vec![0_u8; 8 * 1024 * 1024];
        for page in chunk.iter_mut().step_by(4096) {
            *page = 1;
        }
        resident_chunks.push(chunk);
        tokio::task::yield_now().await;
        if workload.is_finished() {
            return workload
                .await
                .wrap_err("joining cgroup AM workload")?
                .and_then(|_| Err(eyre!("cgroup AM workload ended before OOM")));
        }
    }
}

async fn wait_for_cgroup_workload_active(conn: &ConnectionOptions) -> Result<()> {
    let observer = connect_isolated_fault(conn, "cgroup workload observer").await?;
    for _ in 0..50 {
        let active: bool = observer
            .query_one(
                "SELECT EXISTS (
                   SELECT 1
                   FROM pg_stat_activity
                   WHERE pid <> pg_backend_pid()
                     AND state = 'active'
                     AND query LIKE 'DO $ecaz_cgroup$%'
                 )",
                &[],
            )
            .await?
            .get(0);
        if active {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    bail!("repeated AM-build workload did not become active within 500ms")
}

async fn run_cgroup_recover(args: CgroupRecoverArgs) -> Result<()> {
    let fixture = selected_single_fixture(args.am, args.distann_codec)?;
    if args.rows <= 0 {
        bail!("--rows must be >= 1");
    }
    let install = find_pgrx_install(args.pg, &args.pgrx_home)?;
    let pg_ctl = install.bin_dir.join("pg_ctl");
    let data_dir = args.runtime_dir.join("data");
    let socket_dir = args.runtime_dir.join("socket");
    let postgres_log = args.artifact_dir.join("postgres.log");
    if !data_dir.join("PG_VERSION").is_file() {
        bail!("missing cgroup worker cluster at {}", data_dir.display());
    }

    let status = Command::new(&pg_ctl)
        .arg("-D")
        .arg(&data_dir)
        .arg("status")
        .status()
        .await
        .wrap_err("checking constrained postmaster status")?;
    if status.success() {
        bail!("constrained postmaster survived OOMPolicy=kill unexpectedly");
    }

    let guard = FaultPgClusterGuard::new(pg_ctl.clone(), data_dir.clone());
    let mut start = Command::new(&pg_ctl);
    start
        .arg("-D")
        .arg(&data_dir)
        .arg("-l")
        .arg(&postgres_log)
        .arg("-o")
        .arg(format!(
            "-p {} -c listen_addresses='' -c unix_socket_directories={} -c shared_preload_libraries=ecaz",
            args.port,
            socket_dir.display()
        ))
        .arg("-w")
        .arg("start");
    run_status(start).await?;
    let conn = isolated_fault_connection(&socket_dir, args.port);
    let client = connect_isolated_fault(&conn, "cgroup recovery").await?;
    client.simple_query("SELECT 1").await?;
    let index = ecaz_fault_injection::workload_index(fixture);
    let index_state = client
        .query_one(
            "SELECT
                 r.index_oid IS NOT NULL,
                 COALESCE(i.indisvalid, false),
                 COALESCE(i.indisready, false)
             FROM (SELECT to_regclass($1::text) AS index_oid) r
             LEFT JOIN pg_index i ON i.indexrelid = r.index_oid",
            &[&index],
        )
        .await?;
    let index_exists = index_state.get::<_, bool>(0);
    let index_valid = index_state.get::<_, bool>(1);
    let index_ready = index_state.get::<_, bool>(2);
    if !(index_exists && index_valid && index_ready) {
        bail!(
            "cgroup recovery index {index} state was exists={index_exists} valid={index_valid} ready={index_ready}"
        );
    }
    let table = ecaz_fault_injection::workload_table(fixture);
    let recovered_rows: i64 = client
        .query_one(&format!("SELECT count(*) FROM {table}"), &[])
        .await
        .wrap_err_with(|| format!("querying recovered {} table", fixture.as_str()))?
        .get(0);
    if recovered_rows != args.rows {
        bail!(
            "cgroup recovery found {recovered_rows} rows for {}, expected {}",
            fixture.as_str(),
            args.rows
        );
    }
    client
        .batch_execute(&workload_scan_sql(fixture))
        .await
        .wrap_err_with(|| format!("running recovered {} AM scan", fixture.as_str()))?;
    drop(client);
    assert_postconditions(&conn, FaultLane::Memory, None, None).await?;
    guard.stop().await?;
    crate::ecaz_println!(
        "[fault] cgroup_recovery am={} postmaster_started=true query_usable=true expected_rows={} recovered_rows={} index_exists=true index_valid=true index_ready=true am_scan=true shared_postconditions=true clean_stop=true",
        fixture.as_str(),
        args.rows,
        recovered_rows
    );
    Ok(())
}

fn selected_single_fixture(
    am: FaultAmArg,
    distann_codec: Option<DistannCodecArg>,
) -> Result<FaultFixture> {
    let fixtures = selected_fixtures(Some(am), distann_codec)?;
    match fixtures.as_slice() {
        [fixture] => Ok(*fixture),
        _ => bail!("cgroup worker requires exactly one AM fixture"),
    }
}

fn assert_fault_install_ready(install: &super::support::PgrxInstall) -> Result<()> {
    let control = install.sharedir.join("extension/ecaz.control");
    let library = install.pkglibdir.join("ecaz.so");
    if !control.is_file() || !library.is_file() {
        bail!(
            "ecaz is not installed for PG18 via {}; missing {} or {}",
            install.pg_config.display(),
            control.display(),
            library.display()
        );
    }
    Ok(())
}

fn isolated_fault_connection(socket_dir: &Path, port: u16) -> ConnectionOptions {
    ConnectionOptions {
        database: "postgres".to_owned(),
        host: Some(socket_dir.to_string_lossy().to_string()),
        port: Some(port),
        user: Some("postgres".to_owned()),
        password: None,
    }
}

async fn connect_isolated_fault(
    conn: &ConnectionOptions,
    label: &str,
) -> Result<tokio_postgres::Client> {
    for _ in 0..50 {
        if let Ok(client) = psql::connect(conn).await {
            return Ok(client);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    bail!("{label} could not connect to isolated PostgreSQL within 5s")
}

struct FaultPgClusterGuard {
    pg_ctl: PathBuf,
    data_dir: PathBuf,
}

impl FaultPgClusterGuard {
    fn new(pg_ctl: PathBuf, data_dir: PathBuf) -> Self {
        Self { pg_ctl, data_dir }
    }

    async fn stop(&self) -> Result<()> {
        let mut stop = Command::new(&self.pg_ctl);
        stop.arg("-D")
            .arg(&self.data_dir)
            .arg("-m")
            .arg("fast")
            .arg("-w")
            .arg("stop");
        run_status(stop).await
    }
}

impl Drop for FaultPgClusterGuard {
    fn drop(&mut self) {
        let _ = std::process::Command::new(&self.pg_ctl)
            .arg("-D")
            .arg(&self.data_dir)
            .arg("-m")
            .arg("fast")
            .arg("-w")
            .arg("stop")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn run_plan(args: PlanArgs) -> Result<()> {
    let fixtures = selected_fixtures(args.am, args.distann_codec)?;
    let cases = args
        .lane
        .map(|lane| required_smoke_cases(lane.into()))
        .unwrap_or_else(all_smoke_cases)
        .into_iter()
        .filter(|case| {
            fixtures.iter().any(|fixture| {
                fixture.access_method() == case.access_method && fixture.codec() == case.codec
            })
        })
        .collect::<Vec<_>>();
    print_cases(&cases);
    print_leak_probes();
    Ok(())
}

fn run_provider_env(args: ProviderEnvArgs) -> Result<()> {
    let mode = ProviderMode::from(args.mode);
    validate_provider_options(mode, args.latency_ms, args.peer_match.as_deref())?;
    let marker = args
        .marker
        .as_deref()
        .map(Path::new)
        .map(absolute_marker_string)
        .transpose()?;
    let arm_file = args
        .arm_file
        .as_deref()
        .map(Path::new)
        .map(absolute_marker_string)
        .transpose()?;
    let env = ecaz_fault_injection::provider_environment(
        mode,
        &args.path_match,
        args.after,
        args.latency_ms,
        marker.as_deref(),
        arm_file.as_deref(),
        args.peer_match.as_deref(),
    );
    for (key, value) in env {
        crate::ecaz_println!("{key}={value}");
    }
    Ok(())
}

async fn run_provider_restart(args: ProviderRestartArgs) -> Result<()> {
    let mode = ProviderMode::from(args.mode);
    let latency_ms = match (mode, args.latency_ms) {
        (ProviderMode::SlowDisk | ProviderMode::SocketSlow, None) => Some(1),
        (_, value) => value,
    };
    validate_provider_options(mode, latency_ms, args.peer_match.as_deref())?;
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let install = find_pgrx_install(args.pg, &pgrx_home)?;
    let marker = args.marker.unwrap_or_else(|| {
        std::env::temp_dir().join(format!("ecaz-fault-provider-{}-pg{}.marker", mode, args.pg))
    });
    std::fs::write(&marker, "")?;
    let marker_string = absolute_marker_string(&marker)?;
    let arm_file_string = args
        .arm_file
        .as_deref()
        .map(absolute_marker_string)
        .transpose()?;
    if let Some(arm_file) = &arm_file_string {
        match std::fs::remove_file(arm_file) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let env = ecaz_fault_injection::provider_environment(
        mode,
        &args.path_match,
        args.after,
        latency_ms,
        Some(&marker_string),
        arm_file_string.as_deref(),
        args.peer_match.as_deref(),
    );
    restart_pgrx_postmaster(
        &install.bin_dir.join("pg_ctl"),
        &pgrx_home,
        args.pg,
        args.port.unwrap_or_else(|| default_pgrx_port(args.pg)),
        &env,
    )
    .await?;
    crate::ecaz_println!("[fault] provider_marker={marker_string}");
    Ok(())
}

fn validate_provider_options(
    mode: ProviderMode,
    latency_ms: Option<u64>,
    peer_match: Option<&str>,
) -> Result<()> {
    if matches!(mode, ProviderMode::SlowDisk | ProviderMode::SocketSlow)
        && latency_ms.unwrap_or(0) == 0
    {
        return Err(eyre!("--latency-ms must be >= 1 for {mode} mode"));
    }
    if matches!(mode, ProviderMode::SocketReset | ProviderMode::SocketSlow)
        && peer_match.is_none_or(str::is_empty)
    {
        return Err(eyre!("--peer-match is required for {mode} mode"));
    }
    if matches!(mode, ProviderMode::SocketReset | ProviderMode::SocketSlow) {
        let peer = peer_match.expect("required above");
        let valid_tcp = peer
            .strip_prefix("tcp:")
            .is_some_and(|identity| !identity.is_empty());
        let valid_unix = peer
            .strip_prefix("unix:")
            .is_some_and(|path| path.starts_with('/') && path.len() > 1);
        if !valid_tcp && !valid_unix {
            return Err(eyre!(
                "--peer-match must be tcp:HOST:PORT or an absolute named unix:/path"
            ));
        }
    }
    if !matches!(mode, ProviderMode::SocketReset | ProviderMode::SocketSlow) && peer_match.is_some()
    {
        return Err(eyre!(
            "--peer-match is valid only for socket-reset or socket-slow mode"
        ));
    }
    Ok(())
}

fn absolute_marker_string(path: &Path) -> Result<String> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    Ok(path.to_string_lossy().to_string())
}

async fn run_provider_restore(args: ProviderRestoreArgs) -> Result<()> {
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let install = find_pgrx_install(args.pg, &pgrx_home)?;
    let pg_ctl = install.bin_dir.join("pg_ctl");
    let port = args.port.unwrap_or_else(|| default_pgrx_port(args.pg));
    match restart_pgrx_postmaster(&pg_ctl, &pgrx_home, args.pg, port, &[]).await {
        Ok(()) => Ok(()),
        Err(error) => {
            crate::ecaz_println!(
                "[fault] provider_restore_fast_restart_failed={error}; falling back to immediate stop/start"
            );
            restore_pgrx_postmaster_immediate(&pg_ctl, &pgrx_home, args.pg, port).await
        }
    }
}

async fn restore_pgrx_postmaster_immediate(
    pg_ctl: &std::path::Path,
    pgrx_home: &std::path::Path,
    pg: u16,
    port: u16,
) -> Result<()> {
    let data_dir = pgrx_home.join(format!("data-{pg}"));
    let log_file = pgrx_home.join(format!("{pg}.log"));
    let options = format!(
        "-i -p {port} -c unix_socket_directories={}",
        pgrx_home.display()
    );

    let mut stop = Command::new(pg_ctl);
    stop.arg("-D")
        .arg(&data_dir)
        .arg("stop")
        .arg("-m")
        .arg("immediate")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    run_status(stop).await?;

    let mut start = Command::new(pg_ctl);
    start
        .arg("-D")
        .arg(data_dir)
        .arg("-l")
        .arg(log_file)
        .arg("-o")
        .arg(options)
        .arg("start")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    for name in [
        "LD_PRELOAD",
        "ECAZ_FAULT_PROVIDER_ENABLE",
        "ECAZ_FAULT_PROVIDER_MODE",
        "ECAZ_FAULT_PROVIDER_MATCH",
        "ECAZ_FAULT_PROVIDER_AFTER",
        "ECAZ_FAULT_PROVIDER_LATENCY_MS",
        "ECAZ_FAULT_PROVIDER_MARKER",
        "ECAZ_FAULT_PROVIDER_ARM_FILE",
        "ECAZ_FAULT_PROVIDER_PEER",
    ] {
        start.env_remove(name);
    }
    run_status(start).await
}

async fn restart_pgrx_postmaster(
    pg_ctl: &std::path::Path,
    pgrx_home: &std::path::Path,
    pg: u16,
    port: u16,
    env: &[(String, String)],
) -> Result<()> {
    let data_dir = pgrx_home.join(format!("data-{pg}"));
    let log_file = pgrx_home.join(format!("{pg}.log"));
    let options = format!(
        "-i -p {port} -c unix_socket_directories={}",
        pgrx_home.display()
    );
    let mut command = Command::new(pg_ctl);
    command
        .arg("-D")
        .arg(data_dir)
        .arg("-l")
        .arg(log_file)
        .arg("-o")
        .arg(options)
        .arg("restart")
        .arg("-m")
        .arg("fast")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    for name in [
        "LD_PRELOAD",
        "ECAZ_FAULT_PROVIDER_ENABLE",
        "ECAZ_FAULT_PROVIDER_MODE",
        "ECAZ_FAULT_PROVIDER_MATCH",
        "ECAZ_FAULT_PROVIDER_AFTER",
        "ECAZ_FAULT_PROVIDER_LATENCY_MS",
        "ECAZ_FAULT_PROVIDER_MARKER",
        "ECAZ_FAULT_PROVIDER_ARM_FILE",
        "ECAZ_FAULT_PROVIDER_PEER",
    ] {
        command.env_remove(name);
    }
    for (name, value) in env {
        command.env(name, value);
    }
    run_status(command).await
}

async fn run_smoke(conn: &ConnectionOptions, args: SmokeArgs) -> Result<()> {
    let lane = FaultLane::from(args.lane);
    let fixtures = selected_fixtures(args.am, args.distann_codec)?;
    let cases = required_smoke_cases(lane)
        .into_iter()
        .filter(|case| {
            fixtures.iter().any(|fixture| {
                fixture.access_method() == case.access_method && fixture.codec() == case.codec
            })
        })
        .collect::<Vec<_>>();
    print_cases(&cases);
    print_leak_probes();
    let pg_stat_io_before = if args.dry_run {
        None
    } else {
        capture_pg_stat_io_total(conn).await?
    };
    let pg_stat_wal_before = if args.dry_run {
        None
    } else {
        capture_pg_stat_wal_snapshot(conn).await?
    };

    if args.dry_run {
        return Ok(());
    }

    match lane {
        FaultLane::Io => {
            let marker = read_provider_marker(args.provider_marker.as_deref(), lane)?;
            let mode = provider_mode_from_marker(&marker)?;
            let path_match = provider_path_match_from_marker(&marker)?;
            if !args.assume_prepared {
                return Err(eyre!(
                    "lane {lane} must run against prebuilt fixtures; run `ecaz dev fault prepare --rows {}` before starting the provider, then rerun with --assume-prepared",
                    args.rows
                ));
            }
            run_io_probe(conn, mode, &path_match, &fixtures).await?;
            assert_provider_fault_marker(
                args.provider_marker.as_deref(),
                mode,
                &path_match,
                &format!("io {mode}"),
            )?;
            if provider_targets_wal(&path_match) && mode == ProviderMode::EnospcWrite {
                crate::ecaz_println!(
                    "[fault] wal_enospc_provider_restore_required=true match={path_match}"
                );
                return Ok(());
            }
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::Cancel => {
            run_cancel_probe(conn, args.rows, &fixtures).await?;
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::Timeout => {
            run_timeout_probe(conn, args.rows, &fixtures).await?;
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::LockTimeout => {
            run_lock_timeout_probe(conn, args.rows, &fixtures).await?;
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::Resource => {
            let provider_marker = args
                .provider_marker
                .as_deref()
                .map(|marker| read_provider_marker(Some(marker), lane))
                .transpose()?;
            run_resource_probe(conn, args.rows, &fixtures, provider_marker.as_deref()).await?;
            if let Some(marker) = provider_marker.as_deref() {
                if resource_provider_targets_temp_spill(marker)? {
                    assert_provider_fault_marker(
                        args.provider_marker.as_deref(),
                        ProviderMode::EnospcWrite,
                        &provider_path_match_from_marker(marker)?,
                        "resource provider temp spill",
                    )?;
                }
            }
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::Memory => {
            run_memory_probe(conn, args.rows, &fixtures).await?;
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::SlowDisk => {
            let marker = read_provider_marker(args.provider_marker.as_deref(), lane)?;
            let latency_ms = provider_latency_ms_from_marker(&marker)?;
            let baseline_ms = args.slow_disk_baseline_ms.ok_or_else(|| {
                eyre!(
                    "live slow-disk requires --slow-disk-baseline-ms from the same provider-off workload"
                )
            })?;
            run_slow_disk_probe(conn, args.rows, &fixtures, baseline_ms, latency_ms).await?;
            assert_provider_fault_marker(
                args.provider_marker.as_deref(),
                ProviderMode::SlowDisk,
                &provider_path_match_from_marker(&marker)?,
                "slow-disk timing",
            )?;
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
    }
}

fn selected_fixtures(
    am: Option<FaultAmArg>,
    distann_codec: Option<DistannCodecArg>,
) -> Result<Vec<FaultFixture>> {
    let codec = distann_codec.map(Into::into);
    match am.map(Into::into) {
        Some(FaultAm::DistAnn) => Ok(FaultFixture::for_access_method(FaultAm::DistAnn)
            .into_iter()
            .filter(|fixture| codec.is_none() || fixture.codec() == codec)
            .collect()),
        Some(access_method) if codec.is_some() => Err(eyre!(
            "--distann-codec is valid only with --am distann, got --am {}",
            access_method.as_str()
        )),
        Some(access_method) => Ok(FaultFixture::for_access_method(access_method)),
        None if codec.is_some() => Err(eyre!("--distann-codec requires --am distann")),
        None => Ok(FaultFixture::ALL.to_vec()),
    }
}

async fn run_io_probe(
    conn: &ConnectionOptions,
    mode: ProviderMode,
    path_match: &str,
    ams: &[FaultFixture],
) -> Result<()> {
    let client = connect_fault(conn, mode.as_str()).await?;
    for &am in ams {
        let label = format!("io {} {}", mode.as_str(), am.as_str());
        match mode {
            ProviderMode::EioRead => {
                let result = client.batch_execute(&workload_scan_sql(am)).await;
                assert_provider_sql_error(&label, result)?;
            }
            ProviderMode::EnospcWrite => {
                match client.batch_execute(&workload_insert_sql(am)).await {
                    Err(error) if error.as_db_error().is_some_and(provider_sqlstate_allowed) => {}
                    Err(error) if error.as_db_error().is_some() => {
                        let db = error.as_db_error().expect("checked above");
                        return Err(eyre!(
                            "{label} returned unexpected provider SQLSTATE {} ({})",
                            db.code().code(),
                            db.message()
                        ));
                    }
                    Err(error) if provider_targets_wal(path_match) => {
                        crate::ecaz_println!(
                            "[fault] wal_enospc_backend_disconnected=true label={label} error={error}"
                        );
                        return Ok(());
                    }
                    Err(error) => return Err(error.into()),
                    Ok(()) => {
                        assert_provider_sql_error(
                            &label,
                            client.batch_execute("CHECKPOINT").await,
                        )?;
                    }
                }
            }
            ProviderMode::SlowDisk | ProviderMode::SocketReset | ProviderMode::SocketSlow => {
                return Err(eyre!(
                    "lane io requires an eio-read or enospc-write provider, got {mode}"
                ))
            }
        };
        match client.simple_query("SELECT 1").await {
            Ok(_) => {}
            Err(error) if mode == ProviderMode::EnospcWrite && provider_targets_wal(path_match) => {
                crate::ecaz_println!(
                    "[fault] wal_enospc_backend_disconnected=true label={label} error={error}"
                );
                return Ok(());
            }
            Err(error) => return Err(eyre!("{label} did not leave the backend usable: {error}")),
        }
    }
    Ok(())
}

fn provider_targets_wal(path_match: &str) -> bool {
    path_match.contains("pg_wal")
}

async fn run_cancel_probe(conn: &ConnectionOptions, rows: i64, ams: &[FaultFixture]) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    for &am in ams {
        run_backend_interrupt_case(
            conn,
            rows,
            am,
            "cancel",
            "SELECT pg_cancel_backend($1)",
            true,
            None,
        )
        .await?;
        run_backend_interrupt_case(
            conn,
            rows,
            am,
            "terminate",
            "SELECT pg_terminate_backend($1)",
            false,
            None,
        )
        .await?;
    }
    Ok(())
}

async fn run_backend_interrupt_case(
    conn: &ConnectionOptions,
    rows: i64,
    am: FaultFixture,
    label: &str,
    interrupt_sql: &str,
    require_query_canceled_sqlstate: bool,
    worker_setup_sql: Option<&str>,
) -> Result<()> {
    let worker = connect_fault(conn, &format!("{label}-worker")).await?;
    let control = connect_fault(conn, &format!("{label}-control")).await?;
    let pid = worker
        .query_one("SELECT pg_backend_pid()", &[])
        .await?
        .get::<_, i32>(0);
    if let Some(worker_setup_sql) = worker_setup_sql {
        worker.batch_execute(worker_setup_sql).await?;
    }
    let sql = workload_repeated_scan_sql(am, repeated_scan_probe_iterations(rows));
    let worker_task = tokio::spawn(async move {
        worker
            .batch_execute(&sql)
            .await
            .map_err(color_eyre::Report::from)
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    control.execute(interrupt_sql, &[&pid]).await?;

    match worker_task.await? {
        Ok(()) => Err(eyre!(
            "{label} probe unexpectedly succeeded for {}",
            am.as_str()
        )),
        Err(error) if require_query_canceled_sqlstate => {
            let canceled = error
                .downcast_ref::<tokio_postgres::Error>()
                .and_then(tokio_postgres::Error::as_db_error)
                .map(|db| db.code().code() == "57014")
                .unwrap_or(false);
            if canceled {
                Ok(())
            } else {
                Err(error)
            }
        }
        Err(_) => Ok(()),
    }
}

async fn run_mutation_control(conn: &ConnectionOptions, args: MutationControlArgs) -> Result<()> {
    if args.rows <= 0 {
        bail!("--rows must be >= 1");
    }
    let fixtures = selected_fixtures(args.am, args.distann_codec)?;
    prepare_workloads(conn, args.rows, &fixtures).await?;

    if matches!(
        args.kind,
        MutationControlKindArg::All | MutationControlKindArg::CancelUnexpectedPalloc
    ) {
        run_cancel_unexpected_palloc_control(conn, args.rows, &fixtures).await?;
    }
    if matches!(
        args.kind,
        MutationControlKindArg::All | MutationControlKindArg::MemoryUnrecoveredPalloc
    ) {
        run_memory_unrecovered_palloc_control(conn, &fixtures).await?;
    }

    assert_postconditions(conn, FaultLane::Memory, None, None).await?;
    crate::ecaz_println!(
        "[fault] mutation_control_complete kind={:?} fixtures={} clean_postconditions=true",
        args.kind,
        fixtures.len()
    );
    Ok(())
}

async fn run_cancel_unexpected_palloc_control(
    conn: &ConnectionOptions,
    rows: i64,
    fixtures: &[FaultFixture],
) -> Result<()> {
    const SETUP: &str = "SELECT ecaz_fault_reset_palloc_counter(); SET ecaz.fault_palloc_nth = 1;";
    for &fixture in fixtures {
        let result = run_backend_interrupt_case(
            conn,
            rows,
            fixture,
            "cancel-mutation",
            "SELECT pg_cancel_backend($1)",
            true,
            Some(SETUP),
        )
        .await;
        match result {
            Ok(()) => {
                bail!(
                    "cancel mutation control unexpectedly accepted a deliberate palloc failure for {}",
                    fixture.as_str()
                )
            }
            Err(error) if report_is_ecaz_palloc_error(&error) => {
                crate::ecaz_println!(
                    "[fault] cancellation_mutation_control am={} injected=palloc_nth_1 normal_cancel_oracle=rejected",
                    fixture.as_str()
                );
            }
            Err(error) => {
                return Err(error).wrap_err_with(|| {
                    format!(
                    "cancel mutation control for {} did not reach the deliberate palloc failure",
                    fixture.as_str()
                )
                })
            }
        }
    }
    assert_postconditions(conn, FaultLane::Cancel, None, None).await
}

async fn run_memory_unrecovered_palloc_control(
    conn: &ConnectionOptions,
    fixtures: &[FaultFixture],
) -> Result<()> {
    for &fixture in fixtures {
        let worker = connect_fault(conn, "mutation-memory-unrecovered").await?;
        worker
            .batch_execute(
                "SELECT ecaz_fault_reset_palloc_counter(); SET ecaz.fault_palloc_nth = 1;",
            )
            .await?;
        let result = worker.batch_execute(&workload_scan_sql(fixture)).await;
        match result {
            Err(error) if is_ecaz_palloc_error(&error) => {}
            Err(error) => return Err(error.into()),
            Ok(()) => {
                bail!(
                    "memory mutation control did not inject palloc failure for {}",
                    fixture.as_str()
                )
            }
        }

        let oracle = run_palloc_recovery_probe(&worker, fixture).await;
        match oracle {
            Err(error) if is_ecaz_palloc_error(&error) => {}
            Err(error) => return Err(error.into()),
            Ok(()) => {
                bail!(
                    "memory mutation control recovery oracle accepted an armed palloc failure for {}",
                    fixture.as_str()
                )
            }
        }
        crate::ecaz_println!(
            "[fault] resource_palloc_mutation_control am={} injected=palloc_nth_1 recovery_without_disarm=true normal_recovery_oracle=rejected",
            fixture.as_str()
        );

        worker
            .batch_execute(
                "SET ecaz.fault_palloc_nth = -1; SELECT ecaz_fault_reset_palloc_counter();",
            )
            .await?;
        run_palloc_recovery_probe(&worker, fixture)
            .await
            .wrap_err_with(|| {
                format!(
                    "memory mutation control did not recover after disarming palloc for {}",
                    fixture.as_str()
                )
            })?;
        drop(worker);
        assert_postconditions(conn, FaultLane::Memory, None, None)
            .await
            .wrap_err_with(|| {
                format!(
                    "memory mutation control cleanup failed for {}",
                    fixture.as_str()
                )
            })?;
    }
    Ok(())
}

async fn run_palloc_recovery_probe(
    client: &tokio_postgres::Client,
    fixture: FaultFixture,
) -> Result<(), tokio_postgres::Error> {
    client.batch_execute(&workload_scan_sql(fixture)).await?;
    client.simple_query("SELECT 1").await?;
    Ok(())
}

fn report_is_ecaz_palloc_error(error: &color_eyre::Report) -> bool {
    error
        .downcast_ref::<tokio_postgres::Error>()
        .is_some_and(is_ecaz_palloc_error)
}

async fn run_timeout_probe(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultFixture],
) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    run_statement_timeout_probe(conn, rows, ams).await?;
    run_idle_in_transaction_timeout_probe(conn, ams).await
}

async fn run_statement_timeout_probe(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultFixture],
) -> Result<()> {
    let client = connect_fault(conn, "statement-timeout").await?;
    for &am in ams {
        let timeout = client
            .batch_execute(&format!(
                "SET statement_timeout = '5ms'; {}",
                workload_repeated_scan_sql(am, repeated_scan_probe_iterations(rows))
            ))
            .await;
        assert_query_canceled(&format!("statement_timeout {}", am.as_str()), timeout)?;
        client.batch_execute("RESET statement_timeout;").await?;
    }
    Ok(())
}

async fn run_idle_in_transaction_timeout_probe(
    conn: &ConnectionOptions,
    ams: &[FaultFixture],
) -> Result<()> {
    for &am in ams {
        let client = connect_fault(conn, "idle-tx-timeout").await?;
        client
            .batch_execute(&format!(
                "SET idle_in_transaction_session_timeout = '50ms';
                 BEGIN;
                 {}",
                workload_scan_sql(am)
            ))
            .await?;
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        match client.simple_query("SELECT 1").await {
            Ok(_) => {
                return Err(eyre!(
                    "idle_in_transaction_session_timeout {} unexpectedly left the backend usable",
                    am.as_str()
                ))
            }
            Err(_) => {}
        }
    }
    Ok(())
}

async fn run_lock_timeout_probe(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultFixture],
) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    let holder = connect_fault(conn, "lock-holder").await?;
    let waiter = connect_fault(conn, "lock-waiter").await?;
    for &am in ams {
        let table = ecaz_fault_injection::workload_table(am);
        run_lock_timeout_case(
            &holder,
            &waiter,
            &table,
            &format!("reindex {}", am.as_str()),
            &workload_reindex_sql(am),
        )
        .await?;
        let create_index = ecaz_fault_injection::workload_create_named_index_sql(
            am,
            &format!("{}_lock_probe_idx", table),
            rows,
        );
        run_lock_timeout_case(
            &holder,
            &waiter,
            &table,
            &format!("create_index {}", am.as_str()),
            &create_index,
        )
        .await?;
        run_lock_timeout_case(
            &holder,
            &waiter,
            &table,
            &format!("vacuum_full {}", am.as_str()),
            &workload_vacuum_full_sql(am),
        )
        .await?;
    }
    Ok(())
}

async fn run_lock_timeout_case(
    holder: &tokio_postgres::Client,
    waiter: &tokio_postgres::Client,
    table: &str,
    label: &str,
    sql: &str,
) -> Result<()> {
    holder
        .batch_execute(&format!(
            "BEGIN; LOCK TABLE {table} IN ACCESS EXCLUSIVE MODE;"
        ))
        .await?;
    waiter.batch_execute("SET lock_timeout = '10ms';").await?;
    let timeout = waiter.batch_execute(sql).await;
    let reset = waiter.batch_execute("RESET lock_timeout;").await;
    let rollback = holder.batch_execute("ROLLBACK;").await;
    reset?;
    rollback?;
    assert_sqlstate(&format!("lock_timeout {label}"), timeout, "55P03")
}

fn repeated_scan_probe_iterations(rows: i64) -> i64 {
    rows.saturating_mul(2_000).clamp(100_000, 1_000_000)
}

async fn run_resource_probe(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultFixture],
    provider_marker: Option<&str>,
) -> Result<()> {
    let pressure_rows = resource_accumulator_rows(rows);
    let pressure_limit = resource_accumulator_limit(pressure_rows);
    prepare_resource_workloads(conn, pressure_rows, pressure_limit, ams).await?;
    let client = connect_fault(conn, "resource").await?;
    let provider_temp_spill = provider_marker
        .map(resource_provider_targets_temp_spill)
        .transpose()?
        .unwrap_or(false);
    if provider_temp_spill {
        crate::ecaz_println!("[fault] resource_temp_spill_provider=enospc-write match=pgsql_tmp");
    }
    for &am in ams {
        client
            .batch_execute("SET work_mem = '64kB'; SET maintenance_work_mem = '1MB';")
            .await?;
        run_resource_accumulator_pressure_probe(&client, am, pressure_rows, pressure_limit).await?;
        client.batch_execute(&workload_scan_sql(am)).await?;
        client.batch_execute(&workload_insert_sql(am)).await?;
        client
            .batch_execute(&workload_vacuum_sql(am))
            .await
            .map_err(|error| {
                let detail = error
                    .as_db_error()
                    .map(|db| db.message().to_owned())
                    .unwrap_or_else(|| error.to_string());
                eyre!("resource probe vacuum {}: {detail}", am.as_str())
            })?;
        client
            .batch_execute(
                "SET work_mem = '64kB';
                 SET maintenance_work_mem = '1MB';
                 SELECT current_setting('work_mem'), current_setting('maintenance_work_mem');",
            )
            .await?;
        if provider_temp_spill {
            run_provider_temp_spill_probe(&client, rows, am).await?;
        } else {
            run_temp_file_limit_probe(&client, rows, am).await?;
        }
        run_wal_rotation_accounting_probe(&client, rows, am).await?;
    }
    Ok(())
}

async fn prepare_resource_workloads(
    conn: &ConnectionOptions,
    rows: i64,
    pressure_limit: i64,
    ams: &[FaultFixture],
) -> Result<()> {
    if rows <= 0 {
        return Err(eyre!("--rows must be >= 1"));
    }
    let client = connect_fault(conn, "resource-prepare").await?;
    client
        .batch_execute("CREATE EXTENSION IF NOT EXISTS ecaz;")
        .await?;
    crate::ecaz_println!(
        "[fault] resource_accumulator_prepare rows={rows} limit={pressure_limit} work_mem=64kB"
    );
    for &am in ams {
        client
            .batch_execute(&workload_resource_setup_sql(am, rows, pressure_limit))
            .await
            .map_err(|error| {
                let detail = error
                    .as_db_error()
                    .map(|db| db.message().to_owned())
                    .unwrap_or_else(|| error.to_string());
                eyre!(
                    "preparing {} resource pressure workload: {detail}",
                    am.as_str()
                )
            })?;
        print_workload_paths(&client, am).await?;
    }
    Ok(())
}

async fn run_resource_accumulator_pressure_probe(
    client: &tokio_postgres::Client,
    am: FaultFixture,
    rows: i64,
    pressure_limit: i64,
) -> Result<()> {
    client
        .batch_execute(
            "SET work_mem = '64kB';
             SET maintenance_work_mem = '1MB';
             SET effective_cache_size = '1MB';
             SET enable_seqscan = off;
             SET enable_bitmapscan = off;
             SET enable_sort = off;",
        )
        .await?;
    client
        .batch_execute(&workload_accumulator_pressure_settings_sql(
            am,
            pressure_limit,
        ))
        .await?;
    let row = client
        .query_one(&workload_accumulator_pressure_sql(am, pressure_limit), &[])
        .await?;
    let count = row.get::<_, i64>(0);
    let target = pressure_limit.min(rows);
    let returned_fraction_ppm = count.saturating_mul(1_000_000) / target.max(1);
    crate::ecaz_println!(
        "[fault] resource_accumulator_pressure am={} rows={rows} limit={pressure_limit} target={target} returned={count} returned_fraction_ppm={returned_fraction_ppm} workload_high_water_marker=returned_count work_mem=64kB effective_cache_size=1MB",
        am.as_str(),
    );
    let minimum = target.saturating_mul(95).saturating_add(99) / 100;
    if count < minimum {
        return Err(eyre!(
            "resource accumulator pressure {} returned {count}, expected at least 95% of target {target} ({minimum})",
            am.as_str()
        ));
    }
    Ok(())
}

fn resource_accumulator_rows(rows: i64) -> i64 {
    rows.saturating_mul(128).clamp(4_096, 20_000)
}

fn resource_accumulator_limit(rows: i64) -> i64 {
    rows.clamp(512, 1_000)
}

fn resource_provider_targets_temp_spill(marker: &str) -> Result<bool> {
    let mode = provider_mode_from_marker(marker)?;
    let path_match = provider_path_match_from_marker(marker)?;
    Ok(mode == ProviderMode::EnospcWrite && path_match.contains("pgsql_tmp"))
}

fn resource_temp_spill_rows(rows: i64) -> i64 {
    rows.saturating_mul(2_000).clamp(100_000, 500_000)
}

async fn run_temp_file_limit_probe(
    client: &tokio_postgres::Client,
    rows: i64,
    am: FaultFixture,
) -> Result<()> {
    let temp_bytes_before = pg_stat_database_temp_bytes(client).await?;
    let temp_spill = client
        .batch_execute(&format!(
            "SET work_mem = '64kB';
             SET temp_file_limit = '64kB';
             {}",
            workload_temp_spill_sql(resource_temp_spill_rows(rows))
        ))
        .await;
    client
        .batch_execute("RESET temp_file_limit; RESET work_mem;")
        .await?;
    assert_temp_file_limit_error(&format!("resource temp spill {}", am.as_str()), temp_spill)?;
    assert_temp_bytes_non_decreasing(client, am, "temp_file_limit", temp_bytes_before).await?;
    client.simple_query("SELECT 1").await.map_err(|error| {
        eyre!(
            "resource temp spill {} did not leave the backend usable: {error}",
            am.as_str()
        )
    })?;
    Ok(())
}

async fn run_provider_temp_spill_probe(
    client: &tokio_postgres::Client,
    rows: i64,
    am: FaultFixture,
) -> Result<()> {
    let temp_bytes_before = pg_stat_database_temp_bytes(client).await?;
    let temp_spill = client
        .batch_execute(&format!(
            "SET work_mem = '64kB';
             SET temp_file_limit = -1;
             {}",
            workload_temp_spill_sql(resource_temp_spill_rows(rows))
        ))
        .await;
    client
        .batch_execute("RESET temp_file_limit; RESET work_mem;")
        .await?;
    assert_provider_sql_error(
        &format!("resource provider temp spill {}", am.as_str()),
        temp_spill,
    )?;
    assert_temp_bytes_non_decreasing(client, am, "provider_enospc", temp_bytes_before).await?;
    client.simple_query("SELECT 1").await.map_err(|error| {
        eyre!(
            "resource provider temp spill {} did not leave the backend usable: {error}",
            am.as_str()
        )
    })?;
    Ok(())
}

async fn run_wal_rotation_accounting_probe(
    client: &tokio_postgres::Client,
    rows: i64,
    am: FaultFixture,
) -> Result<()> {
    let wal_before = pg_stat_wal_snapshot(client).await?;
    let lsn_before = current_wal_lsn(client).await?;
    client
        .batch_execute(&format!(
            "{};
             CHECKPOINT;
             SELECT pg_switch_wal();
             CHECKPOINT;",
            workload_bulk_insert_sql(am, wal_rotation_rows(rows))
        ))
        .await
        .map_err(|error| {
            let detail = error
                .as_db_error()
                .map(|db| db.message().to_owned())
                .unwrap_or_else(|| error.to_string());
            eyre!("WAL rotation accounting {}: {detail}", am.as_str())
        })?;
    force_pg_stat_flush(client).await?;
    let lsn_after = current_wal_lsn(client).await?;
    let lsn_advanced = client
        .query_one(
            "SELECT ($1::text)::pg_lsn < ($2::text)::pg_lsn",
            &[&lsn_before.as_str(), &lsn_after.as_str()],
        )
        .await?
        .get::<_, bool>(0);
    if !lsn_advanced {
        return Err(eyre!(
            "WAL rotation accounting {} did not advance LSN from {lsn_before} to {lsn_after}",
            am.as_str()
        ));
    }

    let Some(wal_before) = wal_before else {
        crate::ecaz_println!(
            "[fault] wal_rotation_accounting am={} lsn_before={lsn_before} lsn_after={lsn_after} pg_stat_wal=unavailable",
            am.as_str()
        );
        return Ok(());
    };
    let Some(wal_after) = pg_stat_wal_snapshot(client).await? else {
        crate::ecaz_println!(
            "[fault] wal_rotation_accounting am={} lsn_before={lsn_before} lsn_after={lsn_after} pg_stat_wal_after=unavailable",
            am.as_str()
        );
        return Ok(());
    };
    crate::ecaz_println!(
        "[fault] wal_rotation_accounting am={} lsn_before={lsn_before} lsn_after={lsn_after} records_before={} records_after={} bytes_before={} bytes_after={}",
        am.as_str(),
        wal_before.records,
        wal_after.records,
        wal_before.bytes,
        wal_after.bytes
    );
    if wal_after.records < wal_before.records || wal_after.bytes < wal_before.bytes {
        return Err(eyre!(
            "WAL rotation accounting {} decreased from records={} bytes={} to records={} bytes={}",
            am.as_str(),
            wal_before.records,
            wal_before.bytes,
            wal_after.records,
            wal_after.bytes
        ));
    }
    Ok(())
}

fn wal_rotation_rows(rows: i64) -> i64 {
    rows.clamp(64, 1_024)
}

async fn run_memory_probe(conn: &ConnectionOptions, rows: i64, ams: &[FaultFixture]) -> Result<()> {
    let client = connect_fault(conn, "memory").await?;
    for &am in ams {
        run_memory_build_probe(&client, rows, am).await?;
    }
    prepare_workloads(conn, rows, ams).await?;
    for &am in ams {
        run_memory_workload_palloc_sweep(&client, am, "scan", &workload_scan_sql(am)).await?;
        run_memory_workload_palloc_sweep(&client, am, "insert", &workload_insert_sql(am)).await?;
        run_memory_workload_palloc_sweep(&client, am, "vacuum", &workload_vacuum_sql(am)).await?;
    }
    drop(client);
    run_memory_rlimit_oom_probe(conn, rows, ams).await?;
    run_memory_oom_kill_probe(conn, rows, ams).await?;
    Ok(())
}

async fn run_memory_build_probe(
    client: &tokio_postgres::Client,
    rows: i64,
    am: FaultFixture,
) -> Result<()> {
    let build_sql = ecaz_fault_injection::workload_create_index_sql(am, rows);
    let mut reached_success = false;
    for nth in 1..=memory_major_workload_sweep_limit() {
        client.batch_execute(&workload_table_sql(am, rows)).await?;
        if run_memory_expected_palloc_probe(client, am, "build", nth, &build_sql, false).await? {
            continue;
        }
        reached_success = true;
        break;
    }
    if !reached_success {
        crate::ecaz_println!(
            "[fault] memory_palloc_sweep_exhausted am={} lane=build limit={}",
            am.as_str(),
            memory_major_workload_sweep_limit()
        );
        client.batch_execute(&workload_table_sql(am, rows)).await?;
        client.batch_execute(&build_sql).await?;
    }
    Ok(())
}

async fn run_memory_workload_palloc_sweep(
    client: &tokio_postgres::Client,
    am: FaultFixture,
    lane: &str,
    sql: &str,
) -> Result<()> {
    let mut reached_success = false;
    for nth in 1..=memory_major_workload_sweep_limit() {
        if run_memory_expected_palloc_probe(client, am, lane, nth, sql, true).await? {
            continue;
        }
        reached_success = true;
        break;
    }
    if !reached_success {
        crate::ecaz_println!(
            "[fault] memory_palloc_sweep_exhausted am={} lane={lane} limit={}",
            am.as_str(),
            memory_major_workload_sweep_limit()
        );
    }
    Ok(())
}

async fn run_memory_expected_palloc_probe(
    client: &tokio_postgres::Client,
    am: FaultFixture,
    lane: &str,
    nth: i32,
    sql: &str,
    verify_am_recovery: bool,
) -> Result<bool> {
    client
        .batch_execute(&format!(
            "SELECT ecaz_fault_reset_palloc_counter(); SET ecaz.fault_palloc_nth = {nth};"
        ))
        .await?;
    let result = client.batch_execute(sql).await;
    let reset = client
        .batch_execute("SET ecaz.fault_palloc_nth = -1; SELECT ecaz_fault_reset_palloc_counter();")
        .await;
    if let Err(reset_error) = reset {
        return match result {
            Ok(()) => Err(reset_error.into()),
            Err(workload_error) => Err(eyre!(
                "memory palloc {} {lane} nth {nth} failed ({workload_error}) and reset failed ({reset_error})",
                am.as_str()
            )),
        };
    }
    match result {
        Ok(()) => {
            crate::ecaz_println!(
                "[fault] memory_palloc_sweep_completed am={} lane={lane} first_success_nth={nth}",
                am.as_str()
            );
            return Ok(false);
        }
        Err(error) if is_ecaz_palloc_error(&error) => {}
        Err(error) => return Err(error.into()),
    }
    if verify_am_recovery {
        run_palloc_recovery_probe(client, am)
            .await
            .map_err(|error| {
                eyre!(
                    "memory palloc {} {lane} nth {nth} did not leave the AM/backend usable: {error}",
                    am.as_str()
                )
            })?;
    } else {
        client.simple_query("SELECT 1").await.map_err(|error| {
            eyre!(
                "memory palloc {} {lane} nth {nth} did not leave the backend usable: {error}",
                am.as_str()
            )
        })?;
    }
    crate::ecaz_println!(
        "[fault] memory_palloc_sweep_fault am={} lane={lane} nth={nth}",
        am.as_str()
    );
    Ok(true)
}

fn memory_major_workload_sweep_limit() -> i32 {
    100
}

#[cfg(target_os = "linux")]
async fn run_memory_rlimit_oom_probe(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultFixture],
) -> Result<()> {
    for &am in ams {
        let rows = rlimit_oom_workload_rows(rows);
        let setup = connect_fault(conn, "rlimit-oom-setup").await?;
        setup.batch_execute(&workload_table_sql(am, rows)).await?;
        drop(setup);
        run_memory_rlimit_oom_case(
            conn,
            am,
            "build",
            &ecaz_fault_injection::workload_create_index_sql(am, rows),
        )
        .await?;
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
async fn run_memory_rlimit_oom_probe(
    _conn: &ConnectionOptions,
    _rows: i64,
    ams: &[FaultFixture],
) -> Result<()> {
    let am_names = ams
        .iter()
        .map(|am| am.as_str())
        .collect::<Vec<_>>()
        .join(",");
    crate::ecaz_println!(
        "[fault] memory_rlimit_oom_skipped target_os={} ams={am_names} reason=linux-only",
        std::env::consts::OS
    );
    Ok(())
}

#[cfg(target_os = "linux")]
async fn run_memory_rlimit_oom_case(
    conn: &ConnectionOptions,
    am: FaultFixture,
    workload: &str,
    sql: &str,
) -> Result<()> {
    let worker = connect_fault(conn, &format!("rlimit-oom-{workload}-worker")).await?;
    let pid = worker
        .query_one("SELECT pg_backend_pid()", &[])
        .await?
        .get::<_, i32>(0);
    worker
        .simple_query(
            "SELECT encode_to_ecvector(ARRAY[0.0::real, 0.0::real, 0.0::real, 0.0::real], 4, 42)",
        )
        .await?;
    let limit_bytes = rlimit_oom_limit_bytes(pid)?;
    set_backend_address_space_limit(pid, limit_bytes)?;
    let label = format!("memory rlimit-oom {} {workload}", am.as_str());
    crate::ecaz_println!("[fault] {label} pid={pid} rlimit_as_bytes={limit_bytes}");

    match worker.batch_execute(sql).await {
        Ok(()) => Err(eyre!("{label} unexpectedly completed under RLIMIT_AS")),
        Err(error) if is_oom_class_error(&error) => {
            crate::ecaz_println!(
                "[fault] {label} oom_error=true sqlstate={} message={}",
                error
                    .as_db_error()
                    .map(|db| db.code().code())
                    .unwrap_or("none"),
                error
                    .as_db_error()
                    .map(|db| db.message())
                    .unwrap_or("connection error")
            );
            assert_new_fault_session_usable(conn, &label).await
        }
        Err(error) if error.as_db_error().is_none() => {
            crate::ecaz_println!("[fault] {label} backend_disconnected=true error={error}");
            wait_for_postmaster_recovery(conn, &label).await
        }
        Err(error) => Err(error.into()),
    }
}

#[cfg(target_os = "linux")]
fn rlimit_oom_workload_rows(rows: i64) -> i64 {
    rows.saturating_mul(500).clamp(20_000, 200_000)
}

#[cfg(target_os = "linux")]
fn rlimit_oom_limit_bytes(pid: i32) -> Result<u64> {
    let vm_size_bytes = linux_backend_vm_size_bytes(pid)?;
    Ok(vm_size_bytes.saturating_add(rlimit_oom_headroom_bytes()))
}

#[cfg(target_os = "linux")]
fn rlimit_oom_headroom_bytes() -> u64 {
    1024 * 1024
}

async fn run_memory_oom_kill_probe(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultFixture],
) -> Result<()> {
    for &am in ams {
        let rows = oom_kill_workload_rows(rows);
        run_memory_oom_kill_build_probe(conn, rows, am).await?;
        prepare_workloads(conn, rows.min(1_024), &[am]).await?;
        run_memory_oom_kill_case(
            conn,
            am,
            "scan",
            &workload_repeated_scan_sql(am, oom_kill_scan_iterations(rows)),
        )
        .await?;
        prepare_workloads(conn, rows.min(1_024), &[am]).await?;
        run_memory_oom_kill_case(conn, am, "insert", &workload_bulk_insert_sql(am, rows)).await?;
    }
    Ok(())
}

async fn run_memory_oom_kill_build_probe(
    conn: &ConnectionOptions,
    rows: i64,
    am: FaultFixture,
) -> Result<()> {
    let setup = connect_fault(conn, "oom-kill-setup").await?;
    setup.batch_execute(&workload_table_sql(am, rows)).await?;
    drop(setup);
    run_memory_oom_kill_case(
        conn,
        am,
        "build",
        &ecaz_fault_injection::workload_create_index_sql(am, rows),
    )
    .await
}

async fn run_memory_oom_kill_case(
    conn: &ConnectionOptions,
    am: FaultFixture,
    workload: &str,
    sql: &str,
) -> Result<()> {
    let worker = connect_fault(conn, &format!("oom-kill-{workload}-worker")).await?;
    let pid = worker
        .query_one("SELECT pg_backend_pid()", &[])
        .await?
        .get::<_, i32>(0);
    let label = format!("memory oom-kill {} {workload}", am.as_str());
    let sql = sql.to_owned();
    let worker_task = tokio::spawn(async move {
        worker
            .batch_execute(&sql)
            .await
            .map_err(color_eyre::Report::from)
    });

    let delay_ms = oom_kill_delay_ms();
    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    crate::ecaz_println!(
        "[fault] {label} sigkill_pid={pid} delay_ms={delay_ms} timing_semantics=probability-tuning-not-critical-section-proof"
    );
    send_sigkill(pid).await?;

    match worker_task.await? {
        Ok(()) => return Err(eyre!("{label} unexpectedly completed before SIGKILL")),
        Err(_) => {}
    }
    wait_for_postmaster_recovery(conn, &label).await
}

fn oom_kill_workload_rows(rows: i64) -> i64 {
    rows.saturating_mul(500).clamp(20_000, 200_000)
}

fn oom_kill_scan_iterations(rows: i64) -> i64 {
    rows.saturating_mul(20).clamp(200_000, 1_000_000)
}

fn oom_kill_delay_ms() -> u64 {
    25
}

async fn send_sigkill(pid: i32) -> Result<()> {
    let status = Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .status()
        .await?;
    if status.success() {
        Ok(())
    } else {
        Err(eyre!("kill -9 {pid} failed with status {status}"))
    }
}

async fn wait_for_postmaster_recovery(conn: &ConnectionOptions, label: &str) -> Result<()> {
    for _ in 0..100 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if let Ok(client) = connect_fault(conn, "oom-kill-recovery").await {
            if client.simple_query("SELECT 1").await.is_ok() {
                crate::ecaz_println!("[fault] {label} postmaster_recovered=true");
                return Ok(());
            }
        }
    }
    Err(eyre!(
        "{label} did not recover a usable postmaster within 10s"
    ))
}

#[cfg(target_os = "linux")]
async fn assert_new_fault_session_usable(conn: &ConnectionOptions, label: &str) -> Result<()> {
    let client = connect_fault(conn, "oom-recovery").await?;
    client
        .simple_query("SELECT 1")
        .await
        .map_err(|error| eyre!("{label} did not leave new sessions usable: {error}"))?;
    crate::ecaz_println!("[fault] {label} new_session_usable=true");
    Ok(())
}

#[cfg(target_os = "linux")]
fn is_oom_class_error(error: &tokio_postgres::Error) -> bool {
    error.as_db_error().is_some_and(|db| {
        db.code().code() == "53200" || db.message().to_ascii_lowercase().contains("memory")
    })
}

#[cfg(target_os = "linux")]
fn linux_backend_vm_size_bytes(pid: i32) -> Result<u64> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status"))
        .map_err(|error| eyre!("reading /proc/{pid}/status: {error}"))?;
    let vm_size_kb = status
        .lines()
        .find_map(|line| line.strip_prefix("VmSize:"))
        .and_then(|value| value.split_whitespace().next())
        .ok_or_else(|| eyre!("could not find VmSize in /proc/{pid}/status"))?
        .parse::<u64>()
        .map_err(|error| eyre!("parsing VmSize for backend {pid}: {error}"))?;
    Ok(vm_size_kb.saturating_mul(1024))
}

#[cfg(target_os = "linux")]
fn set_backend_address_space_limit(pid: i32, limit_bytes: u64) -> Result<()> {
    let limit = libc::rlimit64 {
        rlim_cur: limit_bytes as libc::rlim64_t,
        rlim_max: limit_bytes as libc::rlim64_t,
    };
    let result = unsafe { libc::prlimit64(pid, libc::RLIMIT_AS, &limit, std::ptr::null_mut()) };
    if result == 0 {
        Ok(())
    } else {
        Err(eyre!(
            "setting RLIMIT_AS for backend {pid} to {limit_bytes} bytes: {}",
            std::io::Error::last_os_error()
        ))
    }
}

async fn run_slow_disk_probe(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultFixture],
    baseline_ms: u128,
    configured_latency_ms: u64,
) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    let client = connect_fault(conn, "slow-disk").await?;
    let started = std::time::Instant::now();
    for &am in ams {
        client.batch_execute(&workload_scan_sql(am)).await?;
        client.batch_execute(&workload_insert_sql(am)).await?;
        client.batch_execute(&workload_vacuum_sql(am)).await?;
    }
    let provider_ms = started.elapsed().as_millis();
    crate::ecaz_println!(
        "[fault] slow_disk_timing baseline_ms={baseline_ms} provider_ms={provider_ms} configured_latency_ms={configured_latency_ms} comparison=provider-greater-than-baseline"
    );
    if provider_ms <= baseline_ms {
        return Err(eyre!(
            "slow-disk provider workload took {provider_ms}ms, not greater than measured baseline {baseline_ms}ms"
        ));
    }
    Ok(())
}

async fn prepare_workloads(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultFixture],
) -> Result<()> {
    if rows <= 0 {
        return Err(eyre!("--rows must be >= 1"));
    }
    let client = connect_fault(conn, "prepare").await?;
    client
        .batch_execute("CREATE EXTENSION IF NOT EXISTS ecaz;")
        .await?;
    for &am in ams {
        client
            .batch_execute(&workload_setup_sql(am, rows))
            .await
            .map_err(|error| {
                let detail = error
                    .as_db_error()
                    .map(|db| db.message().to_owned())
                    .unwrap_or_else(|| error.to_string());
                eyre!("preparing {} fault workload: {detail}", am.as_str())
            })?;
        print_workload_paths(&client, am).await?;
    }
    Ok(())
}

async fn print_workload_paths(client: &tokio_postgres::Client, am: FaultFixture) -> Result<()> {
    let table = ecaz_fault_injection::workload_table(am);
    let index = ecaz_fault_injection::workload_index(am);
    let table_path = relation_filepath(client, &table).await?;
    let index_path = relation_filepath(client, &index).await?;
    crate::ecaz_println!(
        "{}\ttable={}\ttable_path={}\tindex={}\tindex_path={}",
        am.as_str(),
        table,
        table_path,
        index,
        index_path
    );
    Ok(())
}

async fn relation_filepath(client: &tokio_postgres::Client, relation: &str) -> Result<String> {
    let row = client
        .query_one(
            "SELECT pg_relation_filepath($1::text::regclass)",
            &[&relation],
        )
        .await?;
    Ok(row.get::<_, String>(0))
}

fn read_provider_marker(marker: Option<&str>, lane: FaultLane) -> Result<String> {
    let marker = marker.ok_or_else(|| {
        eyre!(
            "lane {lane} requires --provider-marker from a postmaster started with `ecaz dev fault provider-env`"
        )
    })?;
    let content = std::fs::read_to_string(marker)
        .map_err(|error| eyre!("reading provider marker {marker:?}: {error}"))?;
    if content.trim().is_empty() {
        return Err(eyre!("provider marker {marker:?} is empty"));
    }
    Ok(content)
}

fn provider_mode_from_marker(content: &str) -> Result<ProviderMode> {
    if content.lines().any(|line| line.contains("mode=eio-read")) {
        Ok(ProviderMode::EioRead)
    } else if content
        .lines()
        .any(|line| line.contains("mode=enospc-write"))
    {
        Ok(ProviderMode::EnospcWrite)
    } else if content.lines().any(|line| line.contains("mode=slow-disk")) {
        Ok(ProviderMode::SlowDisk)
    } else if content
        .lines()
        .any(|line| line.contains("mode=socket-reset"))
    {
        Ok(ProviderMode::SocketReset)
    } else if content
        .lines()
        .any(|line| line.contains("mode=socket-slow"))
    {
        Ok(ProviderMode::SocketSlow)
    } else {
        Err(eyre!(
            "provider marker did not include a supported mode line"
        ))
    }
}

fn provider_path_match_from_marker(content: &str) -> Result<String> {
    content
        .lines()
        .find_map(|line| {
            line.split_whitespace()
                .find_map(|field| field.strip_prefix("match="))
        })
        .map(ToOwned::to_owned)
        .ok_or_else(|| eyre!("provider marker did not include a match field"))
}

fn provider_latency_ms_from_marker(content: &str) -> Result<u64> {
    content
        .lines()
        .find_map(|line| {
            line.split_whitespace()
                .find_map(|field| field.strip_prefix("latency_ms="))
        })
        .ok_or_else(|| eyre!("provider marker did not include a latency_ms field"))?
        .parse()
        .map_err(|error| eyre!("provider marker latency_ms is invalid: {error}"))
}

fn assert_provider_fault_marker(
    marker: Option<&str>,
    mode: ProviderMode,
    path_match: &str,
    label: &str,
) -> Result<()> {
    let marker = marker.ok_or_else(|| eyre!("{label} requires --provider-marker"))?;
    let content = std::fs::read_to_string(marker)
        .map_err(|error| eyre!("reading provider marker {marker:?}: {error}"))?;
    let count = content
        .lines()
        .filter(|line| {
            line.contains("fault=1")
                && line.contains(&format!("mode={}", mode.as_str()))
                && line.contains(path_match)
        })
        .count();
    crate::ecaz_println!(
        "[fault] provider_fault_events label={label} mode={} match={path_match} count={count}",
        mode.as_str()
    );
    if count == 0 {
        return Err(eyre!(
            "{label} did not record a provider fault event for mode={} match={path_match}",
            mode.as_str()
        ));
    }
    Ok(())
}

async fn connect_fault(conn: &ConnectionOptions, label: &str) -> Result<tokio_postgres::Client> {
    let client = psql::connect(conn).await?;
    client
        .execute(
            "SELECT set_config('application_name', $1, false)",
            &[&format!("ecaz-fault-{label}")],
        )
        .await?;
    Ok(client)
}

async fn assert_postconditions(
    conn: &ConnectionOptions,
    lane: FaultLane,
    pg_stat_io_before: Option<i64>,
    pg_stat_wal_before: Option<PgStatWalSnapshot>,
) -> Result<()> {
    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    let client = connect_fault(conn, "postcondition").await?;
    for &sql in leak_probe_sql() {
        let row = client.query_one(sql, &[]).await?;
        let count = row.get::<_, i64>(0);
        if count != 0 {
            return Err(eyre!("{lane} postcondition failed: {sql} returned {count}"));
        }
    }
    assert_pg_buffercache_fixture_pins(&client, lane).await?;
    assert_pg_stat_io_non_decreasing(&client, lane, pg_stat_io_before).await?;
    assert_pg_stat_wal_non_decreasing(&client, lane, pg_stat_wal_before).await?;
    Ok(())
}

async fn capture_pg_stat_io_total(conn: &ConnectionOptions) -> Result<Option<i64>> {
    let client = connect_fault(conn, "precondition").await?;
    pg_stat_io_total(&client).await
}

#[derive(Clone, Copy, Debug)]
struct PgStatWalSnapshot {
    records: i64,
    bytes: i64,
}

async fn capture_pg_stat_wal_snapshot(
    conn: &ConnectionOptions,
) -> Result<Option<PgStatWalSnapshot>> {
    let client = connect_fault(conn, "wal-precondition").await?;
    pg_stat_wal_snapshot(&client).await
}

async fn assert_pg_buffercache_fixture_pins(
    client: &tokio_postgres::Client,
    lane: FaultLane,
) -> Result<()> {
    let available = client
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM pg_available_extensions WHERE name = 'pg_buffercache')",
            &[],
        )
        .await?
        .get::<_, bool>(0);
    if !available {
        crate::ecaz_println!("[fault] pg_buffercache unavailable; skipping pin probe");
        return Ok(());
    }
    if let Err(error) = client
        .batch_execute("CREATE EXTENSION IF NOT EXISTS pg_buffercache")
        .await
    {
        if error
            .as_db_error()
            .map(|db| db.code().code() == "42501")
            .unwrap_or(false)
        {
            crate::ecaz_println!("[fault] pg_buffercache privilege denied; skipping pin probe");
            return Ok(());
        }
        return Err(error.into());
    }

    let pinned = client
        .query_one(
            "SELECT count(*)::bigint
             FROM pg_buffercache b
             JOIN pg_class c ON c.relfilenode = b.relfilenode
             WHERE b.reldatabase = (SELECT oid FROM pg_database WHERE datname = current_database())
               AND c.relname LIKE 'ecaz_fault_%'
               AND b.pinning_backends > 0",
            &[],
        )
        .await?
        .get::<_, i64>(0);
    crate::ecaz_println!("[fault] pg_buffercache_fixture_pins={pinned}");
    if pinned != 0 {
        crate::ecaz_println!("[fault] pg_buffercache_fixture_pins_ok=false pins={pinned}");
        return Err(eyre!(
            "{lane} postcondition failed: pg_buffercache fixture pin count returned {pinned}"
        ));
    }
    crate::ecaz_println!("[fault] pg_buffercache_fixture_pins_ok=true pins=0");
    Ok(())
}

async fn assert_pg_stat_io_non_decreasing(
    client: &tokio_postgres::Client,
    lane: FaultLane,
    before: Option<i64>,
) -> Result<()> {
    let Some(before) = before else {
        crate::ecaz_println!("[fault] pg_stat_io unavailable; skipping io counter probe");
        return Ok(());
    };
    let Some(after) = pg_stat_io_total(client).await? else {
        crate::ecaz_println!(
            "[fault] pg_stat_io unavailable after lane; skipping io counter probe"
        );
        return Ok(());
    };
    crate::ecaz_println!("[fault] pg_stat_io_ops_before={before} after={after}");
    if after < before {
        if lane == FaultLane::Memory {
            crate::ecaz_println!(
                "[fault] pg_stat_io_reset_after_crash_recovery=true before={before} after={after}"
            );
            return Ok(());
        }
        return Err(eyre!(
            "{lane} postcondition failed: pg_stat_io total operations decreased from {before} to {after}"
        ));
    }
    Ok(())
}

async fn assert_pg_stat_wal_non_decreasing(
    client: &tokio_postgres::Client,
    lane: FaultLane,
    before: Option<PgStatWalSnapshot>,
) -> Result<()> {
    let Some(before) = before else {
        crate::ecaz_println!("[fault] pg_stat_wal unavailable; skipping wal counter probe");
        return Ok(());
    };
    let Some(after) = pg_stat_wal_snapshot(client).await? else {
        crate::ecaz_println!(
            "[fault] pg_stat_wal unavailable after lane; skipping wal counter probe"
        );
        return Ok(());
    };
    crate::ecaz_println!(
        "[fault] pg_stat_wal_records_before={} after={} bytes_before={} after={}",
        before.records,
        after.records,
        before.bytes,
        after.bytes
    );
    if after.records < before.records || after.bytes < before.bytes {
        if lane == FaultLane::Memory {
            crate::ecaz_println!(
                "[fault] pg_stat_wal_reset_after_crash_recovery=true records_before={} records_after={} bytes_before={} bytes_after={}",
                before.records,
                after.records,
                before.bytes,
                after.bytes
            );
            return Ok(());
        }
        return Err(eyre!(
            "{lane} postcondition failed: pg_stat_wal decreased from records={} bytes={} to records={} bytes={}",
            before.records,
            before.bytes,
            after.records,
            after.bytes
        ));
    }
    Ok(())
}

async fn pg_stat_io_total(client: &tokio_postgres::Client) -> Result<Option<i64>> {
    match client
        .query_one(
            "SELECT coalesce(sum(reads + writes + writebacks + extends + fsyncs), 0)::bigint
             FROM pg_stat_io",
            &[],
        )
        .await
    {
        Ok(row) => Ok(Some(row.get::<_, i64>(0))),
        Err(error)
            if error.as_db_error().is_some_and(|db| {
                let sqlstate = db.code().code();
                sqlstate == "42P01" || sqlstate == "42703"
            }) =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

async fn pg_stat_wal_snapshot(
    client: &tokio_postgres::Client,
) -> Result<Option<PgStatWalSnapshot>> {
    match client
        .query_one(
            "SELECT wal_records::bigint, wal_bytes::bigint FROM pg_stat_wal",
            &[],
        )
        .await
    {
        Ok(row) => Ok(Some(PgStatWalSnapshot {
            records: row.get::<_, i64>(0),
            bytes: row.get::<_, i64>(1),
        })),
        Err(error)
            if error.as_db_error().is_some_and(|db| {
                let sqlstate = db.code().code();
                sqlstate == "42P01" || sqlstate == "42703"
            }) =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

async fn current_wal_lsn(client: &tokio_postgres::Client) -> Result<String> {
    let row = client
        .query_one("SELECT pg_current_wal_lsn()::text", &[])
        .await?;
    Ok(row.get::<_, String>(0))
}

async fn force_pg_stat_flush(client: &tokio_postgres::Client) -> Result<()> {
    match client
        .simple_query("SELECT pg_stat_force_next_flush()")
        .await
    {
        Ok(_) => Ok(()),
        Err(error)
            if error.as_db_error().is_some_and(|db| {
                let sqlstate = db.code().code();
                sqlstate == "42883" || sqlstate == "42501"
            }) =>
        {
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

async fn pg_stat_database_temp_bytes(client: &tokio_postgres::Client) -> Result<Option<i64>> {
    match client
        .query_one(
            "SELECT temp_bytes::bigint FROM pg_stat_database WHERE datname = current_database()",
            &[],
        )
        .await
    {
        Ok(row) => Ok(Some(row.get::<_, i64>(0))),
        Err(error)
            if error.as_db_error().is_some_and(|db| {
                let sqlstate = db.code().code();
                sqlstate == "42P01" || sqlstate == "42703"
            }) =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

async fn assert_temp_bytes_non_decreasing(
    client: &tokio_postgres::Client,
    am: FaultFixture,
    mode: &str,
    before: Option<i64>,
) -> Result<()> {
    let Some(before) = before else {
        crate::ecaz_println!(
            "[fault] pg_stat_database temp_bytes unavailable; skipping temp accounting probe"
        );
        return Ok(());
    };
    let Some(after) = pg_stat_database_temp_bytes(client).await? else {
        crate::ecaz_println!(
            "[fault] pg_stat_database temp_bytes unavailable after temp spill; skipping temp accounting probe"
        );
        return Ok(());
    };
    let delta = after.saturating_sub(before);
    crate::ecaz_println!(
        "[fault] resource_temp_spill_accounting am={} mode={mode} temp_bytes_before={before} after={after} delta={delta}",
        am.as_str()
    );
    if after < before {
        return Err(eyre!(
            "resource temp spill accounting {} {mode} decreased from {before} to {after}",
            am.as_str()
        ));
    }
    Ok(())
}

fn assert_query_canceled(label: &str, result: Result<(), tokio_postgres::Error>) -> Result<()> {
    assert_sqlstate(label, result, "57014")
}

// Provider-backed I/O can surface through several PostgreSQL error classes,
// but unexpected SQLSTATEs still indicate the lane stopped proving EIO/ENOSPC.
fn assert_provider_sql_error(label: &str, result: Result<(), tokio_postgres::Error>) -> Result<()> {
    match result {
        Ok(()) => Err(eyre!("{label} probe unexpectedly succeeded")),
        Err(error) if error.as_db_error().is_some_and(provider_sqlstate_allowed) => Ok(()),
        Err(error) if error.as_db_error().is_some() => {
            let db = error.as_db_error().expect("checked above");
            Err(eyre!(
                "{label} returned unexpected provider SQLSTATE {} ({})",
                db.code().code(),
                db.message()
            ))
        }
        Err(error) => Err(error.into()),
    }
}

fn provider_sqlstate_allowed(db: &tokio_postgres::error::DbError) -> bool {
    // PostgreSQL checkpoint failures can wrap the provider's ENOSPC in XX000.
    // Keep this allowance message-narrow so unrelated internal errors fail.
    matches!(db.code().code(), "53100" | "58030")
        || (db.code().code() == "XX000" && db.message().contains("checkpoint request failed"))
        || (db.code().code() == "XX000" && db.message().contains("No space left on device"))
}

fn is_ecaz_palloc_error(error: &tokio_postgres::Error) -> bool {
    error
        .as_db_error()
        .map(|db| db.message().contains("ecaz fault injection palloc failure"))
        .unwrap_or(false)
}

fn assert_temp_file_limit_error(
    label: &str,
    result: Result<(), tokio_postgres::Error>,
) -> Result<()> {
    match result {
        Ok(()) => Err(eyre!("{label} probe unexpectedly succeeded")),
        Err(error)
            if error
                .as_db_error()
                .map(|db| db.message().contains("temporary file size exceeds"))
                .unwrap_or(false) =>
        {
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn assert_sqlstate(
    label: &str,
    result: Result<(), tokio_postgres::Error>,
    sqlstate: &str,
) -> Result<()> {
    match result {
        Ok(()) => Err(eyre!("{label} probe unexpectedly succeeded")),
        Err(error)
            if error
                .as_db_error()
                .map(|db| db.code().code() == sqlstate)
                .unwrap_or(false) =>
        {
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn print_cases(cases: &[ecaz_fault_injection::FaultCase]) {
    for case in cases {
        crate::ecaz_println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            case.id,
            case.lane,
            case.access_method.as_str(),
            case.codec.map(|codec| codec.as_str()).unwrap_or("n/a"),
            case.fault,
            case.expected
        );
    }
}

fn print_leak_probes() {
    crate::ecaz_println!("postcondition probes:");
    for sql in leak_probe_sql() {
        crate::ecaz_println!("{sql}");
    }
    for sql in optional_leak_probe_sql() {
        crate::ecaz_println!("{sql}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cgroup_roots_must_be_disjoint_and_runtime_must_avoid_evidence_trees() {
        let repo = Path::new("/repo");
        assert!(validate_cgroup_roots(
            Path::new("/repo/reviews/task-38/004/artifacts"),
            Path::new("/repo/target/fault-cgroup-runtime"),
            repo
        )
        .is_ok());

        for runtime in [
            "/repo/reviews/task-38/004/artifacts/runtime",
            "/repo/benchmarks/task-38-runtime",
            "/repo/target/fault-cgroup/evidence/runtime",
        ] {
            assert!(
                validate_cgroup_roots(
                    Path::new("/repo/target/fault-cgroup/evidence"),
                    Path::new(runtime),
                    repo
                )
                .is_err(),
                "runtime root {runtime} should be rejected"
            );
        }
    }

    #[test]
    fn socket_provider_requires_stable_peer_identity() {
        assert!(validate_provider_options(
            ProviderMode::SocketReset,
            None,
            Some("tcp:127.0.0.1:39711")
        )
        .is_ok());
        assert!(validate_provider_options(
            ProviderMode::SocketSlow,
            Some(1),
            Some("unix:/tmp/.s.PGSQL.39424")
        )
        .is_ok());

        for unstable_peer in ["unix:", "unix:relative.sock", "abstract:peer"] {
            let error =
                validate_provider_options(ProviderMode::SocketReset, None, Some(unstable_peer))
                    .expect_err("unstable peer identities are unsupported");
            assert!(error.to_string().contains("absolute named unix:/path"));
        }
    }
}
