//! ecaz CLI entry point.
//!
//! See the crate-level README for the full command surface. This file
//! stays thin: clap parsing, top-level routing, error reporting.

use clap::Parser;
use color_eyre::eyre::Result;
use std::process::ExitCode;

mod cli;
mod commands;
mod manifest;
mod output;
mod profiles;
mod psql;
mod reloptions;
mod tsv;

#[tokio::main]
async fn main() -> ExitCode {
    match try_main().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            crate::ecaz_eprintln!("{err:?}");
            ExitCode::FAILURE
        }
    }
}

async fn try_main() -> Result<()> {
    color_eyre::install()?;
    let cli = cli::Cli::parse();
    output::init(cli.log_file.as_deref())?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(output::StderrMirror)
        .compact()
        .init();
    cli.run().await
}
