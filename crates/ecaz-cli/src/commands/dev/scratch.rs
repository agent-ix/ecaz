use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

use super::support::{
    default_pgrx_home, find_pgrx_install, refresh_debug_helpers_sql, repo_root, resolve_pgrx_home,
    run_status, scratch_socket_dir, SCRATCH_DEFAULT_PORT,
};

#[derive(Subcommand, Debug)]
pub enum ScratchCommand {
    /// Restart the pg17 scratch cluster with the requested runtime environment.
    Restart(ScratchRestartArgs),
    /// Run psql against the pg17 scratch cluster.
    Sql(ScratchSqlArgs),
    /// Refresh the bundled ADR-030 scratch debug SQL wrappers in the target database.
    RefreshDebugHelpers(ScratchRefreshDebugHelpersArgs),
}

impl ScratchCommand {
    pub async fn run(self, database: &str) -> Result<()> {
        match self {
            ScratchCommand::Restart(args) => run_restart(args).await,
            ScratchCommand::Sql(args) => run_sql(database, args).await,
            ScratchCommand::RefreshDebugHelpers(args) => run_refresh(args).await,
        }
    }
}

#[derive(Args, Debug)]
pub struct ScratchRestartArgs {
    #[arg(long, default_value_t = 64)]
    window: u32,

    #[arg(long, default_value = "binary")]
    grouped_score_mode: String,

    #[arg(long, default_value = "heap_f32")]
    rerank_mode: String,

    #[arg(long)]
    rerank_source_column: Option<String>,

    #[arg(long)]
    exact_scope: Option<String>,

    #[arg(long)]
    exact_strategy: Option<String>,

    #[arg(long)]
    exact_limit: Option<String>,

    /// Extra environment assignment. Repeatable `NAME=VALUE`.
    #[arg(long = "env")]
    env: Vec<String>,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct ScratchSqlArgs {
    /// Target database. Defaults to the global `--database`.
    #[arg(long)]
    db: Option<String>,

    /// Explicit socket directory.
    #[arg(long)]
    socket_dir: Option<PathBuf>,

    /// Scratch-cluster port.
    #[arg(long, default_value_t = SCRATCH_DEFAULT_PORT)]
    port: u16,

    /// Emit raw psql output instead of aligned-off, tuples-only TSV.
    #[arg(long)]
    raw: bool,

    /// SQL to run.
    #[arg(long)]
    sql: Option<String>,

    /// SQL file to run.
    #[arg(long)]
    file: Option<PathBuf>,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct ScratchRefreshDebugHelpersArgs {
    /// Target database. Defaults to the global `--database`.
    #[arg(long)]
    db: Option<String>,
}

async fn run_restart(args: ScratchRestartArgs) -> Result<()> {
    let repo_root = repo_root()?;
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let data_dir = pgrx_home.join("data-17");
    stop_existing_postmaster(&data_dir).await?;

    println!("[scratch] repo={}", repo_root.display());
    println!("[scratch] pgrx_home={}", pgrx_home.display());
    println!("[scratch] window={}", args.window);
    println!("[scratch] grouped_score_mode={}", args.grouped_score_mode);
    println!("[scratch] rerank_mode={}", args.rerank_mode);
    println!(
        "[scratch] rerank_source_column={}",
        args.rerank_source_column
            .as_deref()
            .unwrap_or("build_source_column")
    );
    if args.exact_scope.is_some() || args.exact_limit.is_some() || args.exact_strategy.is_some() {
        println!(
            "[scratch] exact_scope={}",
            args.exact_scope.as_deref().unwrap_or("all")
        );
        println!(
            "[scratch] exact_strategy={}",
            args.exact_strategy.as_deref().unwrap_or("expansion")
        );
        println!(
            "[scratch] exact_limit={}",
            args.exact_limit.as_deref().unwrap_or("all")
        );
    } else {
        println!("[scratch] exact_scope=disabled");
    }
    for assignment in &args.env {
        println!("[scratch] extra_env={assignment}");
    }

    let mut command = Command::new("cargo");
    command
        .arg("pgrx")
        .arg("start")
        .arg("pg17")
        .current_dir(&repo_root)
        .env("PGRX_HOME", &pgrx_home)
        .env("TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW", args.window.to_string())
        .env(
            "TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE",
            &args.grouped_score_mode,
        )
        .env("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", &args.rerank_mode)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(column) = &args.rerank_source_column {
        command.env("TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN", column);
    }
    if args.exact_scope.is_some() || args.exact_limit.is_some() || args.exact_strategy.is_some() {
        command.env("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL", "1");
        command.env(
            "TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_SCOPE",
            args.exact_scope.as_deref().unwrap_or("all"),
        );
        command.env(
            "TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_STRATEGY",
            args.exact_strategy.as_deref().unwrap_or("expansion"),
        );
        if let Some(limit) = &args.exact_limit {
            command.env("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_LIMIT", limit);
        }
    }
    for assignment in args.env {
        let (name, value) = assignment.split_once('=').ok_or_else(|| {
            color_eyre::eyre::eyre!("--env values must be NAME=VALUE, got: {assignment}")
        })?;
        if name.is_empty() {
            bail!("--env values must include a variable name");
        }
        command.env(name, value);
    }
    run_status(command).await
}

async fn stop_existing_postmaster(data_dir: &Path) -> Result<()> {
    let pid_file = data_dir.join("postmaster.pid");
    if !pid_file.is_file() {
        return Ok(());
    }
    let pid = fs::read_to_string(&pid_file)
        .wrap_err_with(|| format!("reading {}", pid_file.display()))?
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    if pid.is_empty() {
        return Ok(());
    }
    let mut kill_check = Command::new("kill");
    kill_check.arg("-0").arg(&pid);
    if kill_check.status().await?.success() {
        println!("[scratch] stopping existing postmaster pid={pid}");
        let mut stop = Command::new("kill");
        stop.arg(&pid);
        run_status(stop).await?;
    }
    Ok(())
}

async fn run_sql(database: &str, args: ScratchSqlArgs) -> Result<()> {
    if args.sql.is_some() && args.file.is_some() {
        bail!("--sql and --file are mutually exclusive");
    }
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let install = find_pgrx_install(17, &pgrx_home)?;
    let socket_dir = scratch_socket_dir(args.socket_dir.as_ref(), &pgrx_home, args.port)?;
    let mut command = Command::new(install.bin_dir.join("psql"));
    command
        .arg("-h")
        .arg(socket_dir)
        .arg("-p")
        .arg(args.port.to_string())
        .arg("-d")
        .arg(args.db.unwrap_or_else(|| database.to_string()))
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if !args.raw {
        command.arg("-A").arg("-t").arg("-F").arg("\t");
    }
    if let Some(sql) = args.sql {
        command.arg("-c").arg(sql);
    } else if let Some(file) = args.file {
        command.arg("-f").arg(file);
    }
    run_status(command).await
}

async fn run_refresh(args: ScratchRefreshDebugHelpersArgs) -> Result<()> {
    let database = args.db.unwrap_or_else(|| "postgres".to_string());
    let sql_file = refresh_debug_helpers_sql()?;
    run_sql(
        &database,
        ScratchSqlArgs {
            db: Some(database.clone()),
            socket_dir: None,
            port: SCRATCH_DEFAULT_PORT,
            raw: false,
            sql: None,
            file: Some(sql_file),
            pgrx_home: Some(default_pgrx_home()),
        },
    )
    .await
}
