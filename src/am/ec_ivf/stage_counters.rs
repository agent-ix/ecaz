//! Task 133: process-global IVF query stage-latency accumulators.
//!
//! Mirrors the per-scan `IvfExplainCounters` stage timers into global atomics
//! so `ecaz bench latency --ivf-stage-counters` can attribute aggregate query
//! wall time across stages (the same surfacing pattern as the block-kernel
//! scoring counters). Stage semantics match the per-scan fields documented on
//! `IvfExplainCounters`; derived shares:
//! posting page I/O + decode = posting_visit − scratch_flush;
//! SoA copy/bookkeeping = scratch_flush − scorer_batch − candidate_record;
//! unattributed = approximate_scan − probe_plan − posting_visit − topk_collect.

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IvfQueryStage {
    ApproximateScan,
    ProbePlan,
    PostingVisit,
    ScratchFlush,
    ScorerBatch,
    CandidateRecord,
    TopkCollect,
    ExactRerank,
    RerankPayloadDecode,
    RerankPayloadScore,
}

impl IvfQueryStage {
    pub(crate) const ALL: [Self; 10] = [
        Self::ApproximateScan,
        Self::ProbePlan,
        Self::PostingVisit,
        Self::ScratchFlush,
        Self::ScorerBatch,
        Self::CandidateRecord,
        Self::TopkCollect,
        Self::ExactRerank,
        Self::RerankPayloadDecode,
        Self::RerankPayloadScore,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::ApproximateScan => "approximate_scan",
            Self::ProbePlan => "probe_plan",
            Self::PostingVisit => "posting_visit",
            Self::ScratchFlush => "scratch_flush",
            Self::ScorerBatch => "scorer_batch",
            Self::CandidateRecord => "candidate_record",
            Self::TopkCollect => "topk_collect",
            Self::ExactRerank => "exact_rerank",
            Self::RerankPayloadDecode => "rerank_payload_decode",
            Self::RerankPayloadScore => "rerank_payload_score",
        }
    }

    fn index(self) -> usize {
        match self {
            Self::ApproximateScan => 0,
            Self::ProbePlan => 1,
            Self::PostingVisit => 2,
            Self::ScratchFlush => 3,
            Self::ScorerBatch => 4,
            Self::CandidateRecord => 5,
            Self::TopkCollect => 6,
            Self::ExactRerank => 7,
            Self::RerankPayloadDecode => 8,
            Self::RerankPayloadScore => 9,
        }
    }
}

const STAGE_COUNT: usize = IvfQueryStage::ALL.len();

static STAGE_ELAPSED_US: [AtomicU64; STAGE_COUNT] =
    [const { AtomicU64::new(0) }; STAGE_COUNT];
static STAGE_SAMPLES: [AtomicU64; STAGE_COUNT] = [const { AtomicU64::new(0) }; STAGE_COUNT];
static SCANS: AtomicU64 = AtomicU64::new(0);

pub(crate) fn record_stage_elapsed_us(stage: IvfQueryStage, elapsed_us: u32) {
    let index = stage.index();
    STAGE_ELAPSED_US[index].fetch_add(u64::from(elapsed_us), Ordering::Relaxed);
    STAGE_SAMPLES[index].fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn record_scan() {
    SCANS.fetch_add(1, Ordering::Relaxed);
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IvfStageSnapshotRow {
    pub stage: IvfQueryStage,
    pub samples: u64,
    pub elapsed_us: u64,
}

pub(crate) fn snapshot() -> (u64, Vec<IvfStageSnapshotRow>) {
    let scans = SCANS.load(Ordering::Relaxed);
    let rows = IvfQueryStage::ALL
        .iter()
        .map(|&stage| IvfStageSnapshotRow {
            stage,
            samples: STAGE_SAMPLES[stage.index()].load(Ordering::Relaxed),
            elapsed_us: STAGE_ELAPSED_US[stage.index()].load(Ordering::Relaxed),
        })
        .collect();
    (scans, rows)
}

pub(crate) fn reset() {
    SCANS.store(0, Ordering::Relaxed);
    for index in 0..STAGE_COUNT {
        STAGE_ELAPSED_US[index].store(0, Ordering::Relaxed);
        STAGE_SAMPLES[index].store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_counters_accumulate_and_reset() {
        reset();
        record_scan();
        record_stage_elapsed_us(IvfQueryStage::ScorerBatch, 5);
        record_stage_elapsed_us(IvfQueryStage::ScorerBatch, 7);
        record_stage_elapsed_us(IvfQueryStage::TopkCollect, 3);
        let (scans, rows) = snapshot();
        assert_eq!(scans, 1);
        let scorer = rows
            .iter()
            .find(|row| row.stage == IvfQueryStage::ScorerBatch)
            .expect("scorer row");
        assert_eq!(scorer.samples, 2);
        assert_eq!(scorer.elapsed_us, 12);
        let topk = rows
            .iter()
            .find(|row| row.stage == IvfQueryStage::TopkCollect)
            .expect("topk row");
        assert_eq!(topk.samples, 1);
        assert_eq!(topk.elapsed_us, 3);
        reset();
        let (scans, rows) = snapshot();
        assert_eq!(scans, 0);
        assert!(rows.iter().all(|row| row.samples == 0 && row.elapsed_us == 0));
    }
}
