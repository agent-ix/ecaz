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

