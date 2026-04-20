DO $sql$
DECLARE
    module_path text;
BEGIN
    SELECT p.probin
      INTO module_path
      FROM pg_proc AS p
      JOIN pg_namespace AS n
        ON n.oid = p.pronamespace
     WHERE n.nspname = 'tests'
       AND p.proname = 'ec_hnsw_debug_scan_result_count'
     LIMIT 1;

    IF module_path IS NULL THEN
        RAISE EXCEPTION 'could not resolve tqvector module path from existing tests schema wrappers';
    END IF;

    EXECUTE 'DROP FUNCTION IF EXISTS tests."ec_hnsw_debug_scan_profile_limited"(oid, real[], integer)';

    EXECUTE format(
        $fmt$
        CREATE FUNCTION tests."ec_hnsw_debug_scan_profile_limited"(
            index_oid oid,
            query real[],
            limit_count integer
        ) RETURNS TABLE (
            rescan_elapsed_us bigint,
            emit_elapsed_us bigint,
            total_elapsed_us bigint,
            rescan_phase text,
            rescan_current_result boolean,
            rescan_ordered_slots integer,
            rescan_pending_heap_tids integer,
            rescan_visited_elements integer,
            rescan_expanded_sources integer,
            rescan_emitted_elements integer,
            rescan_bootstrap_expansions integer,
            rescan_bootstrap_pages_read integer,
            rescan_quantizer_cache_hit boolean,
            result_count integer,
            final_phase text,
            final_ordered_slots integer,
            total_bootstrap_expansions integer,
            total_bootstrap_pages_read integer,
            total_linear_pages_read integer,
            total_elements_scored integer,
            total_elements_skipped integer,
            total_heap_tids_returned integer,
            total_quantizer_cache_hit boolean,
            total_emitted_elements integer
        )
        LANGUAGE c STRICT
        AS %L, 'ec_hnsw_debug_scan_profile_limited_wrapper'
        $fmt$,
        module_path
    );

    EXECUTE 'DROP FUNCTION IF EXISTS tests."ec_hnsw_debug_scan_hot_path_profile"(oid, real[])';

    EXECUTE format(
        $fmt$
        CREATE FUNCTION tests."ec_hnsw_debug_scan_hot_path_profile"(
            index_oid oid,
            query real[]
        ) RETURNS TABLE (
            rescan_amrescan_total_elapsed_us bigint,
            rescan_query_decode_elapsed_us bigint,
            rescan_scan_setup_elapsed_us bigint,
            rescan_store_query_elapsed_us bigint,
            rescan_prepare_query_elapsed_us bigint,
            rescan_reset_state_elapsed_us bigint,
            rescan_initialize_entry_elapsed_us bigint,
            rescan_upper_layer_seed_elapsed_us bigint,
            rescan_layer0_seed_elapsed_us bigint,
            rescan_stage_ordered_results_elapsed_us bigint,
            rescan_initial_prefetch_elapsed_us bigint,
            rescan_frontier_consume_elapsed_us bigint,
            rescan_graph_result_materialize_elapsed_us bigint,
            graph_element_cache_hits integer,
            graph_element_cache_misses integer,
            graph_element_load_elapsed_us bigint,
            graph_neighbor_cache_hits integer,
            graph_neighbor_cache_misses integer,
            graph_neighbor_load_elapsed_us bigint,
            candidate_score_calls integer,
            candidate_score_elapsed_us bigint,
            score_cache_hits integer,
            score_cache_misses integer,
            grouped_traversal_approx_score_calls integer,
            grouped_traversal_approx_score_elapsed_us bigint,
            grouped_traversal_exact_score_calls integer,
            grouped_traversal_exact_score_elapsed_us bigint,
            grouped_traversal_budgeted_expansions integer,
            grouped_traversal_budgeted_candidates integer,
            grouped_traversal_budgeted_exact_candidates integer
        )
        LANGUAGE c STRICT
        AS %L, 'ec_hnsw_debug_scan_hot_path_profile_wrapper'
        $fmt$,
        module_path
    );

    EXECUTE 'DROP FUNCTION IF EXISTS tests."ec_hnsw_debug_scan_heap_fetch_profile"(oid, real[], integer, integer)';

    EXECUTE format(
        $fmt$
        CREATE FUNCTION tests."ec_hnsw_debug_scan_heap_fetch_profile"(
            index_oid oid,
            query real[],
            limit_count integer,
            project_attnum integer
        ) RETURNS TABLE (
            rescan_elapsed_us bigint,
            emit_elapsed_us bigint,
            total_elapsed_us bigint,
            slot_fetch_elapsed_us bigint,
            projection_elapsed_us bigint,
            result_count integer,
            slot_fetch_count integer,
            projected_count integer
        )
        LANGUAGE c STRICT
        AS %L, 'ec_hnsw_debug_scan_heap_fetch_profile_wrapper'
        $fmt$,
        module_path
    );

    EXECUTE 'DROP FUNCTION IF EXISTS tests."ec_hnsw_debug_grouped_rerank_profile"(oid, real[])';
    EXECUTE 'DROP FUNCTION IF EXISTS tests."ec_hnsw_debug_grouped_rerank_profile"(oid, real[], integer)';

    EXECUTE format(
        $fmt$
        CREATE FUNCTION tests."ec_hnsw_debug_grouped_rerank_profile"(
            index_oid oid,
            query real[],
            limit_count integer
        ) RETURNS TABLE (
            rescan_amrescan_total_elapsed_us bigint,
            rescan_graph_result_materialize_elapsed_us bigint,
            emit_elapsed_us bigint,
            total_elapsed_us bigint,
            result_count integer,
            grouped_rerank_quantized_score_calls integer,
            grouped_rerank_quantized_score_elapsed_us bigint,
            grouped_rerank_heap_score_calls integer,
            grouped_rerank_heap_score_elapsed_us bigint,
            grouped_rerank_heap_rows_fetched integer,
            grouped_rerank_heap_fetch_elapsed_us bigint,
            grouped_rerank_heap_decode_elapsed_us bigint,
            grouped_rerank_heap_dot_elapsed_us bigint
        )
        LANGUAGE c STRICT
        AS %L, 'ec_hnsw_debug_grouped_rerank_profile_wrapper'
        $fmt$,
        module_path
    );

    EXECUTE 'DROP FUNCTION IF EXISTS tests."ec_hnsw_debug_pack_f32_bytea"(real[])';

    EXECUTE format(
        $fmt$
        CREATE FUNCTION tests."ec_hnsw_debug_pack_f32_bytea"(real[])
        RETURNS bytea
        LANGUAGE c STRICT
        AS %L, 'ec_hnsw_debug_pack_f32_bytea_wrapper'
        $fmt$,
        module_path
    );

    EXECUTE 'DROP FUNCTION IF EXISTS tests."ec_hnsw_debug_adr030_runtime_settings"()';

    EXECUTE format(
        $fmt$
        CREATE FUNCTION tests."ec_hnsw_debug_adr030_runtime_settings"()
        RETURNS TABLE (
            grouped_build_enabled boolean,
            grouped_scan_enabled boolean,
            grouped_scan_window text,
            grouped_scan_score_mode text,
            grouped_scan_rerank_mode text,
            grouped_scan_rerank_source_column text,
            grouped_exact_traversal_enabled boolean,
            grouped_exact_traversal_scope text,
            grouped_exact_traversal_strategy text,
            grouped_exact_traversal_limit text
        )
        LANGUAGE c
        AS %L, 'ec_hnsw_debug_adr030_runtime_settings_wrapper'
        $fmt$,
        module_path
    );
END
$sql$;
