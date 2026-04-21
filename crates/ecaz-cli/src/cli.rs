//! Top-level clap surface.
//!
//! Subcommand groups mirror the conceptual split in the CLI README:
//! `corpus` (data in/out of Postgres), `bench` (measurements against loaded
//! corpora), `compare` (cross-engine comparison), and `stress` (correctness
//! under load). Adding a new group means adding one variant to `Command`
//! and one module under `commands/`.

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

use crate::{commands, psql};

#[derive(Parser, Debug)]
#[command(
    name = "ecaz",
    version,
    about = "Operator CLI for the Ecaz Postgres extension",
    long_about = "ecaz — corpus loading, benchmarking (recall / latency / storage), \
                  and cross-engine comparison for the Ecaz Postgres vector extension. \
                  Access methods (ec_hnsw, ec_ivf, ec_diskann) are selected via \
                  `--profile`; every command is profile-aware."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

    /// PostgreSQL database name. Defaults to $PGDATABASE or 'tqvector_bench'.
    #[arg(
        long,
        global = true,
        env = "PGDATABASE",
        default_value = "tqvector_bench"
    )]
    pub database: String,

    /// PostgreSQL host name or Unix socket directory.
    #[arg(long, global = true, env = "PGHOST")]
    pub host: Option<String>,

    /// PostgreSQL port.
    #[arg(long, global = true, env = "PGPORT")]
    pub port: Option<u16>,

    /// PostgreSQL user name.
    #[arg(long, global = true, env = "PGUSER")]
    pub user: Option<String>,

    /// PostgreSQL password. Prefer `.pgpass` for non-local use.
    #[arg(long, global = true, env = "PGPASSWORD", hide_env_values = true)]
    pub password: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Corpus plumbing: load fixtures, inspect what's loaded, verify manifests.
    Corpus {
        #[command(subcommand)]
        command: commands::corpus::CorpusCommand,
    },
    /// Benchmarks against a loaded corpus (recall, latency, storage, ...).
    Bench {
        #[command(subcommand)]
        command: commands::bench::BenchCommand,
    },
    /// Compare Ecaz against external vector-search engines on the same corpus.
    Compare {
        #[command(subcommand)]
        command: commands::compare::CompareCommand,
    },
    /// Development/setup/test helpers that own the old wrapper-script surface.
    Dev {
        #[command(subcommand)]
        command: commands::dev::DevCommand,
    },
    /// Offline quantizer feasibility / recall studies (no DB required).
    Quant {
        #[command(subcommand)]
        command: commands::quant::QuantCommand,
    },
    /// Correctness-under-load harnesses (vacuum concurrency, crash recovery, ...).
    Stress {
        #[command(subcommand)]
        command: commands::stress::StressCommand,
    },
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let conn = psql::ConnectionOptions {
            database: self.database,
            host: self.host,
            port: self.port,
            user: self.user,
            password: self.password,
        };
        match self.command {
            Command::Corpus { command } => command.run(&conn).await,
            Command::Bench { command } => command.run(&conn).await,
            Command::Compare { command } => command.run(&conn).await,
            Command::Dev { command } => command.run(&conn.database).await,
            Command::Quant { command } => command.run(&conn.database).await,
            Command::Stress { command } => command.run(&conn).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn cli_parses_explicit_connection_overrides() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "--database",
            "bench",
            "--host",
            "/home/peter/.pgrx",
            "--port",
            "28818",
            "--user",
            "peter",
            "--password",
            "secret",
            "corpus",
            "list",
        ])
        .expect("cli parses");
        assert_eq!(cli.database, "bench");
        assert_eq!(cli.host.as_deref(), Some("/home/peter/.pgrx"));
        assert_eq!(cli.port, Some(28818));
        assert_eq!(cli.user.as_deref(), Some("peter"));
        assert_eq!(cli.password.as_deref(), Some("secret"));
    }

    #[test]
    fn cli_parses_corpus_fetch_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "--database",
            "postgres",
            "corpus",
            "fetch",
            "--dataset",
            "dbpedia-openai3-large-1536-1m",
            "--output-dir",
            "/data/real-corpus",
            "--revision",
            "main",
            "--force",
        ])
        .expect("cli parses");
        assert_eq!(cli.database, "postgres");
    }
}
