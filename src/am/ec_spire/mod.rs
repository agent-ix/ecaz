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
pub(crate) use self::vacuum::debug_spire_vacuum_remove_heap_tids;

pub(super) const EC_SPIRE_DEFAULT_NLISTS: i32 = 0;
pub(super) const EC_SPIRE_MIN_NLISTS: i32 = 0;
pub(super) const EC_SPIRE_MAX_NLISTS: i32 = 1_000_000;
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
pub(super) const EC_SPIRE_DEFAULT_PQ_GROUP_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MIN_PQ_GROUP_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MAX_PQ_GROUP_SIZE: i32 = 32;

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
pub(crate) struct SpireIndexOptionsSnapshot {
    pub(crate) nlists: i32,
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

fn assignment_payload_scannability(
    format: quantizer::SpireAssignmentPayloadFormat,
) -> (bool, &'static str, &'static str) {
    match format {
        quantizer::SpireAssignmentPayloadFormat::TurboQuant
        | quantizer::SpireAssignmentPayloadFormat::RaBitQ => (
            true,
            "supported",
            "format can be used for current SPIRE scans",
        ),
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => (
            false,
            "deferred_model_metadata",
            "persist grouped-PQ model metadata before using pq_fastscan with SPIRE",
        ),
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

fn leaf_maintenance_thresholds(effective_total: u64, leaf_count: u64) -> (u64, u64) {
    if leaf_count == 0 {
        return (0, 0);
    }
    let average = effective_total.div_ceil(leaf_count);
    let split_threshold = average.saturating_mul(4).max(32);
    let merge_threshold = average / 4;
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

fn hierarchy_snapshot_status(
    root_routing_object_count: u64,
    internal_routing_object_count: u64,
    leaf_object_count: u64,
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
    if internal_routing_object_count == 0 {
        return (
            "single_level_foundation",
            "recursive build coordinator and level-aware scan routing are not implemented",
        );
    }
    (
        "hierarchy_metadata_present",
        "recursive build coordinator and level-aware scan routing are not implemented",
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

pub(crate) unsafe fn index_options_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexOptionsSnapshot {
    let result = (|| -> Result<SpireIndexOptionsSnapshot, String> {
        let relation_options = unsafe { options::relation_options(index_relation) };
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
            scan::count_snapshot_single_level_leaf_pids(&snapshot, &object_store)?
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
            scan::count_snapshot_single_level_leaf_pids(&snapshot, &object_store)?
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
        manifests
            .sort_by_key(|(tid, manifest)| (manifest.epoch, tid.block_number, tid.offset_number));

        let now_micros = unsafe { pg_sys::GetCurrentTimestamp() };
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
                let cleanup_blocked_reason = if is_latest_manifest {
                    epoch_cleanup_blocked_reason(
                        &manifest,
                        now_micros,
                        is_active_root_manifest,
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
        let mut rows_by_leaf_pid: HashMap<u64, SpireIndexLeafSnapshotRow> = HashMap::new();

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "leaf snapshot")?;
            let placement = lookup.placement;
            if placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let header = unsafe { object_store.read_object_header(placement)? };
            match header.kind {
                storage::SpirePartitionObjectKind::Leaf => {
                    rows_by_leaf_pid.insert(
                        header.pid,
                        SpireIndexLeafSnapshotRow {
                            active_epoch: root_control.active_epoch,
                            leaf_pid: header.pid,
                            parent_pid: header.parent_pid,
                            object_version: header.object_version,
                            node_id: placement.node_id,
                            local_store_id: placement.local_store_id,
                            placement_state: placement_state_name(placement.state),
                            base_assignment_count: u64::from(header.assignment_count),
                            delta_object_count: 0,
                            delta_insert_assignment_count: 0,
                            delta_delete_assignment_count: 0,
                            effective_assignment_count: u64::from(header.assignment_count),
                            split_assignment_threshold: 0,
                            merge_assignment_threshold: 0,
                            split_recommended: false,
                            merge_recommended: false,
                            maintenance_action: "none",
                            maintenance_reason: "not_evaluated",
                            leaf_object_bytes: u64::from(placement.object_bytes),
                            delta_object_bytes: 0,
                        },
                    );
                }
                storage::SpirePartitionObjectKind::Delta => {
                    let delta_object = unsafe { object_store.read_delta_object(placement)? };
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
                        } else if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
                            != 0
                        {
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
            let (status, recommendation) = hierarchy_snapshot_status(0, 0, 0);
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
                }
                storage::SpirePartitionObjectKind::Leaf => {
                    leaf_object_count = leaf_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire hierarchy snapshot leaf object count overflow".to_owned()
                    })?;
                    leaf_parent_pids.insert(header.parent_pid);
                }
                storage::SpirePartitionObjectKind::Delta => {
                    delta_object_count = delta_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire hierarchy snapshot delta object count overflow".to_owned()
                    })?;
                }
            }
        }

        let hierarchy_depth = if root_routing_object_count == 0 {
            0
        } else {
            max_observed_level.max(root_level)
        };
        let (status, recommendation) = hierarchy_snapshot_status(
            root_routing_object_count,
            internal_routing_object_count,
            leaf_object_count,
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
            recursive_routing_supported: false,
            per_level_nprobe_supported: false,
            status,
            recommendation,
        })
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
        let rows = root_object
            .children()
            .map(|child| {
                let child_lookup =
                    snapshot.require_lookup(child.child_pid, "root routing child")?;
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
            .collect::<Result<Vec<_>, String>>()?;
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
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
