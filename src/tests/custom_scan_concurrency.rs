    #[pg_test]
    fn test_ec_spire_customscan_idle_transaction_timeout_cursor_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_IDLE_TIMEOUT",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");

        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_customscan_idle_timeout_remote_sql; \
                 CREATE TABLE ec_spire_customscan_idle_timeout_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector); \
                 INSERT INTO ec_spire_customscan_idle_timeout_remote_sql (id, title, embedding) VALUES \
                     (101, 'remote idle alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (102, 'remote idle beta', encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42)); \
                 CREATE INDEX ec_spire_customscan_idle_timeout_remote_idx \
                     ON ec_spire_customscan_idle_timeout_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
            )
            .expect("loopback remote idle-timeout fixture should be created");
        let remote_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_customscan_idle_timeout_remote_idx",
        );

        Spi::run(
            "CREATE TABLE ec_spire_customscan_idle_timeout_coord_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("coordinator idle-timeout table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_customscan_idle_timeout_coord_sql (id, title, embedding) VALUES \
             (1, 'coordinator idle alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coordinator idle beta', encode_to_ecvector(ARRAY[0.5, 0.5], 4, 42))",
        )
        .expect("coordinator idle-timeout insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_customscan_idle_timeout_coord_idx \
             ON ec_spire_customscan_idle_timeout_coord_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
        )
        .expect("coordinator idle-timeout index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_idle_timeout_coord_idx'::regclass::oid",
        )
        .expect("coordinator idle-timeout index oid query should succeed")
        .expect("coordinator idle-timeout index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_customscan_idle_timeout_coord_idx'::regclass)",
        )
        .expect("coordinator idle-timeout active epoch query should succeed")
        .expect("coordinator idle-timeout active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_customscan_idle_timeout_coord_idx'::regclass)",
        )
        .expect("coordinator idle-timeout leaf pid query should succeed")
        .expect("coordinator idle-timeout leaf pids should exist");

        unsafe {
            for pid in &coord_leaf_pids {
                am::debug_spire_rewrite_placement_node(index_oid, *pid as u64, 2);
            }
        }
        let register_result = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, 108, 'spire/remote/customscan/idle_timeout', \
                     decode('{remote_identity_hex}', 'hex'), \
                     'ec_spire_customscan_idle_timeout_remote_idx', 'active', {active_epoch}, \
                     {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("idle-timeout remote descriptor registration should succeed")
        .expect("idle-timeout remote descriptor registration result should exist");
        assert!(register_result);

        let mut cursor_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("cursor client connection should succeed");
        cursor_client
            .batch_execute("SET enable_seqscan = off; SET enable_indexscan = off")
            .expect("cursor client planner GUCs should be set");
        let plan = cursor_client
            .query(
                "EXPLAIN (COSTS OFF) \
                 SELECT id, title \
                   FROM ec_spire_customscan_idle_timeout_coord_sql \
                  ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                  LIMIT 2",
                &[],
            )
            .expect("idle-timeout CustomScan explain should succeed")
            .into_iter()
            .map(|row| {
                row.try_get::<_, String>(0)
                    .expect("idle-timeout explain row should decode")
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            plan.contains("Custom Scan (EcSpireDistributedScan)"),
            "expected idle-timeout cursor fixture to use EcSpireDistributedScan:\n{plan}"
        );
        cursor_client
            .batch_execute(
                "BEGIN; \
                 SET LOCAL idle_in_transaction_session_timeout = '100ms'; \
                 SET LOCAL enable_seqscan = off; \
                 SET LOCAL enable_indexscan = off; \
                 DECLARE ec_spire_idle_timeout_cursor CURSOR FOR \
                     SELECT id, title \
                       FROM ec_spire_customscan_idle_timeout_coord_sql \
                      ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[], id \
                      LIMIT 2",
            )
            .expect("cursor over CustomScan should open before idling");
        std::thread::sleep(std::time::Duration::from_millis(350));

        let disconnect_error = cursor_client
            .batch_execute("SELECT 1")
            .expect_err("idle-in-transaction timeout should disconnect the cursor backend");
        let disconnect_message = disconnect_error.to_string();
        assert!(
            disconnect_message.contains("closed")
                || disconnect_message.contains("terminating connection")
                || disconnect_message.contains("connection"),
            "{disconnect_message}"
        );

        let prepared_prefix = format!("ec_spire_insert_{}_%", u32::from(index_oid));
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint FROM pg_prepared_xacts WHERE gid LIKE $1",
                &[&prepared_prefix],
            )
            .expect("prepared xact cleanup query should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared xact cleanup count should decode");
        assert_eq!(prepared_count, 0);
    }
