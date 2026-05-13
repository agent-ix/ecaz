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
const SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED: &str = "tuple_transport_retired";
const SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE: &str = "remote_payload_too_large";
const SPIRE_REMOTE_STATUS_SCHEMA_DRIFT: &str = "schema_drift";
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
const SPIRE_REMOTE_TUPLE_TRANSPORT_RETIRED_HINT: &str =
    "upgrade the remote ecaz extension, refresh the descriptor, and ensure tuple_transport_capabilities includes pg_binary_attr_v1";
const SPIRE_REMOTE_PAYLOAD_TOO_LARGE_HINT: &str =
    "reduce remote tuple projection width or raise ec_spire.max_remote_payload_bytes_per_row / ec_spire.max_remote_payload_rows_per_batch with packet-local benchmark evidence";
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
const SPIRE_PREPARED_XACT_INTENT_PREPARE_REQUESTED: &str = "prepare_requested";
const SPIRE_PREPARED_XACT_INTENT_PREPARE_ACKED: &str = "prepare_acked";
const SPIRE_PREPARED_XACT_INTENT_COMMIT_LOCAL: &str = "commit_local";
const SPIRE_PREPARED_XACT_INTENT_ROLLBACK_LOCAL: &str = "rollback_local";
const SPIRE_PREPARED_XACT_REAPER_ROLLED_BACK: &str = "rolled_back";
const SPIRE_PREPARED_XACT_REAPER_ROLLED_BACK_MISSING_INTENT: &str =
    "rolled_back_missing_intent";
const SPIRE_PREPARED_XACT_REAPER_SKIPPED_COMMIT_LOCAL: &str = "skipped_commit_local";
const SPIRE_PREPARED_XACT_REAPER_SKIPPED_XID_LIVE: &str = "skipped_xid_still_live";
const SPIRE_PREPARED_XACT_REAPER_SKIPPED_NODE_MISMATCH: &str = "skipped_node_mismatch";
const SPIRE_PREPARED_XACT_REAPER_SKIPPED_UNPARSEABLE_GID: &str = "skipped_unparseable_gid";
const SPIRE_PREPARED_XACT_REAPER_ROLLBACK_FAILED: &str = "rollback_failed";
