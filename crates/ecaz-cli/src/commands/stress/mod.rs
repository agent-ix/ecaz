//! `ecaz stress` — correctness-under-load harnesses.

use clap::Subcommand;
use color_eyre::eyre::Result;

mod ivf_insert;
mod ivf_vacuum_scale;
mod vacuum;

pub use ivf_insert::IvfInsertArgs;
pub use ivf_vacuum_scale::IvfVacuumScaleArgs;
pub use vacuum::VacuumArgs;

#[derive(Subcommand, Debug)]
pub enum StressCommand {
    /// IVF live-insert throughput under concurrent worker connections.
    IvfInsert(IvfInsertArgs),
    /// IVF VACUUM scale harness for wall time, index size, and backend RSS.
    IvfVacuumScale(IvfVacuumScaleArgs),
    /// Vacuum concurrency: drive concurrent inserts/deletes/scans + VACUUM
    /// against an ec_hnsw index and assert structural invariants hold.
    Vacuum(VacuumArgs),
}

impl StressCommand {
    pub async fn run(self, database: &str) -> Result<()> {
        match self {
            StressCommand::IvfInsert(a) => ivf_insert::run(database, a).await,
            StressCommand::IvfVacuumScale(a) => ivf_vacuum_scale::run(database, a).await,
            StressCommand::Vacuum(a) => vacuum::run(database, a).await,
        }
    }
}
