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
            descriptor_state_check_values(include_str!("../../../sql/bootstrap.sql"));
        let upgrade_states =
            descriptor_state_check_values(include_str!("../../../ecaz--0.1.0--0.1.1.sql"));
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
