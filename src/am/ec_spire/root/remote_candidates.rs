fn remote_candidate_assignment_role_rank(candidate: &SpireRemoteSearchCandidateRow) -> u8 {
    u8::from(candidate.assignment_flags & storage::SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0)
}

fn remote_search_candidate_cmp(
    left: &SpireRemoteSearchCandidateRow,
    right: &SpireRemoteSearchCandidateRow,
) -> std::cmp::Ordering {
    left.score
        .total_cmp(&right.score)
        .then_with(|| {
            remote_candidate_assignment_role_rank(left)
                .cmp(&remote_candidate_assignment_role_rank(right))
        })
        .then_with(|| right.served_epoch.cmp(&left.served_epoch))
        .then_with(|| left.node_id.cmp(&right.node_id))
        .then_with(|| left.pid.cmp(&right.pid))
        .then_with(|| right.object_version.cmp(&left.object_version))
        .then_with(|| left.row_index.cmp(&right.row_index))
        .then_with(|| left.row_locator.cmp(&right.row_locator))
}

const SPIRE_REMOTE_TARGET_LOCAL: &str = "local";
const SPIRE_REMOTE_TARGET_REMOTE: &str = "remote";
const SPIRE_REMOTE_TARGET_SKIPPED: &str = "skipped";
const SPIRE_REMOTE_STATUS_READY: &str = "ready";
const SPIRE_REMOTE_STATUS_EMPTY_TOP_K: &str = "empty_top_k";
const SPIRE_REMOTE_STATUS_DEGRADED_READY: &str = "degraded_ready";
const SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED: &str = "degraded_skipped";
const SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR: &str = "requires_remote_node_descriptor";
const SPIRE_REMOTE_STATUS_REQUIRES_SECRET: &str = "requires_conninfo_secret_resolution";
const SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ: &str = "requires_libpq_transport";
const SPIRE_REMOTE_STATUS_MISSING_DESCRIPTOR: &str = "missing_descriptor";
const SPIRE_REMOTE_STATUS_OPTIONAL_DESCRIPTOR_MISSING: &str = "optional_descriptor_missing";
const SPIRE_REMOTE_STATUS_STALE_EPOCH: &str = "stale_epoch";
const SPIRE_REMOTE_STATUS_RETENTION_GAP: &str = "retention_gap";
const SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION: &str =
    "incompatible_extension_version";
const SPIRE_REMOTE_STATUS_CONSISTENCY_MODE_MISMATCH: &str = "consistency_mode_mismatch";
const SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH: &str = "endpoint_identity_mismatch";
const SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD: &str = "remote_executor_overload";
const SPIRE_REMOTE_STATUS_REQUIRES_FINGERPRINT_BINDING: &str = "requires_fingerprint_binding";
const SPIRE_REMOTE_STATUS_REQUIRES_OPCLASS_BINDING: &str = "requires_opclass_binding";
const SPIRE_REMOTE_STATUS_REQUIRES_SCORING_OPTION_BINDING: &str =
    "requires_scoring_option_binding";
const SPIRE_REMOTE_STATUS_REQUIRES_RABITQ_STORAGE_FORMAT: &str =
    "requires_rabitq_storage_format";
const SPIRE_REMOTE_TRANSPORT_LOCAL_DIRECT: &str = "local_direct";
const SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE: &str = "libpq_pipeline";
const SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION: &str = "open_pipeline_and_send_remote_search";
const SPIRE_REMOTE_DISPATCH_BLOCKED_ACTION: &str = "blocked_before_dispatch";
const SPIRE_REMOTE_NONE: &str = "none";
const SPIRE_REMOTE_EXECUTOR_REQUIRED: &str = "requires_libpq_executor";
const SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR: &str = "remote_node_descriptor";
const SPIRE_REMOTE_EXECUTOR_STEP_EPOCH_WINDOW: &str = "remote_epoch_window";
const SPIRE_REMOTE_EXECUTOR_STEP_EXTENSION_VERSION: &str = "remote_extension_version";
const SPIRE_REMOTE_EXECUTOR_STEP_BUDGET: &str = "remote_executor_budget";
const SPIRE_REMOTE_EXECUTOR_STEP_GOVERNANCE: &str = "remote_executor_governance";
const SPIRE_REMOTE_EXECUTOR_STEP_SECRET: &str = "conninfo_secret_resolution";
const SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT: &str = "production_transport_adapter";
const SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE: &str = "compact_candidate_receive";
const SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION: &str = "remote_heap_resolution";
const SPIRE_REMOTE_EXECUTOR_STEP_CUSTOM_SCAN_TUPLE_DELIVERY: &str =
    "custom_scan_tuple_delivery";
const SPIRE_REMOTE_EXECUTOR_STEP_CANCELLATION: &str = "remote_executor_cancellation";
const SPIRE_REMOTE_EXECUTOR_STEP_CONSISTENCY_POLICY: &str = "remote_consistency_policy";
const SPIRE_REMOTE_ENDPOINT_SEARCH: &str = "ec_spire_remote_search";
const SPIRE_REMOTE_INDEX_SOURCE_LOCAL_OID: &str = "local_index_oid";
const SPIRE_REMOTE_DESCRIPTOR_SOURCE: &str = "remote_node_descriptor";
const SPIRE_REMOTE_CONNINFO_READY: &str = "secret_reference_ready";
const SPIRE_REMOTE_CONNINFO_RESOLVED: &str = "resolved_conninfo";
const SPIRE_REMOTE_PRODUCTION_STATE_MODEL: &str = "spire_remote_fanout_executor_v1";
const SPIRE_REMOTE_PRODUCTION_TRANSPORT_PENDING: &str = "async_or_pipeline_transport_pending";
const SPIRE_REMOTE_STATUS_REQUIRES_PRODUCTION_TRANSPORT: &str =
    "requires_production_transport_adapter";
const SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED: &str = "remote_transport_failed";
const SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNINFO_PARSE_FAILED: &str = "conninfo_parse_failed";
const SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED: &str = "connect_failed";
const SPIRE_REMOTE_PRODUCTION_TRANSPORT_STATEMENT_TIMEOUT_SETUP_FAILED: &str =
    "statement_timeout_setup_failed";
const SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED: &str = "remote_query_failed";
const SPIRE_REMOTE_PRODUCTION_REMOTE_STATEMENT_TIMEOUT: &str = "remote_statement_timeout";
const SPIRE_REMOTE_PRODUCTION_REMOTE_QUERY_CANCELLED: &str = "remote_query_cancelled";
const SPIRE_REMOTE_PRODUCTION_REMOTE_BACKEND_TERMINATED: &str = "remote_backend_terminated";
const SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE: &str = "remote_index_unavailable";
const SPIRE_REMOTE_PRODUCTION_CANDIDATE_DECODE_FAILED: &str = "candidate_decode_failed";
const SPIRE_REMOTE_PRODUCTION_CANDIDATE_VALIDATION_FAILED: &str =
    "candidate_batch_validation_failed";
const SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH: &str = "served_epoch_mismatch";
const SPIRE_REMOTE_PRODUCTION_REQUESTED_EPOCH_MISMATCH: &str = "requested_epoch_mismatch";
const SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS: &str = "candidate_invalid_parameters";
const SPIRE_REMOTE_PRODUCTION_PROTOCOL_VERSION_MISMATCH: &str = "protocol_version_mismatch";
const SPIRE_REMOTE_PRODUCTION_EXTENSION_VERSION_MISMATCH: &str = "extension_version_mismatch";
const SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED: &str =
    "remote_heap_resolution_failed";
const SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_MISSING: &str = "remote_heap_row_missing";
const SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_DEAD: &str = "remote_heap_row_dead";
const SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_STALE: &str = "remote_heap_row_stale";
const SPIRE_REMOTE_STATUS_REQUIRES_COMPACT_CANDIDATE_RECEIVE: &str =
    "requires_compact_candidate_receive";
const SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED: &str = "remote_candidate_receive_failed";
const SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED: &str = "remote_executor_cancelled";
const SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED: &str = "local_query_cancelled";
const SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT: &str = "local_statement_timeout";
const SPIRE_REMOTE_CANDIDATE_FORMAT_LOCAL: &str = "local";
const SPIRE_REMOTE_CANDIDATE_FORMAT_V1: &str = "ec_spire_remote_search_v1";
const SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1: &str = "pg_binary_attr_v1";
const SPIRE_REMOTE_ROW_LOCATOR_POLICY: &str = "opaque_origin_node_bytes";
const SPIRE_REMOTE_VEC_ID_DEDUPE_KEY: &str = "global_vec_id_or_node_scoped_local_vec_id";
const SPIRE_REMOTE_VEC_ID_KEY_GLOBAL: u8 = 0xA0;
const SPIRE_REMOTE_VEC_ID_KEY_NODE_LOCAL: u8 = 0xA1;
const SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION: &str = "coordinator_local_heap";
const SPIRE_REMOTE_HEAP_RESOLUTION: &str = "origin_node_row_locator";
const SPIRE_REMOTE_FINAL_STATUS_LOCAL_READY: &str = "local_ready";
const SPIRE_REMOTE_FINAL_STATUS_REMOTE_READY: &str = "remote_ready";
const SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES: &str = "no_candidate_batches";
const SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP: &str = "requires_remote_heap_resolution";
const SPIRE_REMOTE_FINAL_STATUS_REQUIRES_CUSTOM_SCAN_TUPLE_DELIVERY: &str =
    "requires_custom_scan_tuple_delivery";
const SPIRE_REMOTE_FINAL_STATUS_BLOCKED: &str = "blocked";
const SPIRE_REMOTE_FINAL_STATUS_PLANNED: &str = "planned";
const SPIRE_REMOTE_RESULT_SOURCE_LOCAL_HEAP_CANDIDATES: &str = "local_heap_candidates";
const SPIRE_REMOTE_RESULT_SOURCE_REMOTE_HEAP_CANDIDATES: &str = "remote_heap_candidates";
const SPIRE_REMOTE_RESULT_SOURCE_BLOCKED: &str = "blocked";
const SPIRE_REMOTE_DESCRIPTOR_STATE_ACTIVE: &str = "active";
const SPIRE_REMOTE_DESCRIPTOR_STATE_DRAINING: &str = "draining";
const SPIRE_REMOTE_DESCRIPTOR_STATE_DISABLED: &str = "disabled";
const SPIRE_REMOTE_DESCRIPTOR_STATE_FAILED: &str = "failed";
const SPIRE_REMOTE_DESCRIPTOR_STATE_MISSING: &str = "missing";
const SPIRE_REMOTE_CONNINFO_ENV_PREFIX: &str = "EC_SPIRE_REMOTE_CONNINFO_";
const SPIRE_COORDINATOR_INSERT_DISPATCH_TRANSPORT_LIBPQ: &str = "libpq";
const SPIRE_COORDINATOR_INSERT_TRANSACTION_PROTOCOL_2PC: &str =
    "remote_prepare_local_placement_commit_remote_prepared";
const SPIRE_COORDINATOR_INSERT_DISPATCH_ACTION_PREPARE: &str =
    "open_remote_transaction_send_insert_prepare_xact";
const SPIRE_COORDINATOR_INSERT_DISPATCH_ACTION_BLOCKED: &str = "blocked";
const SPIRE_COORDINATOR_INSERT_NEXT_STEP_PREPARE: &str = "remote_insert_prepare_transaction";
const SPIRE_COORDINATOR_INSERT_NEXT_STEP_LOCAL_PLACEMENT: &str =
    "local_placement_directory_write";
const SPIRE_COORDINATOR_INSERT_PREPARED_STATUS: &str = "remote_insert_prepared";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireRemoteSearchLibpqExecutorBudgetLimits {
    max_nodes: u64,
    max_pids: u64,
    max_pids_per_node: u64,
    max_concurrent_dispatches: u64,
    max_concurrent_dispatches_per_node: u64,
    connect_timeout_ms: u64,
    statement_timeout_ms: u64,
}

impl SpireRemoteSearchLibpqExecutorBudgetLimits {
    fn from_session() -> Self {
        Self {
            max_nodes: session_limit_to_u64(options::current_session_remote_search_max_nodes()),
            max_pids: session_limit_to_u64(options::current_session_remote_search_max_pids()),
            max_pids_per_node: session_limit_to_u64(
                options::current_session_remote_search_max_pids_per_node(),
            ),
            max_concurrent_dispatches: session_limit_to_u64(
                options::current_session_remote_search_max_concurrent_dispatches(),
            ),
            max_concurrent_dispatches_per_node: session_limit_to_u64(
                options::current_session_remote_search_max_concurrent_dispatches_per_node(),
            ),
            connect_timeout_ms: session_limit_to_u64(
                options::current_session_remote_search_connect_timeout_ms(),
            ),
            statement_timeout_ms: session_limit_to_u64(
                options::current_session_remote_search_statement_timeout_ms(),
            ),
        }
    }

    fn has_node_cap(self) -> bool {
        self.max_nodes > 0
    }

    fn has_pid_cap(self) -> bool {
        self.max_pids > 0
    }

    fn has_pid_per_node_cap(self) -> bool {
        self.max_pids_per_node > 0
    }

    fn has_concurrent_dispatch_cap(self) -> bool {
        self.max_concurrent_dispatches > 0
    }

    fn has_concurrent_dispatch_per_node_cap(self) -> bool {
        self.max_concurrent_dispatches_per_node > 0
    }
}

fn session_limit_to_u64(value: i32) -> u64 {
    u64::try_from(value.max(0)).expect("non-negative session limit should fit in u64")
}

fn remote_search_pre_dispatch_blocker_step(status: &str) -> &'static str {
    match status {
        SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR => SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR,
        SPIRE_REMOTE_STATUS_STALE_EPOCH | SPIRE_REMOTE_STATUS_RETENTION_GAP => {
            SPIRE_REMOTE_EXECUTOR_STEP_EPOCH_WINDOW
        }
        SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION => {
            SPIRE_REMOTE_EXECUTOR_STEP_EXTENSION_VERSION
        }
        SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD => SPIRE_REMOTE_EXECUTOR_STEP_BUDGET,
        _ => SPIRE_REMOTE_NONE,
    }
}

fn remote_search_pre_dispatch_blocker_recommendation(status: &str) -> &'static str {
    match status {
        SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR => {
            "register active or draining remote node descriptors before libpq executor startup"
        }
        SPIRE_REMOTE_STATUS_STALE_EPOCH | SPIRE_REMOTE_STATUS_RETENTION_GAP => {
            "refresh remote node served epoch window before libpq fanout execution"
        }
        SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION => {
            "upgrade remote node extension before libpq fanout execution"
        }
        SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD => {
            "raise ec_spire remote-search executor budgets or reduce remote fanout before libpq dispatch"
        }
        _ => SPIRE_REMOTE_NONE,
    }
}

pub(crate) fn remote_operator_entrypoint_contract_rows(
) -> Vec<SpireRemoteOperatorEntrypointContractRow> {
    vec![
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 1,
            entrypoint_name: "ec_spire_remote_search_coordinator_gate_summary",
            area: "search",
            operator_use: "pre_result_gate",
            status_source: "status,next_blocker,libpq_executor_next_step",
            next_action: "resolve_reported_blocker_before_expect_result_rows",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 2,
            entrypoint_name: "ec_spire_remote_search_coordinator_result_summary",
            area: "search",
            operator_use: "final_result_source",
            status_source: "result_source,status,next_blocker",
            next_action: "consume_local_heap_candidates_or_resolve_blocker",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 3,
            entrypoint_name: "ec_spire_remote_node_snapshot",
            area: "descriptor",
            operator_use: "node_inventory",
            status_source: "descriptor_state,status,recommendation",
            next_action: "register_or_refresh_remote_node_descriptor",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 4,
            entrypoint_name: "ec_spire_remote_node_descriptor_readiness_summary",
            area: "descriptor",
            operator_use: "descriptor_field_gate",
            status_source: "ready_field_count,blocked_field_count,status",
            next_action: "fill_required_descriptor_fields_before_transport",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 5,
            entrypoint_name: "ec_spire_remote_node_capability_summary",
            area: "capability",
            operator_use: "remote_serving_window_gate",
            status_source: "ready_node_count,blocked_node_count,status",
            next_action: "refresh_served_epoch_or_remote_index_identity",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 6,
            entrypoint_name: "ec_spire_remote_epoch_publish_gate_summary",
            area: "manifest",
            operator_use: "distributed_publish_gate",
            status_source: "publish_decision,status,next_blocker",
            next_action: "persist_manifest_only_after_publish_gate_ready",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 7,
            entrypoint_name: "ec_spire_remote_epoch_manifest_catalog_summary",
            area: "manifest",
            operator_use: "manifest_persistence_gate",
            status_source: "catalog_status,persisted_manifest_count,persisted_entry_mismatch_count",
            next_action: "persist_or_refresh_remote_epoch_manifest_catalog",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 8,
            entrypoint_name: "ec_spire_remote_epoch_manifest_publication_result_summary",
            area: "publication",
            operator_use: "manifest_publication_result",
            status_source: "result_source,status,next_blocker",
            next_action: "run_libpq_executor_or_resolve_publication_blocker",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 9,
            entrypoint_name: "ec_spire_remote_search_libpq_secret_summary",
            area: "search",
            operator_use: "search_conninfo_secret_gate",
            status_source: "status,next_executor_step,blocked_secret_count",
            next_action: "resolve_missing_conninfo_secrets_before_opening_libpq_connections",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 10,
            entrypoint_name: "ec_spire_remote_conninfo_secret_resolution_status",
            area: "secret",
            operator_use: "single_conninfo_secret_probe",
            status_source: "status,provider_lookup_key,resolved_conninfo_bytes",
            next_action: "set_executor_owned_secret_provider_value_without_exposing_raw_conninfo",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 11,
            entrypoint_name: "ec_spire_remote_search_production_executor_state_summary",
            area: "search",
            operator_use: "production_executor_dry_state",
            status_source: "state_model,dispatch_count,next_executor_step,status",
            next_action: "use_dry_state_for_planning_before_async_pipeline_transport_lands",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 12,
            entrypoint_name: "ec_spire_remote_pipeline_steps",
            area: "search",
            operator_use: "consolidated_remote_pipeline_steps_dry",
            status_source: "step_name,status,item_count,next_blocker",
            next_action: "inspect_first_non_ready_step_before_live_probe_or_narrow_surfaces",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 13,
            entrypoint_name: "ec_spire_remote_pipeline_steps_live",
            area: "search",
            operator_use: "consolidated_remote_pipeline_steps_live_probe",
            status_source: "step_name,status,item_count,next_blocker",
            next_action: "run_only_when_connection_and_remote_executor_probe_cost_is_expected",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 14,
            entrypoint_name: "ec_spire_remote_search_vector_identity_contract",
            area: "search",
            operator_use: "remote_dedupe_identity_contract",
            status_source: "contract_item,contract_value,status",
            next_action: "require_global_vec_ids_before_cross_node_replica_dedupe",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 15,
            entrypoint_name: "ec_spire_remote_search_endpoint_contract",
            area: "search",
            operator_use: "remote_endpoint_contract_gate",
            status_source: "contract_item,contract_value,status,validator",
            next_action: "resolve_non_ready_endpoint_contract_rows_before_production_remote_merge",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 16,
            entrypoint_name: "ec_spire_remote_search_endpoint_identity",
            area: "search",
            operator_use: "remote_endpoint_identity_gate",
            status_source: "protocol_version,opclass_identity,profile_fingerprint,status",
            next_action: "require_ready_endpoint_identity_before_accepting_remote_candidate_scores",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 17,
            entrypoint_name: "ec_spire_remote_search_libpq_executor_receive_attempts",
            area: "search",
            operator_use: "per_node_remote_receive_attempt_diagnostics",
            status_source: "node_id,status,next_blocker,failure_action,failure_reason",
            next_action: "use_strict_fail_closed_or_degraded_skip_reason_before_merge",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 18,
            entrypoint_name: "ec_spire_remote_search_libpq_executor_budget_summary",
            area: "search",
            operator_use: "remote_executor_resource_governance",
            status_source: "status,next_executor_step,max_nodes,max_pids,max_pids_per_node",
            next_action: "tune_remote_executor_budgets_or_reduce_fanout_before_dispatch",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 19,
            entrypoint_name: "ec_spire_remote_search_production_scan_handoff_summary",
            area: "search",
            operator_use: "production_am_scan_candidate_handoff",
            status_source: "selected_pid_count,candidate_row_count,status,next_blocker",
            next_action: "resolve remote heap rows before allowing remote SQL tuple return",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 20,
            entrypoint_name: "ec_spire_remote_search_stage_e_fault_matrix",
            area: "search",
            operator_use: "local_multi_instance_fault_fixture_contract",
            status_source: "fault_case,failure_category,strict_action,degraded_action,counter_delta",
            next_action: "implement_stage_e_fault_fixture_against_each_named_case",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 21,
            entrypoint_name: "ec_spire_remote_search_operator_diagnostics",
            area: "search",
            operator_use: "packet_friendly_production_readiness_rollup",
            status_source: "remote_readiness_status,candidate_batch_count,final_heap_fetch_status,am_delivery_status,next_blocker",
            next_action: "inspect_single_rollup_before_running_multi_instance_fault_fixture",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 22,
            entrypoint_name: "ec_spire_remote_search_stage_e_lifecycle_matrix",
            area: "search",
            operator_use: "local_multi_instance_lifecycle_fixture_contract",
            status_source: "lifecycle_case,strict_action,degraded_action,required_detection,next_executor_step",
            next_action: "implement_drop_reindex_create_concurrently_fixture_against_each_named_case",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 23,
            entrypoint_name: "ec_spire_remote_epoch_manifest_freshness",
            area: "manifest",
            operator_use: "stage_e_manifest_freshness_assertion",
            status_source: "node_id,freshness_status,persisted_entry_matches,next_action",
            next_action: "persist_or_refresh_manifest_before_stage_e_fixture_execution",
        },
    ]
}

pub(crate) fn remote_libpq_connection_lifecycle_contract_rows(
) -> Vec<SpireRemoteLibpqConnectionLifecycleContractRow> {
    vec![
        SpireRemoteLibpqConnectionLifecycleContractRow {
            surface: "ec_spire_remote_search_libpq_executor",
            connection_lifecycle_policy: "per_query",
            pooling_policy: "no_pooling_v1",
            secret_resolution_policy: "conninfo_secret_name_resolved_by_executor",
            conninfo_exposure_policy: "never_expose_raw_conninfo_in_sql",
            failure_policy: "fail_closed_no_implicit_retry",
            resource_limit_policy: "bounded_by_ec_spire_remote_search_session_caps",
            validator: "must_close_connection_before_coordinator_returns",
            recommendation: "enforce remote executor budgets before secret lookup or socket open",
        },
        SpireRemoteLibpqConnectionLifecycleContractRow {
            surface: "ec_spire_remote_epoch_manifest_publication_libpq_executor",
            connection_lifecycle_policy: "per_query",
            pooling_policy: "no_pooling_v1",
            secret_resolution_policy: "conninfo_secret_name_resolved_by_executor",
            conninfo_exposure_policy: "never_expose_raw_conninfo_in_sql",
            failure_policy: "fail_closed_no_implicit_retry",
            resource_limit_policy: "one_connection_per_ready_remote_node_per_publication",
            validator: "must_close_connection_before_publication_result_returns",
            recommendation: "share executor secret provider with remote search publication transport",
        },
    ]
}

pub(crate) fn remote_conninfo_secret_resolution_contract_rows(
) -> Vec<SpireRemoteConninfoSecretResolutionContractRow> {
    vec![
        SpireRemoteConninfoSecretResolutionContractRow {
            provider_ordinal: 1,
            provider_policy: "external_executor_secret_provider",
            provider_status: "selected_v1",
            secret_reference_field: "conninfo_secret_name",
            sql_storage_policy: "descriptor_catalog_stores_secret_reference_only",
            raw_conninfo_allowed: false,
            executor_action: "resolve_conninfo_secret_reference",
            failure_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_resolve_to_libpq_conninfo_without_sql_exposure",
            recommendation: "configure the executor secret provider before enabling remote transport",
        },
        SpireRemoteConninfoSecretResolutionContractRow {
            provider_ordinal: 2,
            provider_policy: "postgres_fdw_user_mapping",
            provider_status: "not_selected_v1",
            secret_reference_field: "conninfo_secret_name",
            sql_storage_policy: "no_postgres_fdw_dependency_for_v1",
            raw_conninfo_allowed: false,
            executor_action: "not_used",
            failure_status: "unsupported_secret_provider",
            validator: "must_not_assume_fdw_user_mapping_exists",
            recommendation: "keep FDW-style mapping as a future integration option",
        },
        SpireRemoteConninfoSecretResolutionContractRow {
            provider_ordinal: 3,
            provider_policy: "in_extension_conninfo_table",
            provider_status: "rejected_v1",
            secret_reference_field: "conninfo_secret_name",
            sql_storage_policy: "never_store_raw_conninfo_in_extension_catalog",
            raw_conninfo_allowed: false,
            executor_action: "not_used",
            failure_status: "unsupported_secret_provider",
            validator: "must_not_persist_raw_conninfo_in_ec_spire_catalogs",
            recommendation: "use external secret storage instead of an extension-owned conninfo table",
        },
    ]
}

pub(crate) fn remote_conninfo_secret_provider_lookup_key(
    conninfo_secret_name: &str,
) -> Result<String, String> {
    if conninfo_secret_name.is_empty() {
        return Err("conninfo_secret_name must be nonempty".to_owned());
    }

    let mut key = String::from(SPIRE_REMOTE_CONNINFO_ENV_PREFIX);
    for byte in conninfo_secret_name.bytes() {
        if byte.is_ascii_alphanumeric() {
            key.push(char::from(byte).to_ascii_uppercase());
        } else {
            key.push('_');
        }
    }
    Ok(key)
}

pub(crate) fn remote_conninfo_secret_resolution_status_row(
    conninfo_secret_name: &str,
) -> SpireRemoteConninfoSecretResolutionStatusRow {
    let provider_lookup_key = remote_conninfo_secret_provider_lookup_key(conninfo_secret_name)
        .unwrap_or_else(|e| pgrx::error!("ec_spire remote conninfo secret reference invalid: {e}"));

    match std::env::var(&provider_lookup_key) {
        Ok(conninfo) if !conninfo.is_empty() => SpireRemoteConninfoSecretResolutionStatusRow {
            provider_policy: "external_executor_secret_provider",
            conninfo_secret_name: conninfo_secret_name.to_owned(),
            provider_lookup_key,
            resolved_conninfo_bytes: u64::try_from(conninfo.len())
                .expect("conninfo byte length should fit in u64"),
            raw_conninfo_exposed: false,
            status: SPIRE_REMOTE_CONNINFO_RESOLVED,
            recommendation: "open libpq connection with executor-owned resolved conninfo",
        },
        Ok(_) => SpireRemoteConninfoSecretResolutionStatusRow {
            provider_policy: "external_executor_secret_provider",
            conninfo_secret_name: conninfo_secret_name.to_owned(),
            provider_lookup_key,
            resolved_conninfo_bytes: 0,
            raw_conninfo_exposed: false,
            status: SPIRE_REMOTE_STATUS_REQUIRES_SECRET,
            recommendation: "configure a nonempty conninfo value in the external secret provider",
        },
        Err(_) => SpireRemoteConninfoSecretResolutionStatusRow {
            provider_policy: "external_executor_secret_provider",
            conninfo_secret_name: conninfo_secret_name.to_owned(),
            provider_lookup_key,
            resolved_conninfo_bytes: 0,
            raw_conninfo_exposed: false,
            status: SPIRE_REMOTE_STATUS_REQUIRES_SECRET,
            recommendation: "configure the external secret provider entry for conninfo_secret_name",
        },
    }
}

pub(crate) fn remote_catalog_lifecycle_contract_rows(
) -> Vec<SpireRemoteCatalogLifecycleContractRow> {
    vec![
        SpireRemoteCatalogLifecycleContractRow {
            lifecycle_ordinal: 1,
            lifecycle_event: "pg_dump_restore",
            oid_stability: "oids_reassigned",
            catalog_risk: "catalog_rows_reference_dump_source_oids",
            operator_action: "run_orphan_cleanup_and_reregister_descriptors",
            cleanup_surface: "ec_spire_remote_catalog_orphan_summary,ec_spire_remote_catalog_orphan_cleanup",
            migration_surface: "bootstrap_remote_catalog_tables",
            status: "requires_operator_reregistration",
            recommendation: "after logical restore, clean orphaned remote catalog rows and re-register remote node descriptors for restored coordinator indexes",
        },
        SpireRemoteCatalogLifecycleContractRow {
            lifecycle_ordinal: 2,
            lifecycle_event: "drop_index",
            oid_stability: "coordinator_oid_removed",
            catalog_risk: "remote_catalog_orphans",
            operator_action: "event_trigger_runs_remote_catalog_index_cleanup",
            cleanup_surface: "ec_spire_remote_catalog_index_cleanup,ec_spire_remote_catalog_orphan_cleanup",
            migration_surface: "ec_spire_remote_catalog_drop_index_cleanup",
            status: "automatic_event_trigger_cleanup",
            recommendation: "DROP INDEX automatically removes matching remote catalog rows; use orphan cleanup for restore-era sweeps",
        },
        SpireRemoteCatalogLifecycleContractRow {
            lifecycle_ordinal: 3,
            lifecycle_event: "basebackup_wal_replay",
            oid_stability: "oids_stable",
            catalog_risk: "catalog_rows_replay_with_database",
            operator_action: "no_reregistration_required",
            cleanup_surface: "none",
            migration_surface: "physical_backup",
            status: "supported",
            recommendation: "physical backups preserve coordinator index OIDs and remote catalog rows through WAL replay",
        },
        SpireRemoteCatalogLifecycleContractRow {
            lifecycle_ordinal: 4,
            lifecycle_event: "extension_upgrade_0_1_0_to_0_1_1",
            oid_stability: "oids_stable",
            catalog_risk: "remote_catalog_tables_absent_before_upgrade",
            operator_action: "apply_extension_upgrade_before_remote_transport",
            cleanup_surface: "none",
            migration_surface: "ecaz--0.1.0--0.1.1.sql",
            status: "supported_after_upgrade_script",
            recommendation: "extension upgrade creates remote catalog tables before remote transport is enabled",
        },
    ]
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct SpireRemoteCountRollup {
    local_count: u64,
    remote_count: u64,
    skipped_count: u64,
    ready_count: u64,
    blocked_count: u64,
    degraded_skipped_count: u64,
    missing_descriptor_count: u64,
    transport_count: u64,
    local_pid_count: u64,
    remote_pid_count: u64,
    skipped_pid_count: u64,
    ready_pid_count: u64,
    blocked_pid_count: u64,
    missing_descriptor_pid_count: u64,
    transport_pid_count: u64,
    first_remote_blocked_status: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpireRemoteSummaryStatusMode {
    RequestPlan,
    Readiness,
    Execution,
    LibpqRequest,
}

fn add_remote_count(value: &mut u64, amount: u64, context: &str, field: &str) -> Result<(), String> {
    *value = value
        .checked_add(amount)
        .ok_or_else(|| format!("ec_spire {context} {field} count overflowed"))?;
    Ok(())
}

impl SpireRemoteCountRollup {
    fn record_target(
        &mut self,
        target_kind: &str,
        pid_count: u64,
        context: &str,
    ) -> Result<(), String> {
        match target_kind {
            SPIRE_REMOTE_TARGET_LOCAL => {
                add_remote_count(&mut self.local_count, 1, context, "local")?;
                add_remote_count(&mut self.local_pid_count, pid_count, context, "local PID")?;
            }
            SPIRE_REMOTE_TARGET_REMOTE => {
                add_remote_count(&mut self.remote_count, 1, context, "remote")?;
                add_remote_count(&mut self.remote_pid_count, pid_count, context, "remote PID")?;
            }
            SPIRE_REMOTE_TARGET_SKIPPED => {
                add_remote_count(&mut self.skipped_count, 1, context, "skipped")?;
                add_remote_count(&mut self.skipped_pid_count, pid_count, context, "skipped PID")?;
            }
            target_kind => {
                return Err(format!(
                    "ec_spire {context} found unknown target_kind '{target_kind}'"
                ));
            }
        }
        Ok(())
    }

    fn record_remote_target(&mut self, pid_count: u64, context: &str) -> Result<(), String> {
        add_remote_count(&mut self.remote_count, 1, context, "remote")?;
        add_remote_count(&mut self.remote_pid_count, pid_count, context, "remote PID")
    }

    fn record_status(
        &mut self,
        status: &str,
        pid_count: u64,
        context: &str,
    ) -> Result<(), String> {
        match status {
            SPIRE_REMOTE_STATUS_READY => {
                add_remote_count(&mut self.ready_count, 1, context, "ready")?;
                add_remote_count(&mut self.ready_pid_count, pid_count, context, "ready PID")?;
            }
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED => {
                add_remote_count(&mut self.degraded_skipped_count, 1, context, "degraded skipped")?;
            }
            SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR => {
                add_remote_count(&mut self.blocked_count, 1, context, "blocked")?;
                add_remote_count(
                    &mut self.missing_descriptor_count,
                    1,
                    context,
                    "missing descriptor",
                )?;
                add_remote_count(&mut self.blocked_pid_count, pid_count, context, "blocked PID")?;
                add_remote_count(
                    &mut self.missing_descriptor_pid_count,
                    pid_count,
                    context,
                    "missing descriptor PID",
                )?;
            }
            SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ => {
                add_remote_count(&mut self.blocked_count, 1, context, "blocked")?;
                add_remote_count(&mut self.transport_count, 1, context, "transport")?;
                add_remote_count(&mut self.blocked_pid_count, pid_count, context, "blocked PID")?;
                add_remote_count(&mut self.transport_pid_count, pid_count, context, "transport PID")?;
            }
            SPIRE_REMOTE_STATUS_STALE_EPOCH
            | SPIRE_REMOTE_STATUS_RETENTION_GAP
            | SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION
            | SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD => {
                add_remote_count(&mut self.blocked_count, 1, context, "blocked")?;
                add_remote_count(&mut self.blocked_pid_count, pid_count, context, "blocked PID")?;
                if self.first_remote_blocked_status.is_none() {
                    self.first_remote_blocked_status = Some(match status {
                        SPIRE_REMOTE_STATUS_STALE_EPOCH => SPIRE_REMOTE_STATUS_STALE_EPOCH,
                        SPIRE_REMOTE_STATUS_RETENTION_GAP => SPIRE_REMOTE_STATUS_RETENTION_GAP,
                        SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION => {
                            SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION
                        }
                        SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD => {
                            SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD
                        }
                        _ => unreachable!("remote blocker status match already narrowed"),
                    });
                }
            }
            status => {
                return Err(format!("ec_spire {context} found unknown status '{status}'"));
            }
        }
        Ok(())
    }

    fn executable_pid_count(&self, context: &str) -> Result<u64, String> {
        self.local_pid_count
            .checked_add(self.remote_pid_count)
            .ok_or_else(|| format!("ec_spire {context} executable PID count overflowed"))
    }

    fn summary_status(&self, top_k: u64, mode: SpireRemoteSummaryStatusMode) -> &'static str {
        if top_k == 0 {
            return SPIRE_REMOTE_STATUS_EMPTY_TOP_K;
        }

        match mode {
            SpireRemoteSummaryStatusMode::RequestPlan => {
                if self.remote_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
                } else if self.skipped_count > 0 {
                    SPIRE_REMOTE_STATUS_DEGRADED_READY
                } else {
                    SPIRE_REMOTE_STATUS_READY
                }
            }
            SpireRemoteSummaryStatusMode::Readiness => {
                if self.missing_descriptor_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                } else if let Some(status) = self.first_remote_blocked_status {
                    status
                } else if self.transport_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
                } else if self.skipped_count > 0 {
                    SPIRE_REMOTE_STATUS_DEGRADED_READY
                } else {
                    SPIRE_REMOTE_STATUS_READY
                }
            }
            SpireRemoteSummaryStatusMode::Execution => {
                if self.missing_descriptor_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                } else if let Some(status) = self.first_remote_blocked_status {
                    status
                } else if self.transport_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
                } else if self.degraded_skipped_count > 0 {
                    SPIRE_REMOTE_STATUS_DEGRADED_READY
                } else {
                    SPIRE_REMOTE_STATUS_READY
                }
            }
            SpireRemoteSummaryStatusMode::LibpqRequest => {
                if self.missing_descriptor_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                } else if let Some(status) = self.first_remote_blocked_status {
                    status
                } else if self.transport_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
                } else {
                    SPIRE_REMOTE_STATUS_READY
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteSearchMergeResult {
    pub(crate) candidates: Vec<SpireRemoteSearchCandidateRow>,
    pub(crate) input_count: u64,
    pub(crate) duplicate_vec_id_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteSearchCandidateBatch {
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) candidates: Vec<SpireRemoteSearchCandidateRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteSearchFanoutTarget {
    node_id: u32,
    selected_pids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteSearchSkippedPlacement {
    node_id: u32,
    pid: u64,
    state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteSearchFanoutPlan {
    requested_epoch: u64,
    local_selected_pids: Vec<u64>,
    remote_targets: Vec<SpireRemoteSearchFanoutTarget>,
    skipped_placements: Vec<SpireRemoteSearchSkippedPlacement>,
}

fn plan_remote_search_fanout(
    snapshot: &meta::SpirePublishedEpochSnapshot<'_>,
    selected_leaf_pids: &[u64],
) -> Result<SpireRemoteSearchFanoutPlan, String> {
    let snapshot = meta::SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    if selected_leaf_pids.is_empty() {
        return Ok(SpireRemoteSearchFanoutPlan {
            requested_epoch: snapshot.epoch_manifest().epoch,
            local_selected_pids: Vec::new(),
            remote_targets: Vec::new(),
            skipped_placements: Vec::new(),
        });
    }

    let mut seen = HashSet::new();
    let mut local_selected_pids = Vec::new();
    let mut remote_by_node = BTreeMap::<u32, Vec<u64>>::new();
    let mut skipped_placements = Vec::new();

    for &pid in selected_leaf_pids {
        if pid == 0 {
            return Err("ec_spire remote search fanout selected PID 0 is invalid".to_owned());
        }
        if !seen.insert(pid) {
            return Err(format!(
                "ec_spire remote search fanout selected PID {pid} appears more than once"
            ));
        }

        let lookup = snapshot.require_lookup(pid, "remote search fanout selected leaf")?;
        if fanout_should_skip_placement(
            snapshot.epoch_manifest().consistency_mode,
            lookup.placement.state,
        )? {
            skipped_placements.push(SpireRemoteSearchSkippedPlacement {
                node_id: lookup.placement.node_id,
                pid,
                state: fanout_placement_state_name(lookup.placement.state),
            });
            continue;
        }

        if lookup.placement.node_id == meta::SPIRE_LOCAL_NODE_ID {
            local_selected_pids.push(pid);
        } else {
            remote_by_node
                .entry(lookup.placement.node_id)
                .or_default()
                .push(pid);
        }
    }

    let remote_targets = remote_by_node
        .into_iter()
        .map(|(node_id, selected_pids)| SpireRemoteSearchFanoutTarget {
            node_id,
            selected_pids,
        })
        .collect();

    Ok(SpireRemoteSearchFanoutPlan {
        requested_epoch: snapshot.epoch_manifest().epoch,
        local_selected_pids,
        remote_targets,
        skipped_placements,
    })
}

fn fanout_should_skip_placement(
    consistency_mode: meta::SpireConsistencyMode,
    state: meta::SpirePlacementState,
) -> Result<bool, String> {
    match (consistency_mode, state) {
        (_, meta::SpirePlacementState::Available) => Ok(false),
        (meta::SpireConsistencyMode::Degraded, meta::SpirePlacementState::Unavailable)
        | (meta::SpireConsistencyMode::Degraded, meta::SpirePlacementState::Skipped) => Ok(true),
        (meta::SpireConsistencyMode::Strict, state) => Err(format!(
            "ec_spire strict remote search fanout cannot skip {:?} placement",
            state
        )),
        (meta::SpireConsistencyMode::Degraded, meta::SpirePlacementState::Stale) => {
            Err("ec_spire degraded remote search fanout cannot use stale placement".to_owned())
        }
    }
}

fn fanout_placement_state_name(state: meta::SpirePlacementState) -> &'static str {
    match state {
        meta::SpirePlacementState::Available => "available",
        meta::SpirePlacementState::Stale => "stale",
        meta::SpirePlacementState::Unavailable => "unavailable",
        meta::SpirePlacementState::Skipped => "skipped",
    }
}

pub(crate) unsafe fn remote_search_fanout_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    selected_pids: Vec<u64>,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchFanoutPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchFanoutPlanRow>, String> {
        if requested_epoch == 0 {
            return Err(
                "ec_spire remote search fanout requested_epoch must be greater than 0".to_owned(),
            );
        }
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch != requested_epoch {
            return Err(format!(
                "ec_spire remote search fanout requested epoch {requested_epoch} does not match active epoch {}",
                root_control.active_epoch
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
        };
        if epoch_manifest.consistency_mode != requested_consistency_mode {
            return Err(format!(
                "ec_spire remote search fanout requested consistency_mode '{consistency_mode}' does not match active epoch consistency mode '{}'",
                consistency_mode_name(epoch_manifest.consistency_mode)
            ));
        }
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let plan = plan_remote_search_fanout(&snapshot, &selected_pids)?;
        let validated_snapshot = meta::SpireValidatedEpochSnapshot::from_snapshot(snapshot)?;
        let mut rows = Vec::with_capacity(
            plan.local_selected_pids.len()
                + plan
                    .remote_targets
                    .iter()
                    .map(|target| target.selected_pids.len())
                    .sum::<usize>()
                + plan.skipped_placements.len(),
        );
        for pid in plan.local_selected_pids {
            let placement_state = fanout_placement_state_name(
                validated_snapshot
                    .require_lookup(pid, "remote search fanout local row")?
                    .placement
                    .state,
            );
            rows.push(SpireRemoteSearchFanoutPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_LOCAL,
                node_id: meta::SPIRE_LOCAL_NODE_ID,
                pid,
                placement_state,
            });
        }
        for target in plan.remote_targets {
            for pid in target.selected_pids {
                let placement_state = fanout_placement_state_name(
                    validated_snapshot
                        .require_lookup(pid, "remote search fanout remote row")?
                        .placement
                        .state,
                );
                rows.push(SpireRemoteSearchFanoutPlanRow {
                    requested_epoch: plan.requested_epoch,
                    target_kind: SPIRE_REMOTE_TARGET_REMOTE,
                    node_id: target.node_id,
                    pid,
                    placement_state,
                });
            }
        }
        rows.extend(plan.skipped_placements.into_iter().map(|skipped| {
            SpireRemoteSearchFanoutPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_SKIPPED,
                node_id: skipped.node_id,
                pid: skipped.pid,
                placement_state: skipped.state,
            }
        }));
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_target_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    selected_pids: Vec<u64>,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchTargetPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchTargetPlanRow>, String> {
        if requested_epoch == 0 {
            return Err(
                "ec_spire remote search target plan requested_epoch must be greater than 0"
                    .to_owned(),
            );
        }
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch != requested_epoch {
            return Err(format!(
                "ec_spire remote search target plan requested epoch {requested_epoch} does not match active epoch {}",
                root_control.active_epoch
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
        };
        if epoch_manifest.consistency_mode != requested_consistency_mode {
            return Err(format!(
                "ec_spire remote search target plan requested consistency_mode '{consistency_mode}' does not match active epoch consistency mode '{}'",
                consistency_mode_name(epoch_manifest.consistency_mode)
            ));
        }
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let plan = plan_remote_search_fanout(&snapshot, &selected_pids)?;
        let mut rows = Vec::new();
        if !plan.local_selected_pids.is_empty() {
            let pid_count = u64::try_from(plan.local_selected_pids.len())
                .map_err(|_| "ec_spire remote search target plan local PID count exceeds u64")?;
            rows.push(SpireRemoteSearchTargetPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_LOCAL,
                node_id: meta::SPIRE_LOCAL_NODE_ID,
                selected_pids: plan.local_selected_pids,
                pid_count,
                placement_state: "available",
                status: SPIRE_REMOTE_STATUS_READY,
            });
        }
        for target in plan.remote_targets {
            let pid_count = u64::try_from(target.selected_pids.len())
                .map_err(|_| "ec_spire remote search target plan remote PID count exceeds u64")?;
            rows.push(SpireRemoteSearchTargetPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_REMOTE,
                node_id: target.node_id,
                selected_pids: target.selected_pids,
                pid_count,
                placement_state: "available",
                status: SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ,
            });
        }

        let mut skipped_by_node_state = BTreeMap::<(u32, &'static str), Vec<u64>>::new();
        for skipped in plan.skipped_placements {
            skipped_by_node_state
                .entry((skipped.node_id, skipped.state))
                .or_default()
                .push(skipped.pid);
        }
        for ((node_id, placement_state), selected_pids) in skipped_by_node_state {
            let pid_count = u64::try_from(selected_pids.len())
                .map_err(|_| "ec_spire remote search target plan skipped PID count exceeds u64")?;
            rows.push(SpireRemoteSearchTargetPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_SKIPPED,
                node_id,
                selected_pids,
                pid_count,
                placement_state,
                status: SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            });
        }

        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_target_readiness_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    selected_pids: Vec<u64>,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchTargetReadinessRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchTargetReadinessRow>, String> {
        let target_rows = unsafe {
            remote_search_target_plan_rows(
                index_relation,
                requested_epoch,
                selected_pids,
                consistency_mode,
            )
        };
        let node_rows = unsafe { remote_node_snapshot(index_relation) }
            .into_iter()
            .map(|row| (row.node_id, row))
            .collect::<BTreeMap<_, _>>();
        let capability_rows = node_rows
            .values()
            .cloned()
            .map(remote_node_capability_plan_row)
            .map(|row| (row.node_id, row))
            .collect::<BTreeMap<_, _>>();

        target_rows
            .into_iter()
            .map(|target| {
                let node = node_rows.get(&target.node_id).ok_or_else(|| {
                    format!(
                        "ec_spire remote search target readiness missing node snapshot for node_id {}",
                        target.node_id
                    )
                })?;
                let capability = capability_rows.get(&target.node_id).ok_or_else(|| {
                    format!(
                        "ec_spire remote search target readiness missing capability plan for node_id {}",
                        target.node_id
                    )
                })?;
                let status = if target.target_kind == SPIRE_REMOTE_TARGET_SKIPPED {
                    target.status
                } else if matches!(
                    node.descriptor_state,
                    SPIRE_REMOTE_DESCRIPTOR_STATE_DISABLED | SPIRE_REMOTE_DESCRIPTOR_STATE_FAILED
                ) {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                } else if node.status != SPIRE_REMOTE_STATUS_READY {
                    node.status
                } else if target.target_kind == SPIRE_REMOTE_TARGET_REMOTE
                    && capability.status != SPIRE_REMOTE_STATUS_READY
                {
                    capability.status
                } else {
                    target.status
                };
                Ok(SpireRemoteSearchTargetReadinessRow {
                    requested_epoch: target.requested_epoch,
                    target_kind: target.target_kind,
                    node_id: target.node_id,
                    selected_pids: target.selected_pids,
                    pid_count: target.pid_count,
                    placement_state: target.placement_state,
                    node_kind: node.node_kind,
                    descriptor_state: node.descriptor_state,
                    node_status: node.status,
                    status,
                })
            })
            .collect()
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_request_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchRequestPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchRequestPlanRow>, String> {
        let query = scan::SpireScanQuery::new(query)?;
        let query_dimension = u64::try_from(query.values().len())
            .map_err(|_| "ec_spire remote search request plan query dimension exceeds u64")?;
        let top_k = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search request plan top_k exceeds u64")?;
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let rows = unsafe {
            remote_search_target_plan_rows(
                index_relation,
                requested_epoch,
                selected_pids,
                consistency_mode,
            )
        };
        Ok(rows
            .into_iter()
            .map(|row| SpireRemoteSearchRequestPlanRow {
                requested_epoch: row.requested_epoch,
                target_kind: row.target_kind,
                node_id: row.node_id,
                selected_pids: row.selected_pids,
                pid_count: row.pid_count,
                query_dimension,
                top_k,
                consistency_mode: consistency_mode_name(requested_consistency_mode),
                endpoint_function: if row.target_kind == SPIRE_REMOTE_TARGET_SKIPPED {
                    SPIRE_REMOTE_NONE
                } else {
                    SPIRE_REMOTE_ENDPOINT_SEARCH
                },
                status: row.status,
            })
            .collect())
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_request_readiness_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchRequestReadinessRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchRequestReadinessRow>, String> {
        let query = scan::SpireScanQuery::new(query)?;
        let query_dimension = u64::try_from(query.values().len())
            .map_err(|_| "ec_spire remote search request readiness query dimension exceeds u64")?;
        let top_k = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search request readiness top_k exceeds u64")?;
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let rows = unsafe {
            remote_search_target_readiness_rows(
                index_relation,
                requested_epoch,
                selected_pids,
                consistency_mode,
            )
        };
        Ok(rows
            .into_iter()
            .map(|row| SpireRemoteSearchRequestReadinessRow {
                requested_epoch: row.requested_epoch,
                target_kind: row.target_kind,
                node_id: row.node_id,
                selected_pids: row.selected_pids,
                pid_count: row.pid_count,
                query_dimension,
                top_k,
                consistency_mode: consistency_mode_name(requested_consistency_mode),
                endpoint_function: if row.target_kind == SPIRE_REMOTE_TARGET_SKIPPED {
                    SPIRE_REMOTE_NONE
                } else {
                    SPIRE_REMOTE_ENDPOINT_SEARCH
                },
                node_kind: row.node_kind,
                descriptor_state: row.descriptor_state,
                node_status: row.node_status,
                status: row.status,
            })
            .collect())
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_request_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchRequestSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchRequestSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search request summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_request_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut rollup = SpireRemoteCountRollup::default();
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_target(row.target_kind, row.pid_count, "remote search request summary")?;
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search request summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let request_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search request summary request count exceeds u64")?;
        let executable_pid_count = rollup.executable_pid_count("remote search request summary")?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::RequestPlan);

        Ok(SpireRemoteSearchRequestSummaryRow {
            requested_epoch,
            request_count,
            local_request_count: rollup.local_count,
            remote_request_count: rollup.remote_count,
            skipped_request_count: rollup.skipped_count,
            executable_pid_count,
            local_pid_count: rollup.local_pid_count,
            remote_pid_count: rollup.remote_pid_count,
            skipped_pid_count: rollup.skipped_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_readiness_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchReadinessSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchReadinessSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search readiness summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_request_readiness_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut rollup = SpireRemoteCountRollup::default();
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_target(row.target_kind, row.pid_count, "remote search readiness summary")?;
            rollup.record_status(row.status, row.pid_count, "remote search readiness summary")?;
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search readiness summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let request_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search readiness summary request count exceeds u64")?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::Readiness);

        Ok(SpireRemoteSearchReadinessSummaryRow {
            requested_epoch,
            request_count,
            ready_request_count: rollup.ready_count,
            blocked_request_count: rollup.blocked_count,
            local_request_count: rollup.local_count,
            remote_request_count: rollup.remote_count,
            skipped_request_count: rollup.skipped_count,
            executable_pid_count: rollup.executable_pid_count("remote search readiness summary")?,
            ready_pid_count: rollup.ready_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            skipped_pid_count: rollup.skipped_pid_count,
            missing_descriptor_request_count: rollup.missing_descriptor_count,
            missing_descriptor_pid_count: rollup.missing_descriptor_pid_count,
            transport_request_count: rollup.transport_count,
            transport_pid_count: rollup.transport_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_execution_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchExecutionPlanRow> {
    let rows = unsafe {
        remote_search_request_readiness_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    rows.into_iter()
        .map(|row| {
            remote_search_execution_plan_row_from_readiness(row)
        })
        .collect()
}

fn remote_search_execution_plan_row_from_readiness(
    row: SpireRemoteSearchRequestReadinessRow,
) -> SpireRemoteSearchExecutionPlanRow {
    let (execution_transport, remote_index_source, conninfo_source, candidate_format) =
        match row.target_kind {
            SPIRE_REMOTE_TARGET_LOCAL => (
                SPIRE_REMOTE_TRANSPORT_LOCAL_DIRECT,
                SPIRE_REMOTE_INDEX_SOURCE_LOCAL_OID,
                SPIRE_REMOTE_CANDIDATE_FORMAT_LOCAL,
                SPIRE_REMOTE_CANDIDATE_FORMAT_LOCAL,
            ),
            SPIRE_REMOTE_TARGET_REMOTE => (
                SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
                SPIRE_REMOTE_DESCRIPTOR_SOURCE,
                SPIRE_REMOTE_DESCRIPTOR_SOURCE,
                SPIRE_REMOTE_CANDIDATE_FORMAT_V1,
            ),
            SPIRE_REMOTE_TARGET_SKIPPED => (
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
            ),
            _ => ("unknown", "unknown", "unknown", "unknown"),
        };
    SpireRemoteSearchExecutionPlanRow {
        requested_epoch: row.requested_epoch,
        target_kind: row.target_kind,
        node_id: row.node_id,
        selected_pids: row.selected_pids,
        pid_count: row.pid_count,
        query_dimension: row.query_dimension,
        top_k: row.top_k,
        consistency_mode: row.consistency_mode,
        execution_transport,
        endpoint_function: row.endpoint_function,
        remote_index_source,
        conninfo_source,
        candidate_format,
        status: row.status,
    }
}

pub(crate) unsafe fn remote_search_execution_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchExecutionSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchExecutionSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search execution summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_execution_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_execution_summary_from_plan_rows(
            requested_epoch,
            &rows,
            query_for_empty_plan,
            top_k_for_empty_plan,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_execution_summary_from_plan_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchExecutionPlanRow],
    query_for_empty_plan: Vec<f32>,
    top_k_for_empty_plan: u64,
    consistency_mode: &str,
) -> Result<SpireRemoteSearchExecutionSummaryRow, String> {
        let mut rollup = SpireRemoteCountRollup::default();
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_target(row.target_kind, row.pid_count, "remote search execution summary")?;
            rollup.record_status(row.status, row.pid_count, "remote search execution summary")?;
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search execution summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let plan_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search execution summary plan count exceeds u64")?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::Execution);

        Ok(SpireRemoteSearchExecutionSummaryRow {
            requested_epoch,
            plan_count,
            local_plan_count: rollup.local_count,
            remote_plan_count: rollup.remote_count,
            skipped_plan_count: rollup.skipped_count,
            ready_plan_count: rollup.ready_count,
            blocked_plan_count: rollup.blocked_count,
            degraded_skipped_plan_count: rollup.degraded_skipped_count,
            local_pid_count: rollup.local_pid_count,
            remote_pid_count: rollup.remote_pid_count,
            skipped_pid_count: rollup.skipped_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
}

const SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE: &str =
    "SELECT * FROM ec_spire_remote_search($1::oid, $2::bigint, $3::real[], $4::bigint[], $5::integer, $6::text)";
const SPIRE_REMOTE_SEARCH_LIBPQ_HEAP_SQL_TEMPLATE: &str =
    "SELECT * FROM ec_spire_remote_search_local_heap_candidates($1::oid, $2::bigint, $3::real[], $4::bigint[], $5::integer, $6::text)";
const SPIRE_REMOTE_SEARCH_LIBPQ_TUPLE_PAYLOAD_SQL_TEMPLATE: &str =
    "SELECT payload.*, payload.tuple_payload::text AS tuple_payload_text \
       FROM ec_spire_remote_search_tuple_payload($1::oid, $2::bigint, $3::real[], $4::bigint[], $5::integer, $6::text, $7::text[]) AS payload";
const SPIRE_REMOTE_SEARCH_LIBPQ_TYPED_TUPLE_PAYLOAD_SQL_TEMPLATE: &str =
    "SELECT requested_epoch, served_epoch, node_id, pid, object_version, row_index, \
            assignment_flags, vec_id, row_locator, heap_block, heap_offset, score, \
            payload_attnums, payload_names, payload_type_oids::text[] AS payload_type_oids, \
            payload_typmods, payload_collations::text[] AS payload_collations, \
            payload_nulls, \
            ARRAY(SELECT encode(payload_value, 'hex') FROM unnest(payload_values) AS payload_value)::text[] AS payload_values_hex, \
            payload_formats, tuple_payload_missing, payload_key, payload_column_count, \
            tuple_transport, tuple_transport_status, status \
       FROM ec_spire_remote_search_tuple_payload_typed($1::oid, $2::bigint, $3::real[], $4::bigint[], $5::integer, $6::text, $7::text[])";
const SPIRE_REMOTE_SEARCH_ENDPOINT_IDENTITY_SQL_TEMPLATE: &str =
    "SELECT * FROM ec_spire_remote_search_endpoint_identity($1::oid)";
const SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT: u64 = 6;
const SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR: &str = "validate_remote_search_candidate_batch";
const SPIRE_REMOTE_SEARCH_MERGE_FUNCTION: &str =
    "merge_validated_remote_search_candidate_batches";

fn remote_search_result_column_count() -> u64 {
    u64::try_from(remote_search_libpq_result_contract_rows().len())
        .expect("remote search result contract row count should fit in u64")
}

pub(crate) unsafe fn remote_search_libpq_request_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqRequestPlanRow> {
    let rows = unsafe {
        remote_search_execution_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_libpq_request_plan_rows_from_execution(&rows)
}

fn remote_search_libpq_request_plan_rows_from_execution(
    rows: &[SpireRemoteSearchExecutionPlanRow],
) -> Vec<SpireRemoteSearchLibpqRequestPlanRow> {
    rows.iter()
        .filter(|row| row.target_kind == SPIRE_REMOTE_TARGET_REMOTE)
        .map(|row| SpireRemoteSearchLibpqRequestPlanRow {
            requested_epoch: row.requested_epoch,
            node_id: row.node_id,
            selected_pids: row.selected_pids.clone(),
            pid_count: row.pid_count,
            query_dimension: row.query_dimension,
            top_k: row.top_k,
            consistency_mode: row.consistency_mode,
            execution_transport: row.execution_transport,
            sql_template: SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
            parameter_count: SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT,
            result_column_count: remote_search_result_column_count(),
            remote_index_source: row.remote_index_source,
            conninfo_source: row.conninfo_source,
            candidate_format: row.candidate_format,
            status: row.status,
        })
        .collect()
}

pub(crate) unsafe fn remote_search_libpq_request_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqRequestSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqRequestSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search libpq request summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_libpq_request_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut rollup = SpireRemoteCountRollup::default();
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_remote_target(row.pid_count, "remote search libpq request summary")?;
            rollup.record_status(row.status, row.pid_count, "remote search libpq request summary")?;
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search libpq request summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let request_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search libpq request summary request count exceeds u64")?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::LibpqRequest);

        Ok(SpireRemoteSearchLibpqRequestSummaryRow {
            requested_epoch,
            request_count,
            ready_request_count: rollup.ready_count,
            blocked_request_count: rollup.blocked_count,
            remote_pid_count: rollup.remote_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            parameter_count_per_request: SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT,
            result_column_count: remote_search_result_column_count(),
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteLibpqConnectionDescriptorRow {
    descriptor_generation: u64,
    conninfo_secret_name: String,
    remote_index_regclass: String,
    remote_index_identity: Vec<u8>,
    remote_index_identity_bytes: u64,
    coordinator_insert_shape_fingerprint: String,
    last_served_epoch: u64,
    min_retained_epoch: u64,
}

fn load_remote_libpq_connection_descriptors(
    index_relid: pg_sys::Oid,
    remote_node_ids: &[u32],
) -> Result<HashMap<u32, SpireRemoteLibpqConnectionDescriptorRow>, String> {
    if remote_node_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let node_id_list = remote_node_ids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT node_id::int4, \
                descriptor_generation::bigint, \
                conninfo_secret_name, \
                remote_index_identity, \
                remote_index_regclass, \
                coordinator_insert_shape_fingerprint, \
                last_served_epoch::bigint, \
                min_retained_epoch::bigint \
           FROM ec_spire_remote_node_descriptor \
          WHERE coordinator_index_oid = '{}'::oid \
            AND node_id = ANY (ARRAY[{}]::integer[]) \
            AND descriptor_state IN ('{}', '{}')",
        u32::from(index_relid),
        node_id_list,
        SPIRE_REMOTE_DESCRIPTOR_STATE_ACTIVE,
        SPIRE_REMOTE_DESCRIPTOR_STATE_DRAINING
    );

    Spi::connect(|client| {
        client
            .select(sql.as_str(), None, &[])
            .map_err(|e| format!("ec_spire libpq connection descriptor read failed: {e}"))?
            .map(|row| {
                let node_id = row["node_id"]
                    .value::<i32>()
                    .map_err(|e| format!("ec_spire libpq connection node_id decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire libpq connection node_id is null".to_owned())
                    .and_then(|value| {
                        u32::try_from(value)
                            .map_err(|_| "ec_spire libpq connection node_id is negative".to_owned())
                    })?;
                let descriptor_generation = row["descriptor_generation"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!(
                            "ec_spire libpq connection descriptor generation decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection descriptor generation is null".to_owned()
                    })
                    .and_then(|value| {
                        u64::try_from(value).map_err(|_| {
                            "ec_spire libpq connection descriptor generation is negative"
                                .to_owned()
                        })
                    })?;
                let conninfo_secret_name = row["conninfo_secret_name"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("ec_spire libpq connection conninfo secret decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection conninfo secret is null".to_owned()
                    })?;
                let remote_index_identity = row["remote_index_identity"]
                    .value::<Vec<u8>>()
                    .map_err(|e| {
                        format!("ec_spire libpq connection remote identity decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection remote identity is null".to_owned()
                    })?;
                let remote_index_regclass = row["remote_index_regclass"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("ec_spire libpq connection remote regclass decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection remote regclass is null".to_owned()
                    })?;
                let remote_index_identity_bytes = u64::try_from(remote_index_identity.len())
                    .map_err(|_| {
                        "ec_spire libpq connection remote identity length exceeds u64".to_owned()
                    })?;
                let coordinator_insert_shape_fingerprint = row
                    ["coordinator_insert_shape_fingerprint"]
                    .value::<String>()
                    .map_err(|e| {
                        format!(
                            "ec_spire libpq connection coordinator insert shape fingerprint decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection coordinator insert shape fingerprint is null"
                            .to_owned()
                    })?;
                let last_served_epoch = row["last_served_epoch"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!("ec_spire libpq connection last served epoch decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection last served epoch is null".to_owned()
                    })
                    .and_then(|value| {
                        u64::try_from(value).map_err(|_| {
                            "ec_spire libpq connection last served epoch is negative".to_owned()
                        })
                    })?;
                let min_retained_epoch = row["min_retained_epoch"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!("ec_spire libpq connection min retained epoch decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection min retained epoch is null".to_owned()
                    })
                    .and_then(|value| {
                        u64::try_from(value).map_err(|_| {
                            "ec_spire libpq connection min retained epoch is negative".to_owned()
                        })
                    })?;

                Ok((
                    node_id,
                    SpireRemoteLibpqConnectionDescriptorRow {
                        descriptor_generation,
                        conninfo_secret_name,
                        remote_index_regclass,
                        remote_index_identity,
                        remote_index_identity_bytes,
                        coordinator_insert_shape_fingerprint,
                        last_served_epoch,
                        min_retained_epoch,
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>, String>>()
    })
}

pub(crate) unsafe fn remote_search_libpq_connection_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqConnectionPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchLibpqConnectionPlanRow>, String> {
        let request_rows = unsafe {
            remote_search_libpq_request_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_libpq_connection_plan_rows_from_requests(
            unsafe { (*index_relation).rd_id },
            &request_rows,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_connection_plan_rows_from_requests(
    index_relid: pg_sys::Oid,
    request_rows: &[SpireRemoteSearchLibpqRequestPlanRow],
) -> Result<Vec<SpireRemoteSearchLibpqConnectionPlanRow>, String> {
    let remote_node_ids = request_rows
        .iter()
        .map(|row| row.node_id)
        .collect::<Vec<_>>();
    let descriptors = load_remote_libpq_connection_descriptors(index_relid, &remote_node_ids)?;

    request_rows
        .iter()
        .map(|row| {
            let descriptor = descriptors.get(&row.node_id);
            let pipeline_ready =
                descriptor.is_some() && row.status == SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ;
            Ok(SpireRemoteSearchLibpqConnectionPlanRow {
                requested_epoch: row.requested_epoch,
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                query_dimension: row.query_dimension,
                top_k: row.top_k,
                consistency_mode: row.consistency_mode,
                execution_transport: row.execution_transport,
                conninfo_secret_name: descriptor
                    .map(|row| row.conninfo_secret_name.clone())
                    .unwrap_or_else(|| SPIRE_REMOTE_NONE.to_owned()),
                remote_index_regclass: descriptor
                    .map(|row| row.remote_index_regclass.clone())
                    .unwrap_or_else(|| SPIRE_REMOTE_NONE.to_owned()),
                descriptor_generation: descriptor
                    .map(|row| row.descriptor_generation)
                    .unwrap_or(0),
                remote_index_identity: descriptor
                    .map(|row| row.remote_index_identity.clone())
                    .unwrap_or_default(),
                remote_index_identity_bytes: descriptor
                    .map(|row| row.remote_index_identity_bytes)
                    .unwrap_or(0),
                conninfo_resolution: if descriptor.is_some() {
                    SPIRE_REMOTE_CONNINFO_READY
                } else {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                },
                pipeline_mode: if pipeline_ready {
                    SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE
                } else {
                    SPIRE_REMOTE_NONE
                },
                status: row.status,
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

pub(crate) unsafe fn remote_search_libpq_connection_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqConnectionSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqConnectionSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search libpq connection summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_libpq_connection_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut rollup = SpireRemoteCountRollup::default();
        let mut descriptor_resolved_connection_count = 0_u64;
        let mut missing_descriptor_connection_count = 0_u64;
        let mut pipeline_connection_count = 0_u64;
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_remote_target(row.pid_count, "remote search libpq connection summary")?;
            rollup
                .record_status(row.status, row.pid_count, "remote search libpq connection summary")?;
            if row.conninfo_resolution == SPIRE_REMOTE_CONNINFO_READY {
                descriptor_resolved_connection_count =
                    descriptor_resolved_connection_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            "ec_spire remote search libpq connection summary resolved count overflow"
                                .to_owned()
                        })?;
            }
            if row.conninfo_resolution == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
                missing_descriptor_connection_count =
                    missing_descriptor_connection_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            "ec_spire remote search libpq connection summary missing descriptor count overflow"
                                .to_owned()
                        })?;
            }
            if row.pipeline_mode == SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE {
                pipeline_connection_count =
                    pipeline_connection_count.checked_add(1).ok_or_else(|| {
                        "ec_spire remote search libpq connection summary pipeline count overflow"
                            .to_owned()
                    })?;
            }
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search libpq connection summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let connection_count = u64::try_from(rows.len()).map_err(|_| {
            "ec_spire remote search libpq connection summary connection count exceeds u64"
        })?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::LibpqRequest);

        Ok(SpireRemoteSearchLibpqConnectionSummaryRow {
            requested_epoch,
            connection_count,
            descriptor_resolved_connection_count,
            missing_descriptor_connection_count,
            pipeline_connection_count,
            remote_pid_count: rollup.remote_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_libpq_dispatch_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqDispatchPlanRow> {
    let connection_rows = unsafe {
        remote_search_libpq_connection_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };

    remote_search_libpq_dispatch_plan_rows_from_connections(&connection_rows)
}

fn remote_search_libpq_dispatch_plan_rows_from_connections(
    connection_rows: &[SpireRemoteSearchLibpqConnectionPlanRow],
) -> Vec<SpireRemoteSearchLibpqDispatchPlanRow> {
    let budget_limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
    let mut admitted_node_count = 0_u64;
    let mut admitted_pid_count = 0_u64;

    connection_rows
        .iter()
        .map(|row| {
            let budget_blocked = remote_search_libpq_dispatch_budget_blocked(
                row,
                budget_limits,
                admitted_node_count,
                admitted_pid_count,
            );
            if row.pipeline_mode == SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE && !budget_blocked {
                admitted_node_count = admitted_node_count.saturating_add(1);
                admitted_pid_count = admitted_pid_count.saturating_add(row.pid_count);
            }

            let pipeline_mode = if budget_blocked {
                SPIRE_REMOTE_NONE
            } else {
                row.pipeline_mode
            };
            let dispatch_action = if pipeline_mode == SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE {
                SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION
            } else {
                SPIRE_REMOTE_DISPATCH_BLOCKED_ACTION
            };
            let status = if budget_blocked {
                SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD
            } else {
                row.status
            };

            SpireRemoteSearchLibpqDispatchPlanRow {
                requested_epoch: row.requested_epoch,
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                query_dimension: row.query_dimension,
                top_k: row.top_k,
                consistency_mode: row.consistency_mode,
                sql_template: SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
                parameter_count: SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT,
                result_column_count: remote_search_result_column_count(),
                conninfo_secret_name: row.conninfo_secret_name.clone(),
                remote_index_regclass: row.remote_index_regclass.clone(),
                descriptor_generation: row.descriptor_generation,
                remote_index_identity: row.remote_index_identity.clone(),
                pipeline_mode,
                dispatch_action,
                receive_validator: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
                status,
            }
        })
        .collect()
}

pub(crate) unsafe fn coordinator_insert_dispatch_plan_row(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
) -> SpireCoordinatorInsertDispatchPlanRow {
    let index_oid = unsafe { (*index_relation).rd_id };
    let result = (|| -> Result<SpireCoordinatorInsertDispatchPlanRow, String> {
        let descriptors = load_remote_libpq_connection_descriptors(index_oid, &[node_id])?;
        let Some(descriptor) = descriptors.get(&node_id) else {
            return Ok(SpireCoordinatorInsertDispatchPlanRow {
                index_oid,
                node_id,
                served_epoch,
                dispatch_transport: SPIRE_COORDINATOR_INSERT_DISPATCH_TRANSPORT_LIBPQ,
                transaction_protocol: SPIRE_COORDINATOR_INSERT_TRANSACTION_PROTOCOL_2PC,
                conninfo_secret_name: SPIRE_REMOTE_NONE.to_owned(),
                conninfo_provider_lookup_key: SPIRE_REMOTE_NONE.to_owned(),
                remote_index_regclass: SPIRE_REMOTE_NONE.to_owned(),
                descriptor_generation: 0,
                remote_index_identity_bytes: 0,
                coordinator_insert_shape_fingerprint: SPIRE_REMOTE_NONE.to_owned(),
                dispatch_action: SPIRE_COORDINATOR_INSERT_DISPATCH_ACTION_BLOCKED,
                status: SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
                next_step: SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR,
            });
        };

        let secret_status =
            remote_conninfo_secret_resolution_status_row(&descriptor.conninfo_secret_name);
        let epoch_status = if served_epoch > descriptor.last_served_epoch {
            Some(SPIRE_REMOTE_STATUS_STALE_EPOCH)
        } else if served_epoch < descriptor.min_retained_epoch {
            Some(SPIRE_REMOTE_STATUS_RETENTION_GAP)
        } else {
            None
        };
        let (dispatch_action, status, next_step) = if let Some(epoch_status) = epoch_status {
            (
                SPIRE_COORDINATOR_INSERT_DISPATCH_ACTION_BLOCKED,
                epoch_status,
                SPIRE_REMOTE_EXECUTOR_STEP_EPOCH_WINDOW,
            )
        } else if secret_status.status == SPIRE_REMOTE_CONNINFO_RESOLVED {
            (
                SPIRE_COORDINATOR_INSERT_DISPATCH_ACTION_PREPARE,
                SPIRE_REMOTE_STATUS_READY,
                SPIRE_COORDINATOR_INSERT_NEXT_STEP_PREPARE,
            )
        } else {
            (
                SPIRE_COORDINATOR_INSERT_DISPATCH_ACTION_BLOCKED,
                secret_status.status,
                SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
            )
        };

        Ok(SpireCoordinatorInsertDispatchPlanRow {
            index_oid,
            node_id,
            served_epoch,
            dispatch_transport: SPIRE_COORDINATOR_INSERT_DISPATCH_TRANSPORT_LIBPQ,
            transaction_protocol: SPIRE_COORDINATOR_INSERT_TRANSACTION_PROTOCOL_2PC,
            conninfo_secret_name: descriptor.conninfo_secret_name.clone(),
            conninfo_provider_lookup_key: secret_status.provider_lookup_key,
            remote_index_regclass: descriptor.remote_index_regclass.clone(),
            descriptor_generation: descriptor.descriptor_generation,
            remote_index_identity_bytes: descriptor.remote_index_identity_bytes,
            coordinator_insert_shape_fingerprint: descriptor
                .coordinator_insert_shape_fingerprint
                .clone(),
            dispatch_action,
            status,
            next_step,
        })
    })();

    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn quote_sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn coordinator_insert_prepared_gid(
    index_oid: pg_sys::Oid,
    node_id: u32,
    served_epoch: u64,
) -> String {
    let transaction_id = unsafe { pg_sys::GetTopTransactionId() };
    format!(
        "ec_spire_insert_{}_{}_{}_{}",
        u32::from(index_oid),
        node_id,
        served_epoch,
        u32::from(transaction_id)
    )
}

fn coordinator_insert_resolve_remote_prepared(
    conninfo: String,
    node_id: u32,
    gid: String,
    commit: bool,
) {
    let context = if commit {
        "coordinator insert remote prepared commit callback"
    } else {
        "coordinator insert remote prepared rollback callback"
    };
    let Ok(mut client) =
        remote_search_libpq_connect_with_session_timeouts(&conninfo, node_id, context)
    else {
        return;
    };
    let command = if commit {
        "COMMIT PREPARED"
    } else {
        "ROLLBACK PREPARED"
    };
    let _ = client.batch_execute(&format!("{command} {}", quote_sql_literal(&gid)));
}

fn coordinator_insert_remote_tuple_payload_sql(
    remote_index_regclass: &str,
    row_payload_json: &str,
    requested_columns: &[String],
) -> Result<String, String> {
    if requested_columns.is_empty() {
        return Err("ec_spire coordinator insert tuple payload column list is empty".to_owned());
    }
    let column_literals = requested_columns
        .iter()
        .map(|column| {
            if column.is_empty() {
                return Err(
                    "ec_spire coordinator insert tuple payload column name is empty".to_owned(),
                );
            }
            Ok(quote_sql_literal(column))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join(", ");
    Ok(format!(
        "SELECT * FROM ec_spire_remote_insert_tuple_payload(\
             {}::regclass, {}::jsonb, ARRAY[{}]::text[])",
        quote_sql_literal(remote_index_regclass),
        quote_sql_literal(row_payload_json),
        column_literals
    ))
}

fn coordinator_update_remote_tuple_payload_sql(
    remote_index_regclass: &str,
    pk_column: &str,
    pk_value: &[u8],
    row_payload_json: &str,
    updated_columns: &[String],
) -> Result<String, String> {
    if pk_column.is_empty() {
        return Err("ec_spire coordinator update pk column is empty".to_owned());
    }
    if updated_columns.is_empty() {
        return Err("ec_spire coordinator update column list is empty".to_owned());
    }
    let column_literals = updated_columns
        .iter()
        .map(|column| {
            if column.is_empty() {
                return Err("ec_spire coordinator update column name is empty".to_owned());
            }
            Ok(quote_sql_literal(column))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join(", ");
    Ok(format!(
        "SELECT * FROM ec_spire_remote_update_tuple_payload(\
             {}::regclass, {}::text, decode({}, 'hex'), {}::jsonb, ARRAY[{}]::text[])",
        quote_sql_literal(remote_index_regclass),
        quote_sql_literal(pk_column),
        quote_sql_literal(&hex::encode(pk_value)),
        quote_sql_literal(row_payload_json),
        column_literals
    ))
}

fn coordinator_delete_remote_tuple_payload_sql(
    remote_index_regclass: &str,
    pk_column: &str,
    pk_value: &[u8],
) -> Result<String, String> {
    if pk_column.is_empty() {
        return Err("ec_spire coordinator delete pk column is empty".to_owned());
    }
    if pk_value.is_empty() {
        return Err("ec_spire coordinator delete pk_value is empty".to_owned());
    }
    Ok(format!(
        "SELECT * FROM ec_spire_remote_delete_tuple_payload(\
             {}::regclass, {}::text, decode({}, 'hex'))",
        quote_sql_literal(remote_index_regclass),
        quote_sql_literal(pk_column),
        quote_sql_literal(&hex::encode(pk_value))
    ))
}

fn coordinator_select_remote_tuple_payload_sql(
    remote_index_regclass: &str,
    pk_column: &str,
    pk_value: &[u8],
    requested_columns: &[String],
) -> Result<String, String> {
    if pk_column.is_empty() {
        return Err("ec_spire coordinator select pk column is empty".to_owned());
    }
    if pk_value.is_empty() {
        return Err("ec_spire coordinator select pk_value is empty".to_owned());
    }
    if requested_columns.is_empty() {
        return Err("ec_spire coordinator select column list is empty".to_owned());
    }
    let column_literals = requested_columns
        .iter()
        .map(|column| {
            if column.is_empty() {
                return Err("ec_spire coordinator select column name is empty".to_owned());
            }
            Ok(quote_sql_literal(column))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join(", ");
    Ok(format!(
        "SELECT * FROM ec_spire_remote_select_tuple_payload(\
             {}::regclass, {}::text, decode({}, 'hex'), ARRAY[{}]::text[])",
        quote_sql_literal(remote_index_regclass),
        quote_sql_literal(pk_column),
        quote_sql_literal(&hex::encode(pk_value)),
        column_literals
    ))
}

fn coordinator_insert_remote_descriptor_metadata(
    client: &mut postgres::Client,
    node_id: u32,
    remote_index_regclass: &str,
) -> Result<(u64, Vec<u8>, String), String> {
    let row = client
        .query_one(
            "SELECT h.active_epoch::bigint AS active_epoch, \
                    e.protocol_version, e.extension_version, e.opclass_identity, \
                    e.storage_format, e.assignment_payload_format, e.quantizer_profile, \
                    e.scoring_profile, e.profile_fingerprint, e.status, e.recommendation \
               FROM ec_spire_index_hierarchy_snapshot($1::text::regclass) h \
              CROSS JOIN ec_spire_remote_search_endpoint_identity($1::text::regclass::oid) e",
            &[&remote_index_regclass],
        )
        .map_err(|error| {
            format!(
                "ec_spire coordinator insert remote descriptor metadata query failed for node_id {node_id}: {error}"
            )
        })?;
    let active_epoch = row
        .try_get::<_, i64>("active_epoch")
        .map_err(|_| {
            "ec_spire coordinator insert remote descriptor active_epoch decode failed".to_owned()
        })
        .and_then(|value| {
            u64::try_from(value).map_err(|_| {
                "ec_spire coordinator insert remote descriptor active_epoch is negative".to_owned()
            })
        })?;
    let profile_fingerprint = row
        .try_get::<_, String>("profile_fingerprint")
        .map_err(|_| {
            "ec_spire coordinator insert remote descriptor profile_fingerprint decode failed"
                .to_owned()
        })?;
    let remote_index_identity = remote_search_endpoint_profile_fingerprint_bytes(&profile_fingerprint)?;
    let extension_version = row
        .try_get::<_, String>("extension_version")
        .map_err(|_| {
            "ec_spire coordinator insert remote descriptor extension_version decode failed".to_owned()
        })?;
    if extension_version.is_empty() {
        return Err(
            "ec_spire coordinator insert remote descriptor extension_version is empty".to_owned(),
        );
    }

    Ok((active_epoch, remote_index_identity, extension_version))
}

const SPIRE_PREPARED_TRANSACTION_CAPACITY_HINT: &str =
    "SPIRE requires max_prepared_transactions > 0 and enough free prepared \
     transaction slots on every remote PostgreSQL instance; increase \
     max_prepared_transactions, restart the remote, and size it for peak \
     concurrent coordinator-routed SPIRE writes plus any non-SPIRE prepared \
     transactions";

fn postgres_prepare_transaction_capacity_failure(
    sqlstate: Option<&str>,
    message: &str,
) -> bool {
    if sqlstate == Some("55000") {
        return true;
    }
    let message = message.to_ascii_lowercase();
    let capacity_message = message.contains("prepared transactions are disabled")
        || message.contains("maximum number of prepared transactions")
        || message.contains("max_prepared_transactions");
    capacity_message && matches!(sqlstate, Some("53300" | "53400") | None)
}

fn spire_remote_prepare_transaction_error(
    operation: &str,
    node_id: u32,
    error: &postgres::Error,
) -> String {
    let base = format!(
        "ec_spire coordinator {operation} remote PREPARE TRANSACTION failed for node_id {node_id}: {error}"
    );
    let (sqlstate, message) = error
        .as_db_error()
        .map(|db_error| (Some(db_error.code().code()), db_error.message()))
        .unwrap_or((None, base.as_str()));
    if postgres_prepare_transaction_capacity_failure(sqlstate, message) {
        format!("{base}; {SPIRE_PREPARED_TRANSACTION_CAPACITY_HINT}")
    } else {
        base
    }
}

fn postgres_error_message_with_detail(error: &postgres::Error) -> String {
    let Some(db_error) = error.as_db_error() else {
        return error.to_string();
    };
    let mut message = format!("{} (SQLSTATE {})", db_error.message(), db_error.code().code());
    if let Some(detail) = db_error.detail() {
        if !detail.is_empty() {
            message.push_str("; DETAIL: ");
            message.push_str(detail);
        }
    }
    if let Some(hint) = db_error.hint() {
        if !hint.is_empty() {
            message.push_str("; HINT: ");
            message.push_str(hint);
        }
    }
    message
}

fn coordinator_write_current_shape_fingerprint(index_oid: pg_sys::Oid) -> Result<String, String> {
    let sql = format!(
        "SELECT ec_spire_coordinator_index_shape_fingerprint('{}'::oid::regclass) AS fingerprint",
        u32::from(index_oid)
    );
    Spi::get_one::<String>(sql.as_str())
        .map_err(|e| format!("ec_spire coordinator write shape fingerprint read failed: {e}"))?
        .ok_or_else(|| {
            "ec_spire coordinator write shape fingerprint returned no row for index".to_owned()
        })
}

fn validate_coordinator_write_shape_fingerprint(
    operation: &str,
    index_oid: pg_sys::Oid,
    descriptor_fingerprint: &str,
) -> Result<(), String> {
    if descriptor_fingerprint == SPIRE_REMOTE_NONE || descriptor_fingerprint == "unset" {
        return Err(format!(
            "ec_spire coordinator {operation} schema drift guard is missing descriptor fingerprint; refresh remote node descriptors before coordinator-routed writes"
        ));
    }
    let current_fingerprint = coordinator_write_current_shape_fingerprint(index_oid)?;
    if current_fingerprint != descriptor_fingerprint {
        return Err(format!(
            "ec_spire coordinator {operation} schema drift detected for index_oid {}: descriptor fingerprint {} does not match current coordinator fingerprint {}; pause writes, apply matching DDL on every remote, refresh descriptors, then retry",
            u32::from(index_oid),
            descriptor_fingerprint,
            current_fingerprint
        ));
    }
    Ok(())
}

pub(crate) unsafe fn coordinator_insert_prepare_remote_sql(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
    remote_sql: &str,
) -> Result<SpireCoordinatorInsertRemotePrepareRow, String> {
    let dispatch =
        unsafe { coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch) };
    if dispatch.status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire coordinator insert remote dispatch for node_id {} is blocked with status {}",
            node_id, dispatch.status
        ));
    }
    validate_coordinator_write_shape_fingerprint(
        "insert",
        dispatch.index_oid,
        &dispatch.coordinator_insert_shape_fingerprint,
    )?;

    let _governance_permit = remote_search_libpq_executor_governance_permit_for_node(node_id)?;
    let conninfo = remote_conninfo_secret_value(&dispatch.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire coordinator insert conninfo secret for node_id {node_id} is not resolved: {status}"
        )
    })?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        node_id,
        "coordinator insert remote prepare",
    )?;
    let prepared_gid = coordinator_insert_prepared_gid(dispatch.index_oid, node_id, served_epoch);
    client
        .batch_execute("BEGIN")
        .map_err(|_| {
            format!(
                "ec_spire coordinator insert failed to begin remote transaction for node_id {node_id}"
            )
    })?;
    let cancel_watcher = SpireSyncPostgresCancelWatcher::start(client.cancel_token());
    if let Err(error) = client.batch_execute(remote_sql) {
        let local_cancel_observed = cancel_watcher.observed_local_cancel();
        drop(cancel_watcher);
        let _ = client.batch_execute("ROLLBACK");
        if local_cancel_observed {
            return Err(coordinator_remote_local_cancel_error(
                "insert",
                node_id,
                postgres_local_cancel_failure_category(),
            ));
        }
        let error = postgres_error_message_with_detail(&error);
        return Err(format!(
            "ec_spire coordinator insert remote SQL failed for node_id {node_id}: {error}"
        ));
    }
    let (remote_last_served_epoch, remote_index_identity, remote_extension_version) =
        match coordinator_insert_remote_descriptor_metadata(
            &mut client,
            node_id,
            &dispatch.remote_index_regclass,
        ) {
            Ok(metadata) => metadata,
            Err(error) => {
                let local_cancel_observed = cancel_watcher.observed_local_cancel();
                drop(cancel_watcher);
                let _ = client.batch_execute("ROLLBACK");
                if local_cancel_observed {
                    return Err(coordinator_remote_local_cancel_error(
                        "insert",
                        node_id,
                        postgres_local_cancel_failure_category(),
                    ));
                }
                return Err(error);
            }
        };
    if let Err(error) = client.batch_execute(&format!(
        "PREPARE TRANSACTION {}",
        quote_sql_literal(&prepared_gid)
    )) {
        let local_cancel_observed = cancel_watcher.observed_local_cancel();
        drop(cancel_watcher);
        let _ = client.batch_execute("ROLLBACK");
        if local_cancel_observed {
            return Err(coordinator_remote_local_cancel_error(
                "insert",
                node_id,
                postgres_local_cancel_failure_category(),
            ));
        }
        return Err(spire_remote_prepare_transaction_error(
            "insert", node_id, &error,
        ));
    }
    let local_cancel_observed = cancel_watcher.observed_local_cancel();
    drop(cancel_watcher);
    if local_cancel_observed {
        coordinator_insert_resolve_remote_prepared(
            conninfo.clone(),
            node_id,
            prepared_gid.clone(),
            false,
        );
        return Err(coordinator_remote_local_cancel_error(
            "insert",
            node_id,
            postgres_local_cancel_failure_category(),
        ));
    }

    let commit_conninfo = conninfo.clone();
    let commit_gid = prepared_gid.clone();
    let rollback_gid = prepared_gid.clone();
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Commit, move || {
        coordinator_insert_resolve_remote_prepared(commit_conninfo, node_id, commit_gid, true);
    });
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Abort, move || {
        coordinator_insert_resolve_remote_prepared(conninfo, node_id, rollback_gid, false);
    });

    Ok(SpireCoordinatorInsertRemotePrepareRow {
        node_id,
        prepared_gid,
        remote_insert_sent: true,
        remote_prepared: true,
        descriptor_generation: dispatch.descriptor_generation.saturating_add(1),
        remote_index_identity,
        remote_last_served_epoch,
        remote_min_retained_epoch: remote_last_served_epoch,
        remote_extension_version,
        status: SPIRE_COORDINATOR_INSERT_PREPARED_STATUS,
        next_step: SPIRE_COORDINATOR_INSERT_NEXT_STEP_LOCAL_PLACEMENT,
    })
}

pub(crate) unsafe fn coordinator_insert_prepare_remote_tuple_payload(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
    row_payload_json: &str,
    requested_columns: &[String],
) -> Result<SpireCoordinatorInsertRemotePrepareRow, String> {
    let dispatch =
        unsafe { coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch) };
    if dispatch.status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire coordinator insert remote dispatch for node_id {} is blocked with status {}",
            node_id, dispatch.status
        ));
    }
    let remote_sql = coordinator_insert_remote_tuple_payload_sql(
        &dispatch.remote_index_regclass,
        row_payload_json,
        requested_columns,
    )?;
    unsafe {
        coordinator_insert_prepare_remote_sql(index_relation, node_id, served_epoch, &remote_sql)
    }
}

pub(crate) unsafe fn coordinator_update_remote_tuple_payload(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
    pk_column: &str,
    pk_value: &[u8],
    row_payload_json: &str,
    updated_columns: &[String],
) -> Result<SpireCoordinatorUpdateRemoteRow, String> {
    let dispatch =
        unsafe { coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch) };
    if dispatch.status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire coordinator update remote dispatch for node_id {} is blocked with status {}",
            node_id, dispatch.status
        ));
    }
    validate_coordinator_write_shape_fingerprint(
        "update",
        dispatch.index_oid,
        &dispatch.coordinator_insert_shape_fingerprint,
    )?;
    let remote_sql = coordinator_update_remote_tuple_payload_sql(
        &dispatch.remote_index_regclass,
        pk_column,
        pk_value,
        row_payload_json,
        updated_columns,
    )?;

    let _governance_permit = remote_search_libpq_executor_governance_permit_for_node(node_id)?;
    let conninfo = remote_conninfo_secret_value(&dispatch.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire coordinator update conninfo secret for node_id {node_id} is not resolved: {status}"
        )
    })?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        node_id,
        "coordinator update remote dispatch",
    )?;
    let row = client.query_one(remote_sql.as_str(), &[]).map_err(|error| {
        format!("ec_spire coordinator update remote SQL failed for node_id {node_id}: {error}")
    })?;
    let remote_updated_count = row
        .try_get::<_, i64>("updated_count")
        .map_err(|_| "ec_spire coordinator update remote updated_count decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value)
                .map_err(|_| "ec_spire coordinator update remote updated_count is negative".to_owned())
        })?;

    Ok(SpireCoordinatorUpdateRemoteRow {
        node_id,
        remote_update_sent: true,
        remote_updated_count,
        status: "remote_update_applied",
        next_step: "done",
    })
}

pub(crate) unsafe fn coordinator_delete_prepare_remote_tuple_payload(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
    pk_column: &str,
    pk_value: &[u8],
) -> Result<SpireCoordinatorDeleteRemotePrepareRow, String> {
    let dispatch =
        unsafe { coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch) };
    if dispatch.status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire coordinator delete remote dispatch for node_id {} is blocked with status {}",
            node_id, dispatch.status
        ));
    }
    validate_coordinator_write_shape_fingerprint(
        "delete",
        dispatch.index_oid,
        &dispatch.coordinator_insert_shape_fingerprint,
    )?;
    let remote_sql =
        coordinator_delete_remote_tuple_payload_sql(&dispatch.remote_index_regclass, pk_column, pk_value)?;

    let _governance_permit = remote_search_libpq_executor_governance_permit_for_node(node_id)?;
    let conninfo = remote_conninfo_secret_value(&dispatch.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire coordinator delete conninfo secret for node_id {node_id} is not resolved: {status}"
        )
    })?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        node_id,
        "coordinator delete remote prepare",
    )?;
    let prepared_gid = coordinator_insert_prepared_gid(dispatch.index_oid, node_id, served_epoch);
    client.batch_execute("BEGIN").map_err(|_| {
        format!(
            "ec_spire coordinator delete failed to begin remote transaction for node_id {node_id}"
        )
    })?;
    let row = match client.query_one(remote_sql.as_str(), &[]) {
        Ok(row) => row,
        Err(error) => {
            let _ = client.batch_execute("ROLLBACK");
            return Err(format!(
                "ec_spire coordinator delete remote SQL failed for node_id {node_id}: {error}"
            ));
        }
    };
    let remote_deleted_count = row
        .try_get::<_, i64>("deleted_count")
        .map_err(|_| "ec_spire coordinator delete remote deleted_count decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value)
                .map_err(|_| "ec_spire coordinator delete remote deleted_count is negative".to_owned())
        })?;
    client
        .batch_execute(&format!(
            "PREPARE TRANSACTION {}",
            quote_sql_literal(&prepared_gid)
        ))
        .map_err(|error| {
            spire_remote_prepare_transaction_error("delete", node_id, &error)
        })?;

    let commit_conninfo = conninfo.clone();
    let commit_gid = prepared_gid.clone();
    let rollback_gid = prepared_gid.clone();
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Commit, move || {
        coordinator_insert_resolve_remote_prepared(commit_conninfo, node_id, commit_gid, true);
    });
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Abort, move || {
        coordinator_insert_resolve_remote_prepared(conninfo, node_id, rollback_gid, false);
    });

    Ok(SpireCoordinatorDeleteRemotePrepareRow {
        node_id,
        prepared_gid,
        remote_delete_sent: true,
        remote_prepared: true,
        remote_deleted_count,
        status: "remote_delete_prepared",
        next_step: "local_placement_directory_delete",
    })
}

pub(crate) unsafe fn coordinator_select_remote_tuple_payload(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
    pk_column: &str,
    pk_value: &[u8],
    requested_columns: &[String],
) -> Result<SpireCoordinatorSelectRemoteRow, String> {
    let dispatch =
        unsafe { coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch) };
    if dispatch.status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire coordinator select remote dispatch for node_id {} is blocked with status {}",
            node_id, dispatch.status
        ));
    }
    let remote_sql = coordinator_select_remote_tuple_payload_sql(
        &dispatch.remote_index_regclass,
        pk_column,
        pk_value,
        requested_columns,
    )?;

    let _governance_permit = remote_search_libpq_executor_governance_permit_for_node(node_id)?;
    let conninfo = remote_conninfo_secret_value(&dispatch.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire coordinator select conninfo secret for node_id {node_id} is not resolved: {status}"
        )
    })?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        node_id,
        "coordinator select remote dispatch",
    )?;
    let row = client.query_one(remote_sql.as_str(), &[]).map_err(|error| {
        format!("ec_spire coordinator select remote SQL failed for node_id {node_id}: {error}")
    })?;
    let remote_selected_count = row
        .try_get::<_, i64>("selected_count")
        .map_err(|_| "ec_spire coordinator select remote selected_count decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value).map_err(|_| {
                "ec_spire coordinator select remote selected_count is negative".to_owned()
            })
        })?;
    let tuple_payload_json = row
        .try_get::<_, Option<String>>("tuple_payload_json")
        .map_err(|_| "ec_spire coordinator select remote tuple payload decode failed".to_owned())?;

    Ok(SpireCoordinatorSelectRemoteRow {
        node_id,
        remote_select_sent: true,
        remote_selected_count,
        tuple_payload_json,
        status: "remote_select_ready",
        next_step: "done",
    })
}

fn remote_search_libpq_dispatch_budget_blocked(
    row: &SpireRemoteSearchLibpqConnectionPlanRow,
    limits: SpireRemoteSearchLibpqExecutorBudgetLimits,
    admitted_node_count: u64,
    admitted_pid_count: u64,
) -> bool {
    if row.pipeline_mode != SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE {
        return false;
    }
    if limits.has_pid_per_node_cap() && row.pid_count > limits.max_pids_per_node {
        return true;
    }
    if limits.has_node_cap() && admitted_node_count >= limits.max_nodes {
        return true;
    }
    if limits.has_pid_cap() && admitted_pid_count.saturating_add(row.pid_count) > limits.max_pids {
        return true;
    }
    false
}

pub(crate) unsafe fn remote_search_libpq_dispatch_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqDispatchSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqDispatchSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search libpq dispatch summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_libpq_dispatch_summary_from_plan_rows(
            requested_epoch,
            &rows,
            query_for_empty_plan,
            top_k_for_empty_plan,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_dispatch_summary_from_plan_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    query_for_empty_plan: Vec<f32>,
    top_k_for_empty_plan: u64,
    consistency_mode: &str,
) -> Result<SpireRemoteSearchLibpqDispatchSummaryRow, String> {
        let mut rollup = SpireRemoteCountRollup::default();
        let mut pipeline_dispatch_count = 0_u64;
        let mut missing_descriptor_dispatch_count = 0_u64;
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_remote_target(row.pid_count, "remote search libpq dispatch summary")?;
            rollup.record_status(row.status, row.pid_count, "remote search libpq dispatch summary")?;
            if row.dispatch_action == SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
                pipeline_dispatch_count = pipeline_dispatch_count.checked_add(1).ok_or_else(|| {
                    "ec_spire remote search libpq dispatch summary pipeline count overflow".to_owned()
                })?;
            }
            if row.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
                missing_descriptor_dispatch_count =
                    missing_descriptor_dispatch_count.checked_add(1).ok_or_else(|| {
                        "ec_spire remote search libpq dispatch summary missing descriptor count overflow"
                            .to_owned()
                    })?;
            }
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search libpq dispatch summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let dispatch_count = u64::try_from(rows.len()).map_err(|_| {
            "ec_spire remote search libpq dispatch summary dispatch count exceeds u64"
        })?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::LibpqRequest);

        Ok(SpireRemoteSearchLibpqDispatchSummaryRow {
            requested_epoch,
            dispatch_count,
            pipeline_dispatch_count,
            missing_descriptor_dispatch_count,
            remote_pid_count: rollup.remote_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
}

pub(crate) unsafe fn remote_search_libpq_executor_budget_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqExecutorBudgetSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqExecutorBudgetSummaryRow, String> {
        let rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_libpq_executor_budget_summary_from_dispatch_rows(requested_epoch, &rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_executor_budget_summary_from_dispatch_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
) -> Result<SpireRemoteSearchLibpqExecutorBudgetSummaryRow, String> {
    let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
    let mut admitted_dispatch_count = 0_u64;
    let mut budget_blocked_dispatch_count = 0_u64;
    let mut remote_pid_count = 0_u64;
    let mut admitted_pid_count = 0_u64;
    let mut budget_blocked_pid_count = 0_u64;

    for row in rows {
        add_remote_count(
            &mut remote_pid_count,
            row.pid_count,
            "remote search libpq executor budget summary",
            "remote PID",
        )?;
        if row.status == SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD {
            add_remote_count(
                &mut budget_blocked_dispatch_count,
                1,
                "remote search libpq executor budget summary",
                "budget-blocked dispatch",
            )?;
            add_remote_count(
                &mut budget_blocked_pid_count,
                row.pid_count,
                "remote search libpq executor budget summary",
                "budget-blocked PID",
            )?;
        } else if row.dispatch_action == SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
            add_remote_count(
                &mut admitted_dispatch_count,
                1,
                "remote search libpq executor budget summary",
                "admitted dispatch",
            )?;
            add_remote_count(
                &mut admitted_pid_count,
                row.pid_count,
                "remote search libpq executor budget summary",
                "admitted PID",
            )?;
        }
    }

    let dispatch_count = u64::try_from(rows.len())
        .map_err(|_| "remote search libpq executor budget dispatch count exceeds u64")?;
    let (next_executor_step, status, recommendation) = if budget_blocked_dispatch_count > 0 {
        (
            SPIRE_REMOTE_EXECUTOR_STEP_BUDGET,
            SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            remote_search_pre_dispatch_blocker_recommendation(
                SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            ),
        )
    } else {
        (SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
    };

    Ok(SpireRemoteSearchLibpqExecutorBudgetSummaryRow {
        requested_epoch,
        dispatch_count,
        admitted_dispatch_count,
        budget_blocked_dispatch_count,
        remote_pid_count,
        admitted_pid_count,
        budget_blocked_pid_count,
        max_nodes: limits.max_nodes,
        max_pids: limits.max_pids,
        max_pids_per_node: limits.max_pids_per_node,
        max_concurrent_dispatches: limits.max_concurrent_dispatches,
        max_concurrent_dispatches_per_node: limits.max_concurrent_dispatches_per_node,
        connect_timeout_ms: limits.connect_timeout_ms,
        statement_timeout_ms: limits.statement_timeout_ms,
        next_executor_step,
        status,
        recommendation,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteProductionTransportProbeRequest {
    pub(crate) node_id: u32,
    pub(crate) conninfo: String,
    pub(crate) sql: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteProductionCandidateReceiveRequest {
    pub(crate) node_id: u32,
    pub(crate) conninfo: String,
    pub(crate) remote_index_regclass: String,
    pub(crate) remote_index_identity: Vec<u8>,
    pub(crate) requested_epoch: u64,
    pub(crate) query: Vec<f32>,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) top_k: usize,
    pub(crate) consistency_mode: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteProductionCandidateReceiveResult {
    pub(crate) node_id: u32,
    pub(crate) started_after_ms: u64,
    pub(crate) completed_after_ms: u64,
    pub(crate) elapsed_ms: u64,
    pub(crate) candidate_count: u64,
    pub(crate) status: &'static str,
    pub(crate) failure_category: &'static str,
    pub(crate) batch: Option<SpireRemoteSearchCandidateBatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteProductionHeapReceiveRequest {
    pub(crate) node_id: u32,
    pub(crate) conninfo: String,
    pub(crate) remote_index_regclass: String,
    pub(crate) remote_index_identity: Vec<u8>,
    pub(crate) requested_epoch: u64,
    pub(crate) query: Vec<f32>,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) top_k: usize,
    pub(crate) consistency_mode: String,
    pub(crate) tuple_payload_columns: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteProductionHeapReceiveResult {
    pub(crate) node_id: u32,
    pub(crate) started_after_ms: u64,
    pub(crate) completed_after_ms: u64,
    pub(crate) elapsed_ms: u64,
    pub(crate) candidate_count: u64,
    pub(crate) status: &'static str,
    pub(crate) failure_category: &'static str,
    pub(crate) candidates: Vec<SpireRemoteSearchLocalHeapCandidateRow>,
}

struct SpireRemoteProductionTransportAdapter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpireRemoteLocalCancelSource {
    None,
    TestAfterMs(u64),
    PostgresInterruptPoll { poll_interval_ms: u64 },
}

const SPIRE_REMOTE_POSTGRES_INTERRUPT_POLL_MS: u64 = 5;
const SPIRE_SYNC_POSTGRES_CANCEL_NONE: u8 = 0;
const SPIRE_SYNC_POSTGRES_CANCEL_OBSERVED: u8 = 1;

struct SpireSyncPostgresCancelWatcher {
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
    observed: std::sync::Arc<std::sync::atomic::AtomicU8>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl SpireSyncPostgresCancelWatcher {
    fn start(cancel_token: postgres::CancelToken) -> Self {
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let observed = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(
            SPIRE_SYNC_POSTGRES_CANCEL_NONE,
        ));
        let thread_done = std::sync::Arc::clone(&done);
        let thread_observed = std::sync::Arc::clone(&observed);
        let handle = std::thread::Builder::new()
            .name("ec_spire_sync_remote_cancel".to_owned())
            .spawn(move || {
                while !thread_done.load(std::sync::atomic::Ordering::Acquire) {
                    if postgres_query_cancel_pending() {
                        thread_observed.store(
                            SPIRE_SYNC_POSTGRES_CANCEL_OBSERVED,
                            std::sync::atomic::Ordering::Release,
                        );
                        let _ = cancel_token.cancel_query(postgres::NoTls);
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(
                        SPIRE_REMOTE_POSTGRES_INTERRUPT_POLL_MS,
                    ));
                }
            })
            .ok();
        Self {
            done,
            observed,
            handle,
        }
    }

    fn observed_local_cancel(&self) -> bool {
        self.observed.load(std::sync::atomic::Ordering::Acquire)
            != SPIRE_SYNC_POSTGRES_CANCEL_NONE
    }
}

impl Drop for SpireSyncPostgresCancelWatcher {
    fn drop(&mut self) {
        self.done
            .store(true, std::sync::atomic::Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl SpireRemoteLocalCancelSource {
    fn production() -> Self {
        Self::PostgresInterruptPoll {
            poll_interval_ms: SPIRE_REMOTE_POSTGRES_INTERRUPT_POLL_MS,
        }
    }

    fn test_after_ms(delay_ms: u64) -> Self {
        Self::TestAfterMs(delay_ms)
    }
}

impl SpireRemoteProductionTransportAdapter {
    fn run_probe_requests(
        requests: Vec<SpireRemoteProductionTransportProbeRequest>,
    ) -> Result<Vec<SpireRemoteProductionTransportProbeRow>, String> {
        Self::run_probe_requests_with_local_cancel_source(
            requests,
            SpireRemoteLocalCancelSource::production(),
        )
    }

    fn run_probe_requests_with_local_cancel(
        requests: Vec<SpireRemoteProductionTransportProbeRequest>,
        local_cancel_after_ms: Option<u64>,
    ) -> Result<Vec<SpireRemoteProductionTransportProbeRow>, String> {
        let cancel_source = local_cancel_after_ms
            .map(SpireRemoteLocalCancelSource::test_after_ms)
            .unwrap_or(SpireRemoteLocalCancelSource::None);
        Self::run_probe_requests_with_local_cancel_source(requests, cancel_source)
    }

    fn run_probe_requests_with_local_cancel_source(
        requests: Vec<SpireRemoteProductionTransportProbeRequest>,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> Result<Vec<SpireRemoteProductionTransportProbeRow>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|_| {
                "ec_spire production transport adapter failed to build runtime".to_owned()
            })?;

        runtime.block_on(async move {
            let batch_start = std::time::Instant::now();
            let futures = requests.into_iter().map(|request| async move {
                Self::run_one_probe_request(request, batch_start, local_cancel_source).await
            });
            Ok(futures_util::future::join_all(futures).await)
        })
    }

    fn run_candidate_receive_requests(
        requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
    ) -> Result<Vec<SpireRemoteProductionCandidateReceiveResult>, String> {
        Self::run_candidate_receive_requests_with_local_cancel_source(
            requests,
            SpireRemoteLocalCancelSource::production(),
        )
    }

    fn run_candidate_receive_requests_with_local_cancel(
        requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
        local_cancel_after_ms: Option<u64>,
    ) -> Result<Vec<SpireRemoteProductionCandidateReceiveResult>, String> {
        let cancel_source = local_cancel_after_ms
            .map(SpireRemoteLocalCancelSource::test_after_ms)
            .unwrap_or(SpireRemoteLocalCancelSource::None);
        Self::run_candidate_receive_requests_with_local_cancel_source(requests, cancel_source)
    }

    fn run_candidate_receive_requests_with_local_cancel_source(
        requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> Result<Vec<SpireRemoteProductionCandidateReceiveResult>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|_| {
                "ec_spire production transport adapter failed to build runtime".to_owned()
            })?;

        runtime.block_on(async move {
            let batch_start = std::time::Instant::now();
            let futures = requests.into_iter().map(|request| async move {
                Self::run_one_candidate_receive_request(
                    request,
                    batch_start,
                    local_cancel_source,
                )
                .await
            });
            Ok(futures_util::future::join_all(futures).await)
        })
    }

    fn run_heap_receive_requests(
        requests: Vec<SpireRemoteProductionHeapReceiveRequest>,
    ) -> Result<Vec<SpireRemoteProductionHeapReceiveResult>, String> {
        Self::run_heap_receive_requests_with_local_cancel_source(
            requests,
            SpireRemoteLocalCancelSource::production(),
        )
    }

    fn run_heap_receive_requests_with_local_cancel_source(
        requests: Vec<SpireRemoteProductionHeapReceiveRequest>,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> Result<Vec<SpireRemoteProductionHeapReceiveResult>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|_| {
                "ec_spire production heap receive adapter failed to build runtime".to_owned()
            })?;

        runtime.block_on(async move {
            let batch_start = std::time::Instant::now();
            let futures = requests.into_iter().map(|request| async move {
                Self::run_one_heap_receive_request(request, batch_start, local_cancel_source).await
            });
            Ok(futures_util::future::join_all(futures).await)
        })
    }

    async fn run_one_probe_request(
        request: SpireRemoteProductionTransportProbeRequest,
        batch_start: std::time::Instant,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> SpireRemoteProductionTransportProbeRow {
        let started_after_ms = elapsed_millis_u64(batch_start);
        let request_start = std::time::Instant::now();
        let _governance_permit =
            match remote_search_libpq_executor_governance_permit_for_node(request.node_id) {
                Ok(permit) => permit,
                Err(error) => {
                    return failed_production_transport_probe_row(
                        request.node_id,
                        batch_start,
                        request_start,
                        production_governance_failure_category(&error),
                    );
                }
            };
        let mut config = match request.conninfo.parse::<tokio_postgres::Config>() {
            Ok(config) => config,
            Err(_) => {
                return failed_production_transport_probe_row(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNINFO_PARSE_FAILED,
                );
            }
        };
        let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
        if limits.connect_timeout_ms > 0 {
            config.connect_timeout(std::time::Duration::from_millis(limits.connect_timeout_ms));
        }

        let (client, connection) = match config.connect(tokio_postgres::NoTls).await {
            Ok(connection) => connection,
            Err(_) => {
                return failed_production_transport_probe_row(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED,
                );
            }
        };
        let connection_task = tokio::spawn(async move {
            let _ = connection.await;
        });

        let cancel_token = client.cancel_token();
        let query_result = Self::run_query_with_optional_local_cancel(
            cancel_token,
            async {
                if limits.statement_timeout_ms > 0 {
                    client
                        .batch_execute(&format!(
                            "SET statement_timeout = {}",
                            limits.statement_timeout_ms
                        ))
                        .await
                        .map_err(|_| {
                            SPIRE_REMOTE_PRODUCTION_TRANSPORT_STATEMENT_TIMEOUT_SETUP_FAILED
                        })?;
                }
                client
                    .simple_query(request.sql)
                    .await
                    .map_err(|error| production_remote_query_failure_category(&error))
            },
            local_cancel_source,
        )
        .await;

        connection_task.abort();
        let completed_after_ms = elapsed_millis_u64(batch_start);
        let elapsed_ms = elapsed_millis_u64(request_start);
        match query_result {
            Ok(messages) => SpireRemoteProductionTransportProbeRow {
                node_id: request.node_id,
                started_after_ms,
                completed_after_ms,
                elapsed_ms,
                row_count: u64::try_from(messages.len()).unwrap_or(u64::MAX),
                status: SPIRE_REMOTE_STATUS_READY,
                failure_category: SPIRE_REMOTE_NONE,
            },
            Err(failure_category) => failed_production_transport_probe_row(
                request.node_id,
                batch_start,
                request_start,
                failure_category,
            ),
        }
    }

    async fn run_one_candidate_receive_request(
        request: SpireRemoteProductionCandidateReceiveRequest,
        batch_start: std::time::Instant,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> SpireRemoteProductionCandidateReceiveResult {
        let started_after_ms = elapsed_millis_u64(batch_start);
        let request_start = std::time::Instant::now();
        let selected_pids = match request
            .selected_pids
            .iter()
            .map(|pid| i64::try_from(*pid))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(selected_pids) => selected_pids,
            Err(_) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let requested_epoch = match i64::try_from(request.requested_epoch) {
            Ok(requested_epoch) => requested_epoch,
            Err(_) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let top_k = match i32::try_from(request.top_k) {
            Ok(top_k) => top_k,
            Err(_) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let _governance_permit =
            match remote_search_libpq_executor_governance_permit_for_node(request.node_id) {
                Ok(permit) => permit,
                Err(error) => {
                    return failed_production_candidate_receive_result(
                        request.node_id,
                        batch_start,
                        request_start,
                        production_governance_failure_category(&error),
                    );
                }
            };
        let mut config = match request.conninfo.parse::<tokio_postgres::Config>() {
            Ok(config) => config,
            Err(_) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNINFO_PARSE_FAILED,
                );
            }
        };
        let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
        if limits.connect_timeout_ms > 0 {
            config.connect_timeout(std::time::Duration::from_millis(limits.connect_timeout_ms));
        }

        let (client, connection) = match config.connect(tokio_postgres::NoTls).await {
            Ok(connection) => connection,
            Err(_) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED,
                );
            }
        };
        let connection_task = tokio::spawn(async move {
            let _ = connection.await;
        });

        let cancel_token = client.cancel_token();
        let result_rows = Self::run_query_with_optional_local_cancel(
            cancel_token,
            async {
                if limits.statement_timeout_ms > 0 {
                    client
                        .batch_execute(&format!(
                            "SET statement_timeout = {}",
                            limits.statement_timeout_ms
                        ))
                        .await
                        .map_err(|_| {
                            SPIRE_REMOTE_PRODUCTION_TRANSPORT_STATEMENT_TIMEOUT_SETUP_FAILED
                        })?;
                }
                let remote_index_oid = client
                    .query_one(
                        "SELECT to_regclass($1)::oid",
                        &[&request.remote_index_regclass.as_str()],
                    )
                    .await
                    .map_err(|error| {
                        let category = production_remote_query_failure_category(&error);
                        if category == SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED {
                            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE
                        } else {
                            category
                        }
                    })?
                    .try_get::<_, Option<u32>>(0)
                    .map_err(|_| SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?
                    .ok_or(SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?;
                let endpoint_identity_row = client
                    .query_one(
                        SPIRE_REMOTE_SEARCH_ENDPOINT_IDENTITY_SQL_TEMPLATE,
                        &[&remote_index_oid],
                    )
                    .await
                    .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                let endpoint_identity =
                    validate_remote_search_endpoint_identity_row(&endpoint_identity_row)
                        .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                if endpoint_identity.profile_fingerprint_bytes.as_slice()
                    != request.remote_index_identity.as_slice()
                {
                    return Err(SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH);
                }
                client
                    .query(
                        SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
                        &[
                            &remote_index_oid,
                            &requested_epoch,
                            &request.query,
                            &selected_pids,
                            &top_k,
                            &request.consistency_mode,
                        ],
                    )
                    .await
                    .map_err(|error| production_remote_query_failure_category(&error))
            },
            local_cancel_source,
        )
        .await;

        connection_task.abort();
        let result_rows = match result_rows {
            Ok(result_rows) => result_rows,
            Err(failure_category) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    failure_category,
                );
            }
        };
        let candidates = match result_rows
            .iter()
            .map(|candidate_row| {
                decode_remote_search_candidate_pg_row(
                    candidate_row,
                    request.node_id,
                    true,
                    Some(&request.remote_index_identity),
                )
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(candidates) => candidates,
            Err(error) => {
                return failed_production_candidate_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    production_candidate_decode_failure_category(&error),
                );
            }
        };
        if let Err(error) = validate_remote_search_candidate_batch(
            request.requested_epoch,
            request.node_id,
            &request.selected_pids,
            &candidates,
        ) {
            return failed_production_candidate_receive_result(
                request.node_id,
                batch_start,
                request_start,
                production_candidate_validation_failure_category(&error),
            );
        }
        let candidate_count = u64::try_from(candidates.len()).unwrap_or(u64::MAX);
        SpireRemoteProductionCandidateReceiveResult {
            node_id: request.node_id,
            started_after_ms,
            completed_after_ms: elapsed_millis_u64(batch_start),
            elapsed_ms: elapsed_millis_u64(request_start),
            candidate_count,
            status: SPIRE_REMOTE_STATUS_READY,
            failure_category: SPIRE_REMOTE_NONE,
            batch: Some(SpireRemoteSearchCandidateBatch {
                node_id: request.node_id,
                selected_pids: request.selected_pids,
                candidates,
            }),
        }
    }

    async fn run_one_heap_receive_request(
        request: SpireRemoteProductionHeapReceiveRequest,
        batch_start: std::time::Instant,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> SpireRemoteProductionHeapReceiveResult {
        let started_after_ms = elapsed_millis_u64(batch_start);
        let request_start = std::time::Instant::now();
        let selected_pids = match request
            .selected_pids
            .iter()
            .map(|pid| i64::try_from(*pid))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(selected_pids) => selected_pids,
            Err(_) => {
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let requested_epoch = match i64::try_from(request.requested_epoch) {
            Ok(requested_epoch) => requested_epoch,
            Err(_) => {
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let top_k = match i32::try_from(request.top_k) {
            Ok(top_k) => top_k,
            Err(_) => {
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_CANDIDATE_INVALID_PARAMETERS,
                );
            }
        };
        let _governance_permit =
            match remote_search_libpq_executor_governance_permit_for_node(request.node_id) {
                Ok(permit) => permit,
                Err(error) => {
                    return failed_production_heap_receive_result(
                        request.node_id,
                        batch_start,
                        request_start,
                        production_governance_failure_category(&error),
                    );
                }
            };
        let mut config = match request.conninfo.parse::<tokio_postgres::Config>() {
            Ok(config) => config,
            Err(_) => {
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNINFO_PARSE_FAILED,
                );
            }
        };
        let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
        if limits.connect_timeout_ms > 0 {
            config.connect_timeout(std::time::Duration::from_millis(limits.connect_timeout_ms));
        }

        let (client, connection) = match config.connect(tokio_postgres::NoTls).await {
            Ok(connection) => connection,
            Err(_) => {
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED,
                );
            }
        };
        let connection_task = tokio::spawn(async move {
            let _ = connection.await;
        });

        let cancel_token = client.cancel_token();
        let result_rows = Self::run_query_with_optional_local_cancel(
            cancel_token,
            async {
                if limits.statement_timeout_ms > 0 {
                    client
                        .batch_execute(&format!(
                            "SET statement_timeout = {}",
                            limits.statement_timeout_ms
                        ))
                        .await
                        .map_err(|_| {
                            SPIRE_REMOTE_PRODUCTION_TRANSPORT_STATEMENT_TIMEOUT_SETUP_FAILED
                        })?;
                }
                let remote_index_oid = client
                    .query_one(
                        "SELECT to_regclass($1)::oid",
                        &[&request.remote_index_regclass.as_str()],
                    )
                    .await
                    .map_err(|error| {
                        let category = production_remote_query_failure_category(&error);
                        if category == SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED {
                            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE
                        } else {
                            category
                        }
                    })?
                    .try_get::<_, Option<u32>>(0)
                    .map_err(|_| SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?
                    .ok_or(SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE)?;
                let endpoint_identity_row = client
                    .query_one(
                        SPIRE_REMOTE_SEARCH_ENDPOINT_IDENTITY_SQL_TEMPLATE,
                        &[&remote_index_oid],
                    )
                    .await
                    .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                let endpoint_identity =
                    validate_remote_search_endpoint_identity_row(&endpoint_identity_row)
                        .map_err(|_| SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)?;
                if endpoint_identity.profile_fingerprint_bytes.as_slice()
                    != request.remote_index_identity.as_slice()
                {
                    return Err(SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH);
                }
                match request.tuple_payload_columns.as_ref() {
                    Some(tuple_payload_columns) => {
                        let sql = if endpoint_identity.prefers_typed_tuple_transport() {
                            SPIRE_REMOTE_SEARCH_LIBPQ_TYPED_TUPLE_PAYLOAD_SQL_TEMPLATE
                        } else {
                            SPIRE_REMOTE_SEARCH_LIBPQ_TUPLE_PAYLOAD_SQL_TEMPLATE
                        };
                        client
                            .query(
                                sql,
                                &[
                                    &remote_index_oid,
                                    &requested_epoch,
                                    &request.query,
                                    &selected_pids,
                                    &top_k,
                                    &request.consistency_mode,
                                    tuple_payload_columns,
                                ],
                            )
                            .await
                            .map_err(|error| production_remote_query_failure_category(&error))
                    }
                    None => client
                        .query(
                            SPIRE_REMOTE_SEARCH_LIBPQ_HEAP_SQL_TEMPLATE,
                            &[
                                &remote_index_oid,
                                &requested_epoch,
                                &request.query,
                                &selected_pids,
                                &top_k,
                                &request.consistency_mode,
                            ],
                        )
                        .await
                        .map_err(|error| production_remote_query_failure_category(&error)),
                }
            },
            local_cancel_source,
        )
        .await;

        connection_task.abort();
        let result_rows = match result_rows {
            Ok(result_rows) => result_rows,
            Err(failure_category) => {
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    failure_category,
                );
            }
        };
        let candidates = match result_rows
            .iter()
            .map(|candidate_row| {
                decode_remote_search_heap_candidate_pg_row(
                    candidate_row,
                    request.requested_epoch,
                    request.node_id,
                )
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(candidates) => candidates,
            Err(error) => {
                pgrx::warning!("ec_spire remote heap receive decode failed: {error}");
                return failed_production_heap_receive_result(
                    request.node_id,
                    batch_start,
                    request_start,
                    production_remote_heap_decode_failure_category(&error),
                );
            }
        };
        let merge_candidates = candidates
            .iter()
            .map(|candidate| SpireRemoteSearchCandidateRow {
                served_epoch: candidate.served_epoch,
                node_id: candidate.node_id,
                pid: candidate.pid,
                object_version: candidate.object_version,
                row_index: candidate.row_index,
                assignment_flags: candidate.assignment_flags,
                vec_id: candidate.vec_id.clone(),
                row_locator: candidate.row_locator.clone(),
                score: candidate.score,
            })
            .collect::<Vec<_>>();
        if let Err(error) = validate_remote_search_candidate_batch(
            request.requested_epoch,
            request.node_id,
            &request.selected_pids,
            &merge_candidates,
        ) {
            return failed_production_heap_receive_result(
                request.node_id,
                batch_start,
                request_start,
                production_remote_heap_decode_failure_category(&error),
            );
        }
        let candidate_count = u64::try_from(candidates.len()).unwrap_or(u64::MAX);
        SpireRemoteProductionHeapReceiveResult {
            node_id: request.node_id,
            started_after_ms,
            completed_after_ms: elapsed_millis_u64(batch_start),
            elapsed_ms: elapsed_millis_u64(request_start),
            candidate_count,
            status: SPIRE_REMOTE_STATUS_READY,
            failure_category: SPIRE_REMOTE_NONE,
            candidates,
        }
    }

    async fn run_query_with_optional_local_cancel<T, F>(
        cancel_token: tokio_postgres::CancelToken,
        query_future: F,
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> Result<T, &'static str>
    where
        F: std::future::Future<Output = Result<T, &'static str>>,
    {
        if local_cancel_source == SpireRemoteLocalCancelSource::None {
            return query_future.await;
        }
        let cancel_signal = Self::local_cancel_signal(local_cancel_source);
        match futures_util::future::select(Box::pin(query_future), Box::pin(cancel_signal)).await {
            futures_util::future::Either::Left((query_result, _)) => query_result,
            futures_util::future::Either::Right((failure_category, _query_future)) => {
                let _ = cancel_token.cancel_query(tokio_postgres::NoTls).await;
                Err(failure_category)
            }
        }
    }

    async fn local_cancel_signal(
        local_cancel_source: SpireRemoteLocalCancelSource,
    ) -> &'static str {
        match local_cancel_source {
            SpireRemoteLocalCancelSource::None => std::future::pending::<&'static str>().await,
            SpireRemoteLocalCancelSource::TestAfterMs(delay_ms) => {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED
            }
            SpireRemoteLocalCancelSource::PostgresInterruptPoll { poll_interval_ms } => {
                let poll_interval_ms = poll_interval_ms.max(1);
                loop {
                    if postgres_query_cancel_pending() {
                        return postgres_local_cancel_failure_category();
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(poll_interval_ms)).await;
                }
            }
        }
    }
}

unsafe extern "C" {
    fn dlsym(
        handle: *mut std::ffi::c_void,
        symbol: *const std::ffi::c_char,
    ) -> *mut std::ffi::c_void;
}

fn postgres_sig_atomic_flag(symbol_name: &'static [u8]) -> i32 {
    let ptr = unsafe { dlsym(std::ptr::null_mut(), symbol_name.as_ptr().cast()) };
    if ptr.is_null() {
        return 0;
    }
    unsafe { *(ptr.cast::<std::ffi::c_int>()) }
}

fn postgres_query_cancel_pending() -> bool {
    postgres_sig_atomic_flag(b"InterruptPending\0") != 0
        && postgres_sig_atomic_flag(b"QueryCancelPending\0") != 0
}

const POSTGRES_STATEMENT_TIMEOUT_ID: std::ffi::c_int = 3;

type PostgresGetTimeoutIndicator =
    unsafe extern "C" fn(std::ffi::c_int, bool) -> bool;

fn postgres_statement_timeout_pending() -> bool {
    let ptr = unsafe { dlsym(std::ptr::null_mut(), b"get_timeout_indicator\0".as_ptr().cast()) };
    if ptr.is_null() {
        return false;
    }
    let get_timeout_indicator: PostgresGetTimeoutIndicator = unsafe { std::mem::transmute(ptr) };
    unsafe { get_timeout_indicator(POSTGRES_STATEMENT_TIMEOUT_ID, false) }
}

fn postgres_local_cancel_failure_category() -> &'static str {
    if postgres_statement_timeout_pending() {
        SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT
    } else {
        SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED
    }
}

fn is_local_cancellation_failure_category(failure_category: &str) -> bool {
    failure_category == SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED
        || failure_category == SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT
}

fn coordinator_remote_local_cancel_error(
    operation: &str,
    node_id: u32,
    failure_category: &str,
) -> String {
    format!(
        "ec_spire coordinator {operation} remote prepare cancelled for node_id {node_id}: {failure_category}"
    )
}

fn production_remote_query_failure_category(error: &tokio_postgres::Error) -> &'static str {
    // `tokio-postgres` reports a backend terminated during an already-open
    // query as a closed connection, while pre-query connection failures are
    // classified earlier as connect failures.
    if error.is_closed() {
        return SPIRE_REMOTE_PRODUCTION_REMOTE_BACKEND_TERMINATED;
    }
    let Some(db_error) = error.as_db_error() else {
        return SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED;
    };
    match db_error.code().code() {
        // PostgreSQL uses SQLSTATE 57014 for query_canceled in general. The
        // statement-timeout message text is the stable PostgreSQL convention
        // that lets operators distinguish timeout remediation from cancellation
        // provenance.
        "57014" if db_error.message().contains("statement timeout") => {
            SPIRE_REMOTE_PRODUCTION_REMOTE_STATEMENT_TIMEOUT
        }
        "57014" => SPIRE_REMOTE_PRODUCTION_REMOTE_QUERY_CANCELLED,
        "57P01" | "57P02" | "57P03" => SPIRE_REMOTE_PRODUCTION_REMOTE_BACKEND_TERMINATED,
        _ => SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED,
    }
}

fn production_governance_failure_category(_error: &str) -> &'static str {
    SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD
}

fn production_candidate_decode_failure_category(error: &str) -> &'static str {
    let status = remote_search_receive_attempt_failure_status(error);
    if status == SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH
        || status == "protocol_version_mismatch"
        || status == "extension_version_mismatch"
    {
        SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH
    } else {
        SPIRE_REMOTE_PRODUCTION_CANDIDATE_DECODE_FAILED
    }
}

fn production_candidate_validation_failure_category(error: &str) -> &'static str {
    if remote_search_receive_attempt_failure_status(error)
        == SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH
    {
        SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH
    } else {
        SPIRE_REMOTE_PRODUCTION_CANDIDATE_VALIDATION_FAILED
    }
}

fn production_remote_heap_decode_failure_category(error: &str) -> &'static str {
    if error.contains(SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_MISSING) {
        SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_MISSING
    } else if error.contains(SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_DEAD) {
        SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_DEAD
    } else if error.contains(SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_STALE) {
        SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_STALE
    } else if remote_search_receive_attempt_failure_status(error)
        == SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH
    {
        SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH
    } else {
        SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED
    }
}

fn failed_production_transport_probe_row(
    node_id: u32,
    batch_start: std::time::Instant,
    request_start: std::time::Instant,
    failure_category: &'static str,
) -> SpireRemoteProductionTransportProbeRow {
    SpireRemoteProductionTransportProbeRow {
        node_id,
        started_after_ms: elapsed_millis_u64(batch_start),
        completed_after_ms: elapsed_millis_u64(batch_start),
        elapsed_ms: elapsed_millis_u64(request_start),
        row_count: 0,
        status: SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
        failure_category,
    }
}

fn elapsed_millis_u64(start: std::time::Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn failed_production_candidate_receive_result(
    node_id: u32,
    batch_start: std::time::Instant,
    request_start: std::time::Instant,
    failure_category: &'static str,
) -> SpireRemoteProductionCandidateReceiveResult {
    SpireRemoteProductionCandidateReceiveResult {
        node_id,
        started_after_ms: elapsed_millis_u64(batch_start),
        completed_after_ms: elapsed_millis_u64(batch_start),
        elapsed_ms: elapsed_millis_u64(request_start),
        candidate_count: 0,
        status: SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
        failure_category,
        batch: None,
    }
}

fn failed_production_heap_receive_result(
    node_id: u32,
    batch_start: std::time::Instant,
    request_start: std::time::Instant,
    failure_category: &'static str,
) -> SpireRemoteProductionHeapReceiveResult {
    SpireRemoteProductionHeapReceiveResult {
        node_id,
        started_after_ms: elapsed_millis_u64(batch_start),
        completed_after_ms: elapsed_millis_u64(batch_start),
        elapsed_ms: elapsed_millis_u64(request_start),
        candidate_count: 0,
        status: SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED,
        failure_category,
        candidates: Vec::new(),
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_transport_probe_for_test(
    requests: Vec<SpireRemoteProductionTransportProbeRequest>,
) -> Vec<SpireRemoteProductionTransportProbeRow> {
    SpireRemoteProductionTransportAdapter::run_probe_requests(requests)
        .unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_transport_probe_summary_for_test(
    requests: Vec<SpireRemoteProductionTransportProbeRequest>,
    consistency_mode: &str,
) -> SpireRemoteProductionExecutorStateSummaryRow {
    let result = (|| -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
        let consistency_mode_name =
            consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        let dispatch_rows = requests
            .iter()
            .map(|request| SpireRemoteSearchLibpqDispatchPlanRow {
                requested_epoch: 1,
                node_id: request.node_id,
                selected_pids: vec![u64::from(request.node_id)],
                pid_count: 1,
                query_dimension: 2,
                top_k: 1,
                consistency_mode: consistency_mode_name,
                sql_template: "SELECT 1",
                parameter_count: 0,
                result_column_count: 1,
                conninfo_secret_name: format!("tests/node/{}", request.node_id),
                remote_index_regclass: "tests.ec_spire_transport_probe_idx".to_owned(),
                descriptor_generation: 1,
                remote_index_identity: Vec::new(),
                pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
                dispatch_action: SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION,
                receive_validator: "test_transport_probe",
                status: SPIRE_REMOTE_STATUS_READY,
            })
            .collect::<Vec<_>>();
        let transport_rows = SpireRemoteProductionTransportAdapter::run_probe_requests(requests)?;
        remote_search_production_executor_state_summary_from_transport_probe_rows_with_consistency_mode(
            1,
            &dispatch_rows,
            &transport_rows,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_transport_probe_with_local_cancel_for_test(
    requests: Vec<SpireRemoteProductionTransportProbeRequest>,
    local_cancel_after_ms: u64,
) -> Vec<SpireRemoteProductionTransportProbeRow> {
    SpireRemoteProductionTransportAdapter::run_probe_requests_with_local_cancel(
        requests,
        Some(local_cancel_after_ms),
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_transport_probe_with_local_cancel_summary_for_test(
    requests: Vec<SpireRemoteProductionTransportProbeRequest>,
    local_cancel_after_ms: u64,
    consistency_mode: &str,
) -> SpireRemoteProductionExecutorStateSummaryRow {
    let result = (|| -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
        let consistency_mode_name =
            consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        let dispatch_rows = requests
            .iter()
            .map(|request| SpireRemoteSearchLibpqDispatchPlanRow {
                requested_epoch: 1,
                node_id: request.node_id,
                selected_pids: vec![u64::from(request.node_id)],
                pid_count: 1,
                query_dimension: 2,
                top_k: 1,
                consistency_mode: consistency_mode_name,
                sql_template: "SELECT 1",
                parameter_count: 0,
                result_column_count: 1,
                conninfo_secret_name: format!("tests/node/{}", request.node_id),
                remote_index_regclass: "tests.ec_spire_transport_probe_idx".to_owned(),
                descriptor_generation: 1,
                remote_index_identity: Vec::new(),
                pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
                dispatch_action: SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION,
                receive_validator: "test_transport_probe",
                status: SPIRE_REMOTE_STATUS_READY,
            })
            .collect::<Vec<_>>();
        let transport_rows =
            SpireRemoteProductionTransportAdapter::run_probe_requests_with_local_cancel(
                requests,
                Some(local_cancel_after_ms),
            )?;
        remote_search_production_executor_state_summary_from_transport_probe_rows_with_consistency_mode(
            1,
            &dispatch_rows,
            &transport_rows,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_candidate_receive_for_test(
    requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
) -> Vec<SpireRemoteProductionCandidateReceiveResult> {
    SpireRemoteProductionTransportAdapter::run_candidate_receive_requests(requests)
        .unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_candidate_receive_summary_for_test(
    requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
    consistency_mode: &str,
) -> SpireRemoteProductionExecutorStateSummaryRow {
    let result = (|| -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
        let consistency_mode_name =
            consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        let requested_epoch = requests
            .first()
            .map(|request| request.requested_epoch)
            .unwrap_or(1);
        let dispatch_rows = requests
            .iter()
            .map(|request| SpireRemoteSearchLibpqDispatchPlanRow {
                requested_epoch,
                node_id: request.node_id,
                selected_pids: request.selected_pids.clone(),
                pid_count: u64::try_from(request.selected_pids.len()).unwrap_or(u64::MAX),
                query_dimension: u64::try_from(request.query.len()).unwrap_or(u64::MAX),
                top_k: u64::try_from(request.top_k).unwrap_or(u64::MAX),
                consistency_mode: consistency_mode_name,
                sql_template: SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
                parameter_count: 6,
                result_column_count: 11,
                conninfo_secret_name: format!("tests/node/{}", request.node_id),
                remote_index_regclass: request.remote_index_regclass.clone(),
                descriptor_generation: 1,
                remote_index_identity: request.remote_index_identity.clone(),
                pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
                dispatch_action: SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION,
                receive_validator: "test_candidate_receive",
                status: SPIRE_REMOTE_STATUS_READY,
            })
            .collect::<Vec<_>>();
        let transport_rows = requests
            .iter()
            .map(|request| SpireRemoteProductionTransportProbeRow {
                node_id: request.node_id,
                started_after_ms: 0,
                completed_after_ms: 0,
                elapsed_ms: 0,
                row_count: 1,
                status: SPIRE_REMOTE_STATUS_READY,
                failure_category: SPIRE_REMOTE_NONE,
            })
            .collect::<Vec<_>>();
        let receive_results =
            SpireRemoteProductionTransportAdapter::run_candidate_receive_requests(requests)?;
        remote_search_production_executor_state_summary_from_candidate_receive_results_with_consistency_mode(
            requested_epoch,
            &dispatch_rows,
            &transport_rows,
            &receive_results,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_production_candidate_receive_with_local_cancel_for_test(
    requests: Vec<SpireRemoteProductionCandidateReceiveRequest>,
    local_cancel_after_ms: u64,
) -> Vec<SpireRemoteProductionCandidateReceiveResult> {
    SpireRemoteProductionTransportAdapter::run_candidate_receive_requests_with_local_cancel(
        requests,
        Some(local_cancel_after_ms),
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpireRemoteProductionDispatchState {
    Planned,
    BlockedBeforeDispatch,
    TransportReady,
    TransportFailed,
    CandidateReceiveReady,
    CandidateReceiveFailed,
    RemoteHeapReady,
    RemoteHeapFailed,
    DegradedSkipped,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireRemoteProductionDispatch {
    node_id: u32,
    selected_pids: Vec<u64>,
    pid_count: u64,
    conninfo_secret_name: String,
    remote_index_regclass: String,
    remote_index_identity: Vec<u8>,
    state: SpireRemoteProductionDispatchState,
    status: &'static str,
    next_executor_step: &'static str,
    transport_row_count: u64,
    transport_failure_category: &'static str,
    candidate_count: u64,
    candidate_failure_category: &'static str,
    degraded_skip_category: &'static str,
    candidate_batch: Option<SpireRemoteSearchCandidateBatch>,
    remote_heap_candidate_count: u64,
    remote_heap_failure_category: &'static str,
    remote_heap_candidates: Vec<SpireRemoteSearchLocalHeapCandidateRow>,
}

impl SpireRemoteProductionDispatch {
    fn from_libpq_dispatch(row: &SpireRemoteSearchLibpqDispatchPlanRow) -> Self {
        if row.dispatch_action == SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
            Self {
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                conninfo_secret_name: row.conninfo_secret_name.clone(),
                remote_index_regclass: row.remote_index_regclass.clone(),
                remote_index_identity: row.remote_index_identity.clone(),
                state: SpireRemoteProductionDispatchState::Planned,
                status: SPIRE_REMOTE_STATUS_REQUIRES_PRODUCTION_TRANSPORT,
                next_executor_step: SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
                transport_row_count: 0,
                transport_failure_category: SPIRE_REMOTE_NONE,
                candidate_count: 0,
                candidate_failure_category: SPIRE_REMOTE_NONE,
                degraded_skip_category: SPIRE_REMOTE_NONE,
                candidate_batch: None,
                remote_heap_candidate_count: 0,
                remote_heap_failure_category: SPIRE_REMOTE_NONE,
                remote_heap_candidates: Vec::new(),
            }
        } else {
            Self {
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                conninfo_secret_name: row.conninfo_secret_name.clone(),
                remote_index_regclass: row.remote_index_regclass.clone(),
                remote_index_identity: row.remote_index_identity.clone(),
                state: SpireRemoteProductionDispatchState::BlockedBeforeDispatch,
                status: row.status,
                next_executor_step: remote_search_pre_dispatch_blocker_step(row.status),
                transport_row_count: 0,
                transport_failure_category: SPIRE_REMOTE_NONE,
                candidate_count: 0,
                candidate_failure_category: SPIRE_REMOTE_NONE,
                degraded_skip_category: SPIRE_REMOTE_NONE,
                candidate_batch: None,
                remote_heap_candidate_count: 0,
                remote_heap_failure_category: SPIRE_REMOTE_NONE,
                remote_heap_candidates: Vec::new(),
            }
        }
    }

    fn apply_transport_probe_row(
        &mut self,
        row: &SpireRemoteProductionTransportProbeRow,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::Planned {
            return Err(format!(
                "ec_spire production executor transport outcome for node_id {} cannot apply to dispatch state {:?}",
                row.node_id, self.state
            ));
        }

        self.transport_row_count = row.row_count;
        self.transport_failure_category = row.failure_category;
        if row.status == SPIRE_REMOTE_STATUS_READY {
            self.state = SpireRemoteProductionDispatchState::TransportReady;
            self.status = SPIRE_REMOTE_STATUS_REQUIRES_COMPACT_CANDIDATE_RECEIVE;
            self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE;
        } else {
            self.state = SpireRemoteProductionDispatchState::TransportFailed;
            self.status = row.status;
            self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT;
        }
        Ok(())
    }

    fn apply_transport_degraded_skip(
        &mut self,
        row: &SpireRemoteProductionTransportProbeRow,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::Planned {
            return Err(format!(
                "ec_spire production executor transport outcome for node_id {} cannot apply to dispatch state {:?}",
                row.node_id, self.state
            ));
        }

        self.transport_row_count = row.row_count;
        self.transport_failure_category = row.failure_category;
        self.apply_degraded_skip(row.failure_category);
        Ok(())
    }

    fn apply_candidate_receive_result(
        &mut self,
        result: &SpireRemoteProductionCandidateReceiveResult,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::TransportReady {
            return Err(format!(
                "ec_spire production executor candidate receive outcome for node_id {} cannot apply to dispatch state {:?}",
                result.node_id, self.state
            ));
        }

        self.candidate_failure_category = result.failure_category;
        if result.status == SPIRE_REMOTE_STATUS_READY {
            let Some(batch) = result.batch.as_ref() else {
                return Err(format!(
                    "ec_spire production executor candidate receive outcome for node_id {} is ready without a candidate batch",
                    result.node_id
                ));
            };
            let batch_candidate_count = u64::try_from(batch.candidates.len()).map_err(|_| {
                "ec_spire production executor candidate receive batch count exceeds u64"
                    .to_owned()
            })?;
            if result.candidate_count != batch_candidate_count {
                return Err(format!(
                    "ec_spire production executor candidate receive outcome for node_id {} reports {} candidates but batch contains {}",
                    result.node_id, result.candidate_count, batch_candidate_count
                ));
            }
            self.candidate_count = result.candidate_count;
            self.candidate_batch = Some(batch.clone());
            self.remote_heap_candidate_count = 0;
            self.remote_heap_failure_category = SPIRE_REMOTE_NONE;
            self.remote_heap_candidates.clear();
            self.state = SpireRemoteProductionDispatchState::CandidateReceiveReady;
            self.status = SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP;
            self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION;
        } else {
            self.candidate_count = 0;
            self.candidate_batch = None;
            self.remote_heap_candidate_count = 0;
            self.remote_heap_failure_category = SPIRE_REMOTE_NONE;
            self.remote_heap_candidates.clear();
            self.state = SpireRemoteProductionDispatchState::CandidateReceiveFailed;
            self.status = SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED;
            self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE;
        }
        Ok(())
    }

    fn apply_candidate_receive_degraded_skip(
        &mut self,
        result: &SpireRemoteProductionCandidateReceiveResult,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::TransportReady {
            return Err(format!(
                "ec_spire production executor candidate receive outcome for node_id {} cannot apply to dispatch state {:?}",
                result.node_id, self.state
            ));
        }

        self.candidate_failure_category = result.failure_category;
        self.apply_degraded_skip(result.failure_category);
        Ok(())
    }

    fn apply_candidate_receive_failure(&mut self, failure_category: &'static str) {
        self.candidate_count = 0;
        self.candidate_failure_category = failure_category;
        self.candidate_batch = None;
        self.remote_heap_candidate_count = 0;
        self.remote_heap_failure_category = SPIRE_REMOTE_NONE;
        self.remote_heap_candidates.clear();
        self.state = SpireRemoteProductionDispatchState::CandidateReceiveFailed;
        self.status = SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED;
        self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE;
    }

    fn apply_remote_heap_receive_result(
        &mut self,
        result: &SpireRemoteProductionHeapReceiveResult,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::CandidateReceiveReady {
            return Err(format!(
                "ec_spire production executor remote heap outcome for node_id {} cannot apply to dispatch state {:?}",
                result.node_id, self.state
            ));
        }

        self.remote_heap_failure_category = result.failure_category;
        if result.status == SPIRE_REMOTE_STATUS_READY {
            let candidate_count = u64::try_from(result.candidates.len()).map_err(|_| {
                "ec_spire production executor remote heap batch count exceeds u64".to_owned()
            })?;
            if result.candidate_count != candidate_count {
                return Err(format!(
                    "ec_spire production executor remote heap outcome for node_id {} reports {} candidates but batch contains {}",
                    result.node_id, result.candidate_count, candidate_count
                ));
            }
            self.remote_heap_candidate_count = result.candidate_count;
            self.remote_heap_candidates = result.candidates.clone();
            self.state = SpireRemoteProductionDispatchState::RemoteHeapReady;
            self.status = SPIRE_REMOTE_STATUS_READY;
            self.next_executor_step = SPIRE_REMOTE_NONE;
        } else {
            self.remote_heap_candidate_count = 0;
            self.remote_heap_candidates.clear();
            self.state = SpireRemoteProductionDispatchState::RemoteHeapFailed;
            self.status = SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED;
            self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION;
        }
        Ok(())
    }

    fn apply_remote_heap_receive_degraded_skip(
        &mut self,
        result: &SpireRemoteProductionHeapReceiveResult,
    ) -> Result<(), String> {
        if self.state != SpireRemoteProductionDispatchState::CandidateReceiveReady {
            return Err(format!(
                "ec_spire production executor remote heap outcome for node_id {} cannot apply to dispatch state {:?}",
                result.node_id, self.state
            ));
        }

        self.remote_heap_failure_category = result.failure_category;
        self.apply_degraded_skip(result.failure_category);
        Ok(())
    }

    fn apply_degraded_skip(&mut self, failure_category: &'static str) {
        self.candidate_count = 0;
        self.degraded_skip_category = failure_category;
        self.candidate_batch = None;
        self.remote_heap_candidate_count = 0;
        self.remote_heap_failure_category = SPIRE_REMOTE_NONE;
        self.remote_heap_candidates.clear();
        self.state = SpireRemoteProductionDispatchState::DegradedSkipped;
        self.status = SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED;
        self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION;
    }

    fn apply_local_query_cancel(&mut self, failure_category: &'static str) {
        self.candidate_count = 0;
        self.candidate_failure_category = failure_category;
        self.degraded_skip_category = SPIRE_REMOTE_NONE;
        self.candidate_batch = None;
        self.remote_heap_candidate_count = 0;
        self.remote_heap_failure_category = SPIRE_REMOTE_NONE;
        self.remote_heap_candidates.clear();
        self.state = SpireRemoteProductionDispatchState::Cancelled;
        self.status = SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED;
        self.next_executor_step = SPIRE_REMOTE_EXECUTOR_STEP_CANCELLATION;
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SpireRemoteFanoutExecutor {
    requested_epoch: u64,
    dispatches: Vec<SpireRemoteProductionDispatch>,
    conninfo_secret_lookup_count: u64,
    socket_open_count: u64,
    endpoint_identity_query_count: u64,
}

impl SpireRemoteFanoutExecutor {
    fn from_libpq_dispatch_rows(
        requested_epoch: u64,
        rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    ) -> Self {
        Self {
            requested_epoch,
            dispatches: rows
                .iter()
                .map(SpireRemoteProductionDispatch::from_libpq_dispatch)
                .collect(),
            conninfo_secret_lookup_count: 0,
            socket_open_count: 0,
            endpoint_identity_query_count: 0,
        }
    }

    fn apply_transport_probe_rows(
        &mut self,
        rows: &[SpireRemoteProductionTransportProbeRow],
    ) -> Result<(), String> {
        self.apply_transport_probe_rows_with_consistency_mode(rows, "strict")
    }

    fn apply_transport_probe_rows_with_consistency_mode(
        &mut self,
        rows: &[SpireRemoteProductionTransportProbeRow],
        consistency_mode: &str,
    ) -> Result<(), String> {
        let degraded =
            parse_remote_search_consistency_mode(consistency_mode)? == meta::SpireConsistencyMode::Degraded;
        if let Some(cancelled_row) = rows
            .iter()
            .find(|row| is_local_cancellation_failure_category(row.failure_category))
        {
            if !self.dispatches.iter().any(|dispatch| {
                dispatch.node_id == cancelled_row.node_id
                    && dispatch.state == SpireRemoteProductionDispatchState::Planned
            }) {
                return Err(format!(
                    "ec_spire production executor transport outcome for node_id {} does not match a planned dispatch",
                    cancelled_row.node_id
                ));
            }
            self.apply_local_query_cancel(cancelled_row.failure_category);
            return Ok(());
        }
        for row in rows {
            let dispatch = self
                .dispatches
                .iter_mut()
                .find(|dispatch| {
                    dispatch.node_id == row.node_id
                        && dispatch.state == SpireRemoteProductionDispatchState::Planned
                })
                .ok_or_else(|| {
                    format!(
                        "ec_spire production executor transport outcome for node_id {} does not match a planned dispatch",
                        row.node_id
                    )
                })?;
            if degraded && row.status != SPIRE_REMOTE_STATUS_READY {
                dispatch.apply_transport_degraded_skip(row)?;
            } else {
                dispatch.apply_transport_probe_row(row)?;
            }
        }
        Ok(())
    }

    fn apply_candidate_receive_results(
        &mut self,
        results: &[SpireRemoteProductionCandidateReceiveResult],
    ) -> Result<(), String> {
        self.apply_candidate_receive_results_with_consistency_mode(results, "strict")
    }

    fn apply_candidate_receive_results_with_consistency_mode(
        &mut self,
        results: &[SpireRemoteProductionCandidateReceiveResult],
        consistency_mode: &str,
    ) -> Result<(), String> {
        let degraded =
            parse_remote_search_consistency_mode(consistency_mode)? == meta::SpireConsistencyMode::Degraded;
        if let Some(cancelled_result) = results
            .iter()
            .find(|result| is_local_cancellation_failure_category(result.failure_category))
        {
            if !self.dispatches.iter().any(|dispatch| {
                dispatch.node_id == cancelled_result.node_id
                    && dispatch.state == SpireRemoteProductionDispatchState::TransportReady
            }) {
                return Err(format!(
                    "ec_spire production executor candidate receive outcome for node_id {} does not match a transport-ready dispatch",
                    cancelled_result.node_id
                ));
            }
            self.apply_local_query_cancel(cancelled_result.failure_category);
            return Ok(());
        }
        for result in results {
            let dispatch = self
                .dispatches
                .iter_mut()
                .find(|dispatch| {
                    dispatch.node_id == result.node_id
                        && dispatch.state == SpireRemoteProductionDispatchState::TransportReady
                })
                .ok_or_else(|| {
                    format!(
                        "ec_spire production executor candidate receive outcome for node_id {} does not match a transport-ready dispatch",
                        result.node_id
                    )
                })?;
            if degraded && result.status != SPIRE_REMOTE_STATUS_READY {
                dispatch.apply_candidate_receive_degraded_skip(result)?;
            } else {
                dispatch.apply_candidate_receive_result(result)?;
            }
        }
        Ok(())
    }

    fn apply_remote_heap_receive_results_with_consistency_mode(
        &mut self,
        results: &[SpireRemoteProductionHeapReceiveResult],
        consistency_mode: &str,
    ) -> Result<(), String> {
        let degraded =
            parse_remote_search_consistency_mode(consistency_mode)? == meta::SpireConsistencyMode::Degraded;
        if let Some(cancelled_result) = results
            .iter()
            .find(|result| is_local_cancellation_failure_category(result.failure_category))
        {
            if !self.dispatches.iter().any(|dispatch| {
                dispatch.node_id == cancelled_result.node_id
                    && dispatch.state == SpireRemoteProductionDispatchState::CandidateReceiveReady
            }) {
                return Err(format!(
                    "ec_spire production executor remote heap outcome for node_id {} does not match a candidate-ready dispatch",
                    cancelled_result.node_id
                ));
            }
            self.apply_local_query_cancel(cancelled_result.failure_category);
            return Ok(());
        }
        for result in results {
            let dispatch = self
                .dispatches
                .iter_mut()
                .find(|dispatch| {
                    dispatch.node_id == result.node_id
                        && dispatch.state == SpireRemoteProductionDispatchState::CandidateReceiveReady
                })
                .ok_or_else(|| {
                    format!(
                        "ec_spire production executor remote heap outcome for node_id {} does not match a candidate-ready dispatch",
                        result.node_id
                    )
                })?;
            if degraded && result.status != SPIRE_REMOTE_STATUS_READY {
                dispatch.apply_remote_heap_receive_degraded_skip(result)?;
            } else {
                dispatch.apply_remote_heap_receive_result(result)?;
            }
        }
        Ok(())
    }

    fn apply_local_query_cancel(&mut self, failure_category: &'static str) {
        for dispatch in &mut self.dispatches {
            dispatch.apply_local_query_cancel(failure_category);
        }
    }

    fn apply_blocked_before_dispatch_degraded_skips(&mut self) {
        for dispatch in &mut self.dispatches {
            if dispatch.state == SpireRemoteProductionDispatchState::BlockedBeforeDispatch {
                dispatch.apply_degraded_skip(dispatch.status);
            }
        }
    }

    fn mark_planned_dispatches_candidate_receive_ready(&mut self) {
        for dispatch in &mut self.dispatches {
            if dispatch.state == SpireRemoteProductionDispatchState::Planned {
                dispatch.state = SpireRemoteProductionDispatchState::TransportReady;
                dispatch.status = SPIRE_REMOTE_STATUS_REQUIRES_COMPACT_CANDIDATE_RECEIVE;
                dispatch.next_executor_step =
                    SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE;
            }
        }
    }

    fn compact_candidate_receive_requests(
        &mut self,
        query: &[f32],
        top_k: usize,
        consistency_mode: &str,
    ) -> Result<Vec<SpireRemoteProductionCandidateReceiveRequest>, String> {
        let degraded =
            parse_remote_search_consistency_mode(consistency_mode)? == meta::SpireConsistencyMode::Degraded;
        let mut requests = Vec::new();
        let mut secret_lookup_count = self.conninfo_secret_lookup_count;
        for dispatch in &mut self.dispatches {
            if dispatch.state != SpireRemoteProductionDispatchState::TransportReady {
                continue;
            }
            add_remote_count(
                &mut secret_lookup_count,
                1,
                "remote production executor compact receive request build",
                "conninfo secret lookup",
            )?;
            match remote_conninfo_secret_value(&dispatch.conninfo_secret_name) {
                Ok(conninfo) => requests.push(SpireRemoteProductionCandidateReceiveRequest {
                    node_id: dispatch.node_id,
                    conninfo,
                    remote_index_regclass: dispatch.remote_index_regclass.clone(),
                    remote_index_identity: dispatch.remote_index_identity.clone(),
                    requested_epoch: self.requested_epoch,
                    query: query.to_vec(),
                    selected_pids: dispatch.selected_pids.clone(),
                    top_k,
                    consistency_mode: consistency_mode.to_owned(),
                }),
                Err(_) if degraded => dispatch.apply_degraded_skip(SPIRE_REMOTE_STATUS_REQUIRES_SECRET),
                Err(_) => dispatch.apply_candidate_receive_failure(SPIRE_REMOTE_STATUS_REQUIRES_SECRET),
            }
        }
        self.conninfo_secret_lookup_count = secret_lookup_count;
        Ok(requests)
    }

    fn run_compact_candidate_receive(
        &mut self,
        query: &[f32],
        top_k: usize,
        consistency_mode: &str,
    ) -> Result<(), String> {
        let requests = self.compact_candidate_receive_requests(query, top_k, consistency_mode)?;
        if requests.is_empty() {
            return Ok(());
        }
        let results =
            SpireRemoteProductionTransportAdapter::run_candidate_receive_requests(requests)?;
        self.apply_candidate_receive_results_with_consistency_mode(&results, consistency_mode)
    }

    fn remote_heap_receive_requests(
        &mut self,
        query: &[f32],
        top_k: usize,
        consistency_mode: &str,
        tuple_payload_columns: Option<&[String]>,
    ) -> Result<Vec<SpireRemoteProductionHeapReceiveRequest>, String> {
        let degraded =
            parse_remote_search_consistency_mode(consistency_mode)? == meta::SpireConsistencyMode::Degraded;
        let mut requests = Vec::new();
        let mut secret_lookup_count = self.conninfo_secret_lookup_count;
        for dispatch in &mut self.dispatches {
            if dispatch.state != SpireRemoteProductionDispatchState::CandidateReceiveReady {
                continue;
            }
            add_remote_count(
                &mut secret_lookup_count,
                1,
                "remote production executor heap receive request build",
                "conninfo secret lookup",
            )?;
            match remote_conninfo_secret_value(&dispatch.conninfo_secret_name) {
                Ok(conninfo) => requests.push(SpireRemoteProductionHeapReceiveRequest {
                    node_id: dispatch.node_id,
                    conninfo,
                    remote_index_regclass: dispatch.remote_index_regclass.clone(),
                    remote_index_identity: dispatch.remote_index_identity.clone(),
                    requested_epoch: self.requested_epoch,
                    query: query.to_vec(),
                    selected_pids: dispatch.selected_pids.clone(),
                    top_k,
                    consistency_mode: consistency_mode.to_owned(),
                    tuple_payload_columns: tuple_payload_columns.map(<[String]>::to_vec),
                }),
                Err(_) if degraded => dispatch.apply_degraded_skip(SPIRE_REMOTE_STATUS_REQUIRES_SECRET),
                Err(_) => {
                    let now = std::time::Instant::now();
                    let result = failed_production_heap_receive_result(
                        dispatch.node_id,
                        now,
                        now,
                        SPIRE_REMOTE_STATUS_REQUIRES_SECRET,
                    );
                    dispatch.apply_remote_heap_receive_result(&result)?;
                }
            }
        }
        self.conninfo_secret_lookup_count = secret_lookup_count;
        Ok(requests)
    }

    fn run_remote_heap_receive(
        &mut self,
        query: &[f32],
        top_k: usize,
        consistency_mode: &str,
        tuple_payload_columns: Option<&[String]>,
    ) -> Result<(), String> {
        let requests =
            self.remote_heap_receive_requests(query, top_k, consistency_mode, tuple_payload_columns)?;
        if requests.is_empty() {
            return Ok(());
        }
        let results = SpireRemoteProductionTransportAdapter::run_heap_receive_requests(requests)?;
        self.apply_remote_heap_receive_results_with_consistency_mode(&results, consistency_mode)
    }

    fn ready_candidate_batches(&self) -> Result<Vec<SpireRemoteSearchCandidateBatch>, String> {
        let mut batches = Vec::new();
        for dispatch in &self.dispatches {
            match dispatch.state {
                SpireRemoteProductionDispatchState::CandidateReceiveReady => {
                    let batch = dispatch.candidate_batch.clone().ok_or_else(|| {
                        format!(
                            "ec_spire production executor candidate receive ready dispatch for node_id {} is missing its candidate batch",
                            dispatch.node_id
                        )
                    })?;
                    batches.push(batch);
                }
                SpireRemoteProductionDispatchState::BlockedBeforeDispatch
                | SpireRemoteProductionDispatchState::Planned
                | SpireRemoteProductionDispatchState::TransportReady
                | SpireRemoteProductionDispatchState::TransportFailed
                | SpireRemoteProductionDispatchState::CandidateReceiveFailed
                | SpireRemoteProductionDispatchState::RemoteHeapReady
                | SpireRemoteProductionDispatchState::RemoteHeapFailed
                | SpireRemoteProductionDispatchState::Cancelled => {
                    return Err(format!(
                        "ec_spire production executor cannot merge compact candidates while node_id {} is in state {:?} with status {}",
                        dispatch.node_id, dispatch.state, dispatch.status
                    ));
                }
                SpireRemoteProductionDispatchState::DegradedSkipped => {}
            }
        }
        Ok(batches)
    }

    fn merge_ready_candidate_batches(
        &self,
        limit: Option<usize>,
    ) -> Result<SpireRemoteSearchMergeResult, String> {
        merge_validated_remote_search_candidate_batches(
            self.requested_epoch,
            self.ready_candidate_batches()?,
            limit,
        )
    }

    fn ready_remote_heap_candidate_rows(
        &self,
    ) -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
        let mut candidates = Vec::new();
        for dispatch in &self.dispatches {
            match dispatch.state {
                SpireRemoteProductionDispatchState::RemoteHeapReady => {
                    candidates.extend(dispatch.remote_heap_candidates.clone());
                }
                SpireRemoteProductionDispatchState::DegradedSkipped => {}
                SpireRemoteProductionDispatchState::BlockedBeforeDispatch
                | SpireRemoteProductionDispatchState::Planned
                | SpireRemoteProductionDispatchState::TransportReady
                | SpireRemoteProductionDispatchState::TransportFailed
                | SpireRemoteProductionDispatchState::CandidateReceiveReady
                | SpireRemoteProductionDispatchState::CandidateReceiveFailed
                | SpireRemoteProductionDispatchState::RemoteHeapFailed
                | SpireRemoteProductionDispatchState::Cancelled => {
                    return Err(format!(
                        "ec_spire production executor cannot merge remote heap candidates while node_id {} is in state {:?} with status {}",
                        dispatch.node_id, dispatch.state, dispatch.status
                    ));
                }
            }
        }
        Ok(candidates)
    }

    fn remote_heap_resolution_counts(&self) -> Result<(u64, u64, u64), String> {
        let mut ready_dispatch_count = 0_u64;
        let mut failed_dispatch_count = 0_u64;
        let mut candidate_count = 0_u64;
        for dispatch in &self.dispatches {
            match dispatch.state {
                SpireRemoteProductionDispatchState::RemoteHeapReady => {
                    add_remote_count(
                        &mut ready_dispatch_count,
                        1,
                        "remote production executor heap resolution counts",
                        "ready dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_count,
                        dispatch.remote_heap_candidate_count,
                        "remote production executor heap resolution counts",
                        "remote heap candidate",
                    )?;
                }
                SpireRemoteProductionDispatchState::RemoteHeapFailed => {
                    add_remote_count(
                        &mut failed_dispatch_count,
                        1,
                        "remote production executor heap resolution counts",
                        "failed dispatch",
                    )?;
                }
                _ => {}
            }
        }
        Ok((ready_dispatch_count, failed_dispatch_count, candidate_count))
    }

    fn degraded_skip_report(
        &self,
    ) -> Result<Vec<SpireRemoteProductionDegradedSkipReportRow>, String> {
        let mut rows = Vec::new();
        for dispatch in &self.dispatches {
            if dispatch.state != SpireRemoteProductionDispatchState::DegradedSkipped {
                continue;
            }
            rows.push(SpireRemoteProductionDegradedSkipReportRow {
                requested_epoch: self.requested_epoch,
                node_id: dispatch.node_id,
                skipped_pid_count: dispatch.pid_count,
                first_skip_category: dispatch.degraded_skip_category,
                status: dispatch.status,
            });
        }
        Ok(rows)
    }

    fn summary(
        &self,
        consistency_mode_source: &'static str,
        consistency_mode: &'static str,
    ) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
        let mut planned_dispatch_count = 0_u64;
        let mut blocked_before_dispatch_count = 0_u64;
        let mut remote_pid_count = 0_u64;
        let mut planned_pid_count = 0_u64;
        let mut blocked_pid_count = 0_u64;
        let mut transport_pending_dispatch_count = 0_u64;
        let mut transport_sent_dispatch_count = 0_u64;
        let mut transport_ready_dispatch_count = 0_u64;
        let mut transport_failed_dispatch_count = 0_u64;
        let mut transport_row_count = 0_u64;
        let mut first_transport_failure_category = SPIRE_REMOTE_NONE;
        let mut candidate_receive_pending_dispatch_count = 0_u64;
        let mut candidate_receive_sent_dispatch_count = 0_u64;
        let mut candidate_receive_ready_dispatch_count = 0_u64;
        let mut candidate_receive_failed_dispatch_count = 0_u64;
        let mut candidate_row_count = 0_u64;
        let mut first_candidate_receive_failure_category = SPIRE_REMOTE_NONE;
        let mut degraded_skipped_dispatch_count = 0_u64;
        let mut first_degraded_skip_category = SPIRE_REMOTE_NONE;
        let mut cancelled_dispatch_count = 0_u64;
        let mut first_cancellation_category = SPIRE_REMOTE_NONE;
        let mut first_blocked_status = SPIRE_REMOTE_STATUS_READY;
        let mut first_blocked_step = SPIRE_REMOTE_NONE;
        let mut remote_heap_ready_dispatch_count = 0_u64;
        let mut remote_heap_failed_dispatch_count = 0_u64;

        for dispatch in &self.dispatches {
            add_remote_count(
                &mut remote_pid_count,
                dispatch.pid_count,
                "remote production executor state summary",
                "remote PID",
            )?;
            match dispatch.state {
                SpireRemoteProductionDispatchState::Planned => {
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_pending_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-pending dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::BlockedBeforeDispatch => {
                    if blocked_before_dispatch_count == 0 {
                        first_blocked_status = dispatch.status;
                        first_blocked_step = dispatch.next_executor_step;
                    }
                    add_remote_count(
                        &mut blocked_before_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "blocked dispatch",
                    )?;
                    add_remote_count(
                        &mut blocked_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "blocked PID",
                    )?;
                }
                SpireRemoteProductionDispatchState::TransportReady => {
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_row_count,
                        dispatch.transport_row_count,
                        "remote production executor state summary",
                        "transport row",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_pending_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-pending dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::TransportFailed => {
                    if transport_failed_dispatch_count == 0 {
                        first_transport_failure_category = dispatch.transport_failure_category;
                    }
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_failed_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-failed dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::CandidateReceiveReady => {
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_row_count,
                        dispatch.transport_row_count,
                        "remote production executor state summary",
                        "transport row",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_row_count,
                        dispatch.candidate_count,
                        "remote production executor state summary",
                        "candidate row",
                    )?;
                }
                SpireRemoteProductionDispatchState::CandidateReceiveFailed => {
                    if candidate_receive_failed_dispatch_count == 0 {
                        first_candidate_receive_failure_category =
                            dispatch.candidate_failure_category;
                    }
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_row_count,
                        dispatch.transport_row_count,
                        "remote production executor state summary",
                        "transport row",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_failed_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-failed dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::RemoteHeapReady => {
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_row_count,
                        dispatch.transport_row_count,
                        "remote production executor state summary",
                        "transport row",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_row_count,
                        dispatch.candidate_count,
                        "remote production executor state summary",
                        "candidate row",
                    )?;
                    add_remote_count(
                        &mut remote_heap_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "remote-heap-ready dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::RemoteHeapFailed => {
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut transport_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "transport-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut transport_row_count,
                        dispatch.transport_row_count,
                        "remote production executor state summary",
                        "transport row",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_sent_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-sent dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_receive_ready_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "candidate-receive-ready dispatch",
                    )?;
                    add_remote_count(
                        &mut candidate_row_count,
                        dispatch.candidate_count,
                        "remote production executor state summary",
                        "candidate row",
                    )?;
                    add_remote_count(
                        &mut remote_heap_failed_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "remote-heap-failed dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::DegradedSkipped => {
                    if degraded_skipped_dispatch_count == 0 {
                        first_degraded_skip_category = dispatch.degraded_skip_category;
                    }
                    add_remote_count(
                        &mut planned_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "planned dispatch",
                    )?;
                    add_remote_count(
                        &mut planned_pid_count,
                        dispatch.pid_count,
                        "remote production executor state summary",
                        "planned PID",
                    )?;
                    add_remote_count(
                        &mut degraded_skipped_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "degraded-skipped dispatch",
                    )?;
                }
                SpireRemoteProductionDispatchState::Cancelled => {
                    if cancelled_dispatch_count == 0 {
                        first_cancellation_category = dispatch.candidate_failure_category;
                    }
                    add_remote_count(
                        &mut cancelled_dispatch_count,
                        1,
                        "remote production executor state summary",
                        "cancelled dispatch",
                    )?;
                }
            }
        }

        let dispatch_count = u64::try_from(self.dispatches.len()).map_err(|_| {
            "ec_spire remote production executor dispatch count exceeds u64".to_owned()
        })?;
        let (next_executor_step, status, recommendation) = if cancelled_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_CANCELLATION,
                SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED,
                "release governance and discard retained remote batches after local cancellation",
            )
        } else if blocked_before_dispatch_count > 0 {
            (
                first_blocked_step,
                first_blocked_status,
                remote_search_pre_dispatch_blocker_recommendation(first_blocked_status),
            )
        } else if transport_failed_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
                SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
                "inspect production transport failure category before compact candidate receive",
            )
        } else if transport_pending_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
                SPIRE_REMOTE_STATUS_REQUIRES_PRODUCTION_TRANSPORT,
                "implement production async or libpq pipeline transport before remote fanout execution",
            )
        } else if candidate_receive_failed_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
                SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
                "inspect production candidate receive failure category before merge",
            )
        } else if candidate_receive_pending_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
                SPIRE_REMOTE_STATUS_REQUIRES_COMPACT_CANDIDATE_RECEIVE,
                "wire production compact candidate receive before AM scan merge",
            )
        } else if remote_heap_failed_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
                SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED,
                "inspect production remote heap failure category before final row delivery",
            )
        } else if remote_heap_ready_dispatch_count > 0 && degraded_skipped_dispatch_count > 0 {
            (
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_STATUS_DEGRADED_READY,
                "return ready heap-resolved rows and report degraded skipped dispatches",
            )
        } else if remote_heap_ready_dispatch_count > 0 {
            (SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
        } else if candidate_receive_ready_dispatch_count > 0 && degraded_skipped_dispatch_count > 0
        {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
                SPIRE_REMOTE_STATUS_DEGRADED_READY,
                "continue with ready remote batches and report degraded skipped dispatches",
            )
        } else if degraded_skipped_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
                SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
                "continue without skipped remote dispatches in degraded mode",
            )
        } else if transport_ready_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
                SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
                "wire origin-node remote heap resolution before returning SQL rows",
            )
        } else {
            (SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
        };

        Ok(SpireRemoteProductionExecutorStateSummaryRow {
            requested_epoch: self.requested_epoch,
            state_model: SPIRE_REMOTE_PRODUCTION_STATE_MODEL,
            transport_mode: SPIRE_REMOTE_PRODUCTION_TRANSPORT_PENDING,
            consistency_mode_source,
            consistency_mode,
            dispatch_count,
            planned_dispatch_count,
            blocked_before_dispatch_count,
            remote_pid_count,
            planned_pid_count,
            blocked_pid_count,
            conninfo_secret_lookup_count: self.conninfo_secret_lookup_count,
            socket_open_count: self.socket_open_count,
            endpoint_identity_query_count: self.endpoint_identity_query_count,
            transport_pending_dispatch_count,
            transport_sent_dispatch_count,
            transport_ready_dispatch_count,
            transport_failed_dispatch_count,
            transport_row_count,
            first_transport_failure_category,
            candidate_receive_pending_dispatch_count,
            candidate_receive_sent_dispatch_count,
            candidate_receive_ready_dispatch_count,
            candidate_receive_failed_dispatch_count,
            candidate_row_count,
            first_candidate_receive_failure_category,
            degraded_skipped_dispatch_count,
            first_degraded_skip_category,
            cancelled_dispatch_count,
            first_cancellation_category,
            next_executor_step,
            status,
            recommendation,
        })
    }
}

fn remote_search_production_executor_state_summary_from_dispatch_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    consistency_mode_source: &'static str,
    consistency_mode: &str,
) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
    let parsed_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
    let consistency_mode = consistency_mode_name(parsed_consistency_mode);
    let mut executor = SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(requested_epoch, rows);
    if parsed_consistency_mode == meta::SpireConsistencyMode::Degraded {
        executor.apply_blocked_before_dispatch_degraded_skips();
    }
    executor.summary(consistency_mode_source, consistency_mode)
}

fn remote_search_production_degraded_skip_report_from_dispatch_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    consistency_mode: &str,
) -> Result<Vec<SpireRemoteProductionDegradedSkipReportRow>, String> {
    let parsed_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
    let mut executor = SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(requested_epoch, rows);
    if parsed_consistency_mode == meta::SpireConsistencyMode::Degraded {
        executor.apply_blocked_before_dispatch_degraded_skips();
    }
    executor.degraded_skip_report()
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_executor_state_summary_from_transport_probe_rows(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
    remote_search_production_executor_state_summary_from_transport_probe_rows_with_consistency_mode(
        requested_epoch,
        dispatch_rows,
        transport_rows,
        "strict",
    )
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_executor_state_summary_from_transport_probe_rows_with_consistency_mode(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
    consistency_mode: &str,
) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
    let mut executor =
        SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(requested_epoch, dispatch_rows);
    executor.apply_transport_probe_rows_with_consistency_mode(transport_rows, consistency_mode)?;
    executor.summary(
        "function_argument",
        consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?),
    )
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_executor_state_summary_from_candidate_receive_results(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
    candidate_receive_results: &[SpireRemoteProductionCandidateReceiveResult],
) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
    remote_search_production_executor_state_summary_from_candidate_receive_results_with_consistency_mode(
        requested_epoch,
        dispatch_rows,
        transport_rows,
        candidate_receive_results,
        "strict",
    )
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_executor_state_summary_from_candidate_receive_results_with_consistency_mode(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
    candidate_receive_results: &[SpireRemoteProductionCandidateReceiveResult],
    consistency_mode: &str,
) -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
    let mut executor =
        SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(requested_epoch, dispatch_rows);
    executor.apply_transport_probe_rows_with_consistency_mode(transport_rows, consistency_mode)?;
    executor.apply_candidate_receive_results_with_consistency_mode(
        candidate_receive_results,
        consistency_mode,
    )?;
    executor.summary(
        "function_argument",
        consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?),
    )
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_compact_merge_from_candidate_receive_results(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
    candidate_receive_results: &[SpireRemoteProductionCandidateReceiveResult],
    limit: Option<usize>,
) -> Result<SpireRemoteSearchMergeResult, String> {
    remote_search_production_compact_merge_from_candidate_receive_results_with_consistency_mode(
        requested_epoch,
        dispatch_rows,
        transport_rows,
        candidate_receive_results,
        limit,
        "strict",
    )
}

#[cfg(any(test, feature = "pg_test"))]
fn remote_search_production_compact_merge_from_candidate_receive_results_with_consistency_mode(
    requested_epoch: u64,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    transport_rows: &[SpireRemoteProductionTransportProbeRow],
    candidate_receive_results: &[SpireRemoteProductionCandidateReceiveResult],
    limit: Option<usize>,
    consistency_mode: &str,
) -> Result<SpireRemoteSearchMergeResult, String> {
    let mut executor =
        SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(requested_epoch, dispatch_rows);
    executor.apply_transport_probe_rows_with_consistency_mode(transport_rows, consistency_mode)?;
    executor.apply_candidate_receive_results_with_consistency_mode(
        candidate_receive_results,
        consistency_mode,
    )?;
    executor.merge_ready_candidate_batches(limit)
}

pub(crate) unsafe fn remote_search_production_consistency_policy_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    consistency_mode_source: &'static str,
    consistency_mode: &str,
) -> SpireRemoteProductionConsistencyPolicySummaryRow {
    let result = (|| -> Result<SpireRemoteProductionConsistencyPolicySummaryRow, String> {
        if requested_epoch == 0 {
            return Err(
                "ec_spire remote search production consistency policy requested_epoch must be greater than 0"
                    .to_owned(),
            );
        }
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let requested_consistency_mode = consistency_mode_name(requested_consistency_mode);
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let (epoch_manifest, _, _) =
            unsafe { load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)? };
        let active_consistency_mode = consistency_mode_name(epoch_manifest.consistency_mode);

        let (
            status,
            failure_category,
            failure_action,
            recommendation,
        ) = if root_control.active_epoch != requested_epoch {
            (
                SPIRE_REMOTE_PRODUCTION_REQUESTED_EPOCH_MISMATCH,
                SPIRE_REMOTE_PRODUCTION_REQUESTED_EPOCH_MISMATCH,
                "fail_closed",
                "request the active epoch before planning production remote fanout",
            )
        } else if active_consistency_mode != requested_consistency_mode {
            (
                SPIRE_REMOTE_STATUS_CONSISTENCY_MODE_MISMATCH,
                SPIRE_REMOTE_STATUS_CONSISTENCY_MODE_MISMATCH,
                "fail_closed",
                "publish a degraded-capable epoch or run the query with the active epoch policy",
            )
        } else {
            (
                SPIRE_REMOTE_STATUS_READY,
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
                "consistency policy is ready for production dispatch planning",
            )
        };

        Ok(SpireRemoteProductionConsistencyPolicySummaryRow {
            requested_epoch,
            active_epoch: root_control.active_epoch,
            consistency_mode_source,
            requested_consistency_mode,
            active_consistency_mode,
            status,
            failure_category,
            failure_action,
            next_executor_step: if status == SPIRE_REMOTE_STATUS_READY {
                SPIRE_REMOTE_EXECUTOR_STEP_BUDGET
            } else {
                SPIRE_REMOTE_EXECUTOR_STEP_CONSISTENCY_POLICY
            },
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_production_session_consistency_policy_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
) -> SpireRemoteProductionConsistencyPolicySummaryRow {
    let consistency_mode = options::current_session_remote_search_consistency_mode_name();
    unsafe {
        remote_search_production_consistency_policy_summary_row(
            index_relation,
            requested_epoch,
            "ec_spire.remote_search_consistency_mode",
            consistency_mode,
        )
    }
}

pub(crate) fn remote_search_production_fault_matrix_rows(
) -> Vec<SpireRemoteProductionFaultMatrixRow> {
    vec![
        production_fault_matrix_row(
            1,
            SPIRE_REMOTE_STATUS_CONSISTENCY_MODE_MISMATCH,
            "consistency_policy",
            SPIRE_REMOTE_EXECUTOR_STEP_CONSISTENCY_POLICY,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CONSISTENCY_MODE_MISMATCH,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CONSISTENCY_MODE_MISMATCH,
            "do not dispatch when the requested consistency mode differs from the published active epoch policy",
        ),
        production_fault_matrix_row(
            2,
            SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            "executor_governance",
            SPIRE_REMOTE_EXECUTOR_STEP_GOVERNANCE,
            "fail_closed",
            SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "release saturated global or per-node governance slots before strict search; degraded mode may skip only that node",
        ),
        production_fault_matrix_row(
            3,
            SPIRE_REMOTE_STATUS_REQUIRES_SECRET,
            "conninfo_secret",
            SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "resolve the conninfo secret before strict search; degraded mode may skip only that node",
        ),
        production_fault_matrix_row(
            4,
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNINFO_PARSE_FAILED,
            "transport",
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "fix sanitized conninfo syntax or skip the affected remote under degraded mode",
        ),
        production_fault_matrix_row(
            5,
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED,
            "transport",
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "treat connect, authentication, certificate, and connect-timeout failures as sanitized transport failures",
        ),
        production_fault_matrix_row(
            6,
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_STATEMENT_TIMEOUT_SETUP_FAILED,
            "transport",
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "do not run an uncapped remote query when remote statement_timeout setup fails",
        ),
        production_fault_matrix_row(
            7,
            SPIRE_REMOTE_PRODUCTION_REMOTE_STATEMENT_TIMEOUT,
            "transport",
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "remote-owned statement timeout is a remote-node failure, not a local cancellation",
        ),
        production_fault_matrix_row(
            8,
            SPIRE_REMOTE_PRODUCTION_REMOTE_BACKEND_TERMINATED,
            "transport",
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "backend termination or connection reset must not be merged as an empty successful result",
        ),
        production_fault_matrix_row(
            9,
            SPIRE_REMOTE_PRODUCTION_REMOTE_QUERY_CANCELLED,
            "transport",
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "remote query cancellation is remote-owned unless the local adapter reports a local cancellation category",
        ),
        production_fault_matrix_row(
            10,
            SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED,
            "local_cancellation",
            SPIRE_REMOTE_EXECUTOR_STEP_CANCELLATION,
            "cancel_query",
            SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED,
            "cancel_query",
            SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED,
            "local query cancellation is query-wide and clears all retained candidate batches in every consistency mode",
        ),
        production_fault_matrix_row(
            11,
            SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT,
            "local_cancellation",
            SPIRE_REMOTE_EXECUTOR_STEP_CANCELLATION,
            "cancel_query",
            SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED,
            "cancel_query",
            SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED,
            "local statement timeout is query-wide and distinct from remote_statement_timeout",
        ),
        production_fault_matrix_row(
            12,
            SPIRE_REMOTE_PRODUCTION_CANDIDATE_DECODE_FAILED,
            "candidate_receive",
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "malformed compact candidate rows cannot enter merge in strict mode",
        ),
        production_fault_matrix_row(
            13,
            SPIRE_REMOTE_PRODUCTION_CANDIDATE_VALIDATION_FAILED,
            "candidate_receive",
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "candidate batch validation failures preserve the exact category before merge",
        ),
        production_fault_matrix_row(
            14,
            SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH,
            "endpoint_identity",
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "descriptor, index, quantizer, opclass, storage, and fingerprint mismatches are identity failures",
        ),
        production_fault_matrix_row(
            15,
            SPIRE_REMOTE_PRODUCTION_PROTOCOL_VERSION_MISMATCH,
            "endpoint_identity",
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "protocol version skew must be rejected before candidate merge",
        ),
        production_fault_matrix_row(
            16,
            SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION,
            "descriptor_version",
            SPIRE_REMOTE_EXECUTOR_STEP_EXTENSION_VERSION,
            "fail_closed",
            SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "descriptor-advertised extension skew blocks strict dispatch planning",
        ),
        production_fault_matrix_row(
            17,
            SPIRE_REMOTE_PRODUCTION_EXTENSION_VERSION_MISMATCH,
            "endpoint_identity",
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "live endpoint extension skew invalidates cached endpoint identity",
        ),
        production_fault_matrix_row(
            18,
            SPIRE_REMOTE_STATUS_STALE_EPOCH,
            "epoch_window",
            SPIRE_REMOTE_EXECUTOR_STEP_EPOCH_WINDOW,
            "fail_closed",
            SPIRE_REMOTE_STATUS_STALE_EPOCH,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "stale descriptor epoch cannot satisfy strict fanout and is a named degraded skip",
        ),
        production_fault_matrix_row(
            19,
            SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH,
            "candidate_receive",
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "served epoch mismatches are rejected after receive instead of folded into generic validation",
        ),
        production_fault_matrix_row(
            20,
            SPIRE_REMOTE_PRODUCTION_REQUESTED_EPOCH_MISMATCH,
            "consistency_policy",
            SPIRE_REMOTE_EXECUTOR_STEP_CONSISTENCY_POLICY,
            "fail_closed",
            SPIRE_REMOTE_PRODUCTION_REQUESTED_EPOCH_MISMATCH,
            "fail_closed",
            SPIRE_REMOTE_PRODUCTION_REQUESTED_EPOCH_MISMATCH,
            "requested epoch mismatch is a coordinator request error, not a degraded remote skip",
        ),
        production_fault_matrix_row(
            21,
            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE,
            "endpoint_identity",
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "missing remote index or regclass resolution failure cannot be treated as an empty batch",
        ),
        production_fault_matrix_row(
            22,
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED,
            "remote_heap_resolution",
            SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
            "fail_closed",
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "remote heap query failure blocks final SQL rows unless degraded mode skips that origin node",
        ),
        production_fault_matrix_row(
            23,
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_MISSING,
            "remote_heap_resolution",
            SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
            "fail_closed",
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_MISSING,
            "skip_candidate",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "missing origin heap rows must be counted and either fail strict or be explicitly skipped",
        ),
        production_fault_matrix_row(
            24,
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_DEAD,
            "remote_heap_resolution",
            SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
            "fail_closed",
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_DEAD,
            "skip_candidate",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "dead origin heap rows must be counted and must not be returned as visible SQL rows",
        ),
        production_fault_matrix_row(
            25,
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_STALE,
            "remote_heap_resolution",
            SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
            "fail_closed",
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_STALE,
            "skip_candidate",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "stale origin row locators must be rejected or explicitly skipped with diagnostics",
        ),
        production_fault_matrix_row(
            26,
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED,
            "transport",
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "uncategorized remote query failures, including remote OOM, cannot enter merge as empty results",
        ),
    ]
}

fn production_fault_matrix_row(
    fault_ordinal: u64,
    failure_category: &'static str,
    fault_scope: &'static str,
    next_executor_step: &'static str,
    strict_action: &'static str,
    strict_status: &'static str,
    degraded_action: &'static str,
    degraded_status: &'static str,
    recommendation: &'static str,
) -> SpireRemoteProductionFaultMatrixRow {
    SpireRemoteProductionFaultMatrixRow {
        fault_ordinal,
        failure_category,
        fault_scope,
        next_executor_step,
        strict_action,
        strict_status,
        degraded_action,
        degraded_status,
        recommendation,
    }
}

pub(crate) fn remote_search_stage_e_fault_matrix_rows() -> Vec<SpireRemoteStageEFaultMatrixRow> {
    vec![
        stage_e_fault_matrix_row(
            1,
            "epoch_mismatch",
            "remote_epoch_window",
            SPIRE_REMOTE_STATUS_STALE_EPOCH,
            SPIRE_REMOTE_EXECUTOR_STEP_EPOCH_WINDOW,
            "fail_closed",
            SPIRE_REMOTE_STATUS_STALE_EPOCH,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "strict_failed_dispatch_count+1; degraded_skipped_dispatch_count+1",
            "one coordinator plus two remotes; stale remote advertises served epoch outside requested retained window",
        ),
        stage_e_fault_matrix_row(
            2,
            "version_skew",
            "remote_extension_version",
            SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION,
            SPIRE_REMOTE_EXECUTOR_STEP_EXTENSION_VERSION,
            "fail_closed",
            SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "strict_failed_dispatch_count+1; degraded_skipped_dispatch_count+1",
            "remote descriptor advertises incompatible extension version before dispatch opens sockets",
        ),
        stage_e_fault_matrix_row(
            3,
            "fingerprint_mismatch",
            "endpoint_identity",
            SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH,
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "identity_cache_invalidations+1; degraded_skipped_dispatch_count+1",
            "live endpoint fingerprint differs from descriptor remote_index_identity",
        ),
        stage_e_fault_matrix_row(
            4,
            "connection_reset_mid_batch",
            "transport",
            SPIRE_REMOTE_PRODUCTION_REMOTE_BACKEND_TERMINATED,
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "transport_failed_dispatch_count+1; degraded_skipped_dispatch_count+1",
            "remote connection closes while an in-flight production request is receiving rows",
        ),
        stage_e_fault_matrix_row(
            5,
            "remote_backend_termination",
            "transport",
            SPIRE_REMOTE_PRODUCTION_REMOTE_BACKEND_TERMINATED,
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "transport_failed_dispatch_count+1; degraded_skipped_dispatch_count+1",
            "remote backend is terminated before it can return a validated batch",
        ),
        stage_e_fault_matrix_row(
            6,
            "remote_statement_timeout",
            "transport",
            SPIRE_REMOTE_PRODUCTION_REMOTE_STATEMENT_TIMEOUT,
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "remote_statement_timeout_count+1; degraded_skipped_dispatch_count+1",
            "remote statement_timeout fires on one remote while other ready remotes can still complete",
        ),
        stage_e_fault_matrix_row(
            7,
            "local_statement_timeout",
            "local_cancellation",
            SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT,
            SPIRE_REMOTE_EXECUTOR_STEP_CANCELLATION,
            "cancel_query",
            SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED,
            "cancel_query",
            SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED,
            "cancelled_dispatch_count=fanout; retained_candidate_batch_count=0",
            "coordinator statement_timeout cancels every in-flight remote and releases governance permits",
        ),
        stage_e_fault_matrix_row(
            8,
            "local_cancel",
            "local_cancellation",
            SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED,
            SPIRE_REMOTE_EXECUTOR_STEP_CANCELLATION,
            "cancel_query",
            SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED,
            "cancel_query",
            SPIRE_REMOTE_STATUS_EXECUTOR_CANCELLED,
            "cancelled_dispatch_count=fanout; retained_candidate_batch_count=0",
            "client query cancel interrupts every in-flight remote and releases governance permits",
        ),
        stage_e_fault_matrix_row(
            9,
            "simulated_network_partition",
            "transport",
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED,
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "connect_failed_count+1; degraded_skipped_dispatch_count+1",
            "one remote conninfo is unreachable while at least one other remote remains ready",
        ),
        stage_e_fault_matrix_row(
            10,
            "remote_oom",
            "transport",
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED,
            SPIRE_REMOTE_EXECUTOR_STEP_PRODUCTION_TRANSPORT,
            "fail_closed",
            SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "remote_query_failed_count+1; degraded_skipped_dispatch_count+1",
            "remote returns a sanitized out-of-memory query failure without leaking raw remote error text",
        ),
        stage_e_fault_matrix_row(
            11,
            "missing_or_reindexed_remote_index",
            "endpoint_identity",
            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE,
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            "remote_index_unavailable_count+1; degraded_skipped_dispatch_count+1",
            "remote index is dropped, renamed, or rebuilt after planning but before receive",
        ),
    ]
}

fn stage_e_fault_matrix_row(
    fault_ordinal: u64,
    fault_case: &'static str,
    fixture_scope: &'static str,
    failure_category: &'static str,
    next_executor_step: &'static str,
    strict_action: &'static str,
    strict_status: &'static str,
    degraded_action: &'static str,
    degraded_status: &'static str,
    counter_delta: &'static str,
    required_evidence: &'static str,
) -> SpireRemoteStageEFaultMatrixRow {
    SpireRemoteStageEFaultMatrixRow {
        fault_ordinal,
        fault_case,
        fixture_scope,
        failure_category,
        next_executor_step,
        strict_action,
        strict_status,
        degraded_action,
        degraded_status,
        counter_delta,
        required_evidence,
    }
}

pub(crate) fn remote_search_stage_e_lifecycle_matrix_rows(
) -> Vec<SpireRemoteStageELifecycleMatrixRow> {
    vec![
        stage_e_lifecycle_matrix_row(
            1,
            "drop_remote_index_before_fanout",
            "DROP INDEX",
            "before_fanout_planning",
            "remote_index_regclass",
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE,
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "planner or receive path resolves remote_index_regclass and records remote_index_unavailable instead of treating the node as empty",
        ),
        stage_e_lifecycle_matrix_row(
            2,
            "drop_remote_index_in_flight",
            "DROP INDEX",
            "after_fanout_before_receive",
            "remote_index_regclass",
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE,
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "in-flight receive validates the remote index still exists before accepting candidate rows",
        ),
        stage_e_lifecycle_matrix_row(
            3,
            "reindex_remote_index_before_fanout",
            "REINDEX INDEX CONCURRENTLY",
            "before_fanout_planning",
            "remote_index_identity",
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH,
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "descriptor remote_index_identity must be refreshed after reindex before dispatch is considered ready",
        ),
        stage_e_lifecycle_matrix_row(
            4,
            "reindex_remote_index_in_flight",
            "REINDEX INDEX CONCURRENTLY",
            "after_fanout_before_receive",
            "remote_index_identity",
            "fail_closed",
            SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH,
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE,
            "candidate receive compares live endpoint identity with the planned descriptor identity before merge",
        ),
        stage_e_lifecycle_matrix_row(
            5,
            "create_index_concurrently_new_descriptor",
            "CREATE INDEX CONCURRENTLY",
            "during_existing_fanout",
            "remote_node_descriptor_generation",
            "defer_new_descriptor",
            SPIRE_REMOTE_STATUS_READY,
            "defer_new_descriptor",
            SPIRE_REMOTE_STATUS_READY,
            "descriptor_generation_snapshot",
            SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR,
            "new remote indexes are ignored by the already-planned fanout until descriptor generation advances for a later query",
        ),
        stage_e_lifecycle_matrix_row(
            6,
            "create_index_concurrently_missing_descriptor",
            "CREATE INDEX CONCURRENTLY",
            "before_descriptor_registration",
            "remote_node_descriptor",
            "fail_closed",
            SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
            "skip_node",
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
            SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR,
            "a newly-created remote index is not used until the coordinator has an active descriptor and identity binding",
        ),
    ]
}

fn stage_e_lifecycle_matrix_row(
    lifecycle_ordinal: u64,
    lifecycle_case: &'static str,
    ddl_event: &'static str,
    fanout_timing: &'static str,
    affected_surface: &'static str,
    strict_action: &'static str,
    strict_status: &'static str,
    degraded_action: &'static str,
    degraded_status: &'static str,
    required_detection: &'static str,
    next_executor_step: &'static str,
    required_evidence: &'static str,
) -> SpireRemoteStageELifecycleMatrixRow {
    SpireRemoteStageELifecycleMatrixRow {
        lifecycle_ordinal,
        lifecycle_case,
        ddl_event,
        fanout_timing,
        affected_surface,
        strict_action,
        strict_status,
        degraded_action,
        degraded_status,
        required_detection,
        next_executor_step,
        required_evidence,
    }
}

pub(crate) unsafe fn remote_search_production_executor_state_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteProductionExecutorStateSummaryRow {
    let result = (|| -> Result<SpireRemoteProductionExecutorStateSummaryRow, String> {
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_production_executor_state_summary_from_dispatch_rows(
            requested_epoch,
            &dispatch_rows,
            "function_argument",
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_production_degraded_skip_report_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteProductionDegradedSkipReportRow> {
    let result = (|| -> Result<Vec<SpireRemoteProductionDegradedSkipReportRow>, String> {
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_production_degraded_skip_report_from_dispatch_rows(
            requested_epoch,
            &dispatch_rows,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_production_executor_session_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
) -> SpireRemoteProductionExecutorSessionSummaryRow {
    let consistency_mode = options::current_session_remote_search_consistency_mode_name();
    let summary = unsafe {
        remote_search_production_executor_state_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    SpireRemoteProductionExecutorSessionSummaryRow {
        requested_epoch: summary.requested_epoch,
        consistency_mode_source: "ec_spire.remote_search_consistency_mode",
        consistency_mode,
        dispatch_count: summary.dispatch_count,
        degraded_skipped_dispatch_count: summary.degraded_skipped_dispatch_count,
        first_degraded_skip_category: summary.first_degraded_skip_category,
        next_executor_step: summary.next_executor_step,
        status: summary.status,
        recommendation: summary.recommendation,
    }
}

pub(crate) unsafe fn remote_search_production_scan_handoff_summary_row(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k: usize,
) -> SpireRemoteProductionScanHandoffSummaryRow {
    let result = (|| -> Result<SpireRemoteProductionScanHandoffSummaryRow, String> {
        let query_for_scan = scan::SpireScanQuery::new(query.clone())?;
        let consistency_mode = options::current_session_remote_search_consistency_mode_name();
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(SpireRemoteProductionScanHandoffSummaryRow {
                requested_epoch: 0,
                consistency_mode_source: "ec_spire.remote_search_consistency_mode",
                consistency_mode,
                effective_nprobe: 0,
                selected_pid_count: 0,
                local_pid_count: 0,
                remote_pid_count: 0,
                skipped_pid_count: 0,
                dispatch_count: 0,
                candidate_receive_ready_dispatch_count: 0,
                candidate_receive_failed_dispatch_count: 0,
                degraded_skipped_dispatch_count: 0,
                first_degraded_skip_category: SPIRE_REMOTE_NONE,
                candidate_row_count: 0,
                merged_candidate_count: 0,
                duplicate_vec_id_count: 0,
                final_heap_fetch_status: SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
                next_blocker: SPIRE_REMOTE_NONE,
                status: "empty_index",
                recommendation: "publish an active SPIRE epoch before production scan handoff",
            });
        }

        let (epoch_manifest, object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
        };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let relation_options = unsafe { options::relation_options(index_relation) };
        let top_graph_plan = relation_options.top_graph_plan()?;
        let leaf_count = scan::count_scan_plan_routable_leaf_pids(&snapshot, &object_store)?;
        let scan_plan = options::resolve_single_level_scan_plan(leaf_count, relation_options)?;
        let selected_leaf_pids = scan::collect_scan_plan_selected_leaf_pids(
            &snapshot,
            &object_store,
            &query_for_scan,
            scan_plan,
            top_graph_plan,
        )?;
        let selected_pid_count = u64::try_from(selected_leaf_pids.len())
            .map_err(|_| "ec_spire production scan handoff selected PID count exceeds u64")?;

        let execution_summary = unsafe {
            remote_search_execution_summary_row(
                index_relation,
                root_control.active_epoch,
                query.clone(),
                selected_leaf_pids.clone(),
                top_k,
                consistency_mode,
            )
        };
        if top_k == 0 {
            return Ok(SpireRemoteProductionScanHandoffSummaryRow {
                requested_epoch: root_control.active_epoch,
                consistency_mode_source: "ec_spire.remote_search_consistency_mode",
                consistency_mode,
                effective_nprobe: u64::from(scan_plan.nprobe),
                selected_pid_count,
                local_pid_count: execution_summary.local_pid_count,
                remote_pid_count: execution_summary.remote_pid_count,
                skipped_pid_count: execution_summary.skipped_pid_count,
                dispatch_count: 0,
                candidate_receive_ready_dispatch_count: 0,
                candidate_receive_failed_dispatch_count: 0,
                degraded_skipped_dispatch_count: 0,
                first_degraded_skip_category: SPIRE_REMOTE_NONE,
                candidate_row_count: 0,
                merged_candidate_count: 0,
                duplicate_vec_id_count: 0,
                final_heap_fetch_status: SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
                next_blocker: SPIRE_REMOTE_NONE,
                status: SPIRE_REMOTE_STATUS_EMPTY_TOP_K,
                recommendation: SPIRE_REMOTE_NONE,
            });
        }

        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                root_control.active_epoch,
                query.clone(),
                selected_leaf_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(root_control.active_epoch, &dispatch_rows);
        executor.mark_planned_dispatches_candidate_receive_ready();
        executor.run_compact_candidate_receive(&query, top_k, consistency_mode)?;
        let summary = executor.summary(
            "ec_spire.remote_search_consistency_mode",
            consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?),
        )?;
        let should_merge = matches!(
            summary.status,
            SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
                | SPIRE_REMOTE_STATUS_DEGRADED_READY
                | SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED
                | SPIRE_REMOTE_STATUS_READY
        );
        let merge = if should_merge {
            executor.merge_ready_candidate_batches(Some(top_k))?
        } else {
            SpireRemoteSearchMergeResult {
                candidates: Vec::new(),
                input_count: 0,
                duplicate_vec_id_count: 0,
            }
        };
        let merged_candidate_count = u64::try_from(merge.candidates.len())
            .map_err(|_| "ec_spire production scan handoff merge count exceeds u64")?;
        let final_heap_fetch_status = if matches!(
            summary.status,
            SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP | SPIRE_REMOTE_STATUS_DEGRADED_READY
        ) {
            SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
        } else if summary.status == SPIRE_REMOTE_STATUS_READY && execution_summary.remote_pid_count == 0
        {
            SPIRE_REMOTE_FINAL_STATUS_LOCAL_READY
        } else if summary.status == SPIRE_REMOTE_STATUS_READY
            || summary.status == SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED
        {
            SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES
        } else {
            SPIRE_REMOTE_FINAL_STATUS_BLOCKED
        };
        let next_blocker = if final_heap_fetch_status == SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
        {
            SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION
        } else if matches!(
            summary.status,
            SPIRE_REMOTE_STATUS_READY | SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED
        ) {
            SPIRE_REMOTE_NONE
        } else {
            summary.next_executor_step
        };
        let (status, recommendation) =
            if final_heap_fetch_status == SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP {
                (
                    SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
                    "add origin-node row locator resolution before returning remote heap rows",
                )
            } else {
                (summary.status, summary.recommendation)
            };

        Ok(SpireRemoteProductionScanHandoffSummaryRow {
            requested_epoch: root_control.active_epoch,
            consistency_mode_source: "ec_spire.remote_search_consistency_mode",
            consistency_mode,
            effective_nprobe: u64::from(scan_plan.nprobe),
            selected_pid_count,
            local_pid_count: execution_summary.local_pid_count,
            remote_pid_count: execution_summary.remote_pid_count,
            skipped_pid_count: execution_summary.skipped_pid_count,
            dispatch_count: summary.dispatch_count,
            candidate_receive_ready_dispatch_count: summary.candidate_receive_ready_dispatch_count,
            candidate_receive_failed_dispatch_count: summary
                .candidate_receive_failed_dispatch_count,
            degraded_skipped_dispatch_count: summary.degraded_skipped_dispatch_count,
            first_degraded_skip_category: summary.first_degraded_skip_category,
            candidate_row_count: summary.candidate_row_count,
            merged_candidate_count,
            duplicate_vec_id_count: merge.duplicate_vec_id_count,
            final_heap_fetch_status,
            next_blocker,
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn production_scan_output_from_heap_candidate(
    candidate: &SpireRemoteSearchLocalHeapCandidateRow,
) -> SpireRemoteProductionScanOutputRow {
    SpireRemoteProductionScanOutputRow {
        requested_epoch: candidate.requested_epoch,
        served_epoch: candidate.served_epoch,
        node_id: candidate.node_id,
        heap_block: candidate.heap_block,
        heap_offset: candidate.heap_offset,
        score: candidate.score,
        heap_lookup_owner: candidate.heap_lookup_owner,
        vec_id: candidate.vec_id.clone(),
        row_locator: candidate.row_locator.clone(),
        tuple_payload_json: candidate.tuple_payload_json.clone(),
        typed_tuple_payload: candidate.typed_tuple_payload.clone(),
        tuple_payload_missing: candidate.tuple_payload_missing,
    }
}

fn production_scan_outputs_from_heap_candidates(
    candidates: &[SpireRemoteSearchLocalHeapCandidateRow],
) -> Vec<SpireRemoteProductionScanOutputRow> {
    candidates
        .iter()
        .map(production_scan_output_from_heap_candidate)
        .collect()
}

fn production_scan_output_is_local_heap_tid(output: &SpireRemoteProductionScanOutputRow) -> bool {
    matches!(
        output.heap_lookup_owner,
        SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION
    )
}

fn production_scan_am_delivery_summary(
    summary: &SpireRemoteProductionScanHeapResolutionSummaryRow,
    outputs: &[SpireRemoteProductionScanOutputRow],
) -> Result<SpireRemoteProductionScanAmDeliverySummaryRow, String> {
    let output_count = u64::try_from(outputs.len())
        .map_err(|_| "ec_spire production scan output count exceeds u64")?;
    let local_heap_tid_output_count = u64::try_from(
        outputs
            .iter()
            .filter(|output| production_scan_output_is_local_heap_tid(output))
            .count(),
    )
    .map_err(|_| "ec_spire production scan local output count exceeds u64")?;
    let remote_origin_output_count = output_count
        .checked_sub(local_heap_tid_output_count)
        .ok_or_else(|| "ec_spire production scan output count underflow".to_owned())?;

    let (am_deliverable_output_count, status, next_blocker, recommendation) =
        if summary.next_blocker != SPIRE_REMOTE_NONE {
            (
                0,
                summary.status,
                summary.next_blocker,
                summary.recommendation,
            )
        } else if remote_origin_output_count > 0 {
            (
                0,
                SPIRE_REMOTE_FINAL_STATUS_REQUIRES_CUSTOM_SCAN_TUPLE_DELIVERY,
                SPIRE_REMOTE_EXECUTOR_STEP_CUSTOM_SCAN_TUPLE_DELIVERY,
                "route remote-origin rows through EcSpireDistributedScan tuple delivery instead of the index AM cursor",
            )
        } else if local_heap_tid_output_count > 0 {
            (
                local_heap_tid_output_count,
                SPIRE_REMOTE_STATUS_READY,
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
            )
        } else {
            (
                0,
                summary.status,
                summary.next_blocker,
                summary.recommendation,
            )
        };

    Ok(SpireRemoteProductionScanAmDeliverySummaryRow {
        requested_epoch: summary.requested_epoch,
        output_count,
        local_heap_tid_output_count,
        remote_origin_output_count,
        am_deliverable_output_count,
        status,
        next_blocker,
        recommendation,
    })
}

fn production_scan_result_stream(
    summary: SpireRemoteProductionScanHeapResolutionSummaryRow,
    outputs: Vec<SpireRemoteProductionScanOutputRow>,
) -> Result<SpireRemoteProductionScanResultStream, String> {
    let am_delivery = production_scan_am_delivery_summary(&summary, &outputs)?;
    Ok(SpireRemoteProductionScanResultStream {
        summary,
        am_delivery,
        outputs,
    })
}

unsafe fn remote_search_production_scan_heap_resolution_result_stream_impl(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k_override: Option<usize>,
    tuple_payload_columns: Option<&[String]>,
) -> Result<SpireRemoteProductionScanResultStream, String> {
        let query_for_scan = scan::SpireScanQuery::new(query.clone())?;
        let consistency_mode = options::current_session_remote_search_consistency_mode_name();
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return production_scan_result_stream(
                SpireRemoteProductionScanHeapResolutionSummaryRow {
                    requested_epoch: 0,
                    consistency_mode_source: "ec_spire.remote_search_consistency_mode",
                    consistency_mode,
                    effective_nprobe: 0,
                    selected_pid_count: 0,
                    local_pid_count: 0,
                    remote_pid_count: 0,
                    skipped_pid_count: 0,
                    dispatch_count: 0,
                    compact_candidate_count: 0,
                    remote_heap_ready_dispatch_count: 0,
                    remote_heap_failed_dispatch_count: 0,
                    remote_heap_candidate_count: 0,
                    local_heap_candidate_count: 0,
                    returned_candidate_count: 0,
                    result_source: SPIRE_REMOTE_NONE,
                    final_heap_fetch_status: SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
                    next_blocker: SPIRE_REMOTE_NONE,
                    status: "empty_index",
                    recommendation: "publish an active SPIRE epoch before production scan heap resolution",
                },
                Vec::new(),
            );
        }

        let (epoch_manifest, object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
        };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let relation_options = unsafe { options::relation_options(index_relation) };
        let top_graph_plan = relation_options.top_graph_plan()?;
        let leaf_count = scan::count_scan_plan_routable_leaf_pids(&snapshot, &object_store)?;
        let scan_plan = options::resolve_single_level_scan_plan(leaf_count, relation_options)?;
        let top_k = match top_k_override {
            Some(top_k) => top_k,
            None => scan_plan
                .candidate_limit
                .ok_or_else(|| "ec_spire production AM scan candidate limit is unavailable".to_owned())?,
        };
        let selected_leaf_pids = scan::collect_scan_plan_selected_leaf_pids(
            &snapshot,
            &object_store,
            &query_for_scan,
            scan_plan,
            top_graph_plan,
        )?;
        let selected_pid_count = u64::try_from(selected_leaf_pids.len())
            .map_err(|_| "ec_spire production scan heap selected PID count exceeds u64")?;
        let execution_summary = unsafe {
            remote_search_execution_summary_row(
                index_relation,
                root_control.active_epoch,
                query.clone(),
                selected_leaf_pids.clone(),
                top_k,
                consistency_mode,
            )
        };

        if top_k == 0 {
            return production_scan_result_stream(
                SpireRemoteProductionScanHeapResolutionSummaryRow {
                    requested_epoch: root_control.active_epoch,
                    consistency_mode_source: "ec_spire.remote_search_consistency_mode",
                    consistency_mode,
                    effective_nprobe: u64::from(scan_plan.nprobe),
                    selected_pid_count,
                    local_pid_count: execution_summary.local_pid_count,
                    remote_pid_count: execution_summary.remote_pid_count,
                    skipped_pid_count: execution_summary.skipped_pid_count,
                    dispatch_count: 0,
                    compact_candidate_count: 0,
                    remote_heap_ready_dispatch_count: 0,
                    remote_heap_failed_dispatch_count: 0,
                    remote_heap_candidate_count: 0,
                    local_heap_candidate_count: 0,
                    returned_candidate_count: 0,
                    result_source: SPIRE_REMOTE_NONE,
                    final_heap_fetch_status: SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
                    next_blocker: SPIRE_REMOTE_NONE,
                    status: SPIRE_REMOTE_STATUS_EMPTY_TOP_K,
                    recommendation: SPIRE_REMOTE_NONE,
                },
                Vec::new(),
            );
        }

        let local_heap_rows = if execution_summary.local_pid_count > 0 {
            unsafe {
                remote_search_local_heap_candidate_rows_for_result_summary(
                    index_relation,
                    root_control.active_epoch,
                    query.clone(),
                    selected_leaf_pids.clone(),
                    top_k,
                    consistency_mode,
                )
            }
        } else {
            Vec::new()
        };
        let local_heap_candidate_count = u64::try_from(
            local_heap_rows
                .iter()
                .filter(|row| row.status == SPIRE_REMOTE_STATUS_READY)
                .count(),
        )
        .map_err(|_| "ec_spire production scan heap local candidate count exceeds u64")?;

        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                root_control.active_epoch,
                query.clone(),
                selected_leaf_pids,
                top_k,
                consistency_mode,
            )
        };
        let dispatch_count = u64::try_from(dispatch_rows.len())
            .map_err(|_| "ec_spire production scan heap dispatch count exceeds u64")?;
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(root_control.active_epoch, &dispatch_rows);
        executor.mark_planned_dispatches_candidate_receive_ready();
        executor.run_compact_candidate_receive(&query, top_k, consistency_mode)?;
        let compact_summary = executor.summary(
            "ec_spire.remote_search_consistency_mode",
            consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?),
        )?;
        let compact_allows_heap = matches!(
            compact_summary.status,
            SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
                | SPIRE_REMOTE_STATUS_DEGRADED_READY
                | SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED
                | SPIRE_REMOTE_STATUS_READY
        );
        if compact_allows_heap {
            executor.run_remote_heap_receive(&query, top_k, consistency_mode, tuple_payload_columns)?;
        }
        let (remote_heap_ready_dispatch_count, remote_heap_failed_dispatch_count, remote_heap_candidate_count) =
            executor.remote_heap_resolution_counts()?;
        let remote_heap_rows = if compact_allows_heap && remote_heap_failed_dispatch_count == 0 {
            executor.ready_remote_heap_candidate_rows()?
        } else {
            Vec::new()
        };

        let mut heap_rows = local_heap_rows
            .into_iter()
            .filter(|row| row.status == SPIRE_REMOTE_STATUS_READY)
            .collect::<Vec<_>>();
        heap_rows.extend(remote_heap_rows);
        let merged = merge_remote_search_heap_candidates_for_result(heap_rows, top_k)?;
        let returned_candidate_count = u64::try_from(merged.len())
            .map_err(|_| "ec_spire production scan heap returned candidate count exceeds u64")?;

        let result_source = if remote_heap_candidate_count > 0 {
            SPIRE_REMOTE_RESULT_SOURCE_REMOTE_HEAP_CANDIDATES
        } else if local_heap_candidate_count > 0 {
            SPIRE_REMOTE_RESULT_SOURCE_LOCAL_HEAP_CANDIDATES
        } else if compact_summary.next_executor_step != SPIRE_REMOTE_NONE {
            SPIRE_REMOTE_RESULT_SOURCE_BLOCKED
        } else {
            SPIRE_REMOTE_NONE
        };
        let final_heap_fetch_status = if remote_heap_failed_dispatch_count > 0 {
            SPIRE_REMOTE_FINAL_STATUS_BLOCKED
        } else if remote_heap_candidate_count > 0 {
            SPIRE_REMOTE_FINAL_STATUS_REMOTE_READY
        } else if local_heap_candidate_count > 0 {
            SPIRE_REMOTE_FINAL_STATUS_LOCAL_READY
        } else {
            SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES
        };
        let (next_blocker, status, recommendation) = if remote_heap_failed_dispatch_count > 0 {
            (
                SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_HEAP_RESOLUTION,
                SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED,
                "inspect production remote heap failure category before final row delivery",
            )
        } else if !compact_allows_heap {
            (
                compact_summary.next_executor_step,
                compact_summary.status,
                compact_summary.recommendation,
            )
        } else if returned_candidate_count > 0 {
            (
                SPIRE_REMOTE_NONE,
                if execution_summary.skipped_pid_count > 0 {
                    SPIRE_REMOTE_STATUS_DEGRADED_READY
                } else {
                    SPIRE_REMOTE_STATUS_READY
                },
                SPIRE_REMOTE_NONE,
            )
        } else {
            (
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
                "inspect local and remote heap visibility for the selected candidates",
            )
        };

        production_scan_result_stream(
            SpireRemoteProductionScanHeapResolutionSummaryRow {
                requested_epoch: root_control.active_epoch,
                consistency_mode_source: "ec_spire.remote_search_consistency_mode",
                consistency_mode,
                effective_nprobe: u64::from(scan_plan.nprobe),
                selected_pid_count,
                local_pid_count: execution_summary.local_pid_count,
                remote_pid_count: execution_summary.remote_pid_count,
                skipped_pid_count: execution_summary.skipped_pid_count,
                dispatch_count,
                compact_candidate_count: compact_summary.candidate_row_count,
                remote_heap_ready_dispatch_count,
                remote_heap_failed_dispatch_count,
                remote_heap_candidate_count,
                local_heap_candidate_count,
                returned_candidate_count,
                result_source,
                final_heap_fetch_status,
                next_blocker,
                status,
                recommendation,
            },
            production_scan_outputs_from_heap_candidates(&merged),
        )
}

pub(crate) unsafe fn remote_search_production_scan_heap_resolution_result_stream(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k: usize,
) -> SpireRemoteProductionScanResultStream {
    let result = unsafe {
        remote_search_production_scan_heap_resolution_result_stream_impl(
            index_relation,
            query,
            Some(top_k),
            None,
        )
    };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_production_scan_tuple_payload_result_stream(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k: usize,
    tuple_payload_columns: &[String],
) -> SpireRemoteProductionScanResultStream {
    let result = unsafe {
        remote_search_production_scan_heap_resolution_result_stream_impl(
            index_relation,
            query,
            Some(top_k),
            Some(tuple_payload_columns),
        )
    };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_production_scan_heap_resolution_am_result_stream(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    query: Vec<f32>,
) -> SpireRemoteProductionScanResultStream {
    let result = (|| -> Result<SpireRemoteProductionScanResultStream, String> {
        let stream = unsafe {
            remote_search_production_scan_heap_resolution_result_stream_impl(
                index_relation,
                query,
                None,
                None,
            )?
        };
        let _ = (heap_relation, snapshot);
        Ok(stream)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_production_scan_heap_resolution_summary_row(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k: usize,
) -> SpireRemoteProductionScanHeapResolutionSummaryRow {
    unsafe {
        remote_search_production_scan_heap_resolution_result_stream(index_relation, query, top_k)
            .summary
    }
}

pub(crate) unsafe fn remote_search_operator_diagnostics_row(
    index_relation: pg_sys::Relation,
    query: Vec<f32>,
    top_k: usize,
) -> SpireRemoteSearchOperatorDiagnosticsRow {
    let result = (|| -> Result<SpireRemoteSearchOperatorDiagnosticsRow, String> {
        let capability = unsafe { remote_node_capability_summary(index_relation) };
        let remote_snapshots = unsafe { remote_node_snapshot(index_relation) }
            .into_iter()
            .filter(|row| row.node_id != meta::SPIRE_LOCAL_NODE_ID)
            .collect::<Vec<_>>();
        let min_remote_last_served_epoch = remote_snapshots
            .iter()
            .map(|row| row.last_served_epoch)
            .min()
            .unwrap_or(0);
        let max_remote_last_served_epoch = remote_snapshots
            .iter()
            .map(|row| row.last_served_epoch)
            .max()
            .unwrap_or(0);
        let ready_remote_node_count = u64::try_from(
            remote_snapshots
                .iter()
                .filter(|row| row.status == SPIRE_REMOTE_STATUS_READY)
                .count(),
        )
        .map_err(|_| "ec_spire operator diagnostics ready remote node count exceeds u64")?;
        let blocked_remote_node_count = u64::try_from(remote_snapshots.len())
            .map_err(|_| "ec_spire operator diagnostics remote node count exceeds u64")?
            .checked_sub(ready_remote_node_count)
            .ok_or_else(|| "ec_spire operator diagnostics remote node count underflow".to_owned())?;
        let stream =
            unsafe { remote_search_production_scan_heap_resolution_result_stream(index_relation, query, top_k) };
        let summary = stream.summary;
        let am_delivery = stream.am_delivery;

        let (next_blocker, status, recommendation) =
            if capability.remote_node_count > 0 && capability.status != SPIRE_REMOTE_STATUS_READY {
                (
                    "remote_node_capability",
                    capability.status,
                    capability.recommendation,
                )
            } else if am_delivery.next_blocker != SPIRE_REMOTE_NONE {
                (
                    am_delivery.next_blocker,
                    am_delivery.status,
                    am_delivery.recommendation,
                )
            } else {
                (summary.next_blocker, summary.status, summary.recommendation)
            };

        Ok(SpireRemoteSearchOperatorDiagnosticsRow {
            active_epoch: summary.requested_epoch,
            consistency_mode: summary.consistency_mode,
            remote_node_count: capability.remote_node_count,
            ready_remote_node_count,
            blocked_remote_node_count,
            min_remote_last_served_epoch,
            max_remote_last_served_epoch,
            remote_readiness_status: capability.status,
            effective_nprobe: summary.effective_nprobe,
            selected_pid_count: summary.selected_pid_count,
            local_pid_count: summary.local_pid_count,
            remote_pid_count: summary.remote_pid_count,
            skipped_pid_count: summary.skipped_pid_count,
            remote_fanout_count: summary.dispatch_count,
            candidate_batch_count: summary.dispatch_count,
            candidate_row_count: summary.compact_candidate_count,
            remote_heap_ready_dispatch_count: summary.remote_heap_ready_dispatch_count,
            remote_heap_failed_dispatch_count: summary.remote_heap_failed_dispatch_count,
            remote_heap_candidate_count: summary.remote_heap_candidate_count,
            local_heap_candidate_count: summary.local_heap_candidate_count,
            returned_candidate_count: summary.returned_candidate_count,
            result_source: summary.result_source,
            final_heap_fetch_status: summary.final_heap_fetch_status,
            merge_status: summary.status,
            am_delivery_status: am_delivery.status,
            am_deliverable_output_count: am_delivery.am_deliverable_output_count,
            remote_origin_output_count: am_delivery.remote_origin_output_count,
            next_blocker,
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_libpq_secret_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqSecretPlanRow> {
    let dispatch_rows = unsafe {
        remote_search_libpq_dispatch_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };

    remote_search_libpq_secret_plan_rows_from_dispatch(&dispatch_rows)
}

fn remote_search_libpq_secret_plan_rows_from_dispatch(
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
) -> Vec<SpireRemoteSearchLibpqSecretPlanRow> {
    dispatch_rows
        .iter()
        .map(|row| {
            if row.dispatch_action != SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
                let next_executor_step = remote_search_pre_dispatch_blocker_step(row.status);
                let recommendation =
                    remote_search_pre_dispatch_blocker_recommendation(row.status);
                return SpireRemoteSearchLibpqSecretPlanRow {
                    requested_epoch: row.requested_epoch,
                    node_id: row.node_id,
                    selected_pids: row.selected_pids.clone(),
                    pid_count: row.pid_count,
                    conninfo_secret_name: row.conninfo_secret_name.clone(),
                    provider_lookup_key: SPIRE_REMOTE_NONE.to_owned(),
                    resolved_conninfo_bytes: 0,
                    raw_conninfo_exposed: false,
                    secret_resolution_action: SPIRE_REMOTE_NONE,
                    next_executor_step,
                    status: row.status,
                    recommendation,
                };
            }

            let secret_status =
                remote_conninfo_secret_resolution_status_row(&row.conninfo_secret_name);
            let (secret_resolution_action, next_executor_step) =
                if secret_status.status == SPIRE_REMOTE_CONNINFO_RESOLVED {
                    ("resolved_conninfo_secret_reference", "open_libpq_connection")
                } else {
                    (
                        "resolve_conninfo_secret_reference",
                        SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
                    )
                };

            SpireRemoteSearchLibpqSecretPlanRow {
                requested_epoch: row.requested_epoch,
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                conninfo_secret_name: row.conninfo_secret_name.clone(),
                provider_lookup_key: secret_status.provider_lookup_key,
                resolved_conninfo_bytes: secret_status.resolved_conninfo_bytes,
                raw_conninfo_exposed: secret_status.raw_conninfo_exposed,
                secret_resolution_action,
                next_executor_step,
                status: secret_status.status,
                recommendation: secret_status.recommendation,
            }
        })
        .collect()
}

pub(crate) unsafe fn remote_search_libpq_secret_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqSecretSummaryRow {
    let rows = unsafe {
        remote_search_libpq_secret_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_libpq_secret_summary_from_plan_rows(requested_epoch, &rows)
        .unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_secret_summary_from_plan_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqSecretPlanRow],
) -> Result<SpireRemoteSearchLibpqSecretSummaryRow, String> {
    let mut resolved_secret_count = 0_u64;
    let mut blocked_secret_count = 0_u64;
    let mut pre_secret_blocked_count = 0_u64;
    let mut remote_pid_count = 0_u64;
    let mut blocked_pid_count = 0_u64;
    let mut first_pre_secret_blocked_status = SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR;
    let mut first_pre_secret_blocked_step = SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR;

    for row in rows {
        add_remote_count(
            &mut remote_pid_count,
            row.pid_count,
            "remote search libpq secret summary",
            "remote PID",
        )?;
        if row.status == SPIRE_REMOTE_CONNINFO_RESOLVED {
            add_remote_count(
                &mut resolved_secret_count,
                1,
                "remote search libpq secret summary",
                "resolved secret",
            )?;
        } else {
            add_remote_count(
                &mut blocked_pid_count,
                row.pid_count,
                "remote search libpq secret summary",
                "blocked PID",
            )?;
            if row.next_executor_step == SPIRE_REMOTE_EXECUTOR_STEP_SECRET {
                add_remote_count(
                    &mut blocked_secret_count,
                    1,
                    "remote search libpq secret summary",
                    "blocked secret",
                )?;
            } else {
                if pre_secret_blocked_count == 0 {
                    first_pre_secret_blocked_status = row.status;
                    first_pre_secret_blocked_step = row.next_executor_step;
                }
                add_remote_count(
                    &mut pre_secret_blocked_count,
                    1,
                    "remote search libpq secret summary",
                    "pre-secret-blocked row",
                )?;
            }
        }
    }

    let secret_count = u64::try_from(rows.len())
        .map_err(|_| "remote search libpq secret count exceeds u64")?;
    let (next_executor_step, status) = if secret_count == 0 {
        (SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY)
    } else if pre_secret_blocked_count > 0 {
        (first_pre_secret_blocked_step, first_pre_secret_blocked_status)
    } else if blocked_secret_count > 0 {
        (
            SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
            SPIRE_REMOTE_STATUS_REQUIRES_SECRET,
        )
    } else {
        ("open_libpq_connection", SPIRE_REMOTE_CONNINFO_RESOLVED)
    };

    Ok(SpireRemoteSearchLibpqSecretSummaryRow {
        requested_epoch,
        secret_count,
        resolved_secret_count,
        blocked_secret_count,
        remote_pid_count,
        blocked_pid_count,
        next_executor_step,
        status,
    })
}

pub(crate) unsafe fn remote_search_libpq_connection_open_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqConnectionOpenPlanRow> {
    let secret_rows = unsafe {
        remote_search_libpq_secret_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };

    remote_search_libpq_connection_open_plan_rows_from_secrets(&secret_rows)
}

fn remote_search_libpq_connection_open_plan_rows_from_secrets(
    secret_rows: &[SpireRemoteSearchLibpqSecretPlanRow],
) -> Vec<SpireRemoteSearchLibpqConnectionOpenPlanRow> {
    secret_rows
        .iter()
        .map(|row| {
            let (connection_action, next_executor_step, status, recommendation) =
                if row.status == SPIRE_REMOTE_CONNINFO_RESOLVED {
                    (
                        "open_libpq_connection",
                        "enter_libpq_pipeline_mode",
                        SPIRE_REMOTE_EXECUTOR_REQUIRED,
                        "open per-query libpq connection with executor-owned resolved conninfo",
                    )
                } else {
                    (
                        "blocked_before_connection",
                        row.next_executor_step,
                        row.status,
                        row.recommendation,
                    )
                };

            SpireRemoteSearchLibpqConnectionOpenPlanRow {
                requested_epoch: row.requested_epoch,
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                conninfo_secret_name: row.conninfo_secret_name.clone(),
                provider_lookup_key: row.provider_lookup_key.clone(),
                resolved_conninfo_bytes: row.resolved_conninfo_bytes,
                connection_lifecycle_policy: "per_query",
                pooling_policy: "no_pooling_v1",
                connection_action,
                next_executor_step,
                status,
                recommendation,
            }
        })
        .collect()
}

pub(crate) unsafe fn remote_search_libpq_connection_open_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqConnectionOpenSummaryRow {
    let rows = unsafe {
        remote_search_libpq_connection_open_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_libpq_connection_open_summary_from_plan_rows(requested_epoch, &rows)
        .unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_connection_open_summary_from_plan_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqConnectionOpenPlanRow],
) -> Result<SpireRemoteSearchLibpqConnectionOpenSummaryRow, String> {
    let mut ready_connection_count = 0_u64;
    let mut blocked_connection_count = 0_u64;
    let mut remote_pid_count = 0_u64;
    let mut blocked_pid_count = 0_u64;
    let mut first_blocked_step = SPIRE_REMOTE_NONE;
    let mut first_blocked_status = SPIRE_REMOTE_STATUS_READY;

    for row in rows {
        add_remote_count(
            &mut remote_pid_count,
            row.pid_count,
            "remote search libpq connection-open summary",
            "remote PID",
        )?;
        if row.connection_action == "open_libpq_connection" {
            add_remote_count(
                &mut ready_connection_count,
                1,
                "remote search libpq connection-open summary",
                "ready connection",
            )?;
        } else {
            if blocked_connection_count == 0 {
                first_blocked_step = row.next_executor_step;
                first_blocked_status = row.status;
            }
            add_remote_count(
                &mut blocked_connection_count,
                1,
                "remote search libpq connection-open summary",
                "blocked connection",
            )?;
            add_remote_count(
                &mut blocked_pid_count,
                row.pid_count,
                "remote search libpq connection-open summary",
                "blocked PID",
            )?;
        }
    }

    let connection_count = u64::try_from(rows.len())
        .map_err(|_| "remote search libpq connection-open count exceeds u64")?;
    let (next_executor_step, status) = if connection_count == 0 {
        (SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY)
    } else if blocked_connection_count > 0 {
        (first_blocked_step, first_blocked_status)
    } else {
        ("enter_libpq_pipeline_mode", SPIRE_REMOTE_EXECUTOR_REQUIRED)
    };

    Ok(SpireRemoteSearchLibpqConnectionOpenSummaryRow {
        requested_epoch,
        connection_count,
        ready_connection_count,
        blocked_connection_count,
        remote_pid_count,
        blocked_pid_count,
        next_executor_step,
        status,
    })
}

pub(crate) unsafe fn remote_search_libpq_executor_readiness_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqExecutorReadinessRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqExecutorReadinessRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search libpq executor readiness top_k exceeds u64")?;
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let dispatch_summary = remote_search_libpq_dispatch_summary_from_plan_rows(
            requested_epoch,
            &dispatch_rows,
            query_for_empty_plan,
            top_k_for_empty_plan,
            consistency_mode,
        )?;
        let secret_rows = remote_search_libpq_secret_plan_rows_from_dispatch(&dispatch_rows);
        let secret_summary =
            remote_search_libpq_secret_summary_from_plan_rows(requested_epoch, &secret_rows)?;

        Ok(remote_search_libpq_executor_readiness_from_summaries(
            requested_epoch,
            &dispatch_summary,
            &secret_summary,
        ))
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_executor_readiness_from_summaries(
    requested_epoch: u64,
    dispatch_summary: &SpireRemoteSearchLibpqDispatchSummaryRow,
    secret_summary: &SpireRemoteSearchLibpqSecretSummaryRow,
) -> SpireRemoteSearchLibpqExecutorReadinessRow {
    let blocked_dispatch_count = dispatch_summary
        .dispatch_count
        .saturating_sub(dispatch_summary.pipeline_dispatch_count);

    let (
        secret_resolution_action,
        connection_action,
        pipeline_action,
        send_action,
        receive_action,
        merge_action,
        next_executor_step,
        status,
        recommendation,
    ) = if dispatch_summary.status == SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD {
        (
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_EXECUTOR_STEP_BUDGET,
            SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            remote_search_pre_dispatch_blocker_recommendation(
                SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            ),
        )
    } else if dispatch_summary.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
        (
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR,
            SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
            "register active or draining remote node descriptors before libpq executor startup",
        )
    } else if dispatch_summary.pipeline_dispatch_count > 0
        && secret_summary.status == SPIRE_REMOTE_CONNINFO_RESOLVED
    {
        (
            "resolve_conninfo_secret_reference",
            "open_libpq_connection",
            "enter_libpq_pipeline_mode",
            "send_remote_search_request",
            SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
            "open_libpq_connection",
            SPIRE_REMOTE_EXECUTOR_REQUIRED,
            "open executor-owned libpq connections before remote dispatch",
        )
    } else if dispatch_summary.pipeline_dispatch_count > 0 {
        (
            "resolve_conninfo_secret_reference",
            "open_libpq_connection",
            "enter_libpq_pipeline_mode",
            "send_remote_search_request",
            SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
            SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
            SPIRE_REMOTE_EXECUTOR_REQUIRED,
            "implement conninfo secret resolution and libpq pipeline execution before remote dispatch",
        )
    } else {
        let next_executor_step = remote_search_pre_dispatch_blocker_step(dispatch_summary.status);
        let recommendation =
            remote_search_pre_dispatch_blocker_recommendation(dispatch_summary.status);
        (
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            next_executor_step,
            dispatch_summary.status,
            recommendation,
        )
    };

    SpireRemoteSearchLibpqExecutorReadinessRow {
        requested_epoch,
        dispatch_count: dispatch_summary.dispatch_count,
        pipeline_dispatch_count: dispatch_summary.pipeline_dispatch_count,
        blocked_dispatch_count,
        secret_resolution_action,
        connection_action,
        pipeline_action,
        send_action,
        receive_action,
        merge_action,
        next_executor_step,
        status,
        recommendation,
    }
}

pub(crate) fn remote_search_libpq_executor_step_contract_rows(
) -> Vec<SpireRemoteSearchLibpqExecutorStepContractRow> {
    vec![
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 1,
            step_name: SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR,
            executor_action: "resolve_remote_node_descriptor",
            input_contract: "remote_leaf_pid_placement",
            output_contract: "active_or_draining_remote_node_descriptor",
            blocking_status: SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
            validator: "descriptor_state_must_allow_pipeline_dispatch",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 2,
            step_name: SPIRE_REMOTE_EXECUTOR_STEP_EPOCH_WINDOW,
            executor_action: "verify_remote_served_epoch_window",
            input_contract: "remote_node_descriptor.last_served_epoch,min_retained_epoch",
            output_contract: "descriptor_epoch_window_covers_requested_epoch",
            blocking_status: SPIRE_REMOTE_STATUS_STALE_EPOCH,
            validator: "descriptor_epoch_window_must_cover_requested_epoch",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 3,
            step_name: SPIRE_REMOTE_EXECUTOR_STEP_EXTENSION_VERSION,
            executor_action: "verify_remote_extension_version",
            input_contract: "remote_node_descriptor.extension_version",
            output_contract: "extension_version_matches_coordinator",
            blocking_status: SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION,
            validator: "descriptor_extension_version_must_match_coordinator",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 4,
            step_name: SPIRE_REMOTE_EXECUTOR_STEP_BUDGET,
            executor_action: "enforce_remote_executor_budget",
            input_contract: "ready_remote_dispatch_rows_and_session_budget_gucs",
            output_contract: "budget_admitted_remote_dispatch_rows",
            blocking_status: SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            validator: "must_block_over_budget_rows_before_secret_lookup_or_socket_open",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 5,
            step_name: SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
            executor_action: "resolve_conninfo_secret_reference",
            input_contract: "conninfo_secret_name",
            output_contract: SPIRE_REMOTE_CONNINFO_READY,
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_resolve_secret_without_exposing_raw_conninfo",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 6,
            step_name: "open_libpq_connection",
            executor_action: "open_libpq_connection",
            input_contract: SPIRE_REMOTE_CONNINFO_READY,
            output_contract: "libpq_connection",
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_target_registered_remote_index",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 7,
            step_name: "enter_libpq_pipeline_mode",
            executor_action: "enter_libpq_pipeline_mode",
            input_contract: "libpq_connection",
            output_contract: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_enter_pipeline_before_sending_remote_search",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 8,
            step_name: "send_remote_search_request",
            executor_action: "send_remote_search_request",
            input_contract: "ec_spire_remote_search_libpq_request_plan",
            output_contract: "pending_remote_search_result",
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_bind_libpq_parameter_contract",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 9,
            step_name: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            executor_action: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            input_contract: "remote_search_result_batch",
            output_contract: "validated_remote_candidate_batch",
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_match_libpq_result_contract",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 10,
            step_name: SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
            executor_action: SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
            input_contract: "validated_remote_candidate_batches",
            output_contract: "coordinator_ranked_candidate_batch",
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_preserve_merge_order_contract",
        },
    ]
}

pub(crate) fn remote_search_libpq_parameter_contract_rows(
) -> Vec<SpireRemoteSearchLibpqParameterContractRow> {
    vec![
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 1,
            parameter_name: "remote_index_oid",
            pg_type: "oid",
            semantic_role: "remote_index_identity",
            validator: "must_resolve_to_ec_spire_index_on_remote_node",
        },
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 2,
            parameter_name: "requested_epoch",
            pg_type: "bigint",
            semantic_role: "served_epoch_gate",
            validator: "must_be_positive_and_served_by_remote_node",
        },
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 3,
            parameter_name: "query",
            pg_type: "real[]",
            semantic_role: "query_vector",
            validator: "must_match_index_dimensions",
        },
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 4,
            parameter_name: "selected_pids",
            pg_type: "bigint[]",
            semantic_role: "selected_leaf_pid_set",
            validator: "must_be_nonempty_positive_unique_remote_leaf_pids_delta_rows_are_leaf_derived",
        },
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 5,
            parameter_name: "top_k",
            pg_type: "integer",
            semantic_role: "candidate_budget",
            validator: "must_be_non_negative",
        },
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 6,
            parameter_name: "consistency_mode",
            pg_type: "text",
            semantic_role: "strict_or_degraded_policy",
            validator: "must_match_active_remote_epoch_consistency_mode",
        },
    ]
}

pub(crate) fn remote_search_libpq_result_contract_rows(
) -> Vec<SpireRemoteSearchLibpqResultContractRow> {
    vec![
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 1,
            column_name: "served_epoch",
            pg_type: "bigint",
            semantic_role: "candidate_epoch",
            nullable: false,
            validator: "must_equal_requested_epoch",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 2,
            column_name: "node_id",
            pg_type: "bigint",
            semantic_role: "candidate_node",
            nullable: false,
            validator: "must_equal_expected_node_id",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 3,
            column_name: "pid",
            pg_type: "bigint",
            semantic_role: "partition_object",
            nullable: false,
            validator: "must_be_selected_leaf_pid_or_leaf_derived_delta_pid",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 4,
            column_name: "object_version",
            pg_type: "bigint",
            semantic_role: "partition_object_version",
            nullable: false,
            validator: "must_be_positive",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 5,
            column_name: "row_index",
            pg_type: "bigint",
            semantic_role: "candidate_row_index",
            nullable: false,
            validator: "must_fit_u32",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 6,
            column_name: "assignment_flags",
            pg_type: "smallint",
            semantic_role: "candidate_assignment_flags",
            nullable: false,
            validator: "must_include_primary_or_boundary_replica",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 7,
            column_name: "vec_id",
            pg_type: "bytea",
            semantic_role: "dedupe_key",
            nullable: false,
            validator: "must_be_nonempty",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 8,
            column_name: "row_locator",
            pg_type: "bytea",
            semantic_role: "origin_node_locator",
            nullable: false,
            validator: "must_be_nonempty_and_opaque",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 9,
            column_name: "score",
            pg_type: "real",
            semantic_role: "candidate_score",
            nullable: false,
            validator: "must_be_finite",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 10,
            column_name: "protocol_version",
            pg_type: "text",
            semantic_role: "remote_endpoint_protocol",
            nullable: false,
            validator: "must_equal_ec_spire_remote_search_v1",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 11,
            column_name: "extension_version",
            pg_type: "text",
            semantic_role: "remote_extension_version",
            nullable: false,
            validator: "must_match_required_extension_version",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 12,
            column_name: "opclass_identity",
            pg_type: "text",
            semantic_role: "remote_opclass_identity",
            nullable: false,
            validator: "must_be_nonempty",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 13,
            column_name: "storage_format",
            pg_type: "text",
            semantic_role: "remote_storage_format",
            nullable: false,
            validator: "must_match_served_endpoint_identity",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 14,
            column_name: "assignment_payload_format",
            pg_type: "text",
            semantic_role: "remote_assignment_payload_format",
            nullable: false,
            validator: "must_match_served_endpoint_identity",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 15,
            column_name: "quantizer_profile",
            pg_type: "text",
            semantic_role: "remote_quantizer_profile",
            nullable: false,
            validator: "must_match_served_endpoint_identity",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 16,
            column_name: "scoring_profile",
            pg_type: "text",
            semantic_role: "remote_scoring_profile",
            nullable: false,
            validator: "must_match_served_endpoint_identity",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 17,
            column_name: "profile_fingerprint",
            pg_type: "text",
            semantic_role: "remote_quantizer_index_fingerprint",
            nullable: false,
            validator: "must_be_nonempty_and_match_served_endpoint_identity",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 18,
            column_name: "endpoint_status",
            pg_type: "text",
            semantic_role: "remote_endpoint_identity_status",
            nullable: false,
            validator: "must_be_ready_before_production_merge",
        },
    ]
}

pub(crate) fn remote_search_endpoint_contract_rows(
) -> Vec<SpireRemoteSearchEndpointContractRow> {
    vec![
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 1,
            contract_item: "endpoint_function",
            contract_value: SPIRE_REMOTE_ENDPOINT_SEARCH,
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_be_registered_strict_pg_extern_endpoint",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 2,
            contract_item: "protocol_version",
            contract_value: SPIRE_REMOTE_CANDIDATE_FORMAT_V1,
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_match_libpq_candidate_format_v1",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 3,
            contract_item: "request_contract",
            contract_value: "remote_index_oid,requested_epoch,query,selected_pids,top_k,consistency_mode",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_match_remote_search_libpq_parameter_contract",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 4,
            contract_item: "response_contract",
            contract_value: "served_epoch,node_id,pid,object_version,row_index,assignment_flags,vec_id,row_locator,score,protocol_version,extension_version,opclass_identity,storage_format,assignment_payload_format,quantizer_profile,scoring_profile,profile_fingerprint,endpoint_status",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_match_remote_search_libpq_result_contract",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 5,
            contract_item: "tuple_transport_capabilities",
            contract_value: "pg_binary_attr_v1",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_advertise_pg_binary_attr_v1_before_custom_scan_typed_receive",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 6,
            contract_item: "tuple_transport_default",
            contract_value: "pg_binary_attr_v1",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_prefer_typed_tuple_transport_when_advertised",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 7,
            contract_item: "tuple_transport_status",
            contract_value: "ready",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_be_ready_before_typed_custom_scan_receive",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 8,
            contract_item: "selected_pid_semantics",
            contract_value: "selected_leaf_pid_set_with_leaf_derived_delta_rows",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "candidate_pid_must_be_selected_leaf_or_leaf_derived_delta_pid",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 9,
            contract_item: "quantizer_family",
            contract_value: "rabitq_only_pq_and_pqfastscan_reserved",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_reject_unsupported_quantizer_families_until_implemented",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 10,
            contract_item: "extension_version_binding",
            contract_value: env!("CARGO_PKG_VERSION"),
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "remote_node_capability_plan_must_match_required_extension_version",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 11,
            contract_item: "scoring_option_binding",
            contract_value: "fixed_index_profile_explicit_request_options_pending",
            status: SPIRE_REMOTE_STATUS_REQUIRES_SCORING_OPTION_BINDING,
            validator: "request_must_bind_scoring_and_rerank_options_before_production_merge",
            recommendation: "add explicit scoring/rerank option fields or a stable served scoring profile binding",
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 12,
            contract_item: "quantizer_index_fingerprint_binding",
            contract_value: "rabitq_profile,code_length,training_stat_fingerprint,storage_format",
            status: SPIRE_REMOTE_STATUS_REQUIRES_FINGERPRINT_BINDING,
            validator: "candidate_batch_must_bind_served_quantizer_index_fingerprint",
            recommendation: "bind fingerprint fields before accepting cross-node remote scores",
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 13,
            contract_item: "opclass_binary_binding",
            contract_value: "opclass_identity_and_binary_score_semantics",
            status: SPIRE_REMOTE_STATUS_REQUIRES_OPCLASS_BINDING,
            validator: "candidate_batch_must_bind_opclass_score_semantics",
            recommendation: "bind opclass identity before accepting remote scores from mixed binaries",
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 14,
            contract_item: "direct_sql_endpoint_status_policy",
            contract_value: "ec_spire_remote_search_exposes_non_ready_endpoint_rows_for_diagnostics_libpq_receive_accepts_ready_only",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_not_treat_direct_sql_rows_as_mergeable_without_libpq_receive_validation",
            recommendation: "use libpq executor readiness surfaces before production remote merge",
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 15,
            contract_item: "remote_heap_candidate_endpoint_identity_preflight",
            contract_value: "libpq_heap_candidate_executor_validates_ec_spire_remote_search_endpoint_identity_before_origin_node_heap_rows",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_validate_ready_endpoint_identity_before_remote_heap_candidate_merge",
            recommendation: SPIRE_REMOTE_NONE,
        },
    ]
}

fn remote_search_assignment_payload_format_name(
    format: quantizer::SpireAssignmentPayloadFormat,
) -> &'static str {
    match format {
        quantizer::SpireAssignmentPayloadFormat::TurboQuant => "turboquant",
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => "pq_fastscan",
        quantizer::SpireAssignmentPayloadFormat::RaBitQ => "rabitq",
    }
}

fn remote_search_endpoint_quantizer_profile(
    format: quantizer::SpireAssignmentPayloadFormat,
) -> &'static str {
    match format {
        quantizer::SpireAssignmentPayloadFormat::RaBitQ => "rabitq_v1",
        quantizer::SpireAssignmentPayloadFormat::TurboQuant => "unsupported_turboquant",
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => "unsupported_pq_fastscan",
    }
}

fn remote_search_endpoint_opclass_identity(index_relid: pg_sys::Oid) -> Result<String, String> {
    let sql = format!(
        "SELECT opc.opcname::text AS opclass_identity \
           FROM pg_index idx \
           JOIN pg_opclass opc ON opc.oid = idx.indclass[0] \
          WHERE idx.indexrelid = '{}'::oid",
        u32::from(index_relid)
    );

    Spi::connect(|client| {
        client
            .select(sql.as_str(), None, &[])
            .map_err(|e| format!("ec_spire endpoint identity opclass read failed: {e}"))?
            .map(|row| {
                row["opclass_identity"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("ec_spire endpoint identity opclass decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire endpoint identity opclass is null".to_owned()
                    })
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or_else(|| "unknown".to_owned()))
    })
}

fn remote_search_endpoint_generation_identity(index_relid: pg_sys::Oid) -> Result<String, String> {
    let sql = format!(
        "SELECT pg_relation_filenode('{}'::oid)::text AS generation_identity",
        u32::from(index_relid)
    );

    Spi::connect(|client| {
        client
            .select(sql.as_str(), None, &[])
            .map_err(|e| format!("ec_spire endpoint identity generation read failed: {e}"))?
            .map(|row| {
                row["generation_identity"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("ec_spire endpoint identity generation decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire endpoint identity generation is null".to_owned()
                    })
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or_else(|| "unknown".to_owned()))
    })
}

fn remote_search_stable_fingerprint(parts: &[String]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for part in parts {
        for byte in part.as_bytes().iter().copied().chain(std::iter::once(0)) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("{hash:016x}")
}

fn remote_search_candidate_endpoint_text(
    row: &postgres::Row,
    column: &str,
) -> Result<String, String> {
    row.try_get::<_, String>(column).map_err(|_| {
        format!("ec_spire remote search executor {column} endpoint identity decode failed")
    })
}

fn validate_remote_search_endpoint_identity_fields(
    protocol_version: &str,
    extension_version: &str,
    opclass_identity: &str,
    storage_format: &str,
    assignment_payload_format: &str,
    quantizer_profile: &str,
    scoring_profile: &str,
    profile_fingerprint: &str,
    endpoint_status: &str,
) -> Result<(), String> {
    if protocol_version != SPIRE_REMOTE_CANDIDATE_FORMAT_V1 {
        return Err(format!(
            "ec_spire remote search executor protocol_version {protocol_version} does not match {}",
            SPIRE_REMOTE_CANDIDATE_FORMAT_V1
        ));
    }

    if extension_version != env!("CARGO_PKG_VERSION") {
        return Err(format!(
            "ec_spire remote search executor extension_version {extension_version} does not match {}",
            env!("CARGO_PKG_VERSION")
        ));
    }

    for (column, value) in [
        ("opclass_identity", opclass_identity),
        ("storage_format", storage_format),
        ("assignment_payload_format", assignment_payload_format),
        ("quantizer_profile", quantizer_profile),
        ("scoring_profile", scoring_profile),
        ("profile_fingerprint", profile_fingerprint),
    ] {
        if value.is_empty() {
            return Err(format!(
                "ec_spire remote search executor {column} endpoint identity is empty"
            ));
        }
    }

    if endpoint_status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire remote search executor endpoint_status {endpoint_status} is not ready"
        ));
    }

    Ok(())
}

fn validate_remote_search_candidate_endpoint_identity(
    row: &postgres::Row,
) -> Result<Vec<u8>, String> {
    let protocol_version = remote_search_candidate_endpoint_text(row, "protocol_version")?;
    let extension_version = remote_search_candidate_endpoint_text(row, "extension_version")?;
    let opclass_identity = remote_search_candidate_endpoint_text(row, "opclass_identity")?;
    let storage_format = remote_search_candidate_endpoint_text(row, "storage_format")?;
    let assignment_payload_format =
        remote_search_candidate_endpoint_text(row, "assignment_payload_format")?;
    let quantizer_profile = remote_search_candidate_endpoint_text(row, "quantizer_profile")?;
    let scoring_profile = remote_search_candidate_endpoint_text(row, "scoring_profile")?;
    let profile_fingerprint = remote_search_candidate_endpoint_text(row, "profile_fingerprint")?;
    let endpoint_status = remote_search_candidate_endpoint_text(row, "endpoint_status")?;

    validate_remote_search_endpoint_identity_fields(
        &protocol_version,
        &extension_version,
        &opclass_identity,
        &storage_format,
        &assignment_payload_format,
        &quantizer_profile,
        &scoring_profile,
        &profile_fingerprint,
        &endpoint_status,
    )?;
    remote_search_endpoint_profile_fingerprint_bytes(&profile_fingerprint)
}

fn remote_search_endpoint_profile_fingerprint_bytes(
    profile_fingerprint: &str,
) -> Result<Vec<u8>, String> {
    if profile_fingerprint.len() % 2 != 0 {
        return Err(
            "ec_spire remote search executor profile_fingerprint endpoint identity has invalid hex length"
                .to_owned(),
        );
    }

    (0..profile_fingerprint.len())
        .step_by(2)
        .map(|offset| {
            u8::from_str_radix(&profile_fingerprint[offset..offset + 2], 16).map_err(|_| {
                "ec_spire remote search executor profile_fingerprint endpoint identity is not hex"
                    .to_owned()
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteValidatedEndpointIdentity {
    protocol_version: String,
    extension_version: String,
    opclass_identity: String,
    storage_format: String,
    assignment_payload_format: String,
    quantizer_profile: String,
    scoring_profile: String,
    tuple_transport_capabilities: Vec<String>,
    tuple_transport_default: String,
    tuple_transport_status: String,
    profile_fingerprint: String,
    profile_fingerprint_bytes: Vec<u8>,
}

impl SpireRemoteValidatedEndpointIdentity {
    fn prefers_typed_tuple_transport(&self) -> bool {
        remote_endpoint_prefers_typed_tuple_transport(
            self.tuple_transport_status.as_str(),
            self.tuple_transport_default.as_str(),
            &self.tuple_transport_capabilities,
            options::current_session_remote_tuple_transport(),
        )
    }
}

fn remote_endpoint_prefers_typed_tuple_transport(
    tuple_transport_status: &str,
    tuple_transport_default: &str,
    tuple_transport_capabilities: &[String],
    session_transport: options::SpireRemoteTupleTransportGuc,
) -> bool {
    if tuple_transport_status != SPIRE_REMOTE_STATUS_READY
        || !tuple_transport_capabilities
            .iter()
            .any(|capability| capability == SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1)
    {
        return false;
    }
    match session_transport {
        options::SpireRemoteTupleTransportGuc::Auto => {
            tuple_transport_default == SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1
        }
        options::SpireRemoteTupleTransportGuc::JsonTuplePayloadV1 => false,
        options::SpireRemoteTupleTransportGuc::PgBinaryAttrV1 => true,
    }
}

#[cfg(test)]
mod remote_tuple_transport_tests {
    use super::*;

    fn pg_binary_capabilities() -> Vec<String> {
        vec![SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1.to_owned()]
    }

    #[test]
    fn remote_tuple_transport_auto_uses_endpoint_default() {
        assert!(remote_endpoint_prefers_typed_tuple_transport(
            SPIRE_REMOTE_STATUS_READY,
            SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1,
            &pg_binary_capabilities(),
            options::SpireRemoteTupleTransportGuc::Auto,
        ));
        assert!(!remote_endpoint_prefers_typed_tuple_transport(
            SPIRE_REMOTE_STATUS_READY,
            "json_tuple_payload_v1",
            &pg_binary_capabilities(),
            options::SpireRemoteTupleTransportGuc::Auto,
        ));
    }

    #[test]
    fn remote_tuple_transport_session_override_keeps_capability_gate() {
        assert!(!remote_endpoint_prefers_typed_tuple_transport(
            SPIRE_REMOTE_STATUS_READY,
            SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1,
            &pg_binary_capabilities(),
            options::SpireRemoteTupleTransportGuc::JsonTuplePayloadV1,
        ));
        assert!(remote_endpoint_prefers_typed_tuple_transport(
            SPIRE_REMOTE_STATUS_READY,
            "json_tuple_payload_v1",
            &pg_binary_capabilities(),
            options::SpireRemoteTupleTransportGuc::PgBinaryAttrV1,
        ));
        assert!(!remote_endpoint_prefers_typed_tuple_transport(
            SPIRE_REMOTE_STATUS_READY,
            "json_tuple_payload_v1",
            &[],
            options::SpireRemoteTupleTransportGuc::PgBinaryAttrV1,
        ));
    }
}

fn validate_remote_search_endpoint_identity_row(
    row: &postgres::Row,
) -> Result<SpireRemoteValidatedEndpointIdentity, String> {
    let protocol_version = remote_search_candidate_endpoint_text(row, "protocol_version")?;
    let extension_version = remote_search_candidate_endpoint_text(row, "extension_version")?;
    let opclass_identity = remote_search_candidate_endpoint_text(row, "opclass_identity")?;
    let storage_format = remote_search_candidate_endpoint_text(row, "storage_format")?;
    let assignment_payload_format =
        remote_search_candidate_endpoint_text(row, "assignment_payload_format")?;
    let quantizer_profile = remote_search_candidate_endpoint_text(row, "quantizer_profile")?;
    let scoring_profile = remote_search_candidate_endpoint_text(row, "scoring_profile")?;
    let profile_fingerprint = remote_search_candidate_endpoint_text(row, "profile_fingerprint")?;
    let endpoint_status = remote_search_candidate_endpoint_text(row, "status")?;
    let tuple_transport_capabilities = row
        .try_get::<_, Vec<String>>("tuple_transport_capabilities")
        .unwrap_or_default();
    let tuple_transport_default = row
        .try_get::<_, String>("tuple_transport_default")
        .unwrap_or_else(|_| "json_tuple_payload_v1".to_owned());
    let tuple_transport_status = row
        .try_get::<_, String>("tuple_transport_status")
        .unwrap_or_else(|_| SPIRE_REMOTE_STATUS_READY.to_owned());

    validate_remote_search_endpoint_identity_fields(
        &protocol_version,
        &extension_version,
        &opclass_identity,
        &storage_format,
        &assignment_payload_format,
        &quantizer_profile,
        &scoring_profile,
        &profile_fingerprint,
        &endpoint_status,
    )?;
    let profile_fingerprint_bytes =
        remote_search_endpoint_profile_fingerprint_bytes(&profile_fingerprint)?;
    Ok(SpireRemoteValidatedEndpointIdentity {
        protocol_version,
        extension_version,
        opclass_identity,
        storage_format,
        assignment_payload_format,
        quantizer_profile,
        scoring_profile,
        tuple_transport_capabilities,
        tuple_transport_default,
        tuple_transport_status,
        profile_fingerprint,
        profile_fingerprint_bytes,
    })
}

pub(crate) unsafe fn remote_search_endpoint_identity_row(
    index_relation: pg_sys::Relation,
) -> SpireRemoteSearchEndpointIdentityRow {
    let result = (|| -> Result<SpireRemoteSearchEndpointIdentityRow, String> {
        let relation_options = unsafe { options::relation_options(index_relation) };
        let assignment_payload_format = relation_options.assignment_payload_format();
        let assignment_payload_format_name =
            remote_search_assignment_payload_format_name(assignment_payload_format);
        let quantizer_profile =
            remote_search_endpoint_quantizer_profile(assignment_payload_format);
        let opclass_identity =
            remote_search_endpoint_opclass_identity(unsafe { (*index_relation).rd_id })?;
        let generation_identity =
            remote_search_endpoint_generation_identity(unsafe { (*index_relation).rd_id })?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let scoring_profile = "inner_product_score_v1";
        let storage_format = relation_options.storage_format.reloption_name();
        let profile_fingerprint = remote_search_stable_fingerprint(&[
            SPIRE_REMOTE_CANDIDATE_FORMAT_V1.to_owned(),
            env!("CARGO_PKG_VERSION").to_owned(),
            opclass_identity.clone(),
            storage_format.to_owned(),
            assignment_payload_format_name.to_owned(),
            quantizer_profile.to_owned(),
            scoring_profile.to_owned(),
            relation_options.nlists.to_string(),
            relation_options.recursive_fanout.to_string(),
            relation_options.training_sample_rows.to_string(),
            relation_options.seed.to_string(),
            relation_options.pq_group_size.to_string(),
            root_control.active_epoch.to_string(),
            generation_identity,
        ]);

        let (status, recommendation) =
            if assignment_payload_format == quantizer::SpireAssignmentPayloadFormat::RaBitQ {
                (SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
            } else {
                (
                    SPIRE_REMOTE_STATUS_REQUIRES_RABITQ_STORAGE_FORMAT,
                    "create or reindex the remote-serving SPIRE index with storage_format = 'rabitq'",
                )
            };

        Ok(SpireRemoteSearchEndpointIdentityRow {
            protocol_version: SPIRE_REMOTE_CANDIDATE_FORMAT_V1,
            extension_version: env!("CARGO_PKG_VERSION"),
            opclass_identity,
            storage_format,
            assignment_payload_format: assignment_payload_format_name,
            quantizer_profile,
            scoring_profile,
            tuple_transport_capabilities: vec![
                SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1.to_owned(),
            ],
            tuple_transport_default: SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1,
            tuple_transport_status: SPIRE_REMOTE_STATUS_READY,
            profile_fingerprint,
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_conninfo_secret_value(conninfo_secret_name: &str) -> Result<String, String> {
    let provider_lookup_key = remote_conninfo_secret_provider_lookup_key(conninfo_secret_name)?;
    match std::env::var(provider_lookup_key) {
        Ok(conninfo) if !conninfo.is_empty() => Ok(conninfo),
        Ok(_) => Err("conninfo_secret_empty".to_owned()),
        Err(_) => Err("conninfo_secret_missing".to_owned()),
    }
}

pub(crate) fn remote_prepared_transaction_registration_warning(
    conninfo_secret_name: &str,
    node_id: i32,
) -> Option<String> {
    let node_id = u32::try_from(node_id).ok()?;
    let conninfo = match remote_conninfo_secret_value(conninfo_secret_name) {
        Ok(conninfo) => conninfo,
        Err(status) => {
            return Some(format!(
                "ec_spire_register_remote_node_descriptor skipped remote \
                 max_prepared_transactions preflight for node_id {node_id}: {status}; \
                 resolve conninfo_secret_name before enabling coordinator-routed writes"
            ));
        }
    };
    let mut client = match remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        node_id,
        "remote node descriptor max_prepared_transactions preflight",
    ) {
        Ok(client) => client,
        Err(error) => {
            return Some(format!(
                "ec_spire_register_remote_node_descriptor could not check remote \
                 max_prepared_transactions for node_id {node_id}: {error}"
            ));
        }
    };
    let setting = match client.query_one("SHOW max_prepared_transactions", &[]) {
        Ok(row) => row
            .try_get::<_, String>(0)
            .unwrap_or_else(|_| "ec_spire_max_prepared_transactions_decode_failed".to_owned()),
        Err(error) => {
            return Some(format!(
                "ec_spire_register_remote_node_descriptor could not read remote \
                 max_prepared_transactions for node_id {node_id}: {error}"
            ));
        }
    };
    let value = match setting.parse::<i64>() {
        Ok(value) => value,
        Err(_) => {
            return Some(format!(
                "ec_spire_register_remote_node_descriptor could not parse remote \
                 max_prepared_transactions value {setting:?} for node_id {node_id}"
            ));
        }
    };
    if value <= 0 {
        Some(format!(
            "ec_spire_register_remote_node_descriptor remote node_id {node_id} reports \
             max_prepared_transactions = {value}; coordinator-routed SPIRE writes require \
             max_prepared_transactions > 0 and enough free prepared transaction slots"
        ))
    } else {
        None
    }
}

pub(crate) fn remote_search_libpq_connect_with_session_timeouts(
    conninfo: &str,
    node_id: u32,
    context: &str,
) -> Result<postgres::Client, String> {
    let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
    let mut config = conninfo
        .parse::<postgres::Config>()
        .map_err(|_| format!("ec_spire {context} conninfo parse failed for node_id {node_id}"))?;
    if limits.connect_timeout_ms > 0 {
        config.connect_timeout(std::time::Duration::from_millis(limits.connect_timeout_ms));
    }
    let mut client = config
        .connect(postgres::NoTls)
        .map_err(|_| format!("ec_spire {context} failed to open connection for node_id {node_id}"))?;
    if limits.statement_timeout_ms > 0 {
        let sql = format!("SET statement_timeout = {}", limits.statement_timeout_ms);
        client.batch_execute(&sql).map_err(|_| {
            format!("ec_spire {context} failed to configure statement_timeout for node_id {node_id}")
        })?;
    }
    Ok(client)
}

const SPIRE_REMOTE_SEARCH_LIBPQ_GLOBAL_LOCK_CLASS_BASE: i32 = 730_000_000;
const SPIRE_REMOTE_SEARCH_LIBPQ_NODE_LOCK_CLASS_BASE: i32 = 731_000_000;
#[cfg(any(test, feature = "pg_test"))]
const SPIRE_REMOTE_SEARCH_LIBPQ_GOVERNANCE_TEST_NAMESPACE_STRIDE: i32 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireRemoteSearchLibpqGovernanceLockKey {
    class_id: i32,
    object_id: i32,
}

#[derive(Debug, Default)]
struct SpireRemoteSearchLibpqGovernancePermit {
    locks: Vec<SpireRemoteSearchLibpqGovernanceLockKey>,
}

impl Drop for SpireRemoteSearchLibpqGovernancePermit {
    fn drop(&mut self) {
        for key in self.locks.iter().rev() {
            let _ = remote_search_libpq_advisory_unlock(*key);
        }
    }
}

fn remote_search_libpq_governance_lock_key(
    class_base: i32,
    object_id: i32,
    slot: u64,
) -> Result<SpireRemoteSearchLibpqGovernanceLockKey, String> {
    let slot = i32::try_from(slot)
        .map_err(|_| "ec_spire remote search executor governance slot exceeds i32".to_owned())?;
    let class_base = remote_search_libpq_governance_class_base(class_base)?;
    let class_id = class_base.checked_add(slot).ok_or_else(|| {
        "ec_spire remote search executor governance advisory lock class overflow".to_owned()
    })?;
    Ok(SpireRemoteSearchLibpqGovernanceLockKey {
        class_id,
        object_id,
    })
}

fn remote_search_libpq_governance_class_base(class_base: i32) -> Result<i32, String> {
    let namespace = options::current_session_remote_search_governance_test_namespace();
    if namespace == 0 {
        return Ok(class_base);
    }

    #[cfg(any(test, feature = "pg_test"))]
    {
        let offset = namespace
            .checked_mul(SPIRE_REMOTE_SEARCH_LIBPQ_GOVERNANCE_TEST_NAMESPACE_STRIDE)
            .ok_or_else(|| {
                "ec_spire remote search executor governance test namespace overflow".to_owned()
            })?;
        return class_base.checked_add(offset).ok_or_else(|| {
            "ec_spire remote search executor governance test class overflow".to_owned()
        });
    }

    #[cfg(not(any(test, feature = "pg_test")))]
    {
        Err("ec_spire remote search executor governance test namespace is unavailable".to_owned())
    }
}

fn remote_search_libpq_advisory_lock_result(
    function_name: &str,
    key: SpireRemoteSearchLibpqGovernanceLockKey,
) -> Result<bool, String> {
    Spi::get_one::<bool>(&format!(
        "SELECT {function_name}({}, {})",
        key.class_id, key.object_id
    ))
    .map_err(|e| {
        format!("ec_spire remote search executor governance advisory lock query failed: {e}")
    })?
    .ok_or_else(|| {
        "ec_spire remote search executor governance advisory lock returned null".to_owned()
    })
}

fn remote_search_libpq_try_advisory_lock(
    key: SpireRemoteSearchLibpqGovernanceLockKey,
) -> Result<bool, String> {
    remote_search_libpq_advisory_lock_result("pg_try_advisory_lock", key)
}

fn remote_search_libpq_advisory_unlock(
    key: SpireRemoteSearchLibpqGovernanceLockKey,
) -> Result<bool, String> {
    remote_search_libpq_advisory_lock_result("pg_advisory_unlock", key)
}

fn remote_search_libpq_try_governance_slot(
    class_base: i32,
    object_id: i32,
    slot_count: u64,
) -> Result<Option<SpireRemoteSearchLibpqGovernanceLockKey>, String> {
    for slot in 0..slot_count {
        let key = remote_search_libpq_governance_lock_key(class_base, object_id, slot)?;
        if remote_search_libpq_try_advisory_lock(key)? {
            return Ok(Some(key));
        }
    }
    Ok(None)
}

fn remote_search_libpq_executor_governance_permit(
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
) -> Result<SpireRemoteSearchLibpqGovernancePermit, String> {
    remote_search_libpq_executor_governance_permit_for_node(row.node_id)
}

fn remote_search_libpq_executor_governance_permit_for_node(
    node_id: u32,
) -> Result<SpireRemoteSearchLibpqGovernancePermit, String> {
    let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
    let mut permit = SpireRemoteSearchLibpqGovernancePermit::default();

    if limits.has_concurrent_dispatch_cap() {
        let key = remote_search_libpq_try_governance_slot(
            SPIRE_REMOTE_SEARCH_LIBPQ_GLOBAL_LOCK_CLASS_BASE,
            0,
            limits.max_concurrent_dispatches,
        )?
        .ok_or_else(|| {
            format!(
                "ec_spire remote search executor remote_executor_overload global concurrency cap {} is saturated",
                limits.max_concurrent_dispatches
            )
        })?;
        permit.locks.push(key);
    }

    if limits.has_concurrent_dispatch_per_node_cap() {
        let key = remote_search_libpq_try_governance_slot(
            SPIRE_REMOTE_SEARCH_LIBPQ_NODE_LOCK_CLASS_BASE,
            i32::from_ne_bytes(node_id.to_ne_bytes()),
            limits.max_concurrent_dispatches_per_node,
        )?
        .ok_or_else(|| {
            format!(
                "ec_spire remote search executor remote_executor_overload per-node concurrency cap {} is saturated for node_id {}",
                limits.max_concurrent_dispatches_per_node, node_id
            )
        })?;
        permit.locks.push(key);
    }

    Ok(permit)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_libpq_global_governance_advisory_key_for_test(
    slot: u64,
) -> (i32, i32) {
    let key = remote_search_libpq_governance_lock_key(
        SPIRE_REMOTE_SEARCH_LIBPQ_GLOBAL_LOCK_CLASS_BASE,
        0,
        slot,
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    (key.class_id, key.object_id)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_search_libpq_node_governance_advisory_key_for_test(
    node_id: u32,
    slot: u64,
) -> (i32, i32) {
    let key = remote_search_libpq_governance_lock_key(
        SPIRE_REMOTE_SEARCH_LIBPQ_NODE_LOCK_CLASS_BASE,
        i32::from_ne_bytes(node_id.to_ne_bytes()),
        slot,
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    (key.class_id, key.object_id)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SpireRemoteEndpointIdentityCacheKey {
    coordinator_index_oid: u32,
    node_id: u32,
    remote_index_regclass: String,
    remote_index_oid: u32,
    descriptor_generation: u64,
    remote_index_identity: Vec<u8>,
    served_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteEndpointIdentityCacheEntry {
    protocol_version: String,
    extension_version: String,
    opclass_identity: String,
    storage_format: String,
    assignment_payload_format: String,
    quantizer_profile: String,
    scoring_profile: String,
    profile_fingerprint: String,
}

impl From<SpireRemoteValidatedEndpointIdentity> for SpireRemoteEndpointIdentityCacheEntry {
    fn from(identity: SpireRemoteValidatedEndpointIdentity) -> Self {
        Self {
            protocol_version: identity.protocol_version,
            extension_version: identity.extension_version,
            opclass_identity: identity.opclass_identity,
            storage_format: identity.storage_format,
            assignment_payload_format: identity.assignment_payload_format,
            quantizer_profile: identity.quantizer_profile,
            scoring_profile: identity.scoring_profile,
            profile_fingerprint: identity.profile_fingerprint,
        }
    }
}

#[derive(Debug, Default)]
struct SpireRemoteSearchLibpqExecutorState {
    endpoint_identity_cache:
        HashMap<SpireRemoteEndpointIdentityCacheKey, SpireRemoteEndpointIdentityCacheEntry>,
    endpoint_identity_query_count: u64,
    endpoint_identity_cache_hit_count: u64,
    endpoint_identity_cache_miss_count: u64,
}

impl SpireRemoteSearchLibpqExecutorState {
    fn increment_counter(counter: &mut u64, counter_name: &str) -> Result<(), String> {
        *counter = counter.checked_add(1).ok_or_else(|| {
            format!("ec_spire remote search libpq executor {counter_name} overflow")
        })?;
        Ok(())
    }

    fn endpoint_identity_cache_entry_count(&self) -> Result<u64, String> {
        u64::try_from(self.endpoint_identity_cache.len()).map_err(|_| {
            "ec_spire remote search libpq executor endpoint identity cache size exceeds u64"
                .to_owned()
        })
    }

    fn endpoint_identity_query_count(&self) -> u64 {
        self.endpoint_identity_query_count
    }

    fn endpoint_identity_cache_hit_count(&self) -> u64 {
        self.endpoint_identity_cache_hit_count
    }

    fn endpoint_identity_cache_miss_count(&self) -> u64 {
        self.endpoint_identity_cache_miss_count
    }

    fn lookup_endpoint_identity(
        &mut self,
        key: &SpireRemoteEndpointIdentityCacheKey,
    ) -> Result<bool, String> {
        if self.endpoint_identity_cache.contains_key(key) {
            Self::increment_counter(
                &mut self.endpoint_identity_cache_hit_count,
                "endpoint identity cache hit count",
            )?;
            Ok(true)
        } else {
            Self::increment_counter(
                &mut self.endpoint_identity_cache_miss_count,
                "endpoint identity cache miss count",
            )?;
            Ok(false)
        }
    }

    fn record_endpoint_identity_query(&mut self) -> Result<(), String> {
        Self::increment_counter(
            &mut self.endpoint_identity_query_count,
            "endpoint identity query count",
        )
    }

    fn insert_endpoint_identity(
        &mut self,
        key: SpireRemoteEndpointIdentityCacheKey,
        identity: SpireRemoteValidatedEndpointIdentity,
    ) {
        self.endpoint_identity_cache.insert(key, identity.into());
    }
}

fn remote_search_endpoint_identity_cache_key(
    coordinator_index_oid: pg_sys::Oid,
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
    remote_index_oid: u32,
) -> SpireRemoteEndpointIdentityCacheKey {
    SpireRemoteEndpointIdentityCacheKey {
        coordinator_index_oid: u32::from(coordinator_index_oid),
        node_id: row.node_id,
        remote_index_regclass: row.remote_index_regclass.clone(),
        remote_index_oid,
        descriptor_generation: row.descriptor_generation,
        remote_index_identity: row.remote_index_identity.clone(),
        served_epoch: row.requested_epoch,
    }
}

fn validate_remote_search_libpq_endpoint_identity_for_dispatch(
    client: &mut postgres::Client,
    coordinator_index_oid: pg_sys::Oid,
    remote_index_oid: u32,
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<(), String> {
    let cache_key = remote_search_endpoint_identity_cache_key(coordinator_index_oid, row, remote_index_oid);
    if executor_state.lookup_endpoint_identity(&cache_key)? {
        return Ok(());
    }

    executor_state.record_endpoint_identity_query()?;
    let endpoint_identity_row = client
        .query_one(
            SPIRE_REMOTE_SEARCH_ENDPOINT_IDENTITY_SQL_TEMPLATE,
            &[&remote_index_oid],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote search libpq executor endpoint identity query failed for node_id {}",
                row.node_id
            )
        })?;
    let endpoint_identity = validate_remote_search_endpoint_identity_row(&endpoint_identity_row)?;
    if endpoint_identity.profile_fingerprint_bytes.as_slice() != row.remote_index_identity.as_slice() {
        return Err(format!(
            "ec_spire remote search executor remote_index_identity does not match endpoint profile_fingerprint for node_id {}",
            row.node_id
        ));
    }
    executor_state.insert_endpoint_identity(cache_key, endpoint_identity);
    Ok(())
}

fn decode_remote_search_candidate_pg_row(
    row: &postgres::Row,
    expected_node_id: u32,
    validate_endpoint_identity: bool,
    expected_remote_index_identity: Option<&[u8]>,
) -> Result<SpireRemoteSearchCandidateRow, String> {
    let served_epoch = row
        .try_get::<_, i64>("served_epoch")
        .map_err(|_| "ec_spire remote search executor served_epoch decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value)
                .map_err(|_| "ec_spire remote search executor served_epoch is negative".to_owned())
        })?;
    let remote_node_id = row
        .try_get::<_, i64>("node_id")
        .map_err(|_| "ec_spire remote search executor node_id decode failed".to_owned())
        .and_then(|value| {
            u32::try_from(value)
                .map_err(|_| "ec_spire remote search executor node_id is invalid".to_owned())
        })?;
    let node_id = if remote_node_id == meta::SPIRE_LOCAL_NODE_ID {
        expected_node_id
    } else {
        remote_node_id
    };
    let pid = row
        .try_get::<_, i64>("pid")
        .map_err(|_| "ec_spire remote search executor pid decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value)
                .map_err(|_| "ec_spire remote search executor pid is negative".to_owned())
        })?;
    let object_version = row
        .try_get::<_, i64>("object_version")
        .map_err(|_| "ec_spire remote search executor object_version decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value).map_err(|_| {
                "ec_spire remote search executor object_version is negative".to_owned()
            })
        })?;
    let row_index = row
        .try_get::<_, i64>("row_index")
        .map_err(|_| "ec_spire remote search executor row_index decode failed".to_owned())
        .and_then(|value| {
            u32::try_from(value)
                .map_err(|_| "ec_spire remote search executor row_index is invalid".to_owned())
        })?;
    let assignment_flags = row
        .try_get::<_, i16>("assignment_flags")
        .map_err(|_| "ec_spire remote search executor assignment_flags decode failed".to_owned())
        .and_then(|value| {
            u16::try_from(value).map_err(|_| {
                "ec_spire remote search executor assignment_flags is negative".to_owned()
            })
        })?;
    let vec_id = row
        .try_get::<_, Vec<u8>>("vec_id")
        .map_err(|_| "ec_spire remote search executor vec_id decode failed".to_owned())?;
    let row_locator = row
        .try_get::<_, Vec<u8>>("row_locator")
        .map_err(|_| "ec_spire remote search executor row_locator decode failed".to_owned())?;
    let score = row
        .try_get::<_, f32>("score")
        .map_err(|_| "ec_spire remote search executor score decode failed".to_owned())?;
    if validate_endpoint_identity {
        let profile_fingerprint_bytes = validate_remote_search_candidate_endpoint_identity(row)?;
        if let Some(expected_remote_index_identity) = expected_remote_index_identity {
            if profile_fingerprint_bytes.as_slice() != expected_remote_index_identity {
                return Err(
                    "ec_spire remote search executor remote_index_identity does not match candidate profile_fingerprint"
                        .to_owned(),
                );
            }
        }
    }

    Ok(SpireRemoteSearchCandidateRow {
        served_epoch,
        node_id,
        pid,
        object_version,
        row_index,
        assignment_flags,
        vec_id,
        row_locator,
        score,
    })
}

fn decode_remote_search_heap_candidate_pg_row(
    row: &postgres::Row,
    expected_requested_epoch: u64,
    expected_node_id: u32,
) -> Result<SpireRemoteSearchLocalHeapCandidateRow, String> {
    let requested_epoch = row
        .try_get::<_, i64>("requested_epoch")
        .map_err(|_| {
            "ec_spire remote heap executor requested_epoch decode failed".to_owned()
        })
        .and_then(|value| {
            u64::try_from(value).map_err(|_| {
                "ec_spire remote heap executor requested_epoch is negative".to_owned()
            })
        })?;
    if requested_epoch != expected_requested_epoch {
        return Err(format!(
            "ec_spire remote heap executor requested_epoch {requested_epoch} does not match expected epoch {expected_requested_epoch}"
        ));
    }
    let candidate = decode_remote_search_candidate_pg_row(row, expected_node_id, false, None)?;
    let heap_block = row
        .try_get::<_, i64>("heap_block")
        .map_err(|_| "ec_spire remote heap executor heap_block decode failed".to_owned())
        .and_then(|value| {
            u32::try_from(value)
                .map_err(|_| "ec_spire remote heap executor heap_block is invalid".to_owned())
        })?;
    let heap_offset = row
        .try_get::<_, i32>("heap_offset")
        .map_err(|_| "ec_spire remote heap executor heap_offset decode failed".to_owned())
        .and_then(|value| {
            u16::try_from(value)
                .map_err(|_| "ec_spire remote heap executor heap_offset is invalid".to_owned())
        })?;
    let status = row
        .try_get::<_, String>("status")
        .map_err(|_| "ec_spire remote heap executor status decode failed".to_owned())?;
    if status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire remote heap executor returned non-ready heap candidate status {status}"
        ));
    }

    Ok(SpireRemoteSearchLocalHeapCandidateRow {
        requested_epoch,
        served_epoch: candidate.served_epoch,
        node_id: candidate.node_id,
        pid: candidate.pid,
        object_version: candidate.object_version,
        row_index: candidate.row_index,
        assignment_flags: candidate.assignment_flags,
        vec_id: candidate.vec_id,
        row_locator: candidate.row_locator,
        heap_block,
        heap_offset,
        score: candidate.score,
        heap_lookup_owner: SPIRE_REMOTE_HEAP_RESOLUTION,
        tuple_payload_json: row.try_get::<_, String>("tuple_payload_text").ok(),
        typed_tuple_payload: decode_remote_search_typed_tuple_payload_pg_row(row)?,
        tuple_payload_missing: row
            .try_get::<_, bool>("tuple_payload_missing")
            .unwrap_or(false),
        status: SPIRE_REMOTE_STATUS_READY,
    })
}

fn decode_remote_search_typed_tuple_payload_pg_row(
    row: &postgres::Row,
) -> Result<Option<SpireRemoteTypedTuplePayload>, String> {
    let Ok(payload_attnums) = row.try_get::<_, Vec<i16>>("payload_attnums") else {
        return Ok(None);
    };
    let payload_names = row
        .try_get::<_, Vec<String>>("payload_names")
        .map_err(|_| "ec_spire remote heap executor typed payload_names decode failed".to_owned())?;
    let payload_type_oids = row
        .try_get::<_, Vec<String>>("payload_type_oids")
        .map_err(|_| {
            "ec_spire remote heap executor typed payload_type_oids decode failed".to_owned()
        })?
        .into_iter()
        .map(|oid| {
            oid.parse::<u32>()
                .map(pg_sys::Oid::from)
                .map_err(|_| "ec_spire remote heap executor typed payload_type_oid is invalid".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let payload_typmods = row
        .try_get::<_, Vec<i32>>("payload_typmods")
        .map_err(|_| "ec_spire remote heap executor typed payload_typmods decode failed".to_owned())?;
    let payload_collations = match row.try_get::<_, Vec<String>>("payload_collations") {
        Ok(collations) => collations
            .into_iter()
            .map(|oid| {
                oid.parse::<u32>()
                    .map(pg_sys::Oid::from)
                    .map_err(|_| {
                        "ec_spire remote heap executor typed payload_collation is invalid"
                            .to_owned()
                    })
            })
            .collect::<Result<Vec<_>, _>>()?,
        Err(_) => vec![pg_sys::InvalidOid; payload_attnums.len()],
    };
    let payload_nulls = row
        .try_get::<_, Vec<bool>>("payload_nulls")
        .map_err(|_| "ec_spire remote heap executor typed payload_nulls decode failed".to_owned())?;
    let payload_values = row
        .try_get::<_, Vec<String>>("payload_values_hex")
        .map_err(|_| {
            "ec_spire remote heap executor typed payload_values_hex decode failed".to_owned()
        })?
        .into_iter()
        .map(|value| {
            hex::decode(&value).map_err(|_| {
                "ec_spire remote heap executor typed payload_values_hex is invalid".to_owned()
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let payload_formats = row
        .try_get::<_, Vec<String>>("payload_formats")
        .map_err(|_| {
            "ec_spire remote heap executor typed payload_formats decode failed".to_owned()
        })?;
    let tuple_transport = row
        .try_get::<_, String>("tuple_transport")
        .map_err(|_| "ec_spire remote heap executor tuple_transport decode failed".to_owned())?;
    let tuple_transport_status = row
        .try_get::<_, String>("tuple_transport_status")
        .map_err(|_| {
            "ec_spire remote heap executor tuple_transport_status decode failed".to_owned()
        })?;
    let payload_width = payload_attnums.len();
    for (label, width) in [
        ("payload_names", payload_names.len()),
        ("payload_type_oids", payload_type_oids.len()),
        ("payload_typmods", payload_typmods.len()),
        ("payload_collations", payload_collations.len()),
        ("payload_nulls", payload_nulls.len()),
        ("payload_values", payload_values.len()),
        ("payload_formats", payload_formats.len()),
    ] {
        if width != payload_width {
            return Err(format!(
                "ec_spire remote heap executor typed {label} width {width} does not match attnum width {payload_width}"
            ));
        }
    }
    if tuple_transport != SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1 {
        return Err(format!(
            "ec_spire remote heap executor unsupported tuple transport {tuple_transport}"
        ));
    }
    if tuple_transport_status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire remote heap executor tuple transport status {tuple_transport_status} is not ready"
        ));
    }
    if payload_formats
        .iter()
        .any(|format| format != SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1)
    {
        return Err(
            "ec_spire remote heap executor typed payload format mismatch".to_owned()
        );
    }

    Ok(Some(SpireRemoteTypedTuplePayload {
        payload_attnums,
        payload_names,
        payload_type_oids,
        payload_typmods,
        payload_collations,
        payload_nulls,
        payload_values,
        payload_formats,
        tuple_transport: SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1,
        tuple_transport_status: SPIRE_REMOTE_STATUS_READY,
    }))
}

fn remote_search_libpq_executor_candidates_for_dispatch(
    index_relid: pg_sys::Oid,
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
    query: &[f32],
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
    if row.dispatch_action != SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
        return Err(format!(
            "ec_spire remote search libpq executor dispatch for node_id {} is blocked with status {}",
            row.node_id, row.status
        ));
    }

    let _governance_permit = remote_search_libpq_executor_governance_permit(row)?;
    let conninfo = remote_conninfo_secret_value(&row.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire remote search libpq executor conninfo secret for node_id {} is not resolved: {status}",
            row.node_id
        )
    })?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        row.node_id,
        "remote search libpq executor",
    )?;
    let remote_index_oid = client
        .query_one(
            "SELECT to_regclass($1)::oid",
            &[&row.remote_index_regclass.as_str()],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote search libpq executor failed to resolve remote index for node_id {}",
                row.node_id
            )
        })?
        .try_get::<_, Option<u32>>(0)
        .map_err(|_| {
            format!(
                "ec_spire remote search libpq executor remote index oid decode failed for node_id {}",
                row.node_id
            )
        })?
        .ok_or_else(|| {
            format!(
                "ec_spire remote search libpq executor remote index is missing for node_id {}",
                row.node_id
            )
        })?;
    validate_remote_search_libpq_endpoint_identity_for_dispatch(
        &mut client,
        index_relid,
        remote_index_oid,
        row,
        executor_state,
    )?;
    let requested_epoch = i64::try_from(row.requested_epoch)
        .map_err(|_| "ec_spire remote search libpq executor requested_epoch exceeds i64")?;
    let selected_pids = row
        .selected_pids
        .iter()
        .map(|pid| {
            i64::try_from(*pid)
                .map_err(|_| "ec_spire remote search libpq executor selected PID exceeds i64")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let top_k = i32::try_from(top_k)
        .map_err(|_| "ec_spire remote search libpq executor top_k exceeds i32")?;

    let result_rows = client
        .query(
            SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
            &[
                &remote_index_oid,
                &requested_epoch,
                &query,
                &selected_pids,
                &top_k,
                &consistency_mode,
            ],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote search libpq executor remote search query failed for node_id {}",
                row.node_id
            )
        })?;
    let candidates = result_rows
        .iter()
        .map(|candidate_row| {
            decode_remote_search_candidate_pg_row(
                candidate_row,
                row.node_id,
                true,
                Some(&row.remote_index_identity),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_remote_search_candidate_batch(
        row.requested_epoch,
        row.node_id,
        &row.selected_pids,
        &candidates,
    )?;

    Ok(candidates)
}

fn remote_search_dispatch_blocked_status(error: &str) -> Option<&str> {
    error
        .strip_prefix("ec_spire remote search libpq executor dispatch for node_id ")
        .and_then(|value| value.rsplit_once(" is blocked with status "))
        .map(|(_, status)| status)
}

fn remote_search_receive_attempt_failure_status(error: &str) -> String {
    if let Some(status) = remote_search_dispatch_blocked_status(error) {
        return status.to_owned();
    }

    let endpoint_status_prefix = "ec_spire remote search executor endpoint_status ";
    let endpoint_status_suffix = " is not ready";
    if let Some(status) = error
        .strip_prefix(endpoint_status_prefix)
        .and_then(|value| value.strip_suffix(endpoint_status_suffix))
    {
        return status.to_owned();
    }

    if error.contains("protocol_version") {
        "protocol_version_mismatch".to_owned()
    } else if error.contains("extension_version") {
        "extension_version_mismatch".to_owned()
    } else if error.contains("served epoch") {
        "served_epoch_mismatch".to_owned()
    } else if error.contains("opclass_identity")
        || error.contains("storage_format")
        || error.contains("assignment_payload_format")
        || error.contains("quantizer_profile")
        || error.contains("scoring_profile")
        || error.contains("profile_fingerprint")
        || error.contains("remote_index_identity")
        || error.contains("endpoint identity")
    {
        SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH.to_owned()
    } else if error.contains(SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD) {
        SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD.to_owned()
    } else if error.contains("conninfo secret") {
        SPIRE_REMOTE_STATUS_REQUIRES_SECRET.to_owned()
    } else if error.contains("failed to open connection") {
        "libpq_connection_failed".to_owned()
    } else if error.contains("remote index") {
        "remote_index_unavailable".to_owned()
    } else {
        "remote_candidate_batch_rejected".to_owned()
    }
}

fn remote_search_receive_attempt_next_blocker(error: &str) -> String {
    if let Some(status) = remote_search_dispatch_blocked_status(error) {
        return remote_search_pre_dispatch_blocker_step(status).to_owned();
    }

    if error.contains("endpoint_status")
        || error.contains("protocol_version")
        || error.contains("extension_version")
        || error.contains("opclass_identity")
        || error.contains("storage_format")
        || error.contains("assignment_payload_format")
        || error.contains("quantizer_profile")
        || error.contains("scoring_profile")
        || error.contains("profile_fingerprint")
        || error.contains("remote_index_identity")
        || error.contains("endpoint identity")
    {
        "remote_endpoint_identity".to_owned()
    } else if error.contains("served epoch") {
        "served_epoch".to_owned()
    } else if error.contains(SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD) {
        SPIRE_REMOTE_EXECUTOR_STEP_GOVERNANCE.to_owned()
    } else if error.contains("conninfo secret") {
        "conninfo_secret_resolution".to_owned()
    } else if error.contains("failed to open connection") {
        "open_libpq_connection".to_owned()
    } else if error.contains("remote index") {
        "remote_index_regclass".to_owned()
    } else {
        "remote_search_candidate_batch".to_owned()
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn remote_search_libpq_identity_cache_contract_probe_counts(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> (u64, u64, u64, u64, String) {
    let result = (|| -> Result<(u64, u64, u64, u64, String), String> {
        let index_relid = unsafe { (*index_relation).rd_id };
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let row = dispatch_rows
            .iter()
            .find(|row| row.dispatch_action == SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION)
            .ok_or_else(|| {
                "ec_spire remote search libpq identity cache contract probe found no ready dispatch"
                    .to_owned()
            })?;
        let conninfo = remote_conninfo_secret_value(&row.conninfo_secret_name).map_err(|status| {
            format!(
                "ec_spire remote search libpq identity cache contract probe conninfo secret for node_id {} is not resolved: {status}",
                row.node_id
            )
        })?;
        let mut client = remote_search_libpq_connect_with_session_timeouts(
            &conninfo,
            row.node_id,
            "remote search libpq identity cache contract probe",
        )?;
        let remote_index_oid = client
            .query_one(
                "SELECT to_regclass($1)::oid",
                &[&row.remote_index_regclass.as_str()],
            )
            .map_err(|_| {
                format!(
                    "ec_spire remote search libpq identity cache contract probe failed to resolve remote index for node_id {}",
                    row.node_id
                )
            })?
            .try_get::<_, Option<u32>>(0)
            .map_err(|_| {
                format!(
                    "ec_spire remote search libpq identity cache contract probe remote index oid decode failed for node_id {}",
                    row.node_id
                )
            })?
            .ok_or_else(|| {
                format!(
                    "ec_spire remote search libpq identity cache contract probe remote index is missing for node_id {}",
                    row.node_id
                )
            })?;

        let mut executor_state = SpireRemoteSearchLibpqExecutorState::default();
        validate_remote_search_libpq_endpoint_identity_for_dispatch(
            &mut client,
            index_relid,
            remote_index_oid,
            row,
            &mut executor_state,
        )?;
        validate_remote_search_libpq_endpoint_identity_for_dispatch(
            &mut client,
            index_relid,
            remote_index_oid,
            row,
            &mut executor_state,
        )?;

        let mut generation_row = row.clone();
        generation_row.descriptor_generation =
            generation_row.descriptor_generation.checked_add(1).ok_or_else(|| {
                "ec_spire remote search libpq identity cache contract probe descriptor generation overflow"
                    .to_owned()
            })?;
        validate_remote_search_libpq_endpoint_identity_for_dispatch(
            &mut client,
            index_relid,
            remote_index_oid,
            &generation_row,
            &mut executor_state,
        )?;

        let mut served_epoch_row = row.clone();
        served_epoch_row.requested_epoch =
            served_epoch_row.requested_epoch.checked_add(1).ok_or_else(|| {
                "ec_spire remote search libpq identity cache contract probe served epoch overflow"
                    .to_owned()
            })?;
        validate_remote_search_libpq_endpoint_identity_for_dispatch(
            &mut client,
            index_relid,
            remote_index_oid,
            &served_epoch_row,
            &mut executor_state,
        )?;

        let mut identity_row = row.clone();
        identity_row.remote_index_identity = if identity_row.remote_index_identity.as_slice() == &[0xff] {
            vec![0x00]
        } else {
            vec![0xff]
        };
        let mismatch_status = match validate_remote_search_libpq_endpoint_identity_for_dispatch(
            &mut client,
            index_relid,
            remote_index_oid,
            &identity_row,
            &mut executor_state,
        ) {
            Ok(()) => SPIRE_REMOTE_STATUS_READY.to_owned(),
            Err(error) => remote_search_receive_attempt_failure_status(&error),
        };

        Ok((
            executor_state.endpoint_identity_cache_entry_count()?,
            executor_state.endpoint_identity_query_count(),
            executor_state.endpoint_identity_cache_hit_count(),
            executor_state.endpoint_identity_cache_miss_count(),
            mismatch_status,
        ))
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
#[allow(clippy::type_complexity)]
pub(crate) fn remote_search_libpq_executor_budget_contract_probe_counts(
) -> (u64, u64, u64, u64, u64, u64, &'static str, &'static str) {
    let connection_rows = vec![
        SpireRemoteSearchLibpqConnectionPlanRow {
            requested_epoch: 7,
            node_id: 2,
            selected_pids: vec![10],
            pid_count: 1,
            query_dimension: 2,
            top_k: 1,
            consistency_mode: "strict",
            execution_transport: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            conninfo_secret_name: "spire/remote/budget/2".to_owned(),
            remote_index_regclass: "remote_spire_idx".to_owned(),
            descriptor_generation: 21,
            remote_index_identity: vec![10],
            remote_index_identity_bytes: 1,
            conninfo_resolution: SPIRE_REMOTE_CONNINFO_READY,
            pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            status: SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ,
        },
        SpireRemoteSearchLibpqConnectionPlanRow {
            requested_epoch: 7,
            node_id: 3,
            selected_pids: vec![20],
            pid_count: 1,
            query_dimension: 2,
            top_k: 1,
            consistency_mode: "strict",
            execution_transport: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            conninfo_secret_name: "spire/remote/budget/3".to_owned(),
            remote_index_regclass: "remote_spire_idx".to_owned(),
            descriptor_generation: 22,
            remote_index_identity: vec![11],
            remote_index_identity_bytes: 1,
            conninfo_resolution: SPIRE_REMOTE_CONNINFO_READY,
            pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            status: SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ,
        },
    ];
    let dispatch_rows = remote_search_libpq_dispatch_plan_rows_from_connections(&connection_rows);
    let budget_summary =
        remote_search_libpq_executor_budget_summary_from_dispatch_rows(7, &dispatch_rows)
            .unwrap_or_else(|e| pgrx::error!("{e}"));
    let secret_rows = remote_search_libpq_secret_plan_rows_from_dispatch(&dispatch_rows);
    let secret_summary = remote_search_libpq_secret_summary_from_plan_rows(7, &secret_rows)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    let dispatch_count = u64::try_from(dispatch_rows.len())
        .unwrap_or_else(|_| pgrx::error!("budget probe dispatch count overflow"));
    let secret_budget_blocked_count = u64::try_from(
        secret_rows
            .iter()
            .filter(|row| {
                row.status == SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD
                    && row.provider_lookup_key == SPIRE_REMOTE_NONE
                    && row.next_executor_step == SPIRE_REMOTE_EXECUTOR_STEP_BUDGET
            })
            .count(),
    )
    .unwrap_or_else(|_| pgrx::error!("budget probe secret blocked count overflow"));

    (
        dispatch_count,
        budget_summary.admitted_dispatch_count,
        budget_summary.budget_blocked_dispatch_count,
        budget_summary.admitted_pid_count,
        budget_summary.budget_blocked_pid_count,
        secret_budget_blocked_count,
        budget_summary.status,
        secret_summary.next_executor_step,
    )
}

pub(crate) unsafe fn remote_search_libpq_executor_receive_attempt_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqReceiveAttemptRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchLibpqReceiveAttemptRow>, String> {
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let index_relid = unsafe { (*index_relation).rd_id };
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query.clone(),
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut executor_state = SpireRemoteSearchLibpqExecutorState::default();
        let mut rows = Vec::with_capacity(dispatch_rows.len());
        for row in &dispatch_rows {
            match remote_search_libpq_executor_candidates_for_dispatch(
                index_relid,
                row,
                &query,
                top_k,
                consistency_mode,
                &mut executor_state,
            ) {
                Ok(candidates) => {
                    rows.push(SpireRemoteSearchLibpqReceiveAttemptRow {
                        requested_epoch: row.requested_epoch,
                        node_id: row.node_id,
                        selected_pids: row.selected_pids.clone(),
                        pid_count: row.pid_count,
                        candidate_count: u64::try_from(candidates.len()).map_err(|_| {
                            "ec_spire remote receive attempt candidate count exceeds u64"
                                .to_owned()
                        })?,
                        status: SPIRE_REMOTE_STATUS_READY.to_owned(),
                        next_blocker: SPIRE_REMOTE_NONE.to_owned(),
                        failure_action: SPIRE_REMOTE_NONE.to_owned(),
                        failure_reason: SPIRE_REMOTE_NONE.to_owned(),
                        recommendation: SPIRE_REMOTE_NONE.to_owned(),
                    });
                }
                Err(error) => {
                    let degraded =
                        requested_consistency_mode == meta::SpireConsistencyMode::Degraded;
                    let failure_action = if degraded {
                        "skip_node"
                    } else {
                        "fail_closed"
                    };
                    let recommendation = if degraded {
                        format!(
                            "skip node_id {} in degraded mode before merge: {error}",
                            row.node_id
                        )
                    } else {
                        format!("strict mode fails closed before merge: {error}")
                    };
                    rows.push(SpireRemoteSearchLibpqReceiveAttemptRow {
                        requested_epoch: row.requested_epoch,
                        node_id: row.node_id,
                        selected_pids: row.selected_pids.clone(),
                        pid_count: row.pid_count,
                        candidate_count: 0,
                        status: remote_search_receive_attempt_failure_status(&error),
                        next_blocker: remote_search_receive_attempt_next_blocker(&error),
                        failure_action: failure_action.to_owned(),
                        failure_reason: error,
                        recommendation,
                    });
                }
            }
        }
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_executor_heap_candidates_for_dispatch(
    index_relid: pg_sys::Oid,
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
    query: &[f32],
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
    if row.dispatch_action != SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
        return Err(format!(
            "ec_spire remote heap executor dispatch for node_id {} is blocked with status {}",
            row.node_id, row.status
        ));
    }

    let _governance_permit = remote_search_libpq_executor_governance_permit(row)?;
    let conninfo = remote_conninfo_secret_value(&row.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire remote heap executor conninfo secret for node_id {} is not resolved: {status}",
            row.node_id
        )
    })?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        row.node_id,
        "remote heap executor",
    )?;
    let remote_index_oid = client
        .query_one(
            "SELECT to_regclass($1)::oid",
            &[&row.remote_index_regclass.as_str()],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote heap executor failed to resolve remote index for node_id {}",
                row.node_id
            )
        })?
        .try_get::<_, Option<u32>>(0)
        .map_err(|_| {
            format!(
                "ec_spire remote heap executor remote index oid decode failed for node_id {}",
                row.node_id
            )
        })?
        .ok_or_else(|| {
            format!(
                "ec_spire remote heap executor remote index is missing for node_id {}",
                row.node_id
            )
        })?;
    validate_remote_search_libpq_endpoint_identity_for_dispatch(
        &mut client,
        index_relid,
        remote_index_oid,
        row,
        executor_state,
    )?;
    let requested_epoch = i64::try_from(row.requested_epoch)
        .map_err(|_| "ec_spire remote heap executor requested_epoch exceeds i64")?;
    let selected_pids = row
        .selected_pids
        .iter()
        .map(|pid| {
            i64::try_from(*pid)
                .map_err(|_| "ec_spire remote heap executor selected PID exceeds i64")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let top_k =
        i32::try_from(top_k).map_err(|_| "ec_spire remote heap executor top_k exceeds i32")?;

    let result_rows = client
        .query(
            SPIRE_REMOTE_SEARCH_LIBPQ_HEAP_SQL_TEMPLATE,
            &[
                &remote_index_oid,
                &requested_epoch,
                &query,
                &selected_pids,
                &top_k,
                &consistency_mode,
            ],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote heap executor remote heap query failed for node_id {}",
                row.node_id
            )
        })?;
    let candidates = result_rows
        .iter()
        .map(|candidate_row| {
            decode_remote_search_heap_candidate_pg_row(
                candidate_row,
                row.requested_epoch,
                row.node_id,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let merge_candidates = candidates
        .iter()
        .map(|candidate| SpireRemoteSearchCandidateRow {
            served_epoch: candidate.served_epoch,
            node_id: candidate.node_id,
            pid: candidate.pid,
            object_version: candidate.object_version,
            row_index: candidate.row_index,
            assignment_flags: candidate.assignment_flags,
            vec_id: candidate.vec_id.clone(),
            row_locator: candidate.row_locator.clone(),
            score: candidate.score,
        })
        .collect::<Vec<_>>();
    validate_remote_search_candidate_batch(
        row.requested_epoch,
        row.node_id,
        &row.selected_pids,
        &merge_candidates,
    )?;

    Ok(candidates)
}

fn remote_search_libpq_executor_candidates_from_dispatch_rows_with_state(
    index_relid: pg_sys::Oid,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    query: &[f32],
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
    let mut candidates = Vec::new();
    for row in dispatch_rows {
        candidates.extend(remote_search_libpq_executor_candidates_for_dispatch(
            index_relid,
            row,
            query,
            top_k,
            consistency_mode,
            executor_state,
        )?);
    }
    Ok(candidates)
}

fn remote_search_libpq_executor_candidate_rows_with_state(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
    let dispatch_rows = unsafe {
        remote_search_libpq_dispatch_plan_rows(
            index_relation,
            requested_epoch,
            query.clone(),
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_libpq_executor_candidates_from_dispatch_rows_with_state(
        unsafe { (*index_relation).rd_id },
        &dispatch_rows,
        &query,
        top_k,
        consistency_mode,
        executor_state,
    )
}

pub(crate) unsafe fn remote_search_libpq_executor_candidate_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchCandidateRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
        let mut executor_state = SpireRemoteSearchLibpqExecutorState::default();
        remote_search_libpq_executor_candidate_rows_with_state(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
            &mut executor_state,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_executor_heap_candidates_from_dispatch_rows_with_state(
    index_relid: pg_sys::Oid,
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    query: &[f32],
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
    let mut candidates = Vec::new();
    for row in dispatch_rows {
        candidates.extend(remote_search_libpq_executor_heap_candidates_for_dispatch(
            index_relid,
            row,
            query,
            top_k,
            consistency_mode,
            executor_state,
        )?);
    }
    candidates.sort_by(|left, right| {
        remote_search_candidate_cmp(
            &SpireRemoteSearchCandidateRow {
                served_epoch: left.served_epoch,
                node_id: left.node_id,
                pid: left.pid,
                object_version: left.object_version,
                row_index: left.row_index,
                assignment_flags: left.assignment_flags,
                vec_id: left.vec_id.clone(),
                row_locator: left.row_locator.clone(),
                score: left.score,
            },
            &SpireRemoteSearchCandidateRow {
                served_epoch: right.served_epoch,
                node_id: right.node_id,
                pid: right.pid,
                object_version: right.object_version,
                row_index: right.row_index,
                assignment_flags: right.assignment_flags,
                vec_id: right.vec_id.clone(),
                row_locator: right.row_locator.clone(),
                score: right.score,
            },
        )
    });
    candidates.truncate(top_k);
    Ok(candidates)
}

fn remote_search_libpq_executor_heap_candidate_rows_with_state(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
    executor_state: &mut SpireRemoteSearchLibpqExecutorState,
) -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
    let dispatch_rows = unsafe {
        remote_search_libpq_dispatch_plan_rows(
            index_relation,
            requested_epoch,
            query.clone(),
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_libpq_executor_heap_candidates_from_dispatch_rows_with_state(
        unsafe { (*index_relation).rd_id },
        &dispatch_rows,
        &query,
        top_k,
        consistency_mode,
        executor_state,
    )
}

pub(crate) unsafe fn remote_search_libpq_executor_heap_candidate_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLocalHeapCandidateRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
        let mut executor_state = SpireRemoteSearchLibpqExecutorState::default();
        remote_search_libpq_executor_heap_candidate_rows_with_state(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
            &mut executor_state,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_libpq_identity_cache_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqIdentityCacheSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqIdentityCacheSummaryRow, String> {
        let index_relid = unsafe { (*index_relation).rd_id };
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query.clone(),
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let dispatch_count = u64::try_from(dispatch_rows.len()).map_err(|_| {
            "ec_spire remote search libpq identity cache dispatch count exceeds u64".to_owned()
        })?;
        let first_blocked_status = dispatch_rows
            .iter()
            .find(|row| row.dispatch_action != SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION)
            .map(|row| row.status);
        let mut executor_state = SpireRemoteSearchLibpqExecutorState::default();
        let (compact_candidate_count, heap_candidate_count, status) = if top_k == 0 {
            (0_u64, 0_u64, SPIRE_REMOTE_STATUS_EMPTY_TOP_K)
        } else if let Some(status) = first_blocked_status {
            (0_u64, 0_u64, status)
        } else {
            match remote_search_libpq_executor_candidates_from_dispatch_rows_with_state(
                index_relid,
                &dispatch_rows,
                &query,
                top_k,
                consistency_mode,
                &mut executor_state,
            ) {
                Ok(compact_candidates) => {
                    let heap_candidates =
                        remote_search_libpq_executor_heap_candidates_from_dispatch_rows_with_state(
                            index_relid,
                            &dispatch_rows,
                            &query,
                            top_k,
                            consistency_mode,
                            &mut executor_state,
                        )?;
                    (
                        u64::try_from(compact_candidates.len()).map_err(|_| {
                            "ec_spire remote search libpq identity cache compact candidate count exceeds u64"
                                .to_owned()
                        })?,
                        u64::try_from(heap_candidates.len()).map_err(|_| {
                            "ec_spire remote search libpq identity cache heap candidate count exceeds u64"
                                .to_owned()
                        })?,
                        SPIRE_REMOTE_STATUS_READY,
                    )
                }
                Err(error) if remote_search_receive_attempt_failure_status(&error)
                    == SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH =>
                {
                    (0_u64, 0_u64, SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH)
                }
                Err(error) => return Err(error),
            }
        };

        Ok(SpireRemoteSearchLibpqIdentityCacheSummaryRow {
            requested_epoch,
            dispatch_count,
            compact_candidate_count,
            heap_candidate_count,
            endpoint_identity_cache_entry_count: executor_state
                .endpoint_identity_cache_entry_count()?,
            endpoint_identity_query_count: executor_state.endpoint_identity_query_count(),
            endpoint_identity_cache_hit_count: executor_state.endpoint_identity_cache_hit_count(),
            endpoint_identity_cache_miss_count: executor_state.endpoint_identity_cache_miss_count(),
            raw_conninfo_cached: false,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_receive_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchReceivePlanRow> {
    let rows = unsafe {
        remote_search_libpq_request_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_receive_plan_rows_from_requests(&rows)
}

fn remote_search_receive_plan_rows_from_requests(
    rows: &[SpireRemoteSearchLibpqRequestPlanRow],
) -> Vec<SpireRemoteSearchReceivePlanRow> {
    rows.iter()
        .map(|row| SpireRemoteSearchReceivePlanRow {
            requested_epoch: row.requested_epoch,
            node_id: row.node_id,
            selected_pids: row.selected_pids.clone(),
            pid_count: row.pid_count,
            expected_candidate_format: row.candidate_format,
            expected_result_column_count: row.result_column_count,
            validator_function: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            row_locator_policy: SPIRE_REMOTE_ROW_LOCATOR_POLICY,
            status: row.status,
        })
        .collect()
}

pub(crate) unsafe fn remote_search_merge_input_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchMergeInputSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchMergeInputSummaryRow, String> {
        let execution_summary = unsafe {
            remote_search_execution_summary_row(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        Ok(remote_search_merge_input_summary_from_execution(
            &execution_summary,
        ))
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_merge_input_summary_from_execution(
    execution_summary: &SpireRemoteSearchExecutionSummaryRow,
) -> SpireRemoteSearchMergeInputSummaryRow {
    let remote_batch_count = execution_summary.remote_plan_count;
    let local_batch_count = execution_summary.local_plan_count;
    let skipped_batch_count = execution_summary.skipped_plan_count;
    let ready_batch_count = execution_summary.ready_plan_count;
    let blocked_batch_count = execution_summary.blocked_plan_count;
    let status = if execution_summary.top_k == 0 {
        SPIRE_REMOTE_STATUS_EMPTY_TOP_K
    } else if blocked_batch_count > 0 {
        execution_summary.status
    } else if execution_summary.degraded_skipped_plan_count > 0 {
        SPIRE_REMOTE_STATUS_DEGRADED_READY
    } else if remote_batch_count > 0 || local_batch_count > 0 {
        SPIRE_REMOTE_STATUS_READY
    } else if skipped_batch_count > 0 {
        SPIRE_REMOTE_STATUS_DEGRADED_READY
    } else {
        SPIRE_REMOTE_STATUS_READY
    };

    SpireRemoteSearchMergeInputSummaryRow {
        requested_epoch: execution_summary.requested_epoch,
        remote_batch_count,
        local_batch_count,
        skipped_batch_count,
        ready_batch_count,
        blocked_batch_count,
        remote_pid_count: execution_summary.remote_pid_count,
        local_pid_count: execution_summary.local_pid_count,
        skipped_pid_count: execution_summary.skipped_pid_count,
        merge_function: SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
        dedupe_key: SPIRE_REMOTE_VEC_ID_DEDUPE_KEY,
        tie_breaker: "score_then_assignment_role_then_epoch_desc_then_node_pid_version_row_locator",
        top_k: execution_summary.top_k,
        status,
    }
}

pub(crate) fn remote_search_merge_order_contract_rows(
) -> Vec<SpireRemoteSearchMergeOrderContractRow> {
    vec![
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 1,
            order_key: "score",
            direction: "ascending",
            semantic_role: "nearest_candidate_first",
            validator: "must_be_finite",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 2,
            order_key: "assignment_role",
            direction: "primary_before_boundary_replica",
            semantic_role: "prefer_primary_placement_on_tie",
            validator: "must_include_visible_assignment_role",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 3,
            order_key: "served_epoch",
            direction: "descending",
            semantic_role: "newer_epoch_wins_tie",
            validator: "must_equal_requested_epoch",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 4,
            order_key: "node_id",
            direction: "ascending",
            semantic_role: "deterministic_node_tie_breaker",
            validator: "must_equal_origin_node",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 5,
            order_key: "pid",
            direction: "ascending",
            semantic_role: "deterministic_partition_tie_breaker",
            validator: "must_be_selected_leaf_pid_or_leaf_derived_delta_pid",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 6,
            order_key: "object_version",
            direction: "descending",
            semantic_role: "newer_object_wins_tie",
            validator: "must_be_positive",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 7,
            order_key: "row_index",
            direction: "ascending",
            semantic_role: "deterministic_row_tie_breaker",
            validator: "must_fit_u32",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 8,
            order_key: "row_locator",
            direction: "lexicographic_ascending",
            semantic_role: "final_stable_tie_breaker",
            validator: "must_be_nonempty_and_opaque",
        },
    ]
}

pub(crate) fn remote_search_row_locator_contract_rows(
) -> Vec<SpireRemoteSearchRowLocatorContractRow> {
    vec![
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "locator_scope",
            contract_value: "origin_node",
            status: "active_contract",
        },
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "coordinator_interpretation",
            contract_value: "opaque_bytes",
            status: "active_contract",
        },
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "receive_validation",
            contract_value: "nonempty_only",
            status: "active_contract",
        },
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "remote_heap_resolution",
            contract_value: "requires_origin_node_resolution",
            status: "deferred_until_remote_heap_fetch",
        },
    ]
}

pub(crate) fn remote_search_vector_identity_contract_rows(
) -> Vec<SpireRemoteSearchVectorIdentityContractRow> {
    vec![
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "global_vec_id_format",
            contract_value: "0x02 || stable_global_payload_bytes",
            status: "active_contract",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "local_vec_id_format",
            contract_value: "0x01 || little_endian_u64",
            status: "compatibility_contract",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "remote_merge_dedupe_key",
            contract_value: SPIRE_REMOTE_VEC_ID_DEDUPE_KEY,
            status: "active_contract",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "local_vec_id_remote_scope",
            contract_value: "node_id || local_vec_id_bytes",
            status: "compatibility_fallback",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "boundary_replica_identity",
            contract_value: "primary_and_boundary_replica_rows_share_identical_vec_id_bytes",
            status: "active_contract",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "cross_node_replica_dedupe",
            contract_value: "requires_global_vec_id_format",
            status: "global_id_required",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "writer_identity_allocation_hook",
            contract_value: "SpireVecIdSourceIdentity",
            status: "phase11_2_landed",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "writer_global_source_identity",
            contract_value: "fixed_16_byte_source_identity_required_not_heap_tid",
            status: "phase11_2_contract_defined",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "writer_global_base_storage",
            contract_value: "leaf_v2_global_bytes_fixed_width_per_object",
            status: "phase11_2_landed",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "writer_global_delta_storage",
            contract_value: "row_encoded_delta_assignments_accept_global_vec_id",
            status: "phase11_2_landed",
        },
    ]
}

pub(crate) fn remote_search_heap_resolution_contract_rows(
) -> Vec<SpireRemoteSearchHeapResolutionContractRow> {
    vec![
        SpireRemoteSearchHeapResolutionContractRow {
            resolution_scope: "local",
            candidate_source: "coordinator_local_candidate_batch",
            heap_lookup_owner: SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
            row_locator_policy: SPIRE_REMOTE_ROW_LOCATOR_POLICY,
            status: SPIRE_REMOTE_STATUS_READY,
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchHeapResolutionContractRow {
            resolution_scope: "remote",
            candidate_source: "libpq_candidate_batch",
            heap_lookup_owner: SPIRE_REMOTE_HEAP_RESOLUTION,
            row_locator_policy: SPIRE_REMOTE_ROW_LOCATOR_POLICY,
            status: SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
            recommendation: "resolve remote row locators on the origin storage node",
        },
    ]
}

pub(crate) unsafe fn remote_search_finalization_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchFinalizationSummaryRow {
    let merge_summary = unsafe {
        remote_search_merge_input_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_finalization_summary_from_merge(&merge_summary)
}

fn remote_search_finalization_summary_from_merge(
    merge_summary: &SpireRemoteSearchMergeInputSummaryRow,
) -> SpireRemoteSearchFinalizationSummaryRow {
    let (final_heap_fetch_status, status, recommendation) = if merge_summary.status
        == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
    {
        (
            SPIRE_REMOTE_FINAL_STATUS_BLOCKED,
            SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
            "register remote node descriptors before remote candidate finalization",
        )
    } else if merge_summary.remote_batch_count > 0 {
        (
            SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
            SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
            "add origin-node row locator resolution before returning remote heap rows",
        )
    } else if merge_summary.local_batch_count > 0 {
        (
            SPIRE_REMOTE_FINAL_STATUS_LOCAL_READY,
            merge_summary.status,
            SPIRE_REMOTE_NONE,
        )
    } else {
        (
            SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
            merge_summary.status,
            SPIRE_REMOTE_NONE,
        )
    };

    SpireRemoteSearchFinalizationSummaryRow {
        requested_epoch: merge_summary.requested_epoch,
        remote_batch_count: merge_summary.remote_batch_count,
        local_batch_count: merge_summary.local_batch_count,
        skipped_batch_count: merge_summary.skipped_batch_count,
        merge_status: merge_summary.status,
        row_locator_policy: SPIRE_REMOTE_ROW_LOCATOR_POLICY,
        local_heap_resolution: SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
        remote_heap_resolution: SPIRE_REMOTE_HEAP_RESOLUTION,
        final_heap_fetch_status,
        status,
        recommendation,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireCoordinatorPipeline {
    execution_summary: SpireRemoteSearchExecutionSummaryRow,
    dispatch_summary: SpireRemoteSearchLibpqDispatchSummaryRow,
    receive_rows: Vec<SpireRemoteSearchReceivePlanRow>,
    finalization_summary: SpireRemoteSearchFinalizationSummaryRow,
    executor_readiness: SpireRemoteSearchLibpqExecutorReadinessRow,
}

impl SpireCoordinatorPipeline {
    unsafe fn execute_once(
        index_relation: pg_sys::Relation,
        requested_epoch: u64,
        query: Vec<f32>,
        selected_pids: Vec<u64>,
        top_k: usize,
        consistency_mode: &str,
    ) -> Result<Self, String> {
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire coordinator pipeline top_k exceeds u64")?;
        let query_for_summary_fallback = query.clone();
        let readiness_rows = unsafe {
            remote_search_request_readiness_rows(
                index_relation,
                requested_epoch,
                query.clone(),
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let execution_rows = readiness_rows
            .into_iter()
            .map(remote_search_execution_plan_row_from_readiness)
            .collect::<Vec<_>>();
        let execution_summary = remote_search_execution_summary_from_plan_rows(
            requested_epoch,
            &execution_rows,
            query_for_summary_fallback.clone(),
            top_k_for_empty_plan,
            consistency_mode,
        )?;
        let request_rows = remote_search_libpq_request_plan_rows_from_execution(&execution_rows);
        let connection_rows = remote_search_libpq_connection_plan_rows_from_requests(
            unsafe { (*index_relation).rd_id },
            &request_rows,
        )?;
        let dispatch_rows = remote_search_libpq_dispatch_plan_rows_from_connections(&connection_rows);
        let dispatch_summary = remote_search_libpq_dispatch_summary_from_plan_rows(
            requested_epoch,
            &dispatch_rows,
            query_for_summary_fallback,
            top_k_for_empty_plan,
            consistency_mode,
        )?;
        let receive_rows = remote_search_receive_plan_rows_from_requests(&request_rows);
        let merge_summary = remote_search_merge_input_summary_from_execution(&execution_summary);
        let finalization_summary = remote_search_finalization_summary_from_merge(&merge_summary);
        let secret_rows = remote_search_libpq_secret_plan_rows_from_dispatch(&dispatch_rows);
        let secret_summary =
            remote_search_libpq_secret_summary_from_plan_rows(requested_epoch, &secret_rows)?;
        let executor_readiness = remote_search_libpq_executor_readiness_from_summaries(
            requested_epoch,
            &dispatch_summary,
            &secret_summary,
        );

        Ok(Self {
            execution_summary,
            dispatch_summary,
            receive_rows,
            finalization_summary,
            executor_readiness,
        })
    }
}

pub(crate) unsafe fn remote_search_coordinator_gate_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchCoordinatorGateSummaryRow {
    let pipeline = unsafe {
        SpireCoordinatorPipeline::execute_once(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    }
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    let execution_summary = &pipeline.execution_summary;
    let dispatch_summary = &pipeline.dispatch_summary;
    let receive_rows = &pipeline.receive_rows;
    let finalization_summary = &pipeline.finalization_summary;
    let executor_readiness = &pipeline.executor_readiness;
    let libpq_receive_count =
        u64::try_from(receive_rows.len()).expect("receive row count should fit in u64");
    let libpq_receive_status = receive_rows
        .iter()
        .find(|row| row.status != SPIRE_REMOTE_STATUS_READY)
        .map(|row| row.status)
        .unwrap_or(SPIRE_REMOTE_STATUS_READY);

    let (next_blocker, status, recommendation) =
        if execution_summary.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
            (
                "remote_node_descriptor",
                SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
                "register remote node descriptors before coordinator execution",
            )
        } else if matches!(
            execution_summary.status,
            SPIRE_REMOTE_STATUS_STALE_EPOCH
                | SPIRE_REMOTE_STATUS_RETENTION_GAP
                | SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION
        ) {
            (
                remote_search_pre_dispatch_blocker_step(execution_summary.status),
                execution_summary.status,
                remote_search_pre_dispatch_blocker_recommendation(execution_summary.status),
            )
        } else if executor_readiness.status == SPIRE_REMOTE_EXECUTOR_REQUIRED {
            (
                executor_readiness.next_executor_step,
                SPIRE_REMOTE_EXECUTOR_REQUIRED,
                executor_readiness.recommendation,
            )
        } else if execution_summary.status == SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ {
            (
                "libpq_transport",
                SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ,
                "add libpq pipeline execution before remote coordinator dispatch",
            )
        } else if finalization_summary.final_heap_fetch_status
            == SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
        {
            (
                "remote_heap_resolution",
                SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
                "add origin-node row locator resolution before returning remote heap rows",
            )
        } else if finalization_summary.status == SPIRE_REMOTE_STATUS_EMPTY_TOP_K {
            (
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_STATUS_EMPTY_TOP_K,
                SPIRE_REMOTE_NONE,
            )
        } else {
            (SPIRE_REMOTE_NONE, finalization_summary.status, SPIRE_REMOTE_NONE)
        };

    SpireRemoteSearchCoordinatorGateSummaryRow {
        requested_epoch,
        local_plan_count: execution_summary.local_plan_count,
        remote_plan_count: execution_summary.remote_plan_count,
        skipped_plan_count: execution_summary.skipped_plan_count,
        local_pid_count: execution_summary.local_pid_count,
        remote_pid_count: execution_summary.remote_pid_count,
        skipped_pid_count: execution_summary.skipped_pid_count,
        execution_status: execution_summary.status,
        libpq_dispatch_count: dispatch_summary.dispatch_count,
        libpq_dispatch_status: dispatch_summary.status,
        libpq_executor_status: executor_readiness.status,
        libpq_executor_next_step: executor_readiness.next_executor_step,
        libpq_receive_count,
        libpq_receive_status,
        merge_status: finalization_summary.merge_status,
        final_heap_fetch_status: finalization_summary.final_heap_fetch_status,
        next_blocker,
        status,
        recommendation,
    }
}

pub(crate) unsafe fn remote_search_heap_resolution_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchHeapResolutionSummaryRow {
    let gate = unsafe {
        remote_search_coordinator_gate_summary_row(
            index_relation,
            requested_epoch,
            query.clone(),
            selected_pids.clone(),
            top_k,
            consistency_mode,
        )
    };

    let decoded_local_locator_count = if gate.remote_plan_count == 0
        && gate.status != SPIRE_REMOTE_STATUS_EMPTY_TOP_K
    {
        let rows = unsafe {
            remote_search_local_heap_resolution_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        u64::try_from(rows.len())
            .unwrap_or_else(|_| pgrx::error!("ec_spire local heap resolution row count overflow"))
    } else {
        0
    };

    let local_heap_resolution_status = if gate.local_plan_count == 0 {
        SPIRE_REMOTE_NONE
    } else if gate.status == SPIRE_REMOTE_STATUS_EMPTY_TOP_K {
        SPIRE_REMOTE_STATUS_EMPTY_TOP_K
    } else if gate.remote_plan_count == 0 && remote_search_status_allows_local_heap_rows(gate.status)
    {
        SPIRE_REMOTE_STATUS_READY
    } else {
        SPIRE_REMOTE_FINAL_STATUS_PLANNED
    };
    let remote_heap_resolution_status = if gate.remote_plan_count == 0 {
        SPIRE_REMOTE_NONE
    } else if gate.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
        || gate.status == SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
        || gate.status == SPIRE_REMOTE_STATUS_STALE_EPOCH
        || gate.status == SPIRE_REMOTE_STATUS_RETENTION_GAP
        || gate.status == SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION
    {
        gate.status
    } else {
        SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
    };

    SpireRemoteSearchHeapResolutionSummaryRow {
        requested_epoch,
        local_plan_count: gate.local_plan_count,
        remote_plan_count: gate.remote_plan_count,
        skipped_plan_count: gate.skipped_plan_count,
        local_pid_count: gate.local_pid_count,
        remote_pid_count: gate.remote_pid_count,
        decoded_local_locator_count,
        local_heap_resolution_status,
        remote_heap_resolution_status,
        status: gate.status,
        recommendation: gate.recommendation,
    }
}

fn validate_remote_candidate_vec_id(
    candidate: &SpireRemoteSearchCandidateRow,
    context: &str,
) -> Result<storage::SpireVecId, String> {
    storage::SpireVecId::from_bytes(&candidate.vec_id).map_err(|e| {
        format!(
            "ec_spire {context} candidate PID {} row_index {} has invalid vec_id {}: {e}",
            candidate.pid,
            candidate.row_index,
            hex::encode(&candidate.vec_id)
        )
    })
}

fn remote_search_candidate_dedupe_key(
    candidate: &SpireRemoteSearchCandidateRow,
) -> Result<Vec<u8>, String> {
    let vec_id = validate_remote_candidate_vec_id(candidate, "remote candidate merge")?;
    let mut key = Vec::new();
    if vec_id.discriminator() == storage::SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR {
        key.reserve_exact(1 + candidate.vec_id.len());
        key.push(SPIRE_REMOTE_VEC_ID_KEY_GLOBAL);
        key.extend_from_slice(&candidate.vec_id);
    } else {
        key.reserve_exact(1 + std::mem::size_of::<u32>() + candidate.vec_id.len());
        key.push(SPIRE_REMOTE_VEC_ID_KEY_NODE_LOCAL);
        key.extend_from_slice(&candidate.node_id.to_le_bytes());
        key.extend_from_slice(&candidate.vec_id);
    }
    Ok(key)
}

/// Validates one target-scoped remote candidate receive batch.
///
/// The batch must match the requested epoch, expected node, selected leaf PID
/// set, visible assignment flags, valid vec_id, nonempty opaque row_locator,
/// and finite score contract before candidates can enter the merge path.
///
/// Delta insert rows are leaf-derived: the storage endpoint is selected by
/// leaf PID, but it returns the delta object PID for row/version tie-breaking.
/// The coordinator can envelope-validate those rows by their delta flag; the
/// origin storage endpoint remains responsible for binding each delta row to a
/// selected parent leaf.
pub(crate) fn validate_remote_search_candidate_batch(
    requested_epoch: u64,
    expected_node_id: u32,
    selected_pids: &[u64],
    candidates: &[SpireRemoteSearchCandidateRow],
) -> Result<(), String> {
    if requested_epoch == 0 {
        return Err(
            "ec_spire remote candidate batch requested_epoch must be greater than 0".to_owned(),
        );
    }

    let mut selected = HashSet::new();
    for &pid in selected_pids {
        if pid == 0 {
            return Err("ec_spire remote candidate batch selected PID 0 is invalid".to_owned());
        }
        if !selected.insert(pid) {
            return Err(format!(
                "ec_spire remote candidate batch selected PID {pid} appears more than once"
            ));
        }
    }

    for candidate in candidates {
        if candidate.served_epoch != requested_epoch {
            return Err(format!(
                "ec_spire remote candidate batch served epoch {} does not match requested epoch {requested_epoch}",
                candidate.served_epoch
            ));
        }
        if candidate.node_id != expected_node_id {
            return Err(format!(
                "ec_spire remote candidate batch node_id {} does not match expected node_id {expected_node_id}",
                candidate.node_id
            ));
        }
        if candidate.pid == 0 {
            return Err("ec_spire remote candidate batch candidate PID 0 is invalid".to_owned());
        }
        if !selected.contains(&candidate.pid)
            && candidate.assignment_flags & storage::SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT == 0
        {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} was not selected for node_id {expected_node_id} and is not a delta insert candidate",
                candidate.pid
            ));
        }
        if candidate.object_version == 0 {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} has object_version 0",
                candidate.pid
            ));
        }
        if !storage::is_visible_scored_assignment_flags(candidate.assignment_flags) {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} has non-visible assignment_flags {}",
                candidate.pid, candidate.assignment_flags
            ));
        }
        validate_remote_candidate_vec_id(candidate, "remote candidate batch")?;
        if candidate.row_locator.is_empty() {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} has empty row_locator",
                candidate.pid
            ));
        }
        if !candidate.score.is_finite() {
            return Err("ec_spire remote candidate batch received non-finite score".to_owned());
        }
    }

    Ok(())
}

/// Merges candidates using globally comparable vec-id bytes when available and
/// node-scoped local vec-id bytes for compatibility with existing local-only
/// indexes.
pub(crate) fn merge_remote_search_candidates<I>(
    candidates: I,
    limit: Option<usize>,
) -> Result<SpireRemoteSearchMergeResult, String>
where
    I: IntoIterator<Item = SpireRemoteSearchCandidateRow>,
{
    let mut input_count = 0_u64;
    let mut duplicate_vec_id_count = 0_u64;
    let mut best_by_vec_id: HashMap<Vec<u8>, SpireRemoteSearchCandidateRow> = HashMap::new();

    for candidate in candidates {
        input_count = input_count
            .checked_add(1)
            .ok_or_else(|| "ec_spire remote candidate merge input count overflow".to_owned())?;
        if !candidate.score.is_finite() {
            return Err("ec_spire remote candidate merge received non-finite score".to_owned());
        }
        let dedupe_key = remote_search_candidate_dedupe_key(&candidate)?;

        match best_by_vec_id.entry(dedupe_key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                duplicate_vec_id_count =
                    duplicate_vec_id_count.checked_add(1).ok_or_else(|| {
                        "ec_spire remote candidate merge duplicate count overflow".to_owned()
                    })?;
                if remote_search_candidate_cmp(&candidate, entry.get()).is_lt() {
                    *entry.get_mut() = candidate;
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
        }
    }

    let mut candidates = best_by_vec_id.into_values().collect::<Vec<_>>();
    candidates.sort_by(remote_search_candidate_cmp);
    if let Some(limit) = limit {
        candidates.truncate(limit);
    }

    Ok(SpireRemoteSearchMergeResult {
        candidates,
        input_count,
        duplicate_vec_id_count,
    })
}

/// Validates each target-scoped receive batch before global candidate merge.
///
/// Global vec-id bytes dedupe across nodes. Local vec-id bytes dedupe only
/// within their origin node as a compatibility fallback.
pub(crate) fn merge_validated_remote_search_candidate_batches(
    requested_epoch: u64,
    batches: Vec<SpireRemoteSearchCandidateBatch>,
    limit: Option<usize>,
) -> Result<SpireRemoteSearchMergeResult, String> {
    for batch in &batches {
        validate_remote_search_candidate_batch(
            requested_epoch,
            batch.node_id,
            &batch.selected_pids,
            &batch.candidates,
        )?;
    }

    merge_remote_search_candidates(
        batches.into_iter().flat_map(|batch| batch.candidates),
        limit,
    )
}

#[cfg(test)]
mod production_executor_state_tests {
    use super::*;

    fn planned_dispatch(node_id: u32, pid_count: u64) -> SpireRemoteSearchLibpqDispatchPlanRow {
        SpireRemoteSearchLibpqDispatchPlanRow {
            requested_epoch: 7,
            node_id,
            selected_pids: (0..pid_count).collect(),
            pid_count,
            query_dimension: 2,
            top_k: 10,
            consistency_mode: "strict",
            sql_template: SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
            parameter_count: SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT,
            result_column_count: remote_search_result_column_count(),
            conninfo_secret_name: format!("spire/remote/{node_id}"),
            remote_index_regclass: format!("ec_spire_remote_{node_id}_idx"),
            descriptor_generation: 1,
            remote_index_identity: vec![u8::try_from(node_id).expect("node id should fit u8")],
            pipeline_mode: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            dispatch_action: SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION,
            receive_validator: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            status: SPIRE_REMOTE_STATUS_READY,
        }
    }

    fn blocked_dispatch(
        node_id: u32,
        pid_count: u64,
        status: &'static str,
    ) -> SpireRemoteSearchLibpqDispatchPlanRow {
        let mut row = planned_dispatch(node_id, pid_count);
        row.pipeline_mode = SPIRE_REMOTE_NONE;
        row.dispatch_action = SPIRE_REMOTE_DISPATCH_BLOCKED_ACTION;
        row.status = status;
        row
    }

    fn ready_transport_row(
        node_id: u32,
        row_count: u64,
    ) -> SpireRemoteProductionTransportProbeRow {
        SpireRemoteProductionTransportProbeRow {
            node_id,
            started_after_ms: 1,
            completed_after_ms: 2,
            elapsed_ms: 1,
            row_count,
            status: SPIRE_REMOTE_STATUS_READY,
            failure_category: SPIRE_REMOTE_NONE,
        }
    }

    fn failed_transport_row(
        node_id: u32,
        failure_category: &'static str,
    ) -> SpireRemoteProductionTransportProbeRow {
        SpireRemoteProductionTransportProbeRow {
            node_id,
            started_after_ms: 1,
            completed_after_ms: 2,
            elapsed_ms: 1,
            row_count: 0,
            status: SPIRE_REMOTE_STATUS_PRODUCTION_TRANSPORT_FAILED,
            failure_category,
        }
    }

    fn candidate_for_state_test(
        node_id: u32,
        pid: u64,
        row_index: u32,
    ) -> SpireRemoteSearchCandidateRow {
        SpireRemoteSearchCandidateRow {
            served_epoch: 7,
            node_id,
            pid,
            object_version: 1,
            row_index,
            assignment_flags: storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: storage::SpireVecId::local(
                (u64::from(node_id) << 32) | (u64::from(row_index) + 1),
            )
            .as_bytes()
            .to_vec(),
            row_locator: vec![row_index as u8 + 1],
            score: row_index as f32,
        }
    }

    fn ready_candidate_receive_result(
        node_id: u32,
        selected_pids: Vec<u64>,
        candidate_count: u32,
    ) -> SpireRemoteProductionCandidateReceiveResult {
        let pid = selected_pids
            .first()
            .copied()
            .expect("selected pid should exist");
        let candidates = (0..candidate_count)
            .map(|row_index| candidate_for_state_test(node_id, pid, row_index))
            .collect::<Vec<_>>();
        SpireRemoteProductionCandidateReceiveResult {
            node_id,
            started_after_ms: 2,
            completed_after_ms: 3,
            elapsed_ms: 1,
            candidate_count: u64::from(candidate_count),
            status: SPIRE_REMOTE_STATUS_READY,
            failure_category: SPIRE_REMOTE_NONE,
            batch: Some(SpireRemoteSearchCandidateBatch {
                node_id,
                selected_pids,
                candidates,
            }),
        }
    }

    fn failed_candidate_receive_result(
        node_id: u32,
        failure_category: &'static str,
    ) -> SpireRemoteProductionCandidateReceiveResult {
        SpireRemoteProductionCandidateReceiveResult {
            node_id,
            started_after_ms: 2,
            completed_after_ms: 3,
            elapsed_ms: 1,
            candidate_count: 0,
            status: SPIRE_REMOTE_STATUS_CANDIDATE_RECEIVE_FAILED,
            failure_category,
            batch: None,
        }
    }

    #[test]
    fn production_fault_matrix_covers_required_categories() {
        let rows = remote_search_production_fault_matrix_rows();
        let categories = rows
            .iter()
            .map(|row| row.failure_category)
            .collect::<std::collections::HashSet<_>>();
        let required = [
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED,
            SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            SPIRE_REMOTE_STATUS_REQUIRES_SECRET,
            SPIRE_REMOTE_PRODUCTION_REMOTE_STATEMENT_TIMEOUT,
            SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT,
            SPIRE_REMOTE_PRODUCTION_REMOTE_BACKEND_TERMINATED,
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED,
            SPIRE_REMOTE_PRODUCTION_REMOTE_QUERY_CANCELLED,
            SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED,
            SPIRE_REMOTE_PRODUCTION_CANDIDATE_VALIDATION_FAILED,
            SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH,
            SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION,
            SPIRE_REMOTE_PRODUCTION_EXTENSION_VERSION_MISMATCH,
            SPIRE_REMOTE_STATUS_STALE_EPOCH,
            SPIRE_REMOTE_PRODUCTION_SERVED_EPOCH_MISMATCH,
            SPIRE_REMOTE_STATUS_CONSISTENCY_MODE_MISMATCH,
            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE,
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_RESOLUTION_FAILED,
            SPIRE_REMOTE_PRODUCTION_REMOTE_HEAP_ROW_MISSING,
        ];

        assert_eq!(rows.len(), categories.len(), "matrix categories should be unique");
        for category in required {
            assert!(categories.contains(category), "missing category {category}");
        }
        let local_timeout = rows
            .iter()
            .find(|row| row.failure_category == SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT)
            .expect("local timeout row should exist");
        let remote_timeout = rows
            .iter()
            .find(|row| row.failure_category == SPIRE_REMOTE_PRODUCTION_REMOTE_STATEMENT_TIMEOUT)
            .expect("remote timeout row should exist");
        let consistency_mismatch = rows
            .iter()
            .find(|row| row.failure_category == SPIRE_REMOTE_STATUS_CONSISTENCY_MODE_MISMATCH)
            .expect("consistency mismatch row should exist");

        assert_eq!(local_timeout.strict_action, "cancel_query");
        assert_eq!(local_timeout.degraded_action, "cancel_query");
        assert_eq!(remote_timeout.strict_action, "fail_closed");
        assert_eq!(remote_timeout.degraded_action, "skip_node");
        assert_eq!(consistency_mismatch.degraded_action, "fail_closed");
    }

    #[test]
    fn stage_e_fault_matrix_covers_fixture_cases() {
        let rows = remote_search_stage_e_fault_matrix_rows();
        let cases = rows
            .iter()
            .map(|row| row.fault_case)
            .collect::<std::collections::HashSet<_>>();
        let required = [
            "epoch_mismatch",
            "version_skew",
            "fingerprint_mismatch",
            "connection_reset_mid_batch",
            "remote_backend_termination",
            "remote_statement_timeout",
            "local_statement_timeout",
            "local_cancel",
            "simulated_network_partition",
            "remote_oom",
            "missing_or_reindexed_remote_index",
        ];

        assert_eq!(rows.len(), cases.len(), "Stage E fault cases should be unique");
        for fault_case in required {
            assert!(cases.contains(fault_case), "missing Stage E case {fault_case}");
        }

        let local_cancel = rows
            .iter()
            .find(|row| row.fault_case == "local_cancel")
            .expect("local cancel case should exist");
        assert_eq!(local_cancel.strict_action, "cancel_query");
        assert_eq!(local_cancel.degraded_action, "cancel_query");
        assert!(local_cancel
            .counter_delta
            .contains("retained_candidate_batch_count=0"));

        let remote_oom = rows
            .iter()
            .find(|row| row.fault_case == "remote_oom")
            .expect("remote OOM case should exist");
        assert_eq!(
            remote_oom.failure_category,
            SPIRE_REMOTE_PRODUCTION_TRANSPORT_REMOTE_QUERY_FAILED
        );
        assert_eq!(remote_oom.degraded_action, "skip_node");

        let missing_index = rows
            .iter()
            .find(|row| row.fault_case == "missing_or_reindexed_remote_index")
            .expect("missing/reindexed index case should exist");
        assert_eq!(
            missing_index.failure_category,
            SPIRE_REMOTE_PRODUCTION_REMOTE_INDEX_UNAVAILABLE
        );
        assert_eq!(
            missing_index.next_executor_step,
            SPIRE_REMOTE_EXECUTOR_STEP_COMPACT_CANDIDATE_RECEIVE
        );
    }

    #[test]
    fn production_executor_state_moves_ready_transport_to_candidate_receive() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 2)];
        let transport_rows = vec![ready_transport_row(2, 4), ready_transport_row(3, 5)];
        let row = remote_search_production_executor_state_summary_from_transport_probe_rows(
            7,
            &dispatch_rows,
            &transport_rows,
        )
        .expect("transport summary should succeed");

        assert_eq!(row.planned_dispatch_count, 2);
        assert_eq!(row.transport_pending_dispatch_count, 0);
        assert_eq!(row.transport_sent_dispatch_count, 2);
        assert_eq!(row.transport_ready_dispatch_count, 2);
        assert_eq!(row.transport_failed_dispatch_count, 0);
        assert_eq!(row.transport_row_count, 9);
        assert_eq!(row.next_executor_step, "compact_candidate_receive");
        assert_eq!(row.status, "requires_compact_candidate_receive");
        assert_eq!(row.first_transport_failure_category, "none");
    }

    #[test]
    fn production_executor_state_preserves_transport_failure_category() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![
            failed_transport_row(2, SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED),
            ready_transport_row(3, 1),
        ];
        let row = remote_search_production_executor_state_summary_from_transport_probe_rows(
            7,
            &dispatch_rows,
            &transport_rows,
        )
        .expect("transport summary should succeed");

        assert_eq!(row.transport_sent_dispatch_count, 2);
        assert_eq!(row.transport_ready_dispatch_count, 1);
        assert_eq!(row.transport_failed_dispatch_count, 1);
        assert_eq!(row.next_executor_step, "production_transport_adapter");
        assert_eq!(row.status, "remote_transport_failed");
        assert_eq!(row.first_transport_failure_category, "connect_failed");
    }

    #[test]
    fn production_executor_degraded_transport_failure_skips_node() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![
            failed_transport_row(2, SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED),
            ready_transport_row(3, 1),
        ];
        let row =
            remote_search_production_executor_state_summary_from_transport_probe_rows_with_consistency_mode(
                7,
                &dispatch_rows,
                &transport_rows,
                "degraded",
            )
            .expect("degraded transport summary should succeed");

        assert_eq!(row.transport_sent_dispatch_count, 1);
        assert_eq!(row.transport_failed_dispatch_count, 0);
        assert_eq!(row.degraded_skipped_dispatch_count, 1);
        assert_eq!(row.first_degraded_skip_category, "connect_failed");
        assert_eq!(row.candidate_receive_pending_dispatch_count, 1);
        assert_eq!(row.next_executor_step, "compact_candidate_receive");
        assert_eq!(row.status, "requires_compact_candidate_receive");
    }

    #[test]
    fn production_executor_degraded_pre_dispatch_block_skips_node() {
        let dispatch_rows = vec![
            blocked_dispatch(2, 1, SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION),
            planned_dispatch(3, 1),
        ];
        let row = remote_search_production_executor_state_summary_from_dispatch_rows(
            7,
            &dispatch_rows,
            "function_argument",
            "degraded",
        )
        .expect("degraded pre-dispatch summary should succeed");

        assert_eq!(row.dispatch_count, 2);
        assert_eq!(row.blocked_before_dispatch_count, 0);
        assert_eq!(row.degraded_skipped_dispatch_count, 1);
        assert_eq!(
            row.first_degraded_skip_category,
            "incompatible_extension_version"
        );
        assert_eq!(row.transport_pending_dispatch_count, 1);
        assert_eq!(row.next_executor_step, "production_transport_adapter");
        assert_eq!(row.status, "requires_production_transport_adapter");
    }

    #[test]
    fn degraded_skip_report_lists_each_skipped_node() {
        let dispatch_rows = vec![
            blocked_dispatch(2, 3, SPIRE_REMOTE_STATUS_STALE_EPOCH),
            blocked_dispatch(4, 2, SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION),
            planned_dispatch(5, 1),
        ];
        let rows = remote_search_production_degraded_skip_report_from_dispatch_rows(
            7,
            &dispatch_rows,
            "degraded",
        )
        .expect("degraded skip report should succeed");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].requested_epoch, 7);
        assert_eq!(rows[0].node_id, 2);
        assert_eq!(rows[0].skipped_pid_count, 3);
        assert_eq!(rows[0].first_skip_category, "stale_epoch");
        assert_eq!(rows[0].status, "degraded_skipped");
        assert_eq!(rows[1].node_id, 4);
        assert_eq!(rows[1].skipped_pid_count, 2);
        assert_eq!(rows[1].first_skip_category, "incompatible_extension_version");
        assert_eq!(rows[1].status, "degraded_skipped");
    }

    #[test]
    fn production_executor_state_rejects_unplanned_transport_result() {
        let dispatch_rows = vec![planned_dispatch(2, 1)];
        let transport_rows = vec![ready_transport_row(3, 1)];
        let error = remote_search_production_executor_state_summary_from_transport_probe_rows(
            7,
            &dispatch_rows,
            &transport_rows,
        )
        .expect_err("unplanned transport row should fail");

        assert!(
            error.contains("does not match a planned dispatch"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn production_executor_state_moves_ready_receive_to_remote_heap_resolution() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            ready_candidate_receive_result(2, vec![0], 2),
            ready_candidate_receive_result(3, vec![0], 1),
        ];
        let row = remote_search_production_executor_state_summary_from_candidate_receive_results(
            7,
            &dispatch_rows,
            &transport_rows,
            &receive_results,
        )
        .expect("candidate receive summary should succeed");

        assert_eq!(row.candidate_receive_pending_dispatch_count, 0);
        assert_eq!(row.candidate_receive_sent_dispatch_count, 2);
        assert_eq!(row.candidate_receive_ready_dispatch_count, 2);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(row.candidate_row_count, 3);
        assert_eq!(row.next_executor_step, "remote_heap_resolution");
        assert_eq!(row.status, "requires_remote_heap_resolution");
        assert_eq!(row.first_candidate_receive_failure_category, "none");
    }

    #[test]
    fn production_executor_heap_receive_requests_carry_tuple_payload_columns() {
        let dispatch_rows = vec![planned_dispatch(82, 1)];
        let transport_rows = vec![ready_transport_row(82, 1)];
        let receive_results = vec![ready_candidate_receive_result(82, vec![0], 1)];
        let secret_key = remote_conninfo_secret_provider_lookup_key("spire/remote/82")
            .expect("secret key should build");
        std::env::set_var(&secret_key, "host=127.0.0.1 port=1");
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        executor
            .apply_candidate_receive_results(&receive_results)
            .expect("receive rows should apply");
        let requested_columns = vec!["id".to_owned(), "title".to_owned()];

        let requests = executor
            .remote_heap_receive_requests(&[1.0, 0.0], 1, "strict", Some(&requested_columns))
            .expect("heap receive requests should build");
        std::env::remove_var(secret_key);

        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].tuple_payload_columns.as_deref(),
            Some(requested_columns.as_slice())
        );
    }

    #[test]
    fn production_executor_state_preserves_candidate_receive_failure_category() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            failed_candidate_receive_result(2, SPIRE_REMOTE_PRODUCTION_CANDIDATE_DECODE_FAILED),
            ready_candidate_receive_result(3, vec![0], 1),
        ];
        let row = remote_search_production_executor_state_summary_from_candidate_receive_results(
            7,
            &dispatch_rows,
            &transport_rows,
            &receive_results,
        )
        .expect("candidate receive summary should succeed");

        assert_eq!(row.candidate_receive_sent_dispatch_count, 2);
        assert_eq!(row.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 1);
        assert_eq!(row.candidate_row_count, 1);
        assert_eq!(row.next_executor_step, "compact_candidate_receive");
        assert_eq!(row.status, "remote_candidate_receive_failed");
        assert_eq!(row.first_candidate_receive_failure_category, "candidate_decode_failed");
    }

    #[test]
    fn production_executor_degraded_receive_failure_allows_ready_merge() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            failed_candidate_receive_result(2, SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH),
            ready_candidate_receive_result(3, vec![30], 1),
        ];
        let row =
            remote_search_production_executor_state_summary_from_candidate_receive_results_with_consistency_mode(
                7,
                &dispatch_rows,
                &transport_rows,
                &receive_results,
                "degraded",
            )
            .expect("degraded candidate receive summary should succeed");
        let merged =
            remote_search_production_compact_merge_from_candidate_receive_results_with_consistency_mode(
                7,
                &dispatch_rows,
                &transport_rows,
                &receive_results,
                Some(10),
                "degraded",
            )
            .expect("degraded candidate receive should merge ready batches");

        assert_eq!(row.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(row.degraded_skipped_dispatch_count, 1);
        assert_eq!(row.first_degraded_skip_category, "endpoint_identity_mismatch");
        assert_eq!(row.next_executor_step, "remote_heap_resolution");
        assert_eq!(row.status, "degraded_ready");
        assert_eq!(merged.input_count, 1);
        assert_eq!(merged.candidates.len(), 1);
        assert_eq!(merged.candidates[0].node_id, 3);
    }

    #[test]
    fn production_executor_state_rejects_receive_without_ready_transport() {
        let dispatch_rows = vec![planned_dispatch(2, 1)];
        let transport_rows = Vec::new();
        let receive_results = vec![ready_candidate_receive_result(2, vec![0], 1)];
        let error = remote_search_production_executor_state_summary_from_candidate_receive_results(
            7,
            &dispatch_rows,
            &transport_rows,
            &receive_results,
        )
        .expect_err("receive before transport should fail");

        assert!(
            error.contains("does not match a transport-ready dispatch"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn production_executor_compact_merge_uses_ready_candidate_batches() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let mut node_two = ready_candidate_receive_result(2, vec![10], 1);
        let mut node_three = ready_candidate_receive_result(3, vec![20], 1);
        let shared = storage::SpireVecId::global(b"shared")
            .expect("test global vec_id should build")
            .as_bytes()
            .to_vec();
        node_two
            .batch
            .as_mut()
            .expect("node two batch should exist")
            .candidates[0]
            .vec_id = shared.clone();
        node_two
            .batch
            .as_mut()
            .expect("node two batch should exist")
            .candidates[0]
            .score = 0.4;
        node_three
            .batch
            .as_mut()
            .expect("node three batch should exist")
            .candidates[0]
            .vec_id = shared;
        node_three
            .batch
            .as_mut()
            .expect("node three batch should exist")
            .candidates[0]
            .score = 0.2;

        let merged = remote_search_production_compact_merge_from_candidate_receive_results(
            7,
            &dispatch_rows,
            &transport_rows,
            &[node_two, node_three],
            Some(1),
        )
        .expect("ready candidate batches should merge");

        assert_eq!(merged.input_count, 2);
        assert_eq!(merged.duplicate_vec_id_count, 1);
        assert_eq!(merged.candidates.len(), 1);
        assert_eq!(merged.candidates[0].node_id, 3);
        assert_eq!(merged.candidates[0].score, 0.2);
    }

    #[test]
    fn production_executor_compact_merge_rejects_failed_receive() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            failed_candidate_receive_result(2, SPIRE_REMOTE_PRODUCTION_CANDIDATE_DECODE_FAILED),
            ready_candidate_receive_result(3, vec![20], 1),
        ];
        let error = remote_search_production_compact_merge_from_candidate_receive_results(
            7,
            &dispatch_rows,
            &transport_rows,
            &receive_results,
            Some(1),
        )
        .expect_err("failed receive should block compact merge");

        assert!(error.contains("remote_candidate_receive_failed"));
    }

    #[test]
    fn production_executor_local_cancel_clears_ready_candidate_batches() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![
            ready_candidate_receive_result(2, vec![10], 1),
            ready_candidate_receive_result(3, vec![20], 1),
        ];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        executor
            .apply_candidate_receive_results(&receive_results)
            .expect("receive rows should apply");
        assert_eq!(
            executor
                .ready_candidate_batches()
                .expect("ready batches should exist before cancel")
                .len(),
            2
        );

        executor.apply_local_query_cancel(SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED);
        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(row.cancelled_dispatch_count, 2);
        assert_eq!(row.first_cancellation_category, "local_query_cancelled");
        assert_eq!(row.candidate_receive_ready_dispatch_count, 0);
        assert_eq!(row.candidate_row_count, 0);
        assert_eq!(row.next_executor_step, "remote_executor_cancellation");
        assert_eq!(row.status, "remote_executor_cancelled");
        assert!(executor
            .dispatches
            .iter()
            .all(|dispatch| dispatch.candidate_batch.is_none()));

        let error = executor
            .merge_ready_candidate_batches(Some(1))
            .expect_err("cancelled batches should not merge");
        assert!(error.contains("remote_executor_cancelled"));
    }

    #[test]
    fn production_executor_transport_local_cancel_result_cancels_all_dispatches() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&[failed_transport_row(
                2,
                SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED,
            )])
            .expect("transport local cancel should apply globally");

        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(row.cancelled_dispatch_count, 2);
        assert_eq!(row.first_cancellation_category, "local_query_cancelled");
        assert_eq!(row.transport_failed_dispatch_count, 0);
        assert_eq!(row.next_executor_step, "remote_executor_cancellation");
        assert_eq!(row.status, "remote_executor_cancelled");
        assert!(executor.dispatches.iter().all(|dispatch| {
            dispatch.state == SpireRemoteProductionDispatchState::Cancelled
                && dispatch.candidate_batch.is_none()
        }));
    }

    #[test]
    fn production_executor_transport_local_statement_timeout_cancels_all_dispatches() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&[failed_transport_row(
                2,
                SPIRE_REMOTE_PRODUCTION_LOCAL_STATEMENT_TIMEOUT,
            )])
            .expect("transport local statement timeout should apply globally");

        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(row.cancelled_dispatch_count, 2);
        assert_eq!(row.first_cancellation_category, "local_statement_timeout");
        assert_eq!(row.transport_failed_dispatch_count, 0);
        assert_eq!(row.next_executor_step, "remote_executor_cancellation");
        assert_eq!(row.status, "remote_executor_cancelled");
        assert!(executor.dispatches.iter().all(|dispatch| {
            dispatch.state == SpireRemoteProductionDispatchState::Cancelled
                && dispatch.candidate_batch.is_none()
        }));
    }

    #[test]
    fn production_executor_receive_local_cancel_result_cancels_all_dispatches() {
        let dispatch_rows = vec![planned_dispatch(2, 1), planned_dispatch(3, 1)];
        let transport_rows = vec![ready_transport_row(2, 1), ready_transport_row(3, 1)];
        let receive_results = vec![failed_candidate_receive_result(
            2,
            SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED,
        )];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        executor
            .apply_candidate_receive_results(&receive_results)
            .expect("receive local cancel should apply globally");

        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(row.cancelled_dispatch_count, 2);
        assert_eq!(row.first_cancellation_category, "local_query_cancelled");
        assert_eq!(row.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(row.candidate_row_count, 0);
        assert_eq!(row.next_executor_step, "remote_executor_cancellation");
        assert_eq!(row.status, "remote_executor_cancelled");
        assert!(executor.dispatches.iter().all(|dispatch| {
            dispatch.state == SpireRemoteProductionDispatchState::Cancelled
                && dispatch.candidate_batch.is_none()
        }));
    }

    #[test]
    fn production_executor_compact_merge_rejects_every_non_ready_state() {
        let mut blocked_row = planned_dispatch(2, 1);
        blocked_row.dispatch_action = SPIRE_REMOTE_DISPATCH_BLOCKED_ACTION;
        blocked_row.status = SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD;
        let blocked_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[blocked_row]);
        assert!(blocked_executor
            .merge_ready_candidate_batches(None)
            .expect_err("blocked dispatch should not merge")
            .contains("remote_executor_overload"));

        let planned_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[planned_dispatch(2, 1)]);
        assert!(planned_executor
            .merge_ready_candidate_batches(None)
            .expect_err("planned dispatch should not merge")
            .contains("requires_production_transport_adapter"));

        let mut transport_ready_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[planned_dispatch(2, 1)]);
        transport_ready_executor
            .apply_transport_probe_rows(&[ready_transport_row(2, 1)])
            .expect("transport row should apply");
        assert!(transport_ready_executor
            .merge_ready_candidate_batches(None)
            .expect_err("transport-ready dispatch should not merge")
            .contains("requires_compact_candidate_receive"));

        let mut transport_failed_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[planned_dispatch(2, 1)]);
        transport_failed_executor
            .apply_transport_probe_rows(&[failed_transport_row(
                2,
                SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED,
            )])
            .expect("failed transport row should apply");
        assert!(transport_failed_executor
            .merge_ready_candidate_batches(None)
            .expect_err("transport-failed dispatch should not merge")
            .contains("remote_transport_failed"));

        let mut receive_failed_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[planned_dispatch(2, 1)]);
        receive_failed_executor
            .apply_transport_probe_rows(&[ready_transport_row(2, 1)])
            .expect("transport row should apply");
        receive_failed_executor
            .apply_candidate_receive_results(&[failed_candidate_receive_result(
                2,
                SPIRE_REMOTE_PRODUCTION_CANDIDATE_DECODE_FAILED,
            )])
            .expect("failed receive row should apply");
        assert!(receive_failed_executor
            .merge_ready_candidate_batches(None)
            .expect_err("receive-failed dispatch should not merge")
            .contains("remote_candidate_receive_failed"));

        let mut cancelled_executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &[planned_dispatch(2, 1)]);
        cancelled_executor.apply_local_query_cancel(SPIRE_REMOTE_PRODUCTION_LOCAL_QUERY_CANCELLED);
        assert!(cancelled_executor
            .merge_ready_candidate_batches(None)
            .expect_err("cancelled dispatch should not merge")
            .contains("remote_executor_cancelled"));
    }

    #[test]
    fn production_executor_compact_receive_requests_use_dispatch_state() {
        let secret_42 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/42").expect("key should build");
        let secret_43 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/43").expect("key should build");
        std::env::set_var(&secret_42, "host=/tmp dbname=postgres");
        std::env::set_var(&secret_43, "host=/tmp dbname=postgres");

        let dispatch_rows = vec![planned_dispatch(42, 2), planned_dispatch(43, 1)];
        let transport_rows = vec![ready_transport_row(42, 1), ready_transport_row(43, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        let requests = executor
            .compact_candidate_receive_requests(&[1.0, 0.0], 4, "strict")
            .expect("request build should succeed");

        std::env::remove_var(&secret_42);
        std::env::remove_var(&secret_43);

        assert_eq!(requests.len(), 2);
        assert_eq!(executor.conninfo_secret_lookup_count, 2);
        assert!(executor
            .dispatches
            .iter()
            .all(|dispatch| dispatch.state == SpireRemoteProductionDispatchState::TransportReady));
        let node_42 = requests
            .iter()
            .find(|request| request.node_id == 42)
            .expect("node 42 request should exist");
        assert_eq!(node_42.remote_index_regclass, "ec_spire_remote_42_idx");
        assert_eq!(node_42.remote_index_identity, vec![42]);
        assert_eq!(node_42.selected_pids, vec![0, 1]);
        assert_eq!(node_42.requested_epoch, 7);
        assert_eq!(node_42.query, vec![1.0, 0.0]);
        assert_eq!(node_42.top_k, 4);
        assert_eq!(node_42.consistency_mode, "strict");
    }

    #[test]
    fn production_executor_compact_receive_request_build_isolates_missing_secret() {
        let secret_52 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/52").expect("key should build");
        let secret_53 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/53").expect("key should build");
        std::env::set_var(&secret_52, "host=/tmp dbname=postgres");
        std::env::remove_var(&secret_53);

        let dispatch_rows = vec![planned_dispatch(52, 1), planned_dispatch(53, 1)];
        let transport_rows = vec![ready_transport_row(52, 1), ready_transport_row(53, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        let requests = executor
            .compact_candidate_receive_requests(&[1.0, 0.0], 3, "strict")
            .expect("request build should isolate missing secrets");

        std::env::remove_var(&secret_52);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].node_id, 52);
        assert_eq!(executor.conninfo_secret_lookup_count, 2);
        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(row.candidate_receive_pending_dispatch_count, 1);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 1);
        assert_eq!(
            row.first_candidate_receive_failure_category,
            "requires_conninfo_secret_resolution"
        );
        assert_eq!(row.next_executor_step, "compact_candidate_receive");
        assert_eq!(row.status, "remote_candidate_receive_failed");
    }

    #[test]
    fn production_executor_degraded_missing_secret_skips_receive_request() {
        let secret_72 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/72").expect("key should build");
        let secret_73 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/73").expect("key should build");
        std::env::set_var(&secret_72, "host=/tmp dbname=postgres");
        std::env::remove_var(&secret_73);

        let dispatch_rows = vec![planned_dispatch(72, 1), planned_dispatch(73, 1)];
        let transport_rows = vec![ready_transport_row(72, 1), ready_transport_row(73, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport rows should apply");
        let requests = executor
            .compact_candidate_receive_requests(&[1.0, 0.0], 3, "degraded")
            .expect("degraded request build should isolate missing secrets");

        std::env::remove_var(&secret_72);

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].node_id, 72);
        assert_eq!(executor.conninfo_secret_lookup_count, 2);
        let row = executor
            .summary("function_argument", "degraded")
            .expect("summary should succeed");
        assert_eq!(row.candidate_receive_pending_dispatch_count, 1);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(row.degraded_skipped_dispatch_count, 1);
        assert_eq!(
            row.first_degraded_skip_category,
            "requires_conninfo_secret_resolution"
        );
        assert_eq!(row.next_executor_step, "compact_candidate_receive");
        assert_eq!(row.status, "requires_compact_candidate_receive");
    }

    #[test]
    fn production_executor_compact_receive_run_applies_adapter_failure() {
        let secret_62 =
            remote_conninfo_secret_provider_lookup_key("spire/remote/62").expect("key should build");
        std::env::set_var(&secret_62, "port=not-a-number dbname=postgres");

        let dispatch_rows = vec![planned_dispatch(62, 1)];
        let transport_rows = vec![ready_transport_row(62, 1)];
        let mut executor =
            SpireRemoteFanoutExecutor::from_libpq_dispatch_rows(7, &dispatch_rows);
        executor
            .apply_transport_probe_rows(&transport_rows)
            .expect("transport row should apply");
        executor
            .run_compact_candidate_receive(&[1.0, 0.0], 3, "strict")
            .expect("adapter failure should stay isolated in executor state");

        std::env::remove_var(&secret_62);

        let row = executor
            .summary("function_argument", "strict")
            .expect("summary should succeed");
        assert_eq!(executor.conninfo_secret_lookup_count, 1);
        assert_eq!(row.candidate_receive_pending_dispatch_count, 0);
        assert_eq!(row.candidate_receive_failed_dispatch_count, 1);
        assert_eq!(row.candidate_row_count, 0);
        assert_eq!(
            row.first_candidate_receive_failure_category,
            "conninfo_parse_failed"
        );
        assert_eq!(row.status, "remote_candidate_receive_failed");
    }

    #[test]
    fn prepare_transaction_capacity_classifier_matches_postgres_errors() {
        assert!(postgres_prepare_transaction_capacity_failure(
            Some("55000"),
            "prepared transactions are disabled"
        ));
        assert!(postgres_prepare_transaction_capacity_failure(
            Some("55000"),
            "object not in prerequisite state"
        ));
        assert!(postgres_prepare_transaction_capacity_failure(
            Some("53300"),
            "maximum number of prepared transactions reached"
        ));
        assert!(postgres_prepare_transaction_capacity_failure(
            Some("53400"),
            "max_prepared_transactions must be increased"
        ));
        assert!(postgres_prepare_transaction_capacity_failure(
            None,
            "maximum number of prepared transactions reached"
        ));
        assert!(!postgres_prepare_transaction_capacity_failure(
            Some("40P01"),
            "deadlock detected"
        ));
        assert!(!postgres_prepare_transaction_capacity_failure(
            Some("53300"),
            "remaining connection slots are reserved"
        ));
    }

    #[test]
    fn prepared_transaction_registration_warning_handles_unresolved_secret() {
        let missing_secret = "spire/tests/prepared-warning/missing";
        let missing_key = remote_conninfo_secret_provider_lookup_key(missing_secret)
            .expect("missing secret lookup key should build");
        std::env::remove_var(&missing_key);
        let missing_warning =
            remote_prepared_transaction_registration_warning(missing_secret, 2)
                .expect("missing secret should warn");
        assert!(missing_warning.contains("max_prepared_transactions preflight"));
        assert!(missing_warning.contains("conninfo_secret_missing"));

        let empty_secret = "spire/tests/prepared-warning/empty";
        let empty_key = remote_conninfo_secret_provider_lookup_key(empty_secret)
            .expect("empty secret lookup key should build");
        std::env::set_var(&empty_key, "");
        let empty_warning =
            remote_prepared_transaction_registration_warning(empty_secret, 3)
                .expect("empty secret should warn");
        std::env::remove_var(&empty_key);
        assert!(empty_warning.contains("max_prepared_transactions preflight"));
        assert!(empty_warning.contains("conninfo_secret_empty"));
    }
}
