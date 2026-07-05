use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, eyre, Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use crate::{profiles, psql};

use super::support::{
    find_pgrx_install, repo_root, resolve_pgrx_home, run_status, DEFAULT_PG_MAJOR,
    PG18_PRELOAD_DEFAULT_PORT,
};

#[derive(Subcommand, Debug)]
pub enum TestCommand {
    /// Run `cargo pgrx test` through the CLI-owned test surface.
    Pgrx(PgrxTestArgs),
    /// Start a repo-local PG18 cluster with preload enabled and validate shared pgstat visibility.
    Pg18PreloadPgstat(Pg18PreloadPgstatArgs),
    /// Run the Task 71 one-cell IVF parallel-build probe against a local test DB.
    IvfParallelBuildProbe(IvfParallelBuildProbeArgs),
    /// Drop Task 71 IVF parallel-build matrix/probe tables from a local test DB.
    IvfParallelBuildClean(IvfParallelBuildCleanArgs),
}

impl TestCommand {
    pub async fn run(self, conn: &psql::ConnectionOptions) -> Result<()> {
        match self {
            TestCommand::Pgrx(args) => run_pgrx(args).await,
            TestCommand::Pg18PreloadPgstat(args) => run_pg18_preload_pgstat(args).await,
            TestCommand::IvfParallelBuildProbe(args) => {
                run_ivf_parallel_build_probe(conn, args).await
            }
            TestCommand::IvfParallelBuildClean(args) => {
                run_ivf_parallel_build_clean(conn, args).await
            }
        }
    }
}

#[derive(Args, Debug)]
pub struct PgrxTestArgs {
    /// PostgreSQL major version to run.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

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
}

#[derive(Args, Debug)]
pub struct IvfParallelBuildProbeArgs {
    /// Drop the probe corpus/query tables before loading.
    #[arg(long)]
    drop_first: bool,

    /// Fixture prefix used for probe tables and index. Defaults to task71_probe_w<workers>.
    #[arg(long)]
    prefix: Option<String>,

    /// Requested parallel workers for both table reloption and session GUCs.
    #[arg(long, default_value_t = 2)]
    workers: i32,

    /// Packet-local artifact directory for the loader log.
    #[arg(long, default_value = "reviews/task-71/003-worker-curve/artifacts")]
    artifact_dir: PathBuf,
}

#[derive(Args, Debug)]
pub struct IvfParallelBuildCleanArgs {
    /// Also drop the one-cell probe prefix for the selected worker count.
    #[arg(long)]
    include_probe: bool,

    /// Probe worker count used when --include-probe is set.
    #[arg(long, default_value_t = 2)]
    probe_workers: i32,
}

async fn run_pgrx(args: PgrxTestArgs) -> Result<()> {
    let repo_root = repo_root()?;
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
    run_status(command).await
}

async fn run_ivf_parallel_build_probe(
    conn: &psql::ConnectionOptions,
    args: IvfParallelBuildProbeArgs,
) -> Result<()> {
    if args.workers < 0 {
        bail!("--workers must be non-negative");
    }
    let prefix = args
        .prefix
        .unwrap_or_else(|| format!("task71_probe_w{}", args.workers));
    profiles::validate_ident(&prefix).wrap_err_with(|| format!("invalid --prefix {prefix:?}"))?;
    fs::create_dir_all(&args.artifact_dir)
        .wrap_err_with(|| format!("creating {}", args.artifact_dir.display()))?;

    ensure_ivf_build_timing_function(conn).await?;

    if args.drop_first {
        let client = psql::connect(conn).await?;
        client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS {prefix}_corpus, {prefix}_queries CASCADE;",
                prefix = prefix
            ))
            .await
            .wrap_err("dropping IVF parallel-build probe tables")?;
        crate::ecaz_println!("[ivf-probe] dropped prefix {}", prefix);
    }

    let exe = std::env::current_exe().wrap_err("resolving current ecaz binary")?;
    let log_file = args.artifact_dir.join(format!(
        "probe-load-real10k-w{}-after-loader-timing.log",
        args.workers
    ));
    let pgoptions = format!(
        "-c max_parallel_maintenance_workers={} -c max_parallel_workers={}",
        args.workers, args.workers
    );
    let mut command = Command::new(exe);
    command
        .env("PGOPTIONS", pgoptions)
        .arg("--database")
        .arg(&conn.database);
    if let Some(host) = &conn.host {
        command.arg("--host").arg(host);
    }
    if let Some(port) = conn.port {
        command.arg("--port").arg(port.to_string());
    }
    if let Some(user) = &conn.user {
        command.arg("--user").arg(user);
    }
    if let Some(password) = &conn.password {
        command.env("PGPASSWORD", password);
    }
    command
        .arg("--log-file")
        .arg(&log_file)
        .arg("corpus")
        .arg("load")
        .arg("--prefix")
        .arg(&prefix)
        .arg("--profile")
        .arg("ec_ivf")
        .arg("--corpus-file")
        .arg("data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_corpus.tsv")
        .arg("--queries-file")
        .arg("data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_queries.tsv")
        .arg("--manifest-file")
        .arg("data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_manifest.json")
        .arg("--allow-manifest-mismatch")
        .arg("--bits")
        .arg("4")
        .arg("--seed")
        .arg("42")
        .arg("--table-reloption")
        .arg(format!("parallel_workers={}", args.workers))
        .arg("--reloption")
        .arg("storage_format=pq_fastscan")
        .arg("--reloption")
        .arg("pq_group_size=8")
        .arg("--reloption")
        .arg("nlists=64")
        .arg("--reloption")
        .arg("nprobe=48")
        .arg("--reloption")
        .arg("rerank=heap_f32")
        .arg("--reloption")
        .arg("rerank_width=750")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    crate::ecaz_println!(
        "[ivf-probe] running prefix={} workers={} log={}",
        prefix,
        args.workers,
        log_file.display()
    );
    run_status(command).await
}

async fn run_ivf_parallel_build_clean(
    conn: &psql::ConnectionOptions,
    args: IvfParallelBuildCleanArgs,
) -> Result<()> {
    if args.probe_workers < 0 {
        bail!("--probe-workers must be non-negative");
    }
    let mut prefixes = Vec::new();
    for scale in ["10k", "25k", "50k", "100k"] {
        for workers in [1, 2, 4, 8] {
            prefixes.push(format!("task71_real{scale}_w{workers}"));
        }
    }
    if args.include_probe {
        prefixes.push(format!("task71_probe_w{}", args.probe_workers));
    }
    for prefix in &prefixes {
        profiles::validate_ident(prefix).wrap_err_with(|| format!("invalid prefix {prefix:?}"))?;
    }

    let drops = prefixes
        .iter()
        .flat_map(|prefix| [format!("{prefix}_corpus"), format!("{prefix}_queries")])
        .collect::<Vec<_>>()
        .join(", ");
    let client = psql::connect(conn).await?;
    client
        .batch_execute(&format!("DROP TABLE IF EXISTS {drops} CASCADE;"))
        .await
        .wrap_err("dropping Task 71 IVF parallel-build tables")?;
    crate::ecaz_println!("[ivf-clean] dropped {} prefixes", prefixes.len());
    Ok(())
}

async fn ensure_ivf_build_timing_function(conn: &psql::ConnectionOptions) -> Result<()> {
    let client = psql::connect(conn).await?;
    client
        .batch_execute(
            "DROP FUNCTION IF EXISTS ec_ivf_last_build_timing();
             CREATE FUNCTION ec_ivf_last_build_timing()
             RETURNS TABLE (
                 requested_workers bigint,
                 workers_launched bigint,
                 heap_tuples bigint,
                 index_tuples bigint,
                 heap_ingest_us bigint,
                 train_model_us bigint,
                 stage_build_plan_us bigint,
                 stage_pq_train_us bigint,
                 stage_centroids_us bigint,
                 stage_assign_us bigint,
                 stage_postings_us bigint,
                 stage_directory_us bigint,
                 flush_build_plan_us bigint,
                 parallel_begin_us bigint,
                 parallel_drain_us bigint,
                 parallel_sort_push_us bigint,
                 parallel_worker_tuple_buffer_capacity bigint,
                 parallel_worker_tuple_buffer_struct_bytes bigint
             )
             STRICT
             LANGUAGE c
             AS '$libdir/ecaz', 'ec_ivf_last_build_timing_wrapper';",
        )
        .await
        .wrap_err("creating ec_ivf_last_build_timing() probe helper")?;
    crate::ecaz_println!("[ivf-probe] ensured ec_ivf_last_build_timing()");
    Ok(())
}

async fn run_pg18_preload_pgstat(args: Pg18PreloadPgstatArgs) -> Result<()> {
    let repo_root = repo_root()?;
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let install = find_pgrx_install(18, &pgrx_home)?;
    assert_preload_install_ready(&install)?;

    let cluster_root = repo_root.join("target/pg18-preload-pgstat");
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
        run_status(command).await?;
    }

    let cluster = PgClusterGuard::new(pg_ctl.clone(), data_dir.clone());
    cluster.stop().await?;

    let mut selected_port = None;
    for offset in 0..10 {
        let candidate = args.port + offset;
        fs::write(&log_file, "").wrap_err_with(|| format!("resetting {}", log_file.display()))?;
        let output = Command::new(&pg_ctl)
            .arg("-D")
            .arg(&data_dir)
            .arg("-l")
            .arg(&log_file)
            .arg("-o")
            .arg(format!(
                "-p {candidate} -c listen_addresses=127.0.0.1 -c unix_socket_directories={} -c shared_preload_libraries=ecaz",
                cluster_root.display()
            ))
            .arg("-w")
            .arg("start")
            .output()
            .await
            .wrap_err("starting PG18 preload validation cluster")?;
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
        .ok_or_else(|| eyre!("could not find a free local port starting at {}", args.port))?;

    let base = psql::ConnectionOptions {
        database: "postgres".into(),
        host: Some("127.0.0.1".into()),
        port: Some(selected_port),
        user: Some("postgres".into()),
        password: None,
    };
    let observer = psql::connect(&base).await?;
    let actor = psql::connect(&base).await?;

    let preload_setting = single_text(&observer, "SHOW shared_preload_libraries").await?;
    if !preload_setting.contains("ecaz") {
        bail!("shared_preload_libraries should include ecaz, got {preload_setting}");
    }

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

    crate::ecaz_println!("[pg18-preload] install={}", install.version_label);
    crate::ecaz_println!("[pg18-preload] shared_preload_libraries={preload_setting}");
    crate::ecaz_println!(
        "[pg18-preload] baseline_scans={baseline_scans} baseline_distance_calcs={baseline_distance}"
    );
    crate::ecaz_println!(
        "[pg18-preload] shared_scans={shared_scans} shared_distance_calcs={shared_distance}"
    );
    crate::ecaz_println!("[pg18-preload] preload-aware PG18 shared pgstat validation passed");
    Ok(())
}

fn assert_preload_install_ready(install: &super::support::PgrxInstall) -> Result<()> {
    let control_file = install.sharedir.join("extension/ecaz.control");
    let library_file = install.pkglibdir.join("ecaz.so");
    if !control_file.is_file() || !library_file.is_file() {
        bail!(
            "ecaz is not installed for PG18 via {}; missing {} or {}; run `cargo pgrx install --features 'pg18 pg_test' --no-default-features` first",
            install.pg_config.display(),
            control_file.display(),
            library_file.display()
        );
    }
    Ok(())
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
        let mut command = Command::new(&self.pg_ctl);
        command
            .arg("-D")
            .arg(&self.data_dir)
            .arg("-m")
            .arg("fast")
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
