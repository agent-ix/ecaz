//! `ecaz dev` — setup, scratch-cluster, and validation helpers.
//!
//! This owns the old wrapper-script surface so operators get one coherent
//! CLI for local installs, scratch clusters, and extension validation.

use clap::Subcommand;
use color_eyre::eyre::Result;

use crate::psql::ConnectionOptions;

mod install;
mod scratch;
mod spire_multicluster;
mod sql;
mod support;
mod test;

#[derive(Subcommand, Debug)]
pub enum DevCommand {
    /// Local install/setup helpers for ecaz and pgvector development.
    Install {
        #[command(subcommand)]
        command: install::InstallCommand,
    },
    /// Scratch-cluster lifecycle and query helpers.
    Scratch {
        #[command(subcommand)]
        command: scratch::ScratchCommand,
    },
    /// SPIRE local multi-cluster fixture helpers.
    SpireMulticluster {
        #[command(subcommand)]
        command: spire_multicluster::SpireMulticlusterCommand,
    },
    /// Run SQL against local pgrx PostgreSQL or a global connection target.
    Sql(sql::SqlArgs),
    /// Validation/test entry points.
    Test {
        #[command(subcommand)]
        command: test::TestCommand,
    },
}

impl DevCommand {
    pub async fn run(self, conn: &ConnectionOptions) -> Result<()> {
        match self {
            DevCommand::Install { command } => command.run(&conn.database).await,
            DevCommand::Scratch { command } => command.run(&conn.database).await,
            DevCommand::SpireMulticluster { command } => command.run(&conn.database).await,
            DevCommand::Sql(args) => sql::run(conn, args).await,
            DevCommand::Test { command } => command.run(&conn.database).await,
        }
    }
}
