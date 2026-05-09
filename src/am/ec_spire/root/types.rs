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
    pub(crate) recursive_beam_width: u64,
    pub(crate) max_leaf_routes: u64,
    pub(crate) max_routing_expansions: u64,
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
    pub(crate) frontier_kind: &'static str,
    pub(crate) frontier_parent_level: u16,
    pub(crate) frontier_child_level: u16,
    pub(crate) frontier_node_count: u64,
    pub(crate) root_child_count: u64,
    pub(crate) active_leaf_count: u64,
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
    pub(crate) object_tuple_count: u64,
    pub(crate) object_segment_count: u64,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteSearchCandidateRow {
    pub(crate) served_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) pid: u64,
    pub(crate) object_version: u64,
    pub(crate) row_index: u32,
    pub(crate) assignment_flags: u16,
    pub(crate) vec_id: Vec<u8>,
    pub(crate) row_locator: Vec<u8>,
    pub(crate) score: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLocalHeapResolutionPlanRow {
    pub(crate) requested_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) pid: u64,
    pub(crate) row_index: u32,
    pub(crate) vec_id: Vec<u8>,
    pub(crate) row_locator: Vec<u8>,
    pub(crate) heap_block: u32,
    pub(crate) heap_offset: u16,
    pub(crate) heap_lookup_owner: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteSearchLocalHeapCandidateRow {
    pub(crate) requested_epoch: u64,
    pub(crate) served_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) pid: u64,
    pub(crate) object_version: u64,
    pub(crate) row_index: u32,
    pub(crate) assignment_flags: u16,
    pub(crate) vec_id: Vec<u8>,
    pub(crate) row_locator: Vec<u8>,
    pub(crate) heap_block: u32,
    pub(crate) heap_offset: u16,
    pub(crate) score: f32,
    pub(crate) heap_lookup_owner: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLocalHeapCandidateSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) local_plan_count: u64,
    pub(crate) remote_plan_count: u64,
    pub(crate) skipped_plan_count: u64,
    pub(crate) local_pid_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) decoded_local_locator_count: u64,
    pub(crate) returned_candidate_count: u64,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchCoordinatorResultSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) local_plan_count: u64,
    pub(crate) remote_plan_count: u64,
    pub(crate) skipped_plan_count: u64,
    pub(crate) local_pid_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) skipped_pid_count: u64,
    pub(crate) decoded_local_locator_count: u64,
    pub(crate) returned_candidate_count: u64,
    pub(crate) result_source: &'static str,
    pub(crate) libpq_receive_count: u64,
    pub(crate) libpq_receive_status: &'static str,
    pub(crate) final_heap_fetch_status: &'static str,
    pub(crate) next_blocker: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchHeapResolutionSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) local_plan_count: u64,
    pub(crate) remote_plan_count: u64,
    pub(crate) skipped_plan_count: u64,
    pub(crate) local_pid_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) decoded_local_locator_count: u64,
    pub(crate) local_heap_resolution_status: &'static str,
    pub(crate) remote_heap_resolution_status: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchFanoutPlanRow {
    pub(crate) requested_epoch: u64,
    pub(crate) target_kind: &'static str,
    pub(crate) node_id: u32,
    pub(crate) pid: u64,
    pub(crate) placement_state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchTargetPlanRow {
    pub(crate) requested_epoch: u64,
    pub(crate) target_kind: &'static str,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) pid_count: u64,
    pub(crate) placement_state: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchTargetReadinessRow {
    pub(crate) requested_epoch: u64,
    pub(crate) target_kind: &'static str,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) pid_count: u64,
    pub(crate) placement_state: &'static str,
    pub(crate) node_kind: &'static str,
    pub(crate) descriptor_state: &'static str,
    pub(crate) node_status: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchRequestPlanRow {
    pub(crate) requested_epoch: u64,
    pub(crate) target_kind: &'static str,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) pid_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) endpoint_function: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchRequestReadinessRow {
    pub(crate) requested_epoch: u64,
    pub(crate) target_kind: &'static str,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) pid_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) endpoint_function: &'static str,
    pub(crate) node_kind: &'static str,
    pub(crate) descriptor_state: &'static str,
    pub(crate) node_status: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchRequestSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) request_count: u64,
    pub(crate) local_request_count: u64,
    pub(crate) remote_request_count: u64,
    pub(crate) skipped_request_count: u64,
    pub(crate) executable_pid_count: u64,
    pub(crate) local_pid_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) skipped_pid_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchReadinessSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) request_count: u64,
    pub(crate) ready_request_count: u64,
    pub(crate) blocked_request_count: u64,
    pub(crate) local_request_count: u64,
    pub(crate) remote_request_count: u64,
    pub(crate) skipped_request_count: u64,
    pub(crate) executable_pid_count: u64,
    pub(crate) ready_pid_count: u64,
    pub(crate) blocked_pid_count: u64,
    pub(crate) skipped_pid_count: u64,
    pub(crate) missing_descriptor_request_count: u64,
    pub(crate) missing_descriptor_pid_count: u64,
    pub(crate) transport_request_count: u64,
    pub(crate) transport_pid_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchExecutionPlanRow {
    pub(crate) requested_epoch: u64,
    pub(crate) target_kind: &'static str,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) pid_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) execution_transport: &'static str,
    pub(crate) endpoint_function: &'static str,
    pub(crate) remote_index_source: &'static str,
    pub(crate) conninfo_source: &'static str,
    pub(crate) candidate_format: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchExecutionSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) plan_count: u64,
    pub(crate) local_plan_count: u64,
    pub(crate) remote_plan_count: u64,
    pub(crate) skipped_plan_count: u64,
    pub(crate) ready_plan_count: u64,
    pub(crate) blocked_plan_count: u64,
    pub(crate) degraded_skipped_plan_count: u64,
    pub(crate) local_pid_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) skipped_pid_count: u64,
    pub(crate) blocked_pid_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqRequestPlanRow {
    pub(crate) requested_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) pid_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) execution_transport: &'static str,
    pub(crate) sql_template: &'static str,
    pub(crate) parameter_count: u64,
    pub(crate) result_column_count: u64,
    pub(crate) remote_index_source: &'static str,
    pub(crate) conninfo_source: &'static str,
    pub(crate) candidate_format: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqRequestSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) request_count: u64,
    pub(crate) ready_request_count: u64,
    pub(crate) blocked_request_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) blocked_pid_count: u64,
    pub(crate) parameter_count_per_request: u64,
    pub(crate) result_column_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqConnectionPlanRow {
    pub(crate) requested_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) pid_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) execution_transport: &'static str,
    pub(crate) conninfo_secret_name: String,
    pub(crate) remote_index_regclass: String,
    pub(crate) remote_index_identity_bytes: u64,
    pub(crate) conninfo_resolution: &'static str,
    pub(crate) pipeline_mode: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqConnectionSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) connection_count: u64,
    pub(crate) descriptor_resolved_connection_count: u64,
    pub(crate) missing_descriptor_connection_count: u64,
    pub(crate) pipeline_connection_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) blocked_pid_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteOperatorEntrypointContractRow {
    pub(crate) entrypoint_ordinal: u64,
    pub(crate) entrypoint_name: &'static str,
    pub(crate) area: &'static str,
    pub(crate) operator_use: &'static str,
    pub(crate) status_source: &'static str,
    pub(crate) next_action: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteLibpqConnectionLifecycleContractRow {
    pub(crate) surface: &'static str,
    pub(crate) connection_lifecycle_policy: &'static str,
    pub(crate) pooling_policy: &'static str,
    pub(crate) secret_resolution_policy: &'static str,
    pub(crate) conninfo_exposure_policy: &'static str,
    pub(crate) failure_policy: &'static str,
    pub(crate) resource_limit_policy: &'static str,
    pub(crate) validator: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteConninfoSecretResolutionContractRow {
    pub(crate) provider_ordinal: u64,
    pub(crate) provider_policy: &'static str,
    pub(crate) provider_status: &'static str,
    pub(crate) secret_reference_field: &'static str,
    pub(crate) sql_storage_policy: &'static str,
    pub(crate) raw_conninfo_allowed: bool,
    pub(crate) executor_action: &'static str,
    pub(crate) failure_status: &'static str,
    pub(crate) validator: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteConninfoSecretResolutionStatusRow {
    pub(crate) provider_policy: &'static str,
    pub(crate) conninfo_secret_name: String,
    pub(crate) provider_lookup_key: String,
    pub(crate) resolved_conninfo_bytes: u64,
    pub(crate) raw_conninfo_exposed: bool,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteCatalogLifecycleContractRow {
    pub(crate) lifecycle_ordinal: u64,
    pub(crate) lifecycle_event: &'static str,
    pub(crate) oid_stability: &'static str,
    pub(crate) catalog_risk: &'static str,
    pub(crate) operator_action: &'static str,
    pub(crate) cleanup_surface: &'static str,
    pub(crate) migration_surface: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqDispatchPlanRow {
    pub(crate) requested_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) pid_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) sql_template: &'static str,
    pub(crate) parameter_count: u64,
    pub(crate) result_column_count: u64,
    pub(crate) conninfo_secret_name: String,
    pub(crate) remote_index_regclass: String,
    pub(crate) pipeline_mode: &'static str,
    pub(crate) dispatch_action: &'static str,
    pub(crate) receive_validator: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqDispatchSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) dispatch_count: u64,
    pub(crate) pipeline_dispatch_count: u64,
    pub(crate) missing_descriptor_dispatch_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) blocked_pid_count: u64,
    pub(crate) query_dimension: u64,
    pub(crate) top_k: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqSecretPlanRow {
    pub(crate) requested_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) pid_count: u64,
    pub(crate) conninfo_secret_name: String,
    pub(crate) provider_lookup_key: String,
    pub(crate) resolved_conninfo_bytes: u64,
    pub(crate) raw_conninfo_exposed: bool,
    pub(crate) secret_resolution_action: &'static str,
    pub(crate) next_executor_step: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqSecretSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) secret_count: u64,
    pub(crate) resolved_secret_count: u64,
    pub(crate) blocked_secret_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) blocked_pid_count: u64,
    pub(crate) next_executor_step: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqConnectionOpenPlanRow {
    pub(crate) requested_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) pid_count: u64,
    pub(crate) conninfo_secret_name: String,
    pub(crate) provider_lookup_key: String,
    pub(crate) resolved_conninfo_bytes: u64,
    pub(crate) connection_lifecycle_policy: &'static str,
    pub(crate) pooling_policy: &'static str,
    pub(crate) connection_action: &'static str,
    pub(crate) next_executor_step: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqConnectionOpenSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) connection_count: u64,
    pub(crate) ready_connection_count: u64,
    pub(crate) blocked_connection_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) blocked_pid_count: u64,
    pub(crate) next_executor_step: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqExecutorReadinessRow {
    pub(crate) requested_epoch: u64,
    pub(crate) dispatch_count: u64,
    pub(crate) pipeline_dispatch_count: u64,
    pub(crate) blocked_dispatch_count: u64,
    pub(crate) secret_resolution_action: &'static str,
    pub(crate) connection_action: &'static str,
    pub(crate) pipeline_action: &'static str,
    pub(crate) send_action: &'static str,
    pub(crate) receive_action: &'static str,
    pub(crate) merge_action: &'static str,
    pub(crate) next_executor_step: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqExecutorStepContractRow {
    pub(crate) step_ordinal: u64,
    pub(crate) step_name: &'static str,
    pub(crate) executor_action: &'static str,
    pub(crate) input_contract: &'static str,
    pub(crate) output_contract: &'static str,
    pub(crate) blocking_status: &'static str,
    pub(crate) validator: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqParameterContractRow {
    pub(crate) parameter_ordinal: u64,
    pub(crate) parameter_name: &'static str,
    pub(crate) pg_type: &'static str,
    pub(crate) semantic_role: &'static str,
    pub(crate) validator: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchLibpqResultContractRow {
    pub(crate) column_ordinal: u64,
    pub(crate) column_name: &'static str,
    pub(crate) pg_type: &'static str,
    pub(crate) semantic_role: &'static str,
    pub(crate) nullable: bool,
    pub(crate) validator: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchReceivePlanRow {
    pub(crate) requested_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) pid_count: u64,
    pub(crate) expected_candidate_format: &'static str,
    pub(crate) expected_result_column_count: u64,
    pub(crate) validator_function: &'static str,
    pub(crate) row_locator_policy: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchMergeInputSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) remote_batch_count: u64,
    pub(crate) local_batch_count: u64,
    pub(crate) skipped_batch_count: u64,
    pub(crate) ready_batch_count: u64,
    pub(crate) blocked_batch_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) local_pid_count: u64,
    pub(crate) skipped_pid_count: u64,
    pub(crate) merge_function: &'static str,
    pub(crate) dedupe_key: &'static str,
    pub(crate) tie_breaker: &'static str,
    pub(crate) top_k: u64,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchMergeOrderContractRow {
    pub(crate) order_ordinal: u64,
    pub(crate) order_key: &'static str,
    pub(crate) direction: &'static str,
    pub(crate) semantic_role: &'static str,
    pub(crate) validator: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchRowLocatorContractRow {
    pub(crate) contract_item: &'static str,
    pub(crate) contract_value: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchVectorIdentityContractRow {
    pub(crate) contract_item: &'static str,
    pub(crate) contract_value: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchFinalizationSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) remote_batch_count: u64,
    pub(crate) local_batch_count: u64,
    pub(crate) skipped_batch_count: u64,
    pub(crate) merge_status: &'static str,
    pub(crate) row_locator_policy: &'static str,
    pub(crate) local_heap_resolution: &'static str,
    pub(crate) remote_heap_resolution: &'static str,
    pub(crate) final_heap_fetch_status: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchHeapResolutionContractRow {
    pub(crate) resolution_scope: &'static str,
    pub(crate) candidate_source: &'static str,
    pub(crate) heap_lookup_owner: &'static str,
    pub(crate) row_locator_policy: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchCoordinatorLocalSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) local_pid_count: u64,
    pub(crate) remote_target_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) skipped_placement_count: u64,
    pub(crate) candidate_input_count: u64,
    pub(crate) duplicate_vec_id_count: u64,
    pub(crate) returned_candidate_count: u64,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteSearchCoordinatorGateSummaryRow {
    pub(crate) requested_epoch: u64,
    pub(crate) local_plan_count: u64,
    pub(crate) remote_plan_count: u64,
    pub(crate) skipped_plan_count: u64,
    pub(crate) local_pid_count: u64,
    pub(crate) remote_pid_count: u64,
    pub(crate) skipped_pid_count: u64,
    pub(crate) execution_status: &'static str,
    pub(crate) libpq_dispatch_count: u64,
    pub(crate) libpq_dispatch_status: &'static str,
    pub(crate) libpq_executor_status: &'static str,
    pub(crate) libpq_executor_next_step: &'static str,
    pub(crate) libpq_receive_count: u64,
    pub(crate) libpq_receive_status: &'static str,
    pub(crate) merge_status: &'static str,
    pub(crate) final_heap_fetch_status: &'static str,
    pub(crate) next_blocker: &'static str,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexEpochCleanupRunResult {
    pub(crate) active_epoch: u64,
    pub(crate) cleanup_epoch_count: u64,
    pub(crate) protected_tuple_count: u64,
    pub(crate) removed_tuple_count: u64,
    pub(crate) removed_tuple_bytes: u64,
    pub(crate) physical_cleanup_status: &'static str,
    pub(crate) cleanup_message: &'static str,
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
pub(crate) struct SpireRemoteNodeSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) node_kind: &'static str,
    pub(crate) descriptor_generation: u64,
    pub(crate) descriptor_state: &'static str,
    pub(crate) placement_count: u64,
    pub(crate) available_placement_count: u64,
    pub(crate) stale_placement_count: u64,
    pub(crate) unavailable_placement_count: u64,
    pub(crate) skipped_placement_count: u64,
    pub(crate) local_store_count: u64,
    pub(crate) last_seen_at_micros: i64,
    pub(crate) last_served_epoch: u64,
    pub(crate) min_retained_epoch: u64,
    pub(crate) extension_version: String,
    pub(crate) last_error: String,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteNodeDescriptorContractRow {
    pub(crate) field_ordinal: u64,
    pub(crate) field_name: &'static str,
    pub(crate) pg_type: &'static str,
    pub(crate) semantic_role: &'static str,
    pub(crate) required: bool,
    pub(crate) validator: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteNodeDescriptorStateContractRow {
    pub(crate) state_ordinal: u64,
    pub(crate) descriptor_state: &'static str,
    pub(crate) state_source: &'static str,
    pub(crate) read_eligible: bool,
    pub(crate) snapshot_status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteNodeDescriptorRegistrationContractRow {
    pub(crate) step_ordinal: u64,
    pub(crate) step_name: &'static str,
    pub(crate) input_field: &'static str,
    pub(crate) semantic_role: &'static str,
    pub(crate) validator: &'static str,
    pub(crate) persistence_action: &'static str,
    pub(crate) failure_status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteNodeDescriptorReadinessRow {
    pub(crate) active_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) field_ordinal: u64,
    pub(crate) field_name: &'static str,
    pub(crate) semantic_role: &'static str,
    pub(crate) required: bool,
    pub(crate) validator: &'static str,
    pub(crate) descriptor_state: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteNodeDescriptorReadinessSummaryRow {
    pub(crate) active_epoch: u64,
    pub(crate) remote_node_count: u64,
    pub(crate) descriptor_field_count: u64,
    pub(crate) required_field_count: u64,
    pub(crate) ready_field_count: u64,
    pub(crate) blocked_field_count: u64,
    pub(crate) missing_required_field_count: u64,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteNodeCapabilityPlanRow {
    pub(crate) active_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) node_kind: &'static str,
    pub(crate) descriptor_generation: u64,
    pub(crate) descriptor_state: &'static str,
    pub(crate) required_last_served_epoch: u64,
    pub(crate) required_min_retained_epoch: u64,
    pub(crate) required_candidate_format: &'static str,
    pub(crate) required_extension_version: &'static str,
    pub(crate) conninfo_source: &'static str,
    pub(crate) remote_index_identity_status: &'static str,
    pub(crate) epoch_window_status: &'static str,
    pub(crate) candidate_format_status: &'static str,
    pub(crate) extension_version_status: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteNodeCapabilitySummaryRow {
    pub(crate) active_epoch: u64,
    pub(crate) node_count: u64,
    pub(crate) local_node_count: u64,
    pub(crate) remote_node_count: u64,
    pub(crate) ready_node_count: u64,
    pub(crate) blocked_node_count: u64,
    pub(crate) missing_descriptor_node_count: u64,
    pub(crate) required_candidate_format: &'static str,
    pub(crate) required_extension_version: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteEpochPublishReadinessRow {
    pub(crate) active_epoch: u64,
    pub(crate) remote_node_count: u64,
    pub(crate) remote_placement_count: u64,
    pub(crate) remote_available_placement_count: u64,
    pub(crate) remote_unavailable_placement_count: u64,
    pub(crate) remote_skipped_placement_count: u64,
    pub(crate) ready_remote_node_count: u64,
    pub(crate) blocked_remote_node_count: u64,
    pub(crate) missing_descriptor_node_count: u64,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteEpochPublishGateSummaryRow {
    pub(crate) active_epoch: u64,
    pub(crate) publish_scope: &'static str,
    pub(crate) publish_decision: &'static str,
    pub(crate) remote_node_count: u64,
    pub(crate) remote_placement_count: u64,
    pub(crate) ready_remote_node_count: u64,
    pub(crate) blocked_remote_node_count: u64,
    pub(crate) missing_descriptor_node_count: u64,
    pub(crate) policy_contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) next_blocker: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteEpochPublishPlanRow {
    pub(crate) active_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) descriptor_state: &'static str,
    pub(crate) placement_count: u64,
    pub(crate) available_placement_count: u64,
    pub(crate) stale_placement_count: u64,
    pub(crate) unavailable_placement_count: u64,
    pub(crate) skipped_placement_count: u64,
    pub(crate) required_last_served_epoch: u64,
    pub(crate) required_min_retained_epoch: u64,
    pub(crate) last_served_epoch: u64,
    pub(crate) min_retained_epoch: u64,
    pub(crate) epoch_window_status: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteEpochManifestPlanRow {
    pub(crate) active_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) descriptor_state: &'static str,
    pub(crate) placement_count: u64,
    pub(crate) required_last_served_epoch: u64,
    pub(crate) required_min_retained_epoch: u64,
    pub(crate) last_served_epoch: u64,
    pub(crate) min_retained_epoch: u64,
    pub(crate) epoch_window_status: &'static str,
    pub(crate) manifest_action: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteEpochManifestSummaryRow {
    pub(crate) active_epoch: u64,
    pub(crate) manifest_scope: &'static str,
    pub(crate) manifest_decision: &'static str,
    pub(crate) manifest_entry_count: u64,
    pub(crate) included_remote_node_count: u64,
    pub(crate) blocked_remote_node_count: u64,
    pub(crate) remote_placement_count: u64,
    pub(crate) publish_decision: &'static str,
    pub(crate) next_blocker: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteDegradationPolicyContractRow {
    pub(crate) consistency_mode: &'static str,
    pub(crate) placement_state: &'static str,
    pub(crate) search_action: &'static str,
    pub(crate) publish_action: &'static str,
    pub(crate) status: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteEpochManifestPublicationContractRow {
    pub(crate) step_ordinal: u64,
    pub(crate) prerequisite: &'static str,
    pub(crate) publication_action: &'static str,
    pub(crate) required_status: &'static str,
    pub(crate) validator: &'static str,
    pub(crate) failure_status: &'static str,
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
    pub(crate) route_count: u64,
    pub(crate) leaf_route_count: u64,
    pub(crate) delta_route_count: u64,
    pub(crate) prefetched_object_count: u64,
    pub(crate) scanned_pid_count: u64,
    pub(crate) leaf_pid_count: u64,
    pub(crate) delta_pid_count: u64,
    pub(crate) candidate_row_count: u64,
    pub(crate) leaf_candidate_row_count: u64,
    pub(crate) delta_candidate_row_count: u64,
    pub(crate) primary_candidate_row_count: u64,
    pub(crate) boundary_replica_candidate_row_count: u64,
    pub(crate) deduped_candidate_row_count: u64,
    pub(crate) deduped_primary_candidate_row_count: u64,
    pub(crate) deduped_boundary_replica_candidate_row_count: u64,
    pub(crate) truncated_candidate_row_count: u64,
    pub(crate) truncated_primary_candidate_row_count: u64,
    pub(crate) truncated_boundary_replica_candidate_row_count: u64,
    pub(crate) candidate_winner_count: u64,
    pub(crate) primary_candidate_winner_count: u64,
    pub(crate) boundary_replica_candidate_winner_count: u64,
    pub(crate) delete_delta_row_count: u64,
    pub(crate) dropped_unselected_delta_route_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexScanRoutingSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) effective_nprobe: u32,
    pub(crate) effective_nprobe_source: &'static str,
    pub(crate) recursive_beam_width: u64,
    pub(crate) max_leaf_routes: u64,
    pub(crate) max_routing_expansions: u64,
    pub(crate) routing_level: u16,
    pub(crate) input_frontier_width: u64,
    pub(crate) expanded_parent_count: u64,
    pub(crate) selected_child_count: u64,
    pub(crate) deduped_route_count: u64,
    pub(crate) truncation_reason: &'static str,
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
