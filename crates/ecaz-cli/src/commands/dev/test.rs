use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, eyre, Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use crate::psql;

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
}

impl TestCommand {
    pub async fn run(self, _database: &str) -> Result<()> {
        match self {
            TestCommand::Pgrx(args) => run_pgrx(args).await,
            TestCommand::Pg18PreloadPgstat(args) => run_pg18_preload_pgstat(args).await,
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
                "-p {candidate} -c listen_addresses=127.0.0.1 -c shared_preload_libraries=ecaz"
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

    let base = psql::ConnectParams {
        database: "postgres".into(),
        host: Some("127.0.0.1".into()),
        port: Some(selected_port),
        user: Some("postgres".into()),
        password: None,
    };
    let observer = psql::connect_with(&base).await?;
    let actor = psql::connect_with(&base).await?;

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

    println!("[pg18-preload] install={}", install.version_label);
    println!("[pg18-preload] shared_preload_libraries={preload_setting}");
    println!(
        "[pg18-preload] baseline_scans={baseline_scans} baseline_distance_calcs={baseline_distance}"
    );
    println!("[pg18-preload] shared_scans={shared_scans} shared_distance_calcs={shared_distance}");
    println!("[pg18-preload] preload-aware PG18 shared pgstat validation passed");
    Ok(())
}

fn assert_preload_install_ready(install: &super::support::PgrxInstall) -> Result<()> {
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
