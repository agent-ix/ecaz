//! `ecaz bench` — measurements against a loaded corpus.
//!
//! All subcommands accept `--profile` and `--prefix` so a single corpus
//! can be measured against multiple access methods without re-loading.

use clap::Subcommand;
use color_eyre::eyre::Result;

pub mod latency;
mod overhead;
pub mod recall;
mod storage;

pub use latency::LatencyArgs;
pub use overhead::OverheadArgs;
pub use recall::RecallArgs;
pub use storage::StorageArgs;

#[derive(Subcommand, Debug)]
pub enum BenchCommand {
    /// Recall@k sweep: measure accuracy vs ground truth for a set of tuning points.
    Recall(RecallArgs),
    /// End-to-end SQL latency at k: wall-clock p50/p95/p99 under configurable concurrency.
    Latency(LatencyArgs),
    /// Storage accounting: corpus table size, per-index size, per-vector datum size.
    Storage(StorageArgs),
    /// Latency overhead breakdown: encode vs internal scan vs residual client/protocol.
    Overhead(OverheadArgs),
}

impl BenchCommand {
    pub async fn run(self, database: &str) -> Result<()> {
        match self {
            BenchCommand::Recall(a) => recall::run(database, a).await,
            BenchCommand::Latency(a) => latency::run(database, a).await,
            BenchCommand::Storage(a) => storage::run(database, a).await,
            BenchCommand::Overhead(a) => overhead::run(database, a).await,
        }
    }
}
