use clap::Args;
use color_eyre::eyre::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use super::support::{
    find_pgrx_install, repo_root, resolve_pgrx_home, run_status, DEFAULT_PG_MAJOR,
};

#[derive(Args, Debug)]
pub struct PgUpgradeSmokeArgs {
    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Explicit PostgreSQL bin directory. Defaults to the newest matching pgrx install.
    #[arg(long)]
    pgbin: Option<PathBuf>,

    /// Store smoke and PostgreSQL logs in a review packet artifact directory.
    #[arg(long)]
    artifact_dir: Option<PathBuf>,

    /// Run directory. Defaults to the script-owned target/ path.
    #[arg(long)]
    run_dir: Option<PathBuf>,

    /// Store PostgreSQL logs outside the run directory.
    #[arg(long)]
    log_dir: Option<PathBuf>,

    /// Tee smoke stdout/stderr to this file.
    #[arg(long)]
    smoke_log: Option<PathBuf>,

    /// Source PostgreSQL port.
    #[arg(long)]
    old_port: Option<u16>,

    /// Upgraded PostgreSQL port.
    #[arg(long)]
    new_port: Option<u16>,

    /// Run id used in the default run directory.
    #[arg(long)]
    run_id: Option<String>,

    /// Skip cargo pgrx install before starting fixture clusters.
    #[arg(long)]
    skip_install: bool,
}

pub async fn run(args: PgUpgradeSmokeArgs) -> Result<()> {
    if args.pg != 18 {
        bail!(
            "pg-upgrade-smoke currently requires --pg 18, got {}",
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
    let script = repo_root.join("scripts/run_pg_upgrade_smoke_pg18.sh");
    if !script.is_file() {
        bail!(
            "PG18 pg_upgrade smoke script is missing: {}",
            script.display()
        );
    }

    crate::ecaz_println!("[pg-upgrade-smoke] repo={}", repo_root.display());
    crate::ecaz_println!("[pg-upgrade-smoke] pgbin={}", pgbin.display());
    if let Some(artifact_dir) = &args.artifact_dir {
        crate::ecaz_println!("[pg-upgrade-smoke] artifact_dir={}", artifact_dir.display());
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
    push_u16_arg(&mut command, "--old-port", args.old_port);
    push_u16_arg(&mut command, "--new-port", args.new_port);
    if let Some(run_id) = args.run_id {
        command.arg("--run-id").arg(run_id);
    }
    if args.skip_install {
        command.arg("--skip-install");
    }

    run_status(command)
        .await
        .wrap_err("running PG18 pg_upgrade smoke fixture")
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
