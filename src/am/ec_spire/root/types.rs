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
    pub(crate) local_store_count: i32,
    pub(crate) local_store_tablespaces: Option<String>,
    pub(crate) boundary_replica_count: i32,
    pub(crate) boundary_replication_enabled: bool,
    pub(crate) scan_dedupe_mode: &'static str,
    pub(crate) active_leaf_count: u32,
    pub(crate) relation_nprobe: i32,
    pub(crate) session_nprobe: Option<i32>,
    pub(crate) effective_nprobe: u32,
    pub(crate) effective_nprobe_source: &'static str,
    pub(crate) effective_nprobe_per_level: Vec<u32>,
    pub(crate) nprobe_policy_per_level: Vec<&'static str>,
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
pub(crate) struct SpireIndexLevelParameterSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) level: u16,
    pub(crate) routing_object_count: u64,
    pub(crate) routing_child_count: u64,
    pub(crate) target_fanout: u32,
    pub(crate) relation_nprobe: i32,
    pub(crate) session_nprobe: Option<i32>,
    pub(crate) effective_nprobe: u32,
    pub(crate) effective_nprobe_source: &'static str,
    pub(crate) nprobe_policy: &'static str,
    pub(crate) training_sample_rows: i32,
    pub(crate) training_iterations: u64,
    pub(crate) centroid_dimensions: u16,
    pub(crate) distance_operator: &'static str,
    pub(crate) assignment_payload_format: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireIndexTopGraphSnapshot {
    pub(crate) active_epoch: u64,
    pub(crate) top_graph_enabled: bool,
    pub(crate) top_graph_count: u64,
    pub(crate) top_graph_pid: u64,
    pub(crate) root_pid: u64,
    pub(crate) object_version: u64,
    pub(crate) published_epoch_backref: u64,
    pub(crate) level: u16,
    pub(crate) node_count: u64,
    pub(crate) dimensions: u16,
    pub(crate) graph_degree: u32,
    pub(crate) build_list_size: u32,
    pub(crate) alpha: f32,
    pub(crate) entry_node: u64,
    pub(crate) edge_count: u64,
    pub(crate) max_node_degree: u64,
    pub(crate) effective_route_count: u32,
    pub(crate) effective_search_list_size: u32,
    pub(crate) configured_search_list_size: Option<u32>,
    pub(crate) object_bytes: u64,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
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
    pub(crate) base_primary_assignment_count: u64,
    pub(crate) base_boundary_replica_assignment_count: u64,
    pub(crate) delta_object_count: u64,
    pub(crate) delta_insert_assignment_count: u64,
    pub(crate) delta_boundary_replica_insert_assignment_count: u64,
    pub(crate) delta_delete_assignment_count: u64,
    pub(crate) effective_assignment_count: u64,
    pub(crate) effective_boundary_replica_assignment_count: u64,
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
    pub(crate) store_relid: u32,
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
    pub(crate) dropped_unselected_delta_route_count: u64,
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
