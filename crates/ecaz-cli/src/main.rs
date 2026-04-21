//! ecaz CLI entry point.
//!
//! See the crate-level README for the full command surface. This file
//! stays thin: clap parsing, top-level routing, error reporting.

use clap::Parser;
use color_eyre::eyre::Result;

mod cli;
mod commands;
mod manifest;
mod profiles;
mod psql;
mod reloptions;
mod tsv;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .compact()
        .init();

    let cli = cli::Cli::parse();
    cli.run().await
}
