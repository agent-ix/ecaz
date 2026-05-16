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

    include!("contracts_libpq.rs");
