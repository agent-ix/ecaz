    #[pg_test]
    fn test_ec_spire_hierarchy_snapshot_sql() {
        Spi::run("CREATE TABLE ec_spire_hierarchy_sql (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_hierarchy_empty_idx ON ec_spire_hierarchy_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");
        let empty_status = Spi::get_one::<String>(
            "SELECT status FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_empty_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");
        let empty_depth = Spi::get_one::<i32>(
            "SELECT hierarchy_depth FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_empty_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");
        let empty_routing_supported = Spi::get_one::<bool>(
            "SELECT recursive_routing_supported FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_empty_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");

        assert_eq!(empty_status, "empty");
        assert_eq!(empty_depth, 0);
        assert!(!empty_routing_supported);

        Spi::run("DROP INDEX ec_spire_hierarchy_empty_idx").expect("drop index should succeed");
        Spi::run(
            "INSERT INTO ec_spire_hierarchy_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_hierarchy_sql_idx ON ec_spire_hierarchy_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let status = Spi::get_one::<String>(
            "SELECT status FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");
        let root_routing_object_count = Spi::get_one::<i64>(
            "SELECT root_routing_object_count FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");
        let internal_routing_object_count = Spi::get_one::<i64>(
            "SELECT internal_routing_object_count FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");
        let leaf_object_count = Spi::get_one::<i64>(
            "SELECT leaf_object_count FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");
        let root_child_count = Spi::get_one::<i64>(
            "SELECT root_child_count FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");
        let centroid_dimensions = Spi::get_one::<i32>(
            "SELECT centroid_dimensions FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");
        let hierarchy_depth = Spi::get_one::<i32>(
            "SELECT hierarchy_depth FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");
        let leaf_parent_count = Spi::get_one::<i64>(
            "SELECT distinct_leaf_parent_count FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");
        let recursive_supported = Spi::get_one::<bool>(
            "SELECT recursive_routing_supported FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");
        let per_level_nprobe_supported = Spi::get_one::<bool>(
            "SELECT per_level_nprobe_supported FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_hierarchy_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("hierarchy row should exist");

        assert_eq!(status, "single_level_foundation");
        assert_eq!(root_routing_object_count, 1);
        assert_eq!(internal_routing_object_count, 0);
        assert_eq!(leaf_object_count, 2);
        assert_eq!(root_child_count, 2);
        assert_eq!(centroid_dimensions, 2);
        assert_eq!(hierarchy_depth, 1);
        assert_eq!(leaf_parent_count, 1);
        assert!(!recursive_supported);
        assert!(!per_level_nprobe_supported);
    }

    #[pg_test]
    fn test_ec_spire_object_snapshot_sql() {
        Spi::run("CREATE TABLE ec_spire_object_sql (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_object_empty_idx ON ec_spire_object_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");
        let empty_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_object_snapshot('ec_spire_object_empty_idx'::regclass)",
        )
        .expect("object snapshot query should succeed")
        .expect("count should exist");
        assert_eq!(empty_rows, 0);

        Spi::run("DROP INDEX ec_spire_object_empty_idx").expect("drop index should succeed");
        Spi::run(
            "INSERT INTO ec_spire_object_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_object_sql_idx ON ec_spire_object_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let object_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_object_snapshot('ec_spire_object_sql_idx'::regclass)",
        )
        .expect("object snapshot query should succeed")
        .expect("count should exist");
        let root_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_object_snapshot('ec_spire_object_sql_idx'::regclass) \
             WHERE object_kind = 'root' AND level = 1 AND parent_pid = 0 AND child_count = 2",
        )
        .expect("object snapshot query should succeed")
        .expect("count should exist");
        let leaf_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_object_snapshot('ec_spire_object_sql_idx'::regclass) \
             WHERE object_kind = 'leaf' AND level = 0 AND assignment_count = 1",
        )
        .expect("object snapshot query should succeed")
        .expect("count should exist");
        let all_available = Spi::get_one::<bool>(
            "SELECT bool_and(placement_state = 'available' AND object_readable) FROM \
             ec_spire_index_object_snapshot('ec_spire_object_sql_idx'::regclass)",
        )
        .expect("object snapshot query should succeed")
        .expect("bool aggregate should exist");
        let published_backrefs = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_object_snapshot('ec_spire_object_sql_idx'::regclass) \
             WHERE published_epoch_backref = active_epoch",
        )
        .expect("object snapshot query should succeed")
        .expect("count should exist");

        assert_eq!(object_count, 3);
        assert_eq!(root_count, 1);
        assert_eq!(leaf_count, 2);
        assert!(all_available);
        assert_eq!(published_backrefs, 3);

        Spi::run(
            "INSERT INTO ec_spire_object_sql (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[1.0, 0.1], 4, 42))",
        )
        .expect("post-build insert should succeed");
        let delta_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_object_snapshot('ec_spire_object_sql_idx'::regclass) \
             WHERE object_kind = 'delta' AND assignment_count = 1",
        )
        .expect("object snapshot query should succeed")
        .expect("count should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT max(active_epoch) FROM \
             ec_spire_index_object_snapshot('ec_spire_object_sql_idx'::regclass)",
        )
        .expect("object snapshot query should succeed")
        .expect("active epoch should exist");

        assert_eq!(delta_count, 1);
        assert_eq!(active_epoch, 2);
    }

    #[pg_test]
    fn test_ec_spire_delta_snapshot_sql() {
        Spi::run("CREATE TABLE ec_spire_delta_sql (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_delta_empty_idx ON ec_spire_delta_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");
        let empty_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_delta_snapshot('ec_spire_delta_empty_idx'::regclass)",
        )
        .expect("delta snapshot query should succeed")
        .expect("count should exist");
        assert_eq!(empty_rows, 0);

        Spi::run("DROP INDEX ec_spire_delta_empty_idx").expect("drop index should succeed");
        Spi::run(
            "INSERT INTO ec_spire_delta_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_delta_sql_idx ON ec_spire_delta_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");
        let initial_delta_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_delta_snapshot('ec_spire_delta_sql_idx'::regclass)",
        )
        .expect("delta snapshot query should succeed")
        .expect("count should exist");
        assert_eq!(initial_delta_rows, 0);

        Spi::run(
            "INSERT INTO ec_spire_delta_sql (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[1.0, 0.1], 4, 42))",
        )
        .expect("post-build insert should succeed");
        let delta_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_delta_snapshot('ec_spire_delta_sql_idx'::regclass)",
        )
        .expect("delta snapshot query should succeed")
        .expect("count should exist");
        let insert_assignment_count = Spi::get_one::<i64>(
            "SELECT sum(insert_assignment_count)::bigint FROM \
             ec_spire_index_delta_snapshot('ec_spire_delta_sql_idx'::regclass)",
        )
        .expect("delta snapshot query should succeed")
        .expect("sum should exist");
        let delete_assignment_count = Spi::get_one::<i64>(
            "SELECT sum(delete_assignment_count)::bigint FROM \
             ec_spire_index_delta_snapshot('ec_spire_delta_sql_idx'::regclass)",
        )
        .expect("delta snapshot query should succeed")
        .expect("sum should exist");
        let parent_leaf_matches = Spi::get_one::<bool>(
            "SELECT bool_and(parent_leaf_pid = leaf_pid) FROM \
             ec_spire_index_delta_snapshot('ec_spire_delta_sql_idx'::regclass) d \
             JOIN ec_spire_index_leaf_snapshot('ec_spire_delta_sql_idx'::regclass) l \
             ON d.parent_leaf_pid = l.leaf_pid",
        )
        .expect("delta snapshot query should succeed")
        .expect("join should produce a row");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT max(active_epoch) FROM \
             ec_spire_index_delta_snapshot('ec_spire_delta_sql_idx'::regclass)",
        )
        .expect("delta snapshot query should succeed")
        .expect("active epoch should exist");

        assert_eq!(delta_rows, 1);
        assert_eq!(insert_assignment_count, 1);
        assert_eq!(delete_assignment_count, 0);
        assert!(parent_leaf_matches);
        assert_eq!(active_epoch, 2);
    }

    #[pg_test]
    fn test_ec_spire_delta_snapshot_sql_delete_delta() {
        Spi::run(
            "CREATE TABLE ec_spire_delta_delete_sql (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_delta_delete_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_delta_delete_sql_idx ON ec_spire_delta_delete_sql \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let deleted_tid = heap_tid_for_row("ec_spire_delta_delete_sql", 1);
        Spi::run("DELETE FROM ec_spire_delta_delete_sql WHERE id = 1")
            .expect("delete should succeed");
        let index_oid = index_oid("ec_spire_delta_delete_sql_idx");
        let stats =
            unsafe { am::debug_spire_vacuum_bulkdelete_heap_tids(index_oid, &[deleted_tid]) };
        assert_eq!(stats.tuples_removed as i64, 1);

        let delta_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_delta_snapshot('ec_spire_delta_delete_sql_idx'::regclass)",
        )
        .expect("delta snapshot query should succeed")
        .expect("count should exist");
        let insert_assignment_count = Spi::get_one::<i64>(
            "SELECT sum(insert_assignment_count)::bigint FROM \
             ec_spire_index_delta_snapshot('ec_spire_delta_delete_sql_idx'::regclass)",
        )
        .expect("delta snapshot query should succeed")
        .expect("sum should exist");
        let delete_assignment_count = Spi::get_one::<i64>(
            "SELECT sum(delete_assignment_count)::bigint FROM \
             ec_spire_index_delta_snapshot('ec_spire_delta_delete_sql_idx'::regclass)",
        )
        .expect("delta snapshot query should succeed")
        .expect("sum should exist");
        let scan_delete_count = Spi::get_one::<i64>(
            "SELECT sum(delete_delta_row_count)::bigint FROM \
             ec_spire_index_scan_placement_snapshot( \
                 'ec_spire_delta_delete_sql_idx'::regclass, ARRAY[1.0, 0.0]::real[])",
        )
        .expect("scan placement diagnostics query should succeed")
        .expect("sum should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT max(active_epoch) FROM \
             ec_spire_index_delta_snapshot('ec_spire_delta_delete_sql_idx'::regclass)",
        )
        .expect("delta snapshot query should succeed")
        .expect("active epoch should exist");

        assert_eq!(delta_rows, 1);
        assert_eq!(insert_assignment_count, 0);
        assert_eq!(delete_assignment_count, 1);
        assert_eq!(scan_delete_count, 1);
        assert_eq!(active_epoch, 2);
    }

    #[pg_test]
    fn test_ec_spire_options_snapshot_sql() {
        Spi::run("CREATE TABLE ec_spire_options_sql (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_options_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_options_sql_idx ON ec_spire_options_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH ( \
                 nlists = 3, \
                 nprobe = 2, \
                 rerank_width = 7, \
                 training_sample_rows = 11, \
                 seed = 13, \
                 local_store_count = 1, \
                 local_store_tablespaces = 'pg_default', \
                 boundary_replica_count = 1, \
                 pq_group_size = 4, \
                 storage_format = 'rabitq' \
             )",
        )
        .expect("ec_spire index creation should succeed");
        Spi::run("SET LOCAL ec_spire.nprobe = 5").expect("SET should succeed");
        Spi::run("SET LOCAL ec_spire.rerank_width = 9").expect("SET should succeed");

        let nlists = Spi::get_one::<i32>(
            "SELECT nlists FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let recursive_fanout = Spi::get_one::<i32>(
            "SELECT recursive_fanout FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let recursive_build_enabled = Spi::get_one::<bool>(
            "SELECT recursive_build_enabled FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let local_store_count = Spi::get_one::<i32>(
            "SELECT local_store_count FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let local_store_tablespaces = Spi::get_one::<String>(
            "SELECT local_store_tablespaces FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let boundary_replica_count = Spi::get_one::<i32>(
            "SELECT boundary_replica_count FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let boundary_replication_enabled = Spi::get_one::<bool>(
            "SELECT boundary_replication_enabled FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let scan_dedupe_mode = Spi::get_one::<String>(
            "SELECT scan_dedupe_mode FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let session_nprobe = Spi::get_one::<i32>(
            "SELECT session_nprobe FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let storage_format = Spi::get_one::<String>(
            "SELECT storage_format FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let assignment_payload_format = Spi::get_one::<String>(
            "SELECT assignment_payload_format FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let assignment_payload_scannable = Spi::get_one::<bool>(
            "SELECT assignment_payload_scannable FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let assignment_payload_status = Spi::get_one::<String>(
            "SELECT assignment_payload_status FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");

        assert_eq!(nlists, 3);
        assert_eq!(recursive_fanout, 0);
        assert!(!recursive_build_enabled);
        assert_eq!(local_store_count, 1);
        assert_eq!(local_store_tablespaces, "pg_default");
        assert_eq!(boundary_replica_count, 1);
        assert!(boundary_replication_enabled);
        assert_eq!(scan_dedupe_mode, "vec_id");
        assert_eq!(session_nprobe, 5);
        assert_eq!(storage_format, "rabitq");
        assert_eq!(assignment_payload_format, "rabitq");
        assert!(assignment_payload_scannable);
        assert_eq!(assignment_payload_status, "supported");
        let active_leaf_count = Spi::get_one::<i64>(
            "SELECT active_leaf_count FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let effective_nprobe = Spi::get_one::<i64>(
            "SELECT effective_nprobe FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let effective_nprobe_source = Spi::get_one::<String>(
            "SELECT effective_nprobe_source FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let effective_nprobe_per_level = Spi::get_one::<Vec<i64>>(
            "SELECT effective_nprobe_per_level FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let nprobe_policy_per_level = Spi::get_one::<Vec<String>>(
            "SELECT nprobe_policy_per_level FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let effective_rerank_width = Spi::get_one::<i32>(
            "SELECT effective_rerank_width FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");
        let effective_rerank_width_source = Spi::get_one::<String>(
            "SELECT effective_rerank_width_source FROM \
             ec_spire_index_options_snapshot('ec_spire_options_sql_idx'::regclass)",
        )
        .expect("options query should succeed")
        .expect("options row should exist");

        assert_eq!(active_leaf_count, 3);
        assert_eq!(effective_nprobe, 3);
        assert_eq!(effective_nprobe_source, "session");
        assert_eq!(effective_nprobe_per_level, vec![3]);
        assert_eq!(nprobe_policy_per_level, vec!["single_level"]);
        assert_eq!(effective_rerank_width, 9);
        assert_eq!(effective_rerank_width_source, "session");

        Spi::run(
            "CREATE TABLE ec_spire_options_recursive_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("recursive options table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_options_recursive_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (4, encode_to_ecvector(ARRAY[-0.8, 0.2], 4, 42))",
        )
        .expect("recursive options insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_options_recursive_sql_idx \
             ON ec_spire_options_recursive_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 4, recursive_fanout = 2, nprobe = 4, nprobe_per_level = '2')",
        )
        .expect("recursive options ec_spire index creation should succeed");
        let recursive_fanout = Spi::get_one::<i32>(
            "SELECT recursive_fanout FROM \
             ec_spire_index_options_snapshot('ec_spire_options_recursive_sql_idx'::regclass)",
        )
        .expect("recursive options query should succeed")
        .expect("recursive options row should exist");
        let recursive_build_enabled = Spi::get_one::<bool>(
            "SELECT recursive_build_enabled FROM \
             ec_spire_index_options_snapshot('ec_spire_options_recursive_sql_idx'::regclass)",
        )
        .expect("recursive options query should succeed")
        .expect("recursive options row should exist");
        let recursive_active_leaf_count = Spi::get_one::<i64>(
            "SELECT active_leaf_count FROM \
             ec_spire_index_options_snapshot('ec_spire_options_recursive_sql_idx'::regclass)",
        )
        .expect("recursive options query should succeed")
        .expect("recursive options row should exist");
        let recursive_effective_nprobe_per_level = Spi::get_one::<Vec<i64>>(
            "SELECT effective_nprobe_per_level FROM \
             ec_spire_index_options_snapshot('ec_spire_options_recursive_sql_idx'::regclass)",
        )
        .expect("recursive options query should succeed")
        .expect("recursive options row should exist");
        let recursive_nprobe_policy_per_level = Spi::get_one::<Vec<String>>(
            "SELECT nprobe_policy_per_level FROM \
             ec_spire_index_options_snapshot('ec_spire_options_recursive_sql_idx'::regclass)",
        )
        .expect("recursive options query should succeed")
        .expect("recursive options row should exist");

        assert_eq!(recursive_fanout, 2);
        assert!(recursive_build_enabled);
        assert_eq!(recursive_active_leaf_count, 4);
        assert_eq!(recursive_effective_nprobe_per_level, vec![4, 2]);
        assert_eq!(
            recursive_nprobe_policy_per_level,
            vec!["relation_or_session_leaf_level", "configured_above_level_1"]
        );

        Spi::run(
            "CREATE TABLE ec_spire_options_pq_empty \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("empty table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_options_pq_empty_idx ON ec_spire_options_pq_empty \
             USING ec_spire (embedding ecvector_spire_ip_ops) \
             WITH (storage_format = 'pq_fastscan')",
        )
        .expect("empty pq_fastscan ec_spire index creation should succeed");
        let pq_storage_format = Spi::get_one::<String>(
            "SELECT storage_format FROM \
             ec_spire_index_options_snapshot('ec_spire_options_pq_empty_idx'::regclass)",
        )
        .expect("pq_fastscan options query should succeed")
        .expect("pq_fastscan options row should exist");
        let pq_assignment_payload_format = Spi::get_one::<String>(
            "SELECT assignment_payload_format FROM \
             ec_spire_index_options_snapshot('ec_spire_options_pq_empty_idx'::regclass)",
        )
        .expect("pq_fastscan options query should succeed")
        .expect("pq_fastscan options row should exist");
        let pq_assignment_payload_scannable = Spi::get_one::<bool>(
            "SELECT assignment_payload_scannable FROM \
             ec_spire_index_options_snapshot('ec_spire_options_pq_empty_idx'::regclass)",
        )
        .expect("pq_fastscan options query should succeed")
        .expect("pq_fastscan options row should exist");
        let pq_assignment_payload_status = Spi::get_one::<String>(
            "SELECT assignment_payload_status FROM \
             ec_spire_index_options_snapshot('ec_spire_options_pq_empty_idx'::regclass)",
        )
        .expect("pq_fastscan options query should succeed")
        .expect("pq_fastscan options row should exist");
        let pq_assignment_payload_recommendation = Spi::get_one::<String>(
            "SELECT assignment_payload_recommendation FROM \
             ec_spire_index_options_snapshot('ec_spire_options_pq_empty_idx'::regclass)",
        )
        .expect("pq_fastscan options query should succeed")
        .expect("pq_fastscan options row should exist");

        assert_eq!(pq_storage_format, "pq_fastscan");
        assert_eq!(pq_assignment_payload_format, "pq_fastscan");
        assert!(!pq_assignment_payload_scannable);
        assert_eq!(pq_assignment_payload_status, "deferred_model_metadata");
        assert!(pq_assignment_payload_recommendation.contains("grouped-PQ model"));
    }

    #[pg_test]
    fn test_ec_spire_scan_sanity_snapshot_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_scan_sanity_sql (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_scan_sanity_empty_idx ON ec_spire_scan_sanity_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");
        let empty_status = Spi::get_one::<String>(
            "SELECT recall_sanity_status FROM \
             ec_spire_index_scan_sanity_snapshot('ec_spire_scan_sanity_empty_idx'::regclass)",
        )
        .expect("scan sanity query should succeed")
        .expect("scan sanity row should exist");
        assert_eq!(empty_status, "empty");

        Spi::run("DROP INDEX ec_spire_scan_sanity_empty_idx").expect("drop index should succeed");
        Spi::run(
            "INSERT INTO ec_spire_scan_sanity_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_scan_sanity_sql_idx ON ec_spire_scan_sanity_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2, nprobe = 1, rerank_width = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let active_leaf_count = Spi::get_one::<i64>(
            "SELECT active_leaf_count FROM \
             ec_spire_index_scan_sanity_snapshot('ec_spire_scan_sanity_sql_idx'::regclass)",
        )
        .expect("scan sanity query should succeed")
        .expect("scan sanity row should exist");
        let exact_leaf_coverage = Spi::get_one::<bool>(
            "SELECT exact_leaf_coverage FROM \
             ec_spire_index_scan_sanity_snapshot('ec_spire_scan_sanity_sql_idx'::regclass)",
        )
        .expect("scan sanity query should succeed")
        .expect("scan sanity row should exist");
        let full_frontier_rerank = Spi::get_one::<bool>(
            "SELECT full_frontier_rerank FROM \
             ec_spire_index_scan_sanity_snapshot('ec_spire_scan_sanity_sql_idx'::regclass)",
        )
        .expect("scan sanity query should succeed")
        .expect("scan sanity row should exist");
        let recall_sanity_status = Spi::get_one::<String>(
            "SELECT recall_sanity_status FROM \
             ec_spire_index_scan_sanity_snapshot('ec_spire_scan_sanity_sql_idx'::regclass)",
        )
        .expect("scan sanity query should succeed")
        .expect("scan sanity row should exist");

        assert_eq!(active_leaf_count, 2);
        assert!(!exact_leaf_coverage);
        assert!(!full_frontier_rerank);
        assert_eq!(recall_sanity_status, "approximate_leaf_coverage");

        Spi::run("SET LOCAL ec_spire.nprobe = 2").expect("SET should succeed");
        Spi::run("SET LOCAL ec_spire.rerank_width = 0").expect("SET should succeed");
        let exact_leaf_coverage = Spi::get_one::<bool>(
            "SELECT exact_leaf_coverage FROM \
             ec_spire_index_scan_sanity_snapshot('ec_spire_scan_sanity_sql_idx'::regclass)",
        )
        .expect("scan sanity query should succeed")
        .expect("scan sanity row should exist");
        let full_frontier_rerank = Spi::get_one::<bool>(
            "SELECT full_frontier_rerank FROM \
             ec_spire_index_scan_sanity_snapshot('ec_spire_scan_sanity_sql_idx'::regclass)",
        )
        .expect("scan sanity query should succeed")
        .expect("scan sanity row should exist");
        let recall_sanity_status = Spi::get_one::<String>(
            "SELECT recall_sanity_status FROM \
             ec_spire_index_scan_sanity_snapshot('ec_spire_scan_sanity_sql_idx'::regclass)",
        )
        .expect("scan sanity query should succeed")
        .expect("scan sanity row should exist");
        let latency_risk_status = Spi::get_one::<String>(
            "SELECT latency_risk_status FROM \
             ec_spire_index_scan_sanity_snapshot('ec_spire_scan_sanity_sql_idx'::regclass)",
        )
        .expect("scan sanity query should succeed")
        .expect("scan sanity row should exist");

        assert!(exact_leaf_coverage);
        assert!(full_frontier_rerank);
        assert_eq!(recall_sanity_status, "exact_leaf_and_frontier_coverage");
        assert_eq!(latency_risk_status, "full_scan");
    }

    #[pg_test]
    fn test_ec_spire_health_snapshot_sql() {
        Spi::run("CREATE TABLE ec_spire_health_sql (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_health_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_health_sql_idx ON ec_spire_health_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let status = Spi::get_one::<String>(
            "SELECT status FROM \
             ec_spire_index_health_snapshot('ec_spire_health_sql_idx'::regclass)",
        )
        .expect("health query should succeed")
        .expect("health row should exist");
        let healthy = Spi::get_one::<bool>(
            "SELECT healthy FROM \
             ec_spire_index_health_snapshot('ec_spire_health_sql_idx'::regclass)",
        )
        .expect("health query should succeed")
        .expect("health row should exist");
        let compaction_recommended = Spi::get_one::<bool>(
            "SELECT compaction_recommended FROM \
             ec_spire_index_health_snapshot('ec_spire_health_sql_idx'::regclass)",
        )
        .expect("health query should succeed")
        .expect("health row should exist");

        assert_eq!(status, "ok");
        assert!(healthy);
        assert!(!compaction_recommended);

        Spi::run(
            "INSERT INTO ec_spire_health_sql (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42))",
        )
        .expect("delta insert should succeed");

        let status = Spi::get_one::<String>(
            "SELECT status FROM \
             ec_spire_index_health_snapshot('ec_spire_health_sql_idx'::regclass)",
        )
        .expect("health query should succeed")
        .expect("health row should exist");
        let recommendation = Spi::get_one::<String>(
            "SELECT recommendation FROM \
             ec_spire_index_health_snapshot('ec_spire_health_sql_idx'::regclass)",
        )
        .expect("health query should succeed")
        .expect("health row should exist");
        let compaction_recommended = Spi::get_one::<bool>(
            "SELECT compaction_recommended FROM \
             ec_spire_index_health_snapshot('ec_spire_health_sql_idx'::regclass)",
        )
        .expect("health query should succeed")
        .expect("health row should exist");
        let delta_object_count = Spi::get_one::<i64>(
            "SELECT delta_object_count FROM \
             ec_spire_index_health_snapshot('ec_spire_health_sql_idx'::regclass)",
        )
        .expect("health query should succeed")
        .expect("health row should exist");

        assert_eq!(status, "maintenance_recommended");
        assert_eq!(
            recommendation,
            "run VACUUM to compact active delta objects into V2 base leaves"
        );
        assert!(compaction_recommended);
        assert_eq!(delta_object_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_relation_storage_snapshot_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_storage_debt_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_storage_debt_empty_idx ON ec_spire_storage_debt_sql \
             USING ec_spire (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");
        let empty_tuple_count = Spi::get_one::<i64>(
            "SELECT relation_object_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot(\
                 'ec_spire_storage_debt_empty_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");
        assert_eq!(empty_tuple_count, 0);

        Spi::run("DROP INDEX ec_spire_storage_debt_empty_idx").expect("drop index should succeed");
        Spi::run(
            "INSERT INTO ec_spire_storage_debt_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_storage_debt_sql_idx ON ec_spire_storage_debt_sql \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let build_tuple_count = Spi::get_one::<i64>(
            "SELECT relation_object_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");
        let build_active_tuple_count = Spi::get_one::<i64>(
            "SELECT active_referenced_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");
        let build_cleanup_candidate_count = Spi::get_one::<i64>(
            "SELECT cleanup_candidate_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");
        let cleanup_supported = Spi::get_one::<bool>(
            "SELECT physical_cleanup_supported FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");

        assert!(build_tuple_count > 0);
        assert_eq!(build_active_tuple_count, build_tuple_count);
        assert_eq!(build_cleanup_candidate_count, 0);
        assert!(cleanup_supported);

        Spi::run(
            "INSERT INTO ec_spire_storage_debt_sql (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42))",
        )
        .expect("post-build insert should publish a delta epoch");

        let post_insert_active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");
        let post_insert_tuple_count = Spi::get_one::<i64>(
            "SELECT relation_object_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");
        let post_insert_cleanup_candidate_count = Spi::get_one::<i64>(
            "SELECT cleanup_candidate_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");
        let post_insert_cleanup_candidate_bytes = Spi::get_one::<i64>(
            "SELECT cleanup_candidate_tuple_bytes FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");
        let cleanup_summary_status = Spi::get_one::<String>(
            "SELECT physical_cleanup_status FROM \
             ec_spire_index_epoch_cleanup_summary('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("cleanup summary query should succeed")
        .expect("cleanup summary row should exist");
        let cleanup_summary_candidate_count = Spi::get_one::<i64>(
            "SELECT cleanup_candidate_tuple_count FROM \
             ec_spire_index_epoch_cleanup_summary('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("cleanup summary query should succeed")
        .expect("cleanup summary row should exist");
        let cleanup_summary_retired_count = Spi::get_one::<i64>(
            "SELECT retired_epoch_count FROM \
             ec_spire_index_epoch_cleanup_summary('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("cleanup summary query should succeed")
        .expect("cleanup summary row should exist");
        let recommendation = Spi::get_one::<String>(
            "SELECT recommendation FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");

        assert_eq!(post_insert_active_epoch, 2);
        assert!(post_insert_tuple_count > build_tuple_count);
        assert!(post_insert_cleanup_candidate_count > 0);
        assert!(post_insert_cleanup_candidate_bytes > 0);
        assert_eq!(cleanup_summary_status, "blocked_by_retention");
        assert_eq!(
            cleanup_summary_candidate_count,
            post_insert_cleanup_candidate_count
        );
        assert_eq!(cleanup_summary_retired_count, 1);
        assert!(recommendation.contains("epoch_cleanup_run"));

        let index_oid = index_oid("ec_spire_storage_debt_sql_idx");
        let stats = unsafe { am::debug_spire_vacuum_remove_heap_tids(index_oid, &[]) };
        assert_eq!(stats.tuples_removed, 0.0);
        assert_eq!(stats.num_index_tuples, 3.0);

        let post_compaction_active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");
        let post_compaction_tuple_count = Spi::get_one::<i64>(
            "SELECT relation_object_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");
        let post_compaction_cleanup_candidate_count = Spi::get_one::<i64>(
            "SELECT cleanup_candidate_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");
        let post_compaction_cleanup_candidate_bytes = Spi::get_one::<i64>(
            "SELECT cleanup_candidate_tuple_bytes FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_storage_debt_sql_idx'::regclass)",
        )
        .expect("storage snapshot query should succeed")
        .expect("storage snapshot row should exist");

        assert_eq!(post_compaction_active_epoch, 3);
        assert!(post_compaction_tuple_count > post_insert_tuple_count);
        assert!(post_compaction_cleanup_candidate_count > post_insert_cleanup_candidate_count);
        assert!(post_compaction_cleanup_candidate_bytes > post_insert_cleanup_candidate_bytes);
    }
