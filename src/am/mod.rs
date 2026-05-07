//! Access-method surfaces grouped by AM and shared helpers.

pub(crate) mod common;
mod ec_diskann;
mod ec_hnsw;
mod ec_ivf;
mod ec_spire;

#[allow(unused_imports)]
pub(crate) use self::common::{cost, explain, stats, stream};
pub(crate) use self::ec_diskann::diagnostics::DiskannGraphSummary;
pub use self::ec_diskann::vamana::{
    approximate_medoid, bfs_reachable, build_vamana_graph_with_pass1_extra_candidates,
    build_vamana_graph_with_stats, greedy_search, MetricSummary, VamanaBuildPassStats,
    VamanaBuildStats, VamanaGraph,
};
#[allow(unused_imports)]
pub(crate) use self::ec_hnsw::{
    graph, page, IndexAdminSnapshot, IndexCostSnapshot, PlannerIntegrationSnapshot,
};
pub(crate) use self::ec_ivf::{
    IndexAdminSnapshot as IvfIndexAdminSnapshot, IndexCostSnapshot as IvfIndexCostSnapshot,
    IndexDriftSnapshot, IndexPageOwnershipSnapshot as IvfIndexPageOwnershipSnapshot,
};
pub(crate) use self::ec_spire::{
    active_snapshot_diagnostics as spire_active_snapshot_diagnostics,
    index_allocator_snapshot as spire_index_allocator_snapshot,
    index_delta_snapshot as spire_index_delta_snapshot,
    index_epoch_snapshot as spire_index_epoch_snapshot,
    index_health_snapshot as spire_index_health_snapshot,
    index_hierarchy_snapshot as spire_index_hierarchy_snapshot,
    index_insert_debt_snapshot as spire_index_insert_debt_snapshot,
    index_leaf_snapshot as spire_index_leaf_snapshot,
    index_level_parameter_snapshot as spire_index_level_parameter_snapshot,
    index_locked_maintenance_plan_snapshot as spire_index_locked_maintenance_plan_snapshot,
    index_locked_maintenance_run_plan as spire_index_locked_maintenance_run_plan,
    index_maintenance_plan_snapshot as spire_index_maintenance_plan_snapshot,
    index_maintenance_run as spire_index_maintenance_run,
    index_object_snapshot as spire_index_object_snapshot,
    index_options_snapshot as spire_index_options_snapshot,
    index_placement_snapshot as spire_index_placement_snapshot,
    index_relation_storage_snapshot as spire_index_relation_storage_snapshot,
    index_root_routing_snapshot as spire_index_root_routing_snapshot,
    index_routing_centroid_snapshot as spire_index_routing_centroid_snapshot,
    index_scan_placement_snapshot as spire_index_scan_placement_snapshot,
    index_scan_sanity_snapshot as spire_index_scan_sanity_snapshot,
    index_top_graph_snapshot as spire_index_top_graph_snapshot,
    remote_epoch_publish_readiness as spire_remote_epoch_publish_readiness,
    remote_node_capability_plan as spire_remote_node_capability_plan,
    remote_node_capability_summary as spire_remote_node_capability_summary,
    remote_node_snapshot as spire_remote_node_snapshot,
    remote_search_candidates as spire_remote_search_candidates,
    remote_search_coordinator_local_candidates as spire_remote_search_coordinator_local_candidates,
    remote_search_coordinator_local_summary as spire_remote_search_coordinator_local_summary,
    remote_search_fanout_plan_rows as spire_remote_search_fanout_plan_rows,
    remote_search_readiness_summary_row as spire_remote_search_readiness_summary_row,
    remote_search_request_plan_rows as spire_remote_search_request_plan_rows,
    remote_search_request_readiness_rows as spire_remote_search_request_readiness_rows,
    remote_search_request_summary_row as spire_remote_search_request_summary_row,
    remote_search_target_plan_rows as spire_remote_search_target_plan_rows,
    remote_search_target_readiness_rows as spire_remote_search_target_readiness_rows,
};

pub(crate) fn register_gucs() {
    ec_diskann::register_gucs();
    ec_hnsw::register_gucs();
    ec_ivf::register_gucs();
    ec_spire::register_gucs();
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ec_hnsw::{
    resolve_pq_fastscan_rerank_mode_decision, resolve_pq_fastscan_traversal_score_mode_decision,
    PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW, PQ_FASTSCAN_DEFAULT_RERANK_MODE_NAME,
    PQ_FASTSCAN_DEFAULT_TRAVERSAL_SCORE_MODE_NAME,
};

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ec_ivf::{
    debug_ec_ivf_build_metadata, debug_ec_ivf_directory_entry, debug_ec_ivf_directory_summary,
    debug_ec_ivf_gettuple_after_rescan_result, debug_ec_ivf_gettuple_outputs,
    debug_ec_ivf_metadata, debug_ec_ivf_pq_fastscan_model_cache_reused,
    debug_ec_ivf_quantizer_cache_ptr, debug_ec_ivf_rerank_mode, debug_ec_ivf_rescan_query_prep,
    debug_ec_ivf_vacuum_remove_heap_tids, debug_ec_ivf_vacuum_stats,
    debug_ec_ivf_validate_no_duplicate_heap_tid,
};

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ec_spire::{
    debug_spire_active_snapshot_diagnostics, debug_spire_empty_manifest_publish_roundtrip,
    debug_spire_relation_leaf_v2_roundtrip, debug_spire_relation_object_tuple_roundtrip,
    debug_spire_relation_two_store_scan_roundtrip, debug_spire_rewrite_consistency_mode,
    debug_spire_rewrite_placement_node, debug_spire_rewrite_placement_state,
    debug_spire_root_control, debug_spire_vacuum_bulkdelete_heap_tids,
    debug_spire_vacuum_remove_heap_tids,
};

pub(crate) unsafe fn index_cost_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> IndexCostSnapshot {
    unsafe { ec_hnsw::index_cost_snapshot(index_relation) }
}

pub(crate) unsafe fn index_admin_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> IndexAdminSnapshot {
    unsafe { ec_hnsw::index_admin_snapshot(index_relation) }
}

pub(crate) unsafe fn planner_integration_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> PlannerIntegrationSnapshot {
    unsafe { ec_hnsw::planner_integration_snapshot(index_relation) }
}

pub(crate) unsafe fn ivf_index_drift_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> IndexDriftSnapshot {
    unsafe { ec_ivf::index_drift_snapshot(index_relation) }
}

pub(crate) unsafe fn ivf_index_admin_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> IvfIndexAdminSnapshot {
    unsafe { ec_ivf::index_admin_snapshot(index_relation) }
}

pub(crate) unsafe fn ivf_index_cost_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> IvfIndexCostSnapshot {
    unsafe { ec_ivf::index_cost_snapshot(index_relation) }
}

pub(crate) unsafe fn ivf_index_page_ownership(
    index_relation: pgrx::pg_sys::Relation,
) -> Vec<IvfIndexPageOwnershipSnapshot> {
    unsafe { ec_ivf::index_page_ownership(index_relation) }
}

pub(crate) unsafe fn diskann_graph_summary(
    index_relation: pgrx::pg_sys::Relation,
) -> Result<DiskannGraphSummary, String> {
    unsafe { ec_diskann::diagnostics::graph_summary(index_relation) }
}

#[cfg(any(test, feature = "pg_test"))]
#[allow(unused_imports)]
pub(crate) use self::ec_hnsw::{
    debug_all_top_level_heap_tids, debug_begin_end_scan, debug_bootstrap_phase_transition,
    debug_candidate_frontier_head_lifecycle, debug_consume_candidate_frontier_head,
    debug_consume_candidate_frontier_head_slots, debug_end_scan_twice,
    debug_entry_candidate_lifecycle, debug_entry_point_neighbor_tids,
    debug_exact_seed_scan_heap_tids, debug_gettuple_after_rescan_result,
    debug_gettuple_backward_after_rescan, debug_gettuple_consumes_bootstrap_candidate,
    debug_gettuple_current_result_heap_progress, debug_gettuple_current_result_lifecycle,
    debug_gettuple_current_result_neighbors, debug_gettuple_current_result_state,
    debug_gettuple_exhaustion_state, debug_gettuple_orderby_score,
    debug_gettuple_orderby_score_lifecycle, debug_gettuple_rescan_after_exhaustion,
    debug_gettuple_rescan_after_partial, debug_gettuple_scan_heap_tids,
    debug_gettuple_scan_heap_tids_with_score_comparisons,
    debug_gettuple_scan_heap_tids_with_scores, debug_gettuple_without_rescan,
    debug_grouped_rerank_profile, debug_grouped_scan_comparison_rows,
    debug_grouped_scan_comparison_summary, debug_grouped_scan_order_drift_summary,
    debug_grouped_scan_windowed_rows, debug_grouped_scan_windowed_summary, debug_index_metadata,
    debug_index_pages, debug_insert_level_for_heap_tid, debug_last_build_timing,
    debug_last_parallel_build_workers_launched, debug_last_parallel_graph_build_workers_launched,
    debug_layer0_reachable_live_element_tids, debug_layer_oracle_k_carrydown_scan_heap_tids,
    debug_layer_oracle_k_seed_layer0_neighbor_heap_tids,
    debug_materialize_bootstrap_candidate_result, debug_planner_tuning_snapshot,
    debug_profile_ordered_scan, debug_profile_ordered_scan_with_heap_fetch,
    debug_profile_ordered_scan_with_limit, debug_rescan_candidate_frontier,
    debug_rescan_entry_candidate_state, debug_rescan_null_query,
    debug_rescan_overwrites_query_dimensions, debug_rescan_query_dimensions,
    debug_rescan_successor_candidate_state, debug_rescan_with_index_qual,
    debug_rescan_with_multiple_orderbys, debug_rescan_with_unused_key_buffer,
    debug_top_level_oracle_k_seed_heap_tids, debug_top_level_oracle_k_seed_scan_heap_tids,
    debug_top_level_oracle_scan_heap_tids, debug_top_level_reachable_heap_tids,
    debug_turboquant_scan_stage_profile, debug_update_index_metadata,
    debug_vacuum_remove_heap_tids, debug_vacuum_stats, debug_visited_seed_lifecycle,
    DebugIndexDataPage, DebugPlannerTuningSnapshot,
};
