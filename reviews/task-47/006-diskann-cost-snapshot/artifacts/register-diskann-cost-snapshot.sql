CREATE OR REPLACE FUNCTION ec_diskann_index_cost_snapshot(index_oid oid)
RETURNS TABLE (
    planner_scan_enabled bool,
    planner_gate_reason text,
    dimensions int,
    graph_degree int,
    build_list_size int,
    relation_list_size int,
    session_list_size int,
    effective_list_size int,
    effective_list_size_source text,
    rerank_budget int,
    top_k int,
    alpha double precision,
    storage_format text,
    resolved_tree_height double precision,
    tree_height_source text,
    index_pages double precision,
    reltuples double precision,
    random_page_cost double precision,
    seq_page_cost double precision,
    cpu_operator_cost double precision,
    modeled_startup_cost double precision,
    modeled_total_cost double precision,
    modeled_selectivity double precision,
    modeled_correlation double precision
)
STRICT STABLE
LANGUAGE c
AS '$libdir/ecaz', 'ec_diskann_index_cost_snapshot_wrapper';
