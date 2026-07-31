//! Task 183/184 benchmark-only aggregate latency and materialization-work
//! attribution for the physical ec_distann read path. The extension functions
//! reset and snapshot these per-backend atomics around timed queries, after
//! warmup has completed.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::{cell::RefCell, thread_local};

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
    TraversalConnectionReady,
    TraversalRequestEncode,
    TraversalOwnerOpenValidate,
    TraversalOwnerGraphRead,
    TraversalOwnerScore,
    TraversalOwnerResponseEncode,
    TraversalOwnerService,
    TraversalTransportWait,
    TraversalStragglerSpread,
    TraversalCoordinatorReceiveDecode,
    TraversalCoordinatorDecode,
    TraversalFrontierInsert,
    ReplicaOpenValidate,
    ReplicaGraphVectorRead,
    ReplicaScore,
}

impl DistannQueryStage {
    pub(crate) const ALL: [Self; 37] = [
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
        Self::TraversalConnectionReady,
        Self::TraversalRequestEncode,
        Self::TraversalOwnerOpenValidate,
        Self::TraversalOwnerGraphRead,
        Self::TraversalOwnerScore,
        Self::TraversalOwnerResponseEncode,
        Self::TraversalOwnerService,
        Self::TraversalTransportWait,
        Self::TraversalStragglerSpread,
        Self::TraversalCoordinatorReceiveDecode,
        Self::TraversalCoordinatorDecode,
        Self::TraversalFrontierInsert,
        Self::ReplicaOpenValidate,
        Self::ReplicaGraphVectorRead,
        Self::ReplicaScore,
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
            Self::TraversalConnectionReady => "traversal_connection_ready",
            Self::TraversalRequestEncode => "traversal_request_encode",
            Self::TraversalOwnerOpenValidate => "traversal_owner_open_validate",
            Self::TraversalOwnerGraphRead => "traversal_owner_graph_read",
            Self::TraversalOwnerScore => "traversal_owner_score",
            Self::TraversalOwnerResponseEncode => "traversal_owner_response_encode",
            Self::TraversalOwnerService => "traversal_owner_service",
            Self::TraversalTransportWait => "traversal_transport_wait",
            Self::TraversalStragglerSpread => "traversal_straggler_spread",
            Self::TraversalCoordinatorReceiveDecode => "traversal_coordinator_receive_decode",
            Self::TraversalCoordinatorDecode => "traversal_coordinator_decode",
            Self::TraversalFrontierInsert => "traversal_frontier_insert",
            Self::ReplicaOpenValidate => "replica_open_validate",
            Self::ReplicaGraphVectorRead => "replica_graph_vector_read",
            Self::ReplicaScore => "replica_score",
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
            Self::TraversalConnectionReady => 22,
            Self::TraversalRequestEncode => 23,
            Self::TraversalOwnerOpenValidate => 24,
            Self::TraversalOwnerGraphRead => 25,
            Self::TraversalOwnerScore => 26,
            Self::TraversalOwnerResponseEncode => 27,
            Self::TraversalOwnerService => 28,
            Self::TraversalTransportWait => 29,
            Self::TraversalStragglerSpread => 30,
            Self::TraversalCoordinatorReceiveDecode => 31,
            Self::TraversalCoordinatorDecode => 32,
            Self::TraversalFrontierInsert => 33,
            Self::ReplicaOpenValidate => 34,
            Self::ReplicaGraphVectorRead => 35,
            Self::ReplicaScore => 36,
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
    TraversalConnectionsOpened,
    TraversalStatementsPrepared,
    TraversalQueryCacheHits,
    TraversalQueryCacheMisses,
    TraversalRequestBytes,
    TraversalResponseBytes,
    ReplicaScans,
    ReplicaFallbacks,
    PushdownRoundsWithThreshold,
    NeighborsPruned,
    GatewayCopiesServed,
}

impl DistannMaterializationWork {
    pub(crate) const ALL: [Self; 30] = [
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
        Self::TraversalConnectionsOpened,
        Self::TraversalStatementsPrepared,
        Self::TraversalQueryCacheHits,
        Self::TraversalQueryCacheMisses,
        Self::TraversalRequestBytes,
        Self::TraversalResponseBytes,
        Self::ReplicaScans,
        Self::ReplicaFallbacks,
        Self::PushdownRoundsWithThreshold,
        Self::NeighborsPruned,
        Self::GatewayCopiesServed,
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
            Self::TraversalConnectionsOpened => "traversal_connections_opened",
            Self::TraversalStatementsPrepared => "traversal_statements_prepared",
            Self::TraversalQueryCacheHits => "traversal_query_cache_hits",
            Self::TraversalQueryCacheMisses => "traversal_query_cache_misses",
            Self::TraversalRequestBytes => "traversal_request_bytes",
            Self::TraversalResponseBytes => "traversal_response_bytes",
            Self::ReplicaScans => "replica_scans",
            Self::ReplicaFallbacks => "replica_fallbacks",
            Self::PushdownRoundsWithThreshold => "pushdown_rounds_with_threshold",
            Self::NeighborsPruned => "neighbors_pruned",
            Self::GatewayCopiesServed => "gateway_copies_served",
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
            Self::TraversalConnectionsOpened => 19,
            Self::TraversalStatementsPrepared => 20,
            Self::TraversalQueryCacheHits => 21,
            Self::TraversalQueryCacheMisses => 22,
            Self::TraversalRequestBytes => 23,
            Self::TraversalResponseBytes => 24,
            Self::ReplicaScans => 25,
            Self::ReplicaFallbacks => 26,
            Self::PushdownRoundsWithThreshold => 27,
            Self::NeighborsPruned => 28,
            Self::GatewayCopiesServed => 29,
        }
    }
}

const WORK_COUNT: usize = DistannMaterializationWork::ALL.len();
static MATERIALIZATION_WORK: [AtomicU64; WORK_COUNT] = [const { AtomicU64::new(0) }; WORK_COUNT];

#[derive(Clone, Copy, Default)]
struct BufferedStage {
    elapsed_ns: u64,
    samples: u64,
}

struct BufferedAttribution {
    stages: [BufferedStage; STAGE_COUNT],
    work: [u64; WORK_COUNT],
}

impl Default for BufferedAttribution {
    fn default() -> Self {
        Self {
            stages: [BufferedStage::default(); STAGE_COUNT],
            work: [0; WORK_COUNT],
        }
    }
}

thread_local! {
    static BUFFERED_ATTRIBUTION: RefCell<Option<BufferedAttribution>> =
        const { RefCell::new(None) };
}

struct AttributionBufferGuard;

impl Drop for AttributionBufferGuard {
    fn drop(&mut self) {
        BUFFERED_ATTRIBUTION.with(|buffer| {
            buffer.borrow_mut().take();
        });
    }
}

pub(crate) fn record(stage: DistannQueryStage, elapsed: Duration) {
    let index = stage.index();
    let nanos = u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
    let buffered = BUFFERED_ATTRIBUTION.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        let Some(buffer) = buffer.as_mut() else {
            return false;
        };
        buffer.stages[index].elapsed_ns = buffer.stages[index].elapsed_ns.saturating_add(nanos);
        buffer.stages[index].samples = buffer.stages[index].samples.saturating_add(1);
        true
    });
    if buffered {
        return;
    }
    STAGE_ELAPSED_NS[index].fetch_add(nanos, Ordering::Relaxed);
    STAGE_SAMPLES[index].fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn record_scan() {
    SCANS.fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn record_work(metric: DistannMaterializationWork, value: usize) {
    let index = metric.index();
    let value = u64::try_from(value).unwrap_or(u64::MAX);
    let buffered = BUFFERED_ATTRIBUTION.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        let Some(buffer) = buffer.as_mut() else {
            return false;
        };
        buffer.work[index] = buffer.work[index].saturating_add(value);
        true
    });
    if buffered {
        return;
    }
    MATERIALIZATION_WORK[index].fetch_add(value, Ordering::Relaxed);
}

/// Buffer one speculative traversal's attribution and publish it only if the
/// traversal succeeds. A replica failure must not leave its hop/frontier work
/// in the same scan totals as the full owner restart.
pub(crate) fn with_successful_attribution<T, E>(
    operation: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    BUFFERED_ATTRIBUTION.with(|buffer| {
        assert!(
            buffer.borrow().is_none(),
            "nested ec_distann attribution buffering is unsupported"
        );
        *buffer.borrow_mut() = Some(BufferedAttribution::default());
    });
    let _guard = AttributionBufferGuard;
    let result = operation();
    let buffered = BUFFERED_ATTRIBUTION
        .with(|buffer| buffer.borrow_mut().take())
        .expect("ec_distann attribution buffer disappeared");
    if result.is_ok() {
        for (index, stage) in buffered.stages.into_iter().enumerate() {
            STAGE_ELAPSED_NS[index].fetch_add(stage.elapsed_ns, Ordering::Relaxed);
            STAGE_SAMPLES[index].fetch_add(stage.samples, Ordering::Relaxed);
        }
        for (counter, value) in MATERIALIZATION_WORK.iter().zip(buffered.work) {
            counter.fetch_add(value, Ordering::Relaxed);
        }
    }
    result
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

    #[test]
    fn speculative_attribution_commits_only_on_success() {
        reset();
        let failed = with_successful_attribution(|| {
            record(
                DistannQueryStage::ReplicaGraphVectorRead,
                Duration::from_nanos(11),
            );
            record_work(DistannMaterializationWork::TraversalHopRounds, 3);
            Err::<(), _>("fallback")
        });
        assert_eq!(failed, Err("fallback"));
        let failed_replica_read = snapshot()
            .1
            .into_iter()
            .find(|row| row.stage == DistannQueryStage::ReplicaGraphVectorRead)
            .expect("replica read row after failed traversal");
        assert_eq!(failed_replica_read.samples, 0);
        assert_eq!(failed_replica_read.elapsed_ns, 0);
        let failed_rounds = materialization_work_snapshot()
            .1
            .into_iter()
            .find(|row| row.metric == DistannMaterializationWork::TraversalHopRounds)
            .expect("hop rounds row after failed traversal");
        assert_eq!(failed_rounds.value, 0);

        with_successful_attribution(|| {
            record(
                DistannQueryStage::ReplicaGraphVectorRead,
                Duration::from_nanos(13),
            );
            record_work(DistannMaterializationWork::TraversalHopRounds, 2);
            Ok::<(), &str>(())
        })
        .expect("successful traversal");
        let replica_read = snapshot()
            .1
            .into_iter()
            .find(|row| row.stage == DistannQueryStage::ReplicaGraphVectorRead)
            .expect("replica read row");
        assert_eq!(replica_read.samples, 1);
        assert_eq!(replica_read.elapsed_ns, 13);
        let rounds = materialization_work_snapshot()
            .1
            .into_iter()
            .find(|row| row.metric == DistannMaterializationWork::TraversalHopRounds)
            .expect("hop rounds row");
        assert_eq!(rounds.value, 2);
    }
}
