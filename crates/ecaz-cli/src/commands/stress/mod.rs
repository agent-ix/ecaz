//! `ecaz stress` — correctness-under-load harnesses.

use clap::Subcommand;
use color_eyre::eyre::Result;

mod vacuum;

pub use vacuum::VacuumArgs;

#[derive(Subcommand, Debug)]
pub enum StressCommand {
    /// Vacuum concurrency: drive concurrent inserts/deletes/scans + VACUUM
    /// against an ec_hnsw index and assert structural invariants hold.
    Vacuum(VacuumArgs),
}

impl StressCommand {
    pub async fn run(self, database: &str) -> Result<()> {
        match self {
            StressCommand::Vacuum(a) => vacuum::run(database, a).await,
        }
    }
}
