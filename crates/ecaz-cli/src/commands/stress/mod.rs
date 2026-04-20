//! `ecaz stress` — correctness-under-load harnesses.
//!
//! v1 status: command tree declared; implementations land in v2.

use clap::{Args, Subcommand};
use color_eyre::eyre::{eyre, Result};

#[derive(Subcommand, Debug)]
pub enum StressCommand {
    /// Vacuum concurrency: drive inserts/deletes while VACUUM runs and assert
    /// invariants hold across the index (ports `vacuum_concurrency_scratch.sh`).
    Vacuum(VacuumArgs),
}

#[derive(Args, Debug)]
pub struct VacuumArgs {
    #[arg(long)]
    pub prefix: String,
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,
    #[arg(long, default_value_t = 60)]
    pub duration_seconds: u64,
}

impl StressCommand {
    pub async fn run(self, _database: &str) -> Result<()> {
        Err(eyre!(
            "ecaz stress vacuum: not yet implemented (ported in a v2 PR)"
        ))
    }
}
