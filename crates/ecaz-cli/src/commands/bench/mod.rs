//! `ecaz bench` — measurements against a loaded corpus.
//!
//! All subcommands here accept `--profile` and `--prefix` so a single corpus
//! can be measured against multiple access methods without re-loading data.
//!
//! v1 status: `recall` and `latency` are implemented. `storage` and
//! `overhead` remain stubs and land in follow-up commits.

use clap::{Args, Subcommand};
use color_eyre::eyre::{eyre, Result};

mod latency;
mod recall;

pub use latency::LatencyArgs;
pub use recall::RecallArgs;

#[derive(Subcommand, Debug)]
pub enum BenchCommand {
    /// Recall@k sweep: measure accuracy vs ground truth for a set of tuning points.
    Recall(RecallArgs),
    /// End-to-end SQL latency at k: wall-clock p50/p95/p99 under configurable concurrency.
    Latency(LatencyArgs),
    /// Storage accounting: corpus table size, per-index size, per-vector datum size.
    Storage(StorageArgs),
    /// Latency overhead breakdown: encode vs internal scan vs residual SQL time.
    Overhead(OverheadArgs),
}

#[derive(Args, Debug)]
pub struct StorageArgs {
    #[arg(long)]
    pub prefix: String,
}

#[derive(Args, Debug)]
pub struct OverheadArgs {
    #[arg(long)]
    pub prefix: String,
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,
}

impl BenchCommand {
    pub async fn run(self, database: &str) -> Result<()> {
        match self {
            BenchCommand::Recall(args) => recall::run(database, args).await,
            BenchCommand::Latency(args) => latency::run(database, args).await,
            BenchCommand::Storage(_) | BenchCommand::Overhead(_) => Err(eyre!(
                "ecaz bench {}: not yet implemented (ported in a follow-up commit)",
                match self {
                    BenchCommand::Storage(_) => "storage",
                    BenchCommand::Overhead(_) => "overhead",
                    _ => unreachable!(),
                }
            )),
        }
    }
}
