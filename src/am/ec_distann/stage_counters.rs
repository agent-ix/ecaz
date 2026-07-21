//! Task 183/184 benchmark-only aggregate latency and materialization-work
//! attribution for the physical ec_distann read path. The extension functions
//! reset and snapshot these per-backend atomics around timed queries, after
//! warmup has completed.

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
    MaterializePrepare,
    MaterializeConnectionReady,
    MaterializeRequestWait,
    MaterializeRequestRoundtripWork,
    MaterializeOwnerEndpointWork,
    MaterializeOwnerEndpointCritical,
    MaterializeOwnerOpenValidateWork,
    MaterializeOwnerNodeLookupWork,
    MaterializeOwnerPayloadSqlWork,
    MaterializeCoordinatorDecode,
    MaterializeMapInsert,
    MaterializeOutputAssociate,
    TraversalCoordinatorPartition,
    TraversalRequestEncode,
    TraversalOwnerGraphRead,
    TraversalOwnerScore,
    TraversalTransportWait,
    TraversalCoordinatorDecode,
    TraversalFrontierInsert,
}

impl DistannQueryStage {
    pub(crate) const ALL: [Self; 28] = [
        Self::QueryPrep,
        Self::HeadScore,
        Self::SeedSelect,
        Self::TraversalTotal,
        Self::LocalExpand,
        Self::RemoteExpand,
        Self::RemoteMaterialize,
        Self::OutputMerge,
        Self::CustomScanTotal,
        Self::MaterializePrepare,
        Self::MaterializeConnectionReady,
        Self::MaterializeRequestWait,
        Self::MaterializeRequestRoundtripWork,
        Self::MaterializeOwnerEndpointWork,
        Self::MaterializeOwnerEndpointCritical,
        Self::MaterializeOwnerOpenValidateWork,
        Self::MaterializeOwnerNodeLookupWork,
        Self::MaterializeOwnerPayloadSqlWork,
        Self::MaterializeCoordinatorDecode,
        Self::MaterializeMapInsert,
        Self::MaterializeOutputAssociate,
        Self::TraversalCoordinatorPartition,
        Self::TraversalRequestEncode,
        Self::TraversalOwnerGraphRead,
        Self::TraversalOwnerScore,
        Self::TraversalTransportWait,
        Self::TraversalCoordinatorDecode,
        Self::TraversalFrontierInsert,
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
            Self::MaterializePrepare => "materialize_prepare",
            Self::MaterializeConnectionReady => "materialize_connection_ready",
            Self::MaterializeRequestWait => "materialize_request_wait",
            Self::MaterializeRequestRoundtripWork => "materialize_request_roundtrip_work",
            Self::MaterializeOwnerEndpointWork => "materialize_owner_endpoint_work",
            Self::MaterializeOwnerEndpointCritical => "materialize_owner_endpoint_critical",
            Self::MaterializeOwnerOpenValidateWork => "materialize_owner_open_validate_work",
            Self::MaterializeOwnerNodeLookupWork => "materialize_owner_node_lookup_work",
            Self::MaterializeOwnerPayloadSqlWork => "materialize_owner_payload_sql_work",
            Self::MaterializeCoordinatorDecode => "materialize_coordinator_decode",
            Self::MaterializeMapInsert => "materialize_map_insert",
            Self::MaterializeOutputAssociate => "materialize_output_associate",
            Self::TraversalCoordinatorPartition => "traversal_coordinator_partition",
            Self::TraversalRequestEncode => "traversal_request_encode",
            Self::TraversalOwnerGraphRead => "traversal_owner_graph_read",
            Self::TraversalOwnerScore => "traversal_owner_score",
            Self::TraversalTransportWait => "traversal_transport_wait",
            Self::TraversalCoordinatorDecode => "traversal_coordinator_decode",
            Self::TraversalFrontierInsert => "traversal_frontier_insert",
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
            Self::MaterializePrepare => 9,
            Self::MaterializeConnectionReady => 10,
            Self::MaterializeRequestWait => 11,
            Self::MaterializeRequestRoundtripWork => 12,
            Self::MaterializeOwnerEndpointWork => 13,
            Self::MaterializeOwnerEndpointCritical => 14,
            Self::MaterializeOwnerOpenValidateWork => 15,
            Self::MaterializeOwnerNodeLookupWork => 16,
            Self::MaterializeOwnerPayloadSqlWork => 17,
            Self::MaterializeCoordinatorDecode => 18,
            Self::MaterializeMapInsert => 19,
            Self::MaterializeOutputAssociate => 20,
            Self::TraversalCoordinatorPartition => 21,
            Self::TraversalRequestEncode => 22,
            Self::TraversalOwnerGraphRead => 23,
            Self::TraversalOwnerScore => 24,
            Self::TraversalTransportWait => 25,
            Self::TraversalCoordinatorDecode => 26,
            Self::TraversalFrontierInsert => 27,
        }
    }
}

const STAGE_COUNT: usize = DistannQueryStage::ALL.len();
static STAGE_ELAPSED_NS: [AtomicU64; STAGE_COUNT] = [const { AtomicU64::new(0) }; STAGE_COUNT];
static STAGE_SAMPLES: [AtomicU64; STAGE_COUNT] = [const { AtomicU64::new(0) }; STAGE_COUNT];
static SCANS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DistannMaterializationWork {
    RankedCandidates,
    RemoteCandidatesRequested,
    RemoteOwnersRequested,
    RemoteRowsReturned,
    RemoteTombstones,
    PayloadColumnsRequested,
    PayloadBytesReturned,
    RemotePayloadsInstalled,
    OutputRowsAssociated,
    ExecutorRowsConsumed,
    ExecutorRemoteRowsConsumed,
    ExecutorLocalRowsConsumed,
    DuplicateRemoteCandidatesRequested,
    TraversalHopRounds,
    TraversalBatchWidth,
    TraversalNodesRequested,
    TraversalNodesReturned,
    TraversalFrontierInsertions,
    TraversalRepeatedNodes,
}

impl DistannMaterializationWork {
    pub(crate) const ALL: [Self; 19] = [
        Self::RankedCandidates,
        Self::RemoteCandidatesRequested,
        Self::RemoteOwnersRequested,
        Self::RemoteRowsReturned,
        Self::RemoteTombstones,
        Self::PayloadColumnsRequested,
        Self::PayloadBytesReturned,
        Self::RemotePayloadsInstalled,
        Self::OutputRowsAssociated,
        Self::ExecutorRowsConsumed,
        Self::ExecutorRemoteRowsConsumed,
        Self::ExecutorLocalRowsConsumed,
        Self::DuplicateRemoteCandidatesRequested,
        Self::TraversalHopRounds,
        Self::TraversalBatchWidth,
        Self::TraversalNodesRequested,
        Self::TraversalNodesReturned,
        Self::TraversalFrontierInsertions,
        Self::TraversalRepeatedNodes,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::RankedCandidates => "ranked_candidates",
            Self::RemoteCandidatesRequested => "remote_candidates_requested",
            Self::RemoteOwnersRequested => "remote_owners_requested",
            Self::RemoteRowsReturned => "remote_rows_returned",
            Self::RemoteTombstones => "remote_tombstones",
            Self::PayloadColumnsRequested => "payload_columns_requested",
            Self::PayloadBytesReturned => "payload_bytes_returned",
            Self::RemotePayloadsInstalled => "remote_payloads_installed",
            Self::OutputRowsAssociated => "output_rows_associated",
            Self::ExecutorRowsConsumed => "executor_rows_consumed",
            Self::ExecutorRemoteRowsConsumed => "executor_remote_rows_consumed",
            Self::ExecutorLocalRowsConsumed => "executor_local_rows_consumed",
            Self::DuplicateRemoteCandidatesRequested => "duplicate_remote_candidates_requested",
            Self::TraversalHopRounds => "traversal_hop_rounds",
            Self::TraversalBatchWidth => "traversal_batch_width",
            Self::TraversalNodesRequested => "traversal_nodes_requested",
            Self::TraversalNodesReturned => "traversal_nodes_returned",
            Self::TraversalFrontierInsertions => "traversal_frontier_insertions",
            Self::TraversalRepeatedNodes => "traversal_repeated_nodes",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::RankedCandidates => 0,
            Self::RemoteCandidatesRequested => 1,
            Self::RemoteOwnersRequested => 2,
            Self::RemoteRowsReturned => 3,
            Self::RemoteTombstones => 4,
            Self::PayloadColumnsRequested => 5,
            Self::PayloadBytesReturned => 6,
            Self::RemotePayloadsInstalled => 7,
            Self::OutputRowsAssociated => 8,
            Self::ExecutorRowsConsumed => 9,
            Self::ExecutorRemoteRowsConsumed => 10,
            Self::ExecutorLocalRowsConsumed => 11,
            Self::DuplicateRemoteCandidatesRequested => 12,
            Self::TraversalHopRounds => 13,
            Self::TraversalBatchWidth => 14,
            Self::TraversalNodesRequested => 15,
            Self::TraversalNodesReturned => 16,
            Self::TraversalFrontierInsertions => 17,
            Self::TraversalRepeatedNodes => 18,
        }
    }
}

const WORK_COUNT: usize = DistannMaterializationWork::ALL.len();
static MATERIALIZATION_WORK: [AtomicU64; WORK_COUNT] = [const { AtomicU64::new(0) }; WORK_COUNT];

pub(crate) fn record(stage: DistannQueryStage, elapsed: Duration) {
    let index = stage.index();
    let nanos = u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
    STAGE_ELAPSED_NS[index].fetch_add(nanos, Ordering::Relaxed);
    STAGE_SAMPLES[index].fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn record_scan() {
    SCANS.fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn record_work(metric: DistannMaterializationWork, value: usize) {
    MATERIALIZATION_WORK[metric.index()]
        .fetch_add(u64::try_from(value).unwrap_or(u64::MAX), Ordering::Relaxed);
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DistannStageSnapshotRow {
    pub(crate) stage: DistannQueryStage,
    pub(crate) samples: u64,
    pub(crate) elapsed_ns: u64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DistannMaterializationWorkSnapshotRow {
    pub(crate) metric: DistannMaterializationWork,
    pub(crate) value: u64,
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

pub(crate) fn materialization_work_snapshot() -> (u64, Vec<DistannMaterializationWorkSnapshotRow>) {
    let scans = SCANS.load(Ordering::Relaxed);
    let rows = DistannMaterializationWork::ALL
        .iter()
        .map(|&metric| DistannMaterializationWorkSnapshotRow {
            metric,
            value: MATERIALIZATION_WORK[metric.index()].load(Ordering::Relaxed),
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
    for counter in &MATERIALIZATION_WORK {
        counter.store(0, Ordering::Relaxed);
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
        record_work(DistannMaterializationWork::RemoteCandidatesRequested, 11);
        let (scans, rows) = snapshot();
        assert_eq!(scans, 1);
        let remote = rows
            .iter()
            .find(|row| row.stage == DistannQueryStage::RemoteExpand)
            .expect("remote stage");
        assert_eq!(remote.samples, 2);
        assert_eq!(remote.elapsed_ns, 12);
        let (_, work) = materialization_work_snapshot();
        let requested = work
            .iter()
            .find(|row| row.metric == DistannMaterializationWork::RemoteCandidatesRequested)
            .expect("requested work counter");
        assert_eq!(requested.value, 11);
        reset();
        let (scans, rows) = snapshot();
        assert_eq!(scans, 0);
        assert!(rows
            .iter()
            .all(|row| row.samples == 0 && row.elapsed_ns == 0));
        let (_, work) = materialization_work_snapshot();
        assert!(work.iter().all(|row| row.value == 0));
    }
}
