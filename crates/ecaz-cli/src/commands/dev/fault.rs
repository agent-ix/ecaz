use clap::{Args, Subcommand, ValueEnum};
use color_eyre::eyre::{eyre, Result};
use ecaz_fault_injection::{
    all_smoke_cases, leak_probe_sql, optional_leak_probe_sql, required_smoke_cases,
    workload_accumulator_pressure_settings_sql, workload_accumulator_pressure_sql,
    workload_bulk_insert_sql, workload_insert_sql, workload_reindex_sql,
    workload_repeated_scan_sql, workload_resource_setup_sql, workload_scan_sql, workload_setup_sql,
    workload_table_sql, workload_temp_spill_sql, workload_vacuum_full_sql, workload_vacuum_sql,
    FaultAm, FaultLane, ProviderMode,
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
            run_io_probe(conn, mode, &path_match, &ams).await?;
            if provider_targets_wal(&path_match) && mode == ProviderMode::EnospcWrite {
                crate::ecaz_println!(
                    "[fault] wal_enospc_provider_restore_required=true match={path_match}"
                );
                return Ok(());
            }
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::Cancel => {
            run_cancel_probe(conn, args.rows, &ams).await?;
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::Timeout => {
            run_timeout_probe(conn, args.rows, &ams).await?;
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::LockTimeout => {
            run_lock_timeout_probe(conn, args.rows, &ams).await?;
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::Resource => {
            let provider_marker = args
                .provider_marker
                .as_deref()
                .map(|marker| read_provider_marker(Some(marker), lane))
                .transpose()?;
            run_resource_probe(conn, args.rows, &ams, provider_marker.as_deref()).await?;
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::Memory => {
            run_memory_probe(conn, args.rows, &ams).await?;
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
        FaultLane::SlowDisk => {
            read_provider_marker(args.provider_marker.as_deref(), lane)?;
            run_slow_disk_probe(conn, args.rows, &ams).await?;
            assert_postconditions(conn, lane, pg_stat_io_before, pg_stat_wal_before).await
        }
    }
}

fn selected_ams(am: Option<FaultAmArg>) -> Vec<FaultAm> {
    am.map(|am| vec![am.into()])
        .unwrap_or_else(|| FaultAm::ALL.to_vec())
}

async fn run_io_probe(
    conn: &ConnectionOptions,
    mode: ProviderMode,
    path_match: &str,
    ams: &[FaultAm],
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
                    Err(error) if error.as_db_error().is_some() => {}
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
            ProviderMode::SlowDisk => {
                return Err(eyre!(
                    "lane io requires an eio-read or enospc-write provider, got slow-disk"
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

async fn run_cancel_probe(conn: &ConnectionOptions, rows: i64, ams: &[FaultAm]) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    for &am in ams {
        run_backend_interrupt_case(
            conn,
            rows,
            am,
            "cancel",
            "SELECT pg_cancel_backend($1)",
            true,
        )
        .await?;
        run_backend_interrupt_case(
            conn,
            rows,
            am,
            "terminate",
            "SELECT pg_terminate_backend($1)",
            false,
        )
        .await?;
    }
    Ok(())
}

async fn run_backend_interrupt_case(
    conn: &ConnectionOptions,
    rows: i64,
    am: FaultAm,
    label: &str,
    interrupt_sql: &str,
    require_query_canceled_sqlstate: bool,
) -> Result<()> {
    let worker = connect_fault(conn, &format!("{label}-worker")).await?;
    let control = connect_fault(conn, &format!("{label}-control")).await?;
    let pid = worker
        .query_one("SELECT pg_backend_pid()", &[])
        .await?
        .get::<_, i32>(0);
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

async fn run_timeout_probe(conn: &ConnectionOptions, rows: i64, ams: &[FaultAm]) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    run_statement_timeout_probe(conn, rows, ams).await?;
    run_idle_in_transaction_timeout_probe(conn, ams).await
}

async fn run_statement_timeout_probe(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultAm],
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
    ams: &[FaultAm],
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
    ams: &[FaultAm],
) -> Result<()> {
    prepare_workloads(conn, rows, ams).await?;
    let holder = connect_fault(conn, "lock-holder").await?;
    let waiter = connect_fault(conn, "lock-waiter").await?;
    for &am in ams {
        let table = ecaz_fault_injection::workload_table(am);
        run_lock_timeout_case(
            &holder,
            &waiter,
            table,
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
            table,
            &format!("create_index {}", am.as_str()),
            &create_index,
        )
        .await?;
        run_lock_timeout_case(
            &holder,
            &waiter,
            table,
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
    ams: &[FaultAm],
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
    }
    Ok(())
}

async fn prepare_resource_workloads(
    conn: &ConnectionOptions,
    rows: i64,
    pressure_limit: i64,
    ams: &[FaultAm],
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
    am: FaultAm,
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
    crate::ecaz_println!(
        "[fault] resource_accumulator_pressure am={} rows={rows} limit={pressure_limit} returned={count} work_mem=64kB effective_cache_size=1MB",
        am.as_str()
    );
    let minimum = pressure_limit.min(rows).min(64);
    if count < minimum {
        return Err(eyre!(
            "resource accumulator pressure {} returned {count}, expected at least {minimum}",
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
    am: FaultAm,
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
    am: FaultAm,
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

async fn run_memory_probe(conn: &ConnectionOptions, rows: i64, ams: &[FaultAm]) -> Result<()> {
    let client = connect_fault(conn, "memory").await?;
    for &am in ams {
        run_memory_build_probe(&client, rows, am).await?;
    }
    prepare_workloads(conn, rows, ams).await?;
    for &am in ams {
        let sweep_limit = memory_scan_sweep_limit(am);
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
        run_memory_workload_palloc_sweep(&client, am, "insert", &workload_insert_sql(am)).await?;
        run_memory_workload_palloc_sweep(&client, am, "vacuum", &workload_vacuum_sql(am)).await?;
    }
    drop(client);
    run_memory_oom_kill_probe(conn, rows, ams).await?;
    Ok(())
}

async fn run_memory_build_probe(
    client: &tokio_postgres::Client,
    rows: i64,
    am: FaultAm,
) -> Result<()> {
    let build_sql = ecaz_fault_injection::workload_create_index_sql(am, rows);
    let mut reached_success = false;
    for nth in 1..=memory_major_workload_sweep_limit() {
        client.batch_execute(&workload_table_sql(am, rows)).await?;
        if run_memory_expected_palloc_probe(client, am, "build", nth, &build_sql).await? {
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
    am: FaultAm,
    lane: &str,
    sql: &str,
) -> Result<()> {
    let mut reached_success = false;
    for nth in 1..=memory_major_workload_sweep_limit() {
        if run_memory_expected_palloc_probe(client, am, lane, nth, sql).await? {
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
    am: FaultAm,
    lane: &str,
    nth: i32,
    sql: &str,
) -> Result<bool> {
    client
        .batch_execute(&format!(
            "SELECT ecaz_fault_reset_palloc_counter(); SET ecaz.fault_palloc_nth = {nth};"
        ))
        .await?;
    let result = client.batch_execute(sql).await;
    client
        .batch_execute("SET ecaz.fault_palloc_nth = -1; SELECT ecaz_fault_reset_palloc_counter();")
        .await?;
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
    client.simple_query("SELECT 1").await.map_err(|error| {
        eyre!(
            "memory palloc {} {lane} nth {nth} did not leave the backend usable: {error}",
            am.as_str()
        )
    })?;
    crate::ecaz_println!(
        "[fault] memory_palloc_sweep_fault am={} lane={lane} nth={nth}",
        am.as_str()
    );
    Ok(true)
}

fn memory_major_workload_sweep_limit() -> i32 {
    8
}

fn memory_scan_sweep_limit(am: FaultAm) -> i32 {
    match am {
        FaultAm::Hnsw => 4,
        FaultAm::Ivf => 4,
        FaultAm::DiskAnn => 1,
        FaultAm::Spire => 3,
    }
}

async fn run_memory_oom_kill_probe(
    conn: &ConnectionOptions,
    rows: i64,
    ams: &[FaultAm],
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
    am: FaultAm,
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
    am: FaultAm,
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

    tokio::time::sleep(std::time::Duration::from_millis(oom_kill_delay_ms())).await;
    crate::ecaz_println!("[fault] {label} sigkill_pid={pid}");
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
    am: FaultAm,
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
    matches!(db.code().code(), "53100" | "58030")
        || (db.code().code() == "XX000" && db.message().contains("checkpoint request failed"))
}

// pgrx reports the injected fault as an internal ERROR, so match the extension
// diagnostic instead of accepting all XX000 failures.
fn assert_ecaz_palloc_error(label: &str, result: Result<(), tokio_postgres::Error>) -> Result<()> {
    match result {
        Ok(()) => Err(eyre!("{label} probe unexpectedly succeeded")),
        Err(error) if is_ecaz_palloc_error(&error) => Ok(()),
        Err(error) => Err(error.into()),
    }
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
    for sql in optional_leak_probe_sql() {
        crate::ecaz_println!("{sql}");
    }
}
