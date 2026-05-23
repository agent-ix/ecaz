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
mod cross_am;
mod graph;
pub mod latency;
mod overhead;
pub mod recall;
mod sidecar_rerank;
mod spire_pipeline;
mod storage;
mod suite;

pub use build_probe::BuildProbeArgs;
pub use cross_am::CrossAmArgs;
pub use graph::GraphArgs;
pub use latency::LatencyArgs;
pub use overhead::OverheadArgs;
pub use recall::RecallArgs;
pub use sidecar_rerank::SidecarRerankArgs;
#[allow(unused_imports)]
pub use spire_pipeline::{SpirePipelineArgs, SpireRemoteTupleTransportMode};
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

const EC_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS: i32 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AdaptiveNprobeBenchOptions {
    pub(crate) enabled: bool,
    pub(crate) score_gap_micros: Option<i32>,
}

pub(crate) fn validate_adaptive_nprobe_options(
    profile: &IndexProfile,
    options: AdaptiveNprobeBenchOptions,
) -> Result<()> {
    if !options.enabled && options.score_gap_micros.is_none() {
        return Ok(());
    }
    if adaptive_nprobe_gucs(profile).is_none() {
        return Err(eyre!(
            "--adaptive-nprobe is only supported with --profile ec_ivf or --profile ec_spire"
        ));
    }
    if options.score_gap_micros.is_some() && !options.enabled {
        return Err(eyre!(
            "--adaptive-nprobe-score-gap-micros requires --adaptive-nprobe"
        ));
    }
    if let Some(value) = options.score_gap_micros {
        if !(0..=EC_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS).contains(&value) {
            return Err(eyre!(
                "--adaptive-nprobe-score-gap-micros must be between 0 and {}",
                EC_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS
            ));
        }
    }
    Ok(())
}

pub(crate) async fn apply_adaptive_nprobe_options(
    client: &Client,
    profile: &IndexProfile,
    options: AdaptiveNprobeBenchOptions,
) -> Result<()> {
    if !options.enabled {
        return Ok(());
    }
    let (enabled_guc, score_gap_guc) = adaptive_nprobe_gucs(profile).ok_or_else(|| {
        eyre!("--adaptive-nprobe is only supported with --profile ec_ivf or --profile ec_spire")
    })?;
    client
        .batch_execute(&format!("SET {enabled_guc} = on"))
        .await
        .wrap_err_with(|| format!("SET {enabled_guc} = on"))?;
    if let Some(score_gap_micros) = options.score_gap_micros {
        client
            .batch_execute(&format!("SET {score_gap_guc} = {score_gap_micros}"))
            .await
            .wrap_err_with(|| format!("SET {score_gap_guc} = {score_gap_micros}"))?;
    }
    Ok(())
}

pub(crate) fn append_adaptive_nprobe_label(
    message: String,
    options: AdaptiveNprobeBenchOptions,
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

pub(crate) fn validate_ivf_scratch_soa_batch_decode(
    profile: &IndexProfile,
    enabled: bool,
) -> Result<()> {
    if enabled && profile.name != "ec_ivf" {
        return Err(eyre!(
            "--ivf-scratch-soa-batch-decode is only supported with --profile ec_ivf"
        ));
    }
    Ok(())
}

pub(crate) async fn apply_ivf_scratch_soa_batch_decode(
    client: &Client,
    profile: &IndexProfile,
    enabled: bool,
) -> Result<()> {
    if !enabled {
        return Ok(());
    }
    validate_ivf_scratch_soa_batch_decode(profile, enabled)?;
    client
        .batch_execute("SET ec_ivf.scratch_soa_batch_decode = on")
        .await
        .wrap_err("SET ec_ivf.scratch_soa_batch_decode = on")?;
    Ok(())
}

pub(crate) fn append_ivf_scratch_soa_batch_decode_label(
    message: String,
    enabled: bool,
) -> String {
    if enabled {
        format!("{message} scratch_soa=on")
    } else {
        message
    }
}

fn adaptive_nprobe_gucs(profile: &IndexProfile) -> Option<(&'static str, &'static str)> {
    match profile.name {
        "ec_ivf" => Some((
            "ec_ivf.adaptive_nprobe",
            "ec_ivf.adaptive_nprobe_score_gap_micros",
        )),
        "ec_spire" => Some((
            "ec_spire.adaptive_nprobe",
            "ec_spire.adaptive_nprobe_score_gap_micros",
        )),
        _ => None,
    }
}

#[derive(Subcommand, Debug)]
pub enum BenchCommand {
    /// Recall@k sweep: measure accuracy vs ground truth for a set of tuning points.
    Recall(RecallArgs),
    /// Cross-AM consistency: compare per-query top-k predictions across AMs.
    CrossAm(CrossAmArgs),
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
    /// IVF/RaBitQ sidecar upper-bound rerank measurement.
    SidecarRerank(SidecarRerankArgs),
    /// SPIRE routing, local pipeline, and optional remote fanout counters.
    SpirePipeline(SpirePipelineArgs),
    /// Expand a configured benchmark suite into packet-style ecaz commands.
    Suite(SuiteArgs),
}

impl BenchCommand {
    pub async fn run(self, conn: &ConnectionOptions) -> Result<()> {
        match self {
            BenchCommand::Recall(a) => recall::run(conn, a).await,
            BenchCommand::CrossAm(a) => cross_am::run(a).await,
            BenchCommand::Latency(a) => latency::run(conn, a).await,
            BenchCommand::Storage(a) => storage::run(conn, a).await,
            BenchCommand::DiskannGraph(a) => graph::run(conn, a).await,
            BenchCommand::DiskannBuildProbe(a) => build_probe::run(conn, a).await,
            BenchCommand::Overhead(a) => overhead::run(conn, a).await,
            BenchCommand::SidecarRerank(a) => sidecar_rerank::run(conn, a).await,
            BenchCommand::SpirePipeline(a) => spire_pipeline::run(conn, a).await,
            BenchCommand::Suite(a) => suite::run(conn, a).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{EC_DISKANN, EC_HNSW, EC_IVF, EC_SPIRE};

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
    fn adaptive_nprobe_bench_options_support_ivf_and_spire() {
        assert!(validate_adaptive_nprobe_options(
            &EC_SPIRE,
            AdaptiveNprobeBenchOptions {
                enabled: true,
                score_gap_micros: Some(0),
            },
        )
        .is_ok());
        assert!(validate_adaptive_nprobe_options(
            &EC_IVF,
            AdaptiveNprobeBenchOptions {
                enabled: true,
                score_gap_micros: Some(0),
            },
        )
        .is_ok());
        assert!(validate_adaptive_nprobe_options(
            &EC_HNSW,
            AdaptiveNprobeBenchOptions {
                enabled: true,
                score_gap_micros: None,
            },
        )
        .unwrap_err()
        .to_string()
        .contains("--profile ec_ivf or --profile ec_spire"));
    }

    #[test]
    fn adaptive_nprobe_threshold_requires_enabled_switch() {
        assert!(validate_adaptive_nprobe_options(
            &EC_SPIRE,
            AdaptiveNprobeBenchOptions {
                enabled: false,
                score_gap_micros: Some(0),
            },
        )
        .unwrap_err()
        .to_string()
        .contains("requires --adaptive-nprobe"));
    }
}
