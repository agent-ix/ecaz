    #[pg_test]
    fn test_ech_debug_scan_result_count_matches_scan_helper() {
        Spi::run(
            "CREATE TABLE ec_hnsw_debug_scan_result_count_fixture \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_debug_scan_result_count_fixture VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.95, 0.05, 0.45, -0.95], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0, -0.5, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_debug_scan_result_count_fixture_idx \
             ON ec_hnsw_debug_scan_result_count_fixture USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_debug_scan_result_count_fixture_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![1.0, 0.0, 0.5, -1.0];
        let rust_count = i32::try_from(unsafe {
            am::debug_gettuple_scan_heap_tids(index_oid, query.clone()).len()
        })
        .expect("scan result count should fit in i32");
        let sql_count = Spi::get_one::<i32>(
            "SELECT tests.ec_hnsw_debug_scan_result_count(
                 'ec_hnsw_debug_scan_result_count_fixture_idx'::regclass::oid,
                 ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
             )",
        )
        .expect("debug scan SQL wrapper should succeed")
        .expect("debug scan SQL wrapper should return a row");

        assert_eq!(
            sql_count, rust_count,
            "the SQL-visible debug scan wrapper should exercise the same live ec_hnsw scan path",
        );
    }

    #[pg_test]
    fn test_ech_debug_scan_profile_reports_graph_first_counters() {
        Spi::run(
            "CREATE TABLE ec_hnsw_debug_scan_profile_fixture \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_debug_scan_profile_fixture VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.95, 0.05, 0.45, -0.95], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0, -0.5, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_debug_scan_profile_fixture_idx \
             ON ec_hnsw_debug_scan_profile_fixture USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_debug_scan_profile_fixture_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            _rescan_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            rescan_phase,
            rescan_current_result,
            _rescan_ordered_slots,
            _rescan_pending_heap_tids,
            _rescan_visited_elements,
            _rescan_expanded_sources,
            _rescan_emitted_elements,
            _rescan_bootstrap_expansions,
            rescan_bootstrap_pages_read,
            _rescan_quantizer_cache_hit,
            result_count,
            final_phase,
            final_ordered_slots,
            _total_bootstrap_expansions,
            _total_bootstrap_pages_read,
            total_linear_pages_read,
            total_elements_scored,
            _total_elements_skipped,
            total_heap_tids_returned,
            _total_quantizer_cache_hit,
            total_emitted_elements,
            rescan_amrescan_total_elapsed_us,
            rescan_query_decode_elapsed_us,
            rescan_scan_setup_elapsed_us,
            rescan_store_query_elapsed_us,
            rescan_prepare_query_elapsed_us,
            rescan_reset_state_elapsed_us,
            rescan_initialize_entry_elapsed_us,
            rescan_upper_layer_seed_elapsed_us,
            rescan_layer0_seed_elapsed_us,
            rescan_stage_ordered_results_elapsed_us,
            rescan_initial_prefetch_elapsed_us,
            rescan_frontier_consume_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            graph_element_cache_hits,
            graph_element_cache_misses,
            graph_element_load_elapsed_us,
            graph_neighbor_cache_hits,
            graph_neighbor_cache_misses,
            graph_neighbor_load_elapsed_us,
            candidate_score_calls,
            candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ) = unsafe { am::debug_profile_ordered_scan(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert_eq!(
            rescan_phase, "graph_traversal",
            "the profile helper should report that ordered scans start in the graph-traversal phase",
        );
        assert!(
            rescan_current_result,
            "amrescan should prefetch the first ordered result into current-result state on a non-empty index",
        );
        assert!(
            rescan_bootstrap_pages_read > 0,
            "prefetching the first ordered result should read at least one graph page",
        );
        assert!(
            rescan_amrescan_total_elapsed_us >= 0
                && rescan_query_decode_elapsed_us >= 0
                && rescan_scan_setup_elapsed_us >= 0
                && rescan_store_query_elapsed_us >= 0
                && rescan_prepare_query_elapsed_us >= 0
                && rescan_reset_state_elapsed_us >= 0
                && rescan_initialize_entry_elapsed_us >= 0
                && rescan_upper_layer_seed_elapsed_us >= 0
                && rescan_layer0_seed_elapsed_us >= 0
                && rescan_stage_ordered_results_elapsed_us >= 0
                && rescan_initial_prefetch_elapsed_us >= 0
                && rescan_frontier_consume_elapsed_us >= 0
                && rescan_graph_result_materialize_elapsed_us >= 0,
            "the profile helper should surface non-negative rescan timing buckets",
        );
        assert!(
            graph_element_cache_misses > 0 && graph_neighbor_cache_misses > 0,
            "profiling should record graph cache misses on a non-empty fixture",
        );
        assert!(
            graph_element_load_elapsed_us >= 0 && graph_neighbor_load_elapsed_us >= 0,
            "profiling should surface non-negative graph load timing buckets",
        );
        assert!(
            graph_element_cache_hits >= 0 && graph_neighbor_cache_hits >= 0,
            "profiling should surface graph cache hit counters even when the fixture is tiny",
        );
        assert!(
            candidate_score_calls > 0 && candidate_score_elapsed_us >= 0,
            "profiling should record candidate scoring work during scan seeding",
        );
        assert!(
            score_cache_hits >= 0 && score_cache_misses > 0,
            "profiling should surface score-cache counters and at least one first-score miss on a non-empty fixture",
        );
        assert_eq!(
            (
                grouped_traversal_approx_score_calls,
                grouped_traversal_approx_score_elapsed_us,
                grouped_traversal_exact_score_calls,
                grouped_traversal_exact_score_elapsed_us,
                grouped_traversal_budgeted_expansions,
                grouped_traversal_budgeted_candidates,
                grouped_traversal_budgeted_exact_candidates,
            ),
            (0, 0, 0, 0, 0, 0, 0),
            "scalar fixtures should leave grouped traversal counters inert",
        );
        assert!(
            result_count > 0,
            "the profiled scan should return at least one heap TID on a non-empty fixture",
        );
        assert_eq!(
            final_phase, "exhausted",
            "a full profiled scan should end in the exhausted phase",
        );
        assert_eq!(
            final_ordered_slots, 0,
            "full scan exhaustion should leave no current result or frontier slots staged",
        );
        assert_eq!(
            total_linear_pages_read, 0,
            "the graph-first ordered runtime should not fall back to linear scanning on this fixture",
        );
        assert!(
            total_elements_scored >= total_emitted_elements,
            "scored elements should cover every emitted ordered element",
        );
        assert_eq!(
            total_heap_tids_returned, result_count,
            "heap-TID return count should match the helper's emitted row count",
        );
    }

    #[pg_test]
    fn test_ech_debug_scan_profile_limit_stops_early() {
        Spi::run(
            "CREATE TABLE ec_hnsw_debug_scan_profile_limited_fixture \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_debug_scan_profile_limited_fixture VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.95, 0.05, 0.45, -0.95], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.9, 0.1, 0.4, -0.9], 4, 42)),
             (4, encode_to_ecvector(ARRAY[-1.0, 0.0, -0.5, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_debug_scan_profile_limited_fixture_idx \
             ON ec_hnsw_debug_scan_profile_limited_fixture USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let full_result_count = Spi::get_one::<i32>(
            "SELECT tests.ec_hnsw_debug_scan_result_count(
                 'ec_hnsw_debug_scan_profile_limited_fixture_idx'::regclass::oid,
                 ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
             )",
        )
        .expect("full result-count query should succeed")
        .expect("full result-count query should return a row");
        let limited_result_count = Spi::get_one::<i32>(
            "SELECT result_count
             FROM tests.ec_hnsw_debug_scan_profile_limited(
                 'ec_hnsw_debug_scan_profile_limited_fixture_idx'::regclass::oid,
                 ARRAY[1.0, 0.0, 0.5, -1.0]::real[],
                 1
             )",
        )
        .expect("limited profile query should succeed")
        .expect("limited profile query should return a row");
        let limited_heap_tids_returned = Spi::get_one::<i32>(
            "SELECT total_heap_tids_returned
             FROM tests.ec_hnsw_debug_scan_profile_limited(
                 'ec_hnsw_debug_scan_profile_limited_fixture_idx'::regclass::oid,
                 ARRAY[1.0, 0.0, 0.5, -1.0]::real[],
                 1
             )",
        )
        .expect("limited heap-tid query should succeed")
        .expect("limited heap-tid query should return a row");
        let limited_final_phase = Spi::get_one::<String>(
            "SELECT final_phase
             FROM tests.ec_hnsw_debug_scan_profile_limited(
                 'ec_hnsw_debug_scan_profile_limited_fixture_idx'::regclass::oid,
                 ARRAY[1.0, 0.0, 0.5, -1.0]::real[],
                 1
             )",
        )
        .expect("limited final-phase query should succeed")
        .expect("limited final-phase query should return a row");

        assert!(
            full_result_count > 1,
            "fixture should expose more than one ordered result so the limit meaningfully truncates the scan",
        );
        assert_eq!(
            limited_result_count, 1,
            "limited scan profile should stop after the requested number of emitted results",
        );
        assert_eq!(
            limited_heap_tids_returned, 1,
            "limited scan profile should report only the emitted heap TIDs it actually returned",
        );
        assert_ne!(
            limited_final_phase, "exhausted",
            "stopping early should preserve a non-exhausted execution phase",
        );
    }

    #[pg_test]
    fn test_ech_debug_scan_heap_fetch_profile_projects_rows() {
        Spi::run(
            "CREATE TABLE ec_hnsw_debug_scan_heap_fetch_fixture \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_debug_scan_heap_fetch_fixture VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.95, 0.05, 0.45, -0.95], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.9, 0.1, 0.4, -0.9], 4, 42)),
             (4, encode_to_ecvector(ARRAY[-1.0, 0.0, -0.5, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_debug_scan_heap_fetch_fixture_idx \
             ON ec_hnsw_debug_scan_heap_fetch_fixture USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let (result_count, slot_fetch_count, projected_count) = Spi::connect(|client| {
            let row = client
                .select(
                    "SELECT result_count, slot_fetch_count, projected_count
                     FROM tests.ec_hnsw_debug_scan_heap_fetch_profile(
                         'ec_hnsw_debug_scan_heap_fetch_fixture_idx'::regclass::oid,
                         ARRAY[1.0, 0.0, 0.5, -1.0]::real[],
                         2,
                         1
                     )",
                    None,
                    &[],
                )
                .expect("heap-fetch profile query should succeed")
                .next()
                .expect("heap-fetch profile query should return one row");
            (
                row["result_count"]
                    .value::<i32>()
                    .expect("result count should decode")
                    .expect("result count should be non-null"),
                row["slot_fetch_count"]
                    .value::<i32>()
                    .expect("slot fetch count should decode")
                    .expect("slot fetch count should be non-null"),
                row["projected_count"]
                    .value::<i32>()
                    .expect("projected count should decode")
                    .expect("projected count should be non-null"),
            )
        });

        assert_eq!(
            result_count, 2,
            "helper should stop after the requested row limit"
        );
        assert_eq!(
            slot_fetch_count, 2,
            "helper should fetch one visible heap tuple into the slot for each returned row on this simple fixture",
        );
        assert_eq!(
            projected_count, 2,
            "helper should project the requested heap attribute for each fetched row",
        );
    }

    #[pg_test]
    fn test_ech_debug_reachable_live_count_matches_admin_snapshot() {
        Spi::run(
            "CREATE TABLE ec_hnsw_debug_reachable_live_fixture \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_debug_reachable_live_fixture VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.95, 0.05, 0.45, -0.95], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.9, 0.1, 0.4, -0.9], 4, 42)),
             (4, encode_to_ecvector(ARRAY[-1.0, 0.0, -0.5, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_debug_reachable_live_fixture_idx \
             ON ec_hnsw_debug_reachable_live_fixture USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_debug_reachable_live_fixture_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let rust_count =
            i32::try_from(unsafe { am::debug_layer0_reachable_live_element_tids(index_oid).len() })
                .expect("reachable live element count should fit in i32");
        let sql_count = Spi::get_one::<i32>(
            "SELECT tests.ec_hnsw_debug_reachable_live_element_count(
                 'ec_hnsw_debug_reachable_live_fixture_idx'::regclass::oid
             )",
        )
        .expect("debug reachability SQL wrapper should succeed")
        .expect("debug reachability SQL wrapper should return a row");
        let live_count = Spi::get_one::<i64>(
            "SELECT total_live_nodes
             FROM ec_hnsw_index_admin_snapshot('ec_hnsw_debug_reachable_live_fixture_idx'::regclass)",
        )
        .expect("admin snapshot query should succeed")
        .expect("admin snapshot should return a row");

        assert_eq!(
            sql_count, rust_count,
            "the SQL-visible reachability wrapper should match the Rust helper",
        );
        assert_eq!(
            i64::from(sql_count),
            live_count,
            "the reachable live element count should match the admin snapshot on a connected fixture",
        );
    }

    #[pg_test]
    fn test_ech_scan_scaffold_allocates_and_frees_state() {
        Spi::run("CREATE TABLE ec_hnsw_scan_scaffold (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_scan_scaffold VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_scan_scaffold_idx ON ec_hnsw_scan_scaffold USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_scan_scaffold_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (has_opaque, cleared_opaque) = unsafe { am::debug_begin_end_scan(index_oid) };
        assert!(has_opaque, "ambeginscan should allocate scan opaque state");
        assert!(cleared_opaque, "amendscan should release scan opaque state");
    }

    #[pg_test]
    fn test_ech_scan_scaffold_amendscan_is_idempotent() {
        Spi::run(
            "CREATE TABLE ec_hnsw_scan_scaffold_idempotent (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_scan_scaffold_idempotent VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_scan_scaffold_idempotent_idx ON ec_hnsw_scan_scaffold_idempotent USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_scan_scaffold_idempotent_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (has_opaque, cleared_after_first, cleared_after_second) =
            unsafe { am::debug_end_scan_twice(index_oid) };
        assert!(has_opaque, "ambeginscan should allocate scan opaque state");
        assert!(
            cleared_after_first,
            "first amendscan call should clear scan opaque state"
        );
        assert!(
            cleared_after_second,
            "second amendscan call should remain a benign no-op"
        );
    }

    #[pg_test]
    fn test_ech_rescan_scaffold_records_query_dimensions() {
        let index_oid = setup_rescan_scaffold_index("ec_hnsw_rescan_scaffold");
        let expected_query = vec![1.0, 0.0, 0.5, -1.0];
        let (
            rescan_called,
            query_dimensions,
            stored_query,
            scan_dimensions,
            scan_bits,
            scan_code_len,
            scan_block_count,
            has_prepared_query,
            prepared_lut_len,
            prepared_sq_len,
        ) = unsafe { am::debug_rescan_query_dimensions(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        assert!(
            rescan_called,
            "amrescan should mark scan state as initialized"
        );
        assert_eq!(query_dimensions, 4);
        assert_eq!(stored_query, expected_query);
        assert_eq!(scan_dimensions, 4);
        assert_eq!(scan_bits, 4);
        assert_eq!(scan_code_len, code_len(4, 4));
        assert!(
            scan_block_count >= 2,
            "rescan should cache the current index block count"
        );
        assert!(
            has_prepared_query,
            "non-empty rescans should cache prepared query state for future ordered search"
        );
        assert_eq!(prepared_lut_len, 32);
        assert_eq!(prepared_sq_len, 4);
    }

    #[pg_test]
    fn test_ech_rescan_repeat_overwrites_query_dimensions() {
        Spi::run("CREATE TABLE ec_hnsw_rescan_repeat (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_rescan_repeat_idx ON ec_hnsw_rescan_repeat USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_rescan_repeat_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let second_query = vec![1.0, 0.0, 0.5];
        let (
            rescan_called,
            query_dimensions,
            stored_query,
            scan_dimensions,
            scan_bits,
            scan_code_len,
            scan_block_count,
            has_prepared_query,
            prepared_lut_len,
            prepared_sq_len,
        ) = unsafe {
            am::debug_rescan_overwrites_query_dimensions(
                index_oid,
                vec![1.0, 0.0, 0.5, -1.0],
                second_query.clone(),
            )
        };
        assert!(
            rescan_called,
            "repeated amrescan should keep scan state initialized"
        );
        assert_eq!(
            query_dimensions, 3,
            "second amrescan should overwrite recorded query dimensions"
        );
        assert_eq!(
            stored_query, second_query,
            "second amrescan should overwrite the stored query payload"
        );
        assert_eq!(scan_dimensions, 0);
        assert_eq!(scan_bits, 0);
        assert_eq!(scan_code_len, 0);
        assert_eq!(scan_block_count, 1);
        assert!(
            !has_prepared_query,
            "empty-index rescans should not allocate prepared query state yet"
        );
        assert_eq!(prepared_lut_len, 0);
        assert_eq!(prepared_sq_len, 0);
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw scan query dimension mismatch")]
    fn test_ech_rescan_scaffold_rejects_wrong_query_dimensions() {
        let index_oid = setup_rescan_scaffold_index("ec_hnsw_rescan_scaffold_mismatch");
        let _ = unsafe { am::debug_rescan_query_dimensions(index_oid, vec![1.0, 0.0, 0.5]) };
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw scan query must not be NULL")]
    fn test_ech_rescan_scaffold_rejects_null_query() {
        let index_oid = setup_rescan_scaffold_index("ec_hnsw_rescan_scaffold_null");
        unsafe { am::debug_rescan_null_query(index_oid) };
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw scan query must not be empty")]
    fn test_ech_rescan_scaffold_rejects_empty_query() {
        let index_oid = setup_rescan_scaffold_index("ec_hnsw_rescan_scaffold_empty");
        let _ = unsafe { am::debug_rescan_query_dimensions(index_oid, vec![]) };
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw scan query dimension 65536 exceeds maximum 65535")]
    fn test_ech_rescan_scaffold_rejects_oversized_query() {
        Spi::run(
            "CREATE TABLE ec_hnsw_rescan_scaffold_oversized (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_rescan_scaffold_oversized_idx ON ec_hnsw_rescan_scaffold_oversized USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_rescan_scaffold_oversized_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let oversized_query = vec![0.0_f32; (u16::MAX as usize) + 1];
        let _ = unsafe { am::debug_rescan_query_dimensions(index_oid, oversized_query) };
    }

    #[pg_test]
    fn test_ech_rescan_scaffold_accepts_unused_zero_key_buffer() {
        let query = vec![1.0, 0.0, 0.5, -1.0];
        let index_oid = setup_rescan_scaffold_index("ec_hnsw_rescan_scaffold_zero_qual_buffer");
        let (
            rescan_called,
            query_dimensions,
            stored_query,
            scan_dimensions,
            scan_bits,
            scan_code_len,
            scan_block_count,
            has_prepared_query,
            prepared_lut_len,
            prepared_sq_len,
        ) = unsafe { am::debug_rescan_with_unused_key_buffer(index_oid, query.clone()) };

        assert!(rescan_called, "amrescan should still initialize scan state");
        assert_eq!(query_dimensions, query.len() as u16);
        assert_eq!(stored_query, query);
        assert_eq!(scan_dimensions, 4);
        assert_eq!(scan_bits, 4);
        assert_eq!(scan_code_len, code_len(4, 4));
        assert!(
            scan_block_count >= 2,
            "rescan should cache the current index block count"
        );
        assert!(
            has_prepared_query,
            "non-empty rescans should prepare the query"
        );
        assert_eq!(prepared_lut_len, 32);
        assert_eq!(prepared_sq_len, 4);
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw scan does not support index quals yet")]
    fn test_ech_rescan_scaffold_rejects_index_quals() {
        let index_oid = setup_rescan_scaffold_index("ec_hnsw_rescan_scaffold_quals");
        unsafe { am::debug_rescan_with_index_qual(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw scan currently requires exactly one ORDER BY query")]
    fn test_ech_rescan_scaffold_rejects_multiple_orderbys() {
        let index_oid = setup_rescan_scaffold_index("ec_hnsw_rescan_scaffold_multi_orderby");
        unsafe { am::debug_rescan_with_multiple_orderbys(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw amgettuple requires amrescan before scan execution")]
    fn test_ech_gettuple_scaffold_requires_rescan() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_scaffold (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_scaffold VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_scaffold_idx ON ec_hnsw_gettuple_scaffold USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_gettuple_scaffold_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        unsafe { am::debug_gettuple_without_rescan(index_oid) };
    }

    #[pg_test]
    fn test_ech_gettuple_returns_heap_tids() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_exec_scaffold (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_exec_scaffold VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_exec_scaffold_idx ON ec_hnsw_gettuple_exec_scaffold USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_exec_scaffold_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let observed_tids =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM ec_hnsw_gettuple_exec_scaffold
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        let mut observed_tids = observed_tids;
        let mut expected_tids = expected_tids;
        observed_tids.sort_unstable();
        expected_tids.sort_unstable();

        assert!(
            observed_tids.contains(&expected_tids[0]),
            "graph-first scan should return the nearest indexed heap tid for the query"
        );
        assert_eq!(
            observed_tids.len(),
            observed_tids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "graph-first scan should not emit duplicate heap tids"
        );
        assert!(
            observed_tids
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "every emitted heap tid should still belong to the indexed table"
        );
    }

    #[pg_test]
    fn test_ech_sql_ordered_index_scan_executes() {
        Spi::run(
            "CREATE TABLE ec_hnsw_sql_ordered_exec (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_sql_ordered_exec VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_sql_ordered_exec_idx ON ec_hnsw_sql_ordered_exec USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("ANALYZE ec_hnsw_sql_ordered_exec").expect("analyze should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM ec_hnsw_sql_ordered_exec \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 2",
                    None,
                    &[],
                )
                .expect("EXPLAIN should succeed")
                .first();
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("plan row should decode")
                        .expect("plan row should not be NULL"),
                );
            }
            lines.join("\n")
        });

        assert!(
            plan.contains("Index Scan") || plan.contains("Index Only Scan"),
            "ordered execution test should route through ec_hnsw at runtime: {plan}"
        );

        let ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_hnsw_sql_ordered_exec \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 2",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(
            ordered_ids.len(),
            2,
            "query should return the requested LIMIT"
        );
        assert_eq!(
            ordered_ids[0], 1,
            "runtime ordered index scan should return the nearest vector first"
        );
    }

    #[pg_test]
    fn test_ech_graph_first_scan_emits_distance_sorted_scores() {
        Spi::run(
            "CREATE TABLE ec_hnsw_graph_first_ordered_scores (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_graph_first_ordered_scores VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.92, 0.08, 0.0, 0.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.75, 0.25, 0.0, 0.0], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.55, 0.45, 0.0, 0.0], 4, 42)),
             (5, encode_to_ecvector(ARRAY[0.35, 0.65, 0.0, 0.0], 4, 42)),
             (6, encode_to_ecvector(ARRAY[0.15, 0.85, 0.0, 0.0], 4, 42)),
             (7, encode_to_ecvector(ARRAY[-0.2, 0.98, 0.0, 0.0], 4, 42)),
             (8, encode_to_ecvector(ARRAY[-0.7, 0.3, 0.0, 0.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_graph_first_ordered_scores_idx ON ec_hnsw_graph_first_ordered_scores USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 4, ef_construction = 64)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_graph_first_ordered_scores_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let ctid_to_id = ctid_id_map("ec_hnsw_graph_first_ordered_scores");
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_scores(index_oid, vec![1.0, 0.05, 0.0, 0.0])
        };

        assert!(
            observed.len() >= 3,
            "non-trivial built indexes should emit multiple graph-first ordered results"
        );
        assert_eq!(
            observed.len(),
            observed
                .iter()
                .map(|(heap_tid, _)| *heap_tid)
                .collect::<HashSet<_>>()
                .len(),
            "graph-first ordered scans should not emit the same heap tid twice"
        );
        assert!(
            observed
                .windows(2)
                .all(|pair| pair[0].1 <= pair[1].1 + f32::EPSILON),
            "graph-first scan should emit tuples in nondecreasing operator-facing <#> score order"
        );
        assert!(
            observed
                .iter()
                .all(|(heap_tid, _)| ctid_to_id.contains_key(heap_tid)),
            "every emitted heap tid should map back to a row in the built index table"
        );
    }

    #[pg_test]
    fn test_ech_gettuple_tracks_current_result_state() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_result_state (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_result_state VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_result_state_idx ON ec_hnsw_gettuple_result_state USING ec_hnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_result_state_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![1.0, 0.0, 0.5, -1.0];
        let (
            before_found,
            before_tid,
            before_score,
            before_score_value,
            found,
            after_tid,
            after_score,
            after_score_value,
        ) = unsafe { am::debug_gettuple_current_result_state(index_oid, query.clone()) };
        let expected_score = Spi::get_one::<f32>(&format!(
            "SELECT embedding <#> ARRAY[{},{},{},{}]::real[] \
             FROM ec_hnsw_gettuple_result_state WHERE id = 1",
            query[0], query[1], query[2], query[3],
        ))
        .expect("score query should succeed")
        .expect("score should exist");

        assert!(
            before_found,
            "seeded graph-first rescans should prefill the first ordered result before amgettuple runs"
        );
        assert_ne!(
            before_tid,
            (u32::MAX, u16::MAX),
            "seeded graph-first rescans should expose a concrete current-result element tid immediately"
        );
        assert!(
            before_score,
            "seeded graph-first rescans should expose an order-by score before the first tuple drain"
        );
        assert_eq!(
            before_score_value, expected_score,
            "prefilled graph-first current-result score should already match the operator-facing <#> value"
        );
        assert!(
            found,
            "first gettuple call should produce a tuple for a non-empty index"
        );
        if after_tid == (u32::MAX, u16::MAX) {
            assert!(
                !after_score,
                "if the graph lane exhausts immediately after the first tuple drain, it should clear the current-result score-valid bit too"
            );
            assert_eq!(
                after_score_value, 0.0,
                "if the graph lane exhausts immediately after the first tuple drain, it should clear the current-result score value too"
            );
        } else {
            assert!(
                after_score,
                "when graph traversal stays hot after the first tuple drain, it should keep the current result score valid"
            );
            assert_ne!(
                after_score_value, 0.0,
                "when graph traversal stays hot after the first tuple drain, it should keep a concrete current-result score populated"
            );
            assert_ne!(
                after_tid, before_tid,
                "when graph traversal has another result ready, the current result should advance to the next prefetched candidate"
            );
        }
    }

    #[pg_test]
    fn test_ech_gettuple_emits_orderby_score() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_orderby_score (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_orderby_score VALUES
             (1, encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_tqvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_orderby_score_idx ON ec_hnsw_gettuple_orderby_score USING ec_hnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_orderby_score_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let query = vec![1.0, 0.0, 0.5, -1.0];
        let (found, orderby_is_null, orderby_score) =
            unsafe { am::debug_gettuple_orderby_score(index_oid, query.clone()) };
        let expected_score = Spi::get_one::<f32>(&format!(
            "SELECT embedding <#> ARRAY[{},{},{},{}]::real[] \
             FROM ec_hnsw_gettuple_orderby_score WHERE id = 1",
            query[0], query[1], query[2], query[3],
        ))
        .expect("score query should succeed")
        .expect("score should exist");

        assert!(
            found,
            "first gettuple call should produce a tuple for a non-empty index"
        );
        assert!(
            !orderby_is_null,
            "visible tuple production should populate xs_orderbynulls[0] as non-null"
        );
        assert_eq!(
            orderby_score, expected_score,
            "amgettuple should publish the current result score through xs_orderbyvals[0]"
        );
    }

    #[pg_test]
    fn test_ech_gettuple_clears_orderby_score_on_rescan() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_orderby_lifecycle (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_orderby_lifecycle VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_orderby_lifecycle_idx ON ec_hnsw_gettuple_orderby_lifecycle USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_orderby_lifecycle_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before, after_first, exhausted, rescanned) = unsafe {
            am::debug_gettuple_orderby_score_lifecycle(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_eq!(
            before, None,
            "order-by output should start empty before tuple production"
        );
        assert!(
            after_first.is_some(),
            "first tuple production should publish a non-null order-by score"
        );
        assert_eq!(
            exhausted, None,
            "exhaustion should clear the visible order-by score instead of leaving stale output"
        );
        assert_eq!(
            rescanned, None,
            "amrescan should clear any prior order-by score before the next tuple is produced"
        );
    }

    #[pg_test]
    fn test_ech_gettuple_current_result_lifecycle() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_result_lifecycle (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_result_lifecycle VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_result_lifecycle_idx ON ec_hnsw_gettuple_result_lifecycle USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_result_lifecycle_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            first_tid,
            second_tid,
            second_score,
            second_score_value,
            exhausted_tid,
            exhausted_score,
            exhausted_score_value,
            rescanned_tid,
            rescanned_score,
        ) = unsafe {
            am::debug_gettuple_current_result_lifecycle(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_ne!(
            second_tid, first_tid,
            "after the last duplicate drain, graph traversal should either prefill the next current result or clear the old one"
        );
        if second_tid == (u32::MAX, u16::MAX) {
            assert!(
                !second_score,
                "if graph traversal exhausts after the last duplicate drain, it should clear the current-result score-valid bit"
            );
            assert_eq!(
                second_score_value, 0.0,
                "if graph traversal exhausts after the last duplicate drain, it should clear the current-result score value"
            );
        } else {
            assert!(
                second_score,
                "prefilling the next graph result should keep the current result score valid"
            );
            assert_ne!(
                second_score_value, 0.0,
                "prefilling the next graph result should keep a concrete score populated"
            );
        }
        assert_eq!(
            exhausted_tid,
            (u32::MAX, u16::MAX),
            "current result state should clear after full scan exhaustion"
        );
        assert!(
            !exhausted_score,
            "exhaustion should clear any current result score-valid bit"
        );
        assert_eq!(
            exhausted_score_value, 0.0,
            "exhaustion should clear the current result score value"
        );
        assert_eq!(
            rescanned_tid,
            first_tid,
            "amrescan should prefill the first graph-ordered current result again before the next tuple is produced"
        );
        assert!(
            rescanned_score,
            "amrescan should restore the current result score-valid bit for the prefetched graph result"
        );
    }

    #[pg_test]
    fn test_ech_gettuple_current_result_tracks_heap_progress() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_result_heap_progress (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_result_heap_progress VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_result_heap_progress_idx ON ec_hnsw_gettuple_result_heap_progress USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_result_heap_progress_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            element_tid,
            first_heap_tid,
            second_element_tid,
            second_heap_tid,
            first_score,
            second_score,
        ) = unsafe {
            am::debug_gettuple_current_result_heap_progress(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_ne!(
            element_tid,
            (u32::MAX, u16::MAX),
            "current result should keep a concrete element tid while draining duplicates"
        );
        assert_ne!(
            first_heap_tid,
            (u32::MAX, u16::MAX),
            "first produced tuple should attach a concrete heap tid to current result state"
        );
        assert_ne!(
            second_element_tid,
            element_tid,
            "after the last duplicate drain, the graph lane should prefill the next current result element"
        );
        assert_ne!(
            second_heap_tid, first_heap_tid,
            "the prefetched next result should not leave the old duplicate heap tid attached"
        );
        assert_eq!(
            second_heap_tid,
            (u32::MAX, u16::MAX),
            "a freshly prefetched next result should not yet have a heap tid attached"
        );
        assert_ne!(
            first_score, second_score,
            "prefilling the next result should allow the current-result score to advance with the graph order"
        );
    }

    #[pg_test]
    fn test_ech_rescan_seeds_entry_candidate_state() {
        Spi::run(
            "CREATE TABLE ec_hnsw_entry_candidate_state (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_entry_candidate_state VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_entry_candidate_state_idx ON ec_hnsw_entry_candidate_state USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_entry_candidate_state_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before_valid, before_tid, before_score, after_valid, after_tid, after_score) =
            unsafe { am::debug_rescan_entry_candidate_state(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert!(
            before_valid,
            "amrescan should seed a concrete graph-ordered starting result for a non-empty index"
        );
        assert_ne!(before_tid, (u32::MAX, u16::MAX));
        assert_ne!(
            before_score, 0.0,
            "the initial graph-ordered result should carry a computed score for future tuple production"
        );
        assert!(
            !after_valid,
            "the initial graph result state should clear once the bootstrap scan fully exhausts"
        );
        assert_eq!(
            after_tid,
            (u32::MAX, u16::MAX),
            "exhaustion should clear the entry candidate tuple pointer"
        );
        assert_eq!(
            after_score, 0.0,
            "exhaustion should clear the entry candidate score"
        );
    }

    #[pg_test]
    fn test_ech_entry_candidate_persists_until_exhaustion() {
        Spi::run(
            "CREATE TABLE ec_hnsw_entry_candidate_lifecycle (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_entry_candidate_lifecycle VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_entry_candidate_lifecycle_idx ON ec_hnsw_entry_candidate_lifecycle USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_entry_candidate_lifecycle_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            before_valid,
            _before_tid,
            _before_score,
            partial_valid,
            partial_tid,
            partial_score,
            partial_result_tid,
            partial_exhausted,
            exhausted_valid,
            exhausted_tid,
            exhausted_score,
        ) = unsafe { am::debug_entry_candidate_lifecycle(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert!(
            before_valid,
            "entry candidate should be seeded before tuple production"
        );
        assert!(
            partial_valid || partial_result_tid != (u32::MAX, u16::MAX) || partial_exhausted,
            "partial scan progress should keep either a remaining frontier candidate, a concrete current result, or an explicit exhausted state"
        );
        if partial_valid {
            assert_ne!(
                partial_tid,
                (u32::MAX, u16::MAX),
                "partial scan progress should keep a concrete frontier candidate tid when one remains"
            );
            assert_ne!(
                partial_score, 0.0,
                "partial scan progress should keep a concrete frontier candidate score when one remains"
            );
        } else {
            if partial_result_tid == (u32::MAX, u16::MAX) {
                assert!(
                    partial_exhausted,
                    "if partial scan progress no longer exposes a frontier head or current result, the graph lane should already be exhausted"
                );
            } else {
                assert_ne!(
                    partial_result_tid,
                    (u32::MAX, u16::MAX),
                    "when the frontier head materializes immediately, partial scan progress should keep a concrete current-result tid"
                );
            }
        }
        assert!(
            !exhausted_valid,
            "entry candidate should clear once the bootstrap scan fully exhausts"
        );
        assert_eq!(
            exhausted_tid,
            (u32::MAX, u16::MAX),
            "entry candidate tid should clear on exhaustion"
        );
        assert_eq!(
            exhausted_score, 0.0,
            "entry candidate score should clear on exhaustion"
        );
    }

    #[pg_test]
    fn test_ech_successor_candidate_from_entry_adjacency() {
        Spi::run(
            "CREATE TABLE ec_hnsw_successor_candidate_state (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_successor_candidate_state VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_successor_candidate_state_idx ON ec_hnsw_successor_candidate_state USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_successor_candidate_state_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            entry_tid,
            entry_neighbors,
            successor_valid,
            successor_tid,
            _successor_source_tid,
            successor_score,
        ) = unsafe {
            am::debug_rescan_successor_candidate_state(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_ne!(
            entry_tid,
            (u32::MAX, u16::MAX),
            "non-empty index should expose a concrete entry point"
        );
        if entry_neighbors.is_empty() {
            assert!(
                !successor_valid,
                "successor candidate should stay empty when the persisted entry adjacency is empty"
            );
            assert_eq!(
                successor_tid,
                (u32::MAX, u16::MAX),
                "empty successor candidate should clear its tuple pointer"
            );
            assert_eq!(
                successor_score, 0.0,
                "empty successor candidate should clear its score"
            );
        } else {
            assert!(
                successor_valid,
                "successor candidate should seed from persisted entry adjacency when a live neighbor exists"
            );
            assert!(
                successor_tid != (u32::MAX, u16::MAX),
                "after amrescan prefill, a live entry adjacency should leave a concrete next ordered slot"
            );
            assert_ne!(
                successor_score, 0.0,
                "seeded successor candidate should carry a computed score"
            );
        }
    }

    #[pg_test]
    fn test_ech_rescan_builds_bootstrap_candidate_frontier() {
        Spi::run("RESET ec_hnsw.ef_search").expect("reset should succeed");
        Spi::run("RESET ec_hnsw.disable_binary_prefilter").expect("reset should succeed");
        Spi::run("RESET ec_hnsw.force_binary_derivation").expect("reset should succeed");
        Spi::run(
            "CREATE TABLE ec_hnsw_candidate_frontier_state (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_candidate_frontier_state VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_candidate_frontier_state_idx ON ec_hnsw_candidate_frontier_state USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_candidate_frontier_state_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (head, frontier, frontier_slots, frontier_provenance, expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        let valid_entry_neighbors = unsafe { am::debug_entry_point_neighbor_tids(index_oid) }
            .into_iter()
            .filter(|tid| *tid != (u32::MAX, u16::MAX))
            .collect::<Vec<_>>();

        assert!(
            !frontier.is_empty(),
            "bootstrap frontier should not be empty immediately after rescan"
        );
        assert!(
            frontier[0].0,
            "first frontier slot should hold the seeded entry candidate"
        );
        assert_ne!(
            frontier[0].1,
            (u32::MAX, u16::MAX),
            "first frontier slot should expose a concrete element tid"
        );
        assert_ne!(
            frontier[0].2, 0.0,
            "first frontier slot should carry a computed score"
        );

        if let Some(second) = frontier.get(1) {
            assert_ne!(
                second.1,
                (u32::MAX, u16::MAX),
                "second frontier slot should expose a concrete element tid when present"
            );
            assert_ne!(
                second.2, 0.0,
                "second frontier slot should carry a computed score when present"
            );
        } else {
            assert_eq!(frontier.len(), 1, "a missing second frontier slot should mean the Vec contains only the seeded entry candidate");
        }

        assert_eq!(
            frontier_slots.first().map(|slot| slot.1),
            Some(frontier_provenance[0].1),
            "frontier and provenance views should agree on the seeded entry candidate tid"
        );
        assert!(
            !frontier_slots.is_empty(),
            "bootstrap frontier should always contain the seeded entry candidate"
        );
        assert!(
            frontier_slots.len() <= 3,
            "bootstrap frontier should stay within the current bounded traversal width"
        );
        assert!(
            frontier_slots.len() > valid_entry_neighbors.len().min(1),
            "bootstrap frontier should at least seed the entry candidate and any immediately discoverable live neighbor"
        );

        let expected_head = frontier_slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.0)
            .min_by(|(left_index, left), (right_index, right)| {
                left.2.total_cmp(&right.2).then(left_index.cmp(right_index))
            })
            .map(|(_, slot)| slot.1);
        assert_eq!(
            head, expected_head,
            "frontier head should pick the best valid candidate across the current widened bootstrap frontier"
        );

        let entry_tid = frontier_provenance
            .first()
            .map(|slot| slot.1)
            .expect("frontier provenance should include the seeded entry candidate");
        assert_eq!(
            frontier_provenance[0].2,
            (u32::MAX, u16::MAX),
            "the seeded entry candidate should not claim discovery from another element"
        );
        let seeded_candidate_tids = frontier_provenance
            .iter()
            .filter_map(|slot| slot.0.then_some(slot.1))
            .collect::<Vec<_>>();
        assert!(
            expanded_sources.contains(&entry_tid),
            "bootstrap expanded-source state should always include the entry candidate"
        );
        assert!(
            expanded_sources
                .iter()
                .all(|source_tid| seeded_candidate_tids.contains(source_tid)),
            "bootstrap expanded-source state should only contain seeded candidate tids"
        );
    }

    #[pg_test]
    fn test_ech_rescan_respects_ef_search_frontier_limit() {
        Spi::run(
            "CREATE TABLE ec_hnsw_candidate_frontier_limit (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_candidate_frontier_limit VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_candidate_frontier_limit_idx ON ec_hnsw_candidate_frontier_limit USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (ef_search = 1)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_candidate_frontier_limit_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_head, frontier, frontier_slots, frontier_provenance, _expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert_eq!(
            frontier.len(),
            1,
            "ef_search=1 should cap the visible bootstrap frontier at one candidate"
        );
        assert_eq!(
            frontier_slots.len(),
            1,
            "debug frontier slots should match the configured bootstrap frontier limit"
        );
        assert_eq!(
            frontier_provenance.len(),
            1,
            "frontier provenance should track only the single retained candidate"
        );
    }

    #[pg_test]
    fn test_ech_session_ef_search_override_limits_runtime_frontier() {
        Spi::run(
            "CREATE TABLE ec_hnsw_session_runtime_frontier_limit (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_session_runtime_frontier_limit VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_session_runtime_frontier_limit_idx ON ec_hnsw_session_runtime_frontier_limit USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (ef_search = 3)",
        )
        .expect("index creation should succeed");
        Spi::run("SET ec_hnsw.ef_search = 1").expect("session override should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_session_runtime_frontier_limit_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_head, frontier, frontier_slots, frontier_provenance, _expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert_eq!(
            frontier.len(),
            1,
            "non-default session ef_search should override the reloption during scan bootstrap"
        );
        assert_eq!(
            frontier_slots.len(),
            1,
            "runtime frontier slots should honor the resolved session override width"
        );
        assert_eq!(
            frontier_provenance.len(),
            1,
            "runtime frontier provenance should also honor the resolved session override width"
        );

        Spi::run("RESET ec_hnsw.ef_search").expect("reset should succeed");
    }

    #[pg_test]
    fn test_ech_session_ef_search_defaults_to_relation_setting() {
        Spi::run(
            "CREATE TABLE ec_hnsw_session_ef_search_reloption (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_session_ef_search_reloption VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_session_ef_search_reloption_idx ON ec_hnsw_session_ef_search_reloption USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (ef_search = 111)",
        )
        .expect("index creation should succeed");
        Spi::run("RESET ec_hnsw.ef_search").expect("reset should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_session_ef_search_reloption_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let snapshot = unsafe { am::debug_planner_tuning_snapshot(index_oid) };

        assert_eq!(snapshot.relation_ef_search, 111);
        assert_eq!(snapshot.session_ef_search, None);
        assert_eq!(
            snapshot.effective_ef_search, 111,
            "default session setting should fall back to the index reloption"
        );
        assert_eq!(snapshot.effective_source, "relation");
        assert!(
            snapshot.planner_scan_enabled,
            "planner-facing scan selection should be live after D2 cost-model activation"
        );
    }

    #[pg_test]
    fn test_ech_session_ef_search_overrides_reloption() {
        Spi::run(
            "CREATE TABLE ec_hnsw_session_ef_search_override (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_session_ef_search_override VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_session_ef_search_override_idx ON ec_hnsw_session_ef_search_override USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (ef_search = 111)",
        )
        .expect("index creation should succeed");
        Spi::run("SET ec_hnsw.ef_search = 7").expect("session override should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_session_ef_search_override_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let snapshot = unsafe { am::debug_planner_tuning_snapshot(index_oid) };

        assert_eq!(snapshot.relation_ef_search, 111);
        assert_eq!(snapshot.session_ef_search, Some(7));
        assert_eq!(
            snapshot.effective_ef_search, 7,
            "non-default session setting should override the index reloption"
        );
        assert_eq!(snapshot.effective_source, "session");

        Spi::run("RESET ec_hnsw.ef_search").expect("reset should succeed");
    }

    #[pg_test]
    fn test_ech_frontier_head_persists_until_exhaustion() {
        Spi::run(
            "CREATE TABLE ec_hnsw_frontier_head_lifecycle (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_frontier_head_lifecycle VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_frontier_head_lifecycle_idx ON ec_hnsw_frontier_head_lifecycle USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_frontier_head_lifecycle_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            before_head,
            before_frontier,
            partial_head,
            partial_frontier,
            exhausted_head,
            exhausted_frontier,
        ) = unsafe {
            am::debug_candidate_frontier_head_lifecycle(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert_ne!(
            before_head, None,
            "seeded graph-first rescans should expose a concrete ordered head immediately after amrescan"
        );
        assert_eq!(
            partial_head.is_some(),
            partial_frontier.iter().any(|slot| slot.0),
            "ordered-head presence should still track whether any graph-ordered candidates remain after partial progress"
        );
        assert!(
            partial_frontier.len() <= before_frontier.len(),
            "draining the first graph-ordered tuple should not grow the remaining ordered runtime state"
        );
        assert_eq!(
            exhausted_head, None,
            "frontier head should clear on full scan exhaustion"
        );
        assert_eq!(
            exhausted_frontier,
            Vec::<(bool, (u32, u16), f32)>::new(),
            "full scan exhaustion should clear both frontier slots"
        );
    }

    #[pg_test]
    fn test_ech_consume_candidate_frontier_head_reselects_or_clears() {
        Spi::run(
            "CREATE TABLE ec_hnsw_frontier_head_consume (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_frontier_head_consume VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_frontier_head_consume_idx ON ec_hnsw_frontier_head_consume USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_frontier_head_consume_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (
            before_head,
            before_frontier,
            after_first_head,
            after_first_frontier,
            after_second_head,
            after_second_frontier,
        ) = unsafe {
            am::debug_consume_candidate_frontier_head(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        if let Some(consumed_tid) = before_head {
            let remaining_slot = before_frontier
                .iter()
                .position(|slot| slot.0 && slot.1 != consumed_tid);

            if let Some(remaining_slot) = remaining_slot {
                assert_eq!(
                    after_first_head,
                    Some(before_frontier[remaining_slot].1),
                    "when another candidate remains valid, consuming the head should expose that remaining candidate as the new head"
                );
                assert!(
                    after_first_frontier
                        .iter()
                        .any(|slot| slot.1 == before_frontier[remaining_slot].1),
                    "consuming the current head should preserve the remaining candidate after compaction"
                );
                assert!(
                    !after_first_frontier
                        .iter()
                        .any(|slot| slot.1 == consumed_tid),
                    "consuming the current head should remove that candidate from the frontier Vec"
                );
            } else {
                assert_eq!(
                    after_first_head, None,
                    "consuming the only valid candidate should invalidate the frontier head"
                );
                assert!(
                    after_first_frontier.is_empty(),
                    "consuming the only valid candidate should leave the compacted frontier empty"
                );
            }
        } else {
            assert_eq!(
                after_first_head, None,
                "after amrescan prefill, a tiny index may have no remaining raw frontier candidate to consume"
            );
            assert!(
                after_first_frontier.is_empty(),
                "without any remaining raw frontier candidates, the consume helper should keep the frontier empty"
            );
        }

        assert_eq!(
            after_second_head, None,
            "consuming the frontier head again should leave the frontier empty"
        );
        assert_eq!(
            after_second_frontier,
            Vec::<(bool, (u32, u16), f32)>::new(),
            "after consuming both available slots, the frontier Vec should be fully cleared"
        );
    }

    #[pg_test]
    fn test_ech_frontier_head_refills_from_consumed_neighbors() {
        Spi::run(
            "CREATE TABLE ec_hnsw_frontier_head_refill (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_frontier_head_refill VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.5, -1.0, 1.0, 0.0], 4, 42)),
             (5, encode_to_ecvector(ARRAY[-0.5, 0.5, 1.0, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_frontier_head_refill_idx ON ec_hnsw_frontier_head_refill USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (ef_search = 3)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_frontier_head_refill_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let valid_entry_neighbors = unsafe { am::debug_entry_point_neighbor_tids(index_oid) }
            .into_iter()
            .filter(|tid| *tid != (u32::MAX, u16::MAX))
            .collect::<Vec<_>>();
        let (
            before_head,
            before_slots,
            consumed_tid,
            consumed_neighbors,
            after_head,
            after_slots,
            after_provenance_slots,
        ) = unsafe {
            am::debug_consume_candidate_frontier_head_slots(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        let expected_visible_width = 1 + valid_entry_neighbors.len().min(2);
        assert!(
            before_slots.len() <= expected_visible_width,
            "prefilled graph-first state should never expose more raw frontier slots than the configured width"
        );
        assert!(
            before_slots.len() >= expected_visible_width.saturating_sub(1),
            "prefilled graph-first state may leave the raw frontier one slot narrower once the current result has already been materialized"
        );
        let after_tids = after_slots
            .iter()
            .map(|slot| slot.1)
            .collect::<std::collections::BTreeSet<_>>();
        if before_slots.is_empty() {
            assert!(
                before_head.is_none(),
                "without any remaining raw frontier slots, the raw frontier head should already be empty"
            );
            assert_eq!(
                consumed_tid,
                (u32::MAX, u16::MAX),
                "when amrescan has already materialized the ordered head into current-result state, the raw frontier helper may have nothing left to consume"
            );
            assert!(
                after_head.is_none() && after_slots.is_empty(),
                "without any remaining raw frontier slots, manual frontier consume/refill should leave the raw frontier empty"
            );
            return;
        }

        assert_ne!(
            consumed_tid,
            (u32::MAX, u16::MAX),
            "non-empty frontier should expose an actually consumed candidate"
        );
        assert!(
            !after_tids.contains(&consumed_tid),
            "consuming the head should remove that candidate from the frontier"
        );
        assert_eq!(
            after_head.is_some(),
            !after_slots.is_empty(),
            "frontier head presence should track whether any candidates remain after consume/refill"
        );
        assert!(
            before_head.is_some(),
            "non-empty frontier should expose a head before consume/refill"
        );
        assert_eq!(
            after_tids.len(),
            after_slots.len(),
            "manual consume/refill should keep the raw frontier deduplicated"
        );

        let _ = after_provenance_slots;
        let _ = consumed_neighbors;
    }

    #[pg_test]
    fn test_ech_gettuple_consumes_bootstrap_candidate_state() {
        Spi::run(
            "CREATE TABLE ec_hnsw_bootstrap_consume_state (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_bootstrap_consume_state VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.5, -1.0, 1.0, 0.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_bootstrap_consume_state_idx ON ec_hnsw_bootstrap_consume_state USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_bootstrap_consume_state_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before_head, before_slots, current_result_tid, after_head, after_slots) = unsafe {
            am::debug_gettuple_consumes_bootstrap_candidate(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        let consumed_slot = before_slots
            .first()
            .copied()
            .expect("seeded graph-first rescans should prefill an ordered slot before the first tuple drain");
        assert_eq!(
            before_head,
            Some(consumed_slot.1),
            "seeded graph-first rescans should expose the prefetched current result as the ordered head before the first tuple drain"
        );
        assert_eq!(
            current_result_tid, consumed_slot.1,
            "the first amgettuple call should drain the already-prefilled ordered result"
        );
        assert_eq!(
            after_head.is_some(),
            !after_slots.is_empty(),
            "ordered-head presence should continue to track whether graph-ordered candidates remain after first tuple drain"
        );
        assert!(
            after_slots
                .iter()
                .all(|slot| slot.1 != consumed_slot.1 || slot.2 != consumed_slot.2),
            "after the first tuple drain, the previously emitted ordered slot should not remain queued as if it were still unseen"
        );
    }

    #[pg_test]
    fn test_ech_bootstrap_candidate_materializes_into_pending_drain() {
        Spi::run(
            "CREATE TABLE ec_hnsw_bootstrap_candidate_materialize (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_bootstrap_candidate_materialize VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_bootstrap_candidate_materialize_idx ON ec_hnsw_bootstrap_candidate_materialize USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_bootstrap_candidate_materialize_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (candidate_before, current_result_tid, pending_heap_tids, materialized) = unsafe {
            am::debug_materialize_bootstrap_candidate_result(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert!(
            candidate_before.0,
            "bootstrap frontier should yield a candidate before direct materialization"
        );
        assert!(
            materialized,
            "bootstrap candidate should materialize into the pending heap-tid drain path"
        );
        assert_eq!(
            current_result_tid, candidate_before.1,
            "materializing the bootstrap candidate should attach current-result state to that candidate"
        );
        assert_eq!(
            pending_heap_tids.len(),
            2,
            "duplicate-coalesced bootstrap candidates should populate all duplicate heap tids into pending drain state"
        );
    }

    #[pg_test]
    fn test_ech_bootstrap_phase_completes_and_resets_on_rescan() {
        Spi::run(
            "CREATE TABLE ec_hnsw_bootstrap_phase_transition (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_bootstrap_phase_transition VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_bootstrap_phase_transition_idx ON ec_hnsw_bootstrap_phase_transition USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_bootstrap_phase_transition_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before_complete, after_complete, after_head, after_frontier, rescanned_complete) =
            unsafe { am::debug_bootstrap_phase_transition(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert!(
            !before_complete,
            "amrescan should always start with bootstrap traversal enabled"
        );
        assert!(
            after_complete,
            "non-empty scan execution should eventually complete the current bootstrap phase"
        );
        assert_eq!(
            after_head, None,
            "once bootstrap phase completes, the visible frontier head should stay cleared"
        );
        assert_eq!(
            after_frontier,
            Vec::<(bool, (u32, u16), f32)>::new(),
            "once bootstrap phase completes, the visible frontier should be cleared too"
        );
        assert!(
            !rescanned_complete,
            "amrescan should reset bootstrap-phase completion for the next execution"
        );
    }

    #[pg_test]
    fn test_ech_visited_seed_state_tracks_frontier_candidates() {
        Spi::run(
            "CREATE TABLE ec_hnsw_visited_seed_state (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_visited_seed_state VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_visited_seed_state_idx ON ec_hnsw_visited_seed_state USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_visited_seed_state_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (head, _frontier, frontier_slots, _frontier_provenance, _expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        let (before, partial, exhausted) =
            unsafe { am::debug_visited_seed_lifecycle(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        let mut expected = frontier_slots
            .into_iter()
            .filter_map(|(valid, tid, _)| valid.then_some(tid))
            .collect::<Vec<_>>();
        expected.sort_unstable();

        assert_ne!(
            head, None,
            "non-empty scan frontier should still expose at least one seeded candidate"
        );
        assert_eq!(
            before, expected,
            "visited state should seed from the currently valid frontier candidate tids"
        );
        assert_eq!(
            partial, before,
            "bootstrap linear scan progress should not mutate seeded visited state yet"
        );
        assert_eq!(
            exhausted, before,
            "visited state should remain stable through bootstrap scan exhaustion until rescan"
        );
    }

    #[pg_test]
    fn test_ech_gettuple_current_result_exposes_neighbor_refs() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_current_neighbors (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_current_neighbors VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_current_neighbors_idx ON ec_hnsw_gettuple_current_neighbors USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_current_neighbors_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (current_result_tid, neighbor_count) = unsafe {
            am::debug_gettuple_current_result_neighbors(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };
        let (_block_count, metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };

        assert_ne!(
            current_result_tid,
            (u32::MAX, u16::MAX),
            "neighbor debug helper should attach to a concrete current result tuple"
        );
        assert!(
            neighbor_count <= am::page::neighbor_slots(metadata.max_level, metadata.m),
            "current-result neighbor count should decode within persisted neighbor capacity"
        );
    }

    #[pg_test]
    fn test_ech_entry_point_neighbor_refs_point_to_elements() {
        Spi::run(
            "CREATE TABLE ec_hnsw_entry_point_neighbors (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_entry_point_neighbors VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_entry_point_neighbors_idx ON ec_hnsw_entry_point_neighbors USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_entry_point_neighbors_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let neighbor_tids = unsafe { am::debug_entry_point_neighbor_tids(index_oid) };
        let (_block_count, _metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let element_tids = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples.iter().enumerate().filter_map(|(idx, tuple)| {
                    is_turboquant_element_tag(tuple.first().copied()).then_some((
                        page.block_number,
                        u16::try_from(idx + 1).expect("page tuple offset should fit in u16"),
                    ))
                })
            })
            .collect::<std::collections::HashSet<_>>();

        for neighbor_tid in neighbor_tids {
            assert!(
                element_tids.contains(&neighbor_tid),
                "entry-point neighbor ref should target an element tuple"
            );
        }
    }

    #[pg_test]
    fn test_ech_gettuple_drains_selected_duplicate_heap_tids() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_duplicate_exec (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_duplicate_exec VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_duplicate_exec_idx ON ec_hnsw_gettuple_duplicate_exec USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_duplicate_exec_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let observed_tids =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM ec_hnsw_gettuple_duplicate_exec
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        let mut observed_tids = observed_tids;
        let mut expected_tids = expected_tids;
        observed_tids.sort_unstable();
        expected_tids.sort_unstable();

        let selected_duplicate_heaptids = expected_tids[..2].to_vec();
        assert!(
            selected_duplicate_heaptids
                .iter()
                .all(|heap_tid| observed_tids.contains(heap_tid)),
            "graph-first scan should still drain every heap tid stored in the selected duplicate-coalesced element"
        );
        assert_eq!(
            observed_tids.len(),
            observed_tids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "graph-first scan should not emit any heap tid twice while draining duplicate-backed results"
        );
        assert!(
            observed_tids
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "every emitted heap tid should still come from the indexed table"
        );
    }

    #[pg_test]
    fn test_ech_gettuple_exhaustion_stays_false() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_exhaustion (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_exhaustion VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_exhaustion_idx ON ec_hnsw_gettuple_exhaustion USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_gettuple_exhaustion_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (observed_tids, exhausted_once, exhausted_twice) =
            unsafe { am::debug_gettuple_exhaustion_state(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM ec_hnsw_gettuple_exhaustion
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        let mut observed_tids = observed_tids;
        let mut expected_tids = expected_tids;
        observed_tids.sort_unstable();
        expected_tids.sort_unstable();
        assert!(
            !observed_tids.is_empty(),
            "graph-first scans should still return at least one heap tid before exhaustion"
        );
        assert!(
            observed_tids.contains(&expected_tids[0]),
            "graph-first exhaustion should still include the nearest indexed heap tid"
        );
        assert_eq!(
            observed_tids.len(),
            observed_tids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "graph-first exhaustion should not emit duplicate heap tids before the scan ends"
        );
        assert!(
            observed_tids
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "every emitted heap tid should still belong to the indexed table"
        );
        assert!(
            !exhausted_once,
            "first amgettuple call after exhausting the scan should return false"
        );
        assert!(
            !exhausted_twice,
            "repeated amgettuple calls after exhaustion should remain false"
        );
    }

    #[pg_test]
    fn test_ech_gettuple_rescan_after_exhaustion_restarts_scan() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_exhaustion_rescan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_exhaustion_rescan VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_exhaustion_rescan_idx ON ec_hnsw_gettuple_exhaustion_rescan USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_exhaustion_rescan_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (first_pass, rescanned_tids) = unsafe {
            am::debug_gettuple_rescan_after_exhaustion(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM ec_hnsw_gettuple_exhaustion_rescan
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert!(
            !first_pass.is_empty(),
            "the first graph-first scan should return at least one heap tid before exhaustion"
        );
        assert!(
            first_pass.contains(&expected_tids[0]),
            "the first graph-first scan should include the nearest indexed heap tid before exhaustion"
        );
        assert_eq!(
            first_pass.len(),
            first_pass
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "the first graph-first scan should not emit duplicate heap tids before exhaustion"
        );
        assert!(
            first_pass
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "the first graph-first scan should only emit heap tids from the indexed table"
        );
        assert_eq!(
            rescanned_tids, first_pass,
            "amrescan after exhaustion should restart tuple production from the beginning of the graph-first output"
        );
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw amgettuple only supports forward scan direction")]
    fn test_ech_gettuple_rejects_backward_scan_direction() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_backward_scan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_backward_scan VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_backward_scan_idx ON ec_hnsw_gettuple_backward_scan USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_backward_scan_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        unsafe { am::debug_gettuple_backward_after_rescan(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
    }

    #[pg_test]
    fn test_ech_gettuple_rescan_resets_duplicate_progress() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_duplicate_rescan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_gettuple_duplicate_rescan VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.0, 1.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_duplicate_rescan_idx ON ec_hnsw_gettuple_duplicate_rescan USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_duplicate_rescan_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (first_tid, rescanned_tids) = unsafe {
            am::debug_gettuple_rescan_after_partial(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM ec_hnsw_gettuple_duplicate_rescan
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(
            first_tid, expected_tids[0],
            "the partial scan should consume the first duplicate heap tid before rescan"
        );
        assert!(
            rescanned_tids.contains(&expected_tids[0]) && rescanned_tids.contains(&expected_tids[1]),
            "amrescan should reset duplicate heap-tid progress back to the start of the graph-ordered duplicate drain"
        );
        assert_eq!(
            rescanned_tids.len(),
            rescanned_tids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "rescanning after partial progress should not introduce duplicate heap tid emission"
        );
        assert!(
            rescanned_tids
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "rescanned heap tids should still belong to the indexed table"
        );
    }

    #[pg_test]
    fn test_ech_gettuple_duplicate_scan_drains_selected_duplicates() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_duplicate_multipage (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");

        let dim = 256_usize;
        let bits = 4_u8;
        let payload_len = code_len(dim, bits);
        let duplicate_payload = vec![0x11_u8; payload_len];
        for id in 1..=10 {
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_gettuple_duplicate_multipage VALUES \
                 ({id}, '[dim={dim},bits={bits},seed=42,gamma=0.5]:{payload}'::tqvector)",
                payload = hex::encode(&duplicate_payload),
            ))
            .expect("duplicate insert should succeed");
        }

        for id in 11..=128 {
            let code = (0..payload_len)
                .map(|offset| ((id * 17 + offset as i32) & 0xff) as u8)
                .collect::<Vec<_>>();
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_gettuple_duplicate_multipage VALUES \
                 ({id}, '[dim={dim},bits={bits},seed=42,gamma=0.5]:{payload}'::tqvector)",
                payload = hex::encode(code),
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_duplicate_multipage_idx ON ec_hnsw_gettuple_duplicate_multipage USING ec_hnsw \
             (embedding tqvector_ip_ops) WITH (m = 4, ef_construction = 64)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_duplicate_multipage_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (block_count, _metadata, _data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert!(
            block_count > 2,
            "duplicate-heavy linear scan coverage should span multiple data pages"
        );

        let mut query = vec![0.0_f32; dim];
        query[0] = 1.0;
        query[2] = 0.5;
        query[3] = -1.0;
        let observed_tids = unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query) };
        let expected_tids = Spi::connect(|client| {
            client
                .select(
                    "SELECT
                        split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                        split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number
                     FROM ec_hnsw_gettuple_duplicate_multipage
                     ORDER BY id",
                    None,
                    &[],
                )
                .expect("ctid query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    (
                        u32::try_from(block_number).expect("block number should be non-negative"),
                        u16::try_from(offset_number).expect("offset number should be positive"),
                    )
                })
                .collect::<Vec<_>>()
        });

        let mut observed_tids = observed_tids;
        let mut expected_tids = expected_tids;
        observed_tids.sort_unstable();
        expected_tids.sort_unstable();

        assert!(
            !observed_tids.is_empty(),
            "graph-first scan should still return at least one heap tid from a duplicate-heavy multipage index"
        );
        assert_eq!(
            observed_tids.len(),
            observed_tids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "graph-first duplicate draining should not emit the same heap tid twice"
        );
        assert!(
            observed_tids
                .iter()
                .all(|heap_tid| expected_tids.contains(heap_tid)),
            "every emitted heap tid should still belong to the indexed table"
        );
        assert!(
            observed_tids.len() < expected_tids.len(),
            "this staged A3 execution path should stop after graph-ordered traversal instead of silently falling back to a full linear tail"
        );
    }

    #[pg_test]
    fn test_ech_gettuple_scaffold_returns_false_for_empty_index() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_empty_scaffold (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_empty_scaffold_idx ON ec_hnsw_gettuple_empty_scaffold USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_empty_scaffold_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let found_tuple =
            unsafe { am::debug_gettuple_after_rescan_result(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };
        assert!(
            !found_tuple,
            "amgettuple should report no tuples for a valid rescan on an empty index"
        );
    }

    #[pg_test]
    fn test_ech_empty_scan_stays_false() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_empty_repeated (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_empty_repeated_idx ON ec_hnsw_gettuple_empty_repeated USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_empty_repeated_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (observed_tids, exhausted_once, exhausted_twice) =
            unsafe { am::debug_gettuple_exhaustion_state(index_oid, vec![1.0, 0.0, 0.5, -1.0]) };

        assert!(
            observed_tids.is_empty(),
            "empty indexes should not produce any heap tids before exhaustion"
        );
        assert!(
            !exhausted_once,
            "first amgettuple call on an empty index should return false"
        );
        assert!(
            !exhausted_twice,
            "repeated amgettuple calls on an empty index should remain false"
        );
    }

    #[pg_test]
    fn test_ech_empty_scan_rescan_stays_false() {
        Spi::run(
            "CREATE TABLE ec_hnsw_gettuple_empty_rescan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_gettuple_empty_rescan_idx ON ec_hnsw_gettuple_empty_rescan USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_gettuple_empty_rescan_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (first_pass, rescanned_tids) = unsafe {
            am::debug_gettuple_rescan_after_exhaustion(index_oid, vec![1.0, 0.0, 0.5, -1.0])
        };

        assert!(
            first_pass.is_empty(),
            "empty indexes should still produce no tuples before a repeated rescan"
        );
        assert!(
            rescanned_tids.is_empty(),
            "amrescan on an empty index should continue to return no tuples"
        );
    }
