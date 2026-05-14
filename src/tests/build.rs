    #[pg_test]
    fn test_ec_spire_boundary_replica_build_writes_and_dedupes_scan() {
        Spi::run(
            "CREATE TABLE ec_spire_boundary_replica_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_boundary_replica_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.9, 0.1], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_boundary_replica_sql_idx \
             ON ec_spire_boundary_replica_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH ( \
                 nlists = 3, \
                 nprobe = 3, \
                 boundary_replica_count = 1 \
             )",
        )
        .expect("boundary replica ec_spire index creation should succeed");

        let leaf_assignment_count = Spi::get_one::<i64>(
            "SELECT leaf_assignment_count FROM \
             ec_spire_index_active_snapshot_diagnostics(\
                 'ec_spire_boundary_replica_sql_idx'::regclass)",
        )
        .expect("active diagnostics should succeed")
        .expect("diagnostics row should exist");
        let base_assignment_count = Spi::get_one::<i64>(
            "SELECT coalesce(sum(base_assignment_count), 0)::bigint FROM \
             ec_spire_index_leaf_snapshot('ec_spire_boundary_replica_sql_idx'::regclass)",
        )
        .expect("leaf snapshot should succeed")
        .expect("sum row should exist");
        let base_primary_assignment_count = Spi::get_one::<i64>(
            "SELECT coalesce(sum(base_primary_assignment_count), 0)::bigint FROM \
             ec_spire_index_leaf_snapshot('ec_spire_boundary_replica_sql_idx'::regclass)",
        )
        .expect("leaf snapshot should succeed")
        .expect("sum row should exist");
        let base_boundary_replica_assignment_count = Spi::get_one::<i64>(
            "SELECT coalesce(sum(base_boundary_replica_assignment_count), 0)::bigint FROM \
             ec_spire_index_leaf_snapshot('ec_spire_boundary_replica_sql_idx'::regclass)",
        )
        .expect("leaf snapshot should succeed")
        .expect("sum row should exist");
        let scan_dedupe_mode = Spi::get_one::<String>(
            "SELECT scan_dedupe_mode FROM \
             ec_spire_index_options_snapshot('ec_spire_boundary_replica_sql_idx'::regclass)",
        )
        .expect("options snapshot should succeed")
        .expect("options row should exist");

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let returned_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM ( \
               SELECT id FROM ec_spire_boundary_replica_sql \
               ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
               LIMIT 10 \
             ) AS ranked",
        )
        .expect("ordered boundary replica scan should succeed")
        .expect("count row should exist");
        let distinct_rows = Spi::get_one::<i64>(
            "SELECT count(DISTINCT id) FROM ( \
               SELECT id FROM ec_spire_boundary_replica_sql \
               ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
               LIMIT 10 \
             ) AS ranked",
        )
        .expect("ordered boundary replica scan should succeed")
        .expect("count row should exist");

        assert_eq!(leaf_assignment_count, 6);
        assert_eq!(base_assignment_count, 6);
        assert_eq!(base_primary_assignment_count, 3);
        assert_eq!(base_boundary_replica_assignment_count, 3);
        assert_eq!(scan_dedupe_mode, "vec_id");
        assert_eq!(returned_rows, 3);
        assert_eq!(distinct_rows, 3);

        Spi::run(
            "INSERT INTO ec_spire_boundary_replica_sql (id, embedding) VALUES \
             (4, encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42))",
        )
        .expect("post-build insert should succeed");
        let delta_insert_assignment_count = Spi::get_one::<i64>(
            "SELECT coalesce(sum(delta_insert_assignment_count), 0)::bigint FROM \
             ec_spire_index_leaf_snapshot('ec_spire_boundary_replica_sql_idx'::regclass)",
        )
        .expect("leaf snapshot should succeed")
        .expect("sum row should exist");
        let delta_boundary_replica_insert_assignment_count = Spi::get_one::<i64>(
            "SELECT coalesce(sum(delta_boundary_replica_insert_assignment_count), 0)::bigint FROM \
             ec_spire_index_leaf_snapshot('ec_spire_boundary_replica_sql_idx'::regclass)",
        )
        .expect("leaf snapshot should succeed")
        .expect("sum row should exist");
        let effective_boundary_replica_assignment_count = Spi::get_one::<i64>(
            "SELECT coalesce(sum(effective_boundary_replica_assignment_count), 0)::bigint FROM \
             ec_spire_index_leaf_snapshot('ec_spire_boundary_replica_sql_idx'::regclass)",
        )
        .expect("leaf snapshot should succeed")
        .expect("sum row should exist");
        let post_insert_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM ( \
               SELECT id FROM ec_spire_boundary_replica_sql \
               ORDER BY embedding <#> ARRAY[0.8, 0.2]::real[] \
               LIMIT 10 \
             ) AS ranked",
        )
        .expect("ordered post-insert boundary replica scan should succeed")
        .expect("count row should exist");
        let post_insert_distinct_rows = Spi::get_one::<i64>(
            "SELECT count(DISTINCT id) FROM ( \
               SELECT id FROM ec_spire_boundary_replica_sql \
               ORDER BY embedding <#> ARRAY[0.8, 0.2]::real[] \
               LIMIT 10 \
             ) AS ranked",
        )
        .expect("ordered post-insert boundary replica scan should succeed")
        .expect("count row should exist");

        assert_eq!(delta_insert_assignment_count, 2);
        assert_eq!(delta_boundary_replica_insert_assignment_count, 1);
        assert_eq!(effective_boundary_replica_assignment_count, 4);
        assert_eq!(post_insert_rows, 4);
        assert_eq!(post_insert_distinct_rows, 4);
    }

    #[pg_test]
    fn test_ec_spire_recursive_boundary_replica_build_dedupes() {
        Spi::run(
            "CREATE TABLE ec_spire_recursive_boundary_replica_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_recursive_boundary_replica_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (4, encode_to_ecvector(ARRAY[-0.8, 0.2], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_recursive_boundary_replica_sql_idx \
             ON ec_spire_recursive_boundary_replica_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH ( \
                 nlists = 4, \
                 nprobe = 4, \
                 recursive_fanout = 2, \
                 boundary_replica_count = 1 \
             )",
        )
        .expect("recursive boundary replica ec_spire index creation should succeed");

        let recursive_supported = Spi::get_one::<bool>(
            "SELECT recursive_routing_supported FROM \
             ec_spire_index_hierarchy_snapshot( \
                 'ec_spire_recursive_boundary_replica_sql_idx'::regclass \
             )",
        )
        .expect("hierarchy snapshot should succeed")
        .expect("hierarchy row should exist");
        let base_assignment_count = Spi::get_one::<i64>(
            "SELECT coalesce(sum(base_assignment_count), 0)::bigint FROM \
             ec_spire_index_leaf_snapshot( \
                 'ec_spire_recursive_boundary_replica_sql_idx'::regclass \
             )",
        )
        .expect("leaf snapshot should succeed")
        .expect("sum row should exist");

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let returned_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM ( \
               SELECT id FROM ec_spire_recursive_boundary_replica_sql \
               ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
               LIMIT 10 \
             ) AS ranked",
        )
        .expect("ordered recursive boundary replica scan should succeed")
        .expect("count row should exist");
        let distinct_rows = Spi::get_one::<i64>(
            "SELECT count(DISTINCT id) FROM ( \
               SELECT id FROM ec_spire_recursive_boundary_replica_sql \
               ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
               LIMIT 10 \
             ) AS ranked",
        )
        .expect("ordered recursive boundary replica scan should succeed")
        .expect("count row should exist");

        assert!(recursive_supported);
        assert_eq!(base_assignment_count, 8);
        assert_eq!(returned_rows, 2);
        assert_eq!(distinct_rows, 2);
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_spire PQ-FastScan encoding requires a persisted grouped-PQ model"
    )]
    fn test_ec_spire_pq_fastscan_populated_build_reports_deferral() {
        Spi::run(
            "CREATE TABLE ec_spire_pq_build_deferral \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_pq_build_deferral (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX ec_spire_pq_build_deferral_idx ON ec_spire_pq_build_deferral \
             USING ec_spire (embedding ecvector_spire_ip_ops) \
             WITH (storage_format = 'pq_fastscan')",
        )
        .expect("populated pq_fastscan SPIRE build should report deferral");
    }

    #[pg_test]
    fn test_ec_spire_populated_build_publishes_root_control() {
        Spi::run(
            "CREATE TABLE ec_spire_populated_build (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_populated_build (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_populated_build_idx ON ec_spire_populated_build USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let index_oid = index_oid("ec_spire_populated_build_idx");
        let (active_epoch, next_pid, next_local_vec_seq) =
            unsafe { am::debug_spire_root_control(index_oid) };
        let diagnostics = unsafe { am::debug_spire_active_snapshot_diagnostics(index_oid) };

        assert_eq!(active_epoch, 1);
        assert_eq!(next_pid, 4);
        assert_eq!(next_local_vec_seq, 4);
        assert_eq!(diagnostics.epoch, 1);
        assert_eq!(diagnostics.object_count, 3);
        assert_eq!(diagnostics.placement_count, 3);
        assert_eq!(diagnostics.local_store_count, 1);
        assert_eq!(diagnostics.available_placement_count, 3);
        assert_eq!(diagnostics.root_object_count, 1);
        assert_eq!(diagnostics.leaf_object_count, 2);
        assert_eq!(diagnostics.routing_child_count, 2);
        assert_eq!(diagnostics.leaf_assignment_count, 3);
        assert!(diagnostics.available_object_bytes > 0);

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let first_id = Spi::get_one::<i64>(
            "SELECT id FROM ec_spire_populated_build \
             ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
             LIMIT 1",
        )
        .expect("ordered populated ec_spire query should succeed")
        .expect("query should return a row");
        assert_eq!(first_id, 1);
    }

    #[pg_test]
    fn test_ec_spire_populated_build_hash_routes_logical_store_set() {
        Spi::run(
            "CREATE TABLE ec_spire_logical_store_build \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_logical_store_build (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (4, encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_logical_store_build_idx \
             ON ec_spire_logical_store_build USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH ( \
                 nlists = 2, \
                 local_store_count = 2, \
                 local_store_tablespaces = 'pg_default,pg_default' \
             )",
        )
        .expect("multi-store logical baseline build should succeed");

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let first_id = Spi::get_one::<i64>(
            "SELECT id FROM ec_spire_logical_store_build \
             ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
             LIMIT 1",
        )
        .expect("ordered populated ec_spire query should succeed")
        .expect("query should return a row");
        let placed_store_count = Spi::get_one::<i64>(
            "SELECT count(DISTINCT local_store_id) FROM \
             ec_spire_index_object_snapshot('ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("object snapshot should succeed")
        .expect("count should exist");
        let placement_store_count = Spi::get_one::<i64>(
            "SELECT count(DISTINCT local_store_id) FROM \
             ec_spire_index_placement_snapshot('ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("placement snapshot should succeed")
        .expect("count should exist");
        let placement_store_relid_count = Spi::get_one::<i64>(
            "SELECT count(DISTINCT store_relid) FROM \
             ec_spire_index_placement_snapshot('ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("placement snapshot should succeed")
        .expect("count should exist");
        let auxiliary_store_relation_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM pg_class \
             WHERE relname LIKE 'ec_spire_store_%' \
             AND relkind = 'r'",
        )
        .expect("store relation query should succeed")
        .expect("count should exist");
        let autovacuum_disabled_auxiliary_relation_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM pg_class \
             WHERE relname LIKE 'ec_spire_store_%' \
             AND relkind = 'r' \
             AND reloptions @> ARRAY['autovacuum_enabled=false']::text[]",
        )
        .expect("store relation reloptions query should succeed")
        .expect("count should exist");
        let active_diag_store_count = Spi::get_one::<i64>(
            "SELECT local_store_count FROM \
             ec_spire_index_active_snapshot_diagnostics(\
                 'ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("active diagnostics should succeed")
        .expect("diagnostics row should exist");
        let options_active_leaf_count = Spi::get_one::<i64>(
            "SELECT active_leaf_count FROM \
             ec_spire_index_options_snapshot('ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("options snapshot should succeed")
        .expect("options row should exist");
        let scan_sanity_active_leaf_count = Spi::get_one::<i64>(
            "SELECT active_leaf_count FROM \
             ec_spire_index_scan_sanity_snapshot('ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("scan sanity snapshot should succeed")
        .expect("scan sanity row should exist");
        let active_referenced_tuple_count = Spi::get_one::<i64>(
            "SELECT active_referenced_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot(\
                 'ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("relation storage snapshot should succeed")
        .expect("storage row should exist");
        let storage_relation_block_count = Spi::get_one::<i64>(
            "SELECT relation_block_count FROM \
             ec_spire_index_relation_storage_snapshot(\
                 'ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("relation storage snapshot should succeed")
        .expect("storage row should exist");
        let storage_object_tuple_count = Spi::get_one::<i64>(
            "SELECT relation_object_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot(\
                 'ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("relation storage snapshot should succeed")
        .expect("storage row should exist");
        let storage_cleanup_candidate_count = Spi::get_one::<i64>(
            "SELECT cleanup_candidate_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot(\
                 'ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("relation storage snapshot should succeed")
        .expect("storage row should exist");
        let candidate_count = Spi::get_one::<i64>(
            "SELECT coalesce(sum(candidate_row_count), 0)::bigint FROM \
             ec_spire_index_scan_placement_snapshot( \
                 'ec_spire_logical_store_build_idx'::regclass, \
                 ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement snapshot should succeed")
        .expect("sum should exist");

        assert_eq!(first_id, 1);
        assert_eq!(placed_store_count, 2);
        assert_eq!(placement_store_count, 2);
        assert_eq!(placement_store_relid_count, 2);
        assert_eq!(auxiliary_store_relation_count, 2);
        assert_eq!(autovacuum_disabled_auxiliary_relation_count, 2);
        assert_eq!(active_diag_store_count, 2);
        assert_eq!(options_active_leaf_count, 2);
        assert_eq!(scan_sanity_active_leaf_count, 2);
        assert!(active_referenced_tuple_count > 0);
        assert!(storage_relation_block_count >= auxiliary_store_relation_count + 1);
        assert_eq!(active_referenced_tuple_count, storage_object_tuple_count);
        assert_eq!(storage_cleanup_candidate_count, 0);
        assert!(candidate_count >= 1);

        Spi::run(
            "INSERT INTO ec_spire_logical_store_build (id, embedding) VALUES \
             (5, encode_to_ecvector(ARRAY[0.9, 0.1], 4, 42))",
        )
        .expect("multi-store post-build insert should succeed");
        let post_insert_delta_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_object_snapshot('ec_spire_logical_store_build_idx'::regclass) \
             WHERE object_kind = 'delta'",
        )
        .expect("object snapshot should succeed")
        .expect("delta count should exist");
        let post_insert_store_relid_count = Spi::get_one::<i64>(
            "SELECT count(DISTINCT store_relid) FROM \
             ec_spire_index_placement_snapshot('ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("placement snapshot should succeed")
        .expect("count should exist");
        let post_insert_rows_returned = Spi::get_one::<i64>(
            "SELECT count(*) FROM ( \
                 SELECT id FROM ec_spire_logical_store_build \
                 ORDER BY embedding <#> ARRAY[0.9, 0.1]::real[] \
                 LIMIT 5 \
             ) ranked",
        )
        .expect("ordered post-insert ec_spire query should succeed")
        .expect("count should exist");
        let post_insert_cleanup_candidate_count = Spi::get_one::<i64>(
            "SELECT cleanup_candidate_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot(\
                 'ec_spire_logical_store_build_idx'::regclass)",
        )
        .expect("relation storage snapshot should succeed")
        .expect("storage row should exist");

        assert_eq!(post_insert_delta_count, 1);
        assert_eq!(post_insert_store_relid_count, 2);
        assert_eq!(post_insert_rows_returned, 5);
        assert!(post_insert_cleanup_candidate_count > 0);
    }
