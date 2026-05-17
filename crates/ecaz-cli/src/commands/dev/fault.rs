use clap::{Args, Subcommand, ValueEnum};
use color_eyre::eyre::{eyre, Result};
use ecaz_fault_injection::{
    all_smoke_cases, leak_probe_sql, required_smoke_cases, workload_insert_sql, workload_scan_sql,
    workload_setup_sql, workload_vacuum_sql, FaultAm, FaultLane,
};

use crate::psql::{self, ConnectionOptions};

#[derive(Subcommand, Debug)]
pub enum FaultCommand {
    /// Print the required PG-level fault-injection matrix.
    Plan(PlanArgs),
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

async fn run_smoke(conn: &ConnectionOptions, args: SmokeArgs) -> Result<()> {
    let lane = FaultLane::from(args.lane);
    let cases = required_smoke_cases(lane);
    print_cases(&cases);
    print_leak_probes();

    if args.dry_run {
        return Ok(());
    }

    match lane {
        FaultLane::Cancel => {
            run_cancel_probe(conn, args.rows).await?;
            assert_postconditions(conn, lane).await
        }
        FaultLane::Timeout => {
            run_statement_timeout_probe(conn, args.rows).await?;
            assert_postconditions(conn, lane).await
        }
        FaultLane::LockTimeout => {
            run_lock_timeout_probe(conn, args.rows).await?;
            assert_postconditions(conn, lane).await
        }
        FaultLane::Resource => {
            run_resource_probe(conn, args.rows).await?;
            assert_postconditions(conn, lane).await
        }
        unsupported => Err(eyre!(
            "lane {unsupported} requires an injection provider; rerun with --dry-run or install the Task 38 provider"
        )),
    }
}

async fn run_cancel_probe(conn: &ConnectionOptions, rows: i64) -> Result<()> {
    prepare_workloads(conn, rows).await?;
    for am in FaultAm::ALL {
        let worker = connect_fault(conn, "cancel-worker").await?;
        let control = connect_fault(conn, "cancel-control").await?;
        let pid = worker
            .query_one("SELECT pg_backend_pid()", &[])
            .await?
            .get::<_, i32>(0);
        let sql = format!("{}; SELECT pg_sleep(5);", workload_scan_sql(am));
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

async fn run_statement_timeout_probe(conn: &ConnectionOptions, rows: i64) -> Result<()> {
    prepare_workloads(conn, rows).await?;
    let client = connect_fault(conn, "timeout").await?;
    for am in FaultAm::ALL {
        let timeout = client
            .batch_execute(&format!(
                "SET statement_timeout = '1ms'; {}; SELECT pg_sleep(0.05);",
                workload_scan_sql(am)
            ))
            .await;
        assert_expected_error(&format!("statement_timeout {}", am.as_str()), timeout)?;
        client.batch_execute("RESET statement_timeout;").await?;
    }
    Ok(())
}

async fn run_lock_timeout_probe(conn: &ConnectionOptions, rows: i64) -> Result<()> {
    prepare_workloads(conn, rows).await?;
    let holder = connect_fault(conn, "lock-holder").await?;
    let waiter = connect_fault(conn, "lock-waiter").await?;
    for am in FaultAm::ALL {
        let table = ecaz_fault_injection::workload_table(am);
        holder
            .batch_execute(&format!(
                "BEGIN; LOCK TABLE {table} IN ACCESS EXCLUSIVE MODE;"
            ))
            .await?;
        let timeout = waiter
            .batch_execute(&format!(
                "SET lock_timeout = '10ms';
                 LOCK TABLE {table} IN ACCESS EXCLUSIVE MODE;"
            ))
            .await;
        holder.batch_execute("ROLLBACK;").await?;
        assert_sqlstate(&format!("lock_timeout {}", am.as_str()), timeout, "55P03")?;
    }
    Ok(())
}

async fn run_resource_probe(conn: &ConnectionOptions, rows: i64) -> Result<()> {
    prepare_workloads(conn, rows).await?;
    let client = connect_fault(conn, "resource").await?;
    for am in FaultAm::ALL {
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

async fn prepare_workloads(conn: &ConnectionOptions, rows: i64) -> Result<()> {
    if rows <= 0 {
        return Err(eyre!("--rows must be >= 1"));
    }
    let client = connect_fault(conn, "prepare").await?;
    client
        .batch_execute("CREATE EXTENSION IF NOT EXISTS ecaz;")
        .await?;
    for am in FaultAm::ALL {
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

fn assert_expected_error(label: &str, result: Result<(), tokio_postgres::Error>) -> Result<()> {
    assert_sqlstate(label, result, "57014")
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
        Err(error) if error.to_string().contains("statement timeout") => Ok(()),
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
