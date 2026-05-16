use clap::Args;
use color_eyre::eyre::{bail, Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use crate::psql::ConnectionOptions;

use super::support::{
    default_pgrx_port, find_pgrx_install, pgrx_socket_dir, resolve_pgrx_home, run_status,
    DEFAULT_PG_MAJOR,
};

#[derive(Args, Debug)]
pub struct SqlArgs {
    /// PostgreSQL major version from the local pgrx install.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
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

    /// Extra psql variable assignment. Repeatable `NAME=VALUE`.
    #[arg(long = "set")]
    set: Vec<String>,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,
}

pub async fn run(conn: &ConnectionOptions, args: SqlArgs) -> Result<()> {
    if args.sql.is_some() && args.file.is_some() {
        bail!("--sql and --file are mutually exclusive");
    }
    if args.sql.is_none() && args.file.is_none() {
        bail!("one of --sql or --file is required");
    }

    let mut command = if conn.host.is_some()
        || conn.port.is_some()
        || conn.user.is_some()
        || conn.password.is_some()
    {
        remote_psql_command(conn, args.db.as_deref())
    } else {
        local_pgrx_psql_command(conn, &args)?
    };
    command
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .stdin(Stdio::inherit());
    for assignment in &args.set {
        parse_psql_variable_assignment(assignment)?;
        command.arg("-v").arg(assignment);
    }

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

fn remote_psql_command(conn: &ConnectionOptions, db: Option<&str>) -> Command {
    let mut command = Command::new("psql");
    if let Some(host) = conn.host.as_deref() {
        command.arg("-h").arg(host);
    }
    if let Some(port) = conn.port {
        command.arg("-p").arg(port.to_string());
    }
    if let Some(user) = conn.user.as_deref() {
        command.arg("-U").arg(user);
    }
    if let Some(password) = conn.password.as_deref() {
        command.env("PGPASSWORD", password);
    }
    command.arg("-d").arg(db.unwrap_or(&conn.database));
    command
}

fn local_pgrx_psql_command(conn: &ConnectionOptions, args: &SqlArgs) -> Result<Command> {
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let install = find_pgrx_install(args.pg, &pgrx_home)?;
    let port = args.port.unwrap_or_else(|| default_pgrx_port(args.pg));
    let socket_dir = pgrx_socket_dir(args.socket_dir.as_ref(), &pgrx_home, port)?;
    let mut command = Command::new(install.bin_dir.join("psql"));
    command
        .arg("-h")
        .arg(socket_dir)
        .arg("-p")
        .arg(port.to_string())
        .arg("-d")
        .arg(args.db.as_deref().unwrap_or(&conn.database));
    Ok(command)
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

fn parse_psql_variable_assignment(assignment: &str) -> Result<(&str, &str)> {
    let (name, value) = parse_env_assignment(assignment)?;
    if name.chars().any(char::is_whitespace) {
        bail!("--set variable names must not contain whitespace: {assignment}");
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

    #[test]
    fn parse_psql_variable_assignment_rejects_whitespace_names() {
        assert_eq!(
            parse_psql_variable_assignment("prefix=ec_spire")
                .expect("valid psql variable assignment"),
            ("prefix", "ec_spire")
        );
        assert!(parse_psql_variable_assignment("bad name=value").is_err());
    }
}
