//! Top-level clap surface.
//!
//! Subcommand groups mirror the conceptual split in the CLI README:
//! `corpus` (data in/out of Postgres), `bench` (measurements against loaded
//! corpora), `compare` (cross-engine comparison), and `stress` (correctness
//! under load). Adding a new group means adding one variant to `Command`
//! and one module under `commands/`.

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use std::path::PathBuf;

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

    /// Mirror CLI stdout/stderr into a file for packet-local artifact capture.
    /// Progress bars are suppressed so the file stays stable and diffable.
    #[arg(long, global = true)]
    pub log_file: Option<PathBuf>,
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
    /// Cloud benchmark harness: provision, install, load, bench, teardown on AWS.
    Cloud {
        #[command(subcommand)]
        command: ecaz_cloud::CloudCommand,
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
            Command::Dev { command } => command.run(&conn).await,
            Command::Quant { command } => command.run(&conn.database).await,
            Command::Stress { command } => command.run(&conn).await,
            Command::Cloud { command } => command.run(find_repo_root()?).await,
        }
    }
}

fn find_repo_root() -> Result<std::path::PathBuf> {
    let mut cwd = std::env::current_dir()?;
    loop {
        if cwd.join("infra/cloud/terraform").exists() {
            return Ok(cwd);
        }
        if !cwd.pop() {
            return Err(color_eyre::eyre::eyre!(
                "could not locate ecaz repo root (no infra/cloud/terraform/ on the way up)"
            ));
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
    fn cli_parses_log_file_override() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "--database",
            "postgres",
            "--log-file",
            "review/11074-task17-ecaz-log-file/artifacts/load.log",
            "corpus",
            "list",
        ])
        .expect("cli parses");
        assert_eq!(
            cli.log_file.as_deref(),
            Some(std::path::Path::new(
                "review/11074-task17-ecaz-log-file/artifacts/load.log"
            ))
        );
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

    #[test]
    fn cli_parses_spire_multicluster_transport_overlap_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "transport-overlap-pg18",
            "--artifact-dir",
            "review/30776-spire-cli-multicluster-transport/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_smoke_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "smoke-pg18",
            "--artifact-dir",
            "review/30968-spire-multicluster-cli-smoke-insert/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_insert_read_after_customscan_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "insert-read-after-customscan-pg18",
            "--artifact-dir",
            "review/30968-spire-multicluster-cli-smoke-insert/artifacts",
            "--insert-mode",
            "trigger",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_pipeline_remote_tuple_transport() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "bench",
            "spire-pipeline",
            "--prefix",
            "bench_pfx",
            "--sweep",
            "8,16",
            "--remote-tuple-transport",
            "pg_binary_attr_v1",
            "--include-local-store-overlap",
            "--include-query-metrics",
            "--include-recall",
            "--query-metric-k",
            "20",
            "--include-cost-snapshot",
            "--cost-routing-dimension-scale",
            "0.02",
            "--cost-rerank-multiplier",
            "2.0",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Bench {
                command: crate::commands::bench::BenchCommand::SpirePipeline(args),
            } => {
                assert_eq!(
                    args.remote_tuple_transport,
                    Some(crate::commands::bench::SpireRemoteTupleTransportMode::PgBinaryAttrV1)
                );
                assert!(args.include_local_store_overlap);
                assert!(args.include_query_metrics);
                assert!(args.include_recall);
                assert!(args.include_cost_snapshot);
                assert_eq!(args.query_metric_k, 20);
                assert_eq!(args.cost_routing_dimension_scale, Some(0.02));
                assert_eq!(args.cost_rerank_multiplier, Some(2.0));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_customscan_read_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "customscan-read-pg18",
            "--artifact-dir",
            "review/30962-spire-customscan-read-cli/artifacts",
            "--remote-port",
            "39228",
            "--coord-port",
            "39229",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_fault_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "fault-pg18",
            "--case",
            "simulated_network_partition",
            "--artifact-dir",
            "review/30778-spire-stage-e-network-partition/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_version_skew_fault_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "fault-pg18",
            "--case",
            "version_skew",
            "--artifact-dir",
            "review/30779-spire-stage-e-version-skew/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_epoch_mismatch_fault_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "fault-pg18",
            "--case",
            "epoch_mismatch",
            "--artifact-dir",
            "review/30780-spire-stage-e-epoch-mismatch/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_missing_remote_index_fault_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "fault-pg18",
            "--case",
            "missing_or_reindexed_remote_index",
            "--artifact-dir",
            "review/30781-spire-stage-e-missing-remote-index/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_fingerprint_mismatch_fault_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "fault-pg18",
            "--case",
            "fingerprint_mismatch",
            "--artifact-dir",
            "review/30782-spire-stage-e-fingerprint-mismatch/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_remote_statement_timeout_fault_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "fault-pg18",
            "--case",
            "remote_statement_timeout",
            "--artifact-dir",
            "review/30783-spire-stage-e-remote-statement-timeout/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_remote_backend_termination_fault_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "fault-pg18",
            "--case",
            "remote_backend_termination",
            "--artifact-dir",
            "review/30784-spire-stage-e-remote-backend-termination/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_local_cancel_fault_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "fault-pg18",
            "--case",
            "local_cancel",
            "--artifact-dir",
            "review/30785-spire-stage-e-local-cancel/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_local_statement_timeout_fault_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "fault-pg18",
            "--case",
            "local_statement_timeout",
            "--artifact-dir",
            "review/30786-spire-stage-e-local-statement-timeout/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_connection_reset_mid_batch_fault_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "fault-pg18",
            "--case",
            "connection_reset_mid_batch",
            "--artifact-dir",
            "review/30787-spire-stage-e-connection-reset-mid-batch/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_remote_oom_fault_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "fault-pg18",
            "--case",
            "remote_oom",
            "--artifact-dir",
            "review/30788-spire-stage-e-remote-oom/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_lifecycle_drop_before_fanout_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "lifecycle-pg18",
            "--case",
            "drop_remote_index_before_fanout",
            "--artifact-dir",
            "review/30789-spire-stage-e-lifecycle-drop-before-fanout/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_lifecycle_drop_in_flight_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "lifecycle-pg18",
            "--case",
            "drop_remote_index_in_flight",
            "--artifact-dir",
            "review/30790-spire-stage-e-lifecycle-drop-in-flight/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_lifecycle_reindex_before_fanout_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "lifecycle-pg18",
            "--case",
            "reindex_remote_index_before_fanout",
            "--artifact-dir",
            "review/30791-spire-stage-e-lifecycle-reindex-before-fanout/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_lifecycle_reindex_in_flight_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "lifecycle-pg18",
            "--case",
            "reindex_remote_index_in_flight",
            "--artifact-dir",
            "review/30792-spire-stage-e-lifecycle-reindex-in-flight/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_lifecycle_create_missing_descriptor_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "lifecycle-pg18",
            "--case",
            "create_index_concurrently_missing_descriptor",
            "--artifact-dir",
            "review/30793-spire-stage-e-lifecycle-create-missing-descriptor/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_spire_multicluster_lifecycle_create_new_descriptor_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "spire-multicluster",
            "lifecycle-pg18",
            "--case",
            "create_index_concurrently_new_descriptor",
            "--artifact-dir",
            "review/30795-spire-stage-e-lifecycle-create-new-descriptor/artifacts",
            "--run-id",
            "parse-test",
            "--skip-install",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::SpireMulticluster { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_fault_smoke_dry_run_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "fault",
            "smoke",
            "--lane",
            "timeout",
            "--dry-run",
            "--provider-marker",
            "/tmp/ecaz-fault-provider.marker",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::Fault { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_fault_provider_env_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "fault",
            "provider-env",
            "--mode",
            "slow-disk",
            "--latency-ms",
            "25",
            "--path-match",
            "base/",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::Fault { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_fault_provider_restart_command() {
        let cli = Cli::try_parse_from([
            "ecaz",
            "dev",
            "fault",
            "provider-restart",
            "--mode",
            "slow-disk",
            "--latency-ms",
            "1",
            "--marker",
            "/tmp/ecaz-fault-provider.marker",
        ])
        .expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::Fault { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_fault_provider_restore_command() {
        let cli =
            Cli::try_parse_from(["ecaz", "dev", "fault", "provider-restore"]).expect("cli parses");
        match cli.command {
            super::Command::Dev {
                command: crate::commands::dev::DevCommand::Fault { command: _command },
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
