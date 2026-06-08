\set ON_ERROR_STOP on
-- Narrow PG18 local refresh for Task 87 SPIRE pipeline benchmark wrappers.
-- Generated from installed pgrx SQL; MODULE_PATHNAME is resolved to $libdir/ecaz for manual application.

-- ecaz::ec_spire_index_cost_tuning_snapshot
CREATE  FUNCTION "ec_spire_index_cost_tuning_snapshot"(
	"index_oid" oid /* pgrx_pg_sys::submodules::oids::Oid */
) RETURNS TABLE (
	"storage_format" TEXT,  /* alloc::string::String */
	"effective_rerank_width" INT,  /* i32 */
	"cost_routing_dimension_scale" double precision,  /* f64 */
	"cost_leaf_dimension_scale" double precision,  /* f64 */
	"cost_index_page_scale" double precision,  /* f64 */
	"cost_local_store_page_fanout_scale" double precision,  /* f64 */
	"cost_storage_scoring_multiplier" double precision,  /* f64 */
	"effective_storage_scoring_multiplier" double precision,  /* f64 */
	"cost_rerank_multiplier" double precision,  /* f64 */
	"effective_rerank_multiplier" double precision  /* f64 */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_index_cost_tuning_snapshot_wrapper';

-- ecaz::ec_spire_index_leaf_target_assignment_snapshot
CREATE  FUNCTION "ec_spire_index_leaf_target_assignment_snapshot"(
	"index_oid" oid, /* pgrx_pg_sys::submodules::oids::Oid */
	"target_local_sequences" bigint[] /* alloc::vec::Vec<i64> */
) RETURNS TABLE (
	"active_epoch" bigint,  /* i64 */
	"target_ordinal" bigint,  /* i64 */
	"target_local_sequence" bigint,  /* i64 */
	"status" TEXT,  /* alloc::string::String */
	"leaf_pid" bigint,  /* core::option::Option<i64> */
	"parent_pid" bigint,  /* core::option::Option<i64> */
	"object_version" bigint,  /* core::option::Option<i64> */
	"row_index" bigint,  /* core::option::Option<i64> */
	"assignment_flags" INT  /* core::option::Option<i32> */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_index_leaf_target_assignment_snapshot_wrapper';

-- ecaz::ec_spire_index_placement_snapshot
CREATE  FUNCTION "ec_spire_index_placement_snapshot"(
	"index_oid" oid /* pgrx_pg_sys::submodules::oids::Oid */
) RETURNS TABLE (
	"active_epoch" bigint,  /* i64 */
	"node_id" bigint,  /* i64 */
	"local_store_id" bigint,  /* i64 */
	"store_relid" oid,  /* pgrx_pg_sys::submodules::oids::Oid */
	"placement_count" bigint,  /* i64 */
	"available_placement_count" bigint,  /* i64 */
	"stale_placement_count" bigint,  /* i64 */
	"unavailable_placement_count" bigint,  /* i64 */
	"skipped_placement_count" bigint,  /* i64 */
	"object_count" bigint,  /* i64 */
	"root_object_count" bigint,  /* i64 */
	"internal_object_count" bigint,  /* i64 */
	"leaf_object_count" bigint,  /* i64 */
	"delta_object_count" bigint,  /* i64 */
	"routing_child_count" bigint,  /* i64 */
	"assignment_count" bigint,  /* i64 */
	"placement_object_bytes" bigint,  /* i64 */
	"available_object_bytes" bigint,  /* i64 */
	"routing_object_bytes" bigint,  /* i64 */
	"leaf_object_bytes" bigint,  /* i64 */
	"delta_object_bytes" bigint  /* i64 */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_index_placement_snapshot_wrapper';

-- ecaz::ec_spire_index_scan_leaf_block_rank_snapshot
CREATE  FUNCTION "ec_spire_index_scan_leaf_block_rank_snapshot"(
	"index_oid" oid, /* pgrx_pg_sys::submodules::oids::Oid */
	"query" real[], /* alloc::vec::Vec<f32> */
	"target_local_sequences" bigint[] /* alloc::vec::Vec<i64> */
) RETURNS TABLE (
	"active_epoch" bigint,  /* i64 */
	"effective_nprobe" bigint,  /* i64 */
	"effective_nprobe_source" TEXT,  /* alloc::string::String */
	"effective_rerank_width" bigint,  /* i64 */
	"effective_rerank_width_source" TEXT,  /* alloc::string::String */
	"target_ordinal" bigint,  /* i64 */
	"target_local_sequence" bigint,  /* i64 */
	"status" TEXT,  /* alloc::string::String */
	"max_global_blocks" bigint,  /* i64 */
	"radius_weight" double precision,  /* f64 */
	"scored_block_count" bigint,  /* i64 */
	"block_rank" bigint,  /* core::option::Option<i64> */
	"selected_by_global_cap" bool,  /* core::option::Option<bool> */
	"pid" bigint,  /* core::option::Option<i64> */
	"node_id" bigint,  /* core::option::Option<i64> */
	"local_store_id" bigint,  /* core::option::Option<i64> */
	"object_version" bigint,  /* core::option::Option<i64> */
	"row_index" bigint,  /* core::option::Option<i64> */
	"row_base" bigint,  /* core::option::Option<i64> */
	"row_end" bigint,  /* core::option::Option<i64> */
	"row_count" bigint,  /* core::option::Option<i64> */
	"block_ip" real,  /* core::option::Option<f32> */
	"cap_block_ip" real,  /* core::option::Option<f32> */
	"block_ip_margin_to_cap" real,  /* core::option::Option<f32> */
	"route_rank" bigint,  /* core::option::Option<i64> */
	"route_score" real,  /* core::option::Option<f32> */
	"assignment_flags" bigint  /* core::option::Option<i64> */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_index_scan_leaf_block_rank_snapshot_wrapper';

-- ecaz::ec_spire_index_scan_leaf_candidate_snapshot
CREATE  FUNCTION "ec_spire_index_scan_leaf_candidate_snapshot"(
	"index_oid" oid, /* pgrx_pg_sys::submodules::oids::Oid */
	"query" real[] /* alloc::vec::Vec<f32> */
) RETURNS TABLE (
	"active_epoch" bigint,  /* i64 */
	"effective_nprobe" bigint,  /* i64 */
	"effective_nprobe_source" TEXT,  /* alloc::string::String */
	"effective_rerank_width" bigint,  /* i64 */
	"effective_rerank_width_source" TEXT,  /* alloc::string::String */
	"pid" bigint,  /* i64 */
	"node_id" bigint,  /* i64 */
	"local_store_id" bigint,  /* i64 */
	"object_version" bigint,  /* i64 */
	"object_bytes" bigint,  /* i64 */
	"route_count" bigint,  /* i64 */
	"scanned_count" bigint,  /* i64 */
	"candidate_row_count" bigint,  /* i64 */
	"leaf_block_available_count" bigint,  /* i64 */
	"leaf_block_selected_count" bigint,  /* i64 */
	"leaf_block_skipped_count" bigint,  /* i64 */
	"leaf_summary_object_bytes" bigint,  /* i64 */
	"leaf_row_object_bytes" bigint,  /* i64 */
	"primary_candidate_row_count" bigint,  /* i64 */
	"boundary_replica_candidate_row_count" bigint,  /* i64 */
	"deduped_candidate_row_count" bigint,  /* i64 */
	"truncated_candidate_row_count" bigint,  /* i64 */
	"candidate_winner_count" bigint,  /* i64 */
	"leaf_object_read_nanos" bigint,  /* i64 */
	"leaf_summary_score_nanos" bigint,  /* i64 */
	"leaf_row_score_nanos" bigint,  /* i64 */
	"candidate_score_nanos" bigint,  /* i64 */
	"candidate_materialize_nanos" bigint,  /* i64 */
	"candidate_heap_append_nanos" bigint,  /* i64 */
	"leaf_row_segment_read_count" bigint,  /* i64 */
	"leaf_row_segment_read_bytes" bigint  /* i64 */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_index_scan_leaf_candidate_snapshot_wrapper';

-- ecaz::ec_spire_index_scan_leaf_target_block_rank_snapshot
CREATE  FUNCTION "ec_spire_index_scan_leaf_target_block_rank_snapshot"(
	"index_oid" oid, /* pgrx_pg_sys::submodules::oids::Oid */
	"query" real[], /* alloc::vec::Vec<f32> */
	"target_local_sequences" bigint[] /* alloc::vec::Vec<i64> */
) RETURNS TABLE (
	"active_epoch" bigint,  /* i64 */
	"effective_nprobe" bigint,  /* i64 */
	"effective_nprobe_source" TEXT,  /* alloc::string::String */
	"effective_rerank_width" bigint,  /* i64 */
	"effective_rerank_width_source" TEXT,  /* alloc::string::String */
	"target_ordinal" bigint,  /* i64 */
	"target_local_sequence" bigint,  /* i64 */
	"status" TEXT,  /* alloc::string::String */
	"max_global_blocks" bigint,  /* i64 */
	"radius_weight" double precision,  /* f64 */
	"scored_block_count" bigint,  /* i64 */
	"block_rank" bigint,  /* core::option::Option<i64> */
	"selected_by_global_cap" bool,  /* core::option::Option<bool> */
	"pid" bigint,  /* core::option::Option<i64> */
	"node_id" bigint,  /* core::option::Option<i64> */
	"local_store_id" bigint,  /* core::option::Option<i64> */
	"object_version" bigint,  /* core::option::Option<i64> */
	"row_index" bigint,  /* core::option::Option<i64> */
	"row_base" bigint,  /* core::option::Option<i64> */
	"row_end" bigint,  /* core::option::Option<i64> */
	"row_count" bigint,  /* core::option::Option<i64> */
	"block_ip" real,  /* core::option::Option<f32> */
	"cap_block_ip" real,  /* core::option::Option<f32> */
	"block_ip_margin_to_cap" real,  /* core::option::Option<f32> */
	"route_rank" bigint,  /* core::option::Option<i64> */
	"route_score" real,  /* core::option::Option<f32> */
	"assignment_flags" bigint  /* core::option::Option<i64> */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_index_scan_leaf_target_block_rank_snapshot_wrapper';

-- ecaz::ec_spire_index_scan_local_store_read_overlap_harness
CREATE  FUNCTION "ec_spire_index_scan_local_store_read_overlap_harness"(
	"index_oid" oid, /* pgrx_pg_sys::submodules::oids::Oid */
	"query" real[] /* alloc::vec::Vec<f32> */
) RETURNS TABLE (
	"active_epoch" bigint,  /* i64 */
	"effective_nprobe" bigint,  /* i64 */
	"node_id" bigint,  /* i64 */
	"local_store_id" bigint,  /* i64 */
	"route_count" bigint,  /* i64 */
	"leaf_route_count" bigint,  /* i64 */
	"delta_route_count" bigint,  /* i64 */
	"candidate_row_count" bigint,  /* i64 */
	"prefetched_object_bytes" bigint,  /* i64 */
	"read_batch_count" bigint,  /* i64 */
	"delta_decode_count" bigint  /* i64 */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_index_scan_local_store_read_overlap_harness_wrapper';

-- ecaz::ec_spire_index_scan_pipeline_snapshot
CREATE  FUNCTION "ec_spire_index_scan_pipeline_snapshot"(
	"index_oid" oid, /* pgrx_pg_sys::submodules::oids::Oid */
	"query" real[] /* alloc::vec::Vec<f32> */
) RETURNS TABLE (
	"step_ordinal" bigint,  /* i64 */
	"step_name" TEXT,  /* &str */
	"active_epoch" bigint,  /* i64 */
	"status" TEXT,  /* &str */
	"item_count" bigint,  /* i64 */
	"ready_count" bigint,  /* i64 */
	"blocked_count" bigint,  /* i64 */
	"route_count" bigint,  /* i64 */
	"candidate_count" bigint,  /* i64 */
	"heap_rerank_row_count" bigint,  /* i64 */
	"remote_fanout_count" bigint,  /* i64 */
	"next_blocker" TEXT,  /* &str */
	"recommendation" TEXT  /* &str */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_index_scan_pipeline_snapshot_wrapper';

-- ecaz::ec_spire_index_scan_rerank_locality_snapshot
CREATE  FUNCTION "ec_spire_index_scan_rerank_locality_snapshot"(
	"index_oid" oid, /* pgrx_pg_sys::submodules::oids::Oid */
	"query" real[] /* alloc::vec::Vec<f32> */
) RETURNS TABLE (
	"active_epoch" bigint,  /* i64 */
	"effective_nprobe" bigint,  /* i64 */
	"effective_nprobe_source" TEXT,  /* alloc::string::String */
	"effective_rerank_width" bigint,  /* i64 */
	"effective_rerank_width_source" TEXT,  /* alloc::string::String */
	"candidate_count" bigint,  /* i64 */
	"rerank_prefix_count" bigint,  /* i64 */
	"unique_heap_block_count" bigint,  /* i64 */
	"heap_block_transition_count" bigint,  /* i64 */
	"heap_block_span" bigint,  /* i64 */
	"heap_block_jump_sum" bigint,  /* i64 */
	"heap_block_jump_max" bigint  /* i64 */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_index_scan_rerank_locality_snapshot_wrapper';

-- ecaz::ec_spire_index_scan_routing_snapshot
CREATE  FUNCTION "ec_spire_index_scan_routing_snapshot"(
	"index_oid" oid, /* pgrx_pg_sys::submodules::oids::Oid */
	"query" real[] /* alloc::vec::Vec<f32> */
) RETURNS TABLE (
	"active_epoch" bigint,  /* i64 */
	"effective_nprobe" bigint,  /* i64 */
	"effective_nprobe_source" TEXT,  /* alloc::string::String */
	"adaptive_nprobe_decision" TEXT,  /* alloc::string::String */
	"recursive_beam_width" bigint,  /* i64 */
	"max_leaf_routes" bigint,  /* i64 */
	"max_routing_expansions" bigint,  /* i64 */
	"routing_level" bigint,  /* i64 */
	"input_frontier_width" bigint,  /* i64 */
	"expanded_parent_count" bigint,  /* i64 */
	"selected_child_count" bigint,  /* i64 */
	"deduped_route_count" bigint,  /* i64 */
	"truncation_reason" TEXT  /* alloc::string::String */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_index_scan_routing_snapshot_wrapper';

-- ecaz::ec_spire_remote_node_snapshot
CREATE  FUNCTION "ec_spire_remote_node_snapshot"(
	"index_oid" oid /* pgrx_pg_sys::submodules::oids::Oid */
) RETURNS TABLE (
	"active_epoch" bigint,  /* i64 */
	"node_id" bigint,  /* i64 */
	"node_kind" TEXT,  /* &str */
	"descriptor_generation" bigint,  /* i64 */
	"descriptor_state" TEXT,  /* &str */
	"placement_count" bigint,  /* i64 */
	"available_placement_count" bigint,  /* i64 */
	"stale_placement_count" bigint,  /* i64 */
	"unavailable_placement_count" bigint,  /* i64 */
	"skipped_placement_count" bigint,  /* i64 */
	"local_store_count" bigint,  /* i64 */
	"last_seen_at_micros" bigint,  /* i64 */
	"last_served_epoch" bigint,  /* i64 */
	"min_retained_epoch" bigint,  /* i64 */
	"extension_version" TEXT,  /* alloc::string::String */
	"last_error" TEXT,  /* alloc::string::String */
	"status" TEXT,  /* &str */
	"recommendation" TEXT  /* &str */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_remote_node_snapshot_wrapper';

-- ecaz::ec_spire_remote_pipeline_steps
CREATE  FUNCTION "ec_spire_remote_pipeline_steps"(
	"index_oid" oid, /* pgrx_pg_sys::submodules::oids::Oid */
	"requested_epoch" bigint, /* i64 */
	"query" real[], /* alloc::vec::Vec<f32> */
	"selected_pids" bigint[], /* alloc::vec::Vec<i64> */
	"top_k" INT, /* i32 */
	"consistency_mode" TEXT /* alloc::string::String */
) RETURNS TABLE (
	"step_ordinal" bigint,  /* i64 */
	"step_name" TEXT,  /* &str */
	"requested_epoch" bigint,  /* i64 */
	"status" TEXT,  /* alloc::string::String */
	"item_count" bigint,  /* i64 */
	"ready_count" bigint,  /* i64 */
	"blocked_count" bigint,  /* i64 */
	"remote_pid_count" bigint,  /* i64 */
	"next_blocker" TEXT,  /* alloc::string::String */
	"recommendation" TEXT  /* alloc::string::String */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_remote_pipeline_steps_wrapper';

-- ecaz::ec_spire_remote_search_degraded_skip_report
CREATE  FUNCTION "ec_spire_remote_search_degraded_skip_report"(
	"index_oid" oid, /* pgrx_pg_sys::submodules::oids::Oid */
	"requested_epoch" bigint, /* i64 */
	"query" real[], /* alloc::vec::Vec<f32> */
	"selected_pids" bigint[], /* alloc::vec::Vec<i64> */
	"top_k" INT, /* i32 */
	"consistency_mode" TEXT /* alloc::string::String */
) RETURNS TABLE (
	"requested_epoch" bigint,  /* i64 */
	"node_id" bigint,  /* i64 */
	"skipped_pid_count" bigint,  /* i64 */
	"first_skip_category" TEXT,  /* &str */
	"first_skip_hint" TEXT,  /* &str */
	"status" TEXT  /* &str */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_remote_search_degraded_skip_report_wrapper';

-- ecaz::ec_spire_remote_search_endpoint_identity
CREATE  FUNCTION "ec_spire_remote_search_endpoint_identity"(
	"index_oid" oid /* pgrx_pg_sys::submodules::oids::Oid */
) RETURNS TABLE (
	"protocol_version" TEXT,  /* &str */
	"extension_version" TEXT,  /* &str */
	"opclass_identity" TEXT,  /* alloc::string::String */
	"storage_format" TEXT,  /* &str */
	"assignment_payload_format" TEXT,  /* &str */
	"quantizer_profile" TEXT,  /* &str */
	"scoring_profile" TEXT,  /* &str */
	"tuple_transport_capabilities" TEXT[],  /* alloc::vec::Vec<alloc::string::String> */
	"tuple_transport_default" TEXT,  /* &str */
	"tuple_transport_status" TEXT,  /* &str */
	"profile_fingerprint" TEXT,  /* alloc::string::String */
	"status" TEXT,  /* &str */
	"recommendation" TEXT  /* &str */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_remote_search_endpoint_identity_wrapper';

-- ecaz::ec_spire_remote_search_production_read_profile
CREATE  FUNCTION "ec_spire_remote_search_production_read_profile"(
	"index_oid" oid, /* pgrx_pg_sys::submodules::oids::Oid */
	"query" real[], /* alloc::vec::Vec<f32> */
	"top_k" INT /* i32 */
) RETURNS TABLE (
	"metric" TEXT,  /* alloc::string::String */
	"value" TEXT  /* alloc::string::String */
)
STRICT STABLE
LANGUAGE c /* Rust */
AS '$libdir/ecaz', 'ec_spire_remote_search_production_read_profile_wrapper';
