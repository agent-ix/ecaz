//! `ecaz quant` — quantizer-level correctness and feasibility harnesses.
//!
//! Separate from `ecaz bench` because this group does not need a
//! loaded corpus in Postgres: it operates on the TSV fixture files
//! directly, and its output is *offline recall vs. exact inner
//! product* rather than index-level latency/throughput.
//!
//! Every quantizer on the roadmap — RaBitQ (task 25, ADR-045 Stage 1),
//! OPQ rotation (task 20), additive residual (task 22), LSQ codebook
//! refinement (task 23), Symphony Stage 2 (task 27) — needs the same
//! recall/bound study before it can be promoted. This module owns the
//! shared harness so each new quantizer costs only an enum variant +
//! an encoder construction line, not a new `src/bin/` clone.

use clap::Subcommand;
use color_eyre::eyre::Result;

pub mod feasibility;

pub use feasibility::FeasibilityArgs;

#[derive(Subcommand, Debug)]
pub enum QuantCommand {
    /// Recall@K of an offline quantizer vs. exact inner product,
    /// plus error-bound distribution calibration.
    Feasibility(FeasibilityArgs),
}

impl QuantCommand {
    pub async fn run(self, _database: &str) -> Result<()> {
        match self {
            QuantCommand::Feasibility(args) => feasibility::run(args).await,
        }
    }
}
