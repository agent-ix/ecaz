BEGIN;

ALTER EXTENSION ecaz DROP FUNCTION ec_spire_index_scan_leaf_candidate_snapshot(oid, real[]);

DROP FUNCTION IF EXISTS ec_spire_index_scan_leaf_candidate_snapshot(oid, real[]);

CREATE FUNCTION ec_spire_index_scan_leaf_candidate_snapshot(
    index_oid oid,
    query real[]
) RETURNS TABLE (
    active_epoch bigint,
    effective_nprobe bigint,
    effective_nprobe_source text,
    effective_rerank_width bigint,
    effective_rerank_width_source text,
    pid bigint,
    node_id bigint,
    local_store_id bigint,
    object_version bigint,
    object_bytes bigint,
    route_count bigint,
    scanned_count bigint,
    candidate_row_count bigint,
    leaf_block_available_count bigint,
    leaf_block_selected_count bigint,
    leaf_block_skipped_count bigint,
    leaf_summary_object_bytes bigint,
    leaf_row_object_bytes bigint,
    primary_candidate_row_count bigint,
    boundary_replica_candidate_row_count bigint,
    deduped_candidate_row_count bigint,
    truncated_candidate_row_count bigint,
    candidate_winner_count bigint,
    leaf_object_read_nanos bigint,
    leaf_summary_score_nanos bigint,
    leaf_row_score_nanos bigint,
    candidate_score_nanos bigint,
    candidate_materialize_nanos bigint,
    candidate_heap_append_nanos bigint,
    leaf_row_segment_read_count bigint,
    leaf_row_segment_read_bytes bigint
)
STRICT STABLE
LANGUAGE c
AS '$libdir/ecaz', 'ec_spire_index_scan_leaf_candidate_snapshot_wrapper';

ALTER EXTENSION ecaz ADD FUNCTION ec_spire_index_scan_leaf_candidate_snapshot(oid, real[]);

COMMIT;
