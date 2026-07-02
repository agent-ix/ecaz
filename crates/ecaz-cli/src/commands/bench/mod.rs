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
mod comparator;
mod cross_am;
mod graph;
pub mod latency;
mod overhead;
mod rabitq_kernel;
pub mod recall;
mod sidecar_rerank;
mod spire_pipeline;
mod storage;
mod suite;

pub use build_probe::BuildProbeArgs;
pub use comparator::ComparatorArgs;
pub use cross_am::CrossAmArgs;
pub use graph::GraphArgs;
pub use latency::LatencyArgs;
pub use overhead::OverheadArgs;
pub use rabitq_kernel::RabitqKernelArgs;
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
const EC_MAX_ADAPTIVE_NPROBE_SCORE_MARGIN_RATIO_BPS: i32 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AdaptiveNprobeBenchOptions {
    pub(crate) enabled: bool,
    pub(crate) score_gap_micros: Option<i32>,
    pub(crate) score_margin_ratio_bps: Option<i32>,
}

pub(crate) fn validate_adaptive_nprobe_options(
    profile: &IndexProfile,
    options: AdaptiveNprobeBenchOptions,
) -> Result<()> {
    if !options.enabled
        && options.score_gap_micros.is_none()
        && options.score_margin_ratio_bps.is_none()
    {
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
    if options.score_margin_ratio_bps.is_some() && !options.enabled {
        return Err(eyre!(
            "--adaptive-nprobe-score-margin-ratio-bps requires --adaptive-nprobe"
        ));
    }
    if options.score_margin_ratio_bps.is_some() && adaptive_nprobe_ratio_guc(profile).is_none() {
        return Err(eyre!(
            "--adaptive-nprobe-score-margin-ratio-bps is only supported with --profile ec_ivf"
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
    if let Some(value) = options.score_margin_ratio_bps {
        if !(0..=EC_MAX_ADAPTIVE_NPROBE_SCORE_MARGIN_RATIO_BPS).contains(&value) {
            return Err(eyre!(
                "--adaptive-nprobe-score-margin-ratio-bps must be between 0 and {}",
                EC_MAX_ADAPTIVE_NPROBE_SCORE_MARGIN_RATIO_BPS
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
    if let Some(score_margin_ratio_bps) = options.score_margin_ratio_bps {
        let ratio_guc = adaptive_nprobe_ratio_guc(profile).ok_or_else(|| {
            eyre!(
                "--adaptive-nprobe-score-margin-ratio-bps is only supported with --profile ec_ivf"
            )
        })?;
        client
            .batch_execute(&format!("SET {ratio_guc} = {score_margin_ratio_bps}"))
            .await
            .wrap_err_with(|| format!("SET {ratio_guc} = {score_margin_ratio_bps}"))?;
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
    match (options.score_gap_micros, options.score_margin_ratio_bps) {
        (_, Some(score_margin_ratio_bps)) => {
            format!("{message} adaptive_nprobe=on margin_ratio_bps={score_margin_ratio_bps}")
        }
        (Some(score_gap_micros), None) => {
            format!("{message} adaptive_nprobe=on gap_micros={score_gap_micros}")
        }
        (None, None) => format!("{message} adaptive_nprobe=on"),
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

fn adaptive_nprobe_ratio_guc(profile: &IndexProfile) -> Option<&'static str> {
    match profile.name {
        "ec_ivf" => Some("ec_ivf.adaptive_nprobe_score_margin_ratio_bps"),
        _ => None,
    }
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

pub(crate) fn append_ivf_scratch_soa_batch_decode_label(message: String, enabled: bool) -> String {
    if enabled {
        format!("{message} scratch_soa=on")
    } else {
        message
    }
}

pub(crate) fn parse_session_gucs(raw: &[String]) -> Result<Vec<(String, String)>> {
    raw.iter()
        .map(|entry| {
            let (name, value) = entry
                .split_once('=')
                .ok_or_else(|| eyre!("--session-guc must use name=value syntax, got {entry:?}"))?;
            validate_guc_name(name)
                .wrap_err_with(|| format!("invalid --session-guc name {name:?}"))?;
            if value.trim().is_empty() {
                return Err(eyre!("--session-guc value must not be empty for {name:?}"));
            }
            Ok((name.to_owned(), value.to_owned()))
        })
        .collect()
}

pub(crate) async fn apply_session_gucs(
    client: &Client,
    session_gucs: &[(String, String)],
) -> Result<()> {
    for (name, value) in session_gucs {
        client
            .batch_execute(&format!("SET {name} = {value}"))
            .await
            .wrap_err_with(|| format!("SET {name} = {value}"))?;
    }
    Ok(())
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Task87CandidateBatchCounterSnapshot {
    pub(crate) surface: String,
    pub(crate) flushes: i64,
    pub(crate) candidates: i64,
    pub(crate) elapsed_nanos: i64,
    pub(crate) lut32_flushes: i64,
    pub(crate) lut32_candidates: i64,
    pub(crate) lut32_pruned_candidates: i64,
    pub(crate) lut32_kept_candidates: i64,
}

impl Task87CandidateBatchCounterSnapshot {
    fn merge(&mut self, other: &Self) {
        self.flushes += other.flushes;
        self.candidates += other.candidates;
        self.elapsed_nanos += other.elapsed_nanos;
        self.lut32_flushes += other.lut32_flushes;
        self.lut32_candidates += other.lut32_candidates;
        self.lut32_pruned_candidates += other.lut32_pruned_candidates;
        self.lut32_kept_candidates += other.lut32_kept_candidates;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BlockKernelCounterSnapshot {
    pub(crate) surface: String,
    pub(crate) quant_kind: String,
    pub(crate) isa: String,
    pub(crate) flushes: i64,
    pub(crate) candidates: i64,
    pub(crate) elapsed_nanos: i64,
    pub(crate) kernel_flushes: i64,
    pub(crate) kernel_candidates: i64,
    pub(crate) kernel_elapsed_nanos: i64,
    pub(crate) scalar_flushes: i64,
    pub(crate) scalar_candidates: i64,
    pub(crate) scalar_elapsed_nanos: i64,
    pub(crate) width_lt8_flushes: i64,
    pub(crate) width_8_15_flushes: i64,
    pub(crate) width_16_31_flushes: i64,
    pub(crate) width_ge32_flushes: i64,
    pub(crate) pruned_candidates: i64,
    pub(crate) kept_candidates: i64,
}

impl BlockKernelCounterSnapshot {
    fn merge(&mut self, other: &Self) {
        self.flushes += other.flushes;
        self.candidates += other.candidates;
        self.elapsed_nanos += other.elapsed_nanos;
        self.kernel_flushes += other.kernel_flushes;
        self.kernel_candidates += other.kernel_candidates;
        self.kernel_elapsed_nanos += other.kernel_elapsed_nanos;
        self.scalar_flushes += other.scalar_flushes;
        self.scalar_candidates += other.scalar_candidates;
        self.scalar_elapsed_nanos += other.scalar_elapsed_nanos;
        self.width_lt8_flushes += other.width_lt8_flushes;
        self.width_8_15_flushes += other.width_8_15_flushes;
        self.width_16_31_flushes += other.width_16_31_flushes;
        self.width_ge32_flushes += other.width_ge32_flushes;
        self.pruned_candidates += other.pruned_candidates;
        self.kept_candidates += other.kept_candidates;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BlockKernelCounterSnapshots {
    pub(crate) block_kernel: Vec<BlockKernelCounterSnapshot>,
    pub(crate) task87_compat: Vec<Task87CandidateBatchCounterSnapshot>,
}

#[allow(dead_code)]
pub(crate) async fn reset_task87_candidate_batch_counters(client: &Client) -> Result<()> {
    reset_block_kernel_counters(client).await
}

pub(crate) async fn reset_block_kernel_counters(client: &Client) -> Result<()> {
    match client
        .batch_execute("SELECT ec_block_kernel_scoring_reset()")
        .await
    {
        Ok(()) => Ok(()),
        Err(block_err) => client
            .batch_execute("SELECT ec_task87_candidate_batch_scoring_reset()")
            .await
            .wrap_err_with(|| {
                format!(
                    "resetting block-kernel counters failed ({block_err}); fallback Task 87 reset also failed"
                )
            }),
    }
}

#[allow(dead_code)]
pub(crate) async fn snapshot_task87_candidate_batch_counters(
    client: &Client,
) -> Result<Vec<Task87CandidateBatchCounterSnapshot>> {
    snapshot_task87_candidate_batch_counters_inner(client).await
}

pub(crate) async fn snapshot_block_kernel_counters(
    client: &Client,
) -> Result<BlockKernelCounterSnapshots> {
    let task87_compat = snapshot_task87_candidate_batch_counters_inner(client).await?;
    let block_kernel = match client
        .query(
            "SELECT surface, quant_kind, isa, flushes, candidates, elapsed_nanos, \
                    kernel_flushes, kernel_candidates, kernel_elapsed_nanos, \
                    scalar_flushes, scalar_candidates, scalar_elapsed_nanos, \
                    width_lt8_flushes, width_8_15_flushes, width_16_31_flushes, \
                    width_ge32_flushes, pruned_candidates, kept_candidates \
             FROM ec_block_kernel_scoring_snapshot() \
             ORDER BY surface, quant_kind, isa",
            &[],
        )
        .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|row| BlockKernelCounterSnapshot {
                surface: row.get(0),
                quant_kind: row.get(1),
                isa: row.get(2),
                flushes: row.get(3),
                candidates: row.get(4),
                elapsed_nanos: row.get(5),
                kernel_flushes: row.get(6),
                kernel_candidates: row.get(7),
                kernel_elapsed_nanos: row.get(8),
                scalar_flushes: row.get(9),
                scalar_candidates: row.get(10),
                scalar_elapsed_nanos: row.get(11),
                width_lt8_flushes: row.get(12),
                width_8_15_flushes: row.get(13),
                width_16_31_flushes: row.get(14),
                width_ge32_flushes: row.get(15),
                pruned_candidates: row.get(16),
                kept_candidates: row.get(17),
            })
            .collect(),
        Err(error) => {
            eprintln!(
                "[block-kernel-counters] snapshot query failed (stale extension catalog? \
                 recreate the bench database after counter-schema changes): {error}"
            );
            Vec::new()
        }
    };
    Ok(BlockKernelCounterSnapshots {
        block_kernel,
        task87_compat,
    })
}

async fn snapshot_task87_candidate_batch_counters_inner(
    client: &Client,
) -> Result<Vec<Task87CandidateBatchCounterSnapshot>> {
    let rows = client
        .query(
            "SELECT surface, flushes, candidates, elapsed_nanos, lut32_flushes, lut32_candidates, \
                    lut32_pruned_candidates, lut32_kept_candidates \
             FROM ec_task87_candidate_batch_scoring_snapshot() \
             ORDER BY surface",
            &[],
        )
        .await
        .wrap_err("snapshotting Task 87 CandidateBatch scoring counters")?;
    Ok(rows
        .into_iter()
        .map(|row| Task87CandidateBatchCounterSnapshot {
            surface: row.get(0),
            flushes: row.get(1),
            candidates: row.get(2),
            elapsed_nanos: row.get(3),
            lut32_flushes: row.get(4),
            lut32_candidates: row.get(5),
            lut32_pruned_candidates: row.get(6),
            lut32_kept_candidates: row.get(7),
        })
        .collect())
}

pub(crate) fn merge_block_kernel_counters(
    snapshots: Vec<BlockKernelCounterSnapshots>,
) -> BlockKernelCounterSnapshots {
    let mut block_merged =
        std::collections::BTreeMap::<(String, String, String), BlockKernelCounterSnapshot>::new();
    let mut task87_sets = Vec::with_capacity(snapshots.len());
    for snapshot_set in snapshots {
        task87_sets.push(snapshot_set.task87_compat);
        for snapshot in snapshot_set.block_kernel {
            block_merged
                .entry((
                    snapshot.surface.clone(),
                    snapshot.quant_kind.clone(),
                    snapshot.isa.clone(),
                ))
                .and_modify(|existing| existing.merge(&snapshot))
                .or_insert(snapshot);
        }
    }
    BlockKernelCounterSnapshots {
        block_kernel: block_merged.into_values().collect(),
        task87_compat: merge_task87_candidate_batch_counters(task87_sets),
    }
}

pub(crate) fn merge_task87_candidate_batch_counters(
    snapshots: Vec<Vec<Task87CandidateBatchCounterSnapshot>>,
) -> Vec<Task87CandidateBatchCounterSnapshot> {
    let mut merged =
        std::collections::BTreeMap::<String, Task87CandidateBatchCounterSnapshot>::new();
    for snapshot_set in snapshots {
        for snapshot in snapshot_set {
            merged
                .entry(snapshot.surface.clone())
                .and_modify(|existing| existing.merge(&snapshot))
                .or_insert(snapshot);
        }
    }
    merged.into_values().collect()
}

pub(crate) fn format_block_kernel_counter_lines(
    command: &str,
    label: &str,
    snapshots: &BlockKernelCounterSnapshots,
) -> String {
    let mut lines = Vec::new();
    for snapshot in &snapshots.block_kernel {
        lines.push(format!(
            "[block-kernel-counters] command={command} label={label} surface={} quant={} isa={} flushes={} candidates={} elapsed_nanos={} elapsed_ms={:.6} kernel_flushes={} kernel_candidates={} kernel_elapsed_nanos={} kernel_elapsed_ms={:.6} scalar_flushes={} scalar_candidates={} scalar_elapsed_nanos={} scalar_elapsed_ms={:.6} width_lt8={} width_8_15={} width_16_31={} width_ge32={} pruned_candidates={} kept_candidates={}",
            snapshot.surface,
            snapshot.quant_kind,
            snapshot.isa,
            snapshot.flushes,
            snapshot.candidates,
            snapshot.elapsed_nanos,
            snapshot.elapsed_nanos as f64 / 1_000_000.0,
            snapshot.kernel_flushes,
            snapshot.kernel_candidates,
            snapshot.kernel_elapsed_nanos,
            snapshot.kernel_elapsed_nanos as f64 / 1_000_000.0,
            snapshot.scalar_flushes,
            snapshot.scalar_candidates,
            snapshot.scalar_elapsed_nanos,
            snapshot.scalar_elapsed_nanos as f64 / 1_000_000.0,
            snapshot.width_lt8_flushes,
            snapshot.width_8_15_flushes,
            snapshot.width_16_31_flushes,
            snapshot.width_ge32_flushes,
            snapshot.pruned_candidates,
            snapshot.kept_candidates,
        ));
    }
    let task87_lines =
        format_task87_candidate_batch_counter_lines(command, label, &snapshots.task87_compat);
    if !task87_lines.is_empty() {
        lines.push(task87_lines);
    }
    lines.join("\n")
}

pub(crate) fn format_task87_candidate_batch_counter_lines(
    command: &str,
    label: &str,
    snapshots: &[Task87CandidateBatchCounterSnapshot],
) -> String {
    let mut lines = Vec::new();
    for snapshot in snapshots {
        lines.push(format!(
            "[task87-counters] command={command} label={label} surface={} flushes={} candidates={} elapsed_nanos={} elapsed_ms={:.6} lut32_flushes={} lut32_candidates={} lut32_pruned_candidates={} lut32_kept_candidates={}",
            snapshot.surface,
            snapshot.flushes,
            snapshot.candidates,
            snapshot.elapsed_nanos,
            snapshot.elapsed_nanos as f64 / 1_000_000.0,
            snapshot.lut32_flushes,
            snapshot.lut32_candidates,
            snapshot.lut32_pruned_candidates,
            snapshot.lut32_kept_candidates
        ));
    }
    lines.join("\n")
}

fn validate_guc_name(name: &str) -> Result<()> {
    let mut parts = name.split('.');
    let Some(first) = parts.next() else {
        return Err(eyre!("GUC name must not be empty"));
    };
    crate::profiles::validate_ident(first)?;
    for part in parts {
        crate::profiles::validate_ident(part)?;
    }
    Ok(())
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
    /// Standalone competitor measurement: recall@k + latency + storage for one external engine.
    Comparator(ComparatorArgs),
    /// DiskANN persisted graph diagnostics: reachability, degree, and edge counters.
    DiskannGraph(GraphArgs),
    /// DiskANN in-memory build diagnostics: candidate pools, pruning, and degree shape.
    DiskannBuildProbe(BuildProbeArgs),
    /// Latency overhead breakdown: encode vs internal scan vs residual client/protocol.
    Overhead(OverheadArgs),
    /// IVF/RaBitQ sidecar upper-bound rerank measurement.
    SidecarRerank(SidecarRerankArgs),
    /// RaBitQ prepared-estimator kernel microbenchmarks.
    RabitqKernel(RabitqKernelArgs),
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
            BenchCommand::Comparator(a) => comparator::run(conn, a).await,
            BenchCommand::DiskannGraph(a) => graph::run(conn, a).await,
            BenchCommand::DiskannBuildProbe(a) => build_probe::run(conn, a).await,
            BenchCommand::Overhead(a) => overhead::run(conn, a).await,
            BenchCommand::SidecarRerank(a) => sidecar_rerank::run(conn, a).await,
            BenchCommand::RabitqKernel(a) => rabitq_kernel::run(a).await,
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
                score_margin_ratio_bps: None,
            },
        )
        .is_ok());
        assert!(validate_adaptive_nprobe_options(
            &EC_IVF,
            AdaptiveNprobeBenchOptions {
                enabled: true,
                score_gap_micros: Some(0),
                score_margin_ratio_bps: None,
            },
        )
        .is_ok());
        assert!(validate_adaptive_nprobe_options(
            &EC_HNSW,
            AdaptiveNprobeBenchOptions {
                enabled: true,
                score_gap_micros: None,
                score_margin_ratio_bps: None,
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
                score_margin_ratio_bps: None,
            },
        )
        .unwrap_err()
        .to_string()
        .contains("requires --adaptive-nprobe"));
    }

    #[test]
    fn adaptive_nprobe_ratio_signal_is_ivf_only() {
        assert!(validate_adaptive_nprobe_options(
            &EC_IVF,
            AdaptiveNprobeBenchOptions {
                enabled: true,
                score_gap_micros: None,
                score_margin_ratio_bps: Some(2500),
            },
        )
        .is_ok());
        assert!(validate_adaptive_nprobe_options(
            &EC_SPIRE,
            AdaptiveNprobeBenchOptions {
                enabled: true,
                score_gap_micros: None,
                score_margin_ratio_bps: Some(2500),
            },
        )
        .unwrap_err()
        .to_string()
        .contains("--profile ec_ivf"));
    }

    #[test]
    fn block_kernel_counter_lines_include_transition_formats() {
        let snapshots = BlockKernelCounterSnapshots {
            block_kernel: vec![BlockKernelCounterSnapshot {
                surface: "ivf".to_owned(),
                quant_kind: "turboquant".to_owned(),
                isa: "scalar".to_owned(),
                flushes: 1,
                candidates: 39,
                elapsed_nanos: 1_500_000,
                kernel_flushes: 1,
                kernel_candidates: 32,
                kernel_elapsed_nanos: 1_100_000,
                scalar_flushes: 1,
                scalar_candidates: 7,
                scalar_elapsed_nanos: 400_000,
                width_lt8_flushes: 1,
                width_8_15_flushes: 0,
                width_16_31_flushes: 0,
                width_ge32_flushes: 1,
                pruned_candidates: 3,
                kept_candidates: 36,
            }],
            task87_compat: vec![Task87CandidateBatchCounterSnapshot {
                surface: "ivf".to_owned(),
                flushes: 1,
                candidates: 39,
                elapsed_nanos: 1_500_000,
                lut32_flushes: 1,
                lut32_candidates: 39,
                lut32_pruned_candidates: 3,
                lut32_kept_candidates: 36,
            }],
        };

        let lines = format_block_kernel_counter_lines("latency", "nprobe=8", &snapshots);

        assert!(lines.contains(
            "[block-kernel-counters] command=latency label=nprobe=8 surface=ivf quant=turboquant isa=scalar flushes=1 candidates=39"
        ));
        assert!(
            lines.contains("kernel_flushes=1 kernel_candidates=32 kernel_elapsed_nanos=1100000")
        );
        assert!(lines.contains("scalar_flushes=1 scalar_candidates=7 scalar_elapsed_nanos=400000"));
        assert!(lines.contains("pruned_candidates=3 kept_candidates=36"));
        assert!(lines.contains(
            "[task87-counters] command=latency label=nprobe=8 surface=ivf flushes=1 candidates=39"
        ));
        assert!(lines.contains("lut32_pruned_candidates=3 lut32_kept_candidates=36"));
    }
}
