use clap::Args;
use color_eyre::eyre::{bail, Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use super::support::{find_pgrx_install, resolve_pgrx_home, run_status, scratch_socket_dir};

#[derive(Args, Debug)]
pub struct SqlArgs {
    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = 18)]
    pg: u16,

    /// Target database. Defaults to the global `--database`.
    #[arg(long)]
    db: Option<String>,

    /// Explicit socket directory or host.
    #[arg(long)]
    socket_dir: Option<PathBuf>,

    /// PostgreSQL port. Defaults to the pgrx convention, e.g. 28818 for PG18.
    #[arg(long)]
    port: Option<u16>,

    /// Emit raw psql output instead of aligned-off, tuples-only TSV.
    #[arg(long)]
    raw: bool,

    /// SQL to run.
    #[arg(long)]
    sql: Option<String>,

    /// SQL file to run.
    #[arg(long)]
    file: Option<PathBuf>,

    /// Write combined stdout/stderr to this file while also echoing it.
    #[arg(long)]
    log_output: Option<PathBuf>,

    /// Extra environment assignment for psql. Repeatable `NAME=VALUE`.
    #[arg(long = "env")]
    env: Vec<String>,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,
}

pub async fn run(database: &str, args: SqlArgs) -> Result<()> {
    if args.sql.is_some() && args.file.is_some() {
        bail!("--sql and --file are mutually exclusive");
    }
    if args.sql.is_none() && args.file.is_none() {
        bail!("one of --sql or --file is required");
    }

    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let install = find_pgrx_install(args.pg, &pgrx_home)?;
    let port = args.port.unwrap_or_else(|| default_pgrx_port(args.pg));
    let socket_dir = scratch_socket_dir(args.socket_dir.as_ref(), &pgrx_home, port)?;
    let mut command = Command::new(install.bin_dir.join("psql"));
    command
        .arg("-h")
        .arg(socket_dir)
        .arg("-p")
        .arg(port.to_string())
        .arg("-d")
        .arg(args.db.unwrap_or_else(|| database.to_string()))
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .stdin(Stdio::inherit());

    if !args.raw {
        command.arg("-A").arg("-t").arg("-F").arg("\t");
    }
    if let Some(sql) = args.sql {
        command.arg("-c").arg(sql);
    } else if let Some(file) = args.file {
        command.arg("-f").arg(file);
    }
    for assignment in &args.env {
        let (name, value) = parse_env_assignment(assignment)?;
        command.env(name, value);
    }

    if let Some(log_output) = args.log_output {
        run_logged(command, log_output).await
    } else {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        run_status(command).await
    }
}

fn default_pgrx_port(pg: u16) -> u16 {
    28800_u16
        .checked_add(pg)
        .expect("supported PostgreSQL major versions fit in a pgrx dev port")
}

fn parse_env_assignment(assignment: &str) -> Result<(&str, &str)> {
    let (name, value) = assignment.split_once('=').ok_or_else(|| {
        color_eyre::eyre::eyre!("--env values must be NAME=VALUE, got: {assignment}")
    })?;
    if name.is_empty() {
        bail!("--env values must include a variable name");
    }
    Ok((name, value))
}

async fn run_logged(mut command: Command, log_output: PathBuf) -> Result<()> {
    let debug = format!("{command:?}");
    let output = command
        .output()
        .await
        .wrap_err_with(|| format!("running {debug}"))?;

    if let Some(parent) = log_output.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    let mut combined = Vec::with_capacity(output.stdout.len() + output.stderr.len());
    combined.extend_from_slice(&output.stdout);
    combined.extend_from_slice(&output.stderr);
    tokio::fs::write(&log_output, &combined)
        .await
        .wrap_err_with(|| format!("writing {}", log_output.display()))?;

    std::io::stdout()
        .write_all(&output.stdout)
        .wrap_err("echoing psql stdout")?;
    std::io::stderr()
        .write_all(&output.stderr)
        .wrap_err("echoing psql stderr")?;

    if !output.status.success() {
        bail!("{debug} failed with status {}", output.status);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pgrx_port_matches_pgrx_convention() {
        assert_eq!(default_pgrx_port(17), 28817);
        assert_eq!(default_pgrx_port(18), 28818);
    }

    #[test]
    fn parse_env_assignment_accepts_name_value() {
        assert_eq!(parse_env_assignment("A=B").unwrap(), ("A", "B"));
        assert_eq!(parse_env_assignment("A=").unwrap(), ("A", ""));
    }

    #[test]
    fn parse_env_assignment_rejects_missing_name_or_separator() {
        assert!(parse_env_assignment("A").is_err());
        assert!(parse_env_assignment("=B").is_err());
    }
}
