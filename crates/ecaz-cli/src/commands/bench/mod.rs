//! `ecaz bench` — measurements against a loaded corpus.
//!
//! All subcommands accept `--profile` and `--prefix` so a single corpus
//! can be measured against multiple access methods without re-loading.

use clap::Subcommand;
use color_eyre::eyre::{eyre, Context, Result};
use tokio_postgres::Client;

use crate::profiles::IndexProfile;
use crate::psql::ConnectionOptions;

mod build_probe;
mod graph;
pub mod latency;
mod overhead;
pub mod recall;
mod storage;
mod suite;

pub use build_probe::BuildProbeArgs;
pub use graph::GraphArgs;
pub use latency::LatencyArgs;
pub use overhead::OverheadArgs;
pub use recall::RecallArgs;
pub use storage::StorageArgs;
pub use suite::SuiteArgs;

pub(crate) fn missing_am_error(profile: &IndexProfile, am: &str) -> String {
    format!(
        "no {am} index found for profile {:?}; build one first with `ecaz corpus load --profile {} ...`",
        profile.name, profile.name
    )
}

pub(crate) fn sweep_value_label(profile: &IndexProfile, value: i32) -> String {
    format!("{}={value}", profile.sweep_axis_label())
}

const EC_SPIRE_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS: i32 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireAdaptiveNprobeBenchOptions {
    pub(crate) enabled: bool,
    pub(crate) score_gap_micros: Option<i32>,
}

pub(crate) fn validate_spire_adaptive_nprobe_options(
    profile: &IndexProfile,
    options: SpireAdaptiveNprobeBenchOptions,
) -> Result<()> {
    if !options.enabled && options.score_gap_micros.is_none() {
        return Ok(());
    }
    if profile.name != "ec_spire" {
        return Err(eyre!(
            "--adaptive-nprobe is only supported with --profile ec_spire"
        ));
    }
    if options.score_gap_micros.is_some() && !options.enabled {
        return Err(eyre!(
            "--adaptive-nprobe-score-gap-micros requires --adaptive-nprobe"
        ));
    }
    if let Some(value) = options.score_gap_micros {
        if !(0..=EC_SPIRE_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS).contains(&value) {
            return Err(eyre!(
                "--adaptive-nprobe-score-gap-micros must be between 0 and {}",
                EC_SPIRE_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS
            ));
        }
    }
    Ok(())
}

pub(crate) async fn apply_spire_adaptive_nprobe_options(
    client: &Client,
    options: SpireAdaptiveNprobeBenchOptions,
) -> Result<()> {
    if !options.enabled {
        return Ok(());
    }
    client
        .batch_execute("SET ec_spire.adaptive_nprobe = on")
        .await
        .wrap_err("SET ec_spire.adaptive_nprobe = on")?;
    if let Some(score_gap_micros) = options.score_gap_micros {
        client
            .batch_execute(&format!(
                "SET ec_spire.adaptive_nprobe_score_gap_micros = {score_gap_micros}"
            ))
            .await
            .wrap_err_with(|| {
                format!("SET ec_spire.adaptive_nprobe_score_gap_micros = {score_gap_micros}")
            })?;
    }
    Ok(())
}

pub(crate) fn append_adaptive_nprobe_label(
    message: String,
    options: SpireAdaptiveNprobeBenchOptions,
) -> String {
    if !options.enabled {
        return message;
    }
    match options.score_gap_micros {
        Some(score_gap_micros) => {
            format!("{message} adaptive_nprobe=on gap_micros={score_gap_micros}")
        }
        None => format!("{message} adaptive_nprobe=on"),
    }
}

#[derive(Subcommand, Debug)]
pub enum BenchCommand {
    /// Recall@k sweep: measure accuracy vs ground truth for a set of tuning points.
    Recall(RecallArgs),
    /// End-to-end SQL latency at k: wall-clock p50/p95/p99 under configurable concurrency.
    Latency(LatencyArgs),
    /// Storage accounting: corpus table size, per-index size, per-vector datum size.
    Storage(StorageArgs),
    /// DiskANN persisted graph diagnostics: reachability, degree, and edge counters.
    DiskannGraph(GraphArgs),
    /// DiskANN in-memory build diagnostics: candidate pools, pruning, and degree shape.
    DiskannBuildProbe(BuildProbeArgs),
    /// Latency overhead breakdown: encode vs internal scan vs residual client/protocol.
    Overhead(OverheadArgs),
    /// Expand a configured benchmark suite into packet-style ecaz commands.
    Suite(SuiteArgs),
}

impl BenchCommand {
    pub async fn run(self, conn: &ConnectionOptions) -> Result<()> {
        match self {
            BenchCommand::Recall(a) => recall::run(conn, a).await,
            BenchCommand::Latency(a) => latency::run(conn, a).await,
            BenchCommand::Storage(a) => storage::run(conn, a).await,
            BenchCommand::DiskannGraph(a) => graph::run(conn, a).await,
            BenchCommand::DiskannBuildProbe(a) => build_probe::run(conn, a).await,
            BenchCommand::Overhead(a) => overhead::run(conn, a).await,
            BenchCommand::Suite(a) => suite::run(conn, a).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{EC_DISKANN, EC_HNSW, EC_SPIRE};

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

    #[test]
    fn adaptive_nprobe_bench_options_are_spire_only() {
        assert!(validate_spire_adaptive_nprobe_options(
            &EC_SPIRE,
            SpireAdaptiveNprobeBenchOptions {
                enabled: true,
                score_gap_micros: Some(0),
            },
        )
        .is_ok());
        assert!(validate_spire_adaptive_nprobe_options(
            &EC_HNSW,
            SpireAdaptiveNprobeBenchOptions {
                enabled: true,
                score_gap_micros: None,
            },
        )
        .unwrap_err()
        .to_string()
        .contains("--profile ec_spire"));
    }

    #[test]
    fn adaptive_nprobe_threshold_requires_enabled_switch() {
        assert!(validate_spire_adaptive_nprobe_options(
            &EC_SPIRE,
            SpireAdaptiveNprobeBenchOptions {
                enabled: false,
                score_gap_micros: Some(0),
            },
        )
        .unwrap_err()
        .to_string()
        .contains("requires --adaptive-nprobe"));
    }
}
