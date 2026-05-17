    #[pg_test]
    fn test_ec_spire_libpq_executor_global_governance_overload() {
        set_remote_governance_test_namespace(6605);
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches = 1")
            .expect("max concurrent dispatch budget SET should succeed");
        let (class_id, object_id) =
            am::remote_search_libpq_global_governance_advisory_key_for_test(0);
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut lock_holder = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback lock-holder connection should succeed");
        lock_holder
            .batch_execute(&format!("SELECT pg_advisory_lock({class_id}, {object_id})"))
            .expect("global governance advisory lock should be held by separate backend");

        Spi::run(
            "CREATE TABLE ec_spire_libpq_governance_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_libpq_governance_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_libpq_governance_idx \
             ON ec_spire_libpq_governance_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_spire_libpq_governance_idx'::regclass::oid")
                .expect("index oid query should succeed")
                .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_libpq_governance_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_libpq_governance_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2);
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 31, 'spire/remote/governance-overload', decode('ff', 'hex'), \
                     'ec_spire_remote_governance_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        let receive_attempts_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_receive_attempts(\
                 'ec_spire_libpq_governance_idx'::regclass, \
                 {active_epoch}, ARRAY[1.0, 0.0]::real[], \
                 ARRAY[{selected_pid}]::bigint[], 1, 'degraded')"
        );
        let receive_attempt_status =
            Spi::get_one::<String>(&format!("SELECT status {receive_attempts_from}"))
                .expect("receive attempt status query should succeed")
                .expect("receive attempt status should exist");
        let receive_attempt_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {receive_attempts_from}"))
                .expect("receive attempt blocker query should succeed")
                .expect("receive attempt blocker should exist");
        let receive_attempt_action =
            Spi::get_one::<String>(&format!("SELECT failure_action {receive_attempts_from}"))
                .expect("receive attempt action query should succeed")
                .expect("receive attempt action should exist");
        let receive_attempt_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT candidate_count {receive_attempts_from}"))
                .expect("receive attempt candidate count query should succeed")
                .expect("receive attempt candidate count should exist");

        assert_eq!(receive_attempt_status, "remote_executor_overload");
        assert_eq!(receive_attempt_blocker, "remote_executor_governance");
        assert_eq!(receive_attempt_action, "skip_node");
        assert_eq!(receive_attempt_candidate_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_libpq_executor_per_node_governance_isolated() {
        set_remote_governance_test_namespace(6606);
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches_per_node = 1")
            .expect("max concurrent per-node dispatch budget SET should succeed");
        let (class_id, object_id) =
            am::remote_search_libpq_node_governance_advisory_key_for_test(2, 0);
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut lock_holder = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback per-node lock-holder connection should succeed");
        lock_holder
            .batch_execute(&format!("SELECT pg_advisory_lock({class_id}, {object_id})"))
            .expect("node 2 governance advisory lock should be held by separate backend");

        Spi::run(
            "CREATE TABLE ec_spire_libpq_per_node_governance_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_libpq_per_node_governance_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_libpq_per_node_governance_idx \
             ON ec_spire_libpq_per_node_governance_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_libpq_per_node_governance_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_libpq_per_node_governance_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_libpq_per_node_governance_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid array should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_nodes(
                index_oid,
                &[(selected_pids[0] as u64, 2), (selected_pids[1] as u64, 3)],
            );
        }
        for (node_id, generation, secret_name) in [
            (2, 41, "spire/remote/per-node-governance/2"),
            (3, 42, "spire/remote/per-node-governance/3"),
        ] {
            let register_result = Spi::get_one::<bool>(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                         '{}'::oid, {node_id}, {generation}, '{secret_name}', \
                         decode('ff', 'hex'), 'ec_spire_remote_per_node_governance_idx', \
                         'active', {active_epoch}, {active_epoch}, '{}', 'none')",
                u32::from(index_oid),
                env!("CARGO_PKG_VERSION")
            ))
            .expect("remote descriptor registration should succeed")
            .expect("remote descriptor registration result should exist");
            assert!(register_result);
        }

        let receive_attempts_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_receive_attempts(\
                 'ec_spire_libpq_per_node_governance_idx'::regclass, \
                 {active_epoch}, ARRAY[1.0, 0.0]::real[], \
                 ARRAY[{}, {}]::bigint[], 1, 'degraded')",
            selected_pids[0], selected_pids[1]
        );
        let node2_status = Spi::get_one::<String>(&format!(
            "SELECT status {receive_attempts_from} WHERE node_id = 2"
        ))
        .expect("node 2 receive attempt status query should succeed")
        .expect("node 2 receive attempt status should exist");
        let node2_blocker = Spi::get_one::<String>(&format!(
            "SELECT next_blocker {receive_attempts_from} WHERE node_id = 2"
        ))
        .expect("node 2 receive attempt blocker query should succeed")
        .expect("node 2 receive attempt blocker should exist");
        let node3_status = Spi::get_one::<String>(&format!(
            "SELECT status {receive_attempts_from} WHERE node_id = 3"
        ))
        .expect("node 3 receive attempt status query should succeed")
        .expect("node 3 receive attempt status should exist");
        let node3_blocker = Spi::get_one::<String>(&format!(
            "SELECT next_blocker {receive_attempts_from} WHERE node_id = 3"
        ))
        .expect("node 3 receive attempt blocker query should succeed")
        .expect("node 3 receive attempt blocker should exist");

        assert_eq!(node2_status, "remote_executor_overload");
        assert_eq!(node2_blocker, "remote_executor_governance");
        assert_eq!(node3_status, "requires_conninfo_secret_resolution");
        assert_eq!(node3_blocker, "conninfo_secret_resolution");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_libpq_executor_loopback_empty() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_LOOPBACK",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_remote_executor_loopback_remote_sql; \
                 CREATE TABLE ec_spire_remote_executor_loopback_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_remote_executor_loopback_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_remote_executor_loopback_remote_sql_idx \
                     ON ec_spire_remote_executor_loopback_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq')",
            )
            .expect("loopback remote fixture should be created");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_remote_executor_loopback_remote_sql_idx",
        );

        Spi::run(
            "CREATE TABLE ec_spire_remote_executor_loopback_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_executor_loopback_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_executor_loopback_coord_sql_idx \
             ON ec_spire_remote_executor_loopback_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_executor_loopback_coord_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_executor_loopback_coord_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_executor_loopback_coord_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");
        let local_pid = Spi::get_one::<i64>(
            "SELECT max(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_executor_loopback_coord_sql_idx'::regclass)",
        )
        .expect("leaf snapshot local PID query should succeed")
        .expect("leaf local PID should exist");
        assert_ne!(selected_pid, local_pid);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 8, 'spire/remote/loopback', decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_remote_executor_loopback_remote_sql_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");

        let args = format!(
            "'ec_spire_remote_executor_loopback_coord_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 0, 'strict'"
        );
        let nonempty_args = format!(
            "'ec_spire_remote_executor_loopback_coord_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'strict'"
        );
        let mixed_args = format!(
            "'ec_spire_remote_executor_loopback_coord_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}, {local_pid}]::bigint[], 2, 'strict'"
        );
        let connection_check_from =
            format!("FROM ec_spire_remote_search_libpq_executor_connection_check({args})");
        let nonempty_connection_check_from =
            format!("FROM ec_spire_remote_search_libpq_executor_connection_check({nonempty_args})");
        let candidates_from =
            format!("FROM ec_spire_remote_search_libpq_executor_candidates({args})");
        let nonempty_candidates_from =
            format!("FROM ec_spire_remote_search_libpq_executor_candidates({nonempty_args})");
        let heap_candidates_from =
            format!("FROM ec_spire_remote_search_libpq_executor_heap_candidates({nonempty_args})");
        let heap_candidate_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_heap_candidate_summary({nonempty_args})"
        );
        let result_summary_from =
            format!("FROM ec_spire_remote_search_coordinator_result_summary({nonempty_args})");
        let mixed_result_summary_from =
            format!("FROM ec_spire_remote_search_coordinator_result_summary({mixed_args})");
        let pipeline_steps_from = format!("FROM ec_spire_remote_pipeline_steps({nonempty_args})");
        let pipeline_steps_live_from =
            format!("FROM ec_spire_remote_pipeline_steps_live({nonempty_args})");
        let identity_cache_summary_from =
            format!("FROM ec_spire_remote_search_libpq_identity_cache_summary({nonempty_args})");
        let manifest_result_from =
            "FROM ec_spire_remote_epoch_manifest_publication_result_summary(\
             'ec_spire_remote_executor_loopback_coord_sql_idx'::regclass)";
        let connection_status =
            Spi::get_one::<String>(&format!("SELECT connection_status {connection_check_from}"))
                .expect("executor connection status query should succeed")
                .expect("executor connection status should exist");
        let connection_attempted = Spi::get_one::<bool>(&format!(
            "SELECT connection_attempted {connection_check_from}"
        ))
        .expect("executor connection attempted query should succeed")
        .expect("executor connection attempted should exist");
        let conninfo_lookup_kind = Spi::get_one::<String>(&format!(
            "SELECT conninfo_lookup_kind {connection_check_from}"
        ))
        .expect("executor connection lookup kind query should succeed")
        .expect("executor connection lookup kind should exist");
        let candidate_count = Spi::get_one::<i64>(&format!("SELECT count(*) {candidates_from}"))
            .expect("executor candidate count query should succeed")
            .expect("executor candidate count should exist");
        let nonempty_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {nonempty_candidates_from}"))
                .expect("executor nonempty candidate count query should succeed")
                .expect("executor nonempty candidate count should exist");
        let nonempty_candidate_node = Spi::get_one::<i64>(&format!(
            "SELECT node_id {nonempty_candidates_from} LIMIT 1"
        ))
        .expect("executor nonempty candidate node query should succeed")
        .expect("executor nonempty candidate node should exist");
        let nonempty_candidate_epoch = Spi::get_one::<i64>(&format!(
            "SELECT served_epoch {nonempty_candidates_from} LIMIT 1"
        ))
        .expect("executor nonempty candidate epoch query should succeed")
        .expect("executor nonempty candidate epoch should exist");
        let nonempty_candidate_locator_bytes = Spi::get_one::<i32>(&format!(
            "SELECT length(row_locator) {nonempty_candidates_from} LIMIT 1"
        ))
        .expect("executor nonempty candidate locator query should succeed")
        .expect("executor nonempty candidate locator should exist");
        let heap_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {heap_candidates_from}"))
                .expect("executor heap candidate count query should succeed")
                .expect("executor heap candidate count should exist");
        let heap_candidate_node =
            Spi::get_one::<i64>(&format!("SELECT node_id {heap_candidates_from} LIMIT 1"))
                .expect("executor heap candidate node query should succeed")
                .expect("executor heap candidate node should exist");
        let heap_candidate_offset = Spi::get_one::<i32>(&format!(
            "SELECT heap_offset {heap_candidates_from} LIMIT 1"
        ))
        .expect("executor heap candidate offset query should succeed")
        .expect("executor heap candidate offset should exist");
        let heap_candidate_owner = Spi::get_one::<String>(&format!(
            "SELECT heap_lookup_owner {heap_candidates_from} LIMIT 1"
        ))
        .expect("executor heap candidate owner query should succeed")
        .expect("executor heap candidate owner should exist");
        let heap_summary_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {heap_candidate_summary_from}"
        ))
        .expect("executor heap summary count query should succeed")
        .expect("executor heap summary count should exist");
        let heap_summary_source = Spi::get_one::<String>(&format!(
            "SELECT result_source {heap_candidate_summary_from}"
        ))
        .expect("executor heap summary source query should succeed")
        .expect("executor heap summary source should exist");
        let heap_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {heap_candidate_summary_from}"))
                .expect("executor heap summary status query should succeed")
                .expect("executor heap summary status should exist");
        let heap_summary_contract = Spi::get_one::<String>(
            "SELECT validator FROM ec_spire_remote_search_coordinator_result_contract() \
             WHERE result_source = 'remote_heap_candidates'",
        )
        .expect("executor heap summary contract query should succeed")
        .expect("executor heap summary contract should exist");
        let coordinator_result_source =
            Spi::get_one::<String>(&format!("SELECT result_source {result_summary_from}"))
                .expect("coordinator remote result source query should succeed")
                .expect("coordinator remote result source should exist");
        let coordinator_returned_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {result_summary_from}"
        ))
        .expect("coordinator remote returned count query should succeed")
        .expect("coordinator remote returned count should exist");
        let coordinator_final_heap_status = Spi::get_one::<String>(&format!(
            "SELECT final_heap_fetch_status {result_summary_from}"
        ))
        .expect("coordinator remote final heap status query should succeed")
        .expect("coordinator remote final heap status should exist");
        let coordinator_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {result_summary_from}"))
                .expect("coordinator remote next blocker query should succeed")
                .expect("coordinator remote next blocker should exist");
        let coordinator_status =
            Spi::get_one::<String>(&format!("SELECT status {result_summary_from}"))
                .expect("coordinator remote status query should succeed")
                .expect("coordinator remote status should exist");
        let mixed_coordinator_result_source =
            Spi::get_one::<String>(&format!("SELECT result_source {mixed_result_summary_from}"))
                .expect("mixed coordinator result source query should succeed")
                .expect("mixed coordinator result source should exist");
        let mixed_coordinator_returned_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {mixed_result_summary_from}"
        ))
        .expect("mixed coordinator returned count query should succeed")
        .expect("mixed coordinator returned count should exist");
        let mixed_coordinator_decoded_count = Spi::get_one::<i64>(&format!(
            "SELECT decoded_local_locator_count {mixed_result_summary_from}"
        ))
        .expect("mixed coordinator decoded count query should succeed")
        .expect("mixed coordinator decoded count should exist");
        let mixed_coordinator_final_heap_status = Spi::get_one::<String>(&format!(
            "SELECT final_heap_fetch_status {mixed_result_summary_from}"
        ))
        .expect("mixed coordinator final heap status query should succeed")
        .expect("mixed coordinator final heap status should exist");
        let mixed_coordinator_status =
            Spi::get_one::<String>(&format!("SELECT status {mixed_result_summary_from}"))
                .expect("mixed coordinator status query should succeed")
                .expect("mixed coordinator status should exist");
        let pipeline_step_names = Spi::get_one::<Vec<String>>(&format!(
            "SELECT array_agg(step_name ORDER BY step_ordinal) {pipeline_steps_from}"
        ))
        .expect("pipeline step names query should succeed")
        .expect("pipeline step names should exist");
        let pipeline_live_step_names = Spi::get_one::<Vec<String>>(&format!(
            "SELECT array_agg(step_name ORDER BY step_ordinal) {pipeline_steps_live_from}"
        ))
        .expect("live pipeline step names query should succeed")
        .expect("live pipeline step names should exist");
        let dispatch_summary_status = Spi::get_one::<String>(&format!(
            "SELECT status FROM ec_spire_remote_search_libpq_dispatch_summary({nonempty_args})"
        ))
        .expect("dispatch summary status query should succeed")
        .expect("dispatch summary status should exist");
        let pipeline_dispatch_status = Spi::get_one::<String>(&format!(
            "SELECT status {pipeline_steps_from} WHERE step_name = 'dispatch_plan'"
        ))
        .expect("pipeline dispatch status query should succeed")
        .expect("pipeline dispatch status should exist");
        let nonempty_connection_terminal_status =
            Spi::get_one::<String>(&format!("SELECT status {nonempty_connection_check_from}"))
                .expect("nonempty connection terminal status query should succeed")
                .expect("nonempty connection terminal status should exist");
        let pipeline_connection_status = Spi::get_one::<String>(&format!(
            "SELECT status {pipeline_steps_from} WHERE step_name = 'connection_check'"
        ))
        .expect("pipeline connection status query should succeed")
        .expect("pipeline connection status should exist");
        let pipeline_live_connection_status = Spi::get_one::<String>(&format!(
            "SELECT status {pipeline_steps_live_from} WHERE step_name = 'connection_check'"
        ))
        .expect("live pipeline connection status query should succeed")
        .expect("live pipeline connection status should exist");
        let pipeline_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT item_count {pipeline_steps_from} WHERE step_name = 'candidates'"
        ))
        .expect("pipeline candidate count query should succeed")
        .expect("pipeline candidate count should exist");
        let pipeline_live_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT item_count {pipeline_steps_live_from} WHERE step_name = 'candidates'"
        ))
        .expect("live pipeline candidate count query should succeed")
        .expect("live pipeline candidate count should exist");
        let pipeline_heap_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT item_count {pipeline_steps_from} WHERE step_name = 'heap_candidates'"
        ))
        .expect("pipeline heap candidate count query should succeed")
        .expect("pipeline heap candidate count should exist");
        let pipeline_live_heap_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT item_count {pipeline_steps_live_from} WHERE step_name = 'heap_candidates'"
        ))
        .expect("live pipeline heap candidate count query should succeed")
        .expect("live pipeline heap candidate count should exist");
        let manifest_result_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_result_from}"))
                .expect("manifest result status query should succeed")
                .expect("manifest result status should exist");
        let pipeline_manifest_status = Spi::get_one::<String>(&format!(
            "SELECT status {pipeline_steps_from} WHERE step_name = 'manifest_apply'"
        ))
        .expect("pipeline manifest status query should succeed")
        .expect("pipeline manifest status should exist");
        let pipeline_coordinator_status = Spi::get_one::<String>(&format!(
            "SELECT status {pipeline_steps_from} WHERE step_name = 'coordinator_result'"
        ))
        .expect("pipeline coordinator status query should succeed")
        .expect("pipeline coordinator status should exist");
        let pipeline_live_coordinator_status = Spi::get_one::<String>(&format!(
            "SELECT status {pipeline_steps_live_from} WHERE step_name = 'coordinator_result'"
        ))
        .expect("live pipeline coordinator status query should succeed")
        .expect("live pipeline coordinator status should exist");
        let pipeline_coordinator_count = Spi::get_one::<i64>(&format!(
            "SELECT item_count {pipeline_steps_from} WHERE step_name = 'coordinator_result'"
        ))
        .expect("pipeline coordinator count query should succeed")
        .expect("pipeline coordinator count should exist");
        let pipeline_live_coordinator_count = Spi::get_one::<i64>(&format!(
            "SELECT item_count {pipeline_steps_live_from} WHERE step_name = 'coordinator_result'"
        ))
        .expect("live pipeline coordinator count query should succeed")
        .expect("live pipeline coordinator count should exist");
        let identity_cache_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT dispatch_count {identity_cache_summary_from}"
        ))
        .expect("identity cache dispatch count query should succeed")
        .expect("identity cache dispatch count should exist");
        let identity_cache_compact_count = Spi::get_one::<i64>(&format!(
            "SELECT compact_candidate_count {identity_cache_summary_from}"
        ))
        .expect("identity cache compact count query should succeed")
        .expect("identity cache compact count should exist");
        let identity_cache_heap_count = Spi::get_one::<i64>(&format!(
            "SELECT heap_candidate_count {identity_cache_summary_from}"
        ))
        .expect("identity cache heap count query should succeed")
        .expect("identity cache heap count should exist");
        let identity_cache_entries = Spi::get_one::<i64>(&format!(
            "SELECT endpoint_identity_cache_entry_count {identity_cache_summary_from}"
        ))
        .expect("identity cache entry count query should succeed")
        .expect("identity cache entry count should exist");
        let identity_cache_queries = Spi::get_one::<i64>(&format!(
            "SELECT endpoint_identity_query_count {identity_cache_summary_from}"
        ))
        .expect("identity cache query count query should succeed")
        .expect("identity cache query count should exist");
        let identity_cache_hits = Spi::get_one::<i64>(&format!(
            "SELECT endpoint_identity_cache_hit_count {identity_cache_summary_from}"
        ))
        .expect("identity cache hit count query should succeed")
        .expect("identity cache hit count should exist");
        let identity_cache_misses = Spi::get_one::<i64>(&format!(
            "SELECT endpoint_identity_cache_miss_count {identity_cache_summary_from}"
        ))
        .expect("identity cache miss count query should succeed")
        .expect("identity cache miss count should exist");
        let identity_cache_raw_conninfo_cached = Spi::get_one::<bool>(&format!(
            "SELECT raw_conninfo_cached {identity_cache_summary_from}"
        ))
        .expect("identity cache raw conninfo query should succeed")
        .expect("identity cache raw conninfo flag should exist");
        let identity_cache_status =
            Spi::get_one::<String>(&format!("SELECT status {identity_cache_summary_from}"))
                .expect("identity cache status query should succeed")
                .expect("identity cache status should exist");
        let index_relation = open_valid_ec_spire_index_guard(
            index_oid,
            "test_ec_spire_libpq_identity_cache_contract_probe",
        );
        let (
            identity_cache_probe_entries,
            identity_cache_probe_queries,
            identity_cache_probe_hits,
            identity_cache_probe_misses,
            identity_cache_probe_mismatch_status,
        ) = unsafe {
            am::spire_remote_search_libpq_identity_cache_contract_probe_counts(
                index_relation.as_ptr(),
                u64::try_from(active_epoch).expect("active epoch should fit u64"),
                vec![1.0, 0.0],
                vec![u64::try_from(selected_pid).expect("selected PID should fit u64")],
                1,
                "strict",
            )
        };
        drop(index_relation);

        assert!(register_result);
        assert!(connection_attempted);
        assert_eq!(connection_status, "libpq_connection_opened");
        assert_eq!(conninfo_lookup_kind, "secret_provider");
        assert_eq!(candidate_count, 0);
        assert_eq!(nonempty_candidate_count, 1);
        assert_eq!(nonempty_candidate_node, 2);
        assert_eq!(nonempty_candidate_epoch, active_epoch);
        assert!(nonempty_candidate_locator_bytes > 0);
        assert_eq!(heap_candidate_count, 1);
        assert_eq!(heap_candidate_node, 2);
        assert!(heap_candidate_offset > 0);
        assert_eq!(heap_candidate_owner, "origin_node_row_locator");
        assert_eq!(heap_summary_count, 1);
        assert_eq!(heap_summary_source, "remote_heap_candidates");
        assert_eq!(heap_summary_status, "ready");
        assert_eq!(
            heap_summary_contract,
            "must_have_positive_returned_candidate_count_and_origin_node_heap_owner"
        );
        assert_eq!(coordinator_result_source, "remote_heap_candidates");
        assert_eq!(coordinator_returned_count, 1);
        assert_eq!(coordinator_final_heap_status, "remote_ready");
        assert_eq!(coordinator_next_blocker, "none");
        assert_eq!(coordinator_status, "ready");
        assert_eq!(mixed_coordinator_result_source, "remote_heap_candidates");
        assert_eq!(mixed_coordinator_returned_count, 2);
        assert_eq!(mixed_coordinator_decoded_count, 1);
        assert_eq!(mixed_coordinator_final_heap_status, "remote_ready");
        assert_eq!(mixed_coordinator_status, "ready");
        assert_eq!(
            pipeline_step_names,
            vec![
                "dispatch_plan",
                "connection_check",
                "candidates",
                "heap_candidates",
                "manifest_apply",
                "coordinator_result",
            ]
        );
        assert_eq!(pipeline_live_step_names, pipeline_step_names);
        assert_eq!(pipeline_dispatch_status, dispatch_summary_status);
        assert_eq!(pipeline_connection_status, "requires_libpq_executor");
        assert_eq!(
            pipeline_live_connection_status,
            nonempty_connection_terminal_status
        );
        assert_eq!(pipeline_candidate_count, 0);
        assert_eq!(pipeline_live_candidate_count, nonempty_candidate_count);
        assert_eq!(pipeline_heap_candidate_count, 0);
        assert_eq!(pipeline_live_heap_candidate_count, heap_summary_count);
        assert_eq!(pipeline_manifest_status, manifest_result_status);
        assert_eq!(pipeline_coordinator_status, "requires_libpq_executor");
        assert_eq!(pipeline_coordinator_count, 0);
        assert_eq!(pipeline_live_coordinator_status, coordinator_status);
        assert_eq!(pipeline_live_coordinator_count, coordinator_returned_count);
        assert_eq!(identity_cache_dispatch_count, 1);
        assert_eq!(identity_cache_compact_count, nonempty_candidate_count);
        assert_eq!(identity_cache_heap_count, heap_summary_count);
        assert_eq!(identity_cache_entries, 1);
        assert_eq!(identity_cache_queries, 1);
        assert_eq!(identity_cache_hits, 1);
        assert_eq!(identity_cache_misses, 1);
        assert!(!identity_cache_raw_conninfo_cached);
        assert_eq!(identity_cache_status, "ready");
        assert_eq!(identity_cache_probe_entries, 3);
        assert_eq!(identity_cache_probe_queries, 4);
        assert_eq!(identity_cache_probe_hits, 1);
        assert_eq!(identity_cache_probe_misses, 4);
        assert_eq!(
            identity_cache_probe_mismatch_status,
            "endpoint_identity_mismatch"
        );
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_spire remote search executor remote_index_identity does not match endpoint profile_fingerprint"
    )]
    fn test_ec_spire_libpq_rejects_identity_mismatch() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_IDENTITY_MISMATCH",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_remote_identity_mismatch_remote_sql; \
                 CREATE TABLE ec_spire_remote_identity_mismatch_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_remote_identity_mismatch_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_remote_identity_mismatch_remote_sql_idx \
                     ON ec_spire_remote_identity_mismatch_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq')",
            )
            .expect("loopback identity mismatch remote fixture should be created");

        Spi::run(
            "CREATE TABLE ec_spire_remote_identity_mismatch_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_identity_mismatch_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_identity_mismatch_coord_sql_idx \
             ON ec_spire_remote_identity_mismatch_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_identity_mismatch_coord_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_identity_mismatch_coord_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_identity_mismatch_coord_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 12, 'spire/remote/identity-mismatch', decode('ff', 'hex'), \
                     'ec_spire_remote_identity_mismatch_remote_sql_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        let receive_attempts_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_receive_attempts(\
                 'ec_spire_remote_identity_mismatch_coord_sql_idx'::regclass, \
                 {active_epoch}, ARRAY[1.0, 0.0]::real[], \
                 ARRAY[{selected_pid}]::bigint[], 1, 'strict')"
        );
        let receive_attempt_status =
            Spi::get_one::<String>(&format!("SELECT status {receive_attempts_from}"))
                .expect("receive attempt status query should succeed")
                .expect("receive attempt status should exist");
        let receive_attempt_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {receive_attempts_from}"))
                .expect("receive attempt blocker query should succeed")
                .expect("receive attempt blocker should exist");
        let receive_attempt_action =
            Spi::get_one::<String>(&format!("SELECT failure_action {receive_attempts_from}"))
                .expect("receive attempt action query should succeed")
                .expect("receive attempt action should exist");

        assert_eq!(receive_attempt_status, "endpoint_identity_mismatch");
        assert_eq!(receive_attempt_blocker, "remote_endpoint_identity");
        assert_eq!(receive_attempt_action, "fail_closed");

        Spi::run(&format!(
            "SELECT count(*) FROM ec_spire_remote_search_libpq_executor_candidates(\
                 'ec_spire_remote_identity_mismatch_coord_sql_idx'::regclass, \
                 {active_epoch}, ARRAY[1.0, 0.0]::real[], \
                 ARRAY[{selected_pid}]::bigint[], 1, 'strict')"
        ))
        .expect("remote identity mismatch should be rejected before merge");
    }

    #[pg_test]
    fn test_ec_spire_libpq_degraded_identity_mismatch_skips() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_DEGRADED_IDENTITY_MISMATCH",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_remote_degraded_identity_mismatch_remote_sql; \
                 CREATE TABLE ec_spire_remote_degraded_identity_mismatch_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_remote_degraded_identity_mismatch_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_remote_degraded_identity_mismatch_remote_sql_idx \
                     ON ec_spire_remote_degraded_identity_mismatch_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq')",
            )
            .expect("loopback degraded identity mismatch remote fixture should be created");
        let remote_index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_degraded_identity_mismatch_remote_sql_idx'::regclass::oid",
        )
        .expect("remote index oid query should succeed")
        .expect("remote index oid should exist");
        unsafe { am::debug_spire_rewrite_consistency_mode(remote_index_oid, "degraded") };

        Spi::run(
            "CREATE TABLE ec_spire_remote_degraded_identity_mismatch_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_degraded_identity_mismatch_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_degraded_identity_mismatch_coord_sql_idx \
             ON ec_spire_remote_degraded_identity_mismatch_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_degraded_identity_mismatch_coord_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_degraded_identity_mismatch_coord_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_degraded_identity_mismatch_coord_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2);
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 13, 'spire/remote/degraded-identity-mismatch', decode('ff', 'hex'), \
                     'ec_spire_remote_degraded_identity_mismatch_remote_sql_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        let args = format!(
            "'ec_spire_remote_degraded_identity_mismatch_coord_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'degraded'"
        );
        let receive_attempts_from =
            format!("FROM ec_spire_remote_search_libpq_executor_receive_attempts({args})");
        let identity_cache_from =
            format!("FROM ec_spire_remote_search_libpq_identity_cache_summary({args})");

        let receive_attempt_status =
            Spi::get_one::<String>(&format!("SELECT status {receive_attempts_from}"))
                .expect("receive attempt status query should succeed")
                .expect("receive attempt status should exist");
        let receive_attempt_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {receive_attempts_from}"))
                .expect("receive attempt blocker query should succeed")
                .expect("receive attempt blocker should exist");
        let receive_attempt_action =
            Spi::get_one::<String>(&format!("SELECT failure_action {receive_attempts_from}"))
                .expect("receive attempt action query should succeed")
                .expect("receive attempt action should exist");
        let receive_attempt_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT candidate_count {receive_attempts_from}"))
                .expect("receive attempt candidate count query should succeed")
                .expect("receive attempt candidate count should exist");
        let identity_cache_dispatch_count =
            Spi::get_one::<i64>(&format!("SELECT dispatch_count {identity_cache_from}"))
                .expect("identity cache dispatch count query should succeed")
                .expect("identity cache dispatch count should exist");
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
        let identity_cache_raw_conninfo_cached =
            Spi::get_one::<bool>(&format!("SELECT raw_conninfo_cached {identity_cache_from}"))
                .expect("identity cache raw conninfo query should succeed")
                .expect("identity cache raw conninfo flag should exist");
        let identity_cache_status =
            Spi::get_one::<String>(&format!("SELECT status {identity_cache_from}"))
                .expect("identity cache status query should succeed")
                .expect("identity cache status should exist");

        assert_eq!(receive_attempt_status, "endpoint_identity_mismatch");
        assert_eq!(receive_attempt_blocker, "remote_endpoint_identity");
        assert_eq!(receive_attempt_action, "skip_node");
        assert_eq!(receive_attempt_candidate_count, 0);
        assert_eq!(identity_cache_dispatch_count, 1);
        assert_eq!(identity_cache_compact_count, 0);
        assert_eq!(identity_cache_heap_count, 0);
        assert_eq!(identity_cache_entries, 0);
        assert_eq!(identity_cache_queries, 1);
        assert_eq!(identity_cache_hits, 0);
        assert_eq!(identity_cache_misses, 1);
        assert!(!identity_cache_raw_conninfo_cached);
        assert_eq!(identity_cache_status, "endpoint_identity_mismatch");
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_spire remote search executor endpoint_status requires_rabitq_storage_format is not ready"
    )]
    fn test_ec_spire_libpq_executor_rejects_non_ready_endpoint() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_NON_READY",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_remote_executor_non_ready_remote_sql; \
                 CREATE TABLE ec_spire_remote_executor_non_ready_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_remote_executor_non_ready_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_remote_executor_non_ready_remote_sql_idx \
                     ON ec_spire_remote_executor_non_ready_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2)",
            )
            .expect("loopback non-ready remote fixture should be created");

        Spi::run(
            "CREATE TABLE ec_spire_remote_executor_non_ready_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_executor_non_ready_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_executor_non_ready_coord_sql_idx \
             ON ec_spire_remote_executor_non_ready_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_executor_non_ready_coord_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_executor_non_ready_coord_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_executor_non_ready_coord_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 9, 'spire/remote/non-ready', decode('01', 'hex'), \
                     'ec_spire_remote_executor_non_ready_remote_sql_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        let receive_attempts_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_receive_attempts(\
                 'ec_spire_remote_executor_non_ready_coord_sql_idx'::regclass, \
                 {active_epoch}, ARRAY[1.0, 0.0]::real[], \
                 ARRAY[{selected_pid}]::bigint[], 1, 'strict')"
        );
        let receive_attempt_status =
            Spi::get_one::<String>(&format!("SELECT status {receive_attempts_from}"))
                .expect("receive attempt status query should succeed")
                .expect("receive attempt status should exist");
        let receive_attempt_action =
            Spi::get_one::<String>(&format!("SELECT failure_action {receive_attempts_from}"))
                .expect("receive attempt action query should succeed")
                .expect("receive attempt action should exist");
        let receive_attempt_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {receive_attempts_from}"))
                .expect("receive attempt blocker query should succeed")
                .expect("receive attempt blocker should exist");
        let receive_attempt_reason =
            Spi::get_one::<String>(&format!("SELECT failure_reason {receive_attempts_from}"))
                .expect("receive attempt reason query should succeed")
                .expect("receive attempt reason should exist");

        assert_eq!(receive_attempt_status, "requires_rabitq_storage_format");
        assert_eq!(receive_attempt_action, "fail_closed");
        assert_eq!(receive_attempt_blocker, "remote_endpoint_identity");
        assert_eq!(
            receive_attempt_reason,
            "ec_spire remote search executor endpoint_status requires_rabitq_storage_format is not ready"
        );

        Spi::run(&format!(
            "SELECT count(*) FROM ec_spire_remote_search_libpq_executor_candidates(\
                 'ec_spire_remote_executor_non_ready_coord_sql_idx'::regclass, \
                 {active_epoch}, ARRAY[1.0, 0.0]::real[], \
                 ARRAY[{selected_pid}]::bigint[], 1, 'strict')"
        ))
        .expect("non-ready remote endpoint should be rejected before merge");
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_spire remote search executor endpoint_status requires_rabitq_storage_format is not ready"
    )]
    fn test_ec_spire_heap_endpoint_rejects_non_ready() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_HEAP_NON_READY",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_remote_heap_non_ready_remote_sql; \
                 CREATE TABLE ec_spire_remote_heap_non_ready_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_remote_heap_non_ready_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_remote_heap_non_ready_remote_sql_idx \
                     ON ec_spire_remote_heap_non_ready_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2)",
            )
            .expect("loopback heap non-ready remote fixture should be created");

        Spi::run(
            "CREATE TABLE ec_spire_remote_heap_non_ready_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_heap_non_ready_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_heap_non_ready_coord_sql_idx \
             ON ec_spire_remote_heap_non_ready_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_heap_non_ready_coord_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_heap_non_ready_coord_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_heap_non_ready_coord_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 11, 'spire/remote/heap-non-ready', decode('01', 'hex'), \
                     'ec_spire_remote_heap_non_ready_remote_sql_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        Spi::run(&format!(
            "SELECT status FROM ec_spire_remote_search_coordinator_result_summary(\
                 'ec_spire_remote_heap_non_ready_coord_sql_idx'::regclass, \
                 {active_epoch}, ARRAY[1.0, 0.0]::real[], \
                 ARRAY[{selected_pid}]::bigint[], 1, 'strict')"
        ))
        .expect("non-ready remote heap endpoint should be rejected before final rows");
    }

    #[pg_test]
    fn test_ec_spire_libpq_receive_attempts_degraded_skip() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_DEGRADED_NON_READY",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_remote_executor_degraded_non_ready_remote_sql; \
                 CREATE TABLE ec_spire_remote_executor_degraded_non_ready_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_remote_executor_degraded_non_ready_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_remote_executor_degraded_non_ready_remote_sql_idx \
                     ON ec_spire_remote_executor_degraded_non_ready_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2)",
            )
            .expect("loopback degraded non-ready remote fixture should be created");
        let remote_index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_executor_degraded_non_ready_remote_sql_idx'::regclass::oid",
        )
        .expect("remote index oid query should succeed")
        .expect("remote index oid should exist");
        unsafe { am::debug_spire_rewrite_consistency_mode(remote_index_oid, "degraded") };

        Spi::run(
            "CREATE TABLE ec_spire_remote_executor_degraded_non_ready_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_executor_degraded_non_ready_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_executor_degraded_non_ready_coord_sql_idx \
             ON ec_spire_remote_executor_degraded_non_ready_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_executor_degraded_non_ready_coord_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_executor_degraded_non_ready_coord_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_executor_degraded_non_ready_coord_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2);
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 10, 'spire/remote/degraded-non-ready', decode('01', 'hex'), \
                     'ec_spire_remote_executor_degraded_non_ready_remote_sql_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        assert!(register_result);

        let receive_attempts_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_receive_attempts(\
                 'ec_spire_remote_executor_degraded_non_ready_coord_sql_idx'::regclass, \
                 {active_epoch}, ARRAY[1.0, 0.0]::real[], \
                 ARRAY[{selected_pid}]::bigint[], 1, 'degraded')"
        );
        let receive_attempt_status =
            Spi::get_one::<String>(&format!("SELECT status {receive_attempts_from}"))
                .expect("receive attempt status query should succeed")
                .expect("receive attempt status should exist");
        let receive_attempt_action =
            Spi::get_one::<String>(&format!("SELECT failure_action {receive_attempts_from}"))
                .expect("receive attempt action query should succeed")
                .expect("receive attempt action should exist");
        let receive_attempt_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT candidate_count {receive_attempts_from}"))
                .expect("receive attempt candidate count query should succeed")
                .expect("receive attempt candidate count should exist");
        let receive_attempt_reason =
            Spi::get_one::<String>(&format!("SELECT failure_reason {receive_attempts_from}"))
                .expect("receive attempt reason query should succeed")
                .expect("receive attempt reason should exist");

        assert_eq!(receive_attempt_status, "requires_rabitq_storage_format");
        assert_eq!(receive_attempt_action, "skip_node");
        assert_eq!(receive_attempt_candidate_count, 0);
        assert_eq!(
            receive_attempt_reason,
            "ec_spire remote search executor endpoint_status requires_rabitq_storage_format is not ready"
        );
    }
