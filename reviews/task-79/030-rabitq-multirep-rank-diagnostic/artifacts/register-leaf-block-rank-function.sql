CREATE OR REPLACE FUNCTION "ec_spire_index_scan_leaf_block_rank_snapshot"(
    "index_oid" oid,
    "query" real[],
    "target_local_sequences" bigint[]
) RETURNS TABLE (
    "active_epoch" bigint,
    "effective_nprobe" bigint,
    "effective_nprobe_source" TEXT,
    "effective_rerank_width" bigint,
    "effective_rerank_width_source" TEXT,
    "target_ordinal" bigint,
    "target_local_sequence" bigint,
    "status" TEXT,
    "max_global_blocks" bigint,
    "radius_weight" double precision,
    "scored_block_count" bigint,
    "block_rank" bigint,
    "selected_by_global_cap" bool,
    "pid" bigint,
    "node_id" bigint,
    "local_store_id" bigint,
    "object_version" bigint,
    "row_index" bigint,
    "row_base" bigint,
    "row_end" bigint,
    "row_count" bigint,
    "block_ip" real,
    "assignment_flags" bigint
)
STRICT STABLE
LANGUAGE c
AS '$libdir/ecaz', 'ec_spire_index_scan_leaf_block_rank_snapshot_wrapper';
