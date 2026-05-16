    #[pg_test]
    fn test_ec_spire_remote_conninfo_secret_resolution_status() {
        let _env_lock = env_var_test_lock();
        let _missing_secret = ScopedEnvVar {
            key: "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STATUS",
            previous: std::env::var_os("EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STATUS"),
        };
        std::env::remove_var("EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STATUS");

        let missing_from =
            "FROM ec_spire_remote_conninfo_secret_resolution_status('spire/remote/status')";
        let missing_lookup_key =
            Spi::get_one::<String>(&format!("SELECT provider_lookup_key {missing_from}"))
                .expect("missing secret lookup key query should succeed")
                .expect("missing secret lookup key should exist");
        let missing_status = Spi::get_one::<String>(&format!("SELECT status {missing_from}"))
            .expect("missing secret status query should succeed")
            .expect("missing secret status should exist");
        let missing_raw_exposed =
            Spi::get_one::<bool>(&format!("SELECT raw_conninfo_exposed {missing_from}"))
                .expect("missing secret exposure query should succeed")
                .expect("missing secret exposure should exist");

        assert_eq!(
            missing_lookup_key,
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STATUS"
        );
        assert_eq!(missing_status, "requires_conninfo_secret_resolution");
        assert!(!missing_raw_exposed);

        let _resolved_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STATUS",
            "host=remote.example.invalid dbname=ecaz",
        );
        let resolved_from =
            "FROM ec_spire_remote_conninfo_secret_resolution_status('spire/remote/status')";
        let resolved_status = Spi::get_one::<String>(&format!("SELECT status {resolved_from}"))
            .expect("resolved secret status query should succeed")
            .expect("resolved secret status should exist");
        let resolved_bytes =
            Spi::get_one::<i64>(&format!("SELECT resolved_conninfo_bytes {resolved_from}"))
                .expect("resolved secret byte query should succeed")
                .expect("resolved secret byte count should exist");
        let resolved_raw_exposed =
            Spi::get_one::<bool>(&format!("SELECT raw_conninfo_exposed {resolved_from}"))
                .expect("resolved secret exposure query should succeed")
                .expect("resolved secret exposure should exist");

        assert_eq!(resolved_status, "resolved_conninfo");
        assert!(resolved_bytes > 0);
        assert!(!resolved_raw_exposed);
    }

    #[pg_test]
    fn test_ec_spire_remote_catalog_orphan_cleanup() {
        Spi::run("SELECT * FROM ec_spire_remote_catalog_orphan_cleanup()")
            .expect("initial orphan cleanup should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_node_descriptor \
             (coordinator_index_oid, node_id, descriptor_generation, conninfo_secret_name, \
              remote_index_identity, remote_index_regclass, descriptor_state, \
              last_served_epoch, min_retained_epoch, extension_version, last_error) \
             VALUES ('4294967294'::oid, 2, 1, 'spire/remote/orphan', '\\x01'::bytea, \
                     'orphan_idx', 'active', 1, 1, 'test', 'none')",
        )
        .expect("orphan descriptor insert should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_epoch_manifest \
             (coordinator_index_oid, active_epoch, manifest_scope, manifest_decision, \
              manifest_entry_count, included_remote_node_count, remote_placement_count, \
              publish_decision, status, persisted_at_micros) \
             VALUES ('4294967294'::oid, 1, 'distributed', \
                     'emit_distributed_epoch_manifest', 1, 1, 1, \
                     'publish_remote_epoch_manifest', 'ready', 1)",
        )
        .expect("orphan manifest insert should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_epoch_manifest_entry \
             (coordinator_index_oid, active_epoch, node_id, descriptor_state, placement_count, \
              required_last_served_epoch, required_min_retained_epoch, last_served_epoch, \
              min_retained_epoch, epoch_window_status, manifest_action, status) \
             VALUES ('4294967294'::oid, 1, 2, 'active', 1, 1, 1, 1, 1, \
                     'ready', 'include_remote_node', 'ready')",
        )
        .expect("orphan manifest entry insert should succeed");
        Spi::run(
            "INSERT INTO ec_spire_placement \
             (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
             VALUES ('4294967294'::oid, decode('01', 'hex'), 2, 7, 1, \
                     decode('000102030405060708090a0b0c0d0e0f', 'hex'))",
        )
        .expect("orphan placement insert should succeed");

        let summary_from = "FROM ec_spire_remote_catalog_orphan_summary()";
        let cleanup_from = "FROM ec_spire_remote_catalog_orphan_cleanup()";
        let descriptor_orphan_count =
            Spi::get_one::<i64>(&format!("SELECT descriptor_orphan_count {summary_from}"))
                .expect("descriptor orphan count query should succeed")
                .expect("descriptor orphan count should exist");
        let manifest_orphan_count =
            Spi::get_one::<i64>(&format!("SELECT manifest_orphan_count {summary_from}"))
                .expect("manifest orphan count query should succeed")
                .expect("manifest orphan count should exist");
        let manifest_entry_orphan_count = Spi::get_one::<i64>(&format!(
            "SELECT manifest_entry_orphan_count {summary_from}"
        ))
        .expect("manifest entry orphan count query should succeed")
        .expect("manifest entry orphan count should exist");
        let row_materialization_orphan_count = Spi::get_one::<i64>(&format!(
            "SELECT row_materialization_orphan_count {summary_from}"
        ))
        .expect("row materialization orphan count query should succeed")
        .expect("row materialization orphan count should exist");
        let placement_orphan_count =
            Spi::get_one::<i64>(&format!("SELECT placement_orphan_count {summary_from}"))
                .expect("placement orphan count query should succeed")
                .expect("placement orphan count should exist");
        let summary_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("orphan summary status query should succeed")
            .expect("orphan summary status should exist");

        let cleanup_counts = Spi::get_one::<String>(&format!(
            "SELECT descriptor_removed_count::text || ',' || \
                    manifest_removed_count::text || ',' || \
                    manifest_entry_removed_count::text || ',' || \
                    row_materialization_removed_count::text || ',' || \
                    placement_removed_count::text \
               {cleanup_from}"
        ))
        .expect("orphan cleanup count query should succeed")
        .expect("orphan cleanup counts should exist");
        let cleanup_counts = cleanup_counts
            .split(',')
            .map(|value| {
                value
                    .parse::<i64>()
                    .expect("cleanup count should parse as i64")
            })
            .collect::<Vec<_>>();
        let post_cleanup_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("post-cleanup summary status query should succeed")
            .expect("post-cleanup summary status should exist");

        assert_eq!(descriptor_orphan_count, 1);
        assert_eq!(manifest_orphan_count, 1);
        assert_eq!(manifest_entry_orphan_count, 1);
        assert_eq!(row_materialization_orphan_count, 0);
        assert_eq!(placement_orphan_count, 1);
        assert_eq!(summary_status, "orphaned_remote_catalog_rows");
        assert_eq!(cleanup_counts, vec![1, 1, 1, 0, 1]);
        assert_eq!(post_cleanup_status, "ready");
    }

    #[pg_test]
    fn test_ec_spire_remote_catalog_index_cleanup() {
        Spi::run(
            "INSERT INTO ec_spire_remote_node_descriptor \
             (coordinator_index_oid, node_id, descriptor_generation, conninfo_secret_name, \
              remote_index_identity, remote_index_regclass, descriptor_state, \
              last_served_epoch, min_retained_epoch, extension_version, last_error) \
             VALUES ('4294967293'::oid, 3, 1, 'spire/remote/index-cleanup', '\\x01'::bytea, \
                     'cleanup_idx', 'active', 1, 1, 'test', 'none')",
        )
        .expect("index cleanup descriptor insert should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_epoch_manifest \
             (coordinator_index_oid, active_epoch, manifest_scope, manifest_decision, \
              manifest_entry_count, included_remote_node_count, remote_placement_count, \
              publish_decision, status, persisted_at_micros) \
             VALUES ('4294967293'::oid, 1, 'distributed', \
                     'emit_distributed_epoch_manifest', 1, 1, 1, \
                     'publish_remote_epoch_manifest', 'ready', 1)",
        )
        .expect("index cleanup manifest insert should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_epoch_manifest_entry \
             (coordinator_index_oid, active_epoch, node_id, descriptor_state, placement_count, \
              required_last_served_epoch, required_min_retained_epoch, last_served_epoch, \
              min_retained_epoch, epoch_window_status, manifest_action, status) \
             VALUES ('4294967293'::oid, 1, 3, 'active', 1, 1, 1, 1, 1, \
                     'ready', 'include_remote_node', 'ready')",
        )
        .expect("index cleanup manifest entry insert should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_epoch_manifest_applied \
             (remote_index_oid, active_epoch, manifest_payload_format, manifest_scope, \
              manifest_decision, manifest_entry_count, included_remote_node_count, \
              remote_placement_count, publish_decision, status, applied_at_micros) \
             VALUES ('4294967293'::oid, 1, 'ec_spire_remote_epoch_manifest_v1', \
                     'distributed', 'emit_distributed_epoch_manifest', 1, 1, 1, \
                     'publish_remote_epoch_manifest', 'ready', 1)",
        )
        .expect("index cleanup applied manifest insert should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_epoch_manifest_applied_entry \
             (remote_index_oid, active_epoch, node_id, descriptor_state, placement_count, \
              required_last_served_epoch, required_min_retained_epoch, last_served_epoch, \
              min_retained_epoch, epoch_window_status, manifest_action, status) \
             VALUES ('4294967293'::oid, 1, 3, 'active', 1, 1, 1, 1, 1, \
                     'ready', 'include_remote_node', 'ready')",
        )
        .expect("index cleanup applied manifest entry insert should succeed");
        Spi::run(
            "INSERT INTO ec_spire_placement \
             (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
             VALUES ('4294967293'::oid, decode('01', 'hex'), 3, 7, 1, \
                     decode('000102030405060708090a0b0c0d0e0f', 'hex'))",
        )
        .expect("index cleanup placement insert should succeed");

        Spi::run(
            "CREATE TEMP TABLE ec_spire_remote_catalog_index_cleanup_result AS \
             SELECT * FROM ec_spire_remote_catalog_index_cleanup('4294967293'::oid)",
        )
        .expect("index cleanup result materialization should succeed");
        let cleanup_from = "FROM ec_spire_remote_catalog_index_cleanup_result";
        let cleanup_counts = Spi::get_one::<String>(&format!(
            "SELECT descriptor_removed_count::text || ',' || \
                    manifest_removed_count::text || ',' || \
                    manifest_entry_removed_count::text || ',' || \
                    row_materialization_removed_count::text || ',' || \
                    placement_removed_count::text || ',' || \
                    applied_manifest_removed_count::text || ',' || \
                    applied_manifest_entry_removed_count::text \
               {cleanup_from}"
        ))
        .expect("index cleanup count query should succeed")
        .expect("index cleanup counts should exist");
        let cleanup_status = Spi::get_one::<String>(&format!("SELECT status {cleanup_from}"))
            .expect("index cleanup status query should succeed")
            .expect("index cleanup status should exist");
        let post_cleanup_count = Spi::get_one::<i64>(
            "SELECT count(*) \
               FROM ec_spire_remote_node_descriptor \
              WHERE coordinator_index_oid = '4294967293'::oid",
        )
        .expect("post index cleanup descriptor query should succeed")
        .expect("post index cleanup descriptor count should exist");
        let post_applied_cleanup_count = Spi::get_one::<i64>(
            "SELECT count(*) \
               FROM ec_spire_remote_epoch_manifest_applied \
              WHERE remote_index_oid = '4294967293'::oid",
        )
        .expect("post index cleanup applied query should succeed")
        .expect("post index cleanup applied count should exist");

        let cleanup_counts = cleanup_counts
            .split(',')
            .map(|value| {
                value
                    .parse::<i64>()
                    .expect("index cleanup count should parse as i64")
            })
            .collect::<Vec<_>>();

        assert_eq!(cleanup_counts, vec![1, 1, 1, 0, 1, 1, 1]);
        assert_eq!(cleanup_status, "removed_index_remote_catalog_rows");
        assert_eq!(post_cleanup_count, 0);
        assert_eq!(post_applied_cleanup_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_remote_catalog_drop_index_event_cleanup() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_catalog_drop_event_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("drop event table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_catalog_drop_event_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("drop event insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_catalog_drop_event_sql_idx \
             ON ec_spire_remote_catalog_drop_event_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 1)",
        )
        .expect("drop event index creation should succeed");
        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_catalog_drop_event_sql_idx'::regclass::oid",
        )
        .expect("drop event index oid query should succeed")
        .expect("drop event index oid should exist");
        let index_oid_u32 = u32::from(index_oid);

        Spi::run(&format!(
            "INSERT INTO ec_spire_remote_node_descriptor \
             (coordinator_index_oid, node_id, descriptor_generation, conninfo_secret_name, \
              remote_index_identity, remote_index_regclass, descriptor_state, \
              last_served_epoch, min_retained_epoch, extension_version, last_error) \
             VALUES ('{index_oid_u32}'::oid, 4, 1, 'spire/remote/drop-event', '\\x01'::bytea, \
                     'drop_event_idx', 'active', 1, 1, 'test', 'none')"
        ))
        .expect("drop event descriptor insert should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_spire_remote_epoch_manifest \
             (coordinator_index_oid, active_epoch, manifest_scope, manifest_decision, \
              manifest_entry_count, included_remote_node_count, remote_placement_count, \
              publish_decision, status, persisted_at_micros) \
             VALUES ('{index_oid_u32}'::oid, 1, 'distributed', \
                     'emit_distributed_epoch_manifest', 1, 1, 1, \
                     'publish_remote_epoch_manifest', 'ready', 1)"
        ))
        .expect("drop event manifest insert should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_spire_remote_epoch_manifest_entry \
             (coordinator_index_oid, active_epoch, node_id, descriptor_state, placement_count, \
              required_last_served_epoch, required_min_retained_epoch, last_served_epoch, \
              min_retained_epoch, epoch_window_status, manifest_action, status) \
             VALUES ('{index_oid_u32}'::oid, 1, 4, 'active', 1, 1, 1, 1, 1, \
                     'ready', 'include_remote_node', 'ready')"
        ))
        .expect("drop event manifest entry insert should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_spire_remote_epoch_manifest_applied \
             (remote_index_oid, active_epoch, manifest_payload_format, manifest_scope, \
              manifest_decision, manifest_entry_count, included_remote_node_count, \
              remote_placement_count, publish_decision, status, applied_at_micros) \
             VALUES ('{index_oid_u32}'::oid, 1, 'ec_spire_remote_epoch_manifest_v1', \
                     'distributed', 'emit_distributed_epoch_manifest', 1, 1, 1, \
                     'publish_remote_epoch_manifest', 'ready', 1)"
        ))
        .expect("drop event applied manifest insert should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_spire_remote_epoch_manifest_applied_entry \
             (remote_index_oid, active_epoch, node_id, descriptor_state, placement_count, \
              required_last_served_epoch, required_min_retained_epoch, last_served_epoch, \
              min_retained_epoch, epoch_window_status, manifest_action, status) \
             VALUES ('{index_oid_u32}'::oid, 1, 4, 'active', 1, 1, 1, 1, 1, \
                     'ready', 'include_remote_node', 'ready')"
        ))
        .expect("drop event applied manifest entry insert should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_spire_placement \
             (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
             VALUES ('{index_oid_u32}'::oid, decode('01', 'hex'), 4, 7, 1, \
                     decode('000102030405060708090a0b0c0d0e0f', 'hex'))"
        ))
        .expect("drop event placement insert should succeed");

        Spi::run("DROP INDEX ec_spire_remote_catalog_drop_event_sql_idx")
            .expect("drop event index drop should succeed");
        let remaining_count = Spi::get_one::<i64>(&format!(
            "SELECT \
                (SELECT count(*) FROM ec_spire_remote_node_descriptor \
                  WHERE coordinator_index_oid = '{index_oid_u32}'::oid) + \
                (SELECT count(*) FROM ec_spire_remote_epoch_manifest \
                  WHERE coordinator_index_oid = '{index_oid_u32}'::oid) + \
                (SELECT count(*) FROM ec_spire_remote_epoch_manifest_entry \
                  WHERE coordinator_index_oid = '{index_oid_u32}'::oid) + \
                (SELECT count(*) FROM ec_spire_placement \
                  WHERE index_oid = '{index_oid_u32}'::oid) + \
                (SELECT count(*) FROM ec_spire_remote_epoch_manifest_applied \
                  WHERE remote_index_oid = '{index_oid_u32}'::oid) + \
                (SELECT count(*) FROM ec_spire_remote_epoch_manifest_applied_entry \
                  WHERE remote_index_oid = '{index_oid_u32}'::oid)"
        ))
        .expect("drop event remaining remote catalog query should succeed")
        .expect("drop event remaining remote catalog count should exist");
        let event_trigger_enabled = Spi::get_one::<bool>(
            "SELECT evtenabled <> 'D' \
               FROM pg_event_trigger \
              WHERE evtname = 'ec_spire_remote_catalog_drop_index_cleanup'",
        )
        .expect("drop event trigger enabled query should succeed")
        .expect("drop event trigger should exist");

        assert_eq!(remaining_count, 0);
        assert!(event_trigger_enabled);
    }

    #[pg_test]
    fn test_ec_spire_prod_consistency_policy_summary_mode_mismatch() {
        Spi::run(
            "CREATE TABLE ec_spire_prod_consistency_policy_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_prod_consistency_policy_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_prod_consistency_policy_idx \
             ON ec_spire_prod_consistency_policy_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_prod_consistency_policy_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");

        Spi::run("SET LOCAL ec_spire.remote_search_consistency_mode = 'degraded'")
            .expect("session consistency mode SET should succeed");
        let summary_from = format!(
            "FROM ec_spire_remote_search_production_policy_session_summary(\
                 'ec_spire_prod_consistency_policy_idx'::regclass, {active_epoch})"
        );
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("policy summary status query should succeed")
            .expect("policy summary status should exist");
        let failure_category =
            Spi::get_one::<String>(&format!("SELECT failure_category {summary_from}"))
                .expect("policy summary failure category query should succeed")
                .expect("policy summary failure category should exist");
        let failure_action =
            Spi::get_one::<String>(&format!("SELECT failure_action {summary_from}"))
                .expect("policy summary failure action query should succeed")
                .expect("policy summary failure action should exist");
        let consistency_mode_source =
            Spi::get_one::<String>(&format!("SELECT consistency_mode_source {summary_from}"))
                .expect("policy summary source query should succeed")
                .expect("policy summary source should exist");
        let requested_consistency_mode =
            Spi::get_one::<String>(&format!("SELECT requested_consistency_mode {summary_from}"))
                .expect("policy summary requested mode query should succeed")
                .expect("policy summary requested mode should exist");
        let active_consistency_mode =
            Spi::get_one::<String>(&format!("SELECT active_consistency_mode {summary_from}"))
                .expect("policy summary active mode query should succeed")
                .expect("policy summary active mode should exist");

        assert_eq!(status, "consistency_mode_mismatch");
        assert_eq!(failure_category, "consistency_mode_mismatch");
        assert_eq!(failure_action, "fail_closed");
        assert_eq!(
            consistency_mode_source,
            "ec_spire.remote_search_consistency_mode"
        );
        assert_eq!(requested_consistency_mode, "degraded");
        assert_eq!(active_consistency_mode, "strict");
    }

    #[pg_test]
    #[should_panic(
        expected = "requested consistency_mode 'degraded' does not match active epoch consistency mode 'strict'"
    )]
    fn test_ec_spire_remote_search_mode_mismatch() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_search_mode_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_search_mode_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_search_mode_sql_idx \
             ON ec_spire_remote_search_mode_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_search_mode_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_search_mode_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");

        Spi::run(&format!(
            "SELECT count(*) FROM ec_spire_remote_search(\
             'ec_spire_remote_search_mode_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}]::bigint[], 1, 'degraded')",
            selected_pids[0],
        ))
        .expect("remote search consistency mismatch should fail");
    }

    #[pg_test]
    #[should_panic(expected = "strict published snapshot requires available placement")]
    fn test_ec_spire_remote_search_strict_unavailable_leaf() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_search_unavailable_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_search_unavailable_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_search_unavailable_sql_idx \
             ON ec_spire_remote_search_unavailable_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_search_unavailable_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_search_unavailable_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_search_unavailable_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_placement_state(index_oid, selected_pid as u64, "unavailable")
        };
        Spi::run(&format!(
            "SELECT count(*) FROM ec_spire_remote_search(\
             'ec_spire_remote_search_unavailable_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'strict')",
        ))
        .expect("strict remote search over unavailable placement should fail");
    }

    #[pg_test]
    #[should_panic(expected = "stale")]
    fn test_ec_spire_remote_search_degraded_stale_leaf() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_search_stale_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_search_stale_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_search_stale_sql_idx \
             ON ec_spire_remote_search_stale_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_search_stale_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_search_stale_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_search_stale_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pid as u64, "stale");
        }
        Spi::run(&format!(
            "SELECT count(*) FROM ec_spire_remote_search(\
             'ec_spire_remote_search_stale_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'degraded')",
        ))
        .expect("degraded remote search over stale placement should fail");
    }


    #[pg_test]
    fn test_ec_spire_reaper_resolves_lost_prepare_ack_fixture() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_REAPER_LOST_ACK",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_reaper_lost_ack_remote; \
                 CREATE TABLE ec_spire_reaper_lost_ack_remote \
                     (id bigint primary key, embedding ecvector, source_identity bytea not null); \
                 INSERT INTO ec_spire_reaper_lost_ack_remote \
                     (id, embedding, source_identity) VALUES \
                 (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('55565758595a5b5c5d5e5f6061626364', 'hex')); \
                 CREATE INDEX ec_spire_reaper_lost_ack_remote_idx \
                     ON ec_spire_reaper_lost_ack_remote USING ec_spire \
                     (embedding ecvector_spire_ip_ops); \
                 DROP TABLE IF EXISTS ec_spire_reaper_lost_ack_payload; \
                 CREATE TABLE ec_spire_reaper_lost_ack_payload (id bigint primary key)",
            )
            .expect("loopback reaper fixture should be created");

        Spi::run(
            "CREATE TABLE ec_spire_reaper_lost_ack_coord \
             (id bigint primary key, embedding ecvector, source_identity bytea not null)",
        )
        .expect("reaper coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_reaper_lost_ack_coord \
                 (id, embedding, source_identity) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
              decode('65666768696a6b6c6d6e6f7071727374', 'hex'))",
        )
        .expect("reaper coordinator seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_reaper_lost_ack_coord_idx \
             ON ec_spire_reaper_lost_ack_coord USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("reaper coordinator index creation should succeed");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_reaper_lost_ack_coord_idx'::regclass)",
        )
        .expect("reaper active epoch query should succeed")
        .expect("reaper active epoch should exist");
        let coord_index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_reaper_lost_ack_coord_idx'::regclass::oid",
        )
        .expect("reaper index oid query should succeed")
        .expect("reaper index oid should exist");
        let remote_identity_hex = Spi::get_one::<String>(
            "SELECT profile_fingerprint \
               FROM ec_spire_remote_search_endpoint_identity(\
                    'ec_spire_reaper_lost_ack_remote_idx'::regclass::oid)",
        )
        .expect("reaper remote identity query should succeed")
        .expect("reaper remote identity should exist");
        Spi::run(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_reaper_lost_ack_coord_idx'::regclass, \
                 33, 37, 'spire/remote/reaper_lost_ack', \
                 decode('{remote_identity_hex}', 'hex'), \
                 'ec_spire_reaper_lost_ack_remote_idx', \
                 'active', {active_epoch}, {active_epoch}, '0.1.1', '')"
        ))
        .expect("reaper descriptor registration should succeed");

        let xid = 987_654_321_u64;
        let gid = format!(
            "ec_spire_insert_{}_33_{}_{}",
            u32::from(coord_index_oid),
            active_epoch,
            xid
        );
        for row in loopback_client
            .query("SELECT gid FROM pg_prepared_xacts WHERE gid = $1", &[&gid])
            .expect("lost-ack stale prepared lookup should succeed")
        {
            let stale_gid = row
                .try_get::<_, String>(0)
                .expect("stale prepared gid should decode");
            let _ = loopback_client.batch_execute(&format!(
                "ROLLBACK PREPARED '{}'",
                stale_gid.replace('\'', "''")
            ));
        }
        loopback_client
            .batch_execute(&format!(
                "BEGIN; \
                 INSERT INTO ec_spire_reaper_lost_ack_payload VALUES (1); \
                 PREPARE TRANSACTION '{}'",
                gid.replace('\'', "''")
            ))
            .expect("lost-ack fixture remote prepare should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_spire_remote_prepared_xact_intent \
                 (index_oid, node_id, served_epoch, xid, gid, intent_state) \
             VALUES ('{}'::oid, 33, {active_epoch}, {xid}, '{}', 'prepare_requested')",
            u32::from(coord_index_oid),
            gid.replace('\'', "''")
        ))
        .expect("lost-ack fixture intent insert should succeed");

        let action = Spi::get_one::<String>(&format!(
            "SELECT action \
               FROM ec_spire_reap_orphaned_remote_prepared_xacts(33) \
              WHERE gid = '{}'",
            gid.replace('\'', "''")
        ))
        .expect("lost-ack fixture reaper should run")
        .expect("lost-ack fixture reaper should return the prepared gid");
        assert_eq!(action, "rolled_back");
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM pg_prepared_xacts WHERE gid = $1",
                &[&gid],
            )
            .expect("lost-ack prepared count query should succeed")
            .try_get::<_, i64>(0)
            .expect("lost-ack prepared count should decode");
        assert_eq!(prepared_count, 0);
        let payload_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM ec_spire_reaper_lost_ack_payload WHERE id = 1",
                &[],
            )
            .expect("lost-ack payload count query should succeed")
            .try_get::<_, i64>(0)
            .expect("lost-ack payload count should decode");
        assert_eq!(payload_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_reaper_prepare_acked_vs_commit_local() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_REAPER_IN_DOUBT",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_reaper_in_doubt_remote; \
                 CREATE TABLE ec_spire_reaper_in_doubt_remote \
                     (id bigint primary key, embedding ecvector, source_identity bytea not null); \
                 INSERT INTO ec_spire_reaper_in_doubt_remote \
                     (id, embedding, source_identity) VALUES \
                 (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('75767778797a7b7c7d7e7f8081828384', 'hex')); \
                 CREATE INDEX ec_spire_reaper_in_doubt_remote_idx \
                     ON ec_spire_reaper_in_doubt_remote USING ec_spire \
                     (embedding ecvector_spire_ip_ops); \
                 DROP TABLE IF EXISTS ec_spire_reaper_in_doubt_payload; \
                 CREATE TABLE ec_spire_reaper_in_doubt_payload (id bigint primary key)",
            )
            .expect("loopback reaper in-doubt fixture should be created");

        Spi::run(
            "CREATE TABLE ec_spire_reaper_in_doubt_coord \
             (id bigint primary key, embedding ecvector, source_identity bytea not null)",
        )
        .expect("reaper in-doubt coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_reaper_in_doubt_coord \
                 (id, embedding, source_identity) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
              decode('85868788898a8b8c8d8e8f9091929394', 'hex'))",
        )
        .expect("reaper in-doubt coordinator seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_reaper_in_doubt_coord_idx \
             ON ec_spire_reaper_in_doubt_coord USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("reaper in-doubt coordinator index creation should succeed");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_reaper_in_doubt_coord_idx'::regclass)",
        )
        .expect("reaper in-doubt active epoch query should succeed")
        .expect("reaper in-doubt active epoch should exist");
        let coord_index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_reaper_in_doubt_coord_idx'::regclass::oid",
        )
        .expect("reaper in-doubt index oid query should succeed")
        .expect("reaper in-doubt index oid should exist");
        let remote_identity_hex = Spi::get_one::<String>(
            "SELECT profile_fingerprint \
               FROM ec_spire_remote_search_endpoint_identity(\
                    'ec_spire_reaper_in_doubt_remote_idx'::regclass::oid)",
        )
        .expect("reaper in-doubt remote identity query should succeed")
        .expect("reaper in-doubt remote identity should exist");
        Spi::run(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                 'ec_spire_reaper_in_doubt_coord_idx'::regclass, \
                 34, 37, 'spire/remote/reaper_in_doubt', \
                 decode('{remote_identity_hex}', 'hex'), \
                 'ec_spire_reaper_in_doubt_remote_idx', \
                 'active', {active_epoch}, {active_epoch}, '0.1.1', '')"
        ))
        .expect("reaper in-doubt descriptor registration should succeed");

        let prepare_acked_xid = 987_654_330_u64;
        let commit_local_xid = 987_654_331_u64;
        let prepare_acked_gid = format!(
            "ec_spire_insert_{}_34_{}_{}",
            u32::from(coord_index_oid),
            active_epoch,
            prepare_acked_xid
        );
        let commit_local_gid = format!(
            "ec_spire_insert_{}_34_{}_{}",
            u32::from(coord_index_oid),
            active_epoch,
            commit_local_xid
        );
        for gid in [&prepare_acked_gid, &commit_local_gid] {
            for row in loopback_client
                .query("SELECT gid FROM pg_prepared_xacts WHERE gid = $1", &[gid])
                .expect("stale in-doubt prepared lookup should succeed")
            {
                let stale_gid = row
                    .try_get::<_, String>(0)
                    .expect("stale in-doubt prepared gid should decode");
                let _ = loopback_client.batch_execute(&format!(
                    "ROLLBACK PREPARED '{}'",
                    stale_gid.replace('\'', "''")
                ));
            }
        }
        loopback_client
            .batch_execute(&format!(
                "BEGIN; \
                 INSERT INTO ec_spire_reaper_in_doubt_payload VALUES (1); \
                 PREPARE TRANSACTION '{}'; \
                 BEGIN; \
                 INSERT INTO ec_spire_reaper_in_doubt_payload VALUES (2); \
                 PREPARE TRANSACTION '{}'",
                prepare_acked_gid.replace('\'', "''"),
                commit_local_gid.replace('\'', "''")
            ))
            .expect("in-doubt fixture remote prepares should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_spire_remote_prepared_xact_intent \
                 (index_oid, node_id, served_epoch, xid, gid, intent_state) \
             VALUES \
                 ('{}'::oid, 34, {active_epoch}, {prepare_acked_xid}, '{}', 'prepare_acked'), \
                 ('{}'::oid, 34, {active_epoch}, {commit_local_xid}, '{}', 'commit_local')",
            u32::from(coord_index_oid),
            prepare_acked_gid.replace('\'', "''"),
            u32::from(coord_index_oid),
            commit_local_gid.replace('\'', "''")
        ))
        .expect("in-doubt fixture intent insert should succeed");

        let prepare_reaper_status = Spi::get_one::<String>(&format!(
            "SELECT intent_state || ':' || action || ':' || coordinator_xid_live::text \
               FROM ec_spire_reap_orphaned_remote_prepared_xacts(34) \
              WHERE gid = '{}'",
            prepare_acked_gid.replace('\'', "''")
        ))
        .expect("in-doubt prepare_acked reaper should run")
        .expect("in-doubt prepare_acked row should exist");
        let commit_reaper_status = Spi::get_one::<String>(&format!(
            "SELECT intent_state || ':' || action || ':' || coordinator_xid_live::text \
               FROM ec_spire_reap_orphaned_remote_prepared_xacts(34) \
              WHERE gid = '{}'",
            commit_local_gid.replace('\'', "''")
        ))
        .expect("in-doubt commit_local reaper should run")
        .expect("in-doubt commit_local row should exist");
        assert_eq!(prepare_reaper_status, "prepare_acked:rolled_back:false");
        assert_eq!(
            commit_reaper_status,
            "commit_local:skipped_commit_local:false"
        );

        let prepare_prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM pg_prepared_xacts WHERE gid = $1",
                &[&prepare_acked_gid],
            )
            .expect("prepare_acked prepared count query should succeed")
            .try_get::<_, i64>(0)
            .expect("prepare_acked prepared count should decode");
        let commit_prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM pg_prepared_xacts WHERE gid = $1",
                &[&commit_local_gid],
            )
            .expect("commit_local prepared count query should succeed")
            .try_get::<_, i64>(0)
            .expect("commit_local prepared count should decode");
        assert_eq!(prepare_prepared_count, 0);
        assert_eq!(commit_prepared_count, 1);

        let prepare_intent_state = Spi::get_one::<String>(&format!(
            "SELECT intent_state \
               FROM ec_spire_remote_prepared_xact_intent \
              WHERE gid = '{}'",
            prepare_acked_gid.replace('\'', "''")
        ))
        .expect("prepare_acked intent state query should succeed")
        .expect("prepare_acked intent state should exist");
        let commit_intent_state = Spi::get_one::<String>(&format!(
            "SELECT intent_state \
               FROM ec_spire_remote_prepared_xact_intent \
              WHERE gid = '{}'",
            commit_local_gid.replace('\'', "''")
        ))
        .expect("commit_local intent state query should succeed")
        .expect("commit_local intent state should exist");
        assert_eq!(prepare_intent_state, "rollback_local");
        assert_eq!(commit_intent_state, "commit_local");

        loopback_client
            .batch_execute(&format!(
                "ROLLBACK PREPARED '{}'",
                commit_local_gid.replace('\'', "''")
            ))
            .expect("commit_local preserved prepared transaction cleanup should succeed");
    }

    #[pg_test]
    fn test_ec_spire_remote_pk_select_isolation_contract_sql() {
        let _env_lock = env_var_test_lock();
        const SECRET_KEY: &str = "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_ISOLATION_PK_SELECT";
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut setup_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback setup connection should succeed");
        setup_client
            .execute(
                "SELECT tests.ec_spire_test_set_env_var($1::text, $2::text)",
                &[&SECRET_KEY, &loopback_conninfo],
            )
            .expect("setup backend should receive conninfo secret env var");
        setup_client
            .batch_execute(
                "DO $$ \
                 DECLARE idx oid := to_regclass('ec_spire_remote_pk_select_isolation_coord_idx'); \
                 BEGIN \
                     IF idx IS NOT NULL THEN \
                         DELETE FROM ec_spire_placement WHERE index_oid = idx; \
                         DELETE FROM ec_spire_remote_node_descriptor \
                          WHERE coordinator_index_oid = idx; \
                     END IF; \
                 END $$; \
                 DROP TABLE IF EXISTS ec_spire_remote_pk_select_isolation_remote_sql; \
                 DROP TABLE IF EXISTS ec_spire_remote_pk_select_isolation_coord_sql; \
                 CREATE TABLE ec_spire_remote_pk_select_isolation_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO ec_spire_remote_pk_select_isolation_remote_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (2606, 'isolation before', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('606162636465666768696a6b6c6d6e6f', 'hex')); \
                 CREATE INDEX ec_spire_remote_pk_select_isolation_remote_idx \
                     ON ec_spire_remote_pk_select_isolation_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops); \
                 CREATE TABLE ec_spire_remote_pk_select_isolation_coord_sql \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO ec_spire_remote_pk_select_isolation_coord_sql \
                     (id, title, embedding, source_identity) VALUES \
                 (1, 'coordinator seed', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                  decode('707172737475767778797a7b7c7d7e7f', 'hex')); \
                 CREATE INDEX ec_spire_remote_pk_select_isolation_coord_idx \
                     ON ec_spire_remote_pk_select_isolation_coord_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops);",
            )
            .expect("loopback isolation fixture should be created");

        let active_epoch = setup_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot(\
                     'ec_spire_remote_pk_select_isolation_coord_idx'::regclass)",
                &[],
            )
            .expect("isolation active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("isolation active epoch should decode");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut setup_client,
            "ec_spire_remote_pk_select_isolation_remote_idx",
        );
        setup_client
            .batch_execute(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                     'ec_spire_remote_pk_select_isolation_coord_idx'::regclass, \
                     31, 41, 'spire/remote/isolation_pk_select', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_remote_pk_select_isolation_remote_idx', \
                     'active', {active_epoch}, {active_epoch}, '{}', ''); \
                 INSERT INTO ec_spire_placement \
                     (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
                 VALUES ('ec_spire_remote_pk_select_isolation_coord_idx'::regclass, \
                         int8send(2606::bigint)::bytea, 31, 2, {active_epoch}, \
                         decode('606162636465666768696a6b6c6d6e6f', 'hex'))",
                env!("CARGO_PKG_VERSION")
            ))
            .expect("isolation descriptor and placement should be registered");

        let mut plan_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback plan connection should succeed");
        plan_client
            .execute(
                "SELECT tests.ec_spire_test_set_env_var($1::text, $2::text)",
                &[&SECRET_KEY, &loopback_conninfo],
            )
            .expect("plan backend should receive conninfo secret env var");
        let plan_lines = plan_client
            .query(
                "EXPLAIN (COSTS OFF) \
                 SELECT id, title \
                   FROM ec_spire_remote_pk_select_isolation_coord_sql \
                  WHERE id = 2606",
                &[],
            )
            .expect("remote PK SELECT isolation EXPLAIN should succeed")
            .into_iter()
            .map(|row| {
                row.try_get::<_, String>(0)
                    .expect("remote PK SELECT isolation plan row should decode")
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            plan_lines.contains("Custom Scan (EcSpireDistributedScan)"),
            "{plan_lines}"
        );

        // Cross-reference begin_exec.rs:420-428: the CustomScan recheck
        // callback intentionally allows stale remote payload rows for v1.
        for (isolation_level, after_title) in [
            ("READ COMMITTED", "isolation after read committed"),
            ("REPEATABLE READ", "isolation after repeatable read"),
            ("SERIALIZABLE", "isolation after serializable"),
        ] {
            let mut reset_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
                .expect("loopback reset connection should succeed");
            reset_client
                .execute(
                    "UPDATE ec_spire_remote_pk_select_isolation_remote_sql \
                        SET title = 'isolation before' \
                      WHERE id = 2606",
                    &[],
                )
                .expect("remote isolation fixture reset should succeed");

            let mut reader = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
                .expect("loopback reader connection should succeed");
            reader
                .execute(
                    "SELECT tests.ec_spire_test_set_env_var($1::text, $2::text)",
                    &[&SECRET_KEY, &loopback_conninfo],
                )
                .expect("reader backend should receive conninfo secret env var");
            reader
                .batch_execute(&format!("BEGIN ISOLATION LEVEL {isolation_level}"))
                .expect("isolation reader transaction should begin");

            let first_title = reader
                .query_one(
                    "SELECT title \
                       FROM ec_spire_remote_pk_select_isolation_coord_sql \
                      WHERE id = 2606",
                    &[],
                )
                .expect("first remote PK SELECT should succeed")
                .try_get::<_, String>(0)
                .expect("first remote PK SELECT title should decode");
            assert_eq!(first_title, "isolation before", "{isolation_level}");

            let mut writer = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
                .expect("loopback writer connection should succeed");
            writer
                .execute(
                    "UPDATE ec_spire_remote_pk_select_isolation_remote_sql \
                        SET title = $1 \
                      WHERE id = 2606",
                    &[&after_title],
                )
                .expect("remote concurrent update should commit");

            let second_title = reader
                .query_one(
                    "SELECT title \
                       FROM ec_spire_remote_pk_select_isolation_coord_sql \
                      WHERE id = 2606",
                    &[],
                )
                .expect("second remote PK SELECT should succeed")
                .try_get::<_, String>(0)
                .expect("second remote PK SELECT title should decode");
            assert_eq!(second_title, after_title, "{isolation_level}");

            reader
                .batch_execute("COMMIT")
                .expect("isolation reader transaction should commit");
        }

        let for_update_plan_lines = plan_client
            .query(
                "EXPLAIN (COSTS OFF) \
                 SELECT id, title \
                   FROM ec_spire_remote_pk_select_isolation_coord_sql \
                  WHERE id = 2606 \
                  FOR UPDATE",
                &[],
            )
            .expect("remote PK SELECT FOR UPDATE isolation EXPLAIN should succeed")
            .into_iter()
            .map(|row| {
                row.try_get::<_, String>(0)
                    .expect("remote PK SELECT FOR UPDATE plan row should decode")
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            for_update_plan_lines.contains("Custom Scan (EcSpireDistributedScan)"),
            "{for_update_plan_lines}"
        );

        let mut reset_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback FOR UPDATE reset connection should succeed");
        reset_client
            .execute(
                "UPDATE ec_spire_remote_pk_select_isolation_remote_sql \
                    SET title = 'isolation before' \
                  WHERE id = 2606",
                &[],
            )
            .expect("remote FOR UPDATE fixture reset should succeed");

        let mut locker = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback FOR UPDATE reader connection should succeed");
        locker
            .execute(
                "SELECT tests.ec_spire_test_set_env_var($1::text, $2::text)",
                &[&SECRET_KEY, &loopback_conninfo],
            )
            .expect("FOR UPDATE reader backend should receive conninfo secret env var");
        locker
            .batch_execute(
                "BEGIN ISOLATION LEVEL READ COMMITTED; \
                 DECLARE spire_epq_stale_row CURSOR FOR \
                 SELECT title \
                   FROM ec_spire_remote_pk_select_isolation_coord_sql \
                  WHERE id = 2606 \
                  FOR UPDATE",
            )
            .expect("FOR UPDATE cursor should begin before concurrent remote update");

        let mut writer = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback FOR UPDATE writer connection should succeed");
        writer
            .execute(
                "UPDATE ec_spire_remote_pk_select_isolation_remote_sql \
                    SET title = 'isolation after for update' \
                  WHERE id = 2606",
                &[],
            )
            .expect("remote concurrent FOR UPDATE update should commit");

        let locked_title = locker
            .query_one("FETCH 1 FROM spire_epq_stale_row", &[])
            .expect("FOR UPDATE cursor fetch should succeed")
            .try_get::<_, String>(0)
            .expect("FOR UPDATE cursor title should decode");
        assert_eq!(locked_title, "isolation before");

        locker
            .batch_execute("COMMIT")
            .expect("FOR UPDATE reader transaction should commit");
    }
