//! `ecaz stress` — correctness-under-load harnesses.

use clap::Subcommand;
use color_eyre::eyre::Result;

use crate::psql::ConnectionOptions;

mod ivf_insert;
mod ivf_vacuum_scale;
mod soak_quant_cache;
mod vacuum;

pub use ivf_insert::IvfInsertArgs;
pub use ivf_vacuum_scale::IvfVacuumScaleArgs;
pub use soak_quant_cache::SoakQuantCacheArgs;
pub use vacuum::VacuumArgs;

#[derive(Subcommand, Debug)]
pub enum StressCommand {
    /// IVF live-insert throughput under concurrent worker connections.
    IvfInsert(IvfInsertArgs),
    /// IVF VACUUM scale harness for wall time, index size, and backend RSS.
    IvfVacuumScale(IvfVacuumScaleArgs),
    /// Soak the `ProdQuantizer::cached` `OnceLock<Mutex<HashMap>>` cache
    /// under sustained concurrent contention (Task 48 kickoff harness).
    /// No PostgreSQL connection required.
    SoakQuantCache(SoakQuantCacheArgs),
    /// Vacuum concurrency: drive concurrent inserts/deletes/scans + VACUUM
    /// against an ec_hnsw index and assert structural invariants hold.
    Vacuum(VacuumArgs),
}

impl StressCommand {
    pub async fn run(self, conn: &ConnectionOptions) -> Result<()> {
        match self {
            StressCommand::IvfInsert(a) => ivf_insert::run(&conn.database, a).await,
            StressCommand::IvfVacuumScale(a) => ivf_vacuum_scale::run(&conn.database, a).await,
            StressCommand::SoakQuantCache(a) => soak_quant_cache::run(a),
            StressCommand::Vacuum(a) => vacuum::run(conn, a).await,
        }
    }
}
