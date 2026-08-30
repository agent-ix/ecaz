//! `ec_distann` is the DistributedANN-style fifth access method (ADR-085):
//! one global Vamana graph over all indexed vectors, lean node records
//! (coarse search code + adjacency + embedded neighbor codes, FR-076) with
//! the full-precision vector held once in a co-placed heap row for
//! node-local exact rerank (D11), searched by a coordinator loop of
//! head-index descent (FR-080) plus batched hop rounds (FR-081).
//!
//! The module now includes the physical distributed-generation lifecycle:
//! coordinator control indexes, immutable participant generations, persisted
//! head seeding, pooled owner fan-out, epoch pinning, and remote row payload
//! materialization (Tasks 162–179).

mod ambuild;
mod build_coordinator;
mod build_gate;
mod coordinator_abandonment;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ambuild::read_metadata_from_index;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ambuild::test_physical_capture_dead_callback_does_not_access_datums;
pub(crate) use self::ambuild::{build_physical_graph_workspace, capture_physical_source_rows};
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::build_coordinator::build_session_lock_count_for_test;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::custom_scan::{
    exec_state_context_cleanups_for_test, reset_exec_state_context_cleanups_for_test,
};
mod canonical_wire;
mod coordinator_retirement;
mod cost;
mod crown_cache;
mod custom_scan;
mod dml;
mod epoch;
mod epoch_manifest;
mod expand;
mod expand_error;
mod fixed_stride;
mod fixed_stride_store;
mod gateway_copy;
mod generation_catalog;
mod generation_descriptor;
mod generation_read;
#[cfg(feature = "pg_test")]
pub(crate) use self::generation_read::read_rpc_probe_for_test;
mod generation_store;
mod handoff;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::handoff::with_restricted_type_io_owner;
mod handoff_router;
mod handoff_wire;
mod head_cache;
mod head_sample;
mod identity;
mod insert;
mod lifecycle_guard;
mod lifecycle_state;
mod lifecycle_wire;
mod manifest_v2;
mod node_registry;
mod options;
pub mod page;
mod participant_lifecycle;
mod payload_projection;
mod payload_sidecar;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::payload_sidecar::resolve_payload_cover as resolve_payload_cover_for_test;
mod physical_dml;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::physical_dml::{
    insert_from_owner_payload_for_test, tombstone_owner_record as tombstone_owner_record_for_test,
};
pub(crate) mod placement;
pub(crate) mod quantizer;
pub(crate) mod reader;
mod remote_endpoint;
mod remote_transport;
#[cfg(feature = "pg_test")]
pub(crate) use self::remote_transport::read_transport_snapshot_for_test;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::remote_transport::remote_timeout_probe_for_test;
mod roster;
mod routine;
mod row_layout;
mod row_schema;
mod scan_registry;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::scan_registry::{
    live_token_count_for_fingerprint as live_scan_token_count_for_test,
    register_scan_token as register_scan_token_for_test,
    release_scan_token as release_scan_token_for_test, RegisterOutcome as ScanRegisterOutcome,
    RegistryError as ScanRegistryError, ScanTokenGuard as ScanTokenGuardForTest,
};
pub(crate) mod scan;
mod shard_build;
mod source_spool;
pub(crate) mod stage_counters;
mod traversal_replica;
pub mod tuple;

pub(crate) fn quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::coordinator_retirement::ensure_fingerprint_not_retiring as ensure_fingerprint_not_retiring_for_test;
pub use self::fixed_stride::DistannFixedStrideLayoutDescriptorV1;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::generation_catalog::extension_relation_name as catalog_relation_name;
pub use self::generation_descriptor::{
    roster_digest, DistannBuildOptions, DistannBuildSpec, DistannCodecArtifact,
    DistannGenerationDescriptor, DistannHeadPolicy, DistannHeadSizingAttestation,
    DistannOwnerExpectation, DistannRosterEntry, DISTANN_BUILD_SPEC_VERSION,
    DISTANN_BUILD_SPEC_VERSION_OFFSET, DISTANN_CODEC_ARTIFACT_VERSION,
    DISTANN_CODEC_ARTIFACT_VERSION_OFFSET, DISTANN_GENERATION_DESCRIPTOR_COORDINATOR_UUID_OFFSET,
    DISTANN_GENERATION_DESCRIPTOR_COVER_VERSION, DISTANN_GENERATION_DESCRIPTOR_DIMENSIONS_OFFSET,
    DISTANN_GENERATION_DESCRIPTOR_FIXED_PREFIX_BYTES,
    DISTANN_GENERATION_DESCRIPTOR_GRAPH_DEGREE_OFFSET,
    DISTANN_GENERATION_DESCRIPTOR_GRAPH_RECORD_OFFSET,
    DISTANN_GENERATION_DESCRIPTOR_HANDOFF_WIRE_OFFSET,
    DISTANN_GENERATION_DESCRIPTOR_HOT_COLD_VERSION,
    DISTANN_GENERATION_DESCRIPTOR_INDEX_FORMAT_OFFSET,
    DISTANN_GENERATION_DESCRIPTOR_PLACEMENT_HASH_OFFSET,
    DISTANN_GENERATION_DESCRIPTOR_ROSTER_COUNT_OFFSET, DISTANN_GENERATION_DESCRIPTOR_VERSION,
    DISTANN_GENERATION_DESCRIPTOR_VERSION_OFFSET, DISTANN_GRAPH_RECORD_VERSION,
    DISTANN_HANDOFF_WIRE_VERSION, DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
    DISTANN_PLACEMENT_HASH_VERSION,
};
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::handoff_router::{DistannHandoffRouteIdentity, DistannStageAck};
pub(crate) use self::handoff_wire::restore_owner_stream_hash_state;
pub use self::handoff_wire::{
    owner_stream_digest, DistannHandoffBatch, DistannHandoffEntry, DistannHandoffShape,
    DISTANN_HANDOFF_BATCH_FIXED_PREFIX_BYTES, DISTANN_HANDOFF_ENTRY_FIXED_PREFIX_BYTES,
    DISTANN_HANDOFF_MAX_BYTES, DISTANN_HANDOFF_VERSION_OFFSET,
    DISTANN_OWNER_STREAM_HASH_STATE_BLOCK_COUNT_OFFSET,
    DISTANN_OWNER_STREAM_HASH_STATE_BUFFER_LENGTH_OFFSET,
    DISTANN_OWNER_STREAM_HASH_STATE_BUFFER_OFFSET, DISTANN_OWNER_STREAM_HASH_STATE_BYTES,
    DISTANN_OWNER_STREAM_HASH_STATE_CHAIN_OFFSET,
    DISTANN_OWNER_STREAM_HASH_STATE_IMPLEMENTATION_OFFSET,
    DISTANN_OWNER_STREAM_HASH_STATE_VERSION_OFFSET,
};
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::identity::vec_id_from_source_identity;
pub use self::lifecycle_wire::{
    DistannAbandonBindingAuditV1, DistannAbandonedBinding, DistannAbandonedBindingSetV1,
    DistannBuildCandidateV1, DistannCancelPublishAuditV1, DistannPublishedEpochIdentity,
    DistannRetireDecisionV1, DistannSuccessorActivationV1, DISTANN_ABANDONED_BINDING_ENTRY_BYTES,
    DISTANN_ABANDONED_BINDING_SET_COUNT_OFFSET, DISTANN_ABANDONED_BINDING_SET_FIXED_PREFIX_BYTES,
    DISTANN_ABANDONED_BINDING_SET_VERSION, DISTANN_ABANDON_BINDING_AUDIT_COORDINATOR_UUID_OFFSET,
    DISTANN_ABANDON_BINDING_AUDIT_FIXED_PREFIX_BYTES,
    DISTANN_ABANDON_BINDING_AUDIT_SUCCESSOR_BUILD_ID_OFFSET,
    DISTANN_ABANDON_BINDING_AUDIT_SUCCESSOR_EPOCH_OFFSET,
    DISTANN_ABANDON_BINDING_AUDIT_SUCCESSOR_FINGERPRINT_LENGTH_OFFSET,
    DISTANN_ABANDON_BINDING_AUDIT_VERSION, DISTANN_ABANDON_BINDING_AUDIT_VERSION_OFFSET,
    DISTANN_BUILD_CANDIDATE_BUILD_SPEC_LENGTH_OFFSET, DISTANN_BUILD_CANDIDATE_FIXED_PREFIX_BYTES,
    DISTANN_BUILD_CANDIDATE_REGISTRATION_DIGEST_OFFSET, DISTANN_BUILD_CANDIDATE_VERSION,
    DISTANN_BUILD_CANDIDATE_VERSION_OFFSET, DISTANN_CANCEL_PUBLISH_AUDIT_BUILD_ID_OFFSET,
    DISTANN_CANCEL_PUBLISH_AUDIT_COORDINATOR_UUID_OFFSET,
    DISTANN_CANCEL_PUBLISH_AUDIT_EPOCH_OFFSET,
    DISTANN_CANCEL_PUBLISH_AUDIT_FINGERPRINT_LENGTH_OFFSET,
    DISTANN_CANCEL_PUBLISH_AUDIT_FIXED_PREFIX_BYTES, DISTANN_CANCEL_PUBLISH_AUDIT_VERSION,
    DISTANN_CANCEL_PUBLISH_AUDIT_VERSION_OFFSET, DISTANN_RETIRE_DECISION_COORDINATOR_UUID_OFFSET,
    DISTANN_RETIRE_DECISION_EPOCH_OFFSET, DISTANN_RETIRE_DECISION_FINGERPRINT_LENGTH_OFFSET,
    DISTANN_RETIRE_DECISION_FIXED_PREFIX_BYTES, DISTANN_RETIRE_DECISION_TARGET_BUILD_ID_OFFSET,
    DISTANN_RETIRE_DECISION_VERSION, DISTANN_RETIRE_DECISION_VERSION_OFFSET,
    DISTANN_SUCCESSOR_ACTIVATION_COORDINATOR_UUID_OFFSET,
    DISTANN_SUCCESSOR_ACTIVATION_FIXED_PREFIX_BYTES,
    DISTANN_SUCCESSOR_ACTIVATION_PREDECESSOR_PRESENT_OFFSET, DISTANN_SUCCESSOR_ACTIVATION_VERSION,
    DISTANN_SUCCESSOR_ACTIVATION_VERSION_OFFSET,
};
pub use self::manifest_v2::{
    DistannEpochFingerprint, DistannEpochManifestV2, DistannManifestBuildOptions,
    DistannManifestCodecParameters, DistannReadyReceipt, DistannReadyReceiptPayloadSidecar,
    DistannSourceSnapshot, DISTANN_EPOCH_FINGERPRINT_BYTES, DISTANN_EPOCH_MANIFEST_COVER_VERSION,
    DISTANN_EPOCH_MANIFEST_HOT_COLD_VERSION, DISTANN_EPOCH_MANIFEST_VERSION,
    DISTANN_EPOCH_MANIFEST_VERSION_OFFSET, DISTANN_MANIFEST_BUILD_OPTIONS_BYTES,
    DISTANN_MANIFEST_BUILD_OPTIONS_VERSION, DISTANN_MANIFEST_BUILD_OPTIONS_VERSION_OFFSET,
    DISTANN_MANIFEST_CODEC_PARAMETERS_BYTES, DISTANN_MANIFEST_CODEC_PARAMETERS_VERSION,
    DISTANN_MANIFEST_CODEC_PARAMETERS_VERSION_OFFSET, DISTANN_READY_RECEIPT_BYTES,
    DISTANN_READY_RECEIPT_COVER_BYTES, DISTANN_READY_RECEIPT_COVER_VERSION,
    DISTANN_READY_RECEIPT_HOT_COLD_BYTES, DISTANN_READY_RECEIPT_HOT_COLD_VERSION,
    DISTANN_READY_RECEIPT_MAX_BYTES, DISTANN_READY_RECEIPT_STATE, DISTANN_READY_RECEIPT_VERSION,
    DISTANN_READY_RECEIPT_VERSION_OFFSET, DISTANN_SOURCE_SNAPSHOT_VERSION,
    DISTANN_SOURCE_SNAPSHOT_VERSION_OFFSET,
};
pub use self::payload_sidecar::DistannPayloadCoverDescriptorV1;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::row_layout::resolve_hot_cold_layout as resolve_hot_cold_layout_for_test;
pub use self::row_layout::DistannRowTierLayoutDescriptorV1;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::row_schema::resolve_relation_schema;
pub use self::row_schema::{
    DistannRowSchemaAttribute, DistannRowSchemaDescriptor, DISTANN_ROW_SCHEMA_VERSION,
    DISTANN_ROW_SCHEMA_VERSION_OFFSET,
};

pub(crate) fn register_gucs() {
    options::register_gucs();
    roster::register_gucs();
    scan_registry::register_gucs();
}

/// Installs the FR-082 scan-token shared-memory request/startup hooks.  The
/// hooks become active only while PostgreSQL is processing
/// `shared_preload_libraries`; a later backend-local `LOAD` leaves the registry
/// unavailable and distributed serving must fail closed.
pub(crate) unsafe fn register_scan_registry_shared_memory() {
    unsafe { scan_registry::register_shared_memory_hooks() };
}

/// Installs the multi-node CustomScan provider + planner hook (from `_PG_init`).
pub(crate) fn register_custom_scan() {
    custom_scan::register_custom_scan();
}

pub(crate) unsafe fn register_build_gate_hooks() {
    unsafe { build_gate::register_build_gate_hooks() }
}

pub(crate) unsafe fn register_generation_cache_invalidation() {
    unsafe { generation_read::register_generation_cache_invalidation() }
}

/// Legacy v4 graph/page surfaces must never interpret a v5 logical control root
/// as an empty local graph.  Physical-generation callers resolve storage by
/// fingerprint through the generation catalog instead.
pub(super) fn require_legacy_local_storage(
    metadata: &page::DistannMetadataPage,
    caller: &str,
) -> Result<(), String> {
    if metadata.is_distributed_control() {
        return Err(format!(
            "EC_GENERATION_MISSING: {caller} requires a Published physical generation; the logical control index contains no graph storage"
        ));
    }
    Ok(())
}

pub(super) const ECDISTANN_DEFAULT_GRAPH_DEGREE: i32 = 32;
pub(super) const ECDISTANN_MIN_GRAPH_DEGREE: i32 = 4;
pub(super) const ECDISTANN_MAX_GRAPH_DEGREE: i32 = 256;

pub(super) const ECDISTANN_DEFAULT_BUILD_LIST_SIZE: i32 = 100;
pub(super) const ECDISTANN_MIN_BUILD_LIST_SIZE: i32 = 10;
pub(super) const ECDISTANN_MAX_BUILD_LIST_SIZE: i32 = 1000;

pub(super) const ECDISTANN_DEFAULT_ALPHA: f32 = 1.2;
pub(super) const ECDISTANN_MIN_ALPHA: f32 = 1.0;
pub(super) const ECDISTANN_MAX_ALPHA: f32 = 2.0;

/// ADR-085 D3: fixed head-index cap C, default 4096; the M0 C-sensitivity
/// bench cell informs whether this default is frozen.
pub(super) const ECDISTANN_DEFAULT_HEAD_INDEX_CAP: i32 = 4096;
pub(super) const ECDISTANN_MIN_HEAD_INDEX_CAP: i32 = 16;
pub(super) const ECDISTANN_MAX_HEAD_INDEX_CAP: i32 = 1_048_576;

/// FR-077 closure-overlap band. Measured at M1 (task-163 packet 002): the M0
/// provisional 0.1 starved boundary nodes of cross-shard edges and the
/// stitched build trailed monolithic recall by up to 0.06 at 100k (gap growing
/// with corpus size). Widening to 0.3 recovers parity — stitched recall@10
/// matches or exceeds monolithic across the operational search band (ef>=64)
/// at 50k/100k — at near-monolithic build cost, and beats wider bands (0.6/1.0)
/// on the recall/cost tradeoff. Only consumed when `build_shards >= 2`.
pub(super) const ECDISTANN_DEFAULT_CLOSURE_EPSILON: f32 = 0.3;
pub(super) const ECDISTANN_MIN_CLOSURE_EPSILON: f32 = 0.0;
pub(super) const ECDISTANN_MAX_CLOSURE_EPSILON: f32 = 1.0;

/// FR-077 build-shard count. `0` = auto (monolithic below ~20k rows, then
/// ~one shard per 25k up to a cap); `1` forces the monolithic fallback path;
/// `>=2` selects the sharded closure-overlap build + stitch. Default is `1`
/// (monolithic) so the single-node default matches the M0-measured behavior
/// until the M1 A/B promotes the sharded path.
pub(super) const ECDISTANN_DEFAULT_BUILD_SHARDS: i32 = 1;
pub(super) const ECDISTANN_MIN_BUILD_SHARDS: i32 = 0;
pub(super) const ECDISTANN_MAX_BUILD_SHARDS: i32 = 4096;

/// FR-081 BW default; Task 215's wide-beam/few-round candidate did not pass
/// the normal-release A/B gate, so the shipped default remains conservative.
pub(super) const ECDISTANN_DEFAULT_BEAM_WIDTH: i32 = 4;
/// The paper's distributed regime reaches BW=128. Keep headroom for the
/// surrounding sweep without making an unbounded GUC; NFR-019 still limits
/// each query by `beam_width * hop_rounds`.
pub(super) const ECDISTANN_MAX_BEAM_WIDTH: i32 = 256;
/// FR-081 L: maximum number of retained unexpanded candidates used to derive
/// the owner-side code-score floor.
pub(super) const ECDISTANN_DEFAULT_CANDIDATE_HEAP_LIMIT: i32 = 32;
pub(super) const ECDISTANN_MAX_CANDIDATE_HEAP_LIMIT: i32 = 4096;

/// FR-081 H default. H is the NFR-019 hard round cap, not the quality knob.
/// Task 215 evaluated H=8 with BW=64; the measured STOP leaves the shipped
/// default and SQL/session rollback at BW=4/H=100.
pub(super) const ECDISTANN_DEFAULT_HOP_ROUNDS: i32 = 100;
/// High ceiling so the bench sweep can match ec_diskann expansion budgets
/// (BW x H up to ~800 at the default BW=4); D9 early-exit terminates long
/// sweeps in practice.
pub(super) const ECDISTANN_MAX_HOP_ROUNDS: i32 = 256;

/// Result-heap size k for the FR-081 convergence early-exit; session GUC
/// because k is a query property, not an index property.
pub(super) const ECDISTANN_DEFAULT_TOP_K: i32 = 10;
pub(super) const ECDISTANN_MAX_TOP_K: i32 = 10_000;
