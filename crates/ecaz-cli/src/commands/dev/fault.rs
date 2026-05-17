use clap::{Args, Subcommand, ValueEnum};
use color_eyre::eyre::{eyre, Result};
use ecaz_fault_injection::{
    all_smoke_cases, leak_probe_sql, required_smoke_cases, workload_insert_sql,
    workload_reindex_sql, workload_repeated_scan_sql, workload_scan_sql, workload_setup_sql,
    workload_vacuum_sql, FaultAm, FaultLane, ProviderMode,
};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use super::support::{
    default_pgrx_port, find_pgrx_install, resolve_pgrx_home, run_status, DEFAULT_PG_MAJOR,
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
}

#[derive(Args, Debug)]
pub struct PlanArgs {
    /// Restrict output to one lane.
    #[arg(long, value_enum)]
    lane: Option<FaultLaneArg>,
}

#[derive(Args, Debug)]
pub struct ProviderEnvArgs {
    /// Provider fault mode to configure.
    #[arg(long, value_enum)]
    mode: ProviderModeArg,
    /// Substring that must appear in the target path, for example "base/".
    #[arg(long, default_value = "base/")]
    path_match: String,
    /// Start injecting on the Nth matching provider operation.
    #[arg(long, default_value_t = 1)]
    after: u64,
    /// Latency in milliseconds for slow-disk mode.
    #[arg(long)]
    latency_ms: Option<u64>,
    /// Optional marker file written by every process that loads the provider.
    #[arg(long)]
    marker: Option<String>,
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
    /// Start injecting on the Nth matching provider operation.
    #[arg(long, default_value_t = 1)]
    after: u64,
    /// Latency in milliseconds for slow-disk mode.
    #[arg(long)]
    latency_ms: Option<u64>,
    /// Marker file written by every process that loads the provider.
    #[arg(long)]
    marker: Option<PathBuf>,
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
    /// Marker file proving the target postmaster loaded the fault provider.
    #[arg(long)]
    provider_marker: Option<String>,
    /// Reuse already-created AM fixtures instead of preparing them in this process.
    #[arg(long)]
    assume_prepared: bool,
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
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FaultAmArg {
    Hnsw,
    Ivf,
    Diskann,
    Spire,
}

impl From<ProviderModeArg> for ProviderMode {
    fn from(value: ProviderModeArg) -> Self {
        match value {
            ProviderModeArg::EioRead => ProviderMode::EioRead,
            ProviderModeArg::EnospcWrite => ProviderMode::EnospcWrite,
            ProviderModeArg::SlowDisk => ProviderMode::SlowDisk,
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
                let ams = selected_ams(args.am);
                prepare_workloads(conn, args.rows, &ams).await
            }
            FaultCommand::Smoke(args) => run_smoke(conn, args).await,
        }
    }
}

fn run_plan(args: PlanArgs) -> Result<()> {
    let cases = args
        .lane
        .map(|lane| required_smoke_cases(lane.into()))
        .unwrap_or_else(all_smoke_cases);
    print_cases(&cases);
    print_leak_probes();
    Ok(())
}

fn run_provider_env(args: ProviderEnvArgs) -> Result<()> {
    let mode = ProviderMode::from(args.mode);
    if mode == ProviderMode::SlowDisk && args.latency_ms.unwrap_or(0) == 0 {
        return Err(eyre!("--latency-ms must be >= 1 for slow-disk mode"));
    }
    let env = ecaz_fault_injection::provider_environment(
        mode,
        &args.path_match,
        args.after,
        args.latency_ms,
        args.marker.as_deref(),
    );
    for (key, value) in env {
        crate::ecaz_println!("{key}={value}");
    }
    Ok(())
}

async fn run_provider_restart(args: ProviderRestartArgs) -> Result<()> {
    let mode = ProviderMode::from(args.mode);
    let latency_ms = match (mode, args.latency_ms) {
        (ProviderMode::SlowDisk, None) => Some(1),
        (_, value) => value,
    };
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let install = find_pgrx_install(args.pg, &pgrx_home)?;
    let marker = args.marker.unwrap_or_else(|| {
        std::env::temp_dir().join(format!("ecaz-fault-provider-{}-pg{}.marker", mode, args.pg))
    });
    std::fs::write(&marker, "")?;
    let marker_string = marker.to_string_lossy().to_string();
    let env = ecaz_fault_injection::provider_environment(
        mode,
        &args.path_match,
        args.after,
        latency_ms,
        Some(&marker_string),
    );
    restart_pgrx_postmaster(
        &install.bin_dir.join("pg_ctl"),
        &pgrx_home,
        args.pg,
        args.port.unwrap_or_else(|| default_pgrx_port(args.pg)),
        &env,
    )
    .await?;
    crate::ecaz_println!("[fault] provider_marker={}", marker.display());
    Ok(())
}

async fn run_provider_restore(args: ProviderRestoreArgs) -> Result<()> {
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let install = find_pgrx_install(args.pg, &pgrx_home)?;
    restart_pgrx_postmaster(
        &install.bin_dir.join("pg_ctl"),
        &pgrx_home,
        args.pg,
        args.port.unwrap_or_else(|| default_pgrx_port(args.pg)),
        &[],
    )
    .await
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
    let ams = selected_ams(args.am);
    let cases = required_smoke_cases(lane)
        .into_iter()
        .filter(|case| ams.contains(&case.access_method))
        .collect::<Vec<_>>();
    print_cases(&cases);
    print_leak_probes();

    if args.dry_run {
        return Ok(());
    }

    match lane {
        FaultLane::Io => {
            let marker = read_provider_marker(args.provider_marker.as_deref(), lane)?;
            let mode = provider_mode_from_marker(&marker)?;
            if !args.assume_prepared {
                return Err(eyre!(
                    "lane {lane} must run against prebuilt fixtures; run `ecaz dev fault prepare --rows {}` before starting the provider, then rerun with --assume-prepared",
                    args.rows
                ));
            }
            run_io_probe(conn, mode, &ams).await?;
            assert_postconditions(conn, lane).await
        }
        FaultLane::Cancel => {
            run_cancel_probe(conn, args.rows, &ams).await?;
            assert_postconditions(conn, lane).await
        }
        FaultLane::Timeout => {
            run_statement_timeout_probe(conn, args.rows, &ams).await?;
            assert_postconditions(conn, lane).await
        }
        FaultLane::LockTimeout => {
            run_lock_timeout_probe(conn, args.rows, &ams).await?;
            assert_postconditions(conn, lane).await
        }
        FaultLane::Resource => {
            run_resource_probe(conn, args.rows, &ams).await?;
            assert_postconditions(conn, lane).await
        }
        FaultLane::Memory => {
            run_memory_probe(conn, args.rows, &ams).await?;
            assert_postconditions(conn, lane).await
        }
        FaultLane::SlowDisk => {
            read_provider_marker(args.provider_marker.as_deref(), lane)?;
            run_slow_disk_probe(conn, args.rows, &ams).await?;
            assert_postconditions(conn, lane).await
        }
    }
}

fn selected_ams(am: Option<FaultAmArg>) -> Vec<FaultAm> {
    am.map(|am| vec![am.into()])
        .unwrap_or_else(|| FaultAm::ALL.to_vec())
}

async fn run_io_probe(conn: &ConnectionOptions, mode: ProviderMode, ams: &[FaultAm]) -> Result<()> {
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
                    Err(error) if error.as_db_error().is_some() => {}
                    Err(error) => return Err(error.into()),
                    Ok(()) => {
                        assert_provider_sql_error(
                            &label,
                            client.batch_execute("CHECKPOINT").await,
                        )?;
                    }
                }
            }
            ProviderMode::SlowDisk => {
                return Err(eyre!(
                    "lane io requires an eio-read or enospc-write provider, got slow-disk"
                ))
            }
        };
        client
            .simple_query("SELECT 1")
            .await
            .map_err(|error| eyre!("{label} did not leave the backend usable: {error}"))?;
    }
    Ok(())
}

async fn run_cancel_probe(conn: &ConnectionOptions, rows: i64, ams: &[FaultAm]) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    for &am in ams {
        let worker = connect_fault(conn, "cancel-worker").await?;
        let control = connect_fault(conn, "cancel-control").await?;
        let pid = worker
            .query_one("SELECT pg_backend_pid()", &[])
            .await?
            .get::<_, i32>(0);
        let sql = workload_repeated_scan_sql(am, cancel_probe_iterations(rows));
        let worker_task = tokio::spawn(async move {
            worker
                .batch_execute(&sql)
                .await
                .map_err(color_eyre::Report::from)
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        control
            .execute("SELECT pg_cancel_backend($1)", &[&pid])
            .await?;

        match worker_task.await? {
            Ok(()) => {
                return Err(eyre!(
                    "cancel probe unexpectedly succeeded for {}",
                    am.as_str()
                ))
            }
            Err(error)
                if error
                    .downcast_ref::<tokio_postgres::Error>()
                    .and_then(tokio_postgres::Error::as_db_error)
                    .map(|db| db.code().code() == "57014")
                    .unwrap_or(false) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

async fn run_statement_timeout_probe(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultAm],
) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    let client = connect_fault(conn, "timeout").await?;
    for &am in ams {
        let timeout = client
            .batch_execute(&format!(
                "SET statement_timeout = '5ms'; {}",
                workload_repeated_scan_sql(am, timeout_probe_iterations(rows))
            ))
            .await;
        assert_query_canceled(&format!("statement_timeout {}", am.as_str()), timeout)?;
        client.batch_execute("RESET statement_timeout;").await?;
    }
    Ok(())
}

async fn run_lock_timeout_probe(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultAm],
) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    let holder = connect_fault(conn, "lock-holder").await?;
    let waiter = connect_fault(conn, "lock-waiter").await?;
    for &am in ams {
        let table = ecaz_fault_injection::workload_table(am);
        holder
            .batch_execute(&format!(
                "BEGIN; LOCK TABLE {table} IN ACCESS EXCLUSIVE MODE;"
            ))
            .await?;
        waiter.batch_execute("SET lock_timeout = '10ms';").await?;
        let timeout = waiter.batch_execute(&workload_reindex_sql(am)).await;
        let reset = waiter.batch_execute("RESET lock_timeout;").await;
        let rollback = holder.batch_execute("ROLLBACK;").await;
        reset?;
        rollback?;
        assert_sqlstate(&format!("lock_timeout {}", am.as_str()), timeout, "55P03")?;
    }
    Ok(())
}

fn cancel_probe_iterations(rows: i64) -> i64 {
    rows.saturating_mul(2_000).clamp(100_000, 1_000_000)
}

fn timeout_probe_iterations(rows: i64) -> i64 {
    rows.saturating_mul(2_000).clamp(100_000, 1_000_000)
}

async fn run_resource_probe(conn: &ConnectionOptions, rows: i64, ams: &[FaultAm]) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    let client = connect_fault(conn, "resource").await?;
    for &am in ams {
        client
            .batch_execute("SET work_mem = '64kB'; SET maintenance_work_mem = '1MB';")
            .await?;
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
    }
    Ok(())
}

async fn run_memory_probe(conn: &ConnectionOptions, rows: i64, ams: &[FaultAm]) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    let client = connect_fault(conn, "memory").await?;
    for &am in ams {
        let sweep_limit = memory_probe_sweep_limit(am);
        for nth in 1..=sweep_limit {
            client
                .batch_execute(&format!(
                    "SELECT ecaz_fault_reset_palloc_counter(); SET ecaz.fault_palloc_nth = {nth};"
                ))
                .await?;
            let result = client
                .batch_execute(&workload_repeated_scan_sql(am, i64::from(sweep_limit)))
                .await;
            client
                .batch_execute(
                    "SET ecaz.fault_palloc_nth = -1; SELECT ecaz_fault_reset_palloc_counter();",
                )
                .await?;
            assert_ecaz_palloc_error(&format!("memory palloc {} nth {nth}", am.as_str()), result)?;
            client.simple_query("SELECT 1").await.map_err(|error| {
                eyre!(
                    "memory palloc {} nth {nth} did not leave the backend usable: {error}",
                    am.as_str()
                )
            })?;
        }
    }
    Ok(())
}

fn memory_probe_sweep_limit(am: FaultAm) -> i32 {
    match am {
        FaultAm::Hnsw => 4,
        FaultAm::Ivf => 4,
        FaultAm::DiskAnn => 1,
        FaultAm::Spire => 3,
    }
}

async fn run_slow_disk_probe(conn: &ConnectionOptions, rows: i64, ams: &[FaultAm]) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    let client = connect_fault(conn, "slow-disk").await?;
    for &am in ams {
        client.batch_execute(&workload_scan_sql(am)).await?;
        client.batch_execute(&workload_insert_sql(am)).await?;
        client.batch_execute(&workload_vacuum_sql(am)).await?;
    }
    Ok(())
}

async fn prepare_workloads(conn: &ConnectionOptions, rows: i64, ams: &[FaultAm]) -> Result<()> {
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

async fn print_workload_paths(client: &tokio_postgres::Client, am: FaultAm) -> Result<()> {
    let table = ecaz_fault_injection::workload_table(am);
    let index = ecaz_fault_injection::workload_index(am);
    let table_path = relation_filepath(client, table).await?;
    let index_path = relation_filepath(client, index).await?;
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
    } else {
        Err(eyre!(
            "provider marker did not include a supported mode line"
        ))
    }
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

async fn assert_postconditions(conn: &ConnectionOptions, lane: FaultLane) -> Result<()> {
    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    let client = connect_fault(conn, "postcondition").await?;
    for &sql in leak_probe_sql() {
        let row = client.query_one(sql, &[]).await?;
        let count = row.get::<_, i64>(0);
        if count != 0 {
            return Err(eyre!("{lane} postcondition failed: {sql} returned {count}"));
        }
    }
    Ok(())
}

fn assert_query_canceled(label: &str, result: Result<(), tokio_postgres::Error>) -> Result<()> {
    assert_sqlstate(label, result, "57014")
}

fn assert_provider_sql_error(label: &str, result: Result<(), tokio_postgres::Error>) -> Result<()> {
    match result {
        Ok(()) => Err(eyre!("{label} probe unexpectedly succeeded")),
        Err(error) if error.as_db_error().is_some() => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn assert_ecaz_palloc_error(label: &str, result: Result<(), tokio_postgres::Error>) -> Result<()> {
    match result {
        Ok(()) => Err(eyre!("{label} probe unexpectedly succeeded")),
        Err(error)
            if error
                .as_db_error()
                .map(|db| db.message().contains("ecaz fault injection palloc failure"))
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
            "{}\t{}\t{}\t{}\t{}",
            case.id,
            case.lane,
            case.access_method.as_str(),
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
}
