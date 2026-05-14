    #[pg_test]
    fn test_ec_spire_remote_search_sql_scores_selected_leaf_pids() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_search_sql (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_search_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_search_sql_idx \
             ON ec_spire_remote_search_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_search_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_search_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let remote_search_from = format!(
            "FROM ec_spire_remote_search(\
             'ec_spire_remote_search_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 1, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {remote_search_from}"))
            .expect("remote search count query should succeed")
            .expect("remote search count should exist");
        let served_epoch_matches = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(served_epoch = {active_epoch}) {remote_search_from}"
        ))
        .expect("remote search epoch query should succeed")
        .expect("served epoch aggregate should exist");
        let local_node = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(node_id = 0) {remote_search_from}"
        ))
        .expect("remote search node query should succeed")
        .expect("node aggregate should exist");
        let selected_pid = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(pid = ANY(ARRAY[{}, {}]::bigint[])) {remote_search_from}",
            selected_pids[0], selected_pids[1]
        ))
        .expect("remote search pid query should succeed")
        .expect("pid aggregate should exist");
        let has_vec_id = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(length(vec_id) > 0) {remote_search_from}"
        ))
        .expect("remote search vec_id query should succeed")
        .expect("vec_id aggregate should exist");
        let row_locator_len = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(length(row_locator) = 6) {remote_search_from}"
        ))
        .expect("remote search locator query should succeed")
        .expect("row locator aggregate should exist");
        let protocol_matches = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(protocol_version = 'ec_spire_remote_search_v1') {remote_search_from}"
        ))
        .expect("remote search protocol query should succeed")
        .expect("protocol aggregate should exist");
        let extension_matches = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(extension_version = '{}') {remote_search_from}",
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote search extension query should succeed")
        .expect("extension aggregate should exist");
        let endpoint_status =
            Spi::get_one::<String>(&format!("SELECT min(endpoint_status) {remote_search_from}"))
                .expect("remote search endpoint status query should succeed")
                .expect("endpoint status should exist");
        let fingerprint_len = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(length(profile_fingerprint) = 16) {remote_search_from}"
        ))
        .expect("remote search fingerprint query should succeed")
        .expect("fingerprint aggregate should exist");

        assert_eq!(row_count, 1);
        assert!(served_epoch_matches);
        assert!(local_node);
        assert!(selected_pid);
        assert!(has_vec_id);
        assert!(row_locator_len);
        assert!(protocol_matches);
        assert!(extension_matches);
        assert_eq!(endpoint_status, "requires_rabitq_storage_format");
        assert!(fingerprint_len);
    }
    #[pg_test]
    fn test_ec_spire_remote_search_coord_local_matches_storage() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_coord_local_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_coord_local_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_coord_local_sql_idx \
             ON ec_spire_remote_coord_local_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_coord_local_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_coord_local_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let args = format!(
            "'ec_spire_remote_coord_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 1, 'strict'",
            selected_pids[0], selected_pids[1],
        );
        let coordinator_from = format!("FROM ec_spire_remote_search_coordinator_local({args})");
        let storage_from = format!("FROM ec_spire_remote_search({args})");
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {coordinator_from}"))
            .expect("coordinator count query should succeed")
            .expect("coordinator count should exist");
        let local_node =
            Spi::get_one::<bool>(&format!("SELECT bool_and(node_id = 0) {coordinator_from}"))
                .expect("coordinator node query should succeed")
                .expect("node aggregate should exist");
        let selected_pid = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(pid = ANY(ARRAY[{}, {}]::bigint[])) {coordinator_from}",
            selected_pids[0], selected_pids[1]
        ))
        .expect("coordinator pid query should succeed")
        .expect("pid aggregate should exist");
        let matches_storage = Spi::get_one::<bool>(&format!(
            "WITH coordinator AS (SELECT * {coordinator_from}), \
                  storage AS (SELECT * {storage_from}) \
             SELECT bool_and(\
                coordinator.served_epoch = storage.served_epoch AND \
                coordinator.node_id = storage.node_id AND \
                coordinator.pid = storage.pid AND \
                coordinator.object_version = storage.object_version AND \
                coordinator.row_index = storage.row_index AND \
                coordinator.assignment_flags = storage.assignment_flags AND \
                coordinator.vec_id = storage.vec_id AND \
                coordinator.row_locator = storage.row_locator AND \
                coordinator.score = storage.score) \
             FROM coordinator JOIN storage USING (vec_id)"
        ))
        .expect("coordinator/storage comparison should succeed")
        .expect("comparison aggregate should exist");

        assert_eq!(row_count, 1);
        assert!(local_node);
        assert!(selected_pid);
        assert!(matches_storage);
    }

    #[pg_test]
    #[should_panic(expected = "requires libpq transport for 1 remote target")]
    fn test_ec_spire_remote_search_coord_local_rejects_remote_target() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_coord_remote_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_coord_remote_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_coord_remote_sql_idx \
             ON ec_spire_remote_coord_remote_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_coord_remote_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_coord_remote_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_coord_remote_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        Spi::run(&format!(
            "SELECT count(*) FROM ec_spire_remote_search_coordinator_local(\
             'ec_spire_remote_coord_remote_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'strict')",
        ))
        .expect("coordinator-local search with remote target should fail before transport");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_coord_summary_counts() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_coord_summary_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_coord_summary_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_coord_summary_sql_idx \
             ON ec_spire_remote_coord_summary_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_coord_summary_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_coord_summary_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_coord_summary_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let summary_from = format!(
            "FROM ec_spire_remote_search_coordinator_local_summary(\
             'ec_spire_remote_coord_summary_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 1, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("summary status query should succeed")
            .expect("summary status should exist");
        let local_pid_count =
            Spi::get_one::<i64>(&format!("SELECT local_pid_count {summary_from}"))
                .expect("summary local pid count query should succeed")
                .expect("local pid count should exist");
        let candidate_input_count =
            Spi::get_one::<i64>(&format!("SELECT candidate_input_count {summary_from}"))
                .expect("summary candidate input count query should succeed")
                .expect("candidate input count should exist");
        let returned_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT returned_candidate_count {summary_from}"))
                .expect("summary returned candidate count query should succeed")
                .expect("returned candidate count should exist");

        assert_eq!(status, "ready");
        assert_eq!(local_pid_count, 2);
        assert_eq!(candidate_input_count, 1);
        assert_eq!(returned_candidate_count, 1);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[0] as u64, 2) };
        let remote_summary_from = format!(
            "FROM ec_spire_remote_search_coordinator_local_summary(\
             'ec_spire_remote_coord_summary_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}]::bigint[], 1, 'strict')",
            selected_pids[0],
        );
        let remote_status = Spi::get_one::<String>(&format!("SELECT status {remote_summary_from}"))
            .expect("remote summary status query should succeed")
            .expect("remote summary status should exist");
        let remote_target_count =
            Spi::get_one::<i64>(&format!("SELECT remote_target_count {remote_summary_from}"))
                .expect("remote target count query should succeed")
                .expect("remote target count should exist");
        let remote_pid_count =
            Spi::get_one::<i64>(&format!("SELECT remote_pid_count {remote_summary_from}"))
                .expect("remote pid count query should succeed")
                .expect("remote pid count should exist");
        let remote_candidate_input_count = Spi::get_one::<i64>(&format!(
            "SELECT candidate_input_count {remote_summary_from}"
        ))
        .expect("remote candidate input count query should succeed")
        .expect("remote candidate input count should exist");

        assert_eq!(remote_status, "requires_libpq_transport");
        assert_eq!(remote_target_count, 1);
        assert_eq!(remote_pid_count, 1);
        assert_eq!(remote_candidate_input_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_coord_summary_degraded_skips() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_coord_degraded_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_coord_degraded_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_coord_degraded_sql_idx \
             ON ec_spire_remote_coord_degraded_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_coord_degraded_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_coord_degraded_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_coord_degraded_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pid as u64, "unavailable");
        }

        let summary_from = format!(
            "FROM ec_spire_remote_search_coordinator_local_summary(\
             'ec_spire_remote_coord_degraded_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'degraded')",
        );
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("degraded summary status query should succeed")
            .expect("degraded summary status should exist");
        let skipped_placement_count =
            Spi::get_one::<i64>(&format!("SELECT skipped_placement_count {summary_from}"))
                .expect("degraded skipped placement count query should succeed")
                .expect("degraded skipped placement count should exist");
        let candidate_input_count =
            Spi::get_one::<i64>(&format!("SELECT candidate_input_count {summary_from}"))
                .expect("degraded candidate input count query should succeed")
                .expect("degraded candidate input count should exist");
        let returned_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT returned_candidate_count {summary_from}"))
                .expect("degraded returned candidate count query should succeed")
                .expect("degraded returned candidate count should exist");
        let coordinator_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM ec_spire_remote_search_coordinator_local(\
             'ec_spire_remote_coord_degraded_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'degraded')",
        ))
        .expect("degraded coordinator query should succeed")
        .expect("degraded coordinator count should exist");

        assert_eq!(status, "degraded_ready");
        assert_eq!(skipped_placement_count, 1);
        assert_eq!(candidate_input_count, 0);
        assert_eq!(returned_candidate_count, 0);
        assert_eq!(coordinator_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_fanout_plan_sql_reports_local_pids() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_fanout_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_fanout_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_fanout_sql_idx \
             ON ec_spire_remote_fanout_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_fanout_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_fanout_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let fanout_from = format!(
            "FROM ec_spire_remote_search_fanout_plan(\
             'ec_spire_remote_fanout_sql_idx'::regclass, \
             {active_epoch}, ARRAY[{}, {}]::bigint[], 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {fanout_from}"))
            .expect("fanout count query should succeed")
            .expect("fanout count should exist");
        let all_local = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(target_kind = 'local') {fanout_from}"
        ))
        .expect("fanout target query should succeed")
        .expect("fanout target aggregate should exist");
        let all_available = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(placement_state = 'available') {fanout_from}"
        ))
        .expect("fanout state query should succeed")
        .expect("fanout state aggregate should exist");
        let selected_match = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(pid = ANY(ARRAY[{}, {}]::bigint[])) {fanout_from}",
            selected_pids[0], selected_pids[1]
        ))
        .expect("fanout pid query should succeed")
        .expect("fanout pid aggregate should exist");

        assert_eq!(row_count, 2);
        assert!(all_local);
        assert!(all_available);
        assert!(selected_match);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_target_plan_groups_targets() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_target_plan_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_target_plan_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_target_plan_sql_idx \
             ON ec_spire_remote_target_plan_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_target_plan_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_target_plan_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_target_plan_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let target_from = format!(
            "FROM ec_spire_remote_search_target_plan(\
             'ec_spire_remote_target_plan_sql_idx'::regclass, \
             {active_epoch}, ARRAY[{}, {}]::bigint[], 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {target_from}"))
            .expect("target plan count query should succeed")
            .expect("target plan count should exist");
        let local_pid_count = Spi::get_one::<i64>(&format!(
            "SELECT pid_count {target_from} WHERE target_kind = 'local'"
        ))
        .expect("local target query should succeed")
        .expect("local target should exist");
        let remote_status = Spi::get_one::<String>(&format!(
            "SELECT status {target_from} WHERE target_kind = 'remote'"
        ))
        .expect("remote target status query should succeed")
        .expect("remote target status should exist");
        let remote_pids = Spi::get_one::<Vec<i64>>(&format!(
            "SELECT selected_pids {target_from} WHERE target_kind = 'remote'"
        ))
        .expect("remote target pids query should succeed")
        .expect("remote target pids should exist");

        assert_eq!(row_count, 2);
        assert_eq!(local_pid_count, 1);
        assert_eq!(remote_status, "requires_libpq_transport");
        assert_eq!(remote_pids, vec![selected_pids[1]]);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_target_plan_groups_degraded_skips() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_target_skip_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_target_skip_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_target_skip_sql_idx \
             ON ec_spire_remote_target_skip_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_target_skip_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_target_skip_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_target_skip_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pid as u64, "unavailable");
        }
        let target_from = format!(
            "FROM ec_spire_remote_search_target_plan(\
             'ec_spire_remote_target_skip_sql_idx'::regclass, \
             {active_epoch}, ARRAY[{selected_pid}]::bigint[], 'degraded')",
        );
        let target_kind = Spi::get_one::<String>(&format!("SELECT target_kind {target_from}"))
            .expect("degraded target kind query should succeed")
            .expect("degraded target kind should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {target_from}"))
            .expect("degraded target status query should succeed")
            .expect("degraded target status should exist");
        let placement_state =
            Spi::get_one::<String>(&format!("SELECT placement_state {target_from}"))
                .expect("degraded target state query should succeed")
                .expect("degraded target state should exist");
        let selected_pids =
            Spi::get_one::<Vec<i64>>(&format!("SELECT selected_pids {target_from}"))
                .expect("degraded target pids query should succeed")
                .expect("degraded target pids should exist");

        assert_eq!(target_kind, "skipped");
        assert_eq!(status, "degraded_skipped");
        assert_eq!(placement_state, "unavailable");
        assert_eq!(selected_pids, vec![selected_pid]);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_target_readiness_remote() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_target_ready_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_target_ready_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_target_ready_sql_idx \
             ON ec_spire_remote_target_ready_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_target_ready_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_target_ready_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_target_ready_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let readiness_from = format!(
            "FROM ec_spire_remote_search_target_readiness(\
             'ec_spire_remote_target_ready_sql_idx'::regclass, \
             {active_epoch}, ARRAY[{}, {}]::bigint[], 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let remote_status = Spi::get_one::<String>(&format!(
            "SELECT status {readiness_from} WHERE target_kind = 'remote'"
        ))
        .expect("remote target readiness status query should succeed")
        .expect("remote target readiness status should exist");
        let remote_node_status = Spi::get_one::<String>(&format!(
            "SELECT node_status {readiness_from} WHERE target_kind = 'remote'"
        ))
        .expect("remote target readiness node status query should succeed")
        .expect("remote target readiness node status should exist");
        let remote_descriptor_state = Spi::get_one::<String>(&format!(
            "SELECT descriptor_state {readiness_from} WHERE target_kind = 'remote'"
        ))
        .expect("remote target readiness descriptor query should succeed")
        .expect("remote target readiness descriptor should exist");
        let local_status = Spi::get_one::<String>(&format!(
            "SELECT status {readiness_from} WHERE target_kind = 'local'"
        ))
        .expect("local target readiness status query should succeed")
        .expect("local target readiness status should exist");

        assert_eq!(remote_status, "requires_remote_node_descriptor");
        assert_eq!(remote_node_status, "requires_remote_node_descriptor");
        assert_eq!(remote_descriptor_state, "missing");
        assert_eq!(local_status, "ready");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_target_readiness_degraded() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_target_ready_skip_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_target_ready_skip_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_target_ready_skip_sql_idx \
             ON ec_spire_remote_target_ready_skip_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_target_ready_skip_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_target_ready_skip_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_target_ready_skip_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pid as u64, "skipped");
        }
        let readiness_from = format!(
            "FROM ec_spire_remote_search_target_readiness(\
             'ec_spire_remote_target_ready_skip_sql_idx'::regclass, \
             {active_epoch}, ARRAY[{selected_pid}]::bigint[], 'degraded')",
        );
        let target_kind = Spi::get_one::<String>(&format!("SELECT target_kind {readiness_from}"))
            .expect("degraded readiness target kind query should succeed")
            .expect("degraded readiness target kind should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {readiness_from}"))
            .expect("degraded readiness status query should succeed")
            .expect("degraded readiness status should exist");
        let node_status = Spi::get_one::<String>(&format!("SELECT node_status {readiness_from}"))
            .expect("degraded readiness node status query should succeed")
            .expect("degraded readiness node status should exist");
        let placement_state =
            Spi::get_one::<String>(&format!("SELECT placement_state {readiness_from}"))
                .expect("degraded readiness placement state query should succeed")
                .expect("degraded readiness placement state should exist");

        assert_eq!(target_kind, "skipped");
        assert_eq!(status, "degraded_skipped");
        assert_eq!(node_status, "ready");
        assert_eq!(placement_state, "skipped");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_target_readiness_mixed_precedence() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_target_ready_mixed_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_target_ready_mixed_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_target_ready_mixed_sql_idx \
             ON ec_spire_remote_target_ready_mixed_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 3)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_target_ready_mixed_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_target_ready_mixed_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_target_ready_mixed_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 3);

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pids[2] as u64, "skipped");
            am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2);
        }
        let readiness_from = format!(
            "FROM ec_spire_remote_search_target_readiness(\
             'ec_spire_remote_target_ready_mixed_sql_idx'::regclass, \
             {active_epoch}, ARRAY[{}, {}, {}]::bigint[], 'degraded')",
            selected_pids[0], selected_pids[1], selected_pids[2],
        );
        let summary_from = format!(
            "FROM ec_spire_remote_search_readiness_summary(\
             'ec_spire_remote_target_ready_mixed_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}, {}]::bigint[], 3, 'degraded')",
            selected_pids[0], selected_pids[1], selected_pids[2],
        );
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {readiness_from}"))
            .expect("mixed readiness count query should succeed")
            .expect("mixed readiness count should exist");
        let local_status = Spi::get_one::<String>(&format!(
            "SELECT status {readiness_from} WHERE target_kind = 'local'"
        ))
        .expect("mixed local readiness query should succeed")
        .expect("mixed local readiness should exist");
        let remote_status = Spi::get_one::<String>(&format!(
            "SELECT status {readiness_from} WHERE target_kind = 'remote'"
        ))
        .expect("mixed remote readiness query should succeed")
        .expect("mixed remote readiness should exist");
        let skipped_status = Spi::get_one::<String>(&format!(
            "SELECT status {readiness_from} WHERE target_kind = 'skipped'"
        ))
        .expect("mixed skipped readiness query should succeed")
        .expect("mixed skipped readiness should exist");
        let skipped_node_status = Spi::get_one::<String>(&format!(
            "SELECT node_status {readiness_from} WHERE target_kind = 'skipped'"
        ))
        .expect("mixed skipped node readiness query should succeed")
        .expect("mixed skipped node readiness should exist");
        let summary_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("mixed readiness summary status query should succeed")
            .expect("mixed readiness summary status should exist");
        let ready_request_count =
            Spi::get_one::<i64>(&format!("SELECT ready_request_count {summary_from}"))
                .expect("mixed readiness summary ready count query should succeed")
                .expect("mixed readiness summary ready count should exist");
        let blocked_request_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_request_count {summary_from}"))
                .expect("mixed readiness summary blocked count query should succeed")
                .expect("mixed readiness summary blocked count should exist");
        let skipped_request_count =
            Spi::get_one::<i64>(&format!("SELECT skipped_request_count {summary_from}"))
                .expect("mixed readiness summary skipped count query should succeed")
                .expect("mixed readiness summary skipped count should exist");
        let missing_descriptor_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_request_count {summary_from}"
        ))
        .expect("mixed readiness summary descriptor count query should succeed")
        .expect("mixed readiness summary descriptor count should exist");

        assert_eq!(row_count, 3);
        assert_eq!(local_status, "ready");
        assert_eq!(remote_status, "requires_remote_node_descriptor");
        assert_eq!(skipped_status, "degraded_skipped");
        assert_eq!(skipped_node_status, "ready");
        assert_eq!(summary_status, "requires_remote_node_descriptor");
        assert_eq!(ready_request_count, 1);
        assert_eq!(blocked_request_count, 1);
        assert_eq!(skipped_request_count, 1);
        assert_eq!(missing_descriptor_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_request_plan_contract() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_request_plan_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_request_plan_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_request_plan_sql_idx \
             ON ec_spire_remote_request_plan_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_request_plan_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_request_plan_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_request_plan_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let request_from = format!(
            "FROM ec_spire_remote_search_request_plan(\
             'ec_spire_remote_request_plan_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {request_from}"))
            .expect("request plan count query should succeed")
            .expect("request plan count should exist");
        let query_dimension =
            Spi::get_one::<i64>(&format!("SELECT min(query_dimension) {request_from}"))
                .expect("request plan dimension query should succeed")
                .expect("request plan dimension should exist");
        let top_k = Spi::get_one::<i64>(&format!("SELECT min(top_k) {request_from}"))
            .expect("request plan top_k query should succeed")
            .expect("request plan top_k should exist");
        let endpoint_ok = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(endpoint_function = 'ec_spire_remote_search') {request_from}"
        ))
        .expect("request plan endpoint query should succeed")
        .expect("request plan endpoint aggregate should exist");
        let remote_status = Spi::get_one::<String>(&format!(
            "SELECT status {request_from} WHERE target_kind = 'remote'"
        ))
        .expect("request plan remote status query should succeed")
        .expect("request plan remote status should exist");

        assert_eq!(row_count, 2);
        assert_eq!(query_dimension, 2);
        assert_eq!(top_k, 3);
        assert!(endpoint_ok);
        assert_eq!(remote_status, "requires_libpq_transport");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_request_plan_degraded_skip() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_request_skip_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_request_skip_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_request_skip_sql_idx \
             ON ec_spire_remote_request_skip_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_request_skip_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_request_skip_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_request_skip_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pid as u64, "skipped");
        }
        let request_from = format!(
            "FROM ec_spire_remote_search_request_plan(\
             'ec_spire_remote_request_skip_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'degraded')",
        );
        let target_kind = Spi::get_one::<String>(&format!("SELECT target_kind {request_from}"))
            .expect("request skip target kind query should succeed")
            .expect("request skip target kind should exist");
        let endpoint = Spi::get_one::<String>(&format!("SELECT endpoint_function {request_from}"))
            .expect("request skip endpoint query should succeed")
            .expect("request skip endpoint should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {request_from}"))
            .expect("request skip status query should succeed")
            .expect("request skip status should exist");
        let consistency_mode =
            Spi::get_one::<String>(&format!("SELECT consistency_mode {request_from}"))
                .expect("request skip mode query should succeed")
                .expect("request skip mode should exist");

        assert_eq!(target_kind, "skipped");
        assert_eq!(endpoint, "none");
        assert_eq!(status, "degraded_skipped");
        assert_eq!(consistency_mode, "degraded");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_req_readiness_remote() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_request_ready_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_request_ready_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_request_ready_sql_idx \
             ON ec_spire_remote_request_ready_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_request_ready_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_request_ready_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_request_ready_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let readiness_from = format!(
            "FROM ec_spire_remote_search_request_readiness(\
             'ec_spire_remote_request_ready_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let remote_status = Spi::get_one::<String>(&format!(
            "SELECT status {readiness_from} WHERE target_kind = 'remote'"
        ))
        .expect("request readiness remote status query should succeed")
        .expect("request readiness remote status should exist");
        let remote_endpoint = Spi::get_one::<String>(&format!(
            "SELECT endpoint_function {readiness_from} WHERE target_kind = 'remote'"
        ))
        .expect("request readiness endpoint query should succeed")
        .expect("request readiness endpoint should exist");
        let remote_node_status = Spi::get_one::<String>(&format!(
            "SELECT node_status {readiness_from} WHERE target_kind = 'remote'"
        ))
        .expect("request readiness node status query should succeed")
        .expect("request readiness node status should exist");
        let query_dimension = Spi::get_one::<i64>(&format!(
            "SELECT query_dimension {readiness_from} WHERE target_kind = 'remote'"
        ))
        .expect("request readiness query dimension query should succeed")
        .expect("request readiness query dimension should exist");
        let top_k = Spi::get_one::<i64>(&format!(
            "SELECT top_k {readiness_from} WHERE target_kind = 'remote'"
        ))
        .expect("request readiness top_k query should succeed")
        .expect("request readiness top_k should exist");

        assert_eq!(remote_status, "requires_remote_node_descriptor");
        assert_eq!(remote_endpoint, "ec_spire_remote_search");
        assert_eq!(remote_node_status, "requires_remote_node_descriptor");
        assert_eq!(query_dimension, 2);
        assert_eq!(top_k, 3);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_req_readiness_degraded() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_request_ready_skip_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_request_ready_skip_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_request_ready_skip_sql_idx \
             ON ec_spire_remote_request_ready_skip_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_request_ready_skip_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_request_ready_skip_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_request_ready_skip_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pid as u64, "unavailable");
        }
        let readiness_from = format!(
            "FROM ec_spire_remote_search_request_readiness(\
             'ec_spire_remote_request_ready_skip_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'degraded')",
        );
        let target_kind = Spi::get_one::<String>(&format!("SELECT target_kind {readiness_from}"))
            .expect("request readiness degraded target query should succeed")
            .expect("request readiness degraded target should exist");
        let endpoint =
            Spi::get_one::<String>(&format!("SELECT endpoint_function {readiness_from}"))
                .expect("request readiness degraded endpoint query should succeed")
                .expect("request readiness degraded endpoint should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {readiness_from}"))
            .expect("request readiness degraded status query should succeed")
            .expect("request readiness degraded status should exist");
        let node_status = Spi::get_one::<String>(&format!("SELECT node_status {readiness_from}"))
            .expect("request readiness degraded node status query should succeed")
            .expect("request readiness degraded node status should exist");

        assert_eq!(target_kind, "skipped");
        assert_eq!(endpoint, "none");
        assert_eq!(status, "degraded_skipped");
        assert_eq!(node_status, "ready");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_req_summary_counts() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_request_summary_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_request_summary_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_request_summary_sql_idx \
             ON ec_spire_remote_request_summary_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_request_summary_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_request_summary_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_request_summary_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let summary_from = format!(
            "FROM ec_spire_remote_search_request_summary(\
             'ec_spire_remote_request_summary_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("request summary status query should succeed")
            .expect("request summary status should exist");
        let request_count = Spi::get_one::<i64>(&format!("SELECT request_count {summary_from}"))
            .expect("request summary count query should succeed")
            .expect("request count should exist");
        let local_request_count =
            Spi::get_one::<i64>(&format!("SELECT local_request_count {summary_from}"))
                .expect("local request count query should succeed")
                .expect("local request count should exist");
        let remote_request_count =
            Spi::get_one::<i64>(&format!("SELECT remote_request_count {summary_from}"))
                .expect("remote request count query should succeed")
                .expect("remote request count should exist");
        let executable_pid_count =
            Spi::get_one::<i64>(&format!("SELECT executable_pid_count {summary_from}"))
                .expect("executable pid count query should succeed")
                .expect("executable pid count should exist");
        let query_dimension =
            Spi::get_one::<i64>(&format!("SELECT query_dimension {summary_from}"))
                .expect("query dimension query should succeed")
                .expect("query dimension should exist");
        let top_k = Spi::get_one::<i64>(&format!("SELECT top_k {summary_from}"))
            .expect("top_k query should succeed")
            .expect("top_k should exist");

        assert_eq!(status, "requires_libpq_transport");
        assert_eq!(request_count, 2);
        assert_eq!(local_request_count, 1);
        assert_eq!(remote_request_count, 1);
        assert_eq!(executable_pid_count, 2);
        assert_eq!(query_dimension, 2);
        assert_eq!(top_k, 3);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_req_summary_degraded() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_request_summary_skip_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_request_summary_skip_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_request_summary_skip_sql_idx \
             ON ec_spire_remote_request_summary_skip_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_request_summary_skip_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_request_summary_skip_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_request_summary_skip_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pid as u64, "unavailable");
        }
        let summary_from = format!(
            "FROM ec_spire_remote_search_request_summary(\
             'ec_spire_remote_request_summary_skip_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'degraded')",
        );
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("request summary skip status query should succeed")
            .expect("request summary skip status should exist");
        let skipped_request_count =
            Spi::get_one::<i64>(&format!("SELECT skipped_request_count {summary_from}"))
                .expect("skipped request count query should succeed")
                .expect("skipped request count should exist");
        let skipped_pid_count =
            Spi::get_one::<i64>(&format!("SELECT skipped_pid_count {summary_from}"))
                .expect("skipped pid count query should succeed")
                .expect("skipped pid count should exist");
        let executable_pid_count =
            Spi::get_one::<i64>(&format!("SELECT executable_pid_count {summary_from}"))
                .expect("executable pid count query should succeed")
                .expect("executable pid count should exist");
        let consistency_mode =
            Spi::get_one::<String>(&format!("SELECT consistency_mode {summary_from}"))
                .expect("consistency mode query should succeed")
                .expect("consistency mode should exist");

        assert_eq!(status, "degraded_ready");
        assert_eq!(skipped_request_count, 1);
        assert_eq!(skipped_pid_count, 1);
        assert_eq!(executable_pid_count, 0);
        assert_eq!(consistency_mode, "degraded");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_ready_summary_blocked() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_ready_summary_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_ready_summary_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_ready_summary_sql_idx \
             ON ec_spire_remote_ready_summary_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_ready_summary_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_ready_summary_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_ready_summary_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let summary_from = format!(
            "FROM ec_spire_remote_search_readiness_summary(\
             'ec_spire_remote_ready_summary_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("readiness summary status query should succeed")
            .expect("readiness summary status should exist");
        let ready_request_count =
            Spi::get_one::<i64>(&format!("SELECT ready_request_count {summary_from}"))
                .expect("ready request count query should succeed")
                .expect("ready request count should exist");
        let blocked_request_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_request_count {summary_from}"))
                .expect("blocked request count query should succeed")
                .expect("blocked request count should exist");
        let missing_descriptor_request_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_request_count {summary_from}"
        ))
        .expect("missing descriptor request count query should succeed")
        .expect("missing descriptor request count should exist");
        let blocked_pid_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_pid_count {summary_from}"))
                .expect("blocked pid count query should succeed")
                .expect("blocked pid count should exist");
        let query_dimension =
            Spi::get_one::<i64>(&format!("SELECT query_dimension {summary_from}"))
                .expect("query dimension query should succeed")
                .expect("query dimension should exist");

        assert_eq!(status, "requires_remote_node_descriptor");
        assert_eq!(ready_request_count, 1);
        assert_eq!(blocked_request_count, 1);
        assert_eq!(missing_descriptor_request_count, 1);
        assert_eq!(blocked_pid_count, 1);
        assert_eq!(query_dimension, 2);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_ready_summary_degraded() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_ready_summary_skip_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_ready_summary_skip_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_ready_summary_skip_sql_idx \
             ON ec_spire_remote_ready_summary_skip_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_ready_summary_skip_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_ready_summary_skip_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_ready_summary_skip_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pid as u64, "skipped");
        }
        let summary_from = format!(
            "FROM ec_spire_remote_search_readiness_summary(\
             'ec_spire_remote_ready_summary_skip_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'degraded')",
        );
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("readiness degraded summary status query should succeed")
            .expect("readiness degraded summary status should exist");
        let skipped_request_count =
            Spi::get_one::<i64>(&format!("SELECT skipped_request_count {summary_from}"))
                .expect("skipped request count query should succeed")
                .expect("skipped request count should exist");
        let blocked_request_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_request_count {summary_from}"))
                .expect("blocked request count query should succeed")
                .expect("blocked request count should exist");
        let skipped_pid_count =
            Spi::get_one::<i64>(&format!("SELECT skipped_pid_count {summary_from}"))
                .expect("skipped pid count query should succeed")
                .expect("skipped pid count should exist");
        let missing_descriptor_request_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_request_count {summary_from}"
        ))
        .expect("missing descriptor request count query should succeed")
        .expect("missing descriptor request count should exist");

        assert_eq!(status, "degraded_ready");
        assert_eq!(skipped_request_count, 1);
        assert_eq!(blocked_request_count, 0);
        assert_eq!(skipped_pid_count, 1);
        assert_eq!(missing_descriptor_request_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_exec_plan_blocked() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_exec_plan_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_exec_plan_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_exec_plan_sql_idx \
             ON ec_spire_remote_exec_plan_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_exec_plan_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_exec_plan_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_exec_plan_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let plan_from = format!(
            "FROM ec_spire_remote_search_execution_plan(\
             'ec_spire_remote_exec_plan_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let summary_from = format!(
            "FROM ec_spire_remote_search_execution_summary(\
             'ec_spire_remote_exec_plan_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let remote_transport = Spi::get_one::<String>(&format!(
            "SELECT execution_transport {plan_from} WHERE target_kind = 'remote'"
        ))
        .expect("remote execution transport query should succeed")
        .expect("remote execution transport should exist");
        let remote_index_source = Spi::get_one::<String>(&format!(
            "SELECT remote_index_source {plan_from} WHERE target_kind = 'remote'"
        ))
        .expect("remote execution index source query should succeed")
        .expect("remote execution index source should exist");
        let remote_conninfo_source = Spi::get_one::<String>(&format!(
            "SELECT conninfo_source {plan_from} WHERE target_kind = 'remote'"
        ))
        .expect("remote execution conninfo source query should succeed")
        .expect("remote execution conninfo source should exist");
        let remote_candidate_format = Spi::get_one::<String>(&format!(
            "SELECT candidate_format {plan_from} WHERE target_kind = 'remote'"
        ))
        .expect("remote execution candidate format query should succeed")
        .expect("remote execution candidate format should exist");
        let remote_status = Spi::get_one::<String>(&format!(
            "SELECT status {plan_from} WHERE target_kind = 'remote'"
        ))
        .expect("remote execution status query should succeed")
        .expect("remote execution status should exist");
        let summary_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("execution summary status query should succeed")
            .expect("execution summary status should exist");
        let blocked_plan_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_plan_count {summary_from}"))
                .expect("execution summary blocked plan query should succeed")
                .expect("execution summary blocked plan count should exist");
        let blocked_pid_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_pid_count {summary_from}"))
                .expect("execution summary blocked pid query should succeed")
                .expect("execution summary blocked pid count should exist");

        assert_eq!(remote_transport, "libpq_pipeline");
        assert_eq!(remote_index_source, "remote_node_descriptor");
        assert_eq!(remote_conninfo_source, "remote_node_descriptor");
        assert_eq!(remote_candidate_format, "ec_spire_remote_search_v1");
        assert_eq!(remote_status, "requires_remote_node_descriptor");
        assert_eq!(summary_status, "requires_remote_node_descriptor");
        assert_eq!(blocked_plan_count, 1);
        assert_eq!(blocked_pid_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_exec_plan_degraded() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_exec_skip_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_exec_skip_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_exec_skip_sql_idx \
             ON ec_spire_remote_exec_skip_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_exec_skip_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_exec_skip_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_exec_skip_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pid as u64, "skipped");
        }
        let plan_from = format!(
            "FROM ec_spire_remote_search_execution_plan(\
             'ec_spire_remote_exec_skip_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'degraded')",
        );
        let summary_from = format!(
            "FROM ec_spire_remote_search_execution_summary(\
             'ec_spire_remote_exec_skip_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 1, 'degraded')",
        );
        let target_kind = Spi::get_one::<String>(&format!("SELECT target_kind {plan_from}"))
            .expect("execution degraded target query should succeed")
            .expect("execution degraded target should exist");
        let execution_transport =
            Spi::get_one::<String>(&format!("SELECT execution_transport {plan_from}"))
                .expect("execution degraded transport query should succeed")
                .expect("execution degraded transport should exist");
        let endpoint = Spi::get_one::<String>(&format!("SELECT endpoint_function {plan_from}"))
            .expect("execution degraded endpoint query should succeed")
            .expect("execution degraded endpoint should exist");
        let candidate_format =
            Spi::get_one::<String>(&format!("SELECT candidate_format {plan_from}"))
                .expect("execution degraded format query should succeed")
                .expect("execution degraded format should exist");
        let summary_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("execution degraded summary status query should succeed")
            .expect("execution degraded summary status should exist");
        let degraded_skipped_plan_count = Spi::get_one::<i64>(&format!(
            "SELECT degraded_skipped_plan_count {summary_from}"
        ))
        .expect("execution degraded skipped plan query should succeed")
        .expect("execution degraded skipped plan count should exist");

        assert_eq!(target_kind, "skipped");
        assert_eq!(execution_transport, "none");
        assert_eq!(endpoint, "none");
        assert_eq!(candidate_format, "none");
        assert_eq!(summary_status, "degraded_ready");
        assert_eq!(degraded_skipped_plan_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_libpq_req_blocked() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_libpq_req_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_libpq_req_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_libpq_req_sql_idx \
             ON ec_spire_remote_libpq_req_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_libpq_req_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_libpq_req_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_libpq_req_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let plan_from = format!(
            "FROM ec_spire_remote_search_libpq_request_plan(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let summary_from = format!(
            "FROM ec_spire_remote_search_libpq_request_summary(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let connection_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_plan(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let connection_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_summary(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let dispatch_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_plan(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let bind_from = format!(
            "FROM ec_spire_remote_search_libpq_bind_plan(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let bind_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_bind_summary(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let work_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_work_plan(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let work_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_work_summary(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let dispatch_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_summary(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let executor_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_readiness(\
             'ec_spire_remote_libpq_req_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let request_count = Spi::get_one::<i64>(&format!("SELECT count(*) {plan_from}"))
            .expect("libpq request count query should succeed")
            .expect("libpq request count should exist");
        let sql_template = Spi::get_one::<String>(&format!("SELECT sql_template {plan_from}"))
            .expect("libpq request SQL template query should succeed")
            .expect("libpq request SQL template should exist");
        let parameter_count = Spi::get_one::<i64>(&format!("SELECT parameter_count {plan_from}"))
            .expect("libpq request parameter count query should succeed")
            .expect("libpq request parameter count should exist");
        let result_column_count =
            Spi::get_one::<i64>(&format!("SELECT result_column_count {plan_from}"))
                .expect("libpq request result column count query should succeed")
                .expect("libpq request result column count should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {plan_from}"))
            .expect("libpq request status query should succeed")
            .expect("libpq request status should exist");
        let summary_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("libpq request summary status query should succeed")
            .expect("libpq request summary status should exist");
        let blocked_request_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_request_count {summary_from}"))
                .expect("libpq request summary blocked count query should succeed")
                .expect("libpq request summary blocked count should exist");
        let blocked_pid_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_pid_count {summary_from}"))
                .expect("libpq request summary blocked pid query should succeed")
                .expect("libpq request summary blocked pid count should exist");
        let connection_count = Spi::get_one::<i64>(&format!("SELECT count(*) {connection_from}"))
            .expect("libpq connection plan count query should succeed")
            .expect("libpq connection plan count should exist");
        let conninfo_resolution =
            Spi::get_one::<String>(&format!("SELECT conninfo_resolution {connection_from}"))
                .expect("libpq connection resolution query should succeed")
                .expect("libpq connection resolution should exist");
        let pipeline_mode =
            Spi::get_one::<String>(&format!("SELECT pipeline_mode {connection_from}"))
                .expect("libpq connection pipeline query should succeed")
                .expect("libpq connection pipeline should exist");
        let missing_descriptor_connection_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_connection_count {connection_summary_from}"
        ))
        .expect("libpq connection summary missing count query should succeed")
        .expect("libpq connection summary missing count should exist");
        let connection_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {connection_summary_from}"))
                .expect("libpq connection summary status query should succeed")
                .expect("libpq connection summary status should exist");
        let dispatch_action =
            Spi::get_one::<String>(&format!("SELECT dispatch_action {dispatch_from}"))
                .expect("libpq dispatch action query should succeed")
                .expect("libpq dispatch action should exist");
        let bind_blocked_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {bind_from} WHERE value_status = 'requires_remote_node_descriptor'"
        ))
        .expect("libpq bind blocked count query should succeed")
        .expect("libpq bind blocked count should exist");
        let bind_remote_index_preview = Spi::get_one::<String>(&format!(
            "SELECT value_preview {bind_from} WHERE parameter_name = 'remote_index_oid'"
        ))
        .expect("libpq bind remote index query should succeed")
        .expect("libpq bind remote index preview should exist");
        let bind_summary_bind_count =
            Spi::get_one::<i64>(&format!("SELECT bind_count {bind_summary_from}"))
                .expect("libpq bind summary count query should succeed")
                .expect("libpq bind summary count should exist");
        let bind_summary_blocked_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_bind_count {bind_summary_from}"))
                .expect("libpq bind summary blocked query should succeed")
                .expect("libpq bind summary blocked count should exist");
        let bind_summary_blocked_pid_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_pid_count {bind_summary_from}"))
                .expect("libpq bind summary blocked pid query should succeed")
                .expect("libpq bind summary blocked pid count should exist");
        let bind_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {bind_summary_from}"))
                .expect("libpq bind summary status query should succeed")
                .expect("libpq bind summary status should exist");
        let work_bind_status = Spi::get_one::<String>(&format!("SELECT bind_status {work_from}"))
            .expect("libpq work bind status query should succeed")
            .expect("libpq work bind status should exist");
        let work_next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {work_from}"))
                .expect("libpq work next step query should succeed")
                .expect("libpq work next step should exist");
        let work_action = Spi::get_one::<String>(&format!("SELECT work_action {work_from}"))
            .expect("libpq work action query should succeed")
            .expect("libpq work action should exist");
        let work_status = Spi::get_one::<String>(&format!("SELECT status {work_from}"))
            .expect("libpq work status query should succeed")
            .expect("libpq work status should exist");
        let work_summary_ready_count =
            Spi::get_one::<i64>(&format!("SELECT ready_work_count {work_summary_from}"))
                .expect("libpq work summary ready count query should succeed")
                .expect("libpq work summary ready count should exist");
        let work_summary_blocked_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_work_count {work_summary_from}"))
                .expect("libpq work summary blocked count query should succeed")
                .expect("libpq work summary blocked count should exist");
        let work_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {work_summary_from}"))
                .expect("libpq work summary status query should succeed")
                .expect("libpq work summary status should exist");
        let dispatch_missing_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_dispatch_count {dispatch_summary_from}"
        ))
        .expect("libpq dispatch summary missing count query should succeed")
        .expect("libpq dispatch summary missing count should exist");
        let dispatch_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {dispatch_summary_from}"))
                .expect("libpq dispatch summary status query should succeed")
                .expect("libpq dispatch summary status should exist");
        let executor_status = Spi::get_one::<String>(&format!("SELECT status {executor_from}"))
            .expect("libpq executor status query should succeed")
            .expect("libpq executor status should exist");
        let executor_next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {executor_from}"))
                .expect("libpq executor step query should succeed")
                .expect("libpq executor step should exist");

        assert_eq!(request_count, 1);
        assert!(sql_template.contains("ec_spire_remote_search"));
        assert_eq!(parameter_count, 6);
        assert_eq!(result_column_count, 18);
        assert_eq!(status, "requires_remote_node_descriptor");
        assert_eq!(summary_status, "requires_remote_node_descriptor");
        assert_eq!(blocked_request_count, 1);
        assert_eq!(blocked_pid_count, 1);
        assert_eq!(connection_count, 1);
        assert_eq!(conninfo_resolution, "requires_remote_node_descriptor");
        assert_eq!(pipeline_mode, "none");
        assert_eq!(missing_descriptor_connection_count, 1);
        assert_eq!(connection_summary_status, "requires_remote_node_descriptor");
        assert_eq!(dispatch_action, "blocked_before_dispatch");
        assert_eq!(bind_blocked_count, 6);
        assert_eq!(bind_remote_index_preview, "none");
        assert_eq!(bind_summary_bind_count, 6);
        assert_eq!(bind_summary_blocked_count, 6);
        assert_eq!(bind_summary_blocked_pid_count, 1);
        assert_eq!(bind_summary_status, "requires_remote_node_descriptor");
        assert_eq!(work_bind_status, "requires_remote_node_descriptor");
        assert_eq!(work_next_step, "remote_node_descriptor");
        assert_eq!(work_action, "blocked_before_executor");
        assert_eq!(work_status, "requires_remote_node_descriptor");
        assert_eq!(work_summary_ready_count, 0);
        assert_eq!(work_summary_blocked_count, 1);
        assert_eq!(work_summary_status, "requires_remote_node_descriptor");
        assert_eq!(dispatch_missing_count, 1);
        assert_eq!(dispatch_summary_status, "requires_remote_node_descriptor");
        assert_eq!(executor_status, "requires_remote_node_descriptor");
        assert_eq!(executor_next_step, "remote_node_descriptor");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_libpq_req_local() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_libpq_req_local_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_libpq_req_local_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_libpq_req_local_sql_idx \
             ON ec_spire_remote_libpq_req_local_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_libpq_req_local_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_libpq_req_local_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        let plan_from = format!(
            "FROM ec_spire_remote_search_libpq_request_plan(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let summary_from = format!(
            "FROM ec_spire_remote_search_libpq_request_summary(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let connection_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_plan(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let connection_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_summary(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let dispatch_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_plan(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let dispatch_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_summary(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let executor_from = format!(
            "FROM ec_spire_remote_search_libpq_executor_readiness(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let receive_summary_from = format!(
            "FROM ec_spire_remote_search_receive_summary(\
             'ec_spire_remote_libpq_req_local_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')",
        );
        let request_count = Spi::get_one::<i64>(&format!("SELECT count(*) {plan_from}"))
            .expect("local libpq request count query should succeed")
            .expect("local libpq request count should exist");
        let summary_request_count =
            Spi::get_one::<i64>(&format!("SELECT request_count {summary_from}"))
                .expect("local libpq summary request count query should succeed")
                .expect("local libpq summary request count should exist");
        let summary_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("local libpq summary status query should succeed")
            .expect("local libpq summary status should exist");
        let connection_count = Spi::get_one::<i64>(&format!("SELECT count(*) {connection_from}"))
            .expect("local libpq connection count query should succeed")
            .expect("local libpq connection count should exist");
        let connection_summary_count = Spi::get_one::<i64>(&format!(
            "SELECT connection_count {connection_summary_from}"
        ))
        .expect("local libpq connection summary count query should succeed")
        .expect("local libpq connection summary count should exist");
        let connection_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {connection_summary_from}"))
                .expect("local libpq connection summary status query should succeed")
                .expect("local libpq connection summary status should exist");
        let dispatch_count = Spi::get_one::<i64>(&format!("SELECT count(*) {dispatch_from}"))
            .expect("local libpq dispatch count query should succeed")
            .expect("local libpq dispatch count should exist");
        let dispatch_summary_count =
            Spi::get_one::<i64>(&format!("SELECT dispatch_count {dispatch_summary_from}"))
                .expect("local libpq dispatch summary count query should succeed")
                .expect("local libpq dispatch summary count should exist");
        let dispatch_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {dispatch_summary_from}"))
                .expect("local libpq dispatch summary status query should succeed")
                .expect("local libpq dispatch summary status should exist");
        let executor_status = Spi::get_one::<String>(&format!("SELECT status {executor_from}"))
            .expect("local libpq executor status query should succeed")
            .expect("local libpq executor status should exist");
        let executor_next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {executor_from}"))
                .expect("local libpq executor step query should succeed")
                .expect("local libpq executor step should exist");
        let receive_summary_count =
            Spi::get_one::<i64>(&format!("SELECT receive_count {receive_summary_from}"))
                .expect("local receive summary count query should succeed")
                .expect("local receive summary count should exist");
        let receive_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {receive_summary_from}"))
                .expect("local receive summary status query should succeed")
                .expect("local receive summary status should exist");

        assert_eq!(request_count, 0);
        assert_eq!(summary_request_count, 0);
        assert_eq!(connection_count, 0);
        assert_eq!(connection_summary_count, 0);
        assert_eq!(dispatch_count, 0);
        assert_eq!(dispatch_summary_count, 0);
        assert_eq!(summary_status, "ready");
        assert_eq!(connection_summary_status, "ready");
        assert_eq!(dispatch_summary_status, "ready");
        assert_eq!(executor_status, "ready");
        assert_eq!(executor_next_step, "none");
        assert_eq!(receive_summary_count, 0);
        assert_eq!(receive_summary_status, "ready");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_receive_contract() {
        let parameter_contract_from = "FROM ec_spire_remote_search_libpq_parameter_contract()";
        let executor_contract_from = "FROM ec_spire_remote_search_libpq_executor_step_contract()";
        let result_contract_from = "FROM ec_spire_remote_search_libpq_result_contract()";
        let endpoint_contract_from = "FROM ec_spire_remote_search_endpoint_contract()";
        let parameter_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {parameter_contract_from}"))
                .expect("parameter contract count query should succeed")
                .expect("parameter contract count should exist");
        let first_parameter = Spi::get_one::<String>(&format!(
            "SELECT parameter_name {parameter_contract_from} WHERE parameter_ordinal = 1"
        ))
        .expect("parameter contract first parameter query should succeed")
        .expect("parameter contract first parameter should exist");
        let selected_pids_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {parameter_contract_from} WHERE parameter_name = 'selected_pids'"
        ))
        .expect("parameter contract selected pids query should succeed")
        .expect("parameter contract selected pids validator should exist");
        let consistency_mode_role = Spi::get_one::<String>(&format!(
            "SELECT semantic_role {parameter_contract_from} WHERE parameter_name = 'consistency_mode'"
        ))
        .expect("parameter contract consistency mode query should succeed")
        .expect("parameter contract consistency mode role should exist");
        let column_count = Spi::get_one::<i64>(&format!("SELECT count(*) {result_contract_from}"))
            .expect("result contract count query should succeed")
            .expect("result contract count should exist");
        let first_column = Spi::get_one::<String>(&format!(
            "SELECT column_name {result_contract_from} WHERE column_ordinal = 1"
        ))
        .expect("result contract first column query should succeed")
        .expect("result contract first column should exist");
        let score_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {result_contract_from} WHERE column_name = 'score'"
        ))
        .expect("result contract score validator query should succeed")
        .expect("result contract score validator should exist");
        let pid_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {result_contract_from} WHERE column_name = 'pid'"
        ))
        .expect("result contract pid validator query should succeed")
        .expect("result contract pid validator should exist");
        let nullable_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {result_contract_from} WHERE nullable"
        ))
        .expect("result contract nullable count query should succeed")
        .expect("result contract nullable count should exist");
        let executor_step_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {executor_contract_from}"))
                .expect("executor contract count query should succeed")
                .expect("executor contract count should exist");
        let first_executor_step = Spi::get_one::<String>(&format!(
            "SELECT step_name {executor_contract_from} WHERE step_ordinal = 1"
        ))
        .expect("executor contract first step query should succeed")
        .expect("executor contract first step should exist");
        let secret_step_action = Spi::get_one::<String>(&format!(
            "SELECT executor_action {executor_contract_from} \
             WHERE step_name = 'conninfo_secret_resolution'"
        ))
        .expect("executor contract secret step query should succeed")
        .expect("executor contract secret step action should exist");
        let budget_step_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {executor_contract_from} \
             WHERE step_name = 'remote_executor_budget'"
        ))
        .expect("executor contract budget step query should succeed")
        .expect("executor contract budget step validator should exist");
        let merge_step_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {executor_contract_from} \
             WHERE step_name = 'merge_validated_remote_search_candidate_batches'"
        ))
        .expect("executor contract merge step query should succeed")
        .expect("executor contract merge step validator should exist");
        let endpoint_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {endpoint_contract_from}"))
                .expect("endpoint contract count query should succeed")
                .expect("endpoint contract count should exist");
        let endpoint_protocol = Spi::get_one::<String>(&format!(
            "SELECT contract_value {endpoint_contract_from} \
             WHERE contract_item = 'protocol_version'"
        ))
        .expect("endpoint contract protocol query should succeed")
        .expect("endpoint contract protocol should exist");
        let endpoint_quantizer = Spi::get_one::<String>(&format!(
            "SELECT contract_value {endpoint_contract_from} \
             WHERE contract_item = 'quantizer_family'"
        ))
        .expect("endpoint contract quantizer query should succeed")
        .expect("endpoint contract quantizer should exist");
        let endpoint_tuple_transport_default = Spi::get_one::<String>(&format!(
            "SELECT contract_value {endpoint_contract_from} \
             WHERE contract_item = 'tuple_transport_default'"
        ))
        .expect("endpoint contract tuple transport query should succeed")
        .expect("endpoint contract tuple transport should exist");
        let endpoint_tuple_transport_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {endpoint_contract_from} \
             WHERE contract_item = 'tuple_transport_capabilities'"
        ))
        .expect("endpoint contract tuple transport validator query should succeed")
        .expect("endpoint contract tuple transport validator should exist");
        let fingerprint_status = Spi::get_one::<String>(&format!(
            "SELECT status {endpoint_contract_from} \
             WHERE contract_item = 'quantizer_index_fingerprint_binding'"
        ))
        .expect("endpoint contract fingerprint query should succeed")
        .expect("endpoint contract fingerprint status should exist");
        let direct_sql_policy_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {endpoint_contract_from} \
             WHERE contract_item = 'direct_sql_endpoint_status_policy'"
        ))
        .expect("endpoint contract direct SQL policy query should succeed")
        .expect("endpoint contract direct SQL policy should exist");
        let heap_preflight_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {endpoint_contract_from} \
             WHERE contract_item = 'remote_heap_candidate_endpoint_identity_preflight'"
        ))
        .expect("endpoint contract heap preflight query should succeed")
        .expect("endpoint contract heap preflight should exist");
        let non_ready_endpoint_rows = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {endpoint_contract_from} WHERE status <> 'ready'"
        ))
        .expect("endpoint contract non-ready query should succeed")
        .expect("endpoint contract non-ready count should exist");

        assert_eq!(parameter_count, 6);
        assert_eq!(first_parameter, "remote_index_oid");
        assert_eq!(
            selected_pids_validator,
            "must_be_nonempty_positive_unique_remote_leaf_pids_delta_rows_are_leaf_derived"
        );
        assert_eq!(consistency_mode_role, "strict_or_degraded_policy");
        assert_eq!(column_count, 18);
        assert_eq!(first_column, "served_epoch");
        assert_eq!(
            pid_validator,
            "must_be_selected_leaf_pid_or_leaf_derived_delta_pid"
        );
        assert_eq!(score_validator, "must_be_finite");
        assert_eq!(nullable_count, 0);
        assert_eq!(executor_step_count, 10);
        assert_eq!(first_executor_step, "remote_node_descriptor");
        assert_eq!(secret_step_action, "resolve_conninfo_secret_reference");
        assert_eq!(
            budget_step_validator,
            "must_block_over_budget_rows_before_secret_lookup_or_socket_open"
        );
        assert_eq!(merge_step_validator, "must_preserve_merge_order_contract");
        assert_eq!(endpoint_count, 15);
        assert_eq!(endpoint_protocol, "ec_spire_remote_search_v1");
        assert_eq!(endpoint_quantizer, "rabitq_only_pq_and_pqfastscan_reserved");
        assert_eq!(endpoint_tuple_transport_default, "pg_binary_attr_v1");
        assert_eq!(
            endpoint_tuple_transport_validator,
            "must_advertise_pg_binary_attr_v1_before_custom_scan_typed_receive"
        );
        assert_eq!(fingerprint_status, "requires_fingerprint_binding");
        assert_eq!(
            direct_sql_policy_validator,
            "must_not_treat_direct_sql_rows_as_mergeable_without_libpq_receive_validation"
        );
        assert_eq!(
            heap_preflight_validator,
            "must_validate_ready_endpoint_identity_before_remote_heap_candidate_merge"
        );
        assert_eq!(non_ready_endpoint_rows, 3);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_endpoint_identity() {
        Spi::run(
            "CREATE TABLE ec_spire_endpoint_identity_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_endpoint_identity_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_endpoint_identity_default_idx \
             ON ec_spire_endpoint_identity_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("default ec_spire index creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_endpoint_identity_rabitq_idx \
             ON ec_spire_endpoint_identity_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 2, storage_format = 'rabitq')",
        )
        .expect("rabitq ec_spire index creation should succeed");

        let default_from =
            "FROM ec_spire_remote_search_endpoint_identity('ec_spire_endpoint_identity_default_idx'::regclass)";
        let rabitq_from =
            "FROM ec_spire_remote_search_endpoint_identity('ec_spire_endpoint_identity_rabitq_idx'::regclass)";
        let default_status = Spi::get_one::<String>(&format!("SELECT status {default_from}"))
            .expect("default identity status query should succeed")
            .expect("default identity status should exist");
        let default_assignment_payload =
            Spi::get_one::<String>(&format!("SELECT assignment_payload_format {default_from}"))
                .expect("default identity payload query should succeed")
                .expect("default identity payload should exist");
        let rabitq_status = Spi::get_one::<String>(&format!("SELECT status {rabitq_from}"))
            .expect("rabitq identity status query should succeed")
            .expect("rabitq identity status should exist");
        let protocol_version =
            Spi::get_one::<String>(&format!("SELECT protocol_version {rabitq_from}"))
                .expect("rabitq identity protocol query should succeed")
                .expect("rabitq identity protocol should exist");
        let extension_version =
            Spi::get_one::<String>(&format!("SELECT extension_version {rabitq_from}"))
                .expect("rabitq identity extension query should succeed")
                .expect("rabitq identity extension should exist");
        let opclass_identity =
            Spi::get_one::<String>(&format!("SELECT opclass_identity {rabitq_from}"))
                .expect("rabitq identity opclass query should succeed")
                .expect("rabitq identity opclass should exist");
        let storage_format =
            Spi::get_one::<String>(&format!("SELECT storage_format {rabitq_from}"))
                .expect("rabitq identity storage query should succeed")
                .expect("rabitq identity storage should exist");
        let assignment_payload =
            Spi::get_one::<String>(&format!("SELECT assignment_payload_format {rabitq_from}"))
                .expect("rabitq identity payload query should succeed")
                .expect("rabitq identity payload should exist");
        let quantizer_profile =
            Spi::get_one::<String>(&format!("SELECT quantizer_profile {rabitq_from}"))
                .expect("rabitq identity profile query should succeed")
                .expect("rabitq identity profile should exist");
        let scoring_profile =
            Spi::get_one::<String>(&format!("SELECT scoring_profile {rabitq_from}"))
                .expect("rabitq identity scoring query should succeed")
                .expect("rabitq identity scoring should exist");
        let tuple_transport_capabilities = Spi::get_one::<Vec<String>>(&format!(
            "SELECT tuple_transport_capabilities {rabitq_from}"
        ))
        .expect("rabitq tuple transport capabilities query should succeed")
        .expect("rabitq tuple transport capabilities should exist");
        let tuple_transport_default =
            Spi::get_one::<String>(&format!("SELECT tuple_transport_default {rabitq_from}"))
                .expect("rabitq tuple transport default query should succeed")
                .expect("rabitq tuple transport default should exist");
        let tuple_transport_status =
            Spi::get_one::<String>(&format!("SELECT tuple_transport_status {rabitq_from}"))
                .expect("rabitq tuple transport status query should succeed")
                .expect("rabitq tuple transport status should exist");
        let fingerprint_length =
            Spi::get_one::<i32>(&format!("SELECT length(profile_fingerprint) {rabitq_from}"))
                .expect("rabitq identity fingerprint query should succeed")
                .expect("rabitq identity fingerprint should exist");
        let fingerprint_before_reindex =
            Spi::get_one::<String>(&format!("SELECT profile_fingerprint {rabitq_from}"))
                .expect("rabitq identity fingerprint query should succeed")
                .expect("rabitq identity fingerprint should exist");

        Spi::run("REINDEX INDEX ec_spire_endpoint_identity_rabitq_idx")
            .expect("reindex should succeed");
        let fingerprint_after_reindex =
            Spi::get_one::<String>(&format!("SELECT profile_fingerprint {rabitq_from}"))
                .expect("reindexed rabitq identity fingerprint query should succeed")
                .expect("reindexed rabitq identity fingerprint should exist");

        assert_eq!(default_status, "requires_rabitq_storage_format");
        assert_eq!(default_assignment_payload, "turboquant");
        assert_eq!(rabitq_status, "ready");
        assert_eq!(protocol_version, "ec_spire_remote_search_v1");
        assert_eq!(extension_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(opclass_identity, "ecvector_spire_ip_ops");
        assert_eq!(storage_format, "rabitq");
        assert_eq!(assignment_payload, "rabitq");
        assert_eq!(quantizer_profile, "rabitq_v1");
        assert_eq!(scoring_profile, "inner_product_score_v1");
        assert_eq!(tuple_transport_capabilities, vec!["pg_binary_attr_v1"]);
        assert_eq!(tuple_transport_default, "pg_binary_attr_v1");
        assert_eq!(tuple_transport_status, "ready");
        assert_eq!(fingerprint_length, 16);
        assert_ne!(fingerprint_before_reindex, fingerprint_after_reindex);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_receive_plan_blocked() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_receive_plan_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_receive_plan_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_receive_plan_sql_idx \
             ON ec_spire_remote_receive_plan_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_receive_plan_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_receive_plan_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_receive_plan_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let receive_from = format!(
            "FROM ec_spire_remote_search_receive_plan(\
             'ec_spire_remote_receive_plan_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let receive_summary_from = format!(
            "FROM ec_spire_remote_search_receive_summary(\
             'ec_spire_remote_receive_plan_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {receive_from}"))
            .expect("receive plan count query should succeed")
            .expect("receive plan count should exist");
        let validator_function =
            Spi::get_one::<String>(&format!("SELECT validator_function {receive_from}"))
                .expect("receive plan validator query should succeed")
                .expect("receive plan validator should exist");
        let row_locator_policy =
            Spi::get_one::<String>(&format!("SELECT row_locator_policy {receive_from}"))
                .expect("receive plan locator policy query should succeed")
                .expect("receive plan locator policy should exist");
        let candidate_format =
            Spi::get_one::<String>(&format!("SELECT expected_candidate_format {receive_from}"))
                .expect("receive plan candidate format query should succeed")
                .expect("receive plan candidate format should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {receive_from}"))
            .expect("receive plan status query should succeed")
            .expect("receive plan status should exist");
        let summary_receive_count =
            Spi::get_one::<i64>(&format!("SELECT receive_count {receive_summary_from}"))
                .expect("receive summary count query should succeed")
                .expect("receive summary count should exist");
        let summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_receive_count {receive_summary_from}"
        ))
        .expect("receive summary ready count query should succeed")
        .expect("receive summary ready count should exist");
        let summary_blocked_count = Spi::get_one::<i64>(&format!(
            "SELECT blocked_receive_count {receive_summary_from}"
        ))
        .expect("receive summary blocked count query should succeed")
        .expect("receive summary blocked count should exist");
        let summary_blocked_pid_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_pid_count {receive_summary_from}"))
                .expect("receive summary blocked pid query should succeed")
                .expect("receive summary blocked pid count should exist");
        let summary_status =
            Spi::get_one::<String>(&format!("SELECT status {receive_summary_from}"))
                .expect("receive summary status query should succeed")
                .expect("receive summary status should exist");

        assert_eq!(row_count, 1);
        assert_eq!(validator_function, "validate_remote_search_candidate_batch");
        assert_eq!(row_locator_policy, "opaque_origin_node_bytes");
        assert_eq!(candidate_format, "ec_spire_remote_search_v1");
        assert_eq!(status, "requires_remote_node_descriptor");
        assert_eq!(summary_receive_count, 1);
        assert_eq!(summary_ready_count, 0);
        assert_eq!(summary_blocked_count, 1);
        assert_eq!(summary_blocked_pid_count, 1);
        assert_eq!(summary_status, "requires_remote_node_descriptor");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_receive_merge_summary() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_receive_merge_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_receive_merge_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_receive_merge_sql_idx \
             ON ec_spire_remote_receive_merge_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_receive_merge_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_receive_merge_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_receive_merge_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let summary_from = format!(
            "FROM ec_spire_remote_search_merge_input_summary(\
             'ec_spire_remote_receive_merge_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let remote_batch_count =
            Spi::get_one::<i64>(&format!("SELECT remote_batch_count {summary_from}"))
                .expect("merge input remote batch query should succeed")
                .expect("merge input remote batch count should exist");
        let local_batch_count =
            Spi::get_one::<i64>(&format!("SELECT local_batch_count {summary_from}"))
                .expect("merge input local batch query should succeed")
                .expect("merge input local batch count should exist");
        let blocked_batch_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_batch_count {summary_from}"))
                .expect("merge input blocked batch query should succeed")
                .expect("merge input blocked batch count should exist");
        let merge_function =
            Spi::get_one::<String>(&format!("SELECT merge_function {summary_from}"))
                .expect("merge input merge function query should succeed")
                .expect("merge input merge function should exist");
        let dedupe_key = Spi::get_one::<String>(&format!("SELECT dedupe_key {summary_from}"))
            .expect("merge input dedupe key query should succeed")
            .expect("merge input dedupe key should exist");
        let tie_breaker = Spi::get_one::<String>(&format!("SELECT tie_breaker {summary_from}"))
            .expect("merge input tie-breaker query should succeed")
            .expect("merge input tie-breaker should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("merge input status query should succeed")
            .expect("merge input status should exist");

        assert_eq!(remote_batch_count, 1);
        assert_eq!(local_batch_count, 1);
        assert_eq!(blocked_batch_count, 1);
        assert_eq!(
            merge_function,
            "merge_validated_remote_search_candidate_batches"
        );
        assert_eq!(dedupe_key, "global_vec_id_or_node_scoped_local_vec_id");
        assert_eq!(
            tie_breaker,
            "score_then_assignment_role_then_epoch_desc_then_node_pid_version_row_locator"
        );
        assert_eq!(status, "requires_remote_node_descriptor");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_final_contract() {
        let locator_contract_from = "FROM ec_spire_remote_search_row_locator_contract()";
        let identity_contract_from = "FROM ec_spire_remote_search_vector_identity_contract()";
        let heap_contract_from = "FROM ec_spire_remote_search_heap_resolution_contract()";
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {locator_contract_from}"))
            .expect("row locator contract count query should succeed")
            .expect("row locator contract count should exist");
        let identity_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {identity_contract_from}"))
                .expect("vector identity contract count query should succeed")
                .expect("vector identity contract count should exist");
        let dedupe_key = Spi::get_one::<String>(&format!(
            "SELECT contract_value {identity_contract_from} \
             WHERE contract_item = 'remote_merge_dedupe_key'"
        ))
        .expect("vector identity dedupe key query should succeed")
        .expect("vector identity dedupe key should exist");
        let local_scope = Spi::get_one::<String>(&format!(
            "SELECT contract_value {identity_contract_from} \
             WHERE contract_item = 'local_vec_id_remote_scope'"
        ))
        .expect("vector identity local scope query should succeed")
        .expect("vector identity local scope should exist");
        let writer_global_source_identity = Spi::get_one::<String>(&format!(
            "SELECT contract_value {identity_contract_from} \
             WHERE contract_item = 'writer_global_source_identity'"
        ))
        .expect("vector identity writer source query should succeed")
        .expect("vector identity writer source should exist");
        let writer_global_base_storage_status = Spi::get_one::<String>(&format!(
            "SELECT status {identity_contract_from} \
             WHERE contract_item = 'writer_global_base_storage'"
        ))
        .expect("vector identity writer storage query should succeed")
        .expect("vector identity writer storage should exist");
        let interpretation = Spi::get_one::<String>(&format!(
            "SELECT contract_value {locator_contract_from} \
             WHERE contract_item = 'coordinator_interpretation'"
        ))
        .expect("row locator interpretation query should succeed")
        .expect("row locator interpretation should exist");
        let remote_resolution_status = Spi::get_one::<String>(&format!(
            "SELECT status {locator_contract_from} \
             WHERE contract_item = 'remote_heap_resolution'"
        ))
        .expect("row locator remote resolution status query should succeed")
        .expect("row locator remote resolution status should exist");
        let heap_resolution_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {heap_contract_from}"))
                .expect("heap resolution contract count query should succeed")
                .expect("heap resolution contract count should exist");
        let local_heap_owner = Spi::get_one::<String>(&format!(
            "SELECT heap_lookup_owner {heap_contract_from} WHERE resolution_scope = 'local'"
        ))
        .expect("local heap resolution owner query should succeed")
        .expect("local heap resolution owner should exist");
        let remote_heap_status = Spi::get_one::<String>(&format!(
            "SELECT status {heap_contract_from} WHERE resolution_scope = 'remote'"
        ))
        .expect("remote heap resolution status query should succeed")
        .expect("remote heap resolution status should exist");
        let remote_locator_policy = Spi::get_one::<String>(&format!(
            "SELECT row_locator_policy {heap_contract_from} WHERE resolution_scope = 'remote'"
        ))
        .expect("remote heap resolution locator query should succeed")
        .expect("remote heap resolution locator should exist");

        assert_eq!(row_count, 4);
        assert_eq!(identity_count, 10);
        assert_eq!(dedupe_key, "global_vec_id_or_node_scoped_local_vec_id");
        assert_eq!(local_scope, "node_id || local_vec_id_bytes");
        assert_eq!(
            writer_global_source_identity,
            "fixed_16_byte_source_identity_required_not_heap_tid"
        );
        assert_eq!(writer_global_base_storage_status, "phase11_2_landed");
        assert_eq!(interpretation, "opaque_bytes");
        assert_eq!(remote_resolution_status, "deferred_until_remote_heap_fetch");
        assert_eq!(heap_resolution_count, 2);
        assert_eq!(local_heap_owner, "coordinator_local_heap");
        assert_eq!(remote_heap_status, "requires_remote_heap_resolution");
        assert_eq!(remote_locator_policy, "opaque_origin_node_bytes");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_local_heap_resolution_plan() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_local_heap_res_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_local_heap_res_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_local_heap_res_sql_idx \
             ON ec_spire_remote_local_heap_res_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_local_heap_res_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_local_heap_res_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let plan_from = format!(
            "FROM ec_spire_remote_search_local_heap_resolution_plan(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let summary_from = format!(
            "FROM ec_spire_remote_search_heap_resolution_summary(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let candidates_from = format!(
            "FROM ec_spire_remote_search_local_heap_candidates(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let candidate_summary_from = format!(
            "FROM ec_spire_remote_search_local_heap_candidate_summary(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let result_summary_from = format!(
            "FROM ec_spire_remote_search_coordinator_result_summary(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {plan_from}"))
            .expect("local heap resolution count query should succeed")
            .expect("local heap resolution count should exist");
        let ready_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {plan_from} WHERE status = 'ready'"
        ))
        .expect("local heap resolution ready count query should succeed")
        .expect("local heap resolution ready count should exist");
        let owner_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {plan_from} WHERE heap_lookup_owner = 'coordinator_local_heap'"
        ))
        .expect("local heap resolution owner count query should succeed")
        .expect("local heap resolution owner count should exist");
        let decoded_locator_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {plan_from} \
             WHERE heap_block >= 0 AND heap_offset > 0 AND length(row_locator) = 6"
        ))
        .expect("local heap resolution locator query should succeed")
        .expect("local heap resolution locator count should exist");
        let node_count =
            Spi::get_one::<i64>(&format!("SELECT count(DISTINCT node_id) {plan_from}"))
                .expect("local heap resolution node count query should succeed")
                .expect("local heap resolution node count should exist");
        let summary_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("heap resolution summary status query should succeed")
            .expect("heap resolution summary status should exist");
        let decoded_summary_count = Spi::get_one::<i64>(&format!(
            "SELECT decoded_local_locator_count {summary_from}"
        ))
        .expect("heap resolution summary decoded count query should succeed")
        .expect("heap resolution summary decoded count should exist");
        let local_resolution_status = Spi::get_one::<String>(&format!(
            "SELECT local_heap_resolution_status {summary_from}"
        ))
        .expect("heap resolution summary local status query should succeed")
        .expect("heap resolution summary local status should exist");
        let remote_resolution_status = Spi::get_one::<String>(&format!(
            "SELECT remote_heap_resolution_status {summary_from}"
        ))
        .expect("heap resolution summary remote status query should succeed")
        .expect("heap resolution summary remote status should exist");
        let candidate_count = Spi::get_one::<i64>(&format!("SELECT count(*) {candidates_from}"))
            .expect("local heap candidate count query should succeed")
            .expect("local heap candidate count should exist");
        let candidate_owner_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {candidates_from} \
             WHERE heap_lookup_owner = 'coordinator_local_heap' AND status = 'ready'"
        ))
        .expect("local heap candidate owner query should succeed")
        .expect("local heap candidate owner count should exist");
        let candidate_locator_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {candidates_from} \
             WHERE served_epoch = requested_epoch \
             AND heap_block >= 0 AND heap_offset > 0 \
             AND length(row_locator) = 6 AND score IS NOT NULL"
        ))
        .expect("local heap candidate locator query should succeed")
        .expect("local heap candidate locator count should exist");
        let returned_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {candidate_summary_from}"
        ))
        .expect("local heap candidate summary return query should succeed")
        .expect("local heap candidate summary return count should exist");
        let candidate_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {candidate_summary_from}"))
                .expect("local heap candidate summary status query should succeed")
                .expect("local heap candidate summary status should exist");
        let result_source =
            Spi::get_one::<String>(&format!("SELECT result_source {result_summary_from}"))
                .expect("coordinator result source query should succeed")
                .expect("coordinator result source should exist");
        let result_receive_count =
            Spi::get_one::<i64>(&format!("SELECT libpq_receive_count {result_summary_from}"))
                .expect("coordinator result receive count query should succeed")
                .expect("coordinator result receive count should exist");
        let result_receive_status = Spi::get_one::<String>(&format!(
            "SELECT libpq_receive_status {result_summary_from}"
        ))
        .expect("coordinator result receive status query should succeed")
        .expect("coordinator result receive status should exist");
        let result_status = Spi::get_one::<String>(&format!("SELECT status {result_summary_from}"))
            .expect("coordinator result status query should succeed")
            .expect("coordinator result status should exist");
        let result_returned_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {result_summary_from}"
        ))
        .expect("coordinator result returned count query should succeed")
        .expect("coordinator result returned count should exist");
        let result_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {result_summary_from}"))
                .expect("coordinator result blocker query should succeed")
                .expect("coordinator result blocker should exist");

        assert_eq!(row_count, 2);
        assert_eq!(ready_count, row_count);
        assert_eq!(owner_count, row_count);
        assert_eq!(decoded_locator_count, row_count);
        assert_eq!(node_count, 1);
        assert_eq!(summary_status, "ready");
        assert_eq!(decoded_summary_count, row_count);
        assert_eq!(local_resolution_status, "ready");
        assert_eq!(remote_resolution_status, "none");
        assert_eq!(candidate_count, row_count);
        assert_eq!(candidate_owner_count, candidate_count);
        assert_eq!(candidate_locator_count, candidate_count);
        assert_eq!(returned_candidate_count, candidate_count);
        assert_eq!(candidate_summary_status, "ready");
        assert_eq!(result_source, "local_heap_candidates");
        assert_eq!(result_receive_count, 0);
        assert_eq!(result_receive_status, "ready");
        assert_eq!(result_status, "ready");
        assert_eq!(result_returned_candidate_count, candidate_count);
        assert_eq!(result_next_blocker, "none");

        Spi::run(
            "INSERT INTO ec_spire_remote_local_heap_res_sql (id, embedding) VALUES \
             (3, encode_to_ecvector(ARRAY[0.9, 0.1], 4, 42))",
        )
        .expect("post-build insert should publish a delta epoch");
        let delta_active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_local_heap_res_sql_idx'::regclass)",
        )
        .expect("post-insert hierarchy snapshot query should succeed")
        .expect("post-insert active epoch should exist");
        let delta_selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_local_heap_res_sql_idx'::regclass)",
        )
        .expect("post-insert leaf snapshot query should succeed")
        .expect("post-insert leaf pids should exist");
        let delta_candidates_from = format!(
            "FROM ec_spire_remote_search_local_heap_candidates(\
             'ec_spire_remote_local_heap_res_sql_idx'::regclass, \
             {delta_active_epoch}, ARRAY[0.9, 0.1]::real[], \
             ARRAY[{}, {}]::bigint[], 3, 'strict')",
            delta_selected_pids[0], delta_selected_pids[1],
        );
        let post_insert_delta_object_count = ec_spire_active_snapshot_i64(
            "ec_spire_remote_local_heap_res_sql_idx",
            "delta_object_count",
        );
        let delta_candidate_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {delta_candidates_from}"))
                .expect("post-insert local heap candidate count query should succeed")
                .expect("post-insert local heap candidate count should exist");
        let delta_candidate_locator_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {delta_candidates_from} \
             WHERE served_epoch = requested_epoch \
             AND heap_block >= 0 AND heap_offset > 0 \
             AND length(row_locator) = 6 AND score IS NOT NULL"
        ))
        .expect("post-insert local heap candidate locator query should succeed")
        .expect("post-insert local heap candidate locator count should exist");

        assert_eq!(delta_active_epoch, active_epoch + 1);
        assert_eq!(post_insert_delta_object_count, 1);
        assert_eq!(delta_candidate_count, 3);
        assert_eq!(delta_candidate_locator_count, delta_candidate_count);
    }

    #[pg_test]
    fn test_ec_spire_remote_search_tuple_payload_side_channel() {
        Spi::run(
            "CREATE TABLE ec_spire_tuple_payload_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_tuple_payload_sql (id, title, embedding) VALUES \
             (1, 'alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_tuple_payload_sql_idx \
             ON ec_spire_tuple_payload_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_tuple_payload_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_tuple_payload_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let payload_from = format!(
            "FROM ec_spire_remote_search_tuple_payload(\
             'ec_spire_tuple_payload_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict', ARRAY['id', 'title']::text[])",
            selected_pids[0], selected_pids[1],
        );
        let payload_count = Spi::get_one::<i64>(&format!("SELECT count(*) {payload_from}"))
            .expect("tuple payload count query should succeed")
            .expect("tuple payload count should exist");
        let key_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {payload_from} WHERE payload_key = 'node_id_vec_id'"
        ))
        .expect("tuple payload key query should succeed")
        .expect("tuple payload key count should exist");
        let column_count =
            Spi::get_one::<i32>(&format!("SELECT min(payload_column_count) {payload_from}"))
                .expect("tuple payload column count query should succeed")
                .expect("tuple payload column count should exist");
        let exact_projection_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {payload_from} \
             WHERE tuple_payload ? 'id' \
               AND tuple_payload ? 'title' \
               AND NOT tuple_payload ? 'embedding'"
        ))
        .expect("tuple payload projection query should succeed")
        .expect("tuple payload projection count should exist");
        let alpha_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {payload_from} \
             WHERE tuple_payload ->> 'id' = '1' \
               AND tuple_payload ->> 'title' = 'alpha'"
        ))
        .expect("tuple payload value query should succeed")
        .expect("tuple payload value count should exist");
        let missing_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {payload_from} \
             WHERE tuple_payload_missing"
        ))
        .expect("tuple payload missing query should succeed")
        .expect("tuple payload missing count should exist");
        let ready_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {payload_from} \
             WHERE status = 'ready'"
        ))
        .expect("tuple payload status query should succeed")
        .expect("tuple payload ready count should exist");

        assert_eq!(payload_count, 2);
        assert_eq!(key_count, payload_count);
        assert_eq!(column_count, 2);
        assert_eq!(exact_projection_count, payload_count);
        assert_eq!(alpha_count, 1);
        assert_eq!(missing_count, 0);
        assert_eq!(ready_count, payload_count);
    }

    #[pg_test]
    fn test_ec_spire_typed_tuple_payload_scalar_parity_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_tuple_payload_typed_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_tuple_payload_typed_sql (id, title, embedding) VALUES \
             (1, 'alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_tuple_payload_typed_sql_idx \
             ON ec_spire_tuple_payload_typed_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_tuple_payload_typed_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_tuple_payload_typed_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let endpoint_args = format!(
            "'ec_spire_tuple_payload_typed_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict', ARRAY['id', 'title']::text[]",
            selected_pids[0], selected_pids[1],
        );
        let json_alpha_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_search_tuple_payload({endpoint_args}) \
              WHERE tuple_payload ->> 'id' = '1' \
                AND tuple_payload ->> 'title' = 'alpha' \
                AND NOT tuple_payload ? 'embedding' \
                AND status = 'ready'"
        ))
        .expect("JSON tuple payload parity query should succeed")
        .expect("JSON tuple payload parity count should exist");
        let typed_summary = Spi::get_one::<String>(&format!(
            "SELECT count(*)::text || '|' || \
                    min(payload_column_count)::text || '|' || \
                    count(*) FILTER (WHERE payload_key = 'node_id_vec_id')::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport = 'pg_binary_attr_v1')::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport_status = 'ready')::text || '|' || \
                    count(*) FILTER (WHERE status = 'ready')::text \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args})"
        ))
        .expect("typed tuple payload summary query should succeed")
        .expect("typed tuple payload summary should exist");
        let typed_alpha_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args}) \
              WHERE payload_attnums = ARRAY[1, 2]::int2[] \
                AND payload_names = ARRAY['id', 'title']::text[] \
                AND payload_type_oids = ARRAY['int8'::regtype::oid, 'text'::regtype::oid]::oid[] \
                AND payload_typmods = ARRAY[-1, -1]::int4[] \
                AND payload_nulls = ARRAY[false, false]::boolean[] \
                AND payload_formats = ARRAY['pg_binary_attr_v1', 'pg_binary_attr_v1']::text[] \
                AND payload_values[1] = int8send(1::bigint)::bytea \
                AND payload_values[2] = textsend('alpha'::text)::bytea \
                AND NOT tuple_payload_missing \
                AND tuple_transport_status = 'ready' \
                AND status = 'ready'"
        ))
        .expect("typed tuple payload scalar value query should succeed")
        .expect("typed tuple payload scalar value count should exist");
        let empty_projection_args = format!(
            "'ec_spire_tuple_payload_typed_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict', ARRAY[]::text[]",
            selected_pids[0], selected_pids[1],
        );
        let empty_projection_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_search_tuple_payload_typed({empty_projection_args}) \
              WHERE payload_column_count = 0 \
                AND payload_attnums = ARRAY[]::int2[] \
                AND payload_names = ARRAY[]::text[] \
                AND payload_type_oids = ARRAY[]::oid[] \
                AND payload_typmods = ARRAY[]::int4[] \
                AND payload_collations = ARRAY[]::oid[] \
                AND payload_nulls = ARRAY[]::boolean[] \
                AND payload_values = ARRAY[]::bytea[] \
                AND payload_formats = ARRAY[]::text[] \
                AND NOT tuple_payload_missing \
                AND tuple_transport = 'pg_binary_attr_v1' \
                AND tuple_transport_status = 'ready' \
                AND status = 'ready'"
        ))
        .expect("typed tuple payload empty projection query should succeed")
        .expect("typed tuple payload empty projection count should exist");

        assert_eq!(json_alpha_count, 1);
        assert_eq!(typed_summary, "2|2|2|2|2|2");
        assert_eq!(typed_alpha_count, 1);
        assert_eq!(empty_projection_count, 2);
    }

    #[pg_test]
    fn test_ec_spire_typed_tuple_payload_null_array_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_tuple_payload_typed_array_sql \
             (id bigint primary key, title text, tags text[] not null, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_tuple_payload_typed_array_sql (id, title, tags, embedding) VALUES \
             (1, NULL, ARRAY['red', 'blue']::text[], encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'beta', ARRAY['green']::text[], encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_tuple_payload_typed_array_idx \
             ON ec_spire_tuple_payload_typed_array_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_tuple_payload_typed_array_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_tuple_payload_typed_array_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let endpoint_args = format!(
            "'ec_spire_tuple_payload_typed_array_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict', ARRAY['id', 'title', 'tags']::text[]",
            selected_pids[0], selected_pids[1],
        );
        let typed_summary = Spi::get_one::<String>(&format!(
            "SELECT count(*)::text || '|' || \
                    min(payload_column_count)::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport = 'pg_binary_attr_v1')::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport_status = 'ready')::text || '|' || \
                    count(*) FILTER (WHERE status = 'ready')::text \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args})"
        ))
        .expect("typed tuple payload summary query should succeed")
        .expect("typed tuple payload summary should exist");
        let alpha_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args}) \
              WHERE payload_attnums = ARRAY[1, 2, 3]::int2[] \
                AND payload_names = ARRAY['id', 'title', 'tags']::text[] \
                AND payload_type_oids = ARRAY[\
                    'int8'::regtype::oid, \
                    'text'::regtype::oid, \
                    'text[]'::regtype::oid]::oid[] \
                AND payload_nulls = ARRAY[false, true, false]::boolean[] \
                AND payload_formats = ARRAY[\
                    'pg_binary_attr_v1', \
                    'pg_binary_attr_v1', \
                    'pg_binary_attr_v1']::text[] \
                AND payload_values[1] = int8send(1::bigint)::bytea \
                AND payload_values[2] = ''::bytea \
                AND payload_values[3] = array_send(ARRAY['red', 'blue']::text[])::bytea \
                AND NOT tuple_payload_missing \
                AND tuple_transport_status = 'ready' \
                AND status = 'ready'"
        ))
        .expect("typed tuple payload NULL/array query should succeed")
        .expect("typed tuple payload NULL/array count should exist");

        assert_eq!(typed_summary, "2|3|2|2|2");
        assert_eq!(alpha_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_typed_tuple_payload_domain_composite_sql() {
        Spi::run(
            "CREATE DOMAIN ec_spire_typed_label_domain AS text \
             CHECK (VALUE <> 'blocked')",
        )
        .expect("domain creation should succeed");
        Spi::run("CREATE TYPE ec_spire_typed_pair AS (code int4, label text)")
            .expect("composite type creation should succeed");
        Spi::run(
            "CREATE TABLE ec_spire_tuple_payload_typed_record_sql \
             (id bigint primary key, \
              label ec_spire_typed_label_domain not null, \
              pair ec_spire_typed_pair not null, \
              embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_tuple_payload_typed_record_sql \
             (id, label, pair, embedding) VALUES \
             (1, 'alpha'::ec_spire_typed_label_domain, \
              ROW(7, 'left')::ec_spire_typed_pair, \
              encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'beta'::ec_spire_typed_label_domain, \
              ROW(9, 'right')::ec_spire_typed_pair, \
              encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_tuple_payload_typed_record_idx \
             ON ec_spire_tuple_payload_typed_record_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_tuple_payload_typed_record_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_tuple_payload_typed_record_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let endpoint_args = format!(
            "'ec_spire_tuple_payload_typed_record_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict', ARRAY['id', 'label', 'pair']::text[]",
            selected_pids[0], selected_pids[1],
        );
        let typed_summary = Spi::get_one::<String>(&format!(
            "SELECT count(*)::text || '|' || \
                    min(payload_column_count)::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport = 'pg_binary_attr_v1')::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport_status = 'ready')::text || '|' || \
                    count(*) FILTER (WHERE status = 'ready')::text \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args})"
        ))
        .expect("typed tuple payload summary query should succeed")
        .expect("typed tuple payload summary should exist");
        let alpha_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args}) \
              WHERE payload_attnums = ARRAY[1, 2, 3]::int2[] \
                AND payload_names = ARRAY['id', 'label', 'pair']::text[] \
                AND payload_type_oids = ARRAY[\
                    'int8'::regtype::oid, \
                    'ec_spire_typed_label_domain'::regtype::oid, \
                    'ec_spire_typed_pair'::regtype::oid]::oid[] \
                AND payload_nulls = ARRAY[false, false, false]::boolean[] \
                AND payload_formats = ARRAY[\
                    'pg_binary_attr_v1', \
                    'pg_binary_attr_v1', \
                    'pg_binary_attr_v1']::text[] \
                AND payload_values[1] = int8send(1::bigint)::bytea \
                AND payload_values[2] = textsend(\
                    'alpha'::ec_spire_typed_label_domain::text)::bytea \
                AND payload_values[3] = record_send(\
                    ROW(7, 'left')::ec_spire_typed_pair)::bytea \
                AND NOT tuple_payload_missing \
                AND tuple_transport_status = 'ready' \
                AND status = 'ready'"
        ))
        .expect("typed tuple payload domain/composite query should succeed")
        .expect("typed tuple payload domain/composite count should exist");

        assert_eq!(typed_summary, "2|3|2|2|2");
        assert_eq!(alpha_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_insert_tuple_payload_endpoint_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_insert_payload_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_insert_payload_sql_idx \
             ON ec_spire_remote_insert_payload_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let insert_status = Spi::get_one::<String>(
            "SELECT status || ':' || inserted_count::text || ':' || payload_column_count::text \
               FROM ec_spire_remote_insert_tuple_payload(\
                    'ec_spire_remote_insert_payload_sql_idx'::regclass, \
                    jsonb_build_object(\
                        'id', 101, \
                        'title', 'remote payload', \
                        'embedding', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)::text), \
                    ARRAY['id', 'title', 'embedding']::text[])",
        )
        .expect("remote insert tuple payload status query should succeed")
        .expect("remote insert tuple payload status should exist");
        let inserted_row = Spi::get_one::<String>(
            "SELECT id::text || ':' || title \
               FROM ec_spire_remote_insert_payload_sql \
              WHERE id = 101",
        )
        .expect("inserted row query should succeed")
        .expect("inserted row should exist");

        assert_eq!(insert_status, "ready:1:3");
        assert_eq!(inserted_row, "101:remote payload");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_tuple_payload_missing_ctid_signal() {
        Spi::run(
            "CREATE TABLE ec_spire_tuple_payload_missing_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_tuple_payload_missing_sql (id, title, embedding) VALUES \
             (1, 'alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        let table_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_tuple_payload_missing_sql'::regclass::oid",
        )
        .expect("table oid query should succeed")
        .expect("table oid should exist");
        let heap_relation_regclass = ec_spire_relation_regclass_text(table_oid)
            .expect("heap relation regclass lookup should succeed");
        let requested_columns = vec!["id".to_owned(), "title".to_owned()];
        let missing_ctid = "(999,1)".to_owned();

        let payloads = ec_spire_remote_search_tuple_payloads_for_ctids(
            &heap_relation_regclass,
            &requested_columns,
            &[missing_ctid.clone(), missing_ctid],
        )
        .expect("tuple payload batch fetch should succeed");
        assert_eq!(payloads.len(), 2);
        for (tuple_payload, tuple_payload_missing) in payloads {
            assert!(tuple_payload_missing);
            assert!(tuple_payload
                .0
                .as_object()
                .expect("missing tuple payload should be a JSON object")
                .is_empty());
        }
    }

    #[pg_test]
    fn test_ec_spire_remote_search_local_heap_degraded_skip_status() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_local_heap_degraded_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_local_heap_degraded_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_local_heap_degraded_sql_idx \
             ON ec_spire_remote_local_heap_degraded_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_local_heap_degraded_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_local_heap_degraded_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_local_heap_degraded_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pids[1] as u64, "skipped");
        }
        let args = format!(
            "'ec_spire_remote_local_heap_degraded_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'degraded'",
            selected_pids[0], selected_pids[1],
        );
        let merge_from = format!("FROM ec_spire_remote_search_merge_input_summary({args})");
        let final_from = format!("FROM ec_spire_remote_search_finalization_summary({args})");
        let heap_from = format!("FROM ec_spire_remote_search_heap_resolution_summary({args})");
        let candidate_from = format!("FROM ec_spire_remote_search_local_heap_candidates({args})");
        let candidate_summary_from =
            format!("FROM ec_spire_remote_search_local_heap_candidate_summary({args})");
        let result_summary_from =
            format!("FROM ec_spire_remote_search_coordinator_result_summary({args})");

        let merge_status = Spi::get_one::<String>(&format!("SELECT status {merge_from}"))
            .expect("degraded merge status query should succeed")
            .expect("degraded merge status should exist");
        let skipped_batch_count =
            Spi::get_one::<i64>(&format!("SELECT skipped_batch_count {merge_from}"))
                .expect("degraded merge skipped query should succeed")
                .expect("degraded merge skipped count should exist");
        let local_batch_count =
            Spi::get_one::<i64>(&format!("SELECT local_batch_count {merge_from}"))
                .expect("degraded merge local query should succeed")
                .expect("degraded merge local count should exist");
        let final_status = Spi::get_one::<String>(&format!("SELECT status {final_from}"))
            .expect("degraded final status query should succeed")
            .expect("degraded final status should exist");
        let final_heap_fetch_status =
            Spi::get_one::<String>(&format!("SELECT final_heap_fetch_status {final_from}"))
                .expect("degraded final heap status query should succeed")
                .expect("degraded final heap status should exist");
        let heap_status = Spi::get_one::<String>(&format!("SELECT status {heap_from}"))
            .expect("degraded heap status query should succeed")
            .expect("degraded heap status should exist");
        let local_heap_resolution_status =
            Spi::get_one::<String>(&format!("SELECT local_heap_resolution_status {heap_from}"))
                .expect("degraded local heap resolution query should succeed")
                .expect("degraded local heap resolution status should exist");
        let decoded_local_locator_count =
            Spi::get_one::<i64>(&format!("SELECT decoded_local_locator_count {heap_from}"))
                .expect("degraded decoded locator query should succeed")
                .expect("degraded decoded locator count should exist");
        let candidate_count = Spi::get_one::<i64>(&format!("SELECT count(*) {candidate_from}"))
            .expect("degraded local candidate count query should succeed")
            .expect("degraded local candidate count should exist");
        let returned_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {candidate_summary_from}"
        ))
        .expect("degraded candidate summary count query should succeed")
        .expect("degraded candidate summary count should exist");
        let candidate_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {candidate_summary_from}"))
                .expect("degraded candidate summary status query should succeed")
                .expect("degraded candidate summary status should exist");
        let result_source =
            Spi::get_one::<String>(&format!("SELECT result_source {result_summary_from}"))
                .expect("degraded result source query should succeed")
                .expect("degraded result source should exist");
        let result_receive_count =
            Spi::get_one::<i64>(&format!("SELECT libpq_receive_count {result_summary_from}"))
                .expect("degraded result receive count query should succeed")
                .expect("degraded result receive count should exist");
        let result_receive_status = Spi::get_one::<String>(&format!(
            "SELECT libpq_receive_status {result_summary_from}"
        ))
        .expect("degraded result receive status query should succeed")
        .expect("degraded result receive status should exist");
        let result_status = Spi::get_one::<String>(&format!("SELECT status {result_summary_from}"))
            .expect("degraded result status query should succeed")
            .expect("degraded result status should exist");
        let result_skipped_pid_count =
            Spi::get_one::<i64>(&format!("SELECT skipped_pid_count {result_summary_from}"))
                .expect("degraded result skipped pid query should succeed")
                .expect("degraded result skipped pid count should exist");

        assert_eq!(merge_status, "degraded_ready");
        assert_eq!(skipped_batch_count, 1);
        assert_eq!(local_batch_count, 1);
        assert_eq!(final_status, "degraded_ready");
        assert_eq!(final_heap_fetch_status, "local_ready");
        assert_eq!(heap_status, "degraded_ready");
        assert_eq!(local_heap_resolution_status, "ready");
        assert_eq!(decoded_local_locator_count, 1);
        assert_eq!(candidate_count, 1);
        assert_eq!(returned_candidate_count, 1);
        assert_eq!(candidate_summary_status, "degraded_ready");
        assert_eq!(result_source, "local_heap_candidates");
        assert_eq!(result_receive_count, 0);
        assert_eq!(result_receive_status, "ready");
        assert_eq!(result_status, "degraded_ready");
        assert_eq!(result_skipped_pid_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_heap_resolution_summary_blocks_remote() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_heap_res_summary_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_heap_res_summary_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_heap_res_summary_sql_idx \
             ON ec_spire_remote_heap_res_summary_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_heap_res_summary_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_heap_res_summary_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_heap_res_summary_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let summary_from = format!(
            "FROM ec_spire_remote_search_heap_resolution_summary(\
             'ec_spire_remote_heap_res_summary_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let candidate_summary_from = format!(
            "FROM ec_spire_remote_search_local_heap_candidate_summary(\
             'ec_spire_remote_heap_res_summary_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let result_summary_from = format!(
            "FROM ec_spire_remote_search_coordinator_result_summary(\
             'ec_spire_remote_heap_res_summary_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("remote heap summary status query should succeed")
            .expect("remote heap summary status should exist");
        let remote_plan_count =
            Spi::get_one::<i64>(&format!("SELECT remote_plan_count {summary_from}"))
                .expect("remote heap summary remote plan query should succeed")
                .expect("remote heap summary remote plan count should exist");
        let remote_pid_count =
            Spi::get_one::<i64>(&format!("SELECT remote_pid_count {summary_from}"))
                .expect("remote heap summary remote pid query should succeed")
                .expect("remote heap summary remote pid count should exist");
        let decoded_local_locator_count = Spi::get_one::<i64>(&format!(
            "SELECT decoded_local_locator_count {summary_from}"
        ))
        .expect("remote heap summary decoded count query should succeed")
        .expect("remote heap summary decoded count should exist");
        let local_resolution_status = Spi::get_one::<String>(&format!(
            "SELECT local_heap_resolution_status {summary_from}"
        ))
        .expect("remote heap summary local status query should succeed")
        .expect("remote heap summary local status should exist");
        let remote_resolution_status = Spi::get_one::<String>(&format!(
            "SELECT remote_heap_resolution_status {summary_from}"
        ))
        .expect("remote heap summary remote status query should succeed")
        .expect("remote heap summary remote status should exist");
        let candidate_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {candidate_summary_from}"))
                .expect("remote heap candidate summary status query should succeed")
                .expect("remote heap candidate summary status should exist");
        let returned_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {candidate_summary_from}"
        ))
        .expect("remote heap candidate summary return query should succeed")
        .expect("remote heap candidate summary return count should exist");
        let result_source =
            Spi::get_one::<String>(&format!("SELECT result_source {result_summary_from}"))
                .expect("remote result source query should succeed")
                .expect("remote result source should exist");
        let result_receive_count =
            Spi::get_one::<i64>(&format!("SELECT libpq_receive_count {result_summary_from}"))
                .expect("remote result receive count query should succeed")
                .expect("remote result receive count should exist");
        let result_receive_status = Spi::get_one::<String>(&format!(
            "SELECT libpq_receive_status {result_summary_from}"
        ))
        .expect("remote result receive status query should succeed")
        .expect("remote result receive status should exist");
        let result_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {result_summary_from}"))
                .expect("remote result blocker query should succeed")
                .expect("remote result blocker should exist");
        let result_returned_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {result_summary_from}"
        ))
        .expect("remote result returned count query should succeed")
        .expect("remote result returned count should exist");

        assert_eq!(status, "requires_remote_node_descriptor");
        assert_eq!(remote_plan_count, 1);
        assert_eq!(remote_pid_count, 1);
        assert_eq!(decoded_local_locator_count, 0);
        assert_eq!(local_resolution_status, "planned");
        assert_eq!(remote_resolution_status, "requires_remote_node_descriptor");
        assert_eq!(candidate_summary_status, "requires_remote_node_descriptor");
        assert_eq!(returned_candidate_count, 0);
        assert_eq!(result_source, "blocked");
        assert_eq!(result_receive_count, 1);
        assert_eq!(result_receive_status, "requires_remote_node_descriptor");
        assert_eq!(result_next_blocker, "remote_node_descriptor");
        assert_eq!(result_returned_candidate_count, 0);
    }

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

    #[pg_test]
    fn test_ec_spire_production_transport_probe_overlaps_ready_remotes() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo.clone(),
                sql: "SELECT pg_sleep(0.30)",
            },
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 3,
                conninfo: loopback_conninfo,
                sql: "SELECT 1",
            },
        ]);
        let slow = rows
            .iter()
            .find(|row| row.node_id == 2)
            .expect("slow row should exist");
        let fast = rows
            .iter()
            .find(|row| row.node_id == 3)
            .expect("fast row should exist");

        assert_eq!(slow.status, "ready");
        assert_eq!(fast.status, "ready");
        assert!(
            fast.completed_after_ms < slow.completed_after_ms,
            "fast remote should complete before slow remote when transport overlaps work: \
             fast={}ms slow={}ms",
            fast.completed_after_ms,
            slow.completed_after_ms
        );
        assert!(
            slow.elapsed_ms >= 250,
            "slow probe should actually exercise delayed remote work: {}ms",
            slow.elapsed_ms
        );
    }

    #[pg_test]
    fn test_ec_spire_production_transport_probe_isolates_node_failure() {
        Spi::run("SET LOCAL ec_spire.remote_search_connect_timeout_ms = 25")
            .expect("connect timeout SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: "host=/tmp/ecaz_missing_pg_socket_30725 dbname=postgres user=postgres"
                    .to_owned(),
                sql: "SELECT 1",
            },
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 3,
                conninfo: loopback_conninfo,
                sql: "SELECT 1",
            },
        ]);
        let failed = rows
            .iter()
            .find(|row| row.node_id == 2)
            .expect("failed row should exist");
        let ready = rows
            .iter()
            .find(|row| row.node_id == 3)
            .expect("ready row should exist");

        assert_eq!(rows.len(), 2);
        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "connect_failed");
        assert_eq!(ready.status, "ready");
        assert_eq!(ready.failure_category, "none");
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_remote_stmt_timeout() {
        Spi::run("SET LOCAL ec_spire.remote_search_statement_timeout_ms = 25")
            .expect("statement timeout SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_sleep(0.30)",
            },
        ]);
        let failed = rows.first().expect("timeout row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "remote_statement_timeout");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_backend_terminated() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_terminate_backend(pg_backend_pid())",
            },
        ]);
        let failed = rows.first().expect("termination row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "remote_backend_terminated");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_remote_query_cancelled() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_cancel_backend(pg_backend_pid()), pg_sleep(0.30)",
            },
        ]);
        let failed = rows.first().expect("cancel row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "remote_query_cancelled");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_governance_overload() {
        set_remote_governance_test_namespace(6601);
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches = 1")
            .expect("global governance cap SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let (class_id, object_id) =
            am::remote_search_libpq_global_governance_advisory_key_for_test(0);
        let mut lock_holder = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback lock-holder connection should succeed");
        lock_holder
            .batch_execute(&format!("SELECT pg_advisory_lock({class_id}, {object_id})"))
            .expect("global governance advisory lock should be held by separate backend");

        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: "invalid_conninfo_before_transport_open".to_owned(),
                sql: "SELECT 1",
            },
        ]);
        let failed = rows.first().expect("governance overload row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "remote_executor_overload");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_local_cancel_remote_cancel() {
        set_remote_governance_test_namespace(6602);
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches = 1")
            .expect("global governance cap SET should succeed");
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches_per_node = 1")
            .expect("per-node governance cap SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let (global_class_id, global_object_id) =
            am::remote_search_libpq_global_governance_advisory_key_for_test(0);
        let (node_class_id, node_object_id) =
            am::remote_search_libpq_node_governance_advisory_key_for_test(2, 0);
        let rows = am::spire_remote_search_production_transport_probe_with_local_cancel_for_test(
            vec![am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_sleep(0.30)",
            }],
            25,
        );
        let failed = rows.first().expect("local cancel row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "local_query_cancelled");
        assert_eq!(failed.row_count, 0);
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        assert_governance_lock_released(
            &loopback_conninfo,
            global_class_id,
            global_object_id,
            "global transport local-cancel",
        );
        assert_governance_lock_released(
            &loopback_conninfo,
            node_class_id,
            node_object_id,
            "per-node transport local-cancel",
        );
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_pg_interrupt_bridge_cancel() {
        let _interrupt_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _flags = unsafe { ScopedPgQueryCancelFlags::set_pending() }
            .expect("PostgreSQL query-cancel flags should resolve inside pg_test backend");
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_sleep(0.30)",
            },
        ]);
        let failed = rows.first().expect("pg interrupt cancel row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "local_query_cancelled");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_pg_statement_timeout_bridge_cancel() {
        let _interrupt_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let timeout_signal = unsafe { ScopedPgStatementTimeoutSignal::trigger_after_ms(1) }
            .expect("PostgreSQL timeout symbols should resolve inside pg_test backend");
        assert!(timeout_signal.statement_timeout_pending());

        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_sleep(0.30)",
            },
        ]);
        let failed = rows
            .first()
            .expect("pg statement-timeout cancel row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "local_statement_timeout");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_production_candidate_receive_loopback() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_production_candidate_receive_remote_sql; \
                 CREATE TABLE ec_spire_production_candidate_receive_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_production_candidate_receive_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_production_candidate_receive_remote_idx \
                     ON ec_spire_production_candidate_receive_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq')",
            )
            .expect("loopback candidate receive fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_production_candidate_receive_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_production_candidate_receive_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let selected_pid = u64::try_from(selected_pid).expect("leaf pid should fit u64");
        let requested_epoch = u64::try_from(active_epoch).expect("active epoch should fit u64");
        let remote_index_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_production_candidate_receive_remote_idx",
        );

        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                remote_index_regclass: "ec_spire_production_candidate_receive_remote_idx"
                    .to_owned(),
                remote_index_identity,
                requested_epoch,
                query: vec![1.0, 0.0],
                selected_pids: vec![selected_pid],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let receive = rows.first().expect("receive row should exist");
        let batch = receive
            .batch
            .as_ref()
            .expect("candidate batch should exist");
        let candidate = batch
            .candidates
            .first()
            .expect("candidate row should exist");

        assert_eq!(receive.status, "ready");
        assert_eq!(receive.failure_category, "none");
        assert_eq!(receive.candidate_count, 1);
        assert_eq!(batch.node_id, 2);
        assert_eq!(batch.selected_pids, vec![selected_pid]);
        assert_eq!(candidate.node_id, 2);
        assert_eq!(candidate.served_epoch, requested_epoch);
        assert_eq!(candidate.pid, selected_pid);
        assert!(!candidate.row_locator.is_empty());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_top_k_zero() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_top_k_zero_sql; \
                 CREATE TABLE ec_spire_candidate_receive_top_k_zero_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_top_k_zero_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_top_k_zero_idx \
                     ON ec_spire_candidate_receive_top_k_zero_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq')",
            )
            .expect("loopback top-k-zero fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_top_k_zero_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_top_k_zero_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let remote_index_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_candidate_receive_top_k_zero_idx",
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_top_k_zero_idx".to_owned(),
                remote_index_identity,
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 0,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let receive = rows.first().expect("top-k-zero row should exist");
        let batch = receive
            .batch
            .as_ref()
            .expect("top-k-zero ready batch should exist");

        assert_eq!(receive.status, "ready");
        assert_eq!(receive.failure_category, "none");
        assert_eq!(receive.candidate_count, 0);
        assert!(batch.candidates.is_empty());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_remote_stmt_timeout() {
        Spi::run("SET LOCAL ec_spire.remote_search_statement_timeout_ms = 25")
            .expect("statement timeout SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_timeout_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_timeout CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_timeout_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_timeout_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_timeout_remote_idx \
                     ON ec_spire_candidate_receive_timeout_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_timeout; \
                 CREATE FUNCTION ec_spire_candidate_receive_timeout.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'aa', 'ready' \
                     FROM pg_sleep(0.30) \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback timeout fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_timeout_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_timeout_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let timeout_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_timeout,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: timeout_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_timeout_remote_idx".to_owned(),
                remote_index_identity: vec![0xaa],
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("timeout row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "remote_statement_timeout");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_remote_query_cancelled() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_cancel_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_cancel CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_cancel_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_cancel_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_cancel_remote_idx \
                     ON ec_spire_candidate_receive_cancel_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_cancel; \
                 CREATE FUNCTION ec_spire_candidate_receive_cancel.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'aa', 'ready' \
                     FROM pg_cancel_backend(pg_backend_pid()), pg_sleep(0.30) \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback cancel fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_cancel_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_cancel_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let cancel_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_cancel,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: cancel_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_cancel_remote_idx".to_owned(),
                remote_index_identity: vec![0xaa],
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("cancel row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "remote_query_cancelled");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_local_cancel_remote_cancel() {
        set_remote_governance_test_namespace(6603);
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches = 1")
            .expect("global governance cap SET should succeed");
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches_per_node = 1")
            .expect("per-node governance cap SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let (global_class_id, global_object_id) =
            am::remote_search_libpq_global_governance_advisory_key_for_test(0);
        let (node_class_id, node_object_id) =
            am::remote_search_libpq_node_governance_advisory_key_for_test(2, 0);
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_local_cancel_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_local_cancel CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_local_cancel_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_local_cancel_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_local_cancel_remote_idx \
                     ON ec_spire_candidate_receive_local_cancel_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_local_cancel; \
                 CREATE FUNCTION ec_spire_candidate_receive_local_cancel.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'aa', 'ready' \
                     FROM pg_sleep(0.30) \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback local cancel fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_local_cancel_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_local_cancel_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let local_cancel_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_local_cancel,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_with_local_cancel_for_test(
            vec![am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: local_cancel_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_local_cancel_remote_idx"
                    .to_owned(),
                remote_index_identity: vec![0xaa],
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            }],
            25,
        );
        let failed = rows.first().expect("local cancel row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "local_query_cancelled");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
        assert_governance_lock_released(
            &loopback_conninfo,
            global_class_id,
            global_object_id,
            "global receive local-cancel",
        );
        assert_governance_lock_released(
            &loopback_conninfo,
            node_class_id,
            node_object_id,
            "per-node receive local-cancel",
        );
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_governance_overload() {
        set_remote_governance_test_namespace(6604);
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches = 1")
            .expect("global governance cap SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let (class_id, object_id) =
            am::remote_search_libpq_global_governance_advisory_key_for_test(0);
        let mut lock_holder = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback lock-holder connection should succeed");
        lock_holder
            .batch_execute(&format!("SELECT pg_advisory_lock({class_id}, {object_id})"))
            .expect("global governance advisory lock should be held by separate backend");

        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: "invalid_conninfo_before_candidate_receive_open".to_owned(),
                remote_index_regclass: "ec_spire_missing_remote_idx".to_owned(),
                remote_index_identity: vec![0xaa],
                requested_epoch: 1,
                query: vec![1.0, 0.0],
                selected_pids: vec![1],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("governance overload row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "remote_executor_overload");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_identity_mismatch() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_identity_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_identity CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_identity_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_identity_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_identity_remote_idx \
                     ON ec_spire_candidate_receive_identity_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_identity; \
                 CREATE FUNCTION ec_spire_candidate_receive_identity.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'bb', 'ready' \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback identity mismatch fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_identity_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_identity_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let identity_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_identity,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: identity_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_identity_remote_idx".to_owned(),
                remote_index_identity: vec![0xaa],
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("identity mismatch row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "endpoint_identity_mismatch");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_stale_epoch() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_stale_epoch_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_stale_epoch CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_stale_epoch_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_stale_epoch_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_stale_epoch_idx \
                     ON ec_spire_candidate_receive_stale_epoch_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_stale_epoch; \
                 CREATE FUNCTION ec_spire_candidate_receive_stale_epoch.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2 - 1, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'aa', 'ready' \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback stale epoch fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_stale_epoch_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_stale_epoch_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let stale_epoch_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_stale_epoch,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: stale_epoch_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_stale_epoch_idx".to_owned(),
                remote_index_identity: vec![0xaa],
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("stale epoch row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "served_epoch_mismatch");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_backend_terminated() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_terminate_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_terminate CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_terminate_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_terminate_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_terminate_remote_idx \
                     ON ec_spire_candidate_receive_terminate_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_terminate; \
                 CREATE FUNCTION ec_spire_candidate_receive_terminate.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'aa', 'ready' \
                     FROM pg_terminate_backend(pg_backend_pid()) \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback terminate fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_terminate_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_terminate_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let terminate_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_terminate,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: terminate_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_terminate_remote_idx".to_owned(),
                remote_index_identity: vec![0xaa],
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("termination row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "remote_backend_terminated");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_isolates_node_failures() {
        Spi::run("SET LOCAL ec_spire.remote_search_connect_timeout_ms = 25")
            .expect("connect timeout SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_production_candidate_receive_ready_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_decode_fail CASCADE; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_validation_fail CASCADE; \
                 CREATE TABLE ec_spire_production_candidate_receive_ready_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_production_candidate_receive_ready_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_production_candidate_receive_ready_idx \
                     ON ec_spire_production_candidate_receive_ready_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_decode_fail; \
                 CREATE FUNCTION ec_spire_candidate_receive_decode_fail.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score text, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 1::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 'not-a-real'::text, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'aa', 'ready' \
                 $function$; \
                 CREATE SCHEMA ec_spire_candidate_receive_validation_fail; \
                 CREATE FUNCTION ec_spire_candidate_receive_validation_fail.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 999::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'aa', 'ready' \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback multi-node candidate receive fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_production_candidate_receive_ready_idx'::regclass)",
                &[],
            )
            .expect("ready active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("ready active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_production_candidate_receive_ready_idx'::regclass)",
                &[],
            )
            .expect("ready leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("ready leaf pid should decode");
        let selected_pid = u64::try_from(selected_pid).expect("ready leaf pid should fit u64");
        let requested_epoch = u64::try_from(active_epoch).expect("ready epoch should fit u64");
        let ready_remote_index_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_production_candidate_receive_ready_idx",
        );
        let decode_fail_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_decode_fail,public'"
        );
        let validation_fail_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_validation_fail,public'"
        );

        let request = |node_id: u32,
                       conninfo: String,
                       remote_index_regclass: &str,
                       remote_index_identity: Vec<u8>,
                       requested_epoch: u64,
                       query: Vec<f32>,
                       selected_pids: Vec<u64>,
                       consistency_mode: &str| {
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id,
                conninfo,
                remote_index_regclass: remote_index_regclass.to_owned(),
                remote_index_identity,
                requested_epoch,
                query,
                selected_pids,
                top_k: 1,
                consistency_mode: consistency_mode.to_owned(),
            }
        };
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            request(
                2,
                loopback_conninfo.clone(),
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                3,
                loopback_conninfo.clone(),
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0],
                vec![u64::MAX],
                "strict",
            ),
            request(
                4,
                "port=not-a-number dbname=postgres".to_owned(),
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                5,
                "host=/tmp/ecaz_missing_pg_socket_30729 port=6543 dbname=postgres user=postgres connect_timeout=1"
                    .to_owned(),
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                6,
                loopback_conninfo.clone(),
                "ec_spire_missing_candidate_receive_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                7,
                loopback_conninfo.clone(),
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity,
                requested_epoch,
                vec![1.0, 0.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                8,
                decode_fail_conninfo,
                "ec_spire_production_candidate_receive_ready_idx",
                vec![0xaa],
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                9,
                validation_fail_conninfo,
                "ec_spire_production_candidate_receive_ready_idx",
                vec![0xaa],
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
        ]);
        let ready = rows
            .iter()
            .find(|row| row.node_id == 2)
            .expect("ready row should exist");
        let ready_batch = ready.batch.as_ref().expect("ready batch should exist");
        let expected_failures = [
            (3, "candidate_invalid_parameters"),
            (4, "conninfo_parse_failed"),
            (5, "connect_failed"),
            (6, "remote_index_unavailable"),
            (7, "remote_query_failed"),
            (8, "candidate_decode_failed"),
            (9, "candidate_batch_validation_failed"),
        ];

        assert_eq!(rows.len(), 8);
        assert_eq!(ready.status, "ready");
        assert_eq!(ready.failure_category, "none");
        assert_eq!(ready.candidate_count, 1);
        assert_eq!(ready_batch.node_id, 2);
        assert_eq!(ready_batch.selected_pids, vec![selected_pid]);
        assert_eq!(ready_batch.candidates.len(), 1);
        assert!(ready_batch
            .candidates
            .iter()
            .all(|candidate| candidate.node_id == 2));
        for (node_id, failure_category) in expected_failures {
            let failed = rows
                .iter()
                .find(|row| row.node_id == node_id)
                .expect("failed row should exist");
            assert_eq!(failed.status, "remote_candidate_receive_failed");
            assert_eq!(failed.failure_category, failure_category);
            assert_eq!(failed.candidate_count, 0);
            assert!(
                failed.batch.is_none(),
                "failed node {node_id} should not return a candidate batch"
            );
        }
    }

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
        let index_relation = unsafe {
            open_valid_ec_spire_index(
                index_oid,
                "test_ec_spire_libpq_identity_cache_contract_probe",
            )
        };
        let (
            identity_cache_probe_entries,
            identity_cache_probe_queries,
            identity_cache_probe_hits,
            identity_cache_probe_misses,
            identity_cache_probe_mismatch_status,
        ) = unsafe {
            am::spire_remote_search_libpq_identity_cache_contract_probe_counts(
                index_relation,
                u64::try_from(active_epoch).expect("active epoch should fit u64"),
                vec![1.0, 0.0],
                vec![u64::try_from(selected_pid).expect("selected PID should fit u64")],
                1,
                "strict",
            )
        };
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

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

    #[pg_test]
    fn test_ec_spire_remote_node_descriptor_stale_generation_rejected() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_node_desc_stale_gen_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_node_desc_stale_gen_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_node_desc_stale_gen_sql_idx \
             ON ec_spire_remote_node_desc_stale_gen_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 1)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_node_desc_stale_gen_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_node_desc_stale_gen_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");

        let first_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 7, 'spire/remote/stale-generation', decode('02', 'hex'), \
                     'remote_spire_idx', 'active', {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("first descriptor registration should succeed")
        .expect("first descriptor registration result should exist");
        assert!(first_result);

        let stale_generation_error = pg_sys::PgTryBuilder::new(|| {
            let _ = Spi::get_one::<bool>(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                         '{}'::oid, 2, 7, 'spire/remote/stale-generation', decode('02', 'hex'), \
                         'remote_spire_idx', 'active', {active_epoch}, {active_epoch}, '{}', 'none')",
                u32::from(index_oid),
                env!("CARGO_PKG_VERSION")
            ));
            "no_error".to_owned()
        })
        .catch_when(
            pg_sys::errcodes::PgSqlErrorCode::ERRCODE_T_R_SERIALIZATION_FAILURE,
            |cause| match cause {
                pg_sys::panic::CaughtError::ErrorReport(report)
                | pg_sys::panic::CaughtError::PostgresError(report) => {
                    format!(
                        "{}|{}",
                        report.message(),
                        report.detail().unwrap_or("")
                    )
                }
                pg_sys::panic::CaughtError::RustPanic { ereport, .. } => {
                    format!(
                        "{}|{}",
                        ereport.message(),
                        ereport.detail().unwrap_or("")
                    )
                }
            },
        )
        .catch_others(|cause| cause.rethrow())
        .execute();

        assert_eq!(
            stale_generation_error,
            "ec_spire_register_remote_node_descriptor descriptor_generation must advance existing descriptor_generation|Retry the whole coordinator write after the winning descriptor refresh commits."
        );
    }

    #[pg_test]
    fn test_ec_spire_remote_node_desc_failed_blocks_libpq_dispatch() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_node_desc_failed_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_node_desc_failed_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_node_desc_failed_sql_idx \
             ON ec_spire_remote_node_desc_failed_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_node_desc_failed_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_node_desc_failed_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_node_desc_failed_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 9, 'spire/remote/failed', decode('03', 'hex'), \
                     'remote_spire_idx', 'failed', {active_epoch}, {active_epoch}, '{}', 'network_error')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("failed descriptor registration should succeed")
        .expect("failed descriptor registration result should exist");

        let readiness_from = format!(
            "FROM ec_spire_remote_search_target_readiness(\
             'ec_spire_remote_node_desc_failed_sql_idx'::regclass, \
             {active_epoch}, ARRAY[{selected_pid}], 'strict')"
        );
        let connection_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_plan(\
             'ec_spire_remote_node_desc_failed_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let connection_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_connection_summary(\
             'ec_spire_remote_node_desc_failed_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let dispatch_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_plan(\
             'ec_spire_remote_node_desc_failed_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );
        let dispatch_summary_from = format!(
            "FROM ec_spire_remote_search_libpq_dispatch_summary(\
             'ec_spire_remote_node_desc_failed_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{selected_pid}]::bigint[], 3, 'strict')"
        );

        let descriptor_state =
            Spi::get_one::<String>(&format!("SELECT descriptor_state {readiness_from}"))
                .expect("failed readiness descriptor query should succeed")
                .expect("failed descriptor state should exist");
        let node_status = Spi::get_one::<String>(&format!("SELECT node_status {readiness_from}"))
            .expect("failed readiness node status query should succeed")
            .expect("failed node status should exist");
        let target_status = Spi::get_one::<String>(&format!("SELECT status {readiness_from}"))
            .expect("failed readiness status query should succeed")
            .expect("failed readiness status should exist");
        let conninfo_secret_name =
            Spi::get_one::<String>(&format!("SELECT conninfo_secret_name {connection_from}"))
                .expect("failed connection secret query should succeed")
                .expect("failed connection secret should exist");
        let conninfo_resolution =
            Spi::get_one::<String>(&format!("SELECT conninfo_resolution {connection_from}"))
                .expect("failed connection resolution query should succeed")
                .expect("failed connection resolution should exist");
        let pipeline_mode =
            Spi::get_one::<String>(&format!("SELECT pipeline_mode {connection_from}"))
                .expect("failed connection pipeline query should succeed")
                .expect("failed connection pipeline should exist");
        let connection_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {connection_summary_from}"))
                .expect("failed connection summary status query should succeed")
                .expect("failed connection summary status should exist");
        let missing_descriptor_connection_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_connection_count {connection_summary_from}"
        ))
        .expect("failed connection summary missing query should succeed")
        .expect("failed connection summary missing count should exist");
        let dispatch_action =
            Spi::get_one::<String>(&format!("SELECT dispatch_action {dispatch_from}"))
                .expect("failed dispatch action query should succeed")
                .expect("failed dispatch action should exist");
        let dispatch_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {dispatch_summary_from}"))
                .expect("failed dispatch summary status query should succeed")
                .expect("failed dispatch summary status should exist");
        let missing_descriptor_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_dispatch_count {dispatch_summary_from}"
        ))
        .expect("failed dispatch summary missing query should succeed")
        .expect("failed dispatch summary missing count should exist");

        assert!(register_result);
        assert_eq!(descriptor_state, "failed");
        assert_eq!(node_status, "failed_remote_node");
        assert_eq!(target_status, "requires_remote_node_descriptor");
        assert_eq!(conninfo_secret_name, "none");
        assert_eq!(conninfo_resolution, "requires_remote_node_descriptor");
        assert_eq!(pipeline_mode, "none");
        assert_eq!(connection_summary_status, "requires_remote_node_descriptor");
        assert_eq!(missing_descriptor_connection_count, 1);
        assert_eq!(dispatch_action, "blocked_before_dispatch");
        assert_eq!(dispatch_summary_status, "requires_remote_node_descriptor");
        assert_eq!(missing_descriptor_dispatch_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_node_descriptor_contract() {
        let contract_from = "FROM ec_spire_remote_node_descriptor_contract()";
        let field_count = Spi::get_one::<i64>(&format!("SELECT count(*) {contract_from}"))
            .expect("descriptor contract count query should succeed")
            .expect("descriptor contract count should exist");
        let secret_role = Spi::get_one::<String>(&format!(
            "SELECT semantic_role {contract_from} \
             WHERE field_name = 'conninfo_secret_name'"
        ))
        .expect("descriptor secret role query should succeed")
        .expect("descriptor secret role should exist");
        let secret_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {contract_from} \
             WHERE field_name = 'conninfo_secret_name'"
        ))
        .expect("descriptor secret validator query should succeed")
        .expect("descriptor secret validator should exist");
        let raw_conninfo_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {contract_from} \
             WHERE field_name = 'conninfo' OR semantic_role = 'raw_connection_string'"
        ))
        .expect("descriptor raw conninfo query should succeed")
        .expect("descriptor raw conninfo count should exist");
        let required_epoch_fields = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {contract_from} \
             WHERE required AND field_name IN ('last_served_epoch', 'min_retained_epoch')"
        ))
        .expect("descriptor epoch field query should succeed")
        .expect("descriptor epoch field count should exist");
        let shape_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {contract_from} \
             WHERE field_name = 'coordinator_insert_shape_fingerprint'"
        ))
        .expect("descriptor shape validator query should succeed")
        .expect("descriptor shape validator should exist");
        let remote_shape_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {contract_from} \
             WHERE field_name = 'remote_insert_shape_fingerprint'"
        ))
        .expect("descriptor remote shape validator query should succeed")
        .expect("descriptor remote shape validator should exist");

        assert_eq!(field_count, 14);
        assert_eq!(secret_role, "indirect_connection_secret");
        assert_eq!(
            secret_validator,
            "must_be_nonempty_noncolliding_secret_reference"
        );
        assert_eq!(shape_validator, "must_match_current_coordinator_heap_shape");
        assert_eq!(
            remote_shape_validator,
            "must_match_current_remote_heap_shape"
        );
        assert_eq!(raw_conninfo_count, 0);
        assert_eq!(required_epoch_fields, 2);
    }

    #[pg_test]
    fn test_ec_spire_remote_node_descriptor_state_contract() {
        let contract_from = "FROM ec_spire_remote_node_descriptor_state_contract()";
        let state_count = Spi::get_one::<i64>(&format!("SELECT count(*) {contract_from}"))
            .expect("descriptor state contract count query should succeed")
            .expect("descriptor state contract count should exist");
        let catalog_state_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {contract_from} WHERE state_source = 'catalog'"
        ))
        .expect("catalog state count query should succeed")
        .expect("catalog state count should exist");
        let active_read_eligible = Spi::get_one::<bool>(&format!(
            "SELECT read_eligible {contract_from} WHERE descriptor_state = 'active'"
        ))
        .expect("active state query should succeed")
        .expect("active state should exist");
        let draining_read_eligible = Spi::get_one::<bool>(&format!(
            "SELECT read_eligible {contract_from} WHERE descriptor_state = 'draining'"
        ))
        .expect("draining state query should succeed")
        .expect("draining state should exist");
        let disabled_read_eligible = Spi::get_one::<bool>(&format!(
            "SELECT read_eligible {contract_from} WHERE descriptor_state = 'disabled'"
        ))
        .expect("disabled state query should succeed")
        .expect("disabled state should exist");
        let failed_status = Spi::get_one::<String>(&format!(
            "SELECT snapshot_status {contract_from} WHERE descriptor_state = 'failed'"
        ))
        .expect("failed state query should succeed")
        .expect("failed state should exist");
        let missing_source = Spi::get_one::<String>(&format!(
            "SELECT state_source {contract_from} WHERE descriptor_state = 'missing'"
        ))
        .expect("missing state query should succeed")
        .expect("missing state should exist");
        let descriptor_state_check_count = Spi::get_one::<i64>(
            "SELECT count(*) \
               FROM pg_constraint c \
              WHERE c.conrelid = 'ec_spire_remote_node_descriptor'::regclass \
                AND c.contype = 'c' \
                AND pg_get_constraintdef(c.oid) LIKE '%descriptor_state%'",
        )
        .expect("descriptor state check count query should succeed")
        .expect("descriptor state check count should exist");
        let catalog_state_check_miss_count = Spi::get_one::<i64>(
            "WITH descriptor_check AS ( \
                 SELECT pg_get_constraintdef(c.oid) AS constraint_def \
                   FROM pg_constraint c \
                  WHERE c.conrelid = 'ec_spire_remote_node_descriptor'::regclass \
                    AND c.contype = 'c' \
                    AND pg_get_constraintdef(c.oid) LIKE '%descriptor_state%' \
                  LIMIT 1 \
             ) \
             SELECT count(*) \
               FROM ec_spire_remote_node_descriptor_state_contract() states \
               JOIN descriptor_check ON true \
              WHERE states.state_source = 'catalog' \
                AND position(quote_literal(states.descriptor_state) in descriptor_check.constraint_def) = 0",
        )
        .expect("descriptor state check invariant query should succeed")
        .expect("descriptor state check invariant count should exist");
        let synthetic_state_check_present = Spi::get_one::<bool>(
            "WITH descriptor_check AS ( \
                 SELECT pg_get_constraintdef(c.oid) AS constraint_def \
                   FROM pg_constraint c \
                  WHERE c.conrelid = 'ec_spire_remote_node_descriptor'::regclass \
                    AND c.contype = 'c' \
                    AND pg_get_constraintdef(c.oid) LIKE '%descriptor_state%' \
                  LIMIT 1 \
             ) \
             SELECT position(quote_literal('missing') in constraint_def) > 0 \
               FROM descriptor_check",
        )
        .expect("descriptor synthetic state check query should succeed")
        .expect("descriptor synthetic state check should exist");

        assert_eq!(state_count, 5);
        assert_eq!(catalog_state_count, 4);
        assert!(active_read_eligible);
        assert!(draining_read_eligible);
        assert!(!disabled_read_eligible);
        assert_eq!(failed_status, "failed_remote_node");
        assert_eq!(missing_source, "synthetic");
        assert_eq!(descriptor_state_check_count, 1);
        assert_eq!(catalog_state_check_miss_count, 0);
        assert!(!synthetic_state_check_present);
    }

    #[pg_test]
    fn test_ec_spire_remote_state_upgrade_check_matches_bootstrap() {
        fn descriptor_state_check_values(sql: &str) -> Vec<String> {
            let marker = "descriptor_state text NOT NULL CHECK (\n        descriptor_state IN (";
            let start = sql
                .find(marker)
                .expect("descriptor_state CHECK marker should exist")
                + marker.len();
            let tail = &sql[start..];
            let end = tail
                .find(')')
                .expect("descriptor_state CHECK list should close");
            tail[..end]
                .split(',')
                .map(|state| state.trim().trim_matches('\'').to_owned())
                .collect()
        }

        let bootstrap_states =
            descriptor_state_check_values(include_str!("../../sql/bootstrap.sql"));
        let upgrade_states =
            descriptor_state_check_values(include_str!("../../ecaz--0.1.0--0.1.1.sql"));
        let catalog_states = Spi::connect(|client| {
            client
                .select(
                    "SELECT descriptor_state \
                       FROM ec_spire_remote_node_descriptor_state_contract() \
                      WHERE state_source = 'catalog' \
                      ORDER BY descriptor_state",
                    None,
                    &[],
                )
                .expect("catalog state contract query should succeed")
                .map(|row| {
                    row["descriptor_state"]
                        .value::<String>()
                        .expect("catalog state decode should succeed")
                        .expect("catalog state should exist")
                })
                .collect::<Vec<_>>()
        });
        let mut bootstrap_states_sorted = bootstrap_states;
        bootstrap_states_sorted.sort();
        let mut upgrade_states_sorted = upgrade_states;
        upgrade_states_sorted.sort();

        assert_eq!(bootstrap_states_sorted, catalog_states);
        assert_eq!(upgrade_states_sorted, catalog_states);
    }

    #[pg_test]
    #[should_panic(
        expected = "conninfo_secret_name maps to provider_lookup_key EC_SPIRE_REMOTE_CONNINFO_NODE_1"
    )]
    fn test_ec_spire_remote_secret_key_collision_rejected() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_node_desc_secret_collision_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_node_desc_secret_collision_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_node_desc_secret_collision_sql_idx \
             ON ec_spire_remote_node_desc_secret_collision_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 1)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_node_desc_secret_collision_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_node_desc_secret_collision_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");

        let first_registered = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 1, 'node-1', decode('01', 'hex'), \
                     'remote_spire_idx_a', 'active', {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("first descriptor registration should succeed")
        .expect("first descriptor registration result should exist");
        assert!(first_registered);

        Spi::run(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 3, 1, 'node_1', decode('02', 'hex'), \
                     'remote_spire_idx_b', 'active', {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("colliding descriptor registration should fail before this point");
    }

    #[pg_test]
    fn test_ec_spire_remote_node_descriptor_registration_contract() {
        let contract_from = "FROM ec_spire_remote_node_descriptor_registration_contract()";
        let step_count = Spi::get_one::<i64>(&format!("SELECT count(*) {contract_from}"))
            .expect("descriptor registration contract count query should succeed")
            .expect("descriptor registration contract count should exist");
        let secret_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {contract_from} \
             WHERE input_field = 'conninfo_secret_name'"
        ))
        .expect("descriptor registration secret validator query should succeed")
        .expect("descriptor registration secret validator should exist");
        let secret_action = Spi::get_one::<String>(&format!(
            "SELECT persistence_action {contract_from} \
             WHERE input_field = 'conninfo_secret_name'"
        ))
        .expect("descriptor registration secret action query should succeed")
        .expect("descriptor registration secret action should exist");
        let generation_action = Spi::get_one::<String>(&format!(
            "SELECT persistence_action {contract_from} \
             WHERE input_field = 'generation'"
        ))
        .expect("descriptor registration generation action query should succeed")
        .expect("descriptor registration generation action should exist");
        let epoch_failure = Spi::get_one::<String>(&format!(
            "SELECT failure_status {contract_from} \
             WHERE input_field = 'last_served_epoch,min_retained_epoch'"
        ))
        .expect("descriptor registration epoch failure query should succeed")
        .expect("descriptor registration epoch failure should exist");
        let raw_conninfo_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {contract_from} \
             WHERE input_field = 'conninfo' OR semantic_role = 'raw_connection_string'"
        ))
        .expect("descriptor registration raw conninfo query should succeed")
        .expect("descriptor registration raw conninfo count should exist");
        let prepared_capacity_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {contract_from} \
             WHERE semantic_role = 'remote_prepared_transaction_capacity'"
        ))
        .expect("descriptor registration prepared capacity validator query should succeed")
        .expect("descriptor registration prepared capacity validator should exist");
        let prepared_capacity_action = Spi::get_one::<String>(&format!(
            "SELECT persistence_action {contract_from} \
             WHERE semantic_role = 'remote_prepared_transaction_capacity'"
        ))
        .expect("descriptor registration prepared capacity action query should succeed")
        .expect("descriptor registration prepared capacity action should exist");
        let remote_shape_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {contract_from} \
             WHERE semantic_role = 'remote_insert_schema_shape'"
        ))
        .expect("descriptor registration remote shape validator query should succeed")
        .expect("descriptor registration remote shape validator should exist");

        assert_eq!(step_count, 12);
        assert_eq!(
            secret_validator,
            "must_be_nonempty_noncolliding_secret_reference"
        );
        assert_eq!(secret_action, "persist_secret_reference_only");
        assert_eq!(generation_action, "atomically_replace_descriptor");
        assert_eq!(epoch_failure, "remote_epoch_not_served");
        assert_eq!(
            prepared_capacity_validator,
            "warn_if_remote_max_prepared_transactions_unavailable_or_zero"
        );
        assert_eq!(prepared_capacity_action, "nonblocking_registration_warning");
        assert_eq!(
            remote_shape_validator,
            "fingerprint_current_remote_heap_columns"
        );
        assert_eq!(raw_conninfo_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_remote_node_descriptor_readiness_missing() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_node_desc_ready_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_node_desc_ready_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_node_desc_ready_sql_idx \
             ON ec_spire_remote_node_desc_ready_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_node_desc_ready_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_node_desc_ready_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let readiness_from = "FROM ec_spire_remote_node_descriptor_readiness(\
             'ec_spire_remote_node_desc_ready_sql_idx'::regclass)";
        let summary_from = "FROM ec_spire_remote_node_descriptor_readiness_summary(\
             'ec_spire_remote_node_desc_ready_sql_idx'::regclass)";
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {readiness_from}"))
            .expect("descriptor readiness count query should succeed")
            .expect("descriptor readiness count should exist");
        let raw_conninfo_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {readiness_from} \
             WHERE field_name = 'conninfo' OR semantic_role = 'raw_connection_string'"
        ))
        .expect("descriptor readiness raw conninfo query should succeed")
        .expect("descriptor readiness raw conninfo count should exist");
        let secret_status = Spi::get_one::<String>(&format!(
            "SELECT status {readiness_from} \
             WHERE node_id = 2 AND field_name = 'conninfo_secret_name'"
        ))
        .expect("descriptor readiness secret status query should succeed")
        .expect("descriptor readiness secret status should exist");
        let optional_status = Spi::get_one::<String>(&format!(
            "SELECT status {readiness_from} \
             WHERE node_id = 2 AND field_name = 'last_error'"
        ))
        .expect("descriptor readiness optional status query should succeed")
        .expect("descriptor readiness optional status should exist");
        let summary_status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("descriptor readiness summary status query should succeed")
            .expect("descriptor readiness summary status should exist");
        let remote_node_count =
            Spi::get_one::<i64>(&format!("SELECT remote_node_count {summary_from}"))
                .expect("descriptor readiness summary node count query should succeed")
                .expect("descriptor readiness summary node count should exist");
        let required_field_count =
            Spi::get_one::<i64>(&format!("SELECT required_field_count {summary_from}"))
                .expect("descriptor readiness summary required count query should succeed")
                .expect("descriptor readiness summary required count should exist");
        let blocked_field_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_field_count {summary_from}"))
                .expect("descriptor readiness summary blocked count query should succeed")
                .expect("descriptor readiness summary blocked count should exist");
        let missing_required_field_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_required_field_count {summary_from}"
        ))
        .expect("descriptor readiness summary missing required query should succeed")
        .expect("descriptor readiness summary missing required count should exist");

        assert_eq!(row_count, 13);
        assert_eq!(raw_conninfo_count, 0);
        assert_eq!(secret_status, "missing_descriptor");
        assert_eq!(optional_status, "optional_descriptor_missing");
        assert_eq!(summary_status, "requires_remote_node_descriptor");
        assert_eq!(remote_node_count, 1);
        assert_eq!(required_field_count, 11);
        assert_eq!(blocked_field_count, 11);
        assert_eq!(missing_required_field_count, 11);
    }

    #[pg_test]
    fn test_ec_spire_remote_node_capability_plan_local() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_node_cap_local_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_node_cap_local_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_node_cap_local_sql_idx \
             ON ec_spire_remote_node_cap_local_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let plan_from = "FROM ec_spire_remote_node_capability_plan(\
             'ec_spire_remote_node_cap_local_sql_idx'::regclass)";
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {plan_from}"))
            .expect("capability plan count query should succeed")
            .expect("capability plan count should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {plan_from}"))
            .expect("capability plan status query should succeed")
            .expect("capability plan status should exist");
        let conninfo_source =
            Spi::get_one::<String>(&format!("SELECT conninfo_source {plan_from}"))
                .expect("capability plan conninfo source query should succeed")
                .expect("capability plan conninfo source should exist");
        let candidate_format =
            Spi::get_one::<String>(&format!("SELECT required_candidate_format {plan_from}"))
                .expect("capability plan candidate format query should succeed")
                .expect("capability plan candidate format should exist");
        let epoch_window_status =
            Spi::get_one::<String>(&format!("SELECT epoch_window_status {plan_from}"))
                .expect("capability plan epoch status query should succeed")
                .expect("capability plan epoch status should exist");

        assert_eq!(row_count, 1);
        assert_eq!(status, "ready");
        assert_eq!(conninfo_source, "local");
        assert_eq!(candidate_format, "local");
        assert_eq!(epoch_window_status, "ready");
    }

    #[pg_test]
    fn test_ec_spire_remote_node_capability_plan_missing_descriptor() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_node_cap_missing_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_node_cap_missing_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_node_cap_missing_sql_idx \
             ON ec_spire_remote_node_cap_missing_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_node_cap_missing_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_node_cap_missing_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_node_cap_missing_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let plan_from = "FROM ec_spire_remote_node_capability_plan(\
             'ec_spire_remote_node_cap_missing_sql_idx'::regclass)";
        let remote_status =
            Spi::get_one::<String>(&format!("SELECT status {plan_from} WHERE node_id = 2"))
                .expect("remote capability status query should succeed")
                .expect("remote capability status should exist");
        let remote_conninfo_source = Spi::get_one::<String>(&format!(
            "SELECT conninfo_source {plan_from} WHERE node_id = 2"
        ))
        .expect("remote capability conninfo query should succeed")
        .expect("remote capability conninfo should exist");
        let remote_identity_status = Spi::get_one::<String>(&format!(
            "SELECT remote_index_identity_status {plan_from} WHERE node_id = 2"
        ))
        .expect("remote capability identity query should succeed")
        .expect("remote capability identity should exist");
        let remote_candidate_status = Spi::get_one::<String>(&format!(
            "SELECT candidate_format_status {plan_from} WHERE node_id = 2"
        ))
        .expect("remote capability candidate status query should succeed")
        .expect("remote capability candidate status should exist");
        let required_epoch = Spi::get_one::<i64>(&format!(
            "SELECT required_last_served_epoch {plan_from} WHERE node_id = 2"
        ))
        .expect("remote capability epoch query should succeed")
        .expect("remote capability epoch should exist");
        let required_format = Spi::get_one::<String>(&format!(
            "SELECT required_candidate_format {plan_from} WHERE node_id = 2"
        ))
        .expect("remote capability format query should succeed")
        .expect("remote capability format should exist");

        assert_eq!(remote_status, "requires_remote_node_descriptor");
        assert_eq!(remote_conninfo_source, "remote_node_descriptor");
        assert_eq!(remote_identity_status, "missing_descriptor");
        assert_eq!(remote_candidate_status, "missing_descriptor");
        assert_eq!(required_epoch, active_epoch);
        assert_eq!(required_format, "ec_spire_remote_search_v1");
    }

    #[pg_test]
    fn test_ec_spire_remote_node_cap_summary_local() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_cap_summary_local_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_cap_summary_local_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_cap_summary_local_sql_idx \
             ON ec_spire_remote_cap_summary_local_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let capability_from = "FROM ec_spire_remote_node_capability_summary(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let publish_from = "FROM ec_spire_remote_epoch_publish_readiness(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let publish_gate_from = "FROM ec_spire_remote_epoch_publish_gate_summary(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let manifest_summary_from = "FROM ec_spire_remote_epoch_manifest_summary(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let manifest_catalog_summary_from = "FROM ec_spire_remote_epoch_manifest_catalog_summary(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let manifest_publication_summary_from =
            "FROM ec_spire_remote_epoch_manifest_publication_summary(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let manifest_libpq_request_from = "FROM ec_spire_remote_epoch_manifest_libpq_request_plan(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let manifest_libpq_request_summary_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_request_summary(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let manifest_payload_summary_from = "FROM ec_spire_remote_epoch_manifest_payload_summary(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let manifest_dispatch_summary_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_dispatch_summary(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let manifest_executor_readiness_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_executor_readiness(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let manifest_receive_summary_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_receive_summary(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let manifest_publication_gate_from =
            "FROM ec_spire_remote_epoch_manifest_publication_gate_summary(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";
        let manifest_publication_result_from =
            "FROM ec_spire_remote_epoch_manifest_publication_result_summary(\
             'ec_spire_remote_cap_summary_local_sql_idx'::regclass)";

        let capability_status = Spi::get_one::<String>(&format!("SELECT status {capability_from}"))
            .expect("capability summary status query should succeed")
            .expect("capability summary status should exist");
        let node_count = Spi::get_one::<i64>(&format!("SELECT node_count {capability_from}"))
            .expect("capability summary node count query should succeed")
            .expect("capability summary node count should exist");
        let remote_node_count =
            Spi::get_one::<i64>(&format!("SELECT remote_node_count {capability_from}"))
                .expect("capability summary remote node count query should succeed")
                .expect("capability summary remote node count should exist");
        let blocked_node_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_node_count {capability_from}"))
                .expect("capability summary blocked node count query should succeed")
                .expect("capability summary blocked node count should exist");
        let required_format = Spi::get_one::<String>(&format!(
            "SELECT required_candidate_format {capability_from}"
        ))
        .expect("capability summary format query should succeed")
        .expect("capability summary format should exist");
        let publish_status = Spi::get_one::<String>(&format!("SELECT status {publish_from}"))
            .expect("epoch publish readiness status query should succeed")
            .expect("epoch publish readiness status should exist");
        let remote_placement_count =
            Spi::get_one::<i64>(&format!("SELECT remote_placement_count {publish_from}"))
                .expect("epoch publish readiness placement query should succeed")
                .expect("epoch publish readiness placement count should exist");
        let publish_scope =
            Spi::get_one::<String>(&format!("SELECT publish_scope {publish_gate_from}"))
                .expect("epoch publish gate scope query should succeed")
                .expect("epoch publish gate scope should exist");
        let publish_decision =
            Spi::get_one::<String>(&format!("SELECT publish_decision {publish_gate_from}"))
                .expect("epoch publish gate decision query should succeed")
                .expect("epoch publish gate decision should exist");
        let next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {publish_gate_from}"))
                .expect("epoch publish gate blocker query should succeed")
                .expect("epoch publish gate blocker should exist");
        let manifest_decision =
            Spi::get_one::<String>(&format!("SELECT manifest_decision {manifest_summary_from}"))
                .expect("epoch manifest decision query should succeed")
                .expect("epoch manifest decision should exist");
        let manifest_entry_count = Spi::get_one::<i64>(&format!(
            "SELECT manifest_entry_count {manifest_summary_from}"
        ))
        .expect("epoch manifest entry count query should succeed")
        .expect("epoch manifest entry count should exist");
        let catalog_status = Spi::get_one::<String>(&format!(
            "SELECT catalog_status {manifest_catalog_summary_from}"
        ))
        .expect("manifest catalog summary status query should succeed")
        .expect("manifest catalog summary status should exist");
        let publication_decision = Spi::get_one::<String>(&format!(
            "SELECT publication_decision {manifest_publication_summary_from}"
        ))
        .expect("manifest publication summary decision query should succeed")
        .expect("manifest publication summary decision should exist");
        let publication_entry_count = Spi::get_one::<i64>(&format!(
            "SELECT publication_entry_count {manifest_publication_summary_from}"
        ))
        .expect("manifest publication summary entry count query should succeed")
        .expect("manifest publication summary entry count should exist");
        let publication_status = Spi::get_one::<String>(&format!(
            "SELECT status {manifest_publication_summary_from}"
        ))
        .expect("manifest publication summary status query should succeed")
        .expect("manifest publication summary status should exist");
        let publication_executor_status = Spi::get_one::<String>(&format!(
            "SELECT publication_executor_status {manifest_publication_summary_from}"
        ))
        .expect("manifest publication summary executor status query should succeed")
        .expect("manifest publication summary executor status should exist");
        let libpq_request_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_libpq_request_from}"))
                .expect("manifest libpq request count query should succeed")
                .expect("manifest libpq request count should exist");
        let libpq_request_summary_count = Spi::get_one::<i64>(&format!(
            "SELECT request_count {manifest_libpq_request_summary_from}"
        ))
        .expect("manifest libpq request summary count query should succeed")
        .expect("manifest libpq request summary count should exist");
        let libpq_request_summary_status = Spi::get_one::<String>(&format!(
            "SELECT status {manifest_libpq_request_summary_from}"
        ))
        .expect("manifest libpq request summary status query should succeed")
        .expect("manifest libpq request summary status should exist");
        let manifest_payload_count = Spi::get_one::<i64>(&format!(
            "SELECT payload_count {manifest_payload_summary_from}"
        ))
        .expect("manifest payload summary count query should succeed")
        .expect("manifest payload summary count should exist");
        let manifest_payload_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_payload_summary_from}"))
                .expect("manifest payload summary status query should succeed")
                .expect("manifest payload summary status should exist");
        let manifest_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT dispatch_count {manifest_dispatch_summary_from}"
        ))
        .expect("manifest dispatch summary count query should succeed")
        .expect("manifest dispatch summary count should exist");
        let manifest_dispatch_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_dispatch_summary_from}"))
                .expect("manifest dispatch summary status query should succeed")
                .expect("manifest dispatch summary status should exist");
        let manifest_executor_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_executor_readiness_from}"))
                .expect("manifest executor readiness status query should succeed")
                .expect("manifest executor readiness status should exist");
        let manifest_executor_next_step = Spi::get_one::<String>(&format!(
            "SELECT next_executor_step {manifest_executor_readiness_from}"
        ))
        .expect("manifest executor readiness next step query should succeed")
        .expect("manifest executor readiness next step should exist");
        let manifest_receive_count = Spi::get_one::<i64>(&format!(
            "SELECT receive_count {manifest_receive_summary_from}"
        ))
        .expect("manifest receive summary count query should succeed")
        .expect("manifest receive summary count should exist");
        let manifest_receive_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_receive_summary_from}"))
                .expect("manifest receive summary status query should succeed")
                .expect("manifest receive summary status should exist");
        let manifest_gate_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_publication_gate_from}"))
                .expect("manifest publication gate status query should succeed")
                .expect("manifest publication gate status should exist");
        let manifest_gate_next_blocker = Spi::get_one::<String>(&format!(
            "SELECT next_blocker {manifest_publication_gate_from}"
        ))
        .expect("manifest publication gate blocker query should succeed")
        .expect("manifest publication gate blocker should exist");
        let manifest_result_source = Spi::get_one::<String>(&format!(
            "SELECT result_source {manifest_publication_result_from}"
        ))
        .expect("manifest publication result source query should succeed")
        .expect("manifest publication result source should exist");
        let manifest_result_receive_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_receive_count {manifest_publication_result_from}"
        ))
        .expect("manifest publication result receive count query should succeed")
        .expect("manifest publication result receive count should exist");
        let manifest_result_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_publication_result_from}"))
                .expect("manifest publication result status query should succeed")
                .expect("manifest publication result status should exist");
        let manifest_result_next_blocker = Spi::get_one::<String>(&format!(
            "SELECT next_blocker {manifest_publication_result_from}"
        ))
        .expect("manifest publication result blocker query should succeed")
        .expect("manifest publication result blocker should exist");

        assert_eq!(capability_status, "ready");
        assert_eq!(node_count, 1);
        assert_eq!(remote_node_count, 0);
        assert_eq!(blocked_node_count, 0);
        assert_eq!(required_format, "local");
        assert_eq!(publish_status, "ready");
        assert_eq!(remote_placement_count, 0);
        assert_eq!(publish_scope, "local_only");
        assert_eq!(publish_decision, "publish_local_epoch");
        assert_eq!(next_blocker, "none");
        assert_eq!(manifest_decision, "emit_local_epoch_manifest");
        assert_eq!(manifest_entry_count, 0);
        assert_eq!(catalog_status, "not_required");
        assert_eq!(publication_decision, "not_required");
        assert_eq!(publication_entry_count, 0);
        assert_eq!(publication_status, "not_required");
        assert_eq!(publication_executor_status, "none");
        assert_eq!(libpq_request_count, 0);
        assert_eq!(libpq_request_summary_count, 0);
        assert_eq!(libpq_request_summary_status, "not_required");
        assert_eq!(manifest_payload_count, 0);
        assert_eq!(manifest_payload_status, "not_required");
        assert_eq!(manifest_dispatch_count, 0);
        assert_eq!(manifest_dispatch_status, "not_required");
        assert_eq!(manifest_executor_status, "not_required");
        assert_eq!(manifest_executor_next_step, "none");
        assert_eq!(manifest_receive_count, 0);
        assert_eq!(manifest_receive_status, "not_required");
        assert_eq!(manifest_gate_status, "not_required");
        assert_eq!(manifest_gate_next_blocker, "none");
        assert_eq!(manifest_result_source, "not_required");
        assert_eq!(manifest_result_receive_count, 0);
        assert_eq!(manifest_result_status, "not_required");
        assert_eq!(manifest_result_next_blocker, "none");
    }

    #[pg_test]
    fn test_ec_spire_remote_node_cap_summary_missing() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_cap_summary_missing_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_cap_summary_missing_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_cap_summary_missing_sql_idx \
             ON ec_spire_remote_cap_summary_missing_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_cap_summary_missing_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_cap_summary_missing_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let capability_from = "FROM ec_spire_remote_node_capability_summary(\
             'ec_spire_remote_cap_summary_missing_sql_idx'::regclass)";
        let publish_from = "FROM ec_spire_remote_epoch_publish_readiness(\
             'ec_spire_remote_cap_summary_missing_sql_idx'::regclass)";
        let publish_gate_from = "FROM ec_spire_remote_epoch_publish_gate_summary(\
             'ec_spire_remote_cap_summary_missing_sql_idx'::regclass)";
        let manifest_plan_from = "FROM ec_spire_remote_epoch_manifest_plan(\
             'ec_spire_remote_cap_summary_missing_sql_idx'::regclass)";
        let manifest_summary_from = "FROM ec_spire_remote_epoch_manifest_summary(\
             'ec_spire_remote_cap_summary_missing_sql_idx'::regclass)";
        let capability_status = Spi::get_one::<String>(&format!("SELECT status {capability_from}"))
            .expect("capability summary status query should succeed")
            .expect("capability summary status should exist");
        let node_count = Spi::get_one::<i64>(&format!("SELECT node_count {capability_from}"))
            .expect("capability summary node count query should succeed")
            .expect("capability summary node count should exist");
        let remote_node_count =
            Spi::get_one::<i64>(&format!("SELECT remote_node_count {capability_from}"))
                .expect("capability summary remote node count query should succeed")
                .expect("capability summary remote node count should exist");
        let blocked_node_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_node_count {capability_from}"))
                .expect("capability summary blocked node count query should succeed")
                .expect("capability summary blocked node count should exist");
        let missing_descriptor_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_node_count {capability_from}"
        ))
        .expect("capability summary missing descriptor query should succeed")
        .expect("capability summary missing descriptor count should exist");
        let required_format = Spi::get_one::<String>(&format!(
            "SELECT required_candidate_format {capability_from}"
        ))
        .expect("capability summary format query should succeed")
        .expect("capability summary format should exist");
        let publish_status = Spi::get_one::<String>(&format!("SELECT status {publish_from}"))
            .expect("epoch publish readiness status query should succeed")
            .expect("epoch publish readiness status should exist");
        let remote_placement_count =
            Spi::get_one::<i64>(&format!("SELECT remote_placement_count {publish_from}"))
                .expect("epoch publish readiness placement query should succeed")
                .expect("epoch publish readiness placement count should exist");
        let remote_available_count = Spi::get_one::<i64>(&format!(
            "SELECT remote_available_placement_count {publish_from}"
        ))
        .expect("epoch publish readiness available placement query should succeed")
        .expect("epoch publish readiness available placement count should exist");
        let blocked_remote_node_count =
            Spi::get_one::<i64>(&format!("SELECT blocked_remote_node_count {publish_from}"))
                .expect("epoch publish readiness blocked node query should succeed")
                .expect("epoch publish readiness blocked node count should exist");
        let publish_scope =
            Spi::get_one::<String>(&format!("SELECT publish_scope {publish_gate_from}"))
                .expect("epoch publish gate scope query should succeed")
                .expect("epoch publish gate scope should exist");
        let publish_decision =
            Spi::get_one::<String>(&format!("SELECT publish_decision {publish_gate_from}"))
                .expect("epoch publish gate decision query should succeed")
                .expect("epoch publish gate decision should exist");
        let next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {publish_gate_from}"))
                .expect("epoch publish gate blocker query should succeed")
                .expect("epoch publish gate blocker should exist");
        let policy_contract =
            Spi::get_one::<String>(&format!("SELECT policy_contract {publish_gate_from}"))
                .expect("epoch publish gate policy query should succeed")
                .expect("epoch publish gate policy should exist");
        let manifest_action =
            Spi::get_one::<String>(&format!("SELECT manifest_action {manifest_plan_from}"))
                .expect("epoch manifest action query should succeed")
                .expect("epoch manifest action should exist");
        let manifest_decision =
            Spi::get_one::<String>(&format!("SELECT manifest_decision {manifest_summary_from}"))
                .expect("epoch manifest decision query should succeed")
                .expect("epoch manifest decision should exist");
        let blocked_manifest_count = Spi::get_one::<i64>(&format!(
            "SELECT blocked_remote_node_count {manifest_summary_from}"
        ))
        .expect("epoch manifest blocked count query should succeed")
        .expect("epoch manifest blocked count should exist");

        assert_eq!(capability_status, "requires_remote_node_descriptor");
        assert_eq!(node_count, 2);
        assert_eq!(remote_node_count, 1);
        assert_eq!(blocked_node_count, 1);
        assert_eq!(missing_descriptor_count, 1);
        assert_eq!(required_format, "ec_spire_remote_search_v1");
        assert_eq!(publish_status, "requires_remote_node_descriptor");
        assert_eq!(remote_placement_count, 1);
        assert_eq!(remote_available_count, 1);
        assert_eq!(blocked_remote_node_count, 1);
        assert_eq!(publish_scope, "distributed");
        assert_eq!(publish_decision, "block_publish");
        assert_eq!(next_blocker, "remote_node_descriptor");
        assert_eq!(
            policy_contract,
            "ec_spire_remote_degradation_policy_contract"
        );
        assert_eq!(manifest_action, "block_manifest");
        assert_eq!(manifest_decision, "block_manifest");
        assert_eq!(blocked_manifest_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_epoch_publish_plan_missing() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_epoch_plan_missing_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_epoch_plan_missing_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_epoch_plan_missing_sql_idx \
             ON ec_spire_remote_epoch_plan_missing_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_epoch_plan_missing_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_epoch_plan_missing_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_epoch_plan_missing_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let plan_from = "FROM ec_spire_remote_epoch_publish_plan(\
             'ec_spire_remote_epoch_plan_missing_sql_idx'::regclass)";
        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) {plan_from}"))
            .expect("epoch publish plan count query should succeed")
            .expect("epoch publish plan count should exist");
        let descriptor_state =
            Spi::get_one::<String>(&format!("SELECT descriptor_state {plan_from}"))
                .expect("epoch publish plan descriptor query should succeed")
                .expect("epoch publish plan descriptor should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {plan_from}"))
            .expect("epoch publish plan status query should succeed")
            .expect("epoch publish plan status should exist");
        let epoch_window_status =
            Spi::get_one::<String>(&format!("SELECT epoch_window_status {plan_from}"))
                .expect("epoch publish plan epoch window query should succeed")
                .expect("epoch publish plan epoch window should exist");
        let required_last_served_epoch =
            Spi::get_one::<i64>(&format!("SELECT required_last_served_epoch {plan_from}"))
                .expect("epoch publish plan required served query should succeed")
                .expect("epoch publish plan required served should exist");
        let last_served_epoch =
            Spi::get_one::<i64>(&format!("SELECT last_served_epoch {plan_from}"))
                .expect("epoch publish plan served query should succeed")
                .expect("epoch publish plan served should exist");
        let placement_count = Spi::get_one::<i64>(&format!("SELECT placement_count {plan_from}"))
            .expect("epoch publish plan placement query should succeed")
            .expect("epoch publish plan placement count should exist");

        assert_eq!(row_count, 1);
        assert_eq!(descriptor_state, "missing");
        assert_eq!(status, "requires_remote_node_descriptor");
        assert_eq!(epoch_window_status, "missing_descriptor");
        assert_eq!(required_last_served_epoch, active_epoch);
        assert_eq!(last_served_epoch, 0);
        assert_eq!(placement_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_epoch_publish_manifest_stale_descriptor() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_epoch_manifest_stale_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_epoch_manifest_stale_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_epoch_manifest_stale_sql_idx \
             ON ec_spire_remote_epoch_manifest_stale_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");
        let stale_served_epoch = active_epoch.saturating_sub(1);
        assert!(stale_served_epoch < active_epoch);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 8, 'spire/remote/stale', decode('02', 'hex'), \
                     'remote_spire_idx', 'active', {stale_served_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("stale remote descriptor registration should succeed")
        .expect("stale remote descriptor registration result should exist");

        let plan_from = "FROM ec_spire_remote_epoch_publish_plan(\
             'ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)";
        let readiness_from = "FROM ec_spire_remote_epoch_publish_readiness(\
             'ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)";
        let gate_from = "FROM ec_spire_remote_epoch_publish_gate_summary(\
             'ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)";
        let manifest_from = "FROM ec_spire_remote_epoch_manifest_summary(\
             'ec_spire_remote_epoch_manifest_stale_sql_idx'::regclass)";

        let plan_status = Spi::get_one::<String>(&format!("SELECT status {plan_from}"))
            .expect("stale publish plan status query should succeed")
            .expect("stale publish plan status should exist");
        let epoch_window_status =
            Spi::get_one::<String>(&format!("SELECT epoch_window_status {plan_from}"))
                .expect("stale publish plan epoch window query should succeed")
                .expect("stale publish plan epoch window should exist");
        let readiness_status = Spi::get_one::<String>(&format!("SELECT status {readiness_from}"))
            .expect("stale publish readiness status query should succeed")
            .expect("stale publish readiness status should exist");
        let blocked_remote_node_count = Spi::get_one::<i64>(&format!(
            "SELECT blocked_remote_node_count {readiness_from}"
        ))
        .expect("stale publish readiness blocked count query should succeed")
        .expect("stale publish readiness blocked count should exist");
        let missing_descriptor_node_count = Spi::get_one::<i64>(&format!(
            "SELECT missing_descriptor_node_count {readiness_from}"
        ))
        .expect("stale publish readiness missing count query should succeed")
        .expect("stale publish readiness missing count should exist");
        let next_blocker = Spi::get_one::<String>(&format!("SELECT next_blocker {gate_from}"))
            .expect("stale publish gate blocker query should succeed")
            .expect("stale publish gate blocker should exist");
        let manifest_decision =
            Spi::get_one::<String>(&format!("SELECT manifest_decision {manifest_from}"))
                .expect("stale manifest decision query should succeed")
                .expect("stale manifest decision should exist");

        assert!(register_result);
        assert_eq!(plan_status, "stale_epoch");
        assert_eq!(epoch_window_status, "stale_epoch");
        assert_eq!(readiness_status, "remote_epoch_window");
        assert_eq!(blocked_remote_node_count, 1);
        assert_eq!(missing_descriptor_node_count, 0);
        assert_eq!(next_blocker, "remote_epoch_window");
        assert_eq!(manifest_decision, "block_manifest");
    }

    #[pg_test]
    fn test_ec_spire_remote_epoch_manifest_persist_ready() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_manifest_persist_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_manifest_persist_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_manifest_persist_sql_idx \
             ON ec_spire_remote_manifest_persist_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_manifest_persist_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_manifest_persist_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_manifest_persist_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 11, 'spire/remote/persist', decode('04', 'hex'), \
                     'remote_spire_idx', 'active', {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        let persist_result = Spi::get_one::<bool>(
            "SELECT ec_spire_persist_remote_epoch_manifest(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)",
        )
        .expect("remote manifest persist should succeed")
        .expect("remote manifest persist result should exist");

        let catalog_from = "FROM ec_spire_remote_epoch_manifest_catalog(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let entry_from = "FROM ec_spire_remote_epoch_manifest_entry_catalog(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let summary_from = "FROM ec_spire_remote_epoch_manifest_catalog_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let publication_from = "FROM ec_spire_remote_epoch_manifest_publication_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let freshness_from = "FROM ec_spire_remote_epoch_manifest_freshness(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let publication_summary_from = "FROM ec_spire_remote_epoch_manifest_publication_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_libpq_request_from = "FROM ec_spire_remote_epoch_manifest_libpq_request_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_libpq_request_summary_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_request_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_payload_from = "FROM ec_spire_remote_epoch_manifest_payload_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_payload_summary_from = "FROM ec_spire_remote_epoch_manifest_payload_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_dispatch_from = "FROM ec_spire_remote_epoch_manifest_libpq_dispatch_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_bind_from = "FROM ec_spire_remote_epoch_manifest_libpq_bind_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_bind_summary_from = "FROM ec_spire_remote_epoch_manifest_libpq_bind_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_work_from = "FROM ec_spire_remote_epoch_manifest_libpq_executor_work_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_work_summary_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_executor_work_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_dispatch_summary_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_dispatch_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_executor_readiness_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_executor_readiness(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_receive_from = "FROM ec_spire_remote_epoch_manifest_libpq_receive_plan(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_receive_summary_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_receive_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_publication_gate_from =
            "FROM ec_spire_remote_epoch_manifest_publication_gate_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let manifest_publication_result_from =
            "FROM ec_spire_remote_epoch_manifest_publication_result_summary(\
             'ec_spire_remote_manifest_persist_sql_idx'::regclass)";
        let catalog_count = Spi::get_one::<i64>(&format!("SELECT count(*) {catalog_from}"))
            .expect("manifest catalog count query should succeed")
            .expect("manifest catalog count should exist");
        let manifest_decision =
            Spi::get_one::<String>(&format!("SELECT manifest_decision {catalog_from}"))
                .expect("manifest catalog decision query should succeed")
                .expect("manifest catalog decision should exist");
        let manifest_entry_count =
            Spi::get_one::<i64>(&format!("SELECT manifest_entry_count {catalog_from}"))
                .expect("manifest catalog entry count query should succeed")
                .expect("manifest catalog entry count should exist");
        let included_remote_node_count =
            Spi::get_one::<i64>(&format!("SELECT included_remote_node_count {catalog_from}"))
                .expect("manifest catalog included node count query should succeed")
                .expect("manifest catalog included node count should exist");
        let persisted_at_micros =
            Spi::get_one::<i64>(&format!("SELECT persisted_at_micros {catalog_from}"))
                .expect("manifest catalog timestamp query should succeed")
                .expect("manifest catalog timestamp should exist");
        let entry_count = Spi::get_one::<i64>(&format!("SELECT count(*) {entry_from}"))
            .expect("manifest entry count query should succeed")
            .expect("manifest entry count should exist");
        let entry_node_id = Spi::get_one::<i64>(&format!("SELECT node_id {entry_from}"))
            .expect("manifest entry node query should succeed")
            .expect("manifest entry node should exist");
        let entry_action = Spi::get_one::<String>(&format!("SELECT manifest_action {entry_from}"))
            .expect("manifest entry action query should succeed")
            .expect("manifest entry action should exist");
        let entry_status = Spi::get_one::<String>(&format!("SELECT status {entry_from}"))
            .expect("manifest entry status query should succeed")
            .expect("manifest entry status should exist");
        let summary_status =
            Spi::get_one::<String>(&format!("SELECT catalog_status {summary_from}"))
                .expect("manifest catalog summary status query should succeed")
                .expect("manifest catalog summary status should exist");
        let summary_persisted_entry_count =
            Spi::get_one::<i64>(&format!("SELECT persisted_entry_count {summary_from}"))
                .expect("manifest catalog summary entry count query should succeed")
                .expect("manifest catalog summary entry count should exist");
        let summary_mismatch_count = Spi::get_one::<i64>(&format!(
            "SELECT persisted_entry_mismatch_count {summary_from}"
        ))
        .expect("manifest catalog summary mismatch count query should succeed")
        .expect("manifest catalog summary mismatch count should exist");
        let publication_action =
            Spi::get_one::<String>(&format!("SELECT publication_action {publication_from}"))
                .expect("manifest publication action query should succeed")
                .expect("manifest publication action should exist");
        let publication_transport =
            Spi::get_one::<String>(&format!("SELECT publication_transport {publication_from}"))
                .expect("manifest publication transport query should succeed")
                .expect("manifest publication transport should exist");
        let publication_status =
            Spi::get_one::<String>(&format!("SELECT status {publication_from}"))
                .expect("manifest publication status query should succeed")
                .expect("manifest publication status should exist");
        let publication_entry_matches = Spi::get_one::<bool>(&format!(
            "SELECT persisted_entry_matches {publication_from}"
        ))
        .expect("manifest publication match query should succeed")
        .expect("manifest publication match should exist");
        let freshness_status =
            Spi::get_one::<String>(&format!("SELECT freshness_status {freshness_from}"))
                .expect("manifest freshness status query should succeed")
                .expect("manifest freshness status should exist");
        let freshness_next_action =
            Spi::get_one::<String>(&format!("SELECT next_action {freshness_from}"))
                .expect("manifest freshness action query should succeed")
                .expect("manifest freshness action should exist");
        let freshness_entry_matches =
            Spi::get_one::<bool>(&format!("SELECT persisted_entry_matches {freshness_from}"))
                .expect("manifest freshness match query should succeed")
                .expect("manifest freshness match should exist");
        let publication_summary_decision = Spi::get_one::<String>(&format!(
            "SELECT publication_decision {publication_summary_from}"
        ))
        .expect("manifest publication summary decision query should succeed")
        .expect("manifest publication summary decision should exist");
        let publication_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_publication_count {publication_summary_from}"
        ))
        .expect("manifest publication summary ready count query should succeed")
        .expect("manifest publication summary ready count should exist");
        let publication_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {publication_summary_from}"))
                .expect("manifest publication summary status query should succeed")
                .expect("manifest publication summary status should exist");
        let publication_summary_executor_status = Spi::get_one::<String>(&format!(
            "SELECT publication_executor_status {publication_summary_from}"
        ))
        .expect("manifest publication summary executor status query should succeed")
        .expect("manifest publication summary executor status should exist");
        let publication_summary_executor_step = Spi::get_one::<String>(&format!(
            "SELECT publication_executor_next_step {publication_summary_from}"
        ))
        .expect("manifest publication summary executor step query should succeed")
        .expect("manifest publication summary executor step should exist");
        let libpq_request_action = Spi::get_one::<String>(&format!(
            "SELECT request_action {manifest_libpq_request_from}"
        ))
        .expect("manifest libpq request action query should succeed")
        .expect("manifest libpq request action should exist");
        let libpq_request_sql = Spi::get_one::<String>(&format!(
            "SELECT sql_template {manifest_libpq_request_from}"
        ))
        .expect("manifest libpq request SQL query should succeed")
        .expect("manifest libpq request SQL should exist");
        let libpq_request_parameter_count = Spi::get_one::<i64>(&format!(
            "SELECT parameter_count {manifest_libpq_request_from}"
        ))
        .expect("manifest libpq request parameter count query should succeed")
        .expect("manifest libpq request parameter count should exist");
        let libpq_request_executor_status = Spi::get_one::<String>(&format!(
            "SELECT executor_status {manifest_libpq_request_from}"
        ))
        .expect("manifest libpq request executor status query should succeed")
        .expect("manifest libpq request executor status should exist");
        let libpq_request_summary_count = Spi::get_one::<i64>(&format!(
            "SELECT request_count {manifest_libpq_request_summary_from}"
        ))
        .expect("manifest libpq request summary count query should succeed")
        .expect("manifest libpq request summary count should exist");
        let libpq_request_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_request_count {manifest_libpq_request_summary_from}"
        ))
        .expect("manifest libpq request summary ready count query should succeed")
        .expect("manifest libpq request summary ready count should exist");
        let libpq_request_summary_result_columns = Spi::get_one::<i64>(&format!(
            "SELECT expected_result_column_count {manifest_libpq_request_summary_from}"
        ))
        .expect("manifest libpq request summary result count query should succeed")
        .expect("manifest libpq request summary result count should exist");
        let libpq_request_summary_status = Spi::get_one::<String>(&format!(
            "SELECT status {manifest_libpq_request_summary_from}"
        ))
        .expect("manifest libpq request summary status query should succeed")
        .expect("manifest libpq request summary status should exist");
        let manifest_payload_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_payload_from}"))
                .expect("manifest payload count query should succeed")
                .expect("manifest payload count should exist");
        let manifest_payload_format = Spi::get_one::<String>(&format!(
            "SELECT manifest_payload_format {manifest_payload_from}"
        ))
        .expect("manifest payload format query should succeed")
        .expect("manifest payload format should exist");
        let manifest_payload_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_payload_from}"))
                .expect("manifest payload status query should succeed")
                .expect("manifest payload status should exist");
        let manifest_payload_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_payload_count {manifest_payload_summary_from}"
        ))
        .expect("manifest payload summary ready count query should succeed")
        .expect("manifest payload summary ready count should exist");
        let payload_validation_status = Spi::get_one::<String>(&format!(
            "SELECT status FROM ec_spire_validate_remote_epoch_manifest_payload(\
                 'ec_spire_remote_manifest_persist_sql_idx'::regclass, \
                 {active_epoch}, \
                 (SELECT manifest_payload {manifest_payload_from} WHERE node_id = 2))"
        ))
        .expect("remote manifest payload validation status query should succeed")
        .expect("remote manifest payload validation status should exist");
        let payload_validation_entry_count = Spi::get_one::<i64>(&format!(
            "SELECT validated_entry_count FROM ec_spire_validate_remote_epoch_manifest_payload(\
                 'ec_spire_remote_manifest_persist_sql_idx'::regclass, \
                 {active_epoch}, \
                 (SELECT manifest_payload {manifest_payload_from} WHERE node_id = 2))"
        ))
        .expect("remote manifest payload validation entry count query should succeed")
        .expect("remote manifest payload validation entry count should exist");
        let validation_epoch_mismatch_status = Spi::get_one::<String>(&format!(
            "SELECT status FROM ec_spire_validate_remote_epoch_manifest_payload(\
                 'ec_spire_remote_manifest_persist_sql_idx'::regclass, \
                 {active_epoch} + 1, \
                 (SELECT manifest_payload {manifest_payload_from} WHERE node_id = 2))"
        ))
        .expect("remote manifest payload validation mismatch query should succeed")
        .expect("remote manifest payload validation mismatch status should exist");
        let dispatch_action =
            Spi::get_one::<String>(&format!("SELECT dispatch_action {manifest_dispatch_from}"))
                .expect("manifest dispatch action query should succeed")
                .expect("manifest dispatch action should exist");
        let dispatch_validator = Spi::get_one::<String>(&format!(
            "SELECT receive_validator {manifest_dispatch_from}"
        ))
        .expect("manifest dispatch validator query should succeed")
        .expect("manifest dispatch validator should exist");
        let manifest_bind_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_bind_from}"))
                .expect("manifest bind count query should succeed")
                .expect("manifest bind count should exist");
        let manifest_bind_contract_mismatch_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_epoch_manifest_libpq_parameter_contract() contract \
               LEFT JOIN (SELECT * {manifest_bind_from}) bind \
                 ON bind.parameter_ordinal = contract.parameter_ordinal \
                AND bind.parameter_name = contract.parameter_name \
                AND bind.pg_type = contract.pg_type \
              WHERE bind.parameter_ordinal IS NULL"
        ))
        .expect("manifest bind contract invariant query should succeed")
        .expect("manifest bind contract invariant count should exist");
        let manifest_bind_remote_index_preview = Spi::get_one::<String>(&format!(
            "SELECT value_preview {manifest_bind_from} WHERE parameter_name = 'remote_index_oid'"
        ))
        .expect("manifest bind remote index query should succeed")
        .expect("manifest bind remote index preview should exist");
        let manifest_bind_payload_element_count = Spi::get_one::<i64>(&format!(
            "SELECT element_count {manifest_bind_from} WHERE parameter_name = 'manifest_payload'"
        ))
        .expect("manifest bind payload element count query should succeed")
        .expect("manifest bind payload element count should exist");
        let manifest_bind_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {manifest_bind_from} WHERE value_status = 'ready'"
        ))
        .expect("manifest bind ready count query should succeed")
        .expect("manifest bind ready count should exist");
        let manifest_bind_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_bind_count {manifest_bind_summary_from}"
        ))
        .expect("manifest bind summary ready count query should succeed")
        .expect("manifest bind summary ready count should exist");
        let manifest_bind_summary_entry_count = Spi::get_one::<i64>(&format!(
            "SELECT manifest_entry_count {manifest_bind_summary_from}"
        ))
        .expect("manifest bind summary entry count query should succeed")
        .expect("manifest bind summary entry count should exist");
        let manifest_bind_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_bind_summary_from}"))
                .expect("manifest bind summary status query should succeed")
                .expect("manifest bind summary status should exist");
        let manifest_work_bind_status =
            Spi::get_one::<String>(&format!("SELECT bind_status {manifest_work_from}"))
                .expect("manifest work bind status query should succeed")
                .expect("manifest work bind status should exist");
        let manifest_work_action =
            Spi::get_one::<String>(&format!("SELECT work_action {manifest_work_from}"))
                .expect("manifest work action query should succeed")
                .expect("manifest work action should exist");
        let manifest_work_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_work_from}"))
                .expect("manifest work status query should succeed")
                .expect("manifest work status should exist");
        let manifest_work_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_work_count {manifest_work_summary_from}"
        ))
        .expect("manifest work summary ready count query should succeed")
        .expect("manifest work summary ready count should exist");
        let manifest_work_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_work_summary_from}"))
                .expect("manifest work summary status query should succeed")
                .expect("manifest work summary status should exist");
        let dispatch_executor_status =
            Spi::get_one::<String>(&format!("SELECT executor_status {manifest_dispatch_from}"))
                .expect("manifest dispatch executor status query should succeed")
                .expect("manifest dispatch executor status should exist");
        let dispatch_pipeline_count = Spi::get_one::<i64>(&format!(
            "SELECT pipeline_dispatch_count {manifest_dispatch_summary_from}"
        ))
        .expect("manifest dispatch summary pipeline count query should succeed")
        .expect("manifest dispatch summary pipeline count should exist");
        let dispatch_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_dispatch_summary_from}"))
                .expect("manifest dispatch summary status query should succeed")
                .expect("manifest dispatch summary status should exist");
        let executor_readiness_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_executor_readiness_from}"))
                .expect("manifest executor readiness status query should succeed")
                .expect("manifest executor readiness status should exist");
        let executor_next_step = Spi::get_one::<String>(&format!(
            "SELECT next_executor_step {manifest_executor_readiness_from}"
        ))
        .expect("manifest executor readiness next step query should succeed")
        .expect("manifest executor readiness next step should exist");
        let executor_send_action = Spi::get_one::<String>(&format!(
            "SELECT send_action {manifest_executor_readiness_from}"
        ))
        .expect("manifest executor readiness send action query should succeed")
        .expect("manifest executor readiness send action should exist");
        let manifest_receive_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_receive_from}"))
                .expect("manifest receive count query should succeed")
                .expect("manifest receive count should exist");
        let manifest_receive_validator = Spi::get_one::<String>(&format!(
            "SELECT validator_function {manifest_receive_from}"
        ))
        .expect("manifest receive validator query should succeed")
        .expect("manifest receive validator should exist");
        let manifest_receive_action =
            Spi::get_one::<String>(&format!("SELECT result_action {manifest_receive_from}"))
                .expect("manifest receive action query should succeed")
                .expect("manifest receive action should exist");
        let manifest_receive_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_receive_from}"))
                .expect("manifest receive status query should succeed")
                .expect("manifest receive status should exist");
        let manifest_receive_summary_ready_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_receive_count {manifest_receive_summary_from}"
        ))
        .expect("manifest receive summary ready count query should succeed")
        .expect("manifest receive summary ready count should exist");
        let manifest_receive_summary_result_columns = Spi::get_one::<i64>(&format!(
            "SELECT expected_result_column_count {manifest_receive_summary_from}"
        ))
        .expect("manifest receive summary result count query should succeed")
        .expect("manifest receive summary result count should exist");
        let manifest_receive_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_receive_summary_from}"))
                .expect("manifest receive summary status query should succeed")
                .expect("manifest receive summary status should exist");
        let manifest_gate_request_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_request_count {manifest_publication_gate_from}"
        ))
        .expect("manifest publication gate request count query should succeed")
        .expect("manifest publication gate request count should exist");
        let manifest_gate_dispatch_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_dispatch_count {manifest_publication_gate_from}"
        ))
        .expect("manifest publication gate dispatch count query should succeed")
        .expect("manifest publication gate dispatch count should exist");
        let manifest_gate_receive_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_receive_count {manifest_publication_gate_from}"
        ))
        .expect("manifest publication gate receive count query should succeed")
        .expect("manifest publication gate receive count should exist");
        let manifest_gate_executor_status = Spi::get_one::<String>(&format!(
            "SELECT libpq_executor_status {manifest_publication_gate_from}"
        ))
        .expect("manifest publication gate executor status query should succeed")
        .expect("manifest publication gate executor status should exist");
        let manifest_gate_next_blocker = Spi::get_one::<String>(&format!(
            "SELECT next_blocker {manifest_publication_gate_from}"
        ))
        .expect("manifest publication gate blocker query should succeed")
        .expect("manifest publication gate blocker should exist");
        let manifest_gate_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_publication_gate_from}"))
                .expect("manifest publication gate status query should succeed")
                .expect("manifest publication gate status should exist");
        let manifest_result_source = Spi::get_one::<String>(&format!(
            "SELECT result_source {manifest_publication_result_from}"
        ))
        .expect("manifest publication result source query should succeed")
        .expect("manifest publication result source should exist");
        let manifest_result_receive_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_receive_count {manifest_publication_result_from}"
        ))
        .expect("manifest publication result receive count query should succeed")
        .expect("manifest publication result receive count should exist");
        let manifest_result_ready_receive_count = Spi::get_one::<i64>(&format!(
            "SELECT ready_receive_count {manifest_publication_result_from}"
        ))
        .expect("manifest publication result ready receive count query should succeed")
        .expect("manifest publication result ready receive count should exist");
        let manifest_result_validation_status = Spi::get_one::<String>(&format!(
            "SELECT validation_result_status {manifest_publication_result_from}"
        ))
        .expect("manifest publication result validation status query should succeed")
        .expect("manifest publication result validation status should exist");
        let manifest_result_next_blocker = Spi::get_one::<String>(&format!(
            "SELECT next_blocker {manifest_publication_result_from}"
        ))
        .expect("manifest publication result blocker query should succeed")
        .expect("manifest publication result blocker should exist");
        let manifest_result_status =
            Spi::get_one::<String>(&format!("SELECT status {manifest_publication_result_from}"))
                .expect("manifest publication result status query should succeed")
                .expect("manifest publication result status should exist");
        let executor_contract_mismatch_count = Spi::get_one::<i64>(&format!(
            "WITH readiness AS ( \
                 SELECT * {manifest_executor_readiness_from} \
             ), expected(step_name, readiness_action) AS ( \
                 VALUES \
                     ('conninfo_secret_resolution', \
                         (SELECT secret_resolution_action FROM readiness)), \
                     ('libpq_connection_open', \
                         (SELECT connection_action FROM readiness)), \
                     ('pipeline_mode_start', \
                         (SELECT pipeline_action FROM readiness)), \
                     ('send_manifest_request', \
                         (SELECT send_action FROM readiness)), \
                     ('receive_payload_validation_result', \
                         (SELECT receive_action FROM readiness)) \
             ) \
             SELECT count(*) \
               FROM expected \
               LEFT JOIN ec_spire_remote_epoch_manifest_libpq_executor_step_contract() contract \
                 ON contract.step_name = expected.step_name \
              WHERE contract.step_name IS NULL \
                 OR contract.executor_action <> expected.readiness_action"
        ))
        .expect("manifest executor contract invariant query should succeed")
        .expect("manifest executor contract invariant count should exist");

        assert!(register_result);
        assert!(persist_result);
        assert_eq!(catalog_count, 1);
        assert_eq!(manifest_decision, "emit_distributed_epoch_manifest");
        assert_eq!(manifest_entry_count, 1);
        assert_eq!(included_remote_node_count, 1);
        assert!(persisted_at_micros > 0);
        assert_eq!(entry_count, 1);
        assert_eq!(entry_node_id, 2);
        assert_eq!(entry_action, "include_remote_node");
        assert_eq!(entry_status, "ready");
        assert_eq!(summary_status, "ready");
        assert_eq!(summary_persisted_entry_count, 1);
        assert_eq!(summary_mismatch_count, 0);
        assert_eq!(publication_action, "publish_remote_epoch_manifest");
        assert_eq!(publication_transport, "libpq_pipeline");
        assert_eq!(publication_status, "ready");
        assert!(publication_entry_matches);
        assert_eq!(freshness_status, "ready");
        assert_eq!(freshness_next_action, "none");
        assert!(freshness_entry_matches);
        assert_eq!(
            publication_summary_decision,
            "publish_remote_epoch_manifest"
        );
        assert_eq!(publication_summary_ready_count, 1);
        assert_eq!(publication_summary_status, "ready");
        assert_eq!(
            publication_summary_executor_status,
            "requires_libpq_executor"
        );
        assert_eq!(
            publication_summary_executor_step,
            "conninfo_secret_resolution"
        );
        assert_eq!(libpq_request_action, "send_remote_epoch_manifest");
        assert!(libpq_request_sql.contains("ec_spire_apply_remote_epoch_manifest_payload"));
        assert_eq!(libpq_request_parameter_count, 3);
        assert_eq!(libpq_request_executor_status, "requires_libpq_executor");
        assert_eq!(libpq_request_summary_count, 1);
        assert_eq!(libpq_request_summary_ready_count, 1);
        assert_eq!(libpq_request_summary_result_columns, 3);
        assert_eq!(libpq_request_summary_status, "ready");
        assert_eq!(manifest_payload_count, 1);
        assert_eq!(manifest_payload_format, "ec_spire_remote_epoch_manifest_v1");
        assert_eq!(manifest_payload_status, "ready");
        assert_eq!(manifest_payload_summary_ready_count, 1);
        assert_eq!(payload_validation_status, "ready");
        assert_eq!(payload_validation_entry_count, 1);
        assert_eq!(validation_epoch_mismatch_status, "manifest_epoch_mismatch");
        assert_eq!(
            dispatch_action,
            "open_pipeline_and_send_remote_epoch_manifest"
        );
        assert_eq!(
            dispatch_validator,
            "ec_spire_remote_epoch_manifest_libpq_result_contract"
        );
        assert_eq!(manifest_bind_count, 3);
        assert_eq!(manifest_bind_contract_mismatch_count, 0);
        assert_eq!(manifest_bind_remote_index_preview, "remote_spire_idx");
        assert_eq!(manifest_bind_payload_element_count, 1);
        assert_eq!(manifest_bind_ready_count, 3);
        assert_eq!(manifest_bind_summary_ready_count, 3);
        assert_eq!(manifest_bind_summary_entry_count, 1);
        assert_eq!(manifest_bind_summary_status, "ready");
        assert_eq!(manifest_work_bind_status, "ready");
        assert_eq!(manifest_work_action, "resolve_conninfo_secret");
        assert_eq!(manifest_work_status, "requires_libpq_executor");
        assert_eq!(manifest_work_summary_ready_count, 1);
        assert_eq!(manifest_work_summary_status, "requires_libpq_executor");
        assert_eq!(dispatch_executor_status, "requires_libpq_executor");
        assert_eq!(dispatch_pipeline_count, 1);
        assert_eq!(dispatch_summary_status, "ready");
        assert_eq!(executor_readiness_status, "requires_libpq_executor");
        assert_eq!(executor_next_step, "conninfo_secret_resolution");
        assert_eq!(executor_send_action, "send_remote_epoch_manifest");
        assert_eq!(manifest_receive_count, 1);
        assert_eq!(
            manifest_receive_validator,
            "ec_spire_remote_epoch_manifest_libpq_result_contract"
        );
        assert_eq!(
            manifest_receive_action,
            "validate_remote_manifest_payload_result"
        );
        assert_eq!(manifest_receive_status, "requires_libpq_executor");
        assert_eq!(manifest_receive_summary_ready_count, 1);
        assert_eq!(manifest_receive_summary_result_columns, 3);
        assert_eq!(manifest_receive_summary_status, "requires_libpq_executor");
        assert_eq!(manifest_gate_request_count, 1);
        assert_eq!(manifest_gate_dispatch_count, 1);
        assert_eq!(manifest_gate_receive_count, 1);
        assert_eq!(manifest_gate_executor_status, "requires_libpq_executor");
        assert_eq!(manifest_gate_next_blocker, "conninfo_secret_resolution");
        assert_eq!(manifest_gate_status, "requires_libpq_executor");
        assert_eq!(manifest_result_source, "pending_libpq_executor");
        assert_eq!(manifest_result_receive_count, 1);
        assert_eq!(manifest_result_ready_receive_count, 1);
        assert_eq!(manifest_result_validation_status, "requires_libpq_executor");
        assert_eq!(manifest_result_next_blocker, "conninfo_secret_resolution");
        assert_eq!(manifest_result_status, "requires_libpq_executor");
        assert_eq!(executor_contract_mismatch_count, 0);

        Spi::run(&format!(
            "UPDATE ec_spire_remote_epoch_manifest_entry \
                SET last_served_epoch = last_served_epoch - 1 \
              WHERE coordinator_index_oid = '{}'::oid \
                AND active_epoch = {active_epoch} \
                AND node_id = 2",
            u32::from(index_oid)
        ))
        .expect("manifest entry drift update should succeed");
        let stale_summary_status =
            Spi::get_one::<String>(&format!("SELECT catalog_status {summary_from}"))
                .expect("stale manifest catalog summary status query should succeed")
                .expect("stale manifest catalog summary status should exist");
        let stale_mismatch_count = Spi::get_one::<i64>(&format!(
            "SELECT persisted_entry_mismatch_count {summary_from}"
        ))
        .expect("stale manifest catalog summary mismatch count query should succeed")
        .expect("stale manifest catalog summary mismatch count should exist");
        let stale_publication_action =
            Spi::get_one::<String>(&format!("SELECT publication_action {publication_from}"))
                .expect("stale manifest publication action query should succeed")
                .expect("stale manifest publication action should exist");
        let stale_publication_status =
            Spi::get_one::<String>(&format!("SELECT status {publication_from}"))
                .expect("stale manifest publication status query should succeed")
                .expect("stale manifest publication status should exist");
        let stale_publication_entry_matches = Spi::get_one::<bool>(&format!(
            "SELECT persisted_entry_matches {publication_from}"
        ))
        .expect("stale manifest publication match query should succeed")
        .expect("stale manifest publication match should exist");
        let stale_freshness_status =
            Spi::get_one::<String>(&format!("SELECT freshness_status {freshness_from}"))
                .expect("stale manifest freshness status query should succeed")
                .expect("stale manifest freshness status should exist");
        let stale_freshness_next_action =
            Spi::get_one::<String>(&format!("SELECT next_action {freshness_from}"))
                .expect("stale manifest freshness action query should succeed")
                .expect("stale manifest freshness action should exist");
        let stale_freshness_entry_matches =
            Spi::get_one::<bool>(&format!("SELECT persisted_entry_matches {freshness_from}"))
                .expect("stale manifest freshness match query should succeed")
                .expect("stale manifest freshness match should exist");
        let stale_publication_summary_decision = Spi::get_one::<String>(&format!(
            "SELECT publication_decision {publication_summary_from}"
        ))
        .expect("stale manifest publication summary decision query should succeed")
        .expect("stale manifest publication summary decision should exist");
        let stale_publication_summary_refresh_count = Spi::get_one::<i64>(&format!(
            "SELECT refresh_required_count {publication_summary_from}"
        ))
        .expect("stale manifest publication summary refresh count query should succeed")
        .expect("stale manifest publication summary refresh count should exist");
        let stale_publication_summary_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {publication_summary_from}"))
                .expect("stale manifest publication summary blocker query should succeed")
                .expect("stale manifest publication summary blocker should exist");
        assert_eq!(stale_summary_status, "stale_remote_epoch_manifest");
        assert_eq!(stale_mismatch_count, 1);
        assert_eq!(stale_publication_action, "refresh_remote_epoch_manifest");
        assert_eq!(stale_publication_status, "stale_remote_epoch_manifest");
        assert!(!stale_publication_entry_matches);
        assert_eq!(stale_freshness_status, "stale_remote_epoch_manifest");
        assert_eq!(stale_freshness_next_action, "refresh_remote_epoch_manifest");
        assert!(!stale_freshness_entry_matches);
        assert_eq!(
            stale_publication_summary_decision,
            "refresh_remote_epoch_manifest"
        );
        assert_eq!(stale_publication_summary_refresh_count, 1);
        assert_eq!(
            stale_publication_summary_next_blocker,
            "remote_epoch_manifest_refresh"
        );
    }

    #[pg_test]
    fn test_ec_spire_boundary_replica_manifest_freshness_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_boundary_manifest_freshness_sql (\
               id bigint primary key, \
               source_identity uuid not null, \
               embedding ecvector\
             )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_boundary_manifest_freshness_sql \
             (id, source_identity, embedding) VALUES \
             (1, '00000000-0000-0000-0000-000000000101', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, '00000000-0000-0000-0000-000000000202', encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
             (3, '00000000-0000-0000-0000-000000000303', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
             (4, '00000000-0000-0000-0000-000000000404', encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_boundary_manifest_freshness_idx \
             ON ec_spire_boundary_manifest_freshness_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) INCLUDE (source_identity) \
             WITH ( \
                 source_identity = 'include', \
                 nlists = 4, \
                 nprobe = 4, \
                 boundary_replica_count = 1 \
             )",
        )
        .expect("boundary replica index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_boundary_manifest_freshness_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_boundary_manifest_freshness_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let remote_leaf_pid = Spi::get_one::<i64>(
            "SELECT pid FROM \
             ec_spire_index_object_snapshot('ec_spire_boundary_manifest_freshness_idx'::regclass) \
             WHERE object_kind = 'leaf' \
             ORDER BY pid \
             LIMIT 1",
        )
        .expect("leaf object query should succeed")
        .expect("leaf object should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, remote_leaf_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 22, 'spire/remote/boundary-freshness', decode('22', 'hex'), \
                     'remote_spire_idx', 'active', {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");

        let freshness_from = "FROM ec_spire_remote_epoch_manifest_freshness(\
             'ec_spire_boundary_manifest_freshness_idx'::regclass)";
        let identity_from = "FROM ec_spire_index_boundary_replica_identity_snapshot(\
             'ec_spire_boundary_manifest_freshness_idx'::regclass)";

        let pre_persist_status =
            Spi::get_one::<String>(&format!("SELECT freshness_status {freshness_from}"))
                .expect("pre-persist freshness status query should succeed")
                .expect("pre-persist freshness status should exist");
        let pre_persist_action =
            Spi::get_one::<String>(&format!("SELECT next_action {freshness_from}"))
                .expect("pre-persist freshness action query should succeed")
                .expect("pre-persist freshness action should exist");
        let remote_identity_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {identity_from} \
             WHERE vec_id_scope = 'global' \
               AND status = 'ready' \
               AND node_count = 2 \
               AND min_node_id = 0 \
               AND max_node_id = 2"
        ))
        .expect("remote boundary identity query should succeed")
        .expect("remote boundary identity count should exist");

        let persist_result = Spi::get_one::<bool>(
            "SELECT ec_spire_persist_remote_epoch_manifest(\
             'ec_spire_boundary_manifest_freshness_idx'::regclass)",
        )
        .expect("remote manifest persist should succeed")
        .expect("remote manifest persist result should exist");
        let ready_status =
            Spi::get_one::<String>(&format!("SELECT freshness_status {freshness_from}"))
                .expect("ready freshness status query should succeed")
                .expect("ready freshness status should exist");
        let ready_action = Spi::get_one::<String>(&format!("SELECT next_action {freshness_from}"))
            .expect("ready freshness action query should succeed")
            .expect("ready freshness action should exist");
        let ready_entry_matches =
            Spi::get_one::<bool>(&format!("SELECT persisted_entry_matches {freshness_from}"))
                .expect("ready freshness match query should succeed")
                .expect("ready freshness match should exist");

        Spi::run(&format!(
            "UPDATE ec_spire_remote_epoch_manifest_entry \
                SET last_served_epoch = last_served_epoch - 1 \
              WHERE coordinator_index_oid = '{}'::oid \
                AND active_epoch = {active_epoch} \
                AND node_id = 2",
            u32::from(index_oid)
        ))
        .expect("manifest entry drift update should succeed");
        let stale_status =
            Spi::get_one::<String>(&format!("SELECT freshness_status {freshness_from}"))
                .expect("stale freshness status query should succeed")
                .expect("stale freshness status should exist");
        let stale_action = Spi::get_one::<String>(&format!("SELECT next_action {freshness_from}"))
            .expect("stale freshness action query should succeed")
            .expect("stale freshness action should exist");
        let stale_entry_matches =
            Spi::get_one::<bool>(&format!("SELECT persisted_entry_matches {freshness_from}"))
                .expect("stale freshness match query should succeed")
                .expect("stale freshness match should exist");

        assert!(register_result);
        assert_eq!(
            pre_persist_status,
            "requires_remote_epoch_manifest_persistence"
        );
        assert_eq!(pre_persist_action, "persist_remote_epoch_manifest");
        assert!(remote_identity_count > 0);
        assert!(persist_result);
        assert_eq!(ready_status, "ready");
        assert_eq!(ready_action, "none");
        assert!(ready_entry_matches);
        assert_eq!(stale_status, "stale_remote_epoch_manifest");
        assert_eq!(stale_action, "refresh_remote_epoch_manifest");
        assert!(!stale_entry_matches);
    }

    #[pg_test]
    fn test_ec_spire_remote_epoch_manifest_libpq_executor_loopback() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_MANIFEST_LOOPBACK",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_remote_manifest_executor_remote_sql; \
                 CREATE TABLE ec_spire_remote_manifest_executor_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_remote_manifest_executor_remote_sql (id, embedding) VALUES \
                     (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_remote_manifest_executor_remote_sql_idx \
                     ON ec_spire_remote_manifest_executor_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
            )
            .expect("loopback remote manifest fixture should be created");

        Spi::run(
            "CREATE TABLE ec_spire_remote_manifest_executor_coord_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_manifest_executor_coord_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_manifest_executor_coord_sql_idx \
             ON ec_spire_remote_manifest_executor_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_manifest_executor_coord_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_manifest_executor_coord_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_manifest_executor_coord_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 12, 'spire/remote/manifest/loopback', decode('05', 'hex'), \
                     'ec_spire_remote_manifest_executor_remote_sql_idx', 'active', \
                     {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");
        let persist_result = Spi::get_one::<bool>(
            "SELECT ec_spire_persist_remote_epoch_manifest(\
             'ec_spire_remote_manifest_executor_coord_sql_idx'::regclass)",
        )
        .expect("remote manifest persist should succeed")
        .expect("remote manifest persist result should exist");

        let executor_from = "FROM ec_spire_remote_epoch_manifest_libpq_executor_results(\
             'ec_spire_remote_manifest_executor_coord_sql_idx'::regclass)";
        let connection_attempted =
            Spi::get_one::<bool>(&format!("SELECT connection_attempted {executor_from}"))
                .expect("manifest executor connection attempted query should succeed")
                .expect("manifest executor connection attempted should exist");
        let connection_status =
            Spi::get_one::<String>(&format!("SELECT connection_status {executor_from}"))
                .expect("manifest executor connection status query should succeed")
                .expect("manifest executor connection status should exist");
        let validated_entry_count =
            Spi::get_one::<i64>(&format!("SELECT validated_entry_count {executor_from}"))
                .expect("manifest executor validated entry query should succeed")
                .expect("manifest executor validated entry should exist");
        let validation_status =
            Spi::get_one::<String>(&format!("SELECT validation_result_status {executor_from}"))
                .expect("manifest executor validation status query should succeed")
                .expect("manifest executor validation status should exist");
        let conninfo_lookup_kind =
            Spi::get_one::<String>(&format!("SELECT conninfo_lookup_kind {executor_from}"))
                .expect("manifest executor lookup kind query should succeed")
                .expect("manifest executor lookup kind should exist");
        let next_step =
            Spi::get_one::<String>(&format!("SELECT next_executor_step {executor_from}"))
                .expect("manifest executor next step query should succeed")
                .expect("manifest executor next step should exist");
        let status = Spi::get_one::<String>(&format!("SELECT status {executor_from}"))
            .expect("manifest executor status query should succeed")
            .expect("manifest executor status should exist");
        let remote_index_oid = loopback_client
            .query_one(
                "SELECT 'ec_spire_remote_manifest_executor_remote_sql_idx'::regclass::oid",
                &[],
            )
            .expect("remote index oid query should succeed")
            .try_get::<_, u32>(0)
            .expect("remote index oid should decode");
        let applied_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_remote_epoch_manifest_applied \
                  WHERE remote_index_oid = $1::oid AND active_epoch = $2::bigint",
                &[&remote_index_oid, &active_epoch],
            )
            .expect("remote applied manifest count query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote applied manifest count should decode");
        let applied_entry_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM ec_spire_remote_epoch_manifest_applied_entry \
                  WHERE remote_index_oid = $1::oid AND active_epoch = $2::bigint",
                &[&remote_index_oid, &active_epoch],
            )
            .expect("remote applied manifest entry count query should succeed")
            .try_get::<_, i64>(0)
            .expect("remote applied manifest entry count should decode");

        assert!(register_result);
        assert!(persist_result);
        assert!(connection_attempted);
        assert_eq!(connection_status, "libpq_connection_opened");
        assert_eq!(validated_entry_count, 1);
        assert_eq!(validation_status, "ready");
        assert_eq!(conninfo_lookup_kind, "secret_provider");
        assert_eq!(next_step, "none");
        assert_eq!(status, "ready");
        assert_eq!(applied_count, 1);
        assert_eq!(applied_entry_count, 1);
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_spire_persist_remote_epoch_manifest cannot persist remote epoch manifest"
    )]
    fn test_ec_spire_remote_epoch_manifest_persist_blocked() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_manifest_blocked_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_manifest_blocked_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_manifest_blocked_sql_idx \
             ON ec_spire_remote_manifest_blocked_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_manifest_blocked_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_manifest_blocked_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let _ = Spi::get_one::<bool>(
            "SELECT ec_spire_persist_remote_epoch_manifest(\
             'ec_spire_remote_manifest_blocked_sql_idx'::regclass)",
        );
    }

    #[pg_test]
    fn test_ec_spire_remote_epoch_manifest_catalog_summary_missing() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_manifest_summary_missing_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_manifest_summary_missing_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_manifest_summary_missing_sql_idx \
             ON ec_spire_remote_manifest_summary_missing_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_manifest_summary_missing_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_manifest_summary_missing_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pid = Spi::get_one::<i64>(
            "SELECT min(leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_manifest_summary_missing_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid should exist");

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pid as u64, 2) };
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 12, 'spire/remote/summary-missing', decode('05', 'hex'), \
                     'remote_spire_idx', 'active', {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("remote descriptor registration should succeed")
        .expect("remote descriptor registration result should exist");

        let summary_from = "FROM ec_spire_remote_epoch_manifest_catalog_summary(\
             'ec_spire_remote_manifest_summary_missing_sql_idx'::regclass)";
        let publication_summary_from = "FROM ec_spire_remote_epoch_manifest_publication_summary(\
             'ec_spire_remote_manifest_summary_missing_sql_idx'::regclass)";
        let publication_result_from =
            "FROM ec_spire_remote_epoch_manifest_publication_result_summary(\
             'ec_spire_remote_manifest_summary_missing_sql_idx'::regclass)";
        let manifest_decision =
            Spi::get_one::<String>(&format!("SELECT current_manifest_decision {summary_from}"))
                .expect("manifest summary decision query should succeed")
                .expect("manifest summary decision should exist");
        let catalog_status =
            Spi::get_one::<String>(&format!("SELECT catalog_status {summary_from}"))
                .expect("manifest summary status query should succeed")
                .expect("manifest summary status should exist");
        let persisted_manifest_count =
            Spi::get_one::<i64>(&format!("SELECT persisted_manifest_count {summary_from}"))
                .expect("manifest summary persisted count query should succeed")
                .expect("manifest summary persisted count should exist");
        let persisted_entry_count =
            Spi::get_one::<i64>(&format!("SELECT persisted_entry_count {summary_from}"))
                .expect("manifest summary entry count query should succeed")
                .expect("manifest summary entry count should exist");
        let persisted_entry_mismatch_count = Spi::get_one::<i64>(&format!(
            "SELECT persisted_entry_mismatch_count {summary_from}"
        ))
        .expect("manifest summary mismatch count query should succeed")
        .expect("manifest summary mismatch count should exist");
        let publication_decision = Spi::get_one::<String>(&format!(
            "SELECT publication_decision {publication_summary_from}"
        ))
        .expect("publication summary decision query should succeed")
        .expect("publication summary decision should exist");
        let persistence_required_count = Spi::get_one::<i64>(&format!(
            "SELECT persistence_required_count {publication_summary_from}"
        ))
        .expect("publication summary persistence count query should succeed")
        .expect("publication summary persistence count should exist");
        let publication_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {publication_summary_from}"))
                .expect("publication summary blocker query should succeed")
                .expect("publication summary blocker should exist");
        let publication_result_source =
            Spi::get_one::<String>(&format!("SELECT result_source {publication_result_from}"))
                .expect("publication result source query should succeed")
                .expect("publication result source should exist");
        let publication_result_receive_count = Spi::get_one::<i64>(&format!(
            "SELECT libpq_receive_count {publication_result_from}"
        ))
        .expect("publication result receive count query should succeed")
        .expect("publication result receive count should exist");
        let publication_result_status =
            Spi::get_one::<String>(&format!("SELECT status {publication_result_from}"))
                .expect("publication result status query should succeed")
                .expect("publication result status should exist");
        let publication_result_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {publication_result_from}"))
                .expect("publication result blocker query should succeed")
                .expect("publication result blocker should exist");

        assert!(register_result);
        assert_eq!(manifest_decision, "emit_distributed_epoch_manifest");
        assert_eq!(catalog_status, "requires_remote_epoch_manifest_persistence");
        assert_eq!(persisted_manifest_count, 0);
        assert_eq!(persisted_entry_count, 0);
        assert_eq!(persisted_entry_mismatch_count, 1);
        assert_eq!(publication_decision, "persist_remote_epoch_manifest");
        assert_eq!(persistence_required_count, 1);
        assert_eq!(
            publication_next_blocker,
            "remote_epoch_manifest_persistence"
        );
        assert_eq!(publication_result_source, "blocked");
        assert_eq!(publication_result_receive_count, 0);
        assert_eq!(
            publication_result_status,
            "requires_remote_epoch_manifest_persistence"
        );
        assert_eq!(
            publication_result_next_blocker,
            "remote_epoch_manifest_persistence"
        );
    }

    #[pg_test]
    fn test_ec_spire_remote_phase7_policy_contracts() {
        let degradation_from = "FROM ec_spire_remote_degradation_policy_contract()";
        let publication_from = "FROM ec_spire_remote_epoch_manifest_publication_contract()";
        let publication_result_from =
            "FROM ec_spire_remote_epoch_manifest_publication_result_contract()";
        let manifest_parameter_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_parameter_contract()";
        let manifest_result_from = "FROM ec_spire_remote_epoch_manifest_libpq_result_contract()";
        let manifest_executor_step_from =
            "FROM ec_spire_remote_epoch_manifest_libpq_executor_step_contract()";
        let operator_entrypoint_from = "FROM ec_spire_remote_operator_entrypoint_contract()";
        let libpq_lifecycle_from = "FROM ec_spire_remote_libpq_connection_lifecycle_contract()";
        let secret_resolution_from = "FROM ec_spire_remote_conninfo_secret_resolution_contract()";
        let catalog_lifecycle_from = "FROM ec_spire_remote_catalog_lifecycle_contract()";
        let search_result_from = "FROM ec_spire_remote_search_coordinator_result_contract()";
        let merge_order_from = "FROM ec_spire_remote_search_merge_order_contract()";
        let identity_contract_from = "FROM ec_spire_remote_search_vector_identity_contract()";
        let degradation_count = Spi::get_one::<i64>(&format!("SELECT count(*) {degradation_from}"))
            .expect("degradation contract count query should succeed")
            .expect("degradation contract count should exist");
        let degraded_unavailable_action = Spi::get_one::<String>(&format!(
            "SELECT search_action {degradation_from} \
             WHERE consistency_mode = 'degraded' AND placement_state = 'unavailable'"
        ))
        .expect("degraded unavailable contract query should succeed")
        .expect("degraded unavailable contract should exist");
        let strict_unavailable_action = Spi::get_one::<String>(&format!(
            "SELECT search_action {degradation_from} \
             WHERE consistency_mode = 'strict' AND placement_state = 'unavailable'"
        ))
        .expect("strict unavailable contract query should succeed")
        .expect("strict unavailable contract should exist");
        let stale_degraded_status = Spi::get_one::<String>(&format!(
            "SELECT status {degradation_from} \
             WHERE consistency_mode = 'degraded' AND placement_state = 'stale'"
        ))
        .expect("degraded stale contract query should succeed")
        .expect("degraded stale contract should exist");
        let merge_order_count = Spi::get_one::<i64>(&format!("SELECT count(*) {merge_order_from}"))
            .expect("merge order contract count query should succeed")
            .expect("merge order contract count should exist");
        let first_order_key = Spi::get_one::<String>(&format!(
            "SELECT order_key {merge_order_from} WHERE order_ordinal = 1"
        ))
        .expect("merge first order query should succeed")
        .expect("merge first order should exist");
        let assignment_direction = Spi::get_one::<String>(&format!(
            "SELECT direction {merge_order_from} WHERE order_key = 'assignment_role'"
        ))
        .expect("merge assignment direction query should succeed")
        .expect("merge assignment direction should exist");
        let dedupe_order = Spi::get_one::<String>(&format!(
            "SELECT string_agg(order_key, ',' ORDER BY order_ordinal) {merge_order_from}"
        ))
        .expect("merge order aggregate query should succeed")
        .expect("merge order aggregate should exist");
        let remote_dedupe_key = Spi::get_one::<String>(&format!(
            "SELECT contract_value {identity_contract_from} \
             WHERE contract_item = 'remote_merge_dedupe_key'"
        ))
        .expect("remote vector identity dedupe key query should succeed")
        .expect("remote vector identity dedupe key should exist");
        let publication_step_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {publication_from}"))
                .expect("manifest publication contract count query should succeed")
                .expect("manifest publication contract count should exist");
        let persistence_action = Spi::get_one::<String>(&format!(
            "SELECT publication_action {publication_from} \
             WHERE failure_status = 'requires_remote_epoch_manifest_persistence'"
        ))
        .expect("manifest publication persistence query should succeed")
        .expect("manifest publication persistence action should exist");
        let stale_action = Spi::get_one::<String>(&format!(
            "SELECT publication_action {publication_from} \
             WHERE failure_status = 'stale_remote_epoch_manifest'"
        ))
        .expect("manifest publication stale query should succeed")
        .expect("manifest publication stale action should exist");
        let transport_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {publication_from} \
             WHERE prerequisite = 'remote_epoch_manifest_transport'"
        ))
        .expect("manifest publication transport query should succeed")
        .expect("manifest publication transport validator should exist");
        let publication_result_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {publication_result_from}"))
                .expect("manifest publication result contract count query should succeed")
                .expect("manifest publication result contract count should exist");
        let pending_result_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {publication_result_from} \
             WHERE result_source = 'pending_libpq_executor'"
        ))
        .expect("manifest publication pending result query should succeed")
        .expect("manifest publication pending result validator should exist");
        let validation_result_recommendation = Spi::get_one::<String>(&format!(
            "SELECT recommendation {publication_result_from} \
             WHERE result_source = 'remote_manifest_validation_result'"
        ))
        .expect("manifest publication validation result query should succeed")
        .expect("manifest publication validation result recommendation should exist");
        let manifest_parameter_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_parameter_from}"))
                .expect("manifest parameter contract count query should succeed")
                .expect("manifest parameter contract count should exist");
        let manifest_payload_type = Spi::get_one::<String>(&format!(
            "SELECT pg_type {manifest_parameter_from} \
             WHERE parameter_name = 'manifest_payload'"
        ))
        .expect("manifest payload parameter query should succeed")
        .expect("manifest payload parameter should exist");
        let manifest_result_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_result_from}"))
                .expect("manifest result contract count query should succeed")
                .expect("manifest result contract count should exist");
        let manifest_status_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {manifest_result_from} WHERE column_name = 'status'"
        ))
        .expect("manifest result status query should succeed")
        .expect("manifest result status should exist");
        let manifest_executor_step_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {manifest_executor_step_from}"))
                .expect("manifest executor step count query should succeed")
                .expect("manifest executor step count should exist");
        let manifest_send_input = Spi::get_one::<String>(&format!(
            "SELECT input_contract {manifest_executor_step_from} \
             WHERE step_name = 'send_manifest_request'"
        ))
        .expect("manifest executor send input query should succeed")
        .expect("manifest executor send input should exist");
        let search_result_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {search_result_from}"))
                .expect("search result contract count query should succeed")
                .expect("search result contract count should exist");
        let search_blocked_validator = Spi::get_one::<String>(&format!(
            "SELECT validator {search_result_from} WHERE result_source = 'blocked'"
        ))
        .expect("search blocked result query should succeed")
        .expect("search blocked result should exist");
        let operator_entrypoint_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {operator_entrypoint_from}"))
                .expect("operator entrypoint count query should succeed")
                .expect("operator entrypoint count should exist");
        let operator_entrypoint_reachable_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_operator_entrypoint_contract() contract \
              WHERE EXISTS ( \
                    SELECT 1 \
                      FROM pg_proc proc \
                     WHERE proc.proname = contract.entrypoint_name)"
        ))
        .expect("operator entrypoint reachability query should succeed")
        .expect("operator entrypoint reachability count should exist");
        let search_gate_next_action = Spi::get_one::<String>(&format!(
            "SELECT next_action {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_coordinator_gate_summary'"
        ))
        .expect("operator search gate entrypoint query should succeed")
        .expect("operator search gate entrypoint should exist");
        let publication_result_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_epoch_manifest_publication_result_summary'"
        ))
        .expect("operator publication result entrypoint query should succeed")
        .expect("operator publication result entrypoint should exist");
        let search_secret_next_action = Spi::get_one::<String>(&format!(
            "SELECT next_action {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_libpq_secret_summary'"
        ))
        .expect("operator search secret entrypoint query should succeed")
        .expect("operator search secret entrypoint should exist");
        let single_secret_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_conninfo_secret_resolution_status'"
        ))
        .expect("operator single secret entrypoint query should succeed")
        .expect("operator single secret entrypoint should exist");
        let production_state_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_production_executor_state_summary'"
        ))
        .expect("operator production state entrypoint query should succeed")
        .expect("operator production state entrypoint should exist");
        let pipeline_steps_action = Spi::get_one::<String>(&format!(
            "SELECT next_action {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_pipeline_steps'"
        ))
        .expect("operator pipeline steps entrypoint query should succeed")
        .expect("operator pipeline steps entrypoint should exist");
        let pipeline_steps_live_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_pipeline_steps_live'"
        ))
        .expect("operator live pipeline steps entrypoint query should succeed")
        .expect("operator live pipeline steps entrypoint should exist");
        let receive_attempts_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_libpq_executor_receive_attempts'"
        ))
        .expect("operator receive attempts entrypoint query should succeed")
        .expect("operator receive attempts entrypoint should exist");
        let budget_entrypoint_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_libpq_executor_budget_summary'"
        ))
        .expect("operator budget entrypoint query should succeed")
        .expect("operator budget entrypoint should exist");
        let stage_e_fault_matrix_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_stage_e_fault_matrix'"
        ))
        .expect("operator Stage E fault matrix entrypoint query should succeed")
        .expect("operator Stage E fault matrix entrypoint should exist");
        let operator_diagnostics_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_operator_diagnostics'"
        ))
        .expect("operator diagnostics entrypoint query should succeed")
        .expect("operator diagnostics entrypoint should exist");
        let stage_e_lifecycle_matrix_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_search_stage_e_lifecycle_matrix'"
        ))
        .expect("operator Stage E lifecycle matrix entrypoint query should succeed")
        .expect("operator Stage E lifecycle matrix entrypoint should exist");
        let manifest_freshness_use = Spi::get_one::<String>(&format!(
            "SELECT operator_use {operator_entrypoint_from} \
             WHERE entrypoint_name = 'ec_spire_remote_epoch_manifest_freshness'"
        ))
        .expect("operator manifest freshness entrypoint query should succeed")
        .expect("operator manifest freshness entrypoint should exist");
        let libpq_lifecycle_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {libpq_lifecycle_from}"))
                .expect("libpq lifecycle count query should succeed")
                .expect("libpq lifecycle count should exist");
        let search_connection_policy = Spi::get_one::<String>(&format!(
            "SELECT connection_lifecycle_policy {libpq_lifecycle_from} \
             WHERE surface = 'ec_spire_remote_search_libpq_executor'"
        ))
        .expect("search lifecycle policy query should succeed")
        .expect("search lifecycle policy should exist");
        let search_secret_policy = Spi::get_one::<String>(&format!(
            "SELECT secret_resolution_policy {libpq_lifecycle_from} \
             WHERE surface = 'ec_spire_remote_search_libpq_executor'"
        ))
        .expect("search lifecycle secret policy query should succeed")
        .expect("search lifecycle secret policy should exist");
        let manifest_conninfo_policy = Spi::get_one::<String>(&format!(
            "SELECT conninfo_exposure_policy {libpq_lifecycle_from} \
             WHERE surface = 'ec_spire_remote_epoch_manifest_publication_libpq_executor'"
        ))
        .expect("manifest lifecycle conninfo policy query should succeed")
        .expect("manifest lifecycle conninfo policy should exist");
        let secret_provider_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {secret_resolution_from}"))
                .expect("secret provider count query should succeed")
                .expect("secret provider count should exist");
        let selected_secret_provider = Spi::get_one::<String>(&format!(
            "SELECT provider_policy {secret_resolution_from} \
             WHERE provider_status = 'selected_v1'"
        ))
        .expect("selected secret provider query should succeed")
        .expect("selected secret provider should exist");
        let selected_raw_conninfo_allowed = Spi::get_one::<bool>(&format!(
            "SELECT raw_conninfo_allowed {secret_resolution_from} \
             WHERE provider_status = 'selected_v1'"
        ))
        .expect("selected raw conninfo query should succeed")
        .expect("selected raw conninfo should exist");
        let rejected_provider_storage = Spi::get_one::<String>(&format!(
            "SELECT sql_storage_policy {secret_resolution_from} \
             WHERE provider_policy = 'in_extension_conninfo_table'"
        ))
        .expect("rejected secret provider storage query should succeed")
        .expect("rejected secret provider storage should exist");
        let catalog_lifecycle_count =
            Spi::get_one::<i64>(&format!("SELECT count(*) {catalog_lifecycle_from}"))
                .expect("catalog lifecycle count query should succeed")
                .expect("catalog lifecycle count should exist");
        let dump_restore_status = Spi::get_one::<String>(&format!(
            "SELECT status {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'pg_dump_restore'"
        ))
        .expect("dump restore lifecycle query should succeed")
        .expect("dump restore lifecycle should exist");
        let drop_index_cleanup_surface = Spi::get_one::<String>(&format!(
            "SELECT cleanup_surface {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'drop_index'"
        ))
        .expect("drop index lifecycle query should succeed")
        .expect("drop index lifecycle should exist");
        let drop_index_migration_surface = Spi::get_one::<String>(&format!(
            "SELECT migration_surface {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'drop_index'"
        ))
        .expect("drop index migration lifecycle query should succeed")
        .expect("drop index migration lifecycle should exist");
        let drop_index_status = Spi::get_one::<String>(&format!(
            "SELECT status {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'drop_index'"
        ))
        .expect("drop index status lifecycle query should succeed")
        .expect("drop index status lifecycle should exist");
        let basebackup_status = Spi::get_one::<String>(&format!(
            "SELECT status {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'basebackup_wal_replay'"
        ))
        .expect("basebackup lifecycle query should succeed")
        .expect("basebackup lifecycle should exist");
        let upgrade_migration_surface = Spi::get_one::<String>(&format!(
            "SELECT migration_surface {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'extension_upgrade_0_1_0_to_0_1_1'"
        ))
        .expect("upgrade lifecycle query should succeed")
        .expect("upgrade lifecycle should exist");
        let upgrade_status = Spi::get_one::<String>(&format!(
            "SELECT status {catalog_lifecycle_from} \
             WHERE lifecycle_event = 'extension_upgrade_0_1_0_to_0_1_1'"
        ))
        .expect("upgrade lifecycle status query should succeed")
        .expect("upgrade lifecycle status should exist");

        assert_eq!(degradation_count, 8);
        assert_eq!(degraded_unavailable_action, "skip_and_report");
        assert_eq!(strict_unavailable_action, "fail_closed");
        assert_eq!(stale_degraded_status, "requires_fresh_epoch");
        assert_eq!(merge_order_count, 8);
        assert_eq!(first_order_key, "score");
        assert_eq!(assignment_direction, "primary_before_boundary_replica");
        assert_eq!(
            dedupe_order,
            "score,assignment_role,served_epoch,node_id,pid,object_version,row_index,row_locator"
        );
        assert_eq!(publication_step_count, 5);
        assert_eq!(persistence_action, "persist_remote_epoch_manifest");
        assert_eq!(stale_action, "refresh_remote_epoch_manifest");
        assert_eq!(
            transport_validator,
            "future_executor_must_use_libpq_pipeline"
        );
        assert_eq!(publication_result_count, 4);
        assert_eq!(pending_result_validator, "must_name_next_executor_step");
        assert!(validation_result_recommendation.contains("remote apply executor"));
        assert_eq!(manifest_parameter_count, 3);
        assert_eq!(manifest_payload_type, "jsonb");
        assert_eq!(manifest_result_count, 3);
        assert_eq!(manifest_status_validator, "must_report_ready_or_blocker");
        assert_eq!(manifest_executor_step_count, 5);
        assert_eq!(
            manifest_send_input,
            "ec_spire_remote_epoch_manifest_libpq_parameter_contract"
        );
        assert_eq!(search_result_count, 4);
        assert_eq!(search_blocked_validator, "must_preserve_next_blocker");
        assert_eq!(
            remote_dedupe_key,
            "global_vec_id_or_node_scoped_local_vec_id"
        );
        assert_eq!(operator_entrypoint_count, 23);
        assert_eq!(operator_entrypoint_reachable_count, 23);
        assert_eq!(
            search_gate_next_action,
            "resolve_reported_blocker_before_expect_result_rows"
        );
        assert_eq!(publication_result_use, "manifest_publication_result");
        assert_eq!(
            search_secret_next_action,
            "resolve_missing_conninfo_secrets_before_opening_libpq_connections"
        );
        assert_eq!(single_secret_use, "single_conninfo_secret_probe");
        assert_eq!(production_state_use, "production_executor_dry_state");
        assert_eq!(
            pipeline_steps_action,
            "inspect_first_non_ready_step_before_live_probe_or_narrow_surfaces"
        );
        assert_eq!(
            pipeline_steps_live_use,
            "consolidated_remote_pipeline_steps_live_probe"
        );
        assert_eq!(
            receive_attempts_use,
            "per_node_remote_receive_attempt_diagnostics"
        );
        assert_eq!(budget_entrypoint_use, "remote_executor_resource_governance");
        assert_eq!(
            stage_e_fault_matrix_use,
            "local_multi_instance_fault_fixture_contract"
        );
        assert_eq!(
            operator_diagnostics_use,
            "packet_friendly_production_readiness_rollup"
        );
        assert_eq!(
            stage_e_lifecycle_matrix_use,
            "local_multi_instance_lifecycle_fixture_contract"
        );
        assert_eq!(
            manifest_freshness_use,
            "stage_e_manifest_freshness_assertion"
        );
        assert_eq!(libpq_lifecycle_count, 2);
        assert_eq!(search_connection_policy, "per_query");
        assert_eq!(
            search_secret_policy,
            "conninfo_secret_name_resolved_by_executor"
        );
        assert_eq!(manifest_conninfo_policy, "never_expose_raw_conninfo_in_sql");
        assert_eq!(secret_provider_count, 3);
        assert_eq!(
            selected_secret_provider,
            "external_executor_secret_provider"
        );
        assert!(!selected_raw_conninfo_allowed);
        assert_eq!(
            rejected_provider_storage,
            "never_store_raw_conninfo_in_extension_catalog"
        );
        assert_eq!(catalog_lifecycle_count, 4);
        assert_eq!(dump_restore_status, "requires_operator_reregistration");
        assert_eq!(
            drop_index_cleanup_surface,
            "ec_spire_remote_catalog_index_cleanup,ec_spire_remote_catalog_orphan_cleanup"
        );
        assert_eq!(
            drop_index_migration_surface,
            "ec_spire_remote_catalog_drop_index_cleanup"
        );
        assert_eq!(drop_index_status, "automatic_event_trigger_cleanup");
        assert_eq!(basebackup_status, "supported");
        assert_eq!(upgrade_migration_surface, "ecaz--0.1.0--0.1.1.sql");
        assert_eq!(upgrade_status, "supported_after_upgrade_script");
    }

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
    }