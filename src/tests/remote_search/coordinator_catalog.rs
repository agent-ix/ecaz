    #[pg_test]
    fn test_ec_spire_remote_search_coordinator_result_ready_empty() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_result_ready_empty_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_result_ready_empty_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_result_ready_empty_sql_idx \
             ON ec_spire_remote_result_ready_empty_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_result_ready_empty_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_result_ready_empty_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let result_summary_from = format!(
            "FROM ec_spire_remote_search_coordinator_result_summary(\
             'ec_spire_remote_result_ready_empty_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 0, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let result_source =
            Spi::get_one::<String>(&format!("SELECT result_source {result_summary_from}"))
                .expect("ready-empty result source query should succeed")
                .expect("ready-empty result source should exist");
        let returned_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {result_summary_from}"
        ))
        .expect("ready-empty returned count query should succeed")
        .expect("ready-empty returned count should exist");
        let next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {result_summary_from}"))
                .expect("ready-empty blocker query should succeed")
                .expect("ready-empty blocker should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {result_summary_from}"))
            .expect("ready-empty status query should succeed")
            .expect("ready-empty status should exist");
        let contract_validator = Spi::get_one::<String>(
            "SELECT validator FROM ec_spire_remote_search_coordinator_result_contract() \
             WHERE result_source = 'none'",
        )
        .expect("ready-empty contract query should succeed")
        .expect("ready-empty contract validator should exist");

        assert_eq!(result_source, "none");
        assert_eq!(returned_candidate_count, 0);
        assert_eq!(next_blocker, "none");
        assert_eq!(status, "empty_top_k");
        assert_eq!(
            contract_validator,
            "must_have_zero_returned_candidate_count_and_no_blocker"
        );
    }

    #[pg_test]
    fn test_ec_spire_remote_search_final_summary_blocked() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_final_blocked_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_final_blocked_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_final_blocked_sql_idx \
             ON ec_spire_remote_final_blocked_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_final_blocked_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_final_blocked_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_final_blocked_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let summary_from = format!(
            "FROM ec_spire_remote_search_finalization_summary(\
             'ec_spire_remote_final_blocked_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("finalization status query should succeed")
            .expect("finalization status should exist");
        let final_heap_fetch_status =
            Spi::get_one::<String>(&format!("SELECT final_heap_fetch_status {summary_from}"))
                .expect("finalization heap fetch status query should succeed")
                .expect("finalization heap fetch status should exist");
        let row_locator_policy =
            Spi::get_one::<String>(&format!("SELECT row_locator_policy {summary_from}"))
                .expect("finalization locator policy query should succeed")
                .expect("finalization locator policy should exist");

        assert_eq!(status, "requires_remote_node_descriptor");
        assert_eq!(final_heap_fetch_status, "blocked");
        assert_eq!(row_locator_policy, "opaque_origin_node_bytes");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_coordinator_gate_summary() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_coord_gate_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_coord_gate_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_coord_gate_sql_idx \
             ON ec_spire_remote_coord_gate_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_coord_gate_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_coord_gate_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_coord_gate_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let local_gate_from = format!(
            "FROM ec_spire_remote_search_coordinator_gate_summary(\
             'ec_spire_remote_coord_gate_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}]::bigint[], 3, 'strict')",
            selected_pids[0],
        );
        let local_status = Spi::get_one::<String>(&format!("SELECT status {local_gate_from}"))
            .expect("local coordinator gate status query should succeed")
            .expect("local coordinator gate status should exist");
        let local_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {local_gate_from}"))
                .expect("local coordinator gate blocker query should succeed")
                .expect("local coordinator gate blocker should exist");
        let local_final_heap_fetch_status =
            Spi::get_one::<String>(&format!("SELECT final_heap_fetch_status {local_gate_from}"))
                .expect("local coordinator gate heap query should succeed")
                .expect("local coordinator gate heap should exist");
        let local_libpq_dispatch_count =
            Spi::get_one::<i64>(&format!("SELECT libpq_dispatch_count {local_gate_from}"))
                .expect("local coordinator gate dispatch count query should succeed")
                .expect("local coordinator gate dispatch count should exist");
        let local_libpq_dispatch_status =
            Spi::get_one::<String>(&format!("SELECT libpq_dispatch_status {local_gate_from}"))
                .expect("local coordinator gate dispatch status query should succeed")
                .expect("local coordinator gate dispatch status should exist");
        let local_libpq_executor_status =
            Spi::get_one::<String>(&format!("SELECT libpq_executor_status {local_gate_from}"))
                .expect("local coordinator gate executor status query should succeed")
                .expect("local coordinator gate executor status should exist");
        let local_libpq_executor_next_step = Spi::get_one::<String>(&format!(
            "SELECT libpq_executor_next_step {local_gate_from}"
        ))
        .expect("local coordinator gate executor step query should succeed")
        .expect("local coordinator gate executor step should exist");
        let local_libpq_receive_count =
            Spi::get_one::<i64>(&format!("SELECT libpq_receive_count {local_gate_from}"))
                .expect("local coordinator gate receive count query should succeed")
                .expect("local coordinator gate receive count should exist");
        let local_libpq_receive_status =
            Spi::get_one::<String>(&format!("SELECT libpq_receive_status {local_gate_from}"))
                .expect("local coordinator gate receive status query should succeed")
                .expect("local coordinator gate receive status should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let remote_gate_from = format!(
            "FROM ec_spire_remote_search_coordinator_gate_summary(\
             'ec_spire_remote_coord_gate_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let remote_status = Spi::get_one::<String>(&format!("SELECT status {remote_gate_from}"))
            .expect("remote coordinator gate status query should succeed")
            .expect("remote coordinator gate status should exist");
        let remote_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {remote_gate_from}"))
                .expect("remote coordinator gate blocker query should succeed")
                .expect("remote coordinator gate blocker should exist");
        let remote_execution_status =
            Spi::get_one::<String>(&format!("SELECT execution_status {remote_gate_from}"))
                .expect("remote coordinator gate execution query should succeed")
                .expect("remote coordinator gate execution should exist");
        let remote_libpq_dispatch_count =
            Spi::get_one::<i64>(&format!("SELECT libpq_dispatch_count {remote_gate_from}"))
                .expect("remote coordinator gate dispatch count query should succeed")
                .expect("remote coordinator gate dispatch count should exist");
        let remote_libpq_dispatch_status =
            Spi::get_one::<String>(&format!("SELECT libpq_dispatch_status {remote_gate_from}"))
                .expect("remote coordinator gate dispatch status query should succeed")
                .expect("remote coordinator gate dispatch status should exist");
        let remote_libpq_executor_status =
            Spi::get_one::<String>(&format!("SELECT libpq_executor_status {remote_gate_from}"))
                .expect("remote coordinator gate executor status query should succeed")
                .expect("remote coordinator gate executor status should exist");
        let remote_libpq_executor_next_step = Spi::get_one::<String>(&format!(
            "SELECT libpq_executor_next_step {remote_gate_from}"
        ))
        .expect("remote coordinator gate executor step query should succeed")
        .expect("remote coordinator gate executor step should exist");
        let remote_libpq_receive_count =
            Spi::get_one::<i64>(&format!("SELECT libpq_receive_count {remote_gate_from}"))
                .expect("remote coordinator gate receive count query should succeed")
                .expect("remote coordinator gate receive count should exist");
        let remote_libpq_receive_status =
            Spi::get_one::<String>(&format!("SELECT libpq_receive_status {remote_gate_from}"))
                .expect("remote coordinator gate receive status query should succeed")
                .expect("remote coordinator gate receive status should exist");
        let remote_plan_count =
            Spi::get_one::<i64>(&format!("SELECT remote_plan_count {remote_gate_from}"))
                .expect("remote coordinator gate remote plan query should succeed")
                .expect("remote coordinator gate remote plan count should exist");
        let remote_pid_count =
            Spi::get_one::<i64>(&format!("SELECT remote_pid_count {remote_gate_from}"))
                .expect("remote coordinator gate remote pid query should succeed")
                .expect("remote coordinator gate remote pid count should exist");

        assert_eq!(local_status, "ready");
        assert_eq!(local_next_blocker, "none");
        assert_eq!(local_final_heap_fetch_status, "local_ready");
        assert_eq!(local_libpq_dispatch_count, 0);
        assert_eq!(local_libpq_dispatch_status, "ready");
        assert_eq!(local_libpq_executor_status, "ready");
        assert_eq!(local_libpq_executor_next_step, "none");
        assert_eq!(local_libpq_receive_count, 0);
        assert_eq!(local_libpq_receive_status, "ready");
        assert_eq!(remote_status, "requires_remote_node_descriptor");
        assert_eq!(remote_next_blocker, "remote_node_descriptor");
        assert_eq!(remote_execution_status, "requires_remote_node_descriptor");
        assert_eq!(remote_libpq_dispatch_count, 1);
        assert_eq!(
            remote_libpq_dispatch_status,
            "requires_remote_node_descriptor"
        );
        assert_eq!(
            remote_libpq_executor_status,
            "requires_remote_node_descriptor"
        );
        assert_eq!(remote_libpq_executor_next_step, "remote_node_descriptor");
        assert_eq!(remote_libpq_receive_count, 1);
        assert_eq!(
            remote_libpq_receive_status,
            "requires_remote_node_descriptor"
        );
        assert_eq!(remote_plan_count, 1);
        assert_eq!(remote_pid_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_node_snapshot_local() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_node_local_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_node_local_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_node_local_sql_idx \
             ON ec_spire_remote_node_local_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let snapshot_from = "FROM ec_spire_remote_node_snapshot(\
             'ec_spire_remote_node_local_sql_idx'::regclass)";
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {snapshot_from}"))
            .expect("remote node snapshot count query should succeed")
            .expect("remote node snapshot count should exist");
        let node_kind = Spi::get_one::<String>(&format!("SELECT node_kind {snapshot_from}"))
            .expect("remote node snapshot kind query should succeed")
            .expect("remote node snapshot kind should exist");
        let descriptor_state =
            Spi::get_one::<String>(&format!("SELECT descriptor_state {snapshot_from}"))
                .expect("remote node snapshot state query should succeed")
                .expect("remote node snapshot state should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {snapshot_from}"))
            .expect("remote node snapshot status query should succeed")
            .expect("remote node snapshot status should exist");
        let served_epoch_matches = Spi::get_one::<bool>(&format!(
            "SELECT last_served_epoch = active_epoch AND min_retained_epoch = active_epoch \
             {snapshot_from}"
        ))
        .expect("remote node snapshot epoch query should succeed")
        .expect("remote node snapshot epoch check should exist");

        assert_eq!(row_count, 1);
        assert_eq!(node_kind, "local");
        assert_eq!(descriptor_state, "active");
        assert_eq!(status, "ready");
        assert!(served_epoch_matches);
    }

    #[pg_test]
    fn test_ec_spire_remote_node_snapshot_missing_descriptor() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_node_missing_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_node_missing_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_node_missing_sql_idx \
             ON ec_spire_remote_node_missing_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_node_missing_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_node_missing_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let snapshot_from = "FROM ec_spire_remote_node_snapshot(\
             'ec_spire_remote_node_missing_sql_idx'::regclass)";
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {snapshot_from}"))
            .expect("remote node snapshot count query should succeed")
            .expect("remote node snapshot count should exist");
        let remote_status =
            Spi::get_one::<String>(&format!("SELECT status {snapshot_from} WHERE node_id = 2"))
                .expect("remote node snapshot status query should succeed")
                .expect("remote node snapshot status should exist");
        let remote_error = Spi::get_one::<String>(&format!(
            "SELECT last_error {snapshot_from} WHERE node_id = 2"
        ))
        .expect("remote node snapshot error query should succeed")
        .expect("remote node snapshot error should exist");
        let remote_placement_count = Spi::get_one::<i64>(&format!(
            "SELECT placement_count {snapshot_from} WHERE node_id = 2"
        ))
        .expect("remote node snapshot placement query should succeed")
        .expect("remote node snapshot placement should exist");
        let local_status =
            Spi::get_one::<String>(&format!("SELECT status {snapshot_from} WHERE node_id = 0"))
                .expect("local node snapshot status query should succeed")
                .expect("local node snapshot status should exist");

        assert_eq!(row_count, 2);
        assert_eq!(remote_status, "requires_remote_node_descriptor");
        assert_eq!(remote_error, "missing_remote_node_descriptor");
        assert_eq!(remote_placement_count, 1);
        assert_eq!(local_status, "ready");
    }

    #[pg_test]
    fn test_ec_spire_remote_node_descriptor_catalog_active() {
        let _env_lock = env_var_test_lock();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_2",
            "host=/tmp/ecaz-missing-socket dbname=ecaz connect_timeout=1",
        );

        Spi::run(
            "CREATE TABLE ec_spire_remote_node_desc_catalog_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_node_desc_catalog_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_node_desc_catalog_sql_idx \
             ON ec_spire_remote_node_desc_catalog_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_node_desc_catalog_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_node_desc_catalog_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_node_desc_catalog_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 7, 'spire/remote/2', decode('01', 'hex'), \
                     'remote_spire_idx', 'active', {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");

        let snapshot_from = "FROM ec_spire_remote_node_snapshot(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass) WHERE node_id = 2";
        let capability_from = "FROM ec_spire_remote_node_capability_plan(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass) WHERE node_id = 2";
        let publish_gate_from = "FROM ec_spire_remote_epoch_publish_gate_summary(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass)";
        let manifest_plan_from = "FROM ec_spire_remote_epoch_manifest_plan(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass)";
        let manifest_summary_from = "FROM ec_spire_remote_epoch_manifest_summary(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass)";
        let coordinator_gate_from = format!(
            "FROM ec_spire_remote_search_coordinator_gate_summary(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let readiness_from = format!(
            "FROM ec_spire_remote_search_target_readiness(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[{selected_pid}], 'strict')"
        );
        let execution_from = format!(
            "FROM ec_spire_remote_search_execution_plan(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let libpq_from = format!(
            "FROM ec_spire_remote_search_libpq_request_plan(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let libpq_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_request_summary(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let connection_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_plan(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let connection_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_summary(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let dispatch_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_plan(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let bind_from = format!(
            "FROM ec_spire_remote_search_libpq_bind_plan(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let bind_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_bind_summary(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let work_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_work_plan(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let work_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_work_summary(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let dispatch_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_summary(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let secret_plan_from = format!(
            "FROM ec_spire_remote_search_libpq_secret_plan(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let secret_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_secret_summary(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let connection_open_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_open_plan(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let connection_open_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_open_summary(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let executor_connection_check_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_connection_check(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let executor_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_readiness(\
             'ec_spire_remote_node_desc_catalog_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );

        let descriptor_state =
            Spi::get_one::<String>(&format!("SELECT descriptor_state {snapshot_from}"))
                .expect("snapshot descriptor query should succeed")
                .expect("descriptor state should exist");
        let descriptor_generation =
            Spi::get_one::<i64>(&format!("SELECT descriptor_generation {snapshot_from}"))
                .expect("snapshot generation query should succeed")
                .expect("descriptor generation should exist");
        let node_status = Spi::get_one::<String>(&format!("SELECT status {snapshot_from}"))
            .expect("snapshot status query should succeed")
            .expect("node status should exist");
        let last_error = Spi::get_one::<String>(&format!("SELECT last_error {snapshot_from}"))
            .expect("snapshot error query should succeed")
            .expect("last error should exist");
        let capability_status = Spi::get_one::<String>(&format!("SELECT status {capability_from}"))
            .expect("capability status query should succeed")
            .expect("capability status should exist");
        let extension_status = Spi::get_one::<String>(&format!(
            "SELECT extension_version_status {capability_from}"
        ))
        .expect("capability extension query should succeed")
        .expect("capability extension status should exist");
        let publish_decision =
            Spi::get_one::<String>(&format!("SELECT publish_decision {publish_gate_from}"))
                .expect("publish decision query should succeed")
                .expect("publish decision should exist");
        let publish_status = Spi::get_one::<String>(&format!("SELECT status {publish_gate_from}"))
            .expect("publish status query should succeed")
            .expect("publish status should exist");
        let manifest_action =
            Spi::get_one::<String>(&format!("SELECT manifest_action {manifest_plan_from}"))
                .expect("manifest action query should succeed")
                .expect("manifest action should exist");
        let manifest_decision =
            Spi::get_one::<String>(&format!("SELECT manifest_decision {manifest_summary_from}"))
                .expect("manifest decision query should succeed")
                .expect("manifest decision should exist");
        let included_manifest_count = Spi::get_one::<i64>(&format!(
            "SELECT included_remote_node_count {manifest_summary_from}"
        ))
        .expect("manifest included count query should succeed")
        .expect("manifest included count should exist");
        let coordinator_libpq_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_dispatch_count {coordinator_gate_from}"
        ))
        .expect("coordinator gate dispatch count query should succeed")
        .expect("coordinator gate dispatch count should exist");
        let coordinator_libpq_dispatch_status = Spi::get_one::<String>(&format!(
            "SELECT libpq_dispatch_status {coordinator_gate_from}"
        ))
        .expect("coordinator gate dispatch status query should succeed")
        .expect("coordinator gate dispatch status should exist");
        let coordinator_status =
            Spi::get_one::<String>(&format!("SELECT status {coordinator_gate_from}"))
                .expect("coordinator gate status query should succeed")
                .expect("coordinator gate status should exist");
        let coordinator_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {coordinator_gate_from}"))
                .expect("coordinator gate blocker query should succeed")
                .expect("coordinator gate blocker should exist");
        let coordinator_executor_status = Spi::get_one::<String>(&format!(
            "SELECT libpq_executor_status {coordinator_gate_from}"
        ))
        .expect("coordinator gate executor status query should succeed")
        .expect("coordinator gate executor status should exist");
        let coordinator_executor_next_step = Spi::get_one::<String>(&format!(
            "SELECT libpq_executor_next_step {coordinator_gate_from}"
        ))
        .expect("coordinator gate executor step query should succeed")
        .expect("coordinator gate executor step should exist");
        let target_status = Spi::get_one::<String>(&format!("SELECT status {readiness_from}"))
            .expect("target readiness query should succeed")
            .expect("target readiness status should exist");
        let execution_status = Spi::get_one::<String>(&format!("SELECT status {execution_from}"))
            .expect("execution plan status query should succeed")
            .expect("execution plan status should exist");
        let execution_transport =
            Spi::get_one::<String>(&format!("SELECT execution_transport {execution_from}"))
                .expect("execution transport query should succeed")
                .expect("execution transport should exist");
        let libpq_status = Spi::get_one::<String>(&format!("SELECT status {libpq_from}"))
            .expect("libpq request status query should succeed")
            .expect("libpq request status should exist");
        let libpq_conninfo_source =
            Spi::get_one::<String>(&format!("SELECT conninfo_source {libpq_from}"))
                .expect("libpq conninfo source query should succeed")
                .expect("libpq conninfo source should exist");
        let libpq_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {libpq_summary_from}"))
                .expect("libpq summary status query should succeed")
                .expect("libpq summary status should exist");
        let libpq_blocked_request_count = Spi::get_one::<i64>(&format!(
            "SELECT blocked_request_count {libpq_summary_from}"
        ))
        .expect("libpq blocked request count query should succeed")
        .expect("libpq blocked request count should exist");
        let conninfo_secret_name =
            Spi::get_one::<String>(&format!("SELECT conninfo_secret_name {connection_from}"))
                .expect("connection secret query should succeed")
                .expect("connection secret should exist");
        let remote_index_regclass =
            Spi::get_one::<String>(&format!("SELECT remote_index_regclass {connection_from}"))
                .expect("connection remote regclass query should succeed")
                .expect("connection remote regclass should exist");
        let remote_index_identity_bytes = Spi::get_one::<i64>(&format!(
            "SELECT remote_index_identity_bytes {connection_from}"
        ))
        .expect("connection identity bytes query should succeed")
        .expect("connection identity bytes should exist");
        let conninfo_resolution =
            Spi::get_one::<String>(&format!("SELECT conninfo_resolution {connection_from}"))
                .expect("connection resolution query should succeed")
                .expect("connection resolution should exist");
        let pipeline_mode =
            Spi::get_one::<String>(&format!("SELECT pipeline_mode {connection_from}"))
                .expect("connection pipeline mode query should succeed")
                .expect("connection pipeline mode should exist");
        let descriptor_resolved_connection_count = Spi::get_one::<i64>(&format!(
            "SELECT descriptor_resolved_connection_count {connection_summary_from}"
        ))
        .expect("connection summary resolved count query should succeed")
        .expect("connection summary resolved count should exist");
        let pipeline_connection_count = Spi::get_one::<i64>(&format!(
            "SELECT pipeline_connection_count {connection_summary_from}"
        ))
        .expect("connection summary pipeline count query should succeed")
        .expect("connection summary pipeline count should exist");
        let connection_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {connection_summary_from}"))
                .expect("connection summary status query should succeed")
                .expect("connection summary status should exist");
        let dispatch_action =
            Spi::get_one::<String>(&format!("SELECT dispatch_action {dispatch_from}"))
                .expect("dispatch action query should succeed")
                .expect("dispatch action should exist");
        let dispatch_receive_validator =
            Spi::get_one::<String>(&format!("SELECT receive_validator {dispatch_from}"))
                .expect("dispatch receive validator query should succeed")
                .expect("dispatch receive validator should exist");
        let bind_count = Spi::get_one::<i64>(&format!("SELECT count(*) {bind_from}"))
            .expect("bind plan count query should succeed")
            .expect("bind plan count should exist");
        let bind_contract_mismatch_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_search_libpq_parameter_contract() contract \
               LEFT JOIN (SELECT * {bind_from}) bind \
                 ON bind.parameter_ordinal = contract.parameter_ordinal \
                AND bind.parameter_name = contract.parameter_name \
                AND bind.pg_type = contract.pg_type \
              WHERE bind.parameter_ordinal IS NULL"
        ))
        .expect("bind contract invariant query should succeed")
        .expect("bind contract invariant count should exist");
        let bind_remote_index_preview = Spi::get_one::<String>(&format!(
            "SELECT value_preview {bind_from} WHERE parameter_name = 'remote_index_oid'"
        ))
        .expect("bind remote index query should succeed")
        .expect("bind remote index preview should exist");
        let bind_query_element_count = Spi::get_one::<i64>(&format!(
            "SELECT element_count {bind_from} WHERE parameter_name = 'query'"
        ))
        .expect("bind query element count query should succeed")
        .expect("bind query element count should exist");
        let bind_selected_pid_count = Spi::get_one::<i64>(&format!(
            "SELECT element_count {bind_from} WHERE parameter_name = 'selected_pids'"
        ))
        .expect("bind selected pid count query should succeed")
        .expect("bind selected pid count should exist");
        let bind_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {bind_from} WHERE value_status = 'ready'"
        ))
        .expect("bind ready count query should succeed")
        .expect("bind ready count should exist");
        let bind_summary_ready_count =
            Spi::get_one::<i64>(&format!("SELECT ready_bind_count {bind_summary_from}"))
                .expect("bind summary ready count query should succeed")
                .expect("bind summary ready count should exist");
        let bind_summary_blocked_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_bind_count {bind_summary_from}"))
                .expect("bind summary blocked count query should succeed")
                .expect("bind summary blocked count should exist");
        let bind_summary_remote_pid_count =
            Spi::get_one::<i64>(&format!("SELECT remote_pid_count {bind_summary_from}"))
                .expect("bind summary remote pid count query should succeed")
                .expect("bind summary remote pid count should exist");
        let bind_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {bind_summary_from}"))
                .expect("bind summary status query should succeed")
                .expect("bind summary status should exist");
        let work_bind_status = Spi::get_one::<String>(&format!("SELECT bind_status {work_from}"))
            .expect("work bind status query should succeed")
            .expect("work bind status should exist");
        let work_action = Spi::get_one::<String>(&format!("SELECT work_action {work_from}"))
            .expect("work action query should succeed")
            .expect("work action should exist");
        let work_status = Spi::get_one::<String>(&format!("SELECT status {work_from}"))
            .expect("work status query should succeed")
            .expect("work status should exist");
        let work_summary_ready_count =
            Spi::get_one::<i64>(&format!("SELECT ready_work_count {work_summary_from}"))
                .expect("work summary ready count query should succeed")
                .expect("work summary ready count should exist");
        let work_summary_blocked_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_work_count {work_summary_from}"))
                .expect("work summary blocked count query should succeed")
                .expect("work summary blocked count should exist");
        let work_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {work_summary_from}"))
                .expect("work summary status query should succeed")
                .expect("work summary status should exist");
        let dispatch_pipeline_count = Spi::get_one::<i64>(&format!(
            "SELECT pipeline_dispatch_count {dispatch_summary_from}"
        ))
        .expect("dispatch summary pipeline count query should succeed")
        .expect("dispatch summary pipeline count should exist");
        let dispatch_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {dispatch_summary_from}"))
                .expect("dispatch summary status query should succeed")
                .expect("dispatch summary status should exist");
        let secret_provider_lookup_key =
            Spi::get_one::<String>(&format!("SELECT provider_lookup_key {secret_plan_from}"))
                .expect("secret provider lookup key query should succeed")
                .expect("secret provider lookup key should exist");
        let secret_plan_status =
            Spi::get_one::<String>(&format!("SELECT status {secret_plan_from}"))
                .expect("secret plan status query should succeed")
                .expect("secret plan status should exist");
        let secret_plan_raw_exposed =
            Spi::get_one::<bool>(&format!("SELECT raw_conninfo_exposed {secret_plan_from}"))
                .expect("secret raw exposure query should succeed")
                .expect("secret raw exposure should exist");
        let secret_plan_resolved_bytes = Spi::get_one::<i64>(&format!(
            "SELECT resolved_conninfo_bytes {secret_plan_from}"
        ))
        .expect("secret resolved bytes query should succeed")
        .expect("secret resolved bytes should exist");
        let secret_plan_action = Spi::get_one::<String>(&format!(
            "SELECT secret_resolution_action {secret_plan_from}"
        ))
        .expect("secret resolution action query should succeed")
        .expect("secret resolution action should exist");
        let secret_plan_next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {secret_plan_from}"))
                .expect("secret next step query should succeed")
                .expect("secret next step should exist");
        let secret_summary_resolved_count = Spi::get_one::<i64>(&format!(
            "SELECT resolved_secret_count {secret_summary_from}"
        ))
        .expect("secret summary resolved count query should succeed")
        .expect("secret summary resolved count should exist");
        let secret_summary_blocked_count = Spi::get_one::<i64>(&format!(
            "SELECT blocked_secret_count {secret_summary_from}"
        ))
        .expect("secret summary blocked count query should succeed")
        .expect("secret summary blocked count should exist");
        let secret_summary_next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {secret_summary_from}"))
                .expect("secret summary next step query should succeed")
                .expect("secret summary next step should exist");
        let secret_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {secret_summary_from}"))
                .expect("secret summary status query should succeed")
                .expect("secret summary status should exist");
        let connection_open_action =
            Spi::get_one::<String>(&format!("SELECT connection_action {connection_open_from}"))
                .expect("connection open action query should succeed")
                .expect("connection open action should exist");
        let connection_open_lifecycle = Spi::get_one::<String>(&format!(
            "SELECT connection_lifecycle_policy {connection_open_from}"
        ))
        .expect("connection open lifecycle query should succeed")
        .expect("connection open lifecycle should exist");
        let connection_open_pooling =
            Spi::get_one::<String>(&format!("SELECT pooling_policy {connection_open_from}"))
                .expect("connection open pooling query should succeed")
                .expect("connection open pooling should exist");
        let connection_open_next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {connection_open_from}"))
                .expect("connection open next step query should succeed")
                .expect("connection open next step should exist");
        let connection_open_status =
            Spi::get_one::<String>(&format!("SELECT status {connection_open_from}"))
                .expect("connection open status query should succeed")
                .expect("connection open status should exist");
        let connection_open_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_connection_count {connection_open_summary_from}"
        ))
        .expect("connection open summary ready count query should succeed")
        .expect("connection open summary ready count should exist");
        let connection_open_summary_blocked_count = Spi::get_one::<i64>(&format!(
            "SELECT blocked_connection_count {connection_open_summary_from}"
        ))
        .expect("connection open summary blocked count query should succeed")
        .expect("connection open summary blocked count should exist");
        let connection_open_summary_next_step = Spi::get_one::<String>(&format!(
            "SELECT next_executor_step {connection_open_summary_from}"
        ))
        .expect("connection open summary next step query should succeed")
        .expect("connection open summary next step should exist");
        let connection_open_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {connection_open_summary_from}"))
                .expect("connection open summary status query should succeed")
                .expect("connection open summary status should exist");
        let executor_connection_attempted = Spi::get_one::<bool>(&format!(
            "SELECT connection_attempted {executor_connection_check_from}"
        ))
        .expect("executor connection check attempted query should succeed")
        .expect("executor connection check attempted should exist");
        let executor_connection_status = Spi::get_one::<String>(&format!(
            "SELECT connection_status {executor_connection_check_from}"
        ))
        .expect("executor connection check status query should succeed")
        .expect("executor connection check status should exist");
        let executor_connection_lookup_kind = Spi::get_one::<String>(&format!(
            "SELECT conninfo_lookup_kind {executor_connection_check_from}"
        ))
        .expect("executor connection check lookup kind query should succeed")
        .expect("executor connection check lookup kind should exist");
        let executor_connection_next_step = Spi::get_one::<String>(&format!(
            "SELECT next_executor_step {executor_connection_check_from}"
        ))
        .expect("executor connection check next step query should succeed")
        .expect("executor connection check next step should exist");
        let executor_connection_terminal_status =
            Spi::get_one::<String>(&format!("SELECT status {executor_connection_check_from}"))
                .expect("executor connection check terminal status query should succeed")
                .expect("executor connection check terminal status should exist");
        let executor_status = Spi::get_one::<String>(&format!("SELECT status {executor_from}"))
            .expect("executor readiness status query should succeed")
            .expect("executor readiness status should exist");
        let executor_next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {executor_from}"))
                .expect("executor readiness next step query should succeed")
                .expect("executor readiness next step should exist");
        let executor_secret_action =
            Spi::get_one::<String>(&format!("SELECT secret_resolution_action {executor_from}"))
                .expect("executor readiness secret action query should succeed")
                .expect("executor readiness secret action should exist");
        let executor_receive_action =
            Spi::get_one::<String>(&format!("SELECT receive_action {executor_from}"))
                .expect("executor readiness receive action query should succeed")
                .expect("executor readiness receive action should exist");
        let executor_contract_mismatch_count = Spi::get_one::<i64>(&format!(
            "WITH readiness AS ( \
                 SELECT * {executor_from} \
             ), expected(step_name, readiness_action) AS ( \
                 VALUES \
                     ('conninfo_secret_resolution', \
                         (SELECT secret_resolution_action FROM readiness)), \
                     ('open_libpq_connection', \
                         (SELECT connection_action FROM readiness)), \
                     ('enter_libpq_pipeline_mode', \
                         (SELECT pipeline_action FROM readiness)), \
                     ('send_remote_search_request', \
                         (SELECT send_action FROM readiness)), \
                     ('validate_remote_search_candidate_batch', \
                         (SELECT receive_action FROM readiness)), \
                     ('merge_validated_remote_search_candidate_batches', \
                         (SELECT merge_action FROM readiness)) \
             ) \
             SELECT count(*) \
               FROM expected \
               LEFT JOIN ec_spire_remote_search_libpq_executor_step_contract() contract \
                 ON contract.step_name = expected.step_name \
              WHERE contract.step_name IS NULL \
                 OR contract.executor_action <> expected.readiness_action"
        ))
        .expect("executor contract invariant query should succeed")
        .expect("executor contract invariant count should exist");

        assert!(register_result);
        assert_eq!(descriptor_state, "active");
        assert_eq!(descriptor_generation, 7);
        assert_eq!(node_status, "ready");
        assert_eq!(last_error, "none");
        assert_eq!(capability_status, "ready");
        assert_eq!(extension_status, "ready");
        assert_eq!(publish_decision, "publish_distributed_epoch");
        assert_eq!(publish_status, "ready");
        assert_eq!(manifest_action, "include_remote_node");
        assert_eq!(manifest_decision, "emit_distributed_epoch_manifest");
        assert_eq!(included_manifest_count, 1);
        assert_eq!(coordinator_libpq_dispatch_count, 1);
        assert_eq!(
            coordinator_libpq_dispatch_status,
            "requires_libpq_transport"
        );
        assert_eq!(coordinator_status, "requires_libpq_executor");
        assert_eq!(coordinator_next_blocker, "open_libpq_connection");
        assert_eq!(coordinator_executor_status, "requires_libpq_executor");
        assert_eq!(coordinator_executor_next_step, "open_libpq_connection");
        assert_eq!(target_status, "requires_libpq_transport");
        assert_eq!(execution_status, "requires_libpq_transport");
        assert_eq!(execution_transport, "libpq_pipeline");
        assert_eq!(libpq_status, "requires_libpq_transport");
        assert_eq!(libpq_conninfo_source, "remote_node_descriptor");
        assert_eq!(libpq_summary_status, "requires_libpq_transport");
        assert_eq!(libpq_blocked_request_count, 1);
        assert_eq!(conninfo_secret_name, "spire/remote/2");
        assert_eq!(remote_index_regclass, "remote_spire_idx");
        assert_eq!(remote_index_identity_bytes, 1);
        assert_eq!(conninfo_resolution, "secret_reference_ready");
        assert_eq!(pipeline_mode, "libpq_pipeline");
        assert_eq!(descriptor_resolved_connection_count, 1);
        assert_eq!(pipeline_connection_count, 1);
        assert_eq!(connection_summary_status, "requires_libpq_transport");
        assert_eq!(dispatch_action, "open_pipeline_and_send_remote_search");
        assert_eq!(
            dispatch_receive_validator,
            "validate_remote_search_candidate_batch"
        );
        assert_eq!(bind_count, 6);
        assert_eq!(bind_contract_mismatch_count, 0);
        assert_eq!(bind_remote_index_preview, "remote_spire_idx");
        assert_eq!(bind_query_element_count, 2);
        assert_eq!(bind_selected_pid_count, 1);
        assert_eq!(bind_ready_count, 6);
        assert_eq!(bind_summary_ready_count, 6);
        assert_eq!(bind_summary_blocked_count, 0);
        assert_eq!(bind_summary_remote_pid_count, 1);
        assert_eq!(bind_summary_status, "ready");
        assert_eq!(work_bind_status, "ready");
        assert_eq!(work_action, "resolve_conninfo_secret_reference");
        assert_eq!(work_status, "requires_libpq_executor");
        assert_eq!(work_summary_ready_count, 1);
        assert_eq!(work_summary_blocked_count, 0);
        assert_eq!(work_summary_status, "requires_libpq_executor");
        assert_eq!(dispatch_pipeline_count, 1);
        assert_eq!(dispatch_summary_status, "requires_libpq_transport");
        assert_eq!(
            secret_provider_lookup_key,
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_2"
        );
        assert_eq!(secret_plan_status, "resolved_conninfo");
        assert!(!secret_plan_raw_exposed);
        assert!(secret_plan_resolved_bytes > 0);
        assert_eq!(secret_plan_action, "resolved_conninfo_secret_reference");
        assert_eq!(secret_plan_next_step, "open_libpq_connection");
        assert_eq!(secret_summary_resolved_count, 1);
        assert_eq!(secret_summary_blocked_count, 0);
        assert_eq!(secret_summary_next_step, "open_libpq_connection");
        assert_eq!(secret_summary_status, "resolved_conninfo");
        assert_eq!(connection_open_action, "open_libpq_connection");
        assert_eq!(connection_open_lifecycle, "per_query");
        assert_eq!(connection_open_pooling, "no_pooling_v1");
        assert_eq!(connection_open_next_step, "enter_libpq_pipeline_mode");
        assert_eq!(connection_open_status, "requires_libpq_executor");
        assert_eq!(connection_open_summary_ready_count, 1);
        assert_eq!(connection_open_summary_blocked_count, 0);
        assert_eq!(
            connection_open_summary_next_step,
            "enter_libpq_pipeline_mode"
        );
        assert_eq!(connection_open_summary_status, "requires_libpq_executor");
        assert!(executor_connection_attempted);
        assert_eq!(executor_connection_status, "libpq_connection_open_failed");
        assert_eq!(executor_connection_lookup_kind, "secret_provider");
        assert_eq!(executor_connection_next_step, "open_libpq_connection");
        assert_eq!(
            executor_connection_terminal_status,
            "libpq_connection_failed"
        );
        assert_eq!(executor_status, "requires_libpq_executor");
        assert_eq!(executor_next_step, "open_libpq_connection");
        assert_eq!(executor_secret_action, "resolve_conninfo_secret_reference");
        assert_eq!(
            executor_receive_action,
            "validate_remote_search_candidate_batch"
        );
        assert_eq!(executor_contract_mismatch_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_libpq_capability_blocks() {
        fn assert_capability_block(
            prefix: &str,
            node_id: i32,
            generation: i64,
            expected_status: &str,
            expected_epoch_status: &str,
            expected_extension_status: &str,
            expected_blocker: &str,
            extension_version: &str,
            consistency_mode: &str,
            expected_failure_action: &str,
        ) {
            let table_name = format!("ec_spire_cap_{prefix}_sql");
            let index_name = format!("ec_spire_cap_{prefix}_idx");
            Spi::run(&format!(
                "CREATE TABLE {table_name} \
                 (id bigint primary key, embedding ecvector)"
            ))
            .expect("table creation should succeed");
            Spi::run(&format!(
                "INSERT INTO {table_name} (id, embedding) VALUES \
                 (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                 (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))"
            ))
            .expect("insert should succeed");
            Spi::run(&format!(
                "CREATE INDEX {index_name} \
                 ON {table_name} USING ec_spire \
                 (embedding ecvector_spire_ip_ops) WITH (nlists = 2)"
            ))
            .expect("ec_spire index creation should succeed");

            let index_oid =
                Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                    .expect("index oid query should succeed")
                    .expect("index oid should exist");
            let active_epoch = Spi::get_one::<i64>(&format!(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('{index_name}'::regclass)"
            ))
            .expect("hierarchy snapshot query should succeed")
            .expect("active epoch should exist");
            let selected_pid = Spi::get_one::<i64>(&format!(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('{index_name}'::regclass)"
            ))
            .expect("leaf snapshot query should succeed")
            .expect("leaf pid should exist");
            let last_served_epoch = if expected_status == "stale_epoch" {
                active_epoch.saturating_sub(1)
            } else {
                active_epoch
            };
            let min_retained_epoch = if expected_status == "retention_gap" {
                active_epoch
                    .checked_add(1)
                    .expect("test active epoch should allow retention gap")
            } else {
                active_epoch
            };
            assert!(last_served_epoch <= active_epoch);

            if consistency_mode == "degraded" {
                unsafe { am::debug_spire_rewrite_consistency_mode(index_oid, "degraded") };
            }
            unsafe {
                am::debug_spire_rewrite_placement_node(
                    index_oid,
                    selected_pid as u64,
                    node_id as u32,
                )
            };
            let register_result = Spi::get_one::<bool>(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                         '{}'::oid, {node_id}, {generation}, 'spire/remote/{prefix}', \
                         decode('0a', 'hex'), 'remote_spire_idx', 'active', \
                         {last_served_epoch}, {min_retained_epoch}, '{extension_version}', 'none')",
                u32::from(index_oid)
            ))
            .expect("remote descriptor registration should succeed")
            .expect("remote descriptor registration result should exist");
            assert!(register_result);

            let capability_from = format!(
                "FROM ec_spire_remote_node_capability_plan('{index_name}'::regclass) \
                 WHERE node_id = {node_id}"
            );
            let target_from = format!(
                "FROM ec_spire_remote_search_target_readiness(\
                 '{index_name}'::regclass, {active_epoch}, \
                 ARRAY[{selected_pid}]::bigint[], '{consistency_mode}') \
                 WHERE node_id = {node_id}"
            );
            let args = format!(
                "'{index_name}'::regclass, {active_epoch}, ARRAY[1.0, 0.0]::real[], \
                 ARRAY[{selected_pid}]::bigint[], 1, '{consistency_mode}'"
            );
            let execution_from = format!("FROM ec_spire_remote_search_execution_plan({args})");
            let libpq_from = format!("FROM ec_spire_remote_search_libpq_request_plan({args})");
            let connection_from =
                format!("FROM ec_spire_remote_search_libpq_connection_plan({args})");
            let dispatch_from = format!("FROM ec_spire_remote_search_libpq_dispatch_plan({args})");
            let dispatch_summary_from =
                format!("FROM ec_spire_remote_search_libpq_dispatch_summary({args})");
            let bind_summary_from =
                format!("FROM ec_spire_remote_search_libpq_bind_summary({args})");
            let secret_from = format!("FROM ec_spire_remote_search_libpq_secret_plan({args})");
            let work_summary_from =
                format!("FROM ec_spire_remote_search_libpq_executor_work_summary({args})");
            let executor_from =
                format!("FROM ec_spire_remote_search_libpq_executor_readiness({args})");
            let receive_attempts_from =
                format!("FROM ec_spire_remote_search_libpq_executor_receive_attempts({args})");
            let gate_from = format!("FROM ec_spire_remote_search_coordinator_gate_summary({args})");
            let heap_resolution_from =
                format!("FROM ec_spire_remote_search_heap_resolution_summary({args})");
            let identity_cache_from =
                format!("FROM ec_spire_remote_search_libpq_identity_cache_summary({args})");

            let capability_status =
                Spi::get_one::<String>(&format!("SELECT status {capability_from}"))
                    .expect("capability status query should succeed")
                    .expect("capability status should exist");
            let epoch_status =
                Spi::get_one::<String>(&format!("SELECT epoch_window_status {capability_from}"))
                    .expect("capability epoch query should succeed")
                    .expect("capability epoch status should exist");
            let extension_status = Spi::get_one::<String>(&format!(
                "SELECT extension_version_status {capability_from}"
            ))
            .expect("capability extension query should succeed")
            .expect("capability extension status should exist");
            let target_status = Spi::get_one::<String>(&format!("SELECT status {target_from}"))
                .expect("target status query should succeed")
                .expect("target status should exist");
            let execution_status =
                Spi::get_one::<String>(&format!("SELECT status {execution_from}"))
                    .expect("execution status query should succeed")
                    .expect("execution status should exist");
            let libpq_status = Spi::get_one::<String>(&format!("SELECT status {libpq_from}"))
                .expect("libpq request status query should succeed")
                .expect("libpq request status should exist");
            let pipeline_mode =
                Spi::get_one::<String>(&format!("SELECT pipeline_mode {connection_from}"))
                    .expect("connection pipeline query should succeed")
                    .expect("connection pipeline should exist");
            let conninfo_resolution =
                Spi::get_one::<String>(&format!("SELECT conninfo_resolution {connection_from}"))
                    .expect("connection resolution query should succeed")
                    .expect("connection resolution should exist");
            let dispatch_action =
                Spi::get_one::<String>(&format!("SELECT dispatch_action {dispatch_from}"))
                    .expect("dispatch action query should succeed")
                    .expect("dispatch action should exist");
            let dispatch_summary_status =
                Spi::get_one::<String>(&format!("SELECT status {dispatch_summary_from}"))
                    .expect("dispatch summary status query should succeed")
                    .expect("dispatch summary status should exist");
            let pipeline_dispatch_count = Spi::get_one::<i64>(&format!(
                "SELECT pipeline_dispatch_count {dispatch_summary_from}"
            ))
            .expect("dispatch summary pipeline query should succeed")
            .expect("dispatch summary pipeline count should exist");
            let bind_summary_status =
                Spi::get_one::<String>(&format!("SELECT status {bind_summary_from}"))
                    .expect("bind summary status query should succeed")
                    .expect("bind summary status should exist");
            let bind_blocked_count =
                Spi::get_one::<i64>(&format!("SELECT blocked_bind_count {bind_summary_from}"))
                    .expect("bind blocked query should succeed")
                    .expect("bind blocked count should exist");
            let secret_status = Spi::get_one::<String>(&format!("SELECT status {secret_from}"))
                .expect("secret status query should succeed")
                .expect("secret status should exist");
            let secret_next_step =
                Spi::get_one::<String>(&format!("SELECT next_executor_step {secret_from}"))
                    .expect("secret next step query should succeed")
                    .expect("secret next step should exist");
            let secret_provider =
                Spi::get_one::<String>(&format!("SELECT provider_lookup_key {secret_from}"))
                    .expect("secret provider query should succeed")
                    .expect("secret provider should exist");
            let work_status = Spi::get_one::<String>(&format!("SELECT status {work_summary_from}"))
                .expect("work status query should succeed")
                .expect("work status should exist");
            let work_next_step =
                Spi::get_one::<String>(&format!("SELECT next_executor_step {work_summary_from}"))
                    .expect("work next step query should succeed")
                    .expect("work next step should exist");
            let executor_status = Spi::get_one::<String>(&format!("SELECT status {executor_from}"))
                .expect("executor status query should succeed")
                .expect("executor status should exist");
            let executor_next_step =
                Spi::get_one::<String>(&format!("SELECT next_executor_step {executor_from}"))
                    .expect("executor next step query should succeed")
                    .expect("executor next step should exist");
            let receive_status =
                Spi::get_one::<String>(&format!("SELECT status {receive_attempts_from}"))
                    .expect("receive status query should succeed")
                    .expect("receive status should exist");
            let receive_next_blocker =
                Spi::get_one::<String>(&format!("SELECT next_blocker {receive_attempts_from}"))
                    .expect("receive next blocker query should succeed")
                    .expect("receive next blocker should exist");
            let receive_failure_action =
                Spi::get_one::<String>(&format!("SELECT failure_action {receive_attempts_from}"))
                    .expect("receive failure action query should succeed")
                    .expect("receive failure action should exist");
            let gate_status = Spi::get_one::<String>(&format!("SELECT status {gate_from}"))
                .expect("gate status query should succeed")
                .expect("gate status should exist");
            let gate_next_blocker =
                Spi::get_one::<String>(&format!("SELECT next_blocker {gate_from}"))
                    .expect("gate next blocker query should succeed")
                    .expect("gate next blocker should exist");
            let gate_executor_next_step =
                Spi::get_one::<String>(&format!("SELECT libpq_executor_next_step {gate_from}"))
                    .expect("gate executor step query should succeed")
                    .expect("gate executor step should exist");
            let heap_remote_status = Spi::get_one::<String>(&format!(
                "SELECT remote_heap_resolution_status {heap_resolution_from}"
            ))
            .expect("heap resolution status query should succeed")
            .expect("heap resolution status should exist");
            let identity_cache_status =
                Spi::get_one::<String>(&format!("SELECT status {identity_cache_from}"))
                    .expect("identity cache status query should succeed")
                    .expect("identity cache status should exist");
            let identity_cache_compact_count = Spi::get_one::<i64>(&format!(
                "SELECT compact_candidate_count {identity_cache_from}"
            ))
            .expect("identity cache compact count query should succeed")
            .expect("identity cache compact count should exist");
            let identity_cache_heap_count = Spi::get_one::<i64>(&format!(
                "SELECT heap_candidate_count {identity_cache_from}"
            ))
            .expect("identity cache heap count query should succeed")
            .expect("identity cache heap count should exist");
            let identity_cache_entries = Spi::get_one::<i64>(&format!(
                "SELECT endpoint_identity_cache_entry_count {identity_cache_from}"
            ))
            .expect("identity cache entry count query should succeed")
            .expect("identity cache entry count should exist");
            let identity_cache_queries = Spi::get_one::<i64>(&format!(
                "SELECT endpoint_identity_query_count {identity_cache_from}"
            ))
            .expect("identity cache query count query should succeed")
            .expect("identity cache query count should exist");
            let identity_cache_hits = Spi::get_one::<i64>(&format!(
                "SELECT endpoint_identity_cache_hit_count {identity_cache_from}"
            ))
            .expect("identity cache hit count query should succeed")
            .expect("identity cache hit count should exist");
            let identity_cache_misses = Spi::get_one::<i64>(&format!(
                "SELECT endpoint_identity_cache_miss_count {identity_cache_from}"
            ))
            .expect("identity cache miss count query should succeed")
            .expect("identity cache miss count should exist");

            assert_eq!(capability_status, expected_status);
            assert_eq!(epoch_status, expected_epoch_status);
            assert_eq!(extension_status, expected_extension_status);
            assert_eq!(target_status, expected_status);
            assert_eq!(execution_status, expected_status);
            assert_eq!(libpq_status, expected_status);
            assert_eq!(pipeline_mode, "none");
            assert_eq!(conninfo_resolution, "secret_reference_ready");
            assert_eq!(dispatch_action, "blocked_before_dispatch");
            assert_eq!(dispatch_summary_status, expected_status);
            assert_eq!(pipeline_dispatch_count, 0);
            assert_eq!(bind_summary_status, expected_status);
            assert_eq!(bind_blocked_count, 6);
            assert_eq!(secret_status, expected_status);
            assert_eq!(secret_next_step, expected_blocker);
            assert_eq!(secret_provider, "none");
            assert_eq!(work_status, expected_status);
            assert_eq!(work_next_step, expected_blocker);
            assert_eq!(executor_status, expected_status);
            assert_eq!(executor_next_step, expected_blocker);
            assert_eq!(receive_status, expected_status);
            assert_eq!(receive_next_blocker, expected_blocker);
            assert_eq!(receive_failure_action, expected_failure_action);
            assert_eq!(gate_status, expected_status);
            assert_eq!(gate_next_blocker, expected_blocker);
            assert_eq!(gate_executor_next_step, expected_blocker);
            assert_eq!(heap_remote_status, expected_status);
            assert_eq!(identity_cache_status, expected_status);
            assert_eq!(identity_cache_compact_count, 0);
            assert_eq!(identity_cache_heap_count, 0);
            assert_eq!(identity_cache_entries, 0);
            assert_eq!(identity_cache_queries, 0);
            assert_eq!(identity_cache_hits, 0);
            assert_eq!(identity_cache_misses, 0);
        }

        assert_capability_block(
            "stale_strict",
            2,
            30,
            "stale_epoch",
            "stale_epoch",
            "ready",
            "remote_epoch_window",
            env!("CARGO_PKG_VERSION"),
            "strict",
            "fail_closed",
        );
        assert_capability_block(
            "version_strict",
            3,
            31,
            "incompatible_extension_version",
            "ready",
            "incompatible_extension_version",
            "remote_extension_version",
            "0.0.0-test-skew",
            "strict",
            "fail_closed",
        );
        assert_capability_block(
            "retention_strict",
            4,
            34,
            "retention_gap",
            "retention_gap",
            "ready",
            "remote_epoch_window",
            env!("CARGO_PKG_VERSION"),
            "strict",
            "fail_closed",
        );
        assert_capability_block(
            "stale_degraded",
            2,
            32,
            "stale_epoch",
            "stale_epoch",
            "ready",
            "remote_epoch_window",
            env!("CARGO_PKG_VERSION"),
            "degraded",
            "skip_node",
        );
        assert_capability_block(
            "retention_degraded",
            4,
            35,
            "retention_gap",
            "retention_gap",
            "ready",
            "remote_epoch_window",
            env!("CARGO_PKG_VERSION"),
            "degraded",
            "skip_node",
        );
        assert_capability_block(
            "version_degraded",
            3,
            33,
            "incompatible_extension_version",
            "ready",
            "incompatible_extension_version",
            "remote_extension_version",
            "0.0.0-test-skew",
            "degraded",
            "skip_node",
        );
    }

    #[pg_test]
    fn test_ec_spire_libpq_executor_budget_limits() {
        Spi::run("SET LOCAL ec_spire.remote_search_max_nodes = 1")
            .expect("max node budget SET should succeed");
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches = 3")
            .expect("max concurrent dispatch budget SET should succeed");
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches_per_node = 2")
            .expect("max concurrent dispatch per-node budget SET should succeed");
        Spi::run("SET LOCAL ec_spire.remote_search_connect_timeout_ms = 25")
            .expect("connect timeout SET should succeed");
        Spi::run("SET LOCAL ec_spire.remote_search_statement_timeout_ms = 75")
            .expect("statement timeout SET should succeed");
        Spi::run(
            "CREATE TABLE ec_spire_libpq_budget_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_libpq_budget_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_libpq_budget_idx \
             ON ec_spire_libpq_budget_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_libpq_budget_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_libpq_budget_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let args = format!(
            "'ec_spire_libpq_budget_idx'::regclass, {active_epoch}, \
             ARRAY[1.0, 0.0]::real[], ARRAY[{}, {}]::bigint[], 1, 'strict'",
            selected_pids[0], selected_pids[1],
        );
        let budget_from =
            format!("FROM ec_spire_remote_search_libpq_executor_budget_summary({args})");

        let (
            probe_dispatch_count,
            probe_admitted_dispatch_count,
            probe_budget_blocked_dispatch_count,
            probe_admitted_pid_count,
            probe_budget_blocked_pid_count,
            probe_secret_budget_blocked_count,
            probe_budget_status,
            probe_secret_next_step,
        ) = am::spire_remote_search_libpq_executor_budget_contract_probe_counts();
        let sql_dispatch_count =
            Spi::get_one::<i64>(&format!("SELECT dispatch_count {budget_from}"))
                .expect("budget dispatch query should succeed")
                .expect("budget dispatch count should exist");
        let budget_status = Spi::get_one::<String>(&format!("SELECT status {budget_from}"))
            .expect("budget status query should succeed")
            .expect("budget status should exist");
        let budget_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {budget_from}"))
                .expect("budget next-step query should succeed")
                .expect("budget next-step should exist");
        let admitted_dispatch_count =
            Spi::get_one::<i64>(&format!("SELECT admitted_dispatch_count {budget_from}"))
                .expect("budget admitted dispatch query should succeed")
                .expect("budget admitted dispatch count should exist");
        let budget_blocked_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT budget_blocked_dispatch_count {budget_from}"
        ))
        .expect("budget blocked dispatch query should succeed")
        .expect("budget blocked dispatch count should exist");
        let max_nodes = Spi::get_one::<i64>(&format!("SELECT max_nodes {budget_from}"))
            .expect("budget max node query should succeed")
            .expect("budget max node should exist");
        let max_concurrent_dispatches =
            Spi::get_one::<i64>(&format!("SELECT max_concurrent_dispatches {budget_from}"))
                .expect("budget max concurrent dispatches query should succeed")
                .expect("budget max concurrent dispatches should exist");
        let max_concurrent_dispatches_per_node = Spi::get_one::<i64>(&format!(
            "SELECT max_concurrent_dispatches_per_node {budget_from}"
        ))
        .expect("budget max concurrent dispatches per node query should succeed")
        .expect("budget max concurrent dispatches per node should exist");
        let connect_timeout_ms =
            Spi::get_one::<i64>(&format!("SELECT connect_timeout_ms {budget_from}"))
                .expect("connect timeout query should succeed")
                .expect("connect timeout should exist");
        let statement_timeout_ms =
            Spi::get_one::<i64>(&format!("SELECT statement_timeout_ms {budget_from}"))
                .expect("statement timeout query should succeed")
                .expect("statement timeout should exist");

        assert_eq!(probe_dispatch_count, 2);
        assert_eq!(probe_admitted_dispatch_count, 1);
        assert_eq!(probe_budget_blocked_dispatch_count, 1);
        assert_eq!(probe_admitted_pid_count, 1);
        assert_eq!(probe_budget_blocked_pid_count, 1);
        assert_eq!(probe_secret_budget_blocked_count, 1);
        assert_eq!(probe_budget_status, "remote_executor_overload");
        assert_eq!(probe_secret_next_step, "remote_executor_budget");
        assert_eq!(sql_dispatch_count, 0);
        assert_eq!(budget_status, "ready");
        assert_eq!(budget_step, "none");
        assert_eq!(admitted_dispatch_count, 0);
        assert_eq!(budget_blocked_dispatch_count, 0);
        assert_eq!(max_nodes, 1);
        assert_eq!(max_concurrent_dispatches, 3);
        assert_eq!(max_concurrent_dispatches_per_node, 2);
        assert_eq!(connect_timeout_ms, 25);
        assert_eq!(statement_timeout_ms, 75);
    }
