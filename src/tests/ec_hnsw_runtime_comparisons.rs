    #[pg_test]
    fn test_pq_fastscan_runtime_profile_frontier_head_exact_counters() {
        let _lock = env_var_test_lock();
        let _exact_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL", "1");
        let _scope_guard =
            ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_SCOPE", "layer0");
        let _strategy_guard = ScopedEnvVar::set(
            "TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_STRATEGY",
            "frontier_head",
        );
        let index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_profile_frontier_head_exact",
            "ec_hnsw_pq_fastscan_runtime_profile_frontier_head_exact_idx",
        );
        let (
            _rescan_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            _rescan_phase,
            _rescan_current_result,
            _rescan_ordered_slots,
            _rescan_pending_heap_tids,
            _rescan_visited_elements,
            _rescan_expanded_sources,
            _rescan_emitted_elements,
            _rescan_bootstrap_expansions,
            _rescan_bootstrap_pages_read,
            _rescan_quantizer_cache_hit,
            _result_count,
            _final_phase,
            _final_ordered_slots,
            _total_bootstrap_expansions,
            _total_bootstrap_pages_read,
            _total_linear_pages_read,
            _total_elements_scored,
            _total_elements_skipped,
            _total_heap_tids_returned,
            _total_quantizer_cache_hit,
            _total_emitted_elements,
            _rescan_amrescan_total_elapsed_us,
            _rescan_query_decode_elapsed_us,
            _rescan_scan_setup_elapsed_us,
            _rescan_store_query_elapsed_us,
            _rescan_prepare_query_elapsed_us,
            _rescan_reset_state_elapsed_us,
            _rescan_initialize_entry_elapsed_us,
            _rescan_upper_layer_seed_elapsed_us,
            _rescan_layer0_seed_elapsed_us,
            _rescan_stage_ordered_results_elapsed_us,
            _rescan_initial_prefetch_elapsed_us,
            _rescan_frontier_consume_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _graph_element_cache_hits,
            _graph_element_cache_misses,
            _graph_element_load_elapsed_us,
            _graph_neighbor_cache_hits,
            _graph_neighbor_cache_misses,
            _graph_neighbor_load_elapsed_us,
            _candidate_score_calls,
            _candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ) = unsafe { am::debug_profile_ordered_scan(index_oid, pq_fastscan_runtime_query()) };

        assert!(
            grouped_traversal_approx_score_calls > 0
                && grouped_traversal_approx_score_elapsed_us >= 0,
            "frontier-head grouped exact traversal should still score grouped approximate candidates first",
        );
        assert!(
            grouped_traversal_exact_score_calls > 0
                && grouped_traversal_exact_score_elapsed_us >= 0,
            "frontier-head grouped exact traversal should surface exact rescoring work",
        );
        let _ = score_cache_hits;
        assert!(
            score_cache_misses > 0,
            "frontier-head grouped exact traversal should still populate the scan-local exact cache",
        );
        assert_eq!(
            (
                grouped_traversal_budgeted_expansions,
                grouped_traversal_budgeted_candidates,
                grouped_traversal_budgeted_exact_candidates,
            ),
            (0, 0, 0),
            "frontier-head grouped exact traversal should not use the per-expansion budget path",
        );
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw PqFastScan traversal score mode must be one of [pq, binary], got \"bogus\""
    )]
    fn test_pq_fastscan_traversal_score_mode_rejects_invalid_env() {
        let _lock = env_var_test_lock();
        let _score_mode_guard =
            ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE", "bogus");
        let index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_invalid_score_mode",
            "ec_hnsw_pq_fastscan_runtime_invalid_score_mode_idx",
        );

        let _ = unsafe { am::debug_profile_ordered_scan(index_oid, pq_fastscan_runtime_query()) };
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw grouped rerank mode must be one of [quantized, heap_f32], got \"bogus\""
    )]
    fn test_pq_fastscan_rerank_mode_rejects_invalid_env() {
        let _lock = env_var_test_lock();
        let _rerank_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", "bogus");
        let index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_invalid_rerank_mode",
            "ec_hnsw_pq_fastscan_runtime_invalid_rerank_mode_idx",
        );

        let _ = unsafe { am::debug_profile_ordered_scan(index_oid, pq_fastscan_runtime_query()) };
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw PqFastScan exact traversal scope must be one of [all, layer0], got \"bogus\""
    )]
    fn test_pq_fastscan_exact_traversal_rejects_invalid_scope_env() {
        let _lock = env_var_test_lock();
        let _exact_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL", "1");
        let _scope_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_SCOPE", "bogus");

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_invalid_exact_scope (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 41 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_invalid_exact_scope VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_invalid_exact_scope_idx ON ec_hnsw_pq_fastscan_runtime_invalid_exact_scope USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let _ = Spi::get_one::<i64>(
            "SELECT id FROM ec_hnsw_pq_fastscan_runtime_invalid_exact_scope \
             ORDER BY embedding <#> ARRAY[0.5, 0.1, 0.4, -0.8, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, -0.1, -0.2, -0.3, -0.4]::real[] \
             LIMIT 1",
        )
        .expect("ordered scan should reach amrescan before rejecting invalid grouped exact traversal scope");
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw PqFastScan exact traversal strategy must be one of [expansion, frontier_head], got \"bogus\""
    )]
    fn test_pq_fastscan_exact_traversal_rejects_invalid_strategy_env() {
        let _lock = env_var_test_lock();
        let _exact_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL", "1");
        let _scope_guard =
            ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_SCOPE", "layer0");
        let _strategy_guard =
            ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_STRATEGY", "bogus");

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_invalid_exact_strategy (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 41 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_invalid_exact_strategy VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_invalid_exact_strategy_idx ON ec_hnsw_pq_fastscan_runtime_invalid_exact_strategy USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let _ = Spi::get_one::<i64>(
            "SELECT id FROM ec_hnsw_pq_fastscan_runtime_invalid_exact_strategy \
             ORDER BY embedding <#> ARRAY[0.5, 0.1, 0.4, -0.8, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, -0.1, -0.2, -0.3, -0.4]::real[] \
             LIMIT 1",
        )
        .expect("ordered scan should reach amrescan before rejecting invalid grouped exact traversal strategy");
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw PqFastScan exact traversal limit must be a positive integer, got bogus"
    )]
    fn test_pq_fastscan_exact_traversal_rejects_invalid_limit_env() {
        let _lock = env_var_test_lock();
        let _exact_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL", "1");
        let _limit_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_LIMIT", "bogus");

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_invalid_exact_limit (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 43 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 31 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_invalid_exact_limit VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_invalid_exact_limit_idx ON ec_hnsw_pq_fastscan_runtime_invalid_exact_limit USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let _ = Spi::get_one::<i64>(
            "SELECT id FROM ec_hnsw_pq_fastscan_runtime_invalid_exact_limit \
             ORDER BY embedding <#> ARRAY[0.5, 0.1, 0.4, -0.8, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, -0.1, -0.2, -0.3, -0.4]::real[] \
             LIMIT 1",
        )
        .expect("ordered scan should reach amrescan before rejecting invalid grouped exact traversal limit");
    }

    #[pg_test]
    fn test_pq_fastscan_comparison_summary_matches_rows() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_summary (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 37 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 23 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_summary VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_summary_idx ON ec_hnsw_pq_fastscan_runtime_summary USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_runtime_summary_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.15_f32, 0.25, 0.35, 0.45, 0.55, 0.65, 0.75, 0.85, 0.95, 1.05, 1.15, 1.25, 1.35, 1.45,
            1.55, 1.65,
        ];
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query.clone())
        };

        let compared_rows = observed
            .iter()
            .filter_map(
                |(_heap_tid, approx_score, comparison_score, _approx_rank)| {
                    comparison_score.map(|exact_score| (*approx_score, exact_score))
                },
            )
            .collect::<Vec<_>>();
        let expected_emitted_result_count =
            i32::try_from(observed.len()).expect("emitted result count should fit in i32");
        let expected_grouped_result_count = expected_emitted_result_count;
        let expected_compared_result_count =
            i32::try_from(compared_rows.len()).expect("compared result count should fit in i32");
        let expected_missing_comparison_count =
            expected_grouped_result_count - expected_compared_result_count;
        let expected_mean_abs_score_delta = if compared_rows.is_empty() {
            0.0
        } else {
            compared_rows
                .iter()
                .map(|(approx_score, exact_score)| f64::from((approx_score - exact_score).abs()))
                .sum::<f64>()
                / f64::from(expected_compared_result_count)
        };
        let expected_max_abs_score_delta = compared_rows
            .iter()
            .map(|(approx_score, exact_score)| (approx_score - exact_score).abs())
            .fold(0.0_f32, f32::max);
        let expected_mean_signed_score_delta = if compared_rows.is_empty() {
            0.0
        } else {
            compared_rows
                .iter()
                .map(|(approx_score, exact_score)| f64::from(approx_score - exact_score))
                .sum::<f64>()
                / f64::from(expected_compared_result_count)
        };

        let query_literal = format_recall_vector_sql_literal(&query);
        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            missing_comparison_count,
            mean_abs_score_delta,
            max_abs_score_delta,
            mean_signed_score_delta,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    &format!(
                        "SELECT
                            emitted_result_count,
                            pq_fastscan_result_count,
                            compared_result_count,
                            missing_comparison_count,
                            mean_abs_score_delta,
                            max_abs_score_delta,
                            mean_signed_score_delta
                         FROM tests.ec_hnsw_debug_pq_fastscan_scan_comparison_summary(
                            'ec_hnsw_pq_fastscan_runtime_summary_idx'::regclass::oid,
                            {query_literal}
                         )"
                    ),
                    None,
                    &[],
                )
                .expect("grouped comparison summary query should succeed")
                .next()
                .expect("grouped comparison summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["pq_fastscan_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["missing_comparison_count"]
                    .value::<i32>()
                    .expect("missing comparison count should decode")
                    .expect("missing comparison count should be non-null"),
                row["mean_abs_score_delta"]
                    .value::<f64>()
                    .expect("mean abs score delta should decode")
                    .expect("mean abs score delta should be non-null"),
                row["max_abs_score_delta"]
                    .value::<f32>()
                    .expect("max abs score delta should decode")
                    .expect("max abs score delta should be non-null"),
                row["mean_signed_score_delta"]
                    .value::<f64>()
                    .expect("mean signed score delta should decode")
                    .expect("mean signed score delta should be non-null"),
            )
        });

        assert_eq!(emitted_result_count, expected_emitted_result_count);
        assert_eq!(grouped_result_count, expected_grouped_result_count);
        assert_eq!(compared_result_count, expected_compared_result_count);
        assert_eq!(missing_comparison_count, expected_missing_comparison_count);
        assert!(
            (mean_abs_score_delta - expected_mean_abs_score_delta).abs() <= 1e-6,
            "mean abs grouped score delta should match the emitted-row summary"
        );
        assert!(
            (max_abs_score_delta - expected_max_abs_score_delta).abs() <= f32::EPSILON,
            "max abs grouped score delta should match the emitted-row summary"
        );
        assert!(
            (mean_signed_score_delta - expected_mean_signed_score_delta).abs() <= 1e-6,
            "mean signed grouped score delta should match the emitted-row summary"
        );
    }

    #[pg_test]
    fn test_scalar_runtime_summary_reports_no_grouped_comparisons() {
        Spi::run(
            "CREATE TABLE ec_hnsw_scalar_runtime_summary (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        Spi::run(
            "INSERT INTO ec_hnsw_scalar_runtime_summary VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.5, 0.5, 0.25, -0.75], 4, 42)),
             (4, encode_to_ecvector(ARRAY[-0.25, 0.9, 0.1, -0.4], 4, 42))",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX ec_hnsw_scalar_runtime_summary_idx ON ec_hnsw_scalar_runtime_summary USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            missing_comparison_count,
            mean_abs_score_delta,
            max_abs_score_delta,
            mean_signed_score_delta,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    "SELECT
                        emitted_result_count,
                        pq_fastscan_result_count,
                        compared_result_count,
                        missing_comparison_count,
                        mean_abs_score_delta,
                        max_abs_score_delta,
                        mean_signed_score_delta
                     FROM tests.ec_hnsw_debug_pq_fastscan_scan_comparison_summary(
                        'ec_hnsw_scalar_runtime_summary_idx'::regclass::oid,
                        ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
                     )",
                    None,
                    &[],
                )
                .expect("scalar comparison summary query should succeed")
                .next()
                .expect("scalar comparison summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["pq_fastscan_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["missing_comparison_count"]
                    .value::<i32>()
                    .expect("missing comparison count should decode")
                    .expect("missing comparison count should be non-null"),
                row["mean_abs_score_delta"]
                    .value::<f64>()
                    .expect("mean abs score delta should decode")
                    .expect("mean abs score delta should be non-null"),
                row["max_abs_score_delta"]
                    .value::<f32>()
                    .expect("max abs score delta should decode")
                    .expect("max abs score delta should be non-null"),
                row["mean_signed_score_delta"]
                    .value::<f64>()
                    .expect("mean signed score delta should decode")
                    .expect("mean signed score delta should be non-null"),
            )
        });

        assert!(emitted_result_count > 0);
        assert_eq!(grouped_result_count, 0);
        assert_eq!(compared_result_count, 0);
        assert_eq!(missing_comparison_count, 0);
        assert_eq!(mean_abs_score_delta, 0.0);
        assert_eq!(max_abs_score_delta, 0.0);
        assert_eq!(mean_signed_score_delta, 0.0);
    }

    #[pg_test]
    fn test_pq_fastscan_runtime_comparison_rows_report_exact_ranks() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_comparison_rows (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 19 + dim) as f32) * 0.04).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.03).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_comparison_rows VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_comparison_rows_idx ON ec_hnsw_pq_fastscan_runtime_comparison_rows USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_runtime_comparison_rows_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.05_f32, 0.15, 0.25, 0.35, 0.45, 0.55, 0.65, 0.75, 0.85, 0.95, 1.05, 1.15, 1.25, 1.35,
            1.45, 1.55,
        ];
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query.clone())
        };
        let mut expected_exact_ranks = vec![None; observed.len()];
        let mut ordered_observed = observed
            .iter()
            .enumerate()
            .map(
                |(
                    idx,
                    ((block_number, offset_number), approx_score, comparison_score, approx_rank),
                )| {
                    (
                        idx,
                        *block_number,
                        *offset_number,
                        *approx_score,
                        *comparison_score,
                        approx_rank.unwrap_or_else(|| {
                            i32::try_from(idx + 1).expect("approx rank should fit in i32")
                        }),
                    )
                },
            )
            .collect::<Vec<_>>();
        ordered_observed.sort_by_key(|row| row.5);
        let mut compared_rows = ordered_observed
            .iter()
            .enumerate()
            .filter_map(
                |(
                    idx,
                    (
                        _live_idx,
                        _block_number,
                        _offset_number,
                        _approx_score,
                        comparison_score,
                        _approx_rank,
                    ),
                )| { comparison_score.map(|exact_score| (idx, exact_score)) },
            )
            .collect::<Vec<_>>();
        compared_rows.sort_by(|(left_idx, left_score), (right_idx, right_score)| {
            let left_approx_rank = ordered_observed[*left_idx].5;
            let right_approx_rank = ordered_observed[*right_idx].5;
            left_score
                .total_cmp(right_score)
                .then_with(|| left_approx_rank.cmp(&right_approx_rank))
        });
        for (rank, (idx, _exact_score)) in compared_rows.into_iter().enumerate() {
            expected_exact_ranks[idx] =
                Some(i32::try_from(rank + 1).expect("exact rank should fit in i32"));
        }
        let expected_rows = ordered_observed
            .iter()
            .enumerate()
            .map(
                |(
                    idx,
                    (
                        _live_idx,
                        block_number,
                        offset_number,
                        approx_score,
                        comparison_score,
                        approx_rank,
                    ),
                )| {
                    let exact_rank = expected_exact_ranks[idx];
                    let exact_rank_shift = exact_rank.map(|rank| approx_rank - rank);
                    (
                        i64::from(*block_number),
                        i32::from(*offset_number),
                        *approx_rank,
                        *approx_score,
                        *comparison_score,
                        exact_rank,
                        exact_rank_shift,
                    )
                },
            )
            .collect::<Vec<_>>();

        let query_literal = format_recall_vector_sql_literal(&query);
        let actual_rows = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT
                            block_number,
                            offset_number,
                            approx_rank,
                            approx_score,
                            comparison_score,
                            exact_rank,
                            exact_rank_shift
                         FROM tests.ec_hnsw_debug_pq_fastscan_scan_comparison_rows(
                            'ec_hnsw_pq_fastscan_runtime_comparison_rows_idx'::regclass::oid,
                            {query_literal}
                         )
                         ORDER BY approx_rank"
                    ),
                    None,
                    &[],
                )
                .expect("grouped comparison rows query should succeed")
                .map(|row| {
                    (
                        row["block_number"]
                            .value::<i64>()
                            .expect("block number should decode")
                            .expect("block number should be non-null"),
                        row["offset_number"]
                            .value::<i32>()
                            .expect("offset number should decode")
                            .expect("offset number should be non-null"),
                        row["approx_rank"]
                            .value::<i32>()
                            .expect("approx rank should decode")
                            .expect("approx rank should be non-null"),
                        row["approx_score"]
                            .value::<f32>()
                            .expect("approx score should decode")
                            .expect("approx score should be non-null"),
                        row["comparison_score"]
                            .value::<f32>()
                            .expect("comparison score should decode"),
                        row["exact_rank"]
                            .value::<i32>()
                            .expect("exact rank should decode"),
                        row["exact_rank_shift"]
                            .value::<i32>()
                            .expect("exact rank shift should decode"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(actual_rows.len(), expected_rows.len());
        for (actual, expected) in actual_rows.iter().zip(expected_rows.iter()) {
            assert_eq!(actual.0, expected.0);
            assert_eq!(actual.1, expected.1);
            assert_eq!(actual.2, expected.2);
            assert_eq!(actual.3.to_bits(), expected.3.to_bits());
            assert_eq!(
                actual.4.map(f32::to_bits),
                expected.4.map(f32::to_bits),
                "comparison score should preserve the emitted exact rerank score"
            );
            assert_eq!(actual.5, expected.5);
            assert_eq!(actual.6, expected.6);
        }
    }

    #[pg_test]
    fn test_scalar_runtime_comparison_rows_leave_exact_order_null() {
        Spi::run(
            "CREATE TABLE ec_hnsw_scalar_runtime_comparison_rows (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        Spi::run(
            "INSERT INTO ec_hnsw_scalar_runtime_comparison_rows VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.5, 0.5, 0.25, -0.75], 4, 42)),
             (4, encode_to_ecvector(ARRAY[-0.25, 0.9, 0.1, -0.4], 4, 42))",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX ec_hnsw_scalar_runtime_comparison_rows_idx ON ec_hnsw_scalar_runtime_comparison_rows USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let rows = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        approx_rank,
                        comparison_score,
                        exact_rank,
                        exact_rank_shift
                     FROM tests.ec_hnsw_debug_pq_fastscan_scan_comparison_rows(
                        'ec_hnsw_scalar_runtime_comparison_rows_idx'::regclass::oid,
                        ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
                     )
                     ORDER BY approx_rank",
                    None,
                    &[],
                )
                .expect("scalar comparison rows query should succeed")
                .map(|row| {
                    (
                        row["approx_rank"]
                            .value::<i32>()
                            .expect("approx rank should decode")
                            .expect("approx rank should be non-null"),
                        row["comparison_score"]
                            .value::<f32>()
                            .expect("comparison score should decode"),
                        row["exact_rank"]
                            .value::<i32>()
                            .expect("exact rank should decode"),
                        row["exact_rank_shift"]
                            .value::<i32>()
                            .expect("exact rank shift should decode"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert!(!rows.is_empty());
        for (idx, (approx_rank, comparison_score, exact_rank, exact_rank_shift)) in
            rows.iter().enumerate()
        {
            assert_eq!(
                *approx_rank,
                i32::try_from(idx + 1).expect("approx rank should fit in i32")
            );
            assert_eq!(*comparison_score, None);
            assert_eq!(*exact_rank, None);
            assert_eq!(*exact_rank_shift, None);
        }
    }

    #[pg_test]
    fn test_pq_fastscan_order_drift_summary_matches_rows() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_order_drift_summary (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 31 + dim) as f32) * 0.025).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 17 + dim) as f32) * 0.035).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_order_drift_summary VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_order_drift_summary_idx ON ec_hnsw_pq_fastscan_runtime_order_drift_summary USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_runtime_order_drift_summary_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.12_f32, 0.22, 0.32, 0.42, 0.52, 0.62, 0.72, 0.82, 0.92, 1.02, 1.12, 1.22, 1.32, 1.42,
            1.52, 1.62,
        ];
        let observed = unsafe { am::debug_grouped_scan_comparison_rows(index_oid, query.clone()) };
        let expected_emitted_result_count =
            i32::try_from(observed.len()).expect("emitted result count should fit in i32");
        let expected_grouped_result_count = expected_emitted_result_count;
        let mut expected_compared_result_count = 0_i32;
        let mut abs_rank_shift_sum = 0.0_f64;
        let mut expected_max_abs_rank_shift = 0_i32;
        let mut d_squared_sum = 0.0_f64;
        let mut expected_exact_best_approx_rank = None;
        let mut expected_exact_top4_max_approx_rank = None;

        for (
            _heap_tid,
            approx_rank,
            _approx_score,
            _comparison_score,
            exact_rank,
            exact_rank_shift,
        ) in &observed
        {
            let Some(exact_rank) = exact_rank else {
                continue;
            };
            expected_compared_result_count += 1;
            let rank_shift =
                exact_rank_shift.expect("grouped comparison rows should populate exact rank shift");
            let abs_rank_shift = rank_shift.abs();
            abs_rank_shift_sum += f64::from(abs_rank_shift);
            expected_max_abs_rank_shift = expected_max_abs_rank_shift.max(abs_rank_shift);
            let d = f64::from(*approx_rank - *exact_rank);
            d_squared_sum += d * d;
            if *exact_rank == 1 {
                expected_exact_best_approx_rank = Some(*approx_rank);
            }
            if *exact_rank <= 4 {
                expected_exact_top4_max_approx_rank = Some(
                    expected_exact_top4_max_approx_rank
                        .map_or(*approx_rank, |max_rank: i32| max_rank.max(*approx_rank)),
                );
            }
        }

        let expected_mean_abs_rank_shift = if expected_compared_result_count == 0 {
            0.0
        } else {
            abs_rank_shift_sum / f64::from(expected_compared_result_count)
        };
        let expected_spearman_rank_correlation = if expected_compared_result_count < 2 {
            0.0
        } else {
            let n = f64::from(expected_compared_result_count);
            1.0 - (6.0 * d_squared_sum / (n * (n * n - 1.0)))
        };
        let expected_window_1_contains_exact_best =
            expected_exact_best_approx_rank.is_some_and(|rank| rank <= 1);
        let expected_window_2_contains_exact_best =
            expected_exact_best_approx_rank.is_some_and(|rank| rank <= 2);
        let expected_window_4_contains_exact_best =
            expected_exact_best_approx_rank.is_some_and(|rank| rank <= 4);
        let expected_window_8_contains_exact_best =
            expected_exact_best_approx_rank.is_some_and(|rank| rank <= 8);

        let query_literal = format_recall_vector_sql_literal(&query);
        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            mean_abs_rank_shift,
            max_abs_rank_shift,
            spearman_rank_correlation,
            exact_best_approx_rank,
            exact_top4_max_approx_rank,
            window_1_contains_exact_best,
            window_2_contains_exact_best,
            window_4_contains_exact_best,
            window_8_contains_exact_best,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    &format!(
                        "SELECT
                            emitted_result_count,
                            pq_fastscan_result_count,
                            compared_result_count,
                            mean_abs_rank_shift,
                            max_abs_rank_shift,
                            spearman_rank_correlation,
                            exact_best_approx_rank,
                            exact_top4_max_approx_rank,
                            window_1_contains_exact_best,
                            window_2_contains_exact_best,
                            window_4_contains_exact_best,
                            window_8_contains_exact_best
                         FROM tests.ec_hnsw_debug_pq_fastscan_scan_order_drift_summary(
                            'ec_hnsw_pq_fastscan_runtime_order_drift_summary_idx'::regclass::oid,
                            {query_literal}
                         )"
                    ),
                    None,
                    &[],
                )
                .expect("grouped order drift summary query should succeed")
                .next()
                .expect("grouped order drift summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["pq_fastscan_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["mean_abs_rank_shift"]
                    .value::<f64>()
                    .expect("mean abs rank shift should decode")
                    .expect("mean abs rank shift should be non-null"),
                row["max_abs_rank_shift"]
                    .value::<i32>()
                    .expect("max abs rank shift should decode")
                    .expect("max abs rank shift should be non-null"),
                row["spearman_rank_correlation"]
                    .value::<f64>()
                    .expect("spearman rank correlation should decode")
                    .expect("spearman rank correlation should be non-null"),
                row["exact_best_approx_rank"]
                    .value::<i32>()
                    .expect("exact best approx rank should decode"),
                row["exact_top4_max_approx_rank"]
                    .value::<i32>()
                    .expect("exact top4 max approx rank should decode"),
                row["window_1_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 1 flag should decode")
                    .expect("window 1 flag should be non-null"),
                row["window_2_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 2 flag should decode")
                    .expect("window 2 flag should be non-null"),
                row["window_4_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 4 flag should decode")
                    .expect("window 4 flag should be non-null"),
                row["window_8_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 8 flag should decode")
                    .expect("window 8 flag should be non-null"),
            )
        });

        assert_eq!(emitted_result_count, expected_emitted_result_count);
        assert_eq!(grouped_result_count, expected_grouped_result_count);
        assert_eq!(compared_result_count, expected_compared_result_count);
        assert!(
            (mean_abs_rank_shift - expected_mean_abs_rank_shift).abs() <= 1e-6,
            "mean abs rank shift should match the emitted-row order summary"
        );
        assert_eq!(max_abs_rank_shift, expected_max_abs_rank_shift);
        assert!(
            (spearman_rank_correlation - expected_spearman_rank_correlation).abs() <= 1e-6,
            "spearman rank correlation should match the emitted-row order summary"
        );
        assert_eq!(exact_best_approx_rank, expected_exact_best_approx_rank);
        assert_eq!(
            exact_top4_max_approx_rank,
            expected_exact_top4_max_approx_rank
        );
        assert_eq!(
            window_1_contains_exact_best,
            expected_window_1_contains_exact_best
        );
        assert_eq!(
            window_2_contains_exact_best,
            expected_window_2_contains_exact_best
        );
        assert_eq!(
            window_4_contains_exact_best,
            expected_window_4_contains_exact_best
        );
        assert_eq!(
            window_8_contains_exact_best,
            expected_window_8_contains_exact_best
        );
    }

    #[pg_test]
    fn test_scalar_order_drift_summary_is_inert() {
        Spi::run(
            "CREATE TABLE ec_hnsw_scalar_runtime_order_drift_summary (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        Spi::run(
            "INSERT INTO ec_hnsw_scalar_runtime_order_drift_summary VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.5, 0.5, 0.25, -0.75], 4, 42)),
             (4, encode_to_ecvector(ARRAY[-0.25, 0.9, 0.1, -0.4], 4, 42))",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX ec_hnsw_scalar_runtime_order_drift_summary_idx ON ec_hnsw_scalar_runtime_order_drift_summary USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            mean_abs_rank_shift,
            max_abs_rank_shift,
            spearman_rank_correlation,
            exact_best_approx_rank,
            exact_top4_max_approx_rank,
            window_1_contains_exact_best,
            window_2_contains_exact_best,
            window_4_contains_exact_best,
            window_8_contains_exact_best,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    "SELECT
                        emitted_result_count,
                        pq_fastscan_result_count,
                        compared_result_count,
                        mean_abs_rank_shift,
                        max_abs_rank_shift,
                        spearman_rank_correlation,
                        exact_best_approx_rank,
                        exact_top4_max_approx_rank,
                        window_1_contains_exact_best,
                        window_2_contains_exact_best,
                        window_4_contains_exact_best,
                        window_8_contains_exact_best
                     FROM tests.ec_hnsw_debug_pq_fastscan_scan_order_drift_summary(
                        'ec_hnsw_scalar_runtime_order_drift_summary_idx'::regclass::oid,
                        ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
                     )",
                    None,
                    &[],
                )
                .expect("scalar order drift summary query should succeed")
                .next()
                .expect("scalar order drift summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["pq_fastscan_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["mean_abs_rank_shift"]
                    .value::<f64>()
                    .expect("mean abs rank shift should decode")
                    .expect("mean abs rank shift should be non-null"),
                row["max_abs_rank_shift"]
                    .value::<i32>()
                    .expect("max abs rank shift should decode")
                    .expect("max abs rank shift should be non-null"),
                row["spearman_rank_correlation"]
                    .value::<f64>()
                    .expect("spearman rank correlation should decode")
                    .expect("spearman rank correlation should be non-null"),
                row["exact_best_approx_rank"]
                    .value::<i32>()
                    .expect("exact best approx rank should decode"),
                row["exact_top4_max_approx_rank"]
                    .value::<i32>()
                    .expect("exact top4 max approx rank should decode"),
                row["window_1_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 1 flag should decode")
                    .expect("window 1 flag should be non-null"),
                row["window_2_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 2 flag should decode")
                    .expect("window 2 flag should be non-null"),
                row["window_4_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 4 flag should decode")
                    .expect("window 4 flag should be non-null"),
                row["window_8_contains_exact_best"]
                    .value::<bool>()
                    .expect("window 8 flag should decode")
                    .expect("window 8 flag should be non-null"),
            )
        });

        assert!(emitted_result_count > 0);
        assert_eq!(grouped_result_count, 0);
        assert_eq!(compared_result_count, 0);
        assert_eq!(mean_abs_rank_shift, 0.0);
        assert_eq!(max_abs_rank_shift, 0);
        assert_eq!(spearman_rank_correlation, 0.0);
        assert_eq!(exact_best_approx_rank, None);
        assert_eq!(exact_top4_max_approx_rank, None);
        assert!(!window_1_contains_exact_best);
        assert!(!window_2_contains_exact_best);
        assert!(!window_4_contains_exact_best);
        assert!(!window_8_contains_exact_best);
    }

    #[pg_test]
    fn test_pq_fastscan_windowed_rows_match_simulation() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_windowed_rows (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 41 + dim) as f32) * 0.02).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 13 + dim) as f32) * 0.04).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_windowed_rows VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_windowed_rows_idx ON ec_hnsw_pq_fastscan_runtime_windowed_rows USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_runtime_windowed_rows_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.14_f32, 0.24, 0.34, 0.44, 0.54, 0.64, 0.74, 0.84, 0.94, 1.04, 1.14, 1.24, 1.34, 1.44,
            1.54, 1.64,
        ];
        let baseline_rows =
            unsafe { am::debug_grouped_scan_comparison_rows(index_oid, query.clone()) };
        let window_size = 4_usize;
        let mut buffered_rows = Vec::with_capacity(window_size);
        let mut next_idx = 0usize;
        let mut expected_rows = Vec::with_capacity(baseline_rows.len());
        while expected_rows.len() < baseline_rows.len() {
            while buffered_rows.len() < window_size && next_idx < baseline_rows.len() {
                buffered_rows.push(baseline_rows[next_idx]);
                next_idx += 1;
            }
            let (selected_idx, _) = buffered_rows
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    left.3
                        .unwrap_or(left.2)
                        .total_cmp(&right.3.unwrap_or(right.2))
                        .then_with(|| left.1.cmp(&right.1))
                })
                .expect("windowed grouped simulation should always have a buffered row");
            let (
                (block_number, offset_number),
                approx_rank,
                approx_score,
                comparison_score,
                exact_rank,
                exact_rank_shift,
            ) = buffered_rows.remove(selected_idx);
            let windowed_rank =
                i32::try_from(expected_rows.len() + 1).expect("windowed rank should fit in i32");
            let windowed_rank_shift = exact_rank.map(|rank| windowed_rank - rank);
            expected_rows.push((
                i64::from(block_number),
                i32::from(offset_number),
                approx_rank,
                windowed_rank,
                approx_score,
                comparison_score,
                exact_rank,
                exact_rank_shift,
                windowed_rank_shift,
            ));
        }

        let query_literal = format_recall_vector_sql_literal(&query);
        let actual_rows = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT
                            block_number,
                            offset_number,
                            approx_rank,
                            windowed_rank,
                            approx_score,
                            comparison_score,
                            exact_rank,
                            exact_rank_shift,
                            windowed_rank_shift
                         FROM tests.ec_hnsw_debug_pq_fastscan_scan_windowed_rows(
                            'ec_hnsw_pq_fastscan_runtime_windowed_rows_idx'::regclass::oid,
                            {query_literal},
                            4
                         )
                         ORDER BY windowed_rank"
                    ),
                    None,
                    &[],
                )
                .expect("grouped windowed rows query should succeed")
                .map(|row| {
                    (
                        row["block_number"]
                            .value::<i64>()
                            .expect("block number should decode")
                            .expect("block number should be non-null"),
                        row["offset_number"]
                            .value::<i32>()
                            .expect("offset number should decode")
                            .expect("offset number should be non-null"),
                        row["approx_rank"]
                            .value::<i32>()
                            .expect("approx rank should decode")
                            .expect("approx rank should be non-null"),
                        row["windowed_rank"]
                            .value::<i32>()
                            .expect("windowed rank should decode")
                            .expect("windowed rank should be non-null"),
                        row["approx_score"]
                            .value::<f32>()
                            .expect("approx score should decode")
                            .expect("approx score should be non-null"),
                        row["comparison_score"]
                            .value::<f32>()
                            .expect("comparison score should decode"),
                        row["exact_rank"]
                            .value::<i32>()
                            .expect("exact rank should decode"),
                        row["exact_rank_shift"]
                            .value::<i32>()
                            .expect("exact rank shift should decode"),
                        row["windowed_rank_shift"]
                            .value::<i32>()
                            .expect("windowed rank shift should decode"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(actual_rows.len(), expected_rows.len());
        for (actual, expected) in actual_rows.iter().zip(expected_rows.iter()) {
            assert_eq!(actual.0, expected.0);
            assert_eq!(actual.1, expected.1);
            assert_eq!(actual.2, expected.2);
            assert_eq!(actual.3, expected.3);
            assert_eq!(actual.4.to_bits(), expected.4.to_bits());
            assert_eq!(actual.5.map(f32::to_bits), expected.5.map(f32::to_bits));
            assert_eq!(actual.6, expected.6);
            assert_eq!(actual.7, expected.7);
            assert_eq!(actual.8, expected.8);
        }
    }

    fn assert_pq_fastscan_runtime_live_window_matches_windowed_simulation(
        window_size: i32,
        configure_window_env: bool,
        require_movement: bool,
    ) {
        let _lock = env_var_test_lock();
        let window_value = window_size.to_string();
        let _window_guard = configure_window_env
            .then(|| ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW", &window_value));

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_live_window (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=32 {
            let source = (0..16)
                .map(|dim| {
                    format!(
                        "{:.6}",
                        (((id * 43 + dim * 7) as f32) * 0.019).cos()
                            + (((id * 17 + dim * 5) as f32) * 0.011).sin()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| {
                    format!(
                        "{:.6}",
                        (((id * 29 + dim * 11) as f32) * 0.023).sin()
                            + (((id * 13 + dim * 3) as f32) * 0.017).cos()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_live_window VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_live_window_idx ON ec_hnsw_pq_fastscan_runtime_live_window USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_runtime_live_window_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let candidate_queries = (0..24)
            .map(|seed| {
                (0..16)
                    .map(|dim| {
                        (((seed * 31 + dim * 7) as f32) * 0.021).sin()
                            + (((seed * 19 + dim * 5) as f32) * 0.014).cos()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let (query, live_rows, baseline_rows) = candidate_queries
            .into_iter()
            .find_map(|query| {
                let live_rows = unsafe {
                    am::debug_gettuple_scan_heap_tids_with_score_comparisons(
                        index_oid,
                        query.clone(),
                    )
                };
                let baseline_rows =
                    unsafe { am::debug_grouped_scan_comparison_rows(index_oid, query.clone()) };
                let actual_live_order = live_rows
                    .iter()
                    .map(|(heap_tid, _approx_score, _comparison_score, _approx_rank)| *heap_tid)
                    .collect::<Vec<_>>();
                let baseline_approx_order = baseline_rows
                    .iter()
                    .map(
                        |(
                            heap_tid,
                            _approx_rank,
                            _approx_score,
                            _comparison_score,
                            _exact_rank,
                            _exact_rank_shift,
                        )| *heap_tid,
                    )
                    .collect::<Vec<_>>();
                (!require_movement || actual_live_order != baseline_approx_order).then_some((
                    query,
                    live_rows,
                    baseline_rows,
                ))
            })
            .expect(if require_movement {
                "at least one deterministic grouped query should exhibit live window movement"
            } else {
                "the deterministic grouped query set should provide at least one candidate"
            });

        let expected_live_order = simulate_grouped_live_window_order(
            &baseline_rows,
            usize::try_from(window_size).expect("window size should fit in usize"),
        );
        let actual_live_order = live_rows
            .iter()
            .map(|(heap_tid, _approx_score, _comparison_score, _approx_rank)| *heap_tid)
            .collect::<Vec<_>>();
        assert_eq!(
            actual_live_order
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>(),
            expected_live_order
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>(),
            "grouped live runtime should emit the same heap tid set as the window-size-{window_size} simulation",
        );

        let baseline_approx_order = baseline_rows
            .iter()
            .map(
                |(
                    heap_tid,
                    _approx_rank,
                    _approx_score,
                    _comparison_score,
                    _exact_rank,
                    _exact_rank_shift,
                )| *heap_tid,
            )
            .collect::<Vec<_>>();
        if require_movement {
            assert_ne!(
                actual_live_order, baseline_approx_order,
                "the selected query should prove the live grouped rerank window changes output order"
            );
        }

        let mut live_rows_sorted_by_approx_rank = live_rows.clone();
        live_rows_sorted_by_approx_rank.sort_by_key(
            |(_heap_tid, _approx_score, _comparison_score, approx_rank)| {
                approx_rank.expect(
                    "grouped live results should preserve baseline approximate rank sidecars",
                )
            },
        );
        let preserved_approx_order = live_rows_sorted_by_approx_rank
            .iter()
            .map(|(heap_tid, _approx_score, _comparison_score, _approx_rank)| *heap_tid)
            .collect::<Vec<_>>();
        assert_eq!(
            preserved_approx_order, baseline_approx_order,
            "grouped comparison rows should still expose baseline approximate order after live rerank cutover"
        );

        let _query_literal = format_recall_vector_sql_literal(&query);
    }

    #[pg_test]
    fn test_pq_fastscan_live_window_matches_simulation() {
        assert_pq_fastscan_runtime_live_window_matches_windowed_simulation(4, false, true);
    }

    #[pg_test]
    fn test_pq_fastscan_runtime_live_window_respects_window_env() {
        assert_pq_fastscan_runtime_live_window_matches_windowed_simulation(8, true, false);
    }

    #[pg_test]
    fn test_pq_fastscan_runtime_live_window_supports_higher_window_env() {
        assert_pq_fastscan_runtime_live_window_matches_windowed_simulation(32, true, false);
    }

    #[pg_test]
    fn test_scalar_windowed_rows_are_inert() {
        Spi::run(
            "CREATE TABLE ec_hnsw_scalar_runtime_windowed_rows (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        Spi::run(
            "INSERT INTO ec_hnsw_scalar_runtime_windowed_rows VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.5, 0.5, 0.25, -0.75], 4, 42)),
             (4, encode_to_ecvector(ARRAY[-0.25, 0.9, 0.1, -0.4], 4, 42))",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX ec_hnsw_scalar_runtime_windowed_rows_idx ON ec_hnsw_scalar_runtime_windowed_rows USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let rows = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        approx_rank,
                        windowed_rank,
                        comparison_score,
                        exact_rank,
                        exact_rank_shift,
                        windowed_rank_shift
                     FROM tests.ec_hnsw_debug_pq_fastscan_scan_windowed_rows(
                        'ec_hnsw_scalar_runtime_windowed_rows_idx'::regclass::oid,
                        ARRAY[1.0, 0.0, 0.5, -1.0]::real[],
                        4
                     )
                     ORDER BY windowed_rank",
                    None,
                    &[],
                )
                .expect("scalar windowed rows query should succeed")
                .map(|row| {
                    (
                        row["approx_rank"]
                            .value::<i32>()
                            .expect("approx rank should decode")
                            .expect("approx rank should be non-null"),
                        row["windowed_rank"]
                            .value::<i32>()
                            .expect("windowed rank should decode")
                            .expect("windowed rank should be non-null"),
                        row["comparison_score"]
                            .value::<f32>()
                            .expect("comparison score should decode"),
                        row["exact_rank"]
                            .value::<i32>()
                            .expect("exact rank should decode"),
                        row["exact_rank_shift"]
                            .value::<i32>()
                            .expect("exact rank shift should decode"),
                        row["windowed_rank_shift"]
                            .value::<i32>()
                            .expect("windowed rank shift should decode"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert!(!rows.is_empty());
        for (
            idx,
            (
                approx_rank,
                windowed_rank,
                comparison_score,
                exact_rank,
                exact_rank_shift,
                windowed_rank_shift,
            ),
        ) in rows.iter().enumerate()
        {
            let expected_rank = i32::try_from(idx + 1).expect("rank should fit in i32");
            assert_eq!(*approx_rank, expected_rank);
            assert_eq!(*windowed_rank, expected_rank);
            assert_eq!(*comparison_score, None);
            assert_eq!(*exact_rank, None);
            assert_eq!(*exact_rank_shift, None);
            assert_eq!(*windowed_rank_shift, None);
        }
    }

    #[pg_test]
    fn test_pq_fastscan_windowed_summary_matches_rows() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_pq_fastscan_runtime_windowed_summary (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 23 + dim) as f32) * 0.035).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 31 + dim) as f32) * 0.025).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_pq_fastscan_runtime_windowed_summary VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_pq_fastscan_runtime_windowed_summary_idx ON ec_hnsw_pq_fastscan_runtime_windowed_summary USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_pq_fastscan_runtime_windowed_summary_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![
            0.16_f32, 0.26, 0.36, 0.46, 0.56, 0.66, 0.76, 0.86, 0.96, 1.06, 1.16, 1.26, 1.36, 1.46,
            1.56, 1.66,
        ];
        let baseline_rows =
            unsafe { am::debug_grouped_scan_comparison_rows(index_oid, query.clone()) };
        let windowed_rows =
            unsafe { am::debug_grouped_scan_windowed_rows(index_oid, query.clone(), 4) };

        let rank_metrics = |rows: &[(i32, Option<i32>, Option<i32>)]| {
            let mut compared_result_count = 0_i32;
            let mut abs_rank_shift_sum = 0.0_f64;
            let mut max_abs_rank_shift = 0_i32;
            let mut d_squared_sum = 0.0_f64;
            let mut exact_best_rank = None;
            let mut exact_top4_max_rank = None;

            for (observed_rank, exact_rank, explicit_rank_shift) in rows {
                let Some(exact_rank) = exact_rank else {
                    continue;
                };
                compared_result_count += 1;
                let abs_rank_shift = explicit_rank_shift
                    .unwrap_or(observed_rank - exact_rank)
                    .abs();
                abs_rank_shift_sum += f64::from(abs_rank_shift);
                max_abs_rank_shift = max_abs_rank_shift.max(abs_rank_shift);
                let d = f64::from(observed_rank - exact_rank);
                d_squared_sum += d * d;
                if *exact_rank == 1 {
                    exact_best_rank = Some(*observed_rank);
                }
                if *exact_rank <= 4 {
                    exact_top4_max_rank = Some(
                        exact_top4_max_rank
                            .map_or(*observed_rank, |max_rank: i32| max_rank.max(*observed_rank)),
                    );
                }
            }

            let mean_abs_rank_shift = if compared_result_count == 0 {
                0.0
            } else {
                abs_rank_shift_sum / f64::from(compared_result_count)
            };
            let spearman_rank_correlation = if compared_result_count < 2 {
                0.0
            } else {
                let n = f64::from(compared_result_count);
                1.0 - (6.0 * d_squared_sum / (n * (n * n - 1.0)))
            };

            (
                compared_result_count,
                mean_abs_rank_shift,
                max_abs_rank_shift,
                spearman_rank_correlation,
                exact_best_rank,
                exact_top4_max_rank,
            )
        };

        let expected_emitted_result_count =
            i32::try_from(baseline_rows.len()).expect("emitted result count should fit in i32");
        let expected_grouped_result_count = expected_emitted_result_count;
        let baseline_metric_rows = baseline_rows
            .iter()
            .map(
                |(
                    _heap_tid,
                    approx_rank,
                    _approx_score,
                    _comparison_score,
                    exact_rank,
                    exact_rank_shift,
                )| { (*approx_rank, *exact_rank, *exact_rank_shift) },
            )
            .collect::<Vec<_>>();
        let windowed_metric_rows = windowed_rows
            .iter()
            .map(
                |(
                    _heap_tid,
                    _approx_rank,
                    windowed_rank,
                    _approx_score,
                    _comparison_score,
                    exact_rank,
                    _exact_rank_shift,
                    windowed_rank_shift,
                )| (*windowed_rank, *exact_rank, *windowed_rank_shift),
            )
            .collect::<Vec<_>>();
        let (
            expected_compared_result_count,
            expected_mean_abs_rank_shift_before,
            expected_max_abs_rank_shift_before,
            expected_spearman_before,
            expected_exact_best_approx_rank,
            expected_exact_top4_max_approx_rank,
        ) = rank_metrics(&baseline_metric_rows);
        let (
            _windowed_compared_result_count,
            expected_mean_abs_rank_shift_after,
            expected_max_abs_rank_shift_after,
            expected_spearman_after,
            expected_exact_best_windowed_rank,
            expected_exact_top4_max_windowed_rank,
        ) = rank_metrics(&windowed_metric_rows);

        let query_literal = format_recall_vector_sql_literal(&query);
        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            window_size,
            exact_best_approx_rank,
            exact_best_windowed_rank,
            exact_top4_max_approx_rank,
            exact_top4_max_windowed_rank,
            mean_abs_rank_shift_before,
            mean_abs_rank_shift_after,
            max_abs_rank_shift_before,
            max_abs_rank_shift_after,
            spearman_before,
            spearman_after,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    &format!(
                        "SELECT
                            emitted_result_count,
                            pq_fastscan_result_count,
                            compared_result_count,
                            window_size,
                            exact_best_approx_rank,
                            exact_best_windowed_rank,
                            exact_top4_max_approx_rank,
                            exact_top4_max_windowed_rank,
                            mean_abs_rank_shift_before,
                            mean_abs_rank_shift_after,
                            max_abs_rank_shift_before,
                            max_abs_rank_shift_after,
                            spearman_rank_correlation_before,
                            spearman_rank_correlation_after
                         FROM tests.ec_hnsw_debug_pq_fastscan_scan_windowed_summary(
                            'ec_hnsw_pq_fastscan_runtime_windowed_summary_idx'::regclass::oid,
                            {query_literal},
                            4
                         )"
                    ),
                    None,
                    &[],
                )
                .expect("grouped windowed summary query should succeed")
                .next()
                .expect("grouped windowed summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["pq_fastscan_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["window_size"]
                    .value::<i32>()
                    .expect("window size should decode")
                    .expect("window size should be non-null"),
                row["exact_best_approx_rank"]
                    .value::<i32>()
                    .expect("exact best approx rank should decode"),
                row["exact_best_windowed_rank"]
                    .value::<i32>()
                    .expect("exact best windowed rank should decode"),
                row["exact_top4_max_approx_rank"]
                    .value::<i32>()
                    .expect("exact top4 max approx rank should decode"),
                row["exact_top4_max_windowed_rank"]
                    .value::<i32>()
                    .expect("exact top4 max windowed rank should decode"),
                row["mean_abs_rank_shift_before"]
                    .value::<f64>()
                    .expect("mean abs rank shift before should decode")
                    .expect("mean abs rank shift before should be non-null"),
                row["mean_abs_rank_shift_after"]
                    .value::<f64>()
                    .expect("mean abs rank shift after should decode")
                    .expect("mean abs rank shift after should be non-null"),
                row["max_abs_rank_shift_before"]
                    .value::<i32>()
                    .expect("max abs rank shift before should decode")
                    .expect("max abs rank shift before should be non-null"),
                row["max_abs_rank_shift_after"]
                    .value::<i32>()
                    .expect("max abs rank shift after should decode")
                    .expect("max abs rank shift after should be non-null"),
                row["spearman_rank_correlation_before"]
                    .value::<f64>()
                    .expect("spearman rank correlation before should decode")
                    .expect("spearman rank correlation before should be non-null"),
                row["spearman_rank_correlation_after"]
                    .value::<f64>()
                    .expect("spearman rank correlation after should decode")
                    .expect("spearman rank correlation after should be non-null"),
            )
        });

        assert_eq!(emitted_result_count, expected_emitted_result_count);
        assert_eq!(grouped_result_count, expected_grouped_result_count);
        assert_eq!(compared_result_count, expected_compared_result_count);
        assert_eq!(window_size, 4);
        assert_eq!(exact_best_approx_rank, expected_exact_best_approx_rank);
        assert_eq!(exact_best_windowed_rank, expected_exact_best_windowed_rank);
        assert_eq!(
            exact_top4_max_approx_rank,
            expected_exact_top4_max_approx_rank
        );
        assert_eq!(
            exact_top4_max_windowed_rank,
            expected_exact_top4_max_windowed_rank
        );
        assert!(
            (mean_abs_rank_shift_before - expected_mean_abs_rank_shift_before).abs() <= 1e-6,
            "baseline mean abs rank shift should match the row aggregation"
        );
        assert!(
            (mean_abs_rank_shift_after - expected_mean_abs_rank_shift_after).abs() <= 1e-6,
            "windowed mean abs rank shift should match the row aggregation"
        );
        assert_eq!(
            max_abs_rank_shift_before,
            expected_max_abs_rank_shift_before
        );
        assert_eq!(max_abs_rank_shift_after, expected_max_abs_rank_shift_after);
        assert!(
            (spearman_before - expected_spearman_before).abs() <= 1e-6,
            "baseline spearman should match the row aggregation"
        );
        assert!(
            (spearman_after - expected_spearman_after).abs() <= 1e-6,
            "windowed spearman should match the row aggregation"
        );
        if let (Some(approx_rank), Some(windowed_rank)) =
            (exact_best_approx_rank, exact_best_windowed_rank)
        {
            assert!(
                windowed_rank <= approx_rank,
                "a sliding rerank window should not push the exact-best emitted row later than its baseline approximate rank"
            );
        }
    }

    #[pg_test]
    fn test_scalar_windowed_summary_is_inert() {
        Spi::run(
            "CREATE TABLE ec_hnsw_scalar_runtime_windowed_summary (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");

        Spi::run(
            "INSERT INTO ec_hnsw_scalar_runtime_windowed_summary VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.5, 0.5, 0.25, -0.75], 4, 42)),
             (4, encode_to_ecvector(ARRAY[-0.25, 0.9, 0.1, -0.4], 4, 42))",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX ec_hnsw_scalar_runtime_windowed_summary_idx ON ec_hnsw_scalar_runtime_windowed_summary USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            window_size,
            exact_best_approx_rank,
            exact_best_windowed_rank,
            exact_top4_max_approx_rank,
            exact_top4_max_windowed_rank,
            mean_abs_rank_shift_before,
            mean_abs_rank_shift_after,
            max_abs_rank_shift_before,
            max_abs_rank_shift_after,
            spearman_before,
            spearman_after,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    "SELECT
                        emitted_result_count,
                        pq_fastscan_result_count,
                        compared_result_count,
                        window_size,
                        exact_best_approx_rank,
                        exact_best_windowed_rank,
                        exact_top4_max_approx_rank,
                        exact_top4_max_windowed_rank,
                        mean_abs_rank_shift_before,
                        mean_abs_rank_shift_after,
                        max_abs_rank_shift_before,
                        max_abs_rank_shift_after,
                        spearman_rank_correlation_before,
                        spearman_rank_correlation_after
                     FROM tests.ec_hnsw_debug_pq_fastscan_scan_windowed_summary(
                        'ec_hnsw_scalar_runtime_windowed_summary_idx'::regclass::oid,
                        ARRAY[1.0, 0.0, 0.5, -1.0]::real[],
                        4
                     )",
                    None,
                    &[],
                )
                .expect("scalar windowed summary query should succeed")
                .next()
                .expect("scalar windowed summary should return one row");
            (
                row["emitted_result_count"]
                    .value::<i32>()
                    .expect("emitted result count should decode")
                    .expect("emitted result count should be non-null"),
                row["pq_fastscan_result_count"]
                    .value::<i32>()
                    .expect("grouped result count should decode")
                    .expect("grouped result count should be non-null"),
                row["compared_result_count"]
                    .value::<i32>()
                    .expect("compared result count should decode")
                    .expect("compared result count should be non-null"),
                row["window_size"]
                    .value::<i32>()
                    .expect("window size should decode")
                    .expect("window size should be non-null"),
                row["exact_best_approx_rank"]
                    .value::<i32>()
                    .expect("exact best approx rank should decode"),
                row["exact_best_windowed_rank"]
                    .value::<i32>()
                    .expect("exact best windowed rank should decode"),
                row["exact_top4_max_approx_rank"]
                    .value::<i32>()
                    .expect("exact top4 max approx rank should decode"),
                row["exact_top4_max_windowed_rank"]
                    .value::<i32>()
                    .expect("exact top4 max windowed rank should decode"),
                row["mean_abs_rank_shift_before"]
                    .value::<f64>()
                    .expect("mean abs rank shift before should decode")
                    .expect("mean abs rank shift before should be non-null"),
                row["mean_abs_rank_shift_after"]
                    .value::<f64>()
                    .expect("mean abs rank shift after should decode")
                    .expect("mean abs rank shift after should be non-null"),
                row["max_abs_rank_shift_before"]
                    .value::<i32>()
                    .expect("max abs rank shift before should decode")
                    .expect("max abs rank shift before should be non-null"),
                row["max_abs_rank_shift_after"]
                    .value::<i32>()
                    .expect("max abs rank shift after should decode")
                    .expect("max abs rank shift after should be non-null"),
                row["spearman_rank_correlation_before"]
                    .value::<f64>()
                    .expect("spearman rank correlation before should decode")
                    .expect("spearman rank correlation before should be non-null"),
                row["spearman_rank_correlation_after"]
                    .value::<f64>()
                    .expect("spearman rank correlation after should decode")
                    .expect("spearman rank correlation after should be non-null"),
            )
        });

        assert!(emitted_result_count > 0);
        assert_eq!(grouped_result_count, 0);
        assert_eq!(compared_result_count, 0);
        assert_eq!(window_size, 4);
        assert_eq!(exact_best_approx_rank, None);
        assert_eq!(exact_best_windowed_rank, None);
        assert_eq!(exact_top4_max_approx_rank, None);
        assert_eq!(exact_top4_max_windowed_rank, None);
        assert_eq!(mean_abs_rank_shift_before, 0.0);
        assert_eq!(mean_abs_rank_shift_after, 0.0);
        assert_eq!(max_abs_rank_shift_before, 0);
        assert_eq!(max_abs_rank_shift_after, 0);
        assert_eq!(spearman_before, 0.0);
        assert_eq!(spearman_after, 0.0);
    }

