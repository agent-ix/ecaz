//! `ecaz bench` — measurements against a loaded corpus.
//!
//! All subcommands here accept `--profile` and `--prefix` so a single corpus
//! can be measured against multiple access methods without re-loading data.
//!
//! v1 status: command tree is declared so the shape is visible; individual
//! bench implementations land in v2 PRs (see crates/ecaz-cli/README.md).

use clap::{Args, Subcommand};
use color_eyre::eyre::{eyre, Result};

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
pub struct RecallArgs {
    #[arg(long)]
    pub prefix: String,
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,
    /// k for recall@k.
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    /// Sweep values for the profile's tuning GUC (e.g. `--sweep 100,200,400`).
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
}

#[derive(Args, Debug)]
pub struct LatencyArgs {
    #[arg(long)]
    pub prefix: String,
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    #[arg(long, default_value_t = 1)]
    pub concurrency: usize,
    #[arg(long, default_value_t = 1000)]
    pub iterations: usize,
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
    pub async fn run(self, _database: &str) -> Result<()> {
        Err(eyre!(
            "ecaz bench {}: not yet implemented (ported in a v2 PR; see crates/ecaz-cli/README.md)",
            match self {
                BenchCommand::Recall(_) => "recall",
                BenchCommand::Latency(_) => "latency",
                BenchCommand::Storage(_) => "storage",
                BenchCommand::Overhead(_) => "overhead",
            }
        ))
    }
}
