    #[pg_test]
    fn test_ec_spire_production_executor_state_summary_is_dry() {
        Spi::run(
            "CREATE TABLE ec_spire_prod_state_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_prod_state_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_prod_state_idx \
             ON ec_spire_prod_state_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_spire_prod_state_idx'::regclass::oid")
                .expect("index oid query should succeed")
                .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_prod_state_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_prod_state_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2);
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 51, 'spire/remote/prod-state', decode('aa', 'hex'), \
                     'ec_spire_remote_prod_state_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        let prod_state_from = format!(
            "FROM ec_spire_remote_search_production_executor_state_summary(\
                 'ec_spire_prod_state_idx'::regclass, \
                 {active_epoch}, ARRAY[1.0, 0.0]::real[], \
                 ARRAY[{selected_pid}]::bigint[], 1, 'strict')"
        );
        let state_model = Spi::get_one::<String>(&format!("SELECT state_model {prod_state_from}"))
            .expect("production state model query should succeed")
            .expect("production state model should exist");
        let planned_dispatch_count =
            Spi::get_one::<i64>(&format!("SELECT planned_dispatch_count {prod_state_from}"))
                .expect("planned dispatch query should succeed")
                .expect("planned dispatch count should exist");
        let blocked_before_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT blocked_before_dispatch_count {prod_state_from}"
        ))
        .expect("blocked dispatch query should succeed")
        .expect("blocked dispatch count should exist");
        let conninfo_secret_lookup_count = Spi::get_one::<i64>(&format!(
            "SELECT conninfo_secret_lookup_count {prod_state_from}"
        ))
        .expect("secret lookup count query should succeed")
        .expect("secret lookup count should exist");
        let socket_open_count =
            Spi::get_one::<i64>(&format!("SELECT socket_open_count {prod_state_from}"))
                .expect("socket open count query should succeed")
                .expect("socket open count should exist");
        let endpoint_identity_query_count = Spi::get_one::<i64>(&format!(
            "SELECT endpoint_identity_query_count {prod_state_from}"
        ))
        .expect("endpoint identity count query should succeed")
        .expect("endpoint identity count should exist");
        let transport_pending_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT transport_pending_dispatch_count {prod_state_from}"
        ))
        .expect("transport pending dispatch count query should succeed")
        .expect("transport pending dispatch count should exist");
        let transport_sent_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT transport_sent_dispatch_count {prod_state_from}"
        ))
        .expect("transport sent dispatch count query should succeed")
        .expect("transport sent dispatch count should exist");
        let first_transport_failure_category = Spi::get_one::<String>(&format!(
            "SELECT first_transport_failure_category {prod_state_from}"
        ))
        .expect("transport failure category query should succeed")
        .expect("transport failure category should exist");
        let candidate_receive_pending_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT candidate_receive_pending_dispatch_count {prod_state_from}"
        ))
        .expect("candidate receive pending dispatch count query should succeed")
        .expect("candidate receive pending dispatch count should exist");
        let candidate_receive_sent_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT candidate_receive_sent_dispatch_count {prod_state_from}"
        ))
        .expect("candidate receive sent dispatch count query should succeed")
        .expect("candidate receive sent dispatch count should exist");
        let first_candidate_receive_failure_category = Spi::get_one::<String>(&format!(
            "SELECT first_candidate_receive_failure_category {prod_state_from}"
        ))
        .expect("candidate receive failure category query should succeed")
        .expect("candidate receive failure category should exist");
        let degraded_skipped_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT degraded_skipped_dispatch_count {prod_state_from}"
        ))
        .expect("degraded skipped dispatch count query should succeed")
        .expect("degraded skipped dispatch count should exist");
        let first_degraded_skip_category = Spi::get_one::<String>(&format!(
            "SELECT first_degraded_skip_category {prod_state_from}"
        ))
        .expect("degraded skip category query should succeed")
        .expect("degraded skip category should exist");
        let cancelled_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT cancelled_dispatch_count {prod_state_from}"
        ))
        .expect("cancelled dispatch count query should succeed")
        .expect("cancelled dispatch count should exist");
        let first_cancellation_category = Spi::get_one::<String>(&format!(
            "SELECT first_cancellation_category {prod_state_from}"
        ))
        .expect("cancellation category query should succeed")
        .expect("cancellation category should exist");
        let next_executor_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {prod_state_from}"))
                .expect("next executor step query should succeed")
                .expect("next executor step should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {prod_state_from}"))
            .expect("status query should succeed")
            .expect("status should exist");

        assert_eq!(state_model, "spire_remote_fanout_executor_v1");
        assert_eq!(planned_dispatch_count, 1);
        assert_eq!(blocked_before_dispatch_count, 0);
        assert_eq!(conninfo_secret_lookup_count, 0);
        assert_eq!(socket_open_count, 0);
        assert_eq!(endpoint_identity_query_count, 0);
        assert_eq!(transport_pending_dispatch_count, 1);
        assert_eq!(transport_sent_dispatch_count, 0);
        assert_eq!(first_transport_failure_category, "none");
        assert_eq!(candidate_receive_pending_dispatch_count, 0);
        assert_eq!(candidate_receive_sent_dispatch_count, 0);
        assert_eq!(first_candidate_receive_failure_category, "none");
        assert_eq!(degraded_skipped_dispatch_count, 0);
        assert_eq!(first_degraded_skip_category, "none");
        assert_eq!(cancelled_dispatch_count, 0);
        assert_eq!(first_cancellation_category, "none");
        assert_eq!(next_executor_step, "production_transport_adapter");
        assert_eq!(status, "requires_production_transport_adapter");
    }

    #[pg_test]
    fn test_ec_spire_production_fault_matrix_contract() {
        let required_categories = [
            "connect_failed",
            "remote_executor_overload",
            "requires_conninfo_secret_resolution",
            "remote_statement_timeout",
            "remote_query_failed",
            "local_statement_timeout",
            "remote_backend_terminated",
            "remote_query_cancelled",
            "local_query_cancelled",
            "candidate_batch_validation_failed",
            "endpoint_identity_mismatch",
            "incompatible_extension_version",
            "extension_version_mismatch",
            "stale_epoch",
            "served_epoch_mismatch",
            "consistency_mode_mismatch",
            "remote_index_unavailable",
            "remote_heap_resolution_failed",
            "remote_heap_row_missing",
        ];
        for category in required_categories {
            let count = Spi::get_one::<i64>(&format!(
                "SELECT count(*) FROM ec_spire_remote_search_production_fault_matrix() \
                 WHERE failure_category = '{category}'"
            ))
            .expect("fault matrix category query should succeed")
            .expect("fault matrix category count should exist");
            assert_eq!(count, 1, "missing or duplicate category {category}");
        }

        let local_timeout = Spi::get_one::<String>(
            "SELECT strict_action FROM ec_spire_remote_search_production_fault_matrix() \
             WHERE failure_category = 'local_statement_timeout'",
        )
        .expect("local timeout action query should succeed")
        .expect("local timeout action should exist");
        let remote_timeout = Spi::get_one::<String>(
            "SELECT degraded_action FROM ec_spire_remote_search_production_fault_matrix() \
             WHERE failure_category = 'remote_statement_timeout'",
        )
        .expect("remote timeout action query should succeed")
        .expect("remote timeout action should exist");
        let executor_overload_step = Spi::get_one::<String>(
            "SELECT next_executor_step FROM ec_spire_remote_search_production_fault_matrix() \
             WHERE failure_category = 'remote_executor_overload'",
        )
        .expect("executor overload step query should succeed")
        .expect("executor overload step should exist");
        let consistency_mismatch = Spi::get_one::<String>(
            "SELECT degraded_action FROM ec_spire_remote_search_production_fault_matrix() \
             WHERE failure_category = 'consistency_mode_mismatch'",
        )
        .expect("consistency mismatch action query should succeed")
        .expect("consistency mismatch action should exist");
        let heap_step = Spi::get_one::<String>(
            "SELECT next_executor_step FROM ec_spire_remote_search_production_fault_matrix() \
             WHERE failure_category = 'remote_heap_resolution_failed'",
        )
        .expect("heap step query should succeed")
        .expect("heap step should exist");
        assert_eq!(local_timeout, "cancel_query");
        assert_eq!(remote_timeout, "skip_node");
        assert_eq!(executor_overload_step, "remote_executor_governance");
        assert_eq!(consistency_mismatch, "fail_closed");
        assert_eq!(heap_step, "remote_heap_resolution");
    }

    #[pg_test]
    fn test_ec_spire_stage_e_fault_matrix_contract() {
        let required_cases = [
            "epoch_mismatch",
            "version_skew",
            "fingerprint_mismatch",
            "connection_reset_mid_batch",
            "remote_backend_termination",
            "remote_statement_timeout",
            "local_statement_timeout",
            "local_cancel",
            "simulated_network_partition",
            "remote_oom",
            "missing_or_reindexed_remote_index",
        ];
        for fault_case in required_cases {
            let count = Spi::get_one::<i64>(&format!(
                "SELECT count(*) FROM ec_spire_remote_search_stage_e_fault_matrix() \
                 WHERE fault_case = '{fault_case}'"
            ))
            .expect("Stage E fault matrix case query should succeed")
            .expect("Stage E fault matrix case count should exist");
            assert_eq!(count, 1, "missing or duplicate Stage E case {fault_case}");
        }

        let local_cancel_action = Spi::get_one::<String>(
            "SELECT strict_action FROM ec_spire_remote_search_stage_e_fault_matrix() \
             WHERE fault_case = 'local_cancel'",
        )
        .expect("local cancel action query should succeed")
        .expect("local cancel action should exist");
        let remote_oom_category = Spi::get_one::<String>(
            "SELECT failure_category FROM ec_spire_remote_search_stage_e_fault_matrix() \
             WHERE fault_case = 'remote_oom'",
        )
        .expect("remote OOM category query should succeed")
        .expect("remote OOM category should exist");
        let missing_index_step = Spi::get_one::<String>(
            "SELECT next_executor_step FROM ec_spire_remote_search_stage_e_fault_matrix() \
             WHERE fault_case = 'missing_or_reindexed_remote_index'",
        )
        .expect("missing index step query should succeed")
        .expect("missing index step should exist");
        let local_timeout_counters = Spi::get_one::<String>(
            "SELECT counter_delta FROM ec_spire_remote_search_stage_e_fault_matrix() \
             WHERE fault_case = 'local_statement_timeout'",
        )
        .expect("local timeout counter query should succeed")
        .expect("local timeout counter should exist");

        assert_eq!(local_cancel_action, "cancel_query");
        assert_eq!(remote_oom_category, "remote_query_failed");
        assert_eq!(missing_index_step, "compact_candidate_receive");
        assert!(local_timeout_counters.contains("retained_candidate_batch_count=0"));
    }

    #[pg_test]
    fn test_ec_spire_stage_e_lifecycle_matrix_contract() {
        let required_cases = [
            "drop_remote_index_before_fanout",
            "drop_remote_index_in_flight",
            "reindex_remote_index_before_fanout",
            "reindex_remote_index_in_flight",
            "create_index_concurrently_new_descriptor",
            "create_index_concurrently_missing_descriptor",
        ];
        for lifecycle_case in required_cases {
            let count = Spi::get_one::<i64>(&format!(
                "SELECT count(*) FROM ec_spire_remote_search_stage_e_lifecycle_matrix() \
                 WHERE lifecycle_case = '{lifecycle_case}'"
            ))
            .expect("Stage E lifecycle matrix case query should succeed")
            .expect("Stage E lifecycle matrix case count should exist");
            assert_eq!(
                count, 1,
                "missing or duplicate Stage E lifecycle case {lifecycle_case}"
            );
        }

        let drop_detection = Spi::get_one::<String>(
            "SELECT required_detection FROM ec_spire_remote_search_stage_e_lifecycle_matrix() \
             WHERE lifecycle_case = 'drop_remote_index_in_flight'",
        )
        .expect("drop index detection query should succeed")
        .expect("drop index detection should exist");
        let reindex_status = Spi::get_one::<String>(
            "SELECT required_detection FROM ec_spire_remote_search_stage_e_lifecycle_matrix() \
             WHERE lifecycle_case = 'reindex_remote_index_in_flight'",
        )
        .expect("reindex status query should succeed")
        .expect("reindex status should exist");
        let create_action = Spi::get_one::<String>(
            "SELECT strict_action FROM ec_spire_remote_search_stage_e_lifecycle_matrix() \
             WHERE lifecycle_case = 'create_index_concurrently_new_descriptor'",
        )
        .expect("create concurrently action query should succeed")
        .expect("create concurrently action should exist");
        let missing_descriptor_step = Spi::get_one::<String>(
            "SELECT next_executor_step FROM ec_spire_remote_search_stage_e_lifecycle_matrix() \
             WHERE lifecycle_case = 'create_index_concurrently_missing_descriptor'",
        )
        .expect("missing descriptor step query should succeed")
        .expect("missing descriptor step should exist");

        assert_eq!(drop_detection, "remote_index_unavailable");
        assert_eq!(reindex_status, "endpoint_identity_mismatch");
        assert_eq!(create_action, "defer_new_descriptor");
        assert_eq!(missing_descriptor_step, "remote_node_descriptor");
    }

    #[pg_test]
    fn test_ec_spire_prod_executor_session_policy_guc() {
        Spi::run(
            "CREATE TABLE ec_spire_prod_session_policy_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_prod_session_policy_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_prod_session_policy_idx \
             ON ec_spire_prod_session_policy_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_spire_prod_session_policy_idx'::regclass::oid")
                .expect("index oid query should succeed")
                .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_prod_session_policy_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_prod_session_policy_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        let session_summary_from = format!(
            "FROM ec_spire_remote_search_production_executor_session_summary(\
                 'ec_spire_prod_session_policy_idx'::regclass, \
                 {active_epoch}, ARRAY[1.0, 0.0]::real[], \
                 ARRAY[{selected_pid}]::bigint[], 1)"
        );
        let default_mode =
            Spi::get_one::<String>(&format!("SELECT consistency_mode {session_summary_from}"))
                .expect("default mode query should succeed")
                .expect("default mode should exist");
        let default_source = Spi::get_one::<String>(&format!(
            "SELECT consistency_mode_source {session_summary_from}"
        ))
        .expect("default mode source query should succeed")
        .expect("default mode source should exist");

        assert_eq!(default_source, "ec_spire.remote_search_consistency_mode");
        assert_eq!(default_mode, "strict");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2);
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 51, 'spire/remote/prod-session-policy', decode('aa', 'hex'), \
                     'ec_spire_remote_prod_session_policy_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        Spi::run("SET LOCAL ec_spire.remote_search_consistency_mode = 'degraded'")
            .expect("remote search consistency mode SET should succeed");
        let degraded_mode =
            Spi::get_one::<String>(&format!("SELECT consistency_mode {session_summary_from}"))
                .expect("degraded mode query should succeed")
                .expect("degraded mode should exist");
        let degraded_dispatch_count =
            Spi::get_one::<i64>(&format!("SELECT dispatch_count {session_summary_from}"))
                .expect("degraded dispatch count query should succeed")
                .expect("degraded dispatch count should exist");
        let degraded_status =
            Spi::get_one::<String>(&format!("SELECT status {session_summary_from}"))
                .expect("degraded status query should succeed")
                .expect("degraded status should exist");

        assert_eq!(degraded_mode, "degraded");
        assert_eq!(degraded_dispatch_count, 1);
        assert_eq!(degraded_status, "requires_production_transport_adapter");
    }

    #[pg_test]
    fn test_ec_spire_prod_scan_handoff_receive() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_SCAN_HANDOFF",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_prod_scan_handoff_remote_sql; \
                 CREATE TABLE ec_spire_prod_scan_handoff_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_prod_scan_handoff_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_prod_scan_handoff_remote_idx \
                     ON ec_spire_prod_scan_handoff_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
            )
            .expect("loopback remote handoff fixture should be created");
        let remote_active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_prod_scan_handoff_remote_idx'::regclass)",
                &[],
            )
            .expect("remote active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote active epoch should decode");
        let remote_leaf_pids = loopback_client
            .query_one(
                "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_prod_scan_handoff_remote_idx'::regclass)",
                &[],
            )
            .expect("remote leaf pid query should succeed")
            .try_get::<_, Vec<i64>>(0)
            .expect("remote leaf pids should decode");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_prod_scan_handoff_remote_idx",
        );

        Spi::run(
            "CREATE TABLE ec_spire_prod_scan_handoff_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_prod_scan_handoff_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("coordinator insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_prod_scan_handoff_coord_idx \
             ON ec_spire_prod_scan_handoff_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
        )
        .expect("coordinator index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_prod_scan_handoff_coord_idx'::regclass::oid",
        )
        .expect("coordinator index oid query should succeed")
        .expect("coordinator index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_prod_scan_handoff_coord_idx'::regclass)",
        )
        .expect("coordinator active epoch query should succeed")
        .expect("coordinator active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_prod_scan_handoff_coord_idx'::regclass)",
        )
        .expect("coordinator leaf pid query should succeed")
        .expect("coordinator leaf pids should exist");
        assert_eq!(remote_active_epoch, active_epoch);
        assert_eq!(remote_leaf_pids, coord_leaf_pids);
        assert_eq!(coord_leaf_pids.len(), 2);

        unsafe {
            am::debug_spire_rewrite_placement_node(index_oid, coord_leaf_pids[0] as u64, 2);
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 61, 'spire/remote/scan-handoff', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_prod_scan_handoff_remote_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        let handoff_from = "FROM ec_spire_remote_search_production_scan_handoff_summary(\
             'ec_spire_prod_scan_handoff_coord_idx'::regclass, \
             ARRAY[1.0, 0.0]::real[], 1)";
        let effective_nprobe =
            Spi::get_one::<i64>(&format!("SELECT effective_nprobe {handoff_from}"))
                .expect("handoff effective nprobe query should succeed")
                .expect("handoff effective nprobe should exist");
        let selected_pid_count =
            Spi::get_one::<i64>(&format!("SELECT selected_pid_count {handoff_from}"))
                .expect("handoff selected pid query should succeed")
                .expect("handoff selected pid count should exist");
        let local_pid_count =
            Spi::get_one::<i64>(&format!("SELECT local_pid_count {handoff_from}"))
                .expect("handoff local pid query should succeed")
                .expect("handoff local pid count should exist");
        let remote_pid_count =
            Spi::get_one::<i64>(&format!("SELECT remote_pid_count {handoff_from}"))
                .expect("handoff remote pid query should succeed")
                .expect("handoff remote pid count should exist");
        let dispatch_count = Spi::get_one::<i64>(&format!("SELECT dispatch_count {handoff_from}"))
            .expect("handoff dispatch count query should succeed")
            .expect("handoff dispatch count should exist");
        let ready_receive_count = Spi::get_one::<i64>(&format!(
            "SELECT candidate_receive_ready_dispatch_count {handoff_from}"
        ))
        .expect("handoff ready receive query should succeed")
        .expect("handoff ready receive count should exist");
        let candidate_row_count =
            Spi::get_one::<i64>(&format!("SELECT candidate_row_count {handoff_from}"))
                .expect("handoff candidate row count query should succeed")
                .expect("handoff candidate row count should exist");
        let merged_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT merged_candidate_count {handoff_from}"))
                .expect("handoff merged candidate count query should succeed")
                .expect("handoff merged candidate count should exist");
        let final_heap_status =
            Spi::get_one::<String>(&format!("SELECT final_heap_fetch_status {handoff_from}"))
                .expect("handoff final heap status query should succeed")
                .expect("handoff final heap status should exist");
        let next_blocker = Spi::get_one::<String>(&format!("SELECT next_blocker {handoff_from}"))
            .expect("handoff next blocker query should succeed")
            .expect("handoff next blocker should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {handoff_from}"))
            .expect("handoff status query should succeed")
            .expect("handoff status should exist");

        assert_eq!(effective_nprobe, 2);
        assert_eq!(selected_pid_count, 2);
        assert_eq!(local_pid_count, 1);
        assert_eq!(remote_pid_count, 1);
        assert_eq!(dispatch_count, 1);
        assert_eq!(ready_receive_count, 1);
        assert_eq!(candidate_row_count, 1);
        assert_eq!(merged_candidate_count, 1);
        assert_eq!(final_heap_status, "requires_remote_heap_resolution");
        assert_eq!(next_blocker, "remote_heap_resolution");
        assert_eq!(status, "requires_remote_heap_resolution");
    }

    #[pg_test]
    fn test_ec_spire_prod_scan_heap_resolution() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_SCAN_HEAP",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_prod_scan_heap_remote_sql; \
                 CREATE TABLE ec_spire_prod_scan_heap_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_prod_scan_heap_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_prod_scan_heap_remote_idx \
                     ON ec_spire_prod_scan_heap_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
            )
            .expect("loopback remote heap fixture should be created");
        let remote_active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_prod_scan_heap_remote_idx'::regclass)",
                &[],
            )
            .expect("remote active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote active epoch should decode");
        let remote_leaf_pids = loopback_client
            .query_one(
                "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_prod_scan_heap_remote_idx'::regclass)",
                &[],
            )
            .expect("remote leaf pid query should succeed")
            .try_get::<_, Vec<i64>>(0)
            .expect("remote leaf pids should decode");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_prod_scan_heap_remote_idx",
        );

        Spi::run(
            "CREATE TABLE ec_spire_prod_scan_heap_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("coordinator table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_prod_scan_heap_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("coordinator insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_prod_scan_heap_coord_idx \
             ON ec_spire_prod_scan_heap_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
        )
        .expect("coordinator index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_prod_scan_heap_coord_idx'::regclass::oid",
        )
        .expect("coordinator index oid query should succeed")
        .expect("coordinator index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_prod_scan_heap_coord_idx'::regclass)",
        )
        .expect("coordinator active epoch query should succeed")
        .expect("coordinator active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_prod_scan_heap_coord_idx'::regclass)",
        )
        .expect("coordinator leaf pid query should succeed")
        .expect("coordinator leaf pids should exist");
        assert_eq!(remote_active_epoch, active_epoch);
        assert_eq!(remote_leaf_pids, coord_leaf_pids);
        assert_eq!(coord_leaf_pids.len(), 2);

        unsafe {
            am::debug_spire_rewrite_placement_node(index_oid, coord_leaf_pids[0] as u64, 2);
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 71, 'spire/remote/scan-heap', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_prod_scan_heap_remote_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        let heap_from = "FROM ec_spire_remote_search_production_scan_heap_resolution_summary(\
             'ec_spire_prod_scan_heap_coord_idx'::regclass, \
             ARRAY[1.0, 0.0]::real[], 2)";
        let effective_nprobe = Spi::get_one::<i64>(&format!("SELECT effective_nprobe {heap_from}"))
            .expect("heap effective nprobe query should succeed")
            .expect("heap effective nprobe should exist");
        let selected_pid_count =
            Spi::get_one::<i64>(&format!("SELECT selected_pid_count {heap_from}"))
                .expect("heap selected pid count query should succeed")
                .expect("heap selected pid count should exist");
        let local_pid_count = Spi::get_one::<i64>(&format!("SELECT local_pid_count {heap_from}"))
            .expect("heap local pid count query should succeed")
            .expect("heap local pid count should exist");
        let remote_pid_count = Spi::get_one::<i64>(&format!("SELECT remote_pid_count {heap_from}"))
            .expect("heap remote pid count query should succeed")
            .expect("heap remote pid count should exist");
        let dispatch_count = Spi::get_one::<i64>(&format!("SELECT dispatch_count {heap_from}"))
            .expect("heap dispatch count query should succeed")
            .expect("heap dispatch count should exist");
        let compact_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT compact_candidate_count {heap_from}"))
                .expect("heap compact candidate count query should succeed")
                .expect("heap compact candidate count should exist");
        let remote_heap_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT remote_heap_ready_dispatch_count {heap_from}"
        ))
        .expect("heap ready dispatch query should succeed")
        .expect("heap ready dispatch count should exist");
        let remote_heap_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT remote_heap_candidate_count {heap_from}"))
                .expect("heap remote candidate count query should succeed")
                .expect("heap remote candidate count should exist");
        let local_heap_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT local_heap_candidate_count {heap_from}"))
                .expect("heap local candidate count query should succeed")
                .expect("heap local candidate count should exist");
        let returned_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT returned_candidate_count {heap_from}"))
                .expect("heap returned candidate count query should succeed")
                .expect("heap returned candidate count should exist");
        let result_source = Spi::get_one::<String>(&format!("SELECT result_source {heap_from}"))
            .expect("heap result source query should succeed")
            .expect("heap result source should exist");
        let final_heap_status =
            Spi::get_one::<String>(&format!("SELECT final_heap_fetch_status {heap_from}"))
                .expect("heap final status query should succeed")
                .expect("heap final status should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {heap_from}"))
            .expect("heap status query should succeed")
            .expect("heap status should exist");

        assert_eq!(effective_nprobe, 2);
        assert_eq!(selected_pid_count, 2);
        assert_eq!(local_pid_count, 1);
        assert_eq!(remote_pid_count, 1);
        assert_eq!(dispatch_count, 1);
        assert_eq!(compact_candidate_count, 1);
        assert_eq!(remote_heap_ready_count, 1);
        assert_eq!(remote_heap_candidate_count, 1);
        assert_eq!(local_heap_candidate_count, 1);
        assert_eq!(returned_candidate_count, 2);
        assert_eq!(result_source, "remote_heap_candidates");
        assert_eq!(final_heap_status, "remote_ready");
        assert_eq!(status, "ready");

        let diagnostics_from = "FROM ec_spire_remote_search_operator_diagnostics(\
             'ec_spire_prod_scan_heap_coord_idx'::regclass, \
             ARRAY[1.0, 0.0]::real[], 2)";
        let diagnostic_remote_node_count =
            Spi::get_one::<i64>(&format!("SELECT remote_node_count {diagnostics_from}"))
                .expect("diagnostic remote node count query should succeed")
                .expect("diagnostic remote node count should exist");
        let diagnostic_ready_remote_node_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_remote_node_count {diagnostics_from}"
        ))
        .expect("diagnostic ready remote node count query should succeed")
        .expect("diagnostic ready remote node count should exist");
        let diagnostic_min_served = Spi::get_one::<i64>(&format!(
            "SELECT min_remote_last_served_epoch {diagnostics_from}"
        ))
        .expect("diagnostic min served epoch query should succeed")
        .expect("diagnostic min served epoch should exist");
        let diagnostic_max_served = Spi::get_one::<i64>(&format!(
            "SELECT max_remote_last_served_epoch {diagnostics_from}"
        ))
        .expect("diagnostic max served epoch query should succeed")
        .expect("diagnostic max served epoch should exist");
        let diagnostic_fanout =
            Spi::get_one::<i64>(&format!("SELECT remote_fanout_count {diagnostics_from}"))
                .expect("diagnostic fanout query should succeed")
                .expect("diagnostic fanout count should exist");
        let diagnostic_candidate_batches =
            Spi::get_one::<i64>(&format!("SELECT candidate_batch_count {diagnostics_from}"))
                .expect("diagnostic candidate batch query should succeed")
                .expect("diagnostic candidate batch count should exist");
        let diagnostic_candidate_rows =
            Spi::get_one::<i64>(&format!("SELECT candidate_row_count {diagnostics_from}"))
                .expect("diagnostic candidate row query should succeed")
                .expect("diagnostic candidate row count should exist");
        let diagnostic_final_heap = Spi::get_one::<String>(&format!(
            "SELECT final_heap_fetch_status {diagnostics_from}"
        ))
        .expect("diagnostic final heap query should succeed")
        .expect("diagnostic final heap status should exist");
        let diagnostic_am_status =
            Spi::get_one::<String>(&format!("SELECT am_delivery_status {diagnostics_from}"))
                .expect("diagnostic AM status query should succeed")
                .expect("diagnostic AM status should exist");
        let diagnostic_remote_origin_outputs = Spi::get_one::<i64>(&format!(
            "SELECT remote_origin_output_count {diagnostics_from}"
        ))
        .expect("diagnostic remote origin output query should succeed")
        .expect("diagnostic remote origin output count should exist");
        let diagnostic_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {diagnostics_from}"))
                .expect("diagnostic next blocker query should succeed")
                .expect("diagnostic next blocker should exist");
        let diagnostic_status =
            Spi::get_one::<String>(&format!("SELECT status {diagnostics_from}"))
                .expect("diagnostic status query should succeed")
                .expect("diagnostic status should exist");

        assert_eq!(diagnostic_remote_node_count, 1);
        assert_eq!(diagnostic_ready_remote_node_count, 1);
        assert_eq!(diagnostic_min_served, active_epoch);
        assert_eq!(diagnostic_max_served, active_epoch);
        assert_eq!(diagnostic_fanout, 1);
        assert_eq!(diagnostic_candidate_batches, 1);
        assert_eq!(diagnostic_candidate_rows, 1);
        assert_eq!(diagnostic_final_heap, "remote_ready");
        assert_eq!(diagnostic_am_status, "requires_custom_scan_tuple_delivery");
        assert_eq!(diagnostic_remote_origin_outputs, 1);
        assert_eq!(diagnostic_next_blocker, "custom_scan_tuple_delivery");
        assert_eq!(diagnostic_status, "requires_custom_scan_tuple_delivery");

        loopback_client
            .batch_execute("DELETE FROM ec_spire_prod_scan_heap_remote_sql")
            .expect("remote heap row delete should succeed");
        let missing_status = Spi::get_one::<String>(&format!("SELECT status {heap_from}"))
            .expect("missing heap status query should succeed")
            .expect("missing heap status should exist");
        let missing_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {heap_from}"))
                .expect("missing heap blocker query should succeed")
                .expect("missing heap blocker should exist");
        let missing_failed_count = Spi::get_one::<i64>(&format!(
            "SELECT remote_heap_failed_dispatch_count {heap_from}"
        ))
        .expect("missing heap failed count query should succeed")
        .expect("missing heap failed count should exist");
        assert_eq!(missing_status, "remote_heap_resolution_failed");
        assert_eq!(missing_next_blocker, "remote_heap_resolution");
        assert_eq!(missing_failed_count, 1);
    }

