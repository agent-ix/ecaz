//! `ecaz stress` — correctness-under-load harnesses.

use clap::Subcommand;
use color_eyre::eyre::Result;

mod ivf_insert;
mod vacuum;

pub use ivf_insert::IvfInsertArgs;
pub use vacuum::VacuumArgs;

#[derive(Subcommand, Debug)]
pub enum StressCommand {
    /// IVF live-insert throughput under concurrent worker connections.
    IvfInsert(IvfInsertArgs),
    /// Vacuum concurrency: drive concurrent inserts/deletes/scans + VACUUM
    /// against an ec_hnsw index and assert structural invariants hold.
    Vacuum(VacuumArgs),
}

impl StressCommand {
    pub async fn run(self, database: &str) -> Result<()> {
        match self {
            StressCommand::IvfInsert(a) => ivf_insert::run(database, a).await,
            StressCommand::Vacuum(a) => vacuum::run(database, a).await,
        }
    }
}
