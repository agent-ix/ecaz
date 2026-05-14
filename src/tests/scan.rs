    #[pg_test]
    fn test_ec_spire_scan_placement_snapshot_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_scan_place_sql (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_scan_place_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_scan_place_sql_idx ON ec_spire_scan_place_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2, nprobe = 1, rerank_width = 10)",
        )
        .expect("ec_spire index creation should succeed");

        let row_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("count should exist");
        let effective_nprobe = Spi::get_one::<i64>(
            "SELECT effective_nprobe FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let leaf_pid_count = Spi::get_one::<i64>(
            "SELECT leaf_pid_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let route_count = Spi::get_one::<i64>(
            "SELECT route_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let prefetched_object_count = Spi::get_one::<i64>(
            "SELECT prefetched_object_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let scanned_pid_count = Spi::get_one::<i64>(
            "SELECT scanned_pid_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let candidate_row_count = Spi::get_one::<i64>(
            "SELECT candidate_row_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let local_store_execution_mode = Spi::get_one::<String>(
            "SELECT local_store_execution_mode FROM \
             ec_spire_index_scan_local_store_execution_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan local-store execution query should succeed")
        .expect("execution diagnostic row should exist");
        let local_store_read_ahead_primitive = Spi::get_one::<String>(
            "SELECT local_store_read_ahead_primitive FROM \
             ec_spire_index_scan_local_store_execution_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan local-store execution query should succeed")
        .expect("execution diagnostic row should exist");
        let local_store_parallelism_next_step = Spi::get_one::<String>(
            "SELECT local_store_parallelism_next_step FROM \
             ec_spire_index_scan_local_store_execution_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan local-store execution query should succeed")
        .expect("execution diagnostic row should exist");
        let delta_pid_count = Spi::get_one::<i64>(
            "SELECT delta_pid_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let primary_candidate_row_count = Spi::get_one::<i64>(
            "SELECT primary_candidate_row_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let candidate_winner_count = Spi::get_one::<i64>(
            "SELECT candidate_winner_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let truncated_candidate_row_count = Spi::get_one::<i64>(
            "SELECT truncated_candidate_row_count FROM \
             ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let routing_row_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM ec_spire_index_scan_routing_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan routing query should succeed")
        .expect("routing count should exist");
        let routing_deduped_route_count = Spi::get_one::<i64>(
            "SELECT deduped_route_count FROM ec_spire_index_scan_routing_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan routing query should succeed")
        .expect("routing diagnostic row should exist");
        let routing_truncation_reason = Spi::get_one::<String>(
            "SELECT truncation_reason FROM ec_spire_index_scan_routing_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan routing query should succeed")
        .expect("routing diagnostic row should exist");
        let routing_adaptive_nprobe_decision = Spi::get_one::<String>(
            "SELECT adaptive_nprobe_decision FROM ec_spire_index_scan_routing_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan routing query should succeed")
        .expect("routing diagnostic row should exist");

        assert_eq!(row_count, 1);
        assert_eq!(effective_nprobe, 1);
        assert_eq!(leaf_pid_count, 1);
        assert_eq!(route_count, 1);
        assert_eq!(prefetched_object_count, 1);
        assert_eq!(scanned_pid_count, 1);
        assert_eq!(delta_pid_count, 0);
        assert!(candidate_row_count > 0);
        assert_eq!(local_store_execution_mode, "sequential_backend");
        assert_eq!(local_store_read_ahead_primitive, "pg18_read_stream");
        assert_eq!(
            local_store_parallelism_next_step,
            "async_or_parallel_store_group_executor"
        );
        assert_eq!(primary_candidate_row_count, 1);
        assert_eq!(candidate_winner_count, 1);
        assert_eq!(truncated_candidate_row_count, 0);
        assert_eq!(routing_row_count, 1);
        assert_eq!(routing_deduped_route_count, 1);
        assert_eq!(routing_truncation_reason, "none");
        assert_eq!(routing_adaptive_nprobe_decision, "disabled");

        Spi::run(
            "INSERT INTO ec_spire_scan_place_sql (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("post-build insert should publish a delta epoch");
        let delta_scanned_pid_count = Spi::get_one::<i64>(
            "SELECT scanned_pid_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let delta_leaf_pid_count = Spi::get_one::<i64>(
            "SELECT leaf_pid_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let delta_route_count = Spi::get_one::<i64>(
            "SELECT route_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let delta_prefetched_object_count = Spi::get_one::<i64>(
            "SELECT prefetched_object_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let delta_delta_pid_count = Spi::get_one::<i64>(
            "SELECT delta_pid_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let delta_candidate_row_count = Spi::get_one::<i64>(
            "SELECT candidate_row_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let delta_leaf_candidate_row_count = Spi::get_one::<i64>(
            "SELECT leaf_candidate_row_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let delta_delta_candidate_row_count = Spi::get_one::<i64>(
            "SELECT delta_candidate_row_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let delete_delta_row_count = Spi::get_one::<i64>(
            "SELECT delete_delta_row_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let dropped_unselected_delta_route_count = Spi::get_one::<i64>(
            "SELECT dropped_unselected_delta_route_count FROM \
             ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");

        assert_eq!(delta_scanned_pid_count, 2);
        assert_eq!(delta_leaf_pid_count, 1);
        assert_eq!(delta_route_count, 2);
        assert_eq!(delta_prefetched_object_count, 2);
        assert_eq!(delta_delta_pid_count, 1);
        assert_eq!(delta_candidate_row_count, 2);
        assert_eq!(delta_leaf_candidate_row_count, 1);
        assert_eq!(delta_delta_candidate_row_count, 1);
        assert_eq!(delete_delta_row_count, 0);
        assert_eq!(dropped_unselected_delta_route_count, 0);

        Spi::run("SET ec_spire.max_candidate_rows = 1")
            .expect("candidate cap override should succeed");
        let capped_candidate_row_count = Spi::get_one::<i64>(
            "SELECT candidate_row_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let capped_truncated_candidate_row_count = Spi::get_one::<i64>(
            "SELECT truncated_candidate_row_count FROM \
             ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        let capped_candidate_winner_count = Spi::get_one::<i64>(
            "SELECT candidate_winner_count FROM ec_spire_index_scan_placement_snapshot(\
             'ec_spire_scan_place_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement query should succeed")
        .expect("diagnostic row should exist");
        Spi::run("RESET ec_spire.max_candidate_rows")
            .expect("candidate cap override reset should succeed");

        assert_eq!(capped_candidate_row_count, 2);
        assert_eq!(capped_truncated_candidate_row_count, 1);
        assert_eq!(capped_candidate_winner_count, 1);
    }
    #[pg_test]
    fn test_ec_spire_multistore_read_overlap_harness_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_multistore_read_harness \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_multistore_read_harness (id, embedding) \
             SELECT i, encode_to_ecvector(\
               ARRAY(SELECT (((i * 17 + d * 31) % 257)::real / 128.0 - 1.0)::real \
                     FROM generate_series(1, 16) AS d), \
               4, 42) \
             FROM generate_series(1, 64) AS i",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_multistore_read_harness_idx \
             ON ec_spire_multistore_read_harness USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH ( \
                 nlists = 8, \
                 nprobe = 8, \
                 rerank_width = 10, \
                 local_store_count = 2, \
                 local_store_tablespaces = 'pg_default,pg_default' \
             )",
        )
        .expect("multi-store ec_spire index creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_multistore_read_harness (id, embedding) \
             SELECT 65, encode_to_ecvector(\
               ARRAY(SELECT (((65 * 17 + d * 31) % 257)::real / 128.0 - 1.0)::real \
                     FROM generate_series(1, 16) AS d), \
               4, 42)",
        )
        .expect("post-build insert should publish a delta epoch");

        let store_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_scan_local_store_read_overlap_harness( \
             'ec_spire_multistore_read_harness_idx'::regclass, \
             ARRAY(SELECT (((7 * 17 + d * 31) % 257)::real / 128.0 - 1.0)::real \
                   FROM generate_series(1, 16) AS d)::real[])",
        )
        .expect("read-overlap harness query should succeed")
        .expect("store count should exist");
        let read_batch_count = Spi::get_one::<i64>(
            "SELECT sum(read_batch_count)::bigint FROM \
             ec_spire_index_scan_local_store_read_overlap_harness( \
             'ec_spire_multistore_read_harness_idx'::regclass, \
             ARRAY(SELECT (((7 * 17 + d * 31) % 257)::real / 128.0 - 1.0)::real \
                   FROM generate_series(1, 16) AS d)::real[])",
        )
        .expect("read-overlap harness query should succeed")
        .expect("read batch count should exist");
        let empty_read_batch_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_scan_local_store_read_overlap_harness( \
             'ec_spire_multistore_read_harness_idx'::regclass, \
             ARRAY(SELECT (((7 * 17 + d * 31) % 257)::real / 128.0 - 1.0)::real \
                   FROM generate_series(1, 16) AS d)::real[]) \
             WHERE read_batch_count = 0",
        )
        .expect("read-overlap harness query should succeed")
        .expect("empty read batch count should exist");
        let route_count = Spi::get_one::<i64>(
            "SELECT sum(route_count)::bigint FROM \
             ec_spire_index_scan_local_store_read_overlap_harness( \
             'ec_spire_multistore_read_harness_idx'::regclass, \
             ARRAY(SELECT (((7 * 17 + d * 31) % 257)::real / 128.0 - 1.0)::real \
                   FROM generate_series(1, 16) AS d)::real[])",
        )
        .expect("read-overlap harness query should succeed")
        .expect("route count should exist");
        let candidate_row_count = Spi::get_one::<i64>(
            "SELECT sum(candidate_row_count)::bigint FROM \
             ec_spire_index_scan_local_store_read_overlap_harness( \
             'ec_spire_multistore_read_harness_idx'::regclass, \
             ARRAY(SELECT (((7 * 17 + d * 31) % 257)::real / 128.0 - 1.0)::real \
                   FROM generate_series(1, 16) AS d)::real[])",
        )
        .expect("read-overlap harness query should succeed")
        .expect("candidate row count should exist");
        let prefetched_object_bytes = Spi::get_one::<i64>(
            "SELECT sum(prefetched_object_bytes)::bigint FROM \
             ec_spire_index_scan_local_store_read_overlap_harness( \
             'ec_spire_multistore_read_harness_idx'::regclass, \
             ARRAY(SELECT (((7 * 17 + d * 31) % 257)::real / 128.0 - 1.0)::real \
                   FROM generate_series(1, 16) AS d)::real[])",
        )
        .expect("read-overlap harness query should succeed")
        .expect("prefetched object bytes should exist");
        let delta_decode_count = Spi::get_one::<i64>(
            "SELECT sum(delta_decode_count)::bigint FROM \
             ec_spire_index_scan_local_store_read_overlap_harness( \
             'ec_spire_multistore_read_harness_idx'::regclass, \
             ARRAY(SELECT (((7 * 17 + d * 31) % 257)::real / 128.0 - 1.0)::real \
                   FROM generate_series(1, 16) AS d)::real[])",
        )
        .expect("read-overlap harness query should succeed")
        .expect("delta decode count should exist");

        assert_eq!(store_count, 2);
        assert_eq!(read_batch_count, store_count);
        assert_eq!(empty_read_batch_count, 0);
        assert!(route_count >= 9);
        assert!(candidate_row_count >= 65);
        assert!(prefetched_object_bytes > 0);
        assert_eq!(delta_decode_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_adaptive_nprobe_routing_snapshot_sql() {
        Spi::run("RESET ec_spire.adaptive_nprobe").expect("adaptive nprobe reset should succeed");
        Spi::run("RESET ec_spire.adaptive_nprobe_score_gap_micros")
            .expect("adaptive nprobe threshold reset should succeed");
        Spi::run(
            "CREATE TABLE ec_spire_adaptive_nprobe_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_adaptive_nprobe_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
             (4, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (5, encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42)), \
             (6, encode_to_ecvector(ARRAY[-0.8, -0.2], 4, 42)), \
             (7, encode_to_ecvector(ARRAY[0.2, 0.8], 4, 42)), \
             (8, encode_to_ecvector(ARRAY[0.2, -0.8], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_adaptive_nprobe_sql_idx \
             ON ec_spire_adaptive_nprobe_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 4, nprobe = 4, rerank_width = 10)",
        )
        .expect("ec_spire index creation should succeed");

        let default_decision = Spi::get_one::<String>(
            "SELECT adaptive_nprobe_decision FROM ec_spire_index_scan_routing_snapshot(\
             'ec_spire_adaptive_nprobe_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("default routing snapshot should succeed")
        .expect("routing diagnostic row should exist");
        Spi::run("SET ec_spire.adaptive_nprobe = on")
            .expect("adaptive nprobe override should succeed");
        Spi::run("SET ec_spire.adaptive_nprobe_score_gap_micros = 0")
            .expect("adaptive nprobe threshold override should succeed");
        let adaptive_effective_nprobe = Spi::get_one::<i64>(
            "SELECT effective_nprobe FROM ec_spire_index_scan_routing_snapshot(\
             'ec_spire_adaptive_nprobe_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("adaptive routing snapshot should succeed")
        .expect("routing diagnostic row should exist");
        let adaptive_effective_nprobe_source = Spi::get_one::<String>(
            "SELECT effective_nprobe_source FROM ec_spire_index_scan_routing_snapshot(\
             'ec_spire_adaptive_nprobe_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("adaptive routing snapshot should succeed")
        .expect("routing diagnostic row should exist");
        let adaptive_decision = Spi::get_one::<String>(
            "SELECT adaptive_nprobe_decision FROM ec_spire_index_scan_routing_snapshot(\
             'ec_spire_adaptive_nprobe_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("adaptive routing snapshot should succeed")
        .expect("routing diagnostic row should exist");
        Spi::run("RESET ec_spire.adaptive_nprobe").expect("adaptive nprobe reset should succeed");
        Spi::run("RESET ec_spire.adaptive_nprobe_score_gap_micros")
            .expect("adaptive nprobe threshold reset should succeed");

        assert_eq!(default_decision, "disabled");
        assert_eq!(adaptive_effective_nprobe, 2);
        assert_eq!(adaptive_effective_nprobe_source, "adaptive");
        assert_eq!(adaptive_decision, "reduced_score_gap");
    }

    #[pg_test]
    fn test_ec_spire_scan_pipeline_snapshot_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_scan_pipeline_sql (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_scan_pipeline_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_scan_pipeline_sql_idx \
             ON ec_spire_scan_pipeline_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2, nprobe = 1, rerank_width = 10)",
        )
        .expect("ec_spire index creation should succeed");

        let snapshot_from = "FROM ec_spire_index_scan_pipeline_snapshot(\
             'ec_spire_scan_pipeline_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])";
        let step_names = Spi::get_one::<Vec<String>>(&format!(
            "SELECT array_agg(step_name ORDER BY step_ordinal) {snapshot_from}"
        ))
        .expect("scan pipeline step query should succeed")
        .expect("scan pipeline steps should exist");
        let routing_route_count = Spi::get_one::<i64>(&format!(
            "SELECT route_count {snapshot_from} WHERE step_name = 'routing'"
        ))
        .expect("scan pipeline routing query should succeed")
        .expect("routing step should exist");
        let placement_item_count = Spi::get_one::<i64>(&format!(
            "SELECT item_count {snapshot_from} WHERE step_name = 'placement'"
        ))
        .expect("scan pipeline placement query should succeed")
        .expect("placement step should exist");
        let prefetch_item_count = Spi::get_one::<i64>(&format!(
            "SELECT item_count {snapshot_from} WHERE step_name = 'prefetch'"
        ))
        .expect("scan pipeline prefetch query should succeed")
        .expect("prefetch step should exist");
        let candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT candidate_count {snapshot_from} WHERE step_name = 'candidates'"
        ))
        .expect("scan pipeline candidate query should succeed")
        .expect("candidate step should exist");
        let heap_rerank_row_count = Spi::get_one::<i64>(&format!(
            "SELECT heap_rerank_row_count {snapshot_from} WHERE step_name = 'heap_rerank'"
        ))
        .expect("scan pipeline heap rerank query should succeed")
        .expect("heap rerank step should exist");
        let remote_fanout_count = Spi::get_one::<i64>(&format!(
            "SELECT remote_fanout_count {snapshot_from} WHERE step_name = 'remote_fanout'"
        ))
        .expect("scan pipeline remote fanout query should succeed")
        .expect("remote fanout step should exist");

        assert_eq!(
            step_names,
            vec![
                "routing".to_owned(),
                "placement".to_owned(),
                "prefetch".to_owned(),
                "candidates".to_owned(),
                "heap_rerank".to_owned(),
                "remote_fanout".to_owned()
            ]
        );
        assert_eq!(routing_route_count, 1);
        assert_eq!(placement_item_count, 1);
        assert_eq!(prefetch_item_count, 1);
        assert!(candidate_count > 0);
        assert_eq!(heap_rerank_row_count, 1);
        assert_eq!(remote_fanout_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_root_routing_snapshot_sql() {
        Spi::run("CREATE TABLE ec_spire_route_sql (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_route_empty_idx ON ec_spire_route_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");
        let empty_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_root_routing_snapshot('ec_spire_route_empty_idx'::regclass)",
        )
        .expect("routing snapshot query should succeed")
        .expect("count should exist");
        assert_eq!(empty_rows, 0);

        Spi::run("DROP INDEX ec_spire_route_empty_idx").expect("drop index should succeed");
        Spi::run(
            "INSERT INTO ec_spire_route_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_route_sql_idx ON ec_spire_route_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let row_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_root_routing_snapshot('ec_spire_route_sql_idx'::regclass)",
        )
        .expect("routing snapshot query should succeed")
        .expect("count should exist");
        let root_child_count = Spi::get_one::<i64>(
            "SELECT max(root_child_count) FROM \
             ec_spire_index_root_routing_snapshot('ec_spire_route_sql_idx'::regclass)",
        )
        .expect("routing snapshot query should succeed")
        .expect("routing row should exist");
        let centroid_dimensions = Spi::get_one::<i32>(
            "SELECT max(centroid_dimensions) FROM \
             ec_spire_index_root_routing_snapshot('ec_spire_route_sql_idx'::regclass)",
        )
        .expect("routing snapshot query should succeed")
        .expect("routing row should exist");
        let leaf_children = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_root_routing_snapshot('ec_spire_route_sql_idx'::regclass) \
             WHERE child_kind = 'leaf'",
        )
        .expect("routing snapshot query should succeed")
        .expect("count should exist");
        let assignment_count = Spi::get_one::<i64>(
            "SELECT coalesce(sum(child_assignment_count)::bigint, 0) FROM \
             ec_spire_index_root_routing_snapshot('ec_spire_route_sql_idx'::regclass)",
        )
        .expect("routing snapshot query should succeed")
        .expect("sum should exist");
        let parent_links_match = Spi::get_one::<bool>(
            "SELECT bool_and(child_parent_pid = root_pid) FROM \
             ec_spire_index_root_routing_snapshot('ec_spire_route_sql_idx'::regclass)",
        )
        .expect("routing snapshot query should succeed")
        .expect("bool aggregate should exist");
        let child_store_relid_count = Spi::get_one::<i64>(
            "SELECT count(DISTINCT child_store_relid) FROM \
             ec_spire_index_root_routing_snapshot('ec_spire_route_sql_idx'::regclass)",
        )
        .expect("routing snapshot query should succeed")
        .expect("count should exist");

        assert_eq!(row_count, 2);
        assert_eq!(root_child_count, 2);
        assert_eq!(centroid_dimensions, 2);
        assert_eq!(leaf_children, 2);
        assert_eq!(assignment_count, 2);
        assert!(parent_links_match);
        assert_eq!(child_store_relid_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_routing_centroid_snapshot_sql() {
        Spi::run("CREATE TABLE ec_spire_centroid_sql (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_centroid_empty_idx ON ec_spire_centroid_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");
        let empty_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_routing_centroid_snapshot('ec_spire_centroid_empty_idx'::regclass)",
        )
        .expect("routing centroid snapshot query should succeed")
        .expect("count should exist");
        assert_eq!(empty_rows, 0);

        Spi::run("DROP INDEX ec_spire_centroid_empty_idx").expect("drop index should succeed");
        Spi::run(
            "INSERT INTO ec_spire_centroid_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (4, encode_to_ecvector(ARRAY[-0.8, 0.2], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_centroid_sql_idx ON ec_spire_centroid_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 4, recursive_fanout = 2)",
        )
        .expect("recursive ec_spire index creation should succeed");

        let row_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_routing_centroid_snapshot('ec_spire_centroid_sql_idx'::regclass)",
        )
        .expect("routing centroid snapshot query should succeed")
        .expect("count should exist");
        let root_parent_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_routing_centroid_snapshot('ec_spire_centroid_sql_idx'::regclass) \
             WHERE parent_kind = 'root'",
        )
        .expect("routing centroid snapshot query should succeed")
        .expect("count should exist");
        let internal_parent_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_routing_centroid_snapshot('ec_spire_centroid_sql_idx'::regclass) \
             WHERE parent_kind = 'internal'",
        )
        .expect("routing centroid snapshot query should succeed")
        .expect("count should exist");
        let internal_child_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_routing_centroid_snapshot('ec_spire_centroid_sql_idx'::regclass) \
             WHERE child_kind = 'internal'",
        )
        .expect("routing centroid snapshot query should succeed")
        .expect("count should exist");
        let leaf_child_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_routing_centroid_snapshot('ec_spire_centroid_sql_idx'::regclass) \
             WHERE child_kind = 'leaf'",
        )
        .expect("routing centroid snapshot query should succeed")
        .expect("count should exist");
        let max_parent_level = Spi::get_one::<i32>(
            "SELECT max(parent_level) FROM \
             ec_spire_index_routing_centroid_snapshot('ec_spire_centroid_sql_idx'::regclass)",
        )
        .expect("routing centroid snapshot query should succeed")
        .expect("max parent level should exist");
        let min_child_level = Spi::get_one::<i32>(
            "SELECT min(child_level) FROM \
             ec_spire_index_routing_centroid_snapshot('ec_spire_centroid_sql_idx'::regclass)",
        )
        .expect("routing centroid snapshot query should succeed")
        .expect("min child level should exist");
        let centroid_lengths_match = Spi::get_one::<bool>(
            "SELECT bool_and(cardinality(centroid) = centroid_dimensions) FROM \
             ec_spire_index_routing_centroid_snapshot('ec_spire_centroid_sql_idx'::regclass)",
        )
        .expect("routing centroid snapshot query should succeed")
        .expect("centroid length aggregate should exist");
        let parent_links_match = Spi::get_one::<bool>(
            "SELECT bool_and(child_parent_pid = parent_pid) FROM \
             ec_spire_index_routing_centroid_snapshot('ec_spire_centroid_sql_idx'::regclass)",
        )
        .expect("routing centroid snapshot query should succeed")
        .expect("parent link aggregate should exist");

        assert_eq!(row_count, 6);
        assert_eq!(root_parent_rows, 2);
        assert_eq!(internal_parent_rows, 4);
        assert_eq!(internal_child_rows, 2);
        assert_eq!(leaf_child_rows, 4);
        assert_eq!(max_parent_level, 2);
        assert_eq!(min_child_level, 0);
        assert!(centroid_lengths_match);
        assert!(parent_links_match);
    }

    #[pg_test]
    fn test_ec_spire_classify_centroid_sql() {
        Spi::run("CREATE TABLE ec_spire_classify_sql (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_classify_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (4, encode_to_ecvector(ARRAY[-0.8, 0.2], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_classify_sql_idx ON ec_spire_classify_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");
        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_spire_classify_sql_idx'::regclass::oid")
                .expect("index oid query should succeed")
                .expect("index oid should exist");
        let expected_centroid_id = Spi::get_one::<i64>(
            "SELECT child_pid \
               FROM ec_spire_index_routing_centroid_snapshot(\
                    'ec_spire_classify_sql_idx'::regclass) r \
               CROSS JOIN LATERAL ( \
                    SELECT sum(q.value * c.value)::real AS score \
                      FROM unnest(ARRAY[1.0, 0.0]::real[]) WITH ORDINALITY q(value, ord) \
                      JOIN unnest(r.centroid) WITH ORDINALITY c(value, ord) USING (ord) \
               ) scored \
              WHERE parent_kind = 'root' AND child_kind = 'leaf' \
              ORDER BY scored.score DESC, centroid_index, child_pid \
              LIMIT 1",
        )
        .expect("expected centroid query should succeed")
        .expect("expected centroid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_classify_sql_idx'::regclass)",
        )
        .expect("active epoch query should succeed")
        .expect("active epoch should exist");

        unsafe {
            am::debug_spire_rewrite_placement_node(index_oid, expected_centroid_id as u64, 7)
        };

        let classification = Spi::get_one::<String>(
            "SELECT node_id::text || ',' || centroid_id::text || ',' || epoch::text \
               FROM ec_spire_classify_centroid(\
                    ARRAY[1.0, 0.0]::real[], 'ec_spire_classify_sql_idx'::regclass)",
        )
        .expect("classification query should succeed")
        .expect("classification should exist");

        assert_eq!(
            classification,
            format!("7,{expected_centroid_id},{active_epoch}")
        );
    }

    #[pg_test]
    fn test_ec_spire_classify_centroid_recursive_sql() {
        Spi::run("CREATE TABLE ec_spire_classify_recursive_sql (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_classify_recursive_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (4, encode_to_ecvector(ARRAY[-0.8, 0.2], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_classify_recursive_sql_idx \
             ON ec_spire_classify_recursive_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 4, recursive_fanout = 2)",
        )
        .expect("recursive ec_spire index creation should succeed");
        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_classify_recursive_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let max_parent_level = Spi::get_one::<i32>(
            "SELECT max(parent_level) FROM \
             ec_spire_index_routing_centroid_snapshot('ec_spire_classify_recursive_sql_idx'::regclass)",
        )
        .expect("routing centroid max level query should succeed")
        .expect("routing centroid max level should exist");
        let expected_leaf_pid = Spi::get_one::<i64>(
            "WITH centroid_scores AS ( \
                 SELECT r.*, scored.score \
                   FROM ec_spire_index_routing_centroid_snapshot(\
                        'ec_spire_classify_recursive_sql_idx'::regclass) r \
                   CROSS JOIN LATERAL ( \
                        SELECT sum(q.value * c.value)::real AS score \
                          FROM unnest(ARRAY[1.0, 0.0]::real[]) WITH ORDINALITY q(value, ord) \
                          JOIN unnest(r.centroid) WITH ORDINALITY c(value, ord) USING (ord) \
                   ) scored \
             ), root_choice AS ( \
                 SELECT child_pid \
                   FROM centroid_scores \
                  WHERE parent_kind = 'root' AND child_kind = 'internal' \
                  ORDER BY score DESC, centroid_index, child_pid \
                  LIMIT 1 \
             ) \
             SELECT child_pid \
               FROM centroid_scores \
              WHERE parent_pid = (SELECT child_pid FROM root_choice) \
                AND child_kind = 'leaf' \
              ORDER BY score DESC, centroid_index, child_pid \
              LIMIT 1",
        )
        .expect("expected recursive leaf query should succeed")
        .expect("expected recursive leaf should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_classify_recursive_sql_idx'::regclass)",
        )
        .expect("active epoch query should succeed")
        .expect("active epoch should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, expected_leaf_pid as u64, 9) };

        let classification = Spi::get_one::<String>(
            "SELECT node_id::text || ',' || centroid_id::text || ',' || epoch::text \
               FROM ec_spire_classify_centroid(\
                    ARRAY[1.0, 0.0]::real[], 'ec_spire_classify_recursive_sql_idx'::regclass)",
        )
        .expect("recursive classification query should succeed")
        .expect("recursive classification should exist");

        assert_eq!(max_parent_level, 2);
        assert_eq!(
            classification,
            format!("9,{expected_leaf_pid},{active_epoch}")
        );
    }

    #[pg_test]
    fn test_ec_spire_empty_build_scan_no_rows() {
        Spi::run("CREATE TABLE ec_spire_empty_scan (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_empty_scan_idx ON ec_spire_empty_scan USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM ( \
                SELECT id FROM ec_spire_empty_scan \
                ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
                LIMIT 1 \
             ) AS ordered_empty",
        )
        .expect("ordered empty ec_spire query should succeed")
        .expect("count should not be NULL");

        assert_eq!(rows, 0);
    }

    #[pg_test]
    fn test_ec_spire_empty_pq_fastscan_build_scan_no_rows() {
        Spi::run("CREATE TABLE ec_spire_empty_pq_scan (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_empty_pq_scan_idx ON ec_spire_empty_pq_scan USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (storage_format = 'pq_fastscan')",
        )
        .expect("empty pq_fastscan ec_spire index creation should succeed");

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM ( \
                SELECT id FROM ec_spire_empty_pq_scan \
                ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
                LIMIT 1 \
             ) AS ordered_empty",
        )
        .expect("ordered empty pq_fastscan ec_spire query should succeed")
        .expect("count should not be NULL");

        assert_eq!(rows, 0);
    }
    #[pg_test]
    fn test_ec_spire_flat_recursive_same_candidate() {
        Spi::run("CREATE TABLE ec_spire_flat_compare (id bigint primary key, embedding ecvector)")
            .expect("flat comparison table creation should succeed");
        Spi::run(
            "CREATE TABLE ec_spire_recursive_compare (id bigint primary key, embedding ecvector)",
        )
        .expect("recursive comparison table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_flat_compare (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (4, encode_to_ecvector(ARRAY[-0.8, 0.2], 4, 42))",
        )
        .expect("flat comparison insert should succeed");
        Spi::run(
            "INSERT INTO ec_spire_recursive_compare (id, embedding) \
             SELECT id, embedding FROM ec_spire_flat_compare",
        )
        .expect("recursive comparison insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_flat_compare_idx \
             ON ec_spire_flat_compare USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 4)",
        )
        .expect("flat comparison ec_spire index creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_recursive_compare_idx \
             ON ec_spire_recursive_compare USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 4, recursive_fanout = 2)",
        )
        .expect("recursive comparison ec_spire index creation should succeed");

        let flat_internal_count = Spi::get_one::<i64>(
            "SELECT internal_routing_object_count FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_flat_compare_idx'::regclass)",
        )
        .expect("flat hierarchy snapshot query should succeed")
        .expect("flat internal count should exist");
        let recursive_internal_count = Spi::get_one::<i64>(
            "SELECT internal_routing_object_count FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_recursive_compare_idx'::regclass)",
        )
        .expect("recursive hierarchy snapshot query should succeed")
        .expect("recursive internal count should exist");
        let flat_depth = Spi::get_one::<i32>(
            "SELECT hierarchy_depth FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_flat_compare_idx'::regclass)",
        )
        .expect("flat hierarchy snapshot query should succeed")
        .expect("flat hierarchy depth should exist");
        let recursive_depth = Spi::get_one::<i32>(
            "SELECT hierarchy_depth FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_recursive_compare_idx'::regclass)",
        )
        .expect("recursive hierarchy snapshot query should succeed")
        .expect("recursive hierarchy depth should exist");
        let flat_supported = Spi::get_one::<bool>(
            "SELECT recursive_routing_supported FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_flat_compare_idx'::regclass)",
        )
        .expect("flat hierarchy snapshot query should succeed")
        .expect("flat recursive support flag should exist");
        let recursive_supported = Spi::get_one::<bool>(
            "SELECT recursive_routing_supported FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_recursive_compare_idx'::regclass)",
        )
        .expect("recursive hierarchy snapshot query should succeed")
        .expect("recursive support flag should exist");
        let flat_root_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_root_routing_snapshot('ec_spire_flat_compare_idx'::regclass)",
        )
        .expect("flat root routing snapshot query should succeed")
        .expect("flat root row count should exist");
        let recursive_root_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_root_routing_snapshot('ec_spire_recursive_compare_idx'::regclass)",
        )
        .expect("recursive root routing snapshot query should succeed")
        .expect("recursive root row count should exist");

        assert_eq!(flat_internal_count, 0);
        assert_eq!(recursive_internal_count, 2);
        assert_eq!(flat_depth, 1);
        assert_eq!(recursive_depth, 2);
        assert!(!flat_supported);
        assert!(recursive_supported);
        assert_eq!(flat_root_rows, 4);
        assert_eq!(recursive_root_rows, 2);

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        fn top_ids(table_name: &str, query: &str, limit: i64) -> Vec<i64> {
            Spi::get_one::<Vec<i64>>(&format!(
                "SELECT array_agg(id ORDER BY id) FROM (\
                 SELECT id FROM {table_name} \
                 ORDER BY embedding <#> {query}::real[] \
                 LIMIT {limit}) ids"
            ))
            .expect("ordered comparison ec_spire query should succeed")
            .expect("comparison query should return rows")
        }

        for (query, expected_top2_ids) in [
            ("ARRAY[1.0, 0.0]", vec![1, 2]),
            ("ARRAY[0.8, 0.2]", vec![1, 2]),
            ("ARRAY[-1.0, 0.0]", vec![3, 4]),
            ("ARRAY[-0.8, 0.2]", vec![3, 4]),
        ] {
            let flat_top2_ids = top_ids("ec_spire_flat_compare", query, 2);
            let recursive_top2_ids = top_ids("ec_spire_recursive_compare", query, 2);
            assert_eq!(flat_top2_ids, expected_top2_ids);
            assert_eq!(recursive_top2_ids, flat_top2_ids);

            let flat_top1_id = top_ids("ec_spire_flat_compare", query, 1);
            let recursive_top1_id = top_ids("ec_spire_recursive_compare", query, 1);
            assert_eq!(recursive_top1_id, flat_top1_id);
        }
    }
