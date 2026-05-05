//! ec_spire access-method scaffold.

use std::collections::{HashMap, HashSet};

mod assign;
mod build;
mod cost;
mod diagnostics;
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

use pgrx::pg_sys;

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
pub(super) const EC_SPIRE_DEFAULT_NPROBE: i32 = 0;
pub(super) const EC_SPIRE_MIN_NPROBE: i32 = 0;
pub(super) const EC_SPIRE_MAX_NPROBE: i32 = 1_000_000;
pub(super) const EC_SPIRE_DEFAULT_RERANK_WIDTH: i32 = 0;
pub(super) const EC_SPIRE_MIN_RERANK_WIDTH: i32 = 0;
pub(super) const EC_SPIRE_MAX_RERANK_WIDTH: i32 = 10_000_000;
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

pub(super) const SPIRE_PUBLISH_LOCK_MODE: pg_sys::LOCKMODE =
    pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE;

pub(super) struct SpireRelationLockGuard {
    relid: pg_sys::Oid,
    lockmode: pg_sys::LOCKMODE,
}

impl Drop for SpireRelationLockGuard {
    fn drop(&mut self) {
        unsafe { pg_sys::UnlockRelationOid(self.relid, self.lockmode) };
    }
}

pub(super) unsafe fn lock_publish_relation(
    index_relation: pg_sys::Relation,
) -> SpireRelationLockGuard {
    // Callers hold an open Relation for the guard lifetime. Capture the relid
    // before locking and unlock by relid so Drop never dereferences the pointer.
    let relid = unsafe { (*index_relation).rd_id };
    unsafe { pg_sys::LockRelationOid(relid, SPIRE_PUBLISH_LOCK_MODE) };
    SpireRelationLockGuard {
        relid,
        lockmode: SPIRE_PUBLISH_LOCK_MODE,
    }
}

struct SpireHeapRelationGuard {
    relation: pg_sys::Relation,
}

impl SpireHeapRelationGuard {
    unsafe fn open_for_index(index_relation: pg_sys::Relation) -> Result<Self, String> {
        let index_oid = unsafe { (*index_relation).rd_id };
        let heap_oid = unsafe { pg_sys::IndexGetRelation(index_oid, false) };
        if heap_oid == pg_sys::InvalidOid {
            return Err("ec_spire maintenance could not resolve heap relation".to_owned());
        }
        let relation =
            unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        if relation.is_null() {
            return Err("ec_spire maintenance failed to open heap relation".to_owned());
        }
        Ok(Self { relation })
    }

    fn relation(&self) -> pg_sys::Relation {
        self.relation
    }
}

impl Drop for SpireHeapRelationGuard {
    fn drop(&mut self) {
        unsafe { pg_sys::table_close(self.relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }
}

struct SpireHeapSlotGuard {
    slot: *mut pg_sys::TupleTableSlot,
}

impl SpireHeapSlotGuard {
    unsafe fn new(heap_relation: pg_sys::Relation) -> Result<Self, String> {
        let slot = unsafe {
            pg_sys::MakeSingleTupleTableSlot(
                (*heap_relation).rd_att,
                pg_sys::table_slot_callbacks(heap_relation),
            )
        };
        if slot.is_null() {
            return Err("ec_spire maintenance failed to allocate a heap tuple slot".to_owned());
        }
        Ok(Self { slot })
    }

    fn as_ptr(&self) -> *mut pg_sys::TupleTableSlot {
        self.slot
    }
}

impl Drop for SpireHeapSlotGuard {
    fn drop(&mut self) {
        unsafe { pg_sys::ExecDropSingleTupleTableSlot(self.slot) };
    }
}

unsafe fn active_spire_maintenance_snapshot() -> Result<pg_sys::Snapshot, String> {
    let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot.is_null() {
        return Err("ec_spire maintenance requires an active heap snapshot".to_owned());
    }
    Ok(snapshot)
}

pub(crate) fn register_gucs() {
    options::register_gucs();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireActiveSnapshotDiagnostics {
    pub(crate) active_epoch: u64,
    pub(crate) next_pid: u64,
    pub(crate) next_local_vec_seq: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) object_count: u64,
    pub(crate) placement_count: u64,
    pub(crate) local_store_count: u64,
    pub(crate) available_placement_count: u64,
    pub(crate) stale_placement_count: u64,
    pub(crate) unavailable_placement_count: u64,
    pub(crate) skipped_placement_count: u64,
    pub(crate) root_object_count: u64,
    pub(crate) internal_object_count: u64,
    pub(crate) leaf_object_count: u64,
    pub(crate) delta_object_count: u64,
    pub(crate) routing_child_count: u64,
    pub(crate) leaf_assignment_count: u64,
    pub(crate) delta_assignment_count: u64,
    pub(crate) available_object_bytes: u64,
    pub(crate) routing_object_bytes: u64,
    pub(crate) leaf_object_bytes: u64,
    pub(crate) delta_object_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexAllocatorSnapshot {
    pub(crate) active_epoch: u64,
    pub(crate) warn_within: u64,
    pub(crate) next_pid: u64,
    pub(crate) remaining_pid_allocations: u64,
    pub(crate) pid_near_exhaustion: bool,
    pub(crate) next_local_vec_seq: u64,
    pub(crate) remaining_local_vec_id_allocations: u64,
    pub(crate) local_vec_id_near_exhaustion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexOptionsSnapshot {
    pub(crate) nlists: i32,
    pub(crate) recursive_fanout: i32,
    pub(crate) recursive_build_enabled: bool,
    pub(crate) active_leaf_count: u32,
    pub(crate) relation_nprobe: i32,
    pub(crate) session_nprobe: Option<i32>,
    pub(crate) effective_nprobe: u32,
    pub(crate) effective_nprobe_source: &'static str,
    pub(crate) relation_rerank_width: i32,
    pub(crate) session_rerank_width: Option<i32>,
    pub(crate) effective_rerank_width: i32,
    pub(crate) effective_rerank_width_source: &'static str,
    pub(crate) training_sample_rows: i32,
    pub(crate) seed: i32,
    pub(crate) pq_group_size: i32,
    pub(crate) storage_format: &'static str,
    pub(crate) assignment_payload_format: &'static str,
    pub(crate) assignment_payload_scannable: bool,
    pub(crate) assignment_payload_status: &'static str,
    pub(crate) assignment_payload_recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexHealthSnapshot {
    pub(crate) active_epoch: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) status: &'static str,
    pub(crate) healthy: bool,
    pub(crate) recommendation: &'static str,
    pub(crate) compaction_recommended: bool,
    pub(crate) object_count: u64,
    pub(crate) leaf_assignment_count: u64,
    pub(crate) delta_assignment_count: u64,
    pub(crate) delta_object_count: u64,
    pub(crate) available_placement_count: u64,
    pub(crate) stale_placement_count: u64,
    pub(crate) unavailable_placement_count: u64,
    pub(crate) skipped_placement_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexRelationStorageSnapshot {
    pub(crate) active_epoch: u64,
    pub(crate) relation_block_count: u64,
    pub(crate) relation_object_tuple_count: u64,
    pub(crate) relation_object_tuple_bytes: u64,
    pub(crate) active_referenced_tuple_count: u64,
    pub(crate) active_referenced_tuple_bytes: u64,
    pub(crate) cleanup_candidate_tuple_count: u64,
    pub(crate) cleanup_candidate_tuple_bytes: u64,
    pub(crate) physical_cleanup_supported: bool,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexScanSanitySnapshot {
    pub(crate) active_epoch: u64,
    pub(crate) active_leaf_count: u32,
    pub(crate) effective_nprobe: u32,
    pub(crate) effective_nprobe_source: &'static str,
    pub(crate) exact_leaf_coverage: bool,
    pub(crate) effective_rerank_width: i32,
    pub(crate) effective_rerank_width_source: &'static str,
    pub(crate) full_frontier_rerank: bool,
    pub(crate) recall_sanity_status: &'static str,
    pub(crate) latency_risk_status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexEpochSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) epoch: u64,
    pub(crate) state: &'static str,
    pub(crate) consistency_mode: &'static str,
    pub(crate) published_at_micros: i64,
    pub(crate) retain_until_micros: i64,
    pub(crate) active_query_count: u64,
    pub(crate) manifest_block: u32,
    pub(crate) manifest_offset: u16,
    pub(crate) is_active_root_manifest: bool,
    pub(crate) cleanup_eligible_now: bool,
    pub(crate) cleanup_blocked_reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexLeafSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) leaf_pid: u64,
    pub(crate) parent_pid: u64,
    pub(crate) object_version: u64,
    pub(crate) node_id: u32,
    pub(crate) local_store_id: u32,
    pub(crate) placement_state: &'static str,
    pub(crate) base_assignment_count: u64,
    pub(crate) delta_object_count: u64,
    pub(crate) delta_insert_assignment_count: u64,
    pub(crate) delta_delete_assignment_count: u64,
    pub(crate) effective_assignment_count: u64,
    pub(crate) split_assignment_threshold: u64,
    pub(crate) merge_assignment_threshold: u64,
    pub(crate) split_recommended: bool,
    pub(crate) merge_recommended: bool,
    pub(crate) maintenance_action: &'static str,
    pub(crate) maintenance_reason: &'static str,
    pub(crate) leaf_object_bytes: u64,
    pub(crate) delta_object_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexMaintenancePlanSnapshot {
    pub(crate) active_epoch: u64,
    pub(crate) planner_status: &'static str,
    pub(crate) planned_action: &'static str,
    pub(crate) planned_reason: &'static str,
    pub(crate) replaced_parent_pid: u64,
    pub(crate) affected_leaf_pids: Vec<u64>,
    pub(crate) replacement_leaf_count: u64,
    pub(crate) replacement_leaf_pids: Vec<u64>,
    pub(crate) publish_epoch: u64,
    pub(crate) next_pid: u64,
    pub(crate) next_local_vec_seq: u64,
    pub(crate) planner_message: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexMaintenanceRunResult {
    pub(crate) active_epoch_before: u64,
    pub(crate) active_epoch_after: u64,
    pub(crate) maintenance_status: &'static str,
    pub(crate) planned_action: &'static str,
    pub(crate) planned_reason: &'static str,
    pub(crate) replaced_parent_pid: u64,
    pub(crate) affected_leaf_pids: Vec<u64>,
    pub(crate) replacement_leaf_count: u64,
    pub(crate) replacement_leaf_pids: Vec<u64>,
    pub(crate) publish_epoch: u64,
    pub(crate) next_pid: u64,
    pub(crate) next_local_vec_seq: u64,
    pub(crate) published: bool,
    pub(crate) maintenance_message: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireScheduledReplacementObjectVersionPlan {
    parent_object_version: u64,
    leaf_object_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexInsertDebtSnapshot {
    pub(crate) active_epoch: u64,
    pub(crate) active_leaf_count: u64,
    pub(crate) leaf_count_with_deltas: u64,
    pub(crate) delta_object_count: u64,
    pub(crate) delta_insert_assignment_count: u64,
    pub(crate) max_delta_objects_per_leaf: u64,
    pub(crate) insert_batching_supported: bool,
    pub(crate) batching_recommended: bool,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexHierarchySnapshot {
    pub(crate) active_epoch: u64,
    pub(crate) root_pid: u64,
    pub(crate) root_level: u16,
    pub(crate) max_observed_level: u16,
    pub(crate) hierarchy_depth: u16,
    pub(crate) routing_object_count: u64,
    pub(crate) root_routing_object_count: u64,
    pub(crate) internal_routing_object_count: u64,
    pub(crate) leaf_object_count: u64,
    pub(crate) delta_object_count: u64,
    pub(crate) centroid_dimensions: u16,
    pub(crate) root_child_count: u64,
    pub(crate) distinct_leaf_parent_count: u64,
    pub(crate) recursive_routing_supported: bool,
    pub(crate) per_level_nprobe_supported: bool,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexObjectSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) pid: u64,
    pub(crate) object_kind: &'static str,
    pub(crate) object_version: u64,
    pub(crate) published_epoch_backref: u64,
    pub(crate) level: u16,
    pub(crate) parent_pid: u64,
    pub(crate) child_count: u64,
    pub(crate) assignment_count: u64,
    pub(crate) node_id: u32,
    pub(crate) local_store_id: u32,
    pub(crate) store_relid: u32,
    pub(crate) placement_state: &'static str,
    pub(crate) object_bytes: u64,
    pub(crate) object_readable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexDeltaSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) delta_pid: u64,
    pub(crate) parent_leaf_pid: u64,
    pub(crate) object_version: u64,
    pub(crate) published_epoch_backref: u64,
    pub(crate) node_id: u32,
    pub(crate) local_store_id: u32,
    pub(crate) store_relid: u32,
    pub(crate) placement_state: &'static str,
    pub(crate) assignment_count: u64,
    pub(crate) insert_assignment_count: u64,
    pub(crate) delete_assignment_count: u64,
    pub(crate) object_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexPlacementSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) local_store_id: u32,
    pub(crate) placement_count: u64,
    pub(crate) available_placement_count: u64,
    pub(crate) stale_placement_count: u64,
    pub(crate) unavailable_placement_count: u64,
    pub(crate) skipped_placement_count: u64,
    pub(crate) object_count: u64,
    pub(crate) root_object_count: u64,
    pub(crate) internal_object_count: u64,
    pub(crate) leaf_object_count: u64,
    pub(crate) delta_object_count: u64,
    pub(crate) routing_child_count: u64,
    pub(crate) assignment_count: u64,
    pub(crate) placement_object_bytes: u64,
    pub(crate) available_object_bytes: u64,
    pub(crate) routing_object_bytes: u64,
    pub(crate) leaf_object_bytes: u64,
    pub(crate) delta_object_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexScanPlacementSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) effective_nprobe: u32,
    pub(crate) effective_nprobe_source: &'static str,
    pub(crate) effective_rerank_width: u64,
    pub(crate) effective_rerank_width_source: &'static str,
    pub(crate) node_id: u32,
    pub(crate) local_store_id: u32,
    pub(crate) scanned_pid_count: u64,
    pub(crate) leaf_pid_count: u64,
    pub(crate) delta_pid_count: u64,
    pub(crate) candidate_row_count: u64,
    pub(crate) leaf_candidate_row_count: u64,
    pub(crate) delta_candidate_row_count: u64,
    pub(crate) delete_delta_row_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexRootRoutingSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) root_pid: u64,
    pub(crate) root_object_version: u64,
    pub(crate) root_level: u16,
    pub(crate) root_child_count: u64,
    pub(crate) centroid_dimensions: u16,
    pub(crate) centroid_index: u32,
    pub(crate) child_pid: u64,
    pub(crate) child_kind: &'static str,
    pub(crate) child_object_version: u64,
    pub(crate) child_level: u16,
    pub(crate) child_parent_pid: u64,
    pub(crate) child_assignment_count: u64,
    pub(crate) child_node_id: u32,
    pub(crate) child_local_store_id: u32,
    pub(crate) child_store_relid: u32,
    pub(crate) child_placement_state: &'static str,
    pub(crate) child_object_bytes: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireIndexRoutingCentroidSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) parent_pid: u64,
    pub(crate) parent_kind: &'static str,
    pub(crate) parent_object_version: u64,
    pub(crate) parent_level: u16,
    pub(crate) parent_child_count: u64,
    pub(crate) centroid_dimensions: u16,
    pub(crate) centroid_index: u32,
    pub(crate) child_pid: u64,
    pub(crate) child_kind: &'static str,
    pub(crate) child_object_version: u64,
    pub(crate) child_level: u16,
    pub(crate) child_parent_pid: u64,
    pub(crate) child_assignment_count: u64,
    pub(crate) child_node_id: u32,
    pub(crate) child_local_store_id: u32,
    pub(crate) child_store_relid: u32,
    pub(crate) child_placement_state: &'static str,
    pub(crate) child_object_bytes: u64,
    pub(crate) centroid: Vec<f32>,
}

impl SpireActiveSnapshotDiagnostics {
    fn empty(root_control: meta::SpireRootControlState) -> Self {
        Self {
            active_epoch: root_control.active_epoch,
            next_pid: root_control.next_pid,
            next_local_vec_seq: root_control.next_local_vec_seq,
            consistency_mode: "none",
            object_count: 0,
            placement_count: 0,
            local_store_count: 0,
            available_placement_count: 0,
            stale_placement_count: 0,
            unavailable_placement_count: 0,
            skipped_placement_count: 0,
            root_object_count: 0,
            internal_object_count: 0,
            leaf_object_count: 0,
            delta_object_count: 0,
            routing_child_count: 0,
            leaf_assignment_count: 0,
            delta_assignment_count: 0,
            available_object_bytes: 0,
            routing_object_bytes: 0,
            leaf_object_bytes: 0,
            delta_object_bytes: 0,
        }
    }
}

fn health_snapshot_from_diagnostics(
    diagnostics: &SpireActiveSnapshotDiagnostics,
) -> SpireIndexHealthSnapshot {
    let has_no_active_epoch = diagnostics.active_epoch == 0;
    let (status, healthy, recommendation, compaction_recommended) = if has_no_active_epoch {
        (
            "empty",
            true,
            "build or insert rows to publish the first SPIRE epoch",
            false,
        )
    } else if diagnostics.unavailable_placement_count > 0 {
        (
            "unavailable_placements",
            false,
            "restore unavailable local placements before relying on this index",
            false,
        )
    } else if diagnostics.stale_placement_count > 0 {
        (
            "stale_placements",
            false,
            "publish a cleanup epoch to remove stale placements",
            false,
        )
    } else if diagnostics.skipped_placement_count > 0 {
        (
            "skipped_placements",
            false,
            "inspect skipped placements before enabling degraded reads",
            false,
        )
    } else if diagnostics.delta_object_count > 0 {
        (
            "maintenance_recommended",
            true,
            "run VACUUM to compact active delta objects into V2 base leaves",
            true,
        )
    } else if diagnostics.consistency_mode == "degraded" {
        (
            "degraded_consistency",
            true,
            "verify degraded-read policy before relying on strict local semantics",
            false,
        )
    } else {
        ("ok", true, "none", false)
    };

    SpireIndexHealthSnapshot {
        active_epoch: diagnostics.active_epoch,
        consistency_mode: diagnostics.consistency_mode,
        status,
        healthy,
        recommendation,
        compaction_recommended,
        object_count: diagnostics.object_count,
        leaf_assignment_count: diagnostics.leaf_assignment_count,
        delta_assignment_count: diagnostics.delta_assignment_count,
        delta_object_count: diagnostics.delta_object_count,
        available_placement_count: diagnostics.available_placement_count,
        stale_placement_count: diagnostics.stale_placement_count,
        unavailable_placement_count: diagnostics.unavailable_placement_count,
        skipped_placement_count: diagnostics.skipped_placement_count,
    }
}

fn assignment_payload_format_name(format: quantizer::SpireAssignmentPayloadFormat) -> &'static str {
    match format {
        quantizer::SpireAssignmentPayloadFormat::TurboQuant => "turboquant",
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => "pq_fastscan",
        quantizer::SpireAssignmentPayloadFormat::RaBitQ => "rabitq",
    }
}

const SPIRE_ASSIGNMENT_PAYLOAD_STATUS_SUPPORTED: &str = "supported";
const SPIRE_ASSIGNMENT_PAYLOAD_STATUS_DEFERRED_MODEL_METADATA: &str = "deferred_model_metadata";

fn assignment_payload_scannability(
    format: quantizer::SpireAssignmentPayloadFormat,
) -> (bool, &'static str, &'static str) {
    match format {
        quantizer::SpireAssignmentPayloadFormat::TurboQuant
        | quantizer::SpireAssignmentPayloadFormat::RaBitQ => (
            true,
            SPIRE_ASSIGNMENT_PAYLOAD_STATUS_SUPPORTED,
            "format can be used for current SPIRE scans",
        ),
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => (
            false,
            SPIRE_ASSIGNMENT_PAYLOAD_STATUS_DEFERRED_MODEL_METADATA,
            "persist grouped-PQ model metadata before using pq_fastscan with SPIRE",
        ),
    }
}

fn count_snapshot_options_leaf_pids(
    snapshot: &meta::SpirePublishedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
    recursive_build_enabled: bool,
) -> Result<u32, String> {
    if recursive_build_enabled {
        scan::count_snapshot_recursive_leaf_pids(snapshot, object_store)
    } else {
        scan::count_snapshot_single_level_leaf_pids(snapshot, object_store)
    }
}

fn scan_sanity_status(
    active_epoch: u64,
    exact_leaf_coverage: bool,
    full_frontier_rerank: bool,
) -> (&'static str, &'static str, &'static str) {
    if active_epoch == 0 {
        return (
            "empty",
            "none",
            "build or insert rows to publish the first SPIRE epoch",
        );
    }
    if exact_leaf_coverage && full_frontier_rerank {
        return (
            "exact_leaf_and_frontier_coverage",
            "full_scan",
            "use this configuration only for recall sanity checks or small indexes",
        );
    }
    if exact_leaf_coverage {
        return (
            "exact_leaf_coverage_bounded_rerank",
            "bounded_rerank",
            "set rerank_width = 0 for full-frontier exact recall sanity checks",
        );
    }
    (
        "approximate_leaf_coverage",
        "bounded_leaf_probe",
        "increase nprobe to active_leaf_count for exact leaf coverage sanity checks",
    )
}

fn consistency_mode_name(mode: meta::SpireConsistencyMode) -> &'static str {
    match mode {
        meta::SpireConsistencyMode::Strict => "strict",
        meta::SpireConsistencyMode::Degraded => "degraded",
    }
}

fn epoch_state_name(state: meta::SpireEpochState) -> &'static str {
    match state {
        meta::SpireEpochState::Building => "building",
        meta::SpireEpochState::Published => "published",
        meta::SpireEpochState::Retired => "retired",
        meta::SpireEpochState::Failed => "failed",
    }
}

fn epoch_cleanup_blocked_reason(
    manifest: &meta::SpireEpochManifest,
    now_micros: i64,
    is_active_root_manifest: bool,
    retained_retired: bool,
    cleanup_eligible_now: bool,
) -> &'static str {
    if cleanup_eligible_now {
        return "cleanup_eligible";
    }
    if is_active_root_manifest {
        return "active_root_manifest";
    }
    match manifest.state {
        meta::SpireEpochState::Building | meta::SpireEpochState::Published => {
            "state_not_cleanup_eligible"
        }
        meta::SpireEpochState::Retired if manifest.active_query_count > 0 => "active_queries",
        meta::SpireEpochState::Retired if retained_retired => "retained_retired_epoch",
        meta::SpireEpochState::Retired | meta::SpireEpochState::Failed
            if now_micros < manifest.retain_until_micros =>
        {
            "retention_window"
        }
        meta::SpireEpochState::Retired | meta::SpireEpochState::Failed => "cleanup_plan_retained",
    }
}

fn epoch_snapshot_rows_from_manifests(
    root_control: meta::SpireRootControlState,
    mut manifests: Vec<(crate::storage::page::ItemPointer, meta::SpireEpochManifest)>,
    now_micros: i64,
) -> Result<Vec<SpireIndexEpochSnapshotRow>, String> {
    manifests.sort_by_key(|(tid, manifest)| (manifest.epoch, tid.block_number, tid.offset_number));

    let mut latest_manifest_tid_by_epoch = HashMap::new();
    for (tid, manifest) in &manifests {
        latest_manifest_tid_by_epoch
            .entry(manifest.epoch)
            .and_modify(|latest_tid: &mut crate::storage::page::ItemPointer| {
                if (tid.block_number, tid.offset_number)
                    > (latest_tid.block_number, latest_tid.offset_number)
                {
                    *latest_tid = *tid;
                }
            })
            .or_insert(*tid);
    }
    let latest_manifests = manifests
        .iter()
        .filter_map(|(tid, manifest)| {
            let latest_tid = latest_manifest_tid_by_epoch.get(&manifest.epoch)?;
            if latest_tid == tid {
                Some(*manifest)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let cleanup_plan =
        meta::plan_epoch_cleanup(&latest_manifests, root_control.active_epoch, now_micros)?;
    let cleanup_epochs: HashSet<u64> = cleanup_plan.cleanup_epochs.into_iter().collect();
    let retained_retired_epochs: HashSet<u64> =
        cleanup_plan.retained_retired_epochs.into_iter().collect();

    Ok(manifests
        .into_iter()
        .map(|(tid, manifest)| {
            let is_latest_manifest = latest_manifest_tid_by_epoch
                .get(&manifest.epoch)
                .is_some_and(|latest_tid| latest_tid == &tid);
            let is_active_root_manifest = root_control.active_epoch == manifest.epoch
                && root_control.epoch_manifest_tid == tid;
            let cleanup_eligible_now =
                is_latest_manifest && cleanup_epochs.contains(&manifest.epoch);
            let retained_retired = retained_retired_epochs.contains(&manifest.epoch);
            let cleanup_blocked_reason = if is_active_root_manifest {
                "active_root_manifest"
            } else if is_latest_manifest {
                epoch_cleanup_blocked_reason(
                    &manifest,
                    now_micros,
                    false,
                    retained_retired,
                    cleanup_eligible_now,
                )
            } else {
                "superseded_manifest"
            };
            SpireIndexEpochSnapshotRow {
                active_epoch: root_control.active_epoch,
                epoch: manifest.epoch,
                state: epoch_state_name(manifest.state),
                consistency_mode: consistency_mode_name(manifest.consistency_mode),
                published_at_micros: manifest.published_at_micros,
                retain_until_micros: manifest.retain_until_micros,
                active_query_count: manifest.active_query_count,
                manifest_block: tid.block_number,
                manifest_offset: tid.offset_number,
                is_active_root_manifest,
                cleanup_eligible_now,
                cleanup_blocked_reason,
            }
        })
        .collect())
}

fn leaf_maintenance_thresholds(effective_total: u64, leaf_count: u64) -> (u64, u64) {
    if leaf_count == 0 {
        return (0, 0);
    }
    let average = effective_total.div_ceil(leaf_count);
    let split_threshold = average
        .saturating_mul(SPIRE_LEAF_SPLIT_AVERAGE_MULTIPLIER)
        .max(SPIRE_LEAF_SPLIT_MIN_ASSIGNMENTS);
    let merge_threshold = average / SPIRE_LEAF_MERGE_AVERAGE_DIVISOR;
    (split_threshold, merge_threshold)
}

fn leaf_maintenance_labels(
    effective_assignment_count: u64,
    split_threshold: u64,
    merge_threshold: u64,
) -> (bool, bool, &'static str, &'static str) {
    if effective_assignment_count >= split_threshold && split_threshold > 0 {
        return (
            true,
            false,
            "split_candidate",
            "effective_assignments_at_or_above_split_threshold",
        );
    }
    if effective_assignment_count <= merge_threshold {
        return (
            false,
            true,
            "merge_candidate",
            "effective_assignments_at_or_below_merge_threshold",
        );
    }
    (false, false, "none", "within_distribution_thresholds")
}

fn apply_leaf_snapshot_base_row(
    rows_by_leaf_pid: &mut HashMap<u64, SpireIndexLeafSnapshotRow>,
    active_epoch: u64,
    header: &storage::SpirePartitionObjectHeader,
    placement: &meta::SpirePlacementEntry,
) {
    let row = rows_by_leaf_pid
        .entry(header.pid)
        .or_insert_with(|| SpireIndexLeafSnapshotRow {
            active_epoch,
            leaf_pid: header.pid,
            parent_pid: header.parent_pid,
            object_version: header.object_version,
            node_id: placement.node_id,
            local_store_id: placement.local_store_id,
            placement_state: placement_state_name(placement.state),
            base_assignment_count: 0,
            delta_object_count: 0,
            delta_insert_assignment_count: 0,
            delta_delete_assignment_count: 0,
            effective_assignment_count: 0,
            split_assignment_threshold: 0,
            merge_assignment_threshold: 0,
            split_recommended: false,
            merge_recommended: false,
            maintenance_action: "none",
            maintenance_reason: "not_evaluated",
            leaf_object_bytes: 0,
            delta_object_bytes: 0,
        });

    row.active_epoch = active_epoch;
    row.leaf_pid = header.pid;
    row.parent_pid = header.parent_pid;
    row.object_version = header.object_version;
    row.node_id = placement.node_id;
    row.local_store_id = placement.local_store_id;
    row.placement_state = placement_state_name(placement.state);
    row.base_assignment_count = u64::from(header.assignment_count);
    row.effective_assignment_count = u64::from(header.assignment_count);
    row.maintenance_action = "none";
    row.maintenance_reason = "not_evaluated";
    row.leaf_object_bytes = u64::from(placement.object_bytes);
}

fn placement_state_name(state: meta::SpirePlacementState) -> &'static str {
    match state {
        meta::SpirePlacementState::Available => "available",
        meta::SpirePlacementState::Stale => "stale",
        meta::SpirePlacementState::Unavailable => "unavailable",
        meta::SpirePlacementState::Skipped => "skipped",
    }
}

fn partition_object_kind_name(kind: storage::SpirePartitionObjectKind) -> &'static str {
    match kind {
        storage::SpirePartitionObjectKind::Root => "root",
        storage::SpirePartitionObjectKind::Internal => "internal",
        storage::SpirePartitionObjectKind::Leaf => "leaf",
        storage::SpirePartitionObjectKind::Delta => "delta",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireHierarchyObjectSummary {
    pid: u64,
    kind: storage::SpirePartitionObjectKind,
    level: u16,
    parent_pid: u64,
    child_pids: Vec<u64>,
}

fn hierarchy_object_summary(
    header: &storage::SpirePartitionObjectHeader,
    child_pids: Vec<u64>,
) -> SpireHierarchyObjectSummary {
    SpireHierarchyObjectSummary {
        pid: header.pid,
        kind: header.kind,
        level: header.level,
        parent_pid: header.parent_pid,
        child_pids,
    }
}

fn validate_recursive_hierarchy_shape(
    objects: &[SpireHierarchyObjectSummary],
) -> Result<bool, String> {
    if objects.is_empty() {
        return Ok(false);
    }

    let mut by_pid = HashMap::with_capacity(objects.len());
    for object in objects {
        if object.pid == 0 {
            return Err("ec_spire hierarchy object pid 0 is invalid".to_owned());
        }
        if by_pid.insert(object.pid, object).is_some() {
            return Err(format!(
                "ec_spire hierarchy contains duplicate active pid {}",
                object.pid
            ));
        }
    }

    let roots = objects
        .iter()
        .filter(|object| object.kind == storage::SpirePartitionObjectKind::Root)
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(format!(
            "ec_spire hierarchy needs exactly one root object, found {}",
            roots.len()
        ));
    }
    let root = roots[0];
    if root.parent_pid != 0 {
        return Err(format!(
            "ec_spire root pid {} must use parent_pid 0, got {}",
            root.pid, root.parent_pid
        ));
    }
    if root.level == 0 {
        return Err(format!("ec_spire root pid {} must use level > 0", root.pid));
    }

    let has_internal = objects
        .iter()
        .any(|object| object.kind == storage::SpirePartitionObjectKind::Internal);
    for object in objects {
        match object.kind {
            storage::SpirePartitionObjectKind::Root
            | storage::SpirePartitionObjectKind::Internal => {
                if object.kind == storage::SpirePartitionObjectKind::Internal
                    && object.parent_pid == 0
                {
                    return Err(format!(
                        "ec_spire internal routing pid {} must have nonzero parent_pid",
                        object.pid
                    ));
                }
                if object.level == 0 {
                    return Err(format!(
                        "ec_spire routing pid {} must use level > 0",
                        object.pid
                    ));
                }
                let mut seen_children = HashSet::with_capacity(object.child_pids.len());
                for child_pid in &object.child_pids {
                    if !seen_children.insert(*child_pid) {
                        return Err(format!(
                            "ec_spire routing pid {} references duplicate child pid {}",
                            object.pid, child_pid
                        ));
                    }
                    let child = by_pid.get(child_pid).ok_or_else(|| {
                        format!(
                            "ec_spire routing pid {} references missing child pid {}",
                            object.pid, child_pid
                        )
                    })?;
                    if child.parent_pid != object.pid {
                        return Err(format!(
                            "ec_spire child pid {} parent_pid {} does not match routing pid {}",
                            child.pid, child.parent_pid, object.pid
                        ));
                    }
                    if object.level == 1 {
                        if child.kind != storage::SpirePartitionObjectKind::Leaf || child.level != 0
                        {
                            return Err(format!(
                                "ec_spire level-1 routing pid {} child pid {} must be a level-0 leaf",
                                object.pid, child.pid
                            ));
                        }
                    } else if child.kind != storage::SpirePartitionObjectKind::Internal
                        || child.level.checked_add(1) != Some(object.level)
                    {
                        return Err(format!(
                            "ec_spire routing pid {} level {} child pid {} has kind {:?} level {}",
                            object.pid, object.level, child.pid, child.kind, child.level
                        ));
                    }
                }
            }
            storage::SpirePartitionObjectKind::Leaf => {
                if object.level != 0 {
                    return Err(format!(
                        "ec_spire leaf pid {} must use level 0, got {}",
                        object.pid, object.level
                    ));
                }
                let parent = by_pid.get(&object.parent_pid).ok_or_else(|| {
                    format!(
                        "ec_spire leaf pid {} references missing parent pid {}",
                        object.pid, object.parent_pid
                    )
                })?;
                if parent.kind != storage::SpirePartitionObjectKind::Root
                    && parent.kind != storage::SpirePartitionObjectKind::Internal
                {
                    return Err(format!(
                        "ec_spire leaf pid {} parent pid {} is not a routing object",
                        object.pid, object.parent_pid
                    ));
                }
                if !parent.child_pids.contains(&object.pid) {
                    return Err(format!(
                        "ec_spire leaf pid {} is not referenced by parent pid {}",
                        object.pid, object.parent_pid
                    ));
                }
            }
            storage::SpirePartitionObjectKind::Delta => {
                if object.level != 0 {
                    return Err(format!(
                        "ec_spire delta pid {} must use level 0, got {}",
                        object.pid, object.level
                    ));
                }
                let parent = by_pid.get(&object.parent_pid).ok_or_else(|| {
                    format!(
                        "ec_spire delta pid {} references missing base leaf pid {}",
                        object.pid, object.parent_pid
                    )
                })?;
                if parent.kind != storage::SpirePartitionObjectKind::Leaf {
                    return Err(format!(
                        "ec_spire delta pid {} parent pid {} is not a leaf",
                        object.pid, object.parent_pid
                    ));
                }
            }
        }
    }

    Ok(has_internal)
}

fn hierarchy_snapshot_status(
    root_routing_object_count: u64,
    internal_routing_object_count: u64,
    leaf_object_count: u64,
    hierarchy_shape_valid: bool,
) -> (&'static str, &'static str) {
    if root_routing_object_count == 0 && leaf_object_count == 0 {
        return ("empty", "none");
    }
    if root_routing_object_count == 0 {
        return (
            "no_root_object",
            "inspect active epoch metadata before enabling recursive routing",
        );
    }
    if root_routing_object_count > 1 {
        return (
            "multiple_root_objects",
            "inspect active epoch metadata before enabling recursive routing",
        );
    }
    if !hierarchy_shape_valid {
        return (
            "invalid_hierarchy_shape",
            "inspect active root/internal/leaf parent-child metadata before scanning recursively",
        );
    }
    if internal_routing_object_count == 0 {
        return (
            "single_level_foundation",
            "set recursive_fanout >= 2 during build to publish recursive routing metadata",
        );
    }
    (
        "hierarchy_metadata_present",
        "recursive routing is available; per-level nprobe metadata remains deferred",
    )
}

pub(crate) unsafe fn active_snapshot_diagnostics(
    index_relation: pg_sys::Relation,
) -> SpireActiveSnapshotDiagnostics {
    let result = (|| -> Result<SpireActiveSnapshotDiagnostics, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(SpireActiveSnapshotDiagnostics::empty(root_control));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let diagnostics = diagnostics::collect_snapshot_diagnostics(&snapshot, &object_store)?;

        Ok(SpireActiveSnapshotDiagnostics {
            active_epoch: root_control.active_epoch,
            next_pid: root_control.next_pid,
            next_local_vec_seq: root_control.next_local_vec_seq,
            consistency_mode: consistency_mode_name(diagnostics.consistency_mode),
            object_count: diagnostics.object_count as u64,
            placement_count: diagnostics.placement_count as u64,
            local_store_count: diagnostics.local_store_count as u64,
            available_placement_count: diagnostics.available_placement_count as u64,
            stale_placement_count: diagnostics.stale_placement_count as u64,
            unavailable_placement_count: diagnostics.unavailable_placement_count as u64,
            skipped_placement_count: diagnostics.skipped_placement_count as u64,
            root_object_count: diagnostics.root_object_count as u64,
            internal_object_count: diagnostics.internal_object_count as u64,
            leaf_object_count: diagnostics.leaf_object_count as u64,
            delta_object_count: diagnostics.delta_object_count as u64,
            routing_child_count: diagnostics.routing_child_count as u64,
            leaf_assignment_count: diagnostics.leaf_assignment_count as u64,
            delta_assignment_count: diagnostics.delta_assignment_count as u64,
            available_object_bytes: diagnostics.available_object_bytes,
            routing_object_bytes: diagnostics.routing_object_bytes,
            leaf_object_bytes: diagnostics.leaf_object_bytes,
            delta_object_bytes: diagnostics.delta_object_bytes,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_allocator_snapshot(
    index_relation: pg_sys::Relation,
    warn_within: u64,
) -> SpireIndexAllocatorSnapshot {
    let result = (|| -> Result<SpireIndexAllocatorSnapshot, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let diagnostics = diagnostics::collect_allocator_diagnostics(&root_control, warn_within)?;
        Ok(SpireIndexAllocatorSnapshot {
            active_epoch: root_control.active_epoch,
            warn_within,
            next_pid: diagnostics.pid.next_value,
            remaining_pid_allocations: diagnostics.pid.remaining_allocations,
            pid_near_exhaustion: diagnostics.pid.near_exhaustion,
            next_local_vec_seq: diagnostics.local_vec_id.next_value,
            remaining_local_vec_id_allocations: diagnostics.local_vec_id.remaining_allocations,
            local_vec_id_near_exhaustion: diagnostics.local_vec_id.near_exhaustion,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_options_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexOptionsSnapshot {
    let result = (|| -> Result<SpireIndexOptionsSnapshot, String> {
        let relation_options = unsafe { options::relation_options(index_relation) };
        let recursive_build_enabled = relation_options.recursive_fanout().is_some();
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let active_leaf_count = if root_control.active_epoch == 0 {
            0
        } else {
            let (epoch_manifest, object_manifest, placement_directory) =
                unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
            let snapshot = meta::SpirePublishedEpochSnapshot::new(
                &epoch_manifest,
                &object_manifest,
                &placement_directory,
            )?;
            let object_store =
                unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
            count_snapshot_options_leaf_pids(&snapshot, &object_store, recursive_build_enabled)?
        };
        let relation_nprobe = u32::try_from(relation_options.nprobe)
            .map_err(|_| "ec_spire nprobe reloption must be non-negative".to_owned())?;
        let nprobe = options::resolve_scan_nprobe(active_leaf_count, relation_nprobe);
        let rerank_width = options::resolve_scan_rerank_width(relation_options.rerank_width);
        let assignment_payload_format = relation_options.assignment_payload_format();
        let (
            assignment_payload_scannable,
            assignment_payload_status,
            assignment_payload_recommendation,
        ) = assignment_payload_scannability(assignment_payload_format);

        Ok(SpireIndexOptionsSnapshot {
            nlists: relation_options.nlists,
            recursive_fanout: relation_options.recursive_fanout,
            recursive_build_enabled,
            active_leaf_count,
            relation_nprobe: relation_options.nprobe,
            session_nprobe: nprobe
                .session_nprobe
                .map(|value| i32::try_from(value).expect("session nprobe should fit in i32")),
            effective_nprobe: nprobe.effective_nprobe,
            effective_nprobe_source: nprobe.source,
            relation_rerank_width: relation_options.rerank_width,
            session_rerank_width: rerank_width.session_rerank_width,
            effective_rerank_width: rerank_width.effective_rerank_width,
            effective_rerank_width_source: rerank_width.source,
            training_sample_rows: relation_options.training_sample_rows,
            seed: relation_options.seed,
            pq_group_size: relation_options.pq_group_size,
            storage_format: relation_options.storage_format.reloption_name(),
            assignment_payload_format: assignment_payload_format_name(assignment_payload_format),
            assignment_payload_scannable,
            assignment_payload_status,
            assignment_payload_recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_scan_sanity_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexScanSanitySnapshot {
    let result = (|| -> Result<SpireIndexScanSanitySnapshot, String> {
        let relation_options = unsafe { options::relation_options(index_relation) };
        let recursive_build_enabled = relation_options.recursive_fanout().is_some();
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let active_leaf_count = if root_control.active_epoch == 0 {
            0
        } else {
            let (epoch_manifest, object_manifest, placement_directory) =
                unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
            let snapshot = meta::SpirePublishedEpochSnapshot::new(
                &epoch_manifest,
                &object_manifest,
                &placement_directory,
            )?;
            let object_store =
                unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
            count_snapshot_options_leaf_pids(&snapshot, &object_store, recursive_build_enabled)?
        };
        let relation_nprobe = u32::try_from(relation_options.nprobe)
            .map_err(|_| "ec_spire nprobe reloption must be non-negative".to_owned())?;
        let nprobe = options::resolve_scan_nprobe(active_leaf_count, relation_nprobe);
        let rerank_width = options::resolve_scan_rerank_width(relation_options.rerank_width);
        let exact_leaf_coverage =
            active_leaf_count > 0 && nprobe.effective_nprobe == active_leaf_count;
        let full_frontier_rerank =
            active_leaf_count > 0 && rerank_width.effective_rerank_width == 0;
        let (recall_sanity_status, latency_risk_status, recommendation) = scan_sanity_status(
            root_control.active_epoch,
            exact_leaf_coverage,
            full_frontier_rerank,
        );

        Ok(SpireIndexScanSanitySnapshot {
            active_epoch: root_control.active_epoch,
            active_leaf_count,
            effective_nprobe: nprobe.effective_nprobe,
            effective_nprobe_source: nprobe.source,
            exact_leaf_coverage,
            effective_rerank_width: rerank_width.effective_rerank_width,
            effective_rerank_width_source: rerank_width.source,
            full_frontier_rerank,
            recall_sanity_status,
            latency_risk_status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_health_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexHealthSnapshot {
    let diagnostics = unsafe { active_snapshot_diagnostics(index_relation) };
    health_snapshot_from_diagnostics(&diagnostics)
}

pub(crate) unsafe fn index_relation_storage_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexRelationStorageSnapshot {
    let result = (|| -> Result<SpireIndexRelationStorageSnapshot, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let mut active_tids = HashSet::new();
        if root_control.active_epoch != 0 {
            active_tids.insert(root_control.epoch_manifest_tid);
            active_tids.insert(root_control.object_manifest_tid);
            active_tids.insert(root_control.placement_directory_tid);

            let (_epoch_manifest, object_manifest, placement_directory) =
                unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
            for entry in &object_manifest.entries {
                active_tids.insert(entry.placement_tid);
            }

            let object_store =
                unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
            for placement in &placement_directory.entries {
                for tid in unsafe { object_store.active_object_tuple_locators(placement)? } {
                    active_tids.insert(tid);
                }
            }
        }

        let relation_block_count = unsafe {
            pg_sys::RelationGetNumberOfBlocksInFork(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
            )
        };
        let mut relation_object_tuple_count = 0_u64;
        let mut relation_object_tuple_bytes = 0_u64;
        let mut active_referenced_tuple_count = 0_u64;
        let mut active_referenced_tuple_bytes = 0_u64;
        unsafe {
            page::scan_object_tuples(index_relation, |tid, tuple| {
                relation_object_tuple_count = relation_object_tuple_count
                    .checked_add(1)
                    .ok_or_else(|| "ec_spire relation object tuple count overflow".to_owned())?;
                let tuple_bytes = u64::try_from(tuple.len())
                    .map_err(|_| "ec_spire relation object tuple bytes exceed u64".to_owned())?;
                relation_object_tuple_bytes = relation_object_tuple_bytes
                    .checked_add(tuple_bytes)
                    .ok_or_else(|| "ec_spire relation object tuple bytes overflow".to_owned())?;
                if active_tids.contains(&tid) {
                    active_referenced_tuple_count = active_referenced_tuple_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            "ec_spire active referenced tuple count overflow".to_owned()
                        })?;
                    active_referenced_tuple_bytes = active_referenced_tuple_bytes
                        .checked_add(tuple_bytes)
                        .ok_or_else(|| {
                            "ec_spire active referenced tuple bytes overflow".to_owned()
                        })?;
                }
                Ok(())
            })?
        };

        let cleanup_candidate_tuple_count =
            relation_object_tuple_count.saturating_sub(active_referenced_tuple_count);
        let cleanup_candidate_tuple_bytes =
            relation_object_tuple_bytes.saturating_sub(active_referenced_tuple_bytes);
        let recommendation = if cleanup_candidate_tuple_count > 0 {
            "old relation object tuples are cleanup candidates once physical reclamation is implemented"
        } else {
            "none"
        };

        Ok(SpireIndexRelationStorageSnapshot {
            active_epoch: root_control.active_epoch,
            relation_block_count: u64::from(relation_block_count),
            relation_object_tuple_count,
            relation_object_tuple_bytes,
            active_referenced_tuple_count,
            active_referenced_tuple_bytes,
            cleanup_candidate_tuple_count,
            cleanup_candidate_tuple_bytes,
            physical_cleanup_supported: false,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_epoch_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexEpochSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexEpochSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let mut manifests = Vec::new();
        unsafe {
            page::scan_object_tuples(index_relation, |tid, tuple| {
                if tuple.len() != meta::SpireEpochManifest::encoded_len() {
                    return Ok(());
                }
                if let Ok(manifest) = meta::SpireEpochManifest::decode(tuple) {
                    manifests.push((tid, manifest));
                }
                Ok(())
            })?
        };
        let now_micros = unsafe { pg_sys::GetCurrentTimestamp() };
        epoch_snapshot_rows_from_manifests(root_control, manifests, now_micros)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_placement_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexPlacementSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexPlacementSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let rows = diagnostics::collect_store_placement_diagnostics(&snapshot, &object_store)?
            .into_iter()
            .map(|row| SpireIndexPlacementSnapshotRow {
                active_epoch: row.epoch,
                node_id: row.node_id,
                local_store_id: row.local_store_id,
                placement_count: row.placement_count as u64,
                available_placement_count: row.available_placement_count as u64,
                stale_placement_count: row.stale_placement_count as u64,
                unavailable_placement_count: row.unavailable_placement_count as u64,
                skipped_placement_count: row.skipped_placement_count as u64,
                object_count: row.object_count as u64,
                root_object_count: row.root_object_count as u64,
                internal_object_count: row.internal_object_count as u64,
                leaf_object_count: row.leaf_object_count as u64,
                delta_object_count: row.delta_object_count as u64,
                routing_child_count: row.routing_child_count as u64,
                assignment_count: row.assignment_count as u64,
                placement_object_bytes: row.placement_object_bytes,
                available_object_bytes: row.available_object_bytes,
                routing_object_bytes: row.routing_object_bytes,
                leaf_object_bytes: row.leaf_object_bytes,
                delta_object_bytes: row.delta_object_bytes,
            })
            .collect();
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_leaf_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexLeafSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexLeafSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        collect_leaf_snapshot_rows(root_control, &snapshot, &object_store)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn collect_leaf_snapshot_rows(
    root_control: meta::SpireRootControlState,
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
) -> Result<Vec<SpireIndexLeafSnapshotRow>, String> {
    let mut rows_by_leaf_pid: HashMap<u64, SpireIndexLeafSnapshotRow> = HashMap::new();

    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "leaf snapshot")?;
        let placement = lookup.placement;
        if placement.state != meta::SpirePlacementState::Available {
            continue;
        }
        let header = object_store.read_object_header(placement)?;
        match header.kind {
            storage::SpirePartitionObjectKind::Leaf => {
                apply_leaf_snapshot_base_row(
                    &mut rows_by_leaf_pid,
                    root_control.active_epoch,
                    &header,
                    placement,
                );
            }
            storage::SpirePartitionObjectKind::Delta => {
                let delta_object = object_store.read_delta_object(placement)?;
                let row = rows_by_leaf_pid
                    .entry(header.parent_pid)
                    .or_insert_with(|| SpireIndexLeafSnapshotRow {
                        active_epoch: root_control.active_epoch,
                        leaf_pid: header.parent_pid,
                        parent_pid: 0,
                        object_version: 0,
                        node_id: placement.node_id,
                        local_store_id: placement.local_store_id,
                        placement_state: "missing_base_leaf",
                        base_assignment_count: 0,
                        delta_object_count: 0,
                        delta_insert_assignment_count: 0,
                        delta_delete_assignment_count: 0,
                        effective_assignment_count: 0,
                        split_assignment_threshold: 0,
                        merge_assignment_threshold: 0,
                        split_recommended: false,
                        merge_recommended: false,
                        maintenance_action: "none",
                        maintenance_reason: "missing_base_leaf",
                        leaf_object_bytes: 0,
                        delta_object_bytes: 0,
                    });
                row.delta_object_count =
                    row.delta_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire leaf snapshot delta object count overflow".to_owned()
                    })?;
                row.delta_object_bytes = row
                    .delta_object_bytes
                    .checked_add(u64::from(placement.object_bytes))
                    .ok_or_else(|| {
                        "ec_spire leaf snapshot delta object bytes overflow".to_owned()
                    })?;
                for assignment in &delta_object.assignments {
                    if storage::is_delete_delta_assignment(assignment) {
                        row.delta_delete_assignment_count = row
                            .delta_delete_assignment_count
                            .checked_add(1)
                            .ok_or_else(|| {
                                "ec_spire leaf snapshot delta delete count overflow".to_owned()
                            })?;
                    } else if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0 {
                        row.delta_insert_assignment_count = row
                            .delta_insert_assignment_count
                            .checked_add(1)
                            .ok_or_else(|| {
                                "ec_spire leaf snapshot delta insert count overflow".to_owned()
                            })?;
                    }
                }
            }
            storage::SpirePartitionObjectKind::Root
            | storage::SpirePartitionObjectKind::Internal => {}
        }
    }

    let mut rows = rows_by_leaf_pid.into_values().collect::<Vec<_>>();
    for row in &mut rows {
        row.effective_assignment_count = row
            .base_assignment_count
            .saturating_add(row.delta_insert_assignment_count)
            .saturating_sub(row.delta_delete_assignment_count);
    }
    let effective_total = rows
        .iter()
        .map(|row| row.effective_assignment_count)
        .try_fold(0_u64, |acc, count| {
            acc.checked_add(count).ok_or_else(|| {
                "ec_spire leaf snapshot effective assignment total overflow".to_owned()
            })
        })?;
    let leaf_count = u64::try_from(rows.len())
        .map_err(|_| "ec_spire leaf snapshot row count exceeds u64".to_owned())?;
    let (split_threshold, merge_threshold) =
        leaf_maintenance_thresholds(effective_total, leaf_count);
    for row in &mut rows {
        row.split_assignment_threshold = split_threshold;
        row.merge_assignment_threshold = merge_threshold;
        let (split, merge, action, reason) = leaf_maintenance_labels(
            row.effective_assignment_count,
            split_threshold,
            merge_threshold,
        );
        row.split_recommended = split;
        row.merge_recommended = merge;
        row.maintenance_action = action;
        row.maintenance_reason = reason;
    }
    rows.sort_by_key(|row| row.leaf_pid);
    Ok(rows)
}

fn no_maintenance_plan_snapshot(
    root_control: meta::SpireRootControlState,
    active_epoch: u64,
    planned_reason: &'static str,
    planner_message: &'static str,
) -> SpireIndexMaintenancePlanSnapshot {
    SpireIndexMaintenancePlanSnapshot {
        active_epoch,
        planner_status: "no_action",
        planned_action: "none",
        planned_reason,
        replaced_parent_pid: 0,
        affected_leaf_pids: Vec::new(),
        replacement_leaf_count: 0,
        replacement_leaf_pids: Vec::new(),
        publish_epoch: 0,
        next_pid: root_control.next_pid,
        next_local_vec_seq: root_control.next_local_vec_seq,
        planner_message,
    }
}

fn no_maintenance_run_result(
    root_control: meta::SpireRootControlState,
    active_epoch: u64,
    planned_reason: &'static str,
    maintenance_message: &'static str,
) -> SpireIndexMaintenanceRunResult {
    SpireIndexMaintenanceRunResult {
        active_epoch_before: active_epoch,
        active_epoch_after: active_epoch,
        maintenance_status: "no_action",
        planned_action: "none",
        planned_reason,
        replaced_parent_pid: 0,
        affected_leaf_pids: Vec::new(),
        replacement_leaf_count: 0,
        replacement_leaf_pids: Vec::new(),
        publish_epoch: 0,
        next_pid: root_control.next_pid,
        next_local_vec_seq: root_control.next_local_vec_seq,
        published: false,
        maintenance_message,
    }
}

fn selected_maintenance_run_result(
    selected: update::SpireSelectedScheduledReplacementPublishLockPlan,
    maintenance_status: &'static str,
    published: bool,
    maintenance_message: &'static str,
) -> Result<SpireIndexMaintenanceRunResult, String> {
    let planned_action = match selected.decision.mode {
        update::SpireLeafReplacementScheduleMode::Split => "split",
        update::SpireLeafReplacementScheduleMode::Merge => "merge",
    };
    let replacement_leaf_count = u64::try_from(selected.decision.replacement_leaf_count)
        .map_err(|_| "ec_spire maintenance run replacement leaf count exceeds u64".to_owned())?;

    Ok(SpireIndexMaintenanceRunResult {
        active_epoch_before: selected.decision.active_epoch,
        active_epoch_after: if published {
            selected.lock_plan.publish_plan.epoch
        } else {
            selected.decision.active_epoch
        },
        maintenance_status,
        planned_action,
        planned_reason: selected.decision.reason,
        replaced_parent_pid: selected.decision.replaced_parent_pid,
        affected_leaf_pids: selected.decision.affected_leaf_pids,
        replacement_leaf_count,
        replacement_leaf_pids: selected.lock_plan.pid_plan.replacement_pids,
        publish_epoch: selected.lock_plan.publish_plan.epoch,
        next_pid: selected.lock_plan.publish_plan.next_pid,
        next_local_vec_seq: selected.lock_plan.publish_plan.next_local_vec_seq,
        published,
        maintenance_message,
    })
}

fn next_spire_object_version(current: u64, label: &str, pid: u64) -> Result<u64, String> {
    if current == 0 {
        return Err(format!(
            "ec_spire {label} object_version 0 is invalid for pid {pid}"
        ));
    }
    current.checked_add(1).ok_or_else(|| {
        format!("ec_spire {label} object_version overflow for pid {pid}: current {current}")
    })
}

fn scheduled_replacement_object_version_plan(
    selected: &update::SpireSelectedScheduledReplacementPublishLockPlan,
    parent_object_version: u64,
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<SpireScheduledReplacementObjectVersionPlan, String> {
    let replacement_parent_object_version = next_spire_object_version(
        parent_object_version,
        "scheduled replacement parent",
        selected.decision.replaced_parent_pid,
    )?;
    let affected_leaf_pids = selected
        .decision
        .affected_leaf_pids
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    let mut seen_leaf_pids = HashSet::with_capacity(affected_leaf_pids.len());
    let mut max_leaf_object_version = None;
    for row in rows {
        if !affected_leaf_pids.contains(&row.leaf_pid) {
            continue;
        }
        if !seen_leaf_pids.insert(row.leaf_pid) {
            return Err(format!(
                "ec_spire scheduled replacement saw duplicate affected leaf pid {}",
                row.leaf_pid
            ));
        }
        let leaf_object_version = next_spire_object_version(
            row.object_version,
            "scheduled replacement leaf",
            row.leaf_pid,
        )?;
        max_leaf_object_version = Some(
            max_leaf_object_version
                .unwrap_or(leaf_object_version)
                .max(leaf_object_version),
        );
    }
    if seen_leaf_pids.len() != affected_leaf_pids.len() {
        let missing = affected_leaf_pids
            .difference(&seen_leaf_pids)
            .copied()
            .collect::<Vec<_>>();
        return Err(format!(
            "ec_spire scheduled replacement object version plan missing affected leaf rows: {missing:?}"
        ));
    }
    let leaf_object_version = max_leaf_object_version.ok_or_else(|| {
        "ec_spire scheduled replacement object version plan requires affected leaf rows".to_owned()
    })?;

    Ok(SpireScheduledReplacementObjectVersionPlan {
        parent_object_version: replacement_parent_object_version,
        leaf_object_version,
    })
}

fn maintenance_run_result_from_rows(
    root_control: meta::SpireRootControlState,
    active_epoch_manifest: &meta::SpireEpochManifest,
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<SpireIndexMaintenanceRunResult, String> {
    if root_control.active_epoch == 0 {
        return Ok(no_maintenance_run_result(
            root_control,
            0,
            "empty_index",
            "build or insert rows to publish the first SPIRE epoch",
        ));
    }

    let mut pid_allocator = assign::SpirePidAllocator::new(root_control.next_pid)?;
    let Some(selected) = update::choose_scheduled_replacement_publish_lock_plan(
        rows,
        &root_control,
        active_epoch_manifest,
        &mut pid_allocator,
    )?
    else {
        return Ok(no_maintenance_run_result(
            root_control,
            active_epoch_manifest.epoch,
            "no_candidate",
            "active leaves are within split/merge thresholds",
        ));
    };

    selected_maintenance_run_result(
        selected,
        "planned",
        false,
        "scheduled replacement candidate selected under publish lock; no epoch was published",
    )
}

unsafe fn build_relation_selected_scheduled_maintenance_input(
    index_relation: pg_sys::Relation,
    snapshot: &meta::SpirePublishedEpochSnapshot<'_>,
    object_store: &storage::SpireRelationObjectStore,
    selected: &update::SpireSelectedScheduledReplacementPublishLockPlan,
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<update::SpireRelationScheduledReplacementExecutionInput, String> {
    let parent = update::load_selected_scheduled_replacement_parent_routing(
        snapshot,
        object_store,
        selected,
    )?;
    let object_versions =
        scheduled_replacement_object_version_plan(selected, parent.header.object_version, rows)?;
    let (published_at_micros, retain_until_micros) =
        unsafe { build::current_epoch_publish_times()? };

    match selected.decision.mode {
        update::SpireLeafReplacementScheduleMode::Split => {
            let heap_relation = unsafe { SpireHeapRelationGuard::open_for_index(index_relation)? };
            let heap_snapshot = unsafe { active_spire_maintenance_snapshot()? };
            let indexed_attribute = unsafe {
                crate::am::ec_hnsw::source::resolve_indexed_vector_attribute(
                    heap_relation.relation(),
                    index_relation,
                    "ec_spire maintenance split replacement source vector",
                )
            };
            let slot = unsafe { SpireHeapSlotGuard::new(heap_relation.relation())? };
            let relation_options = options::relation_options(index_relation);
            unsafe {
                update::build_relation_selected_scheduled_split_replacement_execution_input_from_heap_sources(
                    heap_relation.relation(),
                    heap_snapshot,
                    slot.as_ptr(),
                    indexed_attribute,
                    snapshot,
                    object_store,
                    selected,
                    usize::from(parent.dimensions),
                    relation_options.seed as u64,
                    build::SPIRE_DEFAULT_KMEANS_ITERATIONS,
                    object_versions.parent_object_version,
                    object_versions.leaf_object_version,
                    published_at_micros,
                    retain_until_micros,
                )
            }
        }
        update::SpireLeafReplacementScheduleMode::Merge => {
            update::build_relation_selected_scheduled_merge_replacement_execution_input_from_snapshot(
                snapshot,
                object_store,
                selected,
                rows,
                object_versions.parent_object_version,
                object_versions.leaf_object_version,
                published_at_micros,
                retain_until_micros,
            )
        }
    }
}

fn maintenance_plan_snapshot_from_rows(
    root_control: meta::SpireRootControlState,
    active_epoch_manifest: &meta::SpireEpochManifest,
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<SpireIndexMaintenancePlanSnapshot, String> {
    if root_control.active_epoch == 0 {
        return Ok(no_maintenance_plan_snapshot(
            root_control,
            0,
            "empty_index",
            "build or insert rows to publish the first SPIRE epoch",
        ));
    }

    let mut pid_allocator = assign::SpirePidAllocator::new(root_control.next_pid)?;
    let Some(selected) = update::choose_scheduled_replacement_publish_lock_plan(
        rows,
        &root_control,
        active_epoch_manifest,
        &mut pid_allocator,
    )?
    else {
        return Ok(no_maintenance_plan_snapshot(
            root_control,
            active_epoch_manifest.epoch,
            "no_candidate",
            "active leaves are within split/merge thresholds",
        ));
    };

    let planned_action = match selected.decision.mode {
        update::SpireLeafReplacementScheduleMode::Split => "split",
        update::SpireLeafReplacementScheduleMode::Merge => "merge",
    };
    let replacement_leaf_count = u64::try_from(selected.decision.replacement_leaf_count)
        .map_err(|_| "ec_spire maintenance plan replacement leaf count exceeds u64".to_owned())?;

    Ok(SpireIndexMaintenancePlanSnapshot {
        active_epoch: selected.decision.active_epoch,
        planner_status: "planned",
        planned_action,
        planned_reason: selected.decision.reason,
        replaced_parent_pid: selected.decision.replaced_parent_pid,
        affected_leaf_pids: selected.decision.affected_leaf_pids,
        replacement_leaf_count,
        replacement_leaf_pids: selected.lock_plan.pid_plan.replacement_pids,
        publish_epoch: selected.lock_plan.publish_plan.epoch,
        next_pid: selected.lock_plan.publish_plan.next_pid,
        next_local_vec_seq: selected.lock_plan.publish_plan.next_local_vec_seq,
        planner_message: "scheduled replacement candidate selected; publish_epoch, next_pid, and next_local_vec_seq are projected and not advanced",
    })
}

pub(crate) unsafe fn index_maintenance_plan_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexMaintenancePlanSnapshot {
    let result = (|| -> Result<SpireIndexMaintenancePlanSnapshot, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(no_maintenance_plan_snapshot(
                root_control,
                0,
                "empty_index",
                "build or insert rows to publish the first SPIRE epoch",
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let rows = collect_leaf_snapshot_rows(root_control, &snapshot, &object_store)?;
        maintenance_plan_snapshot_from_rows(root_control, &epoch_manifest, &rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_locked_maintenance_plan_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexMaintenancePlanSnapshot {
    let _guard = unsafe { lock_publish_relation(index_relation) };
    unsafe { index_maintenance_plan_snapshot(index_relation) }
}

pub(crate) unsafe fn index_locked_maintenance_run_plan(
    index_relation: pg_sys::Relation,
) -> SpireIndexMaintenanceRunResult {
    let _guard = unsafe { lock_publish_relation(index_relation) };
    let result = (|| -> Result<SpireIndexMaintenanceRunResult, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(no_maintenance_run_result(
                root_control,
                0,
                "empty_index",
                "build or insert rows to publish the first SPIRE epoch",
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let rows = collect_leaf_snapshot_rows(root_control, &snapshot, &object_store)?;
        maintenance_run_result_from_rows(root_control, &epoch_manifest, &rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_maintenance_run(
    index_relation: pg_sys::Relation,
) -> SpireIndexMaintenanceRunResult {
    let _guard = unsafe { lock_publish_relation(index_relation) };
    let result = (|| -> Result<SpireIndexMaintenanceRunResult, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(no_maintenance_run_result(
                root_control,
                0,
                "empty_index",
                "build or insert rows to publish the first SPIRE epoch",
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let published_snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let validated_snapshot =
            meta::SpireValidatedEpochSnapshot::from_snapshot(published_snapshot)?;
        let mut object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let rows = collect_leaf_snapshot_rows(root_control, &validated_snapshot, &object_store)?;
        let mut pid_allocator = assign::SpirePidAllocator::new(root_control.next_pid)?;
        let Some(selected) = update::choose_scheduled_replacement_publish_lock_plan(
            &rows,
            &root_control,
            &epoch_manifest,
            &mut pid_allocator,
        )?
        else {
            return Ok(no_maintenance_run_result(
                root_control,
                epoch_manifest.epoch,
                "no_candidate",
                "active leaves are within split/merge thresholds",
            ));
        };
        let input = unsafe {
            build_relation_selected_scheduled_maintenance_input(
                index_relation,
                &published_snapshot,
                &object_store,
                &selected,
                &rows,
            )?
        };
        unsafe {
            update::publish_relation_selected_scheduled_replacement_epoch(
                index_relation,
                epoch_manifest,
                &published_snapshot,
                &selected,
                input,
                &mut object_store,
            )?;
        }

        selected_maintenance_run_result(
            selected,
            "published",
            true,
            "scheduled replacement epoch was published",
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_insert_debt_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexInsertDebtSnapshot {
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    let leaf_rows = unsafe { index_leaf_snapshot(index_relation) };
    let active_leaf_count = u64::try_from(leaf_rows.len())
        .unwrap_or_else(|_| pgrx::error!("ec_spire leaf row count exceeds u64"));
    let leaf_count_with_deltas = leaf_rows
        .iter()
        .filter(|row| row.delta_object_count > 0)
        .count()
        .try_into()
        .unwrap_or_else(|_| pgrx::error!("ec_spire leaf delta row count exceeds u64"));
    let delta_object_count = leaf_rows
        .iter()
        .map(|row| row.delta_object_count)
        .sum::<u64>();
    let delta_insert_assignment_count = leaf_rows
        .iter()
        .map(|row| row.delta_insert_assignment_count)
        .sum::<u64>();
    let max_delta_objects_per_leaf = leaf_rows
        .iter()
        .map(|row| row.delta_object_count)
        .max()
        .unwrap_or(0);
    let batching_recommended =
        max_delta_objects_per_leaf > 1 || delta_object_count > active_leaf_count;
    let recommendation = if batching_recommended {
        "batch post-build inserts by routed base leaf before publishing replacement epochs"
    } else {
        "none"
    };

    SpireIndexInsertDebtSnapshot {
        active_epoch: root_control.active_epoch,
        active_leaf_count,
        leaf_count_with_deltas,
        delta_object_count,
        delta_insert_assignment_count,
        max_delta_objects_per_leaf,
        insert_batching_supported: false,
        batching_recommended,
        recommendation,
    }
}

pub(crate) unsafe fn index_hierarchy_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexHierarchySnapshot {
    let result = (|| -> Result<SpireIndexHierarchySnapshot, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            let (status, recommendation) = hierarchy_snapshot_status(0, 0, 0, true);
            return Ok(SpireIndexHierarchySnapshot {
                active_epoch: 0,
                root_pid: 0,
                root_level: 0,
                max_observed_level: 0,
                hierarchy_depth: 0,
                routing_object_count: 0,
                root_routing_object_count: 0,
                internal_routing_object_count: 0,
                leaf_object_count: 0,
                delta_object_count: 0,
                centroid_dimensions: 0,
                root_child_count: 0,
                distinct_leaf_parent_count: 0,
                recursive_routing_supported: false,
                per_level_nprobe_supported: false,
                status,
                recommendation,
            });
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };

        let mut root_pid = 0_u64;
        let mut root_level = 0_u16;
        let mut max_observed_level = 0_u16;
        let mut routing_object_count = 0_u64;
        let mut root_routing_object_count = 0_u64;
        let mut internal_routing_object_count = 0_u64;
        let mut leaf_object_count = 0_u64;
        let mut delta_object_count = 0_u64;
        let mut centroid_dimensions = 0_u16;
        let mut root_child_count = 0_u64;
        let mut leaf_parent_pids = HashSet::new();
        let mut hierarchy_objects = Vec::new();

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "hierarchy snapshot")?;
            let placement = lookup.placement;
            if placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let header = unsafe { object_store.read_object_header(placement)? };
            max_observed_level = max_observed_level.max(header.level);
            match header.kind {
                storage::SpirePartitionObjectKind::Root => {
                    let routing_object = unsafe { object_store.read_routing_object(placement)? };
                    routing_object_count =
                        routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot routing object count overflow".to_owned()
                        })?;
                    root_routing_object_count =
                        root_routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot root object count overflow".to_owned()
                        })?;
                    root_pid = header.pid;
                    root_level = header.level;
                    centroid_dimensions = routing_object.dimensions;
                    hierarchy_objects.push(hierarchy_object_summary(
                        &routing_object.header,
                        routing_object.child_pids.clone(),
                    ));
                    root_child_count =
                        u64::try_from(routing_object.child_count()).map_err(|_| {
                            "ec_spire hierarchy snapshot root child count exceeds u64".to_owned()
                        })?;
                }
                storage::SpirePartitionObjectKind::Internal => {
                    routing_object_count =
                        routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot routing object count overflow".to_owned()
                        })?;
                    internal_routing_object_count = internal_routing_object_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            "ec_spire hierarchy snapshot internal object count overflow".to_owned()
                        })?;
                    let routing_object = unsafe { object_store.read_routing_object(placement)? };
                    hierarchy_objects.push(hierarchy_object_summary(
                        &routing_object.header,
                        routing_object.child_pids.clone(),
                    ));
                }
                storage::SpirePartitionObjectKind::Leaf => {
                    leaf_object_count = leaf_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire hierarchy snapshot leaf object count overflow".to_owned()
                    })?;
                    leaf_parent_pids.insert(header.parent_pid);
                    hierarchy_objects.push(hierarchy_object_summary(&header, Vec::new()));
                }
                storage::SpirePartitionObjectKind::Delta => {
                    delta_object_count = delta_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire hierarchy snapshot delta object count overflow".to_owned()
                    })?;
                    hierarchy_objects.push(hierarchy_object_summary(&header, Vec::new()));
                }
            }
        }

        let hierarchy_depth = if root_routing_object_count == 0 {
            0
        } else {
            max_observed_level.max(root_level)
        };
        let hierarchy_shape_valid = validate_recursive_hierarchy_shape(&hierarchy_objects).is_ok();
        let (status, recommendation) = hierarchy_snapshot_status(
            root_routing_object_count,
            internal_routing_object_count,
            leaf_object_count,
            hierarchy_shape_valid,
        );

        Ok(SpireIndexHierarchySnapshot {
            active_epoch: root_control.active_epoch,
            root_pid,
            root_level,
            max_observed_level,
            hierarchy_depth,
            routing_object_count,
            root_routing_object_count,
            internal_routing_object_count,
            leaf_object_count,
            delta_object_count,
            centroid_dimensions,
            root_child_count,
            distinct_leaf_parent_count: u64::try_from(leaf_parent_pids.len()).map_err(|_| {
                "ec_spire hierarchy snapshot leaf parent count exceeds u64".to_owned()
            })?,
            recursive_routing_supported: hierarchy_shape_valid && internal_routing_object_count > 0,
            per_level_nprobe_supported: false,
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_object_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexObjectSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexObjectSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let mut rows = Vec::with_capacity(snapshot.object_manifest().entries.len());

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "object snapshot")?;
            let placement = lookup.placement;
            let mut row = SpireIndexObjectSnapshotRow {
                active_epoch: root_control.active_epoch,
                pid: manifest_entry.pid,
                object_kind: "unreadable",
                object_version: manifest_entry.object_version,
                published_epoch_backref: 0,
                level: 0,
                parent_pid: 0,
                child_count: 0,
                assignment_count: 0,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                store_relid: placement.store_relid,
                placement_state: placement_state_name(placement.state),
                object_bytes: u64::from(placement.object_bytes),
                object_readable: false,
            };
            if placement.state == meta::SpirePlacementState::Available {
                let header = unsafe { object_store.read_object_header(placement)? };
                row.object_kind = partition_object_kind_name(header.kind);
                row.object_version = header.object_version;
                row.published_epoch_backref = header.published_epoch_backref;
                row.level = header.level;
                row.parent_pid = header.parent_pid;
                row.child_count = u64::from(header.child_count);
                row.assignment_count = u64::from(header.assignment_count);
                row.object_readable = true;
            }
            rows.push(row);
        }

        rows.sort_by_key(|row| row.pid);
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_delta_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexDeltaSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexDeltaSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let mut rows = Vec::new();

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "delta snapshot")?;
            let placement = lookup.placement;
            if placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let header = unsafe { object_store.read_object_header(placement)? };
            if header.kind != storage::SpirePartitionObjectKind::Delta {
                continue;
            }
            let delta_object = unsafe { object_store.read_delta_object(placement)? };
            let mut insert_assignment_count = 0_u64;
            let mut delete_assignment_count = 0_u64;
            for assignment in &delta_object.assignments {
                if storage::is_delete_delta_assignment(assignment) {
                    delete_assignment_count =
                        delete_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire delta snapshot delete assignment count overflow".to_owned()
                        })?;
                } else if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0 {
                    insert_assignment_count =
                        insert_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire delta snapshot insert assignment count overflow".to_owned()
                        })?;
                }
            }
            rows.push(SpireIndexDeltaSnapshotRow {
                active_epoch: root_control.active_epoch,
                delta_pid: header.pid,
                parent_leaf_pid: header.parent_pid,
                object_version: header.object_version,
                published_epoch_backref: header.published_epoch_backref,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                store_relid: placement.store_relid,
                placement_state: placement_state_name(placement.state),
                assignment_count: u64::from(header.assignment_count),
                insert_assignment_count,
                delete_assignment_count,
                object_bytes: u64::from(placement.object_bytes),
            });
        }

        rows.sort_by_key(|row| row.delta_pid);
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_scan_placement_snapshot(
    index_relation: pg_sys::Relation,
    query_values: Vec<f32>,
) -> Vec<SpireIndexScanPlacementSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexScanPlacementSnapshotRow>, String> {
        let query = scan::SpireScanQuery::new(query_values)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let diagnostics = scan::collect_single_level_scan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            options::relation_options(index_relation),
        )?;
        let rows = diagnostics
            .stores
            .into_iter()
            .map(|store| SpireIndexScanPlacementSnapshotRow {
                active_epoch: store.epoch,
                effective_nprobe: diagnostics.scan_plan.nprobe,
                effective_nprobe_source: diagnostics.scan_plan.nprobe_source,
                effective_rerank_width: diagnostics.scan_plan.rerank_width as u64,
                effective_rerank_width_source: diagnostics.scan_plan.rerank_width_source,
                node_id: store.node_id,
                local_store_id: store.local_store_id,
                scanned_pid_count: store.scanned_pid_count as u64,
                leaf_pid_count: store.leaf_pid_count as u64,
                delta_pid_count: store.delta_pid_count as u64,
                candidate_row_count: store.candidate_row_count as u64,
                leaf_candidate_row_count: store.leaf_candidate_row_count as u64,
                delta_candidate_row_count: store.delta_candidate_row_count as u64,
                delete_delta_row_count: store.delete_delta_row_count as u64,
            })
            .collect();
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_root_routing_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexRootRoutingSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexRootRoutingSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        collect_root_routing_snapshot_rows(&snapshot, &object_store)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_routing_centroid_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexRoutingCentroidSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexRoutingCentroidSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        collect_routing_centroid_snapshot_rows(&snapshot, &object_store)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn collect_root_routing_snapshot_rows(
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
) -> Result<Vec<SpireIndexRootRoutingSnapshotRow>, String> {
    let mut root = None;
    // Walk the full manifest so malformed epochs with multiple roots are reported.
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "root routing snapshot")?;
        let header = object_store.read_object_header(lookup.placement)?;
        if header.kind != storage::SpirePartitionObjectKind::Root {
            continue;
        }
        if root.is_some() {
            return Err("ec_spire root routing snapshot found multiple root objects".to_owned());
        }
        root = Some((
            manifest_entry.pid,
            manifest_entry.object_version,
            object_store.read_routing_object(lookup.placement)?,
        ));
    }

    let Some((root_pid, root_object_version, root_object)) = root else {
        return Err("ec_spire root routing snapshot found no active root object".to_owned());
    };
    let root_child_count = u64::try_from(root_object.child_count())
        .map_err(|_| "ec_spire root routing child count exceeds u64".to_owned())?;
    root_object
        .children()
        .map(|child| {
            let child_lookup = snapshot.require_lookup(child.child_pid, "root routing child")?;
            let child_header = object_store.read_object_header(child_lookup.placement)?;
            Ok(SpireIndexRootRoutingSnapshotRow {
                active_epoch: snapshot.epoch_manifest().epoch,
                root_pid,
                root_object_version,
                root_level: root_object.header.level,
                root_child_count,
                centroid_dimensions: root_object.dimensions,
                centroid_index: child.centroid_index,
                child_pid: child.child_pid,
                child_kind: partition_object_kind_name(child_header.kind),
                child_object_version: child_header.object_version,
                child_level: child_header.level,
                child_parent_pid: child_header.parent_pid,
                child_assignment_count: u64::from(child_header.assignment_count),
                child_node_id: child_lookup.placement.node_id,
                child_local_store_id: child_lookup.placement.local_store_id,
                child_store_relid: child_lookup.placement.store_relid,
                child_placement_state: placement_state_name(child_lookup.placement.state),
                child_object_bytes: u64::from(child_lookup.placement.object_bytes),
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

fn collect_routing_centroid_snapshot_rows(
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
) -> Result<Vec<SpireIndexRoutingCentroidSnapshotRow>, String> {
    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup =
            snapshot.require_lookup(manifest_entry.pid, "routing centroid snapshot parent")?;
        let parent_header = object_store.read_object_header(lookup.placement)?;
        if parent_header.kind != storage::SpirePartitionObjectKind::Root
            && parent_header.kind != storage::SpirePartitionObjectKind::Internal
        {
            continue;
        }
        let parent = object_store.read_routing_object(lookup.placement)?;
        let parent_child_count = u64::try_from(parent.child_count())
            .map_err(|_| "ec_spire routing centroid child count exceeds u64".to_owned())?;
        for child in parent.children() {
            let child_lookup =
                snapshot.require_lookup(child.child_pid, "routing centroid snapshot child")?;
            let child_header = object_store.read_object_header(child_lookup.placement)?;
            rows.push(SpireIndexRoutingCentroidSnapshotRow {
                active_epoch: snapshot.epoch_manifest().epoch,
                parent_pid: parent.header.pid,
                parent_kind: partition_object_kind_name(parent.header.kind),
                parent_object_version: parent.header.object_version,
                parent_level: parent.header.level,
                parent_child_count,
                centroid_dimensions: parent.dimensions,
                centroid_index: child.centroid_index,
                child_pid: child.child_pid,
                child_kind: partition_object_kind_name(child_header.kind),
                child_object_version: child_header.object_version,
                child_level: child_header.level,
                child_parent_pid: child_header.parent_pid,
                child_assignment_count: u64::from(child_header.assignment_count),
                child_node_id: child_lookup.placement.node_id,
                child_local_store_id: child_lookup.placement.local_store_id,
                child_store_relid: child_lookup.placement.store_relid,
                child_placement_state: placement_state_name(child_lookup.placement.state),
                child_object_bytes: u64::from(child_lookup.placement.object_bytes),
                centroid: child.centroid.to_vec(),
            });
        }
    }
    Ok(rows)
}

fn not_implemented(callback: &str) -> ! {
    pgrx::error!("ec_spire {callback} is not implemented yet")
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_relation_object_tuple_roundtrip(
    index_oid: pg_sys::Oid,
) -> (u32, u16, u64, u32, u64, u64, u32, u64) {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = unsafe { pg_sys::index_open(index_oid, lockmode) };
    let result = (|| -> Result<(u32, u16, u64, u32, u64, u64, u32, u64), String> {
        let store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let object = storage::SpireRoutingPartitionObject::root(
            10,
            1,
            2,
            vec![storage::SpireRoutingChildEntry {
                centroid_index: 0,
                child_pid: 11,
                centroid: vec![1.0, 0.0],
            }],
        )?;

        let placement = unsafe { store.insert_routing_object(1, &object)? };
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let decoded = unsafe { store.read_routing_object(&placement)? };
        let child = decoded
            .children()
            .next()
            .ok_or_else(|| "ec_spire debug routing object lost its child".to_owned())?;

        Ok((
            placement.object_tid.block_number,
            placement.object_tid.offset_number,
            root_control.active_epoch,
            placement.store_relid,
            decoded.header.pid,
            decoded.header.object_version,
            decoded.header.child_count,
            child.child_pid,
        ))
    })();
    unsafe { pg_sys::index_close(index_relation, lockmode) };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_relation_leaf_v2_roundtrip(
    index_oid: pg_sys::Oid,
) -> (u32, u16, u32, u32, u64, u32) {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = unsafe { pg_sys::index_open(index_oid, lockmode) };
    let result = (|| -> Result<(u32, u16, u32, u32, u64, u32), String> {
        let store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let assignments = vec![
            storage::SpireLeafAssignmentRow {
                flags: storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: storage::SpireVecId::local(1),
                heap_tid: crate::storage::page::ItemPointer {
                    block_number: 42,
                    offset_number: 1,
                },
                payload_format: storage::SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: 0.5,
                encoded_payload: vec![1, 2, 3, 4],
            },
            storage::SpireLeafAssignmentRow {
                flags: storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: storage::SpireVecId::local(2),
                heap_tid: crate::storage::page::ItemPointer {
                    block_number: 43,
                    offset_number: 2,
                },
                payload_format: storage::SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: 0.75,
                encoded_payload: vec![5, 6, 7, 8],
            },
        ];

        let placement =
            unsafe { store.insert_leaf_object_v2_from_rows(1, 20, 1, 10, &assignments)? };
        let leaf = unsafe { store.read_leaf_object_v2(&placement)? };
        let rows = leaf.assignment_rows()?;
        let first_row = rows
            .first()
            .ok_or_else(|| "ec_spire debug leaf V2 lost its first row".to_owned())?;

        Ok((
            placement.object_tid.block_number,
            placement.object_tid.offset_number,
            leaf.meta.header.assignment_count,
            leaf.meta.segment_count,
            first_row
                .vec_id
                .local_sequence()
                .ok_or_else(|| "ec_spire debug leaf V2 first row lost local vec_id".to_owned())?,
            first_row.heap_tid.block_number,
        ))
    })();
    unsafe { pg_sys::index_close(index_relation, lockmode) };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_empty_manifest_publish_roundtrip(
    index_oid: pg_sys::Oid,
) -> (u64, u64, u64, u32, u16, u32, u16, u32, u16) {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = unsafe { pg_sys::index_open(index_oid, lockmode) };
    let result = (|| -> Result<(u64, u64, u64, u32, u16, u32, u16, u32, u16), String> {
        let epoch_manifest = meta::SpireEpochManifest {
            epoch: 1,
            state: meta::SpireEpochState::Published,
            consistency_mode: meta::SpireConsistencyMode::Strict,
            published_at_micros: 1,
            retain_until_micros: 600_000_001,
            active_query_count: 0,
        };
        let object_manifest = meta::SpireObjectManifest::from_entries(1, Vec::new())?;
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, Vec::new())?;
        let input = build::SpirePublishCoordinatorInput {
            epoch_manifest: &epoch_manifest,
            object_manifest: &object_manifest,
            placement_directory: &placement_directory,
            next_pid: assign::SPIRE_FIRST_PID,
            next_local_vec_seq: assign::SPIRE_FIRST_LOCAL_VEC_SEQ,
        };
        let manifests = build::encode_manifest_bundle_for_publish(input)?;
        let locators =
            unsafe { build::write_manifest_bundle_to_relation(index_relation, &manifests)? };
        let root_control = build::root_control_state_for_publish(input, locators)?;
        unsafe { page::initialize_root_control_page(index_relation, root_control) };
        let persisted = unsafe { page::read_root_control_page(index_relation) };

        Ok((
            persisted.active_epoch,
            persisted.next_pid,
            persisted.next_local_vec_seq,
            persisted.epoch_manifest_tid.block_number,
            persisted.epoch_manifest_tid.offset_number,
            persisted.object_manifest_tid.block_number,
            persisted.object_manifest_tid.offset_number,
            persisted.placement_directory_tid.block_number,
            persisted.placement_directory_tid.offset_number,
        ))
    })();
    unsafe { pg_sys::index_close(index_relation, lockmode) };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_root_control(index_oid: pg_sys::Oid) -> (u64, u64, u64) {
    let lockmode = pg_sys::AccessShareLock as pg_sys::LOCKMODE;
    let index_relation = unsafe { pg_sys::index_open(index_oid, lockmode) };
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    unsafe { pg_sys::index_close(index_relation, lockmode) };
    (
        root_control.active_epoch,
        root_control.next_pid,
        root_control.next_local_vec_seq,
    )
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireDebugSnapshotDiagnostics {
    pub(crate) epoch: u64,
    pub(crate) object_count: u64,
    pub(crate) placement_count: u64,
    pub(crate) local_store_count: u64,
    pub(crate) available_placement_count: u64,
    pub(crate) root_object_count: u64,
    pub(crate) leaf_object_count: u64,
    pub(crate) routing_child_count: u64,
    pub(crate) leaf_assignment_count: u64,
    pub(crate) available_object_bytes: u64,
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_active_snapshot_diagnostics(
    index_oid: pg_sys::Oid,
) -> SpireDebugSnapshotDiagnostics {
    let lockmode = pg_sys::AccessShareLock as pg_sys::LOCKMODE;
    let index_relation = unsafe { pg_sys::index_open(index_oid, lockmode) };
    let result = (|| -> Result<SpireDebugSnapshotDiagnostics, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let diagnostics = diagnostics::collect_snapshot_diagnostics(&snapshot, &object_store)?;

        Ok(SpireDebugSnapshotDiagnostics {
            epoch: diagnostics.epoch,
            object_count: diagnostics.object_count as u64,
            placement_count: diagnostics.placement_count as u64,
            local_store_count: diagnostics.local_store_count as u64,
            available_placement_count: diagnostics.available_placement_count as u64,
            root_object_count: diagnostics.root_object_count as u64,
            leaf_object_count: diagnostics.leaf_object_count as u64,
            routing_child_count: diagnostics.routing_child_count as u64,
            leaf_assignment_count: diagnostics.leaf_assignment_count as u64,
            available_object_bytes: diagnostics.available_object_bytes,
        })
    })();
    unsafe { pg_sys::index_close(index_relation, lockmode) };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tid(block_number: u32, offset_number: u16) -> crate::storage::page::ItemPointer {
        crate::storage::page::ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn published_epoch_manifest(epoch: u64) -> meta::SpireEpochManifest {
        meta::SpireEpochManifest {
            epoch,
            state: meta::SpireEpochState::Published,
            consistency_mode: meta::SpireConsistencyMode::Strict,
            published_at_micros: 1,
            retain_until_micros: 1,
            active_query_count: 0,
        }
    }

    fn retired_epoch_manifest(epoch: u64) -> meta::SpireEpochManifest {
        meta::SpireEpochManifest {
            epoch,
            state: meta::SpireEpochState::Retired,
            consistency_mode: meta::SpireConsistencyMode::Strict,
            published_at_micros: 1,
            retain_until_micros: 1,
            active_query_count: 0,
        }
    }

    fn manifest_entry_for(placement: &meta::SpirePlacementEntry) -> meta::SpireManifestEntry {
        meta::SpireManifestEntry {
            epoch: placement.epoch,
            pid: placement.pid,
            object_version: placement.object_version,
            placement_tid: placement.object_tid,
        }
    }

    fn empty_leaf_row(
        store: &mut storage::SpireLocalObjectStore,
        pid: u64,
        parent_pid: u64,
    ) -> meta::SpirePlacementEntry {
        store
            .insert_leaf_object_v2_from_rows(1, pid, 1, parent_pid, &[])
            .expect("empty leaf object should store")
    }

    #[test]
    fn scan_sanity_status_reports_empty_approximate_and_full_scan() {
        assert_eq!(
            scan_sanity_status(0, false, false),
            (
                "empty",
                "none",
                "build or insert rows to publish the first SPIRE epoch"
            )
        );
        assert_eq!(
            scan_sanity_status(1, false, false),
            (
                "approximate_leaf_coverage",
                "bounded_leaf_probe",
                "increase nprobe to active_leaf_count for exact leaf coverage sanity checks"
            )
        );
        assert_eq!(
            scan_sanity_status(1, true, false),
            (
                "exact_leaf_coverage_bounded_rerank",
                "bounded_rerank",
                "set rerank_width = 0 for full-frontier exact recall sanity checks"
            )
        );
        assert_eq!(
            scan_sanity_status(1, true, true),
            (
                "exact_leaf_and_frontier_coverage",
                "full_scan",
                "use this configuration only for recall sanity checks or small indexes"
            )
        );
    }

    #[test]
    fn epoch_snapshot_partial_retired_residue_keeps_root_manifest_authoritative() {
        let active_tid = tid(10, 1);
        let retired_residue_tid = tid(10, 2);
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, active_tid, tid(10, 3), tid(10, 4))
                .expect("root/control should build");

        let rows = epoch_snapshot_rows_from_manifests(
            root_control,
            vec![
                (active_tid, published_epoch_manifest(7)),
                (retired_residue_tid, retired_epoch_manifest(7)),
            ],
            2,
        )
        .expect("epoch snapshot rows should build");

        assert_eq!(rows.len(), 2);
        let active_row = rows
            .iter()
            .find(|row| row.manifest_offset == active_tid.offset_number)
            .expect("active root row should exist");
        let retired_residue_row = rows
            .iter()
            .find(|row| row.manifest_offset == retired_residue_tid.offset_number)
            .expect("retired residue row should exist");

        assert_eq!(active_row.state, "published");
        assert!(active_row.is_active_root_manifest);
        assert!(!active_row.cleanup_eligible_now);
        assert_eq!(active_row.cleanup_blocked_reason, "active_root_manifest");
        assert_eq!(retired_residue_row.state, "retired");
        assert!(!retired_residue_row.is_active_root_manifest);
        assert!(!retired_residue_row.cleanup_eligible_now);
        assert_eq!(
            retired_residue_row.cleanup_blocked_reason,
            "retained_retired_epoch"
        );
    }

    #[test]
    fn epoch_snapshot_bundle_residue_keeps_previous_root_manifest_authoritative() {
        let active_tid = tid(10, 1);
        let retired_residue_tid = tid(10, 2);
        let bundle_residue_tid = tid(10, 3);
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, active_tid, tid(10, 4), tid(10, 5))
                .expect("root/control should build");

        let rows = epoch_snapshot_rows_from_manifests(
            root_control,
            vec![
                (active_tid, published_epoch_manifest(7)),
                (retired_residue_tid, retired_epoch_manifest(7)),
                (bundle_residue_tid, published_epoch_manifest(8)),
            ],
            2,
        )
        .expect("epoch snapshot rows should build");

        assert_eq!(rows.len(), 3);
        let active_row = rows
            .iter()
            .find(|row| row.manifest_offset == active_tid.offset_number)
            .expect("active root row should exist");
        let bundle_residue_row = rows
            .iter()
            .find(|row| row.epoch == 8)
            .expect("bundle residue row should exist");

        assert_eq!(active_row.epoch, 7);
        assert_eq!(active_row.state, "published");
        assert!(active_row.is_active_root_manifest);
        assert!(!active_row.cleanup_eligible_now);
        assert_eq!(active_row.cleanup_blocked_reason, "active_root_manifest");
        assert_eq!(bundle_residue_row.state, "published");
        assert!(!bundle_residue_row.is_active_root_manifest);
        assert!(!bundle_residue_row.cleanup_eligible_now);
        assert_eq!(
            bundle_residue_row.cleanup_blocked_reason,
            "state_not_cleanup_eligible"
        );
    }

    #[test]
    fn leaf_maintenance_thresholds_use_named_split_merge_policy() {
        assert_eq!(leaf_maintenance_thresholds(0, 0), (0, 0));
        assert_eq!(leaf_maintenance_thresholds(2, 3), (32, 0));
        assert_eq!(leaf_maintenance_thresholds(120, 3), (160, 10));
    }

    fn root_for_child(pid: u64, child_pid: u64) -> storage::SpireRoutingPartitionObject {
        storage::SpireRoutingPartitionObject::root(
            pid,
            1,
            2,
            vec![storage::SpireRoutingChildEntry {
                centroid_index: 0,
                child_pid,
                centroid: vec![1.0, 0.0],
            }],
        )
        .expect("root routing object should build")
    }

    fn hierarchy_summary(
        pid: u64,
        kind: storage::SpirePartitionObjectKind,
        level: u16,
        parent_pid: u64,
        child_pids: Vec<u64>,
    ) -> SpireHierarchyObjectSummary {
        SpireHierarchyObjectSummary {
            pid,
            kind,
            level,
            parent_pid,
            child_pids,
        }
    }

    #[test]
    fn recursive_hierarchy_shape_accepts_single_level_root_to_leaves() {
        let objects = vec![
            hierarchy_summary(
                1,
                storage::SpirePartitionObjectKind::Root,
                1,
                0,
                vec![11, 12],
            ),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                1,
                Vec::new(),
            ),
            hierarchy_summary(
                12,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                1,
                Vec::new(),
            ),
        ];

        let has_internal =
            validate_recursive_hierarchy_shape(&objects).expect("shape should validate");

        assert!(!has_internal);
    }

    #[test]
    fn recursive_hierarchy_shape_accepts_internal_level_between_root_and_leaves() {
        let objects = vec![
            hierarchy_summary(1, storage::SpirePartitionObjectKind::Root, 2, 0, vec![10]),
            hierarchy_summary(
                10,
                storage::SpirePartitionObjectKind::Internal,
                1,
                1,
                vec![11, 12],
            ),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                10,
                Vec::new(),
            ),
            hierarchy_summary(
                12,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                10,
                Vec::new(),
            ),
        ];

        let has_internal =
            validate_recursive_hierarchy_shape(&objects).expect("shape should validate");

        assert!(has_internal);
    }

    #[test]
    fn recursive_hierarchy_shape_rejects_level_skip_to_leaf() {
        let objects = vec![
            hierarchy_summary(1, storage::SpirePartitionObjectKind::Root, 2, 0, vec![11]),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                1,
                Vec::new(),
            ),
        ];

        let err = validate_recursive_hierarchy_shape(&objects).unwrap_err();

        assert!(err.contains("child pid 11 has kind Leaf level 0"));
    }

    #[test]
    fn recursive_hierarchy_shape_rejects_orphan_leaf_parent_link() {
        let objects = vec![
            hierarchy_summary(1, storage::SpirePartitionObjectKind::Root, 1, 0, vec![11]),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                99,
                Vec::new(),
            ),
        ];

        let err = validate_recursive_hierarchy_shape(&objects).unwrap_err();

        assert!(err.contains("parent_pid 99 does not match routing pid 1"));
    }

    fn maintenance_leaf_row(
        leaf_pid: u64,
        parent_pid: u64,
        effective_assignment_count: u64,
        split_recommended: bool,
        merge_recommended: bool,
    ) -> SpireIndexLeafSnapshotRow {
        SpireIndexLeafSnapshotRow {
            active_epoch: 7,
            leaf_pid,
            parent_pid,
            object_version: 1,
            node_id: meta::SPIRE_LOCAL_NODE_ID,
            local_store_id: meta::SPIRE_SINGLE_LOCAL_STORE_ID,
            placement_state: "available",
            base_assignment_count: effective_assignment_count,
            delta_object_count: 0,
            delta_insert_assignment_count: 0,
            delta_delete_assignment_count: 0,
            effective_assignment_count,
            split_assignment_threshold: 32,
            merge_assignment_threshold: 1,
            split_recommended,
            merge_recommended,
            maintenance_action: "none",
            maintenance_reason: "test",
            leaf_object_bytes: 1,
            delta_object_bytes: 0,
        }
    }

    #[test]
    fn maintenance_plan_snapshot_reports_selected_split_plan() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 100, true, false),
        ];

        let snapshot =
            maintenance_plan_snapshot_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance plan should build");

        assert_eq!(snapshot.active_epoch, 7);
        assert_eq!(snapshot.planner_status, "planned");
        assert_eq!(snapshot.planned_action, "split");
        assert_eq!(snapshot.planned_reason, "largest_split_candidate");
        assert_eq!(snapshot.replaced_parent_pid, 1);
        assert_eq!(snapshot.affected_leaf_pids, vec![12]);
        assert_eq!(snapshot.replacement_leaf_count, 2);
        assert_eq!(snapshot.replacement_leaf_pids, vec![40, 41]);
        assert_eq!(snapshot.publish_epoch, 8);
        assert_eq!(snapshot.next_pid, 42);
        assert_eq!(snapshot.next_local_vec_seq, 100);
    }

    #[test]
    fn maintenance_plan_snapshot_reports_no_action_without_candidate() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 11, false, false),
        ];

        let snapshot =
            maintenance_plan_snapshot_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance plan should build");

        assert_eq!(snapshot.active_epoch, 7);
        assert_eq!(snapshot.planner_status, "no_action");
        assert_eq!(snapshot.planned_action, "none");
        assert_eq!(snapshot.planned_reason, "no_candidate");
        assert_eq!(snapshot.replacement_leaf_pids, Vec::<u64>::new());
        assert_eq!(snapshot.next_pid, 40);
        assert_eq!(snapshot.next_local_vec_seq, 100);
    }

    #[test]
    fn maintenance_plan_snapshot_reports_selected_merge_plan() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 3, false, true),
            maintenance_leaf_row(12, 1, 1, false, true),
            maintenance_leaf_row(13, 2, 20, false, false),
        ];

        let snapshot =
            maintenance_plan_snapshot_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance plan should build");

        assert_eq!(snapshot.active_epoch, 7);
        assert_eq!(snapshot.planner_status, "planned");
        assert_eq!(snapshot.planned_action, "merge");
        assert_eq!(snapshot.planned_reason, "sparsest_same_parent_merge_pair");
        assert_eq!(snapshot.replaced_parent_pid, 1);
        assert_eq!(snapshot.affected_leaf_pids, vec![11, 12]);
        assert_eq!(snapshot.replacement_leaf_count, 1);
        assert_eq!(snapshot.replacement_leaf_pids, vec![40]);
        assert_eq!(snapshot.publish_epoch, 8);
        assert_eq!(snapshot.next_pid, 41);
        assert_eq!(snapshot.next_local_vec_seq, 100);
    }

    fn selected_split_maintenance_plan() -> update::SpireSelectedScheduledReplacementPublishLockPlan
    {
        update::SpireSelectedScheduledReplacementPublishLockPlan {
            decision: update::SpireLeafReplacementScheduleDecision {
                mode: update::SpireLeafReplacementScheduleMode::Split,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![12],
                replacement_leaf_count: 2,
                reason: "largest_split_candidate",
            },
            lock_plan: update::SpireScheduledReplacementPublishLockPlan {
                pid_plan: update::SpireLeafReplacementPidPlan {
                    replacement_pids: vec![40, 41],
                    reuses_existing_pid: false,
                    next_pid: 42,
                },
                publish_plan: update::SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: meta::SpireConsistencyMode::Strict,
                    next_pid: 42,
                    next_local_vec_seq: 100,
                },
            },
        }
    }

    #[test]
    fn maintenance_run_result_reports_no_action() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");

        let result = no_maintenance_run_result(
            root_control,
            7,
            "no_candidate",
            "active leaves are within split/merge thresholds",
        );

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "no_action");
        assert_eq!(result.planned_action, "none");
        assert_eq!(result.planned_reason, "no_candidate");
        assert_eq!(result.replacement_leaf_pids, Vec::<u64>::new());
        assert_eq!(result.publish_epoch, 0);
        assert_eq!(result.next_pid, 40);
        assert_eq!(result.next_local_vec_seq, 100);
        assert!(!result.published);
    }

    #[test]
    fn maintenance_run_result_reports_projected_selected_plan() {
        let result = selected_maintenance_run_result(
            selected_split_maintenance_plan(),
            "planned",
            false,
            "scheduled replacement selected; no epoch was published",
        )
        .expect("maintenance run result should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "planned");
        assert_eq!(result.planned_action, "split");
        assert_eq!(result.planned_reason, "largest_split_candidate");
        assert_eq!(result.replaced_parent_pid, 1);
        assert_eq!(result.affected_leaf_pids, vec![12]);
        assert_eq!(result.replacement_leaf_count, 2);
        assert_eq!(result.replacement_leaf_pids, vec![40, 41]);
        assert_eq!(result.publish_epoch, 8);
        assert_eq!(result.next_pid, 42);
        assert_eq!(result.next_local_vec_seq, 100);
        assert!(!result.published);
    }

    #[test]
    fn maintenance_run_result_reports_published_selected_plan() {
        let result = selected_maintenance_run_result(
            selected_split_maintenance_plan(),
            "published",
            true,
            "scheduled replacement epoch was published",
        )
        .expect("maintenance run result should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 8);
        assert_eq!(result.maintenance_status, "published");
        assert_eq!(result.planned_action, "split");
        assert_eq!(result.publish_epoch, 8);
        assert!(result.published);
    }

    #[test]
    fn maintenance_run_plan_from_rows_reports_selected_split_without_publishing() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 100, true, false),
        ];

        let result =
            maintenance_run_result_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance run plan should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "planned");
        assert_eq!(result.planned_action, "split");
        assert_eq!(result.planned_reason, "largest_split_candidate");
        assert_eq!(result.replacement_leaf_pids, vec![40, 41]);
        assert_eq!(result.publish_epoch, 8);
        assert_eq!(result.next_pid, 42);
        assert!(!result.published);
    }

    #[test]
    fn maintenance_run_plan_from_rows_reports_no_candidate() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 11, false, false),
        ];

        let result =
            maintenance_run_result_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance run plan should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "no_action");
        assert_eq!(result.planned_action, "none");
        assert_eq!(result.planned_reason, "no_candidate");
        assert_eq!(result.replacement_leaf_pids, Vec::<u64>::new());
        assert_eq!(result.next_pid, 40);
        assert!(!result.published);
    }

    #[test]
    fn scheduled_replacement_object_version_plan_uses_successor_versions() {
        let selected = selected_split_maintenance_plan();
        let mut unaffected = maintenance_leaf_row(11, 1, 10, false, false);
        unaffected.object_version = 9;
        let mut affected = maintenance_leaf_row(12, 1, 100, true, false);
        affected.object_version = 3;

        let plan = scheduled_replacement_object_version_plan(&selected, 4, &[unaffected, affected])
            .expect("object version plan should build");

        assert_eq!(plan.parent_object_version, 5);
        assert_eq!(plan.leaf_object_version, 4);
    }

    #[test]
    fn scheduled_replacement_object_version_plan_uses_max_affected_leaf_successor() {
        let selected = update::SpireSelectedScheduledReplacementPublishLockPlan {
            decision: update::SpireLeafReplacementScheduleDecision {
                mode: update::SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "sparsest_same_parent_merge_pair",
            },
            lock_plan: update::SpireScheduledReplacementPublishLockPlan {
                pid_plan: update::SpireLeafReplacementPidPlan {
                    replacement_pids: vec![40],
                    reuses_existing_pid: false,
                    next_pid: 41,
                },
                publish_plan: update::SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: meta::SpireConsistencyMode::Strict,
                    next_pid: 41,
                    next_local_vec_seq: 100,
                },
            },
        };
        let mut first = maintenance_leaf_row(11, 1, 3, false, true);
        first.object_version = 2;
        let mut second = maintenance_leaf_row(12, 1, 1, false, true);
        second.object_version = 5;

        let plan = scheduled_replacement_object_version_plan(&selected, 4, &[first, second])
            .expect("object version plan should build");

        assert_eq!(plan.parent_object_version, 5);
        assert_eq!(plan.leaf_object_version, 6);
    }

    #[test]
    fn scheduled_replacement_object_version_plan_rejects_missing_affected_leaf() {
        let selected = selected_split_maintenance_plan();
        let rows = vec![maintenance_leaf_row(11, 1, 10, false, false)];

        let err = scheduled_replacement_object_version_plan(&selected, 4, &rows).unwrap_err();

        assert!(err.contains("missing affected leaf rows"));
    }

    #[test]
    fn leaf_snapshot_base_row_preserves_prior_delta_counts() {
        let mut rows_by_leaf_pid = HashMap::new();
        rows_by_leaf_pid.insert(
            20,
            SpireIndexLeafSnapshotRow {
                active_epoch: 7,
                leaf_pid: 20,
                parent_pid: 0,
                object_version: 0,
                node_id: meta::SPIRE_LOCAL_NODE_ID,
                local_store_id: meta::SPIRE_SINGLE_LOCAL_STORE_ID,
                placement_state: "missing_base_leaf",
                base_assignment_count: 0,
                delta_object_count: 2,
                delta_insert_assignment_count: 3,
                delta_delete_assignment_count: 1,
                effective_assignment_count: 0,
                split_assignment_threshold: 0,
                merge_assignment_threshold: 0,
                split_recommended: false,
                merge_recommended: false,
                maintenance_action: "none",
                maintenance_reason: "missing_base_leaf",
                leaf_object_bytes: 0,
                delta_object_bytes: 44,
            },
        );
        let header = storage::SpirePartitionObjectHeader {
            kind: storage::SpirePartitionObjectKind::Leaf,
            pid: 20,
            object_version: 9,
            published_epoch_backref: 7,
            level: 1,
            parent_pid: 10,
            child_count: 0,
            assignment_count: 5,
            flags: 0,
        };
        let placement = meta::SpirePlacementEntry::local_single_store_available(
            7,
            20,
            12345,
            9,
            crate::storage::page::ItemPointer {
                block_number: 30,
                offset_number: 4,
            },
            88,
        );

        apply_leaf_snapshot_base_row(&mut rows_by_leaf_pid, 7, &header, &placement);

        let row = rows_by_leaf_pid.get(&20).expect("leaf row should exist");
        assert_eq!(row.parent_pid, 10);
        assert_eq!(row.object_version, 9);
        assert_eq!(row.base_assignment_count, 5);
        assert_eq!(row.leaf_object_bytes, 88);
        assert_eq!(row.placement_state, "available");
        assert_eq!(row.maintenance_reason, "not_evaluated");
        assert_eq!(row.delta_object_count, 2);
        assert_eq!(row.delta_insert_assignment_count, 3);
        assert_eq!(row.delta_delete_assignment_count, 1);
        assert_eq!(row.delta_object_bytes, 44);
    }

    #[test]
    fn root_routing_snapshot_rejects_active_manifest_without_root() {
        let mut store = storage::SpireLocalObjectStore::with_default_page_size(12345)
            .expect("local store should build");
        let leaf = empty_leaf_row(&mut store, 20, 10);
        let epoch_manifest = published_epoch_manifest(1);
        let object_manifest =
            meta::SpireObjectManifest::from_entries(1, vec![manifest_entry_for(&leaf)])
                .expect("object manifest should build");
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, vec![leaf])
            .expect("placement directory should build");
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .expect("snapshot should validate");

        let err = collect_root_routing_snapshot_rows(&snapshot, &store)
            .expect_err("rootless active snapshot should fail");

        assert_eq!(
            err,
            "ec_spire root routing snapshot found no active root object"
        );
    }

    #[test]
    fn root_routing_snapshot_rejects_active_manifest_with_multiple_roots() {
        let mut store = storage::SpireLocalObjectStore::with_default_page_size(12345)
            .expect("local store should build");
        let leaf = empty_leaf_row(&mut store, 20, 10);
        let first_root = store
            .insert_routing_object(1, &root_for_child(10, 20))
            .expect("first root should store");
        let second_root = store
            .insert_routing_object(1, &root_for_child(11, 20))
            .expect("second root should store");
        let epoch_manifest = published_epoch_manifest(1);
        let placements = vec![first_root, second_root, leaf];
        let object_manifest = meta::SpireObjectManifest::from_entries(
            1,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .expect("object manifest should build");
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, placements)
            .expect("placement directory should build");
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .expect("snapshot should validate");

        let err = collect_root_routing_snapshot_rows(&snapshot, &store)
            .expect_err("multi-root active snapshot should fail");

        assert_eq!(
            err,
            "ec_spire root routing snapshot found multiple root objects"
        );
    }

    #[test]
    fn root_routing_snapshot_reports_child_rows_from_local_store() {
        let mut store = storage::SpireLocalObjectStore::with_default_page_size(12345)
            .expect("local store should build");
        let leaf = empty_leaf_row(&mut store, 20, 10);
        let root = store
            .insert_routing_object(1, &root_for_child(10, 20))
            .expect("root should store");
        let epoch_manifest = published_epoch_manifest(1);
        let placements = vec![root, leaf];
        let object_manifest = meta::SpireObjectManifest::from_entries(
            1,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .expect("object manifest should build");
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, placements)
            .expect("placement directory should build");
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .expect("snapshot should validate");

        let rows = collect_root_routing_snapshot_rows(&snapshot, &store)
            .expect("root routing rows should collect");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].root_pid, 10);
        assert_eq!(rows[0].child_pid, 20);
        assert_eq!(rows[0].child_kind, "leaf");
        assert_eq!(rows[0].child_store_relid, 12345);
    }
}
