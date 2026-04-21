//! `ecaz dev` — setup, scratch-cluster, and validation helpers.
//!
//! This owns the old wrapper-script surface so operators get one coherent
//! CLI for local installs, scratch clusters, and extension validation.

use clap::Subcommand;
use color_eyre::eyre::Result;

mod install;
mod scratch;
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
    /// Validation/test entry points.
    Test {
        #[command(subcommand)]
        command: test::TestCommand,
    },
}

impl DevCommand {
    pub async fn run(self, database: &str) -> Result<()> {
        match self {
            DevCommand::Install { command } => command.run(database).await,
            DevCommand::Scratch { command } => command.run(database).await,
            DevCommand::Test { command } => command.run(database).await,
        }
    }
}
