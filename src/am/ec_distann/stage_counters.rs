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
    MaterializeOwnerSidecarLookupWork,
    MaterializeLocalSidecarInitial,
    MaterializeLocalSidecarRetry,
}

impl DistannQueryStage {
    pub(crate) const ALL: [Self; 40] = [
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
        Self::MaterializeOwnerSidecarLookupWork,
        Self::MaterializeLocalSidecarInitial,
        Self::MaterializeLocalSidecarRetry,
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
            Self::MaterializeOwnerSidecarLookupWork => "materialize_owner_sidecar_lookup_work",
            Self::MaterializeLocalSidecarInitial => "materialize_local_sidecar_initial",
            Self::MaterializeLocalSidecarRetry => "materialize_local_sidecar_retry",
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
            Self::MaterializeOwnerSidecarLookupWork => 37,
            Self::MaterializeLocalSidecarInitial => 38,
            Self::MaterializeLocalSidecarRetry => 39,
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
    TraversalFrontierRetries,
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
    HeadReplicaFallbacks,
    HeadReplicaShardsServed,
    RemoteSidecarBatches,
    RemoteSidecarRowsRequested,
    RemoteSidecarRowsReturned,
    RemoteSidecarRowsMissing,
    RemoteSidecarPayloadBytes,
    RemoteSidecarRowTierVisibilityProbes,
    LocalSidecarInitialBatches,
    LocalSidecarInitialRowsRequested,
    LocalSidecarInitialRowsReturned,
    LocalSidecarInitialRowsMissing,
    LocalSidecarInitialPayloadBytes,
    LocalSidecarInitialRowTierVisibilityProbes,
    LocalSidecarRetryBatches,
    LocalSidecarRetryRowsRequested,
    LocalSidecarRetryRowsReturned,
    LocalSidecarRetryRowsMissing,
    LocalSidecarRetryPayloadBytes,
    LocalSidecarRetryRowTierVisibilityProbes,
    HotTierRelationOpens,
    ColdTierRelationOpens,
    HotTierTupleReads,
    ColdTierTupleReads,
    HotTierBlocksRequested,
    ColdTierBlocksRequested,
    HotTierPayloadBytes,
    ColdTierPayloadBytes,
    ExactVectorReads,
    ExactVectorBytes,
    FixedStrideLogicalBlocksTouched,
    FixedStrideLogicalBytesTouched,
    GraphDirectoryProbes,
    FixedStrideSharedBufferHits,
    FixedStrideSharedBufferReads,
}

impl DistannMaterializationWork {
    // Keep DISTANN_WORK_ROWS in
    // crates/ecaz-cli/src/commands/dev/distann_multicluster.rs synchronized:
    // the CLI contract is this server count plus client_result_rows.
    pub(crate) const ALL: [Self; 66] = [
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
        Self::TraversalFrontierRetries,
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
        Self::HeadReplicaFallbacks,
        Self::HeadReplicaShardsServed,
        Self::RemoteSidecarBatches,
        Self::RemoteSidecarRowsRequested,
        Self::RemoteSidecarRowsReturned,
        Self::RemoteSidecarRowsMissing,
        Self::RemoteSidecarPayloadBytes,
        Self::RemoteSidecarRowTierVisibilityProbes,
        Self::LocalSidecarInitialBatches,
        Self::LocalSidecarInitialRowsRequested,
        Self::LocalSidecarInitialRowsReturned,
        Self::LocalSidecarInitialRowsMissing,
        Self::LocalSidecarInitialPayloadBytes,
        Self::LocalSidecarInitialRowTierVisibilityProbes,
        Self::LocalSidecarRetryBatches,
        Self::LocalSidecarRetryRowsRequested,
        Self::LocalSidecarRetryRowsReturned,
        Self::LocalSidecarRetryRowsMissing,
        Self::LocalSidecarRetryPayloadBytes,
        Self::LocalSidecarRetryRowTierVisibilityProbes,
        Self::HotTierRelationOpens,
        Self::ColdTierRelationOpens,
        Self::HotTierTupleReads,
        Self::ColdTierTupleReads,
        Self::HotTierBlocksRequested,
        Self::ColdTierBlocksRequested,
        Self::HotTierPayloadBytes,
        Self::ColdTierPayloadBytes,
        Self::ExactVectorReads,
        Self::ExactVectorBytes,
        Self::FixedStrideLogicalBlocksTouched,
        Self::FixedStrideLogicalBytesTouched,
        Self::GraphDirectoryProbes,
        Self::FixedStrideSharedBufferHits,
        Self::FixedStrideSharedBufferReads,
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
            Self::TraversalFrontierRetries => "traversal_frontier_retries",
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
            Self::HeadReplicaFallbacks => "head_replica_fallbacks",
            Self::HeadReplicaShardsServed => "head_replica_shards_served",
            Self::RemoteSidecarBatches => "remote_sidecar_batches",
            Self::RemoteSidecarRowsRequested => "remote_sidecar_rows_requested",
            Self::RemoteSidecarRowsReturned => "remote_sidecar_rows_returned",
            Self::RemoteSidecarRowsMissing => "remote_sidecar_rows_missing",
            Self::RemoteSidecarPayloadBytes => "remote_sidecar_payload_bytes",
            Self::RemoteSidecarRowTierVisibilityProbes => {
                "remote_sidecar_row_tier_visibility_probes"
            }
            Self::LocalSidecarInitialBatches => "local_sidecar_initial_batches",
            Self::LocalSidecarInitialRowsRequested => "local_sidecar_initial_rows_requested",
            Self::LocalSidecarInitialRowsReturned => "local_sidecar_initial_rows_returned",
            Self::LocalSidecarInitialRowsMissing => "local_sidecar_initial_rows_missing",
            Self::LocalSidecarInitialPayloadBytes => "local_sidecar_initial_payload_bytes",
            Self::LocalSidecarInitialRowTierVisibilityProbes => {
                "local_sidecar_initial_row_tier_visibility_probes"
            }
            Self::LocalSidecarRetryBatches => "local_sidecar_retry_batches",
            Self::LocalSidecarRetryRowsRequested => "local_sidecar_retry_rows_requested",
            Self::LocalSidecarRetryRowsReturned => "local_sidecar_retry_rows_returned",
            Self::LocalSidecarRetryRowsMissing => "local_sidecar_retry_rows_missing",
            Self::LocalSidecarRetryPayloadBytes => "local_sidecar_retry_payload_bytes",
            Self::LocalSidecarRetryRowTierVisibilityProbes => {
                "local_sidecar_retry_row_tier_visibility_probes"
            }
            Self::HotTierRelationOpens => "hot_tier_relation_opens",
            Self::ColdTierRelationOpens => "cold_tier_relation_opens",
            Self::HotTierTupleReads => "hot_tier_tuple_reads",
            Self::ColdTierTupleReads => "cold_tier_tuple_reads",
            Self::HotTierBlocksRequested => "hot_tier_blocks_requested",
            Self::ColdTierBlocksRequested => "cold_tier_blocks_requested",
            Self::HotTierPayloadBytes => "hot_tier_payload_bytes",
            Self::ColdTierPayloadBytes => "cold_tier_payload_bytes",
            Self::ExactVectorReads => "exact_vector_reads",
            Self::ExactVectorBytes => "exact_vector_bytes",
            Self::FixedStrideLogicalBlocksTouched => "fixed_stride_logical_blocks_touched",
            Self::FixedStrideLogicalBytesTouched => "fixed_stride_logical_bytes_touched",
            Self::GraphDirectoryProbes => "graph_directory_probes",
            Self::FixedStrideSharedBufferHits => "fixed_stride_shared_buffer_hits",
            Self::FixedStrideSharedBufferReads => "fixed_stride_shared_buffer_reads",
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
            Self::TraversalFrontierRetries => 19,
            Self::TraversalConnectionsOpened => 20,
            Self::TraversalStatementsPrepared => 21,
            Self::TraversalQueryCacheHits => 22,
            Self::TraversalQueryCacheMisses => 23,
            Self::TraversalRequestBytes => 24,
            Self::TraversalResponseBytes => 25,
            Self::ReplicaScans => 26,
            Self::ReplicaFallbacks => 27,
            Self::PushdownRoundsWithThreshold => 28,
            Self::NeighborsPruned => 29,
            Self::GatewayCopiesServed => 30,
            Self::HeadReplicaFallbacks => 31,
            Self::HeadReplicaShardsServed => 32,
            Self::RemoteSidecarBatches => 33,
            Self::RemoteSidecarRowsRequested => 34,
            Self::RemoteSidecarRowsReturned => 35,
            Self::RemoteSidecarRowsMissing => 36,
            Self::RemoteSidecarPayloadBytes => 37,
            Self::RemoteSidecarRowTierVisibilityProbes => 38,
            Self::LocalSidecarInitialBatches => 39,
            Self::LocalSidecarInitialRowsRequested => 40,
            Self::LocalSidecarInitialRowsReturned => 41,
            Self::LocalSidecarInitialRowsMissing => 42,
            Self::LocalSidecarInitialPayloadBytes => 43,
            Self::LocalSidecarInitialRowTierVisibilityProbes => 44,
            Self::LocalSidecarRetryBatches => 45,
            Self::LocalSidecarRetryRowsRequested => 46,
            Self::LocalSidecarRetryRowsReturned => 47,
            Self::LocalSidecarRetryRowsMissing => 48,
            Self::LocalSidecarRetryPayloadBytes => 49,
            Self::LocalSidecarRetryRowTierVisibilityProbes => 50,
            Self::HotTierRelationOpens => 51,
            Self::ColdTierRelationOpens => 52,
            Self::HotTierTupleReads => 53,
            Self::ColdTierTupleReads => 54,
            Self::HotTierBlocksRequested => 55,
            Self::ColdTierBlocksRequested => 56,
            Self::HotTierPayloadBytes => 57,
            Self::ColdTierPayloadBytes => 58,
            Self::ExactVectorReads => 59,
            Self::ExactVectorBytes => 60,
            Self::FixedStrideLogicalBlocksTouched => 61,
            Self::FixedStrideLogicalBytesTouched => 62,
            Self::GraphDirectoryProbes => 63,
            Self::FixedStrideSharedBufferHits => 64,
            Self::FixedStrideSharedBufferReads => 65,
        }
    }
}

const WORK_COUNT: usize = DistannMaterializationWork::ALL.len();
static MATERIALIZATION_WORK: [AtomicU64; WORK_COUNT] = [const { AtomicU64::new(0) }; WORK_COUNT];

/// Task 167 physical insert work. These counters are intentionally separate
/// from query/materialization attribution: they are reset immediately before
/// the insert A/B arm and make the bounded per-insert graph work inspectable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DistannInsertWork {
    InsertAttempts,
    SearchCandidates,
    ForwardNeighborsSelected,
    BacklinkAmendments,
    BacklinkAlreadyPresent,
    BacklinkNoRoom,
    OwnerWrites,
    GraphRecordsAppended,
    PayloadSidecarRowsAppended,
    PayloadSidecarBytesAppended,
}

impl DistannInsertWork {
    pub(crate) const ALL: [Self; 10] = [
        Self::InsertAttempts,
        Self::SearchCandidates,
        Self::ForwardNeighborsSelected,
        Self::BacklinkAmendments,
        Self::BacklinkAlreadyPresent,
        Self::BacklinkNoRoom,
        Self::OwnerWrites,
        Self::GraphRecordsAppended,
        Self::PayloadSidecarRowsAppended,
        Self::PayloadSidecarBytesAppended,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::InsertAttempts => "insert_attempts",
            Self::SearchCandidates => "search_candidates",
            Self::ForwardNeighborsSelected => "forward_neighbors_selected",
            Self::BacklinkAmendments => "backlink_amendments",
            Self::BacklinkAlreadyPresent => "backlink_already_present",
            Self::BacklinkNoRoom => "backlink_no_room",
            Self::OwnerWrites => "owner_writes",
            Self::GraphRecordsAppended => "graph_records_appended",
            Self::PayloadSidecarRowsAppended => "payload_sidecar_rows_appended",
            Self::PayloadSidecarBytesAppended => "payload_sidecar_bytes_appended",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::InsertAttempts => 0,
            Self::SearchCandidates => 1,
            Self::ForwardNeighborsSelected => 2,
            Self::BacklinkAmendments => 3,
            Self::BacklinkAlreadyPresent => 4,
            Self::BacklinkNoRoom => 5,
            Self::OwnerWrites => 6,
            Self::GraphRecordsAppended => 7,
            Self::PayloadSidecarRowsAppended => 8,
            Self::PayloadSidecarBytesAppended => 9,
        }
    }
}

const INSERT_WORK_COUNT: usize = DistannInsertWork::ALL.len();
static INSERT_WORK: [AtomicU64; INSERT_WORK_COUNT] =
    [const { AtomicU64::new(0) }; INSERT_WORK_COUNT];

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

pub(crate) const DISTANN_QUERY_TRACE_LOCATOR_LIMIT: usize = 65_536;

/// One bounded traversal-round snapshot for Task 227 residual attribution.
/// Distances use the scan's native `-ip` ordering (smaller is better).
#[derive(Debug, Default)]
pub(crate) struct DistannQueryRoundTrace {
    pub(crate) round: usize,
    pub(crate) requested_ids: Vec<u64>,
    pub(crate) returned_ids: Vec<u64>,
    pub(crate) exact_input_ids: Vec<u64>,
    pub(crate) exact_input_dists: Vec<f32>,
    pub(crate) retained_ids: Vec<u64>,
    pub(crate) retained_code_dists: Vec<f32>,
    pub(crate) code_threshold: Option<f32>,
    pub(crate) candidate_limit: usize,
    pub(crate) heap_saturated: bool,
    pub(crate) frontier_stable: bool,
    pub(crate) frontier_score_gap: Option<f32>,
    pub(crate) convergence_gap: Option<f32>,
    pub(crate) owner_ordinals: Vec<u32>,
    pub(crate) owner_request_counts: Vec<u32>,
    pub(crate) request_bytes: usize,
    pub(crate) response_bytes: usize,
}

/// Compact per-query provenance used only by benchmark endpoints. Task 185
/// consumes the seed-origin fields; Task 227 adds bounded round/frontier and
/// exact-result state so the CLI can join stable ids to held-out truth.
#[derive(Debug, Default)]
pub(crate) struct DistannSeedTrace {
    pub(crate) seed_ids: Vec<u64>,
    pub(crate) seed_code_dists: Vec<f32>,
    pub(crate) seed_expanded_counts: Vec<u32>,
    pub(crate) seed_hit_counts: Vec<u32>,
    pub(crate) hit_ids: Vec<u64>,
    pub(crate) hit_origin_masks: Vec<u32>,
    pub(crate) expanded_unique: u64,
    pub(crate) expanded_overlap: u64,
    pub(crate) rounds: Vec<DistannQueryRoundTrace>,
    pub(crate) exact_rerank_ids: Vec<u64>,
    pub(crate) exact_rerank_dists: Vec<f32>,
    pub(crate) final_ids: Vec<u64>,
    pub(crate) final_dists: Vec<f32>,
    pub(crate) rounds_executed: usize,
    pub(crate) early_exit: bool,
    pub(crate) beam_exhausted: bool,
    pub(crate) truncated: bool,
    captured_locators: usize,
}

impl DistannSeedTrace {
    fn remaining_locator_capacity(&self) -> usize {
        DISTANN_QUERY_TRACE_LOCATOR_LIMIT.saturating_sub(self.captured_locators)
    }

    fn reserve_locators(&mut self, requested: usize) -> usize {
        let captured = requested.min(self.remaining_locator_capacity());
        self.captured_locators = self.captured_locators.saturating_add(captured);
        self.truncated |= captured < requested;
        captured
    }

    pub(crate) fn truncate_final_results(&mut self, result_limit: usize) {
        self.final_ids.truncate(result_limit);
        self.final_dists.truncate(result_limit);
    }
}

thread_local! {
    static ACTIVE_SEED_TRACE: RefCell<Option<DistannSeedTrace>> =
        const { RefCell::new(None) };
}

pub(crate) fn with_seed_trace<T>(operation: impl FnOnce() -> T) -> (T, DistannSeedTrace) {
    ACTIVE_SEED_TRACE.with(|trace| {
        assert!(
            trace.borrow().is_none(),
            "nested ec_distann seed tracing is unsupported"
        );
        *trace.borrow_mut() = Some(DistannSeedTrace::default());
    });
    let result = operation();
    let trace = ACTIVE_SEED_TRACE
        .with(|trace| trace.borrow_mut().take())
        .expect("ec_distann seed trace disappeared");
    (result, trace)
}

pub(crate) fn seed_trace_start(seeds: &[(u64, f32)]) {
    ACTIVE_SEED_TRACE.with(|trace| {
        let mut trace = trace.borrow_mut();
        let Some(trace) = trace.as_mut() else {
            return;
        };
        // A scan may abandon a replica traversal and restart through owners.
        // Keep only the last attempt so a failed partial path cannot leak into
        // the successful query trace.
        *trace = DistannSeedTrace::default();
        let captured = trace.reserve_locators(seeds.len());
        trace
            .seed_ids
            .extend(seeds.iter().take(captured).map(|(vec_id, _)| *vec_id));
        trace
            .seed_code_dists
            .extend(seeds.iter().take(captured).map(|(_, dist)| *dist));
        trace.seed_expanded_counts = vec![0; captured];
        trace.seed_hit_counts = vec![0; captured];
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn seed_trace_round_start(
    round: usize,
    requested_ids: &[u64],
    code_threshold: Option<f32>,
    candidate_limit: usize,
) {
    ACTIVE_SEED_TRACE.with(|trace| {
        let mut trace = trace.borrow_mut();
        let Some(trace) = trace.as_mut() else {
            return;
        };
        let captured = trace.reserve_locators(requested_ids.len());
        trace.rounds.push(DistannQueryRoundTrace {
            round,
            requested_ids: requested_ids.iter().copied().take(captured).collect(),
            code_threshold,
            candidate_limit,
            ..DistannQueryRoundTrace::default()
        });
    });
}

pub(crate) fn seed_trace_owner_fanout(owner_requests: &[(usize, usize)]) {
    ACTIVE_SEED_TRACE.with(|trace| {
        let mut trace = trace.borrow_mut();
        let Some(round) = trace.as_mut().and_then(|trace| trace.rounds.last_mut()) else {
            return;
        };
        round.owner_ordinals.extend(
            owner_requests
                .iter()
                .map(|(ordinal, _)| u32::try_from(*ordinal).unwrap_or(u32::MAX)),
        );
        round.owner_request_counts.extend(
            owner_requests
                .iter()
                .map(|(_, count)| u32::try_from(*count).unwrap_or(u32::MAX)),
        );
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn seed_trace_round_finish(
    returned_ids: &[u64],
    exact_inputs: &[(u64, f32)],
    retained: &[(u64, f32)],
    candidate_limit: usize,
    kth_exact_dist: Option<f32>,
    request_bytes: usize,
    response_bytes: usize,
) {
    ACTIVE_SEED_TRACE.with(|trace| {
        let mut trace = trace.borrow_mut();
        let Some(trace) = trace.as_mut() else {
            return;
        };
        let previous_retained_ids = trace
            .rounds
            .iter()
            .rev()
            .nth(1)
            .map(|round| round.retained_ids.clone());
        let returned_count = trace.reserve_locators(returned_ids.len());
        let exact_count = trace.reserve_locators(exact_inputs.len());
        let retained_count = trace.reserve_locators(retained.len());
        let Some(round) = trace.rounds.last_mut() else {
            return;
        };
        round
            .returned_ids
            .extend(returned_ids.iter().copied().take(returned_count));
        round.exact_input_ids.extend(
            exact_inputs
                .iter()
                .take(exact_count)
                .map(|(vec_id, _)| *vec_id),
        );
        round
            .exact_input_dists
            .extend(exact_inputs.iter().take(exact_count).map(|(_, dist)| *dist));
        round.retained_ids.extend(
            retained
                .iter()
                .take(retained_count)
                .map(|(vec_id, _)| *vec_id),
        );
        round
            .retained_code_dists
            .extend(retained.iter().take(retained_count).map(|(_, dist)| *dist));
        round.heap_saturated = retained.len() >= candidate_limit;
        round.frontier_stable = previous_retained_ids
            .as_deref()
            .is_some_and(|previous| previous == round.retained_ids);
        round.frontier_score_gap = retained
            .first()
            .zip(retained.last())
            .map(|((_, best), (_, worst))| *worst - *best);
        round.convergence_gap = retained
            .first()
            .zip(kth_exact_dist)
            .map(|((_, best), kth)| *best - kth);
        round.request_bytes = request_bytes;
        round.response_bytes = response_bytes;
    });
}

pub(crate) fn seed_trace_terminal(
    exact_rerank: &[(u64, f32)],
    top_k: usize,
    rounds_executed: usize,
    early_exit: bool,
    beam_exhausted: bool,
) {
    ACTIVE_SEED_TRACE.with(|trace| {
        let mut trace = trace.borrow_mut();
        let Some(trace) = trace.as_mut() else {
            return;
        };
        let rerank_count = trace.reserve_locators(exact_rerank.len());
        let final_requested = exact_rerank.len().min(top_k);
        let final_count = trace.reserve_locators(final_requested);
        trace.exact_rerank_ids.extend(
            exact_rerank
                .iter()
                .take(rerank_count)
                .map(|(vec_id, _)| *vec_id),
        );
        trace.exact_rerank_dists.extend(
            exact_rerank
                .iter()
                .take(rerank_count)
                .map(|(_, dist)| *dist),
        );
        trace.final_ids.extend(
            exact_rerank
                .iter()
                .take(final_count)
                .map(|(vec_id, _)| *vec_id),
        );
        trace
            .final_dists
            .extend(exact_rerank.iter().take(final_count).map(|(_, dist)| *dist));
        trace.rounds_executed = rounds_executed;
        trace.early_exit = early_exit;
        trace.beam_exhausted = beam_exhausted;
    });
}

pub(crate) fn seed_trace_expanded(origin_mask: u32) {
    ACTIVE_SEED_TRACE.with(|trace| {
        let mut trace = trace.borrow_mut();
        let Some(trace) = trace.as_mut() else {
            return;
        };
        trace.expanded_unique = trace.expanded_unique.saturating_add(1);
        trace.expanded_overlap = trace
            .expanded_overlap
            .saturating_add(u64::from(origin_mask.count_ones().saturating_sub(1)));
        for (index, count) in trace.seed_expanded_counts.iter_mut().enumerate() {
            if u32::try_from(index)
                .ok()
                .and_then(|index| 1_u32.checked_shl(index))
                .is_some_and(|bit| origin_mask & bit != 0)
            {
                *count = count.saturating_add(1);
            }
        }
    });
}

pub(crate) fn seed_trace_hit(vec_id: u64, origin_mask: u32) {
    ACTIVE_SEED_TRACE.with(|trace| {
        let mut trace = trace.borrow_mut();
        let Some(trace) = trace.as_mut() else {
            return;
        };
        if trace.reserve_locators(1) == 1 {
            trace.hit_ids.push(vec_id);
            trace.hit_origin_masks.push(origin_mask);
        }
        for (index, count) in trace.seed_hit_counts.iter_mut().enumerate() {
            if u32::try_from(index)
                .ok()
                .and_then(|index| 1_u32.checked_shl(index))
                .is_some_and(|bit| origin_mask & bit != 0)
            {
                *count = count.saturating_add(1);
            }
        }
    });
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct DistannInsertWorkSnapshotRow {
    pub(crate) metric: DistannInsertWork,
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

pub(crate) fn record_insert_work(metric: DistannInsertWork, value: usize) {
    let value = u64::try_from(value).unwrap_or(u64::MAX);
    INSERT_WORK[metric.index()].fetch_add(value, Ordering::Relaxed);
}

pub(crate) fn insert_work_snapshot() -> (u64, Vec<DistannInsertWorkSnapshotRow>) {
    let inserts = INSERT_WORK[DistannInsertWork::InsertAttempts.index()].load(Ordering::Relaxed);
    let rows = DistannInsertWork::ALL
        .iter()
        .map(|&metric| DistannInsertWorkSnapshotRow {
            metric,
            value: INSERT_WORK[metric.index()].load(Ordering::Relaxed),
        })
        .collect();
    (inserts, rows)
}

pub(crate) fn reset_stage_scoring() {
    SCANS.store(0, Ordering::Relaxed);
    for index in 0..STAGE_COUNT {
        STAGE_ELAPSED_NS[index].store(0, Ordering::Relaxed);
        STAGE_SAMPLES[index].store(0, Ordering::Relaxed);
    }
    for counter in &MATERIALIZATION_WORK {
        counter.store(0, Ordering::Relaxed);
    }
}

pub(crate) fn reset_insert_work() {
    for counter in &INSERT_WORK {
        counter.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
fn reset() {
    reset_stage_scoring();
    reset_insert_work();
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
        record_insert_work(DistannInsertWork::InsertAttempts, 2);
        record_insert_work(DistannInsertWork::ForwardNeighborsSelected, 5);
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
        let (inserts, insert_work) = insert_work_snapshot();
        assert_eq!(inserts, 2);
        let forward = insert_work
            .iter()
            .find(|row| row.metric == DistannInsertWork::ForwardNeighborsSelected)
            .expect("forward insert work counter");
        assert_eq!(forward.value, 5);
        reset();
        let (scans, rows) = snapshot();
        assert_eq!(scans, 0);
        assert!(rows
            .iter()
            .all(|row| row.samples == 0 && row.elapsed_ns == 0));
        let (_, work) = materialization_work_snapshot();
        assert!(work.iter().all(|row| row.value == 0));
        let (inserts, work) = insert_work_snapshot();
        assert_eq!(inserts, 0);
        assert!(work.iter().all(|row| row.value == 0));
    }

    #[test]
    fn stage_and_insert_resets_are_independent() {
        reset();
        record_scan();
        record(DistannQueryStage::RemoteExpand, Duration::from_nanos(7));
        record_work(DistannMaterializationWork::RemoteCandidatesRequested, 11);
        record_insert_work(DistannInsertWork::InsertAttempts, 2);
        record_insert_work(DistannInsertWork::ForwardNeighborsSelected, 5);

        reset_stage_scoring();
        assert_eq!(snapshot().0, 0);
        assert!(snapshot()
            .1
            .iter()
            .all(|row| row.samples == 0 && row.elapsed_ns == 0));
        assert!(materialization_work_snapshot()
            .1
            .iter()
            .all(|row| row.value == 0));
        assert_eq!(insert_work_snapshot().0, 2);

        record_scan();
        record_insert_work(DistannInsertWork::InsertAttempts, 1);
        reset_insert_work();
        assert_eq!(insert_work_snapshot().0, 0);
        assert_eq!(snapshot().0, 1);
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

    #[cfg(feature = "distann-head-attribution-benchmark")]
    #[test]
    fn query_trace_is_bounded_and_resets_between_operations() {
        let oversized = (0..=DISTANN_QUERY_TRACE_LOCATOR_LIMIT)
            .map(|vec_id| (vec_id as u64, vec_id as f32))
            .collect::<Vec<_>>();
        let (_, first) = with_seed_trace(|| {
            seed_trace_start(&oversized);
            seed_trace_expanded(1);
            seed_trace_hit(0, 1);
        });
        assert_eq!(first.seed_ids.len(), DISTANN_QUERY_TRACE_LOCATOR_LIMIT);
        assert_eq!(
            first.seed_code_dists.len(),
            DISTANN_QUERY_TRACE_LOCATOR_LIMIT
        );
        assert!(first.truncated);
        assert_eq!(first.seed_expanded_counts[0], 1);
        assert_eq!(first.seed_hit_counts[0], 1);

        let (_, second) = with_seed_trace(|| {
            seed_trace_start(&[(6, -0.6)]);
            seed_trace_round_start(0, &[6], None, 4);
            seed_trace_start(&[(7, -0.7)]);
        });
        assert_eq!(second.seed_ids, vec![7]);
        assert_eq!(second.seed_code_dists, vec![-0.7]);
        assert!(!second.truncated);
        assert!(second.rounds.is_empty());
    }

    #[cfg(feature = "distann-head-attribution-benchmark")]
    #[test]
    fn query_trace_preserves_rerank_input_when_final_results_are_truncated() {
        let (_, mut trace) = with_seed_trace(|| {
            seed_trace_terminal(&[(1, -0.9), (2, -0.8), (3, -0.7)], 3, 1, false, true);
        });
        trace.truncate_final_results(2);
        assert_eq!(trace.exact_rerank_ids, vec![1, 2, 3]);
        assert_eq!(trace.final_ids, vec![1, 2]);
        assert_eq!(trace.final_dists, vec![-0.9, -0.8]);
    }
}
