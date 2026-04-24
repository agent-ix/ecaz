use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, eyre, Context, Result};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use crate::psql;

use super::support::{
    find_pgrx_install, repo_root, resolve_pgrx_home, run_status, PG18_PARALLEL_SCAN_DEFAULT_PORT,
    PG18_PRELOAD_DEFAULT_PORT,
};

#[derive(Subcommand, Debug)]
pub enum TestCommand {
    /// Run `cargo pgrx test` through the CLI-owned test surface.
    Pgrx(PgrxTestArgs),
    /// Start a repo-local PG18 cluster with preload enabled and validate shared pgstat visibility.
    Pg18PreloadPgstat(Pg18PreloadPgstatArgs),
    /// Start a repo-local PG18 cluster and diagnose planner-visible parallel scan readiness.
    Pg18ParallelScan(Pg18ParallelScanArgs),
}

impl TestCommand {
    pub async fn run(self, _database: &str) -> Result<()> {
        match self {
            TestCommand::Pgrx(args) => run_pgrx(args).await,
            TestCommand::Pg18PreloadPgstat(args) => run_pg18_preload_pgstat(args).await,
            TestCommand::Pg18ParallelScan(args) => run_pg18_parallel_scan(args).await,
        }
    }
}

#[derive(Args, Debug)]
pub struct PgrxTestArgs {
    /// PostgreSQL major version to run.
    #[arg(long, default_value_t = 18)]
    pg: u16,

    /// Environment override passed to spawned test commands, as KEY=VALUE.
    #[arg(long = "env", value_name = "KEY=VALUE")]
    env: Vec<String>,

    /// Extra arguments passed through to `cargo pgrx test`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    cargo_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct Pg18PreloadPgstatArgs {
    /// Starting port for the repo-local cluster. The command will try this port and the next 9.
    #[arg(long, default_value_t = PG18_PRELOAD_DEFAULT_PORT)]
    port: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Environment override passed to the PG18 validation server, as KEY=VALUE.
    #[arg(long = "env", value_name = "KEY=VALUE")]
    env: Vec<String>,
}

#[derive(Args, Debug)]
pub struct Pg18ParallelScanArgs {
    /// Starting port for the repo-local cluster. The command will try this port and the next 9.
    #[arg(long, default_value_t = PG18_PARALLEL_SCAN_DEFAULT_PORT)]
    port: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Environment override passed to the PG18 validation server, as KEY=VALUE.
    #[arg(long = "env", value_name = "KEY=VALUE")]
    env: Vec<String>,

    /// Planned parallel workers per gather.
    #[arg(long, default_value_t = 4)]
    workers: u16,

    /// Fixture row count.
    #[arg(long, default_value_t = 512)]
    rows: i32,

    /// Query LIMIT for serial/parallel comparison.
    #[arg(long, default_value_t = 16)]
    limit: i64,

    /// ef_search override. Defaults to a full-traversal budget for the fixture size.
    #[arg(long)]
    ef_search: Option<i32>,

    /// Require PostgreSQL to choose and launch a real Parallel Index Scan.
    #[arg(long)]
    expect_parallel: bool,

    /// Capture only non-executing planner output for the ordered candidate path.
    #[arg(long)]
    planner_only: bool,

    /// Disable leader participation when planning and executing parallel paths.
    #[arg(long)]
    disable_parallel_leader_participation: bool,

    /// Print planner/catalog diagnostics for PG18 parallel index path activation.
    #[arg(long)]
    diagnose_planner: bool,

    /// Write the emitted PG18 parallel-scan diagnostic output to this path.
    #[arg(long)]
    log_output: Option<PathBuf>,
}

async fn run_pgrx(args: PgrxTestArgs) -> Result<()> {
    let repo_root = repo_root()?;
    let env_overrides = parse_env_overrides(&args.env)?;
    let mut command = Command::new("cargo");
    command
        .arg("pgrx")
        .arg("test")
        .arg(format!("pg{}", args.pg))
        .args(args.cargo_args)
        .current_dir(repo_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    apply_env_overrides(&mut command, &env_overrides);
    run_status(command).await
}

async fn run_pg18_preload_pgstat(args: Pg18PreloadPgstatArgs) -> Result<()> {
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let env_overrides = parse_env_overrides(&args.env)?;
    let cluster =
        start_pg18_validation_cluster(&pgrx_home, "pg18-preload-pgstat", args.port, &env_overrides)
            .await?;
    let observer = psql::connect_with(&cluster.base).await?;
    let actor = psql::connect_with(&cluster.base).await?;

    observer
        .batch_execute(
            "
DROP TABLE IF EXISTS pg18_preload_pgstat_fixture CASCADE;
DROP EXTENSION IF EXISTS ecaz CASCADE;
CREATE EXTENSION ecaz;
CREATE TABLE pg18_preload_pgstat_fixture (id bigint primary key, embedding ecvector);
INSERT INTO pg18_preload_pgstat_fixture VALUES
  (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
  (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
  (3, encode_to_ecvector(ARRAY[0.5, 0.5, -0.5, 1.0], 4, 42));
CREATE INDEX pg18_preload_pgstat_fixture_idx ON pg18_preload_pgstat_fixture USING ec_hnsw (embedding ecvector_ip_ops);
",
        )
        .await?;

    let planner_row = observer
        .query_one(
            "
SELECT pg18_diagnostics_surface_ready, next_pg18_blocker
FROM ec_hnsw_planner_integration_snapshot('pg18_preload_pgstat_fixture_idx'::regclass)
",
            &[],
        )
        .await?;
    let diagnostics_ready: bool = planner_row.get(0);
    let next_pg18_blocker: String = planner_row.get(1);
    if !diagnostics_ready {
        bail!("planner snapshot should report PG18 diagnostics surface ready under preload");
    }
    if next_pg18_blocker != "no merged PG18 blocker remains on main" {
        bail!("unexpected PG18 blocker under preload: {next_pg18_blocker}");
    }

    let baseline = observer
        .query_one(
            "SELECT total_scans_started, total_distance_calcs FROM ecaz_stats()",
            &[],
        )
        .await?;
    let baseline_scans: i64 = baseline.get(0);
    let baseline_distance: i64 = baseline.get(1);

    actor
        .batch_execute(
            "
SET enable_seqscan = off;
SELECT id
FROM pg18_preload_pgstat_fixture
ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
LIMIT 1
",
        )
        .await?;

    let shared = observer
        .query_one(
            "SELECT total_scans_started, total_distance_calcs FROM ecaz_stats()",
            &[],
        )
        .await?;
    let shared_scans: i64 = shared.get(0);
    let shared_distance: i64 = shared.get(1);
    if shared_scans <= baseline_scans {
        bail!("observer backend should see shared scan count increase");
    }
    if shared_distance <= baseline_distance {
        bail!("observer backend should see shared distance calculations increase");
    }

    println!("[pg18-preload] install={}", cluster.install_version_label);
    println!(
        "[pg18-preload] shared_preload_libraries={}",
        cluster.preload_setting
    );
    println!(
        "[pg18-preload] env={}",
        format_env_override_keys(&env_overrides)
    );
    println!(
        "[pg18-preload] baseline_scans={baseline_scans} baseline_distance_calcs={baseline_distance}"
    );
    println!("[pg18-preload] shared_scans={shared_scans} shared_distance_calcs={shared_distance}");
    println!("[pg18-preload] preload-aware PG18 shared pgstat validation passed");
    Ok(())
}

async fn run_pg18_parallel_scan(args: Pg18ParallelScanArgs) -> Result<()> {
    if args.workers == 0 {
        bail!("--workers must be at least 1");
    }
    if args.rows < 1 {
        bail!("--rows must be at least 1");
    }
    if args.limit < 1 {
        bail!("--limit must be at least 1");
    }
    if args.limit > i64::from(args.rows) {
        bail!("--limit must be less than or equal to --rows");
    }

    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let env_overrides = parse_env_overrides(&args.env)?;
    let cluster =
        start_pg18_validation_cluster(&pgrx_home, "pg18-parallel-scan", args.port, &env_overrides)
            .await?;
    let mut client = psql::connect_with(&cluster.base).await?;
    let ef_search = args.ef_search.unwrap_or_else(|| {
        i32::from(args.workers)
            .saturating_add(1)
            .saturating_mul(args.rows)
            .saturating_mul(2)
            .max(args.rows)
            .min(1000)
    });
    if !(1..=1000).contains(&ef_search) {
        bail!("--ef-search must be between 1 and 1000");
    }

    client
        .batch_execute(
            format!(
                "
DROP TABLE IF EXISTS pg18_parallel_scan_fixture CASCADE;
DROP EXTENSION IF EXISTS ecaz CASCADE;
CREATE EXTENSION ecaz;
CREATE TABLE pg18_parallel_scan_fixture (id bigint primary key, embedding ecvector);
INSERT INTO pg18_parallel_scan_fixture
SELECT g::bigint,
       ARRAY[
         ((g % 97)::real / 97.0),
         (((g * 3) % 89)::real / 89.0),
         (((g * 7) % 83)::real / 83.0),
         (-(((g * 11) % 79)::real) / 79.0)
       ]::real[]::ecvector
FROM generate_series(1, {rows}) AS g;
ALTER TABLE pg18_parallel_scan_fixture SET (parallel_workers = {workers});
CREATE INDEX pg18_parallel_scan_fixture_idx
ON pg18_parallel_scan_fixture USING ec_hnsw (embedding ecvector_ip_ops)
WITH (m = 8, ef_construction = 80);
ANALYZE pg18_parallel_scan_fixture;
",
                rows = args.rows,
                workers = args.workers,
            )
            .as_str(),
        )
        .await?;

    client
        .batch_execute(
            format!(
                "
SET enable_seqscan = off;
SET enable_bitmapscan = off;
SET min_parallel_index_scan_size = 0;
SET min_parallel_table_scan_size = 0;
SET parallel_setup_cost = 0;
SET parallel_tuple_cost = 0;
SET max_parallel_workers = {workers};
SET max_parallel_workers_per_gather = 0;
{parallel_leader_participation_setting}
SET ec_hnsw.ef_search = {ef_search};
",
                workers = args.workers,
                ef_search = ef_search,
                parallel_leader_participation_setting = parallel_leader_participation_setting(
                    args.disable_parallel_leader_participation
                ),
            )
            .as_str(),
        )
        .await?;
    let serial_ids = if args.planner_only {
        None
    } else {
        Some(pg18_parallel_fixture_ids(&client, args.limit).await?)
    };

    client
        .batch_execute(format!("SET max_parallel_workers_per_gather = {};", args.workers).as_str())
        .await?;
    let plan = if args.planner_only {
        pg18_parallel_fixture_explain_json(&mut client, args.limit, args.workers).await?
    } else {
        pg18_parallel_fixture_explain_analyze(&client, args.limit).await?
    };
    let has_parallel_index_scan = if args.planner_only {
        pg18_plan_json_has_parallel_index_scan(&plan)
    } else {
        plan.contains("Parallel Index Scan")
    };
    let has_launched_workers = !args.planner_only && plan.contains("Workers Launched:");
    let candidate_ids = if args.planner_only {
        None
    } else {
        Some(pg18_parallel_fixture_ids(&client, args.limit).await?)
    };
    let parallel_seqscan_plan = if args.planner_only {
        pg18_parallel_fixture_parallel_seqscan_json(&mut client).await?
    } else {
        pg18_parallel_fixture_parallel_seqscan_plan(&mut client).await?
    };
    let has_parallel_seqscan = parallel_seqscan_plan.contains("Parallel Seq Scan")
        || parallel_seqscan_plan.contains("\"Node Type\": \"Gather\"");
    let planner_diagnostics = if args.diagnose_planner || args.planner_only {
        Some(
            pg18_parallel_fixture_planner_diagnostics(&mut client, args.limit, args.workers)
                .await?,
        )
    } else {
        None
    };
    let diagnostics_for_error = planner_diagnostics
        .as_deref()
        .map(|diagnostics| format!("\n\nplanner diagnostics:\n{diagnostics}"))
        .unwrap_or_default();
    if args.expect_parallel && !has_parallel_index_scan {
        bail!(
            "expected a Parallel Index Scan plan, got:\n{plan}\n\nparallel seqscan control plan:\n{parallel_seqscan_plan}{diagnostics_for_error}"
        );
    }
    if args.expect_parallel && !args.planner_only && !has_launched_workers {
        bail!(
            "expected EXPLAIN ANALYZE to launch parallel workers, got:\n{plan}\n\nparallel seqscan control plan:\n{parallel_seqscan_plan}{diagnostics_for_error}"
        );
    }

    if let (Some(candidate_ids), Some(serial_ids)) = (&candidate_ids, &serial_ids) {
        if candidate_ids != serial_ids {
            bail!(
                "parallel-enabled ordered IDs diverged from serial\nserial={serial_ids:?}\ncandidate={candidate_ids:?}\nplan:\n{plan}\n\nparallel seqscan control plan:\n{parallel_seqscan_plan}"
            );
        }
    }

    let planner_diagnostics_output = planner_diagnostics
        .as_deref()
        .map(|diagnostics| format!("[pg18-parallel] planner diagnostics:\n{diagnostics}\n"))
        .unwrap_or_default();
    let final_status = if args.planner_only && has_parallel_index_scan {
        "[pg18-parallel] planner-only Parallel Index Scan plan validation passed"
    } else if has_parallel_index_scan && has_launched_workers {
        "[pg18-parallel] planner-visible Parallel Index Scan validation passed"
    } else if has_parallel_seqscan {
        "[pg18-parallel] PostgreSQL can launch workers for the fixture, but did not choose a real Parallel Index Scan; use --expect-parallel once AM planner path activation is ready"
    } else {
        "[pg18-parallel] PostgreSQL did not choose a real Parallel Index Scan or the parallel seqscan control path; inspect worker availability before using --expect-parallel"
    };
    let output = format!(
        "[pg18-parallel] install={}\n\
         [pg18-parallel] shared_preload_libraries={}\n\
         [pg18-parallel] env={}\n\
         [pg18-parallel] rows={} workers={} limit={} ef_search={}\n\
         [pg18-parallel] planner_only={} disable_parallel_leader_participation={}\n\
         [pg18-parallel] plan:\n{plan}\n\
         [pg18-parallel] parallel seqscan control plan:\n{parallel_seqscan_plan}\n\
         {planner_diagnostics_output}\
         [pg18-parallel] serial_ids={}\n\
         [pg18-parallel] candidate_ids={}\n\
         {final_status}\n",
        cluster.install_version_label,
        cluster.preload_setting,
        format_env_override_keys(&env_overrides),
        args.rows,
        args.workers,
        args.limit,
        ef_search,
        args.planner_only,
        args.disable_parallel_leader_participation,
        format_optional_ids(serial_ids.as_deref()),
        format_optional_ids(candidate_ids.as_deref()),
    );
    if let Some(log_output) = &args.log_output {
        if let Some(parent) = log_output.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .wrap_err_with(|| format!("creating {}", parent.display()))?;
            }
        }
        fs::write(log_output, &output)
            .wrap_err_with(|| format!("writing {}", log_output.display()))?;
    }
    print!("{output}");
    Ok(())
}

fn parse_env_overrides(raw: &[String]) -> Result<Vec<(String, String)>> {
    raw.iter()
        .map(|assignment| parse_env_override(assignment))
        .collect()
}

fn parse_env_override(raw: &str) -> Result<(String, String)> {
    let Some((key, value)) = raw.split_once('=') else {
        bail!("--env must use KEY=VALUE, got {raw:?}");
    };
    if key.is_empty() {
        bail!("--env key must not be empty");
    }
    if key.contains('\0') || value.contains('\0') {
        bail!("--env must not contain NUL bytes");
    }
    Ok((key.to_owned(), value.to_owned()))
}

fn apply_env_overrides(command: &mut Command, env_overrides: &[(String, String)]) {
    for (key, value) in env_overrides {
        command.env(key, value);
    }
}

fn format_env_override_keys(env_overrides: &[(String, String)]) -> String {
    if env_overrides.is_empty() {
        return "[]".to_owned();
    }

    let keys = env_overrides
        .iter()
        .map(|(key, _)| key.as_str())
        .collect::<Vec<_>>();
    format!("{keys:?}")
}

fn parallel_leader_participation_setting(disabled: bool) -> &'static str {
    if disabled {
        "SET parallel_leader_participation = off;"
    } else {
        ""
    }
}

fn pg18_plan_json_has_parallel_index_scan(plan: &str) -> bool {
    plan.contains("\"Node Type\": \"Gather Merge\"")
        && plan.contains("\"Index Name\": \"pg18_parallel_scan_fixture_idx\"")
        && plan.contains("\"Parallel Aware\": true")
}

fn format_optional_ids(ids: Option<&[i64]>) -> String {
    ids.map(|ids| format!("{ids:?}"))
        .unwrap_or_else(|| "not collected (planner-only)".to_owned())
}

async fn pg18_parallel_fixture_planner_diagnostics(
    client: &mut tokio_postgres::Client,
    limit: i64,
    workers: u16,
) -> Result<String> {
    let mut diagnostics = String::new();
    append_pg18_parallel_fixture_settings(client, &mut diagnostics).await?;
    append_pg18_parallel_fixture_catalog(client, &mut diagnostics).await?;

    let serial_json = pg18_parallel_fixture_explain_json(client, limit, 0).await?;
    append_section(
        &mut diagnostics,
        "serial ordered JSON plan (max_parallel_workers_per_gather=0)",
        &serial_json,
    );

    let (candidate_json, planner_pathlist_snapshot) =
        pg18_parallel_fixture_explain_json_with_pathlist_snapshot(client, limit, workers).await?;
    append_section(
        &mut diagnostics,
        "parallel-candidate ordered JSON plan",
        &candidate_json,
    );
    append_section(
        &mut diagnostics,
        "PG18 planner pathlist snapshot after parallel-candidate ordered plan",
        &planner_pathlist_snapshot,
    );

    let seqscan_json = pg18_parallel_fixture_parallel_seqscan_json(client).await?;
    append_section(
        &mut diagnostics,
        "parallel seqscan control JSON plan",
        &seqscan_json,
    );

    let ordered_seqscan_json =
        pg18_parallel_fixture_parallel_ordered_seqscan_json(client, limit).await?;
    append_section(
        &mut diagnostics,
        "parallel ordered seqscan control JSON plan",
        &ordered_seqscan_json,
    );

    Ok(diagnostics.trim_end().to_owned())
}

async fn append_pg18_parallel_fixture_settings(
    client: &tokio_postgres::Client,
    diagnostics: &mut String,
) -> Result<()> {
    let rows = client
        .query(
            "
SELECT name, setting
FROM pg_settings
WHERE name IN (
  'cpu_tuple_cost',
  'enable_bitmapscan',
  'enable_gathermerge',
  'enable_indexonlyscan',
  'enable_indexscan',
  'enable_incremental_sort',
  'enable_seqscan',
  'enable_sort',
  'max_parallel_workers',
  'max_parallel_workers_per_gather',
  'min_parallel_index_scan_size',
  'min_parallel_table_scan_size',
  'parallel_leader_participation',
  'parallel_setup_cost',
  'parallel_tuple_cost'
)
ORDER BY name
",
            &[],
        )
        .await?;

    diagnostics.push_str("settings:\n");
    for row in rows {
        let name: String = row.get(0);
        let setting: String = row.get(1);
        diagnostics.push_str(&format!("  {name}={setting}\n"));
    }
    diagnostics.push('\n');
    Ok(())
}

async fn append_pg18_parallel_fixture_catalog(
    client: &tokio_postgres::Client,
    diagnostics: &mut String,
) -> Result<()> {
    let relation_rows = client
        .query(
            "
SELECT c.relname::text,
       c.relkind::text,
       c.relpages::bigint,
       c.reltuples::double precision,
       COALESCE(array_to_string(c.reloptions, ','), '') AS reloptions
FROM pg_class c
WHERE c.oid IN (
  'pg18_parallel_scan_fixture'::regclass,
  'pg18_parallel_scan_fixture_idx'::regclass
)
ORDER BY c.relname
",
            &[],
        )
        .await?;
    diagnostics.push_str("relations:\n");
    for row in relation_rows {
        let relname: String = row.get(0);
        let relkind: String = row.get(1);
        let relpages: i64 = row.get(2);
        let reltuples: f64 = row.get(3);
        let reloptions: String = row.get(4);
        diagnostics.push_str(&format!(
            "  {relname} kind={relkind} relpages={relpages} reltuples={reltuples:.0} reloptions={reloptions}\n"
        ));
    }
    diagnostics.push('\n');

    let operator_rows = client
        .query(
            "
SELECT o.oid::regoperator::text,
       p.oid::regprocedure::text,
       CASE p.proparallel
         WHEN 's' THEN 'safe'
         WHEN 'r' THEN 'restricted'
         WHEN 'u' THEN 'unsafe'
         ELSE p.proparallel::text
       END AS proparallel,
       CASE p.provolatile
         WHEN 'i' THEN 'immutable'
         WHEN 's' THEN 'stable'
         WHEN 'v' THEN 'volatile'
         ELSE p.provolatile::text
       END AS provolatile
FROM pg_operator o
JOIN pg_proc p ON p.oid = o.oprcode
WHERE o.oprname = '<#>'
  AND o.oprleft = 'ecvector'::regtype
  AND o.oprright = 'real[]'::regtype
",
            &[],
        )
        .await?;
    diagnostics.push_str("operator:\n");
    for row in operator_rows {
        let operator: String = row.get(0);
        let procedure: String = row.get(1);
        let proparallel: String = row.get(2);
        let provolatile: String = row.get(3);
        diagnostics.push_str(&format!(
            "  {operator} procedure={procedure} parallel={proparallel} volatility={provolatile}\n"
        ));
    }
    diagnostics.push('\n');

    let opclass_rows = client
        .query(
            "
SELECT opc.opcname::text,
       am.amname::text,
       opc.opcintype::regtype::text,
       opf.opfname::text,
       pg_index_has_property('pg18_parallel_scan_fixture_idx'::regclass, 'index_scan')::text,
       pg_index_has_property('pg18_parallel_scan_fixture_idx'::regclass, 'bitmap_scan')::text,
       pg_index_column_has_property('pg18_parallel_scan_fixture_idx'::regclass, 1, 'distance_orderable')::text
FROM pg_opclass opc
JOIN pg_am am ON am.oid = opc.opcmethod
JOIN pg_opfamily opf ON opf.oid = opc.opcfamily
WHERE opc.opcname = 'ecvector_ip_ops'
",
            &[],
        )
        .await?;
    diagnostics.push_str("opclass:\n");
    for row in opclass_rows {
        let opcname: String = row.get(0);
        let amname: String = row.get(1);
        let opcintype: String = row.get(2);
        let opfamily: String = row.get(3);
        let index_scan: Option<String> = row.get(4);
        let bitmap_scan: Option<String> = row.get(5);
        let distance_orderable: Option<String> = row.get(6);
        diagnostics.push_str(&format!(
            "  {opcname} am={amname} input={opcintype} opfamily={opfamily} index_scan={} bitmap_scan={} distance_orderable={}\n",
            index_scan.unwrap_or_else(|| "NULL".to_owned()),
            bitmap_scan.unwrap_or_else(|| "NULL".to_owned()),
            distance_orderable.unwrap_or_else(|| "NULL".to_owned()),
        ));
    }
    diagnostics.push('\n');

    let snapshot_rows = client
        .query(
            "
SELECT planner_scan_enabled,
       ordered_scan_ready,
       runtime_ordered_scan_ready,
       planner_cost_model_ready,
       planner_cost_callback_live,
       pg18_callback_surface_ready,
       pg18_diagnostics_surface_ready,
       pg18_read_stream_surface_ready,
       effective_ef_search,
       effective_source,
       next_runtime_blocker
FROM ec_hnsw_planner_integration_snapshot('pg18_parallel_scan_fixture_idx'::regclass)
",
            &[],
        )
        .await?;
    diagnostics.push_str("ec_hnsw planner snapshot:\n");
    for row in snapshot_rows {
        let planner_scan_enabled: bool = row.get(0);
        let ordered_scan_ready: bool = row.get(1);
        let runtime_ordered_scan_ready: bool = row.get(2);
        let planner_cost_model_ready: bool = row.get(3);
        let planner_cost_callback_live: bool = row.get(4);
        let pg18_callback_surface_ready: bool = row.get(5);
        let pg18_diagnostics_surface_ready: bool = row.get(6);
        let pg18_read_stream_surface_ready: bool = row.get(7);
        let effective_ef_search: i32 = row.get(8);
        let effective_source: String = row.get(9);
        let next_runtime_blocker: String = row.get(10);
        diagnostics.push_str(&format!(
            "  planner_scan_enabled={planner_scan_enabled} ordered_scan_ready={ordered_scan_ready} runtime_ordered_scan_ready={runtime_ordered_scan_ready} planner_cost_model_ready={planner_cost_model_ready} planner_cost_callback_live={planner_cost_callback_live}\n"
        ));
        diagnostics.push_str(&format!(
            "  pg18_callback_surface_ready={pg18_callback_surface_ready} pg18_diagnostics_surface_ready={pg18_diagnostics_surface_ready} pg18_read_stream_surface_ready={pg18_read_stream_surface_ready}\n"
        ));
        diagnostics.push_str(&format!(
            "  effective_ef_search={effective_ef_search} effective_source={effective_source}\n"
        ));
        diagnostics.push_str(&format!("  next_runtime_blocker={next_runtime_blocker}\n"));
    }
    diagnostics.push('\n');

    let cost_rows = client
        .query(
            "
SELECT relation_ef_search,
       session_ef_search,
       effective_ef_search,
       effective_source,
       m,
       dimensions,
       max_level,
       resolved_tree_height,
       tree_height_source,
       pg18_tree_height_callback_ready,
       index_pages,
       reltuples,
       random_page_cost,
       seq_page_cost,
       cpu_operator_cost,
       modeled_startup_cost,
       modeled_total_cost,
       modeled_selectivity,
       modeled_correlation
FROM ec_hnsw_index_cost_snapshot('pg18_parallel_scan_fixture_idx'::regclass)
",
            &[],
        )
        .await?;
    diagnostics.push_str("ec_hnsw cost snapshot:\n");
    for row in cost_rows {
        let relation_ef_search: i32 = row.get(0);
        let session_ef_search: Option<i32> = row.get(1);
        let effective_ef_search: i32 = row.get(2);
        let effective_source: String = row.get(3);
        let m: i32 = row.get(4);
        let dimensions: i32 = row.get(5);
        let max_level: i32 = row.get(6);
        let resolved_tree_height: f64 = row.get(7);
        let tree_height_source: String = row.get(8);
        let pg18_tree_height_callback_ready: bool = row.get(9);
        let index_pages: f64 = row.get(10);
        let reltuples: f64 = row.get(11);
        let random_page_cost: f64 = row.get(12);
        let seq_page_cost: f64 = row.get(13);
        let cpu_operator_cost: f64 = row.get(14);
        let modeled_startup_cost: f64 = row.get(15);
        let modeled_total_cost: f64 = row.get(16);
        let modeled_selectivity: f64 = row.get(17);
        let modeled_correlation: f64 = row.get(18);
        let modeled_run_cost = modeled_total_cost - modeled_startup_cost;
        let startup_fraction = if modeled_total_cost.is_finite() && modeled_total_cost > 0.0 {
            modeled_startup_cost / modeled_total_cost
        } else {
            0.0
        };
        let session_ef_search = session_ef_search
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".to_owned());
        diagnostics.push_str(&format!(
            "  effective_ef_search={effective_ef_search} effective_source={effective_source} relation_ef_search={relation_ef_search} session_ef_search={session_ef_search} m={m} dimensions={dimensions} max_level={max_level}\n"
        ));
        diagnostics.push_str(&format!(
            "  index_pages={index_pages:.0} reltuples={reltuples:.0} resolved_tree_height={resolved_tree_height:.3} tree_height_source={tree_height_source} pg18_tree_height_callback_ready={pg18_tree_height_callback_ready}\n"
        ));
        diagnostics.push_str(&format!(
            "  cost_constants random_page_cost={random_page_cost:.3} seq_page_cost={seq_page_cost:.3} cpu_operator_cost={cpu_operator_cost:.6}\n"
        ));
        diagnostics.push_str(&format!(
            "  modeled_startup_cost={modeled_startup_cost:.3} modeled_total_cost={modeled_total_cost:.3} modeled_run_cost={modeled_run_cost:.3} startup_fraction={startup_fraction:.6} modeled_selectivity={modeled_selectivity:.3} modeled_correlation={modeled_correlation:.3}\n"
        ));
    }
    diagnostics.push('\n');
    Ok(())
}

async fn pg18_parallel_fixture_explain_json(
    client: &mut tokio_postgres::Client,
    limit: i64,
    max_parallel_workers_per_gather: u16,
) -> Result<String> {
    let transaction = client.transaction().await?;
    transaction
        .batch_execute(
            format!(
                "SET LOCAL max_parallel_workers_per_gather = {max_parallel_workers_per_gather};"
            )
            .as_str(),
        )
        .await?;
    let row = transaction
        .query_one(
            format!(
                "
EXPLAIN (VERBOSE, FORMAT JSON)
SELECT id
FROM pg18_parallel_scan_fixture
ORDER BY embedding <#> ARRAY[0.75, 0.25, 0.5, -0.5]::real[]
LIMIT {limit}
"
            )
            .as_str(),
            &[],
        )
        .await?;
    let plan: Value = row.get(0);
    transaction.commit().await?;
    Ok(serde_json::to_string_pretty(&plan)?)
}

async fn pg18_parallel_fixture_explain_json_with_pathlist_snapshot(
    client: &mut tokio_postgres::Client,
    limit: i64,
    max_parallel_workers_per_gather: u16,
) -> Result<(String, String)> {
    client
        .batch_execute("SELECT ec_hnsw_reset_planner_path_snapshot();")
        .await?;
    let plan =
        pg18_parallel_fixture_explain_json(client, limit, max_parallel_workers_per_gather).await?;
    let pathlist_snapshot = pg18_parallel_fixture_pathlist_snapshot(client).await?;
    Ok((plan, pathlist_snapshot))
}

async fn pg18_parallel_fixture_pathlist_snapshot(
    client: &tokio_postgres::Client,
) -> Result<String> {
    let rows = client
        .query(
            "
SELECT hook_registered,
       observed,
       relid,
       consider_parallel,
       rel_parallel_workers,
       ec_hnsw_index_count,
       amcanparallel_seen,
       path_count,
       index_path_count,
       ec_hnsw_index_path_count,
       partial_path_count,
       partial_index_path_count,
       partial_ec_hnsw_index_path_count,
       best_plain_ec_hnsw_startup_cost,
       best_plain_ec_hnsw_total_cost,
       best_plain_ec_hnsw_parallel_workers,
       best_plain_ec_hnsw_pathkeys,
       best_partial_ec_hnsw_startup_cost,
       best_partial_ec_hnsw_total_cost,
       best_partial_ec_hnsw_parallel_workers,
       best_partial_ec_hnsw_parallel_aware,
       best_partial_ec_hnsw_pathkeys
FROM ec_hnsw_planner_path_snapshot()
",
            &[],
        )
        .await?;

    let mut snapshot = String::new();
    for row in rows {
        let hook_registered: bool = row.get(0);
        let observed: bool = row.get(1);
        let relid: i32 = row.get(2);
        let consider_parallel: bool = row.get(3);
        let rel_parallel_workers: i32 = row.get(4);
        let ec_hnsw_index_count: i32 = row.get(5);
        let amcanparallel_seen: bool = row.get(6);
        let path_count: i32 = row.get(7);
        let index_path_count: i32 = row.get(8);
        let ec_hnsw_index_path_count: i32 = row.get(9);
        let partial_path_count: i32 = row.get(10);
        let partial_index_path_count: i32 = row.get(11);
        let partial_ec_hnsw_index_path_count: i32 = row.get(12);
        let best_plain_ec_hnsw_startup_cost: Option<f64> = row.get(13);
        let best_plain_ec_hnsw_total_cost: Option<f64> = row.get(14);
        let best_plain_ec_hnsw_parallel_workers: Option<i32> = row.get(15);
        let best_plain_ec_hnsw_pathkeys: Option<i32> = row.get(16);
        let best_partial_ec_hnsw_startup_cost: Option<f64> = row.get(17);
        let best_partial_ec_hnsw_total_cost: Option<f64> = row.get(18);
        let best_partial_ec_hnsw_parallel_workers: Option<i32> = row.get(19);
        let best_partial_ec_hnsw_parallel_aware: Option<bool> = row.get(20);
        let best_partial_ec_hnsw_pathkeys: Option<i32> = row.get(21);

        snapshot.push_str(&format!(
            "  hook_registered={hook_registered} observed={observed} relid={relid} consider_parallel={consider_parallel} rel_parallel_workers={rel_parallel_workers}\n"
        ));
        snapshot.push_str(&format!(
            "  ec_hnsw_index_count={ec_hnsw_index_count} amcanparallel_seen={amcanparallel_seen}\n"
        ));
        snapshot.push_str(&format!(
            "  path_count={path_count} index_path_count={index_path_count} ec_hnsw_index_path_count={ec_hnsw_index_path_count}\n"
        ));
        snapshot.push_str(&format!(
            "  partial_path_count={partial_path_count} partial_index_path_count={partial_index_path_count} partial_ec_hnsw_index_path_count={partial_ec_hnsw_index_path_count}\n"
        ));
        snapshot.push_str(&format!(
            "  best_plain_ec_hnsw startup_cost={} total_cost={} parallel_workers={} pathkeys={}\n",
            format_optional_cost(best_plain_ec_hnsw_startup_cost),
            format_optional_cost(best_plain_ec_hnsw_total_cost),
            format_optional_i32(best_plain_ec_hnsw_parallel_workers),
            format_optional_i32(best_plain_ec_hnsw_pathkeys),
        ));
        snapshot.push_str(&format!(
            "  best_partial_ec_hnsw startup_cost={} total_cost={} parallel_workers={} parallel_aware={} pathkeys={}\n",
            format_optional_cost(best_partial_ec_hnsw_startup_cost),
            format_optional_cost(best_partial_ec_hnsw_total_cost),
            format_optional_i32(best_partial_ec_hnsw_parallel_workers),
            format_optional_bool(best_partial_ec_hnsw_parallel_aware),
            format_optional_i32(best_partial_ec_hnsw_pathkeys),
        ));
    }

    Ok(snapshot.trim_end().to_owned())
}

fn format_optional_cost(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "NULL".to_owned())
}

fn format_optional_i32(value: Option<i32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "NULL".to_owned())
}

fn format_optional_bool(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "NULL".to_owned())
}

async fn pg18_parallel_fixture_parallel_seqscan_json(
    client: &mut tokio_postgres::Client,
) -> Result<String> {
    let transaction = client.transaction().await?;
    transaction
        .batch_execute(
            "
SET LOCAL enable_seqscan = on;
SET LOCAL enable_indexscan = off;
SET LOCAL enable_indexonlyscan = off;
SET LOCAL enable_bitmapscan = off;
",
        )
        .await?;
    let row = transaction
        .query_one(
            "
EXPLAIN (VERBOSE, FORMAT JSON)
SELECT id
FROM pg18_parallel_scan_fixture
WHERE id > 0
",
            &[],
        )
        .await?;
    let plan: Value = row.get(0);
    transaction.commit().await?;
    Ok(serde_json::to_string_pretty(&plan)?)
}

async fn pg18_parallel_fixture_parallel_ordered_seqscan_json(
    client: &mut tokio_postgres::Client,
    limit: i64,
) -> Result<String> {
    let transaction = client.transaction().await?;
    transaction
        .batch_execute(
            "
SET LOCAL enable_seqscan = on;
SET LOCAL enable_indexscan = off;
SET LOCAL enable_indexonlyscan = off;
SET LOCAL enable_bitmapscan = off;
",
        )
        .await?;
    let row = transaction
        .query_one(
            format!(
                "
EXPLAIN (VERBOSE, FORMAT JSON)
SELECT id
FROM pg18_parallel_scan_fixture
ORDER BY embedding <#> ARRAY[0.75, 0.25, 0.5, -0.5]::real[]
LIMIT {limit}
"
            )
            .as_str(),
            &[],
        )
        .await?;
    let plan: Value = row.get(0);
    transaction.commit().await?;
    Ok(serde_json::to_string_pretty(&plan)?)
}

fn append_section(diagnostics: &mut String, label: &str, body: &str) {
    diagnostics.push_str(label);
    diagnostics.push_str(":\n");
    diagnostics.push_str(body);
    diagnostics.push_str("\n\n");
}

async fn start_pg18_validation_cluster(
    pgrx_home: &std::path::Path,
    cluster_name: &str,
    port: u16,
    env_overrides: &[(String, String)],
) -> Result<Pg18ValidationCluster> {
    let repo_root = repo_root()?;
    let install = find_pgrx_install(18, pgrx_home)?;
    assert_pg18_install_ready(&install)?;

    let cluster_root = repo_root.join("target").join(cluster_name);
    let data_dir = cluster_root.join("data");
    let log_file = cluster_root.join("postgres.log");
    fs::create_dir_all(&cluster_root)
        .wrap_err_with(|| format!("creating {}", cluster_root.display()))?;

    let initdb = install.bin_dir.join("initdb");
    let pg_ctl = install.bin_dir.join("pg_ctl");
    if !data_dir.join("PG_VERSION").is_file() {
        let mut command = Command::new(&initdb);
        command
            .arg("-D")
            .arg(&data_dir)
            .arg("-A")
            .arg("trust")
            .arg("-U")
            .arg("postgres");
        apply_env_overrides(&mut command, env_overrides);
        run_status(command).await?;
    }

    let cluster = PgClusterGuard::new(pg_ctl.clone(), data_dir.clone());
    cluster.stop().await?;

    let mut selected_port = None;
    for offset in 0..10 {
        let candidate = port + offset;
        fs::write(&log_file, "").wrap_err_with(|| format!("resetting {}", log_file.display()))?;
        let mut command = Command::new(&pg_ctl);
        command
            .arg("-D")
            .arg(&data_dir)
            .arg("-l")
            .arg(&log_file)
            .arg("-o")
            .arg(format!(
                "-p {candidate} -c listen_addresses=127.0.0.1 -c shared_preload_libraries=ecaz"
            ))
            .arg("-w")
            .arg("start");
        apply_env_overrides(&mut command, env_overrides);
        let output = command
            .output()
            .await
            .wrap_err("starting PG18 validation cluster")?;
        if output.status.success() {
            selected_port = Some(candidate);
            break;
        }
        let log = fs::read_to_string(&log_file).unwrap_or_default();
        if !log.contains("Address already in use") {
            bail!(
                "pg_ctl start failed on port {}: {}{}",
                candidate,
                String::from_utf8_lossy(&output.stderr),
                log
            );
        }
    }
    let selected_port = selected_port
        .ok_or_else(|| eyre!("could not find a free local port starting at {port}"))?;

    let base = psql::ConnectParams {
        database: "postgres".into(),
        host: Some("127.0.0.1".into()),
        port: Some(selected_port),
        user: Some("postgres".into()),
        password: None,
    };
    let observer = psql::connect_with(&base).await?;
    let preload_setting = single_text(&observer, "SHOW shared_preload_libraries").await?;
    if !preload_setting.contains("ecaz") {
        bail!("shared_preload_libraries should include ecaz, got {preload_setting}");
    }

    Ok(Pg18ValidationCluster {
        install_version_label: install.version_label,
        base,
        preload_setting,
        _guard: cluster,
    })
}

async fn pg18_parallel_fixture_ids(
    client: &tokio_postgres::Client,
    limit: i64,
) -> Result<Vec<i64>> {
    let rows = client
        .query(
            format!(
                "
SELECT id
FROM pg18_parallel_scan_fixture
ORDER BY embedding <#> ARRAY[0.75, 0.25, 0.5, -0.5]::real[]
LIMIT {limit}
"
            )
            .as_str(),
            &[],
        )
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| row.get::<_, i64>(0))
        .collect::<Vec<_>>())
}

async fn pg18_parallel_fixture_explain_analyze(
    client: &tokio_postgres::Client,
    limit: i64,
) -> Result<String> {
    let rows = client
        .query(
            format!(
                "
EXPLAIN (ANALYZE, COSTS OFF, SUMMARY OFF)
SELECT id
FROM pg18_parallel_scan_fixture
ORDER BY embedding <#> ARRAY[0.75, 0.25, 0.5, -0.5]::real[]
LIMIT {limit}
"
            )
            .as_str(),
            &[],
        )
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| row.get::<_, String>(0))
        .collect::<Vec<_>>()
        .join("\n"))
}

async fn pg18_parallel_fixture_parallel_seqscan_plan(
    client: &mut tokio_postgres::Client,
) -> Result<String> {
    let transaction = client.transaction().await?;
    transaction
        .batch_execute(
            "
SET LOCAL enable_seqscan = on;
SET LOCAL enable_indexscan = off;
SET LOCAL enable_indexonlyscan = off;
SET LOCAL enable_bitmapscan = off;
",
        )
        .await?;
    let rows = transaction
        .query(
            "
EXPLAIN (ANALYZE, COSTS OFF, SUMMARY OFF)
SELECT id
FROM pg18_parallel_scan_fixture
WHERE id > 0
",
            &[],
        )
        .await?;
    transaction.commit().await?;
    Ok(rows
        .into_iter()
        .map(|row| row.get::<_, String>(0))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn assert_pg18_install_ready(install: &super::support::PgrxInstall) -> Result<()> {
    let control_file = install.root.join("share/postgresql/extension/ecaz.control");
    let library_file = install.root.join("lib/postgresql/ecaz.so");
    if !control_file.is_file() || !library_file.is_file() {
        bail!(
            "ecaz is not installed in the local PG18 pgrx tree at {}; run `cargo pgrx test pg18` or `cargo pgrx install --features 'pg18 pg_test' --no-default-features` first",
            install.root.display()
        );
    }
    Ok(())
}

struct Pg18ValidationCluster {
    install_version_label: String,
    base: psql::ConnectParams,
    preload_setting: String,
    _guard: PgClusterGuard,
}

async fn single_text(client: &tokio_postgres::Client, sql: &str) -> Result<String> {
    let row = client.query_one(sql, &[]).await?;
    Ok(row.get::<_, String>(0))
}

struct PgClusterGuard {
    pg_ctl: PathBuf,
    data_dir: PathBuf,
}

impl PgClusterGuard {
    fn new(pg_ctl: PathBuf, data_dir: PathBuf) -> Self {
        Self { pg_ctl, data_dir }
    }

    async fn stop(&self) -> Result<()> {
        if !self.data_dir.join("PG_VERSION").is_file() {
            return Ok(());
        }
        let output = Command::new(&self.pg_ctl)
            .arg("-D")
            .arg(&self.data_dir)
            .arg("status")
            .output()
            .await
            .wrap_err("checking PG cluster status")?;
        if !output.status.success() {
            return Ok(());
        }
        if self.stop_with_mode("fast").await.is_ok() {
            return Ok(());
        }

        self.stop_with_mode("immediate").await
    }

    async fn stop_with_mode(&self, mode: &str) -> Result<()> {
        let mut command = Command::new(&self.pg_ctl);
        command
            .arg("-D")
            .arg(&self.data_dir)
            .arg("-m")
            .arg(mode)
            .arg("-w")
            .arg("stop");
        run_status(command).await
    }
}

impl Drop for PgClusterGuard {
    fn drop(&mut self) {
        if !self.data_dir.join("PG_VERSION").is_file() {
            return;
        }
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
