//! ec_spire access-method scaffold.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

mod assign;
mod build;
mod cost;
mod custom_scan;
mod diagnostics;
mod dml_frontdoor;
mod insert;
mod meta;
mod options;
mod page;
mod quantizer;
mod routine;
mod scan;
mod storage;
mod update;
mod vacuum;

use pgrx::{pg_sys, Spi};

use self::storage::SpireObjectReader;

pub(crate) use self::cost::{index_cost_snapshot, index_cost_tuning_snapshot};
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::custom_scan::custom_scan_dml_plan_private_copy_roundtrip_for_test;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::custom_scan::custom_scan_store_tuple_payload_json_for_test;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::custom_scan::{
    custom_scan_cleanup_counters_for_test, custom_scan_memory_context_snapshot_for_test,
    custom_scan_rescan_snapshot_for_test, custom_scan_reset_cleanup_counters_for_test,
    custom_scan_reset_rescan_snapshot_for_test,
};
pub(crate) use self::custom_scan::{
    custom_scan_index_eligibility_row, custom_scan_status_row, register_custom_scan,
};
pub(crate) use self::dml_frontdoor::{
    classify_dml_frontdoor_query, SpireDmlFrontdoorCustomScanMode, SpireDmlFrontdoorPkValuePlan,
    SpireDmlFrontdoorQueryContext,
};
pub(crate) use self::dml_frontdoor::{
    dml_frontdoor_bigint_pk_value_bytes, dml_frontdoor_hook_status_row,
    dml_frontdoor_pk_argument_from_replacement_decision,
    dml_frontdoor_pk_select_primitive_plan_expr_from_baserel,
    dml_frontdoor_primitive_invocation_from_plan,
    dml_frontdoor_primitive_plan_const_pk_value_bytes,
    dml_frontdoor_primitive_plan_expr_catalog_row, dml_frontdoor_primitive_plan_expr_from_baserel,
    dml_frontdoor_primitive_plan_from_replacement_decision,
    dml_frontdoor_primitive_plan_pk_value_bytes, dml_frontdoor_relation_context_cache_row,
    dml_frontdoor_relation_context_catalog_row, dml_frontdoor_relation_context_row,
    dml_frontdoor_replacement_decision_catalog_row, dml_frontdoor_target_relation_oid,
    register_dml_frontdoor_planner_hook,
};
pub use self::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireLocalStoreConfig,
    SpireLocalStoreDescriptor, SpireLocalStoreState, SpireManifestEntry, SpireObjectManifest,
    SpirePlacementDirectory, SpirePlacementEntry, SpirePlacementState,
    SPIRE_EPOCH_MANIFEST_ACTIVE_QUERY_COUNT_OFFSET, SPIRE_EPOCH_MANIFEST_BYTES,
    SPIRE_EPOCH_MANIFEST_CONSISTENCY_MODE_OFFSET, SPIRE_EPOCH_MANIFEST_EPOCH_OFFSET,
    SPIRE_EPOCH_MANIFEST_FORMAT_VERSION_OFFSET, SPIRE_EPOCH_MANIFEST_MAGIC,
    SPIRE_EPOCH_MANIFEST_MAGIC_OFFSET, SPIRE_EPOCH_MANIFEST_PUBLISHED_AT_MICROS_OFFSET,
    SPIRE_EPOCH_MANIFEST_RETAIN_UNTIL_MICROS_OFFSET, SPIRE_EPOCH_MANIFEST_STATE_OFFSET,
    SPIRE_LOCAL_STORE_CONFIG_FORMAT_VERSION_OFFSET, SPIRE_LOCAL_STORE_CONFIG_GENERATION_OFFSET,
    SPIRE_LOCAL_STORE_CONFIG_HEADER_BYTES, SPIRE_LOCAL_STORE_CONFIG_MAGIC,
    SPIRE_LOCAL_STORE_CONFIG_MAGIC_OFFSET, SPIRE_LOCAL_STORE_CONFIG_RESERVED_OFFSET,
    SPIRE_LOCAL_STORE_CONFIG_STORE_COUNT_OFFSET, SPIRE_LOCAL_STORE_DESCRIPTOR_BYTES,
    SPIRE_LOCAL_STORE_DESCRIPTOR_LOCAL_STORE_ID_OFFSET,
    SPIRE_LOCAL_STORE_DESCRIPTOR_RESERVED_OFFSET, SPIRE_LOCAL_STORE_DESCRIPTOR_STATE_OFFSET,
    SPIRE_LOCAL_STORE_DESCRIPTOR_STORE_RELID_OFFSET,
    SPIRE_LOCAL_STORE_DESCRIPTOR_TABLESPACE_OID_OFFSET, SPIRE_MANIFEST_ENTRY_BYTES,
    SPIRE_MANIFEST_ENTRY_EPOCH_OFFSET, SPIRE_MANIFEST_ENTRY_FORMAT_VERSION_OFFSET,
    SPIRE_MANIFEST_ENTRY_OBJECT_VERSION_OFFSET, SPIRE_MANIFEST_ENTRY_PID_OFFSET,
    SPIRE_MANIFEST_ENTRY_PLACEMENT_TID_OFFSET, SPIRE_MANIFEST_ENTRY_RESERVED_OFFSET,
    SPIRE_META_FORMAT_VERSION, SPIRE_OBJECT_MANIFEST_ENTRY_COUNT_OFFSET,
    SPIRE_OBJECT_MANIFEST_EPOCH_OFFSET, SPIRE_OBJECT_MANIFEST_FORMAT_VERSION_OFFSET,
    SPIRE_OBJECT_MANIFEST_HEADER_BYTES, SPIRE_OBJECT_MANIFEST_MAGIC,
    SPIRE_OBJECT_MANIFEST_MAGIC_OFFSET, SPIRE_OBJECT_MANIFEST_RESERVED_OFFSET,
    SPIRE_PLACEMENT_DIRECTORY_ENTRY_COUNT_OFFSET, SPIRE_PLACEMENT_DIRECTORY_EPOCH_OFFSET,
    SPIRE_PLACEMENT_DIRECTORY_FORMAT_VERSION_OFFSET, SPIRE_PLACEMENT_DIRECTORY_HEADER_BYTES,
    SPIRE_PLACEMENT_DIRECTORY_MAGIC, SPIRE_PLACEMENT_DIRECTORY_MAGIC_OFFSET,
    SPIRE_PLACEMENT_DIRECTORY_RESERVED_OFFSET, SPIRE_PLACEMENT_ENTRY_BYTES,
    SPIRE_PLACEMENT_ENTRY_EPOCH_OFFSET, SPIRE_PLACEMENT_ENTRY_FORMAT_VERSION_OFFSET,
    SPIRE_PLACEMENT_ENTRY_LOCAL_STORE_ID_OFFSET, SPIRE_PLACEMENT_ENTRY_NODE_ID_OFFSET,
    SPIRE_PLACEMENT_ENTRY_OBJECT_BYTES_OFFSET, SPIRE_PLACEMENT_ENTRY_OBJECT_TID_OFFSET,
    SPIRE_PLACEMENT_ENTRY_OBJECT_VERSION_OFFSET, SPIRE_PLACEMENT_ENTRY_PID_OFFSET,
    SPIRE_PLACEMENT_ENTRY_RESERVED_OFFSET, SPIRE_PLACEMENT_ENTRY_STATE_OFFSET,
    SPIRE_PLACEMENT_ENTRY_STORE_RELID_OFFSET,
};
pub use self::storage::{
    spire_assignment_row_gamma_offset, spire_assignment_row_heap_tid_offset,
    spire_assignment_row_payload_format_offset, spire_assignment_row_payload_len_offset,
    spire_assignment_row_payload_offset, spire_decode_delta_partition_object_fixture,
    spire_decode_leaf_partition_object_fixture, spire_decode_routing_partition_object_fixture,
    spire_decode_top_graph_partition_object_fixture, spire_leaf_v2_segment_gammas_offset,
    spire_leaf_v2_segment_heap_tids_offset, spire_leaf_v2_segment_payloads_offset,
    spire_leaf_v2_segment_vec_ids_offset, SpireAssignmentRowFixture,
    SpireDeltaPartitionObjectFixture, SpireLeafPartitionObjectFixture, SpirePartitionHeaderFixture,
    SpireRoutingPartitionObjectFixture, SpireTopGraphNodeFixture,
    SpireTopGraphPartitionObjectFixture, SPIRE_ASSIGNMENT_ROW_FIXED_PREFIX_BYTES,
    SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES, SPIRE_ASSIGNMENT_ROW_FLAGS_OFFSET,
    SPIRE_ASSIGNMENT_ROW_VEC_ID_LEN_OFFSET, SPIRE_ASSIGNMENT_ROW_VEC_ID_OFFSET,
    SPIRE_LEAF_V2_LOCAL_VEC_ID_STRIDE, SPIRE_LEAF_V2_META_BODY_BYTES,
    SPIRE_LEAF_V2_META_FIRST_SEGMENT_LOCATOR_OFFSET, SPIRE_LEAF_V2_META_FLAG,
    SPIRE_LEAF_V2_META_OBJECT_BYTES_TOTAL_OFFSET, SPIRE_LEAF_V2_META_PAYLOAD_FORMAT_OFFSET,
    SPIRE_LEAF_V2_META_PAYLOAD_STRIDE_OFFSET, SPIRE_LEAF_V2_META_RESERVED2_OFFSET,
    SPIRE_LEAF_V2_META_RESERVED_OFFSET, SPIRE_LEAF_V2_META_SEGMENT_COUNT_OFFSET,
    SPIRE_LEAF_V2_META_VEC_ID_KIND_OFFSET, SPIRE_LEAF_V2_META_VEC_ID_STRIDE_OFFSET,
    SPIRE_LEAF_V2_SEGMENT_FLAG, SPIRE_LEAF_V2_SEGMENT_FLAGS_OFFSET,
    SPIRE_LEAF_V2_SEGMENT_NEXT_LOCATOR_OFFSET, SPIRE_LEAF_V2_SEGMENT_NO_OFFSET,
    SPIRE_LEAF_V2_SEGMENT_PREFIX_BYTES, SPIRE_LEAF_V2_SEGMENT_ROW_BASE_OFFSET,
    SPIRE_LEAF_V2_SEGMENT_ROW_COUNT_OFFSET, SPIRE_PARTITION_OBJECT_ASSIGNMENT_COUNT_OFFSET,
    SPIRE_PARTITION_OBJECT_CHILD_COUNT_OFFSET, SPIRE_PARTITION_OBJECT_FLAGS_OFFSET,
    SPIRE_PARTITION_OBJECT_FORMAT_VERSION_OFFSET, SPIRE_PARTITION_OBJECT_FORMAT_VERSION_V1,
    SPIRE_PARTITION_OBJECT_FORMAT_VERSION_V2, SPIRE_PARTITION_OBJECT_HEADER_BYTES,
    SPIRE_PARTITION_OBJECT_KIND_OFFSET, SPIRE_PARTITION_OBJECT_LEVEL_OFFSET,
    SPIRE_PARTITION_OBJECT_MAGIC, SPIRE_PARTITION_OBJECT_MAGIC_OFFSET,
    SPIRE_PARTITION_OBJECT_OBJECT_VERSION_OFFSET, SPIRE_PARTITION_OBJECT_PARENT_PID_OFFSET,
    SPIRE_PARTITION_OBJECT_PID_OFFSET, SPIRE_PARTITION_OBJECT_PUBLISHED_EPOCH_BACKREF_OFFSET,
    SPIRE_PARTITION_OBJECT_RESERVED_OFFSET, SPIRE_PARTITION_OBJECT_V2_CHAIN_META_BODY_BYTES,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_META_DIMENSIONS_OFFSET,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_META_FIRST_SEGMENT_LOCATOR_OFFSET,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_META_FLAG,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_META_OBJECT_BYTES_TOTAL_OFFSET,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_META_RESERVED_OFFSET,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_META_SEGMENT_COUNT_OFFSET,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_BYTE_BASE_OFFSET,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_FLAG,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_NEXT_LOCATOR_OFFSET,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_NO_OFFSET,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_PAYLOAD_OFFSET,
    SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_PREFIX_BYTES,
};
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::vacuum::{
    debug_spire_vacuum_bulkdelete_heap_tids, debug_spire_vacuum_remove_heap_tids,
};

pub(super) const EC_SPIRE_DEFAULT_NLISTS: i32 = 0;
pub(super) const EC_SPIRE_MIN_NLISTS: i32 = 0;
pub(super) const EC_SPIRE_MAX_NLISTS: i32 = 1_000_000;
pub(super) const EC_SPIRE_DEFAULT_RECURSIVE_FANOUT: i32 = 0;
pub(super) const EC_SPIRE_MIN_RECURSIVE_FANOUT: i32 = 0;
pub(super) const EC_SPIRE_MAX_RECURSIVE_FANOUT: i32 = 1_000_000;
pub(super) const EC_SPIRE_DEFAULT_LOCAL_STORE_COUNT: i32 = 1;
pub(super) const EC_SPIRE_MIN_LOCAL_STORE_COUNT: i32 = 1;
pub(super) const EC_SPIRE_MAX_LOCAL_STORE_COUNT: i32 = 16;
pub(super) const EC_SPIRE_DEFAULT_BOUNDARY_REPLICA_COUNT: i32 = 0;
pub(super) const EC_SPIRE_MIN_BOUNDARY_REPLICA_COUNT: i32 = 0;
pub(super) const EC_SPIRE_MAX_BOUNDARY_REPLICA_COUNT: i32 = 8;
pub(super) const EC_SPIRE_DEFAULT_NPROBE: i32 = 0;
pub(super) const EC_SPIRE_MIN_NPROBE: i32 = 0;
pub(super) const EC_SPIRE_MAX_NPROBE: i32 = 1_000_000;
pub(super) const EC_SPIRE_DEFAULT_RERANK_WIDTH: i32 = 0;
pub(super) const EC_SPIRE_MIN_RERANK_WIDTH: i32 = 0;
pub(super) const EC_SPIRE_MAX_RERANK_WIDTH: i32 = 10_000_000;
pub(super) const EC_SPIRE_DEFAULT_MAX_CANDIDATE_ROWS: i32 = 0;
pub(super) const EC_SPIRE_MIN_MAX_CANDIDATE_ROWS: i32 = 0;
pub(super) const EC_SPIRE_MAX_MAX_CANDIDATE_ROWS: i32 = 10_000_000;
pub(super) const EC_SPIRE_DEFAULT_TRAINING_SAMPLE_ROWS: i32 = 0;
pub(super) const EC_SPIRE_MIN_TRAINING_SAMPLE_ROWS: i32 = 0;
pub(super) const EC_SPIRE_MAX_TRAINING_SAMPLE_ROWS: i32 = 10_000_000;
pub(super) const EC_SPIRE_DEFAULT_SEED: i32 = 42;
pub(super) const EC_SPIRE_MIN_SEED: i32 = 0;
pub(super) const EC_SPIRE_MAX_SEED: i32 = i32::MAX;

const SPIRE_LEAF_SPLIT_AVERAGE_MULTIPLIER: u64 = 4;
const SPIRE_LEAF_SPLIT_MIN_ASSIGNMENTS: u64 = 32;
const SPIRE_LEAF_MERGE_AVERAGE_DIVISOR: u64 = 4;
pub(super) const EC_SPIRE_DEFAULT_PQ_GROUP_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MIN_PQ_GROUP_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MAX_PQ_GROUP_SIZE: i32 = 32;
pub(super) const EC_SPIRE_DEFAULT_TOP_GRAPH_ENABLED: i32 = 0;
pub(super) const EC_SPIRE_MIN_TOP_GRAPH_ENABLED: i32 = 0;
pub(super) const EC_SPIRE_MAX_TOP_GRAPH_ENABLED: i32 = 1;
pub(super) const EC_SPIRE_DEFAULT_TOP_GRAPH_DEGREE: i32 = 32;
pub(super) const EC_SPIRE_MIN_TOP_GRAPH_DEGREE: i32 = 1;
pub(super) const EC_SPIRE_MAX_TOP_GRAPH_DEGREE: i32 = 1024;
pub(super) const EC_SPIRE_DEFAULT_TOP_GRAPH_BUILD_LIST_SIZE: i32 = 100;
pub(super) const EC_SPIRE_MIN_TOP_GRAPH_BUILD_LIST_SIZE: i32 = 1;
pub(super) const EC_SPIRE_MAX_TOP_GRAPH_BUILD_LIST_SIZE: i32 = 100_000;
pub(super) const EC_SPIRE_DEFAULT_TOP_GRAPH_ALPHA: f32 = 1.2;
pub(super) const EC_SPIRE_MIN_TOP_GRAPH_ALPHA: f32 = 1.0;
pub(super) const EC_SPIRE_MAX_TOP_GRAPH_ALPHA: f32 = 10.0;
pub(super) const EC_SPIRE_DEFAULT_TOP_GRAPH_SEARCH_LIST_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MIN_TOP_GRAPH_SEARCH_LIST_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MAX_TOP_GRAPH_SEARCH_LIST_SIZE: i32 = 1_000_000;

pub(super) const SPIRE_PUBLISH_LOCK_MODE: pg_sys::LOCKMODE =
    pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE;

include!("coordinator/lifecycle.rs");
include!("coordinator/types.rs");
include!("coordinator/remote_candidates/mod.rs");
include!("coordinator/diagnostics.rs");
include!("coordinator/hierarchy_shape.rs");
include!("coordinator/snapshots.rs");
include!("coordinator/maintenance.rs");
include!("coordinator/hierarchy_snapshots.rs");
include!("coordinator/debug.rs");
include!("coordinator/tests.rs");
