//! `ecaz bench` — measurements against a loaded corpus.
//!
//! All subcommands accept `--profile` and `--prefix` so a single corpus
//! can be measured against multiple access methods without re-loading.

use clap::Subcommand;
use color_eyre::eyre::Result;

use crate::profiles::IndexProfile;

pub mod latency;
mod overhead;
pub mod recall;
mod storage;

pub use latency::LatencyArgs;
pub use overhead::OverheadArgs;
pub use recall::RecallArgs;
pub use storage::StorageArgs;

pub(crate) fn missing_am_error(profile: &IndexProfile, am: &str) -> String {
    format!(
        "no {am} index found for profile {:?}; build one first with `ecaz corpus load --profile {} ...`",
        profile.name, profile.name
    )
}

pub(crate) fn sweep_value_label(profile: &IndexProfile, value: i32) -> String {
    format!("{}={value}", profile.sweep_axis_label())
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{EC_DISKANN, EC_HNSW};

    #[test]
    fn missing_am_error_points_operator_at_matching_profile_load() {
        assert_eq!(
            missing_am_error(&EC_DISKANN, "ec_diskann"),
            "no ec_diskann index found for profile \"ec_diskann\"; build one first with `ecaz corpus load --profile ec_diskann ...`"
        );
    }

    #[test]
    fn missing_am_error_preserves_explicit_am_argument() {
        assert_eq!(
            missing_am_error(&EC_HNSW, "custom_am"),
            "no custom_am index found for profile \"ec_hnsw\"; build one first with `ecaz corpus load --profile ec_hnsw ...`"
        );
    }

    #[test]
    fn sweep_value_label_uses_profile_axis_name() {
        assert_eq!(sweep_value_label(&EC_HNSW, 100), "ef_search=100");
        assert_eq!(sweep_value_label(&EC_DISKANN, 200), "list_size=200");
    }
}
