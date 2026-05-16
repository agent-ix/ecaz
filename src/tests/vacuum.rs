    #[pg_test]
    fn test_ec_spire_epoch_cleanup_run_reclaims_old_tuples_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_epoch_cleanup_run_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_epoch_cleanup_run_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("initial insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_epoch_cleanup_run_idx ON ec_spire_epoch_cleanup_run_sql \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");
        for id in 3..=6 {
            Spi::run(&format!(
                "INSERT INTO ec_spire_epoch_cleanup_run_sql (id, embedding) VALUES \
                 ({id}, encode_to_ecvector(ARRAY[{id}.0, 0.5], 4, 42))",
            ))
            .expect("post-build insert should publish a delta epoch");
        }

        let index_oid = index_oid("ec_spire_epoch_cleanup_run_idx");
        let aged = unsafe { am::debug_spire_age_retired_epoch_manifests(index_oid, 1) };
        assert!(aged >= 3);

        let pre_tuple_count = Spi::get_one::<i64>(
            "SELECT relation_object_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_epoch_cleanup_run_idx'::regclass)",
        )
        .expect("storage snapshot should succeed")
        .expect("tuple count should exist");
        let pre_cleanup_eligible_count = Spi::get_one::<i64>(
            "SELECT cleanup_eligible_epoch_count FROM \
             ec_spire_index_epoch_cleanup_summary('ec_spire_epoch_cleanup_run_idx'::regclass)",
        )
        .expect("cleanup summary should succeed")
        .expect("cleanup eligible count should exist");
        let pre_cleanup_status = Spi::get_one::<String>(
            "SELECT physical_cleanup_status FROM \
             ec_spire_index_epoch_cleanup_summary('ec_spire_epoch_cleanup_run_idx'::regclass)",
        )
        .expect("cleanup summary should succeed")
        .expect("cleanup status should exist");

        Spi::run(
            "CREATE TEMP TABLE ec_spire_epoch_cleanup_run_result AS \
             SELECT * FROM \
             ec_spire_index_epoch_cleanup_run('ec_spire_epoch_cleanup_run_idx'::regclass)",
        )
        .expect("physical cleanup run should succeed");

        let run_status = Spi::get_one::<String>(
            "SELECT physical_cleanup_status FROM ec_spire_epoch_cleanup_run_result",
        )
        .expect("cleanup run result should succeed")
        .expect("cleanup run status should exist");
        let removed_tuple_count = Spi::get_one::<i64>(
            "SELECT removed_tuple_count FROM ec_spire_epoch_cleanup_run_result",
        )
        .expect("cleanup run result should succeed")
        .expect("removed tuple count should exist");
        let removed_tuple_bytes = Spi::get_one::<i64>(
            "SELECT removed_tuple_bytes FROM ec_spire_epoch_cleanup_run_result",
        )
        .expect("cleanup run result should succeed")
        .expect("removed tuple bytes should exist");
        let post_tuple_count = Spi::get_one::<i64>(
            "SELECT relation_object_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot('ec_spire_epoch_cleanup_run_idx'::regclass)",
        )
        .expect("storage snapshot should succeed")
        .expect("tuple count should exist");
        let visible_count =
            Spi::get_one::<i64>("SELECT count(*) FROM ec_spire_epoch_cleanup_run_sql")
                .expect("heap count should succeed")
                .expect("heap count should exist");
        let top_id = Spi::get_one::<i64>(
            "SELECT id FROM ec_spire_epoch_cleanup_run_sql \
             ORDER BY embedding <#> ARRAY[6.0, 0.5]::real[] LIMIT 1",
        )
        .expect("post-cleanup scan should succeed")
        .expect("top row should exist");

        assert!(pre_cleanup_eligible_count > 0);
        assert_eq!(pre_cleanup_status, "supported");
        assert_eq!(run_status, "reclaimed");
        assert!(removed_tuple_count > 0);
        assert!(removed_tuple_bytes > 0);
        assert!(post_tuple_count < pre_tuple_count);
        assert_eq!(visible_count, 6);
        assert_eq!(top_id, 6);
    }

    #[pg_test]
    fn test_ec_spire_epoch_snapshot_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_epoch_snapshot_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_epoch_snapshot_empty_idx ON ec_spire_epoch_snapshot_sql \
             USING ec_spire (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");
        let empty_epoch_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_empty_idx'::regclass)",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");
        assert_eq!(empty_epoch_count, 0);

        Spi::run("DROP INDEX ec_spire_epoch_snapshot_empty_idx")
            .expect("drop index should succeed");
        Spi::run(
            "INSERT INTO ec_spire_epoch_snapshot_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_epoch_snapshot_sql_idx ON ec_spire_epoch_snapshot_sql \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let build_epoch_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass)",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");
        let build_active_root_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass) \
             WHERE is_active_root_manifest",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");
        let build_state = Spi::get_one::<String>(
            "SELECT state FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass)",
        )
        .expect("epoch snapshot query should succeed")
        .expect("epoch row should exist");
        let build_cleanup_reason = Spi::get_one::<String>(
            "SELECT cleanup_blocked_reason FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass)",
        )
        .expect("epoch snapshot query should succeed")
        .expect("epoch row should exist");
        assert_eq!(build_epoch_count, 1);
        assert_eq!(build_active_root_count, 1);
        assert_eq!(build_state, "published");
        assert_eq!(build_cleanup_reason, "active_root_manifest");

        Spi::run(
            "INSERT INTO ec_spire_epoch_snapshot_sql (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42))",
        )
        .expect("post-build insert should publish a delta epoch");

        let post_insert_manifest_row_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass)",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");
        let post_insert_distinct_epoch_count = Spi::get_one::<i64>(
            "SELECT count(DISTINCT epoch) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass)",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");
        let retired_epoch_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass) \
             WHERE state = 'retired'",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");
        let superseded_manifest_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass) \
             WHERE cleanup_blocked_reason = 'superseded_manifest'",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT max(active_epoch) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass)",
        )
        .expect("epoch snapshot query should succeed")
        .expect("max row should exist");
        let active_root_epoch = Spi::get_one::<i64>(
            "SELECT epoch FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass) \
             WHERE is_active_root_manifest",
        )
        .expect("epoch snapshot query should succeed")
        .expect("active root row should exist");
        let cleanup_eligible_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass) \
             WHERE cleanup_eligible_now",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");

        assert_eq!(post_insert_manifest_row_count, 3);
        assert_eq!(post_insert_distinct_epoch_count, 2);
        assert_eq!(retired_epoch_count, 1);
        assert_eq!(superseded_manifest_count, 1);
        assert_eq!(active_epoch, 2);
        assert_eq!(active_root_epoch, 2);
        assert_eq!(cleanup_eligible_count, 0);

        let index_oid = index_oid("ec_spire_epoch_snapshot_sql_idx");
        let stats = unsafe { am::debug_spire_vacuum_remove_heap_tids(index_oid, &[]) };
        assert_eq!(stats.tuples_removed, 0.0);
        assert_eq!(stats.num_index_tuples, 3.0);

        let post_compaction_manifest_row_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass)",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");
        let post_compaction_distinct_epoch_count = Spi::get_one::<i64>(
            "SELECT count(DISTINCT epoch) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass)",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");
        let post_compaction_retired_epoch_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass) \
             WHERE state = 'retired'",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");
        let post_compaction_superseded_manifest_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass) \
             WHERE cleanup_blocked_reason = 'superseded_manifest'",
        )
        .expect("epoch snapshot query should succeed")
        .expect("count row should exist");
        let post_compaction_active_epoch = Spi::get_one::<i64>(
            "SELECT max(active_epoch) FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass)",
        )
        .expect("epoch snapshot query should succeed")
        .expect("max row should exist");
        let post_compaction_active_root_epoch = Spi::get_one::<i64>(
            "SELECT epoch FROM \
             ec_spire_index_epoch_snapshot('ec_spire_epoch_snapshot_sql_idx'::regclass) \
             WHERE is_active_root_manifest",
        )
        .expect("epoch snapshot query should succeed")
        .expect("active root row should exist");

        assert_eq!(post_compaction_manifest_row_count, 5);
        assert_eq!(post_compaction_distinct_epoch_count, 3);
        assert_eq!(post_compaction_retired_epoch_count, 2);
        assert_eq!(post_compaction_superseded_manifest_count, 2);
        assert_eq!(post_compaction_active_epoch, 3);
        assert_eq!(post_compaction_active_root_epoch, 3);
    }

    #[pg_test]
    fn test_ec_spire_maintenance_run_empty_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_maintenance_run_empty_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_maintenance_run_empty_idx \
             ON ec_spire_maintenance_run_empty_sql \
             USING ec_spire (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");

        let status = Spi::get_one::<String>(
            "SELECT maintenance_status FROM \
             ec_spire_index_maintenance_run('ec_spire_maintenance_run_empty_idx'::regclass)",
        )
        .expect("maintenance run should succeed")
        .expect("status row should exist");
        let action = Spi::get_one::<String>(
            "SELECT planned_action FROM \
             ec_spire_index_maintenance_run('ec_spire_maintenance_run_empty_idx'::regclass)",
        )
        .expect("maintenance run should succeed")
        .expect("action row should exist");
        let reason = Spi::get_one::<String>(
            "SELECT planned_reason FROM \
             ec_spire_index_maintenance_run('ec_spire_maintenance_run_empty_idx'::regclass)",
        )
        .expect("maintenance run should succeed")
        .expect("reason row should exist");
        let published = Spi::get_one::<bool>(
            "SELECT published FROM \
             ec_spire_index_maintenance_run('ec_spire_maintenance_run_empty_idx'::regclass)",
        )
        .expect("maintenance run should succeed")
        .expect("published row should exist");
        let active_epoch_after = Spi::get_one::<i64>(
            "SELECT active_epoch_after FROM \
             ec_spire_index_maintenance_run('ec_spire_maintenance_run_empty_idx'::regclass)",
        )
        .expect("maintenance run should succeed")
        .expect("active epoch row should exist");
        let scheduler_status = Spi::get_one::<String>(
            "SELECT scheduler_status FROM \
             ec_spire_index_maintenance_scheduler_plan(\
             'ec_spire_maintenance_run_empty_idx'::regclass)",
        )
        .expect("scheduler plan should succeed")
        .expect("scheduler status row should exist");
        let scheduler_reason = Spi::get_one::<String>(
            "SELECT planned_reason FROM \
             ec_spire_index_maintenance_scheduler_plan(\
             'ec_spire_maintenance_run_empty_idx'::regclass)",
        )
        .expect("scheduler plan should succeed")
        .expect("scheduler reason row should exist");
        let scheduler_run_status = Spi::get_one::<String>(
            "SELECT scheduler_status FROM \
             ec_spire_index_maintenance_scheduler_run(\
             'ec_spire_maintenance_run_empty_idx'::regclass)",
        )
        .expect("scheduler run should succeed")
        .expect("scheduler run status row should exist");

        assert_eq!(status, "no_action");
        assert_eq!(action, "none");
        assert_eq!(reason, "empty_index");
        assert!(!published);
        assert_eq!(active_epoch_after, 0);
        assert_eq!(scheduler_status, "idle");
        assert_eq!(scheduler_reason, "empty_index");
        assert_eq!(scheduler_run_status, "idle");
    }

    #[pg_test]
    fn test_ec_spire_locked_maintenance_run_plan_no_write_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_locked_maintenance_run_plan_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_locked_maintenance_run_plan_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_locked_maintenance_run_plan_idx \
             ON ec_spire_locked_maintenance_run_plan_sql \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 3)",
        )
        .expect("populated ec_spire index creation should succeed");

        let pre_active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_active_snapshot_diagnostics(\
             'ec_spire_locked_maintenance_run_plan_idx'::regclass)",
        )
        .expect("active snapshot query should succeed")
        .expect("active epoch row should exist");
        let pre_next_pid = Spi::get_one::<i64>(
            "SELECT next_pid FROM \
             ec_spire_index_active_snapshot_diagnostics(\
             'ec_spire_locked_maintenance_run_plan_idx'::regclass)",
        )
        .expect("active snapshot query should succeed")
        .expect("next pid row should exist");
        let pre_leaf_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_leaf_snapshot(\
             'ec_spire_locked_maintenance_run_plan_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("count row should exist");

        Spi::run(
            "CREATE TEMP TABLE ec_spire_locked_maintenance_run_plan_result AS \
             SELECT * FROM \
             ec_spire_index_locked_maintenance_run_plan(\
             'ec_spire_locked_maintenance_run_plan_idx'::regclass)",
        )
        .expect("locked maintenance run plan should succeed");

        let status = Spi::get_one::<String>(
            "SELECT maintenance_status FROM ec_spire_locked_maintenance_run_plan_result",
        )
        .expect("maintenance run plan result query should succeed")
        .expect("status row should exist");
        let action = Spi::get_one::<String>(
            "SELECT planned_action FROM ec_spire_locked_maintenance_run_plan_result",
        )
        .expect("maintenance run plan result query should succeed")
        .expect("action row should exist");
        let active_epoch_before = Spi::get_one::<i64>(
            "SELECT active_epoch_before FROM ec_spire_locked_maintenance_run_plan_result",
        )
        .expect("maintenance run plan result query should succeed")
        .expect("active epoch before row should exist");
        let active_epoch_after = Spi::get_one::<i64>(
            "SELECT active_epoch_after FROM ec_spire_locked_maintenance_run_plan_result",
        )
        .expect("maintenance run plan result query should succeed")
        .expect("active epoch after row should exist");
        let publish_epoch = Spi::get_one::<i64>(
            "SELECT publish_epoch FROM ec_spire_locked_maintenance_run_plan_result",
        )
        .expect("maintenance run plan result query should succeed")
        .expect("publish epoch row should exist");
        let affected_leaf_pids = Spi::get_one::<String>(
            "SELECT affected_leaf_pids FROM ec_spire_locked_maintenance_run_plan_result",
        )
        .expect("maintenance run plan result query should succeed")
        .expect("affected leaf pids row should exist");
        let replacement_leaf_pids = Spi::get_one::<String>(
            "SELECT replacement_leaf_pids FROM ec_spire_locked_maintenance_run_plan_result",
        )
        .expect("maintenance run plan result query should succeed")
        .expect("replacement leaf pids row should exist");
        let next_pid =
            Spi::get_one::<i64>("SELECT next_pid FROM ec_spire_locked_maintenance_run_plan_result")
                .expect("maintenance run plan result query should succeed")
                .expect("next pid row should exist");
        let published = Spi::get_one::<bool>(
            "SELECT published FROM ec_spire_locked_maintenance_run_plan_result",
        )
        .expect("maintenance run plan result query should succeed")
        .expect("published row should exist");
        let post_active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_active_snapshot_diagnostics(\
             'ec_spire_locked_maintenance_run_plan_idx'::regclass)",
        )
        .expect("active snapshot query should succeed")
        .expect("active epoch row should exist");
        let post_next_pid = Spi::get_one::<i64>(
            "SELECT next_pid FROM \
             ec_spire_index_active_snapshot_diagnostics(\
             'ec_spire_locked_maintenance_run_plan_idx'::regclass)",
        )
        .expect("active snapshot query should succeed")
        .expect("next pid row should exist");
        let post_leaf_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_leaf_snapshot(\
             'ec_spire_locked_maintenance_run_plan_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("count row should exist");

        assert_eq!(status, "planned");
        assert_eq!(action, "merge");
        assert_eq!(active_epoch_before, pre_active_epoch);
        assert_eq!(active_epoch_after, pre_active_epoch);
        assert_eq!(publish_epoch, pre_active_epoch + 1);
        assert!(!published);
        assert_eq!(post_active_epoch, pre_active_epoch);
        assert_eq!(post_next_pid, pre_next_pid);
        assert_eq!(post_leaf_count, pre_leaf_count);

        let scheduler_status = Spi::get_one::<String>(
            "SELECT scheduler_status FROM \
             ec_spire_index_maintenance_scheduler_plan(\
             'ec_spire_locked_maintenance_run_plan_idx'::regclass)",
        )
        .expect("scheduler plan query should succeed")
        .expect("scheduler status row should exist");
        let scheduler_policy = Spi::get_one::<String>(
            "SELECT scheduler_policy FROM \
             ec_spire_index_maintenance_scheduler_plan(\
             'ec_spire_locked_maintenance_run_plan_idx'::regclass)",
        )
        .expect("scheduler plan query should succeed")
        .expect("scheduler policy row should exist");
        let scheduler_lock_recheck = Spi::get_one::<bool>(
            "SELECT lock_time_recheck FROM \
             ec_spire_index_maintenance_scheduler_plan(\
             'ec_spire_locked_maintenance_run_plan_idx'::regclass)",
        )
        .expect("scheduler plan query should succeed")
        .expect("scheduler lock-time recheck row should exist");

        assert_eq!(scheduler_status, "due");
        assert_eq!(scheduler_policy, "operator_periodic_job");
        assert!(scheduler_lock_recheck);

        Spi::run(
            "CREATE TEMP TABLE ec_spire_locked_maintenance_run_publish_result AS \
             SELECT * FROM \
             ec_spire_index_maintenance_scheduler_run(\
             'ec_spire_locked_maintenance_run_plan_idx'::regclass)",
        )
        .expect("scheduler run should publish the planned replacement");

        let run_scheduler_status = Spi::get_one::<String>(
            "SELECT scheduler_status FROM ec_spire_locked_maintenance_run_publish_result",
        )
        .expect("scheduler run result query should succeed")
        .expect("scheduler status row should exist");
        let run_status = Spi::get_one::<String>(
            "SELECT maintenance_status FROM ec_spire_locked_maintenance_run_publish_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("status row should exist");
        let run_action = Spi::get_one::<String>(
            "SELECT planned_action FROM ec_spire_locked_maintenance_run_publish_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("action row should exist");
        let run_affected_leaf_pids = Spi::get_one::<String>(
            "SELECT affected_leaf_pids FROM ec_spire_locked_maintenance_run_publish_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("affected leaf pids row should exist");
        let run_replacement_leaf_pids = Spi::get_one::<String>(
            "SELECT replacement_leaf_pids FROM ec_spire_locked_maintenance_run_publish_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("replacement leaf pids row should exist");
        let run_publish_epoch = Spi::get_one::<i64>(
            "SELECT publish_epoch FROM ec_spire_locked_maintenance_run_publish_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("publish epoch row should exist");
        let run_active_epoch_after = Spi::get_one::<i64>(
            "SELECT active_epoch_after FROM ec_spire_locked_maintenance_run_publish_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("active epoch after row should exist");
        let run_next_pid = Spi::get_one::<i64>(
            "SELECT next_pid FROM ec_spire_locked_maintenance_run_publish_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("next pid row should exist");
        let run_published = Spi::get_one::<bool>(
            "SELECT published FROM ec_spire_locked_maintenance_run_publish_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("published row should exist");

        assert_eq!(run_scheduler_status, "ran");
        assert_eq!(run_status, "published");
        assert_eq!(run_action, action);
        assert_eq!(run_affected_leaf_pids, affected_leaf_pids);
        assert_eq!(run_replacement_leaf_pids, replacement_leaf_pids);
        assert_eq!(run_publish_epoch, publish_epoch);
        assert_eq!(run_active_epoch_after, publish_epoch);
        assert_eq!(run_next_pid, next_pid);
        assert!(run_published);
    }

    #[pg_test]
    fn test_ec_spire_maintenance_run_no_candidate_sql() {
        let maintenance_run_volatility = Spi::get_one::<String>(
            "SELECT provolatile::text FROM pg_proc \
             WHERE proname = 'ec_spire_index_maintenance_run' AND pronargs = 1",
        )
        .expect("pg_proc volatility query should succeed")
        .expect("maintenance run function should exist");
        assert_eq!(maintenance_run_volatility, "v");

        Spi::run(
            "CREATE TABLE ec_spire_maintenance_run_no_candidate_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_maintenance_run_no_candidate_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_maintenance_run_no_candidate_idx \
             ON ec_spire_maintenance_run_no_candidate_sql \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let pre_active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_active_snapshot_diagnostics(\
             'ec_spire_maintenance_run_no_candidate_idx'::regclass)",
        )
        .expect("active snapshot query should succeed")
        .expect("active epoch row should exist");
        let pre_leaf_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_leaf_snapshot(\
             'ec_spire_maintenance_run_no_candidate_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("count row should exist");
        assert_eq!(pre_active_epoch, 1);
        assert_eq!(pre_leaf_count, 2);

        Spi::run(
            "CREATE TEMP TABLE ec_spire_maintenance_run_no_candidate_result AS \
             SELECT * FROM \
             ec_spire_index_maintenance_run(\
             'ec_spire_maintenance_run_no_candidate_idx'::regclass)",
        )
        .expect("maintenance run should return no candidate");

        let status = Spi::get_one::<String>(
            "SELECT maintenance_status FROM ec_spire_maintenance_run_no_candidate_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("status row should exist");
        let action = Spi::get_one::<String>(
            "SELECT planned_action FROM ec_spire_maintenance_run_no_candidate_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("action row should exist");
        let reason = Spi::get_one::<String>(
            "SELECT planned_reason FROM ec_spire_maintenance_run_no_candidate_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("reason row should exist");
        let active_epoch_before = Spi::get_one::<i64>(
            "SELECT active_epoch_before FROM ec_spire_maintenance_run_no_candidate_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("active epoch before row should exist");
        let active_epoch_after = Spi::get_one::<i64>(
            "SELECT active_epoch_after FROM ec_spire_maintenance_run_no_candidate_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("active epoch after row should exist");
        let published = Spi::get_one::<bool>(
            "SELECT published FROM ec_spire_maintenance_run_no_candidate_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("published row should exist");
        let post_active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_active_snapshot_diagnostics(\
             'ec_spire_maintenance_run_no_candidate_idx'::regclass)",
        )
        .expect("active snapshot query should succeed")
        .expect("active epoch row should exist");
        let post_leaf_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_leaf_snapshot(\
             'ec_spire_maintenance_run_no_candidate_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("count row should exist");

        assert_eq!(status, "no_action");
        assert_eq!(action, "none");
        assert_eq!(reason, "no_candidate");
        assert_eq!(active_epoch_before, pre_active_epoch);
        assert_eq!(active_epoch_after, pre_active_epoch);
        assert!(!published);
        assert_eq!(post_active_epoch, pre_active_epoch);
        assert_eq!(post_leaf_count, pre_leaf_count);
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_spire maintenance split/merge is deferred for recursive SPIRE indexes until recursive update propagation lands"
    )]
    fn test_ec_spire_recursive_maintenance_run_rejected() {
        Spi::run(
            "CREATE TABLE ec_spire_recursive_maintenance_rejected \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_recursive_maintenance_rejected (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (4, encode_to_ecvector(ARRAY[-0.8, 0.2], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_recursive_maintenance_rejected_idx \
             ON ec_spire_recursive_maintenance_rejected \
             USING ec_spire (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 4, recursive_fanout = 2)",
        )
        .expect("recursive ec_spire index creation should succeed");

        Spi::run(
            "SELECT * FROM ec_spire_index_maintenance_run(\
             'ec_spire_recursive_maintenance_rejected_idx'::regclass)",
        )
        .expect("recursive maintenance should be rejected");
    }

    #[pg_test]
    fn test_ec_spire_maintenance_run_merge_publish_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_maintenance_run_merge_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_maintenance_run_merge_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_maintenance_run_merge_idx \
             ON ec_spire_maintenance_run_merge_sql \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 3)",
        )
        .expect("populated ec_spire index creation should succeed");

        let pre_merge_candidates = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_maintenance_run_merge_idx'::regclass) \
             WHERE merge_recommended",
        )
        .expect("leaf snapshot query should succeed")
        .expect("count row should exist");
        assert_eq!(pre_merge_candidates, 2);

        Spi::run(
            "CREATE TEMP TABLE ec_spire_maintenance_run_merge_result AS \
             SELECT * FROM \
             ec_spire_index_maintenance_run('ec_spire_maintenance_run_merge_idx'::regclass)",
        )
        .expect("maintenance run should publish merge epoch");

        let status = Spi::get_one::<String>(
            "SELECT maintenance_status FROM ec_spire_maintenance_run_merge_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("status row should exist");
        let action = Spi::get_one::<String>(
            "SELECT planned_action FROM ec_spire_maintenance_run_merge_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("action row should exist");
        let reason = Spi::get_one::<String>(
            "SELECT planned_reason FROM ec_spire_maintenance_run_merge_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("reason row should exist");
        let active_epoch_after = Spi::get_one::<i64>(
            "SELECT active_epoch_after FROM ec_spire_maintenance_run_merge_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("active epoch row should exist");
        let published =
            Spi::get_one::<bool>("SELECT published FROM ec_spire_maintenance_run_merge_result")
                .expect("maintenance run result query should succeed")
                .expect("published row should exist");
        let post_leaf_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_maintenance_run_merge_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("count row should exist");

        assert_eq!(status, "published");
        assert_eq!(action, "merge");
        assert_eq!(reason, "sparsest_same_parent_merge_pair");
        assert_eq!(active_epoch_after, 2);
        assert!(published);
        assert_eq!(post_leaf_count, 2);

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let post_merge_first_id = Spi::get_one::<i64>(
            "SELECT id FROM ec_spire_maintenance_run_merge_sql \
             ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
             LIMIT 1",
        )
        .expect("ordered post-merge ec_spire query should succeed")
        .expect("query should return a row");
        assert_eq!(post_merge_first_id, 1);

        Spi::run(
            "CREATE TEMP TABLE ec_spire_maintenance_run_merge_second_result AS \
             SELECT * FROM \
             ec_spire_index_maintenance_run('ec_spire_maintenance_run_merge_idx'::regclass)",
        )
        .expect("second maintenance run should return no action");

        let second_status = Spi::get_one::<String>(
            "SELECT maintenance_status FROM ec_spire_maintenance_run_merge_second_result",
        )
        .expect("second maintenance run result query should succeed")
        .expect("status row should exist");
        let second_reason = Spi::get_one::<String>(
            "SELECT planned_reason FROM ec_spire_maintenance_run_merge_second_result",
        )
        .expect("second maintenance run result query should succeed")
        .expect("reason row should exist");
        let second_active_epoch_before = Spi::get_one::<i64>(
            "SELECT active_epoch_before FROM ec_spire_maintenance_run_merge_second_result",
        )
        .expect("second maintenance run result query should succeed")
        .expect("active epoch before row should exist");
        let second_active_epoch_after = Spi::get_one::<i64>(
            "SELECT active_epoch_after FROM ec_spire_maintenance_run_merge_second_result",
        )
        .expect("second maintenance run result query should succeed")
        .expect("active epoch after row should exist");
        let second_published = Spi::get_one::<bool>(
            "SELECT published FROM ec_spire_maintenance_run_merge_second_result",
        )
        .expect("second maintenance run result query should succeed")
        .expect("published row should exist");
        let second_post_leaf_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_maintenance_run_merge_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("count row should exist");

        assert_eq!(second_status, "no_action");
        assert_eq!(second_reason, "no_candidate");
        assert_eq!(second_active_epoch_before, 2);
        assert_eq!(second_active_epoch_after, 2);
        assert!(!second_published);
        assert_eq!(second_post_leaf_count, 2);
    }

    #[pg_test]
    fn test_ec_spire_maintenance_run_split_publish_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_maintenance_run_split_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_maintenance_run_split_sql (id, embedding) \
             SELECT gs, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42) \
             FROM generate_series(1, 60) AS gs",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_maintenance_run_split_idx \
             ON ec_spire_maintenance_run_split_sql \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 10)",
        )
        .expect("populated ec_spire index creation should succeed");

        let pre_split_candidates = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_maintenance_run_split_idx'::regclass) \
             WHERE split_recommended",
        )
        .expect("leaf snapshot query should succeed")
        .expect("count row should exist");
        assert_eq!(pre_split_candidates, 1);

        Spi::run(
            "CREATE TEMP TABLE ec_spire_maintenance_run_split_result AS \
             SELECT * FROM \
             ec_spire_index_maintenance_run('ec_spire_maintenance_run_split_idx'::regclass)",
        )
        .expect("maintenance run should publish split epoch");

        let status = Spi::get_one::<String>(
            "SELECT maintenance_status FROM ec_spire_maintenance_run_split_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("status row should exist");
        let action = Spi::get_one::<String>(
            "SELECT planned_action FROM ec_spire_maintenance_run_split_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("action row should exist");
        let active_epoch_after = Spi::get_one::<i64>(
            "SELECT active_epoch_after FROM ec_spire_maintenance_run_split_result",
        )
        .expect("maintenance run result query should succeed")
        .expect("active epoch row should exist");
        let published =
            Spi::get_one::<bool>("SELECT published FROM ec_spire_maintenance_run_split_result")
                .expect("maintenance run result query should succeed")
                .expect("published row should exist");
        let post_leaf_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_maintenance_run_split_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("count row should exist");

        assert_eq!(status, "published");
        assert_eq!(action, "split");
        assert_eq!(active_epoch_after, 2);
        assert!(published);
        assert_eq!(post_leaf_count, 11);

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let post_split_rows_returned = Spi::get_one::<i64>(
            "SELECT count(*) FROM ( \
                 SELECT id FROM ec_spire_maintenance_run_split_sql \
                 ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
                 LIMIT 20 \
             ) ranked",
        )
        .expect("ordered post-split ec_spire query should succeed")
        .expect("count should exist");
        assert_eq!(post_split_rows_returned, 20);
    }


    #[pg_test]
    fn test_ec_spire_vacuum_delete_delta_suppresses_visible_row() {
        Spi::run("CREATE TABLE ec_spire_vacuum_delta (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_vacuum_delta (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_vacuum_delta_idx ON ec_spire_vacuum_delta \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let index_oid = index_oid("ec_spire_vacuum_delta_idx");
        let deleted_tid = heap_tid_for_row("ec_spire_vacuum_delta", 2);
        let stats = unsafe { am::debug_spire_vacuum_remove_heap_tids(index_oid, &[deleted_tid]) };

        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 1.0);
        let (active_epoch, next_pid, next_local_vec_seq) =
            unsafe { am::debug_spire_root_control(index_oid) };
        assert_eq!(active_epoch, 3);
        assert_eq!(next_pid, 5);
        assert_eq!(next_local_vec_seq, 3);
        let leaf_assignment_count = Spi::get_one::<i64>(
            "SELECT leaf_assignment_count FROM \
             ec_spire_index_active_snapshot_diagnostics('ec_spire_vacuum_delta_idx'::regclass)",
        )
        .expect("diagnostics query should succeed")
        .expect("diagnostics row should exist");
        let delta_object_count = Spi::get_one::<i64>(
            "SELECT delta_object_count FROM \
             ec_spire_index_active_snapshot_diagnostics('ec_spire_vacuum_delta_idx'::regclass)",
        )
        .expect("diagnostics query should succeed")
        .expect("diagnostics row should exist");
        assert_eq!(leaf_assignment_count, 1);
        assert_eq!(delta_object_count, 0);

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let first_id = Spi::get_one::<i64>(
            "SELECT id FROM ec_spire_vacuum_delta \
             ORDER BY embedding <#> ARRAY[0.0, 1.0]::real[] \
             LIMIT 1",
        )
        .expect("ordered ec_spire query should succeed")
        .expect("query should return a row");
        assert_eq!(first_id, 1);
    }

    #[pg_test]
    fn test_ec_spire_vacuum_cleanup_no_delta_is_noop() {
        Spi::run(
            "CREATE TABLE ec_spire_vacuum_no_delta (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_vacuum_no_delta (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_vacuum_no_delta_idx ON ec_spire_vacuum_no_delta \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let index_oid = index_oid("ec_spire_vacuum_no_delta_idx");
        let stats = unsafe { am::debug_spire_vacuum_remove_heap_tids(index_oid, &[]) };

        assert_eq!(stats.tuples_removed, 0.0);
        assert_eq!(stats.num_index_tuples, 2.0);
        let (active_epoch, next_pid, next_local_vec_seq) =
            unsafe { am::debug_spire_root_control(index_oid) };
        assert_eq!(active_epoch, 1);
        assert_eq!(next_pid, 4);
        assert_eq!(next_local_vec_seq, 3);
        assert_eq!(
            ec_spire_active_snapshot_i64("ec_spire_vacuum_no_delta_idx", "leaf_assignment_count"),
            2
        );
        assert_eq!(
            ec_spire_active_snapshot_i64("ec_spire_vacuum_no_delta_idx", "delta_object_count"),
            0
        );
    }

    #[pg_test]
    fn test_ec_spire_vacuum_cleanup_compacts_insert_delta() {
        Spi::run(
            "CREATE TABLE ec_spire_vacuum_insert_delta \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_vacuum_insert_delta (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_vacuum_insert_delta_idx ON ec_spire_vacuum_insert_delta \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_vacuum_insert_delta (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("post-build insert should publish a delta epoch");

        let index_oid = index_oid("ec_spire_vacuum_insert_delta_idx");
        assert_eq!(
            ec_spire_active_snapshot_i64("ec_spire_vacuum_insert_delta_idx", "delta_object_count"),
            1
        );
        assert_eq!(
            ec_spire_active_snapshot_i64(
                "ec_spire_vacuum_insert_delta_idx",
                "delta_assignment_count"
            ),
            1
        );

        let stats = unsafe { am::debug_spire_vacuum_remove_heap_tids(index_oid, &[]) };

        assert_eq!(stats.tuples_removed, 0.0);
        assert_eq!(stats.num_index_tuples, 3.0);
        let (active_epoch, next_pid, next_local_vec_seq) =
            unsafe { am::debug_spire_root_control(index_oid) };
        assert_eq!(active_epoch, 3);
        assert_eq!(next_pid, 5);
        assert_eq!(next_local_vec_seq, 4);
        assert_eq!(
            ec_spire_active_snapshot_i64(
                "ec_spire_vacuum_insert_delta_idx",
                "leaf_assignment_count"
            ),
            3
        );
        assert_eq!(
            ec_spire_active_snapshot_i64("ec_spire_vacuum_insert_delta_idx", "delta_object_count"),
            0
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let first_id = Spi::get_one::<i64>(
            "SELECT id FROM ec_spire_vacuum_insert_delta \
             ORDER BY embedding <#> ARRAY[0.0, 1.0]::real[] \
             LIMIT 1",
        )
        .expect("ordered ec_spire query should succeed")
        .expect("query should return a row");
        assert_eq!(first_id, 3);
    }

    #[pg_test]
    fn test_ec_spire_vacuum_cleanup_compacts_mixed_delta_on_leaf() {
        Spi::run(
            "CREATE TABLE ec_spire_vacuum_mixed_delta \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_vacuum_mixed_delta (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_vacuum_mixed_delta_idx ON ec_spire_vacuum_mixed_delta \
             USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 1)",
        )
        .expect("populated ec_spire index creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_vacuum_mixed_delta (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[1.0, 0.1], 4, 42))",
        )
        .expect("post-build insert should publish an insert-delta epoch");

        let index_oid = index_oid("ec_spire_vacuum_mixed_delta_idx");
        let deleted_tid = heap_tid_for_row("ec_spire_vacuum_mixed_delta", 1);
        let stats = unsafe { am::debug_spire_vacuum_remove_heap_tids(index_oid, &[deleted_tid]) };

        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 2.0);
        let (active_epoch, next_pid, next_local_vec_seq) =
            unsafe { am::debug_spire_root_control(index_oid) };
        assert_eq!(active_epoch, 4);
        assert_eq!(next_pid, 5);
        assert_eq!(next_local_vec_seq, 4);
        assert_eq!(
            ec_spire_active_snapshot_i64(
                "ec_spire_vacuum_mixed_delta_idx",
                "leaf_assignment_count"
            ),
            2
        );
        assert_eq!(
            ec_spire_active_snapshot_i64("ec_spire_vacuum_mixed_delta_idx", "delta_object_count"),
            0
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let first_id = Spi::get_one::<i64>(
            "SELECT id FROM ec_spire_vacuum_mixed_delta \
             ORDER BY embedding <#> ARRAY[1.0, 0.1]::real[] \
             LIMIT 1",
        )
        .expect("ordered ec_spire query should succeed")
        .expect("query should return a row");
        assert_eq!(first_id, 3);
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_ec_spire_sql_vacuum_mixed_delta() {
        const TABLE_NAME: &str = "ec_spire_sql_vacuum_mixed_delta";
        const INDEX_NAME: &str = "ec_spire_sql_vacuum_mixed_delta_idx";

        let connection = pg_test_psql_connection();
        run_psql_script(
            &connection,
            "ec_spire SQL vacuum mixed-delta setup",
            &format!(
                "DROP TABLE IF EXISTS {TABLE_NAME};
                 CREATE TABLE {TABLE_NAME} (id bigint primary key, embedding ecvector);
                 INSERT INTO {TABLE_NAME} (id, embedding) VALUES
                   (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
                   (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42));
                 CREATE INDEX {INDEX_NAME} ON {TABLE_NAME} USING ec_spire
                   (embedding ecvector_spire_ip_ops)
                   WITH (nlists = 1, nprobe = 1, training_sample_rows = 2);",
            ),
        );
        run_psql_script(
            &connection,
            "ec_spire SQL vacuum mixed-delta insert",
            &format!(
                "INSERT INTO {TABLE_NAME} (id, embedding)
                 VALUES (3, encode_to_ecvector(ARRAY[1.0, 0.1], 4, 42));",
            ),
        );
        run_psql_script(
            &connection,
            "ec_spire SQL vacuum mixed-delta delete",
            &format!("DELETE FROM {TABLE_NAME} WHERE id = 1;"),
        );
        run_psql_script(
            &connection,
            "ec_spire SQL vacuum mixed-delta VACUUM",
            &format!("VACUUM {TABLE_NAME};"),
        );

        let heap_count = Spi::get_one::<i64>(&format!("SELECT count(*) FROM {TABLE_NAME}"))
            .expect("SPI query should succeed")
            .expect("heap count should exist");
        let index_oid = index_oid(INDEX_NAME);
        let (active_epoch, next_pid, next_local_vec_seq) =
            unsafe { am::debug_spire_root_control(index_oid) };
        assert_eq!(heap_count, 2);
        assert_eq!(active_epoch, 3);
        assert_eq!(next_pid, 4);
        assert_eq!(next_local_vec_seq, 4);
        assert_eq!(
            ec_spire_active_snapshot_i64(INDEX_NAME, "leaf_assignment_count"),
            3
        );
        assert_eq!(
            ec_spire_active_snapshot_i64(INDEX_NAME, "delta_object_count"),
            0
        );
        assert_eq!(
            ec_spire_active_snapshot_i64(INDEX_NAME, "delta_assignment_count"),
            0
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let returned_deleted_row = Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM ( \
                 SELECT id FROM {TABLE_NAME} \
                 ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] \
                 LIMIT 2 \
             ) ranked WHERE id = 1"
        ))
        .expect("ordered ec_spire query should succeed")
        .expect("count should exist");
        let inserted_row_returned = Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM ( \
                 SELECT id FROM {TABLE_NAME} \
                 ORDER BY embedding <#> ARRAY[1.0, 0.1]::real[] \
                 LIMIT 2 \
             ) ranked WHERE id = 3"
        ))
        .expect("ordered ec_spire query should succeed")
        .expect("count should exist");
        assert_eq!(returned_deleted_row, 0);
        assert_eq!(inserted_row_returned, 1);
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores() {
        const TABLE_NAME: &str = "ec_spire_multistore_sql_vacuum";
        const INDEX_NAME: &str = "ec_spire_multistore_sql_vacuum_idx";

        let connection = pg_test_psql_connection();
        run_psql_script(
            &connection,
            "ec_spire multistore SQL vacuum setup",
            &format!(
                "DROP TABLE IF EXISTS {TABLE_NAME};
                 CREATE TABLE {TABLE_NAME} (id bigint primary key, embedding ecvector);
                 INSERT INTO {TABLE_NAME} (id, embedding) VALUES
                   (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
                   (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)),
                   (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)),
                   (4, encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42));
                 CREATE INDEX {INDEX_NAME} ON {TABLE_NAME} USING ec_spire
                   (embedding ecvector_spire_ip_ops)
                   WITH (
                       nlists = 2,
                       nprobe = 2,
                       training_sample_rows = 4,
                       local_store_count = 2,
                       local_store_tablespaces = 'pg_default,pg_default'
                   );",
            ),
        );
        run_psql_script(
            &connection,
            "ec_spire multistore SQL vacuum insert",
            &format!(
                "INSERT INTO {TABLE_NAME} (id, embedding)
                 VALUES (5, encode_to_ecvector(ARRAY[0.9, 0.1], 4, 42));",
            ),
        );
        run_psql_script(
            &connection,
            "ec_spire multistore SQL vacuum delete",
            &format!("DELETE FROM {TABLE_NAME} WHERE id = 5;"),
        );
        run_psql_script(
            &connection,
            "ec_spire multistore SQL vacuum VACUUM",
            &format!("VACUUM {TABLE_NAME};"),
        );

        // Use SPIRE diagnostic snapshots rather than broad heap SELECTs here:
        // the coordinator DML frontdoor intentionally fail-closes unsupported
        // broad SELECTs on ec_spire-indexed tables, which would obscure the
        // VACUUM placement assertions this fixture owns.
        let placement_store_count = Spi::get_one::<i64>(&format!(
            "SELECT count(DISTINCT local_store_id) FROM \
             ec_spire_index_placement_snapshot('{INDEX_NAME}'::regclass)"
        ))
        .expect("placement snapshot should succeed")
        .expect("count should exist");
        let placement_node_store_count = Spi::get_one::<i64>(&format!(
            "SELECT count(DISTINCT (node_id, local_store_id)) FROM \
             ec_spire_index_placement_snapshot('{INDEX_NAME}'::regclass)"
        ))
        .expect("placement snapshot should succeed")
        .expect("count should exist");
        let placement_store_relid_count = Spi::get_one::<i64>(&format!(
            "SELECT count(DISTINCT store_relid) FROM \
             ec_spire_index_placement_snapshot('{INDEX_NAME}'::regclass)"
        ))
        .expect("placement snapshot should succeed")
        .expect("count should exist");
        let placement_store_key_count = Spi::get_one::<i64>(&format!(
            "SELECT count(DISTINCT (node_id, local_store_id, store_relid)) FROM \
             ec_spire_index_placement_snapshot('{INDEX_NAME}'::regclass)"
        ))
        .expect("placement snapshot should succeed")
        .expect("count should exist");
        let scan_selected_store_count = Spi::get_one::<i64>(&format!(
            "SELECT count(DISTINCT local_store_id) FROM \
             ec_spire_index_scan_placement_snapshot( \
                 '{INDEX_NAME}'::regclass, \
                 ARRAY[0.9, 0.1]::real[])"
        ))
        .expect("scan placement snapshot should succeed")
        .expect("count should exist");
        let scan_selected_node_store_count = Spi::get_one::<i64>(&format!(
            "SELECT count(DISTINCT (node_id, local_store_id)) FROM \
             ec_spire_index_scan_placement_snapshot( \
                 '{INDEX_NAME}'::regclass, \
                 ARRAY[0.9, 0.1]::real[])"
        ))
        .expect("scan placement snapshot should succeed")
        .expect("count should exist");
        let scan_selected_nonlocal_node_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM \
             ec_spire_index_scan_placement_snapshot( \
                 '{INDEX_NAME}'::regclass, \
                 ARRAY[0.9, 0.1]::real[]) \
             WHERE node_id <> 0"
        ))
        .expect("scan placement snapshot should succeed")
        .expect("count should exist");
        let delta_object_count = ec_spire_active_snapshot_i64(INDEX_NAME, "delta_object_count");
        let delta_assignment_count =
            ec_spire_active_snapshot_i64(INDEX_NAME, "delta_assignment_count");

        assert_eq!(placement_store_count, 2);
        assert_eq!(placement_node_store_count, 2);
        assert_eq!(placement_store_relid_count, 2);
        assert_eq!(placement_store_key_count, 2);
        assert!(scan_selected_store_count >= 1);
        assert_eq!(scan_selected_node_store_count, scan_selected_store_count);
        assert_eq!(scan_selected_nonlocal_node_count, 0);
        assert_eq!(delta_object_count, 0);
        assert_eq!(delta_assignment_count, 0);
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_ec_spire_concurrent_insert_vacuum_scan() {
        const TABLE_NAME: &str = "ec_spire_concurrent_insert_vacuum_scan";
        const INDEX_NAME: &str = "ec_spire_concurrent_insert_vacuum_scan_idx";
        // Test-unique advisory-lock id; conventionally `<review-packet>0`.
        const BARRIER_KEY: i64 = 303_520;

        let connection = pg_test_psql_connection();
        run_psql_script(
            &connection,
            "ec_spire heterogeneous concurrency setup",
            &format!(
                "DROP TABLE IF EXISTS {TABLE_NAME};
                 CREATE TABLE {TABLE_NAME} (id bigint primary key, embedding ecvector);
                 INSERT INTO {TABLE_NAME} (id, embedding)
                 VALUES (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42));
                 CREATE INDEX {INDEX_NAME} ON {TABLE_NAME} USING ec_spire
                   (embedding ecvector_spire_ip_ops)
                   WITH (nlists = 1, nprobe = 1, training_sample_rows = 1);
                 INSERT INTO {TABLE_NAME} (id, embedding)
                 VALUES (2, encode_to_ecvector(ARRAY[1.0, 0.1], 4, 42));
                 DELETE FROM {TABLE_NAME} WHERE id = 1;",
            ),
        );

        Spi::run(&format!("SELECT pg_advisory_lock({BARRIER_KEY})"))
            .expect("barrier lock should be acquired");
        let barrier_sql = format!(
            "SET lock_timeout = '10s';
             SET statement_timeout = '30s';
             SELECT pg_advisory_lock_shared({BARRIER_KEY});
             SELECT pg_advisory_unlock_shared({BARRIER_KEY});"
        );
        let scan_retry_sql = |query: &str| {
            format!(
                "SET enable_seqscan = off;
                 DO $$
                 DECLARE
                     attempt integer;
                     scan_count bigint;
                 BEGIN
                     FOR attempt IN 1..5 LOOP
                         BEGIN
                             SELECT count(*) INTO scan_count FROM (
                                 SELECT id FROM {TABLE_NAME}
                                 ORDER BY embedding <#> {query}::real[]
                                 LIMIT 3
                             ) ranked;
                             IF scan_count > 0 THEN
                                 RETURN;
                             END IF;
                             RAISE EXCEPTION 'ec_spire scan returned no rows';
                         EXCEPTION WHEN OTHERS THEN
                             IF SQLERRM LIKE 'ec_spire remote search target plan requested epoch % does not match active epoch %' THEN
                                 PERFORM pg_sleep(0.05);
                             ELSE
                                 RAISE;
                             END IF;
                         END;
                     END LOOP;
                     RAISE EXCEPTION 'ec_spire scan did not stabilize after epoch mismatch retries';
                 END $$;"
            )
        };
        let workers = vec![
            (
                "spire heterogeneous insert worker",
                spawn_psql_commands(
                    &connection,
                    "spire heterogeneous insert worker",
                    &[
                        barrier_sql.clone(),
                        format!(
                            "INSERT INTO {TABLE_NAME} (id, embedding)
                             VALUES (3, encode_to_ecvector(ARRAY[1.0, 0.2], 4, 42));"
                        ),
                    ],
                ),
            ),
            (
                "spire heterogeneous vacuum worker",
                spawn_psql_commands(
                    &connection,
                    "spire heterogeneous vacuum worker",
                    &[barrier_sql.clone(), format!("VACUUM {TABLE_NAME};")],
                ),
            ),
            (
                "spire heterogeneous scan worker",
                spawn_psql_commands(
                    &connection,
                    "spire heterogeneous scan worker",
                    &[
                        barrier_sql,
                        scan_retry_sql("ARRAY[1.0, 0.2]"),
                        scan_retry_sql("ARRAY[1.0, 0.1]"),
                    ],
                ),
            ),
        ];
        wait_for_advisory_lock_waiters(BARRIER_KEY, 3);
        Spi::run(&format!("SELECT pg_advisory_unlock({BARRIER_KEY})"))
            .expect("barrier lock should be released");

        for (label, worker) in workers {
            let output = worker
                .wait_with_output()
                .unwrap_or_else(|e| panic!("{label} wait failed: {e}"));
            assert_psql_success(label, output);
        }
        let heap_count = Spi::get_one::<i64>(&format!("SELECT count(*) FROM {TABLE_NAME}"))
            .expect("SPI query should succeed")
            .expect("heap count should exist");
        let index_oid = index_oid(INDEX_NAME);
        let (active_epoch, next_pid, next_local_vec_seq) =
            unsafe { am::debug_spire_root_control(index_oid) };
        assert_eq!(heap_count, 3);
        assert_eq!(active_epoch, 4);
        assert_eq!(next_pid, 5);
        assert_eq!(next_local_vec_seq, 4);

        let leaf_assignment_count =
            ec_spire_active_snapshot_i64(INDEX_NAME, "leaf_assignment_count");
        let delta_object_count = ec_spire_active_snapshot_i64(INDEX_NAME, "delta_object_count");
        let delta_assignment_count =
            ec_spire_active_snapshot_i64(INDEX_NAME, "delta_assignment_count");
        assert_eq!(leaf_assignment_count + delta_assignment_count, 3);
        assert!(delta_object_count <= 1);
        assert!(delta_assignment_count <= 1);

        run_psql_script(
            &connection,
            "spire heterogeneous post-concurrency visibility check",
            &format!(
                "SET enable_seqscan = off;
                 DO $$
                 DECLARE
                     visible_live_rows bigint;
                 BEGIN
                     SELECT count(*) INTO visible_live_rows FROM (
                         SELECT id FROM {TABLE_NAME}
                         ORDER BY embedding <#> ARRAY[1.0, 0.2]::real[]
                         LIMIT 3
                     ) ranked WHERE id IN (2, 3);
                     IF visible_live_rows <> 2 THEN
                         RAISE EXCEPTION 'live row count after concurrent VACUUM was %',
                             visible_live_rows;
                     END IF;
                 END $$;"
            ),
        );
    }
