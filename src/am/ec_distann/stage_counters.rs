//! Task 183 benchmark-only aggregate latency attribution for the physical
//! ec_distann read path. The extension functions reset and snapshot these
//! per-backend atomics around timed queries, after warmup has completed.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DistannQueryStage {
    QueryPrep,
    HeadScore,
    SeedSelect,
    TraversalTotal,
    LocalExpand,
    RemoteExpand,
    RemoteMaterialize,
    OutputMerge,
    CustomScanTotal,
}

impl DistannQueryStage {
    pub(crate) const ALL: [Self; 9] = [
        Self::QueryPrep,
        Self::HeadScore,
        Self::SeedSelect,
        Self::TraversalTotal,
        Self::LocalExpand,
        Self::RemoteExpand,
        Self::RemoteMaterialize,
        Self::OutputMerge,
        Self::CustomScanTotal,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::QueryPrep => "query_prep",
            Self::HeadScore => "head_score",
            Self::SeedSelect => "seed_select",
            Self::TraversalTotal => "traversal_total",
            Self::LocalExpand => "local_expand",
            Self::RemoteExpand => "remote_expand",
            Self::RemoteMaterialize => "remote_materialize",
            Self::OutputMerge => "output_merge",
            Self::CustomScanTotal => "custom_scan_total",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::QueryPrep => 0,
            Self::HeadScore => 1,
            Self::SeedSelect => 2,
            Self::TraversalTotal => 3,
            Self::LocalExpand => 4,
            Self::RemoteExpand => 5,
            Self::RemoteMaterialize => 6,
            Self::OutputMerge => 7,
            Self::CustomScanTotal => 8,
        }
    }
}

const STAGE_COUNT: usize = DistannQueryStage::ALL.len();
static STAGE_ELAPSED_NS: [AtomicU64; STAGE_COUNT] = [const { AtomicU64::new(0) }; STAGE_COUNT];
static STAGE_SAMPLES: [AtomicU64; STAGE_COUNT] = [const { AtomicU64::new(0) }; STAGE_COUNT];
static SCANS: AtomicU64 = AtomicU64::new(0);

pub(crate) fn record(stage: DistannQueryStage, elapsed: Duration) {
    let index = stage.index();
    let nanos = u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
    STAGE_ELAPSED_NS[index].fetch_add(nanos, Ordering::Relaxed);
    STAGE_SAMPLES[index].fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn record_scan() {
    SCANS.fetch_add(1, Ordering::Relaxed);
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DistannStageSnapshotRow {
    pub(crate) stage: DistannQueryStage,
    pub(crate) samples: u64,
    pub(crate) elapsed_ns: u64,
}

pub(crate) fn snapshot() -> (u64, Vec<DistannStageSnapshotRow>) {
    let scans = SCANS.load(Ordering::Relaxed);
    let rows = DistannQueryStage::ALL
        .iter()
        .map(|&stage| DistannStageSnapshotRow {
            stage,
            samples: STAGE_SAMPLES[stage.index()].load(Ordering::Relaxed),
            elapsed_ns: STAGE_ELAPSED_NS[stage.index()].load(Ordering::Relaxed),
        })
        .collect();
    (scans, rows)
}

pub(crate) fn reset() {
    SCANS.store(0, Ordering::Relaxed);
    for index in 0..STAGE_COUNT {
        STAGE_ELAPSED_NS[index].store(0, Ordering::Relaxed);
        STAGE_SAMPLES[index].store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_accumulate_nested_samples_and_reset() {
        reset();
        record_scan();
        record(DistannQueryStage::RemoteExpand, Duration::from_nanos(5));
        record(DistannQueryStage::RemoteExpand, Duration::from_nanos(7));
        let (scans, rows) = snapshot();
        assert_eq!(scans, 1);
        let remote = rows
            .iter()
            .find(|row| row.stage == DistannQueryStage::RemoteExpand)
            .expect("remote stage");
        assert_eq!(remote.samples, 2);
        assert_eq!(remote.elapsed_ns, 12);
        reset();
        let (scans, rows) = snapshot();
        assert_eq!(scans, 0);
        assert!(rows
            .iter()
            .all(|row| row.samples == 0 && row.elapsed_ns == 0));
    }
}
