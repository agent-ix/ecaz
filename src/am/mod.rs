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
    build_vamana_graph_with_stats, greedy_search, greedy_search_view, MetricSummary,
    VamanaBuildPassStats, VamanaBuildStats, VamanaGraph, VamanaGraphView,
};
#[allow(unused_imports)]
pub(crate) use self::ec_hnsw::{
    graph, page, IndexAdminSnapshot, IndexCostSnapshot, PlannerIntegrationSnapshot,
};
pub(crate) use self::ec_ivf::{
    IndexAdminSnapshot as IvfIndexAdminSnapshot, IndexCostSnapshot as IvfIndexCostSnapshot,
    IndexDriftSnapshot, IndexPageOwnershipSnapshot as IvfIndexPageOwnershipSnapshot,
};
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ec_spire::custom_scan_cleanup_counters_for_test;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ec_spire::custom_scan_dml_plan_private_copy_roundtrip_for_test as spire_custom_scan_dml_plan_private_copy_roundtrip_for_test;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ec_spire::custom_scan_rescan_snapshot_for_test;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ec_spire::custom_scan_reset_cleanup_counters_for_test;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ec_spire::custom_scan_reset_rescan_snapshot_for_test;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ec_spire::custom_scan_store_tuple_payload_json_for_test as spire_custom_scan_store_tuple_payload_json_for_test;
pub(crate) use self::ec_spire::{
    active_epoch as spire_active_epoch,
    active_snapshot_diagnostics as spire_active_snapshot_diagnostics,
    classify_centroid as spire_classify_centroid,
    classify_dml_frontdoor_query as spire_classify_dml_frontdoor_query,
    coordinator_delete_prepare_remote_tuple_payload as spire_coordinator_delete_prepare_remote_tuple_payload,
    coordinator_insert_dispatch_plan_row as spire_coordinator_insert_dispatch_plan_row,
    coordinator_insert_prepare_remote_sql as spire_coordinator_insert_prepare_remote_sql,
    coordinator_insert_prepare_remote_tuple_payload as spire_coordinator_insert_prepare_remote_tuple_payload,
    coordinator_insert_prepare_remote_tuple_payload_batch as spire_coordinator_insert_prepare_remote_tuple_payload_batch,
    coordinator_select_remote_tuple_payload as spire_coordinator_select_remote_tuple_payload,
    coordinator_update_remote_tuple_payload as spire_coordinator_update_remote_tuple_payload,
    custom_scan_index_eligibility_row as spire_custom_scan_index_eligibility_row,
    custom_scan_status_row as spire_custom_scan_status_row,
    dml_frontdoor_bigint_pk_value_bytes as spire_dml_frontdoor_bigint_pk_value_bytes,
    dml_frontdoor_hook_status_row as spire_dml_frontdoor_hook_status_row,
    dml_frontdoor_pk_argument_from_replacement_decision as spire_dml_frontdoor_pk_argument_from_replacement_decision,
    dml_frontdoor_primitive_invocation_from_plan as spire_dml_frontdoor_primitive_invocation_from_plan,
    dml_frontdoor_primitive_plan_const_pk_value_bytes as spire_dml_frontdoor_primitive_plan_const_pk_value_bytes,
    dml_frontdoor_primitive_plan_expr_catalog_row as spire_dml_frontdoor_primitive_plan_expr_catalog_row,
    dml_frontdoor_primitive_plan_from_replacement_decision as spire_dml_frontdoor_primitive_plan_from_replacement_decision,
    dml_frontdoor_primitive_plan_pk_value_bytes as spire_dml_frontdoor_primitive_plan_pk_value_bytes,
    dml_frontdoor_relation_context_cache_row as spire_dml_frontdoor_relation_context_cache_row,
    dml_frontdoor_relation_context_catalog_row as spire_dml_frontdoor_relation_context_catalog_row,
    dml_frontdoor_relation_context_row as spire_dml_frontdoor_relation_context_row,
    dml_frontdoor_replacement_decision_catalog_row as spire_dml_frontdoor_replacement_decision_catalog_row,
    dml_frontdoor_target_relation_oid as spire_dml_frontdoor_target_relation_oid,
    index_allocator_snapshot as spire_index_allocator_snapshot,
    index_boundary_replica_identity_snapshot as spire_index_boundary_replica_identity_snapshot,
    index_boundary_replica_placement_diagnostics as spire_index_boundary_replica_placement_diagnostics,
    index_cost_snapshot as spire_index_cost_snapshot,
    index_cost_tuning_snapshot as spire_index_cost_tuning_snapshot,
    index_delta_snapshot as spire_index_delta_snapshot,
    index_epoch_cleanup_run as spire_index_epoch_cleanup_run,
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
    index_scan_routing_snapshot as spire_index_scan_routing_snapshot,
    index_scan_sanity_snapshot as spire_index_scan_sanity_snapshot,
    index_selected_pid_placement_snapshot as spire_index_selected_pid_placement_snapshot,
    index_top_graph_snapshot as spire_index_top_graph_snapshot,
    index_writer_identity_snapshot as spire_index_writer_identity_snapshot,
    reap_orphaned_remote_prepared_xacts as spire_reap_orphaned_remote_prepared_xacts,
    reap_orphaned_remote_prepared_xacts_all as spire_reap_orphaned_remote_prepared_xacts_all,
    remote_catalog_lifecycle_contract_rows as spire_remote_catalog_lifecycle_contract_rows,
    remote_conninfo_secret_provider_lookup_key as spire_remote_conninfo_secret_provider_lookup_key,
    remote_conninfo_secret_resolution_contract_rows as spire_remote_conninfo_secret_resolution_contract_rows,
    remote_conninfo_secret_resolution_status_row as spire_remote_conninfo_secret_resolution_status_row,
    remote_degradation_policy_contract_rows as spire_remote_degradation_policy_contract_rows,
    remote_epoch_manifest_plan as spire_remote_epoch_manifest_plan,
    remote_epoch_manifest_publication_contract_rows as spire_remote_epoch_manifest_publication_contract_rows,
    remote_epoch_manifest_summary as spire_remote_epoch_manifest_summary,
    remote_epoch_publish_gate_summary as spire_remote_epoch_publish_gate_summary,
    remote_epoch_publish_plan as spire_remote_epoch_publish_plan,
    remote_epoch_publish_readiness as spire_remote_epoch_publish_readiness,
    remote_libpq_connection_lifecycle_contract_rows as spire_remote_libpq_connection_lifecycle_contract_rows,
    remote_node_capability_plan as spire_remote_node_capability_plan,
    remote_node_capability_summary as spire_remote_node_capability_summary,
    remote_node_descriptor_catalog_state_is_supported as spire_remote_node_descriptor_catalog_state_is_supported,
    remote_node_descriptor_contract_rows as spire_remote_node_descriptor_contract_rows,
    remote_node_descriptor_readiness as spire_remote_node_descriptor_readiness,
    remote_node_descriptor_readiness_summary as spire_remote_node_descriptor_readiness_summary,
    remote_node_descriptor_registration_contract_rows as spire_remote_node_descriptor_registration_contract_rows,
    remote_node_descriptor_state_contract_rows as spire_remote_node_descriptor_state_contract_rows,
    remote_node_snapshot as spire_remote_node_snapshot,
    remote_operator_entrypoint_contract_rows as spire_remote_operator_entrypoint_contract_rows,
    remote_prepared_transaction_registration_warning as spire_remote_prepared_transaction_registration_warning,
    remote_search_candidates as spire_remote_search_candidates,
    remote_search_coordinator_gate_summary_row as spire_remote_search_coordinator_gate_summary_row,
    remote_search_coordinator_local_candidates as spire_remote_search_coordinator_local_candidates,
    remote_search_coordinator_local_summary as spire_remote_search_coordinator_local_summary,
    remote_search_coordinator_result_summary_row as spire_remote_search_coordinator_result_summary_row,
    remote_search_endpoint_contract_rows as spire_remote_search_endpoint_contract_rows,
    remote_search_endpoint_identity_row as spire_remote_search_endpoint_identity_row,
    remote_search_execution_plan_rows as spire_remote_search_execution_plan_rows,
    remote_search_execution_summary_row as spire_remote_search_execution_summary_row,
    remote_search_fanout_plan_rows as spire_remote_search_fanout_plan_rows,
    remote_search_finalization_summary_row as spire_remote_search_finalization_summary_row,
    remote_search_heap_resolution_contract_rows as spire_remote_search_heap_resolution_contract_rows,
    remote_search_heap_resolution_summary_row as spire_remote_search_heap_resolution_summary_row,
    remote_search_libpq_connect_with_session_timeouts as spire_remote_search_libpq_connect_with_session_timeouts,
    remote_search_libpq_connection_open_plan_rows as spire_remote_search_libpq_connection_open_plan_rows,
    remote_search_libpq_connection_open_summary_row as spire_remote_search_libpq_connection_open_summary_row,
    remote_search_libpq_connection_plan_rows as spire_remote_search_libpq_connection_plan_rows,
    remote_search_libpq_connection_summary_row as spire_remote_search_libpq_connection_summary_row,
    remote_search_libpq_dispatch_plan_rows as spire_remote_search_libpq_dispatch_plan_rows,
    remote_search_libpq_dispatch_summary_row as spire_remote_search_libpq_dispatch_summary_row,
    remote_search_libpq_executor_budget_summary_row as spire_remote_search_libpq_executor_budget_summary_row,
    remote_search_libpq_executor_candidate_rows as spire_remote_search_libpq_executor_candidate_rows,
    remote_search_libpq_executor_heap_candidate_rows as spire_remote_search_libpq_executor_heap_candidate_rows,
    remote_search_libpq_executor_readiness_row as spire_remote_search_libpq_executor_readiness_row,
    remote_search_libpq_executor_receive_attempt_rows as spire_remote_search_libpq_executor_receive_attempt_rows,
    remote_search_libpq_executor_step_contract_rows as spire_remote_search_libpq_executor_step_contract_rows,
    remote_search_libpq_identity_cache_summary_row as spire_remote_search_libpq_identity_cache_summary_row,
    remote_search_libpq_parameter_contract_rows as spire_remote_search_libpq_parameter_contract_rows,
    remote_search_libpq_request_plan_rows as spire_remote_search_libpq_request_plan_rows,
    remote_search_libpq_request_summary_row as spire_remote_search_libpq_request_summary_row,
    remote_search_libpq_result_contract_rows as spire_remote_search_libpq_result_contract_rows,
    remote_search_libpq_secret_plan_rows as spire_remote_search_libpq_secret_plan_rows,
    remote_search_libpq_secret_summary_row as spire_remote_search_libpq_secret_summary_row,
    remote_search_local_heap_candidate_rows as spire_remote_search_local_heap_candidate_rows,
    remote_search_local_heap_candidate_summary_row as spire_remote_search_local_heap_candidate_summary_row,
    remote_search_local_heap_resolution_plan_rows as spire_remote_search_local_heap_resolution_plan_rows,
    remote_search_merge_input_summary_row as spire_remote_search_merge_input_summary_row,
    remote_search_merge_order_contract_rows as spire_remote_search_merge_order_contract_rows,
    remote_search_operator_diagnostics_row as spire_remote_search_operator_diagnostics_row,
    remote_search_production_consistency_policy_summary_row as spire_remote_search_production_consistency_policy_summary_row,
    remote_search_production_degraded_skip_report_rows as spire_remote_search_production_degraded_skip_report_rows,
    remote_search_production_executor_session_summary_row as spire_remote_search_production_executor_session_summary_row,
    remote_search_production_executor_state_summary_row as spire_remote_search_production_executor_state_summary_row,
    remote_search_production_fault_matrix_rows as spire_remote_search_production_fault_matrix_rows,
    remote_search_production_scan_handoff_summary_row as spire_remote_search_production_scan_handoff_summary_row,
    remote_search_production_scan_heap_resolution_summary_row as spire_remote_search_production_scan_heap_resolution_summary_row,
    remote_search_production_session_consistency_policy_summary_row as spire_remote_search_production_session_consistency_policy_summary_row,
    remote_search_readiness_summary_row as spire_remote_search_readiness_summary_row,
    remote_search_receive_plan_rows as spire_remote_search_receive_plan_rows,
    remote_search_request_plan_rows as spire_remote_search_request_plan_rows,
    remote_search_request_readiness_rows as spire_remote_search_request_readiness_rows,
    remote_search_request_summary_row as spire_remote_search_request_summary_row,
    remote_search_row_locator_contract_rows as spire_remote_search_row_locator_contract_rows,
    remote_search_stage_e_fault_matrix_rows as spire_remote_search_stage_e_fault_matrix_rows,
    remote_search_stage_e_lifecycle_matrix_rows as spire_remote_search_stage_e_lifecycle_matrix_rows,
    remote_search_target_plan_rows as spire_remote_search_target_plan_rows,
    remote_search_target_readiness_rows as spire_remote_search_target_readiness_rows,
    remote_search_vector_identity_contract_rows as spire_remote_search_vector_identity_contract_rows,
    remote_write_shape_fingerprint_from_secret as spire_remote_write_shape_fingerprint_from_secret,
    SpireDmlFrontdoorCustomScanMode, SpireDmlFrontdoorPkValuePlan, SpireDmlFrontdoorQueryContext,
};

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ec_spire::remote_search_libpq_executor_budget_contract_probe_counts as spire_remote_search_libpq_executor_budget_contract_probe_counts;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ec_spire::remote_search_libpq_identity_cache_contract_probe_counts as spire_remote_search_libpq_identity_cache_contract_probe_counts;

pub(crate) fn register_gucs() {
    ec_diskann::register_gucs();
    ec_hnsw::register_gucs();
    ec_ivf::register_gucs();
    ec_spire::register_gucs();
}

pub(crate) unsafe fn register_custom_scan() {
    unsafe { ec_spire::register_custom_scan() };
}

pub(crate) unsafe fn register_dml_frontdoor_planner_hook() {
    unsafe { ec_spire::register_dml_frontdoor_planner_hook() };
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
    debug_spire_active_snapshot_diagnostics, debug_spire_age_retired_epoch_manifests,
    debug_spire_empty_manifest_publish_roundtrip, debug_spire_relation_leaf_v2_roundtrip,
    debug_spire_relation_object_tuple_roundtrip, debug_spire_relation_two_store_scan_roundtrip,
    debug_spire_rewrite_consistency_mode, debug_spire_rewrite_placement_node,
    debug_spire_rewrite_placement_nodes, debug_spire_rewrite_placement_state,
    debug_spire_root_control, debug_spire_vacuum_bulkdelete_heap_tids,
    debug_spire_vacuum_remove_heap_tids,
    remote_search_libpq_global_governance_advisory_key_for_test,
    remote_search_libpq_node_governance_advisory_key_for_test,
    remote_search_production_candidate_receive_for_test as spire_remote_search_production_candidate_receive_for_test,
    remote_search_production_candidate_receive_summary_for_test as spire_remote_search_production_candidate_receive_summary_for_test,
    remote_search_production_candidate_receive_with_local_cancel_for_test as spire_remote_search_production_candidate_receive_with_local_cancel_for_test,
    remote_search_production_transport_probe_for_test as spire_remote_search_production_transport_probe_for_test,
    remote_search_production_transport_probe_summary_for_test as spire_remote_search_production_transport_probe_summary_for_test,
    remote_search_production_transport_probe_with_local_cancel_for_test as spire_remote_search_production_transport_probe_with_local_cancel_for_test,
    remote_search_production_transport_probe_with_local_cancel_summary_for_test as spire_remote_search_production_transport_probe_with_local_cancel_summary_for_test,
    SpireRemoteProductionCandidateReceiveRequest, SpireRemoteProductionTransportProbeRequest,
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
